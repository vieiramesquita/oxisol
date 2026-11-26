//! # `oxisol-core`
//!
//! Clean-room, zero-`unsafe`, high-performance Rust implementation of the
//! CENTURY monthly biogeochemical ecosystem model (Carbon, Nitrogen, Phosphorus,
//! Sulfur, soil water balance, grassland/crop, forest/savanna, and land
//! management).
//!
//! ## Architecture Highlights
//! - **`#![forbid(unsafe_code)]`** throughout the scientific core.
//! - **Generic precision (`Float` trait)**: run modern science in IEEE-754 `f64`
//!   (`CompatibilityProfile::ScientificClean`) or audit numerical parity against
//!   historical single-precision datasets in `f32`
//!   (`CompatibilityProfile::VemapV45Parity`).
//! - **Zero global mutable state**: every cell owns a cache-line-aligned
//!   [`CellState`] (~3 KiB in `f32`, ~5.8 KiB in `f64`, fitting inside 32 KiB L1d)
//!   and per-cell [`CellDerivedParams`], while biome tables live in immutable
//!   `Arc<RawBiomeParams>` for contention-free scaling from desktop Ryzen CPUs
//!   to high-core-count modern high-performance servers (128–256+ threads).
//! - **Exact mass conservation**: all pool transfers flow through a stack-allocated
//!   [`FlowLedger`] with strongly-typed [`PoolId`]s and optional per-step
//!   conservation auditing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::needless_range_loop)]

pub mod config;
pub mod float;
pub mod ledger;
pub mod params;
pub mod state;
pub mod step;
pub mod subsystems;

pub use config::{CompatibilityProfile, ElementMode, IsotopeMode, SimConfig, SystemKind};
pub use float::Float;
pub use ledger::{Flow, FlowLedger, MassBalanceReport, PoolId};
pub use params::{
    CellDerivedParams, CellSiteInput, CropParams, CultParams, FertParams, FireParams, FixParams,
    GrazParams, HarvParams, IrriParams, OmadParams, RawBiomeParams, SiteParams, TreeParams,
    TremParams,
};
pub use state::{AnnualOutput, CellState, MonthlyOutput, MonthlyWeather};
pub use step::{
    initialize_cell, run_cell_from_state, run_cell_from_state_compiled, run_cell_simulation,
    run_cell_simulation_compiled, step_month, step_year,
};
pub use subsystems::scheduler::{
    CompiledBlock, CompiledSchedule, EventKind, ResolvedMonthEvents, Schedule, ScheduleBlock,
    ScheduledEvent,
};
