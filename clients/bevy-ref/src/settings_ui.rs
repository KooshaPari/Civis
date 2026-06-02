#![cfg(all(feature = "bevy", feature = "egui"))]

//! Settings / Options panel for the Civis reference client.
//!
//! A themed (via [`crate::ui_theme`]) egui overlay covering the five standard
//! option groups players expect from a Cities-Skylines / Empire-at-War class
//! game:
//!
//! 1. **Graphics** — resolution and a quality preset plus granular rendering
//!    controls.
//! 2. **Display** — resolution, windowing mode, VSync, and framerate cap.
//! 3. **Audio** — master / music / SFX volumes.
//! 4. **Gameplay** — default sim speed and autosave interval.
//! 5. **Controls** — a read-only reference list of the client's hotkeys.
//!
//! State lives in the [`GameSettings`] resource which is `serde`-serialisable
//! and round-trips to `settings.ron` next to the executable. Open / close the
//! panel with `O` (or Esc to close).
//!
//! # Usage
//! ```no_run
//! # use civ_bevy_ref::settings_ui::SettingsPlugin;
//! # use bevy::prelude::*;
//! # let mut app = App::new();
//! app.add_plugins(SettingsPlugin);
//! ```
//!
//! `SettingsPlugin` does **not** add `EguiPlugin` — that remains the
//! responsibility of `GameUiPlugin`, matching the other HUD modules.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use serde::{Deserialize, Serialize};

use crate::ui_theme;

const SETTINGS_PATH: &str = "settings.ron";

// ---------------------------------------------------------------------------
// Enums for preset-style options
// ---------------------------------------------------------------------------

/// A windowed/fullscreen resolution preset shown in the display group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionPreset {
    /// 1280x720.
    R720p,
    /// 1920x1080.
    R1080p,
    /// 2560x1440.
    R1440p,
    /// 3840x2160.
    R2160p,
}

impl ResolutionPreset {
    /// All presets in menu order.
    pub const ALL: [ResolutionPreset; 4] =
        [Self::R720p, Self::R1080p, Self::R1440p, Self::R2160p];

    /// Pixel dimensions `(width, height)` for the preset.
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::R720p => (1280, 720),
            Self::R1080p => (1920, 1080),
            Self::R1440p => (2560, 1440),
            Self::R2160p => (3840, 2160),
        }
    }

    /// Human-readable label, e.g. `"1920 x 1080"`.
    pub fn label(self) -> &'static str {
        match self {
            Self::R720p => "1280 x 720",
            Self::R1080p => "1920 x 1080",
            Self::R1440p => "2560 x 1440",
            Self::R2160p => "3840 x 2160 (4K)",
        }
    }
}

/// Overall graphics quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    /// Lowest settings — maximum framerate.
    Low,
    /// Balanced default.
    Medium,
    /// High fidelity.
    High,
    /// Everything maxed.
    Ultra,
}

impl QualityPreset {
    /// All presets in menu order.
    pub const ALL: [QualityPreset; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }
}

/// Shadow quality levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    /// No shadows.
    Off,
    /// Lightweight shadows.
    Low,
    /// Balanced shadows.
    Medium,
    /// High-quality shadows.
    High,
    /// Maximum shadow quality.
    Ultra,
}

impl ShadowQuality {
    /// All options in menu order.
    pub const ALL: [ShadowQuality; 5] =
        [Self::Off, Self::Low, Self::Medium, Self::High, Self::Ultra];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }
}

impl Default for ShadowQuality {
    fn default() -> Self {
        Self::Medium
    }
}

/// Anti-aliasing modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiAliasing {
    /// No anti-aliasing.
    Off,
    /// Fast post-process AA.
    FXAA,
    /// Temporal AA.
    TAA,
    /// Multi-sample AA.
    MSAA,
}

impl AntiAliasing {
    /// All options in menu order.
    pub const ALL: [AntiAliasing; 4] = [Self::Off, Self::FXAA, Self::TAA, Self::MSAA];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::FXAA => "FXAA",
            Self::TAA => "TAA",
            Self::MSAA => "MSAA",
        }
    }
}

impl Default for AntiAliasing {
    fn default() -> Self {
        Self::TAA
    }
}

/// Texture quality options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureQuality {
    /// Low-detail textures.
    Low,
    /// Medium-detail textures.
    Medium,
    /// High-detail textures.
    High,
}

impl TextureQuality {
    /// All options in menu order.
    pub const ALL: [TextureQuality; 3] = [Self::Low, Self::Medium, Self::High];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
}

impl Default for TextureQuality {
    fn default() -> Self {
        Self::High
    }
}

/// Window display modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    /// Normal window.
    Windowed,
    /// Borderless window.
    Borderless,
    /// Fullscreen mode.
    Fullscreen,
}

impl WindowMode {
    /// All options in menu order.
    pub const ALL: [WindowMode; 3] = [Self::Windowed, Self::Borderless, Self::Fullscreen];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless",
            Self::Fullscreen => "Fullscreen",
        }
    }
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::Windowed
    }
}

/// Settings page tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SettingsTab {
    /// Quality and render options.
    Graphics,
    /// Windowing and framerate options.
    Display,
    /// Audio mix options.
    Audio,
    /// Simulation-level behaviour.
    Gameplay,
    /// Input reference panel.
    Controls,
}

impl SettingsTab {
    const ALL: [SettingsTab; 5] = [
        Self::Graphics,
        Self::Display,
        Self::Audio,
        Self::Gameplay,
        Self::Controls,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Graphics => "Graphics",
            Self::Display => "Display",
            Self::Audio => "Audio",
            Self::Gameplay => "Gameplay",
            Self::Controls => "Controls",
        }
    }
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Graphics
    }
}

// ---------------------------------------------------------------------------
// Sub-setting groups
// ---------------------------------------------------------------------------

/// Graphics / video options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Selected resolution preset.
    pub resolution: ResolutionPreset,
    /// Vertical sync (cap framerate to refresh rate).
    pub vsync: bool,
    /// Overall quality preset.
    pub quality: QualityPreset,
    /// Render scale multiplier.
    #[serde(default)]
    pub resolution_scale: f32,
    /// Shadow quality.
    #[serde(default)]
    pub shadow_quality: ShadowQuality,
    /// Anti-aliasing mode.
    #[serde(default)]
    pub anti_aliasing: AntiAliasing,
    /// View distance in chunks / units.
    #[serde(default)]
    pub view_distance: u32,
    /// Texture quality.
    #[serde(default)]
    pub texture_quality: TextureQuality,
    /// Ambient occlusion toggle.
    #[serde(default)]
    pub ambient_occlusion: bool,
    /// Bloom toggle.
    #[serde(default)]
    pub bloom: bool,
    /// Motion blur toggle.
    #[serde(default)]
    pub motion_blur: bool,
    /// Raytraced global illumination feature toggle (maps to `gi` build).
    pub gi: bool,
    /// Particle / screen VFX feature toggle.
    pub vfx: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: ResolutionPreset::R1080p,
            vsync: true,
            quality: QualityPreset::High,
            resolution_scale: 1.0,
            shadow_quality: ShadowQuality::Medium,
            anti_aliasing: AntiAliasing::TAA,
            view_distance: 256,
            texture_quality: TextureQuality::High,
            ambient_occlusion: true,
            bloom: true,
            motion_blur: false,
            gi: false,
            vfx: true,
        }
    }
}

impl GraphicsSettings {
    /// Apply a convenience preset to all individual graphics controls.
    pub fn apply_preset(&mut self, preset: QualityPreset) {
        self.quality = preset;
        match preset {
            QualityPreset::Low => {
                self.resolution_scale = 0.5;
                self.shadow_quality = ShadowQuality::Low;
                self.anti_aliasing = AntiAliasing::FXAA;
                self.view_distance = 96;
                self.texture_quality = TextureQuality::Low;
                self.ambient_occlusion = false;
                self.bloom = false;
                self.motion_blur = false;
                self.gi = false;
                self.vfx = false;
            }
            QualityPreset::Medium => {
                self.resolution_scale = 1.0;
                self.shadow_quality = ShadowQuality::Medium;
                self.anti_aliasing = AntiAliasing::TAA;
                self.view_distance = 256;
                self.texture_quality = TextureQuality::Medium;
                self.ambient_occlusion = true;
                self.bloom = true;
                self.motion_blur = false;
                self.gi = false;
                self.vfx = true;
            }
            QualityPreset::High => {
                self.resolution_scale = 1.5;
                self.shadow_quality = ShadowQuality::High;
                self.anti_aliasing = AntiAliasing::MSAA;
                self.view_distance = 640;
                self.texture_quality = TextureQuality::High;
                self.ambient_occlusion = true;
                self.bloom = true;
                self.motion_blur = false;
                self.gi = true;
                self.vfx = true;
            }
            QualityPreset::Ultra => {
                self.resolution_scale = 2.0;
                self.shadow_quality = ShadowQuality::Ultra;
                self.anti_aliasing = AntiAliasing::MSAA;
                self.view_distance = 1024;
                self.texture_quality = TextureQuality::High;
                self.ambient_occlusion = true;
                self.bloom = true;
                self.motion_blur = true;
                self.gi = true;
                self.vfx = true;
            }
        }
    }
}

/// Display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Window mode.
    #[serde(default)]
    pub window_mode: WindowMode,
    /// Target framerate (unused when `fps_uncapped`).
    #[serde(default)]
    pub target_fps: u32,
    /// Uncapped framerate.
    #[serde(default)]
    pub fps_uncapped: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            window_mode: WindowMode::Windowed,
            target_fps: 120,
            fps_uncapped: false,
        }
    }
}

/// Audio mix volumes, each in `0.0..=1.0`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master output volume.
    pub master: f32,
    /// Music bus volume.
    pub music: f32,
    /// SFX bus volume.
    pub sfx: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { master: 0.8, music: 0.6, sfx: 0.8 }
    }
}

/// Gameplay options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Default simulation speed multiplier applied on load.
    pub default_sim_speed: f32,
    /// Whether autosave is enabled.
    pub autosave: bool,
    /// Autosave interval in minutes (ignored when `autosave` is false).
    pub autosave_minutes: u32,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self { default_sim_speed: 1.0, autosave: true, autosave_minutes: 5 }
    }
}

/// A single keybind row in the reference list (`action`, `key`).
///
/// Stored so the list survives serialization and can later become rebindable
/// without changing the resource shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybind {
    /// What the key does.
    pub action: String,
    /// The bound key label.
    pub key: String,
}

impl Keybind {
    fn new(action: &str, key: &str) -> Self {
        Self { action: action.into(), key: key.into() }
    }
}

/// The default hotkeys shipped with the reference client.
fn default_keybinds() -> Vec<Keybind> {
    vec![
        Keybind::new("Toggle Settings", "O"),
        Keybind::new("Toggle Diplomacy", "G"),
        Keybind::new("Toggle Tech Tree", "T"),
        Keybind::new("Pause / Resume Sim", "Space"),
        Keybind::new("Cycle Sim Speed", "+ / -"),
        Keybind::new("Pan Camera", "W A S D"),
        Keybind::new("Rotate Camera", "Q / E"),
        Keybind::new("Zoom Camera", "Mouse Wheel"),
        Keybind::new("Select / Inspect", "Left Click"),
        Keybind::new("Close Panel", "Esc"),
    ]
}

// ---------------------------------------------------------------------------
// Root resource
// ---------------------------------------------------------------------------

/// Persisted player settings + transient open/close state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Graphics / video group.
    pub graphics: GraphicsSettings,
    /// Display / windowing group.
    #[serde(default)]
    pub display: DisplaySettings,
    /// Audio mix group.
    pub audio: AudioSettings,
    /// Gameplay group.
    pub gameplay: GameplaySettings,
    /// Keybind reference list.
    pub keybinds: Vec<Keybind>,
    /// Active tab in the settings panel.
    #[serde(skip)]
    pub active_tab: SettingsTab,
    /// Whether the panel is currently visible (not persisted).
    #[serde(skip)]
    pub open: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            display: DisplaySettings::default(),
            audio: AudioSettings::default(),
            gameplay: GameplaySettings::default(),
            keybinds: default_keybinds(),
            active_tab: SettingsTab::default(),
            open: false,
        }
    }
}

impl GameSettings {
    /// Load settings from [`SETTINGS_PATH`], falling back to defaults when the
    /// file is missing or cannot be parsed.
    pub fn load() -> Self {
        match std::fs::read_to_string(SETTINGS_PATH) {
            Ok(text) => match ron::from_str::<GameSettings>(&text) {
                Ok(mut s) => {
                    if s.keybinds.is_empty() {
                        s.keybinds = default_keybinds();
                    }
                    s.active_tab = SettingsTab::default();
                    s.open = false;
                    s
                }
                Err(e) => {
                    warn!("settings.ron parse failed ({e}); using defaults");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Serialize and write to [`SETTINGS_PATH`].
    pub fn save(&self) {
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(text) => {
                if let Err(e) = std::fs::write(SETTINGS_PATH, text) {
                    error!("failed to write {SETTINGS_PATH}: {e}");
                }
            }
            Err(e) => error!("failed to serialize settings: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers [`GameSettings`] (loaded from disk) and wires the toggle + draw
/// systems. Does **not** add `EguiPlugin`.
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSettings::load())
            .add_systems(Startup, open_settings_for_autoshot)
            .add_systems(Update, toggle_settings_panel)
            .add_systems(EguiPrimaryContextPass, draw_settings_panel);
    }
}

/// Verification hook: when `CIVIS_SETTINGS_OPEN=1` is set, open the settings
/// Window at startup so a headless autoshot can frame the tabbed/granular page
/// (it is otherwise behind the `O` key and invisible in captures).
fn open_settings_for_autoshot(mut settings: ResMut<GameSettings>) {
    if std::env::var("CIVIS_SETTINGS_OPEN").as_deref() == Ok("1") {
        settings.open = true;
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn toggle_settings_panel(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<GameSettings>) {
    if keys.just_pressed(KeyCode::KeyO) {
        settings.open = !settings.open;
    }
    if settings.open && keys.just_pressed(KeyCode::Escape) {
        settings.open = false;
        settings.save();
    }
}

fn draw_settings_panel(mut contexts: EguiContexts, mut settings: ResMut<GameSettings>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if !settings.open {
        return;
    }

    ui_theme::apply_theme(ctx);
    let mut open = settings.open;
    let mut dirty = false;

    egui::Window::new("\u{2699} Settings")
        .open(&mut open)
        .default_size(egui::vec2(680.0, 540.0))
        .resizable(true)
        .collapsible(false)
        .frame(ui_theme::liquid_glass_frame(egui::Margin::same(14), 14))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    dirty = draw_settings_tabs(ui, &mut settings.active_tab);
                });
                ui.separator();
                ui.allocate_space(egui::vec2(4.0, 0.0));
                draw_settings_page(ui, &mut settings, &mut dirty);
            });
            ui_theme::hairline(ui);
                draw_footer(ui, &mut settings, &mut dirty);
        });

    if !open {
        settings.open = false;
        dirty = true;
    }

    if dirty {
        settings.save();
    }
}

fn draw_settings_tabs(ui: &mut egui::Ui, active_tab: &mut SettingsTab) -> bool {
    let mut changed = false;
    for tab in SettingsTab::ALL {
        let selected = *active_tab == tab;
        let color = if selected { ui_theme::ACCENT } else { ui_theme::TEXT };
        let label = egui::RichText::new(tab.label()).color(color).strong();
        if ui.selectable_label(selected, label).clicked() {
            *active_tab = tab;
            changed = true;
        }
    }
    changed
}

fn draw_settings_page(ui: &mut egui::Ui, settings: &mut GameSettings, dirty: &mut bool) {
    *dirty |= match settings.active_tab {
        SettingsTab::Graphics => graphics_tab(ui, &mut settings.graphics),
        SettingsTab::Display => display_tab(ui, &mut settings.display, &mut settings.graphics),
        SettingsTab::Audio => audio_tab(ui, &mut settings.audio),
        SettingsTab::Gameplay => gameplay_tab(ui, &mut settings.gameplay),
        SettingsTab::Controls => {
            keybinds_tab(ui, &settings.keybinds);
            false
        }
    };
}

fn draw_footer(ui: &mut egui::Ui, settings: &mut GameSettings, dirty: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            *dirty = true;
        }
        if ui.button("Reset to Defaults").clicked() {
            let keep = std::mem::take(&mut settings.keybinds);
            let mut def = GameSettings::default();
            def.keybinds = keep;
            def.open = true;
            *settings = def;
            *dirty = true;
        }
    });
}

fn section_heading(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.label(
        egui::RichText::new(format!("{icon}  {title}"))
            .color(ui_theme::ACCENT)
            .strong()
            .size(16.0),
    );
    ui.add_space(4.0);
}

fn enum_combo<T>(
    ui: &mut egui::Ui,
    label: &str,
    current: &mut T,
    all: &[T],
    to_text: impl Fn(T) -> &'static str + Copy,
) -> bool
where
    T: Copy + PartialEq,
{
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(ui_theme::DIM));
        let selected = to_text(*current).to_owned();
        egui::ComboBox::from_id_salt(label)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                for &entry in all {
                    changed |= ui.selectable_value(current, entry, to_text(entry)).changed();
                }
            });
    });
    changed
}

fn graphics_tab(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f5a5}", "Graphics");
    changed |= graphics_quality_preset_row(ui, g);
    changed |= graphics_quality_fields(ui, g);
    changed |= graphics_special_toggles(ui, g);
    changed
}

fn graphics_quality_preset_row(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    let mut preset = g.quality;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Quality preset").color(ui_theme::DIM));
        changed |= ui.selectable_value(&mut preset, QualityPreset::Low, QualityPreset::Low.label()).changed();
        changed |= ui.selectable_value(&mut preset, QualityPreset::Medium, QualityPreset::Medium.label()).changed();
        changed |= ui.selectable_value(&mut preset, QualityPreset::High, QualityPreset::High.label()).changed();
        changed |= ui.selectable_value(&mut preset, QualityPreset::Ultra, QualityPreset::Ultra.label()).changed();
    });
    if preset != g.quality {
        g.apply_preset(preset);
        changed = true;
    }
    changed
}

fn graphics_quality_fields(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    changed |= enum_combo(ui, "Shadows", &mut g.shadow_quality, &ShadowQuality::ALL, |v| v.label());
    changed |= enum_combo(
        ui,
        "Anti-aliasing",
        &mut g.anti_aliasing,
        &AntiAliasing::ALL,
        |v| v.label(),
    );
    changed |= enum_combo(
        ui,
        "Texture quality",
        &mut g.texture_quality,
        &TextureQuality::ALL,
        |v| v.label(),
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Resolution scale").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(&mut g.resolution_scale, 0.5..=2.0).show_value(true).fixed_decimals(2))
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("View distance").color(ui_theme::DIM));
        changed |= ui.add(egui::Slider::new(&mut g.view_distance, 64..=1024)).changed();
    });
    changed
}

fn graphics_special_toggles(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    changed |= ui.checkbox(&mut g.ambient_occlusion, "Ambient Occlusion").changed();
    changed |= ui.checkbox(&mut g.bloom, "Bloom").changed();
    changed |= ui.checkbox(&mut g.motion_blur, "Motion Blur").changed();
    changed |= ui.checkbox(&mut g.vsync, "VSync").changed();
    changed |= ui.checkbox(&mut g.gi, "Raytraced Global Illumination").changed();
    changed |= ui.checkbox(&mut g.vfx, "Particle / Screen VFX").changed();
    changed
}

fn display_tab(ui: &mut egui::Ui, display: &mut DisplaySettings, graphics: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f4fa}", "Display");

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Resolution").color(ui_theme::DIM));
        egui::ComboBox::from_id_salt("resolution")
            .selected_text(graphics.resolution.label())
            .show_ui(ui, |ui| {
                for res in ResolutionPreset::ALL {
                    changed |= ui.selectable_value(&mut graphics.resolution, res, res.label()).changed();
                }
            });
    });

    changed |= enum_combo(
        ui,
        "Window mode",
        &mut display.window_mode,
        &WindowMode::ALL,
        |m| m.label(),
    );

    changed |= ui.checkbox(&mut graphics.vsync, "VSync").changed();
    changed |= ui.checkbox(&mut display.fps_uncapped, "Uncapped").changed();
    if !display.fps_uncapped {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Target FPS").color(ui_theme::DIM));
            changed |= ui
                .add(egui::Slider::new(&mut display.target_fps, 30..=240).suffix(" fps"))
                .changed();
        });
    }
    changed
}

fn audio_tab(ui: &mut egui::Ui, a: &mut AudioSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f50a}", "Audio");
    changed |= volume_slider(ui, "Master", &mut a.master);
    changed |= volume_slider(ui, "Music", &mut a.music);
    changed |= volume_slider(ui, "SFX", &mut a.sfx);
    changed
}

fn volume_slider(ui: &mut egui::Ui, label: &str, value: &mut f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(ui_theme::DIM));
        changed = ui
            .add(egui::Slider::new(value, 0.0..=1.0).show_value(true).fixed_decimals(2))
            .changed();
    });
    changed
}

fn gameplay_tab(ui: &mut egui::Ui, p: &mut GameplaySettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f3ae}", "Gameplay");

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Default Sim Speed").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(&mut p.default_sim_speed, 0.25..=8.0).suffix("x"))
            .changed();
    });

    changed |= ui.checkbox(&mut p.autosave, "Autosave").changed();
    if p.autosave {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Autosave Interval").color(ui_theme::DIM));
            changed |= ui
                .add(egui::Slider::new(&mut p.autosave_minutes, 1..=60).suffix(" min"))
                .changed();
        });
    }
    changed
}

fn keybinds_tab(ui: &mut egui::Ui, binds: &[Keybind]) {
    section_heading(ui, "\u{2328}", "Controls");
    egui::Grid::new("keybinds")
        .num_columns(2)
        .striped(true)
        .spacing(egui::vec2(24.0, 4.0))
        .show(ui, |ui| {
            for b in binds {
                ui.label(egui::RichText::new(&b.action).color(ui_theme::TEXT));
                ui.label(egui::RichText::new(&b.key).color(ui_theme::ACCENT).strong());
                ui.end_row();
            }
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let s = GameSettings::default();
        assert_eq!(s.graphics.resolution, ResolutionPreset::R1080p);
        assert!(s.graphics.vsync);
        assert_eq!(s.display.window_mode, WindowMode::Windowed);
        assert_eq!(s.display.target_fps, 120);
        assert!(!s.keybinds.is_empty());
        assert!((s.audio.master - 0.8).abs() < f32::EPSILON);
        assert!((s.gameplay.default_sim_speed - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resolution_dimensions_match_labels() {
        assert_eq!(ResolutionPreset::R2160p.dimensions(), (3840, 2160));
        assert_eq!(ResolutionPreset::R720p.dimensions(), (1280, 720));
    }

    #[test]
    fn settings_round_trip_through_ron() {
        let s = GameSettings::default();
        let text = ron::ser::to_string_pretty(&s, ron::ser::PrettyConfig::default()).unwrap();
        let back: GameSettings = ron::from_str(&text).unwrap();
        assert_eq!(back.graphics.resolution, s.graphics.resolution);
        assert_eq!(back.graphics.quality, s.graphics.quality);
        assert_eq!(back.keybinds.len(), s.keybinds.len());
        // `open` and `active_tab` are `#[serde(skip)]` → default on round-trip.
        assert!(!back.open);
        assert_eq!(back.display.target_fps, s.display.target_fps);
    }
}
