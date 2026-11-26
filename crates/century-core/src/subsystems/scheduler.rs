//! Block-based simulation schedule engine (`.sch` semantics).
//!
//! Specification: `docs/spec/06_management_and_scheduling.md` §1.

use serde::{Deserialize, Serialize};

use crate::config::{IsotopeMode, SystemKind};
use crate::float::Float;
use crate::params::{
    CropParams, CultParams, FertParams, FireParams, GrazParams, HarvParams, IrriParams, OmadParams,
    RawBiomeParams, TreeParams, TremParams,
};

/// All event commands supported in CENTURY schedule blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    /// Select active crop/grassland parameter set.
    Crop(String),
    /// Start herbaceous growing season (`FRST`).
    Frst,
    /// End herbaceous growing season (`LAST`).
    Last,
    /// Herbaceous senescence month (`SENM`).
    Senm,
    /// Planting month (`PLTM`).
    Pltm,
    /// Select active forest/tree parameter set.
    Tree(String),
    /// Start forest growing season (`TFST`).
    Tfst,
    /// End forest growing season (`TLST`).
    Tlst,
    /// Cultivation / tillage event (`CULT`).
    Cult(String),
    /// Fertilization event (`FERT`).
    Fert(String),
    /// Grassland/understory fire event (`FIRE`).
    Fire(String),
    /// Grazing event (`GRAZ`).
    Graz(String),
    /// Crop harvest event (`HARV`).
    Harv(String),
    /// Irrigation event (`IRRI`).
    Irri(String),
    /// Organic matter addition (`OMAD`).
    Omad(String),
    /// Tree removal / canopy burn (`TREM`).
    Trem(String),
    /// Soil erosion rate ($\text{kg}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$, `EROD`).
    Erod(f64),
}

/// Single scheduled event within a repeating block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    /// Relative year inside the block's repeat cycle (`1..=repeat_years`).
    pub rel_year: i32,
    /// Calendar month (`1..=12`).
    pub month: u8,
    /// Event command and option payload.
    pub kind: EventKind,
}

/// Sequential block of repeating management/phenology events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleBlock {
    /// 1-based block index.
    pub block_num: i32,
    /// Optional human-readable block label.
    pub label: String,
    /// Inclusive final simulation year for this block.
    pub end_year: i32,
    /// Repeat period in years (`>= 1`).
    pub repeat_years: i32,
    /// Weather mode character (`'M'` = mean, `'S'` = stochastic, `'F'` = file).
    pub weather_mode: char,
    /// Events scheduled in this block.
    pub events: Vec<ScheduledEvent>,
}

/// Parsed complete simulation schedule (`.sch`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Starting simulation year (1-based).
    pub start_year: i32,
    /// Last simulation year (inclusive).
    pub last_year: i32,
    /// Site parameter file name (e.g. `"lu3.100"`).
    pub site_file: String,
    /// Isotope labeling mode.
    pub isotope: IsotopeMode,
    /// Year when isotope labeling begins (`-1` if none).
    pub label_year: i32,
    /// Initial ecosystem system type.
    pub initial_system: SystemKind,
    /// Initial active crop option name.
    pub initial_crop: String,
    /// Initial active tree option name.
    pub initial_tree: String,
    /// Sequential blocks covering `[start_year, last_year]`.
    pub blocks: Vec<ScheduleBlock>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            start_year: 1,
            last_year: 100,
            site_file: "site.100".to_string(),
            isotope: IsotopeMode::Unlabeled,
            label_year: -1,
            initial_system: SystemKind::GrasslandCrop,
            initial_crop: "TMC3".to_string(),
            initial_tree: String::new(),
            blocks: vec![ScheduleBlock {
                block_num: 1,
                label: "Default Grassland".to_string(),
                end_year: 100,
                repeat_years: 1,
                weather_mode: 'M',
                events: vec![
                    ScheduledEvent {
                        rel_year: 1,
                        month: 4,
                        kind: EventKind::Frst,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 10,
                        kind: EventKind::Senm,
                    },
                    ScheduledEvent {
                        rel_year: 1,
                        month: 10,
                        kind: EventKind::Last,
                    },
                ],
            }],
        }
    }
}

impl Schedule {
    /// Resolve all events scheduled for `(year, month)` across sequential blocks
    /// into a caller-provided stack buffer (zero heap allocation in the hot loop).
    pub fn events_for_buf<'a, const N: usize>(
        &'a self,
        year: i32,
        month: u8,
        out: &mut [Option<&'a EventKind>; N],
    ) -> usize {
        let mut count = 0usize;
        let mut block_start = self.start_year;
        for blk in &self.blocks {
            if year >= block_start && year <= blk.end_year {
                let rpt = blk.repeat_years.max(1);
                let rel_year = ((year - block_start) % rpt) + 1;
                for ev in &blk.events {
                    if ev.rel_year == rel_year && ev.month == month && count < N {
                        out[count] = Some(&ev.kind);
                        count += 1;
                    }
                }
                return count;
            }
            block_start = blk.end_year + 1;
        }
        count
    }

    /// Resolve all events scheduled for `(year, month)` across sequential blocks.
    pub fn events_for<'a>(&'a self, year: i32, month: u8, out: &mut Vec<&'a EventKind>) {
        out.clear();
        let mut block_start = self.start_year;
        for blk in &self.blocks {
            if year >= block_start && year <= blk.end_year {
                let rpt = blk.repeat_years.max(1);
                let rel_year = ((year - block_start) % rpt) + 1;
                for ev in &blk.events {
                    if ev.rel_year == rel_year && ev.month == month {
                        out.push(&ev.kind);
                    }
                }
                return;
            }
            block_start = blk.end_year + 1;
        }
    }

    /// Pre-index all [`ScheduleBlock`]s by `(rel_year, month)` and pre-resolve all
    /// biome parameter references for O(1), zero-hash, zero-scan monthly lookup.
    pub fn compile<'a, F: Float>(
        &'a self,
        biome: &'a RawBiomeParams<F>,
    ) -> CompiledSchedule<'a, F> {
        CompiledSchedule::build(self, biome)
    }
}

/// Pre-resolved events for a single `(rel_year, month)` slot inside a [`CompiledBlock`].
#[derive(Debug, Clone, Copy)]
pub struct ResolvedMonthEvents<'a, F: Float> {
    /// Crop selection change `(name, resolved params)`.
    pub crop: Option<(&'a str, Option<&'a CropParams<F>>)>,
    /// Tree selection change `(name, resolved params)`.
    pub tree: Option<(&'a str, Option<&'a TreeParams<F>>)>,
    /// Crop growing-season flag update (`FRST`/`PLTM` -> `Some(true)`, `LAST` -> `Some(false)`).
    pub set_crpgrw: Option<bool>,
    /// Forest growing-season flag update (`TFST` -> `Some(true)`, `TLST` -> `Some(false)`).
    pub set_forgrw: Option<bool>,
    /// Senescence month flag (`SENM`).
    pub senm: bool,
    /// Planting month flag (`PLTM`).
    pub planted: bool,
    /// Pre-resolved irrigation parameters (`IRRI`).
    pub irri: Option<&'a IrriParams<F>>,
    /// Pre-resolved fertilizer parameters (`FERT`).
    pub fert: Option<&'a FertParams<F>>,
    /// Pre-resolved organic amendment parameters (`OMAD`).
    pub omad: Option<&'a OmadParams<F>>,
    /// Pre-resolved cultivation parameters (`CULT`).
    pub cult: Option<&'a CultParams<F>>,
    /// Pre-resolved grazing parameters (`GRAZ`).
    pub graz: Option<&'a GrazParams<F>>,
    /// Pre-resolved harvest parameters (`HARV`).
    pub harv: Option<&'a HarvParams<F>>,
    /// Pre-resolved fire parameters (`FIRE`).
    pub fire: Option<&'a FireParams<F>>,
    /// Pre-resolved tree removal parameters (`TREM`).
    pub trem: Option<&'a TremParams<F>>,
    /// Soil erosion rate (`EROD`).
    pub erod: Option<F>,
    /// Fast-path flag indicating whether any management disturbance occurs this month.
    pub has_any_mgmt: bool,
}

impl<'a, F: Float> Default for ResolvedMonthEvents<'a, F> {
    #[inline]
    fn default() -> Self {
        Self {
            crop: None,
            tree: None,
            set_crpgrw: None,
            set_forgrw: None,
            senm: false,
            planted: false,
            irri: None,
            fert: None,
            omad: None,
            cult: None,
            graz: None,
            harv: None,
            fire: None,
            trem: None,
            erod: None,
            has_any_mgmt: false,
        }
    }
}

impl<'a, F: Float> ResolvedMonthEvents<'a, F> {
    fn apply_event(&mut self, kind: &'a EventKind, biome: &'a RawBiomeParams<F>) {
        match kind {
            EventKind::Crop(name) => {
                self.crop = Some((name.as_str(), biome.crops.get(name.as_str())));
            }
            EventKind::Tree(name) => {
                self.tree = Some((name.as_str(), biome.trees.get(name.as_str())));
            }
            EventKind::Frst => self.set_crpgrw = Some(true),
            EventKind::Last => self.set_crpgrw = Some(false),
            EventKind::Senm => self.senm = true,
            EventKind::Pltm => {
                self.set_crpgrw = Some(true);
                self.planted = true;
            }
            EventKind::Tfst => self.set_forgrw = Some(true),
            EventKind::Tlst => self.set_forgrw = Some(false),
            EventKind::Irri(n) => {
                self.irri = biome.irris.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Fert(n) => {
                self.fert = biome.ferts.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Omad(n) => {
                self.omad = biome.omads.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Cult(n) => {
                self.cult = biome.cults.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Graz(n) => {
                self.graz = biome.grazs.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Harv(n) => {
                self.harv = biome.harvs.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Fire(n) => {
                self.fire = biome.fires.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Trem(n) => {
                self.trem = biome.trems.get(n.as_str());
                self.has_any_mgmt = true;
            }
            EventKind::Erod(r) => {
                self.erod = Some(F::from_f64(*r));
                self.has_any_mgmt = true;
            }
        }
    }
}

/// Pre-indexed schedule block mapping `(rel_year, month)` in O(1) time.
#[derive(Debug, Clone)]
enum BlockSlots<'a, F: Float> {
    /// Dense table indexed by `(rel_year - 1) * 12 + (month - 1)` for `repeat_years <= 256`.
    Dense(Vec<ResolvedMonthEvents<'a, F>>),
    /// Sparse O(1) map for very long repeat periods (`repeat_years > 256`).
    Sparse(std::collections::HashMap<u32, ResolvedMonthEvents<'a, F>>),
}

/// Single pre-indexed block inside a [`CompiledSchedule`].
#[derive(Debug, Clone)]
pub struct CompiledBlock<'a, F: Float> {
    /// Inclusive first simulation year covered by this block.
    pub start_year: i32,
    /// Inclusive final simulation year covered by this block.
    pub end_year: i32,
    /// Repeat period in years (`>= 1`).
    pub repeat_years: i32,
    slots: BlockSlots<'a, F>,
}

impl<'a, F: Float> CompiledBlock<'a, F> {
    /// Return pre-resolved events for `(year, month)` in O(1) time.
    #[inline(always)]
    pub fn events_for(&self, year: i32, month: u8) -> ResolvedMonthEvents<'a, F> {
        if !(1..=12).contains(&month) {
            return ResolvedMonthEvents::default();
        }
        let rel_year_0 = ((year - self.start_year) % self.repeat_years) as usize;
        let slot = rel_year_0 * 12 + (month - 1) as usize;
        match &self.slots {
            BlockSlots::Dense(v) => v.get(slot).copied().unwrap_or_default(),
            BlockSlots::Sparse(m) => m.get(&(slot as u32)).copied().unwrap_or_default(),
        }
    }
}

/// Pre-indexed [`Schedule`] with O(1) block cursor and O(1) `(rel_year, month)` event lookup.
///
/// Built once per cell (or per biome) to eliminate all per-month linear block/event scans
/// and `HashMap<String, _>` option lookups in multi-century spatial grid runs.
#[derive(Debug, Clone)]
pub struct CompiledSchedule<'a, F: Float> {
    /// Year when isotope labeling begins (`-1` if none).
    pub label_year: i32,
    /// Sequential pre-indexed blocks.
    pub blocks: Vec<CompiledBlock<'a, F>>,
}

impl<'a, F: Float> CompiledSchedule<'a, F> {
    /// Build a pre-indexed schedule from a [`Schedule`] and its biome parameter tables.
    pub fn build(schedule: &'a Schedule, biome: &'a RawBiomeParams<F>) -> Self {
        let mut blocks = Vec::with_capacity(schedule.blocks.len());
        let mut block_start = schedule.start_year;
        for blk in &schedule.blocks {
            let rpt = blk.repeat_years.max(1);
            let slots = if rpt <= 256 {
                let mut dense = vec![ResolvedMonthEvents::default(); (rpt as usize) * 12];
                for ev in &blk.events {
                    if ev.rel_year >= 1 && ev.rel_year <= rpt && (1..=12).contains(&ev.month) {
                        let slot = (ev.rel_year - 1) as usize * 12 + (ev.month - 1) as usize;
                        dense[slot].apply_event(&ev.kind, biome);
                    }
                }
                BlockSlots::Dense(dense)
            } else {
                let mut sparse = std::collections::HashMap::with_capacity(blk.events.len());
                for ev in &blk.events {
                    if ev.rel_year >= 1 && ev.rel_year <= rpt && (1..=12).contains(&ev.month) {
                        let slot = ((ev.rel_year - 1) as u32) * 12 + (ev.month - 1) as u32;
                        sparse
                            .entry(slot)
                            .or_insert_with(ResolvedMonthEvents::default)
                            .apply_event(&ev.kind, biome);
                    }
                }
                BlockSlots::Sparse(sparse)
            };
            blocks.push(CompiledBlock {
                start_year: block_start,
                end_year: blk.end_year,
                repeat_years: rpt,
                slots,
            });
            block_start = blk.end_year + 1;
        }
        Self {
            label_year: schedule.label_year,
            blocks,
        }
    }

    /// Look up pre-resolved events for `(year, month)` in O(1) amortized time using a
    /// caller-maintained block cursor (which monotonically tracks the active block).
    #[inline(always)]
    pub fn events_for_cursor(
        &self,
        year: i32,
        month: u8,
        cursor: &mut usize,
    ) -> ResolvedMonthEvents<'a, F> {
        // Fast path: current cursor matches `year`.
        if let Some(blk) = self.blocks.get(*cursor) {
            if year >= blk.start_year && year <= blk.end_year {
                return blk.events_for(year, month);
            }
            // Common monotonic advance into the next sequential block.
            if year > blk.end_year {
                let mut next = *cursor + 1;
                while let Some(nb) = self.blocks.get(next) {
                    if year <= nb.end_year {
                        *cursor = next;
                        return if year >= nb.start_year {
                            nb.events_for(year, month)
                        } else {
                            ResolvedMonthEvents::default()
                        };
                    }
                    next += 1;
                }
                *cursor = self.blocks.len();
                return ResolvedMonthEvents::default();
            }
        }
        // Fallback for out-of-order year queries: binary search by `end_year`.
        let idx = self.blocks.partition_point(|b| b.end_year < year);
        *cursor = idx;
        if let Some(blk) = self.blocks.get(idx) {
            if year >= blk.start_year && year <= blk.end_year {
                return blk.events_for(year, month);
            }
        }
        ResolvedMonthEvents::default()
    }
}
