//! Land management and disturbance events: `CULT`, `FERT`, `FIRE`, `GRAZ`,
//! `HARV`, `IRRI`, `OMAD`, `TREM`, and `EROD`.
//!
//! All transfers flow strictly through [`FlowLedger`] to guarantee exact C/N/P/S
//! mass conservation across every disturbance.
//!
//! Specification: `docs/spec/06_management_and_scheduling.md`.

use crate::config::ElementMode;
use crate::float::Float;
use crate::ledger::{FlowLedger, PoolId};
use crate::params::{
    CellDerivedParams, CropParams, CultParams, FertParams, FireParams, FixParams, GrazParams,
    HarvParams, IrriParams, OmadParams, SiteParams, TreeParams, TremParams, MAX_ELEM,
};
use crate::state::CellState;
use crate::subsystems::mineralization::partition_residue;

/// Compute monthly irrigation water (cm) and add irrigation S (when `FullCnps`).
pub fn apply_irrigation<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    site: &SiteParams<F>,
    irri: &IrriParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) -> F {
    let f = F::from_f64;
    let cm = match irri.auirri {
        0 => irri.irramt.max(F::ZERO),
        1 => {
            let target = derived.twhc_root * irri.fawhc.clamp_val(F::ZERO, F::ONE);
            (target - state.avh2o[0]).max(F::ZERO)
        }
        _ => {
            let trigger = derived.twhc_root * irri.fawhc.clamp_val(F::ZERO, F::ONE);
            if state.avh2o[0] < trigger {
                irri.irraut.max(F::ZERO)
            } else {
                F::ZERO
            }
        }
    };

    // Irrigation sulfur input: 1 cm water = 10 L/m2; sirri mg/L * 10 L/m2 / 1000 = 0.01 * cm * sirri g S/m2.
    if cm > F::ZERO && elements.nelem() >= 3 && site.sirri > F::ZERO {
        let s_in = cm * site.sirri * f(0.01);
        ledger.schedule(PoolId::ExternalInputE(2), PoolId::Minerl(0, 2), s_in);
        ledger.commit(state);
    }
    cm
}

/// Apply fixed or automatic fertilization (`FERT`).
pub fn apply_fertilizer<F: Float>(
    state: &mut CellState<F>,
    fert: &FertParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let nelem = elements.nelem();
    for e in 0..nelem {
        let amt = if fert.aufert > F::ZERO {
            // Auto-fertilization tops up layer 0 mineral pool to target level.
            let target = fert.aufert * F::from_f64(10.0);
            (target - state.minerl[0][e]).max(F::ZERO)
        } else {
            fert.feramt[e].max(F::ZERO)
        };
        if amt > F::ZERO {
            ledger.schedule(PoolId::ExternalInputE(e), PoolId::Minerl(0, e), amt);
        }
    }
    ledger.commit(state);
}

/// Apply organic matter amendment (`OMAD`), partitioning into surface structural & metabolic litter.
pub fn apply_omad<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    omad: &OmadParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let c_in = omad.astgc.max(F::ZERO);
    if c_in <= F::EPS_GUARD {
        return;
    }
    let mut e_in = [F::ZERO; MAX_ELEM];
    for e in 0..elements.nelem() {
        if omad.astrec[e] > F::EPS_GUARD {
            e_in[e] = c_in / omad.astrec[e];
        }
    }
    partition_residue(
        state,
        fix,
        0,
        PoolId::ExternalInputC,
        [
            PoolId::ExternalInputE(0),
            PoolId::ExternalInputE(1),
            PoolId::ExternalInputE(2),
        ],
        c_in,
        e_in,
        omad.astlig,
        Some(omad.astlbl.clamp_val(F::ZERO, F::ONE)),
        elements,
        ledger,
    );
    ledger.commit(state);
}

/// Apply cultivation / tillage (`CULT`): incorporates surface residues and sets `clteff`.
pub fn apply_cultivation<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    cult: &CultParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let f = F::from_f64;
    let nelem = elements.nelem();

    // Activate decomposition multipliers for 1 month (decaying back to 1.0 afterwards).
    state.clteff = [
        cult.clteff[0].max(F::ONE),
        cult.clteff[1].max(F::ONE),
        cult.clteff[2].max(F::ONE),
        cult.clteff[3].max(F::ONE),
    ];
    state.cult_months = 1;

    // 1. Live shoots -> surface litter (`cultra[0]`) and soil litter (`cultra[1]`).
    let agc = state.aglivc.total();
    if agc > F::EPS_GUARD {
        let f_surf = cult.cultra[0].clamp_val(F::ZERO, F::ONE);
        let f_soil = cult.cultra[1].clamp_val(F::ZERO, (F::ONE - f_surf).max(F::ZERO));
        if f_surf > F::ZERO {
            let mut ea = [F::ZERO; MAX_ELEM];
            for e in 0..nelem {
                ea[e] = state.aglive[e] * f_surf;
            }
            partition_residue(
                state,
                fix,
                0,
                PoolId::AglivC,
                [PoolId::AglivE(0), PoolId::AglivE(1), PoolId::AglivE(2)],
                agc * f_surf,
                ea,
                state.pltlig[0],
                None,
                elements,
                ledger,
            );
        }
        if f_soil > F::ZERO {
            let mut ea = [F::ZERO; MAX_ELEM];
            for e in 0..nelem {
                ea[e] = state.aglive[e] * f_soil;
            }
            partition_residue(
                state,
                fix,
                1,
                PoolId::AglivC,
                [PoolId::AglivE(0), PoolId::AglivE(1), PoolId::AglivE(2)],
                agc * f_soil,
                ea,
                state.pltlig[0],
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);

    // 2. Standing dead -> surface (`cultra[2]`) and soil (`cultra[3]`).
    let sdc = state.stdedc.total();
    if sdc > F::EPS_GUARD {
        let f_surf = cult.cultra[2].clamp_val(F::ZERO, F::ONE);
        let f_soil = cult.cultra[3].clamp_val(F::ZERO, (F::ONE - f_surf).max(F::ZERO));
        if f_surf > F::ZERO {
            let mut ea = [F::ZERO; MAX_ELEM];
            for e in 0..nelem {
                ea[e] = state.stdede[e] * f_surf;
            }
            partition_residue(
                state,
                fix,
                0,
                PoolId::StdedC,
                [PoolId::StdedE(0), PoolId::StdedE(1), PoolId::StdedE(2)],
                sdc * f_surf,
                ea,
                state.pltlig[0],
                None,
                elements,
                ledger,
            );
        }
        if f_soil > F::ZERO {
            let mut ea = [F::ZERO; MAX_ELEM];
            for e in 0..nelem {
                ea[e] = state.stdede[e] * f_soil;
            }
            partition_residue(
                state,
                fix,
                1,
                PoolId::StdedC,
                [PoolId::StdedE(0), PoolId::StdedE(1), PoolId::StdedE(2)],
                sdc * f_soil,
                ea,
                state.pltlig[0],
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);

    // 3. Surface structural & metabolic incorporation into soil litter (`cultra[4]`, `cultra[5]`).
    let f_sstr = cult.cultra[4].clamp_val(F::ZERO, f(0.95));
    if f_sstr > F::ZERO && state.strucc[0].total() > F::EPS_GUARD {
        let c_mv = state.strucc[0].total() * f_sstr;
        let old_soil = state.strucc[1].total();
        if old_soil + c_mv > F::EPS_GUARD {
            state.strlig[1] =
                (old_soil * state.strlig[1] + c_mv * state.strlig[0]) / (old_soil + c_mv);
        }
        ledger.schedule(PoolId::StrucC(0), PoolId::StrucC(1), c_mv);
        for e in 0..nelem {
            ledger.schedule(
                PoolId::StrucE(0, e),
                PoolId::StrucE(1, e),
                state.struce[0][e] * f_sstr,
            );
        }
    }
    let f_smet = cult.cultra[5].clamp_val(F::ZERO, f(0.95));
    if f_smet > F::ZERO && state.metabc[0].total() > F::EPS_GUARD {
        ledger.schedule(
            PoolId::MetabC(0),
            PoolId::MetabC(1),
            state.metabc[0].total() * f_smet,
        );
        for e in 0..nelem {
            ledger.schedule(
                PoolId::MetabE(0, e),
                PoolId::MetabE(1, e),
                state.metabe[0][e] * f_smet,
            );
        }
    }

    // 4. Live root kill (`cultra[6]`) -> soil litter.
    let f_bg = cult.cultra[6].clamp_val(F::ZERO, f(0.95));
    let bgc = state.bglivc.total();
    if f_bg > F::ZERO && bgc > F::EPS_GUARD {
        let mut ea = [F::ZERO; MAX_ELEM];
        for e in 0..nelem {
            ea[e] = state.bglive[e] * f_bg;
        }
        partition_residue(
            state,
            fix,
            1,
            PoolId::BglivC,
            [PoolId::BglivE(0), PoolId::BglivE(1), PoolId::BglivE(2)],
            bgc * f_bg,
            ea,
            state.pltlig[1],
            None,
            elements,
            ledger,
        );
    }
    ledger.commit(state);
}

/// Apply grazing event (`GRAZ`): removes live shoots & standing dead, returns feces & urine.
pub fn apply_grazing<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    graz: &GrazParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let nelem = elements.nelem();
    let f_live = graz.flgrem.clamp_val(F::ZERO, F::ONE);
    let f_dead = graz.fdgrem.clamp_val(F::ZERO, F::ONE);

    let ag_rem = state.aglivc.total() * f_live;
    let sd_rem = state.stdedc.total() * f_dead;
    let c_ret_frac = graz.gfcret.clamp_val(F::ZERO, F::ONE);

    // Process live shoots removed.
    if ag_rem > F::EPS_GUARD {
        let c_feces = ag_rem * c_ret_frac;
        let c_exp = ag_rem - c_feces;
        if c_exp > F::ZERO {
            ledger.schedule(PoolId::AglivC, PoolId::ExportC, c_exp);
            state.month_c_removed += c_exp;
        }
        let mut e_feces = [F::ZERO; MAX_ELEM];
        for e in 0..nelem {
            let e_rem = state.aglive[e] * f_live;
            let e_ret = e_rem * graz.gret[e].clamp_val(F::ZERO, F::ONE);
            let e_exp = (e_rem - e_ret).max(F::ZERO);
            let e_fec = e_ret * graz.fecf[e].clamp_val(F::ZERO, F::ONE);
            let e_uri = (e_ret - e_fec).max(F::ZERO);
            e_feces[e] = e_fec;
            if e_exp > F::ZERO {
                ledger.schedule(PoolId::AglivE(e), PoolId::ExportE(e), e_exp);
            }
            if e_uri > F::ZERO {
                ledger.schedule(PoolId::AglivE(e), PoolId::Minerl(0, e), e_uri);
            }
        }
        if c_feces > F::EPS_GUARD {
            partition_residue(
                state,
                fix,
                0,
                PoolId::AglivC,
                [PoolId::AglivE(0), PoolId::AglivE(1), PoolId::AglivE(2)],
                c_feces,
                e_feces,
                graz.feclig,
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);

    // Process standing dead removed.
    if sd_rem > F::EPS_GUARD {
        let c_feces = sd_rem * c_ret_frac;
        let c_exp = sd_rem - c_feces;
        if c_exp > F::ZERO {
            ledger.schedule(PoolId::StdedC, PoolId::ExportC, c_exp);
            state.month_c_removed += c_exp;
        }
        let mut e_feces = [F::ZERO; MAX_ELEM];
        for e in 0..nelem {
            let e_rem = state.stdede[e] * f_dead;
            let e_ret = e_rem * graz.gret[e].clamp_val(F::ZERO, F::ONE);
            let e_exp = (e_rem - e_ret).max(F::ZERO);
            let e_fec = e_ret * graz.fecf[e].clamp_val(F::ZERO, F::ONE);
            let e_uri = (e_ret - e_fec).max(F::ZERO);
            e_feces[e] = e_fec;
            if e_exp > F::ZERO {
                ledger.schedule(PoolId::StdedE(e), PoolId::ExportE(e), e_exp);
            }
            if e_uri > F::ZERO {
                ledger.schedule(PoolId::StdedE(e), PoolId::Minerl(0, e), e_uri);
            }
        }
        if c_feces > F::EPS_GUARD {
            partition_residue(
                state,
                fix,
                0,
                PoolId::StdedC,
                [PoolId::StdedE(0), PoolId::StdedE(1), PoolId::StdedE(2)],
                c_feces,
                e_feces,
                graz.feclig,
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);
}

/// Apply harvest event (`HARV`): grain harvest with moisture-stressed HI, straw removal,
/// residue return, root harvest, and N volatilization.
pub fn apply_harvest<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    crop: &CropParams<F>,
    harv: &HarvParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let nelem = elements.nelem();
    let agc = state.aglivc.total();
    let f_aff = harv.aglrem.clamp_val(F::ZERO, F::ONE);

    if agc > F::EPS_GUARD && f_aff > F::ZERO {
        let mut c_grain = F::ZERO;
        let mut e_grain = [F::ZERO; MAX_ELEM];

        // 1. Grain harvest.
        if harv.flghrv == 1 && crop.himax > F::ZERO {
            let ws_ratio = if state.grain_tr_months > 0 {
                (state.grain_tr_sum / F::from_usize(state.grain_tr_months as usize))
                    .clamp_val(F::ZERO, F::ONE)
            } else {
                F::ONE
            };
            let hi = (crop.himax * (F::ONE - crop.hiwsf * (F::ONE - ws_ratio)))
                .clamp_val(F::ZERO, F::from_f64(0.85));
            c_grain = agc * f_aff * hi;
            if c_grain > F::ZERO {
                ledger.schedule(PoolId::AglivC, PoolId::ExportC, c_grain);
                state.month_grain_c += c_grain;
                state.month_c_removed += c_grain;
            }
            for e in 0..nelem {
                let eg = state.aglive[e]
                    * f_aff
                    * (crop.efrgrn[e] * ws_ratio.sqrt()).clamp_val(F::ZERO, F::from_f64(0.95));
                e_grain[e] = eg;
                if eg > F::ZERO {
                    ledger.schedule(PoolId::AglivE(e), PoolId::ExportE(e), eg);
                }
            }
        }

        // 2. Remaining affected shoot biomass -> straw export, standing dead, or surface litter.
        let c_rem = (agc * f_aff - c_grain).max(F::ZERO);
        let c_straw_exp = c_rem * harv.rmvstr.clamp_val(F::ZERO, F::ONE);
        let c_left = (c_rem - c_straw_exp).max(F::ZERO);
        let c_to_sd = c_left * harv.remwsd.clamp_val(F::ZERO, F::ONE);
        let c_to_lit = (c_left - c_to_sd).max(F::ZERO);

        if c_straw_exp > F::ZERO {
            ledger.schedule(PoolId::AglivC, PoolId::ExportC, c_straw_exp);
            state.month_c_removed += c_straw_exp;
        }
        if c_to_sd > F::ZERO {
            ledger.schedule(PoolId::AglivC, PoolId::StdedC, c_to_sd);
        }

        let mut e_to_lit = [F::ZERO; MAX_ELEM];
        for e in 0..nelem {
            let mut e_rem = (state.aglive[e] * f_aff - e_grain[e]).max(F::ZERO);
            // Aboveground N volatilization on harvest.
            if e == 0 && crop.vlossp > F::ZERO {
                let vol = e_rem * crop.vlossp.clamp_val(F::ZERO, F::from_f64(0.5));
                ledger.schedule(PoolId::AglivE(0), PoolId::AtmosphereE(0), vol);
                state.month_n_volat += vol;
                e_rem = (e_rem - vol).max(F::ZERO);
            }
            let e_exp = e_rem * harv.rmvstr.clamp_val(F::ZERO, F::ONE);
            let e_left = (e_rem - e_exp).max(F::ZERO);
            let e_sd = e_left * harv.remwsd.clamp_val(F::ZERO, F::ONE);
            e_to_lit[e] = (e_left - e_sd).max(F::ZERO);
            if e_exp > F::ZERO {
                ledger.schedule(PoolId::AglivE(e), PoolId::ExportE(e), e_exp);
            }
            if e_sd > F::ZERO {
                ledger.schedule(PoolId::AglivE(e), PoolId::StdedE(e), e_sd);
            }
        }
        if c_to_lit > F::EPS_GUARD {
            partition_residue(
                state,
                fix,
                0,
                PoolId::AglivC,
                [PoolId::AglivE(0), PoolId::AglivE(1), PoolId::AglivE(2)],
                c_to_lit,
                e_to_lit,
                state.pltlig[0],
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);

    // 3. Belowground root harvest / kill.
    let bgc = state.bglivc.total();
    let f_bg = harv.bglrem.clamp_val(F::ZERO, F::ONE);
    if bgc > F::EPS_GUARD && f_bg > F::ZERO {
        let c_bg_aff = bgc * f_bg;
        let c_bg_exp = c_bg_aff * harv.hibg.clamp_val(F::ZERO, F::ONE);
        let c_bg_lit = (c_bg_aff - c_bg_exp).max(F::ZERO);
        if c_bg_exp > F::ZERO {
            ledger.schedule(PoolId::BglivC, PoolId::ExportC, c_bg_exp);
            state.month_c_removed += c_bg_exp;
        }
        let mut e_bg_lit = [F::ZERO; MAX_ELEM];
        for e in 0..nelem {
            let e_aff = state.bglive[e] * f_bg;
            let e_exp = e_aff * harv.hibg.clamp_val(F::ZERO, F::ONE);
            e_bg_lit[e] = (e_aff - e_exp).max(F::ZERO);
            if e_exp > F::ZERO {
                ledger.schedule(PoolId::BglivE(e), PoolId::ExportE(e), e_exp);
            }
        }
        if c_bg_lit > F::EPS_GUARD {
            partition_residue(
                state,
                fix,
                1,
                PoolId::BglivC,
                [PoolId::BglivE(0), PoolId::BglivE(1), PoolId::BglivE(2)],
                c_bg_lit,
                e_bg_lit,
                state.pltlig[1],
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);

    // Reset grain fill accumulators.
    state.grain_tr_sum = F::ZERO;
    state.grain_tr_months = 0;
}

/// Apply grassland/understory fire (`FIRE`): burns shoots, standing dead, and surface litter,
/// returning ash nutrients (`fret`) to `minerl[0]`.
pub fn apply_fire<F: Float>(
    state: &mut CellState<F>,
    fire: &FireParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let nelem = elements.nelem();
    let targets = [
        (
            fire.flfrem.clamp_val(F::ZERO, F::ONE),
            state.aglivc.total(),
            state.aglive,
            PoolId::AglivC,
            [PoolId::AglivE(0), PoolId::AglivE(1), PoolId::AglivE(2)],
        ),
        (
            fire.fdfrem[0].clamp_val(F::ZERO, F::ONE),
            state.stdedc.total(),
            state.stdede,
            PoolId::StdedC,
            [PoolId::StdedE(0), PoolId::StdedE(1), PoolId::StdedE(2)],
        ),
        (
            fire.fdfrem[1].clamp_val(F::ZERO, F::ONE),
            state.strucc[0].total(),
            state.struce[0],
            PoolId::StrucC(0),
            [
                PoolId::StrucE(0, 0),
                PoolId::StrucE(0, 1),
                PoolId::StrucE(0, 2),
            ],
        ),
        (
            fire.fdfrem[1].clamp_val(F::ZERO, F::ONE),
            state.metabc[0].total(),
            state.metabe[0],
            PoolId::MetabC(0),
            [
                PoolId::MetabE(0, 0),
                PoolId::MetabE(0, 1),
                PoolId::MetabE(0, 2),
            ],
        ),
    ];

    for (frac, c_tot, e_tot, c_id, e_ids) in targets {
        if frac > F::ZERO && c_tot > F::EPS_GUARD {
            let c_burn = c_tot * frac;
            ledger.schedule(c_id, PoolId::ExportC, c_burn);
            state.month_c_removed += c_burn;
            for e in 0..nelem {
                let e_burn = e_tot[e] * frac;
                let e_ash = e_burn * fire.fret[e].clamp_val(F::ZERO, F::ONE);
                let e_vol = (e_burn - e_ash).max(F::ZERO);
                if e_ash > F::ZERO {
                    ledger.schedule(e_ids[e], PoolId::Minerl(0, e), e_ash);
                }
                if e_vol > F::ZERO {
                    ledger.schedule(e_ids[e], PoolId::ExportE(e), e_vol);
                    if e == 0 {
                        state.month_n_volat += e_vol;
                    }
                }
            }
        }
    }

    state.fire_frtsh = fire.frtsh.max(F::ZERO);
    ledger.commit(state);
}

/// Apply tree removal / logging / canopy fire (`TREM`).
pub fn apply_trem<F: Float>(
    state: &mut CellState<F>,
    fix: &FixParams<F>,
    tree: Option<&TreeParams<F>>,
    trem: &TremParams<F>,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    let nelem = elements.nelem();
    let is_burn = trem.evntyp == 1;
    let wdlig = tree.map(|t| t.wdlig).unwrap_or([
        F::from_f64(0.20),
        F::from_f64(0.25),
        F::from_f64(0.25),
        F::from_f64(0.30),
        F::from_f64(0.30),
    ]);

    // 1. Leaves (`remf[0]`): removed/burned or slash -> surface litter.
    let f_lv = trem.remf[0].clamp_val(F::ZERO, F::ONE);
    let lv_c = state.rleavc.total();
    if f_lv > F::ZERO && lv_c > F::EPS_GUARD {
        let c_rem = lv_c * f_lv;
        if is_burn {
            ledger.schedule(PoolId::RleavC, PoolId::ExportC, c_rem);
            state.month_c_removed += c_rem;
            for e in 0..nelem {
                let er = state.rleave[e] * f_lv;
                let ret = er * trem.retf[e][0].clamp_val(F::ZERO, F::ONE);
                let vol = (er - ret).max(F::ZERO);
                if ret > F::ZERO {
                    ledger.schedule(PoolId::RleavE(e), PoolId::Minerl(0, e), ret);
                }
                if vol > F::ZERO {
                    ledger.schedule(PoolId::RleavE(e), PoolId::ExportE(e), vol);
                }
            }
        } else {
            let mut ea = [F::ZERO; MAX_ELEM];
            for e in 0..nelem {
                ea[e] = state.rleave[e] * f_lv;
            }
            partition_residue(
                state,
                fix,
                0,
                PoolId::RleavC,
                [PoolId::RleavE(0), PoolId::RleavE(1), PoolId::RleavE(2)],
                c_rem,
                ea,
                wdlig[0],
                None,
                elements,
                ledger,
            );
        }
    }
    ledger.commit(state);

    // 2. Fine branches (`remf[1]`), Large wood (`remf[2]`), Wood1 (`remf[3]`), Wood2 (`remf[4]`).
    let wood_ops = [
        (
            trem.remf[1],
            state.fbrchc.total(),
            state.fbrche,
            PoolId::FbrchC,
            Some(PoolId::Wood1C),
            [PoolId::FbrchE(0), PoolId::FbrchE(1), PoolId::FbrchE(2)],
            Some([PoolId::Wood1E(0), PoolId::Wood1E(1), PoolId::Wood1E(2)]),
            1usize,
        ),
        (
            trem.remf[2],
            state.rlwodc.total(),
            state.rlwode,
            PoolId::RlwodC,
            Some(PoolId::Wood2C),
            [PoolId::RlwodE(0), PoolId::RlwodE(1), PoolId::RlwodE(2)],
            Some([PoolId::Wood2E(0), PoolId::Wood2E(1), PoolId::Wood2E(2)]),
            2usize,
        ),
        (
            trem.remf[3],
            state.wood1c.total(),
            state.wood1e,
            PoolId::Wood1C,
            None,
            [PoolId::Wood1E(0), PoolId::Wood1E(1), PoolId::Wood1E(2)],
            None,
            3usize,
        ),
        (
            trem.remf[4],
            state.wood2c.total(),
            state.wood2e,
            PoolId::Wood2C,
            None,
            [PoolId::Wood2E(0), PoolId::Wood2E(1), PoolId::Wood2E(2)],
            None,
            3usize,
        ),
    ];

    for (rem_f, c_tot, e_tot, src_c, slash_c, src_e, slash_e, ret_col) in wood_ops {
        let frac = rem_f.clamp_val(F::ZERO, F::ONE);
        if frac > F::ZERO && c_tot > F::EPS_GUARD {
            let c_rem = c_tot * frac;
            ledger.schedule(src_c, PoolId::ExportC, c_rem);
            state.month_c_removed += c_rem;
            for e in 0..nelem {
                let er = e_tot[e] * frac;
                if is_burn {
                    let ash = er * trem.retf[e][ret_col].clamp_val(F::ZERO, F::ONE);
                    let vol = (er - ash).max(F::ZERO);
                    if ash > F::ZERO {
                        ledger.schedule(src_e[e], PoolId::Minerl(0, e), ash);
                    }
                    if vol > F::ZERO {
                        ledger.schedule(src_e[e], PoolId::ExportE(e), vol);
                    }
                } else {
                    ledger.schedule(src_e[e], PoolId::ExportE(e), er);
                }
            }
            // In non-burn cutting, remaining fraction of live wood can enter dead wood slash if clearcut.
            if !is_burn && frac > F::from_f64(0.5) {
                if let (Some(sc), Some(se)) = (slash_c, slash_e) {
                    let slash_frac = (F::ONE - frac) * F::from_f64(0.5);
                    if slash_frac > F::ZERO {
                        ledger.schedule(src_c, sc, c_tot * slash_frac);
                        for e in 0..nelem {
                            ledger.schedule(src_e[e], se[e], e_tot[e] * slash_frac);
                        }
                    }
                }
            }
        }
    }
    ledger.commit(state);

    // 3. Root mortality from tree removal (`fd[0]` fine roots -> soil litter, `fd[1]` coarse roots -> wood3).
    let f_fr = trem.fd[0].clamp_val(F::ZERO, F::ONE);
    let fr_c = state.frootc.total();
    if f_fr > F::ZERO && fr_c > F::EPS_GUARD {
        let mut ea = [F::ZERO; MAX_ELEM];
        for e in 0..nelem {
            ea[e] = state.froote[e] * f_fr;
        }
        partition_residue(
            state,
            fix,
            1,
            PoolId::FrootC,
            [PoolId::FrootE(0), PoolId::FrootE(1), PoolId::FrootE(2)],
            fr_c * f_fr,
            ea,
            wdlig[1],
            None,
            elements,
            ledger,
        );
    }
    let f_cr = trem.fd[1].clamp_val(F::ZERO, F::ONE);
    let cr_c = state.crootc.total();
    if f_cr > F::ZERO && cr_c > F::EPS_GUARD {
        ledger.schedule(PoolId::CrootC, PoolId::Wood3C, cr_c * f_cr);
        for e in 0..nelem {
            ledger.schedule(PoolId::CrootE(e), PoolId::Wood3E(e), state.croote[e] * f_cr);
        }
    }
    ledger.commit(state);
}

/// Apply soil erosion event (`EROD`): removes fraction of surface/soil SOM & litter
/// proportional to sediment loss `erod_kg_m2` ($\text{kg soil}\cdot\text{m}^{-2}$).
pub fn apply_erosion<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    fix: &FixParams<F>,
    erod_kg_m2: F,
    elements: ElementMode,
    ledger: &mut FlowLedger<F>,
) {
    if erod_kg_m2 <= F::EPS_GUARD {
        return;
    }
    let f = F::from_f64;
    // Mass of top soil layer (edepth m * bulkd g/cm3 * 1000 kg/m3 per g/cm3).
    let top_soil_kg = (fix.edepth * derived.bulkd * f(1000.0)).max(f(10.0));
    let loss_frac = ((erod_kg_m2 / top_soil_kg) * fix.enrich).clamp_val(F::ZERO, f(0.5));
    let nelem = elements.nelem();

    let som_pools = [
        (
            state.som1c[0].total(),
            state.som1e[0],
            PoolId::Som1C(0),
            [
                PoolId::Som1E(0, 0),
                PoolId::Som1E(0, 1),
                PoolId::Som1E(0, 2),
            ],
        ),
        (
            state.som1c[1].total(),
            state.som1e[1],
            PoolId::Som1C(1),
            [
                PoolId::Som1E(1, 0),
                PoolId::Som1E(1, 1),
                PoolId::Som1E(1, 2),
            ],
        ),
        (
            state.som2c.total(),
            state.som2e,
            PoolId::Som2C,
            [PoolId::Som2E(0), PoolId::Som2E(1), PoolId::Som2E(2)],
        ),
        (
            state.som3c.total(),
            state.som3e,
            PoolId::Som3C,
            [PoolId::Som3E(0), PoolId::Som3E(1), PoolId::Som3E(2)],
        ),
        (
            state.strucc[0].total(),
            state.struce[0],
            PoolId::StrucC(0),
            [
                PoolId::StrucE(0, 0),
                PoolId::StrucE(0, 1),
                PoolId::StrucE(0, 2),
            ],
        ),
        (
            state.metabc[0].total(),
            state.metabe[0],
            PoolId::MetabC(0),
            [
                PoolId::MetabE(0, 0),
                PoolId::MetabE(0, 1),
                PoolId::MetabE(0, 2),
            ],
        ),
    ];

    for (c_tot, e_tot, c_id, e_ids) in som_pools {
        if c_tot > F::EPS_GUARD {
            let c_loss = c_tot * loss_frac;
            ledger.schedule(c_id, PoolId::ExportC, c_loss);
            state.month_c_removed += c_loss;
            for e in 0..nelem {
                let e_loss = e_tot[e] * loss_frac;
                if e_loss > F::ZERO {
                    ledger.schedule(e_ids[e], PoolId::ExportE(e), e_loss);
                }
            }
        }
    }
    ledger.commit(state);
}
