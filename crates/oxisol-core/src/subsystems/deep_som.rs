//! Vertically resolved multi-depth Soil Organic Matter (`MultiLayerSOM`) profile
//! extending classic CENTURY `0–20 cm` surface SOM to **IPCC `0–30 cm`** and
//! **deep `0–100 cm`** soil profiles for tropical forests, Cerrado savannas, and
//! deep-rooted exotic C4 pastures (*Urochloa brizantha* / *Panicum maximum*).
//!
//! ## Scientific Foundation
//! - **Root depth distribution** $Y(d) = 1 - \beta^d$ ($d$ in cm):
//!   - Jackson et al. (1996), *Oecologia*, <https://doi.org/10.1007/BF00333714>
//!   - Jobbágy & Jackson (2000), *Ecological Applications*, <https://doi.org/10.1890/1051-0761(2000)010[0423:TVDOSO]2.0.CO;2>
//!   - Fisher et al. (1994), *Nature* (deep-rooted South American tropical pastures), <https://doi.org/10.1038/371236a0>
//! - **Depth-attenuated decomposition** $k(z) = k_0 \exp(-z / z_\tau)$ and vertical transport:
//!   - Koven et al. (2013), *Biogeosciences*, <https://doi.org/10.5194/bg-10-7109-2013>
//!
//! ## Architecture
//! Classic `0–20 cm` `CellState` pools (`som1c`, `som2c`, `som3c`) are preserved unmodified
//! for exact VEMAP/CENTURY parity. `DeepSomProfile` couples three subsoil horizons:
//! - Horizon 1: **`20–30 cm`** (midpoint $z = 25\ \text{cm}$) → yields exact **IPCC `0–30 cm`** stock
//! - Horizon 2: **`30–60 cm`** (midpoint $z = 45\ \text{cm}$)
//! - Horizon 3: **`60–100 cm`** (midpoint $z = 80\ \text{cm}$) → yields full **`0–100 cm`** stock

use serde::{Deserialize, Serialize};

use crate::float::Float;

/// Number of coupled subsoil horizons below the classic `0–20 cm` CENTURY layer.
pub const SUBSOIL_HORIZONS: usize = 3;

/// Subsoil horizon upper and lower depths in cm: `[20–30, 30–60, 60–100]`.
pub const SUBSOIL_BOUNDS_CM: [(f64, f64); SUBSOIL_HORIZONS] =
    [(20.0, 30.0), (30.0, 60.0), (60.0, 100.0)];

/// Subsoil horizon midpoints in cm: `[25, 45, 80]`.
const SUBSOIL_MID_CM: [f64; SUBSOIL_HORIZONS] = [25.0, 45.0, 80.0];

/// Depth e-folding attenuation length $z_\tau$ (cm) for microbial decomposition
/// (Koven et al., 2013, *Biogeosciences*).
const Z_TAU_CM: f64 = 55.0;

/// Biome root-depth parameterization ($\beta$ in Jackson et al., 1996: $Y(d) = 1 - \beta^d$).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RootDepthProfile {
    /// Jackson (1996) root extinction coefficient $\beta \in [0.90, 0.990]$.
    pub beta: f64,
    /// Characteristic annual subsoil bioturbation / DOC advection rate ($\text{yr}^{-1}$).
    pub doc_advect_yr: f64,
}

impl RootDepthProfile {
    /// Select a literature-grounded root depth profile for a named Oxisol ecosystem preset.
    pub fn for_preset(preset_id: &str) -> Self {
        match preset_id {
            // Tropical rainforest / evergreen broadleaf (deep woody roots, Jackson 1996: ~0.972–0.980)
            "amazon_rainforest" => Self {
                beta: 0.978,
                doc_advect_yr: 0.016,
            },
            // Tropical savanna / Cerrado deep woody + C4 root system (Jackson 1996: ~0.972–0.976)
            "cerrado_savanna" => Self {
                beta: 0.975,
                doc_advect_yr: 0.013,
            },
            // Deep-rooted exotic African C4 pasture (Urochloa brizantha / Panicum maximum; Fisher et al. 1994)
            "tropical_pasture_urochloa" => Self {
                beta: 0.972,
                doc_advect_yr: 0.012,
            },
            // Temperate deciduous broadleaf forest (Jackson 1996: 0.966)
            "temperate_deciduous" => Self {
                beta: 0.968,
                doc_advect_yr: 0.011,
            },
            // Semi-arid deep-rooted shrubland (Jackson 1996: 0.964–0.975)
            "semiarid_shrubland" => Self {
                beta: 0.968,
                doc_advect_yr: 0.006,
            },
            // Annual tilled/no-till grain cropland (shallow roots; Jackson 1996: 0.961)
            "cropland_corn_wheat" => Self {
                beta: 0.955,
                doc_advect_yr: 0.008,
            },
            // Boreal conifer forest (cold/shallow rooting; Jackson 1996: 0.943)
            "boreal_conifer" => Self {
                beta: 0.946,
                doc_advect_yr: 0.007,
            },
            // Temperate C3 grassland (Jackson 1996: 0.943)
            _ => Self {
                beta: 0.952,
                doc_advect_yr: 0.009,
            },
        }
    }

    /// Fraction of total `0–100 cm` roots located within `[d_top, d_bot]` cm.
    pub fn layer_root_fraction(&self, d_top: f64, d_bot: f64) -> f64 {
        let total_100 = (1.0 - self.beta.powf(100.0)).max(1.0e-6);
        let y_top = 1.0 - self.beta.powf(d_top);
        let y_bot = 1.0 - self.beta.powf(d_bot);
        ((y_bot - y_top) / total_100).clamp(0.0, 1.0)
    }
}

/// Three-pool (`active`, `slow`, `passive`) SOC state for one subsoil horizon ($\text{g C}\cdot\text{m}^{-2}$).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SubsoilHorizon<F: Float> {
    /// Active microbial carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som1: F,
    /// Slow mineral-associated / occluded carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som2: F,
    /// Passive recalcitrant carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub som3: F,
}

impl<F: Float> SubsoilHorizon<F> {
    /// Total SOC in this horizon ($\text{g C}\cdot\text{m}^{-2}$).
    #[inline]
    pub fn total(&self) -> F {
        self.som1 + self.som2 + self.som3
    }
}

/// Multi-depth SOC summary returned for APIs, Studio charts, and COG exports ($\text{g C}\cdot\text{m}^{-2}$).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DepthSomSummary {
    /// Classic CENTURY `0–20 cm` SOC ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_0_20: f64,
    /// `20–30 cm` subsoil horizon SOC ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_20_30: f64,
    /// `30–60 cm` subsoil horizon SOC ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_30_60: f64,
    /// `60–100 cm` deep subsoil horizon SOC ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_60_100: f64,
    /// Cumulative **IPCC Tier 2/3 `0–30 cm`** SOC (`0–20 + 20–30`, $\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_0_30: f64,
    /// Cumulative **full `0–100 cm`** profile SOC ($\text{g C}\cdot\text{m}^{-2}$).
    pub somsc_0_100: f64,
    /// Subsoil (`20–100 cm`) share of total `0–100 cm` SOC (`0..=1`).
    pub subsoil_share: f64,
}

/// Vertically resolved subsoil SOC tracker (`20–30`, `30–60`, `60–100 cm`) coupled to `oxisol-core`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSomProfile<F: Float> {
    /// Root depth extinction & DOC advection profile for the active biome.
    pub root_profile: RootDepthProfile,
    /// Three coupled subsoil horizons: `[20–30 cm, 30–60 cm, 60–100 cm]`.
    pub horizons: [SubsoilHorizon<F>; SUBSOIL_HORIZONS],
    /// Cumulative C entering subsoil (`20–100 cm`) via deep roots & DOC ($\text{g C}\cdot\text{m}^{-2}$).
    pub cum_subsoil_c_in: F,
    /// Cumulative heterotrophic respiration leaving subsoil ($\text{g C}\cdot\text{m}^{-2}$).
    pub cum_subsoil_c_resp: F,
    /// Initial subsoil total carbon at creation (for exact conservation auditing).
    pub initial_subsoil_c: F,
}

impl<F: Float> DeepSomProfile<F> {
    /// Initialize subsoil horizons in realistic pedological proportion to initial `0–20 cm` `somsc`
    /// using the biome's Jackson (1996) / Jobbágy & Jackson (2000) vertical distribution.
    pub fn initialize(somsc_0_20: F, clay: F, preset_id: &str) -> Self {
        let f = F::from_f64;
        let root_profile = RootDepthProfile::for_preset(preset_id);
        let f_top20 = root_profile.layer_root_fraction(0.0, 20.0).max(0.15);
        // Clay/iron-oxide rich soils (Oxisols) stabilize slightly more passive carbon at depth.
        let clay_boost = 1.0 + 0.25 * clay.to_f64().clamp(0.0, 0.85);
        let som_top = somsc_0_20.to_f64().max(50.0);

        let mut horizons = [SubsoilHorizon::default(); SUBSOIL_HORIZONS];
        let mut init_tot = 0.0_f64;
        for (i, &(d_top, d_bot)) in SUBSOIL_BOUNDS_CM.iter().enumerate() {
            let f_h = root_profile.layer_root_fraction(d_top, d_bot);
            // Jobbagy & Jackson (2000): SOC decreases more slowly with depth than roots alone
            // because subsoil turnover slows exponentially with depth.
            let depth_stab = (SUBSOIL_MID_CM[i] / 25.0).powf(0.18) * clay_boost;
            let h_tot = som_top * (f_h / f_top20) * 0.72 * depth_stab;
            // Deeper horizons carry a higher share of mineral-associated passive SOM3.
            let f1 = 0.03 * (-SUBSOIL_MID_CM[i] / 60.0).exp();
            let f3 = (0.48 + 0.003 * SUBSOIL_MID_CM[i]).clamp(0.45, 0.78);
            let f2 = (1.0 - f1 - f3).max(0.15);
            horizons[i] = SubsoilHorizon {
                som1: f(h_tot * f1),
                som2: f(h_tot * f2),
                som3: f(h_tot * f3),
            };
            init_tot += h_tot;
        }

        Self {
            root_profile,
            horizons,
            cum_subsoil_c_in: F::ZERO,
            cum_subsoil_c_resp: F::ZERO,
            initial_subsoil_c: f(init_tot),
        }
    }

    /// Switch biome root depth profile dynamically (e.g. during Forest → Pasture LULC transition)
    /// while conserving 100% of existing subsoil carbon stocks.
    pub fn set_preset(&mut self, preset_id: &str) {
        self.root_profile = RootDepthProfile::for_preset(preset_id);
    }

    /// Advance the 3 coupled subsoil horizons (`20–30`, `30–60`, `60–100 cm`) by one monthly step.
    ///
    /// Inputs:
    /// - `month_npp`: monthly ecosystem NPP ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$)
    /// - `somsc_0_20`: current `0–20 cm` surface SOMSC ($\text{g C}\cdot\text{m}^{-2}$)
    /// - `defac`: surface monthly abiotic decomposition factor (`0..1`)
    /// - `anerb`: anaerobic drainage factor (`0..1`)
    /// - `clay`: soil clay fraction (`0..1`)
    /// - `drain_flux_cm`: monthly water percolation leaving the 30 cm layer (cm)
    ///
    /// Returns monthly subsoil heterotrophic respiration ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$).
    pub fn step_month(
        &mut self,
        month_npp: F,
        somsc_0_20: F,
        defac: F,
        anerb: F,
        clay: F,
        drain_flux_cm: F,
    ) -> F {
        let f = F::from_f64;
        let dt = 1.0 / 12.0;
        let npp = month_npp.to_f64().max(0.0);
        let top_c = somsc_0_20.to_f64().max(0.0);
        let defac_v = defac.to_f64().clamp(0.001, 2.0);
        let anerb_v = anerb.to_f64().clamp(0.05, 1.0);
        let clay_v = clay.to_f64().clamp(0.0, 0.85);
        let perc_scale = (0.35 + (drain_flux_cm.to_f64().max(0.0) / 4.0).min(1.5)).min(1.85);

        // Clay / iron-oxide stabilization in weathered soils slows active->resp and boosts passive accrual.
        let r1_resp = (0.62 - 0.28 * clay_v).clamp(0.32, 0.72);
        let f1_to_3 = (0.004 + 0.055 * clay_v).clamp(0.004, 0.06);
        let f2_to_3 = (0.005 + 0.035 * clay_v).clamp(0.005, 0.04);

        // Fraction of NPP allocated to deep belowground root turnover + exudation below 20 cm
        // (~38% of NPP goes belowground, distributed by Jackson 1996 horizon fractions).
        let belowground_input_total = npp * 0.36;

        // Vertical DOC + bioturbation cascade from surface 0–20 cm downward.
        let mut incoming_doc = top_c * self.root_profile.doc_advect_yr * dt * perc_scale * 0.14;
        let mut total_resp = 0.0_f64;

        for i in 0..SUBSOIL_HORIZONS {
            let (d_top, d_bot) = SUBSOIL_BOUNDS_CM[i];
            let z_mid = SUBSOIL_MID_CM[i];
            let depth_att = (-z_mid / Z_TAU_CM).exp(); // Koven et al. (2013)

            // 1. Deep root exudation & fine-root detritus input into this horizon.
            let f_root_h = self.root_profile.layer_root_fraction(d_top, d_bot);
            let root_in = belowground_input_total * f_root_h;
            let total_in = root_in + incoming_doc;

            // Only root_in + top-layer DOC (i == 0) enter the subsoil domain from outside.
            if i == 0 {
                self.cum_subsoil_c_in += f(total_in);
            } else {
                self.cum_subsoil_c_in += f(root_in);
            }

            let mut s1 = self.horizons[i].som1.to_f64() + root_in * 0.55 + incoming_doc * 0.65;
            let mut s2 = self.horizons[i].som2.to_f64() + root_in * 0.45 + incoming_doc * 0.35;
            let mut s3 = self.horizons[i].som3.to_f64();

            // 2. Depth-attenuated first-order decay (CENTURY k1=7.3, k2=0.20, k3=0.0045 yr^-1).
            let env = defac_v * anerb_v * depth_att * dt;
            let d1 = (s1 * 7.3 * env * (1.0 - 0.55 * clay_v)).min(s1 * 0.85);
            let d2 = (s2 * 0.20 * env).min(s2 * 0.50);
            let d3 = (s3 * 0.0045 * env).min(s3 * 0.10);

            // Downward DOC transfer to next deeper horizon (only from active pool; bottom horizon retains).
            let doc_out = if i + 1 < SUBSOIL_HORIZONS {
                (d1 * 0.08 * perc_scale).min(d1 * 0.20)
            } else {
                0.0
            };

            let resp1 = d1 * r1_resp;
            let s1_to_3 = d1 * f1_to_3;
            let s1_to_2 = (d1 - resp1 - s1_to_3 - doc_out).max(0.0);

            let resp2 = d2 * 0.55;
            let s2_to_3 = d2 * f2_to_3;
            let s2_to_1 = (d2 - resp2 - s2_to_3).max(0.0);

            let resp3 = d3 * 0.55;
            let s3_to_1 = (d3 - resp3).max(0.0);

            s1 = (s1 - d1 + s2_to_1 + s3_to_1).max(0.0);
            s2 = (s2 - d2 + s1_to_2).max(0.0);
            s3 = (s3 - d3 + s1_to_3 + s2_to_3).max(0.0);

            self.horizons[i] = SubsoilHorizon {
                som1: f(s1),
                som2: f(s2),
                som3: f(s3),
            };

            let h_resp = resp1 + resp2 + resp3;
            total_resp += h_resp;
            incoming_doc = doc_out;
        }

        self.cum_subsoil_c_resp += f(total_resp);
        f(total_resp)
    }

    /// Total carbon currently stored across the `20–100 cm` subsoil horizons ($\text{g C}\cdot\text{m}^{-2}$).
    pub fn subsoil_total(&self) -> F {
        self.horizons[0].total() + self.horizons[1].total() + self.horizons[2].total()
    }

    /// Exact subsoil mass-balance residual:
    /// `|(current_subsoil - initial_subsoil) - (cum_in - cum_resp)|`.
    pub fn mass_balance_error(&self) -> f64 {
        let lhs = self.subsoil_total().to_f64() - self.initial_subsoil_c.to_f64();
        let rhs = self.cum_subsoil_c_in.to_f64() - self.cum_subsoil_c_resp.to_f64();
        (lhs - rhs).abs()
    }

    /// Produce full multi-depth summary combining classic `0–20 cm` with `20–100 cm` subsoil.
    pub fn summary(&self, somsc_0_20: F) -> DepthSomSummary {
        let s0 = somsc_0_20.to_f64();
        let s1 = self.horizons[0].total().to_f64();
        let s2 = self.horizons[1].total().to_f64();
        let s3 = self.horizons[2].total().to_f64();
        let s_30 = s0 + s1;
        let s_100 = s_30 + s2 + s3;
        DepthSomSummary {
            somsc_0_20: s0,
            somsc_20_30: s1,
            somsc_30_60: s2,
            somsc_60_100: s3,
            somsc_0_30: s_30,
            somsc_0_100: s_100,
            subsoil_share: if s_100 > 1.0e-6 {
                (s1 + s2 + s3) / s_100
            } else {
                0.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_som_conserves_mass_and_differentiates_forest_pasture_vs_cropland() {
        let mut forest = DeepSomProfile::<f64>::initialize(3500.0, 0.55, "amazon_rainforest");
        let mut pasture =
            DeepSomProfile::<f64>::initialize(2600.0, 0.48, "tropical_pasture_urochloa");
        let mut crop = DeepSomProfile::<f64>::initialize(2600.0, 0.25, "cropland_corn_wheat");

        for _ in 0..600 {
            forest.step_month(55.0, 3500.0, 0.65, 1.0, 0.55, 4.0);
            pasture.step_month(40.0, 2600.0, 0.55, 1.0, 0.48, 2.0);
            crop.step_month(40.0, 2600.0, 0.55, 1.0, 0.25, 2.0);
        }

        assert!(forest.mass_balance_error() < 1.0e-9);
        assert!(pasture.mass_balance_error() < 1.0e-9);
        assert!(crop.mass_balance_error() < 1.0e-9);

        let sf = forest.summary(3500.0);
        let sp = pasture.summary(2600.0);
        let sc = crop.summary(2600.0);

        // Tropical forest & deep-rooted Urochloa pasture store a larger subsoil share than shallow crops.
        assert!(
            sf.subsoil_share > sc.subsoil_share,
            "forest subsoil_share={} should exceed crop={}",
            sf.subsoil_share,
            sc.subsoil_share
        );
        assert!(
            sp.subsoil_share > sc.subsoil_share,
            "tropical pasture subsoil_share={} should exceed crop={}",
            sp.subsoil_share,
            sc.subsoil_share
        );
        assert!(sf.somsc_0_100 > sf.somsc_0_30 && sf.somsc_0_30 > sf.somsc_0_20);
    }
}
