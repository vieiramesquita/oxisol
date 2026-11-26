# 03. Decomposition, Soil Organic Matter, and Mineralization

## 1. Abiotic Decomposition Factor (`defac`)
All microbial decay rates are multiplied by the dimensionless monthly abiotic factor:
$$\text{defac} = f_T(T_{\text{soil}}) \cdot f_W\!\left(\frac{\text{avh2o}_3 + P_{\text{eff}}}{\text{PET}}\right)$$

### 1.1 Temperature Response $f_T(T_{\text{soil}})$
Using the CENTURY generalized Poisson density / arctangent response (`teff[0..3]`):
- **Arctangent form (V4.0/V4.5 standard when `teff` parameters specify `tcalc`)**:
  $$f_T(T) = \text{clamp}\!\left(t_1 + \frac{t_2}{\pi}\arctan\!\left(\pi\,t_3\,(T - t_4)\right),\ 0.01,\ 1.0\right)$$
  normalized so $f_T(30^\circ\text{C}) \approx 1.0$, or generalized Poisson density function (`gpdf`) when parameterized with $(T_{\text{opt}}, T_{\max}, a, b)$:
  $$f_T(T) = \left(\frac{T_{\max} - T}{T_{\max} - T_{\text{opt}}}\right)^a \exp\!\left(\frac{a}{b}\left(1 - \left(\frac{T_{\max} - T}{T_{\max} - T_{\text{opt}}}\right)^b\right)\right)$$

### 1.2 Moisture Response $f_W(r)$
Let $r = \frac{\text{avh2o}_3 + P_{\text{liq}}}{\max(\text{PET}, 0.01)}$ be the ratio of available surface water plus precipitation to PET:
$$f_W(r) = \frac{1}{1 + 30\,\exp(-8.5\,r)}$$
(clamped to $[0.0, 1.0]$). Under poor drainage (`drain < 1.0`), excessive water triggers an **anaerobic reduction factor** `anerb` $\in [\text{aneref}_3, 1.0]$ that reduces soil decomposition when $r > \text{aneref}_1$.

## 2. Litter, Dead Wood, and SOM Decay Rates
Each pool decays via first-order kinetics with monthly step $\Delta t = \frac{1}{12}$ (or 4 sub-monthly steps $\Delta t_{\text{sub}} = \frac{1}{48}$ for `somdec` stability):

| Pool | Maximum Annual Rate ($k_{\max}$, $\text{yr}^{-1}$) | Modifiers |
|---|---|---|
| Surface structural (`strucc[0]`) | `dec1[0]` ($\approx 3.9$) | $\text{defac} \cdot e^{-\text{pligst}_1 L_0} \cdot \text{clteff}_1$ |
| Soil structural (`strucc[1]`) | `dec1[1]` ($\approx 4.9$) | $\text{defac} \cdot e^{-\text{pligst}_2 L_1} \cdot \text{anerb} \cdot \text{clteff}_1$ |
| Surface metabolic (`metabc[0]`) | `dec2[0]` ($\approx 14.8$) | $\text{defac} \cdot \text{clteff}_2$ |
| Soil metabolic (`metabc[1]`) | `dec2[1]` ($\approx 18.5$) | $\text{defac} \cdot \text{anerb} \cdot \text{clteff}_2$ |
| Surface microbes (`som1c[0]`) | `dec3[0]` ($\approx 6.0$) | $\text{defac} \cdot \text{clteff}_3$ |
| Soil active SOM (`som1c[1]`) | `dec3[1]` ($\approx 7.3$) | $\text{defac} \cdot \text{eftext} \cdot \text{anerb} \cdot \text{clteff}_3$ |
| Slow SOM (`som2c`) | `dec5` ($\approx 0.2$) | $\text{defac} \cdot \text{anerb} \cdot \text{clteff}_4$ |
| Passive SOM (`som3c`) | `dec4` ($\approx 0.0045$) | $\text{defac} \cdot \text{anerb}$ |
| Fine branch wood (`wood1c`) | `decw1` | $\text{defac} \cdot e^{-\text{pligst}_1 L_{w1}}$ |
| Large dead wood (`wood2c`) | `decw2` | $\text{defac} \cdot e^{-\text{pligst}_1 L_{w2}}$ |
| Coarse dead root (`wood3c`) | `decw3` | $\text{defac} \cdot e^{-\text{pligst}_2 L_{w3}} \cdot \text{anerb}$ |

### Texture Effects (Parton et al. 1987, 1993)
- Active SOM turnover multiplier: $\text{eftext} = \text{peftxa} + \text{peftxb} \cdot \text{sand}$ (default $0.25 + 0.75\cdot\text{sand}$).
- Active SOM respiration fraction: $p_{1\text{co2}}(2) = \text{p1co2a}_2 + \text{p1co2b}_2 \cdot \text{sand}$ (default $0.17 + 0.68\cdot\text{sand}$).
- Active $\to$ Passive transfer fraction: $f_{s1 \to s3} = \text{ps1s3}_1 + \text{ps1s3}_2 \cdot \text{clay}$.
- Slow $\to$ Passive transfer fraction: $f_{s2 \to s3} = \text{ps2s3}_1 + \text{ps2s3}_2 \cdot \text{clay}$.

## 3. Carbon Routing and Heterotrophic Respiration
- **Structural Litter**:
  - Lignin fraction $L$ flows directly to Slow SOM (`som2c`) with respiration fraction `rsplig` ($\approx 0.30$).
  - Non-lignin fraction $(1 - L)$ flows to Surface Microbes (`som1c[0]`) for surface litter or Active SOM (`som1c[1]`) for soil litter, with respiration fractions `ps1co2[0]` ($\approx 0.45$) and `ps1co2[1]` ($\approx 0.55$).
- **Metabolic Litter**:
  - Flows to `som1c[0]` (surface) or `som1c[1]` (soil) with respiration fractions `pmco2[0]` and `pmco2[1]` ($\approx 0.55$).
- **Surface Microbes (`som1c[0]`)**:
  - Flows to Slow SOM (`som2c`) with respiration fraction `p1co2a[0]` ($\approx 0.60$).
- **Active SOM (`som1c[1]`)**:
  - Respired fraction: $p_{1\text{co2}}(2)$.
  - Leached organic fraction: $f_{\text{omleach}}$ (if water flux exceeds threshold).
  - Stabilized to Passive SOM (`som3c`): $f_{s1 \to s3}$.
  - Remainder flows to Slow SOM (`som2c`): $1 - p_{1\text{co2}}(2) - f_{\text{omleach}} - f_{s1 \to s3}$.
- **Slow SOM (`som2c`)**:
  - Respired fraction: `p2co2` ($\approx 0.55$).
  - Stabilized to Passive SOM (`som3c`): $f_{s2 \to s3}$.
  - Remainder flows to Active SOM (`som1c[1]`): $1 - \text{p2co2} - f_{s2 \to s3}$.
- **Passive SOM (`som3c`)**:
  - Respired fraction: `p3co2` ($\approx 0.55$).
  - Remainder flows to Active SOM (`som1c[1]`): $1 - \text{p3co2}$.

## 4. Nutrient Mineralization and Immobilization (`candec`)
When carbon $\Delta C$ flows from donor pool $A$ (with ratio $r_A(e) = C_A / E_A$) to receiver pool $B$ (target ratio $r_B(e)$ determined linearly from soil mineral pool `minerl[0][e]` between `varat[min]` and `varat[max]`):
- Donor releases nutrient $E_{\text{rel}} = \Delta C / r_A(e)$.
- Receiver requires nutrient $E_{\text{req}} = \Delta C_{\text{net}} / r_B(e)$ where $\Delta C_{\text{net}} = (1 - f_{\text{resp}})\Delta C$.
- If $E_{\text{rel}} \ge E_{\text{req}}$: net **mineralization** of $(E_{\text{rel}} - E_{\text{req}})$ into `minerl[0][e]` (with N volatilization fraction `vlossg` on gross mineralization).
- If $E_{\text{rel}} < E_{\text{req}}$: net **immobilization** of $(E_{\text{req}} - E_{\text{rel}})$ from `minerl[0][e]`.
- **Decomposition constraint (`candec`)**: If available mineral nutrient in layer 0 is less than required immobilization, the decomposition flow $\Delta C$ is scaled down (or halted) so `minerl[0][e]` never becomes negative!
