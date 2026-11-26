# Vegetation, CO₂ & Land Management

## Herbaceous (Grassland & Crop) Production

- Potential monthly growth combines `prdx1`, temperature response (`ppdf`), soil moisture production curve (`pprdwc`), canopy shading (`pmxbio`), and CO₂ growth multiplier (`co2ipr`).
- Realized growth is constrained by the most limiting available nutrient across active elements (`N`, `P`, `S`) using `favail[0..6]` (including dynamic P availability scaling) and internal crop storage (`crpstg`).
- Grain harvest index (`himax`, `hiwsf`) tracks cumulative growing-season transpiration ratio across grain-fill months.

## Forest & Savanna Ecosystems

- Five live woody components: foliage (`rleavc`), fine roots (`frootc`), fine branches (`fbrchc`), large wood/bole (`rlwodc`), and coarse roots (`crootc`), plus three dead-wood pools (`wood1`, `wood2`, `wood3`).
- Canopy Leaf Area Index (LAI) drives light interception and savanna understory shading (`laitop`, `klai`, `btolai`, `basfct`, `sitpot`).
- **Hemisphere-Aware Deciduous Phenology:** For deciduous trees (`decid == 1`), Oxisol automatically shifts the autumn abscission and spring flush calendar by 6 months when `lat < 0` — ensuring Southern Hemisphere forests and Cerrado savannas shed leaves in austral autumn (April–June) rather than austral spring.

## Atmospheric CO₂ Response

Elevated atmospheric CO₂ continuously interpolates between `fix.co2ppm[0]` and `fix.co2ppm[1]`, scaling:
- Potential production (`co2ipr`)
- Transpiration water-use efficiency (`co2itr`)
- Plant C:E stoichiometry (`co2ice`)
- Root-to-shoot allocation (`co2irs`)

## All 9 Land Management Disturbances

Oxisol implements all official CENTURY management parameter semantics (Metherell et al. 1993, Appendix 2):

| Command | Mechanism | Key Parameter Semantics |
|---|---|---|
| **`CULT`** | Tillage / soil disturbance | `CULTRA(1..7)` routes shoots → standing dead/surface/soil, standing dead → surface/soil, surface litter → soil, roots → soil; boosts decomposition via `CLTEFF(1..4)` |
| **`FERT`** | Mineral fertilization | Fixed `[N, P, S]` additions (`feramt`) or auto-fertilization (`aufert`) |
| **`OMAD`** | Organic matter addition | Compost/manure carbon (`astgc`), lignin (`astlig`), and C:E (`astrec`) |
| **`IRRI`** | Irrigation | All 4 modes: `AUIRRI = 0` (fixed), `1` (PET-projected refill to field capacity below `fawhc`), `2` (fixed auto), `3` (soil-moisture trigger) |
| **`HARV`** | Crop harvest | `AGLREM` & `BGLREM` are **remaining unaffected fractions** (`1 - aglrem` harvested/killed); grain (`flghrv`), straw removal (`rmvstr`), and tuber harvest (`hibg`) |
| **`GRAZ`** | Livestock grazing | Live & standing-dead removal (`flgrem`, `fdgrem`), fecal/urine C & N/P/S return (`gfcret`, `gret`, `fecf`) |
| **`FIRE`** | Surface/grass fire | Combusts live shoots, standing dead, and surface litter; returns mineral ash (`fret`) and temporarily shifts root:shoot (`frtsh`) |
| **`TREM`** | Logging / canopy fire | `RETF[part][elem]` returns slash (`evntyp = 0`) or ash (`evntyp = 1`) accurately across leaves, fine branches, large wood, and dead wood |
| **`EROD`** | Soil erosion | Exports surface SOM/litter proportional to `kg/m²/mo` rate and replenishes lower horizon via `lhzf` |
