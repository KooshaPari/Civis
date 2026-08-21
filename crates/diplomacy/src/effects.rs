//! FR-DIP-002: Advanced diplomacy effects.
//!
//! Self-contained module providing four diplomacy effects that operate on a
//! minimal [`WorldState`]:
//!
//! * [`CulturalInfluenceEffect`] - border-sharing factions drift culturally.
//! * [`TradeEmbargoEffect`] - factions impose trade embargoes, reducing
//!   efficiency between them.
//! * [`MilitaryAllianceEffect`] - factions form military alliances, sharing
//!   intelligence and receiving combat bonuses.
//! * [`TributeEffect`] - dominant factions demand tribute from weaker ones.
//!
//! # Determinism
//!
//! All computation is integer-only over [`BTreeMap`]-backed collections.
//! Given the same [`WorldState`], the same effect produces identical events
//! and state mutations. No RNG, no floating-point, no wall-clock.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Pair, PolityId};

// ---------------------------------------------------------------------------
// World state
// ---------------------------------------------------------------------------

/// Minimal world state accessible to diplomacy effects. Contains only the
/// information the effects need; does not duplicate the full sim state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    /// Cultural profiles keyed by faction.
    pub cultural_profiles: BTreeMap<PolityId, CulturalProfile>,
    /// Trade efficiency between pairs (0-100). Higher = more efficient.
    pub trade_efficiency: BTreeMap<Pair, u32>,
    /// Resources owned by each faction.
    pub resources: BTreeMap<PolityId, u64>,
    /// Shared border cell count between pairs.
    pub shared_borders: BTreeMap<Pair, u32>,
    /// Active trade embargoes between pairs.
    pub embargoes: BTreeMap<Pair, bool>,
    /// Active military alliances between pairs.
    pub military_alliances: BTreeMap<Pair, bool>,
    /// Combat bonuses granted by alliances (per pair, additive percentage).
    pub combat_bonuses: BTreeMap<Pair, u32>,
    /// Military intelligence shared between alliance members (bitmask).
    pub shared_intel: BTreeMap<Pair, u64>,
    /// Tribute relationships: (dominant, vassal) -> amount per tick.
    pub tribute_relationships: BTreeMap<Pair, TributeRelationship>,
    /// Current simulation tick.
    pub tick: u64,
}

/// Cultural profile of a faction across three axes. Values are 0-100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CulturalProfile {
    /// Artistic influence.
    pub art: u32,
    /// Religious influence.
    pub religion: u32,
    /// Technological influence.
    pub technology: u32,
}

impl CulturalProfile {
    /// Create a new profile with all axes set to `value`.
    pub fn uniform(value: u32) -> Self {
        Self {
            art: value,
            religion: value,
            technology: value,
        }
    }

    /// Compute the Manhattan distance to another profile.
    pub fn distance(&self, other: &Self) -> i64 {
        let a = self.art as i64 - other.art as i64;
        let r = self.religion as i64 - other.religion as i64;
        let t = self.technology as i64 - other.technology as i64;
        a.abs() + r.abs() + t.abs()
    }

    /// Drift this profile toward `target` by `rate` (1-100). Returns the
    /// new profile, clamped to [0, 100].
    pub fn drift_toward(&self, target: &CulturalProfile, rate: u32) -> Self {
        let rate = rate.min(100);
        Self {
            art: drift_axis(self.art, target.art, rate),
            religion: drift_axis(self.religion, target.religion, rate),
            technology: drift_axis(self.technology, target.technology, rate),
        }
    }
}

/// Drift a single axis toward a target by `rate` percent.
fn drift_axis(current: u32, target: u32, rate: u32) -> u32 {
    let diff = target as i64 - current as i64;
    let shift = (diff * rate as i64) / 100;
    let result = current as i64 + shift;
    result.clamp(0, 100) as u32
}

/// Tribute relationship metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TributeRelationship {
    /// The dominant (receiver) faction.
    pub dominant: PolityId,
    /// The vassal (payer) faction.
    pub vassal: PolityId,
    /// Amount of resources extracted per tick.
    pub amount: u64,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by diplomacy effects. Downstream systems (AI, scenario,
/// JSON-RPC, replay bus) consume these.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiplomacyEvent {
    /// Cultural profiles drifted between two factions.
    CulturalShift {
        /// The pair whose profiles shifted.
        pair: Pair,
        /// Cultural profile of the first faction after drift.
        profile_lo: CulturalProfile,
        /// Cultural profile of the second faction after drift.
        profile_hi: CulturalProfile,
        /// Tick this occurred on.
        tick: u64,
    },
    /// A trade embargo was imposed or lifted.
    TradeEmbargo {
        /// The pair involved.
        pair: Pair,
        /// `true` if embargo was imposed, `false` if lifted.
        imposed: bool,
        /// Tick this occurred on.
        tick: u64,
    },
    /// A military alliance was formed or dissolved.
    MilitaryAlliance {
        /// The pair involved.
        pair: Pair,
        /// `true` if alliance formed, `false` if dissolved.
        formed: bool,
        /// Combat bonus percentage granted by the alliance.
        combat_bonus: u32,
        /// Tick this occurred on.
        tick: u64,
    },
    /// Tribute was extracted from one faction to another.
    Tribute {
        /// The vassal (payer).
        from: PolityId,
        /// The dominant (receiver).
        to: PolityId,
        /// Amount of resources extracted.
        amount: u64,
        /// Tick this occurred on.
        tick: u64,
    },
}

// ---------------------------------------------------------------------------
// Effect structs
// ---------------------------------------------------------------------------

/// Cultural influence effect: when factions share borders, their cultural
/// profiles drift toward each other each tick. The drift rate is proportional
/// to the number of shared border cells (more overlap = faster convergence).
///
/// The drift is deterministic: each axis moves toward the neighbor's value
/// by `min(shared_borders, max_rate)` percent per tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalInfluenceEffect {
    /// Maximum drift rate per tick (percent, 1-100). Actual rate is
    /// `min(shared_borders, max_rate)`.
    pub max_rate: u32,
}

impl Default for CulturalInfluenceEffect {
    fn default() -> Self {
        Self { max_rate: 10 }
    }
}

impl CulturalInfluenceEffect {
    /// Apply cultural drift for all pairs that share borders.
    ///
    /// For each pair with `shared_borders > 0`, both factions' profiles
    /// drift toward each other by `min(shared_cells, max_rate)` percent.
    /// Events are emitted for every pair where profiles actually changed.
    pub fn apply(&self, world: &mut WorldState) -> Vec<DiplomacyEvent> {
        let rate = self.max_rate.clamp(1, 100);
        let tick = world.tick;
        let pairs: Vec<Pair> = world.shared_borders.keys().copied().collect();
        let mut events = Vec::new();

        for pair in pairs {
            let shared = *world.shared_borders.get(&pair).unwrap_or(&0);
            if shared == 0 {
                continue;
            }

            // Both profiles must exist for drift to occur.
            let (lo_profile, hi_profile) = match (
                world.cultural_profiles.get(&pair.lo).copied(),
                world.cultural_profiles.get(&pair.hi).copied(),
            ) {
                (Some(l), Some(h)) => (l, h),
                _ => continue,
            };

            let effective_rate = shared.min(rate);
            let new_lo = lo_profile.drift_toward(&hi_profile, effective_rate);
            let new_hi = hi_profile.drift_toward(&lo_profile, effective_rate);

            // Only emit events if something actually changed.
            if new_lo == lo_profile && new_hi == hi_profile {
                continue;
            }

            world.cultural_profiles.insert(pair.lo, new_lo);
            world.cultural_profiles.insert(pair.hi, new_hi);

            events.push(DiplomacyEvent::CulturalShift {
                pair,
                profile_lo: new_lo,
                profile_hi: new_hi,
                tick,
            });
        }

        events
    }
}

/// Trade embargo effect: factions can impose trade embargoes that reduce
/// trade efficiency between them. When an embargo is active, the trade
/// efficiency is set to 0 regardless of its previous value. Lifting the
/// embargo restores efficiency to a configured baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeEmbargoEffect {
    /// Baseline trade efficiency restored when an embargo is lifted (0-100).
    pub baseline_efficiency: u32,
}

impl Default for TradeEmbargoEffect {
    fn default() -> Self {
        Self {
            baseline_efficiency: 50,
        }
    }
}

impl TradeEmbargoEffect {
    /// Impose an embargo between two factions.
    ///
    /// Sets trade efficiency to 0 and records the embargo. Returns the
    /// event if the embargo was newly imposed (idempotent: re-imposing an
    /// existing embargo emits no event).
    pub fn impose(
        &self,
        world: &mut WorldState,
        a: PolityId,
        b: PolityId,
    ) -> Option<DiplomacyEvent> {
        let pair = Pair::new(a, b);
        let already = world.embargoes.get(&pair).copied().unwrap_or(false);
        if already {
            return None;
        }
        world.embargoes.insert(pair, true);
        world.trade_efficiency.insert(pair, 0);
        Some(DiplomacyEvent::TradeEmbargo {
            pair,
            imposed: true,
            tick: world.tick,
        })
    }

    /// Lift an embargo between two factions.
    ///
    /// Restores trade efficiency to the baseline and removes the embargo.
    /// Returns the event if the embargo was active (idempotent: lifting a
    /// non-existent embargo emits no event).
    pub fn lift(&self, world: &mut WorldState, a: PolityId, b: PolityId) -> Option<DiplomacyEvent> {
        let pair = Pair::new(a, b);
        let active = world.embargoes.get(&pair).copied().unwrap_or(false);
        if !active {
            return None;
        }
        world.embargoes.remove(&pair);
        world
            .trade_efficiency
            .insert(pair, self.baseline_efficiency);
        Some(DiplomacyEvent::TradeEmbargo {
            pair,
            imposed: false,
            tick: world.tick,
        })
    }

    /// Apply the embargo effect each tick: ensure any active embargo
    /// keeps trade efficiency at 0.
    pub fn apply(&self, world: &mut WorldState) -> Vec<DiplomacyEvent> {
        let active_pairs: Vec<Pair> = world
            .embargoes
            .iter()
            .filter(|(_, &active)| active)
            .map(|(&pair, _)| pair)
            .collect();

        for pair in active_pairs {
            // Enforce efficiency = 0 for embargoed pairs.
            if let Some(eff) = world.trade_efficiency.get_mut(&pair) {
                if *eff != 0 {
                    *eff = 0;
                }
            } else {
                world.trade_efficiency.insert(pair, 0);
            }
        }

        // No events emitted by the per-tick enforcement; events come from
        // impose/lift calls. This method exists for the uniform `apply`
        // interface.
        Vec::new()
    }
}

/// Military alliance effect: factions form alliances that grant shared
/// intelligence and combat bonuses. The combat bonus is a percentage
/// added to combat effectiveness (integer arithmetic, 0-100).
///
/// Alliance formation requires an explicit call; dissolution occurs via
/// a separate call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilitaryAllianceEffect {
    /// Combat bonus percentage granted by an alliance (0-100).
    pub combat_bonus: u32,
    /// Bitmask of intelligence categories shared between allies.
    pub intel_categories: u64,
}

impl Default for MilitaryAllianceEffect {
    fn default() -> Self {
        Self {
            combat_bonus: 15,
            intel_categories: 0b1111, // unit positions, strength, movement, supply
        }
    }
}

impl MilitaryAllianceEffect {
    /// Form a military alliance between two factions.
    ///
    /// Grants the combat bonus and shares intelligence. Returns the event
    /// if the alliance was newly formed (idempotent).
    pub fn form(&self, world: &mut WorldState, a: PolityId, b: PolityId) -> Option<DiplomacyEvent> {
        let pair = Pair::new(a, b);
        let already = world
            .military_alliances
            .get(&pair)
            .copied()
            .unwrap_or(false);
        if already {
            return None;
        }
        world.military_alliances.insert(pair, true);
        world.combat_bonuses.insert(pair, self.combat_bonus);
        world.shared_intel.insert(pair, self.intel_categories);
        Some(DiplomacyEvent::MilitaryAlliance {
            pair,
            formed: true,
            combat_bonus: self.combat_bonus,
            tick: world.tick,
        })
    }

    /// Dissolve a military alliance between two factions.
    ///
    /// Removes the combat bonus and shared intelligence. Returns the event
    /// if the alliance was active (idempotent).
    pub fn dissolve(
        &self,
        world: &mut WorldState,
        a: PolityId,
        b: PolityId,
    ) -> Option<DiplomacyEvent> {
        let pair = Pair::new(a, b);
        let active = world
            .military_alliances
            .get(&pair)
            .copied()
            .unwrap_or(false);
        if !active {
            return None;
        }
        world.military_alliances.remove(&pair);
        world.combat_bonuses.remove(&pair);
        world.shared_intel.remove(&pair);
        Some(DiplomacyEvent::MilitaryAlliance {
            pair,
            formed: false,
            combat_bonus: 0,
            tick: world.tick,
        })
    }

    /// Apply the military alliance effect each tick: ensure active
    /// alliances maintain their combat bonuses and shared intelligence.
    pub fn apply(&self, world: &mut WorldState) -> Vec<DiplomacyEvent> {
        let active_pairs: Vec<Pair> = world
            .military_alliances
            .iter()
            .filter(|(_, &active)| active)
            .map(|(&pair, _)| pair)
            .collect();

        for pair in active_pairs {
            // Ensure bonus and intel are present for active alliances.
            world
                .combat_bonuses
                .entry(pair)
                .or_insert(self.combat_bonus);
            world
                .shared_intel
                .entry(pair)
                .or_insert(self.intel_categories);
        }

        Vec::new()
    }
}

/// Tribute effect: dominant factions can demand tribute from weaker ones,
/// extracting resources each tick. Tribute can only flow from the weaker
/// faction (lower total resources) to the stronger one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TributeEffect {
    /// Maximum tribute amount that can be demanded per tick.
    pub max_tribute: u64,
}

impl Default for TributeEffect {
    fn default() -> Self {
        Self { max_tribute: 100 }
    }
}

impl TributeEffect {
    /// Impose a tribute relationship where `dominant` demands `amount`
    /// resources per tick from `vassal`.
    ///
    /// Returns `None` and does nothing if:
    /// - The vassal has equal or more resources than the dominant
    ///   (tribute requires a power imbalance).
    /// - The amount exceeds `max_tribute`.
    /// - A tribute relationship already exists for this pair.
    pub fn impose(
        &self,
        world: &mut WorldState,
        dominant: PolityId,
        vassal: PolityId,
        amount: u64,
    ) -> Option<DiplomacyEvent> {
        if dominant == vassal {
            return None;
        }
        if amount == 0 || amount > self.max_tribute {
            return None;
        }

        let pair = Pair::new(dominant, vassal);
        if world.tribute_relationships.contains_key(&pair) {
            return None;
        }

        let dom_res = world.resources.get(&dominant).copied().unwrap_or(0);
        let vas_res = world.resources.get(&vassal).copied().unwrap_or(0);

        // Tribute requires the vassal to be strictly weaker.
        if vas_res >= dom_res {
            return None;
        }

        world.tribute_relationships.insert(
            pair,
            TributeRelationship {
                dominant,
                vassal,
                amount,
            },
        );

        // Extract resources immediately on imposition.
        let actual = amount.min(vas_res);
        if actual > 0 {
            if let Some(r) = world.resources.get_mut(&vassal) {
                *r = r.saturating_sub(actual);
            }
            *world.resources.entry(dominant).or_insert(0) += actual;
        }

        Some(DiplomacyEvent::Tribute {
            from: vassal,
            to: dominant,
            amount: actual,
            tick: world.tick,
        })
    }

    /// Remove a tribute relationship.
    ///
    /// Returns `None` if no relationship exists (idempotent).
    pub fn release(
        &self,
        world: &mut WorldState,
        dominant: PolityId,
        vassal: PolityId,
    ) -> Option<TributeRelationship> {
        let pair = Pair::new(dominant, vassal);
        world.tribute_relationships.remove(&pair)
    }

    /// Apply the tribute effect each tick: extract the configured amount
    /// from each vassal to its dominant. Returns events for each
    /// extraction. If the vassal lacks resources, extraction is partial
    /// or skipped.
    pub fn apply(&self, world: &mut WorldState) -> Vec<DiplomacyEvent> {
        let tick = world.tick;
        let pairs: Vec<(Pair, TributeRelationship)> = world
            .tribute_relationships
            .iter()
            .map(|(&pair, &rel)| (pair, rel))
            .collect();

        let mut events = Vec::new();

        for (_pair, rel) in pairs {
            let dom = rel.dominant;
            let vas = rel.vassal;

            let vas_res = world.resources.get(&vas).copied().unwrap_or(0);
            let actual = rel.amount.min(vas_res);

            if actual == 0 {
                continue;
            }

            if let Some(r) = world.resources.get_mut(&vas) {
                *r = r.saturating_sub(actual);
            }
            *world.resources.entry(dom).or_insert(0) += actual;

            events.push(DiplomacyEvent::Tribute {
                from: vas,
                to: dom,
                amount: actual,
                tick,
            });
        }

        events
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    fn pair(a: u32, b: u32) -> Pair {
        Pair::new(p(a), p(b))
    }

    // -- CulturalInfluenceEffect tests ---------------------------------------

    #[test]
    fn cultural_drift_converges_profiles() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.cultural_profiles.insert(
            p(1),
            CulturalProfile {
                art: 10,
                religion: 20,
                technology: 30,
            },
        );
        world.cultural_profiles.insert(
            p(2),
            CulturalProfile {
                art: 90,
                religion: 80,
                technology: 70,
            },
        );
        world.shared_borders.insert(pair(1, 2), 5);

        let effect = CulturalInfluenceEffect { max_rate: 10 };
        let events = effect.apply(&mut world);

        assert_eq!(events.len(), 1);
        let p1 = world.cultural_profiles.get(&p(1)).unwrap();
        let p2 = world.cultural_profiles.get(&p(2)).unwrap();

        // Both profiles should have moved toward each other.
        assert!(p1.art > 10, "faction 1 art should increase");
        assert!(p2.art < 90, "faction 2 art should decrease");
        // Distance should be smaller.
        let original_dist = CulturalProfile {
            art: 10,
            religion: 20,
            technology: 30,
        }
        .distance(&CulturalProfile {
            art: 90,
            religion: 80,
            technology: 70,
        });
        assert!(p1.distance(p2) < original_dist);
    }

    #[test]
    fn cultural_no_shared_borders_no_effect() {
        let mut world = WorldState::default();
        world.tick = 1;
        world
            .cultural_profiles
            .insert(p(1), CulturalProfile::uniform(10));
        world
            .cultural_profiles
            .insert(p(2), CulturalProfile::uniform(90));
        // No shared borders.
        let effect = CulturalInfluenceEffect::default();
        let events = effect.apply(&mut world);
        assert!(events.is_empty());
    }

    #[test]
    fn cultural_missing_profile_skips_pair() {
        let mut world = WorldState::default();
        world.tick = 1;
        world
            .cultural_profiles
            .insert(p(1), CulturalProfile::uniform(10));
        // p(2) has no profile.
        world.shared_borders.insert(pair(1, 2), 5);

        let effect = CulturalInfluenceEffect::default();
        let events = effect.apply(&mut world);
        assert!(events.is_empty());
    }

    #[test]
    fn cultural_same_profiles_no_event() {
        let mut world = WorldState::default();
        world.tick = 1;
        world
            .cultural_profiles
            .insert(p(1), CulturalProfile::uniform(50));
        world
            .cultural_profiles
            .insert(p(2), CulturalProfile::uniform(50));
        world.shared_borders.insert(pair(1, 2), 5);

        let effect = CulturalInfluenceEffect::default();
        let events = effect.apply(&mut world);
        assert!(events.is_empty());
    }

    #[test]
    fn cultural_drift_is_symmetric() {
        let mut world_a = WorldState::default();
        world_a.tick = 1;
        world_a.cultural_profiles.insert(
            p(1),
            CulturalProfile {
                art: 0,
                religion: 0,
                technology: 0,
            },
        );
        world_a.cultural_profiles.insert(
            p(2),
            CulturalProfile {
                art: 100,
                religion: 100,
                technology: 100,
            },
        );
        world_a.shared_borders.insert(pair(1, 2), 5);

        let effect = CulturalInfluenceEffect { max_rate: 10 };
        effect.apply(&mut world_a);

        let a1 = *world_a.cultural_profiles.get(&p(1)).unwrap();
        let a2 = *world_a.cultural_profiles.get(&p(2)).unwrap();

        // With uniform profiles and symmetric drift, the distance covered
        // by each should be the same per axis.
        assert_eq!(a1.art, 100 - a2.art);
        assert_eq!(a1.religion, 100 - a2.religion);
        assert_eq!(a1.technology, 100 - a2.technology);
    }

    #[test]
    fn cultural_rate_clamped_to_max() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.cultural_profiles.insert(
            p(1),
            CulturalProfile {
                art: 0,
                religion: 0,
                technology: 0,
            },
        );
        world.cultural_profiles.insert(
            p(2),
            CulturalProfile {
                art: 100,
                religion: 100,
                technology: 100,
            },
        );
        // Many shared borders, but rate is capped at max_rate.
        world.shared_borders.insert(pair(1, 2), 500);

        let effect = CulturalInfluenceEffect { max_rate: 5 };
        let events = effect.apply(&mut world);

        assert_eq!(events.len(), 1);
        let p1 = world.cultural_profiles.get(&p(1)).unwrap();
        // Rate is capped at 5%, so art should move from 0 to 5.
        assert_eq!(p1.art, 5);
    }

    // -- TradeEmbargoEffect tests --------------------------------------------

    #[test]
    fn embargo_impose_sets_efficiency_to_zero() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.trade_efficiency.insert(pair(1, 2), 80);

        let effect = TradeEmbargoEffect::default();
        let event = effect.impose(&mut world, p(1), p(2));

        assert!(event.is_some());
        assert_eq!(world.trade_efficiency[&pair(1, 2)], 0);
        assert!(world.embargoes[&pair(1, 2)]);
    }

    #[test]
    fn embargo_impose_idempotent() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.trade_efficiency.insert(pair(1, 2), 80);

        let effect = TradeEmbargoEffect::default();
        let e1 = effect.impose(&mut world, p(1), p(2));
        let e2 = effect.impose(&mut world, p(1), p(2));

        assert!(e1.is_some());
        assert!(e2.is_none(), "second impose should be idempotent");
    }

    #[test]
    fn embargo_lift_restores_efficiency() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.trade_efficiency.insert(pair(1, 2), 80);

        let effect = TradeEmbargoEffect {
            baseline_efficiency: 50,
        };
        effect.impose(&mut world, p(1), p(2));
        let event = effect.lift(&mut world, p(1), p(2));

        assert!(event.is_some());
        assert_eq!(world.trade_efficiency[&pair(1, 2)], 50);
        assert!(!world.embargoes.contains_key(&pair(1, 2)));
    }

    #[test]
    fn embargo_lift_idempotent() {
        let mut world = WorldState::default();
        world.tick = 1;

        let effect = TradeEmbargoEffect::default();
        let event = effect.lift(&mut world, p(1), p(2));
        assert!(event.is_none(), "lifting non-existent embargo is noop");
    }

    #[test]
    fn embargo_apply_enforces_zero_efficiency() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.embargoes.insert(pair(1, 2), true);
        world.trade_efficiency.insert(pair(1, 2), 42);

        let effect = TradeEmbargoEffect::default();
        let events = effect.apply(&mut world);

        assert!(events.is_empty(), "apply enforcement emits no events");
        assert_eq!(world.trade_efficiency[&pair(1, 2)], 0);
    }

    #[test]
    fn embargo_symmetric_application() {
        let mut world = WorldState::default();
        world.tick = 1;

        let effect = TradeEmbargoEffect::default();
        // Impose from a->b
        effect.impose(&mut world, p(1), p(2));
        // Pair is canonical, so lifting from b->a should work.
        let event = effect.lift(&mut world, p(2), p(1));
        assert!(event.is_some());
    }

    // -- MilitaryAllianceEffect tests ----------------------------------------

    #[test]
    fn alliance_grants_bonus_and_intel() {
        let mut world = WorldState::default();
        world.tick = 1;

        let effect = MilitaryAllianceEffect {
            combat_bonus: 20,
            intel_categories: 0b1010,
        };
        let event = effect.form(&mut world, p(1), p(2));

        assert!(event.is_some());
        let ev = event.unwrap();
        match ev {
            DiplomacyEvent::MilitaryAlliance {
                pair: pp,
                formed,
                combat_bonus,
                tick: _,
            } => {
                assert!(formed);
                assert_eq!(combat_bonus, 20);
                assert_eq!(pp, pair(1, 2));
            }
            _ => panic!("expected MilitaryAlliance event"),
        }

        assert_eq!(world.combat_bonuses[&pair(1, 2)], 20);
        assert_eq!(world.shared_intel[&pair(1, 2)], 0b1010);
    }

    #[test]
    fn alliance_form_idempotent() {
        let mut world = WorldState::default();
        world.tick = 1;

        let effect = MilitaryAllianceEffect::default();
        let e1 = effect.form(&mut world, p(1), p(2));
        let e2 = effect.form(&mut world, p(1), p(2));

        assert!(e1.is_some());
        assert!(e2.is_none());
    }

    #[test]
    fn alliance_dissolve_removes_bonuses() {
        let mut world = WorldState::default();
        world.tick = 1;

        let effect = MilitaryAllianceEffect::default();
        effect.form(&mut world, p(1), p(2));
        let event = effect.dissolve(&mut world, p(1), p(2));

        assert!(event.is_some());
        let ev = event.unwrap();
        match ev {
            DiplomacyEvent::MilitaryAlliance {
                formed,
                combat_bonus,
                pair: pp,
                ..
            } => {
                assert!(!formed);
                assert_eq!(combat_bonus, 0);
                assert_eq!(pp, pair(1, 2));
            }
            _ => panic!("expected MilitaryAlliance event"),
        }

        assert!(!world.military_alliances.contains_key(&pair(1, 2)));
        assert!(!world.combat_bonuses.contains_key(&pair(1, 2)));
        assert!(!world.shared_intel.contains_key(&pair(1, 2)));
    }

    #[test]
    fn alliance_dissolve_idempotent() {
        let mut world = WorldState::default();
        world.tick = 1;

        let effect = MilitaryAllianceEffect::default();
        let event = effect.dissolve(&mut world, p(1), p(2));
        assert!(event.is_none());
    }

    #[test]
    fn alliance_apply_ensures_bonus_present() {
        let mut world = WorldState::default();
        world.tick = 1;
        // Alliance exists but bonus was somehow removed.
        world.military_alliances.insert(pair(1, 2), true);

        let effect = MilitaryAllianceEffect {
            combat_bonus: 25,
            intel_categories: 0xFF,
        };
        let events = effect.apply(&mut world);

        assert!(events.is_empty());
        assert_eq!(world.combat_bonuses[&pair(1, 2)], 25);
        assert_eq!(world.shared_intel[&pair(1, 2)], 0xFF);
    }

    #[test]
    fn alliance_event_tick_matches_world_tick() {
        let mut world = WorldState::default();
        world.tick = 42;

        let effect = MilitaryAllianceEffect::default();
        let event = effect.form(&mut world, p(1), p(2)).unwrap();
        match event {
            DiplomacyEvent::MilitaryAlliance { tick, .. } => assert_eq!(tick, 42),
            _ => panic!("expected MilitaryAlliance event"),
        }
    }

    // -- TributeEffect tests ------------------------------------------------

    #[test]
    fn tribute_impose_transfers_resources() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 500); // dominant
        world.resources.insert(p(2), 100); // vassal

        let effect = TributeEffect { max_tribute: 50 };
        let event = effect.impose(&mut world, p(1), p(2), 30);

        assert!(event.is_some());
        let ev = event.unwrap();
        match ev {
            DiplomacyEvent::Tribute {
                from, to, amount, ..
            } => {
                assert_eq!(amount, 30);
                assert_eq!(from, p(2));
                assert_eq!(to, p(1));
            }
            _ => panic!("expected Tribute event"),
        }

        assert_eq!(world.resources[&p(1)], 530);
        assert_eq!(world.resources[&p(2)], 70);
    }

    #[test]
    fn tribute_requires_power_imbalance() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 50);
        world.resources.insert(p(2), 100); // vassal is stronger

        let effect = TributeEffect { max_tribute: 50 };
        let event = effect.impose(&mut world, p(1), p(2), 10);
        assert!(event.is_none(), "vassal cannot be richer than dominant");
    }

    #[test]
    fn tribute_requires_strictly_weaker_vassal() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 100);
        world.resources.insert(p(2), 100); // equal resources

        let effect = TributeEffect { max_tribute: 50 };
        let event = effect.impose(&mut world, p(1), p(2), 10);
        assert!(event.is_none(), "equal resources means no tribute");
    }

    #[test]
    fn tribute_capped_by_max() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 1000);
        world.resources.insert(p(2), 500);

        let effect = TributeEffect { max_tribute: 50 };
        let event = effect.impose(&mut world, p(1), p(2), 200);
        assert!(event.is_none(), "amount exceeds max_tribute");
    }

    #[test]
    fn tribute_capped_by_available_resources() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 1000);
        world.resources.insert(p(2), 5); // very poor vassal

        let effect = TributeEffect { max_tribute: 100 };
        let event = effect.impose(&mut world, p(1), p(2), 50);

        assert!(event.is_some());
        let ev = event.unwrap();
        match ev {
            DiplomacyEvent::Tribute { amount, .. } => {
                assert_eq!(amount, 5, "should only take what vassal has");
            }
            _ => panic!("expected Tribute event"),
        }
        assert_eq!(world.resources[&p(2)], 0);
        assert_eq!(world.resources[&p(1)], 1005);
    }

    #[test]
    fn tribute_per_tick_extraction() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 500);
        world.resources.insert(p(2), 1000);

        let effect = TributeEffect { max_tribute: 20 };
        // p(1) has fewer resources, so it becomes the vassal when
        // p(2) is dominant.
        effect.impose(&mut world, p(2), p(1), 10);
        // impose already extracted 10: p(1)=490, p(2)=1010
        assert_eq!(world.resources[&p(1)], 490);
        assert_eq!(world.resources[&p(2)], 1010);

        // Tick 1: extract another 10
        world.tick = 1;
        let events = effect.apply(&mut world);
        assert_eq!(events.len(), 1);
        assert_eq!(world.resources[&p(1)], 480);
        assert_eq!(world.resources[&p(2)], 1020);

        // Tick 2: extract again
        world.tick = 2;
        let events = effect.apply(&mut world);
        assert_eq!(events.len(), 1);
        assert_eq!(world.resources[&p(1)], 470);
        assert_eq!(world.resources[&p(2)], 1030);
    }

    #[test]
    fn tribute_stops_when_vassal_broke() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 15); // very poor vassal
        world.resources.insert(p(2), 500); // rich dominant

        let effect = TributeEffect { max_tribute: 10 };
        effect.impose(&mut world, p(2), p(1), 10);
        // impose already extracted 10: p(1)=5, p(2)=510
        assert_eq!(world.resources[&p(1)], 5);
        assert_eq!(world.resources[&p(2)], 510);

        // Tick 1: extract only 5 (partial - vassal nearly broke)
        world.tick = 1;
        let events = effect.apply(&mut world);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DiplomacyEvent::Tribute { amount, .. } => {
                assert_eq!(*amount, 5);
            }
            _ => panic!("expected Tribute event"),
        }
        assert_eq!(world.resources[&p(1)], 0);

        // Tick 2: no extraction (nothing left)
        world.tick = 2;
        let events = effect.apply(&mut world);
        assert!(events.is_empty());
        assert_eq!(world.resources[&p(1)], 0);
    }

    #[test]
    fn tribute_release_removes_relationship() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 500);
        world.resources.insert(p(2), 100);

        let effect = TributeEffect::default();
        effect.impose(&mut world, p(1), p(2), 20);
        let removed = effect.release(&mut world, p(1), p(2));
        assert!(removed.is_some());

        // No more extractions.
        world.tick = 2;
        let events = effect.apply(&mut world);
        assert!(events.is_empty());
    }

    #[test]
    fn tribute_self_is_noop() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 100);

        let effect = TributeEffect::default();
        let event = effect.impose(&mut world, p(1), p(1), 10);
        assert!(event.is_none());
    }

    #[test]
    fn tribute_impose_idempotent() {
        let mut world = WorldState::default();
        world.tick = 1;
        world.resources.insert(p(1), 500);
        world.resources.insert(p(2), 100);

        let effect = TributeEffect::default();
        let e1 = effect.impose(&mut world, p(1), p(2), 10);
        let e2 = effect.impose(&mut world, p(1), p(2), 10);
        assert!(e1.is_some());
        assert!(e2.is_none());
    }

    // -- Determinism tests ---------------------------------------------------

    #[test]
    fn cultural_influence_is_deterministic() {
        let build = || {
            let mut world = WorldState::default();
            world.tick = 5;
            world.cultural_profiles.insert(
                p(1),
                CulturalProfile {
                    art: 10,
                    religion: 20,
                    technology: 30,
                },
            );
            world.cultural_profiles.insert(
                p(2),
                CulturalProfile {
                    art: 90,
                    religion: 80,
                    technology: 70,
                },
            );
            world.shared_borders.insert(pair(1, 2), 3);
            let effect = CulturalInfluenceEffect { max_rate: 8 };
            let events = effect.apply(&mut world);
            (world, events)
        };

        let (w1, e1) = build();
        let (w2, e2) = build();
        assert_eq!(w1, w2);
        assert_eq!(e1, e2);
    }

    #[test]
    fn tribute_extraction_is_deterministic() {
        let build = || {
            let mut world = WorldState::default();
            world.tick = 10;
            world.resources.insert(p(1), 500);
            world.resources.insert(p(2), 1000);
            let effect = TributeEffect { max_tribute: 30 };
            effect.impose(&mut world, p(2), p(1), 15);
            world.tick = 11;
            let events = effect.apply(&mut world);
            (world, events)
        };

        let (w1, e1) = build();
        let (w2, e2) = build();
        assert_eq!(w1, w2);
        assert_eq!(e1, e2);
    }

    // -- CulturalProfile drift tests -----------------------------------------

    #[test]
    fn drift_axis_moves_toward_target() {
        assert_eq!(drift_axis(0, 100, 10), 10);
        assert_eq!(drift_axis(100, 0, 10), 90);
        assert_eq!(drift_axis(50, 50, 10), 50);
    }

    #[test]
    fn drift_axis_clamped_to_bounds() {
        // 10% drift from 95 toward 100: diff=5, shift=5*10/100=0 (integer div)
        assert_eq!(drift_axis(95, 100, 10), 95);
        // 100% drift from 95 toward 100: diff=5, shift=5*100/100=5
        assert_eq!(drift_axis(95, 100, 100), 100);
        // Drift toward 0
        assert_eq!(drift_axis(5, 0, 10), 5);
        // 100% drift toward 0
        assert_eq!(drift_axis(5, 0, 100), 0);
    }

    #[test]
    fn cultural_profile_distance_is_symmetric() {
        let a = CulturalProfile {
            art: 10,
            religion: 20,
            technology: 30,
        };
        let b = CulturalProfile {
            art: 90,
            religion: 80,
            technology: 70,
        };
        assert_eq!(a.distance(&b), b.distance(&a));
    }

    #[test]
    fn cultural_profile_drift_toward_same_is_identity() {
        let a = CulturalProfile {
            art: 42,
            religion: 55,
            technology: 77,
        };
        let drifted = a.drift_toward(&a, 50);
        assert_eq!(drifted, a);
    }

    // -- Integration test: multiple effects on one world ---------------------

    #[test]
    fn combined_effects_on_shared_world() {
        let mut world = WorldState::default();
        world.tick = 1;

        // Set up cultural profiles, trade, and resources.
        world.cultural_profiles.insert(
            p(1),
            CulturalProfile {
                art: 10,
                religion: 10,
                technology: 10,
            },
        );
        world.cultural_profiles.insert(
            p(2),
            CulturalProfile {
                art: 90,
                religion: 90,
                technology: 90,
            },
        );
        world.shared_borders.insert(pair(1, 2), 3);
        world.trade_efficiency.insert(pair(1, 2), 80);
        world.resources.insert(p(1), 1000);
        world.resources.insert(p(2), 200);

        // Cultural influence.
        let cultural = CulturalInfluenceEffect { max_rate: 10 };
        let cultural_events = cultural.apply(&mut world);
        assert_eq!(cultural_events.len(), 1);

        // Embargo.
        let embargo = TradeEmbargoEffect::default();
        let embargo_event = embargo.impose(&mut world, p(1), p(2));
        assert!(embargo_event.is_some());
        assert_eq!(world.trade_efficiency[&pair(1, 2)], 0);

        // Military alliance.
        let alliance = MilitaryAllianceEffect::default();
        let alliance_event = alliance.form(&mut world, p(1), p(2));
        assert!(alliance_event.is_some());

        // Tribute: p(1) is richer, p(2) is vassal.
        let tribute = TributeEffect { max_tribute: 50 };
        let tribute_event = tribute.impose(&mut world, p(1), p(2), 25);
        assert!(tribute_event.is_some());
        assert_eq!(world.resources[&p(2)], 175);
        assert_eq!(world.resources[&p(1)], 1025);

        // All state is consistent.
        assert_eq!(world.cultural_profiles.len(), 2);
        assert_eq!(world.embargoes.len(), 1);
        assert_eq!(world.military_alliances.len(), 1);
        assert_eq!(world.tribute_relationships.len(), 1);
    }
}
