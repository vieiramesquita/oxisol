# Introduction to Oxisol

**Oxisol** is an **open-source, clean-room, MIT-licensed *CENTURY-like* biogeochemical ecosystem engine** written from scratch in safe, modern Rust (`#![forbid(unsafe_code)]`).

## The Story Behind Oxisol

Soils store more organic carbon than the atmosphere and all living terrestrial vegetation combined ([Scharlemann et al., 2014](https://doi.org/10.4155/cmt.13.77)). In the late 1980s, William J. Parton and colleagues at Colorado State University formulated the **CENTURY** framework ([Parton et al., 1987](https://doi.org/10.2136/sssaj1987.03615995005100050015x); [Parton et al., 1988](https://doi.org/10.1007/BF02180320); [Parton et al., 1993](https://doi.org/10.1029/93GB02042); Metherell et al., 1993) to simulate carbon, nitrogen, phosphorus, and sulfur cycling across active, slow, and passive soil organic matter pools. The model became a cornerstone of continental syntheses such as **VEMAP** ([VEMAP Members, 1995](https://doi.org/10.1029/95GB02746); [Parton et al., 2005](https://doi.org/10.3334/ORNLDAAC/820)) and regional policy assessments ([EC JRC MIDAS](https://web.jrc.ec.europa.eu/policy-model-inventory/explore/models/model-century/)).

Yet while the underlying equations were published openly, the canonical 1990s Fortran 77/C distribution remained proprietary (*All Rights Reserved* / non-free software), single-threaded, and anchored to Northern-Hemisphere assumptions.

**Named after *Oxisols* (*Latossolos*)** — the deeply weathered, iron-oxide-rich tropical soils whose **rust** hue mirrors the **Rust** language — **Oxisol** brings this foundational biogeochemical science into the open-source era: auditable, memory-safe, hemisphere-aware, **14.2× faster**, and ready for modern servers and interactive web exploration.

## Legacy Distribution vs. Oxisol

| Capability | Legacy 1990s Distribution | **Oxisol (Rust, MIT)** |
|---|---|---|
| **License** | Proprietary / Non-Free (*All Rights Reserved*) | **MIT Open Source** (clean-room from published literature) |
| **Memory & Thread Safety** | Global `COMMON` blocks, raw C pointers | `#![forbid(unsafe_code)]`, zero global state, compile-time `PoolId` |
| **Numerical Precision** | Hardcoded single-precision `REAL*4` | Generic `Float` (`f64` default + `f32` parity audit mode) |
| **Mass Conservation** | Unchecked implicit float drift | Optional per-step `< 1e-10 g/m²` C/N/P/S conservation audit |
| **Single-Thread Throughput** | ~20,500 cell-yr/s (`gfortran -O2`) | **43,225 cell-yr/s** (**2.11×** faster on 1 core) |
| **Multi-Core Throughput** | Single-threaded sequential | **290,200 cell-yr/s** on 8C/16T (**14.2×** faster); built for **128C+ modern high-performance multi-core servers** |
| **Hemisphere Support** | Northern-Hemisphere-assumed deciduous logic | Native Northern & Southern Hemisphere phenology |
| **User Interface** | None (text files + legacy scripts) | Single-binary embedded **Oxisol Studio** (EN/PT-BR) + REST API |
| **Runtime Dependencies** | System `libnetcdf.so` / Fortran runtime | **Zero C runtime dependencies** (100% pure Rust) |

## Clean-Room Provenance

Every equation, data structure, and algorithm in Oxisol was independently engineered from peer-reviewed biogeochemical literature (Parton et al. 1983, 1987, 1988, 1993, 1994; Metherell et al. 1993; Probert et al. 1995; Rawls et al. 1982; Gupta & Larson 1979; Ryan 1991; VEMAP 1995). No proprietary source code, `COMMON` block layouts, or non-free data assets are included anywhere in this repository.

## Documentation Map

- **[Getting Started](getting-started/installation.md):** Compile locally or optimize for high-performance multi-core servers, then run your first simulation in 60 seconds.
- **[User Guide](user-guide/studio.md):** Master **Oxisol Studio**, the `oxisol` CLI, and all input/checkpoint formats.
- **[Scientific Reference](science/overview.md):** Equations, pool diagrams, hydrology, decomposition, vegetation, management disturbances, and continental validation benchmarks.
- **[Architecture & API](architecture/hpc-scaling.md):** Rust workspace crates, L1d cache layout, `rayon` + SIMD optimization, REST JSON endpoints, and [Rustdoc API Reference](../api/oxisol_core/index.html).

## Authorship

- **Author:** [Vinícius Mesquita](https://github.com/vieiramesquita) (`@vieiramesquita`)
- **AI Co-Author:** Antigravity (Google DeepMind)
