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

Full continental US VEMAP spinup (**3,168 cells × 1,000 years = 38.0 million cell-months**) and 99-year historical transient run (**1895–1993** from lossless checkpoint with `pptT3.nc`/`tnT3.nc`/`txT3.nc` + `co2.dat`):

| Engine / Mode | Precision | Threads | Wall Time | Throughput | Speedup vs Legacy |
|---|---:|---:|---:|---:|---:|
| Legacy Fortran/C (`gfortran -O2`) | `f32` | 1 | 154.5 s (extrapolated from 181 cells in 8.83 s) | 20,500 cell-yr/s | 1.00× |
| `century-rs` Serial (`run_grid_serial`) | `f32` | 1 | ~73.3 s (128 cells in 2.96 s) | **43,225 cell-yr/s** | **2.11×** |
| `century-rs` Parallel (`rayon`, `VemapV45Parity`) | `f32` | 16 | **10.92 s** (3,168×1,000 yr) | **290,200 cell-yr/s** (3.48M cell-mo/s) | **14.16×** |
| `century-rs` Parallel (`rayon`, `ScientificClean`) | `f64` | 16 | **11.64 s** (3,168×1,000 yr) | **272,253 cell-yr/s** (3.27M cell-mo/s) | **13.28×** |
| `century-rs` Parallel (`rayon`, `ScientificClean`) | `f32` | 16 | **11.13 s** (3,168×1,000 yr) | **284,715 cell-yr/s** (3.42M cell-mo/s) | **13.89×** |
| `century-rs` Transient (`1895–1993`, `ScientificClean`) | `f64` | 16 | **1.19 s** (3,168×99 yr) | **264,148 cell-yr/s** (3.17M cell-mo/s) | — |
| `century-grid` SIMD `f32x8` SOM kernel | `f32` | 1 | — | **7.88M cell-yr/s** (94.60M cell-mo/s) | — |

### Scaling to AMD EPYC + 512 GB RAM Servers
- Single-threaded `century-rs` outperforms legacy `gfortran -O2` by **2.11×** (`43.2k` vs `20.5k cell-yr/s`) via pre-compiled O(1) biome schedules (`CompiledSchedule`), const-specialized `NELEM` decomposition kernels with direct mass-conserving pool transfers, and cached spinup PET.
- On the 8-physical-core Ryzen 7 5700X, `century-grid` achieves **6.71×** parallel scaling (**83.9% physical-core efficiency**, **14.2×** over legacy) with zero locks and zero per-month heap allocations.
- Because `CellState<f64>` fits in L1d and all grid inputs are preloaded once in read-only memory (`Arc<RawBiomeParams>`), a 128-core / 256-thread AMD EPYC server with 512 GB RAM can hold continental/global 1 km grids (>1M cells) entirely in RAM and sustain near-linear scaling via `--threads <N> --chunk-size 32` (optionally compiled with `RUSTFLAGS="-C target-cpu=native"`).

---

## Validation Summary

1. **Mass Conservation & Edge-Case Invariants (`cargo test --workspace`):** Verified exact C, N, P, and S conservation to `< 1e-10 g/m²` per monthly step across all subsystems, all management events (`CULT` `CULTRA(1..7)`, `FERT`, `IRRI` modes 0–3, `OMAD`, `EROD`, `GRAZ`, `HARV` remaining-fraction `AGLREM`/`BGLREM`, `FIRE`, `TREM` `RETF(part, elem)`), C-14 isotope propagation, sub-zero snowmelt above `tmelt[0]`, hemisphere-aware once-per-autumn deciduous leaf abscission (Northern & Southern Hemispheres), `CompiledSchedule` exact equivalence, non-finite transfer rejection, hyper-arid zero-precip deserts, continuous permafrost, extreme 100% sand/clay/90% rock/1-layer soils, and `FlowLedger` >300-flow overflow bursts (`tests/oracle_tier_b.rs`).
2. **Tier A Continental VEMAP Comparison (3,168 cells × 1,000 years, `VemapV45Parity` vs archived VEMAP `site.nc`):**
   - Hydraulic pedotransfer (`afiel0`, `awilt0`): exact to single-precision rock-grid rounding (`MedRel <= 7.95e-8`, `MaxRel <= 8.08e-5` across all 3,168 cells).
   - Soil water (`asmos0`): `3.436` vs `3.397 cm` (**median rel diff `1.52%`**, `RMSE = 0.461 cm`, zero cells below wilting point).
   - Soil organic carbon (**all 17/17 biomes within `0.1%–22.4%` median error**): total SOM (`somsc`) **`4,134.9` vs `4,231.8 g C/m²`** (`-2.29%` continental mean diff, **median rel diff `6.08%`**, **`RMSE = 362.5 g C/m²`**, **`P99 = 28.6%`**), passive SOM (`som3c` **`1,635.3` vs `1,636.9`**, **median rel diff `5.76%`**, `RMSE = 115.1`), slow SOM (`som2c` `2,380` vs `2,475`, **median rel diff `7.40%`**, `RMSE = 248.1`), active soil SOM (`som1c_soil` **`119.5` vs `120.0`**, median `11.25%`, `RMSE = 16.6`).
   - Vegetation & mineral N: large wood (`rlwodc` `3,806` vs `3,657 g C/m²`, median `16.0%`, `RMSE = 1,236`), surface mineral N (`minerl_n0` **`0.609` vs `0.616 g N/m²`**, **median rel diff `16.9%`**, `RMSE = 0.179`), belowground roots (`bglivc` `168` vs `197 g C/m²`, median `27.4%`), and foliage (`rleavc` `45.8` vs `63.2 g C/m²`, median `30.1%`).
   - `f32` vs `f64` 1,000-year numerical drift (`ScientificClean`): `somsc` median rel drift **`2.42e-5`** (`P99 = 5.06e-4`), `rlwodc` median rel drift **`1.11e-5`**, `asmos0` median rel drift **`1.15e-7`**.
   - *Note:* Because `century-rs` is a strict clean-room implementation from published documentation (not a line-by-line Fortran port), individual cell-level equilibrium values exhibit expected structural differences from the 1999 CSU binary while achieving tight continental and biome-level biogeochemical parity. See `docs/spec/07_compatibility_profiles.md` for documented V4.0 → V4.5 deltas.
3. **Tier B Head-to-Head Perturbation Probes:** Verified **5/5** directional agreement against the local black-box oracle on an isolated VEMAP cell (`CULT P`, `FERT+OMAD`, `IRRI A50`, `FIRE H`, `DRAIN=0.2`).

---

## Quick Start

```bash
# Build and run all unit, mass-balance, edge-case, and Tier B coverage tests
cargo test --workspace

# Run a 1,000-year spatial grid spinup (default: ScientificClean f64)
cargo run --release -p century-cli -- run \
  --data-dir /path/to/vemap_data \
  --years 1000 \
  --profile clean \
  --precision f64 \
  --out-nc output_site.nc \
  --out-checkpoint spinup_checkpoint.json

# Run a 99-year transient simulation (1895-1993) from a spinup checkpoint with NetCDF weather & CO2
cargo run --release -p century-cli -- run \
  --data-dir /path/to/vemap_data \
  --start-year 1895 \
  --years 99 \
  --co2-file /path/to/co2.dat \
  --transient-ppt /path/to/pptT3.nc \
  --transient-tn /path/to/tnT3.nc \
  --transient-tx /path/to/txT3.nc \
  --restart-checkpoint spinup_checkpoint.json \
  --profile clean \
  --precision f64 \
  --out-nc transient_1895_1993.nc

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
