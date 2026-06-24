//! FR-CIV-BUILD-001 / 002 / 003 — Building Tier & Production Chain tests.
//!
//! These tests pin down the spec at
//! `agileplus-specs/civ-004-building-tiers/spec.md`. They reference the
//! public surface that the implementation must provide:
//!
//! * `civ_build::{BuildingTier, ProductionChain, ProductionGood, BuildingSpec, BuildSite,
//!                ProductionEvent, BuildingSpecOverride, BuildingSpecOverrideError}`
//! * `civ_economy::{EconomyState, Stocks}` (with `stocks().food` / `wood` / `metal` /
//!   `energy` / `tools` accessors)
//!
//! The tests currently FAIL TO COMPILE because that surface does not exist
//! yet — that is the red step of TDD. The implementation lands in a follow-up
//! commit; once it does, the tests compile and pass.

use civ_build::{
    BuildingGraph, BuildingId, BuildingSpec, BuildingTier, ProductionChain, ProductionGood,
};
use civ_economy::{EconomyState, Stocks};
use civ_voxel::WorldCoord;

// =====================================================================
// FR-CIV-BUILD-001 — BuildingTier enum with distinct input/output slots.
// =====================================================================

/// All four `BuildingTier` variants must be creatable with distinct slot
/// counts (Primitive=1, Artisan=2, Industrial=3, Advanced=4).
#[test]
fn fr_build_001_all_tiers_have_distinct_slot_counts() {
    let primitive = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm);
    let artisan = BuildingSpec::minimal(BuildingTier::Artisan, ProductionChain::Farm);
    let industrial = BuildingSpec::minimal(BuildingTier::Industrial, ProductionChain::Farm);
    let advanced = BuildingSpec::minimal(BuildingTier::Advanced, ProductionChain::Farm);

    assert_eq!(primitive.input_slots(), 1);
    assert_eq!(artisan.input_slots(), 2);
    assert_eq!(industrial.input_slots(), 3);
    assert_eq!(advanced.input_slots(), 4);

    assert_eq!(primitive.output_slots(), 1);
    assert_eq!(artisan.output_slots(), 2);
    assert_eq!(industrial.output_slots(), 3);
    assert_eq!(advanced.output_slots(), 4);
}

/// Tier determines the base Joule cost (Primitive < Artisan < Industrial < Advanced).
#[test]
fn fr_build_001_base_joule_cost_scales_with_tier() {
    let p = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm).base_joules();
    let a = BuildingSpec::minimal(BuildingTier::Artisan, ProductionChain::Farm).base_joules();
    let i = BuildingSpec::minimal(BuildingTier::Industrial, ProductionChain::Farm).base_joules();
    let adv = BuildingSpec::minimal(BuildingTier::Advanced, ProductionChain::Farm).base_joules();

    assert!(p < a, "Primitive ({p}) must be cheaper than Artisan ({a})");
    assert!(a < i, "Artisan ({a}) must be cheaper than Industrial ({i})");
    assert!(i < adv, "Industrial ({i}) must be cheaper than Advanced ({adv})");
}

/// Building tier is serializable (for replay + YAML scenario config).
#[test]
fn fr_build_001_tier_ron_round_trips() {
    use ron::{from_str, to_string};

    let spec = BuildingSpec::minimal(BuildingTier::Industrial, ProductionChain::Workshop);
    let encoded = to_string(&spec).expect("serialize BuildingSpec");
    let decoded: BuildingSpec = from_str(&encoded).expect("deserialize BuildingSpec");
    assert_eq!(decoded, spec);
}

/// Each tier has a deterministic construction tick count (Primitive fastest,
/// Advanced slowest) — drives the `phase_buildings` per-tick progress loop.
#[test]
fn fr_build_001_construction_ticks_scale_with_tier() {
    assert!(
        BuildingTier::Primitive.construction_ticks()
            < BuildingTier::Artisan.construction_ticks()
    );
    assert!(
        BuildingTier::Artisan.construction_ticks()
            < BuildingTier::Industrial.construction_ticks()
    );
    assert!(
        BuildingTier::Industrial.construction_ticks()
            < BuildingTier::Advanced.construction_ticks()
    );
}

// =====================================================================
// FR-CIV-BUILD-002 — Production chain halts when any required input is zero.
// =====================================================================

/// Farm chain: 0 inputs, 1 output (Food). Produces every tick.
#[test]
fn fr_build_002_farm_produces_food_when_active() {
    let spec = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm);
    let mut economy = EconomyState::with_energy_budget(1_000);
    let mut site = BuildSite::new(BuildingId(1), spec, WorldCoord { x: 0, y: 0, z: 0 });
    site.complete(); // skip construction; test production only

    let before = economy.stocks().food();
    site.produce(&mut economy);
    let after = economy.stocks().food();

    assert!(after > before, "Farm must produce food (before={before}, after={after})");
}

/// Workshop chain: 1 input (Wood), 1 output (Tools). Halt when Wood = 0.
#[test]
fn fr_build_002_workshop_halts_when_wood_is_zero() {
    let spec = BuildingSpec::minimal(BuildingTier::Artisan, ProductionChain::Workshop);
    let mut economy = EconomyState::with_energy_budget(1_000);
    economy.stocks_mut().wood = 0;

    let mut site = BuildSite::new(BuildingId(2), spec, WorldCoord { x: 0, y: 0, z: 0 });
    site.complete();

    let before_tools = economy.stocks().tools();
    site.produce(&mut economy);
    let after_tools = economy.stocks().tools();

    assert_eq!(before_tools, after_tools, "Workshop must halt when Wood = 0");
}

/// Factory chain: 2 inputs (Wood + Metal), 2 outputs (Energy + Tools). Halt
/// when either is zero (no phantom output).
#[test]
fn fr_build_002_factory_halts_when_any_input_is_zero() {
    let spec = BuildingSpec::minimal(BuildingTier::Industrial, ProductionChain::Factory);
    let mut economy = EconomyState::with_energy_budget(10_000);
    economy.stocks_mut().wood = 5;
    economy.stocks_mut().metal = 0; // halt trigger

    let mut site = BuildSite::new(BuildingId(3), spec, WorldCoord { x: 0, y: 0, z: 0 });
    site.complete();

    let before_energy = economy.stocks().energy();
    let before_tools = economy.stocks().tools();
    let before_wood = economy.stocks().wood();

    site.produce(&mut economy);

    assert_eq!(economy.stocks().energy(), before_energy, "no phantom energy");
    assert_eq!(economy.stocks().tools(), before_tools, "no phantom tools");
    assert_eq!(economy.stocks().wood(), before_wood, "no input consumption when halted");
}

/// Production chain halt is logged via a `ProductionEvent` (FR-CIV-BUILD-002
/// non-functional: "Production events logged with entity ID, good type,
/// quantity, tick number").
#[test]
fn fr_build_002_halts_emit_production_event() {
    let spec = BuildingSpec::minimal(BuildingTier::Artisan, ProductionChain::Workshop);
    let mut economy = EconomyState::with_energy_budget(1_000);
    economy.stocks_mut().wood = 0;

    let mut site = BuildSite::new(BuildingId(42), spec, WorldCoord { x: 0, y: 0, z: 0 });
    site.complete();

    let events: Vec<civ_build::ProductionEvent> = site.produce_and_collect(&mut economy, 7);
    assert_eq!(events.len(), 1, "expected one halt event");
    assert!(matches!(events[0], civ_build::ProductionEvent::Halted { tick: 7, .. }));
}

// =====================================================================
// FR-CIV-BUILD-003 — Scenario YAML overrides apply without recompile.
// =====================================================================

/// `ProductionChain::Farm` MUST consume zero inputs and produce Food only.
/// Pins the contract that `BuildSite::produce` and YAML schema validation
/// both honor.
#[test]
fn fr_build_003_chain_input_output_signatures_are_pinned() {
    let farm = ProductionChain::Farm;
    assert!(farm.inputs().is_empty(), "Farm consumes no inputs");
    assert_eq!(farm.outputs(), vec![ProductionGood::Food]);

    let workshop = ProductionChain::Workshop;
    assert_eq!(workshop.inputs(), vec![ProductionGood::Wood]);
    assert_eq!(workshop.outputs(), vec![ProductionGood::Tools]);

    let factory = ProductionChain::Factory;
    assert_eq!(factory.inputs(), vec![ProductionGood::Wood, ProductionGood::Metal]);
    assert_eq!(
        factory.outputs(),
        vec![ProductionGood::Energy, ProductionGood::Tools]
    );
}

/// YAML scenario override changes `BuildingSpec::production_rate` without
/// touching code. This is the FR-CIV-BUILD-003 acceptance criterion:
/// "Building spec overrides in scenario YAML apply without recompile".
#[test]
fn fr_build_003_yaml_override_changes_production_rate() {
    let base = BuildingSpec::minimal(BuildingTier::Industrial, ProductionChain::Factory);
    let overridden = base
        .clone()
        .apply_override(civ_build::BuildingSpecOverride { production_rate: 5 })
        .expect("valid override");

    assert_eq!(overridden.production_rate(), 5);
    assert_eq!(overridden.tier(), base.tier());
    assert_eq!(overridden.chain(), base.chain());
}

/// Invalid YAML (missing required field) is rejected with a descriptive
/// error referencing the field path.
#[test]
fn fr_build_003_yaml_override_rejects_missing_field() {
    let base = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm);
    let err = base
        .apply_override(civ_build::BuildingSpecOverride { production_rate: 0 })
        .expect_err("production_rate=0 must be rejected");

    assert!(matches!(err, civ_build::BuildingSpecOverrideError::InvalidValue { .. }));
    assert!(err.to_string().contains("production_rate"), "error message must name the field");
}

// =====================================================================
// `BuildSite` construction lifecycle (per-tick progress + completion).
// =====================================================================

/// Construction site progresses by exactly 1 tick per `BuildSite::tick`.
#[test]
fn build_site_progresses_one_tick_at_a_time() {
    let spec = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm);
    let ticks_needed = spec.tier.construction_ticks();
    let mut site = BuildSite::new(BuildingId(10), spec, WorldCoord { x: 0, y: 0, z: 0 });

    for i in 0..ticks_needed {
        assert!(!site.is_complete(), "site should not be complete at tick {i}");
        let event = site.tick();
        assert!(event.is_none(), "no completion event yet at tick {i}");
        assert_eq!(site.progress(), i + 1);
    }

    let event = site.tick();
    assert!(event.is_some(), "completion event expected on final tick");
    assert!(site.is_complete());
}

/// Construction site does NOT produce resources before completion.
#[test]
fn build_site_does_not_produce_before_complete() {
    let spec = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm);
    let mut economy = EconomyState::with_energy_budget(1_000);

    let mut site = BuildSite::new(BuildingId(11), spec, WorldCoord { x: 0, y: 0, z: 0 });
    let before = economy.stocks().food();
    site.produce(&mut economy);
    let after = economy.stocks().food();
    assert_eq!(before, after, "incomplete site must not produce");
}

/// Multiple construction sites tracked simultaneously — common case where
/// player queues 3 farms and 1 workshop in a single tick.
#[test]
fn multiple_build_sites_tick_independently() {
    let farm = BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm);
    let workshop = BuildingSpec::minimal(BuildingTier::Artisan, ProductionChain::Workshop);

    let mut sites = vec![
        BuildSite::new(BuildingId(20), farm, WorldCoord { x: 0, y: 0, z: 0 }),
        BuildSite::new(BuildingId(21), farm, WorldCoord { x: 1, y: 0, z: 0 }),
        BuildSite::new(BuildingId(22), workshop, WorldCoord { x: 2, y: 0, z: 0 }),
    ];

    for _ in 0..100 {
        for site in &mut sites {
            site.tick();
        }
    }

    for site in &sites {
        assert!(site.is_complete(), "all sites complete after 100 ticks");
    }
}

// =====================================================================
// `BuildingGraph` integration — completed sites recorded in graph.
// =====================================================================

/// After `phase_buildings` completes a site, the building is recorded in
/// `BuildingGraph::completed` with its tier and chain metadata.
#[test]
fn completed_site_lands_in_building_graph() {
    let mut graph = BuildingGraph::new();
    let spec = BuildingSpec::minimal(BuildingTier::Industrial, ProductionChain::Factory);
    let mut site = BuildSite::new(BuildingId(99), spec, WorldCoord { x: 0, y: 0, z: 0 });

    while !site.is_complete() {
        site.tick();
    }

    graph.record_completed(&site);

    assert_eq!(graph.completed_count(), 1);
    let recorded = graph.completed(BuildingId(99)).expect("site 99 recorded");
    assert_eq!(recorded.tier(), BuildingTier::Industrial);
    assert_eq!(recorded.chain(), ProductionChain::Factory);
}
