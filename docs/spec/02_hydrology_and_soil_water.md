# 02. Hydrology, Soil Water Retention, Leaching, and Soil Temperature

## 1. Soil Profile and Hydraulic Properties
The soil column is discretized into `nlayer` $\le 10$ layers with thicknesses $\Delta z_i$ (`adep[i]`, cm; standard VEMAP: four 15 cm upper layers and up to four 30 cm deeper layers, totaling up to 180 cm). Active plant root depth spans `nlaypg` $\le$ `nlayer` layers.

### 1.1 Water Retention Pedotransfer Functions
When soil water retention is computed from texture (`swflag`):
1. **Rawls et al. (1982)** (`swflag = 0`, VEMAP default):
   - Field capacity ($-0.1\text{ bar}$):
     $$\theta_{\text{fc}} = 0.4118 - 0.0030\,S_{\%} + 0.0023\,C_{\%} + 0.0317\,\text{OM}_{\%}$$
   - Wilting point ($-15\text{ bar}$):
     $$\theta_{\text{wp}} = 0.0260 + 0.0050\,C_{\%} + 0.0158\,\text{OM}_{\%}$$
   where $S_{\%} = 100 \cdot \text{sand}$, $C_{\%} = 100 \cdot \text{clay}$, and $\text{OM}_{\%} = \frac{\text{somsc} \cdot 1.724}{10^4 \cdot \rho_b \cdot 20\text{ cm}}$ (organic matter % in top 20 cm).
2. **Gupta & Larson (1979)** (`swflag = 1`):
   - $\theta_{\text{fc}} = 0.003075\,S_{\%} + 0.005886\,\text{Si}_{\%} + 0.008039\,C_{\%} + 0.002208\,\text{OM}_{\%} - 0.1434\,\rho_b$
   - $\theta_{\text{wp}} = -0.000059\,S_{\%} + 0.001142\,\text{Si}_{\%} + 0.005766\,C_{\%} + 0.002228\,\text{OM}_{\%} + 0.02671\,\rho_b$

### 1.2 Rock Fragment Volume Adjustment
For stony soils with volumetric rock fraction $f_{\text{rock}} \in [0, 0.90]$ and mineral soil depth $z_{\text{minrl}}$:
- If total profile depth $Z = \sum_{i=1}^{\text{nlayer}} \Delta z_i \le 150\text{ cm}$ and $z_{\text{minrl}} < Z$:
  $$f_{\text{rock,eff}} = \min\!\left(0.90,\ f_{\text{rock}} + \frac{Z - z_{\text{minrl}}}{Z}\right)$$
- If $Z > 150\text{ cm}$, $f_{\text{rock,eff}} = 0$.
- Effective volumetric water limits:
  $$\theta_{\text{fc,eff}} = \theta_{\text{fc}}(1 - f_{\text{rock,eff}}), \qquad \theta_{\text{wp,eff}} = \theta_{\text{wp}}(1 - f_{\text{rock,eff}})$$
- Per-layer available water holding capacity: $\text{awhc}_i = (\theta_{\text{fc,eff},i} - \theta_{\text{wp,eff},i}) \Delta z_i$ (cm).

## 2. Potential Evapotranspiration (PET)
Monthly PET (cm/month) is computed via **Linacre (1977)** from monthly mean air temperature $T_{\text{mean}} = \frac{T_{\min} + T_{\max}}{2}$ ($^\circ\text{C}$), diurnal range $T_{\text{range}} = T_{\max} - T_{\min}$, site latitude $\phi$ (degrees), and elevation $z_{\text{elev}}$ (m):
1. Temperature adjustment for elevation: $T_m = T_{\text{mean}} + 0.006\,z_{\text{elev}}$.
2. Dewpoint depression estimate (Linacre 1977):
   $$\Delta T_d = 0.0023\,z_{\text{elev}} + 0.37\,T_{\text{mean}} + 0.53\,T_{\text{range}} + 0.35\,T_{\text{ann\_range}} - 10.9$$
   where $T_{\text{ann\_range}}$ is the difference between warmest and coldest monthly mean temperatures.
3. Daily PET ($\text{mm}\cdot\text{d}^{-1}$):
   $$\text{PET}_d = \frac{\frac{700\,T_m}{100 - |\phi|} + 15\,\Delta T_d}{80 - T_{\text{mean}}}$$
4. Monthly PET (cm/month), clamped to $\ge \text{fwloss}_4 \cdot \text{min\_pet}$:
   $$\text{PET} = \max\!\left(0.5,\ \text{PET}_d \cdot \frac{30}{10}\right) \cdot \text{fwloss}_4$$
   with a daylight-length scaling factor (`petdly` / astronomical daylength from latitude $\phi$ and month day-of-year).

### Astronomical Daylength
Given solar declination $\delta = 0.4014\,\sin\!\left(\frac{2\pi (\text{DOY} - 77)}{365}\right)$ and latitude $\phi_{\text{rad}} = \phi \frac{\pi}{180}$:
- `ScientificClean`: $\cos H_0 = \text{clamp}(-\tan \phi_{\text{rad}} \cdot \tan \delta,\ -1,\ 1)$, $\text{daylen} = \frac{24}{\pi}\arccos(\cos H_0)$ (note the legacy V4.5 bug omitting $\tan(\delta)$ is documented in Spec 07).

## 3. Monthly Water Balance
1. **Snow Accumulation and Melt**:
   - If $T_{\text{mean}} \le 0^\circ\text{C}$, precipitation accumulates into `snow` pack; sublimation reduces snow up to $0.87\,\text{PET}$.
   - If $T_{\text{mean}} > 0^\circ\text{C}$, snowmelt $M = \min(\text{snow},\ \text{tmelt}_2 (T_{\text{mean}} - \text{tmelt}_1))$ enters liquid water input `winputs`.
2. **Canopy & Litter Interception**:
   - Interception $I = \min\!\left(P_{\text{liq}},\ \text{PET},\ \text{fwloss}_1 \cdot P_{\text{liq}} \cdot (a_1 + b_1 B_{\text{live}} + c_1 B_{\text{dead}})\right)$ (clamped to available water and PET).
3. **Infiltration, Redistribution, and Runoff**:
   - Remaining liquid input infiltrates layer by layer from top ($i=0$) to bottom (`nlayer-1`).
   - Excess above field capacity $\text{fc}_i = \theta_{\text{fc},i}\Delta z_i$ drains into layer $i+1$.
   - Excess leaving the bottom layer (`nlayer-1`) enters deep storage (`asmos[nlayer]`) or splits into stormflow (`stormf`), baseflow (`basef`), and deep drainage (`drain`).
4. **Bare Soil Evaporation**:
   - Limited by remaining PET, shading by live/dead biomass, and surface water in layer 0 above a residual threshold:
     $$E_{\text{soil}} = \text{fwloss}_2 \cdot \text{PET}_{\text{rem}} \cdot \exp(-0.0035\,B_{\text{total}}) \cdot f_{\text{evap}}(\theta_0)$$
5. **Transpiration**:
   - Driven by live biomass (`aglivc`, `rleavc`), `fwloss_3`, CO₂ stomatal conductance modifier (`co2ctr`), and remaining PET, extracted proportionally to available water across rooting layers `0..nlaypg`.
6. **Available Water Indices**:
   - `avh2o[0]` (plant-available water in root zone `0..nlaypg`)
   - `avh2o[1]` (plant-available water in full profile `0..nlayer`)
   - `avh2o[2]` (plant-available water in top 30 cm — used by decomposition moisture modifier `wfunc`)

## 4. Solute Leaching
When vertical water flux $F_{\text{leach}}$ (cm/month) exits the active leaching layer (`nlaypg` or 30 cm boundary) and exceeds critical flux `h2olec` (`cm`):
- Fraction of mineral nutrient leached (Parton et al. 1988):
  $$f_{\text{leach}}(e) = \text{clamp}\!\left(\frac{F_{\text{leach}} - h_{\text{crit}}}{18.0},\ 0,\ 1\right) \cdot \text{fleach}(e, \text{sand})$$
  where $\text{fleach}(N) = \text{fleach}_1 + \text{fleach}_2 \cdot \text{sand}$ increases in sandy soils; P leaching is small (`fleach_3`); S leaching scales similarly (`fleach_4 + fleach_5 * sand`).
- Organic C/N/P/S leaching from `som1c[1]` occurs when water flux exceeds `omlech_3`, scaled by `omlech_1 + omlech_2 * sand`.

## 5. Soil Temperature (`soil_temp`)
Near-surface monthly soil temperature $T_{\text{soil}}$ ($^\circ\text{C}$, 0–10 cm) is estimated from monthly air temperature and insulating plant biomass (`aglivc + stdedc + wood_litter`):
- Biomass insulation attenuates both maximum and minimum extremes:
  $$T_{\text{soil,max}} = T_{\max} + (25.4 / (1 + 18\,e^{-0.20 T_{\max}})) \cdot e^{-0.0035 B_{\text{surf}}} - 0.13\,T_{\max}$$
  $$T_{\text{soil,min}} = T_{\min} + 0.006\,B_{\text{surf}} - 1.82$$
  (bounded so $T_{\text{soil,min}} \le T_{\text{soil,max}}$), and $T_{\text{soil}} = \frac{1}{2}(T_{\text{soil,max}} + T_{\text{soil,min}})$.
- Under snow cover (`snow > 0`), $T_{\text{soil}}$ is buffered near $0^\circ\text{C}$.
