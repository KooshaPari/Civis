#![cfg(all(feature = "bevy", feature = "egui"))]

//! Game-over / outcome screen for the Civis reference client.
//!
//! Shows a full-screen frosted-glass panel with final session stats
//! (tick, population, era, factions, duration) and a scrollable event
//! log. Provides "Restart" (→ main menu) and "View Summary" (toggle
//! expanded log) buttons.

use bevy::prelude::*;
use bevy::state::condition::in_state;
use bevy_egui::{egui, EguiContexts};

use crate::menus::{AppState, MainMenuCommand, MenuCommand};
use crate::ui_theme::liquid_glass_frame;

// ---------------------------------------------------------------------------
// Theme constants (match menus.rs)
// ---------------------------------------------------------------------------

/// Cyan accent for titles and interactive highlights.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Dark glassmorphism panel fill (premultiplied alpha).
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
/// Dimmed label colour for secondary text.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
/// Full-screen dark overlay behind the outcome panel.
const OVERLAY_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 160);

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Snapshot of the world state at the moment the civilisation ends.
///
/// Populated by the simulation layer when the game-over condition is
/// detected and consumed each frame by [`render_game_over`].
#[derive(Resource, Debug, Clone)]
pub struct GameOverSnapshot {
    /// Final simulation tick count.
    pub tick: u64,
    /// Total population at game-over time.
    pub population: u64,
    /// Era label at the end of the session.
    pub era: String,
    /// Number of active factions.
    pub factions: u32,
    /// Real-world wall-clock duration of the session in seconds.
    pub duration_seconds: f64,
    /// Chronological event-log entries shown in the scrollable section.
    pub event_log: Vec<String>,
}

impl Default for GameOverSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 0,
            era: "0".to_string(),
            factions: 0,
            duration_seconds: 0.0,
            event_log: Vec::new(),
        }
    }
}

/// Tracks whether the event log section is expanded / collapsed.
#[derive(Resource, Default, Debug)]
pub struct GameOverLogExpanded(pub bool);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Bevy plugin for the game-over / outcome screen.
///
/// Registers [`GameOverSnapshot`], [`GameOverLogExpanded`], and the
/// [`render_game_over`] system which gates on [`AppState::GameOver`].
pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameOverSnapshot>()
            .init_resource::<GameOverLogExpanded>()
            .add_systems(
                Update,
                render_game_over.run_if(in_state(AppState::GameOver)),
            );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Render the game-over outcome panel.
///
/// Early-return when the state is not [`AppState::GameOver`]. The panel
/// covers the full screen with a dark overlay and a centred liquid-glass
/// card. It is gated on the `in_state(AppState::GameOver)` run condition,
/// so it only runs while the game-over screen is active.
fn render_game_over(
    mut contexts: EguiContexts,
    snapshot: Res<GameOverSnapshot>,
    mut command: ResMut<MenuCommand>,
    mut log_expanded: ResMut<GameOverLogExpanded>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    draw_game_over_overlay(ctx, &snapshot, &mut command, &mut log_expanded);
}

// ---------------------------------------------------------------------------
// UI layout helpers
// ---------------------------------------------------------------------------

fn draw_game_over_overlay(
    ctx: &egui::Context,
    snapshot: &GameOverSnapshot,
    command: &mut MenuCommand,
    log_expanded: &mut GameOverLogExpanded,
) {
    // Full-screen dim overlay background.
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new("game_over_dim"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(screen, egui::CornerRadius::ZERO, OVERLAY_DIM);
        });

    // Centered outcome panel.
    egui::Area::new(egui::Id::new("game_over_panel"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .liquid_glass_frame(egui::Margin::same(18), crate::ui_theme::RADIUS_PANEL)
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(28))
                .show(ui, |ui| {
                    ui.set_min_width(520.0);
                    ui.vertical_centered(|ui| {
                        // — Title —
                        ui.label(
                            egui::RichText::new("Civilization Outcome")
                                .size(36.0)
                                .color(ACCENT)
                                .strong(),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Your civilization has reached its end.")
                                .size(14.0)
                                .color(DIM),
                        );
                        ui.add_space(20.0);

                        // — Stats panel —
                        outcome_stats(ui, snapshot);
                        ui.add_space(16.0);

                        // — Event log section —
                        outcome_event_log(ui, snapshot, log_expanded);
                        ui.add_space(20.0);

                        // — Action buttons —
                        outcome_buttons(ui, command, log_expanded);
                    });
                });
        });
}

/// Display large stat cards for tick, population, era, factions, duration.
fn outcome_stats(ui: &mut egui::Ui, snapshot: &GameOverSnapshot) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 40))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            // Row 1: Tick & Population
            ui.horizontal(|ui| {
                stat_card(ui, "Ticks Elapsed", &format_count(snapshot.tick), ACCENT);
                ui.add_space(24.0);
                stat_card(
                    ui,
                    "Population",
                    &format_count(snapshot.population),
                    egui::Color32::from_rgb(120, 220, 130),
                );
            });
            ui.add_space(12.0);

            // Row 2: Era & Factions
            ui.horizontal(|ui| {
                stat_card(ui, "Era Reached", &snapshot.era, ACCENT);
                ui.add_space(24.0);
                stat_card(
                    ui,
                    "Active Factions",
                    &format_count(snapshot.factions as u64),
                    egui::Color32::from_rgb(240, 200, 90),
                );
            });
            ui.add_space(12.0);

            // Row 3: Session Duration
            ui.horizontal(|ui| {
                stat_card(
                    ui,
                    "Session Duration",
                    &format_duration(snapshot.duration_seconds),
                    DIM,
                );
            });
        });
}

/// A single stat label + large value pair.
fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, colour: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).size(11.0).color(DIM).small());
        ui.label(
            egui::RichText::new(value)
                .size(24.0)
                .color(colour)
                .strong()
                .monospace(),
        );
    });
}

/// Scrollable event-log section. Collapsed when [`GameOverLogExpanded`] is false.
fn outcome_event_log(
    ui: &mut egui::Ui,
    snapshot: &GameOverSnapshot,
    expanded: &mut GameOverLogExpanded,
) {
    if !expanded.0 {
        // Only show the toggle label when collapsed; the button lives in
        // the action area below.
        return;
    }

    ui.label(
        egui::RichText::new("Event Log")
            .size(14.0)
            .color(ACCENT)
            .strong(),
    );
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .max_height(180.0)
        .auto_shrink([true; 2])
        .show(ui, |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 30))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_min_width(440.0);
                    if snapshot.event_log.is_empty() {
                        ui.label(
                            egui::RichText::new("No events recorded.")
                                .color(DIM)
                                .italics(),
                        );
                    } else {
                        for entry in &snapshot.event_log {
                            ui.label(
                                egui::RichText::new(entry)
                                    .size(12.0)
                                    .color(egui::Color32::WHITE),
                            );
                        }
                    }
                });
        });
}

/// Action buttons: "Restart" returns to the main menu; "View Summary"
/// toggles the expanded event log.
fn outcome_buttons(
    ui: &mut egui::Ui,
    command: &mut MenuCommand,
    log_expanded: &mut GameOverLogExpanded,
) {
    ui.horizontal(|ui| {
        // Centered button group within the vertical layout.
        ui.vertical_centered(|ui| {
            if menu_button(ui, "Restart").clicked() {
                command.action = MainMenuCommand::ExitToMainMenu;
            }
            ui.add_space(8.0);
            let summary_label = if log_expanded.0 {
                "Hide Summary"
            } else {
                "View Summary"
            };
            if menu_button(ui, summary_label).clicked() {
                log_expanded.0 = !log_expanded.0;
            }
        });
    });
}

/// Styled menu button matching the appearance used by [`menus`](crate::menus).
fn menu_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(egui::RichText::new(label).size(16.0))
        .fill(ACCENT.gamma_multiply(0.18))
        .min_size(egui::vec2(220.0, 40.0))
        .corner_radius(egui::CornerRadius::same(8));
    ui.add(btn)
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Compact human-readable count (e.g. 1_234 → "1,234").
fn format_count(value: u64) -> String {
    let s = value.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Format a wall-clock duration in seconds into a human-readable string
/// (e.g. `"1h 23m 45s"`, `"45m 12s"`, or `"32s"`).
fn format_duration(total_seconds: f64) -> String {
    let total = total_seconds.max(0.0) as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {secs}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_defaults_are_zero() {
        let snap = GameOverSnapshot::default();
        assert_eq!(snap.tick, 0);
        assert_eq!(snap.population, 0);
        assert_eq!(snap.era, "0");
        assert_eq!(snap.factions, 0);
        assert!(snap.event_log.is_empty());
    }

    #[test]
    fn format_count_separates_thousands() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(1), "1");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }

    #[test]
    fn format_duration_variants() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(120.0), "2m 0s");
        assert_eq!(format_duration(3661.0), "1h 1m 1s");
        assert_eq!(format_duration(7200.0), "2h 0m 0s");
    }

    #[test]
    fn log_expanded_resource_defaults_false() {
        assert!(!GameOverLogExpanded::default().0);
    }

    #[test]
    fn format_duration_handles_negative() {
        assert_eq!(format_duration(-10.0), "0s");
    }
}
