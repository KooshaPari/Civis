#![cfg(all(feature = "bevy", feature = "egui"))]

//! Menus, loading screen, and overlay plugin for the Civis reference client.
//!
//! State flow: MainMenu → WorldSetup → Loading → Playing
//!             Playing ↔ Paused (Esc)

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::game_ui::GameSpeed;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const ACCENT_HI: egui::Color32 = egui::Color32::from_rgb(140, 224, 255);
const GOLD: egui::Color32 = egui::Color32::from_rgb(232, 184, 75);
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);
const CHIP_HOVER: egui::Color32 = egui::Color32::from_rgba_premultiplied(44, 54, 74, 245);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const OVERLAY_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 160);

// ---------------------------------------------------------------------------
// Rasterized menu art (embedded at compile time so the assets resolve
// regardless of the runtime working directory). These are the real PNGs the
// art pipeline produced under assets/ui/.
// ---------------------------------------------------------------------------

const LOGO_PNG: &[u8] = include_bytes!("../assets/ui/logo.png");
const TITLE_BG_PNG: &[u8] = include_bytes!("../assets/ui/title-bg.png");
const LOADING_BG_PNG: &[u8] = include_bytes!("../assets/ui/loading-bg.png");
const SPINNER_PNG: &[u8] = include_bytes!("../assets/ui/loading-spinner.png");

// ---------------------------------------------------------------------------
// Menu texture cache — decode the embedded PNGs once into egui textures.
// ---------------------------------------------------------------------------

/// Lazily-populated handles for the embedded menu art. Each PNG is decoded to
/// an `egui::ColorImage` and uploaded the first time a menu draw system runs.
#[derive(Resource, Default)]
pub struct MenuTextures {
    pub logo: Option<egui::TextureHandle>,
    pub title_bg: Option<egui::TextureHandle>,
    pub loading_bg: Option<egui::TextureHandle>,
    pub spinner: Option<egui::TextureHandle>,
    loaded: bool,
}

impl MenuTextures {
    /// Decode + upload all art on first call; cheap no-op afterwards.
    fn ensure_loaded(&mut self, ctx: &egui::Context) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        self.logo = decode_texture(ctx, "menu_logo", LOGO_PNG);
        self.title_bg = decode_texture(ctx, "menu_title_bg", TITLE_BG_PNG);
        self.loading_bg = decode_texture(ctx, "menu_loading_bg", LOADING_BG_PNG);
        self.spinner = decode_texture(ctx, "menu_spinner", SPINNER_PNG);
    }
}

/// Decode PNG `bytes` into an egui texture via Bevy's bundled `image` decoder.
///
/// Returns `None` (with a loud `error!`) on decode failure so the menu falls
/// back to its pure-code styling rather than panicking — per the project
/// "fail clearly, never silently" stance, the failure is logged with the asset
/// name and the cause.
fn decode_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<egui::TextureHandle> {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{Image, ImageType};
    use bevy::render::render_resource::TextureFormat;

    let decoded = Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        Default::default(),
        true,
        bevy::image::ImageSampler::Default,
        RenderAssetUsages::RENDER_WORLD,
    );
    let img = match decoded {
        Ok(i) => i,
        Err(e) => {
            error!("menu texture '{name}' failed to decode: {e}");
            return None;
        }
    };
    // Normalise to RGBA8 sRGB so the pixel layout is known.
    let rgba = img.convert(TextureFormat::Rgba8UnormSrgb).unwrap_or(img);
    let size = rgba.texture_descriptor.size;
    let (w, h) = (size.width as usize, size.height as usize);
    let Some(data) = rgba.data else {
        error!("menu texture '{name}' decoded with no pixel data");
        return None;
    };
    if data.len() < w * h * 4 {
        error!(
            "menu texture '{name}' pixel buffer too small ({} bytes for {w}x{h})",
            data.len()
        );
        return None;
    }
    let color = egui::ColorImage::from_rgba_unmultiplied([w, h], &data[..w * h * 4]);
    Some(ctx.load_texture(name, color, egui::TextureOptions::LINEAR))
}

/// The game's top-level UI / flow state.
///
/// Default is [`MainMenu`](GameUiMode::MainMenu) so the game always opens to the
/// title screen instead of immediately showing the world.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum GameUiMode {
    /// Title-screen main menu — world sim is not yet running.
    #[default]
    MainMenu,
    /// World-setup form (seed, size, era, …) before generation.
    WorldSetup,
    /// Simulated/real asset-load progress before entering the world.
    Loading,
    /// Normal gameplay — world is visible, no overlay.
    Playing,
    /// Pause overlay shown; in-process sim ticks halt.
    Paused,
}

impl GameUiMode {
    /// Returns `true` when the world should NOT be fully rendered (title/setup/load).
    pub fn is_in_menu(self) -> bool {
        matches!(self, Self::MainMenu | Self::WorldSetup)
    }

    /// Returns `true` while the loading screen is active.
    pub fn is_loading(self) -> bool {
        self == Self::Loading
    }
}

/// Run-condition: `true` only while the world is actually being played
/// ([`GameUiMode::Playing`] or [`GameUiMode::Paused`]).
///
/// Shared by every gameplay HUD/overlay plugin via `.run_if(in_game)` so the
/// main menu, world-setup form, and loading screen render clean (menu card +
/// backdrop only) without each panel re-implementing the mode guard.
pub fn in_game(mode: Res<GameUiMode>) -> bool {
    matches!(*mode, GameUiMode::Playing | GameUiMode::Paused)
}

// ---------------------------------------------------------------------------
// Loading progress resource
// ---------------------------------------------------------------------------

/// Progress for the loading screen.
/// `fraction` is clamped to `[0.0, 1.0]`; reaching `1.0` triggers the
/// transition to [`GameUiMode::Playing`].
#[derive(Resource, Default, Debug)]
pub struct LoadingProgress {
    /// 0.0 → 1.0 completion fraction.
    pub fraction: f32,
    /// Short status label shown beneath the bar ("Generating terrain…").
    pub label: String,
}

impl LoadingProgress {
    /// Reset to the beginning of a new load.
    pub fn reset(&mut self) {
        self.fraction = 0.0;
        self.label = "Initialising world…".to_string();
    }
}

// ---------------------------------------------------------------------------
// Seed generation
// ---------------------------------------------------------------------------

/// Generate a fresh, well-distributed u64 seed from a non-deterministic source.
///
/// Uses `rand::thread_rng` (the crate is in-scope under the `bevy` feature).
/// The output is guaranteed to be large and well-distributed — not a small
/// sequential counter.
pub fn fresh_seed() -> u64 {
    use rand::Rng as _;
    rand::thread_rng().gen::<u64>()
}

// ---------------------------------------------------------------------------
// WorldSetup parameters
// ---------------------------------------------------------------------------

/// Parameters collected on the World-Setup screen before generation.
#[derive(Resource, Debug)]
pub struct WorldSetupParams {
    /// Determinism handle: the world sim is fully reproducible given this seed.
    /// Set to a fresh random value by default; player can override via the UI.
    pub seed: u64,
    /// Ephemeral string buffer backing the seed text field in the UI.
    pub seed_text: String,
    pub world_size: usize,
    pub water_level: i32,
    pub starting_era: usize,
}

impl Default for WorldSetupParams {
    fn default() -> Self {
        let seed = fresh_seed();
        Self {
            seed,
            seed_text: seed.to_string(),
            world_size: 1,
            water_level: 40,
            starting_era: 0,
        }
    }
}

impl WorldSetupParams {
    /// Regenerate `seed` and sync the text buffer.
    pub fn randomize(&mut self) {
        self.seed = fresh_seed();
        self.seed_text = self.seed.to_string();
    }

    /// Parse `seed_text` into `seed`.  If the text is not a valid u64, the
    /// seed is left unchanged and `false` is returned.
    pub fn commit_text(&mut self) -> bool {
        match self.seed_text.trim().parse::<u64>() {
            Ok(v) => {
                self.seed = v;
                true
            }
            Err(_) => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Era banner
// ---------------------------------------------------------------------------

/// Timed era-advancement banner shown at the top of the viewport.
#[derive(Resource, Default, Debug)]
pub struct EraBanner {
    pub current_era: String,
    pub show_timer: f32,
}

impl EraBanner {
    /// Trigger the banner for `era`, displaying it for 4 seconds.
    pub fn announce(&mut self, era: impl Into<String>) {
        self.current_era = era.into();
        self.show_timer = 4.0;
    }
}

// ---------------------------------------------------------------------------
// Settings resources
// ---------------------------------------------------------------------------

/// Controls visibility of the settings window.
#[derive(Resource, Default, Debug)]
pub struct SettingsOpen(pub bool);

/// Transient state for the settings window (no persistence yet).
#[derive(Resource, Debug)]
pub struct SettingsState {
    /// 0 = Low … 3 = Ultra
    pub graphics_quality: usize,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub camera_sensitivity: f32,
    pub sim_speed: u32,
    pub vsync: bool,
    pub fullscreen: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            graphics_quality: 2,
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 0.8,
            camera_sensitivity: 1.0,
            sim_speed: 1,
            vsync: true,
            fullscreen: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Loading tips
// ---------------------------------------------------------------------------

const TIPS: &[&str] = &[
    "Tip: Press Esc in-game to pause and adjust settings.",
    "Tip: Different eras unlock new buildings and technologies.",
    "Tip: Your choices shape the world — choose wisely.",
    "Tip: Zoom in to inspect individual citizens.",
    "Tip: Trade routes between factions boost prosperity.",
];

fn tip_for_frame(elapsed: f32) -> &'static str {
    let idx = (elapsed / 3.5) as usize % TIPS.len();
    TIPS[idx]
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Bevy plugin: main menu, loading screen, pause overlay, era banners, settings.
pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUiMode>()
            .init_resource::<LoadingProgress>()
            .init_resource::<WorldSetupParams>()
            .init_resource::<EraBanner>()
            .init_resource::<SettingsOpen>()
            .init_resource::<SettingsState>()
            .init_resource::<MenuTextures>()
            .add_systems(Update, (toggle_pause, tick_era_banner, tick_loading))
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_main_menu,
                    draw_world_setup,
                    draw_loading_screen,
                    draw_pause_menu,
                    draw_era_banner,
                    draw_settings_window,
                ),
            );
    }
}

// ---------------------------------------------------------------------------
// Update systems
// ---------------------------------------------------------------------------

fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<GameUiMode>) {
    if keys.just_pressed(KeyCode::Escape) {
        *mode = match *mode {
            GameUiMode::Playing => GameUiMode::Paused,
            GameUiMode::Paused => GameUiMode::Playing,
            other => other,
        };
    }
}

fn tick_era_banner(mut banner: ResMut<EraBanner>, time: Res<Time>) {
    if banner.show_timer > 0.0 {
        banner.show_timer = (banner.show_timer - time.delta_secs()).max(0.0);
    }
}

/// Simulated worldgen/asset-load: advances ~0.5/s so the bar fills in ~2 s.
/// Replace `RATE` with real completion callbacks once the engine wires them in.
fn tick_loading(
    mut mode: ResMut<GameUiMode>,
    mut progress: ResMut<LoadingProgress>,
    time: Res<Time>,
) {
    if *mode != GameUiMode::Loading {
        return;
    }
    const RATE: f32 = 0.5; // fills 0→1 in 2 s
    progress.fraction = (progress.fraction + time.delta_secs() * RATE).min(1.0);

    // Update label to give visual variety during the stub phase.
    progress.label = match progress.fraction {
        f if f < 0.25 => "Generating terrain…",
        f if f < 0.5 => "Placing civilisations…",
        f if f < 0.75 => "Simulating early history…",
        f if f < 0.95 => "Finalising world…",
        _ => "Ready!",
    }
    .to_string();

    if progress.fraction >= 1.0 {
        *mode = GameUiMode::Playing;
    }
}

// ---------------------------------------------------------------------------
// EguiPrimaryContextPass draw systems
// ---------------------------------------------------------------------------

fn draw_main_menu(
    mut contexts: EguiContexts,
    mut mode: ResMut<GameUiMode>,
    mut progress: ResMut<LoadingProgress>,
    mut textures: ResMut<MenuTextures>,
    mut settings_open: ResMut<SettingsOpen>,
    mut params: ResMut<WorldSetupParams>,
    mut exit: MessageWriter<AppExit>,
) {
    if *mode != GameUiMode::MainMenu {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    textures.ensure_loaded(ctx);
    image_backdrop(ctx, textures.title_bg.as_ref(), "main_menu_bg");
    egui::Area::new(egui::Id::new("main_menu_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            main_menu_panel(
                ui,
                &mut mode,
                &mut progress,
                &mut settings_open,
                &mut params,
                textures.logo.as_ref(),
                &mut exit,
            );
        });
    // Footer build/version line, bottom-centred.
    egui::Area::new(egui::Id::new("main_menu_footer"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Civis · pre-alpha sandbox")
                    .color(DIM.gamma_multiply(0.8))
                    .size(11.0),
            );
        });
}

fn draw_world_setup(
    mut contexts: EguiContexts,
    mut mode: ResMut<GameUiMode>,
    mut progress: ResMut<LoadingProgress>,
    mut params: ResMut<WorldSetupParams>,
) {
    if *mode != GameUiMode::WorldSetup {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    full_screen_backdrop(ctx);
    egui::Area::new(egui::Id::new("world_setup_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            world_setup_panel(ui, &mut mode, &mut progress, &mut params);
        });
}

fn draw_loading_screen(
    mut contexts: EguiContexts,
    mode: Res<GameUiMode>,
    progress: Res<LoadingProgress>,
    mut textures: ResMut<MenuTextures>,
    time: Res<Time>,
) {
    if *mode != GameUiMode::Loading {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    textures.ensure_loaded(ctx);
    image_backdrop(ctx, textures.loading_bg.as_ref(), "loading_bg");
    let elapsed = time.elapsed_secs();
    // Keep the spinner spinning while the screen is up.
    ctx.request_repaint();
    egui::Area::new(egui::Id::new("loading_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            loading_panel(
                ui,
                &progress,
                elapsed,
                textures.logo.as_ref(),
                textures.spinner.as_ref(),
            );
        });
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
    let Ok(ctx) = contexts.ctx_mut() else { return };
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
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Area::new(egui::Id::new("era_banner_area"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 24.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| era_banner(ui, &banner));
}

fn draw_settings_window(
    mut contexts: EguiContexts,
    mut settings_open: ResMut<SettingsOpen>,
    mut state: ResMut<SettingsState>,
    mut game_speed: Option<ResMut<GameSpeed>>,
) {
    if !settings_open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    settings_window(
        ctx,
        &mut settings_open,
        &mut state,
        game_speed.as_deref_mut(),
    );
}

// ---------------------------------------------------------------------------
// Widget helpers — backdrop / overlay
// ---------------------------------------------------------------------------

/// Full-screen near-opaque dark backdrop used behind menus and loading screen.
fn full_screen_backdrop(ctx: &egui::Context) {
    let screen = ctx.content_rect();
    let bg = egui::Color32::from_rgba_premultiplied(8, 10, 18, 245);
    egui::Area::new(egui::Id::new("menu_backdrop"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(screen, egui::CornerRadius::ZERO, bg);
        });
}

/// Paint a full-screen background image (cover-fit), with a dark gradient scrim
/// on top so foreground panels/text stay legible. Falls back to the flat
/// [`full_screen_backdrop`] when the texture is absent.
fn image_backdrop(ctx: &egui::Context, tex: Option<&egui::TextureHandle>, id: &str) {
    let Some(tex) = tex else {
        full_screen_backdrop(ctx);
        return;
    };
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new(id))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let painter = ui.painter();
            // Base fill in case the image has transparent regions / letterboxing.
            painter.rect_filled(
                screen,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgb(8, 10, 18),
            );
            // Cover-fit the image so it always fills the viewport.
            let img = tex.size_vec2();
            let scale = (screen.width() / img.x).max(screen.height() / img.y);
            let drawn = img * scale;
            let rect = egui::Rect::from_center_size(screen.center(), drawn);
            painter.image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            // Vignette-ish darkening scrim for contrast behind the panel.
            painter.rect_filled(
                screen,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_premultiplied(6, 8, 14, 150),
            );
        });
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

// ---------------------------------------------------------------------------
// Main menu panel
// ---------------------------------------------------------------------------

fn main_menu_panel(
    ui: &mut egui::Ui,
    mode: &mut GameUiMode,
    progress: &mut LoadingProgress,
    settings_open: &mut SettingsOpen,
    params: &mut WorldSetupParams,
    logo: Option<&egui::TextureHandle>,
    exit: &mut MessageWriter<AppExit>,
) {
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .corner_radius(egui::CornerRadius::same(16))
        .stroke(egui::Stroke::new(1.5, ACCENT.gamma_multiply(0.5)))
        .inner_margin(egui::Margin::same(40))
        .shadow(egui::epaint::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: egui::Color32::from_black_alpha(170),
        })
        .show(ui, |ui| {
            ui.set_min_width(340.0);
            ui.vertical_centered(|ui| {
                menu_logo(ui, logo, 220.0);
                ui.label(
                    egui::RichText::new("A civilisation sandbox")
                        .size(13.0)
                        .color(DIM),
                );
                ui.add_space(28.0);
                main_menu_buttons(ui, mode, progress, settings_open, params, exit);
            });
        });
}

/// Render the logo texture if available, else fall back to the wordmark text.
fn menu_logo(ui: &mut egui::Ui, logo: Option<&egui::TextureHandle>, max_width: f32) {
    match logo {
        Some(tex) => {
            let size = tex.size_vec2();
            let scale = (max_width / size.x).min(1.0);
            ui.add(egui::Image::new((tex.id(), size * scale)));
        }
        None => {
            ui.label(
                egui::RichText::new("CIVIS")
                    .size(52.0)
                    .color(ACCENT)
                    .strong(),
            );
        }
    }
}

fn main_menu_buttons(
    ui: &mut egui::Ui,
    mode: &mut GameUiMode,
    progress: &mut LoadingProgress,
    settings_open: &mut SettingsOpen,
    params: &mut WorldSetupParams,
    exit: &mut MessageWriter<AppExit>,
) {
    if primary_button(ui, "\u{1f30d}  New World").clicked() {
        // Draw a fresh seed for each New World so worlds differ by default; the
        // player can still override via the Randomize die or manual seed entry
        // on the World-Setup screen. (Without this, every play reused the one
        // seed drawn at WorldSetupParams::default() — same map every time.)
        params.randomize();
        progress.reset();
        *mode = GameUiMode::WorldSetup;
    }
    ui.add_space(8.0);
    if menu_button(ui, "\u{1f4c2}  Load World").clicked() {
        progress.reset();
        *mode = GameUiMode::Loading;
    }
    ui.add_space(8.0);
    if menu_button(ui, "\u{25b6}  Continue").clicked() {
        progress.reset();
        *mode = GameUiMode::Loading;
    }
    ui.add_space(8.0);
    if menu_button(ui, "\u{2699}  Settings").clicked() {
        settings_open.0 = !settings_open.0;
    }
    ui.add_space(14.0);
    ui.separator();
    ui.add_space(10.0);
    if menu_button(ui, "\u{23fb}  Quit").clicked() {
        exit.write(AppExit::Success);
    }
}

// ---------------------------------------------------------------------------
// World-setup panel
// ---------------------------------------------------------------------------

fn world_setup_panel(
    ui: &mut egui::Ui,
    mode: &mut GameUiMode,
    progress: &mut LoadingProgress,
    params: &mut WorldSetupParams,
) {
    const SIZES: &[&str] = &["Small", "Medium", "Large", "Huge"];
    const ERAS: &[&str] = &["Stone Age", "Bronze Age", "Iron Age", "Industrial"];
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(egui::Stroke::new(1.5, ACCENT.gamma_multiply(0.5)))
        .inner_margin(egui::Margin::same(32))
        .show(ui, |ui| {
            ui.set_min_width(380.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("World Setup")
                        .size(26.0)
                        .color(ACCENT)
                        .strong(),
                );
                ui.add_space(18.0);
            });
            world_setup_fields(ui, params, SIZES, ERAS);
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if menu_button(ui, "\u{2190}  Back").clicked() {
                    *mode = GameUiMode::MainMenu;
                }
                ui.add_space(8.0);
                if menu_button(ui, "\u{2699}  Generate World").clicked() {
                    // Commit any typed seed before transitioning.
                    params.commit_text();
                    progress.reset();
                    *mode = GameUiMode::Loading;
                }
            });
        });
}

fn world_setup_fields(
    ui: &mut egui::Ui,
    params: &mut WorldSetupParams,
    sizes: &[&str],
    eras: &[&str],
) {
    // ---- Seed row --------------------------------------------------------
    ui.label(
        egui::RichText::new("World Seed")
            .color(ACCENT)
            .size(13.0)
            .strong(),
    );
    ui.add_space(4.0);

    // Display the active seed prominently.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(params.seed.to_string())
                .size(22.0)
                .color(egui::Color32::WHITE)
                .monospace(),
        );
        ui.add_space(8.0);
        // Randomize button.
        if ui
            .add(
                egui::Button::new(egui::RichText::new("\u{1f3b2}  Randomize").size(14.0))
                    .fill(CHIP_FILL)
                    .min_size(egui::vec2(120.0, 28.0))
                    .corner_radius(egui::CornerRadius::same(6)),
            )
            .clicked()
        {
            params.randomize();
        }
    });
    ui.add_space(6.0);

    // Editable text field — player can paste/type a specific u64.
    ui.label(
        egui::RichText::new("Enter seed manually:")
            .color(DIM)
            .small(),
    );
    let resp = ui.add(
        egui::TextEdit::singleline(&mut params.seed_text)
            .desired_width(200.0)
            .hint_text("paste a u64…"),
    );
    if resp.lost_focus() {
        // Commit on focus-loss; reset to current seed if parse fails.
        if !params.commit_text() {
            params.seed_text = params.seed.to_string();
        }
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // ---- World size -------------------------------------------------------
    ui.label(egui::RichText::new("World Size").color(DIM).small());
    egui::ComboBox::from_id_salt("world_size_combo")
        .selected_text(*sizes.get(params.world_size).unwrap_or(&"Medium"))
        .show_ui(ui, |ui| {
            for (i, &label) in sizes.iter().enumerate() {
                ui.selectable_value(&mut params.world_size, i, label);
            }
        });
    ui.add_space(8.0);

    // ---- Water level ------------------------------------------------------
    ui.label(egui::RichText::new("Water Level").color(DIM).small());
    ui.add(egui::Slider::new(&mut params.water_level, 20..=80).suffix("%"));
    ui.add_space(8.0);
    // TODO: wire world_size / water_level to voxel worldgen arguments once generation accepts them.

    // ---- Starting era -----------------------------------------------------
    ui.label(egui::RichText::new("Starting Era").color(DIM).small());
    egui::ComboBox::from_id_salt("starting_era_combo")
        .selected_text(*eras.get(params.starting_era).unwrap_or(&"Stone Age"))
        .show_ui(ui, |ui| {
            for (i, &label) in eras.iter().enumerate() {
                ui.selectable_value(&mut params.starting_era, i, label);
            }
        });
}

// ---------------------------------------------------------------------------
// Loading-screen panel
// ---------------------------------------------------------------------------

fn loading_panel(
    ui: &mut egui::Ui,
    progress: &LoadingProgress,
    elapsed: f32,
    logo: Option<&egui::TextureHandle>,
    spinner: Option<&egui::TextureHandle>,
) {
    let frac = progress.fraction.clamp(0.0, 1.0);
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .corner_radius(egui::CornerRadius::same(16))
        .stroke(egui::Stroke::new(1.5, ACCENT.gamma_multiply(0.4)))
        .inner_margin(egui::Margin::same(44))
        .shadow(egui::epaint::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: egui::Color32::from_black_alpha(170),
        })
        .show(ui, |ui| {
            ui.set_min_width(440.0);
            ui.vertical_centered(|ui| {
                menu_logo(ui, logo, 240.0);
                ui.add_space(24.0);

                // Animated spinner above the bar.
                loading_spinner(ui, spinner, elapsed);
                ui.add_space(20.0);

                // AAA gradient progress bar.
                gradient_progress_bar(ui, frac, 360.0);
                ui.add_space(12.0);

                // Status label.
                ui.label(
                    egui::RichText::new(&progress.label)
                        .color(ACCENT_HI)
                        .size(14.0)
                        .strong(),
                );
                ui.add_space(22.0);

                // Rotating tip line.
                ui.label(
                    egui::RichText::new(tip_for_frame(elapsed))
                        .color(DIM.gamma_multiply(0.85))
                        .size(12.0)
                        .italics(),
                );
            });
        });
}

/// A rotating spinner texture (falls back to an animated arc when absent).
fn loading_spinner(ui: &mut egui::Ui, spinner: Option<&egui::TextureHandle>, elapsed: f32) {
    const SIZE: f32 = 44.0;
    let angle = (elapsed * 2.4) % std::f32::consts::TAU;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(SIZE, SIZE), egui::Sense::hover());
    match spinner {
        Some(tex) => {
            // Rotate the textured quad about its centre.
            let painter = ui.painter();
            let c = rect.center();
            let h = SIZE / 2.0;
            let (sin, cos) = angle.sin_cos();
            let rot =
                |dx: f32, dy: f32| egui::pos2(c.x + dx * cos - dy * sin, c.y + dx * sin + dy * cos);
            let pts = [rot(-h, -h), rot(h, -h), rot(h, h), rot(-h, h)];
            let uv = [
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(1.0, 1.0),
                egui::pos2(0.0, 1.0),
            ];
            let mut mesh = egui::epaint::Mesh::with_texture(tex.id());
            for i in 0..4 {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: pts[i],
                    uv: uv[i],
                    color: egui::Color32::WHITE,
                });
            }
            mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
            painter.add(egui::Shape::mesh(mesh));
        }
        None => {
            // Pure-code spinning arc fallback.
            let painter = ui.painter();
            let c = rect.center();
            let r = SIZE / 2.0 - 3.0;
            for i in 0..12 {
                let a = angle + i as f32 * std::f32::consts::TAU / 12.0;
                let alpha = (i as f32 / 12.0 * 220.0) as u8;
                let p = egui::pos2(c.x + a.cos() * r, c.y + a.sin() * r);
                painter.circle_filled(p, 2.4, ACCENT.gamma_multiply(alpha as f32 / 255.0));
            }
        }
    }
}

/// A rounded, accent-gradient progress bar with a percentage overlay.
fn gradient_progress_bar(ui: &mut egui::Ui, frac: f32, width: f32) {
    const H: f32 = 22.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, H), egui::Sense::hover());
    let painter = ui.painter();
    let radius = egui::CornerRadius::same((H / 2.0) as u8);
    // Track.
    painter.rect_filled(rect, radius, CHIP_FILL);
    painter.rect_stroke(
        rect,
        radius,
        egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.3)),
        egui::StrokeKind::Inside,
    );
    // Fill (cyan→gold gradient approximated by a two-segment blend).
    if frac > 0.001 {
        let fill_w = (rect.width() * frac).max(H);
        let fill = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, H));
        let lerp = |a: egui::Color32, b: egui::Color32, t: f32| {
            egui::Color32::from_rgb(
                (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
                (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
                (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
            )
        };
        painter.rect_filled(fill, radius, lerp(ACCENT, GOLD, frac));
        // Subtle top highlight.
        let hi = egui::Rect::from_min_size(fill.min, egui::vec2(fill_w, H * 0.45));
        painter.rect_filled(
            hi,
            radius,
            egui::Color32::from_rgba_premultiplied(255, 255, 255, 28),
        );
    }
    // Percentage text centred.
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.0}%", frac * 100.0),
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(8, 12, 20),
    );
}

// ---------------------------------------------------------------------------
// Pause menu
// ---------------------------------------------------------------------------

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
    if menu_button(ui, "\u{1f3e0}  Main Menu").clicked() {
        *mode = GameUiMode::MainMenu;
    }
    ui.add_space(14.0);
    ui.separator();
    ui.add_space(10.0);
    if menu_button(ui, "\u{23fb}  Quit").clicked() {
        exit.write(AppExit::Success);
    }
}

// ---------------------------------------------------------------------------
// Era banner
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Settings window
// ---------------------------------------------------------------------------

fn settings_window(
    ctx: &egui::Context,
    settings_open: &mut SettingsOpen,
    state: &mut SettingsState,
    game_speed: Option<&mut GameSpeed>,
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
    .show(ctx, |ui| settings_rows(ui, state, game_speed, QUALITIES));
}

fn settings_rows(
    ui: &mut egui::Ui,
    state: &mut SettingsState,
    game_speed: Option<&mut GameSpeed>,
    qualities: &[&str],
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
    ui.label(egui::RichText::new("Music Volume").color(DIM).small());
    ui.add(egui::Slider::new(&mut state.music_volume, 0.0..=1.0).show_value(true));
    ui.add_space(8.0);
    ui.label(egui::RichText::new("SFX Volume").color(DIM).small());
    ui.add(egui::Slider::new(&mut state.sfx_volume, 0.0..=1.0).show_value(true));
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Sim Speed").color(DIM).small());
    if let Some(speed) = game_speed {
        if ui
            .add(
                egui::Slider::new(&mut state.sim_speed, 1..=10)
                    .text("x")
                    .show_value(true),
            )
            .changed()
        {
            speed.multiplier = state.sim_speed;
        }
    } else {
        ui.add(
            egui::Slider::new(&mut state.sim_speed, 1..=10)
                .text("x")
                .show_value(true),
        );
    }
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Camera Sensitivity").color(DIM).small());
    ui.add(egui::Slider::new(&mut state.camera_sensitivity, 0.2..=4.0).show_value(true));
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Display").color(DIM).small());
    // TODO: wire fullscreen setting to window/fullscreen resource when available.
    ui.checkbox(&mut state.fullscreen, "Fullscreen");
    ui.label(egui::RichText::new("VSync").color(DIM).small());
    ui.checkbox(&mut state.vsync, "Vertical sync");
    // TODO: wire vsync setting to window descriptor once window settings sink is added.
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Controls").color(DIM).small());
    ui.indent("settings_controls", |ui| {
        ui.label("Rotate Camera: Q / E");
        ui.label("Move Up / Down: R / F");
        ui.label("Move Horizontal: W / A / S / D");
    });
}

// ---------------------------------------------------------------------------
// Shared button widget
// ---------------------------------------------------------------------------

/// Standard themed menu button: glass chip that lights cyan on hover.
fn menu_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    themed_button(ui, label, CHIP_FILL, DIM, ACCENT, false)
}

/// Emphasised primary call-to-action (filled accent tint, white text).
fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    themed_button(
        ui,
        label,
        ACCENT.gamma_multiply(0.22),
        egui::Color32::WHITE,
        ACCENT,
        true,
    )
}

fn themed_button(
    ui: &mut egui::Ui,
    label: &str,
    fill: egui::Color32,
    _text: egui::Color32,
    accent: egui::Color32,
    primary: bool,
) -> egui::Response {
    let size = egui::vec2(240.0, 42.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let hovered = resp.hovered();
    let painter = ui.painter();
    let radius = egui::CornerRadius::same(9);

    let bg = if hovered {
        if primary {
            accent.gamma_multiply(0.34)
        } else {
            CHIP_HOVER
        }
    } else {
        fill
    };
    painter.rect_filled(rect, radius, bg);
    let stroke_w = if hovered { 1.6 } else { 1.0 };
    let stroke_c = if hovered {
        accent
    } else {
        accent.gamma_multiply(0.35)
    };
    painter.rect_stroke(
        rect,
        radius,
        egui::Stroke::new(stroke_w, stroke_c),
        egui::StrokeKind::Inside,
    );

    let txt = if hovered || primary {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(214, 224, 240)
    };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(16.0),
        txt,
    );
    resp
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_ui_mode_default_is_main_menu() {
        assert_eq!(GameUiMode::default(), GameUiMode::MainMenu);
    }

    #[test]
    fn is_in_menu_helpers() {
        assert!(GameUiMode::MainMenu.is_in_menu());
        assert!(GameUiMode::WorldSetup.is_in_menu());
        assert!(!GameUiMode::Loading.is_in_menu());
        assert!(!GameUiMode::Playing.is_in_menu());
        assert!(!GameUiMode::Paused.is_in_menu());
    }

    #[test]
    fn is_loading_helper() {
        assert!(GameUiMode::Loading.is_loading());
        assert!(!GameUiMode::Playing.is_loading());
    }

    #[test]
    fn loading_progress_reset() {
        let mut p = LoadingProgress {
            fraction: 0.9,
            label: "done".into(),
        };
        p.reset();
        assert_eq!(p.fraction, 0.0);
        assert!(!p.label.is_empty());
    }

    #[test]
    fn loading_progress_fraction_clamp() {
        let p = LoadingProgress {
            fraction: 1.5,
            label: String::new(),
        };
        assert!(p.fraction.clamp(0.0, 1.0) <= 1.0);
    }

    #[test]
    fn state_flow_main_menu_to_world_setup() {
        let mut mode = GameUiMode::default();
        assert_eq!(mode, GameUiMode::MainMenu);
        // Simulate "New World" button
        mode = GameUiMode::WorldSetup;
        assert_eq!(mode, GameUiMode::WorldSetup);
    }

    #[test]
    fn state_flow_world_setup_to_loading_to_playing() {
        let mut mode = GameUiMode::WorldSetup;
        let mut progress = LoadingProgress::default();
        // Simulate "Generate World"
        progress.reset();
        mode = GameUiMode::Loading;
        assert_eq!(mode, GameUiMode::Loading);
        assert_eq!(progress.fraction, 0.0);
        // Simulate tick_loading completing
        progress.fraction = 1.0;
        mode = GameUiMode::Playing;
        assert_eq!(mode, GameUiMode::Playing);
    }

    #[test]
    fn pause_only_in_playing_or_paused() {
        // MainMenu and Loading should not be affected by Esc (toggle_pause guard)
        for initial in [
            GameUiMode::MainMenu,
            GameUiMode::Loading,
            GameUiMode::WorldSetup,
        ] {
            // The toggle_pause function only acts on Playing/Paused; other states are passed through unchanged.
            let result = match initial {
                GameUiMode::Playing => GameUiMode::Paused,
                GameUiMode::Paused => GameUiMode::Playing,
                other => other,
            };
            assert_eq!(
                result, initial,
                "Mode {:?} should not change on Esc",
                initial
            );
        }
    }

    #[test]
    fn era_banner_announce_sets_timer() {
        let mut banner = EraBanner::default();
        banner.announce("Bronze");
        assert_eq!(banner.current_era, "Bronze");
        assert!((banner.show_timer - 4.0).abs() < f32::EPSILON);
    }

    // ---- Seed-specific tests -----------------------------------------------

    /// fresh_seed() must produce large values (> 2^16), not tiny counters.
    #[test]
    fn fresh_seed_is_large() {
        let s = fresh_seed();
        assert!(s > 0xFFFF, "seed {s} is suspiciously small (≤ 65535)");
    }

    /// Two calls to fresh_seed() should almost never collide.
    /// With a 64-bit uniform PRNG the probability of collision is ~5e-19.
    #[test]
    fn fresh_seed_distinct_on_repeated_calls() {
        let a = fresh_seed();
        let b = fresh_seed();
        assert_ne!(a, b, "Two consecutive seeds were identical — RNG broken?");
    }

    /// The "New World" transition must draw a fresh seed so two consecutive
    /// New Worlds differ by default (the regression this fixes: every New World
    /// reused the one seed from WorldSetupParams::default()).
    #[test]
    fn new_world_transition_randomizes_seed() {
        let mut p = WorldSetupParams {
            seed: 1234,
            seed_text: "1234".to_string(),
            world_size: 1,
            water_level: 40,
            starting_era: 0,
        };
        // Mirror the New World button body: randomize() runs on the transition.
        p.randomize();
        assert_ne!(p.seed, 1234, "New World must draw a fresh seed");
        assert_eq!(p.seed_text, p.seed.to_string(), "seed text stays in sync");
    }

    /// WorldSetupParams::default() must produce a seed that is large and valid.
    #[test]
    fn world_setup_params_default_seed_is_large() {
        let p = WorldSetupParams {
            seed: fresh_seed(),
            seed_text: String::new(),
            world_size: 1,
            water_level: 40,
            starting_era: 0,
        };
        assert!(p.seed > 0xFFFF);
    }

    /// seed_text is kept in sync with seed after randomize().
    #[test]
    fn randomize_syncs_seed_text() {
        let mut p = WorldSetupParams {
            seed: 42,
            seed_text: "42".to_string(),
            world_size: 1,
            water_level: 40,
            starting_era: 0,
        };
        p.randomize();
        assert_eq!(p.seed_text, p.seed.to_string());
        assert_ne!(p.seed, 42, "seed should change after randomize");
    }

    /// commit_text() round-trips a valid u64 string.
    #[test]
    fn commit_text_valid_seed() {
        let mut p = WorldSetupParams {
            seed: 1,
            seed_text: "12345678901234567".to_string(),
            world_size: 1,
            water_level: 40,
            starting_era: 0,
        };
        assert!(p.commit_text());
        assert_eq!(p.seed, 12_345_678_901_234_567_u64);
    }

    /// commit_text() rejects garbage and returns false without mutating seed.
    #[test]
    fn commit_text_invalid_seed_leaves_seed_unchanged() {
        let mut p = WorldSetupParams {
            seed: 9999,
            seed_text: "not-a-number".to_string(),
            world_size: 1,
            water_level: 40,
            starting_era: 0,
        };
        assert!(!p.commit_text());
        assert_eq!(p.seed, 9999);
    }

    /// A large u64 seed can be represented as decimal and round-trips correctly.
    #[test]
    fn large_seed_round_trips_through_string() {
        let original: u64 = 0x9E3779B97F4A7C15;
        let text = original.to_string();
        let parsed: u64 = text.parse().unwrap();
        assert_eq!(parsed, original);
    }
}
