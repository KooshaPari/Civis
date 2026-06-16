# Criticality Metrics Gap Analysis

**Status:** Read-only research (2026-06-16)  
**Scope:** `crates/civ-emergence-metrics`, engine sampler wiring, charter design in `docs/design/emergence-dashboard.md`  
**Constraint:** No source changes in this artifact.

---

## 1. Executive summary

The `civ-emergence-metrics` crate ships **pure-math primitives** for weak-emergence / SOC observability. Today only **Shannon entropy** and **structure count** are wired into the live tick loop; a **power-law fit** module exists but is not consumed. The charter’s six SOC-criticality metrics (§3.1–§3.6 of `docs/design/emergence-dashboard.md`) are **mostly missing at runtime**: novelty rate, coupling MI, branching ratio, power-law KS distance, and structure-count exponent β are unimplemented or unwired.

The **single highest-value next metric** is the **branching ratio** `σ` (§3.6). It is the only charter signal that directly discriminates all three regimes named in the problem statement—heat-death (`σ < 0.85`), edge-of-chaos (`σ ∈ [0.95, 0.99]`), and explosion (`σ > 1.0`)—and it is the input driver for the power-law avalanche-size distribution (§3.1).

---

## 2. What `civ-emergence-metrics` computes today

### 2.1 Crate modules (`crates/civ-emergence-metrics/src/`)

| Module | Metric | Formula / output | `Metric` trait | Unit tests |
|--------|--------|------------------|----------------|------------|
| `shannon.rs` | Shannon entropy | `H = −Σ pᵢ log₂ pᵢ`; normalised `H / log₂ N` | `shannon_entropy` → bits | Yes |
| `structure.rs` | Structure count | 6-connectivity CC on binary mask; `count`, `largest`, `foreground` | `structure_count_largest_fraction` (histogram proxy only) | Yes |
| `power_law.rs` | Power-law fit | OLS on log-log rank-frequency; `α`, `R²` | `power_law_alpha` → α only | Yes |
| `dashboard.rs` | Five social tiles | `cluster_entropy`, `ideology_homophily`, `sentience_fraction`, `psyche_stability`, `diplomacy_tension` | N/A (not `Metric`) | Yes |

Shared types: `Histogram`, `Grid<'a, T>`, `Metric` trait, `SCHEMA_VERSION = "0.3.0-dashboard-block"`.

### 2.2 Explicitly scoped but **not implemented** in the crate

From `lib.rs` lines 39–41:

> The remaining dashboard metrics (power-law fit, novelty rate, mutual information between sim layers, branching ratio) are scoped for follow-up PRs…

Interpretation:

| Charter metric (§) | Crate status |
|--------------------|--------------|
| §3.1 Power-law `(α, D)` | **Partial** — `α` and `R²` in `power_law.rs`; **no KS distance `D`**, no event-size rolling window |
| §3.2 Shannon entropy | **Complete** |
| §3.3 Structure count + `β` | **Partial** — `C_t`, `L_t` only; **no `β` exponent fit** over sampled region size |
| §3.4 Novelty rate | **Missing** |
| §3.5 Coupling MI | **Missing** |
| §3.6 Branching ratio `σ` | **Missing** |

### 2.3 `dashboard.rs` tiles are **not** charter criticality signals

The five `EmergenceDashboard` fields (FR-CIV-EMERG-001) are **social observability** proxies—cluster spread, ideology homophily, sentience share, mood variance, diplomacy tension. They help detect “theater” and illegibility but do **not** implement the SOC signatures in charter §2 (`σ`, `α`, `β`, avalanche statistics). They are orthogonal to the heat-death / explosion / edge-of-chaos triage.

---

## 3. What the engine wires today

**Sampler:** `crates/engine/src/emergence_metrics.rs`  
**Tick hook:** end of `Simulation::tick_with_emergence_source`, after `phase_chronicle`, before `replay_log.record_tick` (`engine.rs` ~1436–1445).

| Signal | Computed | Cached on `EmergenceSample` | JSON-RPC / snapshot | Replay bus |
|--------|----------|----------------------------|---------------------|------------|
| Material Shannon `H`, `H_norm` | Every 50 ticks | `entropy_bits`, `entropy_norm` | Yes | No |
| Structure `C_t`, `L_t`, foreground | Every 50 ticks | `structure_count`, `structure_largest`, `structure_foreground` | Yes | No |
| Dashboard five-tile block | Every 50 ticks | `dashboard.*` | Yes | `emergence.metrics.v1` (dashboard only) |
| Power-law `α` | **No** | — | — | — |
| Novelty rate | **No** | — | — | — |
| Coupling MI | **No** | — | — | — |
| Branching ratio `σ` | **No** | — | — | — |

**Cadence note:** Charter §4 calls for 10 Hz entropy and 1 Hz structure; implementation uses a unified **50-tick (5 s)** boundary via `EMERGENCE_SAMPLE_INTERVAL`.

**Criticality test gap:** `emergence_stays_bounded_and_dynamic_over_5000_ticks` checks scalar bounds (belief, cohesion, research tier) but does **not** assert entropy, `σ`, or power-law bands.

---

## 4. Charter criticality signals — full gap matrix

Source: `docs/design/emergence-dashboard.md` §2–§3, §7 acceptance criteria.

| ID | Charter signal | Detects | Crate | Engine wire | Alarm IDs |
|----|----------------|---------|-------|-------------|-----------|
| §3.1 | Power-law `α`, KS `D` on event sizes | SOC vs sub/super-critical; “theater” | Partial (`α` only) | No | MT-001, MT-002, MT-003 |
| §3.2 | Normalised Shannon `H_L` (5 layers) | Heat-death collapse; single-bin takeover | Yes (1 layer: voxel material) | Partial (material only) | MT-004, MT-005 |
| §3.3 | `C_t`, `L_t`, exponent `β` | Structural collapse; single-cluster domination | Partial (`C_t`, `L_t`) | Partial (one 16³ chunk) | MT-006, MT-007 |
| §3.4 | Per-capita novelty rate | Stasis / theater; churn explosion | **Missing** | No | MT-008, MT-009 |
| §3.5 | Normalised MI between layer pairs | Over-coupling; decoupled silos | **Missing** | No | MT-010, MT-011 |
| §3.6 | Rolling-mean branching ratio `σ` | **Primary** heat-death vs edge-of-chaos vs explosion | **Missing** | No | MT-012, MT-013 |
| §3.6→§3.1 | Avalanche size stream | Feeds power-law window | **Missing** | No | AC-002, AC-004 |

### 4.1 Failure-mode coverage with current instrumentation

| Failure mode | Best current detector | Gap |
|--------------|----------------------|-----|
| **Heat-death** | Low `entropy_norm`, declining `structure_count` | No subcritical `σ`; no novelty stagnation (AC-007); no `β` trend |
| **Explosion** | High `entropy_norm` (clumping) only weakly | **No `σ > 1` alarm** (AC-002); no per-capita novelty ceiling |
| **Edge-of-chaos** | None operational | **No `σ ∈ [0.95, 0.99]` band**; no `α ∈ [1.4, 2.0]` |
| **Emergence theater** | Dashboard homophily / cluster entropy | No MI over-coupling (AC-003, AC-006) |

---

## 5. Recommended next metric: branching ratio `σ`

### 5.1 Why `σ` over the other gaps

| Candidate | Value | Blocker |
|-----------|-------|---------|
| **Branching ratio `σ`** | **Highest** — charter §2 names it alongside `α` and `β`; AC-002/AC-005 are `σ`-only; uniquely triages all three regimes | Needs avalanche ledger (new), but **O(1) per event** |
| Power-law fit | High — `PowerLawFit` already in crate | Requires **4096-event rolling window** (§3.1); depends on avalanche definition anyway |
| Coupling MI | Medium — detects theater / silos | Does not distinguish heat-death from explosion |
| Novelty rate | Medium — stasis / churn | Slow alarm (4096 ticks); weak explosion signal |
| Structure `β` | Medium — percolation signature | Needs multi-scale sampling over `S`; secondary to `σ` |

**Entropy alone is insufficient:** a subcritical frozen world and a supercritical runaway can both present moderate histogram entropy depending on layer; `σ` measures whether activity **amplifies or dies**—the operational SOC discriminator.

### 5.2 Exact formula (charter §3.6)

For each avalanche `a` (a connected burst of micro-activity that continues while child events fire within the same tick window):

```
σ_a = N_descendants(a, t+1) / N_actors(a, t)
```

Where:

- `N_actors(a, t)` — count of micro-actor actions that **seeded** avalanche `a` at tick `t`
- `N_descendants(a, t+1)` — count of child micro-actions in tick `t+1` **causally attributed** to avalanche `a`

**Rolling metric** (exposed to dashboard):

```
σ̄_W = (1/W) · Σ_{a ∈ window} σ_a        (default W = last 10 avalanches for AC-002)
```

**Normalised edge-of-chaos score** (optional dashboard tile, 0..1):

```
σ_score = clamp((σ̄_W − 0.85) / (0.99 − 0.85), 0, 1)
```

**Alarm thresholds** (from charter):

| Condition | Regime | Alarm |
|-----------|--------|-------|
| `σ̄ > 1.0` for ≥ 10 consecutive avalanches | Supercritical / explosion | MT-012 (AC-002) |
| `σ̄ < 0.85` for ≥ 100 consecutive ticks | Subcritical / heat-death | MT-013 (AC-005) |
| `σ̄ ∈ [0.95, 0.99]` sustained | Edge-of-chaos band | Green (no alarm) |

### 5.3 v1 avalanche bootstrap (no new gameplay systems)

Charter §3.1.1 defines events as typed bursts: voxel writes, CC merge/split, combat engagements, market clears, faction membership clusters. For a **minimal deterministic v1**, attribute actors/descendants from existing tick outputs:

| Event source | Actor (tick `t`) | Descendant (tick `t+1`) |
|--------------|------------------|-------------------------|
| `phase_disasters` | disaster cells mutated | adjacent CA activations next tick |
| `phase_diplomacy` | `DiplomacyEvent` count this tick | treasury / relation delta magnitude next diplomacy window |
| `phase_unrest` | `unrest_delta > 0` drivers fired | population-affecting phases next tick |
| Voxel writes (existing diff) | `Δvoxels` this tick | `Δvoxels` next tick on same chunk |

**Avalanche closure rule:** an avalanche ends when a tick produces zero descendants for all open avalanches, or `sim.max_avalanche_size` is hit (charter §6 knob).

### 5.4 Where to hook into the tick loop

```
Simulation::tick_with_emergence_source()
  │
  ├─ [existing phases: economy … institutions, chronicle]
  │
  ├─ NEW: phase_emergence_events_close()     ← end open avalanches; push σ_a to ring buffer
  │         (after chronicle, before sampler)
  │
  ├─ sample_emergence()                      ← extend EmergenceSample with σ̄_W, σ_score
  │     └─ civ_emergence_metrics::branching::rolling_mean(&ledger)
  │
  ├─ replay_log.record_emergence_metrics()   ← add σ̄_W field to emergence.metrics.v1
  │
  └─ replay_log.record_tick()
```

**Per-tick (hot path, O(1)):** at end of each phase that emits micro-events, call `EmergenceEventLedger::record_actors(n)` / `record_descendants(n)` on the open avalanche. Store ledger on `Simulation` alongside `emergence_sample: Option<EmergenceSample>`.

**New crate module:** `crates/civ-emergence-metrics/src/branching.rs`

```rust
pub struct BranchingLedger { /* ring buffer of σ_a, tick of last close */ }

pub fn sigma_a(actors: u32, descendants: u32) -> f32 {
    if actors == 0 { 0.0 } else { descendants as f32 / actors as f32 }
}

pub fn rolling_mean_sigma(ledger: &BranchingLedger, window: usize) -> f32 { /* … */ }
```

**Wire surfaces:**

| Surface | Field |
|---------|-------|
| `EmergenceSample` | `branching_sigma: f32`, `branching_sigma_score: f32` |
| `EmergenceSampleFields` (JSON-RPC) | same |
| `emergence.metrics.v1` replay event | extend payload |
| `emergence.alarm.v1` | MT-012 / MT-013 when thresholds breach |

### 5.5 Implementation dependency order

1. **`branching.rs`** + ledger (pure math, unit tests on synthetic actor/descendant sequences)
2. **Engine ledger** + per-phase actor/descendant hooks (start with diplomacy + unrest + voxel Δ)
3. **Extend `sample_emergence`** + JSON-RPC + replay bus
4. **Power-law §3.1** — reuse same avalanche size stream for `W_pow = 4096` window (unblocks `PowerLawFit` already in crate)
5. MI, novelty, `β` fit — follow charter §4 cadence table

---

## 6. Quick reference — files

| Path | Role |
|------|------|
| `crates/civ-emergence-metrics/src/lib.rs` | Crate scope; lists deferred metrics |
| `crates/civ-emergence-metrics/src/shannon.rs` | Shannon entropy |
| `crates/civ-emergence-metrics/src/structure.rs` | 6-connectivity structure count |
| `crates/civ-emergence-metrics/src/power_law.rs` | Power-law α (unwired) |
| `crates/civ-emergence-metrics/src/dashboard.rs` | FR-CIV-EMERG-001 social tiles |
| `crates/engine/src/emergence_metrics.rs` | Runtime sampler + tick hook |
| `crates/engine/src/engine.rs` | `tick_with_emergence_source` calls `sample_emergence` |
| `crates/server/src/jsonrpc.rs` | `EmergenceSampleFields` wire shape |
| `docs/design/emergence-dashboard.md` | Authoritative charter metric definitions |

---

## 7. Acceptance criteria still open after `σ`

| AC | Requires |
|----|----------|
| AC-001 Heat-death | `H_voxel` + `C_t` trend (partial) + `σ < 0.85` |
| AC-002 Explosion | **`σ > 1.0`** + `max_avalanche_size` |
| AC-003 Theater | MI + `L_t/C_t` trend |
| AC-004 Power-law breakdown | `D` KS distance (not in crate) |
| AC-005 Subcritical drift | **`σ < 0.85`** |
| AC-006 Over-coupling | MI on ≥ 2 layer pairs |
| AC-007 Novelty stagnation | Per-capita novelty rate |

Adding **branching ratio `σ`** closes AC-002, AC-005, and unlocks the avalanche-size stream for AC-004 and §3.1 power-law wiring.
