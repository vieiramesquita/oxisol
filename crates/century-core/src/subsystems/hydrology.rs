//! Monthly hydrology, Linacre (1977) PET, astronomical daylength, snowpack,
//! multi-layer soil water redistribution, transpiration, and solute leaching.
//!
//! Specification: `docs/spec/02_hydrology_and_soil_water.md`.

use crate::config::{CompatibilityProfile, ElementMode};
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{CellDerivedParams, FixParams, MAX_LAYERS};
use crate::state::{CellState, MonthlyWeather};

/// Mid-month day-of-year for months 1..=12.
const MID_MONTH_DOY: [f64; 12] = [
    15.0, 45.0, 74.0, 105.0, 135.0, 166.0, 196.0, 227.0, 258.0, 288.0, 319.0, 349.0,
];

/// Hydrological outputs passed to downstream decomposition and vegetation modules.
#[derive(Debug, Clone, Copy)]
pub struct HydrologyStepResult<F: Float> {
    /// Monthly potential evapotranspiration (cm).
    pub pet: F,
    /// Monthly actual evapotranspiration (cm).
    pub aet: F,
    /// Actual transpiration (cm).
    pub tran: F,
    /// Potential transpiration (cm).
    pub ptran: F,
    /// Effective liquid water input (rain + snowmelt + irrigation) before interception/evap (cm).
    pub liquid_input: F,
    /// Water flux draining past rooting/leaching boundary (cm).
    pub leach_flux: F,
    /// Water flux exiting the 30 cm layer (layer index 1) for organic matter leaching (cm).
    pub amov_30cm: F,
    /// Total runoff + baseflow + stormflow (cm).
    pub stream: F,
    /// Moisture ratio `(avh2o_prev[2] + liquid_input) / pet` used by `defac` (`idef = 2`) and `anerb`.
    pub rel_water_surface: F,
    /// Root-zone moisture ratio `(avh2o_prev[0] + liquid_input) / pet` used by plant `pprdwc`.
    pub rel_water_root: F,
    /// Top-layer relative water content `(asmos[0] - wp) / (fc - wp)` used when `idef = 1`.
    pub top_rwcf: F,
}

/// Astronomical daylength in hours for a given month (`1..=12`) and latitude (degrees).
///
/// Honors `CompatibilityProfile` (Spec 07 Delta #1).
#[inline]
pub fn daylength_hours<F: Float>(month: u8, lat_deg: F, profile: CompatibilityProfile) -> F {
    let f = F::from_f64;
    let m_idx = (month.clamp(1, 12) - 1) as usize;
    let doy = f(MID_MONTH_DOY[m_idx]);
    let lat_rad = lat_deg * F::PI / f(180.0);
    let adelt = f(0.4014) * (f(2.0) * F::PI * (doy - f(77.0)) / f(365.0)).sin();

    let decl_term = match profile {
        CompatibilityProfile::ScientificClean => adelt.tan(),
        // Documented V4.5 legacy behavior: omitted tan() on declination angle.
        CompatibilityProfile::VemapV45Parity => adelt,
    };

    let cos_h0 = (-lat_rad.tan() * decl_term).clamp_val(f(-1.0), f(1.0));
    let half_day_rad = cos_h0.acos();
    (f(24.0) / F::PI) * half_day_rad
}

/// Linacre (1977) monthly Potential Evapotranspiration (cm/month).
///
/// Linacre's formula already incorporates site latitude via `100 - |lat|`; no artificial
/// daylength multiplier is applied. In `VemapV45Parity`, uses sea-level Linacre (`elev = 0`)
/// matching CENTURY V4.0/V4.5 `pevap`.
pub fn linacre_pet<F: Float>(
    _month: u8,
    weather: &MonthlyWeather<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    profile: CompatibilityProfile,
) -> F {
    let f = F::from_f64;
    let tmean = (weather.tmin_c + weather.tmax_c) * f(0.5);
    let trange = (weather.tmax_c - weather.tmin_c).max(F::ZERO);
    let elev = match profile {
        CompatibilityProfile::VemapV45Parity => F::ZERO,
        CompatibilityProfile::ScientificClean => derived.elevation.clamp_val(F::ZERO, f(4000.0)),
    };
    let tm = tmean + f(0.006) * elev;

    let td_dep =
        f(0.0023) * elev + f(0.37) * tmean + f(0.53) * trange + f(0.35) * derived.trange_annual
            - f(10.9);

    let lat_denom = (f(100.0) - derived.lat.abs()).max(f(10.0));
    let t_denom = (f(80.0) - tmean).max(f(10.0));
    let pet_daily_mm = ((f(700.0) * tm / lat_denom) + f(15.0) * td_dep) / t_denom;

    // Convert mm/day -> cm/month (30 days / 10 mm/cm = 3.0); minimum pre-fwloss PET is 1.5 cm.
    let pet_cm = (pet_daily_mm * f(3.0)).max(f(1.5)) * fix.fwloss[3];
    pet_cm.max(f(0.5))
}

/// Execute monthly soil water budget and solute leaching (Manual §3.3).
#[allow(clippy::too_many_arguments)]
pub fn step_hydrology<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    weather: &MonthlyWeather<F>,
    month: u8,
    irri_cm: F,
    co2_transp_mult: F,
    elements: ElementMode,
    profile: CompatibilityProfile,
    ledger: &mut FlowLedger<F>,
) -> HydrologyStepResult<F> {
    let pet = linacre_pet(month, weather, derived, fix, profile);
    step_hydrology_with_pet(
        state,
        derived,
        fix,
        weather,
        pet,
        irri_cm,
        co2_transp_mult,
        elements,
        ledger,
    )
}

/// Execute monthly soil water budget and solute leaching using a precomputed monthly PET (cm).
#[allow(clippy::too_many_arguments)]
pub fn step_hydrology_with_pet<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    weather: &MonthlyWeather<F>,
    pet: F,
    irri_cm: F,
    co2_transp_mult: F,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) -> HydrologyStepResult<F> {
    let f = F::from_f64;
    let prev_avh2o_root = state.avh2o[0];
    let prev_avh2o_top30 = state.avh2o[2];
    let tmean = (weather.tmin_c + weather.tmax_c) * f(0.5);
    let mut pet_rem = pet;

    // 1. Snow accumulation, melt above `tmelt[0]`, and sublimation (Manual §3.3 p. 3-4).
    // Precipitation accumulates as snow when `tmean <= 0 °C`; snowpack melts whenever
    // `tmean > tmelt[0]` (which can be negative, e.g. -6 °C) before sublimation acts on
    // any remaining snowpack.
    let mut liquid_input = irri_cm.max(F::ZERO);
    let mut evap_subl = F::ZERO;
    let is_snow_month = tmean <= F::ZERO;
    let precip = weather.precip_cm.max(F::ZERO);
    if is_snow_month {
        state.snow += precip;
    } else {
        liquid_input += precip;
    }
    if state.snow > F::ZERO && tmean > fix.tmelt[0] {
        let pot_melt = fix.tmelt[1] * (tmean - fix.tmelt[0]);
        let melt = state.snow.min(pot_melt.max(F::ZERO));
        state.snow = (state.snow - melt).max(F::ZERO);
        liquid_input += melt;
    }
    if state.snow > F::ZERO && pet_rem > F::ZERO {
        let subl = state.snow.min(pet_rem * f(0.87));
        state.snow = (state.snow - subl).max(F::ZERO);
        evap_subl += subl;
        pet_rem = (pet_rem - subl / f(0.87)).max(F::ZERO);
    }

    let gross_liquid = liquid_input;

    // 2. Canopy/litter interception & pre-infiltration bare-soil evaporation (Manual §3.3 p. 3-4).
    // Both are computed as fractions of incoming precipitation and subtracted BEFORE infiltration,
    // preventing unrealistic soil drying below wilting point during winter/snow months.
    let live_bio = (state.aglivc.total() + state.rleavc.total()) * f(2.5);
    let dead_bio =
        (state.stdedc.total() + state.strucc[0].total() + state.metabc[0].total()) * f(2.5);
    let tot_bio = live_bio + dead_bio;

    let mut interception = F::ZERO;
    let mut bare_evap = F::ZERO;
    if gross_liquid > F::ZERO && pet_rem > F::ZERO {
        // Interception increases with live + dead aboveground biomass (capped at 45% of rain
        // and 0.5 * fwloss[0] * PET so wet canopies retain PET headroom for transpiration).
        let inter_frac = ((f(0.0003) * live_bio + f(0.0006) * dead_bio) * fix.fwloss[0])
            .clamp_val(F::ZERO, f(0.45));
        let inter_pet_cap = pet_rem * f(0.5) * fix.fwloss[0].clamp_val(F::ZERO, F::TWO);
        interception = (liquid_input * inter_frac)
            .min(inter_pet_cap)
            .min(pet_rem)
            .min(liquid_input);
        liquid_input = (liquid_input - interception).max(F::ZERO);
        pet_rem = (pet_rem - interception).max(F::ZERO);

        // Bare soil evaporation of incoming precipitation decreases exponentially with litter/canopy cover.
        if !is_snow_month && liquid_input > F::ZERO && pet_rem > F::ZERO {
            let bare_frac = (f(0.55) * (-f(0.0007) * tot_bio).exp() * fix.fwloss[1])
                .clamp_val(F::ZERO, f(0.75));
            bare_evap = (liquid_input * bare_frac).min(pet_rem).min(liquid_input);
            liquid_input = (liquid_input - bare_evap).max(F::ZERO);
            pet_rem = (pet_rem - bare_evap).max(F::ZERO);
        }
    }

    // 3. Infiltration and layered soil water redistribution, recording per-layer flux `amov`.
    let mut flux = liquid_input;
    let mut stormflow = F::ZERO;
    let mut amov = [F::ZERO; MAX_LAYERS];
    let mut any_amov = false;
    let leach_layer = derived.nlaypg.min(derived.nlayer).saturating_sub(1);

    for (l, amov_l) in amov.iter_mut().enumerate().take(derived.nlayer) {
        let fc_cm = derived.afiel[l] * derived.adep[l];
        state.asmos[l] += flux;
        if state.asmos[l] > fc_cm {
            let drained = state.asmos[l] - fc_cm;
            state.asmos[l] = fc_cm;
            flux = drained;
            if l <= leach_layer && drained > F::ZERO {
                any_amov = true;
            }
        } else {
            flux = F::ZERO;
        }
        *amov_l = flux;
    }

    let leach_flux = amov[leach_layer];
    let amov_30cm = amov[1.min(derived.nlayer.saturating_sub(1))];

    // Deep storage, stormflow, and baseflow below nlayer.
    let deep_idx = derived.nlayer.min(MAX_LAYERS);
    if flux > F::ZERO {
        let sf = flux * derived.stormf;
        stormflow += sf;
        state.asmos[deep_idx] += flux - sf;
    }
    let baseflow = state.asmos[deep_idx] * derived.basef;
    state.asmos[deep_idx] = (state.asmos[deep_idx] - baseflow).max(F::ZERO);
    let stream = stormflow + baseflow;

    // 4. Plant transpiration across rooting layers `0..nlaypg` (never dries below wilting point).
    // Per Parton et al. (1987, 1993) and Manual §3.3, potential transpiration scales with
    // `pet * fwloss[2] * (1 - exp(-0.020 * live_bio))` (capped at `pet_rem`) and draws directly
    // from available root-zone soil water down to wilting point weighted by `awtl[l]`.
    let mut tran = F::ZERO;
    let ptran = if !is_snow_month && pet_rem > F::ZERO {
        let bio_transp = (F::ONE - (-f(0.020) * live_bio).exp()).clamp_val(f(0.02), F::ONE);
        (pet * fix.fwloss[2] * bio_transp * co2_transp_mult).min(pet_rem)
    } else {
        F::ZERO
    };

    // Precompute wilting-point storage per layer once for transpiration and avh2o updates.
    let mut wp_cm = [F::ZERO; MAX_LAYERS];
    for (l, wp_l) in wp_cm.iter_mut().enumerate().take(derived.nlayer) {
        *wp_l = derived.awilt[l] * derived.adep[l];
    }

    if ptran > F::ZERO {
        let mut avail_weighted = F::ZERO;
        let mut total_avail_root = F::ZERO;
        let mut layer_avail = [F::ZERO; MAX_LAYERS];
        for l in 0..derived.nlaypg {
            let av = (state.asmos[l] - wp_cm[l]).max(F::ZERO);
            layer_avail[l] = av;
            total_avail_root += av;
            avail_weighted += av * fix.awtl[l];
        }
        if avail_weighted > F::EPS_GUARD && total_avail_root > F::EPS_GUARD {
            let target_tran = ptran.min(total_avail_root);
            let scale = target_tran / avail_weighted;
            for l in 0..derived.nlaypg {
                let take = (scale * (layer_avail[l] * fix.awtl[l])).min(layer_avail[l]);
                state.asmos[l] = (state.asmos[l] - take).max(wp_cm[l]);
                tran += take;
            }
        }
    }

    // 5. Update available water indices avh2o[0..3] and top-layer relative water content.
    let mut av_root = F::ZERO;
    let mut av_prof = F::ZERO;
    let mut av_top30 = F::ZERO;
    let mut depth_acc = F::ZERO;
    for l in 0..derived.nlayer {
        let av = (state.asmos[l] - wp_cm[l]).max(F::ZERO);
        av_prof += av;
        if l < derived.nlaypg {
            av_root += av;
        }
        if depth_acc < f(30.0) {
            let frac_in_30 = ((f(30.0) - depth_acc) / derived.adep[l]).clamp_val(F::ZERO, F::ONE);
            av_top30 += av * frac_in_30;
        }
        depth_acc += derived.adep[l];
    }
    state.avh2o = [av_root, av_prof, av_top30];

    let top_awhc = ((derived.afiel[0] - derived.awilt[0]) * derived.adep[0]).max(F::EPS_GUARD);
    let top_rwcf = ((state.asmos[0] - wp_cm[0]) / top_awhc).clamp_val(F::ZERO, F::ONE);

    let aet = evap_subl + interception + bare_evap + tran;
    state.pet = pet;
    state.aet = aet;
    state.stream = stream;

    // 6. Per-layer saturated solute leaching (Manual §3.3 p. 3-6, A2-9/A2-10):
    // Linear function of saturated water flow `amov[l]` up to critical flow `fix.minlch` (18 cm).
    if any_amov {
        let minlch_denom = fix.minlch.max(f(0.1));
        let fleach = [derived.fleach_n, derived.fleach_p, derived.fleach_s];
        let nelem = elements.nelem();
        for e in 0..nelem {
            if fleach[e] <= F::ZERO {
                continue;
            }
            for l in (0..=leach_layer).rev() {
                if amov[l] <= F::ZERO {
                    continue;
                }
                let leach_scale = (amov[l] / minlch_denom).clamp_val(F::ZERO, F::ONE);
                let frac = (leach_scale * fleach[e]).clamp_val(F::ZERO, f(0.95));
                let amt = state.minerl[l][e] * frac;
                if amt > F::ZERO {
                    let dst = if l == leach_layer {
                        state.month_leach_e[e] += amt;
                        PoolId::LeachingE(e)
                    } else {
                        PoolId::Minerl(l + 1, e)
                    };
                    ledger.schedule(PoolId::Minerl(l, e), dst, amt);
                }
            }
        }
        ledger.commit(state);
    }

    let pet_denom = pet.max(f(0.1));
    let rel_water_surface = (prev_avh2o_top30 + gross_liquid) / pet_denom;
    let rel_water_root = (prev_avh2o_root + gross_liquid) / pet_denom;

    HydrologyStepResult {
        pet,
        aet,
        tran,
        ptran,
        liquid_input: gross_liquid,
        leach_flux,
        amov_30cm,
        stream,
        rel_water_surface,
        rel_water_root,
        top_rwcf,
    }
}
