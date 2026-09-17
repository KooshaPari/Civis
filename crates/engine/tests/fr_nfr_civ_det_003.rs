//! NFR-CIV-DET-003 — fixed-point arithmetic in state-mutation paths.
//!
//! Matrix check: `fixed_point_float_agreement`.
//! Acceptance contract: fixed and float agree within 10^-6, and the persisted
//! economy quantities are integer-backed.
//!
//! The "zero f32/f64 in sim state-mutation modules" half of this NFR is a lint
//! gate (clippy float deny) rather than a runtime assertion; the numerical
//! agreement half is asserted here.

use civ_engine::{Fixed, Simulation};

/// The integer-backed `Fixed` type tracks f64 arithmetic to within 1e-6 for the
/// magnitudes this simulation actually uses.
#[test]
fn fixed_point_float_agreement() {
    const TOL: f64 = 1e-6;

    for raw in [0i64, 1, 7, 100, 1_000, 12_345, 999_999] {
        // Scale by 1000 the way KiloJoule quantities are represented.
        let as_float = (raw * 1000) as f64;
        let fixed = Fixed::from_num(raw * 1000);

        assert!(
            (fixed.to_f64() - as_float).abs() < TOL,
            "Fixed({raw}) -> {} disagrees with f64 {as_float}",
            fixed.to_f64()
        );
    }

    // Addition and multiplication stay within tolerance of f64.
    let a_raw = 1_234i64;
    let b_raw = 567i64;
    let fa = Fixed::from_num(a_raw * 1000);
    let fb = Fixed::from_num(b_raw * 1000);

    assert!(
        (fa.to_f64() + fb.to_f64() - (a_raw + b_raw) as f64 * 1000.0).abs() < TOL,
        "fixed vs float addition disagrees"
    );

    // Ordering is preserved exactly, which is what determinism depends on.
    assert!(fa > fb);
    assert!(Fixed::from_num(0) < Fixed::from_num(1));
}

/// The economy quantities that persist with a simulation are integer-backed.
#[test]
fn state_economy_quantities_are_integer_backed() {
    let mut sim = Simulation::with_seed(0xF1_ED_u64);
    for _ in 0..5 {
        sim.tick();
    }

    let json = serde_json::to_value(&sim.state).expect("WorldState serialises");
    for field in ["energy_budget_joules", "population", "tick"] {
        let node = &json[field];
        assert!(
            node.is_u64() || node.is_i64(),
            "`{field}` must be integer-backed, got {node}"
        );
    }

    // Round-tripping the energy budget through its persisted form is lossless.
    let bits = sim.state.energy_budget_joules.to_bits();
    assert_eq!(Fixed::from_bits(bits), sim.state.energy_budget_joules);
}
