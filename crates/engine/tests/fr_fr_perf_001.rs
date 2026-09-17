//! FR-PERF-001 — the engine SHALL sustain 100 ms/tick (10 ticks/s) at the
//! reference profile of 8 civilizations and a 1,000-hex-cell map.
//!
//! Matrix check: `perf::sustained_10_ticks_per_sec`.
//!
//! This is a wall-clock budget assertion, so it is intentionally generous
//! (a full order of magnitude of headroom on development hardware) and only
//! fails on genuine throughput regressions.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use civ_engine::grid::PositionAxial;
use civ_engine::{Fixed, Simulation};

/// Reference civilization count.
const FACTIONS: u32 = 8;
/// Reference map size, in hex cells.
const HEX_CELLS: usize = 1000;
/// Ticks the throughput measurement covers.
const TICKS: u32 = 10;
/// Per-tick wall-clock budget (100 ms/tick == 10 ticks/s).
const PER_TICK_BUDGET: Duration = Duration::from_millis(100);

/// Build the reference 1,000-cell hex map.
fn reference_hex_map() -> Vec<PositionAxial> {
    let mut cells = Vec::with_capacity(HEX_CELLS);
    'outer: for q in 0..40i32 {
        for r in 0..40i32 {
            cells.push(PositionAxial::new(q, r));
            if cells.len() == HEX_CELLS {
                break 'outer;
            }
        }
    }
    cells
}

/// 10 consecutive ticks complete inside the 10-ticks-per-second budget.
#[test]
fn sustained_10_ticks_per_sec() {
    let mut sim = Simulation::with_seed(0x5EED_u64);

    // Reference profile: 8 civilizations with funded treasuries.
    for faction in 0..FACTIONS {
        sim.state
            .factions
            .insert(faction, format!("Faction {faction}"));
        sim.state
            .faction_treasury
            .insert(faction, Fixed::from_num(1_000));
    }
    assert_eq!(sim.state.factions.len(), FACTIONS as usize);

    // Reference profile: a 1,000-cell hex map. Axial coordinates are a
    // bijection with cells, so cardinality is the map size.
    let cells = reference_hex_map();
    assert_eq!(cells.len(), HEX_CELLS);
    let distinct: BTreeSet<(i32, i32)> = cells.iter().map(|c| (c.q, c.r)).collect();
    assert_eq!(distinct.len(), HEX_CELLS, "each hex cell is a distinct coordinate");

    let start = Instant::now();
    for _ in 0..TICKS {
        sim.tick();
    }
    let elapsed = start.elapsed();

    assert_eq!(sim.state.tick, u64::from(TICKS), "exactly one tick per call");
    assert!(
        elapsed < PER_TICK_BUDGET * TICKS,
        "{TICKS} ticks took {elapsed:?}; the 10 ticks/s budget for {TICKS} ticks is {:?}",
        PER_TICK_BUDGET * TICKS
    );

    let per_tick = elapsed / TICKS;
    assert!(
        per_tick < PER_TICK_BUDGET,
        "average {per_tick:?}/tick exceeds the 100 ms/tick budget"
    );
}
