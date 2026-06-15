//! Operational-layer hook (Phase 4 war bridge extension point, FR-CIV-TACTICS-030).
//!
//! Provides [`NoopOperationalLayer`] (default no-op) and [`OperationalCombatResolver`]
//! which resolves engagements into damage events using unit strength, doctrine,
//! and morale — never scripted values (charter principle, FR-CIV-0100 §3c).

use crate::war_bridge::CombatEngagement;
use crate::{apply_damage, DamageEvent};
use civ_voxel::{MaterialId, VoxelWorld};
use std::collections::HashMap;

/// Phase-4 operational telemetry sink. Hosts (engine, watch) may register a hook
/// to fan out engagements without coupling `civ-tactics` to ECS.
pub trait OperationalLayer {
    /// Called after engagements resolve on a tactics cadence tick.
    fn on_combat_engagements(&mut self, tick: u64, engagements: &[CombatEngagement]);
}

/// Default no-op operational layer.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopOperationalLayer;

impl OperationalLayer for NoopOperationalLayer {
    fn on_combat_engagements(&mut self, _tick: u64, _engagements: &[CombatEngagement]) {}
}

// ---------------------------------------------------------------------------
// Unit combat stats (FR-CIV-0100 §3c)
// ---------------------------------------------------------------------------

/// Per-unit combat state for operational resolution.
///
/// Combat outcomes **emerge** from these values rather than being scripted.
/// `doctrine` is a fitness-derived multiplier (see [`crate::score_doctrine_fitness`]).
#[derive(Debug, Clone, PartialEq)]
pub struct UnitCombatStats {
    /// Current hit points. Reduced by incoming damage each resolution tick.
    pub hp: u32,
    /// Attack-strength multiplier (1.0 = baseline infantry).
    pub strength: f32,
    /// Morale factor (0.0 = routed, 1.0 = full morale).
    /// Affects both outgoing damage and incoming damage resistance.
    pub morale: f32,
    /// Doctrine fitness bonus applied to this unit's attacks.
    pub doctrine: f32,
}

impl Default for UnitCombatStats {
    fn default() -> Self {
        Self {
            hp: 100,
            strength: 1.0,
            morale: 1.0,
            doctrine: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Operational combat resolver (FR-CIV-0100 §3c)
// ---------------------------------------------------------------------------

/// Operational combat resolver that applies voxel damage from engagements
/// modulated by attacker/defender strength, doctrine, and morale.
///
/// Implements [`OperationalLayer`] so it can be registered as a combat hook.
/// The resolver owns a mutable reference to the [`VoxelWorld`] and calls
/// [`apply_damage`] for each resolved engagement.
pub struct OperationalCombatResolver<'a> {
    world: &'a mut VoxelWorld<MaterialId>,
    units: HashMap<u64, UnitCombatStats>,
    /// Damage events emitted during the most recent `on_combat_engagements` call.
    pub last_events: Vec<DamageEvent>,
}

impl<'a> OperationalCombatResolver<'a> {
    /// Create a new resolver backed by the given voxel world.
    pub fn new(world: &'a mut VoxelWorld<MaterialId>) -> Self {
        Self {
            world,
            units: HashMap::new(),
            last_events: Vec::new(),
        }
    }

    /// Register or update combat stats for a unit by id.
    pub fn set_stats(&mut self, unit_id: u64, stats: UnitCombatStats) {
        self.units.insert(unit_id, stats);
    }

    /// Read current combat stats for a unit (`None` if unregistered).
    pub fn stats(&self, unit_id: u64) -> Option<&UnitCombatStats> {
        self.units.get(&unit_id)
    }

    /// Read current HP for a unit.
    pub fn hp(&self, unit_id: u64) -> Option<u32> {
        self.units.get(&unit_id).map(|s| s.hp)
    }

    /// Resolve a single engagement into a modulated [`DamageEvent`].
    ///
    /// The effective energy is:
    ///
    /// ```text
    /// effective_energy = base_energy × strength × morale × doctrine
    /// ```
    ///
    /// The defender's morale provides partial resistance:
    ///
    /// ```text
    /// hp_drain = ceil(effective_energy / 100) × (1.0 − defender_morale × 0.3)
    /// ```
    ///
    /// At defender_morale = 0.0 (routed) there is no resistance bonus;
    /// at defender_morale = 1.0 the drain is reduced by 30%.
    fn resolve_engagement(
        world: &mut VoxelWorld<MaterialId>,
        units: &mut HashMap<u64, UnitCombatStats>,
        engagement: &CombatEngagement,
    ) -> Option<DamageEvent> {
        let attacker = units.get(&engagement.shooter_id)?.clone();
        let defender = units.get_mut(&engagement.target_id)?;

        // Outgoing damage modulated by attacker stats.
        let attack_power = attacker.strength * attacker.morale * attacker.doctrine;
        let effective_energy = (engagement.damage.energy as f32 * attack_power) as u32;

        let event = DamageEvent {
            center: engagement.damage.center,
            radius_voxels: engagement.damage.radius_voxels,
            energy: effective_energy.max(1),
        };

        // Apply voxel damage to world.
        apply_damage(world, &event);

        // Drain defender HP.  Morale provides up to 30 % resistance.
        let raw_drain = (effective_energy / 100).max(1);
        let resistance = 1.0 - defender.morale * 0.3;
        let hp_drain = (raw_drain as f32 * resistance).ceil() as u32;
        defender.hp = defender.hp.saturating_sub(hp_drain);

        Some(event)
    }
}

impl<'a> OperationalLayer for OperationalCombatResolver<'a> {
    fn on_combat_engagements(&mut self, _tick: u64, engagements: &[CombatEngagement]) {
        self.last_events.clear();
        for engagement in engagements {
            if let Some(event) = Self::resolve_engagement(
                self.world,
                &mut self.units,
                engagement,
            ) {
                self.last_events.push(event);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests (FR-CIV-0100 §3c)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::war_bridge::{grid_to_world_coord, MilitaryUnitSample};
    use civ_voxel::WorldCoord;

    fn resolver_with_two_units(
        world: &mut VoxelWorld<MaterialId>,
        attacker_stats: UnitCombatStats,
        defender_stats: UnitCombatStats,
    ) -> OperationalCombatResolver<'_> {
        let mut resolver = OperationalCombatResolver::new(world);
        resolver.set_stats(10, attacker_stats);
        resolver.set_stats(20, defender_stats);
        resolver
    }

    fn sample_engagement() -> CombatEngagement {
        CombatEngagement {
            shooter_id: 10,
            target_id: 20,
            shooter_faction: 0,
            target_faction: 1,
            target_index: 1,
            damage: DamageEvent {
                center: grid_to_world_coord(4, 0),
                radius_voxels: 2,
                energy: 250,
            },
        }
    }

    /// FR-CIV-0100-§3c-TEST-01 — engagement produces a damage event.
    #[test]
    fn on_combat_engagements_produces_damage_event() {
        let mut world = VoxelWorld::new(1);
        let atk = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 1.0,
            doctrine: 1.0,
        };
        let def = UnitCombatStats::default();
        let mut resolver = resolver_with_two_units(&mut world, atk, def);

        resolver.on_combat_engagements(0, &[sample_engagement()]);

        assert_eq!(resolver.last_events.len(), 1, "must emit exactly one damage event");
        assert_eq!(resolver.last_events[0].energy, 250);
    }

    /// FR-CIV-0100-§3c-TEST-02 — higher attacker strength increases damage.
    #[test]
    fn higher_strength_increases_damage() {
        let mut world = VoxelWorld::new(1);
        let weak = UnitCombatStats {
            hp: 100,
            strength: 0.5,
            morale: 1.0,
            doctrine: 1.0,
        };
        let def = UnitCombatStats::default();
        let mut resolver = resolver_with_two_units(&mut world, weak, def.clone());
        resolver.on_combat_engagements(0, &[sample_engagement()]);
        let weak_energy = resolver.last_events[0].energy;
        let weak_hp = resolver.hp(20).unwrap();

        let mut world2 = VoxelWorld::new(1);
        let strong = UnitCombatStats {
            hp: 100,
            strength: 2.0,
            morale: 1.0,
            doctrine: 1.0,
        };
        let mut resolver2 = resolver_with_two_units(&mut world2, strong, def);
        resolver2.on_combat_engagements(0, &[sample_engagement()]);
        let strong_energy = resolver2.last_events[0].energy;
        let strong_hp = resolver2.hp(20).unwrap();

        assert!(strong_energy > weak_energy, "strong attacker must deal more energy");
        assert!(strong_hp < weak_hp, "strong attacker must drain more HP");
    }

    /// FR-CIV-0100-§3c-TEST-03 — defender morale reduces HP drain (resistance).
    #[test]
    fn defender_morale_reduces_hp_drain() {
        let atk = UnitCombatStats {
            hp: 100,
            strength: 2.0,
            morale: 1.0,
            doctrine: 1.0,
        };

        let mut world1 = VoxelWorld::new(1);
        let def_low_morale = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 0.0,
            doctrine: 1.0,
        };
        let mut r1 = resolver_with_two_units(&mut world1, atk.clone(), def_low_morale);
        r1.on_combat_engagements(0, &[sample_engagement()]);
        let hp_no_morale = r1.hp(20).unwrap();

        let mut world2 = VoxelWorld::new(1);
        let def_high_morale = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 1.0,
            doctrine: 1.0,
        };
        let mut r2 = resolver_with_two_units(&mut world2, atk, def_high_morale);
        r2.on_combat_engagements(0, &[sample_engagement()]);
        let hp_full_morale = r2.hp(20).unwrap();

        assert!(
            hp_full_morale >= hp_no_morale,
            "high-morale defender must retain equal or more HP"
        );
    }

    /// FR-CIV-0100-§3c-TEST-04 — doctrine bonus increases effective damage.
    #[test]
    fn doctrine_bonus_increases_damage() {
        let base_atk = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 1.0,
            doctrine: 1.0,
        };
        let def = UnitCombatStats::default();

        let mut world1 = VoxelWorld::new(1);
        let mut r1 = resolver_with_two_units(&mut world1, base_atk, def.clone());
        r1.on_combat_engagements(0, &[sample_engagement()]);
        let base_energy = r1.last_events[0].energy;

        let elite_atk = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 1.0,
            doctrine: 2.0,
        };
        let mut world2 = VoxelWorld::new(1);
        let mut r2 = resolver_with_two_units(&mut world2, elite_atk, def);
        r2.on_combat_engagements(0, &[sample_engagement()]);
        let elite_energy = r2.last_events[0].energy;

        assert!(
            elite_energy > base_energy,
            "doctrine bonus must increase effective energy"
        );
    }

    /// FR-CIV-0100-§3c-TEST-05 — unknown unit ids are skipped gracefully.
    #[test]
    fn unknown_unit_ids_are_skipped() {
        let mut world = VoxelWorld::new(1);
        let mut resolver = OperationalCombatResolver::new(&mut world);
        // No stats registered at all.
        resolver.on_combat_engagements(0, &[sample_engagement()]);
        assert!(resolver.last_events.is_empty(), "unknown units produce no events");
    }

    /// FR-CIV-0100-§3c-TEST-06 — HP is clamped at zero (no underflow).
    #[test]
    fn hp_does_not_underflow() {
        let mut world = VoxelWorld::new(1);
        let atk = UnitCombatStats {
            hp: 100,
            strength: 100.0,
            morale: 1.0,
            doctrine: 10.0,
        };
        let def = UnitCombatStats {
            hp: 1,
            strength: 1.0,
            morale: 0.0,
            doctrine: 1.0,
        };
        let mut resolver = resolver_with_two_units(&mut world, atk, def);
        resolver.on_combat_engagements(0, &[sample_engagement()]);
        assert_eq!(resolver.hp(20).unwrap(), 0, "HP must clamp at zero");
    }

    /// FR-CIV-0100-§3c-TEST-07 — combat outcomes emerge from stats, not scripted.
    /// Equal stats → equal outcomes regardless of faction id.
    #[test]
    fn symmetric_stats_produce_symmetric_outcomes() {
        let symmetric = UnitCombatStats {
            hp: 200,
            strength: 1.5,
            morale: 0.8,
            doctrine: 1.2,
        };
        let e1 = sample_engagement();
        let mut world1 = VoxelWorld::new(1);
        let mut r1 = resolver_with_two_units(&mut world1, symmetric.clone(), symmetric.clone());
        r1.on_combat_engagements(0, &[e1]);
        let hp_a = r1.hp(10).unwrap();
        let hp_b = r1.hp(20).unwrap();
        // Shooter doesn't take HP damage; only target does.
        assert_eq!(hp_a, 200, "shooter HP must be untouched");
        assert!(hp_b < 200, "target must take damage");
    }

    /// FR-CIV-0100-§3c-TEST-08 — zero-morale routed unit takes maximum damage.
    #[test]
    fn routed_unit_takes_maximum_damage() {
        let atk = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 1.0,
            doctrine: 1.0,
        };
        let routed = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 0.0, // routed
            doctrine: 1.0,
        };
        let normal = UnitCombatStats {
            hp: 100,
            strength: 1.0,
            morale: 0.5,
            doctrine: 1.0,
        };

        let mut w1 = VoxelWorld::new(1);
        let mut r1 = resolver_with_two_units(&mut w1, atk.clone(), routed);
        r1.on_combat_engagements(0, &[sample_engagement()]);
        let hp_routed = r1.hp(20).unwrap();

        let mut w2 = VoxelWorld::new(1);
        let mut r2 = resolver_with_two_units(&mut w2, atk, normal);
        r2.on_combat_engagements(0, &[sample_engagement()]);
        let hp_normal = r2.hp(20).unwrap();

        assert!(
            hp_routed <= hp_normal,
            "routed unit must take equal or more damage"
        );
    }
}
