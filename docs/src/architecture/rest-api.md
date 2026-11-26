# REST JSON API Reference

When running `oxisol serve`, Oxisol exposes a typed HTTP/JSON API powered by `axum` alongside the embedded **Oxisol Studio** UI.

## Endpoints

### `GET /api/health`
Returns service liveness and version metadata:
```json
{ "status": "ok", "service": "Oxisol", "version": "0.1.0" }
```

### `GET /api/system`
Returns host hardware telemetry and attached grid status:
```json
{
  "name": "Oxisol Studio",
  "version": "0.1.0",
  "logical_cpus": 16,
  "cell_state_f64_bytes": 5952,
  "cell_state_f32_bytes": 3008,
  "has_attached_grid": false,
  "attached_grid_label": null
}
```

### `GET /api/presets`
Returns the 6 built-in calibrated ecosystem presets (`temperate_grassland`, `cerrado_savanna`, `temperate_deciduous`, `boreal_conifer`, `cropland_corn_wheat`, `semiarid_shrubland`) with bilingual EN/PT-BR metadata and default `SimulateRequest` payloads.

### `POST /api/simulate`
Executes an interactive single-cell simulation and returns annual time series (`series`) plus final compartment stocks and mass-balance audit residual (`final_pools`).

**Example (`curl`):**
```bash
curl -s http://127.0.0.1:8080/api/simulate \
  -H "Content-Type: application/json" \
  -d '{
    "preset_id": "cerrado_savanna",
    "mean_temp_c": 23.0, "temp_amp_c": 3.5, "annual_precip_cm": 145.0,
    "lat": -15.8, "co2_start_ppm": 420.0, "co2_end_ppm": 560.0,
    "sand": 0.48, "silt": 0.17, "clay": 0.35, "bulkd": 1.24,
    "ph": 5.3, "rock_frac": 0.04, "drain": 1.0,
    "years": 100, "elements": "cnps", "profile": "clean",
    "precision": "f64", "audit_mass_balance": true,
    "tillage": "none", "fertilizer_n": 0.0, "fertilizer_p": 0.0,
    "irrigation": "none", "fire_interval_yr": 5,
    "grazing_intensity": 0.0, "omad_c": 0.0
  }' | jq '.final_pools'
```

### `POST /api/grid/run`
Executes a multi-core parallel spatial grid run via `oxisol-grid` (`rayon`), returning throughput telemetry, high-performance server scaling projection, and per-cell biogeochemical states.

```bash
curl -s http://127.0.0.1:8080/api/grid/run \
  -H "Content-Type: application/json" \
  -d '{
    "use_attached_grid": false,
    "rows": 24, "cols": 48, "years": 100,
    "threads": 16, "chunk_size": 16,
    "profile": "clean", "precision": "f64"
  }' | jq '{ncells, elapsed_secs, cell_years_per_sec, mean_somsc}'
```
