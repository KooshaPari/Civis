#![cfg(all(feature = "bevy", feature = "egui"))]

//! Gameplay HUD panel — faction leaderboard, victory progress, outcome banner (FR-CIV-GAME-001).
//!
//! Toggle with `F9`. Reads live faction data from `LiveStreamScene` and outcome
//! state from `OutcomeOverlayState`. Panel renders three sections:
//! 1. Faction leaderboard ranked by composite treasury score.
//! 2. Victory progress toward each win condition.
//! 3. Prominent outcome banner when victory or defeat is detected.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_stream::LiveStreamScene;
use crate::outcome_overlay::OutcomeOverlayState;

// ── Palette (mirrors emergence_dashboard / faction_hud) ───────────────────────

const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 210, 120);
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);
const TEAL: egui::Color32 = egui::Color32::from_rgb(126, 186, 181);

// Victory thresholds (mirrors conditions.rs constants for progress display)
const POPULATION_VICTORY_TARGET: u32 = 10_000;
const TECH_VICTORY_TARGET: usize = 12;
const PEACE_TICKS_TARGET: u64 = 500;

// ── Resource ──────────────────────────────────────────────────────────────────

/// Whether the gameplay HUD panel is open.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayHudOpen(pub bool);

impl Default for GameplayHudOpen {
    fn default() -> Self {
        Self(false)
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Registers the gameplay HUD (F9 toggle).
pub struct GameplayHudPlugin;

impl Plugin for GameplayHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayHudOpen>()
            .add_systems(Update, toggle_gameplay_hud)
            .add_systems(EguiPrimaryContextPass, draw_gameplay_hud);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_gameplay_hud(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<GameplayHudOpen>,
) {
    if keys.just_pressed(KeyCode::F9) {
        open.0 = !open.0;
    }
}

fn draw_gameplay_hud(
    mut contexts: EguiContexts,
    open: Res<GameplayHudOpen>,
    scene: Res<LiveStreamScene>,
    outcome_state: Option<Res<OutcomeOverlayState>>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Build ranked faction list sorted by treasury (descending).
    let mut factions: Vec<_> = scene.faction_entries.iter().collect();
    factions.sort_by(|a, b| {
        b.treasury.amount.partial_cmp(&a.treasury.amount).unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_pop: u32 = scene.population_by_faction.values().sum();
    let max_treasury = factions.first().map(|f| f.treasury.amount).unwrap_or(1.0).max(1.0);

    let outcome = outcome_state.as_deref().and_then(|s| s.outcome.as_ref());

    egui::Window::new("Gameplay HUD")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 260.0))
        .default_width(300.0)
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
                    egui::RichText::new("Gameplay HUD")
                        .color(ACCENT)
                        .strong()
                        .size(14.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("[F9] hide").color(DIM).small().italics());
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            // ── Section 1: Outcome Banner ────────────────────────────────
            if let Some(od) = outcome {
                draw_outcome_banner(ui, od.tag.as_str(), od.reason.as_str(), od.tick);
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);
            }

            // ── Section 2: Faction Leaderboard ──────────────────────────
            ui.label(egui::RichText::new("Faction Leaderboard").color(GOLD).strong().small());
            ui.add_space(4.0);

            if factions.is_empty() {
                ui.label(egui::RichText::new("No faction data yet…").color(DIM).italics().small());
            } else {
                for (rank, entry) in factions.iter().enumerate() {
                    let pop = scene.population_by_faction.get(&entry.id).copied().unwrap_or(0);
                    let treasury_norm = (entry.treasury.amount / max_treasury).clamp(0.0, 1.0) as f32;
                    let pop_norm = if total_pop > 0 { pop as f32 / total_pop as f32 } else { 0.0 };
                    draw_faction_row(ui, rank + 1, entry.id, pop, treasury_norm, pop_norm);
                }
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Section 3: Victory Progress ──────────────────────────────
            ui.label(egui::RichText::new("Victory Progress").color(GOLD).strong().small());
            ui.add_space(4.0);

            // Population victory
            let pop_progress = (total_pop as f32 / POPULATION_VICTORY_TARGET as f32).clamp(0.0, 1.0);
            victory_bar(
                ui,
                "Population",
                pop_progress,
                &format!("{}/{}", total_pop, POPULATION_VICTORY_TARGET),
            );

            // Tech victory (use faction count as proxy when no tech data available)
            let tech_count = scene.faction_entries.iter().map(|e| e.era as usize).max().unwrap_or(0);
            let tech_progress = (tech_count as f32 / TECH_VICTORY_TARGET as f32).clamp(0.0, 1.0);
            victory_bar(
                ui,
                "Technology (era)",
                tech_progress,
                &format!("{}/{}", tech_count, TECH_VICTORY_TARGET),
            );

            // Peace victory (we don't have tick info here, show as unknown)
            victory_bar(
                ui,
                "Peace (500 ticks)",
                0.0,
                "—",
            );
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn draw_outcome_banner(ui: &mut egui::Ui, tag: &str, reason: &str, tick: u64) {
    let (label, color) = if tag == "victory" {
        ("VICTORY", TEAL)
    } else {
        ("DEFEAT", RED)
    };
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(label).color(color).size(22.0).strong());
        if !reason.is_empty() {
            ui.label(egui::RichText::new(reason).color(egui::Color32::WHITE).size(13.0));
        }
        ui.label(egui::RichText::new(format!("Tick {tick}")).color(DIM).small());
    });
}

fn draw_faction_row(
    ui: &mut egui::Ui,
    rank: usize,
    faction_id: u32,
    population: u32,
    treasury_norm: f32,
    pop_norm: f32,
) {
    ui.horizontal(|ui| {
        let rank_color = match rank {
            1 => GOLD,
            2 => egui::Color32::from_rgb(192, 192, 192),
            3 => egui::Color32::from_rgb(205, 127, 50),
            _ => DIM,
        };
        ui.label(egui::RichText::new(format!("#{rank}")).color(rank_color).small().strong());
        ui.label(egui::RichText::new(format!("F{faction_id}")).color(ACCENT).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("pop:{population}")).color(DIM).small());
        });
    });

    // Treasury bar
    sub_bar(ui, "treasury", treasury_norm, GOLD);
    // Population share bar
    sub_bar(ui, "pop share", pop_norm, GREEN);
    ui.add_space(3.0);
}

fn sub_bar(ui: &mut egui::Ui, label: &str, fraction: f32, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
    });
    let (bg_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 5.0), egui::Sense::hover());
    ui.painter().rect_filled(
        bg_rect,
        egui::CornerRadius::same(2),
        egui::Color32::from_rgba_premultiplied(40, 45, 60, 200),
    );
    let fill_w = (bg_rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    let fill_rect = egui::Rect::from_min_size(bg_rect.min, egui::vec2(fill_w, bg_rect.height()));
    ui.painter().rect_filled(fill_rect, egui::CornerRadius::same(2), color);
    ui.add_space(2.0);
}

fn victory_bar(ui: &mut egui::Ui, label: &str, fraction: f32, value_str: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value_str).strong().small());
        });
    });
    let bar_color = if fraction >= 1.0 { TEAL } else { ACCENT };
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_hud_default_is_closed() {
        let open = GameplayHudOpen::default();
        assert!(!open.0, "gameplay HUD should be closed by default");
    }
}
