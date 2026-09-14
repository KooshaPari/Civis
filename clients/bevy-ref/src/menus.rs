#![cfg(all(feature = "bevy", feature = "egui"))]

//! Menus and overlay plugin for the Civis reference client (FR-CIV-BEVY-024 / item 49).
//! Settings GPU readout: FR-CIV-BEVY-036 / item 61.

use crate::event_feed::{EventFeed, EventKind};
use crate::faction_hud::PlayerFactionId;
use crate::game_ui::GameSpeed;
use crate::gpu_features::GpuCapabilities;
use crate::live_attach::LiveAttachBridge;
use crate::live_stream::{clear_live_stream_scene_in_world, LiveStreamScene};
use crate::outcome_overlay::{
    begin_player_session, end_player_session, outcome_modal_visible, OutcomeEscapeBlock,
    OutcomeOverlayState, OutcomeSessionGate,
};
use crate::save_load_ui::SaveLoadPanel;
use crate::settings_ui::{GameSettings, KeyBinding, ACTION_PAUSE_SIM};
use crate::ui_theme::{CHIP_FILL, GLASS_FILL, KC_ACCENT};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use std::time::{Duration, Instant};

const WORLDGEN_PRESETS: [&str; 4] = [
    "single-race-ardani",
    "three-race-balanced",
    "ardani-dominant",
    "lush-frontier",
];
const WORLDGEN_DEFAULT_SEED: u64 = 0xC1F1_5EED_D3AD_BEEF;
const WORLDGEN_TIMEOUT_SECONDS: f32 = 30.0;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Opaque enough to keep pause chrome readable over bright world / title art.
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(12, 14, 22, 248);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
/// Stronger scrim so shipped backgrounds do not wash out the pause panel.
const OVERLAY_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 210);

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
    /// Retry the failed new-world or save-load operation.
    RetryWorldLoad,
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

const LIVE_SPEED_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// One live speed request retained until a terminal server response arrives.
/// This prevents the local shell from claiming an unacknowledged pause/resume.
#[derive(Resource, Default)]
pub struct PendingSimSpeed {
    request: Option<PendingSimSpeedRequest>,
}

struct PendingSimSpeedRequest {
    ticket: crate::ws_client::RpcTicket,
    multiplier: u32,
    shell_mode: Option<GameUiMode>,
    sent_at: Instant,
}

/// Queue a server-authoritative speed change. A second request waits for the
/// first request's terminal reply rather than overwriting its ticket.
pub fn request_live_speed(
    pending: &mut PendingSimSpeed,
    bridge: Option<&LiveAttachBridge>,
    multiplier: u32,
    shell_mode: Option<GameUiMode>,
) -> bool {
    let Some(bridge) = bridge else {
        return false;
    };
    if pending.request.is_some() {
        return false;
    }
    pending.request = Some(PendingSimSpeedRequest {
        ticket: bridge
            .client
            .request_rpc("sim.set_speed", serde_json::json!({ "multiplier": multiplier })),
        multiplier,
        shell_mode,
        sent_at: Instant::now(),
    });
    true
}

/// Map standalone-only 5x/10x values to the server's supported 4x/8x values.
fn server_speed_multiplier(speed: f32) -> u32 {
    match speed.round() as u32 {
        1 | 2 | 4 | 8 => speed.round() as u32,
        5 => 4,
        10 => 8,
        _ => 1,
    }
}

fn accepted_speed_multiplier(
    result: Result<serde_json::Value, String>,
    requested: u32,
) -> Result<u32, String> {
    let result = result?;
    if result.get("accepted").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err("The server did not accept the speed change.".to_owned());
    }
    let multiplier = result
        .get("multiplier")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| "The server reply did not include a valid speed multiplier.".to_owned())?;
    if multiplier != requested {
        return Err(format!(
            "The server acknowledged speed {multiplier}x, not the requested {requested}x."
        ));
    }
    Ok(multiplier)
}

fn reconcile_live_speed_request(
    mut pending: ResMut<PendingSimSpeed>,
    mut mode: ResMut<GameUiMode>,
    mut speed: ResMut<GameSpeed>,
    mut feed: Option<ResMut<EventFeed>>,
) {
    let completed = pending.request.as_ref().and_then(|request| {
        request
            .ticket
            .try_recv()
            .map(|reply| (reply, false))
            .or_else(|| {
                (request.sent_at.elapsed() >= LIVE_SPEED_REQUEST_TIMEOUT).then_some((
                    Err("The server did not reply; simulation speed is unknown.".to_owned()),
                    true,
                ))
            })
    });
    let Some((reply, timed_out)) = completed else {
        return;
    };
    let request = pending.request.take().expect("pending speed request exists");
    match accepted_speed_multiplier(reply, request.multiplier) {
        Ok(multiplier) => {
            speed.multiplier = multiplier as f32;
            speed.remember_non_zero();
            if let Some(shell_mode) = request.shell_mode {
                *mode = shell_mode;
            }
        }
        Err(error) => {
            if let Some(feed) = feed.as_deref_mut() {
                let prefix = if timed_out {
                    "Simulation speed request timed out"
                } else {
                    "Simulation speed request failed"
                };
                feed.push(EventKind::System, format!("{prefix}: {error}"));
            }
        }
    }
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

/// System condition: true only when the game is actively playing (not in a menu).
/// Used as `.run_if(in_playing)` on draw systems to prevent HUD bleed-through.
pub fn in_playing(mode: Res<GameUiMode>) -> bool {
    *mode == GameUiMode::Playing
}

/// Per-world setup parameters shared by voxel generation and the map view.
#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldSetupParams {
    /// World seed selected for the current run.
    pub seed: u64,
    /// Climate / scenario preset index into [`WORLDGEN_PRESETS`].
    pub climate_preset: usize,
    /// Local player faction id (0 = Ardani, 1 = Velthari, 2 = Grundak).
    pub player_faction: u32,
}

impl Default for WorldSetupParams {
    fn default() -> Self {
        Self {
            seed: 0xC1F1_5EED_D3AD_BEEF,
            climate_preset: 0,
            player_faction: 0,
        }
    }
}

/// Transient world-gen boot timer (standalone server attach).
#[derive(Resource, Default, Debug)]
pub struct WorldGenBoot {
    /// Elapsed seconds while in [`AppState::WorldGen`].
    pub elapsed: f32,
    /// Whether the previous menu command queued a scene clear that must apply
    /// before an existing scene can satisfy the readiness check.
    pub scene_clear_pending: bool,
    /// A visible terminal failure; elapsed time never implies a ready world.
    pub error: Option<String>,
    pending: Option<crate::ws_client::RpcTicket>,
    generation: Option<u64>,
    retry_action: MainMenuCommand,
}

/// Marks the supported in-process heightmap once its mesh has been spawned.
#[derive(Component, Debug)]
pub struct LocalTerrainReady;

/// Rasterised shell artwork (PNG only; vector sources are documented in PIPELINE.md).
#[derive(Resource, Default)]
pub struct MainMenuTitleAssets {
    /// Full-menu background (`ui/title-bg.png`).
    pub background: Option<Handle<Image>>,
    /// World-generation/loading background (`ui/loading-bg.png`).
    pub loading_background: Option<Handle<Image>>,
    /// Rotating worldgen emblem (`ui/loading-spinner.png`).
    pub loading_spinner: Option<Handle<Image>>,
    /// Logo mark (`ui/logo.png`).
    pub logo: Option<Handle<Image>>,
    /// Wordmark (`ui/wordmark.png`).
    pub wordmark: Option<Handle<Image>>,
}

/// Bevy plugin: pause overlay, era banners, settings window.
pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<GameUiMode>()
            .init_resource::<PendingSimSpeed>()
            .init_resource::<EraBanner>()
            .init_resource::<SettingsOpen>()
            .init_resource::<WorldSetupParams>()
            .init_resource::<MenuCommand>()
            .init_resource::<MainMenuSaves>()
            .init_resource::<WorldGenBoot>()
            // Idempotent with OutcomeOverlayPlugin (live attach).
            .init_resource::<OutcomeSessionGate>()
            .init_resource::<OutcomeOverlayState>()
            .init_resource::<MainMenuTitleAssets>()
            .add_systems(Startup, load_main_menu_title_assets)
            .add_systems(
                Update,
                (toggle_pause, reconcile_live_speed_request, tick_era_banner).chain(),
            )
            .add_systems(
                Update,
                (
                    sync_app_state_with_game_mode,
                    consume_menu_commands,
                    advance_worldgen_to_playing,
                )
                    .chain(),
            )
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

/// Admit only acknowledged replacement terrain, or the existing in-process bootstrap.
pub fn advance_worldgen_to_playing(
    mut commands: Commands,
    time: Res<Time>,
    mut boot: ResMut<WorldGenBoot>,
    state: Option<Res<State<AppState>>>,
    scene: Option<Res<LiveStreamScene>>,
    bridge: Option<Res<LiveAttachBridge>>,
    attach_mode: Option<Res<crate::AttachMode>>,
    local_world: (
        Option<Res<crate::sim_bridge::SimState>>,
        Query<&Mesh3d, With<LocalTerrainReady>>,
        Option<Res<Assets<Mesh>>>,
    ),
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(state) = state else {
        return;
    };
    if *state.get() != AppState::WorldGen {
        boot.elapsed = 0.0;
        return;
    }

    if boot.error.is_some() {
        return;
    }
    if let Some((connection, reply)) = boot.pending.as_ref().and_then(|ticket| {
        ticket
            .try_recv()
            .map(|reply| (ticket.connection_id(), reply))
    }) {
        boot.pending = None;
        match reply.and_then(world_load_generation) {
            Ok(generation) => {
                let Some(bridge) = bridge.as_ref() else {
                    boot.error = Some(
                        "The server connection is unavailable. Retry after reconnecting."
                            .to_owned(),
                    );
                    return;
                };
                let client = bridge.client.clone();
                commands.queue(move |world: &mut World| {
                    clear_live_stream_scene_in_world(world);
                    client.install_world_generation(generation, connection);
                });
                boot.generation = Some(generation);
                boot.scene_clear_pending = true;
                return;
            }
            Err(error) => {
                boot.error = Some(error);
                return;
            }
        }
    }

    if boot.scene_clear_pending {
        boot.scene_clear_pending = false;
        // A queued scene clear needs one frame to settle. Headless consumers
        // without a LiveStreamScene have nothing to drain, so do not delay
        // their deterministic WorldGen -> Playing transition.
        if scene.is_some() {
            return;
        }
    }

    boot.elapsed += time.delta_secs();

    let is_local = matches!(attach_mode.as_deref(), Some(crate::AttachMode::Standalone))
        || (attach_mode.is_none() && bridge.is_none() && scene.is_none());
    let ready = if is_local {
        let (sim, terrain, meshes) = &local_world;
        sim.is_some()
            && terrain.iter().any(|mesh| {
                meshes
                    .as_ref()
                    .is_some_and(|assets| assets.get(&mesh.0).is_some())
            })
    } else {
        match (scene.as_deref(), boot.generation) {
            (Some(scene), Some(generation)) => {
                if !bridge
                    .as_ref()
                    .is_some_and(|b| b.client.world_generation_is_active(generation))
                {
                    boot.error =
                        Some("Connection changed while loading. Reconnect and retry.".to_owned());
                    return;
                }
                live_stream_has_content(scene)
            }
            _ => false,
        }
    };

    if ready {
        if let Some(bridge) = bridge.as_ref() {
            bridge.client.finish_world_load();
        }
        next_state.set(AppState::Playing);
        boot.elapsed = 0.0;
    } else if boot.elapsed >= WORLDGEN_TIMEOUT_SECONDS {
        boot.pending = None;
        boot.error = Some(if boot.generation.is_some() {
            "The server accepted the world, but no terrain was rendered. Check the server connection and retry."
        } else {
            "The server did not finish loading the world. Check the connection and retry."
        }.to_owned());
    }
}

fn world_load_generation(result: serde_json::Value) -> Result<u64, String> {
    result
        .get("scene_generation")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            "This server cannot confirm a fresh world. Update the server and retry.".to_owned()
        })
}

/// Consume one-shot [`MenuCommand`] actions from shell buttons.
pub fn consume_menu_commands(
    mut commands: Commands,
    mut menu_command: ResMut<MenuCommand>,
    state: Option<Res<State<AppState>>>,
    mut next_state: ResMut<NextState<AppState>>,
    connection: (
        Option<Res<LiveAttachBridge>>,
        Option<Res<crate::AttachMode>>,
    ),
    mut save_panel: Option<ResMut<SaveLoadPanel>>,
    saves: Res<MainMenuSaves>,
    params: Res<WorldSetupParams>,
    mut game_mode: ResMut<GameUiMode>,
    mut exit: MessageWriter<AppExit>,
    gate: Option<ResMut<OutcomeSessionGate>>,
    overlay: Option<ResMut<OutcomeOverlayState>>,
    #[allow(unused_mut)] mut boot: ResMut<WorldGenBoot>,
    #[allow(unused_mut)] mut game_settings: Option<ResMut<GameSettings>>,
    #[allow(unused_mut)] mut settings_open: Option<ResMut<SettingsOpen>>,
    #[allow(unused_mut)] mut game_speed: Option<ResMut<GameSpeed>>,
) {
    let (bridge, attach_mode) = connection;
    let bridge = if matches!(attach_mode.as_deref(), Some(crate::AttachMode::Standalone)) {
        None
    } else {
        bridge
    };
    let Some(state) = state else {
        return;
    };
    if menu_command.action == MainMenuCommand::None {
        return;
    }

    let action = if menu_command.action == MainMenuCommand::RetryWorldLoad {
        boot.retry_action
    } else {
        menu_command.action
    };
    menu_command.action = MainMenuCommand::None;
    match action {
        MainMenuCommand::None | MainMenuCommand::RetryWorldLoad => {}
        MainMenuCommand::NewWorld => {
            next_state.set(AppState::WorldSetup);
        }
        MainMenuCommand::ConfirmWorldSetup => {
            *boot = WorldGenBoot {
                retry_action: action,
                ..Default::default()
            };
            commands.insert_resource(PlayerFactionId(params.player_faction));
            if let Some(bridge) = bridge.as_ref() {
                let preset = WORLDGEN_PRESETS
                    .get(params.climate_preset % WORLDGEN_PRESETS.len())
                    .copied()
                    .unwrap_or(WORLDGEN_PRESETS[0]);
                bridge.client.suspend_world_stream();
                boot.pending = Some(start_world_boot(&bridge.client, preset, params.seed));
            }
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                begin_player_session(
                    bridge.as_ref().map(|bridge| &bridge.client),
                    &mut gate,
                    &mut overlay,
                );
            }
            boot.elapsed = 0.0;
            next_state.set(AppState::WorldGen);
        }
        MainMenuCommand::CancelWorldSetup => {
            next_state.set(AppState::MainMenu);
            *boot = WorldGenBoot::default();
        }
        MainMenuCommand::Continue => {
            *boot = WorldGenBoot {
                retry_action: action,
                ..Default::default()
            };
            if let Some(bridge) = bridge.as_ref() {
                bridge.client.clear_outcomes();
                let slot_name = saves
                    .preferred_slot
                    .as_deref()
                    .unwrap_or("slot-1")
                    .to_string();
                bridge.client.suspend_world_stream();
                boot.pending = Some(
                    bridge
                        .client
                        .request_rpc("save.load", serde_json::json!({ "slot_name": slot_name })),
                );
            }
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                begin_player_session(
                    bridge.as_ref().map(|bridge| &bridge.client),
                    &mut gate,
                    &mut overlay,
                );
            }
            boot.elapsed = 0.0;
            next_state.set(AppState::WorldGen);
        }
        MainMenuCommand::LoadGame => {
            if let Some(save_panel) = save_panel.as_mut() {
                save_panel.visible = true;
            }
        }
        MainMenuCommand::Resume => {
            resume_shell_pause(&mut game_mode, game_speed.as_deref_mut());
            if *state.get() == AppState::Paused {
                next_state.set(AppState::Playing);
            }
        }
        MainMenuCommand::OpenSettings => {
            if let Some(mut settings) = game_settings {
                settings.open = true;
                settings.active_tab = crate::settings_ui::SettingsTab::Graphics;
            }
            if let Some(flag) = settings_open.as_mut() {
                flag.0 = true;
            }
        }
        MainMenuCommand::OpenSavePanel => {
            if let Some(save_panel) = save_panel.as_mut() {
                save_panel.visible = true;
            }
        }
        MainMenuCommand::ExitToMainMenu => {
            commands.queue(|world: &mut World| {
                clear_live_stream_scene_in_world(world);
            });
            if let (Some(mut gate), Some(mut overlay)) = (gate, overlay) {
                end_player_session(&mut gate, &mut overlay);
            }
            next_state.set(AppState::MainMenu);
            *game_mode = GameUiMode::Playing;
            *boot = WorldGenBoot::default();
            if let Some(bridge) = bridge.as_ref() {
                bridge.client.finish_world_load();
            }
        }
        MainMenuCommand::Quit => {
            *game_mode = GameUiMode::Playing;
            exit.write(AppExit::Success);
        }
    }
}

pub(crate) fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings: Option<Res<GameSettings>>,
    app_state: Option<Res<State<AppState>>>,
    outcome_overlay: Option<Res<OutcomeOverlayState>>,
    escape_block: Option<Res<OutcomeEscapeBlock>>,
    mut controls_help: Option<ResMut<crate::controls_help::ControlsHelpOpen>>,
    mut mode: ResMut<GameUiMode>,
    mut game_speed: Option<ResMut<GameSpeed>>,
    attach_mode: Option<Res<crate::AttachMode>>,
    bridge: Option<Res<LiveAttachBridge>>,
    mut pending_speed: ResMut<PendingSimSpeed>,
) {
    // Single owner for ACTION_PAUSE_SIM: shell pause overlay (Space default).
    // Esc remains a hard fallback so Close Panel / overlay escape still works.
    // game_ui no longer toggles GameSpeed on this action.
    let pause_binding = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_PAUSE_SIM))
        .unwrap_or(KeyBinding::Key(KeyCode::Space));
    let binding_pressed = pause_binding.is_just_pressed(&keys, &mouse_buttons);
    let esc_pressed = keys.just_pressed(KeyCode::Escape);
    if !esc_pressed && !binding_pressed {
        return;
    }

    // Same-frame: outcome modal dismissed Esc → do not also toggle pause.
    if escape_block.map(|block| block.0).unwrap_or(false) {
        return;
    }
    if let Some(overlay) = outcome_overlay.as_ref() {
        if outcome_modal_visible(overlay) {
            return;
        }
    }

    // Controls cheat sheet owns Esc while open.
    if esc_pressed {
        if let Some(help) = controls_help.as_deref_mut() {
            if help.0 {
                help.0 = false;
                return;
            }
        }
    }

    // Settings panel owns Esc while open (Close Panel). MenusPlugin is
    // registered before SettingsPlugin so `open` is still true this frame.
    // Only gate the Esc path so a rebound ACTION_PAUSE_SIM can still toggle.
    if esc_pressed && settings.as_ref().is_some_and(|s| s.open) {
        return;
    }

    if let Some(app_state) = app_state {
        if *app_state.get() != AppState::Playing && *app_state.get() != AppState::Paused {
            return;
        }
    }

    let server_attach = attach_mode.is_some_and(|attach| *attach == crate::AttachMode::Server);
    if server_attach {
        let (multiplier, shell_mode) = match *mode {
            GameUiMode::Playing => (0, GameUiMode::Paused),
            GameUiMode::Paused => (
                game_speed
                    .as_deref()
                    .map_or(1, |speed| server_speed_multiplier(speed.last_non_zero)),
                GameUiMode::Playing,
            ),
        };
        request_live_speed(
            &mut pending_speed,
            bridge.as_deref(),
            multiplier,
            Some(shell_mode),
        );
        return;
    }

    match *mode {
        GameUiMode::Playing => {
            *mode = GameUiMode::Paused;
            // Keep HUD speed chips in sync with the overlay (sim already stops via
            // GameUiMode in sim_bridge; zeroing makes Resume restore the prior rate).
            if let Some(speed) = game_speed.as_deref_mut() {
                speed.remember_non_zero();
                speed.multiplier = 0.0;
            }
        }
        GameUiMode::Paused => resume_shell_pause(&mut mode, game_speed.as_deref_mut()),
    }
}

/// Clear shell pause overlay and unstick sim speed if it was left at 0.
fn resume_shell_pause(mode: &mut GameUiMode, speed: Option<&mut GameSpeed>) {
    *mode = GameUiMode::Playing;
    if let Some(speed) = speed {
        speed.restore_after_resume();
    }
}

/// Resume the visible pause menu without claiming a live-server resume before
/// its `sim.set_speed` acknowledgement arrives.
fn resume_from_pause_menu(
    mode: &mut GameUiMode,
    speed: Option<&mut GameSpeed>,
    server_attach: bool,
    bridge: Option<&LiveAttachBridge>,
    pending_speed: &mut PendingSimSpeed,
) {
    if server_attach {
        let multiplier = speed
            .as_deref()
            .map_or(1, |speed| server_speed_multiplier(speed.last_non_zero));
        request_live_speed(
            pending_speed,
            bridge,
            multiplier,
            Some(GameUiMode::Playing),
        );
    } else {
        resume_shell_pause(mode, speed);
    }
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

/// System condition: true only when AppState is Playing or Paused.
/// This is the correct gate for in-game UI — unlike `in_game` which
/// checks GameUiMode (default Playing), this checks AppState (default MainMenu).
pub fn in_playing_state(state: Res<State<AppState>>) -> bool {
    let s: AppState = state.get().clone();
    s == AppState::Playing || s == AppState::Paused
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
        .anchor(egui::Align2::LEFT_CENTER, egui::vec2(72.0, 0.0))
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
                        if menu_button(ui, "✕  Quit").clicked() {
                            command.action = MainMenuCommand::Quit;
                        }
                    });
                });
        });

    // Right-side enrichment panel (banners + stat tile)
    egui::Area::new(egui::Id::new("main_menu_right"))
        .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-48.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(GLASS_FILL)
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.set_min_width(300.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("WHATS NEW").size(13.0).color(DIM).strong(),
                        );
                        ui.add_space(8.0);
                        banner_tile(
                            ui,
                            "Sandbox Reactions",
                            "Fire burns wood, lava makes steam + stone — material physics are live.",
                            KC_ACCENT,
                        );
                        ui.add_space(10.0);
                        banner_tile(
                            ui,
                            "Emergent Civ",
                            "30+ simulation phases run every tick. Watch civs form, trade, and grow.",
                            CHIP_FILL,
                        );
                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);
                        egui::Frame::NONE
                            .fill(CHIP_FILL)
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::same(16))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("VERSION").size(12.0).color(DIM).strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new("v0.5").size(15.0).color(KC_ACCENT).strong());
                                    });
                                });
                            });
                    });
                });
        });
}

fn banner_tile(ui: &mut egui::Ui, title: &str, body: &str, accent: egui::Color32) {
    egui::Frame::NONE
        .fill(CHIP_FILL)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_width(272.0);
            ui.label(egui::RichText::new(title).size(16.0).color(accent).strong());
            ui.add_space(4.0);
            ui.label(egui::RichText::new(body).size(13.0).color(DIM));
        });
}

fn draw_world_setup(
    mut contexts: EguiContexts,
    state: Option<Res<State<AppState>>>,
    mut params: ResMut<WorldSetupParams>,
    mut command: ResMut<MenuCommand>,
    mut seed_edit: Local<Option<String>>,
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
                        let seed_text =
                            seed_edit.get_or_insert_with(|| format!("{:016X}", params.seed));
                        if ui
                            .add(
                                egui::TextEdit::singleline(seed_text)
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
                                *seed_text = format!("{:016X}", params.seed);
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
    titles: Res<MainMenuTitleAssets>,
    images: Res<Assets<Image>>,
    mut menu_command: ResMut<MenuCommand>,
) {
    let Some(state) = state else {
        return;
    };
    if *state.get() != AppState::WorldGen {
        return;
    }
    let bg_tex = titles.loading_background.as_ref().and_then(|handle| {
        images
            .get(handle)
            .map(|_| contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone())))
    });
    let spinner_tex = titles.loading_spinner.as_ref().and_then(|handle| {
        images
            .get(handle)
            .map(|_| contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone())))
    });
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let preset = WORLDGEN_PRESETS
        .get(params.climate_preset % WORLDGEN_PRESETS.len())
        .copied()
        .unwrap_or(WORLDGEN_PRESETS[0]);

    if let Some(id) = bg_tex {
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("worldgen_bg"))
            .fixed_pos(screen.min)
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                ui.image((id, screen.size()));
            });
    }

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
                            egui::RichText::new(if boot.error.is_some() {
                                "World could not be loaded"
                            } else {
                                "Loading world"
                            })
                            .size(28.0)
                            .color(KC_ACCENT)
                            .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!("{preset} · seed {:016X}", params.seed))
                                .color(DIM),
                        );
                        ui.add_space(12.0);
                        if let Some(id) = spinner_tex {
                            let angle = boot.elapsed * 1.75;
                            ui.add(
                                egui::Image::new((id, egui::vec2(72.0, 72.0)))
                                    .rotate(angle, egui::vec2(0.5, 0.5)),
                            );
                            ui.add_space(8.0);
                        }
                        if let Some(error) = &boot.error {
                            ui.label(egui::RichText::new(error).color(DIM));
                            if ui.button("Retry").clicked() {
                                menu_command.action = MainMenuCommand::RetryWorldLoad;
                            }
                        } else {
                            ui.spinner();
                            let step = if boot.generation.is_some() {
                                "World accepted. Waiting for terrain…"
                            } else {
                                "Waiting for the world to load…"
                            };
                            ui.label(egui::RichText::new(step).color(DIM).italics());
                        }
                        if ui.button("Back to menu").clicked() {
                            menu_command.action = MainMenuCommand::ExitToMainMenu;
                        }
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
    mut settings_open: ResMut<SettingsOpen>,
    mut game_speed: Option<ResMut<GameSpeed>>,
    attach_mode: Option<Res<crate::AttachMode>>,
    bridge: Option<Res<LiveAttachBridge>>,
    mut pending_speed: ResMut<PendingSimSpeed>,
    mut exit: MessageWriter<AppExit>,
) {
    if *mode != GameUiMode::Paused {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    dim_overlay(ctx);

    let settings_is_open = game_settings
        .as_ref()
        .map(|s| s.open)
        .unwrap_or(settings_open.0);
    // Mirror into SettingsOpen so shell consumers stay aligned.
    settings_open.0 = settings_is_open;
    // Settings opened from pause: keep dim + cue, hide the button stack so Esc
    // closing settings returns to this menu instead of a buried / dead-end state.
    if settings_is_open {
        egui::Area::new(egui::Id::new("pause_settings_cue"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -36.0))
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(PANEL_FILL)
                    .corner_radius(egui::CornerRadius::same(8))
                    .stroke(egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.35)))
                    .inner_margin(egui::Margin::symmetric(18, 10))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Settings open — Esc returns to pause menu")
                                .size(14.0)
                                .color(DIM),
                        );
                    });
            });
        return;
    }

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
                        ui.label(
                            egui::RichText::new("Space / Esc — resume")
                                .size(13.0)
                                .color(DIM)
                                .italics(),
                        );
                        ui.add_space(16.0);
                        if menu_button(ui, "\u{25b6}  Resume").clicked() {
                            let server_attach = attach_mode
                                .as_deref()
                                .is_some_and(|attach| *attach == crate::AttachMode::Server);
                            resume_from_pause_menu(
                                &mut mode,
                                game_speed.as_deref_mut(),
                                server_attach,
                                bridge.as_deref(),
                                &mut pending_speed,
                            );
                        }
                        ui.add_space(6.0);
                        if menu_button(ui, "\u{2699}  Settings").clicked() {
                            if let Some(settings) = game_settings.as_mut() {
                                settings.open = true;
                                settings.active_tab = crate::settings_ui::SettingsTab::Graphics;
                            }
                            settings_open.0 = true;
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
                        if menu_button(ui, "✕  Quit").clicked() {
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
#[allow(dead_code)] // surfaced via the GPU unavailable flag in the settings panel
pub fn format_gpu_capabilities_unavailable_message() -> &'static str {
    "GPU capabilities unavailable (headless or still starting up)"
}

#[allow(dead_code)] // surfaced via the GPU unavailable flag in the settings panel
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
        .min_size(egui::vec2(260.0, 46.0))
        .corner_radius(egui::CornerRadius::same(10));
    ui.add(btn)
}

fn load_main_menu_title_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Always request via AssetServer; draw_main_menu falls back to text when
    // handles fail to resolve (shipped builds must not depend on CARGO_MANIFEST_DIR).
    commands.insert_resource(MainMenuTitleAssets {
        background: Some(asset_server.load("ui/title-bg.png")),
        loading_background: Some(asset_server.load("ui/loading-bg.png")),
        loading_spinner: Some(asset_server.load("ui/loading-spinner.png")),
        logo: Some(asset_server.load("ui/logo.png")),
        wordmark: Some(asset_server.load("ui/wordmark.png")),
    });
}

fn live_stream_has_content(scene: &LiveStreamScene) -> bool {
    !scene.chunks.is_empty()
}

fn start_world_boot(
    client: &crate::ws_client::WsClient,
    preset: &str,
    seed: u64,
) -> crate::ws_client::RpcTicket {
    let init_seed = if seed == 0 {
        WORLDGEN_DEFAULT_SEED
    } else {
        seed
    };
    client.request_rpc(
        "sim.load_scenario",
        serde_json::json!({ "preset": preset, "seed": init_seed }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_boot_sends_one_scenario_request_without_reset() {
        let (client, requests) = crate::ws_client::WsClient::test_rpc_client();
        let ticket = start_world_boot(&client, "three-race-balanced", 987);
        let request: serde_json::Value = serde_json::from_str(&requests.recv().unwrap()).unwrap();
        assert_eq!(request["id"].as_u64(), Some(ticket.id));
        assert_eq!(request["method"], "sim.load_scenario");
        assert_eq!(request["params"]["seed"], 987);
        assert!(
            requests.try_recv().is_err(),
            "a second reset would discard the preset"
        );
        assert!(
            world_load_generation(serde_json::json!({"accepted":true})).is_err(),
            "an older server must not silently satisfy fresh-world readiness"
        );
    }

    fn world_boot_app() -> (App, crate::ws_client::WsClient, u64, Entity) {
        let (client, _requests) = crate::ws_client::WsClient::test_rpc_client();
        client.suspend_world_stream();
        let ticket = client.request_rpc("sim.load_scenario", serde_json::json!({}));
        let id = ticket.id;
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .insert_resource(Time::<()>::default())
            .insert_resource(LiveAttachBridge {
                client: client.clone(),
            })
            .insert_resource(crate::AttachMode::Server)
            .insert_resource(WorldGenBoot {
                pending: Some(ticket),
                ..Default::default()
            })
            .insert_resource(LiveStreamScene::default())
            .add_systems(Update, advance_worldgen_to_playing);
        let old = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<LiveStreamScene>()
            .chunks
            .insert(1, old);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldGen);
        (app, client, id, old)
    }

    #[test]
    fn world_boot_failed_reply_preserves_old_scene_and_blocks_playing() {
        let (mut app, client, id, old) = world_boot_app();
        client.test_complete_rpc(id, Err("preset not found".to_string()));
        app.update();
        assert_eq!(
            app.world().resource::<State<AppState>>().get(),
            &AppState::WorldGen
        );
        assert_eq!(
            app.world().resource::<WorldGenBoot>().error.as_deref(),
            Some("preset not found")
        );
        assert_eq!(
            app.world().resource::<LiveStreamScene>().chunks.get(&1),
            Some(&old)
        );
        assert!(app.world().get_entity(old).is_ok());
    }

    #[test]
    fn world_boot_ack_clears_previous_scene_before_new_terrain_can_play() {
        let (mut app, client, id, old) = world_boot_app();
        app.update();
        assert_eq!(
            app.world().resource::<LiveStreamScene>().chunks.get(&1),
            Some(&old)
        );
        client.test_complete_rpc(id, Ok(serde_json::json!({"scene_generation":2})));
        app.update();
        assert!(app.world().resource::<LiveStreamScene>().chunks.is_empty());
        assert!(app.world().get_entity(old).is_err());
        app.update(); // Clear barrier settles.
        app.update(); // An ACK alone still does not represent rendered terrain.
        assert_eq!(
            app.world().resource::<State<AppState>>().get(),
            &AppState::WorldGen
        );
        let fresh = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<LiveStreamScene>()
            .chunks
            .insert(2, fresh);
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<State<AppState>>().get(),
            &AppState::Playing
        );
    }

    /// FR-CIV-BEVY-024 — pause/state transition helpers exercise menu-path and world-setup behavior.
    #[test]
    fn era_banner_announce_sets_timer() {
        let mut banner = EraBanner::default();
        banner.announce("Bronze");
        assert_eq!(banner.current_era, "Bronze");
        assert!((banner.show_timer - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn app_state_default_is_main_menu() {
        assert_eq!(AppState::default(), AppState::MainMenu);
    }

    #[test]
    fn game_ui_mode_default_is_playing() {
        assert_eq!(GameUiMode::default(), GameUiMode::Playing);
    }

    #[test]
    fn world_gen_boot_default_is_zero_elapsed() {
        assert!((WorldGenBoot::default().elapsed).abs() < f32::EPSILON);
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

    #[test]
    fn resume_shell_pause_restores_stuck_zero_speed() {
        let mut mode = GameUiMode::Paused;
        let mut speed = GameSpeed {
            multiplier: 0.0,
            last_non_zero: 2.0,
        };
        resume_shell_pause(&mut mode, Some(&mut speed));
        assert_eq!(mode, GameUiMode::Playing);
        assert!((speed.multiplier - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pause_menu_resume_in_server_attach_waits_for_speed_acknowledgement() {
        use std::time::Duration;

        let (client, requests) = crate::ws_client::WsClient::test_rpc_client();
        let bridge = LiveAttachBridge { client };
        let mut mode = GameUiMode::Paused;
        let mut speed = GameSpeed {
            multiplier: 0.0,
            last_non_zero: 2.0,
        };
        let mut pending = PendingSimSpeed::default();

        resume_from_pause_menu(
            &mut mode,
            Some(&mut speed),
            true,
            Some(&bridge),
            &mut pending,
        );

        let request: serde_json::Value = serde_json::from_str(
            &requests
                .recv_timeout(Duration::from_secs(1))
                .expect("pause-menu resume request queued"),
        )
        .unwrap();
        assert_eq!(request["method"], "sim.set_speed");
        assert_eq!(request["params"]["multiplier"], 2);
        assert_eq!(mode, GameUiMode::Paused);
        assert_eq!(speed.multiplier, 0.0);
    }

    #[test]
    fn space_pause_binding_toggles_overlay_and_zeros_speed() {
        use bevy::prelude::{App, ButtonInput, Time, Update};
        use std::time::Duration;

        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(Time::<()>::default());
        app.insert_resource(GameUiMode::Playing);
        app.init_resource::<PendingSimSpeed>();
        app.insert_resource(GameSpeed {
            multiplier: 1.0,
            last_non_zero: 1.0,
        });
        app.insert_resource(GameSettings::default());
        app.add_systems(Update, toggle_pause);

        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::Space);
        }
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(16));
        app.update();

        assert_eq!(*app.world().resource::<GameUiMode>(), GameUiMode::Paused);
        assert_eq!(app.world().resource::<GameSpeed>().multiplier, 0.0);
        assert!((app.world().resource::<GameSpeed>().last_non_zero - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn live_pause_and_resume_wait_for_set_speed_acknowledgements() {
        use bevy::prelude::{App, ButtonInput, Time, Update};
        use std::time::Duration;

        let (client, requests) = crate::ws_client::WsClient::test_rpc_client();
        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .insert_resource(Time::<()>::default())
            .insert_resource(GameUiMode::Playing)
            .insert_resource(GameSpeed {
                multiplier: 1.0,
                last_non_zero: 1.0,
            })
            .insert_resource(GameSettings::default())
            .insert_resource(crate::AttachMode::Server)
            .insert_resource(LiveAttachBridge {
                client: client.clone(),
            })
            .init_resource::<PendingSimSpeed>()
            .add_systems(Update, (toggle_pause, reconcile_live_speed_request).chain());

        let press_space = |app: &mut App| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Space);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_millis(16));
            app.update();
        };
        let release_space = |app: &mut App| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::Space);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_millis(16));
            app.update();
        };

        press_space(&mut app);
        let pause: serde_json::Value = serde_json::from_str(
            &requests
                .recv_timeout(Duration::from_secs(1))
                .expect("pause request queued"),
        )
        .unwrap();
        assert_eq!(pause["method"], "sim.set_speed");
        assert_eq!(pause["params"]["multiplier"], 0);
        assert_eq!(*app.world().resource::<GameUiMode>(), GameUiMode::Playing);
        assert_eq!(app.world().resource::<GameSpeed>().multiplier, 1.0);

        let pause_id = pause["id"].as_u64().unwrap();
        client.test_complete_rpc(
            pause_id,
            Ok(serde_json::json!({"accepted": true, "multiplier": 0})),
        );
        app.update();
        assert_eq!(*app.world().resource::<GameUiMode>(), GameUiMode::Paused);
        assert_eq!(app.world().resource::<GameSpeed>().multiplier, 0.0);

        release_space(&mut app);
        press_space(&mut app);
        let resume: serde_json::Value = serde_json::from_str(
            &requests
                .recv_timeout(Duration::from_secs(1))
                .expect("resume request queued"),
        )
        .unwrap();
        assert_eq!(resume["method"], "sim.set_speed");
        assert_eq!(resume["params"]["multiplier"], 1);
        assert_eq!(*app.world().resource::<GameUiMode>(), GameUiMode::Paused);

        let resume_id = resume["id"].as_u64().unwrap();
        client.test_complete_rpc(
            resume_id,
            Ok(serde_json::json!({"accepted": true, "multiplier": 1})),
        );
        app.update();
        assert_eq!(*app.world().resource::<GameUiMode>(), GameUiMode::Playing);
        assert_eq!(app.world().resource::<GameSpeed>().multiplier, 1.0);
    }
}
