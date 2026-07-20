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
use bevy_egui::egui;
use civ_holocron::descriptor::VerbDescriptor;
use civ_holocron::group::VerbGroup;
use civ_holocron::registry::VerbRegistry;
use civ_holocron::risk::RiskTier;
use crate::live_stream::LiveBridge;
use serde_json::json;

use crate::god_actions::GodActionRequest;
use crate::god_panel::GodPanelState;
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

// ---------------------------------------------------------------------------
// egui overlay — Command‑K style
// ---------------------------------------------------------------------------

/// Draw the Command‑K overlay as an egui `Window` centered on screen.
fn draw_holocron_overlay(
    mut state: ResMut<HolocronState>,
    mut contexts: bevy_egui::EguiContexts,
    bridge: Option<Res<LiveBridge>>,
) {
    if !state.overlay_visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let filtered: Vec<VerbDescriptor> = matched_verbs(&state.registry, &state.filter)
        .into_iter()
        .cloned()
        .collect();
    // Clamp cursor.
    if !filtered.is_empty() && state.cursor >= filtered.len() {
        state.cursor = 0;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::from_black_alpha(200)))
        .show(ctx, |ui| {
            // Centre a fixed‑size region vertically and horizontally.
            let avail = ui.available_size();
            let panel_w = (avail.x * 0.55).min(640.0).max(300.0);
            let panel_h = (avail.y * 0.60).min(480.0).max(200.0);
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(panel_w, panel_h),
                egui::Sense::hover(),
            );
            //  Center the alloc rect
            let base = ui.min_rect().min;
            let dx = (avail.x - panel_w) * 0.5 - base.x;
            let dy = (avail.y - panel_h) * 0.4 - base.y;
            egui::Area::new(egui::Id::new("cmdk-overlay"))
                .fixed_pos(egui::pos2(rect.min.x + dx, rect.min.y + dy))
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(22, 22, 30))
                        .show(ui, |ui| {
                        ui.set_max_width(panel_w);
                        ui.set_max_height(panel_h);

                        // ── Search bar ──
                        let search_resp = ui.add(
                            egui::TextEdit::singleline(&mut state.filter)
                                .hint_text("Search verbs… (fire with Enter, Esc to dismiss)")
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Heading),
                        );
                        search_resp.request_focus();

                        // ── Results ──
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

                                    egui::Frame::none()
                                        .fill(bg)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Risk badge
                                                let badge = risk_label(verb.risk);
                                                let badge_col = match verb.risk {
                                                    RiskTier::ReadOnly | RiskTier::Cosmetic => {
                                                        egui::Color32::from_rgb(80, 160, 80)
                                                    }
                                                    RiskTier::Minor | RiskTier::Reversible => {
                                                        egui::Color32::from_rgb(200, 160, 40)
                                                    }
                                                    RiskTier::Major | RiskTier::Critical | RiskTier::Irreversible => {
                                                        egui::Color32::from_rgb(200, 60, 60)
                                                    }
                                                };
                                                ui.colored_label(badge_col, badge);
                                                ui.label(
                                                    egui::RichText::new(&verb.name)
                                                        .text_style(egui::TextStyle::Body),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        // Hotkey hint
                                                        if let Some(hk) = &verb.hotkey {
                                                            ui.label(
                                                                egui::RichText::new(format!("[{}]", hk))
                                                                    .color(egui::Color32::GRAY)
                                                                    .text_style(
                                                                        egui::TextStyle::Monospace,
                                                                    ),
                                                            );
                                                        }
                                                        // Provenance badge
                                                        let prov = verb.provenance.label();
                                                        ui.colored_label(
                                                            egui::Color32::GRAY,
                                                            prov,
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                }
                            });
                        });
                });
        });

    let escape_pressed = ctx.input(|input| input.key_pressed(egui::Key::Escape));
    let enter_pressed = ctx.input(|input| input.key_pressed(egui::Key::Enter));
    let arrow_down_pressed = ctx.input(|input| input.key_pressed(egui::Key::ArrowDown));
    let arrow_up_pressed = ctx.input(|input| input.key_pressed(egui::Key::ArrowUp));

    // Handle keyboard — Enter fires the selected verb, Esc dismisses.
    if escape_pressed {
        state.overlay_visible = false;
        state.filter.clear();
        state.cursor = 0;
    }
    // Enter fires the selected verb.
    if enter_pressed && !filtered.is_empty() {
        let verb = &filtered[state.cursor.min(filtered.len() - 1)];
        if let Some(bridge) = bridge.as_ref() {
            bridge
                .client
                .send_rpc("sim.god_action", json!({ "action": verb.id }));
        }
        info!("Holocron fire: {} (id={})", verb.name, verb.id);
        state.overlay_visible = false;
        state.filter.clear();
        state.cursor = 0;
    }
    // Keyboard navigation.
    if arrow_down_pressed && !filtered.is_empty() {
        state.cursor = (state.cursor + 1) % filtered.len();
    }
    if arrow_up_pressed && !filtered.is_empty() {
        state.cursor = (state.cursor + filtered.len() - 1) % filtered.len();
    }
}

// ---------------------------------------------------------------------------
// Public toggle helper — called from game_ui.rs keyboard handler
// ---------------------------------------------------------------------------

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
