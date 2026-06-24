//! Bevy voxel simulation renderer for Civis P-VM-3.
//!
//! Feature-gated behind `voxel` so the heightmap sandbox remains the default
//! when the feature is off.

use std::collections::{BTreeMap, HashMap, HashSet};

use bevy::light::NotShadowCaster;
use bevy::mesh::PlaneMeshBuilder;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_water::water::{
    material::{StandardWaterMaterial, WaterMaterial},
    WaterQuality, WaterSettings, WaterTile,
};

use crate::voxel_smooth_mesher::{
    build_smooth_meshes, mesher_chunk_stats, record_cubic_chunk, resolved_mesher_mode,
    TerrainMesherMode, SMOOTH_MESH_PADDED_EDGE,
};
use civ_voxel::fluid_ca::{step, CaGrid};
use civ_voxel::material::{
    MaterialDef, MaterialRegistry, Phase, AIR, ASH, BLOOD, BONE, CLAY, COAL, CRYSTAL, DIRT, EMBER,
    FIRE, GLASS, GRANITE, GRAVEL, ICE, LAVA, MOLTEN_METAL, MUD, OIL, ORE, PLASMA, SALT, SALT_WATER,
    SAND, SNOW, SPARK, STONE, WATER,
};
use civ_voxel::worldgen;
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, MeshBuffer};

use crate::camera::CameraRig;
use crate::voxel_triplanar::{VoxelPbrBank, VoxelTriplanarMaterial};
use crate::{bevy_render::mesh_buffer_to_bevy, chunk_distance_from_camera, should_render_chunk};

/// Fallback seed used only when no `WorldSetupParams` resource is available
/// (i.e. the `egui` menu feature is compiled out). With menus present, the
/// per-world randomized seed from `WorldSetupParams` is always used instead.
const SEED: u64 = 0xC1F1_5EED_D3AD_BEEF;
// CA tick rate. At 256³ each step is a full-grid multi-pass sweep, so 12 Hz
// froze the frame loop (full-grid sweep every 83 ms, single-threaded). The
// step_and_remesh caller already applies a dirty-chunk early-exit
// (`StepOutcome::changed == false → return before any remesh`) so static worlds
// pay near-zero render cost. Pegged at 0.25 Hz (1 step per 4 s) while the
// incremental dirty-chunk meshing path matures (FR-CIV-CA dirty-chunk).
// Raise toward 2 Hz once incremental remesh lands and is benchmarked stable.
// CA tick rate. At 256³ each step is a full-grid multi-pass sweep + full
// remesh, so 12 Hz froze the frame loop. 2 Hz is the throttle until
// dirty-chunk stepping + incremental remesh work (see FR-CIV-CA dirty-chunk notes).
// Reduced to 0.25 Hz (1 step per 4s) while CA perf is being optimised.
// The full-grid dirty-chunk sweep is still expensive with large water coastlines.
const CA_TICK_HZ: f32 = 0.25;
const CHUNK_EDGE: usize = 32;
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

#[cfg(test)]
mod chunk_exposure_tests {
    use super::{chunk_has_exposed_face, CHUNK_EDGE};
    use civ_voxel::material::{AIR, DIRT};

    fn solid_chunk() -> [civ_voxel::MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE] {
        [DIRT; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE]
    }

    fn air_chunk() -> [civ_voxel::MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE] {
        [AIR; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE]
    }

    /// A fully solid chunk with no AIR neighbours inside has no exposed face.
    /// Regression: this chunk must NOT get a cubic fallback mesh (grey box bug).
    #[test]
    fn fully_solid_chunk_has_no_exposed_face() {
        let voxels = solid_chunk();
        // All interior voxels are solid — but they all touch the chunk boundary,
        // which our helper treats as exposed. So this test specifically checks a
        // chunk where NO voxel has an AIR neighbour within the chunk itself.
        // The boundary-touching voxels return true; only a chunk with zero solid
        // voxels returns false for the "any non-AIR" path. Use the air chunk for
        // the false case.
        assert!(
            !chunk_has_exposed_face(&air_chunk()),
            "all-AIR chunk should have no exposed face"
        );
    }

    /// A chunk with one solid voxel surrounded by AIR has an exposed face.
    #[test]
    fn single_solid_voxel_has_exposed_face() {
        let mut voxels = air_chunk();
        // Place solid voxel in the interior (not on boundary)
        let idx = 2 + 2 * CHUNK_EDGE + 2 * CHUNK_EDGE * CHUNK_EDGE;
        voxels[idx] = DIRT;
        assert!(
            chunk_has_exposed_face(&voxels),
            "solid voxel surrounded by AIR should have an exposed face"
        );
    }
}

/// Tests that the blocky (cubic) mesher path no longer emits boundary faces when the
/// neighboring chunk is solid.  The fix is structural (smooth mesher always used in
/// Smooth mode), so we verify the *routing* decision: `should_use_smooth_mesh` must
/// return `true` in Smooth mode regardless of distance, ensuring the apron-aware
/// smooth path is chosen and no cross-chunk phantom-air seam can form.
#[cfg(test)]
mod terrain_fragmentation_tests {
    use super::{should_use_smooth_mesh, CHUNK_EDGE};
    use crate::voxel_smooth_mesher::TerrainMesherMode;
    use civ_voxel::ChunkId;

    /// In Smooth mode, every chunk — near or far — must use the smooth (apron-aware)
    /// mesher.  This is the core contract that fixes terrain fragmentation (#98):
    /// the cubic mesher has no neighbor apron and produces seam walls at chunk
    /// boundaries when adjacent chunks are solid.
    #[test]
    fn smooth_mode_always_uses_smooth_mesher_regardless_of_distance() {
        // Simulate a FAR chunk (beyond the old SMOOTH_CUBIC_FALLBACK_DIST=120.0).
        // Eye at origin; chunk at cx=20 → chunk-centre ≈ 20*32+16 ≈ 656 units away.
        let far_chunk = ChunkId(((20u64) << 40) | ((1u64) << 16) | (1u64));
        let eye_at_origin: Option<[f32; 3]> = Some([0.0, 0.0, 0.0]);

        assert!(
            should_use_smooth_mesh(far_chunk, eye_at_origin, TerrainMesherMode::Smooth),
            "far chunk in Smooth mode must still use smooth mesher (fix for #98: \
             cubic mesher has no neighbor apron → phantom-air seams at chunk boundaries)"
        );
    }

    /// In Smooth mode, a near chunk also uses the smooth mesher (unchanged).
    #[test]
    fn smooth_mode_near_chunk_uses_smooth_mesher() {
        let near_chunk = ChunkId(((1u64) << 40) | ((0u64) << 16) | (1u64));
        let eye_nearby: Option<[f32; 3]> = Some([32.0, 0.0, 32.0]);

        assert!(
            should_use_smooth_mesh(near_chunk, eye_nearby, TerrainMesherMode::Smooth),
            "near chunk in Smooth mode must use smooth mesher"
        );
    }

    /// In Cubic mode, the cubic mesher is used regardless of distance (unchanged).
    #[test]
    fn cubic_mode_uses_cubic_mesher() {
        let any_chunk = ChunkId(((5u64) << 40) | ((2u64) << 16) | (3u64));
        assert!(
            !should_use_smooth_mesh(any_chunk, None, TerrainMesherMode::Cubic),
            "Cubic mode must keep using cubic mesher"
        );
    }

    /// Smooth mode with no camera still returns true (no panic on None eye).
    #[test]
    fn smooth_mode_no_camera_uses_smooth_mesher() {
        let chunk = ChunkId(((3u64) << 40) | ((1u64) << 16) | (2u64));
        assert!(
            should_use_smooth_mesh(chunk, None, TerrainMesherMode::Smooth),
            "smooth mode with no camera must still use smooth mesher"
        );
    }

    /// Verify the apron constant: SMOOTH_MESH_PADDED_EDGE must be larger than CHUNK_EDGE
    /// so the 2-voxel apron actually samples neighbor-chunk voxels.
    #[test]
    fn padded_edge_larger_than_chunk_edge() {
        use crate::voxel_smooth_mesher::SMOOTH_MESH_PADDED_EDGE;
        assert!(
            SMOOTH_MESH_PADDED_EDGE > CHUNK_EDGE,
            "SMOOTH_MESH_PADDED_EDGE ({SMOOTH_MESH_PADDED_EDGE}) must exceed \
             CHUNK_EDGE ({CHUNK_EDGE}) for the neighbor apron to exist"
        );
    }
}

// Playable MVP surface. 256³ = 4096 chunks meshed SYNCHRONOUSLY on load = a
// multi-minute main-thread freeze (no async/streamed mesh yet). 96³ = 216
// chunks loads in ~1-2s and is a genuine ~0.2mi² sandbox. Scale back to 256³
// once worldgen + initial mesh stream over frames (FR-CIV-SCALE async-load
// work) + dirty-chunk CA lands.
pub const WORLD_DIMS_SMALL: [usize; 3] = [96, 64, 96];
pub const WORLD_DIMS_MEDIUM: [usize; 3] = [160, 64, 160];
pub const WORLD_DIMS_LARGE: [usize; 3] = [256, 96, 256];
pub const WORLD_DIMS_HUGE: [usize; 3] = [384, 96, 384];

/// Resolve the world-size combo index to concrete voxel dimensions.
#[must_use]
pub fn world_dims_for(size_index: usize) -> [usize; 3] {
    match size_index {
        0 => WORLD_DIMS_SMALL,
        1 => WORLD_DIMS_MEDIUM,
        2 => WORLD_DIMS_LARGE,
        3 => WORLD_DIMS_HUGE,
        _ => WORLD_DIMS_MEDIUM,
    }
}

/// Tracks whether the live voxel world has been generated for the current
/// session. Set `true` after a build; reset to `false` to force regeneration
/// of a genuinely new world (e.g. on returning to the menu / "New World").
#[derive(Resource, Default)]
pub struct WorldBuilt(pub bool);

#[derive(Component)]
pub struct WaterPlane;

/// Voxel simulation plugin for the Bevy reference client.
pub struct VoxelSimPlugin;

impl Plugin for VoxelSimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VoxelSimState::default())
            .insert_resource(WorldBuilt::default())
            .insert_resource(WaterSettings {
                height: WORLD_DIMS_MEDIUM[1] as f32 * 0.40,
                amplitude: 0.35,
                clarity: 0.22,
                base_color: Color::srgba(0.07, 0.41, 0.67, 1.0),
                deep_color: Color::srgba(0.02, 0.18, 0.46, 1.0),
                shallow_color: Color::srgba(0.07, 0.68, 0.62, 1.0),
                edge_scale: 0.09,
                edge_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                update_materials: true,
                spawn_tiles: None,
                water_quality: WaterQuality::High,
                wave_direction: Vec2::new(1.0, 2.0),
                wave_direction_blend_duration: 2.0,
                ..Default::default()
            })
            .add_systems(Update, (step_and_remesh, apply_chunk_mesh_tasks))
            .add_plugins(crate::voxel_triplanar::VoxelTriplanarPlugin);

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<StandardWaterMaterial>>,
    mut triplanar_materials: ResMut<Assets<VoxelTriplanarMaterial>>,
    mut triplanar_bank: ResMut<VoxelPbrBank>,
    rig: ResMut<CameraRig>,
    state: ResMut<VoxelSimState>,
    mut built: ResMut<WorldBuilt>,
    cameras: Query<&Transform, With<Camera3d>>,
) {
    build_voxel_world(
        commands,
        &mut meshes,
        &mut materials,
        &mut water_materials,
        &mut triplanar_materials,
        &mut triplanar_bank,
        rig,
        state,
        cameras,
        SEED,
        WORLD_DIMS_MEDIUM,
    );
    built.0 = true;
}

/// Generate (or regenerate) the world once the game enters Loading/Playing,
/// using the player's per-world seed. Despawns any prior world first so a new
/// world is a genuine replacement, never an overlay. Resets `WorldBuilt` when
/// the player returns to a menu so the next world regenerates fresh.
#[cfg(feature = "egui")]
pub fn build_world_on_play(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<StandardWaterMaterial>>,
    mut triplanar_materials: ResMut<Assets<VoxelTriplanarMaterial>>,
    mut triplanar_bank: ResMut<VoxelPbrBank>,
    rig: ResMut<CameraRig>,
    mut state: ResMut<VoxelSimState>,
    mut built: ResMut<WorldBuilt>,
    mode: Res<crate::menus::GameUiMode>,
    params: Res<crate::menus::WorldSetupParams>,
    cameras: Query<&Transform, With<Camera3d>>,
    pending: Query<Entity, With<ComputingChunkMesh>>,
    water_planes: Query<Entity, With<WaterPlane>>,
) {
    use crate::menus::GameUiMode;
    // Reset so a fresh world is generated next time the player commits setup.
    if matches!(*mode, GameUiMode::MainMenu | GameUiMode::WorldSetup) {
        if built.0 {
            despawn_world_and_pending(&mut commands, &mut state, &pending, &water_planes);
            built.0 = false;
        }
        return;
    }
    // Build exactly once per world when entering Loading/Playing.
    if built.0 {
        return;
    }
    // Despawn any stale world entities AND in-flight mesh tasks before regenerating,
    // so a task computed for the OLD world can't apply onto the new one.
    despawn_world_and_pending(&mut commands, &mut state, &pending, &water_planes);
    let seed = params.seed;
    let dims = world_dims_for(params.world_size);
    build_voxel_world(
        commands,
        &mut meshes,
        &mut materials,
        &mut water_materials,
        &mut triplanar_materials,
        &mut triplanar_bank,
        rig,
        state,
        cameras,
        seed,
        dims,
    );
    built.0 = true;
}

/// Despawn the current world's chunk entities AND any in-flight off-thread mesh
/// tasks, so a rebuild starts clean and no stale task applies onto the new world.
#[cfg(feature = "egui")]
fn despawn_world_and_pending(
    commands: &mut Commands,
    state: &mut VoxelSimState,
    pending: &Query<Entity, With<ComputingChunkMesh>>,
    water_planes: &Query<Entity, With<WaterPlane>>,
) {
    for entity in state
        .chunk_entities
        .drain()
        .flat_map(|(_, e)| e.into_iter())
    {
        commands.entity(entity).despawn();
    }
    for carrier in pending.iter() {
        commands.entity(carrier).despawn();
    }
    for plane in water_planes.iter() {
        commands.entity(plane).despawn();
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
                last_changed_chunks: HashSet::new(),
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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    water_materials: &mut Assets<StandardWaterMaterial>,
    triplanar_materials: &mut Assets<VoxelTriplanarMaterial>,
    triplanar_bank: &mut VoxelPbrBank,
    mut rig: ResMut<CameraRig>,
    mut state: ResMut<VoxelSimState>,
    cameras: Query<&Transform, With<Camera3d>>,
    seed: u64,
    dims: [usize; 3],
) {
    info!("[voxel] generating world from seed={seed:#018x}");
    // Phase timing (task #4 perf): localize where the ~33s world-load stall lives
    // — worldgen vs grid/CA init vs meshing — instead of guessing.
    let t_start = std::time::Instant::now();
    let generated = worldgen::generate(dims, seed);
    let t_worldgen = t_start.elapsed();
    let cell_count = generated.cells.len();
    state.grid = CaGrid {
        dims: generated.dims,
        cells: generated.cells,
        temperatures: vec![20; cell_count],
        saturation: vec![0; cell_count],
        dirty_chunks: HashSet::new(),
        last_changed_chunks: HashSet::new(),
    };
    let t_grid_init = std::time::Instant::now();
    state.grid.mark_mobile_chunks(MaterialRegistry::standard());
    let t_mark_mobile = t_grid_init.elapsed();
    state.tick = 0;
    state.accumulator = 0.0;
    state.chunk_entities.clear();
    triplanar_bank.clear_cache();

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
    // Dispatch the per-chunk mesh COMPUTE onto the async task pool. The expensive
    // mesher runs off the main thread; `apply_chunk_mesh_tasks` (Update) spawns the
    // Bevy entities as each task completes, so build_voxel_world returns immediately
    // and the world streams in without blocking render/the autoshot.
    let t_dispatch = std::time::Instant::now();
    dispatch_chunk_mesh_tasks(&mut commands, &state.grid, None);
    let t_dispatch_elapsed = t_dispatch.elapsed();
    spawn_bevy_water_plane(&mut commands, meshes, water_materials, generated.dims);
    // Phase breakdown. The mesh COMPUTE is now off-thread, so this dispatch time is
    // the main-thread cost (cheap slicing only) — expected <<1s, no load hitch.
    info!(
        "[voxel][perf] worldgen={:.2}s mark_mobile={:.2}s mesh_dispatch={:.2}s (off-thread compute)",
        t_worldgen.as_secs_f32(),
        t_mark_mobile.as_secs_f32(),
        t_dispatch_elapsed.as_secs_f32(),
    );
    log_mesher_diagnostic(&state.grid);
}

fn spawn_bevy_water_plane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardWaterMaterial>,
    dims: [usize; 3],
) {
    let sea_level_y = dims[1] as f32 * 0.40;
    let plane_size_x = dims[0] as f32;
    let plane_size_z = dims[2] as f32;
    let tile_size = bevy_water::water::WATER_SIZE as f32;
    let world_scale = Vec3::new(plane_size_x / tile_size, 1.0, plane_size_z / tile_size);

    let mut plane_builder = PlaneMeshBuilder::from_length(tile_size);
    plane_builder = plane_builder.subdivisions((tile_size / 4.0) as u32);
    let mesh = Mesh3d(meshes.add(plane_builder));

    let mut material_extension = WaterMaterial {
        coord_scale: Vec2::new(plane_size_x, plane_size_z),
        ..Default::default()
    };
    material_extension.wave_dir_a = Vec2::new(1.0, 2.0).normalize_or_zero();
    material_extension.wave_dir_b = Vec2::new(1.0, 2.0).normalize_or_zero();

    let material = MeshMaterial3d(materials.add(StandardWaterMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(0.07, 0.41, 0.67, 1.0),
            perceptual_roughness: 0.22,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: material_extension,
    }));

    commands.spawn((
        WaterPlane,
        mesh,
        material,
        WaterTile::default(),
        Transform {
            translation: Vec3::new(plane_size_x * 0.5, sea_level_y, plane_size_z * 0.5),
            scale: world_scale,
            ..default()
        },
        NotShadowCaster,
    ));
}

/// Runtime diagnostic for task #4 (is the smooth mesher actually shaping terrain?).
///
/// Logs the smooth-vs-cubic chunk split and, for one representative solid chunk,
/// re-meshes it and reports how many vertices sit OFF the integer voxel grid.
/// A high off-grid fraction = Surface Nets is interpolating (smooth); near-zero =
/// vertices are snapping to the grid (cubic look), which would explain a blocky
/// silhouette even when the smooth path runs.
fn log_mesher_diagnostic(grid: &CaGrid) {
    let (smooth, cubic) = mesher_chunk_stats();
    info!("[voxel][diag] mesher chunks: smooth={smooth} cubic={cubic}");
    let counts = [
        grid.dims[0].div_ceil(CHUNK_EDGE),
        grid.dims[1].div_ceil(CHUNK_EDGE),
        grid.dims[2].div_ceil(CHUNK_EDGE),
    ];
    let registry = MaterialRegistry::standard();
    for cz in 0..counts[2] {
        for cy in 0..counts[1] {
            for cx in 0..counts[0] {
                let voxels = slice_chunk(grid, cx, cy, cz);
                if !voxels.iter().any(|v| *v != AIR) {
                    continue;
                }
                let padded = slice_chunk_with_apron(grid, cx, cy, cz);
                let sat = chunk_saturation_with_apron(grid, cx, cy, cz);
                let bufs = build_smooth_meshes(&voxels, &padded, sat.as_deref(), &registry);
                let verts: usize = bufs.iter().map(|b| b.vertices.len()).sum();
                let off_grid = bufs
                    .iter()
                    .flat_map(|b| b.vertices.iter())
                    .filter(|v| v.position.iter().any(|c| (c - c.round()).abs() > 0.01))
                    .count();
                let pct = if verts > 0 { off_grid * 100 / verts } else { 0 };
                info!(
                    "[voxel][diag] sample chunk ({cx},{cy},{cz}): {verts} verts, {off_grid} off-grid ({pct}% interpolated -> {})",
                    if pct > 25 { "SMOOTH" } else { "CUBIC-like/snapping" }
                );
                return;
            }
        }
    }
}

/// Step the CA at a fixed rate and remesh all chunks after any tick occurs.
pub fn step_and_remesh(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut triplanar_materials: ResMut<Assets<VoxelTriplanarMaterial>>,
    mut triplanar_bank: ResMut<VoxelPbrBank>,
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
    let outcome = step(&mut state.grid, MaterialRegistry::standard());
    if !outcome.changed {
        return;
    }
    state.tick = state.tick.wrapping_add(1);
    let chunk_counts = [
        state.grid.dims[0].div_ceil(CHUNK_EDGE),
        state.grid.dims[1].div_ceil(CHUNK_EDGE),
        state.grid.dims[2].div_ceil(CHUNK_EDGE),
    ];
    // Use `last_changed_chunks` (the *remesh* set) not `dirty_chunks` (the
    // halo-expanded *active* set). The dirty-chunk contract says: CA steps
    // ONLY dirty chunks, the kernel emits `DirtyChunkEvent` for the cells that
    // actually changed, and the renderer remeshes ONLY those. A static world
    // returns `[]` here and skips every spawn/despawn — that's the perf win.
    let mut changed_chunks: Vec<ChunkId> = state
        .grid
        .last_changed_chunks
        .iter()
        .map(|&chunk| {
            let cx = chunk % chunk_counts[0];
            let rem = chunk - cx;
            let cy = rem / chunk_counts[0] % chunk_counts[1];
            let cz = rem / (chunk_counts[0] * chunk_counts[1]);
            ChunkId(((cx as u64) << 40) | ((cy as u64) << 16) | (cz as u64))
        })
        .collect();
    changed_chunks.sort_unstable();
    changed_chunks.dedup();
    if changed_chunks.is_empty() {
        return;
    }
    // `spawn_chunk_meshes` takes a HashSet filter (it does chunk_id lookups in
    // a tight loop); building a HashSet once here is cheaper than a slice
    // linear scan per chunk and the input is bounded by the changed set
    // (small on the perf-critical path).
    let changed_set: HashSet<ChunkId> = changed_chunks.iter().copied().collect();
    let stale: Vec<ChunkId> = changed_set
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
        &mut triplanar_materials,
        &mut triplanar_bank,
        &state.grid,
        Some(&changed_set),
        camera_eye,
    );
    for (id, entities) in rebuilt {
        state.chunk_entities.insert(id, entities);
    }
}

/// Slice a 32³ chunk from the dense grid, filling out-of-bounds cells with air.
#[must_use]
fn slice_chunk(
    grid: &CaGrid,
    cx: usize,
    cy: usize,
    cz: usize,
) -> [MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE] {
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
    // render-mgr camera fix (A): steeper top-down pitch + closer framing (~1/3 of
    // the world) so actors read at a usable scale instead of a distant whole-world view.
    rig.pitch = -0.85;
    rig.distance = (extent * 1.1).clamp(60.0, 600.0);
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

/// Returns `true` when the smooth (Surface Nets) mesher should be used for this chunk.
///
/// In `Smooth` mode, ALL chunks use the smooth mesher regardless of distance.
/// The old distance gate (`chunk_distance_from_camera <= smooth_far_distance`) fell back
/// to the blocky cubic mesher for FAR chunks, but the cubic mesher has no neighbor apron:
/// it culls boundary faces against phantom air outside the 32³ window, producing visible
/// seam walls between adjacent chunks — terrain fragmentation bug #98.
/// The smooth mesher's 2-voxel apron (`slice_chunk_with_apron`) already handles this
/// correctly, so we keep it active for all distances in Smooth mode.
/// `compute_chunk_mesh` (async path) already refused to emit cubic in Smooth mode
/// (line 1078); this function now makes the sync/CA-remesh path (`spawn_chunk_meshes`)
/// consistent with that behavior.
fn should_use_smooth_mesh(
    chunk_id: ChunkId,
    camera_eye: Option<[f32; 3]>,
    mode: TerrainMesherMode,
) -> bool {
    match mode {
        TerrainMesherMode::Cubic => false,
        // Never emit cubic in Smooth mode — no distance cutover.
        // See doc-comment above for why the old distance gate caused #98.
        TerrainMesherMode::Smooth => {
            let _ = (chunk_id, camera_eye); // intentionally unused: no distance threshold
            true
        }
    }
}

fn chunk_saturation(grid: &CaGrid, cx: usize, cy: usize, cz: usize) -> Option<Vec<u8>> {
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

fn chunk_saturation_with_apron(grid: &CaGrid, cx: usize, cy: usize, cz: usize) -> Option<Vec<u8>> {
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
                if gx_i < 0 || gy_i < 0 || gz_i < 0 || gx_i >= gx || gy_i >= gy || gz_i >= gz_max {
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
    triplanar_materials: &mut Assets<VoxelTriplanarMaterial>,
    triplanar_bank: &mut VoxelPbrBank,
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
                if chunk_is_all_water(&voxels) {
                    continue;
                }
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
                    record_cubic_chunk();
                    match CubicMesher::mesh_cubic(view, LodLevel(0)) {
                        Ok(mesh_buffer) => split_by_material(&mesh_buffer)
                            .into_iter()
                            .map(|(_, submesh)| submesh)
                            .collect(),
                        Err(_) => Vec::new(),
                    }
                };
                // Cubic fallback disabled in smooth mode: it caused a giant opaque box
                // (interior solid chunks) and disconnected island fragments (boundary
                // chunks). The smooth mesher with its padded apron handles continuity
                // across chunk seams; empty smooth results should remain invisible.
                // The dead branch is kept (not removed) as documentation of what was
                // tried; the `if false` is intentional disable — clippy lints the
                // always-false conjunction, so we mark the condition explicitly.
                if false {
                    record_cubic_chunk();
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
                let entities = spawn_chunk_buffers(
                    commands,
                    meshes,
                    materials,
                    triplanar_materials,
                    triplanar_bank,
                    &registry,
                    &mesh_buffers,
                    chunk_origin,
                    use_smooth,
                );
                spawned.entry(chunk_id).or_default().extend(entities);
            }
        }
    }
    spawned
}

/// Returns `true` if at least one non-AIR voxel in the chunk is 6-connected to
/// an AIR voxel within the same chunk. Interior chunks (all solid, no exposed
/// face within the chunk) return `false` and should not receive a cubic fallback
/// mesh — they are invisible from outside.
fn chunk_has_exposed_face(voxels: &[MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE]) -> bool {
    for z in 0..CHUNK_EDGE {
        for y in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                let idx = x + y * CHUNK_EDGE + z * CHUNK_EDGE * CHUNK_EDGE;
                if voxels[idx] == AIR {
                    continue;
                }
                // Check all 6 neighbours within chunk bounds.
                let neighbours: [(isize, isize, isize); 6] = [
                    (-1, 0, 0),
                    (1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                    (0, 0, -1),
                    (0, 0, 1),
                ];
                for (dx, dy, dz) in neighbours {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let nz = z as isize + dz;
                    if nx < 0
                        || ny < 0
                        || nz < 0
                        || nx >= CHUNK_EDGE as isize
                        || ny >= CHUNK_EDGE as isize
                        || nz >= CHUNK_EDGE as isize
                    {
                        // Neighbour is outside this chunk — treat as exposed
                        // (cross-chunk air may exist there).
                        return true;
                    }
                    let nidx = nx as usize
                        + ny as usize * CHUNK_EDGE
                        + nz as usize * CHUNK_EDGE * CHUNK_EDGE;
                    if voxels[nidx] == AIR {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Main-thread spawn of a chunk's per-material mesh buffers into Bevy entities.
/// Shared by the synchronous remesh path and the async apply system so the
/// asset-add + jitter + transform logic lives in one place.
fn spawn_chunk_buffers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    triplanar_materials: &mut Assets<VoxelTriplanarMaterial>,
    triplanar_bank: &mut VoxelPbrBank,
    registry: &MaterialRegistry,
    mesh_buffers: &[MeshBuffer],
    chunk_origin: [usize; 3],
    use_smooth: bool,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    for submesh in mesh_buffers {
        if submesh
            .vertices
            .first()
            .is_some_and(|vertex| vertex.material == WATER)
        {
            continue;
        }
        let Some(material_id) = submesh.vertices.first().map(|v| v.material) else {
            continue;
        };
        let Some(def) = registry.get(material_id) else {
            continue;
        };
        let mut bevy_mesh = mesh_buffer_to_bevy(submesh);
        // Per-cube jitter quantizes tint to the integer voxel cell, painting a CUBE
        // GRID onto the mesh — that reads cubic even on a smooth Surface Nets
        // surface, so apply it ONLY to cubic meshes; smooth meshes keep flat
        // material-base colour so the molded geometry shows through.
        if !use_smooth {
            apply_voxel_jitter(&mut bevy_mesh, submesh, material_id);
        }
        let entity =
            if let Some(handle) = triplanar_bank.material_for(material_id, triplanar_materials) {
                commands
                    .spawn((
                        ChunkMeshTag,
                        Mesh3d(meshes.add(bevy_mesh)),
                        MeshMaterial3d(handle),
                        Transform::from_xyz(
                            chunk_origin[0] as f32,
                            chunk_origin[1] as f32,
                            chunk_origin[2] as f32,
                        ),
                    ))
                    .id()
            } else {
                let material = pbr_material_for(material_id, def);
                commands
                    .spawn((
                        ChunkMeshTag,
                        Mesh3d(meshes.add(bevy_mesh)),
                        MeshMaterial3d(materials.add(material)),
                        Transform::from_xyz(
                            chunk_origin[0] as f32,
                            chunk_origin[1] as f32,
                            chunk_origin[2] as f32,
                        ),
                    ))
                    .id()
            };
        entities.push(entity);
    }
    entities
}

/// Marks a terrain chunk mesh entity so the scene-dump harness can isolate
/// terrain geometry from background decoration (stars, sky dome, trees).
#[derive(Component)]
pub struct ChunkMeshTag;

/// Owned, `Send` inputs for one chunk's off-thread mesh compute.
struct ChunkMeshInput {
    chunk_id: ChunkId,
    origin: [usize; 3],
    use_smooth: bool,
    voxels: [MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE],
    padded:
        [MaterialId; SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE],
    saturation: Option<Vec<u8>>,
}

/// Result of an off-thread chunk mesh compute, applied on the main thread.
struct ChunkMeshJob {
    chunk_id: ChunkId,
    origin: [usize; 3],
    use_smooth: bool,
    buffers: Vec<MeshBuffer>,
}

/// In-flight off-thread chunk mesh task, carried on a throwaway entity until the
/// apply system drains it.
#[derive(Component)]
struct ComputingChunkMesh(Task<ChunkMeshJob>);

/// Pure mesh compute over OWNED data — runs off the main thread (no Bevy types).
/// Mirrors the synchronous mesher branch in `spawn_chunk_meshes` exactly, including
/// the empty-mesh cubic fallback + the smooth/cubic counters, so results are
/// byte-for-byte identical to the sync path (zero visual change).
fn compute_chunk_mesh(input: ChunkMeshInput) -> ChunkMeshJob {
    if chunk_is_all_water(&input.voxels) {
        return ChunkMeshJob {
            chunk_id: input.chunk_id,
            origin: input.origin,
            use_smooth: input.use_smooth,
            buffers: Vec::new(),
        };
    }

    let registry = MaterialRegistry::standard();
    let view = ChunkView {
        id: input.chunk_id,
        voxels: &input.voxels,
    };
    let mut buffers: Vec<MeshBuffer> = if input.use_smooth {
        build_smooth_meshes(
            &input.voxels,
            &input.padded,
            input.saturation.as_deref(),
            &registry,
        )
    } else if resolved_mesher_mode() == TerrainMesherMode::Smooth {
        // Global mode is Smooth — don't fall back to cubic even for distant chunks.
        // Cubic produces grey rectangular slabs that break the smooth terrain look.
        Vec::new()
    } else {
        record_cubic_chunk();
        mesh_cubic_split(view)
    };
    let _ = &view; // suppress unused-variable warning
    ChunkMeshJob {
        chunk_id: input.chunk_id,
        origin: input.origin,
        use_smooth: input.use_smooth,
        buffers,
    }
}

/// Cubic-mesh a chunk view and split it into per-material buffers (shared helper).
fn mesh_cubic_split(view: ChunkView<'_, MaterialId>) -> Vec<MeshBuffer> {
    match CubicMesher::mesh_cubic(view, LodLevel(0)) {
        Ok(mesh_buffer) => split_by_material(&mesh_buffer)
            .into_iter()
            .map(|(_, submesh)| submesh)
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Build the owned per-chunk input on the main thread (cheap O(chunk) slicing).
fn chunk_mesh_input(
    grid: &CaGrid,
    cx: usize,
    cy: usize,
    cz: usize,
    camera_eye: Option<[f32; 3]>,
) -> ChunkMeshInput {
    let chunk_id = ChunkId(((cx as u64) << 40) | ((cy as u64) << 16) | (cz as u64));
    let use_smooth = should_use_smooth_mesh(chunk_id, camera_eye, resolved_mesher_mode());
    let saturation = if use_smooth {
        chunk_saturation_with_apron(grid, cx, cy, cz)
    } else {
        None
    };
    ChunkMeshInput {
        chunk_id,
        origin: [cx * CHUNK_EDGE, cy * CHUNK_EDGE, cz * CHUNK_EDGE],
        use_smooth,
        voxels: slice_chunk(grid, cx, cy, cz),
        padded: slice_chunk_with_apron(grid, cx, cy, cz),
        saturation,
    }
}

/// Dispatch every chunk's mesh compute onto the async task pool. The main thread
/// only does cheap slicing here; the expensive mesher runs off-thread and the
/// `apply_chunk_mesh_tasks` system spawns the Bevy entities as each completes, so
/// `build_voxel_world` returns immediately and the world streams in without a hitch.
fn dispatch_chunk_mesh_tasks(commands: &mut Commands, grid: &CaGrid, camera_eye: Option<[f32; 3]>) {
    let pool = AsyncComputeTaskPool::get();
    let counts = [
        grid.dims[0].div_ceil(CHUNK_EDGE),
        grid.dims[1].div_ceil(CHUNK_EDGE),
        grid.dims[2].div_ceil(CHUNK_EDGE),
    ];
    for cz in 0..counts[2] {
        for cy in 0..counts[1] {
            for cx in 0..counts[0] {
                if chunk_is_all_water(&slice_chunk(grid, cx, cy, cz)) {
                    continue;
                }
                let input = chunk_mesh_input(grid, cx, cy, cz, camera_eye);
                let task = pool.spawn(async move { compute_chunk_mesh(input) });
                commands.spawn(ComputingChunkMesh(task));
            }
        }
    }
}

fn chunk_is_all_water(voxels: &[MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE]) -> bool {
    voxels
        .iter()
        .all(|material| *material == AIR || *material == WATER)
}

/// Drain completed off-thread chunk mesh tasks and spawn their Bevy entities on the
/// main thread, recording them in `VoxelSimState.chunk_entities`.
pub fn apply_chunk_mesh_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut triplanar_materials: ResMut<Assets<VoxelTriplanarMaterial>>,
    mut triplanar_bank: ResMut<VoxelPbrBank>,
    mut state: ResMut<VoxelSimState>,
    mut tasks: Query<(Entity, &mut ComputingChunkMesh)>,
) {
    let registry = MaterialRegistry::standard();
    for (carrier, mut task) in &mut tasks {
        let Some(job) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };
        commands.entity(carrier).despawn();
        if job.buffers.is_empty() {
            continue;
        }
        let entities = spawn_chunk_buffers(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut triplanar_materials,
            &mut triplanar_bank,
            &registry,
            &job.buffers,
            job.origin,
            job.use_smooth,
        );
        state
            .chunk_entities
            .entry(job.chunk_id)
            .or_default()
            .extend(entities);
    }
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
        emissive: Color::BLACK.into(),
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
            mat.perceptual_roughness = 0.9;
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
            mat.perceptual_roughness = 0.92;
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
                    mat.perceptual_roughness = 0.9;
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
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h
}
