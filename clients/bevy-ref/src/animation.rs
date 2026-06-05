//! Actor rigging — glTF skeletal animation driven by emergent activity.
//!
//! Makes the CC0 character / creature models in `assets/models/*.glb`
//! (Quaternius / KayKit rigs) come **alive**: instead of static `SceneRoot`
//! meshes, each spawned agent plays a skeletal clip chosen from its current
//! motion — `Idle` when standing, `Walking_*` when moving slowly, `Running_*`
//! when moving fast — with smooth cross-fades between states.
//!
//! # How it stays decoupled from `sim_bridge` / `gltf_models`
//!
//! This module owns a NEW file and deliberately does **not** depend on the
//! private `SimCivilianMarker` in [`crate::sim_bridge`] nor edit the spawn path
//! in [`crate::gltf_models`]. It hooks purely on Bevy's own animation runtime:
//!
//! 1. When a GLTF `SceneRoot` finishes loading, Bevy inserts an
//!    [`AnimationPlayer`] somewhere inside the spawned scene hierarchy.
//! 2. [`attach_actor_animation`] binds each `AnimationPlayer` under a `SceneRoot`
//!    (retries until the parent glTF asset is loaded),
//!    walks up to the `SceneRoot` ancestor, matches that scene handle back to
//!    the owning glTF document, builds an [`AnimationGraph`] from that document's
//!    *named* clips, and attaches [`AnimationGraphHandle`] +
//!    [`AnimationTransitions`] + per-actor [`ActorAnim`] state.
//! 3. [`drive_actor_animation`] reads each actor root's world-space `Transform`
//!    delta every frame to estimate speed, maps speed → clip, and cross-fades.
//!
//! Models with **no** animations are handled gracefully: if the matched glTF
//! exposes no named clips, the player is tagged [`ActorAnim::STATIC`] and left
//! untouched (static mesh fallback — no panic, no spam).
//!
//! # Rich "alive" touches
//!
//! - **Per-agent speed jitter**: each actor gets a deterministic playback-speed
//!   multiplier (~0.85–1.15×) seeded from its entity id, so a crowd of idlers is
//!   not robotically synchronised.
//! - **Facing**: actors rotate to face their movement direction (yaw only),
//!   smoothly slerped, so they walk where they are going.
//! - (Scale / tint variety is owned by `sim_bridge`'s `FactionTint`; not
//!   duplicated here.)
//!
//! # Wiring (closer adds; do NOT edit lib.rs / standalone.rs from here)
//!
//! 1. In `clients/bevy-ref/src/lib.rs`, beside the other model-gated modules:
//!    ```ignore
//!    #[cfg(all(feature = "bevy", feature = "models"))]
//!    pub mod animation;
//!    ```
//! 2. In `clients/bevy-ref/src/bin/standalone.rs`, beside the other
//!    `.add_plugins(...)` calls (after `GltfModelsPlugin`):
//!    ```ignore
//!    #[cfg(feature = "models")]
//!    app.add_plugins(civ_bevy_ref::animation::ActorAnimationPlugin);
//!    ```
//!
//! Bevy 0.18 animation API used: [`AnimationGraph::from_clips`],
//! [`AnimationGraphHandle`], [`AnimationPlayer::play`], [`AnimationTransitions`].

use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Speed (world-units / second of root translation) below which an actor is
/// considered idle.
const IDLE_SPEED: f32 = 0.4;
/// Speed above which an actor is considered running rather than walking.
const RUN_SPEED: f32 = 14.0;
/// Cross-fade duration between animation states.
const TRANSITION: Duration = Duration::from_millis(250);
/// How aggressively the model yaw chases the movement direction (per second).
const FACING_SLERP: f32 = 8.0;

/// Coarse locomotion state derived from an actor's motion. Maps 1:1 to the clip
/// categories shared by the Quaternius / KayKit rigs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gait {
    Idle,
    Walk,
    Run,
}

impl Gait {
    /// Pick a gait from a world-space speed (units / second).
    fn from_speed(speed: f32) -> Self {
        if speed < IDLE_SPEED {
            Gait::Idle
        } else if speed < RUN_SPEED {
            Gait::Walk
        } else {
            Gait::Run
        }
    }
}

/// Resolved animation-graph node indices for one actor's gait clips. Any field
/// may be `None` when the source rig lacks that clip (e.g. a creature with only
/// `Idle`); callers fall back to the nearest available gait.
#[derive(Debug, Clone, Copy, Default)]
struct GaitNodes {
    idle: Option<AnimationNodeIndex>,
    walk: Option<AnimationNodeIndex>,
    run: Option<AnimationNodeIndex>,
}

impl GaitNodes {
    /// Resolve a gait to a concrete node, degrading gracefully: run→walk→idle,
    /// walk→idle, idle→walk. Returns `None` only when the rig has no locomotion
    /// clips at all.
    fn node_for(&self, gait: Gait) -> Option<AnimationNodeIndex> {
        match gait {
            Gait::Idle => self.idle.or(self.walk).or(self.run),
            Gait::Walk => self.walk.or(self.idle).or(self.run),
            Gait::Run => self.run.or(self.walk).or(self.idle),
        }
    }

    /// True when the rig exposes at least one usable locomotion clip.
    fn any(&self) -> bool {
        self.idle.is_some() || self.walk.is_some() || self.run.is_some()
    }
}

/// Per-actor animation state attached to the entity that owns the
/// [`AnimationPlayer`]. Persists the resolved gait nodes, the current gait, the
/// per-agent speed jitter, and the actor-root entity whose transform we sample
/// for motion.
#[derive(Component)]
struct ActorAnim {
    nodes: GaitNodes,
    current: Gait,
    /// Per-agent playback-speed multiplier so idlers are not synchronised.
    speed_jitter: f32,
    /// The `SceneRoot` ancestor whose world transform encodes the agent's
    /// motion (the `AnimationPlayer` itself is a deep child that does not move
    /// in world space relative to the root the sim drives).
    root: Entity,
    /// Last sampled world position of `root`, for per-frame speed estimation.
    last_pos: Vec3,
    /// True when the rig had no locomotion clips — left as a static mesh.
    is_static: bool,
}

impl ActorAnim {
    /// Marker value for a rig with no animations (static fallback).
    const STATIC: () = ();
}

/// Plugin: register the actor-animation systems. Add after `GltfModelsPlugin`.
pub struct ActorAnimationPlugin;

impl Plugin for ActorAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                attach_actor_animation,
                kick_start_actor_idle,
                drive_actor_animation,
            )
                .chain(),
        );
    }
}

/// Deterministic per-entity speed jitter in roughly `[0.85, 1.15]`.
fn jitter_for(entity: Entity) -> f32 {
    // Cheap integer hash of the entity bits → fraction.
    let bits = entity.to_bits();
    let h = bits.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let frac = ((h >> 40) & 0xFFFF) as f32 / 65535.0; // [0,1]
    0.85 + frac * 0.30
}

/// Walk up the `ChildOf` chain from `start` to the topmost ancestor that holds
/// a [`SceneRoot`], returning that entity and its scene handle.
fn scene_root_ancestor(
    start: Entity,
    parents: &Query<&ChildOf>,
    scene_roots: &Query<&SceneRoot>,
) -> Option<(Entity, AssetId<Scene>)> {
    let mut cur = start;
    let mut found: Option<(Entity, AssetId<Scene>)> = None;
    loop {
        if let Ok(sr) = scene_roots.get(cur) {
            found = Some((cur, sr.0.id()));
        }
        match parents.get(cur) {
            Ok(parent) => cur = parent.parent(),
            Err(_) => break,
        }
    }
    found
}

/// Bind each scene [`AnimationPlayer`] that lacks [`ActorAnim`] once its glTF
/// document is loaded (retries every frame until the asset is ready).
#[allow(clippy::too_many_arguments)]
fn attach_actor_animation(
    mut commands: Commands,
    players: Query<
        Entity,
        (
            With<AnimationPlayer>,
            Without<ActorAnim>,
            Without<AnimationGraphHandle>,
        ),
    >,
    parents: Query<&ChildOf>,
    scene_roots: Query<&SceneRoot>,
    gltfs: Res<Assets<Gltf>>,
    transforms: Query<&GlobalTransform>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for player_entity in &players {
        let Some((root_entity, scene_id)) =
            scene_root_ancestor(player_entity, &parents, &scene_roots)
        else {
            continue;
        };

        // Find the glTF document whose default scene is this SceneRoot's scene,
        // so we can read its *named* clips.
        let Some(gltf) = gltfs
            .iter()
            .map(|(_, g)| g)
            .find(|g| g.scenes.iter().any(|s| s.id() == scene_id))
        else {
            // glTF not in `Assets<Gltf>` yet (async scene load). Retry next frame.
            continue;
        };

        // Build a graph from every named clip; record the locomotion nodes.
        let mut graph = AnimationGraph::new();
        let mut named: HashMap<&str, AnimationNodeIndex> = HashMap::new();
        for (name, clip) in &gltf.named_animations {
            let node = graph.add_clip(clip.clone(), 1.0, graph.root);
            named.insert(&**name, node);
        }

        let nodes = GaitNodes {
            idle: pick(&named, &["Idle", "Unarmed_Idle", "2H_Melee_Idle", "Idle_B"]),
            walk: pick(
                &named,
                &["Walking_A", "Walking_B", "Walking_C", "Walking_D_Skeletons"],
            ),
            run: pick(&named, &["Running_A", "Running_B", "Running_C"]),
        };

        let graph_handle = graphs.add(graph);
        let last_pos = transforms
            .get(root_entity)
            .map(|t| t.translation())
            .unwrap_or(Vec3::ZERO);

        if !nodes.any() {
            // Static fallback: rig has no locomotion clips. Tag so we don't
            // reconsider, but do not attach a graph/transitions.
            commands.entity(player_entity).insert(ActorAnim {
                nodes,
                current: Gait::Idle,
                speed_jitter: 1.0,
                root: root_entity,
                last_pos,
                is_static: true,
            });
            continue;
        }

        let mut entity = commands.entity(player_entity);
        entity
            .insert(AnimationGraphHandle(graph_handle))
            .insert(AnimationTransitions::new())
            .insert(ActorAnim {
                nodes,
                current: Gait::Idle,
                speed_jitter: jitter_for(player_entity),
                root: root_entity,
                last_pos,
                is_static: false,
            });
    }
}

/// Pick the first present node from a priority list of clip names.
fn pick(named: &HashMap<&str, AnimationNodeIndex>, names: &[&str]) -> Option<AnimationNodeIndex> {
    names.iter().find_map(|n| named.get(*n).copied())
}

/// Start the idle clip the same frame we attach graph + [`ActorAnim`].
fn kick_start_actor_idle(
    mut actors: Query<
        (&ActorAnim, &mut AnimationPlayer, &mut AnimationTransitions),
        Added<ActorAnim>,
    >,
) {
    for (anim, mut player, mut transitions) in &mut actors {
        if anim.is_static {
            continue;
        }
        if let Some(node) = anim.nodes.node_for(Gait::Idle) {
            transitions
                .play(&mut player, node, Duration::ZERO)
                .set_repeat(RepeatAnimation::Forever);
        }
    }
}

/// Every frame: estimate each actor's speed from its root-transform delta, pick
/// a gait, cross-fade to its clip on state change, apply per-agent speed jitter,
/// and rotate the actor root to face its movement direction.
fn drive_actor_animation(
    time: Res<Time>,
    mut actors: Query<(
        &mut ActorAnim,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    global: Query<&GlobalTransform>,
    mut roots: Query<&mut Transform>,
) {
    let dt = time.delta_secs().max(1e-4);

    for (mut anim, mut player, mut transitions) in &mut actors {
        if anim.is_static {
            continue;
        }

        // World position + per-frame motion of the sim-driven root.
        let Ok(world) = global.get(anim.root) else {
            continue;
        };
        let pos = world.translation();
        let delta = pos - anim.last_pos;
        anim.last_pos = pos;

        // Horizontal speed (ignore vertical terrain seating).
        let horizontal = Vec3::new(delta.x, 0.0, delta.z);
        let speed = horizontal.length() / dt;
        let gait = Gait::from_speed(speed);

        // Face movement direction (yaw), smoothly, when actually moving.
        if horizontal.length_squared() > 1e-6 {
            if let Ok(mut tf) = roots.get_mut(anim.root) {
                let target_yaw = horizontal.x.atan2(horizontal.z);
                let target = Quat::from_rotation_y(target_yaw);
                let t = (FACING_SLERP * dt).min(1.0);
                tf.rotation = tf.rotation.slerp(target, t);
            }
        }

        // Cross-fade on gait change.
        if gait != anim.current {
            if let Some(node) = anim.nodes.node_for(gait) {
                transitions
                    .play(&mut player, node, TRANSITION)
                    .set_repeat(RepeatAnimation::Forever);
                anim.current = gait;
            }
        } else if player.playing_animations().next().is_none() {
            // First-time kick: nothing playing yet → start the current gait.
            if let Some(node) = anim.nodes.node_for(gait) {
                transitions
                    .play(&mut player, node, Duration::ZERO)
                    .set_repeat(RepeatAnimation::Forever);
                anim.current = gait;
            }
        }

        // Per-agent playback-speed jitter (and a gentle gait-scaled tempo) so a
        // crowd never marches in lockstep.
        let tempo = match anim.current {
            Gait::Idle => 1.0,
            Gait::Walk => 1.0 + (speed / RUN_SPEED) * 0.4,
            Gait::Run => 1.0,
        };
        for (_, active) in player.playing_animations_mut() {
            active.set_speed(anim.speed_jitter * tempo);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gait_thresholds() {
        assert_eq!(Gait::from_speed(0.0), Gait::Idle);
        assert_eq!(Gait::from_speed(IDLE_SPEED - 0.01), Gait::Idle);
        assert_eq!(Gait::from_speed(1.0), Gait::Walk);
        assert_eq!(Gait::from_speed(RUN_SPEED + 1.0), Gait::Run);
    }

    #[test]
    fn gait_degrades_to_available_clip() {
        // Rig with only idle: walk/run resolve to idle.
        let only_idle = GaitNodes {
            idle: Some(AnimationNodeIndex::new(1)),
            ..Default::default()
        };
        assert_eq!(only_idle.node_for(Gait::Walk), only_idle.idle);
        assert_eq!(only_idle.node_for(Gait::Run), only_idle.idle);
        assert!(only_idle.any());

        // Empty rig → no node, treated as static.
        let none = GaitNodes::default();
        assert!(none.node_for(Gait::Idle).is_none());
        assert!(!none.any());
    }

    #[test]
    fn jitter_is_in_range_and_deterministic() {
        // bevy 0.18 removed the infallible `Entity::from_raw`; use the checked
        // u32 constructor (any valid index works — the test only needs stable bits).
        let e = Entity::from_raw_u32(42).expect("valid raw entity index");
        let j = jitter_for(e);
        assert!((0.85..=1.15).contains(&j));
        assert_eq!(j, jitter_for(e)); // deterministic
    }
}
