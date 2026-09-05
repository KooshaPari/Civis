#![cfg(all(feature = "bevy", feature = "egui"))]

//! Settings / Options panel for the Civis reference client.
//!
//! **Intentionally local-only** — this panel manages client-side settings
//! (keybinds, audio, gameplay) and does not communicate with the server
//! via JSON-RPC.
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

/// Canonical persisted GPU backend preference, re-exported for API compatibility.
pub use crate::graphics_settings::BackendPref as RenderEngine;
use crate::live_stream::ServerBridge;
use crate::ui_theme;
#[cfg(feature = "audio")]
use bevy_kira_audio::prelude::AudioChannel;
#[cfg(feature = "audio")]
use bevy_kira_audio::AudioControl;

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
    /// Server connection settings (WebSocket endpoint URL, auth token, etc.).
    Network,
}

impl SettingsTab {
    const ALL: [SettingsTab; 7] = [
        Self::Graphics,
        Self::Display,
        Self::Audio,
        Self::Gameplay,
        Self::World,
        Self::Controls,
        Self::Network,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Graphics => "Graphics",
            Self::Display => "Display",
            Self::Audio => "Audio",
            Self::Gameplay => "Gameplay",
            Self::Controls => "Controls",
            Self::World => "World / Game",
            Self::Network => "Network",
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
    /// GPU API / render engine (DX12 Ultimate / Vulkan / Auto). Restart required.
    #[serde(default)]
    pub render_engine: RenderEngine,
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
    /// Bevy built-in SSAO pass toggle.
    #[serde(default = "default_true")]
    pub ssao_enabled: bool,
    /// Bevy built-in SSR pass toggle.
    #[serde(default = "default_true")]
    pub ssr_enabled: bool,
    /// Bevy built-in volumetric fog pass toggle.
    #[serde(default = "default_true")]
    pub volumetric_fog_enabled: bool,
    /// Bevy built-in tonemapping pass toggle.
    #[serde(default = "default_true")]
    pub tonemapping_enabled: bool,
    /// Bevy built-in color grading pass toggle.
    #[serde(default = "default_true")]
    pub color_grading_enabled: bool,
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
    // --- Phase 6.1 / Phase 7.3: custom GPU compute post-FX (civ_postfx) ---
    /// Master switch for the WGSL compute SSAO + Bloom + SSGI + ACES +
    /// Vignette + Chromatic + LUT passes.
    #[serde(default = "default_true")]
    pub postfx_enabled: bool,
    /// Per-pass toggle for the custom WGSL SSAO compute pass.
    #[serde(default = "default_true")]
    pub postfx_ssao: bool,
    /// Per-pass toggle for the custom WGSL Bloom compute pass.
    #[serde(default = "default_true")]
    pub postfx_bloom: bool,
    /// Per-pass toggle for the custom WGSL SSGI compute pass.
    #[serde(default = "default_true")]
    pub postfx_ssgi: bool,
    /// Per-pass toggle for the custom WGSL ACES tonemapping pass.
    #[serde(default = "default_true")]
    pub postfx_aces: bool,
    /// Per-pass toggle for the custom WGSL Vignette pass.
    #[serde(default = "default_true")]
    pub postfx_vignette: bool,
    /// Per-pass toggle for the custom WGSL Chromatic Aberration pass.
    #[serde(default = "default_true")]
    pub postfx_chromatic: bool,
    /// Per-pass toggle for the custom WGSL LUT color-grading pass.
    #[serde(default = "default_true")]
    pub postfx_lut: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            render_engine: RenderEngine::default(),
            resolution: ResolutionPreset::R1080p,
            vsync: true,
            quality: QualityPreset::High,
            resolution_scale: 1.0,
            shadow_quality: ShadowQuality::Medium,
            anti_aliasing: AntiAliasing::TAA,
            view_distance: 256,
            texture_quality: TextureQuality::High,
            ambient_occlusion: true,
            ssao_enabled: true,
            ssr_enabled: true,
            volumetric_fog_enabled: true,
            tonemapping_enabled: true,
            color_grading_enabled: true,
            bloom: true,
            motion_blur: false,
            gi: false,
            vfx: true,
            // Phase 6.1 / Phase 7.3 — match `CivPostFxToggle::default()` so
            // persisted settings files written before these fields existed
            // deserialize to a working "everything on" state.
            postfx_enabled: true,
            postfx_ssao: true,
            postfx_bloom: true,
            postfx_ssgi: true,
            postfx_aces: true,
            postfx_vignette: true,
            postfx_chromatic: true,
            postfx_lut: true,
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
                self.ssao_enabled = false;
                self.ssr_enabled = false;
                self.volumetric_fog_enabled = false;
                self.tonemapping_enabled = false;
                self.color_grading_enabled = false;
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
                self.ssao_enabled = true;
                self.ssr_enabled = true;
                self.volumetric_fog_enabled = true;
                self.tonemapping_enabled = true;
                self.color_grading_enabled = true;
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
                self.ssao_enabled = true;
                self.ssr_enabled = true;
                self.volumetric_fog_enabled = true;
                self.tonemapping_enabled = true;
                self.color_grading_enabled = true;
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
                self.ssao_enabled = true;
                self.ssr_enabled = true;
                self.volumetric_fog_enabled = true;
                self.tonemapping_enabled = true;
                self.color_grading_enabled = true;
                self.bloom = true;
                self.motion_blur = true;
                self.gi = true;
                self.vfx = true;
            }
            QualityPreset::Custom => {}
        }
        // Phase 6.1 / Phase 7.3 — preset-driven defaults for the custom WGSL
        // compute post-FX. Low turns everything off (perf), Medium keeps
        // Bloom (cheapest visual), High enables SSAO+Bloom+ACES+Vignette,
        // Ultra enables everything.
        match preset {
            QualityPreset::Low => {
                self.postfx_enabled = false;
                self.postfx_ssao = false;
                self.postfx_bloom = false;
                self.postfx_ssgi = false;
                self.postfx_aces = false;
                self.postfx_vignette = false;
                self.postfx_chromatic = false;
                self.postfx_lut = false;
            }
            QualityPreset::Medium => {
                self.postfx_enabled = true;
                self.postfx_ssao = false;
                self.postfx_bloom = true;
                self.postfx_ssgi = false;
                self.postfx_aces = true;
                self.postfx_vignette = false;
                self.postfx_chromatic = false;
                self.postfx_lut = false;
            }
            QualityPreset::High => {
                self.postfx_enabled = true;
                self.postfx_ssao = true;
                self.postfx_bloom = true;
                self.postfx_ssgi = false;
                self.postfx_aces = true;
                self.postfx_vignette = true;
                self.postfx_chromatic = false;
                self.postfx_lut = true;
            }
            QualityPreset::Ultra => {
                self.postfx_enabled = true;
                self.postfx_ssao = true;
                self.postfx_bloom = true;
                self.postfx_ssgi = true;
                self.postfx_aces = true;
                self.postfx_vignette = true;
                self.postfx_chromatic = true;
                self.postfx_lut = true;
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

fn default_true() -> bool {
    true
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
pub const ACTION_CAMERA_ORBIT_LEFT: &str = "Orbit Camera Left";
pub const ACTION_CAMERA_ORBIT_RIGHT: &str = "Orbit Camera Right";
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
    pub fn is_pressed(self, keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>) -> bool {
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
            Self::Key(KeyCode::KeyR) => "R".into(),
            Self::Key(KeyCode::KeyF) => "F".into(),
            Self::Key(KeyCode::KeyE) => "E".into(),
            Self::Key(KeyCode::Home) => "Home".into(),
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
        // Space = pause/resume; Esc still closes panels / toggles overlay via menus.
        Keybind::new(ACTION_PAUSE_SIM, KeyBinding::Key(KeyCode::Space)),
        Keybind::new(ACTION_CYCLE_SIM_SPEED, KeyBinding::Key(KeyCode::Equal)),
        Keybind::new(ACTION_SPEED_1X, KeyBinding::Key(KeyCode::Digit1)),
        Keybind::new(ACTION_SPEED_2X, KeyBinding::Key(KeyCode::Digit2)),
        Keybind::new(ACTION_SPEED_5X, KeyBinding::Key(KeyCode::Digit3)),
        Keybind::new(ACTION_SPEED_10X, KeyBinding::Key(KeyCode::Digit4)),
        Keybind::new(ACTION_CAMERA_MOVE_FORWARD, KeyBinding::Key(KeyCode::KeyW)),
        Keybind::new(ACTION_CAMERA_MOVE_BACKWARD, KeyBinding::Key(KeyCode::KeyS)),
        Keybind::new(ACTION_CAMERA_MOVE_LEFT, KeyBinding::Key(KeyCode::KeyA)),
        Keybind::new(ACTION_CAMERA_MOVE_RIGHT, KeyBinding::Key(KeyCode::KeyD)),
        Keybind::new(ACTION_CAMERA_RAISE, KeyBinding::Key(KeyCode::KeyR)),
        Keybind::new(ACTION_CAMERA_LOWER, KeyBinding::Key(KeyCode::KeyF)),
        Keybind::new(ACTION_CAMERA_ROTATE, KeyBinding::Mouse(MouseButton::Right)),
        Keybind::new(ACTION_CAMERA_ORBIT_LEFT, KeyBinding::Key(KeyCode::KeyQ)),
        Keybind::new(ACTION_CAMERA_ORBIT_RIGHT, KeyBinding::Key(KeyCode::KeyE)),
        Keybind::new(ACTION_CAMERA_ZOOM, KeyBinding::Mouse(MouseButton::Middle)),
        Keybind::new(ACTION_CAMERA_RESET, KeyBinding::Key(KeyCode::Home)),
        Keybind::new(ACTION_CAMERA_ZOOM_IN, KeyBinding::Key(KeyCode::Equal)),
        Keybind::new(ACTION_CAMERA_ZOOM_OUT, KeyBinding::Key(KeyCode::Minus)),
        Keybind::new(ACTION_SELECT_OR_PICK, KeyBinding::Mouse(MouseButton::Left)),
        Keybind::new(ACTION_CLOSE_PANEL, KeyBinding::Key(KeyCode::Escape)),
    ]
}

/// Fill missing actions and migrate the pre-2026-07 stock camera/pause layout.
fn reconcile_keybinds(keybinds: &mut Vec<Keybind>) {
    let stock_old = |action: &str, binding: KeyBinding| -> bool {
        keybinds
            .iter()
            .find(|b| b.action == action)
            .is_some_and(|b| b.binding == binding)
    };
    let still_stock_pause_raise = stock_old(ACTION_PAUSE_SIM, KeyBinding::Key(KeyCode::Escape))
        && stock_old(ACTION_CAMERA_RAISE, KeyBinding::Key(KeyCode::Space))
        && stock_old(ACTION_CAMERA_LOWER, KeyBinding::Key(KeyCode::ShiftLeft))
        && stock_old(ACTION_CAMERA_RESET, KeyBinding::Key(KeyCode::KeyR));
    if still_stock_pause_raise {
        for bind in keybinds.iter_mut() {
            match bind.action.as_str() {
                a if a == ACTION_PAUSE_SIM => bind.binding = KeyBinding::Key(KeyCode::Space),
                a if a == ACTION_CAMERA_RAISE => bind.binding = KeyBinding::Key(KeyCode::KeyR),
                a if a == ACTION_CAMERA_LOWER => bind.binding = KeyBinding::Key(KeyCode::KeyF),
                a if a == ACTION_CAMERA_RESET => bind.binding = KeyBinding::Key(KeyCode::Home),
                _ => {}
            }
        }
    }
    for def in default_keybinds() {
        if !keybinds.iter().any(|b| b.action == def.action) {
            keybinds.push(def);
        }
    }
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
    /// Server connection settings (WebSocket URL, auth token, etc.).
    #[serde(default)]
    pub network: NetworkSettings,
    /// Active tab in the settings panel.
    #[serde(skip)]
    pub active_tab: SettingsTab,
    /// Whether the panel is currently visible (not persisted).
    #[serde(skip)]
    pub open: bool,
    /// Whether the user has skipped/dismissed the tutorial. Persisted so it
    /// is not re-shown every launch. Defaults to false (show on first run).
    #[serde(default)]
    pub tutorial_skipped: bool,
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
            network: NetworkSettings::default(),
            active_tab: SettingsTab::default(),
            open: false,
            tutorial_skipped: false,
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

/// Server connection settings surfaced through the Network tab.
///
/// These values feed the `live_attach` bridge when the user picks
/// "Connect to Server" on the main menu. Default endpoint matches the
/// `civ-server` listen address used by the workspace tests / MCP smoke
/// (`ws://127.0.0.1:3800/ws`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// WebSocket URL of the `civ-server` JSON-RPC endpoint, e.g.
    /// `ws://127.0.0.1:3800/ws`. The UI rejects an empty string and falls
    /// back to the default.
    #[serde(default = "default_server_url")]
    pub server_url: String,
    /// Optional bearer token forwarded as the `Authorization` header on the
    /// WebSocket upgrade (matches `CIVIS_AUTH_TOKEN` on the server side).
    #[serde(default)]
    pub auth_token: String,
    /// Whether the client should auto-reconnect on disconnect.
    #[serde(default = "network_default_true")]
    pub auto_reconnect: bool,
    /// Snapshot polling interval in seconds (defaults to match the ws client's
    /// `SNAPSHOT_POLL_SECS = 2`).
    #[serde(default = "default_snapshot_interval")]
    pub snapshot_interval_secs: u32,
}

fn default_server_url() -> String {
    "ws://127.0.0.1:3800/ws".to_string()
}

/// `#[serde(default = ...)]` helper for `NetworkSettings::auto_reconnect` —
/// renamed to avoid colliding with the global `default_true` used by
/// `DisplaySettings` / `AudioSettings`.
fn network_default_true() -> bool {
    true
}

fn default_snapshot_interval() -> u32 {
    2
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
            auth_token: String::new(),
            auto_reconnect: true,
            snapshot_interval_secs: 2,
        }
    }
}

impl NetworkSettings {
    /// Return the persisted URL, falling back to the default if empty.
    #[must_use]
    pub fn resolved_url(&self) -> String {
        if self.server_url.trim().is_empty() {
            default_server_url()
        } else {
            self.server_url.clone()
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
                    } else {
                        reconcile_keybinds(&mut s.keybinds);
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
    ///
    /// Does not mutate process env here — wgpu/backend selection must be applied
    /// before adapter search via [`Self::apply_boot_render_engine`] (or a
    /// restart). Concurrent env mutation from Bevy UI schedules is undefined.
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

    /// Apply persisted render-engine preference before wgpu adapter search.
    pub fn apply_boot_render_engine() {
        let settings = Self::load();
        settings.graphics.render_engine.apply_to_env();
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
            .add_systems(
                EguiPrimaryContextPass,
                draw_settings_panel.run_if(crate::menus::in_playing),
            );
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
    bridge: Option<Res<ServerBridge>>,
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
        .default_size(egui::vec2(920.0, 640.0))
        .min_size(egui::vec2(720.0, 480.0))
        .resizable(true)
        .collapsible(false)
        .frame(ui_theme::liquid_glass_frame(egui::Margin::same(14), 14))
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(168.0, ui.available_height().max(420.0)),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(
                            egui::RichText::new("OPTIONS")
                                .size(12.0)
                                .color(ui_theme::DIM)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        dirty = draw_settings_tabs(ui, &mut settings.active_tab);
                    },
                );
                ui.separator();
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height().max(420.0)),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                draw_settings_page(
                                    ui,
                                    &mut settings,
                                    &mut capture,
                                    &mut dirty,
                                    bridge.as_deref(),
                                );
                            });
                    },
                );
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
        // Push gameplay policy to server so sim picks up difficulty/scarcity changes
        if let Some(ref bridge) = bridge {
            bridge.send_rpc(
                "sim.set_policy",
                serde_json::json!({
                    "scarcity_multiplier": settings.gameplay.difficulty,
                    "base_consumption_joules": (settings.gameplay.disaster_frequency * 10000.0) as u64,
                }),
            );
        }
    }
}

fn draw_settings_tabs(ui: &mut egui::Ui, active_tab: &mut SettingsTab) -> bool {
    let mut changed = false;
    for tab in SettingsTab::ALL {
        let selected = *active_tab == tab;
        let fill = if selected {
            ui_theme::ACCENT.gamma_multiply(0.22)
        } else {
            egui::Color32::TRANSPARENT
        };
        let label = egui::RichText::new(tab.label())
            .color(if selected {
                ui_theme::ACCENT
            } else {
                ui_theme::TEXT
            })
            .strong()
            .size(14.0);
        let response = ui.add_sized(
            egui::vec2(156.0, 36.0),
            egui::Button::new(label)
                .fill(fill)
                .corner_radius(egui::CornerRadius::same(6)),
        );
        if response.clicked() {
            *active_tab = tab;
            changed = true;
        }
        ui.add_space(4.0);
    }
    changed
}

fn draw_settings_page(
    ui: &mut egui::Ui,
    settings: &mut GameSettings,
    capture: &mut KeybindCaptureState,
    dirty: &mut bool,
    bridge: Option<&ServerBridge>,
) {
    *dirty |= match settings.active_tab {
        SettingsTab::Graphics => graphics_tab(ui, &mut settings.graphics),
        SettingsTab::Display => display_tab(ui, &mut settings.display, &mut settings.graphics),
        SettingsTab::Audio => audio_tab(ui, &mut settings.audio),
        SettingsTab::Gameplay => gameplay_tab(ui, &mut settings.gameplay),
        SettingsTab::Controls => controls_tab(ui, settings, capture),
        SettingsTab::World => world_tab(ui, settings),
        SettingsTab::Network => network_tab(ui, &mut settings.network, bridge),
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
    section_heading(ui, "\u{26a1}", "Render Engine");
    ui.label(
        egui::RichText::new(
            "Same style as AAA PC titles: pick the GPU API. Civis uses wgpu as the \
             engine layer over a native HAL — not GLES / browser WebGPU. Changing \
             the engine requires a restart.",
        )
        .color(ui_theme::DIM)
        .small(),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        changed |= enum_combo(
            ui,
            "Graphics API",
            &mut g.render_engine,
            &RenderEngine::ALL,
            RenderEngine::label,
        );
        ui.label(
            egui::RichText::new("Restart required")
                .color(egui::Color32::from_rgb(0xe0, 0xb0, 0x4a))
                .small()
                .strong(),
        );
    });
    ui.label(
        egui::RichText::new(match g.render_engine {
            RenderEngine::Auto => {
                "Auto → DX12 Ultimate on Windows, Vulkan on Linux, Metal on macOS."
            }
            RenderEngine::DX12 => {
                "DirectX 12 Ultimate — DXR / mesh shaders / DLSS path when the driver supports it."
            }
            RenderEngine::Vulkan => {
                "Vulkan — cross-vendor native HAL; full RT/DLSS parity on NVIDIA."
            }
        })
        .color(ui_theme::DIM)
        .small(),
    );
    ui.add_space(12.0);
    ui.separator();
    section_heading(ui, "\u{1f5a5}", "Quality");
    changed |= graphics_quality_preset_row(ui, g);
    changed |= graphics_resolution_row(ui, g);
    changed |= graphics_quality_fields(ui, g);
    ui.separator();
    section_heading(ui, "\u{2728}", "Post-process & advanced");
    changed |= graphics_special_toggles(ui, g);
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("Anisotropy / sharpen")
            .color(ui_theme::DIM)
            .small(),
    );
    ui.horizontal(|ui| {
        ui.label("Render scale");
        changed |= ui
            .add(
                egui::Slider::new(&mut g.resolution_scale, 0.5..=2.0)
                    .show_value(true)
                    .fixed_decimals(2),
            )
            .changed();
    });
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
    if ui.checkbox(&mut g.ssao_enabled, "SSAO").changed() {
        g.ambient_occlusion = g.ssao_enabled;
        changed = true;
    }
    changed |= ui.checkbox(&mut g.ssr_enabled, "SSR").changed();
    changed |= ui
        .checkbox(&mut g.volumetric_fog_enabled, "Volumetric Fog")
        .changed();
    changed |= ui
        .checkbox(&mut g.tonemapping_enabled, "Tonemapping")
        .changed();
    changed |= ui
        .checkbox(&mut g.color_grading_enabled, "Color Grading")
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
    ui.label(
        egui::RichText::new("Configured controls update instantly from the game settings.")
            .color(ui_theme::DIM)
            .small(),
    );
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

/// Network tab — lets the player configure the `civ-server` endpoint URL,
/// auth token, and reconnect cadence. "Test Connection" fires `sim.status`
/// over the [`ServerBridge`] so the player can confirm the endpoint
/// responds without leaving the settings screen.
fn network_tab(
    ui: &mut egui::Ui,
    network: &mut NetworkSettings,
    bridge: Option<&ServerBridge>,
) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f4f6}", "Network");
    ui.label(
        egui::RichText::new(
            "Server connection settings for the live-attach (`civ-server`) bridge. \
             The default endpoint matches the workspace smoke-test server.",
        )
        .color(ui_theme::DIM)
        .small(),
    );
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Server URL").color(ui_theme::DIM));
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut network.server_url)
                    .hint_text("ws://127.0.0.1:3800/ws")
                    .desired_width(ui.available_width() - 100.0),
            )
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Auth token").color(ui_theme::DIM));
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut network.auth_token)
                    .hint_text("(optional bearer token)")
                    .password(true)
                    .desired_width(ui.available_width() - 100.0),
            )
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Auto-reconnect").color(ui_theme::DIM));
        changed |= ui
            .checkbox(&mut network.auto_reconnect, "Reconnect on disconnect")
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Snapshot interval").color(ui_theme::DIM));
        changed |= ui
            .add(egui::DragValue::new(&mut network.snapshot_interval_secs).range(1..=60))
            .changed();
        ui.label(egui::RichText::new("seconds").color(ui_theme::DIM));
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "Effective URL: {}",
                network.resolved_url()
            ))
            .color(ui_theme::ACCENT)
            .small(),
        );
    });
    ui.horizontal(|ui| {
        if let Some(bridge) = bridge {
            if ui
                .add(egui::Button::new("\u{1f50d} Test Connection"))
                .on_hover_text("Fire sim.status at the server")
                .clicked()
            {
                bridge.send_rpc("sim.status", serde_json::json!({}));
            }
            if ui
                .add(egui::Button::new("\u{1f4e1} Force Reconnect"))
                .on_hover_text("Send sim.reset (force a fresh session)")
                .clicked()
            {
                bridge.send_rpc(
                    "sim.reset",
                    serde_json::json!({ "seed": 0 }),
                );
            }
        } else {
            ui.label(
                egui::RichText::new("(no live bridge attached)")
                    .color(ui_theme::DIM)
                    .italics(),
            );
        }
    });
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

    changed |= ui
        .checkbox(&mut display.fps_uncapped, "Uncapped framerate")
        .changed();
    if !display.fps_uncapped {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Target FPS").color(ui_theme::DIM));
            changed |= ui
                .add(egui::Slider::new(&mut display.target_fps, 30..=240).suffix(" fps"))
                .changed();
        });
    }
    ui.separator();
    section_heading(ui, "\u{1f4bb}", "Monitor & HUD");
    ui.label(
        egui::RichText::new(
            "UI scale, HDR output, and multi-monitor picker wire through display prefs next.",
        )
        .color(ui_theme::DIM)
        .small(),
    );
    let mut ui_scale = 1.0_f32;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("UI scale").color(ui_theme::DIM));
        let _ = ui.add(egui::Slider::new(&mut ui_scale, 0.75..=1.5).fixed_decimals(2));
    });
    let mut hdr = false;
    let _ = ui.checkbox(&mut hdr, "HDR output (when display supports)");
    let mut borderless = display.window_mode == WindowMode::Borderless;
    if ui
        .checkbox(&mut borderless, "Prefer borderless fullscreen")
        .changed()
    {
        display.window_mode = if borderless {
            WindowMode::Borderless
        } else {
            WindowMode::Fullscreen
        };
        changed = true;
    }
    changed
}

fn audio_tab(ui: &mut egui::Ui, a: &mut AudioSettings) -> bool {
    let mut changed = false;
    section_heading(ui, "\u{1f50a}", "Audio");
    changed |= volume_slider(ui, "Master", &mut a.master);
    changed |= volume_slider(ui, "Music", &mut a.music);
    changed |= volume_slider(ui, "SFX", &mut a.sfx);
    ui.separator();
    section_heading(ui, "\u{1f399}", "Mix");
    ui.label(
        egui::RichText::new(
            "Ambient / voice / UI buses land with the audio kit; sliders reserve the AAA layout.",
        )
        .color(ui_theme::DIM)
        .small(),
    );
    let mut ambient = (a.music * 0.85).clamp(0.0, 1.0);
    let mut ui_bus = (a.sfx * 0.7).clamp(0.0, 1.0);
    let mut voice = 0.8_f32;
    if volume_slider(ui, "Ambient", &mut ambient) {
        a.music = (ambient / 0.85).clamp(0.0, 1.0);
        changed = true;
    }
    if volume_slider(ui, "UI", &mut ui_bus) {
        a.sfx = (ui_bus / 0.7).clamp(0.0, 1.0);
        changed = true;
    }
    let _ = volume_slider(ui, "Voice (reserved)", &mut voice);
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
                ui.add(egui::Button::new(
                    egui::RichText::new(&label).color(ui_theme::ACCENT),
                ))
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
    if !settings.is_changed() {
        return;
    }
    if let Some(amb) = ambient {
        let vol = settings.audio.master * settings.audio.music;
        amb.set_volume(vol);
    }
    if let Some(sfx) = sfx_ch {
        let vol = settings.audio.master * settings.audio.sfx;
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
        assert_eq!(
            s.key_for(ACTION_PAUSE_SIM),
            Some(KeyBinding::Key(KeyCode::Space))
        );
        assert_eq!(
            s.key_for(ACTION_CAMERA_RAISE),
            Some(KeyBinding::Key(KeyCode::KeyR))
        );
        assert_eq!(
            s.key_for(ACTION_CAMERA_LOWER),
            Some(KeyBinding::Key(KeyCode::KeyF))
        );
        assert_eq!(
            s.key_for(ACTION_CAMERA_ORBIT_LEFT),
            Some(KeyBinding::Key(KeyCode::KeyQ))
        );
        assert_eq!(
            s.key_for(ACTION_CAMERA_ORBIT_RIGHT),
            Some(KeyBinding::Key(KeyCode::KeyE))
        );
        assert_eq!(
            s.key_for(ACTION_CAMERA_RESET),
            Some(KeyBinding::Key(KeyCode::Home))
        );
        assert!((s.audio.master - 0.8).abs() < f32::EPSILON);
        assert!((s.gameplay.default_sim_speed - 1.0).abs() < f32::EPSILON);
        assert_eq!(s.world.world_size, 1);
    }

    #[test]
    fn reconcile_migrates_stock_space_raise_scheme() {
        let mut keybinds = vec![
            Keybind::new(ACTION_PAUSE_SIM, KeyBinding::Key(KeyCode::Escape)),
            Keybind::new(ACTION_CAMERA_RAISE, KeyBinding::Key(KeyCode::Space)),
            Keybind::new(ACTION_CAMERA_LOWER, KeyBinding::Key(KeyCode::ShiftLeft)),
            Keybind::new(ACTION_CAMERA_RESET, KeyBinding::Key(KeyCode::KeyR)),
        ];
        reconcile_keybinds(&mut keybinds);
        let lookup = |action: &str| {
            keybinds
                .iter()
                .find(|b| b.action == action)
                .map(|b| b.binding)
        };
        assert_eq!(
            lookup(ACTION_PAUSE_SIM),
            Some(KeyBinding::Key(KeyCode::Space))
        );
        assert_eq!(
            lookup(ACTION_CAMERA_RAISE),
            Some(KeyBinding::Key(KeyCode::KeyR))
        );
        assert_eq!(
            lookup(ACTION_CAMERA_LOWER),
            Some(KeyBinding::Key(KeyCode::KeyF))
        );
        assert_eq!(
            lookup(ACTION_CAMERA_RESET),
            Some(KeyBinding::Key(KeyCode::Home))
        );
        assert_eq!(
            lookup(ACTION_CAMERA_ORBIT_LEFT),
            Some(KeyBinding::Key(KeyCode::KeyQ))
        );
        assert_eq!(
            lookup(ACTION_CAMERA_ORBIT_RIGHT),
            Some(KeyBinding::Key(KeyCode::KeyE))
        );
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
        assert!(g.ssao_enabled);
        assert!(g.ssr_enabled);
        assert!(g.volumetric_fog_enabled);
        assert!(g.tonemapping_enabled);
        assert!(g.color_grading_enabled);
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
    fn default_graphics_settings_enable_ssao() {
        assert!(GraphicsSettings::default().ssao_enabled);
    }

    #[test]
    fn default_graphics_settings_enable_ssr() {
        assert!(GraphicsSettings::default().ssr_enabled);
    }

    #[test]
    fn default_graphics_settings_enable_volumetric_fog() {
        assert!(GraphicsSettings::default().volumetric_fog_enabled);
    }

    #[test]
    fn default_graphics_settings_enable_tonemapping() {
        assert!(GraphicsSettings::default().tonemapping_enabled);
    }

    #[test]
    fn default_graphics_settings_enable_color_grading() {
        assert!(GraphicsSettings::default().color_grading_enabled);
    }

    #[test]
    fn render_engine_combo_labels_and_env_tokens_match_aaa_api_picker() {
        assert_eq!(RenderEngine::Auto.label(), "Auto (recommended)");
        assert_eq!(RenderEngine::DX12.label(), "DirectX 12 Ultimate");
        assert_eq!(RenderEngine::Vulkan.label(), "Vulkan");
        assert_eq!(RenderEngine::Auto.env_token(), None);
        assert_eq!(RenderEngine::DX12.env_token(), Some("dx12"));
        assert_eq!(RenderEngine::Vulkan.env_token(), Some("vulkan"));
        assert_eq!(
            ron::from_str::<RenderEngine>("DirectX12Ultimate").expect("legacy backend"),
            RenderEngine::DX12
        );
        assert_eq!(
            GraphicsSettings::default().render_engine,
            RenderEngine::Auto
        );
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
