#![cfg(all(feature = "bevy", feature = "egui"))]

//! Scenario objectives progress panel.
//!
//! Displays live progress toward victory conditions (population, research,
//! peace) as extracted from the [`OutcomeProgressHud`] resource. This is a
//! thin display wrapper — the underlying data arrives via `sim.snapshot`
//! through the [`ServerBridge`](crate::live_stream::ServerBridge) in server
//! mode or from the in-process simulation in standalone mode.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

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
        app.add_systems(EguiPrimaryContextPass, draw_scenario_objectives);
    }
}

/// Renders a small objective-progress card when in playing state.
fn draw_scenario_objectives(
    mut contexts: EguiContexts,
    mode: Res<GameUiMode>,
    progress: Res<OutcomeProgressHud>,
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
                    ui.label(egui::RichText::new("Objectives").color(ACCENT).strong());
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
