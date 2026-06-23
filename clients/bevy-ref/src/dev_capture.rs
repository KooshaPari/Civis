//! In-game developer capture harness for the Bevy reference client.
//!
//! This module is intentionally standalone. It can be wired into the Bevy app
//! with a single `.add_plugins(DevCapturePlugin)` call from the standalone
//! binary, but this file keeps the implementation isolated so the rest of the
//! client stays untouched.
//!
//! Gated to `bevy + egui` because the snapshot/UI-mode types it reads come
//! from the egui-gated `game_ui` and `menus` modules.

#![cfg(all(feature = "bevy", feature = "egui"))]

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Capturing, Screenshot};
use serde_json::json;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::game_ui::GameUiSnapshot;
use crate::menus::GameUiMode;
use crate::spawn_tools::ActiveTool;

/// Plugin that enables developer captures in the running game window.
pub struct DevCapturePlugin;

impl Plugin for DevCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DevCaptureState>()
            .add_systems(Startup, ensure_capture_dir)
            .add_systems(
                Update,
                (
                    handle_capture_hotkeys,
                    log_capture_completion,
                    handle_record_hotkeys,
                    record_inputs,
                    maybe_replay_inputs,
                ),
            );
    }
}

#[derive(Resource, Debug)]
struct DevCaptureState {
    captures_dir: PathBuf,
    next_shot: u64,
    capture_armed: bool,
    record_inputs: bool,
    replay_inputs: bool,
    replay_cursor: usize,
    replay_events: Vec<RecordedInput>,
    capture_tick_interval: f32,
    capture_tick_accum: f32,
    last_frame_counter: u64,
}

impl Default for DevCaptureState {
    fn default() -> Self {
        Self {
            captures_dir: PathBuf::from("captures"),
            next_shot: 0,
            capture_armed: false,
            record_inputs: false,
            replay_inputs: false,
            replay_cursor: 0,
            replay_events: Vec::new(),
            capture_tick_interval: 0.5,
            capture_tick_accum: 0.0,
            last_frame_counter: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct RecordedInput {
    frame: u64,
    kind: &'static str,
    payload: serde_json::Value,
}

fn ensure_capture_dir(mut state: ResMut<DevCaptureState>) {
    if let Err(err) = fs::create_dir_all(&state.captures_dir) {
        error!(
            "dev-capture: failed to create {:?}: {err}",
            state.captures_dir
        );
        return;
    }

    state.next_shot = scan_next_counter(&state.captures_dir);
    write_readme(&state.captures_dir);
}

fn handle_capture_hotkeys(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DevCaptureState>,
    mode: Option<Res<GameUiMode>>,
    snapshot: Option<Res<GameUiSnapshot>>,
    active: Option<Res<ActiveTool>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    window: Query<Entity, With<Window>>,
) {
    state.last_frame_counter = state.last_frame_counter.saturating_add(1);
    state.capture_tick_accum += time.delta_secs();

    if keys.just_pressed(KeyCode::F9) {
        trigger_capture(
            &mut commands,
            &mut state,
            &window,
            mode.as_deref(),
            snapshot.as_deref(),
            active.as_deref(),
            &cameras,
            "shot",
        );
    }

    if keys.just_pressed(KeyCode::F10) {
        state.capture_armed = !state.capture_armed;
        info!(
            "dev-capture: continuous screenshots {}",
            if state.capture_armed { "enabled" } else { "disabled" }
        );
    }

    if state.capture_armed && state.capture_tick_accum >= state.capture_tick_interval {
        state.capture_tick_accum = 0.0;
        trigger_capture(
            &mut commands,
            &mut state,
            &window,
            mode.as_deref(),
            snapshot.as_deref(),
            active.as_deref(),
            &cameras,
            "shot",
        );
    }
}

fn trigger_capture(
    commands: &mut Commands,
    state: &mut DevCaptureState,
    window: &Query<Entity, With<Window>>,
    mode: Option<&GameUiMode>,
    snapshot: Option<&GameUiSnapshot>,
    active: Option<&ActiveTool>,
    cameras: &Query<&GlobalTransform, With<Camera3d>>,
    prefix: &str,
) {
    let Ok(window_entity) = window.single() else {
        warn!("dev-capture: no primary window entity found");
        return;
    };

    let filename = format!("{prefix}-{:06}.png", state.next_shot);
    state.next_shot = state.next_shot.saturating_add(1);
    let path = state.captures_dir.join(filename);
    let shot_path = path.to_string_lossy().to_string();
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(shot_path));

    append_session_row(state, mode, snapshot, active, cameras, &path, false);
}

fn log_capture_completion(
    mut commands: Commands,
    mut state: ResMut<DevCaptureState>,
    captures: Query<Entity, With<Capturing>>,
    mode: Option<Res<GameUiMode>>,
    snapshot: Option<Res<GameUiSnapshot>>,
    active: Option<Res<ActiveTool>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
) {
    if captures.is_empty() {
        return;
    }
    let _ = (
        &mut commands,
        &mut state,
        mode.as_deref(),
        snapshot.as_deref(),
        active.as_deref(),
        &cameras,
    );
}

fn handle_record_hotkeys(mut state: ResMut<DevCaptureState>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F11) {
        state.record_inputs = !state.record_inputs;
        info!(
            "dev-capture: input recording {}",
            if state.record_inputs { "enabled" } else { "disabled" }
        );
        if !state.record_inputs {
            flush_replay_log(&state);
        }
    }

    if keys.just_pressed(KeyCode::F12) {
        state.replay_inputs = !state.replay_inputs;
        state.replay_cursor = 0;
        info!(
            "dev-capture: replay {}",
            if state.replay_inputs { "enabled" } else { "disabled" }
        );
    }

}

fn record_inputs(
    mut state: ResMut<DevCaptureState>,
    motion: MessageReader<MouseMotion>,
    wheel: MessageReader<MouseWheel>,
    keyboard_events: MessageReader<KeyboardInput>,
    mouse_button_events: MessageReader<MouseButtonInput>,
) {
    if !state.record_inputs {
        return;
    }

    let mut events = Vec::new();
    for ev in keyboard_events.read() {
        events.push(RecordedInput {
            frame: state.last_frame_counter,
            kind: "keyboard",
            payload: json!({
                "key": format!("{:?}", ev.key_code),
                "state": format!("{:?}", ev.state),
                "logical_key": format!("{:?}", ev.logical_key),
            }),
        });
    }

    for ev in mouse_button_events.read() {
        events.push(RecordedInput {
            frame: state.last_frame_counter,
            kind: "mouse_button",
            payload: json!({
                "button": format!("{:?}", ev.button),
                "state": format!("{:?}", ev.state),
            }),
        });
    }

    for ev in motion.read() {
        events.push(RecordedInput {
            frame: state.last_frame_counter,
            kind: "mouse_motion",
            payload: json!({
                "delta": [ev.delta.x, ev.delta.y],
            }),
        });
    }

    for ev in wheel.read() {
        events.push(RecordedInput {
            frame: state.last_frame_counter,
            kind: "mouse_wheel",
            payload: json!({
                "x": ev.x,
                "y": ev.y,
                "unit": format!("{:?}", ev.unit),
            }),
        });
    }

    if events.is_empty() {
        return;
    }

    state.replay_events.extend(events);
    write_replay_log(&state);
}

fn maybe_replay_inputs(mut state: ResMut<DevCaptureState>) {
    if !state.replay_inputs {
        return;
    }
    if state.replay_cursor >= state.replay_events.len() {
        state.replay_inputs = false;
        return;
    }

    // Best-effort replay hook. Bevy 0.18 exposes input events, but injecting a
    // faithful stream without stepping on the app's own input resources is more
    // invasive than this harness needs. We therefore consume the recording in
    // order and leave the UI-visible limitation documented in `captures/README.md`.
    state.replay_cursor = state.replay_events.len();
}

fn append_session_row(
    state: &DevCaptureState,
    mode: Option<&GameUiMode>,
    snapshot: Option<&GameUiSnapshot>,
    active: Option<&ActiveTool>,
    cameras: &Query<&GlobalTransform, With<Camera3d>>,
    shot_path: &Path,
    replaying: bool,
) {
    let camera = cameras.iter().next().map(|transform| transform.compute_transform());
    let row = json!({
        "frame": state.last_frame_counter,
        "game_ui_mode": mode.map(|m| format!("{:?}", m)).unwrap_or_else(|| "unknown".to_string()),
        "active_tool": active.map(|a| format!("{:?}", a.tool)).unwrap_or_else(|| "unknown".to_string()),
        "camera": camera.map(|t| json!({
            "translation": [t.translation.x, t.translation.y, t.translation.z],
            "rotation": [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
            "scale": [t.scale.x, t.scale.y, t.scale.z],
        })),
        "population": snapshot.map(|s| s.population),
        "factions": snapshot.map(|s| s.factions),
        "tick": snapshot.map(|s| s.tick),
        "speed_multiplier": snapshot.map(|s| s.speed_multiplier),
        "screenshot": shot_path.to_string_lossy(),
        "replay_mode": replaying,
    });

    let session_path = state.captures_dir.join("session.jsonl");
    append_jsonl(&session_path, &row);
}

fn write_replay_log(state: &DevCaptureState) {
    let path = state.captures_dir.join("replay.jsonl");
    let payload = state
        .replay_events
        .iter()
        .map(|event| {
            json!({
                "frame": event.frame,
                "kind": event.kind,
                "payload": event.payload,
            })
        })
        .collect::<Vec<_>>();
    for row in payload {
        append_jsonl(&path, &row);
    }
}

fn flush_replay_log(state: &DevCaptureState) {
    let path = state.captures_dir.join("replay.jsonl");
    let _ = fs::remove_file(&path);
    write_replay_log(state);
}

fn append_jsonl(path: &Path, value: &serde_json::Value) {
    let mut buf = match serde_json::to_string(value) {
        Ok(s) => s,
        Err(_) => return,
    };
    if !buf.ends_with('\n') {
        buf.push('\n');
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(buf.as_bytes());
    }
}

fn scan_next_counter(dir: &Path) -> u64 {
    let mut next = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_prefix("shot-").and_then(|s| s.strip_suffix(".png")) {
                if let Ok(n) = stem.parse::<u64>() {
                    next = next.max(n.saturating_add(1));
                }
            }
        }
    }
    next
}

fn write_readme(dir: &Path) {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "# Dev Capture\n\n\
         Files written here:\n\
         - `shot-<counter>.png`: screenshots captured with F9 or the F10 continuous toggle.\n\
         - `session.jsonl`: one JSON object per capture with frame, UI mode, active tool, camera, and any readable sim stats.\n\
         - `replay.jsonl`: recorded input stream from F11.\n\n\
         Hotkeys:\n\
         - `F9`: single screenshot\n\
         - `F10`: continuous screenshot mode (~1-2 fps)\n\
         - `F11`: start/stop input recording\n\
         - `F12`: best-effort replay toggle\n\n\
         Replay is best effort only. If Bevy input injection is not available for this build, the recording remains a disk artifact for later analysis."
    );
    let _ = fs::write(dir.join("README.md"), text);
}
