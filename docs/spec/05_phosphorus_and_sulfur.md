# 05. Phosphorus (P) and Sulfur (S) Geochemical Cycles

## 1. Scope
When `ElementMode` is `CarbonNitrogenPhosphorus` ($N_{\text{elem}} \ge 2$) or `FullCnps` ($N_{\text{elem}} = 3$), CENTURY simulates coupled organic mineralization/immobilization (Spec 03) alongside inorganic geochemical transformations (`pschem`, Parton et al. 1988; Metherell et al. 1993).

## 2. Phosphorus Geochemical Submodel ($e = 1$)
Inorganic soil P is partitioned among four pools:
1. **Labile mineral P** (`minerl[layer][P]`): Immediately plant-available phosphate in soil solution and rapidly exchangeable surfaces.
2. **Parent material P** (`parent[P]`): Primary apatite/mineral P in unweathered rock/soil.
3. **Secondary P** (`secndy[P]`): Sorbed/precipitated Fe/Al/Ca phosphates in slow reversible equilibrium with labile P.
4. **Occluded P** (`occlud`): Very slowly exchangeable occluded phosphates.

### 2.1 pH-Dependent Solubility (`fsfunc`)
Phosphorus availability and sorption equilibria depend on soil pH via a piecewise-linear response `fsfunc(minerl_P, pH, texesp)`:
- Sorption rate from labile P to secondary P (`p_lab_to_sec`) scales with soil texture (`texesp[0] + texesp[1] * clay`) and pH.
- Desorption rate from secondary P back to labile P (`p_sec_to_lab`) is controlled by `psecmn[0]`, `psecmn[1]` (texture/pH modified), and `defac`.

### 2.2 Monthly P Geochemical Fluxes
1. **Parent Weathering**:
   $$F_{\text{par} \to \text{lab}} = \text{pparmn}_1 \cdot \text{parent}[P] \cdot \text{defac} \cdot \Delta t$$
   (in `CellDerivedParams`, `pparmn_2` reflects clay/pH modulation without mutating shared biome parameters).
2. **Labile $\leftrightarrow$ Secondary Equilibrium**:
   - Sorption: $F_{\text{lab} \to \text{sec}} = k_{\text{sorb}} \cdot \text{minerl}[0][P] \cdot \Delta t$
   - Desorption: $F_{\text{sec} \to \text{lab}} = k_{\text{desorb}} \cdot \text{secndy}[P] \cdot \text{defac} \cdot \Delta t$
3. **Secondary $\leftrightarrow$ Occluded Exchange**:
   - Occlusion: $F_{\text{sec} \to \text{occ}} = \text{psecoc}_1 \cdot \text{secndy}[P] \cdot \text{defac} \cdot \Delta t$
   - De-occlusion: $F_{\text{occ} \to \text{sec}} = \text{psecoc}_2 \cdot \text{occlud} \cdot \text{defac} \cdot \Delta t$
4. **Biochemical Mineralization of Organic P**:
   In addition to biological C-coupled mineralization, phosphatase activity directly mineralizes ester-bonded organic P from Slow SOM (`som2e[P]`) and Passive SOM (`som3e[P]`) when labile P is low (`biop` response curve, McGill & Cole 1981).

## 3. Sulfur Geochemical Submodel ($e = 2$)
Sulfur shares the organic pool structure and includes:
1. **Parent Weathering**: $F_{\text{par} \to \text{lab}}(S) = \text{sparmn} \cdot \text{parent}[S] \cdot \text{defac} \cdot \Delta t$.
2. **Labile $\leftrightarrow$ Secondary Adsorption**: Reversible sulfate adsorption between `minerl[0][S]` and `secndy[S]`.
3. **Atmospheric S Deposition & Irrigation S**: Monthly inputs from precipitation and irrigation water (`sirri`).
4. **Biochemical S Mineralization**: Sulfatase-mediated direct mineralization of ester-S from Slow and Passive SOM when mineral sulfate is depleted.

## 4. Conservation Guarantee
All geochemical transfers between `parent`, `minerl`, `secndy`, `occlud`, and SOM organic P/S pools are routed exclusively through `FlowLedger<F>`, guaranteeing exact conservation of total P and total S at every monthly step.
