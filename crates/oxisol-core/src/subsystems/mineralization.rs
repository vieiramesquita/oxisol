//! Residue partitioning (`partit`), SOM target C:E ratio interpolation (`varat`),
//! decomposition nutrient availability check (`candec`), mineralization/immobilization,
//! and atmospheric/non-symbiotic N/S deposition inputs.
//!
//! Specifications: `docs/spec/01_pools_and_mass_balance.md` §6 and
//! `docs/spec/03_decomposition_and_som.md` §4.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{FixParams, SiteParams, MAX_ELEM};
use crate::state::CellState;

/// Partition incoming plant residue or organic amendment `(c_amt, e_amts)` with
/// lignin fraction `lignin` into surface (`layer=0`) or soil (`layer=1`) structural
/// and metabolic litter pools via `FlowLedger`.
///
/// Preserves exact mass conservation for Carbon and all active elements.
#[allow(clippy::too_many_arguments)]
pub fn partition_residue<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    layer: usize,
    from_c: PoolId,
    from_e: [PoolId; MAX_ELEM],
    c_amt: F,
    e_amts: [F; MAX_ELEM],
    lignin: F,
    lab_frac: Option<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    if c_amt <= F::EPS_GUARD {
        return;
    }
    let f = F::from_f64;
    let lig = lignin.clamp_val(F::ZERO, f(0.95));

    // Lignin-to-N ratio determines metabolic fraction: fm = spl[0] - spl[1] * (L / (N/C)).
    let r_n = if e_amts[0] > F::EPS_GUARD {
        (e_amts[0] / c_amt).max(f(1.0e-4))
    } else {
        f(0.01)
    };
    let ln_ratio = lig / r_n;
    let max_fm = (F::ONE - lig).max(F::ZERO);
    let fm = (fix.spl[0] - fix.spl[1] * ln_ratio).clamp_val(F::ZERO, max_fm);

    let c_metab = c_amt * fm;
    let c_struc = c_amt - c_metab;

    // Update structural lignin fraction as weighted average.
    let old_sc = state.strucc[layer].total();
    let new_sc = old_sc + c_struc;
    if new_sc > F::EPS_GUARD {
        let new_lig = (old_sc * state.strlig[layer] + c_amt * lig) / new_sc;
        state.strlig[layer] = new_lig.clamp_val(F::ZERO, F::ONE);
    }

    // Route carbon.
    if c_struc > F::ZERO {
        ledger.schedule_with_iso(from_c, PoolId::StrucC(layer), c_struc, lab_frac);
    }
    if c_metab > F::ZERO {
        ledger.schedule_with_iso(from_c, PoolId::MetabC(layer), c_metab, lab_frac);
    }

    // Route nutrients: structural takes up to c_struc / rcestr[e], remainder to metabolic.
    let nelem = elements.nelem();
    for e in 0..nelem {
        let avail = e_amts[e].max(F::ZERO);
        if avail <= F::ZERO {
            continue;
        }
        let target_struc_e = if fix.rcestr[e] > F::EPS_GUARD {
            c_struc / fix.rcestr[e]
        } else {
            F::ZERO
        };
        let e_struc = target_struc_e.min(avail);
        let e_metab = (avail - e_struc).max(F::ZERO);
        if e_struc > F::ZERO {
            ledger.schedule(from_e[e], PoolId::StrucE(layer, e), e_struc);
        }
        if e_metab > F::ZERO {
            ledger.schedule(from_e[e], PoolId::MetabE(layer, e), e_metab);
        }
    }
}

/// Interpolate target C:E ratio for a receiving SOM compartment (`varat = [max, min, ref_minerl]`)
/// based on available top-layer mineral nutrient `minerl0`.
#[inline]
pub fn target_ce_ratio<F: Float>(varat: [F; 3], minerl0: F) -> F {
    let rmax = varat[0];
    let rmin = varat[1];
    let mref = varat[2].max(F::EPS_GUARD);
    let frac = (minerl0 / mref).clamp_val(F::ZERO, F::ONE);
    // High mineral nutrient -> tighter (lower) C:E ratio toward rmin.
    (rmax - frac * (rmax - rmin)).max(F::from_f64(1.0))
}

/// Compute target C:E ratio for surface microbes (`som1c[0]`) using `fix.pcemic`
/// as a function of decomposing residue's E:C ratio (Manual §3.4 p. 3-8, A2-11).
#[inline]
pub fn surface_microbe_ce_ratio<F: Float>(pcemic: [F; 3], donor_e_to_c: F) -> F {
    let rmax = pcemic[0];
    let rmin = pcemic[1];
    let thresh = pcemic[2].max(F::EPS_GUARD);
    let frac = (donor_e_to_c / thresh).clamp_val(F::ZERO, F::ONE);
    (rmax - frac * (rmax - rmin)).max(F::from_f64(1.0))
}

/// Check whether decomposition of `c_flow` from donor pool `(donor_c, donor_e)` into
/// receiver pool with net carbon `c_net` and target C:E ratios `target_ce` can proceed
/// given available top-layer mineral nutrients `minerl0`.
///
/// Returns a scaling factor in `[0.0, 1.0]` so `minerl[0][e]` never goes negative.
pub fn candec_scale<F: Float>(
    donor_c: F,
    donor_e: [F; MAX_ELEM],
    c_flow: F,
    c_net: F,
    target_ce: [F; MAX_ELEM],
    minerl0: [F; MAX_ELEM],
    elements: ElementMode,
) -> F {
    if donor_c <= F::EPS_GUARD || c_flow <= F::EPS_GUARD {
        return F::ZERO;
    }
    let mut scale = F::ONE;
    let frac_donor = (c_flow / donor_c).clamp_val(F::ZERO, F::ONE);
    for e in 0..elements.nelem() {
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

/// Record nutrient transfer, mineralization (with N volatilization `vlossg`), or
/// immobilization for an organic matter decomposition flow.
#[allow(clippy::too_many_arguments)]
pub fn route_decomp_nutrients<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    donor_c: F,
    donor_e: [F; MAX_ELEM],
    donor_e_pools: [PoolId; MAX_ELEM],
    recv_e_pools: [PoolId; MAX_ELEM],
    c_flow: F,
    c_net: F,
    target_ce: [F; MAX_ELEM],
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    if donor_c <= F::EPS_GUARD || c_flow <= F::EPS_GUARD {
        return;
    }
    let frac_donor = (c_flow / donor_c).clamp_val(F::ZERO, F::ONE);
    for e in 0..elements.nelem() {
        let e_rel = donor_e[e] * frac_donor;
        let e_req = if target_ce[e] > F::EPS_GUARD {
            c_net / target_ce[e]
        } else {
            F::ZERO
        };
        if e_rel >= e_req {
            // Net mineralization with gross-mineralization N volatilization (Manual p. 157).
            let to_recv = e_req;
            let net_min = e_rel - e_req;
            if to_recv > F::ZERO {
                ledger.schedule(donor_e_pools[e], recv_e_pools[e], to_recv);
            }
            if net_min > F::ZERO {
                if e == 0 && fix.vlossg > F::ZERO {
                    // Gross mineralization volatilization (Manual §3.5): vlossg * gross N released,
                    // capped at net_min to maintain strict non-negativity.
                    let volat =
                        (e_rel * fix.vlossg.clamp_val(F::ZERO, F::from_f64(0.5))).min(net_min);
                    let to_min = net_min - volat;
                    ledger.schedule(donor_e_pools[e], PoolId::AtmosphereE(0), volat);
                    if to_min > F::ZERO {
                        ledger.schedule(donor_e_pools[e], PoolId::Minerl(0, 0), to_min);
                    }
                    state.month_n_volat += volat;
                    state.month_net_min[0] += to_min;
                } else {
                    ledger.schedule(donor_e_pools[e], PoolId::Minerl(0, e), net_min);
                    state.month_net_min[e] += net_min;
                }
            }
        } else {
            // Net immobilization from top mineral layer.
            let immob = (e_req - e_rel).min(state.minerl[0][e].max(F::ZERO));
            if e_rel > F::ZERO {
                ledger.schedule(donor_e_pools[e], recv_e_pools[e], e_rel);
            }
            if immob > F::ZERO {
                ledger.schedule(PoolId::Minerl(0, e), recv_e_pools[e], immob);
                state.month_net_min[e] -= immob;
            }
        }
    }
}

/// Apply monthly atmospheric N & S deposition and non-symbiotic soil N fixation.
///
/// Follows CENTURY V4 annual-to-monthly precipitation weighting (Manual Ch. 3 §3.4):
/// - Atmospheric N deposition: annual `epnfa[0] + epnfa[1] * min(ann_ppt, 80 cm)` distributed
///   proportionally to `precip_cm / ann_ppt`.
/// - Non-symbiotic soil N fixation: annual `min(epnfs[1] * ann_ppt, 1.0)` when `epnfs[0] >= 1.0`
///   (threshold form used in VEMAP `lu*.100`) or `epnfs[0] + epnfs[1] * ann_ppt` when `epnfs[0] < 1.0`,
///   distributed proportionally to `precip_cm / ann_ppt`.
pub fn apply_atmospheric_inputs<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    site: &SiteParams<F>,
    precip_cm: F,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let f = F::from_f64;
    let ann_ppt = state.ann_ppt.max(f(1.0));
    let ppt_frac = (precip_cm.max(F::ZERO) / ann_ppt).clamp_val(F::ZERO, F::ONE);

    // Atmospheric N deposition (g N / m2 / yr -> monthly via precip fraction).
    let ann_n_dep = (site.epnfa[0] + site.epnfa[1] * ann_ppt.min(f(80.0))).max(F::ZERO);
    let n_dep = ann_n_dep * ppt_frac;
    if n_dep > F::ZERO {
        ledger.schedule(PoolId::AtmosphereE(0), PoolId::Minerl(0, 0), n_dep);
    }

    // Non-symbiotic soil N fixation (g N / m2 / yr -> monthly via precip fraction).
    // When EPNFS(1) >= 1.0 (e.g. 30.0 cm in VEMAP lu*.100), EPNFS(1) is the annual AET
    // threshold above which fixation scales linearly as EPNFS(2) * max(0, ann_aet - EPNFS(1)).
    // When EPNFS(1) < 1.0, uses linear precipitation regression EPNFS(1) + EPNFS(2) * ann_ppt.
    let ann_n_fix = if site.epnfs[0] >= F::ONE {
        let excess_aet = (state.ann_aet - site.epnfs[0]).max(F::ZERO);
        (site.epnfs[1] * excess_aet).clamp_val(F::ZERO, F::ONE)
    } else {
        (site.epnfs[0] + site.epnfs[1] * ann_ppt).clamp_val(F::ZERO, f(2.0))
    };
    let n_fix = ann_n_fix * ppt_frac;
    if n_fix > F::ZERO {
        ledger.schedule(PoolId::AtmosphereE(0), PoolId::Minerl(0, 0), n_fix);
        state.month_n_fix += n_fix;
    }

    // Atmospheric S deposition when FullCnps is active.
    if elements.nelem() >= 3 {
        let ann_s_dep = (site.satmos[0] + site.satmos[1] * ann_ppt).max(F::ZERO);
        let s_dep = ann_s_dep * ppt_frac;
        if s_dep > F::ZERO {
            ledger.schedule(PoolId::AtmosphereE(2), PoolId::Minerl(0, 2), s_dep);
        }
    }

    // Excess mineral N gaseous loss (`volex`, Manual §3.4 p. 3-8):
    // Monthly loss of `vlosse / 12` from top-layer mineral N.
    let n0 = state.minerl[0][0];
    if n0 > F::ZERO && fix.vlosse > F::ZERO {
        let volex = (n0 * (fix.vlosse.clamp_val(F::ZERO, f(0.5)) / f(12.0))).min(n0);
        if volex > F::ZERO {
            ledger.schedule(PoolId::Minerl(0, 0), PoolId::AtmosphereE(0), volex);
            state.month_n_volat += volex;
        }
    }

    ledger.commit(state);
}
