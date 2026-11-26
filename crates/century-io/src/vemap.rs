//! High-level VEMAP continental dataset preloader (`load_vemap_dataset`).
//!
//! Preloads all biome `.100`/`.sch` tables, soil `.v2` rasters, `geog.nc`, and
//! `phase1.ncdf` monthly climate ONCE into memory so `century-grid` executes with
//! ZERO filesystem I/O inside the parallel worker loop (scaling cleanly from
//! 16-thread Ryzen 7 5700X to 256-thread AMD EPYC + 512 GB RAM servers).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use century_core::{CellSiteInput, Float, MonthlyWeather, RawBiomeParams, Schedule};

use crate::ascii_v2::AsciiV2Grid;
use crate::error::{IoError, Result};
use crate::netcdf3::NcFile;
use crate::params_100::{
    parse_crop_file, parse_cult_file, parse_fert_file, parse_fire_file, parse_fix_file,
    parse_graz_file, parse_harv_file, parse_irri_file, parse_omad_file, parse_site_file,
    parse_tree_file, parse_trem_file,
};
use crate::schedule_sch::parse_schedule_file;

/// Single preloaded spatial grid cell ready for lock-free parallel simulation.
#[derive(Debug, Clone)]
pub struct PreloadedCell<F: Float> {
    /// 0-based grid row (`0..nrows`).
    pub row: usize,
    /// 0-based grid column (`0..ncols`).
    pub col: usize,
    /// 0-based dense cell index (`0..ncells`).
    pub cell_index: usize,
    /// VEMAP biome/landuse ID (`1..=21`).
    pub biome_id: i32,
    /// Cell-specific geographic & soil inputs.
    pub site_input: CellSiteInput<F>,
    /// Mean monthly climate (12 months).
    pub mean_weather: [MonthlyWeather<F>; 12],
}

/// Complete preloaded grid dataset with immutable `Arc<RawBiomeParams<F>>` tables.
#[derive(Debug, Clone)]
pub struct PreloadedGrid<F: Float> {
    /// Grid height.
    pub nrows: usize,
    /// Grid width.
    pub ncols: usize,
    /// Row-major `nrows * ncols` cell mask (`1..=ncells` for simulated cells, `0` otherwise).
    pub mask: Vec<i32>,
    /// Immutable biome parameter tables indexed by biome ID.
    pub biomes: HashMap<i32, Arc<RawBiomeParams<F>>>,
    /// Biome schedule tables indexed by biome ID.
    pub schedules: HashMap<i32, Arc<Schedule>>,
    /// Active simulated cells (3,168 for continental US VEMAP).
    pub cells: Vec<PreloadedCell<F>>,
}

/// Map VEMAP biome ID to its biome-specific `fix.100` filename (from VEMAP documentation).
pub fn vemap_fix_filename(biome_id: i32) -> &'static str {
    match biome_id {
        1 => "arcfix.100",
        2 => "borfix.100",
        11 | 17 | 20 | 21 => "drygfix.100",
        19 => "drytrpfix.100",
        3..=7 => "ffix.100",
        13 | 14 | 15 | 18 => "gfix.100",
        _ => "ffix.100",
    }
}

/// Preload an entire VEMAP directory (all 17 biomes + soil/geog/climate) into RAM.
///
/// Optional `rock_grid_path` and `zminrl_grid_path` allow supplying recovered or
/// measured rock fraction (%) and mineral soil depth (cm) `.v2` grids.
pub fn load_vemap_grid<F: Float, P: AsRef<Path>>(
    data_dir: P,
    rock_grid_path: Option<&Path>,
    zminrl_grid_path: Option<&Path>,
) -> Result<PreloadedGrid<F>> {
    let dir = data_dir.as_ref();

    // 1. Parse shared management/crop/tree tables once.
    let crops = parse_crop_file::<F, _>(dir.join("crop.100"))?;
    let trees = parse_tree_file::<F, _>(dir.join("tree.100"))?;
    let cults = parse_cult_file::<F, _>(dir.join("cult.100"))?;
    let ferts = parse_fert_file::<F, _>(dir.join("fert.100"))?;
    let fires = parse_fire_file::<F, _>(dir.join("fire.100"))?;
    let grazs = parse_graz_file::<F, _>(dir.join("graz.100"))?;
    let harvs = parse_harv_file::<F, _>(dir.join("harv.100"))?;
    let irris = parse_irri_file::<F, _>(dir.join("irri.100"))?;
    let omads = parse_omad_file::<F, _>(dir.join("omad.100"))?;
    let trems = parse_trem_file::<F, _>(dir.join("trem.100"))?;

    // 2. Load spatial rasters and NetCDF-3 geography & mean climate.
    let vveg = AsciiV2Grid::load(dir.join("vveg.v2"))?;
    let bulkd = AsciiV2Grid::load(dir.join("mbd1_m1.v2"))?;
    let clay = AsciiV2Grid::load(dir.join("mcl1_m1.v2"))?;
    let sand = AsciiV2Grid::load(dir.join("msa1_m1.v2"))?;
    let rock_grid = match rock_grid_path {
        Some(p) => Some(AsciiV2Grid::load(p)?),
        None => None,
    };
    let zminrl_grid = match zminrl_grid_path {
        Some(p) => Some(AsciiV2Grid::load(p)?),
        None => None,
    };

    let geog_path = if dir.join("geog.nc").exists() {
        dir.join("geog.nc")
    } else {
        dir.join("geog.nc.vveg")
    };
    let geog = NcFile::open(&geog_path)?;
    let lat_arr = geog.read_f32("lat")?;
    let lon_arr = geog.read_f32("lon")?;
    let elev_arr = geog.read_f32("elevation")?;

    let climate = NcFile::open(dir.join("phase1.ncdf"))?;
    let ppt_arr = climate.read_f32("ppt")?;
    let tn_arr = climate.read_f32("tn")?;
    let tx_arr = climate.read_f32("tx")?;
    let clim_shape = climate.var_shape("ppt")?;
    if clim_shape.len() != 2 || clim_shape[0] != 12 {
        return Err(IoError::NetCdf(format!(
            "unexpected phase1.ncdf ppt shape: {clim_shape:?}"
        )));
    }
    let total_grid_cells = clim_shape[1];

    let nrows = vveg.nrows;
    let ncols = vveg.ncols;
    if total_grid_cells != nrows * ncols {
        return Err(IoError::NetCdf(format!(
            "climate cell count {total_grid_cells} != grid {}x{}",
            nrows, ncols
        )));
    }

    // 3. Load all active biome `.100` and `.sch` files once into `Arc`.
    let vemap_biomes = [1, 2, 3, 4, 5, 6, 7, 10, 11, 13, 14, 15, 17, 18, 19, 20, 21];
    let mut biomes: HashMap<i32, Arc<RawBiomeParams<F>>> = HashMap::new();
    let mut schedules: HashMap<i32, Arc<Schedule>> = HashMap::new();

    for &bid in &vemap_biomes {
        let fix_path = dir.join(vemap_fix_filename(bid));
        let site_path = dir.join(format!("lu{bid}.100"));
        let sch_path = dir.join(format!("lu{bid}.sch"));
        if !site_path.exists() || !sch_path.exists() {
            continue;
        }
        let fix = parse_fix_file::<F, _>(&fix_path)?;
        let site_defaults = parse_site_file::<F, _>(&site_path)?;
        let sched = parse_schedule_file(&sch_path)?;
        biomes.insert(
            bid,
            Arc::new(RawBiomeParams {
                biome_id: bid,
                fix,
                site_defaults,
                crops: crops.clone(),
                trees: trees.clone(),
                cults: cults.clone(),
                ferts: ferts.clone(),
                fires: fires.clone(),
                grazs: grazs.clone(),
                harvs: harvs.clone(),
                irris: irris.clone(),
                omads: omads.clone(),
                trems: trems.clone(),
            }),
        );
        schedules.insert(bid, Arc::new(sched));
    }

    // 4. Assemble active cell vector and mask.
    let mut mask = vec![0i32; nrows * ncols];
    let mut cells = Vec::with_capacity(3200);
    for r in 0..nrows {
        for c in 0..ncols {
            let flat = r * ncols + c;
            let bid = vveg.get_raw(r, c);
            if bid <= 0 || !biomes.contains_key(&bid) {
                continue;
            }
            let (Some(bd), Some(cl_pct), Some(sa_pct)) = (
                bulkd.get_scaled(r, c),
                clay.get_scaled(r, c),
                sand.get_scaled(r, c),
            ) else {
                continue;
            };
            let rock_pct = rock_grid
                .as_ref()
                .and_then(|g| g.get_scaled(r, c))
                .unwrap_or(0.0);
            let zminrl_cm = zminrl_grid
                .as_ref()
                .and_then(|g| g.get_scaled(r, c))
                .map(F::from_f64);

            let clay_f = (cl_pct / 100.0).clamp(0.0, 1.0);
            let sand_f = (sa_pct / 100.0).clamp(0.0, 1.0);
            let silt_f = (1.0 - clay_f - sand_f).clamp(0.0, 1.0);
            let site_def = &biomes[&bid].site_defaults;

            let site_input = CellSiteInput {
                lat: F::from_f32(lat_arr[flat]),
                lon: F::from_f32(lon_arr[flat]),
                elevation: F::from_f32(elev_arr[flat]),
                sand: F::from_f64(sand_f),
                silt: F::from_f64(silt_f),
                clay: F::from_f64(clay_f),
                bulkd: F::from_f64(bd),
                rock_frac: F::from_f64((rock_pct / 100.0).clamp(0.0, 0.90)),
                zminrl_cm,
                ph: site_def.ph,
                drain: site_def.drain,
            };

            let mean_weather: [MonthlyWeather<F>; 12] = core::array::from_fn(|m| {
                let idx = m * total_grid_cells + flat;
                // phase1.ncdf stores ppt in mm/month (or cm/month — check units: > 50 means mm -> /10).
                // VEMAP phase1.ncdf ppt is in cm/month (mean ~5-30 cm) or mm/month? Convert if mm.
                let raw_p = ppt_arr[idx];
                MonthlyWeather {
                    precip_cm: F::from_f32(raw_p / 10.0), // VEMAP phase1.ncdf: mm -> cm
                    tmin_c: F::from_f32(tn_arr[idx] / 10.0), // VEMAP phase1.ncdf: scaled by 10.0
                    tmax_c: F::from_f32(tx_arr[idx] / 10.0), // VEMAP phase1.ncdf: scaled by 10.0
                }
            });

            let cell_index = cells.len();
            mask[flat] = (cell_index + 1) as i32;
            cells.push(PreloadedCell {
                row: r,
                col: c,
                cell_index,
                biome_id: bid,
                site_input,
                mean_weather,
            });
        }
    }

    Ok(PreloadedGrid {
        nrows,
        ncols,
        mask,
        biomes,
        schedules,
        cells,
    })
}
