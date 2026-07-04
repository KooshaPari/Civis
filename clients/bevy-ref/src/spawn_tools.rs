//! WorldBox-style spawn tools for the Bevy reference client.
//!
//! This module owns the click-to-terrain hit test, active tool state, cursor
//! marker, and local spawn/selection/destruction behavior.
//!
//! Two hard-won correctness rules govern everything here:
//!
//! 1. **Egui owns the pointer first.** Every system that interprets a left
//!    click as a *world* interaction MUST early-return when egui wants the
//!    pointer (a HUD button / panel is under the cursor). Otherwise clicking a
//!    toolbar button both selects the tool *and* fires a world action at the
//!    terrain behind the panel — which the user sees as the camera/selection
//!    "teleporting" away. The check is centralised in [`PointerOverUi`].
//! 2. **Spawned actors are seated and non-neon.** Civilians/buildings created
//!    by these tools sit on the terrain surface (`y = surface + half_height`)
//!    and use a saturated faction colour with near-zero emissive so they read
//!    as solid actors, not hovering neon pills.

#[cfg(feature = "models")]
use crate::gltf_models::{actor_scene, building_scene, ModelOrPrimitive};
use bevy::math::primitives::{Capsule3d, Circle, Cuboid};
use bevy::prelude::*;
use civ_agents::ActorVisualKind;

use crate::minimap::MinimapCamera;
use crate::terrain::{terrain_height, terrain_surface_y, WORLD_SIZE};
#[cfg(feature = "voxel")]
use crate::voxel_sim::VoxelSimState;
#[cfg(feature = "voxel")]
use civ_voxel::material::AIR;

/// Civilian capsule radius (world units).
const CIVILIAN_RADIUS: f32 = 1.4;
/// Civilian capsule cylinder length (between hemisphere caps).
const CIVILIAN_BODY: f32 = 3.2;
/// Half the total civilian height, used to seat the base on the terrain.
const CIVILIAN_HALF_HEIGHT: f32 = CIVILIAN_BODY * 0.5 + CIVILIAN_RADIUS;
// Match sim_bridge: voxel world is ~256 units tall, so a ~1.8m glb must be
// scaled up to read against the terrain (sub-pixel mesh-scale bug).
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
/// Building cuboid full extents (x, y, z).
const BUILDING_EXTENTS: Vec3 = Vec3::new(7.0, 12.0, 7.0);
/// Half the building height, used to seat the base on the terrain.
const BUILDING_HALF_HEIGHT: f32 = BUILDING_EXTENTS.y * 0.5;

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
    /// Drag-to-draw a surfaced road along a desire path (`RoadKind::Road`).
    Road,
    /// Drag-to-draw a foot trail (`RoadKind::Trail`).
    Trail,
    /// Drag-to-draw a high-throughput highway (`RoadKind::Highway`).
    Highway,
    /// Drag-to-draw a water-spanning bridge (`RoadKind::Bridge`).
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
    /// Paint the selected material into the voxel grid (drag-to-paint). The
    /// actual paint is performed by `material_brush_ui::paint_into_voxel_grid`,
    /// gated on `MaterialPaintArmed`; this variant is what arms it.
    PaintMaterial,
}

impl SpawnTool {
    /// True for the drag-to-draw road family (`Road`/`Trail`/`Highway`/`Bridge`).
    #[must_use]
    pub fn is_road_draw(self) -> bool {
        matches!(
            self,
            SpawnTool::Road | SpawnTool::Trail | SpawnTool::Highway | SpawnTool::Bridge
        )
    }

    /// True for click-to-place structure tools that seat a cuboid on terrain.
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

/// Currently active tool (mutated by the HUD tool palette, read here).
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

/// Whether egui currently owns the pointer (HUD button/panel under cursor).
///
/// Recomputed every frame *before* the world-click systems. When `true`, all
/// world interaction (cursor raycast, spawn/select/destroy) is suppressed so a
/// HUD click never falls through to the terrain.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PointerOverUi(pub bool);

/// Cursor state for the terrain hit marker.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct CursorMarker {
    /// World-space position on the terrain surface.
    pub position: Option<Vec3>,
    /// Whether the marker should be visible.
    pub visible: bool,
}

/// Marker for entities created/owned by the sandbox spawn tools. Select and
/// Destroy operate exclusively over these so HUD/camera/decoration entities are
/// never picked.
#[derive(Component, Debug, Clone, Copy)]
pub struct SandboxEntity;

/// Request to spawn a civilian at the clicked point.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SpawnCivilianRequest {
    /// World-space click position (terrain-seated).
    pub position: Vec3,
    /// Which glTF rig the sim + renderer should use (herd tool vs organism).
    pub model_kind: ActorVisualKind,
}

/// Request to spawn a building at the clicked point.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SpawnBuildingRequest {
    /// World-space click position (terrain-seated).
    pub position: Vec3,
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
    /// Terrain-surface points collected so far this stroke (world space).
    pub points: Vec<Vec3>,
    /// The road tool that started the stroke.
    pub tool: Option<SpawnTool>,
}

/// Request to lay a connected road polyline (drag-to-draw release).
#[derive(Message, Debug, Clone, PartialEq)]
pub struct PlaceRoadRequest {
    /// Ordered terrain points; consecutive pairs become undirected segments.
    pub points: Vec<Vec3>,
    /// Which road-family tool authored the stroke.
    pub kind: SpawnTool,
}

/// Request to seat a structure or vehicle actor on the terrain at a click.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct PlaceStructureRequest {
    /// World-space click position (terrain-seated by the handler).
    pub position: Vec3,
    /// Which structure/vehicle tool authored the placement.
    pub kind: SpawnTool,
}

/// Plugin that wires the tool state, ray hit test, and cursor marker together.
pub struct SpawnToolsPlugin;

impl Plugin for SpawnToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTool>()
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

        // The egui pointer gate runs first so the click systems see a current
        // value. Without the egui feature it stays `false` (no HUD to block).
        #[cfg(feature = "egui")]
        app.add_systems(Update, update_pointer_over_ui);

        app.add_systems(
            Update,
            (
                update_cursor_marker,
                handle_spawn_tool_clicks,
                accumulate_road_draft,
                apply_spawn_requests,
                apply_place_road_requests,
                apply_place_structure_requests,
                resolve_selection_and_destruction,
                apply_cursor_marker_visuals,
            )
                .chain(),
        );
    }
}

/// Recompute [`PointerOverUi`] from the egui context each frame.
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

/// Spawn the flat ground ring used as the terrain cursor marker.
fn spawn_cursor_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ring_mesh = Mesh::from(Circle::new(1.6));
    // Soft, low-intensity ring — readable but not a neon flare.
    let material = StandardMaterial {
        base_color: Color::srgba(0.95, 0.85, 0.40, 0.45),
        emissive: LinearRgba::rgb(0.25, 0.20, 0.05),
        alpha_mode: AlphaMode::Blend,
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

/// Raycast the cursor onto the terrain, updating [`CursorMarker`]. Suppressed
/// (hidden) whenever egui owns the pointer so the ring vanishes over the HUD.
fn update_cursor_marker(
    windows: Query<&Window>,
    // MUST exclude the minimap camera: with two `Camera3d` entities a plain
    // `With<Camera3d>` query is ambiguous and `single()` returns `Err`, so the
    // raycast silently produced `None` every frame and *all* map clicks no-oped.
    cameras: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    over_ui: Res<PointerOverUi>,
    mut marker: ResMut<CursorMarker>,
    // Present only when `VoxelSimPlugin` is active (the `voxel` feature). When
    // it is, clicks must raycast the VISIBLE voxel surface, not the analytic
    // heightmap (which is no longer rendered under `voxel`).
    #[cfg(feature = "voxel")] voxel: Option<Res<VoxelSimState>>,
) {
    if over_ui.0 {
        marker.visible = false;
        marker.position = None;
        return;
    }
    let had_hit = marker.position.is_some();
    let hit = cursor_terrain_hit(
        &windows,
        &cameras,
        #[cfg(feature = "voxel")]
        voxel.as_deref(),
    );
    log_hit_transition(had_hit, hit.is_some());
    marker.position = hit;
    marker.visible = hit.is_some();
}

/// Emit a one-shot diagnostic when the terrain hit transitions present<->absent
/// so stderr proves whether the raycast resolves (without spamming per frame).
fn log_hit_transition(prev: bool, now: bool) {
    if prev != now {
        if now {
            info!("[tools] cursor terrain hit ACQUIRED");
        } else {
            info!("[tools] cursor terrain hit LOST (no ray/terrain intersection)");
        }
    }
}

/// Resolve the cursor's terrain hit (world space), if the window, camera, and
/// ray all produce a valid terrain intersection this frame.
fn cursor_terrain_hit(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<MinimapCamera>)>,
    #[cfg(feature = "voxel")] voxel: Option<&VoxelSimState>,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    // Under the voxel feature the chunk meshes are the visible world, so the
    // click must hit a real voxel cell. Fall back to the heightmap analytic
    // surface only when no voxel grid is loaded (heightmap sandbox build).
    #[cfg(feature = "voxel")]
    if let Some(state) = voxel {
        if !state.grid.cells.is_empty() {
            return raycast_to_voxel(&state.grid, ray.origin, ray.direction.as_vec3());
        }
    }
    raycast_to_terrain(ray.origin, ray.direction.as_vec3())
}

/// March a ray through the dense voxel grid (world-space == grid coords; chunk
/// meshes are spawned at raw cell offsets with no centring) and return the
/// surface point of the first non-air cell hit, or `None` if the ray misses.
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
    // Fine fixed-step DDA: 0.25-cell steps keep thin surfaces from being
    // skipped while staying cheap for a single click-frame query.
    let step = 0.25_f32;
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
            // Return the entry point on the cell (where the ray first touched
            // solid) so spawned actors seat on the surface, not inside it.
            return Some(p);
        }
        t += step;
    }
    None
}

/// Translate a left-click (when not over the HUD) into the active tool's
/// request message, using the same-frame cursor hit.
fn handle_spawn_tool_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    active: Res<ActiveTool>,
    over_ui: Res<PointerOverUi>,
    marker: Res<CursorMarker>,
    #[cfg(feature = "egui")] sub: Res<crate::tool_categories::ActiveSubTool>,
    #[cfg(feature = "voxel")] mut brush: ResMut<crate::terraform_brush::BrushSettings>,
    mut spawn_civilian: MessageWriter<SpawnCivilianRequest>,
    mut spawn_building: MessageWriter<SpawnBuildingRequest>,
    mut select_entity: MessageWriter<SelectEntityRequest>,
    mut destroy_entity: MessageWriter<DestroyEntityRequest>,
    mut place_structure: MessageWriter<PlaceStructureRequest>,
) {
    // Road-family tools are drag-to-draw; handled by `accumulate_road_draft`.
    if active.tool.is_road_draw() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    // A left click happened — log every gate so stderr proves the path.
    if over_ui.0 {
        info!("[tools] left-click BLOCKED by egui (pointer over HUD)");
        return;
    }
    let Some(position) = marker.position else {
        info!(
            "[tools] left-click passed egui, tool={:?}, but NO terrain hit this frame",
            active.tool
        );
        return;
    };
    info!(
        "[tools] left-click ACTION tool={:?} at {:?}",
        active.tool, position
    );

    match active.tool {
        SpawnTool::Select => {
            select_entity.write(SelectEntityRequest { position });
        }
        SpawnTool::SpawnCivilian => {
            #[cfg(feature = "egui")]
            let model_kind = match sub.current {
                crate::tool_categories::SubTool::SpawnHerd => ActorVisualKind::Herd,
                _ => ActorVisualKind::Humanoid,
            };
            #[cfg(not(feature = "egui"))]
            let model_kind = ActorVisualKind::Humanoid;
            spawn_civilian.write(SpawnCivilianRequest {
                position,
                model_kind,
            });
        }
        SpawnTool::SpawnBuilding => {
            spawn_building.write(SpawnBuildingRequest { position });
        }
        SpawnTool::Terraform => {
            #[cfg(all(feature = "voxel", feature = "egui"))]
            {
                let op = match sub.current {
                    crate::tool_categories::SubTool::Raise => {
                        crate::terraform_brush::BrushOp::Raise
                    }
                    crate::tool_categories::SubTool::Lower => {
                        crate::terraform_brush::BrushOp::Lower
                    }
                    crate::tool_categories::SubTool::Flatten => {
                        crate::terraform_brush::BrushOp::Flatten
                    }
                    crate::tool_categories::SubTool::PaintBiome => {
                        crate::terraform_brush::BrushOp::DropBiome
                    }
                    _ => brush.op,
                };
                brush.select_op(op);
            }
        }
        SpawnTool::PaintMaterial => {
            // Painting is handled by the material brush's own per-frame system
            // (gated on `MaterialPaintArmed`); nothing to dispatch on click here.
        }
        SpawnTool::Destroy => {
            destroy_entity.write(DestroyEntityRequest { position });
        }
        SpawnTool::House
        | SpawnTool::Farm
        | SpawnTool::Workshop
        | SpawnTool::Market
        | SpawnTool::Wall
        | SpawnTool::Vehicle => {
            place_structure.write(PlaceStructureRequest {
                position,
                kind: active.tool,
            });
        }
        SpawnTool::Road | SpawnTool::Trail | SpawnTool::Highway | SpawnTool::Bridge => {
            // Unreachable: filtered by the `is_road_draw` early-return above.
        }
    }
}

/// Drag-to-draw accumulator for the road family. While a road tool is active and
/// the left button is held, append the current terrain hit (deduplicated). On
/// release, emit a [`PlaceRoadRequest`] for the polyline and reset the draft.
fn accumulate_road_draft(
    buttons: Res<ButtonInput<MouseButton>>,
    active: Res<ActiveTool>,
    over_ui: Res<PointerOverUi>,
    marker: Res<CursorMarker>,
    mut draft: ResMut<RoadDraft>,
    mut place_road: MessageWriter<PlaceRoadRequest>,
) {
    if !active.tool.is_road_draw() {
        if !draft.points.is_empty() {
            draft.points.clear();
            draft.tool = None;
        }
        return;
    }

    if buttons.pressed(MouseButton::Left) && !over_ui.0 {
        if draft.tool.is_none() {
            draft.tool = Some(active.tool);
        }
        if let Some(p) = marker.position {
            let keep = draft
                .points
                .last()
                .map_or(true, |last| last.distance_squared(p) > 2.25);
            if keep {
                draft.points.push(p);
            }
        }
    }

    if buttons.just_released(MouseButton::Left) {
        if draft.points.len() >= 2 {
            let kind = draft.tool.unwrap_or(active.tool);
            place_road.write(PlaceRoadRequest {
                points: std::mem::take(&mut draft.points),
                kind,
            });
            info!("[tools] road stroke released -> PlaceRoadRequest ({kind:?})");
        } else {
            draft.points.clear();
        }
        draft.tool = None;
    }
}

/// Spawn locally-owned, terrain-seated, non-neon actors in response to the
/// spawn requests produced this frame.
fn apply_spawn_requests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "models")] models: Option<Res<crate::gltf_models::GameModels>>,
    mut civilians: MessageReader<SpawnCivilianRequest>,
    mut buildings: MessageReader<SpawnBuildingRequest>,
) {
    for request in civilians.read() {
        #[cfg(feature = "models")]
        {
            let mut spawned = false;
            if let Some(models) = models
                .as_ref()
                .and_then(|models| model_root_for_spawn(models, request.model_kind))
            {
                let seated = seat_on_terrain(request.position, CIVILIAN_HALF_HEIGHT);
                commands.spawn((
                    SandboxEntity,
                    models,
                    Transform::from_translation(seated)
                        .with_scale(Vec3::splat(model_scale_for_kind(request.model_kind))),
                ));
                spawned = true;
            }
            if !spawned {
                spawn_civilian_entity(&mut commands, &mut meshes, &mut materials, request.position);
            }
        }
        #[cfg(not(feature = "models"))]
        spawn_civilian_entity(&mut commands, &mut meshes, &mut materials, request.position);
        info!("[tools] SPAWNED civilian at {:?}", request.position);
    }
    for request in buildings.read() {
        #[cfg(feature = "models")]
        {
            let mut spawned = false;
            if let Some(models) = models
                .as_ref()
                .and_then(|models| building_root_for_spawn(models))
            {
                let seated = seat_on_terrain(request.position, BUILDING_HALF_HEIGHT);
                commands.spawn((
                    SandboxEntity,
                    models,
                    Transform::from_translation(seated)
                        .with_scale(Vec3::splat(BUILDING_MODEL_SCALE)),
                ));
                spawned = true;
            }
            if !spawned {
                spawn_building_entity(&mut commands, &mut meshes, &mut materials, request.position);
            }
        }
        #[cfg(not(feature = "models"))]
        spawn_building_entity(&mut commands, &mut meshes, &mut materials, request.position);
        info!("[tools] SPAWNED building at {:?}", request.position);
    }
}

#[cfg(feature = "models")]
fn model_scale_for_kind(kind: ActorVisualKind) -> f32 {
    match kind {
        ActorVisualKind::Humanoid => CIVILIAN_MODEL_SCALE,
        ActorVisualKind::Herd => HERD_MODEL_SCALE,
    }
}

#[cfg(feature = "models")]
fn model_root_for_spawn(
    models: &crate::gltf_models::GameModels,
    kind: ActorVisualKind,
) -> Option<SceneRoot> {
    match actor_scene(models, kind, 0) {
        ModelOrPrimitive::Model(root) => Some(root),
        ModelOrPrimitive::Primitive => None,
    }
}

#[cfg(feature = "models")]
fn building_root_for_spawn(models: &crate::gltf_models::GameModels) -> Option<SceneRoot> {
    match building_scene(models) {
        ModelOrPrimitive::Model(root) => Some(root),
        ModelOrPrimitive::Primitive => None,
    }
}

/// Shared data tag carried by every user-placed infra actor (road segment,
/// structure, or vehicle). Records which [`SpawnTool`] authored it so the
/// renderer / economy can treat user- and sim-placed infra identically while
/// still being able to style or query by kind.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedInfra {
    /// The tool that authored this actor.
    pub kind: SpawnTool,
}

/// Thin road-segment cuboid height (world units) — a low seated slab.
const ROAD_SEGMENT_THICKNESS: f32 = 0.6;

/// Half-width (world units) of a road slab by tool.
fn road_half_width(kind: SpawnTool) -> f32 {
    match kind {
        SpawnTool::Trail => 1.0,
        SpawnTool::Road => 1.8,
        SpawnTool::Highway => 3.0,
        SpawnTool::Bridge => 2.2,
        _ => 1.8,
    }
}

/// Muted, non-neon surface colour per road tool.
fn road_color(kind: SpawnTool) -> Color {
    match kind {
        SpawnTool::Trail => Color::srgb(0.45, 0.36, 0.24),
        SpawnTool::Road => Color::srgb(0.32, 0.32, 0.34),
        SpawnTool::Highway => Color::srgb(0.22, 0.22, 0.25),
        SpawnTool::Bridge => Color::srgb(0.40, 0.30, 0.20),
        _ => Color::srgb(0.32, 0.32, 0.34),
    }
}

/// Lay a drawn road polyline as a chain of thin seated cuboids (primitive-mesh
/// fallback until the Asset Lead's road meshes land). Each consecutive point
/// pair becomes one oriented slab tagged [`PlacedInfra`].
fn apply_place_road_requests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut roads: MessageReader<PlaceRoadRequest>,
) {
    for request in roads.read() {
        if request.points.len() < 2 {
            continue;
        }
        let half_w = road_half_width(request.kind);
        let color = road_color(request.kind);
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::rgb(0.01, 0.01, 0.01),
            perceptual_roughness: 0.95,
            ..default()
        });
        let mut segments = 0_usize;
        for pair in request.points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let delta = b - a;
            let len = delta.length();
            if len < 1e-3 {
                continue;
            }
            let mesh = meshes.add(Mesh::from(Cuboid::new(
                len,
                ROAD_SEGMENT_THICKNESS,
                half_w * 2.0,
            )));
            let mid = (a + b) * 0.5;
            let seated = seat_on_terrain(mid, ROAD_SEGMENT_THICKNESS * 0.5);
            let yaw = (-delta.z).atan2(delta.x);
            commands.spawn((
                SandboxEntity,
                PlacedInfra { kind: request.kind },
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(seated).with_rotation(Quat::from_rotation_y(yaw)),
            ));
            segments += 1;
        }
        info!(
            "[tools] PLACED road {:?}: {segments} segment(s) over {} point(s)",
            request.kind,
            request.points.len()
        );
    }
}

/// Seat a structure or vehicle actor on the terrain in response to a click.
fn apply_place_structure_requests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut requests: MessageReader<PlaceStructureRequest>,
) {
    for request in requests.read() {
        let (extents, color) = structure_profile(request.kind);
        let half_height = extents.y * 0.5;
        let mesh = meshes.add(Mesh::from(Cuboid::new(extents.x, extents.y, extents.z)));
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::rgb(0.01, 0.01, 0.01),
            perceptual_roughness: 0.85,
            ..default()
        });
        let seated = seat_on_terrain(request.position, half_height);
        commands.spawn((
            SandboxEntity,
            PlacedInfra { kind: request.kind },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(seated),
        ));
        info!(
            "[tools] PLACED {:?} at {:?}",
            request.kind, request.position
        );
    }
}

/// Cuboid extents + muted colour for each click-placed structure/vehicle kind.
fn structure_profile(kind: SpawnTool) -> (Vec3, Color) {
    match kind {
        SpawnTool::House => (Vec3::new(6.0, 8.0, 6.0), Color::srgb(0.74, 0.62, 0.46)),
        SpawnTool::Farm => (Vec3::new(10.0, 1.2, 10.0), Color::srgb(0.45, 0.55, 0.25)),
        SpawnTool::Workshop => (Vec3::new(8.0, 7.0, 8.0), Color::srgb(0.55, 0.45, 0.40)),
        SpawnTool::Market => (Vec3::new(9.0, 5.0, 9.0), Color::srgb(0.70, 0.55, 0.30)),
        SpawnTool::Wall => (Vec3::new(6.0, 6.0, 1.5), Color::srgb(0.50, 0.50, 0.52)),
        SpawnTool::Vehicle => (Vec3::new(3.0, 2.2, 1.6), Color::srgb(0.35, 0.30, 0.28)),
        _ => (BUILDING_EXTENTS, Color::srgb(0.74, 0.62, 0.46)),
    }
}

/// Spawn one visible civilian capsule seated on the terrain at `hit`.
fn spawn_civilian_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    hit: Vec3,
) {
    let mesh = meshes.add(Mesh::from(Capsule3d::new(CIVILIAN_RADIUS, CIVILIAN_BODY)));
    // Saturated but natural faction colour; emissive near zero so it reads as a
    // solid actor, not a glowing pill.
    let base = sandbox_faction_color(hit);
    let material = materials.add(StandardMaterial {
        base_color: base,
        emissive: LinearRgba::rgb(0.02, 0.02, 0.02),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });
    let seated = seat_on_terrain(hit, CIVILIAN_HALF_HEIGHT);
    commands.spawn((
        SandboxEntity,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(seated),
    ));
}

/// Spawn one visible building cuboid seated on the terrain at `hit`.
fn spawn_building_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    hit: Vec3,
) {
    let mesh = meshes.add(Mesh::from(Cuboid::new(
        BUILDING_EXTENTS.x,
        BUILDING_EXTENTS.y,
        BUILDING_EXTENTS.z,
    )));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.74, 0.62, 0.46),
        emissive: LinearRgba::rgb(0.01, 0.01, 0.01),
        perceptual_roughness: 0.85,
        ..default()
    });
    let seated = seat_on_terrain(hit, BUILDING_HALF_HEIGHT);
    commands.spawn((
        SandboxEntity,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(seated),
    ));
}

/// Lift a terrain hit so an actor of `half_height` rests its base on the
/// surface (re-samples the surface to be robust to the marker's small Y lift).
fn seat_on_terrain(hit: Vec3, half_height: f32) -> Vec3 {
    let surface = terrain_surface_y(hit.x + WORLD_SIZE * 0.5, hit.z + WORLD_SIZE * 0.5);
    Vec3::new(hit.x, surface + half_height, hit.z)
}

/// Pick a stable, saturated, non-neon faction colour from the hit position.
fn sandbox_faction_color(hit: Vec3) -> Color {
    let h = ((hit.x * 7.3 + hit.z * 3.1).rem_euclid(360.0)).abs();
    Color::hsl(h, 0.7, 0.5)
}

/// Apply select / destroy requests against the sandbox entities only.
fn resolve_selection_and_destruction(
    mut commands: Commands,
    mut selected: ResMut<SelectedEntity>,
    mut select_entity: MessageReader<SelectEntityRequest>,
    mut destroy_entity: MessageReader<DestroyEntityRequest>,
    entities: Query<(Entity, &GlobalTransform), With<SandboxEntity>>,
) {
    for request in select_entity.read() {
        selected.0 = nearest_entity(request.position, &entities);
        info!("[tools] SELECT -> {:?}", selected.0);
    }

    for request in destroy_entity.read() {
        if let Some(entity) = nearest_entity(request.position, &entities) {
            if selected.0 == Some(entity) {
                selected.0 = None;
            }
            commands.entity(entity).despawn();
            info!("[tools] DESTROYED {:?}", entity);
        } else {
            info!("[tools] DESTROY request but no sandbox entity near hit");
        }
    }
}

/// Move/show the cursor ring at the current hit; hide it when there is none.
fn apply_cursor_marker_visuals(
    marker: Res<CursorMarker>,
    mut query: Query<(&mut Transform, &mut Visibility), With<SpawnCursorMarker>>,
) {
    let Ok((mut transform, mut visibility)) = query.single_mut() else {
        return;
    };
    match (marker.visible, marker.position) {
        (true, Some(position)) => {
            transform.translation = position + Vec3::Y * 0.1;
            transform.scale = Vec3::splat(1.0);
            *visibility = Visibility::Visible;
        }
        _ => *visibility = Visibility::Hidden,
    }
}

/// March a ray from `origin` along `direction`, returning the first terrain
/// surface crossing (refined by bisection) within the world bounds.
fn raycast_to_terrain(origin: Vec3, direction: Vec3) -> Option<Vec3> {
    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let bounds = WORLD_SIZE * 0.5;
    let max_distance = 2_000.0;
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

/// Signed distance of `point` above the terrain (positive = above surface).
///
/// Must be `point.y - surface` to match the documented convention and the hit
/// test in [`raycast_to_terrain`] (`err <= 0 && prev_err > 0` = a downward ray
/// crossing from above to below the surface). The previous `surface - point.y`
/// inverted the sign, so a sky-down ray never registered a crossing and the
/// cast always returned `None`.
fn terrain_error(point: Vec3) -> f32 {
    point.y - terrain_height(point.x + WORLD_SIZE * 0.5, point.z + WORLD_SIZE * 0.5)
}

/// Bisect between an above-surface and below-surface sample to the crossing.
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

/// Nearest sandbox entity to `position` by squared distance, if any.
fn nearest_entity(
    position: Vec3,
    entities: &Query<(Entity, &GlobalTransform), With<SandboxEntity>>,
) -> Option<Entity> {
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
    fn terrain_raycast_hits_centre_near_height() {
        let origin = Vec3::new(0.0, 300.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let hit = raycast_to_terrain(origin, dir).expect("terrain hit");
        assert!(hit.y >= 0.0);
        assert!(hit.y <= crate::terrain::HEIGHT_SCALE);
        // Straight-down ray must land at the column it was fired through.
        assert!(hit.x.abs() < 1e-2 && hit.z.abs() < 1e-2);
    }

    #[test]
    fn raycast_rejects_zero_direction() {
        assert!(raycast_to_terrain(Vec3::new(0.0, 100.0, 0.0), Vec3::ZERO).is_none());
    }

    #[test]
    fn raycast_misses_when_pointing_away_from_terrain() {
        // Ray climbing straight up never crosses the surface from above.
        let hit = raycast_to_terrain(Vec3::new(0.0, 300.0, 0.0), Vec3::Y);
        assert!(hit.is_none());
    }

    #[test]
    fn seat_on_terrain_lifts_base_to_surface() {
        let hit = Vec3::new(0.0, 0.0, 0.0);
        let seated = seat_on_terrain(hit, CIVILIAN_HALF_HEIGHT);
        let surface = terrain_surface_y(WORLD_SIZE * 0.5, WORLD_SIZE * 0.5);
        assert!((seated.y - (surface + CIVILIAN_HALF_HEIGHT)).abs() < 1e-3);
        assert_eq!(seated.x, hit.x);
        assert_eq!(seated.z, hit.z);
    }

    #[test]
    fn faction_color_is_not_neon() {
        // HSL lightness 0.5 keeps colours saturated but well short of white,
        // and the type is a normal sRGB colour (no emissive component here).
        let c = sandbox_faction_color(Vec3::new(12.0, 0.0, -8.0)).to_srgba();
        assert!(c.red <= 1.0 && c.green <= 1.0 && c.blue <= 1.0);
        let max = c.red.max(c.green).max(c.blue);
        assert!(max <= 0.86, "channel too hot: {max}");
    }

    #[test]
    fn road_tools_are_drag_draw_others_are_not() {
        for t in [
            SpawnTool::Road,
            SpawnTool::Trail,
            SpawnTool::Highway,
            SpawnTool::Bridge,
        ] {
            assert!(t.is_road_draw(), "{t:?} should be drag-draw");
        }
        for t in [
            SpawnTool::Select,
            SpawnTool::House,
            SpawnTool::Vehicle,
            SpawnTool::Market,
        ] {
            assert!(!t.is_road_draw(), "{t:?} should not be drag-draw");
        }
    }

    #[test]
    fn road_draw_classification() {
        assert!(SpawnTool::House.is_structure());
        assert!(SpawnTool::Wall.is_structure());
        assert!(!SpawnTool::Vehicle.is_structure());
        assert!(!SpawnTool::Road.is_structure());
    }

    #[test]
    fn road_width_increases_up_the_ladder() {
        assert!(road_half_width(SpawnTool::Trail) < road_half_width(SpawnTool::Road));
        assert!(road_half_width(SpawnTool::Road) < road_half_width(SpawnTool::Highway));
    }

    #[test]
    fn road_and_structure_colors_are_not_neon() {
        for c in [
            road_color(SpawnTool::Trail),
            road_color(SpawnTool::Highway),
            structure_profile(SpawnTool::House).1,
            structure_profile(SpawnTool::Vehicle).1,
        ] {
            let s = c.to_srgba();
            let max = s.red.max(s.green).max(s.blue);
            assert!(max <= 0.86, "channel too hot: {max}");
        }
    }

    #[test]
    fn structure_profiles_seat_above_zero_height() {
        for t in [
            SpawnTool::House,
            SpawnTool::Farm,
            SpawnTool::Workshop,
            SpawnTool::Market,
            SpawnTool::Wall,
            SpawnTool::Vehicle,
        ] {
            let (extents, _) = structure_profile(t);
            assert!(extents.x > 0.0 && extents.y > 0.0 && extents.z > 0.0);
        }
    }
}
