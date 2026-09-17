//! FR-CORE-010 — integer quantities in world state SHALL use fixed-point /
//! integer representations (`FixedI32<U16>`, `i64` KiloJoules, `i64`
//! MilliCredits).
//!
//! Matrix check: `numerics::integer_quantities_use_fixed_point`.
//!
//! Two notes on scope:
//!
//! - The requirement names `FixedI32<U16>`; this codebase ships its own
//!   `civ_engine::Fixed(i64)` instead. See
//!   `docs/adr/ADR-022-runtime-representation-deviations.md`.
//! - The requirement governs *integer quantities* (Joules, credits). It is not
//!   a blanket ban on floats: `ADR-determinism-dropped.md`
//!   explicitly permits floating-point where it serves emergence, and the
//!   culture / language / psyche subsystems do store `f32` state. Those are out
//!   of scope here and are asserted separately below so the boundary is
//!   explicit rather than accidental.

use civ_engine::{Fixed, Simulation};
use serde_json::Value;

/// True if a JSON number required a floating-point representation.
fn is_float(n: &Value) -> bool {
    matches!(n, Value::Number(num) if num.as_i64().is_none() && num.as_u64().is_none())
}

/// The designated energy and money quantities are integer-backed.
#[test]
fn integer_quantities_use_fixed_point() {
    let mut sim = Simulation::with_seed(0xC0FFEEu64);
    for _ in 0..10 {
        sim.tick();
    }
    let json = serde_json::to_value(&sim.state).expect("WorldState serialises");

    // Energy budget: `Fixed` must land as an integer node, never a float.
    let joules = &json["energy_budget_joules"];
    assert!(
        !is_float(joules),
        "energy_budget_joules must be integer-backed, got {joules}"
    );
    assert!(joules.is_u64() || joules.is_i64());

    // Faction treasuries (credits) are integers too.
    let treasury = json["faction_treasury"]
        .as_object()
        .expect("faction_treasury is a map");
    assert!(!treasury.is_empty(), "the sim has funded factions");
    for (faction, balance) in treasury {
        assert!(
            !is_float(balance),
            "faction {faction} treasury must be integer-backed, got {balance}"
        );
    }

    // Population and tick counters are plain integers.
    assert!(!is_float(&json["population"]));
    assert!(!is_float(&json["tick"]));
}

/// `Fixed` is integer-backed, exact at its scale, and round-trips losslessly.
#[test]
fn fixed_is_integer_backed() {
    let a = Fixed::from_num(3);
    let b = Fixed::from_num(4);
    assert_eq!(a * b, Fixed::from_num(12), "fixed-point multiply is exact here");
    assert!(a < b);

    let encoded = serde_json::to_value(a).expect("Fixed serialises");
    assert!(
        !is_float(&encoded),
        "Fixed must serialise to an integer representation, got {encoded}"
    );
    let decoded: Fixed = serde_json::from_value(encoded).expect("Fixed deserialises");
    assert_eq!(decoded, a);

    // Ordering and addition hold at scale, without float drift.
    let mut acc = Fixed::from_num(0);
    for _ in 0..100 {
        acc += Fixed::from_num(1);
    }
    assert_eq!(acc, Fixed::from_num(100));
}

/// Floats are permitted only in the emergence-oriented subsystems, and the
/// energy/credit fields above must stay integer-backed.
///
/// This pins the boundary so a future change that leaks a float into the
/// economy state is caught, while not failing on sanctioned emergence state.
#[test]
fn float_boundary_is_economy_clean() {
    let mut sim = Simulation::with_seed(11u64);
    for _ in 0..5 {
        sim.tick();
    }
    let json = serde_json::to_value(&sim.state).expect("WorldState serialises");

    // Economy-shaped subtrees must be float-free.
    for key in ["faction_treasury", "faction_resources", "energy_budget_joules"] {
        let value = &json[key];
        assert!(
            !contains_float(value),
            "economy field `{key}` must not contain floats, got {value}"
        );
    }
}

fn contains_float(v: &Value) -> bool {
    match v {
        Value::Number(_) => is_float(v),
        Value::Array(items) => items.iter().any(contains_float),
        Value::Object(map) => map.values().any(contains_float),
        Value::Null | Value::Bool(_) | Value::String(_) => false,
    }
}
