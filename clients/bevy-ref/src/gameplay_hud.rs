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
use crate::live_stream::ServerBridge;
use crate::outcome_overlay::{outcome_modal_visible, OutcomeOverlayState, OutcomeSessionGate};
use crate::{MusicCues, OutcomeProgressHud};

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
            .init_resource::<MusicCues>()
            .init_resource::<OutcomeProgressHud>()
            .add_systems(Update, toggle_gameplay_hud)
            .add_systems(
                EguiPrimaryContextPass,
                draw_gameplay_hud.run_if(crate::menus::in_playing),
            );
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_gameplay_hud(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<GameplayHudOpen>) {
    if keys.just_pressed(KeyCode::F9) {
        open.0 = !open.0;
    }
}

fn draw_gameplay_hud(
    mut contexts: EguiContexts,
    open: Res<GameplayHudOpen>,
    scene: Res<LiveStreamScene>,
    music_cues: Res<MusicCues>,
    outcome_progress: Res<OutcomeProgressHud>,
    outcome_state: Option<Res<OutcomeOverlayState>>,
    session_gate: Option<Res<OutcomeSessionGate>>,
    bridge: Option<Res<ServerBridge>>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Build ranked faction list sorted by treasury (descending).
    let mut factions: Vec<_> = scene.faction_entries.iter().collect();
    factions.sort_by(|a, b| {
        b.treasury
            .amount
            .partial_cmp(&a.treasury.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_pop: u32 = scene.population_by_faction.values().sum();
    let max_treasury = factions
        .first()
        .map(|f| f.treasury.amount)
        .unwrap_or(1.0)
        .max(1.0);

    let outcome = outcome_state.as_deref().and_then(|state| {
        let session_active = session_gate
            .as_deref()
            .map(|gate| gate.session_active)
            .unwrap_or(false);
        if session_active && outcome_modal_visible(state) {
            state.outcome.as_ref()
        } else {
            None
        }
    });

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
                    ui.label(
                        egui::RichText::new("[F9] hide")
                            .color(DIM)
                            .small()
                            .italics(),
                    );
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            let music_label = music_cues
                .dominant()
                .map(|cue| {
                    let tempo = cue
                        .tempo_bpm
                        .map(|bpm| format!(", {bpm} bpm"))
                        .unwrap_or_default();
                    format!("{} ({:.0}%{tempo})", cue.mood, cue.intensity * 100.0)
                })
                .unwrap_or_else(|| "awaiting cues".to_string());
            ui.label(
                egui::RichText::new(format!(
                    "Music cues: {} — {music_label}",
                    music_cues.0.len()
                ))
                .color(DIM)
                .small(),
            );
            ui.add_space(4.0);

            // ── Section 1: Outcome Banner ────────────────────────────────
            if let Some(od) = outcome {
                draw_outcome_banner(ui, od.tag.as_str(), od.reason.as_str(), od.tick);
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);
            }

            // ── Section 2: Faction Leaderboard ──────────────────────────
            ui.label(
                egui::RichText::new("Faction Leaderboard")
                    .color(GOLD)
                    .strong()
                    .small(),
            );
            ui.add_space(4.0);

            if factions.is_empty() {
                ui.label(
                    egui::RichText::new("No faction data yet…")
                        .color(DIM)
                        .italics()
                        .small(),
                );
            } else {
                for (rank, entry) in factions.iter().enumerate() {
                    let pop = scene
                        .population_by_faction
                        .get(&entry.id)
                        .copied()
                        .unwrap_or(0);
                    let treasury_norm =
                        (entry.treasury.amount / max_treasury).clamp(0.0, 1.0) as f32;
                    let pop_norm = if total_pop > 0 {
                        pop as f32 / total_pop as f32
                    } else {
                        0.0
                    };
                    draw_faction_row(ui, rank + 1, entry.id, pop, treasury_norm, pop_norm);
                }
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Section 3: Victory Progress ──────────────────────────────
            ui.label(
                egui::RichText::new("Victory Progress")
                    .color(GOLD)
                    .strong()
                    .small(),
            );
            ui.add_space(4.0);

            let live = outcome_progress.0.as_ref();

            // Population victory — snapshot population is authoritative when attached.
            let population = live.map(|p| p.population).unwrap_or(u64::from(total_pop));
            let population_target = live
                .map(|p| p.population_target)
                .unwrap_or(u64::from(POPULATION_VICTORY_TARGET));
            let pop_progress = if population_target > 0 {
                (population as f32 / population_target as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            victory_bar(
                ui,
                "Population",
                pop_progress,
                &format!("{population}/{population_target}"),
            );

            // Tech victory — retain an explicitly-labelled era fallback for old servers.
            let era_proxy = scene
                .faction_entries
                .iter()
                .map(|e| e.era as usize)
                .max()
                .unwrap_or(0);
            let tech_count = live.map(|p| p.researched_techs).unwrap_or(era_proxy);
            let tech_target = live
                .map(|p| p.researched_techs_target)
                .unwrap_or(TECH_VICTORY_TARGET);
            let tech_progress = if tech_target > 0 {
                (tech_count as f32 / tech_target as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            victory_bar(
                ui,
                if live.is_some() {
                    "Technology"
                } else {
                    "Technology (era proxy)"
                },
                tech_progress,
                &format!("{tech_count}/{tech_target}"),
            );

            // Peace victory — never present a hard-coded zero as live data.
            if let Some(progress) = live {
                let peace_fraction = if progress.peace_ticks_target > 0 {
                    (progress.peace_ticks as f32 / progress.peace_ticks_target as f32)
                        .clamp(0.0, 1.0)
                } else {
                    0.0
                };
                victory_bar(
                    ui,
                    "Peace",
                    peace_fraction,
                    &format!("{}/{}", progress.peace_ticks, progress.peace_ticks_target),
                );
            } else {
                victory_bar(ui, "Peace", 0.0, "awaiting snapshot");
            }

            ui.add_space(4.0);
            if let Some(ref bridge) = bridge {
                if ui.small_button("Refresh from Server").clicked() {
                    bridge.send_rpc("sim.status", serde_json::json!({}));
                }
            }
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
            ui.label(
                egui::RichText::new(reason)
                    .color(egui::Color32::WHITE)
                    .size(13.0),
            );
        }
        ui.label(
            egui::RichText::new(format!("Tick {tick}"))
                .color(DIM)
                .small(),
        );
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
        ui.label(
            egui::RichText::new(format!("#{rank}"))
                .color(rank_color)
                .small()
                .strong(),
        );
        ui.label(
            egui::RichText::new(format!("F{faction_id}"))
                .color(ACCENT)
                .small(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("pop:{population}"))
                    .color(DIM)
                    .small(),
            );
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
    let (bg_rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 5.0), egui::Sense::hover());
    ui.painter().rect_filled(
        bg_rect,
        egui::CornerRadius::same(2),
        egui::Color32::from_rgba_premultiplied(40, 45, 60, 200),
    );
    let fill_w = (bg_rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    let fill_rect = egui::Rect::from_min_size(bg_rect.min, egui::vec2(fill_w, bg_rect.height()));
    ui.painter()
        .rect_filled(fill_rect, egui::CornerRadius::same(2), color);
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
    let (bg_rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
    ui.painter().rect_filled(
        bg_rect,
        egui::CornerRadius::same(3),
        egui::Color32::from_rgba_premultiplied(40, 45, 60, 200),
    );
    let fill_w = (bg_rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    let fill_rect = egui::Rect::from_min_size(bg_rect.min, egui::vec2(fill_w, bg_rect.height()));
    ui.painter()
        .rect_filled(fill_rect, egui::CornerRadius::same(3), bar_color);
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
