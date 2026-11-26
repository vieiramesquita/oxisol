//! Tolerant pure-Rust parsers for CENTURY `.100` parameter files (`fix.100`,
//! `<site>.100`, `crop.100`, `tree.100`, `cult.100`, `fert.100`, `fire.100`,
//! `graz.100`, `harv.100`, `irri.100`, `omad.100`, `trem.100`).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use oxisol_core::{
    CropParams, CultParams, FertParams, FireParams, FixParams, Float, GrazParams, HarvParams,
    IrriParams, OmadParams, SiteParams, TreeParams, TremParams,
};

use crate::error::Result;

/// Generic parsed `.100` block: `(option_name, uppercase_key -> f64)`.
#[derive(Debug, Clone)]
pub struct Raw100Block {
    /// Block / option identifier (first token of header line).
    pub name: String,
    /// Map of normalized parameter key (e.g. `"ADEP(1)"`, `"PRDX(1)"`) -> numeric value.
    pub params: HashMap<String, f64>,
}

/// Parse any `.100` file into one or more named parameter blocks.
pub fn parse_100_blocks(content: &str) -> Vec<Raw100Block> {
    let mut blocks: Vec<Raw100Block> = Vec::new();
    let mut current: Option<Raw100Block> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("***") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let first = match parts.next() {
            Some(tok) => tok,
            None => continue,
        };
        if let Ok(val) = first.parse::<f64>() {
            // Parameter line: `<float> <'KEY'>`
            if let Some(key_tok) = parts.next() {
                let key = key_tok
                    .trim_matches(|c| c == '\'' || c == '"')
                    .to_ascii_uppercase();
                if let Some(ref mut blk) = current {
                    blk.params.insert(key, val);
                }
            }
        } else {
            // New option header line (e.g. "TMC3 Temperate C3 grasses")
            if let Some(prev) = current.take() {
                if !prev.params.is_empty() {
                    blocks.push(prev);
                }
            }
            current = Some(Raw100Block {
                name: first.to_string(),
                params: HashMap::new(),
            });
        }
    }
    if let Some(last) = current {
        if !last.params.is_empty() {
            blocks.push(last);
        }
    }
    blocks
}

#[inline]
fn get_f<F: Float>(m: &HashMap<String, f64>, key: &str, default: F) -> F {
    m.get(key).map(|&v| F::from_f64(v)).unwrap_or(default)
}

#[inline]
fn get_i(m: &HashMap<String, f64>, key: &str, default: i32) -> i32 {
    m.get(key).map(|&v| v.round() as i32).unwrap_or(default)
}

/// Parse a `fix.100` file into [`FixParams`].
pub fn parse_fix_file<F: Float, P: AsRef<Path>>(path: P) -> Result<FixParams<F>> {
    let txt = fs::read_to_string(path)?;
    Ok(parse_fix_str(&txt))
}

/// Parse `fix.100` content string.
pub fn parse_fix_str<F: Float>(content: &str) -> FixParams<F> {
    let mut fix = FixParams::<F>::default();
    let blocks = parse_100_blocks(content);
    let Some(blk) = blocks.first() else {
        return fix;
    };
    let m = &blk.params;

    for i in 0..10 {
        fix.adep[i] = get_f(m, &format!("ADEP({})", i + 1), fix.adep[i]);
        fix.awtl[i] = get_f(m, &format!("AWTL({})", i + 1), fix.awtl[i]);
    }
    fix.agppa = get_f(m, "AGPPA", fix.agppa);
    fix.agppb = get_f(m, "AGPPB", fix.agppb);
    for i in 0..3 {
        fix.aneref[i] = get_f(m, &format!("ANEREF({})", i + 1), fix.aneref[i]);
        fix.damrmn[i] = get_f(m, &format!("DAMRMN({})", i + 1), fix.damrmn[i]);
        fix.lhzf[i] = get_f(m, &format!("LHZF({})", i + 1), fix.lhzf[i]);
        fix.omlech[i] = get_f(m, &format!("OMLECH({})", i + 1), fix.omlech[i]);
        fix.pmnsec[i] = get_f(m, &format!("PMNSEC({})", i + 1), fix.pmnsec[i]);
        fix.pparmn[i] = get_f(m, &format!("PPARMN({})", i + 1), fix.pparmn[i]);
        fix.pprpts[i] = get_f(m, &format!("PPRPTS({})", i + 1), fix.pprpts[i]);
        fix.psecmn[i] = get_f(m, &format!("PSECMN({})", i + 1), fix.psecmn[i]);
        fix.rcestr[i] = get_f(m, &format!("RCESTR({})", i + 1), fix.rcestr[i]);
        fix.teff[i] = get_f(m, &format!("TEFF({})", i + 1), fix.teff[i]);
        fix.texesp[i] = get_f(m, &format!("TEXESP({})", i + 1), fix.texesp[i]);
    }
    fix.animpt = get_f(m, "ANIMPT", fix.animpt);
    fix.bgppa = get_f(m, "BGPPA", fix.bgppa);
    fix.bgppb = get_f(m, "BGPPB", fix.bgppb);
    for i in 0..2 {
        fix.co2ppm[i] = get_f(m, &format!("CO2PPM({})", i + 1), fix.co2ppm[i]);
        fix.dec1[i] = get_f(m, &format!("DEC1({})", i + 1), fix.dec1[i]);
        fix.dec2[i] = get_f(m, &format!("DEC2({})", i + 1), fix.dec2[i]);
        fix.dec3[i] = get_f(m, &format!("DEC3({})", i + 1), fix.dec3[i]);
        fix.p1co2a[i] = get_f(m, &format!("P1CO2A({})", i + 1), fix.p1co2a[i]);
        fix.p1co2b[i] = get_f(m, &format!("P1CO2B({})", i + 1), fix.p1co2b[i]);
        fix.pligst[i] = get_f(m, &format!("PLIGST({})", i + 1), fix.pligst[i]);
        fix.pmco2[i] = get_f(m, &format!("PMCO2({})", i + 1), fix.pmco2[i]);
        fix.ps1co2[i] = get_f(m, &format!("PS1CO2({})", i + 1), fix.ps1co2[i]);
        fix.ps1s3[i] = get_f(m, &format!("PS1S3({})", i + 1), fix.ps1s3[i]);
        fix.ps2s3[i] = get_f(m, &format!("PS2S3({})", i + 1), fix.ps2s3[i]);
        fix.spl[i] = get_f(m, &format!("SPL({})", i + 1), fix.spl[i]);
        fix.strmax[i] = get_f(m, &format!("STRMAX({})", i + 1), fix.strmax[i]);
        fix.tmelt[i] = get_f(m, &format!("TMELT({})", i + 1), fix.tmelt[i]);
    }
    fix.co2rmp = get_f(m, "CO2RMP", fix.co2rmp);
    for r in 0..2 {
        for c in 0..3 {
            fix.damr[r][c] = get_f(m, &format!("DAMR({},{})", r + 1, c + 1), fix.damr[r][c]);
        }
    }
    fix.dec4 = get_f(m, "DEC4", fix.dec4);
    fix.dec5 = get_f(m, "DEC5", fix.dec5);
    fix.deck5 = get_f(m, "DECK5", fix.deck5);
    fix.dligdf = get_f(m, "DLIGDF", fix.dligdf);
    fix.dresp = get_f(m, "DRESP", fix.dresp);
    fix.edepth = get_f(m, "EDEPTH", fix.edepth);
    fix.elitst = get_f(m, "ELITST", fix.elitst);
    fix.enrich = get_f(m, "ENRICH", fix.enrich);
    for i in 0..6 {
        fix.favail[i] = get_f(m, &format!("FAVAIL({})", i + 1), fix.favail[i]);
    }
    for i in 0..5 {
        fix.fleach[i] = get_f(m, &format!("FLEACH({})", i + 1), fix.fleach[i]);
        fix.texepp[i] = get_f(m, &format!("TEXEPP({})", i + 1), fix.texepp[i]);
    }
    for i in 0..4 {
        fix.fwloss[i] = get_f(m, &format!("FWLOSS({})", i + 1), fix.fwloss[i]);
        fix.phesp[i] = get_f(m, &format!("PHESP({})", i + 1), fix.phesp[i]);
    }
    fix.fxmca = get_f(m, "FXMCA", fix.fxmca);
    fix.fxmcb = get_f(m, "FXMCB", fix.fxmcb);
    fix.fxmxs = get_f(m, "FXMXS", fix.fxmxs);
    fix.fxnpb = get_f(m, "FXNPB", fix.fxnpb);
    fix.gremb = get_f(m, "GREMB", fix.gremb);
    fix.idef = get_i(m, "IDEF", fix.idef);
    fix.minlch = get_f(m, "MINLCH", fix.minlch);
    fix.nsnfix = get_i(m, "NSNFIX", fix.nsnfix);
    fix.ntspm = get_i(m, "NTSPM", fix.ntspm);
    fix.p2co2 = get_f(m, "P2CO2", fix.p2co2);
    fix.p3co2 = get_f(m, "P3CO2", fix.p3co2);
    fix.pabres = get_f(m, "PABRES", fix.pabres);
    fix.peftxa = get_f(m, "PEFTXA", fix.peftxa);
    fix.peftxb = get_f(m, "PEFTXB", fix.peftxb);
    fix.pmntmp = get_f(m, "PMNTMP", fix.pmntmp);
    fix.pmxbio = get_f(m, "PMXBIO", fix.pmxbio);
    fix.pmxtmp = get_f(m, "PMXTMP", fix.pmxtmp);
    fix.psecoc = get_f(m, "PSECOC", fix.psecoc);
    fix.rictrl = get_f(m, "RICTRL", fix.rictrl);
    fix.riint = get_f(m, "RIINT", fix.riint);
    fix.rsplig = get_f(m, "RSPLIG", fix.rsplig);
    fix.vlosse = get_f(m, "VLOSSE", fix.vlosse);
    fix.vlossg = get_f(m, "VLOSSG", fix.vlossg);

    for e in 0..3 {
        for j in 0..3 {
            fix.pcemic[e][j] = get_f(m, &format!("PCEMIC({},{})", j + 1, e + 1), fix.pcemic[e][j]);
            fix.rad1p[e][j] = get_f(m, &format!("RAD1P({},{})", j + 1, e + 1), fix.rad1p[e][j]);
            fix.varat1[e][j] = get_f(m, &format!("VARAT1({},{})", j + 1, e + 1), fix.varat1[e][j]);
            fix.varat2[e][j] = get_f(m, &format!("VARAT2({},{})", j + 1, e + 1), fix.varat2[e][j]);
            fix.varat3[e][j] = get_f(m, &format!("VARAT3({},{})", j + 1, e + 1), fix.varat3[e][j]);
        }
    }

    fix
}

/// Parse a `<site>.100` (e.g. `lu3.100`) file into [`SiteParams`].
pub fn parse_site_file<F: Float, P: AsRef<Path>>(path: P) -> Result<SiteParams<F>> {
    let txt = fs::read_to_string(path)?;
    Ok(parse_site_str(&txt))
}

/// Parse `<site>.100` content string.
pub fn parse_site_str<F: Float>(content: &str) -> SiteParams<F> {
    let mut s = SiteParams::<F>::default();
    let blocks = parse_100_blocks(content);
    let Some(blk) = blocks.first() else {
        return s;
    };
    let m = &blk.params;

    for i in 0..12 {
        s.precip[i] = get_f(m, &format!("PRECIP({})", i + 1), s.precip[i]);
        s.tmn2m[i] = get_f(m, &format!("TMN2M({})", i + 1), s.tmn2m[i]);
        s.tmx2m[i] = get_f(m, &format!("TMX2M({})", i + 1), s.tmx2m[i]);
    }
    s.ivauto = get_i(m, "IVAUTO", s.ivauto);
    s.nelem = get_i(m, "NELEM", s.nelem as i32).clamp(1, 3) as usize;
    s.sitlat = get_f(m, "SITLAT", s.sitlat);
    s.sitlng = get_f(m, "SITLNG", s.sitlng);
    s.sand = get_f(m, "SAND", s.sand);
    s.silt = get_f(m, "SILT", s.silt);
    s.clay = get_f(m, "CLAY", s.clay);
    s.bulkd = get_f(m, "BULKD", s.bulkd);
    s.nlayer = get_i(m, "NLAYER", s.nlayer as i32).clamp(1, 10) as usize;
    s.nlaypg = get_i(m, "NLAYPG", s.nlaypg as i32).clamp(1, s.nlayer as i32) as usize;
    s.drain = get_f(m, "DRAIN", s.drain);
    s.basef = get_f(m, "BASEF", s.basef);
    s.stormf = get_f(m, "STORMF", s.stormf);
    s.swflag = get_i(m, "SWFLAG", s.swflag);
    for i in 0..10 {
        s.awilt[i] = get_f(m, &format!("AWILT({})", i + 1), s.awilt[i]);
        s.afiel[i] = get_f(m, &format!("AFIEL({})", i + 1), s.afiel[i]);
        s.rwcf[i] = get_f(m, &format!("RWCF({})", i + 1), s.rwcf[i]);
        for e in 0..3 {
            s.minerl[i][e] = get_f(m, &format!("MINERL({},{})", i + 1, e + 1), s.minerl[i][e]);
        }
    }
    s.ph = get_f(m, "PH", s.ph);
    s.pslsrb = get_f(m, "PSLSRB", s.pslsrb);
    s.sorpmx = get_f(m, "SORPMX", s.sorpmx);
    for i in 0..2 {
        s.epnfa[i] = get_f(m, &format!("EPNFA({})", i + 1), s.epnfa[i]);
        s.epnfs[i] = get_f(m, &format!("EPNFS({})", i + 1), s.epnfs[i]);
        s.satmos[i] = get_f(m, &format!("SATMOS({})", i + 1), s.satmos[i]);
        s.som2ci[i] = get_f(m, &format!("SOM2CI({})", i + 1), s.som2ci[i]);
        s.som3ci[i] = get_f(m, &format!("SOM3CI({})", i + 1), s.som3ci[i]);
        s.aglcis[i] = get_f(m, &format!("AGLCIS({})", i + 1), s.aglcis[i]);
        s.bglcis[i] = get_f(m, &format!("BGLCIS({})", i + 1), s.bglcis[i]);
        s.stdcis[i] = get_f(m, &format!("STDCIS({})", i + 1), s.stdcis[i]);
        s.rlvcis[i] = get_f(m, &format!("RLVCIS({})", i + 1), s.rlvcis[i]);
        s.fbrcis[i] = get_f(m, &format!("FBRCIS({})", i + 1), s.fbrcis[i]);
        s.rlwcis[i] = get_f(m, &format!("RLWCIS({})", i + 1), s.rlwcis[i]);
        s.frtcis[i] = get_f(m, &format!("FRTCIS({})", i + 1), s.frtcis[i]);
        s.crtcis[i] = get_f(m, &format!("CRTCIS({})", i + 1), s.crtcis[i]);
        s.wd1cis[i] = get_f(m, &format!("WD1CIS({})", i + 1), s.wd1cis[i]);
        s.wd2cis[i] = get_f(m, &format!("WD2CIS({})", i + 1), s.wd2cis[i]);
        s.wd3cis[i] = get_f(m, &format!("WD3CIS({})", i + 1), s.wd3cis[i]);
        for j in 0..2 {
            s.som1ci[i][j] = get_f(m, &format!("SOM1CI({},{})", i + 1, j + 1), s.som1ci[i][j]);
            s.clittr[i][j] = get_f(m, &format!("CLITTR({},{})", i + 1, j + 1), s.clittr[i][j]);
        }
        for e in 0..3 {
            s.rces1[i][e] = get_f(m, &format!("RCES1({},{})", i + 1, e + 1), s.rces1[i][e]);
            s.rcelit[i][e] = get_f(m, &format!("RCELIT({},{})", i + 1, e + 1), s.rcelit[i][e]);
        }
    }
    for e in 0..3 {
        s.rces2[e] = get_f(m, &format!("RCES2({})", e + 1), s.rces2[e]);
        s.rces3[e] = get_f(m, &format!("RCES3({})", e + 1), s.rces3[e]);
        s.aglive[e] = get_f(m, &format!("AGLIVE({})", e + 1), s.aglive[e]);
        s.bglive[e] = get_f(m, &format!("BGLIVE({})", e + 1), s.bglive[e]);
        s.stdede[e] = get_f(m, &format!("STDEDE({})", e + 1), s.stdede[e]);
        s.rleave[e] = get_f(m, &format!("RLEAVE({})", e + 1), s.rleave[e]);
        s.fbrche[e] = get_f(m, &format!("FBRCHE({})", e + 1), s.fbrche[e]);
        s.rlwode[e] = get_f(m, &format!("RLWODE({})", e + 1), s.rlwode[e]);
        s.froote[e] = get_f(m, &format!("FROOTE({})", e + 1), s.froote[e]);
        s.croote[e] = get_f(m, &format!("CROOTE({})", e + 1), s.croote[e]);
        s.parent[e] = get_f(m, &format!("PARENT({})", e + 1), s.parent[e]);
        s.secndy[e] = get_f(m, &format!("SECNDY({})", e + 1), s.secndy[e]);
    }
    s.sirri = get_f(m, "SIRRI", s.sirri);
    s.w1lig = get_f(m, "W1LIG", s.w1lig);
    s.w2lig = get_f(m, "W2LIG", s.w2lig);
    s.w3lig = get_f(m, "W3LIG", s.w3lig);
    s.occlud = get_f(m, "OCCLUD", s.occlud);
    s.snlq = get_f(m, "SNLQ", s.snlq);
    s.snow = get_f(m, "SNOW", s.snow);
    s
}

/// Parse `crop.100` into a lookup map of [`CropParams`].
pub fn parse_crop_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, CropParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        let mut c = CropParams::<F> {
            name: blk.name.clone(),
            ..CropParams::default()
        };
        c.prdx1 = get_f(m, "PRDX(1)", c.prdx1);
        for i in 0..4 {
            c.ppdf[i] = get_f(m, &format!("PPDF({})", i + 1), c.ppdf[i]);
            c.fsdeth[i] = get_f(m, &format!("FSDETH({})", i + 1), c.fsdeth[i]);
        }
        c.bioflg = get_f(m, "BIOFLG", c.bioflg);
        c.biok5 = get_f(m, "BIOK5", c.biok5);
        c.pltmrf = get_f(m, "PLTMRF", c.pltmrf);
        c.fulcan = get_f(m, "FULCAN", c.fulcan);
        for i in 0..3 {
            c.frtc[i] = get_f(m, &format!("FRTC({})", i + 1), c.frtc[i]);
            c.efrgrn[i] = get_f(m, &format!("EFRGRN({})", i + 1), c.efrgrn[i]);
            c.crprtf[i] = get_f(m, &format!("CRPRTF({})", i + 1), c.crprtf[i]);
        }
        c.biomax = get_f(m, "BIOMAX", c.biomax);
        for e in 0..3 {
            for j in 0..2 {
                c.pramn[e][j] = get_f(m, &format!("PRAMN({},{})", e + 1, j + 1), c.pramn[e][j]);
                c.pramx[e][j] = get_f(m, &format!("PRAMX({},{})", e + 1, j + 1), c.pramx[e][j]);
                c.prbmn[e][j] = get_f(m, &format!("PRBMN({},{})", e + 1, j + 1), c.prbmn[e][j]);
                c.prbmx[e][j] = get_f(m, &format!("PRBMX({},{})", e + 1, j + 1), c.prbmx[e][j]);
                c.co2ice[j][e] =
                    get_f(m, &format!("CO2ICE(1,{},{})", j + 1, e + 1), c.co2ice[j][e]);
            }
        }
        for i in 0..2 {
            for j in 0..2 {
                c.fligni[i][j] = get_f(m, &format!("FLIGNI({},{})", i + 1, j + 1), c.fligni[i][j]);
            }
        }
        c.himax = get_f(m, "HIMAX", c.himax);
        c.hiwsf = get_f(m, "HIWSF", c.hiwsf);
        c.himon[0] = get_i(m, "HIMON(1)", c.himon[0]);
        c.himon[1] = get_i(m, "HIMON(2)", c.himon[1]);
        c.vlossp = get_f(m, "VLOSSP", c.vlossp);
        c.fallrt = get_f(m, "FALLRT", c.fallrt);
        c.rdr = get_f(m, "RDR", c.rdr);
        c.rtdtmp = get_f(m, "RTDTMP", c.rtdtmp);
        c.snfxmx = get_f(m, "SNFXMX(1)", c.snfxmx);
        c.del13c = get_f(m, "DEL13C", c.del13c);
        c.co2ipr = get_f(m, "CO2IPR(1)", get_f(m, "CO2IPR", c.co2ipr));
        c.co2itr = get_f(m, "CO2ITR(1)", get_f(m, "CO2ITR", c.co2itr));
        c.co2irs = get_f(m, "CO2IRS(1)", get_f(m, "CO2IRS", c.co2irs));
        map.insert(blk.name, c);
    }
    Ok(map)
}

/// Parse `tree.100` into a lookup map of [`TreeParams`].
pub fn parse_tree_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, TreeParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        let mut t = TreeParams::<F> {
            name: blk.name.clone(),
            ..TreeParams::default()
        };
        t.decid = get_i(m, "DECID", t.decid);
        t.prdx3 = get_f(m, "PRDX(3)", t.prdx3);
        t.prdx4 = get_f(m, "PRDX(4)", t.prdx4);
        for i in 0..4 {
            t.ppdf[i] = get_f(m, &format!("PPDF({})", i + 1), t.ppdf[i]);
        }
        for b in 0..3 {
            for p in 0..5 {
                for e in 0..3 {
                    t.cerfor[b][p][e] = get_f(
                        m,
                        &format!("CERFOR({},{},{})", b + 1, p + 1, e + 1),
                        t.cerfor[b][p][e],
                    );
                }
            }
        }
        t.decw[0] = get_f(m, "DECW1", t.decw[0]);
        t.decw[1] = get_f(m, "DECW2", t.decw[1]);
        t.decw[2] = get_f(m, "DECW3", t.decw[2]);
        for p in 0..5 {
            t.fcfrac[0][p] = get_f(m, &format!("FCFRAC({},1)", p + 1), t.fcfrac[0][p]);
            t.fcfrac[1][p] = get_f(m, &format!("FCFRAC({},2)", p + 1), t.fcfrac[1][p]);
            t.wdlig[p] = get_f(m, &format!("WDLIG({})", p + 1), t.wdlig[p]);
            t.wooddr[p] = get_f(m, &format!("WOODDR({})", p + 1), t.wooddr[p]);
        }
        for mo in 0..12 {
            t.leafdr[mo] = get_f(m, &format!("LEAFDR({})", mo + 1), t.leafdr[mo]);
        }
        t.btolai = get_f(m, "BTOLAI", t.btolai);
        t.klai = get_f(m, "KLAI", t.klai);
        t.laitop = get_f(m, "LAITOP", t.laitop);
        t.maxlai = get_f(m, "MAXLAI", t.maxlai);
        t.maxldr = get_f(m, "MAXLDR", t.maxldr);
        for e in 0..3 {
            t.forrtf[e] = get_f(m, &format!("FORRTF({})", e + 1), t.forrtf[e]);
            for j in 0..2 {
                let k2 = format!("CO2ICE(2,{},{})", j + 1, e + 1);
                let k1 = format!("CO2ICE(1,{},{})", j + 1, e + 1);
                t.co2ice[j][e] = get_f(m, &k2, get_f(m, &k1, t.co2ice[j][e]));
            }
        }
        t.sapk = get_f(m, "SAPK", t.sapk);
        t.swold = get_f(m, "SWOLD", t.swold);
        t.snfxmx = get_f(m, "SNFXMX(2)", t.snfxmx);
        t.del13c = get_f(m, "DEL13C", t.del13c);
        t.co2ipr = get_f(m, "CO2IPR(2)", get_f(m, "CO2IPR", t.co2ipr));
        t.co2itr = get_f(m, "CO2ITR(2)", get_f(m, "CO2ITR", t.co2itr));
        t.co2irs = get_f(m, "CO2IRS(2)", get_f(m, "CO2IRS", t.co2irs));
        t.basfc2 = get_f(m, "BASFC2", t.basfc2);
        t.basfct = get_f(m, "BASFCT", t.basfct);
        t.sitpot = get_f(m, "SITPOT", t.sitpot);
        map.insert(blk.name, t);
    }
    Ok(map)
}

/// Parse `cult.100`.
pub fn parse_cult_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, CultParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        let cultra = core::array::from_fn(|i| get_f(m, &format!("CULTRA({})", i + 1), F::ZERO));
        let clteff = core::array::from_fn(|i| get_f(m, &format!("CLTEFF({})", i + 1), F::ONE));
        map.insert(
            blk.name.clone(),
            CultParams {
                name: blk.name,
                cultra,
                clteff,
            },
        );
    }
    Ok(map)
}

/// Parse `fert.100`.
pub fn parse_fert_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, FertParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        let feramt = core::array::from_fn(|i| get_f(m, &format!("FERAMT({})", i + 1), F::ZERO));
        let aufert = get_f(m, "AUFERT", F::ZERO);
        map.insert(
            blk.name.clone(),
            FertParams {
                name: blk.name,
                feramt,
                aufert,
            },
        );
    }
    Ok(map)
}

/// Parse `fire.100`.
pub fn parse_fire_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, FireParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        map.insert(
            blk.name.clone(),
            FireParams {
                name: blk.name,
                flfrem: get_f(m, "FLFREM", F::ZERO),
                fdfrem: [
                    get_f(m, "FDFREM(1)", F::ZERO),
                    get_f(m, "FDFREM(2)", F::ZERO),
                ],
                fret: core::array::from_fn(|i| get_f(m, &format!("FRET({})", i + 1), F::ZERO)),
                frtsh: get_f(m, "FRTSH", F::ZERO),
                fnue: [get_f(m, "FNUE(1)", F::ONE), get_f(m, "FNUE(2)", F::ONE)],
            },
        );
    }
    Ok(map)
}

/// Parse `graz.100`.
pub fn parse_graz_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, GrazParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        map.insert(
            blk.name.clone(),
            GrazParams {
                name: blk.name,
                flgrem: get_f(m, "FLGREM", F::ZERO),
                fdgrem: get_f(m, "FDGREM", F::ZERO),
                gfcret: get_f(m, "GFCRET", F::ZERO),
                gret: core::array::from_fn(|i| get_f(m, &format!("GRET({})", i + 1), F::ZERO)),
                grzeff: get_i(m, "GRZEFF", 0),
                fecf: core::array::from_fn(|i| get_f(m, &format!("FECF({})", i + 1), F::ZERO)),
                feclig: get_f(m, "FECLIG", F::from_f64(0.25)),
            },
        );
    }
    Ok(map)
}

/// Parse `harv.100`.
pub fn parse_harv_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, HarvParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        map.insert(
            blk.name.clone(),
            HarvParams {
                name: blk.name,
                aglrem: get_f(m, "AGLREM", F::ZERO),
                bglrem: get_f(m, "BGLREM", F::ZERO),
                flghrv: get_i(m, "FLGHRV", 0),
                rmvstr: get_f(m, "RMVSTR", F::ZERO),
                remwsd: get_f(m, "REMWSD", F::ZERO),
                hibg: get_f(m, "HIBG", F::ZERO),
            },
        );
    }
    Ok(map)
}

/// Parse `irri.100`.
pub fn parse_irri_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, IrriParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        map.insert(
            blk.name.clone(),
            IrriParams {
                name: blk.name,
                auirri: get_i(m, "AUIRRI", 0),
                fawhc: get_f(m, "FAWHC", F::ZERO),
                irraut: get_f(m, "IRRAUT", F::ZERO),
                irramt: get_f(m, "IRRAMT", F::ZERO),
            },
        );
    }
    Ok(map)
}

/// Parse `omad.100`.
pub fn parse_omad_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, OmadParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        map.insert(
            blk.name.clone(),
            OmadParams {
                name: blk.name,
                astgc: get_f(m, "ASTGC", F::ZERO),
                astlbl: get_f(m, "ASTLBL", F::ZERO),
                astlig: get_f(m, "ASTLIG", F::from_f64(0.2)),
                astrec: core::array::from_fn(|i| {
                    get_f(m, &format!("ASTREC({})", i + 1), F::from_f64(20.0))
                }),
            },
        );
    }
    Ok(map)
}

/// Parse `trem.100`.
pub fn parse_trem_file<F: Float, P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, TremParams<F>>> {
    let txt = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    for blk in parse_100_blocks(&txt) {
        let m = &blk.params;
        let remf = core::array::from_fn(|i| get_f(m, &format!("REMF({})", i + 1), F::ZERO));
        let fd = [get_f(m, "FD(1)", F::ZERO), get_f(m, "FD(2)", F::ZERO)];
        let retf = core::array::from_fn(|e| {
            core::array::from_fn(|c| get_f(m, &format!("RETF({},{})", e + 1, c + 1), F::ZERO))
        });
        map.insert(
            blk.name.clone(),
            TremParams {
                name: blk.name,
                evntyp: get_i(m, "EVNTYP", 0),
                remf,
                fd,
                retf,
            },
        );
    }
    Ok(map)
}
