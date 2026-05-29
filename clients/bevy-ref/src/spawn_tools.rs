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

use bevy::math::primitives::{Capsule3d, Circle, Cuboid};
use bevy::prelude::*;

use crate::terrain::{terrain_height, terrain_surface_y, WORLD_SIZE};

/// Civilian capsule radius (world units).
const CIVILIAN_RADIUS: f32 = 1.4;
/// Civilian capsule cylinder length (between hemisphere caps).
const CIVILIAN_BODY: f32 = 3.2;
/// Half the total civilian height, used to seat the base on the terrain.
const CIVILIAN_HALF_HEIGHT: f32 = CIVILIAN_BODY * 0.5 + CIVILIAN_RADIUS;
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

/// Plugin that wires the tool state, ray hit test, and cursor marker together.
pub struct SpawnToolsPlugin;

impl Plugin for SpawnToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTool>()
            .init_resource::<SelectedEntity>()
            .init_resource::<CursorMarker>()
            .init_resource::<PointerOverUi>()
            .add_message::<SpawnCivilianRequest>()
            .add_message::<SpawnBuildingRequest>()
            .add_message::<SelectEntityRequest>()
            .add_message::<DestroyEntityRequest>()
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
                apply_spawn_requests,
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
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    over_ui: Res<PointerOverUi>,
    mut marker: ResMut<CursorMarker>,
) {
    if over_ui.0 {
        marker.visible = false;
        marker.position = None;
        return;
    }
    let hit = cursor_terrain_hit(&windows, &cameras);
    marker.position = hit;
    marker.visible = hit.is_some();
}

/// Resolve the cursor's terrain hit (world space), if the window, camera, and
/// ray all produce a valid terrain intersection this frame.
fn cursor_terrain_hit(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    raycast_to_terrain(ray.origin, ray.direction.as_vec3())
}

/// Translate a left-click (when not over the HUD) into the active tool's
/// request message, using the same-frame cursor hit.
fn handle_spawn_tool_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    active: Res<ActiveTool>,
    over_ui: Res<PointerOverUi>,
    marker: Res<CursorMarker>,
    mut spawn_civilian: MessageWriter<SpawnCivilianRequest>,
    mut spawn_building: MessageWriter<SpawnBuildingRequest>,
    mut select_entity: MessageWriter<SelectEntityRequest>,
    mut destroy_entity: MessageWriter<DestroyEntityRequest>,
) {
    if over_ui.0 || !buttons.just_pressed(MouseButton::Left) {
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
            spawn_building.write(SpawnBuildingRequest { position });
        }
        SpawnTool::Terraform => {}
        SpawnTool::Destroy => {
            destroy_entity.write(DestroyEntityRequest { position });
        }
    }
}

/// Spawn locally-owned, terrain-seated, non-neon actors in response to the
/// spawn requests produced this frame.
fn apply_spawn_requests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut civilians: MessageReader<SpawnCivilianRequest>,
    mut buildings: MessageReader<SpawnBuildingRequest>,
) {
    for request in civilians.read() {
        spawn_civilian_entity(&mut commands, &mut meshes, &mut materials, request.position);
    }
    for request in buildings.read() {
        spawn_building_entity(&mut commands, &mut meshes, &mut materials, request.position);
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
fn terrain_error(point: Vec3) -> f32 {
    terrain_height(point.x + WORLD_SIZE * 0.5, point.z + WORLD_SIZE * 0.5) - point.y
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
}
