//! `century` CLI — run, validate, and benchmark `century-rs` biogeochemical simulations.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use century_core::{CompatibilityProfile, ElementMode, Float, SimConfig};
use century_grid::{
    batch_defac_f32x8, batch_som_decay_f32x8, run_grid_parallel, run_grid_parallel_with_restart,
    run_grid_serial, GridRunConfig, SomSoaBatchF32,
};
use century_io::checkpoint::{
    read_native_checkpoint, read_site_nc, write_native_checkpoint, write_site_nc,
    NativeCellCheckpoint, SiteNcPools,
};
use century_io::co2_dat::Co2Series;
use century_io::vemap::{attach_transient_weather, load_vemap_grid};
use clap::{Parser, Subcommand, ValueEnum};
use wide::f32x8;

#[derive(Parser)]
#[command(
    name = "century",
    version,
    about = "Clean-room open-source Rust implementation of the CENTURY biogeochemical model"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum ProfileArg {
    Clean,
    Parity,
}

impl From<ProfileArg> for CompatibilityProfile {
    fn from(p: ProfileArg) -> Self {
        match p {
            ProfileArg::Clean => CompatibilityProfile::ScientificClean,
            ProfileArg::Parity => CompatibilityProfile::VemapV45Parity,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum PrecisionArg {
    F64,
    F32,
}

#[derive(clap::Args)]
struct RunArgs {
    /// Path to input dataset directory containing .100, .sch, .v2, geog.nc, phase1.ncdf.
    #[arg(long)]
    data_dir: PathBuf,
    /// Optional path to rock fraction (%) .v2 grid.
    #[arg(long)]
    rock: Option<PathBuf>,
    /// Optional path to mineral soil depth (cm) .v2 grid.
    #[arg(long)]
    zminrl: Option<PathBuf>,
    /// Starting simulation year (default 1).
    #[arg(long, default_value_t = 1)]
    start_year: i32,
    /// Number of simulation years (default 1000).
    #[arg(long, default_value_t = 1000)]
    years: i32,
    /// Optional path to annual CO2 table (`co2.dat`, columns: `year ppm`).
    #[arg(long)]
    co2_file: Option<PathBuf>,
    /// Optional transient precipitation NetCDF-3 file (e.g. `pptT3.nc`).
    #[arg(long)]
    transient_ppt: Option<PathBuf>,
    /// Optional transient min temperature NetCDF-3 file (e.g. `tnT3.nc`).
    #[arg(long)]
    transient_tn: Option<PathBuf>,
    /// Optional transient max temperature NetCDF-3 file (e.g. `txT3.nc`).
    #[arg(long)]
    transient_tx: Option<PathBuf>,
    /// Optional lossless native JSON checkpoint to restart from (e.g. spinup -> transient).
    #[arg(long)]
    restart_checkpoint: Option<PathBuf>,
    /// Compatibility profile (`clean` = ScientificClean default, `parity` = VemapV45Parity).
    #[arg(long, value_enum, default_value_t = ProfileArg::Clean)]
    profile: ProfileArg,
    /// Floating-point precision (`f64` recommended for clean science, `f32` for parity).
    #[arg(long, value_enum, default_value_t = PrecisionArg::F64)]
    precision: PrecisionArg,
    /// Number of worker threads (defaults to all logical cores; tune for EPYC NUMA).
    #[arg(long)]
    threads: Option<usize>,
    /// Cells per parallel chunk (default 16).
    #[arg(long, default_value_t = 16)]
    chunk_size: usize,
    /// Optional path to write `site.nc`-compatible NetCDF-3 output.
    #[arg(long)]
    out_nc: Option<PathBuf>,
    /// Optional path to write full lossless native JSON checkpoint.
    #[arg(long)]
    out_checkpoint: Option<PathBuf>,
    /// Optional limit on number of cells (for quick smoke runs).
    #[arg(long)]
    max_cells: Option<usize>,
    /// Optional single biome ID filter.
    #[arg(long)]
    biome: Option<i32>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a spatial grid simulation (VEMAP or custom grid directory, spinup or transient).
    Run(Box<RunArgs>),

    /// Validate a `century-rs` NetCDF output against an oracle `site.nc` file.
    Validate {
        /// Path to `century-rs` output NetCDF-3 file.
        #[arg(long)]
        rust_out: PathBuf,
        /// Path to reference/oracle `site.nc` file.
        #[arg(long)]
        oracle_out: PathBuf,
        /// Optional maximum relative error threshold for non-zero exit code.
        #[arg(long)]
        max_rel_err: Option<f64>,
    },

    /// Benchmark single-thread vs multi-thread (`rayon`) vs SIMD `f32x8` throughput.
    Bench {
        /// Path to VEMAP dataset directory (runs synthetic grid if omitted).
        #[arg(long)]
        data_dir: Option<PathBuf>,
        /// Optional rock .v2 grid path.
        #[arg(long)]
        rock: Option<PathBuf>,
        /// Optional zminrl .v2 grid path.
        #[arg(long)]
        zminrl: Option<PathBuf>,
        /// Simulation years per cell (default 1000).
        #[arg(long, default_value_t = 1000)]
        years: i32,
        /// Number of threads for parallel run (defaults to all logical cores).
        #[arg(long)]
        threads: Option<usize>,
    },
}

#[allow(clippy::too_many_arguments)]
fn run_grid_typed<F: Float + serde::Serialize + serde::de::DeserializeOwned>(
    data_dir: &PathBuf,
    rock: Option<&PathBuf>,
    zminrl: Option<&PathBuf>,
    start_year: i32,
    years: i32,
    co2_file: Option<&PathBuf>,
    transient_ppt: Option<&PathBuf>,
    transient_tn: Option<&PathBuf>,
    transient_tx: Option<&PathBuf>,
    restart_checkpoint: Option<&PathBuf>,
    profile: CompatibilityProfile,
    threads: Option<usize>,
    chunk_size: usize,
    out_nc: Option<&PathBuf>,
    out_checkpoint: Option<&PathBuf>,
    max_cells: Option<usize>,
    biome_filter: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let t_load = Instant::now();
    let mut grid = load_vemap_grid::<F, _>(
        data_dir,
        rock.map(|p| p.as_path()),
        zminrl.map(|p| p.as_path()),
    )?;
    if let (Some(pp), Some(tn), Some(tx)) = (transient_ppt, transient_tn, transient_tx) {
        let ny = attach_transient_weather(&mut grid, pp, tn, tx)?;
        println!("Attached transient NetCDF weather ({ny} years/cell).");
    }
    if let Some(bf) = biome_filter {
        grid.cells.retain(|c| c.biome_id == bf);
    }
    if let Some(mc) = max_cells {
        grid.cells.truncate(mc);
    }
    for (i, c) in grid.cells.iter_mut().enumerate() {
        c.cell_index = i;
    }

    let co2_vec: Option<Vec<F>> = if let Some(cf) = co2_file {
        let series = Co2Series::load(cf)?;
        let n = years.max(0) as usize;
        let v: Vec<F> = (0..n)
            .map(|i| F::from_f64(series.concentration(start_year + i as i32, 350.0)))
            .collect();
        println!(
            "Loaded CO2 series from {} ({} entries).",
            cf.display(),
            series.by_year.len()
        );
        Some(v)
    } else {
        None
    };

    let ckpts: Option<Vec<NativeCellCheckpoint<F>>> = if let Some(rc) = restart_checkpoint {
        let loaded = read_native_checkpoint::<F, _>(rc)?;
        println!(
            "Loaded {} cell checkpoints from {} for restart.",
            loaded.len(),
            rc.display()
        );
        Some(loaded)
    } else {
        None
    };

    let load_s = t_load.elapsed().as_secs_f64();

    let end_year = start_year + years - 1;
    let cfg = GridRunConfig {
        sim: SimConfig {
            profile,
            elements: ElementMode::CarbonNitrogen,
            start_year,
            end_year,
            ..SimConfig::default()
        },
        num_threads: threads,
        chunk_size,
        group_by_biome: true,
    };

    println!(
        "Loaded {} cells across {} biomes in {:.3}s. Running years {}..={} ({:?}, {})...",
        grid.cells.len(),
        grid.biomes.len(),
        load_s,
        start_year,
        end_year,
        profile,
        std::any::type_name::<F>()
    );

    let (results, stats) =
        run_grid_parallel_with_restart(&grid, &cfg, co2_vec.as_deref(), ckpts.as_deref());
    println!(
        "Completed {} cells x {} years in {:.3}s | {:.1} cell-years/s | {:.2}M cell-months/s",
        stats.ncells,
        stats.nyears,
        stats.elapsed_secs,
        stats.cell_years_per_sec,
        stats.cell_months_per_sec / 1.0e6
    );

    // Summary aggregate metrics across simulated cells.
    if !results.is_empty() {
        let n = results.len() as f64;
        let mean_somsc: f64 = results
            .iter()
            .map(|r| r.state.somsc().to_f64())
            .sum::<f64>()
            / n;
        let mean_npp: f64 = results
            .iter()
            .map(|r| r.last_annual.annual_npp.to_f64())
            .sum::<f64>()
            / n;
        let mean_aet: f64 = results
            .iter()
            .map(|r| r.last_annual.annual_aet.to_f64())
            .sum::<f64>()
            / n;
        println!(
            "Grid means: SOMSC = {:.1} g C/m2 | Annual NPP = {:.1} g C/m2/yr | AET = {:.1} cm/yr",
            mean_somsc, mean_npp, mean_aet
        );
    }

    if let Some(nc_path) = out_nc {
        let pairs: Vec<_> = results
            .iter()
            .map(|r| (r.state.clone(), r.derived.clone()))
            .collect();
        let mut mask = vec![0i32; grid.nrows * grid.ncols];
        for (i, r) in results.iter().enumerate() {
            mask[r.row * grid.ncols + r.col] = (i + 1) as i32;
        }
        write_site_nc(
            nc_path,
            grid.nrows,
            grid.ncols,
            &mask,
            &pairs,
            1,
            &format!("century-rs {:?} {} yrs", profile, years),
        )?;
        println!("Wrote NetCDF-3 output to {}", nc_path.display());
    }

    if let Some(ck_path) = out_checkpoint {
        let ck: Vec<NativeCellCheckpoint<F>> = results
            .into_iter()
            .map(|r| NativeCellCheckpoint {
                row: r.row,
                col: r.col,
                biome_id: r.biome_id,
                state: r.state,
                derived: r.derived,
            })
            .collect();
        write_native_checkpoint(ck_path, &ck)?;
        println!("Wrote full native checkpoint to {}", ck_path.display());
    }

    Ok(())
}

struct VarStats {
    name: &'static str,
    max_abs: f64,
    max_rel: f64,
    p99_rel: f64,
    median_rel: f64,
    rmse: f64,
    mean_rust: f64,
    mean_oracle: f64,
}

fn compute_var_stats(name: &'static str, rust_vals: &[f64], oracle_vals: &[f64]) -> VarStats {
    let n = rust_vals.len().min(oracle_vals.len()).max(1);
    let mut max_abs = 0.0f64;
    let mut sq_sum = 0.0f64;
    let mut rels = Vec::with_capacity(n);
    let mut sum_r = 0.0;
    let mut sum_o = 0.0;
    for i in 0..n {
        let r = rust_vals[i];
        let o = oracle_vals[i];
        sum_r += r;
        sum_o += o;
        let ae = (r - o).abs();
        if ae > max_abs {
            max_abs = ae;
        }
        sq_sum += ae * ae;
        let denom = o.abs().max(1.0e-3);
        rels.push(ae / denom);
    }
    rels.sort_by(|a, b| a.total_cmp(b));
    let max_rel = *rels.last().unwrap_or(&0.0);
    let median_rel = rels[rels.len() / 2];
    let p99_idx = ((rels.len() as f64) * 0.99).floor() as usize;
    let p99_rel = rels[p99_idx.min(rels.len() - 1)];
    VarStats {
        name,
        max_abs,
        max_rel,
        p99_rel,
        median_rel,
        rmse: (sq_sum / n as f64).sqrt(),
        mean_rust: sum_r / n as f64,
        mean_oracle: sum_o / n as f64,
    }
}

fn run_validate(
    rust_out: &PathBuf,
    oracle_out: &PathBuf,
    max_rel_err: Option<f64>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let (r_pools, _r_mask, _r_shape) = read_site_nc(rust_out)?;
    let (o_pools, _o_mask, _o_shape) = read_site_nc(oracle_out)?;
    let n = r_pools.len().min(o_pools.len());
    println!(
        "Validating {} cells: rust={} vs oracle={}",
        n,
        rust_out.display(),
        oracle_out.display()
    );

    type PoolGetter = fn(&SiteNcPools) -> f32;
    let extract = |pools: &[SiteNcPools], f: PoolGetter| -> Vec<f64> {
        pools.iter().take(n).map(|p| f(p) as f64).collect()
    };

    let vars: Vec<(&'static str, PoolGetter)> = vec![
        ("afiel0", |p| p.afiel0),
        ("awilt0", |p| p.awilt0),
        ("som1c_surf", |p| p.som1c_surf),
        ("som1c_soil", |p| p.som1c_soil),
        ("som2c", |p| p.som2c),
        ("som3c", |p| p.som3c),
        ("somsc", |p| p.somsc),
        ("aglivc", |p| p.aglivc),
        ("bglivc", |p| p.bglivc),
        ("rleavc", |p| p.rleavc),
        ("rlwodc", |p| p.rlwodc),
        ("minerl_n0", |p| p.minerl_n0),
        ("asmos0", |p| p.asmos0),
    ];

    println!(
        "{:<12} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Variable", "MeanRust", "MeanOrac", "RMSE", "MaxAbs", "MedRel", "P99Rel", "MaxRel"
    );
    let mut all_pass = true;
    for (name, getter) in vars {
        let st = compute_var_stats(name, &extract(&r_pools, getter), &extract(&o_pools, getter));
        println!(
            "{:<12} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3e} {:>10.3e} {:>10.3e}",
            st.name,
            st.mean_rust,
            st.mean_oracle,
            st.rmse,
            st.max_abs,
            st.median_rel,
            st.p99_rel,
            st.max_rel
        );
        if let Some(thresh) = max_rel_err {
            if st.max_rel > thresh {
                all_pass = false;
            }
        }
    }
    Ok(all_pass)
}

fn run_bench(
    data_dir: Option<&PathBuf>,
    rock: Option<&PathBuf>,
    zminrl: Option<&PathBuf>,
    years: i32,
    threads: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== century-rs Hardware Throughput Benchmark ===");
    if let Some(dir) = data_dir {
        let grid =
            load_vemap_grid::<f32, _>(dir, rock.map(|p| p.as_path()), zminrl.map(|p| p.as_path()))?;
        let cfg_par = GridRunConfig {
            sim: SimConfig {
                profile: CompatibilityProfile::VemapV45Parity,
                elements: ElementMode::CarbonNitrogen,
                start_year: 1,
                end_year: years,
                ..SimConfig::default()
            },
            num_threads: threads,
            chunk_size: 16,
            group_by_biome: true,
        };

        // Run a 128-cell single-thread baseline to measure scaling without waiting minutes.
        let mut sub_grid = grid.clone();
        sub_grid.cells.truncate(128.min(grid.cells.len()));
        for (i, c) in sub_grid.cells.iter_mut().enumerate() {
            c.cell_index = i;
        }
        let (_, st_1t) = run_grid_serial(&sub_grid, &cfg_par, None);
        println!(
            "[1T Serial]   {} cells x {} yrs: {:.3}s | {:.1} cell-yr/s ({:.2}M cell-mo/s)",
            st_1t.ncells,
            st_1t.nyears,
            st_1t.elapsed_secs,
            st_1t.cell_years_per_sec,
            st_1t.cell_months_per_sec / 1.0e6
        );

        let (_, st_par) = run_grid_parallel(&grid, &cfg_par, None);
        let speedup = st_par.cell_years_per_sec / st_1t.cell_years_per_sec.max(1.0e-9);
        println!(
            "[Rayon Par]   {} cells x {} yrs: {:.3}s | {:.1} cell-yr/s ({:.2}M cell-mo/s) | {:.2}x vs 1T",
            st_par.ncells,
            st_par.nyears,
            st_par.elapsed_secs,
            st_par.cell_years_per_sec,
            st_par.cell_months_per_sec / 1.0e6,
            speedup
        );
    }

    // Tier 2 SIMD f32x8 hot SOM kernel throughput benchmark (8 cells x 100,000 years = 800k cell-years).
    let mut batch = SomSoaBatchF32 {
        som1c: [150.0; 8],
        som2c: [2000.0; 8],
        som3c: [1500.0; 8],
        eftext: [0.55; 8],
        p1co2: [0.44; 8],
        fps1s3: [0.011; 8],
        fps2s3: [0.005; 8],
        resp: [0.0; 8],
    };
    let temps = f32x8::from([5.0, 10.0, 15.0, 20.0, 22.0, 25.0, 28.0, 31.0]);
    let relw = f32x8::from([0.3, 0.5, 0.7, 0.9, 1.0, 1.2, 1.4, 1.6]);
    let teff = [15.4f32, 11.75, 29.7];
    let n_substeps = 48 * 100_000usize;
    let t0 = Instant::now();
    for _ in 0..n_substeps {
        let d = batch_defac_f32x8(temps, relw, teff);
        batch_som_decay_f32x8(
            &mut batch,
            d,
            f32x8::splat(1.0),
            7.3,
            0.2,
            0.0045,
            0.55,
            0.55,
            1.0 / 48.0,
        );
    }
    let dt_s = t0.elapsed().as_secs_f64().max(1.0e-9);
    let cy_s = (8.0 * 100_000.0) / dt_s;
    println!(
        "[SIMD f32x8]  8 cells x 100k yrs (1T SOM kernel): {:.3}s | {:.0} cell-yr/s ({:.2}M cell-mo/s) [sink={:.1}]",
        dt_s,
        cy_s,
        (cy_s * 12.0) / 1.0e6,
        batch.resp[0]
    );
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let res: Result<bool, Box<dyn std::error::Error>> = match cli.command {
        Commands::Run(args) => {
            let a = *args;
            let prof: CompatibilityProfile = a.profile.into();
            match a.precision {
                PrecisionArg::F64 => run_grid_typed::<f64>(
                    &a.data_dir,
                    a.rock.as_ref(),
                    a.zminrl.as_ref(),
                    a.start_year,
                    a.years,
                    a.co2_file.as_ref(),
                    a.transient_ppt.as_ref(),
                    a.transient_tn.as_ref(),
                    a.transient_tx.as_ref(),
                    a.restart_checkpoint.as_ref(),
                    prof,
                    a.threads,
                    a.chunk_size,
                    a.out_nc.as_ref(),
                    a.out_checkpoint.as_ref(),
                    a.max_cells,
                    a.biome,
                )
                .map(|()| true),
                PrecisionArg::F32 => run_grid_typed::<f32>(
                    &a.data_dir,
                    a.rock.as_ref(),
                    a.zminrl.as_ref(),
                    a.start_year,
                    a.years,
                    a.co2_file.as_ref(),
                    a.transient_ppt.as_ref(),
                    a.transient_tn.as_ref(),
                    a.transient_tx.as_ref(),
                    a.restart_checkpoint.as_ref(),
                    prof,
                    a.threads,
                    a.chunk_size,
                    a.out_nc.as_ref(),
                    a.out_checkpoint.as_ref(),
                    a.max_cells,
                    a.biome,
                )
                .map(|()| true),
            }
        }
        Commands::Validate {
            rust_out,
            oracle_out,
            max_rel_err,
        } => run_validate(&rust_out, &oracle_out, max_rel_err),
        Commands::Bench {
            data_dir,
            rock,
            zminrl,
            years,
            threads,
        } => run_bench(
            data_dir.as_ref(),
            rock.as_ref(),
            zminrl.as_ref(),
            years,
            threads,
        )
        .map(|()| true),
    };

    match res {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!("Validation threshold exceeded.");
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
