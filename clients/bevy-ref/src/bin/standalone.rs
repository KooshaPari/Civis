<<<<<<< HEAD
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use civ_bevy_ref::{gpu_features::GpuFeaturesPlugin, native_backend::native_render_plugin};
use std::f32::consts::{PI, TAU};

const GRID: usize = 256;
const WORLD_SIZE: f32 = 256.0;
const HEIGHT_SCALE: f32 = 120.0;
const DAY_LENGTH_SECONDS: f32 = 10.0 * 60.0;
const CAMERA_START: Vec3 = Vec3::new(60.0, 80.0, 60.0);
const CAMERA_TARGET_START: Vec3 = Vec3::new(128.0, 30.0, 128.0);
const STAR_COUNT: usize = 240;
const STAR_SHELL_RADIUS: f32 = 1_500.0;

#[derive(Resource, Clone, Copy)]
struct CameraRig {
    target: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
=======
#![cfg(feature = "bevy")]

mod terrain;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::log::LogPlugin;
use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::ui::IsDefaultUiCamera;
use civ_agents::{spawn_civilian_at, Civilian as AgentCivilian, Position3d};
use civ_bevy_ref::{agent_scale_multiplier, CameraTarget};
use civ_engine::JobType;
use civ_engine::Simulation;
use civ_voxel::FIXED_SCALE;
use terrain::{Biome, Terrain, SIZE};

const TITLE: &str = "Civis 3D — Standalone";
const TERRAIN_WORLD_SIZE: f32 = 256.0;
const TERRAIN_SCALE_XZ: f32 = TERRAIN_WORLD_SIZE / (SIZE as f32 - 1.0);
const TERRAIN_HEIGHT_SCALE: f32 = 28.0;
const WATER_LEVEL: f32 = 0.38;
const ORBIT_DRAG_SENSITIVITY: f32 = 0.005;
const ORBIT_SCROLL_SENSITIVITY: f32 = 2.0;
const MIN_ORBIT_ELEVATION: f32 = 0.08;
const MIN_ORBIT_DISTANCE: f32 = 20.0;
const MAX_ORBIT_DISTANCE: f32 = 500.0;
const CIVILIAN_POOL: usize = 256;
const SIM_TICK_RATE_HZ: f32 = 10.0;

#[derive(Resource, Debug, Clone, Copy)]
struct OrbitCamera {
    centre: [f32; 3],
    azimuth: f32,
    elevation: f32,
    distance: f32,
}

impl OrbitCamera {
    fn from_target(target: CameraTarget) -> Self {
>>>>>>> origin/main
        Self {
            target: CAMERA_TARGET_START,
            yaw: -0.12,
            pitch: -0.72,
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct DayNightCycle {
    time_of_day: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self { time_of_day: 0.0 }
    }
}

<<<<<<< HEAD
#[derive(Component)]
struct SunLight;
=======
#[derive(Resource)]
struct StandaloneSim {
    sim: Simulation,
    terrain: Terrain,
    paused: bool,
}

#[derive(Resource)]
struct SimTickTimer(Timer);

#[derive(Resource, Default)]
struct TerrainVisuals {
    water: Option<Entity>,
    trees: Vec<Entity>,
}

#[derive(Resource, Default)]
struct CivilianVisuals {
    pool: Vec<Entity>,
    materials: Vec<(JobType, Handle<StandardMaterial>)>,
}

#[derive(Resource, Default)]
struct UiState {
    text: Option<Entity>,
}
>>>>>>> origin/main

#[derive(Component)]
struct MoonLight;

#[derive(Component)]
struct StarField;

fn main() {
    App::new()
<<<<<<< HEAD
        .insert_resource(ClearColor(Color::srgba(0.54, 0.74, 0.92, 1.0)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(DayNightCycle::default())
        .insert_resource(CameraRig::default())
        .add_plugins(DefaultPlugins.set(native_render_plugin()))
        .add_plugins(GpuFeaturesPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_input, update_camera, advance_day_night_cycle, update_day_night_lighting))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(CAMERA_START).looking_at(CAMERA_TARGET_START, Vec3::Y),
    ));
=======
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "bevy_ui::layout=error".to_string(),
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
            WireframePlugin,
        ))
        .insert_resource(StandaloneSim {
            sim: Simulation::with_seed(42),
            terrain: Terrain::generate(42),
            paused: false,
        })
        .insert_resource(SimTickTimer(Timer::from_seconds(
            1.0 / SIM_TICK_RATE_HZ,
            TimerMode::Repeating,
        )))
        .insert_resource(OrbitCamera::from_target(CameraTarget {
            centre: [TERRAIN_WORLD_SIZE * 0.5, 0.0, TERRAIN_WORLD_SIZE * 0.5],
            distance: 240.0,
            azimuth_rad: std::f32::consts::FRAC_PI_4,
            elevation_rad: 0.8,
        }))
        .insert_resource(TerrainVisuals::default())
        .insert_resource(CivilianVisuals::default())
        .insert_resource(UiState::default())
        .add_systems(Startup, setup_all)
        .add_systems(
            Update,
            (
                orbit_camera_input,
                orbit_camera_transform,
                input_controls,
                tick_simulation,
                update_civilian_meshes,
                update_overlay,
            ),
        )
        .run();
}

fn setup_all(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_visuals: ResMut<TerrainVisuals>,
    mut civilian_visuals: ResMut<CivilianVisuals>,
    mut ui_state: ResMut<UiState>,
    sim_state: Res<StandaloneSim>,
) {
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
        IsDefaultUiCamera,
    ));

    commands.spawn((PbrBundle {
        mesh: meshes.add(
            Sphere::new(2.0)
                .mesh()
                .ico(5)
                .expect("failed to build startup sphere"),
        ),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.12, 0.12),
            perceptual_roughness: 0.8,
            ..default()
        }),
        transform: Transform::from_xyz(128.0, 20.0, 128.0),
        ..default()
    },));

    commands.spawn((PbrBundle {
        mesh: meshes.add(
            Plane3d::default()
                .mesh()
                .size(TERRAIN_WORLD_SIZE, TERRAIN_WORLD_SIZE),
        ),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.6, 0.18),
            perceptual_roughness: 1.0,
            ..default()
        }),
        transform: Transform::from_xyz(TERRAIN_WORLD_SIZE * 0.5, 0.0, TERRAIN_WORLD_SIZE * 0.5),
        ..default()
    },));

    commands.insert_resource(ClearColor(Color::srgb(0.54, 0.74, 0.92)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.72, 0.78, 0.9),
        brightness: 1_300.0,
    });
>>>>>>> origin/main

    commands.spawn((
        SunLight,
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -PI / 4.0,
            PI / 8.0,
            0.0,
        )),
    ));

<<<<<<< HEAD
    commands.spawn((
        MoonLight,
        Visibility::Hidden,
        DirectionalLight {
            illuminance: 500.0,
            shadows_enabled: false,
            color: Color::srgb(0.35, 0.45, 0.75).into(),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            PI / 4.0,
            -PI / 8.0,
            0.0,
        )),
    ));

    commands.spawn((
        Mesh3d(meshes.add(terrain_mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..default()
        })),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 2.0 }))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.05, 0.05),
            ..default()
        })),
        Transform::from_xyz(128.0, 20.0, 128.0),
    ));

    let star_mesh = meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 1.0 }));
    let star_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE.into(),
        unlit: true,
        ..default()
    });
    commands.spawn((StarField, Visibility::Hidden)).with_children(|parent| {
        for i in 0..STAR_COUNT {
            let (theta, phi) = star_angles(i as u32);
            let dir = Vec3::new(
                theta.cos() * phi.sin(),
                phi.cos(),
                theta.sin() * phi.sin(),
            );
            parent.spawn((
                Mesh3d(star_mesh.clone()),
                MeshMaterial3d(star_material.clone()),
                Transform::from_translation(dir * STAR_SHELL_RADIUS).with_scale(Vec3::splat(0.75)),
            ));
        }
    });
}

fn advance_day_night_cycle(time: Res<Time>, mut cycle: ResMut<DayNightCycle>) {
    let delta = time.delta_secs() / DAY_LENGTH_SECONDS;
    cycle.time_of_day = (cycle.time_of_day + delta).fract();
}

fn update_day_night_lighting(
    cycle: Res<DayNightCycle>,
    mut clear_color: ResMut<ClearColor>,
    mut sun_query: Query<&mut DirectionalLight, With<SunLight>>,
    mut sun_transform_query: Query<&mut Transform, (With<SunLight>, Without<MoonLight>)>,
    mut moon_query: Query<(&mut DirectionalLight, &mut Transform, &mut Visibility), With<MoonLight>>,
    mut star_query: Query<&mut Visibility, With<StarField>>,
) {
    let t = cycle.time_of_day;
    let sun_angle = t * TAU - PI * 0.5;
    let sun_dir = Vec3::new(sun_angle.cos(), sun_angle.sin(), 0.35).normalize();
    let moon_dir = -sun_dir;
    let daylight = ((sun_dir.y + 0.15) / 1.15).clamp(0.0, 1.0);
    let sun_color = lerp_color(
        Color::srgb(1.0, 0.42, 0.18),
        Color::WHITE,
        daylight,
    );
    let dusk_color = Color::srgb(0.85, 0.16, 0.12);
    let dawn_weight = smoothstep(0.0, 0.16, t) * (1.0 - smoothstep(0.34, 0.5, t));
    let dusk_weight = smoothstep(0.5, 0.66, t) * (1.0 - smoothstep(0.84, 1.0, t));
    let night_blue = Color::srgb(0.03, 0.06, 0.16);
    let sky_day = Color::srgb(0.53, 0.76, 0.95);
    let sky_dusk = Color::srgb(0.78, 0.37, 0.23);
    let sky_dawn = Color::srgb(0.94, 0.61, 0.34);
    let sky = blend_colors(
        blend_colors(night_blue, sky_dawn, dawn_weight),
        sky_dusk,
        dusk_weight,
    );
    let sky = blend_colors(sky, sky_day, daylight);
    clear_color.0 = sky;

    if let Ok(mut sun_light) = sun_query.single_mut() {
        sun_light.color = if daylight > 0.55 {
            Color::WHITE.into()
        } else if dawn_weight > dusk_weight {
            sun_color.into()
        } else if dusk_weight > 0.0 {
            dusk_color.into()
        } else {
            Color::srgb(0.4, 0.5, 0.8).into()
        };
        sun_light.illuminance = if daylight > 0.1 { 15_000.0 * daylight.max(0.15) } else { 200.0 };
    }
    if let Ok(mut sun_transform) = sun_transform_query.single_mut() {
        *sun_transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, sun_dir));
    }

    if let Ok((mut moon_light, mut moon_transform, mut moon_visibility)) = moon_query.single_mut() {
        let is_night = daylight < 0.1;
        *moon_visibility = if is_night { Visibility::Visible } else { Visibility::Hidden };
        moon_light.color = Color::srgb(0.35, 0.45, 0.75).into();
        moon_light.illuminance = if is_night { 500.0 } else { 0.0 };
        *moon_transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, moon_dir));
    }

    if let Ok(mut stars_visibility) = star_query.single_mut() {
        *stars_visibility = if daylight < 0.1 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn camera_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut rig: ResMut<CameraRig>,
) {
    let dt = time.delta_secs();
    let mut move_dir = Vec3::ZERO;
    let forward_flat = Vec3::new(rig.yaw.sin(), 0.0, rig.yaw.cos());
    let right_flat = Vec3::new(forward_flat.z, 0.0, -forward_flat.x);

    if keys.pressed(KeyCode::KeyW) {
        move_dir += forward_flat;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir -= forward_flat;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir += right_flat;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_dir -= right_flat;
    }
    if keys.pressed(KeyCode::Space) {
        move_dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * 90.0 * dt;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
        rig.yaw -= delta.x * 0.003;
        rig.pitch = (rig.pitch - delta.y * 0.003).clamp(-1.45, -0.2);
    } else {
        mouse_motion.clear();
    }
}

fn update_camera(mut query: Query<&mut Transform, With<Camera3d>>, rig: Res<CameraRig>) {
    let distance = 170.0;
    let dir = Vec3::new(
        rig.yaw.sin() * rig.pitch.cos(),
        rig.pitch.sin(),
        rig.yaw.cos() * rig.pitch.cos(),
    );
    let eye = rig.target - dir * distance + Vec3::Y * 28.0;
    for mut transform in &mut query {
        *transform = Transform::from_translation(eye).looking_at(rig.target, Vec3::Y);
    }
}

fn terrain_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(GRID * GRID);
    let mut normals = Vec::with_capacity(GRID * GRID);
    let mut colors = Vec::with_capacity(GRID * GRID);
    let half = WORLD_SIZE * 0.5;

    for z in 0..GRID {
        for x in 0..GRID {
            let fx = x as f32 / (GRID - 1) as f32;
            let fz = z as f32 / (GRID - 1) as f32;
            let wx = fx * WORLD_SIZE;
            let wz = fz * WORLD_SIZE;
            let height = terrain_height(wx, wz);
            positions.push([wx - half, height, wz - half]);
            normals.push([0.0, 1.0, 0.0]);
            colors.push(color_for_height(height));
=======
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
}

fn spawn_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    terrain: &Terrain,
    visuals: &mut TerrainVisuals,
) {
    let h_min = terrain.heights.iter().cloned().fold(f32::MAX, f32::min);
    let h_max = terrain.heights.iter().cloned().fold(f32::MIN, f32::max);
    info!(
        "[standalone] terrain height range raw={:.3}..{:.3} world_y={:.1}..{:.1}",
        h_min,
        h_max,
        h_min * TERRAIN_HEIGHT_SCALE,
        h_max * TERRAIN_HEIGHT_SCALE
    );

    let mut positions = Vec::<[f32; 3]>::with_capacity(SIZE * SIZE);
    let mut normals = Vec::<[f32; 3]>::with_capacity(SIZE * SIZE);
    let mut colors = Vec::<[f32; 4]>::with_capacity(SIZE * SIZE);

    for z in 0..SIZE {
        for x in 0..SIZE {
            let idx = z * SIZE + x;
            let height = terrain.heights[idx];
            let normal = terrain_vertex_normal(terrain, x, z);
            positions.push([
                x as f32 * TERRAIN_SCALE_XZ,
                height * TERRAIN_HEIGHT_SCALE,
                z as f32 * TERRAIN_SCALE_XZ,
            ]);
            normals.push(normal);
            let biome = terrain.biomes[idx];
            let rgb = biome.rgb();
            colors.push([
                rgb[0] as f32 / 255.0,
                rgb[1] as f32 / 255.0,
                rgb[2] as f32 / 255.0,
                1.0,
            ]);
>>>>>>> origin/main
        }
    }

    let mut indices = Vec::with_capacity((GRID - 1) * (GRID - 1) * 6);
    for z in 0..GRID - 1 {
        for x in 0..GRID - 1 {
            let i = (z * GRID + x) as u32;
            indices.extend_from_slice(&[
                i,
                i + GRID as u32,
                i + 1,
                i + 1,
                i + GRID as u32,
                i + GRID as u32 + 1,
            ]);
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
    mesh
}

<<<<<<< HEAD
fn terrain_height(x: f32, z: f32) -> f32 {
    let nx = x / WORLD_SIZE - 0.5;
    let nz = z / WORLD_SIZE - 0.5;
    let mut h = 0.0;
    let mut amp = 1.0;
    let mut freq = 0.018;
    for _ in 0..5 {
        h += value_noise(nx * freq, nz * freq) * amp;
        freq *= 2.0;
        amp *= 0.5;
=======
fn seed_initial_civilians(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sim: &Simulation,
    visuals: &mut CivilianVisuals,
) {
    let mesh = meshes.add(Cuboid::new(0.5, 1.0, 0.5));
    for entity in 0..CIVILIAN_POOL {
        let id = commands
            .spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(0.95, 0.95, 0.95),
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
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
        let job = job_type_for_civilian_id(civilian.id);
        let handle = job_material(job, materials, &mut visuals.materials);
        if let Some(entity) = visuals.pool.get(idx).copied() {
            commands.entity(entity).insert(handle);
        }
>>>>>>> origin/main
    }
    h = h / 1.9375;
    let ridge = (1.0 - (nx.abs() * 1.55).min(1.0)) * (1.0 - (nz.abs() * 1.55).min(1.0));
    let island = 1.0 - ((nx * nx + nz * nz).sqrt() * 1.85).clamp(0.0, 1.0);
    ((h * 0.62 + ridge * 0.18 + island * 0.20) - 0.18).clamp(0.0, 1.0) * HEIGHT_SCALE
}

fn color_for_height(height: f32) -> [f32; 4] {
    let t = height / HEIGHT_SCALE;
    if t < 0.18 {
        [0.20, 0.40, 0.86, 1.0]
    } else if t < 0.24 {
        [0.86, 0.78, 0.52, 1.0]
    } else if t < 0.48 {
        [0.28, 0.58, 0.24, 1.0]
    } else if t < 0.68 {
        [0.12, 0.34, 0.12, 1.0]
    } else if t < 0.85 {
        [0.50, 0.50, 0.52, 1.0]
    } else {
        [0.97, 0.97, 0.97, 1.0]
    }
}

fn value_noise(x: f32, z: f32) -> f32 {
    let xi = x.floor();
    let zi = z.floor();
    let xf = x - xi;
    let zf = z - zi;
    let u = smooth(xf);
    let v = smooth(zf);
    let h00 = hash(xi, zi);
    let h10 = hash(xi + 1.0, zi);
    let h01 = hash(xi, zi + 1.0);
    let h11 = hash(xi + 1.0, zi + 1.0);
    let a = lerp(h00, h10, u);
    let b = lerp(h01, h11, u);
    lerp(a, b, v)
}

<<<<<<< HEAD
fn hash(x: f32, z: f32) -> f32 {
    ((x * 127.1 + z * 311.7).sin() * 43_758.547).fract().abs()
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
=======
fn update_civilian_meshes(
    mut commands: Commands,
    state: Res<StandaloneSim>,
    mut visuals: ResMut<CivilianVisuals>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
) {
    let mut binding = state.sim.world.query::<(&AgentCivilian, &Position3d)>();
    let mut civilians: Vec<_> = binding.iter().collect();
    civilians.sort_by_key(|(_, (c, _))| c.id);

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
        if let Some(&(_, (civilian, pos))) = visible {
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
            let job = job_type_for_civilian_id(civilian.id);
            if let Some((_, handle)) = visuals.materials.iter().find(|(j, _)| *j == job) {
                commands.entity(entity).insert(handle.clone());
            }
        }
    }
}

fn job_material(
    job: JobType,
    materials: &mut Assets<StandardMaterial>,
    cache: &mut Vec<(JobType, Handle<StandardMaterial>)>,
) -> Handle<StandardMaterial> {
    if let Some((_, handle)) = cache.iter().find(|(cached_job, _)| *cached_job == job) {
        return handle.clone();
    }

    let color = match job {
        JobType::Farmer => Color::srgb(0.18, 0.72, 0.22),
        JobType::Warrior => Color::srgb(0.86, 0.18, 0.18),
        JobType::Scholar => Color::srgb(0.32, 0.5, 0.92),
        JobType::Trader => Color::srgb(0.86, 0.62, 0.16),
        JobType::Priest => Color::srgb(0.72, 0.44, 0.88),
        JobType::Admin => Color::srgb(0.4, 0.4, 0.4),
        JobType::Unemployed => Color::srgb(0.82, 0.82, 0.82),
    };
    let handle = materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.65,
        ..default()
    });
    cache.push((job, handle.clone()));
    handle
}

fn job_type_for_civilian_id(id: u64) -> JobType {
    match id % 7 {
        0 => JobType::Farmer,
        1 => JobType::Warrior,
        2 => JobType::Scholar,
        3 => JobType::Trader,
        4 => JobType::Priest,
        5 => JobType::Admin,
        _ => JobType::Unemployed,
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
>>>>>>> origin/main
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn blend_colors(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        lerp(a.red, b.red, t),
        lerp(a.green, b.green, t),
        lerp(a.blue, b.blue, t),
        lerp(a.alpha, b.alpha, t),
    )
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    blend_colors(a, b, t.clamp(0.0, 1.0))
}

fn star_angles(seed: u32) -> (f32, f32) {
    let x = seed as f32 + 1.0;
    let u = hash(x * 1.13, x * 0.37);
    let v = hash(x * 0.73, x * 1.91);
    let theta = u * TAU;
    let phi = (v * 2.0 - 1.0).acos();
    (theta, phi)
}

fn terrain_vertex_normal(terrain: &Terrain, x: usize, z: usize) -> [f32; 3] {
    let height_at = |x: isize, z: isize| -> f32 {
        let x = x.clamp(0, (terrain.size - 1) as isize) as usize;
        let z = z.clamp(0, (terrain.size - 1) as isize) as usize;
        terrain.heights[z * terrain.size + x] * TERRAIN_HEIGHT_SCALE
    };

    let left = height_at(x as isize - 1, z as isize);
    let right = height_at(x as isize + 1, z as isize);
    let down = height_at(x as isize, z as isize - 1);
    let up = height_at(x as isize, z as isize + 1);

    let scale_x = 1.0 / TERRAIN_SCALE_XZ;
    let scale_z = 1.0 / TERRAIN_SCALE_XZ;
    let nx = (left - right) * scale_x;
    let ny = 2.0;
    let nz = (down - up) * scale_z;
    Vec3::new(nx, ny, nz).normalize().to_array()
}
