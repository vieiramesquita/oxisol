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
    /// Effective liquid water reaching soil surface after snow/melt (cm).
    pub liquid_input: F,
    /// Water flux draining past rooting/leaching boundary (cm).
    pub leach_flux: F,
    /// Total runoff + baseflow + stormflow (cm).
    pub stream: F,
    /// Moisture ratio `(avh2o[2] + liquid_input) / pet` used by `defac` and `anerb`.
    pub rel_water_surface: F,
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
pub fn linacre_pet<F: Float>(
    month: u8,
    weather: &MonthlyWeather<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    profile: CompatibilityProfile,
) -> F {
    let f = F::from_f64;
    let tmean = (weather.tmin_c + weather.tmax_c) * f(0.5);
    let trange = (weather.tmax_c - weather.tmin_c).max(F::ZERO);
    let tm = tmean + f(0.006) * derived.elevation;

    let td_dep = f(0.0023) * derived.elevation
        + f(0.37) * tmean
        + f(0.53) * trange
        + f(0.35) * derived.trange_annual
        - f(10.9);

    let lat_denom = (f(100.0) - derived.lat.abs()).max(f(10.0));
    let t_denom = (f(80.0) - tmean).max(f(10.0));
    let pet_daily_mm = ((f(700.0) * tm / lat_denom) + f(15.0) * td_dep) / t_denom;

    // Daylength scaling normalized to 12 h reference.
    let dl = daylength_hours(month, derived.lat, profile);
    let dl_scale = (dl / f(12.0)).clamp_val(f(0.25), f(1.75));

    // Convert mm/day -> cm/month (30 days / 10 mm/cm = 3.0); minimum pre-fwloss PET is 1.5 cm.
    let pet_cm = (pet_daily_mm * f(3.0) * dl_scale).max(f(1.5)) * fix.fwloss[3];
    pet_cm.max(f(0.5))
}

/// Execute monthly soil water budget and solute leaching.
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
    let f = F::from_f64;
    let tmean = (weather.tmin_c + weather.tmax_c) * f(0.5);
    let pet = linacre_pet(month, weather, derived, fix, profile);
    let mut pet_rem = pet;

    // 1. Snow accumulation, sublimation, and melt.
    let mut liquid_input = irri_cm.max(F::ZERO);
    let mut evap_subl = F::ZERO;
    if tmean <= F::ZERO {
        state.snow += weather.precip_cm.max(F::ZERO);
        let subl = state.snow.min(pet_rem * f(0.87));
        state.snow = (state.snow - subl).max(F::ZERO);
        evap_subl += subl;
        pet_rem = (pet_rem - subl / f(0.87)).max(F::ZERO);
    } else {
        liquid_input += weather.precip_cm.max(F::ZERO);
        if state.snow > F::ZERO && tmean > fix.tmelt[0] {
            let pot_melt = fix.tmelt[1] * (tmean - fix.tmelt[0]);
            let melt = state.snow.min(pot_melt.max(F::ZERO));
            state.snow = (state.snow - melt).max(F::ZERO);
            liquid_input += melt;
        }
    }

    // 2. Canopy and litter interception.
    let live_bio = state.aglivc.total() + state.rleavc.total();
    let dead_bio = state.stdedc.total() + state.strucc[0].total() + state.metabc[0].total();
    let inter_frac =
        (f(0.0003) * live_bio + f(0.0006) * dead_bio).clamp_val(F::ZERO, f(0.45)) * fix.fwloss[0];
    let interception = (liquid_input * inter_frac).min(pet_rem).min(liquid_input);
    liquid_input = (liquid_input - interception).max(F::ZERO);
    pet_rem = (pet_rem - interception).max(F::ZERO);

    // 3. Infiltration and layered soil water redistribution.
    let mut flux = liquid_input;
    let mut stormflow = F::ZERO;
    let mut leach_flux = F::ZERO;
    let leach_layer = derived.nlaypg.min(derived.nlayer).saturating_sub(1);

    for l in 0..derived.nlayer {
        let fc_cm = derived.afiel[l] * derived.adep[l];
        state.asmos[l] += flux;
        if state.asmos[l] > fc_cm {
            let excess = state.asmos[l] - fc_cm;
            // Apply drainage impedance (drain < 1 slows drainage, leaving water above FC).
            let drained = excess * derived.drain;
            state.asmos[l] -= drained;
            flux = drained;
        } else {
            flux = F::ZERO;
        }
        if l == leach_layer {
            leach_flux = flux;
        }
    }

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

    // 4. Bare soil evaporation from top layer.
    let shade_cover = (-f(0.0035) * (live_bio + dead_bio)).exp();
    let top_wp = derived.awilt[0] * derived.adep[0];
    let top_fc = derived.afiel[0] * derived.adep[0];
    let top_rel = if top_fc > F::EPS_GUARD {
        (state.asmos[0] / top_fc).clamp_val(F::ZERO, F::ONE)
    } else {
        F::ZERO
    };
    let evap_pot = pet_rem * fix.fwloss[1] * shade_cover * top_rel;
    // Bare soil evaporation can dry top layer slightly below wilting point (down to 0.5*wp).
    let evap_avail = (state.asmos[0] - top_wp * f(0.5)).max(F::ZERO);
    let bare_evap = evap_pot.min(evap_avail).min(pet_rem);
    state.asmos[0] = (state.asmos[0] - bare_evap).max(F::ZERO);
    pet_rem = (pet_rem - bare_evap).max(F::ZERO);

    // 5. Plant transpiration across rooting layers `0..nlaypg`.
    let bio_transp = (F::ONE - (-f(0.007) * live_bio).exp()).clamp_val(F::ZERO, F::ONE);
    let ptran = (pet_rem * fix.fwloss[2] * bio_transp * co2_transp_mult).min(pet_rem);
    let mut avail_weighted = F::ZERO;
    let mut layer_avail = [F::ZERO; MAX_LAYERS];
    for l in 0..derived.nlaypg {
        let wp_cm = derived.awilt[l] * derived.adep[l];
        let av = (state.asmos[l] - wp_cm).max(F::ZERO);
        layer_avail[l] = av;
        avail_weighted += av * fix.awtl[l];
    }

    let mut tran = F::ZERO;
    if ptran > F::ZERO && avail_weighted > F::EPS_GUARD {
        let mut total_avail_root = F::ZERO;
        for l in 0..derived.nlaypg {
            total_avail_root += layer_avail[l];
        }
        // Water stress curve reduces actual transpiration as root zone dries.
        let root_frac = if derived.twhc_root > F::EPS_GUARD {
            (total_avail_root / derived.twhc_root).clamp_val(F::ZERO, F::ONE)
        } else {
            F::ZERO
        };
        let stress = (f(1.0) - (-f(3.5) * root_frac).exp()).clamp_val(F::ZERO, F::ONE);
        let target_tran = (ptran * stress).min(total_avail_root);
        for l in 0..derived.nlaypg {
            let take =
                (target_tran * (layer_avail[l] * fix.awtl[l]) / avail_weighted).min(layer_avail[l]);
            state.asmos[l] = (state.asmos[l] - take).max(F::ZERO);
            tran += take;
        }
    }

    // 6. Update available water indices avh2o[0..3].
    let mut av_root = F::ZERO;
    let mut av_prof = F::ZERO;
    let mut av_top30 = F::ZERO;
    let mut depth_acc = F::ZERO;
    for l in 0..derived.nlayer {
        let wp_cm = derived.awilt[l] * derived.adep[l];
        let av = (state.asmos[l] - wp_cm).max(F::ZERO);
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

    let aet = evap_subl + interception + bare_evap + tran;
    state.pet = pet;
    state.aet = aet;
    state.stream = stream;

    // 7. Solute leaching (mineral N/P/S from soil layers when leach_flux > minlch).
    if leach_flux > fix.minlch {
        let leach_scale = ((leach_flux - fix.minlch) / f(18.0)).clamp_val(F::ZERO, F::ONE);
        let fleach = [derived.fleach_n, derived.fleach_p, derived.fleach_s];
        let nelem = elements.nelem();
        for e in 0..nelem {
            let frac = (leach_scale * fleach[e]).clamp_val(F::ZERO, f(0.95));
            if frac > F::ZERO {
                // Move fraction down through layers and export from bottom leaching layer.
                for l in (0..=leach_layer).rev() {
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
        }
    }
    ledger.commit(state);

    let rel_water_surface = (state.avh2o[2] + liquid_input) / pet.max(f(0.1));

    HydrologyStepResult {
        pet,
        aet,
        tran,
        ptran,
        liquid_input,
        leach_flux,
        stream,
        rel_water_surface,
    }
}
