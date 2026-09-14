use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

#[cfg(feature = "egui")]
use crate::settings_ui::{
    GameSettings, KeyBinding, ACTION_CAMERA_LOWER, ACTION_CAMERA_MOVE_BACKWARD,
    ACTION_CAMERA_MOVE_FORWARD, ACTION_CAMERA_MOVE_LEFT, ACTION_CAMERA_MOVE_RIGHT,
    ACTION_CAMERA_ORBIT_LEFT, ACTION_CAMERA_ORBIT_RIGHT, ACTION_CAMERA_RAISE, ACTION_CAMERA_RESET,
    ACTION_CAMERA_ROTATE,
};

const PAN_SPEED: f32 = 90.0;
const YAW_SPEED: f32 = 1.5;
const SCROLL_DISTANCE_PER_LINE: f32 = 15.0;
const MIN_ORBIT_DISTANCE: f32 = 4.0;
const MAX_ORBIT_DISTANCE: f32 = 800.0;
const MIN_PITCH: f32 = -1.5;
const MAX_PITCH: f32 = 0.6;

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
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings: Option<Res<GameSettings>>,
    mut rig: ResMut<CameraRig>,
) {
    let dt = time.delta_secs();
    let mut move_dir = Vec3::ZERO;
    let forward_flat = Vec3::new(rig.yaw.sin(), 0.0, rig.yaw.cos());
    // Matches requirements_bdd: right = (-forward.z, 0, forward.x)
    let right_flat = Vec3::new(-forward_flat.z, 0.0, forward_flat.x);

    let binding_pressed = |action: &str, fallback: KeyCode| -> bool {
        settings
            .as_ref()
            .and_then(|s| s.key_for(action))
            .unwrap_or(KeyBinding::Key(fallback))
            .is_pressed(&keys, &mouse_buttons)
    };

    if binding_pressed(ACTION_CAMERA_MOVE_FORWARD, KeyCode::KeyW) {
        move_dir += forward_flat;
    }
    if binding_pressed(ACTION_CAMERA_MOVE_BACKWARD, KeyCode::KeyS) {
        move_dir -= forward_flat;
    }
    if binding_pressed(ACTION_CAMERA_MOVE_RIGHT, KeyCode::KeyD) {
        move_dir += right_flat;
    }
    if binding_pressed(ACTION_CAMERA_MOVE_LEFT, KeyCode::KeyA) {
        move_dir -= right_flat;
    }
    if binding_pressed(ACTION_CAMERA_RAISE, KeyCode::KeyR) {
        move_dir += Vec3::Y;
    }
    if binding_pressed(ACTION_CAMERA_LOWER, KeyCode::KeyF) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * PAN_SPEED * dt;
    }

    if binding_pressed(ACTION_CAMERA_ORBIT_LEFT, KeyCode::KeyQ) {
        rig.yaw += YAW_SPEED * dt;
    }
    if binding_pressed(ACTION_CAMERA_ORBIT_RIGHT, KeyCode::KeyE) {
        rig.yaw -= YAW_SPEED * dt;
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
        rig.pitch = (rig.pitch - delta.y * 0.003).clamp(MIN_PITCH, MAX_PITCH);
    } else {
        mouse_motion.clear();
    }

    for event in mouse_wheel.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.05,
        };
        rig.distance = (rig.distance + scroll * SCROLL_DISTANCE_PER_LINE)
            .clamp(MIN_ORBIT_DISTANCE, MAX_ORBIT_DISTANCE);
    }

    // One-shot reset: snap rig back to the framed-default target, distance,
    // and neutral pitch/yaw. Triggered by the user-rebindable
    // ACTION_CAMERA_RESET (default: Home key).
    if settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_CAMERA_RESET))
        .is_some_and(|b| b.is_just_pressed(&keys, &mouse_buttons))
    {
        let defaults = CameraRig::default();
        rig.target = defaults.target;
        rig.yaw = defaults.yaw;
        rig.pitch = defaults.pitch;
        rig.distance = defaults.distance;
    }
}

#[cfg(not(feature = "egui"))]
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
    let right_flat = Vec3::new(-forward_flat.z, 0.0, forward_flat.x);

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
    if keys.pressed(KeyCode::KeyR) {
        move_dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::KeyF) {
        move_dir -= Vec3::Y;
    }
    if move_dir.length_squared() > 0.0 {
        rig.target += move_dir.normalize() * PAN_SPEED * dt;
    }

    if keys.pressed(KeyCode::KeyQ) {
        rig.yaw += YAW_SPEED * dt;
    }
    if keys.pressed(KeyCode::KeyE) {
        rig.yaw -= YAW_SPEED * dt;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
        rig.yaw -= delta.x * 0.004;
        rig.pitch = (rig.pitch - delta.y * 0.003).clamp(MIN_PITCH, MAX_PITCH);
    } else {
        mouse_motion.clear();
    }

    for event in mouse_wheel.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.05,
        };
        rig.distance = (rig.distance + scroll * SCROLL_DISTANCE_PER_LINE)
            .clamp(MIN_ORBIT_DISTANCE, MAX_ORBIT_DISTANCE);
    }

    // One-shot reset: snap rig back to framed defaults. See the egui branch
    // above for full context. The non-egui build uses Home as the fixed
    // binding because GameSettings isn't wired without egui.
    if keys.just_pressed(KeyCode::Home) {
        let defaults = CameraRig::default();
        rig.target = defaults.target;
        rig.yaw = defaults.yaw;
        rig.pitch = defaults.pitch;
        rig.distance = defaults.distance;
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

#[cfg(all(test, feature = "bevy"))]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;
    use bevy::input::mouse::MouseWheel;
    use std::time::Duration;

    fn camera_input_app() -> App {
        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(Time::<()>::default());
        app.insert_resource(CameraRig::default());
        app.add_message::<MouseMotion>();
        app.add_message::<MouseWheel>();
        #[cfg(feature = "egui")]
        app.insert_resource(crate::settings_ui::GameSettings::default());
        app.add_systems(Update, camera_input);
        app
    }

    #[test]
    fn qe_yaw_rf_raise_wasd_scroll() {
        let base = CameraRig::default();
        let dt = 1.0;
        let speed = PAN_SPEED;

        let mut app = camera_input_app();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::KeyQ);
        }
        app.update();
        assert!(
            (app.world().resource::<CameraRig>().yaw - (base.yaw + YAW_SPEED * dt)).abs() < 1e-4
        );

        let mut app = camera_input_app();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::KeyR);
        }
        app.update();
        assert!(
            (app.world().resource::<CameraRig>().target.y - (base.target.y + speed * dt)).abs()
                < 1e-3
        );

        let mut app = camera_input_app();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        {
            let mut wheel = app.world_mut().resource_mut::<Messages<MouseWheel>>();
            wheel.clear();
            wheel.write(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: 2.0,
                window: Entity::from_raw_u32(0).unwrap(),
            });
        }
        app.update();
        assert_eq!(app.world().resource::<CameraRig>().distance, 200.0);
    }

    /// Reset (Home key) restores the rig to CameraRig::default() in one shot.
    /// The player pans around freely, hits Home, and the camera snaps back to
    /// the framed-default target / yaw / pitch / distance.
    #[test]
    fn home_resets_rig_to_default() {
        let defaults = CameraRig::default();
        let mut app = camera_input_app();
        // Seed the rig with non-default values (player has panned, orbited,
        // and zoomed).
        {
            let mut rig = app.world_mut().resource_mut::<CameraRig>();
            rig.target = Vec3::new(200.0, 80.0, -150.0);
            rig.yaw = 1.2;
            rig.pitch = -0.2;
            rig.distance = 480.0;
        }
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::Home);
        }
        app.update();
        let rig = app.world().resource::<CameraRig>();
        assert_eq!(rig.target, defaults.target);
        assert_eq!(rig.yaw, defaults.yaw);
        assert_eq!(rig.pitch, defaults.pitch);
        assert_eq!(rig.distance, defaults.distance);
    }

    /// Pressing a non-reset key (e.g. W) without pressing Home does not
    /// trigger the reset. This guards against accidental widening of the
    /// reset trigger condition in future edits.
    #[test]
    fn non_reset_key_press_does_not_trigger_reset() {
        let mut app = camera_input_app();
        let initial = {
            let mut rig = app.world_mut().resource_mut::<CameraRig>();
            rig.target = Vec3::new(75.0, 25.0, 40.0);
            rig.yaw = 0.3;
            rig.distance = 250.0;
            rig.target
        };
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::KeyW);
        }
        app.update();
        let rig = app.world().resource::<CameraRig>();
        // Target should be unchanged (or moved by WASD panning, but not reset
        // to defaults). Without dt advancement the WASD pan won't move it,
        // so the target stays put.
        assert_eq!(
            rig.target.x, initial.x,
            "W press alone must not trigger camera reset"
        );
        assert_eq!(rig.yaw, 0.3, "yaw must not change from a W press alone");
        assert_eq!(
            rig.distance, 250.0,
            "distance must not change from a W press alone"
        );
    }
}
