//! WorldBox-style spawn tools for the Bevy reference client.
//!
//! This module owns the click-to-terrain hit test, active tool state, cursor
//! marker, and local selection/destruction behavior.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::math::primitives::Circle;
use bevy::prelude::*;

#[cfg(feature = "models")]
use crate::gltf_models::{actor_scene, building_scene, ModelOrPrimitive};
use crate::live_ground::ChunkVoxelCache;
use crate::live_stream::{LiveBridge, LiveStreamScene, ServerBridge};
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
use crate::ws_client::RpcTicket;
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
    /// Existing server palette aliases for the corresponding engine buildings.
    const fn rpc_kind(self) -> &'static str {
        match self {
            Self::CityCenter => "airport",
            Self::Market => "port",
            Self::Barracks => "hangar",
        }
    }

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

/// Footprint of the placement ghost for a given tool.
///
/// The ghost uses world-space extents so the player can see exactly where the
/// structure will sit before clicking. For tools without a clear footprint
/// (Select / Destroy / paint / weather) we return `None` to signal that the
/// ghost should be hidden.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhostFootprint {
    /// Half-extents of the preview box on X / Z; full Y extent is fixed by
    /// `BUILDING_EXTENTS` so the preview matches what will be spawned.
    pub half_width: f32,
    pub half_depth: f32,
    /// Full height of the preview box.
    pub full_height: f32,
}

impl GhostFootprint {
    /// Build a footprint from world-space full extents.
    const fn from_extents(extents: Vec3) -> Self {
        Self {
            half_width: extents.x * 0.5,
            half_depth: extents.z * 0.5,
            full_height: extents.y,
        }
    }

    /// Footprint shown when the active tool is a structure placer.
    pub fn for_tool(tool: SpawnTool) -> Option<Self> {
        if tool.is_structure() || matches!(tool, SpawnTool::SpawnBuilding) {
            Some(Self::from_extents(BUILDING_EXTENTS))
        } else {
            None
        }
    }
}

/// State of the placement ghost preview.
///
/// Updated by `update_ghost_preview` each frame and rendered by
/// `apply_ghost_preview_visuals`. Holds the resolved footprint so the render
/// side can paint a translucent box without re-deriving the active tool.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct GhostPreview {
    /// Tool whose footprint is currently being previewed, if any.
    pub tool: Option<SpawnTool>,
    /// World-space position the ghost sits at, if visible.
    pub position: Option<Vec3>,
    /// Whether the ghost is currently rendered.
    pub visible: bool,
}

impl GhostPreview {
    /// True when the ghost should appear at the given cursor position.
    ///
    /// A ghost is shown iff:
    /// - a structure tool is active,
    /// - the cursor marker resolved a world position, and
    /// - the marker is reported visible (e.g. not hidden behind UI).
    pub fn should_render(&self, marker: &CursorMarker) -> bool {
        self.tool.is_some() && marker.visible && marker.position.is_some() && self.position.is_some()
    }
}

/// Why an attached client currently has no authoritative terrain marker.
///
/// Attached tools deliberately do not fall back to local terrain. Retaining the
/// immediate reason lets the rate-limited authoring feedback distinguish an
/// input/UI problem from a streamed-world problem without exposing protocol
/// details to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttachedMarkerUnavailable {
    PointerOverUi,
    InputUnavailable,
    CacheUnavailable,
    CacheEmpty,
    RayMiss { cached_chunks: usize },
}

impl AttachedMarkerUnavailable {
    fn summary(self) -> String {
        match self {
            Self::PointerOverUi => "pointer is captured by the UI".to_owned(),
            Self::InputUnavailable => "window, cursor, or camera input is unavailable".to_owned(),
            Self::CacheUnavailable => "streamed terrain cache is unavailable".to_owned(),
            Self::CacheEmpty => "streamed terrain cache is empty (0 chunks)".to_owned(),
            Self::RayMiss { cached_chunks } => {
                format!("ray missed streamed terrain ({cached_chunks} cached chunks)")
            }
        }
    }
}

/// Latest attached-marker diagnostic, updated together with [`CursorMarker`].
#[derive(Resource, Debug, Default, Clone, Copy)]
struct AttachedMarkerStatus {
    unavailable: Option<AttachedMarkerUnavailable>,
}

/// Bounds attached-tool diagnostics so a held or repeated click cannot flood
/// the event feed while a terrain marker is unavailable.
#[cfg(feature = "egui")]
#[derive(Resource, Default)]
struct AttachedMarkerDiagnostic {
    last_reported: Option<std::time::Instant>,
    messages: Vec<String>,
}

const ATTACHED_MARKER_DIAGNOSTIC_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(2);

fn is_server_authoring_tool(mode: Option<&crate::AttachMode>, tool: SpawnTool) -> bool {
    matches!(mode, Some(crate::AttachMode::Server))
        && !matches!(tool, SpawnTool::Select | SpawnTool::Destroy)
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

#[derive(Resource, Default)]
struct PendingTerrainStamps {
    tickets: Vec<(RpcTicket, std::time::Instant)>,
    errors: Vec<String>,
}

#[derive(Resource, Default)]
struct PendingBuildingPlacements {
    tickets: Vec<(BuildingSpawnKind, RpcTicket, std::time::Instant)>,
    messages: Vec<String>,
}

#[derive(SystemParam)]
struct PendingToolRequests<'w> {
    terrain: ResMut<'w, PendingTerrainStamps>,
    buildings: ResMut<'w, PendingBuildingPlacements>,
    marker_status: Res<'w, AttachedMarkerStatus>,
    #[cfg(feature = "egui")]
    marker_diagnostic: ResMut<'w, AttachedMarkerDiagnostic>,
}

pub(crate) fn server_tools_active(mode: Option<&crate::AttachMode>, bridge_present: bool) -> bool {
    match mode {
        Some(crate::AttachMode::Standalone) => false,
        Some(crate::AttachMode::Server) => true,
        None => bridge_present,
    }
}

impl Plugin for SpawnToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTool>()
            .init_resource::<PendingTerrainStamps>()
            .init_resource::<PendingBuildingPlacements>()
            .init_resource::<BuildingSpawnKind>()
            .init_resource::<SelectedEntity>()
            .init_resource::<CursorMarker>()
            .init_resource::<GhostPreview>()
            .init_resource::<AttachedMarkerStatus>()
            .init_resource::<PointerOverUi>()
            .init_resource::<RoadDraft>()
            .add_message::<SpawnCivilianRequest>()
            .add_message::<SpawnBuildingRequest>()
            .add_message::<SelectEntityRequest>()
            .add_message::<DestroyEntityRequest>()
            .add_message::<PlaceRoadRequest>()
            .add_message::<PlaceStructureRequest>()
            .add_systems(Startup, (spawn_cursor_marker, spawn_ghost_preview));

        #[cfg(feature = "egui")]
        app.init_resource::<crate::event_feed::EventFeed>()
            .init_resource::<AttachedMarkerDiagnostic>()
            .add_systems(
                Update,
                (
                    update_pointer_over_ui,
                    report_terrain_stamps,
                    report_building_placements,
                    report_attached_marker_diagnostics,
                ),
            );

        app.add_systems(
            Update,
            (
                update_cursor_marker,
                update_ghost_preview,
                handle_spawn_tool_clicks,
                resolve_selection_and_destruction,
                apply_cursor_marker_visuals,
                apply_ghost_preview_visuals,
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
    mut attached_status: ResMut<AttachedMarkerStatus>,
    live: Option<Res<LiveStreamScene>>,
    bridge: Option<Res<ServerBridge>>,
    mode: Option<Res<crate::AttachMode>>,
    #[cfg(feature = "voxel")] voxel: Option<Res<VoxelSimState>>,
) {
    let attached = server_tools_active(mode.as_deref(), bridge.is_some());
    if over_ui.0 {
        marker.visible = false;
        marker.position = None;
        attached_status.unavailable = attached.then_some(AttachedMarkerUnavailable::PointerOverUi);
        return;
    }

    if attached {
        match attached_cursor_terrain_hit(
            &windows,
            &cameras,
            live.as_deref().map(|scene| &scene.chunk_voxels),
        ) {
            Ok(hit) => {
                marker.position = Some(hit);
                marker.visible = true;
                attached_status.unavailable = None;
            }
            Err(reason) => {
                marker.position = None;
                marker.visible = false;
                attached_status.unavailable = Some(reason);
            }
        }
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
    attached_status.unavailable = None;
}

fn cursor_terrain_hit(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    #[cfg(feature = "voxel")] voxel: Option<&VoxelSimState>,
) -> Option<Vec3> {
    let (origin, direction) = cursor_world_ray(windows, cameras)?;
    #[cfg(feature = "voxel")]
    if let Some(state) = voxel {
        if !state.grid.cells.is_empty() {
            return raycast_to_voxel(&state.grid, origin, direction);
        }
    }
    raycast_to_terrain(origin, direction)
}

fn cursor_world_ray(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
) -> Option<(Vec3, Vec3)> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    Some((ray.origin, ray.direction.as_vec3()))
}

fn attached_cursor_terrain_hit(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    cache: Option<&ChunkVoxelCache>,
) -> Result<Vec3, AttachedMarkerUnavailable> {
    let (origin, direction) =
        cursor_world_ray(windows, cameras).ok_or(AttachedMarkerUnavailable::InputUnavailable)?;
    attached_cache_hit(cache, origin, direction)
}

fn attached_cache_hit(
    cache: Option<&ChunkVoxelCache>,
    origin: Vec3,
    direction: Vec3,
) -> Result<Vec3, AttachedMarkerUnavailable> {
    let cache = cache.ok_or(AttachedMarkerUnavailable::CacheUnavailable)?;
    let cached_chunks = cache.chunks().len();
    if cached_chunks == 0 {
        return Err(AttachedMarkerUnavailable::CacheEmpty);
    }
    // An attached client must never aim using a different, local world.
    raycast_to_live_voxels(cache, origin, direction)
        .ok_or(AttachedMarkerUnavailable::RayMiss { cached_chunks })
}

fn raycast_to_live_voxels(cache: &ChunkVoxelCache, origin: Vec3, direction: Vec3) -> Option<Vec3> {
    let direction = direction.normalize_or_zero();
    if !origin.is_finite() || !direction.is_finite() || direction == Vec3::ZERO {
        return None;
    }
    let hit_distance = |centre: Vec3, half: f32| {
        crate::live_pick::ray_aabb_hit_distance(
            origin.to_array(),
            direction.to_array(),
            centre.to_array(),
            [half; 3],
        )
    };
    let mut closest: Option<f32> = None;
    for (&id, voxels) in cache.chunks() {
        let (cx, cy, cz) = crate::decode_chunk_id(civ_voxel::ChunkId(id));
        let base = Vec3::new(cx as f32, cy as f32, cz as f32) * 16.0;
        if hit_distance(base + Vec3::splat(8.0), 8.0).is_none() {
            continue;
        }
        for (index, material) in voxels.iter().enumerate() {
            if material.0 == 0 {
                continue;
            }
            let cell = Vec3::new(
                (index % 16) as f32,
                ((index / 16) % 16) as f32,
                (index / 256) as f32,
            );
            if let Some(distance) = hit_distance(base + cell + Vec3::splat(0.5), 0.5) {
                closest = Some(closest.map_or(distance, |old| old.min(distance)));
            }
        }
    }
    closest.map(|distance| origin + direction * distance)
}

fn terrain_stamp_params(position: Vec3, material: u16, radius: u8) -> serde_json::Value {
    let scale = civ_voxel::FIXED_SCALE as f64;
    serde_json::json!({
        "x": (f64::from(position.x.floor()) * scale) as i64,
        "y": (f64::from(position.y.floor()) * scale) as i64,
        "z": (f64::from(position.z.floor()) * scale) as i64,
        "op": "raise",
        "material": material,
        "radius": radius,
        "role": "operator",
    })
}

fn terrain_stamp_feedback(result: Result<serde_json::Value, String>) -> String {
    match result {
        Ok(value) if value.get("ok").and_then(|v| v.as_bool()) == Some(true) => {
            match value
                .get("writes")
                .and_then(|v| v.as_u64())
                .filter(|&n| n > 0)
            {
                Some(writes) => format!("Server applied terrain disk: {writes} voxel writes"),
                None => "Terrain stamp failed: server returned no voxel write receipt".to_owned(),
            }
        }
        Ok(_) => "Terrain stamp failed: invalid server acknowledgment".to_owned(),
        Err(error) => format!("Terrain stamp failed: {error}"),
    }
}

fn building_placement_params(
    position: Vec3,
    kind: BuildingSpawnKind,
) -> Result<serde_json::Value, String> {
    let half = WORLD_SIZE * 0.5;
    if !position.is_finite() || position.x.abs() > half || position.z.abs() > half {
        return Err("placement is outside the supported map bounds".to_owned());
    }
    // The existing spawn API takes normalized horizontal map coordinates.
    // Its `y` is map Z, not terrain elevation; the live renderer seats the building.
    Ok(serde_json::json!({
        "kind": kind.rpc_kind(),
        "x": (position.x + half) / WORLD_SIZE,
        "y": (position.z + half) / WORLD_SIZE,
        "role": "operator",
    }))
}

fn request_building_placement(
    position: Vec3,
    kind: BuildingSpawnKind,
    live: Option<&LiveBridge>,
    pending: &mut PendingBuildingPlacements,
) {
    let params = match building_placement_params(position, kind) {
        Ok(params) => params,
        Err(error) => {
            pending
                .messages
                .push(format!("{} placement failed: {error}", kind.label()));
            return;
        }
    };
    let Some(live) = live else {
        pending.messages.push(format!(
            "{} placement failed: live command connection unavailable",
            kind.label()
        ));
        return;
    };
    let ticket = live.client.request_rpc("sim.spawn_entity", params);
    let ticket_id = ticket.id;
    pending
        .tickets
        .push((kind, ticket, std::time::Instant::now()));
    pending.messages.push(format!(
        "building placement rpc #{ticket_id} sent; awaiting server."
    ));
}

fn building_placement_feedback(
    kind: BuildingSpawnKind,
    result: Result<serde_json::Value, String>,
) -> String {
    match result {
        Ok(value)
            if value["ok"].as_bool() == Some(true)
                && value["accepted"].as_bool() == Some(true)
                && value["kind"].as_str() == Some(kind.rpc_kind())
                && value["entity_id"].as_u64().is_some() =>
        {
            format!(
                "Server created {} (entity {}); awaiting a world update.",
                kind.label(),
                value["entity_id"]
            )
        }
        Ok(_) => format!(
            "{} placement failed: invalid server acknowledgment",
            kind.label()
        ),
        Err(error) => format!("{} placement failed: {error}", kind.label()),
    }
}

#[cfg(feature = "egui")]
fn report_building_placements(
    mut pending: ResMut<PendingBuildingPlacements>,
    mut feed: ResMut<crate::event_feed::EventFeed>,
) {
    for message in pending.messages.drain(..) {
        feed.push(crate::event_feed::EventKind::System, message);
    }
    pending.tickets.retain(|(kind, ticket, sent_at)| {
        if let Some(result) = ticket.try_recv() {
            feed.push(crate::event_feed::EventKind::System, building_placement_feedback(*kind, result));
            false
        } else if sent_at.elapsed() >= std::time::Duration::from_secs(10) {
            feed.push(crate::event_feed::EventKind::System, format!("{} placement timed out; the result is unknown. Check the world before retrying.", kind.label()));
            false
        } else {
            true
        }
    });
}

#[cfg(feature = "egui")]
fn report_attached_marker_diagnostics(
    mut diagnostic: ResMut<AttachedMarkerDiagnostic>,
    mut feed: ResMut<crate::event_feed::EventFeed>,
) {
    for message in diagnostic.messages.drain(..) {
        feed.push(crate::event_feed::EventKind::System, message);
    }
}

#[cfg(feature = "egui")]
fn report_terrain_stamps(
    mut pending: ResMut<PendingTerrainStamps>,
    mut feed: ResMut<crate::event_feed::EventFeed>,
) {
    for error in pending.errors.drain(..) {
        feed.push(crate::event_feed::EventKind::System, error);
    }
    pending.tickets.retain(|(ticket, sent_at)| {
        if let Some(result) = ticket.try_recv() {
            feed.push(
                crate::event_feed::EventKind::System,
                terrain_stamp_feedback(result),
            );
            false
        } else if sent_at.elapsed() >= std::time::Duration::from_secs(10) {
            feed.push(
                crate::event_feed::EventKind::System,
                "Terrain stamp timed out; the result is unknown. Check the world before retrying.",
            );
            false
        } else {
            true
        }
    });
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
    live: Option<Res<LiveBridge>>,
    mut pending: PendingToolRequests,
    mode: Option<Res<crate::AttachMode>>,
    #[cfg(feature = "egui")] brush: Option<Res<crate::material_brush_ui::SelectedMaterial>>,
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
        #[cfg(feature = "egui")]
        if is_server_authoring_tool(mode.as_deref(), active.tool)
            && pending
                .marker_diagnostic
                .last_reported
                .is_none_or(|reported| reported.elapsed() >= ATTACHED_MARKER_DIAGNOSTIC_COOLDOWN)
        {
            pending.marker_diagnostic.last_reported = Some(std::time::Instant::now());
            let reason = pending
                .marker_status
                .unavailable
                .map(AttachedMarkerUnavailable::summary)
                .unwrap_or_else(|| "marker state has not been sampled yet".to_owned());
            pending.marker_diagnostic.messages.push(format!(
                "{:?} not sent: attached terrain marker is unavailable ({reason}); move over streamed terrain and retry.",
                active.tool,
            ));
        }
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
            if server_tools_active(mode.as_deref(), bridge.is_some() || live.is_some()) {
                request_building_placement(
                    position,
                    *building_kind,
                    live.as_deref(),
                    &mut pending.buildings,
                );
            } else {
                spawn_building.write(SpawnBuildingRequest {
                    position,
                    kind: *building_kind,
                });
            }
        }
        SpawnTool::Terraform => {
            // The server contract is a one-layer disk, not a
            // spherical material brush. Its reply, not the click, confirms it.
            if server_tools_active(mode.as_deref(), bridge.is_some()) {
                let mut material = civ_voxel::default_material_for_op("raise").0;
                let mut radius = 3;
                #[cfg(feature = "egui")]
                if let Some(brush) = brush.as_deref() {
                    material = brush.material.0;
                    radius = brush.clamped_size().round() as u8;
                }
                if let Some(ref live) = live {
                    let ticket = live.client.request_rpc(
                        "sim.terraform_extent",
                        terrain_stamp_params(position, material, radius),
                    );
                    let ticket_id = ticket.id;
                    pending
                        .terrain
                        .tickets
                        .push((ticket, std::time::Instant::now()));
                    pending
                        .terrain
                        .errors
                        .push(format!("terraform rpc #{ticket_id} sent; awaiting server."));
                } else {
                    pending.terrain.errors.push(
                        "Terrain stamp failed: live command connection unavailable".to_owned(),
                    );
                }
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
            if server_tools_active(mode.as_deref(), bridge.is_some() || live.is_some()) {
                if active.tool == SpawnTool::Market {
                    request_building_placement(
                        position,
                        BuildingSpawnKind::Market,
                        live.as_deref(),
                        &mut pending.buildings,
                    );
                } else {
                    pending.buildings.messages.push(
                        "This structure is not supported by the live building API.".to_owned(),
                    );
                }
            } else {
                spawn_building.write(SpawnBuildingRequest {
                    position,
                    kind: BuildingSpawnKind::CityCenter, // Existing standalone structure behavior.
                });
            }
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

/// Derive the ghost-preview state from the current tool + cursor marker.
///
/// Runs every frame before `apply_ghost_preview_visuals` so the render-side
/// system just copies the resource into the entity transform.
fn update_ghost_preview(
    active: Res<ActiveTool>,
    marker: Res<CursorMarker>,
    mut ghost: ResMut<GhostPreview>,
) {
    let footprint = GhostFootprint::for_tool(active.tool);
    ghost.tool = footprint.map(|_| active.tool);
    ghost.position = if footprint.is_some() {
        marker.position
    } else {
        None
    };
    ghost.visible = ghost.should_render(&marker);
}

/// Marker for the spawned ghost preview entity.
#[derive(Component)]
pub struct SpawnGhostPreview;

/// Spawn a translucent box used as the placement ghost.
///
/// Default position is the world origin; visibility is hidden so the ghost
/// does not flash before `update_ghost_preview` runs.
fn spawn_ghost_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let half = BUILDING_EXTENTS.x * 0.5;
    let depth_half = BUILDING_EXTENTS.z * 0.5;
    let half_height = BUILDING_EXTENTS.y * 0.5;
    let mesh = Mesh::from(Cuboid::new(
        half * 2.0,
        BUILDING_EXTENTS.y,
        depth_half * 2.0,
    ));
    let material = StandardMaterial {
        base_color: Color::srgba(0.35, 0.85, 1.0, 0.25),
        emissive: Color::srgba(0.35, 0.85, 1.0, 0.45).into(),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    };

    commands.spawn((
        SpawnGhostPreview,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        // Lift so the box bottom sits on the terrain (cursor sits at surface).
        Transform::from_xyz(0.0, half_height + 0.05, 0.0),
        Visibility::Hidden,
    ));
}

/// Push the resource-driven ghost state into the rendered entity.
fn apply_ghost_preview_visuals(
    ghost: Res<GhostPreview>,
    mut query: Query<(&mut Transform, &mut Visibility), With<SpawnGhostPreview>>,
) {
    let Ok((mut transform, mut visibility)) = query.single_mut() else {
        return;
    };
    if let Some(position) = ghost.position {
        let half_height = BUILDING_EXTENTS.y * 0.5;
        transform.translation = position + Vec3::Y * (half_height + 0.05);
        *visibility = if ghost.visible {
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
    fn ghost_footprint_returns_none_for_non_structure_tools() {
        // Select / Destroy / paint / weather should never show a placement
        // ghost because there is no building-shaped footprint for them.
        for tool in [
            SpawnTool::Select,
            SpawnTool::Destroy,
            SpawnTool::PaintMaterial,
            SpawnTool::Weather,
        ] {
            assert_eq!(
                GhostFootprint::for_tool(tool),
                None,
                "{tool:?} must not produce a ghost footprint"
            );
        }
    }

    #[test]
    fn ghost_footprint_matches_building_extents_for_structure_tools() {
        // Every structure placer must share the same footprint as the
        // BUILDING_EXTENTS constant so the preview matches what will spawn.
        let expected_half_x = BUILDING_EXTENTS.x * 0.5;
        let expected_half_z = BUILDING_EXTENTS.z * 0.5;
        for tool in [
            SpawnTool::SpawnBuilding,
            SpawnTool::House,
            SpawnTool::Farm,
            SpawnTool::Workshop,
            SpawnTool::Market,
            SpawnTool::Wall,
        ] {
            let fp = GhostFootprint::for_tool(tool)
                .unwrap_or_else(|| panic!("{tool:?} must produce a ghost footprint"));
            assert!(
                (fp.half_width - expected_half_x).abs() < 1e-4,
                "{tool:?}: half_width mismatch"
            );
            assert!(
                (fp.half_depth - expected_half_z).abs() < 1e-4,
                "{tool:?}: half_depth mismatch"
            );
            assert!(
                (fp.full_height - BUILDING_EXTENTS.y).abs() < 1e-4,
                "{tool:?}: full_height mismatch"
            );
        }
    }

    #[test]
    fn ghost_preview_should_render_requires_tool_and_visible_cursor() {
        let pos = Some(Vec3::new(1.0, 2.0, 3.0));
        // No tool: ghost never renders, even with a visible cursor.
        let marker = CursorMarker {
            position: pos,
            visible: true,
        };
        let ghost = GhostPreview {
            tool: None,
            position: pos,
            visible: true,
        };
        assert!(!ghost.should_render(&marker));

        // Has tool but cursor hidden behind UI: ghost hides.
        let ghost = GhostPreview {
            tool: Some(SpawnTool::House),
            position: pos,
            visible: true,
        };
        let marker = CursorMarker {
            position: pos,
            visible: false,
        };
        assert!(!ghost.should_render(&marker));

        // Has tool but cursor ray missed terrain.
        let ghost = GhostPreview {
            tool: Some(SpawnTool::House),
            position: pos,
            visible: true,
        };
        let marker = CursorMarker {
            position: None,
            visible: true,
        };
        assert!(!ghost.should_render(&marker));

        // Happy path: structure tool + visible cursor + a resolved world
        // position. Ghost renders.
        let ghost = GhostPreview {
            tool: Some(SpawnTool::House),
            position: pos,
            visible: true,
        };
        let marker = CursorMarker {
            position: pos,
            visible: true,
        };
        assert!(ghost.should_render(&marker));
    }

    #[test]
    fn building_placement_uses_existing_palette_and_horizontal_normalized_coordinates() {
        for (kind, alias) in [
            (BuildingSpawnKind::CityCenter, "airport"),
            (BuildingSpawnKind::Market, "port"),
            (BuildingSpawnKind::Barracks, "hangar"),
        ] {
            let params = building_placement_params(Vec3::new(-64.5, 47.0, 32.25), kind).unwrap();
            assert_eq!(params["kind"], alias);
            assert_eq!(params["x"], (128.0 - 64.5) / 256.0);
            assert_eq!(params["y"], (128.0 + 32.25) / 256.0);
            assert!(params.get("z").is_none());
            assert_eq!(params["role"], "operator");
        }
        for position in [
            Vec3::new(-128.1, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 128.1),
            Vec3::splat(f32::NAN),
        ] {
            assert!(building_placement_params(position, BuildingSpawnKind::CityCenter).is_err());
        }
    }

    #[test]
    fn building_placement_feedback_requires_matching_created_entity_receipt() {
        for value in [
            serde_json::json!({}),
            serde_json::json!({"accepted":true,"kind":"airport"}),
            serde_json::json!({"accepted":true,"ok":true,"kind":"airport"}),
            serde_json::json!({"accepted":true,"ok":true,"kind":"port","entity_id":1}),
        ] {
            assert!(
                building_placement_feedback(BuildingSpawnKind::CityCenter, Ok(value))
                    .contains("failed")
            );
        }
        let success = building_placement_feedback(
            BuildingSpawnKind::CityCenter,
            Ok(serde_json::json!({"accepted":true,"ok":true,"kind":"airport","entity_id":0})),
        );
        assert!(success.contains("Server created City Center"));
        assert!(success.contains("awaiting a world update"));
        assert!(!success.contains("resume if paused"));
        assert!(!success.contains("airport"));
        assert!(building_placement_feedback(
            BuildingSpawnKind::CityCenter,
            Err("Forbidden: operator required".into())
        )
        .contains("Forbidden"));
    }

    #[cfg(feature = "egui")]
    fn building_test_app(
        mode: crate::AttachMode,
    ) -> (
        App,
        crossbeam_channel::Receiver<String>,
        crossbeam_channel::Receiver<String>,
    ) {
        let (client, requests) = crate::ws_client::WsClient::test_rpc_client();
        let (legacy_tx, legacy_rx) = crossbeam_channel::unbounded();
        let mut app = App::new();
        app.insert_resource(mode)
            .insert_resource(LiveBridge { client })
            .insert_resource(ServerBridge::new(legacy_tx))
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ActiveTool {
                tool: SpawnTool::SpawnBuilding,
            })
            .insert_resource(CursorMarker {
                position: Some(Vec3::new(-64.5, 47.0, 32.25)),
                visible: true,
            })
            .init_resource::<BuildingSpawnKind>()
            .init_resource::<PendingTerrainStamps>()
            .init_resource::<PendingBuildingPlacements>()
            .init_resource::<AttachedMarkerDiagnostic>()
            .init_resource::<AttachedMarkerStatus>()
            .init_resource::<crate::event_feed::EventFeed>()
            .add_message::<MouseWheel>()
            .add_message::<SpawnCivilianRequest>()
            .add_message::<SpawnBuildingRequest>()
            .add_message::<SelectEntityRequest>()
            .add_message::<DestroyEntityRequest>()
            .add_systems(
                Update,
                (
                    handle_spawn_tool_clicks,
                    report_attached_marker_diagnostics,
                    report_building_placements,
                )
                    .chain(),
            );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        (app, requests, legacy_rx)
    }

    #[cfg(feature = "egui")]
    fn terraform_test_app() -> (App, crossbeam_channel::Receiver<String>) {
        let (client, requests) = crate::ws_client::WsClient::test_rpc_client();
        let (legacy_tx, _legacy_rx) = crossbeam_channel::unbounded();
        let mut app = App::new();
        app.insert_resource(crate::AttachMode::Server)
            .insert_resource(LiveBridge { client })
            .insert_resource(ServerBridge::new(legacy_tx))
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ActiveTool {
                tool: SpawnTool::Terraform,
            })
            .insert_resource(CursorMarker {
                position: Some(Vec3::new(-64.5, 47.0, 32.25)),
                visible: true,
            })
            .init_resource::<BuildingSpawnKind>()
            .init_resource::<PendingTerrainStamps>()
            .init_resource::<PendingBuildingPlacements>()
            .init_resource::<AttachedMarkerDiagnostic>()
            .init_resource::<AttachedMarkerStatus>()
            .init_resource::<crate::event_feed::EventFeed>()
            .add_message::<MouseWheel>()
            .add_message::<SpawnCivilianRequest>()
            .add_message::<SpawnBuildingRequest>()
            .add_message::<SelectEntityRequest>()
            .add_message::<DestroyEntityRequest>()
            .add_systems(
                Update,
                (
                    handle_spawn_tool_clicks,
                    report_terrain_stamps,
                    report_attached_marker_diagnostics,
                )
                    .chain(),
            );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        (app, requests)
    }

    #[cfg(feature = "egui")]
    #[test]
    fn terraform_click_reports_only_its_correlated_rpc_ticket() {
        let (mut app, requests) = terraform_test_app();
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        let request: serde_json::Value =
            serde_json::from_str(&requests.try_recv().expect("terraform request")).unwrap();
        let id = request["id"].as_u64().expect("request ticket id");
        assert_eq!(request["method"], "sim.terraform_extent");
        assert!(app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text.contains(&format!("terraform rpc #{id} sent"))));

        app.world()
            .resource::<LiveBridge>()
            .client
            .test_complete_rpc(id + 1, Ok(serde_json::json!({ "ok": true, "writes": 99 })));
        app.update();
        assert_eq!(
            app.world().resource::<PendingTerrainStamps>().tickets.len(),
            1,
            "a reply for another ticket must not complete this Terraform action"
        );

        app.world()
            .resource::<LiveBridge>()
            .client
            .test_complete_rpc(id, Ok(serde_json::json!({ "ok": true, "writes": 49 })));
        app.update();
        assert!(app
            .world()
            .resource::<PendingTerrainStamps>()
            .tickets
            .is_empty());
        assert!(app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text == "Server applied terrain disk: 49 voxel writes"));
    }

    #[cfg(feature = "egui")]
    #[test]
    fn building_click_waits_for_correlated_receipt_without_local_mirror() {
        let (mut app, requests, legacy) = building_test_app(crate::AttachMode::Server);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        let request: serde_json::Value =
            serde_json::from_str(&requests.try_recv().unwrap()).unwrap();
        assert_eq!(request["method"], "sim.spawn_entity");
        assert_eq!(request["params"]["kind"], "airport");
        assert!(legacy.try_recv().is_err());
        assert_eq!(
            app.world()
                .resource::<Messages<SpawnBuildingRequest>>()
                .len(),
            0
        );
        assert!(!app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text.contains("Server created")));
        let id = request["id"].as_u64().unwrap();
        app.world()
            .resource::<LiveBridge>()
            .client
            .test_complete_rpc(
                id + 100,
                Ok(serde_json::json!({"accepted":true,"ok":true,"kind":"airport","entity_id":7})),
            );
        app.update();
        assert_eq!(
            app.world()
                .resource::<PendingBuildingPlacements>()
                .tickets
                .len(),
            1
        );
        app.world()
            .resource::<LiveBridge>()
            .client
            .test_complete_rpc(
                id,
                Ok(serde_json::json!({"accepted":true,"ok":true,"kind":"airport","entity_id":7})),
            );
        app.update();
        assert!(app
            .world()
            .resource::<PendingBuildingPlacements>()
            .tickets
            .is_empty());
        assert!(app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text.contains("Server created City Center")));
        assert!(requests.try_recv().is_err());
    }

    #[cfg(feature = "egui")]
    #[test]
    fn server_authoring_click_with_no_marker_reports_once_without_sending() {
        let (mut app, requests, legacy) = building_test_app(crate::AttachMode::Server);
        app.world_mut().resource_mut::<CursorMarker>().position = None;
        app.world_mut()
            .resource_mut::<AttachedMarkerStatus>()
            .unavailable = Some(AttachedMarkerUnavailable::CacheEmpty);

        app.update();
        assert!(requests.try_recv().is_err());
        assert!(legacy.try_recv().is_err());
        let feed = app.world().resource::<crate::event_feed::EventFeed>();
        assert_eq!(feed.events.len(), 1);
        assert!(feed
            .events
            .front()
            .unwrap()
            .text
            .contains("SpawnBuilding not sent"));
        assert!(feed
            .events
            .front()
            .unwrap()
            .text
            .contains("streamed terrain cache is empty (0 chunks)"));

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world()
                .resource::<crate::event_feed::EventFeed>()
                .events
                .len(),
            1,
            "the two-second cooldown must prevent repeated click spam"
        );
    }

    #[cfg(feature = "egui")]
    #[test]
    fn building_click_stays_local_in_standalone_even_with_live_bridge() {
        let (mut app, requests, legacy) = building_test_app(crate::AttachMode::Standalone);
        app.update();
        assert!(requests.try_recv().is_err());
        assert!(legacy.try_recv().is_err());
        let messages: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<SpawnBuildingRequest>>()
            .drain()
            .collect();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].kind, BuildingSpawnKind::CityCenter);
        assert_eq!(messages[0].position, Vec3::new(-64.5, 47.0, 32.25));
    }

    #[cfg(feature = "egui")]
    #[test]
    fn building_click_reports_timeout_as_unknown_without_retry() {
        let (mut app, requests, _) = building_test_app(crate::AttachMode::Server);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        requests.try_recv().unwrap();
        app.world_mut()
            .resource_mut::<PendingBuildingPlacements>()
            .tickets[0]
            .2 = std::time::Instant::now() - std::time::Duration::from_secs(11);
        app.update();
        assert!(app
            .world()
            .resource::<PendingBuildingPlacements>()
            .tickets
            .is_empty());
        assert!(requests.try_recv().is_err());
        assert!(app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text.contains("result is unknown")));
    }

    #[cfg(feature = "egui")]
    #[test]
    fn building_click_reports_missing_connection_and_server_rejection() {
        let (mut app, requests, legacy) = building_test_app(crate::AttachMode::Server);
        app.world_mut().remove_resource::<LiveBridge>();
        app.update();
        assert!(requests.try_recv().is_err());
        assert!(legacy.try_recv().is_err());
        assert_eq!(
            app.world()
                .resource::<Messages<SpawnBuildingRequest>>()
                .len(),
            0
        );
        assert!(app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text.contains("connection unavailable")));

        let (mut app, requests, _) = building_test_app(crate::AttachMode::Server);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        let request: serde_json::Value =
            serde_json::from_str(&requests.try_recv().unwrap()).unwrap();
        app.world()
            .resource::<LiveBridge>()
            .client
            .test_complete_rpc(
                request["id"].as_u64().unwrap(),
                Err("Forbidden: operator required".to_string()),
            );
        app.update();
        assert!(app
            .world()
            .resource::<PendingBuildingPlacements>()
            .tickets
            .is_empty());
        assert!(app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event
                .text
                .contains("City Center placement failed: Forbidden")));
        assert!(!app
            .world()
            .resource::<crate::event_feed::EventFeed>()
            .events
            .iter()
            .any(|event| event.text.contains("Server created")));
    }

    #[test]
    fn terrain_stamp_serializes_fixed_world_coordinates_and_selected_brush() {
        let params = terrain_stamp_params(Vec3::new(-0.1, 7.0, 16.9), 7, 5);
        assert_eq!(params["x"], -civ_voxel::FIXED_SCALE);
        assert_eq!(params["y"], 7 * civ_voxel::FIXED_SCALE);
        assert_eq!(params["z"], 16 * civ_voxel::FIXED_SCALE);
        assert_eq!(params["material"], 7);
        assert_eq!(params["radius"], 5);
        assert_eq!(params["op"], "raise");
        assert_eq!(params["role"], "operator");
    }

    #[test]
    fn standalone_tools_remain_local_even_with_disconnected_bridge() {
        assert!(!server_tools_active(
            Some(&crate::AttachMode::Standalone),
            true
        ));
        assert!(server_tools_active(Some(&crate::AttachMode::Server), false));
        assert!(server_tools_active(None, true));
        assert!(!server_tools_active(None, false));
    }

    #[test]
    fn marker_diagnostic_is_limited_to_server_authoring_tools() {
        assert!(is_server_authoring_tool(
            Some(&crate::AttachMode::Server),
            SpawnTool::Terraform
        ));
        assert!(is_server_authoring_tool(
            Some(&crate::AttachMode::Server),
            SpawnTool::SpawnBuilding
        ));
        assert!(!is_server_authoring_tool(
            Some(&crate::AttachMode::Server),
            SpawnTool::Select
        ));
        assert!(!is_server_authoring_tool(
            Some(&crate::AttachMode::Standalone),
            SpawnTool::Terraform
        ));
        assert!(!is_server_authoring_tool(None, SpawnTool::Terraform));
    }

    #[test]
    fn live_terrain_raycast_uses_non_air_payload_and_negative_chunk_coordinates() {
        let mut cache = ChunkVoxelCache::new();
        let mut voxels = vec![civ_voxel::MaterialId(0); 4096];
        voxels[15 + 2 * 16 + 4 * 256] = civ_voxel::MaterialId(1);
        cache.insert(crate::encode_chunk_id(-1, 0, 0), voxels);
        let hit = raycast_to_live_voxels(&cache, Vec3::new(-0.5, 20.0, 4.5), -Vec3::Y)
            .expect("streamed solid voxel");
        assert_eq!(hit, Vec3::new(-0.5, 3.0, 4.5));
        assert!(raycast_to_live_voxels(&cache, Vec3::new(-1.5, 20.0, 4.5), -Vec3::Y).is_none());
        assert!(raycast_to_live_voxels(&ChunkVoxelCache::new(), Vec3::Y, -Vec3::Y).is_none());
    }

    #[test]
    fn authoritative_live_cache_payload_produces_attached_raycast_hit() {
        let mut scene = LiveStreamScene::default();
        let mut voxels = vec![civ_voxel::MaterialId(0); 16 * 16 * 16];
        voxels[4 + 3 * 16 + 5 * 256] = civ_voxel::MaterialId(1);
        scene
            .chunk_voxels
            .insert(crate::encode_chunk_id(0, 0, 0), voxels);

        let hit = attached_cache_hit(
            Some(&scene.chunk_voxels),
            Vec3::new(4.5, 20.0, 5.5),
            -Vec3::Y,
        )
        .expect("authoritative 4096-cell payload must be targetable");
        assert_eq!(scene.chunk_voxels.chunks().len(), 1);
        assert_eq!(hit, Vec3::new(4.5, 4.0, 5.5));
    }

    #[test]
    fn attached_marker_diagnostic_classifies_input_cache_and_ray_failures() {
        assert_eq!(
            AttachedMarkerUnavailable::PointerOverUi.summary(),
            "pointer is captured by the UI"
        );
        assert_eq!(
            AttachedMarkerUnavailable::InputUnavailable.summary(),
            "window, cursor, or camera input is unavailable"
        );
        assert_eq!(
            attached_cache_hit(None, Vec3::Y, -Vec3::Y),
            Err(AttachedMarkerUnavailable::CacheUnavailable)
        );
        assert_eq!(
            attached_cache_hit(Some(&ChunkVoxelCache::new()), Vec3::Y, -Vec3::Y),
            Err(AttachedMarkerUnavailable::CacheEmpty)
        );

        let mut cache = ChunkVoxelCache::new();
        cache.insert(
            crate::encode_chunk_id(0, 0, 0),
            vec![civ_voxel::MaterialId(0); 16 * 16 * 16],
        );
        let miss = attached_cache_hit(Some(&cache), Vec3::Y, -Vec3::Y);
        assert_eq!(
            miss,
            Err(AttachedMarkerUnavailable::RayMiss { cached_chunks: 1 })
        );
        assert_eq!(
            (AttachedMarkerUnavailable::RayMiss { cached_chunks: 1 }).summary(),
            "ray missed streamed terrain (1 cached chunks)"
        );
    }

    #[test]
    fn terrain_stamp_only_reports_success_for_real_write_receipt() {
        assert_eq!(
            terrain_stamp_feedback(Ok(serde_json::json!({"ok":true,"writes":29}))),
            "Server applied terrain disk: 29 voxel writes"
        );
        for value in [
            serde_json::json!({"ok":true}),
            serde_json::json!({"ok":true,"writes":0}),
            serde_json::json!({}),
        ] {
            assert!(terrain_stamp_feedback(Ok(value)).starts_with("Terrain stamp failed:"));
        }
        assert!(
            terrain_stamp_feedback(Err("Forbidden: operator role required".to_owned()))
                .contains("Forbidden: operator role required")
        );
    }

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
