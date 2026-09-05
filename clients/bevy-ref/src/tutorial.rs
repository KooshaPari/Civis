#![cfg(all(feature = "bevy", feature = "egui"))]

//! 6-step tutorial hint system (FR-CIV-CLIENT-011).
//! Shown bottom-centre during InGame. Enter/click advances; H replays.
//!
//! **Intentionally local-only** — this panel displays static tutorial
//! content and does not communicate with the server via JSON-RPC.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use serde::{Deserialize, Serialize};

use crate::live_stream::ServerBridge;
use crate::menus::in_playing_state;
use crate::settings_ui::GameSettings;

/// Live simulation status snapshot received from the server.
#[derive(Resource, Default, Clone, Debug)]
pub struct SimStatusSnapshot {
    /// Current server tick.
    pub tick: u64,
    /// Human-readable era name.
    pub era: String,
    /// Total civilian population.
    pub population: u64,
    /// Whether the sim is paused server-side.
    pub paused: bool,
}

const HINTS: &[&str] = &[
    "Welcome to Civis! Your civilization is emerging. Watch the minimap for faction spread. [M] cycles map modes.",
    "Press [F1] to see your faction's stats - population, treasury, and government type.",
    "Events appear in the feed [N]. Disasters and diplomacy shape your world.",
    "Use [T] to research technologies. Each unlock accelerates your civilization.",
    "Open [D] to manage diplomacy - propose treaties or declare war.",
    "Press [?] anytime for all controls. Good luck!",
];

#[derive(Resource, Serialize, Deserialize)]
pub struct TutorialState {
    pub enabled: bool,
    pub step: u8,
    pub acknowledged: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            enabled: true,
            step: 0,
            acknowledged: false,
        }
    }
}

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .init_resource::<SimStatusSnapshot>()
            .add_systems(Startup, apply_persisted_tutorial_skip)
            .add_systems(
                Update,
                (handle_tutorial_keys, draw_tutorial_hint)
                    .chain()
                    .run_if(in_playing_state),
            );
    }
}

/// On startup, honour the persisted "skipped tutorial" preference so the
/// hint is not re-shown on every launch once the player dismisses it.
fn apply_persisted_tutorial_skip(
    mut state: ResMut<TutorialState>,
    settings: Res<GameSettings>,
) {
    if settings.tutorial_skipped {
        state.enabled = false;
        state.step = 0;
        state.acknowledged = true;
    }
}

fn handle_tutorial_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TutorialState>,
    mut settings: ResMut<GameSettings>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        state.enabled = true;
        state.step = 0;
        state.acknowledged = false;
        return;
    }
    if !state.enabled {
        return;
    }
    // Escape skips the tutorial and remembers that choice across launches.
    if keys.just_pressed(KeyCode::Escape) {
        state.enabled = false;
        state.acknowledged = true;
        settings.tutorial_skipped = true;
        settings.save();
        return;
    }
    // Enter — Space owns pause/resume in the shell.
    if keys.just_pressed(KeyCode::Enter) {
        if advance(&mut state) && !settings.tutorial_skipped {
            settings.tutorial_skipped = true;
            settings.save();
        }
    }
}

/// Returns true when the tutorial reached completion AND was disabled.
fn advance(state: &mut TutorialState) -> bool {
    if state.step as usize + 1 >= HINTS.len() {
        state.enabled = false;
        true
    } else {
        state.step += 1;
        state.acknowledged = false;
        false
    }
}

fn should_show(state: &TutorialState) -> bool {
    state.enabled
}

fn draw_tutorial_hint(
    mut contexts: EguiContexts,
    mut state: ResMut<TutorialState>,
    mut ran_once: Local<bool>,
    bridge: Option<Res<ServerBridge>>,
    mut sim_status: ResMut<SimStatusSnapshot>,
    mut sent_subscribe: Local<bool>,
    mut settings: ResMut<GameSettings>,
) {
    // egui panics if ctx rect/fonts are accessed before its first run; skip frame 1.
    if !*ran_once {
        *ran_once = true;
        return;
    }
    // Server: subscribe to sim.status for live tutorial context
    if let Some(ref bridge) = bridge {
        if !*sent_subscribe {
            bridge.send_rpc("sim.status", serde_json::json!({}));
            *sent_subscribe = true;
        }
    }
    if !should_show(&state) {
        return;
    }

    let hint = HINTS[state.step as usize];
    let step = state.step;
    let total = HINTS.len() as u8;

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut clicked = false;
    // Anchor to true screen centre, well above the bottom toolbar (which
    // lives at CENTER_BOTTOM with offset -12). Width 560 is centred via
    // 0.5px × width offset so the popup is always pixel-perfectly centred.
    egui::Area::new(egui::Id::new("tutorial_hint"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 230))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(126, 186, 181),
                ))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(16, 10))
                .show(ui, |ui| {
                    ui.set_width(560.0);
                    ui.label(
                        egui::RichText::new(hint)
                            .color(egui::Color32::from_rgb(220, 230, 230))
                            .size(13.0),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}/{}", step + 1, total))
                                .color(egui::Color32::from_rgb(126, 186, 181))
                                .size(11.0),
                        );
                        // Live sim context from server
                        if !sim_status.era.is_empty() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "tick {} | pop {} | {}{}",
                                    sim_status.tick,
                                    sim_status.population,
                                    sim_status.era,
                                    if sim_status.paused { " (paused)" } else { "" },
                                ))
                                .color(egui::Color32::from_rgb(100, 110, 120))
                                .size(10.0),
                            );
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if step + 1 >= total { "Got it" } else { "Next" };
                            if ui.small_button(label).clicked() {
                                clicked = true;
                            }
                            ui.label(
                                egui::RichText::new("Enter or click to advance")
                                    .color(egui::Color32::from_rgb(100, 110, 120))
                                    .size(10.0),
                            );
                        });
                    });
                });
        });

    if clicked {
        if advance(&mut state) && !settings.tutorial_skipped {
            settings.tutorial_skipped = true;
            settings.save();
        }
    }
}
