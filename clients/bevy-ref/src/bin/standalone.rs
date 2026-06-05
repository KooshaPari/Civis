//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy_water::WaterPlugin;
use civ_bevy_ref::{
    atmosphere::{animate_water, setup_atmosphere, update_lighting, DayNightCycle},
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    live_attach::LiveAttachPlugin,
    native_backend::native_render_plugin,
    resolve_attach_mode_from_env,
    terrain::terrain_mesh,
    AttachMode,
};

fn main() {
    // Persist any panic to `civ-panic.log` in the working dir so a crash is
    // captured even when launched from a shortcut with no console (per the
    // "fail loudly, never silently" stance). Chains to the default hook.
    {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let loc = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "<unknown>".to_string());
            let msg = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_string());
            use std::io::Write as _;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("civ-panic.log")
            {
                let _ = f.write_all(format!("PANIC at {loc}: {msg}\n").as_bytes());
            }
            default_hook(info);
        }));
    }

    // Build-identity banner: print on EVERY launch so it is provable which build
    // is actually running (not trusting binary mtime). `CIVIS_GIT_HASH` /
    // `CIVIS_BUILD_TIME` are injected at compile time by the build/deploy script
    // when available; they fall back to "dev"/"unknown" for a plain `cargo run`.
    eprintln!(
        "[civis] build v{} git={} built={} feat=voxel,models,egui",
        env!("CARGO_PKG_VERSION"),
        option_env!("CIVIS_GIT_HASH").unwrap_or("dev"),
        option_env!("CIVIS_BUILD_TIME").unwrap_or("unknown"),
    );

    let attach_mode = resolve_attach_mode_from_env();
    let window_title = match attach_mode {
        AttachMode::Standalone => "Civis — Bevy standalone".to_string(),
        AttachMode::Server => "Civis — Bevy standalone (live attach)".to_string(),
    };

    let mut app = App::new();
    app.insert_resource(DayNightCycle::default())
        .insert_resource(CameraRig::default())
        .insert_resource(attach_mode)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: window_title,
                        ..default()
                    }),
                    ..default()
                })
                .set(native_render_plugin()),
        )
        .add_plugins(GpuFeaturesPlugin)
        // Civis app/window icon (graphite + neon voxel-world glyph). Sets the
        // embedded icon on the primary winit window at startup.
        .add_plugins(civ_bevy_ref::window_icon::WindowIconPlugin)
        .add_plugins(civ_bevy_ref::sim_bridge::SimBridgePlugin)
        .add_plugins(civ_bevy_ref::skybox::SkyboxPlugin)
        .add_plugins(civ_bevy_ref::post_fx::PostFxPlugin)
        .add_plugins(civ_bevy_ref::game_ui::GameUiPlugin)
        .add_plugins(civ_bevy_ref::tech_tree_ui::TechTreeUiPlugin)
        .add_plugins(civ_bevy_ref::event_feed::EventFeedPlugin)
        .add_plugins(civ_bevy_ref::menus::MenusPlugin)
        .add_plugins(civ_bevy_ref::spawn_tools::SpawnToolsPlugin)
        .add_plugins(civ_bevy_ref::minimap::MinimapPlugin)
        .init_resource::<civ_bevy_ref::game_ui::GameUiSnapshot>()
        .add_systems(Startup, setup_atmosphere)
        .add_systems(
            Startup,
            (
                setup_camera,
                // Heightmap terrain + decorations are the default-playable
                // fallback. Under the `voxel` feature VoxelSimPlugin owns the
                // world instead (see `heightmap_enabled`).
                setup_sandbox_terrain
                    .run_if(in_sandbox_attach_mode)
                    .run_if(heightmap_enabled),
                spawn_decorations
                    .run_if(in_sandbox_attach_mode)
                    .run_if(heightmap_enabled),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (camera_input, update_camera, animate_water, update_lighting),
        );

    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::diplomacy_ui::DiplomacyUiPlugin);

    // 2D procedural/SVG alternate map view (M key + far-zoom auto-engage).
    // Normally suppressed under autoshot (the far-zoom auto-engage could flip the
    // headless 3D capture into map mode), but MUST be present when we explicitly
    // ask to capture the map via CIVIS_MAP_OPEN=1 — otherwise the plugin (and its
    // draw_map_view system) never exists and the map can't open in the frame.
    #[cfg(feature = "egui")]
    if std::env::var("CIVIS_AUTOSHOT").is_err()
        || std::env::var("CIVIS_MAP_OPEN").as_deref() == Ok("1")
    {
        app.add_plugins(civ_bevy_ref::map2d::Map2dPlugin);
    }

    // Perception layer: CS2-style info-view overlays (Tab) + click-to-inspect.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::info_views::InfoViewsPlugin);
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::inspect::InspectPlugin);

    // Event-feed / toast notifications.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::notifications::NotificationsPlugin);

    // Terrain sculpting brush (raise/lower/flatten); bevy-only, no egui needed.
    #[cfg(feature = "bevy")]
    app.add_plugins(civ_bevy_ref::terraform_brush::TerraformBrushPlugin);

    // God-game disaster actions (meteor/flood/quake/storm/wildfire) that mutate
    // the voxel world; bevy-only, gated systems handle egui/voxel internally.
    #[cfg(feature = "bevy")]
    app.add_plugins(civ_bevy_ref::disaster_tools::DisasterToolsPlugin);

    // Material brush palette + voxel paint (Powder-Toy-style); bevy+egui.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::material_brush_ui::MaterialBrushPlugin);

    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::game_laws::GameLawsPlugin);

    // Settings / options panel (RON-persisted); bevy+egui.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::settings_ui::SettingsPlugin);
    // Ambient + SFX audio (feature-gated).
    #[cfg(feature = "audio")]
    app.add_plugins(civ_bevy_ref::audio::CivisAudioPlugin);
    // GPU particle VFX for events (feature-gated).
    #[cfg(feature = "vfx")]
    app.add_plugins(civ_bevy_ref::vfx::VfxPlugin);
    // Real-time RT global illumination via bevy_solari (feature-gated).
    #[cfg(feature = "gi")]
    app.add_plugins(civ_bevy_ref::lighting_gi::SolariGiPlugin);

    // P-VM-3: real volumetric voxel material world (replaces the heightmap).
    // `voxel_stream` takes precedence: when enabled, the camera-driven streaming
    // sandbox owns the world instead of the bounded dense `VoxelSimPlugin`.
    #[cfg(all(feature = "voxel", not(feature = "voxel_stream")))]
    app.add_plugins(civ_bevy_ref::voxel_sim::VoxelSimPlugin);

    #[cfg(feature = "voxel")]
    app.add_plugins(WaterPlugin);

    // FR-CIV-VOXEL-020: camera-driven chunk streaming over the 20mi voxel world.
    #[cfg(feature = "voxel_stream")]
    app.add_plugins(civ_bevy_ref::voxel_stream::VoxelStreamPlugin);

    // CC0 GLTF models: populate GameModels so sim_bridge swaps capsule/cuboid
    // primitives for real Knight/house scenes (per-asset primitive fallback).
    #[cfg(feature = "models")]
    app.add_plugins(civ_bevy_ref::gltf_models::GltfModelsPlugin);

    // Actor rigging: drive glTF skeletal animation from emergent motion so
    // agents idle / walk / run + face their heading instead of sliding statically.
    #[cfg(feature = "models")]
    app.add_plugins(civ_bevy_ref::animation::ActorAnimationPlugin);

    if attach_mode == AttachMode::Server {
        app.add_plugins(LiveAttachPlugin);
    }

    // Headless "boot into gameplay" hook: when CIVIS_AUTOSTART=1 is set, drive
    // the app straight into the live world instead of the main menu. Worldgen is
    // deferred until the game enters Loading/Playing
    // (`voxel_sim::build_world_on_play`, keyed off the per-world randomized
    // `WorldSetupParams.seed`); forcing `Playing` triggers that exact path so a
    // headless screenshot captures a freshly generated world, not the title card.
    if std::env::var("CIVIS_AUTOSTART").as_deref() == Ok("1") {
        app.add_systems(Startup, |mut mode: ResMut<civ_bevy_ref::menus::GameUiMode>| {
            *mode = civ_bevy_ref::menus::GameUiMode::Playing;
        });
    }

    // Headless brush-mutation proof: when CIVIS_PAINT_DEMO=1, stamp a bright
    // lava blob into the live voxel grid after the world has generated. This
    // exercises the exact paint path the fix repairs — grid.set marks the chunk
    // dirty, step_and_remesh re-meshes it — so the autoshot screenshot shows a
    // painted blob on the terrain, proving the mutation→remesh→render chain.
    #[cfg(feature = "voxel")]
    if std::env::var("CIVIS_PAINT_DEMO").as_deref() == Ok("1") {
        app.insert_resource(PaintDemo {
            timer: Timer::from_seconds(6.0, TimerMode::Once),
            done: false,
        })
        .add_systems(Update, paint_demo_blob);
    }

    // Headless verification hook: when CIVIS_AUTOSHOT=<path> is set, capture one
    // screenshot after a short warm-up (so chunk meshes / GLTF scenes are loaded
    // and the camera has framed the world) and then exit. This lets a debug
    // worker confirm voxel terrain visibility by pixels without manual F9.
    if let Ok(path) = std::env::var("CIVIS_AUTOSHOT") {
        let warmup_seconds = std::env::var("CIVIS_AUTOSHOT_WARMUP")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(4.0);
        info!("[autoshot] armed: path set, warmup={warmup_seconds:.1}s (wall-clock)");
        app.insert_resource(AutoShot {
            path,
            armed_at: std::time::Instant::now(),
            warmup: std::time::Duration::from_secs_f32(warmup_seconds),
            taken_at: None,
            ticking_logged: false,
        })
        .add_systems(Update, auto_screenshot);
    }

    // Machine-level scene + sim dump (CIVIS_DUMP=<path>) — writes authoritative
    // scene-graph + sim-counter JSON after warmup, then exits. Lets a verifier
    // find floating actors / dissolved terrain / T-poses / wrong counters from
    // data, never from pixels.
    #[cfg(feature = "voxel")]
    let _ = civ_bevy_ref::scene_dump::arm_from_env(&mut app);

    app.run();
}

/// Headless brush-mutation demo state (see `CIVIS_PAINT_DEMO`).
#[cfg(feature = "voxel")]
#[derive(Resource)]
struct PaintDemo {
    timer: Timer,
    done: bool,
}

/// After warm-up, stamp a bright blob into the live grid centred over the world
/// so the next screenshot proves the paint→dirty→remesh→render path works.
#[cfg(feature = "voxel")]
fn paint_demo_blob(
    time: Res<Time>,
    mut demo: ResMut<PaintDemo>,
    mut sim: ResMut<civ_bevy_ref::voxel_sim::VoxelSimState>,
) {
    use civ_voxel::material::MaterialRegistry;
    if demo.done || !demo.timer.tick(time.delta()).just_finished() {
        return;
    }
    if sim.grid.cells.is_empty() {
        demo.timer = Timer::from_seconds(0.5, TimerMode::Once);
        return;
    }
    let dims = sim.grid.dims;
    let mat = MaterialRegistry::standard()
        .by_name("Lava")
        .map_or(civ_voxel::MaterialId(6), |m| m.id);
    // Sit the blob on the surface near the world centre.
    let (cx, cz) = (dims[0] / 2, dims[2] / 2);
    let cy = surface_demo(&sim.grid, cx, cz);
    let r = 10i64;
    for dz in -r..=r {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy + dz * dz > r * r {
                    continue;
                }
                let (x, y, z) = (cx as i64 + dx, cy as i64 + dy, cz as i64 + dz);
                if x < 0 || y < 0 || z < 0 {
                    continue;
                }
                sim.grid.set(x as usize, y as usize, z as usize, mat);
            }
        }
    }
    info!("[paint-demo] stamped lava blob r={r} at ({cx},{cy},{cz})");
    demo.done = true;
}

/// Surface scan for the paint demo (lowest air resting on solid).
#[cfg(feature = "voxel")]
fn surface_demo(grid: &civ_voxel::fluid_ca::CaGrid, x: usize, z: usize) -> usize {
    use civ_voxel::material::AIR;
    for y in (0..grid.dims[1]).rev() {
        if grid.get(x, y, z) != AIR {
            return (y + 4).min(grid.dims[1] - 1);
        }
    }
    grid.dims[1] / 2
}

#[derive(Resource)]
struct AutoShot {
    path: String,
    /// Wall-clock instant the resource was armed (app start). Warmup + exit gating
    /// use WALL-CLOCK (`Instant`), not `Time::delta()`, because Bevy clamps delta to
    /// `max_delta` (~0.25s/frame): during the multi-second synchronous world mesh,
    /// ticking a Timer with the clamped delta accumulates game-time far slower than
    /// real time, so a 12s warmup never fired within any reasonable wall-clock wait.
    armed_at: std::time::Instant,
    /// Warmup duration before the screenshot is requested.
    warmup: std::time::Duration,
    /// When the screenshot was requested (post-capture flush is gated off this).
    taken_at: Option<std::time::Instant>,
    /// One-shot "system scheduled + running" heartbeat trace.
    ticking_logged: bool,
}

fn auto_screenshot(
    mut commands: Commands,
    mut shot: ResMut<AutoShot>,
    mut exit: MessageWriter<AppExit>,
) {
    // Post-capture: exit once the png is actually on disk (deterministic, no race
    // with the GPU readback flush), with a wall-clock safety cap so a failed save
    // can't hang the process forever.
    if let Some(taken_at) = shot.taken_at {
        let png_exists = std::path::Path::new(&shot.path).exists();
        let waited = taken_at.elapsed();
        if png_exists {
            info!("[autoshot] saved {} -> exiting", shot.path);
            exit.write(AppExit::Success);
        } else if waited >= std::time::Duration::from_secs(15) {
            warn!("[autoshot] exit safety cap: no png after {:.0}s, exiting", waited.as_secs_f32());
            exit.write(AppExit::Success);
        }
        return;
    }
    // Heartbeat once, proving the system is scheduled + Update is running it.
    if !shot.ticking_logged && shot.armed_at.elapsed().as_secs_f32() >= 1.0 {
        info!("[autoshot] ticking (system scheduled + running, wall-clock)");
        shot.ticking_logged = true;
    }
    // Warmup is WALL-CLOCK: fire once enough real time has passed regardless of how
    // many frames the synchronous world build stalled.
    if shot.armed_at.elapsed() >= shot.warmup {
        let path = shot.path.clone();
        info!(
            "[autoshot] warmup done at {:.1}s -> requesting screenshot",
            shot.armed_at.elapsed().as_secs_f32()
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        info!("[autoshot] requested -> {path}");
        shot.taken_at = Some(std::time::Instant::now());
    }
}

fn in_sandbox_attach_mode(mode: Res<AttachMode>) -> bool {
    *mode == AttachMode::Standalone
}

/// True when the heightmap terrain should spawn — i.e. the `voxel` feature is
/// OFF. Under `voxel`, `VoxelSimPlugin` owns the world instead.
fn heightmap_enabled() -> bool {
    cfg!(not(feature = "voxel"))
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Far plane raised well past the sky dome (radius 4000) and star shell
        // (1500) so the skybox/stars are not frustum-culled at any zoom.
        Projection::Perspective(PerspectiveProjection {
            far: 10_000.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 90.0, 150.0).looking_at(Vec3::new(0.0, 12.0, 0.0), Vec3::Y),
    ));
}

fn setup_sandbox_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain = terrain_mesh();
    commands.spawn((
        Mesh3d(meshes.add(terrain)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.62, 0.38),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // NOTE: the single water body is owned by `atmosphere::setup_atmosphere`
    // (a `WaterSurface` plane at `WATER_LEVEL`). Spawning another here would
    // double up the water; intentionally not spawned in this setup.
}
