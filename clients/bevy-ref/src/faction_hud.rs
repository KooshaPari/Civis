#![cfg(all(feature = "bevy", feature = "egui"))]

//! Player faction HUD panel — shows the player-owned faction stats (top-left corner).
//!
//! Toggle with `F`. Panel reads `PlayerFactionId` to locate the entry in
//! `LiveStreamScene::faction_entries`, then displays government, era,
//! treasury, and live civilian count for that faction.
//!
//! Crest art: loads `assets/ui/faction-crests/crest-*.png` (rasterized from the
//! sibling SVGs) and registers them with egui the same way [`crate::game_ui`]
//! loads [`crate::game_ui::HudPanelAssets`] / tool icons.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_stream::LiveStreamScene;

use crate::ui_theme::{ACCENT, DIM, GOLD, GREEN, PANEL_FILL};

// CHIP_FILL: local tint not present in ui_theme (different from GRAPHITE_700)
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);

/// Header crest display size (logical px).
const CREST_SIZE: f32 = 32.0;

/// (stem, asset path) pairs for faction crest PNGs.
///
/// Index into this list with `faction_id % len` for a stable identity per id.
/// Order mirrors the world-glyph palette (gold → blue → violet → green → …).
const FACTION_CREST_PATHS: &[(&str, &str)] = &[
    ("gold", "ui/faction-crests/crest-gold.png"),
    ("blue", "ui/faction-crests/crest-blue.png"),
    ("violet", "ui/faction-crests/crest-violet.png"),
    ("green", "ui/faction-crests/crest-green.png"),
    ("red", "ui/faction-crests/crest-red.png"),
    ("cyan", "ui/faction-crests/crest-cyan.png"),
];

// ── Resources ────────────────────────────────────────────────────────────────

/// The faction id the local player controls (0 = Ardani, 1 = Velthari, 2 = Grundak).
///
/// Set from [`crate::menus::WorldSetupParams::player_faction`] on ConfirmWorldSetup.
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

/// Faction crest PNG handles + registered egui texture IDs.
#[derive(Resource, Default)]
pub struct FactionCrestAssets {
    /// Bevy strong handles keeping PNGs alive (one per [`FACTION_CREST_PATHS`] entry).
    pub handles: Vec<Handle<Image>>,
    /// Registered egui texture IDs keyed by crest stem (`"gold"`, `"blue"`, …).
    pub textures: HashMap<&'static str, egui::TextureId>,
    /// `true` once all crest images have been registered with egui.
    pub registered: bool,
}

impl FactionCrestAssets {
    /// Crest texture for a faction id, if registration has completed.
    #[must_use]
    pub fn texture_for_faction(&self, id: u32) -> Option<egui::TextureId> {
        let (key, _) = FACTION_CREST_PATHS[id as usize % FACTION_CREST_PATHS.len()];
        self.textures.get(key).copied()
    }
}

/// Crest stem key for a faction id (`"gold"`, `"blue"`, …).
#[must_use]
pub fn faction_crest_key(id: u32) -> &'static str {
    FACTION_CREST_PATHS[id as usize % FACTION_CREST_PATHS.len()].0
}

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Registers the player faction HUD panel.
pub struct FactionHudPlugin;

impl Plugin for FactionHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerFactionId>()
            .init_resource::<FactionHudOpen>()
            .init_resource::<FactionCrestAssets>()
            .add_systems(Startup, queue_faction_crest_handles)
            .add_systems(Update, toggle_faction_hud)
            // Register crests before draw (Bevy 0.18: avoid `.chain()` on 2-tuples).
            .add_systems(
                EguiPrimaryContextPass,
                load_faction_crests.before(draw_faction_hud),
            )
            .add_systems(EguiPrimaryContextPass, draw_faction_hud);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn queue_faction_crest_handles(mut crests: ResMut<FactionCrestAssets>, asset_server: Res<AssetServer>) {
    crests.handles = FACTION_CREST_PATHS
        .iter()
        .map(|(_, path)| asset_server.load::<Image>(*path))
        .collect();
}

/// Register crest PNGs with egui once every handle has finished loading.
fn load_faction_crests(
    mut contexts: EguiContexts,
    mut crests: ResMut<FactionCrestAssets>,
    asset_server: Res<AssetServer>,
) {
    if crests.registered {
        return;
    }
    let all_loaded = crests
        .handles
        .iter()
        .all(|h| asset_server.is_loaded_with_dependencies(h));
    if crests.handles.is_empty() || !all_loaded {
        return;
    }
    let handles = crests.handles.clone();
    for ((key, _), handle) in FACTION_CREST_PATHS.iter().zip(handles) {
        let id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle));
        crests.textures.insert(*key, id);
    }
    crests.registered = true;
}

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
    crests: Res<FactionCrestAssets>,
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
    let faction_population = scene
        .population_by_faction
        .get(&player.0)
        .copied()
        .unwrap_or(0);
    let total_civilians = scene.civilian_ids.len();
    let crest_tex = crests.texture_for_faction(player.0);

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

            // Header: faction crest (or colour swatch fallback) + name
            ui.horizontal(|ui| {
                if let Some(tex) = crest_tex {
                    ui.image((tex, egui::vec2(CREST_SIZE, CREST_SIZE)));
                } else {
                    let color = faction_egui_color(player.0);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(3), color);
                }
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(faction_display_name(player.0, &faction))
                            .color(ACCENT)
                            .strong()
                            .size(14.0),
                    );
                });
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
                ui.label(
                    egui::RichText::new("Awaiting faction data...")
                        .color(DIM)
                        .italics(),
                );
            }

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);

            // Population row: per-faction count from FactionState frame (FR-CIV-PROTO-001).
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Population").color(DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let pop_label = if faction_population > 0 {
                        format_count(faction_population as usize)
                    } else {
                        format!("~{}", format_count(total_civilians))
                    };
                    ui.label(egui::RichText::new(pop_label).strong());
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
            ui.label(
                egui::RichText::new("[F] to hide")
                    .color(DIM)
                    .small()
                    .italics(),
            );
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Human-readable identity for a faction id (shared seed names + numeric fallback).
///
/// Ids 0..=2 map to Ardani / Velthari / Grundak; higher ids fall back to
/// `"Faction {id}"`.
#[must_use]
pub fn faction_identity_name(id: u32) -> String {
    match id {
        0 => "Ardani".to_string(),
        1 => "Velthari".to_string(),
        2 => "Grundak".to_string(),
        _ => format!("Faction {id}"),
    }
}

fn faction_display_name(id: u32, _entry: &Option<civ_protocol_3d::FactionStateEntry>) -> String {
    faction_identity_name(id)
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

/// Crest-aligned swatch when PNG registration has not completed yet.
///
/// Order and hexes match [`FACTION_CREST_PATHS`] / `assets/ui/README.md`
/// (`gold` `#E8B84B`, `blue` `#2980b9`, `violet` `#9b59b6`, `green` `#27ae60`,
/// `red` `#c0392b`, `cyan` `#50C8F0`).
fn faction_egui_color(id: u32) -> egui::Color32 {
    const CREST_COLORS: [egui::Color32; 6] = [
        egui::Color32::from_rgb(0xE8, 0xB8, 0x4B), // gold
        egui::Color32::from_rgb(0x29, 0x80, 0xB9), // blue
        egui::Color32::from_rgb(0x9B, 0x59, 0xB6), // violet
        egui::Color32::from_rgb(0x27, 0xAE, 0x60), // green
        egui::Color32::from_rgb(0xC0, 0x39, 0x2B), // red
        egui::Color32::from_rgb(0x50, 0xC8, 0xF0), // cyan
    ];
    CREST_COLORS[id as usize % CREST_COLORS.len()]
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, value_color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .color(value_color)
                    .strong()
                    .small(),
            );
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
