#![cfg(all(feature = "bevy", feature = "egui"))]

//! 6-step tutorial hint system (FR-CIV-CLIENT-011).
//! Shown bottom-centre during InGame. Space/click advances; H replays.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::menus::in_game;

/// Tutorial visibility states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialVisibility {
    /// Always show at start of new game (first-run detection).
    Auto,
    /// Manually toggled with [H].
    Manual,
    /// Hidden (completed or dismissed).
    Hidden,
}

const HINTS: &[&str] = &[
    "Welcome to Civis! Your civilization is emerging. Watch the minimap for faction spread. [M] cycles map modes.",
    "Press [F] to see your faction's stats - population, treasury, and government type.",
    "Events appear in the feed [N]. Disasters and diplomacy shape your world.",
    "Use [T] to research technologies. Each unlock accelerates your civilization.",
    "Open [D] to manage diplomacy - propose treaties or declare war.",
    "Press [?] anytime for all controls. Good luck!",
];

/// State-dependent filter: which step index to show based on sim progress.
const STATE_GATES: &[(u8, &str)] = &[
    (0, ""),                              // step 0: always
    (1, "faction"),                        // step 1: once a faction exists
    (5, "technology"),                     // step 5: once tech is unlocked
];

/// Persistent tutorial state saved to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialSaveData {
    /// Has the user ever completed the tutorial?
    pub completed_once: bool,
}

impl Default for TutorialSaveData {
    fn default() -> Self {
        Self { completed_once: false }
    }
}

#[derive(Resource)]
pub struct TutorialState {
    pub visibility: TutorialVisibility,
    pub step: u8,
    pub acknowledged: bool,
    /// Cached save-data from user_data.
    pub save_data: TutorialSaveData,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            visibility: TutorialVisibility::Auto,
            step: 0,
            acknowledged: false,
            save_data: TutorialSaveData::default(),
        }
    }
}

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .add_systems(Update, (handle_tutorial_keys, draw_tutorial_hint).chain().run_if(in_game));
    }
}

fn handle_tutorial_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TutorialState>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        state.visibility = TutorialVisibility::Manual;
        state.step = 0;
        state.acknowledged = false;
        return;
    }
    if state.visibility == TutorialVisibility::Hidden { return; }
    if keys.just_pressed(KeyCode::Space) {
        advance(&mut state);
    }
}

fn advance(state: &mut TutorialState) {
    if state.step as usize + 1 >= HINTS.len() {
        state.visibility = TutorialVisibility::Hidden;
        state.save_data.completed_once = true;
    } else {
        // Skip ahead to next step whose gate is open.
        let mut next = state.step + 1;
        while (next as usize) < HINTS.len() && !is_gate_open(next) {
            next += 1;
        }
        state.step = next.min((HINTS.len() - 1) as u8);
        state.acknowledged = false;
    }
}

/// Returns true when the user has progressed far enough to see this step.
fn is_gate_open(step: u8) -> bool {
    // For now, all gates are open (no sim-state check in this stub).
    // Future: check emergence_metrics, faction count, tech tree.
    let _ = step;
    true
}

/// Returns true if the tutorial should show this frame.
fn should_show(state: &TutorialState) -> bool {
    match state.visibility {
        TutorialVisibility::Auto => !state.save_data.completed_once,
        TutorialVisibility::Manual => true,
        TutorialVisibility::Hidden => false,
    }
}

fn draw_tutorial_hint(
    mut contexts: EguiContexts,
    mut state: ResMut<TutorialState>,
) {
    if !should_show(&state) { return; }

    let idx = (state.step as usize).min(HINTS.len() - 1);
    let hint = HINTS[idx];
    let step = state.step;
    let total = HINTS.len() as u8;

    let ctx = contexts.ctx_mut();
    let screen = ctx.screen_rect();

    let mut clicked = false;
    egui::Area::new(egui::Id::new("tutorial_hint"))
        .fixed_pos(egui::pos2(screen.center().x - 280.0, screen.max.y - 110.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 230))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(126, 186, 181)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
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
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if step + 1 >= total { "Got it" } else { "Next" };
                            if ui.small_button(label).clicked() {
                                clicked = true;
                            }
                            ui.label(
                                egui::RichText::new("Space or click to advance")
                                    .color(egui::Color32::from_rgb(100, 110, 120))
                                    .size(10.0),
                            );
                        });
                    });
                });
        });

    if clicked {
        advance(&mut state);
    }
}