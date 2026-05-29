use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

/// Minimum / maximum orbit stand-off distance for mouse-wheel zoom.
const MIN_DISTANCE: f32 = 20.0;
const MAX_DISTANCE: f32 = 600.0;

#[derive(Resource, Clone, Copy)]
pub struct CameraRig {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    /// Orbit stand-off distance (world units); driven by mouse-wheel zoom.
    pub distance: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            // Map is centred on the origin (terrain/water span roughly
            // -WORLD_SIZE/2..WORLD_SIZE/2), so frame the centre, not the old
            // corner-based (128,30,128) target.
            // Frame the map centre slightly above sea level (WATER_LEVEL ≈ 64)
            // so the camera looks down onto the islands/relief instead of along
            // a flooded plane at y≈12 (which filled the frame with water).
            target: Vec3::new(0.0, 70.0, 0.0),
            yaw: -0.12,
            pitch: -0.72,
            distance: 220.0,
        }
    }
}

pub fn camera_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
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
    if keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyZ) {
        move_dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * 90.0 * dt;
    }

    // Mouse-wheel zoom adjusts the orbit stand-off distance.
    let scroll: f32 = mouse_wheel.read().map(|ev| ev.y).sum();
    if scroll != 0.0 {
        rig.distance = (rig.distance - scroll * 12.0).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion.read().fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
        rig.yaw -= delta.x * 0.003;
        rig.pitch = (rig.pitch - delta.y * 0.003).clamp(-1.5, 0.6);
    } else {
        mouse_motion.clear();
    }
}

pub fn update_camera(
    mut query: Query<&mut Transform, (With<Camera3d>, Without<crate::minimap::MinimapCamera>)>,
    rig: Res<CameraRig>,
) {
    let distance = rig.distance;
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
