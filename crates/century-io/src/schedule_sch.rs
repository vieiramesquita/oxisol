//! Pure-Rust parser for CENTURY `.sch` multi-block simulation schedule files.

use std::fs;
use std::path::Path;

use century_core::{EventKind, IsotopeMode, Schedule, ScheduleBlock, ScheduledEvent, SystemKind};

use crate::error::{IoError, Result};

/// Parse a `.sch` file from disk.
pub fn parse_schedule_file<P: AsRef<Path>>(path: P) -> Result<Schedule> {
    let txt = fs::read_to_string(path)?;
    parse_schedule_str(&txt)
}

/// Parse a `.sch` file from a string slice.
pub fn parse_schedule_str(content: &str) -> Result<Schedule> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 12 {
        return Err(IoError::Parse("schedule file too short".into()));
    }

    let first_tok = |idx: usize| -> &str {
        lines
            .get(idx)
            .and_then(|l| l.split_whitespace().next())
            .unwrap_or("")
    };
    let parse_i = |idx: usize, def: i32| -> i32 { first_tok(idx).parse::<i32>().unwrap_or(def) };

    let start_year = parse_i(0, 1);
    let last_year = parse_i(1, 100);
    let site_file = first_tok(2).to_string();
    let isotope = IsotopeMode::from_code(parse_i(3, 0));
    let label_year = parse_i(4, -1);
    // Line 5: Microcosm, Line 6: CO2 Systems count, Line 7: CO2 system IDs
    let co2_cnt = parse_i(6, 0);
    let sys_line_idx = if co2_cnt > 0 { 8 } else { 7 };
    let initial_system = SystemKind::from_code(parse_i(sys_line_idx, 1));
    let initial_crop = first_tok(sys_line_idx + 1).to_string();
    let initial_tree = first_tok(sys_line_idx + 2).to_string();

    // Find "Year Month Option" marker
    let mut i = sys_line_idx + 3;
    while i < lines.len() && !lines[i].to_ascii_lowercase().contains("year") {
        i += 1;
    }
    i += 1; // Advance past "Year Month Option"

    let mut blocks = Vec::new();
    while i < lines.len() {
        // Skip empty lines
        while i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
        }
        if i + 6 >= lines.len() {
            break;
        }
        let block_num = parse_i(i, (blocks.len() + 1) as i32);
        let label = lines[i].trim().to_string();
        let end_year = parse_i(i + 1, last_year);
        let repeat_years = parse_i(i + 2, 1).max(1);
        // i+3: output starting year, i+4: output month, i+5: output interval
        let weather_mode = first_tok(i + 6).chars().next().unwrap_or('M');
        i += 7;

        let mut events = Vec::new();
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                continue;
            }
            let toks: Vec<&str> = trimmed.split_whitespace().collect();
            if toks.len() >= 2 && toks[0] == "-999" {
                i += 1;
                break;
            }
            if toks.len() >= 3 {
                if let (Ok(ry), Ok(mo)) = (toks[0].parse::<i32>(), toks[1].parse::<u8>()) {
                    let cmd = toks[2].to_ascii_uppercase();
                    let next_arg = |idx: &mut usize| -> String {
                        *idx += 1;
                        lines
                            .get(*idx)
                            .and_then(|l| l.split_whitespace().next())
                            .unwrap_or("")
                            .to_string()
                    };
                    let kind = match cmd.as_str() {
                        "CROP" => Some(EventKind::Crop(next_arg(&mut i))),
                        "TREE" => Some(EventKind::Tree(next_arg(&mut i))),
                        "CULT" => Some(EventKind::Cult(next_arg(&mut i))),
                        "FERT" => Some(EventKind::Fert(next_arg(&mut i))),
                        "FIRE" => Some(EventKind::Fire(next_arg(&mut i))),
                        "GRAZ" => Some(EventKind::Graz(next_arg(&mut i))),
                        "HARV" => Some(EventKind::Harv(next_arg(&mut i))),
                        "IRRI" => Some(EventKind::Irri(next_arg(&mut i))),
                        "OMAD" => Some(EventKind::Omad(next_arg(&mut i))),
                        "TREM" => Some(EventKind::Trem(next_arg(&mut i))),
                        "EROD" => {
                            let arg = next_arg(&mut i);
                            Some(EventKind::Erod(arg.parse::<f64>().unwrap_or(0.0)))
                        }
                        "FRST" => Some(EventKind::Frst),
                        "LAST" => Some(EventKind::Last),
                        "SENM" => Some(EventKind::Senm),
                        "PLTM" => Some(EventKind::Pltm),
                        "TFST" => Some(EventKind::Tfst),
                        "TLST" => Some(EventKind::Tlst),
                        _ => None,
                    };
                    if let Some(k) = kind {
                        events.push(ScheduledEvent {
                            rel_year: ry,
                            month: mo,
                            kind: k,
                        });
                    }
                }
            }
            i += 1;
        }

        blocks.push(ScheduleBlock {
            block_num,
            label,
            end_year,
            repeat_years,
            weather_mode,
            events,
        });
    }

    Ok(Schedule {
        start_year,
        last_year,
        site_file,
        isotope,
        label_year,
        initial_system,
        initial_crop,
        initial_tree,
        blocks,
    })
}
