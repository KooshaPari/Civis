#![cfg(feature = "bevy")]

mod terrain;

use std::collections::HashMap;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
#[cfg(feature = "models")]
use civ_bevy_ref::animation::ActorAnimationPlugin;
#[cfg(feature = "models")]
use civ_bevy_ref::gltf_models::GltfModelsPlugin;
#[cfg(feature = "gi")]
use civ_bevy_ref::lighting_gi::SolariGiPlugin;
#[cfg(feature = "voxel")]
use civ_bevy_ref::ocean::OceanPlugin;
#[cfg(feature = "egui")]
use civ_bevy_ref::settings_ui::{AntiAliasing, GameSettings, SettingsPlugin};
use civ_bevy_ref::{
    atmosphere::{animate_water, setup_atmosphere, update_lighting, DayNightCycle, WaterSurface},
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    live_attach::LiveAttachPlugin,
    native_backend::native_render_plugin,
    post_fx::PostFxSettings,
    resolve_attach_mode_from_env,
    terrain::{terrain_mesh, WORLD_SIZE},
    AttachMode,
};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Loading,
    InGame,
    Paused,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.to_string(),
                    ..default()
                })
                .set(native_render_plugin()),
        )
        .add_plugins(GpuFeaturesPlugin)
        // Frame diagnostics: emit `FrameTime` + `SystemInformation` once per
        // second at INFO so the 90s frame-budget profile has a measurable
        // signal. See `docs/audits/frame-budget-baseline-2026-06-10.md`.
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // Civis app/window icon (graphite + neon voxel-world glyph). Sets the
        // embedded icon on the primary winit window at startup.
        .add_plugins(civ_bevy_ref::window_icon::WindowIconPlugin)
        .add_plugins(civ_bevy_ref::sim_bridge::SimBridgePlugin)
        .add_plugins(civ_bevy_ref::post_fx::PostFxPlugin)
        .add_plugins(civ_bevy_ref::game_ui::GameUiPlugin)
        .add_plugins(civ_bevy_ref::tech_tree_ui::TechTreeUiPlugin)
        .add_plugins(civ_bevy_ref::diplomacy_ui::DiplomacyUiPlugin)
        .add_plugins(civ_bevy_ref::event_feed::EventFeedPlugin)
        .add_plugins(civ_bevy_ref::menus::MenusPlugin)
        .add_plugins(civ_bevy_ref::spawn_tools::SpawnToolsPlugin)
        .add_plugins(civ_bevy_ref::minimap::MinimapPlugin)
        .init_resource::<civ_bevy_ref::game_ui::GameUiSnapshot>()
        .init_state::<AppState>()
        .add_systems(Startup, setup_atmosphere)
        .add_systems(OnEnter(AppState::MainMenu), enter_loading_from_main_menu)
        .add_systems(
            OnEnter(AppState::Loading),
            (
                setup_camera,
                setup_sandbox_terrain.run_if(in_sandbox_attach_mode),
                spawn_decorations.run_if(in_sandbox_attach_mode),
                finish_loading,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                camera_input,
                update_camera,
                animate_water,
                update_lighting,
                escape_to_pause,
            )
                .run_if(in_state(AppState::InGame)),
        );
    #[cfg(feature = "egui")]
    {
        app.add_plugins(SettingsPlugin)
            .add_systems(Startup, sync_post_fx_from_settings)
            .add_systems(
                Update,
                sync_post_fx_from_settings.run_if(resource_changed::<GameSettings>),
            );
    }

    #[cfg(feature = "models")]
    {
        app.add_plugins((GltfModelsPlugin, ActorAnimationPlugin));
    }

    #[cfg(feature = "gi")]
    {
        app.add_plugins(SolariGiPlugin);
    }

    if attach_mode == AttachMode::Standalone {
        #[cfg(feature = "pbr-textures")]
        app.add_plugins(civ_bevy_ref::materials::BiomeMaterialsPlugin);
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

    // OceanPlugin — wraps bevy_water::WaterPlugin.  Gated on `voxel` (which
    // pulls bevy_water).  Two modes:
    //
    // • voxel + voxel_stream  → full mode (OceanPlugin::default): WaterPlugin
    //   + WaterSettings + wave-plane spawn.  VoxelStreamPlugin does NOT spawn
    //   a water plane, so OceanPlugin owns the surface here.
    //
    // • voxel only (VoxelSimPlugin active) → thin mode (water_plugin_only):
    //   registers WaterPlugin shader infrastructure but skips the spawn because
    //   VoxelSimPlugin::spawn_bevy_water_plane already owns the wave surface.
    #[cfg(all(feature = "voxel", feature = "voxel_stream"))]
    app.add_plugins(OceanPlugin::default());
    #[cfg(all(feature = "voxel", not(feature = "voxel_stream")))]
    app.add_plugins(OceanPlugin::water_plugin_only());

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

    app.run();
}

#[cfg(feature = "egui")]
fn sync_post_fx_from_settings(settings: Res<GameSettings>, mut post_fx: ResMut<PostFxSettings>) {
    let graphics = &settings.graphics;
    post_fx.aces = graphics.anti_aliasing != AntiAliasing::Off;
    post_fx.bloom = graphics.bloom;
    post_fx.ssao = graphics.ambient_occlusion;
    post_fx.taa = graphics.anti_aliasing == AntiAliasing::TAA;
}

fn enter_loading_from_main_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Loading);
}

fn finish_loading(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}

fn in_sandbox_attach_mode(mode: Res<AttachMode>) -> bool {
    *mode == AttachMode::Standalone
}

fn escape_to_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if *state.get() == AppState::InGame && keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Paused);
    }
}

fn setup_camera(mut commands: Commands) {
    let camera_target = CameraTarget {
        centre: [TERRAIN_WORLD_SIZE * 0.5, 0.0, TERRAIN_WORLD_SIZE * 0.5],
        distance: 240.0,
        azimuth_rad: std::f32::consts::FRAC_PI_4,
        elevation_rad: 0.8,
    };
    let eye = camera_target.orbit_position();
    let centre = Vec3::from_array(camera_target.centre);

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(eye[0], eye[1], eye[2]).looking_at(centre, Vec3::Y),
    ));
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_visuals: ResMut<TerrainVisuals>,
    mut civilian_visuals: ResMut<CivilianVisuals>,
    mut ui_state: ResMut<UiState>,
    sim_state: Res<StandaloneSim>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let camera_target = CameraTarget {
        centre: [TERRAIN_WORLD_SIZE * 0.5, 0.0, TERRAIN_WORLD_SIZE * 0.5],
        distance: 240.0,
        azimuth_rad: std::f32::consts::FRAC_PI_4,
        elevation_rad: 0.8,
    };
    let eye = camera_target.orbit_position();
    let centre = Vec3::from_array(camera_target.centre);

    commands.insert_resource(ClearColor(Color::srgb(0.54, 0.74, 0.92)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.72, 0.78, 0.9),
        brightness: 1_300.0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(eye[0] + 80.0, eye[1] + 120.0, eye[2] + 40.0)
            .looking_at(centre, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(eye[0], eye[1], eye[2]).looking_at(centre, Vec3::Y),
    ));

    spawn_terrain(
        &mut commands,
        &mut meshes,
        &mut materials,
        &sim_state.terrain,
        &mut terrain_visuals,
    );
    seed_initial_civilians(
        &mut commands,
        &mut meshes,
        &mut materials,
        &sim_state.sim,
        &mut civilian_visuals,
    );

    let overlay = commands
        .spawn((
            TextBundle::from_section(
                "loading...",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.96, 0.97, 0.99),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                ..default()
            }),
            OverlayText,
        ))
        .id();
    ui_state.text = Some(overlay);

    next_state.set(AppState::Playing);
}

fn spawn_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    terrain: &Terrain,
    visuals: &mut TerrainVisuals,
) {
    let mut positions = Vec::<[f32; 3]>::with_capacity(SIZE * SIZE);
    let mut normals = Vec::<[f32; 3]>::with_capacity(SIZE * SIZE);
    let mut colors = Vec::<[f32; 4]>::with_capacity(SIZE * SIZE);

    for z in 0..SIZE {
        for x in 0..SIZE {
            let idx = z * SIZE + x;
            let height = terrain.heights[idx];
            positions.push([
                x as f32 * TERRAIN_SCALE_XZ,
                height * TERRAIN_HEIGHT_SCALE,
                z as f32 * TERRAIN_SCALE_XZ,
            ]);
            normals.push([0.0, 1.0, 0.0]);
            let biome = terrain.biomes[idx];
            let rgb = biome.rgb();
            colors.push([
                rgb[0] as f32 / 255.0,
                rgb[1] as f32 / 255.0,
                rgb[2] as f32 / 255.0,
                1.0,
            ]);
        }
    }

    let mut indices = Vec::<u32>::new();
    for z in 0..(SIZE - 1) {
        for x in 0..(SIZE - 1) {
            let i0 = (z * SIZE + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + SIZE as u32;
            let i3 = i2 + 1;
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            }),
            ..default()
        },
        TerrainMarker,
        Wireframe,
        WireframeColor {
            color: Color::srgba(0.2, 0.3, 0.35, 0.2),
        },
    ));

    let water_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(TERRAIN_WORLD_SIZE, TERRAIN_WORLD_SIZE),
    );
    let water = commands
        .spawn((
            PbrBundle {
                mesh: water_mesh,
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba(0.15, 0.3, 0.55, 0.45),
                    alpha_mode: AlphaMode::Blend,
                    unlit: false,
                    ..default()
                }),
                transform: Transform::from_xyz(
                    TERRAIN_WORLD_SIZE * 0.5,
                    WATER_LEVEL * TERRAIN_HEIGHT_SCALE,
                    TERRAIN_WORLD_SIZE * 0.5,
                ),
                ..default()
            },
            TerrainMarker,
        ))
        .id();
    visuals.water = Some(water);

    let tree_mesh = meshes.add(Cuboid::new(1.2, 2.4, 1.2));
    for z in 0..SIZE {
        for x in 0..SIZE {
            let idx = z * SIZE + x;
            if terrain.biomes[idx] != Biome::Forest {
                continue;
            }
            if (x + z) % 11 != 0 {
                continue;
            }
            let world_x = x as f32 * TERRAIN_SCALE_XZ;
            let world_z = z as f32 * TERRAIN_SCALE_XZ;
            let y = terrain.heights[idx] * TERRAIN_HEIGHT_SCALE + 1.2;
            let tree = commands
                .spawn((
                    PbrBundle {
                        mesh: tree_mesh.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::srgb(0.2, 0.34, 0.18),
                            perceptual_roughness: 1.0,
                            ..default()
                        }),
                        transform: Transform::from_xyz(world_x, y, world_z),
                        ..default()
                    },
                    TreeMarker,
                ))
                .id();
            visuals.trees.push(tree);
        }
    }
}

fn seed_initial_civilians(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sim: &Simulation,
    visuals: &mut CivilianVisuals,
) {
    let mesh = meshes.add(Cuboid::new(0.8, 1.6, 0.8));
    for entity in 0..CIVILIAN_POOL {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.95, 0.95),
            perceptual_roughness: 0.8,
            ..default()
        });
        let id = commands
            .spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                CivilianMarker { pool_index: entity },
            ))
            .id();
        visuals.pool.push(id);
    }

    for (idx, (_, civilian)) in sim
        .world
        .query::<&AgentCivilian>()
        .iter()
        .take(CIVILIAN_POOL)
        .enumerate()
    {
        let material_key = civilian.faction;
        let rgb = agent_color_from_id(civilian.id);
        let handle = visuals
            .materials
            .entry(material_key)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    perceptual_roughness: 0.6,
                    ..default()
                })
            })
            .clone();
        if let Some(entity) = visuals.pool.get(idx).copied() {
            commands.entity(entity).insert(handle);
        }
    }
}

fn tick_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimTickTimer>,
    mut state: ResMut<StandaloneSim>,
) {
    if state.paused {
        return;
    }
    if timer.0.tick(time.delta()).just_finished() {
        state.sim.tick();
    }
}

fn input_controls(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut orbit: ResMut<OrbitCamera>,
    mut state: ResMut<StandaloneSim>,
) {
    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }

    for key in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ] {
        if keys.just_pressed(key) {
            let idx = match key {
                KeyCode::Digit1 => 0,
                KeyCode::Digit2 => 1,
                KeyCode::Digit3 => 2,
                _ => 3,
            };
            if let Some(faction) = state.sim.spectator_view().factions.get(idx) {
                orbit.centre = [
                    faction.capital[0] * TERRAIN_WORLD_SIZE,
                    0.0,
                    faction.capital[1] * TERRAIN_WORLD_SIZE,
                ];
            }
        }
    }

    if buttons.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.get_single() {
            if let Some(cursor) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_q.get_single() {
                    if let Some(ray) = camera.viewport_to_world(camera_transform, cursor) {
                        let dir = ray.direction;
                        let origin = ray.origin;
                        if dir.y.abs() > f32::EPSILON {
                            let t = (0.0 - origin.y) / dir.y;
                            if t > 0.0 {
                                let hit = origin + dir * t;
                                let x = (hit.x / TERRAIN_WORLD_SIZE).clamp(0.0, 0.99);
                                let y = (hit.z / TERRAIN_WORLD_SIZE).clamp(0.0, 0.99);
                                let mut rng = state.sim.rng_mut().clone();
                                let next_id = 10_000_000 + state.sim.state.tick;
                                let _ = spawn_civilian_at(
                                    &mut state.sim.world,
                                    next_id,
                                    0,
                                    x,
                                    y,
                                    &mut rng,
                                );
                                *state.sim.rng_mut() = rng;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn orbit_camera_input(
    mut orbit: ResMut<OrbitCamera>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut wheel: EventReader<MouseWheel>,
) {
    if buttons.pressed(MouseButton::Right) {
        let delta = motion.read().fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
        orbit.azimuth -= delta.x * ORBIT_DRAG_SENSITIVITY;
        orbit.elevation = (orbit.elevation + delta.y * ORBIT_DRAG_SENSITIVITY)
            .clamp(MIN_ORBIT_ELEVATION, std::f32::consts::FRAC_PI_2 - 0.05);
    } else {
        motion.clear();
    }

    for ev in wheel.read() {
        let amount = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y * 0.02,
        };
        orbit.adjust_distance(-amount * ORBIT_SCROLL_SENSITIVITY);
    }
}

fn orbit_camera_transform(
    orbit: Res<OrbitCamera>,
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera_q.get_single_mut() else {
        return;
    };
    let eye = orbit.as_target().orbit_position();
    *transform = Transform::from_xyz(eye[0], eye[1], eye[2])
        .looking_at(Vec3::from_array(orbit.centre), Vec3::Y);
}

fn update_civilian_meshes(
    mut commands: Commands,
    state: Res<StandaloneSim>,
    mut visuals: ResMut<CivilianVisuals>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut binding = state
        .sim
        .world
        .query::<(&AgentCivilian, &Position3d, &Velocity)>();
    let mut civilians: Vec<_> = binding.iter().collect();
    civilians.sort_by_key(|(_, (c, _, _))| c.id);

    let pool = visuals.pool.clone();
    for (slot, entity) in pool.into_iter().enumerate() {
        let visible = civilians.get(slot);
        if let Ok(mut vis) = visibility.get_mut(entity) {
            *vis = if visible.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if let Some((_, (civilian, pos, _vel))) = visible {
            let x = pos.coord.x as f32 / FIXED_SCALE as f32 * TERRAIN_WORLD_SIZE;
            let z = pos.coord.z as f32 / FIXED_SCALE as f32 * TERRAIN_WORLD_SIZE;
            let y = sample_height(
                &state.terrain,
                x / TERRAIN_WORLD_SIZE,
                z / TERRAIN_WORLD_SIZE,
            ) * TERRAIN_HEIGHT_SCALE
                + 1.0;
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = Vec3::new(x, y, z);
                transform.scale = Vec3::splat(agent_scale_multiplier(1.0));
            }
            let material_key = civilian.faction;
            let rgb = agent_color_from_id(civilian.id);
            let handle = if let Some(handle) = visuals.materials.get(&material_key) {
                handle.clone()
            } else {
                let handle = materials.add(StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    perceptual_roughness: 0.65,
                    ..default()
                });
                visuals.materials.insert(material_key, handle.clone());
                handle
            };
            commands.entity(entity).insert(handle);
        }
    }
}

fn update_overlay(
    state: Res<StandaloneSim>,
    ui: Res<UiState>,
    mut query: Query<&mut Text, With<OverlayText>>,
) {
    let Some(entity) = ui.text else {
        return;
    };
    let Ok(mut text) = query.get_mut(entity) else {
        return;
    };
    let climate = state.sim.climate();
    let is_day = climate.day_phase >= 0.25 && climate.day_phase < 0.75;
    let civilians = state.sim.world.query::<&AgentCivilian>().iter().count();
    text.sections[0].value = format!(
        "tick: {}\npopulation: {}\nera: {}\nday/night: {}\npaused: {}\ncivilians: {}",
        state.sim.state.tick,
        state.sim.state.population,
        state.sim.state.tick / 600,
        if is_day { "day" } else { "night" },
        state.paused,
        civilians,
    );
}

fn sample_height(terrain: &Terrain, x: f32, z: f32) -> f32 {
    let x = x.clamp(0.0, 0.999_999);
    let z = z.clamp(0.0, 0.999_999);
    let ix = (x * (terrain.size as f32 - 1.0)).round() as usize;
    let iz = (z * (terrain.size as f32 - 1.0)).round() as usize;
    terrain.heights[iz * terrain.size + ix]
}
