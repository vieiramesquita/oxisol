# 06. Management Events and Simulation Scheduling

## 1. Schedule Architecture (`.sch` Files)
A simulation schedule consists of an initial header (start year, last year, site file, isotope labeling mode, CO₂ system, initial ecosystem type, initial crop/tree) followed by one or more sequential **Blocks**. Each block specifies:
- Block end year (`blktnd`)
- Repeat cycle length in years (`rptyrs`, e.g., 1-year annual cycle, 2-year crop rotation, 300-year fire interval)
- Output interval and weather mode (`M` = mean monthly climate, `S` = stochastic, `F` = file/transient)
- List of dated events `(relative_year, month, event_kind, option_name)`

## 2. Supported Event Commands

### 2.1 Phenology and System Selection
- `CROP <name>`: Select active herbaceous crop/grassland parameter set from `crop.100`.
- `FRST`: Start herbaceous growing season (`crpgrw = true`).
- `LAST`: End herbaceous growing season (`crpgrw = false`).
- `SENM`: Trigger herbaceous senescence month (`senm = true`).
- `PLTM`: Planting month (initializes seed/crop establishment if live shoots are zero).
- `TREE <name>`: Select active forest/woody parameter set from `tree.100`.
- `TFST`: Start forest growing season (`forgrw = true`).
- `TLST`: End forest growing season (`forgrw = false`).

### 2.2 Agricultural & Ecological Disturbances
1. **`CULT <name>` (Cultivation / Tillage, `cult.100`)**:
   - Transfers fractions `cultra[0..6]` of live shoots, standing dead, and surface litter into soil litter/SOM pools.
   - Multiplies decomposition rates (`clteff[0..3]`) during the cultivation month (and optionally decaying over subsequent months).
2. **`FERT <name>` (Fertilization, `fert.100`)**:
   - Fixed mode: adds `feramt[e]` ($\text{g}\cdot\text{m}^{-2}$) to `minerl[0][e]`, with optional N volatilization fraction `ninhib`/`vlossf`.
   - Automatic mode (`aufert > 0`): dynamically tops up `minerl[0][e]` to meet target fraction of potential plant uptake.
3. **`FIRE <name>` (Grassland/Understory Fire, `fire.100`)**:
   - Removes fraction `flfrem` of live shoots, `fdfrem[0]` of standing dead, and `fdfrem[1]` of surface litter.
   - Exported C leaves via `FireVolatilization`; unvolatilized nutrients (`fret[e]`) return as ash to `minerl[0][e]`.
   - Temporarily increases root:shoot ratio (`frtsh`) and shifts shoot C:N (`fnue`).
4. **`GRAZ <name>` (Grazing, `graz.100`)**:
   - Removes fraction `flgrem` of live shoots and `fdgrem` of standing dead.
   - Exported C/nutrients split between animal export and excreta return (`gfcret`, `gret[e]`), where feces partition into surface litter (with lignin `feclig`) and urine returns to `minerl[0][e]` (minus volatilization).
   - Optional grazing feedback on root:shoot ratio and production (`grzeff`).
5. **`HARV <name>` (Harvest, `harv.100`)**:
   - Grain harvest (`flghrv = 1`): computes moisture-stressed harvest index `HI` from `himax`, `hiwsf`, and transpiration ratio during grain-fill months (`himon`), exporting grain C and `efrgrn[e]` of shoot nutrients (`HarvestExport`), with aboveground N volatilization (`vlossp`).
   - Straw/hay removal (`rmvstr`), residue return (`remwsd`), and optional crop termination (`aglrem`, `bglrem`).
6. **`IRRI <name>` (Irrigation, `irri.100`)**:
   - Adds fixed water amount (`irramt`, cm) or auto-irrigates (`auirri`) to refill root-zone soil water to target fraction (`fawhc`) of available water capacity when depleted below threshold (`awhc_trig`).
   - Adds irrigation sulfur (`sirri`) when $N_{\text{elem}} = 3$.
7. **`OMAD <name>` (Organic Matter Addition, `omad.100`)**:
   - Adds `astgc` ($\text{g C}\cdot\text{m}^{-2}$) with lignin fraction `astlig` and C:E ratios `astrec[e]` into surface (or incorporated soil) structural and metabolic litter via standard residue partitioning.
8. **`TREM <name>` (Tree Removal / Logging / Canopy Fire, `trem.100`)**:
   - `evntyp = 0` (cutting/logging): removes specified fractions (`remf[0..4]`) of live leaves, fine branches, large wood, and dead wood; unremoved slash enters surface litter and dead wood pools (`wood1`, `wood2`, `wood3`), and root mortality transfers `frootc`/`crootc` to soil litter/dead wood.
   - `evntyp = 1` (canopy burn): volatilizes burned fractions and returns ash nutrients (`retf[0..3][e]`) to `minerl[0][e]`.
9. **`EROD <rate>` (Soil Erosion)**:
   - Removes top-soil sediment (`erod` $\text{kg}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$), exporting proportional fractions of surface/soil SOM (`som1`, `som2`, `som3`) and structural/metabolic litter via `ErosionExport` with optional enrichment ratio (`lhze` lower-horizon replenishment).

## 3. Monthly Execution Order
Within each monthly step:
1. Update calendar/schedule events for `(year, month)` and active CO₂ concentration.
2. Compute hydrology (`h2olos`: PET, snow, interception, infiltration, evaporation, transpiration, drainage, leaching) and soil temperature (`soil_temp`).
3. Apply atmospheric deposition, non-symbiotic N fixation, irrigation, fertilization, and organic amendments.
4. Execute vegetation growth (`crop` and/or `forest`), nutrient uptake, symbiotic N fixation, senescence, death, and litterfall.
5. Execute management disturbances (`CULT`, `GRAZ`, `HARV`, `FIRE`, `TREM`, `EROD`).
6. Execute decomposition (`litdec`, `woodec`, `somdec`) and geochemical P/S equilibria (`pschem`).
7. Commit all `FlowLedger` transfers, update isotope fractions, accumulate monthly/annual diagnostics, and assert mass conservation (when auditing is enabled).
