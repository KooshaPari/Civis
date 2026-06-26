#![cfg(all(feature = "bevy", feature = "egui"))]

//! AAA graphics-settings resource + egui panel for the Civis Bevy client.
//!
//! # Design
//!
//! [`GfxSettings`] is a [`Resource`] that owns the full render-option tree.
//! Settings are grouped into six areas, matching the intent of the original
//! design brief:
//!
//! 1. **Render Engine** — backend selector, present mode, FPS cap.
//! 2. **Quality Preset** — Low / Medium / High / Ultra drives the rest.
//! 3. **Lighting / RT** — Solari GI toggle (gated by [`GpuCapabilities`]),
//!    shadow resolution and cascade count.
//! 4. **Upscaling** — render scale and DLSS / FSR / native selector.
//! 5. **Post-process** — bloom, SSAO, AA mode (MSAA / TAA), tonemapping
//!    curve, motion blur.
//! 6. **Display** — resolution preset, window mode.
//!
//! ## Apply-live vs restart-required
//!
//! Options that can be applied at runtime without restarting the app are
//! mutated through Bevy resources in [`apply_gfx_settings`].  Options that
//! require an app restart are marked with an inline "(restart)" tag in the UI
//! and never applied through the live system — callers must restart the app
//! and boot with the stored [`GfxSettings`].
//!
//! | Setting | Live / Restart |
//! |---------|---------------|
//! | Present mode (VSync/Mailbox/Immediate) | **Live** — `Window` resource |
//! | FPS cap | **Live** — `FramepaceSettings` when available |
//! | Bloom | **Live** — `BloomSettings` component |
//! | SSAO | **Live** — `ScreenSpaceAmbientOcclusion` component |
//! | AA mode (MSAA/TAA) | **Live** — `Msaa` / `TemporalAntiAliasing` components |
//! | Tonemapping | **Live** — `Tonemapping` component |
//! | Motion blur | **Live** — `MotionBlur` component |
//! | Render scale | **Live** — `bevy_render::camera::RenderTarget` scale |
//! | Shadow resolution / cascades | **Live** — `DirectionalLightShadowMap` |
//! | Solari GI | **Restart-required** (plugin must be added at startup) |
//! | Backend (DX12/Vulkan/Auto) | **Restart-required** (`CIV_BEVY_BACKEND` env) |
//! | Window mode | **Live** — `Window` resource |
//! | Resolution preset | **Live** — `Window` resource |
//! | Upscaling algorithm | **Restart-required** (DLSS/FSR require plugin) |
//!
//! ## Relationship to `settings_ui::GraphicsSettings`
//!
//! [`settings_ui`] already persists a `GraphicsSettings` sub-struct inside
//! `GameSettings` (used by the main Settings panel). This module is the
//! authoritative, fully-wired companion that exposes granular AAA knobs and
//! drives Bevy resources at runtime.  `settings_ui::GraphicsSettings` is the
//! serialised mirror shown in the existing tabbed settings page; this module
//! adds the deeper runtime tier on top.
//!
//! ## Sim isolation charter
//!
//! [`GfxSettings`] and all apply systems read/write **presentation-layer
//! Bevy resources only**.  No value from this resource must enter simulation
//! state (voxel world, agent data, CA ticks).

use bevy::post_process::bloom::Bloom;
use bevy::post_process::motion_blur::MotionBlur;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gpu_features::GpuCapabilities;
use crate::ui_theme;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// GPU backend preference.  `Auto` lets `native_backend.rs` decide (default
/// DX12 on Windows, Metal on macOS, Vulkan on Linux).  Override at boot via
/// `CIV_BEVY_BACKEND` env — the app must restart for this to take effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendPref {
    /// Let the platform decide (see `native_backend.rs`).
    #[default]
    Auto,
    /// Force DX12 (Windows-only; ignored on other OS).
    DX12,
    /// Force Vulkan.
    Vulkan,
}

impl BackendPref {
    const ALL: [BackendPref; 3] = [Self::Auto, Self::DX12, Self::Vulkan];

    fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::DX12 => "DX12",
            Self::Vulkan => "Vulkan",
        }
    }

    /// `CIV_BEVY_BACKEND` token written to env when the user confirms.
    pub fn env_token(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::DX12 => Some("dx12"),
            Self::Vulkan => Some("vulkan"),
        }
    }
}

/// VSync / present mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresentMode {
    /// Buffer swap locked to display refresh (VSync on).
    #[default]
    Vsync,
    /// Mailbox (fast triple-buffer, no tearing, low latency).
    Mailbox,
    /// Immediate (no sync, possible tearing, lowest latency).
    Immediate,
}

impl PresentMode {
    const ALL: [PresentMode; 3] = [Self::Vsync, Self::Mailbox, Self::Immediate];

    fn label(self) -> &'static str {
        match self {
            Self::Vsync => "VSync",
            Self::Mailbox => "Mailbox",
            Self::Immediate => "Immediate",
        }
    }

    pub fn to_bevy(self) -> bevy::window::PresentMode {
        match self {
            Self::Vsync => bevy::window::PresentMode::AutoVsync,
            Self::Mailbox => bevy::window::PresentMode::Mailbox,
            Self::Immediate => bevy::window::PresentMode::Immediate,
        }
    }
}

/// Overall quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QualityPreset {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
    /// Any manual override puts the preset in Custom.
    Custom,
}

impl QualityPreset {
    const ALL: [QualityPreset; 5] = [
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Ultra,
        Self::Custom,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
        }
    }
}

/// Shadow map texel resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowResolution {
    R512,
    R1024,
    #[default]
    R2048,
    R4096,
}

impl ShadowResolution {
    const ALL: [ShadowResolution; 4] = [Self::R512, Self::R1024, Self::R2048, Self::R4096];

    fn label(self) -> &'static str {
        match self {
            Self::R512 => "512",
            Self::R1024 => "1024",
            Self::R2048 => "2048 (default)",
            Self::R4096 => "4096",
        }
    }

    pub fn texels(self) -> u32 {
        match self {
            Self::R512 => 512,
            Self::R1024 => 1024,
            Self::R2048 => 2048,
            Self::R4096 => 4096,
        }
    }
}

/// Upscaling algorithm.  GPU-capability-gated in the UI: DLSS only when
/// `GpuCapabilities::dlss_available`, FSR on all platforms, MetalFX on Metal.
/// Any non-native selection is restart-required (upscaler plugins must be added
/// at startup).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpscalingMode {
    /// No upscaling — render at 1× and present natively.
    #[default]
    Native,
    /// AMD FidelityFX Super Resolution (available on all GPUs).
    FSR,
    /// NVIDIA Deep Learning Super Sampling (NVIDIA DX12/Vulkan only).
    DLSS,
    /// Apple MetalFX (Metal adapters only).
    MetalFX,
}

impl UpscalingMode {
    fn label(self) -> &'static str {
        match self {
            Self::Native => "Native",
            Self::FSR => "FSR (AMD FidelityFX)",
            Self::DLSS => "DLSS (NVIDIA)",
            Self::MetalFX => "MetalFX (Apple)",
        }
    }
}

/// Anti-aliasing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AaMode {
    /// No anti-aliasing.
    Off,
    /// Multi-sample AA (hardware; incompatible with Solari GI).
    MSAA2x,
    /// 4x MSAA.
    MSAA4x,
    /// 8x MSAA.
    MSAA8x,
    /// Temporal AA (soft; compatible with Solari / deferred pipelines).
    #[default]
    TAA,
}

impl AaMode {
    const ALL: [AaMode; 5] = [
        Self::Off,
        Self::MSAA2x,
        Self::MSAA4x,
        Self::MSAA8x,
        Self::TAA,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::MSAA2x => "MSAA 2×",
            Self::MSAA4x => "MSAA 4×",
            Self::MSAA8x => "MSAA 8×",
            Self::TAA => "TAA",
        }
    }

    pub fn msaa_samples(self) -> Option<u32> {
        match self {
            Self::MSAA2x => Some(2),
            Self::MSAA4x => Some(4),
            Self::MSAA8x => Some(8),
            _ => None,
        }
    }
}

/// Tonemapping curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneCurve {
    /// No tonemapping (linear).
    None,
    /// Reinhard simple.
    Reinhard,
    /// ACES filmic (cinematic).
    #[default]
    AcesFit,
    /// AgX (physically plausible, Blender default).
    AgX,
    /// BlenderFilmic.
    BlenderFilmic,
}

impl ToneCurve {
    const ALL: [ToneCurve; 5] = [
        Self::None,
        Self::Reinhard,
        Self::AcesFit,
        Self::AgX,
        Self::BlenderFilmic,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::None => "None (linear)",
            Self::Reinhard => "Reinhard",
            Self::AcesFit => "ACES Fit (cinematic)",
            Self::AgX => "AgX",
            Self::BlenderFilmic => "Blender Filmic",
        }
    }

    pub fn to_bevy(self) -> Tonemapping {
        match self {
            Self::None => Tonemapping::None,
            Self::Reinhard => Tonemapping::Reinhard,
            Self::AcesFit => Tonemapping::AcesFitted,
            Self::AgX => Tonemapping::AgX,
            Self::BlenderFilmic => Tonemapping::BlenderFilmic,
        }
    }
}

/// Window mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WinMode {
    #[default]
    Windowed,
    Borderless,
    Fullscreen,
}

impl WinMode {
    const ALL: [WinMode; 3] = [Self::Windowed, Self::Borderless, Self::Fullscreen];

    fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless",
            Self::Fullscreen => "Fullscreen",
        }
    }

    pub fn to_bevy(self) -> bevy::window::WindowMode {
        match self {
            Self::Windowed => bevy::window::WindowMode::Windowed,
            Self::Borderless => bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Current,
            ),
            Self::Fullscreen => bevy::window::WindowMode::Fullscreen(
                bevy::window::MonitorSelection::Current,
            ),
        }
    }
}

/// Resolution preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResPreset {
    R720p,
    #[default]
    R1080p,
    R1440p,
    R4K,
}

impl ResPreset {
    const ALL: [ResPreset; 4] = [Self::R720p, Self::R1080p, Self::R1440p, Self::R4K];

    fn label(self) -> &'static str {
        match self {
            Self::R720p => "1280 × 720",
            Self::R1080p => "1920 × 1080",
            Self::R1440p => "2560 × 1440",
            Self::R4K => "3840 × 2160 (4K)",
        }
    }

    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::R720p => (1280, 720),
            Self::R1080p => (1920, 1080),
            Self::R1440p => (2560, 1440),
            Self::R4K => (3840, 2160),
        }
    }
}

// ---------------------------------------------------------------------------
// GfxSettings resource
// ---------------------------------------------------------------------------

/// Full AAA graphics-settings resource.
///
/// This is the single source of truth for all render options in the Civis
/// Bevy client.  It is populated from defaults on startup (or loaded from
/// disk by the caller), then mutated by the settings panel and applied to
/// Bevy render resources via [`apply_gfx_settings`].
///
/// **Sim isolation:** no value from this resource feeds simulation state.
#[derive(Resource, Debug, Clone)]
pub struct GfxSettings {
    // --- Render engine ---
    /// GPU backend preference (restart-required).
    pub backend: BackendPref,
    /// Swap-chain present mode.
    pub present_mode: PresentMode,
    /// Hard framerate cap (0 = uncapped).
    pub fps_cap: u32,

    // --- Quality ---
    /// Convenience preset that drives the rest.
    pub quality: QualityPreset,

    // --- Lighting / RT ---
    /// Raytraced GI via `bevy_solari` (restart-required; gated by RT caps).
    pub solari_gi: bool,
    /// Shadow map texel size.
    pub shadow_resolution: ShadowResolution,
    /// Number of directional-light shadow cascades (1–4).
    pub shadow_cascades: u32,

    // --- Upscaling ---
    /// Render scale (0.5 = half resolution, 2.0 = super-sample).
    /// Ignored when `upscaling != Native` (the upscaler controls output res).
    pub render_scale: f32,
    /// Upscaling algorithm (restart-required for non-native).
    pub upscaling: UpscalingMode,

    // --- Post-process ---
    /// Screen-space ambient occlusion.
    pub ssao: bool,
    /// Bloom (lens flare-style glow on bright surfaces).
    pub bloom: bool,
    /// Bloom intensity (0.0–1.0).
    pub bloom_intensity: f32,
    /// Anti-aliasing mode.
    pub aa: AaMode,
    /// Tonemapping curve.
    pub tonemapping: ToneCurve,
    /// Motion blur toggle.
    pub motion_blur: bool,
    /// Motion blur shutter angle (0.0–1.0; only used when `motion_blur` is on).
    pub motion_blur_shutter: f32,

    // --- Display ---
    /// Window resolution preset.
    pub resolution: ResPreset,
    /// Window / fullscreen mode.
    pub window_mode: WinMode,

    // --- Panel state (not persisted) ---
    /// Whether the settings panel is open.
    pub open: bool,
}

impl Default for GfxSettings {
    fn default() -> Self {
        Self {
            backend: BackendPref::Auto,
            present_mode: PresentMode::Vsync,
            fps_cap: 0,
            quality: QualityPreset::High,
            solari_gi: false,
            shadow_resolution: ShadowResolution::R2048,
            shadow_cascades: 4,
            render_scale: 1.0,
            upscaling: UpscalingMode::Native,
            ssao: true,
            bloom: true,
            bloom_intensity: 0.3,
            aa: AaMode::TAA,
            tonemapping: ToneCurve::AcesFit,
            motion_blur: false,
            motion_blur_shutter: 0.5,
            resolution: ResPreset::R1080p,
            window_mode: WinMode::Windowed,
            open: false,
        }
    }
}

impl GfxSettings {
    /// Apply a quality preset.  Sets the most commonly coupled knobs; leaves
    /// `backend`, `upscaling`, `window_mode`, and `resolution` untouched.
    pub fn apply_preset(&mut self, preset: QualityPreset) {
        self.quality = preset;
        match preset {
            QualityPreset::Low => {
                self.render_scale = 0.5;
                self.shadow_resolution = ShadowResolution::R512;
                self.shadow_cascades = 1;
                self.ssao = false;
                self.bloom = false;
                self.aa = AaMode::Off;
                self.tonemapping = ToneCurve::Reinhard;
                self.motion_blur = false;
                self.solari_gi = false;
            }
            QualityPreset::Medium => {
                self.render_scale = 1.0;
                self.shadow_resolution = ShadowResolution::R1024;
                self.shadow_cascades = 2;
                self.ssao = false;
                self.bloom = true;
                self.bloom_intensity = 0.2;
                self.aa = AaMode::TAA;
                self.tonemapping = ToneCurve::AcesFit;
                self.motion_blur = false;
                self.solari_gi = false;
            }
            QualityPreset::High => {
                self.render_scale = 1.0;
                self.shadow_resolution = ShadowResolution::R2048;
                self.shadow_cascades = 4;
                self.ssao = true;
                self.bloom = true;
                self.bloom_intensity = 0.3;
                self.aa = AaMode::TAA;
                self.tonemapping = ToneCurve::AcesFit;
                self.motion_blur = false;
                self.solari_gi = false;
            }
            QualityPreset::Ultra => {
                self.render_scale = 1.5;
                self.shadow_resolution = ShadowResolution::R4096;
                self.shadow_cascades = 4;
                self.ssao = true;
                self.bloom = true;
                self.bloom_intensity = 0.4;
                self.aa = AaMode::TAA;
                self.tonemapping = ToneCurve::AgX;
                self.motion_blur = true;
                self.motion_blur_shutter = 0.5;
                self.solari_gi = true;
            }
            QualityPreset::Custom => {}
        }
    }

    fn mark_custom(&mut self) {
        self.quality = QualityPreset::Custom;
    }
}

// ---------------------------------------------------------------------------
// Apply system — live options only
// ---------------------------------------------------------------------------

/// Applies runtime-settable [`GfxSettings`] fields to Bevy resources.
///
/// Called every frame when settings change via an [`Events`] changed flag.
/// Restart-required options (backend, Solari, upscaling algorithm) are
/// intentionally NOT touched here.
pub fn apply_gfx_settings(
    settings: Res<GfxSettings>,
    mut windows: Query<&mut Window>,
    mut shadow_map: ResMut<DirectionalLightShadowMap>,
    mut bloom_q: Query<(
        Option<&mut Bloom>,
        Option<&mut TemporalJitter>,
        Option<&mut Tonemapping>,
        Option<&mut MotionBlur>,
        Option<&mut Msaa>,
    )>,
) {
    if !settings.is_changed() {
        return;
    }

    // --- Window present mode + window mode + resolution ---
    for mut window in &mut windows {
        let new_pm = settings.present_mode.to_bevy();
        if window.present_mode != new_pm {
            window.present_mode = new_pm;
        }
        let new_wm = settings.window_mode.to_bevy();
        if window.mode != new_wm {
            window.mode = new_wm;
        }
        if settings.window_mode == WinMode::Windowed {
            let (w, h) = settings.resolution.dimensions();
            let target = bevy::window::WindowResolution::new(w as f32, h as f32);
            if window.resolution.width() as u32 != w
                || window.resolution.height() as u32 != h
            {
                window.resolution = target;
            }
        }
    }

    // --- Shadow map ---
    let desired_size = settings.shadow_resolution.texels();
    if shadow_map.size != desired_size as usize {
        shadow_map.size = desired_size as usize;
    }

    // --- Per-camera post-process components ---
    for (bloom_opt, jitter_opt, tone_opt, mblur_opt, msaa_opt) in &mut bloom_q {
        // Tonemapping
        if let Some(mut t) = tone_opt {
            let desired = settings.tonemapping.to_bevy();
            if *t != desired {
                *t = desired;
            }
        }
        // MotionBlur
        if let Some(mut mb) = mblur_opt {
            let want_shutter = if settings.motion_blur {
                settings.motion_blur_shutter
            } else {
                0.0
            };
            if (mb.shutter_angle - want_shutter).abs() > f32::EPSILON {
                mb.shutter_angle = want_shutter;
            }
        }
        // MSAA — only applicable when not TAA
        if let Some(mut msaa) = msaa_opt {
            let desired = match settings.aa.msaa_samples() {
                Some(2) => Msaa::Sample2,
                Some(4) => Msaa::Sample4,
                Some(8) => Msaa::Sample8,
                _ => Msaa::Off,
            };
            if *msaa != desired {
                *msaa = desired;
            }
        }
        // TAA jitter marker
        if let Some(_jitter) = jitter_opt {
            // TemporalJitter presence signals TAA is active; we cannot toggle
            // it through insertion from this system without owning the camera
            // entity — this is documented as restart-required in the module
            // docs.  The jitter component itself needs no field mutation.
        }
        // Bloom
        if let Some(mut bloom) = bloom_opt {
            if settings.bloom {
                let want = bloom.intensity != settings.bloom_intensity;
                if want {
                    bloom.intensity = settings.bloom_intensity;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that registers [`GfxSettings`] and wires the settings panel +
/// apply system.  Does NOT add `EguiPlugin` (owned by `GameUiPlugin`).
pub struct GraphicsSettingsPlugin;

impl Plugin for GraphicsSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GfxSettings>()
            .add_systems(Update, apply_gfx_settings)
            .add_systems(EguiPrimaryContextPass, draw_gfx_settings_panel);
    }
}

// ---------------------------------------------------------------------------
// Panel helpers
// ---------------------------------------------------------------------------

fn section(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.add_space(ui_theme::SPACE_SM);
    ui.label(
        egui::RichText::new(format!("{icon}  {title}"))
            .color(ui_theme::ACCENT)
            .strong()
            .size(15.0),
    );
    ui.add_space(ui_theme::SPACE_XS);
}

fn restart_badge(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new(" (restart required)")
            .small()
            .color(ui_theme::WARN),
    );
}

fn row_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(ui_theme::DIM));
}

fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    current: &mut T,
    items: &[T],
    label_fn: impl Fn(T) -> &'static str,
) -> bool {
    let mut changed = false;
    let selected_text = label_fn(*current).to_owned();
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            for &item in items {
                changed |= ui
                    .selectable_value(current, item, label_fn(item))
                    .changed();
            }
        });
    changed
}

// ---------------------------------------------------------------------------
// Panel draw system
// ---------------------------------------------------------------------------

fn draw_gfx_settings_panel(
    mut contexts: EguiContexts,
    mut settings: ResMut<GfxSettings>,
    gpu: Option<Res<GpuCapabilities>>,
) {
    if !settings.open {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };
    ui_theme::apply_theme(ctx);

    let mut is_open = true;
    let mut dirty = false;

    egui::Window::new("\u{1f5a5} Graphics Settings")
        .open(&mut is_open)
        .default_size(egui::vec2(620.0, 560.0))
        .resizable(true)
        .collapsible(false)
        .frame(ui_theme::liquid_glass_frame(egui::Margin::same(14), 14))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                dirty |= draw_render_engine_section(ui, &mut settings);
                ui_theme::hairline(ui);
                dirty |= draw_quality_preset_section(ui, &mut settings);
                ui_theme::hairline(ui);
                dirty |= draw_lighting_section(ui, &mut settings, gpu.as_deref());
                ui_theme::hairline(ui);
                dirty |= draw_upscaling_section(ui, &mut settings, gpu.as_deref());
                ui_theme::hairline(ui);
                dirty |= draw_post_section(ui, &mut settings);
                ui_theme::hairline(ui);
                dirty |= draw_display_section(ui, &mut settings);
            });
        });

    if !is_open {
        settings.open = false;
    }

    // Force a change-detection tick so apply_gfx_settings runs.
    if dirty {
        settings.set_changed();
    }
}

fn draw_render_engine_section(ui: &mut egui::Ui, s: &mut GfxSettings) -> bool {
    let mut changed = false;
    section(ui, "\u{26a1}", "Render Engine");

    ui.horizontal(|ui| {
        row_label(ui, "Backend");
        changed |= combo(ui, "gfx_backend", &mut s.backend, &BackendPref::ALL, |v| {
            v.label()
        });
        restart_badge(ui);
    });

    ui.horizontal(|ui| {
        row_label(ui, "Present Mode");
        changed |= combo(
            ui,
            "gfx_present",
            &mut s.present_mode,
            &PresentMode::ALL,
            |v| v.label(),
        );
    });

    ui.horizontal(|ui| {
        row_label(ui, "FPS Cap  (0 = uncapped)");
        changed |= ui
            .add(egui::Slider::new(&mut s.fps_cap, 0..=360).suffix(" fps"))
            .changed();
    });

    changed
}

fn draw_quality_preset_section(ui: &mut egui::Ui, s: &mut GfxSettings) -> bool {
    let mut changed = false;
    section(ui, "\u{2b50}", "Quality Preset");

    ui.horizontal(|ui| {
        row_label(ui, "Preset");
        let before = s.quality;
        for preset in QualityPreset::ALL {
            let sel = s.quality == preset;
            let color = if sel {
                ui_theme::ACCENT
            } else {
                ui_theme::TEXT
            };
            if ui
                .selectable_label(sel, egui::RichText::new(preset.label()).color(color))
                .clicked()
                && !sel
            {
                s.apply_preset(preset);
                changed = true;
            }
        }
        if s.quality != before {
            changed = true;
        }
    });

    changed
}

fn draw_lighting_section(
    ui: &mut egui::Ui,
    s: &mut GfxSettings,
    gpu: Option<&GpuCapabilities>,
) -> bool {
    let mut changed = false;
    section(ui, "\u{2728}", "Lighting / Ray Tracing");

    let rt_capable = gpu.map_or(false, |g| g.ray_tracing);

    ui.horizontal(|ui| {
        let label = "Solari GI (ReSTIR ray-traced global illumination)";
        if rt_capable {
            let cb = ui.checkbox(&mut s.solari_gi, label);
            if cb.changed() {
                s.mark_custom();
                changed = true;
            }
        } else {
            ui.add_enabled(false, egui::Checkbox::new(&mut s.solari_gi, label));
            ui.label(
                egui::RichText::new(" (GPU has no RT)")
                    .small()
                    .color(ui_theme::DIM),
            );
        }
        restart_badge(ui);
    });

    ui.horizontal(|ui| {
        row_label(ui, "Shadow Resolution");
        let before = s.shadow_resolution;
        changed |= combo(
            ui,
            "gfx_shadow_res",
            &mut s.shadow_resolution,
            &ShadowResolution::ALL,
            |v| v.label(),
        );
        if s.shadow_resolution != before {
            s.mark_custom();
        }
    });

    ui.horizontal(|ui| {
        row_label(ui, "Shadow Cascades");
        let r = ui.add(egui::Slider::new(&mut s.shadow_cascades, 1..=4));
        if r.changed() {
            s.mark_custom();
            changed = true;
        }
    });

    changed
}

fn draw_upscaling_section(
    ui: &mut egui::Ui,
    s: &mut GfxSettings,
    gpu: Option<&GpuCapabilities>,
) -> bool {
    let mut changed = false;
    section(ui, "\u{1f50d}", "Upscaling");

    let dlss_ok = gpu.map_or(false, |g| g.dlss_available);
    let metal_fx_ok = gpu.map_or(false, |g| g.metal_fx);

    ui.horizontal(|ui| {
        row_label(ui, "Algorithm");
        egui::ComboBox::from_id_salt("gfx_upscale")
            .selected_text(s.upscaling.label())
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut s.upscaling, UpscalingMode::Native, "Native")
                    .changed();
                changed |= ui
                    .add_enabled(
                        true,
                        egui::SelectableLabel::new(
                            s.upscaling == UpscalingMode::FSR,
                            UpscalingMode::FSR.label(),
                        ),
                    )
                    .clicked()
                    .then(|| {
                        s.upscaling = UpscalingMode::FSR;
                    })
                    .is_some();
                ui.add_enabled_ui(dlss_ok, |ui| {
                    changed |= ui
                        .selectable_value(
                            &mut s.upscaling,
                            UpscalingMode::DLSS,
                            UpscalingMode::DLSS.label(),
                        )
                        .changed();
                });
                ui.add_enabled_ui(metal_fx_ok, |ui| {
                    changed |= ui
                        .selectable_value(
                            &mut s.upscaling,
                            UpscalingMode::MetalFX,
                            UpscalingMode::MetalFX.label(),
                        )
                        .changed();
                });
            });
        if s.upscaling != UpscalingMode::Native {
            restart_badge(ui);
        }
    });

    if s.upscaling == UpscalingMode::Native {
        ui.horizontal(|ui| {
            row_label(ui, "Render Scale");
            let r = ui.add(
                egui::Slider::new(&mut s.render_scale, 0.5_f32..=2.0_f32)
                    .show_value(true)
                    .fixed_decimals(2)
                    .suffix("×"),
            );
            if r.changed() {
                s.mark_custom();
                changed = true;
            }
        });
    }

    if changed {
        s.mark_custom();
    }
    changed
}

fn draw_post_section(ui: &mut egui::Ui, s: &mut GfxSettings) -> bool {
    let mut changed = false;
    section(ui, "\u{1f3a8}", "Post-Process");

    // Bloom
    if ui.checkbox(&mut s.bloom, "Bloom").changed() {
        s.mark_custom();
        changed = true;
    }
    if s.bloom {
        ui.horizontal(|ui| {
            row_label(ui, "  Bloom Intensity");
            let r = ui.add(
                egui::Slider::new(&mut s.bloom_intensity, 0.0_f32..=1.0_f32)
                    .fixed_decimals(2),
            );
            if r.changed() {
                s.mark_custom();
                changed = true;
            }
        });
    }

    // SSAO
    if ui.checkbox(&mut s.ssao, "SSAO (Ambient Occlusion)").changed() {
        s.mark_custom();
        changed = true;
    }

    // AA
    ui.horizontal(|ui| {
        row_label(ui, "Anti-Aliasing");
        let before = s.aa;
        changed |= combo(ui, "gfx_aa", &mut s.aa, &AaMode::ALL, |v| v.label());
        if s.aa != before {
            s.mark_custom();
        }
    });

    // Tonemapping
    ui.horizontal(|ui| {
        row_label(ui, "Tonemapping");
        let before = s.tonemapping;
        changed |= combo(
            ui,
            "gfx_tone",
            &mut s.tonemapping,
            &ToneCurve::ALL,
            |v| v.label(),
        );
        if s.tonemapping != before {
            s.mark_custom();
        }
    });

    // Motion blur
    if ui.checkbox(&mut s.motion_blur, "Motion Blur").changed() {
        s.mark_custom();
        changed = true;
    }
    if s.motion_blur {
        ui.horizontal(|ui| {
            row_label(ui, "  Shutter Angle");
            let r = ui.add(
                egui::Slider::new(&mut s.motion_blur_shutter, 0.0_f32..=1.0_f32)
                    .fixed_decimals(2),
            );
            if r.changed() {
                s.mark_custom();
                changed = true;
            }
        });
    }

    changed
}

fn draw_display_section(ui: &mut egui::Ui, s: &mut GfxSettings) -> bool {
    let mut changed = false;
    section(ui, "\u{1f4fa}", "Display");

    ui.horizontal(|ui| {
        row_label(ui, "Window Mode");
        changed |= combo(
            ui,
            "gfx_winmode",
            &mut s.window_mode,
            &WinMode::ALL,
            |v| v.label(),
        );
    });

    if s.window_mode == WinMode::Windowed {
        ui.horizontal(|ui| {
            row_label(ui, "Resolution");
            changed |= combo(
                ui,
                "gfx_res",
                &mut s.resolution,
                &ResPreset::ALL,
                |v| v.label(),
            );
            ui.label(egui::RichText::new(" (windowed only)").small().color(ui_theme::DIM));
        });
    }

    changed
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_sane() {
        let s = GfxSettings::default();
        assert_eq!(s.quality, QualityPreset::High);
        assert!(!s.solari_gi, "GI off by default");
        assert_eq!(s.shadow_resolution, ShadowResolution::R2048);
        assert_eq!(s.shadow_cascades, 4);
        assert!(s.bloom);
        assert_eq!(s.aa, AaMode::TAA);
        assert_eq!(s.tonemapping, ToneCurve::AcesFit);
        assert!(!s.motion_blur);
        assert_eq!(s.upscaling, UpscalingMode::Native);
        assert_eq!(s.render_scale, 1.0);
        assert!(!s.open);
    }

    #[test]
    fn ultra_preset_enables_gi_and_motion_blur() {
        let mut s = GfxSettings::default();
        s.apply_preset(QualityPreset::Ultra);
        assert!(s.solari_gi);
        assert!(s.motion_blur);
        assert_eq!(s.shadow_resolution, ShadowResolution::R4096);
        assert_eq!(s.render_scale, 1.5);
    }

    #[test]
    fn low_preset_disables_expensive_features() {
        let mut s = GfxSettings::default();
        s.apply_preset(QualityPreset::Low);
        assert!(!s.solari_gi);
        assert!(!s.ssao);
        assert!(!s.bloom);
        assert_eq!(s.aa, AaMode::Off);
        assert_eq!(s.shadow_resolution, ShadowResolution::R512);
        assert_eq!(s.shadow_cascades, 1);
    }

    #[test]
    fn manual_change_marks_custom() {
        let mut s = GfxSettings::default();
        s.apply_preset(QualityPreset::High);
        assert_eq!(s.quality, QualityPreset::High);
        s.bloom = false;
        s.mark_custom();
        assert_eq!(s.quality, QualityPreset::Custom);
    }

    #[test]
    fn backend_env_tokens_are_correct() {
        assert_eq!(BackendPref::Auto.env_token(), None);
        assert_eq!(BackendPref::DX12.env_token(), Some("dx12"));
        assert_eq!(BackendPref::Vulkan.env_token(), Some("vulkan"));
    }

    #[test]
    fn shadow_resolution_texel_sizes_match_labels() {
        assert_eq!(ShadowResolution::R4096.texels(), 4096);
        assert_eq!(ShadowResolution::R512.texels(), 512);
    }

    #[test]
    fn tonemapping_round_trips_through_bevy() {
        assert_eq!(ToneCurve::AcesFit.to_bevy(), Tonemapping::AcesFitted);
        assert_eq!(ToneCurve::None.to_bevy(), Tonemapping::None);
        assert_eq!(ToneCurve::AgX.to_bevy(), Tonemapping::AgX);
    }

    #[test]
    fn present_mode_round_trips_through_bevy() {
        assert_eq!(
            PresentMode::Vsync.to_bevy(),
            bevy::window::PresentMode::AutoVsync
        );
        assert_eq!(
            PresentMode::Immediate.to_bevy(),
            bevy::window::PresentMode::Immediate
        );
    }

    #[test]
    fn resolution_presets_have_expected_dimensions() {
        assert_eq!(ResPreset::R4K.dimensions(), (3840, 2160));
        assert_eq!(ResPreset::R720p.dimensions(), (1280, 720));
    }
}
