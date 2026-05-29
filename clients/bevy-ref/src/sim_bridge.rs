//! In-process simulation bridge for the standalone Bevy client.

use std::collections::HashMap;

use bevy::prelude::*;
use civ_agents::{spawn_civilian_at, Civilian};
use civ_engine::{spawn::spawn_airport_at, Building, BuildingType, Simulation};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::spawn_tools::{SpawnBuildingRequest, SpawnCivilianRequest};
use crate::terrain::{terrain_surface_y, WORLD_SIZE};
use crate::{live_attach::is_server_attach_mode, AttachMode};

/// Half-height of the civilian capsule (used to seat its base on terrain).
const CIVILIAN_HALF_HEIGHT: f32 = 1.6 + 1.4; // capsule half-length + radius
/// Half-height of the building cuboid (seats its base on terrain).
const BUILDING_HALF_HEIGHT: f32 = 7.0;

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
            )))
            .add_systems(
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
            sync_game_ui_snapshot.run_if(in_process_sim_active),
        );
        app.add_systems(Update, sync_visible_gameplay.run_if(in_process_sim_active));
    }
}

fn advance_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimTickTimer>,
    mut sim: ResMut<SimState>,
    #[cfg(feature = "egui")] mode: Res<crate::menus::GameUiMode>,
    #[cfg(feature = "egui")] speed: Res<crate::game_ui::GameSpeed>,
) {
    #[cfg(feature = "egui")]
    if *mode == crate::menus::GameUiMode::Paused || speed.multiplier == 0 {
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        sim.0.tick();
    }
}

fn world_to_norm(position: Vec3) -> (f32, f32) {
    let wx = position.x + WORLD_SIZE * 0.5;
    let wz = position.z + WORLD_SIZE * 0.5;
    (
        (wx / WORLD_SIZE).clamp(0.0, 1.0),
        (wz / WORLD_SIZE).clamp(0.0, 1.0),
    )
}

fn apply_spawn_civilian_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnCivilianRequest>,
) {
    for request in requests.read() {
        let (nx, ny) = world_to_norm(request.position);
        let id = next_civilian_id(&sim.0);
        let seed = id.wrapping_add(nx.to_bits() as u64 ^ ny.to_bits() as u64);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        spawn_civilian_at(&mut sim.0.world, id, 0, nx, ny, &mut rng);
    }
}

fn apply_spawn_building_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnBuildingRequest>,
) {
    for request in requests.read() {
        let (nx, ny) = world_to_norm(request.position);
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
fn sync_visible_gameplay(
    mut commands: Commands,
    sim: Res<SimState>,
    assets: Option<Res<GameplayAssets>>,
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

    for (_, (civilian, position)) in sim
        .0
        .world
        .query::<(&Civilian, &civ_agents::Position3d)>()
        .iter()
    {
        civ_count += 1;
        seen_civilians.push(civilian.id);
        let world_pos = sim_position_to_world(position) + Vec3::Y * CIVILIAN_HALF_HEIGHT;
        if first_world_pos.is_none() {
            first_world_pos = Some(world_pos);
        }

        if let Some(&entity) = rendered.civilians.get(&civilian.id) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = world_pos;
            }
        } else {
            let color = faction_color(civilian.faction);
            let material = materials.add(StandardMaterial {
                base_color: color,
                emissive: color.into(),
                perceptual_roughness: 0.55,
                ..default()
            });
            let entity = commands
                .spawn((
                    SimCivilianMarker,
                    Mesh3d(assets.civilian_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(world_pos),
                ))
                .id();
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
        let world_pos = building_world_position(building) + Vec3::Y * BUILDING_HALF_HEIGHT;

        if let Some(&entity) = rendered.buildings.get(&id) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = world_pos;
            }
        } else {
            let color = building_color(building.building_type);
            let material = materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.9,
                ..default()
            });
            let entity = commands
                .spawn((
                    SimBuildingMarker,
                    Mesh3d(assets.building_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(world_pos),
                ))
                .id();
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
fn sim_position_to_world(position: &civ_agents::Position3d) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    let nx = position.coord.x as f32 / scale;
    let nz = position.coord.z as f32 / scale;
    let mesh_x = nx * WORLD_SIZE;
    let mesh_z = nz * WORLD_SIZE;
    let y = terrain_surface_y(mesh_x, mesh_z);
    Vec3::new(mesh_x - WORLD_SIZE * 0.5, y, mesh_z - WORLD_SIZE * 0.5)
}

/// Map an integer building grid tile to centred, terrain-seated world XZ.
fn building_world_position(building: &Building) -> Vec3 {
    let mesh_x = WORLD_SIZE * 0.5 + building.position.x as f32 * 14.0;
    let mesh_z = WORLD_SIZE * 0.5 + building.position.y as f32 * 14.0;
    let y = terrain_surface_y(mesh_x, mesh_z);
    Vec3::new(mesh_x - WORLD_SIZE * 0.5, y, mesh_z - WORLD_SIZE * 0.5)
}

fn faction_color(faction: u32) -> Color {
    let hue = (faction as f32 * 85.0) % 360.0;
    Color::hsla(hue, 0.75, 0.55, 1.0)
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
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
) {
    if !sim.is_changed() {
        return;
    }

    let population = sim.0.world.query::<&Civilian>().iter().count() as u64;
    let factions = sim
        .0
        .world
        .query::<&Civilian>()
        .iter()
        .map(|(_, civilian)| civilian.faction)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let tick = sim.0.state.tick;
    snapshot.set_sim_state(tick, population, factions, tick.to_string(), 1);
}
