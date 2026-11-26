# Oxisol Studio (Web Interface)

**Oxisol Studio** is an interactive web workbench embedded directly inside the compiled `oxisol` binary (`crates/oxisol-web`). It requires **zero Node.js, npm, or external CDN access**, making it equally suitable for air-gapped HPC clusters, modern high-performance research servers, and local laptops.

## Launching Studio

```bash
oxisol serve --host 127.0.0.1 --port 8080
```

### Remote Access on a High-Performance Server (Recommended Pattern)

Bind to loopback (`127.0.0.1`) on the remote server and forward the port over SSH — keeping the server completely protected without opening firewall ports:

```bash
# On the remote server:
oxisol serve --host 127.0.0.1 --port 8080 --data-dir /data/continental_grid

# On your local workstation:
ssh -L 8080:127.0.0.1:8080 user@hpc-server
# Then open http://127.0.0.1:8080 locally
```

## Bilingual Interface (`EN` / `PT-BR`)

Use the **`EN` / `PT-BR`** toggle in the top-right header bar to switch all labels, preset descriptions, KPIs, and hints instantaneously between English and Brazilian Portuguese.

## Workspace Tabs

### 1. Scenario Lab (Single-Cell Interactive Modeling)

Explore climate, soil, CO₂, and land-management sensitivities in real time:

- **6 Calibrated Ecosystem Presets:**
  1. *Temperate C3 Grassland (Great Plains, 40.8°N)*
  2. *Tropical Savanna / Cerrado (Southern Hemisphere, 15.8°S)* — demonstrates austral phenology and seasonal fire dynamics
  3. *Temperate Deciduous Forest (39.5°N)* — autumn foliar abscission + elevated CO₂ ramp
  4. *Boreal Coniferous Forest / Taiga (58.0°N)* — snowpack-buffered slow decomposition
  5. *Intensive Cropland (42.0°N)* — ready for comparing Moldboard Plow vs. No-Till Harvest
  6. *Semi-Arid Shrubland (35.5°N)* — water-limited steppe sensitive to irrigation and grazing
- **A vs. B Scenario Comparison ("Pin as Baseline"):**
  Click **Pin as Baseline (A)** to freeze your current run, then modify any slider (e.g., `+2 °C`, `560 ppm CO₂`, or switch from *Moldboard Plow* to *No-Till + Compost*). Studio overlays the dashed Baseline curve on all charts and renders exact `Δ` badges on every KPI card.
- **Live Biogeochemical Pool Architecture:**
  An interactive SVG diagram displays live end-of-simulation stocks across `Atmosphere CO₂`, `Live Vegetation`, `Litter & Dead Wood`, `SOM1 Active`, `SOM2 Slow`, `SOM3 Passive`, and `Profile Mineral N/P/S`.
- **Exact Mass-Balance Telemetry:**
  Every interactive run audits step-by-step C/N/P/S conservation and reports the maximum residual (`< 1e-10 g/m²` in `f64`).
- **One-Click Data Export:** Download full annual series as **CSV** or **JSON**.

### 2. Spatial Grid & HPC Explorer

Test and visualize lock-free `rayon` multi-core execution directly in your browser:

- Runs either an **attached VEMAP/spatial dataset** (`--data-dir`) or the **built-in synthetic continental gradient grid** (`rows × cols` spanning boreal → subtropical and arid → humid zones).
- Adjust **Worker Threads**, **Chunk Size**, **Years / Cell**, **Float Precision (`f64`/`f32`)**, and **Profile** live.
- Inspect real-time **Measured Throughput (`cell-yr/s`, `cell-mo/s`)** and a conservative **256-thread high-performance server projection**.
- Switch heatmap layers (`SOMSC`, `SOM3 Passive`, `Large Wood`, `Annual NPP`, `Annual AET`, `Soil Water`, `Mineral N`, `Biome ID`) and **click any cell** to inspect its full state in the Cell Inspector panel.

### 3. REST API & Docs

Provides live host hardware diagnostics (`CellState<f64>` L1d cache size, logical thread count), the endpoint reference, and copy-paste Python automation snippets.
