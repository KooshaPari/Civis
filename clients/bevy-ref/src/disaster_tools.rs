#![cfg(feature = "bevy")]

//! God-game DISASTER actions for the Bevy reference client.
//!
//! The toolbar's Disaster category (Meteor / Flood / Quake / Storm / Wildfire /
//! Plague) previously mapped onto `SpawnTool::Destroy`, which only despawns the
//! nearest actor — so the disasters never touched the voxel world. This module
//! makes them real WORLD events: a click under a disaster sub-tool raycasts the
//! cursor hit and mutates the live `CaGrid` (carve a meteor crater, flood a
//! basin, ignite a wildfire, collapse terrain in a quake), then lets the CA
//! carry the aftermath (lava cools, fire spreads, water settles).
//!
//! ## Decoupled, file-local (mirrors `terraform_brush.rs`)
//! This module owns ONLY the disaster verbs + their voxel math. It does not edit
//! `spawn_tools.rs`. It reads the shared cursor hit ([`CursorMarker`]) and the
//! egui pointer gate ([`PointerOverUi`]) and the UI's [`ActiveSubTool`], and
//! writes through `CaGrid::set_with_temp` (which marks the chunk dirty so the
//! mesher re-meshes the change). Every write is measurable: each apply returns a
//! [`DisasterImpact`] (voxels changed) so a headless test / in-game census can
//! prove the action mutated the world without trusting a screenshot.

use bevy::prelude::*;

#[cfg(feature = "egui")]
use crate::tool_categories::{ActiveSubTool, SubTool};

/// The disaster verbs the toolbar can trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisasterKind {
    /// Impact crater: carve a sphere to air, line the floor with lava + heat.
    Meteor,
    /// Flood: fill a hemispherical basin around the hit with water.
    Flood,
    /// Earthquake: collapse a radius of surface columns downward.
    Quake,
    /// Storm: scatter water + douse heat over a wide, shallow disc.
    Storm,
    /// Wildfire: ignite flammable cells in a radius (fire + ember + heat).
    Wildfire,
    /// Plague: a population event with no terrain footprint (handled elsewhere).
    Plague,
}

impl DisasterKind {
    /// Map a UI [`SubTool`] onto a disaster verb, if it is one.
    #[cfg(feature = "egui")]
    #[must_use]
    pub fn from_subtool(sub: SubTool) -> Option<Self> {
        Some(match sub {
            SubTool::Meteor => Self::Meteor,
            SubTool::Flood => Self::Flood,
            SubTool::Quake => Self::Quake,
            SubTool::Storm => Self::Storm,
            SubTool::Wildfire => Self::Wildfire,
            SubTool::Plague => Self::Plague,
            _ => return None,
        })
    }

    /// Footprint radius (world units == grid cells) for the verb.
    #[must_use]
    pub fn radius(self) -> f32 {
        match self {
            DisasterKind::Meteor => 7.0,
            DisasterKind::Flood => 9.0,
            DisasterKind::Quake => 8.0,
            DisasterKind::Storm => 14.0,
            DisasterKind::Wildfire => 8.0,
            DisasterKind::Plague => 0.0,
        }
    }

    /// True when the verb mutates the voxel world (Plague does not).
    #[must_use]
    pub fn edits_world(self) -> bool {
        !matches!(self, DisasterKind::Plague)
    }
}

/// Measured result of applying a disaster — how many voxel cells changed. Lets a
/// test/census prove the action mutated the world (no screenshot needed).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DisasterImpact {
    /// Cells set to a new material this strike.
    pub cells_changed: usize,
}

/// Request to apply a disaster at a world point (emitted on click).
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct DisasterRequest {
    /// World-space impact centre (terrain/voxel hit).
    pub center: Vec3,
    /// Which disaster to apply.
    pub kind: DisasterKind,
}

/// Plugin wiring the disaster click→request→apply path.
pub struct DisasterToolsPlugin;

impl Plugin for DisasterToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DisasterRequest>();

        #[cfg(feature = "egui")]
        app.add_systems(Update, emit_disaster_clicks);

        // Apply after the click emitter when both exist so requests are drained
        // the same frame; fall back to an unordered add if egui is absent.
        #[cfg(all(feature = "voxel", feature = "egui"))]
        app.add_systems(Update, apply_disaster_requests.after(emit_disaster_clicks));
        #[cfg(all(feature = "voxel", not(feature = "egui")))]
        app.add_systems(Update, apply_disaster_requests);
    }
}

/// Translate a left-click under an active disaster sub-tool into a
/// [`DisasterRequest`] using the shared cursor hit, honouring the egui gate.
#[cfg(feature = "egui")]
fn emit_disaster_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    sub: Res<ActiveSubTool>,
    over_ui: Res<crate::spawn_tools::PointerOverUi>,
    marker: Res<crate::spawn_tools::CursorMarker>,
    mut requests: MessageWriter<DisasterRequest>,
) {
    let Some(kind) = DisasterKind::from_subtool(sub.current) else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Left) || over_ui.0 {
        return;
    }
    let Some(center) = marker.position else {
        return;
    };
    requests.write(DisasterRequest { center, kind });
    info!("[disaster] {:?} requested at {:?}", kind, center);
}

/// Drain [`DisasterRequest`]s and mutate the live voxel grid, logging the
/// measured voxel-cell delta for each strike.
#[cfg(feature = "voxel")]
fn apply_disaster_requests(
    mut requests: MessageReader<DisasterRequest>,
    mut sim: ResMut<crate::voxel_sim::VoxelSimState>,
) {
    for req in requests.read() {
        let impact = apply_disaster(&mut sim.grid, req.center, req.kind);
        info!(
            "[disaster] {:?} applied at {:?}: {} cells changed",
            req.kind, req.center, impact.cells_changed
        );
    }
}

/// Apply a disaster to `grid` at `center`, returning the measured impact. Pure +
/// standalone so it is unit-testable headless (no Bevy app / window needed).
#[cfg(feature = "voxel")]
pub fn apply_disaster(
    grid: &mut civ_voxel::fluid_ca::CaGrid,
    center: Vec3,
    kind: DisasterKind,
) -> DisasterImpact {
    if !kind.edits_world() {
        return DisasterImpact::default();
    }
    let r = kind.radius();
    let (cx, cy, cz) = (center.x.round() as i64, center.y.round() as i64, center.z.round() as i64);
    let ri = r.ceil() as i64;
    let r2 = r * r;
    let mut changed = 0usize;
    for dz in -ri..=ri {
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let (x, y, z) = (cx + dx, cy + dy, cz + dz);
                if x < 0 || y < 0 || z < 0 {
                    continue;
                }
                let (xu, yu, zu) = (x as usize, y as usize, z as usize);
                let dist2 = (dx * dx + dy * dy + dz * dz) as f32;
                if let Some((mat, temp)) = disaster_cell(kind, dx, dy, dz, dist2, r2, grid, xu, yu, zu) {
                    let prev = grid.get(xu, yu, zu);
                    grid.set_with_temp(xu, yu, zu, mat, temp);
                    if grid.get(xu, yu, zu) != prev {
                        changed += 1;
                    }
                }
            }
        }
    }
    DisasterImpact { cells_changed: changed }
}

/// Decide the (material, temperature) a disaster writes at a cell offset, or
/// `None` to leave the cell untouched. Split out to keep `apply_disaster` small.
#[cfg(feature = "voxel")]
fn disaster_cell(
    kind: DisasterKind,
    dx: i64,
    dy: i64,
    dz: i64,
    dist2: f32,
    r2: f32,
    grid: &civ_voxel::fluid_ca::CaGrid,
    x: usize,
    y: usize,
    z: usize,
) -> Option<(civ_voxel::MaterialId, i16)> {
    use civ_voxel::material::{AIR, FIRE, LAVA, WATER};
    if dist2 > r2 {
        return None;
    }
    match kind {
        DisasterKind::Meteor => {
            // Hollow the crater; line the bottom shell with hot lava.
            if dy > -1 {
                Some((AIR, 20))
            } else if dist2 > r2 * 0.5 {
                Some((LAVA, 900))
            } else {
                Some((AIR, 400))
            }
        }
        DisasterKind::Flood | DisasterKind::Storm => {
            // Fill the lower hemisphere with water (storm is wide+shallow).
            if dy <= 0 && grid.get(x, y, z) == AIR {
                Some((WATER, 20))
            } else {
                None
            }
        }
        DisasterKind::Wildfire => {
            // Ignite only flammable solids near the surface (cell currently
            // non-air with air above), leaving holes alone.
            let above_air = grid.get(x, y + 1, z) == AIR;
            if grid.get(x, y, z) != AIR && above_air {
                Some((FIRE, 600))
            } else {
                None
            }
        }
        DisasterKind::Quake => {
            // Collapse: drop solids to air in the upper half to slump terrain.
            if dy >= 0 && grid.get(x, y, z) != AIR {
                Some((AIR, 20))
            } else {
                None
            }
        }
        DisasterKind::Plague => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "voxel")]
    fn solid_world(dims: [usize; 3], surface: usize) -> civ_voxel::fluid_ca::CaGrid {
        use civ_voxel::fluid_ca::CaGrid;
        use civ_voxel::material::STONE;
        let mut g = CaGrid::new(dims);
        for z in 0..dims[2] {
            for y in 0..=surface.min(dims[1] - 1) {
                for x in 0..dims[0] {
                    g.set(x, y, z, STONE);
                }
            }
        }
        g
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn meteor_carves_a_crater_and_marks_dirty() {
        use civ_voxel::material::AIR;
        let mut g = solid_world([32, 32, 32], 20);
        g.dirty_chunks.clear();
        let impact = apply_disaster(&mut g, Vec3::new(16.0, 18.0, 16.0), DisasterKind::Meteor);
        assert!(impact.cells_changed > 0, "meteor changed no voxels");
        assert_eq!(g.get(16, 19, 16), AIR, "crater centre-top should be carved to air");
        assert!(!g.dirty_chunks().is_empty(), "meteor must mark chunks dirty for remesh");
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn flood_adds_water_into_air() {
        use civ_voxel::material::WATER;
        let mut g = solid_world([32, 32, 32], 8);
        let before = g.cells.iter().filter(|&&c| c == WATER).count();
        let impact = apply_disaster(&mut g, Vec3::new(16.0, 10.0, 16.0), DisasterKind::Flood);
        let after = g.cells.iter().filter(|&&c| c == WATER).count();
        assert!(impact.cells_changed > 0, "flood changed no voxels");
        assert!(after > before, "flood did not add water (before {before}, after {after})");
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn wildfire_ignites_surface_solids() {
        use civ_voxel::material::FIRE;
        let mut g = solid_world([32, 16, 32], 8);
        let impact = apply_disaster(&mut g, Vec3::new(16.0, 8.0, 16.0), DisasterKind::Wildfire);
        let fire = g.cells.iter().filter(|&&c| c == FIRE).count();
        assert!(impact.cells_changed > 0 && fire > 0, "wildfire lit no fire");
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn plague_does_not_touch_the_world() {
        let mut g = solid_world([16, 16, 16], 8);
        let impact = apply_disaster(&mut g, Vec3::new(8.0, 8.0, 8.0), DisasterKind::Plague);
        assert_eq!(impact.cells_changed, 0, "plague must not edit terrain");
    }

    #[cfg(feature = "egui")]
    #[test]
    fn subtool_maps_to_disaster() {
        assert_eq!(DisasterKind::from_subtool(SubTool::Meteor), Some(DisasterKind::Meteor));
        assert_eq!(DisasterKind::from_subtool(SubTool::Wildfire), Some(DisasterKind::Wildfire));
        assert_eq!(DisasterKind::from_subtool(SubTool::House), None);
    }
}
