//! Forest and savanna woody production (`potfor`), LAI calculation, 5-part allocation,
//! internal nutrient storage (`forstg`), and woody mortality / leaf drop (`wdeath`).
//!
//! Specification: `docs/spec/04_vegetation_crop_and_forest.md` §2.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{CellDerivedParams, FixParams, TreeParams, FOREST_PARTS, MAX_ELEM};
use crate::state::{CellState, MonthlyWeather};
use crate::subsystems::co2::Co2Effects;
use crate::subsystems::crop::{element_favail, pprdwc};
use crate::subsystems::decomposition::gpdf;
use crate::subsystems::mineralization::partition_residue;

/// Compute forest Leaf Area Index (LAI) from leaf carbon and large wood biomass.
#[inline]
pub fn compute_tree_lai<F: Float>(state: &CellState<F>, tree: &TreeParams<F>) -> F {
    let f = F::from_f64;
    let leaf_c = state.rleavc.total();
    if leaf_c <= F::EPS_GUARD {
        return F::ZERO;
    }
    // Biomass-based LAI bounded by wood allometric max LAI.
    let wood_max_lai = if tree.klai > F::EPS_GUARD {
        tree.maxlai * (state.rlwodc.total() / (tree.klai + state.rlwodc.total()))
    } else {
        tree.maxlai
    };
    let lai_from_leaves = if tree.btolai > F::EPS_GUARD {
        leaf_c * f(2.5) * tree.btolai
    } else {
        leaf_c * f(0.01)
    };
    lai_from_leaves
        .min(wood_max_lai.max(f(0.5)))
        .clamp_val(F::ZERO, tree.maxlai)
}

/// Compute savanna tree-grass competition factors (Manual §3.7.3 p. 3-32/3-35, A2-23):
/// returns `(tree_lai, grass_shade_fac, grass_n_share)`.
///
/// - Canopy light attenuation uses Beer-Lambert `exp(laitop * lai)`.
/// - Basal area `ba = rlwodc * 2.5 / basfct` partitions mineral N between trees and grass
///   scaled by `sitpot` and `basfc2`; closed-canopy forests (`rlwodc >> sitpot`) strongly
///   suppress understory grass while open savannas allow vigorous grass-tree coexistence.
#[inline]
pub fn savanna_competition<F: Float>(state: &CellState<F>, tree: &TreeParams<F>) -> (F, F, F) {
    let f = F::from_f64;
    let lai = compute_tree_lai(state, tree);
    let k_ext = tree.laitop.abs().max(f(0.2));
    let grass_shade_fac = (-k_ext * lai).exp().clamp_val(F::ZERO, F::ONE);

    // Tree dominance from basal area & large wood vs site potential (`sitpot`, Manual §3.7.3).
    // Closed forests command ~85-98% of available mineral N; open savannas share N with grass.
    let basfct = tree.basfct.max(f(1.0));
    let basfc2 = tree.basfc2.max(f(0.1));
    let sitpot = tree.sitpot.max(f(200.0));
    let wood_c = state.rlwodc.total();
    let ba_term = (wood_c * f(2.5) / basfct) * basfc2;
    let half_ba = (sitpot / f(150.0)).max(f(1.0));
    let tree_dom = (ba_term / (ba_term + half_ba)).clamp_val(f(0.20), f(0.98));
    let grass_n_share = (F::ONE - tree_dom).clamp_val(f(0.02), f(0.80));

    (lai, grass_shade_fac, grass_n_share)
}

/// Execute monthly forest production, 5-part nutrient uptake, leaf drop, and woody mortality.
/// Returns the canopy LAI.
#[allow(clippy::too_many_arguments)]
pub fn step_forest<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    tree: &TreeParams<F>,
    weather: &MonthlyWeather<F>,
    month: u8,
    liquid_input: F,
    co2: &Co2Effects<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) -> F {
    let f = F::from_f64;
    let lai = compute_tree_lai(state, tree);

    // --- 1. Potential & Actual Forest Production ---
    if state.forgrw {
        let t_avg = (weather.tmin_c + weather.tmax_c) * f(0.5);
        // Manual §3.7.2 p. 3-27: forest temperature growth response uses soil temperature `stemp`.
        let t_fac = gpdf(
            state.stemp,
            tree.ppdf[0],
            tree.ppdf[1],
            tree.ppdf[2],
            tree.ppdf[3],
        );
        let rel_w = (state.avh2o[0] + liquid_input) / state.pet.max(f(0.1));
        let w_fac = pprdwc(rel_w, fix.pprpts);

        // Beer-Lambert canopy light interception.
        // For deciduous forests (DECID == 1, Manual §3.7.2 p. 3-32), spring leaf-out uses
        // the stand's allometric canopy potential from large wood (`klai`, `maxlai`) so
        // dormant winter abscission does not choke early-season canopy flush.
        let k_ext = tree.laitop.abs().max(f(0.2));
        let wood_pot_lai = if tree.klai > F::EPS_GUARD {
            tree.maxlai * (state.rlwodc.total() / (tree.klai + state.rlwodc.total()))
        } else {
            tree.maxlai
        };
        let eff_lai = if tree.decid == 1 {
            lai.max(wood_pot_lai * f(0.75)).max(f(1.0))
        } else {
            lai.max(f(0.2))
        };
        let light_fac = (F::ONE - (-k_ext * eff_lai).exp()).clamp_val(f(0.05), F::ONE);

        // Gross potential forest production: PRDX(4) is in g biomass/m2/month (Manual A2-20),
        // converted to g C/m2/month via 0.40 C/biomass factor (1 / 2.5), minus Ryan (1991)
        // maintenance respiration of live foliage, fine roots, and live sapwood.
        let gross_for = (tree.prdx4 * f(0.40)) * t_fac * w_fac * light_fac * co2.forest_prod;
        let sap_frac = if tree.sapk > F::EPS_GUARD {
            (tree.sapk / (tree.sapk + state.rlwodc.total())).clamp_val(f(0.05), F::ONE)
        } else {
            f(0.2)
        };
        let live_n = state.rleave[0]
            + state.froote[0]
            + state.fbrche[0] * f(0.5)
            + (state.rlwode[0] + state.croote[0]) * sap_frac;
        let q10_resp = f(2.0).powf(((t_avg - f(20.0)) / f(10.0)).clamp_val(f(-3.0), f(3.0)));
        // Ryan (1991): ~0.0106 g C / g N / day * 30 days/month.
        let maint_resp = (live_n * f(0.0106 * 30.0) * q10_resp).min(gross_for * f(0.75));
        let net_cap = (tree.prdx3 * t_fac * w_fac * light_fac * co2.forest_prod).max(F::ZERO);
        let pot_for = (gross_for - maint_resp).max(F::ZERO).min(net_cap);

        if pot_for > F::EPS_GUARD {
            // Select juvenile (0) vs mature (1) allocation fractions.
            let wood_tot = state.rlwodc.total() + state.fbrchc.total();
            let alloc = if wood_tot > f(500.0) {
                tree.fcfrac[1]
            } else {
                tree.fcfrac[0]
            };
            let mut alloc_sum = F::ZERO;
            for &a in &alloc {
                alloc_sum += a.max(F::ZERO);
            }
            let mut frac = [F::ZERO; FOREST_PARTS];
            if alloc_sum > F::EPS_GUARD {
                for p in 0..FOREST_PARTS {
                    frac[p] = alloc[p].max(F::ZERO) / alloc_sum;
                }
            } else {
                frac = [f(0.25), f(0.20), f(0.15), f(0.30), f(0.10)];
            }

            // Cap foliar allocation when leaf carbon approaches theoretical max LAI ceiling,
            // redirecting excess carbon to large wood and coarse roots (Manual §3.7.2).
            let max_leaf_c = if tree.btolai > F::EPS_GUARD {
                (tree.maxlai / (f(2.5) * tree.btolai)).clamp_val(f(50.0), f(800.0))
            } else {
                f(400.0)
            };
            let leaf_headroom = (max_leaf_c - state.rleavc.total()).max(F::ZERO);
            if pot_for * frac[0] > leaf_headroom {
                let allowed_f0 = (leaf_headroom / pot_for).clamp_val(F::ZERO, frac[0]);
                let excess_f0 = frac[0] - allowed_f0;
                frac[0] = allowed_f0;
                frac[3] += excess_f0 * f(0.75);
                frac[4] += excess_f0 * f(0.25);
            }

            let nelem = elements.nelem();
            let mut c_limit = pot_for;
            let mut e_avail = [F::ZERO; MAX_ELEM];
            let mut min_e_per_c = [F::ZERO; MAX_ELEM];
            let mut max_e_per_c = [F::ZERO; MAX_ELEM];

            for e in 0..nelem {
                let fav = element_favail(e, state.minerl[0][0], fix);
                let soil_avail = state.mineral_sum(derived.nlaypg, e) * fav;
                e_avail[e] = soil_avail + state.forstg[e];

                let mut mn = F::ZERO;
                let mut mx = F::ZERO;
                for p in 0..FOREST_PARTS {
                    let r_min = (tree.cerfor[0][p][e] * co2.forest_ce[0][e]).max(f(5.0));
                    let r_max = (tree.cerfor[1][p][e] * co2.forest_ce[1][e]).max(r_min);
                    mn += frac[p] / r_max;
                    mx += frac[p] / r_min;
                }
                min_e_per_c[e] = mn;
                max_e_per_c[e] = mx;

                if mn > F::EPS_GUARD {
                    let sym_bonus = if e == 0 && tree.snfxmx > F::ZERO {
                        tree.snfxmx * pot_for
                    } else {
                        F::ZERO
                    };
                    let max_c_e = (e_avail[e] + sym_bonus) / mn;
                    if max_c_e < c_limit {
                        c_limit = max_c_e;
                    }
                }
            }

            let actual_c = c_limit.clamp_val(F::ZERO, pot_for);
            if actual_c > F::EPS_GUARD {
                let c_pools = [
                    PoolId::RleavC,
                    PoolId::FrootC,
                    PoolId::FbrchC,
                    PoolId::RlwodC,
                    PoolId::CrootC,
                ];
                for p in 0..FOREST_PARTS {
                    let cp = actual_c * frac[p];
                    if cp > F::ZERO {
                        ledger.schedule(PoolId::AtmosphereC, c_pools[p], cp);
                    }
                }
                state.month_forest_npp += actual_c;
                state.month_npp += actual_c;

                for e in 0..nelem {
                    let e_min_tot = actual_c * min_e_per_c[e];
                    let e_max_tot = actual_c * max_e_per_c[e];
                    let target_tot = e_avail[e].clamp_val(e_min_tot, e_max_tot);

                    let from_stg = target_tot.min(state.forstg[e]);
                    let rem1 = (target_tot - from_stg).max(F::ZERO);
                    let soil_tot = state.mineral_sum(derived.nlaypg, e);
                    let from_soil = rem1.min(soil_tot);
                    let rem2 = (rem1 - from_soil).max(F::ZERO);
                    let from_sym = if e == 0 && tree.snfxmx > F::ZERO {
                        rem2.min(tree.snfxmx * actual_c)
                    } else {
                        F::ZERO
                    };

                    let e_pools = [
                        PoolId::RleavE(e),
                        PoolId::FrootE(e),
                        PoolId::FbrchE(e),
                        PoolId::RlwodE(e),
                        PoolId::CrootE(e),
                    ];

                    // Distribute nutrient uptake proportionally to each part's max demand.
                    let mut part_w = [F::ZERO; FOREST_PARTS];
                    let mut w_sum = F::ZERO;
                    for p in 0..FOREST_PARTS {
                        let r_min = (tree.cerfor[0][p][e] * co2.forest_ce[0][e]).max(f(5.0));
                        part_w[p] = frac[p] / r_min;
                        w_sum += part_w[p];
                    }
                    if w_sum > F::EPS_GUARD {
                        for p in 0..FOREST_PARTS {
                            let share = part_w[p] / w_sum;
                            if from_stg > F::ZERO {
                                ledger.schedule(PoolId::ForStg(e), e_pools[p], from_stg * share);
                            }
                            if from_soil > F::ZERO && soil_tot > F::EPS_GUARD {
                                for l in 0..derived.nlaypg {
                                    let take_l = (from_soil * (state.minerl[l][e] / soil_tot))
                                        .min(state.minerl[l][e]);
                                    if take_l > F::ZERO {
                                        ledger.schedule(
                                            PoolId::Minerl(l, e),
                                            e_pools[p],
                                            take_l * share,
                                        );
                                    }
                                }
                            }
                            if from_sym > F::ZERO {
                                ledger.schedule(
                                    PoolId::AtmosphereE(0),
                                    e_pools[p],
                                    from_sym * share,
                                );
                            }
                        }
                        if from_sym > F::ZERO {
                            state.month_n_fix += from_sym;
                        }
                    }
                }
            }
        }
        ledger.commit(state);
    }

    // --- 2. Forest Mortality and Leaf Drop (wdeath) ---
    let m_idx = (month.clamp(1, 12) - 1) as usize;

    // 2a. Leaf drop: rleavc -> surface litter (with foliar nutrient retranslocation to forstg).
    // For deciduous trees (DECID == 1, Manual §3.7.2 p. 3-27/3-32), trigger full seasonal
    // abscission (`wooddr[0]`, typically 0.5-1.0) at autumn senescence (`senm`), dormancy
    // (`!forgrw`), or autumn soil cooling (`month >= 10 && stemp < 8 °C`).
    let leaf_c = state.rleavc.total();
    let is_decid_drop =
        tree.decid == 1 && (state.senm || !state.forgrw || (month >= 10 && state.stemp < f(8.0)));
    let ldr = if is_decid_drop {
        let drop_rate = if tree.wooddr[0] > f(0.05) {
            tree.wooddr[0]
        } else {
            f(0.95)
        };
        drop_rate.clamp_val(F::ZERO, F::ONE)
    } else {
        tree.leafdr[m_idx].clamp_val(F::ZERO, f(0.99))
    };
    if leaf_c > F::EPS_GUARD && ldr > F::ZERO {
        let c_drop = leaf_c * ldr;
        let mut e_to_lit = [F::ZERO; MAX_ELEM];
        for e in 0..elements.nelem() {
            let e_drop = state.rleave[e] * ldr;
            let retrans = e_drop * tree.forrtf[e].clamp_val(F::ZERO, f(0.85));
            e_to_lit[e] = (e_drop - retrans).max(F::ZERO);
            if retrans > F::ZERO {
                ledger.schedule(PoolId::RleavE(e), PoolId::ForStg(e), retrans);
            }
        }
        partition_residue(
            state,
            fix,
            0,
            PoolId::RleavC,
            [PoolId::RleavE(0), PoolId::RleavE(1), PoolId::RleavE(2)],
            c_drop,
            e_to_lit,
            tree.wdlig[0],
            None,
            elements,
            ledger,
        );
    }
    ledger.commit(state);

    // 2b. Fine root mortality: frootc -> soil litter.
    let fr_c = state.frootc.total();
    let fr_dr = tree.wooddr[1].clamp_val(F::ZERO, f(0.95));
    if fr_c > F::EPS_GUARD && fr_dr > F::ZERO {
        let c_dead = fr_c * fr_dr;
        let mut e_dead = [F::ZERO; MAX_ELEM];
        for e in 0..elements.nelem() {
            e_dead[e] = state.froote[e] * fr_dr;
        }
        partition_residue(
            state,
            fix,
            1,
            PoolId::FrootC,
            [PoolId::FrootE(0), PoolId::FrootE(1), PoolId::FrootE(2)],
            c_dead,
            e_dead,
            tree.wdlig[1],
            None,
            elements,
            ledger,
        );
    }
    ledger.commit(state);

    // 2c. Fine branches -> wood1, Large wood -> wood2, Coarse roots -> wood3.
    let wood_deaths = [
        (
            state.fbrchc.total(),
            state.fbrche,
            tree.wooddr[2],
            PoolId::FbrchC,
            PoolId::Wood1C,
            [PoolId::FbrchE(0), PoolId::FbrchE(1), PoolId::FbrchE(2)],
            [PoolId::Wood1E(0), PoolId::Wood1E(1), PoolId::Wood1E(2)],
            0usize,
            tree.wdlig[2],
        ),
        (
            state.rlwodc.total(),
            state.rlwode,
            tree.wooddr[3],
            PoolId::RlwodC,
            PoolId::Wood2C,
            [PoolId::RlwodE(0), PoolId::RlwodE(1), PoolId::RlwodE(2)],
            [PoolId::Wood2E(0), PoolId::Wood2E(1), PoolId::Wood2E(2)],
            1usize,
            tree.wdlig[3],
        ),
        (
            state.crootc.total(),
            state.croote,
            tree.wooddr[4],
            PoolId::CrootC,
            PoolId::Wood3C,
            [PoolId::CrootE(0), PoolId::CrootE(1), PoolId::CrootE(2)],
            [PoolId::Wood3E(0), PoolId::Wood3E(1), PoolId::Wood3E(2)],
            2usize,
            tree.wdlig[4],
        ),
    ];

    for (wc, we, dr, src_c, dst_c, src_e, dst_e, w_idx, w_lig) in wood_deaths {
        let rate = dr.clamp_val(F::ZERO, f(0.5));
        if wc > F::EPS_GUARD && rate > F::ZERO {
            let c_m = wc * rate;
            ledger.schedule(src_c, dst_c, c_m);
            for e in 0..elements.nelem() {
                let e_m = we[e] * rate;
                if e_m > F::ZERO {
                    ledger.schedule(src_e[e], dst_e[e], e_m);
                }
            }
            state.wdlig[w_idx] = w_lig.clamp_val(f(0.05), f(0.8));
        }
    }
    ledger.commit(state);

    lai
}
