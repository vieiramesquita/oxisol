# Oxisol

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Safe Rust](https://img.shields.io/badge/unsafe-forbidden-brightgreen.svg)](crates/oxisol-core/src/lib.rs)
[![Docs](https://img.shields.io/badge/Docs-vieiramesquita.github.io%2Foxisol-06b6d4.svg)](https://vieiramesquita.github.io/oxisol/)

**[Documentation Portal (`mdBook`)](https://vieiramesquita.github.io/oxisol/) · [Rust API Reference (`rustdoc`)](https://vieiramesquita.github.io/oxisol/api/oxisol_core/index.html) · [Benchmarks & Parity Report](BENCHMARKS.md)**

---

## The Story Behind Oxisol

Soils are the largest terrestrial reservoir of organic carbon on Earth — storing more carbon than the atmosphere and all living vegetation combined ([Scharlemann et al., 2014](https://doi.org/10.4155/cmt.13.77)). Small shifts in how soil organic matter decomposes or accumulates under warming, changing rainfall, deforestation, or agricultural management can tip regional ecosystems from carbon sinks into carbon sources.

In the late 1980s, William J. Parton and colleagues at Colorado State University set out to understand what governs soil organic matter across the North American Great Plains. Their framework — the **CENTURY** model ([Parton et al., 1987](https://doi.org/10.2136/sssaj1987.03615995005100050015x); [Parton et al., 1988](https://doi.org/10.1007/BF02180320); [Parton et al., 1993](https://doi.org/10.1029/93GB02042); Metherell et al., 1993) — partitioned soil carbon and nutrients into *active*, *slow*, and *passive* pools turning over on annual, decadal, and centennial timescales. Over the following decades, CENTURY became one of the most influential biogeochemical models in Earth system science, powering landmark continental intercomparisons such as **VEMAP** ([VEMAP Members, 1995](https://doi.org/10.1029/95GB02746); [Parton et al., 2005](https://doi.org/10.3334/ORNLDAAC/820)) and informing high-resolution continental policy inventories such as the European Commission's 1 km² agricultural soil carbon assessments ([EC JRC MIDAS](https://web.jrc.ec.europa.eu/policy-model-inventory/explore/models/model-century/)).

Yet nearly forty years later, a strange paradox remained: **the science had always been published openly in peer-reviewed journals, while the reference code stayed locked behind a proprietary, non-free 1990s Fortran 77/C distribution** (*All Rights Reserved* / non-open-source per [EC JRC MIDAS](https://web.jrc.ec.europa.eu/policy-model-inventory/explore/models/model-century/)), tied to single-threaded execution, mutable global `COMMON` blocks, Northern-Hemisphere phenology assumptions, and legacy numerical quirks.

**Foundational climate and soil carbon science should not stay trapped inside closed-source 1990s code.**

That is why **Oxisol** was created. Named after **Oxisols** (*Latossolos*) — the deeply weathered, iron-oxide-rich tropical soils of the Amazon and Cerrado whose **rust** hue mirrors the **Rust** language — **Oxisol** is an **open-source, clean-room, MIT-licensed *CENTURY-like* biogeochemical ecosystem engine** built from first principles of the published scientific literature:

- **100% Safe, Pure Rust (`#![forbid(unsafe_code)]`):** Zero C/Fortran runtime dependencies, deterministic memory-safe parallelism, and strict `< 1e-10 g/m²` C/N/P/S mass-conservation auditing in `f64`.
- **Scientifically Validated at Continental Scale:** Reproduces archived VEMAP CENTURY V4.5 equilibrium soil organic carbon across **3,168 cells × 1,000 years** at **$R^2 = 0.9756$** ($r = 0.9877$, median relative difference **6.08%**) — while fixing legacy bugs and adding native **Southern Hemisphere** deciduous phenology.
- **Engineered for Modern Servers & Interactive Science:** Runs **2.17× faster than `gfortran -O2` on a single core**, **14.22× faster on 8C/16T** (a full 38-million cell-month continental millennium in **10.87 seconds**), and ships with a zero-dependency embedded web workbench (**Oxisol Studio**) and REST API in a single binary.

---

## Key Highlights

- **Complete CENTURY-like Biogeochemical Core (`oxisol-core`, `#![forbid(unsafe_code)]`):**
  - Full **Carbon, Nitrogen, Phosphorus, and Sulfur** cycling (`ElementMode::CarbonNitrogen`, `CarbonNitrogenPhosphorus`, `FullCnps`) plus **C-14 / C-13 isotope tracking** (`IsotopeMode`).
  - **10 Modular Subsystems:** Linacre (1977) PET & astronomical daylength, multi-layer soil hydrology & solute leaching, biomass/snow-insulated soil temperature, CO₂ physiological response, 4-substep litter/wood/SOM1–3 decomposition, mineralization/volatilization, grassland/crop growth, forest/savanna dynamics with **Northern & Southern Hemisphere deciduous phenology**, inorganic P & S geochemistry, and all **9 management disturbances** (`CULT`, `FERT`, `FIRE`, `GRAZ`, `HARV`, `IRRI`, `OMAD`, `TREM`, `EROD`) with $O(1)$ pre-compiled schedules.
  - **Exact Mass Conservation:** Stack-allocated `FlowLedger<F>` (with automatic overflow spill) and `< 1e-10 g/m²` C/N/P/S conservation auditing in `f64`.
  - **Explicit Profiles:** `CompatibilityProfile::ScientificClean` (`f64`, default) and `CompatibilityProfile::VemapV45Parity` (`f32`, historical VEMAP V4.5 audit mode).
- **Single-Binary Embedded Web UI & REST API (`oxisol-web` / `oxisol serve`):**
  - **Oxisol Studio** ships inside the compiled `oxisol` binary with **zero Node.js or CDN runtime dependencies** — run locally or on a headless high-performance server over an SSH tunnel.
  - Bilingual **EN / PT-BR** interface featuring the **Scenario Lab** (7 calibrated global presets incl. Southern Hemisphere Cerrado, A/B Baseline pinning with live `Δ` badges, interactive SVG charts, live pool flow diagram, CSV/JSON export) and **Spatial Grid & HPC Explorer** (live multi-core `rayon` execution, 2D interactive heatmap canvas, click-to-inspect cells, high-performance server throughput projection).
- **100% Pure-Rust I/O (`oxisol-io`):**
  - Native parsers for `.100`, `.sch`, `.v2` ASCII rasters, `co2.dat`, pure-Rust NetCDF-3 Classic & 64-bit Offset reader/writer, `site.nc` I/O, and complete lossless JSON checkpoints.
- **Built for POWERFUL Servers (`oxisol-grid`):**
  - 64-byte cache-line-aligned `CellState<f64>` (~5.8 KiB, L1d-resident), zero shared mutable state (`Arc<RawBiomeParams>`), lock-free `rayon` chunked parallelism, and portable SIMD (`wide::f32x8` / `f64x4`).

---

## Workspace Layout

```text
oxisol/
├── crates/
│   ├── oxisol-core/           # Biogeochemical engine (#![forbid(unsafe_code)], f32/f64, FlowLedger)
│   ├── oxisol-io/             # Pure-Rust .100, .sch, .v2, co2.dat, NetCDF-3, checkpoints, VEMAP loader
│   ├── oxisol-grid/           # Rayon multi-core runner + biome-grouped portable SIMD + Criterion benches
│   ├── oxisol-web/            # Axum REST API + embedded Oxisol Studio SPA (EN / PT-BR)
│   └── oxisol-cli/            # `oxisol` CLI (`serve`, `run`, `validate`, `bench`)
├── docs/
│   ├── book.toml              # mdBook documentation portal (MathJax enabled)
│   ├── src/                   # Full User Guide, Scientific Reference, HPC & REST API docs
│   └── spec/                  # Formal 7-part clean-room mathematical specification
├── LICENSE                    # MIT License
└── .github/workflows/         # Cross-platform CI (`ci.yml`) + GitHub Pages (`docs.yml`)
```

---

## Oxisol vs. Legacy CENTURY — Benchmarks & Scientific Parity

> **Full report & 17-biome breakdown:** see [`BENCHMARKS.md`](BENCHMARKS.md).

![Oxisol vs. Legacy CENTURY Throughput Benchmark](docs/assets/benchmark_throughput.svg)

Full continental US VEMAP spinup (**3,168 cells × 1,000 years = 38.0 million cell-months**) and 99-year historical transient run (**1895–1993** from lossless checkpoint with `pptT3.nc`/`tnT3.nc`/`txT3.nc` + `co2.dat`):

| Engine / Mode | Precision | Threads | Wall Time | Throughput | Speedup vs. Legacy |
|---|---:|---:|---:|---:|---:|
| Legacy 1990s Fortran/C (`gfortran -O2`) | `f32` | 1 | 154.5 s *(extrapolated)* | 20,500 cell-yr/s | 1.00× |
| `oxisol` Serial (`run_grid_serial`) | `f32` | 1 | ~71.2 s | **44,495 cell-yr/s** | **2.17×** |
| `oxisol` Parallel (`rayon`, `VemapV45Parity`) | `f32` | 16 | **10.87 s** | **291,420 cell-yr/s** (`3.50M cell-mo/s`) | **14.22×** |
| `oxisol` Parallel (`rayon`, `ScientificClean`) | `f64` | 16 | **12.34 s** | **256,840 cell-yr/s** (`3.08M cell-mo/s`) | **12.53×** |
| `oxisol` Transient (`1895–1993`, 99 yr) | `f64` | 16 | **1.19 s** | **264,148 cell-yr/s** (`3.17M cell-mo/s`) | — |
| `oxisol-grid` SIMD `f32x8` SOM kernel | `f32` | 1 | — | **8.88M cell-yr/s** (`106.59M cell-mo/s`) | — |

On production servers, compile with `RUSTFLAGS="-C target-cpu=native"` and pass `--threads <N> --chunk-size 32`.

---

## Continental Validation Summary (`R² = 0.9756` across 3,168 Cells × 1,000 yr)

![Oxisol vs. CENTURY V4.5 Scientific Parity Across 17 Biomes](docs/assets/benchmark_parity.svg)

- **Tier A (3,168 cells × 1,000 yr vs. archived CENTURY V4.5 `site.nc`):**
  - **Total SOM (`somsc`):** **`4,134.9` vs. `4,231.8 g C/m²`** (**$r = 0.9877$, $R^2 = 0.9756$**, `-2.29%` mean diff, median rel diff **`6.08%`**)
  - **Slow SOM (`som2c`):** **`2,380.1` vs. `2,474.9 g C/m²`** (**$r = 0.9889$, $R^2 = 0.9779$**)
  - **Passive SOM (`som3c`):** **`1,635.3` vs. `1,636.9 g C/m²`** (**$r = 0.9864$, $R^2 = 0.9731$**, **`-0.09%`** mean diff)
  - **Active SOM (`som1c_soil`):** **`119.5` vs. `120.0 g C/m²`** (**$r = 0.9763$, $R^2 = 0.9531$**, **`-0.38%`** mean diff)
  - **Forest Wood (`rlwodc`):** **`3,806.0` vs. `3,656.9 g C/m²`** (**$r = 0.9786$, $R^2 = 0.9576$**)
  - **Surface Mineral N & Soil Water:** `minerl_n0` **`0.609` vs. `0.616 g N/m²`** (`-1.14%`), `asmos0` **`3.436` vs. `3.397 cm`** (**`1.52%`** median diff).
- **Tier B Perturbations & Invariants:** **5/5** directional agreement vs. black-box oracle (`CULT P`, `FERT+OMAD`, `IRRI A50`, `FIRE H`, `DRAIN=0.2`), plus strict `< 1e-10 g/m²` C/N/P/S mass conservation across all 9 disturbances, P/S geochemistry, C-14 isotopes, and extreme climates/soils.
- See [`BENCHMARKS.md`](BENCHMARKS.md) for the full 17-biome table and the list of legacy 1990s bugs eliminated in `ScientificClean`.

---

## Quick Start

```bash
# 1. Launch Oxisol Studio (embedded Web UI + REST API, EN / PT-BR)
cargo run --release -p oxisol-cli -- serve --port 8080
# Open http://127.0.0.1:8080

# 2. Run all unit, mass-balance, Tier B, SIMD, I/O, and REST API tests
cargo test --workspace

# 3. Run a spatial grid spinup (1,000 years, ScientificClean f64)
cargo run --release -p oxisol-cli -- run \
  --data-dir /path/to/vemap_data \
  --years 1000 \
  --out-nc output_site.nc \
  --out-checkpoint spinup_checkpoint.json

# 4. Run hardware throughput benchmarks
cargo run --release -p oxisol-cli -- bench --years 1000

# 5. Build full documentation (mdBook + Rustdoc)
mdbook serve docs --open
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --open
```

---

## Authorship

- **Author:** [Vinícius Mesquita](https://github.com/vieiramesquita) (`@vieiramesquita`)
- **AI Co-Author:** Antigravity (Google DeepMind)

---

## Scientific References & License

Oxisol is an independent, clean-room implementation grounded in the following peer-reviewed literature and public technical specifications:

- **Scharlemann, J. P. W., Tanner, E. V. J., Hiederer, R., & Kapos, V. (2014).** Global soil carbon: understanding and managing the largest terrestrial carbon pool. *Carbon Management*, 5(1), 81–91. https://doi.org/10.4155/cmt.13.77
- **Parton, W. J., Schimel, D. S., Cole, C. V., & Ojima, D. S. (1987).** Analysis of Factors Controlling Soil Organic Matter Levels in Great Plains Grasslands. *Soil Science Society of America Journal*, 51(5), 1173–1179. https://doi.org/10.2136/sssaj1987.03615995005100050015x
- **Parton, W. J., Stewart, J. W. B., & Cole, C. V. (1988).** Dynamics of C, N, P and S in grassland soils: a model. *Biogeochemistry*, 5, 109–131. https://doi.org/10.1007/BF02180320
- **Parton, W. J., Scurlock, J. M. O., Ojima, D. S., Gilmanov, T. G., et al. (1993).** Observations and modeling of biomass and soil organic matter dynamics for the grassland biome worldwide. *Global Biogeochemical Cycles*, 7(4), 785–809. https://doi.org/10.1029/93GB02042
- **Metherell, A. K., Harding, L. A., Cole, C. V., & Parton, W. J. (1993).** *CENTURY Soil Organic Matter Model Environment. Technical Documentation. Agroecosystem Version 4.0.* Great Plains Systems Research Unit (GPSR) Technical Report No. 4, USDA-ARS, Fort Collins, CO.
- **VEMAP Members (1995).** Vegetation/ecosystem modeling and analysis project: Comparing biogeography and biogeochemistry models in a continental-scale study of terrestrial ecosystem responses to climate change and CO₂ doubling. *Global Biogeochemical Cycles*, 9(4), 407–437. https://doi.org/10.1029/95GB02746
- **Parton, W. J., Ojima, D. S., & Schimel, D. S. (2005).** CENTURY: Modeling Ecosystem Responses to Climate Change, Version 4 (VEMAP 1995). *ORNL Distributed Active Archive Center (DAAC)*. https://doi.org/10.3334/ORNLDAAC/820
- **Linacre, E. T. (1977).** A simple formula for estimating evaporation rates in various climates, using temperature data alone. *Agricultural Meteorology*, 18(6), 409–424. https://doi.org/10.1016/0002-1571(77)90007-3
- **Gupta, S. C., & Larson, W. E. (1979).** Estimating soil water retention characteristics from particle size distribution, organic matter percent, and bulk density. *Water Resources Research*, 15(6), 1633–1635. https://doi.org/10.1029/WR015i006p01633
- **Rawls, W. J., Brakensiek, D. L., & Saxton, K. E. (1982).** Estimation of Soil Water Properties. *Transactions of the ASAE*, 25(5), 1316–1320. https://doi.org/10.13031/2013.33720
- **Ryan, M. G. (1991).** Effects of Climate Change on Plant Respiration. *Ecological Applications*, 1(2), 157–167. https://doi.org/10.2307/1941808
- **Probert, M. E., Keating, B. A., Thompson, J. P., & Parton, W. J. (1995).** Modelling water, nitrogen, and crop yield for a long-term fallow management experiment. *Australian Journal of Experimental Agriculture*, 35(7), 941–950. https://doi.org/10.1071/EA9950941
- **European Commission — Joint Research Centre (JRC).** MIDAS Policy Model Inventory: *CENTURY agroecosystem dynamic model*. https://web.jrc.ec.europa.eu/policy-model-inventory/explore/models/model-century/

Released under the **[MIT License](LICENSE)**.
