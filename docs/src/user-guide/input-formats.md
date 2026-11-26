# Input Formats & Checkpoints

All file parsers and writers live in **`oxisol-io`** (`#![forbid(unsafe_code)]`) and are implemented in **100% pure Rust** — no external C libraries required.

## 1. Parameter Tables (`.100`)

Oxisol reads classic CENTURY-style whitespace-delimited parameter files with parameter labels in column 2:
- **Biome Fixed Parameters (`*fix.100`):** Decomposition decay constants (`DEC1..DEC5`), texture controls (`PEFTXA/B`, `PS1S3`, `PS2S3`), leaching (`MINLCH`, `FLEACH`, `OMLECH`), soil layer depths (`ADEP`), and stoichiometric bounds (`VARAT1..3`).
- **Site Defaults (`lu*.100` / `<site>.100`):** Initial SOM, litter, woody, herbaceous, and mineral pools, `IVAUTO`, `NELEM`, `SWFLAG`, `DRAIN`, `EPNFA`, `EPNFS`, `SATMOS`.
- **Vegetation & Management Tables:** `crop.100`, `tree.100`, `cult.100`, `fert.100`, `fire.100`, `graz.100`, `harv.100`, `irri.100`, `omad.100`, `trem.100`.

## 2. Multi-Block Schedules (`.sch`)

Block-repeating schedules define simulation start/end years, initial ecosystem (`SystemKind`), initial crop/tree options, isotope mode, and repeating `(rel_year, month, EventKind)` blocks.

Before simulation starts, `oxisol-core` compiles each `.sch` file once into a **`CompiledSchedule`** with $O(1)$ lookup per `(rel_year, month)` — eliminating repeated linear scans during long 1,000-year spinups.

## 3. Spatial Grids & Weather

- **VEMAP ASCII Rasters (`.v2`):** Header + row-major grids for biome classification (`vveg.v2`), bulk density (`mbd1_m1.v2`), clay (`mcl1_m1.v2`), sand (`msa1_m1.v2`), and optional `rock` / `zminrl`.
- **Mean Climate NetCDF-3 (`phase1.ncdf`):** `(12, ncells)` monthly climatology (`ppt` in mm → cm; `tn`/`tx` scaled by `0.1 °C`).
- **Transient Climate NetCDF-3 (`pptT3.nc`, `tnT3.nc`, `txT3.nc`):** `(n_masked_cells, n_months)` historical/projected monthly weather indexed via `geog.nc` `mask2` IDs (`ppt` in mm → cm; `tmin`/`tmax` in °C).
- **Atmospheric CO₂ Table (`co2.dat`):** Two-column whitespace-separated `year ppm` table supporting continuous historical or SSP/RCP CO₂ trajectories.

## 4. Checkpoints: Native Lossless JSON vs. Legacy `site.nc`

> **Why Native Checkpoints Matter**
> Legacy 235-column `site.nc` files omit structural/metabolic litter nutrient pools (`struce`, `metabe`), lignin fractions (`strlig`, `pltlig`), and forest internal nutrient storage (`forstg`), and suffered from uninitialized stack bytes in column 169 (`occlud`) in historical distributions.

Oxisol supports **both**:
1. **Native Lossless JSON Checkpoints (`--out-checkpoint` / `--restart-checkpoint`):** Serializes the **complete** `CellState<F>` and `CellDerivedParams<F>` for every cell with zero state loss and zero cross-cell leakage across spinup → transient restarts.
2. **Legacy-Compatible NetCDF-3 (`--out-nc` / `site.nc`):** Reads and writes NetCDF-3 Classic (`CDF\x01`) and 64-bit Offset (`CDF\x02`) files via `oxisol_io::netcdf3` for interoperability with Python `netCDF4`/`xarray` and validation against archived VEMAP datasets.
