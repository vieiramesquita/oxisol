#!/usr/bin/env python3
"""Automate Oxisol spatial raster simulations via the embedded Axum REST API.

Demonstrates running ONLY Brazilian Cerrado Pasture (MapBiomas Col. 11 code 15)
with custom grazing intensity & nitrogen fertilization, exporting Mg C/ha COGs
for 0-20 cm, 0-30 cm (IPCC), and 0-100 cm (Deep Oxisol profile).

Start the server first:
    ./target/release/oxisol serve --port 8080
Then run:
    python3 examples/spatial_raster/run_spatial_api.py
"""

import json
import urllib.request
from pathlib import Path

BASE_URL = "http://127.0.0.1:8080"
DATA_DIR = Path("./sample_rasters").resolve()


def main():
    payload = {
        "mode": "geotiff" if (DATA_DIR / "mapbiomas_2024.tif").exists() else "demo",
        "scheme_id": "mapbiomas_br",
        # Filter exclusively to MapBiomas code 15 (Pasture) with custom management:
        "only_classes": [15],
        "class_mappings": [
            {
                "code": 15,
                "label_en": "Pasture (Urochloa brizantha / Panicum maximum)",
                "label_pt": "Pastagem (Urochloa brizantha / Panicum maximum)",
                "color": "#edde8e",
                "preset_id": "tropical_pasture_urochloa",
                "grazing": 0.22,      # Moderate rotational grazing (0.45 = degraded/overgrazed)
                "fert_n": 3.0,        # 30 kg N/ha/yr maintenance fertilization
                "is_cropland": False,
            }
        ],
        "lulc_paths": [str(DATA_DIR / "mapbiomas_2024.tif")],
        "precip_path": str(DATA_DIR / "terraclimate_precip_12m_mm.tif"),
        "tmin_path": str(DATA_DIR / "terraclimate_tmin_12m_c.tif"),
        "tmax_path": str(DATA_DIR / "terraclimate_tmax_12m_c.tif"),
        "precip_unit": "mm",
        "temp_unit": "celsius",
        "sand_path": str(DATA_DIR / "soilgrids_sand_gkg.tif"),
        "clay_path": str(DATA_DIR / "soilgrids_clay_gkg.tif"),
        "bulkd_path": str(DATA_DIR / "soilgrids_bdod_cgcm3.tif"),
        "ph_path": str(DATA_DIR / "soilgrids_phh2o_x10.tif"),
        "soilgrids_units": True,
        "target_resolution_m": 1000.0,  # 1 km laptop-friendly modeling (set None for native)
        "output_unit": "mgha",          # Export COGs directly in Mg C/ha
        "spinup_years": 60,
        "transient_years": 30,
        "co2_start_ppm": 354.0,
        "co2_end_ppm": 422.0,
        "export_cog_dir": str(Path("./out_api_pasture_mgha").resolve()),
    }

    req = urllib.request.Request(
        f"{BASE_URL}/api/lulc/run",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        out = json.loads(resp.read().decode("utf-8"))

    print(f"Source:        {out['source_label']}")
    print(f"Active pixels: {out['n_active']} ({out['threads_used']} threads, {out['elapsed_secs']:.2f}s)")
    print(f"Throughput:    {out['pixel_years_per_sec']:,.0f} pixel-yr/s")
    print(f"Mean SOMSC 0-20 cm:       {out['mean_somsc'] / 100:.2f} Mg C/ha")
    print(f"Mean SOMSC 0-30 cm IPCC:  {out['mean_somsc_0_30'] / 100:.2f} Mg C/ha")
    print(f"Mean SOMSC 0-100 cm Deep: {out['mean_somsc_0_100'] / 100:.2f} Mg C/ha")
    print(f"Exported COGs ({out['cog_unit']}):")
    for cog in out["exported_cogs"]:
        print(f"  - {cog}")


if __name__ == "__main__":
    main()
