# Emergence Dashboard — Branching Ratio (σ) Implementation Spec

**Status:** SPEC (read-only research → implementation handoff)  
**Date:** 2026-06-16  
**Scope:** Single highest-value missing SOC criticality metric  
**Sources:** `crates/civ-emergence-metrics`, `crates/engine/src/emergence_metrics.rs`, `docs/design/emergence-dashboard.md` §3.6, `CRITICALITY_METRICS_GAP.md`  
**Constraint:** This document is the implementer handoff; it does not modify source.

---

## 1. Executive summary

`civ-emergence-metrics` ships pure-math primitives for weak-emergence observability. **Only Shannon entropy and structure count are wired into the live tick loop**; a power-law fit module exists but is unused. The charter’s six SOC-criticality metrics (§3.1–§3.6) are mostly absent at runtime.

**Recommended next metric:** rolling-mean **branching ratio** `σ̄` (charter §3.6). It is the only charter signal that directly discriminates all three named regimes—**heat-death** (`σ̄ < 0.85`), **edge-of-chaos** (`σ̄ ∈ [0.95, 0.99]`), and **explosion** (`σ̄ > 1.0`)—and it is the upstream event-size stream for power-law fitting (§3.1).

---

## 2. Current state — what `civ-emergence-metrics` computes

### 2.1 Crate modules (`crates/civ-emergence-metrics/src/`)

| Module | Metric | Formula / output | `Metric` trait | Engine wired |
|--------|--------|------------------|----------------|--------------|
| `shannon.rs` | Shannon entropy | `H = −Σ pᵢ log₂ pᵢ`; normalised `H / log₂ N` | `shannon_entropy` → bits | **Yes** (material layer) |
| `structure.rs` | Structure count | 6-connectivity CC on binary mask; `count`, `largest`, `foreground` | `structure_count_largest_fraction` (histogram proxy on trait path) | **Yes** (one 16³ chunk) |
| `power_law.rs` | Power-law fit | OLS log-log rank-frequency; `α`, `R²` | `power_law_alpha` → α only | **No** |
| `dashboard.rs` | Five social tiles | `cluster_entropy`, `ideology_homophily`, `sentience_fraction`, `psyche_stability`, `diplomacy_tension` | N/A | **Yes** (social observability, not SOC) |

Shared types: `Histogram`, `Grid<'a, T>`, `Metric` trait, `SCHEMA_VERSION = "0.3.0-dashboard-block"`.

`lib.rs` explicitly defers: power-law windowing, novelty rate, coupling MI, **branching ratio**.

### 2.2 Engine sampler (`crates/engine/src/emergence_metrics.rs`)

**Tick hook:** end of `Simulation::tick_with_emergence_source`, after `phase_chronicle`, before `replay_log.record_tick` (`engine.rs` ~1451–1461).

**Cadence:** unified `EMERGENCE_SAMPLE_INTERVAL = 50` ticks (5 s at 100 ms). Charter §4 calls for 10 Hz entropy and 1 Hz structure; implementation uses one 5 s boundary for both.

| Signal | Computed | On `EmergenceSample` | JSON-RPC / snapshot | Replay bus |
|--------|----------|------------------------|---------------------|------------|
| Material Shannon `H`, `H_norm` | Every 50 ticks | `entropy_bits`, `entropy_norm` | Yes | No |
| Structure `C_t`, `L_t`, foreground | Every 50 ticks | `structure_count`, `structure_largest`, `structure_foreground` | Yes | No |
| Dashboard five-tile block | Every 50 ticks | `dashboard.*` | Yes | `emergence.metrics.v1` (dashboard only) |
| Power-law `α` | No | — | — | — |
| Novelty rate | No | — | — | — |
| Coupling MI | No | — | — | — |
| Branching ratio `σ` | No | — | — | — |

### 2.3 Charter gap matrix (§3.1–§3.6)

| Charter § | Signal | Crate | Engine | Primary alarms |
|-----------|--------|-------|--------|----------------|
| §3.1 | Power-law `(α, D)` | Partial (`α`, `R²` only; no KS `D`) | No | MT-001, MT-002, MT-003 |
| §3.2 | Normalised Shannon `H_L` | Yes (1 layer) | Partial | MT-004, MT-005 |
| §3.3 | `C_t`, `L_t`, exponent `β` | Partial (`C_t`, `L_t` only) | Partial | MT-006, MT-007 |
| §3.4 | Per-capita novelty rate | **Missing** | No | MT-008, MT-009 |
| §3.5 | Normalised MI between layers | **Missing** | No | MT-010, MT-011 |
| §3.6 | Rolling-mean branching ratio `σ` | **Missing** | No | **MT-012, MT-013** |

### 2.4 Failure-mode coverage today

| Failure mode | Best current detector | Gap |
|--------------|----------------------|-----|
| **Heat-death** | Low `entropy_norm`, declining `structure_count` | No subcritical `σ`; no novelty stagnation (AC-007) |
| **Explosion** | High `entropy_norm` (weak) | **No `σ > 1` alarm** (AC-002) |
| **Edge-of-chaos** | None operational | **No `σ ∈ [0.95, 0.99]` band** |
| **Emergence theater** | Dashboard homophily / cluster entropy | No MI over-coupling (AC-003) |

Entropy alone is insufficient: subcritical frozen worlds and supercritical runaways can both present moderate histogram entropy; `σ` measures whether activity **amplifies or dies**—the operational SOC discriminator.

---

## 3. Metric selection — why branching ratio `σ` first

| Candidate | Value | Blocker |
|-----------|-------|---------|
| **Branching ratio `σ`** | **Highest** — AC-002/AC-005 are `σ`-only; triages all three regimes | Needs avalanche ledger (new), but **O(1) per event** |
| Power-law fit | High — `PowerLawFit` already in crate | Requires **4096-event rolling window**; depends on avalanche definition |
| Coupling MI | Medium — theater / silos | Does not distinguish heat-death from explosion |
| Novelty rate | Medium — stasis / churn | Slow alarm (4096 ticks); weak explosion signal |
| Structure `β` | Medium — percolation signature | Needs multi-scale sampling; secondary to `σ` |

**Dependency unlock:** avalanche size stream from `σ` ledger unblocks §3.1 power-law wiring with existing `power_law.rs`.

---

## 4. Branching ratio `σ` — concrete specification

### 4.1 Definitions

**Micro-actor action:** one counted unit of sim activity attributable to a phase output (see §4.4). Deterministic; no RNG in counting.

**Avalanche `a`:** a connected burst of micro-activity that continues while child events fire within the same tick window (charter §3.6). An avalanche **opens** on tick `t` when seed actors fire; it **closes** when tick `t+k` produces zero descendants for all open avalanches, or when `s_a ≥ sim.max_avalanche_size`.

**Per-avalanche branching ratio:**

```
σ_a = N_descendants(a, t+1) / N_actors(a, t)
```

Where:

- `N_actors(a, t)` — count of micro-actor actions that **seeded** avalanche `a` at tick `t` (denominator; must be > 0 for a valid ratio)
- `N_descendants(a, t+1)` — count of child micro-actions in tick `t+1` **causally attributed** to avalanche `a`

**Edge cases (pure math, `branching.rs`):**

```
σ_a = 0.0                    if N_actors = 0  (no seed; do not push to ledger)
σ_a = N_desc / N_actors      otherwise         (f32; no cap except fuse)
```

**Rolling metric** (dashboard primary scalar):

```
σ̄_W = (1 / min(W, |ledger|)) · Σ_{a ∈ last W closed avalanches} σ_a
```

Defaults:

| Parameter | Default | Charter ref |
|-----------|---------|-------------|
| `W` (rolling window) | 10 avalanches | AC-002 consecutive-avalanche count |
| `max_avalanche_size` | scenario knob `sim.max_avalanche_size` | §6, AC-002 fuse |

**Normalised edge-of-chaos score** (optional dashboard tile, `0..1`):

```
σ_score = clamp((σ̄_W − 0.85) / (0.99 − 0.85), 0, 1)
```

Interpretation: `0` → deep subcritical; `1` → top of critical band; values above `1` clamp (supercritical).

**Avalanche size** (feeds §3.1 later):

```
s_a = Σ_{ticks in avalanche} N_actors(a, tick) + N_descendants(a, tick)
```

Push `s_a` to a separate ring buffer (`W_pow = 4096`) on avalanche close.

### 4.2 Regime interpretation and alarms

| Condition | Regime | Dashboard colour | Alarm | AC |
|-----------|--------|------------------|-------|-----|
| `σ̄_W < 0.85` sustained ≥ 100 consecutive **ticks** | **Subcritical / heat-death** | Blue / cold | **MT-013** | AC-005 |
| `σ̄_W ∈ [0.95, 0.99]` sustained (no alarm) | **Edge-of-chaos / SOC** | Green | none | target operating band |
| `0.85 ≤ σ̄_W < 0.95` or `0.99 < σ̄_W ≤ 1.0` | Transitional / watch | Amber | optional advisory | — |
| `σ̄_W > 1.0` for ≥ 10 consecutive **closed avalanches** | **Supercritical / explosion** | Red | **MT-012** | AC-002 |
| any single `s_a > sim.max_avalanche_size` | **Explosion fuse** | Red | **MT-012** (immediate) | AC-002 |

**Tick vs avalanche counters:**

- MT-013 uses **tick** streak (charter AC-005: "≥ 100 ticks") — implement as `subcritical_tick_streak` incremented each tick where `σ̄_W < 0.85` (or no avalanches closed and ledger empty → treat as subcritical silence).
- MT-012 uses **avalanche** streak — `supercritical_avalanche_streak` incremented on each avalanche close where `σ_a > 1.0`; reset to 0 when `σ_a ≤ 1.0`.

**Composability with existing metrics:**

| Regime | `σ̄_W` | Supporting signals (already wired) |
|--------|--------|-------------------------------------|
| Heat-death | `< 0.85` | `entropy_norm` declining; `structure_count` ↓ ≥30% / 1024 ticks (AC-001) |
| Edge-of-chaos | `[0.95, 0.99]` | `entropy_norm ∈ [0.6, 0.9]` on ≥3 layers (future); `β ∈ [0.35, 0.50]` (future) |
| Explosion | `> 1.0` | `entropy_norm` spike or clumping (MT-005); `max_avalanche_size` breach |

### 4.3 Pure-math crate module

**New file:** `crates/civ-emergence-metrics/src/branching.rs`

**Public API (spec-level; implementer writes code):**

| Type / fn | Responsibility |
|-----------|----------------|
| `BranchingLedger` | Fixed-capacity ring buffer of closed `(σ_a, s_a, close_tick)` |
| `sigma_a(actors: u32, descendants: u32) -> f32` | Per-avalanche ratio with zero-actor guard |
| `rolling_mean_sigma(ledger: &BranchingLedger, window: usize) -> f32` | `σ̄_W`; returns `0.0` if ledger empty |
| `sigma_score(sigma_bar: f32) -> f32` | Normalised `0..1` tile |
| `BranchingLedger::push_closed(...)` | Append closed avalanche; evict oldest at capacity |

**Tests (required before engine wire):**

1. `σ_a`: actors=10, descendants=9 → `0.9`
2. `σ_a`: actors=0 → `0.0` (no ledger push)
3. `σ̄_W`: sequence `[0.8, 0.9, 1.1, 0.95]` window=4 → `0.9375`
4. `sigma_score(0.85)=0`, `sigma_score(0.99)=1`, `sigma_score(1.1)=1` (clamped)
5. Determinism: same push sequence → same `σ̄_W` (no HashMap, no float platform variance beyond `f32`)

Bump `SCHEMA_VERSION` to `0.4.0-branching-ratio` on wire shape change.

### 4.4 v1 avalanche bootstrap — actor/descendant attribution

Charter §3.1.1 defines events as typed bursts. **v1 uses existing per-tick counters** already cleared at tick start in `Simulation`—no new gameplay systems.

#### Actor sources (tick `t`, count toward open avalanche seeds)

| Phase / field | Actor count rule |
|---------------|------------------|
| `phase_voxel` | `last_tick_voxel_events.len()` (dirty chunk events) |
| `phase_disasters` | cells mutated this tick (disaster write count) |
| `phase_diplomacy` | `diplomacy_events.len()` after `phase_diplomacy` |
| `phase_unrest` / `phase_faction_unrest` | agents with positive unrest delta this tick |
| `last_tick_combat_pulses` | `last_tick_combat_pulses.len()` |
| `last_tick_engagements` | `last_tick_engagements.len()` |

**Aggregation rule v1:** one global avalanche per tick-pair window (simplest deterministic bootstrap). `N_actors(t) = sum(actor counts above)`. Future PRs may split by `AvalancheKind` per charter event types.

#### Descendant sources (tick `t+1`, attributed to avalanche opened at `t`)

| Source | Descendant count rule |
|--------|----------------------|
| Voxel | `last_tick_voxel_events.len()` on `t+1` if same chunk coords overlap seed tick (v1: any voxel activity on `t+1` counts) |
| Diplomacy | `diplomacy_events.len()` on `t+1` |
| Unrest | positive unrest deltas on `t+1` |
| Combat | `last_tick_combat_pulses.len()` + `last_tick_engagements.len()` on `t+1` |

**Closure rule:** at start of `phase_emergence_events_close()` (see §4.5):

1. If open avalanche exists and `current_tick > seed_tick`: compute `σ_a`, push to ledger, push `s_a` to power-law buffer.
2. If `N_descendants(current_tick) == 0` OR `s_a ≥ max_avalanche_size`: close avalanche.
3. Else: accumulate descendants into running `s_a`, keep avalanche open.

**Silence ticks** (zero actors and zero descendants): close any open avalanche with last computed `σ_a`; do not open a new avalanche.

### 4.5 Exact tick-loop hook

Current phase order ends with `phase_chronicle` → `sample_emergence` → `record_tick`. **Insert branching ledger updates without reordering existing phases.**

```
Simulation::tick_with_emergence_source()
  │
  ├─ [existing] state.tick += 1; clear last_tick_* buffers
  ├─ [existing] phase_production … phase_chronicle
  │
  ├─ NEW: phase_emergence_events_close()          ← after chronicle, BEFORE sample_emergence
  │         • read actor/descendant counts from last_tick_* and diplomacy_events
  │         • update/open/close EmergenceAvalancheState on Simulation
  │         • on close: BranchingLedger::push_closed(σ_a, s_a, tick)
  │         • evaluate MT-012 / MT-013 streak counters
  │         • optional: replay_log.record_emergence_alarm(...) on breach
  │
  ├─ [extend] sample_emergence()                  ← every 50 ticks AND every tick for σ̄
  │         • existing: Shannon, structure, dashboard
  │         • NEW: branching_sigma = rolling_mean_sigma(&ledger, W)
  │         • NEW: branching_sigma_score = sigma_score(branching_sigma)
  │         • NEW: avalanche_count_closed, supercritical_streak (diagnostics)
  │
  ├─ [extend] replay_log.record_emergence_metrics()  ← add σ̄_W to payload
  │
  └─ [existing] replay_log.record_tick()
```

**Per-tick hot path (O(1)):** `phase_emergence_events_close` runs once per tick; cost is integer sums over existing `Vec` lengths—no grid walks.

**Cadence note:** Charter §4 says branching ratio updates **per event**. v1 approximates with per-tick aggregation; document as `σ̄_W (tick-aggregated v1)` in wire schema. v2 can subdivide within-tick event ordering when replay bus carries typed micro-events.

**New `Simulation` fields:**

| Field | Type | Purpose |
|-------|------|---------|
| `emergence_avalanche` | `Option<OpenAvalanche>` | seed tick, running actor/descendant totals |
| `branching_ledger` | `BranchingLedger` | ring buffer |
| `subcritical_tick_streak` | `u32` | MT-013 |
| `supercritical_avalanche_streak` | `u32` | MT-012 |

### 4.6 Wire surfaces

| Surface | New fields |
|---------|------------|
| `EmergenceSample` (`emergence_metrics.rs`) | `branching_sigma: f32`, `branching_sigma_score: f32`, `branching_window: u32`, `avalanches_closed: u64` |
| `EmergenceSampleFields` (`jsonrpc.rs`) | same |
| `sim.snapshot` `.emergence` | same |
| `emergence.metrics.v1` replay event | extend JSON: `{ branching_sigma, branching_sigma_score, ... }` |
| `emergence.alarm.v1` replay event | `{ id: "MT-012"\|"MT-013", tick, value, threshold, window }` |

**Dashboard tile copy:**

| `σ̄_W` range | Label |
|------------|-------|
| `< 0.85` | Subcritical (heat-death risk) |
| `[0.85, 0.95)` | Subcritical → critical transition |
| `[0.95, 0.99]` | Edge of chaos (target) |
| `(0.99, 1.0]` | Near-supercritical |
| `> 1.0` | Supercritical (explosion risk) |

---

## 5. Implementation dependency DAG

| Phase | Task ID | Description | Depends on |
|-------|---------|-------------|------------|
| 1 | BR-001 | `branching.rs` + unit tests | — |
| 2 | BR-002 | `EmergenceAvalancheState` + `phase_emergence_events_close` in engine | BR-001 |
| 3 | BR-003 | Extend `EmergenceSample` + `sample_emergence` | BR-002 |
| 4 | BR-004 | JSON-RPC + replay bus + alarm events | BR-003 |
| 5 | BR-005 | Scenario knob `sim.max_avalanche_size` default | BR-002 |
| 6 | PL-001 | Feed `s_a` stream to `PowerLawFit` (§3.1) | BR-002 |
| 7 | QA-001 | Determinism test: same seed → same `σ̄_W` tick-for-tick | BR-004 |
| 8 | QA-002 | Synthetic scenarios: subcritical / critical / supercritical `σ` bands | BR-004 |

**Estimated agent effort:** BR-001..BR-004 ≈ 15–25 tool calls, ~8–15 min wall clock (pure math + engine wire + JSON-RPC).

---

## 6. Acceptance criteria closed by this metric

| AC | Detection rule | Closed by `σ`? |
|----|--------------|----------------|
| AC-002 Explosion | `σ̄ > 1.0` ≥ 10 avalanches OR `s_a > max_avalanche_size` | **Yes** |
| AC-005 Subcritical drift | `σ̄ < 0.85` ≥ 100 ticks | **Yes** |
| AC-001 Heat-death | `H_voxel` + `C_t` trend + `σ` | **Partial** (σ leg added) |
| AC-004 Power-law breakdown | `D` KS distance | **Unblocked** (avalanche sizes) |

Still open after `σ`: AC-003 (MI), AC-006 (MI pairs), AC-007 (novelty), AC-004 (`D` fit).

---

## 7. File reference

| Path | Role |
|------|------|
| `crates/civ-emergence-metrics/src/lib.rs` | Crate scope; lists deferred metrics |
| `crates/civ-emergence-metrics/src/shannon.rs` | Shannon entropy (wired) |
| `crates/civ-emergence-metrics/src/structure.rs` | Structure count (wired) |
| `crates/civ-emergence-metrics/src/power_law.rs` | Power-law α (unwired; consumer of `s_a`) |
| `crates/civ-emergence-metrics/src/dashboard.rs` | FR-CIV-EMERG-001 social tiles (wired) |
| `crates/engine/src/emergence_metrics.rs` | Runtime sampler |
| `crates/engine/src/engine.rs` | `tick_with_emergence_source` hook point ~1451 |
| `crates/server/src/jsonrpc.rs` | `EmergenceSampleFields` wire shape |
| `docs/design/emergence-dashboard.md` | Authoritative charter §3.6 |
| `CRITICALITY_METRICS_GAP.md` | Gap analysis (this spec supersedes for `σ` handoff) |

---

## 8. Non-goals (v1)

- Auto-tuning knobs (charter §6: recommendations only)
- Per-event within-tick avalanche splitting (v2)
- Clauset–Shalizi–Newman `D` KS distance (separate PR after `s_a` stream exists)
- MI, novelty rate, structure exponent `β` (follow charter §4 cadence table)
