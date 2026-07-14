#![cfg(all(feature = "bevy", feature = "egui"))]

//! Menus and overlay plugin for the Civis reference client (FR-CIV-BEVY-024 / item 49).
//! Settings GPU readout: FR-CIV-BEVY-036 / item 61.

use crate::gpu_features::GpuCapabilities;
use crate::live_attach::LiveAttachBridge;
use crate::live_stream::LiveStreamScene;
use crate::outcome_overlay::{
    begin_player_session, end_player_session, outcome_modal_visible, OutcomeEscapeBlock,
    OutcomeOverlayState, OutcomeSessionGate,
};
use crate::save_load_ui::SaveLoadPanel;
use crate::settings_ui::{GameSettings, KeyBinding, ACTION_PAUSE_SIM};
use crate::ui_theme::{GLASS_FILL, KC_ACCENT, CHIP_FILL};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

const WORLDGEN_PRESETS: [&str; 4] = [
    "single-race-ardani",
    "three-race-balanced",
    "ardani-dominant",
    "lush-frontier",
];
const WORLDGEN_DEFAULT_SEED: u64 = 0xC1F1_5EED_D3AD_BEEF;
const WORLDGEN_BOOT_SECONDS: f32 = 2.0;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const OVERLAY_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 160);

/// Shell state used by the Bevy window client (main menu + gameplay + pause states).
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    MainMenu,
    /// Map / scenario setup before worldgen boots.
    WorldSetup,
    WorldGen,
    Playing,
    Paused,
}

impl Default for AppState {
    fn default() -> Self {
        Self::MainMenu
    }
}

/// One-shot intent emitted by menu buttons and consumed by `bevy_window`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainMenuCommand {
    None,
    /// Open the world / map setup panel (does not boot yet).
    NewWorld,
    /// Confirm world setup and start generation / live attach.
    ConfirmWorldSetup,
    /// Abort world setup back to the title screen.
    CancelWorldSetup,
    Continue,
    LoadGame,
    Resume,
    OpenSettings,
    OpenSavePanel,
    ExitToMainMenu,
    Quit,
}

impl Default for MainMenuCommand {
    fn default() -> Self {
        Self::None
    }
}

/// Resource that carries the latest main-menu shell command.
#[derive(Resource, Default, Debug)]
pub struct MenuCommand {
    pub action: MainMenuCommand,
}

/// Continuation availability discovered from server save metadata.
#[derive(Resource, Default, Debug)]
pub struct MainMenuSaves {
    pub can_continue: bool,
    pub preferred_slot: Option<String>,
}

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

/// Per-world setup parameters shared by voxel generation and the map view.
#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldSetupParams {
    /// World seed selected for the current run.
    pub seed: u64,
    /// World-size preset index mirrored by the settings UI.
    pub world_size: usize,
    /// Climate / scenario preset index into [`WORLDGEN_PRESETS`].
    pub climate_preset: usize,
    /// Whether continents favour islands vs contiguous land (UI knob).
    pub archipelago: bool,
    /// Starting population density 0..=1.
    pub population_density: f32,
}

impl Default for WorldSetupParams {
    fn default() -> Self {
        Self {
            seed: 0xC1F1_5EED_D3AD_BEEF,
            world_size: 1,
            climate_preset: 0,
            archipelago: false,
            population_density: 0.45,
        }
    }
}

/// Human labels for world-size combo (UI).
const WORLD_SIZE_LABELS: [&str; 4] = ["Tiny", "Standard", "Large", "Huge"];

/// Transient world-gen boot timer (standalone server attach).
#[derive(Resource, Default, Debug)]
pub struct WorldGenBoot {
    /// Elapsed seconds while in [`AppState::WorldGen`].
    pub elapsed: f32,
}

/// Optional rasterised main-menu title assets (PNG only; SVG sources are in PIPELINE.md).
#[derive(Resource, Default)]
pub struct MainMenuTitleAssets {
    /// Full-menu background (`ui/title-bg.png`).
    pub background: Option<Handle<Image>>,
    /// Logo mark (`ui/logo.png`).
    pub logo: Option<Handle<Image>>,
    /// Wordmark (`ui/wordmark.png`).
    pub wordmark: Option<Handle<Image>>,
}

/// Transient state for the settings window (no persistence yet).
#[derive(Resource, Debug)]
pub struct SettingsState {
    /// 0 = Low, 1 = Medium, 2 = High, 3 = Ultra
    pub graphics_quality: usize,
    /// 0.0 – 1.0
    pub master_volume: f32,
    /// Tick speed multiplier.
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
            .init_resource::<WorldSetupParams>()
            .init_resource::<SettingsState>()
            .init_resource::<MenuCommand>()
            .init_resource::<MainMenuSaves>()
            .init_resource::<WorldGenBoot>()
            // Idempotent with OutcomeOverlayPlugin (live attach).
            .init_resource::<OutcomeSessionGate>()
            .init_resource::<OutcomeOverlayState>()
            .init_resource::<MainMenuTitleAssets>()
            .add_systems(Startup, load_main_menu_title_assets)
            .add_systems(Update, (toggle_pause, tick_era_banner))
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_main_menu,
                    draw_world_setup,
                    draw_worldgen_overlay,
                    draw_pause_menu,
                    draw_era_banner,
                ),
            );
    }
}

/// Sync [`GameUiMode`] pause overlay with [`AppState`] when both are present.
pub fn sync_app_state_with_game_mode(
    state: Option<Res<State<AppState>>>,
    mode: Res<GameUiMode>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(state) = state else {
        return;
    };
    match (*mode, state.get()) {
        (GameUiMode::Paused, AppState::Playing) => next_state.set(AppState::Paused),
        (GameUiMode::Playing, AppState::Paused) => next_state.set(AppState::Playing),
        _ => {}
    }
}

/// Advance from world generation to gameplay after boot timer or live scene readiness.
pub fn advance_worldgen_to_playing(
    time: Res<Time>,
    mut boot: ResMut<WorldGenBoot>,
    state: Option<Res<State<AppState>>>,
    scene: Option<Res<LiveStreamScene>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(state) = state else {
        return;
    };
    if *state.get() != AppState::WorldGen {
        boot.elapsed = 0.0;
        return;
    }

    boot.elapsed += time.delta_secs();

    let ready = match scene.as_deref() {
        Some(scene) => live_stream_has_content(scene) || boot.elapsed >= WORLDGEN_BOOT_SECONDS,
        None => true,
    };

    if ready {
        next_state.set(AppState::Playing);
        boot.elapsed = 0.0;
    }
}

/// Consume one-shot [`MenuCommand`] actions from shell buttons.
pub fn consume_menu_commands(
    mut menu_command: ResMut<MenuCommand>,
    state: Option<Res<State<AppState>>>,
    mut next_state: ResMut<NextState<AppState>>,
    bridge: Option<Res<LiveAttachBridge>>,
    mut save_panel: Option<ResMut<SaveLoadPanel>>,
    saves: Res<MainMenuSaves>,
    params: Res<WorldSetupParams>,
    mut game_mode: ResMut<GameUiMode>,
    mut exit: MessageWriter<AppExit>,
    gate: Option<ResMut<OutcomeSessionGate>>,
    overlay: Option<ResMut<OutcomeOverlayState>>,
    mut boot: ResMut<WorldGenBoot>,
    mut game_settings: Option<ResMut<GameSettings>>,
) {
    let Some(state) = state else {
        return;
    };
    if menu_command.action == MainMenuCommand::None {
        return;
    }

    let action = menu_command.action;
    menu_command.action = MainMenuCommand::None;
    match action {
        MainMenuCommand::None => {}
        MainMenuCommand::NewWorld => {
            next_state.set(AppState::WorldSetup);
        }
        MainMenuCommand::ConfirmWorldSetup => {
            if let Some(bridge) = bridge.as_ref() {
                let preset = WORLDGEN_PRESETS
                    .get(params.climate_preset % WORLDGEN_PRESETS.len())
                    .copied()
                    .unwrap_or(WORLDGEN_PRESETS[0]);
                start_world_boot(&bridge.client, preset, params.seed);
            }
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                begin_player_session(&mut gate, &mut overlay);
            }
            boot.elapsed = 0.0;
            next_state.set(AppState::WorldGen);
        }
        MainMenuCommand::CancelWorldSetup => {
            next_state.set(AppState::MainMenu);
            boot.elapsed = 0.0;
        }
        MainMenuCommand::Continue => {
            if let Some(bridge) = bridge.as_ref() {
                let slot_name = saves
                    .preferred_slot
                    .as_deref()
                    .unwrap_or("slot-1")
                    .to_string();
                let slot_id = slot_name
                    .strip_prefix("slot-")
                    .and_then(|raw| raw.parse::<u32>().ok())
                    .map(|slot| 2010 + slot)
                    .unwrap_or(2010);
                let json = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": slot_id,
                    "method": "save.load",
                    "params": { "slot_name": slot_name },
                })
                .to_string();
                bridge.client.send_rpc_raw(json);
            }
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                begin_player_session(&mut gate, &mut overlay);
            }
            boot.elapsed = 0.0;
            next_state.set(AppState::WorldGen);
        }
        MainMenuCommand::LoadGame => {
            if let Some(save_panel) = save_panel.as_mut() {
                save_panel.visible = true;
            }
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                begin_player_session(&mut gate, &mut overlay);
            }
            boot.elapsed = 0.0;
            next_state.set(AppState::WorldGen);
        }
        MainMenuCommand::Resume => {
            if *state.get() == AppState::Paused {
                next_state.set(AppState::Playing);
            }
        }
        MainMenuCommand::OpenSettings => {
            if let Some(mut settings) = game_settings {
                settings.open = true;
                settings.active_tab = crate::settings_ui::SettingsTab::Graphics;
            }
        }
        MainMenuCommand::OpenSavePanel => {
            if let Some(save_panel) = save_panel.as_mut() {
                save_panel.visible = true;
            }
        }
        MainMenuCommand::ExitToMainMenu => {
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                end_player_session(&mut gate, &mut overlay);
            }
            next_state.set(AppState::MainMenu);
            *game_mode = GameUiMode::Playing;
            boot.elapsed = 0.0;
        }
        MainMenuCommand::Quit => {
            *game_mode = GameUiMode::Playing;
            exit.write(AppExit::Success);
        }
    }
}

pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings: Option<Res<GameSettings>>,
    app_state: Option<Res<State<AppState>>>,
    outcome_overlay: Option<Res<OutcomeOverlayState>>,
    escape_block: Option<Res<OutcomeEscapeBlock>>,
    mut mode: ResMut<GameUiMode>,
) {
    let pause_binding = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_PAUSE_SIM))
        .unwrap_or(KeyBinding::Key(KeyCode::Escape));
    if !pause_binding.is_just_pressed(&keys, &mouse_buttons) {
        return;
    }

    if escape_block.map(|block| block.0).unwrap_or(false) {
        return;
    }
    if let Some(overlay) = outcome_overlay.as_ref() {
        if outcome_modal_visible(overlay) {
            return;
        }
    }

    if let Some(app_state) = app_state {
        if *app_state.get() != AppState::Playing && *app_state.get() != AppState::Paused {
            return;
        }
    }

    *mode = match *mode {
        GameUiMode::Playing => GameUiMode::Paused,
        GameUiMode::Paused => GameUiMode::Playing,
    };
}

fn tick_era_banner(mut banner: ResMut<EraBanner>, time: Res<Time>) {
    if banner.show_timer > 0.0 {
        banner.show_timer = (banner.show_timer - time.delta_secs()).max(0.0);
    }
}

/// True while the player is in live gameplay rather than the paused overlay.
#[must_use]
pub fn in_game(mode: Res<GameUiMode>) -> bool {
    *mode == GameUiMode::Playing
}

fn draw_main_menu(
    mut contexts: EguiContexts,
    state: Option<Res<State<AppState>>>,
    mut command: ResMut<MenuCommand>,
    saves: Res<MainMenuSaves>,
    titles: Res<MainMenuTitleAssets>,
    images: Res<Assets<Image>>,
) {
    let Some(state) = state else {
        return;
    };
    if *state.get() != AppState::MainMenu {
        return;
    }

    let bg_tex = titles.background.as_ref().and_then(|handle| {
        images
            .get(handle)
            .map(|_| contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone())))
    });
    let title_tex = titles
        .wordmark
        .as_ref()
        .or(titles.logo.as_ref())
        .and_then(|handle| {
            images
                .get(handle)
                .map(|_| contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone())))
        });
    let title_is_wordmark = titles.wordmark.is_some();

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if let Some(id) = bg_tex {
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("main_menu_bg"))
            .fixed_pos(screen.min)
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                ui.image((id, screen.size()));
            });
    }

    egui::Area::new(egui::Id::new("main_menu_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(GLASS_FILL)
                .inner_margin(egui::Margin::same(28))
                .show(ui, |ui| {
                    ui.set_min_width(420.0);
                    ui.vertical_centered(|ui| {
                        let mut drew_title = false;
                        if let Some(id) = title_tex {
                            let size = if title_is_wordmark {
                                egui::vec2(360.0, 90.0)
                            } else {
                                egui::vec2(320.0, 120.0)
                            };
                            ui.image((id, size));
                            drew_title = true;
                        }
                        if !drew_title {
                            ui.label(
                                egui::RichText::new("Civis")
                                    .size(52.0)
                                    .color(KC_ACCENT)
                                    .strong(),
                            );
                        }
                        ui.label(
                            egui::RichText::new("Main menu")
                                .size(16.0)
                                .color(DIM)
                                .italics(),
                        );
                        ui.add_space(16.0);

                        if menu_button(ui, "\u{25b6}  New World").clicked() {
                            command.action = MainMenuCommand::NewWorld;
                        }
                        ui.add_space(8.0);

                        let continue_label = if saves.can_continue {
                            "\u{1f3c3}  Continue"
                        } else {
                            "\u{1f3c3}  Continue (no save)"
                        };
                        let continue_btn = ui.add_enabled(
                            saves.can_continue,
                            egui::Button::new(egui::RichText::new(continue_label).size(16.0))
                                .fill(KC_ACCENT.gamma_multiply(0.15))
                                .min_size(egui::vec2(220.0, 40.0))
                                .corner_radius(egui::CornerRadius::same(8)),
                        );
                        if continue_btn.clicked() {
                            command.action = MainMenuCommand::Continue;
                        }
                        ui.add_space(8.0);

                        if menu_button(ui, "\u{1f4be}  Load Game").clicked() {
                            command.action = MainMenuCommand::LoadGame;
                        }
                        ui.add_space(8.0);

                        if menu_button(ui, "\u{2699}  Settings").clicked() {
                            command.action = MainMenuCommand::OpenSettings;
                        }
                        ui.add_space(8.0);
                        if menu_button(ui, "\u{23fb}  Quit").clicked() {
                            command.action = MainMenuCommand::Quit;
                        }
                    });
                });
        });
}

fn draw_world_setup(
    mut contexts: EguiContexts,
    state: Option<Res<State<AppState>>>,
    mut params: ResMut<WorldSetupParams>,
    mut command: ResMut<MenuCommand>,
) {
    let Some(state) = state else {
        return;
    };
    if *state.get() != AppState::WorldSetup {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Area::new(egui::Id::new("world_setup_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(GLASS_FILL)
                .inner_margin(egui::Margin::same(28))
                .corner_radius(egui::CornerRadius::same(12))
                .stroke(egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.45)))
                .show(ui, |ui| {
                    ui.set_min_width(480.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("New World")
                                .size(28.0)
                                .color(KC_ACCENT)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("Map generation & scenario")
                                .size(14.0)
                                .color(DIM)
                                .italics(),
                        );
                        ui.add_space(16.0);

                        ui.label(egui::RichText::new("World seed").color(DIM).small());
                        let mut seed_text = format!("{:016X}", params.seed);
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut seed_text)
                                    .desired_width(220.0)
                                    .hint_text("hex seed"),
                            )
                            .changed()
                        {
                            if let Ok(parsed) = u64::from_str_radix(seed_text.trim(), 16) {
                                params.seed = parsed;
                            }
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Randomize").clicked() {
                                params.seed = ((std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_nanos())
                                    .unwrap_or(0)
                                    ^ 0xC1F1_5EED_u128)
                                    as u64)
                                    .wrapping_mul(0x9E37_79B9_7F4A_7C15);
                            }
                        });
                        ui.add_space(10.0);

                        ui.label(egui::RichText::new("Map size").color(DIM).small());
                        egui::ComboBox::from_id_salt("world_size_combo")
                            .selected_text(
                                *WORLD_SIZE_LABELS
                                    .get(params.world_size % WORLD_SIZE_LABELS.len())
                                    .unwrap_or(&"Standard"),
                            )
                            .show_ui(ui, |ui| {
                                for (i, label) in WORLD_SIZE_LABELS.iter().enumerate() {
                                    ui.selectable_value(&mut params.world_size, i, *label);
                                }
                            });
                        ui.add_space(10.0);

                        ui.label(egui::RichText::new("Climate / scenario").color(DIM).small());
                        let climate_label = WORLDGEN_PRESETS
                            .get(params.climate_preset % WORLDGEN_PRESETS.len())
                            .copied()
                            .unwrap_or(WORLDGEN_PRESETS[0]);
                        egui::ComboBox::from_id_salt("climate_preset_combo")
                            .selected_text(climate_label)
                            .width(280.0)
                            .show_ui(ui, |ui| {
                                for (i, label) in WORLDGEN_PRESETS.iter().enumerate() {
                                    ui.selectable_value(&mut params.climate_preset, i, *label);
                                }
                            });
                        ui.add_space(10.0);

                        ui.checkbox(&mut params.archipelago, "Archipelago landmass bias");
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("Starting population density").color(DIM).small());
                        ui.add(
                            egui::Slider::new(&mut params.population_density, 0.05..=1.0)
                                .show_value(true)
                                .fixed_decimals(2),
                        );

                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if menu_button(ui, "\u{25b6}  Generate").clicked() {
                                command.action = MainMenuCommand::ConfirmWorldSetup;
                            }
                            ui.add_space(12.0);
                            if menu_button(ui, "Cancel").clicked() {
                                command.action = MainMenuCommand::CancelWorldSetup;
                            }
                        });
                    });
                });
        });
}

fn draw_worldgen_overlay(
    mut contexts: EguiContexts,
    state: Option<Res<State<AppState>>>,
    boot: Res<WorldGenBoot>,
    params: Res<WorldSetupParams>,
) {
    let Some(state) = state else {
        return;
    };
    if *state.get() != AppState::WorldGen {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let progress = (boot.elapsed / WORLDGEN_BOOT_SECONDS).clamp(0.0, 1.0);
    let preset = WORLDGEN_PRESETS
        .get(params.climate_preset % WORLDGEN_PRESETS.len())
        .copied()
        .unwrap_or(WORLDGEN_PRESETS[0]);

    egui::Area::new(egui::Id::new("worldgen_panel_area"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(GLASS_FILL)
                .inner_margin(egui::Margin::same(24))
                .corner_radius(egui::CornerRadius::same(12))
                .show(ui, |ui| {
                    ui.set_min_width(420.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Generating world")
                                .size(28.0)
                                .color(KC_ACCENT)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{preset} · seed {:016X}",
                                params.seed
                            ))
                            .color(DIM),
                        );
                        ui.add_space(16.0);
                        let bar = egui::ProgressBar::new(progress)
                            .desired_width(320.0)
                            .show_percentage();
                        ui.add(bar);
                        ui.add_space(8.0);
                        let step = if progress < 0.25 {
                            "Seeding continents…"
                        } else if progress < 0.5 {
                            "Raising terrain & biomes…"
                        } else if progress < 0.75 {
                            "Spawning factions…"
                        } else {
                            "Streaming first chunks…"
                        };
                        ui.label(egui::RichText::new(step).color(DIM).italics());
                    });
                });
        });
}

fn draw_pause_menu(
    mut contexts: EguiContexts,
    mut mode: ResMut<GameUiMode>,
    mut command: ResMut<MenuCommand>,
    mut save_panel: ResMut<SaveLoadPanel>,
    mut game_settings: Option<ResMut<GameSettings>>,
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
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .corner_radius(egui::CornerRadius::same(12))
                .stroke(egui::Stroke::new(1.5, ACCENT.gamma_multiply(0.5)))
                .inner_margin(egui::Margin::same(32))
                .show(ui, |ui| {
                    ui.set_min_width(300.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("\u{23f8} PAUSED")
                                .size(28.0)
                                .color(ACCENT)
                                .strong(),
                        );
                        ui.add_space(20.0);
                        if menu_button(ui, "\u{25b6}  Resume").clicked() {
                            *mode = GameUiMode::Playing;
                        }
                        ui.add_space(6.0);
                        if menu_button(ui, "\u{2699}  Settings").clicked() {
                            if let Some(settings) = game_settings.as_mut() {
                                settings.open = true;
                            }
                            command.action = MainMenuCommand::OpenSettings;
                        }
                        ui.add_space(6.0);
                        if menu_button(ui, "\u{1f4be}  Save/Load").clicked() {
                            command.action = MainMenuCommand::OpenSavePanel;
                            save_panel.visible = true;
                        }
                        if menu_button(ui, "\u{1f30d}  Main Menu").clicked() {
                            command.action = MainMenuCommand::ExitToMainMenu;
                        }
                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);
                        if menu_button(ui, "\u{23fb}  Quit").clicked() {
                            exit.write(AppExit::Success);
                        }
                    });
                });
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

fn ui_png_exists(stem: &str) -> bool {
    let path = format!("{}/assets/ui/{stem}.png", env!("CARGO_MANIFEST_DIR"));
    std::path::Path::new(&path).exists()
}

fn load_main_menu_title_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut assets = MainMenuTitleAssets::default();
    if ui_png_exists("title-bg") {
        assets.background = Some(asset_server.load("ui/title-bg.png"));
    }
    if ui_png_exists("logo") {
        assets.logo = Some(asset_server.load("ui/logo.png"));
    }
    if ui_png_exists("wordmark") {
        assets.wordmark = Some(asset_server.load("ui/wordmark.png"));
    }
    commands.insert_resource(assets);
}

fn live_stream_has_content(scene: &LiveStreamScene) -> bool {
    !scene.chunks.is_empty()
        || !scene.agents.is_empty()
        || !scene.buildings.is_empty()
        || !scene.graph_parcels.is_empty()
}

fn start_world_boot(client: &crate::ws_client::WsClient, preset: &str, seed: u64) {
    let init_seed = if seed == 0 {
        WORLDGEN_DEFAULT_SEED
    } else {
        seed
    };
    client.send_rpc(
        "sim.load_scenario",
        serde_json::json!({ "preset": preset, "seed": init_seed }),
    );
    client.send_rpc("sim.reset", serde_json::json!({ "seed": init_seed }));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-024 — pause/state transition helpers exercise menu-path and world-setup behavior.
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
