#![cfg(all(feature = "bevy", feature = "egui"))]
//! God-mode intervention panel (FR-CIV-GAME-002). G key toggles.

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
}

const ACTIONS: &[&str] = &["smite", "bless", "earthquake", "plague", "miracle"];
const ACTION_DESCS: &[&str] = &[
    "Strike (x,y) with a meteor — terrain damage, fires, belief spike",
    "Boost target faction treasury + belief",
    "Trigger ground quake at (x,y) — rubble, infrastructure damage",
    "Reduce target faction treasury (disease proxy)",
    "Raise all faction belief + small treasury boost",
];

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
    bridge: Option<Res<LiveBridge>>,
    mut ran_once: Local<bool>,
) {
    if !*ran_once { *ran_once = true; return; }
    if !state.visible { return; }
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let screen = ctx.content_rect();

    let mut fire: Option<String> = None;
    egui::Window::new("God Mode")
        .fixed_pos(egui::pos2(screen.max.x - 310.0, screen.center().y - 180.0))
        .fixed_size([290.0, 360.0])
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
                fire = Some(action_name.to_string());
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

    if let Some(action) = fire {
        let needs_pos = matches!(action.as_str(), "smite" | "earthquake");
        let needs_faction = matches!(action.as_str(), "bless" | "plague");
        let payload = serde_json::json!({
            "action": action,
            "x": if needs_pos { Some(state.target_x) } else { None::<f32> },
            "y": if needs_pos { Some(state.target_y) } else { None::<f32> },
            "target_faction": if needs_faction { Some(state.target_faction) } else { None::<u32> },
            "magnitude": state.magnitude,
        });
        if let Some(ref bridge) = bridge {
            bridge.client.send_rpc("sim.god_action", payload);
        }
        state.status = Some(format!("Invoked: {}", ACTIONS[state.selected_action]));
    }

    // Dispatch a substrate verb button. The `param_builder` produces the
    // matching `GodToolRequest` JSON shape; `sim.god_action` is the same
    // JSON-RPC method the legacy Invoke button uses, so this stays on
    // the exact wire path PR #762 wired up.
    if let Some(idx) = fire_substrate {
        let v = &SUBSTRATE_VERBS[idx];
        let payload = (v.param_builder)(&state);
        if let Some(ref bridge) = bridge {
            bridge.client.send_rpc("sim.god_action", payload);
        }
        state.status = Some(format!("Fired: {} ({})", v.label, v.verb));
    }
}
