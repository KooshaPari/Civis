#![cfg(all(feature = "bevy", feature = "egui"))]

//! Menus and overlay plugin for the Civis reference client (FR-CIV-BEVY-024 / item 49).
//! Settings GPU readout: FR-CIV-BEVY-036 / item 61.

use crate::gpu_features::GpuCapabilities;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const OVERLAY_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 160);

/// Whether the game is currently playing or paused (overlay visible).
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum GameUiMode {
    /// Normal gameplay — no pause overlay.
    #[default]
    Playing,
    /// Pause overlay is shown; in-process sim ticks halt.
    Paused,
}

/// Timed era-advancement banner shown at the top of the viewport.
#[derive(Resource, Default, Debug)]
pub struct EraBanner {
    /// Name of the era being announced (empty when no banner is active).
    pub current_era: String,
    /// Seconds remaining until the banner disappears.
    pub show_timer: f32,
}

impl EraBanner {
    /// Trigger the banner for `era`, displaying it for 4 seconds.
    pub fn announce(&mut self, era: impl Into<String>) {
        self.current_era = era.into();
        self.show_timer = 4.0;
    }
}

/// Controls visibility of the settings window.
#[derive(Resource, Default, Debug)]
pub struct SettingsOpen(pub bool);

/// Transient state for the settings window (no persistence yet).
#[derive(Resource, Debug)]
pub struct SettingsState {
    /// 0 = Low, 1 = Medium, 2 = High, 3 = Ultra
    pub graphics_quality: usize,
    /// 0.0 – 1.0
    pub master_volume: f32,
    /// Tick speed multiplier stub.
    pub sim_speed: u32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            graphics_quality: 2,
            master_volume: 0.8,
            sim_speed: 1,
        }
    }
}

/// Bevy plugin: pause overlay, era banners, settings window.
pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUiMode>()
            .init_resource::<EraBanner>()
            .init_resource::<SettingsOpen>()
            .init_resource::<SettingsState>()
            .add_systems(Update, (toggle_pause, tick_era_banner))
            .add_systems(
                EguiPrimaryContextPass,
                (draw_pause_menu, draw_era_banner, draw_settings_window),
            );
    }
}

fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<GameUiMode>) {
    if keys.just_pressed(KeyCode::Escape) {
        *mode = match *mode {
            GameUiMode::Playing => GameUiMode::Paused,
            GameUiMode::Paused => GameUiMode::Playing,
        };
    }
}

fn tick_era_banner(mut banner: ResMut<EraBanner>, time: Res<Time>) {
    if banner.show_timer > 0.0 {
        banner.show_timer = (banner.show_timer - time.delta_secs()).max(0.0);
    }
}

fn draw_pause_menu(
    mut contexts: EguiContexts,
    mut mode: ResMut<GameUiMode>,
    mut settings_open: ResMut<SettingsOpen>,
    mut exit: MessageWriter<AppExit>,
) {
    if *mode != GameUiMode::Paused {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    dim_overlay(ctx);
    egui::Area::new(egui::Id::new("pause_panel_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            pause_panel(ui, &mut mode, &mut settings_open, &mut exit)
        });
}

fn draw_era_banner(mut contexts: EguiContexts, banner: Res<EraBanner>) {
    if banner.show_timer <= 0.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Area::new(egui::Id::new("era_banner_area"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 24.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| era_banner(ui, &banner));
}

fn draw_settings_window(
    mut contexts: EguiContexts,
    mut settings_open: ResMut<SettingsOpen>,
    mut state: ResMut<SettingsState>,
    gpu_caps: Option<Res<GpuCapabilities>>,
) {
    if !settings_open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    settings_window(ctx, &mut settings_open, &mut state, gpu_caps.as_deref());
}

fn dim_overlay(ctx: &egui::Context) {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("pause_dim_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(screen, egui::CornerRadius::ZERO, OVERLAY_DIM);
        });
}

fn pause_panel(
    ui: &mut egui::Ui,
    mode: &mut GameUiMode,
    settings_open: &mut SettingsOpen,
    exit: &mut MessageWriter<AppExit>,
) {
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(egui::Stroke::new(1.5, ACCENT.gamma_multiply(0.5)))
        .inner_margin(egui::Margin::same(32))
        .show(ui, |ui| {
            ui.set_min_width(280.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("\u{23f8} PAUSED")
                        .size(28.0)
                        .color(ACCENT)
                        .strong(),
                );
                ui.add_space(20.0);
                pause_menu_buttons(ui, mode, settings_open, exit);
            });
        });
}

fn pause_menu_buttons(
    ui: &mut egui::Ui,
    mode: &mut GameUiMode,
    settings_open: &mut SettingsOpen,
    exit: &mut MessageWriter<AppExit>,
) {
    if menu_button(ui, "\u{25b6}  Resume").clicked() {
        *mode = GameUiMode::Playing;
    }
    ui.add_space(6.0);
    if menu_button(ui, "\u{2699}  Settings").clicked() {
        settings_open.0 = !settings_open.0;
    }
    ui.add_space(6.0);
    menu_button(ui, "\u{1f4be}  Save");
    ui.add_space(6.0);
    menu_button(ui, "\u{1f30d}  New World");
    ui.add_space(14.0);
    ui.separator();
    ui.add_space(10.0);
    if menu_button(ui, "\u{23fb}  Quit").clicked() {
        exit.write(AppExit::Success);
    }
}

fn era_banner(ui: &mut egui::Ui, banner: &EraBanner) {
    const TOTAL: f32 = 4.0;
    const FADE_IN: f32 = 0.4;
    const FADE_OUT: f32 = 0.8;
    let elapsed = TOTAL - banner.show_timer;
    let alpha = if elapsed < FADE_IN {
        elapsed / FADE_IN
    } else if banner.show_timer < FADE_OUT {
        banner.show_timer / FADE_OUT
    } else {
        1.0
    }
    .clamp(0.0, 1.0);
    let panel_fill = egui::Color32::from_rgba_unmultiplied(17, 20, 31, (220.0 * alpha) as u8);
    let text_color = egui::Color32::from_rgba_unmultiplied(
        ACCENT.r(),
        ACCENT.g(),
        ACCENT.b(),
        (255.0 * alpha) as u8,
    );
    egui::Frame::NONE
        .fill(panel_fill)
        .corner_radius(egui::CornerRadius::same(10))
        .stroke(egui::Stroke::new(1.0, text_color))
        .inner_margin(egui::Margin::symmetric(40, 14))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("\u{27d0} Entering the {} Era", banner.current_era))
                    .size(20.0)
                    .color(text_color)
                    .strong(),
            );
        });
}

fn settings_window(
    ctx: &egui::Context,
    settings_open: &mut SettingsOpen,
    state: &mut SettingsState,
    gpu_caps: Option<&GpuCapabilities>,
) {
    const QUALITIES: &[&str] = &["Low", "Medium", "High", "Ultra"];
    egui::Window::new(
        egui::RichText::new("\u{2699} Settings")
            .color(ACCENT)
            .strong(),
    )
    .collapsible(false)
    .resizable(false)
    .min_width(320.0)
    .frame(
        egui::Frame::NONE
            .fill(PANEL_FILL)
            .corner_radius(egui::CornerRadius::same(10))
            .stroke(egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.4)))
            .inner_margin(egui::Margin::same(18)),
    )
    .open(&mut settings_open.0)
    .show(ctx, |ui| settings_rows(ui, state, QUALITIES, gpu_caps));
}

fn settings_rows(
    ui: &mut egui::Ui,
    state: &mut SettingsState,
    qualities: &[&str],
    gpu_caps: Option<&GpuCapabilities>,
) {
    ui.label(egui::RichText::new("Graphics Quality").color(DIM).small());
    egui::ComboBox::from_id_salt("graphics_quality_combo")
        .selected_text(*qualities.get(state.graphics_quality).unwrap_or(&"High"))
        .show_ui(ui, |ui| {
            for (i, &label) in qualities.iter().enumerate() {
                ui.selectable_value(&mut state.graphics_quality, i, label);
            }
        });
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Master Volume").color(DIM).small());
    ui.add(egui::Slider::new(&mut state.master_volume, 0.0..=1.0).show_value(true));
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Sim Speed").color(DIM).small());
    ui.add(
        egui::Slider::new(&mut state.sim_speed, 1..=10)
            .text("x")
            .show_value(true),
    );
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    gpu_capabilities_settings_section(ui, gpu_caps);
}

/// User-facing yes/no for read-only GPU capability flags.
#[must_use]
pub fn format_gpu_capability_flag(enabled: bool) -> &'static str {
    if enabled {
        "Yes"
    } else {
        "No"
    }
}

/// Read-only settings labels for detected GPU capabilities (FR-CIV-BEVY-036).
#[must_use]
pub fn format_gpu_settings_labels(caps: &GpuCapabilities) -> Vec<(&'static str, String)> {
    vec![
        ("Backend", caps.backend_name.clone()),
        ("Est. VRAM", format_gpu_vram_label_mb(caps.max_vram_mb)),
        (
            "Ray tracing",
            format_gpu_capability_flag(caps.ray_tracing).to_string(),
        ),
        (
            "DLSS",
            format_gpu_capability_flag(caps.dlss_available).to_string(),
        ),
        (
            "FSR",
            format_gpu_capability_flag(caps.fsr_available).to_string(),
        ),
    ]
}

/// Format estimated VRAM for the settings panel.
#[must_use]
pub fn format_gpu_vram_label_mb(max_vram_mb: u32) -> String {
    if max_vram_mb == 0 {
        "Unknown".to_string()
    } else {
        format!("{max_vram_mb} MB")
    }
}

/// Message when [`GpuCapabilities`] is not on the main world yet (headless / pre-startup).
#[must_use]
pub fn format_gpu_capabilities_unavailable_message() -> &'static str {
    "GPU capabilities unavailable (headless or still starting up)"
}

fn gpu_capabilities_settings_section(ui: &mut egui::Ui, gpu_caps: Option<&GpuCapabilities>) {
    ui.label(
        egui::RichText::new("GPU (detected)")
            .color(DIM)
            .small()
            .strong(),
    );
    ui.add_space(4.0);
    let Some(caps) = gpu_caps else {
        ui.label(
            egui::RichText::new(format_gpu_capabilities_unavailable_message())
                .color(DIM)
                .italics(),
        );
        return;
    };
    for (name, value) in format_gpu_settings_labels(caps) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{name}:")).color(DIM));
            ui.label(value);
        });
    }
}

fn menu_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(egui::RichText::new(label).size(16.0))
        .fill(CHIP_FILL)
        .min_size(egui::vec2(220.0, 40.0))
        .corner_radius(egui::CornerRadius::same(8));
    ui.add(btn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn era_banner_announce_sets_timer() {
        let mut banner = EraBanner::default();
        banner.announce("Bronze");
        assert_eq!(banner.current_era, "Bronze");
        assert!((banner.show_timer - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn game_ui_mode_default_is_playing() {
        assert_eq!(GameUiMode::default(), GameUiMode::Playing);
    }

    #[test]
    fn format_gpu_settings_labels_lists_backend_vram_and_flags() {
        let caps = GpuCapabilities {
            ray_tracing: true,
            mesh_shaders: false,
            dlss_available: true,
            fsr_available: false,
            metal_fx: false,
            max_vram_mb: 8192,
            backend_name: "Vulkan".to_string(),
        };
        let labels = format_gpu_settings_labels(&caps);
        assert_eq!(labels[0], ("Backend", "Vulkan".to_string()));
        assert_eq!(labels[1], ("Est. VRAM", "8192 MB".to_string()));
        assert_eq!(labels[2], ("Ray tracing", "Yes".to_string()));
        assert_eq!(labels[3], ("DLSS", "Yes".to_string()));
        assert_eq!(labels[4], ("FSR", "No".to_string()));
    }

    #[test]
    fn format_gpu_vram_label_mb_unknown_when_zero() {
        assert_eq!(format_gpu_vram_label_mb(0), "Unknown");
        assert_eq!(format_gpu_vram_label_mb(512), "512 MB");
    }

    #[test]
    fn format_gpu_capability_flag_yes_no() {
        assert_eq!(format_gpu_capability_flag(true), "Yes");
        assert_eq!(format_gpu_capability_flag(false), "No");
    }
}
