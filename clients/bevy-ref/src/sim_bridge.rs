//! In-process simulation bridge for the standalone Bevy client.

use std::collections::HashMap;

use bevy::prelude::*;
use civ_agents::{spawn_civilian_at, ActorVisual, ActorVisualKind, Civilian};
use civ_engine::{spawn::spawn_airport_at, Building, BuildingType, Simulation};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::spawn_tools::{SpawnBuildingRequest, SpawnCivilianRequest};
#[cfg(feature = "voxel")]
use crate::voxel_sim::{voxel_surface_y, VoxelSimState};
#[cfg(not(feature = "voxel"))]
use crate::terrain::{terrain_surface_y, WATER_LEVEL, WORLD_SIZE};
use crate::{live_attach::is_server_attach_mode, AttachMode};

/// Half-height of the civilian capsule (used to seat its base on terrain).
const CIVILIAN_HALF_HEIGHT: f32 = 1.6 + 1.4; // capsule half-length + radius
/// Half-height of the building cuboid (seats its base on terrain).
const BUILDING_HALF_HEIGHT: f32 = 7.0;

/// Uniform scale for the CC0 GLTF civilian (KayKit Knight) so it reads near the
/// gameplay capsule's height.
#[cfg(feature = "models")]
#[cfg(feature = "voxel")]
const CIVILIAN_MODEL_SCALE: f32 = 1.7;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const CIVILIAN_MODEL_SCALE: f32 = 3.0;
/// Scale for herd / fauna rigs (skeleton minion).
#[cfg(feature = "models")]
const HERD_MODEL_SCALE: f32 = 2.4;
/// Uniform scale for the CC0 GLTF building (KayKit hexagon home).
#[cfg(feature = "models")]
#[cfg(feature = "voxel")]
const BUILDING_MODEL_SCALE: f32 = 0.65;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const BUILDING_MODEL_SCALE: f32 = 6.0;

/// Faction tag for a model-backed civilian so a later material-tint pass can
/// colour the GLTF scene per faction. Primitives bake the colour in directly.
#[cfg(feature = "models")]
#[derive(Component)]
struct FactionTint(#[allow(dead_code)] u32);

/// Resolve the loaded CC0 actor scene for `kind`, else `None` (capsule fallback).
#[cfg(feature = "models")]
fn actor_model_root(
    models: Option<&crate::gltf_models::GameModels>,
    kind: ActorVisualKind,
    faction: u32,
) -> Option<SceneRoot> {
    use crate::gltf_models::{actor_scene, ModelOrPrimitive};
    match models.map(|m| actor_scene(m, kind, faction)) {
        Some(ModelOrPrimitive::Model(root)) => Some(root),
        _ => None,
    }
}

#[cfg(feature = "models")]
fn model_scale_for(kind: ActorVisualKind) -> f32 {
    match kind {
        ActorVisualKind::Humanoid => CIVILIAN_MODEL_SCALE,
        ActorVisualKind::Herd => HERD_MODEL_SCALE,
    }
}

/// Resolve the loaded CC0 building scene to a spawnable `SceneRoot`, else `None`
/// to fall back to the procedural cuboid.
#[cfg(feature = "models")]
fn building_model_root(models: Option<&crate::gltf_models::GameModels>) -> Option<SceneRoot> {
    use crate::gltf_models::{building_scene, ModelOrPrimitive};
    match models.map(building_scene) {
        Some(ModelOrPrimitive::Model(root)) => Some(root),
        _ => None,
    }
}

/// Live simulation state shared by the minimap, HUD, and spawn tools.
#[derive(Resource)]
pub struct SimState(pub Simulation);

/// Spawn-once registries mapping a stable sim id to its rendered entity.
#[derive(Resource, Default)]
struct RenderedEntities {
    civilians: HashMap<u64, Entity>,
    buildings: HashMap<u64, Entity>,
}

/// Shared mesh/material handles so we spawn the assets once, not per entity.
#[derive(Resource)]
struct GameplayAssets {
    civilian_mesh: Handle<Mesh>,
    building_mesh: Handle<Mesh>,
}

#[derive(Component)]
struct SimCivilianMarker;

#[derive(Component)]
struct SimBuildingMarker;

impl Default for SimState {
    fn default() -> Self {
        Self(Simulation::default())
    }
}

/// In-process simulation tick interval for the standalone client.
const SIM_TICK_SECONDS: f32 = 0.1;

#[derive(Resource)]
struct SimTickTimer(Timer);

fn in_process_sim_active(mode: Res<AttachMode>) -> bool {
    !is_server_attach_mode(*mode)
}

/// Returns true only when the game is in the Playing state.
/// Used as a run condition to gate all sim systems.
#[cfg(feature = "egui")]
fn is_playing(mode: Res<crate::menus::GameUiMode>) -> bool {
    *mode == crate::menus::GameUiMode::Playing
}

/// Wires spawn-tool messages into the ECS simulation and optional HUD sync.
#[derive(Default)]
pub struct SimBridgePlugin;

impl Plugin for SimBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimState>()
            .init_resource::<RenderedEntities>()
            .add_systems(Startup, init_gameplay_assets)
            .insert_resource(SimTickTimer(Timer::from_seconds(
                SIM_TICK_SECONDS,
                TimerMode::Repeating,
            )));

        // All sim systems are gated: in_process_sim_active AND (egui-gated) is_playing.
        // Without egui the game has no menu system so we allow free running
        // (same behaviour as before this patch).
        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            (
                maybe_reinit_sim_on_new_world,
                advance_simulation
                    .run_if(in_process_sim_active)
                    .run_if(is_playing),
                apply_spawn_civilian_requests
                    .run_if(in_process_sim_active)
                    .run_if(is_playing),
                apply_spawn_building_requests
                    .run_if(in_process_sim_active)
                    .run_if(is_playing),
            ),
        );

        #[cfg(not(feature = "egui"))]
        app.add_systems(
            Update,
            (
                advance_simulation.run_if(in_process_sim_active),
                apply_spawn_civilian_requests.run_if(in_process_sim_active),
                apply_spawn_building_requests.run_if(in_process_sim_active),
            ),
        );

        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            sync_game_ui_snapshot
                .run_if(in_process_sim_active)
                .run_if(is_playing),
        );

        // sync_visible_gameplay must also be gated: only render entities in Playing/Paused.
        // In Paused we keep entities visible (frozen) but don't tick; in menus we despawn all.
        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            sync_visible_gameplay
                .run_if(in_process_sim_active)
                .run_if(is_playing_or_paused),
        );

        #[cfg(not(feature = "egui"))]
        app.add_systems(Update, sync_visible_gameplay.run_if(in_process_sim_active));
    }
}

/// Run condition: allow entity sync while Playing or Paused (freeze-in-place).
#[cfg(feature = "egui")]
fn is_playing_or_paused(mode: Res<crate::menus::GameUiMode>) -> bool {
    matches!(
        *mode,
        crate::menus::GameUiMode::Playing | crate::menus::GameUiMode::Paused
    )
}

/// Watches for the transition into `Playing` and, when detected, reinitialises
/// the simulation from `WorldSetupParams.seed` and clears all previously-rendered
/// entities so a New World always starts clean.
///
/// Uses a `Local<Option<GameUiMode>>` to track the previous frame's mode so the
/// reinit fires exactly once per MainMenu→Playing (or Loading→Playing) edge.
#[cfg(feature = "egui")]
fn maybe_reinit_sim_on_new_world(
    mut commands: Commands,
    mode: Res<crate::menus::GameUiMode>,
    params: Res<crate::menus::WorldSetupParams>,
    mut sim: ResMut<SimState>,
    mut rendered: ResMut<RenderedEntities>,
    mut prev_mode: Local<Option<crate::menus::GameUiMode>>,
) {
    let current = *mode;
    let previous = *prev_mode;
    *prev_mode = Some(current);

    // Only act on the frame we transition INTO Playing from a non-Playing state.
    let just_entered_playing = current == crate::menus::GameUiMode::Playing
        && previous != Some(crate::menus::GameUiMode::Playing);

    if !just_entered_playing {
        return;
    }

    // `WorldSetupParams.seed` is already a parsed u64 (the menus agent keeps it
    // in sync via `commit_text()`).  Use it directly; no string parsing needed.
    let seed: u64 = params.seed;

    info!(
        "[sim_bridge] New World: reinitialising simulation with seed={}",
        seed
    );

    // Despawn every previously-rendered civilian entity.
    for (_, entity) in rendered.civilians.drain() {
        commands.entity(entity).despawn();
    }
    // Despawn every previously-rendered building entity.
    for (_, entity) in rendered.buildings.drain() {
        commands.entity(entity).despawn();
    }

    // Replace the simulation with a fresh one seeded from WorldSetupParams.
    sim.0 = Simulation::with_seed(seed);
}

fn advance_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimTickTimer>,
    mut sim: ResMut<SimState>,
    #[cfg(feature = "egui")] speed: Res<crate::game_ui::GameSpeed>,
) {
    // Paused guard: the system is already excluded from Paused by run conditions,
    // but keep the speed-multiplier zero-check for game-speed=0 edge case.
    #[cfg(feature = "egui")]
    if speed.multiplier == 0 {
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        sim.0.tick();
    }
}

#[cfg(not(feature = "voxel"))]
fn world_to_norm(position: Vec3) -> (f32, f32) {
    let wx = position.x + WORLD_SIZE * 0.5;
    let wz = position.z + WORLD_SIZE * 0.5;
    (
        (wx / WORLD_SIZE).clamp(0.0, 1.0),
        (wz / WORLD_SIZE).clamp(0.0, 1.0),
    )
}

#[cfg(feature = "voxel")]
fn world_to_norm(position: Vec3, voxel: Option<&VoxelSimState>) -> (f32, f32) {
    let (Some(voxel),) = (voxel,) else {
        return (0.5, 0.5);
    };
    let dims = voxel.grid.dims;
    if dims[0] == 0 || dims[2] == 0 {
        return (0.5, 0.5);
    }
    (
        (position.x / dims[0] as f32).clamp(0.0, 1.0),
        (position.z / dims[2] as f32).clamp(0.0, 1.0),
    )
}

fn apply_spawn_civilian_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnCivilianRequest>,
    #[cfg(feature = "voxel")]
    voxel_state: Option<Res<VoxelSimState>>,
) {
    for request in requests.read() {
        let (nx, ny) = {
            #[cfg(not(feature = "voxel"))]
            {
                world_to_norm(request.position)
            }
            #[cfg(feature = "voxel")]
            {
                world_to_norm(request.position, voxel_state.as_deref())
            }
        };
        let id = next_civilian_id(&sim.0);
        let seed = id.wrapping_add(nx.to_bits() as u64 ^ ny.to_bits() as u64);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        spawn_civilian_at(
            &mut sim.0.world,
            id,
            0,
            nx,
            ny,
            request.model_kind,
            &mut rng,
        );
    }
}

fn apply_spawn_building_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnBuildingRequest>,
    #[cfg(feature = "voxel")]
    voxel_state: Option<Res<VoxelSimState>>,
) {
    for request in requests.read() {
        let (nx, ny) = {
            #[cfg(not(feature = "voxel"))]
            {
                world_to_norm(request.position)
            }
            #[cfg(feature = "voxel")]
            {
                world_to_norm(request.position, voxel_state.as_deref())
            }
        };
        spawn_airport_at(&mut sim.0.world, nx, ny);
    }
}

/// Build the shared civilian/building meshes once at startup.
fn init_gameplay_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let civilian_mesh = meshes.add(Mesh::from(bevy::math::primitives::Capsule3d::new(1.4, 3.2)));
    let building_mesh = meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(7.0, 14.0, 7.0)));
    commands.insert_resource(GameplayAssets {
        civilian_mesh,
        building_mesh,
    });
}

/// Incrementally reconcile rendered civilians/buildings with the simulation.
///
/// Spawns one entity per stable sim id (tracked in [`RenderedEntities`]),
/// updates the transform of already-spawned entities every change, and
/// despawns entities whose sim id has disappeared. Civilians/buildings are
/// seated on the procedural terrain surface and centred on the origin to match
/// the centred terrain mesh.
///
/// Only runs in Playing (and Paused to keep entities visible but frozen).
fn sync_visible_gameplay(
    mut commands: Commands,
    sim: Res<SimState>,
    assets: Option<Res<GameplayAssets>>,
    #[cfg(feature = "voxel")]
    voxel_state: Option<Res<VoxelSimState>>,
    #[cfg(feature = "models")] models: Option<Res<crate::gltf_models::GameModels>>,
    mut rendered: ResMut<RenderedEntities>,
    mut transforms: Query<&mut Transform>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !sim.is_changed() {
        return;
    }
    let Some(assets) = assets else {
        return;
    };

    let mut civ_count = 0_usize;
    let mut first_world_pos: Option<Vec3> = None;
    let mut seen_civilians: Vec<u64> = Vec::new();

    for (_, (civilian, position, visual)) in sim
        .0
        .world
        .query::<(&Civilian, &civ_agents::Position3d, Option<&ActorVisual>)>()
        .iter()
    {
        let visual_kind = visual.map(|v| v.0).unwrap_or(ActorVisualKind::Humanoid);
        civ_count += 1;
        seen_civilians.push(civilian.id);
        let world_pos = {
            #[cfg(not(feature = "voxel"))]
            {
                sim_position_to_world(position, None) + Vec3::Y * CIVILIAN_HALF_HEIGHT
            }
            #[cfg(feature = "voxel")]
            {
                sim_position_to_world(position, voxel_state.as_deref()) + Vec3::Y * CIVILIAN_HALF_HEIGHT
            }
        };
        if first_world_pos.is_none() {
            first_world_pos = Some(world_pos);
        }

        if let Some(&entity) = rendered.civilians.get(&civilian.id) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = world_pos;
            }
        } else {
            #[cfg(feature = "models")]
            let scene_root = actor_model_root(models.as_deref(), visual_kind, civilian.faction);
            #[cfg(not(feature = "models"))]
            let scene_root: Option<SceneRoot> = None;

            let entity = if let Some(scene_root) = scene_root {
                #[cfg(feature = "models")]
                {
                    commands
                        .spawn((
                            SimCivilianMarker,
                            FactionTint(civilian.faction),
                            scene_root,
                            Transform::from_translation(world_pos - Vec3::Y * CIVILIAN_HALF_HEIGHT)
                                .with_scale(Vec3::splat(model_scale_for(visual_kind))),
                        ))
                        .id()
                }
                #[cfg(not(feature = "models"))]
                {
                    let _ = scene_root;
                    unreachable!()
                }
            } else {
                let color = faction_color(civilian.faction);
                let material = materials.add(StandardMaterial {
                    base_color: color,
                    // Solid lit object, not a glowing pill: kill the emissive bloom.
                    emissive: LinearRgba::BLACK,
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                });
                commands
                    .spawn((
                        SimCivilianMarker,
                        Mesh3d(assets.civilian_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(world_pos),
                    ))
                    .id()
            };
            rendered.civilians.insert(civilian.id, entity);
        }
    }

    // Despawn civilians that no longer exist in the simulation.
    rendered.civilians.retain(|id, &mut entity| {
        if seen_civilians.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });

    let mut seen_buildings: Vec<u64> = Vec::new();
    let mut building_idx = 0_u64;
    for (_, building) in sim.0.world.query::<&Building>().iter() {
        let id = building_idx;
        building_idx += 1;
        seen_buildings.push(id);
        let world_pos = {
            #[cfg(not(feature = "voxel"))]
            {
                building_world_position(building, None) + Vec3::Y * BUILDING_HALF_HEIGHT
            }
            #[cfg(feature = "voxel")]
            {
                building_world_position(building, voxel_state.as_deref()) + Vec3::Y * BUILDING_HALF_HEIGHT
            }
        };

        if let Some(&entity) = rendered.buildings.get(&id) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = world_pos;
            }
        } else {
            #[cfg(feature = "models")]
            let scene_root = building_model_root(models.as_deref());
            #[cfg(not(feature = "models"))]
            let scene_root: Option<SceneRoot> = None;

            let entity = if let Some(scene_root) = scene_root {
                #[cfg(feature = "models")]
                {
                    commands
                        .spawn((
                            SimBuildingMarker,
                            scene_root,
                            Transform::from_translation(world_pos - Vec3::Y * BUILDING_HALF_HEIGHT)
                                .with_scale(Vec3::splat(BUILDING_MODEL_SCALE)),
                        ))
                        .id()
                }
                #[cfg(not(feature = "models"))]
                {
                    let _ = scene_root;
                    unreachable!()
                }
            } else {
                let color = building_color(building.building_type);
                let material = materials.add(StandardMaterial {
                    base_color: color,
                    // No additive/neon glow: read as a solid lit structure.
                    emissive: LinearRgba::BLACK,
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                });
                commands
                    .spawn((
                        SimBuildingMarker,
                        Mesh3d(assets.building_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(world_pos),
                    ))
                    .id()
            };
            rendered.buildings.insert(id, entity);
        }
    }
    rendered.buildings.retain(|id, &mut entity| {
        if seen_buildings.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });

    info!(
        "[sim_bridge] civilians={} buildings={} civilian[0] world={:?}",
        civ_count,
        seen_buildings.len(),
        first_world_pos
    );
}

/// Map a normalised sim agent coordinate to centred, terrain-seated world XZ.
#[cfg(not(feature = "voxel"))]
fn sim_position_to_world(
    position: &civ_agents::Position3d,
    _voxel: Option<&VoxelSimState>,
) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    let nx = position.coord.x as f32 / scale;
    let nz = position.coord.z as f32 / scale;
    let mesh_x = nx * WORLD_SIZE;
    let mesh_z = nz * WORLD_SIZE;
    // Seat on the surface, but never below sea level (no underwater civilians).
    let y = terrain_surface_y(mesh_x, mesh_z).max(WATER_LEVEL);
    Vec3::new(mesh_x - WORLD_SIZE * 0.5, y, mesh_z - WORLD_SIZE * 0.5)
}

#[cfg(feature = "voxel")]
fn sim_position_to_world(position: &civ_agents::Position3d, voxel: Option<&VoxelSimState>) -> Vec3 {
    // Mapping choice A: use voxel grid dims as the single world extent.
    // Sim-normalized [0,1] maps directly to [0,dims] so 0.5 lands at center and
    // corners at 0 and 1 match voxel box corners.
    let (Some(voxel),) = (voxel,) else {
        return Vec3::ZERO;
    };
    let scale = civ_voxel::FIXED_SCALE as f32;
    let dims = voxel.grid.dims;
    if dims[0] == 0 || dims[2] == 0 {
        return Vec3::ZERO;
    }
    let nx = (position.coord.x as f32 / scale).clamp(0.0, 1.0);
    let nz = (position.coord.z as f32 / scale).clamp(0.0, 1.0);
    let mesh_x = nx * dims[0] as f32;
    let mesh_z = nz * dims[2] as f32;
    let y = voxel_surface_y(&voxel.grid, mesh_x, mesh_z);
    Vec3::new(mesh_x, y, mesh_z)
}

/// Map an integer building grid tile to centred, terrain-seated world XZ.
///
/// `Building::position` is a centred grid coordinate in `[-64, 63]` (0,0 is
/// map centre). We normalise it to `0..1` (matching `engine::grid_to_norm`),
/// scale onto the mesh extent, then sample the surface. If that surface is
/// below sea level the base is clamped to [`WATER_LEVEL`] so the building rests
/// at the shoreline instead of being submerged or floating on the water plane.
#[cfg(not(feature = "voxel"))]
fn building_world_position(building: &Building, _voxel: Option<&VoxelSimState>) -> Vec3 {
    let nx = ((building.position.x + 64) as f32 / 127.0).clamp(0.0, 1.0);
    let nz = ((building.position.y + 64) as f32 / 127.0).clamp(0.0, 1.0);
    let mesh_x = nx * WORLD_SIZE;
    let mesh_z = nz * WORLD_SIZE;
    let y = terrain_surface_y(mesh_x, mesh_z).max(WATER_LEVEL);
    Vec3::new(mesh_x - WORLD_SIZE * 0.5, y, mesh_z - WORLD_SIZE * 0.5)
}
#[cfg(feature = "voxel")]
fn building_world_position(building: &Building, voxel: Option<&VoxelSimState>) -> Vec3 {
    let (Some(voxel),) = (voxel,) else {
        return Vec3::ZERO;
    };
    let dims = voxel.grid.dims;
    if dims[0] == 0 || dims[2] == 0 {
        return Vec3::ZERO;
    }
    let x_span = (dims[0] as f32 - 1.0).max(1.0);
    let z_span = (dims[2] as f32 - 1.0).max(1.0);
    let nx = ((building.position.x as f32 + x_span * 0.5) / x_span).clamp(0.0, 1.0);
    let nz = ((building.position.y as f32 + z_span * 0.5) / z_span).clamp(0.0, 1.0);
    let mesh_x = nx * dims[0] as f32;
    let mesh_z = nz * dims[2] as f32;
    let y = voxel_surface_y(&voxel.grid, mesh_x, mesh_z);
    Vec3::new(mesh_x, y, mesh_z)
}

fn faction_color(faction: u32) -> Color {
    let hue = (faction as f32 * 85.0) % 360.0;
    // Normal saturated faction colour (not over-bright) for a lit PBR body.
    Color::hsla(hue, 0.6, 0.45, 1.0)
}

fn building_color(building_type: BuildingType) -> Color {
    match building_type {
        BuildingType::Farm => Color::srgb(0.55, 0.75, 0.35),
        BuildingType::Mine => Color::srgb(0.52, 0.48, 0.42),
        BuildingType::Barracks => Color::srgb(0.72, 0.34, 0.34),
        BuildingType::Temple => Color::srgb(0.72, 0.62, 0.88),
        BuildingType::Market => Color::srgb(0.88, 0.67, 0.25),
        BuildingType::House => Color::srgb(0.79, 0.59, 0.40),
        BuildingType::CityCenter => Color::srgb(0.38, 0.58, 0.86),
    }
}

fn next_civilian_id(sim: &Simulation) -> u64 {
    sim.world
        .query::<&Civilian>()
        .iter()
        .map(|(_, civilian)| civilian.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

#[cfg(feature = "egui")]
fn sync_game_ui_snapshot(
    sim: Res<SimState>,
    speed: Res<crate::game_ui::GameSpeed>,
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
    mut resources: ResMut<crate::game_ui::WorldResources>,
    mut roster: ResMut<crate::game_ui::FactionRoster>,
) {
    if !sim.is_changed() {
        return;
    }
    let sim = &sim.0;

    // Pull the authoritative per-tick snapshot for stocks + vital stats.
    let snap = sim.snapshot();
    let population = snap.population;

    // Emergent clusters (settlements) drive both the faction count chip and the
    // left-panel roster — these are NOT hardcoded factions.
    let roster_rows = build_faction_roster(sim);
    let faction_count = roster_rows.len() as u32;
    roster.factions = roster_rows;

    snapshot.set_sim_state(
        snap.tick,
        population,
        faction_count,
        snap.tick.to_string(),
        speed.multiplier.max(1),
    );

    // World resource strip: global economy stocks + the pooled settlement
    // commons as a stand-in treasury, plus births/deaths this tick.
    let food = snap.resources.food.to_f64();
    let materials = snap.resources.wood.to_f64() + snap.resources.metal.to_f64();
    let energy = snap.resources.energy.to_f64();
    let treasury: f64 = sim
        .cluster_stocks()
        .values()
        .map(|stock| stock.total() as f64)
        .sum();
    resources.update_stocks(
        food,
        materials,
        energy,
        treasury,
        snap.births_this_tick,
        snap.deaths_this_tick,
    );
}

/// Build the left-panel roster from emergent clusters.
///
/// Counts civilians per [`civ_agents::ClusterMember`] cluster id; civilians with
/// no membership component are pooled under the id-0 "Unaffiliated" bucket so
/// the panel always reflects the full live population. Rows are ordered by
/// descending size so the largest settlements lead.
#[cfg(feature = "egui")]
fn build_faction_roster(sim: &Simulation) -> Vec<crate::game_ui::FactionInfo> {
    use std::collections::BTreeMap;
    let mut sizes: BTreeMap<u64, u64> = BTreeMap::new();
    for (_, (_civilian, member)) in sim
        .world
        .query::<(&Civilian, Option<&civ_agents::ClusterMember>)>()
        .iter()
    {
        let id = member.map(|m| m.cluster.0).unwrap_or(0);
        *sizes.entry(id).or_insert(0) += 1;
    }
    let mut rows: Vec<_> = sizes
        .into_iter()
        .map(|(id, count)| crate::game_ui::FactionInfo::from_cluster(id, count))
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    rows
}
