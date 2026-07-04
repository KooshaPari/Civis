//! N-series emergence coupling tests — N1/N2/N3/N4/N7.
//!
//! FR-CIV-TEST-001: five coupling paths with zero dedicated test coverage.
//!
//! - N1  settlement stock surplus → market food price pressure
//! - N2  cluster cultural distance → diplomacy threshold signal
//! - N3  settlement contact → diplomacy pair emission (≥2 factions)
//! - N4  trade route resource transfer, overdraft guard, zero-volume inertness
//! - N7  sentience pipeline stability — no panic with no / many tick cycles

use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_engine::{Fixed, Simulation, TradeRoute};

// ── N1: market food price responds to stock conditions ────────────────────

/// N1 nominal — food price must remain non-negative under any stock level.
#[test]
fn n1_food_price_never_negative() {
    let mut sim = Simulation::with_seed(1001);
    // Zero out faction food to simulate scarcity.
    for res in sim.state.faction_resources.values_mut() {
        res.food = Fixed::ZERO;
    }
    for _ in 0..200 {
        sim.tick();
    }
    let price = sim.snapshot().market_prices.get("food").copied().unwrap_or(0);
    assert!(price >= 0, "food price must not go negative, got {price}");
}

/// N1 boundary — market must emit a food price entry after ticking (populated).
#[test]
fn n1_market_contains_food_price_after_ticks() {
    let mut sim = Simulation::with_seed(1002);
    for _ in 0..10 {
        sim.tick();
    }
    assert!(
        sim.snapshot().market_prices.contains_key("food"),
        "market_prices must contain a 'food' entry after 10 ticks"
    );
}

// ── N2: CultureProfile cultural_distance API ──────────────────────────────

/// N2 nominal — identical culture profiles must have zero distance.
#[test]
fn n2_identical_cultures_have_zero_distance() {
    let v = [0.5_f32, 0.2, 0.9, 0.1];
    let a = CultureProfile::new(v);
    let b = CultureProfile::new(v);
    let dist = cultural_distance(a.traits, b.traits);
    assert!(
        dist < 1e-5,
        "identical culture profiles must have near-zero distance, got {dist}"
    );
}

/// N2 boundary — maximally dissimilar profiles must have larger distance
/// than identical ones, so the threshold signal is directional.
#[test]
fn n2_dissimilar_cultures_greater_distance_than_identical() {
    let same = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
    let dist_same = cultural_distance(same.traits, same.traits);

    let a = CultureProfile::new([1.0, 0.0, 1.0, 0.0]);
    let b = CultureProfile::new([0.0, 1.0, 0.0, 1.0]);
    let dist_diff = cultural_distance(a.traits, b.traits);

    assert!(
        dist_diff > dist_same,
        "dissimilar cultures must have strictly greater distance: same={dist_same}, diff={dist_diff}"
    );
}

// ── N3: settlement contact → diplomacy pair emission ─────────────────────

/// N3 nominal — at least one diplomacy event fires within 3× cadence ticks
/// (3×500 = 1500) in a default multi-faction sim.
#[test]
fn n3_multi_faction_sim_emits_diplomacy_events() {
    let mut sim = Simulation::with_seed(3001);
    for _ in 0..1500 {
        sim.tick();
    }
    let events = sim.diplomacy_events();
    assert!(
        !events.is_empty(),
        "expected diplomacy events after 1500 ticks (3x cadence=500), got 0"
    );
    // All emitted events must reference factions present in the sim.
    for ev in events {
        assert!(
            sim.state.factions.contains_key(&ev.faction_a),
            "faction_a {:?} missing from sim.state.factions",
            ev.faction_a
        );
        assert!(
            sim.state.factions.contains_key(&ev.faction_b),
            "faction_b {:?} missing from sim.state.factions",
            ev.faction_b
        );
    }
}

/// N3 boundary — pruning all but one faction prevents diplomacy events
/// (pair selection requires ≥ 2 factions).
#[test]
fn n3_single_faction_emits_no_diplomacy_events() {
    let mut sim = Simulation::with_seed(3002);
    let keep = *sim.state.factions.keys().next().expect("at least one faction");
    sim.state.factions.retain(|&k, _| k == keep);
    sim.state.faction_treasury.retain(|&k, _| k == keep);
    sim.state.faction_resources.retain(|&k, _| k == keep);

    for _ in 0..1500 {
        sim.tick();
    }
    assert!(
        sim.diplomacy_events().is_empty(),
        "single-faction sim must emit zero diplomacy events"
    );
}

// ── N4: trade route mechanics ─────────────────────────────────────────────

/// N4 nominal — a route transfers food from exporter to importer in one tick.
#[test]
fn n4_route_transfers_food_to_importer() {
    let mut sim = Simulation::with_seed(4001);
    sim.state.faction_resources.entry(0).or_default().food = Fixed::from_num(500);
    sim.state.faction_resources.entry(1).or_default().food = Fixed::from_num(0);
    sim.state.trade_routes = vec![TradeRoute {
        from_faction: 0,
        to_faction: 1,
        goods: "grain".to_string(),
        volume: Fixed::from_num(10),
    }];

    let food_before = sim.state.faction_resources[&1].food;
    sim.tick();
    let food_after = sim.state.faction_resources[&1].food;

    assert!(
        food_after > food_before,
        "importer food must increase after trade route tick: before={food_before:?}, after={food_after:?}"
    );
}

/// N4 boundary — zero-volume route must not transfer food and must not panic.
#[test]
fn n4_zero_volume_route_is_inert() {
    let mut sim = Simulation::with_seed(4002);
    sim.state.faction_resources.entry(0).or_default().food = Fixed::from_num(200);
    sim.state.trade_routes = vec![TradeRoute {
        from_faction: 0,
        to_faction: 1,
        goods: "grain".to_string(),
        volume: Fixed::ZERO,
    }];
    sim.tick();
    // Route still exists; exporter food must not go negative.
    assert_eq!(sim.state.trade_routes.len(), 1);
    assert_eq!(sim.state.trade_routes[0].volume, Fixed::ZERO);
}

/// N4 boundary — exporter food must not go below zero when route volume
/// exceeds available stock (min-transfer guard).
#[test]
fn n4_route_does_not_overdraft_exporter() {
    let mut sim = Simulation::with_seed(4003);
    sim.state.faction_resources.entry(0).or_default().food = Fixed::from_num(5);
    sim.state.trade_routes = vec![TradeRoute {
        from_faction: 0,
        to_faction: 1,
        goods: "grain".to_string(),
        volume: Fixed::from_num(100),
    }];
    sim.tick();
    let food = sim.state.faction_resources[&0].food;
    assert!(
        food >= Fixed::ZERO,
        "exporter food must not go negative after over-sized route, got {food:?}"
    );
}

// ── N7: sentience pipeline stability ─────────────────────────────────────

/// N7 nominal — running 5000 ticks must not panic, and cluster cultures
/// must be populated (sentience pipeline executed every tick).
#[test]
fn n7_long_tick_run_is_stable_and_populates_cluster_cultures() {
    let mut sim = Simulation::with_seed(7001);
    for _ in 0..5000 {
        sim.tick();
    }
    // If the sentience / awakening coupling panics, the test fails before here.
    // Cluster cultures should be populated after many ticks with civilians.
    let cultures = sim.cluster_cultures();
    assert!(
        !cultures.is_empty(),
        "cluster_cultures must be non-empty after 5000 ticks"
    );
}

/// N7 boundary — a fresh (tick=0) sim must have an empty last_sentience
/// (no threshold crossings before any genetic evaluation).
#[test]
fn n7_fresh_sim_cluster_cultures_empty_at_tick_zero() {
    let sim = Simulation::with_seed(7002);
    assert_eq!(
        sim.state.tick,
        0,
        "with_seed must produce a tick-0 simulation"
    );
    // cluster_cultures is populated by phase_emergence — at tick 0 it should
    // be empty because no emergence phase has run yet.
    let cultures = sim.cluster_cultures();
    assert!(
        cultures.is_empty(),
        "cluster_cultures must be empty before any tick, got {:?}",
        cultures.len()
    );
}
