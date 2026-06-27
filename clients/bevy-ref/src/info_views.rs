//! Data-driven info-view overlay suite for the Civis Bevy reference client.
//!
//! Cities-Skylines-2-style overlays: a registry of named overlays, each mapping
//! a world sample point (and optional sim state) to a colour on a legend scale.
//! A toggle panel (UI buttons in the left HUD) switches the *active* terrain
//! overlay; the legend shows the colour ramp. [`NearbyCountsOverlay`] (hotkey
//! `Tab`) shows live nearby entity counts. New overlays are a [`InfoOverlay`]
//! registration,
//! not a code fork.
//!
//! Overlays here are sourced from data Civis already computes in the standalone
//! sandbox: procedural terrain (`terrain.rs`) for elevation / water / climate /
//! material bands, and the live `SimState` agent world for population density,
//! needs-pressure, and cluster / territory. Each overlay is rendered as a recolor
//! gizmo grid sampled over the terrain surface — a lightweight stand-in for a
//! full recolor pass that keeps the standalone build GPU-cheap and deterministic.
//!
//! Requirements:
//! - `FR-CIV-INFOVIEW-900` — overlay registry + active-overlay toggle.
//! - `FR-CIV-INFOVIEW-901` — legend / colour-scale presentation.
//! - `FR-CIV-INFOVIEW-910` — high-value overlays over already-computed data.

#[cfg(feature = "egui")]
use bevy::prelude::*;

use crate::terrain::{terrain_height, HEIGHT_SCALE, WATER_LEVEL, WORLD_SIZE};
use civ_agents::Civilian;

fn civilian_faction_id(civilian: &Civilian) -> u32 {
    match civilian.alignment {
        civ_agents::Alignment::Faction(faction) => faction,
        _ => 0,
    }
}

/// A single colour-ramp stop: a value in `0.0..=1.0` mapped to an sRGB triple.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegendStop {
    /// Normalised position on the ramp (`0.0` = low end, `1.0` = high end).
    pub at: f32,
    /// Human label shown beside the swatch (e.g. `"Sea level"`).
    pub label: &'static str,
    /// sRGB colour for this stop.
    pub rgb: [f32; 3],
}

/// Sample context handed to an overlay's colour function for one world cell.
///
/// Carries the raw world-space XZ position plus pre-derived terrain fields so
/// individual overlays don't each re-sample the noise field. Sim-derived scalars
/// (population density, needs pressure, cluster id) are filled per-cell by the
/// renderer before the overlay colour function runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlaySample {
    /// World-space X (centred, `-WORLD_SIZE/2 ..= WORLD_SIZE/2`).
    pub world_x: f32,
    /// World-space Z (centred).
    pub world_z: f32,
    /// Terrain surface height at this cell (`0.0 ..= HEIGHT_SCALE`).
    pub height: f32,
    /// Normalised height (`height / HEIGHT_SCALE`).
    pub height_norm: f32,
    /// Population density at this cell, normalised `0.0..=1.0` (agents nearby).
    pub population_density: f32,
    /// Aggregate needs-pressure at this cell, normalised `0.0..=1.0`.
    pub needs_pressure: f32,
    /// Dominant faction / cluster id at this cell, if any agents are present.
    pub cluster: Option<u32>,
}

/// Result of an overlay colour function: an RGBA tint, or `None` to skip (no
/// data at this cell — the underlying terrain shows through).
pub type OverlayColor = Option<[f32; 4]>;

/// A registered overlay: identity, legend, and a pure colour function.
///
/// The colour function is the *only* thing that differs between overlays, which
/// is what makes the suite data-driven: adding "soil fertility" is a new
/// [`InfoOverlay`] value, not a new render system.
#[derive(Clone)]
pub struct InfoOverlay {
    /// Stable machine id (e.g. `"elevation"`).
    pub id: &'static str,
    /// Display name for the toggle button / legend header.
    pub name: &'static str,
    /// One-line description shown as a tooltip on the toggle button.
    pub description: &'static str,
    /// Legend ramp stops (ordered low → high).
    pub legend: &'static [LegendStop],
    /// Whether this overlay reads live sim data (vs. pure terrain).
    pub uses_sim: bool,
    /// Pure colour function: sample → tint.
    pub color_fn: fn(&OverlaySample) -> OverlayColor,
}

impl core::fmt::Debug for InfoOverlay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InfoOverlay")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("uses_sim", &self.uses_sim)
            .finish()
    }
}

/// Linear interpolation across a sorted legend ramp for a normalised value.
#[must_use]
pub fn ramp_color(legend: &[LegendStop], value: f32) -> [f32; 3] {
    if legend.is_empty() {
        return [0.5, 0.5, 0.5];
    }
    let v = value.clamp(0.0, 1.0);
    if v <= legend[0].at {
        return legend[0].rgb;
    }
    if let Some(last) = legend.last() {
        if v >= last.at {
            return last.rgb;
        }
    }
    // Snap to an exact stop so on-stop values are precise (no float drift).
    for stop in legend {
        if (v - stop.at).abs() <= f32::EPSILON {
            return stop.rgb;
        }
    }
    for pair in legend.windows(2) {
        let (lo, hi) = (pair[0], pair[1]);
        if v >= lo.at && v <= hi.at {
            let span = (hi.at - lo.at).max(f32::EPSILON);
            let t = (v - lo.at) / span;
            return [
                lo.rgb[0] + (hi.rgb[0] - lo.rgb[0]) * t,
                lo.rgb[1] + (hi.rgb[1] - lo.rgb[1]) * t,
                lo.rgb[2] + (hi.rgb[2] - lo.rgb[2]) * t,
            ];
        }
    }
    legend[legend.len() - 1].rgb
}

// ---------------------------------------------------------------------------
// Legend ramps (static — shared by colour functions + the legend panel).
// ---------------------------------------------------------------------------

const RAMP_ELEVATION: &[LegendStop] = &[
    LegendStop {
        at: 0.0,
        label: "Deep",
        rgb: [0.10, 0.20, 0.55],
    },
    LegendStop {
        at: 0.18,
        label: "Coast",
        rgb: [0.86, 0.78, 0.52],
    },
    LegendStop {
        at: 0.48,
        label: "Lowland",
        rgb: [0.28, 0.58, 0.24],
    },
    LegendStop {
        at: 0.7,
        label: "Highland",
        rgb: [0.50, 0.45, 0.40],
    },
    LegendStop {
        at: 1.0,
        label: "Peak",
        rgb: [0.97, 0.97, 0.97],
    },
];

const RAMP_WATER: &[LegendStop] = &[
    LegendStop {
        at: 0.0,
        label: "Dry",
        rgb: [0.80, 0.72, 0.50],
    },
    LegendStop {
        at: 1.0,
        label: "Submerged",
        rgb: [0.10, 0.35, 0.75],
    },
];

const RAMP_TEMPERATURE: &[LegendStop] = &[
    LegendStop {
        at: 0.0,
        label: "Cold",
        rgb: [0.20, 0.40, 0.95],
    },
    LegendStop {
        at: 0.5,
        label: "Temperate",
        rgb: [0.30, 0.85, 0.40],
    },
    LegendStop {
        at: 1.0,
        label: "Hot",
        rgb: [0.95, 0.25, 0.15],
    },
];

const RAMP_MATERIAL: &[LegendStop] = &[
    LegendStop {
        at: 0.0,
        label: "Water",
        rgb: [0.20, 0.40, 0.86],
    },
    LegendStop {
        at: 0.25,
        label: "Sand",
        rgb: [0.86, 0.78, 0.52],
    },
    LegendStop {
        at: 0.5,
        label: "Grass",
        rgb: [0.28, 0.58, 0.24],
    },
    LegendStop {
        at: 0.75,
        label: "Rock",
        rgb: [0.50, 0.50, 0.52],
    },
    LegendStop {
        at: 1.0,
        label: "Snow",
        rgb: [0.97, 0.97, 0.97],
    },
];

const RAMP_DENSITY: &[LegendStop] = &[
    LegendStop {
        at: 0.0,
        label: "Empty",
        rgb: [0.15, 0.15, 0.20],
    },
    LegendStop {
        at: 0.5,
        label: "Settled",
        rgb: [0.95, 0.80, 0.20],
    },
    LegendStop {
        at: 1.0,
        label: "Crowded",
        rgb: [0.90, 0.10, 0.10],
    },
];

const RAMP_NEEDS: &[LegendStop] = &[
    LegendStop {
        at: 0.0,
        label: "Content",
        rgb: [0.20, 0.80, 0.30],
    },
    LegendStop {
        at: 0.5,
        label: "Strained",
        rgb: [0.95, 0.85, 0.20],
    },
    LegendStop {
        at: 1.0,
        label: "Critical",
        rgb: [0.90, 0.10, 0.10],
    },
];

// ---------------------------------------------------------------------------
// Colour functions — one per overlay (pure; FR-CIV-INFOVIEW-910).
// ---------------------------------------------------------------------------

fn color_elevation(s: &OverlaySample) -> OverlayColor {
    Some(rgba(ramp_color(RAMP_ELEVATION, s.height_norm), 0.55))
}

fn color_water(s: &OverlaySample) -> OverlayColor {
    // Hydrology: highlight cells at/below the sim water level.
    let submerged = if s.height <= WATER_LEVEL { 1.0 } else { 0.0 };
    if submerged == 0.0 && s.height_norm > 0.30 {
        return None; // dry uplands: let terrain show through.
    }
    Some(rgba(ramp_color(RAMP_WATER, submerged), 0.5))
}

fn color_temperature(s: &OverlaySample) -> OverlayColor {
    // Climate proxy from already-computed fields: hotter at low elevation /
    // toward the equator band (centre Z), colder on peaks — a lapse-rate model
    // that mirrors civ-planet weather without a live server.
    let lapse = 1.0 - s.height_norm; // high ground is colder.
    let lat = 1.0 - (s.world_z / (WORLD_SIZE * 0.5)).abs(); // equator warm.
    let temp = (lapse * 0.65 + lat * 0.35).clamp(0.0, 1.0);
    Some(rgba(ramp_color(RAMP_TEMPERATURE, temp), 0.5))
}

fn color_material(s: &OverlaySample) -> OverlayColor {
    // Material type by terrain band — matches `terrain::color_for_height`.
    let t = s.height_norm;
    let slot = if t < 0.18 {
        0.0
    } else if t < 0.24 {
        0.25
    } else if t < 0.48 {
        0.5
    } else if t < 0.85 {
        0.75
    } else {
        1.0
    };
    Some(rgba(ramp_color(RAMP_MATERIAL, slot), 0.55))
}

fn color_population(s: &OverlaySample) -> OverlayColor {
    if s.population_density <= 0.0 {
        return None;
    }
    Some(rgba(ramp_color(RAMP_DENSITY, s.population_density), 0.6))
}

fn color_needs(s: &OverlaySample) -> OverlayColor {
    if s.population_density <= 0.0 {
        return None; // needs-pressure only meaningful where agents live.
    }
    Some(rgba(ramp_color(RAMP_NEEDS, s.needs_pressure), 0.6))
}

fn color_cluster(s: &OverlaySample) -> OverlayColor {
    let cluster = s.cluster?;
    Some(rgba(cluster_color(cluster), 0.6))
}

fn rgba(rgb: [f32; 3], a: f32) -> [f32; 4] {
    [rgb[0], rgb[1], rgb[2], a]
}

/// Deterministic distinct colour for a cluster / faction id (hash → hue).
#[must_use]
pub fn cluster_color(cluster: u32) -> [f32; 3] {
    let hash = (u64::from(cluster))
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .rotate_left(13);
    let hue = (hash as f32 / u64::MAX as f32).fract();
    hsv_to_rgb(hue, 0.65, 0.9)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h = h.fract().max(0.0);
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i.rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// Sample terrain-derived fields for an overlay at a centred world XZ point.
///
/// Sim-derived fields (`population_density`, `needs_pressure`, `cluster`) start
/// at zero / `None`; the renderer fills them from `SimState` per cell.
#[must_use]
pub fn terrain_sample(world_x: f32, world_z: f32) -> OverlaySample {
    let height = terrain_height(world_x + WORLD_SIZE * 0.5, world_z + WORLD_SIZE * 0.5);
    OverlaySample {
        world_x,
        world_z,
        height,
        height_norm: (height / HEIGHT_SCALE).clamp(0.0, 1.0),
        population_density: 0.0,
        needs_pressure: 0.0,
        cluster: None,
    }
}

// ---------------------------------------------------------------------------
// Registry (FR-CIV-INFOVIEW-900).
// ---------------------------------------------------------------------------

/// Build the default overlay registry: the high-value overlays shipped first,
/// ordered for the toggle panel. New overlays append here.
#[must_use]
pub fn default_overlays() -> Vec<InfoOverlay> {
    vec![
        InfoOverlay {
            id: "elevation",
            name: "Elevation",
            description: "Terrain height above sea level",
            legend: RAMP_ELEVATION,
            uses_sim: false,
            color_fn: color_elevation,
        },
        InfoOverlay {
            id: "water",
            name: "Water",
            description: "Hydrology — cells at/below the sim water level",
            legend: RAMP_WATER,
            uses_sim: false,
            color_fn: color_water,
        },
        InfoOverlay {
            id: "temperature",
            name: "Temperature",
            description: "Climate proxy — lapse rate + latitude band",
            legend: RAMP_TEMPERATURE,
            uses_sim: false,
            color_fn: color_temperature,
        },
        InfoOverlay {
            id: "material",
            name: "Material",
            description: "Surface material type by terrain band",
            legend: RAMP_MATERIAL,
            uses_sim: false,
            color_fn: color_material,
        },
        InfoOverlay {
            id: "population",
            name: "Population",
            description: "Agent density from the live simulation",
            legend: RAMP_DENSITY,
            uses_sim: true,
            color_fn: color_population,
        },
        InfoOverlay {
            id: "needs",
            name: "Needs Pressure",
            description: "Aggregate civilian needs (food/shelter/safety/belonging)",
            legend: RAMP_NEEDS,
            uses_sim: true,
            color_fn: color_needs,
        },
        InfoOverlay {
            id: "cluster",
            name: "Territory",
            description: "Dominant faction / cluster per cell",
            legend: &[],
            uses_sim: true,
            color_fn: color_cluster,
        },
    ]
}

/// Registry resource holding overlays + the active selection + enabled flag.
#[cfg_attr(feature = "egui", derive(Resource))]
#[derive(Debug, Clone)]
pub struct InfoViewRegistry {
    /// All registered overlays.
    pub overlays: Vec<InfoOverlay>,
    /// Index of the active overlay, or `None` when the suite is off.
    pub active: Option<usize>,
    /// Number of gizmo grid cells per axis when rendering (resolution).
    pub grid_resolution: usize,
}

impl Default for InfoViewRegistry {
    fn default() -> Self {
        Self {
            overlays: default_overlays(),
            active: None,
            grid_resolution: 48,
        }
    }
}

impl InfoViewRegistry {
    /// The active overlay, if one is selected.
    #[must_use]
    pub fn active_overlay(&self) -> Option<&InfoOverlay> {
        self.active.and_then(|i| self.overlays.get(i))
    }

    /// Index of the active overlay in the registry, if one is selected.
    #[must_use]
    pub fn active_index(&self) -> Option<usize> {
        self.active.filter(|&i| i < self.overlays.len())
    }

    /// The active overlay id, if one is selected.
    #[must_use]
    pub fn active_id(&self) -> Option<&'static str> {
        self.active_overlay().map(|overlay| overlay.id)
    }

    /// Whether any overlay is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Cycle to the next overlay, wrapping through `None` (off) at the end.
    ///
    /// Order: off → 0 → 1 → … → n-1 → off.
    pub fn cycle(&mut self) {
        let n = self.overlays.len();
        self.active = match self.active {
            None if n > 0 => Some(0),
            Some(i) if i + 1 < n => Some(i + 1),
            _ => None,
        };
    }

    /// Activate the overlay with the given id, returning `true` on success.
    pub fn activate_id(&mut self, id: &str) -> bool {
        if let Some(i) = self.overlays.iter().position(|o| o.id == id) {
            self.active = Some(i);
            true
        } else {
            false
        }
    }

    /// Toggle the overlay with the given id.
    ///
    /// If the requested overlay is already active, the suite turns off. If a
    /// different overlay is active, this switches to the requested overlay.
    /// Returns `true` when the id exists in the registry.
    pub fn toggle_id(&mut self, id: &str) -> bool {
        let Some(index) = self.overlays.iter().position(|overlay| overlay.id == id) else {
            return false;
        };

        self.active = match self.active {
            Some(active) if active == index => None,
            _ => Some(index),
        };
        true
    }

    /// Activate the overlay at the given index, returning `true` on success.
    pub fn activate_index(&mut self, index: usize) -> bool {
        if index < self.overlays.len() {
            self.active = Some(index);
            true
        } else {
            false
        }
    }

    /// Turn the suite off (terrain shows through).
    pub fn deactivate(&mut self) {
        self.active = None;
    }
}

/// Live nearby-entity counts for the Tab overlay (P1.3.2).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NearbyEntityCounts {
    /// Civilians within the nearby radius.
    pub civilians: usize,
    /// Buildings within the nearby radius.
    pub buildings: usize,
    /// Civilian count per faction id.
    pub factions: std::collections::BTreeMap<u32, usize>,
}

fn faction_letter_label(id: u32) -> String {
    if id < 26 {
        format!("Faction {}", (b'A' + id as u8) as char)
    } else {
        format!("Faction {id}")
    }
}

/// Format the nearby summary line shown in the Tab overlay.
#[must_use]
pub fn format_nearby_counts_line(counts: &NearbyEntityCounts) -> String {
    let civ_word = if counts.civilians == 1 {
        "civilian"
    } else {
        "civilians"
    };
    let bld_word = if counts.buildings == 1 {
        "building"
    } else {
        "buildings"
    };
    let faction_part = if counts.factions.is_empty() || counts.civilians == 0 {
        String::new()
    } else {
        let breakdown: Vec<String> = counts
            .factions
            .iter()
            .map(|(id, count)| format!("{}: {count}", faction_letter_label(*id)))
            .collect();
        format!(" ({})", breakdown.join(", "))
    };
    format!(
        "Nearby: {} {}{}, {} {}.",
        counts.civilians, civ_word, faction_part, counts.buildings, bld_word
    )
}

#[cfg(feature = "egui")]
pub use plugin::*;

#[cfg(feature = "egui")]
mod plugin {
    use super::*;
    use crate::camera::CameraRig;
    use crate::live_attach::is_server_attach_mode;
    use crate::live_stream::{LiveAgentTag, LiveBuildingTag, LiveStreamScene};
    use crate::sim_bridge::{SimBuildingMarker, SimCivilianMarker, SimState};
    use crate::ui_theme::{TEXT, TEXT_LOW};
    use crate::AttachMode;
    use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
    use civ_agents::{Civilian, Needs};

    /// Radius (world units) around the camera target for nearby-entity counts.
    const NEARBY_RADIUS: f32 = 100.0;

    /// Tab-toggled HUD listing live nearby entity counts (P1.3.2).
    #[derive(Resource, Default, Debug)]
    pub struct NearbyCountsOverlay {
        /// Whether the semi-transparent summary panel is visible.
        pub visible: bool,
    }

    /// Plugin: overlay registry, Tab nearby-counts HUD, and terrain recolor pass.
    pub struct InfoViewsPlugin;

    impl Plugin for InfoViewsPlugin {
        fn build(&self, app: &mut App) {
            // The overlay PICKER UI lives in the left HUD cluster's "Info Views"
            // tab (see `info_view_tab_body`); this plugin keeps the registry,
            // the Tab nearby-counts overlay, and the terrain recolor pass.
            app.init_resource::<InfoViewRegistry>()
                .init_resource::<NearbyCountsOverlay>()
                .add_systems(Update, (toggle_nearby_overlay_hotkey, render_active_overlay))
                .add_systems(
                    EguiPrimaryContextPass,
                    draw_nearby_counts_overlay.run_if(crate::menus::in_game),
                );
        }
    }

    fn toggle_nearby_overlay_hotkey(
        keys: Res<ButtonInput<KeyCode>>,
        mut overlay: ResMut<NearbyCountsOverlay>,
    ) {
        if keys.just_pressed(KeyCode::Tab) {
            overlay.visible = !overlay.visible;
        }
        if keys.just_pressed(KeyCode::Escape) && overlay.visible {
            overlay.visible = false;
        }
    }

    fn is_nearby(eye: Vec3, pos: Vec3) -> bool {
        let dx = pos.x - eye.x;
        let dz = pos.z - eye.z;
        (dx * dx + dz * dz) <= NEARBY_RADIUS * NEARBY_RADIUS
    }

    fn collect_nearby_counts_standalone(
        eye: Vec3,
        civilians: &Query<(&GlobalTransform, &SimCivilianMarker)>,
        buildings: &Query<&GlobalTransform, With<SimBuildingMarker>>,
    ) -> NearbyEntityCounts {
        let mut counts = NearbyEntityCounts::default();
        for (transform, marker) in civilians.iter() {
            if !is_nearby(eye, transform.translation()) {
                continue;
            }
            counts.civilians += 1;
            *counts.factions.entry(marker.faction).or_insert(0) += 1;
        }
        for transform in buildings.iter() {
            if is_nearby(eye, transform.translation()) {
                counts.buildings += 1;
            }
        }
        counts
    }

    fn collect_nearby_counts_live(
        eye: Vec3,
        scene: Option<&LiveStreamScene>,
        agents: &Query<(&GlobalTransform, &LiveAgentTag)>,
        buildings: &Query<&GlobalTransform, With<LiveBuildingTag>>,
    ) -> NearbyEntityCounts {
        let mut counts = NearbyEntityCounts::default();
        let civilian_entries = scene.map(|s| &s.civilian_entries);
        for (transform, tag) in agents.iter() {
            if !is_nearby(eye, transform.translation()) {
                continue;
            }
            counts.civilians += 1;
            let faction = civilian_entries
                .and_then(|entries| entries.get(&tag.id))
                .map(|entry| entry.faction_id)
                .unwrap_or(0);
            *counts.factions.entry(faction).or_insert(0) += 1;
        }
        for transform in buildings.iter() {
            if is_nearby(eye, transform.translation()) {
                counts.buildings += 1;
            }
        }
        counts
    }

    fn draw_nearby_counts_overlay(
        mut contexts: EguiContexts,
        overlay: Res<NearbyCountsOverlay>,
        attach: Res<AttachMode>,
        rig: Res<CameraRig>,
        sim_civilians: Query<(&GlobalTransform, &SimCivilianMarker)>,
        sim_buildings: Query<&GlobalTransform, With<SimBuildingMarker>>,
        live_agents: Query<(&GlobalTransform, &LiveAgentTag)>,
        live_buildings: Query<&GlobalTransform, With<LiveBuildingTag>>,
        scene: Option<Res<LiveStreamScene>>,
    ) {
        if !overlay.visible {
            return;
        }

        let eye = rig.target;
        let counts = if is_server_attach_mode(*attach) {
            collect_nearby_counts_live(eye, scene.as_deref(), &live_agents, &live_buildings)
        } else {
            collect_nearby_counts_standalone(eye, &sim_civilians, &sim_buildings)
        };
        let summary = format_nearby_counts_line(&counts);

        let Ok(ctx) = contexts.ctx_mut() else {
            return;
        };
        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("nearby_counts_overlay"))
            .fixed_pos(egui::pos2(screen.center().x - 220.0, 72.0))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 200))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(126, 186, 181)))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(16_i8, 10_i8))
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);
                        ui.label(
                            egui::RichText::new("Info View — Nearby")
                                .heading()
                                .color(egui::Color32::from_rgb(126, 186, 181)),
                        );
                        ui.label(egui::RichText::new(summary).color(TEXT).size(15.0));
                        ui.label(
                            egui::RichText::new(format!(
                                "Within {NEARBY_RADIUS:.0}m of camera focus · Tab or Esc to close"
                            ))
                            .color(TEXT_LOW)
                            .small(),
                        );
                    });
            });
    }

    /// Draw the Info Views overlay picker + legend as a tab body (no window
    /// chrome). Used by the left HUD cluster's "Info Views" tab so the overlay
    /// suite lives inside the unified left panel instead of a separate window.
    pub fn info_view_tab_body(ui: &mut egui::Ui, registry: &mut InfoViewRegistry) {
        ui.label(egui::RichText::new("Terrain overlay:").color(TEXT));
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(!registry.is_active(), "Off").clicked() {
                registry.deactivate();
            }
            let count = registry.overlays.len();
            for i in 0..count {
                let (name, desc, selected) = {
                    let o = &registry.overlays[i];
                    (o.name, o.description, registry.active == Some(i))
                };
                if ui
                    .selectable_label(selected, name)
                    .on_hover_text(desc)
                    .clicked()
                {
                    registry.active = Some(i);
                }
            }
        });
        if let Some(overlay) = registry.active_overlay() {
            ui.separator();
            ui.label(
                egui::RichText::new(format!("Legend — {}", overlay.name))
                    .heading()
                    .color(TEXT),
            );
            draw_legend(ui, overlay);
        }
    }

    fn draw_legend(ui: &mut egui::Ui, overlay: &InfoOverlay) {
        if overlay.legend.is_empty() {
            ui.label("Distinct colour per territory");
            return;
        }
        for stop in overlay.legend {
            ui.horizontal(|ui| {
                let c = stop.rgb;
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(18.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    2.0,
                    egui::Color32::from_rgb(
                        (c[0] * 255.0) as u8,
                        (c[1] * 255.0) as u8,
                        (c[2] * 255.0) as u8,
                    ),
                );
                ui.label(egui::RichText::new(stop.label).color(TEXT_LOW).small());
            });
        }
    }

    /// Render the active overlay as a recolor gizmo grid over the terrain.
    ///
    /// Samples the terrain field on a `grid_resolution`² lattice, fills
    /// sim-derived fields from `SimState`, runs the overlay colour function, and
    /// draws a coloured quad (two gizmo triangles → cross) hovering just above
    /// the surface. Cheap, deterministic, and GPU-light for the sandbox.
    fn render_active_overlay(
        registry: Res<InfoViewRegistry>,
        sim: Res<SimState>,
        mut gizmos: Gizmos,
    ) {
        let Some(overlay) = registry.active_overlay() else {
            return;
        };
        let res = registry.grid_resolution.max(2);
        let half = WORLD_SIZE * 0.5;
        let step = WORLD_SIZE / res as f32;
        let cell = step * 0.45;

        let agents = if overlay.uses_sim {
            collect_agent_field(&sim, res)
        } else {
            AgentField::empty(res)
        };

        for gz in 0..res {
            for gx in 0..res {
                let wx = -half + (gx as f32 + 0.5) * step;
                let wz = -half + (gz as f32 + 0.5) * step;
                let mut sample = terrain_sample(wx, wz);
                if overlay.uses_sim {
                    let (density, needs, cluster) = agents.cell(gx, gz);
                    sample.population_density = density;
                    sample.needs_pressure = needs;
                    sample.cluster = cluster;
                }
                let Some(color) = (overlay.color_fn)(&sample) else {
                    continue;
                };
                let pos = Vec3::new(wx, sample.height + 0.6, wz);
                let tint = Color::srgba(color[0], color[1], color[2], color[3]);
                // Draw a small "+" gizmo per cell as the recolor marker.
                gizmos.line(pos - Vec3::X * cell, pos + Vec3::X * cell, tint);
                gizmos.line(pos - Vec3::Z * cell, pos + Vec3::Z * cell, tint);
            }
        }
    }

    /// Per-cell aggregate of the live agent world for sim-backed overlays.
    struct AgentField {
        res: usize,
        density: Vec<f32>,
        needs: Vec<f32>,
        cluster: Vec<Option<u32>>,
    }

    impl AgentField {
        fn empty(res: usize) -> Self {
            Self {
                res,
                density: vec![0.0; res * res],
                needs: vec![0.0; res * res],
                cluster: vec![None; res * res],
            }
        }

        fn cell(&self, gx: usize, gz: usize) -> (f32, f32, Option<u32>) {
            let i = gz * self.res + gx;
            (self.density[i], self.needs[i], self.cluster[i])
        }
    }

    /// Build the per-cell agent field from `SimState`: counts, mean needs, and
    /// dominant faction per lattice cell, with density normalised to the busiest
    /// cell so the ramp always spans the active world.
    fn collect_agent_field(sim: &SimState, res: usize) -> AgentField {
        let mut field = AgentField::empty(res);
        let mut counts = vec![0u32; res * res];
        let mut needs_sum = vec![0.0f32; res * res];
        let mut faction_votes: Vec<std::collections::HashMap<u32, u32>> =
            vec![std::collections::HashMap::new(); res * res];

        let mut world = sim.0.world.query::<(&Civilian, Option<&Needs>)>();
        for (_, (civ, needs)) in world.iter() {
            // Civilians spawn at normalised XY; map to lattice via the same
            // world-norm convention used by the sim bridge.
            let (nx, nz) = agent_norm_xy(civ.id);
            let gx = ((nx * res as f32) as usize).min(res - 1);
            let gz = ((nz * res as f32) as usize).min(res - 1);
            let i = gz * res + gx;
            counts[i] += 1;
            if let Some(n) = needs {
                needs_sum[i] += needs_pressure(n);
            }
            *faction_votes[i]
                .entry(civilian_faction_id(civ))
                .or_insert(0) += 1;
        }

        let max_count = counts.iter().copied().max().unwrap_or(0).max(1) as f32;
        for i in 0..res * res {
            field.density[i] = counts[i] as f32 / max_count;
            field.needs[i] = if counts[i] > 0 {
                (needs_sum[i] / counts[i] as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            field.cluster[i] = faction_votes[i]
                .iter()
                .max_by_key(|(_, c)| **c)
                .map(|(f, _)| *f);
        }
        field
    }

    /// Stable normalised XY for an agent id (sandbox agents lack a live
    /// `Position3d`; reuse the deterministic spawn hash so the overlay is
    /// reproducible run-to-run).
    fn agent_norm_xy(id: u64) -> (f32, f32) {
        let h = id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let nx = ((h >> 11) as f32 / (1u64 << 53) as f32).fract();
        let nz = ((h >> 5) as f32 / (1u64 << 53) as f32).fract();
        (nx.clamp(0.0, 1.0), nz.clamp(0.0, 1.0))
    }

    /// Mean of the four need scalars → single pressure value in `0.0..=1.0`.
    fn needs_pressure(n: &Needs) -> f32 {
        ((n.food + n.shelter + n.safety + n.belonging) / 4.0).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-INFOVIEW-900 — registry ships the high-value overlays.
    #[test]
    fn registry_ships_high_value_overlays() {
        let reg = InfoViewRegistry::default();
        for id in [
            "elevation",
            "water",
            "temperature",
            "material",
            "population",
            "needs",
            "cluster",
        ] {
            assert!(
                reg.overlays.iter().any(|o| o.id == id),
                "missing overlay {id}"
            );
        }
        assert!(!reg.is_active(), "default starts off");
    }

    /// P1.3.2 — nearby summary formats civilians, factions, and buildings.
    #[test]
    fn nearby_counts_line_formats_breakdown() {
        let counts = NearbyEntityCounts {
            civilians: 12,
            buildings: 3,
            factions: [(0, 8), (1, 4)].into_iter().collect(),
        };
        let line = format_nearby_counts_line(&counts);
        assert!(line.contains("12 civilians"));
        assert!(line.contains("Faction A: 8"));
        assert!(line.contains("Faction B: 4"));
        assert!(line.contains("3 buildings"));
    }

    /// FR-CIV-INFOVIEW-900 — cycle walks off → each → off.
    #[test]
    fn cycle_wraps_through_off() {
        let mut reg = InfoViewRegistry::default();
        let n = reg.overlays.len();
        assert_eq!(reg.active, None);
        for expect in 0..n {
            reg.cycle();
            assert_eq!(reg.active, Some(expect));
        }
        reg.cycle();
        assert_eq!(reg.active, None, "wraps back to off");
    }

    /// FR-CIV-INFOVIEW-900 — activate by id is data-driven.
    #[test]
    fn activate_by_id_selects_overlay() {
        let mut reg = InfoViewRegistry::default();
        assert!(reg.activate_id("temperature"));
        assert_eq!(reg.active_overlay().map(|o| o.id), Some("temperature"));
        assert!(!reg.activate_id("does-not-exist"));
    }

    /// FR-CIV-INFOVIEW-900 — toggling by id is data-driven and idempotent.
    #[test]
    fn toggle_by_id_switches_and_turns_off() {
        let mut reg = InfoViewRegistry::default();
        assert!(reg.toggle_id("water"));
        assert_eq!(reg.active_id(), Some("water"));
        assert!(reg.toggle_id("elevation"));
        assert_eq!(reg.active_id(), Some("elevation"));
        assert!(reg.toggle_id("elevation"));
        assert_eq!(reg.active_id(), None);
        assert!(!reg.toggle_id("missing"));
    }

    /// FR-CIV-INFOVIEW-900 — activation by index supports registry-driven UI.
    #[test]
    fn activate_by_index_tracks_selection() {
        let mut reg = InfoViewRegistry::default();
        assert!(reg.activate_index(2));
        assert_eq!(reg.active_index(), Some(2));
        assert_eq!(reg.active_overlay().map(|o| o.id), Some("temperature"));
        assert!(!reg.activate_index(reg.overlays.len()));
    }

    /// FR-CIV-INFOVIEW-901 — legend ramp interpolates and clamps.
    #[test]
    fn ramp_color_interpolates_and_clamps() {
        let lo = ramp_color(RAMP_TEMPERATURE, -1.0);
        assert_eq!(lo, RAMP_TEMPERATURE[0].rgb);
        let hi = ramp_color(RAMP_TEMPERATURE, 2.0);
        assert_eq!(hi, RAMP_TEMPERATURE[RAMP_TEMPERATURE.len() - 1].rgb);
        let mid = ramp_color(RAMP_TEMPERATURE, 0.5);
        assert_eq!(mid, RAMP_TEMPERATURE[1].rgb);
        // A value between stops blends.
        let q = ramp_color(RAMP_TEMPERATURE, 0.25);
        assert!(q[0] > RAMP_TEMPERATURE[0].rgb[0]);
    }

    /// FR-CIV-INFOVIEW-910 — elevation overlay always tints (terrain-backed).
    #[test]
    fn elevation_overlay_always_returns_color() {
        let reg = InfoViewRegistry::default();
        let elev = reg.overlays.iter().find(|o| o.id == "elevation").unwrap();
        let s = terrain_sample(0.0, 0.0);
        assert!((elev.color_fn)(&s).is_some());
    }

    /// FR-CIV-INFOVIEW-910 — population overlay skips empty cells.
    #[test]
    fn population_overlay_skips_empty_cells() {
        let reg = InfoViewRegistry::default();
        let pop = reg.overlays.iter().find(|o| o.id == "population").unwrap();
        let empty = terrain_sample(0.0, 0.0); // density 0.0
        assert!((pop.color_fn)(&empty).is_none());
        let mut occupied = empty;
        occupied.population_density = 0.8;
        assert!((pop.color_fn)(&occupied).is_some());
    }

    /// FR-CIV-INFOVIEW-910 — water overlay flags submerged cells.
    #[test]
    fn water_overlay_flags_submerged() {
        let mut s = terrain_sample(0.0, 0.0);
        s.height = WATER_LEVEL - 1.0;
        s.height_norm = s.height / HEIGHT_SCALE;
        assert!(color_water(&s).is_some());
    }

    /// Cluster colours are deterministic + distinct.
    #[test]
    fn cluster_colors_distinct_and_stable() {
        assert_eq!(cluster_color(1), cluster_color(1));
        assert_ne!(cluster_color(1), cluster_color(2));
        for c in cluster_color(7) {
            assert!((0.0..=1.0).contains(&c));
        }
    }

    /// terrain_sample fills height fields from the procedural field.
    #[test]
    fn terrain_sample_populates_height() {
        let s = terrain_sample(10.0, -10.0);
        assert!(s.height >= 0.0 && s.height <= HEIGHT_SCALE);
        assert!((s.height_norm - s.height / HEIGHT_SCALE).abs() < f32::EPSILON);
    }
}
