//! Live WebSocket scene sync — voxel chunks and agent markers from `Frame3d` streams.

use std::collections::HashMap;

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};
use bevy::ui::FocusPolicy;
use civ_protocol_3d::{
    agent_world_translation, AgentAppearanceFrame, BuildingDiffFrame, BuildingKind3d, Frame3d,
    VoxelDeltaFrame,
};
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel};

use crate::bevy_render::{apply_chunk_material, mesh_buffer_to_bevy};
use crate::live_attach::{LiveAttachBridge, LiveAttachState};
use crate::minimap::{MinimapDot, MinimapRoot, MINIMAP_SIZE};
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
const MINIMAP_WORLD_MIN: f32 = 0.0;
const MINIMAP_WORLD_MAX: f32 = 256.0;
const MINIMAP_LIVE_DOT: f32 = 4.0;
const AGENT_GROUND_Y: f32 = 0.8;
const BUILDING_GROUND_Y: f32 = 1.25;

/// Tracks spawned entities for streamed voxel chunks and agents.
#[derive(Resource, Default)]
pub struct LiveScene {
    chunks: HashMap<u64, Entity>,
    agents: HashMap<u64, Entity>,
    buildings: HashMap<u64, Entity>,
    agent_materials: HashMap<u64, Handle<StandardMaterial>>,
    building_materials: HashMap<u64, Handle<StandardMaterial>>,
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

/// Applies `Frame3d` voxel/agent payloads and maintains streamed scene entities.
pub struct LiveScenePlugin;

impl Plugin for LiveScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveScene>()
            .init_resource::<DebugRender>()
            .add_systems(Startup, setup_live_scene_assets)
            .add_systems(
                Update,
                (
                    apply_live_scene_frames,
                    update_chunk_fade,
                    sync_live_minimap_dots,
                ),
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
        let (x, y, z) = agent_world_translation(&update, AGENT_GROUND_Y);
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
    if frame.buildings.is_empty() {
        return;
    }

    let incoming: std::collections::HashSet<u64> =
        frame.buildings.iter().map(|entry| entry.id).collect();
    for (id, entity) in scene.buildings.clone() {
        if !incoming.contains(&id) {
            commands.entity(entity).despawn();
            scene.buildings.remove(&id);
            scene.building_materials.remove(&id);
        }
    }

    for entry in frame.buildings {
        let color = building_kind_color(entry.kind);
        let material_handle = scene
            .building_materials
            .entry(entry.id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.9,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = color;
        }

        let transform = Transform::from_xyz(entry.position.x, BUILDING_GROUND_Y, entry.position.z);
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
    agents: Query<&Transform, With<AgentTag>>,
    buildings: Query<&Transform, With<BuildingTag>>,
    mut commands: Commands,
    roots: Query<Entity, With<MinimapRoot>>,
    existing: Query<Entity, With<MinimapDot>>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    if !scene.is_changed() && !state.is_changed() {
        return;
    }

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
            let uv = world_to_minimap_uv(world);
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
            let uv = world_to_minimap_uv(transform.translation);
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
            let uv = world_to_minimap_uv(transform.translation);
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
                BackgroundColor(Color::WHITE),
                MinimapDot,
                FocusPolicy::Pass,
            ));
        }
    });
}

fn world_to_minimap_uv(position: Vec3) -> Vec2 {
    let u = ((position.x - MINIMAP_WORLD_MIN) / (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN))
        .clamp(0.0, 1.0);
    let v = ((position.z - MINIMAP_WORLD_MIN) / (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN))
        .clamp(0.0, 1.0);
    Vec2::new(u, 1.0 - v)
}

fn chunk_transform(id: ChunkId) -> Transform {
    let (x, y, z) = decode_chunk_id(id);
    Transform::from_xyz(
        x as f32 * CHUNK_EDGE as f32,
        y as f32 * CHUNK_EDGE as f32,
        z as f32 * CHUNK_EDGE as f32,
    )
}
