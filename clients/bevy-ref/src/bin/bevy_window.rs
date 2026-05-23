use std::collections::HashMap;

use bevy::prelude::*;
use civ_bevy_ref::{
    bevy_render::{mesh_buffer_to_bevy, spawn_default_scene},
    ws_client::WsClient,
    CubicMesher,
};
use civ_protocol_3d::{AgentAppearanceFrame, Frame3d, VoxelDeltaFrame};
use civ_voxel::{ChunkId, ChunkView, LodLevel};

const CHUNK_EDGE: usize = 16;
const LIVE_WS_URL: &str = "ws://127.0.0.1:8765/ws";

#[derive(Resource)]
struct LiveBridge {
    client: WsClient,
}

#[derive(Resource, Default)]
struct LiveScene {
    chunks: HashMap<u64, Entity>,
    agents: HashMap<u64, Entity>,
}

#[derive(Component)]
#[allow(dead_code)]
struct ChunkTag {
    id: ChunkId,
}

#[derive(Component)]
#[allow(dead_code)]
struct AgentTag {
    id: u64,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Civis 3D — Bevy reference (live)".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(LiveScene::default())
        .add_systems(Startup, setup)
        .add_systems(Update, apply_live_frames)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_default_scene(&mut commands);
    commands.insert_resource(LiveBridge {
        client: WsClient::spawn(LIVE_WS_URL.to_string()),
    });
}

fn apply_live_frames(
    mut commands: Commands,
    bridge: Res<LiveBridge>,
    mut scene: ResMut<LiveScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for frame in bridge.client.poll() {
        match frame {
            Frame3d::VoxelDelta(delta) => apply_voxel_delta(
                &mut commands,
                &mut scene,
                &mut meshes,
                &mut materials,
                delta,
            ),
            Frame3d::AgentAppearance(agents) => apply_agent_appearance(
                &mut commands,
                &mut scene,
                &mut meshes,
                &mut materials,
                agents,
            ),
            Frame3d::BuildingDiff(_) => {}
        }
    }
}

fn apply_voxel_delta(
    commands: &mut Commands,
    scene: &mut LiveScene,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    delta: VoxelDeltaFrame,
) {
    for chunk in delta.deltas {
        if chunk.voxels.len() != CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE {
            continue;
        }

        let chunk_view = ChunkView {
            id: chunk.event.chunk_id,
            voxels: &chunk.voxels,
        };
        let Ok(mesh_buffer) = CubicMesher::mesh_cubic(chunk_view, LodLevel(0)) else {
            continue;
        };
        let mesh = meshes.add(mesh_buffer_to_bevy(&mesh_buffer));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.72, 0.69, 0.62),
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        });
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
        commands.entity(entity).insert(PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        });
    }
}

fn apply_agent_appearance(
    commands: &mut Commands,
    scene: &mut LiveScene,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    agents: AgentAppearanceFrame,
) {
    for update in agents.updates {
        let mesh = meshes.add(Cuboid::new(0.8, 1.6, 0.8));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.85),
            perceptual_roughness: 0.7,
            metallic: 0.0,
            ..default()
        });
        let transform = Transform::from_xyz(update.agent_id as f32, 0.8, 0.0);

        let entity = *scene.agents.entry(update.agent_id).or_insert_with(|| {
            commands
                .spawn((
                    AgentTag {
                        id: update.agent_id,
                    },
                    Transform::default(),
                ))
                .id()
        });
        commands.entity(entity).insert(PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        });
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

fn decode_chunk_id(id: ChunkId) -> (i32, i32, i32) {
    let raw = id.0;
    let mut cx = ((raw >> 40) & 0x00ff_ffff) as i32;
    let mut cy = ((raw >> 16) & 0x00ff_ffff) as i32;
    let mut cz = (raw & 0x0000_ffff) as i32;
    if cx & 0x0080_0000 != 0 {
        cx |= !0x00ff_ffff;
    }
    if cy & 0x0080_0000 != 0 {
        cy |= !0x00ff_ffff;
    }
    if cz & 0x0000_8000 != 0 {
        cz |= !0x0000_ffff;
    }
    (cx, cy, cz)
}
