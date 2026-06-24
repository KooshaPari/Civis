//! Live WebSocket scene sync — voxel chunks and agent markers from `Frame3d` streams.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use civ_protocol_3d::Frame3d;
use civ_voxel::ChunkId;

use crate::bevy_render::apply_chunk_material;
use crate::camera::CameraRig;
#[cfg(feature = "egui")]
use crate::event_feed::EventFeed;
use crate::live_attach::{LiveAttachBridge, LiveAttachState};
use crate::live_focus::{compute_live_scene_focus, LiveSceneFocus, LIVE_FOCUS_LERP_SPEED};
use crate::live_minimap::{
    chunk_centre_world_xz, live_building_dot_color, spawn_minimap_dot, MinimapDotLayout,
    MinimapFocusRect, LIVE_MINIMAP_AGENT_COLOR, LIVE_MINIMAP_CHUNK_COLOR, LIVE_MINIMAP_DOT,
    LIVE_MINIMAP_GRAPH_DOT_SCALE,
};
#[cfg(feature = "egui")]
use crate::live_stream::apply_event_feed_frame;
use crate::live_stream::{
    apply_agent_appearance_frame_with_labels, apply_building_diff_frame,
    apply_civilian_state_frame, apply_faction_state_frame, apply_voxel_delta_frame,
    default_stream_meshes, AgentLabelConfig, LiveAgentTag, LiveBuildingTag, LiveChunkFade,
    LiveGraphParcelTag, LiveStreamMeshes, LiveStreamScene, StreamCulling, LIVE_CHUNK_EDGE,
};
use crate::minimap::{MinimapCamera, MinimapDot, MinimapRoot, MINIMAP_SIZE};
use crate::{chunk_fade_complete, AttachMode, DebugRender, LiveHudSnapshot};

const LIVE_RENDER_MAX_DISTANCE: f32 = 200.0;
const MINIMAP_CAMERA_HEIGHT: f32 = 180.0;
const LIVE_MINIMAP_PANEL_LAYOUT: MinimapDotLayout = MinimapDotLayout::FullPanel {
    panel_size: MINIMAP_SIZE,
};

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
    mut hud: ResMut<LiveHudSnapshot>,
    mut scene: ResMut<LiveStreamScene>,
    debug: Res<DebugRender>,
    assets: Res<LiveStreamMeshes>,
    cameras: Query<&Transform, (With<Camera3d>, Without<crate::minimap::MinimapCamera>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "egui")] mut event_feed: Option<ResMut<EventFeed>>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    let frames = bridge.client.poll();
    if frames.is_empty() {
        return;
    }

    state.connected = true;
    hud.connected = true;
    let eye = cameras
        .single()
        .map(|transform| transform.translation.to_array())
        .unwrap_or([8.0, 8.0, 8.0]);
    let culling = StreamCulling {
        eye,
        max_distance: LIVE_RENDER_MAX_DISTANCE,
    };

    for frame in frames {
        let tick = frame.tick();
        state.tick = Some(tick);
        hud.tick = Some(tick);
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
            Frame3d::CivilianState(civilian) => apply_civilian_state_frame(&mut scene, civilian),
            Frame3d::FactionState(faction) => apply_faction_state_frame(&mut scene, faction),
            #[cfg(feature = "egui")]
            Frame3d::EventFeed(event_frame) => {
                if let Some(feed) = event_feed.as_mut() {
                    apply_event_feed_frame(feed, event_frame);
                }
            }
            Frame3d::Climate(_) => {}
            #[cfg(not(feature = "egui"))]
            Frame3d::EventFeed(_) => {}
            Frame3d::Climate(_) => {}
        }
    }
}

fn update_chunk_fade(
    attach: Res<AttachMode>,
    time: Res<Time>,
    debug: Res<DebugRender>,
    mut commands: Commands,
    mut fades: Query<(
        Entity,
        &mut LiveChunkFade,
        &MeshMaterial3d<StandardMaterial>,
    )>,
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

    let building_dot = live_building_dot_color(scene.building_provenance);
    let focus_rect = MinimapFocusRect {
        centre_x: focus.centre.x,
        centre_z: focus.centre.z,
        half_extent: focus.half_extent,
    };

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Ok(root) = roots.single() else {
        return;
    };

    commands.entity(root).with_children(|parent| {
        for raw in scene.chunks.keys() {
            let (x, z) = chunk_centre_world_xz(ChunkId(*raw), LIVE_CHUNK_EDGE);
            let uv = focus_rect.world_to_uv(x, z);
            spawn_minimap_dot(
                parent,
                LIVE_MINIMAP_PANEL_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT,
                LIVE_MINIMAP_CHUNK_COLOR,
                true,
            );
        }

        for transform in &agents {
            let uv = focus_rect.world_to_uv(transform.translation.x, transform.translation.z);
            spawn_minimap_dot(
                parent,
                LIVE_MINIMAP_PANEL_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT,
                LIVE_MINIMAP_AGENT_COLOR,
                true,
            );
        }

        for transform in &buildings {
            let uv = focus_rect.world_to_uv(transform.translation.x, transform.translation.z);
            spawn_minimap_dot(
                parent,
                LIVE_MINIMAP_PANEL_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT,
                building_dot,
                true,
            );
        }

        for transform in &graph_parcels {
            let uv = focus_rect.world_to_uv(transform.translation.x, transform.translation.z);
            spawn_minimap_dot(
                parent,
                LIVE_MINIMAP_PANEL_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT * LIVE_MINIMAP_GRAPH_DOT_SCALE,
                building_dot,
                true,
            );
        }
    });
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
