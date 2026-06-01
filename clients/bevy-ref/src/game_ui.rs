#![cfg(all(feature = "bevy", feature = "egui"))]

//! Bevy Egui gameplay HUD for the Civis reference client.
//!
//! This module keeps the HUD state isolated from the renderer binaries. The
//! UI is compile-gated behind the `egui` feature so `standalone.rs` stays
//! untouched. It renders an AAA-styled dark-glass shell with a **category-based
//! tool system** modelled on Cities Skylines (clean toolbar + flyout drawers),
//! WorldBox (chunky icon palette + sub-tool drawers), Empire at War (command
//! panels) and DINO:
//!
//! * **Top bar** — stat chips (tick / era / population / factions) + a global
//!   resource strip, with a grouped speed/time control on the right.
//! * **Bottom bar** — a *category* toolbar (Select, Life, Structure, Infra,
//!   Terraform, Material, Disaster, Diplomacy, Policy). Clicking a category
//!   opens a **flyout drawer** of sub-tools above it; picking a sub-tool sets
//!   the active tool. Active category + sub-tool are clearly lit; hover
//!   tooltips show labels + hotkeys.
//! * **Right inspector** — a selection card with name/kind, group, attributes
//!   and a colour-coded health bar, plus an empty-state hint.
//! * **Left panel** — a faction/group list with colour swatches + counts and a
//!   reserved space above the minimap (drawn by `live_minimap.rs`).
//! * **Bottom-right** — left clear for the event feed toasts (`event_feed.rs`).
//!
//! The theme primitives live in [`crate::ui_theme`] and the tool taxonomy in
//! [`crate::tool_categories`] so this file stays focused on layout + wiring.
//!
//! Two hard constraints are intentional and must be preserved:
//! 1. Draw on [`EguiPrimaryContextPass`] — moving to `Update` panics in
//!    `bevy_egui`'s current schedule contract.
//! 2. The HUD is hidden entirely unless [`GameUiMode::Playing`] so menus,
//!    loading and the pause overlay own the screen alone.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use crate::holo_minimap::HoloMinimapPlugin;
use crate::menus::GameUiMode;
use crate::spawn_tools::{ActiveTool, SelectedEntity};
use crate::tool_categories::{ActiveSubTool, Category, SubTool, CATEGORIES};
use crate::ui_theme::{
    accent_frame, apply_theme, compact, deck_chip, deck_rim_frame, hairline, inner_glow, top_sheen,
    DECK_AMBER, DECK_BORDER, DECK_GLASS, DECK_SUCCESS, DECK_TEXT, DECK_TEXT_MID, GOLD, GREEN, RED,
    INSET_FILL, RADIUS, RADIUS_BTN, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, BORDER, DIM,
    TEXT,
};

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Lightweight sim snapshot consumed by the HUD.
#[derive(Resource, Debug, Clone)]
pub struct GameUiSnapshot {
    /// Current simulation tick.
    pub tick: u64,
    /// Total population.
    pub population: u64,
    /// Number of factions.
    pub factions: u32,
    /// Current era label.
    pub era: String,
    /// Current tick speed multiplier.
    pub speed_multiplier: u32,
    /// Live attach scene stats line (`LiveHudSnapshot::format_overlay`) when in server mode.
    pub live_hud_overlay: Option<String>,
}

impl Default for GameUiSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 0,
            factions: 0,
            era: "0".to_string(),
            speed_multiplier: 1,
            live_hud_overlay: None,
        }
    }
}

impl GameUiSnapshot {
    /// Update the snapshot from live sim state.
    pub fn set_sim_state(
        &mut self,
        tick: u64,
        population: u64,
        factions: u32,
        era: impl Into<String>,
        speed_multiplier: u32,
    ) {
        self.tick = tick;
        self.population = population;
        self.factions = factions;
        self.era = era.into();
        self.speed_multiplier = speed_multiplier.max(1);
    }
}

/// Global resource totals shown in the top-bar resource strip.
///
/// `delta` fields are per-tick changes (rolling) used for the ↑/↓ arrows. The
/// sim bridge fills these in; defaults keep the strip readable before any tick.
#[derive(Resource, Debug, Clone, Default)]
pub struct WorldResources {
    /// Stored food units.
    pub food: f64,
    /// Per-tick change in food.
    pub food_delta: f64,
    /// Stored raw materials / production stock.
    pub materials: f64,
    /// Per-tick change in materials.
    pub materials_delta: f64,
    /// Stored energy (joules-equivalent).
    pub energy: f64,
    /// Per-tick change in energy.
    pub energy_delta: f64,
    /// Treasury / gold.
    pub treasury: f64,
    /// Per-tick change in treasury.
    pub treasury_delta: f64,
    /// Births recorded on the most recent tick.
    pub births: u32,
    /// Deaths recorded on the most recent tick.
    pub deaths: u32,
}

impl WorldResources {
    /// Replace the stored stocks with fresh values, deriving each `_delta`
    /// from the difference against the previous values. Births/deaths are
    /// per-tick counts straight from the sim snapshot.
    #[allow(clippy::too_many_arguments)]
    pub fn update_stocks(
        &mut self,
        food: f64,
        materials: f64,
        energy: f64,
        treasury: f64,
        births: u32,
        deaths: u32,
    ) {
        self.food_delta = food - self.food;
        self.materials_delta = materials - self.materials;
        self.energy_delta = energy - self.energy;
        self.treasury_delta = treasury - self.treasury;
        self.food = food;
        self.materials = materials;
        self.energy = energy;
        self.treasury = treasury;
        self.births = births;
        self.deaths = deaths;
    }
}

/// A single faction/group row for the left panel list.
#[derive(Debug, Clone)]
pub struct FactionInfo {
    /// Display name.
    pub name: String,
    /// Member / population count.
    pub count: u64,
    /// Swatch colour (sRGB 0..=255).
    pub color: [u8; 3],
}

/// The faction/group roster shown in the left panel.
#[derive(Resource, Debug, Clone, Default)]
pub struct FactionRoster {
    /// Ordered faction rows.
    pub factions: Vec<FactionInfo>,
}

impl FactionInfo {
    /// Build a roster row for an emergent cluster from its stable id + size.
    ///
    /// The colour is derived deterministically from the cluster id so a given
    /// settlement keeps the same swatch across ticks; the label reads
    /// "Settlement N" (or "Unaffiliated" for the id-0 catch-all bucket).
    pub fn from_cluster(cluster_id: u64, count: u64) -> Self {
        let name = if cluster_id == 0 {
            "Unaffiliated".to_string()
        } else {
            format!("Settlement {cluster_id}")
        };
        Self {
            name,
            count,
            color: cluster_color(cluster_id),
        }
    }
}

/// Deterministic sRGB swatch for an emergent cluster id (golden-angle hue).
fn cluster_color(cluster_id: u64) -> [u8; 3] {
    let hue = (cluster_id.wrapping_mul(47) % 360) as f32;
    let c = egui::ecolor::Hsva::new(hue / 360.0, 0.62, 0.82, 1.0).to_srgb();
    [c[0], c[1], c[2]]
}

/// Display details for the selected entity, populated by spawn_tools on Select.
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedEntityDetails {
    /// Name shown in the right panel.
    pub name: String,
    /// Entity kind/category ("Civilian", "Structure", …).
    pub kind: String,
    /// Faction label shown in the right panel.
    pub faction: String,
    /// Health shown in the right panel.
    pub health: String,
    /// Profession shown in the right panel.
    pub profession: String,
    /// World position shown in the right panel.
    pub position: String,
}

/// Tick speed resource used by the HUD controls.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameSpeed {
    /// Tick speed multiplier. `0` means paused.
    pub multiplier: u32,
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self { multiplier: 1 }
    }
}

impl GameSpeed {
    /// Human-readable speed label. Retained as the canonical multiplier→label
    /// mapping (covered by tests) and consumed by external HUD/log overlays.
    #[allow(dead_code)]
    fn display_label(self) -> String {
        match self.multiplier {
            0 => "Paused".to_string(),
            1 => "1x".to_string(),
            2 => "2x".to_string(),
            3 => "5x".to_string(),
            4 => "10x".to_string(),
            value => format!("{value}x"),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that renders the gameplay HUD and binds keyboard speed shortcuts.
pub struct GameUiPlugin;

/// Rasterized PNG tool-icon textures registered with egui, keyed by the
/// category [`Category::icon_key`] stem. Empty until [`load_tool_icons`] has run;
/// missing keys fall back to the unicode glyph in [`paint_icon_label`].
#[derive(Resource, Default)]
pub struct ToolIcons {
    /// Bevy image handles (kept alive so the textures are not unloaded).
    handles: Vec<Handle<Image>>,
    /// Map of icon-key → egui texture id, populated once images are loaded.
    ids: std::collections::HashMap<&'static str, egui::TextureId>,
    /// True once registration with egui has completed.
    registered: bool,
}

/// Icon-key → asset path under the crate `assets/` root.
const TOOL_ICON_PATHS: &[(&str, &str)] = &[
    ("select", "ui/tool-icons/select.png"),
    ("spawn-life", "ui/tool-icons/spawn-life.png"),
    ("spawn-structure", "ui/tool-icons/spawn-structure.png"),
    ("terraform", "ui/tool-icons/terraform.png"),
    ("spawn-material", "ui/tool-icons/spawn-material.png"),
    ("disaster", "ui/tool-icons/disaster.png"),
    ("diplomacy", "ui/tool-icons/diplomacy.png"),
    ("policy", "ui/tool-icons/policy.png"),
];

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(HoloMinimapPlugin)
            .init_resource::<GameUiSnapshot>()
            .init_resource::<WorldResources>()
            .init_resource::<FactionRoster>()
            .init_resource::<SelectedEntityDetails>()
            .init_resource::<GameSpeed>()
            .init_resource::<ActiveSubTool>()
            .init_resource::<ToolIcons>()
            .add_systems(Startup, queue_tool_icon_handles)
            .add_systems(Update, (handle_speed_shortcuts, handle_category_hotkeys))
            // EguiPrimaryContextPass is REQUIRED: moving draw to Update panics.
            // `load_tool_icons` registers the PNGs as egui textures and must run
            // before `draw_game_ui` consumes them.
            .add_systems(
                EguiPrimaryContextPass,
                (load_tool_icons, draw_game_ui).chain(),
            );
    }
}

/// Startup: queue each tool-icon PNG on the [`AssetServer`].
fn queue_tool_icon_handles(mut icons: ResMut<ToolIcons>, asset_server: Res<AssetServer>) {
    icons.handles = TOOL_ICON_PATHS
        .iter()
        .map(|(_, path)| asset_server.load::<Image>(*path))
        .collect();
}

/// Register the loaded tool-icon images with egui (once), storing the resulting
/// [`egui::TextureId`]s in [`ToolIcons`]. No-op after the first successful pass.
fn load_tool_icons(
    mut contexts: EguiContexts,
    mut icons: ResMut<ToolIcons>,
    asset_server: Res<AssetServer>,
) {
    if icons.registered {
        return;
    }
    // Only register once every image has finished loading, so add_image gets a
    // valid GPU texture rather than a placeholder.
    let all_loaded = icons
        .handles
        .iter()
        .all(|h| asset_server.is_loaded_with_dependencies(h));
    if icons.handles.is_empty() || !all_loaded {
        return;
    }
    let handles = icons.handles.clone();
    for ((key, _), handle) in TOOL_ICON_PATHS.iter().zip(handles) {
        // egui keeps a strong handle; our `ToolIcons.handles` also retains one so
        // the image is never unloaded for the lifetime of the app.
        let id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle));
        icons.ids.insert(key, id);
    }
    icons.registered = true;
}

fn handle_speed_shortcuts(keys: Res<ButtonInput<KeyCode>>, mut speed: ResMut<GameSpeed>) {
    if keys.just_pressed(KeyCode::Space) {
        speed.multiplier = if speed.multiplier == 0 { 1 } else { 0 };
    }
    for (key, mult) in [
        (KeyCode::Digit1, 1u32),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
    ] {
        if keys.just_pressed(key) {
            speed.multiplier = mult;
        }
    }
}

/// Toggle a category flyout open when its hotkey letter is pressed.
fn handle_category_hotkeys(keys: Res<ButtonInput<KeyCode>>, mut sub: ResMut<ActiveSubTool>) {
    for (idx, cat) in CATEGORIES.iter().enumerate() {
        if let Some(code) = hotkey_to_code(cat.hotkey) {
            if keys.just_pressed(code) {
                sub.open_category = if sub.open_category == Some(idx) { None } else { Some(idx) };
            }
        }
    }
}

/// Map a single-letter hotkey to its [`KeyCode`] (letters used by categories).
fn hotkey_to_code(hotkey: &str) -> Option<KeyCode> {
    match hotkey {
        "Q" => Some(KeyCode::KeyQ),
        "E" => Some(KeyCode::KeyE),
        "R" => Some(KeyCode::KeyR),
        "C" => Some(KeyCode::KeyC),
        "T" => Some(KeyCode::KeyT),
        "A" => Some(KeyCode::KeyA),
        "X" => Some(KeyCode::KeyX),
        "D" => Some(KeyCode::KeyD),
        "F" => Some(KeyCode::KeyF),
        _ => None,
    }
}

/// Parameters bundle for the bottom-bar so the draw fn stays small.
struct BottomBarCtx<'a> {
    active: &'a mut ActiveTool,
    sub: &'a mut ActiveSubTool,
    speed: &'a mut GameSpeed,
    icons: &'a std::collections::HashMap<&'static str, egui::TextureId>,
}

#[allow(clippy::too_many_arguments)]
fn draw_game_ui(
    mut contexts: EguiContexts,
    snapshot: Res<GameUiSnapshot>,
    resources: Res<WorldResources>,
    roster: Res<FactionRoster>,
    // Use spawn_tools::SelectedEntity (tuple struct) as the source of truth.
    selected: Res<SelectedEntity>,
    details: Res<SelectedEntityDetails>,
    attach_mode: Res<crate::AttachMode>,
    live_attach: Option<Res<crate::live_attach::LiveAttachState>>,
    mut speed: ResMut<GameSpeed>,
    mut active_tool: ResMut<ActiveTool>,
    mut sub_tool: ResMut<ActiveSubTool>,
    ui_mode: Res<GameUiMode>,
    tool_icons: Res<ToolIcons>,
) {
    // Hide HUD entirely when not in Playing mode (pause overlay, loading, etc.).
    if *ui_mode != GameUiMode::Playing {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    apply_theme(ctx);

    egui::TopBottomPanel::top("civis_game_top_bar")
        .frame(deck_rim_frame(egui::Margin::symmetric(SPACE_LG as i8, SPACE_SM as i8)))
        .show(ctx, |ui| {
            top_bar_ui(ui, &snapshot, &resources, &attach_mode, live_attach.as_deref());
            top_sheen(ui.painter(), ui.min_rect());
        });

    egui::TopBottomPanel::bottom("civis_game_bottom_bar")
        .frame(deck_rim_frame(egui::Margin::symmetric(SPACE_LG as i8, SPACE_MD as i8)))
        .show(ctx, |ui| {
            let mut bottom = BottomBarCtx {
                active: &mut active_tool,
                sub: &mut sub_tool,
                speed: &mut speed,
                icons: &tool_icons.ids,
            };
            category_bar_ui(ui, &mut bottom);
            ui.add_space(SPACE_XS);
            help_hint_ui(ui);
            top_sheen(ui.painter(), ui.min_rect());
        });

    egui::SidePanel::left("civis_game_left_panel")
        .resizable(false)
        .exact_width(214.0)
        .frame(deck_rim_frame(egui::Margin::same(SPACE_MD as i8)))
        .show(ctx, |ui| faction_panel_ui(ui, &roster));

    // selected.0 is the Option<Entity> from spawn_tools::SelectedEntity.
    egui::SidePanel::right("civis_game_selected_panel")
        .resizable(true)
        .default_width(276.0)
        .frame(deck_rim_frame(egui::Margin::same(SPACE_LG as i8)))
        .show(ctx, |ui| inspector_ui(ui, selected.0.is_some(), &details));
}

// ---------------------------------------------------------------------------
// Top bar
// ---------------------------------------------------------------------------

/// A resource chip with a per-tick delta arrow (↑ success / ↓ red / → dim).
fn resource_chip(ui: &mut egui::Ui, icon: &str, value: &str, delta: f64, color: egui::Color32) {
    let (arrow, dcol) = if delta > 0.0 {
        ("\u{2191}", DECK_SUCCESS)
    } else if delta < 0.0 {
        ("\u{2193}", RED)
    } else {
        ("\u{2192}", DECK_TEXT_MID)
    };
    egui::Frame::NONE
        .fill(DECK_GLASS.gamma_multiply(0.85))
        .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .inner_margin(egui::Margin::symmetric(SPACE_MD as i8, SPACE_XS as i8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(egui::RichText::new(value).monospace().color(DECK_TEXT).strong());
            ui.label(
                egui::RichText::new(format!("{arrow}{:+.0}", delta))
                    .color(dcol)
                    .small()
                    .monospace(),
            );
        });
}

/// Top bar: identity + stat chips, resource strip, and right-aligned WS status.
fn top_bar_ui(
    ui: &mut egui::Ui,
    snapshot: &GameUiSnapshot,
    resources: &WorldResources,
    attach_mode: &crate::AttachMode,
    live_attach: Option<&crate::live_attach::LiveAttachState>,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("CIVIS")
                .color(DECK_AMBER)
                .strong()
                .size(17.0),
        );
        ui.add_space(SPACE_XS);
        deck_chip(ui, "tick", &snapshot.tick.to_string(), DECK_AMBER);
        deck_chip(ui, "era", &snapshot.era, DECK_AMBER);
        deck_chip(ui, "pop", &snapshot.population.to_string(), DECK_SUCCESS);
        deck_chip(ui, "factions", &snapshot.factions.to_string(), DECK_AMBER);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ws_status_ui(ui, snapshot, attach_mode, live_attach);
        });
    });
    ui.add_space(SPACE_SM);
    ui.horizontal(|ui| {
        resource_chip(ui, "\u{1f33e}", &compact(resources.food), resources.food_delta, DECK_AMBER);
        resource_chip(
            ui,
            "\u{2699}",
            &compact(resources.materials),
            resources.materials_delta,
            DECK_TEXT_MID,
        );
        resource_chip(
            ui,
            "\u{26a1}",
            &compact(resources.energy),
            resources.energy_delta,
            DECK_SUCCESS,
        );
        resource_chip(
            ui,
            "\u{1f4b0}",
            &compact(resources.treasury),
            resources.treasury_delta,
            DECK_AMBER,
        );
    });
}

/// Right-aligned WebSocket connection status chip (server attach mode only).
fn ws_status_ui(
    ui: &mut egui::Ui,
    snapshot: &GameUiSnapshot,
    attach_mode: &crate::AttachMode,
    live_attach: Option<&crate::live_attach::LiveAttachState>,
) {
    if *attach_mode != crate::AttachMode::Server {
        return;
    }
    let connected = live_attach.map(|s| s.connected).unwrap_or(false);
    let (dot, text, color) = if connected {
        ("\u{1f7e2}", "WS Live", DECK_SUCCESS)
    } else {
        ("\u{1f7e1}", "WS \u{2026}", DECK_AMBER)
    };
    deck_chip(ui, dot, text, color);
    if let Some(overlay) = &snapshot.live_hud_overlay {
        ui.label(egui::RichText::new(overlay).color(DECK_TEXT_MID).small().monospace());
    }
}

// ---------------------------------------------------------------------------
// Bottom category toolbar + flyout drawers
// ---------------------------------------------------------------------------

/// Bottom bar: a flyout drawer (when a category is open) above a centred
/// category toolbar, plus a right-aligned segmented speed control.
fn category_bar_ui(ui: &mut egui::Ui, ctx: &mut BottomBarCtx) {
    // Draw the open flyout first so it stacks above the toolbar row.
    if let Some(idx) = ctx.sub.open_category {
        if let Some(cat) = CATEGORIES.get(idx) {
            flyout_drawer_ui(ui, cat, ctx);
            ui.add_space(6.0);
        }
    }
    category_toolbar_ui(ui, ctx);
}

/// The centred row of top-level category buttons + speed control on the right.
fn category_toolbar_ui(ui: &mut egui::Ui, ctx: &mut BottomBarCtx) {
    const BTN_W: f32 = 64.0;
    const GAP: f32 = 8.0;
    ui.horizontal(|ui| {
        let available = ui.available_width();
        let bar_w = CATEGORIES.len() as f32 * (BTN_W + GAP);
        let right_w = 240.0;
        let left_pad = ((available - bar_w - right_w) * 0.5).max(0.0);
        ui.add_space(left_pad);
        let active_cat = ctx.sub.active_category();
        for (idx, cat) in CATEGORIES.iter().enumerate() {
            let is_open = ctx.sub.open_category == Some(idx);
            let is_active = active_cat == Some(idx);
            let icon_tex = cat.icon_key().and_then(|k| ctx.icons.get(k).copied());
            if category_button(ui, cat, is_active, is_open, icon_tex).clicked() {
                ctx.sub.open_category = if is_open { None } else { Some(idx) };
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            speed_control_ui(ui, ctx.speed);
        });
    });
}

/// Render one 64x60 category button (PNG icon or glyph + label), lit when active/open.
fn category_button(
    ui: &mut egui::Ui,
    cat: &Category,
    active: bool,
    open: bool,
    icon_tex: Option<egui::TextureId>,
) -> egui::Response {
    let size = egui::vec2(64.0, 60.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let lit = active || open;
    let fill = if lit {
        DECK_AMBER.gamma_multiply(0.22)
    } else if resp.hovered() {
        DECK_GLASS.gamma_multiply(1.05)
    } else {
        DECK_GLASS.gamma_multiply(0.92)
    };
    let stroke = if lit {
        egui::Stroke::new(1.5, DECK_AMBER)
    } else if resp.hovered() {
        egui::Stroke::new(1.0, DECK_AMBER.gamma_multiply(0.55))
    } else {
        egui::Stroke::new(1.0, DECK_BORDER)
    };
    let p = ui.painter();
    p.rect_filled(rect, RADIUS_BTN as f32, fill);
    p.rect_stroke(rect, RADIUS_BTN as f32, stroke, egui::StrokeKind::Inside);
    if lit {
        inner_glow(p, rect, DECK_AMBER, RADIUS_BTN);
    }
    let accent = if lit { DECK_AMBER } else { cat.accent };
    paint_icon_label(p, rect, cat.icon, cat.label, lit, accent, icon_tex);
    // A small caret marks that the slot opens a flyout drawer.
    let caret = rect.center_top() + egui::vec2(0.0, 4.0);
    let caret_col = if open { DECK_AMBER } else { DECK_TEXT_MID.gamma_multiply(0.7) };
    p.text(caret, egui::Align2::CENTER_TOP, "\u{25be}", egui::FontId::proportional(9.0), caret_col);
    resp.on_hover_text(format!("{} \u{25b8}  [{}]", cat.label, cat.hotkey))
}

/// Paint a centred icon + caption inside `rect` (shared by category/sub-tool).
///
/// When `icon_tex` is `Some`, the rasterized PNG tool-icon is drawn; otherwise it
/// falls back to the unicode `icon` glyph so the toolbar is never empty.
fn paint_icon_label(
    p: &egui::Painter,
    rect: egui::Rect,
    icon: &str,
    label: &str,
    lit: bool,
    accent: egui::Color32,
    icon_tex: Option<egui::TextureId>,
) {
    let icon_color = if lit { accent } else { DECK_TEXT };
    let icon_at = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.40);
    if let Some(tex) = icon_tex {
        // Draw the PNG centred on the icon anchor; tint white when lit for a
        // subtle highlight, otherwise near-full opacity for an inert look.
        let side = 26.0_f32;
        let img_rect = egui::Rect::from_center_size(icon_at, egui::vec2(side, side));
        let tint = if lit {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_white_alpha(220)
        };
        p.image(tex, img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), tint);
    } else {
        p.text(icon_at, egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(22.0), icon_color);
    }
    let label_color = if lit { accent } else { DECK_TEXT_MID };
    let label_at = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.80);
    p.text(label_at, egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(10.5), label_color);
}

/// The flyout drawer: a framed panel of sub-tool buttons for the open category.
fn flyout_drawer_ui(ui: &mut egui::Ui, cat: &Category, ctx: &mut BottomBarCtx) {
    accent_frame(egui::Margin::symmetric(12, 9), cat.accent).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{}  {}", cat.icon, cat.label)).color(cat.accent).strong());
            ui.label(egui::RichText::new(format!("\u{2022}  {} tools", cat.subtools.len())).color(DIM).small());
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for &st in cat.subtools {
                let is_active = ctx.sub.current == st;
                if subtool_button(ui, st, is_active, cat.accent).clicked() {
                    select_subtool(ctx, st);
                }
            }
        });
    });
}

/// Pick a sub-tool: set the UI-side current tool + sync the backing SpawnTool.
fn select_subtool(ctx: &mut BottomBarCtx, st: SubTool) {
    ctx.sub.current = st;
    if let Some(tool) = st.spawn_tool() {
        ctx.active.tool = tool; // keep spawn_tools::ActiveTool in lockstep
    }
    // Sub-tools without a backing variant are intentional no-ops until the
    // Infra Lead grows SpawnTool; the UI still lights them as the picked tool.
}

/// Render one 70x56 sub-tool button inside a flyout, lit when it is current.
fn subtool_button(ui: &mut egui::Ui, st: SubTool, active: bool, accent: egui::Color32) -> egui::Response {
    let size = egui::vec2(70.0, 56.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let inert = !st.is_active_capable();
    let fill = if active {
        accent.gamma_multiply(0.24)
    } else if resp.hovered() {
        DECK_GLASS.gamma_multiply(1.05)
    } else {
        DECK_GLASS.gamma_multiply(0.9)
    };
    let stroke = if active {
        egui::Stroke::new(1.5, accent)
    } else if resp.hovered() {
        egui::Stroke::new(1.0, DECK_AMBER.gamma_multiply(0.5))
    } else {
        egui::Stroke::new(1.0, DECK_BORDER)
    };
    let p = ui.painter();
    p.rect_filled(rect, RADIUS_SM as f32, fill);
    p.rect_stroke(rect, RADIUS_SM as f32, stroke, egui::StrokeKind::Inside);
    if active {
        inner_glow(p, rect, accent, RADIUS_SM);
    }
    let lit = active && !inert;
    // Sub-tools keep the unicode glyph (no per-sub-tool PNG set yet) — pass None.
    paint_icon_label(p, rect, st.icon(), st.label(), lit, accent, None);
    let tip = if inert {
        format!("{} — coming soon", st.label())
    } else {
        st.label().to_string()
    };
    resp.on_hover_text(tip)
}

/// Segmented speed control: pause / 1x / 2x / 5x / 10x wired to GameSpeed.
fn speed_control_ui(ui: &mut egui::Ui, speed: &mut GameSpeed) {
    // Reversed because the parent layout is right_to_left.
    let steps = [(4u32, "10x"), (3, "5x"), (2, "2x"), (1, "1x"), (0, "\u{23f8}")];
    for (mult, label) in steps {
        let active = speed.multiplier == mult;
        let mut text = egui::RichText::new(label).size(13.0).monospace();
        text = if active {
            text.color(DECK_AMBER).strong()
        } else {
            text.color(DECK_TEXT_MID)
        };
        let btn = egui::Button::new(text)
            .fill(if active {
                DECK_AMBER.gamma_multiply(0.22)
            } else {
                DECK_GLASS.gamma_multiply(0.9)
            })
            .stroke(if active {
                egui::Stroke::new(1.5, DECK_AMBER)
            } else {
                egui::Stroke::new(1.0, DECK_BORDER)
            })
            .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
            .min_size(egui::vec2(40.0, 34.0));
        if ui.add(btn).clicked() {
            speed.multiplier = mult;
        }
    }
    ui.label(egui::RichText::new("\u{23f5} Speed").color(DECK_TEXT_MID).small());
}

/// Persistent help / hotkey hint line under the toolbar.
fn help_hint_ui(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(
                "Space pause  \u{2022}  1-4 speed  \u{2022}  Q/E/R/C/T/A/X/D/F categories  \u{2022}  L event log  \u{2022}  Esc menu",
            )
            .color(DECK_TEXT_MID.gamma_multiply(0.85))
            .small(),
        );
    });
}

// ---------------------------------------------------------------------------
// Left faction / group panel
// ---------------------------------------------------------------------------

/// Left panel: faction/group roster with colour swatches + counts, then a
/// reserved minimap area (the minimap itself is drawn by `live_minimap.rs`).
fn faction_panel_ui(ui: &mut egui::Ui, roster: &FactionRoster) {
    ui.label(egui::RichText::new("\u{1f6a9} Factions").color(DECK_AMBER).heading());
    ui.add_space(4.0);
    hairline(ui);

    if roster.factions.is_empty() {
        ui.label(egui::RichText::new("No factions yet.").color(DECK_TEXT_MID).small());
        ui.label(
            egui::RichText::new("Spawn life to seed groups.")
                .color(DECK_TEXT_MID.gamma_multiply(0.8))
                .small(),
        );
    } else {
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 150.0)
            .show(ui, |ui| {
                for faction in &roster.factions {
                    faction_row(ui, faction);
                }
            });
    }

    // Reserve space above the minimap so live_minimap.rs has a clear anchor.
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        ui.add_space(140.0); // minimap footprint owned by live_minimap.rs
        ui.label(egui::RichText::new("MINIMAP").color(DECK_TEXT_MID.gamma_multiply(0.7)).small());
    });
}

/// One faction row: colour swatch, name, and right-aligned member count.
fn faction_row(ui: &mut egui::Ui, faction: &FactionInfo) {
    let swatch = egui::Color32::from_rgb(faction.color[0], faction.color[1], faction.color[2]);
    egui::Frame::NONE
        .fill(INSET_FILL)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, swatch);
                ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, BORDER), egui::StrokeKind::Inside);
                ui.label(egui::RichText::new(&faction.name).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(compact(faction.count as f64)).color(DIM));
                });
            });
        });
    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// Right inspector card
// ---------------------------------------------------------------------------

/// Right-side selection inspector card with empty-state fallback.
fn inspector_ui(ui: &mut egui::Ui, has_selection: bool, details: &SelectedEntityDetails) {
    ui.label(egui::RichText::new("\u{25a4} Inspector").color(DECK_AMBER).heading());
    ui.add_space(4.0);
    hairline(ui);

    if !has_selection {
        inspector_empty_state(ui);
        return;
    }

    let kind = if details.kind.is_empty() { "Entity" } else { &details.kind };
    ui.horizontal(|ui| {
        let name = if details.name.is_empty() { "Unnamed" } else { &details.name };
        ui.label(egui::RichText::new(name).strong().size(16.0));
    });
    ui.label(egui::RichText::new(kind).color(GOLD).small());
    ui.add_space(8.0);

    inspector_row(ui, "Group", &details.faction);
    inspector_row(ui, "Profession", &details.profession);
    inspector_row(ui, "Position", &details.position);
    ui.add_space(6.0);
    health_bar_ui(ui, &details.health);
}

/// Friendly empty state shown when nothing is selected.
fn inspector_empty_state(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("\u{1f9ed}").size(34.0).color(DIM.gamma_multiply(0.8)));
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Nothing selected").color(DIM).strong());
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Pick the Select tool and click an\nactor to inspect its details.")
                .color(DIM.gamma_multiply(0.8))
                .small(),
        );
    });
}

/// Health field rendered as a colour-coded progress bar when parseable.
fn health_bar_ui(ui: &mut egui::Ui, health: &str) {
    ui.label(egui::RichText::new("Health").color(DIM).small());
    match parse_health_fraction(health) {
        Some(frac) => {
            let color = if frac > 0.66 {
                GREEN
            } else if frac > 0.33 {
                GOLD
            } else {
                RED
            };
            ui.add(egui::ProgressBar::new(frac).fill(color).text(health.to_string()));
        }
        None => {
            let shown = if health.is_empty() { "—" } else { health };
            ui.label(egui::RichText::new(shown).strong());
        }
    }
}

/// A dimmed-label / bright-value inspector row.
fn inspector_row(ui: &mut egui::Ui, name: &str, value: &str) {
    let shown = if value.is_empty() { "—" } else { value };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(name).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(shown).strong());
        });
    });
}

/// Parse a health string into a 0..=1 fraction.
///
/// Accepts `"87"`, `"87%"`, `"87/100"`, or `"0.87"`; returns `None` otherwise.
fn parse_health_fraction(raw: &str) -> Option<f32> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Some((num, den)) = s.split_once('/') {
        let n: f32 = num.trim().parse().ok()?;
        let d: f32 = den.trim().parse().ok()?;
        if d <= 0.0 {
            return None;
        }
        return Some((n / d).clamp(0.0, 1.0));
    }
    if let Some(pct) = s.strip_suffix('%') {
        let v: f32 = pct.trim().parse().ok()?;
        return Some((v / 100.0).clamp(0.0, 1.0));
    }
    let v: f32 = s.parse().ok()?;
    if (0.0..=1.0).contains(&v) {
        Some(v)
    } else if (0.0..=100.0).contains(&v) {
        Some(v / 100.0)
    } else {
        Some(v.clamp(0.0, 1.0))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_label_mapping() {
        assert_eq!(GameSpeed { multiplier: 0 }.display_label(), "Paused");
        assert_eq!(GameSpeed { multiplier: 1 }.display_label(), "1x");
        assert_eq!(GameSpeed { multiplier: 2 }.display_label(), "2x");
        assert_eq!(GameSpeed { multiplier: 3 }.display_label(), "5x");
        assert_eq!(GameSpeed { multiplier: 4 }.display_label(), "10x");
        assert_eq!(GameSpeed { multiplier: 7 }.display_label(), "7x");
    }

    #[test]
    fn health_parse_variants() {
        assert_eq!(parse_health_fraction("87%"), Some(0.87));
        assert_eq!(parse_health_fraction("50/100"), Some(0.5));
        assert_eq!(parse_health_fraction("0.25"), Some(0.25));
        assert_eq!(parse_health_fraction("100"), Some(1.0));
        assert_eq!(parse_health_fraction("75"), Some(0.75));
        assert_eq!(parse_health_fraction(""), None);
        assert_eq!(parse_health_fraction("Healthy"), None);
        assert_eq!(parse_health_fraction("10/0"), None);
    }

    #[test]
    fn snapshot_set_sim_state_clamps_speed() {
        let mut snap = GameUiSnapshot::default();
        snap.set_sim_state(10, 20, 3, "Bronze", 0);
        assert_eq!(snap.tick, 10);
        assert_eq!(snap.population, 20);
        assert_eq!(snap.factions, 3);
        assert_eq!(snap.era, "Bronze");
        assert_eq!(snap.speed_multiplier, 1);
    }

    #[test]
    fn world_resources_default_is_zeroed() {
        let r = WorldResources::default();
        assert_eq!(r.food, 0.0);
        assert_eq!(r.treasury_delta, 0.0);
    }

    #[test]
    fn faction_roster_default_empty() {
        assert!(FactionRoster::default().factions.is_empty());
    }

    #[test]
    fn every_category_hotkey_maps_to_a_keycode() {
        for cat in CATEGORIES {
            assert!(hotkey_to_code(cat.hotkey).is_some(), "{} hotkey unmapped", cat.label);
        }
    }
}
