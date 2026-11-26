use std::collections::HashMap;
use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oxisol_core::{
    CellSiteInput, CompatibilityProfile, CropParams, ElementMode, FixParams, MonthlyWeather,
    RawBiomeParams, Schedule, SimConfig, SiteParams, TreeParams,
};
use oxisol_grid::{
    batch_defac_f32x8, batch_som_decay_f32x8, run_grid_parallel, run_grid_serial, GridRunConfig,
    SomSoaBatchF32,
};
use oxisol_io::vemap::{PreloadedCell, PreloadedGrid};
use wide::f32x8;

fn synthetic_grid(ncells: usize) -> PreloadedGrid<f32> {
    let mut biome = RawBiomeParams::<f32> {
        biome_id: 3,
        fix: FixParams::default(),
        site_defaults: SiteParams::default(),
        ..RawBiomeParams::default()
    };
    biome.crops.insert("TMC3".into(), CropParams::default());
    biome.trees.insert("MTCF".into(), TreeParams::default());

    let mut biomes = HashMap::new();
    biomes.insert(3, Arc::new(biome));
    let mut schedules = HashMap::new();
    schedules.insert(3, Arc::new(Schedule::default()));

    let weather = core::array::from_fn(|m| MonthlyWeather {
        precip_cm: 6.0 + (m as f32 * 0.5).sin() * 2.0,
        tmin_c: 5.0 + (m as f32 * 0.52).sin() * 10.0,
        tmax_c: 15.0 + (m as f32 * 0.52).sin() * 10.0,
    });
    let site_in = CellSiteInput::from_site(&SiteParams::default());

    let cells = (0..ncells)
        .map(|i| PreloadedCell {
            row: i / 115,
            col: i % 115,
            cell_index: i,
            transient_cell_id: None,
            biome_id: 3,
            site_input: site_in.clone(),
            mean_weather: weather,
            transient_weather: None,
        })
        .collect();

    PreloadedGrid {
        nrows: ncells.div_ceil(115),
        ncols: 115,
        mask: vec![1; ncells],
        biomes,
        schedules,
        cells,
    }
}

fn bench_grid_scaling(c: &mut Criterion) {
    let grid = synthetic_grid(256);
    let cfg = GridRunConfig {
        sim: SimConfig {
            profile: CompatibilityProfile::VemapV45Parity,
            elements: ElementMode::CarbonNitrogen,
            start_year: 1,
            end_year: 50,
            ..SimConfig::default()
        },
        num_threads: None,
        chunk_size: 16,
        group_by_biome: true,
    };

    let mut group = c.benchmark_group("grid_256cells_50yrs");
    group.sample_size(10);
    group.bench_function("serial_1t", |b| {
        b.iter(|| black_box(run_grid_serial(&grid, &cfg, None)));
    });
    group.bench_function("rayon_all_threads", |b| {
        b.iter(|| black_box(run_grid_parallel(&grid, &cfg, None)));
    });
    group.finish();
}

fn bench_simd_kernels(c: &mut Criterion) {
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
    let teff = [15.4, 11.75, 29.7];

    c.bench_function("simd_f32x8_som_1000yr_8cells", |b| {
        b.iter(|| {
            let mut local = batch;
            for _ in 0..48000 {
                let d = batch_defac_f32x8(temps, relw, teff);
                batch_som_decay_f32x8(
                    &mut local,
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
            batch = black_box(local);
        });
    });
}

criterion_group!(benches, bench_grid_scaling, bench_simd_kernels);
criterion_main!(benches);
