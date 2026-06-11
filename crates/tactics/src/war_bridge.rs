//! Phase-4 war bridge: military grid positions → per-soldier combat + voxel damage (FR-CIV-TACTICS-022/024).
//!
//! [`WarBridge`] owns a reference to the voxel world and exposes two primary
//! methods:
//! - [`WarBridge::resolve_combat`] — runs the engagement tick with per-unit LOS
//!   gating; a shooter never fires at a target it cannot see.
//! - [`WarBridge::formation_move`] — returns the target grid positions for a
//!   squad moving in formation, computed via [`formation_positions`].

use crate::fog_of_war::FogOfWar;
use crate::formation::{formation_positions, Facing, FormationKind};
use crate::los::line_of_sight;
use crate::DamageEvent;
use civ_voxel::{MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE};

/// Minimal military unit sample for the war bridge (grid plane).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MilitaryUnitSample {
    /// Stable pin id (matches `civ-server` military pin ids).
    pub unit_id: u64,
    /// Owning faction id.
    pub faction_id: u32,
    /// Grid X (hex plane).
    pub grid_x: i32,
    /// Grid Y (hex plane).
    pub grid_y: i32,
}

/// Per-soldier engagement resolved on the war bridge cadence (FR-CIV-TACTICS-024).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatEngagement {
    /// Shooter pin id.
    pub shooter_id: u64,
    /// Target pin id.
    pub target_id: u64,
    /// Shooter faction.
    pub shooter_faction: u32,
    /// Target faction.
    pub target_faction: u32,
    /// Voxel damage queued for the target cell.
    pub damage: DamageEvent,
    /// Index into the `MilitaryUnitSample` slice for strength application.
    pub target_index: usize,
}

/// Cadence and combat parameters for the war → tactics bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarBridgeConfig {
    /// Run engagement resolution when `tick % cadence_ticks == 0`.
    pub cadence_ticks: u64,
    /// Manhattan engage range on the grid plane.
    pub engage_range_grid: i32,
    /// Voxel damage radius for a successful engagement.
    pub damage_radius_voxels: u8,
    /// Energy passed through to [`DamageEvent`].
    pub damage_energy: u32,
    /// Fixed-point strength drained from the target (`civ-engine` Fixed scale 1e6).
    pub strength_damage_fixed: u32,
    /// When `Some`, shooters may only engage targets visible on their faction fog grid (FR-CIV-TACTICS-042).
    pub fog_vision_radius: Option<u32>,
    /// Square fog grid edge length when fog is enabled (clamped 16..=256).
    pub fog_grid_size: u32,
}

impl Default for WarBridgeConfig {
    fn default() -> Self {
        Self {
            cadence_ticks: 16,
            engage_range_grid: 8,
            damage_radius_voxels: 2,
            damage_energy: 250,
            strength_damage_fixed: 50_000,
            fog_vision_radius: None,
            fog_grid_size: 64,
        }
    }
}

/// Map a grid cell to a voxel world coordinate (deterministic, Y-up voxel axis).
pub fn grid_to_world_coord(grid_x: i32, grid_y: i32) -> WorldCoord {
    let step = FIXED_SCALE / 16;
    WorldCoord {
        x: i64::from(grid_x) * step,
        y: 0,
        z: i64::from(grid_y) * step,
    }
}

fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// Build an updated fog grid when [`WarBridgeConfig::fog_vision_radius`] is set.
#[must_use]
pub fn build_fog_for_units(
    config: &WarBridgeConfig,
    units: &[MilitaryUnitSample],
    world: &VoxelWorld<MaterialId>,
) -> Option<FogOfWar> {
    let radius = config.fog_vision_radius?;
    let extent = units
        .iter()
        .map(|u| u.grid_x.unsigned_abs().max(u.grid_y.unsigned_abs()))
        .max()
        .unwrap_or(0);
    let grid_size = config.fog_grid_size.max(extent + 8).clamp(16, 256);
    let mut fog = FogOfWar::new(grid_size, Some(radius));
    fog.update(units, world);
    Some(fog)
}

/// Resolve cross-faction engagements with per-soldier ids and LOS.
pub fn tick_war_bridge(
    tick: u64,
    config: &WarBridgeConfig,
    units: &[MilitaryUnitSample],
    world: &VoxelWorld<MaterialId>,
    fog: Option<&FogOfWar>,
) -> Vec<CombatEngagement> {
    if config.cadence_ticks == 0 || tick % config.cadence_ticks != 0 {
        return Vec::new();
    }
    let range = config.engage_range_grid.max(1);
    let mut engagements = Vec::new();
    let mut damaged_targets = Vec::new();

    for (i, shooter) in units.iter().enumerate() {
        let from = grid_to_world_coord(shooter.grid_x, shooter.grid_y);
        let mut best: Option<(usize, i32)> = None;
        for (j, target) in units.iter().enumerate() {
            if i == j || shooter.faction_id == target.faction_id {
                continue;
            }
            if damaged_targets.contains(&j) {
                continue;
            }
            let dist = manhattan(
                (shooter.grid_x, shooter.grid_y),
                (target.grid_x, target.grid_y),
            );
            if dist > range {
                continue;
            }
            let to = grid_to_world_coord(target.grid_x, target.grid_y);
            if !line_of_sight(world, from, to) {
                continue;
            }
            if let Some(fog) = fog {
                let cell = (target.grid_x, target.grid_y);
                if !fog.is_visible(shooter.faction_id, cell) {
                    continue;
                }
            }
            match best {
                None => best = Some((j, dist)),
                Some((_, best_dist)) if dist < best_dist => best = Some((j, dist)),
                _ => {}
            }
        }
        if let Some((target_idx, _)) = best {
            let target = &units[target_idx];
            damaged_targets.push(target_idx);
            engagements.push(CombatEngagement {
                shooter_id: shooter.unit_id,
                target_id: target.unit_id,
                shooter_faction: shooter.faction_id,
                target_faction: target.faction_id,
                target_index: target_idx,
                damage: DamageEvent {
                    center: grid_to_world_coord(target.grid_x, target.grid_y),
                    radius_voxels: config.damage_radius_voxels,
                    energy: config.damage_energy,
                },
            });
        }
    }

    engagements
}

// ---------------------------------------------------------------------------
// WarBridge struct
// ---------------------------------------------------------------------------

/// Bridges the tactics system (LOS, formations) with the voxel simulation
/// engine (FR-CIV-TACTICS-022/024).
///
/// `WarBridge` holds a reference to the [`VoxelWorld`] so that callers do not
/// need to plumb the world into every call site.  Two primary methods are
/// exposed:
///
/// * [`WarBridge::resolve_combat`] — runs an engagement tick; every attacker
///   must have clear LOS to its chosen target before a [`CombatEngagement`] is
///   produced.
/// * [`WarBridge::formation_move`] — returns the absolute target grid positions
///   for a squad moving into the requested formation.
pub struct WarBridge<'world> {
    world: &'world VoxelWorld<MaterialId>,
    config: WarBridgeConfig,
}

impl<'world> WarBridge<'world> {
    /// Construct a new bridge backed by `world` with the given `config`.
    pub fn new(world: &'world VoxelWorld<MaterialId>, config: WarBridgeConfig) -> Self {
        Self { world, config }
    }

    /// Construct a bridge with [`WarBridgeConfig::default`].
    pub fn with_defaults(world: &'world VoxelWorld<MaterialId>) -> Self {
        Self::new(world, WarBridgeConfig::default())
    }

    /// Resolve cross-faction engagements for `tick`.
    ///
    /// Delegates to [`tick_war_bridge`]; LOS is checked per shooter–target pair
    /// using the voxel world held by this bridge.
    ///
    /// Returns an empty `Vec` when the tick does not fall on the configured
    /// cadence boundary.
    pub fn resolve_combat(
        &self,
        tick: u64,
        units: &[MilitaryUnitSample],
        fog: Option<&FogOfWar>,
    ) -> Vec<CombatEngagement> {
        tick_war_bridge(tick, &self.config, units, self.world, fog)
    }

    /// Compute the **absolute target grid positions** for `units` when they
    /// move into `formation` facing `facing`.
    ///
    /// The formation is anchored at the centroid of the current unit positions.
    /// Each returned position corresponds to the same slot index in `units`.
    ///
    /// Returns an empty `Vec` for an empty squad.
    pub fn formation_move(
        &self,
        units: &[MilitaryUnitSample],
        formation: FormationKind,
        facing: Facing,
    ) -> Vec<(i32, i32)> {
        if units.is_empty() {
            return Vec::new();
        }
        let positions: Vec<(i32, i32)> = units.iter().map(|u| (u.grid_x, u.grid_y)).collect();
        formation_positions(&positions, formation, facing)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::WorldCoord;

    fn empty_world() -> VoxelWorld<MaterialId> {
        VoxelWorld::new(1)
    }

    fn two_enemy_units() -> [MilitaryUnitSample; 2] {
        [
            MilitaryUnitSample {
                unit_id: 1,
                faction_id: 0,
                grid_x: 0,
                grid_y: 0,
            },
            MilitaryUnitSample {
                unit_id: 2,
                faction_id: 1,
                grid_x: 4,
                grid_y: 0,
            },
        ]
    }

    fn bridge_config_immediate() -> WarBridgeConfig {
        WarBridgeConfig {
            cadence_ticks: 1,
            engage_range_grid: 16,
            ..WarBridgeConfig::default()
        }
    }

    // -----------------------------------------------------------------------
    // FR-CIV-TACTICS-022 — LOS-gated combat resolution
    // -----------------------------------------------------------------------

    /// Attack is blocked when solid voxels fill the entire path between shooter
    /// and target.
    #[test]
    fn attack_blocked_when_no_los() {
        let mut world = empty_world();
        let units = two_enemy_units();
        // Fill the grid cells between the two units with solid voxels.
        // grid_to_world_coord maps grid steps to voxel coordinates; we place a
        // wall directly in the world-coordinate corridor.
        let shooter_wc = grid_to_world_coord(0, 0);
        let target_wc = grid_to_world_coord(4, 0);
        // Step halfway along x in world coords.
        let mid_x = (shooter_wc.x + target_wc.x) / 2;
        world.write(
            WorldCoord {
                x: mid_x,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );

        let bridge = WarBridge::new(&world, bridge_config_immediate());
        let engagements = bridge.resolve_combat(1, &units, None);
        // Neither unit should be able to fire; the wall blocks both directions.
        assert!(
            engagements.is_empty(),
            "expected no engagements when LOS is blocked, got {:?}",
            engagements
        );
    }

    /// Attack succeeds when the path between shooter and target is clear.
    #[test]
    fn attack_succeeds_with_clear_los() {
        let world = empty_world(); // no obstacles
        let units = two_enemy_units();

        let bridge = WarBridge::new(&world, bridge_config_immediate());
        let engagements = bridge.resolve_combat(1, &units, None);
        assert_eq!(
            engagements.len(),
            2,
            "both units should engage each other in clear space"
        );
        assert!(engagements
            .iter()
            .any(|e| e.shooter_id == 1 && e.target_id == 2));
        assert!(engagements
            .iter()
            .any(|e| e.shooter_id == 2 && e.target_id == 1));
    }

    // -----------------------------------------------------------------------
    // FR-CIV-TACTICS-024 — formation movement produces valid positions
    // -----------------------------------------------------------------------

    /// A squad in Line formation produces distinct positions equal in count to
    /// the number of units.
    #[test]
    fn formation_move_line_produces_valid_positions() {
        let world = empty_world();
        let units: Vec<MilitaryUnitSample> = (0..4)
            .map(|i| MilitaryUnitSample {
                unit_id: i as u64,
                faction_id: 0,
                grid_x: i,
                grid_y: 0,
            })
            .collect();

        let bridge = WarBridge::with_defaults(&world);
        let positions = bridge.formation_move(&units, FormationKind::Line, Facing::East);

        assert_eq!(
            positions.len(),
            units.len(),
            "position count matches unit count"
        );

        // No duplicates.
        let mut sorted = positions.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), positions.len(), "positions are unique");
    }

    /// A squad in Column formation produces positions ordered along the
    /// dominant axis.
    #[test]
    fn formation_move_column_produces_valid_positions() {
        let world = empty_world();
        let units: Vec<MilitaryUnitSample> = (0..3)
            .map(|i| MilitaryUnitSample {
                unit_id: i as u64,
                faction_id: 0,
                grid_x: i,
                grid_y: 0,
            })
            .collect();

        let bridge = WarBridge::with_defaults(&world);
        let positions = bridge.formation_move(&units, FormationKind::Column, Facing::North);

        assert_eq!(positions.len(), 3);
        // Column facing North: units file along -Y.  All x should be equal
        // (the centroid x), and y values should be distinct.
        let xs: Vec<i32> = positions.iter().map(|p| p.0).collect();
        let mut ys: Vec<i32> = positions.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        ys.dedup();
        assert_eq!(ys.len(), 3, "distinct y positions");
        assert!(xs.iter().all(|&x| x == xs[0]), "all on same x axis");
    }

    /// An empty squad produces an empty position list.
    #[test]
    fn formation_move_empty_squad_returns_empty() {
        let world = empty_world();
        let bridge = WarBridge::with_defaults(&world);
        let positions = bridge.formation_move(&[], FormationKind::Wedge, Facing::South);
        assert!(positions.is_empty());
    }

    /// Covers FR-CIV-TACTICS-042.
    /// FR-CIV-TACTICS-042 — fog hides distant targets even within engage range.
    #[test]
    fn fog_blocks_engagement_beyond_vision() {
        let world = empty_world();
        let units = [
            MilitaryUnitSample {
                unit_id: 1,
                faction_id: 0,
                grid_x: 0,
                grid_y: 0,
            },
            MilitaryUnitSample {
                unit_id: 2,
                faction_id: 1,
                grid_x: 20,
                grid_y: 0,
            },
        ];
        let config = WarBridgeConfig {
            cadence_ticks: 1,
            engage_range_grid: 24,
            fog_vision_radius: Some(3),
            fog_grid_size: 32,
            ..WarBridgeConfig::default()
        };
        let fog = build_fog_for_units(&config, &units, &world).expect("fog");
        let bridge = WarBridge::new(&world, config);
        let engagements = bridge.resolve_combat(1, &units, Some(&fog));
        assert!(
            engagements.is_empty(),
            "target outside vision radius must not be engaged"
        );
    }

    /// resolve_combat respects the cadence — no engagements on off-cadence ticks.
    #[test]
    fn combat_respects_cadence() {
        let world = empty_world();
        let units = two_enemy_units();
        let config = WarBridgeConfig {
            cadence_ticks: 8,
            engage_range_grid: 16,
            ..WarBridgeConfig::default()
        };
        let bridge = WarBridge::new(&world, config);
        // Off-cadence ticks -> empty.
        assert!(bridge.resolve_combat(1, &units, None).is_empty());
        assert!(bridge.resolve_combat(7, &units, None).is_empty());
        // On-cadence tick -> engagements.
        assert!(!bridge.resolve_combat(8, &units, None).is_empty());
    }
}
