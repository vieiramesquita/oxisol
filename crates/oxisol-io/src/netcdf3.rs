//! Pure-Rust NetCDF-3 Classic (`CDF\x01`) and 64-bit Offset (`CDF\x02`) reader & writer.
//!
//! Zero C dependencies (`libnetcdf.so` not required), enabling portable native execution
//! across Linux (modern high-performance servers), WSL2, Windows, and macOS.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{IoError, Result};

const NC_DIMENSION: u32 = 10;
const NC_VARIABLE: u32 = 11;
const NC_ATTRIBUTE: u32 = 12;

/// NetCDF-3 scalar data type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NcType {
    /// 8-bit signed integer (`NC_BYTE = 1`).
    Byte = 1,
    /// 8-bit character (`NC_CHAR = 2`).
    Char = 2,
    /// 16-bit signed big-endian integer (`NC_SHORT = 3`).
    Short = 3,
    /// 32-bit signed big-endian integer (`NC_INT = 4`).
    Int = 4,
    /// 32-bit IEEE-754 big-endian float (`NC_FLOAT = 5`).
    Float = 5,
    /// 64-bit IEEE-754 big-endian float (`NC_DOUBLE = 6`).
    Double = 6,
}

impl NcType {
    fn from_u32(v: u32) -> Result<Self> {
        match v {
            1 => Ok(Self::Byte),
            2 => Ok(Self::Char),
            3 => Ok(Self::Short),
            4 => Ok(Self::Int),
            5 => Ok(Self::Float),
            6 => Ok(Self::Double),
            _ => Err(IoError::NetCdf(format!(
                "unsupported NetCDF-3 type tag: {v}"
            ))),
        }
    }

    fn byte_size(self) -> usize {
        match self {
            Self::Byte | Self::Char => 1,
            Self::Short => 2,
            Self::Int | Self::Float => 4,
            Self::Double => 8,
        }
    }
}

/// Decoded NetCDF-3 variable metadata and raw data access.
#[derive(Debug, Clone)]
pub struct NcVar {
    /// Variable name.
    pub name: String,
    /// Dimension indices in `NcFile::dims`.
    pub dim_ids: Vec<usize>,
    /// Data type.
    pub nc_type: NcType,
    /// Padded byte size of one record (for record vars) or entire variable (for fixed vars).
    pub vsize: usize,
    /// Byte offset from start of file.
    pub begin: u64,
    /// True if the leading dimension is the unlimited record dimension.
    pub is_record: bool,
}

/// In-memory NetCDF-3 dataset handle.
#[derive(Debug, Clone)]
pub struct NcFile {
    /// Dimension `(name, length)` table (unlimited dimension stores `numrecs`).
    pub dims: Vec<(String, usize)>,
    /// Index of unlimited dimension, if present.
    pub unlimited_dim: Option<usize>,
    /// Number of records in the unlimited dimension.
    pub numrecs: usize,
    /// Byte stride of one full record across all record variables.
    pub rec_size: usize,
    /// Variable lookup table.
    pub vars: HashMap<String, NcVar>,
    /// Raw file bytes (fast zero-copy decoding for datasets up to multi-GB on high-memory servers).
    bytes: Vec<u8>,
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn read_u32(&mut self) -> Result<u32> {
        if self.pos + 4 > self.buf.len() {
            return Err(IoError::NetCdf("unexpected EOF reading u32".into()));
        }
        let v = u32::from_be_bytes([
            self.buf[self.pos],
            self.buf[self.pos + 1],
            self.buf[self.pos + 2],
            self.buf[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    fn read_u64(&mut self) -> Result<u64> {
        if self.pos + 8 > self.buf.len() {
            return Err(IoError::NetCdf("unexpected EOF reading u64".into()));
        }
        let v = u64::from_be_bytes([
            self.buf[self.pos],
            self.buf[self.pos + 1],
            self.buf[self.pos + 2],
            self.buf[self.pos + 3],
            self.buf[self.pos + 4],
            self.buf[self.pos + 5],
            self.buf[self.pos + 6],
            self.buf[self.pos + 7],
        ]);
        self.pos += 8;
        Ok(v)
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_u32()? as usize;
        let padded = (len + 3) & !3;
        if self.pos + padded > self.buf.len() {
            return Err(IoError::NetCdf("unexpected EOF reading string".into()));
        }
        let s = String::from_utf8_lossy(&self.buf[self.pos..self.pos + len]).into_owned();
        self.pos += padded;
        Ok(s)
    }

    fn skip_attrs(&mut self) -> Result<()> {
        let tag = self.read_u32()?;
        let count = self.read_u32()?;
        if tag == 0 && count == 0 {
            return Ok(());
        }
        if tag != NC_ATTRIBUTE {
            return Err(IoError::NetCdf(format!("expected NC_ATTRIBUTE, got {tag}")));
        }
        for _ in 0..count {
            let _name = self.read_string()?;
            let nc_type = NcType::from_u32(self.read_u32()?)?;
            let nelems = self.read_u32()? as usize;
            let raw_bytes = nelems * nc_type.byte_size();
            let padded = (raw_bytes + 3) & !3;
            if self.pos + padded > self.buf.len() {
                return Err(IoError::NetCdf("unexpected EOF skipping attribute".into()));
            }
            self.pos += padded;
        }
        Ok(())
    }
}

impl NcFile {
    /// Open and parse a NetCDF-3 classic (`CDF\x01`) or 64-bit offset (`CDF\x02`) file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut f = File::open(path)?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        Self::from_bytes(bytes)
    }

    /// Parse a NetCDF-3 buffer from memory.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < 8 || &bytes[0..3] != b"CDF" {
            return Err(IoError::NetCdf("invalid NetCDF-3 magic header".into()));
        }
        let version = bytes[3];
        if version != 1 && version != 2 {
            return Err(IoError::NetCdf(format!(
                "unsupported NetCDF version byte: {version}"
            )));
        }
        let mut cur = Cursor {
            buf: &bytes,
            pos: 4,
        };
        let numrecs = cur.read_u32()? as usize;

        // Dimensions
        let mut dims = Vec::new();
        let mut unlimited_dim = None;
        let dim_tag = cur.read_u32()?;
        let dim_count = cur.read_u32()?;
        if dim_tag == NC_DIMENSION {
            for idx in 0..dim_count as usize {
                let name = cur.read_string()?;
                let len = cur.read_u32()? as usize;
                if len == 0 {
                    unlimited_dim = Some(idx);
                    dims.push((name, numrecs));
                } else {
                    dims.push((name, len));
                }
            }
        } else if !(dim_tag == 0 && dim_count == 0) {
            return Err(IoError::NetCdf(format!("unexpected dim tag: {dim_tag}")));
        }

        // Global attributes
        cur.skip_attrs()?;

        // Variables
        let mut vars = HashMap::new();
        let mut rec_size = 0usize;
        let var_tag = cur.read_u32()?;
        let var_count = cur.read_u32()?;
        if var_tag == NC_VARIABLE {
            for _ in 0..var_count {
                let name = cur.read_string()?;
                let ndims = cur.read_u32()? as usize;
                let mut dim_ids = Vec::with_capacity(ndims);
                for _ in 0..ndims {
                    dim_ids.push(cur.read_u32()? as usize);
                }
                cur.skip_attrs()?;
                let nc_type = NcType::from_u32(cur.read_u32()?)?;
                let vsize = cur.read_u32()? as usize;
                let begin = if version == 1 {
                    cur.read_u32()? as u64
                } else {
                    cur.read_u64()?
                };
                let is_record = !dim_ids.is_empty() && Some(dim_ids[0]) == unlimited_dim;
                if is_record {
                    rec_size += vsize;
                }
                vars.insert(
                    name.clone(),
                    NcVar {
                        name,
                        dim_ids,
                        nc_type,
                        vsize,
                        begin,
                        is_record,
                    },
                );
            }
        } else if !(var_tag == 0 && var_count == 0) {
            return Err(IoError::NetCdf(format!("unexpected var tag: {var_tag}")));
        }

        Ok(Self {
            dims,
            unlimited_dim,
            numrecs,
            rec_size,
            vars,
            bytes,
        })
    }

    /// Return the shape of variable `name`.
    pub fn var_shape(&self, name: &str) -> Result<Vec<usize>> {
        let v = self
            .vars
            .get(name)
            .ok_or_else(|| IoError::NetCdf(format!("variable not found: {name}")))?;
        Ok(v.dim_ids.iter().map(|&id| self.dims[id].1).collect())
    }

    /// Read all values of variable `name` as `Vec<f32>` (casts from `i16`/`i32`/`f32`/`f64`).
    pub fn read_f32(&self, name: &str) -> Result<Vec<f32>> {
        let v = self
            .vars
            .get(name)
            .ok_or_else(|| IoError::NetCdf(format!("variable not found: {name}")))?;
        let shape: Vec<usize> = v.dim_ids.iter().map(|&id| self.dims[id].1).collect();
        let total: usize = shape.iter().product();
        let mut out = Vec::with_capacity(total);
        self.decode_var(v, &shape, |raw, ty| {
            let val = match ty {
                NcType::Byte => raw[0] as i8 as f32,
                NcType::Char => raw[0] as f32,
                NcType::Short => i16::from_be_bytes([raw[0], raw[1]]) as f32,
                NcType::Int => i32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as f32,
                NcType::Float => f32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]),
                NcType::Double => f64::from_be_bytes([
                    raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
                ]) as f32,
            };
            out.push(val);
        })?;
        Ok(out)
    }

    /// Read all values of variable `name` as `Vec<i32>` (casts from `i16`/`i32`).
    pub fn read_i32(&self, name: &str) -> Result<Vec<i32>> {
        let v = self
            .vars
            .get(name)
            .ok_or_else(|| IoError::NetCdf(format!("variable not found: {name}")))?;
        let shape: Vec<usize> = v.dim_ids.iter().map(|&id| self.dims[id].1).collect();
        let total: usize = shape.iter().product();
        let mut out = Vec::with_capacity(total);
        self.decode_var(v, &shape, |raw, ty| {
            let val = match ty {
                NcType::Byte => raw[0] as i8 as i32,
                NcType::Char => raw[0] as i32,
                NcType::Short => i16::from_be_bytes([raw[0], raw[1]]) as i32,
                NcType::Int => i32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]),
                NcType::Float => f32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as i32,
                NcType::Double => f64::from_be_bytes([
                    raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
                ]) as i32,
            };
            out.push(val);
        })?;
        Ok(out)
    }

    fn decode_var<F>(&self, v: &NcVar, shape: &[usize], mut emit: F) -> Result<()>
    where
        F: FnMut(&[u8], NcType),
    {
        let esz = v.nc_type.byte_size();
        if v.is_record {
            let nrec = shape[0];
            let inner: usize = shape[1..].iter().product();
            for r in 0..nrec {
                let base = v.begin as usize + r * self.rec_size;
                let end = base + inner * esz;
                if end > self.bytes.len() {
                    return Err(IoError::NetCdf(format!(
                        "record {r} out of bounds for {}",
                        v.name
                    )));
                }
                for i in 0..inner {
                    let off = base + i * esz;
                    emit(&self.bytes[off..off + esz], v.nc_type);
                }
            }
        } else {
            let total: usize = shape.iter().product();
            let base = v.begin as usize;
            let end = base + total * esz;
            if end > self.bytes.len() {
                return Err(IoError::NetCdf(format!(
                    "fixed var out of bounds for {}",
                    v.name
                )));
            }
            for i in 0..total {
                let off = base + i * esz;
                emit(&self.bytes[off..off + esz], v.nc_type);
            }
        }
        Ok(())
    }
}

/// Simple pure-Rust NetCDF-3 Classic (`CDF\x01`) writer for fixed-size variables.
#[derive(Debug, Default)]
pub struct NcWriter {
    dims: Vec<(String, usize)>,
    global_attrs: Vec<(String, String)>,
    vars: Vec<WriteVar>,
}

#[derive(Debug)]
struct WriteVar {
    name: String,
    dim_ids: Vec<usize>,
    nc_type: NcType,
    raw_be: Vec<u8>,
}

impl NcWriter {
    /// Create a new NetCDF-3 builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a fixed dimension and return its dimension ID.
    pub fn add_dim(&mut self, name: &str, len: usize) -> usize {
        let id = self.dims.len();
        self.dims.push((name.to_string(), len));
        id
    }

    /// Add a global text attribute.
    pub fn add_global_attr(&mut self, name: &str, text: &str) {
        self.global_attrs.push((name.to_string(), text.to_string()));
    }

    /// Add a `f32` variable.
    pub fn add_var_f32(&mut self, name: &str, dim_ids: Vec<usize>, data: &[f32]) {
        let mut raw = Vec::with_capacity(data.len() * 4);
        for &x in data {
            raw.extend_from_slice(&x.to_be_bytes());
        }
        self.vars.push(WriteVar {
            name: name.to_string(),
            dim_ids,
            nc_type: NcType::Float,
            raw_be: raw,
        });
    }

    /// Add an `i32` variable.
    pub fn add_var_i32(&mut self, name: &str, dim_ids: Vec<usize>, data: &[i32]) {
        let mut raw = Vec::with_capacity(data.len() * 4);
        for &x in data {
            raw.extend_from_slice(&x.to_be_bytes());
        }
        self.vars.push(WriteVar {
            name: name.to_string(),
            dim_ids,
            nc_type: NcType::Int,
            raw_be: raw,
        });
    }

    /// Serialize to a NetCDF-3 Classic file on disk.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut out = Vec::new();
        out.extend_from_slice(b"CDF\x01");
        out.extend_from_slice(&0u32.to_be_bytes()); // numrecs = 0

        // Dimensions
        if self.dims.is_empty() {
            out.extend_from_slice(&0u32.to_be_bytes());
            out.extend_from_slice(&0u32.to_be_bytes());
        } else {
            out.extend_from_slice(&NC_DIMENSION.to_be_bytes());
            out.extend_from_slice(&(self.dims.len() as u32).to_be_bytes());
            for (name, len) in &self.dims {
                write_padded_str(&mut out, name);
                out.extend_from_slice(&(*len as u32).to_be_bytes());
            }
        }

        // Global attributes
        if self.global_attrs.is_empty() {
            out.extend_from_slice(&0u32.to_be_bytes());
            out.extend_from_slice(&0u32.to_be_bytes());
        } else {
            out.extend_from_slice(&NC_ATTRIBUTE.to_be_bytes());
            out.extend_from_slice(&(self.global_attrs.len() as u32).to_be_bytes());
            for (k, v) in &self.global_attrs {
                write_padded_str(&mut out, k);
                out.extend_from_slice(&(NcType::Char as u32).to_be_bytes());
                write_padded_bytes(&mut out, v.as_bytes());
            }
        }

        // Variables header with placeholder offsets
        let mut begin_positions = Vec::with_capacity(self.vars.len());
        if self.vars.is_empty() {
            out.extend_from_slice(&0u32.to_be_bytes());
            out.extend_from_slice(&0u32.to_be_bytes());
        } else {
            out.extend_from_slice(&NC_VARIABLE.to_be_bytes());
            out.extend_from_slice(&(self.vars.len() as u32).to_be_bytes());
            for v in &self.vars {
                write_padded_str(&mut out, &v.name);
                out.extend_from_slice(&(v.dim_ids.len() as u32).to_be_bytes());
                for &d in &v.dim_ids {
                    out.extend_from_slice(&(d as u32).to_be_bytes());
                }
                // Empty variable attributes
                out.extend_from_slice(&0u32.to_be_bytes());
                out.extend_from_slice(&0u32.to_be_bytes());
                out.extend_from_slice(&(v.nc_type as u32).to_be_bytes());
                let padded_sz = (v.raw_be.len() + 3) & !3;
                out.extend_from_slice(&(padded_sz as u32).to_be_bytes());
                begin_positions.push(out.len());
                out.extend_from_slice(&0u32.to_be_bytes()); // placeholder
            }
        }

        // Patch offsets and write variable payloads
        for (v, &patch_pos) in self.vars.iter().zip(begin_positions.iter()) {
            let offset = out.len() as u32;
            out[patch_pos..patch_pos + 4].copy_from_slice(&offset.to_be_bytes());
            out.extend_from_slice(&v.raw_be);
            let pad = (4 - (v.raw_be.len() % 4)) % 4;
            out.extend(std::iter::repeat(0u8).take(pad));
        }

        let mut f = File::create(path)?;
        f.write_all(&out)?;
        Ok(())
    }
}

fn write_padded_str(buf: &mut Vec<u8>, s: &str) {
    write_padded_bytes(buf, s.as_bytes());
}

fn write_padded_bytes(buf: &mut Vec<u8>, b: &[u8]) {
    buf.extend_from_slice(&(b.len() as u32).to_be_bytes());
    buf.extend_from_slice(b);
    let pad = (4 - (b.len() % 4)) % 4;
    buf.extend(std::iter::repeat(0u8).take(pad));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_netcdf3_classic() {
        let mut w = NcWriter::new();
        let d0 = w.add_dim("ncells", 2);
        let d1 = w.add_dim("nvars", 3);
        w.add_global_attr("title", "oxisol test");
        w.add_var_f32("vals", vec![d0, d1], &[1.5, 2.5, 3.5, -4.25, 0.0, 99.0]);
        w.add_var_i32("ids", vec![d0], &[10, 20]);

        let tmp = std::env::temp_dir().join("century_rs_nc3_test.nc");
        w.write_to_file(&tmp).unwrap();

        let nc = NcFile::open(&tmp).unwrap();
        assert_eq!(nc.var_shape("vals").unwrap(), vec![2, 3]);
        assert_eq!(
            nc.read_f32("vals").unwrap(),
            vec![1.5, 2.5, 3.5, -4.25, 0.0, 99.0]
        );
        assert_eq!(nc.read_i32("ids").unwrap(), vec![10, 20]);
        let _ = std::fs::remove_file(&tmp);
    }
}
