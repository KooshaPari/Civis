use civ_engine::{MilitaryUnit, MilitaryUnitSample, Simulation};
use civ_tactics::{tick_war_bridge, WarBridgeConfig};

#[test]
fn diag_combat() {
    let mut sim = Simulation::with_seed(1);
    // run to tick 16
    for _ in 0..16 {
        sim.tick();
    }

    // Build samples exactly like phase_military does (grid_x/grid_y from position).
    let samples: Vec<MilitaryUnitSample> = sim
        .world
        .query::<&MilitaryUnit>()
        .iter()
        .enumerate()
        .map(|(idx, (_, u))| MilitaryUnitSample {
            unit_id: (idx as u64) + 1,
            faction_id: u.faction_id,
            grid_x: u.position.x,
            grid_y: u.position.y,
        })
        .collect();
    eprintln!("samples: {:?}", samples);

    // Test with default config (engage_range 8, cadence 16) at tick 16.
    let cfg = WarBridgeConfig::default();
    let eng_default = tick_war_bridge(16, &cfg, &samples, sim.voxel(), None);
    eprintln!("default cfg engagements @16: {}", eng_default.len());

    // Test with LOS disabled influence: huge range, cadence 1, no fog.
    let cfg2 = WarBridgeConfig { cadence_ticks: 1, engage_range_grid: 64, ..Default::default() };
    let eng2 = tick_war_bridge(1, &cfg2, &samples, sim.voxel(), None);
    eprintln!("wide cfg engagements @1: {}", eng2.len());

    // Test against a guaranteed-empty voxel world to isolate LOS-vs-terrain.
    let empty = civ_voxel::VoxelWorld::<civ_voxel::MaterialId>::new(civ_voxel::FIXED_SCALE);
    let eng3 = tick_war_bridge(1, &cfg2, &samples, &empty, None);
    eprintln!("wide cfg, empty voxel @1: {}", eng3.len());

    panic!("diag done");
}
