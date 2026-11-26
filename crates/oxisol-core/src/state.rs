//! Compact, cache-line-aligned per-cell state (`CellState<F>`) and I/O record types.
//!
//! `CellState<f32>` is ~3 KiB and `CellState<f64>` is ~5.8 KiB — fitting cleanly
//! inside the 32 KiB L1d cache per core on both AMD Ryzen 7 5700X (Zen 3) and
//! modern multi-core server processors with zero false sharing across threads.

use serde::{Deserialize, Serialize};

use crate::config::SystemKind;
use crate::float::Float;
use crate::params::{MAX_ELEM, MAX_LAYERS};

/// Carbon isotope pair `(unlabeled, labeled)` in $\text{g C}\cdot\text{m}^{-2}$.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct CarbonPool<F: Float> {
    /// Unlabeled carbon ($\text{g C}\cdot\text{m}^{-2}$).
    pub unlab: F,
    /// Labeled carbon ($\text{g C}\cdot\text{m}^{-2}$, C-14 or C-13).
    pub lab: F,
}

impl<F: Float> CarbonPool<F> {
    /// Create a carbon pool with total carbon `c` and labeled fraction `lab_frac`.
    #[inline(always)]
    pub fn from_total(c: F, lab_frac: F) -> Self {
        let lf = lab_frac.clamp_val(F::ZERO, F::ONE);
        Self {
            unlab: c * (F::ONE - lf),
            lab: c * lf,
        }
    }

    /// Total carbon (`unlab + lab`).
    #[inline(always)]
    pub fn total(self) -> F {
        self.unlab + self.lab
    }

    /// Fraction of carbon that is isotopically labeled (`lab / total`).
    #[inline(always)]
    pub fn labeled_fraction(self) -> F {
        let tot = self.total();
        if tot > F::EPS_GUARD {
            (self.lab / tot).clamp_val(F::ZERO, F::ONE)
        } else {
            F::ZERO
        }
    }

    /// Add carbon `delta` with labeled fraction `lab_frac`.
    #[inline(always)]
    pub fn add(&mut self, delta: F, lab_frac: F) {
        if lab_frac == F::ZERO {
            self.unlab += delta;
            return;
        }
        let lf = lab_frac.clamp_val(F::ZERO, F::ONE);
        self.unlab += delta * (F::ONE - lf);
        self.lab += delta * lf;
    }

    /// Remove carbon `amount` preserving the pool's isotopic ratio; returns actual removed.
    #[inline(always)]
    pub fn sub_preserving_ratio(&mut self, amount: F) -> (F, F) {
        if self.lab == F::ZERO {
            if self.unlab <= F::EPS_GUARD || amount <= F::ZERO {
                return (F::ZERO, F::ZERO);
            }
            let actual = amount.min(self.unlab);
            self.unlab = (self.unlab - actual).max(F::ZERO);
            return (actual, F::ZERO);
        }
        let tot = self.total();
        if tot <= F::EPS_GUARD || amount <= F::ZERO {
            return (F::ZERO, F::ZERO);
        }
        let actual = amount.min(tot);
        let lf = self.labeled_fraction();
        let d_lab = actual * lf;
        let d_unlab = actual - d_lab;
        self.unlab = (self.unlab - d_unlab).max(F::ZERO);
        self.lab = (self.lab - d_lab).max(F::ZERO);
        (actual, lf)
    }
}

/// Complete biogeochemical and hydrological state of a single spatial cell.
///
/// Aligned to 64-byte cache lines (`#[repr(C, align(64))]`) to eliminate
/// false sharing across concurrent worker threads on high-core-count CPUs.
#[repr(C, align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellState<F: Float> {
    // --- Litter & Soil Organic Matter Pools (g/m2) ---
    /// Structural litter C `[0=surface, 1=soil]`.
    pub strucc: [CarbonPool<F>; 2],
    /// Structural litter nutrients `[0=surface, 1=soil][N, P, S]`.
    pub struce: [[F; MAX_ELEM]; 2],
    /// Structural litter lignin fraction `[0=surface, 1=soil]`.
    pub strlig: [F; 2],
    /// Metabolic litter C `[0=surface, 1=soil]`.
    pub metabc: [CarbonPool<F>; 2],
    /// Metabolic litter nutrients `[0=surface, 1=soil][N, P, S]`.
    pub metabe: [[F; MAX_ELEM]; 2],
    /// Active SOM C `[0=surface microbes, 1=soil active]`.
    pub som1c: [CarbonPool<F>; 2],
    /// Active SOM nutrients `[0=surface microbes, 1=soil active][N, P, S]`.
    pub som1e: [[F; MAX_ELEM]; 2],
    /// Slow SOM C.
    pub som2c: CarbonPool<F>,
    /// Slow SOM nutrients `[N, P, S]`.
    pub som2e: [F; MAX_ELEM],
    /// Passive SOM C.
    pub som3c: CarbonPool<F>,
    /// Passive SOM nutrients `[N, P, S]`.
    pub som3e: [F; MAX_ELEM],

    // --- Herbaceous (Grassland/Crop) Pools (g/m2) ---
    /// Aboveground live shoot C.
    pub aglivc: CarbonPool<F>,
    /// Aboveground live shoot nutrients `[N, P, S]`.
    pub aglive: [F; MAX_ELEM],
    /// Belowground live root C.
    pub bglivc: CarbonPool<F>,
    /// Belowground live root nutrients `[N, P, S]`.
    pub bglive: [F; MAX_ELEM],
    /// Standing dead shoot C.
    pub stdedc: CarbonPool<F>,
    /// Standing dead shoot nutrients `[N, P, S]`.
    pub stdede: [F; MAX_ELEM],
    /// Internal crop nutrient storage `[N, P, S]`.
    pub crpstg: [F; MAX_ELEM],
    /// Current year's plant residue lignin `[0=shoot, 1=root]`.
    pub pltlig: [F; 2],

    // --- Forest / Savanna Woody Pools (g/m2) ---
    /// Live foliage C.
    pub rleavc: CarbonPool<F>,
    /// Live foliage nutrients `[N, P, S]`.
    pub rleave: [F; MAX_ELEM],
    /// Live fine root C.
    pub frootc: CarbonPool<F>,
    /// Live fine root nutrients `[N, P, S]`.
    pub froote: [F; MAX_ELEM],
    /// Live fine branch C.
    pub fbrchc: CarbonPool<F>,
    /// Live fine branch nutrients `[N, P, S]`.
    pub fbrche: [F; MAX_ELEM],
    /// Live large wood (bole) C.
    pub rlwodc: CarbonPool<F>,
    /// Live large wood nutrients `[N, P, S]`.
    pub rlwode: [F; MAX_ELEM],
    /// Live coarse root C.
    pub crootc: CarbonPool<F>,
    /// Live coarse root nutrients `[N, P, S]`.
    pub croote: [F; MAX_ELEM],
    /// Internal forest nutrient storage `[N, P, S]`.
    pub forstg: [F; MAX_ELEM],
    /// Dead fine branch C (`wood1`).
    pub wood1c: CarbonPool<F>,
    /// Dead fine branch nutrients `[N, P, S]`.
    pub wood1e: [F; MAX_ELEM],
    /// Dead large wood C (`wood2`).
    pub wood2c: CarbonPool<F>,
    /// Dead large wood nutrients `[N, P, S]`.
    pub wood2e: [F; MAX_ELEM],
    /// Dead coarse root C (`wood3`).
    pub wood3c: CarbonPool<F>,
    /// Dead coarse root nutrients `[N, P, S]`.
    pub wood3e: [F; MAX_ELEM],
    /// Dead wood lignin fractions `[wood1, wood2, wood3]`.
    pub wdlig: [F; 3],

    // --- Inorganic Mineral & Geochemical Pools (g/m2) ---
    /// Plant-available mineral nutrients `[layer 0..10][N, P, S]`.
    pub minerl: [[F; MAX_ELEM]; MAX_LAYERS],
    /// Weatherable parent mineral `[N, P, S]`.
    pub parent: [F; MAX_ELEM],
    /// Secondary sorbed mineral `[N, P, S]`.
    pub secndy: [F; MAX_ELEM],
    /// Occluded inorganic phosphorus.
    pub occlud: F,

    // --- Soil Water & Snow State (cm) ---
    /// Volumetric water storage per soil layer + deep layer (`0..=MAX_LAYERS`).
    pub asmos: [F; MAX_LAYERS + 1],
    /// Available soil water indices `[0=root_zone, 1=profile, 2=top_30cm]`.
    pub avh2o: [F; 3],
    /// Snowpack water equivalent (cm).
    pub snow: F,
    /// Liquid water stored inside snowpack (cm).
    pub snlq: F,

    // --- Dynamic Environment & Phenology State ---
    /// Active ecosystem type.
    pub system: SystemKind,
    /// Active crop option name index/flag (`crpgrw`).
    pub crpgrw: bool,
    /// Active forest growing season flag (`forgrw`).
    pub forgrw: bool,
    /// Active senescence month flag (`senm`).
    pub senm: bool,
    /// Whether deciduous forest autumn leaf abscission has already occurred this year.
    #[serde(default)]
    pub decid_dropped: bool,
    /// Months since last cultivation (for decaying `clteff`).
    pub cult_months: u8,
    /// Active cultivation decomposition multipliers `[som1, som2, som3, structural]` (Manual A2-32).
    pub clteff: [F; 4],
    /// Post-fire temporary root:shoot boost (decays annually).
    pub fire_frtsh: F,
    /// Current monthly soil temperature (°C).
    pub stemp: F,
    /// Current monthly abiotic decomposition factor `defac`.
    pub defac: F,
    /// Current monthly anaerobic decomposition modifier `anerb`.
    pub anerb: F,
    /// Current monthly PET (cm).
    pub pet: F,
    /// Current monthly actual ET (cm).
    pub aet: F,
    /// Current monthly runoff/streamflow (cm).
    pub stream: F,
    /// Current atmospheric CO2 (ppm).
    pub co2_ppm: F,
    /// Current labeled fraction of atmospheric CO2 (for C14/C13).
    pub atm_lab_frac: F,
    /// Rolling 12-month precipitation sum (cm) for annual allocation/fixation.
    pub ann_ppt: F,
    /// Previous year's cumulative actual evapotranspiration (cm/yr) for non-symbiotic N fixation.
    pub ann_aet: F,
    /// Cumulative growing-season transpiration ratio accumulator for grain HI.
    pub grain_tr_sum: F,
    /// Count of grain-fill months accumulated.
    pub grain_tr_months: u8,

    // --- Cumulative System Boundary Counters (for exact mass balance auditing) ---
    /// Cumulative carbon entering the ecosystem (NPP + OMAD).
    pub cum_c_in: F,
    /// Cumulative carbon leaving the ecosystem (Resp + Harvest + Fire + Graze + Erosion + Leach).
    pub cum_c_out: F,
    /// Cumulative nutrients entering `[N, P, S]`.
    pub cum_e_in: [F; MAX_ELEM],
    /// Cumulative nutrients leaving `[N, P, S]`.
    pub cum_e_out: [F; MAX_ELEM],

    // --- Current Month Diagnostics ---
    /// Monthly heterotrophic respiration ($\text{g C}\cdot\text{m}^{-2}$).
    pub month_resp: F,
    /// Monthly net primary production (herbaceous + woody, $\text{g C}\cdot\text{m}^{-2}$).
    pub month_npp: F,
    /// Monthly herbaceous NPP.
    pub month_crop_npp: F,
    /// Monthly forest NPP.
    pub month_forest_npp: F,
    /// Monthly net nutrient mineralization `[N, P, S]`.
    pub month_net_min: [F; MAX_ELEM],
    /// Monthly nutrient leaching `[N, P, S]`.
    pub month_leach_e: [F; MAX_ELEM],
    /// Monthly organic carbon leaching.
    pub month_leach_c: F,
    /// Monthly N volatilization.
    pub month_n_volat: F,
    /// Monthly N fixation (symbiotic + non-symbiotic).
    pub month_n_fix: F,
    /// Monthly grain C harvested.
    pub month_grain_c: F,
    /// Monthly C removed by fire/grazing/harvest/erosion.
    pub month_c_removed: F,
}

impl<F: Float> Default for CellState<F> {
    fn default() -> Self {
        Self {
            strucc: [CarbonPool::default(); 2],
            struce: [[F::ZERO; MAX_ELEM]; 2],
            strlig: [F::from_f64(0.2); 2],
            metabc: [CarbonPool::default(); 2],
            metabe: [[F::ZERO; MAX_ELEM]; 2],
            som1c: [CarbonPool::default(); 2],
            som1e: [[F::ZERO; MAX_ELEM]; 2],
            som2c: CarbonPool::default(),
            som2e: [F::ZERO; MAX_ELEM],
            som3c: CarbonPool::default(),
            som3e: [F::ZERO; MAX_ELEM],
            aglivc: CarbonPool::default(),
            aglive: [F::ZERO; MAX_ELEM],
            bglivc: CarbonPool::default(),
            bglive: [F::ZERO; MAX_ELEM],
            stdedc: CarbonPool::default(),
            stdede: [F::ZERO; MAX_ELEM],
            crpstg: [F::ZERO; MAX_ELEM],
            pltlig: [F::from_f64(0.15), F::from_f64(0.20)],
            rleavc: CarbonPool::default(),
            rleave: [F::ZERO; MAX_ELEM],
            frootc: CarbonPool::default(),
            froote: [F::ZERO; MAX_ELEM],
            fbrchc: CarbonPool::default(),
            fbrche: [F::ZERO; MAX_ELEM],
            rlwodc: CarbonPool::default(),
            rlwode: [F::ZERO; MAX_ELEM],
            crootc: CarbonPool::default(),
            croote: [F::ZERO; MAX_ELEM],
            forstg: [F::ZERO; MAX_ELEM],
            wood1c: CarbonPool::default(),
            wood1e: [F::ZERO; MAX_ELEM],
            wood2c: CarbonPool::default(),
            wood2e: [F::ZERO; MAX_ELEM],
            wood3c: CarbonPool::default(),
            wood3e: [F::ZERO; MAX_ELEM],
            wdlig: [F::from_f64(0.25), F::from_f64(0.30), F::from_f64(0.30)],
            minerl: [[F::ZERO; MAX_ELEM]; MAX_LAYERS],
            parent: [F::ZERO; MAX_ELEM],
            secndy: [F::ZERO; MAX_ELEM],
            occlud: F::ZERO,
            asmos: [F::ZERO; MAX_LAYERS + 1],
            avh2o: [F::ZERO; 3],
            snow: F::ZERO,
            snlq: F::ZERO,
            system: SystemKind::GrasslandCrop,
            crpgrw: false,
            forgrw: false,
            senm: false,
            decid_dropped: false,
            cult_months: 0,
            clteff: [F::ONE; 4],
            fire_frtsh: F::ZERO,
            stemp: F::from_f64(10.0),
            defac: F::from_f64(0.5),
            anerb: F::ONE,
            pet: F::from_f64(5.0),
            aet: F::ZERO,
            stream: F::ZERO,
            co2_ppm: F::from_f64(350.0),
            atm_lab_frac: F::ZERO,
            ann_ppt: F::from_f64(70.0),
            ann_aet: F::from_f64(45.0),
            grain_tr_sum: F::ZERO,
            grain_tr_months: 0,
            cum_c_in: F::ZERO,
            cum_c_out: F::ZERO,
            cum_e_in: [F::ZERO; MAX_ELEM],
            cum_e_out: [F::ZERO; MAX_ELEM],
            month_resp: F::ZERO,
            month_npp: F::ZERO,
            month_crop_npp: F::ZERO,
            month_forest_npp: F::ZERO,
            month_net_min: [F::ZERO; MAX_ELEM],
            month_leach_e: [F::ZERO; MAX_ELEM],
            month_leach_c: F::ZERO,
            month_n_volat: F::ZERO,
            month_n_fix: F::ZERO,
            month_grain_c: F::ZERO,
            month_c_removed: F::ZERO,
        }
    }
}

impl<F: Float> CellState<F> {
    /// Total soil organic carbon (`som1c[1] + som2c + som3c`, excluding surface microbes).
    #[inline]
    pub fn somsc(&self) -> F {
        self.som1c[1].total() + self.som2c.total() + self.som3c.total()
    }

    /// Total organic carbon including surface microbes (`som1c[0] + somsc`).
    #[inline]
    pub fn somtc(&self) -> F {
        self.som1c[0].total() + self.somsc()
    }

    /// Total carbon across EVERY ecosystem pool (plant + litter + dead wood + SOM).
    pub fn total_carbon(&self) -> F {
        self.strucc[0].total()
            + self.strucc[1].total()
            + self.metabc[0].total()
            + self.metabc[1].total()
            + self.som1c[0].total()
            + self.som1c[1].total()
            + self.som2c.total()
            + self.som3c.total()
            + self.aglivc.total()
            + self.bglivc.total()
            + self.stdedc.total()
            + self.rleavc.total()
            + self.frootc.total()
            + self.fbrchc.total()
            + self.rlwodc.total()
            + self.crootc.total()
            + self.wood1c.total()
            + self.wood2c.total()
            + self.wood3c.total()
    }

    /// Total labeled carbon across EVERY ecosystem pool.
    pub fn total_labeled_carbon(&self) -> F {
        self.strucc[0].lab
            + self.strucc[1].lab
            + self.metabc[0].lab
            + self.metabc[1].lab
            + self.som1c[0].lab
            + self.som1c[1].lab
            + self.som2c.lab
            + self.som3c.lab
            + self.aglivc.lab
            + self.bglivc.lab
            + self.stdedc.lab
            + self.rleavc.lab
            + self.frootc.lab
            + self.fbrchc.lab
            + self.rlwodc.lab
            + self.crootc.lab
            + self.wood1c.lab
            + self.wood2c.lab
            + self.wood3c.lab
    }

    /// Total mass of nutrient element `e` (`0=N, 1=P, 2=S`) across all organic & inorganic pools.
    pub fn total_element(&self, e: usize) -> F {
        let mut tot = self.struce[0][e]
            + self.struce[1][e]
            + self.metabe[0][e]
            + self.metabe[1][e]
            + self.som1e[0][e]
            + self.som1e[1][e]
            + self.som2e[e]
            + self.som3e[e]
            + self.aglive[e]
            + self.bglive[e]
            + self.stdede[e]
            + self.crpstg[e]
            + self.rleave[e]
            + self.froote[e]
            + self.fbrche[e]
            + self.rlwode[e]
            + self.croote[e]
            + self.forstg[e]
            + self.wood1e[e]
            + self.wood2e[e]
            + self.wood3e[e]
            + self.parent[e]
            + self.secndy[e];
        for layer in 0..MAX_LAYERS {
            tot += self.minerl[layer][e];
        }
        if e == 1 {
            tot += self.occlud;
        }
        tot
    }

    /// Total mineral nutrient `e` across layers `0..layers`.
    #[inline]
    pub fn mineral_sum(&self, layers: usize, e: usize) -> F {
        let mut s = F::ZERO;
        for l in 0..layers.min(MAX_LAYERS) {
            s += self.minerl[l][e];
        }
        s
    }

    /// Reset per-month diagnostic accumulators before stepping.
    pub fn reset_monthly_diagnostics(&mut self) {
        self.month_resp = F::ZERO;
        self.month_npp = F::ZERO;
        self.month_crop_npp = F::ZERO;
        self.month_forest_npp = F::ZERO;
        self.month_net_min = [F::ZERO; MAX_ELEM];
        self.month_leach_e = [F::ZERO; MAX_ELEM];
        self.month_leach_c = F::ZERO;
        self.month_n_volat = F::ZERO;
        self.month_n_fix = F::ZERO;
        self.month_grain_c = F::ZERO;
        self.month_c_removed = F::ZERO;
    }
}

/// Monthly meteorological forcing for a single cell.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MonthlyWeather<F: Float> {
    /// Monthly precipitation (cm/month).
    pub precip_cm: F,
    /// Monthly mean minimum air temperature (°C).
    pub tmin_c: F,
    /// Monthly mean maximum air temperature (°C).
    pub tmax_c: F,
}

/// Compact monthly summary output for a single cell.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MonthlyOutput<F: Float> {
    /// Simulation year.
    pub year: i32,
    /// Month (`1..=12`).
    pub month: u8,
    /// Soil organic carbon (`som1c[1] + som2c + som3c`).
    pub somsc: F,
    /// Soil active SOM C (`som1c[1]`).
    pub som1c_soil: F,
    /// Slow SOM C (`som2c`).
    pub som2c: F,
    /// Passive SOM C (`som3c`).
    pub som3c: F,
    /// Live herbaceous shoots C.
    pub aglivc: F,
    /// Live herbaceous roots C.
    pub bglivc: F,
    /// Live forest foliage C.
    pub rleavc: F,
    /// Live forest large wood C.
    pub rlwodc: F,
    /// Top-layer mineral N.
    pub minerl_n0: F,
    /// Total profile water (cm).
    pub total_h2o: F,
    /// Monthly PET (cm).
    pub pet: F,
    /// Monthly AET (cm).
    pub aet: F,
    /// Monthly heterotrophic respiration ($\text{g C}\cdot\text{m}^{-2}$).
    pub resp: F,
    /// Monthly total NPP ($\text{g C}\cdot\text{m}^{-2}$).
    pub npp: F,
    /// Monthly soil temperature (°C).
    pub stemp: F,
    /// Monthly abiotic decomposition factor `defac`.
    pub defac: F,
}

/// Compact annual summary output for a single cell (cache-line aligned for parallel grid buffers).
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AnnualOutput<F: Float> {
    /// Simulation year.
    pub year: i32,
    /// End-of-year soil organic carbon (`somsc`).
    pub somsc: F,
    /// End-of-year active SOM C (`som1c[1]`).
    pub som1c_soil: F,
    /// End-of-year slow SOM C (`som2c`).
    pub som2c: F,
    /// End-of-year passive SOM C (`som3c`).
    pub som3c: F,
    /// End-of-year herbaceous live shoot C.
    pub aglivc: F,
    /// End-of-year herbaceous live root C.
    pub bglivc: F,
    /// End-of-year forest leaf C.
    pub rleavc: F,
    /// End-of-year forest large wood C.
    pub rlwodc: F,
    /// End-of-year total mineral N across profile.
    pub total_minerl_n: F,
    /// Annual cumulative NPP ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub annual_npp: F,
    /// Annual cumulative heterotrophic respiration ($\text{g C}\cdot\text{m}^{-2}\cdot\text{yr}^{-1}$).
    pub annual_resp: F,
    /// Annual cumulative AET (cm/yr).
    pub annual_aet: F,
    /// Annual cumulative PET (cm/yr).
    pub annual_pet: F,
    /// Annual cumulative streamflow (cm/yr).
    pub annual_stream: F,
}
