//! Strongly-typed HTTP/JSON request and response DTOs for the Oxisol REST API
//! and embedded Oxisol Studio web interface.

use serde::{Deserialize, Serialize};

/// Health check response returned by `GET /api/health`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Static service status string (`"ok"`).
    pub status: String,
    /// Service name (`"Oxisol"`).
    pub service: String,
    /// Crate version string.
    pub version: String,
}

/// Host hardware and runtime metadata returned by `GET /api/system`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoResponse {
    /// Application brand name.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Number of logical CPU threads detected on this machine/server.
    pub logical_cpus: usize,
    /// Byte size of `CellState<f64>` (64-byte aligned for L1d residency).
    pub cell_state_f64_bytes: usize,
    /// Byte size of `CellState<f32>`.
    pub cell_state_f32_bytes: usize,
    /// Whether an external VEMAP/spatial dataset directory was attached at startup.
    pub has_attached_grid: bool,
    /// Display label for the attached dataset directory, if any.
    pub attached_grid_label: Option<String>,
}

/// Management tillage regime exposed in the Studio UI and REST API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TillageMode {
    /// Natural ecosystem with no cultivation or grain harvest.
    #[default]
    None,
    /// Annual harvest with zero soil disturbance (no-till conservation agriculture).
    NoTillHarvest,
    /// Annual harvest plus spring moldboard plowing (`CULT P`).
    MoldboardPlow,
}

/// Irrigation regime exposed in the Studio UI and REST API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IrrigationMode {
    /// Rainfed system (no irrigation).
    #[default]
    None,
    /// Automatic irrigation maintaining root-zone water above 75% AWHC (`AUIRRI = 1`).
    Auto75,
}

/// Ecosystem preset summary returned by `GET /api/presets` so the Studio UI
/// can populate calibrated defaults across global biomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSummary {
    /// Unique preset identifier.
    pub id: String,
    /// English display title.
    pub name_en: String,
    /// Portuguese (PT-BR) display title.
    pub name_pt: String,
    /// Short English ecological summary.
    pub desc_en: String,
    /// Short Portuguese ecological summary.
    pub desc_pt: String,
    /// Ecosystem system kind label (`"grassland"`, `"forest"`, `"savanna"`, `"cropland"`).
    pub system_kind: String,
    /// Default request payload corresponding to this preset.
    pub defaults: SimulateRequest,
}

/// Request payload for `POST /api/simulate` (single-cell interactive scenario run).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRequest {
    /// Base ecosystem preset ID (e.g. `"temperate_grassland"`, `"cerrado_savanna"`).
    pub preset_id: String,
    /// Mean annual air temperature (°C).
    pub mean_temp_c: f64,
    /// Seasonal temperature half-amplitude (°C, summer-to-mean).
    pub temp_amp_c: f64,
    /// Total annual precipitation (cm/yr).
    pub annual_precip_cm: f64,
    /// Site latitude in degrees (`-65.0..=70.0`; negative = Southern Hemisphere).
    pub lat: f64,
    /// Starting atmospheric CO₂ concentration (ppm).
    pub co2_start_ppm: f64,
    /// Ending atmospheric CO₂ concentration (ppm; linearly ramped across simulation).
    pub co2_end_ppm: f64,
    /// Soil sand fraction `[0.02, 0.96]`.
    pub sand: f64,
    /// Soil silt fraction `[0.02, 0.96]`.
    pub silt: f64,
    /// Soil clay fraction `[0.02, 0.96]`.
    pub clay: f64,
    /// Soil bulk density ($\text{g}\cdot\text{cm}^{-3}$).
    pub bulkd: f64,
    /// Soil pH (`4.0..=9.0`).
    pub ph: f64,
    /// Volumetric rock fraction `[0.0, 0.85]`.
    pub rock_frac: f64,
    /// Soil drainage factor (`0.1` poorly drained anaerobic .. `1.0` well drained).
    pub drain: f64,
    /// Number of simulation years (`5..=1000`).
    pub years: i32,
    /// Active nutrient cycles: `"cn"`, `"cnp"`, or `"cnps"`.
    pub elements: String,
    /// Compatibility profile: `"clean"` (`ScientificClean`) or `"parity"` (`VemapV45Parity`).
    pub profile: String,
    /// Floating-point precision: `"f64"` or `"f32"`.
    pub precision: String,
    /// Whether to audit strict step-by-step C/N/P/S mass balance.
    #[serde(default = "default_true")]
    pub audit_mass_balance: bool,
    /// Tillage & harvest regime.
    #[serde(default)]
    pub tillage: TillageMode,
    /// Annual mineral N fertilizer addition ($\text{g N}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    #[serde(default)]
    pub fertilizer_n: f64,
    /// Annual mineral P fertilizer addition ($\text{g P}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    #[serde(default)]
    pub fertilizer_p: f64,
    /// Irrigation mode.
    #[serde(default)]
    pub irrigation: IrrigationMode,
    /// Prescribed fire return interval in years (`0` = no fire).
    #[serde(default)]
    pub fire_interval_yr: i32,
    /// Grazing intensity fraction (`0.0` = none .. `0.6` = heavy).
    #[serde(default)]
    pub grazing_intensity: f64,
    /// Annual organic matter (manure/compost) addition ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    #[serde(default)]
    pub omad_c: f64,
}

fn default_true() -> bool {
    true
}

/// Annual time-series point returned by `POST /api/simulate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualPoint {
    /// Simulation year (1-based).
    pub year: i32,
    /// Active SOM carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som1c: f64,
    /// Slow SOM carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som2c: f64,
    /// Passive SOM carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som3c: f64,
    /// Total soil organic matter carbon (`som1c_soil + som2c + som3c`, $\text{g C}\cdot\text{m}^{-2}$).
    pub somsc: f64,
    /// Aboveground herbaceous live carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub aglivc: f64,
    /// Belowground herbaceous root carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub bglivc: f64,
    /// Forest large wood carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub rlwodc: f64,
    /// Forest leaf carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub rleavc: f64,
    /// Total surface + soil litter carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub litter_c: f64,
    /// Annual net primary production ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub npp: f64,
    /// Annual heterotrophic respiration ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub resp: f64,
    /// Annual actual evapotranspiration ($\text{cm}\cdot\text{yr}^{-1}$).
    pub aet: f64,
    /// Annual potential evapotranspiration ($\text{cm}\cdot\text{yr}^{-1}$).
    pub pet: f64,
    /// Top-layer volumetric soil water at year end (cm).
    pub soil_water_cm: f64,
    /// Surface mineral nitrogen ($\text{g N}\cdot\text{m}^{-2}$).
    pub minerl_n: f64,
    /// Surface mineral phosphorus ($\text{g P}\cdot\text{m}^{-2}$).
    pub minerl_p: f64,
    /// Surface mineral sulfur ($\text{g S}\cdot\text{m}^{-2}$).
    pub minerl_s: f64,
    /// Atmospheric CO₂ concentration for this year (ppm).
    pub co2_ppm: f64,
}

/// Final compartment snapshot for the interactive biogeochemical flow diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshot {
    /// Total live vegetation carbon (herbaceous + woody, $\text{g C}\cdot\text{m}^{-2}$).
    pub vegetation_c: f64,
    /// Total litter + dead wood carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub litter_and_deadwood_c: f64,
    /// Active SOM carbon (surface + soil, $\text{g C}\cdot\text{m}^{-2}$).
    pub som1_c: f64,
    /// Slow SOM carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som2_c: f64,
    /// Passive SOM carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som3_c: f64,
    /// Total soil organic carbon (`somsc`, $\text{g C}\cdot\text{m}^{-2}$).
    pub somsc: f64,
    /// Total soil organic nitrogen ($\text{g N}\cdot\text{m}^{-2}$).
    pub som_n: f64,
    /// Total mineral N across soil profile ($\text{g N}\cdot\text{m}^{-2}$).
    pub profile_minerl_n: f64,
    /// Total mineral P across soil profile ($\text{g P}\cdot\text{m}^{-2}$).
    pub profile_minerl_p: f64,
    /// Total mineral S across soil profile ($\text{g S}\cdot\text{m}^{-2}$).
    pub profile_minerl_s: f64,
    /// Final-year annual NPP ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub final_npp: f64,
    /// Final-year annual heterotrophic respiration ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub final_resp: f64,
    /// Final-year annual AET ($\text{cm}\cdot\text{yr}^{-1}$).
    pub final_aet: f64,
    /// Cumulative carbon input over the full run ($\text{g C}\cdot\text{m}^{-2}$).
    pub cum_c_in: f64,
    /// Cumulative carbon export + respiration ($\text{g C}\cdot\text{m}^{-2}$).
    pub cum_c_out: f64,
    /// Maximum absolute C/N/P/S mass-balance residual observed across all months.
    pub max_mass_balance_err: f64,
}

/// Complete response returned by `POST /api/simulate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResponse {
    /// Elapsed wall-clock time in milliseconds.
    pub elapsed_ms: f64,
    /// Effective single-cell throughput in cell-years per second.
    pub throughput_cell_yr_s: f64,
    /// Profile used (`"ScientificClean"` or `"VemapV45Parity"`).
    pub profile: String,
    /// Precision used (`"f64"` or `"f32"`).
    pub precision: String,
    /// Annual trajectory over the requested simulation horizon.
    pub series: Vec<AnnualPoint>,
    /// Final biogeochemical pool snapshot and conservation summary.
    pub final_pools: PoolSnapshot,
}

/// Request payload for `POST /api/grid/run` (multi-core spatial grid execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRunRequest {
    /// Use the server's attached VEMAP/spatial dataset if available (`false` = synthetic gradient grid).
    #[serde(default = "default_true")]
    pub use_attached_grid: bool,
    /// Synthetic grid rows (ignored when using attached dataset; clamped `4..=48`).
    #[serde(default = "default_grid_rows")]
    pub rows: usize,
    /// Synthetic grid columns (ignored when using attached dataset; clamped `4..=64`).
    #[serde(default = "default_grid_cols")]
    pub cols: usize,
    /// Simulation years per cell (clamped `5..=1000`).
    #[serde(default = "default_grid_years")]
    pub years: i32,
    /// Optional worker thread count (`None` = all logical cores).
    pub threads: Option<usize>,
    /// Parallel chunk size for `rayon` work-stealing (`1..=256`).
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    /// Compatibility profile (`"clean"` or `"parity"`).
    #[serde(default = "default_clean")]
    pub profile: String,
    /// Precision (`"f64"` or `"f32"`).
    #[serde(default = "default_f64")]
    pub precision: String,
}

fn default_grid_rows() -> usize {
    24
}
fn default_grid_cols() -> usize {
    48
}
fn default_grid_years() -> i32 {
    100
}
fn default_chunk_size() -> usize {
    16
}
fn default_clean() -> String {
    "clean".to_string()
}
fn default_f64() -> String {
    "f64".to_string()
}

/// Compact per-cell output record for the 2D Spatial Heatmap Canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCellSummary {
    /// Grid row index.
    pub row: usize,
    /// Grid column index.
    pub col: usize,
    /// Biome identifier.
    pub biome_id: i32,
    /// Total soil organic carbon (`somsc`, $\text{g C}\cdot\text{m}^{-2}$).
    pub somsc: f64,
    /// Passive SOM carbon (`som3c`, $\text{g C}\cdot\text{m}^{-2}$).
    pub som3c: f64,
    /// Forest large wood carbon (`rlwodc`, $\text{g C}\cdot\text{m}^{-2}$).
    pub rlwodc: f64,
    /// Surface-layer soil water (`asmos[0]`, cm).
    pub soil_water_cm: f64,
    /// Surface mineral nitrogen (`minerl[0][0]`, $\text{g N}\cdot\text{m}^{-2}$).
    pub minerl_n: f64,
    /// Final-year annual NPP ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub npp: f64,
    /// Final-year annual AET ($\text{cm}\cdot\text{yr}^{-1}$).
    pub aet: f64,
}

/// Response returned by `POST /api/grid/run`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRunResponse {
    /// Human-readable label of the executed grid source.
    pub source_label: String,
    /// Total rows in the grid bounding box.
    pub rows: usize,
    /// Total columns in the grid bounding box.
    pub cols: usize,
    /// Number of active cells simulated.
    pub ncells: usize,
    /// Number of years simulated per cell.
    pub nyears: usize,
    /// Worker threads used by `rayon`.
    pub threads_used: usize,
    /// Wall-clock duration in seconds.
    pub elapsed_secs: f64,
    /// Measured throughput in cell-years per second.
    pub cell_years_per_sec: f64,
    /// Measured throughput in cell-months per second.
    pub cell_months_per_sec: f64,
    /// Linearly projected throughput on a modern 128-core / 256-thread high-performance server.
    pub hpc_256t_est_cell_yr_s: f64,
    /// Mean continental `somsc` across active cells ($\text{g C}\cdot\text{m}^{-2}$).
    pub mean_somsc: f64,
    /// Mean continental annual NPP ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub mean_npp: f64,
    /// Per-cell spatial map outputs.
    pub cells: Vec<GridCellSummary>,
}
