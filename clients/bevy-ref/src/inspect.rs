//! Inspect-anything: click any world cell / agent / structure to read its real
//! state, hover for a lightweight tooltip, and a "god-hand" cursor readout.
//!
//! This is WorldBox / Dwarf Fortress / RimWorld's "click to understand the
//! world" loop. A left-click raycasts to the terrain, classifies what was hit
//! (nearest agent / structure within a pick radius, else the bare cell), and
//! fills [`crate::game_ui::SelectedEntityDetails`] — including its
//! [`InspectKind`] — with the entity's live state. Hovering populates a
//! one-line tooltip drawn near the cursor.
//!
//! Requirements:
//! - `FR-CIV-INSPECT-900` — raycast pick → classify → populate inspector.
//! - `FR-CIV-INSPECT-910` — hover tooltip + god-hand cursor readout.

use crate::terrain::{terrain_height, HEIGHT_SCALE, WATER_LEVEL, WORLD_SIZE};

/// What the inspector is currently looking at. Mirrors WorldBox's pick targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectKind {
    /// Nothing selected.
    #[default]
    None,
    /// A bare terrain cell (no agent / structure on it).
    Cell,
    /// A civilian agent.
    Agent,
    /// A building / structure.
    Structure,
    /// A road / traffic tier segment.
    Road,
}

impl InspectKind {
    /// Short human label for the inspector header.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            InspectKind::None => "—",
            InspectKind::Cell => "Cell",
            InspectKind::Agent => "Agent",
            InspectKind::Structure => "Structure",
            InspectKind::Road => "Road",
        }
    }
}

/// Climate / material readout for a terrain cell — the data shown when a bare
/// cell is inspected. Pure function of the procedural terrain, so it matches the
/// elevation / water / material / temperature info-view overlays exactly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellReadout {
    /// World-space X (centred).
    pub world_x: f32,
    /// World-space Z (centred).
    pub world_z: f32,
    /// Surface height (`0.0 ..= HEIGHT_SCALE`).
    pub height: f32,
    /// Whether the cell is at/below the sim water level.
    pub submerged: bool,
    /// Material band label (Water / Sand / Grass / Rock / Snow).
    pub material: &'static str,
    /// Climate temperature proxy in `0.0..=1.0` (lapse rate + latitude).
    pub temperature: f32,
}

impl CellReadout {
    /// Build a readout from a centred world XZ point.
    #[must_use]
    pub fn sample(world_x: f32, world_z: f32) -> Self {
        let height = terrain_height(world_x + WORLD_SIZE * 0.5, world_z + WORLD_SIZE * 0.5);
        let t = (height / HEIGHT_SCALE).clamp(0.0, 1.0);
        let material = if t < 0.18 {
            "Water"
        } else if t < 0.24 {
            "Sand"
        } else if t < 0.48 {
            "Grass"
        } else if t < 0.85 {
            "Rock"
        } else {
            "Snow"
        };
        let lapse = 1.0 - t;
        let lat = 1.0 - (world_z / (WORLD_SIZE * 0.5)).abs();
        let temperature = (lapse * 0.65 + lat * 0.35).clamp(0.0, 1.0);
        Self {
            world_x,
            world_z,
            height,
            submerged: height <= WATER_LEVEL,
            material,
            temperature,
        }
    }

    /// One-line god-hand tooltip string (FR-CIV-INSPECT-910).
    #[must_use]
    pub fn tooltip(&self) -> String {
        let water = if self.submerged { " · underwater" } else { "" };
        format!(
            "({:.0}, {:.0}) · {} · h={:.0} · {}°{}",
            self.world_x,
            self.world_z,
            self.material,
            self.height,
            temperature_band(self.temperature),
            water
        )
    }
}

/// Coarse temperature band label for a `0.0..=1.0` proxy value.
#[must_use]
pub fn temperature_band(temp: f32) -> &'static str {
    match (temp.clamp(0.0, 1.0) * 4.0) as u32 {
        0 => "frigid",
        1 => "cold",
        2 => "mild",
        3 => "warm",
        _ => "hot",
    }
}

#[cfg(feature = "egui")]
pub use plugin::*;

#[cfg(feature = "egui")]
mod plugin {
    use super::*;
    use crate::game_ui::SelectedEntityDetails;

    /// Inspector data the right HUD panel reads — newtype over the shared
    /// `SelectedEntityDetails` (kept local to this module on the integration
    /// branch; the perception worktree had carried it in game_ui).
    #[derive(Resource, Default)]
    pub struct InspectedDetails(pub SelectedEntityDetails);
    use crate::sim_bridge::SimState;
    use crate::spawn_tools::{CursorMarker, SelectEntityRequest};
    use crate::terrain::WORLD_SIZE;
    use bevy::prelude::*;
    use bevy_egui::{egui, EguiContexts};
    use civ_agents::{Civilian, Needs};

    /// Marker for a structure entity the inspector can read (buildings spawned
    /// by the spawn tools). Optional — present only when the building spawner
    /// tags entities; the inspector degrades to "Structure" otherwise.
    #[derive(Component, Debug, Clone, Copy)]
    pub struct InspectableStructure {
        /// Building kind label.
        pub kind: &'static str,
        /// Current occupancy.
        pub occupancy: u32,
    }

    /// Pick radius (world units) for snapping a click to a nearby agent /
    /// structure before falling back to the bare cell.
    pub const PICK_RADIUS: f32 = 3.0;

    /// God-hand hover state — the cell under the cursor, refreshed each frame.
    #[derive(Resource, Debug, Clone, Copy, Default)]
    pub struct HoverReadout {
        /// The cell currently under the cursor, if the cursor is over terrain.
        pub cell: Option<CellReadout>,
    }

    /// Plugin: hover readout, click-to-inspect classification, tooltip + inspector.
    pub struct InspectPlugin;

    impl Plugin for InspectPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<HoverReadout>()
                .init_resource::<InspectedDetails>()
                .add_systems(
                    Update,
                    (update_hover_readout, classify_inspection, draw_hover_tooltip),
                );
        }
    }

    /// Refresh the god-hand hover readout from the shared cursor marker.
    fn update_hover_readout(marker: Res<CursorMarker>, mut hover: ResMut<HoverReadout>) {
        hover.cell = marker
            .position
            .filter(|_| marker.visible)
            .map(|p| CellReadout::sample(p.x, p.z));
    }

    /// On a select click, classify the hit and populate the inspector details
    /// with the real state of the picked agent / structure / cell.
    #[allow(clippy::type_complexity)]
    fn classify_inspection(
        mut requests: MessageReader<SelectEntityRequest>,
        mut details: ResMut<InspectedDetails>,
        sim: Res<SimState>,
        structures: Query<(&GlobalTransform, &InspectableStructure)>,
    ) {
        for request in requests.read() {
            let pos = request.position;
            // Civilians live in the hecs sim world (not Bevy entities), so pick
            // from `SimState` using the same deterministic position mapping the
            // population / needs overlays use.
            if let Some(d) = pick_agent(pos, &sim) {
                details.0 = d;
            } else if let Some(d) = pick_structure(pos, &structures) {
                details.0 = d;
            } else {
                details.0 = cell_details(pos);
            }
        }
    }

    /// Deterministic world-space position for a sim agent (mirrors the overlay
    /// `agent_norm_xy` mapping so inspect + overlay agree on where agents are).
    fn agent_world_pos(id: u64) -> Vec3 {
        let h = id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let nx = ((h >> 11) as f32 / (1u64 << 53) as f32).fract().clamp(0.0, 1.0);
        let nz = ((h >> 5) as f32 / (1u64 << 53) as f32).fract().clamp(0.0, 1.0);
        let wx = nx * WORLD_SIZE - WORLD_SIZE * 0.5;
        let wz = nz * WORLD_SIZE - WORLD_SIZE * 0.5;
        Vec3::new(wx, 0.0, wz)
    }

    fn pick_agent(pos: Vec3, sim: &SimState) -> Option<SelectedEntityDetails> {
        let r2 = PICK_RADIUS * PICK_RADIUS;
        let click_xz = Vec3::new(pos.x, 0.0, pos.z);
        let mut best: Option<(f32, SelectedEntityDetails)> = None;
        let mut world = sim.0.world.query::<(&Civilian, Option<&Needs>)>();
        for (_, (civ, needs)) in world.iter() {
            let agent_pos = agent_world_pos(civ.id);
            let d2 = agent_pos.distance_squared(click_xz);
            if d2 > r2 {
                continue;
            }
            let pressure = needs
                .map(|n| (n.food + n.shelter + n.safety + n.belonging) / 4.0)
                .unwrap_or(0.0);
            let det = SelectedEntityDetails {
                kind: "Civilian".to_string(),
                name: format!("Civilian #{}", civ.id),
                faction: format!("Faction {}", civ.faction),
                health: format!("Needs pressure {:.0}%", pressure * 100.0),
                profession: needs
                    .map(|n| {
                        format!(
                            "food {:.0} · shelter {:.0} · safety {:.0} · social {:.0}",
                            n.food * 100.0,
                            n.shelter * 100.0,
                            n.safety * 100.0,
                            n.belonging * 100.0
                        )
                    })
                    .unwrap_or_else(|| "—".to_string()),
                position: format!("age {} · cluster {}", civ.age, civ.faction),
            };
            if best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
                best = Some((d2, det));
            }
        }
        best.map(|(_, d)| d)
    }

    fn pick_structure(
        pos: Vec3,
        structures: &Query<(&GlobalTransform, &InspectableStructure)>,
    ) -> Option<SelectedEntityDetails> {
        let r2 = PICK_RADIUS * PICK_RADIUS;
        let mut best: Option<(f32, SelectedEntityDetails)> = None;
        for (tf, s) in structures.iter() {
            let d2 = tf.translation().distance_squared(pos);
            if d2 > r2 {
                continue;
            }
            let det = SelectedEntityDetails {
                kind: "Structure".to_string(),
                name: s.kind.to_string(),
                faction: "—".to_string(),
                health: format!("Occupancy {}", s.occupancy),
                profession: "Structure".to_string(),
                position: format!("({:.0}, {:.0})", tf.translation().x, tf.translation().z),
            };
            if best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
                best = Some((d2, det));
            }
        }
        best.map(|(_, d)| d)
    }

    fn cell_details(pos: Vec3) -> SelectedEntityDetails {
        let cell = CellReadout::sample(pos.x, pos.z);
        SelectedEntityDetails {
            kind: "Cell".to_string(),
            name: format!("Cell ({:.0}, {:.0})", cell.world_x, cell.world_z),
            faction: "—".to_string(),
            health: if cell.submerged { "Submerged" } else { "Dry" }.to_string(),
            profession: format!("Material: {}", cell.material),
            position: format!(
                "h={:.0} · {} ({})",
                cell.height,
                temperature_band(cell.temperature),
                cell.material
            ),
        }
    }

    /// Draw the god-hand hover tooltip near the cursor (FR-CIV-INSPECT-910).
    fn draw_hover_tooltip(mut contexts: EguiContexts, hover: Res<HoverReadout>) {
        let Some(cell) = hover.cell else {
            return;
        };
        let Ok(ctx) = contexts.ctx_mut() else {
            return;
        };
        let pointer = ctx.pointer_hover_pos();
        egui::Area::new(egui::Id::new("civis_godhand_tooltip"))
            .order(egui::Order::Tooltip)
            .fixed_pos(
                pointer
                    .map(|p| egui::pos2(p.x + 16.0, p.y + 16.0))
                    .unwrap_or(egui::pos2(20.0, 20.0)),
            )
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(cell.tooltip());
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-INSPECT-900 — a bare cell readout reflects real terrain state.
    #[test]
    fn cell_readout_reflects_terrain() {
        let cell = CellReadout::sample(0.0, 0.0);
        assert!(cell.height >= 0.0 && cell.height <= HEIGHT_SCALE);
        assert!((0.0..=1.0).contains(&cell.temperature));
        assert!(matches!(
            cell.material,
            "Water" | "Sand" | "Grass" | "Rock" | "Snow"
        ));
    }

    /// FR-CIV-INSPECT-900 — submerged flag tracks the water level.
    ///
    /// The procedural island sits mostly at/below `WATER_LEVEL`, with dry land
    /// concentrated near the centre, so scan the full 2D grid (not one edge).
    #[test]
    fn submerged_flag_tracks_water_level() {
        let mut found_wet = false;
        let mut found_dry = false;
        let half = WORLD_SIZE * 0.5;
        let mut z = -half;
        while z < half {
            let mut x = -half;
            while x < half {
                let cell = CellReadout::sample(x, z);
                if cell.submerged {
                    found_wet = true;
                } else {
                    found_dry = true;
                }
                if found_wet && found_dry {
                    return; // both classes proven to exist.
                }
                x += 4.0;
            }
            z += 4.0;
        }
        panic!("expected both submerged and dry cells (wet={found_wet}, dry={found_dry})");
    }

    /// FR-CIV-INSPECT-910 — tooltip is a non-empty one-liner with coords.
    #[test]
    fn tooltip_is_informative() {
        let cell = CellReadout::sample(12.0, -34.0);
        let tip = cell.tooltip();
        assert!(tip.contains("12"));
        assert!(tip.contains(cell.material));
        assert!(!tip.contains('\n'), "tooltip is one line");
    }

    /// Temperature bands cover the full range.
    #[test]
    fn temperature_bands_span_range() {
        assert_eq!(temperature_band(0.0), "frigid");
        assert_eq!(temperature_band(1.0), "hot");
        assert_ne!(temperature_band(0.5), temperature_band(0.0));
    }

    /// InspectKind labels are stable + non-empty.
    #[test]
    fn inspect_kind_labels() {
        assert_eq!(InspectKind::Agent.label(), "Agent");
        assert_eq!(InspectKind::Cell.label(), "Cell");
        assert_eq!(InspectKind::None.label(), "—");
    }
}
