# 01. State Pools, Stoichiometry, Isotopes, and Mass Balance

## 1. Scientific Foundation
This specification defines the compartment topology and conservation invariants of the monthly biogeochemical soil-plant ecosystem model described in Parton et al. (1983, 1987, 1988, 1993, 1994) and Metherell et al. (1993, USDA-ARS GPSR Technical Report No. 4).

All mass pools are expressed in $\text{g element} \cdot \text{m}^{-2}$ unless otherwise stated. Time advances in discrete monthly steps $\Delta t = \frac{1}{12}\text{ yr}$ (`month` $\in \{1, \dots, 12\}$).

## 2. Element Modes
The model supports three element configurations (`ElementMode`):
- `CarbonNitrogen` ($N_{\text{elem}} = 1$): Carbon (C) and Nitrogen (N).
- `CarbonNitrogenPhosphorus` ($N_{\text{elem}} = 2$): C, N, and Phosphorus (P).
- `FullCnps` ($N_{\text{elem}} = 3$): C, N, P, and Sulfur (S).

Element index $e \in \{0=\text{N}, 1=\text{P}, 2=\text{S}\}$.

## 3. Carbon and Nutrient Compartments

### 3.1 Soil Organic Matter (SOM) and Litter Pools
1. **Surface Structural Litter** (`strucc[0]`, `struce[0][e]`, lignin fraction `strlig[0]`): Resistant surface plant residues (turnover ~1–5 yr).
2. **Soil Structural Litter** (`strucc[1]`, `struce[1][e]`, lignin fraction `strlig[1]`): Resistant belowground plant residues.
3. **Surface Metabolic Litter** (`metabc[0]`, `metabe[0][e]`): Labile surface residues (turnover ~0.1–1 yr).
4. **Soil Metabolic Litter** (`metabc[1]`, `metabe[1][e]`): Labile belowground residues.
5. **Surface Active Microbial Pool** (`som1c[0]`, `som1e[0][e]`): Surface microbial biomass and byproducts (texture-independent turnover).
6. **Soil Active SOM** (`som1c[1]`, `som1e[1][e]`): Soil microbes and labile microbial products (texture-dependent turnover and respiration).
7. **Slow SOM** (`som2c`, `som2e[e]`): Physically/chemically stabilized organic matter (turnover ~20–50 yr).
8. **Passive SOM** (`som3c`, `som3e[e]`): Recalcitrant humic material (turnover ~200–1500 yr).

### 3.2 Herbaceous (Grassland/Crop) Plant Pools
- **Live Shoots** (`aglivc`, `aglive[e]`): Aboveground living herbaceous biomass.
- **Live Roots** (`bglivc`, `bglive[e]`): Belowground living herbaceous roots.
- **Standing Dead** (`stdedc`, `stdede[e]`): Standing dead herbaceous shoots.
- **Crop Storage** (`crpstg[e]`): Internal nutrient storage buffer for herbaceous plants.

### 3.3 Woody (Forest/Savanna) Plant Pools
- **Live Leaves** (`rleavc`, `rleave[e]`): Canopy foliage.
- **Fine Roots** (`frootc`, `froote[e]`): Active fine roots.
- **Fine Branches** (`fbrchc`, `fbrche[e]`): Live fine branches.
- **Large Wood** (`rlwodc`, `rlwode[e]`): Live bole/stem wood.
- **Coarse Roots** (`crootc`, `croote[e]`): Structural coarse roots.
- **Forest Storage** (`forstg[e]`): Internal nutrient reserve (translocation during senescence).
- **Dead Wood Pools**:
  - Dead fine branches (`wood1c`, `wood1e[e]`)
  - Dead large wood (`wood2c`, `wood2e[e]`)
  - Dead coarse roots (`wood3c`, `wood3e[e]`)

### 3.4 Inorganic Mineral & Geochemical Pools
- **Layered Mineral Nutrients** (`minerl[layer][e]`, `layer` $\in \{0, \dots, 9\}$): Plant-available mineral N ($\text{NH}_4^+ + \text{NO}_3^-$), labile P, and mineral S across up to 10 soil layers.
- **Parent Material** (`parent[e]`): Weatherable mineral nutrient source (primarily P and S).
- **Secondary Mineral Pool** (`secndy[e]`): Sorbed/secondary mineral P and S in reversible equilibrium with labile mineral pools.
- **Occluded P** (`occlud`): Irreversibly/slowly occluded inorganic phosphorus.

## 4. Carbon Isotope Tracking
Each carbon pool $X$ is paired with an unlabeled/labeled isotope pair $(X_{\text{unlab}}, X_{\text{lab}})$ such that:
$$X_{\text{total}} = X_{\text{unlab}} + X_{\text{lab}}$$
- `Unlabeled` (`labtyp = 0`): All C enters unlabeled ($f_{\text{lab}} = 0$).
- `C14` (`labtyp = 1`): Radiocarbon labeling; microbial respiration does not fractionate except via specified discrimination.
- `C13` (`labtyp = 2`): Stable carbon isotope $\delta^{13}\text{C}$ tracking with photosynthetic discrimination (`cisofr`).

## 5. Conservation of Mass Invariants
Using a strongly-typed `FlowLedger<F>` with explicit source and sink `PoolId`s (including explicit system boundary sinks/sources: `AtmosphereC`, `AtmosphereN`, `Leaching`, `HarvestExport`, `FireVolatilization`, `ErosionExport`, `FertilizerInput`, `AtmosphericDeposition`, `FixationInput`, `WeatheringInput`, `OrganicAmendmentInput`), every monthly step satisfies:

$$\Delta C_{\text{ecosystem}} = C_{\text{NPP}} + C_{\text{OMAD}} - \left(R_{\text{heterotrophic}} + C_{\text{harvest}} + C_{\text{fire}} + C_{\text{graze\_export}} + C_{\text{erosion}} + C_{\text{leach\_org}}\right)$$

$$\Delta E_{\text{ecosystem}} = E_{\text{dep}} + E_{\text{fix}} + E_{\text{fert}} + E_{\text{omad}} + E_{\text{irri}} - \left(E_{\text{volat}} + E_{\text{leach}} + E_{\text{harvest}} + E_{\text{fire}} + E_{\text{graze\_export}} + E_{\text{erosion}}\right)$$

In `ScientificClean` (`f64`), mass conservation holds to $|\epsilon| \le 10^{-10}\ \text{g}\cdot\text{m}^{-2}$ per monthly step across all subsystems and management events.

## 6. Residue Partitioning into Structural and Metabolic Pools
Incoming plant residue or organic amendment with carbon mass $C$, lignin fraction $L \in [0, 1]$, and nutrient-to-carbon ratio $r_N = N / C$ (so $\text{L:N} = L / r_N$) is partitioned into metabolic fraction $f_m$ (Parton et al. 1987):
$$f_m = \text{clamp}\!\left(0.85 - 0.018 \cdot \frac{L}{r_N},\ f_{m,\min},\ 1 - L\right)$$
where the minimum metabolic fraction ensures structural carbon never carries a lignin fraction exceeding $1.0$ (specifically $f_m \le 1 - L$ and in standard CENTURY $f_{m,\min} = \text{pmco2}$ or $0.0$ bounded so $L_{\text{struct}} = \frac{L}{1 - f_m} \le 1.0$). All lignin enters the structural pool:
$$C_{\text{metab}} = f_m C, \qquad C_{\text{struct}} = (1 - f_m) C, \qquad L_{\text{struct,new}} = \frac{C_{\text{old}} L_{\text{old}} + C \cdot L}{C_{\text{old}} + C_{\text{struct}}}$$
Structural nutrient allocation uses fixed structural C:E ratio $r_{\text{str}}(e)$ (default C:N = 150, C:P = 500, C:S = 500):
$$E_{\text{struct}} = \min\!\left(\frac{C_{\text{struct}}}{r_{\text{str}}(e)},\ E_{\text{avail}}\right), \qquad E_{\text{metab}} = E_{\text{total}} - E_{\text{struct}}$$
If $E_{\text{total}}$ is insufficient to satisfy $C_{\text{struct}} / r_{\text{str}}(e)$, mineral nutrient is immobilized from layer 1 (`minerl[0][e]`) when available.
