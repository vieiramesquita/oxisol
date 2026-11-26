//! Phosphorus (P) and Sulfur (S) geochemical cycles (`pschem`, `fsfunc`, parent
//! weathering, labile <-> secondary equilibrium, secondary <-> occluded P, and
//! biochemical ester-P/S mineralization).
//!
//! Specification: `docs/spec/05_phosphorus_and_sulfur.md`.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{CellDerivedParams, FixParams, SiteParams};
use crate::state::CellState;

/// Soil pH and texture-dependent P availability/sorption response (`fsfunc`).
#[inline]
pub fn p_solubility_factor<F: Float>(ph: F, texesp_2: F, fix: &FixParams<F>) -> F {
    let f = F::from_f64;
    // Optimal P availability around neutral pH (6.0-7.0); lower in strongly acidic or alkaline soils.
    let ph_opt = fix.phesp[2].max(f(5.5));
    let ph_dist = (ph - ph_opt).abs();
    let ph_mod = (F::ONE - f(0.15) * ph_dist).clamp_val(f(0.2), F::ONE);
    (ph_mod / texesp_2.max(f(0.2))).clamp_val(f(0.1), f(2.0))
}

/// Execute monthly inorganic P & S geochemical transformations and biochemical
/// mineralization from Slow & Passive SOM.
pub fn step_phosphorus_sulfur<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    site: &SiteParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let nelem = elements.nelem();
    if nelem < 2 {
        return;
    }

    let f = F::from_f64;
    let dt = f(1.0 / 12.0);
    let defac = state.defac;

    // --- Phosphorus Geochemistry (e = 1) ---
    {
        let e = 1usize;
        let sol = p_solubility_factor(derived.ph, derived.texesp_2, fix);

        // 1. Parent material weathering -> labile mineral P (layer 0).
        let par_flow =
            (state.parent[e] * derived.pparmn_2 * defac * dt).clamp_val(F::ZERO, state.parent[e]);
        if par_flow > F::ZERO {
            ledger.schedule(PoolId::Parent(e), PoolId::Minerl(0, e), par_flow);
        }

        // 2. Reversible labile <-> secondary P sorption/desorption.
        let sorb_rate = (fix.pmnsec[e] * derived.texesp_2 * site.pslsrb.max(f(0.1)) * dt)
            .clamp_val(F::ZERO, f(0.5));
        let lab_to_sec = (state.minerl[0][e] * sorb_rate).clamp_val(F::ZERO, state.minerl[0][e]);
        if lab_to_sec > F::ZERO {
            ledger.schedule(PoolId::Minerl(0, e), PoolId::Secndy(e), lab_to_sec);
        }

        let sec_to_lab = (state.secndy[e] * derived.psecmn_2 * sol * defac * dt)
            .clamp_val(F::ZERO, state.secndy[e]);
        if sec_to_lab > F::ZERO {
            ledger.schedule(PoolId::Secndy(e), PoolId::Minerl(0, e), sec_to_lab);
        }

        // 3. Secondary <-> Occluded P exchange.
        let sec_to_occ =
            (state.secndy[e] * fix.psecoc * defac * dt).clamp_val(F::ZERO, state.secndy[e]);
        if sec_to_occ > F::ZERO {
            ledger.schedule(PoolId::Secndy(e), PoolId::OccludP, sec_to_occ);
        }
        let occ_to_sec =
            (state.occlud * fix.psecoc * f(0.25) * defac * dt).clamp_val(F::ZERO, state.occlud);
        if occ_to_sec > F::ZERO {
            ledger.schedule(PoolId::OccludP, PoolId::Secndy(e), occ_to_sec);
        }

        // 4. Biochemical phosphatase mineralization from Slow & Passive SOM when labile P is low.
        let biop_drive = (F::ONE - (state.minerl[0][e] / f(2.0)).clamp_val(F::ZERO, F::ONE))
            * defac
            * dt
            * f(0.02);
        if biop_drive > F::ZERO {
            let from_s2 = (state.som2e[e] * biop_drive).min(state.som2e[e] * f(0.2));
            let from_s3 = (state.som3e[e] * biop_drive * f(0.2)).min(state.som3e[e] * f(0.1));
            if from_s2 > F::ZERO {
                ledger.schedule(PoolId::Som2E(e), PoolId::Minerl(0, e), from_s2);
                state.month_net_min[e] += from_s2;
            }
            if from_s3 > F::ZERO {
                ledger.schedule(PoolId::Som3E(e), PoolId::Minerl(0, e), from_s3);
                state.month_net_min[e] += from_s3;
            }
        }
    }

    // --- Sulfur Geochemistry (e = 2) ---
    if nelem >= 3 {
        let e = 2usize;

        // 1. Parent weathering -> mineral S.
        let par_s =
            (state.parent[e] * fix.pparmn[e] * defac * dt).clamp_val(F::ZERO, state.parent[e]);
        if par_s > F::ZERO {
            ledger.schedule(PoolId::Parent(e), PoolId::Minerl(0, e), par_s);
        }

        // 2. Labile <-> Secondary sulfate adsorption.
        let lab_to_sec_s = (state.minerl[0][e] * fix.pmnsec[e] * dt)
            .clamp_val(F::ZERO, state.minerl[0][e] * f(0.5));
        if lab_to_sec_s > F::ZERO {
            ledger.schedule(PoolId::Minerl(0, e), PoolId::Secndy(e), lab_to_sec_s);
        }
        let sec_to_lab_s = (state.secndy[e] * fix.psecmn[e] * defac * dt)
            .clamp_val(F::ZERO, state.secndy[e] * f(0.5));
        if sec_to_lab_s > F::ZERO {
            ledger.schedule(PoolId::Secndy(e), PoolId::Minerl(0, e), sec_to_lab_s);
        }

        // 3. Biochemical sulfatase mineralization from Slow & Passive SOM when mineral S is low.
        let bios_drive = (F::ONE - (state.minerl[0][e] / f(2.0)).clamp_val(F::ZERO, F::ONE))
            * defac
            * dt
            * f(0.015);
        if bios_drive > F::ZERO {
            let from_s2 = (state.som2e[e] * bios_drive).min(state.som2e[e] * f(0.2));
            let from_s3 = (state.som3e[e] * bios_drive * f(0.2)).min(state.som3e[e] * f(0.1));
            if from_s2 > F::ZERO {
                ledger.schedule(PoolId::Som2E(e), PoolId::Minerl(0, e), from_s2);
                state.month_net_min[e] += from_s2;
            }
            if from_s3 > F::ZERO {
                ledger.schedule(PoolId::Som3E(e), PoolId::Minerl(0, e), from_s3);
                state.month_net_min[e] += from_s3;
            }
        }
    }

    ledger.commit(state);
}
