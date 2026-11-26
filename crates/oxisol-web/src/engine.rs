//! Execution bridge connecting HTTP requests to `oxisol-core` and `oxisol-grid`.

use std::path::Path;
use std::time::Instant;

use oxisol_core::{
    initialize_cell, step::ActiveSelection, step_month, CompatibilityProfile, ElementMode, Float,
    IsotopeMode, SimConfig,
};
use oxisol_grid::{run_grid_parallel, GridRunConfig};
use oxisol_io::vemap::{load_vemap_grid, PreloadedGrid};

use crate::models::{
    AnnualPoint, GridCellSummary, GridRunRequest, GridRunResponse, PoolSnapshot, SimulateRequest,
    SimulateResponse,
};
use crate::presets::{
    base_biome_params, build_schedule_from_request, build_synthetic_climate, build_synthetic_grid,
};

fn parse_profile(s: &str) -> CompatibilityProfile {
    match s.to_ascii_lowercase().as_str() {
        "parity" | "vemap" | "vemap_v45_parity" => CompatibilityProfile::VemapV45Parity,
        _ => CompatibilityProfile::ScientificClean,
    }
}

fn parse_elements(s: &str) -> ElementMode {
    match s.to_ascii_lowercase().as_str() {
        "cn" | "1" => ElementMode::CarbonNitrogen,
        "cnp" | "2" => ElementMode::CarbonNitrogenPhosphorus,
        _ => ElementMode::FullCnps,
    }
}

/// Run a single-cell interactive scenario and return annual trajectories plus
/// final biogeochemical pool stocks and mass-conservation audit metrics.
pub fn execute_single_cell(req: &SimulateRequest) -> SimulateResponse {
    if req.precision.eq_ignore_ascii_case("f32") {
        execute_single_cell_typed::<f32>(req, "f32")
    } else {
        execute_single_cell_typed::<f64>(req, "f64")
    }
}

fn execute_single_cell_typed<F: Float>(
    req: &SimulateRequest,
    precision_label: &str,
) -> SimulateResponse {
    let t0 = Instant::now();
    let f = F::from_f64;

    let years = req.years.clamp(5, 1000);
    let profile = parse_profile(&req.profile);
    let elements = parse_elements(&req.elements);

    let mut biome = base_biome_params::<F>(&req.preset_id);
    biome.site_defaults.nelem = elements.nelem();

    // Normalize soil texture fractions cleanly so sand + silt + clay == 1.0
    let s_raw = req.sand.clamp(0.02, 0.96);
    let si_raw = req.silt.clamp(0.02, 0.96);
    let c_raw = req.clay.clamp(0.02, 0.96);
    let tex_sum = s_raw + si_raw + c_raw;
    let sand = s_raw / tex_sum;
    let silt = si_raw / tex_sum;
    let clay = c_raw / tex_sum;

    // Apply custom management rates from sliders
    if let Some(fert) = biome.ferts.get_mut("CUSTOM") {
        fert.feramt = [
            f(req.fertilizer_n.clamp(0.0, 50.0)),
            f(req.fertilizer_p.clamp(0.0, 20.0)),
            F::ZERO,
        ];
    }
    if let Some(graz) = biome.grazs.get_mut("CUSTOM") {
        let g = req.grazing_intensity.clamp(0.0, 0.65);
        graz.flgrem = f(g);
        graz.fdgrem = f(g * 0.5);
    }
    if let Some(omad) = biome.omads.get_mut("COMPOST") {
        omad.astgc = f(req.omad_c.clamp(0.0, 1000.0));
    }

    let lat = req.lat.clamp(-70.0, 75.0);
    let weather = build_synthetic_climate::<F>(
        req.mean_temp_c.clamp(-20.0, 40.0),
        req.temp_amp_c.clamp(0.5, 30.0),
        req.annual_precip_cm.clamp(2.0, 800.0),
        lat,
    );
    let schedule = build_schedule_from_request(req, years);

    let cell_in = oxisol_core::CellSiteInput {
        lat: f(lat),
        lon: f(0.0),
        elevation: F::ZERO,
        sand: f(sand),
        silt: f(silt),
        clay: f(clay),
        bulkd: f(req.bulkd.clamp(0.7, 1.9)),
        rock_frac: f(req.rock_frac.clamp(0.0, 0.85)),
        zminrl_cm: None,
        ph: f(req.ph.clamp(3.8, 9.2)),
        drain: f(req.drain.clamp(0.1, 1.0)),
    };

    let co2_start = req.co2_start_ppm.clamp(180.0, 1500.0);
    let co2_end = req.co2_end_ppm.clamp(180.0, 1500.0);

    let cfg = SimConfig {
        profile,
        elements,
        isotope: IsotopeMode::Unlabeled,
        co2_ppm_override: Some(co2_start),
        audit_mass_balance: req.audit_mass_balance,
        start_year: 1,
        end_year: years,
    };

    let (mut state, derived) = initialize_cell(&biome, &cell_in, &weather, &schedule, &cfg);
    let mut sel = ActiveSelection::from_schedule(&schedule);
    let mut series = Vec::with_capacity(years as usize);
    let mut max_err = 0.0_f64;

    let mut final_npp = 0.0_f64;
    let mut final_resp = 0.0_f64;
    let mut final_aet = 0.0_f64;

    for yr in 1..=years {
        let frac = if years > 1 {
            (yr - 1) as f64 / (years - 1) as f64
        } else {
            0.0
        };
        let co2_yr = co2_start + (co2_end - co2_start) * frac;

        let mut ann_npp = 0.0_f64;
        let mut ann_resp = 0.0_f64;
        let mut ann_aet = 0.0_f64;
        let mut ann_pet = 0.0_f64;

        for (m, w_mo) in weather.iter().enumerate() {
            let (mo, audit) = step_month(
                &mut state,
                &derived,
                &biome,
                &mut sel,
                &schedule,
                w_mo,
                yr,
                (m + 1) as u8,
                f(co2_yr),
                &cfg,
            );
            ann_npp += mo.npp.to_f64();
            ann_resp += mo.resp.to_f64();
            ann_aet += mo.aet.to_f64();
            ann_pet += mo.pet.to_f64();

            if let Some(rep) = audit {
                max_err = max_err.max(rep.carbon_err.to_f64().abs());
                for e in 0..elements.nelem() {
                    max_err = max_err.max(rep.elem_err[e].to_f64().abs());
                }
            }
        }

        final_npp = ann_npp;
        final_resp = ann_resp;
        final_aet = ann_aet;

        series.push(AnnualPoint {
            year: yr,
            som1c: (state.som1c[0].total() + state.som1c[1].total()).to_f64(),
            som2c: state.som2c.total().to_f64(),
            som3c: state.som3c.total().to_f64(),
            somsc: state.somsc().to_f64(),
            aglivc: state.aglivc.total().to_f64(),
            bglivc: state.bglivc.total().to_f64(),
            rlwodc: state.rlwodc.total().to_f64(),
            rleavc: state.rleavc.total().to_f64(),
            litter_c: (state.strucc[0].total()
                + state.strucc[1].total()
                + state.metabc[0].total()
                + state.metabc[1].total())
            .to_f64(),
            npp: ann_npp,
            resp: ann_resp,
            aet: ann_aet,
            pet: ann_pet,
            soil_water_cm: state.asmos[0].to_f64(),
            minerl_n: state.minerl[0][0].to_f64(),
            minerl_p: state.minerl[0][1].to_f64(),
            minerl_s: state.minerl[0][2].to_f64(),
            co2_ppm: co2_yr,
        });
    }

    let elapsed_s = t0.elapsed().as_secs_f64().max(1.0e-9);
    let veg_c = (state.aglivc.total()
        + state.bglivc.total()
        + state.rleavc.total()
        + state.frootc.total()
        + state.fbrchc.total()
        + state.rlwodc.total()
        + state.crootc.total())
    .to_f64();
    let litter_dw_c = (state.strucc[0].total()
        + state.strucc[1].total()
        + state.metabc[0].total()
        + state.metabc[1].total()
        + state.stdedc.total()
        + state.wood1c.total()
        + state.wood2c.total()
        + state.wood3c.total())
    .to_f64();
    let som_n = (state.som1e[0][0] + state.som1e[1][0] + state.som2e[0] + state.som3e[0]).to_f64();

    let mut prof_n = 0.0_f64;
    let mut prof_p = 0.0_f64;
    let mut prof_s = 0.0_f64;
    for l in 0..derived.nlayer {
        prof_n += state.minerl[l][0].to_f64();
        prof_p += state.minerl[l][1].to_f64();
        prof_s += state.minerl[l][2].to_f64();
    }

    SimulateResponse {
        elapsed_ms: elapsed_s * 1000.0,
        throughput_cell_yr_s: (years as f64) / elapsed_s,
        profile: format!("{profile:?}"),
        precision: precision_label.to_string(),
        series,
        final_pools: PoolSnapshot {
            vegetation_c: veg_c,
            litter_and_deadwood_c: litter_dw_c,
            som1_c: (state.som1c[0].total() + state.som1c[1].total()).to_f64(),
            som2_c: state.som2c.total().to_f64(),
            som3_c: state.som3c.total().to_f64(),
            somsc: state.somsc().to_f64(),
            som_n,
            profile_minerl_n: prof_n,
            profile_minerl_p: prof_p,
            profile_minerl_s: prof_s,
            final_npp,
            final_resp,
            final_aet,
            cum_c_in: state.cum_c_in.to_f64(),
            cum_c_out: state.cum_c_out.to_f64(),
            max_mass_balance_err: max_err,
        },
    }
}

/// Run a multi-core spatial grid simulation via `oxisol-grid`'s lock-free
/// `rayon` engine, using either an attached VEMAP dataset or a built-in
/// synthetic continental gradient grid.
pub fn execute_grid_run(
    req: &GridRunRequest,
    attached_data_dir: Option<&Path>,
    attached_rock: Option<&Path>,
    attached_zminrl: Option<&Path>,
) -> Result<GridRunResponse, String> {
    if req.precision.eq_ignore_ascii_case("f32") {
        execute_grid_run_typed::<f32>(req, attached_data_dir, attached_rock, attached_zminrl)
    } else {
        execute_grid_run_typed::<f64>(req, attached_data_dir, attached_rock, attached_zminrl)
    }
}

fn execute_grid_run_typed<F: Float>(
    req: &GridRunRequest,
    attached_data_dir: Option<&Path>,
    attached_rock: Option<&Path>,
    attached_zminrl: Option<&Path>,
) -> Result<GridRunResponse, String> {
    let years = req.years.clamp(5, 1000);
    let profile = parse_profile(&req.profile);
    let max_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let threads_used = req
        .threads
        .map(|t| t.clamp(1, max_cpus))
        .unwrap_or(max_cpus);

    let (grid, source_label): (PreloadedGrid<F>, String) =
        match (req.use_attached_grid, attached_data_dir) {
            (true, Some(dir)) => {
                let g = load_vemap_grid::<F, _>(dir, attached_rock, attached_zminrl)
                    .map_err(|e| format!("Failed to load spatial dataset: {e}"))?;
                let label = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Attached Spatial Grid".to_string());
                (g, format!("VEMAP / Spatial ({label})"))
            }
            _ => {
                let rows = req.rows.clamp(4, 48);
                let cols = req.cols.clamp(4, 64);
                (
                    build_synthetic_grid::<F>(rows, cols, years),
                    format!("Synthetic Continental Grid ({rows}x{cols})"),
                )
            }
        };

    let cfg = GridRunConfig {
        sim: SimConfig {
            profile,
            elements: ElementMode::CarbonNitrogen,
            isotope: IsotopeMode::Unlabeled,
            co2_ppm_override: None,
            audit_mass_balance: false,
            start_year: 1,
            end_year: years,
        },
        num_threads: Some(threads_used),
        chunk_size: req.chunk_size.clamp(1, 256),
        group_by_biome: true,
    };

    let (results, stats) = run_grid_parallel(&grid, &cfg, None);
    let mut sum_somsc = 0.0_f64;
    let mut sum_npp = 0.0_f64;
    let cells: Vec<GridCellSummary> = results
        .iter()
        .map(|r| {
            let somsc = r.state.somsc().to_f64();
            let npp = r.last_annual.annual_npp.to_f64();
            sum_somsc += somsc;
            sum_npp += npp;
            GridCellSummary {
                row: r.row,
                col: r.col,
                biome_id: r.biome_id,
                somsc,
                som3c: r.state.som3c.total().to_f64(),
                rlwodc: r.state.rlwodc.total().to_f64(),
                soil_water_cm: r.state.asmos[0].to_f64(),
                minerl_n: r.state.minerl[0][0].to_f64(),
                npp,
                aet: r.last_annual.annual_aet.to_f64(),
            }
        })
        .collect();

    let n = cells.len().max(1) as f64;
    // Conservative 85% scaling projection from current logical threads to 256-thread high-performance server
    let per_thread_rate = stats.cell_years_per_sec / (threads_used.max(1) as f64);
    let hpc_est = per_thread_rate * 256.0 * 0.85;

    Ok(GridRunResponse {
        source_label,
        rows: grid.nrows,
        cols: grid.ncols,
        ncells: stats.ncells,
        nyears: stats.nyears,
        threads_used,
        elapsed_secs: stats.elapsed_secs,
        cell_years_per_sec: stats.cell_years_per_sec,
        cell_months_per_sec: stats.cell_months_per_sec,
        hpc_256t_est_cell_yr_s: hpc_est,
        mean_somsc: sum_somsc / n,
        mean_npp: sum_npp / n,
        cells,
    })
}
