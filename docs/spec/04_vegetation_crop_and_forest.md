# 04. Vegetation Submodels: Grassland/Crop, Forest, Savanna, and CO₂ Response

## 1. Grassland / Crop Submodel (`crop`)

### 1.1 Potential Production (`potcrp`)
During active growing months (`crpgrw == true`), potential monthly C production $P_{\text{pot}}$ ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$) is:
$$P_{\text{pot}} = \text{prdx}_1 \cdot f_T(T_{\text{soil}}) \cdot f_W\!\left(\frac{\text{avh2o}_1 + P}{\text{PET}}\right) \cdot f_{\text{shade}}(B_{\text{dead}}, \text{LAI}_{\text{tree}}) \cdot f_{\text{CO2}} \cdot f_{\text{bio}}$$
where:
- `prdx[0]` is maximum gross monthly C production ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$).
- $f_T$ uses the crop generalized Poisson curve (`ppdf[0..3]`).
- $f_W$ uses the texture-dependent linear moisture response (`pprdwc`, Manual §3.7.1 Fig. 3-9) of beginning-of-month relative moisture index $(\text{avh2o}_{\text{prev},1} + P_{\text{liq}})/\text{PET}$, capturing the reverse-texture effect via x-intercept $\text{pprpts}_1 + \text{pprpts}_2 (\theta_{\text{fc},0} - \theta_{\text{wp},0})$ reaching $1.0$ at $\text{pprpts}_3$.
- $f_{\text{shade}}$ attenuates herbaceous production by standing dead litter (`stdedc`) and, in savanna systems, tree canopy LAI (`rlai`).
- $f_{\text{bio}}$ scales growth when live biomass is below full canopy (`biomax`).

### 1.2 Root-to-Shoot Allocation
Root-to-shoot ratio $R/S$ depends on annual precipitation (or fixed crop ratio `frtc`) and shifts upward under moisture stress or after fire (`frtsh`):
$$f_{\text{root}} = \frac{R/S}{1 + R/S}, \qquad f_{\text{shoot}} = 1 - f_{\text{root}}$$

### 1.3 Nutrient Limitation and Liebig's Law (`restrp`)
For each active element $e \in \{0..N_{\text{elem}}-1\}$:
- Shoot C:E bounds $[r_{\min,\text{ag}}, r_{\max,\text{ag}}]$ interpolate linearly with live shoot biomass (`aglivc` / `biomax`) between `pramn(*,1)..pramn(*,2)` and `pramx(*,1)..pramx(*,2)`.
- Root C:E bounds $[r_{\min,\text{bg}}, r_{\max,\text{bg}}]$ interpolate with annual precipitation (`prbmn`, `prbmx`).
- Available nutrient supply equals root-zone mineral nutrient (`minerl[0..nlaypg][e]`) weighted by root impact factor `rtimp(bglivc)` plus crop internal storage `crpstg[e]`.
- Symbiotic N fixation adds up to `snfxmx[0] * P_actual` for N-fixing crops when mineral N is limiting.
- Actual C production $P_{\text{act}} = \min(P_{\text{pot}},\ \min_e P_{\max}(e))$ where $P_{\max}(e)$ is the maximum C producible at the widest allowed ratio $r_{\max}(e)$.
- Actual nutrient uptake is interpolated between minimum and maximum concentration based on available supply, preserving exact mass balance between `minerl` and plant tissues.

### 1.4 Senescence, Death, and Litterfall
- **Shoot death** (`dshoot`): Increases with drought stress (`1 - avh2o_2/awhc_total`), self-shading when `aglivc > fsdeth[3]`, or fixed senescence fraction `fsdeth[1]` during `SENM` months. Transfers `aglivc` $\to$ `stdedc` (with nutrient retranslocation to `crpstg` or soil/root).
- **Standing dead fall** (`falstd`): Transfers `fallrt * stdedc` $\to$ surface litter (`strucc[0]`/`metabc[0]`).
- **Root death** (`droot`): Active when $T_{\text{soil}} > \text{rtdtmp}$; drought-modified fraction `rdr` transfers `bglivc` $\to$ soil litter (`strucc[1]`/`metabc[1]`).

## 2. Forest and Savanna Submodel (`forest`)

### 2.1 Potential Forest Production (`potfor`)
During tree growing months (`forgrw == true`):
$$P_{\text{for,gross}} = \text{prdx}_4 \cdot f_T(T_{\text{soil}}) \cdot f_W \cdot f_{\text{LAI}}(\text{LAI}_{\text{eff}}) \cdot f_{\text{CO2}}$$
$$P_{\text{for,pot}} = \min\!\left(\max(0,\ P_{\text{for,gross}} - R_{\text{maint}}),\ \text{prdx}_3 \cdot f_T \cdot f_W \cdot f_{\text{LAI}} \cdot f_{\text{CO2}}\right)$$
where $\text{prdx}_4$ is maximum gross monthly forest C production ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$), $R_{\text{maint}}$ is Ryan (1991) $Q_{10}$ maintenance respiration of live foliage, fine roots, and live sapwood (`sapk`), leaf area index is bounded by wood allometry $\text{maxlai}\cdot \text{rlwodc}/(\text{klai} + \text{rlwodc})$, and light interception efficiency is $f_{\text{LAI}} = 1 - \exp(-|\text{laitop}|\cdot \text{LAI}_{\text{eff}})$.

### 2.2 Five-Compartment Wood/Foliage Allocation
Total forest C production is allocated across 5 live components (`fcfrac[0..4]`):
0. Leaves (`rleavc`)
1. Fine roots (`frootc`)
2. Fine branches (`fbrchc`)
3. Large wood (`rlwodc`)
4. Coarse roots (`crootc`)

Allocation can switch between juvenile (`fcfrac[*,0]`) and mature (`fcfrac[*,1]`) regimes based on total forest biomass or `basfc2`.

### 2.3 Nutrient Uptake, Storage, and Savanna Tree-Grass Partitioning
In pure forests, trees access full root-zone mineral nutrients (`minerl[0..nlaypg][e]`) plus internal tree storage `forstg[e]` and symbiotic forest N fixation (`snfxmx[1]`). In savanna systems (`SystemKind::Savanna`), available soil mineral nutrients are partitioned symmetrically between trees (`tree_n_share` from basal area `rlwodc * 2.5 / basfct` relative to `sitpot`) and understory grass (`1 - tree_n_share`) while Beer-Lambert canopy shading attenuates grass light availability. Target C:E limits are defined per component in `cerfor[min/max][comp][e]`.

### 2.4 Forest Mortality and Litterfall (`wdeath`)
- Monthly leaf drop (`leafdr[month]` for evergreen stands; hemisphere-aware once-per-autumn abscission fraction `wooddr[0]` for `decid == 1` deciduous and mixed stands, shifting seasonal months by 6 for `lat < 0`) transfers `rleavc` $\to$ surface litter (`strucc[0]`/`metabc[0]`), retranslocating a fraction `forrtf[e]` of leaf nutrients into `forstg[e]`.
- Fine root death (`wooddr[1]`) transfers `frootc` $\to$ soil litter (`strucc[1]`/`metabc[1]`).
- Fine branch death (`wooddr[2]`) $\to$ `wood1c`; large wood death (`wooddr[3]`) $\to$ `wood2c`; coarse root death (`wooddr[4]`) $\to$ `wood3c`.

## 3. Atmospheric CO₂ Fertilization and Water-Use Efficiency (`co2`)
Given atmospheric $[\text{CO}_2]$ (ppm) relative to baseline `co2ppm[0]` (default 350 ppm) and doubled reference `co2ppm[1]` (default 700 ppm), logarithmic/linear interpolation (`co2ipr`, `co2itr`, `co2ice`, `co2irs`) modulates:
1. **Production multiplier (`co2cpr`)**: enhances photosynthesis (stronger for C3 than C4).
2. **Transpiration multiplier (`co2ctr`)**: reduces stomatal conductance / transpiration per unit biomass.
3. **C:E stoichiometry multiplier (`co2cce`)**: widens plant C:N ratios under elevated CO₂.
4. **Root/shoot multiplier (`co2crs`)**: shifts allocation toward roots under elevated CO₂.

## 4. Non-Symbiotic N Inputs
- **Atmospheric N deposition**: Monthly wet + dry deposition proportional to precipitation (`epnfa[0] + epnfa[1] * ppt`), or gridded N-deposition scaler when enabled.
- **Non-symbiotic soil N fixation**: Scales with annual precipitation (`epnfs[0] + epnfs[1] * annppt`), added directly to `minerl[0][N]`.
