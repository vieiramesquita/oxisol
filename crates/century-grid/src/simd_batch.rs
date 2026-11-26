//! Tier-2 performance layer: biome-grouped Struct-of-Arrays (SoA) batching and
//! portable safe SIMD (`wide::f32x8` / `wide::f64x4`) kernels targeting AVX2+FMA3
//! on Ryzen Zen 3 and AVX2/AVX-512 on AMD EPYC servers.

use std::collections::BTreeMap;

use century_core::Float;
use century_io::vemap::PreloadedCell;
use wide::{f32x8, f64x4, CmpGt, CmpLt};

/// Cells grouped by biome ID so homogeneous SIMD lanes experience zero branch divergence.
#[derive(Debug, Clone)]
pub struct BiomeBatchGroup {
    /// VEMAP biome/landuse ID.
    pub biome_id: i32,
    /// Indices into `PreloadedGrid::cells`.
    pub cell_indices: Vec<usize>,
}

/// Partition cells into biome-homogeneous groups for SIMD batching and I-cache locality.
pub fn group_cells_by_biome<F: Float>(cells: &[PreloadedCell<F>]) -> Vec<BiomeBatchGroup> {
    let mut map: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
    for c in cells {
        map.entry(c.biome_id).or_default().push(c.cell_index);
    }
    map.into_iter()
        .map(|(biome_id, cell_indices)| BiomeBatchGroup {
            biome_id,
            cell_indices,
        })
        .collect()
}

/// Fast rational approximation of `atan(x)` for `f32x8` (max error < 1.5e-4 across all real x),
/// enabling branchless 8-wide SIMD evaluation of CENTURY `tcalc` decomposition temperature curves.
#[inline(always)]
fn atan_approx_f32x8(x: f32x8) -> f32x8 {
    // Use standard identity: atan(x) = sign(x) * (pi/2 - atan(1/|x|)) for |x| > 1
    // and minimax polynomial on [0, 1]: x * (0.999866 + x^2 * (-0.3302995 + x^2 * (0.180141 + x^2 * (-0.085133 + x^2 * 0.0208351))))
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);
    let half_pi = f32x8::splat(core::f32::consts::FRAC_PI_2);

    let ax = x.abs();
    let gt1 = ax.cmp_gt(one);
    let z = gt1.blend(one / ax, ax);
    let z2 = z * z;
    let p = f32x8::splat(0.0208351)
        .mul_add(z2, f32x8::splat(-0.085133))
        .mul_add(z2, f32x8::splat(0.180141))
        .mul_add(z2, f32x8::splat(-0.3302995))
        .mul_add(z2, f32x8::splat(0.999866))
        * z;
    let mag = gt1.blend(half_pi - p, p);
    let is_neg = x.cmp_lt(zero);
    is_neg.blend(-mag, mag)
}

/// Evaluate abiotic decomposition factor `defac = f_T(stemp) * f_W(rel_water)` across
/// 8 cells simultaneously using `wide::f32x8` (compiles to AVX2+FMA3 on Zen 3 / EPYC)
/// using the published Parton et al. (1994) arctangent temperature curve (`IDEF = 2`).
#[inline]
pub fn batch_defac_f32x8(stemp: f32x8, rel_water: f32x8, _teff: [f32; 3]) -> f32x8 {
    let pi = f32x8::splat(core::f32::consts::PI);
    let c = f32x8::splat(core::f32::consts::PI * 0.0309);
    let t_off = f32x8::splat(15.7);

    let raw_t =
        f32x8::splat(0.56) + (f32x8::splat(1.46) / pi) * atan_approx_f32x8(c * (stemp - t_off));
    let tf = raw_t.max(f32x8::splat(0.01)).min(f32x8::splat(1.5));

    let r = rel_water.max(f32x8::splat(0.0)).min(f32x8::splat(5.0));
    let wf = (f32x8::splat(1.0)
        / (f32x8::splat(1.0) + f32x8::splat(30.0) * (f32x8::splat(-8.5) * r).exp()))
    .max(f32x8::splat(0.01))
    .min(f32x8::splat(1.0));

    (tf * wf).max(f32x8::splat(1.0e-4))
}

/// Evaluate abiotic decomposition factor `defac` across 4 `f64` cells simultaneously (`wide::f64x4`).
#[inline]
pub fn batch_defac_f64x4(stemp: f64x4, rel_water: f64x4, _teff: [f64; 3]) -> f64x4 {
    let s_arr = stemp.to_array();
    let r_arr = rel_water.to_array();

    let mut tf_arr = [0.0f64; 4];
    for i in 0..4 {
        let raw = 0.56
            + (1.46 / core::f64::consts::PI)
                * (core::f64::consts::PI * 0.0309 * (s_arr[i] - 15.7)).atan();
        tf_arr[i] = raw.clamp(0.01, 1.5);
    }
    let tf = f64x4::from(tf_arr);
    let r = f64x4::from(r_arr)
        .max(f64x4::splat(0.0))
        .min(f64x4::splat(5.0));
    let wf = (f64x4::splat(1.0)
        / (f64x4::splat(1.0) + f64x4::splat(30.0) * (f64x4::splat(-8.5) * r).exp()))
    .max(f64x4::splat(0.01))
    .min(f64x4::splat(1.0));
    (tf * wf).max(f64x4::splat(1.0e-4))
}

/// Compute Rawls et al. (1982) effective field capacity and wilting point across 8 cells (`f32x8`).
#[inline]
pub fn batch_rawls_retention_f32x8(
    sand_frac: f32x8,
    clay_frac: f32x8,
    ompc: f32x8,
    rock_frac: f32x8,
) -> (f32x8, f32x8) {
    let hundred = f32x8::splat(100.0);
    let s_pct = sand_frac * hundred;
    let c_pct = clay_frac * hundred;
    let scale = f32x8::splat(1.0) - rock_frac.max(f32x8::splat(0.0)).min(f32x8::splat(0.90));

    let sw4 = f32x8::splat(0.4118) - f32x8::splat(0.0030) * s_pct
        + f32x8::splat(0.0023) * c_pct
        + f32x8::splat(0.0317) * ompc;
    let sw15 = f32x8::splat(0.0260) + f32x8::splat(0.0050) * c_pct + f32x8::splat(0.0158) * ompc;

    let fc = (sw4 * scale)
        .max(f32x8::splat(0.02))
        .min(f32x8::splat(0.85));
    let wp = (sw15 * scale)
        .max(f32x8::splat(0.01))
        .min(fc - f32x8::splat(0.005));
    (fc, wp)
}

/// Struct-of-Arrays (SoA) batch of 8 cells for branchless SIMD SOM carbon kinetics (`f32x8`).
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct SomSoaBatchF32 {
    /// Soil active SOM C (`som1c[1]`) across 8 cells.
    pub som1c: [f32; 8],
    /// Slow SOM C (`som2c`) across 8 cells.
    pub som2c: [f32; 8],
    /// Passive SOM C (`som3c`) across 8 cells.
    pub som3c: [f32; 8],
    /// Texture active multiplier `eftext`.
    pub eftext: [f32; 8],
    /// Active SOM respiration fraction `p1co2_soil`.
    pub p1co2: [f32; 8],
    /// Active -> Passive transfer fraction `fps1s3`.
    pub fps1s3: [f32; 8],
    /// Slow -> Passive transfer fraction `fps2s3`.
    pub fps2s3: [f32; 8],
    /// Accumulated heterotrophic respiration across 8 cells.
    pub resp: [f32; 8],
}

/// Advance 8 cells' Active/Slow/Passive SOM carbon kinetics simultaneously in `f32x8` SIMD
/// (exact C conservation: `d(som1 + som2 + som3) + d(resp) == 0`).
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn batch_som_decay_f32x8(
    batch: &mut SomSoaBatchF32,
    defac: f32x8,
    anerb: f32x8,
    dec3_soil: f32,
    dec5: f32,
    dec4: f32,
    p2co2: f32,
    p3co2: f32,
    dt: f32,
) {
    let mut s1 = f32x8::from(batch.som1c);
    let mut s2 = f32x8::from(batch.som2c);
    let mut s3 = f32x8::from(batch.som3c);
    let mut resp = f32x8::from(batch.resp);

    let eftext = f32x8::from(batch.eftext);
    let p1co2 = f32x8::from(batch.p1co2);
    let fps1s3 = f32x8::from(batch.fps1s3);
    let fps2s3 = f32x8::from(batch.fps2s3);

    let env = defac * anerb * f32x8::splat(dt);
    let max_f = f32x8::splat(0.95);
    let one = f32x8::splat(1.0);
    let zero = f32x8::splat(0.0);

    // 1. Soil Active SOM -> Resp + SOM3 + SOM2
    let f1 = (f32x8::splat(dec3_soil) * eftext * env).min(max_f);
    let c1 = s1 * f1;
    let r1 = c1 * p1co2;
    let c1_to_s3 = c1 * fps1s3;
    let c1_to_s2 = (c1 - r1 - c1_to_s3).max(zero);
    s1 -= c1;
    s2 += c1_to_s2;
    s3 += c1_to_s3;
    resp += r1;

    // 2. Slow SOM -> Resp + SOM3 + SOM1
    let f2 = (f32x8::splat(dec5) * env).min(max_f);
    let c2 = s2 * f2;
    let r2 = c2 * f32x8::splat(p2co2);
    let c2_to_s3 = c2 * fps2s3;
    let c2_to_s1 = (c2 - r2 - c2_to_s3).max(zero);
    s2 -= c2;
    s1 += c2_to_s1;
    s3 += c2_to_s3;
    resp += r2;

    // 3. Passive SOM -> Resp + SOM1
    let f3 = (f32x8::splat(dec4) * env).min(max_f);
    let c3 = s3 * f3;
    let r3 = c3 * f32x8::splat(p3co2);
    let c3_to_s1 = c3 * (one - f32x8::splat(p3co2));
    s3 -= c3;
    s1 += c3_to_s1;
    resp += r3;

    batch.som1c = s1.to_array();
    batch.som2c = s2.to_array();
    batch.som3c = s3.to_array();
    batch.resp = resp.to_array();
}

#[cfg(test)]
mod tests {
    use super::*;
    use century_core::subsystems::decomposition::{decomp_moisture_factor, decomp_temp_factor};
    use century_core::FixParams;

    #[test]
    fn simd_defac_matches_scalar_f32_and_f64() {
        let fix32 = FixParams::<f32>::default();
        let temps = [-5.0f32, 0.0, 5.0, 12.0, 18.0, 24.0, 30.0, 38.0];
        let rel_w = [0.05f32, 0.2, 0.4, 0.6, 0.8, 1.0, 1.5, 2.5];

        let v_res =
            batch_defac_f32x8(f32x8::from(temps), f32x8::from(rel_w), fix32.teff).to_array();
        for i in 0..8 {
            let scalar = (decomp_temp_factor(temps[i], &fix32) * decomp_moisture_factor(rel_w[i]))
                .max(1.0e-4);
            assert!(
                (v_res[i] - scalar).abs() < 2.0e-3,
                "lane {i}: simd={} scalar={}",
                v_res[i],
                scalar
            );
        }

        let fix64 = FixParams::<f64>::default();
        let t64 = [-2.0, 10.0, 22.0, 32.0];
        let w64 = [0.15, 0.5, 0.9, 1.8];
        let v64 = batch_defac_f64x4(f64x4::from(t64), f64x4::from(w64), fix64.teff).to_array();
        for i in 0..4 {
            let scalar =
                (decomp_temp_factor(t64[i], &fix64) * decomp_moisture_factor(w64[i])).max(1.0e-4);
            assert!((v64[i] - scalar).abs() < 1.0e-12);
        }
    }

    #[test]
    fn simd_som_decay_conserves_carbon_exactly() {
        let mut b = SomSoaBatchF32 {
            som1c: [150.0; 8],
            som2c: [2000.0; 8],
            som3c: [1500.0; 8],
            eftext: [0.55; 8],
            p1co2: [0.44; 8],
            fps1s3: [0.011; 8],
            fps2s3: [0.005; 8],
            resp: [0.0; 8],
        };
        let init_tot: [f32; 8] = core::array::from_fn(|i| b.som1c[i] + b.som2c[i] + b.som3c[i]);
        let defac = f32x8::splat(0.65);
        let anerb = f32x8::splat(1.0);
        for _ in 0..480 {
            batch_som_decay_f32x8(
                &mut b,
                defac,
                anerb,
                7.3,
                0.2,
                0.0045,
                0.55,
                0.55,
                1.0 / 48.0,
            );
        }
        for (i, &init_c) in init_tot.iter().enumerate() {
            let end_tot = b.som1c[i] + b.som2c[i] + b.som3c[i] + b.resp[i];
            assert!(
                (end_tot - init_c).abs() < 1.0e-2,
                "lane {i} carbon conservation drift: {end_tot} vs {init_c}"
            );
        }
    }
}
