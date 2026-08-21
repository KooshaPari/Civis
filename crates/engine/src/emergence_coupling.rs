//! Emergence-coupling free functions extracted from `engine.rs`.
//!
//! Contains standalone free functions, constants, and helper types that
//! implement the emergence-coupling logic (FR-CIV-0100). Extracted from the
//! monolithic `engine.rs` to reduce file size and improve modularity.

use crate::engine::{
    Building, BuildingType, Citizen, CohesionEvent, CohesionEventKind, CohesionSnapshot,
    CombatDamagePulse, EconomicFocus, InstitutionEvent, JobType, KinshipKind, LanguageState,
    MembershipPayoffTotals, MilitaryUnit, MoodSnapshot, Position, Production, ReligionEvent,
    ReligionEventKind, ResearchCache, Resources, Sim, SimSeed, Simulation, SimulationSnapshot,
    StratBand, StratificationEvent, StratificationEventKind, StratificationReport, UnitType,
};
use crate::fixed_math::{Fixed, FixedFromNum};
use crate::invariants::InvariantError;
use crate::SCALE;
use civ_agents::{DiplomacyOutcome, LodTier, Needs, Psyche, SocialGraph, Tools, Wardrobe};
use civ_build::{BuildSite, DemandSignals};
use civ_genetics::sentience::{cognition_score, CognitionTraitProfile, SentienceThreshold};
use civ_genetics::Dna;
use civ_planet::{BiomeKind, WeatherCell};
use civ_tactics::{CombatEngagement, DoctrineLibrary};

use hecs::World;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Maximum chronicle history lines retained in [`WorldState::chronicle`].
#[allow(dead_code)] // Reserved for future simulation integration
const CHRONICLE_MAX_LEN: usize = 200;

/// Food units each cluster member adds to settlement stock per tick in
/// [`Simulation::phase_life`].
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const CLUSTER_FOOD_PRODUCTION_PER_MEMBER: i64 = 1;
/// Food units each cluster member drains per tick in
/// [`Simulation::phase_settlement_consumption`]. Must be >= production so the
/// accumulator stays bounded (net zero at matched rates; converges toward zero
/// when strictly greater).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const CLUSTER_FOOD_CONSUMPTION_PER_MEMBER: i64 = 1;
/// Market weight for settlement food commons before pressure scaling (N1).
#[allow(dead_code)] // Reserved for future simulation integration
const SETTLEMENT_FOOD_MARKET_WEIGHT: i64 = 2;
/// Divisor mapping population-scale demand/supply (and settlement commons) into
/// the capped per-tick food price step (N1: local abundance must move price
/// within `MarketState::apply_pressure`'s Ã‚Â±8 cent clamp).
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
/// At or below the baseline price (abundance) the factor is `1.0` Ã¢â‚¬â€ surplus
/// does NOT boost births above the natural rate (conservative; abundance is
/// already expressed via the ECS food-needs path). As the price rises above
/// baseline the factor falls as `baseline / price`, so a 2x price halves the
/// birth chance. The factor never reaches zero, so a starving society can still
/// recover, and it only ever scales births DOWN Ã¢â‚¬â€ population is never reduced
/// by this coupling.
#[allow(dead_code)] // Reserved for future simulation integration
fn food_scarcity_birth_factor(food_price: i64) -> f64 {
    let price = food_price.max(FOOD_SCARCITY_BASELINE);
    (FOOD_SCARCITY_BASELINE as f64 / price as f64).clamp(0.0, 1.0)
}

/// Per-tick change in societal unrest from food-market scarcity (FR-CIV-0100 Ã‚Â§3
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
/// double-counting. Per-tick clamped to [-DECAY, MAX_RISE] Ã¢â‚¬â€ no runaway.
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
/// Ã‚Â§3 emergence). Comfortable treasury and food sit at baseline; shortfall pushes
/// the shadow above baseline so [`unrest_delta`] accrues faction unrest.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn faction_wealth_scarcity_shadow(treasury: Fixed, resources: &Resources) -> i64 {
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
pub(crate) fn faction_unrest_delta_from_shadow(scarcity_shadow: i64) -> i64 {
    unrest_delta(scarcity_shadow)
}

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): energy depletion breeds unrest.
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

/// Upward causation (FR-CIV-0100 Ã‚Â§3): the mean MISERY of agents (negative Psyche
/// mood valence) adds to societal unrest. Reuses the ECS Psyche component Ã¢â‚¬â€ the
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

/// Upward causation (FR-CIV-0100 Ã‚Â§3): micro ideology consensus (`Psyche.beliefs[0]`)
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

/// Upward causation (FR-CIV-0100 Ã‚Â§3): mean positive agent tie trust caches a
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
pub(crate) fn avg_faction_kinship(world: &hecs::World) -> f32 {
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
pub(crate) fn avg_social_affinity(world: &hecs::World) -> f32 {
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

/// N12: bias magnitude for the affinityÃ¢â€ â€™diplomacy threshold (FR-CIV-EMERGENCE-N12).
/// `avg_affinity Ã¢Ë†Ë† [-1, 1]` scaled by this yields the threshold bias in `[-5000, 5000]`,
/// bounded below by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` at the combination site.
#[allow(dead_code)] // Reserved for future simulation integration
const N12_AFFINITY_BIAS_SCALE: f32 = 5_000.0;

/// N12: collective affinity threshold bias. Positive goodwill raises the conflict
/// threshold (more tolerance before fighting); hostility lowers it. The input is
/// clamped to `[-1, 1]` so the bias is bounded to `[-5000, 5000]`. Returns i64.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn affinity_threshold_bias(avg_affinity: f32) -> i64 {
    (avg_affinity.clamp(-1.0, 1.0) * N12_AFFINITY_BIAS_SCALE) as i64
}

/// Maximum peace bonus from shared patron veneration (religion Ã¢â€ â€™ diplomacy coupling).
#[allow(dead_code)] // Reserved for future simulation integration
const RELIGIOUS_UNITY_PEACE_CAP: i64 = 1_000;

/// RELIGIONÃ¢â€ â€™DIPLOMACY emergence coupling (FR-CIV-RELIGION-002).
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
pub(crate) fn religious_unity_peace_bonus(has_patron: bool) -> i64 {
    if has_patron {
        RELIGIOUS_UNITY_PEACE_CAP
    } else {
        0
    }
}

/// Maximum peace bonus from mutual language intelligibility (language Ã¢â€ â€™ diplomacy coupling).
#[allow(dead_code)] // Reserved for future simulation integration
const LANGUAGE_INTELLIGIBILITY_PEACE_CAP: i64 = 1_200;

/// LANGUAGEÃ¢â€ â€™DIPLOMACY emergence coupling.
///
/// Low language distance (mutually intelligible factions) Ã¢â€ â€™ positive peace bonus
/// on the conflict threshold, mirroring the N9/N12/#564 magnitudes.
/// High distance Ã¢â€ â€™ 0 bonus (no effect on threshold).
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
pub(crate) fn candidate_economic_focus(
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): research raises production yield Ã¢â‚¬â€
/// better tools/techniques lift per-building output. +10% per research tier,
/// capped at +100% (2x). De-silos phase_production, which read no emergent state.
#[allow(dead_code)] // Reserved for future simulation integration
fn production_yield_factor(research_tier: u64) -> Fixed {
    let bonus_permille = research_tier.saturating_mul(100).min(1_000) as i64;
    Fixed::from_num(1_000 + bonus_permille) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-CONTENT-001): terrain biome modulates food
/// production Ã¢â‚¬â€ fertile land grows more food, barren land grows less.  The factor
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): social cohesion speeds military
/// morale recovery Ã¢â‚¬â€ a unified society's troops rally faster. Returns the
/// per-tick morale recovery increment, rising with cohesion from a 0.010 base
/// up to a 0.050 cap.
#[allow(dead_code)] // Reserved for future simulation integration
fn morale_recovery_rate(cohesion: u64) -> Fixed {
    const BASE_PERMILLE: i64 = 10;
    const CAP_PERMILLE: i64 = 50;
    let bonus = (cohesion / 25_000).min((CAP_PERMILLE - BASE_PERMILLE) as u64) as i64;
    Fixed::from_num(BASE_PERMILLE + bonus) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): overcrowding breeds unrest
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): social cohesion accelerates
/// research Ã¢â‚¬â€ a unified society collaborates. Returns a per-mille bonus to the
/// per-tick research contribution, up to +50%.
#[allow(dead_code)] // Reserved for future simulation integration
fn cohesion_research_bonus_permille(cohesion: u64) -> u64 {
    (cohesion / 2_000).min(500)
}

/// The wealth gap (in whole currency units) between the richest and poorest
/// faction Ã¢â‚¬â€ an emergent measure of structural inequality across the society.
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): structural inequality breeds class
/// unrest. A wide wealth gap between factions adds unrest scaled by the gap,
/// capped per tick. Distinct from scarcity Ã¢â‚¬â€ this is about distribution.
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
/// is gradual Ã¢â‚¬â€ hysteresis).
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3 emergence): research mitigates
/// unrest Ã¢â‚¬â€ advanced food logistics (storage, distribution) blunt the
/// scarcity-driven rise. Only the positive (rising) part is damped; decay is
/// untouched. The mitigation is bounded (tier capped at 9 Ã¢â€ â€™ at most a 10x
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): research accelerates construction.
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

/// Emergent construction demand (FR-CIV-0100 Ã‚Â§3): the built environment responds
/// to society Ã¢â‚¬â€ crowding drives housing, research drives industry, cohesion
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
pub(crate) const COHESION_BELIEF_DIVISOR: u64 = 200;
/// Unrest units that fray one unit of cohesion per tick.
#[allow(dead_code)] // Reserved for future simulation integration
const COHESION_UNREST_DIVISOR: u64 = 50;

/// Emergence policy (FR-CIV-0100 Ã‚Â§3): the social fabric's per-tick change is the
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
/// identity Ã¢â‚¬â€ "we are the people who woke"). Kept SMALL relative to the
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

/// Arbitrage policy (FR-CIV-0100 Ã‚Â§3 emergence): trade volume scales with the
/// surplus gap between exporter and importer Ã¢â‚¬â€ a well-stocked source feeding a
/// scarce destination ships MORE. Returns a multiplier in `[1.0, 2.0]`, bounded
/// at 2x so the priceÃ¢â€ â€volumeÃ¢â€ â€treasuryÃ¢â€ â€demand loop self-limits rather than
/// running away (design-layer criticality bound). No boost when the source is
/// not in surplus relative to the destination.
#[allow(dead_code)] // Reserved for future simulation integration
fn trade_volume_multiplier(from_stock: Fixed, to_stock: Fixed) -> Fixed {
    let gap = (from_stock - to_stock).max(Fixed::ZERO);
    let normalized = (gap / Fixed::from_num(TRADE_GAP_SCALE)).min(Fixed::from_num(1));
    Fixed::from_num(1) + normalized
}

/// Floor (per-mille) below which unrest cannot throttle trade Ã¢â‚¬â€ even a society
/// in turmoil keeps half its commerce moving.
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_TRADE_FLOOR_PERMILLE: i64 = 500;
/// Units of standing unrest that throttle trade by one per-mille.
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_PER_TRADE_PERMILLE: u64 = 4;

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3 emergence): societal unrest
/// disrupts commerce. Returns a trade-volume factor in `[0.5, 1.0]` Ã¢â‚¬â€ `1.0`
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

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): macro cohesion AND cached micro
/// interpersonal trust lift trade volume. Returns factor in [1.0, 1.75].
#[allow(dead_code)] // Reserved for future simulation integration
fn society_trade_factor(cohesion: u64, micro_trust_permille: u64) -> Fixed {
    let cohesion_boost =
        (cohesion / COHESION_PER_TRADE_PERMILLE).min(COHESION_TRADE_CAP_PERMILLE as u64) as i64;
    let micro_boost = micro_trust_permille.min(MICRO_TRUST_CAP_PERMILLE) as i64;
    let total = (cohesion_boost + micro_boost).min(SOCIETY_TRADE_BOOST_CAP_PERMILLE);
    Fixed::from_num(1_000 + total) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3): a cohesive society trades MORE Ã¢â‚¬â€
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
pub(crate) const DIPLOMACY_BASE_CONFLICT_THRESHOLD: i64 = 10_000;
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
pub(crate) const DIPLOMACY_TRADE_DRIFT: f32 = 0.08;
/// Competition drift per unit signal in [`DiplomacyMatrix::apply_signal`].
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const DIPLOMACY_COMPETITION_DRIFT: f32 = 0.12;
/// Max threshold shift from a saturated pairwise relation score (`Ã‚Â±1.0`).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const FACTION_RELATION_THRESHOLD_SPAN: i64 = 5_000;
/// Max peace bonus from identical pairwise cultural traits (N2 coupling).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const CULTURE_PEACE_SPAN: f32 = 3_000.0;

pub use crate::settlement_helpers::*;
