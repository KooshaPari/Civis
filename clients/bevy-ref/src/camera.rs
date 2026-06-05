use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

/// Minimum / maximum orbit stand-off distance for mouse-wheel zoom.
/// Min is low enough to dolly down onto a single actor/building (actors stand
/// ~14 units tall in the 96-unit world); max still frames the whole map.
const MIN_DISTANCE: f32 = 12.0;
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

/// Handles all camera movement and orbit input.
///
/// WASD moves along yaw-projected ground-plane vectors so 'W' always goes toward
/// the look direction regardless of yaw angle:
///   forward_flat = (sin(yaw), 0, cos(yaw))
///   right_flat   = (cos(yaw), 0, -sin(yaw))  [= forward rotated 90° CW in XZ]
///
/// TELEPORT NOTE: this function does NOT read left-click or set rig.target from
/// any cursor/world-pick.  If clicking a tool button or the map teleports the
/// camera, the source is minimap.rs (owned by a separate agent) — camera.rs is
/// not the culprit.  To guard against accidental clicks leaking into camera
/// position from any future egui integration, target mutations here are driven
/// exclusively by held keys and right-drag orbit.
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
    let mut yaw_delta = 0.0;

    // Yaw-projected ground-plane axes — W/S/A/D move relative to camera facing.
    // forward_flat: direction the camera looks projected onto XZ.
    // right_flat:   90° clockwise rotation of forward_flat in XZ.
    let forward_flat = Vec3::new(rig.yaw.sin(), 0.0, rig.yaw.cos());
    let right_flat = Vec3::new(-forward_flat.z, 0.0, forward_flat.x); // negated: orbit cam looks +Z, so screen-right is -X-ish (fixes D-goes-left)

    if keys.pressed(KeyCode::KeyW) {
        move_dir += forward_flat;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        move_dir += forward_flat;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir -= forward_flat;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        move_dir -= forward_flat;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir += right_flat;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        move_dir += right_flat;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_dir -= right_flat;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        move_dir -= right_flat;
    }
    if keys.pressed(KeyCode::KeyR) {
        move_dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::KeyF) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * 90.0 * dt;
    }

    if keys.pressed(KeyCode::KeyQ) {
        yaw_delta += 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        yaw_delta -= 1.0;
    }

    // Mouse-wheel zoom adjusts the orbit stand-off distance.
    let scroll: f32 = mouse_wheel.read().map(|ev| ev.y).sum();
    if scroll != 0.0 {
        rig.distance = (rig.distance - scroll * 10.0).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    if yaw_delta != 0.0 {
        rig.yaw += yaw_delta * 1.5 * dt;
    }

    // Right-drag orbits; consume motion events when not orbiting to avoid drift.
    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;
    use bevy::input::keyboard::KeyCode;
    use bevy::input::mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel};
    use bevy::message::Messages;
    use std::time::Duration;

    fn camera_input_app() -> App {
        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(Time::default());
        app.insert_resource(CameraRig::default());
        app.add_message::<MouseMotion>();
        app.add_message::<MouseWheel>();
        app.add_systems(Update, super::camera_input);
        app
    }

    fn dispatch_camera_input(
        app: &mut App,
        dt: f32,
        keys: &[KeyCode],
        right_mouse: bool,
        mouse_motion_y: Option<f32>,
        wheel_y: Option<f32>,
    ) {
        app.world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(dt));

        {
            let mut key_input = app.world.resource_mut::<ButtonInput<KeyCode>>();
            key_input.clear();
            for key in keys {
                key_input.press(*key);
            }
        }

        {
            let mut mouse_button_input = app.world.resource_mut::<ButtonInput<MouseButton>>();
            mouse_button_input.clear();
            if right_mouse {
                mouse_button_input.press(MouseButton::Right);
            }
        }

        {
            let mut mouse_motion = app.world.resource_mut::<Messages<MouseMotion>>();
            mouse_motion.clear();
            if let Some(delta_y) = mouse_motion_y {
                mouse_motion.write(MouseMotion { delta: Vec2::new(0.0, delta_y) });
            }
        }

        {
            let mut mouse_wheel = app.world.resource_mut::<Messages<MouseWheel>>();
            mouse_wheel.clear();
            if let Some(y) = wheel_y {
                mouse_wheel.write(MouseWheel {
                    unit: MouseScrollUnit::Line,
                    x: 0.0,
                    y,
                    window: Entity::from_raw(0),
                });
            }
        }

        app.update();
    }

    fn expected_forward_flat(yaw: f32) -> Vec3 {
        Vec3::new(yaw.sin(), 0.0, yaw.cos())
    }

    fn expected_right_flat(forward_flat: Vec3) -> Vec3 {
        Vec3::new(-forward_flat.z, 0.0, forward_flat.x)
    }

    pub fn assert_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit() {
        let base = CameraRig::default();
        let dt = 1.0;
        let speed = 90.0;
        let forward_flat = expected_forward_flat(base.yaw);
        let right_flat = expected_right_flat(forward_flat);

        let mut yaw_q_app = camera_input_app();
        dispatch_camera_input(&mut yaw_q_app, dt, &[KeyCode::KeyQ], false, None, None);
        let yaw_q = yaw_q_app.world.resource::<CameraRig>().yaw;
        assert!((yaw_q - (base.yaw + 1.5 * dt)).abs() < f32::EPSILON * 100.0);

        let mut yaw_e_app = camera_input_app();
        dispatch_camera_input(&mut yaw_e_app, dt, &[KeyCode::KeyE], false, None, None);
        let yaw_e = yaw_e_app.world.resource::<CameraRig>().yaw;
        assert!((yaw_e - (base.yaw - 1.5 * dt)).abs() < f32::EPSILON * 100.0);

        let mut pitch_upper_app = camera_input_app();
        dispatch_camera_input(&mut pitch_upper_app, dt, &[], true, Some(-500.0), None);
        let upper_pitch = pitch_upper_app.world.resource::<CameraRig>().pitch;
        assert_eq!(upper_pitch, 0.6);

        let mut pitch_lower_app = camera_input_app();
        dispatch_camera_input(&mut pitch_lower_app, dt, &[], true, Some(1000.0), None);
        let lower_pitch = pitch_lower_app.world.resource::<CameraRig>().pitch;
        assert_eq!(lower_pitch, -1.5);

        let mut pan_app = camera_input_app();
        dispatch_camera_input(&mut pan_app, dt, &[KeyCode::KeyR], false, None, None);
        let pan_r_target = pan_app.world.resource::<CameraRig>().target;
        assert!((pan_r_target.y - (base.target.y + speed * dt)).abs() < f32::EPSILON * 100.0);

        let mut pan_app = camera_input_app();
        dispatch_camera_input(&mut pan_app, dt, &[KeyCode::KeyF], false, None, None);
        let pan_f_target = pan_app.world.resource::<CameraRig>().target;
        assert!((pan_f_target.y - (base.target.y - speed * dt)).abs() < f32::EPSILON * 100.0);

        let mut pan_app = camera_input_app();
        dispatch_camera_input(&mut pan_app, dt, &[KeyCode::KeyW], false, None, None);
        let pan_w_target = pan_app.world.resource::<CameraRig>().target;
        assert!(
            (pan_w_target - (base.target + forward_flat.normalize() * speed * dt)).length()
                < f32::EPSILON * 100.0
        );

        let mut pan_app = camera_input_app();
        dispatch_camera_input(&mut pan_app, dt, &[KeyCode::KeyA], false, None, None);
        let pan_a_target = pan_app.world.resource::<CameraRig>().target;
        assert!(
            (pan_a_target - (base.target - right_flat.normalize() * speed * dt)).length()
                < f32::EPSILON * 100.0
        );

        let mut pan_app = camera_input_app();
        dispatch_camera_input(&mut pan_app, dt, &[KeyCode::KeyS], false, None, None);
        let pan_s_target = pan_app.world.resource::<CameraRig>().target;
        assert!(
            (pan_s_target - (base.target - forward_flat.normalize() * speed * dt)).length()
                < f32::EPSILON * 100.0
        );

        let mut pan_app = camera_input_app();
        dispatch_camera_input(&mut pan_app, dt, &[KeyCode::KeyD], false, None, None);
        let pan_d_target = pan_app.world.resource::<CameraRig>().target;
        assert!(
            (pan_d_target - (base.target + right_flat.normalize() * speed * dt)).length()
                < f32::EPSILON * 100.0
        );

        let mut wheel_app = camera_input_app();
        dispatch_camera_input(&mut wheel_app, dt, &[], false, None, Some(2.0));
        let zoom_distance = wheel_app.world.resource::<CameraRig>().distance;
        assert_eq!(zoom_distance, 200.0);
    }

    #[test]
    fn requirement_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit() {
        assert_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit();
    }
}
