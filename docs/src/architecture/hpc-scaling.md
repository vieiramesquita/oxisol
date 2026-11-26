# Rust Engine & High-Performance Server Scaling

Oxisol is organized as a five-crate Cargo workspace with **`#![forbid(unsafe_code)]`** enforced across every crate:

```text
oxisol/
├── crates/
│   ├── oxisol-core/   # Pure biogeochemical kernel (Float f32/f64, CellState, FlowLedger, 10 subsystems)
│   ├── oxisol-io/     # 100% pure-Rust .100/.sch/.v2/co2.dat/NetCDF-3/JSON checkpoint codecs
│   ├── oxisol-grid/   # Lock-free rayon parallel runner + portable SIMD (wide::f32x8/f64x4)
│   ├── oxisol-web/    # Single-binary Axum REST API + embedded Oxisol Studio SPA (EN/PT-BR)
│   └── oxisol-cli/    # `oxisol` binary (`serve`, `run`, `validate`, `bench`)
├── docs/              # mdBook documentation portal + formal spec/ appendix
└── .github/workflows/ # Cross-platform CI + automated GitHub Pages (mdBook + Rustdoc)
```

## Key Performance Engineering Decisions

1. **L1d Cache Residency & Zero False Sharing (`#[repr(C, align(64))]`):**
   - `CellState<f32>` (~3 KiB) and `CellState<f64>` (~5.8 KiB) fit completely inside a single core's **32 KiB L1 data cache** on Zen 3 / Zen 4 / Zen 5 CPUs.
   - 64-byte cache-line alignment eliminates false sharing between concurrent worker threads.
2. **Immutable Shared Parameters (`Arc<RawBiomeParams<F>>`):**
   - Legacy Fortran mutated global `COMMON` arrays per cell. Oxisol separates immutable biome tables (`Arc<RawBiomeParams<F>>`) from per-cell `CellDerivedParams<F>`, enabling lock-free, contention-free scaling across 128–256+ hardware threads.
3. **O(1) Compiled Schedules & Const-Specialized `NELEM` Hot Loops:**
   - `.sch` blocks are pre-compiled once into `CompiledSchedule` indexed directly by `(rel_year, month)`.
   - Decomposition substeps dispatch through const-generic `NELEM ∈ {1, 2, 3}` direct mass-conserving transfers — pushing **single-thread Rust (`43.2k cell-yr/s`) to 2.11× faster than legacy `gfortran -O2` (`20.5k cell-yr/s`)**.
4. **Zero-I/O Worker Loop & Large-RAM Server Readiness (High-Memory Multi-Core Servers):**
   - All climate, soil, geography, and biome tables are preloaded once into read-only memory before `rayon` dispatches biome-grouped chunks.
   - Output vectors are assembled in deterministic cell order (`0..ncells`) independent of thread scheduling.
   - On a 128-core / 256-thread high-performance server, multi-million-cell 1 km² continental grids fit cleanly in memory and scale nearly linearly when built with `RUSTFLAGS="-C target-cpu=native"`.

## Measured Throughput Benchmarks (Test Machine: Ryzen 7 5700X, 8C/16T)

| Engine / Mode | Precision | Threads | Wall Time (3,168 × 1,000 yr) | Throughput | Speedup vs. Legacy |
|---|---:|---:|---:|---:|---:|
| Legacy `gfortran -O2` | `f32` | 1 | 154.5 s *(extrapolated)* | 20,500 cell-yr/s | 1.00× |
| `oxisol` Serial | `f32` | 1 | ~73.3 s | **43,225 cell-yr/s** | **2.11×** |
| `oxisol` Parallel (`VemapV45Parity`) | `f32` | 16 | **10.92 s** | **290,200 cell-yr/s** (`3.48M cell-mo/s`) | **14.16×** |
| `oxisol` Parallel (`ScientificClean`) | `f64` | 16 | **11.64 s** | **272,253 cell-yr/s** (`3.27M cell-mo/s`) | **13.28×** |
| `oxisol` Transient (`1895–1993`, 99 yr) | `f64` | 16 | **1.19 s** | **264,148 cell-yr/s** (`3.17M cell-mo/s`) | — |
| `oxisol-grid` SIMD `f32x8` SOM kernel | `f32` | 1 | — | **7.88M cell-yr/s** (`94.60M cell-mo/s`) | — |
