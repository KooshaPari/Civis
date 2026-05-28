//! Live WebSocket scene sync — voxel chunks and agent markers from `Frame3d` streams.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use civ_protocol_3d::Frame3d;
use civ_voxel::ChunkId;

use crate::bevy_render::apply_chunk_material;
use crate::live_attach::{LiveAttachBridge, LiveAttachState};
use crate::live_stream::{
    apply_agent_appearance_frame_with_labels, apply_building_diff_frame, apply_voxel_delta_frame,
    building_minimap_dot_color, default_stream_meshes, AgentLabelConfig, LiveAgentTag,
    LiveBuildingTag, LiveChunkFade, LiveGraphParcelTag, LiveStreamMeshes, LiveStreamScene,
    StreamCulling, LIVE_CHUNK_EDGE,
};
use crate::minimap::{MinimapCamera, MinimapDot, MinimapRoot, MINIMAP_SIZE};
use crate::camera::CameraRig;
use crate::{chunk_fade_complete, decode_chunk_id, AttachMode, DebugRender};

const LIVE_RENDER_MAX_DISTANCE: f32 = 200.0;
const MINIMAP_LIVE_DOT: f32 = 4.0;
const MINIMAP_CAMERA_HEIGHT: f32 = 180.0;
const LIVE_FOCUS_LERP_SPEED: f32 = 2.5;
const LIVE_FOCUS_MIN_HALF_EXTENT: f32 = 32.0;

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

/// Entity maps and voxel cache for the live attach renderer (alias of [`LiveStreamScene`]).
pub type LiveScene = LiveStreamScene;

/// Shared marker meshes for streamed agents and buildings.
pub type LiveSceneAssets = LiveStreamMeshes;

/// Applies `Frame3d` voxel/agent payloads and maintains streamed scene entities.
pub struct LiveScenePlugin;

impl Plugin for LiveScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveStreamScene>()
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
    commands.insert_resource(default_stream_meshes(&mut meshes));
}

fn apply_live_scene_frames(
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<LiveAttachState>,
    mut scene: ResMut<LiveStreamScene>,
    debug: Res<DebugRender>,
    assets: Res<LiveStreamMeshes>,
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
    let culling = StreamCulling {
        eye,
        max_distance: LIVE_RENDER_MAX_DISTANCE,
    };

    for frame in frames {
        state.tick = Some(frame.tick());
        match frame {
            Frame3d::VoxelDelta(delta) => apply_voxel_delta_frame(
                &mut commands,
                &mut scene,
                &mut meshes,
                &mut materials,
                culling,
                debug.as_ref(),
                delta,
                None,
            ),
            Frame3d::AgentAppearance(agents) => {
                apply_agent_appearance_frame_with_labels(
                    &mut commands,
                    &mut scene,
                    &mut materials,
                    assets.as_ref(),
                    agents,
                    AgentLabelConfig { enabled: true },
                );
            }
            Frame3d::BuildingDiff(building) => apply_building_diff_frame(
                &mut commands,
                &mut scene,
                &mut materials,
                assets.as_ref(),
                building,
            ),
        }
    }
}

fn update_chunk_fade(
    attach: Res<AttachMode>,
    time: Res<Time>,
    debug: Res<DebugRender>,
    mut commands: Commands,
    mut fades: Query<(Entity, &mut LiveChunkFade, &MeshMaterial3d<StandardMaterial>)>,
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
            commands.entity(entity).remove::<LiveChunkFade>();
        }
    }
}

fn sync_live_minimap_dots(
    attach: Res<AttachMode>,
    state: Res<LiveAttachState>,
    scene: Res<LiveStreamScene>,
    focus: Res<LiveSceneFocus>,
    agents: Query<&Transform, With<LiveAgentTag>>,
    buildings: Query<&Transform, With<LiveBuildingTag>>,
    graph_parcels: Query<&Transform, With<LiveGraphParcelTag>>,
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
                (cx as f32 + 0.5) * LIVE_CHUNK_EDGE as f32,
                0.0,
                (cz as f32 + 0.5) * LIVE_CHUNK_EDGE as f32,
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
    scene: Res<LiveStreamScene>,
    agents: Query<&Transform, With<LiveAgentTag>>,
    buildings: Query<&Transform, With<LiveBuildingTag>>,
    graph_parcels: Query<&Transform, With<LiveGraphParcelTag>>,
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
    scene: &LiveStreamScene,
    agents: &Query<&Transform, With<LiveAgentTag>>,
    buildings: &Query<&Transform, With<LiveBuildingTag>>,
    graph_parcels: &Query<&Transform, With<LiveGraphParcelTag>>,
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
            (cx as f32 + 0.5) * LIVE_CHUNK_EDGE as f32,
            (cz as f32 + 0.5) * LIVE_CHUNK_EDGE as f32,
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
