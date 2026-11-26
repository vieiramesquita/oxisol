//! Pure-Rust geospatial raster I/O (GeoTIFF, Cloud-Optimized GeoTIFF, NetCDF,
//! ASCII grids) powered by [`terrio`] — zero `libgdal` / C runtime dependencies.
//!
//! Enables Oxisol to read real Land-Use / Land-Cover (LULC) maps (e.g. MapBiomas,
//! ESA WorldCover), multi-band modeled climate rasters (ERA5-Land, WorldClim,
//! TerraClimate, CHIRPS, CMIP6), and soil rasters (SoilGrids), and export
//! georeferenced Cloud-Optimized GeoTIFFs (COGs) ready for QGIS, Python, or GEE.

use std::path::Path;

use terrio::formats::RasterDriver;
use terrio::raster::CrsInfo;
use terrio::{DataType, Raster, RasterConfig, RasterDataset, RasterWriteOptions};

use crate::error::{IoError, Result};

/// In-memory georeferenced multi-band raster backed by [`terrio::RasterDataset`].
#[derive(Debug, Clone)]
pub struct GeoRaster {
    /// Underlying `terrio` raster dataset (pixels + CRS + affine geotransform).
    pub dataset: RasterDataset,
    /// Affine geotransform `[origin_x, pixel_w, rot_x, origin_y, rot_y, pixel_h]`.
    pub transform: [f64; 6],
}

impl GeoRaster {
    /// Open any raster supported by `terrio` (GeoTIFF, COG, NetCDF, HDF5, AAIGrid, ENVI).
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p_str = path.as_ref().to_string_lossy();
        let dataset = terrio::open_raster(&p_str)
            .map_err(|e| IoError::Parse(format!("terrio open_raster('{p_str}') failed: {e}")))?;
        let transform = dataset.geo_transform();
        Ok(Self { dataset, transform })
    }

    /// Construct a single-band Float32 geographic (`EPSG:4326`) raster from row-major data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_epsg4326(
        rows: usize,
        cols: usize,
        x_min: f64,
        y_min: f64,
        cell_size_x: f64,
        cell_size_y: f64,
        nodata: f64,
        data: Vec<f64>,
    ) -> Result<Self> {
        let cfg = RasterConfig {
            cols,
            rows,
            bands: 1,
            x_min,
            y_min,
            cell_size: cell_size_x.abs(),
            cell_size_y: Some(-cell_size_y.abs()),
            nodata,
            data_type: DataType::F32,
            crs: CrsInfo::from_epsg(4326),
            metadata: vec![("SOFTWARE".into(), "Oxisol (terrio pure-Rust COG)".into())],
        };
        let raster = Raster::from_data(cfg, data)
            .map_err(|e| IoError::Parse(format!("terrio Raster::from_data failed: {e}")))?;
        let dataset = RasterDataset::new(raster, true);
        let transform = dataset.geo_transform();
        Ok(Self { dataset, transform })
    }

    /// Build a new single-band Float32 raster inheriting exact grid dimensions,
    /// extent, geotransform, and CRS from `self`.
    pub fn derive_like(&self, nodata: f64, data: Vec<f64>) -> Result<Self> {
        let src = &self.dataset.raster;
        let cfg = RasterConfig {
            cols: src.cols,
            rows: src.rows,
            bands: 1,
            x_min: src.x_min,
            y_min: src.y_min,
            cell_size: src.cell_size_x,
            cell_size_y: Some(-src.cell_size_y),
            nodata,
            data_type: DataType::F32,
            crs: src.crs.clone(),
            metadata: vec![("SOFTWARE".into(), "Oxisol (terrio pure-Rust COG)".into())],
        };
        let raster = Raster::from_data(cfg, data)
            .map_err(|e| IoError::Parse(format!("terrio Raster::from_data failed: {e}")))?;
        let dataset = RasterDataset::new(raster, true);
        let transform = dataset.geo_transform();
        Ok(Self { dataset, transform })
    }

    /// Write this raster as a DEFLATE-compressed Cloud-Optimized GeoTIFF (COG).
    pub fn write_cog<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let p_str = path.as_ref().to_string_lossy();
        let opts = RasterWriteOptions {
            driver: Some(RasterDriver::Cog),
            compression: Some("DEFLATE".to_string()),
            ..Default::default()
        };
        terrio::write_raster_dataset(&self.dataset, &p_str, &opts).map_err(|e| {
            IoError::Parse(format!(
                "terrio write_raster_dataset('{p_str}') failed: {e}"
            ))
        })
    }

    /// Raster height in pixels.
    #[inline]
    pub fn rows(&self) -> usize {
        self.dataset.raster.rows
    }

    /// Raster width in pixels.
    #[inline]
    pub fn cols(&self) -> usize {
        self.dataset.raster.cols
    }

    /// Number of bands (`1` for LULC/soil, `12` or `12*N` for monthly climate).
    #[inline]
    pub fn bands(&self) -> usize {
        self.dataset.raster.bands
    }

    /// Center map coordinates `(x, y)` for `(row, col)` using the affine geotransform.
    #[inline]
    pub fn pixel_xy(&self, row: usize, col: usize) -> (f64, f64) {
        let gt = self.transform;
        let c = col as f64 + 0.5;
        let r = row as f64 + 0.5;
        let x = gt[0] + c * gt[1] + r * gt[2];
        let y = gt[3] + c * gt[4] + r * gt[5];
        (x, y)
    }

    /// Estimate `(lat_deg, lon_deg)` for `(row, col)`. For geographic grids
    /// (`|x| <= 180, |y| <= 90`), uses the affine coordinates directly; otherwise
    /// falls back to `fallback_lat` so projected grids still run safely.
    #[inline]
    pub fn pixel_lat_lon(&self, row: usize, col: usize, fallback_lat: f64) -> (f64, f64) {
        let (x, y) = self.pixel_xy(row, col);
        if (-180.5..=180.5).contains(&x) && (-90.0..=90.0).contains(&y) {
            (y.clamp(-70.0, 75.0), x.clamp(-180.0, 180.0))
        } else {
            (fallback_lat.clamp(-70.0, 75.0), 0.0)
        }
    }

    /// Read pixel `(band, row, col)` returning `None` for out-of-bounds, NaN, or NoData.
    #[inline]
    pub fn value_at(&self, band: usize, row: usize, col: usize) -> Option<f64> {
        let v = self
            .dataset
            .raster
            .get_opt(band as isize, row as isize, col as isize)?;
        if !v.is_finite() {
            return None;
        }
        let nd = self.dataset.raster.nodata;
        if nd.is_finite() && (v - nd).abs() <= 1.0e-6 * nd.abs().max(1.0) {
            return None;
        }
        Some(v)
    }

    /// Approximate pixel size in meters (handles both geographic EPSG:4326 degrees and projected meters).
    #[inline]
    pub fn pixel_resolution_meters(&self) -> f64 {
        let gt = self.transform;
        let dx = gt[1].abs();
        let dy = gt[5].abs();
        let avg = 0.5 * (dx + dy);
        // Heuristic: coordinates within [-180.5..180.5] x [-90..90] with small pixel size (< 5 deg) are degrees.
        let (cx, cy) = self.pixel_xy(self.rows() / 2, self.cols() / 2);
        if (-180.5..=180.5).contains(&cx) && (-90.0..=90.0).contains(&cy) && avg < 5.0 {
            let cos_lat = (cy.to_radians().cos()).abs().max(0.2);
            let mx = dx * 111_320.0 * cos_lat;
            let my = dy * 111_320.0;
            0.5 * (mx + my)
        } else {
            avg
        }
    }

    /// Compute integer aggregation stride (`>= 1`) needed to model at `target_resolution_m` meters.
    ///
    /// Returns `1` when `target_resolution_m <= 0.0` or when native resolution is already coarser.
    #[inline]
    pub fn stride_for_target_meters(&self, target_resolution_m: f64) -> usize {
        if !target_resolution_m.is_finite() || target_resolution_m <= 0.0 {
            return 1;
        }
        let native_m = self.pixel_resolution_meters().max(1.0e-3);
        let ratio = (target_resolution_m / native_m).round() as usize;
        ratio.max(1)
    }

    /// Aggregate a categorical single-band raster (e.g., MapBiomas LULC) by integer `stride`
    /// using **majority (mode)** voting inside each `stride × stride` block, preserving exact
    /// geographic extent, scaled affine geotransform, and CRS.
    ///
    /// When `stride <= 1`, returns a clone of `self`.
    pub fn aggregate_categorical_mode(&self, stride: usize) -> Result<Self> {
        if stride <= 1 || self.rows() <= stride || self.cols() <= stride {
            return Ok(self.clone());
        }
        let src = &self.dataset.raster;
        let out_rows = self.rows().div_ceil(stride);
        let out_cols = self.cols().div_ceil(stride);
        let nd = src.nodata;
        let mut out_data = vec![nd; out_rows * out_cols];
        let mut counts: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();

        for or in 0..out_rows {
            let r0 = or * stride;
            let r1 = (r0 + stride).min(self.rows());
            for oc in 0..out_cols {
                let c0 = oc * stride;
                let c1 = (c0 + stride).min(self.cols());
                counts.clear();
                for r in r0..r1 {
                    for c in c0..c1 {
                        if let Some(v) = self.value_at(0, r, c) {
                            *counts.entry(v.round() as i32).or_insert(0) += 1;
                        }
                    }
                }
                if let Some((&mode_code, _)) =
                    counts.iter().max_by_key(|&(code, cnt)| (*cnt, -*code))
                {
                    out_data[or * out_cols + oc] = mode_code as f64;
                }
            }
        }

        let new_cx = src.cell_size_x * (stride as f64);
        let new_cy = src.cell_size_y * (stride as f64);
        // Preserve north-edge (y_max) alignment so aggregated grid stays geographically anchored.
        let y_max = src.y_min + (src.rows as f64) * src.cell_size_y;
        let new_y_min = y_max - (out_rows as f64) * new_cy;
        let cfg = RasterConfig {
            cols: out_cols,
            rows: out_rows,
            bands: 1,
            x_min: src.x_min,
            y_min: new_y_min,
            cell_size: new_cx,
            cell_size_y: Some(-new_cy),
            nodata: nd,
            data_type: DataType::F32,
            crs: src.crs.clone(),
            metadata: vec![("SOFTWARE".into(), "Oxisol (terrio pure-Rust COG)".into())],
        };
        let raster = Raster::from_data(cfg, out_data).map_err(|e| {
            IoError::Parse(format!("terrio aggregate_categorical_mode failed: {e}"))
        })?;
        let dataset = RasterDataset::new(raster, true);
        let transform = dataset.geo_transform();
        Ok(Self { dataset, transform })
    }

    /// Sample `band` at target geographic/map coordinate `(x, y)` using nearest-neighbor
    /// lookup via inverse geotransform, falling back to proportional `(row, col)` mapping
    /// if the source raster does not overlap `(x, y)`.
    pub fn sample_at_xy_or_frac(
        &self,
        band: usize,
        x: f64,
        y: f64,
        frac_row: f64,
        frac_col: f64,
    ) -> Option<f64> {
        let b = band.min(self.bands().saturating_sub(1));
        let gt = self.transform;
        if gt[1].abs() > 1.0e-12 && gt[5].abs() > 1.0e-12 {
            let col_f = (x - gt[0]) / gt[1];
            let row_f = (y - gt[3]) / gt[5];
            if col_f >= 0.0
                && row_f >= 0.0
                && (col_f as usize) < self.cols()
                && (row_f as usize) < self.rows()
            {
                return self.value_at(b, row_f as usize, col_f as usize);
            }
        }
        let r = ((frac_row * self.rows() as f64) as usize).min(self.rows().saturating_sub(1));
        let c = ((frac_col * self.cols() as f64) as usize).min(self.cols().saturating_sub(1));
        self.value_at(b, r, c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrio_cog_geotiff_roundtrip() {
        let rows = 8;
        let cols = 10;
        let nodata = -9999.0;
        let mut vals = vec![0.0_f64; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                vals[r * cols + c] = if r == 0 && c == 0 {
                    nodata
                } else {
                    1500.0 + (r * 100 + c * 10) as f64
                };
            }
        }
        let gr = GeoRaster::from_epsg4326(rows, cols, -56.0, -14.0, 0.1, 0.1, nodata, vals)
            .expect("create EPSG:4326 raster");
        let tmp = std::env::temp_dir().join("oxisol_terrio_roundtrip_test.tif");
        gr.write_cog(&tmp).expect("write COG");

        let back = GeoRaster::open(&tmp).expect("open COG back");
        let _ = std::fs::remove_file(&tmp);

        assert_eq!(back.rows(), rows);
        assert_eq!(back.cols(), cols);
        assert_eq!(back.bands(), 1);
        assert!(back.value_at(0, 0, 0).is_none(), "NoData preserved");
        let v = back.value_at(0, 3, 4).expect("valid pixel");
        assert!((v - 1840.0).abs() < 1.0e-2, "expected 1840.0, got {v}");
        let (lat, lon) = back.pixel_lat_lon(0, 0, 0.0);
        assert!(lat < -13.0 && lon < -55.0, "lat={lat}, lon={lon}");
    }
}
