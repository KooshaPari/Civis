#![cfg(all(feature = "bevy", feature = "egui"))]

//! Player faction HUD panel — shows the player-owned faction stats (top-left corner).
//!
//! Toggle with `F`. Panel reads `PlayerFactionId` to locate the entry in
//! `LiveStreamScene::faction_entries`, then displays government, era,
//! treasury, and live civilian count for that faction.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_stream::LiveStreamScene;

// Palette (mirrors diplomacy_ui.rs / game_ui.rs)
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 210, 120);
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);

// ── Resources ────────────────────────────────────────────────────────────────

/// The faction id the local player controls (0 = Ardani, 1 = Velthari, 2 = Grundak).
///
/// Set from [`crate::game_ui::ScenarioPanel`] on scenario launch.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerFactionId(pub u32);

/// HUD open/closed toggle state.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactionHudOpen(pub bool);

impl Default for FactionHudOpen {
    fn default() -> Self {
        Self(true)
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Registers the player faction HUD panel.
pub struct FactionHudPlugin;

impl Plugin for FactionHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerFactionId>()
            .init_resource::<FactionHudOpen>()
            .add_systems(Update, toggle_faction_hud)
            .add_systems(EguiPrimaryContextPass, draw_faction_hud);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_faction_hud(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<FactionHudOpen>) {
    if keys.just_pressed(KeyCode::KeyF) {
        open.0 = !open.0;
    }
}

fn draw_faction_hud(
    mut contexts: EguiContexts,
    open: Res<FactionHudOpen>,
    player: Res<PlayerFactionId>,
    scene: Res<LiveStreamScene>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let faction = scene
        .faction_entries
        .iter()
        .find(|e| e.id == player.0)
        .cloned();

    // Counts derived from civilians that are tracked (no per-faction breakdown
    // in the wire protocol yet — civilian_entries lack faction_id).
    let total_civilians = scene.civilian_ids.len();

    egui::Window::new("Faction")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(12))
                .corner_radius(egui::CornerRadius::same(10)),
        )
        .show(ctx, |ui| {
            ui.set_min_width(200.0);

            // Header: faction colour swatch + name
            ui.horizontal(|ui| {
                let color = faction_egui_color(player.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, egui::CornerRadius::same(3), color);
                ui.label(
                    egui::RichText::new(faction_display_name(player.0, &faction))
                        .color(ACCENT)
                        .strong()
                        .size(14.0),
                );
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if let Some(ref entry) = faction {
                stat_row(ui, "Era", &entry.era.to_string(), GOLD);
                stat_row(ui, "Government", government_label(&entry.government), DIM);
                let treasury_label = if entry.treasury.currency.is_empty() {
                    format!("{:.0}", entry.treasury.amount)
                } else {
                    format!("{:.0} {}", entry.treasury.amount, entry.treasury.currency)
                };
                stat_row(ui, "Treasury", &treasury_label, GREEN);
            } else {
                ui.label(egui::RichText::new("Awaiting faction data...").color(DIM).italics());
            }

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);

            // Population row: total civilian count from live stream (faction breakdown unavailable)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Population").color(DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format_count(total_civilians)).strong());
                });
            });

            // Faction count (all factions observed)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Rival factions").color(DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let rivals = scene.faction_entries.len().saturating_sub(1);
                    ui.label(egui::RichText::new(rivals.to_string()).strong());
                });
            });

            ui.add_space(4.0);
            ui.label(egui::RichText::new("[F] to hide").color(DIM).small().italics());
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn faction_display_name(
    id: u32,
    entry: &Option<civ_protocol_3d::FactionStateEntry>,
) -> String {
    let gov = entry
        .as_ref()
        .map(|e| government_label(&e.government))
        .unwrap_or("Faction");
    format!("{gov} #{id}")
}

fn government_label(government: &civ_protocol_3d::Government3d) -> &'static str {
    use civ_protocol_3d::Government3d;
    match government {
        Government3d::Unknown => "Faction",
        Government3d::Monarchy => "Monarchy",
        Government3d::Republic => "Republic",
        Government3d::Theocracy => "Theocracy",
        Government3d::Junta => "Junta",
        Government3d::Council => "Council",
        Government3d::Corporate => "Corporate",
    }
}

fn faction_egui_color(id: u32) -> egui::Color32 {
    let [r, g, b] = crate::diplomacy_ui::faction_color_from_id(id);
    let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgb(to_u8(r), to_u8(g), to_u8(b))
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, value_color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).color(value_color).strong().small());
        });
    });
}

fn format_count(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}