//! Integration tests for configure_military_fog and apply_scenario_military.
use civis_engine::{scenario::ScenarioMilitary, Simulation};

/// configure_military_fog does not panic on any combination of valid inputs.
#[test]
fn test_configure_military_fog_no_panic() {
    let mut sim = Simulation::with_seed(42);
    // Some radius -- grid_size below 16 gets clamped to 16
    sim.configure_military_fog(Some(5), 8);
    let cfg = sim.military_phase_config();
    assert_eq!(cfg.war.fog_vision_radius, Some(5));
    assert_eq!(cfg.war.fog_grid_size, 16);

    // None radius -- no mutation, previous values preserved
    sim.configure_military_fog(None, 32);
    let cfg2 = sim.military_phase_config();
    assert_eq!(cfg2.war.fog_vision_radius, Some(5), "radius unchanged when None");
    assert_eq!(cfg2.war.fog_grid_size, 16, "grid_size unchanged when None");
}

/// apply_scenario_military with default (all-None) params leaves military state sane.
#[test]
fn test_apply_scenario_military_default() {
    let mut sim = Simulation::with_seed(43);
    let before_cadence = sim.military_phase_config().movement.cadence_ticks;
    let before_pulses = sim.military_phase_config().movement_pulses_per_cadence;
    let before_war = sim.military_phase_config().war.cadence_ticks;
    let before_range = sim.military_phase_config().war.engage_range_grid;

    // Default ScenarioMilitary has all fields None -- nothing should change
    sim.apply_scenario_military(&ScenarioMilitary::default());

    let cfg = sim.military_phase_config();
    assert_eq!(cfg.movement.cadence_ticks, before_cadence);
    assert_eq!(cfg.movement_pulses_per_cadence, before_pulses);
    assert_eq!(cfg.war.cadence_ticks, before_war);
    assert_eq!(cfg.war.engage_range_grid, before_range);
    // engage_range is always >= 1 (clamped on write)
    assert!(cfg.war.engage_range_grid >= 1, "engage_range must be >= 1");
}