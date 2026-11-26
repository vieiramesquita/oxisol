# Quick Start

Get up and running with Oxisol in under a minute — via the interactive web UI, the CLI, or directly as a Rust library.

## 1. Launch Oxisol Studio (Web Interface)

No Node.js, Python, or external data files are required:

```bash
cargo run --release -p oxisol-cli -- serve --port 8080
```

Open **`http://127.0.0.1:8080`** in your browser. You can immediately:
- Pick one of **6 calibrated global ecosystem presets** (including Southern Hemisphere *Cerrado / Tropical Savanna* and *Intensive Cropland*).
- Pin a scenario as **Baseline (A)**, adjust climate (`+2 °C`, `560 ppm CO₂`) or switch **Moldboard Plow → No-Till**, and inspect real-time deltas (`ΔSOMSC`, `ΔNPP`, `ΔAET`) alongside exact `< 1e-10` mass-balance verification.
- Switch to the **Spatial Grid & HPC** tab to execute a parallel multi-core continental gradient simulation and inspect every cell on an interactive 2D heatmap.

To mount a real VEMAP / spatial dataset directory inside Studio:

```bash
./target/release/oxisol serve \
  --host 127.0.0.1 \
  --port 8080 \
  --data-dir /path/to/spatial_grid
```

## 2. Run a Continental Spatial Grid from the CLI

```bash
# 1,000-year equilibrium spinup in f64 (ScientificClean default)
./target/release/oxisol run \
  --data-dir /path/to/vemap_data \
  --years 1000 \
  --profile clean \
  --precision f64 \
  --out-nc spinup_site.nc \
  --out-checkpoint spinup_checkpoint.json

# 99-year historical transient run (1895–1993) from lossless checkpoint
./target/release/oxisol run \
  --data-dir /path/to/vemap_data \
  --start-year 1895 \
  --years 99 \
  --co2-file /path/to/co2.dat \
  --transient-ppt /path/to/pptT3.nc \
  --transient-tn /path/to/tnT3.nc \
  --transient-tx /path/to/txT3.nc \
  --restart-checkpoint spinup_checkpoint.json \
  --out-nc transient_1895_1993.nc
```

## 3. Run Hardware Throughput Benchmarks

```bash
./target/release/oxisol bench --years 1000
```

## 4. Embed `oxisol-core` in Your Own Rust Program

Add `oxisol-core` to your `Cargo.toml`:

```rust
use oxisol_core::{
    initialize_cell, run_cell_simulation, CellSiteInput, CompatibilityProfile,
    ElementMode, IsotopeMode, MonthlyWeather, RawBiomeParams, Schedule, SimConfig,
};

fn main() {
    let biome = RawBiomeParams::<f64>::default();
    let cell_in = CellSiteInput::from_site(&biome.site_defaults);
    let weather = [MonthlyWeather {
        precip_cm: 6.5,
        tmin_c: 5.0,
        tmax_c: 19.0,
    }; 12];
    let schedule = Schedule::default();
    let cfg = SimConfig {
        profile: CompatibilityProfile::ScientificClean,
        elements: ElementMode::FullCnps,
        isotope: IsotopeMode::Unlabeled,
        co2_ppm_override: Some(420.0),
        audit_mass_balance: true,
        start_year: 1,
        end_year: 100,
    };

    let (state, _derived, last_annual) =
        run_cell_simulation(&biome, &cell_in, &[weather], &schedule, None, &cfg);

    println!("Final SOMSC: {:.1} g C/m2", state.somsc());
    println!("Final Annual NPP: {:.1} g C/m2/yr", last_annual.annual_npp);
}
```
