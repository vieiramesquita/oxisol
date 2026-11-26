//! Soil organic matter, plant litter, and dead wood decomposition (`defac`, `anerb`,
//! 4-substep monthly kinetics, texture stabilization, and heterotrophic respiration).
//!
//! Specification: `docs/spec/03_decomposition_and_som.md`.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{CellDerivedParams, FixParams, TreeParams, MAX_ELEM};
use crate::state::CellState;
use crate::subsystems::mineralization::{
    candec_scale, route_decomp_nutrients, surface_microbe_ce_ratio, target_ce_ratio,
};

/// Generalized Poisson Density Function (`gpdf`) used for temperature responses.
#[inline]
pub fn gpdf<F: Float>(x: F, x_opt: F, x_max: F, a: F, b: F) -> F {
    if x >= x_max {
        return F::ZERO;
    }
    let denom = (x_max - x_opt).max(F::EPS_GUARD);
    let frac = ((x_max - x) / denom).max(F::ZERO);
    if frac <= F::ZERO {
        return F::ZERO;
    }
    let exponent = (a / b.max(F::EPS_GUARD)) * (F::ONE - frac.powf(b));
    (frac.powf(a) * exponent.exp()).clamp_val(F::ZERO, F::ONE)
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
#[inline]
pub fn anaerobic_factor<F: Float>(rel_water: F, drain: F, fix: &FixParams<F>) -> F {
    let f = F::from_f64;
    if drain >= f(0.999) {
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
    let anerb = anaerobic_factor(rel_water_surface, derived.drain, fix);
    state.defac = defac;
    state.anerb = anerb;
    (defac, anerb)
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
    ledger: &mut FlowLedger<F>,
) {
    let f = F::from_f64;
    let nsub = fix.ntspm.clamp(1, 12) as usize;
    let dt = F::ONE / f((12 * nsub) as f64);

    let defac = state.defac;
    let anerb = state.anerb;
    let clt = state.clteff;
    let decw = tree.map(|t| t.decw).unwrap_or([f(1.5), f(0.5), f(0.6)]);

    // Precompute lignin inhibition exponentials once per month (strlig and wdlig are
    // invariant across the `ntspm` decomposition sub-steps).
    let str_lig = [
        state.strlig[0].clamp_val(F::ZERO, F::ONE),
        state.strlig[1].clamp_val(F::ZERO, F::ONE),
    ];
    let str_lig_exp = [
        (-fix.pligst[0] * str_lig[0]).exp(),
        (-fix.pligst[1] * str_lig[1]).exp(),
    ];
    let wd_lig = [
        state.wdlig[0].clamp_val(F::ZERO, F::ONE),
        state.wdlig[1].clamp_val(F::ZERO, F::ONE),
        state.wdlig[2].clamp_val(F::ZERO, F::ONE),
    ];
    let wd_lig_exp = [
        (-fix.pligst[0] * wd_lig[0]).exp(),
        (-fix.pligst[0] * wd_lig[1]).exp(),
        (-fix.pligst[1] * wd_lig[2]).exp(),
    ];

    for _ in 0..nsub {
        // Compute target C:E ratios for SOM1, SOM2, SOM3 from current top-layer mineral pools.
        let mut r_som1 = [F::ONE; MAX_ELEM];
        let mut r_som2 = [F::ONE; MAX_ELEM];
        let mut r_som3 = [F::ONE; MAX_ELEM];
        for e in 0..elements.nelem() {
            let m0 = state.minerl[0][e];
            r_som1[e] = target_ce_ratio(fix.varat1[e], m0);
            r_som2[e] = target_ce_ratio(fix.varat2[e], m0);
            r_som3[e] = target_ce_ratio(fix.varat3[e], m0);
        }

        // --- 1. Structural Litter Decomposition (surface=0, soil=1) ---
        for layer in 0..2 {
            let sc = state.strucc[layer].total();
            if sc > F::EPS_GUARD {
                let eff_c = sc.min(fix.strmax[layer]);
                let an_mult = if layer == 1 { anerb } else { F::ONE };
                let clt_mult = if layer == 1 { clt[3] } else { F::ONE };
                // Manual A2-32: clteff[3] is soil structural decomposition multiplier.
                let k = fix.dec1[layer] * defac * str_lig_exp[layer] * an_mult * clt_mult;
                let raw_c = (eff_c * k * dt).min(sc * f(0.95));

                let lig = str_lig[layer];
                let c_lig = raw_c * lig;
                let c_nonlig = raw_c - c_lig;

                // Lignin -> SOM2 (respired fraction rsplig).
                if c_lig > F::EPS_GUARD {
                    let c_net = c_lig * (F::ONE - fix.rsplig);
                    let scale = candec_scale(
                        sc,
                        state.struce[layer],
                        c_lig,
                        c_net,
                        r_som2,
                        state.minerl[0],
                        elements,
                    );
                    let cf = c_lig * scale;
                    if cf > F::EPS_GUARD {
                        let resp = cf * fix.rsplig;
                        let net = cf - resp;
                        ledger.schedule(PoolId::StrucC(layer), PoolId::AtmosphereC, resp);
                        ledger.schedule(PoolId::StrucC(layer), PoolId::Som2C, net);
                        state.month_resp += resp;
                        route_decomp_nutrients(
                            state,
                            fix,
                            sc,
                            state.struce[layer],
                            [
                                PoolId::StrucE(layer, 0),
                                PoolId::StrucE(layer, 1),
                                PoolId::StrucE(layer, 2),
                            ],
                            [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
                            cf,
                            net,
                            r_som2,
                            elements,
                            ledger,
                        );
                    }
                }

                // Non-lignin -> SOM1[layer] (respired fraction ps1co2[layer]).
                // Surface layer (0) uses `pcemic` based on donor E:C; soil layer (1) uses `varat1`.
                if c_nonlig > F::EPS_GUARD {
                    let resp_f = fix.ps1co2[layer];
                    let c_net = c_nonlig * (F::ONE - resp_f);
                    let r_recv = if layer == 0 {
                        let mut r_surf = [F::ONE; MAX_ELEM];
                        for e in 0..elements.nelem() {
                            r_surf[e] =
                                surface_microbe_ce_ratio(fix.pcemic[e], state.struce[0][e] / sc);
                        }
                        r_surf
                    } else {
                        r_som1
                    };
                    let scale = candec_scale(
                        sc,
                        state.struce[layer],
                        c_nonlig,
                        c_net,
                        r_recv,
                        state.minerl[0],
                        elements,
                    );
                    let cf = c_nonlig * scale;
                    if cf > F::EPS_GUARD {
                        let resp = cf * resp_f;
                        let net = cf - resp;
                        ledger.schedule(PoolId::StrucC(layer), PoolId::AtmosphereC, resp);
                        ledger.schedule(PoolId::StrucC(layer), PoolId::Som1C(layer), net);
                        state.month_resp += resp;
                        route_decomp_nutrients(
                            state,
                            fix,
                            sc,
                            state.struce[layer],
                            [
                                PoolId::StrucE(layer, 0),
                                PoolId::StrucE(layer, 1),
                                PoolId::StrucE(layer, 2),
                            ],
                            [
                                PoolId::Som1E(layer, 0),
                                PoolId::Som1E(layer, 1),
                                PoolId::Som1E(layer, 2),
                            ],
                            cf,
                            net,
                            r_recv,
                            elements,
                            ledger,
                        );
                    }
                }
            }
        }
        ledger.commit(state);

        // --- 2. Metabolic Litter Decomposition (surface=0, soil=1) ---
        for layer in 0..2 {
            let mc = state.metabc[layer].total();
            if mc > F::EPS_GUARD {
                let an_mult = if layer == 1 { anerb } else { F::ONE };
                let k = fix.dec2[layer] * defac * an_mult;
                let raw_c = (mc * k * dt).min(mc * f(0.95));
                let resp_f = fix.pmco2[layer];
                let c_net = raw_c * (F::ONE - resp_f);
                let r_recv = if layer == 0 {
                    let mut r_surf = [F::ONE; MAX_ELEM];
                    for e in 0..elements.nelem() {
                        r_surf[e] =
                            surface_microbe_ce_ratio(fix.pcemic[e], state.metabe[0][e] / mc);
                    }
                    r_surf
                } else {
                    r_som1
                };
                let scale = candec_scale(
                    mc,
                    state.metabe[layer],
                    raw_c,
                    c_net,
                    r_recv,
                    state.minerl[0],
                    elements,
                );
                let cf = raw_c * scale;
                if cf > F::EPS_GUARD {
                    let resp = cf * resp_f;
                    let net = cf - resp;
                    ledger.schedule(PoolId::MetabC(layer), PoolId::AtmosphereC, resp);
                    ledger.schedule(PoolId::MetabC(layer), PoolId::Som1C(layer), net);
                    state.month_resp += resp;
                    route_decomp_nutrients(
                        state,
                        fix,
                        mc,
                        state.metabe[layer],
                        [
                            PoolId::MetabE(layer, 0),
                            PoolId::MetabE(layer, 1),
                            PoolId::MetabE(layer, 2),
                        ],
                        [
                            PoolId::Som1E(layer, 0),
                            PoolId::Som1E(layer, 1),
                            PoolId::Som1E(layer, 2),
                        ],
                        cf,
                        net,
                        r_recv,
                        elements,
                        ledger,
                    );
                }
            }
        }
        ledger.commit(state);

        // --- 3. Dead Wood Decomposition (wood1, wood2, wood3) ---
        let wood_specs: [(PoolId, [PoolId; 3], usize, F, F); 3] = [
            (
                PoolId::Wood1C,
                [PoolId::Wood1E(0), PoolId::Wood1E(1), PoolId::Wood1E(2)],
                0,
                decw[0],
                F::ONE,
            ),
            (
                PoolId::Wood2C,
                [PoolId::Wood2E(0), PoolId::Wood2E(1), PoolId::Wood2E(2)],
                0,
                decw[1],
                F::ONE,
            ),
            (
                PoolId::Wood3C,
                [PoolId::Wood3E(0), PoolId::Wood3E(1), PoolId::Wood3E(2)],
                1,
                decw[2],
                anerb,
            ),
        ];
        for (idx, (w_c_id, w_e_ids, som_lyr, dec_rate, an_m)) in wood_specs.into_iter().enumerate()
        {
            let (wc, we) = match idx {
                0 => (state.wood1c.total(), state.wood1e),
                1 => (state.wood2c.total(), state.wood2e),
                _ => (state.wood3c.total(), state.wood3e),
            };
            if wc > F::EPS_GUARD {
                let lig = wd_lig[idx];
                let k = dec_rate * defac * wd_lig_exp[idx] * an_m;
                let raw_c = (wc * k * dt).min(wc * f(0.95));
                let c_lig = raw_c * lig;
                let c_nonlig = raw_c - c_lig;

                if c_lig > F::EPS_GUARD {
                    let c_net = c_lig * (F::ONE - fix.rsplig);
                    let scale =
                        candec_scale(wc, we, c_lig, c_net, r_som2, state.minerl[0], elements);
                    let cf = c_lig * scale;
                    if cf > F::EPS_GUARD {
                        let resp = cf * fix.rsplig;
                        let net = cf - resp;
                        ledger.schedule(w_c_id, PoolId::AtmosphereC, resp);
                        ledger.schedule(w_c_id, PoolId::Som2C, net);
                        state.month_resp += resp;
                        route_decomp_nutrients(
                            state,
                            fix,
                            wc,
                            we,
                            w_e_ids,
                            [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
                            cf,
                            net,
                            r_som2,
                            elements,
                            ledger,
                        );
                    }
                }
                if c_nonlig > F::EPS_GUARD {
                    let resp_f = fix.ps1co2[som_lyr];
                    let c_net = c_nonlig * (F::ONE - resp_f);
                    let scale =
                        candec_scale(wc, we, c_nonlig, c_net, r_som1, state.minerl[0], elements);
                    let cf = c_nonlig * scale;
                    if cf > F::EPS_GUARD {
                        let resp = cf * resp_f;
                        let net = cf - resp;
                        ledger.schedule(w_c_id, PoolId::AtmosphereC, resp);
                        ledger.schedule(w_c_id, PoolId::Som1C(som_lyr), net);
                        state.month_resp += resp;
                        route_decomp_nutrients(
                            state,
                            fix,
                            wc,
                            we,
                            w_e_ids,
                            [
                                PoolId::Som1E(som_lyr, 0),
                                PoolId::Som1E(som_lyr, 1),
                                PoolId::Som1E(som_lyr, 2),
                            ],
                            cf,
                            net,
                            r_som1,
                            elements,
                            ledger,
                        );
                    }
                }
            }
        }
        ledger.commit(state);

        // --- 4. Surface Active Microbes (som1c[0]) -> SOM2 ---
        let s10 = state.som1c[0].total();
        if s10 > F::EPS_GUARD {
            let k = fix.dec3[0] * defac;
            let raw_c = (s10 * k * dt).min(s10 * f(0.95));
            let resp_f = fix.p1co2a[0].clamp_val(f(0.05), f(0.95));
            let c_net = raw_c * (F::ONE - resp_f);
            let scale = candec_scale(
                s10,
                state.som1e[0],
                raw_c,
                c_net,
                r_som2,
                state.minerl[0],
                elements,
            );
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f;
                let net = cf - resp;
                ledger.schedule(PoolId::Som1C(0), PoolId::AtmosphereC, resp);
                ledger.schedule(PoolId::Som1C(0), PoolId::Som2C, net);
                state.month_resp += resp;
                route_decomp_nutrients(
                    state,
                    fix,
                    s10,
                    state.som1e[0],
                    [
                        PoolId::Som1E(0, 0),
                        PoolId::Som1E(0, 1),
                        PoolId::Som1E(0, 2),
                    ],
                    [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
                    cf,
                    net,
                    r_som2,
                    elements,
                    ledger,
                );
            }
        }
        ledger.commit(state);

        // --- 5. Soil Active SOM (som1c[1]) -> Resp + Organic Leach + SOM3 + SOM2 ---
        let s11 = state.som1c[1].total();
        if s11 > F::EPS_GUARD {
            let k = fix.dec3[1] * defac * derived.eftext * anerb * clt[0];
            let raw_c = (s11 * k * dt).min(s11 * f(0.95));
            let resp_f = derived.p1co2_soil;
            // Organic matter leaching scales linearly with 30 cm water flux up to omlech[2] (60 cm).
            let om_leach_f = if leach_flux > F::ZERO {
                ((leach_flux / fix.omlech[2].max(f(0.1))).clamp_val(F::ZERO, F::ONE)
                    * derived.omlech_frac)
                    .min(F::ONE - resp_f)
            } else {
                F::ZERO
            };
            let to_s3_f = derived
                .fps1s3
                .min((F::ONE - resp_f - om_leach_f).max(F::ZERO));
            let to_s2_f = (F::ONE - resp_f - om_leach_f - to_s3_f).max(F::ZERO);

            // Check combined immobilization demand into SOM2/SOM3.
            let c_to_s2 = raw_c * to_s2_f;
            let c_to_s3 = raw_c * to_s3_f;
            let scale = candec_scale(
                s11,
                state.som1e[1],
                raw_c,
                c_to_s2 + c_to_s3,
                r_som2,
                state.minerl[0],
                elements,
            );
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f;
                let leach_c = cf * om_leach_f;
                let net_s3 = cf * to_s3_f;
                let net_s2 = cf - resp - leach_c - net_s3;

                ledger.schedule(PoolId::Som1C(1), PoolId::AtmosphereC, resp);
                if leach_c > F::ZERO {
                    ledger.schedule(PoolId::Som1C(1), PoolId::LeachingC, leach_c);
                    state.month_leach_c += leach_c;
                    // Export organic nutrients proportional to leached C.
                    let frac_l = leach_c / s11;
                    for e in 0..elements.nelem() {
                        let le = state.som1e[1][e] * frac_l;
                        if le > F::ZERO {
                            ledger.schedule(PoolId::Som1E(1, e), PoolId::LeachingE(e), le);
                            state.month_leach_e[e] += le;
                        }
                    }
                }
                if net_s3 > F::ZERO {
                    ledger.schedule(PoolId::Som1C(1), PoolId::Som3C, net_s3);
                    route_decomp_nutrients(
                        state,
                        fix,
                        s11,
                        state.som1e[1],
                        [
                            PoolId::Som1E(1, 0),
                            PoolId::Som1E(1, 1),
                            PoolId::Som1E(1, 2),
                        ],
                        [PoolId::Som3E(0), PoolId::Som3E(1), PoolId::Som3E(2)],
                        cf * to_s3_f / (F::ONE - resp_f - om_leach_f).max(F::EPS_GUARD)
                            * (F::ONE - om_leach_f),
                        net_s3,
                        r_som3,
                        elements,
                        ledger,
                    );
                }
                if net_s2 > F::ZERO {
                    ledger.schedule(PoolId::Som1C(1), PoolId::Som2C, net_s2);
                    route_decomp_nutrients(
                        state,
                        fix,
                        s11,
                        state.som1e[1],
                        [
                            PoolId::Som1E(1, 0),
                            PoolId::Som1E(1, 1),
                            PoolId::Som1E(1, 2),
                        ],
                        [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
                        cf * to_s2_f / (F::ONE - resp_f - om_leach_f).max(F::EPS_GUARD)
                            * (F::ONE - om_leach_f),
                        net_s2,
                        r_som2,
                        elements,
                        ledger,
                    );
                }
                state.month_resp += resp;
            }
        }
        ledger.commit(state);

        // --- 6. Slow SOM (som2c) -> Resp + SOM3 + SOM1[1] ---
        let s2 = state.som2c.total();
        if s2 > F::EPS_GUARD {
            let k = fix.dec5 * defac * anerb * clt[1];
            let raw_c = (s2 * k * dt).min(s2 * f(0.95));
            let resp_f = fix.p2co2;
            let to_s3_f = derived.fps2s3.min((F::ONE - resp_f).max(F::ZERO));
            let to_s1_f = (F::ONE - resp_f - to_s3_f).max(F::ZERO);
            let scale = candec_scale(
                s2,
                state.som2e,
                raw_c,
                raw_c * (to_s1_f + to_s3_f),
                r_som1,
                state.minerl[0],
                elements,
            );
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f;
                let net_s3 = cf * to_s3_f;
                let net_s1 = cf - resp - net_s3;
                ledger.schedule(PoolId::Som2C, PoolId::AtmosphereC, resp);
                state.month_resp += resp;

                let share_denom = (to_s1_f + to_s3_f).max(F::EPS_GUARD);
                if net_s3 > F::ZERO {
                    ledger.schedule(PoolId::Som2C, PoolId::Som3C, net_s3);
                    route_decomp_nutrients(
                        state,
                        fix,
                        s2,
                        state.som2e,
                        [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
                        [PoolId::Som3E(0), PoolId::Som3E(1), PoolId::Som3E(2)],
                        cf * (to_s3_f / share_denom),
                        net_s3,
                        r_som3,
                        elements,
                        ledger,
                    );
                }
                if net_s1 > F::ZERO {
                    ledger.schedule(PoolId::Som2C, PoolId::Som1C(1), net_s1);
                    route_decomp_nutrients(
                        state,
                        fix,
                        s2,
                        state.som2e,
                        [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
                        [
                            PoolId::Som1E(1, 0),
                            PoolId::Som1E(1, 1),
                            PoolId::Som1E(1, 2),
                        ],
                        cf * (to_s1_f / share_denom),
                        net_s1,
                        r_som1,
                        elements,
                        ledger,
                    );
                }
            }
        }
        ledger.commit(state);

        // --- 7. Passive SOM (som3c) -> Resp + SOM1[1] ---
        let s3 = state.som3c.total();
        if s3 > F::EPS_GUARD {
            let k = fix.dec4 * defac * anerb * clt[2];
            let raw_c = (s3 * k * dt).min(s3 * f(0.95));
            let resp_f = fix.p3co2;
            let c_net = raw_c * (F::ONE - resp_f);
            let scale = candec_scale(
                s3,
                state.som3e,
                raw_c,
                c_net,
                r_som1,
                state.minerl[0],
                elements,
            );
            let cf = raw_c * scale;
            if cf > F::EPS_GUARD {
                let resp = cf * resp_f;
                let net = cf - resp;
                ledger.schedule(PoolId::Som3C, PoolId::AtmosphereC, resp);
                ledger.schedule(PoolId::Som3C, PoolId::Som1C(1), net);
                state.month_resp += resp;
                route_decomp_nutrients(
                    state,
                    fix,
                    s3,
                    state.som3e,
                    [PoolId::Som3E(0), PoolId::Som3E(1), PoolId::Som3E(2)],
                    [
                        PoolId::Som1E(1, 0),
                        PoolId::Som1E(1, 1),
                        PoolId::Som1E(1, 2),
                    ],
                    cf,
                    net,
                    r_som1,
                    elements,
                    ledger,
                );
            }
        }
        ledger.commit(state);
    }
}
