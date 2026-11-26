//! Top-level cell initialization (`initialize_cell`) and deterministic monthly/annual
//! simulation orchestrators (`step_month`, `step_year`, `run_cell_simulation`).

use crate::config::{SimConfig, SystemKind};
use crate::float::Float;
use crate::ledger::{check_mass_balance, FlowLedger, MassBalanceReport, PoolId};
use crate::params::{
    CellDerivedParams, CellSiteInput, CropParams, RawBiomeParams, TreeParams, MAX_ELEM,
};
use crate::state::{AnnualOutput, CarbonPool, CellState, MonthlyOutput, MonthlyWeather};
use crate::subsystems::{
    co2::compute_co2_effects,
    crop::step_crop,
    decomposition::{step_decomposition, update_abiotic_factors},
    forest::{savanna_competition, step_forest},
    hydrology::{linacre_pet, step_hydrology_with_pet},
    management::{
        apply_cultivation, apply_erosion, apply_fertilizer, apply_fire, apply_grazing,
        apply_harvest, apply_irrigation, apply_omad, apply_trem,
    },
    mineralization::{apply_atmospheric_inputs, partition_residue},
    phosphorus_sulfur::step_phosphorus_sulfur,
    scheduler::{CompiledSchedule, EventKind, ResolvedMonthEvents, Schedule},
    soil_temp::compute_soil_temp,
};

/// Active runtime crop/tree name selections carried across months for a cell.
#[derive(Debug, Clone)]
pub struct ActiveSelection {
    /// Currently active crop option name in `RawBiomeParams::crops`.
    pub crop_name: String,
    /// Currently active tree option name in `RawBiomeParams::trees`.
    pub tree_name: String,
}

impl ActiveSelection {
    /// Create initial selection from a [`Schedule`].
    pub fn from_schedule(schedule: &Schedule) -> Self {
        Self {
            crop_name: schedule.initial_crop.clone(),
            tree_name: schedule.initial_tree.clone(),
        }
    }
}

/// Initialize a fresh [`CellState`] and [`CellDerivedParams`] for a single grid cell.
///
/// Supports `ivauto ∈ {0, 1, 2, 3}` (explicit site pools vs. automatic grassland/crop/forest
/// SOM estimation from climate & texture, Parton et al. 1987) and honors
/// [`CompatibilityProfile`].
pub fn initialize_cell<F: Float>(
    biome: &RawBiomeParams<F>,
    cell_input: &CellSiteInput<F>,
    mean_weather: &[MonthlyWeather<F>; 12],
    schedule: &Schedule,
    config: &SimConfig,
) -> (CellState<F>, CellDerivedParams<F>) {
    let f = F::from_f64;
    let site = &biome.site_defaults;
    let fix = &biome.fix;
    let nelem = config.elements.nelem();

    // Compute mean annual precipitation and monthly mean air temperatures.
    let mut ann_ppt = F::ZERO;
    let mut mean_temps = [F::ZERO; 12];
    for m in 0..12 {
        ann_ppt += mean_weather[m].precip_cm.max(F::ZERO);
        mean_temps[m] = (mean_weather[m].tmin_c + mean_weather[m].tmax_c) * f(0.5);
    }

    let mut state = CellState::<F> {
        system: schedule.initial_system,
        ann_ppt: ann_ppt.max(f(1.0)),
        ann_aet: (ann_ppt * f(0.65)).clamp_val(f(5.0), f(60.0)),
        co2_ppm: config
            .co2_ppm_override
            .map(F::from_f64)
            .unwrap_or(fix.co2ppm[0]),
        snow: site.snow.max(F::ZERO),
        snlq: site.snlq.max(F::ZERO),
        ..CellState::default()
    };

    // 1. Initialize SOM pools (`ivauto == 0` uses site.100; `1..=3` uses Parton et al. regression).
    if site.ivauto == 0 {
        state.som1c[0] = CarbonPool {
            unlab: site.som1ci[0][0].max(F::ZERO),
            lab: site.som1ci[0][1].max(F::ZERO),
        };
        state.som1c[1] = CarbonPool {
            unlab: site.som1ci[1][0].max(F::ZERO),
            lab: site.som1ci[1][1].max(F::ZERO),
        };
        state.som2c = CarbonPool {
            unlab: site.som2ci[0].max(F::ZERO),
            lab: site.som2ci[1].max(F::ZERO),
        };
        state.som3c = CarbonPool {
            unlab: site.som3ci[0].max(F::ZERO),
            lab: site.som3ci[1].max(F::ZERO),
        };
    } else {
        // Automatic SOM initialization (ivauto = 1 grassland, 2 crop, 3 forest).
        let t_ann = mean_temps.iter().copied().fold(F::ZERO, |a, b| a + b) / f(12.0);
        let silt_clay = (cell_input.silt + cell_input.clay).clamp_val(f(0.05), f(0.95));
        let base_som = ((-f(0.08) * t_ann).exp()
            * (f(1500.0) + f(45.0) * ann_ppt.min(f(150.0)))
            * (f(0.5) + silt_clay))
            .clamp_val(f(500.0), f(25000.0));
        let sys_scale = match site.ivauto {
            1 => f(1.0),  // Grassland
            2 => f(0.75), // Cultivated crop
            _ => f(1.15), // Forest
        };
        let tot_som = base_som * sys_scale;
        state.som1c[0] = CarbonPool::from_total(tot_som * f(0.01), F::ZERO);
        state.som1c[1] = CarbonPool::from_total(tot_som * f(0.03), F::ZERO);
        state.som2c = CarbonPool::from_total(tot_som * f(0.56), F::ZERO);
        state.som3c = CarbonPool::from_total(tot_som * f(0.40), F::ZERO);
    }

    // SOM nutrient pools from initial C:E ratios.
    for e in 0..nelem {
        state.som1e[0][e] = state.som1c[0].total() / site.rces1[0][e].max(f(1.0));
        state.som1e[1][e] = state.som1c[1].total() / site.rces1[1][e].max(f(1.0));
        state.som2e[e] = state.som2c.total() / site.rces2[e].max(f(1.0));
        state.som3e[e] = state.som3c.total() / site.rces3[e].max(f(1.0));
    }

    // Compute per-cell derived soil/hydraulic parameters (honoring CompatibilityProfile).
    let derived = CellDerivedParams::compute(
        fix,
        site,
        cell_input,
        state.somsc(),
        &mean_temps,
        config.profile,
    );

    // 2. Partition initial surface & soil litter into structural/metabolic pools.
    let mut init_ledger = FlowLedger::<F>::new();
    for layer in 0..2 {
        let lit_c = (site.clittr[layer][0] + site.clittr[layer][1]).max(F::ZERO);
        if lit_c > F::EPS_GUARD {
            let lf = if lit_c > F::EPS_GUARD {
                site.clittr[layer][1] / lit_c
            } else {
                F::ZERO
            };
            let mut lit_e = [F::ZERO; MAX_ELEM];
            for e in 0..nelem {
                lit_e[e] = lit_c / site.rcelit[layer][e].max(f(5.0));
            }
            partition_residue(
                &mut state,
                fix,
                layer,
                PoolId::ExternalInputC,
                [
                    PoolId::ExternalInputE(0),
                    PoolId::ExternalInputE(1),
                    PoolId::ExternalInputE(2),
                ],
                lit_c,
                lit_e,
                f(0.20),
                Some(lf),
                config.elements,
                &mut init_ledger,
            );
        }
    }
    init_ledger.commit(&mut state);
    // Reset boundary counters after initial state construction.
    state.cum_c_in = F::ZERO;
    state.cum_c_out = F::ZERO;
    state.cum_e_in = [F::ZERO; MAX_ELEM];
    state.cum_e_out = [F::ZERO; MAX_ELEM];

    // 3. Initial plant pools (herbaceous + woody + dead wood).
    state.aglivc = CarbonPool {
        unlab: site.aglcis[0].max(F::ZERO),
        lab: site.aglcis[1].max(F::ZERO),
    };
    state.bglivc = CarbonPool {
        unlab: site.bglcis[0].max(F::ZERO),
        lab: site.bglcis[1].max(F::ZERO),
    };
    state.stdedc = CarbonPool {
        unlab: site.stdcis[0].max(F::ZERO),
        lab: site.stdcis[1].max(F::ZERO),
    };
    state.rleavc = CarbonPool {
        unlab: site.rlvcis[0].max(F::ZERO),
        lab: site.rlvcis[1].max(F::ZERO),
    };
    state.fbrchc = CarbonPool {
        unlab: site.fbrcis[0].max(F::ZERO),
        lab: site.fbrcis[1].max(F::ZERO),
    };
    state.rlwodc = CarbonPool {
        unlab: site.rlwcis[0].max(F::ZERO),
        lab: site.rlwcis[1].max(F::ZERO),
    };
    state.frootc = CarbonPool {
        unlab: site.frtcis[0].max(F::ZERO),
        lab: site.frtcis[1].max(F::ZERO),
    };
    state.crootc = CarbonPool {
        unlab: site.crtcis[0].max(F::ZERO),
        lab: site.crtcis[1].max(F::ZERO),
    };
    state.wood1c = CarbonPool {
        unlab: site.wd1cis[0].max(F::ZERO),
        lab: site.wd1cis[1].max(F::ZERO),
    };
    state.wood2c = CarbonPool {
        unlab: site.wd2cis[0].max(F::ZERO),
        lab: site.wd2cis[1].max(F::ZERO),
    };
    state.wood3c = CarbonPool {
        unlab: site.wd3cis[0].max(F::ZERO),
        lab: site.wd3cis[1].max(F::ZERO),
    };
    state.wdlig = [site.w1lig, site.w2lig, site.w3lig];

    for e in 0..nelem {
        state.aglive[e] = site.aglive[e].max(F::ZERO);
        state.bglive[e] = site.bglive[e].max(F::ZERO);
        state.stdede[e] = site.stdede[e].max(F::ZERO);
        state.rleave[e] = site.rleave[e].max(F::ZERO);
        state.fbrche[e] = site.fbrche[e].max(F::ZERO);
        state.rlwode[e] = site.rlwode[e].max(F::ZERO);
        state.froote[e] = site.froote[e].max(F::ZERO);
        state.croote[e] = site.croote[e].max(F::ZERO);
        // Dead wood nutrients initialized from default woody C:E ratios if C > 0.
        let ce_w = [f(150.0), f(1000.0), f(1000.0)][e];
        state.wood1e[e] = state.wood1c.total() / ce_w;
        state.wood2e[e] = state.wood2c.total() / ce_w;
        state.wood3e[e] = state.wood3c.total() / ce_w;
        state.parent[e] = site.parent[e].max(F::ZERO);
        state.secndy[e] = site.secndy[e].max(F::ZERO);
        for l in 0..derived.nlayer {
            state.minerl[l][e] = site.minerl[l][e].max(F::ZERO);
        }
    }
    state.occlud = if nelem >= 2 {
        site.occlud.max(F::ZERO)
    } else {
        F::ZERO
    };

    // 4. Initialize soil water per layer from relative water content `rwcf` and populate `avh2o`.
    let mut av_root = F::ZERO;
    let mut av_prof = F::ZERO;
    let mut av_top30 = F::ZERO;
    let mut depth_acc = F::ZERO;
    for l in 0..derived.nlayer {
        let wp = derived.awilt[l] * derived.adep[l];
        let fc = derived.afiel[l] * derived.adep[l];
        let rw = site.rwcf[l].clamp_val(F::ZERO, F::ONE);
        state.asmos[l] = wp + rw * (fc - wp);
        let av = (state.asmos[l] - wp).max(F::ZERO);
        av_prof += av;
        if l < derived.nlaypg {
            av_root += av;
        }
        if depth_acc < f(30.0) {
            let frac_in_30 = ((f(30.0) - depth_acc) / derived.adep[l]).clamp_val(F::ZERO, F::ONE);
            av_top30 += av * frac_in_30;
        }
        depth_acc += derived.adep[l];
    }
    state.avh2o = [av_root, av_prof, av_top30];

    // Ensure non-zero seed biomass if system has forest/crop so growth can initiate cleanly.
    if state.system.has_forest()
        && state.rleavc.total() <= F::EPS_GUARD
        && state.rlwodc.total() <= F::EPS_GUARD
    {
        state.rleavc = CarbonPool::from_total(f(10.0), F::ZERO);
        state.frootc = CarbonPool::from_total(f(10.0), F::ZERO);
        state.rlwodc = CarbonPool::from_total(f(50.0), F::ZERO);
        for e in 0..nelem {
            state.rleave[e] = state.rleavc.total() / f(50.0);
            state.froote[e] = state.frootc.total() / f(55.0);
            state.rlwode[e] = state.rlwodc.total() / f(300.0);
        }
    }

    (state, derived)
}

/// Pre-resolved active crop/tree parameter references cached across months to avoid
/// per-month `HashMap<String, _>` hashing in long multi-century grid runs.
struct CachedParams<'a, F: Float> {
    crop: Option<&'a CropParams<F>>,
    tree: Option<&'a TreeParams<F>>,
}

impl<'a, F: Float> CachedParams<'a, F> {
    #[inline]
    fn resolve(biome: &'a RawBiomeParams<F>, selection: &ActiveSelection) -> Self {
        Self {
            crop: if selection.crop_name.is_empty() {
                None
            } else {
                biome.crops.get(&selection.crop_name)
            },
            tree: if selection.tree_name.is_empty() {
                None
            } else {
                biome.trees.get(&selection.tree_name)
            },
        }
    }
}

/// Execute one deterministic monthly timestep (`month` $\in 1..=12$) for a single cell.
///
/// Returns `(MonthlyOutput<F>, Option<MassBalanceReport<F>>)`.
#[allow(clippy::too_many_arguments)]
pub fn step_month<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    biome: &RawBiomeParams<F>,
    selection: &mut ActiveSelection,
    schedule: &Schedule,
    weather: &MonthlyWeather<F>,
    year: i32,
    month: u8,
    co2_ppm: F,
    config: &SimConfig,
) -> (MonthlyOutput<F>, Option<MassBalanceReport<F>>) {
    let mut ledger = FlowLedger::<F>::new();
    let mut cached = CachedParams::resolve(biome, selection);
    let mut events_buf: [Option<&EventKind>; 24] = [None; 24];
    let n_events = schedule.events_for_buf(year, month, &mut events_buf);
    let mut resolved = ResolvedMonthEvents::default();
    for ev in events_buf[..n_events].iter().flatten() {
        resolved.apply_event(ev, biome);
    }
    step_month_resolved(
        state,
        derived,
        biome,
        Some(selection),
        &mut cached,
        schedule.label_year,
        resolved,
        weather,
        None,
        year,
        month,
        co2_ppm,
        config,
        &mut ledger,
    )
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn step_month_resolved<'a, F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    biome: &'a RawBiomeParams<F>,
    mut selection: Option<&mut ActiveSelection>,
    cached: &mut CachedParams<'a, F>,
    label_year: i32,
    ev: ResolvedMonthEvents<'a, F>,
    weather: &MonthlyWeather<F>,
    pet_override: Option<F>,
    year: i32,
    month: u8,
    co2_ppm: F,
    config: &SimConfig,
    ledger: &mut FlowLedger<F>,
) -> (MonthlyOutput<F>, Option<MassBalanceReport<F>>) {
    let f = F::from_f64;

    // Optional pre-step mass snapshot for strict conservation audit.
    let pre_snap = if config.audit_mass_balance {
        Some((
            state.total_carbon(),
            [
                state.total_element(0),
                state.total_element(1),
                state.total_element(2),
            ],
            state.cum_c_in,
            state.cum_c_out,
            state.cum_e_in,
            state.cum_e_out,
        ))
    } else {
        None
    };

    state.reset_monthly_diagnostics();
    state.co2_ppm = co2_ppm;
    state.senm = ev.senm;

    // Update isotope atmospheric labeled fraction when labeling is active.
    state.atm_lab_frac = match config.isotope {
        crate::config::IsotopeMode::Unlabeled => F::ZERO,
        crate::config::IsotopeMode::C14 => {
            if label_year >= 0 && year >= label_year {
                f(1.0)
            } else {
                F::ZERO
            }
        }
        crate::config::IsotopeMode::C13 => f(0.011),
    };

    // Decay transient cultivation effect & post-fire root:shoot boost.
    if state.cult_months > 0 {
        state.cult_months -= 1;
        if state.cult_months == 0 {
            state.clteff = [F::ONE; 4];
        }
    }
    if month == 1 {
        state.fire_frtsh *= f(0.5);
    }

    // 1. Apply O(1) pre-resolved schedule events for (year, month).
    if let Some((name, cp)) = ev.crop {
        cached.crop = cp;
        if let Some(sel) = selection.as_deref_mut() {
            if sel.crop_name != name {
                sel.crop_name.clear();
                sel.crop_name.push_str(name);
            }
        }
        if state.system == SystemKind::Forest {
            state.system = SystemKind::Savanna;
        }
    }
    if let Some((name, tp)) = ev.tree {
        cached.tree = tp;
        if let Some(sel) = selection {
            if sel.tree_name != name {
                sel.tree_name.clear();
                sel.tree_name.push_str(name);
            }
        }
        if state.system == SystemKind::GrasslandCrop {
            state.system = SystemKind::Savanna;
        }
    }
    if let Some(v) = ev.set_crpgrw {
        state.crpgrw = v;
    }
    if let Some(v) = ev.set_forgrw {
        state.forgrw = v;
    }

    ledger.clear();

    let crop_params = cached.crop;
    let tree_params = cached.tree;
    let co2 = compute_co2_effects(co2_ppm, &biome.fix, crop_params, tree_params);

    let pet_val = pet_override
        .unwrap_or_else(|| linacre_pet(month, weather, derived, &biome.fix, config.profile));

    // 2. Irrigation water calculation (applied into hydrology).
    let irri_cm = if let Some(ip) = ev.irri {
        apply_irrigation(
            state,
            derived,
            &biome.site_defaults,
            ip,
            weather.precip_cm.max(F::ZERO),
            pet_val,
            config.elements,
            ledger,
        )
    } else {
        F::ZERO
    };

    // 3. Hydrology & Soil Temperature.
    let transp_mult = if state.system == SystemKind::Forest {
        co2.forest_transp
    } else {
        co2.crop_transp
    };
    let hydro = step_hydrology_with_pet(
        state,
        derived,
        &biome.fix,
        weather,
        pet_val,
        irri_cm,
        transp_mult,
        config.elements,
        ledger,
    );
    compute_soil_temp(state, &biome.fix, weather);
    update_abiotic_factors(
        state,
        derived,
        &biome.fix,
        hydro.rel_water_surface,
        hydro.top_rwcf,
    );

    // Track grain-fill transpiration ratio if inside crop.himon window.
    if let Some(cp) = crop_params {
        if cp.himon[0] > 0 && (month as i32) >= cp.himon[0] && (month as i32) <= cp.himon[1] {
            let tr = if hydro.ptran > F::EPS_GUARD {
                (hydro.tran / hydro.ptran).clamp_val(F::ZERO, F::ONE)
            } else {
                F::ONE
            };
            state.grain_tr_sum += tr;
            state.grain_tr_months = state.grain_tr_months.saturating_add(1);
        }
    }

    // 4. Atmospheric deposition, non-symbiotic N fixation, fertilization, and OMAD.
    apply_atmospheric_inputs(
        state,
        &biome.fix,
        &biome.site_defaults,
        weather.precip_cm.max(F::ZERO),
        config.elements,
        ledger,
    );
    if ev.has_any_mgmt {
        if let Some(fp) = ev.fert {
            apply_fertilizer(state, fp, config.elements, ledger);
        }
        if let Some(op) = ev.omad {
            apply_omad(state, &biome.fix, op, config.elements, ledger);
        }
        if let Some(cp) = ev.cult {
            apply_cultivation(state, &biome.fix, cp, config.elements, ledger);
        }
    }

    // 5. Forest and Crop vegetation production & senescence.
    let (tree_shade_fac, tree_n_share, grass_n_share) = if state.system == SystemKind::Savanna {
        if let Some(tp) = tree_params {
            let (_lai, shade, t_share, g_share) = savanna_competition(state, tp);
            (shade, t_share, g_share)
        } else {
            (F::ONE, F::ONE, F::ONE)
        }
    } else {
        (F::ONE, F::ONE, F::ONE)
    };

    if state.system.has_forest() {
        if let Some(tp) = tree_params {
            step_forest(
                state,
                derived,
                &biome.fix,
                tp,
                weather,
                month,
                hydro.rel_water_root,
                tree_n_share,
                &co2,
                config.elements,
                ledger,
            );
        }
    }

    if state.system.has_crop() {
        if let Some(cp) = crop_params {
            // Apply planting-month potential production reduction factor `pltmrf` (Manual A2-2).
            let crop_shade_and_pltm = if ev.planted && cp.pltmrf > F::ZERO {
                tree_shade_fac * cp.pltmrf.clamp_val(F::ZERO, F::ONE)
            } else {
                tree_shade_fac
            };
            step_crop(
                state,
                derived,
                &biome.fix,
                cp,
                weather,
                hydro.rel_water_root,
                crop_shade_and_pltm,
                grass_n_share,
                &co2,
                config.elements,
                ledger,
            );
        }
    }

    // 6. Management disturbances (GRAZ, HARV, FIRE, TREM, EROD).
    if ev.has_any_mgmt {
        if let Some(gp) = ev.graz {
            apply_grazing(state, &biome.fix, gp, config.elements, ledger);
        }
        if let (Some(hp), Some(cp)) = (ev.harv, crop_params) {
            apply_harvest(state, &biome.fix, cp, hp, config.elements, ledger);
        }
        if let Some(fp) = ev.fire {
            apply_fire(state, fp, config.elements, ledger);
        }
        if let Some(tp) = ev.trem {
            apply_trem(state, &biome.fix, tree_params, tp, config.elements, ledger);
        }
        if let Some(er) = ev.erod {
            apply_erosion(state, derived, &biome.fix, er, config.elements, ledger);
        }
    }

    // 7. Decomposition & P/S Geochemistry (organic leaching driven by 30 cm water flux).
    step_decomposition(
        state,
        derived,
        &biome.fix,
        tree_params,
        hydro.amov_30cm,
        config.elements,
        ledger,
    );
    if config.elements != crate::config::ElementMode::CarbonNitrogen {
        step_phosphorus_sulfur(
            state,
            derived,
            &biome.fix,
            &biome.site_defaults,
            config.elements,
            ledger,
        );
    }

    let mut total_h2o = F::ZERO;
    for l in 0..derived.nlayer {
        total_h2o += state.asmos[l];
    }

    let out = MonthlyOutput {
        year,
        month,
        somsc: state.somsc(),
        som1c_soil: state.som1c[1].total(),
        som2c: state.som2c.total(),
        som3c: state.som3c.total(),
        aglivc: state.aglivc.total(),
        bglivc: state.bglivc.total(),
        rleavc: state.rleavc.total(),
        rlwodc: state.rlwodc.total(),
        minerl_n0: state.minerl[0][0],
        total_h2o,
        pet: hydro.pet,
        aet: hydro.aet,
        resp: state.month_resp,
        npp: state.month_npp,
        stemp: state.stemp,
        defac: state.defac,
    };

    let audit = pre_snap
        .map(|(bc, be, bci, bco, bei, beo)| check_mass_balance(bc, be, bci, bco, bei, beo, state));

    (out, audit)
}

/// Execute one full calendar year (months `1..=12`) and return annual summary output.
#[allow(clippy::too_many_arguments)]
pub fn step_year<F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    biome: &RawBiomeParams<F>,
    selection: &mut ActiveSelection,
    schedule: &Schedule,
    year_weather: &[MonthlyWeather<F>; 12],
    year: i32,
    co2_ppm: F,
    config: &SimConfig,
) -> AnnualOutput<F> {
    let mut ledger = FlowLedger::<F>::new();
    let mut cached = CachedParams::resolve(biome, selection);
    let compiled = CompiledSchedule::build(schedule, biome);
    let mut block_cursor = 0usize;
    step_year_compiled(
        state,
        derived,
        biome,
        Some(selection),
        &mut cached,
        &compiled,
        &mut block_cursor,
        year_weather,
        None,
        year,
        co2_ppm,
        config,
        &mut ledger,
    )
}

#[allow(clippy::too_many_arguments)]
fn step_year_compiled<'a, F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    biome: &'a RawBiomeParams<F>,
    mut selection: Option<&mut ActiveSelection>,
    cached: &mut CachedParams<'a, F>,
    compiled: &CompiledSchedule<'a, F>,
    block_cursor: &mut usize,
    year_weather: &[MonthlyWeather<F>; 12],
    cached_pet: Option<&[F; 12]>,
    year: i32,
    co2_ppm: F,
    config: &SimConfig,
    ledger: &mut FlowLedger<F>,
) -> AnnualOutput<F> {
    let mut ann_npp = F::ZERO;
    let mut ann_resp = F::ZERO;
    let mut ann_aet = F::ZERO;
    let mut ann_pet = F::ZERO;
    let mut ann_stream = F::ZERO;
    let mut ann_ppt = F::ZERO;

    for m in 0..12 {
        ann_ppt += year_weather[m].precip_cm.max(F::ZERO);
        let month = (m + 1) as u8;
        let ev = compiled.events_for_cursor(year, month, block_cursor);
        let (mo, audit) = step_month_resolved(
            state,
            derived,
            biome,
            selection.as_deref_mut(),
            cached,
            compiled.label_year,
            ev,
            &year_weather[m],
            cached_pet.map(|p| p[m]),
            year,
            month,
            co2_ppm,
            config,
            ledger,
        );
        if let Some(rep) = audit {
            debug_assert!(
                rep.is_conserved(config.elements.nelem(), F::MASS_TOL),
                "Mass conservation violation at year {year} month {}: {rep:?}",
                m + 1
            );
        }
        ann_npp += mo.npp;
        ann_resp += mo.resp;
        ann_aet += mo.aet;
        ann_pet += mo.pet;
        ann_stream += state.stream;
    }

    state.ann_ppt = ann_ppt.max(F::from_f64(1.0));
    state.ann_aet = ann_aet;

    AnnualOutput {
        year,
        somsc: state.somsc(),
        som1c_soil: state.som1c[1].total(),
        som2c: state.som2c.total(),
        som3c: state.som3c.total(),
        aglivc: state.aglivc.total(),
        bglivc: state.bglivc.total(),
        rleavc: state.rleavc.total(),
        rlwodc: state.rlwodc.total(),
        total_minerl_n: state.mineral_sum(derived.nlayer, 0),
        annual_npp: ann_npp,
        annual_resp: ann_resp,
        annual_aet: ann_aet,
        annual_pet: ann_pet,
        annual_stream: ann_stream,
    }
}

/// Continue a multi-year simulation from an existing `(CellState, CellDerivedParams)` using
/// a pre-compiled [`CompiledSchedule`] (zero per-month allocation, scan, or HashMap lookup).
#[allow(clippy::too_many_arguments)]
pub fn run_cell_from_state_compiled<'a, F: Float>(
    mut state: CellState<F>,
    derived: CellDerivedParams<F>,
    biome: &'a RawBiomeParams<F>,
    weather_by_year: &[[MonthlyWeather<F>; 12]],
    schedule: &'a Schedule,
    compiled: &CompiledSchedule<'a, F>,
    co2_by_year: Option<&[F]>,
    config: &SimConfig,
) -> (CellState<F>, CellDerivedParams<F>, AnnualOutput<F>) {
    assert!(
        !weather_by_year.is_empty(),
        "weather_by_year must not be empty"
    );
    let mut cached = CachedParams {
        crop: if schedule.initial_crop.is_empty() {
            None
        } else {
            biome.crops.get(schedule.initial_crop.as_str())
        },
        tree: if schedule.initial_tree.is_empty() {
            None
        } else {
            biome.trees.get(schedule.initial_tree.as_str())
        },
    };
    let mut ledger = FlowLedger::<F>::new();
    let mut block_cursor = 0usize;

    let default_co2 = config
        .co2_ppm_override
        .map(F::from_f64)
        .unwrap_or(biome.fix.co2ppm[0]);

    let static_pet: Option<[F; 12]> = if weather_by_year.len() == 1 {
        let w0 = &weather_by_year[0];
        Some(core::array::from_fn(|m| {
            linacre_pet((m + 1) as u8, &w0[m], &derived, &biome.fix, config.profile)
        }))
    } else {
        None
    };

    let mut last_out = AnnualOutput::default();
    let total_years = (config.end_year - config.start_year + 1).max(0) as usize;
    for idx in 0..total_years {
        let year = config.start_year + idx as i32;
        let w = &weather_by_year[idx % weather_by_year.len()];
        let co2 = co2_by_year
            .and_then(|arr| arr.get(idx).copied())
            .unwrap_or(default_co2);
        last_out = step_year_compiled(
            &mut state,
            &derived,
            biome,
            None,
            &mut cached,
            compiled,
            &mut block_cursor,
            w,
            static_pet.as_ref(),
            year,
            co2,
            config,
            &mut ledger,
        );
    }

    (state, derived, last_out)
}

/// Continue a multi-year (e.g. transient climate / CO₂) simulation from an existing
/// `(CellState, CellDerivedParams)` checkpoint with zero per-month allocation or HashMap lookup.
#[allow(clippy::too_many_arguments)]
pub fn run_cell_from_state<F: Float>(
    state: CellState<F>,
    derived: CellDerivedParams<F>,
    biome: &RawBiomeParams<F>,
    weather_by_year: &[[MonthlyWeather<F>; 12]],
    schedule: &Schedule,
    co2_by_year: Option<&[F]>,
    config: &SimConfig,
) -> (CellState<F>, CellDerivedParams<F>, AnnualOutput<F>) {
    let compiled = CompiledSchedule::build(schedule, biome);
    run_cell_from_state_compiled(
        state,
        derived,
        biome,
        weather_by_year,
        schedule,
        &compiled,
        co2_by_year,
        config,
    )
}

/// Run a complete multi-year simulation using a pre-compiled [`CompiledSchedule`].
#[allow(clippy::too_many_arguments)]
pub fn run_cell_simulation_compiled<'a, F: Float>(
    biome: &'a RawBiomeParams<F>,
    cell_input: &CellSiteInput<F>,
    weather_by_year: &[[MonthlyWeather<F>; 12]],
    schedule: &'a Schedule,
    compiled: &CompiledSchedule<'a, F>,
    co2_by_year: Option<&[F]>,
    config: &SimConfig,
) -> (CellState<F>, CellDerivedParams<F>, AnnualOutput<F>) {
    assert!(
        !weather_by_year.is_empty(),
        "weather_by_year must not be empty"
    );
    let (state, derived) =
        initialize_cell(biome, cell_input, &weather_by_year[0], schedule, config);
    run_cell_from_state_compiled(
        state,
        derived,
        biome,
        weather_by_year,
        schedule,
        compiled,
        co2_by_year,
        config,
    )
}

/// Run a complete multi-year simulation for a single cell with repeating or
/// year-indexed weather forcing, returning the final `(CellState, CellDerivedParams, AnnualOutput)`.
#[allow(clippy::too_many_arguments)]
pub fn run_cell_simulation<F: Float>(
    biome: &RawBiomeParams<F>,
    cell_input: &CellSiteInput<F>,
    weather_by_year: &[[MonthlyWeather<F>; 12]],
    schedule: &Schedule,
    co2_by_year: Option<&[F]>,
    config: &SimConfig,
) -> (CellState<F>, CellDerivedParams<F>, AnnualOutput<F>) {
    let compiled = CompiledSchedule::build(schedule, biome);
    run_cell_simulation_compiled(
        biome,
        cell_input,
        weather_by_year,
        schedule,
        &compiled,
        co2_by_year,
        config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CompatibilityProfile, ElementMode, IsotopeMode};
    use crate::params::{
        CropParams, CultParams, FertParams, FireParams, GrazParams, HarvParams, IrriParams,
        OmadParams, TreeParams, TremParams,
    };
    use crate::subsystems::scheduler::{ScheduleBlock, ScheduledEvent};

    fn synthetic_biome_f64() -> RawBiomeParams<f64> {
        let mut b = RawBiomeParams::<f64>::default();
        let crop = CropParams::<f64> {
            himax: 0.45,
            hiwsf: 0.5,
            himon: [7, 8],
            snfxmx: 0.02,
            vlossp: 0.05,
            ..CropParams::default()
        };
        b.crops.insert("TMC3".into(), crop);
        b.trees.insert("MTCF".into(), TreeParams::default());
        b.cults.insert(
            "PLOW".into(),
            CultParams {
                name: "PLOW".into(),
                cultra: [0.0, 0.1, 0.9, 0.1, 0.9, 0.9, 1.0],
                clteff: [1.6, 1.6, 1.6, 1.6],
            },
        );
        b.ferts.insert(
            "NPK".into(),
            FertParams {
                name: "NPK".into(),
                feramt: [10.0, 3.0, 2.0],
                aufert: 0.0,
            },
        );
        b.fires.insert(
            "BURN".into(),
            FireParams {
                name: "BURN".into(),
                flfrem: 0.6,
                fdfrem: [0.8, 0.5],
                fret: [0.2, 0.5, 0.4],
                frtsh: 0.2,
                fnue: [1.0, 1.0],
            },
        );
        b.grazs.insert(
            "MOD".into(),
            GrazParams {
                name: "MOD".into(),
                flgrem: 0.3,
                fdgrem: 0.2,
                gfcret: 0.35,
                gret: [0.8, 0.85, 0.8],
                grzeff: 1,
                fecf: [0.5, 0.9, 0.5],
                feclig: 0.25,
            },
        );
        b.harvs.insert(
            "GRAIN".into(),
            HarvParams {
                name: "GRAIN".into(),
                aglrem: 0.0,
                bglrem: 0.0,
                flghrv: 1,
                rmvstr: 0.5,
                remwsd: 0.5,
                hibg: 0.0,
            },
        );
        b.irris.insert(
            "AUTO".into(),
            IrriParams {
                name: "AUTO".into(),
                auirri: 1,
                fawhc: 0.8,
                irraut: 5.0,
                irramt: 0.0,
            },
        );
        b.omads.insert(
            "MANURE".into(),
            OmadParams {
                name: "MANURE".into(),
                astgc: 150.0,
                astlbl: 0.1,
                astlig: 0.18,
                astrec: [18.0, 120.0, 150.0],
            },
        );
        b.trems.insert(
            "LOG".into(),
            TremParams {
                name: "LOG".into(),
                evntyp: 0,
                remf: [0.9, 0.8, 0.85, 0.0, 0.0],
                fd: [0.7, 0.2],
                retf: [
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            },
        );
        b
    }

    fn sample_weather_f64() -> [MonthlyWeather<f64>; 12] {
        let p = [5.0, 5.5, 6.5, 8.0, 9.5, 9.0, 7.5, 7.0, 6.5, 5.5, 5.0, 5.0];
        let tn = [
            -4.0, -2.0, 2.0, 7.0, 12.0, 16.0, 19.0, 18.0, 13.0, 7.0, 2.0, -3.0,
        ];
        let tx = [
            4.0, 7.0, 12.0, 18.0, 23.0, 28.0, 31.0, 30.0, 25.0, 19.0, 11.0, 5.0,
        ];
        core::array::from_fn(|m| MonthlyWeather {
            precip_cm: p[m],
            tmin_c: tn[m],
            tmax_c: tx[m],
        })
    }

    #[test]
    fn cell_state_fits_in_l1d_cache() {
        let sz_f32 = core::mem::size_of::<CellState<f32>>();
        let sz_f64 = core::mem::size_of::<CellState<f64>>();
        assert!(sz_f32 <= 4096, "CellState<f32> too large: {sz_f32} B");
        assert!(sz_f64 <= 8192, "CellState<f64> too large: {sz_f64} B");
        assert_eq!(core::mem::align_of::<CellState<f64>>(), 64);
    }

    #[test]
    fn strict_mass_balance_all_events_full_cnps_f64() {
        let mut biome = synthetic_biome_f64();
        biome.site_defaults.nelem = 3;
        biome.site_defaults.sirri = 15.0;
        let cell_in = CellSiteInput::from_site(&biome.site_defaults);
        let weather = sample_weather_f64();

        let schedule = Schedule {
            start_year: 1,
            last_year: 5,
            site_file: "test.100".into(),
            isotope: IsotopeMode::C14,
            label_year: 2,
            initial_system: SystemKind::Savanna,
            initial_crop: "TMC3".into(),
            initial_tree: "MTCF".into(),
            blocks: vec![ScheduleBlock {
                block_num: 1,
                label: "All Management Events".into(),
                end_year: 5,
                repeat_years: 1,
                weather_mode: 'M',
                events: vec![
                    ScheduledEvent {
                        rel_year: 1,
                        month: 3,
                        kind: EventKind::Cult("PLOW".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 4,
                        kind: EventKind::Fert("NPK".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 4,
                        kind: EventKind::Omad("MANURE".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 4,
                        kind: EventKind::Pltm,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 4,
                        kind: EventKind::Tfst,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 6,
                        kind: EventKind::Graz("MOD".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 7,
                        kind: EventKind::Irri("AUTO".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 8,
                        kind: EventKind::Harv("GRAIN".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 9,
                        kind: EventKind::Fire("BURN".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 9,
                        kind: EventKind::Erod(2.5),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 10,
                        kind: EventKind::Trem("LOG".into()),
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 10,
                        kind: EventKind::Senm,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 10,
                        kind: EventKind::Last,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 10,
                        kind: EventKind::Tlst,
                    },
                ],
            }],
        };

        let cfg = SimConfig {
            profile: CompatibilityProfile::ScientificClean,
            elements: ElementMode::FullCnps,
            isotope: IsotopeMode::C14,
            co2_ppm_override: Some(450.0),
            audit_mass_balance: true,
            start_year: 1,
            end_year: 5,
        };

        let (mut state, derived) = initialize_cell(&biome, &cell_in, &weather, &schedule, &cfg);
        let mut sel = ActiveSelection::from_schedule(&schedule);

        for yr in 1..=5 {
            for m in 0..12 {
                let (_mo, audit) = step_month(
                    &mut state,
                    &derived,
                    &biome,
                    &mut sel,
                    &schedule,
                    &weather[m],
                    yr,
                    (m + 1) as u8,
                    450.0,
                    &cfg,
                );
                let rep = audit.expect("audit enabled");
                assert!(
                    rep.is_conserved(3, 1.0e-10),
                    "Mass balance failed at yr={yr} m={}: {rep:?}",
                    m + 1
                );
            }
        }
        assert!(
            state.total_labeled_carbon() > 0.0,
            "C14 labeling should accumulate"
        );
        assert!(state.somsc() > 0.0 && state.somsc().is_finite());
    }

    #[test]
    fn compatibility_profiles_diverge_cleanly_on_documented_deltas() {
        let biome = synthetic_biome_f64();
        let cell_in = CellSiteInput {
            lat: 65.0,
            rock_frac: 0.25,
            zminrl_cm: Some(165.0),
            ..CellSiteInput::from_site(&biome.site_defaults)
        };
        let mean_t = [0.0; 12];

        let clean = CellDerivedParams::compute(
            &biome.fix,
            &biome.site_defaults,
            &cell_in,
            5000.0,
            &mean_t,
            CompatibilityProfile::ScientificClean,
        );
        let legacy = CellDerivedParams::compute(
            &biome.fix,
            &biome.site_defaults,
            &cell_in,
            5000.0,
            &mean_t,
            CompatibilityProfile::VemapV45Parity,
        );
        assert!(clean.effective_rockfrac > 0.2);
        assert_eq!(legacy.effective_rockfrac, 0.0);
        assert!((clean.afiel[0] - legacy.afiel[0]).abs() > 1.0e-3);
    }

    #[test]
    fn anaerobic_drainage_reduces_decomposition() {
        let mut b_well = synthetic_biome_f64();
        b_well.site_defaults.drain = 1.0;
        let mut b_poor = synthetic_biome_f64();
        b_poor.site_defaults.drain = 0.1;

        let wet_weather = [MonthlyWeather {
            precip_cm: 25.0,
            tmin_c: 12.0,
            tmax_c: 22.0,
        }; 12];
        let sched = Schedule::default();
        let cfg = SimConfig {
            start_year: 1,
            end_year: 20,
            ..SimConfig::default()
        };

        let (st_well, _, _) = run_cell_simulation(
            &b_well,
            &CellSiteInput::from_site(&b_well.site_defaults),
            &[wet_weather],
            &sched,
            None,
            &cfg,
        );
        let (st_poor, _, _) = run_cell_simulation(
            &b_poor,
            &CellSiteInput::from_site(&b_poor.site_defaults),
            &[wet_weather],
            &sched,
            None,
            &cfg,
        );
        assert!(
            st_poor.somsc() > st_well.somsc(),
            "Poor drainage (anerb < 1) should preserve more SOM: poor={} well={}",
            st_poor.somsc(),
            st_well.somsc()
        );
    }

    #[test]
    fn compiled_schedule_matches_step_month_exactly() {
        let biome = synthetic_biome_f64();
        let cell_in = CellSiteInput::from_site(&biome.site_defaults);
        let weather = [MonthlyWeather {
            precip_cm: 7.0,
            tmin_c: 5.0,
            tmax_c: 19.0,
        }; 12];
        let schedule = Schedule {
            start_year: 1,
            last_year: 6,
            initial_system: SystemKind::Savanna,
            initial_crop: "TMC3".into(),
            initial_tree: "MTCF".into(),
            blocks: vec![
                ScheduleBlock {
                    block_num: 1,
                    label: "Block 1".into(),
                    end_year: 3,
                    repeat_years: 2,
                    weather_mode: 'M',
                    events: vec![
                        ScheduledEvent {
                            rel_year: 1,
                            month: 4,
                            kind: EventKind::Frst,
                        },
                        ScheduledEvent {
                            rel_year: 1,
                            month: 4,
                            kind: EventKind::Tfst,
                        },
                        ScheduledEvent {
                            rel_year: 1,
                            month: 7,
                            kind: EventKind::Graz("MOD".into()),
                        },
                        ScheduledEvent {
                            rel_year: 2,
                            month: 5,
                            kind: EventKind::Fert("NPK".into()),
                        },
                        ScheduledEvent {
                            rel_year: 2,
                            month: 10,
                            kind: EventKind::Senm,
                        },
                        ScheduledEvent {
                            rel_year: 2,
                            month: 11,
                            kind: EventKind::Last,
                        },
                    ],
                },
                ScheduleBlock {
                    block_num: 2,
                    label: "Block 2".into(),
                    end_year: 6,
                    repeat_years: 1,
                    weather_mode: 'M',
                    events: vec![
                        ScheduledEvent {
                            rel_year: 1,
                            month: 5,
                            kind: EventKind::Frst,
                        },
                        ScheduledEvent {
                            rel_year: 1,
                            month: 9,
                            kind: EventKind::Fire("BURN".into()),
                        },
                    ],
                },
            ],
            ..Schedule::default()
        };
        let cfg = SimConfig {
            profile: CompatibilityProfile::ScientificClean,
            elements: ElementMode::FullCnps,
            start_year: 1,
            end_year: 6,
            ..SimConfig::default()
        };

        // Run via uncompiled per-month API.
        let (mut st_uncomp, der) = initialize_cell(&biome, &cell_in, &weather, &schedule, &cfg);
        let mut sel = ActiveSelection::from_schedule(&schedule);
        let co2 = biome.fix.co2ppm[0];
        for year in 1..=6 {
            for m in 0..12 {
                let _ = step_month(
                    &mut st_uncomp,
                    &der,
                    &biome,
                    &mut sel,
                    &schedule,
                    &weather[m],
                    year,
                    (m + 1) as u8,
                    co2,
                    &cfg,
                );
            }
        }

        // Run via compiled schedule API.
        let compiled = CompiledSchedule::build(&schedule, &biome);
        let (st_comp, _, _) = run_cell_simulation_compiled(
            &biome,
            &cell_in,
            &[weather],
            &schedule,
            &compiled,
            None,
            &cfg,
        );

        assert!((st_uncomp.somsc() - st_comp.somsc()).abs() < 1.0e-12);
        assert!((st_uncomp.aglivc.total() - st_comp.aglivc.total()).abs() < 1.0e-12);
        assert!((st_uncomp.bglivc.total() - st_comp.bglivc.total()).abs() < 1.0e-12);
        assert!((st_uncomp.rlwodc.total() - st_comp.rlwodc.total()).abs() < 1.0e-12);
        assert!((st_uncomp.asmos[0] - st_comp.asmos[0]).abs() < 1.0e-12);
        assert!((st_uncomp.minerl[0][0] - st_comp.minerl[0][0]).abs() < 1.0e-12);
    }

    #[test]
    fn snowpack_melts_above_tmelt0_even_below_freezing() {
        let mut biome = synthetic_biome_f64();
        biome.fix.tmelt = [-6.0, 2.0];
        let cell_in = CellSiteInput::from_site(&biome.site_defaults);
        let cold = [MonthlyWeather {
            precip_cm: 5.0,
            tmin_c: -11.0,
            tmax_c: -3.0, // tmean = -7.0 <= tmelt[0] (-6.0): accumulates snow
        }; 12];
        let mild_freeze = MonthlyWeather {
            precip_cm: 5.0,
            tmin_c: -6.0,
            tmax_c: 2.0, // tmean = -2.0 > tmelt[0] (-6.0) but <= 0: melts into soil
        };
        let sched = Schedule::default();
        let cfg = SimConfig::default();
        let (mut st, der) = initialize_cell(&biome, &cell_in, &cold, &sched, &cfg);
        let mut sel = ActiveSelection::from_schedule(&sched);
        let co2 = biome.fix.co2ppm[0];

        // Month 11 at -2 C: all 5 cm should melt into soil (snow == 0, asmos[0] increases).
        let asmos_before = st.asmos[0];
        let _ = step_month(
            &mut st,
            &der,
            &biome,
            &mut sel,
            &sched,
            &mild_freeze,
            1,
            11,
            co2,
            &cfg,
        );
        assert_eq!(st.snow, 0.0, "Snow should melt when tmean > tmelt[0]");
        assert!(
            st.asmos[0] > asmos_before,
            "Melted snow must infiltrate into soil"
        );

        // Month 12 at -7 C: below tmelt[0], snow must accumulate minus sublimation.
        let _ = step_month(
            &mut st, &der, &biome, &mut sel, &sched, &cold[11], 1, 12, co2, &cfg,
        );
        assert!(
            st.snow > 3.5 && st.snow < 5.0,
            "Snow should accumulate below tmelt[0]: snow={}",
            st.snow
        );
    }

    #[test]
    fn deciduous_mixed_forest_drops_leaves_once_per_autumn() {
        let mut biome = synthetic_biome_f64();
        let mut mixed = TreeParams::<f64> {
            name: "MIXED".into(),
            decid: 1,
            leafdr: [0.01; 12],
            ..TreeParams::default()
        };
        mixed.wooddr[0] = 0.50; // 50% deciduous, 50% evergreen
        biome.trees.insert("MIXED".into(), mixed);
        biome.site_defaults.rlvcis = [200.0, 0.0];
        biome.site_defaults.rleave = [5.0, 0.5, 0.5];
        biome.site_defaults.rlwcis = [5000.0, 0.0];
        biome.site_defaults.rlwode = [25.0, 2.5, 2.5];

        let cell_in = CellSiteInput::from_site(&biome.site_defaults);
        let cool_autumn = [MonthlyWeather {
            precip_cm: 6.0,
            tmin_c: 0.0,
            tmax_c: 8.0, // stemp < 8 C across months 10, 11, 12
        }; 12];
        let sched = Schedule {
            initial_system: SystemKind::Forest,
            initial_tree: "MIXED".into(),
            blocks: vec![ScheduleBlock {
                block_num: 1,
                label: "Autumn".into(),
                end_year: 2,
                repeat_years: 1,
                weather_mode: 'M',
                events: vec![
                    ScheduledEvent {
                        rel_year: 1,
                        month: 11,
                        kind: EventKind::Senm,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 12,
                        kind: EventKind::Tlst,
                    },
                ],
            }],
            ..Schedule::default()
        };
        let cfg = SimConfig::default();
        let (mut st, der) = initialize_cell(&biome, &cell_in, &cool_autumn, &sched, &cfg);
        st.forgrw = false; // No growth so we isolate leaf drop cleanly.
        let mut sel = ActiveSelection::from_schedule(&sched);
        let co2 = biome.fix.co2ppm[0];

        for month in [10u8, 11, 12] {
            let _ = step_month(
                &mut st,
                &der,
                &biome,
                &mut sel,
                &sched,
                &cool_autumn[(month - 1) as usize],
                1,
                month,
                co2,
                &cfg,
            );
        }
        // Should retain ~50% * 0.99 * 0.99 ≈ 98 g C (NOT 200 * 0.5^3 = 25 g C!).
        let rem = st.rleavc.total();
        assert!(
            rem > 90.0 && rem < 105.0,
            "Mixed deciduous/evergreen canopy should drop 50% ONCE per autumn, got rleavc={rem}"
        );
    }

    #[test]
    fn southern_hemisphere_deciduous_drops_leaves_in_austral_autumn_not_spring() {
        let mut biome = synthetic_biome_f64();
        let mut mixed = TreeParams::<f64> {
            name: "MIXED".into(),
            decid: 1,
            leafdr: [0.01; 12],
            ..TreeParams::default()
        };
        mixed.wooddr[0] = 0.50;
        biome.trees.insert("MIXED".into(), mixed);
        biome.site_defaults.rlvcis = [200.0, 0.0];
        biome.site_defaults.rleave = [5.0, 0.5, 0.5];
        biome.site_defaults.rlwcis = [5000.0, 0.0];
        biome.site_defaults.rlwode = [25.0, 2.5, 2.5];

        let mut cell_in = CellSiteInput::from_site(&biome.site_defaults);
        cell_in.lat = -35.0; // Southern Hemisphere (austral autumn = months 4..=6)
        let cool_autumn = [MonthlyWeather {
            precip_cm: 6.0,
            tmin_c: 0.0,
            tmax_c: 8.0,
        }; 12];
        let sched = Schedule {
            initial_system: SystemKind::Forest,
            initial_tree: "MIXED".into(),
            blocks: vec![ScheduleBlock {
                block_num: 1,
                label: "Austral Autumn".into(),
                end_year: 2,
                repeat_years: 1,
                weather_mode: 'M',
                events: vec![ScheduledEvent {
                    rel_year: 1,
                    month: 5,
                    kind: EventKind::Senm,
                }],
            }],
            ..Schedule::default()
        };
        let cfg = SimConfig::default();
        let (mut st, der) = initialize_cell(&biome, &cell_in, &cool_autumn, &sched, &cfg);
        st.forgrw = false;
        let mut sel = ActiveSelection::from_schedule(&sched);
        let co2 = biome.fix.co2ppm[0];

        // Austral autumn months 4, 5, 6 should trigger leaf drop ONCE (not reset mid-autumn).
        for month in [4u8, 5, 6] {
            let _ = step_month(
                &mut st,
                &der,
                &biome,
                &mut sel,
                &sched,
                &cool_autumn[(month - 1) as usize],
                1,
                month,
                co2,
                &cfg,
            );
        }
        let rem = st.rleavc.total();
        assert!(
            rem > 90.0 && rem < 105.0,
            "Southern Hemisphere deciduous forest should drop 50% ONCE in months 4..=6, got rleavc={rem}"
        );
        // Austral spring (months 9, 10) must reset latch without triggering another autumn drop.
        for month in [9u8, 10] {
            let _ = step_month(
                &mut st,
                &der,
                &biome,
                &mut sel,
                &sched,
                &cool_autumn[(month - 1) as usize],
                1,
                month,
                co2,
                &cfg,
            );
        }
        assert!(
            st.rleavc.total() > 85.0 && !st.decid_dropped,
            "Austral spring must reset decid_dropped without dropping leaves, got rleavc={}",
            st.rleavc.total()
        );
    }

    #[test]
    fn management_harv_cult_trem_match_century_manual_parameter_semantics() {
        let mut biome = synthetic_biome_f64();
        let crop = biome.crops.get("TMC3").unwrap().clone();
        let tree = biome.trees.get("MTCF").unwrap().clone();
        biome.site_defaults.aglcis = [100.0, 0.0];
        biome.site_defaults.aglive = [4.0, 0.4, 0.4];
        biome.site_defaults.bglcis = [80.0, 0.0];
        biome.site_defaults.bglive = [2.0, 0.2, 0.2];
        biome.site_defaults.stdcis = [50.0, 0.0];
        biome.site_defaults.stdede = [1.0, 0.1, 0.1];
        biome.site_defaults.rlvcis = [200.0, 0.0];
        biome.site_defaults.rleave = [4.0, 0.4, 0.4];
        biome.site_defaults.fbrcis = [300.0, 0.0];
        biome.site_defaults.fbrche = [3.0, 0.3, 0.3];
        biome.site_defaults.rlwcis = [1000.0, 0.0];
        biome.site_defaults.rlwode = [5.0, 0.5, 0.5];

        let cell_in = CellSiteInput::from_site(&biome.site_defaults);
        let w = sample_weather_f64();
        let sched = Schedule::default();
        let cfg = SimConfig {
            elements: ElementMode::FullCnps,
            ..SimConfig::default()
        };

        // 1. HARV ANN (AGLREM=0, BGLREM=0 -> 100% affected / killed, Manual p. A2-17).
        let (mut st_h, _der) = initialize_cell(&biome, &cell_in, &w, &sched, &cfg);
        let mut ledger = FlowLedger::<f64>::new();
        let ann_harv = HarvParams {
            name: "ANN".into(),
            aglrem: 0.0,
            bglrem: 0.0,
            flghrv: 0,
            rmvstr: 0.0,
            remwsd: 0.5,
            hibg: 0.0,
        };
        apply_harvest(
            &mut st_h,
            &biome.fix,
            &crop,
            &ann_harv,
            ElementMode::FullCnps,
            &mut ledger,
        );
        assert!(
            st_h.aglivc.total() < 1.0e-9 && st_h.bglivc.total() < 1.0e-9,
            "HARV ANN (aglrem=0, bglrem=0) must terminate 100% of live shoots and roots: ag={} bg={}",
            st_h.aglivc.total(),
            st_h.bglivc.total()
        );

        // 2. CULT P (CULTRA = [0.0, 0.1, 0.9, 0.1, 0.9, 0.9, 1.0], Manual p. A2-4).
        let (mut st_c, _) = initialize_cell(&biome, &cell_in, &w, &sched, &cfg);
        let plow = biome.cults.get("PLOW").unwrap().clone();
        apply_cultivation(
            &mut st_c,
            &biome.fix,
            &plow,
            ElementMode::FullCnps,
            &mut ledger,
        );
        assert!(
            st_c.aglivc.total() < 1.0e-9 && st_c.bglivc.total() < 1.0e-9,
            "CULT P must plow down 100% of shoots (0.1 surf + 0.9 soil) and roots: ag={} bg={}",
            st_c.aglivc.total(),
            st_c.bglivc.total()
        );

        // 3. TREM BURN (RETF[part][C,N,P,S] = [0.0, 0.30, 1.00, 0.00], Manual p. A2-24).
        let (mut st_t, _) = initialize_cell(&biome, &cell_in, &w, &sched, &cfg);
        let burn = TremParams {
            name: "BURN".into(),
            evntyp: 1,
            remf: [0.9, 0.9, 0.7, 0.7, 0.7],
            fd: [0.9, 0.7],
            retf: [
                [0.0, 0.30, 1.0, 0.0],
                [0.0, 0.30, 1.0, 0.0],
                [0.0, 0.30, 1.0, 0.0],
            ],
        };
        let min_n_before = st_t.minerl[0][0];
        let min_p_before = st_t.minerl[0][1];
        apply_trem(
            &mut st_t,
            &biome.fix,
            Some(&tree),
            &burn,
            ElementMode::FullCnps,
            &mut ledger,
        );
        // Expected N ash = 0.30 * (0.9*4.0 + 0.9*3.0 + 0.7*5.0) = 0.30 * 9.8 = 2.94 g N/m2.
        let n_ash = st_t.minerl[0][0] - min_n_before;
        let p_ash = st_t.minerl[0][1] - min_p_before;
        assert!(
            (n_ash - 2.94).abs() < 1.0e-9,
            "TREM BURN must return 30% of burned N as ash (2.94), got {n_ash}"
        );
        assert!(
            (p_ash - 0.98).abs() < 1.0e-9,
            "TREM BURN must return 100% of burned P as ash (0.98), got {p_ash}"
        );
    }

    #[test]
    fn direct_transfers_reject_non_finite_and_compiled_block_guards_year_range() {
        use crate::ledger::{
            export_c_direct, export_e_direct, transfer_c_direct, transfer_e_direct,
        };

        let mut src_c = CarbonPool::from_total(100.0f64, 0.0);
        let mut dst_c = CarbonPool::from_total(10.0f64, 0.0);
        let mut out_c = 0.0f64;
        assert_eq!(transfer_c_direct(&mut src_c, &mut dst_c, f64::NAN), 0.0);
        assert_eq!(export_c_direct(&mut src_c, &mut out_c, f64::INFINITY), 0.0);
        assert_eq!(src_c.total(), 100.0);

        let mut src_e = 5.0f64;
        let mut dst_e = 1.0f64;
        let mut out_e = 0.0f64;
        assert_eq!(transfer_e_direct(&mut src_e, &mut dst_e, f64::NAN), 0.0);
        assert_eq!(export_e_direct(&mut src_e, &mut out_e, f64::INFINITY), 0.0);
        assert_eq!(src_e, 5.0);

        // CompiledBlock::events_for must return default (not wrap usize) when year < start_year.
        let biome = synthetic_biome_f64();
        let sched = Schedule {
            start_year: 1900,
            last_year: 2000,
            ..Schedule::default()
        };
        let compiled = CompiledSchedule::build(&sched, &biome);
        let ev_before = compiled.blocks[0].events_for(1850, 4);
        assert!(ev_before.set_crpgrw.is_none() && !ev_before.has_any_mgmt);

        // initialize_cell must populate avh2o from initial rwcf so Month 1 Year 1 is accurate.
        let mut b2 = synthetic_biome_f64();
        b2.site_defaults.rwcf = [0.5; crate::params::MAX_LAYERS];
        let (st, _der) = initialize_cell(
            &b2,
            &CellSiteInput::from_site(&b2.site_defaults),
            &sample_weather_f64(),
            &Schedule::default(),
            &SimConfig::default(),
        );
        assert!(
            st.avh2o[0] > 0.0 && st.avh2o[2] > 0.0,
            "initialize_cell must initialize avh2o from rwcf, got {:?}",
            st.avh2o
        );
    }
}
