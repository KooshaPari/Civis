//! Viewport picking for streamed live entities (agents, buildings, graph parcels).

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

#[cfg(feature = "egui")]
use crate::settings_ui::{GameSettings, KeyBinding, ACTION_SELECT_OR_PICK};
use crate::live_stream::{LiveAgentTag, LiveBuildingTag, LiveGraphParcelTag};
use crate::minimap::{MinimapCamera, MinimapRoot};
use crate::{
    LiveEntityKind, SelectedLiveEntity, AGENT_MARKER_DEPTH, AGENT_MARKER_HEIGHT, AGENT_MARKER_WIDTH,
};

/// Optional live selection for HUD overlays and inspectors.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveSelection(pub Option<SelectedLiveEntity>);

/// Tracks left-button drag so orbit camera motion does not trigger a pick.
#[derive(Resource, Debug, Default)]
pub struct LivePickPointer {
    /// Set when the cursor moves while the left button is held.
    pub left_dragged: bool,
}

/// Wires pointer state and click-to-pick for live attach clients.
pub struct LivePickPlugin;

impl Plugin for LivePickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveSelection>()
            .init_resource::<LivePickPointer>()
            .add_systems(
                Update,
                (
                    reset_live_pick_drag_on_press,
                    mark_live_pick_drag,
                    pick_live_entity_on_release,
                )
                    .chain(),
            );
    }
}

/// Half-extents of the default agent marker mesh (before entity scale).
#[must_use]
pub const fn agent_marker_half_extents() -> [f32; 3] {
    [
        AGENT_MARKER_WIDTH * 0.5,
        AGENT_MARKER_HEIGHT * 0.5,
        AGENT_MARKER_DEPTH * 0.5,
    ]
}

/// Half-extents of the default building marker mesh (`Cuboid::new(2, 2.5, 2)`).
#[must_use]
pub const fn building_marker_half_extents() -> [f32; 3] {
    [1.0, 1.25, 1.0]
}

/// Ray–AABB intersection distance along `direction` (normalized), if any.
#[must_use]
pub fn ray_aabb_hit_distance(
    origin: [f32; 3],
    direction: [f32; 3],
    centre: [f32; 3],
    half_extents: [f32; 3],
) -> Option<f32> {
    if !origin
        .iter()
        .chain(direction.iter())
        .all(|value| value.is_finite())
        || !centre
            .iter()
            .chain(half_extents.iter())
            .all(|value| value.is_finite())
    {
        return None;
    }
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let o = origin[axis];
        let d = direction[axis];
        let min = centre[axis] - half_extents[axis];
        let max = centre[axis] + half_extents[axis];

        if d.abs() < f32::EPSILON {
            if o < min || o > max {
                return None;
            }
            continue;
        }

        let inv = 1.0 / d;
        let mut t0 = (min - o) * inv;
        let mut t1 = (max - o) * inv;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max < t_min {
            return None;
        }
    }

    if t_max < 0.0 {
        return None;
    }
    let hit = if t_min >= 0.0 { t_min } else { t_max };
    if hit.is_finite() && hit >= 0.0 {
        Some(hit)
    } else {
        None
    }
}

/// Pick the closest streamed entity along a world-space ray.
#[must_use]
pub fn pick_live_entity_along_ray(
    origin: [f32; 3],
    direction: [f32; 3],
    agents: &[(u64, Vec3, Vec3)],
    buildings: &[(u64, Vec3, Vec3)],
    graph_parcels: &[(u64, Vec3, Vec3)],
) -> Option<SelectedLiveEntity> {
    let dir_len_sq =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if !dir_len_sq.is_finite() || dir_len_sq < f32::EPSILON {
        return None;
    }
    let inv_len = dir_len_sq.sqrt().recip();
    let dir = [
        direction[0] * inv_len,
        direction[1] * inv_len,
        direction[2] * inv_len,
    ];

    let agent_half = agent_marker_half_extents();
    let building_half = building_marker_half_extents();

    let mut best: Option<(f32, SelectedLiveEntity)> = None;

    let mut consider = |distance: f32, selection: SelectedLiveEntity| match best {
        None => best = Some((distance, selection)),
        Some((best_t, _)) if distance < best_t => best = Some((distance, selection)),
        _ => {}
    };

    for &(id, centre, scale) in agents {
        if !centre.is_finite() || !scale.is_finite() || scale.min_element() <= 0.0 {
            continue;
        }
        let half = [
            agent_half[0] * scale.x,
            agent_half[1] * scale.y,
            agent_half[2] * scale.z,
        ];
        if let Some(t) = ray_aabb_hit_distance(origin, dir, centre.to_array(), half) {
            consider(
                t,
                SelectedLiveEntity {
                    kind: LiveEntityKind::Agent,
                    id,
                },
            );
        }
    }

    for &(id, centre, scale) in buildings {
        if !centre.is_finite() || !scale.is_finite() || scale.min_element() <= 0.0 {
            continue;
        }
        let half = [
            building_half[0] * scale.x,
            building_half[1] * scale.y,
            building_half[2] * scale.z,
        ];
        if let Some(t) = ray_aabb_hit_distance(origin, dir, centre.to_array(), half) {
            consider(
                t,
                SelectedLiveEntity {
                    kind: LiveEntityKind::Building,
                    id,
                },
            );
        }
    }

    for &(id, centre, scale) in graph_parcels {
        if !centre.is_finite() || !scale.is_finite() || scale.min_element() <= 0.0 {
            continue;
        }
        let half = [
            building_half[0] * scale.x,
            building_half[1] * scale.y,
            building_half[2] * scale.z,
        ];
        if let Some(t) = ray_aabb_hit_distance(origin, dir, centre.to_array(), half) {
            consider(
                t,
                SelectedLiveEntity {
                    kind: LiveEntityKind::GraphParcel,
                    id,
                },
            );
        }
    }

    best.map(|(_, selection)| selection)
}

/// True when the cursor is over a live minimap panel (blocks viewport entity pick).
#[must_use]
pub fn pointer_over_live_minimap(
    panels: &Query<(&Interaction, &RelativeCursorPosition), With<MinimapRoot>>,
) -> bool {
    panels
        .single()
        .map(|(interaction, cursor)| {
            *interaction != Interaction::None && cursor.normalized.is_some()
        })
        .unwrap_or(false)
}

#[cfg(feature = "egui")]
fn reset_live_pick_drag_on_press(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Option<Res<GameSettings>>,
    mut pointer: ResMut<LivePickPointer>,
) {
    let binding_pressed = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_SELECT_OR_PICK))
        .unwrap_or(KeyBinding::Mouse(MouseButton::Left));
    if binding_pressed.is_pressed(&keys, &mouse) {
        pointer.left_dragged = false;
    }
}

#[cfg(not(feature = "egui"))]
fn reset_live_pick_drag_on_press(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut pointer: ResMut<LivePickPointer>,
) {
    if MouseButton::Left.is_pressed(&keys, &mouse) {
        pointer.left_dragged = false;
    }
}

#[cfg(feature = "egui")]
fn mark_live_pick_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: MessageReader<MouseMotion>,
    settings: Option<Res<GameSettings>>,
    mut pointer: ResMut<LivePickPointer>,
) {
    let binding_pressed = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_SELECT_OR_PICK))
        .unwrap_or(KeyBinding::Mouse(MouseButton::Left));
    if !binding_pressed.is_pressed(&keys, &mouse) {
        motion.clear();
        return;
    }
    if motion.read().next().is_some() {
        pointer.left_dragged = true;
    }
}

#[cfg(not(feature = "egui"))]
fn mark_live_pick_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut pointer: ResMut<LivePickPointer>,
) {
    if !mouse.pressed(MouseButton::Left) {
        motion.clear();
        return;
    }
    if motion.read().next().is_some() {
        pointer.left_dragged = true;
    }
}

#[cfg(feature = "egui")]
fn pick_live_entity_on_release(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Option<Res<GameSettings>>,
    pointer: Res<LivePickPointer>,
    minimap: Query<(&Interaction, &RelativeCursorPosition), With<MinimapRoot>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    mut selection: ResMut<LiveSelection>,
    agents: Query<(&LiveAgentTag, &GlobalTransform)>,
    buildings: Query<(&LiveBuildingTag, &GlobalTransform)>,
    graph_parcels: Query<(&LiveGraphParcelTag, &GlobalTransform)>,
) {
    let binding_released = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_SELECT_OR_PICK))
        .map(|binding| match binding {
            KeyBinding::Mouse(button) => mouse.just_released(button),
            KeyBinding::Key(key) => keys.just_released(key),
        })
        .unwrap_or_else(|| mouse.just_released(MouseButton::Left));
    if !binding_released || pointer.left_dragged {
        return;
    }
    if pointer_over_live_minimap(&minimap) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };

    let agents: Vec<_> = agents
        .iter()
        .map(|(tag, transform)| (tag.id, transform.translation(), transform.scale()))
        .collect();
    let buildings: Vec<_> = buildings
        .iter()
        .map(|(tag, transform)| (tag.id, transform.translation(), transform.scale()))
        .collect();
    let graph_parcels: Vec<_> = graph_parcels
        .iter()
        .map(|(tag, transform)| (tag.id, transform.translation(), transform.scale()))
        .collect();

    selection.0 = pick_live_entity_along_ray(
        ray.origin.to_array(),
        ray.direction.to_array(),
        &agents,
        &buildings,
        &graph_parcels,
    );
}

#[cfg(not(feature = "egui"))]
fn pick_live_entity_on_release(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<LivePickPointer>,
    minimap: Query<(&Interaction, &RelativeCursorPosition), With<MinimapRoot>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    mut selection: ResMut<LiveSelection>,
    agents: Query<(&LiveAgentTag, &GlobalTransform)>,
    buildings: Query<(&LiveBuildingTag, &GlobalTransform)>,
    graph_parcels: Query<(&LiveGraphParcelTag, &GlobalTransform)>,
) {
    if !mouse.just_released(MouseButton::Left) || pointer.left_dragged {
        return;
    }
    if pointer_over_live_minimap(&minimap) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };

    let agents: Vec<_> = agents
        .iter()
        .map(|(tag, transform)| (tag.id, transform.translation(), transform.scale()))
        .collect();
    let buildings: Vec<_> = buildings
        .iter()
        .map(|(tag, transform)| (tag.id, transform.translation(), transform.scale()))
        .collect();
    let graph_parcels: Vec<_> = graph_parcels
        .iter()
        .map(|(tag, transform)| (tag.id, transform.translation(), transform.scale()))
        .collect();

    selection.0 = pick_live_entity_along_ray(
        ray.origin.to_array(),
        ray.direction.to_array(),
        &agents,
        &buildings,
        &graph_parcels,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-025 — live entity pick helpers support attach smoke and HUD selection paths.
    #[test]
    fn ray_aabb_hit_returns_entry_distance() {
        let origin = [0.0, 0.0, -5.0];
        let direction = [0.0, 0.0, 1.0];
        let centre = [0.0, 0.0, 0.0];
        let half = [1.0, 1.0, 1.0];
        let t = ray_aabb_hit_distance(origin, direction, centre, half).expect("hit");
        assert!((t - 4.0).abs() < 1e-4);
    }

    #[test]
    fn ray_aabb_misses_when_parallel_outside_slab() {
        let origin = [0.0, 5.0, -2.0];
        let direction = [0.0, 0.0, 1.0];
        let centre = [0.0, 0.0, 0.0];
        let half = [1.0, 1.0, 1.0];
        assert!(ray_aabb_hit_distance(origin, direction, centre, half).is_none());
    }

    /// FR-CIV-BEVY-025 — picking order remains deterministic under same-ray overlap.
    #[test]
    fn pick_live_entity_prefers_nearest_along_ray() {
        let origin = [0.0, 1.0, -10.0];
        let direction = [0.0, 0.0, 1.0];
        let agents = [(1_u64, Vec3::new(0.0, 1.0, 0.0), Vec3::ONE)];
        let buildings = [(9_u64, Vec3::new(0.0, 1.0, 5.0), Vec3::ONE)];
        let picked =
            pick_live_entity_along_ray(origin, direction, &agents, &buildings, &[]).expect("pick");
        assert_eq!(picked.kind, LiveEntityKind::Agent);
        assert_eq!(picked.id, 1);
    }

    #[test]
    fn agent_marker_half_extents_match_mesh_constants() {
        let half = agent_marker_half_extents();
        assert!((half[0] - AGENT_MARKER_WIDTH * 0.5).abs() < f32::EPSILON);
        assert!((half[1] - AGENT_MARKER_HEIGHT * 0.5).abs() < f32::EPSILON);
        assert!((half[2] - AGENT_MARKER_DEPTH * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn building_marker_half_extents_match_cuboid_mesh() {
        let half = building_marker_half_extents();
        assert_eq!(half, [1.0, 1.25, 1.0]);
    }

    #[test]
    fn ray_aabb_hit_rejects_box_behind_ray_origin() {
        let origin = [0.0, 0.0, 5.0];
        let direction = [0.0, 0.0, 1.0];
        let centre = [0.0, 0.0, 0.0];
        let half = [1.0, 1.0, 1.0];
        assert!(ray_aabb_hit_distance(origin, direction, centre, half).is_none());
    }

    #[test]
    fn pick_live_entity_rejects_zero_direction() {
        assert!(
            pick_live_entity_along_ray([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], &[], &[], &[]).is_none()
        );
    }

    #[test]
    fn pick_live_entity_rejects_non_finite_hits() {
        assert!(ray_aabb_hit_distance(
            [0.0, 0.0, f32::NAN],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0]
        )
        .is_none());
        assert!(pick_live_entity_along_ray(
            [0.0, 0.0, 0.0],
            [f32::INFINITY, 0.0, 1.0],
            &[],
            &[],
            &[]
        )
        .is_none());
    }
}
