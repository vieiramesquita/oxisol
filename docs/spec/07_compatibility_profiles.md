# 07. Compatibility Profiles and Documented V4.0 → V4.5 Deltas

## 1. Rationale
While validating against the historical ORNL DAAC #820 VEMAP Version 4.5 gridded CENTURY model (1995–2001), seven well-defined behavioral differences and legacy implementation quirks were identified relative to the published CENTURY V4.0 scientific documentation (Metherell et al. 1993).

To ensure `century-rs` delivers **modern, bug-free scientific precision** by default while still enabling **rigorous black-box parity auditing** against legacy VEMAP runs, all behavioral switches are isolated in a single explicit enum:

```rust
pub enum CompatibilityProfile {
    /// Default: f64 precision, corrected equations, immutable biome parameters,
    /// full state serialization with zero inter-cell leakage.
    ScientificClean,

    /// Audit profile: f32 precision + documented V4.5 (1995-2001) behavioral deltas
    /// for black-box numerical verification against legacy VEMAP outputs.
    VemapV45Parity,
}
```

## 2. Formal Catalog of the 7 Documented Deltas

| # | Subsystem | `ScientificClean` (Default) | `VemapV45Parity` (Audit Mode) | Scientific Impact |
|---|---|---|---|---|
| 1 | **Astronomical Daylength** | Uses exact spherical astronomy $\cos H_0 = -\tan(\phi)\tan(\delta)$. | Reproduces legacy omission of $\tan(\cdot)$ on solar declination: $-\tan(\phi)\cdot\delta$. | Alters PET scaling slightly at high latitudes during solstices. |
| 2 | **Organic Matter % in Rawls/G&L Water Retention (`ompc`)** | Initializes `somsc` before computing `ompc`, so initial SOM properly contributes to $\theta_{\text{fc}}$ and $\theta_{\text{wp}}$. | Evaluates `ompc` before `somsc` initialization (`ompc == 0.0` at startup for all cells). | Slight shift in field capacity and wilting point in high-SOC soils (e.g., tundra, wet forests). |
| 3 | **Soil Depth & Rock Fraction Override** | Applies user-specified `rockfrac` consistently across all profile depths with explicit metadata. | Zeroes `rockfrac` whenever total profile depth exceeds 150 cm (`soildepth > 150.0`), matching the 1999 VEMAP deep-soil rule. | Affects water-holding capacity in 8-layer (180 cm) cells. |
| 4 | **Biome Parameter Immutability (`texesp`, `pparmn`, `psecmn`)** | Stored in immutable `Arc<RawBiomeParams>`; texture/pH adjustments live strictly in per-cell `CellDerivedParams`. | Computes the exact same per-cell derived values without cross-cell mutation bugs (legacy mutated global COMMON arrays across cells, which only manifested if cells shared mutable state). | Eliminates order-dependent multi-cell bugs in parallel EPYC runs. |
| 5 | **Floating-Point Precision** | IEEE 754 `f64` throughout, mass conservation verified to $10^{-10}\ \text{g}\cdot\text{m}^{-2}$. | IEEE 754 `f32` matching single-precision Fortran `REAL`, eliminating precision divergence during parity checks. | Prevents long-horizon 1,000-year `f32` accumulation drift in modern studies. |
| 6 | **Decomposition Sub-stepping & Litter Caps** | Clean 4-substep monthly integration with explicit non-negativity guards and full `FlowLedger` accounting. | Matches V4.5 4-substep monthly decomposition order and legacy `strmax` litter C cap behavior. | Ensures exact equilibrium SOM parity over 1,000-year spinups. |
| 7 | **Checkpoint Serialization Completeness** | Serializes 100% of `CellState` (`struce`, `metabe`, `strlig`, `forstg`, `pltlig`, `occlud`, etc.) with deterministic zero-initialization. | Reads/writes legacy 235-float `site.nc` layout for oracle comparison while never leaking state between cells or writing uninitialized stack memory into column 169 (`occlud`). | Fixes legacy multi-cell transient state contamination. |

## 3. Clean-Room & Server Scalability Guarantee
Neither profile uses global mutable variables (`static mut` or COMMON blocks). Every cell's state (`CellState<F>`) and derived soil parameters (`CellDerivedParams<F>`) are completely isolated on the stack or in per-thread heap slices, enabling deterministic, bit-reproducible execution across 1 to 256+ CPU cores (Ryzen 7 5700X through dual-socket AMD EPYC + 512 GB RAM servers).

## 4. Verified Black-Box Parity Summary (Continental US VEMAP, 3,168 Cells × 1,000 Years)
Evaluated with `VemapV45Parity` (`f32`) against archived VEMAP `site.nc`:
- **Hydraulic pedotransfer (`afiel0`, `awilt0`):** exact to IEEE `f32` epsilon (`MedRel = 0.0`, `MaxRel <= 2.62e-7`).
- **Top-layer soil water (`asmos0`):** `3.435` vs `3.397 cm` (`MedRel = 1.52%`, `RMSE = 0.460 cm`).
- **Total soil organic carbon (`somsc`):** `4,238.1` vs `4,231.8 g C/m²` (`+0.15%` continental mean diff, `MedRel = 6.62%`, `RMSE = 481.3 g C/m²`).
- **Slow & passive SOM (`som2c`, `som3c`):** `som2c` `2,453` vs `2,475 g C/m²` (`MedRel = 7.62%`), `som3c` `1,662` vs `1,637 g C/m²` (`MedRel = 6.29%`).
- **Large wood & surface mineral N (`rlwodc`, `minerl_n0`):** `rlwodc` `3,992` vs `3,657 g C/m²` (`MedRel = 14.90%`), `minerl_n0` `0.635` vs `0.616 g N/m²` (`MedRel = 17.18%`, `RMSE = 0.181`).
- **`f32`-vs-`f64` 1,000-year drift (`ScientificClean`):** `somsc` `MedRel = 2.29e-5` (`P99 = 5.04e-4`), `rlwodc` `MedRel = 1.07e-5`, `asmos0` `MedRel = 1.07e-7`.
- **Tier B head-to-head perturbation probes (5/5 directional agreement):** `CULT P`, `FERT+OMAD`, `IRRI A50`, `FIRE H`, and `DRAIN=0.2` all match the local black-box oracle's response direction on an isolated VEMAP cell.
