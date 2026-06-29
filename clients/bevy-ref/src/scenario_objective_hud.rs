#![cfg(all(feature = "bevy", feature = "egui"))]

//! Scenario objective and progress HUD — displays current objective text and win/progress.
//!
//! When a scenario is active (via `SimState`), renders a small panel showing:
//! - Objective label ("Population Goal", "Survival", etc.)
//! - Progress bar and numeric progress (e.g. "50/100")
//! - Faction/goal context
//!
//! If no scenario is active, defaults to a simple "reach population N" goal.
//! Option-wraps all resource reads to avoid panics on startup.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::sim_bridge::SimState;

// ── Palette ───────────────────────────────────────────────────────────────────

const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const TEAL: egui::Color32 = egui::Color32::from_rgb(126, 186, 181);

// Default objective targets for standalone (no scenario)
const DEFAULT_POPULATION_GOAL: u32 = 500;

// ── Resource ──────────────────────────────────────────────────────────────────

/// Current scenario objective state for HUD display.
#[derive(Resource, Debug, Clone)]
pub struct ObjectiveState {
    /// Human-readable objective description.
    pub objective_text: String,
    /// Current progress value (e.g. current population).
    pub current: u32,
    /// Target/threshold value (e.g. goal population).
    pub target: u32,
    /// Faction context (if applicable), e.g. "Faction 1".
    pub faction_context: Option<String>,
}

impl Default for ObjectiveState {
    fn default() -> Self {
        Self {
            objective_text: "Reach Population Goal".to_string(),
            current: 0,
            target: DEFAULT_POPULATION_GOAL,
            faction_context: None,
        }
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct ScenarioObjectiveHudPlugin;

impl Plugin for ScenarioObjectiveHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectiveState>()
            .add_systems(Update, update_objective_state)
            .add_systems(EguiPrimaryContextPass, draw_objective_hud);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Sync objective state from the active simulation.
fn update_objective_state(
    sim: Option<Res<SimState>>,
    mut objective: ResMut<ObjectiveState>,
) {
    // If we have a simulation, read current population and update objective.
    if let Some(sim_res) = sim {
        let current_pop = sim_res.0.state.population as u32;
        objective.current = current_pop;
        // Default goal: reach DEFAULT_POPULATION_GOAL
        if objective.target == 0 {
            objective.target = DEFAULT_POPULATION_GOAL;
        }
    }
}

/// Draw the scenario objective HUD as a small top-left panel.
fn draw_objective_hud(
    mut contexts: EguiContexts,
    objective: Res<ObjectiveState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Small panel in the top-left corner (just below the game info overlays).
    egui::Window::new("Objective")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .default_width(280.0)
        .resizable(false)
        .collapsible(false)
        .title_bar(true)
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(10))
                .corner_radius(egui::CornerRadius::same(8)),
        )
        .show(ctx, |ui| {
            // ── Objective Label ──────────────────────────────────────────────
            ui.label(
                egui::RichText::new(&objective.objective_text)
                    .color(ACCENT)
                    .strong()
                    .size(12.0),
            );

            // Faction context if present
            if let Some(faction) = &objective.faction_context {
                ui.label(
                    egui::RichText::new(faction)
                        .color(DIM)
                        .italics()
                        .size(10.0),
                );
            }

            ui.add_space(4.0);

            // ── Progress Bar ─────────────────────────────────────────────────
            let progress = (objective.current as f32 / objective.target as f32).clamp(0.0, 1.0);
            let (bg_rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 12.0),
                egui::Sense::hover(),
            );

            // Background
            ui.painter().rect_filled(
                bg_rect,
                egui::CornerRadius::same(3),
                egui::Color32::from_rgba_premultiplied(40, 45, 60, 200),
            );

            // Fill
            let fill_w = (bg_rect.width() * progress).max(0.0);
            let fill_rect = egui::Rect::from_min_size(bg_rect.min, egui::vec2(fill_w, bg_rect.height()));
            let bar_color = if progress >= 1.0 { TEAL } else { ACCENT };
            ui.painter()
                .rect_filled(fill_rect, egui::CornerRadius::same(3), bar_color);

            ui.add_space(2.0);

            // ── Progress Text ────────────────────────────────────────────────
            let progress_text = format!("{}/{}", objective.current, objective.target);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(progress_text).color(DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let percent_text = format!("{:.0}%", progress * 100.0);
                    ui.label(egui::RichText::new(percent_text).color(ACCENT).small());
                });
            });
        });
}
