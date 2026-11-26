# Compatibility Profiles & Validation

## `CompatibilityProfile`: Clean Science vs. Historical Parity

During our clean-room investigation, we documented 7 post-1993 V4.0 → V4.5 deltas and several historical software quirks (such as uninitialized `ompc` order-of-evaluation, missing `tan()` in `daylen`, and legacy `site.nc` omissions). Rather than silently hardcoding legacy bugs or breaking reproducibility with published 1990s datasets, Oxisol makes this choice **explicit**:

- **`CompatibilityProfile::ScientificClean` (default, recommended):**
  - IEEE-754 `f64` precision.
  - Exact spherical-astronomy daylength, properly ordered organic-matter pedotransfer `ompc`, hemisphere-aware deciduous phenology, and lossless checkpoints.
- **`CompatibilityProfile::VemapV45Parity` (audit mode):**
  - Reproduces documented VEMAP V4.5 (1995–2001) hydrology, Rawls `swflag = 4`, `soildepth > 150 cm` rock-fraction cutoff, and legacy `daylen` behavior for black-box auditing against archived VEMAP datasets.

## Tier A: Continental US VEMAP Benchmark (3,168 Cells × 1,000 Years)

Validated in black-box mode (`VemapV45Parity`, `f32`) against archived ORNL DAAC #820 `site.nc` across **all 17 biomes** (Tundra through Subtropical Arid Shrubland) — achieving **$r = 0.9877$ ($R^2 = 0.9756$)** on Total Soil Organic Carbon (`somsc`):

![Throughput Benchmark](../assets/benchmark_throughput.svg)

![Scientific Parity Across 17 Biomes](../assets/benchmark_parity.svg)

| Variable | Oxisol Mean | Archived CENTURY Mean | Mean Diff | **$R^2$** | Median Rel Diff | RMSE |
|---|---:|---:|---:|---:|---:|---:|
| `afiel0` (Field Capacity) | `0.269` | `0.269` | `0.00%` | **`1.0000`** | `0.00%` | `0.000` |
| `asmos0` (Top Soil Water, cm) | `3.436` | `3.397` | `+1.15%` | `0.8293` | **`1.52%`** | `0.461` |
| `somsc` (Total SOM, g C/m²) | **`4,134.9`** | **`4,231.8`** | **`-2.29%`** | **`0.9756`** | **`6.08%`** | `362.5` |
| `som3c` (Passive SOM, g C/m²) | **`1,635.3`** | **`1,636.9`** | **`-0.09%`** | **`0.9731`** | **`5.76%`** | `115.1` |
| `som2c` (Slow SOM, g C/m²) | `2,380.1` | `2,474.9` | `-3.83%` | **`0.9779`** | **`7.40%`** | `248.1` |
| `som1c_soil` (Active SOM, g C/m²) | **`119.5`** | **`120.0`** | **`-0.38%`** | **`0.9531`** | `11.25%` | `16.6` |
| `rlwodc` (Large Wood, g C/m²) | `3,806.0` | `3,656.9` | `+4.08%` | **`0.9576`** | `16.03%` | `1,236.2` |
| `minerl_n0` (Surface N, g N/m²) | **`0.609`** | **`0.616`** | **`-1.14%`** | `0.8596` | `16.94%` | `0.179` |

- **All 17/17 biomes** fall within `0.1%–22.4%` median `somsc` difference (e.g., Biome 6 `-0.1%`, Biome 7 `+1.9%`, Biome 5 `-3.6%`).
- **`f32` vs. `f64` 1,000-Year Drift (`ScientificClean`, 3,168 cells):** `somsc` median relative drift `2.42e-5` (`P99 = 5.06e-4`), `rlwodc` `1.11e-5`, `asmos0` `1.15e-7`.

## Tier B: Perturbation & Edge-Case Verification

Because continental VEMAP uses `NELEM = 1`, `IVAUTO = 0`, and `DRAIN = 1.0`, Oxisol adds a dedicated **Tier B** integration test suite (`crates/oxisol-core/tests/oracle_tier_b.rs`) validating:
- **5/5 directional agreement** head-to-head against a local black-box oracle on isolated perturbations (`CULT P`, `FERT+OMAD`, `IRRI A50`, `FIRE H`, `DRAIN = 0.2`).
- Full **P (`nelem = 2`)** and **S (`nelem = 3`)** geochemistry, `IVAUTO ∈ {1, 2, 3}`, continuous CO₂ ramp, C-14 isotope labeling, `EROD`, Southern Hemisphere deciduous abscission, zero-precip hyper-arid desert, continuous permafrost, 100% sand/clay, 90% rock, 1-layer soils, and >300-flow `FlowLedger` bursts — all maintaining `< 1e-10 g/m²` mass conservation in `f64`.
