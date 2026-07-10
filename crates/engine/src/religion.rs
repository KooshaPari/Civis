// TODO(cleanup-surgeon): stub module — religion/Belief/Religion types were
// removed by an earlier cleanup lane. Downstream files (`lib.rs`,
// `engine.rs`) still hold `use crate::religion;` and `pub use religion::`
// imports. Restore the original implementation or rewrite the callers in
// the next pass. Keeping this as `pub mod` so the crate surface compiles.

use serde::{Deserialize, Serialize};

/// Maximum unrest signal that can be carried into a substrate gradient
/// (FR-CIV-BELIEF-001 §10.1). Anything above 1.0 saturates the gradient
/// component, so the constant is used as a hard ceiling.
pub const MAX_MISERY_UNREST: f32 = 1.0;

/// Maximum per-tick delta caps used by `phase_belief` when applying
/// the Norenzayan Big-Gods response curve. Stub values so the engine
/// compiles until the original constants are restored.
pub const MAX_D_MONITORING_PER_TICK: f32 = 0.05;
pub const MAX_D_COHERENCE_PER_TICK: f32 = 0.05;
pub const MAX_D_UNCERT_REDUCTION_TICK: f32 = 0.05;

/// Substrate gradient snapshot for one settlement (FR-CIV-BELIEF-001 §7).
/// Each field is in the [0.0, 1.0] range after clamping.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SubstrateGradients {
    /// Hardship gradient (drought, siege, war, etc.).
    pub grad_T: f32,
    /// Material scarcity gradient.
    pub grad_M: f32,
    /// Belief/mythic pressure gradient.
    pub grad_B: f32,
    /// Density of kinship edges per settlement.
    pub kinship_density: f32,
    /// Aggregated unrest score (0..MAX_MISERY_UNREST).
    pub unrest: f32,
    /// Migration rate into/out of the settlement.
    pub migration_rate: f32,
    /// Distance to the nearest contact's language (0..1).
    pub language_distance: f32,
}

impl SubstrateGradients {
    /// All-zero gradients (used by the engine's `religion_gradients_for_settlement`
    /// when the settlement has no recorded substrate signal yet).
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Per-settlement emergent religious profile (FR-CIV-BELIEF-001 §8).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReligiousProfile {
    /// Settlement id.
    pub settlement_id: u32,
    /// Settlement population used for scaling the §9 reaction curve.
    pub population: u32,
    /// Monitoring score (0..1) — how strongly elites can detect defection.
    pub monitoring: f32,
    /// Mythic coherence (0..1) — how tightly the settlement holds shared myth.
    pub mythic_coherence: f32,
    /// Uncertainty reduction (0..1) — perceived risk from stochastic events.
    pub uncertainty_reduction: f32,
    /// Tick the profile was first seeded.
    pub created_tick: u64,
}

impl ReligiousProfile {
    /// Build a fresh profile for `population` at `tick`.
    #[must_use]
    pub fn new(population: u32, tick: u64) -> Self {
        Self {
            settlement_id: 0,
            population,
            monitoring: 0.0,
            mythic_coherence: 0.0,
            uncertainty_reduction: 0.0,
            created_tick: tick,
        }
    }
}

/// Apply the Norenzayan Big-Gods response curve to `profile` for `tick`.
/// Stub: identity — profile fields are returned untouched.
pub fn apply_big_gods_response(
    profile: &mut ReligiousProfile,
    gradients: &SubstrateGradients,
    tick: u64,
) {
    profile.monitoring = (profile.monitoring + gradients.grad_T * 0.01).clamp(0.0, 1.0);
    profile.mythic_coherence = (profile.mythic_coherence + gradients.grad_B * 0.01).clamp(0.0, 1.0);
    profile.uncertainty_reduction =
        (profile.uncertainty_reduction + gradients.unrest * 0.01).clamp(0.0, 1.0);
    profile.created_tick = tick;
}

/// Per-settlement substrate gradient sample. Stub: returns zero gradients.
#[must_use]
pub fn substrate_gradients_for(_settlement_id: u32) -> SubstrateGradients {
    SubstrateGradients::zero()
}

/// Last religion sample for a settlement (per the `emergence.metrics` consumer
/// stub — returns an empty profile so the surface compiles).
#[must_use]
pub fn last_religion_sample(settlement_id: u32) -> ReligiousProfile {
    ReligiousProfile {
        settlement_id,
        ..ReligiousProfile::default()
    }
}

/// Per-tick `ReligionEvent` snapshot for the belief phase (FR-CIV-BELIEF-001 §10).
/// Distinct from the engine-side `ReligionEvent` enum in `engine.rs` — this
/// struct is what the religion module's API surface (and `phase_belief`)
/// consumes; the engine-side enum is the wire shape.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReligionEvent {
    /// Settlement id the event pertains to.
    pub settlement_id: u32,
    /// Monitoring component of the event.
    pub monitoring: f32,
    /// Mythic coherence component of the event.
    pub mythic_coherence: f32,
    /// Uncertainty reduction component of the event.
    pub uncertainty_reduction: f32,
    /// Tick the event was emitted.
    pub tick: u64,
}

impl ReligionEvent {
    /// Construct a per-tick `ReligionEvent` for `settlement_id`.
    #[must_use]
    pub fn tick(
        settlement_id: u32,
        monitoring: f32,
        mythic_coherence: f32,
        uncertainty_reduction: f32,
        tick: u64,
    ) -> Self {
        Self {
            settlement_id,
            monitoring,
            mythic_coherence,
            uncertainty_reduction,
            tick,
        }
    }

    /// Whether the event is "notable" enough to surface on the wire / in
    /// legends ingest. Stub: always true so the engine keeps emitting
    /// events until the real threshold is wired.
    #[must_use]
    pub fn is_notable(&self) -> bool {
        true
    }
}
