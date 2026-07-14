//! SHELL-ATTEST-001 — headless shell smoke for AppState + menu command wiring.
//!
//! Exercises `consume_menu_commands`, `advance_worldgen_to_playing`, and session
//! gate helpers without opening a GPU window.

#![cfg(all(feature = "bevy", feature = "egui"))]

use std::time::Duration;

use bevy::app::AppExit;
use bevy::prelude::*;
use civ_bevy_ref::live_stream::LiveStreamScene;
use civ_bevy_ref::menus::{
    advance_worldgen_to_playing, consume_menu_commands, sync_app_state_with_game_mode, AppState,
    GameUiMode, MainMenuCommand, MainMenuSaves, MenuCommand, WorldGenBoot, WorldSetupParams,
};
use civ_bevy_ref::outcome_overlay::{
    begin_player_session, end_player_session, OutcomeOverlayState, OutcomeSessionGate,
};
use civ_bevy_ref::settings_ui::{GameSettings, SettingsTab};

fn shell_smoke_app() -> App {
    let mut app = App::new();
    app.init_state::<AppState>()
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
    app.update();
}

fn advance_time(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

#[test]
fn app_state_defaults_to_main_menu() {
    let app = shell_smoke_app();
    assert_eq!(current_app_state(&app), AppState::MainMenu);
}

#[test]
fn new_world_and_confirm_boot_session_and_reach_playing() {
    let mut app = shell_smoke_app();

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    assert_eq!(current_app_state(&app), AppState::WorldSetup);

    dispatch_menu(&mut app, MainMenuCommand::ConfirmWorldSetup);
    assert_eq!(current_app_state(&app), AppState::WorldGen);

    let gate = app.world().resource::<OutcomeSessionGate>();
    assert!(gate.session_active, "ConfirmWorldSetup should begin player session");

    // No LiveStreamScene → advance_worldgen_to_playing transitions immediately.
    assert_eq!(current_app_state(&app), AppState::Playing);
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
    app.update();
    assert_eq!(current_app_state(&app), AppState::Paused);

    *app.world_mut().resource_mut::<GameUiMode>() = GameUiMode::Playing;
    app.update();
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
