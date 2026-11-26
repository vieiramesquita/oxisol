//! Near-surface soil temperature estimation (`soil_temp`) with biomass insulation
//! and snowpack thermal buffering.
//!
//! Specification: `docs/spec/02_hydrology_and_soil_water.md` §5.

use crate::float::Float;
use crate::state::{CellState, MonthlyWeather};

/// Compute monthly mean near-surface soil temperature (°C) from air temperature,
/// insulating surface plant biomass/litter, and snowpack.
#[inline]
pub fn compute_soil_temp<F: Float>(state: &mut CellState<F>, weather: &MonthlyWeather<F>) -> F {
    let f = F::from_f64;

    // Total insulating surface biomass (g dry matter/m2 ≈ 2.5 * g C/m2, capped at 1000).
    let surf_c = state.aglivc.total()
        + state.stdedc.total()
        + state.strucc[0].total()
        + state.metabc[0].total()
        + state.rleavc.total() * f(0.5);
    let bio = (surf_c * f(2.5)).min(f(1000.0));

    let tmax_air = weather.tmax_c;
    let tmin_air = weather.tmin_c;

    // Empirical CENTURY surface soil temperature equations (Parton et al. 1987).
    let tmax_soil = tmax_air
        + (f(25.4) / (F::ONE + f(18.0) * (-f(0.20) * tmax_air).exp())) * (-f(0.0035) * bio).exp()
        - f(0.13) * tmax_air;
    let tmin_soil = tmin_air + f(0.006) * bio - f(1.82);

    let tmin_clamped = tmin_soil.min(tmax_soil);
    let mut stemp = (tmax_soil + tmin_clamped) * f(0.5);

    // Snowpack buffers soil temperature toward 0 °C when snow is present.
    if state.snow > F::ZERO {
        let snow_buf = (-f(0.15) * state.snow).exp();
        stemp *= snow_buf;
    }

    state.stemp = stemp;
    stemp
}
