//! Parameter structures cleanly separated into immutable biome-level parameters
//! (`RawBiomeParams<F>`, shared across threads via `Arc`) and per-cell derived
//! hydraulic/pedotransfer/stoichiometric parameters (`CellDerivedParams<F>`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::CompatibilityProfile;
use crate::float::Float;

/// Maximum number of soil water layers supported by CENTURY.
pub const MAX_LAYERS: usize = 10;
/// Maximum number of nutrient elements beyond Carbon (N=0, P=1, S=2).
pub const MAX_ELEM: usize = 3;
/// Number of live forest plant components (leaf, fine root, fine branch, large wood, coarse root).
pub const FOREST_PARTS: usize = 5;

/// Fixed ecosystem parameters (`fix.100`). Immutable once loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixParams<F: Float> {
    /// Thickness of each soil layer (cm).
    pub adep: [F; MAX_LAYERS],
    /// Aboveground annual precipitation productivity intercept & slope.
    pub agppa: F,
    /// Aboveground precipitation slope.
    pub agppb: F,
    /// Anaerobic decomposition parameters `[ratio_start, ratio_end, min_factor]`.
    pub aneref: [F; 3],
    /// Slope of anaerobic effect.
    pub animpt: F,
    /// Relative root water uptake weighting by layer.
    pub awtl: [F; MAX_LAYERS],
    /// Belowground annual precipitation productivity intercept & slope.
    pub bgppa: F,
    /// Belowground precipitation slope.
    pub bgppb: F,
    /// Baseline and doubled reference CO2 concentrations (ppm).
    pub co2ppm: [F; 2],
    /// CO2 ramp flag.
    pub co2rmp: F,
    /// Fraction of surface/soil N absorbed directly by residue `[2][3]`.
    pub damr: [[F; MAX_ELEM]; 2],
    /// Minimum mineral nutrient for direct absorption `[3]`.
    pub damrmn: [F; MAX_ELEM],
    /// Maximum annual structural decomposition rates `[surface, soil]` ($\text{yr}^{-1}$).
    pub dec1: [F; 2],
    /// Maximum annual metabolic decomposition rates `[surface, soil]` ($\text{yr}^{-1}$).
    pub dec2: [F; 2],
    /// Maximum annual active SOM decomposition rates `[surface, soil]` ($\text{yr}^{-1}$).
    pub dec3: [F; 2],
    /// Maximum annual passive SOM decomposition rate ($\text{yr}^{-1}$).
    pub dec4: F,
    /// Maximum annual slow SOM decomposition rate ($\text{yr}^{-1}$).
    pub dec5: F,
    /// Structural litter decay modifier `deck5`.
    pub deck5: F,
    /// Lignin decay factor `dligdf`.
    pub dligdf: F,
    /// Fraction of microbial biomass respired on death.
    pub dresp: F,
    /// Soil erosion depth (m).
    pub edepth: F,
    /// Litter-to-SOM stabilization effect.
    pub elitst: F,
    /// Erosion nutrient enrichment ratio.
    pub enrich: F,
    /// Available nutrient fraction for plant uptake `[N, P_min, S, P_max, P_s1, P_s2]`.
    pub favail: [F; 6],
    /// Leaching texture & element parameters `[intercept, sand_slope, N_mult, P_mult, S_mult]`.
    pub fleach: [F; 5],
    /// Water loss multipliers `[interception, bare_evap, transp, pet]`.
    pub fwloss: [F; 4],
    /// Symbiotic N-fixation moisture intercept & slope.
    pub fxmca: F,
    /// Symbiotic N-fixation moisture slope.
    pub fxmcb: F,
    /// Maximum non-symbiotic fixation scaler.
    pub fxmxs: F,
    /// N:P ratio control on fixation.
    pub fxnpb: F,
    /// Grazing effect parameter.
    pub gremb: F,
    /// Decomposition moisture function selector (`1` = relative water content, `2` = precip/PET ratio).
    pub idef: i32,
    /// Lower horizon fraction replenished on erosion `[som1, som2, som3]`.
    pub lhzf: [F; 3],
    /// Minimum critical water flux for solute leaching (cm/month).
    pub minlch: F,
    /// Non-symbiotic N fixation mode flag.
    pub nsnfix: i32,
    /// Number of decomposition sub-steps per month (default 4).
    pub ntspm: i32,
    /// Organic matter leaching parameters `[int, sand_slope, min_flux]`.
    pub omlech: [F; 3],
    /// Active SOM respiration intercept `[surface, soil]`.
    pub p1co2a: [F; 2],
    /// Active SOM respiration sand slope `[surface, soil]`.
    pub p1co2b: [F; 2],
    /// Slow SOM respiration fraction.
    pub p2co2: F,
    /// Passive SOM respiration fraction.
    pub p3co2: F,
    /// Residual P/S parameter `pabres`.
    pub pabres: F,
    /// Surface microbe C:E ratio limits `[3][3]`.
    pub pcemic: [[F; 3]; 3],
    /// Active SOM texture effect intercept.
    pub peftxa: F,
    /// Active SOM texture effect sand slope.
    pub peftxb: F,
    /// Soil pH effect on P sorption `[4]`.
    pub phesp: [F; 4],
    /// Lignin inhibition coefficient on structural decay `[surface, soil]`.
    pub pligst: [F; 2],
    /// Metabolic litter respiration fractions `[surface, soil]`.
    pub pmco2: [F; 2],
    /// Labile -> secondary mineral transfer rate `[N, P, S]`.
    pub pmnsec: [F; MAX_ELEM],
    /// Minimum soil temperature for growth/decomposition.
    pub pmntmp: F,
    /// Maximum biomass for shading effect.
    pub pmxbio: F,
    /// Maximum soil temperature scaling parameter.
    pub pmxtmp: F,
    /// Parent material weathering rate `[N, P, S]`.
    pub pparmn: [F; MAX_ELEM],
    /// Moisture production response parameters `[3]`.
    pub pprpts: [F; 3],
    /// Structural non-lignin respiration fraction `[surface, soil]`.
    pub ps1co2: [F; 2],
    /// Active -> Passive transfer `[intercept, clay_slope]`.
    pub ps1s3: [F; 2],
    /// Slow -> Passive transfer `[intercept, clay_slope]`.
    pub ps2s3: [F; 2],
    /// Secondary -> Labile desorption rate `[N, P, S]`.
    pub psecmn: [F; MAX_ELEM],
    /// Secondary -> Occluded P transfer rate.
    pub psecoc: F,
    /// Biochemical P/S mineralization rate parameters `[3][3]`.
    pub rad1p: [[F; 3]; 3],
    /// Fixed C:E ratio of structural litter `[N, P, S]`.
    pub rcestr: [F; MAX_ELEM],
    /// Root impact control `rictrl` and intercept `riint`.
    pub rictrl: F,
    /// Root impact intercept `riint`.
    pub riint: F,
    /// Lignin respiration fraction when flowing to Slow SOM.
    pub rsplig: F,
    /// Partitioning metabolic intercept and L:N slope `[0.85, 0.018]`.
    pub spl: [F; 2],
    /// Maximum structural litter C cap `[surface, soil]`.
    pub strmax: [F; 2],
    /// Texture effect on P sorption parameters `[5]`.
    pub texepp: [F; 5],
    /// Texture effect on P equilibrium `[0, 1, 2]`.
    pub texesp: [F; 3],
    /// Decomposition temperature response parameters `[t1, t2, t3]`.
    pub teff: [F; 3],
    /// Snowmelt temperature threshold & rate `[tmin, rate]`.
    pub tmelt: [F; 2],
    /// Active SOM target C:E ratio parameters `[max, min, mineral_ref]` per element `[3][3]`.
    pub varat1: [[F; 3]; MAX_ELEM],
    /// Slow SOM target C:E ratio parameters `[max, min, mineral_ref]` per element `[3][3]`.
    pub varat2: [[F; 3]; MAX_ELEM],
    /// Passive SOM target C:E ratio parameters `[max, min, mineral_ref]` per element `[3][3]`.
    pub varat3: [[F; 3]; MAX_ELEM],
    /// N volatilization fraction on crop harvest / senescence (`vlosse`).
    pub vlosse: F,
    /// N volatilization fraction on gross mineralization (`vlossg`).
    pub vlossg: F,
}

impl<F: Float> Default for FixParams<F> {
    fn default() -> Self {
        let f = F::from_f64;
        Self {
            adep: [
                f(15.0),
                f(15.0),
                f(15.0),
                f(15.0),
                f(30.0),
                f(30.0),
                f(30.0),
                f(30.0),
                f(30.0),
                f(30.0),
            ],
            agppa: f(-40.0),
            agppb: f(7.7),
            aneref: [f(1.5), f(3.0), f(0.3)],
            animpt: f(5.0),
            awtl: [
                f(0.8),
                f(0.6),
                f(0.4),
                f(0.3),
                f(0.2),
                f(0.2),
                f(0.2),
                f(0.2),
                f(0.0),
                f(0.0),
            ],
            bgppa: f(100.0),
            bgppb: f(7.0),
            co2ppm: [f(350.0), f(700.0)],
            co2rmp: f(1.0),
            damr: [[f(0.0), f(0.0), f(0.01)], [f(0.02), f(0.02), f(0.04)]],
            damrmn: [f(15.0), f(150.0), f(150.0)],
            dec1: [f(3.9), f(4.9)],
            dec2: [f(14.8), f(18.5)],
            dec3: [f(6.0), f(7.3)],
            dec4: f(0.0045),
            dec5: f(0.2),
            deck5: f(5.0),
            dligdf: f(0.0),
            dresp: f(0.0),
            edepth: f(0.2),
            elitst: f(0.4),
            enrich: f(2.0),
            favail: [f(0.9), f(0.1), f(0.5), f(0.2), f(0.4), f(2.0)],
            fleach: [f(0.2), f(0.7), f(1.0), f(0.0), f(0.1)],
            fwloss: [f(0.8), f(0.8), f(0.65), f(0.70)],
            fxmca: f(0.0),
            fxmcb: f(0.0),
            fxmxs: f(1.0),
            fxnpb: f(7.0),
            gremb: f(0.0),
            idef: 2,
            lhzf: [f(0.01), f(0.49), f(0.50)],
            minlch: f(18.0),
            nsnfix: 0,
            ntspm: 4,
            omlech: [f(0.03), f(0.12), f(60.0)],
            p1co2a: [f(0.6), f(0.17)],
            p1co2b: [f(0.0), f(0.68)],
            p2co2: f(0.55),
            p3co2: f(0.55),
            pabres: f(100.0),
            pcemic: [
                [f(16.0), f(10.0), f(0.02)],
                [f(16.0), f(10.0), f(0.02)],
                [f(16.0), f(10.0), f(0.02)],
            ],
            peftxa: f(0.25),
            peftxb: f(0.75),
            phesp: [f(1.0), f(0.0), f(6.5), f(0.0)],
            pligst: [f(3.0), f(3.0)],
            pmco2: [f(0.55), f(0.55)],
            pmnsec: [f(0.0), f(0.06), f(0.02)],
            pmntmp: f(0.004),
            pmxbio: f(600.0),
            pmxtmp: f(-0.0035),
            pparmn: [f(0.0), f(0.0005), f(0.001)],
            pprpts: [f(0.0), f(1.0), f(0.8)],
            ps1co2: [f(0.45), f(0.55)],
            ps1s3: [f(0.003), f(0.032)],
            ps2s3: [f(0.003), f(0.009)],
            psecmn: [f(0.0), f(0.0015), f(0.005)],
            psecoc: f(0.0001),
            rad1p: [
                [f(12.0), f(3.0), f(5.0)],
                [f(12.0), f(3.0), f(5.0)],
                [f(12.0), f(3.0), f(5.0)],
            ],
            rcestr: [f(150.0), f(500.0), f(500.0)],
            rictrl: f(0.015),
            riint: f(0.8),
            rsplig: f(0.3),
            spl: [f(0.85), f(0.018)],
            strmax: [f(5000.0), f(5000.0)],
            texepp: [f(1.0), f(0.0), f(0.0), f(0.0), f(0.0)],
            texesp: [f(1.0), f(1.0), f(1.0)],
            teff: [f(15.4), f(11.75), f(29.7)],
            tmelt: [f(-8.0), f(4.0)],
            varat1: [
                [f(14.0), f(3.0), f(2.0)],
                [f(80.0), f(30.0), f(2.0)],
                [f(80.0), f(30.0), f(2.0)],
            ],
            varat2: [
                [f(20.0), f(12.0), f(2.0)],
                [f(400.0), f(100.0), f(2.0)],
                [f(400.0), f(100.0), f(2.0)],
            ],
            varat3: [
                [f(10.0), f(6.0), f(2.0)],
                [f(200.0), f(50.0), f(2.0)],
                [f(200.0), f(50.0), f(2.0)],
            ],
            vlosse: f(0.02),
            vlossg: f(0.02),
        }
    }
}

/// Site-level initial conditions and defaults (`<site>.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteParams<F: Float> {
    /// Monthly mean precipitation (cm).
    pub precip: [F; 12],
    /// Monthly mean minimum temperature (°C).
    pub tmn2m: [F; 12],
    /// Monthly mean maximum temperature (°C).
    pub tmx2m: [F; 12],
    /// Automatic initial SOM mode (`0` = use file values, `1` = grassland, `2` = crop, `3` = forest).
    pub ivauto: i32,
    /// Number of active nutrient elements (`1`=N, `2`=NP, `3`=NPS).
    pub nelem: usize,
    /// Site latitude (degrees).
    pub sitlat: F,
    /// Site longitude (degrees).
    pub sitlng: F,
    /// Soil sand fraction `[0, 1]`.
    pub sand: F,
    /// Soil silt fraction `[0, 1]`.
    pub silt: F,
    /// Soil clay fraction `[0, 1]`.
    pub clay: F,
    /// Soil bulk density ($\text{g}\cdot\text{cm}^{-3}$).
    pub bulkd: F,
    /// Number of soil layers (`1..=10`).
    pub nlayer: usize,
    /// Number of rooting layers (`1..=nlayer`).
    pub nlaypg: usize,
    /// Drainage fraction (`0..=1`, `1` = well drained).
    pub drain: F,
    /// Baseflow fraction of deep water storage.
    pub basef: F,
    /// Stormflow fraction of surface excess.
    pub stormf: F,
    /// Water retention equation flag (`0` = Rawls 1982, `1` = Gupta & Larson 1979, `2` = explicit).
    pub swflag: i32,
    /// Explicit wilting point per layer (volumetric).
    pub awilt: [F; MAX_LAYERS],
    /// Explicit field capacity per layer (volumetric).
    pub afiel: [F; MAX_LAYERS],
    /// Soil pH.
    pub ph: F,
    /// P sorption slope and max.
    pub pslsrb: F,
    /// Maximum P sorption capacity.
    pub sorpmx: F,
    /// Atmospheric N deposition `[intercept, precip_slope]`.
    pub epnfa: [F; 2],
    /// Non-symbiotic N fixation `[intercept, precip_slope]`.
    pub epnfs: [F; 2],
    /// Atmospheric S deposition `[intercept, precip_slope]`.
    pub satmos: [F; 2],
    /// Irrigation water S concentration (mg S / L).
    pub sirri: F,
    /// Initial SOM1 C `[surface/soil][unlab/lab]`.
    pub som1ci: [[F; 2]; 2],
    /// Initial SOM2 C `[unlab, lab]`.
    pub som2ci: [F; 2],
    /// Initial SOM3 C `[unlab, lab]`.
    pub som3ci: [F; 2],
    /// Initial SOM1 C:E ratios `[surface/soil][N, P, S]`.
    pub rces1: [[F; MAX_ELEM]; 2],
    /// Initial SOM2 C:E ratios `[N, P, S]`.
    pub rces2: [F; MAX_ELEM],
    /// Initial SOM3 C:E ratios `[N, P, S]`.
    pub rces3: [F; MAX_ELEM],
    /// Initial litter C `[surface/soil][unlab/lab]`.
    pub clittr: [[F; 2]; 2],
    /// Initial litter C:E ratios `[surface/soil][N, P, S]`.
    pub rcelit: [[F; MAX_ELEM]; 2],
    /// Initial herbaceous live shoot C `[unlab, lab]` and nutrients.
    pub aglcis: [F; 2],
    /// Initial live shoot nutrients `[N, P, S]`.
    pub aglive: [F; MAX_ELEM],
    /// Initial herbaceous live root C `[unlab, lab]`.
    pub bglcis: [F; 2],
    /// Initial live root nutrients `[N, P, S]`.
    pub bglive: [F; MAX_ELEM],
    /// Initial standing dead C `[unlab, lab]`.
    pub stdcis: [F; 2],
    /// Initial standing dead nutrients `[N, P, S]`.
    pub stdede: [F; MAX_ELEM],
    /// Initial forest leaf C `[unlab, lab]`.
    pub rlvcis: [F; 2],
    /// Initial forest leaf nutrients `[N, P, S]`.
    pub rleave: [F; MAX_ELEM],
    /// Initial fine branch C `[unlab, lab]`.
    pub fbrcis: [F; 2],
    /// Initial fine branch nutrients `[N, P, S]`.
    pub fbrche: [F; MAX_ELEM],
    /// Initial large wood C `[unlab, lab]`.
    pub rlwcis: [F; 2],
    /// Initial large wood nutrients `[N, P, S]`.
    pub rlwode: [F; MAX_ELEM],
    /// Initial fine root C `[unlab, lab]`.
    pub frtcis: [F; 2],
    /// Initial fine root nutrients `[N, P, S]`.
    pub froote: [F; MAX_ELEM],
    /// Initial coarse root C `[unlab, lab]`.
    pub crtcis: [F; 2],
    /// Initial coarse root nutrients `[N, P, S]`.
    pub croote: [F; MAX_ELEM],
    /// Initial dead wood C (`wood1`, `wood2`, `wood3`) `[unlab, lab]`.
    pub wd1cis: [F; 2],
    /// Initial dead large wood C `[unlab, lab]`.
    pub wd2cis: [F; 2],
    /// Initial dead coarse root C `[unlab, lab]`.
    pub wd3cis: [F; 2],
    /// Dead wood lignin fractions.
    pub w1lig: F,
    /// Dead large wood lignin fraction.
    pub w2lig: F,
    /// Dead coarse root lignin fraction.
    pub w3lig: F,
    /// Initial mineral nutrients `[layer][elem]`.
    pub minerl: [[F; MAX_ELEM]; MAX_LAYERS],
    /// Initial parent material `[N, P, S]`.
    pub parent: [F; MAX_ELEM],
    /// Initial secondary mineral `[N, P, S]`.
    pub secndy: [F; MAX_ELEM],
    /// Initial occluded P.
    pub occlud: F,
    /// Initial relative water content per layer `[0, 1]`.
    pub rwcf: [F; MAX_LAYERS],
    /// Initial liquid snow water (cm).
    pub snlq: F,
    /// Initial snowpack water equivalent (cm).
    pub snow: F,
}

impl<F: Float> Default for SiteParams<F> {
    fn default() -> Self {
        let f = F::from_f64;
        let mut minerl = [[F::ZERO; MAX_ELEM]; MAX_LAYERS];
        minerl[0] = [f(1.5), f(0.5), f(0.5)];
        minerl[1] = [f(1.0), f(0.4), f(0.4)];
        minerl[2] = [f(0.5), f(0.3), f(0.3)];
        Self {
            precip: [
                f(5.0),
                f(5.0),
                f(6.0),
                f(7.0),
                f(8.0),
                f(8.0),
                f(7.0),
                f(6.0),
                f(6.0),
                f(5.0),
                f(5.0),
                f(5.0),
            ],
            tmn2m: [
                f(-5.0),
                f(-3.0),
                f(1.0),
                f(6.0),
                f(11.0),
                f(15.0),
                f(18.0),
                f(17.0),
                f(12.0),
                f(6.0),
                f(1.0),
                f(-4.0),
            ],
            tmx2m: [
                f(4.0),
                f(7.0),
                f(12.0),
                f(18.0),
                f(23.0),
                f(28.0),
                f(31.0),
                f(30.0),
                f(25.0),
                f(19.0),
                f(11.0),
                f(5.0),
            ],
            ivauto: 0,
            nelem: 1,
            sitlat: f(40.0),
            sitlng: f(-105.0),
            sand: f(0.40),
            silt: f(0.35),
            clay: f(0.25),
            bulkd: f(1.35),
            nlayer: 6,
            nlaypg: 4,
            drain: f(1.0),
            basef: f(0.3),
            stormf: f(0.6),
            swflag: 0,
            awilt: [f(0.12); MAX_LAYERS],
            afiel: [f(0.32); MAX_LAYERS],
            ph: f(6.5),
            pslsrb: f(1.0),
            sorpmx: f(200.0),
            epnfa: [f(0.2), f(0.005)],
            epnfs: [f(0.1), f(0.003)],
            satmos: [f(0.1), f(0.002)],
            sirri: f(0.0),
            som1ci: [[f(20.0), F::ZERO], [f(150.0), F::ZERO]],
            som2ci: [f(2000.0), F::ZERO],
            som3ci: [f(1500.0), F::ZERO],
            rces1: [[f(15.0), f(150.0), f(150.0)], [f(10.0), f(80.0), f(80.0)]],
            rces2: [f(16.0), f(200.0), f(200.0)],
            rces3: [f(9.0), f(100.0), f(100.0)],
            clittr: [[f(100.0), F::ZERO], [f(100.0), F::ZERO]],
            rcelit: [[f(50.0), f(400.0), f(400.0)], [f(50.0), f(400.0), f(400.0)]],
            aglcis: [f(50.0), F::ZERO],
            aglive: [f(2.0), f(0.2), f(0.2)],
            bglcis: [f(100.0), F::ZERO],
            bglive: [f(2.5), f(0.25), f(0.25)],
            stdcis: [f(50.0), F::ZERO],
            stdede: [f(1.0), f(0.1), f(0.1)],
            rlvcis: [F::ZERO, F::ZERO],
            rleave: [F::ZERO; MAX_ELEM],
            fbrcis: [F::ZERO, F::ZERO],
            fbrche: [F::ZERO; MAX_ELEM],
            rlwcis: [F::ZERO, F::ZERO],
            rlwode: [F::ZERO; MAX_ELEM],
            frtcis: [F::ZERO, F::ZERO],
            froote: [F::ZERO; MAX_ELEM],
            crtcis: [F::ZERO, F::ZERO],
            croote: [F::ZERO; MAX_ELEM],
            wd1cis: [F::ZERO, F::ZERO],
            wd2cis: [F::ZERO, F::ZERO],
            wd3cis: [F::ZERO, F::ZERO],
            w1lig: f(0.25),
            w2lig: f(0.30),
            w3lig: f(0.30),
            minerl,
            parent: [F::ZERO, f(50.0), f(30.0)],
            secndy: [F::ZERO, f(20.0), f(10.0)],
            occlud: f(5.0),
            rwcf: [f(0.5); MAX_LAYERS],
            snlq: F::ZERO,
            snow: F::ZERO,
        }
    }
}

/// Crop / grassland option parameters (`crop.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropParams<F: Float> {
    /// Option identifier (e.g. `"TMC3"`, `"C4G"`).
    pub name: String,
    /// Potential monthly aboveground production ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$).
    pub prdx1: F,
    /// Temperature response parameters `[t_opt, t_max, a, b]`.
    pub ppdf: [F; 4],
    /// Live biomass shading flag (`1` = active).
    pub bioflg: F,
    /// Biomass at which growth is 50% of max (`biok5`).
    pub biok5: F,
    /// Planting month reduction factor.
    pub pltmrf: F,
    /// Full canopy biomass threshold (`fulcan`).
    pub fulcan: F,
    /// Fixed root-to-shoot allocation parameters `[f_init, f_final, months]`.
    pub frtc: [F; 3],
    /// Biomass for maximum shoot C:E interpolation.
    pub biomax: F,
    /// Minimum shoot C:E `[elem][0=zero_bio, 1=max_bio]`.
    pub pramn: [[F; 2]; MAX_ELEM],
    /// Maximum shoot C:E `[elem][0=zero_bio, 1=max_bio]`.
    pub pramx: [[F; 2]; MAX_ELEM],
    /// Minimum root C:E `[elem][0=intercept, 1=precip_slope]`.
    pub prbmn: [[F; 2]; MAX_ELEM],
    /// Maximum root C:E `[elem][0=intercept, 1=precip_slope]`.
    pub prbmx: [[F; 2]; MAX_ELEM],
    /// Lignin parameters `fligni[0=intercept/1=slope][0=shoot/1=root]`.
    pub fligni: [[F; 2]; 2],
    /// Maximum grain harvest index.
    pub himax: F,
    /// Harvest index water stress factor.
    pub hiwsf: F,
    /// Grain fill window months `[start, end]`.
    pub himon: [i32; 2],
    /// Fraction of aboveground E partitioned to grain `[N, P, S]`.
    pub efrgrn: [F; MAX_ELEM],
    /// Volatilization loss of N at harvest/senescence.
    pub vlossp: F,
    /// Shoot death parameters `[max_drought, senm_frac, shade_extra, shade_bio_thresh]`.
    pub fsdeth: [F; 4],
    /// Standing dead fall rate per month.
    pub fallrt: F,
    /// Maximum monthly root death rate.
    pub rdr: F,
    /// Soil temperature threshold for root death (°C).
    pub rtdtmp: F,
    /// Crop nutrient retranslocation fraction on shoot death `[N, P, S]`.
    pub crprtf: [F; MAX_ELEM],
    /// Maximum symbiotic N fixation (g N / g C fixed).
    pub snfxmx: F,
    /// $\delta^{13}\text{C}$ signature for C-13 isotope tracking.
    pub del13c: F,
    /// CO2 production multiplier at doubled CO2.
    pub co2ipr: F,
    /// CO2 transpiration multiplier at doubled CO2.
    pub co2itr: F,
    /// CO2 C:E ratio multipliers `[min/max][N, P, S]`.
    pub co2ice: [[F; MAX_ELEM]; 2],
    /// CO2 root/shoot multiplier at doubled CO2.
    pub co2irs: F,
}

impl<F: Float> Default for CropParams<F> {
    fn default() -> Self {
        let f = F::from_f64;
        Self {
            name: "TMC3".to_string(),
            prdx1: f(270.0),
            ppdf: [f(18.0), f(35.0), f(1.2), f(4.5)],
            bioflg: f(1.0),
            biok5: f(60.0),
            pltmrf: f(1.0),
            fulcan: f(100.0),
            frtc: [F::ZERO, F::ZERO, F::ZERO],
            biomax: f(400.0),
            pramn: [
                [f(20.0), f(30.0)],
                [f(390.0), f(390.0)],
                [f(340.0), f(340.0)],
            ],
            pramx: [
                [f(30.0), f(40.0)],
                [f(440.0), f(440.0)],
                [f(440.0), f(440.0)],
            ],
            prbmn: [[f(40.0), F::ZERO], [f(390.0), F::ZERO], [f(340.0), F::ZERO]],
            prbmx: [[f(50.0), F::ZERO], [f(420.0), F::ZERO], [f(420.0), F::ZERO]],
            fligni: [[f(0.02), f(0.26)], [f(0.0012), f(-0.0015)]],
            himax: F::ZERO,
            hiwsf: F::ZERO,
            himon: [0, 0],
            efrgrn: [f(0.5), f(0.5), f(0.5)],
            vlossp: f(0.0),
            fsdeth: [f(0.2), f(0.5), f(0.2), f(150.0)],
            fallrt: f(0.1),
            rdr: f(0.05),
            rtdtmp: f(2.0),
            crprtf: [f(0.0), f(0.0), f(0.0)],
            snfxmx: F::ZERO,
            del13c: f(-27.0),
            co2ipr: f(1.3),
            co2itr: f(0.75),
            co2ice: [[f(1.0); MAX_ELEM]; 2],
            co2irs: f(1.0),
        }
    }
}

/// Forest / tree option parameters (`tree.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeParams<F: Float> {
    /// Option identifier (e.g. `"MTCF"`, `"AASS"`).
    pub name: String,
    /// Deciduous flag (`1` = deciduous, `0` = evergreen).
    pub decid: i32,
    /// Potential monthly total forest production (`prdx(4)` / `prdx(3)`).
    pub prdx3: F,
    /// Potential monthly forest C production ($\text{g C}\cdot\text{m}^{-2}\cdot\text{mo}^{-1}$).
    pub prdx4: F,
    /// Forest temperature response parameters `[t_opt, t_max, a, b]`.
    pub ppdf: [F; 4],
    /// Forest C:E ratios `[0=min, 1=max, 2=initial][part][elem]`.
    pub cerfor: [[[F; MAX_ELEM]; FOREST_PARTS]; 3],
    /// Dead wood decay rates (`wood1`, `wood2`, `wood3`, $\text{yr}^{-1}$).
    pub decw: [F; 3],
    /// Carbon allocation fractions `[0=juvenile, 1=mature][part]`.
    pub fcfrac: [[F; FOREST_PARTS]; 2],
    /// Monthly leaf drop fraction (`month 0..11`).
    pub leafdr: [F; 12],
    /// Leaf biomass to LAI conversion (`btolai`).
    pub btolai: F,
    /// Large wood mass at half max LAI (`klai`).
    pub klai: F,
    /// Canopy light extinction coefficient (`laitop`).
    pub laitop: F,
    /// Maximum theoretical LAI.
    pub maxlai: F,
    /// Maximum drought leaf drop multiplier.
    pub maxldr: F,
    /// Foliar nutrient retranslocation fraction on leaf drop `[N, P, S]`.
    pub forrtf: [F; MAX_ELEM],
    /// Sapwood fraction parameters.
    pub sapk: F,
    /// Sapwood turnover parameter.
    pub swold: F,
    /// Lignin fractions for the 5 forest components.
    pub wdlig: [F; FOREST_PARTS],
    /// Monthly mortality rates for the 5 forest components.
    pub wooddr: [F; FOREST_PARTS],
    /// Forest symbiotic N fixation (g N / g C fixed).
    pub snfxmx: F,
    /// $\delta^{13}\text{C}$ signature.
    pub del13c: F,
    /// CO2 production multiplier at doubled CO2.
    pub co2ipr: F,
    /// CO2 transpiration multiplier at doubled CO2.
    pub co2itr: F,
    /// CO2 C:E ratio multipliers `[min/max][N, P, S]`.
    pub co2ice: [[F; MAX_ELEM]; 2],
    /// CO2 root/shoot multiplier.
    pub co2irs: F,
    /// Basal area / tree-grass competition parameters.
    pub basfc2: F,
    /// Basal area factor.
    pub basfct: F,
    /// Savanna site tree potential.
    pub sitpot: F,
}

impl<F: Float> Default for TreeParams<F> {
    fn default() -> Self {
        let f = F::from_f64;
        Self {
            name: "MTCF".to_string(),
            decid: 0,
            prdx3: f(9999.0),
            prdx4: f(400.0),
            ppdf: [f(18.0), f(35.0), f(1.0), f(4.5)],
            cerfor: [
                [
                    [f(35.0), f(400.0), f(350.0)],
                    [f(40.0), f(400.0), f(350.0)],
                    [f(100.0), f(800.0), f(800.0)],
                    [f(200.0), f(1500.0), f(1500.0)],
                    [f(120.0), f(1000.0), f(1000.0)],
                ],
                [
                    [f(65.0), f(600.0), f(550.0)],
                    [f(70.0), f(600.0), f(550.0)],
                    [f(180.0), f(1200.0), f(1200.0)],
                    [f(400.0), f(2500.0), f(2500.0)],
                    [f(200.0), f(1500.0), f(1500.0)],
                ],
                [
                    [f(50.0), f(500.0), f(450.0)],
                    [f(55.0), f(500.0), f(450.0)],
                    [f(140.0), f(1000.0), f(1000.0)],
                    [f(300.0), f(2000.0), f(2000.0)],
                    [f(160.0), f(1200.0), f(1200.0)],
                ],
            ],
            decw: [f(1.5), f(0.5), f(0.6)],
            fcfrac: [
                [f(0.30), f(0.25), f(0.15), f(0.20), f(0.10)],
                [f(0.25), f(0.20), f(0.15), f(0.30), f(0.10)],
            ],
            leafdr: [f(0.02); 12],
            btolai: f(0.004),
            klai: f(2000.0),
            laitop: f(-0.47),
            maxlai: f(10.0),
            maxldr: f(1.0),
            forrtf: [f(0.5), f(0.5), f(0.5)],
            sapk: f(1500.0),
            swold: f(0.1),
            wdlig: [f(0.20), f(0.25), f(0.25), f(0.30), f(0.30)],
            wooddr: [f(0.02), f(0.03), f(0.005), f(0.001), f(0.002)],
            snfxmx: F::ZERO,
            del13c: f(-27.0),
            co2ipr: f(1.3),
            co2itr: f(0.75),
            co2ice: [[f(1.0); MAX_ELEM]; 2],
            co2irs: f(1.0),
            basfc2: f(1.0),
            basfct: f(1.0),
            sitpot: f(1000.0),
        }
    }
}

/// Cultivation event parameters (`cult.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Transfer fractions `cultra[0..7]`:
    /// 0: aglivc->surface litter, 1: aglivc->soil litter, 2: stdedc->surface litter,
    /// 3: stdedc->soil litter, 4: surface structural->soil structural,
    /// 5: surface metabolic->soil metabolic, 6: bglivc->soil litter.
    pub cultra: [F; 7],
    /// Decomposition multipliers `[som1, som2, som3, structural]` (Manual A2-4/A2-32).
    pub clteff: [F; 4],
}

/// Fertilization event parameters (`fert.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FertParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Fixed nutrient addition `[N, P, S]` ($\text{g}\cdot\text{m}^{-2}$).
    pub feramt: [F; MAX_ELEM],
    /// Auto-fertilization target fraction (`0` = fixed mode).
    pub aufert: F,
}

/// Fire event parameters (`fire.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Fraction of live shoots burned.
    pub flfrem: F,
    /// Fraction of `[standing_dead, surface_litter]` burned.
    pub fdfrem: [F; 2],
    /// Fraction of burned `[N, P, S]` returned to mineral ash.
    pub fret: [F; MAX_ELEM],
    /// Post-fire root:shoot ratio increase.
    pub frtsh: F,
    /// Post-fire shoot C:N adjustment `[min, max]`.
    pub fnue: [F; 2],
}

/// Grazing event parameters (`graz.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrazParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Fraction of live shoots removed by grazing.
    pub flgrem: F,
    /// Fraction of standing dead removed by grazing.
    pub fdgrem: F,
    /// Fraction of consumed C returned in feces.
    pub gfcret: F,
    /// Fraction of consumed `[N, P, S]` returned in excreta.
    pub gret: [F; MAX_ELEM],
    /// Grazing production/root feedback code.
    pub grzeff: i32,
    /// Fraction of returned `[N, P, S]` in feces (remainder in urine).
    pub fecf: [F; MAX_ELEM],
    /// Lignin fraction of feces.
    pub feclig: F,
}

/// Harvest event parameters (`harv.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Fraction of live shoots killed/affected.
    pub aglrem: F,
    /// Fraction of live roots killed.
    pub bglrem: F,
    /// Grain harvest flag (`1` = harvest grain).
    pub flghrv: i32,
    /// Fraction of straw removed off-site.
    pub rmvstr: F,
    /// Fraction of remaining straw transferred to standing dead (vs surface litter).
    pub remwsd: F,
    /// Belowground tuber/root harvest fraction.
    pub hibg: F,
}

/// Irrigation event parameters (`irri.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrriParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Auto-irrigation mode (`0` = fixed amount, `1` = refill to `fawhc`, `2` = add `irraut`).
    pub auirri: i32,
    /// Target fraction of available water holding capacity.
    pub fawhc: F,
    /// Auto-irrigation trigger/fixed addition (cm).
    pub irraut: F,
    /// Fixed monthly irrigation addition (cm).
    pub irramt: F,
}

/// Organic matter addition parameters (`omad.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmadParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Carbon added ($\text{g C}\cdot\text{m}^{-2}$).
    pub astgc: F,
    /// Labeled isotope fraction of added C.
    pub astlbl: F,
    /// Lignin fraction of added organic matter.
    pub astlig: F,
    /// C:E ratios of added organic matter `[N, P, S]`.
    pub astrec: [F; MAX_ELEM],
}

/// Tree removal / logging / canopy burn parameters (`trem.100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TremParams<F: Float> {
    /// Option name.
    pub name: String,
    /// Event type (`0` = cut/harvest, `1` = fire/canopy burn).
    pub evntyp: i32,
    /// Removal fractions `[leaves, fine_branch, large_wood, wood1, wood2]`.
    pub remf: [F; 5],
    /// Fractional mortality of `[fine_roots, coarse_roots]` on cut/burn.
    pub fd: [F; 2],
    /// Nutrient return fractions `[elem][0=leaves, 1=fbrch, 2=rlwod, 3=dead_wood]`.
    pub retf: [[F; 4]; MAX_ELEM],
}

/// Complete immutable biome parameter bundle, designed to be wrapped in `Arc`
/// and shared with zero contention across all CPU threads (including 256-thread EPYC).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawBiomeParams<F: Float> {
    /// Biome/landuse numeric ID.
    pub biome_id: i32,
    /// Fixed parameters (`fix.100`).
    pub fix: FixParams<F>,
    /// Site default template (`lu*.100`).
    pub site_defaults: SiteParams<F>,
    /// Crop parameter lookup table (`crop.100`).
    pub crops: HashMap<String, CropParams<F>>,
    /// Tree parameter lookup table (`tree.100`).
    pub trees: HashMap<String, TreeParams<F>>,
    /// Cultivation lookup table (`cult.100`).
    pub cults: HashMap<String, CultParams<F>>,
    /// Fertilization lookup table (`fert.100`).
    pub ferts: HashMap<String, FertParams<F>>,
    /// Fire lookup table (`fire.100`).
    pub fires: HashMap<String, FireParams<F>>,
    /// Grazing lookup table (`graz.100`).
    pub grazs: HashMap<String, GrazParams<F>>,
    /// Harvest lookup table (`harv.100`).
    pub harvs: HashMap<String, HarvParams<F>>,
    /// Irrigation lookup table (`irri.100`).
    pub irris: HashMap<String, IrriParams<F>>,
    /// Organic amendment lookup table (`omad.100`).
    pub omads: HashMap<String, OmadParams<F>>,
    /// Tree removal lookup table (`trem.100`).
    pub trems: HashMap<String, TremParams<F>>,
}

/// Per-cell geographic and soil overrides (e.g. from VEMAP `geog.nc` and `.v2` grids).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSiteInput<F: Float> {
    /// Latitude (degrees).
    pub lat: F,
    /// Longitude (degrees).
    pub lon: F,
    /// Elevation (m).
    pub elevation: F,
    /// Sand fraction `[0, 1]`.
    pub sand: F,
    /// Silt fraction `[0, 1]`.
    pub silt: F,
    /// Clay fraction `[0, 1]`.
    pub clay: F,
    /// Bulk density ($\text{g}\cdot\text{cm}^{-3}$).
    pub bulkd: F,
    /// Rock volume fraction `[0, 1]`.
    pub rock_frac: F,
    /// Mineral soil depth (cm). `None` uses `site_defaults.nlayer`.
    pub zminrl_cm: Option<F>,
    /// Soil pH.
    pub ph: F,
    /// Drainage fraction (`0..=1`).
    pub drain: F,
}

impl<F: Float> CellSiteInput<F> {
    /// Construct from a `SiteParams` template with zero rock fraction.
    pub fn from_site(site: &SiteParams<F>) -> Self {
        Self {
            lat: site.sitlat,
            lon: site.sitlng,
            elevation: F::ZERO,
            sand: site.sand,
            silt: site.silt,
            clay: site.clay,
            bulkd: site.bulkd,
            rock_frac: F::ZERO,
            zminrl_cm: None,
            ph: site.ph,
            drain: site.drain,
        }
    }
}

/// Per-cell derived soil, hydraulic, and geochemical parameters.
///
/// Computed once during cell initialization without mutating `RawBiomeParams`,
/// eliminating legacy cross-cell global-array contamination bugs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellDerivedParams<F: Float> {
    /// Latitude (degrees).
    pub lat: F,
    /// Longitude (degrees).
    pub lon: F,
    /// Elevation (m).
    pub elevation: F,
    /// Texture fractions `[0, 1]`.
    pub sand: F,
    /// Silt fraction `[0, 1]`.
    pub silt: F,
    /// Clay fraction `[0, 1]`.
    pub clay: F,
    /// Bulk density ($\text{g}\cdot\text{cm}^{-3}$).
    pub bulkd: F,
    /// Soil pH.
    pub ph: F,
    /// Drainage factor (`0..=1`).
    pub drain: F,
    /// Baseflow fraction.
    pub basef: F,
    /// Stormflow fraction.
    pub stormf: F,
    /// Active soil layer count (`1..=10`).
    pub nlayer: usize,
    /// Active rooting layer count (`1..=nlayer`).
    pub nlaypg: usize,
    /// Layer thicknesses (cm).
    pub adep: [F; MAX_LAYERS],
    /// Effective volumetric rock fraction `[0, 0.90]`.
    pub effective_rockfrac: F,
    /// Total mineral profile depth (cm).
    pub soil_depth_cm: F,
    /// Effective field capacity per layer (volumetric).
    pub afiel: [F; MAX_LAYERS],
    /// Effective wilting point per layer (volumetric).
    pub awilt: [F; MAX_LAYERS],
    /// Available water holding capacity per layer (cm).
    pub awhc: [F; MAX_LAYERS],
    /// Total available water holding capacity across rooting layers (cm).
    pub twhc_root: F,
    /// Total available water holding capacity across full profile (cm).
    pub twhc_profile: F,
    /// Active SOM texture decomposition multiplier (`peftxa + peftxb * sand`).
    pub eftext: F,
    /// Soil active SOM respiration fraction (`p1co2a[1] + p1co2b[1] * sand`).
    pub p1co2_soil: F,
    /// Active -> Passive transfer fraction (`ps1s3[0] + ps1s3[1] * clay`).
    pub fps1s3: F,
    /// Slow -> Passive transfer fraction (`ps2s3[0] + ps2s3[1] * clay`).
    pub fps2s3: F,
    /// Mineral N leaching texture fraction.
    pub fleach_n: F,
    /// Mineral P leaching fraction.
    pub fleach_p: F,
    /// Mineral S leaching texture fraction.
    pub fleach_s: F,
    /// Organic matter leaching texture fraction.
    pub omlech_frac: F,
    /// Per-cell texture/pH-adjusted P sorption slope (`texesp_2`).
    pub texesp_2: F,
    /// Per-cell texture/pH-adjusted P parent weathering (`pparmn_2`).
    pub pparmn_2: F,
    /// Per-cell texture/pH-adjusted P secondary desorption (`psecmn_2`).
    pub psecmn_2: F,
    /// Annual air temperature range $T_{\text{warmest}} - T_{\text{coldest}}$ (°C) for Linacre PET.
    pub trange_annual: F,
}

impl<F: Float> CellDerivedParams<F> {
    /// Derive all per-cell hydraulic and soil-texture parameters cleanly from
    /// immutable biome parameters and cell-level inputs.
    pub fn compute(
        fix: &FixParams<F>,
        site: &SiteParams<F>,
        cell: &CellSiteInput<F>,
        initial_somsc: F,
        annual_mean_temps: &[F; 12],
        profile: CompatibilityProfile,
    ) -> Self {
        let f = F::from_f64;

        // Determine nlayer / nlaypg from zminrl_cm if provided (VEMAP convention).
        let (nlayer, nlaypg) = if let Some(z) = cell.zminrl_cm {
            let z_val = z.to_f64();
            if z_val <= 30.0 {
                (2, 2)
            } else if z_val <= 120.0 {
                (6, 4.min(site.nlaypg).max(2))
            } else if z_val <= 150.0 {
                (7, 6.min(site.nlaypg).max(4))
            } else {
                (8, 7.min(site.nlaypg).max(6))
            }
        } else {
            let nl = site.nlayer.clamp(1, MAX_LAYERS);
            let np = site.nlaypg.clamp(1, nl);
            (nl, np)
        };

        let mut soil_depth = F::ZERO;
        for i in 0..nlayer {
            soil_depth += fix.adep[i];
        }

        // Effective rock fraction (Spec 02 §1.2 & Spec 07 Delta #3).
        let mut eff_rock = cell.rock_frac.clamp_val(F::ZERO, f(0.90));
        if let Some(z) = cell.zminrl_cm {
            if z < soil_depth && soil_depth > F::ZERO {
                eff_rock += (soil_depth - z) / soil_depth;
            }
        }
        eff_rock = eff_rock.clamp_val(F::ZERO, f(0.90));
        if profile == CompatibilityProfile::VemapV45Parity && soil_depth > f(150.0) {
            eff_rock = F::ZERO;
        }

        // Organic matter % in top 20 cm (Spec 07 Delta #2).
        let ompc = match profile {
            CompatibilityProfile::VemapV45Parity => F::ZERO,
            CompatibilityProfile::ScientificClean => {
                // somsc (g C/m2) * 1.724 g OM/g C / (bulkd g/cm3 * 20 cm * 1e4 cm2/m2) * 100%
                let denom = (cell.bulkd * f(2000.0)).max(f(1.0));
                (initial_somsc * f(1.724) / denom).clamp_val(F::ZERO, f(20.0))
            }
        };

        let sand_pct = cell.sand * f(100.0);
        let silt_pct = cell.silt * f(100.0);
        let clay_pct = cell.clay * f(100.0);

        let mut afiel = site.afiel;
        let mut awilt = site.awilt;

        // Official CENTURY V4 SWFLAG codes (Manual p. 168):
        // 0 = explicit site.100, 1 = G&L (-0.33/-15), 2 = G&L (-0.10/-15),
        // 3 = Rawls (-0.33/-15), 4 = Rawls (-0.10/-15), 5/6 = Rawls afiel + explicit awilt.
        // VEMAP V4.5 gridded runs always use Rawls (-0.10/-15) with effective rock fraction.
        let eff_swflag = match profile {
            CompatibilityProfile::VemapV45Parity => 4,
            CompatibilityProfile::ScientificClean => site.swflag,
        };

        let rock_scale = F::ONE - eff_rock;
        match eff_swflag {
            1 => {
                // Gupta & Larson (1979): -0.33 bar FC, -15 bar WP.
                let sw4 = f(0.003075) * sand_pct
                    + f(0.005886) * silt_pct
                    + f(0.008039) * clay_pct
                    + f(0.002208) * ompc
                    - f(0.1434) * cell.bulkd;
                let sw15 = f(-0.000059) * sand_pct
                    + f(0.001142) * silt_pct
                    + f(0.005766) * clay_pct
                    + f(0.002228) * ompc
                    + f(0.02671) * cell.bulkd;
                for i in 0..nlayer {
                    afiel[i] = (sw4 * rock_scale).clamp_val(f(0.02), f(0.85));
                    awilt[i] = (sw15 * rock_scale).clamp_val(f(0.01), afiel[i] - f(0.005));
                }
            }
            2 => {
                // Gupta & Larson (1979): -0.10 bar FC, -15 bar WP.
                let sw4 = f(0.005018) * sand_pct
                    + f(0.008548) * silt_pct
                    + f(0.008833) * clay_pct
                    + f(0.004966) * ompc
                    - f(0.2423) * cell.bulkd;
                let sw15 = f(-0.000059) * sand_pct
                    + f(0.001142) * silt_pct
                    + f(0.005766) * clay_pct
                    + f(0.002228) * ompc
                    + f(0.02671) * cell.bulkd;
                for i in 0..nlayer {
                    afiel[i] = (sw4 * rock_scale).clamp_val(f(0.02), f(0.85));
                    awilt[i] = (sw15 * rock_scale).clamp_val(f(0.01), afiel[i] - f(0.005));
                }
            }
            3 => {
                // Rawls et al. (1982): -0.33 bar FC, -15 bar WP.
                let sw4 =
                    f(0.2576) - f(0.0020) * sand_pct + f(0.0036) * clay_pct + f(0.0299) * ompc;
                let sw15 = f(0.0260) + f(0.0050) * clay_pct + f(0.0158) * ompc;
                for i in 0..nlayer {
                    afiel[i] = (sw4 * rock_scale).clamp_val(f(0.02), f(0.85));
                    awilt[i] = (sw15 * rock_scale).clamp_val(f(0.01), afiel[i] - f(0.005));
                }
            }
            4 => {
                // Rawls et al. (1982): -0.10 bar FC, -15 bar WP (VEMAP standard).
                let sw4 =
                    f(0.4118) - f(0.0030) * sand_pct + f(0.0023) * clay_pct + f(0.0317) * ompc;
                let sw15 = f(0.0260) + f(0.0050) * clay_pct + f(0.0158) * ompc;
                for i in 0..nlayer {
                    afiel[i] = (sw4 * rock_scale).clamp_val(f(0.02), f(0.85));
                    awilt[i] = (sw15 * rock_scale).clamp_val(f(0.01), afiel[i] - f(0.005));
                }
            }
            5 => {
                let sw4 =
                    f(0.2576) - f(0.0020) * sand_pct + f(0.0036) * clay_pct + f(0.0299) * ompc;
                for i in 0..nlayer {
                    afiel[i] = (sw4 * rock_scale).clamp_val(f(0.02), f(0.85));
                }
            }
            6 => {
                let sw4 =
                    f(0.4118) - f(0.0030) * sand_pct + f(0.0023) * clay_pct + f(0.0317) * ompc;
                for i in 0..nlayer {
                    afiel[i] = (sw4 * rock_scale).clamp_val(f(0.02), f(0.85));
                }
            }
            _ => {
                // SWFLAG = 0: keep explicit awilt/afiel from site.100.
            }
        }

        let mut awhc = [F::ZERO; MAX_LAYERS];
        let mut twhc_root = F::ZERO;
        let mut twhc_profile = F::ZERO;
        for i in 0..nlayer {
            awhc[i] = ((afiel[i] - awilt[i]) * fix.adep[i]).max(F::ZERO);
            twhc_profile += awhc[i];
            if i < nlaypg {
                twhc_root += awhc[i];
            }
        }

        // Texture-dependent SOM rates (Spec 03 §2).
        let eftext = (fix.peftxa + fix.peftxb * cell.sand).clamp_val(f(0.1), f(2.0));
        let p1co2_soil = (fix.p1co2a[1] + fix.p1co2b[1] * cell.sand).clamp_val(f(0.05), f(0.95));
        let fps1s3 = (fix.ps1s3[0] + fix.ps1s3[1] * cell.clay).clamp_val(F::ZERO, f(0.5));
        let fps2s3 = (fix.ps2s3[0] + fix.ps2s3[1] * cell.clay).clamp_val(F::ZERO, f(0.5));

        // Mineral leaching fractions (Manual §3.3 p. 3-6, A2-9):
        // fleach_base = fleach[0] + fleach[1] * sand; element multipliers are fleach[2]=N, [3]=P, [4]=S.
        let fleach_base = (fix.fleach[0] + fix.fleach[1] * cell.sand).clamp_val(F::ZERO, F::ONE);
        let fleach_n = (fleach_base * fix.fleach[2]).clamp_val(F::ZERO, F::ONE);
        let fleach_p = (fleach_base * fix.fleach[3]).clamp_val(F::ZERO, F::ONE);
        let fleach_s = (fleach_base * fix.fleach[4]).clamp_val(F::ZERO, F::ONE);
        let omlech_frac = (fix.omlech[0] + fix.omlech[1] * cell.sand).clamp_val(F::ZERO, f(0.5));

        // Per-cell P geochemistry texture/pH factors (without mutating FixParams!).
        let texesp_2 = (fix.texesp[0] + fix.texesp[2] * cell.clay).clamp_val(f(0.1), f(10.0));
        let ph_factor = (F::ONE + f(0.1) * (cell.ph - f(6.5))).clamp_val(f(0.2), f(3.0));
        let pparmn_2 = (fix.pparmn[1] * ph_factor).max(F::ZERO);
        let psecmn_2 = (fix.psecmn[1] * ph_factor).max(F::ZERO);

        // Annual mean temperature range for Linacre (1977) PET (coldest month floored at -10 °C).
        let mut tmin_yr = annual_mean_temps[0];
        let mut tmax_yr = annual_mean_temps[0];
        for &t in annual_mean_temps.iter().skip(1) {
            if t < tmin_yr {
                tmin_yr = t;
            }
            if t > tmax_yr {
                tmax_yr = t;
            }
        }
        let trange_annual = (tmax_yr - tmin_yr.max(f(-10.0))).max(F::ZERO);

        Self {
            lat: cell.lat,
            lon: cell.lon,
            elevation: cell.elevation,
            sand: cell.sand,
            silt: cell.silt,
            clay: cell.clay,
            bulkd: cell.bulkd,
            ph: cell.ph,
            drain: cell.drain,
            basef: site.basef,
            stormf: site.stormf,
            nlayer,
            nlaypg,
            adep: fix.adep,
            effective_rockfrac: eff_rock,
            soil_depth_cm: soil_depth,
            afiel,
            awilt,
            awhc,
            twhc_root,
            twhc_profile,
            eftext,
            p1co2_soil,
            fps1s3,
            fps2s3,
            fleach_n,
            fleach_p,
            fleach_s,
            omlech_frac,
            texesp_2,
            pparmn_2,
            psecmn_2,
            trange_annual,
        }
    }
}
