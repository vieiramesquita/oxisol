# Hydrology, PET & Soil Temperature

## Potential Evapotranspiration (Linacre 1977)

Oxisol computes monthly sea-level potential evapotranspiration (cm/month) following Linacre (1977):

- Monthly mean air temperature $T_m = \frac{T_{\min} + T_{\max}}{2}$
- Annual temperature range $R_a = T_{\text{warmest}} - \max(T_{\text{coldest}}, -10^\circ\text{C})$
- Daily PET scaled by days-in-month and `fix.fwloss[3]`.

## Astronomical Daylength (Hemisphere-Aware)

Daylength is computed from solar declination $\delta$ and latitude $\phi$ in radians:

- Under `CompatibilityProfile::ScientificClean`, uses the exact spherical astronomy relation $\cos(H_0) = -\tan(\phi)\tan(\delta)$ with polar day/night clamping.
- Under `CompatibilityProfile::VemapV45Parity`, preserves the documented legacy `daylen` approximation for exact historical auditing.

## Soil Water Retention (Pedotransfer Functions)

Per-cell field capacity (`afiel`) and wilting point (`awilt`) are computed once during `CellDerivedParams::compute` without mutating shared biome tables:

- Supports explicit `site.100` curves (`swflag = 0`), Gupta & Larson (1979) (`swflag = 1, 2`), and Rawls et al. (1982) (`swflag = 3, 4`).
- Applies volumetric rock-fraction and mineral-depth scaling `(1 - effective_rockfrac)`.
- Under `ScientificClean`, top-layer soil organic matter percentage (`ompc`) enters the pedotransfer equation cleanly; under `VemapV45Parity`, `ompc = 0` matches the historical initialization ordering.

## Multi-Layer Water Balance & Solute Leaching

1. **Snowpack:** Precipitation falls as snow below freezing; snowmelt activates when mean air temperature exceeds `tmelt[0]` (even below 0 °C) and drains via liquid snow storage (`snlq`).
2. **Pre-Infiltration Losses:** Canopy interception (`fwloss[0]`) and bare-soil evaporation (`fwloss[1]`, suppressed under snowpack) act on liquid water before infiltration — guaranteeing surface soil moisture never erroneously dries below wilting point during winter.
3. **Runoff:** Excess surface water follows the Probert et al. (1995) runoff curve (suppressed on snowpack).
4. **Infiltration & Percolation:** Water cascades through up to 10 soil layers (`adep[0..nlayer]`); excess above field capacity becomes layer saturated flux `amov[l]`, driving mineral solute leaching proportional to `clamp(amov[l] / minlch, 0, 1) * (fleach[0] + fleach[1]*sand) * fleach[e+2]` and organic C leaching proportional to `amov_30cm / omlech[2]`.
5. **Soil Temperature:** Surface biomass (`aglivc + stdedc + litter`) insulates soil maxima and minima via `pmxtmp`, `pmntmp`, and `pmxbio`, with snowpack buffering soil temperatures toward 0 °C.
