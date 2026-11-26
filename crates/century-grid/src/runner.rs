//! Lock-free, NUMA-friendly parallel grid execution engine (`rayon`).

use std::collections::HashMap;
use std::time::Instant;

use century_core::{
    run_cell_from_state, run_cell_simulation, AnnualOutput, CellDerivedParams, CellState, Float,
    SimConfig,
};
use century_io::checkpoint::NativeCellCheckpoint;
use century_io::vemap::{PreloadedCell, PreloadedGrid};
use rayon::prelude::*;

/// Execution configuration for parallel multi-core grid runs (Ryzen -> EPYC 256T).
#[derive(Debug, Clone)]
pub struct GridRunConfig {
    /// Core biogeochemical simulation configuration.
    pub sim: SimConfig,
    /// Optional explicit thread count (`None` uses all logical cores).
    pub num_threads: Option<usize>,
    /// Cells per parallel chunk to amortize scheduling overhead & preserve CCD/NUMA locality.
    pub chunk_size: usize,
    /// Group cells by biome before dispatch to eliminate branch divergence across cores.
    pub group_by_biome: bool,
}

impl Default for GridRunConfig {
    fn default() -> Self {
        Self {
            sim: SimConfig::default(),
            num_threads: None,
            chunk_size: 16,
            group_by_biome: true,
        }
    }
}

/// Output record for a single completed cell.
#[derive(Debug, Clone)]
pub struct CellRunResult<F: Float> {
    /// Dense cell index matching `PreloadedCell::cell_index`.
    pub cell_index: usize,
    /// Grid row.
    pub row: usize,
    /// Grid column.
    pub col: usize,
    /// Biome ID.
    pub biome_id: i32,
    /// Final cell state.
    pub state: CellState<F>,
    /// Derived per-cell parameters.
    pub derived: CellDerivedParams<F>,
    /// Final year's annual summary.
    pub last_annual: AnnualOutput<F>,
}

/// Aggregate throughput & timing statistics for a grid simulation run.
#[derive(Debug, Clone, Copy)]
pub struct GridRunStats {
    /// Number of cells simulated.
    pub ncells: usize,
    /// Number of simulation years per cell.
    pub nyears: usize,
    /// Wall-clock duration in seconds.
    pub elapsed_secs: f64,
    /// Throughput in cell-years per second.
    pub cell_years_per_sec: f64,
    /// Throughput in cell-months per second.
    pub cell_months_per_sec: f64,
}

#[inline]
fn run_one_cell<F: Float>(
    grid: &PreloadedGrid<F>,
    cell: &PreloadedCell<F>,
    sim: &SimConfig,
    co2_by_year: Option<&[F]>,
    restart: Option<&(CellState<F>, CellDerivedParams<F>)>,
) -> CellRunResult<F> {
    let biome = &grid.biomes[&cell.biome_id];
    let schedule = &grid.schedules[&cell.biome_id];
    let weather_slice = cell
        .transient_weather
        .as_deref()
        .unwrap_or(std::slice::from_ref(&cell.mean_weather));
    let (state, derived, last_annual) = if let Some((st, der)) = restart {
        run_cell_from_state(
            st.clone(),
            der.clone(),
            biome,
            weather_slice,
            schedule,
            co2_by_year,
            sim,
        )
    } else {
        run_cell_simulation(
            biome,
            &cell.site_input,
            weather_slice,
            schedule,
            co2_by_year,
            sim,
        )
    };
    CellRunResult {
        cell_index: cell.cell_index,
        row: cell.row,
        col: cell.col,
        biome_id: cell.biome_id,
        state,
        derived,
        last_annual,
    }
}

/// Execute all cells serially on a single thread (baseline for scaling & profiling).
pub fn run_grid_serial<F: Float>(
    grid: &PreloadedGrid<F>,
    cfg: &GridRunConfig,
    co2_by_year: Option<&[F]>,
) -> (Vec<CellRunResult<F>>, GridRunStats) {
    let t0 = Instant::now();
    let results: Vec<CellRunResult<F>> = grid
        .cells
        .iter()
        .map(|cell| run_one_cell(grid, cell, &cfg.sim, co2_by_year, None))
        .collect();
    let elapsed = t0.elapsed().as_secs_f64().max(1.0e-9);
    let nyears = (cfg.sim.end_year - cfg.sim.start_year + 1).max(0) as usize;
    let cy = (results.len() * nyears) as f64;
    let stats = GridRunStats {
        ncells: results.len(),
        nyears,
        elapsed_secs: elapsed,
        cell_years_per_sec: cy / elapsed,
        cell_months_per_sec: (cy * 12.0) / elapsed,
    };
    (results, stats)
}

/// Execute all cells in parallel using `rayon` with biome-grouped work ordering,
/// zero shared mutable state, and deterministic output order (`0..ncells`).
///
/// Designed for linear scaling across both 16-thread desktop CPUs and
/// 128–256-thread dual-socket AMD EPYC + 512 GB RAM servers.
pub fn run_grid_parallel<F: Float>(
    grid: &PreloadedGrid<F>,
    cfg: &GridRunConfig,
    co2_by_year: Option<&[F]>,
) -> (Vec<CellRunResult<F>>, GridRunStats) {
    run_grid_parallel_with_restart(grid, cfg, co2_by_year, None)
}

type CheckpointMap<F> = HashMap<(usize, usize), (CellState<F>, CellDerivedParams<F>)>;

/// Execute all cells in parallel using `rayon`, optionally restarting each cell from a
/// lossless [`NativeCellCheckpoint`] (matched by `(row, col)`).
pub fn run_grid_parallel_with_restart<F: Float>(
    grid: &PreloadedGrid<F>,
    cfg: &GridRunConfig,
    co2_by_year: Option<&[F]>,
    checkpoints: Option<&[NativeCellCheckpoint<F>]>,
) -> (Vec<CellRunResult<F>>, GridRunStats) {
    let t0 = Instant::now();

    let ckpt_map: Option<CheckpointMap<F>> = checkpoints.map(|slice| {
        slice
            .iter()
            .map(|c| ((c.row, c.col), (c.state.clone(), c.derived.clone())))
            .collect()
    });

    // Order cell indices by biome to maximize instruction-cache & biome-table L2/L3 locality.
    let mut order: Vec<usize> = (0..grid.cells.len()).collect();
    if cfg.group_by_biome {
        order.sort_unstable_by_key(|&i| (grid.cells[i].biome_id, i));
    }

    let chunk = cfg.chunk_size.max(1);
    let execute = || {
        order
            .par_chunks(chunk)
            .flat_map_iter(|idx_chunk| {
                idx_chunk.iter().map(|&idx| {
                    let cell = &grid.cells[idx];
                    let restart = ckpt_map.as_ref().and_then(|m| m.get(&(cell.row, cell.col)));
                    run_one_cell(grid, cell, &cfg.sim, co2_by_year, restart)
                })
            })
            .collect::<Vec<CellRunResult<F>>>()
    };

    let mut results = if let Some(n_threads) = cfg.num_threads {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n_threads.max(1))
            .build()
            .expect("failed to build rayon thread pool");
        pool.install(execute)
    } else {
        execute()
    };

    // Restore deterministic canonical cell_index order.
    if cfg.group_by_biome {
        results.sort_unstable_by_key(|r| r.cell_index);
    }

    let elapsed = t0.elapsed().as_secs_f64().max(1.0e-9);
    let nyears = (cfg.sim.end_year - cfg.sim.start_year + 1).max(0) as usize;
    let cy = (results.len() * nyears) as f64;
    let stats = GridRunStats {
        ncells: results.len(),
        nyears,
        elapsed_secs: elapsed,
        cell_years_per_sec: cy / elapsed,
        cell_months_per_sec: (cy * 12.0) / elapsed,
    };
    (results, stats)
}
