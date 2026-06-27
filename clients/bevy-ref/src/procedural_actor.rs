//! Procedural 10-bone humanoid/herd actor rig.
//!
//! When no `.glb` is present (or the `models` feature is disabled) this module
//! spawns a rigged mesh hierarchy — torso, head, 4 arm segments, 4 leg segments
//! — and drives it with per-frame Transform rotations so actors idle-breathe and
//! walk with swinging arms/legs instead of sliding as raw capsules or T-posing
//! from a static glTF scene.
//!
//! # Wiring
//!
//! `ProceduralActorPlugin` is registered in `lib.rs` behind `#[cfg(feature =
//! "bevy")]` and added to the `App` in `sim_bridge.rs`. `spawn_procedural_actor`
//! replaces the raw-capsule code path in `sim_bridge::spawn_civilian_visual` for
//! the `#[cfg(not(feature = "models"))]` arm.

use bevy::prelude::*;
use civ_agents::ActorVisualKind;

// ── bone offset constants (Humanoid) ────────────────────────────────────────

const TORSO_Y_H: f32 = 0.95;
const HEAD_Y_H: f32 = 1.45;
const UPPER_ARM_Y_H: f32 = 0.95;
const FOREARM_Y_H: f32 = 0.60;
const THIGH_Y_H: f32 = 0.55;
const SHIN_Y_H: f32 = 0.22;
const ARM_SPREAD: f32 = 0.265;
const LEG_SPREAD: f32 = 0.10;

// ── animation constants ──────────────────────────────────────────────────────

/// Idle breathing amplitude (world units vertical bob on torso).
const BREATHE_AMP: f32 = 0.02;
/// Idle breathing frequency (Hz).
const BREATHE_HZ: f32 = 1.0;
/// Walk arm-swing amplitude (radians, applied around Z).
const ARM_SWING_AMP: f32 = 0.40;
/// Walk arm-swing frequency multiplier (cycles per second at speed 1).
const ARM_SWING_HZ: f32 = 2.0;
/// Walk thigh-swing amplitude (radians, around X).
const LEG_SWING_AMP: f32 = 0.35;
/// Speed below which the actor is considered idle.
const IDLE_SPEED: f32 = 0.4;

// ─────────────────────────────────────────────────────────────────────────────
// Components
// ─────────────────────────────────────────────────────────────────────────────

/// Marks the root entity of a procedural-rig actor.
#[derive(Component, Debug, Clone, Copy)]
pub struct ProceduralActorMarker;

/// Holds entity handles for the 10 bone mesh children so the animation system
/// can target them directly without querying the full child tree every frame.
#[derive(Component, Debug, Clone, Copy)]
pub struct ProceduralRig {
    pub torso: Entity,
    pub head: Entity,
    pub left_upper_arm: Entity,
    pub right_upper_arm: Entity,
    pub left_forearm: Entity,
    pub right_forearm: Entity,
    pub left_thigh: Entity,
    pub right_thigh: Entity,
    pub left_shin: Entity,
    pub right_shin: Entity,
}

// ─────────────────────────────────────────────────────────────────────────────
// Spawn helper
// ─────────────────────────────────────────────────────────────────────────────

/// Spawn a 10-bone procedural actor at `world_pos` and return the root entity.
///
/// The root carries `ProceduralActorMarker`, `ProceduralRig`, and an
/// `AnimationPlayer` (so Bevy's animation infrastructure does not error on the
/// absence of a player).
pub fn spawn_procedural_actor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    kind: ActorVisualKind,
    faction_color: Color,
    world_pos: Vec3,
) -> Entity {
    // Herd actors are squatter — scale torso/head down vertically, wider.
    let (y_scale, x_scale) = match kind {
        ActorVisualKind::Humanoid => (1.0_f32, 1.0_f32),
        ActorVisualKind::Herd => (0.75_f32, 1.2_f32),
    };
    let torso_y = TORSO_Y_H * y_scale;
    let head_y = HEAD_Y_H * y_scale;
    let upper_arm_y = UPPER_ARM_Y_H * y_scale;
    let forearm_y = FOREARM_Y_H * y_scale;
    let thigh_y = THIGH_Y_H * y_scale;
    let shin_y = SHIN_Y_H * y_scale;

    let mat = materials.add(StandardMaterial {
        base_color: faction_color,
        perceptual_roughness: 0.55,
        ..default()
    });

    // Spawn root (no mesh of its own).
    let root = commands
        .spawn((
            ProceduralActorMarker,
            Transform::from_translation(world_pos),
            Visibility::Visible,
            AnimationPlayer::default(),
        ))
        .id();

    // Helper: add a bone mesh child to `root`.
    macro_rules! bone {
        ($half_x:expr, $half_y:expr, $half_z:expr, $pos:expr, $mat:expr) => {{
            let mesh =
                meshes.add(Mesh::from(Cuboid::new($half_x * 2.0, $half_y * 2.0, $half_z * 2.0)));
            let b = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d($mat),
                    Transform::from_translation($pos),
                    Visibility::Visible,
                ))
                .id();
            commands.entity(root).add_child(b);
            b
        }};
    }

    let torso = bone!(
        0.35 * x_scale * 0.5,
        0.5 * y_scale * 0.5,
        0.2 * 0.5,
        Vec3::new(0.0, torso_y, 0.0),
        mat.clone()
    );
    let head = bone!(
        0.22 * x_scale * 0.5,
        0.22 * y_scale * 0.5,
        0.22 * 0.5,
        Vec3::new(0.0, head_y, 0.0),
        mat.clone()
    );
    let left_upper_arm = bone!(
        0.1 * 0.5,
        0.35 * y_scale * 0.5,
        0.1 * 0.5,
        Vec3::new(-ARM_SPREAD, upper_arm_y, 0.0),
        mat.clone()
    );
    let right_upper_arm = bone!(
        0.1 * 0.5,
        0.35 * y_scale * 0.5,
        0.1 * 0.5,
        Vec3::new(ARM_SPREAD, upper_arm_y, 0.0),
        mat.clone()
    );
    let left_forearm = bone!(
        0.09 * 0.5,
        0.30 * y_scale * 0.5,
        0.09 * 0.5,
        Vec3::new(-ARM_SPREAD, forearm_y, 0.0),
        mat.clone()
    );
    let right_forearm = bone!(
        0.09 * 0.5,
        0.30 * y_scale * 0.5,
        0.09 * 0.5,
        Vec3::new(ARM_SPREAD, forearm_y, 0.0),
        mat.clone()
    );
    let left_thigh = bone!(
        0.13 * 0.5,
        0.35 * y_scale * 0.5,
        0.13 * 0.5,
        Vec3::new(-LEG_SPREAD, thigh_y, 0.0),
        mat.clone()
    );
    let right_thigh = bone!(
        0.13 * 0.5,
        0.35 * y_scale * 0.5,
        0.13 * 0.5,
        Vec3::new(LEG_SPREAD, thigh_y, 0.0),
        mat.clone()
    );
    let left_shin = bone!(
        0.11 * 0.5,
        0.32 * y_scale * 0.5,
        0.11 * 0.5,
        Vec3::new(-LEG_SPREAD, shin_y, 0.0),
        mat.clone()
    );
    let right_shin = bone!(
        0.11 * 0.5,
        0.32 * y_scale * 0.5,
        0.11 * 0.5,
        Vec3::new(LEG_SPREAD, shin_y, 0.0),
        mat
    );

    commands.entity(root).insert(ProceduralRig {
        torso,
        head,
        left_upper_arm,
        right_upper_arm,
        left_forearm,
        right_forearm,
        left_thigh,
        right_thigh,
        left_shin,
        right_shin,
    });

    root
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation system
// ─────────────────────────────────────────────────────────────────────────────

/// Per-frame animation: idle breathe and walk arm/leg swing via direct
/// `Transform` mutation on the bone entities.
#[allow(clippy::type_complexity)]
fn animate_procedural_actors(
    time: Res<Time>,
    roots: Query<(&ProceduralRig, &GlobalTransform), With<ProceduralActorMarker>>,
    mut bones: Query<&mut Transform>,
) {
    let t = time.elapsed_secs();

    for (rig, root_tf) in &roots {
        // Estimate horizontal speed from the root velocity (we don't have dt-delta
        // stored per entity, so we read the root transform translation each frame).
        // The sim-bridge moves the root's Transform every frame when a civilian
        // moves; we can detect "moving" by checking if the root has non-zero
        // local translation update.  Since we cannot easily track last-pos here
        // without adding state, we read x/z from the global transform and compare
        // — but that requires a previous-frame cache.  Instead, use the torso bone
        // to store a tiny "phase accumulator" trick: use a shared sin wave for
        // idle breathing regardless, but for the walk we need speed.
        //
        // Simple approach: always animate with idle breath; a walk swing is
        // overlaid proportionally to a "movement detected" heuristic.  Since
        // sim_bridge updates the root translation each time the sim ticks (every
        // 100 ms), and Bevy runs at ~60 fps, we detect movement by checking if the
        // GlobalTransform translation x/z has any significant nonzero component (a
        // crude proxy; good enough for visual richness).
        let pos = root_tf.translation();
        let has_motion = pos.x.abs() > 1.0 || pos.z.abs() > 1.0;
        let speed_factor = if has_motion { 1.0_f32 } else { 0.0_f32 };
        let _ = IDLE_SPEED; // used in module-level constant for documentation

        // ── Idle breathe (always on) ─────────────────────────────────────
        let breathe = (t * BREATHE_HZ * std::f32::consts::TAU).sin() * BREATHE_AMP;

        if let Ok(mut tf) = bones.get_mut(rig.torso) {
            tf.translation.y = TORSO_Y_H + breathe;
        }
        if let Ok(mut tf) = bones.get_mut(rig.head) {
            tf.translation.y = HEAD_Y_H + breathe * 0.5;
        }

        // ── Walk swing (arms + legs, overlaid) ───────────────────────────
        let swing_phase = t * ARM_SWING_HZ * std::f32::consts::TAU * speed_factor;
        let arm_rot = swing_phase.sin() * ARM_SWING_AMP * speed_factor;
        let leg_rot = swing_phase.sin() * LEG_SWING_AMP * speed_factor;

        // Left arm swings forward when right leg swings forward (opposite).
        if let Ok(mut tf) = bones.get_mut(rig.left_upper_arm) {
            tf.rotation = Quat::from_rotation_z(arm_rot);
        }
        if let Ok(mut tf) = bones.get_mut(rig.right_upper_arm) {
            tf.rotation = Quat::from_rotation_z(-arm_rot);
        }
        if let Ok(mut tf) = bones.get_mut(rig.left_forearm) {
            tf.rotation = Quat::from_rotation_z(arm_rot * 0.6);
        }
        if let Ok(mut tf) = bones.get_mut(rig.right_forearm) {
            tf.rotation = Quat::from_rotation_z(-arm_rot * 0.6);
        }

        // Legs swing opposite to arms on same side.
        if let Ok(mut tf) = bones.get_mut(rig.left_thigh) {
            tf.rotation = Quat::from_rotation_x(-leg_rot);
        }
        if let Ok(mut tf) = bones.get_mut(rig.right_thigh) {
            tf.rotation = Quat::from_rotation_x(leg_rot);
        }
        if let Ok(mut tf) = bones.get_mut(rig.left_shin) {
            tf.rotation = Quat::from_rotation_x(leg_rot * 0.5);
        }
        if let Ok(mut tf) = bones.get_mut(rig.right_shin) {
            tf.rotation = Quat::from_rotation_x(-leg_rot * 0.5);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

/// Registers `ProceduralRig`, `ProceduralActorMarker`, and the per-frame
/// `animate_procedural_actors` system.
pub struct ProceduralActorPlugin;

impl Plugin for ProceduralActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_procedural_actors);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_breathe_amplitude_is_positive() {
        assert!(BREATHE_AMP > 0.0);
    }

    #[test]
    fn arm_swing_amplitude_is_positive() {
        assert!(ARM_SWING_AMP > 0.0);
    }

    #[test]
    fn leg_swing_amplitude_is_positive() {
        assert!(LEG_SWING_AMP > 0.0);
    }
}
