#![cfg(all(feature = "bevy", feature = "egui"))]

//! Emergence dashboard panel — 6-metric criticality readout (FR-CIV-EMERGE-DASH-001).
//!
//! Toggle with `E`. Reads `HudState::snapshot.emergence` (polled every 10 s by
//! `poll_emergence` in `bevy_window`). Displays progress bars + regime badge.

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
const CYAN: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);

// ── Resource ──────────────────────────────────────────────────────────────────

/// Whether the emergence dashboard is open.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmergenceDashboardOpen(pub bool);

impl Default for EmergenceDashboardOpen {
    fn default() -> Self {
        Self(false)
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct EmergenceDashboardPlugin;

impl Plugin for EmergenceDashboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmergenceDashboardOpen>()
            .add_systems(Update, toggle_emergence_dashboard)
            .add_systems(EguiPrimaryContextPass, draw_emergence_dashboard);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_emergence_dashboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<EmergenceDashboardOpen>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        open.0 = !open.0;
    }
}

fn draw_emergence_dashboard(
    mut contexts: EguiContexts,
    open: Res<EmergenceDashboardOpen>,
    emergence_data: Option<Res<EmergenceHudData>>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

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
            // ── Header ──────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Emergence Dashboard")
                        .color(ACCENT)
                        .strong()
                        .size(14.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("[E] hide").color(DIM).small().italics());
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
                    // 1. Shannon Entropy
                    metric_bar(
                        ui,
                        "Entropy",
                        em.entropy_norm,
                        &format!("{:.3} norm", em.entropy_norm),
                        bar_color_neutral(em.entropy_norm),
                    );

                    // 2. Power-law α (target 2.0–3.0; scale: 0..=5)
                    let alpha_norm = (em.power_law_alpha / 5.0).clamp(0.0, 1.0);
                    let alpha_in_target = em.power_law_alpha >= 2.0 && em.power_law_alpha <= 3.0;
                    let alpha_color = if alpha_in_target { GREEN } else { GOLD };
                    metric_bar(
                        ui,
                        "Power-law \u{03b1}",
                        alpha_norm,
                        &format!("{:.2} (target 2\u{2013}3)", em.power_law_alpha),
                        alpha_color,
                    );

                    // 3. Structure count
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

                    // 4. Novelty rate
                    let novelty_norm = (em.novelty_rate * 10.0).clamp(0.0, 1.0);
                    metric_bar(
                        ui,
                        "Novelty rate",
                        novelty_norm,
                        &format!("{:.4}/tick", em.novelty_rate),
                        bar_color_neutral(novelty_norm),
                    );

                    // 5. Coupling MI
                    let mi_val = em.mi_material_faction_norm.unwrap_or(0.0);
                    let mi_label = if em.mi_material_faction_norm.is_some() {
                        format!("{:.3}", mi_val)
                    } else {
                        "\u{2014}".to_string()
                    };
                    metric_bar(
                        ui,
                        "Coupling MI",
                        mi_val.clamp(0.0, 1.0),
                        &mi_label,
                        bar_color_neutral(mi_val),
                    );

                    // 6. Criticality regime badge
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                    let (regime_label, regime_color) = regime_badge(&em.branching_regime);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Criticality").color(DIM).small());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(regime_label)
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

fn metric_bar(ui: &mut egui::Ui, label: &str, fraction: f32, value_str: &str, bar_color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value_str).strong().small());
        });
    });
    let (bg_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
    ui.painter().rect_filled(bg_rect, egui::CornerRadius::same(3), egui::Color32::from_rgba_premultiplied(40, 45, 60, 200));
    let fill_w = (bg_rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    let fill_rect = egui::Rect::from_min_size(bg_rect.min, egui::vec2(fill_w, bg_rect.height()));
    ui.painter().rect_filled(fill_rect, egui::CornerRadius::same(3), bar_color);
    ui.add_space(4.0);
}

fn bar_color_neutral(fraction: f32) -> egui::Color32 {
    // Low = blue-dim, mid = cyan, high = gold
    if fraction < 0.4 {
        egui::Color32::from_rgb(60, 120, 200)
    } else if fraction < 0.75 {
        CYAN
    } else {
        GOLD
    }
}

fn regime_badge(regime: &str) -> (&'static str, egui::Color32) {
    match regime.trim().to_ascii_uppercase().as_str() {
        r if r.contains("SUPER") => ("SUPERCRITICAL", RED),
        r if r.contains("SUB") => ("SUBCRITICAL", egui::Color32::from_rgb(100, 140, 220)),
        _ => ("CRITICAL \u{2714}", GREEN),
    }
}