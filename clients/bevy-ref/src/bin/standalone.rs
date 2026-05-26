use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::PI;

const GRID: usize = 256;
const WORLD_SIZE: f32 = 256.0;
const HEIGHT_SCALE: f32 = 46.0;
const CAMERA_START: Vec3 = Vec3::new(128.0, 200.0, 300.0);
const CAMERA_TARGET_START: Vec3 = Vec3::new(128.0, 0.0, 128.0);

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgba(0.54, 0.74, 0.92, 1.0)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
        })
        .insert_resource(CameraRig::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_input, update_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(CAMERA_START)
            .looking_at(CAMERA_TARGET_START, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -PI / 4.0,
            PI / 8.0,
            0.0,
        )),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(terrain_mesh()),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..default()
        }),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 2.0 })),
        material: materials.add(Color::srgb(0.9, 0.05, 0.05)),
        transform: Transform::from_xyz(128.0, 20.0, 128.0),
        ..default()
    });
}

fn camera_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut rig: ResMut<CameraRig>,
) {
    let dt = time.delta_seconds();
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

fn update_camera(mut query: Query<&mut Transform, With<Camera>>, rig: Res<CameraRig>) {
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
