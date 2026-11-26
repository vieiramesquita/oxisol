# Installation & HPC Builds

Oxisol is written in **100% pure, safe Rust** (`#![forbid(unsafe_code)]`) with **zero C runtime dependencies** (`libnetcdf` is not required). It compiles cleanly on Linux, WSL2, Windows, and macOS.

## Prerequisites

- **Rust toolchain ≥ 1.80** (`rustup`):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Git**

## Standard Release Build (Workstation / Laptop)

```bash
git clone https://github.com/oxisol/oxisol.git
cd oxisol
cargo build --release -p oxisol-cli
```

The single self-contained binary (including the CLI, parallel grid engine, pure-Rust NetCDF-3 codec, and embedded **Oxisol Studio** web UI) is produced at:

```text
target/release/oxisol        # Linux / WSL2 / macOS
target/release/oxisol.exe    # Windows
```

## Optimized Server Build (AMD EPYC + 512 GB RAM / HPC)

When deploying Oxisol on high-core-count **AMD EPYC** (Zen 3 / Zen 4 / Zen 5) or Intel Xeon servers, enable native CPU instruction set targeting (`AVX2+FMA3` or `AVX-512`) during compilation:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release -p oxisol-cli
```

### HPC Runtime Tuning Recommendations

| Server Class | Recommended Flags | Expected Behavior |
|---|---|---|
| Desktop 8C / 16T (Ryzen 7) | `--threads 16 --chunk-size 16` | ~290k cell-yr/s (`3.48M cell-mo/s`) |
| Single-Socket EPYC 64C / 128T | `--threads 128 --chunk-size 32` | Near-linear scaling; zero lock contention |
| Dual-Socket EPYC 128C / 256T (512 GB RAM) | `numactl --interleave=all oxisol run --threads 256 --chunk-size 32 ...` | Entire 1 km² continental rasters held read-only in RAM; `CellState<f64>` (~5.8 KiB) stays resident in per-core 32 KiB L1d cache |

> **Tip for WSL2 Users:** For heavy I/O workloads (thousands of files or large NetCDF rasters), store input/output datasets inside the native Linux filesystem (`~/...`) rather than `/mnt/c` or `/mnt/e` to avoid Windows–WSL 9P filesystem overhead.

## Verifying Your Installation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./target/release/oxisol --help
```

## Building Documentation Locally

```bash
# Rustdoc API reference for all crates
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --open

# mdBook documentation portal (install once: cargo install mdbook)
mdbook serve docs --open
```
