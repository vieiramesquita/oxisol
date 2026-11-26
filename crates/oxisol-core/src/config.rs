//! Simulation configuration and compatibility profile controls.

use serde::{Deserialize, Serialize};

/// Controls scientific vs. historical VEMAP V4.5 parity behavior.
///
/// See `docs/spec/07_compatibility_profiles.md` for the full catalog of deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompatibilityProfile {
    /// Default modern profile: `f64` recommended, exact spherical astronomy
    /// daylength (`tan(decl)`), organic matter % computed after SOM init,
    /// consistent rock-fraction adjustment, and full state checkpoints.
    #[default]
    ScientificClean,

    /// Historical VEMAP V4.5 (1995–2001) audit profile: reproduces documented
    /// legacy behaviors (daylength declination omission, `ompc == 0` at init,
    /// deep-soil `soildepth > 150 cm` rockfrac zeroing, V4.5 sub-step order)
    /// for black-box numerical verification against archived outputs.
    VemapV45Parity,
}

/// Active biogeochemical element set (`NELEM`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ElementMode {
    /// Carbon + Nitrogen (`NELEM = 1`).
    #[default]
    CarbonNitrogen = 1,
    /// Carbon + Nitrogen + Phosphorus (`NELEM = 2`).
    CarbonNitrogenPhosphorus = 2,
    /// Carbon + Nitrogen + Phosphorus + Sulfur (`NELEM = 3`).
    FullCnps = 3,
}

impl ElementMode {
    /// Number of active nutrient elements beyond Carbon (`1`, `2`, or `3`).
    #[inline(always)]
    pub const fn nelem(self) -> usize {
        self as usize
    }

    /// Construct from legacy integer code (`1`, `2`, `3`).
    pub const fn from_nelem(n: usize) -> Self {
        match n {
            1 => Self::CarbonNitrogen,
            2 => Self::CarbonNitrogenPhosphorus,
            _ => Self::FullCnps,
        }
    }
}

/// Carbon isotope tracking mode (`LABTYP`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum IsotopeMode {
    /// No isotope labeling (`labtyp = 0`).
    #[default]
    Unlabeled = 0,
    /// Radiocarbon C-14 labeling (`labtyp = 1`).
    C14 = 1,
    /// Stable isotope C-13 tracking (`labtyp = 2`).
    C13 = 2,
}

impl IsotopeMode {
    /// Construct from integer code (`0`, `1`, `2`).
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::C14,
            2 => Self::C13,
            _ => Self::Unlabeled,
        }
    }
}

/// Active plant-soil ecosystem type (`CURSYS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum SystemKind {
    /// Grassland or agricultural crop system (`cursys = 1`).
    #[default]
    GrasslandCrop = 1,
    /// Forest system (`cursys = 2`).
    Forest = 2,
    /// Savanna (coupled herbaceous + woody with shading & nutrient competition, `cursys = 3`).
    Savanna = 3,
}

impl SystemKind {
    /// Construct from legacy integer code (`1`, `2`, `3`).
    pub const fn from_code(code: i32) -> Self {
        match code {
            2 => Self::Forest,
            3 => Self::Savanna,
            _ => Self::GrasslandCrop,
        }
    }

    /// Returns true if herbaceous crop/grassland submodel is active.
    #[inline(always)]
    pub const fn has_crop(self) -> bool {
        matches!(self, Self::GrasslandCrop | Self::Savanna)
    }

    /// Returns true if woody forest submodel is active.
    #[inline(always)]
    pub const fn has_forest(self) -> bool {
        matches!(self, Self::Forest | Self::Savanna)
    }
}

/// Top-level simulation execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    /// Scientific vs. VEMAP V4.5 parity profile.
    pub profile: CompatibilityProfile,
    /// Active nutrient cycles (C+N, C+N+P, or C+N+P+S).
    pub elements: ElementMode,
    /// Carbon isotope mode.
    pub isotope: IsotopeMode,
    /// Optional constant or time-varying atmospheric CO2 concentration (ppm).
    /// When `None`, uses `fix.co2ppm[0]`.
    pub co2_ppm_override: Option<f64>,
    /// Enable per-step strict mass-balance verification (recommended in tests).
    pub audit_mass_balance: bool,
    /// Starting simulation year (1-based).
    pub start_year: i32,
    /// Final simulation year (inclusive).
    pub end_year: i32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            profile: CompatibilityProfile::ScientificClean,
            elements: ElementMode::CarbonNitrogen,
            isotope: IsotopeMode::Unlabeled,
            co2_ppm_override: None,
            audit_mass_balance: false,
            start_year: 1,
            end_year: 100,
        }
    }
}
