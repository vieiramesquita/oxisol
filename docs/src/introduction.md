# Introduction to Oxisol

**Oxisol** is an **open-source, clean-room, MIT-licensed *CENTURY-like* biogeochemical ecosystem engine** written from scratch in safe, modern Rust (`#![forbid(unsafe_code)]`).

> **Why "Oxisol"?**
> *Oxisols* (Latossolos) are deeply weathered tropical soils rich in iron oxides — literally colored like **rust**. The name celebrates both soil science and the **Rust** language powering this next-generation engine.

## Why Rebuild a CENTURY-like Model in the 2020s?

For more than three decades, CENTURY-family process models have underpinned global soil organic carbon (SOC) accounting, climate impact assessments, and agricultural policy evaluations (including 1 km² continental soil inventories). Yet the canonical legacy distribution remains **proprietary, non-open-source software** locked inside 1990s single-threaded Fortran 77/C with global `COMMON` state, legacy disk-bound I/O, and undocumented platform quirks.

**Foundational climate and soil carbon science should not stay trapped behind closed-source 1990s code.**

**Oxisol** delivers a transparent, auditable, community-owned alternative:

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
