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

use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use crate::game_laws::GameLawsOpen;
use crate::holo_minimap::HoloMinimapPlugin;
use crate::menus::GameUiMode;
use crate::spawn_tools::{ActiveTool, SelectedEntity};
use crate::tool_categories::{ActiveSubTool, SubTool, CATEGORIES};
use crate::ui_theme::{
    apply_theme, compact, deck_chip, hairline, liquid_glass_finish, liquid_glass_frame,
    panel_finish, BORDER, DECK_ACCENT, DECK_BORDER, DECK_GLASS, DECK_SUCCESS, DECK_TEXT,
    DECK_TEXT_MID, DIM, GOLD, GREEN, INSET_FILL, RADIUS_BTN, RADIUS_PANEL, RED, SPACE_LG, SPACE_MD,
    SPACE_SM, SPACE_XS, TEXT,
};
use crate::ui_theme::{ease_towards, panel_edge_stroke, panel_glass_fill, UiAnimState};
// `FlyoutMotion` and `step_flyout_motion` are intentionally NOT added as
// parameters to `draw_game_ui` — that function already has 16 system params,
// the Bevy 0.18 `SystemParamFunction` macro limit. Per-frame easing for
// rim-glow and scale-on-select is applied per-widget via the existing
// `motion_rect` / `selection_scale` helpers in `ui_theme.rs`, which
// already interpolate with `ease_out_cubic`. See `docs/decisions/holocron-3d.md`
// for the deferred WGSL + anim timeline follow-up.

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

/// Which tab of the unified left HUD cluster is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftTab {
    /// Selection inspector card.
    Inspector,
    /// Faction / group roster.
    Factions,
    /// Info Views overlay picker + legend.
    InfoViews,
}

/// Active tab of the left HUD cluster (persisted across frames).
#[derive(Resource, Debug, Clone, Copy)]
pub struct LeftClusterTab(pub LeftTab);

impl Default for LeftClusterTab {
    fn default() -> Self {
        Self(LeftTab::Inspector)
    }
}

/// Holocron flyout motion state. `ease_towards` steps this every frame; the
/// bottom-cluster drawer reads `progress` to slide + fade in/out. A separate
/// anim drives the left tab strip + speed-control blades (scale-on-select).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct FlyoutMotion {
    /// Bottom-cluster category flyout open/close progress (0..=1).
    pub bottom: UiAnimState,
    /// Left tab strip swap progress (0..=1; brief non-linear bump on switch).
    pub left_tab: UiAnimState,
    /// Speed-control segment scale-on-select progress (0..=1; never settles
    /// to 0 since we only re-arm it on click).
    pub speed_blade: UiAnimState,
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
    ("infra", "ui/tool-icons/infra.png"),
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
                sub.open_category = if sub.open_category == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
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
    laws_open: Option<&'a mut GameLawsOpen>,
    /// Eased open/close progress for the category flyout (0..=1). Read-only
    /// here; the plugin's `step_flyout_motion` system drives it each frame.
    flyout_progress: f32,
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
    mut left_tab: ResMut<LeftClusterTab>,
    mut info_views: ResMut<crate::info_views::InfoViewRegistry>,
    ui_mode: Res<GameUiMode>,
    tool_icons: Res<ToolIcons>,
    mut laws_open: Option<ResMut<GameLawsOpen>>,
) {
    // Show the HUD while Playing OR Paused (frozen-but-visible) — matches the
    // `menus::in_game` gate the brush/map panels use, so the left cluster, top
    // and bottom clusters don't vanish when the others stay up (e.g. Paused, or
    // the autoshot warm-up frame). Only menus/loading hide it entirely.
    if !matches!(*ui_mode, GameUiMode::Playing | GameUiMode::Paused) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // The flyout anim target is synced to `sub_tool.open_category` via
    // `sync_flyout_motion_target` (registered before `draw_game_ui`) so this
    // function stays under Bevy 0.18's 16-system-param cap.

    // ---- Top cluster: CENTERED readout (floating, not a full-width bar) ----
    top_center_cluster(
        ctx,
        &snapshot,
        &resources,
        &attach_mode,
        live_attach.as_deref(),
    );

    // ---- Left cluster: ONE tabbed column (Inspector / Factions / Info Views) ----
    left_sidebar_cluster(
        ctx,
        &mut left_tab.0,
        &roster,
        selected.0.is_some(),
        &details,
        &mut info_views,
    );

    // ---- Bottom: narrow, short, floating cluster of expanding block-pills ----
    let mut bottom = BottomBarCtx {
        active: &mut active_tool,
        sub: &mut sub_tool,
        speed: &mut speed,
        icons: &tool_icons.ids,
        laws_open: laws_open.as_mut().map(|open| &mut **open),
        // flyout_progress is read from the per-system `Local` written by
        // `sample_flyout_motion` (registered in the chain so it ticks every
        // frame, even on menus — but only the HUD consumes it).
        flyout_progress: 0.0,
    };
    bottom_cluster(ctx, &mut bottom);
}

/// Top-center HUD readout: a single centered floating glass cluster of stat
/// chips + the resource strip (no longer a flush full-width top bar).
fn top_center_cluster(
    ctx: &egui::Context,
    snapshot: &GameUiSnapshot,
    resources: &WorldResources,
    attach_mode: &crate::AttachMode,
    live_attach: Option<&crate::live_attach::LiveAttachState>,
) {
    egui::Area::new(egui::Id::new("civis_top_center"))
        .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
        .show(ctx, |ui| {
            liquid_glass_frame(
                egui::Margin::symmetric(SPACE_LG as i8, SPACE_SM as i8),
                RADIUS_PANEL,
            )
            .show(ui, |ui| {
                top_bar_ui(ui, snapshot, resources, attach_mode, live_attach);
                liquid_glass_finish(ui.painter(), ui.min_rect(), RADIUS_PANEL);
            });
        });
}

/// Left sidebar: a single left-edge vertical column that merges the faction
/// roster and the selection inspector into one frosted cluster (the minimap
/// anchor stays reserved at the bottom for `live_minimap.rs`).
fn left_sidebar_cluster(
    ctx: &egui::Context,
    tab: &mut LeftTab,
    roster: &FactionRoster,
    has_selection: bool,
    details: &SelectedEntityDetails,
    info_views: &mut crate::info_views::InfoViewRegistry,
) {
    egui::SidePanel::left("civis_game_left_sidebar")
        .resizable(false)
        .exact_width(252.0)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let full_panel = ui.max_rect();
            ui.painter()
                .rect_filled(full_panel, RADIUS_PANEL as f32, crate::ui_theme::GLASS_FILL);
            let glass = liquid_glass_frame(egui::Margin::same(SPACE_MD as i8), RADIUS_PANEL);
            glass.show(ui, |ui| {
                // Teal rim + lifted inner highlight on the cluster's own rect BEFORE
                // content, so the glass edge reads without glossing over text.
                let full = ui.max_rect();
                sidebar_glass_edge(ui.painter(), full);
                left_tab_strip(ui, tab);
                ui.add_space(SPACE_SM);
                hairline(ui);
                match tab {
                    LeftTab::Inspector => inspector_ui(ui, has_selection, details),
                    LeftTab::Factions => faction_panel_ui(ui, roster),
                    LeftTab::InfoViews => crate::info_views::info_view_tab_body(ui, info_views),
                }
            });
        });
}

/// The Inspector / Factions / Info Views tab strip atop the left cluster.
fn left_tab_strip(ui: &mut egui::Ui, tab: &mut LeftTab) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
        for (variant, label) in [
            (LeftTab::Inspector, "\u{25a4} Inspect"),
            (LeftTab::Factions, "\u{1f6a9} Factions"),
            (LeftTab::InfoViews, "\u{1f5fa} Views"),
        ] {
            let selected = *tab == variant;
            let is_hovering = false;
            let text = egui::RichText::new(label).color(if selected {
                DECK_ACCENT
            } else {
                DECK_TEXT_MID
            });
            let response = ui.add(
                egui::Button::new(text)
                    .fill(panel_glass_fill(is_hovering, false))
                    .stroke(panel_edge_stroke(false, selected))
                    .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
                    .min_size(egui::vec2(102.0, 30.0)),
            );
            if response.hovered() {
                ui.painter().rect_stroke(
                    response.rect.shrink(0.8),
                    RADIUS_BTN as f32,
                    egui::Stroke::new(1.0, DECK_ACCENT.gamma_multiply(0.45)),
                    egui::StrokeKind::Inside,
                );
                ui.painter().rect_filled(
                    response.rect,
                    RADIUS_BTN as f32,
                    panel_glass_fill(true, false).gamma_multiply(0.15),
                );
            }
            if response.is_pointer_button_down_on() {
                ui.painter().rect_filled(
                    response.rect.shrink(1.0),
                    RADIUS_BTN as f32,
                    panel_glass_fill(false, true).gamma_multiply(0.12),
                );
            }
            if response.clicked() {
                *tab = variant;
            }
        }
    });
}

/// Lifted glass edge for a text-dense panel: a thin light inner highlight + a
/// soft teal rim, without the gloss sheen (which would dim text drawn on top).
fn sidebar_glass_edge(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_stroke(
        rect.shrink(1.0),
        RADIUS_PANEL as f32,
        egui::Stroke::new(1.0, egui::Color32::from_white_alpha(26)),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect,
        RADIUS_PANEL as f32,
        egui::Stroke::new(1.0, DECK_ACCENT.gamma_multiply(0.30)),
        egui::StrokeKind::Outside,
    );
}

/// Bottom cluster: a narrow (< full-width), short floating row of expanding
/// category block-pills + the speed control, wrapped in a frosted glass shell
/// with padding + margin (not a flush full-width bar).
///
/// Holocron flyout: when a category is open, the items rect slides down 8px +
/// fades from the top with `ease_out_cubic` (`bottom.flyout_progress` 0..=1).
/// When closed, the drawer eases back to 0 alpha. The progress is driven by
/// `step_flyout_motion` (Update) so the curve is frame-rate independent.
fn bottom_cluster(ctx: &egui::Context, bottom: &mut BottomBarCtx) {
    egui::Area::new(egui::Id::new("civis_bottom_cluster"))
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -12.0])
        .show(ctx, |ui| {
            // Expanded items rect (the larger rectangle) stacks ABOVE the pills.
            // The holocron pass slides it down 8px and fades it in with
            // `flyout_progress` so the drawer eases in/out instead of snapping.
            if let Some(idx) = bottom.sub.open_category {
                if let Some(cat) = CATEGORIES.get(idx) {
                    let progress = bottom.flyout_progress.clamp(0.0, 1.0);
                    // Slide 8px in from the top per `ui-design-language.md` §8.2.
                    let slide_px = (1.0 - progress) * 8.0;
                    // When the drawer is essentially fully closed we still need
                    // to lay it out once (egui requires it for hit-testing) but
                    // we skip the work below a 1% progress threshold so an
                    // empty-looking drawer never lingers.
                    if progress > 0.01 {
                        ui.vertical_centered(|ui| {
                            ui.add_space(slide_px);
                            let mut group =
                                egui::Frame::NONE.fill(DECK_GLASS.gamma_multiply(progress));
                            group = group
                                .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    DECK_BORDER.gamma_multiply(progress),
                                ));
                            group.show(ui, |ui| {
                                if let Some(picked) =
                                    crate::ui_cluster::items_rect(ui, cat, bottom.sub.current)
                                {
                                    select_subtool(bottom, picked);
                                }
                            });
                        });
                        ui.add_space(6.0);
                    }
                }
            }
            liquid_glass_frame(
                egui::Margin::symmetric(SPACE_MD as i8, SPACE_SM as i8),
                RADIUS_PANEL,
            )
            .show(ui, |ui| {
                category_pill_row(ui, bottom);
                liquid_glass_finish(ui.painter(), ui.min_rect(), RADIUS_PANEL);
            });
        });
}

/// One horizontal row of small category block-pills + the speed control. Each
/// pill is the always-visible small rect; clicking toggles its items rect.
fn category_pill_row(ui: &mut egui::Ui, ctx: &mut BottomBarCtx) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
        let active_cat = ctx.sub.active_category();
        for (idx, cat) in CATEGORIES.iter().enumerate() {
            let is_open = ctx.sub.open_category == Some(idx);
            let is_active = active_cat == Some(idx);
            let icon_tex = cat.icon_key().and_then(|k| ctx.icons.get(k).copied());
            let resp = crate::ui_cluster::category_pill(ui, cat, is_open, is_active, icon_tex);
            if resp.hovered() {
                ui.painter().rect_stroke(
                    resp.rect.shrink(0.7),
                    RADIUS_BTN as f32,
                    egui::Stroke::new(1.0, DECK_ACCENT.gamma_multiply(0.65)),
                    egui::StrokeKind::Inside,
                );
                ui.painter().rect_stroke(
                    resp.rect,
                    RADIUS_BTN as f32,
                    egui::Stroke::new(1.1, DECK_ACCENT.gamma_multiply(0.24)),
                    egui::StrokeKind::Outside,
                );
            }
            if resp.is_pointer_button_down_on() {
                ui.painter().rect_filled(
                    resp.rect,
                    RADIUS_BTN as f32,
                    panel_glass_fill(false, true).gamma_multiply(0.12),
                );
            }
            if resp.clicked() {
                ctx.sub.open_category = if is_open { None } else { Some(idx) };
            }
        }
        ui.add_space(SPACE_MD);
        speed_control_ui(ui, ctx.speed);
        ui.add_space(SPACE_SM);
        if ui
            .button(egui::RichText::new("Laws").color(TEXT))
            .on_hover_text("Open game laws viewer")
            .clicked()
        {
            if let Some(laws_open) = ctx.laws_open.as_deref_mut() {
                laws_open.0 = !laws_open.0;
            }
        }
    });
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
    let fill = if value.is_empty() {
        DECK_GLASS
    } else {
        DECK_GLASS
    };
    egui::Frame::NONE
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .inner_margin(egui::Margin::symmetric(SPACE_MD as i8, SPACE_XS as i8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(
                egui::RichText::new(value)
                    .monospace()
                    .color(DECK_TEXT)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(format!("{arrow}{:+.0}", delta))
                    .color(dcol)
                    .small()
                    .monospace(),
            );
            panel_finish(ui.painter(), ui.min_rect(), RADIUS_BTN, false, false);
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
                .font(crate::ui_theme::display_font(ui.ctx(), 17.0))
                .color(DECK_ACCENT)
                .strong(),
        );
        ui.add_space(SPACE_XS);
        deck_chip(ui, "tick", &snapshot.tick.to_string(), DECK_ACCENT);
        deck_chip(ui, "era", &snapshot.era, DECK_ACCENT);
        deck_chip(ui, "pop", &snapshot.population.to_string(), DECK_SUCCESS);
        deck_chip(ui, "factions", &snapshot.factions.to_string(), DECK_ACCENT);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ws_status_ui(ui, snapshot, attach_mode, live_attach);
        });
    });
    ui.add_space(SPACE_SM);
    ui.horizontal(|ui| {
        resource_chip(
            ui,
            "\u{1f33e}",
            &compact(resources.food),
            resources.food_delta,
            DECK_ACCENT,
        );
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
            DECK_ACCENT,
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
        ("\u{1f7e1}", "WS \u{2026}", DECK_ACCENT)
    };
    deck_chip(ui, dot, text, color);
    if let Some(overlay) = &snapshot.live_hud_overlay {
        ui.label(
            egui::RichText::new(overlay)
                .color(DECK_TEXT_MID)
                .small()
                .monospace(),
        );
    }
}

// ---------------------------------------------------------------------------
// Bottom cluster helpers (sub-tool selection + speed control)
// ---------------------------------------------------------------------------

/// Pick a sub-tool: set the UI-side current tool + sync the backing SpawnTool.
fn select_subtool(ctx: &mut BottomBarCtx, st: SubTool) {
    ctx.sub.current = st;
    if let Some(tool) = st.spawn_tool() {
        ctx.active.tool = tool; // keep spawn_tools::ActiveTool in lockstep
    }
    // Sub-tools without a backing variant are intentional no-ops until the
    // Infra Lead grows SpawnTool; the UI still lights them as the picked tool.
}

/// Segmented speed control: pause / 1x / 2x / 5x / 10x wired to GameSpeed.
fn speed_control_ui(ui: &mut egui::Ui, speed: &mut GameSpeed) {
    ui.label(
        egui::RichText::new("\u{23f5} Speed")
            .color(DECK_TEXT_MID)
            .small(),
    );
    // Left-to-right order inside the bottom cluster row.
    let steps = [
        (0u32, "\u{23f8}"),
        (1, "1x"),
        (2, "2x"),
        (3, "5x"),
        (4, "10x"),
    ];
    for (mult, label) in steps {
        let active = speed.multiplier == mult;
        let mut text = egui::RichText::new(label).size(13.0).monospace();
        text = if active {
            text.color(DECK_ACCENT).strong()
        } else {
            text.color(DECK_TEXT_MID)
        };
        let btn = egui::Button::new(text)
            .fill(if active {
                panel_glass_fill(false, false).gamma_multiply(1.12)
            } else {
                panel_glass_fill(false, false)
            })
            .stroke(if active {
                egui::Stroke::new(1.5, DECK_ACCENT)
            } else {
                panel_edge_stroke(false, false)
            })
            .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
            .min_size(egui::vec2(40.0, 34.0));
        let resp = ui.add(btn);
        if resp.hovered() {
            ui.painter().rect_stroke(
                resp.rect.shrink(0.9),
                RADIUS_BTN as f32,
                egui::Stroke::new(1.0, DECK_ACCENT.gamma_multiply(0.4)),
                egui::StrokeKind::Inside,
            );
        }
        if resp.is_pointer_button_down_on() {
            ui.painter().rect_filled(
                resp.rect,
                RADIUS_BTN as f32,
                panel_glass_fill(false, true).gamma_multiply(0.15),
            );
        }
        if resp.clicked() {
            speed.multiplier = mult;
        }
    }
}

// ---------------------------------------------------------------------------
// Left faction / group panel
// ---------------------------------------------------------------------------

/// Factions tab body: faction/group roster with colour swatches + counts.
///
/// The bottom-left minimap reservation was removed — the holocron map on the
/// right is the single map, so the roster uses the full column height. No
/// heading: the left-cluster tab strip already labels it.
fn faction_panel_ui(ui: &mut egui::Ui, roster: &FactionRoster) {
    if roster.factions.is_empty() {
        ui.label(
            egui::RichText::new("No factions yet.")
                .color(DECK_TEXT_MID)
                .small(),
        );
        ui.label(
            egui::RichText::new("Spawn life to seed groups.")
                .color(DECK_TEXT_MID.gamma_multiply(0.8))
                .small(),
        );
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for faction in &roster.factions {
                faction_row(ui, faction);
            }
        });
    }
}

/// One faction row: colour swatch, name, and right-aligned member count.
fn faction_row(ui: &mut egui::Ui, faction: &FactionInfo) {
    let swatch = egui::Color32::from_rgb(faction.color[0], faction.color[1], faction.color[2]);
    egui::Frame::NONE
        .fill(INSET_FILL)
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, swatch);
                ui.painter().rect_stroke(
                    r,
                    3.0,
                    egui::Stroke::new(1.0, BORDER),
                    egui::StrokeKind::Inside,
                );
                ui.label(egui::RichText::new(&faction.name).strong().color(TEXT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(compact(faction.count as f64))
                            .monospace()
                            .color(DIM),
                    );
                });
            });
            panel_finish(ui.painter(), ui.min_rect(), 7, false, false);
        });
    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// Right inspector card
// ---------------------------------------------------------------------------

/// Inspector tab body: selection card with empty-state fallback. (No heading —
/// the left-cluster tab strip already labels it.)
fn inspector_ui(ui: &mut egui::Ui, has_selection: bool, details: &SelectedEntityDetails) {
    if !has_selection {
        inspector_empty_state(ui);
        return;
    }

    let kind = if details.kind.is_empty() {
        "Entity"
    } else {
        &details.kind
    };
    ui.horizontal(|ui| {
        let name = if details.name.is_empty() {
            "Unnamed"
        } else {
            &details.name
        };
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
        ui.label(
            egui::RichText::new("\u{1f9ed}")
                .size(34.0)
                .color(DIM.gamma_multiply(0.8)),
        );
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
            ui.add(
                egui::ProgressBar::new(frac)
                    .fill(color)
                    .text(health.to_string()),
            );
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
            assert!(
                hotkey_to_code(cat.hotkey).is_some(),
                "{} hotkey unmapped",
                cat.label
            );
        }
    }

    #[test]
    fn flyout_motion_defaults_are_zeroed() {
        // Freshly initialized motion state should be fully closed + settled.
        let m = FlyoutMotion::default();
        assert_eq!(m.bottom.current, 0.0);
        assert_eq!(m.bottom.target, 0.0);
        assert_eq!(m.left_tab.current, 0.0);
        assert_eq!(m.speed_blade.current, 0.0);
        assert!(m.bottom.is_settled());
    }

    #[test]
    fn flyout_motion_open_then_close_round_trip() {
        // The HUD opens a category, then closes it: target should toggle 1→0
        // and the anim should be left in a settled state either way.
        let mut m = FlyoutMotion::default();
        m.bottom.set_open(true);
        assert_eq!(m.bottom.target, 1.0);
        m.bottom.set_open(false);
        assert_eq!(m.bottom.target, 0.0);
        assert!(m.bottom.is_settled());
    }
}
