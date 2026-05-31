//! Bevy voxel simulation renderer for Civis P-VM-3.
//!
//! Feature-gated behind `voxel` so the heightmap sandbox remains the default
//! when the feature is off.

use std::collections::BTreeMap;

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

use crate::camera::CameraRig;
use crate::{bevy_render::mesh_buffer_to_bevy, should_render_chunk};

/// Fallback seed used only when no `WorldSetupParams` resource is available
/// (i.e. the `egui` menu feature is compiled out). With menus present, the
/// per-world randomized seed from `WorldSetupParams` is always used instead.
const SEED: u64 = 0xC1F1_5EED_D3AD_BEEF;
const CA_TICK_HZ: f32 = 12.0;
const CHUNK_EDGE: usize = 16;
const RENDER_MAX_DIST: f32 = 160.0;
// MVP playable surface: 0.5 mi² (~256³ @ ~2.8m/voxel), single resident region,
// no streaming yet. Y=128 gives hill + cave headroom. True uncapped HW-bounded
// streaming (LOD rings + horizon-fade + sim-LOD) is a later phase.
// Fallback to [192, 96, 192] if 256³ gen/mesh lags on first build.
const WORLD_DIMS: [usize; 3] = [256, 128, 256];

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
            for entity in state.chunk_entities.drain(..) {
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
    for entity in state.chunk_entities.drain(..) {
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
    state.grid = CaGrid {
        dims: generated.dims,
        cells: generated.cells,
    };
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
        camera_eye,
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
                    spawned.push(entity);
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
