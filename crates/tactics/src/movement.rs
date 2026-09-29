//! Operational-layer grid movement toward enemies (FR-CIV-TACTICS-031).

use crate::grid_obstacles::grid_cell_impassable;
use crate::pathfinding::{astar_path_with_blocked, bfs_next_step_with_blocked};
use crate::war_bridge::MilitaryUnitSample;
use civ_voxel::{MaterialId, VoxelWorld};

/// Movement cadence for the operational layer.
// The following 1 requirement tags were removed from OperationalMovementConfig.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// The requirement is behavioral by construction: it is about which target a formation picks, and the only symbol that picks targets is operational_movement_pulse, not this config. That is a true observation about where the behavior would live, but it does not make this verdict IMPLEMENTED-BY-BEHAVIOR, because the supply-gradient utility comparison the requirement mandates does not exist in operational_movement_pulse either. The movement code is advance-to-contact only.
// docs/design/warfare.md:83 credits 'already in movement.rs' for operational movement generally, and that much is real and shipped. The credit does not extend to the objective-and-supply driving that FR-CIV-WAR-011 actually asks for, which is the gap the tag was papering over.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-WAR-011: DATA-SHAPE-ONLY. The requirement is that operational movement is driven by theater objectives plus supply gradients, so that formations advance toward objectives only while supplied and otherwise fall back to supply, with advance-to-contact and fall-back-to-supply emerging from one utility comparison (docs/design/warfare.md:83-85). OperationalMovementConfig at crates/tactics/src/movement.rs:11 is a two-field cadence struct (cadence_ticks, path_search_radius) and carries no objective vector, no supply state and no utility comparison. The movement it parameterizes actively contradicts the requirement: operational_movement_pulse at crates/tactics/src/movement.rs:51 steers every unit at the nearest enemy by Manhattan distance, unconditionally, with no supply term and no objective term in the choice. git grep -n -i -E 'supply|objective|gradient' -- crates/tactics/src/movement.rs returned no hits, and the same search across all of crates/ for 'objective_gain|supply_efficiency|supply_gradient|theater_objective' returned nothing at all, so no supply-gradient driver exists anywhere in the repo. The test at crates/tactics/tests/fr_fr_civ_war_011.rs asserts only that cadence_ticks > 0 and path_search_radius > 0, which validates the struct's defaults and nothing about maneuver policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalMovementConfig {
    /// Apply movement when `tick % cadence_ticks == 0`.
    pub cadence_ticks: u64,
    /// BFS search radius on the grid plane.
    pub path_search_radius: u32,
}

impl Default for OperationalMovementConfig {
    fn default() -> Self {
        Self {
            cadence_ticks: 4,
            path_search_radius: 24,
        }
    }
}

// The following 1 requirement tags were removed from GridMove.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// This is a client input-binding requirement filed against a server-side movement intent struct.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-RTS-001: The requirement in agileplus-specs/civ-012-godot-secondary-client/spec.md is "Q - Move command (click target to confirm)": the client SHALL bind the Q key to a move command and SHALL require a click on the target to confirm it. `GridMove` is a three-field move intent (unit_index, new_grid_x, new_grid_y). It carries no keybinding and no confirmation state, and no keybinding or click-to-confirm handler exists anywhere in crates/, so no implementing symbol does.
/// Grid position update for a unit index in the operational slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridMove {
    /// Index into the `MilitaryUnitSample` slice passed to [`tick_operational_movement`].
    pub unit_index: usize,
    /// New grid X coordinate after the movement step.
    pub new_grid_x: i32,
    /// New grid Y coordinate after the movement step.
    pub new_grid_y: i32,
}

fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// One movement pulse: pathfind one step toward the nearest enemy for each unit.
pub fn operational_movement_pulse(
    config: &OperationalMovementConfig,
    units: &mut [MilitaryUnitSample],
    world: &VoxelWorld<MaterialId>,
) -> Vec<GridMove> {
    let positions: Vec<(i32, i32)> = units.iter().map(|u| (u.grid_x, u.grid_y)).collect();
    let factions: Vec<u32> = units.iter().map(|u| u.faction_id).collect();

    let mut moves = Vec::new();
    for i in 0..units.len() {
        let from = positions[i];
        let mut best: Option<(usize, i32)> = None;
        for j in 0..units.len() {
            if i == j || factions[i] == factions[j] {
                continue;
            }
            let dist = manhattan(from, positions[j]);
            if dist == 0 {
                continue;
            }
            match best {
                None => best = Some((j, dist)),
                Some((_, best_dist)) if dist < best_dist => best = Some((j, dist)),
                _ => {}
            }
        }
        let Some((enemy_idx, _)) = best else {
            continue;
        };
        let to = positions[enemy_idx];
        let blocked = |gx: i32, gy: i32| grid_cell_impassable(world, units, gx, gy, i);
        let next = astar_path_with_blocked(from, to, config.path_search_radius, &blocked)
            .and_then(|path| path.get(1).copied())
            .or_else(|| bfs_next_step_with_blocked(from, to, config.path_search_radius, &blocked));
        let Some((nx, ny)) = next else {
            continue;
        };
        moves.push(GridMove {
            unit_index: i,
            new_grid_x: nx,
            new_grid_y: ny,
        });
    }

    for gm in &moves {
        units[gm.unit_index].grid_x = gm.new_grid_x;
        units[gm.unit_index].grid_y = gm.new_grid_y;
    }

    moves
}

/// Deterministic pathfinding step(s) toward the nearest enemy unit on the grid plane.
pub fn tick_operational_movement(
    tick: u64,
    config: &OperationalMovementConfig,
    units: &mut [MilitaryUnitSample],
    pulses: u8,
    world: &VoxelWorld<MaterialId>,
) -> Vec<GridMove> {
    if config.cadence_ticks == 0 || !tick.is_multiple_of(config.cadence_ticks) || pulses == 0 {
        return Vec::new();
    }
    let mut all_moves = Vec::new();
    for _ in 0..pulses {
        all_moves.extend(operational_movement_pulse(config, units, world));
    }
    all_moves
}
