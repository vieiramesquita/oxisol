//! Atmospheric CO₂ physiological response modifiers (production, transpiration,
//! C:E stoichiometry, and root:shoot allocation).
//!
//! Specification: `docs/spec/04_vegetation_crop_and_forest.md` §3.

use crate::float::Float;
use crate::params::{CropParams, FixParams, TreeParams, MAX_ELEM};

/// Computed CO₂ response multipliers for a single monthly step.
#[derive(Debug, Clone, Copy)]
pub struct Co2Effects<F: Float> {
    /// Crop production multiplier.
    pub crop_prod: F,
    /// Forest production multiplier.
    pub forest_prod: F,
    /// Crop transpiration multiplier.
    pub crop_transp: F,
    /// Forest transpiration multiplier.
    pub forest_transp: F,
    /// Crop C:E ratio multipliers `[min/max][elem]`.
    pub crop_ce: [[F; MAX_ELEM]; 2],
    /// Forest C:E ratio multipliers `[min/max][elem]`.
    pub forest_ce: [[F; MAX_ELEM]; 2],
    /// Crop root:shoot multiplier.
    pub crop_rs: F,
    /// Forest root:shoot multiplier.
    pub forest_rs: F,
}

impl<F: Float> Default for Co2Effects<F> {
    fn default() -> Self {
        Self {
            crop_prod: F::ONE,
            forest_prod: F::ONE,
            crop_transp: F::ONE,
            forest_transp: F::ONE,
            crop_ce: [[F::ONE; MAX_ELEM]; 2],
            forest_ce: [[F::ONE; MAX_ELEM]; 2],
            crop_rs: F::ONE,
            forest_rs: F::ONE,
        }
    }
}

/// Interpolate a multiplier between `1.0` at `co2_base` and `doubled_mult` at `co2_double`
/// using logarithmic CO₂ scaling (`ln(co2 / base) / ln(double / base)`).
#[inline]
fn interp_co2<F: Float>(co2: F, base: F, double: F, doubled_mult: F) -> F {
    if (double - base).abs() <= F::EPS_GUARD || co2 <= F::EPS_GUARD || base <= F::EPS_GUARD {
        return F::ONE;
    }
    let ratio = (co2 / base).ln() / (double / base).ln();
    (F::ONE + ratio * (doubled_mult - F::ONE)).max(F::from_f64(0.1))
}

/// Evaluate CO₂ physiological effects for active crop and tree parameter sets.
pub fn compute_co2_effects<F: Float>(
    co2_ppm: F,
    fix: &FixParams<F>,
    crop: Option<&CropParams<F>>,
    tree: Option<&TreeParams<F>>,
) -> Co2Effects<F> {
    let base = fix.co2ppm[0].max(F::from_f64(100.0));
    let dbl = if (fix.co2ppm[1] - base).abs() > F::from_f64(1.0) {
        fix.co2ppm[1]
    } else {
        base * F::TWO
    };

    let mut eff = Co2Effects::default();
    if let Some(c) = crop {
        eff.crop_prod = interp_co2(co2_ppm, base, dbl, c.co2ipr);
        eff.crop_transp = interp_co2(co2_ppm, base, dbl, c.co2itr);
        eff.crop_rs = interp_co2(co2_ppm, base, dbl, c.co2irs);
        for bound in 0..2 {
            for e in 0..MAX_ELEM {
                eff.crop_ce[bound][e] = interp_co2(co2_ppm, base, dbl, c.co2ice[bound][e]);
            }
        }
    }
    if let Some(t) = tree {
        eff.forest_prod = interp_co2(co2_ppm, base, dbl, t.co2ipr);
        eff.forest_transp = interp_co2(co2_ppm, base, dbl, t.co2itr);
        eff.forest_rs = interp_co2(co2_ppm, base, dbl, t.co2irs);
        for bound in 0..2 {
            for e in 0..MAX_ELEM {
                eff.forest_ce[bound][e] = interp_co2(co2_ppm, base, dbl, t.co2ice[bound][e]);
            }
        }
    }
    eff
}
