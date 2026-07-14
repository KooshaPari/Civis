#![cfg(all(feature = "bevy", feature = "egui"))]

//! Full-screen game-over / victory overlay (FR-CIV-GAME-001).
//!
//! Polls `sim.outcome` every 30 s via the WsClient background thread.
//! On a non-Ongoing result renders a modal overlay with the outcome tag,
//! reason, and a [New Game] button that sends `sim.reset`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_attach::LiveAttachBridge;
use crate::menus::{toggle_pause, AppState};
use crate::OutcomeProgressHud;

/// Gates when terminal outcomes may surface the modal (Paradox-style shell).
#[derive(Resource, Debug)]
pub struct OutcomeSessionGate {
    /// False until the player starts a session from the main menu.
    pub session_active: bool,
    /// Tick of the first `ongoing` poll after session start.
    pub first_poll_tick: Option<u64>,
    /// Ticks after first ongoing poll before terminal outcomes may show.
    pub grace_ticks: u64,
}

impl Default for OutcomeSessionGate {
    fn default() -> Self {
        Self {
            session_active: false,
            first_poll_tick: None,
            grace_ticks: 120,
        }
    }
}

/// Bevy resource caching the last non-Ongoing outcome received.
#[derive(Resource, Debug, Default)]
pub struct OutcomeOverlayState {
    pub outcome: Option<crate::OutcomeHudData>,
    pub dismissed: bool,
}

/// One-frame flag: escape dismissed the outcome modal this frame (blocks pause toggle).
#[derive(Resource, Default)]
pub(crate) struct OutcomeEscapeBlock(pub bool);

/// Begin an interactive session from the main menu (clears stale server outcomes).
pub fn begin_player_session(gate: &mut OutcomeSessionGate, overlay: &mut OutcomeOverlayState) {
    gate.session_active = true;
    gate.first_poll_tick = None;
    overlay.outcome = None;
    overlay.dismissed = false;
}

/// End the session when returning to the main menu.
pub fn end_player_session(gate: &mut OutcomeSessionGate, overlay: &mut OutcomeOverlayState) {
    gate.session_active = false;
    gate.first_poll_tick = None;
    overlay.outcome = None;
    overlay.dismissed = false;
}

/// True when the full-screen outcome modal should be shown.
#[must_use]
pub fn outcome_modal_visible(state: &OutcomeOverlayState) -> bool {
    !state.dismissed && state.outcome.is_some()
}

/// Apply a polled `sim.outcome` payload to overlay + progress HUD state.
pub fn apply_outcome_poll(
    gate: &mut OutcomeSessionGate,
    state: &mut OutcomeOverlayState,
    progress: &mut OutcomeProgressHud,
    data: crate::OutcomeHudData,
) {
    if data.tag == "ongoing" {
        if let Some(snapshot) = data.progress {
            progress.0 = Some(snapshot);
        }
        if gate.session_active && gate.first_poll_tick.is_none() {
            gate.first_poll_tick = Some(data.tick);
        }
        return;
    }

    if let Some(snapshot) = data.progress {
        progress.0 = Some(snapshot);
    }

    let grace_met = gate
        .first_poll_tick
        .map(|first| data.tick.saturating_sub(first) >= gate.grace_ticks)
        .unwrap_or(false);

    if !gate.session_active || !grace_met {
        return;
    }

    if state
        .outcome
        .as_ref()
        .map(|o| o.tag != data.tag)
        .unwrap_or(true)
    {
        state.dismissed = false;
    }
    state.outcome = Some(data);
}

pub struct OutcomeOverlayPlugin;

impl Plugin for OutcomeOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutcomeSessionGate>()
            .init_resource::<OutcomeOverlayState>()
            .init_resource::<OutcomeProgressHud>()
            .init_resource::<OutcomeEscapeBlock>()
            .add_systems(
                Update,
                (
                    clear_outcome_escape_block,
                    dismiss_outcome_overlay_on_escape,
                )
                    .chain()
                    .before(toggle_pause),
            )
            .add_systems(Update, poll_outcome_system)
            .add_systems(EguiPrimaryContextPass, draw_outcome_overlay);
    }
}

fn clear_outcome_escape_block(mut block: ResMut<OutcomeEscapeBlock>) {
    block.0 = false;
}

fn dismiss_outcome_overlay_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<OutcomeOverlayState>,
    mut block: ResMut<OutcomeEscapeBlock>,
) {
    if keys.just_pressed(KeyCode::Escape) && outcome_modal_visible(&state) {
        state.dismissed = true;
        block.0 = true;
    }
}

fn poll_outcome_system(
    bridge: Res<LiveAttachBridge>,
    mut gate: ResMut<OutcomeSessionGate>,
    mut state: ResMut<OutcomeOverlayState>,
    mut progress: ResMut<OutcomeProgressHud>,
) {
    if let Some(data) = bridge.client.poll_outcome() {
        apply_outcome_poll(&mut gate, &mut state, &mut progress, data);
    }
}

fn draw_outcome_overlay(
    mut contexts: EguiContexts,
    gate: Res<OutcomeSessionGate>,
    mut state: ResMut<OutcomeOverlayState>,
    bridge: Res<LiveAttachBridge>,
    app_state: Option<Res<State<AppState>>>,
) {
    if !gate.session_active {
        return;
    }
    if let Some(app_state) = app_state {
        if *app_state.get() != AppState::Playing && *app_state.get() != AppState::Paused {
            return;
        }
    }
    let Some(ref outcome) = state.outcome.clone() else {
        return;
    };
    if state.dismissed {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let is_victory = outcome.tag == "victory";
    let header_color = if is_victory {
        egui::Color32::from_rgb(0x7e, 0xba, 0xb5) // teal
    } else {
        egui::Color32::from_rgb(0xe0, 0x5c, 0x5c) // red
    };

    egui::Area::new(egui::Id::new("outcome_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let screen = ctx.content_rect();
            ui.allocate_ui_with_layout(
                screen.size(),
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    // dim backdrop
                    ui.painter().rect_filled(
                        screen,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(9, 10, 12, 210),
                    );

                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgba_unmultiplied(9, 10, 12, 240))
                        .stroke(egui::Stroke::new(1.5, header_color))
                        .inner_margin(egui::Margin::same(40))
                        .corner_radius(egui::CornerRadius::same(8))
                        .show(ui, |ui| {
                            ui.set_max_width(500.0);
                            ui.spacing_mut().item_spacing.y = 16.0;

                            let label = if is_victory { "VICTORY" } else { "DEFEAT" };
                            ui.colored_label(
                                header_color,
                                egui::RichText::new(label).size(36.0).strong(),
                            );
                            ui.colored_label(
                                egui::Color32::WHITE,
                                egui::RichText::new(&outcome.reason).size(20.0),
                            );
                            ui.colored_label(egui::Color32::GRAY, format!("Tick {}", outcome.tick));

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .button(egui::RichText::new("New Game").size(16.0))
                                    .clicked()
                                {
                                    bridge
                                        .client
                                        .send_rpc("sim.reset", serde_json::json!({"seed": 0}));
                                    state.dismissed = true;
                                    state.outcome = None;
                                }
                                if ui
                                    .button(egui::RichText::new("Dismiss").size(16.0))
                                    .clicked()
                                {
                                    state.dismissed = true;
                                }
                            });
                        });
                },
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutcomeProgressHudData;

    fn sample_progress() -> OutcomeProgressHudData {
        OutcomeProgressHudData {
            population: 4_321,
            population_target: 10_000,
            researched_techs: 3,
            researched_techs_target: 12,
            peace_ticks: 50,
            peace_ticks_target: 500,
        }
    }

    fn active_gate_with_grace_met() -> OutcomeSessionGate {
        OutcomeSessionGate {
            session_active: true,
            first_poll_tick: Some(1),
            grace_ticks: 120,
        }
    }

    #[test]
    fn apply_outcome_poll_ignores_ongoing_without_overlay() {
        let mut gate = OutcomeSessionGate::default();
        let mut state = OutcomeOverlayState::default();
        let mut progress = OutcomeProgressHud::default();
        apply_outcome_poll(
            &mut gate,
            &mut state,
            &mut progress,
            crate::OutcomeHudData {
                tag: "ongoing".to_string(),
                reason: String::new(),
                tick: 9,
                progress: Some(sample_progress()),
            },
        );
        assert!(state.outcome.is_none());
        assert!(!state.dismissed);
        assert_eq!(progress.0, Some(sample_progress()));
        assert!(gate.first_poll_tick.is_none());
    }

    #[test]
    fn apply_outcome_poll_sets_first_poll_tick_when_session_active() {
        let mut gate = OutcomeSessionGate {
            session_active: true,
            ..Default::default()
        };
        let mut state = OutcomeOverlayState::default();
        let mut progress = OutcomeProgressHud::default();
        apply_outcome_poll(
            &mut gate,
            &mut state,
            &mut progress,
            crate::OutcomeHudData {
                tag: "ongoing".to_string(),
                reason: String::new(),
                tick: 42,
                progress: None,
            },
        );
        assert_eq!(gate.first_poll_tick, Some(42));
        assert!(state.outcome.is_none());
    }

    #[test]
    fn apply_outcome_poll_blocks_terminal_before_grace() {
        let mut gate = OutcomeSessionGate {
            session_active: true,
            first_poll_tick: Some(100),
            grace_ticks: 120,
        };
        let mut state = OutcomeOverlayState::default();
        let mut progress = OutcomeProgressHud::default();
        apply_outcome_poll(
            &mut gate,
            &mut state,
            &mut progress,
            crate::OutcomeHudData {
                tag: "victory".to_string(),
                reason: "too soon".to_string(),
                tick: 150,
                progress: Some(sample_progress()),
            },
        );
        assert!(state.outcome.is_none());
        assert_eq!(progress.0, Some(sample_progress()));
    }

    #[test]
    fn apply_outcome_poll_surfaces_victory_and_resets_dismissed() {
        let mut gate = active_gate_with_grace_met();
        let mut state = OutcomeOverlayState {
            outcome: Some(crate::OutcomeHudData {
                tag: "defeat".to_string(),
                reason: "lost".to_string(),
                tick: 1,
                progress: None,
            }),
            dismissed: true,
        };
        let mut progress = OutcomeProgressHud::default();
        apply_outcome_poll(
            &mut gate,
            &mut state,
            &mut progress,
            crate::OutcomeHudData {
                tag: "victory".to_string(),
                reason: "population".to_string(),
                tick: 200,
                progress: Some(sample_progress()),
            },
        );
        assert!(!state.dismissed);
        let outcome = state.outcome.expect("victory outcome");
        assert_eq!(outcome.tag, "victory");
        assert_eq!(outcome.reason, "population");
        assert_eq!(progress.0, Some(sample_progress()));
    }

    #[test]
    fn apply_outcome_poll_keeps_dismissed_for_same_tag() {
        let mut gate = active_gate_with_grace_met();
        let mut state = OutcomeOverlayState {
            outcome: Some(crate::OutcomeHudData {
                tag: "defeat".to_string(),
                reason: "first".to_string(),
                tick: 1,
                progress: None,
            }),
            dismissed: true,
        };
        let mut progress = OutcomeProgressHud::default();
        apply_outcome_poll(
            &mut gate,
            &mut state,
            &mut progress,
            crate::OutcomeHudData {
                tag: "defeat".to_string(),
                reason: "second".to_string(),
                tick: 200,
                progress: None,
            },
        );
        assert!(state.dismissed);
        assert_eq!(state.outcome.as_ref().expect("outcome").reason, "second");
    }
}
