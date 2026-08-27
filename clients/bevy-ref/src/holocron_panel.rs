#![cfg(all(feature = "bevy", feature = "egui"))]
//! Holocron Command‑K overlay — discoverability launcher over live god verbs.
//!
//! The standalone `crates/holocron` catalog is not yet a workspace package
//! (missing `Cargo.toml` / modules). This panel uses the same `sim.god_action`
//! verb ids as [`crate::god_panel`] and fires [`GodActionRequest`] on Enter.
//!
//! Toggle: **Ctrl+K** (plain `K` stays audio mute). Esc dismisses.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::god_actions::GodActionRequest;
use crate::god_panel::GodPanelState;
use crate::live_stream::ServerBridge;
use crate::menus::in_game;

/// One searchable god verb surfaced in the overlay.
#[derive(Clone, Copy, Debug)]
struct HolocronVerb {
    id: &'static str,
    label: &'static str,
    hint: &'static str,
    needs_pos: bool,
    needs_faction: bool,
}

/// Live/WS `sim.god_action` catalog (parity with god panel).
const VERBS: &[HolocronVerb] = &[
    HolocronVerb {
        id: "smite",
        label: "Smite",
        hint: "Meteor strike at crosshair",
        needs_pos: true,
        needs_faction: false,
    },
    HolocronVerb {
        id: "bless",
        label: "Bless",
        hint: "Boost target faction",
        needs_pos: false,
        needs_faction: true,
    },
    HolocronVerb {
        id: "earthquake",
        label: "Earthquake",
        hint: "Quake at crosshair",
        needs_pos: true,
        needs_faction: false,
    },
    HolocronVerb {
        id: "plague",
        label: "Plague",
        hint: "Hit target faction treasury",
        needs_pos: false,
        needs_faction: true,
    },
    HolocronVerb {
        id: "miracle",
        label: "Miracle",
        hint: "Belief + treasury bump",
        needs_pos: false,
        needs_faction: false,
    },
    HolocronVerb {
        id: "life.spawn_organism",
        label: "Spawn Organism",
        hint: "One agent at crosshair",
        needs_pos: true,
        needs_faction: true,
    },
    HolocronVerb {
        id: "life.spawn_herd",
        label: "Spawn Herd",
        hint: "N organisms for faction",
        needs_pos: false,
        needs_faction: true,
    },
    HolocronVerb {
        id: "disaster.wildfire",
        label: "Wildfire",
        hint: "Ignite at crosshair",
        needs_pos: true,
        needs_faction: false,
    },
    HolocronVerb {
        id: "disaster.flood",
        label: "Flood",
        hint: "Flood at crosshair",
        needs_pos: true,
        needs_faction: false,
    },
];

/// Per‑frame Command‑K overlay state.
#[derive(Resource)]
pub struct HolocronState {
    pub overlay_visible: bool,
    pub filter: String,
    pub cursor: usize,
}

impl Default for HolocronState {
    fn default() -> Self {
        Self {
            overlay_visible: false,
            filter: String::new(),
            cursor: 0,
        }
    }
}

pub struct HolocronPanelPlugin;

impl Plugin for HolocronPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HolocronState>()
            .add_systems(Update, toggle_cmdk.run_if(in_game))
            .add_systems(
                EguiPrimaryContextPass,
                draw_holocron_overlay.run_if(in_game),
            );
    }
}

fn fuzzy_match(query: &str, candidate: &str) -> bool {
    let query_lower = query.to_lowercase();
    let q_words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();
    if q_words.is_empty() {
        return true;
    }
    let c_lower = candidate.to_lowercase();
    q_words.iter().all(|qw| c_lower.contains(qw))
}

fn matched_verbs(filter: &str) -> Vec<&'static HolocronVerb> {
    VERBS
        .iter()
        .filter(|v| {
            fuzzy_match(filter, v.id) || fuzzy_match(filter, v.label) || fuzzy_match(filter, v.hint)
        })
        .collect()
}

fn toggle_cmdk(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<HolocronState>) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyK) {
        toggle_cmdk_overlay(&mut state);
    }
}

/// Toggle the Command‑K overlay on/off.
pub fn toggle_cmdk_overlay(state: &mut HolocronState) {
    state.overlay_visible = !state.overlay_visible;
    if state.overlay_visible {
        state.filter.clear();
        state.cursor = 0;
    }
}

/// Returns `true` when the overlay is currently drawing (consumes kb focus).
pub fn overlay_active(state: &HolocronState) -> bool {
    state.overlay_visible
}

fn fire_verb(
    verb: &HolocronVerb,
    panel: &GodPanelState,
    requests: &mut MessageWriter<GodActionRequest>,
) {
    let norm_x = if verb.needs_pos { panel.target_x } else { 0.5 };
    let norm_y = if verb.needs_pos { panel.target_y } else { 0.5 };
    let target_faction = if verb.needs_faction {
        panel.target_faction
    } else {
        0
    };
    requests.write(GodActionRequest {
        action: verb.id.to_string(),
        norm_x,
        norm_y,
        target_faction,
        magnitude: panel.magnitude.clamp(0.05, 1.0),
    });
    info!("Holocron fire: {} → GodActionRequest", verb.id);
}

fn draw_holocron_overlay(
    mut contexts: EguiContexts,
    mut state: ResMut<HolocronState>,
    panel: Option<Res<GodPanelState>>,
    mut requests: MessageWriter<GodActionRequest>,
    bridge: Option<Res<ServerBridge>>,
) {
    if !state.overlay_visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let filtered = matched_verbs(&state.filter);
    if !filtered.is_empty() && state.cursor >= filtered.len() {
        state.cursor = 0;
    }

    // Server: fetch civilization history (sim.legends) when overlay is visible
    if let Some(ref bridge) = bridge {
        bridge.send_rpc("sim.legends", serde_json::json!({}));
    }

    let mut dismiss = false;
    let mut fire = false;
    let mut cursor_delta: i32 = 0;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_black_alpha(200)))
        .show(ctx, |ui| {
            let avail = ui.available_size();
            let panel_w = (avail.x * 0.55).min(640.0).max(300.0);
            let panel_h = (avail.y * 0.60).min(480.0).max(200.0);

            egui::Area::new(egui::Id::new("cmdk-overlay"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -40.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(22, 22, 30))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.set_min_width(panel_w);
                            ui.set_max_width(panel_w);
                            ui.set_max_height(panel_h);

                            let search_resp = ui.add(
                                egui::TextEdit::singleline(&mut state.filter)
                                    .hint_text("Search verbs… (Enter fire, Esc dismiss)")
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Heading),
                            );
                            search_resp.request_focus();

                            ui.separator();
                            egui::ScrollArea::vertical()
                                .max_height(panel_h - 60.0)
                                .show(ui, |ui| {
                                    if filtered.is_empty() {
                                        ui.label(
                                            egui::RichText::new("No verbs match the filter.")
                                                .color(egui::Color32::GRAY),
                                        );
                                        return;
                                    }
                                    for (i, verb) in filtered.iter().enumerate() {
                                        let selected = i == state.cursor;
                                        let bg = if selected {
                                            egui::Color32::from_rgb(40, 60, 120)
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };
                                        let row = egui::Frame::NONE.fill(bg).show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(verb.label).strong().color(
                                                        egui::Color32::from_rgb(126, 186, 181),
                                                    ),
                                                );
                                                ui.label(
                                                    egui::RichText::new(verb.id)
                                                        .monospace()
                                                        .color(egui::Color32::GRAY),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(
                                                            egui::RichText::new(verb.hint)
                                                                .color(egui::Color32::DARK_GRAY),
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                        if row.response.clicked() {
                                            state.cursor = i;
                                            fire = true;
                                        }
                                    }
                                });
                        });
                });
        });

    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            dismiss = true;
        }
        if i.key_pressed(egui::Key::Enter) {
            fire = true;
        }
        if i.key_pressed(egui::Key::ArrowDown) {
            cursor_delta = 1;
        }
        if i.key_pressed(egui::Key::ArrowUp) {
            cursor_delta = -1;
        }
    });

    if cursor_delta != 0 && !filtered.is_empty() {
        let n = filtered.len() as i32;
        state.cursor = ((state.cursor as i32 + cursor_delta).rem_euclid(n)) as usize;
    }

    if fire && !filtered.is_empty() {
        let verb = filtered[state.cursor.min(filtered.len() - 1)];
        let fallback = GodPanelState {
            magnitude: 0.5,
            target_x: 0.5,
            target_y: 0.5,
            ..Default::default()
        };
        let effective = panel.as_deref().unwrap_or(&fallback);
        fire_verb(verb, effective, &mut requests);
        dismiss = true;
    }

    if dismiss {
        state.overlay_visible = false;
        state.filter.clear();
        state.cursor = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_finds_smite() {
        let hits = matched_verbs("smi");
        assert!(hits.iter().any(|v| v.id == "smite"));
    }

    #[test]
    fn empty_filter_lists_all() {
        assert_eq!(matched_verbs("").len(), VERBS.len());
    }
}
