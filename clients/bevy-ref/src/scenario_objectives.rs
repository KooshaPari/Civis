#![cfg(all(feature = "bevy", feature = "egui"))]

//! Scenario objectives progress panel.
//!
//! Displays live progress toward victory conditions (population, research,
//! peace) as extracted from the [`OutcomeProgressHud`] resource. This is a
//! thin display wrapper; the underlying data arrives via `sim.snapshot` /
//! `sim.outcome` over the [`ServerBridge`](crate::live_stream::ServerBridge)
//! (server mode) or from the in-process simulation in standalone mode.
//!
//! **Server-wired** — fires `sim.outcome` on first draw so the panel can
//! refresh victory-progress counters (population, research, peace) from the
//! server's authoritative outcome progress block.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_stream::ServerBridge;
use crate::menus::{in_playing, GameUiMode};
use crate::ui_theme::CHIP_FILL;
use crate::OutcomeProgressHud;
use crate::OutcomeProgressHudData;

/// Accent colour for objective labels.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Glass panel fill.
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
/// Dimmed label colour.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);

/// Bevy plugin that renders the scenario objectives progress panel.
pub struct ScenarioObjectivesPlugin;

impl Plugin for ScenarioObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            request_objective_progress.run_if(in_playing),
        )
        .add_systems(EguiPrimaryContextPass, draw_scenario_objectives);
    }
}

/// Fire `sim.outcome` once when the objectives panel is shown so the progress
/// block (`OutcomeProgressHud`) is filled with server-authoritative counters.
/// Idempotent: the `Local<bool>` gate ensures only one RPC is sent per
/// session.
fn request_objective_progress(
    bridge: Option<Res<ServerBridge>>,
    mut sent: Local<bool>,
) {
    if *sent {
        return;
    }
    let Some(ref bridge) = bridge else {
        *sent = true; // mark sent so we don't re-check the resource every frame
        return;
    };
    bridge.send_rpc("sim.outcome", serde_json::json!({}));
    *sent = true;
}

/// Renders a small objective-progress card when in playing state.
fn draw_scenario_objectives(
    mut contexts: EguiContexts,
    mode: Res<GameUiMode>,
    progress: Res<OutcomeProgressHud>,
    bridge: Option<Res<ServerBridge>>,
) {
    if !in_playing(mode) {
        return;
    }
    let Some(data) = &progress.0 else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Area::new(egui::Id::new("scenario_objectives_panel"))
        .anchor(egui::Align2::RIGHT_TOP, [-8.0, 60.0])
        .show(ctx, |ui| {
            ui.set_max_width(240.0);
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .stroke(egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.18)))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Objectives").color(ACCENT).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(ref bridge) = bridge {
                                if ui
                                    .add(egui::Button::new("\u{21bb}").small().frame(false))
                                    .on_hover_text("Refresh progress (sim.outcome)")
                                    .clicked()
                                {
                                    bridge.send_rpc("sim.outcome", serde_json::json!({}));
                                }
                            }
                        });
                    });
                    ui.add_space(4.0);
                    objective_row(ui, "Population", data.population, data.population_target);
                    objective_row(
                        ui,
                        "Research",
                        data.researched_techs as u64,
                        data.researched_techs_target as u64,
                    );
                    objective_row(ui, "Peace", data.peace_ticks, data.peace_ticks_target);
                });
        });
}

/// Renders a single objective row with a progress bar.
fn objective_row(ui: &mut egui::Ui, label: &str, current: u64, target: u64) {
    if target == 0 {
        return;
    }
    let fraction = (current as f32 / target as f32).clamp(0.0, 1.0);
    let color = if fraction >= 1.0 {
        egui::Color32::from_rgb(120, 220, 130)
    } else {
        ACCENT
    };
    ui.label(egui::RichText::new(label).color(DIM).small());
    ui.add(
        egui::ProgressBar::new(fraction)
            .fill(color)
            .text(format!("{current} / {target}")),
    );
    ui.add_space(2.0);
}
