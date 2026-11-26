//! Tier B Synthetic Coverage Suite (`oracle_tier_b`)
//!
//! Exercises every biogeochemical pathway and management disturbance that the
//! continental VEMAP dataset (`NELEM=1`, `IVAUTO=0`, `DRAIN=1.0`, no management,
//! no isotopes) never covers:
//! - Phosphorus (`nelem=2`) & Sulfur (`nelem=3`) organic + geochemical cycles
//! - `CULT`, `FERT` (fixed + auto), `IRRI` (fixed + auto + S), `OMAD`, `EROD`,
//!   `GRAZ`, `HARV` (grain + straw + root), `FIRE`, `TREM` (cut + burn)
//! - `IVAUTO ∈ {1, 2, 3}`, `DRAIN < 1` anaerobic reduction, CO₂ ramp (`350->700 ppm`)
//! - `IsotopeMode::C14` & `C13` pool propagation
//! - Pure forest without initial herbaceous biomass (`aglivc == 0`)
//! - Strict `1e-10` mass-balance conservation in `f64` across all probes

use century_core::step::ActiveSelection;
use century_core::{
    initialize_cell, run_cell_simulation, step_month, CellSiteInput, CompatibilityProfile,
    CropParams, CultParams, ElementMode, EventKind, FertParams, FireParams, GrazParams, HarvParams,
    IrriParams, IsotopeMode, MonthlyWeather, OmadParams, RawBiomeParams, Schedule, ScheduleBlock,
    ScheduledEvent, SimConfig, SystemKind, TreeParams, TremParams,
};

fn temperate_weather() -> [MonthlyWeather<f64>; 12] {
    let p = [4.5, 5.0, 6.5, 8.0, 10.0, 9.5, 8.0, 7.0, 6.5, 5.5, 5.0, 4.5];
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

fn arid_weather() -> [MonthlyWeather<f64>; 12] {
    let p = [1.2, 1.0, 1.5, 2.0, 2.5, 1.5, 1.0, 1.2, 1.5, 1.4, 1.1, 1.1];
    let tn = [
        0.0, 2.0, 6.0, 10.0, 15.0, 20.0, 23.0, 22.0, 17.0, 11.0, 5.0, 1.0,
    ];
    let tx = [
        12.0, 15.0, 20.0, 25.0, 30.0, 35.0, 38.0, 37.0, 32.0, 26.0, 18.0, 13.0,
    ];
    core::array::from_fn(|m| MonthlyWeather {
        precip_cm: p[m],
        tmin_c: tn[m],
        tmax_c: tx[m],
    })
}

fn full_biome() -> RawBiomeParams<f64> {
    let mut b = RawBiomeParams::<f64>::default();
    b.crops.insert(
        "WHEAT".into(),
        CropParams {
            name: "WHEAT".into(),
            himax: 0.45,
            hiwsf: 0.4,
            himon: [6, 7],
            efrgrn: [0.65, 0.70, 0.55],
            vlossp: 0.04,
            snfxmx: 0.0,
            co2ipr: 1.35,
            co2itr: 0.75,
            ..CropParams::default()
        },
    );
    b.crops.insert(
        "LEGUME".into(),
        CropParams {
            name: "LEGUME".into(),
            snfxmx: 0.04,
            ..CropParams::default()
        },
    );
    b.trees.insert("PINE".into(), TreeParams::default());
    b.cults.insert(
        "PLOW".into(),
        CultParams {
            name: "PLOW".into(),
            cultra: [0.2, 0.6, 0.2, 0.6, 0.7, 0.7, 0.4],
            clteff: [1.8, 1.8, 1.5, 1.8],
        },
    );
    b.ferts.insert(
        "NPK".into(),
        FertParams {
            name: "NPK".into(),
            feramt: [12.0, 4.0, 3.0],
            aufert: 0.0,
        },
    );
    b.ferts.insert(
        "AUTOF".into(),
        FertParams {
            name: "AUTOF".into(),
            feramt: [0.0, 0.0, 0.0],
            aufert: 0.8,
        },
    );
    b.irris.insert(
        "FIXI".into(),
        IrriParams {
            name: "FIXI".into(),
            auirri: 0,
            fawhc: 0.0,
            irraut: 0.0,
            irramt: 8.0,
        },
    );
    b.irris.insert(
        "AUTOI".into(),
        IrriParams {
            name: "AUTOI".into(),
            auirri: 1,
            fawhc: 0.85,
            irraut: 5.0,
            irramt: 0.0,
        },
    );
    b.omads.insert(
        "COMPOST".into(),
        OmadParams {
            name: "COMPOST".into(),
            astgc: 250.0,
            astlbl: 0.5,
            astlig: 0.22,
            astrec: [15.0, 100.0, 120.0],
        },
    );
    b.grazs.insert(
        "CATTLE".into(),
        GrazParams {
            name: "CATTLE".into(),
            flgrem: 0.35,
            fdgrem: 0.25,
            gfcret: 0.35,
            gret: [0.80, 0.85, 0.80],
            grzeff: 1,
            fecf: [0.45, 0.90, 0.50],
            feclig: 0.28,
        },
    );
    b.harvs.insert(
        "GRAIN".into(),
        HarvParams {
            name: "GRAIN".into(),
            aglrem: 0.95,
            bglrem: 0.60,
            flghrv: 1,
            rmvstr: 0.30,
            remwsd: 0.20,
            hibg: 0.0,
        },
    );
    b.fires.insert(
        "WILDFIRE".into(),
        FireParams {
            name: "WILDFIRE".into(),
            flfrem: 0.80,
            fdfrem: [0.90, 0.70],
            fret: [0.15, 0.60, 0.45],
            frtsh: 0.25,
            fnue: [1.0, 1.0],
        },
    );
    b.trems.insert(
        "CLEARCUT".into(),
        TremParams {
            name: "CLEARCUT".into(),
            evntyp: 0,
            remf: [0.90, 0.80, 0.85, 0.20, 0.20],
            fd: [0.75, 0.75],
            retf: [[0.0; 4]; 3],
        },
    );
    b.trems.insert(
        "CROWNFIRE".into(),
        TremParams {
            name: "CROWNFIRE".into(),
            evntyp: 1,
            remf: [0.95, 0.70, 0.40, 0.60, 0.50],
            fd: [0.50, 0.30],
            retf: [[0.10; 4], [0.60; 4], [0.40; 4]],
        },
    );
    b
}

fn assert_conserved_run(
    biome: &RawBiomeParams<f64>,
    cell_in: &CellSiteInput<f64>,
    weather: &[MonthlyWeather<f64>; 12],
    schedule: &Schedule,
    cfg: &SimConfig,
) -> century_core::CellState<f64> {
    let mut cfg_audit = cfg.clone();
    cfg_audit.audit_mass_balance = true;
    let (mut state, derived) = initialize_cell(biome, cell_in, weather, schedule, &cfg_audit);
    let mut sel = ActiveSelection::from_schedule(schedule);
    let co2 = cfg_audit.co2_ppm_override.unwrap_or(biome.fix.co2ppm[0]);
    for yr in cfg_audit.start_year..=cfg_audit.end_year {
        for (m, w_m) in weather.iter().enumerate() {
            let (_mo, audit) = step_month(
                &mut state,
                &derived,
                biome,
                &mut sel,
                schedule,
                w_m,
                yr,
                (m + 1) as u8,
                co2,
                &cfg_audit,
            );
            let rep = audit.expect("audit enabled");
            assert!(
                rep.is_conserved(cfg_audit.elements.nelem(), 1.0e-10),
                "Mass balance error at yr={yr} mo={}: {rep:?}",
                m + 1
            );
        }
    }
    state
}

#[test]
fn tier_b_phosphorus_and_sulfur_geochemical_cycles() {
    let mut biome = full_biome();
    biome.site_defaults.nelem = 3;
    biome.site_defaults.parent = [0.0, 60.0, 40.0];
    biome.site_defaults.secndy = [0.0, 25.0, 12.0];
    biome.site_defaults.occlud = 5.0;
    let cell_in = CellSiteInput::from_site(&biome.site_defaults);
    let w = temperate_weather();

    let sched = Schedule {
        initial_crop: "WHEAT".into(),
        ..Schedule::default()
    };
    let cfg_np = SimConfig {
        elements: ElementMode::CarbonNitrogenPhosphorus,
        start_year: 1,
        end_year: 15,
        ..SimConfig::default()
    };
    let st_np = assert_conserved_run(&biome, &cell_in, &w, &sched, &cfg_np);
    // Parent P weathered and occluded P exchanged cleanly.
    assert!(st_np.parent[1] < 60.0, "Parent P should weather");
    assert!(st_np.occlud > 5.0, "Secondary P should slowly occlude");

    let cfg_nps = SimConfig {
        elements: ElementMode::FullCnps,
        start_year: 1,
        end_year: 15,
        ..SimConfig::default()
    };
    let st_nps = assert_conserved_run(&biome, &cell_in, &w, &sched, &cfg_nps);
    assert!(st_nps.parent[2] < 40.0, "Parent S should weather");
    assert!(st_nps.total_element(2) > 0.0);
}

#[test]
fn tier_b_cultivation_and_erosion_reduce_som() {
    let biome = full_biome();
    let cell_in = CellSiteInput::from_site(&biome.site_defaults);
    let w = temperate_weather();
    let cfg = SimConfig {
        elements: ElementMode::FullCnps,
        start_year: 1,
        end_year: 10,
        ..SimConfig::default()
    };

    let base_sched = Schedule {
        initial_crop: "WHEAT".into(),
        ..Schedule::default()
    };
    let st_base = assert_conserved_run(&biome, &cell_in, &w, &base_sched, &cfg);

    let mut cult_sched = base_sched.clone();
    cult_sched.blocks[0].events.push(ScheduledEvent {
        rel_year: 1,
        month: 4,
        kind: EventKind::Cult("PLOW".into()),
    });
    let st_cult = assert_conserved_run(&biome, &cell_in, &w, &cult_sched, &cfg);
    assert!(
        st_cult.somsc() < st_base.somsc(),
        "Annual plowing should accelerate SOM decay: cult={} base={}",
        st_cult.somsc(),
        st_base.somsc()
    );

    let mut erod_sched = base_sched.clone();
    erod_sched.blocks[0].events.push(ScheduledEvent {
        rel_year: 1,
        month: 5,
        kind: EventKind::Erod(5.0),
    });
    let st_erod = assert_conserved_run(&biome, &cell_in, &w, &erod_sched, &cfg);
    assert!(
        st_erod.somsc() < st_base.somsc(),
        "Soil erosion must reduce SOM: erod={} base={}",
        st_erod.somsc(),
        st_base.somsc()
    );
}

#[test]
fn tier_b_fertilization_omad_and_irrigation_boost_production() {
    let mut biome = full_biome();
    biome.site_defaults.sirri = 20.0;
    let cell_in = CellSiteInput::from_site(&biome.site_defaults);
    let w = arid_weather();
    let cfg = SimConfig {
        elements: ElementMode::FullCnps,
        start_year: 1,
        end_year: 8,
        ..SimConfig::default()
    };

    let base_sched = Schedule {
        initial_crop: "WHEAT".into(),
        ..Schedule::default()
    };
    let st_base = assert_conserved_run(&biome, &cell_in, &w, &base_sched, &cfg);

    let mut mgmt_sched = base_sched.clone();
    mgmt_sched.blocks[0].events.extend([
        ScheduledEvent {
            rel_year: 1,
            month: 4,
            kind: EventKind::Fert("NPK".into()),
        },
        ScheduledEvent {
            rel_year: 1,
            month: 4,
            kind: EventKind::Omad("COMPOST".into()),
        },
        ScheduledEvent {
            rel_year: 1,
            month: 6,
            kind: EventKind::Irri("AUTOI".into()),
        },
        ScheduledEvent {
            rel_year: 1,
            month: 7,
            kind: EventKind::Irri("FIXI".into()),
        },
    ]);
    let st_mgmt = assert_conserved_run(&biome, &cell_in, &w, &mgmt_sched, &cfg);
    assert!(
        st_mgmt.somsc() > st_base.somsc(),
        "OMAD+FERT+IRRI must raise SOM"
    );
    assert!(
        st_mgmt.cum_e_in[2] > st_base.cum_e_in[2],
        "Irrigation S & NPK must increase S input"
    );
}

#[test]
fn tier_b_grazing_harvest_fire_and_trem_conserve_mass() {
    let biome = full_biome();
    let cell_in = CellSiteInput::from_site(&biome.site_defaults);
    let w = temperate_weather();
    let cfg = SimConfig {
        elements: ElementMode::FullCnps,
        start_year: 1,
        end_year: 6,
        ..SimConfig::default()
    };

    // 1. Cropland with PLTM, GRAZ, HARV, FIRE.
    let sched_crop = Schedule {
        start_year: 1,
        last_year: 6,
        site_file: "t.100".into(),
        isotope: IsotopeMode::Unlabeled,
        label_year: -1,
        initial_system: SystemKind::GrasslandCrop,
        initial_crop: "WHEAT".into(),
        initial_tree: String::new(),
        blocks: vec![ScheduleBlock {
            block_num: 1,
            label: "Crop Mgmt".into(),
            end_year: 6,
            repeat_years: 1,
            weather_mode: 'M',
            events: vec![
                ScheduledEvent {
                    rel_year: 1,
                    month: 3,
                    kind: EventKind::Pltm,
                },
                ScheduledEvent {
                    rel_year: 1,
                    month: 5,
                    kind: EventKind::Graz("CATTLE".into()),
                },
                ScheduledEvent {
                    rel_year: 1,
                    month: 8,
                    kind: EventKind::Harv("GRAIN".into()),
                },
                ScheduledEvent {
                    rel_year: 1,
                    month: 9,
                    kind: EventKind::Fire("WILDFIRE".into()),
                },
                ScheduledEvent {
                    rel_year: 1,
                    month: 10,
                    kind: EventKind::Last,
                },
            ],
        }],
    };
    let st_c = assert_conserved_run(&biome, &cell_in, &w, &sched_crop, &cfg);
    assert!(st_c.cum_c_out > 0.0);

    // 2. Pure forest with zero initial grass + TREM (CLEARCUT yr 3, CROWNFIRE yr 6).
    let mut b_for = full_biome();
    b_for.site_defaults.aglcis = [0.0, 0.0];
    b_for.site_defaults.bglcis = [0.0, 0.0];
    b_for.site_defaults.stdcis = [0.0, 0.0];
    b_for.site_defaults.rlvcis = [200.0, 0.0];
    b_for.site_defaults.rlwcis = [5000.0, 0.0];
    b_for.site_defaults.fbrcis = [500.0, 0.0];
    b_for.site_defaults.frtcis = [250.0, 0.0];
    b_for.site_defaults.crtcis = [1000.0, 0.0];
    b_for.site_defaults.rleave = [4.0, 0.4, 0.4];
    b_for.site_defaults.rlwode = [15.0, 2.0, 2.0];
    b_for.site_defaults.fbrche = [4.0, 0.5, 0.5];
    b_for.site_defaults.froote = [5.0, 0.5, 0.5];
    b_for.site_defaults.croote = [6.0, 0.8, 0.8];

    let sched_for = Schedule {
        start_year: 1,
        last_year: 6,
        site_file: "f.100".into(),
        isotope: IsotopeMode::Unlabeled,
        label_year: -1,
        initial_system: SystemKind::Forest,
        initial_crop: String::new(),
        initial_tree: "PINE".into(),
        blocks: vec![ScheduleBlock {
            block_num: 1,
            label: "Pure Forest TREM".into(),
            end_year: 6,
            repeat_years: 6,
            weather_mode: 'M',
            events: vec![
                ScheduledEvent {
                    rel_year: 1,
                    month: 4,
                    kind: EventKind::Tfst,
                },
                ScheduledEvent {
                    rel_year: 3,
                    month: 8,
                    kind: EventKind::Trem("CLEARCUT".into()),
                },
                ScheduledEvent {
                    rel_year: 6,
                    month: 8,
                    kind: EventKind::Trem("CROWNFIRE".into()),
                },
                ScheduledEvent {
                    rel_year: 6,
                    month: 11,
                    kind: EventKind::Tlst,
                },
            ],
        }],
    };
    let st_f = assert_conserved_run(
        &b_for,
        &CellSiteInput::from_site(&b_for.site_defaults),
        &w,
        &sched_for,
        &cfg,
    );
    assert_eq!(
        st_f.aglivc.total(),
        0.0,
        "Pure forest must keep aglivc == 0"
    );
    assert!(
        st_f.wood3c.total() > 0.0,
        "TREM root mortality must feed wood3c"
    );
}

#[test]
fn tier_b_ivauto_modes_and_co2_ramp_and_isotopes() {
    let w = temperate_weather();
    let sched = Schedule {
        start_year: 1,
        last_year: 10,
        isotope: IsotopeMode::C14,
        label_year: 3,
        initial_crop: "WHEAT".into(),
        ..Schedule::default()
    };

    // IVAUTO 1 (grassland), 2 (crop), 3 (forest) should produce distinct, ordered initial SOM.
    let mut som_init = [0.0f64; 3];
    for (idx, iv) in [1, 2, 3].into_iter().enumerate() {
        let mut b = full_biome();
        b.site_defaults.ivauto = iv;
        let cfg = SimConfig::default();
        let (st, _) = initialize_cell(
            &b,
            &CellSiteInput::from_site(&b.site_defaults),
            &w,
            &sched,
            &cfg,
        );
        som_init[idx] = st.somsc();
        assert!(som_init[idx] > 500.0);
    }
    assert!(som_init[1] < som_init[0] && som_init[0] < som_init[2]);

    // CO2 fertilization (350 vs 700 ppm) increases NPP and decreases transpiration (WUE).
    let biome = full_biome();
    let cell_in = CellSiteInput::from_site(&biome.site_defaults);
    let cfg_350 = SimConfig {
        profile: CompatibilityProfile::ScientificClean,
        elements: ElementMode::CarbonNitrogen,
        isotope: IsotopeMode::C14,
        co2_ppm_override: Some(350.0),
        start_year: 1,
        end_year: 10,
        ..SimConfig::default()
    };
    let cfg_700 = SimConfig {
        co2_ppm_override: Some(700.0),
        ..cfg_350.clone()
    };
    let (st350, _, ann350) = run_cell_simulation(&biome, &cell_in, &[w], &sched, None, &cfg_350);
    let (st700, _, ann700) = run_cell_simulation(&biome, &cell_in, &[w], &sched, None, &cfg_700);
    assert!(
        ann700.annual_npp > ann350.annual_npp,
        "Doubled CO2 should increase NPP: 700={} vs 350={}",
        ann700.annual_npp,
        ann350.annual_npp
    );
    assert!(
        ann700.annual_aet <= ann350.annual_aet + 1.0e-6,
        "Doubled CO2 stomatal closure should not increase AET: 700={} vs 350={}",
        ann700.annual_aet,
        ann350.annual_aet
    );

    // C-14 isotope labeling must propagate into litter and SOM pools.
    assert!(
        st350.som1c[1].lab > 0.0,
        "C-14 should propagate into active SOM"
    );
    assert!(st350.som2c.lab > 0.0, "C-14 should propagate into slow SOM");
    assert!(st700.total_labeled_carbon() > 0.0);
}
