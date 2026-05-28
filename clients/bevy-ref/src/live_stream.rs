//! Shared `Frame3d` entity sync for live attach clients (`live_scene`, `bevy_window`).

use std::collections::{HashMap, HashSet};

use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};
use civ_protocol_3d::{
    agent_world_translation, map_build_provenance, AgentAppearanceFrame, BuildingDiffFrame,
    BuildingGraph, BuildingKind3d, BuildingProvenance, FacadeStyle, ParcelKind, VoxelDeltaFrame,
    WorldXZ,
};
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId};

use crate::bevy_render::{apply_chunk_material, mesh_buffer_to_bevy};
use crate::live_ground::{live_ground_y, ChunkVoxelCache};
use crate::{
    agent_color_from_id, agent_label_stub, agent_scale_multiplier, chunk_distance_from_camera,
    decode_chunk_id, mesh_lod_level, should_render_chunk, DebugRender, AGENT_MARKER_DEPTH,
    AGENT_MARKER_HEIGHT, AGENT_MARKER_WIDTH,
};

/// Chunk edge length in voxels (matches kernel).
pub const LIVE_CHUNK_EDGE: usize = 16;
/// Default chunk albedo for streamed meshes.
pub const LIVE_CHUNK_BASE_COLOR: [f32; 3] = [0.72, 0.69, 0.62];

/// Vertical offset for streamed agent markers above ground.
pub const AGENT_GROUND_Y: f32 = 0.8;
/// Vertical offset for streamed building markers above ground.
pub const BUILDING_GROUND_Y: f32 = 1.25;
/// Font size for optional agent name labels.
pub const AGENT_LABEL_FONT_SIZE: f32 = 10.0;
/// Local Y offset for agent name labels.
pub const AGENT_LABEL_Y_OFFSET: f32 = 1.05;

/// Distance culling parameters for streamed chunk meshes.
#[derive(Clone, Copy, Debug)]
pub struct StreamCulling {
    /// Camera position in world space.
    pub eye: [f32; 3],
    /// Maximum render distance in world units.
    pub max_distance: f32,
}

/// Marker for a streamed voxel chunk entity.
#[derive(Component)]
pub struct LiveChunkTag {
    /// Chunk id from the live bridge.
    pub id: ChunkId,
}

/// Fade-in state for newly spawned chunk meshes.
#[derive(Component)]
pub struct LiveChunkFade {
    /// Elapsed fade time in seconds.
    pub elapsed: f32,
    /// Base RGB before fade overlay.
    pub base_rgb: [f32; 3],
}

impl LiveChunkFade {
    /// New fade starting at zero elapsed time.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            base_rgb: LIVE_CHUNK_BASE_COLOR,
        }
    }
}

/// Marker for a streamed agent entity.
#[derive(Component)]
pub struct LiveAgentTag {
    /// Server agent id.
    pub id: u64,
}

/// Child label entity for [`LiveAgentTag`].
#[derive(Component)]
pub struct LiveAgentLabel;

/// Marker for a streamed building diff entry.
#[derive(Component)]
pub struct LiveBuildingTag {
    /// Building id from the diff frame.
    pub id: u64,
}

/// Marker for a streamed building-graph parcel.
#[derive(Component)]
pub struct LiveGraphParcelTag {
    /// Parcel id (raw bits).
    pub id: u64,
}

/// Entity maps and voxel cache shared by live attach renderers.
#[derive(Resource)]
pub struct LiveStreamScene {
    pub chunks: HashMap<u64, Entity>,
    pub chunk_voxels: ChunkVoxelCache,
    pub agents: HashMap<u64, Entity>,
    pub buildings: HashMap<u64, Entity>,
    pub graph_parcels: HashMap<u64, Entity>,
    pub agent_materials: HashMap<u64, Handle<StandardMaterial>>,
    pub building_materials: HashMap<u64, Handle<StandardMaterial>>,
    pub graph_parcel_materials: HashMap<u64, Handle<StandardMaterial>>,
    pub building_provenance: BuildingProvenance,
}

impl Default for LiveStreamScene {
    fn default() -> Self {
        Self {
            chunks: HashMap::default(),
            chunk_voxels: ChunkVoxelCache::default(),
            agents: HashMap::default(),
            buildings: HashMap::default(),
            graph_parcels: HashMap::default(),
            agent_materials: HashMap::default(),
            building_materials: HashMap::default(),
            graph_parcel_materials: HashMap::default(),
            // BuildingProvenance does not derive Default in the wire crate
            // (intentional — every diff carries an explicit tag). The renderer
            // simply tracks the last-observed provenance, starting Procedural.
            building_provenance: BuildingProvenance::Procedural,
        }
    }
}

/// Shared marker meshes for agents and buildings.
#[derive(Resource, Clone)]
pub struct LiveStreamMeshes {
    pub agent_mesh: Handle<Mesh>,
    pub building_mesh: Handle<Mesh>,
}

/// Whether to spawn floating agent id labels.
#[derive(Clone, Copy, Default)]
pub struct AgentLabelConfig {
    pub enabled: bool,
}

impl AgentLabelConfig {
    /// Labels enabled (standalone / window default).
    #[must_use]
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }
}

/// Builds default agent/building marker meshes.
#[must_use]
pub fn default_stream_meshes(meshes: &mut Assets<Mesh>) -> LiveStreamMeshes {
    LiveStreamMeshes {
        agent_mesh: meshes.add(Cuboid::new(
            AGENT_MARKER_WIDTH,
            AGENT_MARKER_HEIGHT,
            AGENT_MARKER_DEPTH,
        )),
        building_mesh: meshes.add(Cuboid::new(2.0, 2.5, 2.0)),
    }
}

/// Default wireframe-off debug state for the live window.
#[must_use]
pub fn default_stream_wireframe() -> DebugRender {
    DebugRender::default()
}

fn chunk_transform(id: ChunkId) -> Transform {
    let (x, y, z) = decode_chunk_id(id);
    Transform::from_xyz(
        x as f32 * LIVE_CHUNK_EDGE as f32,
        y as f32 * LIVE_CHUNK_EDGE as f32,
        z as f32 * LIVE_CHUNK_EDGE as f32,
    )
}

/// Applies a voxel delta frame (caches voxels, meshes in-range chunks).
pub fn apply_voxel_delta_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    mesh_assets: &mut Assets<Mesh>,
    material_assets: &mut Assets<StandardMaterial>,
    culling: StreamCulling,
    debug: &DebugRender,
    delta: VoxelDeltaFrame,
    wireframe_line_color: Option<Color>,
) {
    for chunk in delta.deltas {
        let chunk_id = chunk.event.chunk_id;
        if chunk.voxels.len() == LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE {
            scene
                .chunk_voxels
                .insert(chunk_id, chunk.voxels.clone());
        }

        if !should_render_chunk(chunk_id, culling.eye, culling.max_distance) {
            if let Some(entity) = scene.chunks.remove(&chunk_id.0) {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if chunk.voxels.len() != LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE {
            continue;
        }

        let chunk_view = ChunkView {
            id: chunk.event.chunk_id,
            voxels: &chunk.voxels,
        };
        let distance =
            chunk_distance_from_camera(chunk.event.chunk_id, culling.eye, LIVE_CHUNK_EDGE as f32);
        let lod = LodLevel(mesh_lod_level(distance));
        let Ok(mesh_buffer) = CubicMesher::mesh_cubic(chunk_view, lod) else {
            continue;
        };
        let mesh = mesh_assets.add(mesh_buffer_to_bevy(&mesh_buffer));
        let mut material = StandardMaterial {
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        };
        apply_chunk_material(
            &mut material,
            LIVE_CHUNK_BASE_COLOR,
            debug.wireframe,
            Some(0.0),
        );
        let material_handle = material_assets.add(material);
        let transform = chunk_transform(chunk.event.chunk_id);

        let entity = *scene
            .chunks
            .entry(chunk.event.chunk_id.0)
            .or_insert_with(|| {
                commands
                    .spawn((
                        LiveChunkTag {
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
            LiveChunkFade::new(),
        ));

        if let Some(color) = wireframe_line_color {
            commands.entity(entity).insert((
                Wireframe,
                WireframeColor { color },
            ));
        } else {
            commands
                .entity(entity)
                .remove::<Wireframe>()
                .remove::<WireframeColor>();
        }
    }
}

/// Applies an agent appearance frame to the live stream scene.
pub fn apply_agent_appearance_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    agents: AgentAppearanceFrame,
) {
    apply_agent_appearance_frame_with_labels(
        commands,
        scene,
        materials,
        meshes,
        agents,
        AgentLabelConfig::enabled(),
    );
}

/// Applies an agent appearance frame with optional name labels.
pub fn apply_agent_appearance_frame_with_labels(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    agents: AgentAppearanceFrame,
    labels: AgentLabelConfig,
) {
    for update in agents.updates {
        let rgb = agent_color_from_id(update.agent_id);
        let scale = agent_scale_multiplier(update.scale);
        let (x, _, z) = agent_world_translation(&update, 0.0);
        let y = live_ground_y(&scene.chunk_voxels, x, z, AGENT_GROUND_Y);
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
                .spawn(LiveAgentTag {
                    id: update.agent_id,
                })
                .id();
            if labels.enabled {
                let label = agent_label_stub(update.agent_id, None);
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text2d::new(label),
                        TextFont::from_font_size(AGENT_LABEL_FONT_SIZE),
                        TextColor(Color::srgba(0.95, 0.96, 0.98, 0.92)),
                        Transform::from_xyz(0.0, AGENT_LABEL_Y_OFFSET, 0.0),
                        LiveAgentLabel,
                    ));
                });
            }
            entity
        });

        commands.entity(entity).insert((
            Mesh3d(meshes.agent_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

/// Applies a building diff frame (optional graph + building entries).
pub fn apply_building_diff_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
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
        apply_building_graph_frame(commands, scene, materials, meshes, graph);
    }

    if buildings.is_empty() {
        return;
    }

    let incoming: HashSet<u64> = buildings.iter().map(|entry| entry.id).collect();
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

        let ground = live_ground_y(
            &scene.chunk_voxels,
            entry.position.x,
            entry.position.z,
            BUILDING_GROUND_Y,
        );
        let transform = Transform::from_xyz(entry.position.x, ground, entry.position.z);
        let entity = *scene.buildings.entry(entry.id).or_insert_with(|| {
            commands
                .spawn(LiveBuildingTag { id: entry.id })
                .id()
        });
        commands.entity(entity).insert((
            Mesh3d(meshes.building_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

/// Applies a building graph snapshot (parcels + facades).
pub fn apply_building_graph_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    graph: BuildingGraph,
) {
    let incoming: HashSet<u64> = graph.parcels.iter().map(|parcel| parcel.id.0).collect();
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
        let ground = live_ground_y(&scene.chunk_voxels, anchor.x, anchor.z, 0.0);
        let height = parcel.size[1] as f32 / scale;
        let width = parcel.size[0] as f32 / scale;
        let depth = parcel.size[2] as f32 / scale;
        let transform = Transform::from_xyz(anchor.x, ground + height * 0.5, anchor.z)
            .with_scale(Vec3::new(width.max(0.5), height.max(0.5), depth.max(0.5)));

        let entity = *scene.graph_parcels.entry(id).or_insert_with(|| {
            commands
                .spawn(LiveGraphParcelTag { id })
                .id()
        });
        commands.entity(entity).insert((
            Mesh3d(meshes.building_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

/// Minimap dot tint for building provenance.
#[must_use]
pub fn building_minimap_dot_color(provenance: BuildingProvenance) -> Color {
    match provenance {
        BuildingProvenance::Procedural => Color::srgba(0.92, 0.90, 0.86, 1.0),
        BuildingProvenance::Freehand => Color::srgba(0.98, 0.62, 0.28, 1.0),
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
        .unwrap_or(MaterialId(0));
    material_id_color(material)
}

fn material_id_color(material: MaterialId) -> Color {
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