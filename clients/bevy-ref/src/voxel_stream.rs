//! Camera-driven chunk streaming sandbox for the large voxel world.
//!
//! Drives the `civ-voxel` [`StreamingWorld`] from the roaming camera: a radius of
//! chunks around the camera pages in/out every frame (LRU + disk-backed dirty cache
//! in the streaming layer), only *loaded* chunks are meshed, far chunks mesh at a
//! coarser LOD, and back-facing chunks are skipped before meshing (a cheap pre-pass
//! on top of Bevy's automatic view-frustum culling of the spawned `Aabb` meshes).
//! A cellular-automaton (CA) style tick runs only over the active hot set.
//!
//! A perf HUD line (`loaded chunks / verts / fps`) is emitted: as an egui overlay
//! when the `egui` feature is on, otherwise via `tracing` once per second.
//!
//! Requirements: `FR-CIV-VOXEL-020..029`, `NFR-SCALE-PERF` (20mi streaming target).
//!
//! ## Borrowed patterns
//! - Camera-radius page-in/out + per-chunk entity keyed by chunk coord mirrors
//!   `bevy_voxel_world`'s chunk-manager loop and Veloren's terrain streaming.
//! - The greedy/cubic meshing + LOD-by-distance split follows `block-mesh-rs` /
//!   `building-blocks` conventions (kernel `CubicMesher` does the face emission).

use std::collections::HashMap;

use bevy::prelude::*;

use civ_voxel::{
    mesh_triangle_count, select_lod, ChunkCoord, ChunkView, CubicMesher, HeightFieldGen, LodLevel,
    LodPolicy, MaterialId, RingIter, StreamConfig, StreamStats, StreamingWorld,
    VoxelScaleMultiplier, CHUNK_EDGE_I32,
};

use crate::bevy_render::mesh_buffer_to_bevy;
use crate::camera::CameraRig;

/// Base voxel edge in metres (1 Bevy world unit == 1 metre in the sandbox).
const BASE_VOXEL_M: f32 = 4.0;
/// World-space edge of one chunk in Bevy units (16 voxels × base voxel size).
const CHUNK_UNITS: f32 = CHUNK_EDGE_I32 as f32 * BASE_VOXEL_M;
/// Chunk radius streamed around the camera on the horizontal plane.
const STREAM_RADIUS: i32 = 6;
/// Vertical chunk band around the camera (worlds are mostly flat heightfields).
const STREAM_VBAND: i32 = 2;
/// Exact chunk count for the configured horizontal disc x vertical band.
const DESIRED_CHUNK_COUNT: usize = 585;

/// Streaming world + render bookkeeping, kept as a Bevy resource.
#[derive(Resource)]
pub struct VoxelStreamState {
    world: StreamingWorld<HeightFieldGen>,
    /// Live mesh entities keyed by chunk coord, so we can despawn on evict.
    spawned: HashMap<ChunkCoord, Entity>,
    lod_scale: VoxelScaleMultiplier,
    lod_policy: LodPolicy,
    material: Handle<StandardMaterial>,
    // Only read by the `tracing` HUD throttle (non-egui build); harmless otherwise.
    last_hud: f64,
    hud: VoxelHud,
}

/// Snapshot for the perf HUD line.
#[derive(Clone, Copy, Default)]
pub struct VoxelHud {
    /// Chunks resident in RAM.
    pub loaded: usize,
    /// Total vertices currently meshed and spawned.
    pub verts: usize,
    /// Frames per second (smoothed by Bevy's diagnostics-free dt estimate).
    pub fps: f32,
    /// Streaming layer stats (regen / disk / evictions).
    pub stats: StreamStats,
}

/// Plugin wiring the camera-driven streaming sandbox.
pub struct VoxelStreamPlugin;

impl Plugin for VoxelStreamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_voxel_stream).add_systems(
            Update,
            (stream_around_camera, ca_tick_hot_set, voxel_hud).chain(),
        );
    }
}

fn setup_voxel_stream(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let cfg = StreamConfig {
        seed: 0xC1A5_5EED,
        active_budget: ((2 * STREAM_RADIUS + 1).pow(2) * (2 * STREAM_VBAND + 1) + 64) as usize,
        base_voxel_m: BASE_VOXEL_M,
        ..StreamConfig::default()
    };
    let gen = HeightFieldGen {
        seed: cfg.seed,
        base_voxel_m: BASE_VOXEL_M,
        sea_level_m: 24.0,
    };
    let world = StreamingWorld::new(cfg, gen).expect("streaming world (RAM-only)");
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.62, 0.38),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.insert_resource(VoxelStreamState {
        world,
        spawned: HashMap::new(),
        lod_scale: VoxelScaleMultiplier(BASE_VOXEL_M),
        lod_policy: LodPolicy::default(),
        material,
        last_hud: 0.0,
        hud: VoxelHud::default(),
    });
}

/// Chunk coord containing a world-space (Bevy units) position.
fn world_to_chunk(pos: Vec3) -> ChunkCoord {
    ChunkCoord {
        cx: (pos.x / CHUNK_UNITS).floor() as i32,
        cy: (pos.y / CHUNK_UNITS).floor() as i32,
        cz: (pos.z / CHUNK_UNITS).floor() as i32,
    }
}

/// World-space centre of a chunk in Bevy units.
fn chunk_center(coord: ChunkCoord) -> Vec3 {
    Vec3::new(
        (coord.cx as f32 + 0.5) * CHUNK_UNITS,
        (coord.cy as f32 + 0.5) * CHUNK_UNITS,
        (coord.cz as f32 + 0.5) * CHUNK_UNITS,
    )
}

/// Build the desired chunk set around the camera (a horizontal disc × vertical band).
fn desired_set(center: ChunkCoord) -> Vec<ChunkCoord> {
    let mut set = Vec::with_capacity(DESIRED_CHUNK_COUNT);
    for ring in 0..=STREAM_RADIUS as u32 {
        for coord in RingIter::new(center, ring, 1) {
            let dx = coord.cx - center.cx;
            let dz = coord.cz - center.cz;
            if dx * dx + dz * dz > STREAM_RADIUS * STREAM_RADIUS {
                continue; // disc, not square — fewer far chunks
            }
            if (coord.cy - center.cy).abs() > STREAM_VBAND {
                continue;
            }
            set.push(coord);
        }
    }
    debug_assert_eq!(set.len(), DESIRED_CHUNK_COUNT);
    set
}

/// Page chunks in/out around the camera and (re)spawn meshes for the loaded set.
fn stream_around_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    rig: Res<CameraRig>,
    cam: Query<&Transform, With<Camera3d>>,
    mut state: ResMut<VoxelStreamState>,
) {
    let cam_pos = cam
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or(rig.target);
    let center = world_to_chunk(rig.target);
    let want = desired_set(center);
    if state.world.load_set(&want).is_err() {
        return; // budget guard; configured budget always covers `want`
    }

    // Despawn meshes for chunks no longer resident.
    let resident: std::collections::HashSet<ChunkCoord> =
        state.world.resident_coords().into_iter().collect();
    let stale: Vec<ChunkCoord> = state
        .spawned
        .keys()
        .copied()
        .filter(|c| !resident.contains(c))
        .collect();
    for coord in stale {
        if let Some(entity) = state.spawned.remove(&coord) {
            commands.entity(entity).despawn();
        }
    }

    // Mesh newly-resident, camera-facing chunks.
    let to_mesh: Vec<ChunkCoord> = want
        .iter()
        .copied()
        .filter(|c| !state.spawned.contains_key(c))
        .filter(|c| chunk_is_visible(*c, cam_pos, &cam))
        .collect();
    let (lod_scale, lod_policy, material) =
        (state.lod_scale, state.lod_policy, state.material.clone());
    let mut total_new_verts = 0usize;
    for coord in to_mesh {
        let dist = (chunk_center(coord) - cam_pos).length();
        let lod = select_lod(dist, lod_scale, lod_policy);
        if let Some(entity) = mesh_chunk(
            &mut commands,
            &mut meshes,
            &state,
            coord,
            lod,
            &material,
            &mut total_new_verts,
        ) {
            state.spawned.insert(coord, entity);
        }
    }
    state.hud.verts = state.hud.verts.saturating_add(total_new_verts);
    state.hud.loaded = resident.len();
    state.hud.stats = state.world.stats();
}

/// Backface pre-pass: keep a chunk if it is in front of the camera or close enough
/// that it may still be in view. Bevy frustum-culls the spawned mesh precisely.
fn chunk_is_visible(
    coord: ChunkCoord,
    cam_pos: Vec3,
    cam: &Query<&Transform, With<Camera3d>>,
) -> bool {
    let Some(t) = cam.iter().next() else {
        return true;
    };
    let to_chunk = chunk_center(coord) - cam_pos;
    if to_chunk.length() < CHUNK_UNITS * 2.0 {
        return true; // never cull near chunks
    }
    t.forward().dot(to_chunk.normalize()) > -0.2
}

/// Mesh one resident chunk into a spawned PBR entity. Returns the entity, or `None`
/// if the chunk is empty (all air) so we do not spawn degenerate meshes.
fn mesh_chunk(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    state: &VoxelStreamState,
    coord: ChunkCoord,
    lod: LodLevel,
    material: &Handle<StandardMaterial>,
    total_new_verts: &mut usize,
) -> Option<Entity> {
    let chunk = state.world.get(coord)?;
    let view = ChunkView {
        id: civ_voxel::ChunkId(0),
        voxels: &chunk.voxels,
    };
    let buf = CubicMesher::mesh_cubic(view, lod).ok()?;
    if buf.vertices.is_empty() {
        return None;
    }
    *total_new_verts += buf.vertices.len();
    let mesh = meshes.add(mesh_buffer_to_bevy(&buf));
    let origin = Vec3::new(
        coord.cx as f32 * CHUNK_UNITS,
        coord.cy as f32 * CHUNK_UNITS,
        coord.cz as f32 * CHUNK_UNITS,
    );
    let entity = commands
        .spawn((
            Mesh3d(mesh),
            bevy::pbr::MeshMaterial3d(material.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(BASE_VOXEL_M)),
        ))
        .id();
    Some(entity)
}

/// CA tick over the active hot set only: a single mass-conserving "settle" step.
/// A solid voxel with air directly below swaps with that air (gravity fall) — mass
/// is conserved (a swap, not a create/destroy). Only resident chunks are scanned;
/// unloaded chunks are frozen and conserve trivially. Edits go through the streaming
/// layer so they are marked dirty and persisted on eviction.
fn ca_tick_hot_set(mut state: ResMut<VoxelStreamState>) {
    let edge = CHUNK_EDGE_I32 as usize;
    let hot = state.world.resident_coords();
    for coord in hot {
        let Some(swap) = first_settle_swap(&state, coord, edge) else {
            continue;
        };
        let (above, below, mat) = swap;
        state.world.edit(coord, below, mat);
        state.world.edit(coord, above, MaterialId(0));
    }
}

/// First (deterministic) solid-over-air pair within a chunk; `(above, below, mat)`.
fn first_settle_swap(
    state: &VoxelStreamState,
    coord: ChunkCoord,
    edge: usize,
) -> Option<(usize, usize, MaterialId)> {
    let chunk = state.world.get(coord)?;
    for y in 1..edge {
        for z in 0..edge {
            for x in 0..edge {
                let above = x + y * edge + z * edge * edge;
                let below = x + (y - 1) * edge + z * edge * edge;
                let mat = chunk.voxels[above];
                if mat != MaterialId(0) && chunk.voxels[below] == MaterialId(0) {
                    return Some((above, below, mat));
                }
            }
        }
    }
    None
}

#[cfg(feature = "egui")]
fn voxel_hud(
    time: Res<Time>,
    mut state: ResMut<VoxelStreamState>,
    mut contexts: bevy_egui::EguiContexts,
) {
    state.hud.fps = 1.0 / time.delta_secs().max(1e-4);
    let hud = state.hud;
    if let Ok(ctx) = contexts.ctx_mut() {
        bevy_egui::egui::Window::new("voxel stream").show(ctx, |ui| {
            ui.label(format!(
                "loaded {} chunks | {} verts | {:.0} fps",
                hud.loaded, hud.verts, hud.fps
            ));
            ui.label(format!(
                "regen {} | disk-load {} | evict(disk {} / drop {})",
                hud.stats.regenerated,
                hud.stats.disk_loads,
                hud.stats.disk_evictions,
                hud.stats.dropped_evictions
            ));
        });
    }
}

#[cfg(not(feature = "egui"))]
fn voxel_hud(time: Res<Time>, mut state: ResMut<VoxelStreamState>) {
    state.hud.fps = 1.0 / time.delta_secs().max(1e-4);
    let now = time.elapsed_secs_f64();
    if now - state.last_hud >= 1.0 {
        state.last_hud = now;
        let hud = state.hud;
        tracing::info!(
            loaded = hud.loaded,
            verts = hud.verts,
            fps = hud.fps,
            "voxel stream HUD"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-VOXEL-020 — chunk/world coordinate round-trip is consistent.
    #[test]
    fn world_chunk_roundtrip() {
        let coord = ChunkCoord {
            cx: 3,
            cy: -1,
            cz: 5,
        };
        let center = chunk_center(coord);
        assert_eq!(world_to_chunk(center), coord);
    }

    /// FR-CIV-VOXEL-020 — desired set is a horizontal disc × vertical band, bounded.
    #[test]
    fn desired_set_is_bounded_disc() {
        let set = desired_set(ChunkCoord {
            cx: 0,
            cy: 0,
            cz: 0,
        });
        assert!(!set.is_empty());
        assert_eq!(set.len(), DESIRED_CHUNK_COUNT);
        let max = ((2 * STREAM_RADIUS + 1).pow(2) * (2 * STREAM_VBAND + 1)) as usize;
        assert!(set.len() <= max);
        // Centre column is always present.
        assert!(set.contains(&ChunkCoord {
            cx: 0,
            cy: 0,
            cz: 0
        }));
        // A corner outside the disc radius is excluded.
        assert!(!set.contains(&ChunkCoord {
            cx: STREAM_RADIUS,
            cy: 0,
            cz: STREAM_RADIUS
        }));
    }

    /// FR-CIV-SCALE — distant chunks mesh with fewer triangles after LOD selection.
    #[test]
    fn distant_chunks_have_fewer_triangles() {
        let mut voxels = vec![MaterialId(0); 16 * 16 * 16];
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    if (x + y + z) % 2 == 0 {
                        voxels[x + y * 16 + z * 16 * 16] = MaterialId(1);
                    }
                }
            }
        }
        let view = ChunkView {
            id: civ_voxel::ChunkId(0),
            voxels: &voxels,
        };
        let near = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("near mesh");
        let far = CubicMesher::mesh_cubic(view, LodLevel(2)).expect("far mesh");
        assert!(
            mesh_triangle_count(&far) < mesh_triangle_count(&near),
            "far triangles {} should be fewer than near triangles {}",
            mesh_triangle_count(&far),
            mesh_triangle_count(&near)
        );
    }
}
