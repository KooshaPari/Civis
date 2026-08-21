//! Social dynamics module — extracted from `engine.rs` (Pass 5).
//!
//! Contains unrest calculation, cohesion, belief, institution level, and
//! diplomacy-threshold helpers that model the social fabric of a civilisation.

use civ_agents::{Psyche, SocialGraph};
use civ_genetics::sentience::CognitionTraitProfile;
use hecs::World;
use std::collections::BTreeMap;

use crate::Fixed;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Baseline food-market price (cents) at/above which scarcity unrest is zero.
pub(crate) const FOOD_SCARCITY_BASELINE: i64 = 1_000;

/// Belief units that contribute one unit of cohesion growth per tick.
const COHESION_BELIEF_DIVISOR: u64 = 200;
/// Unrest units that fray one unit of cohesion per tick.
const COHESION_UNREST_DIVISOR: u64 = 50;

/// Max institution level (criticality cap on the belief->temple->belief loop).
pub const MAX_INSTITUTION_LEVEL: u32 = 5;

/// Maximum peace bonus from mutual language intelligibility (language -> diplomacy coupling).
const LANGUAGE_INTELLIGIBILITY_PEACE_CAP: i64 = 1_200;

/// Maximum peace bonus from shared patron veneration (religion -> diplomacy coupling).
const RELIGIOUS_UNITY_PEACE_CAP: i64 = 1_000;

/// Belief units that contribute one unit of cohesion growth per tick
/// (used by diplomacy threshold calculations).
const BELIEF_PEACE_DIVISOR: u64 = 50;
/// Cap on the belief-driven peace bonus: shared faith can at most double a
/// society's tolerance for inequality -- it never makes conflict impossible.
const BELIEF_PEACE_CAP: i64 = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
/// Unrest units required to erode the conflict threshold by one currency unit.
const UNREST_WAR_DIVISOR: u64 = 50;
/// Cap on how much unrest can erode the threshold (currency units).
const UNREST_WAR_CAP: i64 = 8_000;
/// Wealth-disparity (in whole currency units) at which two factions clash when
/// they share no faith. Above this gap the have-nots turn on the haves.
const DIPLOMACY_BASE_CONFLICT_THRESHOLD: i64 = 10_000;
/// Floor on the conflict threshold: even a furious, faithless society still
/// needs SOME wealth disparity to go to war -- discontent alone is not casus belli.
const DIPLOMACY_MIN_CONFLICT_THRESHOLD: i64 = 2_000;

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: each lineage crossing the sentience
/// threshold this tick mints a small bounded pulse of cohesion (shared
/// identity -- "we are the people who woke"). Kept SMALL relative to the
/// existing cohesion bind/fray inputs so the moment of awakening nudges
/// the social fabric without dominating it; the per-tick cap mirrors the
/// spirit of [`crate::emergence`] emergence caps.
pub(crate) const COHESION_PER_AWAKENING: i64 = 2;
/// Hard per-tick cap on awakening-driven cohesion nudge (signed i64 so the
/// existing floored-at-zero cohesion mutator absorbs any overshoot cleanly).
pub(crate) const MAX_AWAKENING_COHESION_PER_TICK: i64 = 10;

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: pure gain fn for the awakening -> belief
/// pulse. Returns a signed i64 and clamps to a small per-tick cap so the
/// compatibility shim stays bounded and deterministic.
pub(crate) const BELIEF_PER_AWAKENING: i64 = 2;
#[allow(dead_code)]
pub(crate) const MAX_AWAKENING_BELIEF_PER_TICK: i64 = 10;

// ---------------------------------------------------------------------------
// Food / commodity unrest
// ---------------------------------------------------------------------------

/// Per-tick change in societal unrest from food-market scarcity (FR-CIV-0100 S3
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
/// double-counting. Per-tick clamped to [-DECAY, MAX_RISE] -- no runaway.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn commodity_unrest_delta(prices: &BTreeMap<String, i64>) -> i64 {
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

// ---------------------------------------------------------------------------
// Energy unrest
// ---------------------------------------------------------------------------

/// Downward-causation policy (FR-CIV-0100 S3): energy depletion breeds unrest.
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

// ---------------------------------------------------------------------------
// Agent misery / psyche -> macro coupling
// ---------------------------------------------------------------------------

/// Upward causation (FR-CIV-0100 S3): the mean MISERY of agents (negative Psyche
/// mood valence) adds to societal unrest. Reuses the ECS Psyche component -- the
/// agent emotional layer feeding the macro web. Returns 0..MAX, bounded.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn agent_misery_unrest(world: &World) -> i64 {
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

/// Upward causation (FR-CIV-0100 S3): micro ideology consensus (`Psyche.beliefs[0]`)
/// binds macro cohesion; polarization frays it. Pure `hecs::World` scan, capped i64.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn micro_cohesion_delta(world: &World) -> i64 {
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

// ---------------------------------------------------------------------------
// Psyche maturity
// ---------------------------------------------------------------------------

/// Upward causation (FR-CIV-EMERGENCE-N11): average psyche maturity across all agents.
/// Mature populations stabilize belief (wisdom = stability). Pure `hecs::World` scan.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn avg_psyche_maturity(world: &World) -> f32 {
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

// ---------------------------------------------------------------------------
// Kinship -> cohesion
// ---------------------------------------------------------------------------

/// Upward causation (FR-CIV-EMERGENCE-N10): average kinship across all social ties.
/// Kinship boosts cohesion (family ties stabilize society). Pure `hecs::World` scan.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn avg_faction_kinship(world: &World) -> f32 {
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
pub(crate) fn kinship_cohesion_boost(world: &World) -> i64 {
    const KINSHIP_RATE: f32 = 0.02;
    const KINSHIP_SCALE: f32 = 100_000.0;
    (avg_faction_kinship(world) * KINSHIP_RATE * KINSHIP_SCALE) as i64
}

// ---------------------------------------------------------------------------
// Unrest accumulator
// ---------------------------------------------------------------------------

/// Apply a signed unrest delta, flooring at zero.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn add_unrest_delta(unrest: &mut u64, delta: i64) {
    if delta >= 0 {
        *unrest = unrest.saturating_add(delta as u64);
    } else {
        *unrest = unrest.saturating_sub((-delta) as u64);
    }
}

// ---------------------------------------------------------------------------
// Language -> diplomacy
// ---------------------------------------------------------------------------

/// LANGUAGE->DIPLOMACY emergence coupling.
///
/// Low language distance (mutually intelligible factions) -> positive peace bonus
/// on the conflict threshold, mirroring the N9/N12/#564 magnitudes.
/// High distance -> 0 bonus (no effect on threshold).
///
/// Called with pre-read centroid values (before any mutable borrow of `self`)
/// to satisfy the borrow-checker (E0502).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn language_intelligibility_peace_bonus(language_distance: f32) -> i64 {
    let raw = LANGUAGE_INTELLIGIBILITY_PEACE_CAP as f32 * (1.0 - language_distance.clamp(0.0, 1.0));
    raw.clamp(0.0, LANGUAGE_INTELLIGIBILITY_PEACE_CAP as f32) as i64
}

// ---------------------------------------------------------------------------
// Religion -> diplomacy coupling
// ---------------------------------------------------------------------------

/// RELIGION->DIPLOMACY emergence coupling (FR-CIV-RELIGION-002).
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

// ---------------------------------------------------------------------------
// Institutions
// ---------------------------------------------------------------------------

/// Institution level that a driver signal supports: one level per THRESHOLD of
/// the signal, capped at MAX_INSTITUTION_LEVEL.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn institution_target_level(signal: u64, per_level: u64) -> u32 {
    (signal / per_level.max(1)).min(MAX_INSTITUTION_LEVEL as u64) as u32
}

/// One-step decay toward target (max 1 level change per tick, so growth/decay
/// is gradual -- hysteresis).
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

// ---------------------------------------------------------------------------
// Research -> unrest mitigation
// ---------------------------------------------------------------------------

/// Downward-causation policy (FR-CIV-0100 S3 emergence): research mitigates
/// unrest -- advanced food logistics (storage, distribution) blunt the
/// scarcity-driven rise. Only the positive (rising) part is damped; decay is
/// untouched. The mitigation is bounded (tier capped at 9 -> at most a 10x
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

// ---------------------------------------------------------------------------
// Sentience profile
// ---------------------------------------------------------------------------

/// Default sentience profile used by [`Simulation::phase_sentience`].
/// Companion of `Simulation::default_sentience_profile` (associated stub).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn default_sentience_profile() -> CognitionTraitProfile {
    CognitionTraitProfile::new(
        "sapient-lineage",
        vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
    )
}

// ---------------------------------------------------------------------------
// Cohesion -> belief / unrest coupling
// ---------------------------------------------------------------------------

/// Emergence policy (FR-CIV-0100 S3): the social fabric's per-tick change is the
/// balance of belief (binds, scaled gently) against unrest (frays, scaled
/// harder, so disorder erodes cohesion faster than faith builds it). Returns a
/// signed delta; the caller floors the running total at zero.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn cohesion_delta(belief: u64, unrest: u64) -> i64 {
    let bind = (belief / COHESION_BELIEF_DIVISOR) as i64;
    let fray = (unrest / COHESION_UNREST_DIVISOR) as i64;
    bind - fray
}

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

// ---------------------------------------------------------------------------
// Diplomacy threshold helpers
// ---------------------------------------------------------------------------

/// Downward-causation policy (FR-CIV-0100 S3 emergence): collective belief and
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

/// Combined religion->diplomacy threshold: belief, cohesion, unrest, patron veneration.
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
