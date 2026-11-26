//! Block-based simulation schedule engine (`.sch` semantics).
//!
//! Specification: `docs/spec/06_management_and_scheduling.md` §1.

use serde::{Deserialize, Serialize};

use crate::config::{IsotopeMode, SystemKind};

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
}
