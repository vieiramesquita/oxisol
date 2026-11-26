# Spatial Raster Examples (MapBiomas + Modeled Climate + SoilGrids)

Run **Oxisol** directly over georeferenced **GeoTIFF / Cloud-Optimized GeoTIFF (COG) / NetCDF** rasters — **100% pure Rust (`terrio`), zero `libgdal` / `PROJ` C runtime dependencies** — exporting **9 DEFLATE-compressed COGs** directly in **`Mg C/ha`** (`0–20 cm` CENTURY Classic, **`0–30 cm` IPCC Tier 2/3**, and **`0–100 cm` Deep Oxisol Profile**).

---

## Hardware Scaling: From an 8 GB Laptop to a 256-Thread HPC Server

Oxisol is designed so **anyone on a standard laptop can explore regional scenarios comfortably**, while **modern multi-socket servers automatically saturate all CPU cores** with bounded memory via row-strip tiling (`--tile-rows`):

| Hardware Tier | Recommended Flags | Typical Use Case |
|---|---|---|
| **Standard Laptop (4–8 cores, 8–16 GB RAM)** | `--resolution-m 1000 --only-classes 15 --tile-rows 32 --threads 4` | Fast municipality/watershed pasture or LUC exploration in seconds; keeps CPU cool & RAM low |
| **Workstation (16–32 cores, 32–64 GB RAM)** | `--resolution-m 250 --tile-rows 64` | State-level or biome-wide 250 m simulations in minutes |
| **Modern High-Performance Server (64–256+ threads)** | `--tile-rows 128 --threads $(nproc)` *(native resolution)* | Full multi-million-pixel continental/biome runs at near-linear multi-core scaling |

> **Tip:** In **Oxisol Studio** (`http://127.0.0.1:8080`, Tab 3: *LULC & Raster*), use the **Modeling Resolution** dropdown (`Native`, `250 m`, `1 km`, `5 km`, `10 km`), **Compute Profile** (`Max Performance` vs. `Laptop Quiet`), and **Filter Classes** box (`15` for Pasture only, or `3,4,15` for Forest + Savanna + Pasture).

---

## Required & Optional Input Rasters (`EPSG:4326` recommended)

> **Automatic Cross-Resolution Sampling:** Climate (`4–10 km`) and SoilGrids (`250 m`) rasters **do NOT need to match the pixel grid or resolution of your LULC raster**. The LULC raster defines the modeling grid (optionally coarsened via **majority-mode aggregation** with `--resolution-m`), and Oxisol samples climate and soil automatically at every pixel's `(lon, lat)` coordinate.

| Layer | Flag | Bands | Supported Units | Open Data Sources |
|---|---|---|---|---|
| **LULC Map(s)** *(required)* | `--lulc` *(repeat per year for dynamic LUC)* | `1` (integer codes) | **MapBiomas Brasil Col. 11** (all 41 official classes: `3` Forest, `4` Savanna, `15` Pasture *Urochloa/Panicum*, `39` Soybean, etc.) or **ESA WorldCover** | [MapBiomas Brasil](https://brasil.mapbiomas.org/), ESA WorldCover |
| **Monthly Precipitation** | `--precip` | `12` *(climatology)* or `12 × N` *(yearly series)* | `--precip-unit mm` *(default)*, `cm`, `meters` (ERA5) | **TerraClimate** (4 km, [Abatzoglou et al., 2018](https://doi.org/10.1038/sdata.2017.191)), **BR-DWGD** (10 km, [Xavier et al., 2022](https://doi.org/10.1002/joc.7731)), **ERA5-Land**, **CHIRPS**, **WorldClim 2.1** |
| **Monthly Tmin** | `--tmin` | `12` or `12 × N` | `--temp-unit celsius` *(default)*, `kelvin` (ERA5/CMIP6), `celsius_x10` (TerraClimate) | TerraClimate (`tmmn`), BR-DWGD, ERA5-Land, WorldClim 2.1 |
| **Monthly Tmax** | `--tmax` | `12` or `12 × N` | `--temp-unit celsius`, `kelvin`, `celsius_x10` | TerraClimate (`tmmx`), BR-DWGD, ERA5-Land, WorldClim 2.1 |
| **Soil Sand / Clay / BulkD / pH** *(optional)* | `--sand`, `--clay`, `--bulkd`, `--ph` | `1` (`0–15` or `0–20 cm`) | Auto-scales **SoilGrids** integer conventions (`g/kg`, `cg/cm³`, `pH×10`) or standard fractions | SoilGrids 250 m, MapBiomas Solo |

---

## Quickstart (Zero External Downloads — Runs in ~15 Seconds!)

### 1. Generate Realistic Sample GeoTIFFs (Cerrado–Amazon Frontier, 1990–2024)
```bash
pip install rasterio numpy
python3 examples/spatial_raster/generate_sample_rasters.py --out-dir ./sample_rasters
```

### 2. Run **Pasture-Only** Simulation (`--only-classes 15`, *Urochloa brizantha / Panicum maximum*)
```bash
# Laptop-friendly run (only MapBiomas Pasture code 15, outputs in Mg C/ha)
./target/release/oxisol raster \
  --lulc ./sample_rasters/mapbiomas_2024.tif \
  --scheme mapbiomas_br \
  --only-classes 15 \
  --precip ./sample_rasters/terraclimate_precip_12m_mm.tif --precip-unit mm \
  --tmin ./sample_rasters/terraclimate_tmin_12m_c.tif --temp-unit celsius \
  --tmax ./sample_rasters/terraclimate_tmax_12m_c.tif \
  --sand ./sample_rasters/soilgrids_sand_gkg.tif \
  --clay ./sample_rasters/soilgrids_clay_gkg.tif \
  --bulkd ./sample_rasters/soilgrids_bdod_cgcm3.tif \
  --ph ./sample_rasters/soilgrids_phh2o_x10.tif \
  --unit mgha \
  --spinup-years 60 \
  --years 30 \
  --tile-rows 32 \
  --out-dir ./out_pasture_mgha
```

### 3. Run Multi-Year Land-Use Change (`1990 → 2005 → 2024`)
```bash
bash examples/spatial_raster/run_pasture_cerrado.sh
```

### 4. Customize Pasture Grazing & Fertilization via Python REST API
Start Oxisol Studio (`./target/release/oxisol serve --port 8080`) and run:
```bash
python3 examples/spatial_raster/run_spatial_api.py
```

### 5. Export Real Data from Google Earth Engine
Copy [`gee_export_inputs.js`](gee_export_inputs.js) into the [Google Earth Engine Code Editor](https://code.earthengine.google.com/), draw or specify your study area (municipality, watershed, or biome), and export ready-to-use MapBiomas + TerraClimate + SoilGrids GeoTIFFs directly to Google Drive!

---

## Exported Cloud-Optimized GeoTIFFs (9 Layers, DEFLATE-Compressed)

All outputs inherit the exact CRS and geotransform of the modeled LULC grid (default unit: **`Mg C/ha`**, or pass `--unit gm2` for `g C/m²`):

| File | Description | Depth / Standard |
|---|---|---|
| `oxisol_somsc_0_20_mgha.tif` | Final Soil Organic Carbon (`0–20 cm`) | Classic CENTURY V4 |
| `oxisol_somsc_0_30_ipcc_mgha.tif` | Final Soil Organic Carbon (`0–30 cm`) | **IPCC Tier 2/3 & MapBiomas Solo** |
| `oxisol_somsc_0_100_deep_mgha.tif` | Final Deep Soil Organic Carbon (`0–100 cm`) | **Deep Profile (Forests & *Urochloa* Pastures)** |
| `oxisol_delta_somsc_0_20_mgha.tif` | Net SOC Change `ΔSOMSC` (`0–20 cm`) | Multiply by `3.667` (`44/12`) for `t CO₂e/ha` |
| `oxisol_delta_somsc_0_30_ipcc_mgha.tif` | Net IPCC `0–30 cm` SOC Change | IPCC LUC Accounting |
| `oxisol_biomass_c_mgha.tif` | Live Vegetation Biomass Carbon (shoots + roots + wood) | Above + Belowground |
| `oxisol_npp_mgha.tif` | Final-Year Annual Net Primary Production (`Mg C/ha/yr`) | Ecosystem Flux |
| `oxisol_aet_cm.tif` | Final-Year Actual Evapotranspiration (`cm/yr`) | Water Balance |
| `oxisol_minerl_n_gm2.tif` | Surface Mineral Nitrogen (`g N/m²`) | Soil Fertility |

---

## Multi-Depth Soil Carbon Scientific References (`0–30 cm` & `0–100 cm`)
- **Jackson, R. B., Canadell, J., Ehleringer, J. R., Mooney, H. A., Sala, O. E., & Schulze, E. D. (1996).** A global analysis of root distributions for terrestrial biomes. *Oecologia*, 108, 389–411. https://doi.org/10.1007/BF00333714
- **Jobbágy, E. G., & Jackson, R. B. (2000).** The vertical distribution of soil organic carbon and its relation to climate and vegetation. *Ecological Applications*, 10(2), 423–436. https://doi.org/10.1890/1051-0761(2000)010[0423:TVDOSO]2.0.CO;2
- **Fisher, M. J., Rao, I. M., Ayarza, M. A., Lascano, C. E., Sanz, J. I., Thomas, R. J., & Vera, R. R. (1994).** Carbon storage by introduced deep-rooted grasses in the South American savannas. *Nature*, 371, 236–238. https://doi.org/10.1038/371236a0
- **Koven, C. D., Riley, W. J., Subin, Z. M., Tang, J. Y., Torn, M. S., et al. (2013).** The effect of vertically resolved soil biogeochemistry and alternate soil C and N models on C dynamics of CLM4. *Biogeosciences*, 10, 7109–7131. https://doi.org/10.5194/bg-10-7109-2013
- **Abatzoglou, J. T., Dobrowski, S. Z., Parks, S. A., & Hegewisch, K. C. (2018).** TerraClimate, a high-resolution global dataset of monthly climate and climatic water balance from 1958–2015. *Scientific Data*, 5, 170191. https://doi.org/10.1038/sdata.2017.191
- **Xavier, A. C., Scanlon, B. R., King, C. W., & Alves, A. I. (2022).** New improved Brazilian daily weather gridded data (1961–2020). *International Journal of Climatology*, 42(16), 8390–8404. https://doi.org/10.1002/joc.7731
