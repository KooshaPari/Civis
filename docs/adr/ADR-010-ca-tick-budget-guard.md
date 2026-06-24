# ADR 010: CA Tick Budget Guard

## Status: Accepted

## Context

The Cellular Automata engine steps the full 256³ grid every tick (2 parity passes + 5 thermodynamic passes, single-threaded) and then despawns and rebuilds all chunks on every tick. This O(grid) + O(all-chunks) work causes the simulation to freeze — the same class of bug as the WSM3D single-thread terrain redraw (see project notes on CA freeze).

Band-aids tried: reducing `CA_TICK_HZ` alone does not reduce per-tick work, only call frequency.

## Decision

Add `CaTickBudget { max_chunks_per_step: 64, tick_hz: 2.0 }`. The CA step function short-circuits after `max_chunks_per_step` dirty chunks have been processed. A rate-limit timer (`tick_hz`) prevents the step from firing more than 2 times per second regardless of frame rate.

## Alternatives considered

- **Reduce CA_TICK_HZ only**: band-aid; per-tick work stays O(grid), freeze returns under load.
- **Dirty-chunk tracking** (Phase 2 target): only step chunks that received a change signal; restores full convergence speed while keeping per-frame work proportional to actual change rate. More invasive — requires a change-propagation queue and chunk ownership tracking. Deferred.
- **Thread-pool parallelism**: chunks can be stepped in parallel if ownership is non-overlapping. Requires a chunk ownership model refactor and careful boundary exchange. Deferred to Phase 3.

## Consequences

- **Frame time**: O(budget) per tick, not O(grid). Freeze eliminated.
- **Convergence speed**: slower — a full 256³ sweep now takes `ceil(256³ / 64)` ticks instead of 1.
- **Upgrade path**: dirty-chunk queue (Phase 2) restores fast convergence. The budget guard stays as a safety ceiling even after Phase 2 lands.
- **Observability**: the step function should emit a `chunks_stepped` metric so the emergence dashboard can detect if budget is chronically saturated (signals Phase 2 is needed).