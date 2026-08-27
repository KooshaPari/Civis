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

/// A specific religion with tenets and tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Religion {
    pub name: String,
    pub tenets: Vec<String>,
    pub adherence: f32,
    pub spread_rate: f32,
    pub founded_tick: u64,
    pub parent_id: Option<u32>,
}

impl Default for Religion {
    fn default() -> Self {
        Self {
            name: String::new(),
            tenets: Vec::new(),
            adherence: 0.0,
            spread_rate: 0.0,
            founded_tick: 0,
            parent_id: None,
        }
    }
}

/// System of doctrines, rituals, and taboos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeliefSystem {
    pub doctrines: Vec<String>,
    pub rituals: Vec<String>,
    pub taboos: Vec<String>,
    pub coherence: f32,
}

/// Per-civilian religious belief state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivilianBelief {
    pub religion_id: Option<u32>,
    pub devotion: f32,
}

impl Default for CivilianBelief {
    fn default() -> Self {
        Self {
            religion_id: None,
            devotion: 0.0,
        }
    }
}

/// Create a new religion with given name, tenets, founding population, and tick.
#[must_use]
pub fn found_religion(
    name: &str,
    tenets: Vec<String>,
    founding_population: u32,
    tick: u64,
) -> Religion {
    let spread_rate = if founding_population > 100 {
        0.05
    } else {
        0.02
    };
    Religion {
        name: name.to_string(),
        tenets,
        adherence: 1.0,
        spread_rate,
        founded_tick: tick,
        parent_id: None,
    }
}

/// Convert a civilian to a religion based on believer fraction.
/// Returns updated devotion level.
#[must_use]
pub fn convert_civilian(
    religion: &Religion,
    current_belief: &CivilianBelief,
    believer_fraction: f32,
) -> f32 {
    let pull = religion.spread_rate * (1.0 - current_belief.devotion) * (1.0 - believer_fraction);
    (current_belief.devotion + pull).clamp(0.0, 1.0)
}

/// Compute adherence ratio from followers and total population.
#[must_use]
pub fn compute_adherence(followers: u32, total_population: u32) -> f32 {
    if total_population == 0 {
        return 0.0;
    }
    followers as f32 / total_population as f32
}

/// Split a religion into parent and child (schism).
pub fn schism(
    parent: &Religion,
    dissenting_tenet: &str,
    split_fraction: f32,
    tick: u64,
) -> (Religion, Religion) {
    let child_tenets: Vec<String> = parent.tenets.iter().cloned().collect();
    let parent_tenets: Vec<String> = parent
        .tenets
        .iter()
        .filter(|t| t.as_str() != dissenting_tenet)
        .cloned()
        .collect();

    let child = Religion {
        name: format!("{} (Reformed)", parent.name),
        tenets: child_tenets,
        adherence: parent.adherence * split_fraction,
        spread_rate: parent.spread_rate * 0.8,
        founded_tick: tick,
        parent_id: Some(0),
    };
    let parent = Religion {
        name: parent.name.clone(),
        tenets: parent_tenets,
        adherence: parent.adherence * (1.0 - split_fraction),
        spread_rate: parent.spread_rate,
        founded_tick: parent.founded_tick,
        parent_id: parent.parent_id,
    };
    (parent, child)
}

/// Advance a religion by one tick.
#[must_use]
pub fn tick_religion(religion: &Religion, population: u32) -> Religion {
    let growth = if population > 0 {
        religion.spread_rate * 0.01
    } else {
        0.0
    };
    let new_adherence = (religion.adherence + growth).clamp(0.0, 1.0);
    Religion {
        adherence: new_adherence,
        ..religion.clone()
    }
}

#[cfg(test)]
mod religion_extended_tests {
    use super::*;

    #[test]
    fn found_religion_basic() {
        let rel = found_religion("Testism", vec!["Tenet A".into()], 200, 10);
        assert_eq!(rel.name, "Testism");
        assert_eq!(rel.tenets.len(), 1);
        assert_eq!(rel.founded_tick, 10);
        assert_eq!(rel.parent_id, None);
        assert!(rel.adherence > 0.0);
    }

    #[test]
    fn found_religion_spread_rate_scales() {
        let big = found_religion("Big", vec![], 200, 0);
        let small = found_religion("Small", vec![], 50, 0);
        assert!(big.spread_rate > small.spread_rate);
    }

    #[test]
    fn convert_civilian_increases_devotion() {
        let rel = found_religion("Faith", vec![], 100, 0);
        let belief = CivilianBelief::default();
        let new_dev = convert_civilian(&rel, &belief, 0.3);
        assert!(new_dev > belief.devotion);
    }

    #[test]
    fn convert_civilian_respects_max() {
        let rel = found_religion("Faith", vec![], 100, 0);
        let belief = CivilianBelief {
            religion_id: None,
            devotion: 0.95,
        };
        let new_dev = convert_civilian(&rel, &belief, 0.1);
        assert!(new_dev <= 1.0);
    }

    #[test]
    fn compute_adherence_normal() {
        assert!((compute_adherence(50, 200) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_adherence_zero_population() {
        assert_eq!(compute_adherence(0, 0), 0.0);
    }

    #[test]
    fn schism_produces_two_religions() {
        let parent = found_religion("Parent", vec!["T1".into(), "T2".into()], 100, 5);
        let (p, c) = schism(&parent, "T1", 0.4, 10);
        assert_eq!(c.name, "Parent (Reformed)");
        assert!(p.tenets.contains(&"T2".to_string()));
        assert!(!p.tenets.contains(&"T1".to_string()));
        assert!(c.tenets.contains(&"T1".to_string()));
        assert_eq!(c.founded_tick, 10);
    }

    #[test]
    fn tick_religion_increases_adherence() {
        let rel = found_religion("Faith", vec![], 100, 0);
        let ticked = tick_religion(&rel, 100);
        assert!(ticked.adherence >= rel.adherence);
    }

    #[test]
    fn religion_serde_roundtrip() {
        let rel = found_religion("Test", vec!["A".into()], 50, 3);
        let json = serde_json::to_string(&rel).unwrap();
        let decoded: Religion = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, rel.name);
        assert_eq!(decoded.tenets, rel.tenets);
    }

    #[test]
    fn belief_system_default() {
        let bs = BeliefSystem::default();
        assert!(bs.doctrines.is_empty());
        assert_eq!(bs.coherence, 0.0);
    }

    #[test]
    fn civilan_belief_default() {
        let cb = CivilianBelief::default();
        assert_eq!(cb.religion_id, None);
        assert_eq!(cb.devotion, 0.0);
    }
}
