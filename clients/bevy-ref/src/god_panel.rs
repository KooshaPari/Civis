#![cfg(all(feature = "bevy", feature = "egui"))]
//! God-mode intervention panel (FR-CIV-GAME-002). G key toggles.

use crate::god_actions::GodActionRequest;
use crate::live_stream::LiveBridge;
use crate::menus::in_game;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use serde_json::{json, Value};

#[derive(Resource, Default)]
pub struct GodPanelState {
    pub visible: bool,
    pub selected_action: usize,
    pub magnitude: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_faction: u32,
    pub herd_count: u32,
    pub status: Option<String>,
}

#[derive(Clone, Copy)]
struct GodActionDef {
    verb: &'static str,
    desc: &'static str,
    needs_pos: bool,
    needs_faction: bool,
    needs_count: bool,
    uses_magnitude: bool,
}

/// Live/WS `sim.god_action` catalog (legacy panel + MCP `god_verb_parity`).
const ACTIONS: &[GodActionDef] = &[
    GodActionDef {
        verb: "smite",
        desc: "Strike (x,y) with a meteor — terrain damage, fires, belief spike",
        needs_pos: true,
        needs_faction: false,
        needs_count: false,
        uses_magnitude: true,
    },
    GodActionDef {
        verb: "bless",
        desc: "Boost target faction treasury + belief",
        needs_pos: false,
        needs_faction: true,
        needs_count: false,
        uses_magnitude: true,
    },
    GodActionDef {
        verb: "earthquake",
        desc: "Trigger ground quake at (x,y) — rubble, infrastructure damage",
        needs_pos: true,
        needs_faction: false,
        needs_count: false,
        uses_magnitude: true,
    },
    GodActionDef {
        verb: "plague",
        desc: "Reduce target faction treasury (disease proxy)",
        needs_pos: false,
        needs_faction: true,
        needs_count: false,
        uses_magnitude: true,
    },
    GodActionDef {
        verb: "miracle",
        desc: "Raise all faction belief + small treasury boost",
        needs_pos: false,
        needs_faction: false,
        needs_count: false,
        uses_magnitude: true,
    },
    GodActionDef {
        verb: "life.spawn_organism",
        desc: "Spawn one agent at (x,y) for the target faction",
        needs_pos: true,
        needs_faction: true,
        needs_count: false,
        uses_magnitude: false,
    },
    GodActionDef {
        verb: "life.spawn_herd",
        desc: "Spawn N organisms with shared genome at jittered positions",
        needs_pos: false,
        needs_faction: true,
        needs_count: true,
        uses_magnitude: false,
    },
    GodActionDef {
        verb: "disaster.wildfire",
        desc: "Ignite a wildfire disaster at (x,y)",
        needs_pos: true,
        needs_faction: false,
        needs_count: false,
        uses_magnitude: false,
    },
    GodActionDef {
        verb: "disaster.flood",
        desc: "Flood terrain at (x,y) — water drop + local sea-level rise",
        needs_pos: true,
        needs_faction: false,
        needs_count: false,
        uses_magnitude: false,
    },
];

pub struct GodPanelPlugin;
impl Plugin for GodPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GodPanelState>().add_systems(
            Update,
            (toggle_god_panel, draw_god_panel).chain().run_if(in_game),
        );
    }
}

fn toggle_god_panel(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<GodPanelState>) {
    if keys.just_pressed(KeyCode::KeyG) {
        state.visible = !state.visible;
        if state.magnitude == 0.0 {
            state.magnitude = 0.5;
        }
        if state.herd_count == 0 {
            state.herd_count = 5;
        }
    }
}

fn build_god_action_payload(state: &GodPanelState, action: &GodActionDef) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("action".to_owned(), json!(action.verb));
    if action.needs_pos {
        obj.insert("x".to_owned(), json!(state.target_x));
        obj.insert("y".to_owned(), json!(state.target_y));
    }
    if action.needs_faction {
        obj.insert("target_faction".to_owned(), json!(state.target_faction));
    }
    if action.needs_count {
        obj.insert("count".to_owned(), json!(state.herd_count.max(1)));
    }
    if action.uses_magnitude {
        obj.insert("magnitude".to_owned(), json!(state.magnitude));
    }
    Value::Object(obj)
}

fn draw_god_panel(
    mut contexts: EguiContexts,
    mut state: ResMut<GodPanelState>,
    bridge: Option<Res<LiveBridge>>,
    mut requests: MessageWriter<GodActionRequest>,
    mut ran_once: Local<bool>,
) {
    if !*ran_once {
        *ran_once = true;
        return;
    }
    if !state.visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen = ctx.content_rect();

    let mut fire: Option<usize> = None;
    egui::Window::new("God Mode")
        .fixed_pos(egui::pos2(screen.max.x - 310.0, screen.center().y - 220.0))
        .fixed_size([290.0, 420.0])
        .collapsible(false)
        .title_bar(true)
        .frame(
            egui::Frame::window(ctx.style().as_ref())
                .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 230))
                .stroke(egui::Stroke::new(
                    1.5,
                    egui::Color32::from_rgb(126, 186, 181),
                )),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Direct Intervention")
                    .color(egui::Color32::from_rgb(126, 186, 181))
                    .size(11.0),
            );
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for (idx, action) in ACTIONS.iter().enumerate() {
                        let selected = state.selected_action == idx;
                        let color = if selected {
                            egui::Color32::from_rgb(126, 186, 181)
                        } else {
                            egui::Color32::from_rgb(160, 170, 180)
                        };
                        if ui
                            .add(egui::SelectableLabel::new(
                                selected,
                                egui::RichText::new(action.verb).color(color).monospace(),
                            ))
                            .clicked()
                        {
                            state.selected_action = idx;
                        }
                        if selected {
                            ui.label(
                                egui::RichText::new(action.desc)
                                    .color(egui::Color32::from_rgb(120, 130, 140))
                                    .size(10.0)
                                    .italics(),
                            );
                        }
                    }
                });
            ui.separator();

            let action = &ACTIONS[state.selected_action.min(ACTIONS.len() - 1)];

            if action.uses_magnitude {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Magnitude:")
                            .color(egui::Color32::from_rgb(160, 170, 180))
                            .size(11.0),
                    );
                    ui.add(egui::Slider::new(&mut state.magnitude, 0.0..=1.0).show_value(true));
                });
            }

            if action.needs_pos {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("X:")
                            .color(egui::Color32::from_rgb(160, 170, 180))
                            .size(11.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.target_x)
                            .speed(0.01)
                            .clamp_range(0.0..=1.0f32),
                    );
                    ui.label(
                        egui::RichText::new("Y:")
                            .color(egui::Color32::from_rgb(160, 170, 180))
                            .size(11.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.target_y)
                            .speed(0.01)
                            .clamp_range(0.0..=1.0f32),
                    );
                });
            }

            if action.needs_faction {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Faction:")
                            .color(egui::Color32::from_rgb(160, 170, 180))
                            .size(11.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.target_faction)
                            .speed(1.0)
                            .clamp_range(0..=255u32),
                    );
                });
            }

            if action.needs_count {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Count:")
                            .color(egui::Color32::from_rgb(160, 170, 180))
                            .size(11.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.herd_count)
                            .speed(1.0)
                            .clamp_range(1..=64u32),
                    );
                });
            }

            ui.separator();
            let fire_btn = ui.add_sized(
                [280.0, 28.0],
                egui::Button::new(
                    egui::RichText::new(format!("Invoke: {}", action.verb))
                        .color(egui::Color32::from_rgb(9, 10, 12))
                        .size(13.0),
                )
                .fill(egui::Color32::from_rgb(126, 186, 181)),
            );
            if fire_btn.clicked() {
                fire = Some(state.selected_action);
            }

            if let Some(ref msg) = state.status {
                ui.label(
                    egui::RichText::new(msg)
                        .color(egui::Color32::from_rgb(200, 200, 100))
                        .size(10.0),
                );
            }
        });

    if let Some(idx) = fire {
        let action = &ACTIONS[idx.min(ACTIONS.len() - 1)];
        let payload = build_god_action_payload(&state, action);
        if let Some(ref bridge) = bridge {
            bridge.client.send_rpc("sim.god_action", payload);
        }
        // Local preview on the streamed chunk cache (does not wait on the wire).
        requests.write(GodActionRequest {
            action: action.verb.to_string(),
            norm_x: state.target_x,
            norm_y: state.target_y,
            target_faction: state.target_faction,
            magnitude: if action.uses_magnitude {
                state.magnitude
            } else {
                0.5
            },
        });
        state.status = Some(format!("Invoked: {}", action.verb));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn life_spawn_organism_payload_matches_ws_catalog() {
        let state = GodPanelState {
            target_x: 0.25,
            target_y: 0.75,
            target_faction: 2,
            ..Default::default()
        };
        let action = ACTIONS
            .iter()
            .find(|a| a.verb == "life.spawn_organism")
            .unwrap();
        let payload = build_god_action_payload(&state, action);
        assert_eq!(payload["action"], "life.spawn_organism");
        assert!((payload["x"].as_f64().unwrap() - 0.25).abs() < 1e-9);
        assert!((payload["y"].as_f64().unwrap() - 0.75).abs() < 1e-9);
        assert_eq!(payload["target_faction"], 2);
    }

    #[test]
    fn life_spawn_herd_payload_matches_ws_catalog() {
        let state = GodPanelState {
            herd_count: 8,
            target_faction: 1,
            ..Default::default()
        };
        let action = ACTIONS
            .iter()
            .find(|a| a.verb == "life.spawn_herd")
            .unwrap();
        let payload = build_god_action_payload(&state, action);
        assert_eq!(payload["action"], "life.spawn_herd");
        assert_eq!(payload["count"], 8);
        assert_eq!(payload["target_faction"], 1);
        assert!(payload.get("x").is_none());
    }

    #[test]
    fn disaster_verbs_include_normalized_position() {
        let state = GodPanelState {
            target_x: 0.5,
            target_y: 0.5,
            ..Default::default()
        };
        for verb in ["disaster.wildfire", "disaster.flood"] {
            let action = ACTIONS.iter().find(|a| a.verb == verb).unwrap();
            let payload = build_god_action_payload(&state, action);
            assert_eq!(payload["action"], verb);
            assert_eq!(payload["x"], 0.5);
            assert_eq!(payload["y"], 0.5);
        }
    }

    #[test]
    fn fire_builds_preview_request_fields() {
        let state = GodPanelState {
            target_x: 0.4,
            target_y: 0.6,
            target_faction: 3,
            magnitude: 0.8,
            ..Default::default()
        };
        let action = ACTIONS.iter().find(|a| a.verb == "smite").unwrap();
        let req = GodActionRequest {
            action: action.verb.to_string(),
            norm_x: state.target_x,
            norm_y: state.target_y,
            target_faction: state.target_faction,
            magnitude: if action.uses_magnitude {
                state.magnitude
            } else {
                0.5
            },
        };
        assert_eq!(req.action, "smite");
        assert!((req.norm_x - 0.4).abs() < 1e-6);
        assert!((req.norm_y - 0.6).abs() < 1e-6);
        assert_eq!(req.target_faction, 3);
        assert!((req.magnitude - 0.8).abs() < 1e-6);
    }
}
