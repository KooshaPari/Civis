//! Per-faction fog-of-war visibility grid (FR-CIV-TACTICS-042).
//!
//! [`FogOfWar`] maintains a per-faction visibility bitmap over a 2D grid.
//! Visibility is recomputed each call to [`FogOfWar::update`] by sweeping
//! every tile within each unit's vision radius and accepting only those for
//! which [`line_of_sight`] returns `true` from the unit's voxel position to
//! the tile's voxel position.
//!
//! ## Coordinate conventions
//! - Grid coordinates are `(i32, i32)` pairs — the same `(grid_x, grid_y)`
//!   used throughout the tactics layer.
//! - The fog grid spans `[0, grid_size)` in both dimensions; queries outside
//!   that range return `false` (not visible).
//! - Voxel conversion uses [`grid_to_world_coord`] from the war bridge, so all
//!   LOS checks are anchored in the same voxel space as combat.

use crate::los::line_of_sight;
use crate::war_bridge::{grid_to_world_coord, MilitaryUnitSample};
use civ_voxel::{MaterialId, VoxelWorld};
use std::collections::HashMap;

/// Default vision radius in grid cells when none is supplied.
const DEFAULT_VISION_RADIUS: u32 = 8;

/// Per-faction visibility state over a fixed-size 2-D grid.
///
/// Visibility is re-derived from scratch on every call to [`Self::update`];
/// there is no incremental state. This keeps the data model simple and
/// deterministic: two identical inputs always produce identical visibility.
pub struct FogOfWar {
    /// Number of grid cells on each axis.
    grid_size: u32,
    /// Vision radius (in grid cells) each unit can see.
    vision_radius: u32,
    /// Packed visibility bits: `visible[faction_id][cell_index]`.
    ///
    /// `cell_index = y * grid_size + x` for grid position `(x, y)`.
    visible: HashMap<u32, Vec<bool>>,
}

impl FogOfWar {
    /// Create a new fog-of-war instance for a square grid of `grid_size` cells.
    ///
    /// `vision_radius` is the maximum number of grid cells any unit can see.
    /// Pass `None` to use the default of [`DEFAULT_VISION_RADIUS`].
    pub fn new(grid_size: u32, vision_radius: Option<u32>) -> Self {
        Self {
            grid_size,
            vision_radius: vision_radius.unwrap_or(DEFAULT_VISION_RADIUS),
            visible: HashMap::new(),
        }
    }

    /// Recompute visibility for all factions from the current unit positions.
    ///
    /// For each unit the algorithm iterates every grid cell within
    /// `vision_radius` (Chebyshev square to bound the search), then accepts
    /// the cell if the Euclidean distance is within the radius **and**
    /// [`line_of_sight`] is unblocked from the unit's voxel position to the
    /// tile's voxel position.
    pub fn update(
        &mut self,
        faction_units: &[MilitaryUnitSample],
        voxel_world: &VoxelWorld<MaterialId>,
    ) {
        self.visible.clear();

        // Pre-collect distinct factions so we can initialise their bitmaps.
        let factions: Vec<u32> = {
            let mut f: Vec<u32> = faction_units.iter().map(|u| u.faction_id).collect();
            f.sort_unstable();
            f.dedup();
            f
        };

        let cell_count = (self.grid_size as usize) * (self.grid_size as usize);

        for &faction_id in &factions {
            let bitmap = self
                .visible
                .entry(faction_id)
                .or_insert_with(|| vec![false; cell_count]);

            for unit in faction_units.iter().filter(|u| u.faction_id == faction_id) {
                let ux = unit.grid_x;
                let uy = unit.grid_y;
                let r = self.vision_radius as i32;

                // Bounding Chebyshev square clamped to the grid.
                let x_min = (ux - r).max(0) as u32;
                let x_max = (ux + r).min(self.grid_size as i32 - 1).max(-1) as u32;
                let y_min = (uy - r).max(0) as u32;
                let y_max = (uy + r).min(self.grid_size as i32 - 1).max(-1) as u32;

                if ux < 0 || uy < 0 || ux >= self.grid_size as i32 || uy >= self.grid_size as i32 {
                    // Unit is off-grid — still allow it to reveal tiles.
                    // We just won't mark it visible itself; iterate the box anyway.
                }

                let unit_wc = grid_to_world_coord(ux, uy);

                for ty in y_min..=y_max {
                    for tx in x_min..=x_max {
                        // Euclidean range check.
                        let ddx = (tx as i32 - ux) as f64;
                        let ddy = (ty as i32 - uy) as f64;
                        if ddx * ddx + ddy * ddy > (r as f64) * (r as f64) {
                            continue;
                        }

                        let tile_wc = grid_to_world_coord(tx as i32, ty as i32);

                        if line_of_sight(voxel_world, unit_wc, tile_wc) {
                            let idx = ty as usize * self.grid_size as usize + tx as usize;
                            bitmap[idx] = true;
                        }
                    }
                }
            }
        }
    }

    /// Returns `true` when `position` is currently visible to `faction`.
    ///
    /// Returns `false` for any faction or position that has not been computed
    /// yet (i.e. before the first call to [`Self::update`]).
    pub fn is_visible(&self, faction: u32, position: (i32, i32)) -> bool {
        let (x, y) = position;
        if x < 0 || y < 0 || x >= self.grid_size as i32 || y >= self.grid_size as i32 {
            return false;
        }
        let idx = y as usize * self.grid_size as usize + x as usize;
        self.visible
            .get(&faction)
            .and_then(|bm| bm.get(idx))
            .copied()
            .unwrap_or(false)
    }

    /// Grid size this fog was constructed with.
    pub fn grid_size(&self) -> u32 {
        self.grid_size
    }

    /// Vision radius this fog was constructed with.
    pub fn vision_radius(&self) -> u32 {
        self.vision_radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::WorldCoord;

    fn empty_world() -> VoxelWorld<MaterialId> {
        VoxelWorld::new(1)
    }

    fn unit(unit_id: u64, faction_id: u32, grid_x: i32, grid_y: i32) -> MilitaryUnitSample {
        MilitaryUnitSample {
            unit_id,
            faction_id,
            grid_x,
            grid_y,
        }
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-040 basic reveal
    // -------------------------------------------------------------------------

    /// A unit at (0,0) reveals the tile at its own position.
    #[test]
    fn unit_reveals_own_tile() {
        let mut fog = FogOfWar::new(16, None);
        let units = [unit(1, 0, 0, 0)];
        fog.update(&units, &empty_world());
        assert!(fog.is_visible(0, (0, 0)));
    }

    /// A unit reveals nearby tiles within its vision radius.
    #[test]
    fn unit_reveals_nearby_tiles() {
        let mut fog = FogOfWar::new(32, Some(5));
        let units = [unit(1, 0, 10, 10)];
        fog.update(&units, &empty_world());

        // Tiles within radius 5 should be visible.
        assert!(fog.is_visible(0, (10, 10)));
        assert!(fog.is_visible(0, (12, 10)));
        assert!(fog.is_visible(0, (10, 14)));
    }

    /// Tiles beyond the vision radius remain in fog.
    #[test]
    fn tiles_beyond_radius_stay_hidden() {
        let mut fog = FogOfWar::new(32, Some(4));
        let units = [unit(1, 0, 10, 10)];
        fog.update(&units, &empty_world());

        // Tile at distance 6 — outside radius 4.
        assert!(!fog.is_visible(0, (10, 16)));
        assert!(!fog.is_visible(0, (16, 10)));
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-040 faction isolation
    // -------------------------------------------------------------------------

    /// Faction 0 cannot see what faction 1 sees.
    #[test]
    fn factions_have_independent_visibility() {
        let mut fog = FogOfWar::new(32, Some(4));
        let units = [
            unit(1, 0, 2, 2),   // faction 0 near corner
            unit(2, 1, 25, 25), // faction 1 far corner
        ];
        fog.update(&units, &empty_world());

        // Faction 0 can see near (2,2).
        assert!(fog.is_visible(0, (2, 2)));
        // Faction 0 cannot see (25,25) — only faction 1 unit is there.
        assert!(!fog.is_visible(0, (25, 25)));
        // Faction 1 can see (25,25).
        assert!(fog.is_visible(1, (25, 25)));
        // Faction 1 cannot see (2,2).
        assert!(!fog.is_visible(1, (2, 2)));
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-040 wall blocking
    // -------------------------------------------------------------------------

    /// A solid wall placed between a unit and a tile blocks visibility.
    #[test]
    fn wall_blocks_vision() {
        let mut world = empty_world();
        // Place a solid wall column between grid_x=2 and grid_x=4.
        // grid_to_world_coord maps grid (3,0) to some voxel; fill the entire
        // Y column there to block any ray passing through x=3.
        let wall_wc = grid_to_world_coord(3, 0);
        for dy in -5_i64..=5 {
            world.write(
                WorldCoord {
                    x: wall_wc.x,
                    y: dy,
                    z: wall_wc.z,
                },
                civ_voxel::MaterialId(1),
            );
        }

        let mut fog = FogOfWar::new(16, Some(6));
        let units = [unit(1, 0, 2, 0)];
        fog.update(&units, &world);

        // Tile at (4,0) is behind the wall from the unit at (2,0).
        assert!(!fog.is_visible(0, (4, 0)));

        // Tile at (1,0) is in front of the wall — should be visible.
        assert!(fog.is_visible(0, (1, 0)));
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-040 dynamic update
    // -------------------------------------------------------------------------

    /// Moving a unit updates the fog correctly.
    #[test]
    fn moving_unit_updates_fog() {
        let mut fog = FogOfWar::new(32, Some(3));
        let world = empty_world();

        // Unit starts at (2,2).
        let mut units = [unit(1, 0, 2, 2)];
        fog.update(&units, &world);
        assert!(fog.is_visible(0, (2, 2)));
        // Tile far from original position not visible.
        assert!(!fog.is_visible(0, (20, 20)));

        // Move unit to (20,20).
        units[0].grid_x = 20;
        units[0].grid_y = 20;
        fog.update(&units, &world);
        // New position is visible.
        assert!(fog.is_visible(0, (20, 20)));
        // Original position is now hidden (fog resets each update).
        assert!(!fog.is_visible(0, (2, 2)));
    }

    // -------------------------------------------------------------------------
    // FR-CIV-TACTICS-040 out-of-bounds safety
    // -------------------------------------------------------------------------

    /// Queries outside grid bounds always return false.
    #[test]
    fn out_of_bounds_query_returns_false() {
        let mut fog = FogOfWar::new(16, None);
        let units = [unit(1, 0, 0, 0)];
        fog.update(&units, &empty_world());

        assert!(!fog.is_visible(0, (-1, 0)));
        assert!(!fog.is_visible(0, (0, -1)));
        assert!(!fog.is_visible(0, (16, 0)));
        assert!(!fog.is_visible(0, (0, 16)));
    }

    /// Unknown faction always returns false.
    #[test]
    fn unknown_faction_returns_false() {
        let mut fog = FogOfWar::new(16, None);
        let units = [unit(1, 0, 5, 5)];
        fog.update(&units, &empty_world());
        // Faction 99 was never part of the update.
        assert!(!fog.is_visible(99, (5, 5)));
    }

    /// Multiple units from the same faction union their visible sets.
    #[test]
    fn multiple_units_union_visibility() {
        let mut fog = FogOfWar::new(32, Some(3));
        let world = empty_world();
        let units = [
            unit(1, 0, 2, 2),   // reveals area around (2,2)
            unit(2, 0, 20, 20), // reveals area around (20,20)
        ];
        fog.update(&units, &world);
        assert!(fog.is_visible(0, (2, 2)));
        assert!(fog.is_visible(0, (20, 20)));
    }
}
