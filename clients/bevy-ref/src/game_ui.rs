#![cfg(all(feature = "bevy", feature = "egui"))]

//! Bevy Egui gameplay HUD for the Civis reference client.
//!
//! This module keeps the HUD state isolated from the renderer binaries. The
//! UI is compile-gated behind the `egui` feature so `standalone.rs` stays
//! untouched. It renders an AAA-styled dark-glass shell modelled on the
//! `CIV-0300` RTS UI/UX spec:
//!
//! * **Top bar** — stat chips (tick / era / population / factions) + a global
//!   resource strip, with a grouped speed/time control on the right.
//! * **Bottom bar** — a god-game tool palette (Select, Inspect, Spawn life,
//!   Spawn structure, Terraform, Material, Disaster, Diplomacy, Policy) with
//!   icon+label buttons, active-lit state, and hover tooltips with hotkeys.
//! * **Right inspector** — a selection card with name/kind, group, attributes
//!   and a colour-coded health bar, plus an empty-state hint.
//! * **Left panel** — a faction/group list with colour swatches + counts and a
//!   reserved space above the minimap (drawn by `live_minimap.rs`).
//! * **Bottom-right** — left clear for the event feed toasts (`event_feed.rs`).
//! * A persistent help/hotkey hint line.
//!
//! Two hard constraints are intentional and must be preserved:
//! 1. Draw on [`EguiPrimaryContextPass`] — moving to `Update` panics in
//!    `bevy_egui`'s current schedule contract.
//! 2. The HUD is hidden entirely unless [`GameUiMode::Playing`] so menus,
//!    loading and the pause overlay own the screen alone.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use crate::menus::GameUiMode;
use crate::spawn_tools::{ActiveTool, SelectedEntity, SpawnTool};

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
// Palette + type scale
// ---------------------------------------------------------------------------

/// Accent cyan used for active widgets and highlights.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Gold accent (#E8B84B) for secondary highlights.
const GOLD: egui::Color32 = egui::Color32::from_rgb(232, 184, 75);
/// Friendly / positive green.
const GREEN: egui::Color32 = egui::Color32::from_rgb(120, 220, 130);
/// Warning / negative red.
const RED: egui::Color32 = egui::Color32::from_rgb(230, 96, 96);
/// Base glass panel fill (premultiplied for `const` construction; alpha ~232).
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(15, 18, 28, 232);
/// Slightly lighter fill used for chips and inactive tool buttons.
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(29, 35, 50, 235);
/// Deeper inset fill for nested cards / list rows.
const INSET_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(22, 27, 40, 235);
/// Dimmed label color for field names + secondary text.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
/// Border color for inactive widgets (subtle, low-contrast).
const BORDER: egui::Color32 = egui::Color32::from_rgb(54, 62, 82);
/// Faint hairline used for separators inside cards.
const HAIRLINE: egui::Color32 = egui::Color32::from_rgb(40, 47, 64);

/// Shared corner radius for the cohesive dark-glass look.
const RADIUS: u8 = 9;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that renders the gameplay HUD and binds keyboard speed shortcuts.
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<GameUiSnapshot>()
            .init_resource::<WorldResources>()
            .init_resource::<FactionRoster>()
            .init_resource::<SelectedEntityDetails>()
            .init_resource::<GameSpeed>()
            .add_systems(Update, handle_speed_shortcuts)
            // EguiPrimaryContextPass is REQUIRED: moving draw to Update panics.
            .add_systems(EguiPrimaryContextPass, draw_game_ui);
    }
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

/// Parameters bundle for the bottom-bar so the draw fn stays small.
struct BottomBarCtx<'a> {
    active: &'a mut ActiveTool,
    speed: &'a mut GameSpeed,
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
    ui_mode: Res<GameUiMode>,
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
        .frame(panel_frame(egui::Margin::symmetric(14, 8)))
        .show(ctx, |ui| {
            top_bar_ui(ui, &snapshot, &resources, &attach_mode, live_attach.as_deref());
        });

    egui::TopBottomPanel::bottom("civis_game_bottom_bar")
        .frame(panel_frame(egui::Margin::symmetric(14, 10)))
        .show(ctx, |ui| {
            let mut bottom = BottomBarCtx { active: &mut active_tool, speed: &mut speed };
            tool_palette_ui(ui, &mut bottom);
            ui.add_space(4.0);
            help_hint_ui(ui);
        });

    egui::SidePanel::left("civis_game_left_panel")
        .resizable(false)
        .exact_width(214.0)
        .frame(panel_frame(egui::Margin::same(12)))
        .show(ctx, |ui| faction_panel_ui(ui, &roster));

    // selected.0 is the Option<Entity> from spawn_tools::SelectedEntity.
    egui::SidePanel::right("civis_game_selected_panel")
        .resizable(true)
        .default_width(276.0)
        .frame(panel_frame(egui::Margin::same(14)))
        .show(ctx, |ui| inspector_ui(ui, selected.0.is_some(), &details));
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// Apply the cohesive dark-glass theme + typography to the egui context.
fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    let r = egui::CornerRadius::same(RADIUS);

    v.panel_fill = PANEL_FILL;
    v.window_fill = PANEL_FILL;
    v.window_corner_radius = r;
    v.window_stroke = egui::Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.corner_radius = r;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, HAIRLINE);
    v.widgets.inactive.corner_radius = r;
    v.widgets.inactive.bg_fill = CHIP_FILL;
    v.widgets.inactive.weak_bg_fill = CHIP_FILL;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    v.widgets.hovered.corner_radius = r;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, ACCENT);
    v.widgets.hovered.bg_fill = CHIP_FILL.gamma_multiply(1.3);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.7));
    v.widgets.active.corner_radius = r;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, ACCENT);
    v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    // Drop-shadow feel under floating windows / panels.
    v.window_shadow = egui::epaint::Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: egui::Color32::from_black_alpha(120),
    };
    style.visuals = v;
    apply_type_scale(&mut style);
    ctx.set_style(style);
}

/// Readable heading/body/small type scale + generous spacing.
fn apply_type_scale(style: &mut egui::Style) {
    use egui::{FontFamily::Proportional, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, Proportional)),
        (TextStyle::Body, FontId::new(14.5, Proportional)),
        (TextStyle::Button, FontId::new(14.5, Proportional)),
        (TextStyle::Small, FontId::new(11.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, egui::FontFamily::Monospace)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(11.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
}

/// Shared rounded glass frame for the HUD panels.
fn panel_frame(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(RADIUS))
}

/// A faint hairline section separator used inside cards.
fn hairline(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter().hline(rect.x_range(), y, egui::Stroke::new(1.0, HAIRLINE));
    ui.add_space(6.0);
}

// ---------------------------------------------------------------------------
// Top bar
// ---------------------------------------------------------------------------

/// A single rounded stat chip: `icon text` on a tinted pill.
fn chip(ui: &mut egui::Ui, icon: &str, text: &str, color: egui::Color32) {
    egui::Frame::NONE
        .fill(CHIP_FILL)
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(egui::RichText::new(text).color(color).strong());
        });
}

/// A resource chip with a per-tick delta arrow (↑ green / ↓ red / → grey).
fn resource_chip(ui: &mut egui::Ui, icon: &str, value: &str, delta: f64, color: egui::Color32) {
    let (arrow, dcol) = if delta > 0.0 {
        ("\u{2191}", GREEN)
    } else if delta < 0.0 {
        ("\u{2193}", RED)
    } else {
        ("\u{2192}", DIM)
    };
    egui::Frame::NONE
        .fill(INSET_FILL)
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(9, 5))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(egui::RichText::new(value).color(egui::Color32::WHITE).strong());
            ui.label(egui::RichText::new(format!("{arrow}{:+.0}", delta)).color(dcol).small());
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
        ui.label(egui::RichText::new("CIVIS").color(ACCENT).strong().size(17.0));
        ui.add_space(4.0);
        chip(ui, "\u{23f1}", &format!("Tick {}", snapshot.tick), ACCENT);
        chip(ui, "\u{1f30d}", &format!("Era {}", snapshot.era), GOLD);
        chip(ui, "\u{1f465}", &format!("{}", snapshot.population), GREEN);
        chip(ui, "\u{1f6a9}", &format!("{}", snapshot.factions), GOLD);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ws_status_ui(ui, snapshot, attach_mode, live_attach);
        });
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        resource_chip(ui, "\u{1f33e}", &compact(resources.food), resources.food_delta, GOLD);
        resource_chip(ui, "\u{2699}", &compact(resources.materials), resources.materials_delta, DIM);
        resource_chip(ui, "\u{26a1}", &compact(resources.energy), resources.energy_delta, ACCENT);
        resource_chip(ui, "\u{1f4b0}", &compact(resources.treasury), resources.treasury_delta, GOLD);
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
        ("\u{1f7e2}", "WS Live", GREEN)
    } else {
        ("\u{1f7e1}", "WS \u{2026}", GOLD)
    };
    chip(ui, dot, text, color);
    if let Some(overlay) = &snapshot.live_hud_overlay {
        ui.label(egui::RichText::new(overlay).color(DIM).small());
    }
}

/// Format a large number compactly (`12.3K`, `4.5M`) for resource chips.
fn compact(value: f64) -> String {
    let v = value.abs();
    if v >= 1.0e9 {
        format!("{:.1}B", value / 1.0e9)
    } else if v >= 1.0e6 {
        format!("{:.1}M", value / 1.0e6)
    } else if v >= 1.0e3 {
        format!("{:.1}K", value / 1.0e3)
    } else {
        format!("{:.0}", value)
    }
}

// ---------------------------------------------------------------------------
// Bottom tool palette + speed control
// ---------------------------------------------------------------------------

/// Definition of a single tool palette button.
struct ToolDef {
    icon: &'static str,
    label: &'static str,
    hotkey: &'static str,
    /// `Some` when the slot maps to a real `SpawnTool`; `None` = present-but-inert.
    tool: Option<SpawnTool>,
}

/// The 9-slot god-game palette. Slots without a `SpawnTool` variant are present
/// but inert (a no-op on click) until `spawn_tools.rs` grows the variant — kept
/// visible so the palette reads as the full design and wiring is one-line later.
const TOOLS: &[ToolDef] = &[
    ToolDef { icon: "\u{1f446}", label: "Select",    hotkey: "Q", tool: Some(SpawnTool::Select) },
    // Inspect reuses Select's pick behaviour for now (no Inspect variant yet).
    ToolDef { icon: "\u{1f50d}", label: "Inspect",   hotkey: "W", tool: Some(SpawnTool::Select) },
    ToolDef { icon: "\u{1f9cd}", label: "Life",      hotkey: "E", tool: Some(SpawnTool::SpawnCivilian) },
    ToolDef { icon: "\u{1f3db}", label: "Structure", hotkey: "R", tool: Some(SpawnTool::SpawnBuilding) },
    ToolDef { icon: "\u{26f0}",  label: "Terraform", hotkey: "T", tool: Some(SpawnTool::Terraform) },
    // Material / Disaster / Diplomacy / Policy have no SpawnTool variant yet.
    ToolDef { icon: "\u{1faa8}", label: "Material",  hotkey: "A", tool: None },
    ToolDef { icon: "\u{1f4a5}", label: "Disaster",  hotkey: "S", tool: Some(SpawnTool::Destroy) },
    ToolDef { icon: "\u{1f91d}", label: "Diplomacy", hotkey: "D", tool: None },
    ToolDef { icon: "\u{1f4dc}", label: "Policy",    hotkey: "F", tool: None },
];

/// Bottom bar: centred tool palette (left) + segmented speed control (right).
///
/// NOTE: if rasterised PNG tool icons land under `assets/ui/tool-icons/*.png`
/// they can be loaded as egui textures and swapped into `tool_button` in place
/// of the emoji glyph. Today only SVG sources exist (Bevy can't load SVG), so
/// the palette uses unicode glyph fallbacks.
fn tool_palette_ui(ui: &mut egui::Ui, ctx: &mut BottomBarCtx) {
    const BTN_W: f32 = 60.0;
    const GAP: f32 = 8.0;
    ui.horizontal(|ui| {
        let available = ui.available_width();
        let palette_w = TOOLS.len() as f32 * BTN_W + (TOOLS.len() as f32 - 1.0) * GAP;
        let right_w = 230.0;
        let left_pad = ((available - palette_w - right_w) * 0.5).max(0.0);
        ui.add_space(left_pad);
        for def in TOOLS {
            let is_active = def.tool.map(|t| t == ctx.active.tool).unwrap_or(false);
            if tool_button(ui, def, is_active).clicked() {
                if let Some(tool) = def.tool {
                    ctx.active.tool = tool; // Wire directly to ActiveTool.
                }
                // Inert slots (tool == None) are intentional no-ops for now.
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            speed_control_ui(ui, ctx.speed);
        });
    });
}

/// Render one 60x58 tool button (glyph + label), accent-lit when active.
fn tool_button(ui: &mut egui::Ui, def: &ToolDef, active: bool) -> egui::Response {
    let size = egui::vec2(60.0, 58.0);
    let (rect, mut resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let inert = def.tool.is_none();

    let fill = if active {
        ACCENT.gamma_multiply(0.30)
    } else if resp.hovered() {
        CHIP_FILL.gamma_multiply(1.4)
    } else {
        CHIP_FILL
    };
    let stroke = if active {
        egui::Stroke::new(2.0, ACCENT)
    } else if resp.hovered() {
        egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.6))
    } else {
        egui::Stroke::new(1.0, BORDER)
    };
    let p = ui.painter();
    p.rect_filled(rect, RADIUS as f32, fill);
    p.rect_stroke(rect, RADIUS as f32, stroke, egui::StrokeKind::Inside);

    let icon_color = if active {
        ACCENT
    } else if inert {
        DIM.gamma_multiply(0.85)
    } else {
        egui::Color32::WHITE
    };
    let icon_at = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.36);
    p.text(icon_at, egui::Align2::CENTER_CENTER, def.icon, egui::FontId::proportional(22.0), icon_color);
    let label_color = if active { ACCENT } else { DIM };
    let label_at = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.78);
    p.text(label_at, egui::Align2::CENTER_CENTER, def.label, egui::FontId::proportional(10.5), label_color);

    let tip = if inert {
        format!("{} [{}] — coming soon", def.label, def.hotkey)
    } else {
        format!("{} [{}]", def.label, def.hotkey)
    };
    resp = resp.on_hover_text(tip);
    resp
}

/// Segmented speed control: pause / 1x / 2x / 5x / 10x wired to GameSpeed.
fn speed_control_ui(ui: &mut egui::Ui, speed: &mut GameSpeed) {
    // Reversed because the parent layout is right_to_left.
    let steps = [(4u32, "10x"), (3, "5x"), (2, "2x"), (1, "1x"), (0, "\u{23f8}")];
    for (mult, label) in steps {
        let active = speed.multiplier == mult;
        let mut text = egui::RichText::new(label).size(13.0);
        text = if active { text.color(ACCENT).strong() } else { text.color(DIM) };
        let btn = egui::Button::new(text)
            .fill(if active { ACCENT.gamma_multiply(0.28) } else { CHIP_FILL })
            .stroke(if active {
                egui::Stroke::new(1.5, ACCENT)
            } else {
                egui::Stroke::new(1.0, BORDER)
            })
            .corner_radius(egui::CornerRadius::same(6))
            .min_size(egui::vec2(40.0, 34.0));
        if ui.add(btn).clicked() {
            speed.multiplier = mult;
        }
    }
    ui.label(egui::RichText::new("\u{23f5} Speed").color(DIM).small());
}

/// Persistent help / hotkey hint line under the palette.
fn help_hint_ui(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(
                "Space pause  \u{2022}  1-4 speed  \u{2022}  Q-F tools  \u{2022}  L event log  \u{2022}  Esc menu",
            )
            .color(DIM.gamma_multiply(0.85))
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
    ui.label(egui::RichText::new("\u{1f6a9} Factions").color(ACCENT).heading());
    ui.add_space(4.0);
    hairline(ui);

    if roster.factions.is_empty() {
        ui.label(egui::RichText::new("No factions yet.").color(DIM).small());
        ui.label(egui::RichText::new("Spawn life to seed groups.").color(DIM.gamma_multiply(0.8)).small());
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
        ui.label(egui::RichText::new("MINIMAP").color(DIM.gamma_multiply(0.7)).small());
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
    ui.label(egui::RichText::new("\u{25a4} Inspector").color(ACCENT).heading());
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
    fn compact_number_formatting() {
        assert_eq!(compact(0.0), "0");
        assert_eq!(compact(950.0), "950");
        assert_eq!(compact(12_300.0), "12.3K");
        assert_eq!(compact(4_500_000.0), "4.5M");
        assert_eq!(compact(2_000_000_000.0), "2.0B");
    }

    #[test]
    fn all_palette_tools_have_labels_and_hotkeys() {
        assert_eq!(TOOLS.len(), 9);
        for def in TOOLS {
            assert!(!def.label.is_empty());
            assert!(!def.hotkey.is_empty());
            assert!(!def.icon.is_empty());
        }
    }

    #[test]
    fn palette_includes_core_active_tools() {
        let mapped: Vec<SpawnTool> = TOOLS.iter().filter_map(|t| t.tool).collect();
        assert!(mapped.contains(&SpawnTool::Select));
        assert!(mapped.contains(&SpawnTool::SpawnCivilian));
        assert!(mapped.contains(&SpawnTool::SpawnBuilding));
        assert!(mapped.contains(&SpawnTool::Terraform));
        assert!(mapped.contains(&SpawnTool::Destroy));
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
}
