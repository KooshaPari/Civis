//! Phase-4 war bridge: military grid positions → voxel damage events (FR-CIV-TACTICS-022).

use crate::los::line_of_sight;
use crate::DamageEvent;
use civ_voxel::{MaterialId, WorldCoord, VoxelWorld, FIXED_SCALE};

/// Minimal military unit sample for the war bridge (grid plane).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MilitaryUnitSample {
    /// Owning faction id.
    pub faction_id: u32,
    /// Grid X (hex plane).
    pub grid_x: i32,
    /// Grid Y (hex plane).
    pub grid_y: i32,
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
}

impl Default for WarBridgeConfig {
    fn default() -> Self {
        Self {
            cadence_ticks: 32,
            engage_range_grid: 8,
            damage_radius_voxels: 2,
            damage_energy: 250,
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

/// Resolve cross-faction engagements and return voxel [`DamageEvent`]s to queue on the sim.
///
/// Deterministic: nested loops over unit indices in input order; first eligible shooter
/// per target wins.
pub fn tick_war_bridge(
    tick: u64,
    config: &WarBridgeConfig,
    units: &[MilitaryUnitSample],
    world: &VoxelWorld<MaterialId>,
) -> Vec<DamageEvent> {
    if config.cadence_ticks == 0 || tick % config.cadence_ticks != 0 {
        return Vec::new();
    }
    let range = config.engage_range_grid.max(1);
    let mut events = Vec::new();
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
            match best {
                None => best = Some((j, dist)),
                Some((_, best_dist)) if dist < best_dist => best = Some((j, dist)),
                _ => {}
            }
        }
        if let Some((target_idx, _)) = best {
            let target = &units[target_idx];
            damaged_targets.push(target_idx);
            events.push(DamageEvent {
                center: grid_to_world_coord(target.grid_x, target.grid_y),
                radius_voxels: config.damage_radius_voxels,
                energy: config.damage_energy,
            });
        }
    }

    events
}
