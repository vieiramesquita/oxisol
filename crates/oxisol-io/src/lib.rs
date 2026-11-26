//! # `oxisol-io`
//!
//! 100% pure-Rust I/O for `Oxisol`:
//! - CENTURY `.100` parameter files (`fix`, `site`, `crop`, `tree`, and all management tables)
//! - CENTURY `.sch` multi-block schedule files
//! - VEMAP `.v2` ASCII spatial grids and `co2.dat` atmospheric series
//! - Pure-Rust NetCDF-3 Classic (`CDF\x01`) & 64-bit Offset (`CDF\x02`) reader/writer
//! - Complete native checkpoints (JSON) and legacy 235-column `site.nc` reader/writer

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ascii_v2;
pub mod checkpoint;
pub mod co2_dat;
pub mod error;
pub mod netcdf3;
pub mod params_100;
pub mod schedule_sch;
pub mod vemap;

pub use error::{IoError, Result};
