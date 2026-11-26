# Oxisol

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Safe Rust](https://img.shields.io/badge/unsafe-forbidden-brightgreen.svg)](crates/oxisol-core/src/lib.rs)
[![Docs](https://img.shields.io/badge/Docs-GitHub%20Pages-06b6d4.svg)](https://vieiramesquita.github.io/oxisol/)

**Oxisol** is an **open-source, clean-room, MIT-licensed *CENTURY-like* biogeochemical ecosystem engine** written from scratch in pure, safe Rust (`#![forbid(unsafe_code)]`) and engineered for 2020s multi-core CPUs, modern high-performance servers, and interactive web workflows.

> **Why "Oxisol" — and why a CENTURY-like rewrite?**
> *Oxisols* (Latossolos) are deeply weathered tropical soils rich in iron oxides — literally colored like **rust**. Meanwhile, the canonical 1990s CENTURY Fortran/C distribution remains proprietary (*All Rights Reserved* / non-open-source per the EU JRC MIDAS Policy Model Inventory). **Foundational climate and soil carbon science shouldn't stay locked inside proprietary 1990s Fortran.** Oxisol delivers an independent, clean-room, community-owned CENTURY-like model derived from published peer-reviewed biogeochemical literature — **2.1× faster than `gfortran -O2` on a single core, 14.2× faster on 8C/16T**, with zero C dependencies and a single-binary embedded Web Studio.

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

## Scientific Citations & License

Oxisol implements biogeochemical concepts established in Parton et al. (1983, 1987, 1988, 1993, 1994), Metherell et al. (1993), Probert et al. (1995), Rawls et al. (1982), Gupta & Larson (1979), Ryan (1991), and VEMAP (1995).

Released under the **[MIT License](LICENSE)**.
