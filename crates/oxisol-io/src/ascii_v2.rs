//! Pure-Rust parser for VEMAP `.v2` ASCII spatial grids (`vveg.v2`, `mbd1_m1.v2`,
//! `mcl1_m1.v2`, `msa1_m1.v2`, and recovered `rock`/`zminrl` grids).

use std::fs;
use std::path::Path;

use crate::error::{IoError, Result};

/// Parsed 2D integer spatial grid in row-major order (`nrows × ncols`).
#[derive(Debug, Clone)]
pub struct AsciiV2Grid {
    /// Number of rows (48 for continental US VEMAP).
    pub nrows: usize,
    /// Number of columns (115 for continental US VEMAP).
    pub ncols: usize,
    /// Scale factor declared in header (e.g. `100.0` for bulk density).
    pub scale: f64,
    /// Row-major grid values (`-9999` denotes masked/ocean cells).
    pub data: Vec<i32>,
}

impl AsciiV2Grid {
    /// Load a VEMAP `.v2` file from disk.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let txt = fs::read_to_string(path)?;
        Self::parse(&txt)
    }

    /// Parse a VEMAP `.v2` grid from text.
    pub fn parse(content: &str) -> Result<Self> {
        let mut lines = content.lines();
        let _h0 = lines.next();
        let _h1 = lines.next();
        let _h2 = lines.next();
        let h3 = lines
            .next()
            .ok_or_else(|| IoError::Parse("v2 missing title header".into()))?;
        let dim_line = lines
            .next()
            .ok_or_else(|| IoError::Parse("v2 missing dimension header".into()))?;

        // Extract "Scaled by: <num>" if present.
        let scale = h3
            .split("Scaled by:")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|tok| tok.parse::<f64>().ok())
            .unwrap_or(1.0);

        let dims: Vec<usize> = dim_line
            .split_whitespace()
            .filter_map(|t| t.parse::<usize>().ok())
            .collect();
        if dims.len() < 4 {
            return Err(IoError::Parse(format!("invalid v2 dim line: {dim_line}")));
        }
        let ncols = dims[1] - dims[0] + 1;
        let nrows = dims[3] - dims[2] + 1;

        let mut data = Vec::with_capacity(nrows * ncols);
        for line in lines {
            for tok in line.split_whitespace() {
                let v = tok
                    .parse::<i32>()
                    .map_err(|e| IoError::Parse(format!("invalid int in v2: {tok} ({e})")))?;
                data.push(v);
            }
        }
        if data.len() != nrows * ncols {
            return Err(IoError::Parse(format!(
                "v2 expected {} values ({nrows}x{ncols}), got {}",
                nrows * ncols,
                data.len()
            )));
        }

        Ok(Self {
            nrows,
            ncols,
            scale,
            data,
        })
    }

    /// Raw integer value at `(row, col)`.
    #[inline(always)]
    pub fn get_raw(&self, row: usize, col: usize) -> i32 {
        self.data[row * self.ncols + col]
    }

    /// Unscaled floating-point value at `(row, col)`, returning `None` for `-9999`.
    #[inline(always)]
    pub fn get_scaled(&self, row: usize, col: usize) -> Option<f64> {
        let v = self.get_raw(row, col);
        if v < -9000 {
            None
        } else {
            Some(v as f64 / self.scale)
        }
    }
}
