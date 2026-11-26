//! Near-surface soil temperature estimation (`soil_temp`) with biomass insulation
//! and snowpack thermal buffering.
//!
//! Specification: `docs/spec/02_hydrology_and_soil_water.md` §5.

use crate::float::Float;
use crate::params::FixParams;
use crate::state::{CellState, MonthlyWeather};

/// Compute monthly mean near-surface soil temperature (°C) from air temperature,
/// insulating surface plant biomass/litter (`fix.pmxbio`, `fix.pmxtmp`, `fix.pmntmp`),
/// and snowpack thermal insulation (Parton 1984 / Manual §3.3).
#[inline]
pub fn compute_soil_temp<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    weather: &MonthlyWeather<F>,
) -> F {
    let f = F::from_f64;

    // Total insulating surface biomass (g dry matter/m2 ≈ 2.5 * g C/m2, capped at fix.pmxbio).
    let surf_c = state.aglivc.total()
        + state.stdedc.total()
        + state.strucc[0].total()
        + state.metabc[0].total()
        + state.rleavc.total() * f(0.5);
    let max_bio = fix.pmxbio.max(f(100.0));
    let bio = (surf_c * f(2.5)).min(max_bio);

    let tmax_air = weather.tmax_c;
    let tmin_air = weather.tmin_c;
    let pmxtmp = if fix.pmxtmp < F::ZERO {
        fix.pmxtmp
    } else {
        f(-0.0035)
    };
    let pmntmp = if fix.pmntmp > F::ZERO {
        fix.pmntmp
    } else {
        f(0.004)
    };

    // Empirical CENTURY surface soil temperature equations (Parton 1984, Manual §3.3).
    let tmax_soil = tmax_air
        + (f(25.4) / (F::ONE + f(18.0) * (-f(0.20) * tmax_air).exp())) * (pmxtmp * bio).exp()
        - f(0.13) * tmax_air;
    let tmin_soil = tmin_air + pmntmp * bio - f(1.82);

    let tmin_clamped = tmin_soil.min(tmax_soil);
    let mut stemp = (tmax_soil + tmin_clamped) * f(0.5);

    // Snowpack thermally insulates near-surface soil near 0 °C.
    if state.snow > F::EPS_GUARD {
        stemp = F::ZERO;
    }

    state.stemp = stemp;
    stemp
}
