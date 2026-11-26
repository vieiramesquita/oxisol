//! Stack-allocated, strongly-typed `FlowLedger<F>` replacing raw-pointer global
//! flow buffers with compile-time pool safety and exact mass conservation auditing.

use crate::float::Float;
use crate::params::{MAX_ELEM, MAX_LAYERS};
use crate::state::{CarbonPool, CellState};

/// Maximum inline stack-queued flows per commit batch (zero heap allocation in normal steps).
/// Any overflow beyond this threshold transparently spills to a heap buffer so flows are
/// NEVER silently dropped even under extreme multi-event bursts.
pub const MAX_LEDGER_FLOWS: usize = 128;

/// Strongly-typed identifier for every carbon pool, nutrient pool, and system boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolId {
    // --- Carbon Pools ---
    /// Surface (`0`) or soil (`1`) structural litter C.
    StrucC(usize),
    /// Surface (`0`) or soil (`1`) metabolic litter C.
    MetabC(usize),
    /// Surface (`0`) or soil (`1`) active SOM C.
    Som1C(usize),
    /// Slow SOM C.
    Som2C,
    /// Passive SOM C.
    Som3C,
    /// Herbaceous live shoots C.
    AglivC,
    /// Herbaceous live roots C.
    BglivC,
    /// Standing dead shoots C.
    StdedC,
    /// Forest live leaves C.
    RleavC,
    /// Forest live fine roots C.
    FrootC,
    /// Forest live fine branches C.
    FbrchC,
    /// Forest live large wood C.
    RlwodC,
    /// Forest live coarse roots C.
    CrootC,
    /// Dead fine branches C (`wood1`).
    Wood1C,
    /// Dead large wood C (`wood2`).
    Wood2C,
    /// Dead coarse roots C (`wood3`).
    Wood3C,

    // --- Nutrient Pools (element index: 0=N, 1=P, 2=S) ---
    /// Surface (`0`) or soil (`1`) structural litter element `e`.
    StrucE(usize, usize),
    /// Surface (`0`) or soil (`1`) metabolic litter element `e`.
    MetabE(usize, usize),
    /// Surface (`0`) or soil (`1`) active SOM element `e`.
    Som1E(usize, usize),
    /// Slow SOM element `e`.
    Som2E(usize),
    /// Passive SOM element `e`.
    Som3E(usize),
    /// Herbaceous live shoots element `e`.
    AglivE(usize),
    /// Herbaceous live roots element `e`.
    BglivE(usize),
    /// Standing dead element `e`.
    StdedE(usize),
    /// Crop internal storage element `e`.
    CrpStg(usize),
    /// Forest live leaves element `e`.
    RleavE(usize),
    /// Forest fine roots element `e`.
    FrootE(usize),
    /// Forest fine branches element `e`.
    FbrchE(usize),
    /// Forest large wood element `e`.
    RlwodE(usize),
    /// Forest coarse roots element `e`.
    CrootE(usize),
    /// Forest internal storage element `e`.
    ForStg(usize),
    /// Dead wood1 element `e`.
    Wood1E(usize),
    /// Dead wood2 element `e`.
    Wood2E(usize),
    /// Dead wood3 element `e`.
    Wood3E(usize),
    /// Layered soil mineral nutrient `(layer, elem)`.
    Minerl(usize, usize),
    /// Parent mineral material `(elem)`.
    Parent(usize),
    /// Secondary sorbed mineral `(elem)`.
    Secndy(usize),
    /// Occluded inorganic phosphorus.
    OccludP,

    // --- Explicit System Boundary Sources & Sinks ---
    /// Atmospheric CO2 source/sink (photosynthesis & heterotrophic respiration).
    AtmosphereC,
    /// Atmospheric nutrient input/volatilization (deposition, N fixation, volatilization).
    AtmosphereE(usize),
    /// External fertilizer / irrigation / OMAD carbon input.
    ExternalInputC,
    /// External fertilizer / irrigation / OMAD nutrient input.
    ExternalInputE(usize),
    /// Export via solute/organic leaching.
    LeachingC,
    /// Export via mineral solute leaching.
    LeachingE(usize),
    /// Export via harvest, grazing, fire, or erosion (C).
    ExportC,
    /// Export via harvest, grazing, fire, or erosion (element `e`).
    ExportE(usize),
}

impl PoolId {
    /// Returns true if this pool represents a carbon compartment or C boundary.
    #[inline(always)]
    pub const fn is_carbon(self) -> bool {
        matches!(
            self,
            Self::StrucC(_)
                | Self::MetabC(_)
                | Self::Som1C(_)
                | Self::Som2C
                | Self::Som3C
                | Self::AglivC
                | Self::BglivC
                | Self::StdedC
                | Self::RleavC
                | Self::FrootC
                | Self::FbrchC
                | Self::RlwodC
                | Self::CrootC
                | Self::Wood1C
                | Self::Wood2C
                | Self::Wood3C
                | Self::AtmosphereC
                | Self::ExternalInputC
                | Self::LeachingC
                | Self::ExportC
        )
    }

    /// Returns true if this pool is an external system boundary (outside ecosystem totals).
    #[inline(always)]
    pub const fn is_boundary(self) -> bool {
        matches!(
            self,
            Self::AtmosphereC
                | Self::AtmosphereE(_)
                | Self::ExternalInputC
                | Self::ExternalInputE(_)
                | Self::LeachingC
                | Self::LeachingE(_)
                | Self::ExportC
                | Self::ExportE(_)
        )
    }
}

/// Single queued mass transfer from `from` to `to`.
#[derive(Debug, Clone, Copy)]
pub struct Flow<F: Float> {
    /// Source pool.
    pub from: PoolId,
    /// Destination pool.
    pub to: PoolId,
    /// Mass to transfer ($\text{g}\cdot\text{m}^{-2}$, non-negative).
    pub amount: F,
    /// Optional explicit labeled isotope fraction (used for boundary C inputs).
    pub lab_frac: Option<F>,
}

/// Mass conservation diagnostic report for one step or full simulation.
#[derive(Debug, Clone, Copy)]
pub struct MassBalanceReport<F: Float> {
    /// Carbon balance error $|\Delta C - (C_{\text{in}} - C_{\text{out}})|$.
    pub carbon_err: F,
    /// Element balance errors `[N, P, S]`.
    pub elem_err: [F; MAX_ELEM],
}

impl<F: Float> MassBalanceReport<F> {
    /// Returns true if all active element errors are within `tol`.
    pub fn is_conserved(&self, nelem: usize, tol: F) -> bool {
        if self.carbon_err > tol || !self.carbon_err.is_finite_val() {
            return false;
        }
        for e in 0..nelem.min(MAX_ELEM) {
            if self.elem_err[e] > tol || !self.elem_err[e].is_finite_val() {
                return false;
            }
        }
        true
    }
}

/// Stack-allocated transfer ledger guaranteeing exact conservation and zero heap allocation
/// in normal steps, with a transparent overflow spill buffer so flows are NEVER dropped.
pub struct FlowLedger<F: Float> {
    flows: [Flow<F>; MAX_LEDGER_FLOWS],
    len: usize,
    spill: Vec<Flow<F>>,
}

impl<F: Float> Default for FlowLedger<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Float> FlowLedger<F> {
    /// Create an empty stack-allocated ledger (zero heap allocation).
    #[inline]
    pub fn new() -> Self {
        Self {
            flows: [Flow {
                from: PoolId::AtmosphereC,
                to: PoolId::AtmosphereC,
                amount: F::ZERO,
                lab_frac: None,
            }; MAX_LEDGER_FLOWS],
            len: 0,
            spill: Vec::new(),
        }
    }

    /// Clear queued flows without deallocating.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.spill.clear();
    }

    /// Number of queued flows (inline + overflow spill).
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len + self.spill.len()
    }

    /// Returns true if no flows are queued.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0 && self.spill.is_empty()
    }

    /// Schedule a transfer `amount` from `from` to `to`.
    /// Ignores zero, non-positive, or non-finite amounts.
    #[inline]
    pub fn schedule(&mut self, from: PoolId, to: PoolId, amount: F) {
        self.schedule_with_iso(from, to, amount, None);
    }

    /// Schedule a carbon transfer with explicit labeled fraction (for boundary C inputs).
    #[inline]
    pub fn schedule_with_iso(&mut self, from: PoolId, to: PoolId, amount: F, lab_frac: Option<F>) {
        if amount <= F::ZERO || !amount.is_finite_val() || from == to {
            return;
        }
        let fl = Flow {
            from,
            to,
            amount,
            lab_frac,
        };
        if self.len < MAX_LEDGER_FLOWS {
            self.flows[self.len] = fl;
            self.len += 1;
        } else {
            self.spill.push(fl);
        }
    }

    /// Schedule and auto-commit if inline stack buffer is nearly full.
    #[inline]
    pub fn schedule_safe(&mut self, state: &mut CellState<F>, from: PoolId, to: PoolId, amount: F) {
        if self.len >= MAX_LEDGER_FLOWS - 2 {
            self.commit(state);
        }
        self.schedule(from, to, amount);
    }

    /// Atomically apply all queued flows (inline + any spill) to `state`, enforcing pool
    /// non-negativity, preserving isotope ratios, and updating boundary conservation counters.
    #[inline]
    pub fn commit(&mut self, state: &mut CellState<F>) {
        let count = self.len;
        if count == 0 && self.spill.is_empty() {
            return;
        }
        self.len = 0;
        for i in 0..count {
            apply_one_flow(state, self.flows[i]);
        }
        if !self.spill.is_empty() {
            for i in 0..self.spill.len() {
                apply_one_flow(state, self.spill[i]);
            }
            self.spill.clear();
        }
    }
}

#[inline(always)]
fn apply_one_flow<F: Float>(state: &mut CellState<F>, fl: Flow<F>) {
    if fl.amount <= F::ZERO || !fl.amount.is_finite_val() {
        return;
    }
    if fl.from.is_carbon() {
        let (actual, lf) = take_carbon(state, fl.from, fl.amount, fl.lab_frac);
        if actual > F::ZERO {
            give_carbon(state, fl.to, actual, lf);
            if fl.from.is_boundary() && !fl.to.is_boundary() {
                state.cum_c_in += actual;
            } else if !fl.from.is_boundary() && fl.to.is_boundary() {
                state.cum_c_out += actual;
            }
        }
    } else {
        let actual = take_elem(state, fl.from, fl.amount);
        if actual > F::ZERO {
            give_elem(state, fl.to, actual);
            if let Some(e) = elem_index(fl.from).or_else(|| elem_index(fl.to)) {
                if fl.from.is_boundary() && !fl.to.is_boundary() {
                    state.cum_e_in[e] += actual;
                } else if !fl.from.is_boundary() && fl.to.is_boundary() {
                    state.cum_e_out[e] += actual;
                }
            }
        }
    }
}

/// Atomically transfer carbon from `src` to `dst` preserving isotope ratios.
/// Guaranteed mass-conserving (internal transfer; boundary counters unchanged).
#[inline(always)]
pub fn transfer_c_direct<F: Float>(
    src: &mut CarbonPool<F>,
    dst: &mut CarbonPool<F>,
    amount: F,
) -> F {
    if amount <= F::ZERO || !amount.is_finite_val() {
        return F::ZERO;
    }
    let (actual, lf) = src.sub_preserving_ratio(amount);
    if actual > F::ZERO {
        dst.add(actual, lf);
    }
    actual
}

/// Atomically export carbon from `src` to an external boundary (respiration or leaching),
/// updating `cum_c_out` identically to `FlowLedger::commit`.
#[inline(always)]
pub fn export_c_direct<F: Float>(src: &mut CarbonPool<F>, cum_c_out: &mut F, amount: F) -> F {
    if amount <= F::ZERO || !amount.is_finite_val() {
        return F::ZERO;
    }
    let (actual, _lf) = src.sub_preserving_ratio(amount);
    if actual > F::ZERO {
        *cum_c_out += actual;
    }
    actual
}

/// Atomically transfer a nutrient element between two internal pools (`src -> dst`),
/// clamping to pool non-negativity identically to `FlowLedger::commit`.
#[inline(always)]
pub fn transfer_e_direct<F: Float>(src: &mut F, dst: &mut F, amount: F) -> F {
    if amount <= F::ZERO || !amount.is_finite_val() {
        return F::ZERO;
    }
    let avail = (*src).max(F::ZERO);
    let actual = amount.min(avail);
    if actual > F::ZERO {
        *src = (*src - actual).max(F::ZERO);
        *dst += actual;
    }
    actual
}

/// Atomically export a nutrient element from `src` to an external boundary (volatilization or
/// leaching), updating `cum_e_out` identically to `FlowLedger::commit`.
#[inline(always)]
pub fn export_e_direct<F: Float>(src: &mut F, cum_e_out: &mut F, amount: F) -> F {
    if amount <= F::ZERO || !amount.is_finite_val() {
        return F::ZERO;
    }
    let avail = (*src).max(F::ZERO);
    let actual = amount.min(avail);
    if actual > F::ZERO {
        *src = (*src - actual).max(F::ZERO);
        *cum_e_out += actual;
    }
    actual
}

/// Compute exact mass-balance conservation error between two snapshots.
pub fn check_mass_balance<F: Float>(
    before_c: F,
    before_e: [F; MAX_ELEM],
    before_c_in: F,
    before_c_out: F,
    before_e_in: [F; MAX_ELEM],
    before_e_out: [F; MAX_ELEM],
    after: &CellState<F>,
) -> MassBalanceReport<F> {
    let dc = after.total_carbon() - before_c;
    let net_c = (after.cum_c_in - before_c_in) - (after.cum_c_out - before_c_out);
    let carbon_err = (dc - net_c).abs();

    let mut elem_err = [F::ZERO; MAX_ELEM];
    for e in 0..MAX_ELEM {
        let de = after.total_element(e) - before_e[e];
        let net_e = (after.cum_e_in[e] - before_e_in[e]) - (after.cum_e_out[e] - before_e_out[e]);
        elem_err[e] = (de - net_e).abs();
    }
    MassBalanceReport {
        carbon_err,
        elem_err,
    }
}

#[inline]
fn carbon_pool_mut<F: Float>(state: &mut CellState<F>, id: PoolId) -> Option<&mut CarbonPool<F>> {
    match id {
        PoolId::StrucC(i) => state.strucc.get_mut(i),
        PoolId::MetabC(i) => state.metabc.get_mut(i),
        PoolId::Som1C(i) => state.som1c.get_mut(i),
        PoolId::Som2C => Some(&mut state.som2c),
        PoolId::Som3C => Some(&mut state.som3c),
        PoolId::AglivC => Some(&mut state.aglivc),
        PoolId::BglivC => Some(&mut state.bglivc),
        PoolId::StdedC => Some(&mut state.stdedc),
        PoolId::RleavC => Some(&mut state.rleavc),
        PoolId::FrootC => Some(&mut state.frootc),
        PoolId::FbrchC => Some(&mut state.fbrchc),
        PoolId::RlwodC => Some(&mut state.rlwodc),
        PoolId::CrootC => Some(&mut state.crootc),
        PoolId::Wood1C => Some(&mut state.wood1c),
        PoolId::Wood2C => Some(&mut state.wood2c),
        PoolId::Wood3C => Some(&mut state.wood3c),
        _ => None,
    }
}

#[inline]
fn take_carbon<F: Float>(
    state: &mut CellState<F>,
    from: PoolId,
    amount: F,
    explicit_lf: Option<F>,
) -> (F, F) {
    if from.is_boundary() {
        let lf = explicit_lf.unwrap_or(state.atm_lab_frac);
        return (amount, lf);
    }
    if let Some(pool) = carbon_pool_mut(state, from) {
        pool.sub_preserving_ratio(amount)
    } else {
        (F::ZERO, F::ZERO)
    }
}

#[inline]
fn give_carbon<F: Float>(state: &mut CellState<F>, to: PoolId, amount: F, lf: F) {
    if to.is_boundary() {
        return;
    }
    if let Some(pool) = carbon_pool_mut(state, to) {
        pool.add(amount, lf);
    }
}

#[inline]
fn elem_pool_mut<F: Float>(state: &mut CellState<F>, id: PoolId) -> Option<&mut F> {
    match id {
        PoolId::StrucE(i, e) if i < 2 && e < MAX_ELEM => Some(&mut state.struce[i][e]),
        PoolId::MetabE(i, e) if i < 2 && e < MAX_ELEM => Some(&mut state.metabe[i][e]),
        PoolId::Som1E(i, e) if i < 2 && e < MAX_ELEM => Some(&mut state.som1e[i][e]),
        PoolId::Som2E(e) if e < MAX_ELEM => Some(&mut state.som2e[e]),
        PoolId::Som3E(e) if e < MAX_ELEM => Some(&mut state.som3e[e]),
        PoolId::AglivE(e) if e < MAX_ELEM => Some(&mut state.aglive[e]),
        PoolId::BglivE(e) if e < MAX_ELEM => Some(&mut state.bglive[e]),
        PoolId::StdedE(e) if e < MAX_ELEM => Some(&mut state.stdede[e]),
        PoolId::CrpStg(e) if e < MAX_ELEM => Some(&mut state.crpstg[e]),
        PoolId::RleavE(e) if e < MAX_ELEM => Some(&mut state.rleave[e]),
        PoolId::FrootE(e) if e < MAX_ELEM => Some(&mut state.froote[e]),
        PoolId::FbrchE(e) if e < MAX_ELEM => Some(&mut state.fbrche[e]),
        PoolId::RlwodE(e) if e < MAX_ELEM => Some(&mut state.rlwode[e]),
        PoolId::CrootE(e) if e < MAX_ELEM => Some(&mut state.croote[e]),
        PoolId::ForStg(e) if e < MAX_ELEM => Some(&mut state.forstg[e]),
        PoolId::Wood1E(e) if e < MAX_ELEM => Some(&mut state.wood1e[e]),
        PoolId::Wood2E(e) if e < MAX_ELEM => Some(&mut state.wood2e[e]),
        PoolId::Wood3E(e) if e < MAX_ELEM => Some(&mut state.wood3e[e]),
        PoolId::Minerl(l, e) if l < MAX_LAYERS && e < MAX_ELEM => Some(&mut state.minerl[l][e]),
        PoolId::Parent(e) if e < MAX_ELEM => Some(&mut state.parent[e]),
        PoolId::Secndy(e) if e < MAX_ELEM => Some(&mut state.secndy[e]),
        PoolId::OccludP => Some(&mut state.occlud),
        _ => None,
    }
}

#[inline]
fn elem_index(id: PoolId) -> Option<usize> {
    match id {
        PoolId::StrucE(_, e)
        | PoolId::MetabE(_, e)
        | PoolId::Som1E(_, e)
        | PoolId::Som2E(e)
        | PoolId::Som3E(e)
        | PoolId::AglivE(e)
        | PoolId::BglivE(e)
        | PoolId::StdedE(e)
        | PoolId::CrpStg(e)
        | PoolId::RleavE(e)
        | PoolId::FrootE(e)
        | PoolId::FbrchE(e)
        | PoolId::RlwodE(e)
        | PoolId::CrootE(e)
        | PoolId::ForStg(e)
        | PoolId::Wood1E(e)
        | PoolId::Wood2E(e)
        | PoolId::Wood3E(e)
        | PoolId::Minerl(_, e)
        | PoolId::Parent(e)
        | PoolId::Secndy(e)
        | PoolId::AtmosphereE(e)
        | PoolId::ExternalInputE(e)
        | PoolId::LeachingE(e)
        | PoolId::ExportE(e) => Some(e),
        PoolId::OccludP => Some(1),
        _ => None,
    }
}

#[inline]
fn take_elem<F: Float>(state: &mut CellState<F>, from: PoolId, amount: F) -> F {
    if from.is_boundary() {
        return amount;
    }
    if let Some(pool) = elem_pool_mut(state, from) {
        let actual = amount.min((*pool).max(F::ZERO));
        *pool = (*pool - actual).max(F::ZERO);
        actual
    } else {
        F::ZERO
    }
}

#[inline]
fn give_elem<F: Float>(state: &mut CellState<F>, to: PoolId, amount: F) {
    if to.is_boundary() {
        return;
    }
    if let Some(pool) = elem_pool_mut(state, to) {
        *pool += amount;
    }
}
