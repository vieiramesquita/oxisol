# Glossary & Acronyms Dictionary

Biogeochemical ecosystem modeling, GIS remote sensing, and high-performance computing combine dozens of specialized acronyms. This dictionary provides plain-language definitions and units for every acronym used across **Oxisol**, **Oxisol Studio**, **CENTURY**, and the spatial raster pipeline.

---

## 1. Soil, Carbon & Nutrient Pools

| Acronym | Full Name | Meaning & Typical Units |
|---|---|---|
| **SOM / MOS** | Soil Organic Matter *(Matéria Orgânica do Solo)* | All dead organic carbon and associated nutrients (N, P, S) stored in the soil matrix. |
| **SOC / COS** | Soil Organic Carbon *(Carbono Orgânico do Solo)* | The carbon component of SOM (roughly ~50–58% of total organic matter mass). |
| **SOMSC** | Soil Organic Matter Soil Carbon | Total mineral-soil organic carbon (`SOM1_soil + SOM2 + SOM3`). Classic CENTURY models **`0–20 cm`**; Oxisol also reports **IPCC `0–30 cm`** and **Deep `0–100 cm`**. Units: **`Mg C/ha`** (`= t C/ha`) or **`g C/m²`** (`1 Mg C/ha = 100 g C/m²`). |
| **ΔSOMSC** | Net Soil Carbon Change | Final `SOMSC` minus post-spinup initial `SOMSC`. Positive = carbon sequestration; negative = soil carbon loss. |
| **SOM1** | Active Soil Organic Matter | Fast-cycling microbial biomass & labile metabolites (turnover: months to ~1–2 years). Split into surface microbes (`som1c[0]`) and soil active (`som1c[1]`). |
| **SOM2** | Slow Soil Organic Matter | Physically protected / particulate & mineral-associated organic matter (turnover: ~20–40 years). Usually the largest dynamic pool responding to management. |
| **SOM3** | Passive Soil Organic Matter | Chemically & physically recalcitrant, clay/oxide-bound carbon (turnover: ~400–2,000 years). |
| **Mg C/ha** | Megagrams of Carbon per Hectare | Standard SI agronomy & IPCC unit (`1 Mg = 1 metric ton = 1,000 kg`). **`1 Mg C/ha = 100 g C/m²`**. |
| **t CO₂e/ha** | Metric Tons of CO₂ Equivalent per Hectare | Greenhouse gas accounting unit: **`t CO₂e/ha = Mg C/ha × (44 / 12) ≈ Mg C/ha × 3.667`**. |
| **C · N · P · S** | Carbon, Nitrogen, Phosphorus, Sulfur | The four coupled biogeochemical elemental cycles simulated by Oxisol (`cn`, `cnp`, or `cnps`). |
| **C:N / C:E** | Carbon-to-Nitrogen / Carbon-to-Element Ratio | Stoichiometric ratio governing whether decomposing microbes release nutrients (**mineralization**) or lock them up (**immobilization**). |
| **DOC** | Dissolved Organic Carbon | Soluble organic carbon leached downward through soil layers with percolating water. |
| **Minerl N / P / S** | Plant-Available Mineral Nutrients | Inorganic nutrient pools (`NO₃⁻/NH₄⁺`, `PO₄³⁻`, `SO₄²⁻`) in soil layers (`g/m²`), directly available for plant uptake and microbial immobilization. |
| **BulkD / BDOD** | Soil Bulk Density | Dry soil mass per unit volume ($\text{g}\cdot\text{cm}^{-3}$). SoilGrids stores it in `cg/cm³` (`126 = 1.26 g/cm³`). |
| **Oxisol** | Oxisol *(Latossolo)* | Deeply weathered, iron/aluminum-oxide-rich tropical soil typical of the Brazilian Cerrado and Amazon — also the namesake of this project (rust = iron oxide = Rust!). |

---

## 2. Vegetation, Production & Climate

| Acronym | Full Name | Meaning & Typical Units |
|---|---|---|
| **NPP** | Net Primary Production | Net annual carbon fixed by plants via photosynthesis minus autotrophic (plant) respiration (`Mg C/ha/yr` or `g C/m²/yr`). |
| **Resp (Rh)** | Heterotrophic Respiration | Annual CO₂ carbon released back to the atmosphere by soil microbes decomposing litter, wood, and SOM (`Mg C/ha/yr` or `g C/m²/yr`). |
| **AET** | Actual Evapotranspiration | Real annual water lost via evaporation + canopy interception + plant transpiration (`cm/yr`, where `1 cm = 10 mm`). |
| **PET** | Potential Evapotranspiration | Atmospheric evaporative water demand estimated from temperature & latitude (`Linacre, 1977`; `cm/yr`). |
| **AWHC / CAD** | Available Water Holding Capacity *(Capacidade de Água Disponível)* | Water held between Field Capacity (`afiel`) and Wilting Point (`awilt`) in the rooting zone (`cm`). |
| **MAT / ΔT** | Mean Annual Temperature / Warming Shift | Average yearly air temperature (°C) and scenario warming offset applied over the transient period. |
| **CO₂ (ppm)** | Atmospheric Carbon Dioxide | Concentration in parts per million (e.g. `350 ppm` ≈ 1988, `422 ppm` ≈ 2024, `560+ ppm` future scenarios), driving CO₂ fertilization & stomatal water-use efficiency. |
| **DEFAC** | Decomposition Abiotic Factor | Dimensionless monthly multiplier (`0..1`) combining soil temperature and moisture controls on microbial decomposition. |
| **ANERB** | Anaerobic Decomposition Factor | Dimensionless multiplier (`aneref[2]..1.0`) slowing soil decomposition under poor drainage (`DRAIN < 1.0`) when warm soils are waterlogged. |
| **C3 / C4** | C3 vs. C4 Photosynthetic Pathways | Cool-season temperate plants (`C3`, e.g. wheat, temperate grasses) vs. warm-season tropical grasses (`C4`, e.g. *Urochloa brizantha*, *Panicum maximum*, maize, sugarcane) with higher heat optimum & water-use efficiency. |
| **LAI** | Leaf Area Index | One-sided green leaf area per unit ground area ($\text{m}^2/\text{m}^2$), controlling canopy shading and light interception. |
| **AGLIVC / BGLIVC** | Aboveground / Belowground Live Carbon | Herbaceous shoot carbon (`AGLIVC`) and live root carbon (`BGLIVC`). |
| **RLWODC** | Large Wood Carbon | Forest stem/trunk woody biomass carbon. |
| **STDEDC** | Standing Dead Carbon | Senesced upright dead herbaceous shoots before falling into surface litter. |
| **Spinup** | Equilibrium Initialization Phase | Pre-simulation years run under baseline land cover & climate so SOM pools (`SOM1/2/3`) reach realistic steady state before transient perturbations begin. |

---

## 3. Management & Disturbance Events (CENTURY `.sch` / `.100`)

| Acronym | Full Name | Meaning |
|---|---|---|
| **CULT** | Cultivation / Tillage | Soil disturbance (e.g. `CULT P` = moldboard plow) burying surface residues and temporarily accelerating decomposition (`clteff`). |
| **FERT / AUFERT** | Fertilization / Automatic Fertilization | Mineral N/P/S addition (`feramt`) or demand-driven automatic nutrient top-up (`aufert`) during crop growth. |
| **OMAD** | Organic Matter Addition | External organic amendments (manure, compost, straw) added to surface litter pools. |
| **IRRI** | Irrigation | Fixed or automatic water addition (e.g. `Auto 75% AWHC`) when root-zone soil water falls below threshold. |
| **FIRE** | Prescribed Fire / Wildfire | Burns live shoots (`FLFREM`), standing dead & surface litter (`FDFREM`), returns ash nutrients (`FRET`), and adjusts post-fire root:shoot (`FRTSH`) & C:N (`FNUE`). |
| **GRAZ** | Grazing | Livestock biomass removal from shoots & standing dead with partial return as feces (surface litter) and urine (mineral N). |
| **HARV** | Harvest | Grain/forage harvest exporting yield (`HI`) and optional straw removal (`rmvstr`). |
| **TREM** | Tree Removal / Logging / Canopy Fire | Selective thinning, clear-cutting, or crown burn in forests/savannas. |
| **EROD** | Soil Erosion | Topsoil sediment removal exporting proportional surface/soil SOM and litter. |
| **FRST / LAST / SENM** | First / Last / Senescence Month | Schedule phenology commands opening (`FRST`) and closing (`LAST`) the herbaceous growing season and triggering autumn senescence (`SENM`). |

---

## 4. Geospatial & Remote Sensing (GIS / Raster)

| Acronym | Full Name | Meaning |
|---|---|---|
| **LULC** | Land Use / Land Cover *(Uso e Cobertura da Terra)* | Categorical satellite map assigning each pixel an integer class code (e.g., MapBiomas Col. 11: `3` = Forest, `4` = Savanna, `15` = Pasture, `39` = Soybean). |
| **LUC** | Land-Use Change *(Mudança de Uso da Terra)* | Multi-year pixel transition between LULC classes (e.g., Forest `3` → Pasture `15` → Soybean `39`). |
| **GeoTIFF / COG** | Cloud-Optimized GeoTIFF | Georeferenced raster image format with internal tiling and DEFLATE compression, ready for QGIS, Python (`rasterio`), and cloud platforms. |
| **CRS / EPSG:4326** | Coordinate Reference System | Spatial map projection definition. `EPSG:4326` is standard WGS84 geographic latitude/longitude degrees. |
| **NoData** | Missing / Masked Pixel Value | Sentinel value (e.g. `-9999`) marking water, urban areas, or pixels outside the region of interest that Oxisol skips cleanly. |
| **GEE** | Google Earth Engine | Cloud geospatial platform used to export clipped MapBiomas, TerraClimate, and SoilGrids GeoTIFFs (see `examples/spatial_raster/gee_export_inputs.js`). |
| **`terrio`** | Pure-Rust Geospatial I/O Library | Rust crate (`opengeos/terrio`) reading/writing GeoTIFF, COG, and NetCDF natively with zero `libgdal` or `PROJ` C runtime dependencies. |
| **VEMAP** | Vegetation/Ecosystem Modeling and Analysis Project | Classic 1995/2004 0.5° continental US benchmark dataset (3,168 cells, 17 biomes) used to validate Oxisol against archived CENTURY V4.5 outputs. |
| **IPCC Tier 1/2/3** | Intergovernmental Panel on Climate Change Methodologies | Tier 1 = default global factors; Tier 2 = country-specific factors; **Tier 3 = process-based mechanistic models like Oxisol / CENTURY** (standard depth `0–30 cm`). |

---

## 5. Computing, Architecture & Rust

| Acronym | Full Name | Meaning |
|---|---|---|
| **`#![forbid(unsafe_code)]`** | Rust Compiler Memory-Safety Guarantee | Compiler-enforced ban on raw-pointer `unsafe` blocks across all 5 Oxisol crates — prevents segfaults, buffer overflows, and parallel data races at compile time. |
| **HPC** | High-Performance Computing | Multi-core/multi-socket servers (64–256+ CPU threads) for continental or high-resolution modeling. |
| **SIMD** | Single Instruction, Multiple Data | Hardware CPU vector instructions (`AVX2`, `AVX-512`, `NEON`) processing 4× `f64` or 8× `f32` soil cells in a single clock cycle. |
| **L1d Cache** | Level-1 Data Cache | Fastest CPU on-core memory (~32–48 KiB/core). Oxisol's `CellState` (~3–5.8 KiB, 64-byte aligned) fits completely inside L1d to avoid slow RAM bottlenecks. |
| **LTO** | Link-Time Optimization | Whole-program Rust compiler optimization (`lto = "thin"` balances peak runtime speed with fast builds). |
| **`f64` / `f32`** | 64-bit / 32-bit IEEE-754 Floating Point | `f64` (double precision, default) guarantees `< 1e-10 g/m²` mass conservation; `f32` (single precision) matches legacy Fortran `REAL*4` parity runs. |
| **CLI / REST API** | Command-Line Interface / HTTP JSON API | Run Oxisol headless in batch scripts (`oxisol raster`, `oxisol run`) or drive it programmatically over HTTP (`POST /api/simulate`, `POST /api/lulc/run`). |
