//! FR-CIV-NOTIFY-9xx cluster — real behavioural oracles.
//!
//! Replaces the auto-generated `fr_fr_civ_notify_9{01,10,11,20,21}.rs`
//! placeholders, which only asserted `WorldState::default().tick == 0` and
//! therefore could not fail if any of these requirements regressed.
//!
//! Requirement text: `docs/specs/requirements/FR-CIV-NOTIFY.md`,
//! `docs/design/onboarding-qol.md`.
//!
//! ## What each ID is anchored to (engine-reachable surface)
//!
//! * **FR-CIV-NOTIFY-901** — "alert thresholds SHALL be data-driven and
//!   configurable … firing reproducible under fixed seed". Engine surface:
//!   `gameplay::{VictoryCondition, ScenarioObjective}` + the scenario YAML
//!   `objectives:` block (thresholds read from config, not from constants),
//!   `social_types::UnrestLevel::from_score` (the measured-state classifier the
//!   alert feed buckets on), and `conditions::{check_outcome, outcome_progress}`.
//! * **FR-CIV-NOTIFY-910** — "statistics dashboards SHALL present time-series
//!   … charts read CIV-0103 time-series; update live". Engine surface: the
//!   append-only, tick-keyed, hash-chained replay series (`ReplayLog`), plus the
//!   per-tick dashboard frames (`gameplay::compute_gameplay_state`,
//!   `last_tick_mood`, `snapshot().outcome_progress`).
//! * **FR-CIV-NOTIFY-911** — "stats SHALL be readable at empire scale (aggregate
//!   + drill-down) … 100k+-agent aggregates without stalling". Engine surface:
//!   `lod::{aggregate_strategic, operational_hex_snapshot}` (region rollup ↔
//!   per-district drill-down) and the per-region stats frames
//!   (`last_tick_mood_all` ↔ `last_tick_mood`).
//! * **FR-CIV-NOTIFY-920** — "onboarding SHALL use progressive disclosure: …
//!   a first-run flow … skippable". Engine surface: `tutorial::TutorialProgress`
//!   (a progressive state machine gated on measured milestones, surfaced on
//!   `SimulationSnapshot::tutorial_progress`) and the fact that the first-run
//!   flow never gates the simulation clock.
//! * **FR-CIV-NOTIFY-921** — "a full, rebindable hotkey map SHALL cover camera,
//!   tools, overlays, speed, and selection … conflict detection". The key→action
//!   table itself lives in the clients (`clients/bevy-ref/src/{camera,
//!   controls_help}.rs`), which a `civ-engine` integration test cannot reach.
//!   What *is* testable here is the action contract those bindings dispatch
//!   into: the validated control actions (`command_queue::CommandKind`) and the
//!   god-tool action catalog (`godtools`), which must be individually nameable
//!   (a binding names exactly one action), persistable (so bindings can be
//!   saved and rebound), and validated (so a bound action cannot silently
//!   no-op), with deterministic effects.
//!
//! ## Known defect found by this file (not weakened, reported instead)
//!
//! `GodToolRequest` documents that it "round-trips cleanly so a Bevy → engine
//! bridge can serialize requests over the same `EditCommand` channel"
//! (`crates/engine/src/godtools.rs`, type-level docs). That is false for every
//! inner-tagged family: `GodToolRequest` carries `#[serde(tag = "kind")]` and so
//! do `LifeRequest` / `DisasterRequest` / `InspectRequest` / `LawRequest`, so the
//! two tags collide on the same JSON object. `serde_json` then emits a duplicate
//! `kind` key and rejects the round-trip:
//!
//! ```text
//! expected: serde_json::from_str::<GodToolRequest>(&serde_json::to_string(&req)?)? == req
//! actual:   Error("duplicate field `kind`", line: 1, column: 21)
//!           for {"kind":"life","kind":"spawn_herd","count":4,...}
//! ```
//!
//! `Terraform`/`Material` (their payloads are plain structs) do round-trip, which
//! is why the catalog-level assertions below round-trip the *payload* types and
//! not the wrapper for those families. Fixing it is a source change (rename the
//! outer tag, e.g. `#[serde(tag = "tool")]`, or use `#[serde(flatten)]`), which is
//! out of scope for this test file.

use std::time::{Duration, Instant};

use civ_engine::command_queue::{Command, CommandError, CommandKind, CommandQueue};
use civ_engine::conditions::{check_outcome, outcome_progress, GameOutcome, POPULATION_VICTORY};
use civ_engine::gameplay::{
    check_domination, compute_gameplay_state, ScenarioObjective, VictoryCondition, VictoryType,
    CULTURAL_BELIEF_THRESHOLD, DOMINATION_TERRITORY_THRESHOLD, ECONOMIC_RESOURCE_THRESHOLD,
    SCIENTIFIC_TECH_TIER,
};
use civ_engine::godtools::{
    ActorFootprintRequest, DisasterRequest, GodToolError, GodToolReceipt, GodToolRequest,
    InspectRequest, LawRequest, LifeRequest, MaterialOp, MaterialRequest, ProbeRequest,
    SpawnHerdRequest, TerraformOp, TerraformRequest,
};
use civ_engine::lod::{aggregate_strategic, operational_hex_snapshot, HexCellSnapshot};
use civ_engine::replay::ReplayEvent;
use civ_engine::scenario::{load_scenario, SCENARIO_SCHEMA_VERSION};
use civ_engine::social_types::{MOOD_CRIME_BASE, MOOD_MAX, MOOD_MIN};
use civ_engine::tech::FactionTechState;
use civ_engine::{
    DiplomacyEvent, DiplomacyKind, Fixed, ReligiousProfile, Resources, Simulation,
    TutorialMilestone, TutorialProgress, UnrestLevel, WorldCoord,
};

// ---------------------------------------------------------------------------
// FR-CIV-NOTIFY-901 — data-driven, configurable alert thresholds
// ---------------------------------------------------------------------------

/// Covers FR-CIV-NOTIFY-901.
///
/// The alert feed buckets measured unrest into `UnrestLevel`; a threshold bug
/// here mis-fires every unrest / riot / collapse alert. Exact boundaries plus the
/// "levels advance one rank at a time" property are the oracle.
#[test]
fn notify_901_unrest_alert_levels_fire_on_documented_score_boundaries() {
    let cases = [
        (0, UnrestLevel::Stable),
        (49, UnrestLevel::Stable),
        (50, UnrestLevel::Restless),
        (149, UnrestLevel::Restless),
        (150, UnrestLevel::Rioting),
        (299, UnrestLevel::Rioting),
        (300, UnrestLevel::Revolting),
        (500, UnrestLevel::Revolting),
    ];
    for (score, expected) in cases {
        assert_eq!(
            UnrestLevel::from_score(score),
            expected,
            "unrest score {score} must classify as {expected:?}"
        );
    }

    // Monotonic across the documented 0..=500 range, and any step up is exactly
    // one rank (a skipped rank would silently drop an alert tier).
    let mut previous = UnrestLevel::from_score(0);
    for score in 0..=500 {
        let level = UnrestLevel::from_score(score);
        assert!(
            level >= previous,
            "unrest classification must never regress: score {score} gave {level:?} after {previous:?}"
        );
        if level != previous {
            assert_eq!(
                level.to_rank(),
                previous.to_rank() + 1,
                "score {score} jumped from {previous:?} to {level:?} without the intermediate rank"
            );
            previous = level;
        }
    }
}

/// Covers FR-CIV-NOTIFY-901.
///
/// "Thresholds in config": a scenario YAML `objectives:` block must be able to
/// replace the built-in constant, and the alert must then fire exactly at the
/// configured value. Also covers the deadline branch (an objective that never
/// fires resolves as a defeat at its configured tick limit).
#[test]
fn notify_901_thresholds_come_from_scenario_config_not_hardcoded_constants() {
    // The descriptor's default thresholds are the documented constants.
    let condition = |victory_type, threshold| VictoryCondition {
        victory_type,
        faction_id: 0,
        threshold,
    };
    assert_eq!(
        condition(VictoryType::Domination, None).effective_threshold(),
        DOMINATION_TERRITORY_THRESHOLD
    );
    assert_eq!(
        condition(VictoryType::Cultural, None).effective_threshold(),
        CULTURAL_BELIEF_THRESHOLD
    );
    assert_eq!(
        condition(VictoryType::Economic, None).effective_threshold(),
        ECONOMIC_RESOURCE_THRESHOLD
    );
    assert_eq!(
        condition(VictoryType::Scientific, None).effective_threshold(),
        SCIENTIFIC_TECH_TIER as f32
    );

    let yaml = [
        format!("version: {SCENARIO_SCHEMA_VERSION}"),
        "name: notify-901-threshold-fixture".to_owned(),
        "tick_start: 0".to_owned(),
        "population: 1000".to_owned(),
        "base_consumption_joules: 10".to_owned(),
        "scarcity_multiplier: 1.0".to_owned(),
        "objectives:".to_owned(),
        "  - condition:".to_owned(),
        "      victory_type: domination".to_owned(),
        "      faction_id: 0".to_owned(),
        "      threshold: 0.5".to_owned(),
        "    tick_limit: 40".to_owned(),
    ]
    .join("\n");

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("notify-901-thresholds.yaml");
    std::fs::write(&path, format!("{yaml}\n")).expect("write scenario fixture");

    let scenario = load_scenario(&path).expect("scenario fixture must load");
    assert_eq!(scenario.objectives.len(), 1, "config objective must be parsed");
    let objective: ScenarioObjective = scenario.objectives[0].clone();
    assert_eq!(
        objective.condition.effective_threshold(),
        0.5,
        "the configured threshold must be used verbatim"
    );
    assert_ne!(
        objective.condition.effective_threshold(),
        DOMINATION_TERRITORY_THRESHOLD,
        "a 0.5 config override must not silently fall back to the 0.75 constant"
    );
    assert_eq!(objective.tick_limit, Some(40));

    let mut sim = scenario.clone().into_simulation(0x901_C0FD);
    sim.state.faction_treasury.clear();
    sim.state.faction_treasury.insert(0, Fixed::from_num(500_i64));
    sim.state.faction_treasury.insert(1, Fixed::from_num(500_i64));

    assert!(
        !check_domination(&sim.state, 0, DOMINATION_TERRITORY_THRESHOLD),
        "a 0.5 share must not satisfy the built-in 0.75 domination constant"
    );
    let fired = objective
        .evaluate(&sim)
        .expect("the configured 0.5 threshold must fire at exactly a 0.5 share");
    assert!(
        matches!(&fired, GameOutcome::Victory(reason) if reason.starts_with("Domination Victory")),
        "expected a domination alert, got {fired:?}"
    );

    // One unit below the configured threshold must stay silent.
    sim.state.faction_treasury.insert(0, Fixed::from_num(499_i64));
    sim.state.faction_treasury.insert(1, Fixed::from_num(501_i64));
    assert_eq!(
        objective.evaluate(&sim),
        None,
        "a 0.499 share must not fire a 0.5 alert threshold"
    );

    // The deadline branch fires once the configured tick limit is reached.
    sim.state.tick = 40;
    let expired = objective
        .evaluate(&sim)
        .expect("the configured tick_limit must resolve the objective");
    assert!(
        matches!(&expired, GameOutcome::Defeat(reason) if reason.contains("expired") && reason.contains("40")),
        "expected a deadline defeat naming tick 40, got {expired:?}"
    );
}

/// Covers FR-CIV-NOTIFY-901.
///
/// "Firing reproducible under fixed seed": the same seed plus the same measured
/// state must fire the same alert, and the population-collapse alert must fire on
/// the measured boundary (0) rather than on an authored scripted event.
#[test]
fn notify_901_alert_firing_is_reproducible_under_a_fixed_seed() {
    const SEED: u64 = 0x901_F00D;

    let mut a = Simulation::with_seed(SEED);
    let mut b = Simulation::with_seed(SEED);
    assert!(
        !a.state.factions.is_empty(),
        "a seeded sim must place factions, otherwise the collapse alert is meaningless"
    );
    for _ in 0..3 {
        a.tick();
        b.tick();
    }
    assert_eq!(
        outcome_progress(&a),
        outcome_progress(&b),
        "same seed must produce the same threshold progress"
    );
    assert_eq!(
        a.replay_log(),
        b.replay_log(),
        "same seed must produce the same history"
    );
    assert_eq!(check_outcome(&a).tag(), check_outcome(&b).tag());

    // Population-collapse alert: fires at the measured boundary, not before.
    a.state.population = 1;
    b.state.population = 1;
    assert_eq!(check_outcome(&a), GameOutcome::Ongoing);

    a.state.population = 0;
    b.state.population = 0;
    let collapse = check_outcome(&a);
    assert!(
        matches!(&collapse, GameOutcome::Defeat(reason) if reason == "Civilization Collapsed"),
        "population collapse must fire the collapse alert, got {collapse:?}"
    );
    assert_eq!(
        check_outcome(&a),
        check_outcome(&b),
        "the same seed + measured state must fire an identical alert"
    );
    assert_eq!(
        compute_gameplay_state(&a).resolved_outcome,
        Some(GameOutcome::Defeat("Extinction".to_owned())),
        "the dashboard defeat condition must agree with the alert"
    );

    // Population-victory alert boundary (the "happiness below X" shape):
    // POPULATION_VICTORY - 1 is silent, POPULATION_VICTORY fires.
    let mut c = Simulation::with_seed(SEED);
    c.state.population = POPULATION_VICTORY - 1;
    assert_eq!(check_outcome(&c), GameOutcome::Ongoing);
    c.state.population = POPULATION_VICTORY;
    assert!(
        matches!(check_outcome(&c), GameOutcome::Victory(reason) if reason == "Thriving Civilization"),
        "population must fire its victory alert exactly at the threshold"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-NOTIFY-910 — statistics dashboard time-series
// ---------------------------------------------------------------------------

fn tick_series(sim: &Simulation) -> Vec<u64> {
    sim.replay_log()
        .events
        .iter()
        .filter_map(|event| match event {
            ReplayEvent::Tick { tick } => Some(*tick),
            _ => None,
        })
        .collect()
}

fn measured_territory_share(sim: &Simulation, faction_id: u32) -> f32 {
    let total: f64 = sim
        .state
        .faction_treasury
        .values()
        .map(|value| value.to_f64().max(0.0))
        .sum();
    if total <= 0.0 {
        return 0.0;
    }
    let share = sim
        .state
        .faction_treasury
        .get(&faction_id)
        .map(|value| value.to_f64().max(0.0))
        .unwrap_or(0.0);
    (share / total) as f32
}

fn resource_units(resources: &Resources) -> f64 {
    resources.food.to_f64().max(0.0)
        + resources.wood.to_f64().max(0.0)
        + resources.metal.to_f64().max(0.0)
        + resources.energy.to_f64().max(0.0)
}

fn measured_resource_share(sim: &Simulation, faction_id: u32) -> f32 {
    let total: f64 = sim.state.faction_resources.values().map(resource_units).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let share = sim
        .state
        .faction_resources
        .get(&faction_id)
        .map(resource_units)
        .unwrap_or(0.0);
    (share / total) as f32
}

/// Covers FR-CIV-NOTIFY-910.
///
/// The dashboard charts read a frozen, append-only, tick-keyed series. Oracle:
/// exactly one point per tick, in order, hash-chained, never rewritten by later
/// ticks (a prefix must be identical after appending), and reproduced bit-for-bit
/// by the same seed.
#[test]
fn notify_910_tick_series_is_append_only_tick_keyed_and_reproducible() {
    let mut sim = Simulation::with_seed(0x910_5E21);
    for _ in 0..12 {
        sim.tick();
    }
    let series = tick_series(&sim);
    assert_eq!(
        series,
        (1..=12).collect::<Vec<u64>>(),
        "the series must carry exactly one tick-keyed point per simulated tick, in order"
    );
    assert!(
        sim.replay_log().events.len() > series.len(),
        "fixture must not be a bare tick log"
    );
    sim.replay_log()
        .verify_hash_chain()
        .expect("the tick series must be hash-chained");
    let root_before = sim.hash_chain_root().expect("chain root after 12 ticks");
    let prefix = sim.replay_log().events.clone();

    for _ in 0..4 {
        sim.tick();
    }
    let after = tick_series(&sim);
    assert_eq!(after.len(), 16);
    assert_eq!(
        &after[..12],
        &series[..],
        "updating the dashboard must not rewrite earlier series points"
    );
    assert_eq!(
        &sim.replay_log().events[..prefix.len()],
        &prefix[..],
        "append-only: the recorded prefix must be identical after more ticks"
    );
    assert_ne!(
        sim.hash_chain_root(),
        Some(root_before),
        "the chained root must advance with the series"
    );
    sim.replay_log()
        .verify_hash_chain()
        .expect("chain must still verify after appends");

    let mut twin = Simulation::with_seed(0x910_5E21);
    for _ in 0..16 {
        twin.tick();
    }
    assert_eq!(
        twin.replay_log(),
        sim.replay_log(),
        "the same seed must reproduce the identical series (charts replay exactly)"
    );
    assert_eq!(tick_series(&twin), after);
}

/// Covers FR-CIV-NOTIFY-910.
///
/// "Charts … update live": each per-tick dashboard frame must equal the measured
/// engine state *after* that tick (no stale frame, no authored value), the mood
/// time-series must carry the measured sub-scores and their per-tick delta, and
/// rendering a frame must not mutate the simulation.
#[test]
fn notify_910_dashboard_frames_track_measured_state_live_each_tick() {
    let mut sim = Simulation::with_seed(0x910_D15A);
    sim.state.factions.clear();
    sim.state.factions.insert(0, "Ardani".to_owned());
    sim.state.factions.insert(1, "Velthari".to_owned());
    sim.state.faction_treasury.clear();
    sim.state.faction_treasury.insert(0, Fixed::from_num(600_i64));
    sim.state.faction_treasury.insert(1, Fixed::from_num(400_i64));
    sim.state.faction_resources.insert(
        0,
        Resources {
            food: Fixed::from_num(300_i64),
            ..Resources::default()
        },
    );
    sim.state.faction_resources.insert(
        1,
        Resources {
            food: Fixed::from_num(100_i64),
            ..Resources::default()
        },
    );
    sim.set_settlement_population(0, 100);
    sim.set_settlement_housing_capacity(0, 100);

    let mut live_frames = 0usize;
    let mut food_series: Vec<i64> = Vec::new();
    let mut crime_series: Vec<i64> = Vec::new();
    let mut previous_mood: Option<i64> = None;
    for tick in 1..=6i64 {
        let stale_share = measured_territory_share(&sim, 0);
        // Scripted, measured inputs between frames: food stock and crime pressure.
        sim.set_settlement_food_stocked(0, tick * 5_000);
        sim.set_settlement_crime_pressure(0, (tick as i32) * 10);
        sim.tick();

        let frame = compute_gameplay_state(&sim);
        assert_eq!(
            frame.tick, sim.state.tick,
            "a dashboard frame must be stamped with its own tick"
        );

        let measured_share = measured_territory_share(&sim, 0);
        let faction0 = frame.faction_progress.get(&0).expect("faction 0 progress");
        assert!(
            (faction0.territory_share - measured_share).abs() < 1e-6,
            "tick {tick}: territory series must read the measured treasury share \
             ({measured_share}), got {}",
            faction0.territory_share
        );
        let measured_resources = measured_resource_share(&sim, 0);
        assert!(
            (faction0.resource_share - measured_resources).abs() < 1e-6,
            "tick {tick}: resource series must read the measured stock share \
             ({measured_resources}), got {}",
            faction0.resource_share
        );
        if (measured_share - stale_share).abs() > 1e-9 {
            // A frame cached at/before bind time would report the stale share.
            assert!(
                (faction0.territory_share - stale_share).abs() > 1e-9,
                "tick {tick}: the frame must be recomputed after the tick, not replayed"
            );
            live_frames += 1;
        }

        let snapshot = sim.last_tick_mood(0).expect("per-tick mood frame").clone();
        let food_score = (tick * 5_000 / 200).clamp(MOOD_MIN, MOOD_MAX);
        let housing_score = 0_i64; // population == capacity
        let crime_score = (MOOD_CRIME_BASE - 4 * tick * 10).clamp(0, MOOD_CRIME_BASE);
        assert_eq!(
            snapshot.food_score, food_score,
            "tick {tick}: food sub-score must track the scripted measured stock"
        );
        assert_eq!(snapshot.housing_score, housing_score);
        assert_eq!(snapshot.crime_score, crime_score);
        let expected_mood = (food_score
            + housing_score
            + crime_score
            + i64::from(snapshot.temple_bonus)
            + i64::from(snapshot.garrison_bonus))
        .clamp(MOOD_MIN, MOOD_MAX);
        assert_eq!(
            snapshot.mood, expected_mood,
            "tick {tick}: the mood frame must be the measured score, not a cached value"
        );
        assert_eq!(
            snapshot.mood_delta,
            expected_mood - previous_mood.unwrap_or(0),
            "tick {tick}: the series must carry the per-tick delta against the previous point"
        );
        previous_mood = Some(expected_mood);
        food_series.push(food_score);
        crime_series.push(crime_score);
    }
    assert!(
        live_frames >= 1,
        "fixture must exercise at least one live frame update"
    );
    assert!(
        food_series.windows(2).any(|window| window[0] != window[1])
            && crime_series.windows(2).any(|window| window[0] != window[1]),
        "the scripted inputs must move the series; a frozen series means the dashboard is dead"
    );

    // Reading a dashboard frame is a pure projection over the simulation.
    let before = serde_json::to_string(&sim.snapshot()).expect("serialize snapshot");
    let _ = compute_gameplay_state(&sim);
    let _ = sim.snapshot();
    assert_eq!(
        serde_json::to_string(&sim.snapshot()).expect("serialize snapshot"),
        before,
        "rendering a stats frame must not mutate simulated state"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-NOTIFY-911 — empire-scale aggregate + drill-down
// ---------------------------------------------------------------------------

/// Covers FR-CIV-NOTIFY-911.
///
/// The strategic stats view is a rollup of per-district drill-down rows; the two
/// must reconcile exactly, and narrowing the selection must reduce the aggregate
/// by exactly the removed region.
#[test]
fn notify_911_region_aggregate_reconciles_with_per_region_drill_down() {
    let districts: Vec<u32> = vec![125_000, 0, 4_096, 7, 999];
    let expected_total: u32 = districts
        .iter()
        .map(|population| u64::from(*population))
        .sum::<u64>() as u32;

    let hexes: Vec<HexCellSnapshot> = districts
        .iter()
        .map(|population| operational_hex_snapshot(*population, population / 2))
        .collect();
    let drill_down_total: u64 = hexes.iter().map(|hex| u64::from(hex.population)).sum();

    assert_eq!(
        u64::from(aggregate_strategic(&districts)),
        drill_down_total,
        "the region aggregate must equal the sum of the per-region drill-down rows"
    );
    assert_eq!(aggregate_strategic(&districts), expected_total);
    assert_eq!(aggregate_strategic(&[]), 0, "an empty view aggregates to zero");

    let mut narrowed = districts.clone();
    let removed = narrowed.pop().expect("fixture has a district to remove");
    assert_eq!(
        aggregate_strategic(&districts) - removed,
        aggregate_strategic(&narrowed),
        "removing one region must drop exactly its population from the aggregate"
    );
}

/// Covers FR-CIV-NOTIFY-911.
///
/// "Renders 100k+-agent aggregates without stalling": the rollup must stay exact
/// at multi-100k scale, and the live per-region frames must reconcile with the
/// aggregate view for every region.
#[test]
fn notify_911_empire_scale_aggregate_is_exact_and_drills_down_per_region() {
    // 120k districts — the aggregate must be exact and must not stall.
    let empire: Vec<u32> = (0..120_000u32).map(|district| district % 37).collect();
    let expected: u64 = empire.iter().map(|population| u64::from(*population)).sum();
    let started = Instant::now();
    let total = aggregate_strategic(&empire);
    let elapsed = started.elapsed();
    assert_eq!(
        u64::from(total),
        expected,
        "the 120k-region rollup must be exact"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "120k-region aggregate took {elapsed:?}, which would stall the stats view"
    );
    assert!(
        aggregate_strategic(&empire) >= aggregate_strategic(&empire[..1_000]),
        "the aggregate must be monotone in the selected regions"
    );

    // Live empire-scale drill-down: 64 regions, 128k inhabitants.
    let mut sim = Simulation::with_seed(0x911_E00);
    for settlement_id in 0..64u32 {
        sim.set_settlement_population(settlement_id, 2_000);
        sim.set_settlement_housing_capacity(settlement_id, 2_000);
    }
    sim.tick();

    let aggregate_view = sim.last_tick_mood_all();
    assert_eq!(
        aggregate_view.len(),
        64,
        "the aggregate stats view must cover every region at empire scale"
    );
    for settlement_id in 0..64u32 {
        let drill_down = sim
            .last_tick_mood(settlement_id)
            .expect("every region must expose a drill-down frame")
            .clone();
        assert_eq!(drill_down.settlement_id, settlement_id);
        let aggregate_entry = aggregate_view
            .iter()
            .find(|frame| frame.settlement_id == settlement_id)
            .expect("aggregate entry for the region");
        assert_eq!(
            aggregate_entry, &drill_down,
            "region {settlement_id}: drill-down must match the aggregate row"
        );
    }
    assert!(
        aggregate_view
            .windows(2)
            .all(|window| window[0].settlement_id < window[1].settlement_id),
        "the aggregate view must be stably ordered by region for chart keying"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-NOTIFY-920 — progressive-disclosure onboarding
// ---------------------------------------------------------------------------

fn all_milestones(progress: &TutorialProgress) -> bool {
    progress.faction_exists
        && progress.tech_unlocked
        && progress.war_declared
        && progress.religion_emerged
}

/// Independent re-derivation of the documented progression contract.
fn expected_milestone(progress: &TutorialProgress) -> TutorialMilestone {
    if all_milestones(progress) {
        return TutorialMilestone::Complete;
    }
    let mut milestone = TutorialMilestone::FirstFaction;
    if progress.tech_unlocked {
        milestone = milestone.max(TutorialMilestone::FirstTech);
    }
    if progress.war_declared {
        milestone = milestone.max(TutorialMilestone::FirstWar);
    }
    if progress.religion_emerged {
        milestone = milestone.max(TutorialMilestone::FirstReligion);
    }
    milestone
}

/// Covers FR-CIV-NOTIFY-920.
///
/// "First-run highlights core verbs": a brand-new session starts at the first
/// verb with nothing marked as already discovered, the milestone order is strict,
/// and the flow is readable from the client snapshot.
#[test]
fn notify_920_first_run_starts_at_first_verb_and_completion_is_gated() {
    let first_run = TutorialProgress::default();
    assert_eq!(first_run.current, TutorialMilestone::FirstFaction);
    assert!(!first_run.faction_exists);
    assert!(!first_run.tech_unlocked);
    assert!(!first_run.war_declared);
    assert!(!first_run.religion_emerged);
    assert!(
        !first_run.clone().completed(),
        "a fresh session is not complete"
    );

    assert!(
        TutorialMilestone::FirstFaction < TutorialMilestone::FirstTech
            && TutorialMilestone::FirstTech < TutorialMilestone::FirstWar
            && TutorialMilestone::FirstWar < TutorialMilestone::FirstReligion
            && TutorialMilestone::FirstReligion < TutorialMilestone::Complete,
        "milestone ordering must be a strict progressive chain"
    );

    let sim = Simulation::with_seed(0x920_0B0A);
    assert_eq!(
        sim.tutorial_progress, first_run,
        "a fresh sim must start on the first-run step"
    );
    assert_eq!(
        sim.snapshot().tutorial_progress,
        sim.tutorial_progress,
        "the first-run flow must be readable from the client snapshot"
    );
    assert!(!sim.snapshot().tutorial_progress.completed());
}

/// Covers FR-CIV-NOTIFY-920.
///
/// Progressive disclosure: each milestone flips only when its *measured* source
/// exists, `current` is the max of the satisfied milestones (never skipping
/// ahead, never rewinding), and the flow only completes once all four are
/// satisfied.
#[test]
fn notify_920_progressive_disclosure_gates_completion_on_measured_milestones() {
    let mut sim = Simulation::with_seed(0x920_B0A2D);
    assert!(
        sim.diplomacy_events().is_empty(),
        "fixture must start without a conflict event"
    );

    // Stage 0: strip every measured source the flow observes.
    sim.state.factions.clear();
    sim.era_progression.faction_tech.clear();
    sim.religious_profiles.clear();

    let mut progress = TutorialProgress::default();
    progress.advance_from_sim(&sim);
    assert!(
        !progress.faction_exists && !progress.tech_unlocked && !progress.religion_emerged,
        "no measured source may light a milestone, got {progress:?}"
    );
    assert!(!progress.war_declared);
    assert_eq!(progress.current, expected_milestone(&progress));
    assert_eq!(progress.clone().completed(), all_milestones(&progress));
    assert!(
        !progress.clone().completed(),
        "an empty world must not complete the first-run flow"
    );

    // Stage 1: a faction exists.
    sim.state.factions.insert(0, "Ardani".to_owned());
    progress.advance_from_sim(&sim);
    assert!(progress.faction_exists, "a placed faction must light the first verb");
    assert!(!progress.tech_unlocked && !progress.war_declared && !progress.religion_emerged);
    assert_eq!(progress.current, TutorialMilestone::FirstFaction);
    assert!(
        !progress.clone().completed(),
        "one of four milestones must not complete the first-run flow"
    );

    // Stage 2: an unlocked tech level.
    sim.era_progression.faction_tech.insert(
        0,
        FactionTechState {
            research_points: 0,
            tech_level: 1,
            diffusion_points: 0,
        },
    );
    progress.advance_from_sim(&sim);
    assert!(progress.tech_unlocked, "an unlocked tech level must light the tech step");
    assert!(!progress.war_declared && !progress.religion_emerged);
    assert_eq!(progress.current, TutorialMilestone::FirstTech);
    assert!(
        !progress.clone().completed(),
        "two of four milestones must not complete the first-run flow"
    );

    // Stage 3: an emergent religious profile.
    sim.religious_profiles
        .insert(0, ReligiousProfile::new(100, sim.state.tick));
    progress.advance_from_sim(&sim);
    assert!(
        progress.religion_emerged,
        "an emergent religious profile must light the religion step"
    );
    assert!(!progress.war_declared);
    assert_eq!(progress.current, TutorialMilestone::FirstReligion);
    assert!(
        !progress.clone().completed(),
        "three of four milestones must not complete the first-run flow"
    );

    // Stage 4: a real conflict event.
    sim.push_diplomacy_event(DiplomacyEvent {
        tick: sim.state.tick,
        faction_a: 0,
        faction_b: 1,
        kind: DiplomacyKind::Conflict,
    });
    progress.advance_from_sim(&sim);
    assert!(progress.war_declared, "a conflict event must light the war step");
    assert_eq!(progress.current, TutorialMilestone::Complete);
    assert!(
        progress.clone().completed(),
        "all four measured milestones must complete the first-run flow"
    );

    // Monotone + sticky over a longer live run: the highlighted step never
    // rewinds, a discovered milestone is never un-discovered, the highlighted step
    // always equals the max satisfied milestone, and the client snapshot mirrors
    // the engine state.
    let mut live = Simulation::with_seed(0x920_B0A2D);
    let mut previous = live.tutorial_progress.clone();
    for _ in 0..16 {
        live.tick();
        let current = live.tutorial_progress.clone();
        assert_eq!(
            current.current,
            expected_milestone(&current),
            "the highlighted step must equal the max satisfied milestone"
        );
        assert_eq!(
            current.clone().completed(),
            all_milestones(&current),
            "completion must require every milestone"
        );
        assert!(
            current.current >= previous.current,
            "the first-run step must never rewind ({:?} -> {:?})",
            previous.current,
            current.current
        );
        assert!(
            (!previous.faction_exists || current.faction_exists)
                && (!previous.tech_unlocked || current.tech_unlocked)
                && (!previous.war_declared || current.war_declared)
                && (!previous.religion_emerged || current.religion_emerged),
            "discovered milestones must stay discovered: {previous:?} -> {current:?}"
        );
        assert!(
            live.state.factions.is_empty() || current.faction_exists,
            "a live faction must be reflected in the flow"
        );
        assert!(
            live
                .era_progression
                .faction_tech
                .values()
                .all(|tech| tech.tech_level == 0)
                || current.tech_unlocked,
            "a live unlocked tech level must be reflected in the flow"
        );
        assert!(
            live.religious_profiles.is_empty() || current.religion_emerged,
            "a live religious profile must be reflected in the flow"
        );
        assert!(
            live.diplomacy_events()
                .iter()
                .all(|event| event.kind != DiplomacyKind::Conflict)
                || current.war_declared,
            "a live conflict must be reflected in the flow"
        );
        assert_eq!(live.snapshot().tutorial_progress, current);
        previous = current;
    }
}

/// Covers FR-CIV-NOTIFY-920.
///
/// "…and skippable": the first-run flow is advisory. It must never gate the
/// simulation clock, and the flow must stay observable while the sim runs.
#[test]
fn notify_920_onboarding_never_gates_the_simulation_clock() {
    let mut sim = Simulation::with_seed(0x920_5C1A);
    assert!(
        !sim.tutorial_progress.clone().completed(),
        "fixture must start with an incomplete first-run flow"
    );
    for expected_tick in 1..=5u64 {
        sim.tick();
        assert_eq!(
            sim.state.tick, expected_tick,
            "an incomplete first-run flow must not pause or gate the simulation clock"
        );
        assert_eq!(
            sim.snapshot().tutorial_progress,
            sim.tutorial_progress,
            "the flow must stay observable while the sim runs"
        );
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-NOTIFY-921 — rebindable hotkey action contract
// ---------------------------------------------------------------------------

/// Covers FR-CIV-NOTIFY-921.
///
/// The control half of the hotkey map (speed, pause/resume, replay, policy) is
/// bound to `CommandKind` actions. Each binding must dispatch with its payload
/// intact (a rebind round-trip), invalid bindings must be rejected rather than
/// silently no-op, dispatch must stay FIFO, and the pending queue must be bounded.
#[test]
fn notify_921_control_actions_round_trip_with_intact_payloads() {
    let mut queue = CommandQueue::new(8);
    let bindings: Vec<(&str, CommandKind)> = vec![
        ("pause", CommandKind::Pause),
        ("resume", CommandKind::Resume),
        ("speed:1", CommandKind::SetSpeed(1)),
        ("speed:8", CommandKind::SetSpeed(8)),
        ("save_replay", CommandKind::SaveReplay),
        (
            "load_replay",
            CommandKind::LoadReplay("saves/rebound.civreplay".to_owned()),
        ),
        (
            "policy_override",
            CommandKind::PolicyOverride {
                key: "scarcity".to_owned(),
                value: 1.75,
            },
        ),
    ];

    for (seq, (_, kind)) in bindings.into_iter().enumerate() {
        queue
            .push(Command {
                client_id: 7,
                seq: seq as u64,
                kind,
                tick_issued: 42,
            })
            .expect("a valid binding must dispatch");
    }
    assert_eq!(queue.len(), 7, "every binding must be queued exactly once");

    // FIFO: the action dispatched is the action bound, with exact payloads.
    let mut seen = Vec::new();
    while let Some(command) = queue.pop() {
        assert_eq!(command.client_id, 7);
        assert_eq!(command.tick_issued, 42);
        seen.push(command.seq);
        match command.kind {
            CommandKind::Pause => seen.push(100),
            CommandKind::Resume => seen.push(101),
            CommandKind::SetSpeed(speed) => {
                assert!(speed > 0, "a dispatched speed binding must carry a valid speed");
                seen.push(200 + u64::from(speed));
            }
            CommandKind::SaveReplay => seen.push(300),
            CommandKind::LoadReplay(path) => {
                assert_eq!(path, "saves/rebound.civreplay", "payload must survive dispatch");
                seen.push(301);
            }
            CommandKind::PolicyOverride { key, value } => {
                assert_eq!(key, "scarcity");
                assert!(
                    (value - 1.75).abs() < f64::EPSILON,
                    "value must survive dispatch"
                );
                seen.push(400);
            }
        }
    }
    assert_eq!(
        seen,
        vec![0, 100, 1, 101, 2, 201, 3, 208, 4, 300, 5, 301, 6, 400],
        "bindings must dispatch in FIFO order with their own payloads"
    );
    assert!(queue.is_empty());

    // Validation: a speed binding with an unusable value is rejected instead of
    // being queued as a silent no-op.
    let mut queue = CommandQueue::new(4);
    let rejected = queue.push(Command {
        client_id: 1,
        seq: 0,
        kind: CommandKind::SetSpeed(0),
        tick_issued: 0,
    });
    assert!(
        matches!(rejected, Err(CommandError::InvalidSpeed)),
        "speed 0 must be rejected, got {rejected:?}"
    );
    assert_eq!(queue.len(), 0, "a rejected binding must not be queued");

    // Capacity is bounded so a stuck key cannot grow the queue without limit.
    let mut queue = CommandQueue::new(3);
    for seq in 0..3u64 {
        queue
            .push(Command {
                client_id: 1,
                seq,
                kind: CommandKind::Pause,
                tick_issued: 0,
            })
            .expect("within capacity");
    }
    let overflow = queue.push(Command {
        client_id: 1,
        seq: 3,
        kind: CommandKind::Pause,
        tick_issued: 0,
    });
    assert!(
        matches!(overflow, Err(CommandError::CapacityExceeded)),
        "the queue must bound pending dispatches, got {overflow:?}"
    );
    assert_eq!(queue.len(), 3);
}

fn tool_center() -> WorldCoord {
    WorldCoord { x: 0, y: 0, z: 0 }
}

fn tool_catalog() -> Vec<GodToolRequest> {
    let mut catalog: Vec<GodToolRequest> = Vec::new();
    for op in [
        TerraformOp::Raise,
        TerraformOp::Lower,
        TerraformOp::Level,
        TerraformOp::Smooth,
        TerraformOp::RaiseMountain,
        TerraformOp::Slope,
        TerraformOp::AddLand,
        TerraformOp::DigOcean,
        TerraformOp::DropBiome,
        TerraformOp::Flatten,
    ] {
        catalog.push(GodToolRequest::Terraform(TerraformRequest {
            op,
            center: tool_center(),
            radius_voxels: 2,
            strength: 4,
            aux_id: 3,
        }));
    }
    for op in [
        MaterialOp::Erase,
        MaterialOp::Replace,
        MaterialOp::SurfacePaint,
        MaterialOp::AdditiveDrop,
        MaterialOp::PourLiquid,
        MaterialOp::SeedSnow,
        MaterialOp::SeedOreDeposit,
        MaterialOp::SeedForest,
    ] {
        catalog.push(GodToolRequest::Material(MaterialRequest {
            op,
            center: tool_center(),
            radius_voxels: 2,
            material_id: 3,
            strength: 1,
            drop_height: 2,
        }));
    }
    for request in [
        LifeRequest::SpawnHerd(SpawnHerdRequest {
            count: 4,
            seed_civilian_id: 900,
            faction: 0,
        }),
        LifeRequest::Extinct(ActorFootprintRequest {
            center: tool_center(),
            radius_voxels: 2,
        }),
    ] {
        catalog.push(GodToolRequest::Life(request));
    }
    for request in [
        DisasterRequest::Meteor { pos: tool_center() },
        DisasterRequest::Wildfire { pos: tool_center() },
        DisasterRequest::Flood { pos: tool_center() },
        DisasterRequest::Quake { pos: tool_center() },
        DisasterRequest::Storm { pos: tool_center() },
        DisasterRequest::Plague { pos: tool_center() },
        DisasterRequest::Lightning {
            from: tool_center(),
            to: WorldCoord { x: 2, y: 0, z: 0 },
        },
        DisasterRequest::Tornado {
            pos: tool_center(),
            radius_voxels: 2,
        },
        DisasterRequest::VolcanicVent {
            pos: tool_center(),
            ticks: 3,
        },
        DisasterRequest::Drought {
            pos: tool_center(),
            reduction_pct: 20,
            ticks: 3,
        },
    ] {
        catalog.push(GodToolRequest::Disaster(request));
    }
    catalog.push(GodToolRequest::Inspect(InspectRequest::Probe(ProbeRequest {
        pos: tool_center(),
    })));
    catalog.push(GodToolRequest::Law(LawRequest::TaxBias {
        target_faction: 0,
        bias: 1_000,
    }));
    catalog.push(GodToolRequest::Law(LawRequest::ReligionPressure {
        pressure: 10,
    }));
    catalog.push(GodToolRequest::Law(LawRequest::DifficultyKnob {
        scarcity_multiplier: 1.5,
    }));
    catalog
}

/// Covers FR-CIV-NOTIFY-921.
///
/// The tools half of the hotkey map binds to god-tool actions. Every action in
/// the catalog must be individually nameable (a distinct serialized form, so two
/// bindings can never alias), persistable (so key→action bindings can be saved
/// and rebound), validated (bad payloads error instead of corrupting state), and
/// deterministic (the same binding sequence produces identical receipts).
#[test]
fn notify_921_tool_action_catalog_is_distinct_serialisable_and_validated() {
    let catalog = tool_catalog();
    assert!(catalog.len() >= 25, "the catalog must cover every tool family");

    // Every bindable action must be individually nameable and persistable.
    let mut encoded: Vec<String> = Vec::new();
    for request in &catalog {
        let json = serde_json::to_string(request).expect("tool action must serialise");
        assert!(
            !encoded.contains(&json),
            "two distinct tool actions share the same wire form, so bindings would alias: {json}"
        );
        encoded.push(json);
    }
    // `Terraform`/`Material` wrap plain structs, so the wrapper itself round-trips.
    for request in catalog.iter().filter(|request| {
        matches!(
            request,
            GodToolRequest::Terraform(_) | GodToolRequest::Material(_)
        )
    }) {
        let json = serde_json::to_string(request).expect("serialise");
        let decoded: GodToolRequest = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(
            &decoded, request,
            "a struct-payload tool action must survive a rebind round-trip"
        );
    }
    // The inner-tagged families round-trip as payloads in their own right; see the
    // file header for the wrapper-level `duplicate field \`kind\`` defect that
    // currently prevents round-tripping them through `GodToolRequest`.
    for request in &catalog {
        match request {
            GodToolRequest::Life(inner) => {
                let json = serde_json::to_string(inner).expect("serialise life payload");
                assert_eq!(
                    &serde_json::from_str::<LifeRequest>(&json).expect("deserialise life payload"),
                    inner
                );
            }
            GodToolRequest::Disaster(inner) => {
                let json = serde_json::to_string(inner).expect("serialise disaster payload");
                assert_eq!(
                    &serde_json::from_str::<DisasterRequest>(&json)
                        .expect("deserialise disaster payload"),
                    inner
                );
            }
            GodToolRequest::Inspect(inner) => {
                let json = serde_json::to_string(inner).expect("serialise inspect payload");
                assert_eq!(
                    &serde_json::from_str::<InspectRequest>(&json)
                        .expect("deserialise inspect payload"),
                    inner
                );
            }
            GodToolRequest::Law(inner) => {
                let json = serde_json::to_string(inner).expect("serialise law payload");
                assert_eq!(
                    &serde_json::from_str::<LawRequest>(&json).expect("deserialise law payload"),
                    inner
                );
            }
            GodToolRequest::Terraform(_) | GodToolRequest::Material(_) => {}
        }
    }

    // Deterministic + non-panicking when dispatched: identical sessions must
    // observe identical receipts, so rebinding a key cannot change the outcome.
    let mut first = Simulation::with_seed(0x921_A7A1);
    let mut second = Simulation::with_seed(0x921_A7A1);
    let receipts_first: Vec<Result<GodToolReceipt, GodToolError>> = catalog
        .clone()
        .into_iter()
        .map(|request| first.apply_god_tool(request))
        .collect();
    let receipts_second: Vec<Result<GodToolReceipt, GodToolError>> = catalog
        .clone()
        .into_iter()
        .map(|request| second.apply_god_tool(request))
        .collect();
    assert_eq!(
        receipts_first, receipts_second,
        "the same binding sequence on the same seed must produce identical receipts"
    );
    for (request, outcome) in catalog.iter().zip(receipts_first.iter()) {
        match outcome {
            Ok(_) => {}
            Err(GodToolError::NotImplemented { verb }) => {
                assert!(!verb.is_empty(), "a near-verb must name itself");
            }
            Err(GodToolError::InvalidRequest(message)) => {
                assert!(
                    !message.is_empty(),
                    "a rejected {request:?} must explain why it was rejected"
                );
            }
            Err(GodToolError::ReadOnly) => {
                panic!("{request:?} was mis-routed as a mutating verb")
            }
        }
    }

    // Spot-check the documented contract of representative actions in each family.
    let mut sim = Simulation::with_seed(0x921_A7A1);
    let raise = sim
        .apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Raise,
            center: tool_center(),
            radius_voxels: 2,
            strength: 4,
            aux_id: 0,
        }))
        .expect("terrain.raise must apply");
    assert!(
        matches!(raise, GodToolReceipt::Terraform { writes, .. } if writes > 0),
        "terrain.raise must report the voxel writes it stamped, got {raise:?}"
    );

    let herd = sim
        .apply_god_tool(GodToolRequest::Life(LifeRequest::SpawnHerd(
            SpawnHerdRequest {
                count: 4,
                seed_civilian_id: 900,
                faction: 0,
            },
        )))
        .expect("life.spawn_herd must apply");
    assert!(
        matches!(herd, GodToolReceipt::Life { affected_count: 4, .. }),
        "life.spawn_herd must report the agents it injected, got {herd:?}"
    );

    let probe = sim
        .apply_god_tool(GodToolRequest::Inspect(InspectRequest::Probe(
            ProbeRequest { pos: tool_center() },
        )))
        .expect("inspect.probe is read-only and must always apply");
    assert!(
        matches!(&probe, GodToolReceipt::Inspect { report } if report.pos == tool_center()),
        "inspect.probe must report the probed coordinate, got {probe:?}"
    );

    let knob = sim
        .apply_god_tool(GodToolRequest::Law(LawRequest::DifficultyKnob {
            scarcity_multiplier: 1.5,
        }))
        .expect("law.difficulty_knob must apply in range");
    assert!(
        matches!(&knob, GodToolReceipt::Law { verb, .. } if verb == "law.difficulty_knob"),
        "law.difficulty_knob must name its verb, got {knob:?}"
    );
    assert!(
        (sim.economy_policy.scarcity_multiplier - 1.5).abs() < 1e-9,
        "the difficulty knob must actually move the measured economy scalar"
    );

    // Invalid payloads are rejected with a specific error, never silently applied.
    let invalid_radius = sim.apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
        op: TerraformOp::Raise,
        center: tool_center(),
        radius_voxels: 0,
        strength: 4,
        aux_id: 0,
    }));
    assert!(
        matches!(&invalid_radius, Err(GodToolError::InvalidRequest(message)) if message.contains("radius")),
        "a zero-radius brush must be rejected, got {invalid_radius:?}"
    );

    let invalid_strength = sim.apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
        op: TerraformOp::Level,
        center: tool_center(),
        radius_voxels: 2,
        strength: -1,
        aux_id: 0,
    }));
    assert!(
        matches!(&invalid_strength, Err(GodToolError::InvalidRequest(message)) if message.contains("strength")),
        "a negative level target must be rejected, got {invalid_strength:?}"
    );

    let invalid_biome = sim.apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
        op: TerraformOp::DropBiome,
        center: tool_center(),
        radius_voxels: 2,
        strength: 0,
        aux_id: 0,
    }));
    assert!(
        matches!(&invalid_biome, Err(GodToolError::InvalidRequest(message)) if message.contains("aux_id")),
        "a missing biome material must be rejected, got {invalid_biome:?}"
    );

    let invalid_knob = sim.apply_god_tool(GodToolRequest::Law(LawRequest::DifficultyKnob {
        scarcity_multiplier: 99.0,
    }));
    assert!(
        matches!(&invalid_knob, Err(GodToolError::InvalidRequest(_))),
        "an out-of-range difficulty knob must be rejected, got {invalid_knob:?}"
    );
}
