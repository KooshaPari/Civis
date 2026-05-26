//! Operational-layer grid movement toward enemies (FR-CIV-TACTICS-031).

use crate::grid_obstacles::grid_cell_blocked;
use crate::pathfinding::bfs_next_step_with_blocked;
use crate::war_bridge::MilitaryUnitSample;
use civ_voxel::{MaterialId, VoxelWorld};

/// Movement cadence for the operational layer.
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
        let blocked = |gx: i32, gy: i32| grid_cell_blocked(world, gx, gy);
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
    if config.cadence_ticks == 0 || tick % config.cadence_ticks != 0 || pulses == 0 {
        return Vec::new();
    }
    let mut all_moves = Vec::new();
    for _ in 0..pulses {
        all_moves.extend(operational_movement_pulse(config, units, world));
    }
    all_moves
}
