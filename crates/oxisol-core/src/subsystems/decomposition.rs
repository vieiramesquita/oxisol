//! Soil organic matter, plant litter, and dead wood decomposition (`defac`, `anerb`,
//! 4-substep monthly kinetics, texture stabilization, and heterotrophic respiration).
//!
//! Specification: `docs/spec/03_decomposition_and_som.md`.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{
    export_c_direct, export_e_direct, transfer_c_direct, transfer_e_direct, FlowLedger,
};
use crate::params::{CellDerivedParams, FixParams, TreeParams, MAX_ELEM};
use crate::state::{CarbonPool, CellState};
use crate::subsystems::mineralization::{surface_microbe_ce_ratio, target_ce_ratio};

/// Generalized Poisson Density Function (`gpdf`) used for temperature responses.
///
/// Reformulated algebraically as $\exp(a \ln f + \frac{a}{b}(1 - \exp(b \ln f)))$
/// using 1 `ln` + 2 `exp` instead of 2 `powf` + 1 `exp` for higher single-thread throughput.
#[inline]
pub fn gpdf<F: Float>(x: F, x_opt: F, x_max: F, a: F, b: F) -> F {
    if x >= x_max {
        return F::ZERO;
    }
    let denom = (x_max - x_opt).max(F::EPS_GUARD);
    let frac = (x_max - x) / denom;
    if frac <= F::ZERO {
        return F::ZERO;
    }
    let ln_frac = frac.ln();
    let frac_b = (b * ln_frac).exp();
    let log_val = a * ln_frac + (a / b.max(F::EPS_GUARD)) * (F::ONE - frac_b);
    log_val.exp().clamp_val(F::ZERO, F::ONE)
}

/// Decomposition temperature modifier (`tcalc`, Parton et al. 1994 arctangent curve).
///
/// Published form: `0.56 + (1.46 / pi) * atan(pi * 0.0309 * (T - 15.7))`,
/// which equals `0.100` at 0 °C and `1.000` at 30 °C.
#[inline]
pub fn decomp_temp_factor<F: Float>(stemp: F, _fix: &FixParams<F>) -> F {
    let f = F::from_f64;
    let raw_t = f(0.56) + (f(1.46) / F::PI) * (F::PI * f(0.0309) * (stemp - f(15.7))).atan();
    raw_t.clamp_val(f(0.01), f(1.5))
}

/// Logistic moisture modifier for decomposition `wfunc(r)` (`idef = 2`, ratio option).
#[inline]
pub fn decomp_moisture_factor<F: Float>(rel_water: F) -> F {
    let f = F::from_f64;
    let r = rel_water.clamp_val(F::ZERO, f(5.0));
    (F::ONE / (F::ONE + f(30.0) * (-f(8.5) * r).exp())).clamp_val(f(0.01), F::ONE)
}

/// Relative soil water content moisture modifier (`idef = 1`, Manual A2-10).
#[inline]
pub fn decomp_rwcf_factor<F: Float>(top_rwcf: F) -> F {
    let f = F::from_f64;
    let w = top_rwcf.clamp_val(F::ZERO, F::ONE);
    // Smooth saturating response from 0.05 at wilting point to 1.0 near field capacity.
    (f(0.05) + f(0.95) * w.powf(f(0.65))).clamp_val(f(0.01), F::ONE)
}

/// Anaerobic decomposition reduction factor `anerb` under poor drainage (`drain < 1`).
///
/// Per Metherell et al. (1993, §3.5): anaerobic suppression only operates when
/// soil/air temperature exceeds 2.0 °C (preventing false winter lockup when PET is near zero),
/// and scales linearly between `aneref[0]` and `aneref[1]` toward `aneref[2]`, modulated by `drain`.
#[inline]
pub fn anaerobic_factor<F: Float>(rel_water: F, drain: F, stemp: F, fix: &FixParams<F>) -> F {
    let f = F::from_f64;
    if drain >= f(0.999) || stemp <= f(2.0) {
        return F::ONE;
    }
    let r1 = fix.aneref[0];
    let r2 = fix.aneref[1].max(r1 + f(0.01));
    let min_anerb = fix.aneref[2].clamp_val(f(0.05), F::ONE);
    if rel_water <= r1 {
        return F::ONE;
    }
    let excess = ((rel_water - r1) / (r2 - r1)).clamp_val(F::ZERO, F::ONE);
    let imp = (F::ONE - drain).clamp_val(F::ZERO, F::ONE);
    (F::ONE - excess * imp * (F::ONE - min_anerb)).clamp_val(min_anerb, F::ONE)
}

/// Compute abiotic `defac` and `anerb` and store on `state` (honoring `fix.idef` moisture mode).
#[inline]
pub fn update_abiotic_factors<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    rel_water_surface: F,
    top_rwcf: F,
) -> (F, F) {
    let tf = decomp_temp_factor(state.stemp, fix);
    let wf = if fix.idef == 1 {
        decomp_rwcf_factor(top_rwcf)
    } else {
        decomp_moisture_factor(rel_water_surface)
    };
    let defac = (tf * wf).max(F::from_f64(1.0e-4));
    let anerb = anaerobic_factor(rel_water_surface, derived.drain, state.stemp, fix);
    state.defac = defac;
    state.anerb = anerb;
    (defac, anerb)
}

/// Const-specialized `candec_scale` so LLVM unrolls `NELEM == 1` to pure scalar arithmetic.
#[inline(always)]
fn candec_scale_const<F: Float, const NELEM: usize>(
    donor_c: F,
    donor_e: [F; MAX_ELEM],
    c_flow: F,
    c_net: F,
    target_ce: [F; MAX_ELEM],
    minerl0: [F; MAX_ELEM],
) -> F {
    if donor_c <= F::EPS_GUARD || c_flow <= F::EPS_GUARD {
        return F::ZERO;
    }
    let mut scale = F::ONE;
    let frac_donor = (c_flow / donor_c).clamp_val(F::ZERO, F::ONE);
    for e in 0..NELEM {
        let e_rel = donor_e[e] * frac_donor;
        let e_req = if target_ce[e] > F::EPS_GUARD {
            c_net / target_ce[e]
        } else {
            F::ZERO
        };
        if e_req > e_rel {
            let immob_needed = e_req - e_rel;
            let avail = minerl0[e].max(F::ZERO);
            if immob_needed > avail {
                let s = (avail / immob_needed).clamp_val(F::ZERO, F::ONE);
                if s < scale {
                    scale = s;
                }
            }
        }
    }
    scale
}

/// Direct, audited nutrient mineralization/immobilization helper matching
/// `mineralization::route_decomp_nutrients` with zero enum-dispatch overhead
/// and compile-time `NELEM` unrolling.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn route_decomp_nutrients_direct<F: Float, const NELEM: usize>(
    donor_e_pool: &mut [F; MAX_ELEM],
    recv_e_pool: &mut [F; MAX_ELEM],
    minerl0: &mut [F; MAX_ELEM],
    cum_e_out: &mut [F; MAX_ELEM],
    month_net_min: &mut [F; MAX_ELEM],
    month_n_volat: &mut F,
    vlossg_clamped: F,
    donor_c: F,
    donor_e_snap: [F; MAX_ELEM],
    c_flow: F,
    c_net: F,
    target_ce: [F; MAX_ELEM],
) {
    if donor_c <= F::EPS_GUARD || c_flow <= F::EPS_GUARD {
        return;
    }
    let frac_donor = (c_flow / donor_c).clamp_val(F::ZERO, F::ONE);
    for e in 0..NELEM {
        let e_rel = donor_e_snap[e] * frac_donor;
        let e_req = if target_ce[e] > F::EPS_GUARD {
            c_net / target_ce[e]
        } else {
            F::ZERO
        };
        if e_rel >= e_req {
            let to_recv = e_req;
            let net_min = e_rel - e_req;
            if to_recv > F::ZERO {
                transfer_e_direct(&mut donor_e_pool[e], &mut recv_e_pool[e], to_recv);
            }
            if net_min > F::ZERO {
                if e == 0 && vlossg_clamped > F::ZERO {
                    let volat = (e_rel * vlossg_clamped).min(net_min);
                    let to_min = net_min - volat;
                    export_e_direct(&mut donor_e_pool[0], &mut cum_e_out[0], volat);
                    if to_min > F::ZERO {
                        transfer_e_direct(&mut donor_e_pool[0], &mut minerl0[0], to_min);
                    }
                    *month_n_volat += volat;
                    month_net_min[0] += to_min;
                } else {
                    transfer_e_direct(&mut donor_e_pool[e], &mut minerl0[e], net_min);
                    month_net_min[e] += net_min;
                }
            }
        } else {
            let immob = (e_req - e_rel).min(minerl0[e].max(F::ZERO));
            if e_rel > F::ZERO {
                transfer_e_direct(&mut donor_e_pool[e], &mut recv_e_pool[e], e_rel);
            }
            if immob > F::ZERO {
                transfer_e_direct(&mut minerl0[e], &mut recv_e_pool[e], immob);
                month_net_min[e] -= immob;
            }
        }
    }
}

/// Helper to decompose one dead wood pool (`wood1`, `wood2`, or `wood3`) directly.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn decomp_one_wood<F: Float, const NELEM: usize>(
    wc_pool: &mut CarbonPool<F>,
    we_pool: &mut [F; MAX_ELEM],
    som1c_target: &mut CarbonPool<F>,
    som1e_target: &mut [F; MAX_ELEM],
    som2c: &mut CarbonPool<F>,
    som2e: &mut [F; MAX_ELEM],
    minerl0: &mut [F; MAX_ELEM],
    cum_c_out: &mut F,
    cum_e_out: &mut [F; MAX_ELEM],
    month_resp: &mut F,
    month_net_min: &mut [F; MAX_ELEM],
    month_n_volat: &mut F,
    fix: &FixParams<F>,
    vlossg_clamped: F,
    som_lyr: usize,
    lig: F,
    lig_exp: F,
    dec_rate: F,
    defac_dt: F,
    an_m: F,
    r_som1: [F; MAX_ELEM],
    r_som2: [F; MAX_ELEM],
) {
    let f = F::from_f64;
    let wc = wc_pool.total();
    if wc <= F::EPS_GUARD {
        return;
    }
    let we = *we_pool;
    let raw_c = (wc * dec_rate * defac_dt * lig_exp * an_m).min(wc * f(0.95));
    let c_lig = raw_c * lig;
    let c_nonlig = raw_c - c_lig;

    if c_lig > F::EPS_GUARD {
        let c_net = c_lig * (F::ONE - fix.rsplig);
        let scale = candec_scale_const::<F, NELEM>(wc, we, c_lig, c_net, r_som2, *minerl0);
        let cf = c_lig * scale;
        if cf > F::EPS_GUARD {
            let resp = cf * fix.rsplig;
            let net = cf - resp;
            export_c_direct(wc_pool, cum_c_out, resp);
            transfer_c_direct(wc_pool, som2c, net);
            *month_resp += resp;
            route_decomp_nutrients_direct::<F, NELEM>(
                we_pool,
                som2e,
                minerl0,
                cum_e_out,
                month_net_min,
                month_n_volat,
                vlossg_clamped,
                wc,
                we,
                cf,
                net,
                r_som2,
            );
        }
    }
    if c_nonlig > F::EPS_GUARD {
        let resp_f = fix.ps1co2[som_lyr];
        let c_net = c_nonlig * (F::ONE - resp_f);
        let scale = candec_scale_const::<F, NELEM>(wc, we, c_nonlig, c_net, r_som1, *minerl0);
        let cf = c_nonlig * scale;
        if cf > F::EPS_GUARD {
            let resp = cf * resp_f;
            let net = cf - resp;
            export_c_direct(wc_pool, cum_c_out, resp);
            transfer_c_direct(wc_pool, som1c_target, net);
            *month_resp += resp;
            route_decomp_nutrients_direct::<F, NELEM>(
                we_pool,
                som1e_target,
                minerl0,
                cum_e_out,
                month_net_min,
                month_n_volat,
                vlossg_clamped,
                wc,
                we,
                cf,
                net,
                r_som1,
            );
        }
    }
}

/// Execute full monthly decomposition (litter, dead wood, surface microbes,
/// active SOM, slow SOM, passive SOM) using `fix.ntspm` sub-steps (default 4).
#[allow(clippy::too_many_arguments)]
pub fn step_decomposition<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    tree: Option<&TreeParams<F>>,
    leach_flux: F,
    elements: ElementMode,
    _ledger: &mut FlowLedger<F>,
) {
    match elements {
        ElementMode::CarbonNitrogen => {
            step_decomposition_impl::<F, 1>(state, derived, fix, tree, leach_flux);
        }
        ElementMode::CarbonNitrogenPhosphorus => {
            step_decomposition_impl::<F, 2>(state, derived, fix, tree, leach_flux);
        }
        ElementMode::FullCnps => {
            step_decomposition_impl::<F, 3>(state, derived, fix, tree, leach_flux);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn step_decomposition_impl<F: Float, const NELEM: usize>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    tree: Option<&TreeParams<F>>,
    leach_flux: F,
) {
    let f = F::from_f64;
    let nsub = fix.ntspm.clamp(1, 12) as usize;
    let dt = F::ONE / f((12 * nsub) as f64);

    let defac = state.defac;
    let defac_dt = defac * dt;
    let anerb = state.anerb;
    let clt = state.clteff;
    let vlossg_clamped = fix.vlossg.clamp_val(F::ZERO, f(0.5));
    let decw = tree.map(|t| t.decw).unwrap_or([f(1.5), f(0.5), f(0.6)]);

    // Guard lignin inhibition exponentials so zero pools skip transcendental work.
    let str_lig = [
        state.strlig[0].clamp_val(F::ZERO, F::ONE),
        state.strlig[1].clamp_val(F::ZERO, F::ONE),
    ];
    let has_struc0 = state.strucc[0].total() > F::EPS_GUARD;
    let has_struc1 = state.strucc[1].total() > F::EPS_GUARD;
    let str_lig_exp = [
        if has_struc0 {
            (-fix.pligst[0] * str_lig[0]).exp()
        } else {
            F::ONE
        },
        if has_struc1 {
            (-fix.pligst[1] * str_lig[1]).exp()
        } else {
            F::ONE
        },
    ];

    let has_wood0 = state.wood1c.total() > F::EPS_GUARD;
    let has_wood1 = state.wood2c.total() > F::EPS_GUARD;
    let has_wood2 = state.wood3c.total() > F::EPS_GUARD;
    let has_any_wood = has_wood0 || has_wood1 || has_wood2;
    let wd_lig = [
        state.wdlig[0].clamp_val(F::ZERO, F::ONE),
        state.wdlig[1].clamp_val(F::ZERO, F::ONE),
        state.wdlig[2].clamp_val(F::ZERO, F::ONE),
    ];
    let wd_lig_exp = if has_any_wood {
        [
            if has_wood0 {
                (-fix.pligst[0] * wd_lig[0]).exp()
            } else {
                F::ONE
            },
            if has_wood1 {
                (-fix.pligst[0] * wd_lig[1]).exp()
            } else {
                F::ONE
            },
            if has_wood2 {
                (-fix.pligst[1] * wd_lig[2]).exp()
            } else {
                F::ONE
            },
        ]
    } else {
        [F::ONE; 3]
    };

    // Precompute organic leaching fraction once per month (invariant across sub-steps).
    let resp_f_s11 = derived.p1co2_soil;
    let om_leach_f = if leach_flux > F::ZERO {
        ((leach_flux / fix.omlech[2].max(f(0.1))).clamp_val(F::ZERO, F::ONE) * derived.omlech_frac)
            .min(F::ONE - resp_f_s11)
    } else {
        F::ZERO
    };
    let to_s3_f_s11 = derived
        .fps1s3
        .min((F::ONE - resp_f_s11 - om_leach_f).max(F::ZERO));
    let to_s2_f_s11 = (F::ONE - resp_f_s11 - om_leach_f - to_s3_f_s11).max(F::ZERO);
    let s11_non_resp_denom = (F::ONE - resp_f_s11 - om_leach_f).max(F::EPS_GUARD);
    let s11_s3_cflow_mult = to_s3_f_s11 / s11_non_resp_denom * (F::ONE - om_leach_f);
    let s11_s2_cflow_mult = to_s2_f_s11 / s11_non_resp_denom * (F::ONE - om_leach_f);

    let resp_f_s2 = fix.p2co2;
    let to_s3_f_s2 = derived.fps2s3.min((F::ONE - resp_f_s2).max(F::ZERO));
    let to_s1_f_s2 = (F::ONE - resp_f_s2 - to_s3_f_s2).max(F::ZERO);
    let s2_share_denom = (to_s1_f_s2 + to_s3_f_s2).max(F::EPS_GUARD);
    let s2_s3_share = to_s3_f_s2 / s2_share_denom;
    let s2_s1_share = to_s1_f_s2 / s2_share_denom;

    let resp_f_s10 = fix.p1co2a[0].clamp_val(f(0.05), f(0.95));

    for _ in 0..nsub {
        // Compute target C:E ratios for SOM1, SOM2, SOM3 from current top-layer mineral pools.
        let mut r_som1 = [F::ONE; MAX_ELEM];
        let mut r_som2 = [F::ONE; MAX_ELEM];
        let mut r_som3 = [F::ONE; MAX_ELEM];
        for e in 0..NELEM {
            let m0 = state.minerl[0][e];
            r_som1[e] = target_ce_ratio(fix.varat1[e], m0);
            r_som2[e] = target_ce_ratio(fix.varat2[e], m0);
            r_som3[e] = target_ce_ratio(fix.varat3[e], m0);
        }

        // --- 1. Structural Litter Decomposition (surface=0, soil=1) ---
        for layer in 0..2 {
            let sc = state.strucc[layer].total();
            if sc > F::EPS_GUARD {
                let se = state.struce[layer];
                let eff_c = sc.min(fix.strmax[layer]);
                let an_mult = if layer == 1 { anerb } else { F::ONE };
                let clt_mult = if layer == 1 { clt[3] } else { F::ONE };
                let raw_c =
                    (eff_c * fix.dec1[layer] * defac_dt * str_lig_exp[layer] * an_mult * clt_mult)
                        .min(sc * f(0.95));

                let lig = str_lig[layer];
                let c_lig = raw_c * lig;
                let c_nonlig = raw_c - c_lig;

                // Lignin -> SOM2 (respired fraction rsplig).
                if c_lig > F::EPS_GUARD {
                    let c_net = c_lig * (F::ONE - fix.rsplig);
                    let scale = candec_scale_const::<F, NELEM>(
                        sc,
                        se,
                        c_lig,
                        c_net,
                        r_som2,
                        state.minerl[0],
                    );
                    let cf = c_lig * scale;
                    if cf > F::EPS_GUARD {
                        let resp = cf * fix.rsplig;
                        let net = cf - resp;
                        export_c_direct(&mut state.strucc[layer], &mut state.cum_c_out, resp);
                        transfer_c_direct(&mut state.strucc[layer], &mut state.som2c, net);
                        state.month_resp += resp;
                        route_decomp_nutrients_direct::<F, NELEM>(
                            &mut state.struce[layer],
                            &mut state.som2e,
                            &mut state.minerl[0],
                            &mut state.cum_e_out,
                            &mut state.month_net_min,
                            &mut state.month_n_volat,
                            vlossg_clamped,
                            sc,
                            se,
                            cf,
                            net,
                            r_som2,
                        );
                    }
                }

                // Non-lignin -> SOM1[layer] (respired fraction ps1co2[layer]).
                if c_nonlig > F::EPS_GUARD {
                    let resp_f = fix.ps1co2[layer];
                    let c_net = c_nonlig * (F::ONE - resp_f);
                    let r_recv = if layer == 0 {
                        let mut r_surf = [F::ONE; MAX_ELEM];
                        for e in 0..NELEM {
                            r_surf[e] = surface_microbe_ce_ratio(fix.pcemic[e], se[e] / sc);
                        }
                        r_surf
                    } else {
                        r_som1
                    };
                    let scale = candec_scale_const::<F, NELEM>(
                        sc,
                        se,
                        c_nonlig,
                        c_net,
                        r_recv,
                        state.minerl[0],
                    );
                    let cf = c_nonlig * scale;
                    if cf > F::EPS_GUARD {
                        let resp = cf * resp_f;
                        let net = cf - resp;
                        export_c_direct(&mut state.strucc[layer], &mut state.cum_c_out, resp);
                        transfer_c_direct(&mut state.strucc[layer], &mut state.som1c[layer], net);
                        state.month_resp += resp;
                        route_decomp_nutrients_direct::<F, NELEM>(
                            &mut state.struce[layer],
                            &mut state.som1e[layer],
                            &mut state.minerl[0],
                            &mut state.cum_e_out,
                            &mut state.month_net_min,
                            &mut state.month_n_volat,
                            vlossg_clamped,
                            sc,
                            se,
                            cf,
                            net,
                            r_recv,
                        );
                    }
                }
            }
        }

        // --- 2. Metabolic Litter Decomposition (surface=0, soil=1) ---
        for layer in 0..2 {
            let mc = state.metabc[layer].total();
            if mc > F::EPS_GUARD {
                let me = state.metabe[layer];
                let an_mult = if layer == 1 { anerb } else { F::ONE };
                let raw_c = (mc * fix.dec2[layer] * defac_dt * an_mult).min(mc * f(0.95));
                let resp_f = fix.pmco2[layer];
                let c_net = raw_c * (F::ONE - resp_f);
                let r_recv = if layer == 0 {
                    let mut r_surf = [F::ONE; MAX_ELEM];
                    for e in 0..NELEM {
                        r_surf[e] = surface_microbe_ce_ratio(fix.pcemic[e], me[e] / mc);
                    }
                    r_surf
                } else {
                    r_som1
                };
                let scale =
                    candec_scale_const::<F, NELEM>(mc, me, raw_c, c_net, r_recv, state.minerl[0]);
                let cf = raw_c * scale;
                if cf > F::EPS_GUARD {
                    let resp = cf * resp_f;
                    let net = cf - resp;
                    export_c_direct(&mut state.metabc[layer], &mut state.cum_c_out, resp);
                    transfer_c_direct(&mut state.metabc[layer], &mut state.som1c[layer], net);
                    state.month_resp += resp;
                    route_decomp_nutrients_direct::<F, NELEM>(
                        &mut state.metabe[layer],
                        &mut state.som1e[layer],
                        &mut state.minerl[0],
                        &mut state.cum_e_out,
                        &mut state.month_net_min,
                        &mut state.month_n_volat,
                        vlossg_clamped,
                        mc,
                        me,
                        cf,
                        net,
                        r_recv,
                    );
                }
            }
        }

        // --- 3. Dead Wood Decomposition (wood1, wood2, wood3) ---
        if has_any_wood {
            if has_wood0 {
                decomp_one_wood::<F, NELEM>(
                    &mut state.wood1c,
                    &mut state.wood1e,
                    &mut state.som1c[0],
                    &mut state.som1e[0],
                    &mut state.som2c,
                    &mut state.som2e,
                    &mut state.minerl[0],
                    &mut state.cum_c_out,
                    &mut state.cum_e_out,
                    &mut state.month_resp,
                    &mut state.month_net_min,
                    &mut state.month_n_volat,
                    fix,
                    vlossg_clamped,
                    0,
                    wd_lig[0],
                    wd_lig_exp[0],
                    decw[0],
                    defac_dt,
                    F::ONE,
                    r_som1,
                    r_som2,
                );
            }
            if has_wood1 {
                decomp_one_wood::<F, NELEM>(
                    &mut state.wood2c,
                    &mut state.wood2e,
                    &mut state.som1c[0],
                    &mut state.som1e[0],
                    &mut state.som2c,
                    &mut state.som2e,
                    &mut state.minerl[0],
                    &mut state.cum_c_out,
                    &mut state.cum_e_out,
                    &mut state.month_resp,
                    &mut state.month_net_min,
                    &mut state.month_n_volat,
                    fix,
                    vlossg_clamped,
                    0,
                    wd_lig[1],
                    wd_lig_exp[1],
                    decw[1],
                    defac_dt,
                    F::ONE,
                    r_som1,
                    r_som2,
                );
            }
            if has_wood2 {
                decomp_one_wood::<F, NELEM>(
                    &mut state.wood3c,
                    &mut state.wood3e,
                    &mut state.som1c[1],
                    &mut state.som1e[1],
                    &mut state.som2c,
                    &mut state.som2e,
                    &mut state.minerl[0],
                    &mut state.cum_c_out,
                    &mut state.cum_e_out,
                    &mut state.month_resp,
                    &mut state.month_net_min,
                    &mut state.month_n_volat,
                    fix,
                    vlossg_clamped,
                    1,
                    wd_lig[2],
                    wd_lig_exp[2],
                    decw[2],
                    defac_dt,
                    anerb,
                    r_som1,
                    r_som2,
                );
            }
        }

        // --- 4. Surface Active Microbes (som1c[0]) -> SOM2 ---
        let s10 = state.som1c[0].total();
        if s10 > F::EPS_GUARD {
            let s10e = state.som1e[0];
            let raw_c = (s10 * fix.dec3[0] * defac_dt).min(s10 * f(0.95));
            let c_net = raw_c * (F::ONE - resp_f_s10);
            let scale =
                candec_scale_const::<F, NELEM>(s10, s10e, raw_c, c_net, r_som2, state.minerl[0]);
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f_s10;
                let net = cf - resp;
                export_c_direct(&mut state.som1c[0], &mut state.cum_c_out, resp);
                transfer_c_direct(&mut state.som1c[0], &mut state.som2c, net);
                state.month_resp += resp;
                route_decomp_nutrients_direct::<F, NELEM>(
                    &mut state.som1e[0],
                    &mut state.som2e,
                    &mut state.minerl[0],
                    &mut state.cum_e_out,
                    &mut state.month_net_min,
                    &mut state.month_n_volat,
                    vlossg_clamped,
                    s10,
                    s10e,
                    cf,
                    net,
                    r_som2,
                );
            }
        }

        // --- 5. Soil Active SOM (som1c[1]) -> Resp + Organic Leach + SOM3 + SOM2 ---
        let s11 = state.som1c[1].total();
        if s11 > F::EPS_GUARD {
            let s11e = state.som1e[1];
            let raw_c =
                (s11 * fix.dec3[1] * defac_dt * derived.eftext * anerb * clt[0]).min(s11 * f(0.95));
            let scale = candec_scale_const::<F, NELEM>(
                s11,
                s11e,
                raw_c,
                raw_c * (to_s2_f_s11 + to_s3_f_s11),
                r_som2,
                state.minerl[0],
            );
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f_s11;
                let leach_c = cf * om_leach_f;
                let net_s3 = cf * to_s3_f_s11;
                let net_s2 = cf - resp - leach_c - net_s3;

                export_c_direct(&mut state.som1c[1], &mut state.cum_c_out, resp);
                if leach_c > F::ZERO {
                    export_c_direct(&mut state.som1c[1], &mut state.cum_c_out, leach_c);
                    state.month_leach_c += leach_c;
                    let frac_l = leach_c / s11;
                    for e in 0..NELEM {
                        let le = s11e[e] * frac_l;
                        if le > F::ZERO {
                            export_e_direct(&mut state.som1e[1][e], &mut state.cum_e_out[e], le);
                            state.month_leach_e[e] += le;
                        }
                    }
                }
                if net_s3 > F::ZERO {
                    transfer_c_direct(&mut state.som1c[1], &mut state.som3c, net_s3);
                    route_decomp_nutrients_direct::<F, NELEM>(
                        &mut state.som1e[1],
                        &mut state.som3e,
                        &mut state.minerl[0],
                        &mut state.cum_e_out,
                        &mut state.month_net_min,
                        &mut state.month_n_volat,
                        vlossg_clamped,
                        s11,
                        s11e,
                        cf * s11_s3_cflow_mult,
                        net_s3,
                        r_som3,
                    );
                }
                if net_s2 > F::ZERO {
                    transfer_c_direct(&mut state.som1c[1], &mut state.som2c, net_s2);
                    route_decomp_nutrients_direct::<F, NELEM>(
                        &mut state.som1e[1],
                        &mut state.som2e,
                        &mut state.minerl[0],
                        &mut state.cum_e_out,
                        &mut state.month_net_min,
                        &mut state.month_n_volat,
                        vlossg_clamped,
                        s11,
                        s11e,
                        cf * s11_s2_cflow_mult,
                        net_s2,
                        r_som2,
                    );
                }
                state.month_resp += resp;
            }
        }

        // --- 6. Slow SOM (som2c) -> Resp + SOM3 + SOM1[1] ---
        let s2 = state.som2c.total();
        if s2 > F::EPS_GUARD {
            let s2e = state.som2e;
            let raw_c = (s2 * fix.dec5 * defac_dt * anerb * clt[1]).min(s2 * f(0.95));
            let scale = candec_scale_const::<F, NELEM>(
                s2,
                s2e,
                raw_c,
                raw_c * (to_s1_f_s2 + to_s3_f_s2),
                r_som1,
                state.minerl[0],
            );
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f_s2;
                let net_s3 = cf * to_s3_f_s2;
                let net_s1 = cf - resp - net_s3;
                export_c_direct(&mut state.som2c, &mut state.cum_c_out, resp);
                state.month_resp += resp;

                if net_s3 > F::ZERO {
                    transfer_c_direct(&mut state.som2c, &mut state.som3c, net_s3);
                    route_decomp_nutrients_direct::<F, NELEM>(
                        &mut state.som2e,
                        &mut state.som3e,
                        &mut state.minerl[0],
                        &mut state.cum_e_out,
                        &mut state.month_net_min,
                        &mut state.month_n_volat,
                        vlossg_clamped,
                        s2,
                        s2e,
                        cf * s2_s3_share,
                        net_s3,
                        r_som3,
                    );
                }
                if net_s1 > F::ZERO {
                    transfer_c_direct(&mut state.som2c, &mut state.som1c[1], net_s1);
                    route_decomp_nutrients_direct::<F, NELEM>(
                        &mut state.som2e,
                        &mut state.som1e[1],
                        &mut state.minerl[0],
                        &mut state.cum_e_out,
                        &mut state.month_net_min,
                        &mut state.month_n_volat,
                        vlossg_clamped,
                        s2,
                        s2e,
                        cf * s2_s1_share,
                        net_s1,
                        r_som1,
                    );
                }
            }
        }

        // --- 7. Passive SOM (som3c) -> Resp + SOM1[1] ---
        let s3 = state.som3c.total();
        if s3 > F::EPS_GUARD {
            let s3e = state.som3e;
            let raw_c = (s3 * fix.dec4 * defac_dt * anerb * clt[2]).min(s3 * f(0.95));
            let resp_f = fix.p3co2;
            let c_net = raw_c * (F::ONE - resp_f);
            let scale =
                candec_scale_const::<F, NELEM>(s3, s3e, raw_c, c_net, r_som1, state.minerl[0]);
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f;
                let net = cf - resp;
                export_c_direct(&mut state.som3c, &mut state.cum_c_out, resp);
                transfer_c_direct(&mut state.som3c, &mut state.som1c[1], net);
                state.month_resp += resp;
                route_decomp_nutrients_direct::<F, NELEM>(
                    &mut state.som3e,
                    &mut state.som1e[1],
                    &mut state.minerl[0],
                    &mut state.cum_e_out,
                    &mut state.month_net_min,
                    &mut state.month_n_volat,
                    vlossg_clamped,
                    s3,
                    s3e,
                    cf,
                    net,
                    r_som1,
                );
            }
        }
    }
}
