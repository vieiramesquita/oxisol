#!/usr/bin/env bash
# End-to-end Oxisol spatial raster workflow (MapBiomas Col. 11 + Climate + SoilGrids)
# Scales cleanly from an 8 GB laptop to a 128-256 thread HPC server.
set -euo pipefail

BIN="${OXISOL_BIN:-./target/release/oxisol}"
DATA_DIR="${1:-./sample_rasters}"

if [ ! -x "$BIN" ]; then
  echo "Building oxisol CLI (release)..."
  cargo build --release -p oxisol-cli
fi

if [ ! -f "$DATA_DIR/mapbiomas_2024.tif" ]; then
  echo "Generating sample Cerrado-Amazon GeoTIFFs in $DATA_DIR..."
  python3 examples/spatial_raster/generate_sample_rasters.py --out-dir "$DATA_DIR"
fi

echo ""
echo "=== 1) Laptop-Friendly Run: PASTURE ONLY (MapBiomas code 15, Urochloa/Panicum) ==="
"$BIN" raster \
  --lulc "$DATA_DIR/mapbiomas_2024.tif" \
  --scheme mapbiomas_br \
  --only-classes 15 \
  --precip "$DATA_DIR/terraclimate_precip_12m_mm.tif" --precip-unit mm \
  --tmin "$DATA_DIR/terraclimate_tmin_12m_c.tif" --temp-unit celsius \
  --tmax "$DATA_DIR/terraclimate_tmax_12m_c.tif" \
  --sand "$DATA_DIR/soilgrids_sand_gkg.tif" \
  --clay "$DATA_DIR/soilgrids_clay_gkg.tif" \
  --bulkd "$DATA_DIR/soilgrids_bdod_cgcm3.tif" \
  --ph "$DATA_DIR/soilgrids_phh2o_x10.tif" \
  --unit mgha \
  --spinup-years 60 \
  --years 30 \
  --tile-rows 32 \
  --out-dir ./out_pasture_only_mgha

echo ""
echo "=== 2) Multi-Year Land-Use Change (1990 -> 2005 -> 2024, All Active Classes) ==="
# Uses all available CPU threads automatically (override with --threads N or --resolution-m 10000)
"$BIN" raster \
  --lulc "$DATA_DIR/mapbiomas_1990.tif" \
  --lulc "$DATA_DIR/mapbiomas_2005.tif" \
  --lulc "$DATA_DIR/mapbiomas_2024.tif" \
  --scheme mapbiomas_br \
  --precip "$DATA_DIR/terraclimate_precip_12m_mm.tif" --precip-unit mm \
  --tmin "$DATA_DIR/terraclimate_tmin_12m_c.tif" --temp-unit celsius \
  --tmax "$DATA_DIR/terraclimate_tmax_12m_c.tif" \
  --sand "$DATA_DIR/soilgrids_sand_gkg.tif" \
  --clay "$DATA_DIR/soilgrids_clay_gkg.tif" \
  --bulkd "$DATA_DIR/soilgrids_bdod_cgcm3.tif" \
  --ph "$DATA_DIR/soilgrids_phh2o_x10.tif" \
  --unit mgha \
  --spinup-years 60 \
  --tile-rows 64 \
  --out-dir ./out_luc_1990_2024_mgha
