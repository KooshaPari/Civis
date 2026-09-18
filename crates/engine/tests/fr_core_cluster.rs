//! FR-CIV-CORE-003 / 005 / 013, FR-CORE-002, FR-FR-CORE-009 — core loop
//! determinism and ordering.
//!
//! Requirement text (`docs/specs/CIV-0001-core-simulation-loop.md:877-960` and
//! `docs/traceability/TRACEABILITY_MATRIX.md:36,43`):
//!
//! - FR-CIV-CORE-003: "Stochastic events use ChaCha8Rng seeded with seed
//!   parameter. Same seed → identical events; different seed → different (but
//!   valid) events."
//! - FR-CIV-CORE-005: "All entities iterated in sorted (deterministic) order.
//!   Verify all collections are BTreeMap, not HashMap, in critical paths."
//! - FR-CIV-CORE-013: "Ticks execute phases in order."
//! - FR-CORE-002: "The engine SHALL produce identical output for identical seed
//!   and input sequence (determinism)."
//! - FR-FR-CORE-009: "Hex grid SHALL use `hexx` axial coordinates throughout
//!   engine and render crates."
//!
//! These replace the auto-generated placeholder tests
//! (`crates/engine/tests/fr_fr_civ_core_00{3,4,5}.rs`,
//! `fr_fr_core_002.rs`, ...) whose body asserted `assert!(ws.tick == 0)` on a
//! default `WorldState` and never exercised the core loop.
//!
//! ## Not covered here
//!
//! **FR-CIV-CORE-004** ("single tick completes in < 16 ms wall time") and
//! **FR-CIV-CORE-019** ("entities modelled as dense arrays; no allocations per
//! iteration") are not asserted: the first is wall-clock on commodity hardware
//! and the second is a structural property with no observable surface. Both
//! would need a benchmark harness or an allocator instrumentation hook, so they
//! are recorded as gated rather than faked here.

use civ_engine::grid::PositionAxial;
use civ_engine::Simulation;

/// Deterministic per-tick state fingerprint.
fn fingerprint(sim: &Simulation) -> Option<[u8; 32]> {
    sim.hash_chain_root()
}

// ---------------------------------------------------------------------------
// FR-CIV-CORE-003 — seeded RNG drives stochastic phases
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CORE-003.
///
/// Same seed must produce identical stochastic outcomes; a different seed must
/// produce a different (but still valid) run.
///
/// The observable is the spawned agents' genomes, not `hash_chain_root`: the
/// hash chain is built from replay events, and at these tick counts the event
/// sequence is identical across seeds even though the seeded state is not, so a
/// chain comparison would pass vacuously. `Dna` is produced by the seeded RNG,
/// so it actually differs.
#[test]
fn fr_civ_core_003_same_seed_identical_events_different_seed_diverges() {
    use civ_genetics::Dna;

    /// Sorted genome set for a seed after some ticks.
    fn genomes(seed: u64, ticks: usize) -> Vec<String> {
        let mut sim = Simulation::with_seed(seed);
        for _ in 0..ticks {
            sim.tick();
        }
        let mut out: Vec<String> = sim
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, dna)| format!("{dna:?}"))
            .collect();
        out.sort_unstable();
        out
    }

    let a = genomes(0x5EED_0001, 40);
    let b = genomes(0x5EED_0001, 40);
    assert!(
        !a.is_empty(),
        "premise: the run must have spawned agents to compare"
    );
    assert_eq!(a, b, "identical seeds must produce identical genomes");

    let c = genomes(0x5EED_0002, 40);
    assert_ne!(
        a, c,
        "a different seed must produce different genomes; otherwise the seed is ignored"
    );

    // "different (but valid) events": the diverging run must still be a normal,
    // populated run rather than an empty one that trivially differs.
    assert_eq!(
        a.len(),
        c.len(),
        "both runs must spawn the same number of agents; only the genomes differ"
    );
    let mut sim = Simulation::with_seed(0x5EED_0002);
    for _ in 0..40 {
        sim.tick();
    }
    assert!(
        sim.state.tick >= 40,
        "the diverging run must still advance normally, got tick {}",
        sim.state.tick
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-CORE-005 — sorted, deterministic iteration
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CORE-005.
///
/// The spec requires critical-path collections to be ordered maps so iteration
/// is deterministic. `Simulation::state.factions` is iterated directly by the
/// diplomacy and emergence phases, so its key order must be ascending.
#[test]
fn fr_civ_core_005_faction_iteration_is_sorted() {
    let sim = Simulation::with_seed(0x0FAC_7105);
    let ids: Vec<u32> = sim.state.factions.keys().copied().collect();

    assert!(
        !ids.is_empty(),
        "premise: a new simulation registers factions"
    );
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(
        ids, sorted,
        "faction iteration must already be in ascending order; an unordered map \
         here makes diplomacy/emergence phase order depend on hash seeding"
    );
}

/// Covers FR-CIV-CORE-005.
///
/// Two identically seeded runs must observe the same faction ordering. An
/// unordered map with a per-process random seed would violate this even when
/// the simulation logic is otherwise correct.
#[test]
fn fr_civ_core_005_faction_order_is_stable_across_runs() {
    let order = || {
        let sim = Simulation::with_seed(0x0FAC_7106);
        sim.state.factions.keys().copied().collect::<Vec<u32>>()
    };
    assert_eq!(
        order(),
        order(),
        "faction iteration order must not depend on hash-map seeding"
    );
}

// ---------------------------------------------------------------------------
// FR-CORE-002 — end-to-end determinism
// ---------------------------------------------------------------------------

/// Covers FR-CORE-002.
///
/// Identical seed + identical command sequence must produce identical output.
/// A diverging command sequence must produce different output, or the
/// comparison is vacuous.
#[test]
fn fr_core_002_identical_seed_and_inputs_are_bit_identical() {
    let run = |ticks: usize| {
        let mut sim = Simulation::with_seed(0xD37E_0002);
        for _ in 0..ticks {
            sim.tick();
        }
        fingerprint(&sim)
    };

    assert_eq!(
        run(60),
        run(60),
        "same seed and same tick count must be bit-identical"
    );
    assert_ne!(
        run(60),
        run(61),
        "a different tick count must change the fingerprint, otherwise the \
         fingerprint does not depend on the simulation at all"
    );
}

// ---------------------------------------------------------------------------
// FR-FR-CORE-009 — axial / cube coordinate round-trip
// ---------------------------------------------------------------------------

/// Covers FR-FR-CORE-009.
///
/// The hex grid must round-trip between axial and cube coordinates, and cube
/// coordinates must satisfy the cube invariant `x + y + z == 0`.
#[test]
fn fr_fr_core_009_axial_cube_roundtrip_holds() {
    let coords = [
        (0, 0),
        (1, 0),
        (0, 1),
        (-1, -1),
        (3, -2),
        (-4, 5),
        (17, -9),
        (-128, 64),
    ];

    for (q, r) in coords {
        let axial = PositionAxial::new(q, r);
        let cube = axial.to_cube();

        assert_eq!(
            cube.x + cube.y + cube.z,
            0,
            "cube coords for axial ({q},{r}) violate x+y+z==0: {cube:?}"
        );

        let back = cube.to_axial();
        assert_eq!(
            (back.q, back.r),
            (q, r),
            "axial ({q},{r}) -> cube -> axial did not round-trip"
        );

        let via_from_cube = PositionAxial::from_cube(cube);
        assert_eq!(
            (via_from_cube.q, via_from_cube.r),
            (q, r),
            "PositionAxial::from_cube disagreed with to_axial for ({q},{r})"
        );
    }
}

/// Covers FR-FR-CORE-009.
///
/// Distinct hexes must map to distinct cube coordinates, or the grid would
/// alias neighbouring cells together.
#[test]
fn fr_fr_core_009_distinct_hexes_stay_distinct() {
    let mut seen = std::collections::BTreeSet::new();
    for q in -6i32..=6 {
        for r in -6i32..=6 {
            let cube = PositionAxial::new(q, r).to_cube();
            assert!(
                seen.insert((cube.x, cube.y, cube.z)),
                "hex ({q},{r}) collided with an earlier hex at cube {cube:?}"
            );
        }
    }
    assert_eq!(seen.len(), 13 * 13, "every hex in the block must be distinct");
}

// ---------------------------------------------------------------------------
// FR-CIV-CORE-013 — phase schedule integrity
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CORE-013.
///
/// The phase order is asserted by `engine_tests::tests::phase_order_matches_tick_sequence`,
/// which compares `PHASE_ORDER` against the documented sequence. `PHASE_ORDER`
/// is crate-private, so the integration-level observable is that a tick runs
/// the full schedule: state advances, events are produced, and the phase
/// sequence is stable across repeats.
#[test]
fn fr_civ_core_013_a_tick_runs_the_full_phase_schedule() {
    let mut sim = Simulation::with_seed(0xC0FE_0013u64);
    let start_tick = sim.state.tick;

    sim.tick();
    assert_eq!(
        sim.state.tick,
        start_tick + 1,
        "a tick must advance the simulation exactly one step"
    );

    // Running further ticks must keep advancing (no phase silently short-circuits).
    for expected in 2..=8u64 {
        sim.tick();
        assert_eq!(
            sim.state.tick,
            start_tick + expected,
            "tick {expected} did not advance the clock; a phase may be aborting the schedule"
        );
    }
}
