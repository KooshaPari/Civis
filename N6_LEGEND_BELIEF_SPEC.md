# N6 — Saga Significance → Belief Accumulation Coupling

**Status:** Research / design handoff (read-only audit, 2026-06-16)  
**Charter gap:** `LEGENDS/RELIGION` — the saga graph (`SagaGraph`) is the most expressive emergence layer (named entities, causal DAG, epoch digests, query API), but its product is **display-only**. The macro `state.belief` is fed exclusively by `phase_belief` (population worship + temple) and `phase_unrest` (hardship → faith); the saga never contributes. **Faith has no famous figures.**  
**Predecessors:** N1 (settlement stocks → market), N2 (culture → diplomacy), N3 (settlement contact → diplomacy pairs), N4 (settlement exchange → emergent trade routes), N5 (language → trade friction).  
**Scope:** Specify the **single highest-leverage coupling** that closes the LEGENDS layer against an already-wired macro consumer (`state.belief`). No source changes in this artifact.

---

## 1. Why this coupling (not saga → cohesion / diplomacy / unrest)

| Weak layer | Current state | Minimal coupling options | Verdict |
|------------|---------------|--------------------------|---------|
| **LEGENDS/RELIGION** | `SagaGraph` ingests births/deaths/diplomacy/sentience every tick in `phase_emergence`; `EntityNode.significance` is a 0..1 (unbounded upward) decayed score with `promoted: bool` once `>=1.0`; `SagaGraph::significant_desc()` iterates the top-N; `IngestOutcome.promoted: Vec<LegendEntityId>` lists entities whose significance crossed the threshold this tick | (A) top-N significance sum + per-promotion bonus → `state.belief`; (B) same → `state.cohesion`; (C) same → diplomacy peace threshold; (D) same → unrest | **(A) chosen** |
| **LANGUAGE** | N5 closes language → trade friction | — | N5 scope |
| **TRADE-ROUTE / network** | N4 closes settlement exchange → emergent routes | — | N4 scope |

**Leverage ranking for the LEGENDS closure:** belief **>** cohesion **>** diplomacy **>** unrest.

| Alternative | Verdict |
|-------------|---------|
| **Saga significance → `state.belief`** *(chosen)* | **Direct identity of subject matter** — `belief` is the macro resource that *is* "famous figures made into faith". The divine-powers economy already spends it (FR-CIV-EMERGENCE) and downstream layers (cohesion, diplomacy, temples) already read it. Adds a bounded, decayed, proportional arm to an existing equilibrium (worship + temple + hardship vs. `BELIEF_DECAY_DIVISOR=500` decay). **No new persistent state.** |
| Saga significance → `state.cohesion` | Redundant — cohesion is *already* fed by belief via `cohesion_delta(belief, unrest)` (`engine.rs:3779`). Routing saga through belief propagates to cohesion via the existing edge with one fewer coupling to test. Adding saga → cohesion directly would be a **fifth** input on a global scalar that already has 4 (`belief`, `unrest`, `micro_cohesion_delta`, `COHESION_DECAY_DIVISOR` decay). Pure duplication of signal. |
| Saga significance → diplomacy peace threshold | **Wrong grain** — diplomacy is pairwise (`faction_relations`, `DIPLOMACY_BASE_CONFLICT_THRESHOLD`); saga is per-entity. Would require a faction-aggregate (the same centroid problem N5 solves for language) with no clear mapping (whose significance wins? a hero of faction A serves as peace-keeper for which pair?). Saga → belief → diplomacy is the natural chain — `BELIEF_PEACE_DIVISOR=50` already turns belief into peace. |
| Saga significance → `state.unrest` | **Conceptually inverted** — the saga is a *cultural-historical* layer; famous figures are an antidote to civilizational despair, not a source of it. Wiring saga → unrest would create a perverse loop where a "great famine" generates famous events that *increase* the unrest that the famine itself generates, contradicting the emergence charter's "narrative binds society" intent. |
| Saga → `chronicle` (HUD) | Observability only; not emergence. Already done in spirit via `EpochDigest::risen`. |
| Saga → trade / per-route price | No unit-typed mapping; emergence trade is settled by N4 topology + N5 friction. |

**Why belief wins:**

1. **Direct identity of subject matter** — the saga records historically significant agents and events; `belief` is the macro "famous-entity-as-faith" resource. The same noun ("significance") maps to the same noun ("faith"). No ontology stretch.
2. **No new state** — `state.belief` already exists, is `#[serde(default)]`-backed, is read by 4 downstream phases, and is the resource the player spends on divine powers. Saga adds one bounded inflow arm to an existing four-arm equilibrium.
3. **Tick ordering is already correct** — `phase_emergence` (line 1668) calls `emergence_legends` *before* `phase_belief` (line 1671). The saga aggregate is **already current** when `phase_belief` runs; no new phase, no new ordering constraint.
4. **The decay term is already there** — `phase_belief` already applies `state.belief / BELIEF_DECAY_DIVISOR` (`engine.rs:2039`). A bounded saga inflow joins the same proportional-decay arm that prevents population worship from running away; no new stability analysis required.
5. **Edge-of-chaos lever** — saga significance is *noisy* (epoch-decayed at `0.9`, plus topology churn from `prune`). Routing that noise into belief before the existing decay produces the small, bounded perturbation the engine needs: enough to feel "a heroic age briefly inflates faith", not enough to dominate `population / 2_000` worship.

---

## 2. Survey — saga + belief state today

### 2.1 Saga data model (`civ_legends`)

| Field | Type | Role |
|-------|------|------|
| `EntityNode.significance` | `f32` (unbounded) | Per-entity rolling decayed score (`graph.rs:281`); `>= promotion_threshold` ⇒ historically significant |
| `EntityNode.promoted` | `bool` | Monotonic — `true` once significance crossed `1.0` |
| `EntityNode.born_epoch` | `Epoch` | First epoch the entity appeared in the graph |
| `IngestOutcome.promoted` | `Vec<LegendEntityId>` | Entities whose significance just crossed the threshold *this event* |
| `LegendsConfig.promotion_threshold` | `f32` (default `1.0`) | Significance floor for promotion |
| `LegendsConfig.decay` | `f32` (default `0.9`) | Per-epoch exponential multiplier |
| `LegendsConfig.ticks_per_epoch` | `u64` (default `64`) | Tick → epoch bucket size |
| `SagaGraph::significant_desc()` | `impl Iterator<Item=LegendEntityId>` | **Descending iteration over `(OrderedF32(score), id)` BTreeSet** — O(1)-per-step via the side index |
| `EpochDigest::risen` | `Vec<EntityRef>` | Entities whose `EventKind::Promotion` was emitted this epoch |

**Significance flow per ingest** (`graph.rs:396`):

```text
delta = magnitude * role.weight() * kind_weight(kind) * reach(eid)
significance += delta
if !promoted && significance >= promotion_threshold { promoted = true; ... }
```

**Per-epoch maintenance** (`worker.rs:46`): every epoch boundary, `decay_epoch()` multiplies every `EntityNode.significance` by `0.9`; `prune()` discards non-promoted entities below `prune_floor=0.01` with no promoted neighbor. Net effect: significance has a fast rise, a slow per-epoch decay, and is **always current** at the moment `phase_belief` reads it.

### 2.2 Belief data model (`engine.rs`)

| Field / fn | Type / shape | Source |
|------------|--------------|--------|
| `WorldState.belief` | `u64` (default `0`, `#[serde(default)]`) | Line 371 |
| `Simulation::belief()` | accessor | Line 1429 |
| `Simulation::add_belief(amount: u64)` | `pub(crate)`, `saturating_add` | Line 1538 |
| `Simulation::try_invoke_divine_power(cost: u64)` | Spender (downstream) | Line 1527 |
| Existing inflow arms | worship = `pop / 2_000`; temple = `temple_level as u64`; hardship = `unrest / 100` (from `phase_unrest`) | Lines 2030-2035, 2076-2077 |
| Existing decay arm | `state.belief -= state.belief / 500` (`BELIEF_DECAY_DIVISOR`) | Line 2039 |
| Downstream consumers | `cohesion_delta` (binds), `diplomacy_conflict_threshold` (peace), `phase_institutions` (temple target), `try_invoke_divine_power` (spender) | Lines 3780, 3905, 2162, 1527 |

`phase_belief` (line 2024) is the **only** belief inflow in the engine hot path. The existing arms are: `worship` (population), `temple_level` (institutional), and `unrest / 100` injected by `phase_unrest` (hardship). All three saturating-add into a global scalar that a proportional decay drains. **No historical, cultural, or narrative signal contributes.**

### 2.3 Tick-order note

`PHASE_ORDER` (`engine.rs:56`) and `tick_with_emergence_source` (line 1641) place the relevant phases in this order:

```text
1668: self.phase_emergence();          // calls emergence_legends() → SagaGraph.ingest
1669: self.phase_research();
1670: self.phase_tech();
1671: self.phase_belief();              // reads state.belief; perfect wiring point
1672: self.phase_unrest();              // injects hardship -> belief (after belief runs)
```

Saga ingest (line 1668) **strictly precedes** belief (line 1671). Saga aggregate is current at the moment of read. No phase reorder needed. The hardship arm lives in `phase_unrest` *after* `phase_belief` so it does not interfere with the same-tick N6 contribution.

### 2.4 Where the `add_belief` helper is already used

| Call site | Role |
|-----------|------|
| `engine.rs:2077` (`phase_unrest`) | `add_belief(unrest / 100)` — hardship arm |
| `engine.rs:5064`, `5176`, `6621` (tests) | Pin belief for assertion |

The pure-fn N6 design returns a `u64` delta that integrates with the same `add_belief` (or direct `state.belief = state.belief.saturating_add(...)`) style.

---

## 3. Gap statement (N6)

| Layer | Evolves | Feeds macro belief / cohesion / diplomacy? |
|-------|---------|---------------------------------------------|
| `SagaGraph.entity_index[].significance` | Yes (per-epoch 0.9 decay + per-event bumps) | **No** — display/HUD only |
| `IngestOutcome.promoted` | Yes (whenever an entity crosses `1.0`) | **No** — narrative feed only |
| `EpochDigest::risen` | Yes (per epoch) | **No** — SLM narrator input only |
| `state.belief` inflow arms | population + temple + hardship | **Yes** — no historical / cultural / narrative arm |

**Charter intent (`docs/guides/emergence-charter.md`):** "… religion EMERGES from named entities and their actions, not from authored deities." The substrate is in place; the macro consumer is missing.

---

## 4. Optimal minimal first coupling

### 4.1 Choice — **bounded top-N significance sum + per-promotion novelty bonus → `state.belief`**

**Mechanism:** Each tick, read the top `SAGA_BELIEF_TOP_N` entities by post-decay significance from `SagaGraph`, sum their scores (hard-capped at `SAGA_BELIEF_SUM_CAP`), divide by `SAGA_BELIEF_DIVISOR`, then add a per-promotion novelty bonus (capped per tick). The result is a `u64` delta added to `state.belief` inside `phase_belief` between the temple contribution and the decay.

**Design stance:** Saga significance is *narrative* signal. It should be a **bounded, decayed, proportional** perturbation on the existing four-arm belief equilibrium — never an unbounded infusion. The hard cap on the sum and the per-tick cap on the promotion bonus are the two safety rails. The existing `BELIEF_DECAY_DIVISOR=500` proportional decay provides the third. The whole contribution is below the floor of population worship (pop/2000 ≈ 500 belief/tick at 1M pop) by design.

### 4.2 Micro signal

**From the saga graph (read-only):**

```text
top_scores: Vec<f32>
  length    ← min(SAGA_BELIEF_TOP_N, significant_set.len())
  values    ← EntityNode.significance for the top-N entities by score
              (SagaGraph::significant_desc, BTreeSet side index, O(top_n))
promotions_this_tick: usize
  ← sum of IngestOutcome.promoted.len() across every ingest call in this tick
    (already collected in emergence_legends; stashed on LegendsWorker)
```

**No new persistent state.** `SagaGraph` keeps the rolling `significance` set already; the engine already knows `IngestOutcome.promoted` per event; the only addition is a `LegendsWorker::last_tick_promotions: usize` accumulator the engine writes each `phase_emergence` and reads once in `phase_belief`.

### 4.3 Macro consumer — pure fn

**Location:** `crates/engine/src/engine.rs` next to `cohesion_delta` / `diplomacy_conflict_threshold`.

```text
/// Top-N significant entities whose rolled-decay scores contribute to belief
/// (FR-CIV-EMERGENCE / FR-CIV-LEGENDS-SIG-05). Small, bounded — the saga arm
/// must be subordinate to population worship so the equilibrium does not invert.
const SAGA_BELIEF_TOP_N: usize = 16;

/// Hard cap on the sum of post-decay significance scores per tick. Even with
/// 16 entities all at significance 1.0 the aggregate saturates here.
const SAGA_BELIEF_SUM_CAP: f32 = 4.0;

/// Aggregate-to-belief divisor. `SAGA_BELIEF_SUM_CAP / SAGA_BELIEF_DIVISOR`
/// is the maximum aggregate contribution per tick.
const SAGA_BELIEF_DIVISOR: u64 = 2;

/// Per-promotion novelty bonus — a famous-figure-rises-to-prominence tick
/// briefly inflates faith the same way a divine-power invocation briefly
/// spends it.
const SAGA_BELIEF_PROMOTION_BONUS: u64 = 25;

/// Cap on the number of promotions that contribute in a single tick; the
/// "founding era" blip cannot run away.
const SAGA_BELIEF_PROMOTION_CAP_PER_TICK: usize = 4;

/// Pure fn: saga significance + per-tick promotion count -> belief delta.
/// Saturating; never returns a negative; bounded by the constants above.
fn saga_significance_belief_delta(
    top_scores: &[f32],
    promotions_this_tick: usize,
) -> u64 {
    let sum = top_scores
        .iter()
        .copied()
        .sum::<f32>()
        .clamp(0.0, SAGA_BELIEF_SUM_CAP);
    let aggregate = (sum / SAGA_BELIEF_DIVISOR as f32) as u64;
    let promo_count = promotions_this_tick.min(SAGA_BELIEF_PROMOTION_CAP_PER_TICK);
    let promotion_bonus = (promo_count as u64) * SAGA_BELIEF_PROMOTION_BONUS;
    aggregate.saturating_add(promotion_bonus)
}
```

**Saga-side helper (new public method on `SagaGraph`, additive / non-breaking):**

```text
impl SagaGraph {
    /// Top-N entity significance scores (post-decay), descending. Read-only,
    /// cheap — uses the `significant_set` BTreeSet side index.
    /// `Vec<f32>` length <= `n`; truncated when fewer entities exist.
    pub fn top_significance(&self, n: usize) -> Vec<f32> {
        self.significant_desc()
            .take(n)
            .filter_map(|id| self.entity(id).map(|e| e.significance))
            .collect()
    }
}
```

**LegendsWorker promotion counter (new public field, additive):**

```text
pub struct LegendsWorker {
    pub graph: SagaGraph,
    last_maintained_epoch: Epoch,
    /// Number of entities newly promoted in the last `ingest` (or sum over
    /// the last `drain` batch). Reset by the engine at the start of each
    /// `phase_emergence` after it has read the value.
    pub last_tick_promotions: usize,
}
```

The engine writes it inside `emergence_legends` (`crates/engine/src/emergence.rs:491`):

```text
fn emergence_legends(&mut self) {
    let tick = self.state.tick;
    let mut tick_promotions: usize = 0;
    let epoch = self.emergence.legends.graph.config.epoch_of(tick);

    for birth in self.last_births().to_vec() { /* ingest + */ tick_promotions += outcome.promoted.len(); }
    for death in self.last_deaths().to_vec() { /* ingest + */ tick_promotions += outcome.promoted.len(); }
    /* … speciation, diplomacy … */

    self.emergence.legends.last_tick_promotions = tick_promotions;
}
```

### 4.4 Sink — single addition in `phase_belief`

```text
fn phase_belief(&mut self) {
    const BELIEF_POP_DIVISOR: u64 = 2_000;
    const BELIEF_DECAY_DIVISOR: u64 = 500;

    let worship = self.state.population / BELIEF_POP_DIVISOR;
    self.state.belief = self.state.belief.saturating_add(worship);
    self.state.belief = self
        .state
        .belief
        .saturating_add(self.state.temple_level as u64);

    // N6 — saga significance → belief (FR-CIV-EMERGENCE / FR-CIV-LEGENDS-SIG-05).
    // Bounded by SAGA_BELIEF_SUM_CAP and SAGA_BELIEF_PROMOTION_CAP_PER_TICK;
    // joins the existing BELIEF_DECAY_DIVISOR proportional decay below.
    let top = self.emergence.legends.graph.top_significance(SAGA_BELIEF_TOP_N);
    let promotions = self.emergence.legends.last_tick_promotions;
    let saga_delta = saga_significance_belief_delta(&top, promotions);
    self.state.belief = self.state.belief.saturating_add(saga_delta);

    self.state.belief = self
        .state
        .belief
        .saturating_sub(self.state.belief / BELIEF_DECAY_DIVISOR);
}
```

**Tuning rationale (constants):**

| Constant | Value | Why |
|----------|-------|-----|
| `SAGA_BELIEF_TOP_N` | `16` | The significant-set side index already exposes O(top_n) read; 16 captures the "headline" entities a populace actually remembers. Larger `n` dilutes the per-entity weight and slows the per-tick collection; smaller `n` makes the arm feel spiky. |
| `SAGA_BELIEF_SUM_CAP` | `4.0` | Per-epoch `0.9` decay bounds the steady-state top-16 sum at ~3–4 for a mature saga. Cap of 4.0 = 1 + 0.9 + 0.81 + 0.73 ≈ "one freshly-promoted + three near-promoted" reading. Higher cap invites run-away when many entities promote in the same epoch. |
| `SAGA_BELIEF_DIVISOR` | `2` | `4.0 / 2 = 2` belief/tick at saturation. With 1M population the existing worship arm yields `1_000_000 / 2_000 = 500` belief/tick. Saga arm is **0.4 % of population worship at saturation** — a small perturbation, not a rival. |
| `SAGA_BELIEF_PROMOTION_BONUS` | `25` | A single promotion (a famous agent's significance just crossed 1.0) yields a 25-belief tick. Within the same magnitude as the test-pinned `add_belief(100)` invocation cost; matches the "small divine-power invocation" feel. |
| `SAGA_BELIEF_PROMOTION_CAP_PER_TICK` | `4` | At most 4 promotions contribute per tick = `4 × 25 = 100` belief/tick from novelty. Combined with the `2` aggregate ceiling → **max 102 belief/tick** from saga, or **~20 % of population worship at 1M pop**. The "founding era" is real but bounded. |
| `BELIEF_DECAY_DIVISOR` (existing) | `500` | Unchanged. The `0.2 %`/tick proportional decay already drains any inflow; the saga arm is at most `102 / 500 ≈ 0.41` extra units of decay/tick in the most extreme case. Equilibrium shift: a "rich saga" epoch lifts steady-state belief by roughly `102 / (1/500) × 0.998 ≈ 51_000` units once the new arm is active. |

**Edge-of-chaos property:** The saga arm has three independent decay / cap mechanisms (per-epoch 0.9 significance decay, hard `SUM_CAP=4.0`, `PROMOTION_CAP_PER_TICK=4`) feeding into the existing `BELIEF_DECAY_DIVISOR=500` proportional decay. A "rich epoch" (many births/deaths/battles) lifts belief for one to two epochs (128–256 ticks) before both the per-epoch significance decay and the belief decay drag it back. A "quiet epoch" (low event rate) drops the contribution to 0 in one epoch. The noise lives at the **epoch scale**, not the tick scale, exactly the cadence the existing `decay_epoch` + `BELIEF_DECAY_DIVISOR` equilibrium is tuned for.

### 4.5 Exact fields touched

| Read | Write |
|------|-------|
| `EmergenceState.legends.graph.entity_index[].significance` (top-N via `significant_desc` side-set) | — |
| `LegendsWorker.last_tick_promotions` | `LegendsWorker.last_tick_promotions` (written in `phase_emergence`, read in `phase_belief`) |
| `state.belief` (read for `saturating_add`) | `state.belief` (saga delta added; existing decay unchanged) |
| — | Downstream consumers (`cohesion_delta`, `diplomacy_conflict_threshold`, `phase_institutions`, `try_invoke_divine_power`) read the *new* `state.belief` value on the **next** tick — same one-tick lag as the existing worship arm. |

**No new persistent `WorldState` fields.** `state.belief` is `#[serde(default)]`-backed; older saves load with belief=0 and the saga arm silently contributes 0 (no `entity_index` yet) — no migration needed. `LegendsWorker::last_tick_promotions` is transient (rebuilt each tick).

**No serde migration.**

### 4.6 Imports (additive)

```text
use civ_legends::SagaGraph;   // top_significance() — new public method
```

The existing `use civ_legends::{...}` block in `crates/engine/src/emergence.rs:21-25` is the single import site; no new crate dependency.

---

## 5. Test specification

### 5.1 Unit test — pure fn shape

**Name:** `saga_significance_belief_delta_bounded_by_caps`  
**File:** `crates/engine/src/engine.rs` `#[cfg(test)]`

```text
// empty saga -> zero contribution
assert_eq!(saga_significance_belief_delta(&[], 0), 0);
assert_eq!(saga_significance_belief_delta(&[], 7), SAGA_BELIEF_PROMOTION_BONUS * 4);

// monotonic in sum (below cap)
let s2 = saga_significance_belief_delta(&[0.5, 0.5], 0);
let s4 = saga_significance_belief_delta(&[1.0, 1.0], 0);
assert!(s4 > s2, "doubling significance doubles the aggregate arm");

// hard cap on sum
let sat = saga_significance_belief_delta(&[4.0_f32; SAGA_BELIEF_TOP_N], 0);
let overshoot = saga_significance_belief_delta(&[16.0_f32; SAGA_BELIEF_TOP_N], 0);
assert_eq!(sat, overshoot, "saga sum is hard-capped at SAGA_BELIEF_SUM_CAP / DIVISOR");

// cap saturates at SAGA_BELIEF_SUM_CAP / SAGA_BELIEF_DIVISOR
assert_eq!(sat, (SAGA_BELIEF_SUM_CAP as u64) / SAGA_BELIEF_DIVISOR);

// promotion bonus caps at SAGA_BELIEF_PROMOTION_CAP_PER_TICK
let promo1 = saga_significance_belief_delta(&[], 1);
let promo4 = saga_significance_belief_delta(&[], 4);
let promo99 = saga_significance_belief_delta(&[], 99);
assert_eq!(promo1, SAGA_BELIEF_PROMOTION_BONUS);
assert_eq!(promo4, SAGA_BELIEF_PROMOTION_BONUS * 4);
assert_eq!(promo99, promo4, "promotion bonus is per-tick capped");

// aggregate + bonus both contribute
let combined = saga_significance_belief_delta(&[1.0, 1.0, 1.0, 1.0], 2);
assert_eq!(
    combined,
    (SAGA_BELIEF_SUM_CAP as u64 / SAGA_BELIEF_DIVISOR)
        + 2 * SAGA_BELIEF_PROMOTION_BONUS
);
```

### 5.2 Unit test — saga-side helper

**Name:** `saga_top_significance_descending_and_truncated`  
**File:** `crates/legends/src/graph.rs` `#[cfg(test)]`

```text
let mut g = SagaGraph::new(LegendsConfig::default());
// insert 20 entities with descending significance
for i in 0..20 {
    let eid = g.resolve_aggregate(
        AggregateKey { kind: EntityKind::Agent, a: ClusterId(i), b: ClusterId(i), start_bucket: 0 },
        Epoch(0),
    );
    g.bump_significance(eid, (20 - i) as f32 * 0.1);
}

let top5 = g.top_significance(5);
assert_eq!(top5.len(), 5);
assert!(top5.windows(2).all(|w| w[0] >= w[1]), "descending order");
assert!(top5[0] > top5[4], "top-1 > top-5");

let top100 = g.top_significance(100);
assert_eq!(top100.len(), 20, "truncated to entity count");
```

### 5.3 Integration test — saga arm wired into `phase_belief`

**Name:** `phase_belief_includes_saga_significance_arm`  
**File:** `crates/engine/src/engine.rs` `#[cfg(test)]`

**Setup:**

1. `Simulation::with_seed(7)`; tick once to populate `state.tick` and run `phase_emergence` baseline.
2. Pin `state.population = 0`, `state.temple_level = 0`, `state.unrest = 0` so the **only** belief inflow is the saga arm.
3. **Case A — empty saga:** clear `self.emergence.legends.graph` (re-instantiate a fresh `SagaGraph`); set `last_tick_promotions = 0`. Call `phase_belief()`. Snapshot `state.belief`.
4. **Case B — saturated saga:** insert 16 `Agent` entities with `significance = 1.0` (the cap). Set `last_tick_promotions = 4`. Call `phase_belief()`. Snapshot `state.belief`.

**Assert (with `BELIEF_POP_DIVISOR=2_000` and `BELIEF_DECAY_DIVISOR=500`):**

```text
// Case A: 0 worship + 0 temple + 0 saga aggregate + 0 promotion bonus
//         - 0 decay (state.belief = 0) == 0
let belief_a = sim.state.belief;
assert_eq!(belief_a, 0, "no inflow, no belief");

// Case B: 0 worship + 0 temple + 2 belief (aggregate cap) + 100 belief (promotion cap)
//         - decay on 102 = 0
let belief_b = sim.state.belief;
assert_eq!(
    belief_b,
    (SAGA_BELIEF_SUM_CAP as u64 / SAGA_BELIEF_DIVISOR)
        + SAGA_BELIEF_PROMOTION_BONUS * SAGA_BELIEF_PROMOTION_CAP_PER_TICK as u64,
    "saga arm contributes bounded aggregate + capped promotion bonus"
);
assert!(
    belief_b > belief_a,
    "saga arm is non-zero only when saga entities exist"
);
```

**Control — saga arm decays with significance:**

5. **Case C — decayed saga:** insert 16 entities with `significance = 1.0`, then call `g.decay_epoch()` 5 times (significance = 0.59049). Re-arm `last_tick_promotions = 4`. Call `phase_belief()`.

```text
let belief_c = sim.state.belief;
assert!(
    belief_c < belief_b,
    "per-epoch 0.9 decay reduces the saga aggregate arm within 5 epochs"
);
```

**Control — saga arm = 0 when no entities exist** is covered by Case A.

---

## 6. What N6 v1 does *not* do

| Deferred | Rationale |
|----------|-----------|
| Saga → cohesion direct | Belief is the canonical input to `cohesion_delta(belief, unrest)`; saga → belief → cohesion is the natural chain. Adding a direct saga → cohesion input duplicates the signal. |
| Saga → diplomacy peace threshold | Diplomacy is pairwise; saga is per-entity. Would need faction-aggregate (N5-style centroids). Saga → belief → `BELIEF_PEACE_DIVISOR` already routes correctly. |
| Saga → unrest | Conceptually inverted; would create a perverse "famous famine" loop. |
| Saga → `chronicle` lines | Already present in spirit via `EpochDigest::risen` (read-only SLM narrator input). |
| Saga → trade-route price | N4 + N5 scope; no clean unit-typed mapping. |
| Saga → `state.temple_level` (institution level) | Indirectly via belief → `phase_institutions` (line 2162: `institution_target_level(state.belief, 5_000)`). |
| Saga-named deity mint | FR-CIV-LEGENDS-NARRATOR-13 / ai-rnd §1.1 (SLM narrator); not the engine's macro consumer. |
| Per-faction belief (`polities[*].belief`) split | `state.belief` is a global scalar in v1; the `PolityMacroState.belief` field at line 317 is a per-polity mirror without independent inflow. Saga routing stays global. |
| Persistence of `last_tick_promotions` across saves | Counter is transient; rebuilt on the first `phase_emergence` after load. |
| Variable per-entity weight by `Role::weight()` (already in `bump_significance`) | The significance score is *already* the integrated role weight; routing it raw is correct. |

---

## 7. Tick-order DAG (N6 slice)

```mermaid
flowchart LR
  PE_prev[phase_emergence T-1<br/>significance drift]
  PE_legends[phase_emergence T<br/>emergence_legends → SagaGraph.ingest<br/>accumulate last_tick_promotions]
  PE_belief[phase_belief T<br/>worship + temple + saga_significance_belief_delta - decay]
  PE_unrest[phase_unrest T<br/>hardship → add_belief]
  PE_cohesion[phase_cohesion T<br/>cohesion_delta(belief, unrest)]
  PE_diplomacy[phase_diplomacy T<br/>diplomacy_conflict_threshold(belief, unrest)]
  PE_inst[phase_institutions T<br/>institution_target_level(belief, 5_000)]
  PE_prev --> PE_legends --> PE_belief --> PE_unrest --> PE_cohesion --> PE_diplomacy --> PE_inst
```

**Depends on:** nothing (the saga aggregate is already current at `phase_emergence`; `phase_belief` is the only consumer).  
**Composes with:** all four existing belief inflow arms (worship / temple / hardship) and all four downstream consumers (cohesion / diplomacy peace / temple target / divine-power spender).

---

## 8. Phased WBS (follow-on)

| Phase | Task ID | Description | Depends on |
|-------|---------|-------------|------------|
| 1 | **N6-A** | `saga_significance_belief_delta` pure fn + constants + unit tests (`crates/engine/src/engine.rs`) | — |
| 2 | **N6-B** | `SagaGraph::top_significance(n)` public method + unit test (`crates/legends/src/graph.rs`) | — |
| 3 | **N6-C** | `LegendsWorker::last_tick_promotions` field + `emergence_legends` accumulator in `crates/engine/src/emergence.rs` | — |
| 4 | **N6-D** | Wire N6-A + N6-B + N6-C into `phase_belief` (single `saturating_add` block) | N6-A, N6-B, N6-C |
| 5 | **N6-D-int** | `phase_belief_includes_saga_significance_arm` integration test | N6-D |
| 6 | N6-E | `emergence_feed` entry for "saga belief inflow" (HUD readout) | N6-D |
| 7 | N6-F | Saga → `state.temple_level` indirect observability + chronicle line | N6-D |
| 8 | N6-G | Saga → per-polity belief (split `polities[*].belief`) | N6-D, M5 polity split |

**Agent effort (aggressive):** N6-A through N6-D-int ≈ 6–10 tool calls, ~3–4 min wall clock. Smaller than N4 (8–12) and N5 (8–12) because no new persistent state, no new phase, no serde migration.

---

## 9. Cross-project reuse

| Candidate | Location | Notes |
|-----------|----------|-------|
| `SagaGraph::significant_desc()` | `crates/legends/src/graph.rs:651` | Already iterates the top-N via the `(OrderedF32, LegendEntityId)` BTreeSet side index; N6-B wraps it |
| `IngestOutcome.promoted` | `crates/legends/src/graph.rs:39` | Per-event promotion set; N6-C sums across the tick's ingest calls |
| `add_belief` helper | `crates/engine/src/engine.rs:1538` | `pub(crate)`, `saturating_add` — used by `phase_unrest` (line 2077); the N6 sink can use either `add_belief(saga_delta)` (consistent with hardship arm) or direct `state.belief = state.belief.saturating_add(saga_delta)` (consistent with the worship/temple arms in `phase_belief`). Either is charter-clean; recommend the direct form to keep the four inflow arms visually grouped in `phase_belief`. |
| `saga_significance_belief_delta` | `crates/engine/src/engine.rs` next to `cohesion_delta` / `diplomacy_conflict_threshold` | Symmetric pure-fn API shape with the other belief-coupling helpers |
| `BELIEF_DECAY_DIVISOR = 500` | `crates/engine/src/engine.rs:2029` | Reused unchanged — N6 leans on the existing proportional decay to keep the saga arm bounded |

---

## 10. References

| Artifact | Path |
|----------|------|
| Saga ingest, `IngestOutcome.promoted`, `significant_desc` | `crates/legends/src/graph.rs` |
| `LegendsConfig.decay`, `ticks_per_epoch`, `promotion_threshold` | `crates/legends/src/config.rs` |
| `LegendsWorker`, per-epoch maintenance | `crates/legends/src/worker.rs` |
| `EntityNode.significance`, `promoted`, `born_epoch` | `crates/legends/src/model.rs` |
| `phase_emergence` → `emergence_legends` | `crates/engine/src/emergence.rs:135`, `:491` |
| `phase_belief` (wiring point) | `crates/engine/src/engine.rs:2024` |
| `WorldState.belief`, `add_belief`, `try_invoke_divine_power` | `crates/engine/src/engine.rs:371`, `:1538`, `:1527` |
| `cohesion_delta`, `diplomacy_conflict_threshold` (downstream pure fns) | `crates/engine/src/engine.rs:3779`, `:3904` |
| `PHASE_ORDER` | `crates/engine/src/engine.rs:56` |
| Emergence charter | `docs/guides/emergence-charter.md` |
| Legends design spec | `docs/design/legends-engine.md` |
| N4 (deferred N4-LEG note) | `N4_COUPLING_SPEC.md` §1, §6 |
| N5 (parallel language coupling) | `N5_LANGUAGE_SPEC.md` |
| FR-CIV-EMERGENCE (divine-powers economy) | `docs/specs/requirements/FR-CIV-EMERGENCE.md` |
| FR-CIV-LEGENDS-SIG-05 (significance scoring) | `docs/specs/requirements/FR-CIV-LEGENDS.md` |

---

## 11. Summary

**Gap:** `SagaGraph` accumulates a measured, decayed, causally-linked history of significant agents and events; `state.belief` is the macro resource for "famous figures made into faith". The two layers have never met — the saga is a HUD silo, belief is a population-worship silo.

**Minimal closure:** Add a bounded, top-N significance sum + per-promotion novelty bonus to `state.belief` inside the existing `phase_belief` between the temple contribution and the `BELIEF_DECAY_DIVISOR=500` decay. The pure fn `saga_significance_belief_delta(top_scores, promotions_this_tick)` is hard-capped at `SAGA_BELIEF_SUM_CAP=4.0` and `SAGA_BELIEF_PROMOTION_CAP_PER_TICK=4`, so the saga arm contributes at most ~102 belief/tick — bounded, edge-of-chaos, **and subordinate to population worship** (~500 belief/tick at 1M pop). One new public method on `SagaGraph` (`top_significance(n)`) and one new public field on `LegendsWorker` (`last_tick_promotions`); no `WorldState` field added, no serde migration, no new phase, no phase reordering.

**Not chosen instead:** Saga → cohesion (redundant — belief already feeds cohesion); saga → diplomacy (wrong grain — pairwise vs per-entity); saga → unrest (conceptually inverted — fame binds society, not frays it).

**Test anchor:** Empty saga + zero population/temple/unrest → belief = 0. Saturated saga (16 entities at `significance=1.0`) + 4 promotions → belief = `4.0/2 + 4*25 = 102` per tick, with `(state.belief / 500) ≈ 0` decay on the same tick. Five per-epoch decays → aggregate arm drops; promotion bonus still capped.
