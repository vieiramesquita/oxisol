#!/usr/bin/env python3
"""Generate realistic georeferenced Cerrado-Amazon sample GeoTIFFs for Oxisol.

Creates MapBiomas Collection 11 LULC maps (1990, 2005, 2024), 12-band monthly
climate rasters (precip mm, tmin C, tmax C), and SoilGrids-convention rasters
(sand g/kg, clay g/kg, bdod cg/cm3, phh2o pH*10) in EPSG:4326 so any user can
test `oxisol raster` locally in seconds.

Usage:
    pip install rasterio numpy
    python3 examples/spatial_raster/generate_sample_rasters.py --out-dir ./sample_rasters
"""

import argparse
from pathlib import Path
import numpy as np
import rasterio
from rasterio.transform import from_origin


def write_tif(path: Path, arr: np.ndarray, transform, nodata=-9999.0, dtype="float32"):
    if arr.ndim == 2:
        bands, h, w = 1, arr.shape[0], arr.shape[1]
        data = arr[np.newaxis, :, :]
    else:
        bands, h, w = arr.shape
        data = arr
    profile = {
        "driver": "GTiff",
        "height": h,
        "width": w,
        "count": bands,
        "dtype": dtype,
        "crs": "EPSG:4326",
        "transform": transform,
        "nodata": nodata,
        "compress": "deflate",
    }
    with rasterio.open(path, "w", **profile) as dst:
        dst.write(data.astype(dtype))
    print(f"  Wrote {path} ({bands} band(s), {h}x{w}, {dtype})")


def main():
    parser = argparse.ArgumentParser(description="Generate Oxisol Cerrado-Amazon sample GeoTIFFs.")
    parser.add_argument("--out-dir", default="./sample_rasters", help="Output directory")
    parser.add_argument("--rows", type=int, default=64, help="LULC raster rows (default 64)")
    parser.add_argument("--cols", type=int, default=80, help="LULC raster cols (default 80)")
    args = parser.parse_args()

    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)

    # Mato Grosso / Cerrado-Amazon transition (~12.5°S to 15.7°S, 56.5°W to 52.5°W; ~0.05° ≈ 5.5 km)
    rows, cols = args.rows, args.cols
    res_deg = 0.05
    lon_min, lat_max = -56.5, -12.5
    lulc_tf = from_origin(lon_min, lat_max, res_deg, res_deg)

    rr, cc = np.meshgrid(np.linspace(0, 1, rows), np.linspace(0, 1, cols), indexing="ij")

    # MapBiomas Collection 11 codes:
    # 3 = Forest Formation, 4 = Savanna Formation, 15 = Pasture, 39 = Soybean, 33 = River/Water
    river = np.abs(cc - (0.50 + 0.08 * np.sin(rr * 2.0 * np.pi))) < 0.035

    lulc_1990 = np.where(rr < 0.45, 3, 4).astype(np.int16)
    lulc_1990[(rr > 0.58) & (cc > 0.55)] = 15  # Existing 1990 pasture in SE Cerrado
    lulc_1990[river] = 33

    lulc_2005 = lulc_1990.copy()
    lulc_2005[(rr > 0.32) & (cc > 0.30) & (~river)] = 15  # Pasture frontier expansion
    lulc_2005[(rr > 0.65) & (cc > 0.62) & (~river)] = 39  # Early soybean conversion

    lulc_2024 = lulc_2005.copy()
    lulc_2024[(rr > 0.22) & (cc > 0.22) & (~river)] = 15  # Continued pasture expansion
    lulc_2024[(rr > 0.48) & (cc > 0.52) & (~river)] = 39  # Soybean consolidation
    lulc_2024[(rr > 0.16) & (rr < 0.26) & (cc > 0.72) & (~river)] = 3  # Riparian regeneration

    write_tif(out / "mapbiomas_1990.tif", lulc_1990, lulc_tf, nodata=0, dtype="int16")
    write_tif(out / "mapbiomas_2005.tif", lulc_2005, lulc_tf, nodata=0, dtype="int16")
    write_tif(out / "mapbiomas_2024.tif", lulc_2024, lulc_tf, nodata=0, dtype="int16")

    # Coarser 12-band monthly climate rasters (0.10° ≈ 11 km) to demonstrate automatic resampling
    c_rows, c_cols = rows // 2, cols // 2
    clim_tf = from_origin(lon_min, lat_max, res_deg * 2, res_deg * 2)
    cr, cc2 = np.meshgrid(np.linspace(0, 1, c_rows), np.linspace(0, 1, c_cols), indexing="ij")

    # Southern Hemisphere tropical monsoon seasonality (wet Nov-Mar, dry Jun-Aug)
    months = np.arange(12)
    phase = 2.0 * np.pi * (months - 0.0) / 12.0  # Peak Jan (SH summer)
    p_weights = np.clip(1.0 + 0.72 * np.cos(phase), 0.12, None)
    p_weights /= p_weights.sum()

    ann_precip_mm = 2150.0 - 650.0 * cr - 180.0 * cc2  # North wetter (Amazon), South Cerrado (~1450 mm)
    mat_c = 26.0 - 2.6 * cr  # 26°C north -> 23.4°C south

    precip = np.stack([ann_precip_mm * w for w in p_weights], axis=0)
    tmean = np.stack([mat_c + 2.4 * np.cos(ph) for ph in phase], axis=0)
    tmin = tmean - 5.2
    tmax = tmean + 5.2

    write_tif(out / "terraclimate_precip_12m_mm.tif", precip, clim_tf)
    write_tif(out / "terraclimate_tmin_12m_c.tif", tmin, clim_tf)
    write_tif(out / "terraclimate_tmax_12m_c.tif", tmax, clim_tf)

    # SoilGrids 250m-style integer conventions (auto-scaled by Oxisol):
    # sand & clay in g/kg (0..1000), bdod in cg/cm³ (e.g. 125 = 1.25 g/cm³), phh2o in pH*10 (54 = 5.4)
    clay_gkg = np.clip(560.0 - 180.0 * rr + 60.0 * cc, 180.0, 720.0)
    sand_gkg = np.clip(280.0 + 180.0 * rr - 50.0 * cc, 120.0, 680.0)
    bdod_cg = np.clip(116.0 + 16.0 * rr, 105.0, 145.0)
    ph_x10 = np.clip(48.0 + 9.0 * rr, 45.0, 62.0)

    write_tif(out / "soilgrids_sand_gkg.tif", sand_gkg, lulc_tf, nodata=-32768, dtype="int16")
    write_tif(out / "soilgrids_clay_gkg.tif", clay_gkg, lulc_tf, nodata=-32768, dtype="int16")
    write_tif(out / "soilgrids_bdod_cgcm3.tif", bdod_cg, lulc_tf, nodata=-32768, dtype="int16")
    write_tif(out / "soilgrids_phh2o_x10.tif", ph_x10, lulc_tf, nodata=-32768, dtype="int16")

    print("\nDone! Run Oxisol with:")
    print(f"  ./target/release/oxisol raster --lulc {out}/mapbiomas_2024.tif --only-classes 15 "
          f"--precip {out}/terraclimate_precip_12m_mm.tif --tmin {out}/terraclimate_tmin_12m_c.tif "
          f"--tmax {out}/terraclimate_tmax_12m_c.tif --sand {out}/soilgrids_sand_gkg.tif "
          f"--clay {out}/soilgrids_clay_gkg.tif --bulkd {out}/soilgrids_bdod_cgcm3.tif "
          f"--ph {out}/soilgrids_phh2o_x10.tif --unit mgha --out-dir ./out_pasture_cogs")


if __name__ == "__main__":
    main()
