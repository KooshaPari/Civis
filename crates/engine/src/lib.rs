//! CivLab Deterministic Simulation Engine
//!
//! Uses fixed-point arithmetic for deterministic simulation results.
//! Uses i64 with scaling for deterministic calculations.
//!
//! ## Modules
//!
//! - `engine` - Full ECS-based simulation with tick loop
//! - `step` - Simple step function for basic simulation
//! - `policy` - Policy/consumption calculations
//! - `metrics` - Tyranny/legitimacy metrics
//! - `io` - File I/O utilities

pub mod building_emergence;
pub mod command_queue;
pub mod conditions;
pub mod culture;
pub mod demographics;
pub mod disasters;
pub mod dormant_phases;
pub mod emergence;
pub mod emergence_metrics;
pub mod engine;
pub mod era;
pub mod faction_decisions;
pub mod gameplay;
pub mod godtools;
pub mod hash_chain;
pub mod integrity;
pub mod invariants;
pub mod io;
pub mod lod;
pub mod metrics;
pub mod perf;
pub mod policy;
pub mod replay;
pub mod replay_format;
pub mod save_bundle;
pub mod scenario;
pub mod spawn;
pub mod spectator;
pub mod technology;
pub mod writing;

pub mod tech;
pub mod tutorial;

/// Fixed-point scaling factor (1 raw unit = SCALE joules). Engine energy
/// quantities are stored in fixed-point `i64` for determinism and converted
/// to `f64`/SI at the economy boundary using this constant.
pub const SCALE: i64 = 1_000;

pub use civ_emergence_metrics::branching::BranchingRegime;
pub use civ_emergence_metrics::dashboard::EmergenceDashboard;
pub use civ_mod_host::{
    format_mod_error_event, format_mod_error_event_json, format_mod_loaded_event,
    format_mod_loaded_event_json, format_mod_unloaded_event_json, load_manifest, ModBrowserEntry,
    ModGuestStateSave, ModHost, ModLoadedRecord, ModType, ModUnloadedRecord,
};
pub use demographics::{
    carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics,
    DemographicsSnapshot,
};
pub use emergence::{CivAiDecision, EmergenceFeedEvent, EmergenceState};
pub use emergence_metrics::{EmergenceBranchingState, EmergenceSample};
pub use engine::{
    job_type_for_civilian_id, Building, BuildingType, Citizen, ClusterStocks, CombatDamagePulse,
    DiplomacyEvent, DiplomacyKind, EconomicFocus, EconomicFocusEvent, InstitutionEvent, JobType,
    MilitaryUnit, Position, PsycheDrivenBehavior, ResourceType, Resources, Sim, SimSeed,
    Simulation, StratBand, StratificationEvent, StratificationEventKind, StratificationReport,
    TradeRoute, UnitType, WorldState,
};
pub use hash_chain::hash_hex;
pub use replay_format::{decode_civreplay, encode_civreplay};
pub use save_bundle::{
    delete_slot, list_slots, load_from_slot, save_to_slot, CivSaveBundle, SaveSlotEntry,
};
pub use spawn::{
    grid_to_norm, spawn_airport_at, spawn_hangar_at, spawn_military_at, spawn_port_at,
    unit_type_label,
};

// FR-CIV-ARCH: Emergent building layouts re-export so callers can use
// `civ_engine::EmergentLayout` and `civ_engine::LayoutStrategy` without
// directly depending on the private `building_layouts` module.
pub use era::{CivEra, EraProgressionState, FactionEraSnapshot};

pub use tutorial::{TutorialMilestone, TutorialProgress};

// FR-CIV-GOV-001/002/003 (civ-007 institutions epic). Re-exported so callers
// (server, clients, tests) can `use civ_engine::InstitutionKind` etc. without
// pulling the `civ-institutions` crate directly.
pub use civ_institutions::InstitutionKind;
pub use civ_planet::{BiomeKind, Climate, GeologyMap, MoonConfig, PlanetConfig, RegionBiome};
pub use civ_tactics::{
    apply_damage, bfs_next_step, evolve_doctrine, formation_offsets, grid_to_world_coord,
    line_of_sight, score_doctrine_fitness, tick_operational_movement, tick_war_bridge,
    CombatEngagement, DamageEvent, Doctrine, DoctrineLibrary, FactionEngagementStats,
    FormationKind, GridMove, MilitaryPhaseConfig, MilitaryUnitSample, NoopOperationalLayer,
    OperationalLayer, OperationalMovementConfig, WarBridgeConfig,
};

// FR-CIV-GOV-030 / FR-CIV-UNREST-001 remain engine-internal method surfaces.
// The event/snapshot structs stay re-exported above; the mutating helpers are
// `Simulation` methods and should not be re-exported as free functions.
pub use integrity::{check_integrity, IntegrityError};
pub use invariants::{check_tick_invariants, InvariantError};
pub use lod::LodTier;
pub use lod::{
    aggregate_strategic, operational_hex_snapshot, project_zoom, should_tick_entity,
    should_tick_entity_with_policy, HexCellSnapshot, LodPolicy, ZoomLevel,
};
pub use metrics::{compute, compute_fixed, Metrics, MetricsFixed};
pub use perf::{phases_over_budget, tick_over_budget, TickProfile};
pub use policy::{
    effective_consumption, policy_from_kind, CapitalistPolicy, ControlSignals, NoopPolicy, Policy,
    PolicyInput, SubsistenceFirstPolicy, DEFAULT_ECONOMY_POLICY,
};
pub use replay::ReplayEvent;
pub use replay_format::{
    load_civreplay, save_civreplay, FOOTER_CHECKSUM_LEN, FORMAT_VERSION, MAGIC,
};
pub use save_bundle::{CivSaveMetadata, SaveBundleError, CIVSAVE_FORMAT_VERSION, CIVSAVE_SPEC_ID};
pub use scenario::{
    baseline_scenario_path, load_scenario, Scenario, ScenarioError, ScenarioMilitary,
    SCENARIO_SCHEMA_VERSION,
};
pub use spectator::{BuildingPin, CivPin, Faction, JobLabel, SpectatorView};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Fixed-point decimal used throughout the engine crate.
pub type Fixed = engine::Fixed;

/// Seeded RNG for deterministic simulation
pub type SimRng = ChaCha8Rng;

/// Create a seeded RNG from world state
pub fn create_rng(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}

/// Advance simulation by one tick (simple API)
pub fn step(mut state: WorldState, consumption_joules: Fixed) -> WorldState {
    state.tick += 1;
    let result = state
        .energy_budget_joules
        .saturating_sub(consumption_joules);
    state.energy_budget_joules = if result.to_bits() < 0 {
        Fixed::ZERO
    } else {
        result
    };
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn step_advances_tick() {
        let s = WorldState::default();
        let n = step(s, Fixed::from_num(100));
        assert_eq!(n.tick, 1);
    }

    #[test]
    fn step_decreases_energy() {
        let s = WorldState::default();
        // Initial energy is 1_000_000_000_000, subtract 1000 = 999_999_999_000
        let expected = Fixed::from_num(1_000_000_000_000i64) - Fixed::from_num(1000i64);
        let n = step(s, Fixed::from_num(1000));
        assert_eq!(n.energy_budget_joules, expected);
    }

    #[test]
    fn step_energy_floor_at_zero() {
        let s = WorldState {
            energy_budget_joules: Fixed::from_num(50),
            ..WorldState::default()
        };
        let n = step(s, Fixed::from_num(100));
        assert_eq!(n.energy_budget_joules, Fixed::ZERO);
    }

    #[test]
    fn determinism_same_seed_same_output() {
        let s1 = WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: Fixed::from_num(1000),
            rng_seed: 12345,
            factions: HashMap::new(),
            faction_treasury: HashMap::new(),
            ..WorldState::default()
        };
        let s2 = WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: Fixed::from_num(1000),
            rng_seed: 12345,
            factions: HashMap::new(),
            faction_treasury: HashMap::new(),
            ..WorldState::default()
        };

        let r1 = step(s1, Fixed::from_num(10));
        let r2 = step(s2, Fixed::from_num(10));

        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }
}
