#![cfg(all(feature = "bevy", feature = "egui"))]

//! Settings / Options panel for the Civis reference client.
//!
//! A themed (via [`crate::ui_theme`]) egui overlay covering the six standard
//! option groups players expect from a Cities-Skylines / Empire-at-War class
//! game:
//!
//! 1. **Graphics** — resolution and a quality preset plus granular rendering
//!    controls.
//! 2. **Display** — resolution, windowing mode, VSync, and framerate cap.
//! 3. **Audio** — master / music / SFX volumes.
//! 4. **Gameplay** — default sim speed and autosave interval.
//! 5. **Controls** — rebindable player hotkeys.
//! 6. **World / Game** — default gameplay and world/session knobs.
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
use serde::{
    de::{self, Deserializer},
    ser::Serializer,
    Deserialize, Serialize,
};

use crate::ui_theme;
#[cfg(feature = "audio")]
use bevy_kira_audio::prelude::AudioChannel;

const SETTINGS_PATH: &str = "settings.ron";

// ---------------------------------------------------------------------------
// Enums for preset-style options
// ---------------------------------------------------------------------------

/// A windowed/fullscreen resolution preset shown in the display group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResolutionPreset {
    /// 1280x720.
    R720p,
    /// 1920x1080.
    #[default]
    R1080p,
    /// 2560x1440.
    R1440p,
    /// 3840x2160.
    R2160p,
}

impl ResolutionPreset {
    /// All presets in menu order.
    pub const ALL: [ResolutionPreset; 4] = [Self::R720p, Self::R1080p, Self::R1440p, Self::R2160p];

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QualityPreset {
    /// Lowest settings — maximum framerate.
    Low,
    /// Balanced default.
    Medium,
    /// High fidelity.
    #[default]
    High,
    /// Everything maxed.
    Ultra,
    /// Manual / mixed settings.
    Custom,
}

impl QualityPreset {
    /// All presets in menu order.
    pub const ALL: [QualityPreset; 5] = [
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Ultra,
        Self::Custom,
    ];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
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
pub enum SettingsTab {
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
    /// World/session defaults and game-rule sliders.
    World,
}

impl SettingsTab {
    const ALL: [SettingsTab; 6] = [
        Self::Graphics,
        Self::Display,
        Self::Audio,
        Self::Gameplay,
        Self::World,
        Self::Controls,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Graphics => "Graphics",
            Self::Display => "Display",
            Self::Audio => "Audio",
            Self::Gameplay => "Gameplay",
            Self::Controls => "Controls",
            Self::World => "World / Game",
        }
    }
}

/// All settings tabs in panel order.
pub fn settings_tabs() -> &'static [SettingsTab] {
    &SettingsTab::ALL
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
    #[serde(default)]
    pub resolution: ResolutionPreset,
    /// Vertical sync (cap framerate to refresh rate).
    #[serde(default)]
    pub vsync: bool,
    /// Overall quality preset.
    #[serde(default)]
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
    #[serde(default)]
    pub gi: bool,
    /// Particle / screen VFX feature toggle.
    #[serde(default)]
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
            QualityPreset::Custom => {}
        }
    }

    pub fn mark_custom(&mut self) {
        if self.quality != QualityPreset::Custom {
            self.quality = QualityPreset::Custom;
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
    #[serde(default)]
    pub master: f32,
    /// Music bus volume.
    #[serde(default)]
    pub music: f32,
    /// SFX bus volume.
    #[serde(default)]
    pub sfx: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 0.8,
            music: 0.6,
            sfx: 0.8,
        }
    }
}

/// Gameplay options.
fn default_sim_speed() -> f32 {
    1.0
}

fn default_gameplay_half() -> f32 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Default simulation speed multiplier applied on load.
    #[serde(default = "default_sim_speed")]
    pub default_sim_speed: f32,
    /// Whether autosave is enabled.
    #[serde(default)]
    pub autosave: bool,
    /// Autosave interval in minutes (ignored when `autosave` is false).
    #[serde(default)]
    pub autosave_minutes: u32,
    /// Difficulty tuning for the simulation layer.
    #[serde(default = "default_gameplay_half")]
    pub difficulty: f32,
    /// Disaster frequency multiplier.
    #[serde(default = "default_gameplay_half")]
    pub disaster_frequency: f32,
    /// Emergence intensity multiplier.
    #[serde(default = "default_gameplay_half")]
    pub emergence_intensity: f32,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            default_sim_speed: 1.0,
            autosave: true,
            autosave_minutes: 5,
            difficulty: 0.5,
            disaster_frequency: 0.5,
            emergence_intensity: 0.5,
        }
    }
}

/// A binding target.
pub const ACTION_TOGGLE_SETTINGS: &str = "Toggle Settings";
pub const ACTION_TOGGLE_DIPLOMACY: &str = "Toggle Diplomacy";
pub const ACTION_TOGGLE_TECH_TREE: &str = "Toggle Tech Tree";
pub const ACTION_TOGGLE_MAP: &str = "Toggle Map";
pub const ACTION_PAUSE_SIM: &str = "Pause / Resume Sim";
pub const ACTION_CYCLE_SIM_SPEED: &str = "Cycle Sim Speed";
pub const ACTION_SPEED_1X: &str = "Set Speed 1x";
pub const ACTION_SPEED_2X: &str = "Set Speed 2x";
pub const ACTION_SPEED_5X: &str = "Set Speed 5x";
pub const ACTION_SPEED_10X: &str = "Set Speed 10x";
pub const ACTION_CAMERA_MOVE_FORWARD: &str = "Move Camera Forward";
pub const ACTION_CAMERA_MOVE_BACKWARD: &str = "Move Camera Backward";
pub const ACTION_CAMERA_MOVE_RIGHT: &str = "Move Camera Right";
pub const ACTION_CAMERA_MOVE_LEFT: &str = "Move Camera Left";
pub const ACTION_CAMERA_RAISE: &str = "Raise Camera";
pub const ACTION_CAMERA_LOWER: &str = "Lower Camera";
pub const ACTION_CAMERA_ROTATE: &str = "Rotate Camera";
pub const ACTION_CAMERA_ZOOM: &str = "Zoom Camera";
pub const ACTION_CAMERA_RESET: &str = "Reset Camera";
pub const ACTION_CAMERA_ZOOM_IN: &str = "Zoom Camera In";
pub const ACTION_CAMERA_ZOOM_OUT: &str = "Zoom Camera Out";
pub const ACTION_SELECT_OR_PICK: &str = "Select / Inspect";
pub const ACTION_CLOSE_PANEL: &str = "Close Panel";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyBinding {
    /// Keyboard key binding.
    Key(KeyCode),
    /// Mouse button binding.
    Mouse(MouseButton),
}

impl KeyBinding {
    fn to_token(&self) -> String {
        match self {
            Self::Key(key) => format!("key:{key:?}"),
            Self::Mouse(MouseButton::Left) => "mouse:left".to_string(),
            Self::Mouse(MouseButton::Right) => "mouse:right".to_string(),
            Self::Mouse(MouseButton::Middle) => "mouse:middle".to_string(),
            Self::Mouse(MouseButton::Back) => "mouse:back".to_string(),
            Self::Mouse(MouseButton::Forward) => "mouse:forward".to_string(),
            Self::Mouse(MouseButton::Other(index)) => format!("mouse:other:{index}"),
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        let (kind, value) = token.split_once(':')?;
        match kind {
            "key" => parse_key_token(value).map(KeyBinding::Key),
            "mouse" => match value {
                "left" => Some(KeyBinding::Mouse(MouseButton::Left)),
                "right" => Some(KeyBinding::Mouse(MouseButton::Right)),
                "middle" => Some(KeyBinding::Mouse(MouseButton::Middle)),
                "back" => Some(KeyBinding::Mouse(MouseButton::Back)),
                "forward" => Some(KeyBinding::Mouse(MouseButton::Forward)),
                _ => {
                    let (kind, index) = value.split_once(':')?;
                    match kind {
                        "other" => Some(KeyBinding::Mouse(MouseButton::Other(
                            index.parse::<u16>().ok()?,
                        ))),
                        _ => None,
                    }
                }
            },
            _ => None,
        }
    }

    #[inline]
    pub fn is_pressed(
        self,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(key) => keys.pressed(key),
            Self::Mouse(button) => mouse.pressed(button),
        }
    }

    #[inline]
    pub fn is_just_pressed(
        self,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(key) => keys.just_pressed(key),
            Self::Mouse(button) => mouse.just_pressed(button),
        }
    }

    fn label(self) -> String {
        match self {
            Self::Key(KeyCode::KeyO) => "O".into(),
            Self::Key(KeyCode::KeyG) => "G".into(),
            Self::Key(KeyCode::KeyT) => "T".into(),
            Self::Key(KeyCode::Space) => "Space".into(),
            Self::Key(KeyCode::Equal) => "=".into(),
            Self::Key(KeyCode::KeyW) => "W".into(),
            Self::Key(KeyCode::KeyQ) => "Q".into(),
            Self::Key(KeyCode::Escape) => "Esc".into(),
            Self::Mouse(MouseButton::Left) => "Left Click".into(),
            Self::Mouse(MouseButton::Right) => "Right Click".into(),
            Self::Mouse(MouseButton::Middle) => "Middle Click".into(),
            Self::Key(k) => format!("{k:?}"),
            Self::Mouse(m) => format!("{m:?}"),
        }
    }
}

fn parse_key_token(token: &str) -> Option<KeyCode> {
    match token {
        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),
        "Digit0" => Some(KeyCode::Digit0),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Space" => Some(KeyCode::Space),
        "Escape" => Some(KeyCode::Escape),
        "Backspace" => Some(KeyCode::Backspace),
        "Tab" => Some(KeyCode::Tab),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "Meta" => Some(KeyCode::Meta),
        "CapsLock" => Some(KeyCode::CapsLock),
        "Enter" => Some(KeyCode::Enter),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "Insert" => Some(KeyCode::Insert),
        "PageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "Delete" => Some(KeyCode::Delete),
        "Backquote" => Some(KeyCode::Backquote),
        "Minus" => Some(KeyCode::Minus),
        "Equal" => Some(KeyCode::Equal),
        "Backslash" => Some(KeyCode::Backslash),
        "BracketLeft" => Some(KeyCode::BracketLeft),
        "BracketRight" => Some(KeyCode::BracketRight),
        "Semicolon" => Some(KeyCode::Semicolon),
        "Quote" => Some(KeyCode::Quote),
        "Comma" => Some(KeyCode::Comma),
        "Period" => Some(KeyCode::Period),
        "Slash" => Some(KeyCode::Slash),
        "IntlBackslash" => Some(KeyCode::IntlBackslash),
        "IntlRo" => Some(KeyCode::IntlRo),
        "IntlYen" => Some(KeyCode::IntlYen),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        "Numpad0" => Some(KeyCode::Numpad0),
        "Numpad1" => Some(KeyCode::Numpad1),
        "Numpad2" => Some(KeyCode::Numpad2),
        "Numpad3" => Some(KeyCode::Numpad3),
        "Numpad4" => Some(KeyCode::Numpad4),
        "Numpad5" => Some(KeyCode::Numpad5),
        "Numpad6" => Some(KeyCode::Numpad6),
        "Numpad7" => Some(KeyCode::Numpad7),
        "Numpad8" => Some(KeyCode::Numpad8),
        "Numpad9" => Some(KeyCode::Numpad9),
        "NumLock" => Some(KeyCode::NumLock),
        _ => None,
    }
}

impl Serialize for KeyBinding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_token())
    }
}

impl<'de> Deserialize<'de> for KeyBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        KeyBinding::from_token(&token)
            .ok_or_else(|| de::Error::custom(format!("invalid key binding token: {token}")))
    }
}

impl std::fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label())
    }
}

// NOTE: do NOT add `impl Display for KeyCode` — that would violate Rust's
// orphan rules (KeyCode is foreign to this crate). `KeyBinding::label()`
// already produces the user-facing text via `format!("{k:?}")` and
// per-key overrides, so the orphan impl was dead code.

/// A single keybind row in the reference list (`action`, `binding`).
///
/// Stored so the list survives serialization and can later become rebindable
/// without changing the resource shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybind {
    /// What the key does.
    pub action: String,
    /// The bound input.
    pub binding: KeyBinding,
}

impl Keybind {
    fn new(action: &str, binding: KeyBinding) -> Self {
        Self {
            action: action.into(),
            binding,
        }
    }
}

/// The default hotkeys shipped with the reference client.
fn default_keybinds() -> Vec<Keybind> {
    vec![
        Keybind::new(ACTION_TOGGLE_SETTINGS, KeyBinding::Key(KeyCode::KeyO)),
        Keybind::new(ACTION_TOGGLE_DIPLOMACY, KeyBinding::Key(KeyCode::KeyG)),
        Keybind::new(ACTION_TOGGLE_TECH_TREE, KeyBinding::Key(KeyCode::KeyT)),
        Keybind::new(ACTION_TOGGLE_MAP, KeyBinding::Key(KeyCode::KeyM)),
        Keybind::new(ACTION_PAUSE_SIM, KeyBinding::Key(KeyCode::Space)),
        Keybind::new(ACTION_CYCLE_SIM_SPEED, KeyBinding::Key(KeyCode::Equal)),
        Keybind::new(ACTION_SPEED_1X, KeyBinding::Key(KeyCode::Digit1)),
        Keybind::new(ACTION_SPEED_2X, KeyBinding::Key(KeyCode::Digit2)),
        Keybind::new(ACTION_SPEED_5X, KeyBinding::Key(KeyCode::Digit3)),
        Keybind::new(ACTION_SPEED_10X, KeyBinding::Key(KeyCode::Digit4)),
        Keybind::new(
            ACTION_CAMERA_MOVE_FORWARD,
            KeyBinding::Key(KeyCode::KeyW),
        ),
        Keybind::new(
            ACTION_CAMERA_MOVE_BACKWARD,
            KeyBinding::Key(KeyCode::KeyS),
        ),
        Keybind::new(ACTION_CAMERA_MOVE_LEFT, KeyBinding::Key(KeyCode::KeyA)),
        Keybind::new(
            ACTION_CAMERA_MOVE_RIGHT,
            KeyBinding::Key(KeyCode::KeyD),
        ),
        Keybind::new(ACTION_CAMERA_RAISE, KeyBinding::Key(KeyCode::Space)),
        Keybind::new(ACTION_CAMERA_LOWER, KeyBinding::Key(KeyCode::ShiftLeft)),
        Keybind::new(ACTION_CAMERA_ROTATE, KeyBinding::Mouse(MouseButton::Right)),
        Keybind::new(ACTION_CAMERA_ZOOM, KeyBinding::Mouse(MouseButton::Middle)),
        Keybind::new(ACTION_CAMERA_RESET, KeyBinding::Key(KeyCode::KeyR)),
        Keybind::new(ACTION_CAMERA_ZOOM_IN, KeyBinding::Key(KeyCode::Equal)),
        Keybind::new(ACTION_CAMERA_ZOOM_OUT, KeyBinding::Key(KeyCode::Minus)),
        Keybind::new(ACTION_SELECT_OR_PICK, KeyBinding::Mouse(MouseButton::Left)),
        Keybind::new(ACTION_CLOSE_PANEL, KeyBinding::Key(KeyCode::Escape)),
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
    #[serde(default)]
    pub keybinds: Vec<Keybind>,
    /// Session/world defaults mirror.
    #[serde(default)]
    pub world: WorldSettings,
    /// Active tab in the settings panel.
    #[serde(skip)]
    pub active_tab: SettingsTab,
    /// Whether the panel is currently visible (not persisted).
    #[serde(skip)]
    pub open: bool,
}

/// Non-persisted state for rebinding capture.
#[derive(Resource, Debug, Default)]
struct KeybindCaptureState {
    pending_action: Option<String>,
    duplicate_warning: Option<String>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            display: DisplaySettings::default(),
            audio: AudioSettings::default(),
            gameplay: GameplaySettings::default(),
            keybinds: default_keybinds(),
            world: WorldSettings::default(),
            active_tab: SettingsTab::default(),
            open: false,
        }
    }
}

/// Session-level defaults shown in the World/Game tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSettings {
    /// Default world size mirror from the menu setup.
    #[serde(default)]
    pub world_size: usize,
    /// Default biome / era mirror if the menus expose it later.
    #[serde(default)]
    pub default_era: usize,
}

impl Default for WorldSettings {
    fn default() -> Self {
        Self {
            world_size: 1,
            default_era: 1,
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

    /// Look up the current binding for an action name.
    #[must_use]
    pub fn key_for(&self, action: &str) -> Option<KeyBinding> {
        self.keybinds
            .iter()
            .find(|bind| bind.action == action)
            .map(|bind| bind.binding)
    }

    /// Check whether a named action's binding is currently pressed.
    #[must_use]
    pub fn action_pressed(
        &self,
        action: &str,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.key_for(action)
            .is_some_and(|binding| binding.is_pressed(keys, mouse))
    }

    /// Check whether a named action's binding is newly pressed this frame.
    #[must_use]
    pub fn action_just_pressed(
        &self,
        action: &str,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.key_for(action)
            .is_some_and(|binding| binding.is_just_pressed(keys, mouse))
    }

    /// Update an action binding in-place.
    pub fn rebind(&mut self, action: &str, new_binding: KeyBinding) {
        if let Some(bind) = self.keybinds.iter_mut().find(|bind| bind.action == action) {
            bind.binding = new_binding;
        }
    }

    fn duplicate_binding(&self, action: &str, binding: KeyBinding) -> Option<String> {
        self.keybinds
            .iter()
            .find(|bind| bind.action != action && bind.binding == binding)
            .map(|bind| bind.action.clone())
    }

    fn reset_keybinds(&mut self) {
        self.keybinds = default_keybinds();
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
            .insert_resource(KeybindCaptureState::default())
            .add_systems(Update, (open_settings_for_autoshot, toggle_settings_panel))
            .add_systems(Update, capture_keybind_input)
            .add_systems(EguiPrimaryContextPass, draw_settings_panel);
        #[cfg(feature = "audio")]
        app.add_systems(Update, sync_audio_settings);
    }
}

/// Verification hook: when `CIVIS_SETTINGS_OPEN=1` is set, hold the settings
/// Window open so a headless autoshot can frame the tabbed/granular page (it is
/// otherwise behind the `O` key and invisible in captures).
///
/// Runs every frame (not just Startup) so the panel stays open through the whole
/// autoshot warm-up regardless of when the autostart→Playing transition or a
/// stray key event lands. The env var is read once via a `Local` cache.
fn open_settings_for_autoshot(
    mut settings: ResMut<GameSettings>,
    mut enabled: Local<Option<bool>>,
) {
    let on =
        *enabled.get_or_insert_with(|| std::env::var("CIVIS_SETTINGS_OPEN").as_deref() == Ok("1"));
    if on {
        settings.open = true;
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn toggle_settings_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<GameSettings>,
) {
    if settings.action_just_pressed(ACTION_TOGGLE_SETTINGS, &keys, &mouse_buttons) {
        settings.open = !settings.open;
    }
    if settings.open && settings.action_just_pressed(ACTION_CLOSE_PANEL, &keys, &mouse_buttons) {
        settings.open = false;
        settings.save();
    }
}

fn capture_keybind_input(
    mut settings: ResMut<GameSettings>,
    mut capture: ResMut<KeybindCaptureState>,
    mut keyboard_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut mouse_button_events: MessageReader<bevy::input::mouse::MouseButtonInput>,
) {
    let Some(action) = capture.pending_action.clone() else {
        capture.duplicate_warning = None;
        return;
    };

    for ev in keyboard_events.read() {
        if ev.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        let key = ev.key_code;
        if key == KeyCode::Escape {
            capture.pending_action = None;
            capture.duplicate_warning = None;
            return;
        }
        let binding = KeyBinding::Key(key);
        capture.duplicate_warning = settings.duplicate_binding(&action, binding);
        if capture.duplicate_warning.is_none() {
            if let Some(entry) = settings.keybinds.iter_mut().find(|b| b.action == action) {
                entry.binding = binding;
            }
            capture.pending_action = None;
        }
        return;
    }

    for ev in mouse_button_events.read() {
        if ev.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        let binding = KeyBinding::Mouse(ev.button);
        capture.duplicate_warning = settings.duplicate_binding(&action, binding);
        if capture.duplicate_warning.is_none() {
            if let Some(entry) = settings.keybinds.iter_mut().find(|b| b.action == action) {
                entry.binding = binding;
            }
            capture.pending_action = None;
        }
        return;
    }
}

fn draw_settings_panel(
    mut contexts: EguiContexts,
    mut settings: ResMut<GameSettings>,
    mut capture: ResMut<KeybindCaptureState>,
) {
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
                draw_settings_page(ui, &mut settings, &mut capture, &mut dirty);
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
        let color = if selected {
            ui_theme::ACCENT
        } else {
            ui_theme::TEXT
        };
        let label = egui::RichText::new(tab.label()).color(color).strong();
        if ui.selectable_label(selected, label).clicked() {
            *active_tab = tab;
            changed = true;
        }
    }
    changed
}

fn draw_settings_page(
    ui: &mut egui::Ui,
    settings: &mut GameSettings,
    capture: &mut KeybindCaptureState,
    dirty: &mut bool,
) {
    *dirty |= match settings.active_tab {
        SettingsTab::Graphics => graphics_tab(ui, &mut settings.graphics),
        SettingsTab::Display => display_tab(ui, &mut settings.display, &mut settings.graphics),
        SettingsTab::Audio => audio_tab(ui, &mut settings.audio),
        SettingsTab::Gameplay => gameplay_tab(ui, &mut settings.gameplay),
        SettingsTab::Controls => controls_tab(ui, settings, capture),
        SettingsTab::World => world_tab(ui, settings),
    };
}

fn draw_footer(ui: &mut egui::Ui, settings: &mut GameSettings, dirty: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            *dirty = true;
        }
        if ui.button("Reset to Defaults").clicked() {
            let mut def = GameSettings::default();
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
                    changed |= ui
                        .selectable_value(current, entry, to_text(entry))
                        .changed();
                }
            });
    });
    changed
}

fn graphics_tab(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f5a5}", "Graphics");
    changed |= graphics_quality_preset_row(ui, g);
    changed |= graphics_resolution_row(ui, g);
    changed |= graphics_quality_fields(ui, g);
    changed |= graphics_special_toggles(ui, g);
    changed
}

fn graphics_quality_preset_row(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    let mut preset = g.quality;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Quality preset").color(ui_theme::DIM));
        changed |= ui
            .selectable_value(&mut preset, QualityPreset::Low, QualityPreset::Low.label())
            .changed();
        changed |= ui
            .selectable_value(
                &mut preset,
                QualityPreset::Medium,
                QualityPreset::Medium.label(),
            )
            .changed();
        changed |= ui
            .selectable_value(
                &mut preset,
                QualityPreset::High,
                QualityPreset::High.label(),
            )
            .changed();
        changed |= ui
            .selectable_value(
                &mut preset,
                QualityPreset::Ultra,
                QualityPreset::Ultra.label(),
            )
            .changed();
        changed |= ui
            .selectable_value(
                &mut preset,
                QualityPreset::Custom,
                QualityPreset::Custom.label(),
            )
            .changed();
    });
    if preset != g.quality {
        g.apply_preset(preset);
        changed = true;
    }
    changed
}

fn graphics_resolution_row(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Resolution").color(ui_theme::DIM));
        egui::ComboBox::from_id_salt("graphics_resolution")
            .selected_text(g.resolution.label())
            .show_ui(ui, |ui| {
                for res in ResolutionPreset::ALL {
                    changed |= ui
                        .selectable_value(&mut g.resolution, res, res.label())
                        .changed();
                }
            });
    });
    if changed {
        g.mark_custom();
    }
    changed
}

fn graphics_quality_fields(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    changed |= enum_combo(
        ui,
        "Shadows",
        &mut g.shadow_quality,
        &ShadowQuality::ALL,
        |v| v.label(),
    );
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
            .add(
                egui::Slider::new(&mut g.resolution_scale, 0.5..=2.0)
                    .show_value(true)
                    .fixed_decimals(2),
            )
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("View distance").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(&mut g.view_distance, 64..=1024))
            .changed();
    });
    if changed {
        g.mark_custom();
    }
    changed
}

fn graphics_special_toggles(ui: &mut egui::Ui, g: &mut GraphicsSettings) -> bool {
    let mut changed = false;
    changed |= ui
        .checkbox(&mut g.ambient_occlusion, "Ambient Occlusion")
        .changed();
    changed |= ui.checkbox(&mut g.bloom, "Bloom").changed();
    changed |= ui.checkbox(&mut g.motion_blur, "Motion Blur").changed();
    changed |= ui.checkbox(&mut g.vsync, "VSync").changed();
    changed |= ui
        .checkbox(&mut g.gi, "Raytraced Global Illumination")
        .changed();
    changed |= ui.checkbox(&mut g.vfx, "Particle / Screen VFX").changed();
    if changed {
        g.mark_custom();
    }
    changed
}

fn controls_tab(
    ui: &mut egui::Ui,
    settings: &mut GameSettings,
    capture: &mut KeybindCaptureState,
) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{2328}", "Controls");
    if let Some(warn) = &capture.duplicate_warning {
        ui.label(egui::RichText::new(format!("Duplicate binding: {warn}")).color(ui_theme::RED));
    }
    ui.horizontal(|ui| {
        if ui.button("Reset to defaults").clicked() {
            settings.reset_keybinds();
            changed = true;
        }
    });
    ui.add_space(6.0);
    egui::Grid::new("keybinds")
        .num_columns(3)
        .striped(true)
        .spacing(egui::vec2(16.0, 6.0))
        .show(ui, |ui| {
            for bind in &settings.keybinds {
                ui.label(egui::RichText::new(&bind.action).color(ui_theme::TEXT));
                ui.label(
                    egui::RichText::new(bind.binding.to_string())
                        .color(ui_theme::ACCENT)
                        .strong(),
                );
                let rebinding = capture.pending_action.as_deref() == Some(bind.action.as_str());
                let button_text = if rebinding {
                    "Press a key…"
                } else {
                    "Rebind"
                };
                if ui.button(button_text).clicked() {
                    capture.pending_action = Some(bind.action.clone());
                    capture.duplicate_warning = None;
                }
                ui.end_row();
            }
        });
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Configured controls update instantly from the game settings.").color(ui_theme::DIM).small());
    changed
}

fn world_tab(ui: &mut egui::Ui, settings: &mut GameSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f30d}", "World / Game");
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Default sim speed").color(ui_theme::DIM));
        changed |= ui
            .add(
                egui::Slider::new(&mut settings.gameplay.default_sim_speed, 0.25..=8.0).suffix("x"),
            )
            .changed();
    });
    changed |= ui
        .checkbox(&mut settings.gameplay.autosave, "Autosave")
        .changed();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Autosave minutes").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(
                &mut settings.gameplay.autosave_minutes,
                1..=60,
            ))
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Difficulty").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(
                &mut settings.gameplay.difficulty,
                0.0..=1.0,
            ))
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Disaster frequency").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(
                &mut settings.gameplay.disaster_frequency,
                0.0..=1.0,
            ))
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Emergence intensity").color(ui_theme::DIM));
        changed |= ui
            .add(egui::Slider::new(
                &mut settings.gameplay.emergence_intensity,
                0.0..=1.0,
            ))
            .changed();
    });
    ui.separator();
    ui.label(egui::RichText::new("World size mirror").color(ui_theme::DIM));
    ui.label(
        egui::RichText::new(format!(
            "{} (mirrors menus.rs WorldSetupParams::world_size)",
            settings.world.world_size
        ))
        .color(ui_theme::ACCENT),
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Default starting era").color(ui_theme::DIM));
        ui.label(
            egui::RichText::new(format!("{}", settings.world.default_era)).color(ui_theme::ACCENT),
        );
    });
    ui.label(egui::RichText::new("Session defaults are read-only mirrors until the world setup menu is wired through settings.").color(ui_theme::DIM).small());
    changed
}

fn display_tab(
    ui: &mut egui::Ui,
    display: &mut DisplaySettings,
    _graphics: &mut GraphicsSettings,
) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f4fa}", "Display");

    changed |= enum_combo(
        ui,
        "Window mode",
        &mut display.window_mode,
        &WindowMode::ALL,
        |m| m.label(),
    );

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
            .add(
                egui::Slider::new(value, 0.0..=1.0)
                    .show_value(true)
                    .fixed_decimals(2),
            )
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

    ui.add_space(4.0);
    ui.separator();
    ui.label(egui::RichText::new("Sim Speed Presets").color(ui_theme::DIM));
    ui.horizontal(|ui| {
        for &spd in &[1u32, 2, 4, 8] {
            let label = format!("{}x", spd);
            let active = (p.default_sim_speed - spd as f32).abs() < 0.01;
            let btn = if active {
                ui.add(egui::Button::new(egui::RichText::new(&label).color(ui_theme::ACCENT)))
            } else {
                ui.button(&label)
            };
            if btn.clicked() {
                p.default_sim_speed = spd as f32;
                changed = true;
            }
        }
    });

    changed
}

#[cfg(feature = "audio")]
fn sync_audio_settings(
    settings: Res<GameSettings>,
    ambient: Option<Res<AudioChannel<crate::audio::AmbientChannel>>>,
    sfx_ch: Option<Res<AudioChannel<crate::audio::SfxChannel>>>,
) {
    if !settings.is_changed() { return; }
    if let Some(amb) = ambient {
        let vol = (settings.audio.master * settings.audio.music) as f64;
        amb.set_volume(vol);
    }
    if let Some(sfx) = sfx_ch {
        let vol = (settings.audio.master * settings.audio.sfx) as f64;
        sfx.set_volume(vol);
    }
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
        assert_eq!(s.world.world_size, 1);
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

    #[test]
    fn preset_application_fills_rich_fields() {
        let mut g = GraphicsSettings::default();
        g.apply_preset(QualityPreset::Ultra);
        assert_eq!(g.quality, QualityPreset::Ultra);
        assert_eq!(g.resolution_scale, 2.0);
        assert_eq!(g.shadow_quality, ShadowQuality::Ultra);
        assert_eq!(g.anti_aliasing, AntiAliasing::MSAA);
        assert_eq!(g.view_distance, 1024);
        assert_eq!(g.texture_quality, TextureQuality::High);
        assert!(g.ambient_occlusion);
        assert!(g.bloom);
        assert!(g.motion_blur);
        assert!(g.gi);
        assert!(g.vfx);
    }

    #[test]
    fn manual_change_flips_to_custom() {
        let mut g = GraphicsSettings::default();
        g.apply_preset(QualityPreset::High);
        g.shadow_quality = ShadowQuality::Low;
        g.mark_custom();
        assert_eq!(g.quality, QualityPreset::Custom);
    }

    #[test]
    fn custom_preset_is_authoritative_no_op() {
        let mut g = GraphicsSettings::default();
        let backup = g.clone();
        g.apply_preset(QualityPreset::Custom);
        assert_eq!(g.quality, QualityPreset::Custom);
        assert_eq!(g.resolution_scale, backup.resolution_scale);
        assert_eq!(g.shadow_quality, backup.shadow_quality);
    }

    #[test]
    fn key_for_looks_up_bindings() {
        let s = GameSettings::default();
        assert_eq!(
            s.key_for("Toggle Settings"),
            Some(KeyBinding::Key(KeyCode::KeyO))
        );
        assert_eq!(
            s.key_for("Zoom Camera"),
            Some(KeyBinding::Mouse(MouseButton::Middle))
        );
        assert_eq!(s.key_for("missing"), None);
    }

    #[test]
    fn duplicate_binding_detection_and_rebind_flow() {
        let mut s = GameSettings::default();
        assert_eq!(
            s.duplicate_binding("Toggle Settings", KeyBinding::Key(KeyCode::KeyG)),
            Some("Toggle Diplomacy".into())
        );
        assert_eq!(
            s.duplicate_binding("Toggle Settings", KeyBinding::Key(KeyCode::KeyP)),
            None
        );
        if let Some(entry) = s
            .keybinds
            .iter_mut()
            .find(|b| b.action == "Toggle Settings")
        {
            entry.binding = KeyBinding::Mouse(MouseButton::Right);
        }
        assert_eq!(
            s.key_for("Toggle Settings"),
            Some(KeyBinding::Mouse(MouseButton::Right))
        );
    }

    #[test]
    fn old_ron_loads_with_defaults() {
        let legacy = r#"(
            graphics: (
                resolution: R720p,
                vsync: true,
                quality: High,
                resolution_scale: 1.0,
                shadow_quality: Medium,
                anti_aliasing: TAA,
                view_distance: 256,
                texture_quality: High,
                ambient_occlusion: true,
                bloom: true,
                motion_blur: false,
                gi: false,
                vfx: true,
            ),
            display: (
                window_mode: Windowed,
                target_fps: 120,
                fps_uncapped: false,
            ),
            audio: (
                master: 0.8,
                music: 0.6,
                sfx: 0.8,
            ),
            gameplay: (
                default_sim_speed: 1.0,
                autosave: true,
                autosave_minutes: 5,
            ),
            keybinds: [
                (action: "Toggle Settings", binding: "key:KeyO"),
            ],
            world: (world_size: 1, default_era: 1),
        )"#;
        let s: GameSettings = ron::from_str(legacy).expect("legacy ron");
        assert_eq!(s.world.world_size, 1);
        assert_eq!(s.gameplay.difficulty, 0.5);
        assert_eq!(s.gameplay.disaster_frequency, 0.5);
        assert_eq!(s.gameplay.emergence_intensity, 0.5);
    }

    #[test]
    fn legacy_ron_without_default_sim_speed_uses_default() {
        let legacy = r#"(
            graphics: (
                resolution: R720p,
                vsync: true,
                quality: High,
                resolution_scale: 1.0,
                shadow_quality: Medium,
                anti_aliasing: TAA,
                view_distance: 256,
                texture_quality: High,
                ambient_occlusion: true,
                bloom: true,
                motion_blur: false,
                gi: false,
                vfx: true,
            ),
            display: (
                window_mode: Windowed,
                target_fps: 120,
                fps_uncapped: false,
            ),
            audio: (
                master: 0.8,
                music: 0.6,
                sfx: 0.8,
            ),
            gameplay: (
                autosave: true,
                autosave_minutes: 5,
            ),
            keybinds: [
                (action: "Toggle Settings", binding: "key:KeyO"),
            ],
            world: (world_size: 1, default_era: 1),
        )"#;

        let s: GameSettings = ron::from_str(legacy).expect("legacy ron missing gameplay default");
        assert_eq!(s.gameplay.default_sim_speed, 1.0);
    }
}
