use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

const GRID: usize = 256;
const WORLD_SIZE: f32 = 256.0;
const HEIGHT_SCALE: f32 = 46.0;
const DAY_LENGTH_SECONDS: f32 = 10.0 * 60.0;
const CAMERA_START: Vec3 = Vec3::new(128.0, 200.0, 300.0);
const CAMERA_TARGET_START: Vec3 = Vec3::new(128.0, 0.0, 128.0);
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

#[derive(Component)]
struct SunLight;

#[derive(Component)]
struct MoonLight;

#[derive(Component)]
struct StarField;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgba(0.54, 0.74, 0.92, 1.0)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(DayNightCycle::default())
        .insert_resource(CameraRig::default())
        .add_plugins(DefaultPlugins)
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

fn hash(x: f32, z: f32) -> f32 {
    ((x * 127.1 + z * 311.7).sin() * 43_758.547).fract().abs()
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
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
