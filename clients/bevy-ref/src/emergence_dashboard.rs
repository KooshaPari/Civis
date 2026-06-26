#![cfg(all(feature = "bevy", feature = "egui"))]

//! Emergence dashboard panel — criticality readout (P2.3 / FR-CIV-EMERGE-DASH-001).
//!
//! Toggle with `E`. Reads `EmergenceHudData` (polled every 10 s via `sim.emergence`
//! in `bevy_window`, or synced from in-process `SimState` in standalone).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use crate::EmergenceHudData;

// Palette (mirrors diplomacy_ui.rs / faction_hud.rs)
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 210, 120);
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);
const BLUE: egui::Color32 = egui::Color32::from_rgb(100, 140, 220);
const CYAN: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);

// ── Resource ──────────────────────────────────────────────────────────────────

/// State resource for the emergence dashboard HUD (P2.3).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmergenceDashboardState {
    /// Whether the panel is currently visible (F7 toggles).
    pub visible: bool,
}

impl Default for EmergenceDashboardState {
    fn default() -> Self {
        Self { visible: false }
    }
}

/// Whether the emergence dashboard is open.
///
/// Alias kept for compatibility with internal callers.
pub type EmergenceDashboardOpen = EmergenceDashboardState;

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct EmergenceDashboardPlugin;

impl Plugin for EmergenceDashboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmergenceDashboardState>()
            .add_systems(Update, toggle_emergence_dashboard)
            .add_systems(EguiPrimaryContextPass, draw_emergence_dashboard);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_emergence_dashboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EmergenceDashboardState>,
) {
    if keys.just_pressed(KeyCode::F7) || keys.just_pressed(KeyCode::KeyE) {
        state.visible = !state.visible;
    }
}

fn draw_emergence_dashboard(
    mut contexts: EguiContexts,
    state: Res<EmergenceDashboardState>,
    emergence_data: Option<Res<EmergenceHudData>>,
) {
    if !state.visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let emergence = emergence_data.as_deref();

    egui::Window::new("Emergence Dashboard")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .default_width(280.0)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(14))
                .corner_radius(egui::CornerRadius::same(10)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Emergence Dashboard")
                        .color(ACCENT)
                        .strong()
                        .size(14.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("[F7] hide").color(DIM).small().italics());
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            match emergence {
                None => {
                    ui.label(
                        egui::RichText::new("Awaiting first emergence sample (tick 50+)...")
                            .color(DIM)
                            .italics(),
                    );
                }
                Some(em) => {
                    let (regime_short, regime_color) = regime_badge(&em.branching_regime);
                    edge_of_chaos_indicator(ui, regime_short, regime_color);

                    metric_bar(
                        ui,
                        "Entropy",
                        em.entropy_norm,
                        &format!("{:.3} norm · {:.2} bits", em.entropy_norm, em.entropy_bits),
                        bar_color_neutral(em.entropy_norm),
                    );

                    let alpha_norm = (em.power_law_alpha / 5.0).clamp(0.0, 1.0);
                    let alpha_in_target = em.power_law_alpha >= 2.0 && em.power_law_alpha <= 3.0;
                    metric_bar(
                        ui,
                        "Power-law α",
                        alpha_norm,
                        &format!("{:.2} (target 2–3)", em.power_law_alpha),
                        if alpha_in_target { GREEN } else { GOLD },
                    );

                    let struct_label = em
                        .structure_count
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "—".to_string());
                    ui.horizontal(|ui| {
                        ui.set_min_width(240.0);
                        ui.label(egui::RichText::new("Structures").color(DIM).small());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(struct_label).strong().small());
                        });
                    });
                    ui.add_space(2.0);

                    let novelty_norm = (em.novelty_rate * 10.0).clamp(0.0, 1.0);
                    metric_bar(
                        ui,
                        "Novelty rate",
                        novelty_norm,
                        &format!("{:.4}/tick", em.novelty_rate),
                        bar_color_neutral(novelty_norm),
                    );

                    let sigma_norm = (em.branching_sigma / 1.5).clamp(0.0, 1.0);
                    metric_bar(
                        ui,
                        "Branching σ",
                        sigma_norm,
                        &format!("{:.3}", em.branching_sigma),
                        regime_color,
                    );

                    let mi_val = em.mi_material_faction_norm.unwrap_or(0.0);
                    let mi_label = if em.mi_material_faction_norm.is_some() {
                        format!("{:.3}", mi_val)
                    } else {
                        "—".to_string()
                    };
                    metric_bar(
                        ui,
                        "Coupling MI",
                        mi_val.clamp(0.0, 1.0),
                        &mi_label,
                        bar_color_neutral(mi_val),
                    );

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Regime").color(DIM).small());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&em.branching_regime)
                                    .color(regime_color)
                                    .strong()
                                    .small(),
                            );
                        });
                    });
                }
            }
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn edge_of_chaos_indicator(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Edge of chaos").color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(label).color(color).strong().small());
        });
    });
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(5),
        egui::Color32::from_rgba_premultiplied(40, 45, 60, 200),
    );
    let inner = rect.shrink(1.0);
    ui.painter().rect_filled(inner, egui::CornerRadius::same(4), color);
    ui.add_space(6.0);
}

fn metric_bar(
    ui: &mut egui::Ui,
    label: &str,
    fraction: f32,
    value_str: &str,
    bar_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value_str).strong().small());
        });
    });
    let (bg_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
    ui.painter().rect_filled(
        bg_rect,
        egui::CornerRadius::same(3),
        egui::Color32::from_rgba_premultiplied(40, 45, 60, 200),
    );
    let fill_w = (bg_rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    let fill_rect = egui::Rect::from_min_size(bg_rect.min, egui::vec2(fill_w, bg_rect.height()));
    ui.painter().rect_filled(fill_rect, egui::CornerRadius::same(3), bar_color);
    ui.add_space(4.0);
}

fn bar_color_neutral(fraction: f32) -> egui::Color32 {
    if fraction < 0.4 {
        egui::Color32::from_rgb(60, 120, 200)
    } else if fraction < 0.75 {
        CYAN
    } else {
        GOLD
    }
}

fn regime_badge(regime: &str) -> (&'static str, egui::Color32) {
    let lower = regime.trim().to_ascii_lowercase();
    if lower.contains("supercritical") || lower.contains("explosion") {
        ("SUPERCRITICAL", RED)
    } else if lower.contains("heat-death")
        || lower.contains("heat death")
        || lower.contains("subcritical")
    {
        ("SUBCRITICAL", BLUE)
    } else {
        ("EDGE OF CHAOS", GREEN)
    }
}

#[cfg(test)]
mod tests {
    use super::EmergenceDashboardState;

    #[test]
    fn test_emergence_dashboard_default_state() {
        assert!(!EmergenceDashboardState::default().visible);
    }
}
