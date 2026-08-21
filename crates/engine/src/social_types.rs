//! Social-system types extracted from `engine.rs`.
//!
//! Covers mood snapshots, stratification bands/events/quantiles, cohesion
//! (kinship, fabric), and unrest types used across the civic simulation
//! phases (FR-CIV-GOV-100, FR-CIV-GOV-020, FR-CIV-GOV-030, FR-CIV-UNREST-001).

use serde::{Deserialize, Serialize};

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

fn institution_kind_key(kind: civ_institutions::InstitutionKind) -> u8 {
    match kind {
        civ_institutions::InstitutionKind::Temple => 1,
        civ_institutions::InstitutionKind::Garrison => 2,
    }
}

/// Per-settlement mood history cap (FR-CIV-GOV-100). The engine keeps at
/// most this many [`MoodSnapshot`] entries per settlement in
/// `Simulation::mood_history_by_settlement`, plus [`MOOD_HISTORY_CAP`] * 8
/// entries in the flat `Simulation::mood_history` ring buffer (test
/// convenience).
pub const MOOD_HISTORY_CAP: usize = 16;

// ---------------------------------------------------------------------------
// Phase 3: phase_stratification types (FR-CIV-GOV-020)
// ---------------------------------------------------------------------------

/// Seed wrapper used by the stratification tests. Wraps a `u64` so the seed
/// surface matches `Simulation::with_seed(seed: u64)` while keeping the call
/// site readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimSeed(pub u64);

impl SimSeed {
    /// Convert a `u64` seed into a `SimSeed`.
    pub const fn from_u64(seed: u64) -> Self {
        SimSeed(seed)
    }
}

impl From<SimSeed> for u64 {
    fn from(seed: SimSeed) -> Self {
        seed.0
    }
}

impl From<u64> for SimSeed {
    fn from(seed: u64) -> Self {
        SimSeed(seed)
    }
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

/// Computes the Gini coefficient (0.0 = perfect equality, 1.0 = one household
/// owns everything) for a slice of household wealth values. Non-finite
/// results are clamped to `0.0` to satisfy the no-NaN/Inf project policy.
fn compute_gini(wealths: &[i64]) -> f64 {
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
// FR-CIV-GOV-030 (Phase A4) — cohesion types (kinship, trust, fabric)
// ---------------------------------------------------------------------------

/// Kinship relation between two actors in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KinshipEdge {
    pub kind: KinshipKind,
    pub target: u64,
}

/// A change in an actor's social-fabric metric produced by
/// [`Simulation::phase_cohesion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CohesionEventKind {
    Bonded,
    Strengthened,
    Weakened,
    Fragmented,
}

/// One actor's per-tick cohesion result.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CohesionEvent {
    pub actor_id: u64,
    pub settlement_id: u32,
    pub kind: CohesionEventKind,
    pub score: i64,
    pub score_delta: i64,
    pub fabric: FabricTier,
}

/// The social-fabric tier of a settlement, derived from its aggregate cohesion score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CohesionSnapshot {
    pub settlement_id: u32,
    pub fabric: FabricTier,
    pub kin_count: u64,
    pub trust_sum: i64,
    pub fragmentation_events: u32,
    pub fragmentations: u64,
    pub faction_count: u64,
}

// ---------------------------------------------------------------------------
// FR-CIV-UNREST-001 (Phase A5) — unrest types
// ---------------------------------------------------------------------------

/// The unrest level of a settlement.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UnrestSnapshot {
    pub settlement_id: u32,
    pub level: UnrestLevel,
    pub score: i32,
    pub events_count: u32,
    pub riots_count: u64,
    pub migrants_count: u64,
    pub mob_size: u64,
}
