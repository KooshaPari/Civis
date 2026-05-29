#![cfg(all(feature = "bevy", feature = "egui"))]

//! Bevy Egui gameplay HUD for the Civis reference client.
//!
//! This module keeps the HUD state isolated from the renderer binaries. The
//! UI is compile-gated behind the `egui` feature so `standalone.rs` stays
//! untouched. The HUD draws an AAA-styled glassmorphism shell: a stat-chip top
//! bar, a tool-palette + speed-control bottom bar, and a selection inspector.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use crate::menus::GameUiMode;
use crate::spawn_tools::{ActiveTool, SelectedEntity, SpawnTool};

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

/// Display details for the selected entity, populated by spawn_tools on Select.
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedEntityDetails {
    /// Name shown in the right panel.
    pub name: String,
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
// Palette
// ---------------------------------------------------------------------------

/// Accent cyan used for active widgets and highlights.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Gold accent for secondary highlights.
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);
/// Glassmorphism panel fill (premultiplied for `const` construction; alpha ~235).
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
/// Slightly lighter fill used for chips and inactive tool buttons.
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);
/// Dimmed label color for inspector field names.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
/// Border color for inactive tool buttons.
const BORDER_INACTIVE: egui::Color32 = egui::Color32::from_rgb(60, 68, 88);

/// Plugin that renders the gameplay HUD and binds keyboard speed shortcuts.
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<GameUiSnapshot>()
            .init_resource::<SelectedEntityDetails>()
            .init_resource::<GameSpeed>()
            .add_systems(Update, handle_speed_shortcuts)
            .add_systems(EguiPrimaryContextPass, draw_game_ui);
    }
}

fn handle_speed_shortcuts(keys: Res<ButtonInput<KeyCode>>, mut speed: ResMut<GameSpeed>) {
    if keys.just_pressed(KeyCode::Space) {
        speed.multiplier = if speed.multiplier == 0 { 1 } else { 0 };
    }
    if keys.just_pressed(KeyCode::Digit1) {
        speed.multiplier = 1;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        speed.multiplier = 2;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        speed.multiplier = 3;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        speed.multiplier = 4;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_game_ui(
    mut contexts: EguiContexts,
    snapshot: Res<GameUiSnapshot>,
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
        .frame(panel_frame(egui::Margin::symmetric(12, 8)))
        .show(ctx, |ui| {
            top_bar_ui(ui, &snapshot, &attach_mode, live_attach.as_deref());
        });

    egui::TopBottomPanel::bottom("civis_game_bottom_bar")
        .frame(panel_frame(egui::Margin::symmetric(12, 8)))
        .show(ctx, |ui| {
            tool_palette_ui(ui, &mut active_tool, &mut speed);
        });

    // selected.0 is the Option<Entity> from spawn_tools::SelectedEntity.
    if selected.0.is_some() {
        egui::SidePanel::right("civis_game_selected_panel")
            .resizable(true)
            .default_width(268.0)
            .frame(panel_frame(egui::Margin::same(14)))
            .show(ctx, |ui| {
                inspector_ui(ui, &details);
            });
    }
}

/// Apply the dark glassmorphism theme + typography to the egui context.
fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    let radius = egui::CornerRadius::same(8);

    v.panel_fill = PANEL_FILL;
    v.window_fill = PANEL_FILL;
    v.window_corner_radius = radius;
    v.widgets.noninteractive.corner_radius = radius;
    v.widgets.inactive.corner_radius = radius;
    v.widgets.inactive.bg_fill = CHIP_FILL;
    v.widgets.inactive.weak_bg_fill = CHIP_FILL;
    v.widgets.hovered.corner_radius = radius;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, ACCENT);
    v.widgets.hovered.bg_fill = CHIP_FILL.gamma_multiply(1.3);
    v.widgets.active.corner_radius = radius;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, ACCENT);
    v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals = v;

    use egui::{FontFamily::Proportional, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0, Proportional)),
        (TextStyle::Body, FontId::new(15.0, Proportional)),
        (TextStyle::Button, FontId::new(15.0, Proportional)),
        (TextStyle::Small, FontId::new(11.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, egui::FontFamily::Monospace)),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

/// Shared rounded glass frame for the HUD panels.
fn panel_frame(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .inner_margin(margin)
        .corner_radius(egui::CornerRadius::same(8))
}

/// A single rounded stat chip: `icon text` on a tinted pill.
fn chip(ui: &mut egui::Ui, icon: &str, text: &str, color: egui::Color32) {
    egui::Frame::NONE
        .fill(CHIP_FILL)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(egui::RichText::new(text).color(color).strong());
        });
}

/// Top bar: stat chips on the left, websocket status on the right.
fn top_bar_ui(
    ui: &mut egui::Ui,
    snapshot: &GameUiSnapshot,
    attach_mode: &crate::AttachMode,
    live_attach: Option<&crate::live_attach::LiveAttachState>,
) {
    let green = egui::Color32::from_rgb(120, 220, 130);
    let speed_label = GameSpeed {
        multiplier: snapshot.speed_multiplier,
    }
    .display_label();

    ui.horizontal(|ui| {
        chip(ui, "\u{23f1}", &format!("{}", snapshot.tick), ACCENT);
        chip(ui, "\u{1f465}", &format!("{}", snapshot.population), green);
        chip(ui, "\u{1f6a9}", &format!("{}", snapshot.factions), GOLD);
        chip(ui, "\u{1f30d}", &format!("Era {}", snapshot.era), egui::Color32::WHITE);
        chip(ui, "\u{25b6}", &speed_label, ACCENT);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if *attach_mode == crate::AttachMode::Server {
                let connected = live_attach.map(|s| s.connected).unwrap_or(false);
                let (dot, text, color) = if connected {
                    ("\u{1f7e2}", "WS Live", green)
                } else {
                    ("\u{1f7e1}", "WS \u{2026}", GOLD)
                };
                chip(ui, dot, text, color);
                if let Some(overlay) = &snapshot.live_hud_overlay {
                    ui.label(egui::RichText::new(overlay).color(DIM).small());
                }
            }
        });
    });
}

/// Definition of a single tool palette button.
struct ToolDef {
    icon: &'static str,
    label: &'static str,
    hotkey: &'static str,
    tool: Option<SpawnTool>,
}

/// Bottom bar: tool palette (left, centered) + segmented speed control (right).
fn tool_palette_ui(ui: &mut egui::Ui, active: &mut ActiveTool, speed: &mut GameSpeed) {
    let tools = [
        ToolDef { icon: "\u{1f446}", label: "Select",   hotkey: "Q", tool: Some(SpawnTool::Select) },
        ToolDef { icon: "\u{1f9cd}", label: "Spawn Civ", hotkey: "W", tool: Some(SpawnTool::SpawnCivilian) },
        ToolDef { icon: "\u{1f3e0}", label: "Building",  hotkey: "E", tool: Some(SpawnTool::SpawnBuilding) },
        ToolDef { icon: "\u{26f0}",  label: "Terraform", hotkey: "R", tool: Some(SpawnTool::Terraform) },
        ToolDef { icon: "\u{1f4a5}", label: "Destroy",   hotkey: "T", tool: Some(SpawnTool::Destroy) },
        // Weather has no SpawnTool variant yet: present but inert.
        ToolDef { icon: "\u{1f327}", label: "Weather",   hotkey: "Y", tool: None },
    ];

    ui.horizontal(|ui| {
        // Center the tool group by consuming available space around it.
        let available = ui.available_width();
        // 6 buttons * 64px wide + 5 gaps * 8px = 424px
        let palette_width = 6.0 * 64.0 + 5.0 * 8.0;
        // Right section: speed label + 5 buttons * 46px + 4 gaps.
        let right_width = 200.0;
        let left_pad = ((available - palette_width - right_width) * 0.5).max(0.0);
        ui.add_space(left_pad);

        for def in &tools {
            let is_active = def.tool.map(|t| t == active.tool).unwrap_or(false);
            if tool_button(ui, def, is_active).clicked() {
                if let Some(tool) = def.tool {
                    // Wire directly to the ActiveTool resource.
                    active.tool = tool;
                }
                // Weather (tool == None) is intentionally a no-op for now.
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            speed_control_ui(ui, speed);
        });
    });
}

/// Render one 64x56 tool button with emoji + label, accent-highlighted if active.
/// Returns a response that reports `.clicked()` correctly.
fn tool_button(ui: &mut egui::Ui, def: &ToolDef, active: bool) -> egui::Response {
    let desired_size = egui::vec2(64.0, 56.0);
    let (rect, mut resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // Hover / active tinting.
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
        egui::Stroke::new(1.0, BORDER_INACTIVE)
    };

    let painter = ui.painter();
    painter.rect_filled(rect, 8.0, fill);
    painter.rect_stroke(rect, 8.0, stroke, egui::StrokeKind::Inside);

    // Icon — large emoji centred in upper portion.
    let icon_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(0.0, 4.0),
        egui::vec2(rect.width(), rect.height() * 0.58),
    );
    let icon_color = if active { ACCENT } else { egui::Color32::WHITE };
    painter.text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        def.icon,
        egui::FontId::proportional(22.0),
        icon_color,
    );

    // Label — small text centred in lower portion.
    let label_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(0.0, rect.height() * 0.60),
        egui::vec2(rect.width(), rect.height() * 0.40),
    );
    let label_color = if active { ACCENT } else { DIM };
    painter.text(
        label_rect.center(),
        egui::Align2::CENTER_CENTER,
        def.label,
        egui::FontId::proportional(10.5),
        label_color,
    );

    // Tooltip with hotkey.
    resp = resp.on_hover_text(format!("{} [{}]", def.label, def.hotkey));
    resp
}

/// Segmented speed control: pause / 1x / 2x / 5x / 10x wired to GameSpeed.
fn speed_control_ui(ui: &mut egui::Ui, speed: &mut GameSpeed) {
    // Reversed because parent layout is right_to_left.
    let steps = [
        (4u32, "10x"),
        (3,    "5x"),
        (2,    "2x"),
        (1,    "1x"),
        (0,    "\u{23f8}"),
    ];
    for (mult, label) in steps {
        let active = speed.multiplier == mult;
        let mut text = egui::RichText::new(label).size(13.0);
        if active {
            text = text.color(ACCENT).strong();
        } else {
            text = text.color(DIM);
        }
        let btn = egui::Button::new(text)
            .fill(if active { ACCENT.gamma_multiply(0.28) } else { CHIP_FILL })
            .stroke(if active {
                egui::Stroke::new(1.5, ACCENT)
            } else {
                egui::Stroke::new(1.0, BORDER_INACTIVE)
            })
            .corner_radius(egui::CornerRadius::same(6))
            .min_size(egui::vec2(38.0, 32.0));
        if ui.add(btn).clicked() {
            speed.multiplier = mult;
        }
    }
    ui.label(egui::RichText::new("Speed").color(DIM).small());
}

/// Right-side selection inspector card.
fn inspector_ui(ui: &mut egui::Ui, details: &SelectedEntityDetails) {
    ui.heading(egui::RichText::new("\u{25a4} Selection").color(ACCENT));
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);

    inspector_row(ui, "Name", &details.name);
    inspector_row(ui, "Faction", &details.faction);

    // Health rendered as a progress bar when it parses to a fraction.
    ui.add_space(2.0);
    ui.label(egui::RichText::new("Health").color(DIM).small());
    if let Some(frac) = parse_health_fraction(&details.health) {
        let color = if frac > 0.66 {
            egui::Color32::from_rgb(120, 220, 130)
        } else if frac > 0.33 {
            egui::Color32::from_rgb(240, 200, 90)
        } else {
            egui::Color32::from_rgb(230, 90, 90)
        };
        ui.add(
            egui::ProgressBar::new(frac)
                .fill(color)
                .text(details.health.clone()),
        );
    } else {
        ui.label(egui::RichText::new(&details.health).strong());
    }
    ui.add_space(2.0);

    inspector_row(ui, "Profession", &details.profession);
    inspector_row(ui, "Position", &details.position);
}

/// A dimmed-label / bright-value inspector row.
fn inspector_row(ui: &mut egui::Ui, name: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(name).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).strong());
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
}
