use bevy::prelude::*;
use civ_bevy_ref::{bevy_render::spawn_default_scene, bevy_render::spawn_voxel_mesh};
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, VoxelWorld};

const CHUNK_EDGE: usize = 16;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Civis 3D — Bevy reference".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_default_scene(&mut commands);

    let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(1_000_000);
    for x in 0..4 {
        for y in 0..4 {
            for z in 0..4 {
                world.write(
                    civ_voxel::WorldCoord {
                        x: x * 1_000_000,
                        y: y * 1_000_000,
                        z: z * 1_000_000,
                    },
                    MaterialId(1),
                );
            }
        }
    }
    let _ = world.drain_dirty();

    let mut chunk_voxels = vec![MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
    for x in 0..4 {
        for y in 0..4 {
            for z in 0..4 {
                chunk_voxels[x + y * CHUNK_EDGE + z * CHUNK_EDGE * CHUNK_EDGE] = MaterialId(1);
            }
        }
    }

    let view = ChunkView {
        id: ChunkId(0),
        voxels: &chunk_voxels,
    };
    let mesh = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("mesh");
    spawn_voxel_mesh(&mut commands, &mut meshes, &mut materials, &mesh);
}
