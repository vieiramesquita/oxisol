# Formal Clean-Room Specification Appendix

For researchers auditing exact equation forms, parameter definitions, and documented V4.0 → V4.5 deltas, the formal 7-part mathematical specification resides directly in [`docs/spec/`](https://github.com/vieiramesquita/oxisol/tree/main/docs/spec):

1. **`01_pools_and_mass_balance.md`** — Pool state definitions, isotopic `(unlab, lab)` representation, and boundary conservation identities.
2. **`02_hydrology_and_soil_water.md`** — Linacre PET, snowpack melt/drainage, Probert runoff, Rawls/Gupta-Larson pedotransfer equations, and solute leaching.
3. **`03_decomposition_and_som.md`** — 4-substep structural/metabolic/wood/SOM1/SOM2/SOM3 decay equations, lignin controls, and texture stabilization.
4. **`04_vegetation_crop_and_forest.md`** — Herbaceous & forest production, savanna competition, Ryan maintenance respiration, and hemisphere-aware deciduous phenology.
5. **`05_phosphorus_and_sulfur.md`** — Weathering, sorption equilibrium, P occlusion, and biochemical mineralization.
6. **`06_management_and_scheduling.md`** — Formal specifications of all 9 management disturbances and `.sch` repeating blocks.
7. **`07_compatibility_profiles.md`** — Complete catalog of `ScientificClean` vs. `VemapV45Parity` scientific deltas.

## Rust API Reference (`rustdoc`)

Browse the auto-generated Rust API documentation for every crate:
- [`oxisol_core`](../api/oxisol_core/index.html)
- [`oxisol_io`](../api/oxisol_io/index.html)
- [`oxisol_grid`](../api/oxisol_grid/index.html)
- [`oxisol_web`](../api/oxisol_web/index.html)
