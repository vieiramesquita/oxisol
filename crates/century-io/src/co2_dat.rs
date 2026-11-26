//! Pure-Rust parser for annual atmospheric CO₂ series (`co2.dat`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::Result;

/// Annual atmospheric CO₂ lookup table (`year -> ppm`).
#[derive(Debug, Clone, Default)]
pub struct Co2Series {
    /// Year-indexed CO₂ concentrations (ppm).
    pub by_year: BTreeMap<i32, f64>,
}

impl Co2Series {
    /// Load `co2.dat` from disk (two whitespace-separated columns: `year ppm`).
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let txt = fs::read_to_string(path)?;
        Ok(Self::parse(&txt))
    }

    /// Parse `co2.dat` from string.
    pub fn parse(content: &str) -> Self {
        let mut by_year = BTreeMap::new();
        for line in content.lines() {
            let mut toks = line.split_whitespace();
            if let (Some(y_s), Some(c_s)) = (toks.next(), toks.next()) {
                if let (Ok(y), Ok(c)) = (y_s.parse::<i32>(), c_s.parse::<f64>()) {
                    by_year.insert(y, c);
                }
            }
        }
        Self { by_year }
    }

    /// Return CO₂ for `year`, clamping to endpoints if out of range.
    pub fn concentration(&self, year: i32, fallback: f64) -> f64 {
        if let Some(&v) = self.by_year.get(&year) {
            return v;
        }
        if let Some((&first_y, &first_v)) = self.by_year.iter().next() {
            if year < first_y {
                return first_v;
            }
        }
        if let Some((&last_y, &last_v)) = self.by_year.iter().next_back() {
            if year > last_y {
                return last_v;
            }
        }
        fallback
    }
}
