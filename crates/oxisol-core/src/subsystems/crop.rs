//! Grassland and agricultural crop production (`potcrp`, `restrp`), root-to-shoot
//! allocation, symbiotic N fixation, senescence, shoot/root death, and standing dead fall.
//!
//! Specification: `docs/spec/04_vegetation_crop_and_forest.md` §1.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{CellDerivedParams, CropParams, FixParams, MAX_ELEM};
use crate::state::{CellState, MonthlyWeather};
use crate::subsystems::co2::Co2Effects;
use crate::subsystems::decomposition::gpdf;
use crate::subsystems::mineralization::partition_residue;

/// Texture-dependent linear moisture growth response (`pprdwc`, Manual §3.7.1 Fig. 3-9, A2-12).
///
/// Accounts for the Noy-Meir (1973) / Sala et al. (1988) reverse-texture effect:
/// x-intercept `intcpt = pprpts[0] + pprpts[1] * wc` (where `wc = afiel[0] - awilt[0]`),
/// reaching `1.0` (unrestricted growth) at `rel_water >= pprpts[2]`.
#[inline]
pub fn pprdwc<F: Float>(wc: F, rel_water: F, pprpts: [F; 3]) -> F {
    let f = F::from_f64;
    let wc_clamped = wc.clamp_val(f(0.02), f(0.50));
    let intcpt = pprpts[0] + pprpts[1] * wc_clamped;
    let upper = pprpts[2].max(intcpt + f(0.05));
    let r = rel_water.max(F::ZERO);
    ((r - intcpt) / (upper - intcpt)).clamp_val(F::ZERO, F::ONE)
}

/// Dynamic monthly available mineral nutrient fraction `favail` (Manual A2-9):
/// - N (`e=0`): `fix.favail[0]`
/// - P (`e=1`): linearly interpolated between `favail[3]` (min) and `favail[4]` (max)
///   as top-layer mineral N increases from 0 to `favail[5]`.
/// - S (`e=2`): `fix.favail[2]`
#[inline]
pub fn element_favail<F: Float>(e: usize, minerl_n0: F, fix: &FixParams<F>) -> F {
    let f = F::from_f64;
    match e {
        0 => fix.favail[0].clamp_val(f(0.05), F::ONE),
        1 => {
            let min_p = fix.favail[3].clamp_val(f(0.05), F::ONE);
            let max_p = fix.favail[4].clamp_val(min_p, F::ONE);
            let ref_n = fix.favail[5].max(F::EPS_GUARD);
            let frac = (minerl_n0 / ref_n).clamp_val(F::ZERO, F::ONE);
            min_p + frac * (max_p - min_p)
        }
        _ => fix.favail[2].clamp_val(f(0.05), F::ONE),
    }
}

/// Root biomass impact factor on available nutrient uptake (`rtimp`).
#[inline]
pub fn root_impact<F: Float>(bglivc: F, fix: &FixParams<F>) -> F {
    (F::ONE - fix.riint * (-fix.rictrl * bglivc).exp()).clamp_val(F::from_f64(0.1), F::ONE)
}

/// Execute monthly grassland/crop production, nutrient uptake, senescence, and litterfall.
///
/// In `Savanna` systems, `tree_shade_fac` (`0..=1`) and `grass_n_share` (`0..=1`) encode
/// woody canopy shading and basal-area nutrient partitioning (Manual §3.7.3).
#[allow(clippy::too_many_arguments)]
pub fn step_crop<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    crop: &CropParams<F>,
    _weather: &MonthlyWeather<F>,
    rel_water_root: F,
    tree_shade_fac: F,
    grass_n_share: F,
    co2: &Co2Effects<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let f = F::from_f64;

    // Update plant residue lignin from annual precipitation.
    let ann_ppt = state.ann_ppt.max(f(1.0));
    state.pltlig[0] = (crop.fligni[0][0] + crop.fligni[1][0] * ann_ppt).clamp_val(f(0.02), f(0.5));
    state.pltlig[1] = (crop.fligni[0][1] + crop.fligni[1][1] * ann_ppt).clamp_val(f(0.02), f(0.5));

    // --- 1. Potential & Actual Production (when growing season is active) ---
    // Note: Pure forest systems (`SystemKind::Forest`) skip `step_crop` entirely in `step_month`.
    // In `GrasslandCrop` and `Savanna` systems, herbaceous growth proceeds whenever `crpgrw == true`.
    if state.crpgrw && tree_shade_fac > F::EPS_GUARD && grass_n_share > F::EPS_GUARD {
        // Manual §3.7.1 p. 3-14: crop temperature response uses root/soil temperature `stemp`.
        let t_fac = gpdf(
            state.stemp,
            crop.ppdf[0],
            crop.ppdf[1],
            crop.ppdf[2],
            crop.ppdf[3],
        );
        let wc = derived.afiel[0] - derived.awilt[0];
        let w_fac = pprdwc(wc, rel_water_root, fix.pprpts);

        // Shading by standing dead litter and (in savannas) tree canopy cover.
        let dead_shade = (F::ONE - f(0.0008) * state.stdedc.total()).clamp_val(f(0.2), F::ONE);
        let tree_shade = tree_shade_fac.clamp_val(F::ZERO, F::ONE);

        // Live biomass establishment curve.
        let bio_fac = if crop.bioflg > f(0.5) && crop.fulcan > F::EPS_GUARD {
            let b = state.aglivc.total();
            if b < crop.fulcan {
                (f(0.3) + f(0.7) * (b / crop.fulcan)).clamp_val(f(0.3), F::ONE)
            } else {
                F::ONE
            }
        } else {
            F::ONE
        };

        // Root-to-shoot allocation.
        let base_rs = if crop.frtc[0] > F::ZERO {
            crop.frtc[0]
        } else {
            // Precipitation-driven natural grassland root:shoot ratio.
            let ag_est = (fix.agppa + fix.agppb * ann_ppt).max(f(10.0));
            let bg_est = (fix.bgppa + fix.bgppb * ann_ppt).max(f(10.0));
            (bg_est / ag_est).clamp_val(f(0.3), f(3.5))
        };
        let rs_ratio = (base_rs * co2.crop_rs + state.fire_frtsh).clamp_val(f(0.1), f(5.0));
        let f_root = rs_ratio / (F::ONE + rs_ratio);
        let f_shoot = F::ONE - f_root;

        let pot_ag = crop.prdx1 * t_fac * w_fac * dead_shade * tree_shade * bio_fac * co2.crop_prod;
        let pot_total = if f_shoot > F::EPS_GUARD {
            pot_ag / f_shoot
        } else {
            pot_ag
        };

        if pot_total > F::EPS_GUARD {
            // --- 2. Nutrient Limitation (Liebig's Law, restrp) ---
            let nelem = elements.nelem();
            let rti = root_impact(state.bglivc.total(), fix);
            let bio_frac = if crop.biomax > F::EPS_GUARD {
                (state.aglivc.total() / crop.biomax).clamp_val(F::ZERO, F::ONE)
            } else {
                F::ONE
            };
            let _n_share = grass_n_share.clamp_val(F::ZERO, F::ONE);

            let mut c_limit = pot_total;
            let mut r_min_ag = [F::ONE; MAX_ELEM];
            let mut r_max_ag = [F::ONE; MAX_ELEM];
            let mut r_min_bg = [F::ONE; MAX_ELEM];
            let mut r_max_bg = [F::ONE; MAX_ELEM];
            let mut e_avail = [F::ZERO; MAX_ELEM];

            for e in 0..nelem {
                let fav = element_favail(e, state.minerl[0][0], fix);
                let soil_tot = state.mineral_sum(derived.nlaypg, e);
                // When `step_forest` already capped its uptake to `1 - grass_n_share`,
                // `soil_tot` already excludes the tree uptake; use `grass_n_share` as a
                // floorless share only if forest left surplus, avoiding double-penalizing grass.
                let soil_avail = soil_tot * fav * rti;
                e_avail[e] = soil_avail + state.crpstg[e];

                r_min_ag[e] = (crop.pramn[e][0] + bio_frac * (crop.pramn[e][1] - crop.pramn[e][0]))
                    * co2.crop_ce[0][e];
                r_max_ag[e] = (crop.pramx[e][0] + bio_frac * (crop.pramx[e][1] - crop.pramx[e][0]))
                    * co2.crop_ce[1][e];
                r_min_bg[e] =
                    (crop.prbmn[e][0] + ann_ppt * crop.prbmn[e][1]).max(f(5.0)) * co2.crop_ce[0][e];
                r_max_bg[e] = (crop.prbmx[e][0] + ann_ppt * crop.prbmx[e][1]).max(r_min_bg[e])
                    * co2.crop_ce[1][e];

                // Minimum nutrient required per unit total C (at widest allowed C:E).
                let min_e_per_c =
                    f_shoot / r_max_ag[e].max(f(1.0)) + f_root / r_max_bg[e].max(f(1.0));
                if min_e_per_c > F::EPS_GUARD {
                    // Symbiotic N fixation adds extra N supply for N-fixing crops (e == 0).
                    let sym_bonus = if e == 0 && crop.snfxmx > F::ZERO {
                        crop.snfxmx * pot_total
                    } else {
                        F::ZERO
                    };
                    let max_c_e = (e_avail[e] + sym_bonus) / min_e_per_c;
                    if max_c_e < c_limit {
                        c_limit = max_c_e;
                    }
                }
            }

            let actual_c = c_limit.clamp_val(F::ZERO, pot_total);
            if actual_c > F::EPS_GUARD {
                let c_ag = actual_c * f_shoot;
                let c_bg = actual_c * f_root;
                ledger.schedule(PoolId::AtmosphereC, PoolId::AglivC, c_ag);
                ledger.schedule(PoolId::AtmosphereC, PoolId::BglivC, c_bg);
                state.month_crop_npp += actual_c;
                state.month_npp += actual_c;

                // Uptake nutrients for shoots and roots.
                for e in 0..nelem {
                    let e_min_ag = c_ag / r_max_ag[e].max(f(1.0));
                    let e_max_ag = c_ag / r_min_ag[e].max(f(1.0));
                    let e_min_bg = c_bg / r_max_bg[e].max(f(1.0));
                    let e_max_bg = c_bg / r_min_bg[e].max(f(1.0));

                    let e_min_tot = e_min_ag + e_min_bg;
                    let e_max_tot = e_max_ag + e_max_bg;
                    let target_tot = e_avail[e].clamp_val(e_min_tot, e_max_tot);

                    // Draw from crop storage first, then root-zone mineral layers, then symbiotic N fixation.
                    let from_stg = target_tot.min(state.crpstg[e]);
                    let rem1 = (target_tot - from_stg).max(F::ZERO);
                    let soil_tot = state.mineral_sum(derived.nlaypg, e);
                    let from_soil = rem1.min(soil_tot);
                    let rem2 = (rem1 - from_soil).max(F::ZERO);
                    let from_sym = if e == 0 && crop.snfxmx > F::ZERO {
                        rem2.min(crop.snfxmx * actual_c)
                    } else {
                        F::ZERO
                    };

                    let actual_e_tot = from_stg + from_soil + from_sym;
                    if actual_e_tot > F::EPS_GUARD {
                        let ag_share = if (e_max_ag + e_max_bg) > F::EPS_GUARD {
                            (e_max_ag / (e_max_ag + e_max_bg)).clamp_val(F::ZERO, F::ONE)
                        } else {
                            f_shoot
                        };
                        let bg_share = F::ONE - ag_share;

                        if from_stg > F::ZERO {
                            ledger.schedule(
                                PoolId::CrpStg(e),
                                PoolId::AglivE(e),
                                from_stg * ag_share,
                            );
                            ledger.schedule(
                                PoolId::CrpStg(e),
                                PoolId::BglivE(e),
                                from_stg * bg_share,
                            );
                        }
                        if from_soil > F::ZERO && soil_tot > F::EPS_GUARD {
                            for l in 0..derived.nlaypg {
                                let take_l = (from_soil * (state.minerl[l][e] / soil_tot))
                                    .min(state.minerl[l][e]);
                                if take_l > F::ZERO {
                                    ledger.schedule(
                                        PoolId::Minerl(l, e),
                                        PoolId::AglivE(e),
                                        take_l * ag_share,
                                    );
                                    ledger.schedule(
                                        PoolId::Minerl(l, e),
                                        PoolId::BglivE(e),
                                        take_l * bg_share,
                                    );
                                }
                            }
                        }
                        if from_sym > F::ZERO {
                            ledger.schedule(
                                PoolId::AtmosphereE(0),
                                PoolId::AglivE(0),
                                from_sym * ag_share,
                            );
                            ledger.schedule(
                                PoolId::AtmosphereE(0),
                                PoolId::BglivE(0),
                                from_sym * bg_share,
                            );
                            state.month_n_fix += from_sym;
                        }
                    }
                }
            }
        }
        ledger.commit(state);
    }

    // --- 3. Shoot Death (dshoot): aglivc -> stdedc ---
    let agc = state.aglivc.total();
    if agc > F::EPS_GUARD {
        let death_frac = if state.senm {
            crop.fsdeth[1].clamp_val(F::ZERO, f(0.99))
        } else {
            let prof_rel = if derived.twhc_profile > F::EPS_GUARD {
                (state.avh2o[1] / derived.twhc_profile).clamp_val(F::ZERO, F::ONE)
            } else {
                F::ONE
            };
            let d = F::ONE - prof_rel;
            let drought_death = crop.fsdeth[0] * d * d;
            let shade_death = if agc > crop.fsdeth[3] {
                crop.fsdeth[2]
            } else {
                F::ZERO
            };
            (drought_death + shade_death).clamp_val(F::ZERO, f(0.95))
        };
        if death_frac > F::ZERO {
            let c_dead = agc * death_frac;
            ledger.schedule(PoolId::AglivC, PoolId::StdedC, c_dead);
            for e in 0..elements.nelem() {
                let e_dead = state.aglive[e] * death_frac;
                let retrans = e_dead * crop.crprtf[e].clamp_val(F::ZERO, f(0.8));
                let mut to_stded = e_dead - retrans;
                if retrans > F::ZERO {
                    ledger.schedule(PoolId::AglivE(e), PoolId::CrpStg(e), retrans);
                }
                // Plant senescence N volatilization (`volpl` via `vlossp`, Manual p. 145).
                if e == 0 && crop.vlossp > F::ZERO && to_stded > F::ZERO {
                    let vol = to_stded * crop.vlossp.clamp_val(F::ZERO, f(0.5));
                    ledger.schedule(PoolId::AglivE(0), PoolId::AtmosphereE(0), vol);
                    state.month_n_volat += vol;
                    to_stded = (to_stded - vol).max(F::ZERO);
                }
                if to_stded > F::ZERO {
                    ledger.schedule(PoolId::AglivE(e), PoolId::StdedE(e), to_stded);
                }
            }
        }
    }
    ledger.commit(state);

    // --- 4. Standing Dead Fall (falstd): stdedc -> surface litter ---
    let sdc = state.stdedc.total();
    if sdc > F::EPS_GUARD && crop.fallrt > F::ZERO {
        let fall_f = crop.fallrt.clamp_val(F::ZERO, f(0.95));
        let c_fall = sdc * fall_f;
        let mut e_fall = [F::ZERO; MAX_ELEM];
        for e in 0..elements.nelem() {
            e_fall[e] = state.stdede[e] * fall_f;
        }
        partition_residue(
            state,
            fix,
            0,
            PoolId::StdedC,
            [PoolId::StdedE(0), PoolId::StdedE(1), PoolId::StdedE(2)],
            c_fall,
            e_fall,
            state.pltlig[0],
            None,
            elements,
            ledger,
        );
    }
    ledger.commit(state);

    // --- 5. Root Death (droot): bglivc -> soil litter ---
    let bgc = state.bglivc.total();
    if bgc > F::EPS_GUARD && state.stemp > crop.rtdtmp && crop.rdr > F::ZERO {
        let root_rel = if derived.twhc_root > F::EPS_GUARD {
            (state.avh2o[0] / derived.twhc_root).clamp_val(F::ZERO, F::ONE)
        } else {
            F::ONE
        };
        let rdeath_f = (crop.rdr * (F::ONE - f(0.5) * root_rel)).clamp_val(F::ZERO, f(0.95));
        let c_rdead = bgc * rdeath_f;
        let mut e_rdead = [F::ZERO; MAX_ELEM];
        for e in 0..elements.nelem() {
            e_rdead[e] = state.bglive[e] * rdeath_f;
        }
        partition_residue(
            state,
            fix,
            1,
            PoolId::BglivC,
            [PoolId::BglivE(0), PoolId::BglivE(1), PoolId::BglivE(2)],
            c_rdead,
            e_rdead,
            state.pltlig[1],
            None,
            elements,
            ledger,
        );
    }
    ledger.commit(state);
}
