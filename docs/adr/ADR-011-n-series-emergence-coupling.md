# ADR 011: N-Series Emergence Coupling Architecture

## Status: Accepted

## Context

Civis implements emergent civilisation behaviour through a series of **N-series bidirectional couplings** in `crates/engine/src/engine.rs` and `crates/engine/src/emergence.rs`. Each coupling connects two simulation layers so that changes in one layer propagate into the other (downward causation), producing Class-4 edge-of-chaos dynamics rather than isolated parallel simulations.

The couplings landed incrementally across multiple PRs. Without a consolidated record, the design intent—shared gradients, conserved resources, bidirectional propagation—is only reconstructable from individual PR commit messages and FR trace snapshots.

Known N-series couplings at time of writing (Snapshot 3, 2026-06-21):

| ID | Coupling | Key symbols | FR refs |
|----|----------|-------------|---------|
| N5 | Language → trade friction + psyche cost | `language_trade_factor`, `faction_language_centroids` | FR-CIV-LANG-001, FR-CIV-PSYCHE-912 |
| N6 | Saga significance → belief | `apply_saga_belief_gain`, `saga_belief_gain` | FR-CIV-LEGENDS-001 |
| N7 | Sentience awakening → belief + cohesion | `awakening_belief_gain`, `awakening_cohesion_gain` | FR-CIV-GENETICS (family) |
| N8 | Non-food commodity scarcity → faction unrest | `commodity_unrest_delta` | FR-CIV-ECON (family), FR-ECON-001 |
| N9 | Faction aggression threshold reduction | `faction_aggression` | FR-CIV-EMERGENCE-N9 |
| N10 | Kinship → cohesion (bidirectional) | (in progress) | FR-CIV-EMERGENCE-N10 |
| N11 | (in progress) | — | FR-CIV-EMERGENCE-N11 |
| N12 | (in progress) | — | FR-CIV-EMERGENCE-N12 |

The primary failure mode these couplings guard against is **theatre emergence**: simulations that look active but whose layers do not actually influence each other (compositionality test fails).

## Decision

All N-series couplings share a single architectural contract:

1. **Shared gradient, not API call.** Couplings propagate via a shared mutable value on the simulation state (e.g. `faction.unrest`, `faction.cohesion`, `agent.belief`) — not via function calls between crates. This keeps the coupling latency-free and avoids crate dependency cycles.

2. **Bidirectional by default.** Each coupling must have a downward path (upper layer affects lower) AND an upward path (lower layer feeds back). Unidirectional couplings are transitional stubs, not final state.

3. **Conserved resource budget.** Coupling deltas must be bounded (e.g. `commodity_unrest_delta` is capped; `awakening_belief_gain` is bounded) so that runaway amplification cannot drive the system to heat-death or explosion.

4. **Three tests minimum per coupling.** Each N-series coupling is considered FULL only when at least 3 `#[test]` functions in `engine.rs` cover: (a) the happy path, (b) a boundary/cap condition, and (c) a decay or reverse-direction case.

5. **FR traceability required.** Every coupling must be tagged with a `FR-CIV-*` identifier in the function doc comment and in the coupling call site.

6. **Emergence dashboard observability.** Each coupling contributes to at least one metric visible in the emergence dashboard (power-law fit, entropy, structure count, novelty rate, or mutual-information coupling strength). This is the compositionality test: if the dashboard shows no signal from a coupling, the coupling is theatre.

## Alternatives considered

- **Event bus between crates:** cleaner crate boundaries but introduces latency (one tick minimum lag) and makes bidirectional coupling harder to reason about. Deferred for couplings that cross process boundaries (e.g. mod-host events).
- **Shared-memory ECS components:** Bevy-style components would allow O(1) random access but require migrating the engine off hecs. Deferred — hecs is the current ECS.
- **Per-coupling crate:** maximum isolation but explosion of crates and ceremony. Rejected — the engine.rs shared-state approach keeps coupling code co-located and easy to audit.
- **Parallel simulation silos:** each layer runs independently, outputs aggregated at display time. This is the #1 anti-pattern (theatre emergence). Rejected categorically.

## Consequences

- `engine.rs` accumulates all coupling call sites — currently 8318 lines. Line count will grow with each N-series addition. Mitigation: periodic extraction of coupling helpers into `emergence.rs`.
- FR trace snapshots (FR_TRACE_SNAPSHOT_*.md) are the authoritative audit of coupling completion status — not code comments alone.
- New couplings must not bypass the 3-test minimum or the bounded-delta rule. CodeRabbit and governance gate are the enforcement surface.
- The upgrade path for N10+ is dirty-chunk propagation (see ADR-010) so that coupling steps are only applied to chunks that received a change signal, avoiding O(grid) coupling scans.