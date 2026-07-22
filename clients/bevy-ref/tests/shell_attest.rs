//! SHELL-ATTEST-001..004 — headless shell smoke for AppState, menu wiring, and
//! shipped shell assets/launchers.
//!
//! Exercises `consume_menu_commands`, `advance_worldgen_to_playing`, and session
//! gate helpers without opening a GPU window. Also asserts faction crest PNGs,
//! HUD chrome rasters, and `Tools/launch-civis.ps1` / `Tools/play.ps1` exist.

#![cfg(all(feature = "bevy", feature = "egui"))]

use std::time::Duration;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
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
fn shipped_hud_chrome_assets_are_present() {
    let hud_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui/hud");
    for file in [
        "panel-frame.png",
        "button.png",
        "button-hover.png",
        "chip-bg.png",
        "resource-population.png",
        "resource-clock.png",
    ] {
        let path = hud_root.join(file);
        let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
            panic!(
                "missing shipped HUD chrome asset {}: {error}",
                path.display()
            )
        });
        assert!(
            metadata.len() > 256,
            "shipped HUD chrome asset is unexpectedly empty: {}",
            path.display()
        );
    }
}

#[test]
fn shipped_hud_panel_frame_is_present() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui/hud/panel-frame.png");
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "missing shipped HUD panel asset {}: {error}",
            path.display()
        )
    });
    assert!(
        metadata.len() > 512,
        "shipped HUD panel asset is unexpectedly empty: {}",
        path.display()
    );
}

#[test]
fn shipped_faction_crest_pngs_are_present() {
    let crest_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui/faction-crests");
    let entries = std::fs::read_dir(&crest_root).unwrap_or_else(|error| {
        panic!(
            "missing faction crest directory {}: {error}",
            crest_root.display()
        )
    });
    let mut crest_pngs = 0usize;
    for entry in entries {
        let entry = entry.expect("read faction crest directory entry");
        let path = entry.path();
        let is_crest_png = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("crest-") && name.ends_with(".png"));
        if !is_crest_png {
            continue;
        }
        let metadata = entry.metadata().unwrap_or_else(|error| {
            panic!("failed to stat crest asset {}: {error}", path.display())
        });
        assert!(
            metadata.len() > 256,
            "shipped faction crest PNG is unexpectedly empty: {}",
            path.display()
        );
        crest_pngs += 1;
    }
    assert!(
        crest_pngs >= 1,
        "expected at least one crest-*.png under {}",
        crest_root.display()
    );
}

#[test]
fn shipped_launcher_scripts_are_present() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for rel in ["Tools/launch-civis.ps1", "Tools/play.ps1"] {
        let path = repo_root.join(rel);
        let metadata = std::fs::metadata(&path)
            .unwrap_or_else(|error| panic!("missing launcher script {}: {error}", path.display()));
        assert!(
            metadata.len() > 64,
            "launcher script is unexpectedly empty: {}",
            path.display()
        );
    }
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
fn confirm_world_setup_clears_stale_live_scene_before_boot() {
    let mut app = shell_smoke_app();
    let mut stale = LiveStreamScene::default();
    // Mark as "has content" without real entities — WorldGen must not skip boot.
    stale.buildings.insert(1, Entity::from_bits(1));
    app.insert_resource(stale);

    dispatch_menu(&mut app, MainMenuCommand::NewWorld);
    dispatch_menu(&mut app, MainMenuCommand::ConfirmWorldSetup);
    assert_eq!(current_app_state(&app), AppState::WorldGen);

    let scene = app.world().resource::<LiveStreamScene>();
    assert!(
        scene.buildings.is_empty(),
        "ConfirmWorldSetup must clear stale streamed buildings"
    );

    advance_time(&mut app, 0.5);
    assert_eq!(
        current_app_state(&app),
        AppState::WorldGen,
        "cleared scene should still wait for boot timer"
    );
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

#[test]
fn crash_dir_avoids_cwd_relative_crashes_when_local_data_present() {
    let dir = civ_bevy_ref::crash_dir();
    let text = dir.to_string_lossy();
    if std::env::var_os("LOCALAPPDATA").is_some() {
        assert!(
            text.contains("Civis") && text.contains("crashes"),
            "expected %LOCALAPPDATA%/Civis/crashes, got {text}"
        );
        assert!(dir.is_absolute(), "crash_dir should be absolute on Windows");
    } else {
        assert!(
            text.contains("crashes"),
            "expected a crashes directory, got {text}"
        );
    }
}
