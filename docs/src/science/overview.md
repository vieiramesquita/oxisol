# Model Overview & Mass Conservation

Oxisol simulates monthly terrestrial ecosystem biogeochemistry across **Carbon (C)**, **Nitrogen (N)**, **Phosphorus (P)**, and **Sulfur (S)**, coupled to multi-layer soil water balance, soil temperature, plant growth, and land management.

## Conceptual Pool Architecture

```text
                       ┌──────────────────────┐
                       │    Atmosphere CO2    │
                       │ (Photosynthesis/Resp)│
                       └──────────┬───────────┘
                                  │ NPP / Respiration
                                  ▼
      ┌─────────────────────────────────────────────────────┐
      │                   Live Vegetation                   │
      │  Herbaceous (agliv, bgliv) · Woody (leaf, wood, rt) │
      └───────────────────────────┬─────────────────────────┘
                                  │ Senescence / Mortality / Harvest
                                  ▼
      ┌─────────────────────────────────────────────────────┐
      │              Litter & Dead Wood Pools               │
      │  Structural & Metabolic (Surface/Soil) · Wood 1/2/3 │
      └────────────────────┬──────────────────┬─────────────┘
                           │                  │ Lignin fraction
                           ▼                  ▼
                  ┌────────────────┐   ┌────────────────┐
                  │  SOM1 Active   │──▶│   SOM2 Slow    │
                  │ (Microbial, yr)│   │  (Decades)     │
                  └────────┬───────┘   └────────┬───────┘
                           │                    │
                           └─────────┬──────────┘
                                     ▼
                            ┌────────────────┐
                            │  SOM3 Passive  │
                            │  (Centuries)   │
                            └────────────────┘
                                     ▲
                                     │ Mineralization / Uptake / Leaching
                            ┌────────┴───────┐
                            │ Mineral N·P·S  │
                            │ (10 soil lyrs) │
                            └────────────────┘
```

## Monthly Execution Order

Every call to `step_month` executes in a deterministic sequence matching published CENTURY ecosystem semantics:

1. **Schedule Resolution:** Resolve $O(1)$ pre-compiled events for `(year, month)` (`CROP`, `TREE`, `FRST`, `LAST`, `PLTM`, `SENM`, `TFST`, `TLST`, `CULT`, `FERT`, `IRRI`, `OMAD`, `GRAZ`, `HARV`, `FIRE`, `TREM`, `EROD`).
2. **Pre-Growth Management:** Apply `CULT` (cultivation residue burial + decomposition boost `clteff`), `FERT`, `OMAD`, and `IRRI`.
3. **Climate & Hydrology:** Compute Linacre (1977) PET, astronomical daylength, snowpack accumulation/melt, interception, bare-soil evaporation, Probert (1995) runoff, layer-by-layer soil infiltration, transpiration, deep drainage, and solute leaching (`amov`).
4. **Soil Temperature & Abiotic Modifiers:** Estimate biomass- and snowpack-insulated monthly soil temperature (`stemp`), moisture/temperature decomposition factor (`defac`), and anaerobic factor (`anerb`).
5. **CO₂ Physiological Effects:** Evaluate `co2ipr`, `co2itr`, `co2ice`, and `co2irs` (short-circuited to `1.0` when at baseline CO₂).
6. **Atmospheric Deposition & Non-Symbiotic Fixation:** Apply monthly N and S deposition plus AET-threshold soil N fixation (`EPNFS`).
7. **4-Substep Decomposition (`ntspm = 4`):** Decompose surface/soil structural & metabolic litter, dead wood (`wood1`–`wood3`), and SOM1/SOM2/SOM3 pools with texture, lignin, and nutrient immobilization controls.
8. **Vegetation Growth & Senescence:** Run herbaceous (`step_crop`) and/or woody (`step_forest`) production, savanna basal-area competition, nutrient uptake, Ryan (1991) maintenance respiration, and hemisphere-aware deciduous abscission.
9. **Inorganic P & S Geochemistry:** Weather parent material, equilibrate labile ↔ secondary sorption, and occlude inorganic P (`nelem >= 2`).
10. **Post-Growth Disturbances:** Apply `GRAZ`, `HARV`, `FIRE`, `TREM`, and `EROD`.

## Exact Mass Conservation Guarantee

When `SimConfig::audit_mass_balance` is enabled, every monthly step verifies:

$$
\left| \left( M_{\text{after}} - M_{\text{before}} \right) - \left( \text{In} - \text{Out} \right) \right| < 10^{-10}\ \text{g}\cdot\text{m}^{-2}
$$

for Carbon and every active nutrient element (`N`, `P`, `S`). Transfers pass through either the stack-allocated **`FlowLedger<F>`** (with automatic heap spill on extreme >128-flow bursts so no flow is ever dropped) or NaN-guarded direct atomic transfer helpers (`transfer_c_direct`, `transfer_e_direct`).
