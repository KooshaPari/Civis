//! Operational-layer grid movement toward enemies (FR-CIV-TACTICS-031).

use crate::war_bridge::MilitaryUnitSample;

/// Movement cadence for the operational layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalMovementConfig {
    /// Apply movement when `tick % cadence_ticks == 0`.
    pub cadence_ticks: u64,
}

impl Default for OperationalMovementConfig {
    fn default() -> Self {
        Self { cadence_ticks: 8 }
    }
}

/// Grid position update for a unit index in the operational slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridMove {
    /// Index into the `MilitaryUnitSample` slice passed to [`tick_operational_movement`].
    pub unit_index: usize,
    pub new_grid_x: i32,
    pub new_grid_y: i32,
}

fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// Deterministic step toward the nearest enemy unit on the grid plane.
pub fn tick_operational_movement(
    tick: u64,
    config: &OperationalMovementConfig,
    units: &[MilitaryUnitSample],
) -> Vec<GridMove> {
    if config.cadence_ticks == 0 || tick % config.cadence_ticks != 0 {
        return Vec::new();
    }
    let mut moves = Vec::new();
    for (i, unit) in units.iter().enumerate() {
        let mut best: Option<(usize, i32)> = None;
        for (j, other) in units.iter().enumerate() {
            if i == j || unit.faction_id == other.faction_id {
                continue;
            }
            let dist = manhattan((unit.grid_x, unit.grid_y), (other.grid_x, other.grid_y));
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
        let enemy = &units[enemy_idx];
        let dx = (enemy.grid_x - unit.grid_x).clamp(-1, 1);
        let dy = (enemy.grid_y - unit.grid_y).clamp(-1, 1);
        if dx == 0 && dy == 0 {
            continue;
        }
        moves.push(GridMove {
            unit_index: i,
            new_grid_x: unit.grid_x + dx,
            new_grid_y: unit.grid_y + dy,
        });
    }
    moves
}
