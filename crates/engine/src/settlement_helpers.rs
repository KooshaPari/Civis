//! Settlement helpers extracted from engine.rs for modularity.
//!
//! Emergent settlement analysis, diplomacy pair selection, trade route management,
//! and related helper functions.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::fixed_math::Fixed;
use crate::engine::{
    ClusterStocks, EconomicFocus, KinshipEdge, FabricTier, Resources, ResourceType,
    TradeRoute, WorldState, DiplomacySignal,
};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Lower saturation bound for the per-settlement `mood` total in
/// [`MoodSnapshot`] (FR-CIV-GOV-100). Chosen symmetric with [`MOOD_MAX`]
/// so the score stays balanced for tests asserting `|mood| <= 200`.
pub const MOOD_MIN: i64 = -200;

/// Upper saturation bound for the per-settlement `mood` total in
/// [`MoodSnapshot`] (FR-CIV-GOV-100). Matches the documentation in
/// [`Simulation::phase_social_mood`].
pub const MOOD_MAX: i64 = 200;

/// `crime_score` baseline used by [`Simulation::phase_social_mood`]
/// (FR-CIV-GOV-100). Crime at `MOOD_CRIME_BASE / 4` (i.e. `75`) saturates
/// `crime_score` to `0`; lower crime gives a linearly higher score.
pub const MOOD_CRIME_BASE: i64 = 300;

/// Per-settlement mood history cap (FR-CIV-GOV-100). The engine keeps at
/// most this many [`MoodSnapshot`] entries per settlement in
/// `Simulation::mood_history_by_settlement`, plus [`MOOD_HISTORY_CAP`] * 8
/// entries in the flat `Simulation::mood_history` ring buffer (test
/// convenience).
pub const MOOD_HISTORY_CAP: usize = 16;

// ============================================================================
// TYPES & STRUCTS
// ============================================================================

/// Per-cluster stockpiles keyed by emergent settlement id.
pub type ClusterStocks = civ_economy::Stocks;

/// Broad economic orientation inferred from a civilization's strongest signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EconomicFocus {
    Balanced,
    Agrarian,
    Industrial,
    Sacred,
    Mercantile,
}

/// A per-settlement event emitted when the expected economic focus changes
/// (FR-CIV-ECON-001 / ADR-020). Carries the settlement id, the previous and
/// proposed focus, and a human-readable cause so downstream phases and the
/// JSON-RPC bridge can attribute the transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicFocusEvent {
    pub settlement_id: u32,
    pub from: EconomicFocus,
    pub to: EconomicFocus,
    pub cause: String,
}

/// Per-settlement mood snapshot emitted by [`Simulation::phase_social_mood`]
/// (FR-CIV-GOV-100 family). Carries the total mood plus the four contributing
/// sub-scores and the institution bonus, so downstream phases and the
/// JSON-RPC bridge (`sim.snapshot.mood`) can attribute changes to specific
/// drivers. `Copy` because the snapshot is pushed to a flat `Vec` for
/// determinism testing and the `mood_history` ring is updated in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodSnapshot {
    /// Settlement this snapshot pertains to.
    pub settlement_id: u32,
    /// Total mood after summing the sub-scores and institution bonuses
    /// (saturated to [`MOOD_MIN`, `MOOD_MAX`]).
    pub mood: i64,
    /// Delta versus the previous tick's mood for this settlement
    /// (`0` if no prior snapshot was recorded).
    pub mood_delta: i64,
    /// `clamp(stocked / 200, MOOD_MIN, MOOD_MAX)` — food surplus contribution.
    pub food_score: i64,
    /// `clamp(2 * (capacity - population), MOOD_MIN, MOOD_MAX)` —
    /// housing surplus / deficit contribution.
    pub housing_score: i64,
    /// `max(0, MOOD_CRIME_BASE - 4 * crime_pressure)` — crime inverse
    /// contribution (clipped to `[0, MOOD_CRIME_BASE]`).
    pub crime_score: i64,
    /// `25 + 25 * level` when a Temple is present, else `0`.
    pub temple_bonus: i32,
    /// `15 + 15 * level` when a Garrison is present, else `0`.
    pub garrison_bonus: i32,
}

/// Stratification band assigned to a household based on wealth + power score.
///
/// Ordered from lowest to highest: Poor < Middle < Rich < Elite. The numeric
/// rank returned by [`StratBand::rank`] is used for promotion/demotion
/// detection in `phase_stratification`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StratBand {
    Poor,
    Middle,
    Rich,
    Elite,
}

impl StratBand {
    /// Numeric rank 0..=3 (Poor=0, Middle=1, Rich=2, Elite=3).
    pub const fn rank(self) -> u8 {
        match self {
            StratBand::Poor => 0,
            StratBand::Middle => 1,
            StratBand::Rich => 2,
            StratBand::Elite => 3,
        }
    }

    /// Promote (or demote) by `delta` ranks, clamping to `[Poor, Elite]`.
    pub fn shift(self, delta: i32) -> Self {
        let new_rank = (self.rank() as i32 + delta).clamp(0, 3) as u8;
        match new_rank {
            0 => StratBand::Poor,
            1 => StratBand::Middle,
            2 => StratBand::Rich,
            _ => StratBand::Elite,
        }
    }
}

/// Per-household stratification event emitted each tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StratificationEvent {
    pub household_id: u64,
    pub kind: StratificationEventKind,
    pub band: StratBand,
    pub score: i64,
    pub score_delta: i64,
}

/// Kind of stratification change detected for a household.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StratificationEventKind {
    /// Household moved into a higher band than last tick.
    Promoted,
    /// Household moved into a lower band than last tick.
    Demoted,
    /// Household remained in the same band as last tick.
    Unchanged,
}

/// Per-tick quantiles computed from household wealth within a settlement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StratQuantiles {
    pub poor: u32,
    pub middle: u32,
    pub rich: u32,
    pub elite: u32,
}

impl StratQuantiles {
    /// Empty quantiles (all bands have 0 wealth sum). Used when a settlement
    /// has no households yet.
    pub fn empty() -> Self {
        StratQuantiles {
            poor: 0,
            middle: 0,
            rich: 0,
            elite: 0,
        }
    }

    /// Accumulate `wealth` into the appropriate band based on the Gini-style
    /// thresholds used in `phase_stratification`.
    pub fn add(&mut self, band: StratBand) {
        match band {
            StratBand::Poor => self.poor = self.poor.saturating_add(1),
            StratBand::Middle => self.middle = self.middle.saturating_add(1),
            StratBand::Rich => self.rich = self.rich.saturating_add(1),
            StratBand::Elite => self.elite = self.elite.saturating_add(1),
        }
    }
}

impl Default for StratQuantiles {
    fn default() -> Self {
        Self::empty()
    }
}

/// Per-settlement stratification report at end of tick.
#[derive(Debug, Clone, PartialEq)]
pub struct StratificationReport {
    pub settlement_id: u32,
    pub quantiles: StratQuantiles,
    pub gini: f64,
    pub class_mobility_count: u32,
    pub tick: u64,
}

/// Kinship relation between two actors in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KinshipKind {
    Family,
    Clan,
    Tribe,
    Guild,
    Oath,
}

impl KinshipKind {
    /// Base weight that this kinship kind contributes to fabric.
    /// Family = 40, Clan = 25, Tribe = 15, Guild = 10, Oath = 5.
    pub fn weight(self) -> i64 {
        match self {
            KinshipKind::Family => 40,
            KinshipKind::Clan => 25,
            KinshipKind::Tribe => 15,
            KinshipKind::Guild => 10,
            KinshipKind::Oath => 5,
        }
    }
}

/// A directed edge carrying one actor's kinship to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KinshipEdge {
    pub kind: KinshipKind,
    pub target: u64,
}

/// A change in an actor's social-fabric metric produced by
/// [`Simulation::phase_cohesion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CohesionEventKind {
    Bonded,
    Strengthened,
    Weakened,
    Fragmented,
}

/// One actor's per-tick cohesion result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CohesionEvent {
    pub actor_id: u64,
    pub settlement_id: u32,
    pub kind: CohesionEventKind,
    pub score: i64,
    pub score_delta: i64,
    pub fabric: FabricTier,
}

/// The social-fabric tier of a settlement, derived from its aggregate cohesion score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FabricTier {
    Tight,
    Loosened,
    Strained,
    Fractured,
}

impl FabricTier {
    /// Classify a raw fabric score into a tier.
    pub fn from_score(score: i64) -> Self {
        if score >= 80 {
            FabricTier::Tight
        } else if score >= 40 {
            FabricTier::Loosened
        } else if score >= 10 {
            FabricTier::Strained
        } else {
            FabricTier::Fractured
        }
    }
}

/// Per-settlement cohesion summary produced every tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CohesionSnapshot {
    pub settlement_id: u32,
    pub fabric: FabricTier,
    pub kin_count: u64,
    pub trust_sum: i64,
    pub fragmentation_events: u32,
    pub fragmentations: u64,
    pub faction_count: u64,
}

/// The unrest level of a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnrestLevel {
    Stable,
    Restless,
    Rioting,
    Revolting,
}

impl UnrestLevel {
    /// Classify a raw unrest score (0-500) into a level.
    pub fn from_score(score: i32) -> Self {
        if score < 50 {
            UnrestLevel::Stable
        } else if score < 150 {
            UnrestLevel::Restless
        } else if score < 300 {
            UnrestLevel::Rioting
        } else {
            UnrestLevel::Revolting
        }
    }

    pub const fn to_rank(self) -> u8 {
        match self {
            UnrestLevel::Stable => 0,
            UnrestLevel::Restless => 1,
            UnrestLevel::Rioting => 2,
            UnrestLevel::Revolting => 3,
        }
    }
}

/// A per-tick event emitted when a settlement's unrest state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrestEvent {
    pub settlement_id: u32,
    pub level: UnrestLevel,
    pub score: i32,
    pub score_delta: i32,
    pub mood: i32,
    pub gini_x100: i32,
    pub fabric: FabricTier,
}

/// Per-settlement unrest snapshot for the last tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnrestSnapshot {
    pub settlement_id: u32,
    pub level: UnrestLevel,
    pub score: i32,
    pub events_count: u32,
    pub riots_count: u64,
    pub migrants_count: u64,
    pub mob_size: u64,
}

/// Per-settlement religious event emitted by [`Simulation::phase_belief`]
/// (FR-CIV-REL-001 §7 + §10 hooks). Consumed by the JSON-RPC bridge in
/// `crates/server/src/ws_bridge.rs` to surface the religion layer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReligionEvent {
    /// The settlement id this event pertains to.
    pub settlement_id: u32,
    /// Which cap was hit (or None if this is a regular profile update).
    pub kind: ReligionEventKind,
    /// The tick on which this event was emitted.
    pub tick: u64,
}

/// Distinguishes the kinds of religion events the `phase_belief` loop can
/// emit. JSON-RPC consumers use this to decide whether to surface a UI
/// notification (caps hit) or just update the profile (regular).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReligionEventKind {
    /// Regular per-tick profile update (no cap hit, no regime change).
    TickUpdate,
    /// `monitoring` deltas hit [`crate::religion::MAX_D_MONITORING_PER_TICK`].
    MonitoringCapped,
    /// `mythic_coherence` deltas hit [`crate::religion::MAX_D_COHERENCE_PER_TICK`].
    CoherenceCapped,
    /// `uncertainty_reduction` deltas hit [`crate::religion::MAX_D_UNCERT_REDUCTION_TICK`].
    UncertaintyCapped,
    /// The profile crossed the Norenzayan Big-Gods threshold upward.
    BigGodsEmerged,
    /// The profile collapsed below the dissolution threshold.
    Dissolved,
}

/// Civic institution event emitted by [`Simulation::phase_institutions`]
/// (FR-CIV-GOV-001/002/003). Consumed by the JSON-RPC bridge in
/// `crates/server/src/ws_bridge.rs` to surface the civil layer to clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionEvent {
    /// The kind of institution that changed.
    pub kind: civ_institutions::InstitutionKind,
    /// The new level (1 = L1 / first spawn, 2 = L2 / first upgrade, ...).
    pub level: u8,
    /// The settlement id this event pertains to.
    pub settlement_id: u32,
}

// ============================================================================
// FREE FUNCTIONS
// ============================================================================

fn institution_kind_key(kind: civ_institutions::InstitutionKind) -> u8 {
    match kind {
        civ_institutions::InstitutionKind::Temple => 1,
        civ_institutions::InstitutionKind::Garrison => 2,
    }
}

/// Computes the Gini coefficient (0.0 = perfect equality, 1.0 = one household
/// owns everything) for a slice of household wealth values. Non-finite
/// results are clamped to `0.0` to satisfy the no-NaN/Inf project policy.
pub fn compute_gini(wealths: &[i64]) -> f64 {
    if wealths.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<i64> = wealths.iter().copied().filter(|w| *w >= 0).collect();
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.sort_unstable();
    let n = sorted.len() as f64;
    let sum: f64 = sorted.iter().map(|w| *w as f64).sum();
    if sum <= 0.0 {
        return 0.0;
    }
    let mut cumulative = 0.0_f64;
    let mut weighted_sum = 0.0_f64;
    for (i, w) in sorted.iter().enumerate() {
        cumulative += *w as f64;
        // Lorenz curve: cumulative wealth / total wealth; index i+1 out of n.
        weighted_sum += (i as f64 + 1.0) * (*w as f64);
    }
    let gini = (2.0 * weighted_sum) / (n * sum) - (n + 1.0) / n;
    if gini.is_finite() {
        gini.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Compatibility stubs (FR-COMPAT)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CompatState {
    faction_count: u32,
    cohesion_events: Vec<CohesionEvent>,
    unrest_events: Vec<UnrestEvent>,
    unrest_levels: BTreeMap<u32, UnrestLevel>,
    settlement_gini: BTreeMap<u32, f64>,
}

fn compat_state() -> &'static Mutex<CompatState> {
    static STATE: OnceLock<Mutex<CompatState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(CompatState::default()))
}

/// Get last tick's cohesion for a settlement (currently empty stub).
pub fn last_tick_cohesion_settlement(settlement_id: u32) -> &'static [CohesionEvent] {
    let events: Vec<CohesionEvent> = compat_state()
        .lock()
        .expect("compat state poisoned")
        .cohesion_events
        .iter()
        .filter(|event| event.settlement_id == settlement_id)
        .cloned()
        .collect();
    Box::leak(events.into_boxed_slice())
}

/// Get last tick's unrest events (currently empty stub).
pub fn last_tick_unrest() -> &'static [UnrestEvent] {
    Box::leak(
        compat_state()
            .lock()
            .expect("compat state poisoned")
            .unrest_events
            .clone()
            .into_boxed_slice(),
    )
}

/// Get last tick's unrest for a settlement (currently empty stub).
pub fn last_tick_unrest_settlement(settlement_id: u32) -> &'static [UnrestEvent] {
    let events: Vec<UnrestEvent> = compat_state()
        .lock()
        .expect("compat state poisoned")
        .unrest_events
        .iter()
        .filter(|event| event.settlement_id == settlement_id)
        .cloned()
        .collect();
    Box::leak(events.into_boxed_slice())
}

/// Set settlement gini coefficient (currently a no-op stub).
pub fn set_settlement_gini(settlement_id: u32, gini: f64) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    let normalized = if gini.is_nan() {
        0.0
    } else {
        gini.clamp(0.0, 1.0)
    };
    state.settlement_gini.insert(settlement_id, normalized);
    let score = (normalized * 200.0).round() as i32;
    let level = UnrestLevel::from_score(score);
    state.unrest_levels.insert(settlement_id, level);
    state.unrest_events.push(UnrestEvent {
        settlement_id,
        level,
        score,
        score_delta: 0,
        mood: 0,
        gini_x100: (normalized * 100.0).round() as i32,
        fabric: FabricTier::Fractured,
    });
}

/// Get the normalized Gini coefficient stored for a settlement.
pub fn settlement_gini(settlement_id: u32) -> Option<f64> {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .settlement_gini
        .get(&settlement_id)
        .copied()
}

/// Get unrest level for a settlement (currently None stub).
pub fn unrest_level(settlement_id: u32) -> Option<UnrestLevel> {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .unrest_levels
        .get(&settlement_id)
        .copied()
}
