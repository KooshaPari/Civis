//! WorldBox-style spawn tools for the Bevy reference client.
//!
//! This module owns the click-to-terrain hit test, active tool state, cursor
//! marker, and local selection/destruction behavior.

use bevy::input::mouse::MouseWheel;
use bevy::math::primitives::Circle;
use bevy::prelude::*;

use crate::settings_ui::{GameSettings, KeyBinding, ACTION_SELECT_OR_PICK};
use crate::terrain::{terrain_height, WORLD_SIZE};

/// Tool palette used by the authoring UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpawnTool {
    /// Pick the entity nearest the clicked point.
    #[default]
    Select,
    /// Request a civilian spawn at the clicked terrain point.
    SpawnCivilian,
    /// Request a building spawn at the clicked terrain point.
    SpawnBuilding,
    /// Reserved for terrain sculpting.
    Terraform,
    /// Remove the entity nearest the clicked point.
    Destroy,
}

/// Currently active tool.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveTool {
    /// The current active tool.
    pub tool: SpawnTool,
}

impl Default for ActiveTool {
    fn default() -> Self {
        Self {
            tool: SpawnTool::Select,
        }
    }
}

/// Currently selected entity, if any.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SelectedEntity(pub Option<Entity>);

/// Building type spawned by the building tool.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildingSpawnKind {
    /// Civic hub / city center.
    #[default]
    CityCenter,
    /// Trade port / market.
    Market,
    /// Military hangar / barracks.
    Barracks,
}

impl BuildingSpawnKind {
    /// Advance to the next building type in the build palette.
    pub const fn next(self) -> Self {
        match self {
            Self::CityCenter => Self::Market,
            Self::Market => Self::Barracks,
            Self::Barracks => Self::CityCenter,
        }
    }

    /// Move to the previous building type in the build palette.
    pub const fn prev(self) -> Self {
        match self {
            Self::CityCenter => Self::Barracks,
            Self::Market => Self::CityCenter,
            Self::Barracks => Self::Market,
        }
    }

    /// Human-readable label for the current building type.
    pub const fn label(self) -> &'static str {
        match self {
            Self::CityCenter => "City Center",
            Self::Market => "Market",
            Self::Barracks => "Barracks",
        }
    }
}

pub(crate) fn select_action_binding(settings: Option<&GameSettings>) -> KeyBinding {
    settings
        .and_then(|s| s.key_for(ACTION_SELECT_OR_PICK))
        .unwrap_or(KeyBinding::Mouse(MouseButton::Left))
}

/// Cursor state for the terrain hit marker.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct CursorMarker {
    /// World-space position on the terrain surface.
    pub position: Option<Vec3>,
    /// Whether the marker should be visible.
    pub visible: bool,
}

/// Request to spawn a civilian at the clicked point.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SpawnCivilianRequest {
    /// World-space click position.
    pub position: Vec3,
}

/// Request to spawn a building at the clicked point.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SpawnBuildingRequest {
    /// World-space click position.
    pub position: Vec3,
    /// Selected building kind.
    pub kind: BuildingSpawnKind,
}

/// Request to select the entity nearest the clicked point.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SelectEntityRequest {
    /// World-space click position.
    pub position: Vec3,
}

/// Request to destroy the entity nearest the clicked point.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct DestroyEntityRequest {
    /// World-space click position.
    pub position: Vec3,
}

/// Plugin that wires the tool state, ray hit test, and cursor marker together.
pub struct SpawnToolsPlugin;

impl Plugin for SpawnToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTool>()
            .init_resource::<BuildingSpawnKind>()
            .init_resource::<SelectedEntity>()
            .init_resource::<CursorMarker>()
            .add_message::<SpawnCivilianRequest>()
            .add_message::<SpawnBuildingRequest>()
            .add_message::<SelectEntityRequest>()
            .add_message::<DestroyEntityRequest>()
            .add_systems(Startup, spawn_cursor_marker)
            .add_systems(
                Update,
                (
                    update_cursor_marker,
                    handle_spawn_tool_clicks,
                    resolve_selection_and_destruction,
                    apply_cursor_marker_visuals,
                ),
            );
    }
}

#[derive(Component)]
struct SpawnCursorMarker;

fn spawn_cursor_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ring_mesh = Mesh::from(Circle::new(1.6));
    let emissive = Color::srgb(1.0, 0.92, 0.35);
    let material = StandardMaterial {
        base_color: Color::srgba(1.0, 0.92, 0.35, 0.35),
        emissive: emissive.into(),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        cull_mode: None,
        ..default()
    };

    commands.spawn((
        SpawnCursorMarker,
        Mesh3d(meshes.add(ring_mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_xyz(0.0, 0.05, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Visibility::Hidden,
    ));
}

fn update_cursor_marker(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut marker: ResMut<CursorMarker>,
) {
    let Ok(window) = windows.single() else {
        marker.visible = false;
        marker.position = None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        marker.visible = false;
        marker.position = None;
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        marker.visible = false;
        marker.position = None;
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        marker.visible = false;
        marker.position = None;
        return;
    };

    if let Some(hit) = raycast_to_terrain(ray.origin, ray.direction.as_vec3()) {
        marker.visible = true;
        marker.position = Some(hit);
    } else {
        marker.visible = false;
        marker.position = None;
    }
}

fn handle_spawn_tool_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    settings: Option<Res<GameSettings>>,
    active: Res<ActiveTool>,
    mut building_kind: ResMut<BuildingSpawnKind>,
    marker: Res<CursorMarker>,
    mut spawn_civilian: MessageWriter<SpawnCivilianRequest>,
    mut spawn_building: MessageWriter<SpawnBuildingRequest>,
    mut select_entity: MessageWriter<SelectEntityRequest>,
    mut destroy_entity: MessageWriter<DestroyEntityRequest>,
) {
    for event in mouse_wheel.read() {
        if active.tool != SpawnTool::SpawnBuilding {
            continue;
        }

        if event.y > 0.0 {
            *building_kind = building_kind.prev();
        } else if event.y < 0.0 {
            *building_kind = building_kind.next();
        }
    }

    if active.tool == SpawnTool::SpawnBuilding && buttons.just_pressed(MouseButton::Right) {
        *building_kind = building_kind.next();
        return;
    }

    let select_pressed =
        select_action_binding(settings.as_deref()).is_just_pressed(&keys, &buttons);
    if !select_pressed {
        return;
    }
    let Some(position) = marker.position else {
        return;
    };

    match active.tool {
        SpawnTool::Select => {
            select_entity.write(SelectEntityRequest { position });
        }
        SpawnTool::SpawnCivilian => {
            spawn_civilian.write(SpawnCivilianRequest { position });
        }
        SpawnTool::SpawnBuilding => {
            spawn_building.write(SpawnBuildingRequest {
                position,
                kind: *building_kind,
            });
        }
        SpawnTool::Terraform => {}
        SpawnTool::Destroy => {
            destroy_entity.write(DestroyEntityRequest { position });
        }
    }
}

fn resolve_selection_and_destruction(
    mut commands: Commands,
    mut selected: ResMut<SelectedEntity>,
    mut select_entity: MessageReader<SelectEntityRequest>,
    mut destroy_entity: MessageReader<DestroyEntityRequest>,
    entities: Query<(Entity, &GlobalTransform)>,
) {
    for request in select_entity.read() {
        selected.0 = nearest_entity(request.position, &entities);
    }

    for request in destroy_entity.read() {
        if let Some(entity) = nearest_entity(request.position, &entities) {
            if selected.0 == Some(entity) {
                selected.0 = None;
            }
            commands.entity(entity).despawn();
        }
    }
}

fn apply_cursor_marker_visuals(
    marker: Res<CursorMarker>,
    mut query: Query<(&mut Transform, &mut Visibility), With<SpawnCursorMarker>>,
) {
    let Ok((mut transform, mut visibility)) = query.single_mut() else {
        return;
    };
    if let Some(position) = marker.position {
        transform.translation = position + Vec3::Y * 0.05;
        transform.scale = Vec3::splat(1.0);
        *visibility = if marker.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn raycast_to_terrain(origin: Vec3, direction: Vec3) -> Option<Vec3> {
    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let bounds = WORLD_SIZE * 0.5;
    let max_distance = 1_000.0;
    let step = 1.0_f32;
    let mut t = 0.0_f32;
    let mut prev_point = origin;
    let mut prev_err = terrain_error(prev_point);

    while t <= max_distance {
        let point = origin + dir * t;
        if point.x.abs() > bounds || point.z.abs() > bounds {
            prev_point = point;
            prev_err = terrain_error(point);
            t += step;
            continue;
        }

        let err = terrain_error(point);
        if err <= 0.0 && prev_err > 0.0 {
            return Some(refine_terrain_hit(prev_point, point));
        }
        prev_point = point;
        prev_err = err;
        t += step;
    }

    None
}

fn terrain_error(point: Vec3) -> f32 {
    terrain_height(point.x + WORLD_SIZE * 0.5, point.z + WORLD_SIZE * 0.5) - point.y
}

fn refine_terrain_hit(start: Vec3, end: Vec3) -> Vec3 {
    let mut a = start;
    let mut b = end;
    for _ in 0..12 {
        let mid = (a + b) * 0.5;
        if terrain_error(mid) > 0.0 {
            a = mid;
        } else {
            b = mid;
        }
    }

    let mut hit = (a + b) * 0.5;
    hit.y = terrain_height(hit.x + WORLD_SIZE * 0.5, hit.z + WORLD_SIZE * 0.5);
    hit
}

fn nearest_entity(position: Vec3, entities: &Query<(Entity, &GlobalTransform)>) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for (entity, transform) in entities.iter() {
        let distance = transform.translation().distance_squared(position);
        match best {
            None => best = Some((entity, distance)),
            Some((_, best_distance)) if distance < best_distance => best = Some((entity, distance)),
            _ => {}
        }
    }
    best.map(|(entity, _)| entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_tool_defaults_to_select() {
        assert_eq!(ActiveTool::default().tool, SpawnTool::Select);
    }

    #[test]
    fn building_spawn_kind_cycles_and_labels() {
        assert_eq!(BuildingSpawnKind::CityCenter.label(), "City Center");
        assert_eq!(BuildingSpawnKind::Market.label(), "Market");
        assert_eq!(BuildingSpawnKind::Barracks.label(), "Barracks");
        assert_eq!(BuildingSpawnKind::CityCenter.next(), BuildingSpawnKind::Market);
        assert_eq!(BuildingSpawnKind::Market.next(), BuildingSpawnKind::Barracks);
        assert_eq!(BuildingSpawnKind::Barracks.next(), BuildingSpawnKind::CityCenter);
        assert_eq!(BuildingSpawnKind::CityCenter.prev(), BuildingSpawnKind::Barracks);
        assert_eq!(BuildingSpawnKind::Market.prev(), BuildingSpawnKind::CityCenter);
        assert_eq!(BuildingSpawnKind::Barracks.prev(), BuildingSpawnKind::Market);
    }

    #[test]
    fn terrain_raycast_hits_centre_near_height() {
        let origin = Vec3::new(0.0, 200.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let hit = raycast_to_terrain(origin, dir).expect("terrain hit");
        assert!(hit.y >= 0.0);
        assert!(hit.y <= crate::terrain::HEIGHT_SCALE);
    }
}
