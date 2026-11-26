# SOM Decomposition & C/N/P/S Cycling

## Three-Pool Soil Organic Matter Concept

Oxisol partitions soil organic matter by turnover timescale (Parton et al. 1987, 1988, 1993):

- **Active SOM (`som1c[0]` surface microbes, `som1c[1]` soil active):** Turnover ~1–5 years (`dec3`), modulated by soil texture `eftext = peftxa + peftxb * sand`.
- **Slow SOM (`som2c`):** Turnover ~20–40 years (`dec5`), receiving stabilized microbial products and lignin-rich structural litter (`rsplig`).
- **Passive SOM (`som3c`):** Turnover ~200–1,000+ years (`dec4`), stabilized strongly by clay content (`fps1s3 = ps1s3[0] + ps1s3[1] * clay`, `fps2s3 = ps2s3[0] + ps2s3[1] * clay`).

## Litter Partitioning & Lignin Inhibition

Incoming plant residues split between **metabolic** (rapidly decomposable) and **structural** (lignin-bearing) litter using the lignin-to-nitrogen ratio:

$$
f_{\text{metab}} = \text{clamp}\left(\text{spl}[0] - \text{spl}[1] \cdot \frac{L}{N},\ 0.05,\ 0.85\right)
$$

Structural litter decay is exponentially inhibited by lignin content via $\exp(-\text{pligst} \cdot \text{strlig})$. In Oxisol, lignin exponentials are hoisted outside the 4-substep loop (`ntspm = 4`) for optimal CPU throughput.

## Abiotic Decomposition Modifier (`defac`) & Anaerobiosis (`anerb`)

- **Temperature factor:** Generalized Poisson/arctan response (`teff[0..3]`) evaluated on monthly soil temperature `stemp`.
- **Moisture factor:** Selected by `fix.idef` (`1` = relative water content `rwcf`, `2` = `(precip + avh2o) / pet` ratio).
- **Anaerobic suppression:** Poorly drained soils (`drain < 1.0`) reduce decomposition when precipitation exceeds PET, slowing SOM mineralization in wetlands and hydric soils.

## Nitrogen, Phosphorus & Sulfur Cycling

- **Dynamic Stoichiometry (`VARAT1..3`, `PCEMIC`):** Receiving SOM pools interpolate target C:E ratios between `[max, min]` as a function of available mineral nutrients, immobilizing mineral N/P/S when residue C:E is high or mineralizing excess nutrients when residue C:E is low.
- **Gaseous Losses (`vlossg`, `vlosse`):** Volatilization tracks both gross mineralization (`vlossg`) and monthly surface mineral N turnover (`vlosse / 12`).
- **Phosphorus & Sulfur Geochemistry (`nelem >= 2`):** Simulates parent mineral weathering (`pparmn`), dynamic labile ↔ secondary sorption (`pmnsec`, `psecmn`) modulated by cell soil pH and texture (`texesp_2`, `pparmn_2`, `psecmn_2`), irreversible P occlusion (`psecoc`), and biochemical P/S mineralization (`rad1p`).
- **Isotope Tracking (`IsotopeMode::C14` / `C13`):** Every `CarbonPool<F>` carries `(unlab, lab)` pairs, preserving exact isotopic ratios across every split, respiration, and harvest export.
