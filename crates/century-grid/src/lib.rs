//! # `century-grid`
//!
//! Spatial grid execution engine for `century-rs`, engineered for linear multi-core
//! scalability from desktop test hardware (AMD Ryzen 7 5700X, 8C/16T) to production
//! high-core-count **AMD EPYC + 512 GB RAM** servers (64–256+ threads):
//!
//! - **Tier 1 (`runner`)**: Zero-I/O in-memory preload, immutable `Arc<RawBiomeParams>`,
//!   lock-free `rayon` work-stealing over cache-aligned chunks, zero false sharing,
//!   deterministic bit-reproducible output ordering regardless of thread count.
//! - **Tier 2 (`simd_batch`)**: Biome-grouped Struct-of-Arrays (SoA) batching +
//!   portable safe SIMD (`wide::f32x8` / `wide::f64x4`) targeting AVX2+FMA3 and
//!   future AVX-512/NEON with `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod runner;
pub mod simd_batch;

pub use runner::{
    run_grid_parallel, run_grid_parallel_with_restart, run_grid_serial, CellRunResult,
    GridRunConfig, GridRunStats,
};
pub use simd_batch::{
    batch_defac_f32x8, batch_defac_f64x4, batch_rawls_retention_f32x8, batch_som_decay_f32x8,
    group_cells_by_biome, BiomeBatchGroup, SomSoaBatchF32,
};
