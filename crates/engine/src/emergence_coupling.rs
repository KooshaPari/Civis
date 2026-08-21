//! Emergence policy coupling functions (FR-CIV-0100).
//!
//! These pure free functions implement downward-causation and upward-causation
//! couplings between the simulation's macro systems. They are reserved for
//! future integration and currently have `#[allow(dead_code)]`.

use crate::engine::{
    Building, BuildingType, Citizen, ClusterStocks, CohesionEvent, CohesionEventKind,
    CohesionSnapshot, CombatDamagePulse, EconomicFocus, FabricTier, InvariantError,
    InstitutionEvent, JobType, KinshipEdge, KinshipKind, LanguageState, MembershipPayoffTotals,
    MilitaryUnit, MoodSnapshot, Position, Production, ReligionEvent,
    ReligionEventKind, ReligiousProfile, ResearchCache, Resources, ResourceType, Sim,
    SimSeed, Simulation, SimulationSnapshot, StratBand, StratificationEvent,
    StratificationEventKind, StratificationReport, TradeRoute, UnitType, WorldState,
};
use crate::fixed_math::{Fixed, FixedFromNum};
use crate::SCALE;
use civ_agents::{
    Alignment, Civilian as AgentCivilian, ClusterId, ClusterMember, DiplomacyMatrix,
    DiplomacyOutcome, DiplomacySignal, LodTier, Needs, Position3d, Psyche, SocialGraph,
    Tools, Wardrobe,
};
use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_agents::diplomacy::GriefAccumulator;
use civ_build::{BuildSite, DemandSignals};
use civ_economy::{Good, SettlementTradeFlow};
use civ_genetics::sentience::{
    cognition_score, CognitionTraitProfile, SentienceThreshold,
};
use civ_genetics::Dna;
use civ_planet::{BiomeKind, WeatherCell};
use civ_tactics::{CombatEngagement, DoctrineLibrary};
use civ_voxel::{WorldCoord, FIXED_SCALE};
use hecs::World;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::culture::{
    culture_cooperation_signal, culture_openness_signal, FactionIdeologyState,
};

/// Maximum chronicle history lines retained in [`WorldState::chronicle`].
#[allow(dead_code)] // Reserved for future simulation integration
const CHRONICLE_MAX_LEN: usize = 200;

/// Food units each cluster member adds to settlement stock per tick in
/// [`Simulation::phase_life`].
#[allow(dead_code)] // Reserved for future simulation integration
const CLUSTER_FOOD_PRODUCTION_PER_MEMBER: i64 = 1;
/// Food units each cluster member drains per tick in
/// [`Simulation::phase_settlement_consumption`]. Must be >= production so the
/// accumulator stays bounded (net zero at matched rates; converges toward zero
/// when strictly greater).
#[allow(dead_code)] // Reserved for future simulation integration
const CLUSTER_FOOD_CONSUMPTION_PER_MEMBER: i64 = 1;
/// Market weight for settlement food commons before pressure scaling (N1).
#[allow(dead_code)] // Reserved for future simulation integration
const SETTLEMENT_FOOD_MARKET_WEIGHT: i64 = 2;
/// Divisor mapping population-scale demand/supply (and settlement commons) into
/// the capped per-tick food price step (N1: local abundance must move price
/// within `MarketState::apply_pressure`'s Â±8 cent clamp).
#[allow(dead_code)] // Reserved for future simulation integration
const FOOD_MARKET_PRESSURE_SCALE: i64 = 500_000;

/// Baseline food clearing price (cents) at which births are unaffected by
/// scarcity. Matches `MarketState::default()`'s food price.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const FOOD_SCARCITY_BASELINE: i64 = 1_000;

/// Tech unlock bits (irreversible, set-only).
#[allow(dead_code)] // Reserved for future simulation integration
pub const TECH_IRRIGATION: u64 = 1 << 0;
#[allow(dead_code)] // Reserved for future simulation integration
pub const TECH_STORAGE: u64 = 1 << 1;
#[allow(dead_code)] // Reserved for future simulation integration
pub const TECH_METALLURGY: u64 = 1 << 2;
#[allow(dead_code)] // Reserved for future simulation integration
pub const TECH_WRITING: u64 = 1 << 3;
#[allow(dead_code)] // Reserved for future simulation integration
pub const TECH_SANITATION: u64 = 1 << 4;
#[allow(dead_code)] // Reserved for future simulation integration
pub const TECH_GUNPOWDER: u64 = 1 << 5;

/// Discrete tech unlocks reached by a given research tier (set-only bitmask).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn tech_unlocks_for_tier(research_tier: u64) -> u64 {
    let mut bits = 0u64;
    if research_tier >= 1 {
        bits |= TECH_IRRIGATION;
    }
    if research_tier >= 2 {
        bits |= TECH_STORAGE;
    }
    if research_tier >= 3 {
        bits |= TECH_METALLURGY;
    }
    if research_tier >= 4 {
        bits |= TECH_WRITING;
    }
    if research_tier >= 5 {
        bits |= TECH_SANITATION;
    }
    if research_tier >= 6 {
        bits |= TECH_GUNPOWDER;
    }
    bits
}

/// Downward-causation policy (FR-CIV-0100 emergence): scarcity in the food
/// market damps the birth rate, closing the research -> carrying-capacity ->
/// economy -> population loop. Returns a multiplier in `(0.0, 1.0]` applied to
/// the per-tick birth chance.
///
/// At or below the baseline price (abundance) the factor is `1.0` â€” surplus
/// does NOT boost births above the natural rate (conservative; abundance is
/// already expressed via the ECS food-needs path). As the price rises above
/// baseline the factor falls as `baseline / price`, so a 2x price halves the
/// birth chance. The factor never reaches zero, so a starving society can still
/// recover, and it only ever scales births DOWN â€” population is never reduced
/// by this coupling.
#[allow(dead_code)] // Reserved for future simulation integration
fn food_scarcity_birth_factor(food_price: i64) -> f64 {
    let price = food_price.max(FOOD_SCARCITY_BASELINE);
    (FOOD_SCARCITY_BASELINE as f64 / price as f64).clamp(0.0, 1.0)
}

/// Per-tick change in societal unrest from food-market scarcity (FR-CIV-0100 Â§3
/// emergence). Above the baseline price unrest rises in proportion to the
/// shortfall (bounded per tick so it walks rather than jumps); at or below
/// baseline it decays toward contentment by a fixed step. The caller floors the
/// running total at zero.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn unrest_delta(food_price: i64) -> i64 {
    /// Largest single-tick rise, so a price spike can't instantly max unrest.
    const MAX_RISE: i64 = 50;
    /// Cents of shortfall that map to one unit of unrest rise.
    const CENTS_PER_UNREST: i64 = 20;
    /// Fixed decay applied each tick of abundance.
    const DECAY: i64 = 10;
    let scarcity = food_price - FOOD_SCARCITY_BASELINE;
    if scarcity > 0 {
        (scarcity / CENTS_PER_UNREST).clamp(1, MAX_RISE)
    } else {
        -DECAY
    }
}

/// FR-CIV-ECON: scarcity in NON-food commodities adds bounded unrest
/// (cost-of-living). Food is owned by unrest_delta(); skipped here to avoid
/// double-counting. Per-tick clamped to [-DECAY, MAX_RISE] â€” no runaway.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn commodity_unrest_delta(prices: &std::collections::BTreeMap<String, i64>) -> i64 {
    const BASELINE: i64 = 1_000;
    const CENTS_PER_UNREST: i64 = 40;
    const MAX_RISE: i64 = 15;
    const DECAY: i64 = 5;
    let mut rise: i64 = 0;
    for (good, &price) in prices {
        if good == "food" {
            continue;
        }
        let scarcity = price - BASELINE;
        if scarcity > 0 {
            rise = rise.saturating_add((scarcity / CENTS_PER_UNREST).min(MAX_RISE));
        } else {
            rise = rise.saturating_sub(DECAY);
        }
    }
    rise.clamp(-DECAY, MAX_RISE)
}

/// Effective food-price shadow for one faction's local wealth/scarcity (FR-CIV-0100
/// Â§3 emergence). Comfortable treasury and food sit at baseline; shortfall pushes
/// the shadow above baseline so [`unrest_delta`] accrues faction unrest.
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_wealth_scarcity_shadow(treasury: Fixed, resources: &Resources) -> i64 {
    const TREASURY_COMFORT: i64 = 8_000;
    const FOOD_COMFORT: i64 = 80;
    const FOOD_WEIGHT: i64 = 50;

    let treasury_i = (i64::from(treasury.to_bits()) / crate::SCALE).max(0);
    let food_i = (i64::from(resources.food.to_bits()) / crate::SCALE).max(0);
    let comfort = TREASURY_COMFORT + FOOD_COMFORT * FOOD_WEIGHT;
    let wealth = treasury_i + food_i * FOOD_WEIGHT;

    if wealth >= comfort {
        FOOD_SCARCITY_BASELINE
    } else {
        FOOD_SCARCITY_BASELINE + (comfort - wealth) / 4
    }
}

/// Per-tick faction unrest delta from that faction's wealth/scarcity shadow.
/// Mirrors global food-scarcity [`unrest_delta`].
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_unrest_delta_from_shadow(scarcity_shadow: i64) -> i64 {
    unrest_delta(scarcity_shadow)
}

/// Downward-causation policy (FR-CIV-0100 Â§3): energy depletion breeds unrest.
/// A fully-drained energy budget (blackout) adds a fixed unrest increment this
/// tick; a solvent budget adds none. An acute shock that bypasses the gradual
/// food-scarcity damping.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn energy_scarcity_unrest(energy_budget: Fixed) -> i64 {
    const BLACKOUT_UNREST: i64 = 15;
    if energy_budget <= Fixed::ZERO {
        BLACKOUT_UNREST
    } else {
        0
    }
}

/// Upward causation (FR-CIV-0100 Â§3): the mean MISERY of agents (negative Psyche
/// mood valence) adds to societal unrest. Reuses the ECS Psyche component â€” the
/// agent emotional layer feeding the macro web. Returns 0..MAX, bounded.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn agent_misery_unrest(world: &hecs::World) -> i64 {
    const MAX_MISERY_UNREST: i64 = 30;
    let (sum, n) = world
        .query::<&Psyche>()
        .iter()
        .fold((0.0f32, 0u32), |(s, n), (_, p)| {
            (s + (-p.mood.valence).max(0.0), n + 1)
        });
    if n == 0 {
        return 0;
    }
    let mean_misery = (sum / n as f32).clamp(0.0, 1.0); // 0 = content, 1 = max misery
    (mean_misery * MAX_MISERY_UNREST as f32) as i64
}

/// Upward causation (FR-CIV-0100 Â§3): micro ideology consensus (`Psyche.beliefs[0]`)
/// binds macro cohesion; polarization frays it. Pure `hecs::World` scan, capped i64.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn micro_cohesion_delta(world: &hecs::World) -> i64 {
    const MICRO_BIND_CAP: i64 = 12;
    const MICRO_FRAY_CAP: i64 = 18;
    const MIN_AGENTS: u32 = 2;
    const CONSENSUS_SCALE: f32 = 4.0;

    let mut n = 0u32;
    let mut sum = 0.0f32;
    let mut sum_sq = 0.0f32;
    for (_, psyche) in world.query::<&Psyche>().iter() {
        let x = psyche.beliefs[0];
        n += 1;
        sum += x;
        sum_sq += x * x;
    }

    if n < MIN_AGENTS {
        return 0;
    }

    let n_f = n as f32;
    let mean = sum / n_f;
    let var = ((sum_sq / n_f) - mean * mean).max(0.0);
    let consensus = 1.0 - (CONSENSUS_SCALE * var).clamp(0.0, 1.0);
    let micro_bind = (consensus * MICRO_BIND_CAP as f32).floor() as i64;
    let micro_fray = ((1.0 - consensus) * MICRO_FRAY_CAP as f32).floor() as i64;
    micro_bind - micro_fray
}

/// Upward causation (FR-CIV-0100 Â§3): mean positive agent tie trust caches a
/// trade permille bonus for the next economy tick. Pure `hecs::World` scan.
#[allow(dead_code)] // Reserved for future simulation integration
fn micro_social_trust_permille(world: &hecs::World) -> u64 {
    const MICRO_TRUST_SCALE: f32 = 250.0;
    const MICRO_TRUST_CAP: u64 = 250;

    let mut n = 0u64;
    let mut sum = 0.0f32;
    for (_, graph) in world.query::<&SocialGraph>().iter() {
        for tie in &graph.ties {
            sum += tie.trust.clamp(0.0, 1.0);
            n += 1;
        }
    }

    if n == 0 {
        return 0;
    }

    let trust_mean = sum / n as f32;
    let raw = (trust_mean * MICRO_TRUST_SCALE).floor() as u64;
    raw.min(MICRO_TRUST_CAP)
}

/// Upward causation (FR-CIV-EMERGENCE-N11): average psyche maturity across all agents.
/// Mature populations stabilize belief (wisdom = stability). Pure `hecs::World` scan.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn avg_psyche_maturity(world: &hecs::World) -> f32 {
    let mut total = 0.0;
    let mut count = 0u32;
    for (_, psyche) in world.query::<&Psyche>().iter() {
        total += psyche.maturity;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Upward causation (FR-CIV-EMERGENCE-N10): average kinship across all social ties.
/// Kinship boosts cohesion (family ties stabilize society). Pure `hecs::World` scan.
#[allow(dead_code)] // Reserved for future simulation integration
fn avg_faction_kinship(world: &hecs::World) -> f32 {
    let mut total_kinship = 0.0;
    let mut count = 0u32;
    for (_, graph) in world.query::<&SocialGraph>().iter() {
        for tie in &graph.ties {
            total_kinship += tie.kinship;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total_kinship / count as f32
    }
}

/// Kinship upward boost applied in [`crate::dormant_phases::Simulation::phase_cohesion`].
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn kinship_cohesion_boost(world: &hecs::World) -> i64 {
    const KINSHIP_RATE: f32 = 0.02;
    const KINSHIP_SCALE: f32 = 100_000.0;
    (avg_faction_kinship(world) * KINSHIP_RATE * KINSHIP_SCALE) as i64
}

/// Apply a signed unrest delta, flooring at zero.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn add_unrest_delta(unrest: &mut u64, delta: i64) {
    if delta >= 0 {
        *unrest = unrest.saturating_add(delta as u64);
    } else {
        *unrest = unrest.saturating_sub((-delta) as u64);
    }
}

/// Upward causation (FR-CIV-EMERGENCE-N12): average affinity across all social ties.
/// Positive collective affinity (goodwill) raises the diplomacy conflict threshold;
/// hostility lowers it. Result is clamped to `[-1, 1]`. Pure `hecs::World` scan.
#[allow(dead_code)] // Reserved for future simulation integration
fn avg_social_affinity(world: &hecs::World) -> f32 {
    let mut total = 0.0;
    let mut count = 0u32;
    for (_, graph) in world.query::<&SocialGraph>().iter() {
        for tie in &graph.ties {
            // Defensive clamp: each tie.affinity is maintained in [-1, 1] by the
            // social graph, but clamp per-tie so a malformed save cannot skew the mean.
            total += tie.affinity.clamp(-1.0, 1.0);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        (total / count as f32).clamp(-1.0, 1.0)
    }
}

/// N12: bias magnitude for the affinityâ†’diplomacy threshold (FR-CIV-EMERGENCE-N12).
/// `avg_affinity âˆˆ [-1, 1]` scaled by this yields the threshold bias in `[-5000, 5000]`,
/// bounded below by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` at the combination site.
#[allow(dead_code)] // Reserved for future simulation integration
const N12_AFFINITY_BIAS_SCALE: f32 = 5_000.0;

/// N12: collective affinity threshold bias. Positive goodwill raises the conflict
/// threshold (more tolerance before fighting); hostility lowers it. The input is
/// clamped to `[-1, 1]` so the bias is bounded to `[-5000, 5000]`. Returns i64.
#[allow(dead_code)] // Reserved for future simulation integration
fn affinity_threshold_bias(avg_affinity: f32) -> i64 {
    (avg_affinity.clamp(-1.0, 1.0) * N12_AFFINITY_BIAS_SCALE) as i64
}

/// Maximum peace bonus from shared patron veneration (religion â†’ diplomacy coupling).
#[allow(dead_code)] // Reserved for future simulation integration
const RELIGIOUS_UNITY_PEACE_CAP: i64 = 1_000;

/// RELIGIONâ†’DIPLOMACY emergence coupling (FR-CIV-RELIGION-002).
///
/// A civilisation that has crystallised a patron deity around a shared legend
/// figure gains social cohesion that spills into inter-faction tolerance: the
/// disparity threshold before war is raised by up to
/// [`RELIGIOUS_UNITY_PEACE_CAP`] currency units.
///
/// The bonus is binary (patron present / absent) rather than proportional
/// because the emergence mechanic already encodes significance in *which*
/// figure becomes patron; the coupling here is "shared veneration exists" not
/// "how devout are they".
///
/// Called with an immutable copy of `has_patron` (read before any mutable
/// borrow of `self`) to satisfy the borrow-checker (E0502).
#[allow(dead_code)] // Reserved for future simulation integration
fn religious_unity_peace_bonus(has_patron: bool) -> i64 {
    if has_patron {
        RELIGIOUS_UNITY_PEACE_CAP
    } else {
        0
    }
}

/// Maximum peace bonus from mutual language intelligibility (language â†’ diplomacy coupling).
#[allow(dead_code)] // Reserved for future simulation integration
const LANGUAGE_INTELLIGIBILITY_PEACE_CAP: i64 = 1_200;

/// LANGUAGEâ†’DIPLOMACY emergence coupling.
///
/// Low language distance (mutually intelligible factions) â†’ positive peace bonus
/// on the conflict threshold, mirroring the N9/N12/#564 magnitudes.
/// High distance â†’ 0 bonus (no effect on threshold).
///
/// Called with pre-read centroid values (before any mutable borrow of `self`)
/// to satisfy the borrow-checker (E0502).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn language_intelligibility_peace_bonus(language_distance: f32) -> i64 {
    let raw = LANGUAGE_INTELLIGIBILITY_PEACE_CAP as f32 * (1.0 - language_distance.clamp(0.0, 1.0));
    raw.clamp(0.0, LANGUAGE_INTELLIGIBILITY_PEACE_CAP as f32) as i64
}

/// Upward causation (FR-CIV-0100): the fraction of sentient agents accelerates
/// research (awakened minds discover faster). Reuses the ECS; returns 0..MAX bonus.
#[allow(dead_code)] // Reserved for future simulation integration
fn sentience_research_bonus(world: &hecs::World) -> u64 {
    const MAX_SENTIENCE_RESEARCH: u64 = 50;
    // Mirrors `EmergenceState::new` sentience profile and threshold.
    let profile = CognitionTraitProfile::new(
        "sapient-lineage",
        vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
    );
    let threshold = SentienceThreshold::new(0.72);
    let (sentient, total) = world
        .query::<&Dna>()
        .iter()
        .fold((0u32, 0u32), |(s, n), (_, dna)| {
            let crossed = cognition_score(dna, &profile) >= threshold.minimum_cognition;
            (s + u32::from(crossed), n + 1)
        });
    if total == 0 {
        return 0;
    }
    let fraction = sentient as f32 / total as f32;
    ((fraction * MAX_SENTIENCE_RESEARCH as f32) as u64).min(MAX_SENTIENCE_RESEARCH)
}

/// The economic focus a civilization tends toward, from its strongest sector.
#[allow(dead_code)] // Reserved for future simulation integration
fn candidate_economic_focus(
    food: i64,
    research_tier: u64,
    belief: u64,
    treasury_total: i64,
) -> EconomicFocus {
    let agr = food;
    let ind = (research_tier as i64) * 50_000;
    let sac = (belief / 4) as i64;
    let mer = treasury_total / 4;
    let max = agr.max(ind).max(sac).max(mer);
    if max <= 0 {
        return EconomicFocus::Balanced;
    }
    if max == agr {
        EconomicFocus::Agrarian
    } else if max == ind {
        EconomicFocus::Industrial
    } else if max == sac {
        EconomicFocus::Sacred
    } else {
        EconomicFocus::Mercantile
    }
}

/// Downward-causation policy (FR-CIV-0100 Â§3): research raises production yield â€”
/// better tools/techniques lift per-building output. +10% per research tier,
/// capped at +100% (2x). De-silos phase_production, which read no emergent state.
#[allow(dead_code)] // Reserved for future simulation integration
fn production_yield_factor(research_tier: u64) -> Fixed {
    let bonus_permille = research_tier.saturating_mul(100).min(1_000) as i64;
    Fixed::from_num(1_000 + bonus_permille) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-CONTENT-001): terrain biome modulates food
/// production â€” fertile land grows more food, barren land grows less.  The factor
/// is a pure multiplier on per-farm output; caller multiplies food output by this.
/// Returns a value in the range [0.1, 1.5] (clamped).
#[allow(dead_code)] // Reserved for future simulation integration
fn biome_yield_factor(biome: BiomeKind) -> Fixed {
    match biome {
        BiomeKind::Rainforest => Fixed::from_num(13) / Fixed::from_num(10),
        BiomeKind::Wetland => Fixed::from_num(12) / Fixed::from_num(10),
        BiomeKind::Grassland => Fixed::from_num(12) / Fixed::from_num(10),
        BiomeKind::Plains => Fixed::from_num(11) / Fixed::from_num(10),
        BiomeKind::Forest => Fixed::from_num(9) / Fixed::from_num(10),
        BiomeKind::Savanna => Fixed::from_num(17) / Fixed::from_num(20),
        BiomeKind::Beach => Fixed::from_num(8) / Fixed::from_num(10),
        BiomeKind::Mountain => Fixed::from_num(6) / Fixed::from_num(10),
        BiomeKind::Taiga => Fixed::from_num(6) / Fixed::from_num(10),
        BiomeKind::Desert => Fixed::from_num(1) / Fixed::from_num(2),
        BiomeKind::Tundra => Fixed::from_num(9) / Fixed::from_num(20),
        BiomeKind::Ocean => Fixed::from_num(1) / Fixed::from_num(5),
        BiomeKind::Glacier => Fixed::from_num(1) / Fixed::from_num(10),
        BiomeKind::Shrubland => Fixed::from_num(8) / Fixed::from_num(10),
        BiomeKind::Steppe => Fixed::from_num(7) / Fixed::from_num(10),
        BiomeKind::Alpine => Fixed::from_num(5) / Fixed::from_num(10),
        _ => Fixed::from_num(1),
    }
}

/// Aggregate biome yield factor over a slice of [`BiomeKind`]s.
///
/// Returns the mean `biome_yield_factor` across the slice, clamped to
/// `[0.1, 1.5]`.  Returns `Fixed::ONE` (neutral) for an empty slice so
/// callers with no geology data are unaffected.
#[allow(dead_code)] // Reserved for future simulation integration
fn aggregate_biome_yield(biomes: &[BiomeKind]) -> Fixed {
    if biomes.is_empty() {
        return Fixed::from_num(1) / Fixed::from_num(1);
    }
    let sum = biomes
        .iter()
        .fold(Fixed::ZERO, |acc, &b| acc + biome_yield_factor(b));
    let mean = sum / Fixed::from_num(biomes.len() as i64);
    let lo = Fixed::from_num(1) / Fixed::from_num(10);
    let hi = Fixed::from_num(15) / Fixed::from_num(10);
    mean.clamp(lo, hi)
}

/// Downward-causation policy (FR-CIV-0100 Â§3): social cohesion speeds military
/// morale recovery â€” a unified society's troops rally faster. Returns the
/// per-tick morale recovery increment, rising with cohesion from a 0.010 base
/// up to a 0.050 cap.
#[allow(dead_code)] // Reserved for future simulation integration
fn morale_recovery_rate(cohesion: u64) -> Fixed {
    const BASE_PERMILLE: i64 = 10;
    const CAP_PERMILLE: i64 = 50;
    let bonus = (cohesion / 25_000).min((CAP_PERMILLE - BASE_PERMILLE) as u64) as i64;
    Fixed::from_num(BASE_PERMILLE + bonus) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-0100 Â§3): overcrowding breeds unrest
/// (Malthusian pressure). Population beyond the carrying capacity adds unrest
/// scaled by the percentage overshoot (10% over => +1), capped per tick. A
/// third unrest driver alongside food scarcity and energy blackout.
#[allow(dead_code)] // Reserved for future simulation integration
fn overcrowding_unrest(population: u64, capacity: i64) -> i64 {
    const MAX_OVERCROWD_UNREST: i64 = 30;
    let cap = capacity.max(1) as u64;
    if population <= cap {
        return 0;
    }
    let overshoot_pct = ((population - cap).saturating_mul(100) / cap).min(i64::MAX as u64) as i64;
    (overshoot_pct / 10).clamp(1, MAX_OVERCROWD_UNREST)
}

/// Downward-causation policy (FR-CIV-0100 Â§3): social cohesion accelerates
/// research â€” a unified society collaborates. Returns a per-mille bonus to the
/// per-tick research contribution, up to +50%.
#[allow(dead_code)] // Reserved for future simulation integration
fn cohesion_research_bonus_permille(cohesion: u64) -> u64 {
    (cohesion / 2_000).min(500)
}

/// The wealth gap (in whole currency units) between the richest and poorest
/// faction â€” an emergent measure of structural inequality across the society.
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_treasury_spread(treasury: &HashMap<u32, Fixed>) -> i64 {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    for t in treasury.values() {
        let v = i64::from(t.to_bits()) / crate::SCALE;
        min = min.min(v);
        max = max.max(v);
    }
    if max >= min {
        max - min
    } else {
        0
    }
}

/// Downward-causation policy (FR-CIV-0100 Â§3): structural inequality breeds class
/// unrest. A wide wealth gap between factions adds unrest scaled by the gap,
/// capped per tick. Distinct from scarcity â€” this is about distribution.
#[allow(dead_code)] // Reserved for future simulation integration
fn inequality_unrest(treasury_spread: i64) -> i64 {
    const MAX_INEQUALITY_UNREST: i64 = 25;
    const SPREAD_PER_UNREST: i64 = 2_000;
    (treasury_spread / SPREAD_PER_UNREST).clamp(0, MAX_INEQUALITY_UNREST)
}

/// The dispossessed share (per-mille) that a society TENDS TOWARD given its
/// wealth gap and social fabric: inequality pushes it up, cohesion pulls it
/// down. Clamped to [0, 1000].
#[allow(dead_code)] // Reserved for future simulation integration
fn dispossession_target_permille(treasury_spread: i64, cohesion: u64) -> u64 {
    const SPREAD_PER_PERMILLE: i64 = 200; // currency-units of gap per +1 permille
    let from_inequality = (treasury_spread.max(0) / SPREAD_PER_PERMILLE) as u64;
    let from_cohesion = cohesion / 5_000; // cohesion erodes dispossession
    from_inequality.saturating_sub(from_cohesion).min(1_000)
}

/// Max institution level (criticality cap on the belief->temple->belief loop).
#[allow(dead_code)] // Reserved for future simulation integration
pub const MAX_INSTITUTION_LEVEL: u32 = 5;

/// Institution level that a driver signal supports: one level per THRESHOLD of
/// the signal, capped at MAX_INSTITUTION_LEVEL.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn institution_target_level(signal: u64, per_level: u64) -> u32 {
    (signal / per_level.max(1)).min(MAX_INSTITUTION_LEVEL as u64) as u32
}

/// One-step decay toward target (max 1 level change per tick, so growth/decay
/// is gradual â€” hysteresis).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn institution_step(current: u32, target: u32) -> u32 {
    if target > current {
        current + 1
    } else if target < current {
        current - 1
    } else {
        current
    }
}

/// One sticky step of the dispossessed share toward its target (max 5 permille
/// per tick), so the class structure persists rather than tracking instantly.
#[allow(dead_code)] // Reserved for future simulation integration
fn dispossession_step(current: u64, target: u64) -> u64 {
    const MAX_STEP: u64 = 5;
    if target > current {
        (current + MAX_STEP.min(target - current)).min(1_000)
    } else {
        current - MAX_STEP.min(current - target)
    }
}

/// A large dispossessed underclass adds unrest, scaled by its share, capped.
#[allow(dead_code)] // Reserved for future simulation integration
fn dispossession_unrest(dispossessed_permille: u64) -> i64 {
    (dispossessed_permille / 40).min(25) as i64
}

/// Downward-causation policy (FR-CIV-0100 Â§3 emergence): research mitigates
/// unrest â€” advanced food logistics (storage, distribution) blunt the
/// scarcity-driven rise. Only the positive (rising) part is damped; decay is
/// untouched. The mitigation is bounded (tier capped at 9 â†’ at most a 10x
/// reduction) and floored at 1, so technology calms a society but never makes
/// it immune to hardship. Returns the research-adjusted unrest delta.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn research_unrest_mitigation(rise: i64, research_tier: u64) -> i64 {
    if rise <= 0 {
        return rise;
    }
    let divisor = 1 + research_tier.min(9) as i64;
    (rise / divisor).max(1)
}

/// Downward-causation policy (FR-CIV-0100 Â§3): research accelerates construction.
/// Each research tier shortens the build cadence (ticks between expansions),
/// floored so an advanced civilisation never busy-builds every single tick.
/// De-silos phase_buildings, which previously read no emergent state.
#[allow(dead_code)] // Reserved for future simulation integration
fn building_cadence(research_tier: u64) -> u64 {
    const BASE: u64 = 16;
    const FLOOR: u64 = 4;
    BASE.saturating_sub(research_tier.saturating_mul(2))
        .max(FLOOR)
}

/// Emergent construction demand (FR-CIV-0100 Â§3): the built environment responds
/// to society â€” crowding drives housing, research drives industry, cohesion
/// drives commerce, unrest drives civic/governance building. Each in [0,1].
/// All channels are scaled by wood/metal headroom so construction stops when
/// stockpiles are depleted (FC-3).
#[allow(dead_code)] // Reserved for future simulation integration
fn building_demand_signals(
    population: u64,
    capacity: i64,
    cohesion: u64,
    research_tier: u64,
    unrest: u64,
    wood: Fixed,
    metal: Fixed,
) -> DemandSignals {
    let cap = capacity.max(1) as f32;
    let cohesion_signal = ((cohesion as f32) / 1_000_000.0).clamp(0.0, 1.0);
    let wood_permille =
        building_material_headroom_permille(wood, BUILDING_WOOD_PER_PARCEL, BUILDING_MATERIAL_GATE);
    let metal_permille = building_material_headroom_permille(
        metal,
        BUILDING_METAL_PER_PARCEL,
        BUILDING_MATERIAL_GATE,
    );
    let material_permille = wood_permille.min(metal_permille);
    let material_factor = material_permille as f32 / 1000.0;
    DemandSignals {
        residential: ((population as f32) / cap).clamp(0.0, 1.0) * material_factor,
        commercial: cohesion_signal * material_factor,
        industrial: ((research_tier as f32) / 5.0).clamp(0.0, 1.0) * material_factor,
        civic: ((unrest as f32) / 500.0).clamp(0.0, 1.0) * material_factor,
    }
}

/// Wood consumed per parcel allocated in [`Simulation::phase_buildings`].
#[allow(dead_code)] // Reserved for future simulation integration
const BUILDING_WOOD_PER_PARCEL: i64 = 10;
/// Metal consumed per parcel allocated in [`Simulation::phase_buildings`].
#[allow(dead_code)] // Reserved for future simulation integration
const BUILDING_METAL_PER_PARCEL: i64 = 5;
/// Stock level (integer units) at which material headroom reaches full strength.
#[allow(dead_code)] // Reserved for future simulation integration
const BUILDING_MATERIAL_GATE: i64 = 500;
#[allow(dead_code)] // Reserved for future simulation integration
const FC3_COMMERCIAL_PARCEL_THRESHOLD: f32 = 0.5;

/// FC-3: reserve one parcel, then quadratic roll-off in permille (0..=1000).
#[allow(dead_code)] // Reserved for future simulation integration
fn building_material_headroom_permille(stock: Fixed, reserve_units: i64, gate_units: i64) -> u64 {
    let reserve = Fixed::from_num(reserve_units);
    let effective = stock.saturating_sub(reserve);
    if effective.to_bits() <= 0 {
        return 0;
    }
    let gate = Fixed::from_num(gate_units);
    let linear =
        ((effective.to_bits() as i128) * 1000 / gate.to_bits().max(1) as i128).min(1000) as u64;
    linear.saturating_mul(linear) / 1000
}

/// Parcels fundable from current wood and metal stockpiles (integer division).
#[allow(dead_code)] // Reserved for future simulation integration
fn building_affordable_parcel_count(wood: Fixed, metal: Fixed) -> usize {
    let wood_per = Fixed::from_num(BUILDING_WOOD_PER_PARCEL);
    let metal_per = Fixed::from_num(BUILDING_METAL_PER_PARCEL);
    let by_wood = if wood_per.to_bits() > 0 {
        (wood.to_bits() / wood_per.to_bits()) as usize
    } else {
        usize::MAX
    };
    let by_metal = if metal_per.to_bits() > 0 {
        (metal.to_bits() / metal_per.to_bits()) as usize
    } else {
        usize::MAX
    };
    by_wood.min(by_metal)
}

/// Keeps the highest-priority saturated signals, zeroing the rest.
#[allow(dead_code)] // Reserved for future simulation integration
fn building_signals_limited(signals: DemandSignals, max_parcels: usize) -> DemandSignals {
    let mut active = [
        (0_u8, signals.residential),
        (1, signals.commercial),
        (2, signals.industrial),
        (3, signals.civic),
    ]
    .into_iter()
    .filter(|(_, strength)| *strength > 0.5)
    .collect::<Vec<_>>();
    active.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    active.truncate(max_parcels);

    let mut out = DemandSignals {
        residential: 0.0,
        commercial: 0.0,
        industrial: 0.0,
        civic: 0.0,
    };
    for (kind, strength) in active {
        match kind {
            0 => out.residential = strength,
            1 => out.commercial = strength,
            2 => out.industrial = strength,
            _ => out.civic = strength,
        }
    }
    out
}

/// FC-3 metal steady-state ceiling (integer metal units) for a cohesion level.
/// Includes two parcel debits of headroom for discrete cadence oscillation.
#[allow(dead_code)] // Reserved for future simulation integration
fn fc3_commercial_metal_steady_ceiling_i64(cohesion: u64) -> i64 {
    let cohesion_signal = ((cohesion as f32) / 1_000_000.0).clamp(0.0, 1.0);
    if cohesion_signal <= FC3_COMMERCIAL_PARCEL_THRESHOLD {
        return i64::MAX;
    }
    let m_star = (BUILDING_MATERIAL_GATE as f32)
        * (FC3_COMMERCIAL_PARCEL_THRESHOLD / cohesion_signal).sqrt();
    (m_star + (BUILDING_METAL_PER_PARCEL as f32) * 2.0).ceil() as i64
}

/// Default sentience profile used by [`Simulation::phase_sentience`].
/// Companion of `Simulation::default_sentience_profile` (associated stub).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn default_sentience_profile() -> CognitionTraitProfile {
    CognitionTraitProfile::new(
        "sapient-lineage",
        vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
    )
}

/// Parcels that would be allocated for saturated demand signals (> 0.5).
#[allow(dead_code)] // Reserved for future simulation integration
fn building_parcel_count(signals: &DemandSignals) -> usize {
    [
        signals.residential,
        signals.commercial,
        signals.industrial,
        signals.civic,
    ]
    .iter()
    .filter(|&&signal| signal > 0.5)
    .count()
}

/// Construction material debit for `parcel_count` new parcels.
#[allow(dead_code)] // Reserved for future simulation integration
fn building_material_cost(parcel_count: usize) -> (Fixed, Fixed) {
    let n = parcel_count as i64;
    (
        Fixed::from_num(BUILDING_WOOD_PER_PARCEL * n),
        Fixed::from_num(BUILDING_METAL_PER_PARCEL * n),
    )
}

/// True when the global stockpile can fund `parcel_count` new parcels.
/// De-silos `resources.wood` / `resources.metal`, which `phase_production` writes.
#[allow(dead_code)] // Reserved for future simulation integration
fn building_materials_affordable(wood: Fixed, metal: Fixed, parcel_count: usize) -> bool {
    let (need_wood, need_metal) = building_material_cost(parcel_count);
    wood >= need_wood && metal >= need_metal
}

/// Belief units that contribute one unit of cohesion growth per tick.
#[allow(dead_code)] // Reserved for future simulation integration
const COHESION_BELIEF_DIVISOR: u64 = 200;
/// Unrest units that fray one unit of cohesion per tick.
#[allow(dead_code)] // Reserved for future simulation integration
const COHESION_UNREST_DIVISOR: u64 = 50;

/// Emergence policy (FR-CIV-0100 Â§3): the social fabric's per-tick change is the
/// balance of belief (binds, scaled gently) against unrest (frays, scaled
/// harder, so disorder erodes cohesion faster than faith builds it). Returns a
/// signed delta; the caller floors the running total at zero.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn cohesion_delta(belief: u64, unrest: u64) -> i64 {
    let bind = (belief / COHESION_BELIEF_DIVISOR) as i64;
    let fray = (unrest / COHESION_UNREST_DIVISOR) as i64;
    bind - fray
}

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: each lineage crossing the sentience
/// threshold this tick mints a small bounded pulse of cohesion (shared
/// identity â€” "we are the people who woke"). Kept SMALL relative to the
/// existing cohesion bind/frac inputs so the moment of awakening nudges
/// the social fabric without dominating it; the per-tick cap mirrors the
/// spirit of [`crate::emergence`] emergence caps.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const COHESION_PER_AWAKENING: i64 = 2;
/// Hard per-tick cap on awakening-driven cohesion nudge (signed i64 so the
/// existing floored-at-zero cohesion mutator absorbs any overshoot cleanly).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const MAX_AWAKENING_COHESION_PER_TICK: i64 = 10;
/// FR-CIV-GENETICS / FR-CIV-LEGENDS: pure gain fn for the awakening -> cohesion
/// pulse. Returns a signed i64 (matches `cohesion_delta`'s contract). The
/// inner product is clamped to the per-tick cap.
#[allow(dead_code)] // Reserved for future simulation integration
#[must_use]
pub fn awakening_cohesion_gain(awakenings_this_tick: usize) -> i64 {
    let raw = (awakenings_this_tick as i64).saturating_mul(COHESION_PER_AWAKENING);
    raw.min(MAX_AWAKENING_COHESION_PER_TICK).max(0)
}

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: pure gain fn for the awakening -> belief
/// pulse. Returns a signed i64 and clamps to a small per-tick cap so the
/// compatibility shim stays bounded and deterministic.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const BELIEF_PER_AWAKENING: i64 = 2;
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const MAX_AWAKENING_BELIEF_PER_TICK: i64 = 10;

#[allow(dead_code)] // Reserved for future simulation integration
#[must_use]
pub(crate) fn awakening_belief_gain(awakenings_this_tick: usize) -> i64 {
    let raw = (awakenings_this_tick as i64).saturating_mul(BELIEF_PER_AWAKENING);
    raw.min(MAX_AWAKENING_BELIEF_PER_TICK).max(0)
}

/// Cohesion absorbs hardship: a strong social fabric damps the per-tick unrest
/// rise (cohesion -> calmer society), bounded and floored at 1. Decay passes through.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn cohesion_unrest_damp(rise: i64, cohesion: u64) -> i64 {
    if rise <= 0 {
        return rise;
    }
    let divisor = 1 + (cohesion / 200).min(9) as i64;
    (rise / divisor).max(1)
}

/// Surplus differential (resource units) at/above which a route ships its full
/// boosted volume.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_GAP_SCALE: i64 = 100;

/// Arbitrage policy (FR-CIV-0100 Â§3 emergence): trade volume scales with the
/// surplus gap between exporter and importer â€” a well-stocked source feeding a
/// scarce destination ships MORE. Returns a multiplier in `[1.0, 2.0]`, bounded
/// at 2x so the priceâ†”volumeâ†”treasuryâ†”demand loop self-limits rather than
/// running away (design-layer criticality bound). No boost when the source is
/// not in surplus relative to the destination.
#[allow(dead_code)] // Reserved for future simulation integration
fn trade_volume_multiplier(from_stock: Fixed, to_stock: Fixed) -> Fixed {
    let gap = (from_stock - to_stock).max(Fixed::ZERO);
    let normalized = (gap / Fixed::from_num(TRADE_GAP_SCALE)).min(Fixed::from_num(1));
    Fixed::from_num(1) + normalized
}

/// Floor (per-mille) below which unrest cannot throttle trade â€” even a society
/// in turmoil keeps half its commerce moving.
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_TRADE_FLOOR_PERMILLE: i64 = 500;
/// Units of standing unrest that throttle trade by one per-mille.
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_PER_TRADE_PERMILLE: u64 = 4;

/// Downward-causation policy (FR-CIV-0100 Â§3 emergence): societal unrest
/// disrupts commerce. Returns a trade-volume factor in `[0.5, 1.0]` â€” `1.0`
/// when calm, declining as unrest rises but floored at half so trade never
/// stops entirely. Makes unrest act on BOTH diplomacy (war) and the economy.
#[allow(dead_code)] // Reserved for future simulation integration
fn unrest_trade_factor(unrest: u64) -> Fixed {
    let max_drop = (1_000 - UNREST_TRADE_FLOOR_PERMILLE) as u64;
    let drop = (unrest / UNREST_PER_TRADE_PERMILLE).min(max_drop) as i64;
    Fixed::from_num(1_000 - drop) / Fixed::from_num(1_000)
}

/// Cohesion units that lift trade volume by one per-mille (social trust greases commerce).
#[allow(dead_code)] // Reserved for future simulation integration
const COHESION_PER_TRADE_PERMILLE: u64 = 4;
/// Cap on cohesion's trade boost (per-mille above 1.0): at most +50% volume.
#[allow(dead_code)] // Reserved for future simulation integration
const COHESION_TRADE_CAP_PERMILLE: i64 = 500;
/// Per-mille trade boost from agent tie trust alone.
#[allow(dead_code)] // Reserved for future simulation integration
const MICRO_TRUST_CAP_PERMILLE: u64 = 250;
/// Combined macro+micro trade boost cap (cohesion 500 + micro 250).
#[allow(dead_code)] // Reserved for future simulation integration
const SOCIETY_TRADE_BOOST_CAP_PERMILLE: i64 = 750;

/// Downward-causation policy (FR-CIV-0100 Â§3): macro cohesion AND cached micro
/// interpersonal trust lift trade volume. Returns factor in [1.0, 1.75].
#[allow(dead_code)] // Reserved for future simulation integration
fn society_trade_factor(cohesion: u64, micro_trust_permille: u64) -> Fixed {
    let cohesion_boost =
        (cohesion / COHESION_PER_TRADE_PERMILLE).min(COHESION_TRADE_CAP_PERMILLE as u64) as i64;
    let micro_boost = micro_trust_permille.min(MICRO_TRUST_CAP_PERMILLE) as i64;
    let total = (cohesion_boost + micro_boost).min(SOCIETY_TRADE_BOOST_CAP_PERMILLE);
    Fixed::from_num(1_000 + total) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-0100 Â§3): a cohesive society trades MORE â€”
/// social trust lowers transaction friction. Returns a factor in [1.0, 1.5],
/// rising with cohesion, capped so the boost can't run away.
#[allow(dead_code)] // Reserved for future simulation integration
fn cohesion_trade_factor(cohesion: u64) -> Fixed {
    society_trade_factor(cohesion, 0)
}

/// Relations bias trade: allies (positive relation) trade more, rivals (negative)
/// less. Returns a factor in [0.5, 1.5] from a relation score in [-1, 1], bounded.
#[allow(dead_code)] // Reserved for future simulation integration
fn relation_trade_factor(relation: f32) -> Fixed {
    let r = relation.clamp(-1.0, 1.0);
    // map [-1,1] to per-mille [500, 1500], then to a Fixed factor in [0.5, 1.5].
    let permille = (1_000.0 + 500.0 * r) as i64;
    Fixed::from_num(permille) / Fixed::from_num(1_000)
}

/// Max per-mille reduction from language barrier (at distance = 1.0).
#[allow(dead_code)] // Reserved for future simulation integration
const LANGUAGE_TRADE_PENALTY_PERMILLE: i64 = 500;
/// Downward-causation (FR-CIV-LANG-001 / FR-CIV-PSYCHE-912): mutually unintelligible
/// languages impose transaction friction. Returns factor in [0.5, 1.0].
#[allow(dead_code)] // Reserved for future simulation integration
fn language_trade_factor(distance: f32) -> Fixed {
    let d = distance.clamp(0.0, 1.0);
    let permille = 1_000 - (d * LANGUAGE_TRADE_PENALTY_PERMILLE as f32).round() as i64;
    Fixed::from_num(permille) / Fixed::from_num(1_000)
}

/// Wealth-disparity (in whole currency units) at which two factions clash when
/// they share no faith. Above this gap the have-nots turn on the haves.
#[allow(dead_code)] // Reserved for future simulation integration
const DIPLOMACY_BASE_CONFLICT_THRESHOLD: i64 = 10_000;
/// Trade-agreement relation drift (+0.05) via [`DiplomacyMatrix`] trade channel.
#[allow(dead_code)] // Reserved for future simulation integration
const FACTION_TRADE_RELATION_SIGNAL: f32 = 0.05 / 0.08;
/// Conflict relation drift (-0.1) via [`DiplomacyMatrix`] competition channel.
#[allow(dead_code)] // Reserved for future simulation integration
const FACTION_CONFLICT_RELATION_SIGNAL: f32 = 0.1 / 0.12;
/// Per diplomacy phase, unstrengthened relations retain this fraction of magnitude.
#[allow(dead_code)] // Reserved for future simulation integration
const FACTION_RELATION_DECAY_FACTOR: f32 = 0.98;
/// Trade drift per unit signal in [`DiplomacyMatrix::apply_signal`].
#[allow(dead_code)] // Reserved for future simulation integration
const DIPLOMACY_TRADE_DRIFT: f32 = 0.08;
/// Competition drift per unit signal in [`DiplomacyMatrix::apply_signal`].
#[allow(dead_code)] // Reserved for future simulation integration
const DIPLOMACY_COMPETITION_DRIFT: f32 = 0.12;
/// Max threshold shift from a saturated pairwise relation score (`Â±1.0`).
#[allow(dead_code)] // Reserved for future simulation integration
const FACTION_RELATION_THRESHOLD_SPAN: i64 = 5_000;
/// Max peace bonus from identical pairwise cultural traits (N2 coupling).
#[allow(dead_code)] // Reserved for future simulation integration
const CULTURE_PEACE_SPAN: f32 = 3_000.0;
/// Minimum members for an emergent settlement (matches `phase_life` HUD filter).
#[allow(dead_code)] // Reserved for future simulation integration
const SETTLEMENT_MIN_MEMBERS: u32 = 2;
/// Co-location radius for emergent settlements (matches `phase_life` cluster radius).
#[allow(dead_code)] // Reserved for future simulation integration
const SETTLEMENT_CLUSTER_RADIUS_FP: i64 = (6 * FIXED_SCALE) / 100;
/// Contact radius between settlement pairs (2Ã— cluster radius).
#[allow(dead_code)] // Reserved for future simulation integration
const SETTLEMENT_CONTACT_RADIUS_FP: i64 = SETTLEMENT_CLUSTER_RADIUS_FP * 2;

#[allow(dead_code)] // Reserved for settlement membership analysis
struct SettlementMembershipPayoff<'a> {
    stock_by_cluster: &'a BTreeMap<u64, ClusterStocks>,
}

#[allow(dead_code)] // Reserved for settlement membership analysis
impl civ_agents::cluster::MembershipPayoff for SettlementMembershipPayoff<'_> {
    fn payoff(&self, _agent_id: u64, cluster: ClusterId) -> f32 {
        let food = self
            .stock_by_cluster
            .get(&cluster.0)
            .map(|stocks| stocks.get(civ_economy::Good::Food))
            .unwrap_or(0);
        (food as f32 / 10.0).clamp(-1.0, 1.0)
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_actors_by_settlement(
    actor_settlement: &BTreeMap<u64, u32>,
) -> BTreeMap<u32, BTreeSet<u64>> {
    let mut by_settlement: BTreeMap<u32, BTreeSet<u64>> = BTreeMap::new();
    for (&actor_id, &settlement_id) in actor_settlement {
        by_settlement
            .entry(settlement_id)
            .or_default()
            .insert(actor_id);
    }
    by_settlement
}

#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_centroid_position(world: &World, settlement_id: u64) -> Option<Position3d> {
    let mut count = 0_i64;
    let mut sx = 0_i128;
    let mut sy = 0_i128;
    let mut sz = 0_i128;
    for (_, (member, pos)) in world.query::<(&ClusterMember, &Position3d)>().iter() {
        if member.cluster.0 == settlement_id {
            count += 1;
            sx += i128::from(pos.coord.x);
            sy += i128::from(pos.coord.y);
            sz += i128::from(pos.coord.z);
        }
    }
    (count > 0).then(|| Position3d {
        coord: WorldCoord {
            x: (sx / i128::from(count)) as i64,
            y: (sy / i128::from(count)) as i64,
            z: (sz / i128::from(count)) as i64,
        },
    })
}

/// Belief units required to raise the conflict threshold by one currency unit.
#[allow(dead_code)] // Reserved for future simulation integration
const BELIEF_PEACE_DIVISOR: u64 = 50;
/// Cap on the belief-driven peace bonus: shared faith can at most double a
/// society's tolerance for inequality â€” it never makes conflict impossible.
#[allow(dead_code)] // Reserved for future simulation integration
const BELIEF_PEACE_CAP: i64 = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
/// Unrest units required to erode the conflict threshold by one currency unit.
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_WAR_DIVISOR: u64 = 50;
/// Cap on how much unrest can erode the threshold (currency units).
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_WAR_CAP: i64 = 8_000;
/// Floor on the conflict threshold: even a furious, faithless society still
/// needs SOME wealth disparity to go to war â€” discontent alone is not casus belli.
#[allow(dead_code)] // Reserved for future simulation integration
const DIPLOMACY_MIN_CONFLICT_THRESHOLD: i64 = 2_000;

/// Downward-causation policy (FR-CIV-0100 Â§3 emergence): collective belief and
/// societal unrest pull diplomacy in opposite directions. Shared faith RAISES
/// the wealth-disparity a faction pair tolerates before fighting (peace);
/// unrest LOWERS it (internal discontent spills into external aggression). The
/// threshold is bounded below by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` so conflict
/// always needs some disparity, and above at `2x` base so peace is never absolute.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn diplomacy_conflict_threshold(belief: u64, unrest: u64) -> i64 {
    let peace = (belief / BELIEF_PEACE_DIVISOR).min(BELIEF_PEACE_CAP as u64) as i64;
    let war = (unrest / UNREST_WAR_DIVISOR).min(UNREST_WAR_CAP as u64) as i64;
    (DIPLOMACY_BASE_CONFLICT_THRESHOLD + peace - war).max(DIPLOMACY_MIN_CONFLICT_THRESHOLD)
}

/// Cohesion-driven peace bonus for diplomacy threshold (FR-CIV-RELIGION-002).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn cohesion_peace_bonus(cohesion: u64) -> i64 {
    (cohesion / COHESION_BELIEF_DIVISOR).min(BELIEF_PEACE_CAP as u64 / 2) as i64
}

/// Combined religionâ†’diplomacy threshold: belief, cohesion, unrest, patron veneration.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn diplomacy_peace_threshold(belief: u64, cohesion: u64, unrest: u64, has_patron: bool) -> i64 {
    diplomacy_conflict_threshold(belief, unrest)
        + cohesion_peace_bonus(cohesion)
        + religious_unity_peace_bonus(has_patron)
}

/// Macro belief plus emergent cluster doctrine strength (FR-CIV-RELIGION / REL-003).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn institution_belief_signal(
    macro_belief: u64,
    cluster_beliefs: &BTreeMap<u64, [f32; 4]>,
) -> u64 {
    if cluster_beliefs.is_empty() {
        return macro_belief;
    }
    let cluster_pulse: u64 = cluster_beliefs
        .values()
        .map(|centroid| (centroid[0] * 2_000.0) as u64)
        .sum();
    macro_belief.saturating_add(cluster_pulse / cluster_beliefs.len() as u64)
}

/// Cluster-divergence temple boost: isolated clusters that develop distinct doctrines
/// independently accelerate local institution growth (FR-CIV-RELIGION cluster divergence).
///
/// `divergence` is the max pairwise belief distance `[0.0, 1.0]` from
/// [`civ_agents::max_cluster_belief_divergence`]. Returns extra belief units to
/// add to the macro signal, capped so the boost can at most double the signal.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn institution_divergence_boost(macro_signal: u64, divergence: f32) -> u64 {
    let bonus = (macro_signal as f32 * divergence.clamp(0.0, 1.0)) as u64;
    macro_signal.saturating_add(bonus)
}

/// Pairwise treasury gap between two factions (currency units).
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_pair_treasury_disparity(treasury: &HashMap<u32, Fixed>, a: u32, b: u32) -> i64 {
    let va = treasury
        .get(&a)
        .map(|t| t.to_bits() / crate::SCALE)
        .unwrap_or(0);
    let vb = treasury
        .get(&b)
        .map(|t| t.to_bits() / crate::SCALE)
        .unwrap_or(0);
    (va - vb).abs()
}

/// N9: maximum reduction to the conflict threshold from maximum aggression.
const AGGRESSION_CONFLICT_BOOST: i64 = 3_000;

/// N9: conflict-threshold reduction driven by mean pairwise aggression.
/// Aggressive species are quicker to fight: a mean aggression of 1.0 reduces
/// the threshold by [`AGGRESSION_CONFLICT_BOOST`] currency units.
#[allow(dead_code)] // Reserved for future simulation integration
fn aggression_threshold_reduction(mean: f32) -> i64 {
    (mean.clamp(0.0, 1.0) * AGGRESSION_CONFLICT_BOOST as f32) as i64
}

/// Threshold bias from emergent faction relation (`relation * 5000`, clamped).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_relation_threshold_bias(relation_score: f32) -> i64 {
    (relation_score.clamp(-1.0, 1.0) * FACTION_RELATION_THRESHOLD_SPAN as f32).round() as i64
}

/// Peace bonus from pairwise cultural similarity (N2 â€” culture â†’ diplomacy).
///
/// Culturally similar factions tolerate more treasury disparity before conflict;
/// divergent pairs add zero bonus (neutral default).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_culture_threshold_bias(
    cultures: &BTreeMap<u64, CultureProfile>,
    faction_a: u32,
    faction_b: u32,
) -> i64 {
    let Some(pa) = cultures.get(&u64::from(faction_a)) else {
        return 0;
    };
    let Some(pb) = cultures.get(&u64::from(faction_b)) else {
        return 0;
    };
    let distance = cultural_distance(pa.traits, pb.traits);
    let similarity = 1.0 - distance;
    (similarity * CULTURE_PEACE_SPAN).round() as i64
}

/// Dominant explicit faction alignment per multi-member settlement cluster (N3).
#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_dominant_factions(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u64, u32> {
    let mut faction_counts: BTreeMap<u64, BTreeMap<u32, u32>> = BTreeMap::new();
    for (_, (civ, member)) in world.query::<(&AgentCivilian, &ClusterMember)>().iter() {
        let cluster_id = member.cluster.0;
        let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
        if members < SETTLEMENT_MIN_MEMBERS {
            continue;
        }
        if let Alignment::Faction(faction_id) = civ.alignment {
            *faction_counts
                .entry(cluster_id)
                .or_default()
                .entry(faction_id)
                .or_insert(0) += 1;
        }
    }

    let mut dominant = BTreeMap::new();
    for (cluster_id, counts) in faction_counts {
        let mut best_faction = None;
        let mut best_count = 0u32;
        for (&faction_id, &count) in &counts {
            let replace = match best_faction {
                None => true,
                Some(prev) => count > best_count || (count == best_count && faction_id < prev),
            };
            if replace {
                best_faction = Some(faction_id);
                best_count = count;
            }
        }
        if let Some(faction_id) = best_faction {
            dominant.insert(cluster_id, faction_id);
        }
    }
    dominant
}

/// Member-weighted per-faction language centroid (FR-CIV-LANG-001 / FR-CIV-PSYCHE-912).
///
/// `cluster_cultures` is `BTreeMap<u64, CultureProfile>` keyed by cluster id; each
/// profile carries a `language: [f32; 4]` vector. `dominant` maps cluster id
/// (u64) to its dominant faction id (u32) as returned by
/// [`settlement_dominant_factions`]. `member_counts` is the cluster membership
/// rollup from `phase_life`. Clusters with fewer than 2 members are ignored so
/// lone wanderers cannot anchor a faction's centroid.
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_language_centroids(
    cultures: &std::collections::BTreeMap<u64, CultureProfile>,
    dominant: &std::collections::BTreeMap<u64, u32>,
    member_counts: &std::collections::BTreeMap<u64, u32>,
) -> std::collections::BTreeMap<u32, [f32; 4]> {
    let mut sums: std::collections::BTreeMap<u32, ([f32; 4], f32)> = Default::default();
    for (cluster_id, faction_id) in dominant {
        let mc = match member_counts.get(cluster_id) {
            Some(&m) if m >= 2 => m,
            _ => continue,
        };
        let lang = match cultures.get(cluster_id) {
            Some(c) => c.language,
            None => continue,
        };
        let e = sums.entry(*faction_id).or_insert(([0.0; 4], 0.0));
        for a in 0..4 {
            e.0[a] += lang[a] * mc as f32;
        }
        e.1 += mc as f32;
    }
    sums.into_iter()
        .map(|(f, (s, w))| {
            let mut c = [0.0f32; 4];
            if w > 0.0 {
                for a in 0..4 {
                    c[a] = (s[a] / w).clamp(0.0, 1.0);
                }
            }
            (f, c)
        })
        .collect()
}

/// Member-weighted per-faction religion signal for culture drift (FR-CIV-CULTURE).
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_religion_signals(
    religious_profiles: &BTreeMap<u32, ReligiousProfile>,
    dominant: &BTreeMap<u64, u32>,
    member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u32, f32> {
    let mut sums: BTreeMap<u32, (f32, f32)> = BTreeMap::new();
    for (&settlement_id, &faction_id) in dominant {
        let Some(profile) = religious_profiles.get(&(settlement_id as u32)) else {
            continue;
        };
        let weight = member_counts
            .get(&settlement_id)
            .copied()
            .unwrap_or(1)
            .max(1) as f32;
        let signal = (profile.monitoring * 0.40
            + profile.mythic_coherence * 0.40
            + profile.uncertainty_reduction * 0.20)
            .clamp(0.0, 1.0);
        let entry = sums.entry(faction_id).or_insert((0.0, 0.0));
        entry.0 += signal * weight;
        entry.1 += weight;
    }

    sums.into_iter()
        .filter_map(|(faction_id, (sum, weight))| {
            (weight > 0.0).then_some((faction_id, (sum / weight).clamp(0.0, 1.0)))
        })
        .collect()
}

#[allow(dead_code)] // Reserved for future simulation integration
fn fabric_tier_signal(fabric: FabricTier) -> f32 {
    match fabric {
        FabricTier::Tight => 1.0,
        FabricTier::Loosened => 0.7,
        FabricTier::Strained => 0.35,
        FabricTier::Fractured => 0.1,
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_actor_hardship_signal(
    settlement_id: u32,
    settlement_actors: &BTreeMap<u32, BTreeSet<u64>>,
    actor_hardship: &BTreeMap<u64, i64>,
) -> f32 {
    let Some(actors) = settlement_actors.get(&settlement_id) else {
        return 0.0;
    };
    if actors.is_empty() {
        return 0.0;
    }
    let sum: i64 = actors
        .iter()
        .map(|actor_id| actor_hardship.get(actor_id).copied().unwrap_or(0).max(0))
        .sum();
    ((sum as f32 / actors.len() as f32) / 1000.0).clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_kinship_density_signal(
    settlement_id: u32,
    settlement_actors: &BTreeMap<u32, BTreeSet<u64>>,
    kinship: &BTreeMap<u64, Vec<KinshipEdge>>,
) -> f32 {
    let Some(actors) = settlement_actors.get(&settlement_id) else {
        return 1.0;
    };
    if actors.len() <= 1 {
        return 1.0;
    }
    let possible = (actors.len() * (actors.len() - 1)) as f32;
    let internal_edges = actors
        .iter()
        .filter_map(|actor_id| kinship.get(actor_id))
        .flat_map(|edges| edges.iter())
        .filter(|edge| actors.contains(&edge.target))
        .count() as f32;
    (internal_edges / possible).clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_trade_contact_signal(settlement_id: u32, flows: &[SettlementTradeFlow]) -> f32 {
    let volume: i64 = flows
        .iter()
        .filter(|flow| {
            flow.from_settlement == u64::from(settlement_id)
                || flow.to_settlement == u64::from(settlement_id)
        })
        .map(|flow| flow.qty.max(0))
        .sum();
    (volume as f32 / 100.0).clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_religion_spread_edges(flows: &[SettlementTradeFlow]) -> BTreeMap<(u32, u32), f32> {
    let mut edges = BTreeMap::new();
    for flow in flows {
        if flow.from_settlement == flow.to_settlement || flow.qty <= 0 {
            continue;
        }
        // Cast SettlementId (u64) â†’ u32; the caller enforces the
        // `u32::MAX` truncation guard before insertion in
        // `spread_religion_between_settlements`.
        let a = flow.from_settlement.min(flow.to_settlement) as u32;
        let b = flow.from_settlement.max(flow.to_settlement) as u32;
        let strength = (flow.qty as f32 / 100.0).clamp(0.05, 1.0);
        edges
            .entry((a, b))
            .and_modify(|existing: &mut f32| *existing = existing.max(strength))
            .or_insert(strength);
    }
    edges
}

#[allow(dead_code)] // Reserved for future simulation integration
fn accumulate_profile_diffusion(
    left: &ReligiousProfile,
    right: &ReligiousProfile,
    rate: f32,
    deltas: &mut BTreeMap<u32, (f32, f32, f32)>,
    left_id: u32,
    right_id: u32,
) {
    let dm = (right.monitoring - left.monitoring) * rate;
    let dc = (right.mythic_coherence - left.mythic_coherence) * rate;
    let du = (right.uncertainty_reduction - left.uncertainty_reduction) * rate;
    let left_delta = deltas.entry(left_id).or_insert((0.0, 0.0, 0.0));
    left_delta.0 += dm;
    left_delta.1 += dc;
    left_delta.2 += du;
    let right_delta = deltas.entry(right_id).or_insert((0.0, 0.0, 0.0));
    right_delta.0 -= dm;
    right_delta.1 -= dc;
    right_delta.2 -= du;
}

/// Canonical settlement contact edges when any cross-cluster agents are within radius (N3).
#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_contact_pairs(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    contact_radius_fp: i64,
) -> BTreeSet<(u64, u64)> {
    let contact_radius_sq = i128::from(contact_radius_fp) * i128::from(contact_radius_fp);
    let mut by_cluster: BTreeMap<u64, Vec<(i64, i64)>> = BTreeMap::new();
    for (_, (member, pos)) in world.query::<(&ClusterMember, &Position3d)>().iter() {
        let cluster_id = member.cluster.0;
        let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
        if members < SETTLEMENT_MIN_MEMBERS {
            continue;
        }
        by_cluster
            .entry(cluster_id)
            .or_default()
            .push((pos.coord.x, pos.coord.z));
    }

    let cluster_ids: Vec<u64> = by_cluster.keys().copied().collect();
    let mut contacts = BTreeSet::new();
    for i in 0..cluster_ids.len() {
        for j in (i + 1)..cluster_ids.len() {
            let ca = cluster_ids[i];
            let cb = cluster_ids[j];
            let Some(agents_a) = by_cluster.get(&ca) else {
                continue;
            };
            let Some(agents_b) = by_cluster.get(&cb) else {
                continue;
            };
            let in_contact = agents_a.iter().any(|&(ax, az)| {
                agents_b.iter().any(|&(bx, bz)| {
                    let dx = i128::from(ax) - i128::from(bx);
                    let dz = i128::from(az) - i128::from(bz);
                    dx * dx + dz * dz <= contact_radius_sq
                })
            });
            if in_contact {
                contacts.insert((ca.min(cb), ca.max(cb)));
            }
        }
    }
    contacts
}

/// Faction pairs implied by contacting settlements with different dominant factions (N3).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_faction_pairs_from_settlement_contact(
    dominant: &BTreeMap<u64, u32>,
    contacts: &BTreeSet<(u64, u64)>,
) -> Vec<(u32, u32)> {
    let mut pairs = BTreeSet::new();
    for &(ca, cb) in contacts {
        let Some(&fa) = dominant.get(&ca) else {
            continue;
        };
        let Some(&fb) = dominant.get(&cb) else {
            continue;
        };
        if fa != fb {
            pairs.insert((fa.min(fb), fa.max(fb)));
        }
    }
    pairs.into_iter().collect()
}

/// Select diplomacy faction pair from settlement contact, then presence, then registry (N3).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_pair_from_settlement_overlap(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    registered_factions: &[u32],
    tick: u64,
) -> (u32, u32) {
    let dominant = settlement_dominant_factions(world, cluster_member_counts);
    let contacts =
        settlement_contact_pairs(world, cluster_member_counts, SETTLEMENT_CONTACT_RADIUS_FP);
    let pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);
    if !pairs.is_empty() {
        let idx = (tick as usize / 500) % pairs.len();
        return pairs[idx];
    }

    let present: Vec<u32> = dominant
        .values()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if present.len() >= 2 {
        let idx = tick as usize % present.len();
        let a = present[idx];
        let b = present[(idx + 1) % present.len()];
        return (a, b);
    }

    let idx = tick as usize;
    let a = registered_factions[idx % registered_factions.len()];
    let b = registered_factions[(idx + 1) % registered_factions.len()];
    (a, b)
}

/// Scales every stored relation toward neutral without overshooting zero.
///
/// [`DiplomacyMatrix`] has no native decay; calibrated `apply_signal` calls
/// achieve `score * factor` per pair (FR-CIV-0100 criticality).
#[allow(dead_code)] // Reserved for future simulation integration
fn decay_faction_relations(matrix: &mut DiplomacyMatrix, factor: f32) {
    let factor = factor.clamp(0.0, 1.0);
    let pairs = matrix.snapshot();
    for (a, b, record) in pairs {
        let score = record.score;
        if score == 0.0 {
            continue;
        }
        let target = score * factor;
        let delta = target - score;
        if delta > 0.0 {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    trade_volume: delta / DIPLOMACY_TRADE_DRIFT,
                    ..Default::default()
                },
            );
        } else {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    resource_competition: (-delta) / DIPLOMACY_COMPETITION_DRIFT,
                    ..Default::default()
                },
            );
        }
    }
}

/// Sustained [`DiplomacyKind::TradeAgreement`] events before an emergent route is born.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD: u32 = 2;
/// Minimum pairwise relation score required to birth an emergent route.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_ROUTE_MIN_RELATION: f32 = 0.0;
/// Hard cap on total trade routes (bootstrap + emergent) to bound memory and tick cost.
#[allow(dead_code)] // Reserved for future simulation integration
const MAX_TRADE_ROUTES: usize = 64;
/// Ticks without resource flow before an emergent route is removed.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_ROUTE_UNUSED_DECAY_TICKS: u32 = 2_000;

#[allow(dead_code)] // Reserved for future simulation integration
fn canonical_faction_pair(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Map a registered faction id to the diplomacy matrix cluster key (N3 bridge).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn faction_cluster_id(faction: u32) -> u32 {
    faction
}

/// Round-robin pair selection over the static faction registry (tests / fallback).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_faction_pair(faction_ids: &[u32], tick: u64) -> (u32, u32) {
    if faction_ids.len() < 2 {
        return (0, 0);
    }
    let idx = tick as usize % faction_ids.len();
    let a = faction_ids[idx];
    let b = faction_ids[(idx + 1) % faction_ids.len()];
    (a, b)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn all_registered_faction_pairs(faction_ids: &[u32]) -> Vec<(u32, u32)> {
    let mut pairs = Vec::new();
    for i in 0..faction_ids.len() {
        for j in (i + 1)..faction_ids.len() {
            pairs.push(canonical_faction_pair(faction_ids[i], faction_ids[j]));
        }
    }
    pairs
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn rollup_cluster_member_counts(world: &World) -> BTreeMap<u64, u32> {
    let mut counts = BTreeMap::new();
    for (_, member) in world.query::<&ClusterMember>().iter() {
        *counts.entry(member.cluster.0).or_insert(0) += 1;
    }
    counts
}

/// Per-cluster member count (alias of [`rollup_cluster_member_counts`]).
/// Stub: same shape so callers can treat settlement/cluster membership
/// uniformly until the engine fully merges the two.
#[allow(dead_code)] // Reserved for future simulation integration
fn settlement_member_counts(world: &World) -> BTreeMap<u64, u32> {
    rollup_cluster_member_counts(world)
}

/// Map a `ResourceType` to its market-state key (coerced to a `&'static str`
/// for the market bus).
#[must_use]
#[allow(dead_code)] // Reserved for future simulation integration
pub fn resource_market_key(resource: ResourceType, _region: u32) -> &'static str {
    match resource {
        ResourceType::Food => "food",
        ResourceType::Wood => "wood",
        ResourceType::Metal => "metal",
        ResourceType::Energy => "energy",
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn treasury_disparity_whole(treasury: &HashMap<u32, Fixed>, a: u32, b: u32) -> i64 {
    let ta = treasury.get(&a).copied().unwrap_or(Fixed::ZERO);
    let tb = treasury.get(&b).copied().unwrap_or(Fixed::ZERO);
    (ta.to_bits() - tb.to_bits()).abs() / crate::SCALE
}

#[allow(dead_code)] // Reserved for future simulation integration
fn mean_pair_aggression(aggression: &BTreeMap<u32, f32>, a: u32, b: u32) -> f32 {
    let aa = aggression.get(&a).copied().unwrap_or(0.0);
    let ab = aggression.get(&b).copied().unwrap_or(0.0);
    (aa + ab) * 0.5
}

#[allow(dead_code)] // Reserved for future simulation integration
fn shared_religion_cohesion(cultures: &BTreeMap<u64, CultureProfile>, a: u32, b: u32) -> f32 {
    let Some(pa) = cultures.get(&u64::from(a)) else {
        return 0.0;
    };
    let Some(pb) = cultures.get(&u64::from(b)) else {
        return 0.0;
    };
    let similarity = 1.0 - cultural_distance(pa.traits, pb.traits);
    similarity.clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn shared_religious_unity(cultures: &BTreeMap<u64, CultureProfile>, a: u32, b: u32) -> bool {
    shared_religion_cohesion(cultures, a, b) >= 0.7
}

#[allow(dead_code)] // Reserved for future simulation integration
fn resource_competition_signal(ra: &Resources, rb: &Resources) -> f32 {
    let overlaps = [
        (ra.food, rb.food),
        (ra.wood, rb.wood),
        (ra.metal, rb.metal),
        (ra.energy, rb.energy),
    ];
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for (a, b) in overlaps {
        if a.to_bits() > 0 && b.to_bits() > 0 {
            let af = a.to_bits() as f32;
            let bf = b.to_bits() as f32;
            sum += af.min(bf) / af.max(bf);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f32).clamp(0.0, 1.0)
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn need_complementarity_signal(ra: &Resources, rb: &Resources) -> f32 {
    let pairs = [
        (ra.food, rb.food),
        (ra.wood, rb.wood),
        (ra.metal, rb.metal),
        (ra.energy, rb.energy),
    ];
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for (a, b) in pairs {
        if a.to_bits() == 0 && b.to_bits() == 0 {
            continue;
        }
        let af = a.to_bits() as f32;
        let bf = b.to_bits() as f32;
        let gap = (af - bf).abs();
        let scale = af.max(bf).max(1.0);
        sum += (gap / scale).clamp(0.0, 1.0);
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f32).clamp(0.0, 1.0)
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn scarcity_pressure_signal(energy_budget: Fixed, ra: &Resources, rb: &Resources) -> f32 {
    const SCARCITY_GATE: i64 = 100;
    let budget_scarce = i64::from(energy_budget.to_bits()) / crate::SCALE < SCARCITY_GATE;
    let a_scarce = i64::from(ra.energy.to_bits()) / crate::SCALE < SCARCITY_GATE
        && i64::from(ra.food.to_bits()) / crate::SCALE < SCARCITY_GATE;
    let b_scarce = i64::from(rb.energy.to_bits()) / crate::SCALE < SCARCITY_GATE
        && i64::from(rb.food.to_bits()) / crate::SCALE < SCARCITY_GATE;
    if budget_scarce && a_scarce && b_scarce {
        1.0
    } else if (a_scarce && !b_scarce) || (b_scarce && !a_scarce) {
        -0.5
    } else {
        0.0
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn trade_volume_signal(routes: &[TradeRoute], a: u32, b: u32) -> f32 {
    let volume: i64 = routes
        .iter()
        .filter(|route| {
            (route.from_faction == a && route.to_faction == b)
                || (route.from_faction == b && route.to_faction == a)
        })
        .map(|route| i64::from(route.volume.to_bits()))
        .sum();
    ((volume as f64) / 1_000_000.0).clamp(0.0, 1.0) as f32
}

#[allow(dead_code)] // Reserved for future simulation integration
fn proximity_signal(
    a: u32,
    b: u32,
    world: &World,
    member_counts: &BTreeMap<u64, u32>,
    routes: &[TradeRoute],
) -> f32 {
    let dominant = settlement_dominant_factions(world, member_counts);
    let contacts = settlement_contact_pairs(world, member_counts, SETTLEMENT_CONTACT_RADIUS_FP);
    let contact_pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);
    let pair = canonical_faction_pair(a, b);
    if contact_pairs.contains(&pair) {
        return 1.0;
    }
    if routes.iter().any(|route| {
        (route.from_faction == a && route.to_faction == b)
            || (route.from_faction == b && route.to_faction == a)
    }) {
        return 0.7;
    }
    if a.abs_diff(b) == 1 {
        return 0.5;
    }
    0.0
}

#[allow(dead_code)] // Reserved for future simulation integration
#[allow(clippy::too_many_arguments)]
fn diplomacy_signal_for_pair(
    a: u32,
    b: u32,
    state: &WorldState,
    world: &World,
    member_counts: &BTreeMap<u64, u32>,
    cultures: &BTreeMap<u64, CultureProfile>,
    faction_ideologies: &BTreeMap<u32, FactionIdeologyState>,
    grief: &GriefAccumulator,
) -> DiplomacySignal {
    let ra = state.faction_resources.get(&a).cloned().unwrap_or_default();
    let rb = state.faction_resources.get(&b).cloned().unwrap_or_default();
    let religion = shared_religion_cohesion(cultures, a, b);
    let cooperation = culture_cooperation_signal(faction_ideologies, a, b);
    let openness = culture_openness_signal(faction_ideologies, a, b);
    let competition = resource_competition_signal(&ra, &rb) * (1.0 - religion * 0.35);
    DiplomacySignal {
        resource_competition: competition,
        trade_volume: trade_volume_signal(&state.trade_routes, a, b),
        proximity: proximity_signal(a, b, world, member_counts, &state.trade_routes),
        combat_grievance: grief.get(a, b),
        need_complementarity: need_complementarity_signal(&ra, &rb)
            + religion * 0.25
            + cooperation * 0.55,
        scarcity_pressure: scarcity_pressure_signal(state.energy_budget_joules, &ra, &rb)
            - openness * 0.20,
    }
}

/// Deterministic goods label from exporter faction id (stable, integer-only).
#[allow(dead_code)] // Reserved for future simulation integration
fn emergent_route_goods(from: u32) -> &'static str {
    match from % 3 {
        0 => "grain",
        1 => "ore",
        _ => "cloth",
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn record_trade_agreement_streak(streak: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) {
    let pair = canonical_faction_pair(a, b);
    *streak.entry(pair).or_default() += 1;
}

#[allow(dead_code)] // Reserved for future simulation integration
fn reset_trade_agreement_streak(streak: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) {
    streak.remove(&canonical_faction_pair(a, b));
}

#[allow(dead_code)] // Reserved for future simulation integration
fn remove_emergent_routes_between(state: &mut WorldState, a: u32, b: u32) {
    let to_remove: Vec<(u32, u32, String)> = state
        .emergent_trade_route_keys
        .iter()
        .filter(|(from, to, _)| (*from == a && *to == b) || (*from == b && *to == a))
        .cloned()
        .collect();
    for key in &to_remove {
        state.emergent_trade_route_keys.remove(key);
        state.trade_route_idle_ticks.remove(key);
    }
    state.trade_routes.retain(|route| {
        let key = (route.from_faction, route.to_faction, route.goods.clone());
        !to_remove.contains(&key)
    });
}

#[allow(dead_code)] // Reserved for future simulation integration
fn decay_idle_emergent_trade_routes(state: &mut WorldState, flowed: &BTreeSet<(u32, u32, String)>) {
    let emergent: Vec<(u32, u32, String)> =
        state.emergent_trade_route_keys.iter().cloned().collect();
    let mut to_remove = Vec::new();
    for key in emergent {
        if flowed.contains(&key) {
            state.trade_route_idle_ticks.insert(key.clone(), 0);
            continue;
        }
        let idle = state.trade_route_idle_ticks.entry(key.clone()).or_insert(0);
        *idle = idle.saturating_add(1);
        if *idle >= TRADE_ROUTE_UNUSED_DECAY_TICKS {
            to_remove.push(key);
        }
    }
    for key in &to_remove {
        state.emergent_trade_route_keys.remove(key);
        state.trade_route_idle_ticks.remove(key);
    }
    if !to_remove.is_empty() {
        state.trade_routes.retain(|route| {
            let key = (route.from_faction, route.to_faction, route.goods.clone());
            !to_remove.contains(&key)
        });
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn route_resource(goods: &str) -> ResourceType {
    match goods {
        "grain" => ResourceType::Food,
        "timber" => ResourceType::Wood,
        "ore" | "tools" => ResourceType::Metal,
        "cloth" | "salt" => ResourceType::Energy,
        _ => ResourceType::Food,
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn resource_amount(resources: &Resources, resource: ResourceType) -> Fixed {
    match resource {
        ResourceType::Food => resources.food,
        ResourceType::Wood => resources.wood,
        ResourceType::Metal => resources.metal,
        ResourceType::Energy => resources.energy,
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn adjust_resource(resources: &mut Resources, resource: ResourceType, delta: Fixed) {
    match resource {
        ResourceType::Food => resources.food += delta,
        ResourceType::Wood => resources.wood += delta,
        ResourceType::Metal => resources.metal += delta,
        ResourceType::Energy => resources.energy += delta,
    }
}
