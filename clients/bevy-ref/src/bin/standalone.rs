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
        Self {
            centre: target.centre,
            azimuth: target.azimuth_rad,
            elevation: target.elevation_rad,
            distance: target.distance,
        }
    }

    fn as_target(&self) -> CameraTarget {
        CameraTarget {
            centre: self.centre,
            distance: self.distance,
            azimuth_rad: self.azimuth,
            elevation_rad: self.elevation,
        }
    }

    fn adjust_distance(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(MIN_ORBIT_DISTANCE, MAX_ORBIT_DISTANCE);
    }

    fn pan_centre(&mut self, right: f32, forward: f32) {
        let sin = self.azimuth.sin();
        let cos = self.azimuth.cos();
        self.centre[0] += right * cos + forward * sin;
        self.centre[2] += -right * sin + forward * cos;
    }
}

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

#[derive(Component)]
struct OverlayText;

#[derive(Component)]
struct CivilianMarker {
    pool_index: usize,
}

#[derive(Component)]
struct TreeMarker;

#[derive(Component)]
struct TerrainMarker;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "bevy_ui::layout=error,wgpu=error".to_string(),
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
        brightness: 0.8,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 3.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(eye[0] + 80.0, eye[1] + 120.0, eye[2] + 40.0)
            .looking_at(centre, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 1.5,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(eye[0] - 120.0, eye[1] + 90.0, eye[2] - 140.0)
            .looking_at(centre, Vec3::Y),
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
            if let Some((_, handle)) = visuals
                .materials
                .iter()
                .find(|(cached_job, _)| *cached_job == job)
            {
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
}

fn sample_height(terrain: &Terrain, x: f32, z: f32) -> f32 {
    let x = x.clamp(0.0, 0.999_999);
    let z = z.clamp(0.0, 0.999_999);
    let ix = (x * (terrain.size as f32 - 1.0)).round() as usize;
    let iz = (z * (terrain.size as f32 - 1.0)).round() as usize;
    terrain.heights[iz * terrain.size + ix]
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
