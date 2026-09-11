#![cfg(all(feature = "bevy", feature = "egui"))]

//! Replay / simulation playback controls.
//!
//! Provides a compact play/pause/step bar. Live speed changes use correlated
//! `sim.set_speed` requests; standalone mode changes local
//! [`GameSpeed`](crate::game_ui::GameSpeed).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::game_ui::GameSpeed;
use crate::live_attach::LiveAttachBridge;
use crate::live_stream::ServerBridge;
use crate::menus::{in_playing, request_live_speed, GameUiMode, PendingSimSpeed};
use crate::ui_theme::CHIP_FILL;
use crate::AttachMode;

/// Accent colour for active playback buttons.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Glass panel fill.
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
/// Dimmed label colour.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);

/// Bevy plugin for the replay / playback controls bar.
pub struct ReplayControlsPlugin;

impl Plugin for ReplayControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, draw_replay_controls);
    }
}

/// Draws a compact play/pause/step control bar and dispatches RPCs or local speed changes.
fn draw_replay_controls(
    mut contexts: EguiContexts,
    mode: Res<GameUiMode>,
    attach: Res<AttachMode>,
    live_bridge: Option<Res<LiveAttachBridge>>,
    bridge: Option<Res<ServerBridge>>,
    mut pending_speed: ResMut<PendingSimSpeed>,
    mut speed: ResMut<GameSpeed>,
) {
    if !in_playing(mode) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let is_paused = speed.multiplier < 0.01;
    let bridge_ref = bridge.as_deref();
    let live_bridge_ref = live_bridge.as_deref();

    egui::Area::new(egui::Id::new("replay_controls_bar"))
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -56.0])
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .stroke(egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.16)))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Play/Pause button
                        let label = if is_paused { "▶ Play" } else { "⏸ Pause" };
                        let btn = egui::Button::new(
                            egui::RichText::new(label).color(ACCENT).strong(),
                        )
                        .fill(if is_paused {
                            ACCENT.gamma_multiply(0.25)
                        } else {
                            CHIP_FILL
                        });
                        if ui.add(btn).clicked() {
                            if is_paused {
                                // Resume
                                if *attach == AttachMode::Server {
                                    request_live_speed(
                                        &mut pending_speed,
                                        live_bridge_ref,
                                        1,
                                        None,
                                    );
                                } else {
                                    speed.restore_after_resume();
                                }
                            } else {
                                // Pause
                                if *attach == AttachMode::Server {
                                    request_live_speed(
                                        &mut pending_speed,
                                        live_bridge_ref,
                                        0,
                                        None,
                                    );
                                } else {
                                    speed.remember_non_zero();
                                    speed.multiplier = 0.0;
                                }
                            }
                        }

                        // Step (forward one tick)
                        let step_btn = egui::Button::new(
                            egui::RichText::new("⏭ Step").color(DIM),
                        )
                        .fill(CHIP_FILL);
                        if ui.add(step_btn).clicked() && *attach == AttachMode::Server {
                            if let Some(bridge) = bridge_ref {
                                bridge.send_rpc(
                                    "sim.command",
                                    serde_json::json!({"action": "step"}),
                                );
                            }
                        }

                        // Speed indicators
                        ui.separator();
                        let speed_steps: &[(f32, &str)] = if *attach == AttachMode::Server {
                            &[(1.0, "1x"), (2.0, "2x"), (4.0, "4x"), (8.0, "8x")]
                        } else {
                            &[(1.0, "1x"), (2.0, "2x"), (5.0, "5x"), (10.0, "10x")]
                        };
                        for &(mult, label) in speed_steps {
                            let active =
                                (speed.multiplier - mult).abs() < 0.01;
                            let btn = egui::Button::new(
                                egui::RichText::new(label)
                                    .color(if active { ACCENT } else { DIM })
                                    .strong(),
                            )
                            .fill(if active {
                                ACCENT.gamma_multiply(0.30)
                            } else {
                                CHIP_FILL
                            });
                            if ui.add(btn).clicked() {
                                if *attach == AttachMode::Server {
                                    request_live_speed(
                                        &mut pending_speed,
                                        live_bridge_ref,
                                        mult as u32,
                                        None,
                                    );
                                } else {
                                    speed.multiplier = mult;
                                }
                            }
                        }

                        ui.label(
                            egui::RichText::new(format!(
                                "{}",
                                if is_paused {
                                    "PAUSED".to_string()
                                } else {
                                    format!("{}x", speed.multiplier as u32)
                                }
                            ))
                            .color(DIM)
                            .small(),
                        );
                    });
                });
        });
}
