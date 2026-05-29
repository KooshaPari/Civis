//! In-process simulation bridge for the standalone Bevy client.

use bevy::prelude::*;
use civ_agents::{spawn_civilian_at, Civilian};
use civ_engine::{spawn::spawn_airport_at, Building, BuildingType, Simulation};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::spawn_tools::{SpawnBuildingRequest, SpawnCivilianRequest};
use crate::terrain::WORLD_SIZE;
use crate::{live_attach::is_server_attach_mode, AttachMode};

/// Live simulation state shared by the minimap, HUD, and spawn tools.
#[derive(Resource)]
pub struct SimState(pub Simulation);

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

fn sync_visible_gameplay(
    mut commands: Commands,
    sim: Res<SimState>,
    existing_civilians: Query<Entity, With<SimCivilianMarker>>,
    existing_buildings: Query<Entity, With<SimBuildingMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !sim.is_changed() {
        return;
    }

    for entity in &existing_civilians {
        commands.entity(entity).despawn();
    }
    for entity in &existing_buildings {
        commands.entity(entity).despawn();
    }

    let civilian_mesh = meshes.add(Mesh::from(bevy::math::primitives::Capsule3d::new(0.45, 1.1)));
    let building_mesh = meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(2.0, 2.5, 2.0)));

    for (_, (civilian, position)) in sim
        .0
        .world
        .query::<(&Civilian, &civ_agents::Position3d)>()
        .iter()
    {
        let world_pos = sim_position_to_world(position);
        let color = faction_color(civilian.faction);
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.into(),
            perceptual_roughness: 0.55,
            ..default()
        });

        commands.spawn((
            SimCivilianMarker,
            Mesh3d(civilian_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(world_pos + Vec3::Y * 0.8),
        ));
    }

    for (_, building) in sim.0.world.query::<&Building>().iter() {
        let world_pos = building_world_position(building);
        let color = building_color(building.building_type);
        let material = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.9,
            ..default()
        });

        commands.spawn((
            SimBuildingMarker,
            Mesh3d(building_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(world_pos + Vec3::Y * 1.25),
        ));
    }
}

fn sim_position_to_world(position: &civ_agents::Position3d) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    Vec3::new(
        position.coord.x as f32 / scale,
        0.0,
        position.coord.z as f32 / scale,
    )
}

fn building_world_position(building: &Building) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    Vec3::new(
        building.position.x as f32 / scale,
        0.0,
        building.position.y as f32 / scale,
    )
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
