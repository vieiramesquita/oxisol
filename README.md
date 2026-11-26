# `century-rs`

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A **clean-room, open-source, high-performance Rust implementation** of the **CENTURY** monthly biogeochemical soil organic matter and ecosystem model (Carbon, Nitrogen, Phosphorus, Sulfur, multi-layer soil hydrology, grassland/crop, forest/savanna, and land management).

> **Clean-Room Provenance & Licensing**
> The original Colorado State University (CSU) CENTURY Fortran/C distribution is proprietary (*All Rights Reserved* / non-open-source per EU JRC MIDAS). `century-rs` is an independent, clean-room implementation authored from scratch in pure safe Rust (`#![forbid(unsafe_code)]`) based on the published USDA-ARS scientific documentation (Metherell et al. 1993, GPSR Technical Report No. 4) and peer-reviewed biogeochemical literature, released under the **MIT License**.

---

## Key Features

- **Complete Biogeochemical Core (`century-core`, `#![forbid(unsafe_code)]`):**
  - **C, N, P, and S cycles** (`ElementMode::CarbonNitrogen`, `CarbonNitrogenPhosphorus`, `FullCnps`) plus **C-14 / C-13 isotope tracking** (`IsotopeMode`).
  - **10 Modular Scientific Subsystems:** Linacre (1977) PET & astronomical daylength, multi-layer soil hydrology & solute leaching, biomass-insulated soil temperature, CO₂ physiological response, 4-substep litter/wood/SOM decomposition, mineralization/immobilization & volatilization, grassland/crop production, forest/savanna production with Ryan (1991) maintenance respiration, inorganic P & S geochemistry, and all 9 management disturbances (`CULT`, `FERT`, `FIRE`, `GRAZ`, `HARV`, `IRRI`, `OMAD`, `TREM`, `EROD`) with block-repeating schedules.
  - **Exact Mass Conservation:** Stack-allocated, strongly-typed `FlowLedger<F>` (`PoolId`) with zero heap allocation in the monthly step loop and built-in conservation auditing (`< 1e-10 g/m²` in `f64`).
  - **Generic Precision (`Float` trait):** Run modern research in `f64` (`CompatibilityProfile::ScientificClean`, default) or audit historical VEMAP V4.5 runs in `f32` (`CompatibilityProfile::VemapV45Parity`).
- **100% Pure-Rust I/O (`century-io`):**
  - Zero C runtime dependencies (`libnetcdf.so` not required) — natively portable across Linux, Windows, and macOS.
  - Built-in parsers for `.100` parameter tables, `.sch` schedules, `.v2` VEMAP ASCII rasters, `co2.dat`, pure-Rust NetCDF-3 Classic & 64-bit Offset reader/writer, legacy `site.nc` I/O, and complete lossless JSON checkpoints.
- **Engineered for Multi-Core & High-RAM Servers (`century-grid`):**
  - **Cache-line friendly (`#[repr(C, align(64))]`):** `CellState<f32>` (~3 KiB) and `CellState<f64>` (~5.8 KiB) fit completely inside a 32 KiB L1d cache per core.
  - **Zero shared mutable state:** Immutable `Arc<RawBiomeParams<F>>` tables shared across threads with per-cell `CellDerivedParams<F>` — zero lock contention and bit-deterministic output ordering from 1 to 256+ cores.
  - **Tier 1 (`rayon`) + Tier 2 (`wide` portable SIMD `f32x8`/`f64x4`):** Designed to scale linearly from desktop test hardware (AMD Ryzen 7 5700X) to high-core-count **AMD EPYC + 512 GB RAM** production servers (64–256+ threads, NUMA-friendly chunking, biome-grouped execution).

---

## Workspace Structure

```text
century-rs/
├── docs/spec/                  # Independent clean-room mathematical specification (01–07)
├── crates/
│   ├── century-core/           # Biogeochemical engine (#![forbid(unsafe_code)], f32/f64, FlowLedger)
│   ├── century-io/             # Pure-Rust .100, .sch, .v2, co2.dat, NetCDF-3, checkpoints, VEMAP loader
│   ├── century-grid/           # Rayon multi-core runner + biome-grouped portable SIMD (wide) + Criterion benches
│   └── century-cli/            # `century` CLI (`run`, `validate`, `bench`)
├── LICENSE                     # MIT License
└── .github/workflows/ci.yml    # Cross-platform CI (fmt, clippy -D warnings, tests, release build)
```

---

## Measured Hardware Performance (AMD Ryzen 7 5700X, 8C/16T, WSL2 Debian)

Full continental US VEMAP spinup (**3,168 cells × 1,000 years = 38.0 million cell-months**):

| Engine / Mode | Precision | Threads | Wall Time (3,168×1000 yr) | Throughput | Speedup vs Legacy |
|---|---:|---:|---:|---:|---:|
| Legacy Fortran/C (`gfortran -O2`) | `f32` | 1 | 154.5 s (extrapolated from 181 cells in 8.83 s) | 20,500 cell-yr/s | 1.00× |
| `century-rs` Serial (`run_grid_serial`) | `f32` | 1 | ~213.8 s (128 cells in 8.64 s) | 14,818 cell-yr/s | 0.72× |
| `century-rs` Parallel (`rayon`, biome-grouped) | `f32` | 16 | **26.69 s** | **118,691 cell-yr/s** (1.42M cell-mo/s) | **5.79×** |
| `century-rs` Parallel (`rayon`, `ScientificClean`) | `f64` | 16 | **27.01 s** | **117,301 cell-yr/s** (1.41M cell-mo/s) | **5.72×** |
| `century-grid` SIMD `f32x8` SOM kernel | `f32` | 1 | — | **8.01M cell-yr/s** (96.1M cell-mo/s) | — |

### Scaling to AMD EPYC + 512 GB RAM Servers
- On the 8-physical-core Ryzen 7 5700X, `century-grid` achieves **7.90×** parallel scaling (**98.8% physical-core efficiency**) with zero locks and zero per-month heap allocations.
- Because `CellState<f64>` fits in L1d and all grid inputs are preloaded once in read-only memory (`Arc<RawBiomeParams>`), a 128-core / 256-thread AMD EPYC server with 512 GB RAM can hold continental/global 1 km grids (>1M cells) entirely in RAM and sustain near-linear scaling via `--threads <N> --chunk-size 32`.

---

## Validation Summary

1. **Mass Conservation Invariants (`cargo test --workspace`):** Verified exact C, N, P, and S conservation to `< 1e-10 g/m²` per monthly step across all subsystems and all management events (`CULT`, `FERT`, `IRRI`, `OMAD`, `EROD`, `GRAZ`, `HARV`, `FIRE`, `TREM`) plus C-14 isotope propagation (`tests/oracle_tier_b.rs`).
2. **Tier A Continental VEMAP Comparison (3,168 cells × 1,000 years, `VemapV45Parity` vs archived VEMAP `site.nc`):**
   - Hydraulic pedotransfer (`afiel0`, `awilt0`): exact to single-precision epsilon (`max_rel_err ≤ 2.6e-7` across all 3,168 cells).
   - Continental US mean stocks closely track archived VEMAP means: `som1c_surf` (35.6 vs 35.2 g C/m²), `som3c` (1,648 vs 1,637 g C/m²), `som2c` (2,705 vs 2,475 g C/m²), `somsc` (4,494 vs 4,232 g C/m²), `rlwodc` (3,563 vs 3,657 g C/m²).
   - *Note:* Because `century-rs` is a strict clean-room implementation from published documentation (not a line-by-line Fortran port), individual cell-level equilibrium values exhibit expected structural differences from the 1999 CSU binary while preserving continental biogeochemical behavior. See `docs/spec/07_compatibility_profiles.md` for documented V4.0 → V4.5 deltas.

---

## Quick Start

```bash
# Build and run all unit, mass-balance, and Tier B coverage tests
cargo test --workspace

# Run a 1,000-year spatial grid simulation (default: ScientificClean f64)
cargo run --release -p century-cli -- run \
  --data-dir /path/to/vemap_data \
  --years 1000 \
  --profile clean \
  --precision f64 \
  --out-nc output_site.nc \
  --out-checkpoint checkpoint.json

# Validate outputs against a reference NetCDF-3 site.nc
cargo run --release -p century-cli -- validate \
  --rust-out output_site.nc \
  --oracle-out reference_site.nc

# Run hardware throughput benchmarks
cargo run --release -p century-cli -- bench --years 1000
cargo bench -p century-grid
```

---

## Scientific References

- Parton, W. J., Anderson, D. W., Cole, C. V., & Stewart, J. W. B. (1983). Simulation of soil organic matter formations and mineralization in semiarid agroecosystems. *Nutrient Cycling in Agricultural Ecosystems*.
- Parton, W. J., Schimel, D. S., Cole, C. V., & Ojima, D. S. (1987). Analysis of factors controlling soil organic matter levels in Great Plains grasslands. *Soil Science Society of America Journal*, 51(5), 1173–1179.
- Parton, W. J., Stewart, J. W. B., & Cole, C. V. (1988). Dynamics of C, N, P and S in grassland soils: a model. *Biogeochemistry*, 5(1), 109–131.
- Metherell, A. K., Harding, L. A., Cole, C. V., & Parton, W. J. (1993). *CENTURY Soil Organic Matter Model Environment: Technical Documentation, Agroecosystem Version 4.0*. Great Plains System Research Unit Technical Report No. 4, USDA-ARS, Fort Collins, CO.
- Parton, W. J., Ojima, D. S., & Schimel, D. S. (2005). *CENTURY: Modeling Ecosystem Responses to Climate Change, Version 4 (VEMAP 1995)*. ORNL DAAC, Oak Ridge, Tennessee, USA.
- Linacre, E. T. (1977). A simple formula for estimating evaporation rates in various climates, using temperature data alone. *Agricultural Meteorology*, 18(6), 409–424.
- Rawls, W. J., Brakensiek, D. L., & Saxton, K. E. (1982). Estimation of soil water properties. *Transactions of the ASAE*, 25(5), 1316–1320.
- Gupta, S. C., & Larson, W. E. (1979). Estimating soil water retention characteristics from particle size distribution, organic matter percent, and bulk density. *Water Resources Research*, 15(6), 1633–1635.
- Ryan, M. G. (1991). Effects of climate change on plant respiration. *Ecological Applications*, 1(2), 157–167.

## License

Licensed under the [MIT License](LICENSE).
