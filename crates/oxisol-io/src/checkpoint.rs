//! Complete native checkpoints (JSON, zero state loss) and legacy 235-column
//! `site.nc` NetCDF-3 reader/writer for black-box oracle validation.

use std::fs;
use std::path::Path;

use oxisol_core::{CellDerivedParams, CellState, Float};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::{IoError, Result};
use crate::netcdf3::{NcFile, NcWriter};

/// Number of float columns in legacy VEMAP `site.nc`.
pub const LEGACY_SITE_NC_COLS: usize = 235;

/// Complete lossless native checkpoint for a single cell (no missing litter/lignin/forstg pools).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeCellCheckpoint<F: Float> {
    /// Grid row index.
    pub row: usize,
    /// Grid column index.
    pub col: usize,
    /// Biome/landuse ID.
    pub biome_id: i32,
    /// Complete cell state.
    pub state: CellState<F>,
    /// Derived per-cell parameters.
    pub derived: CellDerivedParams<F>,
}

/// Save a complete lossless native checkpoint grid to JSON.
pub fn write_native_checkpoint<F: Float + Serialize, P: AsRef<Path>>(
    path: P,
    cells: &[NativeCellCheckpoint<F>],
) -> Result<()> {
    let json = serde_json::to_string(cells)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load a complete lossless native checkpoint grid from JSON.
pub fn read_native_checkpoint<F: Float + DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<Vec<NativeCellCheckpoint<F>>> {
    let txt = fs::read_to_string(path)?;
    let cells = serde_json::from_str(&txt)?;
    Ok(cells)
}

/// Key biogeochemical variables extracted from a 235-column `site.nc` record.
#[derive(Debug, Clone, Copy, Default)]
pub struct SiteNcPools {
    /// Latitude (degrees).
    pub lat: f32,
    /// Longitude (degrees).
    pub lon: f32,
    /// Surface active microbes C (`som1c[0]`).
    pub som1c_surf: f32,
    /// Soil active SOM C (`som1c[1]`).
    pub som1c_soil: f32,
    /// Slow SOM C (`som2c`).
    pub som2c: f32,
    /// Passive SOM C (`som3c`).
    pub som3c: f32,
    /// Total soil organic carbon (`som1c[1] + som2c + som3c`).
    pub somsc: f32,
    /// Herbaceous live shoots C (`aglivc`).
    pub aglivc: f32,
    /// Herbaceous live roots C (`bglivc`).
    pub bglivc: f32,
    /// Standing dead C (`stdedc`).
    pub stdedc: f32,
    /// Forest live leaf C (`rleavc`).
    pub rleavc: f32,
    /// Forest live large wood C (`rlwodc`).
    pub rlwodc: f32,
    /// Surface mineral N (`minerl[0][0]`).
    pub minerl_n0: f32,
    /// Top-layer soil water (`asmos[0]`).
    pub asmos0: f32,
    /// Top-layer field capacity (`afiel[0]`).
    pub afiel0: f32,
    /// Top-layer wilting point (`awilt[0]`).
    pub awilt0: f32,
}

impl SiteNcPools {
    /// Decode key variables from a 235-float `site.nc` row.
    pub fn from_row(row: &[f32]) -> Result<Self> {
        if row.len() < LEGACY_SITE_NC_COLS {
            return Err(IoError::NetCdf(format!(
                "expected >= {LEGACY_SITE_NC_COLS} cols in site.nc row, got {}",
                row.len()
            )));
        }
        let som1c_surf = row[39] + row[40];
        let som1c_soil = row[41] + row[42];
        let som2c = row[43] + row[44];
        let som3c = row[45] + row[46];
        Ok(Self {
            lat: row[0],
            lon: row[1],
            som1c_surf,
            som1c_soil,
            som2c,
            som3c,
            somsc: som1c_soil + som2c + som3c,
            aglivc: row[71] + row[72],
            bglivc: row[76] + row[77],
            stdedc: row[81] + row[82],
            rleavc: row[86] + row[87],
            rlwodc: row[96] + row[97],
            minerl_n0: row[126],
            asmos0: row[179],
            afiel0: row[19],
            awilt0: row[9],
        })
    }

    /// Encode a [`CellState`] and [`CellDerivedParams`] into a deterministic
    /// 235-float `site.nc` row (zero-initialized, never emitting uninitialized stack memory).
    pub fn encode_row<F: Float>(
        state: &CellState<F>,
        derived: &CellDerivedParams<F>,
    ) -> [f32; LEGACY_SITE_NC_COLS] {
        let mut r = [0.0f32; LEGACY_SITE_NC_COLS];
        r[0] = derived.lat.to_f32();
        r[1] = derived.lon.to_f32();
        r[2] = derived.sand.to_f32();
        r[3] = derived.silt.to_f32();
        r[4] = derived.clay.to_f32();
        r[5] = derived.bulkd.to_f32();
        r[6] = derived.drain.to_f32();
        r[7] = derived.basef.to_f32();
        r[8] = derived.stormf.to_f32();
        for l in 0..10 {
            r[9 + l] = derived.awilt[l].to_f32();
            r[19 + l] = derived.afiel[l].to_f32();
            r[179 + l] = state.asmos[l].to_f32();
        }
        r[29] = derived.ph.to_f32();
        r[39] = state.som1c[0].unlab.to_f32();
        r[40] = state.som1c[0].lab.to_f32();
        r[41] = state.som1c[1].unlab.to_f32();
        r[42] = state.som1c[1].lab.to_f32();
        r[43] = state.som2c.unlab.to_f32();
        r[44] = state.som2c.lab.to_f32();
        r[45] = state.som3c.unlab.to_f32();
        r[46] = state.som3c.lab.to_f32();
        for e in 0..3 {
            r[47 + e] = state.som1e[0][e].to_f32();
            r[50 + e] = state.som1e[1][e].to_f32();
            r[53 + e] = state.som2e[e].to_f32();
            r[56 + e] = state.som3e[e].to_f32();
        }
        r[63] = state.strucc[0].unlab.to_f32();
        r[64] = state.strucc[0].lab.to_f32();
        r[65] = state.strucc[1].unlab.to_f32();
        r[66] = state.strucc[1].lab.to_f32();
        r[67] = state.metabc[0].unlab.to_f32();
        r[68] = state.metabc[0].lab.to_f32();
        r[69] = state.metabc[1].unlab.to_f32();
        r[70] = state.metabc[1].lab.to_f32();

        r[71] = state.aglivc.unlab.to_f32();
        r[72] = state.aglivc.lab.to_f32();
        r[76] = state.bglivc.unlab.to_f32();
        r[77] = state.bglivc.lab.to_f32();
        r[81] = state.stdedc.unlab.to_f32();
        r[82] = state.stdedc.lab.to_f32();
        r[86] = state.rleavc.unlab.to_f32();
        r[87] = state.rleavc.lab.to_f32();
        r[91] = state.fbrchc.unlab.to_f32();
        r[92] = state.fbrchc.lab.to_f32();
        r[96] = state.rlwodc.unlab.to_f32();
        r[97] = state.rlwodc.lab.to_f32();
        r[101] = state.frootc.unlab.to_f32();
        r[102] = state.frootc.lab.to_f32();
        r[106] = state.crootc.unlab.to_f32();
        r[107] = state.crootc.lab.to_f32();
        r[111] = state.wood1c.unlab.to_f32();
        r[112] = state.wood1c.lab.to_f32();
        r[113] = state.wood2c.unlab.to_f32();
        r[114] = state.wood2c.lab.to_f32();
        r[115] = state.wood3c.unlab.to_f32();
        r[116] = state.wood3c.lab.to_f32();
        for e in 0..3 {
            r[73 + e] = state.aglive[e].to_f32();
            r[78 + e] = state.bglive[e].to_f32();
            r[83 + e] = state.stdede[e].to_f32();
            r[88 + e] = state.rleave[e].to_f32();
            r[93 + e] = state.fbrche[e].to_f32();
            r[98 + e] = state.rlwode[e].to_f32();
            r[103 + e] = state.froote[e].to_f32();
            r[108 + e] = state.croote[e].to_f32();
            for l in 0..10 {
                r[126 + e * 10 + l] = state.minerl[l][e].to_f32();
            }
        }
        r[169] = state.occlud.to_f32();
        r[189] = state.avh2o[0].to_f32();
        r[190] = state.avh2o[1].to_f32();
        r[191] = state.avh2o[2].to_f32();
        r
    }
}

/// Parsed legacy `site.nc` file contents: `(cell_pools, mask, (nrows, ncols))`.
pub type LegacySiteNcData = (Vec<SiteNcPools>, Vec<i32>, (usize, usize));

/// Read all cells' [`SiteNcPools`] and spatial `mask` from a `site.nc` NetCDF-3 file.
pub fn read_site_nc<P: AsRef<Path>>(path: P) -> Result<LegacySiteNcData> {
    let nc = NcFile::open(path)?;
    let flt = nc.read_f32("fltvar")?;
    let shape = nc.var_shape("fltvar")?;
    if shape.len() != 2 || shape[1] < LEGACY_SITE_NC_COLS {
        return Err(IoError::NetCdf(format!(
            "unexpected fltvar shape: {shape:?}"
        )));
    }
    let ncells = shape[0];
    let ncols_v = shape[1];
    let mut pools = Vec::with_capacity(ncells);
    for i in 0..ncells {
        let row = &flt[i * ncols_v..(i + 1) * ncols_v];
        pools.push(SiteNcPools::from_row(row)?);
    }
    let mask = nc.read_i32("mask")?;
    let mshape = nc.var_shape("mask")?;
    Ok((pools, mask, (mshape[0], mshape[1])))
}

/// Write a grid simulation's final states to a `site.nc`-compatible pure-Rust NetCDF-3 file.
pub fn write_site_nc<F: Float, P: AsRef<Path>>(
    path: P,
    nrows: usize,
    ncols: usize,
    mask: &[i32],
    cells: &[(CellState<F>, CellDerivedParams<F>)],
    nelem: usize,
    history: &str,
) -> Result<()> {
    let ncells = cells.len();
    let mut fltvar = Vec::with_capacity(ncells * LEGACY_SITE_NC_COLS);
    let mut intvar = Vec::with_capacity(ncells * 3);
    for (st, der) in cells {
        let row = SiteNcPools::encode_row(st, der);
        fltvar.extend_from_slice(&row);
        intvar.push(nelem as i32);
        intvar.push(der.nlayer as i32);
        intvar.push(der.nlaypg as i32);
    }

    let mut w = NcWriter::new();
    let d_cells = w.add_dim("ncells", ncells);
    let d_nouts = w.add_dim("nouts", LEGACY_SITE_NC_COLS);
    let d_iouts = w.add_dim("iouts", 3);
    let d_rows = w.add_dim("nrows", nrows);
    let d_cols = w.add_dim("ncols", ncols);
    w.add_global_attr("history", history);
    w.add_var_i32("mask", vec![d_rows, d_cols], mask);
    w.add_var_f32("fltvar", vec![d_cells, d_nouts], &fltvar);
    w.add_var_i32("intvar", vec![d_cells, d_iouts], &intvar);
    w.write_to_file(path)
}
