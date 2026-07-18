//! SHELL-ATTEST-001 — headless shell smoke for AppState + menu command wiring.
//!
//! Exercises `consume_menu_commands`, `advance_worldgen_to_playing`, and session
//! gate helpers without opening a GPU window.

#![cfg(all(feature = "bevy", feature = "egui"))]

use std::time::Duration;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use civ_bevy_ref::live_stream::LiveStreamScene;
use civ_bevy_ref::menus::{
    AppState, GameUiMode, MainMenuCommand, MainMenuSaves, MenuCommand, WorldGenBoot,
    WorldSetupParams, advance_worldgen_to_playing, consume_menu_commands,
    sync_app_state_with_game_mode,
};
use civ_bevy_ref::outcome_overlay::{
    OutcomeOverlayState, OutcomeSessionGate, begin_player_session, end_player_session,
};
use civ_bevy_ref::settings_ui::{GameSettings, SettingsTab};

fn shell_smoke_app() -> App {
    let mut app = App::new();
    // Headless: StatesPlugin (not DefaultPlugins) so init_state has StateTransition.
    app.add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .insert_resource(MenuCommand::default())
        .insert_resource(WorldGenBoot::default())
        .insert_resource(WorldSetupParams::default())
        .insert_resource(GameUiMode::default())
        .insert_resource(MainMenuSaves::default())
        .insert_resource(OutcomeSessionGate::default())
        .insert_resource(OutcomeOverlayState::default())
        .insert_resource(Time::<()>::default())
        .add_message::<AppExit>()
        .add_systems(
            Update,
            (
                consume_menu_commands,
                advance_worldgen_to_playing,
                sync_app_state_with_game_mode,
            )
                .chain(),
        );
    app
}

fn current_app_state(app: &App) -> AppState {
    app.world().resource::<State<AppState>>().get().clone()
}

fn dispatch_menu(app: &mut App, action: MainMenuCommand) {
    app.world_mut().resource_mut::<MenuCommand>().action = action;
    // StateTransition runs before Update, so NextState set in Update applies next
    // frame. ConfirmWorldSetup can chain WorldGen→Playing in one more frame.
    flush_state(app);
}

fn advance_time(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    // Consume the delta on a single frame, then drain NextState without
    // re-applying the same elapsed amount across flush updates.
    app.update();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);
    for _ in 0..2 {
        app.update();
    }
}

fn flush_state(app: &mut App) {
    for _ in 0..3 {
        app.update();
    }
}

#[test]
fn app_state_defaults_to_main_menu() {
    let app = shell_smoke_app();
    assert_eq!(current_app_state(&app), AppState::MainMenu);
}

#[test]
fn shipped_shell_backgrounds_are_present() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
    for file in ["title-bg.png", "loading-bg.png", "loading-spinner.png"] {
        let path = asset_root.join(file);
        let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
            panic!("missing shipped shell asset {}: {error}", path.display())
        });
        assert!(
            metadata.len() > 1024,
            "shipped shell asset is unexpectedly empty: {}",
            path.display()
        );
    }
}

#[test]
fn shipped_hud_panel_frame_is_present() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/hud/panel-frame.png");
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!("missing shipped HUD panel asset {}: {error}", path.display())
    });
    assert!(
        metadata.len() > 512,
        "shipped HUD panel asset is unexpectedly empty: {}",
        path.display()
    );
}

#[test]
fn new_world_and_confirm_boot_session_and_reach_playing() {
    let mut app = shell_smoke_app();

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    assert_eq!(current_app_state(&app), AppState::WorldSetup);

    dispatch_menu(&mut app, MainMenuCommand::ConfirmWorldSetup);
    // No LiveStreamScene → WorldGen→Playing is drained within flush_state.
    assert_eq!(current_app_state(&app), AppState::Playing);
    assert!(
        app.world().resource::<OutcomeSessionGate>().session_active,
        "ConfirmWorldSetup should begin player session"
    );
}

#[test]
fn cancel_world_setup_returns_to_main_menu() {
    let mut app = shell_smoke_app();

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    assert_eq!(current_app_state(&app), AppState::WorldSetup);

    dispatch_menu(&mut app, MainMenuCommand::CancelWorldSetup);
    assert_eq!(current_app_state(&app), AppState::MainMenu);
}

#[test]
fn exit_to_main_menu_ends_player_session() {
    let mut app = shell_smoke_app();

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    dispatch_menu(&mut app, MainMenuCommand::ConfirmWorldSetup);
    assert!(app.world().resource::<OutcomeSessionGate>().session_active);

    dispatch_menu(&mut app, MainMenuCommand::ExitToMainMenu);
    assert_eq!(current_app_state(&app), AppState::MainMenu);
    assert!(
        !app.world().resource::<OutcomeSessionGate>().session_active,
        "ExitToMainMenu should end player session"
    );
}

#[test]
fn open_settings_opens_game_settings_panel() {
    let mut app = shell_smoke_app();
    app.insert_resource(GameSettings::default());

    dispatch_menu(&mut app, MainMenuCommand::OpenSettings);

    let settings = app.world().resource::<GameSettings>();
    assert!(settings.open);
    assert_eq!(settings.active_tab, SettingsTab::Graphics);
}

#[test]
fn advance_worldgen_waits_for_boot_timer_with_empty_live_scene() {
    let mut app = shell_smoke_app();
    app.insert_resource(LiveStreamScene::default());

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    dispatch_menu(&mut app, MainMenuCommand::ConfirmWorldSetup);
    assert_eq!(current_app_state(&app), AppState::WorldGen);

    advance_time(&mut app, 0.5);
    assert_eq!(
        current_app_state(&app),
        AppState::WorldGen,
        "empty live scene should wait for boot timer"
    );

    advance_time(&mut app, 2.0);
    assert_eq!(current_app_state(&app), AppState::Playing);
}

#[test]
fn sync_app_state_with_game_mode_maps_pause_overlay() {
    let mut app = shell_smoke_app();

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    dispatch_menu(&mut app, MainMenuCommand::ConfirmWorldSetup);
    assert_eq!(current_app_state(&app), AppState::Playing);

    *app.world_mut().resource_mut::<GameUiMode>() = GameUiMode::Paused;
    flush_state(&mut app);
    assert_eq!(current_app_state(&app), AppState::Paused);

    *app.world_mut().resource_mut::<GameUiMode>() = GameUiMode::Playing;
    flush_state(&mut app);
    assert_eq!(current_app_state(&app), AppState::Playing);
}

#[test]
fn begin_and_end_player_session_reset_gate_and_overlay() {
    let mut gate = OutcomeSessionGate::default();
    let mut overlay = OutcomeOverlayState {
        outcome: Some(civ_bevy_ref::OutcomeHudData {
            tag: "defeat".to_string(),
            reason: "stale".to_string(),
            tick: 1,
            progress: None,
        }),
        dismissed: true,
    };

    begin_player_session(&mut gate, &mut overlay);
    assert!(gate.session_active);
    assert!(overlay.outcome.is_none());
    assert!(!overlay.dismissed);
    assert!(gate.first_poll_tick.is_none());

    end_player_session(&mut gate, &mut overlay);
    assert!(!gate.session_active);
    assert!(overlay.outcome.is_none());
    assert!(!overlay.dismissed);
}
