//! Bevy voxel simulation renderer for Civis P-VM-3.
//!
//! Feature-gated behind `voxel` so the heightmap sandbox remains the default
//! when the feature is off.

use std::collections::BTreeMap;

use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;

use civ_voxel::fluid_ca::{step, CaGrid};
use civ_voxel::material::{MaterialRegistry, Phase, AIR};
use civ_voxel::worldgen;
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, MeshBuffer};

use crate::camera::CameraRig;
use crate::{bevy_render::mesh_buffer_to_bevy, should_render_chunk};

const SEED: u64 = 0xC1F1_5EED_D3AD_BEEF;
const CA_TICK_HZ: f32 = 12.0;
const CHUNK_EDGE: usize = 16;
const RENDER_MAX_DIST: f32 = 160.0;

/// Voxel simulation plugin for the Bevy reference client.
pub struct VoxelSimPlugin;

impl Plugin for VoxelSimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VoxelSimState::default())
            .add_systems(Startup, setup_voxel_world)
            .add_systems(Update, step_and_remesh);
    }
}

/// Live voxel simulation state for the standalone renderer.
#[derive(Resource)]
pub struct VoxelSimState {
    /// Dense CA grid mirrored from world generation.
    pub grid: CaGrid,
    /// Number of CA ticks executed so far.
    pub tick: u64,
    /// Fixed-step accumulator in seconds.
    pub accumulator: f32,
    /// Currently spawned chunk mesh entities.
    pub chunk_entities: Vec<Entity>,
}

impl Default for VoxelSimState {
    fn default() -> Self {
        Self {
            grid: CaGrid {
                dims: [0, 0, 0],
                cells: Vec::new(),
            },
            tick: 0,
            accumulator: 0.0,
            chunk_entities: Vec::new(),
        }
    }
}

/// Build the initial voxel world, then mesh and spawn all visible chunks.
pub fn setup_voxel_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rig: ResMut<CameraRig>,
    mut state: ResMut<VoxelSimState>,
    cameras: Query<&Transform, With<Camera3d>>,
) {
    let generated = worldgen::generate([64, 48, 64], SEED);
    state.grid = CaGrid {
        dims: generated.dims,
        cells: generated.cells,
    };
    state.tick = 0;
    state.accumulator = 0.0;
    state.chunk_entities.clear();
    reframe_camera_once(
        &mut rig,
        Vec3::new(
            generated.dims[0] as f32 * 0.5,
            generated.dims[1] as f32 * 0.5,
            generated.dims[2] as f32 * 0.5,
        ),
    );
    let camera_eye = cameras.iter().next().map(|t| t.translation.to_array());
    state.chunk_entities = spawn_chunk_meshes(
        &mut commands,
        &mut meshes,
        &mut materials,
        &state.grid,
        camera_eye,
    );
}

/// Step the CA at a fixed rate and remesh all chunks after any tick occurs.
pub fn step_and_remesh(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<&Transform, With<Camera3d>>,
    mut state: ResMut<VoxelSimState>,
) {
    state.accumulator += time.delta_secs();
    let mut stepped = false;
    let step_dt = 1.0 / CA_TICK_HZ;
    while state.accumulator >= step_dt {
        step(&mut state.grid, MaterialRegistry::standard());
        state.tick = state.tick.wrapping_add(1);
        state.accumulator -= step_dt;
        stepped = true;
    }
    if !stepped {
        return;
    }

    for entity in state.chunk_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    let camera_eye = cameras.iter().next().map(|t| t.translation.to_array());
    state.chunk_entities = spawn_chunk_meshes(
        &mut commands,
        &mut meshes,
        &mut materials,
        &state.grid,
        camera_eye,
    );
}

/// Slice a 16³ chunk from the dense grid, filling out-of-bounds cells with air.
#[must_use]
fn slice_chunk(grid: &CaGrid, cx: usize, cy: usize, cz: usize) -> [MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE] {
    let mut voxels = [AIR; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
    for z in 0..CHUNK_EDGE {
        for y in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                let gx = cx * CHUNK_EDGE + x;
                let gy = cy * CHUNK_EDGE + y;
                let gz = cz * CHUNK_EDGE + z;
                voxels[x + y * CHUNK_EDGE + z * CHUNK_EDGE * CHUNK_EDGE] = grid.get(gx, gy, gz);
            }
        }
    }
    voxels
}

/// Frame the camera on the voxel volume by updating the orbit rig, not the
/// raw transform. This keeps yaw/pitch/zoom recoverable after startup.
fn reframe_camera_once(rig: &mut CameraRig, target: Vec3) {
    let extent = target.length().max(1.0);
    rig.target = target;
    rig.yaw = -0.12;
    rig.pitch = -0.72;
    rig.distance = (extent * 2.5).clamp(20.0, 600.0);
}

/// Split a mesh buffer into per-material buffers, reindexing each triangle group.
#[must_use]
fn split_by_material(buf: &MeshBuffer) -> Vec<(MaterialId, MeshBuffer)> {
    let mut groups: BTreeMap<MaterialId, MeshBuffer> = BTreeMap::new();
    for tri in buf.indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let material = buf.vertices[tri[0] as usize].material;
        let group = groups.entry(material).or_default();
        let base = group.vertices.len() as u32;
        for &index in tri {
            group.vertices.push(buf.vertices[index as usize]);
        }
        group.indices.extend_from_slice(&[base, base + 1, base + 2]);
    }
    groups.into_iter().collect()
}

/// Mesh one chunk, split it by material, and spawn world-offset Bevy entities.
fn spawn_chunk_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    grid: &CaGrid,
    camera_eye: Option<[f32; 3]>,
) -> Vec<Entity> {
    let chunk_counts = [
        grid.dims[0].div_ceil(CHUNK_EDGE),
        grid.dims[1].div_ceil(CHUNK_EDGE),
        grid.dims[2].div_ceil(CHUNK_EDGE),
    ];
    let registry = MaterialRegistry::standard();
    let mut spawned = Vec::new();
    for cz in 0..chunk_counts[2] {
        for cy in 0..chunk_counts[1] {
            for cx in 0..chunk_counts[0] {
                let chunk_origin = [cx * CHUNK_EDGE, cy * CHUNK_EDGE, cz * CHUNK_EDGE];
                let chunk_id = ChunkId(((cx as u64) << 40) | ((cy as u64) << 16) | (cz as u64));
                if let Some(eye) = camera_eye {
                    if !should_render_chunk(chunk_id, eye, RENDER_MAX_DIST) {
                        continue;
                    }
                }
                let voxels = slice_chunk(grid, cx, cy, cz);
                let view = ChunkView { id: chunk_id, voxels: &voxels };
                let Ok(mesh_buffer) = CubicMesher::mesh_cubic(view, LodLevel(0)) else {
                    continue;
                };
                for (material_id, submesh) in split_by_material(&mesh_buffer) {
                    let Some(def) = registry.get(material_id) else {
                        continue;
                    };
                    let mut material = StandardMaterial {
                        base_color: Color::srgba(
                            f32::from(def.color[0]) / 255.0,
                            f32::from(def.color[1]) / 255.0,
                            f32::from(def.color[2]) / 255.0,
                            f32::from(def.color[3]) / 255.0,
                        ),
                        ..default()
                    };
                    material.alpha_mode = match def.phase {
                        Phase::Liquid | Phase::Gas => AlphaMode::Blend,
                        Phase::Powder | Phase::Solid | Phase::Empty => AlphaMode::Opaque,
                    };
                    let entity = commands
                        .spawn((
                            Mesh3d(meshes.add(mesh_buffer_to_bevy(&submesh))),
                            MeshMaterial3d(materials.add(material)),
                            Transform::from_xyz(
                                chunk_origin[0] as f32,
                                chunk_origin[1] as f32,
                                chunk_origin[2] as f32,
                            ),
                        ))
                        .id();
                    spawned.push(entity);
                }
            }
        }
    }
    spawned
}
