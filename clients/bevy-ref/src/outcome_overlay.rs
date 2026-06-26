#![cfg(all(feature = "bevy", feature = "egui"))]

//! Full-screen game-over / victory overlay (FR-CIV-GAME-001).
//!
//! Polls `sim.outcome` every 30 s via the WsClient background thread.
//! On a non-Ongoing result renders a modal overlay with the outcome tag,
//! reason, and a [New Game] button that sends `sim.reset`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_attach::LiveAttachBridge;

/// Bevy resource caching the last non-Ongoing outcome received.
#[derive(Resource, Debug, Default)]
pub struct OutcomeOverlayState {
    pub outcome: Option<crate::OutcomeHudData>,
    pub dismissed: bool,
}

pub struct OutcomeOverlayPlugin;

impl Plugin for OutcomeOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutcomeOverlayState>()
            .add_systems(Update, poll_outcome_system)
            .add_systems(EguiPrimaryContextPass, draw_outcome_overlay);
    }
}

fn poll_outcome_system(
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<OutcomeOverlayState>,
) {
    if let Some(data) = bridge.client.poll_outcome() {
        if data.tag != "ongoing" {
            if state.outcome.as_ref().map(|o| o.tag != data.tag).unwrap_or(true) {
                state.dismissed = false;
            }
            state.outcome = Some(data);
        }
    }
}

fn draw_outcome_overlay(
    mut contexts: EguiContexts,
    mut state: ResMut<OutcomeOverlayState>,
    bridge: Res<LiveAttachBridge>,
) {
    let Some(ref outcome) = state.outcome.clone() else { return };
    if state.dismissed { return }

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
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.allocate_ui_with_layout(
                screen.size(),
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    // dim backdrop
                    ui.painter().rect_filled(screen, 0.0, egui::Color32::from_rgba_unmultiplied(9, 10, 12, 210));

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(9, 10, 12, 240))
                        .stroke(egui::Stroke::new(1.5, header_color))
                        .inner_margin(egui::Margin::same(40))
                        .corner_radius(egui::CornerRadius::same(8))
                        .show(ui, |ui| {
                            ui.set_max_width(500.0);
                            ui.spacing_mut().item_spacing.y = 16.0;

                            let label = if is_victory { "VICTORY" } else { "DEFEAT" };
                            ui.colored_label(header_color,
                                egui::RichText::new(label).size(36.0).strong());
                            ui.colored_label(egui::Color32::WHITE,
                                egui::RichText::new(&outcome.reason).size(20.0));
                            ui.colored_label(egui::Color32::GRAY,
                                format!("Tick {}", outcome.tick));

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("New Game").size(16.0)).clicked() {
                                    bridge.client.send_rpc("sim.reset", serde_json::json!({"seed": 0}));
                                    state.dismissed = true;
                                    state.outcome = None;
                                }
                                if ui.button(egui::RichText::new("Dismiss").size(16.0)).clicked() {
                                    state.dismissed = true;
                                }
                            });
                        });
                },
            );
        });
}
