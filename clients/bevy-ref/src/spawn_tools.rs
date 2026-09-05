//! WorldBox-style spawn tools for the Bevy reference client.
//!
//! This module owns the click-to-terrain hit test, active tool state, cursor
//! marker, and local selection/destruction behavior.

use bevy::input::mouse::MouseWheel;
use bevy::math::primitives::Circle;
use bevy::prelude::*;

#[cfg(feature = "models")]
use crate::gltf_models::{actor_scene, building_scene, ModelOrPrimitive};
use crate::live_stream::ServerBridge;
use crate::minimap::MinimapCamera;
#[cfg(feature = "egui")]
pub(crate) use crate::settings_ui::GameSettings;
#[cfg(feature = "egui")]
pub(crate) use crate::settings_ui::KeyBinding;
#[cfg(feature = "egui")]
use crate::settings_ui::ACTION_SELECT_OR_PICK;
use crate::terrain::{terrain_height, WORLD_SIZE};
#[cfg(feature = "voxel")]
use crate::voxel_sim::VoxelSimState;
#[cfg(feature = "voxel")]
use civ_voxel::material::AIR;

const CIVILIAN_RADIUS: f32 = 1.4;
const CIVILIAN_BODY: f32 = 3.2;
const CIVILIAN_HALF_HEIGHT: f32 = CIVILIAN_BODY * 0.5 + CIVILIAN_RADIUS;
#[cfg(all(feature = "models", feature = "voxel"))]
const CIVILIAN_MODEL_SCALE: f32 = 8.0;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const CIVILIAN_MODEL_SCALE: f32 = 1.7;
#[cfg(all(feature = "models", feature = "voxel"))]
const HERD_MODEL_SCALE: f32 = 10.0;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const HERD_MODEL_SCALE: f32 = 2.4;
#[cfg(all(feature = "models", feature = "voxel"))]
const BUILDING_MODEL_SCALE: f32 = 4.0;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const BUILDING_MODEL_SCALE: f32 = 6.0;
const BUILDING_EXTENTS: Vec3 = Vec3::new(7.0, 12.0, 7.0);
const BUILDING_HALF_HEIGHT: f32 = BUILDING_EXTENTS.y * 0.5;
const ROAD_SEGMENT_THICKNESS: f32 = 0.6;

#[cfg(not(feature = "egui"))]
#[derive(Resource)]
pub struct GameSettings;

#[cfg(not(feature = "egui"))]
#[derive(Clone, Copy)]
pub enum KeyBinding {
    Mouse(MouseButton),
}

#[cfg(not(feature = "egui"))]
impl KeyBinding {
    pub(crate) fn is_pressed(
        self,
        _keys: &ButtonInput<KeyCode>,
        buttons: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Mouse(button) => buttons.pressed(button),
        }
    }

    pub(crate) fn is_just_pressed(
        self,
        _keys: &ButtonInput<KeyCode>,
        buttons: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Mouse(button) => buttons.just_pressed(button),
        }
    }
}

/// Shared UI pointer gate for Bevy tool systems.
///
/// Egui builds update this resource from UI code; Bevy-only builds keep the
/// default `false` value so tools remain usable without the UI crate feature.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PointerOverUi(pub bool);

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
    /// Drag-to-draw a surfaced road along a desire path.
    Road,
    /// Drag-to-draw a foot trail.
    Trail,
    /// Drag-to-draw a high-throughput highway.
    Highway,
    /// Drag-to-draw a water-spanning bridge.
    Bridge,
    /// Click-to-place a dwelling.
    House,
    /// Click-to-place an agricultural plot.
    Farm,
    /// Click-to-place a production workshop.
    Workshop,
    /// Click-to-place a trade market.
    Market,
    /// Click-to-place a defensive wall segment.
    Wall,
    /// Click-to-place a movement/trade vehicle.
    Vehicle,
    /// Paint the selected material into the voxel grid.
    PaintMaterial,
    /// Trigger a weather actor at the clicked point (rain/storm clear-out).
    Weather,
}

impl SpawnTool {
    #[must_use]
    pub fn is_road_draw(self) -> bool {
        matches!(
            self,
            SpawnTool::Road | SpawnTool::Trail | SpawnTool::Highway | SpawnTool::Bridge
        )
    }

    #[must_use]
    pub fn is_structure(self) -> bool {
        matches!(
            self,
            SpawnTool::House
                | SpawnTool::Farm
                | SpawnTool::Workshop
                | SpawnTool::Market
                | SpawnTool::Wall
        )
    }
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

#[cfg(feature = "egui")]
pub(crate) fn select_action_binding(settings: Option<&GameSettings>) -> KeyBinding {
    settings
        .and_then(|s| s.key_for(ACTION_SELECT_OR_PICK))
        .unwrap_or(KeyBinding::Mouse(MouseButton::Left))
}

#[cfg(not(feature = "egui"))]
pub(crate) fn select_action_binding(settings: Option<&GameSettings>) -> KeyBinding {
    let _ = settings;
    KeyBinding::Mouse(MouseButton::Left)
}

/// Cursor state for the terrain hit marker.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct CursorMarker {
    /// World-space position on the terrain surface.
    pub position: Option<Vec3>,
    /// Whether the marker should be visible.
    pub visible: bool,
}

/// Marker for entities created/owned by the sandbox spawn tools.
#[derive(Component, Debug, Clone, Copy)]
pub struct SandboxEntity;

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

/// Accumulator for the active drag-to-draw road stroke.
#[derive(Resource, Debug, Default, Clone)]
pub struct RoadDraft {
    /// Terrain-surface points collected so far this stroke.
    pub points: Vec<Vec3>,
    /// The road tool that started the stroke.
    pub tool: Option<SpawnTool>,
}

/// Request to lay a connected road polyline.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct PlaceRoadRequest {
    /// Ordered terrain points; consecutive pairs become segments.
    pub points: Vec<Vec3>,
    /// Which road-family tool authored the stroke.
    pub kind: SpawnTool,
}

/// Request to seat a structure or vehicle actor on terrain.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct PlaceStructureRequest {
    /// World-space click position.
    pub position: Vec3,
    /// Which structure/vehicle tool authored the placement.
    pub kind: SpawnTool,
}

/// Plugin that wires the tool state, ray hit test, and cursor marker together.
pub struct SpawnToolsPlugin;

impl Plugin for SpawnToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTool>()
            .init_resource::<BuildingSpawnKind>()
            .init_resource::<SelectedEntity>()
            .init_resource::<CursorMarker>()
            .init_resource::<PointerOverUi>()
            .init_resource::<RoadDraft>()
            .add_message::<SpawnCivilianRequest>()
            .add_message::<SpawnBuildingRequest>()
            .add_message::<SelectEntityRequest>()
            .add_message::<DestroyEntityRequest>()
            .add_message::<PlaceRoadRequest>()
            .add_message::<PlaceStructureRequest>()
            .add_systems(Startup, spawn_cursor_marker);

        #[cfg(feature = "egui")]
        app.add_systems(Update, update_pointer_over_ui);

        app.add_systems(
            Update,
            (
                update_cursor_marker,
                handle_spawn_tool_clicks,
                resolve_selection_and_destruction,
                apply_cursor_marker_visuals,
            )
                .chain(),
        );
    }
}

#[cfg(feature = "egui")]
fn update_pointer_over_ui(
    mut contexts: bevy_egui::EguiContexts,
    mut over_ui: ResMut<PointerOverUi>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        over_ui.0 = false;
        return;
    };
    over_ui.0 = ctx.wants_pointer_input() || ctx.is_pointer_over_area();
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
    cameras: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    over_ui: Res<PointerOverUi>,
    mut marker: ResMut<CursorMarker>,
    #[cfg(feature = "voxel")] voxel: Option<Res<VoxelSimState>>,
) {
    if over_ui.0 {
        marker.visible = false;
        marker.position = None;
        return;
    }

    let hit = cursor_terrain_hit(
        &windows,
        &cameras,
        #[cfg(feature = "voxel")]
        voxel.as_deref(),
    );
    marker.position = hit;
    marker.visible = hit.is_some();
}

fn cursor_terrain_hit(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    #[cfg(feature = "voxel")] voxel: Option<&VoxelSimState>,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    #[cfg(feature = "voxel")]
    if let Some(state) = voxel {
        if !state.grid.cells.is_empty() {
            return raycast_to_voxel(&state.grid, ray.origin, ray.direction.as_vec3());
        }
    }
    raycast_to_terrain(ray.origin, ray.direction.as_vec3())
}

#[cfg(feature = "voxel")]
fn raycast_to_voxel(
    grid: &civ_voxel::fluid_ca::CaGrid,
    origin: Vec3,
    direction: Vec3,
) -> Option<Vec3> {
    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }
    let dims = grid.dims;
    let max_axis = dims[0].max(dims[1]).max(dims[2]) as f32;
    let max_distance = max_axis * 4.0 + 64.0;
    let mut t = 0.0_f32;
    while t <= max_distance {
        let p = origin + dir * t;
        let (x, y, z) = (p.x.floor(), p.y.floor(), p.z.floor());
        if x >= 0.0
            && y >= 0.0
            && z >= 0.0
            && (x as usize) < dims[0]
            && (y as usize) < dims[1]
            && (z as usize) < dims[2]
            && grid.get(x as usize, y as usize, z as usize) != AIR
        {
            return Some(p);
        }
        t += 0.25;
    }
    None
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
    bridge: Option<Res<ServerBridge>>,
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
            // Server: send building placement via sim.command
            if let Some(ref bridge) = bridge {
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "spawn",
                        "kind": "building",
                        "building": building_kind.label(),
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }),
                );
            }
            spawn_building.write(SpawnBuildingRequest {
                position,
                kind: *building_kind,
            });
        }
        SpawnTool::Terraform => {
            // Terraform: raise terrain at clicked point
            if let Some(ref bridge) = bridge {
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "terraform",
                        "kind": "raise",
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }),
                );
            }
        }
        SpawnTool::PaintMaterial => {
            // Material paint is handled by the material brush sync system
            // (tool_categories::paint_material_name → brush).
            // Click here sends the painted position.
            if let Some(ref bridge) = bridge {
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "paint",
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }),
                );
            }
        }
        SpawnTool::Destroy => {
            destroy_entity.write(DestroyEntityRequest { position });
        }
        SpawnTool::Weather => {
            // Trigger a weather actor at the clicked terrain point.
            if let Some(ref bridge) = bridge {
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "weather",
                        "kind": "storm",
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }),
                );
            }
        }
        // Structure placement tools — each sends a distinct building kind.
        SpawnTool::House
        | SpawnTool::Farm
        | SpawnTool::Workshop
        | SpawnTool::Market
        | SpawnTool::Wall => {
            if let Some(ref bridge) = bridge {
                let kind = match active.tool {
                    SpawnTool::House => "House",
                    SpawnTool::Farm => "Farm",
                    SpawnTool::Workshop => "Workshop",
                    SpawnTool::Market => "Market",
                    SpawnTool::Wall => "Wall",
                    _ => unreachable!(),
                };
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "spawn",
                        "kind": "building",
                        "building": kind,
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }),
                );
            }
            spawn_building.write(SpawnBuildingRequest {
                position,
                kind: BuildingSpawnKind::CityCenter, // generic structure
            });
        }
        // Road tools start a drag-to-draw stroke.
        SpawnTool::Road | SpawnTool::Trail | SpawnTool::Highway | SpawnTool::Bridge => {
            // Road/Trail/Highway/Bridge are drag-to-draw tools.
            // The first click starts the RoadDraft; subsequent movement
            // adds points; mouse-up fires PlaceRoadRequest.
            // For single-click: send a short segment at the click point.
            if let Some(ref bridge) = bridge {
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "road",
                        "kind": format!("{:?}", active.tool).to_lowercase(),
                        "points": [
                            {"x": position.x, "z": position.z},
                            {"x": position.x + 5.0, "z": position.z + 5.0}
                        ],
                    }),
                );
            }
        }
        SpawnTool::Vehicle => {
            // Vehicle placement
            if let Some(ref bridge) = bridge {
                bridge.send_rpc(
                    "sim.command",
                    serde_json::json!({
                        "action": "spawn",
                        "kind": "vehicle",
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }),
                );
            }
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
        // Detect crossing from at/above the surface (prev_err <= 0) into the
        // terrain (err > 0). This is the entry point for a downward ray (e.g. a
        // spawn/placement raycast). The previous inverted condition only caught
        // upward rays exiting the terrain, so downward rays never registered.
        if err > 0.0 && prev_err <= 0.0 {
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
        assert_eq!(
            BuildingSpawnKind::CityCenter.next(),
            BuildingSpawnKind::Market
        );
        assert_eq!(
            BuildingSpawnKind::Market.next(),
            BuildingSpawnKind::Barracks
        );
        assert_eq!(
            BuildingSpawnKind::Barracks.next(),
            BuildingSpawnKind::CityCenter
        );
        assert_eq!(
            BuildingSpawnKind::CityCenter.prev(),
            BuildingSpawnKind::Barracks
        );
        assert_eq!(
            BuildingSpawnKind::Market.prev(),
            BuildingSpawnKind::CityCenter
        );
        assert_eq!(
            BuildingSpawnKind::Barracks.prev(),
            BuildingSpawnKind::Market
        );
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
