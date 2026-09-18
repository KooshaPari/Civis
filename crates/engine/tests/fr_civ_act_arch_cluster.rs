//! Behavioural oracles for the `FR-CIV-0001-TICK` / `FR-CIV-ACT-*` /
//! `FR-CIV-ARCH-00{1,2}` / `FR-CIV-L5` / `FR-CIV-RES-001` cluster.
//!
//! This file replaces the auto-generated placeholders
//! (`crates/engine/tests/fr_fr_civ_{0001_tick,act_003,act_004,act_005,arch_001,arch_002,l5,res_001}.rs`)
//! for the IDs it covers. Every assertion below pins a concrete value, boundary,
//! count, invariant, or determinism property that a regression would break.
//!
//! Requirement sources:
//!
//! * `FR-CIV-0001-TICK` — engine-side deterministic tick loop. `Simulation::tick`
//!   documents the contract: phases run in `PHASE_ORDER` and "exactly one
//!   `ReplayEvent::Tick` is appended after all phases finish".
//! * `FR-CIV-ACT-003` — citizen job assignment
//!   (`docs/reference/REFERENCE_GAME_ANALYSIS.md:511`).
//! * `FR-CIV-ACT-004` — citizen well-being / standard-of-living scalar
//!   (`docs/reference/REFERENCE_GAME_ANALYSIS.md:183`).
//! * `FR-CIV-ACT-005` — citizen migration (`docs/reports/STATUS_REPORT.md:98`).
//! * `FR-CIV-ARCH-001/002` — `FUNCTIONAL_REQUIREMENTS.md:461-462`.
//! * `FR-CIV-L5` — `docs/development-guide/fr-l5-visual-pass.md`.
//! * `FR-CIV-RES-001` — scenario API (`docs/reference/FR_TRACKER.md:40`; real
//!   implementation `crates/engine/src/scenario.rs`).

use std::fs;
use std::path::{Path, PathBuf};

use civ_build::{
    facade_for_emergence, pick_tile_set, Allocator, ArchitectureMode, BuildingGraph,
    BuildingProvenance, DemandSignals, EmergentStyleKey, ParcelKind, TileSetProfile,
};
use civ_engine::emergent_migration::{
    candidate_pull, evaluate_migration, home_pressure, is_eligible, migration_tick,
    settlement_net_migration, AgentSnapshot, MigrationConfig, SettlementSnapshot,
};
use civ_engine::engine::{attach_citizen_to_agents, Citizen, JobType};
use civ_engine::invariants::check_tick_invariants;
use civ_engine::replay::ReplayEvent;
use civ_engine::scenario::{
    baseline_scenario_path, load_scenario, preset_names, preset_scenario_path, ScenarioError,
    SCENARIO_SCHEMA_VERSION,
};
use civ_engine::spectator::JobLabel;
use civ_engine::{Fixed, Simulation};
use civ_voxel::WorldCoord;

// ---------------------------------------------------------------------------
// shared helpers
// ---------------------------------------------------------------------------

fn tick_n(sim: &mut Simulation, ticks: usize) {
    for _ in 0..ticks {
        sim.tick();
    }
}

fn tick_marker_count(sim: &Simulation) -> usize {
    sim.replay_log()
        .events
        .iter()
        .filter(|event| matches!(event, ReplayEvent::Tick { .. }))
        .count()
}

/// Every citizen actually present in the ECS world (not the planned
/// `state.population` number).
fn citizens(sim: &Simulation) -> Vec<Citizen> {
    sim.world
        .query::<&Citizen>()
        .iter()
        .map(|(_, citizen)| *citizen)
        .collect()
}

fn migration_config() -> MigrationConfig {
    MigrationConfig {
        food_weight: 1.0,
        safety_weight: 0.0,
        labor_weight: 0.0,
        social_weight: 0.0,
        migration_threshold: 1.5,
        sample_size: 5,
        eligibility_delay: 10,
    }
}

fn settlement(food: f32, housing: i32) -> SettlementSnapshot {
    SettlementSnapshot {
        food_per_capita: food,
        safety: 0.0,
        labor_opportunity: 0.0,
        social_bonds: 0,
        housing_capacity: housing,
        population: 10,
    }
}

fn agent_at(settlement_index: u32, age_ticks: u32) -> AgentSnapshot {
    AgentSnapshot {
        age_ticks,
        settlement_index,
        has_strong_bonds: false,
        origin_settlement: 0,
    }
}

fn scenario_yaml(extra: &str) -> String {
    scenario_yaml_full("unit-test", 100, "1.0", extra)
}

fn scenario_yaml_full(name: &str, population: u64, scarcity: &str, extra: &str) -> String {
    format!(
        "name: {name}\ntick_start: 0\npopulation: {population}\n\
         base_consumption_joules: 1000\nscarcity_multiplier: {scarcity}\n{extra}"
    )
}

fn load_scenario_body(
    dir: &Path,
    body: &str,
) -> Result<civ_engine::scenario::Scenario, ScenarioError> {
    let path = dir.join("scenario.yaml");
    fs::write(&path, body).expect("write scenario fixture");
    load_scenario(&path)
}

fn workspace_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

// ===========================================================================
// FR-CIV-0001-TICK — deterministic engine tick loop
// ===========================================================================

/// Covers FR-CIV-0001-TICK.
///
/// The tick counter advances by exactly one per `tick()` call, is echoed into
/// the snapshot, and `with_seed` records the seed it was constructed from.
#[test]
fn fr_civ_0001_tick_counter_advances_exactly_once_per_call() {
    let mut sim = Simulation::with_seed(4_242);
    assert_eq!(sim.state.tick, 0, "a fresh simulation starts at tick 0");
    assert_eq!(sim.snapshot().tick, 0);
    assert_eq!(sim.state.rng_seed, 4_242, "seed is recorded in world state");

    for expected in 1..=8_u64 {
        sim.tick();
        assert_eq!(
            sim.state.tick, expected,
            "tick {expected}: one tick() call must advance the counter by exactly one"
        );
        assert_eq!(sim.snapshot().tick, sim.state.tick);
    }
}

/// Covers FR-CIV-0001-TICK.
///
/// `Simulation::tick` documents "exactly one `ReplayEvent::Tick` is appended
/// after all phases finish", so the tail of the append-only replay log is a
/// `Tick` marker for the tick that just completed, and the number of markers
/// equals the tick counter.
#[test]
fn fr_civ_0001_tick_records_one_trailing_replay_marker_per_tick() {
    let mut sim = Simulation::with_seed(99);

    for expected in 1..=6_u64 {
        sim.tick();
        let events = &sim.replay_log().events;
        assert_eq!(
            events.last(),
            Some(&ReplayEvent::Tick { tick: expected }),
            "the Tick marker must be the last event appended by tick {expected}"
        );
        assert_eq!(tick_marker_count(&sim), expected as usize);
    }

    // Determinism: the same seed replays the same marker sequence.
    let mut twin = Simulation::with_seed(99);
    tick_n(&mut twin, 6);
    assert_eq!(sim.replay_log().events, twin.replay_log().events);
    assert_eq!(sim.hash_chain_root(), twin.hash_chain_root());
}

/// Covers FR-CIV-0001-TICK.
///
/// Post-tick invariants (`civ_engine::invariants`) hold after every tick, and
/// the planet phase is tick-driven: `day_phase` is `(tick % 24000) / 24000`
/// using the earth-like `day_length_ticks` default, which proves `phase_planet`
/// ran inside the loop rather than being skipped.
#[test]
fn fr_civ_0001_tick_post_conditions_hold_and_planet_phase_is_tick_driven() {
    let mut sim = Simulation::with_seed(7);

    for expected in 1..=12_u64 {
        sim.tick();

        check_tick_invariants(&sim).expect("post-tick invariants must hold");

        let snapshot = sim.snapshot();
        assert_eq!(snapshot.climate.tick, expected, "climate is re-derived per tick");
        assert_eq!(
            snapshot.climate.day_phase,
            (expected % 24_000) as f32 / 24_000.0,
            "day_phase must be tick-derived with the 24_000-tick earth-like day"
        );
        assert!(snapshot.population > 0, "population stays non-empty");
    }
}

// ===========================================================================
// FR-CIV-ACT-003 — citizen job assignment
// ===========================================================================

/// Covers FR-CIV-ACT-003.
///
/// `job_type_for_civilian_id` is a total, pure function of the civilian id with
/// a uniform `id % 7` split: over the first 700 ids each of the seven
/// `JobType`s appears exactly 100 times, and every residue maps to a fixed job.
#[test]
fn fr_civ_act_003_job_assignment_is_total_pure_and_uniform() {
    let expected = [
        (0_u64, JobType::Farmer),
        (1, JobType::Warrior),
        (2, JobType::Scholar),
        (3, JobType::Trader),
        (4, JobType::Priest),
        (5, JobType::Admin),
        (6, JobType::Unemployed),
        (7, JobType::Farmer),
        (13, JobType::Unemployed),
        (u64::MAX, JobType::Warrior), // u64::MAX % 7 == 1
    ];
    for (id, job) in expected {
        assert_eq!(
            civ_engine::job_type_for_civilian_id(id),
            job,
            "id {id} must map to {job:?}"
        );
    }

    let mut counts = std::collections::BTreeMap::new();
    for id in 0..700_u64 {
        let job = civ_engine::job_type_for_civilian_id(id);
        assert_eq!(
            job,
            civ_engine::job_type_for_civilian_id(id),
            "assignment must be pure for id {id}"
        );
        *counts.entry(format!("{job:?}")).or_insert(0_u32) += 1;
    }
    assert_eq!(counts.len(), 7, "all seven labor categories are reachable: {counts:?}");
    for (job, count) in counts {
        assert_eq!(count, 100, "{job} must be assigned to exactly 100 of ids 0..700");
    }
}

/// Covers FR-CIV-ACT-003.
///
/// `attach_citizen_to_agents` is the single place jobs are bound to agent
/// entities: it attaches a `Citizen` with a derived job to civilians that lack
/// one, and never overwrites a job that already exists.
#[test]
fn fr_civ_act_003_attach_assigns_missing_jobs_once_and_keeps_existing_ones() {
    let mut sim = Simulation::with_seed(21);

    // Find one agent civilian and strip its Citizen to model a bare agent.
    let target = sim
        .world
        .query::<&civ_agents::Civilian>()
        .iter()
        .map(|(entity, civilian)| (entity, civilian.id))
        .next()
        .expect("seed world spawns agent civilians");
    let (entity, civilian_id) = target;
    sim.world
        .remove_one::<Citizen>(entity)
        .expect("agent civilians carry a Citizen after with_seed");
    assert!(
        sim.world.get::<&Citizen>(entity).is_err(),
        "Citizen must be gone before re-attaching"
    );

    attach_citizen_to_agents(&mut sim.world);

    let attached = *sim.world.get::<&Citizen>(entity).expect("job re-attached");
    assert_eq!(
        attached.job,
        Some(civ_engine::job_type_for_civilian_id(civilian_id)),
        "re-attachment derives the job from the civilian id"
    );
    assert_eq!(attached.welfare, Fixed::from_num(7) / Fixed::from_num(10));

    // A pre-existing job is authoritative and must survive a second pass.
    {
        let mut citizen = sim.world.get::<&mut Citizen>(entity).expect("citizen");
        citizen.job = Some(JobType::Scholar);
    }
    attach_citizen_to_agents(&mut sim.world);
    assert_eq!(
        sim.world.get::<&Citizen>(entity).expect("citizen").job,
        Some(JobType::Scholar),
        "attach_citizen_to_agents must not clobber an existing job"
    );
}

/// Covers FR-CIV-ACT-003.
///
/// Jobs are assigned at spawn, survive the tick loop, and reach the spectator
/// pin payload used by every render client (the L5 "job colors on pins" slice):
/// every pin's label must equal the assignment function applied to its id.
#[test]
fn fr_civ_act_003_jobs_survive_ticks_and_match_spectator_pins() {
    let mut sim = Simulation::with_seed(3);
    tick_n(&mut sim, 3);

    let view = sim.spectator_view();
    assert_eq!(view.civ_pins.len(), 128, "4 factions x 32 civilians");

    let expected_labels = [
        JobLabel::Farmer,
        JobLabel::Warrior,
        JobLabel::Scholar,
        JobLabel::Trader,
        JobLabel::Priest,
        JobLabel::Admin,
        JobLabel::Unemployed,
    ];
    let mut seen = std::collections::BTreeSet::new();
    for pin in &view.civ_pins {
        let job = pin
            .job
            .expect("every agent civilian has a job after the tick loop");
        let derived = JobLabel::from(civ_engine::job_type_for_civilian_id(u64::from(pin.idx)));
        assert_eq!(
            job, derived,
            "pin {} reported {job:?} but the assignment function says {derived:?}",
            pin.idx
        );
        seen.insert(format!("{job:?}"));
    }
    assert_eq!(
        seen.len(),
        expected_labels.len(),
        "the seeded population spans all seven labor categories, got {seen:?}"
    );
}

// ===========================================================================
// FR-CIV-ACT-004 — citizen well-being / standard of living
// ===========================================================================

/// Covers FR-CIV-ACT-004.
///
/// The citizen well-being scalar is `Citizen.welfare`, a fixed-point value in
/// `[0.0, 1.0]` initialised to exactly 0.700 for every spawned or attached
/// agent, and health stays in `[0.0, 1.0]` across a long tick run.
#[test]
fn fr_civ_act_004_citizen_wellbeing_scalar_is_initialised_and_bounded() {
    // `Fixed` uses scale 1_000, so 0.700 is the exact bit pattern 700.
    let expected_welfare_bits = 700;
    let zero = Fixed::ZERO.to_bits();
    let one = Fixed::ONE.to_bits();

    let sim = Simulation::with_seed(17);
    let fresh = citizens(&sim);
    assert_eq!(fresh.len(), 228, "100 seed citizens + 128 agent civilians");
    for citizen in &fresh {
        assert_eq!(
            citizen.welfare.to_bits(),
            expected_welfare_bits,
            "fresh citizens start at welfare = 0.7"
        );
        assert_eq!(citizen.health.to_bits(), one, "fresh citizens start at full health");
    }

    let mut sim = sim;
    tick_n(&mut sim, 200);
    for citizen in citizens(&sim) {
        assert!(
            (zero..=one).contains(&citizen.welfare.to_bits()),
            "welfare must stay in [0, 1], got {}",
            citizen.welfare
        );
        assert!(
            (zero..=one).contains(&citizen.health.to_bits()),
            "health must stay in [0, 1], got {}",
            citizen.health
        );
        assert!(
            citizen.age <= 200,
            "age is a per-year counter, got {} after 200 ticks",
            citizen.age
        );
    }
}

/// Covers FR-CIV-ACT-004.
///
/// TODO(FR-CIV-ACT-004): `docs/reference/REFERENCE_GAME_ANALYSIS.md:183` maps
/// the reference game's `Pop.standard_of_living` to a *citizen happiness scalar
/// bounded 0..100*. No such field exists in this workspace yet — the only actor
/// well-being scalar is `Citizen.welfare` (0..1). This test pins the current
/// ABSENCE on the client-facing actor payloads so the guard fails (and must be
/// replaced by a real oracle) the moment a happiness/standard-of-living field is
/// wired in. It is deliberately not a compliance claim.
#[test]
fn fr_civ_act_004_happiness_scalar_absent_from_actor_payloads_todo() {
    let sim = Simulation::with_seed(5);

    let spectator = serde_json::to_string(&sim.spectator_view()).expect("serialize spectator view");
    let snapshot = serde_json::to_string(&sim.snapshot()).expect("serialize snapshot");

    for (label, payload) in [("SpectatorView", &spectator), ("SimulationSnapshot", &snapshot)] {
        for forbidden in ["happiness", "standard_of_living", "standardOfLiving"] {
            assert!(
                !payload.contains(forbidden),
                "{label} unexpectedly exposes `{forbidden}`; FR-CIV-ACT-004 now needs a real \
                 value/boundary oracle instead of this TODO guard"
            );
        }
    }

    // What does exist: `Citizen.welfare` serialises as the actor welfare scalar.
    let citizen = citizens(&sim).into_iter().next().expect("citizens exist");
    let citizen_json = serde_json::to_string(&citizen).expect("serialize citizen");
    assert!(citizen_json.contains("welfare"), "got {citizen_json}");
}

// ===========================================================================
// FR-CIV-ACT-005 — citizen migration
// ===========================================================================

/// Covers FR-CIV-ACT-005.
///
/// `home_pressure` inverts settlement quality: deprivation plus resource
/// pressure, minus a social-bond dampener, clamped to `[0, 1]`. With unit food
/// weight the values are exact binary fractions.
#[test]
fn fr_civ_act_005_home_pressure_inverts_settlement_quality() {
    let config = migration_config();

    let full = home_pressure(&settlement(1.0, 10), &config);
    assert_eq!(full, 0.0, "a perfect settlement exerts no pressure");

    let half = home_pressure(&settlement(0.5, 10), &config);
    assert_eq!(half, 0.5, "half the food doubles into half the pressure");

    let none = home_pressure(&settlement(0.0, 10), &config);
    assert_eq!(none, 1.0, "deprivation saturates at the documented upper bound");

    // Social bonds dampen pressure by exactly half the social weight.
    let bonded = MigrationConfig {
        social_weight: 0.4,
        ..config
    };
    let with_bonds = SettlementSnapshot {
        social_bonds: 7,
        ..settlement(0.5, 10)
    };
    assert!(
        (home_pressure(&with_bonds, &bonded) - (0.5 - 0.2)).abs() < 1e-6,
        "bonds reduce pressure by social_weight * 0.5, got {}",
        home_pressure(&with_bonds, &bonded)
    );

    // Default configuration: starving settlements pressure, prosperous ones not.
    let default = MigrationConfig::default();
    let starving = SettlementSnapshot {
        food_per_capita: 0.05,
        safety: 0.8,
        labor_opportunity: 0.5,
        social_bonds: 0,
        housing_capacity: 10,
        population: 100,
    };
    let prosperous = SettlementSnapshot {
        food_per_capita: 0.9,
        safety: 0.9,
        labor_opportunity: 0.8,
        social_bonds: 5,
        housing_capacity: 20,
        population: 50,
    };
    assert!(home_pressure(&starving, &default) > 0.5);
    assert!(home_pressure(&prosperous, &default) < 0.2);
}

/// Covers FR-CIV-ACT-005.
///
/// `candidate_pull` cannot exceed the home pressure unless the destination has
/// housing: a full settlement has exactly zero pull, so "cannot migrate to a
/// settlement with zero housing_capacity" holds structurally.
#[test]
fn fr_civ_act_005_candidate_pull_requires_housing() {
    let config = migration_config();
    let agent = agent_at(0, 30);

    assert_eq!(
        candidate_pull(&settlement(1.0, 0), &agent, &config),
        0.0,
        "no housing capacity means zero pull"
    );
    assert_eq!(
        candidate_pull(&settlement(1.0, -5), &agent, &config),
        0.0,
        "negative housing capacity also means zero pull"
    );
    assert_eq!(
        candidate_pull(&settlement(0.8, 4), &agent, &config),
        0.8,
        "with room, pull equals the weighted quality sum"
    );

    // Pull is clamped to 1.0 even when the weights sum above one.
    let over = MigrationConfig {
        food_weight: 1.4,
        ..config
    };
    assert_eq!(candidate_pull(&settlement(1.0, 4), &agent, &over), 1.0);
}

/// Covers FR-CIV-ACT-005.
///
/// Migration eligibility is an inclusive age threshold: an agent becomes
/// eligible on exactly `eligibility_delay` ticks of age, not before.
#[test]
fn fr_civ_act_005_eligibility_boundary_is_inclusive_at_delay() {
    let config = migration_config();
    assert_eq!(config.eligibility_delay, 10);

    assert!(!is_eligible(&agent_at(0, 9), &config), "9 < 10 is ineligible");
    assert!(is_eligible(&agent_at(0, 10), &config), "age == delay is eligible");
    assert!(is_eligible(&agent_at(0, 11), &config), "11 > 10 is eligible");

    let no_delay = MigrationConfig {
        eligibility_delay: 0,
        ..config
    };
    assert!(
        is_eligible(&agent_at(0, 0), &no_delay),
        "a zero delay admits newborns"
    );
}

/// Covers FR-CIV-ACT-005.
///
/// The migration decision is a STRICT comparison against
/// `home_pressure * migration_threshold`, with an absolute floor of 0.01 on the
/// destination pull: a tie does not migrate, and a destination barely better
/// than nothing does not migrate either.
#[test]
fn fr_civ_act_005_decision_boundary_is_strict_with_pull_floor() {
    let config = migration_config(); // threshold 1.5, home pressure 0.5 -> bar 0.75
    let agent = agent_at(0, 30);
    let home = settlement(0.5, 10);

    assert!(
        evaluate_migration(&agent, &home, &[settlement(0.75, 10)], &config, 5).is_none(),
        "pull exactly at home_pressure * threshold must not migrate (strict >)"
    );

    let event = evaluate_migration(&agent, &home, &[settlement(0.8, 10)], &config, 5)
        .expect("pull above the bar must migrate");
    assert_eq!(event.source_settlement, 0);
    assert_eq!(event.target_settlement, 0);
    assert_eq!(event.tick, 5);
    assert_eq!(event.source_pressure, 0.5);
    assert_eq!(event.target_pull, 0.8);

    // Best candidate wins, not the first.
    let best = evaluate_migration(
        &agent,
        &home,
        &[settlement(0.6, 10), settlement(0.9, 10)],
        &config,
        6,
    )
    .expect("migration");
    assert_eq!(best.target_settlement, 1, "index of the best candidate entry");
    assert_eq!(best.target_pull, 0.9);

    // A full destination loses to a modest one with room, even with better quality.
    let room = evaluate_migration(
        &agent,
        &home,
        &[settlement(1.0, 0), settlement(0.8, 10)],
        &config,
        7,
    )
    .expect("the housing-backed candidate wins");
    assert_eq!(room.target_settlement, 1);
    assert_eq!(room.target_pull, 0.8);

    // The 0.01 pull floor blocks a negligible gain even with a zero bar.
    let floor = MigrationConfig {
        migration_threshold: 0.0,
        ..config
    };
    assert!(
        evaluate_migration(&agent, &settlement(1.0, 10), &[settlement(0.001, 10)], &floor, 7)
            .is_none(),
        "pull below the 0.01 floor must not migrate"
    );

    // Ineligible agents never migrate even from a desperate home.
    assert!(
        evaluate_migration(&agent_at(0, 9), &home, &[settlement(1.0, 10)], &config, 8).is_none(),
        "eligibility is a hard gate on the decision"
    );
}

/// Covers FR-CIV-ACT-005.
///
/// TODO(FR-CIV-ACT-005): defect guard.
///
/// `MigrationEvent::target_settlement` is documented as "Target settlement
/// index", but `evaluate_migration` returns the index of the best entry in the
/// *filtered* candidate list (`emergent_migration.rs:194`). `migration_tick`
/// drops the agent's own settlement from that list, so for any agent whose home
/// is not settlement 0 the reported target is shifted by one and can even equal
/// the source: the event claims an agent moved from settlement 1 to settlement 1
/// when it actually moved to settlement 2.
///
/// EXPECTED: `target_settlement == 2` (where the housing and the pull are).
/// ACTUAL  : `target_settlement == 1 == source_settlement`.
///
/// This pins the ACTUAL value so the defect stays visible and tracked; delete
/// this guard and assert the EXPECTED value once the indices are reconciled.
#[test]
fn fr_civ_act_005_target_index_conflates_candidate_and_settlement_order_todo() {
    let config = MigrationConfig {
        migration_threshold: 0.5,
        ..migration_config()
    };
    let settlements = vec![
        settlement(0.0, 10), // 0: no pull
        settlement(0.0, 10), // 1: the agent's starving home
        settlement(1.0, 10), // 2: the real destination
    ];

    let events = migration_tick(&[agent_at(1, 40)], &settlements, &config, 9);
    let event = events.first().expect("the agent leaves the starving home");

    assert_eq!(
        event.source_settlement, 1,
        "the agent really lives in settlement 1"
    );
    assert_eq!(event.source_pressure, 1.0, "home pressure saturates");
    assert_eq!(event.target_pull, 1.0, "settlement 2 is the attractive candidate");
    assert_eq!(
        event.target_settlement, 1,
        "ACTUAL (defective): filtered-list index, not the settlement index"
    );
    assert_eq!(
        settlement_net_migration(&events, 2),
        0,
        "ACTUAL (defective): the real destination is credited with no arrivals"
    );
}

/// Covers FR-CIV-ACT-005.
///
/// A migration tick evaluates every eligible agent exactly once, never lets the
/// ineligible ones move, conserves net population across the settlement set, and
/// is deterministic for identical inputs.
///
/// `MigrationEvent::target_settlement` is a filtered-candidate index rather than
/// a settlement index (see
/// `fr_civ_act_005_target_index_conflates_candidate_and_settlement_order_todo`),
/// so this test only pins its range; the destination identity is pinned through
/// `evaluate_migration` in the decision tests above.
#[test]
fn fr_civ_act_005_tick_events_are_valid_and_conserve_population() {
    let config = MigrationConfig {
        migration_threshold: 0.5,
        ..migration_config()
    };
    let settlements = vec![
        settlement(0.0, 10),  // 0: starving home of the desperate agents
        settlement(1.0, 20),  // 1: the attractive destination
        settlement(0.9, -3),  // 2: attractive on paper, but full
    ];
    let agents = vec![
        agent_at(0, 40),
        agent_at(0, 45),
        agent_at(0, 3), // too young to migrate
    ];

    let events = migration_tick(&agents, &settlements, &config, 123);
    assert_eq!(
        events.len(),
        2,
        "exactly the two eligible adults leave; the child stays"
    );

    for event in &events {
        assert_eq!(event.tick, 123, "the tick is stamped on every event");
        assert_eq!(
            event.source_settlement, 0,
            "the two eligible agents live in the starving settlement"
        );
        assert_eq!(event.source_pressure, 1.0, "their home pressure saturates");
        assert_eq!(
            event.target_pull, 1.0,
            "the housing-backed candidate is the only pull"
        );
        assert!((event.target_settlement as usize) < settlements.len());
        // TODO(FR-CIV-ACT-005): `MigrationEvent::agent_index` is documented as the
        // index of the migrating agent but is filled from `agent.settlement_index`
        // ("placeholder — real index set by caller", emergent_migration.rs:192), so
        // per-agent uniqueness cannot be asserted yet. Only its range is pinned.
        assert!((event.agent_index as usize) < settlements.len());
    }

    // Conservation: every event moves one agent out of one settlement and into
    // another, so net migration over the whole set is exactly zero.
    let net: i64 = (0..settlements.len() as u32)
        .map(|index| settlement_net_migration(&events, index))
        .sum();
    assert_eq!(net, 0, "population is conserved across the settlement set");

    // Determinism: identical inputs produce an identical event list.
    let again = migration_tick(&agents, &settlements, &config, 123);
    assert_eq!(
        serde_json::to_string(&events).expect("serialize events"),
        serde_json::to_string(&again).expect("serialize events"),
        "migration_tick must be deterministic"
    );

    // A lone settlement has no candidate, so nothing migrates for its residents.
    assert!(
        migration_tick(&agents, &settlements[..1], &config, 123).is_empty(),
        "with no other settlement there is nowhere to go"
    );
}

/// Covers FR-CIV-ACT-005.
///
/// The shipped migration tuning is pinned: the four quality weights sum to 1.0,
/// the threshold is above 1.0 (only real gains move an agent), and the sampler
/// and eligibility warm-up are non-degenerate.
#[test]
fn fr_civ_act_005_default_config_is_pinned() {
    let config = MigrationConfig::default();
    assert_eq!(config.food_weight, 0.35);
    assert_eq!(config.safety_weight, 0.25);
    assert_eq!(config.labor_weight, 0.25);
    assert_eq!(config.social_weight, 0.15);
    assert_eq!(
        config.food_weight + config.safety_weight + config.labor_weight + config.social_weight,
        1.0,
        "quality weights are a convex combination"
    );
    assert_eq!(config.migration_threshold, 1.3);
    assert_eq!(config.sample_size, 5);
    assert_eq!(config.eligibility_delay, 10);
}

// ===========================================================================
// FR-CIV-ARCH-001 — 3D tiled WFC solver wraps an external crate
// ===========================================================================

/// Covers FR-CIV-ARCH-001.
///
/// The tile vocabulary the external solver is supposed to consume is shipped by
/// `architecture_tile_sets()`: a deterministic, id-stable registry that covers
/// every `(culture, era)` pair, encodes the culture in the facade name, and
/// weights every tile-set as its own neighbour.
#[test]
fn fr_civ_arch_001_tile_registry_is_deterministic_and_complete() {
    let first = civ_engine::architecture_tile_sets();
    let second = civ_engine::architecture_tile_sets();
    assert_eq!(first, second, "the registry is lazily built but never varies");
    assert_eq!(first.len(), 24, "4 cultures x 6 eras");

    let ids: Vec<u16> = first.iter().map(|profile| profile.id).collect();
    let unique: std::collections::BTreeSet<u16> = ids.iter().copied().collect();
    assert_eq!(unique.len(), 24, "tile-set ids are unique: {ids:?}");
    assert_eq!(unique.iter().next(), Some(&1), "ids start at 1");
    assert_eq!(unique.iter().next_back(), Some(&24), "ids end at 24");

    let mut pairs: std::collections::BTreeSet<(u16, u16)> = std::collections::BTreeSet::new();
    for profile in first {
        assert!(
            pairs.insert((profile.culture, profile.era)),
            "duplicate (culture, era) pair for tile-set {}",
            profile.id
        );
        assert!(
            profile.facade.name.ends_with(&format!("-c{}", profile.culture)),
            "tile-set {} facade {:?} must encode culture {}",
            profile.id,
            profile.facade.name,
            profile.culture
        );
        assert_eq!(
            profile.adjacency_weights.get(&profile.id),
            Some(&10),
            "tile-set {} must weight itself as a neighbour",
            profile.id
        );
        assert_eq!(profile.wealth_bucket, 0);
    }
    assert_eq!(pairs.len(), 24, "every (culture, era) combination is covered");

    // The registry is the engine's single source of truth for the solver.
    let vector = civ_build::CultureEraWealthVector::new(2, 3, 0);
    let signals = DemandSignals {
        residential: 1.0,
        commercial: 0.0,
        industrial: 0.0,
        civic: 0.0,
    };
    let selected = civ_build::adjacency_weights_for_vector(
        &vector,
        &signals,
        first,
        ArchitectureMode::Canonical,
        None,
    );
    let expected = first
        .iter()
        .find(|profile| profile.id == selected_id(&vector, &signals, first))
        .expect("selected tile-set is in the registry");
    assert_eq!(&selected, &expected.adjacency_weights);
}

fn selected_id(
    vector: &civ_build::CultureEraWealthVector,
    signals: &DemandSignals,
    tile_sets: &[TileSetProfile],
) -> u16 {
    pick_tile_set(vector, signals, tile_sets, ArchitectureMode::Canonical, None)
        .expect("canonical mode resolves a tile-set for every culture/era")
}

/// Covers FR-CIV-ARCH-001.
///
/// TODO(FR-CIV-ARCH-001): the requirement states the 3D tiled solver SHALL wrap
/// `ghx_proc_gen` / `bevy_procedural_tilemaps`. That wrapper does NOT exist yet
/// (no such dependency in `Cargo.lock`, no constraint solver in `crates/build`
/// or `crates/voxel`) — tile-sets are selected by the deterministic registry
/// above instead. This guard pins the ABSENCE and must be replaced by a real
/// solver-driven oracle when the wrapper lands. It is not a compliance claim.
#[test]
fn fr_civ_arch_001_external_wfc_wrapper_absent_todo() {
    let lock = fs::read_to_string(workspace_file("../../Cargo.lock")).expect("read Cargo.lock");
    for crate_name in ["ghx_proc_gen", "bevy_procedural_tilemaps"] {
        assert!(
            !lock.contains(crate_name),
            "`{crate_name}` is now a dependency; FR-CIV-ARCH-001 needs a real solver oracle"
        );
    }

    let root = workspace_file("../..");
    for source_dir in ["crates/build/src", "crates/voxel/src"] {
        let mut pending = vec![root.join(source_dir)];
        while let Some(dir) = pending.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                if path.extension().is_some_and(|ext| ext == "rs") {
                    let text = fs::read_to_string(&path).expect("read source");
                    assert!(
                        !text.to_ascii_lowercase().contains("wave function collapse"),
                        "{} now mentions a hand-rolled WFC core, which FR-CIV-ARCH-001 forbids",
                        path.display()
                    );
                }
            }
        }
    }
}

// ===========================================================================
// FR-CIV-ARCH-002 — BuildingGraph stays the authoritative structural schema
// ===========================================================================

/// Covers FR-CIV-ARCH-002.
///
/// `Allocator::select_tile_set_id` (the graph-side selector) and the free
/// `pick_tile_set` must resolve the same tile-set for the same input, and
/// applying the selected facade must not replace or renumber the graph's
/// parcels: the graph is the authority, tiles are a decoration of it.
#[test]
fn fr_civ_arch_002_tile_selection_decrates_the_graph_without_replacing_it() {
    let mut graph = BuildingGraph::new();
    let mut allocator = Allocator::new(4_242);
    let signals = DemandSignals {
        residential: 0.9,
        commercial: 0.9,
        industrial: 0.9,
        civic: 0.9,
    };
    let origin = WorldCoord { x: 0, y: 0, z: 0 };
    let allocated = allocator.allocate(&mut graph, &signals, 3, origin, 16);
    assert_eq!(allocated.len(), 4, "one parcel per saturated demand channel");

    let parcels_before: Vec<_> = graph
        .parcels
        .iter()
        .map(|parcel| (parcel.id, parcel.kind, parcel.origin, parcel.era_min))
        .collect();
    assert_eq!(
        graph
            .provenance
            .values()
            .filter(|tag| **tag == BuildingProvenance::Procedural)
            .count(),
        4,
        "allocated parcels are tagged procedural"
    );

    let tile_sets = civ_engine::architecture_tile_sets();
    let vector = civ_build::CultureEraWealthVector::new(1, 3, 0);
    let via_fn = pick_tile_set(&vector, &signals, tile_sets, ArchitectureMode::Canonical, None);
    let via_allocator = Allocator::select_tile_set_id(
        &vector,
        &signals,
        tile_sets,
        ArchitectureMode::Canonical,
        None,
    );
    assert!(via_fn.is_some(), "canonical mode resolves a tile-set");
    assert_eq!(via_fn, via_allocator, "one selector, two call paths");

    let key = EmergentStyleKey::new(1, 3, 0, civ_build::BiomeStyleTag::FOREST);
    let facade = facade_for_emergence(key, &signals, tile_sets);
    for id in &allocated {
        graph.set_facade(*id, facade.clone());
    }

    let parcels_after: Vec<_> = graph
        .parcels
        .iter()
        .map(|parcel| (parcel.id, parcel.kind, parcel.origin, parcel.era_min))
        .collect();
    assert_eq!(
        parcels_before, parcels_after,
        "tile/facade selection must not mutate or renumber the authoritative graph"
    );
    assert_eq!(graph.facades.len(), 4);
    for id in &allocated {
        assert_eq!(
            graph.facades.get(id),
            Some(&facade),
            "facade is recorded against the graph parcel id"
        );
    }
    assert_eq!(graph.total_capacity(), 4, "one residential parcel = 4 housing slots");
    assert!(facade.name.contains("-biome"), "biome bias is applied on top: {facade:?}");
}

/// Covers FR-CIV-ARCH-002.
///
/// The live engine grows its `BuildingGraph` on the construction cadence and
/// then decorates every parcel with an emergence facade; parcels are never
/// removed, ids stay unique, and every facade key resolves to a live parcel.
#[test]
fn fr_civ_arch_002_engine_graph_grows_on_cadence_with_referentially_intact_facades() {
    let mut sim = Simulation::with_seed(11);
    assert_eq!(sim.building_graph().parcels.len(), 0, "no parcels before the first tick");

    tick_n(&mut sim, 15);
    assert_eq!(
        sim.building_graph().parcels.len(),
        0,
        "parcel allocation only happens on the 16-tick construction cadence"
    );

    sim.tick(); // tick 16
    let after_first = sim.building_graph().parcels.len();
    assert!(after_first > 0, "the first cadence boundary allocates parcels");

    tick_n(&mut sim, 16); // tick 32
    let after_second = sim.building_graph().parcels.len();
    assert!(
        after_second > after_first,
        "the graph grows again at the next cadence ({after_first} -> {after_second})"
    );

    let graph = sim.building_graph();
    let parcel_ids: std::collections::BTreeSet<_> =
        graph.parcels.iter().map(|parcel| parcel.id).collect();
    assert_eq!(parcel_ids.len(), graph.parcels.len(), "parcel ids are unique");
    let mut kinds = std::collections::HashSet::new();
    for parcel in &graph.parcels {
        kinds.insert(parcel.kind);
        assert_eq!(parcel.era_min, 1, "parcels are stamped with the target era");
        assert_eq!(
            graph.provenance.get(&parcel.id),
            Some(&BuildingProvenance::Procedural)
        );
        assert!(
            graph.facades.contains_key(&parcel.id),
            "parcel {:?} must carry an emergence facade",
            parcel.id
        );
    }
    // The allocator only inserts parcels for channels above the 0.5 saturation
    // threshold (residential 0.75, civic 0.75; commercial/industrial 0.25).
    //
    // TODO(FR-CIV-ARCH-003): the era-gated wrapper (`emergence_demand_signals`)
    // only filters the *facade* demand: `phase_construction_sites` hands the raw
    // signals to `Allocator::allocate`. Civic parcels therefore appear at era 1
    // even though `civ_build::parcel_kind_min_era(ParcelKind::Civic) == 3`. If
    // allocation is ever moved behind the era gate, this expected set loses Civic.
    assert_eq!(
        kinds,
        [ParcelKind::Residential, ParcelKind::Civic]
            .into_iter()
            .collect::<std::collections::HashSet<_>>(),
        "saturated channels only"
    );
    assert!(
        civ_build::tiers::parcel_kind_unlocked(ParcelKind::Residential, 1)
            && !civ_build::tiers::parcel_kind_unlocked(ParcelKind::Civic, 1),
        "the era gate is part of the contract the allocator bypasses"
    );
    let facade_ids: std::collections::BTreeSet<_> = graph.facades.keys().copied().collect();
    assert!(
        facade_ids.is_subset(&parcel_ids),
        "facades may only reference live graph parcels: {facade_ids:?} vs {parcel_ids:?}"
    );
}

// ===========================================================================
// FR-CIV-L5 — incremental visual presentation pass
// ===========================================================================

/// Covers FR-CIV-L5.
///
/// The `is_day` lighting flag every 3D client reads is derived from the
/// snapshot climate: `(0.25..0.75).contains(&day_phase)`. The lower bound is
/// inclusive, the upper bound is exclusive, so 0.25 is day and 0.75 is night.
#[test]
fn fr_civ_l5_is_day_flag_tracks_climate_day_phase_boundaries() {
    let mut sim = Simulation::with_seed(2);

    for (day_phase, expected) in [
        (0.0_f32, false),
        (0.24, false),
        (0.25, true),
        (0.5, true),
        (0.749_999, true),
        (0.75, false),
        (0.99, false),
    ] {
        sim.climate.day_phase = day_phase;
        assert_eq!(
            sim.spectator_view().is_day,
            expected,
            "day_phase {day_phase} must render as is_day = {expected}"
        );
    }

    // Same predicate as `civ-watch` (`crate::watch` uses `>= 0.25 && < 0.75`).
    sim.climate.day_phase = 0.5;
    assert!(sim.spectator_view().is_day);
}

/// Covers FR-CIV-L5.
///
/// The spawn slice: a civilian spawned at normalised (0.4, 0.6) appears in the
/// very next `sim.snapshot`-equivalent spectator payload with the correct
/// normalised position and a non-null job label, and pins are sorted and capped.
#[test]
fn fr_civ_l5_spawn_lands_in_spectator_pins_with_job_label() {
    let mut sim = Simulation::with_seed(9);
    let startup = sim.spectator_view();
    assert_eq!(startup.civ_pins.len(), 128);
    assert!(
        startup.civ_pins.iter().all(|pin| pin.job.is_some()),
        "every startup pin carries a job label (L5 acceptance)"
    );

    let mut rng = sim.rng_mut().clone();
    let _ = civ_agents::spawn_civilian_at(
        &mut sim.world,
        42_007,
        civ_agents::Alignment::None,
        0.4,
        0.6,
        civ_agents::ActorVisualKind::Humanoid,
        &mut rng,
    );
    *sim.rng_mut() = rng;
    attach_citizen_to_agents(&mut sim.world);

    let view = sim.spectator_view();
    assert_eq!(view.civ_pins.len(), 129, "the spawned civilian is a new pin");
    let pin = view
        .civ_pins
        .iter()
        .find(|pin| pin.idx == 42_007)
        .expect("spawned id 42_007 is pinned");
    assert!((pin.x - 0.4).abs() < 1e-6, "x = {}", pin.x);
    assert!((pin.y - 0.6).abs() < 1e-6, "y = {}", pin.y);
    assert_eq!(
        pin.job,
        Some(JobLabel::Farmer),
        "42_007 % 7 == 0 assigns Farmer to the spawned pin"
    );
    assert!(
        view.civ_pins.windows(2).all(|pair| pair[0].idx <= pair[1].idx),
        "pins are published in ascending id order"
    );
    assert!(view.civ_pins.len() <= 256, "pin payload is capped at 256 entries");
    assert!(
        view.civ_pins.iter().all(|pin| (0.0..=1.0).contains(&pin.x)
            && (0.0..=1.0).contains(&pin.y)),
        "pin coordinates are normalised"
    );
}

/// Covers FR-CIV-L5.
///
/// Territory and building pins are deterministic functions of the tick: faction
/// radii grow by 0.018 per tick with a 2.75 spacing per faction id, the twelve
/// faction pins cover all four factions with three kinds each, and the building
/// era ladder advances every 120 ticks.
#[test]
fn fr_civ_l5_faction_and_building_pins_are_tick_driven() {
    let mut sim = Simulation::with_seed(6);
    let startup = sim.spectator_view();

    assert_eq!(startup.factions.len(), 4, "four seeded factions");
    for (index, faction) in startup.factions.iter().enumerate() {
        assert_eq!(faction.id, index as u32);
        let expected = 18.0_f32 + (index as f32) * 2.75_f32;
        assert_eq!(
            faction.radius, expected,
            "faction {index} territory radius at tick 0"
        );
    }

    let faction_pins: Vec<_> = startup
        .buildings
        .iter()
        .filter(|pin| pin.id < 12)
        .collect();
    assert_eq!(faction_pins.len(), 12, "4 factions x 3 building pins");
    for faction in 0..4_u32 {
        let owned: Vec<_> = faction_pins
            .iter()
            .filter(|pin| pin.faction_id == faction)
            .collect();
        assert_eq!(owned.len(), 3, "faction {faction} owns three building pins");
        let kinds: Vec<_> = owned.iter().map(|pin| pin.kind).collect();
        assert_eq!(
            kinds,
            vec![
                civ_engine::spectator::BuildingKind::Residential,
                civ_engine::spectator::BuildingKind::Commercial,
                civ_engine::spectator::BuildingKind::Industrial,
            ],
            "pin kinds cycle per faction at tick 0"
        );
        assert!(owned.iter().all(|pin| pin.era == 0), "pre-120 ticks is era 0");
    }
    assert!(
        startup.buildings.iter().any(|pin| pin.id >= 9_000),
        "ECS CityCenter/Market/Barracks entities also surface as pins"
    );

    tick_n(&mut sim, 16);
    let later = sim.spectator_view();
    assert_eq!(later.factions[0].radius, 18.0_f32 + 16.0_f32 * 0.018_f32);
    assert!(
        later.factions[3].radius > later.factions[0].radius,
        "faction radii stay separated by 2.75 each"
    );
    assert!(later.buildings.iter().all(|pin| pin.era == 0));
}

// ===========================================================================
// FR-CIV-RES-001 — scenario API
// ===========================================================================

/// Covers FR-CIV-RES-001.
///
/// The canonical `scenarios/baseline.yaml` loads against the current schema
/// version and every documented field survives the trip through the loader.
#[test]
fn fr_civ_res_001_baseline_scenario_loads_with_canonical_values() {
    let scenario = load_scenario(baseline_scenario_path()).expect("baseline.yaml loads");

    assert_eq!(scenario.version, SCENARIO_SCHEMA_VERSION);
    assert_eq!(SCENARIO_SCHEMA_VERSION, 1);
    assert_eq!(scenario.name, "baseline");
    assert_eq!(scenario.tick_start, 0);
    assert_eq!(scenario.population, 1_000_000);
    assert_eq!(scenario.base_consumption_joules, 5_000_000_000);
    assert_eq!(scenario.scarcity_multiplier, 1.0);
    assert_eq!(scenario.fog_vision_radius, Some(8));
    assert_eq!(scenario.fog_grid_size, 64);
    assert_eq!(scenario.military.movement_cadence_ticks, Some(4));
    assert_eq!(scenario.military.movement_pulses_per_cadence, Some(2));
    assert_eq!(scenario.military.war_cadence_ticks, Some(16));
    assert_eq!(scenario.military.engage_range_grid, Some(10));
    assert_eq!(
        scenario.mods,
        vec!["mods/example-policy".to_owned(), "mods/example-economic".to_owned()]
    );
    assert_eq!(scenario.seeds, vec!["scenarios/canonical_seeds.ron".to_owned()]);
    assert_eq!(scenario.active_seed.as_deref(), Some("raw_organism"));
    assert_eq!(scenario.policy.kind, "noop");
    assert!(scenario.objectives.is_empty());
    assert_eq!(scenario.starting_conditions.civilians_per_faction, 32);
    assert_eq!(scenario.starting_conditions.faction_count, 4);
    assert_eq!(scenario.starting_conditions.quadrant_spread, 2_500);
    assert!(scenario.starting_conditions.seed_mix.is_empty());
    assert!(scenario.divergence_override.is_none());
    assert!(scenario.taxation.rates_bp.is_empty());
}

/// Covers FR-CIV-RES-001.
///
/// Omitted optional blocks fall back to the documented defaults, so old
/// scenario files keep parsing: 32 civilians per faction, 4 factions, spread
/// 2500, no mods, no divergence override, noop policy, and version 1.
#[test]
fn fr_civ_res_001_omitted_blocks_fall_back_to_documented_defaults() {
    let dir = tempfile::tempdir().expect("temp dir");
    let scenario = load_scenario_body(dir.path(), &scenario_yaml("")).expect("minimal scenario");

    assert_eq!(scenario.version, 1, "version defaults to the schema version");
    assert_eq!(scenario.starting_conditions.civilians_per_faction, 32);
    assert_eq!(scenario.starting_conditions.faction_count, 4);
    assert_eq!(scenario.starting_conditions.quadrant_spread, 2_500);
    assert!(scenario.starting_conditions.seed_mix.is_empty());
    assert_eq!(scenario.fog_grid_size, 64);
    assert_eq!(scenario.policy.kind, "noop");
    assert_eq!(scenario.military.movement_cadence_ticks, None);
    assert!(scenario.mods.is_empty());
    assert!(scenario.active_seed.is_none());

    // The scenario only applies its declared starting world state.
    let mut state = civ_engine::WorldState::default();
    scenario.apply_world_state(&mut state);
    assert_eq!(state.tick, 0);
    assert_eq!(state.population, 100);
    let policy = scenario.policy_input();
    assert_eq!(policy.base_consumption_joules, 1_000.0);
    assert_eq!(policy.scarcity_multiplier, 1.0);
}

/// Covers FR-CIV-RES-001.
///
/// Every validation rule in `Scenario::validate` rejects its own violation with
/// the offending field name, an unsupported schema version is refused outright,
/// and a missing file reports as an I/O error rather than a panic.
#[test]
fn fr_civ_res_001_validation_rejects_each_invalid_field() {
    let dir = tempfile::tempdir().expect("temp dir");
    let cases: [(&str, String); 9] = [
        ("name", scenario_yaml_full("'   '", 100, "1.0", "")),
        ("population", scenario_yaml_full("unit-test", 0, "1.0", "")),
        ("scarcity_multiplier", scenario_yaml_full("unit-test", 100, "-0.5", "")),
        ("divergence_override", scenario_yaml("divergence_override: 1.5\n")),
        (
            "starting_conditions.faction_count",
            scenario_yaml("starting_conditions:\n  faction_count: 0\n"),
        ),
        (
            "starting_conditions.faction_count",
            scenario_yaml("starting_conditions:\n  faction_count: 65\n"),
        ),
        (
            "starting_conditions.civilians_per_faction",
            scenario_yaml("starting_conditions:\n  civilians_per_faction: 100001\n"),
        ),
        (
            "starting_conditions.quadrant_spread",
            scenario_yaml("starting_conditions:\n  quadrant_spread: 0\n"),
        ),
        (
            "military.movement_cadence_ticks",
            scenario_yaml("military:\n  movement_cadence_ticks: 0\n"),
        ),
    ];

    for (field, body) in cases {
        match load_scenario_body(dir.path(), &body) {
            Err(ScenarioError::Validation { field: got, .. }) => {
                assert_eq!(got, field, "wrong field reported for {body:?}")
            }
            other => panic!("expected Validation on `{field}` for {body:?}, got {other:?}"),
        }
    }

    // A zero `seed_mix` weight is refused as well.
    let body = scenario_yaml("starting_conditions:\n  seed_mix:\n    - seed: Ardani\n      weight: 0.0\n");
    match load_scenario_body(dir.path(), &body) {
        Err(ScenarioError::Validation { field, .. }) => {
            assert_eq!(field, "starting_conditions.seed_mix")
        }
        other => panic!("expected seed_mix Validation, got {other:?}"),
    }

    // Too-new schema versions are refused before any field validation.
    let body = scenario_yaml("version: 2\n");
    match load_scenario_body(dir.path(), &body) {
        Err(ScenarioError::UnsupportedVersion {
            version, supported, ..
        }) => {
            assert_eq!(version, 2);
            assert_eq!(supported, SCENARIO_SCHEMA_VERSION);
        }
        other => panic!("expected UnsupportedVersion, got {other:?}"),
    }

    // A type error is a parse error, not a validation error.
    match load_scenario_body(dir.path(), "name: unit-test\npopulation: not-a-number\n") {
        Err(ScenarioError::Parse { .. }) => {}
        other => panic!("expected Parse error, got {other:?}"),
    }

    // A missing file reports I/O rather than panicking.
    let missing = dir.path().join("does-not-exist.yaml");
    match load_scenario(&missing) {
        Err(ScenarioError::Io { .. }) => {}
        other => panic!("expected Io error, got {other:?}"),
    }
}

/// Covers FR-CIV-RES-001.
///
/// The curated preset enumeration resolves to real, loadable scenario files, and
/// every preset shipped under `scenarios/presets/` satisfies the loader — so a
/// content edit can never silently ship an invalid preset.
#[test]
fn fr_civ_res_001_presets_are_enumerable_and_all_load() {
    assert_eq!(
        preset_names(),
        &[
            "single-race-ardani",
            "three-race-balanced",
            "ardani-dominant",
            "lush-frontier",
        ],
        "the enumerated preset set is part of the public API"
    );

    for name in preset_names() {
        let path = preset_scenario_path(name);
        let scenario = load_scenario(&path)
            .unwrap_or_else(|err| panic!("enumerated preset {name} must load: {err}"));
        assert_eq!(scenario.name, *name, "preset file name matches its `name` field");
        assert_eq!(scenario.version, SCENARIO_SCHEMA_VERSION);
        assert!(scenario.population > 0);
    }

    let presets_dir = workspace_file("../../scenarios/presets");
    let mut on_disk = std::collections::BTreeSet::new();
    for entry in fs::read_dir(&presets_dir)
        .expect("presets directory exists")
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("utf-8 preset name")
                .to_owned();
            load_scenario(&path)
                .unwrap_or_else(|err| panic!("preset {} must load: {err}", path.display()));
            on_disk.insert(name);
        }
    }
    assert!(on_disk.len() >= preset_names().len());
    for name in preset_names() {
        assert!(
            on_disk.contains(*name),
            "enumerated preset {name} has no file under scenarios/presets"
        );
    }
}

/// Covers FR-CIV-RES-001.
///
/// `Scenario::into_simulation` applies the scenario's declared world state,
/// economic policy, fog configuration, and registered mod paths to a live
/// simulation, without respawning civilians when the starting conditions are the
/// documented defaults.
#[test]
fn fr_civ_res_001_into_simulation_applies_scenario_state() {
    let scenario = load_scenario(baseline_scenario_path()).expect("baseline.yaml loads");
    let sim = scenario.into_simulation(11);

    assert_eq!(sim.state.tick, 0);
    assert_eq!(sim.state.population, 1_000_000);
    assert_eq!(
        civ_agents::count_civilians(&sim.world),
        128,
        "default starting conditions keep the 4 x 32 seeded agent population"
    );
    assert_eq!(sim.economy_policy.base_consumption_joules, 5_000_000_000.0);
    assert_eq!(sim.economy_policy.scarcity_multiplier, 1.0);
    assert_eq!(
        sim.mod_host().mods().len(),
        2,
        "both scenario mod manifests are registered"
    );
    assert_eq!(
        sim.building_graph().parcels.len(),
        0,
        "a fresh scenario simulation has not allocated parcels yet"
    );
}
