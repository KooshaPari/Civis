#![cfg(all(feature = "bevy", feature = "egui"))]

//! Bevy Egui gameplay HUD for the Civis reference client.
//!
//! This module keeps the HUD state isolated from the renderer binaries. The
//! UI is compile-gated behind the `egui` feature so `standalone.rs` stays
//! untouched. The HUD draws an AAA-styled glassmorphism shell: a stat-chip top
//! bar, a tool-palette + speed-control bottom bar, and a selection inspector.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use crate::ui_theme::CHIP_FILL;

use civ_protocol_3d::{CivilianNeeds3d, CivilianStateEntry};

use crate::game_laws::GameLawsOpen;
use crate::live_pick::LiveSelection;
use crate::spawn_tools::{ActiveTool, BuildingSpawnKind, SpawnTool};
use crate::{AttachMode, LiveEntityKind, SelectedLiveEntity};
use crate::settings_ui::{
    ACTION_CYCLE_SIM_SPEED, ACTION_PAUSE_SIM, ACTION_SPEED_1X, ACTION_SPEED_10X, ACTION_SPEED_2X,
    ACTION_SPEED_5X, GameSettings, KeyBinding,
};

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
    pub speed_multiplier: f32,
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
            speed_multiplier: 1.0,
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
        speed_multiplier: f32,
    ) {
        self.tick = tick;
        self.population = population;
        self.factions = factions;
        self.era = era.into();
        self.speed_multiplier = speed_multiplier;
    }
}

/// Tracks the currently selected entity in the HUD.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectedEntity {
    /// Selected Bevy entity, if any.
    pub entity: Option<Entity>,
}

/// Display details for the selected entity.
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
    /// Species label shown in the right panel.
    pub species: String,
    /// Needs summary (food / shelter / safety / social / rest).
    pub needs: String,
    /// World position shown in the right panel.
    pub position: String,
}

/// Tick speed resource used by the HUD controls.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct GameSpeed {
    /// Tick speed multiplier. `0` means paused.
    pub multiplier: f32,
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

impl GameSpeed {
    fn display_label(self) -> String {
        match self.multiplier {
            value if speed_value_matches(value, 0.0) => "Paused".to_string(),
            value if speed_value_matches(value, 1.0) => "1x".to_string(),
            value if speed_value_matches(value, 2.0) => "2x".to_string(),
            value if speed_value_matches(value, 5.0) => "5x".to_string(),
            value if speed_value_matches(value, 10.0) => "10x".to_string(),
            value if speed_value_matches(value.fract(), 0.0) => format!("{value:.0}x"),
            value => format!("{value:.2}x"),
        }
    }

    pub const fn factor(self) -> f32 {
        self.multiplier
    }
}

fn speed_value_matches(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 0.01
}

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

/// Accent cyan used for active widgets and highlights.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Glassmorphism panel fill (premultiplied for `const` construction; alpha ~235).
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
/// Dimmed label color for inspector field names.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);

/// Plugin that renders the gameplay HUD and binds keyboard speed shortcuts.
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<GameLawsOpen>()
            .init_resource::<GameUiSnapshot>()
            .init_resource::<SelectedEntity>()
            .init_resource::<SelectedEntityDetails>()
            .init_resource::<GameSpeed>()
            .init_resource::<ActiveSubTool>()
            .init_resource::<LeftClusterTab>()
            // Holocron motion state is intentionally NOT registered here yet —
            // `step_flyout_motion` exists as `#[allow(dead_code)]` for the
            // deferred WGSL/3D anim timeline (see the doc comment on
            // `step_flyout_motion`). Registering `FlyoutMotion` as a resource
            // now would be a one-line cost, but until a system reads it the
            // `#[allow(dead_code)]` would also have to be removed from the
            // resource itself, so we skip both for this PR.
            // .init_resource::<FlyoutMotion>()
            // Info Views tab reads this; init defensively (idempotent) so the
            // HUD never panics if GameUiPlugin runs without InfoViewsPlugin.
            .init_resource::<crate::info_views::InfoViewRegistry>()
            .init_resource::<ToolIcons>()
            .add_systems(Startup, queue_tool_icon_handles)
            .add_systems(Update, (handle_speed_shortcuts, handle_category_hotkeys))
            // EguiPrimaryContextPass is REQUIRED: moving draw to Update panics.
            // `load_tool_icons` registers the PNGs as egui textures and must run
            // before `draw_game_ui` consumes them. Bevy 0.18 dropped the
            // `IntoSystemConfigs::chain()` method on 2-element system tuples
            // (the call resolves to `Curve::chain` instead), so we declare two
            // named sets and order them via `.before()`. `step_flyout_motion`
            // lives on Update so the anim ticks every frame the world is
            // stepping (the HUD itself is gated on `GameUiMode`, but the
            // motion resource is always present so a player who opens a menu
            // while a drawer is mid-animation finds it settled next time the
            // HUD comes back).
            .add_systems(
                EguiPrimaryContextPass,
                // apply_keycap_theme MUST run first: it sets the global egui
                // Style/Visuals (Keycap Palette + holocron chrome) before any
                // draw call can consume it. load_tool_icons and draw_game_ui
                // follow in order.
                (apply_keycap_theme, load_tool_icons, draw_game_ui).chain(),
            );
    }
}

/// Global egui theme system — runs first in every [`EguiPrimaryContextPass`] frame.
///
/// Applies the Phenotype Keycap Palette + holocron command-deck chrome:
/// - Background: midnight `#090a0c` / `#1a1e24` (GRAPHITE_900) surfaces
/// - Primary accent: teal `#7ebab5` on edges, selection, and active strokes only
///   (never as a large fill — "neon-as-signal" rule)
/// - Holographic glass panels: frosted DECK_GLASS fill + DECK_BORDER rim
/// - Colored teal rim-glow on focus (not white)
/// - Rounded corners (8 px buttons, 12 px panels)
/// - Drop shadows for depth hierarchy
/// - Montserrat (body), JetBrains Mono (numeric), Bricolage Grotesque (display)
///
/// Delegates to [`crate::ui_theme::apply_theme`] which is the canonical
/// implementation; this system exists purely to give it an explicit, named place
/// in the Bevy schedule and to separate theming from HUD draw logic.
fn apply_keycap_theme(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        apply_theme(ctx);
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

#[cfg(feature = "egui")]
fn sync_initial_game_speed_from_settings(
    settings: Option<Res<GameSettings>>,
    mut speed: ResMut<GameSpeed>,
) {
    let Some(settings) = settings else {
        return;
    };
    speed.multiplier = settings.gameplay.default_sim_speed.max(0.0);
}

fn handle_speed_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings: Option<Res<GameSettings>>,
    mut speed: ResMut<GameSpeed>,
) {
    let binding_just_pressed =
        |action: &str, fallback: KeyCode, settings: &Option<Res<GameSettings>>| -> bool {
            settings
                .as_ref()
                .and_then(|s| s.key_for(action))
                .unwrap_or(KeyBinding::Key(fallback))
                .is_just_pressed(&keys, &mouse_buttons)
        };

    if binding_just_pressed(ACTION_PAUSE_SIM, KeyCode::Space, &settings) {
        speed.multiplier = if speed_value_matches(speed.multiplier, 0.0) {
            1.0
        } else {
            0.0
        };
    }
    if binding_just_pressed(ACTION_SPEED_1X, KeyCode::Digit1, &settings) {
        speed.multiplier = 1.0;
    }
    if binding_just_pressed(ACTION_SPEED_2X, KeyCode::Digit2, &settings) {
        speed.multiplier = 2.0;
    }
    if binding_just_pressed(ACTION_SPEED_5X, KeyCode::Digit3, &settings) {
        speed.multiplier = 5.0;
    }
    if binding_just_pressed(ACTION_SPEED_10X, KeyCode::Digit4, &settings) {
        speed.multiplier = 10.0;
    }
    if binding_just_pressed(ACTION_CYCLE_SIM_SPEED, KeyCode::Equal, &settings) {
        speed.multiplier = match speed.multiplier {
            value if speed_value_matches(value, 0.0) => 1.0,
            value if speed_value_matches(value, 1.0) => 2.0,
            value if speed_value_matches(value, 2.0) => 5.0,
            value if speed_value_matches(value, 5.0) => 10.0,
            _ => 1.0,
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_game_ui(
    mut contexts: EguiContexts,
    snapshot: Res<GameUiSnapshot>,
    selected: Res<SelectedEntity>,
    live_selection: Res<LiveSelection>,
    details: Res<SelectedEntityDetails>,
    attach_mode: Res<AttachMode>,
    live_attach: Option<Res<crate::live_attach::LiveAttachState>>,
    mut laws_open: ResMut<GameLawsOpen>,
    mut speed: ResMut<GameSpeed>,
    mut active_tool: ResMut<ActiveTool>,
    mut building_kind: ResMut<BuildingSpawnKind>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    apply_theme(ctx);

    egui::TopBottomPanel::top("civis_game_top_bar")
        .frame(panel_frame(egui::Margin::symmetric(12, 8)))
        .show(ctx, |ui| {
            top_bar_ui(
                ui,
                &snapshot,
                &attach_mode,
                live_attach.as_deref(),
                &mut laws_open,
            );
        });

    egui::TopBottomPanel::bottom("civis_game_bottom_bar")
        .frame(panel_frame(egui::Margin::symmetric(12, 8)))
        .show(ctx, |ui| {
            tool_palette_ui(ui, &mut active_tool, &mut building_kind, &mut speed);
        });

    let show_live_inspector = *attach_mode == AttachMode::Server && live_selection.0.is_some();
    if selected.entity.is_some() || show_live_inspector {
        egui::SidePanel::right("civis_game_selected_panel")
            .resizable(true)
            .default_width(268.0)
            .frame(panel_frame(egui::Margin::same(14)))
            .show(ctx, |ui| {
                inspector_ui(ui, &details);
            });
    }
}

/// Compact needs line for the selection inspector (`F 82% · S 70% · …`).
#[must_use]
pub fn format_civilian_needs_summary(needs: &CivilianNeeds3d) -> String {
    format!(
        "F {:.0}% · Sh {:.0}% · Sa {:.0}% · So {:.0}% · R {:.0}%",
        needs.food * 100.0,
        needs.shelter * 100.0,
        needs.safety * 100.0,
        needs.social * 100.0,
        needs.rest * 100.0,
    )
}

/// Health label for the inspector progress bar (`87%`).
#[must_use]
pub fn format_civilian_health_display(health: f32) -> String {
    format!("{:.0}%", health.clamp(0.0, 1.0) * 100.0)
}

/// Display name for a civilian wire entry (genome summary or stable id).
#[must_use]
pub fn civilian_display_name(entry: &CivilianStateEntry) -> String {
    if entry.genome_summary.summary.is_empty() {
        format!("Civilian #{}", entry.id)
    } else {
        entry.genome_summary.summary.clone()
    }
}

/// Faction label from genome lineage (`faction-3` → `Faction 3`).
#[must_use]
pub fn civilian_faction_label(entry: &CivilianStateEntry) -> String {
    let lineage = entry.genome_summary.lineage.trim();
    if lineage.is_empty() {
        return "—".to_string();
    }
    if let Some(id) = lineage.strip_prefix("faction-") {
        return format!("Faction {id}");
    }
    lineage.to_string()
}

/// Build inspector rows from a `CivilianState` wire entry.
#[must_use]
pub fn inspector_details_from_civilian(entry: &CivilianStateEntry) -> SelectedEntityDetails {
    let profession = if entry.profession.is_empty() {
        "—".to_string()
    } else {
        entry.profession.clone()
    };
    let species = if entry.species.is_empty() {
        "—".to_string()
    } else {
        entry.species.clone()
    };
    SelectedEntityDetails {
        name: civilian_display_name(entry),
        faction: civilian_faction_label(entry),
        health: format_civilian_health_display(entry.health),
        profession,
        species,
        needs: format_civilian_needs_summary(&entry.needs),
        position: "—".to_string(),
    }
}

/// Format a world-space point for the inspector position row.
#[must_use]
pub fn format_world_position(position: Vec3) -> String {
    format!("{:.1}, {:.1}, {:.1}", position.x, position.y, position.z)
}

/// Inspector rows for a live-streamed entity without civilian wire data.
#[must_use]
pub fn inspector_details_for_live_entity(
    entity: SelectedLiveEntity,
    position: Option<Vec3>,
) -> SelectedEntityDetails {
    let position = position
        .map(format_world_position)
        .unwrap_or_else(|| "—".to_string());
    let (name, profession) = match entity.kind {
        LiveEntityKind::Agent => (format!("Agent #{}", entity.id), "—".to_string()),
        LiveEntityKind::Building => (format!("Building #{}", entity.id), "—".to_string()),
        LiveEntityKind::GraphParcel => (format!("Parcel #{}", entity.id), "—".to_string()),
    };
    SelectedEntityDetails {
        name,
        faction: "—".to_string(),
        health: "—".to_string(),
        profession,
        species: "—".to_string(),
        needs: "—".to_string(),
        position,
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
        (
            TextStyle::Monospace,
            FontId::new(13.0, egui::FontFamily::Monospace),
        ),
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
    laws_open: &mut GameLawsOpen,
) {
    let gold = egui::Color32::from_rgb(240, 200, 90);
    let green = egui::Color32::from_rgb(120, 220, 130);
    let speed_label = GameSpeed {
        multiplier: snapshot.speed_multiplier,
    }
    .display_label();

    ui.horizontal(|ui| {
        chip(ui, "\u{23f1}", &format!("{}", snapshot.tick), ACCENT);
        chip(ui, "\u{1f465}", &format!("{}", snapshot.population), green);
        chip(ui, "\u{1f6a9}", &format!("{}", snapshot.factions), gold);
        chip(
            ui,
            "\u{1f30d}",
            &format!("Era {}", snapshot.era),
            egui::Color32::WHITE,
        );
        chip(ui, "\u{25b6}", &speed_label, ACCENT);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("⚖ Laws").clicked() {
                laws_open.0 = !laws_open.0;
            }
            if *attach_mode == crate::AttachMode::Server {
                let connected = live_attach.map(|s| s.connected).unwrap_or(false);
                let (dot, text, color) = if connected {
                    ("\u{1f7e2}", "WS Live", green)
                } else {
                    ("\u{1f7e1}", "WS \u{2026}", gold)
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

/// Bottom bar: tool palette (left) + segmented speed control (right).
fn tool_palette_ui(
    ui: &mut egui::Ui,
    active: &mut ActiveTool,
    building_kind: &mut BuildingSpawnKind,
    speed: &mut GameSpeed,
) {
    let tools = [
        ToolDef {
            icon: "\u{1f446}",
            label: "Select",
            hotkey: "Q",
            tool: Some(SpawnTool::Select),
        },
        ToolDef {
            icon: "\u{1f9cd}",
            label: "Spawn Civ",
            hotkey: "W",
            tool: Some(SpawnTool::SpawnCivilian),
        },
        ToolDef {
            icon: "\u{1f3e0}",
            label: "Building",
            hotkey: "E",
            tool: Some(SpawnTool::SpawnBuilding),
        },
        ToolDef {
            icon: "\u{26f0}",
            label: "Terraform",
            hotkey: "R",
            tool: Some(SpawnTool::Terraform),
        },
        ToolDef {
            icon: "\u{1f4a5}",
            label: "Destroy",
            hotkey: "T",
            tool: Some(SpawnTool::Destroy),
        },
        // Weather has no SpawnTool variant yet: present but inert.
        ToolDef {
            icon: "\u{1f327}",
            label: "Weather",
            hotkey: "Y",
            tool: None,
        },
    ];

    ui.horizontal(|ui| {
        for def in &tools {
            let is_active = def.tool == Some(active.tool);
            let is_building = def.tool == Some(SpawnTool::SpawnBuilding);
            if tool_button(ui, def, is_active).clicked() {
                if let Some(tool) = def.tool {
                    active.tool = tool;
                }
                // Weather (tool == None) is intentionally a no-op for now.
            }
            if is_building && is_active {
                ui.label(
                    egui::RichText::new(building_kind.label())
                        .color(DIM)
                        .small(),
                );
            }
        }

        if active.tool == SpawnTool::SpawnBuilding {
            ui.separator();
            ui.label(egui::RichText::new("Building").color(DIM).small());
            if ui
                .button(egui::RichText::new(building_kind.label()).color(ACCENT).strong())
                .on_hover_text("Right-click or scroll while the build tool is active to cycle building type.")
                .clicked()
            {
                *building_kind = building_kind.next();
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            speed_control_ui(ui, speed);
        });
    });
}

/// Render one 56x56 tool button with emoji + label, accent-highlighted if active.
fn tool_button(ui: &mut egui::Ui, def: &ToolDef, active: bool) -> egui::Response {
    let fill = if active {
        ACCENT.gamma_multiply(0.30)
    } else {
        CHIP_FILL
    };
    let stroke = if active {
        egui::Stroke::new(1.5, ACCENT)
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 68, 88))
    };

    let resp = egui::Frame::NONE
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(4))
        .show(ui, |ui| {
            ui.set_width(48.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(def.icon).size(22.0));
                let lbl = egui::RichText::new(def.label).small();
                ui.label(if active {
                    lbl.color(ACCENT).strong()
                } else {
                    lbl.color(DIM)
                });
            });
        })
        .response;

    resp.interact(egui::Sense::click())
        .on_hover_text(format!("{} ({})", def.label, def.hotkey))
}

/// Segmented speed control: pause / 1x / 2x / 5x / 10x wired to GameSpeed.
fn speed_control_ui(ui: &mut egui::Ui, speed: &mut GameSpeed) {
    // Reversed because parent layout is right_to_left.
    let steps = [
        (10.0f32, "10x"),
        (5.0, "5x"),
        (2.0, "2x"),
        (1.0, "1x"),
        (0.0, "\u{23f8}"),
    ];
    for (mult, label) in steps {
        let active = speed_value_matches(speed.multiplier, mult);
        let mut text = egui::RichText::new(label);
        if active {
            text = text.color(ACCENT).strong();
        } else {
            text = text.color(DIM);
        }
        let btn = egui::Button::new(text)
            .fill(if active {
                ACCENT.gamma_multiply(0.30)
            } else {
                CHIP_FILL
            })
            .min_size(egui::vec2(38.0, 30.0));
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
    inspector_row(ui, "Species", &details.species);
    inspector_row(ui, "Needs", &details.needs);
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
        if !v.is_finite() {
            return None;
        }
        return Some((v / 100.0).clamp(0.0, 1.0));
    }
    let v: f32 = s.parse().ok()?;
    if !v.is_finite() {
        return None;
    }
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
        assert_eq!(GameSpeed { multiplier: 0.0 }.display_label(), "Paused");
        assert_eq!(GameSpeed { multiplier: 1.0 }.display_label(), "1x");
        assert_eq!(GameSpeed { multiplier: 2.0 }.display_label(), "2x");
        assert_eq!(GameSpeed { multiplier: 5.0 }.display_label(), "5x");
        assert_eq!(GameSpeed { multiplier: 10.0 }.display_label(), "10x");
        assert_eq!(GameSpeed { multiplier: 7.0 }.display_label(), "7x");
        assert_eq!(GameSpeed { multiplier: 0.25 }.display_label(), "0.25x");
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
    fn civilian_needs_summary_formats_percentages() {
        let needs = CivilianNeeds3d {
            food: 0.82,
            shelter: 0.7,
            safety: 0.55,
            social: 0.4,
            rest: 0.9,
        };
        let summary = format_civilian_needs_summary(&needs);
        assert!(summary.contains("F 82%"));
        assert!(summary.contains("Sh 70%"));
        assert!(summary.contains("R 90%"));
    }

    #[test]
    fn inspector_details_from_civilian_entry() {
        use civ_protocol_3d::GenomeSummary3d;

        let entry = CivilianStateEntry {
            id: 42,
            needs: CivilianNeeds3d {
                food: 1.0,
                shelter: 0.5,
                safety: 0.5,
                social: 0.5,
                rest: 0.5,
            },
            profession: "farmer".to_string(),
            genome_summary: GenomeSummary3d {
                summary: "era-2".to_string(),
                lineage: "faction-3".to_string(),
                traits: Vec::new(),
            },
            species: "human".to_string(),
            health: 0.87,
        };
        let details = inspector_details_from_civilian(&entry);
        assert_eq!(details.name, "era-2");
        assert_eq!(details.faction, "Faction 3");
        assert_eq!(details.health, "87%");
        assert_eq!(details.profession, "farmer");
        assert_eq!(details.species, "human");
        assert!(details.needs.contains("F 100%"));
    }

    #[test]
    fn civilian_display_name_falls_back_to_id() {
        use civ_protocol_3d::GenomeSummary3d;

        let entry = CivilianStateEntry {
            id: 7,
            needs: CivilianNeeds3d::default(),
            profession: String::new(),
            genome_summary: GenomeSummary3d::default(),
            species: String::new(),
            health: 1.0,
        };
        assert_eq!(civilian_display_name(&entry), "Civilian #7");
    }

    #[test]
    fn snapshot_set_sim_state_clamps_speed() {
        let mut snap = GameUiSnapshot::default();
        snap.set_sim_state(10, 20, 3, "Bronze", 0);
        assert_eq!(snap.tick, 10);
        assert_eq!(snap.population, 20);
        assert_eq!(snap.factions, 3);
        assert_eq!(snap.era, "Bronze");
        assert_eq!(snap.speed_multiplier, 0.0);
    }
}
