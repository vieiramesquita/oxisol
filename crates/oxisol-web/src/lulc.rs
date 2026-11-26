//! Spatial Land-Use / Land-Cover (LULC) & Modeled Climate Raster Engine.
//!
//! Powered by [`oxisol_io::raster_geo::GeoRaster`] (`terrio` pure-Rust GeoTIFF / COG / NetCDF)
//! and `rayon` parallel pixel execution. Supports both:
//! 1. **Built-in Amazon–Cerrado Frontier LUC Demo (`1990–2024`)**: instant georeferenced
//!    EPSG:4326 landscape with progressive forest-to-pasture-to-cropland conversion,
//!    climate warming / rainfall perturbation sliders, and real COG export.
//! 2. **Real GeoTIFF / COG / NetCDF Rasters (`terrio`)**: reads static or multi-year LULC maps
//!    (MapBiomas, ESA WorldCover, or custom class tables), multi-band monthly climate rasters
//!    (`precip`, `tmin`, `tmax`), and optional soil rasters (SoilGrids / HWSD), simulating
//!    per-pixel spinup + dynamic land-use transitions and exporting compressed COGs.

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use oxisol_core::step::ActiveSelection;
use oxisol_core::{
    initialize_cell, step_month, CellSiteInput, CompatibilityProfile, ElementMode, IsotopeMode,
    MonthlyWeather, RawBiomeParams, SimConfig, SystemKind,
};
use oxisol_io::raster_geo::GeoRaster;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{IrrigationMode, SimulateRequest, TillageMode};
use crate::presets::{base_biome_params, build_schedule_from_request, build_synthetic_climate};

/// Single LULC class code mapping entry (`code -> preset + management + legend color`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LulcClassMapping {
    /// Integer LULC raster class code (e.g. MapBiomas `3` = Forest, `15` = Pasture).
    pub code: i32,
    /// English class label.
    pub label_en: String,
    /// Portuguese (PT-BR) class label.
    pub label_pt: String,
    /// Hex color for categorical map rendering (e.g. `"#1f8d49"`).
    pub color: String,
    /// Oxisol ecosystem preset ID (`"amazon_rainforest"`, `"cerrado_savanna"`,
    /// `"temperate_grassland"`, `"cropland_corn_wheat"`, `"temperate_deciduous"`,
    /// `"boreal_conifer"`, `"semiarid_shrubland"`, or `"masked"` to skip).
    pub preset_id: String,
    /// Default grazing intensity for this class (`0.0..=0.5`).
    #[serde(default)]
    pub grazing: f64,
    /// Default annual N fertilizer for this class ($\text{g N}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    #[serde(default)]
    pub fert_n: f64,
    /// Whether this class represents managed cropland (uses request `cropland_tillage`).
    #[serde(default)]
    pub is_cropland: bool,
}

/// Predefined LULC classification scheme returned by `GET /api/lulc/schemes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LulcScheme {
    /// Scheme identifier (`"mapbiomas_br"` or `"esa_worldcover"`).
    pub id: String,
    /// English title.
    pub name_en: String,
    /// Portuguese title.
    pub name_pt: String,
    /// Class code mappings.
    pub classes: Vec<LulcClassMapping>,
}

/// Climate precipitation input unit for real raster mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrecipUnit {
    /// Millimeters per month (WorldClim, ERA5-Land, CHIRPS, TerraClimate → ÷ 10).
    #[default]
    Mm,
    /// Centimeters per month (native CENTURY units).
    Cm,
    /// Meters per month (raw ERA5 total precipitation → × 100).
    Meters,
}

/// Climate temperature input unit for real raster mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TempUnit {
    /// Degrees Celsius.
    #[default]
    Celsius,
    /// Kelvin (raw ERA5 / CMIP6 → − 273.15).
    Kelvin,
    /// Tenths of degrees Celsius (WorldClim v1 / TerraClimate scaled → ÷ 10).
    CelsiusX10,
}

/// Request payload for `POST /api/lulc/run`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LulcRunRequest {
    /// `"demo"` (built-in Amazon–Cerrado LUC 1990–2024 landscape) or `"geotiff"` (real `terrio` rasters).
    #[serde(default = "default_demo_mode")]
    pub mode: String,
    /// Classification scheme ID (`"mapbiomas_br"` or `"esa_worldcover"`).
    #[serde(default = "default_mapbiomas")]
    pub scheme_id: String,
    /// Optional custom class mappings overriding the scheme defaults.
    #[serde(default)]
    pub class_mappings: Vec<LulcClassMapping>,
    /// One or more LULC GeoTIFF/COG file paths (one per year for dynamic LUC transitions).
    #[serde(default)]
    pub lulc_paths: Vec<String>,
    /// Multi-band monthly precipitation GeoTIFF/NetCDF path (`12` or `12*N` bands).
    #[serde(default)]
    pub precip_path: Option<String>,
    /// Multi-band monthly Tmin GeoTIFF/NetCDF path (`12` or `12*N` bands).
    #[serde(default)]
    pub tmin_path: Option<String>,
    /// Multi-band monthly Tmax GeoTIFF/NetCDF path (`12` or `12*N` bands).
    #[serde(default)]
    pub tmax_path: Option<String>,
    /// Precipitation unit conversion.
    #[serde(default)]
    pub precip_unit: PrecipUnit,
    /// Temperature unit conversion.
    #[serde(default)]
    pub temp_unit: TempUnit,
    /// Optional soil sand GeoTIFF path.
    #[serde(default)]
    pub sand_path: Option<String>,
    /// Optional soil clay GeoTIFF path.
    #[serde(default)]
    pub clay_path: Option<String>,
    /// Optional soil bulk density GeoTIFF path.
    #[serde(default)]
    pub bulkd_path: Option<String>,
    /// Optional soil pH GeoTIFF path.
    #[serde(default)]
    pub ph_path: Option<String>,
    /// Optional coarse rock fragment (`cfvo`) GeoTIFF path.
    #[serde(default)]
    pub rock_path: Option<String>,
    /// Auto-scale SoilGrids integer conventions (`sand/clay` g/kg → frac, `bulkd` cg/cm³ → g/cm³, `pH*10` → pH).
    #[serde(default = "default_true")]
    pub soilgrids_units: bool,
    /// Equilibrium spinup years on each pixel's initial LULC class (`10..=500`).
    #[serde(default = "default_spinup")]
    pub spinup_years: i32,
    /// Transient simulation years (for demo or static LULC; multi-year LULC uses path count).
    #[serde(default = "default_transient")]
    pub transient_years: i32,
    /// Starting atmospheric CO₂ (ppm).
    #[serde(default = "default_co2_start")]
    pub co2_start_ppm: f64,
    /// Ending atmospheric CO₂ (ppm).
    #[serde(default = "default_co2_end")]
    pub co2_end_ppm: f64,
    /// Climate warming scenario offset applied over transient years (°C, e.g. `+2.0`).
    #[serde(default)]
    pub temp_delta_c: f64,
    /// Climate precipitation multiplier applied over transient years (e.g. `0.85` = −15% rainfall).
    #[serde(default = "default_one")]
    pub precip_scale: f64,
    /// Tillage regime applied to cropland classes (`no_till_harvest` vs `moldboard_plow`).
    #[serde(default = "default_plow")]
    pub cropland_tillage: TillageMode,
    /// Optional worker thread count (`None` = all logical cores).
    pub threads: Option<usize>,
    /// Optional output directory to write compressed Cloud-Optimized GeoTIFFs (`.tif`) via `terrio`.
    #[serde(default)]
    pub export_cog_dir: Option<String>,
}

fn default_demo_mode() -> String {
    "demo".into()
}
fn default_mapbiomas() -> String {
    "mapbiomas_br".into()
}
fn default_true() -> bool {
    true
}
fn default_spinup() -> i32 {
    80
}
fn default_transient() -> i32 {
    35
}
fn default_co2_start() -> f64 {
    354.0
}
fn default_co2_end() -> f64 {
    422.0
}
fn default_one() -> f64 {
    1.0
}
fn default_plow() -> TillageMode {
    TillageMode::MoldboardPlow
}

/// Per-pixel simulation output for the interactive 2D LULC & Carbon map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LulcPixelResult {
    /// Grid row index.
    pub row: usize,
    /// Grid column index.
    pub col: usize,
    /// Pixel center latitude (degrees).
    pub lat: f64,
    /// Pixel center longitude (degrees).
    pub lon: f64,
    /// Initial LULC class code.
    pub lulc_initial: i32,
    /// Final LULC class code.
    pub lulc_final: i32,
    /// Whether this pixel experienced at least one land-use transition.
    pub converted: bool,
    /// Post-spinup equilibrium SOMSC ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_initial: f64,
    /// Final SOMSC after transient period ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_final: f64,
    /// Net soil carbon change `somsc_final - somsc_initial` ($\text{g C}\cdot\text{m}^{-2}$).
    pub delta_somsc: f64,
    /// Final passive SOM3 ($\text{g C}\cdot\text{m}^{-2}$).
    pub som3c: f64,
    /// Final total live vegetation biomass carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub biomass_c: f64,
    /// Final-year annual NPP ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub npp: f64,
    /// Final-year annual AET ($\text{cm}\cdot\text{yr}^{-1}$).
    pub aet: f64,
    /// Final surface mineral nitrogen ($\text{g N}\cdot\text{m}^{-2}$).
    pub minerl_n: f64,
}

/// Complete response returned by `POST /api/lulc/run`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LulcRunResponse {
    /// Source label (`"Amazon–Cerrado Frontier LUC Demo (EPSG:4326)"` or GeoTIFF path summary).
    pub source_label: String,
    /// Grid height.
    pub rows: usize,
    /// Grid width.
    pub cols: usize,
    /// Number of active simulated pixels.
    pub n_active: usize,
    /// Number of pixels that underwent land-use conversion.
    pub n_converted: usize,
    /// Total simulated years per pixel (`spinup + transient`).
    pub total_years_per_pixel: usize,
    /// Number of `rayon` worker threads used.
    pub threads_used: usize,
    /// Wall-clock time in seconds.
    pub elapsed_secs: f64,
    /// Throughput in pixel-years per second.
    pub pixel_years_per_sec: f64,
    /// Mean final `SOMSC` across active pixels ($\text{g C}\cdot\text{m}^{-2}$).
    pub mean_somsc: f64,
    /// Mean net `ΔSOMSC` across active pixels ($\text{g C}\cdot\text{m}^{-2}$).
    pub mean_delta_somsc: f64,
    /// Mean net `ΔSOMSC` converted to $\text{t C}\cdot\text{ha}^{-1}$ (`g/m² ÷ 100`).
    pub mean_delta_t_c_ha: f64,
    /// Active class legend used for rendering.
    pub legend: Vec<LulcClassMapping>,
    /// Paths of Cloud-Optimized GeoTIFFs written via `terrio` (if `export_cog_dir` was provided).
    pub exported_cogs: Vec<String>,
    /// Per-pixel spatial results.
    pub pixels: Vec<LulcPixelResult>,
}

/// Helper constructor for [`LulcClassMapping`] entries.
#[allow(clippy::too_many_arguments)]
fn map_class(
    code: i32,
    label_en: &str,
    label_pt: &str,
    color: &str,
    preset_id: &str,
    grazing: f64,
    fert_n: f64,
    is_cropland: bool,
) -> LulcClassMapping {
    LulcClassMapping {
        code,
        label_en: label_en.into(),
        label_pt: label_pt.into(),
        color: color.into(),
        preset_id: preset_id.into(),
        grazing,
        fert_n,
        is_cropland,
    }
}

/// Built-in LULC classification templates (`MapBiomas Brasil Collection 11` and `ESA WorldCover`).
///
/// MapBiomas Brazil Collection 11 codes, bilingual class names, and official HEX palette follow
/// the official legend table published by MapBiomas Brasil (`13/08/2026`, 41 classes).
pub fn builtin_lulc_schemes() -> Vec<LulcScheme> {
    vec![
        LulcScheme {
            id: "mapbiomas_br".into(),
            name_en: "MapBiomas Brasil (Collection 11)".into(),
            name_pt: "MapBiomas Brasil (Coleção 11)".into(),
            classes: vec![
                // 1. Forest / Floresta
                map_class(
                    1,
                    "Forest",
                    "Floresta",
                    "#1f8d49",
                    "amazon_rainforest",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    3,
                    "Forest Formation",
                    "Formação Florestal",
                    "#1f8d49",
                    "amazon_rainforest",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    6,
                    "Floodable Forest",
                    "Floresta Alagável",
                    "#007785",
                    "amazon_rainforest",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    4,
                    "Savanna Formation",
                    "Formação Savânica",
                    "#7dc975",
                    "cerrado_savanna",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    7,
                    "Flooded Savanna (beta)",
                    "Savana Alagada (beta)",
                    "#228c70",
                    "cerrado_savanna",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    5,
                    "Mangrove",
                    "Mangue",
                    "#04381d",
                    "amazon_rainforest",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    49,
                    "Wooded Sandbank Vegetation",
                    "Restinga Arbórea",
                    "#02d659",
                    "cerrado_savanna",
                    0.0,
                    0.0,
                    false,
                ),
                // 2. Herbaceous or Shrubby Vegetation / Vegetação Herbácea e Arbustiva
                map_class(
                    10,
                    "Herbaceous or Shrubby Vegetation",
                    "Vegetação Herbácea e Arbustiva",
                    "#d6bc74",
                    "temperate_grassland",
                    0.05,
                    0.0,
                    false,
                ),
                map_class(
                    12,
                    "Grassland Formation",
                    "Formação Campestre",
                    "#d6bc74",
                    "temperate_grassland",
                    0.08,
                    0.0,
                    false,
                ),
                map_class(
                    77,
                    "Herbaceous and Shrub Formation",
                    "Formação Herbáceo Arbustiva",
                    "#86b074",
                    "semiarid_shrubland",
                    0.05,
                    0.0,
                    false,
                ),
                map_class(
                    11,
                    "Wetland",
                    "Campo Alagado e Área Pantanosa",
                    "#519799",
                    "temperate_grassland",
                    0.05,
                    0.0,
                    false,
                ),
                map_class(
                    84,
                    "Salt Marsh (beta)",
                    "Marisma (beta)",
                    "#81dbbf",
                    "temperate_grassland",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    50,
                    "Herbaceous Sandbank Vegetation",
                    "Restinga Herbácea ou Arbustiva",
                    "#ffaa5f",
                    "semiarid_shrubland",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    32,
                    "Hypersaline Tidal Flat (Masked)",
                    "Apicum (Máscara)",
                    "#fc8114",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    29,
                    "Rocky Outcrop (Masked)",
                    "Afloramento Rochoso (Máscara)",
                    "#ad5100",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                // 3. Farming / Agropecuária
                map_class(
                    14,
                    "Farming",
                    "Agropecuária",
                    "#ffefc3",
                    "temperate_grassland",
                    0.15,
                    4.0,
                    false,
                ),
                map_class(
                    15,
                    "Pasture",
                    "Pastagem",
                    "#edde8e",
                    "tropical_pasture_urochloa",
                    0.25,
                    0.0,
                    false,
                ),
                map_class(
                    18,
                    "Agriculture",
                    "Agricultura",
                    "#E974ED",
                    "cropland_corn_wheat",
                    0.0,
                    10.0,
                    true,
                ),
                map_class(
                    19,
                    "Temporary Crop",
                    "Lavoura Temporária",
                    "#C27BA0",
                    "cropland_corn_wheat",
                    0.0,
                    10.0,
                    true,
                ),
                map_class(
                    39,
                    "Soybean",
                    "Soja",
                    "#f5b3c8",
                    "cropland_corn_wheat",
                    0.0,
                    12.0,
                    true,
                ),
                map_class(
                    20,
                    "Sugar cane",
                    "Cana",
                    "#db7093",
                    "cropland_corn_wheat",
                    0.0,
                    10.0,
                    true,
                ),
                map_class(
                    40,
                    "Rice",
                    "Arroz",
                    "#c71585",
                    "cropland_corn_wheat",
                    0.0,
                    10.0,
                    true,
                ),
                map_class(
                    62,
                    "Cotton (beta)",
                    "Algodão (beta)",
                    "#ff69b4",
                    "cropland_corn_wheat",
                    0.0,
                    12.0,
                    true,
                ),
                map_class(
                    41,
                    "Other Temporary Crops",
                    "Outras Lavouras Temporárias",
                    "#f54ca9",
                    "cropland_corn_wheat",
                    0.0,
                    10.0,
                    true,
                ),
                map_class(
                    36,
                    "Perennial Crop",
                    "Lavoura Perene",
                    "#d082de",
                    "cerrado_savanna",
                    0.0,
                    8.0,
                    false,
                ),
                map_class(
                    46,
                    "Coffee",
                    "Café",
                    "#d68fe2",
                    "cerrado_savanna",
                    0.0,
                    12.0,
                    false,
                ),
                map_class(
                    47,
                    "Citrus",
                    "Citrus",
                    "#9932cc",
                    "cerrado_savanna",
                    0.0,
                    10.0,
                    false,
                ),
                map_class(
                    35,
                    "Palm Oil",
                    "Dendê",
                    "#9065d0",
                    "amazon_rainforest",
                    0.0,
                    8.0,
                    false,
                ),
                map_class(
                    48,
                    "Other Perennial Crops",
                    "Outras Lavouras Perenes",
                    "#e6ccff",
                    "cerrado_savanna",
                    0.0,
                    8.0,
                    false,
                ),
                map_class(
                    9,
                    "Forest Plantation",
                    "Silvicultura",
                    "#7a5900",
                    "temperate_deciduous",
                    0.0,
                    2.0,
                    false,
                ),
                map_class(
                    21,
                    "Mosaic of Uses",
                    "Mosaico de Usos",
                    "#ffefc3",
                    "temperate_grassland",
                    0.15,
                    4.0,
                    false,
                ),
                // 4. Non vegetated area / Área não vegetada
                map_class(
                    22,
                    "Non vegetated area (Masked)",
                    "Área não vegetada (Máscara)",
                    "#d4271e",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    23,
                    "Beach, Dune and Sand Spot (Masked)",
                    "Praia, Duna e Areal (Máscara)",
                    "#ffa07a",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    24,
                    "Urban Area (Masked)",
                    "Área Urbanizada (Máscara)",
                    "#d4271e",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    30,
                    "Mining (Masked)",
                    "Mineração (Máscara)",
                    "#9c0027",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    75,
                    "Photovoltaic Power Plant (Masked)",
                    "Usina Fotovoltaica (Máscara)",
                    "#757272",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    91,
                    "Wind farm (beta, Masked)",
                    "Parque eólico (beta, Máscara)",
                    "#403d3e",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    25,
                    "Other non Vegetated Areas (Masked)",
                    "Outras Áreas não Vegetadas (Máscara)",
                    "#db4d4f",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                // 5. Water / Corpo D'água
                map_class(
                    26,
                    "Water (Masked)",
                    "Corpo D'água (Máscara)",
                    "#2532e4",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    33,
                    "River, Lake and Ocean (Masked)",
                    "Rio, Lago e Oceano (Máscara)",
                    "#2532e4",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
                map_class(
                    31,
                    "Aquaculture (Masked)",
                    "Aquicultura (Máscara)",
                    "#091077",
                    "masked",
                    0.0,
                    0.0,
                    false,
                ),
            ],
        },
        LulcScheme {
            id: "esa_worldcover".into(),
            name_en: "ESA WorldCover (10m)".into(),
            name_pt: "ESA WorldCover (10m)".into(),
            classes: vec![
                LulcClassMapping {
                    code: 10,
                    label_en: "Tree Cover".into(),
                    label_pt: "Cobertura Arbórea".into(),
                    color: "#006400".into(),
                    preset_id: "temperate_deciduous".into(),
                    grazing: 0.0,
                    fert_n: 0.0,
                    is_cropland: false,
                },
                LulcClassMapping {
                    code: 20,
                    label_en: "Shrubland".into(),
                    label_pt: "Arbustal".into(),
                    color: "#ffbb22".into(),
                    preset_id: "semiarid_shrubland".into(),
                    grazing: 0.05,
                    fert_n: 0.0,
                    is_cropland: false,
                },
                LulcClassMapping {
                    code: 30,
                    label_en: "Grassland".into(),
                    label_pt: "Pastagem / Campo".into(),
                    color: "#ffff4c".into(),
                    preset_id: "temperate_grassland".into(),
                    grazing: 0.15,
                    fert_n: 0.0,
                    is_cropland: false,
                },
                LulcClassMapping {
                    code: 40,
                    label_en: "Cropland".into(),
                    label_pt: "Agricultura".into(),
                    color: "#f096ff".into(),
                    preset_id: "cropland_corn_wheat".into(),
                    grazing: 0.0,
                    fert_n: 12.0,
                    is_cropland: true,
                },
                LulcClassMapping {
                    code: 80,
                    label_en: "Water (Masked)".into(),
                    label_pt: "Água (Máscara)".into(),
                    color: "#0064c8".into(),
                    preset_id: "masked".into(),
                    grazing: 0.0,
                    fert_n: 0.0,
                    is_cropland: false,
                },
            ],
        },
    ]
}

struct PixelTask {
    row: usize,
    col: usize,
    lat: f64,
    lon: f64,
    lulc_series: Vec<i32>,
    weather_series: Vec<[MonthlyWeather<f64>; 12]>,
    sand: f64,
    silt: f64,
    clay: f64,
    bulkd: f64,
    ph: f64,
    rock_frac: f64,
}

fn convert_precip(val: f64, unit: PrecipUnit) -> f64 {
    match unit {
        PrecipUnit::Mm => (val / 10.0).max(0.0),
        PrecipUnit::Cm => val.max(0.0),
        PrecipUnit::Meters => (val * 100.0).max(0.0),
    }
}

fn convert_temp(val: f64, unit: TempUnit) -> f64 {
    match unit {
        TempUnit::Celsius => val,
        TempUnit::Kelvin => val - 273.15,
        TempUnit::CelsiusX10 => val / 10.0,
    }
}

fn build_demo_frontier(
    transient_years: usize,
    temp_delta_c: f64,
    precip_scale: f64,
) -> (GeoRaster, Vec<PixelTask>) {
    let rows = 36usize;
    let cols = 56usize;
    let x_min = -58.0;
    let y_min = -12.0;
    let cell = 0.14;
    let nodata = -9999.0;

    // Template EPSG:4326 GeoRaster for optional real COG export via `terrio`
    let template = GeoRaster::from_epsg4326(
        rows,
        cols,
        x_min,
        y_min,
        cell,
        cell,
        nodata,
        vec![0.0; rows * cols],
    )
    .expect("valid demo EPSG:4326 raster");

    let mut tasks = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        let fy = r as f64 / (rows - 1) as f64; // 0 = north (Amazon -6°S), 1 = south (Cerrado -12°S)
        for c in 0..cols {
            let fx = c as f64 / (cols - 1) as f64;
            let (lat, lon) = template.pixel_lat_lon(r, c, -9.0);

            // Sinuous river corridor across the landscape (MapBiomas code 33 = Water/Masked)
            let river_center = 0.28 + 0.12 * (fy * 5.2).sin() + 0.22 * fy;
            if (fx - river_center).abs() < 0.028 {
                continue;
            }

            // Natural baseline ecotone: Amazon Forest (3) north-west -> Cerrado Savanna (4) south-east
            let ecotone = fy * 0.72 + fx * 0.48 + 0.08 * (fx * 9.0).sin() * (fy * 7.0).cos();
            let init_code = if ecotone < 0.56 { 3 } else { 4 };

            // Frontier road cutting diagonally from SE -> NW driving deforestation (15) and soy (39)
            let dist_road = ((fx - (0.85 - 0.55 * fy)).abs() + 0.35 * (1.0 - fy)).max(0.0);
            let pasture_yr = if dist_road < 0.36 {
                Some(((dist_road / 0.36) * (transient_years as f64 * 0.65)) as usize)
            } else {
                None
            };
            let crop_yr = if dist_road < 0.22 && fy > 0.32 {
                Some(
                    (((dist_road / 0.22) * (transient_years as f64 * 0.45)) as usize
                        + transient_years / 3)
                        .min(transient_years.saturating_sub(1)),
                )
            } else {
                None
            };

            let mut lulc_series = Vec::with_capacity(transient_years);
            for y in 0..transient_years {
                let code = if crop_yr.is_some_and(|cy| y >= cy) {
                    39 // Soybean / Annual Crop
                } else if pasture_yr.is_some_and(|py| y >= py) {
                    15 // Pasture
                } else {
                    init_code
                };
                lulc_series.push(code);
            }

            // Realistic climate gradient: 27°C & 285 cm (NW Amazon) -> 23.5°C & 145 cm (SE Cerrado)
            let base_t = 27.2 - 3.6 * fy - 0.6 * fx;
            let amp_t = 1.4 + 2.4 * fy;
            let base_p = (285.0 - 135.0 * fy - 20.0 * fx).max(115.0);

            let mut weather_series = Vec::with_capacity(transient_years);
            for y in 0..transient_years {
                let frac = if transient_years > 1 {
                    y as f64 / (transient_years - 1) as f64
                } else {
                    0.0
                };
                let t_yr = base_t + temp_delta_c * frac;
                let p_yr = (base_p * (1.0 + (precip_scale - 1.0) * frac)).clamp(20.0, 600.0);
                weather_series.push(build_synthetic_climate::<f64>(t_yr, amp_t, p_yr, lat));
            }

            // Clayey Amazonian Oxisol (NW) -> Medium-textured Cerrado Oxisol (SE)
            let clay = (0.58 - 0.24 * fy + 0.05 * (fx * 6.0).cos()).clamp(0.18, 0.68);
            let sand = (0.24 + 0.28 * fy).clamp(0.18, 0.62);
            let silt = (1.0 - clay - sand).max(0.08);
            let ph = (4.6 + 0.9 * fy).clamp(4.2, 6.2);
            let bulkd = (1.14 + 0.14 * fy).clamp(1.05, 1.42);

            tasks.push(PixelTask {
                row: r,
                col: c,
                lat,
                lon,
                lulc_series,
                weather_series,
                sand,
                silt,
                clay,
                bulkd,
                ph,
                rock_frac: 0.03,
            });
        }
    }

    (template, tasks)
}

fn load_real_geotiff_tasks(
    req: &LulcRunRequest,
    class_map: &HashMap<i32, LulcClassMapping>,
) -> Result<(GeoRaster, Vec<PixelTask>, String), String> {
    if req.lulc_paths.is_empty() {
        return Err("Provide at least one LULC GeoTIFF/COG path in `lulc_paths`.".into());
    }
    let mut lulc_rasters = Vec::with_capacity(req.lulc_paths.len());
    for p in &req.lulc_paths {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            continue;
        }
        let gr = GeoRaster::open(trimmed).map_err(|e| e.to_string())?;
        lulc_rasters.push(gr);
    }
    if lulc_rasters.is_empty() {
        return Err("No valid LULC paths provided.".into());
    }

    let template = lulc_rasters[0].clone();
    let rows = template.rows();
    let cols = template.cols();
    if rows * cols > 1_000_000 {
        return Err(format!(
            "Raster {rows}x{cols} exceeds 1,000,000 interactive Studio pixel limit; use `oxisol raster` CLI for ultra-large rasters."
        ));
    }

    let precip_r = match req.precip_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };
    let tmin_r = match req.tmin_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };
    let tmax_r = match req.tmax_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };

    let sand_r = match req.sand_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };
    let clay_r = match req.clay_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };
    let bulkd_r = match req.bulkd_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };
    let ph_r = match req.ph_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };
    let rock_r = match req.rock_path.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => Some(GeoRaster::open(p).map_err(|e| e.to_string())?),
        _ => None,
    };

    let n_years = if lulc_rasters.len() > 1 {
        lulc_rasters.len()
    } else {
        req.transient_years.clamp(1, 200) as usize
    };

    let mut tasks = Vec::new();
    for r in 0..rows {
        let fr = r as f64 / rows.max(1) as f64;
        for c in 0..cols {
            let fc = c as f64 / cols.max(1) as f64;
            let Some(v0) = template.value_at(0, r, c) else {
                continue;
            };
            let code0 = v0.round() as i32;
            let Some(m0) = class_map.get(&code0) else {
                continue;
            };
            if m0.preset_id == "masked" {
                continue;
            }

            let (x, y) = template.pixel_xy(r, c);
            let (lat, lon) = template.pixel_lat_lon(r, c, -15.0);

            let mut lulc_series = Vec::with_capacity(n_years);
            for yr in 0..n_years {
                let code = if lulc_rasters.len() > 1 {
                    lulc_rasters[yr]
                        .sample_at_xy_or_frac(0, x, y, fr, fc)
                        .map(|v| v.round() as i32)
                        .unwrap_or(code0)
                } else {
                    code0
                };
                lulc_series.push(code);
            }

            // Extract climate (real multi-band rasters if supplied, else latitude-scaled fallback)
            let fallback_clim = build_synthetic_climate::<f64>(
                (27.0 - 0.45 * lat.abs()).clamp(-4.0, 28.0),
                (2.0 + 0.25 * lat.abs()).clamp(1.5, 16.0),
                140.0,
                lat,
            );
            let mut weather_series = Vec::with_capacity(n_years);
            for yr in 0..n_years {
                let frac = if n_years > 1 {
                    yr as f64 / (n_years - 1) as f64
                } else {
                    0.0
                };
                let t_off = req.temp_delta_c * frac;
                let p_mult = 1.0 + (req.precip_scale - 1.0) * frac;

                let yr_weather: [MonthlyWeather<f64>; 12] = core::array::from_fn(|m| {
                    let b_idx = yr * 12 + m;
                    let p_cm = precip_r
                        .as_ref()
                        .and_then(|pr| {
                            let b = if pr.bands() >= (yr + 1) * 12 {
                                b_idx
                            } else {
                                m % pr.bands().max(1)
                            };
                            pr.sample_at_xy_or_frac(b, x, y, fr, fc)
                        })
                        .map(|v| convert_precip(v, req.precip_unit))
                        .unwrap_or(fallback_clim[m].precip_cm);

                    let tn_c = tmin_r
                        .as_ref()
                        .and_then(|tr| {
                            let b = if tr.bands() >= (yr + 1) * 12 {
                                b_idx
                            } else {
                                m % tr.bands().max(1)
                            };
                            tr.sample_at_xy_or_frac(b, x, y, fr, fc)
                        })
                        .map(|v| convert_temp(v, req.temp_unit))
                        .unwrap_or(fallback_clim[m].tmin_c);

                    let tx_c = tmax_r
                        .as_ref()
                        .and_then(|tr| {
                            let b = if tr.bands() >= (yr + 1) * 12 {
                                b_idx
                            } else {
                                m % tr.bands().max(1)
                            };
                            tr.sample_at_xy_or_frac(b, x, y, fr, fc)
                        })
                        .map(|v| convert_temp(v, req.temp_unit))
                        .unwrap_or(fallback_clim[m].tmax_c.max(tn_c + 6.0));

                    MonthlyWeather {
                        precip_cm: (p_cm * p_mult).max(0.05),
                        tmin_c: tn_c + t_off,
                        tmax_c: (tx_c + t_off).max(tn_c + t_off + 1.0),
                    }
                });
                weather_series.push(yr_weather);
            }

            let scale_frac = |v: f64| -> f64 {
                if req.soilgrids_units && v > 1.0 {
                    if v > 100.0 {
                        v / 1000.0
                    } else {
                        v / 100.0
                    }
                } else {
                    v
                }
            };
            let sand = sand_r
                .as_ref()
                .and_then(|sr| sr.sample_at_xy_or_frac(0, x, y, fr, fc))
                .map(scale_frac)
                .unwrap_or(0.40)
                .clamp(0.05, 0.90);
            let clay = clay_r
                .as_ref()
                .and_then(|cr| cr.sample_at_xy_or_frac(0, x, y, fr, fc))
                .map(scale_frac)
                .unwrap_or(0.30)
                .clamp(0.05, 0.90 - sand);
            let silt = (1.0 - sand - clay).max(0.04);
            let bulkd = bulkd_r
                .as_ref()
                .and_then(|br| br.sample_at_xy_or_frac(0, x, y, fr, fc))
                .map(|v| {
                    if req.soilgrids_units && v > 10.0 {
                        v / 100.0
                    } else {
                        v
                    }
                })
                .unwrap_or(1.28)
                .clamp(0.75, 1.85);
            let ph = ph_r
                .as_ref()
                .and_then(|pr| pr.sample_at_xy_or_frac(0, x, y, fr, fc))
                .map(|v| {
                    if req.soilgrids_units && v > 14.0 {
                        v / 10.0
                    } else {
                        v
                    }
                })
                .unwrap_or(5.6)
                .clamp(3.8, 9.0);
            let rock_frac = rock_r
                .as_ref()
                .and_then(|rr| rr.sample_at_xy_or_frac(0, x, y, fr, fc))
                .map(scale_frac)
                .unwrap_or(0.04)
                .clamp(0.0, 0.85);

            tasks.push(PixelTask {
                row: r,
                col: c,
                lat,
                lon,
                lulc_series,
                weather_series,
                sand,
                silt,
                clay,
                bulkd,
                ph,
                rock_frac,
            });
        }
    }

    let label = format!(
        "GeoTIFF/terrio ({}x{}, {} LULC yr{})",
        rows,
        cols,
        lulc_rasters.len(),
        if lulc_rasters.len() > 1 { "s" } else { "" }
    );
    Ok((template, tasks, label))
}

fn build_biome_and_schedule(
    mapping: &LulcClassMapping,
    lat: f64,
    years: i32,
    cropland_tillage: TillageMode,
) -> (RawBiomeParams<f64>, oxisol_core::Schedule) {
    let mut biome = base_biome_params::<f64>(&mapping.preset_id);
    biome.site_defaults.nelem = 1;
    if let Some(fert) = biome.ferts.get_mut("CUSTOM") {
        fert.feramt = [mapping.fert_n.clamp(0.0, 40.0), 0.0, 0.0];
    }
    if let Some(graz) = biome.grazs.get_mut("CUSTOM") {
        let g = mapping.grazing.clamp(0.0, 0.6);
        graz.flgrem = g;
        graz.fdgrem = g * 0.5;
    }
    let dummy_req = SimulateRequest {
        preset_id: mapping.preset_id.clone(),
        mean_temp_c: 20.0,
        temp_amp_c: 5.0,
        annual_precip_cm: 120.0,
        lat,
        co2_start_ppm: 354.0,
        co2_end_ppm: 422.0,
        sand: 0.4,
        silt: 0.3,
        clay: 0.3,
        bulkd: 1.25,
        ph: 5.5,
        rock_frac: 0.04,
        drain: 1.0,
        years,
        elements: "cn".into(),
        profile: "clean".into(),
        precision: "f64".into(),
        audit_mass_balance: false,
        tillage: if mapping.is_cropland {
            cropland_tillage
        } else {
            TillageMode::None
        },
        fertilizer_n: mapping.fert_n,
        fertilizer_p: 0.0,
        irrigation: IrrigationMode::None,
        fire_interval_yr: 0,
        grazing_intensity: mapping.grazing,
        omad_c: 0.0,
    };
    let sched = build_schedule_from_request(&dummy_req, years);
    (biome, sched)
}

/// Apply realistic land-use conversion disturbance when a pixel transitions between
/// LULC classes (e.g., Forest -> Pasture deforestation burn, or Pasture -> Cropland plowing).
fn apply_luc_transition(
    state: &mut oxisol_core::CellState<f64>,
    prev: &LulcClassMapping,
    next: &LulcClassMapping,
) {
    let was_woody = matches!(
        prev.preset_id.as_str(),
        "amazon_rainforest" | "temperate_deciduous" | "boreal_conifer" | "cerrado_savanna"
    );
    let now_woody = matches!(
        next.preset_id.as_str(),
        "amazon_rainforest" | "temperate_deciduous" | "boreal_conifer" | "cerrado_savanna"
    );

    if was_woody && !now_woody {
        // Deforestation / slash-and-burn clearing: 75% woody biomass exported/combusted,
        // 25% enters dead wood & surface structural litter; temporary root/microbial pulse.
        let slash_frac = 0.25;
        let leaf_c = state.rleavc.total();
        let fbr_c = state.fbrchc.total();
        let rlw_c = state.rlwodc.total();
        let frt_c = state.frootc.total();
        let crt_c = state.crootc.total();

        state.strucc[0].add((leaf_c + fbr_c) * slash_frac, 0.0);
        state.wood2c.add(rlw_c * 0.15, 0.0);
        state.wood3c.add((crt_c + frt_c) * 0.80, 0.0);

        state.rleavc = Default::default();
        state.fbrchc = Default::default();
        state.rlwodc = Default::default();
        state.frootc = Default::default();
        state.crootc = Default::default();
        state.rleave = [0.0; 3];
        state.fbrche = [0.0; 3];
        state.rlwode = [0.0; 3];
        state.froote = [0.0; 3];
        state.croote = [0.0; 3];

        // Ash mineral N pulse + grass establishment seed
        state.minerl[0][0] += 1.2;
        state.aglivc.add(15.0, 0.0);
        state.bglivc.add(25.0, 0.0);
        state.system = SystemKind::GrasslandCrop;
    } else if !prev.is_cropland && next.is_cropland {
        // Pasture/grassland conversion to annual cropland: soil disturbance pulse
        state.clteff = [1.5, 1.5, 1.4, 1.4];
        state.cult_months = 2;
        state.system = SystemKind::GrasslandCrop;
    } else if !was_woody && now_woody {
        // Restoration / secondary forest regrowth
        state.rleavc.add(12.0, 0.0);
        state.frootc.add(15.0, 0.0);
        state.rlwodc.add(25.0, 0.0);
        state.system = if next.preset_id == "cerrado_savanna" {
            SystemKind::Savanna
        } else {
            SystemKind::Forest
        };
    }
}

/// Execute the spatial LULC + Modeled Climate raster simulation in parallel across
/// CPU threads via `rayon`, optionally exporting georeferenced Cloud-Optimized
/// GeoTIFFs via [`GeoRaster::write_cog`].
pub fn execute_lulc_run(req: &LulcRunRequest) -> Result<LulcRunResponse, String> {
    let t0 = Instant::now();
    let spinup_years = req.spinup_years.clamp(10, 500);
    let transient_years = req.transient_years.clamp(5, 150) as usize;
    let co2_start = req.co2_start_ppm.clamp(180.0, 1500.0);
    let co2_end = req.co2_end_ppm.clamp(180.0, 1500.0);

    // Resolve active class legend
    let legend = if !req.class_mappings.is_empty() {
        req.class_mappings.clone()
    } else {
        builtin_lulc_schemes()
            .into_iter()
            .find(|s| s.id == req.scheme_id)
            .unwrap_or_else(|| builtin_lulc_schemes().remove(0))
            .classes
    };
    let class_map: HashMap<i32, LulcClassMapping> =
        legend.iter().map(|m| (m.code, m.clone())).collect();

    let (template, tasks, source_label) = if req.mode.eq_ignore_ascii_case("geotiff") {
        load_real_geotiff_tasks(req, &class_map)?
    } else {
        let (tpl, tks) = build_demo_frontier(
            transient_years,
            req.temp_delta_c.clamp(-6.0, 10.0),
            req.precip_scale.clamp(0.4, 2.0),
        );
        (
            tpl,
            tks,
            format!("Amazon–Cerrado Frontier LUC Demo (EPSG:4326, {transient_years} yr)"),
        )
    };

    let max_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let threads_used = req
        .threads
        .map(|t| t.clamp(1, max_cpus))
        .unwrap_or(max_cpus);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads_used)
        .build()
        .map_err(|e| format!("rayon thread pool error: {e}"))?;

    let cropland_tillage = req.cropland_tillage;
    let pixels: Vec<LulcPixelResult> = pool.install(|| {
        tasks
            .par_iter()
            .filter_map(|task| {
                let init_code = *task.lulc_series.first()?;
                let init_map = class_map.get(&init_code)?;
                if init_map.preset_id == "masked" {
                    return None;
                }

                let total_tr = task.lulc_series.len() as i32;
                let (mut biome, mut sched) = build_biome_and_schedule(
                    init_map,
                    task.lat,
                    spinup_years + total_tr,
                    cropland_tillage,
                );

                let cell_in = CellSiteInput {
                    lat: task.lat,
                    lon: task.lon,
                    elevation: 250.0,
                    sand: task.sand,
                    silt: task.silt,
                    clay: task.clay,
                    bulkd: task.bulkd,
                    rock_frac: task.rock_frac,
                    zminrl_cm: None,
                    ph: task.ph,
                    drain: 1.0,
                };
                let cfg = SimConfig {
                    profile: CompatibilityProfile::ScientificClean,
                    elements: ElementMode::CarbonNitrogen,
                    isotope: IsotopeMode::Unlabeled,
                    co2_ppm_override: Some(co2_start),
                    audit_mass_balance: false,
                    start_year: 1,
                    end_year: spinup_years + total_tr,
                };

                let (mut state, derived) =
                    initialize_cell(&biome, &cell_in, &task.weather_series[0], &sched, &cfg);
                let mut sel = ActiveSelection::from_schedule(&sched);

                // Phase 1: Equilibrium spinup on initial LULC class
                for yr in 1..=spinup_years {
                    for (m, w_mo) in task.weather_series[0].iter().enumerate() {
                        let _ = step_month(
                            &mut state,
                            &derived,
                            &biome,
                            &mut sel,
                            &sched,
                            w_mo,
                            yr,
                            (m + 1) as u8,
                            co2_start,
                            &cfg,
                        );
                    }
                }
                let somsc_initial = state.somsc();

                // Phase 2: Transient multi-year LULC + modeled climate + CO2 ramp
                let mut curr_code = init_code;
                let mut converted = false;
                let mut last_npp = 0.0_f64;
                let mut last_aet = 0.0_f64;

                for (i, &code) in task.lulc_series.iter().enumerate() {
                    if code != curr_code {
                        if let (Some(prev_m), Some(next_m)) =
                            (class_map.get(&curr_code), class_map.get(&code))
                        {
                            if next_m.preset_id != "masked" {
                                apply_luc_transition(&mut state, prev_m, next_m);
                                let (new_b, new_s) = build_biome_and_schedule(
                                    next_m,
                                    task.lat,
                                    spinup_years + total_tr,
                                    cropland_tillage,
                                );
                                biome = new_b;
                                sched = new_s;
                                sel = ActiveSelection::from_schedule(&sched);
                                curr_code = code;
                                converted = true;
                            }
                        }
                    }
                    let frac = if task.lulc_series.len() > 1 {
                        i as f64 / (task.lulc_series.len() - 1) as f64
                    } else {
                        0.0
                    };
                    let co2_yr = co2_start + (co2_end - co2_start) * frac;
                    let yr = spinup_years + (i as i32) + 1;
                    let w_yr = &task.weather_series[i.min(task.weather_series.len() - 1)];

                    let mut ann_npp = 0.0;
                    let mut ann_aet = 0.0;
                    for (m, w_mo) in w_yr.iter().enumerate() {
                        let (mo, _) = step_month(
                            &mut state,
                            &derived,
                            &biome,
                            &mut sel,
                            &sched,
                            w_mo,
                            yr,
                            (m + 1) as u8,
                            co2_yr,
                            &cfg,
                        );
                        ann_npp += mo.npp;
                        ann_aet += mo.aet;
                    }
                    last_npp = ann_npp;
                    last_aet = ann_aet;
                }

                let somsc_final = state.somsc();
                let biomass_c = state.aglivc.total()
                    + state.bglivc.total()
                    + state.rleavc.total()
                    + state.frootc.total()
                    + state.fbrchc.total()
                    + state.rlwodc.total()
                    + state.crootc.total();

                Some(LulcPixelResult {
                    row: task.row,
                    col: task.col,
                    lat: task.lat,
                    lon: task.lon,
                    lulc_initial: init_code,
                    lulc_final: curr_code,
                    converted,
                    somsc_initial,
                    somsc_final,
                    delta_somsc: somsc_final - somsc_initial,
                    som3c: state.som3c.total(),
                    biomass_c,
                    npp: last_npp,
                    aet: last_aet,
                    minerl_n: state.minerl[0][0],
                })
            })
            .collect()
    });

    // Optional real Cloud-Optimized GeoTIFF export via `terrio`
    let mut exported_cogs = Vec::new();
    if let Some(dir_str) = req.export_cog_dir.as_deref().map(str::trim) {
        if !dir_str.is_empty() {
            let out_dir = Path::new(dir_str);
            std::fs::create_dir_all(out_dir)
                .map_err(|e| format!("Failed to create export dir '{dir_str}': {e}"))?;
            let rows = template.rows();
            let cols = template.cols();
            let nd = -9999.0_f64;

            type LayerGetter = fn(&LulcPixelResult) -> f64;
            let layers: [(&str, LayerGetter); 6] = [
                ("oxisol_somsc.tif", |p| p.somsc_final),
                ("oxisol_delta_somsc.tif", |p| p.delta_somsc),
                ("oxisol_biomass_c.tif", |p| p.biomass_c),
                ("oxisol_npp.tif", |p| p.npp),
                ("oxisol_aet.tif", |p| p.aet),
                ("oxisol_minerl_n.tif", |p| p.minerl_n),
            ];

            for (fname, getter) in layers {
                let mut buf = vec![nd; rows * cols];
                for p in &pixels {
                    buf[p.row * cols + p.col] = getter(p);
                }
                let out_path = out_dir.join(fname);
                let out_raster = template
                    .derive_like(nd, buf)
                    .map_err(|e| format!("derive_like failed for {fname}: {e}"))?;
                out_raster
                    .write_cog(&out_path)
                    .map_err(|e| format!("COG write failed for {fname}: {e}"))?;
                exported_cogs.push(out_path.display().to_string());
            }
        }
    }

    let elapsed_secs = t0.elapsed().as_secs_f64().max(1.0e-9);
    let n_active = pixels.len();
    let n_converted = pixels.iter().filter(|p| p.converted).count();
    let total_yrs = (spinup_years as usize) + transient_years;
    let sum_som: f64 = pixels.iter().map(|p| p.somsc_final).sum();
    let sum_delta: f64 = pixels.iter().map(|p| p.delta_somsc).sum();
    let denom = n_active.max(1) as f64;
    let mean_delta = sum_delta / denom;

    Ok(LulcRunResponse {
        source_label,
        rows: template.rows(),
        cols: template.cols(),
        n_active,
        n_converted,
        total_years_per_pixel: total_yrs,
        threads_used,
        elapsed_secs,
        pixel_years_per_sec: (n_active * total_yrs) as f64 / elapsed_secs,
        mean_somsc: sum_som / denom,
        mean_delta_somsc: mean_delta,
        mean_delta_t_c_ha: mean_delta / 100.0,
        legend,
        exported_cogs,
        pixels,
    })
}
