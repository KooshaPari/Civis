//! Bevy voxel simulation renderer for Civis P-VM-3.
//!
//! Feature-gated behind `voxel` so the heightmap sandbox remains the default
//! when the feature is off.

use std::collections::{BTreeMap, HashMap, HashSet};

use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;

use civ_voxel::fluid_ca::{step, CaGrid};
use civ_voxel::material::{
    MaterialDef, MaterialRegistry, Phase, AIR, ASH, BLOOD, BONE, CLAY, COAL, CRYSTAL, DIRT, EMBER,
    FIRE, GLASS, GRANITE, GRAVEL, ICE, LAVA, MOLTEN_METAL, MUD, OIL, ORE, PLASMA, SALT, SALT_WATER,
    SAND, SNOW, SPARK, STONE, WATER,
};
use civ_voxel::worldgen;
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, MeshBuffer};
use crate::voxel_smooth_mesher::{
    build_smooth_meshes, resolved_mesher_mode, TerrainMesherMode, SMOOTH_MESH_PADDED_EDGE,
};

use crate::camera::CameraRig;
use crate::{bevy_render::mesh_buffer_to_bevy, chunk_distance_from_camera, should_render_chunk};

/// Fallback seed used only when no `WorldSetupParams` resource is available
/// (i.e. the `egui` menu feature is compiled out). With menus present, the
/// per-world randomized seed from `WorldSetupParams` is always used instead.
const SEED: u64 = 0xC1F1_5EED_D3AD_BEEF;
// CA tick rate. At 256³ each step is a full-grid multi-pass sweep + full
// remesh, so 12 Hz froze the frame loop. 2 Hz is the throttle until
// dirty-chunk stepping + incremental remesh land (see FR-CIV-CA dirty-chunk TODO).
const CA_TICK_HZ: f32 = 2.0;
const CHUNK_EDGE: usize = 16;
const RENDER_MAX_DIST: f32 = 160.0;
const SMOOTH_CUBIC_FALLBACK_DIST: f32 = 120.0;

fn smooth_far_distance() -> f32 {
    let configured = std::env::var("CIVIS_SMOOTH_FAR_DIST")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(SMOOTH_CUBIC_FALLBACK_DIST);

    configured.max(RENDER_MAX_DIST)
}

#[cfg(test)]
mod smooth_mesh_distance_tests {
    use super::smooth_far_distance;
    use super::RENDER_MAX_DIST;

    #[test]
    fn smooth_far_distance_defaults_to_render_max_when_env_unset_or_invalid() {
        let prev = std::env::var_os("CIVIS_SMOOTH_FAR_DIST");
        std::env::remove_var("CIVIS_SMOOTH_FAR_DIST");

        let resolved = smooth_far_distance();
        assert!(resolved >= RENDER_MAX_DIST);

        std::env::set_var("CIVIS_SMOOTH_FAR_DIST", "not_a_number");
        let resolved_invalid = smooth_far_distance();
        assert!(resolved_invalid >= RENDER_MAX_DIST);

        match prev {
            Some(value) => std::env::set_var("CIVIS_SMOOTH_FAR_DIST", value),
            None => std::env::remove_var("CIVIS_SMOOTH_FAR_DIST"),
        }
    }

    #[test]
    fn smooth_far_distance_reads_env_and_respects_mandated_minimum() {
        let prev = std::env::var_os("CIVIS_SMOOTH_FAR_DIST");

        std::env::set_var("CIVIS_SMOOTH_FAR_DIST", "256");
        assert_eq!(smooth_far_distance(), 256.0);

        std::env::set_var("CIVIS_SMOOTH_FAR_DIST", "40");
        assert!(smooth_far_distance() >= RENDER_MAX_DIST);

        match prev {
            Some(value) => std::env::set_var("CIVIS_SMOOTH_FAR_DIST", value),
            None => std::env::remove_var("CIVIS_SMOOTH_FAR_DIST"),
        }
    }
}

// Playable MVP surface. 256³ = 4096 chunks meshed SYNCHRONOUSLY on load = a
// multi-minute main-thread freeze (no async/streamed mesh yet). 96³ = 216
// chunks loads in ~1-2s and is a genuine ~0.2mi² sandbox. Scale back to 256³
// once worldgen + initial mesh stream over frames (FR-CIV-SCALE async-load
// TODO) + dirty-chunk CA lands.
const WORLD_DIMS: [usize; 3] = [96, 64, 96];

/// Tracks whether the live voxel world has been generated for the current
/// session. Set `true` after a build; reset to `false` to force regeneration
/// of a genuinely new world (e.g. on returning to the menu / "New World").
#[derive(Resource, Default)]
pub struct WorldBuilt(pub bool);

/// Voxel simulation plugin for the Bevy reference client.
pub struct VoxelSimPlugin;

impl Plugin for VoxelSimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VoxelSimState::default())
            .insert_resource(WorldBuilt::default())
            .add_systems(Update, step_and_remesh);

        // With the menu/egui feature, worldgen is deferred until the user
        // commits World Setup → Loading (or autostart jumps to Playing). This
        // means the world is built from the player's per-world randomized seed
        // and is NOT rendered behind the World Setup popup.
        #[cfg(feature = "egui")]
        app.add_systems(Update, build_world_on_play);

        // Without menus there is no WorldSetupParams; fall back to building one
        // world at Startup using the constant seed.
        #[cfg(not(feature = "egui"))]
        app.add_systems(Startup, setup_voxel_world_startup);
    }
}

/// When menus are absent, build a single world at Startup from the fallback seed.
#[cfg(not(feature = "egui"))]
pub fn setup_voxel_world_startup(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    rig: ResMut<CameraRig>,
    state: ResMut<VoxelSimState>,
    mut built: ResMut<WorldBuilt>,
    cameras: Query<&Transform, With<Camera3d>>,
) {
    build_voxel_world(commands, meshes, materials, rig, state, cameras, SEED);
    built.0 = true;
}

/// Generate (or regenerate) the world once the game enters Loading/Playing,
/// using the player's per-world seed. Despawns any prior world first so a new
/// world is a genuine replacement, never an overlay. Resets `WorldBuilt` when
/// the player returns to a menu so the next world regenerates fresh.
#[cfg(feature = "egui")]
pub fn build_world_on_play(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    rig: ResMut<CameraRig>,
    mut state: ResMut<VoxelSimState>,
    mut built: ResMut<WorldBuilt>,
    mode: Res<crate::menus::GameUiMode>,
    params: Res<crate::menus::WorldSetupParams>,
    cameras: Query<&Transform, With<Camera3d>>,
) {
    use crate::menus::GameUiMode;
    // Reset so a fresh world is generated next time the player commits setup.
    if matches!(*mode, GameUiMode::MainMenu | GameUiMode::WorldSetup) {
        if built.0 {
            for entity in state.chunk_entities.drain().flat_map(|(_, e)| e.into_iter()) {
                commands.entity(entity).despawn();
            }
            built.0 = false;
        }
        return;
    }
    // Build exactly once per world when entering Loading/Playing.
    if built.0 {
        return;
    }
    // Despawn any stale world entities before regenerating.
    for entity in state.chunk_entities.drain().flat_map(|(_, e)| e.into_iter()) {
        commands.entity(entity).despawn();
    }
    let seed = params.seed;
    build_voxel_world(commands, meshes, materials, rig, state, cameras, seed);
    built.0 = true;
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
    pub chunk_entities: HashMap<ChunkId, Vec<Entity>>,
}

impl Default for VoxelSimState {
    fn default() -> Self {
        Self {
            grid: CaGrid {
                dims: [0, 0, 0],
                cells: Vec::new(),
                temperatures: Vec::new(),
                saturation: Vec::new(),
                dirty_chunks: HashSet::new(),
            },
            tick: 0,
            accumulator: 0.0,
            chunk_entities: HashMap::new(),
        }
    }
}

/// Return the top of the first non-AIR voxel at `(x, z)` in world-space.
/// Used by Bevy sim bridge placement so actor/building spawns match voxel terrain.
#[must_use]
pub fn voxel_surface_y(grid: &CaGrid, x: f32, z: f32) -> f32 {
    let max_x = (grid.dims[0].max(1) - 1) as f32;
    let max_z = (grid.dims[2].max(1) - 1) as f32;
    let xi = x.clamp(0.0, max_x).floor() as usize;
    let zi = z.clamp(0.0, max_z).floor() as usize;
    for yi in (0..grid.dims[1]).rev() {
        if grid.get(xi, yi, zi) != AIR {
            return yi as f32 + 1.0;
        }
    }
    0.0
}

/// Build the voxel world from `seed`, then mesh and spawn all visible chunks.
pub fn build_voxel_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rig: ResMut<CameraRig>,
    mut state: ResMut<VoxelSimState>,
    cameras: Query<&Transform, With<Camera3d>>,
    seed: u64,
) {
    info!("[voxel] generating world from seed={seed:#018x}");
    let generated = worldgen::generate(WORLD_DIMS, seed);
    let cell_count = generated.cells.len();
    state.grid = CaGrid {
        dims: generated.dims,
        cells: generated.cells,
        temperatures: vec![20; cell_count],
        saturation: vec![0; cell_count],
        dirty_chunks: HashSet::new(),
    };
    state.grid.mark_mobile_chunks(MaterialRegistry::standard());
    state.tick = 0;
    state.accumulator = 0.0;
    state.chunk_entities.clear();

    // --- Diagnostic: world dims + non-air cell census so we can prove the
    // generator actually filled solid material (an empty grid = invisible). ---
    let dims = state.grid.dims;
    let total_cells = dims[0] * dims[1] * dims[2];
    let mut non_air = 0usize;
    let mut max_solid_y = 0usize;
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                if state.grid.get(x, y, z) != AIR {
                    non_air += 1;
                    if y > max_solid_y {
                        max_solid_y = y;
                    }
                }
            }
        }
    }
    info!(
        "[voxel] world dims={:?} total_cells={} non_air={} ({:.1}%) max_solid_y={} AABB=(0,0,0)..({},{},{})",
        dims,
        total_cells,
        non_air,
        100.0 * non_air as f32 / total_cells.max(1) as f32,
        max_solid_y,
        dims[0],
        dims[1],
        dims[2],
    );

    let target = Vec3::new(
        generated.dims[0] as f32 * 0.5,
        generated.dims[1] as f32 * 0.5,
        generated.dims[2] as f32 * 0.5,
    );
    reframe_camera_once(&mut rig, target);
    info!(
        "[voxel] camera reframed target={:?} yaw={:.2} pitch={:.2} distance={:.1}",
        rig.target, rig.yaw, rig.pitch, rig.distance
    );
    let camera_eye = cameras.iter().next().map(|t| t.translation.to_array());
    info!("[voxel] camera_eye at spawn time = {:?}", camera_eye);
    state.chunk_entities = spawn_chunk_meshes(
        &mut commands,
        &mut meshes,
        &mut materials,
        &state.grid,
        None,
        None,
    );
    info!(
        "[voxel] spawned {} chunk-submesh entities",
        state.chunk_entities.len()
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
    let step_dt = 1.0 / CA_TICK_HZ;
    if state.accumulator < step_dt {
        return;
    }
    // Single step per frame (never catch-up-spiral: at 256³ each step is a
    // full-grid multi-pass sweep, so running N steps in one frame freezes).
    // Clamp the accumulator so a long stall doesn't queue a backlog.
    state.accumulator = (state.accumulator - step_dt).min(step_dt);
    let changed = step(&mut state.grid, MaterialRegistry::standard());
    if !changed {
        return;
    }
    state.tick = state.tick.wrapping_add(1);
    let chunk_counts = [
        state.grid.dims[0].div_ceil(CHUNK_EDGE),
        state.grid.dims[1].div_ceil(CHUNK_EDGE),
        state.grid.dims[2].div_ceil(CHUNK_EDGE),
    ];
    let changed_chunks: HashSet<ChunkId> = state
        .grid
        .dirty_chunks()
        .into_iter()
        .map(|chunk| {
            let cx = chunk % chunk_counts[0];
            let rem = chunk - cx;
            let cy = rem / chunk_counts[0] % chunk_counts[1];
            let cz = rem / (chunk_counts[0] * chunk_counts[1]);
            ChunkId(((cx as u64) << 40) | ((cy as u64) << 16) | (cz as u64))
        })
        .collect();
    if changed_chunks.is_empty() {
        return;
    }
    let stale: Vec<ChunkId> = changed_chunks
        .iter()
        .filter(|chunk| state.chunk_entities.contains_key(chunk))
        .cloned()
        .collect();
    for chunk_id in stale {
        let Some(entities) = state.chunk_entities.remove(&chunk_id) else {
            continue;
        };
        for entity in entities {
            commands.entity(entity).despawn();
        }
    }
    let camera_eye = cameras.iter().next().map(|t| t.translation.to_array());
    let rebuilt = spawn_chunk_meshes(
        &mut commands,
        &mut meshes,
        &mut materials,
        &state.grid,
        Some(&changed_chunks),
        camera_eye,
    );
    for (id, entities) in rebuilt {
        state.chunk_entities.insert(id, entities);
    }
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

fn slice_chunk_with_apron(
    grid: &CaGrid,
    cx: usize,
    cy: usize,
    cz: usize,
) -> [MaterialId; SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE] {
    // Padded chunk for the smooth mesher: a 2-voxel apron of neighbour-chunk
    // voxels on every side so the 5x5x5 blur + Surface Nets round across chunk
    // seams. PADDED_EDGE + APRON are kept in sync with the mesher via
    // `SMOOTH_MESH_PADDED_EDGE`; APRON is half the padding on each side.
    const PADDED_EDGE: usize = SMOOTH_MESH_PADDED_EDGE;
    const APRON: isize = ((PADDED_EDGE - CHUNK_EDGE) / 2) as isize;
    let origin_x = cx * CHUNK_EDGE;
    let origin_y = cy * CHUNK_EDGE;
    let origin_z = cz * CHUNK_EDGE;
    let mut voxels = [AIR; PADDED_EDGE * PADDED_EDGE * PADDED_EDGE];
    for z in 0..PADDED_EDGE {
        for y in 0..PADDED_EDGE {
            for x in 0..PADDED_EDGE {
                let gx_i = isize::try_from(origin_x + x).unwrap_or(isize::MAX) - APRON;
                let gy_i = isize::try_from(origin_y + y).unwrap_or(isize::MAX) - APRON;
                let gz_i = isize::try_from(origin_z + z).unwrap_or(isize::MAX) - APRON;
                let gx = isize::try_from(grid.dims[0]).unwrap_or(isize::MAX);
                let gy = isize::try_from(grid.dims[1]).unwrap_or(isize::MAX);
                let gz_max = isize::try_from(grid.dims[2]).unwrap_or(isize::MAX);
                let idx = x + y * PADDED_EDGE + z * PADDED_EDGE * PADDED_EDGE;
                voxels[idx] = if gx_i < 0
                    || gy_i < 0
                    || gz_i < 0
                    || gx_i >= gx
                    || gy_i >= gy
                    || gz_i >= gz_max
                {
                    AIR
                } else {
                    grid.get(
                        usize::try_from(gx_i).unwrap_or_default(),
                        usize::try_from(gy_i).unwrap_or_default(),
                        usize::try_from(gz_i).unwrap_or_default(),
                    )
                };
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

fn should_use_smooth_mesh(
    chunk_id: ChunkId,
    camera_eye: Option<[f32; 3]>,
    mode: TerrainMesherMode,
) -> bool {
    match mode {
        TerrainMesherMode::Cubic => false,
        TerrainMesherMode::Smooth => {
            let Some(eye) = camera_eye else {
                return true;
            };
            chunk_distance_from_camera(chunk_id, eye, CHUNK_EDGE as f32) <= smooth_far_distance()
        }
    }
}

fn chunk_saturation(
    grid: &CaGrid,
    cx: usize,
    cy: usize,
    cz: usize,
) -> Option<Vec<u8>> {
    if grid.saturation.is_empty() {
        return None;
    }
    let mut saturation = Vec::with_capacity(CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE);
    for z in 0..CHUNK_EDGE {
        for y in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                let gx = cx * CHUNK_EDGE + x;
                let gy = cy * CHUNK_EDGE + y;
                let gz = cz * CHUNK_EDGE + z;
                if gx >= grid.dims[0] || gy >= grid.dims[1] || gz >= grid.dims[2] {
                    saturation.push(0);
                    continue;
                }
                let idx = gx + gy * grid.dims[0] + gz * grid.dims[0] * grid.dims[1];
                saturation.push(grid.saturation[idx]);
            }
        }
    }
    Some(saturation)
}

fn chunk_saturation_with_apron(
    grid: &CaGrid,
    cx: usize,
    cy: usize,
    cz: usize,
) -> Option<Vec<u8>> {
    if grid.saturation.is_empty() {
        return None;
    }
    // Saturation apron MUST match the voxel apron edge/offset (SMOOTH_MESH_PADDED_EDGE,
    // 2-voxel apron) so per-cell saturation aligns with the blurred occupancy samples.
    const PADDED_EDGE: usize = SMOOTH_MESH_PADDED_EDGE;
    const APRON: isize = ((PADDED_EDGE - CHUNK_EDGE) / 2) as isize;
    let origin_x = cx * CHUNK_EDGE;
    let origin_y = cy * CHUNK_EDGE;
    let origin_z = cz * CHUNK_EDGE;
    let mut saturation = Vec::with_capacity(PADDED_EDGE * PADDED_EDGE * PADDED_EDGE);
    for z in 0..PADDED_EDGE {
        for y in 0..PADDED_EDGE {
            for x in 0..PADDED_EDGE {
                let gx_i = isize::try_from(origin_x + x).unwrap_or(isize::MAX) - APRON;
                let gy_i = isize::try_from(origin_y + y).unwrap_or(isize::MAX) - APRON;
                let gz_i = isize::try_from(origin_z + z).unwrap_or(isize::MAX) - APRON;
                let gx = isize::try_from(grid.dims[0]).unwrap_or(isize::MAX);
                let gy = isize::try_from(grid.dims[1]).unwrap_or(isize::MAX);
                let gz_max = isize::try_from(grid.dims[2]).unwrap_or(isize::MAX);
                if gx_i < 0
                    || gy_i < 0
                    || gz_i < 0
                    || gx_i >= gx
                    || gy_i >= gy
                    || gz_i >= gz_max
                {
                    saturation.push(0);
                    continue;
                }
                let gx = usize::try_from(gx_i).unwrap_or_default();
                let gy = usize::try_from(gy_i).unwrap_or_default();
                let gz = usize::try_from(gz_i).unwrap_or_default();
                let idx = gx + gy * grid.dims[0] + gz * grid.dims[0] * grid.dims[1];
                saturation.push(grid.saturation[idx]);
            }
        }
    }
    Some(saturation)
}

/// Mesh one chunk, split it by material, and spawn world-offset Bevy entities.
fn spawn_chunk_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    grid: &CaGrid,
    filter: Option<&HashSet<ChunkId>>,
    camera_eye: Option<[f32; 3]>,
) -> HashMap<ChunkId, Vec<Entity>> {
    let chunk_counts = [
        grid.dims[0].div_ceil(CHUNK_EDGE),
        grid.dims[1].div_ceil(CHUNK_EDGE),
        grid.dims[2].div_ceil(CHUNK_EDGE),
    ];
    let registry = MaterialRegistry::standard();
    let mut spawned: HashMap<ChunkId, Vec<Entity>> = HashMap::new();
    for cz in 0..chunk_counts[2] {
        for cy in 0..chunk_counts[1] {
            for cx in 0..chunk_counts[0] {
                let chunk_origin = [cx * CHUNK_EDGE, cy * CHUNK_EDGE, cz * CHUNK_EDGE];
                let chunk_id = ChunkId(((cx as u64) << 40) | ((cy as u64) << 16) | (cz as u64));
                if let Some(filters) = filter {
                    if !filters.contains(&chunk_id) {
                        continue;
                    }
                }
                if let Some(eye) = camera_eye {
                    if !should_render_chunk(chunk_id, eye, RENDER_MAX_DIST) {
                        continue;
                    }
                }
                let voxels = slice_chunk(grid, cx, cy, cz);
                let padded_voxels = slice_chunk_with_apron(grid, cx, cy, cz);
                let view = ChunkView {
                    id: chunk_id,
                    voxels: &voxels,
                };
                let mode = resolved_mesher_mode();
                let use_smooth = should_use_smooth_mesh(chunk_id, camera_eye, mode);
                let mut mesh_buffers: Vec<MeshBuffer> = if use_smooth {
                    let saturation = chunk_saturation_with_apron(grid, cx, cy, cz);
                    build_smooth_meshes(&voxels, &padded_voxels, saturation.as_deref(), &registry)
                } else {
                    match CubicMesher::mesh_cubic(view, LodLevel(0)) {
                        Ok(mesh_buffer) => split_by_material(&mesh_buffer)
                            .into_iter()
                            .map(|(_, submesh)| submesh)
                            .collect(),
                        Err(_) => Vec::new(),
                    }
                };
                if use_smooth && mesh_buffers.is_empty() && voxels.iter().any(|v| *v != AIR) {
                    match CubicMesher::mesh_cubic(view, LodLevel(0)) {
                        Ok(mesh_buffer) => {
                            mesh_buffers.extend(
                                split_by_material(&mesh_buffer)
                                    .into_iter()
                                    .map(|(_, submesh)| submesh),
                            );
                        }
                        Err(_) => {}
                    }
                }
                if mesh_buffers.is_empty() {
                    continue;
                }
                for submesh in mesh_buffers {
                    let material_id = match submesh.vertices.first() {
                        Some(v) => v.material,
                        None => continue,
                    };
                    let Some(def) = registry.get(material_id) else {
                        continue;
                    };
                    let material = pbr_material_for(material_id, def);
                    let mut bevy_mesh = mesh_buffer_to_bevy(&submesh);
                    apply_voxel_jitter(&mut bevy_mesh, &submesh, material_id);
                    let entity = commands
                        .spawn((
                            Mesh3d(meshes.add(bevy_mesh)),
                            MeshMaterial3d(materials.add(material)),
                            Transform::from_xyz(
                                chunk_origin[0] as f32,
                                chunk_origin[1] as f32,
                                chunk_origin[2] as f32,
                            ),
                        ))
                        .id();
                    spawned.entry(chunk_id).or_default().push(entity);
                }
            }
        }
    }
    spawned
}

/// Linearized base color from a material's sRGB hint.
fn base_color(def: &MaterialDef) -> Color {
    Color::srgba(
        f32::from(def.color[0]) / 255.0,
        f32::from(def.color[1]) / 255.0,
        f32::from(def.color[2]) / 255.0,
        f32::from(def.color[3]) / 255.0,
    )
}

/// Build a perceptually-tuned [`StandardMaterial`] for a voxel material family.
///
/// Instead of a flat RGB base color, each family gets characteristic PBR
/// parameters: matte powders, glossy/translucent liquids, glowing hot
/// materials, refractive crystal/glass, and metallic ores. This is what makes
/// stone read as stone and lava read as molten rather than as paint chips.
#[must_use]
fn pbr_material_for(id: MaterialId, def: &MaterialDef) -> StandardMaterial {
    let mut mat = StandardMaterial {
        base_color: base_color(def),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        reflectance: 0.5,
        // Voxel hull faces at chunk boundaries can present with a normal/winding
        // that leaves them facing away from the lit hemisphere, rendering pure
        // black. Light both sides so every visible face reads regardless of the
        // emitted normal orientation. `cull_mode = None` keeps back faces drawn.
        double_sided: true,
        cull_mode: None,
        ..default()
    };

    // Emissive families: glow proportional to heat. Color biased warm for
    // combustion, magenta for plasma, yellow-white for sparks.
    let emissive = |r: f32, g: f32, b: f32, strength: f32| {
        LinearRgba::new(r * strength, g * strength, b * strength, 1.0)
    };

    match id {
        // --- Liquids: smooth, specular, partly transparent. ---
        WATER | SALT_WATER => {
            mat.perceptual_roughness = 0.08;
            mat.reflectance = 0.55;
            mat.base_color = mat.base_color.with_alpha(0.62);
            mat.alpha_mode = AlphaMode::Blend;
            mat.specular_transmission = 0.25;
            mat.ior = 1.33;
        }
        OIL => {
            mat.perceptual_roughness = 0.12;
            mat.reflectance = 0.45;
            mat.base_color = mat.base_color.with_alpha(0.9);
            mat.alpha_mode = AlphaMode::Blend;
        }
        BLOOD => {
            mat.perceptual_roughness = 0.18;
            mat.reflectance = 0.45;
        }
        MUD => {
            mat.perceptual_roughness = 0.85;
            mat.reflectance = 0.25;
        }
        // --- Molten / hot: glow. ---
        LAVA => {
            mat.perceptual_roughness = 0.55;
            mat.emissive = emissive(1.0, 0.32, 0.06, 6.0);
        }
        MOLTEN_METAL => {
            mat.perceptual_roughness = 0.4;
            mat.metallic = 0.7;
            mat.emissive = emissive(1.0, 0.7, 0.3, 3.0);
        }
        FIRE => {
            mat.perceptual_roughness = 1.0;
            mat.base_color = mat.base_color.with_alpha(0.7);
            mat.alpha_mode = AlphaMode::Add;
            mat.emissive = emissive(1.0, 0.45, 0.12, 9.0);
        }
        EMBER => {
            mat.perceptual_roughness = 0.8;
            mat.emissive = emissive(1.0, 0.3, 0.07, 4.0);
        }
        SPARK => {
            mat.alpha_mode = AlphaMode::Add;
            mat.emissive = emissive(1.0, 0.92, 0.55, 10.0);
        }
        PLASMA => {
            mat.base_color = mat.base_color.with_alpha(0.6);
            mat.alpha_mode = AlphaMode::Add;
            mat.emissive = emissive(1.0, 0.25, 0.75, 12.0);
        }
        // --- Translucent solids: ice, glass, crystal refract light. ---
        ICE => {
            mat.perceptual_roughness = 0.1;
            mat.reflectance = 0.6;
            mat.base_color = mat.base_color.with_alpha(0.7);
            mat.alpha_mode = AlphaMode::Blend;
            mat.specular_transmission = 0.35;
            mat.ior = 1.31;
        }
        GLASS => {
            mat.perceptual_roughness = 0.03;
            mat.reflectance = 0.6;
            mat.base_color = mat.base_color.with_alpha(0.35);
            mat.alpha_mode = AlphaMode::Blend;
            mat.specular_transmission = 0.85;
            mat.ior = 1.5;
        }
        CRYSTAL => {
            mat.perceptual_roughness = 0.05;
            mat.reflectance = 0.65;
            mat.base_color = mat.base_color.with_alpha(0.55);
            mat.alpha_mode = AlphaMode::Blend;
            mat.specular_transmission = 0.6;
            mat.ior = 1.6;
            mat.emissive = emissive(0.3, 0.6, 1.0, 0.4);
        }
        // --- Metallic / ore. ---
        ORE => {
            mat.perceptual_roughness = 0.55;
            mat.metallic = 0.65;
            mat.reflectance = 0.55;
        }
        // --- Hard rock: low-ish roughness, slight reflectance. ---
        STONE | GRANITE => {
            mat.perceptual_roughness = 0.82;
            mat.reflectance = 0.35;
        }
        // --- Powders & soils: very matte, no specular. ---
        SAND | DIRT | GRAVEL | CLAY | SALT => {
            mat.perceptual_roughness = 0.97;
            mat.reflectance = 0.18;
        }
        ASH | COAL => {
            mat.perceptual_roughness = 1.0;
            mat.reflectance = 0.08;
        }
        SNOW => {
            mat.perceptual_roughness = 0.85;
            mat.reflectance = 0.5;
        }
        BONE => {
            mat.perceptual_roughness = 0.7;
            mat.reflectance = 0.3;
        }
        _ => {
            // Phase-based fallback for anything not explicitly tuned.
            match def.phase {
                Phase::Liquid => {
                    mat.perceptual_roughness = 0.2;
                    mat.base_color = mat.base_color.with_alpha(0.7);
                    mat.alpha_mode = AlphaMode::Blend;
                }
                Phase::Gas => {
                    mat.perceptual_roughness = 1.0;
                    mat.base_color = mat.base_color.with_alpha(0.45);
                    mat.alpha_mode = AlphaMode::Blend;
                }
                Phase::Powder => {
                    mat.perceptual_roughness = 0.95;
                    mat.reflectance = 0.2;
                }
                Phase::Solid | Phase::Empty => {
                    mat.perceptual_roughness = 0.8;
                }
            }
        }
    }

    // Gas alpha already encodes translucency via the palette; honor it.
    if matches!(def.phase, Phase::Gas) && mat.alpha_mode == AlphaMode::Opaque {
        mat.base_color = base_color(def);
        mat.alpha_mode = AlphaMode::Blend;
    }

    mat
}

/// Per-voxel hue/value jitter written as vertex colors so a flat material face
/// breaks up into natural variation (a stone wall stops looking like one grey
/// quad). Emissive/transparent families are skipped to avoid flickering glow.
fn apply_voxel_jitter(mesh: &mut Mesh, buf: &MeshBuffer, id: MaterialId) {
    // Hot/transparent materials should stay uniform; jitter only opaque solids,
    // powders and dense liquids.
    if matches!(id, FIRE | SPARK | PLASMA | LAVA | EMBER | GLASS | CRYSTAL) {
        return;
    }
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(buf.vertices.len());
    for v in &buf.vertices {
        // Quantize to the voxel cell so all verts of a face share one tint.
        let cell = [
            v.position[0].floor() as i64,
            v.position[1].floor() as i64,
            v.position[2].floor() as i64,
        ];
        let h = hash3(cell[0], cell[1], cell[2]);
        // Map hash to [-1, 1] then to a small multiplicative value jitter and a
        // tiny hue shift via per-channel offsets.
        let n = ((h & 0xFFFF) as f32 / 65535.0) * 2.0 - 1.0;
        let n2 = (((h >> 16) & 0xFFFF) as f32 / 65535.0) * 2.0 - 1.0;
        let value = 1.0 + n * 0.14; // +/-14% brightness
        let warm = n2 * 0.05; // subtle warm/cool tilt
        colors.push([
            (value + warm).clamp(0.6, 1.4),
            value.clamp(0.6, 1.4),
            (value - warm).clamp(0.6, 1.4),
            1.0,
        ]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

/// Cheap integer hash (splitmix-style) for stable per-cell jitter.
#[must_use]
fn hash3(x: i64, y: i64, z: i64) -> u64 {
    let mut h = (x as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h
}
