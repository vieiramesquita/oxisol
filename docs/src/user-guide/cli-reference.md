# CLI Reference (`oxisol`)

The `oxisol` binary provides four subcommands: `serve`, `run`, `validate`, and `bench`.

## `oxisol serve`

Launch the embedded **Oxisol Studio** web interface and HTTP/JSON REST API.

```bash
oxisol serve [OPTIONS]
```

| Flag | Default | Description |
|---|---|---|
| `--host <ADDR>` | `127.0.0.1` | Bind IP address (`0.0.0.0` for LAN access) |
| `--port <PORT>` | `8080` | TCP listening port |
| `--data-dir <PATH>` | *(none)* | Optional spatial/VEMAP dataset folder to expose in Spatial Grid Explorer |
| `--rock <PATH>` | *(none)* | Optional rock fraction (%) `.v2` ASCII grid |
| `--zminrl <PATH>` | *(none)* | Optional mineral soil depth (cm) `.v2` ASCII grid |

---

## `oxisol run`

Execute a multi-core spatial grid simulation (equilibrium spinup or historical/future transient run).

```bash
oxisol run --data-dir <DIR> [OPTIONS]
```

| Flag | Default | Description |
|---|---|---|
| `--data-dir <PATH>` | *(required)* | Directory containing `.100`, `.sch`, `.v2`, `geog.nc`, `phase1.ncdf` |
| `--rock <PATH>` | *(none)* | Rock fraction (%) `.v2` grid |
| `--zminrl <PATH>` | *(none)* | Mineral depth (cm) `.v2` grid |
| `--start-year <INT>` | `1` | Starting calendar/simulation year |
| `--years <INT>` | `1000` | Number of simulation years |
| `--co2-file <PATH>` | *(none)* | Annual atmospheric CO₂ table (`co2.dat`, `year ppm`) |
| `--transient-ppt <PATH>` | *(none)* | Transient monthly precipitation NetCDF-3 (`pptT3.nc`, mm) |
| `--transient-tn <PATH>` | *(none)* | Transient monthly min temp NetCDF-3 (`tnT3.nc`, °C) |
| `--transient-tx <PATH>` | *(none)* | Transient monthly max temp NetCDF-3 (`txT3.nc`, °C) |
| `--restart-checkpoint <PATH>` | *(none)* | Lossless native JSON checkpoint from a prior spinup |
| `--profile <clean\|parity>` | `clean` | `clean` (`ScientificClean`) or `parity` (`VemapV45Parity`) |
| `--precision <f64\|f32>` | `f64` | IEEE-754 working precision |
| `--threads <N>` | *(all CPUs)* | Number of `rayon` worker threads |
| `--chunk-size <N>` | `16` | Cells per parallel work-stealing chunk |
| `--out-nc <PATH>` | *(none)* | Output `site.nc`-compatible NetCDF-3 file |
| `--out-checkpoint <PATH>` | *(none)* | Output lossless native JSON checkpoint |
| `--max-cells <N>` | *(all)* | Limit cell count for quick smoke tests |
| `--biome <ID>` | *(all)* | Filter simulation to a single biome ID |

---

## `oxisol validate`

Compare an `oxisol` NetCDF-3 `site.nc` output against a reference/oracle `site.nc` file, reporting mean, median relative error, P99 relative error, max error, and RMSE across hydraulic, carbon, nitrogen, and vegetation pools.

```bash
oxisol validate \
  --rust-out spinup_parity.nc \
  --oracle-out reference_site.nc \
  --max-rel-err 0.15
```

Exits with code `0` if within threshold (or if `--max-rel-err` is omitted), and code `2` if exceeded.

---

## `oxisol bench`

Benchmark single-thread serial vs. `rayon` parallel vs. 8-wide SIMD `f32x8` SOM kernel throughput on your machine:

```bash
# Synthetic benchmark (no external data needed)
oxisol bench --years 1000

# Real spatial dataset benchmark
oxisol bench --data-dir /path/to/vemap_data --years 1000 --threads 16
```
