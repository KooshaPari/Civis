#![cfg(all(feature = "bevy", feature = "egui"))]

//! Settings / Options panel for the Civis reference client.
//!
//! A themed (via [`crate::ui_theme`]) egui overlay covering the four standard
//! option groups players expect from a Cities-Skylines / Empire-at-War class
//! game:
//!
//! 1. **Graphics** — resolution preset, vsync, an overall quality preset, plus
//!    per-feature toggles for the heavyweight render passes (raytraced GI, VFX).
//! 2. **Audio** — master / music / SFX volumes.
//! 3. **Gameplay** — default sim speed and autosave interval.
//! 4. **Keybinds** — a read-only reference list of the client's hotkeys.
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

/// File the settings persist to, resolved relative to the working directory.
const SETTINGS_PATH: &str = "settings.ron";

// ---------------------------------------------------------------------------
// Enums for preset-style options
// ---------------------------------------------------------------------------

/// A windowed/fullscreen resolution preset shown in the graphics group.
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
    /// Raytraced global illumination feature toggle (maps to the `gi` build).
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
            gi: false,
            vfx: true,
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
    /// Audio mix group.
    pub audio: AudioSettings,
    /// Gameplay group.
    pub gameplay: GameplaySettings,
    /// Keybind reference list.
    pub keybinds: Vec<Keybind>,
    /// Whether the panel is currently visible (not persisted).
    #[serde(skip)]
    pub open: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            gameplay: GameplaySettings::default(),
            keybinds: default_keybinds(),
            open: false,
        }
    }
}

impl GameSettings {
    /// Load settings from [`SETTINGS_PATH`], falling back to defaults when the
    /// file is missing or cannot be parsed (a corrupt file should never block
    /// startup — it is reported via `warn!` and replaced on next save).
    pub fn load() -> Self {
        match std::fs::read_to_string(SETTINGS_PATH) {
            Ok(text) => match ron::from_str::<GameSettings>(&text) {
                Ok(mut s) => {
                    // A persisted file with an empty keybind list (older schema)
                    // should still surface the reference hotkeys.
                    if s.keybinds.is_empty() {
                        s.keybinds = default_keybinds();
                    }
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

    /// Serialize and write to [`SETTINGS_PATH`]. Errors are logged loudly rather
    /// than silently swallowed.
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
            .add_systems(Update, toggle_settings_panel)
            .add_systems(EguiPrimaryContextPass, draw_settings_panel);
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
        .default_size(egui::vec2(560.0, 480.0))
        .resizable(true)
        .collapsible(false)
        .frame(ui_theme::accent_frame(egui::Margin::same(14), ui_theme::ACCENT))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                dirty |= graphics_section(ui, &mut settings.graphics);
                ui_theme::hairline(ui);
                dirty |= audio_section(ui, &mut settings.audio);
                ui_theme::hairline(ui);
                dirty |= gameplay_section(ui, &mut settings.gameplay);
                ui_theme::hairline(ui);
                keybinds_section(ui, &settings.keybinds);

                ui_theme::hairline(ui);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        dirty = true;
                    }
                    if ui.button("Reset to Defaults").clicked() {
                        let keep = std::mem::take(&mut settings.keybinds);
                        let mut def = GameSettings::default();
                        def.keybinds = keep;
                        def.open = true;
                        *settings = def;
                        dirty = true;
                    }
                });
            });
        });

    // Esc inside the egui window closes via the `[x]`; reflect that + persist.
    if !open {
        settings.open = false;
        dirty = true;
    }

    if dirty {
        settings.save();
    }
}

/// Section heading helper using the themed accent colour.
fn section_heading(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.label(
        egui::RichText::new(format!("{icon}  {title}"))
            .color(ui_theme::ACCENT)
            .strong()
            .size(16.0),
    );
    ui.add_space(4.0);
}

fn graphics_section(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f5a5}", "Graphics");

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Resolution").color(ui_theme::DIM));
        egui::ComboBox::from_id_salt("res")
            .selected_text(g.resolution.label())
            .show_ui(ui, |ui| {
                for r in ResolutionPreset::ALL {
                    changed |= ui.selectable_value(&mut g.resolution, r, r.label()).changed();
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Quality").color(ui_theme::DIM));
        egui::ComboBox::from_id_salt("quality")
            .selected_text(g.quality.label())
            .show_ui(ui, |ui| {
                for q in QualityPreset::ALL {
                    changed |= ui.selectable_value(&mut g.quality, q, q.label()).changed();
                }
            });
    });

    changed |= ui.checkbox(&mut g.vsync, "VSync").changed();
    changed |= ui.checkbox(&mut g.gi, "Raytraced Global Illumination").changed();
    changed |= ui.checkbox(&mut g.vfx, "Particle / Screen VFX").changed();
    changed
}

fn audio_section(ui: &mut egui::Ui, a: &mut AudioSettings) -> bool {
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

fn gameplay_section(ui: &mut egui::Ui, p: &mut GameplaySettings) -> bool {
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

fn keybinds_section(ui: &mut egui::Ui, binds: &[Keybind]) {
    section_heading(ui, "\u{2328}", "Keybinds");
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
        // `open` is `#[serde(skip)]` → always false after a round-trip.
        assert!(!back.open);
    }
}
