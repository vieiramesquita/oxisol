//! Scientifically calibrated built-in ecosystem presets and synthetic spatial
//! gradient grids so Oxisol Studio runs immediately out of the box without
//! external proprietary files.

use std::collections::HashMap;
use std::sync::Arc;

use oxisol_core::{
    CellSiteInput, CropParams, CultParams, EventKind, FertParams, FireParams, FixParams, Float,
    GrazParams, HarvParams, IrriParams, IsotopeMode, MonthlyWeather, OmadParams, RawBiomeParams,
    Schedule, ScheduleBlock, ScheduledEvent, SiteParams, SystemKind, TreeParams, TremParams,
};
use oxisol_io::vemap::{PreloadedCell, PreloadedGrid};

use crate::models::{IrrigationMode, PresetSummary, SimulateRequest, TillageMode};

/// Generate realistic 12-month climatology `(tmin, tmax, precip)` from summary
/// climate controls, automatically inverting seasonal phase for Southern
/// Hemisphere latitudes (`lat < 0`).
pub fn build_synthetic_climate<F: Float>(
    mean_temp_c: f64,
    temp_amp_c: f64,
    annual_precip_cm: f64,
    lat: f64,
) -> [MonthlyWeather<F>; 12] {
    let f = F::from_f64;
    // Peak warmth in July (month index 6) in NH, January (month index 0) in SH.
    let peak_month = if lat >= 0.0 { 6.0 } else { 0.0 };
    // Moderate summer-wet monsoon seasonality for tropics/grasslands.
    let precip_seasonality = if lat.abs() < 25.0 { 0.55 } else { 0.28 };
    let diurnal_half = 5.5_f64;

    let mut weights = [0.0_f64; 12];
    let mut w_sum = 0.0_f64;
    for (m, w) in weights.iter_mut().enumerate() {
        let phase = 2.0 * std::f64::consts::PI * ((m as f64) - peak_month) / 12.0;
        let val = (1.0 + precip_seasonality * phase.cos()).max(0.15);
        *w = val;
        w_sum += val;
    }

    core::array::from_fn(|m| {
        let phase = 2.0 * std::f64::consts::PI * ((m as f64) - peak_month) / 12.0;
        let t_mean = mean_temp_c + temp_amp_c * phase.cos();
        let p_cm = (annual_precip_cm.max(1.0) * (weights[m] / w_sum)).max(0.05);
        MonthlyWeather {
            precip_cm: f(p_cm),
            tmin_c: f(t_mean - diurnal_half),
            tmax_c: f(t_mean + diurnal_half),
        }
    })
}

/// Build the full library of calibrated management and vegetation option tables
/// used by interactive presets.
pub fn base_biome_params<F: Float>(preset_id: &str) -> RawBiomeParams<F> {
    let f = F::from_f64;
    let mut fix = FixParams::<F>::default();
    let mut site = SiteParams::<F> {
        nelem: 3,
        ivauto: 0,
        swflag: 4,
        nlayer: 6,
        nlaypg: 4,
        epnfa: [f(0.25), f(0.008)],
        epnfs: [f(25.0), f(0.010)],
        satmos: [f(0.30), f(0.005)],
        sirri: f(8.0),
        parent: [F::ZERO, f(45.0), f(30.0)],
        secndy: [F::ZERO, f(15.0), f(10.0)],
        occlud: f(25.0),
        som1ci: [[f(35.0), F::ZERO], [f(110.0), F::ZERO]],
        som2ci: [f(2200.0), F::ZERO],
        som3ci: [f(1400.0), F::ZERO],
        clittr: [[f(90.0), F::ZERO], [f(110.0), F::ZERO]],
        ..Default::default()
    };

    let mut crops = HashMap::new();
    // Cool-season C3 grass
    let c3 = CropParams::<F> {
        name: "C3GR".into(),
        prdx1: f(240.0),
        ppdf: [f(20.0), f(36.0), f(1.0), f(3.5)],
        co2ipr: f(1.30),
        co2itr: f(0.75),
        ..Default::default()
    };
    crops.insert(c3.name.clone(), c3);

    // Warm-season C4 tropical grass
    let c4 = CropParams::<F> {
        name: "C4GR".into(),
        prdx1: f(340.0),
        ppdf: [f(28.0), f(44.0), f(1.2), f(3.0)],
        co2ipr: f(1.12),
        co2itr: f(0.70),
        ..Default::default()
    };
    crops.insert(c4.name.clone(), c4);

    // Annual grain crop (maize/wheat style)
    let grain = CropParams::<F> {
        name: "GRAIN".into(),
        prdx1: f(420.0),
        ppdf: [f(24.0), f(40.0), f(1.0), f(3.2)],
        himax: f(0.45),
        hiwsf: f(0.50),
        co2ipr: f(1.25),
        co2itr: f(0.75),
        ..Default::default()
    };
    crops.insert(grain.name.clone(), grain);

    // Arid shrubland herbaceous cover
    let shrub = CropParams::<F> {
        name: "SHRB".into(),
        prdx1: f(145.0),
        ppdf: [f(22.0), f(42.0), f(1.0), f(3.5)],
        fligni: [[f(0.10), f(0.28)], [f(0.001), f(-0.001)]],
        ..Default::default()
    };
    crops.insert(shrub.name.clone(), shrub);

    let mut trees = HashMap::new();
    // Temperate Deciduous Forest
    let decid = TreeParams::<F> {
        name: "TMDF".into(),
        decid: 1,
        prdx4: f(430.0),
        ppdf: [f(21.0), f(38.0), f(1.0), f(4.0)],
        wooddr: [f(1.0), f(0.03), f(0.006), f(0.0015), f(0.0025)],
        co2ipr: f(1.30),
        co2itr: f(0.75),
        ..Default::default()
    };
    trees.insert(decid.name.clone(), decid);

    // Boreal Coniferous Forest
    let boreal = TreeParams::<F> {
        name: "BORE".into(),
        decid: 0,
        prdx4: f(260.0),
        ppdf: [f(14.0), f(30.0), f(1.0), f(4.5)],
        leafdr: [f(0.015); 12],
        wooddr: [f(0.015), f(0.025), f(0.004), f(0.001), f(0.002)],
        co2ipr: f(1.25),
        ..Default::default()
    };
    trees.insert(boreal.name.clone(), boreal);

    // Tropical Savanna / Cerrado Woody Canopy
    let cerr = TreeParams::<F> {
        name: "CERR".into(),
        decid: 1,
        prdx4: f(360.0),
        ppdf: [f(27.0), f(43.0), f(1.2), f(3.5)],
        wooddr: [f(0.85), f(0.03), f(0.008), f(0.002), f(0.003)],
        sitpot: f(650.0),
        basfct: f(1.2),
        co2ipr: f(1.28),
        ..Default::default()
    };
    trees.insert(cerr.name.clone(), cerr);

    // Tropical Evergreen Rainforest (Amazonian Oxisol / Latossolo)
    let amaz = TreeParams::<F> {
        name: "AMAZ".into(),
        decid: 0,
        prdx4: f(540.0),
        ppdf: [f(27.5), f(45.0), f(1.0), f(3.5)],
        leafdr: [f(0.035); 12],
        wooddr: [f(0.035), f(0.035), f(0.007), f(0.0018), f(0.003)],
        maxlai: f(12.0),
        co2ipr: f(1.30),
        co2itr: f(0.72),
        ..Default::default()
    };
    trees.insert(amaz.name.clone(), amaz);

    // Standard CENTURY-compliant management options
    let mut cult = HashMap::new();
    cult.insert(
        "PLOW".into(),
        CultParams {
            name: "PLOW".into(),
            cultra: [f(0.0), f(0.1), f(0.9), f(0.1), f(0.9), f(0.9), f(1.0)],
            clteff: [f(1.6), f(1.6), f(1.5), f(1.5)],
        },
    );

    let mut harv = HashMap::new();
    harv.insert(
        "GRAIN".into(),
        HarvParams {
            name: "GRAIN".into(),
            aglrem: f(0.0),
            bglrem: f(0.0),
            flghrv: 1,
            rmvstr: f(0.30),
            remwsd: f(0.0),
            hibg: F::ZERO,
        },
    );

    let mut fert = HashMap::new();
    fert.insert(
        "CUSTOM".into(),
        FertParams {
            name: "CUSTOM".into(),
            feramt: [F::ZERO; 3],
            aufert: F::ZERO,
        },
    );

    let mut irri = HashMap::new();
    irri.insert(
        "AUTO75".into(),
        IrriParams {
            name: "AUTO75".into(),
            auirri: 1,
            fawhc: f(0.75),
            irraut: F::ZERO,
            irramt: F::ZERO,
        },
    );

    let mut fire = HashMap::new();
    fire.insert(
        "PRESC".into(),
        FireParams {
            name: "PRESC".into(),
            flfrem: f(0.70),
            fdfrem: [f(0.80), f(0.75)],
            fret: [f(0.25), f(0.80), f(0.60)],
            frtsh: f(0.10),
            fnue: [f(0.0), f(0.0)],
        },
    );

    let mut graz = HashMap::new();
    graz.insert(
        "CUSTOM".into(),
        GrazParams {
            name: "CUSTOM".into(),
            flgrem: f(0.20),
            fdgrem: f(0.10),
            gfcret: f(0.30),
            gret: [f(0.75), f(0.75), f(0.75)],
            grzeff: 1,
            fecf: [f(0.5), f(0.5), f(0.5)],
            feclig: f(0.18),
        },
    );

    let mut omad = HashMap::new();
    omad.insert(
        "COMPOST".into(),
        OmadParams {
            name: "COMPOST".into(),
            astgc: F::ZERO,
            astlbl: F::ZERO,
            astlig: f(0.20),
            astrec: [f(16.0), f(80.0), f(100.0)],
        },
    );

    let mut trem = HashMap::new();
    trem.insert(
        "THIN".into(),
        TremParams {
            name: "THIN".into(),
            evntyp: 0,
            remf: [f(0.4), f(0.3), f(0.3), F::ZERO, F::ZERO],
            fd: [f(0.5), f(0.1)],
            retf: [
                [f(1.0), f(1.0), f(1.0), f(1.0)],
                [f(1.0), f(1.0), f(1.0), f(1.0)],
                [F::ZERO, F::ZERO, F::ZERO, F::ZERO],
            ],
        },
    );

    // Biome-specific fine-tuning of fix/site defaults
    match preset_id {
        "temperate_deciduous" => {
            fix.varat2[0] = [f(25.0), f(14.0), f(2.0)];
            site.rlvcis = [f(120.0), F::ZERO];
            site.fbrcis = [f(260.0), F::ZERO];
            site.rlwcis = [f(3500.0), F::ZERO];
            site.frtcis = [f(180.0), F::ZERO];
            site.crtcis = [f(850.0), F::ZERO];
        }
        "boreal_conifer" => {
            fix.varat2[0] = [f(32.0), f(18.0), f(2.0)];
            site.som2ci = [f(3800.0), F::ZERO];
            site.som3ci = [f(2400.0), F::ZERO];
            site.rlvcis = [f(280.0), F::ZERO];
            site.fbrcis = [f(320.0), F::ZERO];
            site.rlwcis = [f(3200.0), F::ZERO];
            site.frtcis = [f(220.0), F::ZERO];
            site.crtcis = [f(780.0), F::ZERO];
        }
        "cerrado_savanna" => {
            site.rlvcis = [f(65.0), F::ZERO];
            site.fbrcis = [f(110.0), F::ZERO];
            site.rlwcis = [f(900.0), F::ZERO];
            site.frtcis = [f(95.0), F::ZERO];
            site.crtcis = [f(280.0), F::ZERO];
            site.aglcis = [f(110.0), F::ZERO];
            site.bglcis = [f(190.0), F::ZERO];
        }
        "amazon_rainforest" => {
            fix.varat2[0] = [f(22.0), f(12.0), f(2.0)];
            site.som2ci = [f(2900.0), F::ZERO];
            site.som3ci = [f(2100.0), F::ZERO];
            site.rlvcis = [f(360.0), F::ZERO];
            site.fbrcis = [f(650.0), F::ZERO];
            site.rlwcis = [f(9500.0), F::ZERO];
            site.frtcis = [f(320.0), F::ZERO];
            site.crtcis = [f(2100.0), F::ZERO];
        }
        "semiarid_shrubland" => {
            fix.vlossg = f(0.05);
            site.som2ci = [f(1000.0), F::ZERO];
            site.som3ci = [f(700.0), F::ZERO];
        }
        _ => {}
    }

    RawBiomeParams {
        biome_id: 1,
        fix,
        site_defaults: site,
        crops,
        trees,
        cults: cult,
        ferts: fert,
        fires: fire,
        grazs: graz,
        harvs: harv,
        irris: irri,
        omads: omad,
        trems: trem,
    }
}

/// Return all 7 built-in calibrated ecosystem presets for `GET /api/presets`.
pub fn builtin_presets() -> Vec<PresetSummary> {
    vec![
        PresetSummary {
            id: "amazon_rainforest".into(),
            name_en: "Amazon Tropical Rainforest (Clayey Oxisol)".into(),
            name_pt: "Floresta Amazônica Tropical (Latossolo / Oxisol)".into(),
            desc_en: "Humid equatorial evergreen rainforest on deeply weathered acidic Oxisol (3,100 mm/yr, 27 °C).".into(),
            desc_pt: "Floresta equatorial úmida em Latossolo ácido profundo (3.100 mm/ano, 27 °C, 3,1°S).".into(),
            system_kind: "forest".into(),
            defaults: SimulateRequest {
                preset_id: "amazon_rainforest".into(),
                mean_temp_c: 27.0,
                temp_amp_c: 1.5,
                annual_precip_cm: 310.0,
                lat: -3.1,
                co2_start_ppm: 420.0,
                co2_end_ppm: 560.0,
                sand: 0.24,
                silt: 0.16,
                clay: 0.60,
                bulkd: 1.15,
                ph: 4.6,
                rock_frac: 0.02,
                drain: 1.0,
                years: 250,
                elements: "cnps".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::None,
                fertilizer_n: 0.0,
                fertilizer_p: 0.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 0,
                grazing_intensity: 0.0,
                omad_c: 0.0,
            },
        },
        PresetSummary {
            id: "temperate_grassland".into(),
            name_en: "Temperate C3 Grassland (Great Plains)".into(),
            name_pt: "Pradaria Temperada C3 (Grandes Planícies)".into(),
            desc_en: "Classic semi-humid steppe benchmark for CENTURY active/slow/passive SOC equilibrium.".into(),
            desc_pt: "Referência clássica de estepe semiúmida para equilíbrio de carbono ativo/lento/passivo.".into(),
            system_kind: "grassland".into(),
            defaults: SimulateRequest {
                preset_id: "temperate_grassland".into(),
                mean_temp_c: 10.5,
                temp_amp_c: 13.5,
                annual_precip_cm: 58.0,
                lat: 40.8,
                co2_start_ppm: 350.0,
                co2_end_ppm: 350.0,
                sand: 0.42,
                silt: 0.36,
                clay: 0.22,
                bulkd: 1.32,
                ph: 6.8,
                rock_frac: 0.05,
                drain: 1.0,
                years: 200,
                elements: "cnps".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::None,
                fertilizer_n: 0.0,
                fertilizer_p: 0.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 0,
                grazing_intensity: 0.10,
                omad_c: 0.0,
            },
        },
        PresetSummary {
            id: "cerrado_savanna".into(),
            name_en: "Tropical Savanna / Cerrado (Southern Hemisphere)".into(),
            name_pt: "Cerrado / Savana Tropical (Hemisfério Sul)".into(),
            desc_en: "Mixed C4 grass & woody canopy at 15.8°S with austral phenology and seasonal fire response.".into(),
            desc_pt: "Mosaico C4 + estrato lenhoso a 15,8°S com fenologia austral e resposta sazonal ao fogo.".into(),
            system_kind: "savanna".into(),
            defaults: SimulateRequest {
                preset_id: "cerrado_savanna".into(),
                mean_temp_c: 23.0,
                temp_amp_c: 3.5,
                annual_precip_cm: 145.0,
                lat: -15.8,
                co2_start_ppm: 420.0,
                co2_end_ppm: 420.0,
                sand: 0.48,
                silt: 0.17,
                clay: 0.35,
                bulkd: 1.24,
                ph: 5.3,
                rock_frac: 0.04,
                drain: 1.0,
                years: 200,
                elements: "cnps".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::None,
                fertilizer_n: 0.0,
                fertilizer_p: 0.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 5,
                grazing_intensity: 0.0,
                omad_c: 0.0,
            },
        },
        PresetSummary {
            id: "temperate_deciduous".into(),
            name_en: "Temperate Deciduous Forest".into(),
            name_pt: "Floresta Decídua Temperada".into(),
            desc_en: "Broadleaf forest with autumn foliar abscission, large-wood carbon accumulation, and rich SOM.".into(),
            desc_pt: "Floresta latifoliada com queda foliar de outono, acúmulo em madeira e alto estoque de MOS.".into(),
            system_kind: "forest".into(),
            defaults: SimulateRequest {
                preset_id: "temperate_deciduous".into(),
                mean_temp_c: 12.0,
                temp_amp_c: 12.0,
                annual_precip_cm: 110.0,
                lat: 39.5,
                co2_start_ppm: 350.0,
                co2_end_ppm: 500.0,
                sand: 0.35,
                silt: 0.42,
                clay: 0.23,
                bulkd: 1.25,
                ph: 5.9,
                rock_frac: 0.10,
                drain: 1.0,
                years: 250,
                elements: "cnps".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::None,
                fertilizer_n: 0.0,
                fertilizer_p: 0.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 0,
                grazing_intensity: 0.0,
                omad_c: 0.0,
            },
        },
        PresetSummary {
            id: "boreal_conifer".into(),
            name_en: "Boreal Coniferous Forest (Taiga)".into(),
            name_pt: "Floresta Boreal de Coníferas (Taiga)".into(),
            desc_en: "Cold snowpack-buffered conifer forest with slow decomposition and deep recalcitrant carbon.".into(),
            desc_pt: "Floresta fria tamponada por neve, decomposição lenta e alto estoque de carbono recalcitrante.".into(),
            system_kind: "forest".into(),
            defaults: SimulateRequest {
                preset_id: "boreal_conifer".into(),
                mean_temp_c: 1.2,
                temp_amp_c: 16.0,
                annual_precip_cm: 62.0,
                lat: 58.0,
                co2_start_ppm: 350.0,
                co2_end_ppm: 350.0,
                sand: 0.50,
                silt: 0.34,
                clay: 0.16,
                bulkd: 1.18,
                ph: 5.2,
                rock_frac: 0.15,
                drain: 0.8,
                years: 300,
                elements: "cn".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::None,
                fertilizer_n: 0.0,
                fertilizer_p: 0.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 0,
                grazing_intensity: 0.0,
                omad_c: 0.0,
            },
        },
        PresetSummary {
            id: "cropland_corn_wheat".into(),
            name_en: "Intensive Cropland (Plow vs. No-Till)".into(),
            name_pt: "Lavoura Intensiva (Aração vs. Plantio Direto)".into(),
            desc_en: "Annual fertilized grain crop designed for comparing conventional tillage against regenerative no-till.".into(),
            desc_pt: "Cultura anual de grãos ideal para comparar aração convencional contra plantio direto regenerativo.".into(),
            system_kind: "cropland".into(),
            defaults: SimulateRequest {
                preset_id: "cropland_corn_wheat".into(),
                mean_temp_c: 11.5,
                temp_amp_c: 14.0,
                annual_precip_cm: 85.0,
                lat: 42.0,
                co2_start_ppm: 415.0,
                co2_end_ppm: 415.0,
                sand: 0.25,
                silt: 0.48,
                clay: 0.27,
                bulkd: 1.35,
                ph: 6.5,
                rock_frac: 0.02,
                drain: 1.0,
                years: 100,
                elements: "cnps".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::MoldboardPlow,
                fertilizer_n: 12.0,
                fertilizer_p: 2.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 0,
                grazing_intensity: 0.0,
                omad_c: 0.0,
            },
        },
        PresetSummary {
            id: "semiarid_shrubland".into(),
            name_en: "Semi-Arid Shrubland".into(),
            name_pt: "Arbustal Semiárido".into(),
            desc_en: "Moisture-limited dryland ecosystem sensitive to rainfall shifts, irrigation, and grazing pressure.".into(),
            desc_pt: "Ecossistema seco limitado por água, sensível a variações de chuva, irrigação e pastejo.".into(),
            system_kind: "grassland".into(),
            defaults: SimulateRequest {
                preset_id: "semiarid_shrubland".into(),
                mean_temp_c: 16.5,
                temp_amp_c: 11.5,
                annual_precip_cm: 28.0,
                lat: 35.5,
                co2_start_ppm: 380.0,
                co2_end_ppm: 380.0,
                sand: 0.65,
                silt: 0.22,
                clay: 0.13,
                bulkd: 1.48,
                ph: 7.6,
                rock_frac: 0.20,
                drain: 1.0,
                years: 200,
                elements: "cn".into(),
                profile: "clean".into(),
                precision: "f64".into(),
                audit_mass_balance: true,
                tillage: TillageMode::None,
                fertilizer_n: 0.0,
                fertilizer_p: 0.0,
                irrigation: IrrigationMode::None,
                fire_interval_yr: 0,
                grazing_intensity: 0.15,
                omad_c: 0.0,
            },
        },
    ]
}

/// Build a repeating annual/multi-year [`Schedule`] from a [`SimulateRequest`].
pub fn build_schedule_from_request(req: &SimulateRequest, years: i32) -> Schedule {
    let sh = req.lat < 0.0;
    let spring = if sh { 10u8 } else { 4u8 };
    let mid_summer = if sh { 1u8 } else { 7u8 };
    let late_summer = if sh { 2u8 } else { 8u8 };
    let autumn = if sh { 4u8 } else { 10u8 };
    let pre_spring = if sh { 9u8 } else { 3u8 };

    let (initial_system, initial_crop, initial_tree) = match req.preset_id.as_str() {
        "amazon_rainforest" => (SystemKind::Forest, String::new(), "AMAZ".to_string()),
        "temperate_deciduous" => (SystemKind::Forest, String::new(), "TMDF".to_string()),
        "boreal_conifer" => (SystemKind::Forest, String::new(), "BORE".to_string()),
        "cerrado_savanna" => (SystemKind::Savanna, "C4GR".to_string(), "CERR".to_string()),
        "cropland_corn_wheat" => (
            SystemKind::GrasslandCrop,
            "GRAIN".to_string(),
            String::new(),
        ),
        "semiarid_shrubland" => (SystemKind::GrasslandCrop, "SHRB".to_string(), String::new()),
        _ => (SystemKind::GrasslandCrop, "C3GR".to_string(), String::new()),
    };

    let repeat_years = if req.fire_interval_yr > 0 {
        req.fire_interval_yr.clamp(1, 50)
    } else {
        1
    };

    let mut events = Vec::new();
    for ry in 1..=repeat_years {
        // Annual management & phenology
        if req.tillage == TillageMode::MoldboardPlow {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: pre_spring,
                kind: EventKind::Cult("PLOW".into()),
            });
        }
        if req.fertilizer_n > 0.0 || req.fertilizer_p > 0.0 {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: spring,
                kind: EventKind::Fert("CUSTOM".into()),
            });
        }
        if req.omad_c > 0.0 {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: spring,
                kind: EventKind::Omad("COMPOST".into()),
            });
        }

        match initial_system {
            SystemKind::GrasslandCrop => {
                if req.tillage != TillageMode::None {
                    events.push(ScheduledEvent {
                        rel_year: ry,
                        month: spring,
                        kind: EventKind::Pltm,
                    });
                } else {
                    events.push(ScheduledEvent {
                        rel_year: ry,
                        month: spring,
                        kind: EventKind::Frst,
                    });
                }
            }
            SystemKind::Forest => {
                events.push(ScheduledEvent {
                    rel_year: ry,
                    month: spring,
                    kind: EventKind::Tfst,
                });
            }
            SystemKind::Savanna => {
                events.push(ScheduledEvent {
                    rel_year: ry,
                    month: spring,
                    kind: EventKind::Frst,
                });
                events.push(ScheduledEvent {
                    rel_year: ry,
                    month: spring,
                    kind: EventKind::Tfst,
                });
            }
        }

        if req.irrigation == IrrigationMode::Auto75 {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: mid_summer,
                kind: EventKind::Irri("AUTO75".into()),
            });
            events.push(ScheduledEvent {
                rel_year: ry,
                month: late_summer,
                kind: EventKind::Irri("AUTO75".into()),
            });
        }

        if req.grazing_intensity > 0.001 && initial_system != SystemKind::Forest {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: mid_summer,
                kind: EventKind::Graz("CUSTOM".into()),
            });
        }

        if req.tillage != TillageMode::None && initial_system != SystemKind::Forest {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: late_summer,
                kind: EventKind::Harv("GRAIN".into()),
            });
        }

        if req.fire_interval_yr > 0 && ry == repeat_years {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: autumn,
                kind: EventKind::Fire("PRESC".into()),
            });
        }

        if initial_system != SystemKind::Forest {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: autumn,
                kind: EventKind::Senm,
            });
            events.push(ScheduledEvent {
                rel_year: ry,
                month: autumn,
                kind: EventKind::Last,
            });
        }
        if initial_system != SystemKind::GrasslandCrop {
            events.push(ScheduledEvent {
                rel_year: ry,
                month: autumn,
                kind: EventKind::Tlst,
            });
        }
    }

    Schedule {
        start_year: 1,
        last_year: years,
        site_file: format!("{}.100", req.preset_id),
        isotope: IsotopeMode::Unlabeled,
        label_year: -1,
        initial_system,
        initial_crop,
        initial_tree,
        blocks: vec![ScheduleBlock {
            block_num: 1,
            label: "Interactive Studio Block".into(),
            end_year: years,
            repeat_years,
            weather_mode: 'M',
            events,
        }],
    }
}

/// Generate a realistic multi-biome synthetic continental grid (`rows x cols`)
/// spanning latitudinal temperature gradients and longitudinal precipitation
/// gradients, used by the Spatial Grid Explorer when no external VEMAP folder
/// is attached.
pub fn build_synthetic_grid<F: Float>(rows: usize, cols: usize, years: i32) -> PreloadedGrid<F> {
    let f = F::from_f64;
    let preset_ids = [
        (1, "boreal_conifer"),
        (2, "temperate_deciduous"),
        (3, "temperate_grassland"),
        (4, "semiarid_shrubland"),
        (5, "cerrado_savanna"),
    ];

    let mut biomes = HashMap::new();
    let mut schedules = HashMap::new();
    for (bid, pid) in preset_ids {
        let mut b = base_biome_params::<F>(pid);
        b.biome_id = bid;
        biomes.insert(bid, Arc::new(b));
        let mut dummy_req = builtin_presets()
            .into_iter()
            .find(|p| p.id == pid)
            .map(|p| p.defaults)
            .unwrap_or_else(|| builtin_presets().remove(0).defaults);
        dummy_req.fire_interval_yr = 0;
        schedules.insert(
            bid,
            Arc::new(build_schedule_from_request(&dummy_req, years)),
        );
    }

    let mut cells = Vec::with_capacity(rows * cols);
    let mut mask = vec![0i32; rows * cols];
    let mut idx = 0usize;
    for r in 0..rows {
        let lat_frac = r as f64 / (rows.saturating_sub(1).max(1) as f64); // 0=north, 1=south
        let lat = 58.0 - lat_frac * 36.0; // 58°N -> 22°N
        let mean_t = 0.5 + lat_frac * 22.0;
        let amp_t = 16.0 - lat_frac * 10.0;
        for c in 0..cols {
            let lon_frac = c as f64 / (cols.saturating_sub(1).max(1) as f64);
            // Smooth synthetic continental coastline mask for realistic geography
            let dx = (lon_frac - 0.52) / 0.48;
            let dy = (lat_frac - 0.48) / 0.48;
            let ripple = 0.12 * (lon_frac * 9.0).sin() * (lat_frac * 7.0).cos();
            if dx * dx + dy * dy + ripple > 0.92 {
                continue;
            }
            let lon = -124.0 + lon_frac * 56.0;
            // West-to-east rainshadow + coastal humidity
            let precip_cm = (26.0 + 115.0 * (0.25 + 0.75 * lon_frac)
                - 32.0 * (-(lon_frac - 0.28).powi(2) / 0.03).exp())
            .clamp(22.0, 155.0);

            let biome_id = if mean_t < 5.0 {
                1 // Boreal conifer
            } else if precip_cm < 42.0 {
                4 // Semi-arid shrubland
            } else if mean_t > 18.0 && precip_cm > 75.0 {
                5 // Savanna / warm woodland
            } else if precip_cm > 82.0 {
                2 // Temperate deciduous
            } else {
                3 // Temperate grassland
            };

            let sand = (0.22 + 0.45 * (1.0 - lon_frac) + 0.08 * ((r * 3 + c) as f64).sin())
                .clamp(0.15, 0.75);
            let clay = (0.14 + 0.24 * lat_frac).clamp(0.10, 0.45);
            let silt = (1.0 - sand - clay).max(0.05);
            let rock_frac = (0.04 + 0.18 * (-(lon_frac - 0.22).powi(2) / 0.015).exp()).min(0.45);

            cells.push(PreloadedCell {
                cell_index: idx,
                transient_cell_id: None,
                row: r,
                col: c,
                biome_id,
                site_input: CellSiteInput {
                    lat: f(lat),
                    lon: f(lon),
                    elevation: f(200.0 + 1400.0 * (-(lon_frac - 0.22).powi(2) / 0.015).exp()),
                    sand: f(sand),
                    silt: f(silt),
                    clay: f(clay),
                    bulkd: f(1.22 + 0.22 * sand),
                    rock_frac: f(rock_frac),
                    zminrl_cm: None,
                    ph: f(5.4 + 2.0 * (1.0 - precip_cm / 160.0)),
                    drain: f(1.0),
                },
                mean_weather: build_synthetic_climate(mean_t, amp_t, precip_cm, lat),
                transient_weather: None,
            });
            mask[r * cols + c] = (idx + 1) as i32;
            idx += 1;
        }
    }

    PreloadedGrid {
        nrows: rows,
        ncols: cols,
        mask,
        biomes,
        schedules,
        cells,
    }
}
