//! Live WebSocket scene sync — voxel chunks and agent markers from `Frame3d` streams.

use std::collections::HashMap;

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;
use civ_protocol_3d::{
    agent_world_translation, map_build_provenance, AgentAppearanceFrame, BuildingDiffFrame,
    BuildingGraph, BuildingKind3d, BuildingProvenance, FacadeStyle, Frame3d, ParcelKind, WorldXZ,
    VoxelDeltaFrame,
};
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel};

use crate::bevy_render::{apply_chunk_material, mesh_buffer_to_bevy};
use crate::live_attach::{LiveAttachBridge, LiveAttachState};
use crate::minimap::{MinimapCamera, MinimapDot, MinimapRoot, MINIMAP_SIZE};
use crate::camera::CameraRig;
use crate::terrain::terrain_surface_y;
use crate::{
    agent_color_from_id, agent_label_stub, agent_scale_multiplier, chunk_distance_from_camera,
    chunk_fade_complete, decode_chunk_id, mesh_lod_level, should_render_chunk, AttachMode,
    DebugRender, AGENT_MARKER_DEPTH, AGENT_MARKER_HEIGHT, AGENT_MARKER_WIDTH,
};

const CHUNK_EDGE: usize = 16;
const CHUNK_BASE_COLOR: [f32; 3] = [0.72, 0.69, 0.62];
const LIVE_RENDER_MAX_DISTANCE: f32 = 200.0;
const AGENT_NAME_LABELS: bool = true;
const AGENT_LABEL_FONT_SIZE: f32 = 10.0;
const AGENT_LABEL_Y_OFFSET: f32 = 1.05;
const MINIMAP_LIVE_DOT: f32 = 4.0;
const MINIMAP_CAMERA_HEIGHT: f32 = 180.0;
const LIVE_FOCUS_LERP_SPEED: f32 = 2.5;
const LIVE_FOCUS_MIN_HALF_EXTENT: f32 = 32.0;
const AGENT_GROUND_Y: f32 = 0.8;
const BUILDING_GROUND_Y: f32 = 1.25;

/// Smoothed world-space centre and half-extent for live attach camera + minimap framing.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct LiveSceneFocus {
    /// World-space centre (XZ from streamed entities).
    pub centre: Vec3,
    /// Half-width of the orthographic/minimap view in world units.
    pub half_extent: f32,
}

impl Default for LiveSceneFocus {
    fn default() -> Self {
        Self {
            centre: Vec3::ZERO,
            half_extent: crate::terrain::WORLD_SIZE * 0.5,
        }
    }
}

/// Tracks spawned entities for streamed voxel chunks and agents.
#[derive(Resource)]
pub struct LiveScene {
    chunks: HashMap<u64, Entity>,
    agents: HashMap<u64, Entity>,
    buildings: HashMap<u64, Entity>,
    graph_parcels: HashMap<u64, Entity>,
    agent_materials: HashMap<u64, Handle<StandardMaterial>>,
    building_materials: HashMap<u64, Handle<StandardMaterial>>,
    graph_parcel_materials: HashMap<u64, Handle<StandardMaterial>>,
    building_provenance: BuildingProvenance,
}

impl Default for LiveScene {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            agents: HashMap::new(),
            buildings: HashMap::new(),
            graph_parcels: HashMap::new(),
            agent_materials: HashMap::new(),
            building_materials: HashMap::new(),
            graph_parcel_materials: HashMap::new(),
            building_provenance: BuildingProvenance::Procedural,
        }
    }
}

/// Shared marker meshes for streamed agents and buildings.
#[derive(Resource)]
pub struct LiveSceneAssets {
    agent_mesh: Handle<Mesh>,
    building_mesh: Handle<Mesh>,
}

#[derive(Component)]
struct ChunkTag {
    #[allow(dead_code)]
    id: ChunkId,
}

#[derive(Component)]
struct ChunkFade {
    elapsed: f32,
    base_rgb: [f32; 3],
}

impl ChunkFade {
    fn new() -> Self {
        Self {
            elapsed: 0.0,
            base_rgb: CHUNK_BASE_COLOR,
        }
    }
}

#[derive(Component)]
struct AgentTag {
    #[allow(dead_code)]
    id: u64,
}

#[derive(Component)]
struct AgentLabel;

#[derive(Component)]
struct BuildingTag {
    #[allow(dead_code)]
    id: u64,
}

#[derive(Component)]
struct GraphParcelTag {
    #[allow(dead_code)]
    id: u64,
}

/// Applies `Frame3d` voxel/agent payloads and maintains streamed scene entities.
pub struct LiveScenePlugin;

impl Plugin for LiveScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveScene>()
            .init_resource::<LiveSceneFocus>()
            .init_resource::<DebugRender>()
            .add_systems(Startup, setup_live_scene_assets)
            .add_systems(
                Update,
                (
                    apply_live_scene_frames,
                    update_live_scene_focus,
                    follow_live_scene_focus,
                    update_live_minimap_camera,
                    update_chunk_fade,
                    sync_live_minimap_dots,
                )
                    .chain(),
            );
    }
}

fn setup_live_scene_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(LiveSceneAssets {
        agent_mesh: meshes.add(Cuboid::new(
            AGENT_MARKER_WIDTH,
            AGENT_MARKER_HEIGHT,
            AGENT_MARKER_DEPTH,
        )),
        building_mesh: meshes.add(Cuboid::new(2.0, 2.5, 2.0)),
    });
}

fn apply_live_scene_frames(
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<LiveAttachState>,
    mut scene: ResMut<LiveScene>,
    debug: Res<DebugRender>,
    assets: Res<LiveSceneAssets>,
    cameras: Query<&Transform, (With<Camera3d>, Without<crate::minimap::MinimapCamera>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    let frames = bridge.client.poll();
    if frames.is_empty() {
        return;
    }

    state.connected = true;
    let eye = cameras
        .single()
        .map(|transform| transform.translation.to_array())
        .unwrap_or([8.0, 8.0, 8.0]);

    for frame in frames {
        state.tick = Some(frame.tick());
        match frame {
            Frame3d::VoxelDelta(delta) => apply_voxel_delta(
                &mut commands,
                &mut scene,
                &mut meshes,
                &mut materials,
                eye,
                debug.as_ref(),
                delta,
            ),
            Frame3d::AgentAppearance(agents) => {
                apply_agent_appearance(&mut commands, &mut scene, &mut materials, &assets, agents);
            }
            Frame3d::BuildingDiff(building) => apply_building_diff(
                &mut commands,
                &mut scene,
                &mut materials,
                &assets,
                building,
            ),
        }
    }
}

fn apply_voxel_delta(
    commands: &mut Commands,
    scene: &mut LiveScene,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    eye: [f32; 3],
    debug: &DebugRender,
    delta: VoxelDeltaFrame,
) {
    let max_dist = LIVE_RENDER_MAX_DISTANCE;

    for chunk in delta.deltas {
        let chunk_id = chunk.event.chunk_id;
        if !should_render_chunk(chunk_id, eye, max_dist) {
            if let Some(entity) = scene.chunks.remove(&chunk_id.0) {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if chunk.voxels.len() != CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE {
            continue;
        }

        let chunk_view = ChunkView {
            id: chunk.event.chunk_id,
            voxels: &chunk.voxels,
        };
        let distance = chunk_distance_from_camera(chunk.event.chunk_id, eye, CHUNK_EDGE as f32);
        let lod = LodLevel(mesh_lod_level(distance));
        let Ok(mesh_buffer) = CubicMesher::mesh_cubic(chunk_view, lod) else {
            continue;
        };
        let mesh = meshes.add(mesh_buffer_to_bevy(&mesh_buffer));
        let mut material = StandardMaterial {
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        };
        apply_chunk_material(&mut material, CHUNK_BASE_COLOR, debug.wireframe, Some(0.0));
        let material_handle = materials.add(material);
        let transform = chunk_transform(chunk.event.chunk_id);

        let entity = *scene
            .chunks
            .entry(chunk.event.chunk_id.0)
            .or_insert_with(|| {
                commands
                    .spawn((
                        ChunkTag {
                            id: chunk.event.chunk_id,
                        },
                        Transform::default(),
                    ))
                    .id()
            });
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            transform,
            ChunkFade::new(),
        ));
    }
}

fn apply_agent_appearance(
    commands: &mut Commands,
    scene: &mut LiveScene,
    materials: &mut Assets<StandardMaterial>,
    assets: &LiveSceneAssets,
    agents: AgentAppearanceFrame,
) {
    for update in agents.updates {
        let rgb = agent_color_from_id(update.agent_id);
        let scale = agent_scale_multiplier(update.scale);
        let (x, _, z) = agent_world_translation(&update, 0.0);
        let y = terrain_surface_y(x, z) + AGENT_GROUND_Y;
        let transform = Transform::from_xyz(x, y, z).with_scale(Vec3::splat(scale));

        let material_handle = scene
            .agent_materials
            .entry(update.agent_id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = Color::srgb(rgb[0], rgb[1], rgb[2]);
        }

        let entity = *scene.agents.entry(update.agent_id).or_insert_with(|| {
            let entity = commands
                .spawn(AgentTag {
                    id: update.agent_id,
                })
                .id();
            if AGENT_NAME_LABELS {
                let label = agent_label_stub(update.agent_id, None);
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text2d::new(label),
                        TextFont::from_font_size(AGENT_LABEL_FONT_SIZE),
                        TextColor(Color::srgba(0.95, 0.96, 0.98, 0.92)),
                        Transform::from_xyz(0.0, AGENT_LABEL_Y_OFFSET, 0.0),
                        AgentLabel,
                    ));
                });
            }
            entity
        });

        commands.entity(entity).insert((
            Mesh3d(assets.agent_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

fn apply_building_diff(
    commands: &mut Commands,
    scene: &mut LiveScene,
    materials: &mut Assets<StandardMaterial>,
    assets: &LiveSceneAssets,
    frame: BuildingDiffFrame,
) {
    let BuildingDiffFrame {
        provenance,
        buildings,
        graph,
        ..
    } = frame;

    scene.building_provenance = provenance;

    if let Some(graph) = graph {
        apply_building_graph(commands, scene, materials, assets, graph);
    }

    if buildings.is_empty() {
        return;
    }

    let incoming: std::collections::HashSet<u64> = buildings.iter().map(|entry| entry.id).collect();
    for (id, entity) in scene.buildings.clone() {
        if !incoming.contains(&id) {
            commands.entity(entity).despawn();
            scene.buildings.remove(&id);
            scene.building_materials.remove(&id);
        }
    }

    for entry in buildings {
        let (base_color, emissive, roughness) = building_material_style(entry.kind, provenance);
        let material_handle = scene
            .building_materials
            .entry(entry.id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color,
                    emissive: emissive.into(),
                    perceptual_roughness: roughness,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = base_color;
            material.emissive = emissive.into();
            material.perceptual_roughness = roughness;
        }

        let ground = terrain_surface_y(entry.position.x, entry.position.z);
        let transform =
            Transform::from_xyz(entry.position.x, ground + BUILDING_GROUND_Y, entry.position.z);
        let entity = *scene.buildings.entry(entry.id).or_insert_with(|| {
            commands
                .spawn(BuildingTag { id: entry.id })
                .id()
        });
        commands.entity(entity).insert((
            Mesh3d(assets.building_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

fn apply_building_graph(
    commands: &mut Commands,
    scene: &mut LiveScene,
    materials: &mut Assets<StandardMaterial>,
    assets: &LiveSceneAssets,
    graph: BuildingGraph,
) {
    let incoming: std::collections::HashSet<u64> =
        graph.parcels.iter().map(|parcel| parcel.id.0).collect();
    for (id, entity) in scene.graph_parcels.clone() {
        if !incoming.contains(&id) {
            commands.entity(entity).despawn();
            scene.graph_parcels.remove(&id);
            scene.graph_parcel_materials.remove(&id);
        }
    }

    let scale = civ_voxel::FIXED_SCALE as f32;
    for parcel in graph.parcels {
        let id = parcel.id.0;
        let provenance = graph
            .provenance
            .get(&parcel.id)
            .copied()
            .map(map_build_provenance)
            .unwrap_or(scene.building_provenance);
        let kind = parcel_kind_to_building_kind(parcel.kind);
        let mut base_color = parcel_kind_color(parcel.kind);
        if let Some(facade) = graph.facades.get(&parcel.id) {
            base_color = facade_material_color(facade);
        }
        let (styled_base, emissive, roughness) = building_material_style(kind, provenance);
        let base_color = blend_facade_base(base_color, styled_base);

        let material_handle = scene
            .graph_parcel_materials
            .entry(id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color,
                    emissive: emissive.into(),
                    perceptual_roughness: roughness,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = base_color;
            material.emissive = emissive.into();
            material.perceptual_roughness = roughness;
        }

        let anchor = WorldXZ::from_fixed_coord(&parcel.origin);
        let ground = terrain_surface_y(anchor.x, anchor.z);
        let height = parcel.size[1] as f32 / scale;
        let width = parcel.size[0] as f32 / scale;
        let depth = parcel.size[2] as f32 / scale;
        let transform = Transform::from_xyz(anchor.x, ground + height * 0.5, anchor.z)
            .with_scale(Vec3::new(width.max(0.5), height.max(0.5), depth.max(0.5)));

        let entity = *scene.graph_parcels.entry(id).or_insert_with(|| {
            commands
                .spawn(GraphParcelTag { id })
                .id()
        });
        commands.entity(entity).insert((
            Mesh3d(assets.building_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

fn parcel_kind_to_building_kind(kind: ParcelKind) -> BuildingKind3d {
    match kind {
        ParcelKind::Residential => BuildingKind3d::House,
        ParcelKind::Commercial => BuildingKind3d::Market,
        ParcelKind::Industrial => BuildingKind3d::Mine,
        ParcelKind::Civic => BuildingKind3d::CityCenter,
    }
}

fn parcel_kind_color(kind: ParcelKind) -> Color {
    building_kind_color(parcel_kind_to_building_kind(kind))
}

fn facade_material_color(facade: &FacadeStyle) -> Color {
    let material = facade
        .materials
        .first()
        .copied()
        .unwrap_or(civ_voxel::MaterialId(0));
    material_id_color(material)
}

fn material_id_color(material: civ_voxel::MaterialId) -> Color {
    let palette = [
        (0.62, 0.52, 0.44),
        (0.55, 0.42, 0.32),
        (0.58, 0.58, 0.60),
        (0.72, 0.48, 0.36),
        (0.68, 0.70, 0.74),
        (0.42, 0.58, 0.72),
        (0.78, 0.80, 0.84),
    ];
    let idx = material.0 as usize % palette.len();
    let (r, g, b) = palette[idx];
    Color::srgb(r, g, b)
}

fn blend_facade_base(facade: Color, styled: Color) -> Color {
    let f = facade.to_srgba();
    let s = styled.to_srgba();
    Color::srgb(
        f.red * 0.65 + s.red * 0.35,
        f.green * 0.65 + s.green * 0.35,
        f.blue * 0.65 + s.blue * 0.35,
    )
}

fn building_kind_color(kind: BuildingKind3d) -> Color {
    match kind {
        BuildingKind3d::Farm => Color::srgb(0.55, 0.75, 0.35),
        BuildingKind3d::Mine => Color::srgb(0.52, 0.48, 0.42),
        BuildingKind3d::Barracks => Color::srgb(0.72, 0.34, 0.34),
        BuildingKind3d::Temple => Color::srgb(0.72, 0.62, 0.88),
        BuildingKind3d::Market => Color::srgb(0.88, 0.67, 0.25),
        BuildingKind3d::House => Color::srgb(0.79, 0.59, 0.40),
        BuildingKind3d::CityCenter => Color::srgb(0.38, 0.58, 0.86),
    }
}

fn building_material_style(
    kind: BuildingKind3d,
    provenance: BuildingProvenance,
) -> (Color, Color, f32) {
    let base = building_kind_color(kind);
    match provenance {
        BuildingProvenance::Procedural => (base, Color::BLACK, 0.92),
        BuildingProvenance::Freehand => {
            let emissive = Color::srgb(
                base.to_srgba().red * 0.55,
                base.to_srgba().green * 0.55,
                base.to_srgba().blue * 0.55,
            );
            (base, emissive, 0.55)
        }
    }
}

fn building_minimap_dot_color(provenance: BuildingProvenance) -> Color {
    match provenance {
        BuildingProvenance::Procedural => Color::srgba(0.92, 0.90, 0.86, 1.0),
        BuildingProvenance::Freehand => Color::srgba(0.98, 0.62, 0.28, 1.0),
    }
}

fn update_chunk_fade(
    attach: Res<AttachMode>,
    time: Res<Time>,
    debug: Res<DebugRender>,
    mut commands: Commands,
    mut fades: Query<(Entity, &mut ChunkFade, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *attach != AttachMode::Server || debug.wireframe {
        return;
    }

    for (entity, mut fade, material) in fades.iter_mut() {
        fade.elapsed += time.delta_secs();
        if let Some(material) = materials.get_mut(&material.0) {
            apply_chunk_material(material, fade.base_rgb, false, Some(fade.elapsed));
        }
        if chunk_fade_complete(fade.elapsed) {
            commands.entity(entity).remove::<ChunkFade>();
        }
    }
}

fn sync_live_minimap_dots(
    attach: Res<AttachMode>,
    state: Res<LiveAttachState>,
    scene: Res<LiveScene>,
    focus: Res<LiveSceneFocus>,
    agents: Query<&Transform, With<AgentTag>>,
    buildings: Query<&Transform, With<BuildingTag>>,
    graph_parcels: Query<&Transform, With<GraphParcelTag>>,
    mut commands: Commands,
    roots: Query<Entity, With<MinimapRoot>>,
    existing: Query<Entity, With<MinimapDot>>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    if !scene.is_changed() && !state.is_changed() && !focus.is_changed() {
        return;
    }

    let building_dot = building_minimap_dot_color(scene.building_provenance);

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Ok(root) = roots.single() else {
        return;
    };

    commands.entity(root).with_children(|parent| {
        for raw in scene.chunks.keys() {
            let chunk_id = ChunkId(*raw);
            let (cx, _cy, cz) = decode_chunk_id(chunk_id);
            let world = Vec3::new(
                (cx as f32 + 0.5) * CHUNK_EDGE as f32,
                0.0,
                (cz as f32 + 0.5) * CHUNK_EDGE as f32,
            );
            let uv = world_to_minimap_uv_focus(world, *focus);
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    width: Val::Px(MINIMAP_LIVE_DOT),
                    height: Val::Px(MINIMAP_LIVE_DOT),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.55, 0.58, 0.62, 0.9)),
                MinimapDot,
                FocusPolicy::Pass,
            ));
        }

        for transform in &agents {
            let uv = world_to_minimap_uv_focus(transform.translation, *focus);
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    width: Val::Px(MINIMAP_LIVE_DOT),
                    height: Val::Px(MINIMAP_LIVE_DOT),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.35, 0.82, 0.95, 1.0)),
                MinimapDot,
                FocusPolicy::Pass,
            ));
        }

        for transform in &buildings {
            let uv = world_to_minimap_uv_focus(transform.translation, *focus);
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    width: Val::Px(MINIMAP_LIVE_DOT),
                    height: Val::Px(MINIMAP_LIVE_DOT),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(building_dot),
                MinimapDot,
                FocusPolicy::Pass,
            ));
        }

        for transform in &graph_parcels {
            let uv = world_to_minimap_uv_focus(transform.translation, *focus);
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_LIVE_DOT * 0.5),
                    width: Val::Px(MINIMAP_LIVE_DOT * 0.85),
                    height: Val::Px(MINIMAP_LIVE_DOT * 0.85),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(building_dot),
                MinimapDot,
                FocusPolicy::Pass,
            ));
        }
    });
}

fn world_to_minimap_uv_focus(position: Vec3, focus: LiveSceneFocus) -> Vec2 {
    let min_x = focus.centre.x - focus.half_extent;
    let max_x = focus.centre.x + focus.half_extent;
    let min_z = focus.centre.z - focus.half_extent;
    let max_z = focus.centre.z + focus.half_extent;
    let span_x = (max_x - min_x).max(f32::EPSILON);
    let span_z = (max_z - min_z).max(f32::EPSILON);
    let u = ((position.x - min_x) / span_x).clamp(0.0, 1.0);
    let v = ((position.z - min_z) / span_z).clamp(0.0, 1.0);
    Vec2::new(u, 1.0 - v)
}

fn update_live_scene_focus(
    attach: Res<AttachMode>,
    scene: Res<LiveScene>,
    agents: Query<&Transform, With<AgentTag>>,
    buildings: Query<&Transform, With<BuildingTag>>,
    graph_parcels: Query<&Transform, With<GraphParcelTag>>,
    mut focus: ResMut<LiveSceneFocus>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    let next = compute_live_scene_focus(&scene, &agents, &buildings, &graph_parcels);
    if next != *focus {
        *focus = next;
    }
}

fn compute_live_scene_focus(
    scene: &LiveScene,
    agents: &Query<&Transform, With<AgentTag>>,
    buildings: &Query<&Transform, With<BuildingTag>>,
    graph_parcels: &Query<&Transform, With<GraphParcelTag>>,
) -> LiveSceneFocus {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    let mut extend = |x: f32, z: f32| {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    };

    for raw in scene.chunks.keys() {
        let (cx, _cy, cz) = decode_chunk_id(ChunkId(*raw));
        extend(
            (cx as f32 + 0.5) * CHUNK_EDGE as f32,
            (cz as f32 + 0.5) * CHUNK_EDGE as f32,
        );
    }
    for transform in agents.iter() {
        extend(transform.translation.x, transform.translation.z);
    }
    for transform in buildings.iter() {
        extend(transform.translation.x, transform.translation.z);
    }
    for transform in graph_parcels.iter() {
        extend(transform.translation.x, transform.translation.z);
    }

    if min_x == f32::MAX {
        return LiveSceneFocus::default();
    }

    let centre = Vec3::new((min_x + max_x) * 0.5, 0.0, (min_z + max_z) * 0.5);
    let half_extent = ((max_x - min_x).max(max_z - min_z) * 0.55)
        .max(LIVE_FOCUS_MIN_HALF_EXTENT)
        .min(crate::terrain::WORLD_SIZE * 0.5);
    LiveSceneFocus {
        centre,
        half_extent,
    }
}

fn follow_live_scene_focus(
    attach: Res<AttachMode>,
    focus: Res<LiveSceneFocus>,
    time: Res<Time>,
    mut rig: ResMut<CameraRig>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    let target = Vec3::new(focus.centre.x, 30.0, focus.centre.z);
    let alpha = (time.delta_secs() * LIVE_FOCUS_LERP_SPEED).clamp(0.0, 1.0);
    rig.target = rig.target.lerp(target, alpha);
}

fn update_live_minimap_camera(
    attach: Res<AttachMode>,
    focus: Res<LiveSceneFocus>,
    mut minimap_cameras: Query<(&mut Transform, &mut Projection), With<MinimapCamera>>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    let viewport_height = (focus.half_extent * 2.2).clamp(64.0, crate::terrain::WORLD_SIZE);
    for (mut transform, mut projection) in &mut minimap_cameras {
        transform.translation = Vec3::new(focus.centre.x, MINIMAP_CAMERA_HEIGHT, focus.centre.z);
        *transform = transform.looking_at(focus.centre, Vec3::NEG_Z);
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scaling_mode = bevy::camera::ScalingMode::FixedVertical { viewport_height };
        }
    }
}

fn chunk_transform(id: ChunkId) -> Transform {
    let (x, y, z) = decode_chunk_id(id);
    Transform::from_xyz(
        x as f32 * CHUNK_EDGE as f32,
        y as f32 * CHUNK_EDGE as f32,
        z as f32 * CHUNK_EDGE as f32,
    )
}
