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
    hydrology::{linacre_pet, step_hydrology},
    management::{
        apply_cultivation, apply_erosion, apply_fertilizer, apply_fire, apply_grazing,
        apply_harvest, apply_irrigation, apply_omad, apply_trem,
    },
    mineralization::{apply_atmospheric_inputs, partition_residue},
    phosphorus_sulfur::step_phosphorus_sulfur,
    scheduler::{EventKind, Schedule},
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

    // 4. Initialize soil water per layer from relative water content `rwcf`.
    for l in 0..derived.nlayer {
        let wp = derived.awilt[l] * derived.adep[l];
        let fc = derived.afiel[l] * derived.adep[l];
        let rw = site.rwcf[l].clamp_val(F::ZERO, F::ONE);
        state.asmos[l] = wp + rw * (fc - wp);
    }

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
    step_month_inner(
        state,
        derived,
        biome,
        selection,
        &mut cached,
        schedule,
        weather,
        year,
        month,
        co2_ppm,
        config,
        &mut ledger,
    )
}

#[allow(clippy::too_many_arguments)]
fn step_month_inner<'a, F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    biome: &'a RawBiomeParams<F>,
    selection: &mut ActiveSelection,
    cached: &mut CachedParams<'a, F>,
    schedule: &Schedule,
    weather: &MonthlyWeather<F>,
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
    state.senm = false;

    // Update isotope atmospheric labeled fraction when labeling is active.
    state.atm_lab_frac = match config.isotope {
        crate::config::IsotopeMode::Unlabeled => F::ZERO,
        crate::config::IsotopeMode::C14 => {
            if schedule.label_year >= 0 && year >= schedule.label_year {
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

    // 1. Resolve scheduled events for (year, month) using a zero-allocation stack buffer.
    let mut events_buf: [Option<&EventKind>; 24] = [None; 24];
    let n_events = schedule.events_for_buf(year, month, &mut events_buf);

    let mut irri_opt: Option<&str> = None;
    let mut fert_opt: Option<&str> = None;
    let mut omad_opt: Option<&str> = None;
    let mut cult_opt: Option<&str> = None;
    let mut graz_opt: Option<&str> = None;
    let mut harv_opt: Option<&str> = None;
    let mut fire_opt: Option<&str> = None;
    let mut trem_opt: Option<&str> = None;
    let mut erod_rate: Option<F> = None;
    let mut planted = false;

    for ev in events_buf[..n_events].iter().flatten() {
        match ev {
            EventKind::Crop(name) => {
                if selection.crop_name != *name {
                    selection.crop_name.clone_from(name);
                    cached.crop = biome.crops.get(name.as_str());
                }
                if state.system == SystemKind::Forest {
                    state.system = SystemKind::Savanna;
                }
            }
            EventKind::Tree(name) => {
                if selection.tree_name != *name {
                    selection.tree_name.clone_from(name);
                    cached.tree = biome.trees.get(name.as_str());
                }
                if state.system == SystemKind::GrasslandCrop {
                    state.system = SystemKind::Savanna;
                }
            }
            EventKind::Frst => state.crpgrw = true,
            EventKind::Last => state.crpgrw = false,
            EventKind::Senm => state.senm = true,
            EventKind::Pltm => {
                state.crpgrw = true;
                planted = true;
            }
            EventKind::Tfst => state.forgrw = true,
            EventKind::Tlst => state.forgrw = false,
            EventKind::Irri(n) => irri_opt = Some(n.as_str()),
            EventKind::Fert(n) => fert_opt = Some(n.as_str()),
            EventKind::Omad(n) => omad_opt = Some(n.as_str()),
            EventKind::Cult(n) => cult_opt = Some(n.as_str()),
            EventKind::Graz(n) => graz_opt = Some(n.as_str()),
            EventKind::Harv(n) => harv_opt = Some(n.as_str()),
            EventKind::Fire(n) => fire_opt = Some(n.as_str()),
            EventKind::Trem(n) => trem_opt = Some(n.as_str()),
            EventKind::Erod(r) => erod_rate = Some(f(*r)),
        }
    }

    ledger.clear();

    // Seed crop if planted with zero live shoots.
    if planted && state.aglivc.total() < f(1.0) {
        ledger.schedule(PoolId::ExternalInputC, PoolId::AglivC, f(2.0));
        ledger.schedule(PoolId::ExternalInputC, PoolId::BglivC, f(2.0));
        for e in 0..config.elements.nelem() {
            let ce = [f(25.0), f(250.0), f(250.0)][e];
            ledger.schedule(PoolId::ExternalInputE(e), PoolId::AglivE(e), f(2.0) / ce);
            ledger.schedule(PoolId::ExternalInputE(e), PoolId::BglivE(e), f(2.0) / ce);
        }
        ledger.commit(state);
    }

    let crop_params = cached.crop;
    let tree_params = cached.tree;
    let co2 = compute_co2_effects(co2_ppm, &biome.fix, crop_params, tree_params);

    // 2. Irrigation water calculation (applied into hydrology).
    let irri_cm = if let Some(name) = irri_opt {
        if let Some(ip) = biome.irris.get(name) {
            let pet_est = linacre_pet(month, weather, derived, &biome.fix, config.profile);
            apply_irrigation(
                state,
                derived,
                &biome.site_defaults,
                ip,
                weather.precip_cm.max(F::ZERO),
                pet_est,
                config.elements,
                ledger,
            )
        } else {
            F::ZERO
        }
    } else {
        F::ZERO
    };

    // 3. Hydrology & Soil Temperature.
    let transp_mult = if state.system == SystemKind::Forest {
        co2.forest_transp
    } else {
        co2.crop_transp
    };
    let hydro = step_hydrology(
        state,
        derived,
        &biome.fix,
        weather,
        month,
        irri_cm,
        transp_mult,
        config.elements,
        config.profile,
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
    if let Some(name) = fert_opt {
        if let Some(fp) = biome.ferts.get(name) {
            apply_fertilizer(state, fp, config.elements, ledger);
        }
    }
    if let Some(name) = omad_opt {
        if let Some(op) = biome.omads.get(name) {
            apply_omad(state, &biome.fix, op, config.elements, ledger);
        }
    }
    if let Some(name) = cult_opt {
        if let Some(cp) = biome.cults.get(name) {
            apply_cultivation(state, &biome.fix, cp, config.elements, ledger);
        }
    }

    // 5. Forest and Crop vegetation production & senescence.
    // Per Manual §3.7.3 (Savanna): compute canopy shading and basal-area N share;
    // woody canopy accesses its share of mineral N while understory grass is shaded
    // by tree LAI and constrained by remaining soil mineral N.
    let (tree_shade_fac, grass_n_share) = if state.system == SystemKind::Savanna {
        if let Some(tp) = tree_params {
            let (_lai, shade, g_share) = savanna_competition(state, tp);
            (shade, g_share)
        } else {
            (F::ONE, F::ONE)
        }
    } else {
        (F::ONE, F::ONE)
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
                hydro.liquid_input,
                &co2,
                config.elements,
                ledger,
            );
        }
    }

    if state.system.has_crop() {
        if let Some(cp) = crop_params {
            step_crop(
                state,
                derived,
                &biome.fix,
                cp,
                weather,
                hydro.liquid_input,
                tree_shade_fac,
                grass_n_share,
                &co2,
                config.elements,
                ledger,
            );
        }
    }

    // 6. Management disturbances (GRAZ, HARV, FIRE, TREM, EROD).
    if let Some(name) = graz_opt {
        if let Some(gp) = biome.grazs.get(name) {
            apply_grazing(state, &biome.fix, gp, config.elements, ledger);
        }
    }
    if let Some(name) = harv_opt {
        if let (Some(hp), Some(cp)) = (biome.harvs.get(name), crop_params) {
            apply_harvest(state, &biome.fix, cp, hp, config.elements, ledger);
        }
    }
    if let Some(name) = fire_opt {
        if let Some(fp) = biome.fires.get(name) {
            apply_fire(state, fp, config.elements, ledger);
        }
    }
    if let Some(name) = trem_opt {
        if let Some(tp) = biome.trems.get(name) {
            apply_trem(state, &biome.fix, tree_params, tp, config.elements, ledger);
        }
    }
    if let Some(er) = erod_rate {
        apply_erosion(state, derived, &biome.fix, er, config.elements, ledger);
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
    step_phosphorus_sulfur(
        state,
        derived,
        &biome.fix,
        &biome.site_defaults,
        config.elements,
        ledger,
    );

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
    step_year_inner(
        state,
        derived,
        biome,
        selection,
        &mut cached,
        schedule,
        year_weather,
        year,
        co2_ppm,
        config,
        &mut ledger,
    )
}

#[allow(clippy::too_many_arguments)]
fn step_year_inner<'a, F: Float>(
    state: &mut CellState<F>,
    derived: &CellDerivedParams<F>,
    biome: &'a RawBiomeParams<F>,
    selection: &mut ActiveSelection,
    cached: &mut CachedParams<'a, F>,
    schedule: &Schedule,
    year_weather: &[MonthlyWeather<F>; 12],
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
        let (mo, audit) = step_month_inner(
            state,
            derived,
            biome,
            selection,
            cached,
            schedule,
            &year_weather[m],
            year,
            (m + 1) as u8,
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

/// Continue a multi-year (e.g. transient climate / CO₂) simulation from an existing
/// `(CellState, CellDerivedParams)` checkpoint with zero per-month allocation or HashMap lookup.
#[allow(clippy::too_many_arguments)]
pub fn run_cell_from_state<F: Float>(
    mut state: CellState<F>,
    derived: CellDerivedParams<F>,
    biome: &RawBiomeParams<F>,
    weather_by_year: &[[MonthlyWeather<F>; 12]],
    schedule: &Schedule,
    co2_by_year: Option<&[F]>,
    config: &SimConfig,
) -> (CellState<F>, CellDerivedParams<F>, AnnualOutput<F>) {
    assert!(
        !weather_by_year.is_empty(),
        "weather_by_year must not be empty"
    );
    let mut selection = ActiveSelection::from_schedule(schedule);
    let mut cached = CachedParams::resolve(biome, &selection);
    let mut ledger = FlowLedger::<F>::new();

    let default_co2 = config
        .co2_ppm_override
        .map(F::from_f64)
        .unwrap_or(biome.fix.co2ppm[0]);

    let mut last_out = AnnualOutput::default();
    let total_years = (config.end_year - config.start_year + 1).max(0) as usize;
    for idx in 0..total_years {
        let year = config.start_year + idx as i32;
        let w = &weather_by_year[idx % weather_by_year.len()];
        let co2 = co2_by_year
            .and_then(|arr| arr.get(idx).copied())
            .unwrap_or(default_co2);
        last_out = step_year_inner(
            &mut state,
            &derived,
            biome,
            &mut selection,
            &mut cached,
            schedule,
            w,
            year,
            co2,
            config,
            &mut ledger,
        );
    }

    (state, derived, last_out)
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
    assert!(
        !weather_by_year.is_empty(),
        "weather_by_year must not be empty"
    );
    let (state, derived) =
        initialize_cell(biome, cell_input, &weather_by_year[0], schedule, config);
    run_cell_from_state(
        state,
        derived,
        biome,
        weather_by_year,
        schedule,
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
                cultra: [0.2, 0.5, 0.2, 0.5, 0.6, 0.6, 0.3],
                clteff: [1.5, 1.5, 1.5, 1.5],
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
                aglrem: 0.9,
                bglrem: 0.5,
                flghrv: 1,
                rmvstr: 0.3,
                remwsd: 0.2,
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
                remf: [0.8, 0.7, 0.85, 0.3, 0.3],
                fd: [0.6, 0.5],
                retf: [[0.2; 4]; 3],
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
}
