//! In-process simulation bridge for the standalone Bevy client.

use bevy::prelude::*;
use civ_agents::{spawn_civilian_at, Civilian};
use civ_engine::{spawn::spawn_airport_at, Simulation};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::spawn_tools::{SpawnBuildingRequest, SpawnCivilianRequest};
use crate::terrain::WORLD_SIZE;

/// Live simulation state shared by the minimap, HUD, and spawn tools.
#[derive(Resource)]
pub struct SimState(pub Simulation);

impl Default for SimState {
    fn default() -> Self {
        Self(Simulation::default())
    }
}

/// In-process simulation tick interval for the standalone client.
const SIM_TICK_SECONDS: f32 = 0.1;

#[derive(Resource)]
struct SimTickTimer(Timer);

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
                    advance_simulation,
                    apply_spawn_civilian_requests,
                    apply_spawn_building_requests,
                ),
            );
        #[cfg(feature = "egui")]
        app.add_systems(Update, sync_game_ui_snapshot);
    }
}

fn advance_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimTickTimer>,
    mut sim: ResMut<SimState>,
) {
    if timer.0.tick(time.delta()) {
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
