#![cfg(all(feature = "bevy", feature = "egui"))]
//! God-mode intervention panel (FR-CIV-GAME-002). G key toggles.
//!
//! # FR-CLIENT-godbuttons — substrate-live verb expansion
//!
//! PR #762 landed the first click-to-fire egui button (the `Invoke: …`
//! button below) which posts `sim.god_action` over JSON-RPC. The dispatch
//! effect (`DispatchEffect::GodAction` in `crates/server/src/jsonrpc.rs`
//! + the `apply_dispatch_effect` handler in `crates/server/src/ws_bridge.rs`)
//! was then widened so 13 more substrate-live verbs route through the
//! **same** JSON-RPC method:
//!
//! * TERRAIN: `terrain.add_land`, `terrain.dig_ocean`, `terrain.drop_biome`
//! * MATERIAL: `material.erase`, `material.replace`, `material.surface_paint`,
//!   `material.pour_liquid`, `material.seed_snow`, `material.seed_ore`
//! * LIFE: `life.spawn_organism`, `life.spawn_herd`
//! * DISASTER: `disaster.wildfire`, `disaster.flood`
//!
//! Every button in the **Substrate Tools** section below fires one of those
//! verbs by calling `bridge.client.send_rpc("sim.god_action", payload)` with
//! the matching `GodToolRequest`-shaped params — exactly mirroring the
//! `Invoke` button's wire path. The `ws_bridge` handler then routes them
//! through `Simulation::apply_god_tool` (the substrate-owned write path)
//! so the CA, replay log, and dirty-event invariant stay intact.
//!
//! See:
//! * `crates/engine/src/godtools.rs` — verb request / receipt shapes.
//! * `crates/server/src/jsonrpc.rs` — `SimGodAction` param parser.
//! * `crates/server/src/ws_bridge.rs` — verb → `GodToolRequest` resolver.
//! * `docs/specs/CIV-0700-modding-api-spec.md` — substrate verb catalog.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::menus::in_game;
use crate::live_stream::LiveBridge;

#[derive(Resource, Default)]
pub struct GodPanelState {
    pub visible: bool,
    pub selected_action: usize,
    pub magnitude: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_faction: u32,
    pub status: Option<String>,

    // FR-CLIENT-godbuttons: rich-verb params shared by the new buttons.
    /// Brush radius in voxels (TERRAIN / MATERIAL verbs; defaults to 3).
    pub radius_voxels: u8,
    /// Target material id (MATERIAL verbs; defaults to STONE).
    pub material_id: u32,
    /// Drop height in fixed-point units (MATERIAL.pour_liquid).
    pub drop_height: i32,
    /// Agent count for `life.spawn_herd` (clamped to `[1, 64]`).
    pub herd_count: u32,
    /// Sticky id seed for `life.spawn_organism` / `life.spawn_herd`
    /// (defaults to a tick-derived id when 0).
    pub seed_civilian_id: u64,
}

const ACTIONS: &[&str] = &["smite", "bless", "earthquake", "plague", "miracle"];
const ACTION_DESCS: &[&str] = &[
    "Strike (x,y) with a meteor — terrain damage, fires, belief spike",
    "Boost target faction treasury + belief",
    "Trigger ground quake at (x,y) — rubble, infrastructure damage",
    "Reduce target faction treasury (disease proxy)",
    "Raise all faction belief + small treasury boost",
];

// FR-CLIENT-godbuttons: per-verb metadata table for the Substrate Tools
// section. `verb` is the exact action id the `ws_bridge` resolver matches on;
// `param_builder` fills the JSON-RPC `params` object from `GodPanelState`
// (mirroring the `GodToolRequest` shape the engine expects); `label` /
// `desc` drive the egui button.
struct GodToolVerb {
    verb: &'static str,
    category: &'static str,
    label: &'static str,
    desc: &'static str,
    param_builder: fn(&GodPanelState) -> serde_json::Value,
}

fn build_terrain_add_land(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "terrain.add_land",
        "x": s.target_x,
        "y": s.target_y,
        "magnitude": s.magnitude,
        "radius_voxels": s.radius_voxels.max(1),
        "strength": s.magnitude.max(0.0) as i32 * 256,
    })
}
fn build_terrain_dig_ocean(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "terrain.dig_ocean",
        "x": s.target_x,
        "y": s.target_y,
        "magnitude": s.magnitude,
        "radius_voxels": s.radius_voxels.max(1),
        "strength": s.magnitude.max(0.0) as i32 * 512,
    })
}
fn build_terrain_drop_biome(s: &GodPanelState) -> serde_json::Value {
    // aux_id = material id; the ws_bridge routes this into
    // `TerraformRequest::aux_id` for `TerraformOp::DropBiome`.
    serde_json::json!({
        "action": "terrain.drop_biome",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
        "material_id": s.material_id,
    })
}
fn build_material_erase(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "material.erase",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
    })
}
fn build_material_replace(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "material.replace",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
        "material_id": s.material_id,
    })
}
fn build_material_surface_paint(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "material.surface_paint",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
        "material_id": s.material_id,
    })
}
fn build_material_pour_liquid(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "material.pour_liquid",
        "x": s.target_x,
        "y": s.target_y,
        "magnitude": s.magnitude,
        "radius_voxels": s.radius_voxels.max(1),
        "material_id": s.material_id,
        "drop_height": s.drop_height,
        "strength": s.magnitude.max(0.0) as i32 * 256,
    })
}
fn build_material_seed_snow(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "material.seed_snow",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
        "strength": 256, // FIXED_SCALE; ws_bridge maps to i32
    })
}
fn build_material_seed_ore(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "material.seed_ore",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
        "strength": 256,
    })
}
fn build_life_spawn_organism(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "life.spawn_organism",
        "x": s.target_x,
        "y": s.target_y,
        "target_faction": s.target_faction,
        "seed_civilian_id": s.seed_civilian_id,
    })
}
fn build_life_spawn_herd(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "life.spawn_herd",
        "target_faction": s.target_faction,
        "count": s.herd_count.clamp(1, 64),
        "seed_civilian_id": s.seed_civilian_id,
    })
}
fn build_disaster_wildfire(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "disaster.wildfire",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
    })
}
fn build_disaster_flood(s: &GodPanelState) -> serde_json::Value {
    serde_json::json!({
        "action": "disaster.flood",
        "x": s.target_x,
        "y": s.target_y,
        "radius_voxels": s.radius_voxels.max(1),
    })
}

/// Substrate-live verbs added under FR-CLIENT-godbuttons. Order is the
/// rendering order inside the panel — TERRAIN first (terrain is what the
/// player sees change most), then MATERIAL (paint/pour), then LIFE, then
/// DISASTER. Each entry corresponds to a verb the ws_bridge resolver
/// already routes through `Simulation::apply_god_tool`.
const SUBSTRATE_VERBS: &[GodToolVerb] = &[
    GodToolVerb {
        verb: "terrain.add_land",
        category: "TERRAIN",
        label: "Add Land",
        desc: "Stamp a chunky STONE band on the surface at (x,y)",
        param_builder: build_terrain_add_land,
    },
    GodToolVerb {
        verb: "terrain.dig_ocean",
        category: "TERRAIN",
        label: "Dig Ocean",
        desc: "Dig a basin at (x,y) and fill with WATER",
        param_builder: build_terrain_dig_ocean,
    },
    GodToolVerb {
        verb: "terrain.drop_biome",
        category: "TERRAIN",
        label: "Drop Biome",
        desc: "Repaint topmost surface to material_id at (x,y)",
        param_builder: build_terrain_drop_biome,
    },
    GodToolVerb {
        verb: "material.erase",
        category: "MATERIAL",
        label: "Erase",
        desc: "Write AIR in the brush footprint at (x,y)",
        param_builder: build_material_erase,
    },
    GodToolVerb {
        verb: "material.replace",
        category: "MATERIAL",
        label: "Replace",
        desc: "Overwrite brush with material_id at (x,y)",
        param_builder: build_material_replace,
    },
    GodToolVerb {
        verb: "material.surface_paint",
        category: "MATERIAL",
        label: "Surface Paint",
        desc: "Paint only the topmost solid voxel per column",
        param_builder: build_material_surface_paint,
    },
    GodToolVerb {
        verb: "material.pour_liquid",
        category: "MATERIAL",
        label: "Pour Liquid",
        desc: "Drop material_id from drop_height above (x,y)",
        param_builder: build_material_pour_liquid,
    },
    GodToolVerb {
        verb: "material.seed_snow",
        category: "MATERIAL",
        label: "Seed Snow",
        desc: "Write SNOW voxels in the brush footprint",
        param_builder: build_material_seed_snow,
    },
    GodToolVerb {
        verb: "material.seed_ore",
        category: "MATERIAL",
        label: "Seed Ore",
        desc: "Write ORE voxels in a stochastic vein pattern",
        param_builder: build_material_seed_ore,
    },
    GodToolVerb {
        verb: "life.spawn_organism",
        category: "LIFE",
        label: "Spawn Civ",
        desc: "Inject one humanoid civilian at (x,y)",
        param_builder: build_life_spawn_organism,
    },
    GodToolVerb {
        verb: "life.spawn_herd",
        category: "LIFE",
        label: "Spawn Herd",
        desc: "Inject herd_count fauna on faction target_faction",
        param_builder: build_life_spawn_herd,
    },
    GodToolVerb {
        verb: "disaster.wildfire",
        category: "DISASTER",
        label: "Wildfire",
        desc: "Ignite flammables in the brush footprint",
        param_builder: build_disaster_wildfire,
    },
    GodToolVerb {
        verb: "disaster.flood",
        category: "DISASTER",
        label: "Flood",
        desc: "Flood the brush footprint with WATER",
        param_builder: build_disaster_flood,
    },
];

/// Category order — drives the egui rendering order so TERRAIN appears
/// first (most visually obvious), then MATERIAL, LIFE, DISASTER. We walk
/// `SUBSTRATE_VERBS` and emit a header each time `v.category` changes,
/// so the iteration order of the table below must match this list. New
/// categories can be appended; existing ones should not be re-ordered.
const SUBSTRATE_CATEGORY_ORDER: &[&str] = &["TERRAIN", "MATERIAL", "LIFE", "DISASTER"];

pub struct GodPanelPlugin;
impl Plugin for GodPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GodPanelState>()
           .add_systems(Update, (toggle_god_panel, draw_god_panel).chain().run_if(in_game));
    }
}

fn toggle_god_panel(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<GodPanelState>) {
    if keys.just_pressed(KeyCode::KeyG) {
        state.visible = !state.visible;
        if state.magnitude == 0.0 { state.magnitude = 0.5; }
        // FR-CLIENT-godbuttons: lazy-init the substrate-verb defaults so
        // a freshly-opened panel doesn't ship 0s for every rich param.
        if state.radius_voxels == 0 { state.radius_voxels = 3; }
        if state.material_id == 0 { state.material_id = 6; } // STONE
        if state.drop_height == 0 { state.drop_height = 768; } // 3 × FIXED_SCALE
        if state.herd_count == 0 { state.herd_count = 5; }
    }
}

fn draw_god_panel(
    mut contexts: EguiContexts,
    mut state: ResMut<GodPanelState>,
    bridge: Res<LiveBridge>,
) {
    if !state.visible { return; }
    let ctx = contexts.ctx_mut();
    let screen = ctx.screen_rect();

    let mut fire_legacy: Option<String> = None;
    let mut fire_substrate: Option<usize> = None;
    egui::Window::new("God Mode")
        .fixed_pos(egui::pos2(screen.max.x - 310.0, screen.center().y - 180.0))
        // FR-CLIENT-godbuttons: taller panel for the Substrate Tools
        // section (13 buttons across 4 categories + brush params).
        .fixed_size([310.0, 720.0])
        .collapsible(false)
        .title_bar(true)
        .frame(egui::Frame::window(ctx.style().as_ref())
            .fill(egui::Color32::from_rgba_premultiplied(9,10,12,230))
            .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(126,186,181))))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Direct Intervention").color(egui::Color32::from_rgb(126,186,181)).size(11.0));
            ui.separator();

            // Action selector
            for (idx, (name, desc)) in ACTIONS.iter().zip(ACTION_DESCS.iter()).enumerate() {
                let selected = state.selected_action == idx;
                let color = if selected { egui::Color32::from_rgb(126,186,181) } else { egui::Color32::from_rgb(160,170,180) };
                if ui.add(egui::SelectableLabel::new(selected, egui::RichText::new(*name).color(color).monospace())).clicked() {
                    state.selected_action = idx;
                }
                if selected {
                    ui.label(egui::RichText::new(*desc).color(egui::Color32::from_rgb(120,130,140)).size(10.0).italics());
                }
            }
            ui.separator();

            // Magnitude
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Magnitude:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                ui.add(egui::Slider::new(&mut state.magnitude, 0.0..=1.0).show_value(true));
            });

            // Target coords (for smite/earthquake)
            let action_name = ACTIONS[state.selected_action];
            let needs_pos = matches!(action_name, "smite" | "earthquake");
            let needs_faction = matches!(action_name, "bless" | "plague");

            if needs_pos {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("X:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                    ui.add(egui::DragValue::new(&mut state.target_x).speed(0.01).clamp_range(0.0..=1.0f32));
                    ui.label(egui::RichText::new("Y:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                    ui.add(egui::DragValue::new(&mut state.target_y).speed(0.01).clamp_range(0.0..=1.0f32));
                });
            }

            if needs_faction {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Faction:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                    ui.add(egui::DragValue::new(&mut state.target_faction).speed(1.0).clamp_range(0..=255u32));
                });
            }

            ui.separator();
            let fire_btn = ui.add_sized([290.0, 28.0], egui::Button::new(
                egui::RichText::new(format!("Invoke: {}", action_name)).color(egui::Color32::from_rgb(9,10,12)).size(13.0)
            ).fill(egui::Color32::from_rgb(126,186,181)));
            if fire_btn.clicked() {
                fire_legacy = Some(action_name.to_string());
            }

            // ============ FR-CLIENT-godbuttons: Substrate Tools ============
            // 13 buttons firing substrate-live verbs via the SAME JSON-RPC
            // path as the Invoke button above (`sim.god_action`). The
            // `param_builder` for each verb fills the matching rich param
            // shape from `GodPanelState`; the `ws_bridge` resolver then
            // routes the call through `Simulation::apply_god_tool`.
            ui.separator();
            ui.label(egui::RichText::new("Substrate Tools (FR-CLIENT-godbuttons)")
                .color(egui::Color32::from_rgb(126,186,181)).size(11.0));
            ui.separator();

            // Shared brush params — apply to every substrate verb below.
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Target (x,y):").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                ui.add(egui::DragValue::new(&mut state.target_x).speed(0.01).clamp_range(0.0..=1.0f32));
                ui.add(egui::DragValue::new(&mut state.target_y).speed(0.01).clamp_range(0.0..=1.0f32));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Radius:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                ui.add(egui::DragValue::new(&mut state.radius_voxels).speed(1.0).clamp_range(1..=32u8));
                ui.label(egui::RichText::new("MatId:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                ui.add(egui::DragValue::new(&mut state.material_id).speed(1.0).clamp_range(1..=255u32));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Drop:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                ui.add(egui::DragValue::new(&mut state.drop_height).speed(16.0).clamp_range(0..=16384i32));
                ui.label(egui::RichText::new("Herd:").color(egui::Color32::from_rgb(160,170,180)).size(11.0));
                ui.add(egui::DragValue::new(&mut state.herd_count).speed(1.0).clamp_range(1..=64u32));
            });

            // Per-category button grid. We walk the SUBSTRATE_VERBS table
            // once and emit a category header the first time we see each
            // label; remaining verbs in that category follow under it.
            let mut last_category: Option<&'static str> = None;
            for (idx, v) in SUBSTRATE_VERBS.iter().enumerate() {
                if last_category != Some(v.category) {
                    ui.separator();
                    ui.label(egui::RichText::new(v.category)
                        .color(egui::Color32::from_rgb(126,186,181))
                        .monospace().size(10.5));
                    last_category = Some(v.category);
                }
                let btn = ui.add_sized([290.0, 22.0], egui::Button::new(
                    egui::RichText::new(format!("{}  [{}]", v.label, v.verb))
                        .color(egui::Color32::from_rgb(9, 10, 12))
                        .size(11.0),
                ).fill(egui::Color32::from_rgb(126, 186, 181)));
                if btn.clicked() {
                    fire_substrate = Some(idx);
                }
                ui.label(egui::RichText::new(v.desc)
                    .color(egui::Color32::from_rgb(120, 130, 140))
                    .size(9.0).italics());
            }
            // Sanity check: every verb's category must appear in
            // SUBSTRATE_CATEGORY_ORDER so the egui header logic below
            // groups them correctly. Only checked in debug builds.
            debug_assert!(
                SUBSTRATE_VERBS
                    .iter()
                    .all(|v| SUBSTRATE_CATEGORY_ORDER.contains(&v.category)),
                "every SUBSTRATE_VERBS entry's category must appear in SUBSTRATE_CATEGORY_ORDER",
            );

            if let Some(ref msg) = state.status {
                ui.separator();
                ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(200,200,100)).size(10.0));
            }
        });

    // Dispatch legacy `Invoke` button (smite / bless / earthquake / plague /
    // miracle). Same payload shape as before — the `ws_bridge` resolver
    // continues to handle the legacy 5 verbs unchanged.
    if let Some(action) = fire_legacy {
        let needs_pos = matches!(action.as_str(), "smite" | "earthquake");
        let needs_faction = matches!(action.as_str(), "bless" | "plague");
        let payload = serde_json::json!({
            "action": action,
            "x": if needs_pos { Some(state.target_x) } else { None::<f32> },
            "y": if needs_pos { Some(state.target_y) } else { None::<f32> },
            "target_faction": if needs_faction { Some(state.target_faction) } else { None::<u32> },
            "magnitude": state.magnitude,
        });
        bridge.client.send_rpc("sim.god_action", payload);
        state.status = Some(format!("Invoked: {}", ACTIONS[state.selected_action]));
    }

    // Dispatch a substrate verb button. The `param_builder` produces the
    // matching `GodToolRequest` JSON shape; `sim.god_action` is the same
    // JSON-RPC method the legacy Invoke button uses, so this stays on
    // the exact wire path PR #762 wired up.
    if let Some(idx) = fire_substrate {
        let v = &SUBSTRATE_VERBS[idx];
        let payload = (v.param_builder)(&state);
        bridge.client.send_rpc("sim.god_action", payload);
        state.status = Some(format!("Fired: {} ({})", v.label, v.verb));
    }
}