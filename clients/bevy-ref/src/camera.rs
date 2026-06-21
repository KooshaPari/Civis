use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

#[cfg(feature = "egui")]
use crate::settings_ui::{
    KeyBinding, GameSettings, ACTION_CAMERA_LOWER, ACTION_CAMERA_MOVE_BACKWARD,
    ACTION_CAMERA_MOVE_FORWARD, ACTION_CAMERA_MOVE_LEFT, ACTION_CAMERA_MOVE_RIGHT,
    ACTION_CAMERA_RAISE, ACTION_CAMERA_ROTATE,
};

#[derive(Resource, Clone, Copy)]
pub struct CameraRig {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            // Map is centred on the origin (terrain/water span roughly
            // -WORLD_SIZE/2..WORLD_SIZE/2), so frame the centre, not the old
            // corner-based (128,30,128) target.
            target: Vec3::new(0.0, 12.0, 0.0),
            yaw: -0.12,
            pitch: -0.72,
            distance: 170.0,
        }
    }
}

#[cfg(feature = "egui")]
pub fn camera_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings: Option<Res<GameSettings>>,
    mut rig: ResMut<CameraRig>,
) {
    let dt = time.delta_secs();
    let mut move_dir = Vec3::ZERO;
    let forward_flat = Vec3::new(rig.yaw.sin(), 0.0, rig.yaw.cos());
    let right_flat = Vec3::new(forward_flat.z, 0.0, -forward_flat.x);

    let movement_binding_pressed = |action: &str, fallback: KeyCode| -> bool {
        settings
            .as_ref()
            .and_then(|s| s.key_for(action))
            .unwrap_or(KeyBinding::Key(fallback))
            .is_pressed(&keys, &mouse_buttons)
    };

    if movement_binding_pressed(ACTION_CAMERA_MOVE_FORWARD, KeyCode::KeyW) {
        move_dir += forward_flat;
    }
    if movement_binding_pressed(ACTION_CAMERA_MOVE_BACKWARD, KeyCode::KeyS) {
        move_dir -= forward_flat;
    }
    if movement_binding_pressed(ACTION_CAMERA_MOVE_RIGHT, KeyCode::KeyD) {
        move_dir += right_flat;
    }
    if movement_binding_pressed(ACTION_CAMERA_MOVE_LEFT, KeyCode::KeyA) {
        move_dir -= right_flat;
    }
    if movement_binding_pressed(ACTION_CAMERA_RAISE, KeyCode::Space) {
        move_dir += Vec3::Y;
    }
    if movement_binding_pressed(ACTION_CAMERA_LOWER, KeyCode::ShiftLeft) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * 90.0 * dt;
    }

    let rotate_pressed = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_CAMERA_ROTATE))
        .unwrap_or(KeyBinding::Mouse(MouseButton::Right))
        .is_pressed(&keys, &mouse_buttons);
    if rotate_pressed {
        let delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
        rig.yaw -= delta.x * 0.003;
        rig.pitch = (rig.pitch - delta.y * 0.003).clamp(-1.45, -0.2);
    } else {
        mouse_motion.clear();
    }
}

#[cfg(not(feature = "egui"))]
pub fn camera_input(
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

    let movement_binding_pressed = |fallback: KeyCode| -> bool {
        keys.pressed(fallback)
    };

    if movement_binding_pressed(KeyCode::KeyW) {
        move_dir += forward_flat;
    }
    if movement_binding_pressed(KeyCode::KeyS) {
        move_dir -= forward_flat;
    }
    if movement_binding_pressed(KeyCode::KeyD) {
        move_dir += right_flat;
    }
    if movement_binding_pressed(KeyCode::KeyA) {
        move_dir -= right_flat;
    }
    if movement_binding_pressed(KeyCode::Space) {
        move_dir += Vec3::Y;
    }
    if movement_binding_pressed(KeyCode::ShiftLeft) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * 90.0 * dt;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
        rig.yaw -= delta.x * 0.004;
        rig.pitch = (rig.pitch - delta.y * 0.003).clamp(-1.2, -0.1);
    }
}

pub fn update_camera(
    mut query: Query<&mut Transform, (With<Camera3d>, Without<crate::minimap::MinimapCamera>)>,
    rig: Res<CameraRig>,
) {
    let dir = Vec3::new(
        rig.yaw.sin() * rig.pitch.cos(),
        rig.pitch.sin(),
        rig.yaw.cos() * rig.pitch.cos(),
    );
    let eye = rig.target - dir * rig.distance + Vec3::Y * 28.0;
    for mut transform in &mut query {
        *transform = Transform::from_translation(eye).looking_at(rig.target, Vec3::Y);
    }
}
