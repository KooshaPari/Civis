//! Bevy-backed frame capture for the verification harness.
//!
//! Only compiled when the `bevy` feature is enabled (see `Cargo.toml`).
//!
//! ## Pipeline
//!
//! 1. Spin up a headless Bevy 0.18 `App` with a primary `Window` and a single
//!    `Camera3d`.
//! 2. Spawn a minimal deterministic scene (cube + point light) so the
//!    captured frame is not all-clear / all-black.
//! 3. Wait [`VerifyOptions::settle_frames`] frames so the GPU has a chance to
//!    warm up.
//! 4. Trigger `Screenshot::primary_window()` with a `save_to_disk` observer
//!    pointing at the requested output path; the Bevy renderer writes the
//!    PNG to disk asynchronously. The app then runs for a few more frames
//!    until the `Capturing` query is empty (see Bevy `examples/window/screenshot.rs`).
//! 5. Return [`VerifyResult`] with the on-disk path, frame count, and a
//!    machine-readable status the MCP shim can compare across runs.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Capturing, Screenshot};
use serde::{Deserialize, Serialize};

/// Resolved options for a single `civis-verify` run.
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    /// Where the PNG frame is written (`CIV_VERIFY_OUT_DIR` or default
    /// `target/verify-frames/frame-0.png`).
    pub output_path: PathBuf,
    /// Number of frames to let the renderer settle before the screenshot is
    /// requested. Default 60 (≈1 s at 60 Hz).
    pub settle_frames: u32,
    /// Window width in logical pixels.
    pub width: u32,
    /// Window height in logical pixels.
    pub height: u32,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("target/verify-frames/frame-0.png"),
            settle_frames: 60,
            width: 640,
            height: 360,
        }
    }
}

/// Result emitted by `civis-verify` (and the `civis_verify` MCP tool).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyResult {
    /// Absolute or relative path the screenshot was written to.
    pub output_path: PathBuf,
    /// Settle frames actually consumed before the screenshot was triggered.
    pub settle_frames: u32,
    /// Width/height the window was opened at.
    pub width: u32,
    pub height: u32,
    /// Library version of the harness at the time the bin ran.
    pub harness_version: &'static str,
    /// Bevy 0.18 schema version this binary was compiled against.
    pub bevy_version: &'static str,
}

/// Compile-time Bevy version string. We can't import `bevy::VERSION` from a
/// `const` context, so we mirror its literal here.
pub const BEVY_VERSION: &str = "0.18";

/// Run the verify pipeline using Bevy 0.18. Returns the path the PNG was
/// written to alongside a [`VerifyResult`] for the bin to print as JSON.
///
/// Must be called from a non-async context (Bevy `App::run` blocks).
pub fn run_verify(options: VerifyOptions) -> Result<VerifyResult, VerifyError> {
    if let Some(parent) = options.output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| VerifyError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "civis-verify".to_string(),
                    resolution: (options.width as f32, options.height as f32).into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                // Bevy 0.18 keeps `screenshot` behind the default render plugin
                // already; no extra feature gate needed at the plugin level.
                ..default()
            }),
    )
    .add_systems(Startup, setup_scene)
    .add_systems(
        Update,
        (tick_settle, trigger_screenshot, watch_capturing).chain(),
    )
    .insert_resource(SettleCounter {
        remaining: options.settle_frames,
        triggered: false,
    })
    .insert_resource(ScreenshotRequest {
        path: options.output_path.clone(),
    });

    // Cap the run at settle_frames + 240 post-screenshot frames so a stuck
    // `Capturing` query never wedges the agent.
    let max_frames = options.settle_frames + 240;
    let mut elapsed: u32 = 0;
    while !app.world().resource::<SettleCounter>().triggered
        || !app
            .world()
            .query::<&Capturing>()
            .iter(&app.world())
            .next()
            .is_none()
    {
        app.update();
        elapsed = elapsed.saturating_add(1);
        if elapsed > max_frames {
            return Err(VerifyError::Timeout { elapsed });
        }
    }

    Ok(VerifyResult {
        output_path: options.output_path,
        settle_frames: options.settle_frames,
        width: options.width,
        height: options.height,
        harness_version: crate::HARNESS_VERSION,
        bevy_version: BEVY_VERSION,
    })
}

#[derive(Resource)]
struct SettleCounter {
    remaining: u32,
    triggered: bool,
}

#[derive(Resource)]
struct ScreenshotRequest {
    path: PathBuf,
}

fn tick_settle(mut settle: ResMut<SettleCounter>) {
    if settle.remaining > 0 {
        settle.remaining -= 1;
    }
}

fn trigger_screenshot(
    mut commands: Commands,
    mut settle: ResMut<SettleCounter>,
    request: Res<ScreenshotRequest>,
) {
    if settle.triggered || settle.remaining > 0 {
        return;
    }
    settle.triggered = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(request.path.clone()));
}

fn watch_capturing(
    mut commands: Commands,
    capturing: Query<Entity, With<Capturing>>,
) {
    for entity in &capturing {
        commands.entity(entity).despawn();
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Errors the verify pipeline can surface to the operator.
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    /// Failed to create the screenshot output directory.
    #[error("io error preparing {path}: {source}")]
    Io {
        /// Directory that could not be created.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The screenshot was never written within the configured frame budget.
    #[error("verify pipeline timed out after {elapsed} frames without releasing the screenshot")]
    Timeout {
        /// Frames actually consumed.
        elapsed: u32,
    },
}
