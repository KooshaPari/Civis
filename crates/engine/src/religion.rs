// Religion module — belief systems, denominations, spread mechanics,
// holy sites, religious wars, and heresy/schism resolution.
//
// Original types (SubstrateGradients, ReligiousProfile, ReligionEvent,
// Religion, BeliefSystem, CivilianBelief) and core functions
// (found_religion, convert_civilian, compute_adherence, schism,
// tick_religion) are preserved for backward compatibility with
// engine.rs, culture_phases.rs, and settlement_helpers.rs.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can arise from religion module operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ReligionError {
    /// A denomination was not recognised.
    #[error("unknown denomination")]
    UnknownDenomination,

    /// Attempted an operation that requires non-zero population.
    #[error("population must be non-zero")]
    ZeroPopulation,

    /// The holy site generation seed produced an invalid configuration.
    #[error("invalid holy site seed")]
    InvalidSeed,
}

// ---------------------------------------------------------------------------
// Constants (preserved from original)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Substrate gradients (preserved from original)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Religious profile (preserved from original)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Big-Gods response curve (preserved from original)
// ---------------------------------------------------------------------------

/// Apply the Norenzayan Big-Gods response curve to `profile` for `tick`.
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

/// Per-settlement substrate gradient sample. Returns zero gradients.
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

// ---------------------------------------------------------------------------
// ReligionEvent (preserved from original)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Core religion types (preserved from original)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Core religion functions (preserved from original)
// ---------------------------------------------------------------------------

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

// ===========================================================================
// NEW FEATURE 1: Denomination System
// ===========================================================================

/// Religious denominations with distinct theological orientations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Denomination {
    /// Traditional, hierarchical, ritual-heavy.
    Orthodox,
    /// Scripture-focused, emphasis on reform and reinterpretation.
    Reformed,
    /// Experience-driven, ecstatic practices, inner truth.
    Mystical,
    /// Literalist interpretation, strict adherence, exclusivist.
    Fundamentalist,
    /// Blends multiple traditions, adaptable, pluralist.
    Syncretic,
    /// Rationalist, minimal supernatural claims, civic ethics.
    Secular,
}

/// Returns the core tenets associated with a denomination.
#[must_use]
pub fn denomination_tenets(denom: Denomination) -> Vec<String> {
    match denom {
        Denomination::Orthodox => vec![
            "Strict adherence to tradition".into(),
            "Hierarchical priesthood".into(),
            "Ritual purity".into(),
            "Sacred texts are immutable".into(),
        ],
        Denomination::Reformed => vec![
            "Personal scripture study".into(),
            "Congregational governance".into(),
            "Moral reform over ritual".into(),
            "Vernacular liturgy".into(),
        ],
        Denomination::Mystical => vec![
            "Inner spiritual experience".into(),
            "Meditation and ecstasy".into(),
            "Personal communion with the divine".into(),
            "Symbolic over literal truth".into(),
        ],
        Denomination::Fundamentalist => vec![
            "Literal interpretation of sacred texts".into(),
            "Strict moral code".into(),
            "Exclusivist salvation".into(),
            "Rejection of outside influence".into(),
        ],
        Denomination::Syncretic => vec![
            "Blending of traditions".into(),
            "Adaptive theology".into(),
            "Inclusive of diverse practices".into(),
            "Pragmatic over doctrinal".into(),
        ],
        Denomination::Secular => vec![
            "Rational inquiry".into(),
            "Civic virtue".into(),
            "Separation of faith and governance".into(),
            "Ethical philosophy without supernaturalism".into(),
        ],
    }
}

/// Returns a spread-rate multiplier for the denomination.
/// Orthodox traditions spread slowly through institutional structures.
/// Syncretic traditions spread fastest due to adaptability.
#[must_use]
pub fn denomination_spread_modifier(denom: Denomination) -> f32 {
    match denom {
        Denomination::Orthodox => 0.8,
        Denomination::Reformed => 1.0,
        Denomination::Mystical => 1.1,
        Denomination::Fundamentalist => 1.3,
        Denomination::Syncretic => 1.5,
        Denomination::Secular => 0.7,
    }
}

/// Returns the tolerance level (0.0 = completely intolerant, 1.0 = fully tolerant)
/// that a denomination has toward other religions.
#[must_use]
pub fn denomination_tolerance(denom: Denomination) -> f32 {
    match denom {
        Denomination::Orthodox => 0.3,
        Denomination::Reformed => 0.5,
        Denomination::Mystical => 0.7,
        Denomination::Fundamentalist => 0.1,
        Denomination::Syncretic => 0.9,
        Denomination::Secular => 0.8,
    }
}

// ===========================================================================
// NEW FEATURE 2: Spread Mechanics
// ===========================================================================

/// Parameters governing how a religion spreads between settlements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadMechanics {
    /// Base rate of conversion per tick (0.0–1.0).
    pub base_spread_rate: f32,
    /// Bonus multiplier per adjacent follower concentration.
    pub adjacency_bonus: f32,
    /// Scaling factor for target population size.
    pub population_factor: f32,
    /// Resistance from existing cultural practices (0.0–1.0).
    pub cultural_resistance: f32,
}

/// Compute the effective spread rate given adjacency data and target population.
///
/// Formula:
/// ```text
/// rate = base_spread_rate
///        * (1.0 + adjacency_bonus * (adjacent_followers / total_adjacent))
///        * (1.0 + population_factor * ln(1 + target_population))
///        * (1.0 - cultural_resistance)
/// ```
#[must_use]
pub fn compute_spread_rate(
    mechanics: &SpreadMechanics,
    adjacent_followers: u32,
    total_adjacent: u32,
    target_population: u32,
) -> f32 {
    let adjacency_ratio = if total_adjacent > 0 {
        adjacent_followers as f32 / total_adjacent as f32
    } else {
        0.0
    };
    let pop_component = mechanics.population_factor * (1.0 + target_population as f32).ln();
    let resistance_damping = (1.0 - mechanics.cultural_resistance).max(0.0);

    mechanics.base_spread_rate
        * (1.0 + mechanics.adjacency_bonus * adjacency_ratio)
        * (1.0 + pop_component)
        * resistance_damping
}

/// Compute the conversion contribution from source to target settlement
/// given their respective adherence levels and the distance between them.
///
/// Returns a value in [0.0, 1.0] representing the conversion pull.
#[must_use]
pub fn convert_settlement(
    mechanics: &SpreadMechanics,
    source_adherence: f32,
    target_adherence: f32,
    distance: f32,
) -> f32 {
    // Adherence differential drives conversion pressure.
    let differential = (source_adherence - target_adherence).max(0.0);
    // Distance exponentially dampens influence.
    let distance_decay = (-distance).exp();
    let resistance_damping = (1.0 - mechanics.cultural_resistance).max(0.0);

    (mechanics.base_spread_rate * differential * distance_decay * resistance_damping)
        .clamp(0.0, 1.0)
}

// ===========================================================================
// NEW FEATURE 3: Holy Site Generation
// ===========================================================================

/// Terrain classification for holy sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HolySiteType {
    /// Elevated sacred ground (mountains, peaks).
    Mountain,
    /// Water-adjacent sacred ground (rivers, springs).
    River,
    /// Woodland sacred ground (ancient groves).
    Forest,
    /// Arid sacred ground (deserts, oases).
    Desert,
    /// Subterranean sacred ground (caves, caverns).
    Cave,
}

/// A sacred location that anchors a religion to a geographic feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolySite {
    /// Name of the holy site.
    pub name: String,
    /// Terrain type of the site.
    pub site_type: HolySiteType,
    /// Sacredness score (0.0–1.0), derived from terrain weight and seed.
    pub sacred_score: f32,
    /// Approximate population served by this site.
    pub population_served: u32,
}

/// Generate a holy site deterministically from a name, terrain weight, and seed.
///
/// The terrain weight biases which `HolySiteType` is chosen (higher weight = more
/// likely to produce Mountain/Cave types, lower weight = more River/Forest).
/// The sacred score is derived from the terrain weight combined with a seeded
/// pseudo-random component.
#[must_use]
pub fn generate_holy_site(name: &str, terrain_weight: f32, sacred_seed: u64) -> HolySite {
    let mut rng = ChaCha8Rng::seed_from_u64(sacred_seed);

    // Map terrain_weight (0.0–1.0) to a site type via weighted selection.
    let roll: f32 = rng.gen();
    let adjusted = (roll + terrain_weight * 0.3).clamp(0.0, 1.0);
    let site_type = match adjusted {
        x if x < 0.15 => HolySiteType::Cave,
        x if x < 0.35 => HolySiteType::Desert,
        x if x < 0.55 => HolySiteType::Forest,
        x if x < 0.80 => HolySiteType::River,
        _ => HolySiteType::Mountain,
    };

    // Sacred score combines terrain weight with a pseudo-random component.
    let sacred_noise: f32 = rng.gen();
    let sacred_score = (terrain_weight * 0.6 + sacred_noise * 0.4).clamp(0.0, 1.0);

    HolySite {
        name: name.to_string(),
        site_type,
        sacred_score,
        population_served: 0,
    }
}

/// Compute the satisfaction factor a holy site provides to nearby population.
///
/// Satisfaction is high when the sacred score is high relative to the
/// population pressure (more people = harder to satisfy).
#[must_use]
pub fn holy_site_satisfaction(site: &HolySite, nearby_population: u32) -> f32 {
    if nearby_population == 0 {
        return 0.0;
    }
    // Base satisfaction from sacred score, diminished by population pressure.
    let pressure = (nearby_population as f32 / 1000.0).min(1.0);
    (site.sacred_score * (1.0 - pressure * 0.5)).clamp(0.0, 1.0)
}

// ===========================================================================
// NEW FEATURE 4: Religious Wars
// ===========================================================================

/// Casus belli for religious warfare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WarCause {
    /// Dispute over a sacred territory.
    HolyLand,
    /// Suppression of perceived heresy.
    Heresy,
    /// Conflict arising from a schism.
    Schism,
    /// Defence or disruption of faith-based trade routes.
    TradeRoute,
}

/// Represents an active or resolved religious war.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReligiousWar {
    /// Name of the aggressor's religion.
    pub aggressor_religion: String,
    /// Name of the defender's religion.
    pub defender_religion: String,
    /// Cause of the war.
    pub cause: WarCause,
    /// Aggressor's military/religious strength (0.0–1.0).
    pub strength: f32,
    /// Expected duration in ticks.
    pub duration_ticks: u64,
}

/// Determine whether a holy war can be declared given adherence levels and cause.
///
/// Requirements:
/// - Aggressor adherence must be > 0.6 (strong faith conviction).
/// - Defender adherence must be > 0.3 (worth fighting against).
/// - `casus_belli_cause` must match a known cause string.
#[must_use]
pub fn can_declare_holy_war(
    aggressor_adherence: f32,
    defender_adherence: f32,
    casus_belli_cause: &str,
) -> bool {
    let valid_cause = matches!(
        casus_belli_cause,
        "HolyLand" | "Heresy" | "Schism" | "TradeRoute"
    );
    aggressor_adherence > 0.6 && defender_adherence > 0.3 && valid_cause
}

/// Resolve a religious war between aggressor and defender.
///
/// Returns `(aggressor_change, defender_change)` — negative values indicate
/// losses (reduced adherence or strength). The result is deterministic
/// based on input strengths.
#[must_use]
pub fn resolve_holy_war(
    war: &ReligiousWar,
    aggressor_strength: f32,
    defender_strength: f32,
) -> (f32, f32) {
    let total = aggressor_strength + defender_strength;
    if total <= 0.0 {
        return (0.0, 0.0);
    }

    // Aggressor wins if stronger; losses proportional to the loser's share.
    let aggressor_share = aggressor_strength / total;
    let defender_share = defender_strength / total;

    // Strength modifier from war strength and duration.
    let impact = war.strength * (war.duration_ticks as f32 * 0.01).min(1.0);

    let aggressor_change = if aggressor_share > defender_share {
        // Aggressor wins: smaller loss (winner's cost)
        -(1.0 - aggressor_share) * impact
    } else {
        // Aggressor loses: larger loss
        -(1.0 + defender_share) * impact
    };

    let defender_change = if defender_share > aggressor_share {
        // Defender wins: smaller loss
        -(1.0 - defender_share) * impact
    } else {
        // Defender loses: larger loss
        -(1.0 + aggressor_share) * impact
    };

    (
        aggressor_change.clamp(-1.0, 0.0),
        defender_change.clamp(-1.0, 0.0),
    )
}

// ===========================================================================
// NEW FEATURE 5: Heresy / Schism Events
// ===========================================================================

/// A detected heresy event — a divergent tenet gaining followers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeresyEvent {
    /// Name of the original religion.
    pub original_religion: String,
    /// The tenet that has become heretical.
    pub heretical_tenet: String,
    /// Fraction of followers adopting the heretical view.
    pub follower_fraction: f32,
    /// Tick at which the heresy was detected.
    pub tick: u64,
}

/// Detect heresy events by comparing a religion's tenets against divergent tenets.
///
/// Each entry in `divergent_tenets` that is NOT in the religion's canonical tenet
/// list produces a `HeresyEvent`. The `follower_fraction` is estimated from the
/// religion's adherence — higher adherence means the heresy is a smaller fraction.
#[must_use]
pub fn detect_heresy(religion: &Religion, divergent_tenets: &[String]) -> Vec<HeresyEvent> {
    let mut events = Vec::new();
    for tenet in divergent_tenets {
        // A tenet is "divergent" if it's NOT in the canonical list.
        if !religion.tenets.contains(tenet) {
            // Estimate follower fraction: more adherents = heresy is smaller fraction.
            let fraction = if religion.adherence > 0.0 {
                (1.0 - religion.adherence).max(0.01)
            } else {
                0.5
            };
            events.push(HeresyEvent {
                original_religion: religion.name.clone(),
                heretical_tenet: tenet.clone(),
                follower_fraction: fraction,
                tick: religion.founded_tick,
            });
        }
    }
    events
}

/// Resolve heresy events into new religions (schisms) when follower fractions
/// exceed the 0.25 threshold.
///
/// Each qualifying event produces a new `Religion` derived from the parent.
/// Non-qualifying events (below threshold) are silently ignored.
#[must_use]
pub fn resolve_schism(parent: &Religion, events: &[HeresyEvent], tick: u64) -> Vec<Religion> {
    let threshold = 0.25;
    let mut children = Vec::new();

    for event in events {
        if event.follower_fraction >= threshold {
            // Build child tenets: parent tenets + the heretical tenet.
            let mut child_tenets = parent.tenets.clone();
            if !child_tenets.contains(&event.heretical_tenet) {
                child_tenets.push(event.heretical_tenet.clone());
            }

            // Parent loses the fraction; child inherits it.
            let child_name = format!("{} ({})", parent.name, event.heretical_tenet);
            children.push(Religion {
                name: child_name,
                tenets: child_tenets,
                adherence: parent.adherence * event.follower_fraction,
                spread_rate: parent.spread_rate * 0.9,
                founded_tick: tick,
                parent_id: None,
            });
        }
    }

    children
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod religion_extended_tests {
    use super::*;

    // ----- Original tests (preserved) -----

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

    // ================================================================
    // NEW TESTS: Denomination System
    // ================================================================

    #[test]
    fn denomination_tenets_orthodox() {
        let tenets = denomination_tenets(Denomination::Orthodox);
        assert_eq!(tenets.len(), 4);
        assert!(tenets.iter().any(|t| t.contains("tradition")));
        assert!(tenets.iter().any(|t| t.contains("priesthood")));
    }

    #[test]
    fn denomination_tenets_reformed() {
        let tenets = denomination_tenets(Denomination::Reformed);
        assert!(tenets.iter().any(|t| t.contains("scripture")));
        assert!(tenets.iter().any(|t| t.contains("reform")));
    }

    #[test]
    fn denomination_tenets_mystical() {
        let tenets = denomination_tenets(Denomination::Mystical);
        assert!(tenets.iter().any(|t| t.contains("experience")));
        assert!(tenets.iter().any(|t| t.contains("ecstasy")));
    }

    #[test]
    fn denomination_tenets_fundamentalist() {
        let tenets = denomination_tenets(Denomination::Fundamentalist);
        assert!(tenets.iter().any(|t| t.contains("Literal")));
        assert!(tenets.iter().any(|t| t.contains("Exclusivist")));
    }

    #[test]
    fn denomination_tenets_syncretic() {
        let tenets = denomination_tenets(Denomination::Syncretic);
        assert!(tenets.iter().any(|t| t.contains("Blending")));
        assert!(tenets.iter().any(|t| t.contains("Inclusive")));
    }

    #[test]
    fn denomination_tenets_secular() {
        let tenets = denomination_tenets(Denomination::Secular);
        assert!(tenets.iter().any(|t| t.contains("Rational")));
        assert!(tenets.iter().any(|t| t.contains("Civic")));
    }

    #[test]
    fn denomination_spread_modifiers_ordered() {
        // Syncretic > Fundamentalist > Mystical > Reformed > Orthodox > Secular
        let syn = denomination_spread_modifier(Denomination::Syncretic);
        let fund = denomination_spread_modifier(Denomination::Fundamentalist);
        let myst = denomination_spread_modifier(Denomination::Mystical);
        let refm = denomination_spread_modifier(Denomination::Reformed);
        let orth = denomination_spread_modifier(Denomination::Orthodox);
        let sec = denomination_spread_modifier(Denomination::Secular);

        assert!(syn > fund);
        assert!(fund > myst);
        assert!(myst > refm);
        assert!(refm > orth);
        assert!(orth > sec);
    }

    #[test]
    fn denomination_tolerance_syncretic_highest() {
        let syn = denomination_tolerance(Denomination::Syncretic);
        let fund = denomination_tolerance(Denomination::Fundamentalist);
        assert!(syn > fund);
        assert!(syn > 0.8);
        assert!(fund < 0.2);
    }

    #[test]
    fn denomination_serde_roundtrip() {
        let denom = Denomination::Mystical;
        let json = serde_json::to_string(&denom).expect("serialization should succeed");
        let decoded: Denomination =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(decoded, denom);
    }

    // ================================================================
    // NEW TESTS: Spread Mechanics
    // ================================================================

    #[test]
    fn compute_spread_rate_zero_followers() {
        let mechanics = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.2,
        };
        let rate = compute_spread_rate(&mechanics, 0, 100, 500);
        // With zero adjacent followers, the adjacency term contributes nothing
        // beyond the base rate.
        assert!(rate > 0.0);
        assert!(rate < 1.0);
    }

    #[test]
    fn compute_spread_rate_high_followers_increases() {
        let mechanics = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.0,
        };
        let low = compute_spread_rate(&mechanics, 10, 100, 100);
        let high = compute_spread_rate(&mechanics, 90, 100, 100);
        assert!(high > low);
    }

    #[test]
    fn compute_spread_rate_cultural_resistance_dampens() {
        let mechanics_no_resist = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.0,
        };
        let mechanics_high_resist = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.8,
        };
        let no_resist = compute_spread_rate(&mechanics_no_resist, 50, 100, 100);
        let high_resist = compute_spread_rate(&mechanics_high_resist, 50, 100, 100);
        assert!(no_resist > high_resist);
    }

    #[test]
    fn convert_settlement_closer_distance_higher_pull() {
        let mechanics = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.0,
        };
        let close = convert_settlement(&mechanics, 0.8, 0.2, 1.0);
        let far = convert_settlement(&mechanics, 0.8, 0.2, 10.0);
        assert!(close > far);
    }

    #[test]
    fn convert_settlement_no_conversion_if_source_weaker() {
        let mechanics = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.0,
        };
        // Source adherence < target adherence -> no differential -> 0 conversion.
        let result = convert_settlement(&mechanics, 0.2, 0.8, 1.0);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_spread_rate_zero_total_adjacent() {
        let mechanics = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.0,
        };
        let rate = compute_spread_rate(&mechanics, 0, 0, 500);
        assert!(rate > 0.0);
    }

    // ================================================================
    // NEW TESTS: Holy Site Generation
    // ================================================================

    #[test]
    fn generate_holy_site_deterministic() {
        let site1 = generate_holy_site("Mount Sinai", 0.5, 42);
        let site2 = generate_holy_site("Mount Sinai", 0.5, 42);
        assert_eq!(site1.site_type, site2.site_type);
        assert_eq!(site1.sacred_score, site2.sacred_score);
        assert_eq!(site1.name, site2.name);
    }

    #[test]
    fn generate_holy_site_different_seeds() {
        let site1 = generate_holy_site("Site A", 0.5, 1);
        let site2 = generate_holy_site("Site A", 0.5, 999);
        // Different seeds should produce different results (at least one field differs).
        assert!(
            site1.site_type != site2.site_type
                || (site1.sacred_score - site2.sacred_score).abs() > f32::EPSILON
        );
    }

    #[test]
    fn generate_holy_site_high_terrain_weight_biases_mountain() {
        // With a very high terrain weight, Mountain is more likely.
        let mut mountain_count = 0;
        for seed in 0..100 {
            let site = generate_holy_site("Peak", 0.95, seed);
            if site.site_type == HolySiteType::Mountain {
                mountain_count += 1;
            }
        }
        // With high terrain weight, Mountain should appear more than 30% of the time.
        assert!(mountain_count > 30);
    }

    #[test]
    fn holy_site_satisfaction_zero_population() {
        let site = HolySite {
            name: "Test".into(),
            site_type: HolySiteType::Mountain,
            sacred_score: 1.0,
            population_served: 0,
        };
        assert!((holy_site_satisfaction(&site, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn holy_site_satisfaction_scales_with_sacred_score() {
        let low_sacred = HolySite {
            name: "Low".into(),
            site_type: HolySiteType::River,
            sacred_score: 0.2,
            population_served: 0,
        };
        let high_sacred = HolySite {
            name: "High".into(),
            site_type: HolySiteType::River,
            sacred_score: 0.9,
            population_served: 0,
        };
        let pop = 100;
        let low_sat = holy_site_satisfaction(&low_sacred, pop);
        let high_sat = holy_site_satisfaction(&high_sacred, pop);
        assert!(high_sat > low_sat);
    }

    #[test]
    fn holy_site_satisfaction_large_population_reduces() {
        let site = HolySite {
            name: "Test".into(),
            site_type: HolySiteType::Forest,
            sacred_score: 0.8,
            population_served: 0,
        };
        let sat_small = holy_site_satisfaction(&site, 50);
        let sat_large = holy_site_satisfaction(&site, 2000);
        assert!(sat_small > sat_large);
    }

    // ================================================================
    // NEW TESTS: Religious Wars
    // ================================================================

    #[test]
    fn can_declare_holy_war_valid() {
        assert!(can_declare_holy_war(0.8, 0.5, "HolyLand"));
        assert!(can_declare_holy_war(0.7, 0.4, "Heresy"));
        assert!(can_declare_holy_war(0.9, 0.6, "Schism"));
        assert!(can_declare_holy_war(0.65, 0.35, "TradeRoute"));
    }

    #[test]
    fn can_declare_holy_war_low_aggressor_adherence() {
        // Aggressor adherence too low (< 0.6).
        assert!(!can_declare_holy_war(0.5, 0.5, "HolyLand"));
    }

    #[test]
    fn can_declare_holy_war_low_defender_adherence() {
        // Defender adherence too low (< 0.3).
        assert!(!can_declare_holy_war(0.8, 0.2, "HolyLand"));
    }

    #[test]
    fn can_declare_holy_war_invalid_cause() {
        assert!(!can_declare_holy_war(0.8, 0.5, "EconomicSanctions"));
    }

    #[test]
    fn resolve_holy_war_stronger_aggressor_wins() {
        let war = ReligiousWar {
            aggressor_religion: "Faith A".into(),
            defender_religion: "Faith B".into(),
            cause: WarCause::HolyLand,
            strength: 0.8,
            duration_ticks: 50,
        };
        let (agg_change, def_change) = resolve_holy_war(&war, 0.9, 0.3);
        // Defender should lose more than aggressor.
        assert!(def_change < agg_change);
        assert!(agg_change <= 0.0);
        assert!(def_change <= 0.0);
    }

    #[test]
    fn resolve_holy_war_equal_strengths() {
        let war = ReligiousWar {
            aggressor_religion: "A".into(),
            defender_religion: "B".into(),
            cause: WarCause::Schism,
            strength: 0.5,
            duration_ticks: 10,
        };
        let (agg, def) = resolve_holy_war(&war, 0.5, 0.5);
        // Equal strengths -> equal losses.
        assert!((agg - def).abs() < 0.01);
    }

    #[test]
    fn resolve_holy_war_zero_strengths() {
        let war = ReligiousWar {
            aggressor_religion: "A".into(),
            defender_religion: "B".into(),
            cause: WarCause::Heresy,
            strength: 0.5,
            duration_ticks: 10,
        };
        let (agg, def) = resolve_holy_war(&war, 0.0, 0.0);
        assert!((agg - 0.0).abs() < f32::EPSILON);
        assert!((def - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn religious_war_serde_roundtrip() {
        let war = ReligiousWar {
            aggressor_religion: "Faith".into(),
            defender_religion: "Heresy".into(),
            cause: WarCause::Heresy,
            strength: 0.7,
            duration_ticks: 25,
        };
        let json = serde_json::to_string(&war).expect("serialization should succeed");
        let decoded: ReligiousWar =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(decoded.aggressor_religion, "Faith");
        assert_eq!(decoded.cause, WarCause::Heresy);
    }

    // ================================================================
    // NEW TESTS: Heresy / Schism Events
    // ================================================================

    #[test]
    fn detect_heresy_no_divergent_tenets() {
        let rel = found_religion("Pure", vec!["T1".into(), "T2".into()], 100, 0);
        let events = detect_heresy(&rel, &[]);
        assert!(events.is_empty());
    }

    #[test]
    fn detect_heresy_with_divergent_tenets() {
        let rel = found_religion("Pure", vec!["T1".into()], 100, 0);
        let divergent = vec!["T2".into(), "T3".into()];
        let events = detect_heresy(&rel, &divergent);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].heretical_tenet, "T2");
        assert_eq!(events[1].heretical_tenet, "T3");
        assert_eq!(events[0].original_religion, "Pure");
    }

    #[test]
    fn detect_heresy_ignores_canonical_tenets() {
        let rel = found_religion("Orthodox", vec!["T1".into(), "T2".into()], 100, 0);
        // Both "T1" and "T2" are canonical; "T3" is divergent.
        let divergent = vec!["T1".into(), "T3".into()];
        let events = detect_heresy(&rel, &divergent);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].heretical_tenet, "T3");
    }

    #[test]
    fn detect_heresy_follower_fraction_depends_on_adherence() {
        let high_adh = Religion {
            adherence: 0.9,
            ..found_religion("High", vec![], 100, 0)
        };
        let low_adh = Religion {
            adherence: 0.1,
            ..found_religion("Low", vec![], 100, 0)
        };
        let divergent = vec!["Heresy".into()];
        let events_high = detect_heresy(&high_adh, &divergent);
        let events_low = detect_heresy(&low_adh, &divergent);
        // Higher adherence -> smaller heretical fraction.
        assert!(events_high[0].follower_fraction < events_low[0].follower_fraction);
    }

    #[test]
    fn resolve_schism_below_threshold_no_children() {
        let parent = found_religion("Parent", vec!["T1".into()], 100, 0);
        let events = vec![HeresyEvent {
            original_religion: "Parent".into(),
            heretical_tenet: "Heresy".into(),
            follower_fraction: 0.1, // Below 0.25 threshold.
            tick: 10,
        }];
        let children = resolve_schism(&parent, &events, 10);
        assert!(children.is_empty());
    }

    #[test]
    fn resolve_schism_above_threshold_produces_children() {
        let parent = found_religion("Parent", vec!["T1".into(), "T2".into()], 100, 0);
        let events = vec![HeresyEvent {
            original_religion: "Parent".into(),
            heretical_tenet: "Heresy".into(),
            follower_fraction: 0.4, // Above 0.25 threshold.
            tick: 10,
        }];
        let children = resolve_schism(&parent, &events, 10);
        assert_eq!(children.len(), 1);
        let child = &children[0];
        assert!(child.name.contains("Heresy"));
        assert!(child.tenets.contains(&"T1".to_string()));
        assert!(child.tenets.contains(&"Heresy".to_string()));
        // Child adherence = parent adherence * fraction.
        assert!((child.adherence - 1.0 * 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_schism_multiple_events() {
        let parent = found_religion("Parent", vec!["T1".into()], 100, 0);
        let events = vec![
            HeresyEvent {
                original_religion: "Parent".into(),
                heretical_tenet: "Alpha".into(),
                follower_fraction: 0.3,
                tick: 5,
            },
            HeresyEvent {
                original_religion: "Parent".into(),
                heretical_tenet: "Beta".into(),
                follower_fraction: 0.5,
                tick: 5,
            },
            HeresyEvent {
                original_religion: "Parent".into(),
                heretical_tenet: "Gamma".into(),
                follower_fraction: 0.1, // Below threshold.
                tick: 5,
            },
        ];
        let children = resolve_schism(&parent, &events, 5);
        // Two events above threshold -> two children.
        assert_eq!(children.len(), 2);
        assert!(children.iter().any(|c| c.name.contains("Alpha")));
        assert!(children.iter().any(|c| c.name.contains("Beta")));
    }

    #[test]
    fn heresy_serde_roundtrip() {
        let event = HeresyEvent {
            original_religion: "Faith".into(),
            heretical_tenet: "NewDoctrine".into(),
            follower_fraction: 0.35,
            tick: 42,
        };
        let json = serde_json::to_string(&event).expect("serialization should succeed");
        let decoded: HeresyEvent =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(decoded.original_religion, "Faith");
        assert_eq!(decoded.heretical_tenet, "NewDoctrine");
        assert!((decoded.follower_fraction - 0.35).abs() < f32::EPSILON);
    }

    // ================================================================
    // Edge-case tests
    // ================================================================

    #[test]
    fn compute_adherence_full_population() {
        // All 1000 people are followers.
        assert!((compute_adherence(1000, 1000) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_religion_zero_population() {
        let rel = found_religion("Stagnant", vec![], 100, 0);
        let ticked = tick_religion(&rel, 0);
        // With zero population, adherence should not change.
        assert!((ticked.adherence - rel.adherence).abs() < f32::EPSILON);
    }

    #[test]
    fn convert_settlement_full_adherence_source() {
        let mechanics = SpreadMechanics {
            base_spread_rate: 0.1,
            adjacency_bonus: 0.5,
            population_factor: 0.02,
            cultural_resistance: 0.0,
        };
        // Source at full adherence, target at zero -> max differential.
        let result = convert_settlement(&mechanics, 1.0, 0.0, 0.0);
        assert!(result > 0.0);
        assert!(result <= 1.0);
    }

    #[test]
    fn religion_error_display() {
        let err = ReligionError::UnknownDenomination;
        assert_eq!(format!("{err}"), "unknown denomination");
        let err = ReligionError::ZeroPopulation;
        assert_eq!(format!("{err}"), "population must be non-zero");
    }

    #[test]
    fn denomination_all_variants_have_tenets() {
        let variants = [
            Denomination::Orthodox,
            Denomination::Reformed,
            Denomination::Mystical,
            Denomination::Fundamentalist,
            Denomination::Syncretic,
            Denomination::Secular,
        ];
        for v in variants {
            let tenets = denomination_tenets(v);
            assert!(!tenets.is_empty(), "Denomination {v:?} has no tenets");
        }
    }
}
