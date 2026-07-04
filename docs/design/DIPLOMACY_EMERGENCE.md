# Emergent Diplomacy Design

> Companion to [ADR-015](../adr/ADR-015-faction-emergence-via-k-means-ideology-clustering.md)
> (faction emergence via k-means) and [ADR-020](../adr/ADR-020-wire-dormant-emergence-phases.md)
> (wired emergence phases). This document specifies how **diplomacy itself
> emerges** from the same substrate as factions — k-means ideology distance
> + resource competition + the wired emergence phases — with **no scripted
> diplomacy**.

**Status.** Design proposal (research branch `research/diplomacy-emergence-design`).
**No code in this PR.** The mechanical wiring lands in a follow-up engine PR
modelled on `EMERGENCE_WIRING_PATCHPLAN.md`.

---

## 1. Scope and non-goals

**In scope.**

- The dataflow from k-means cluster centroids + macro scalars from the
  11 wired emergence phases (per ADR-020 / `EMERGENCE_WIRING_PATCHPLAN.md`)
  into the six-driver `DiplomacySignal` already declared in
  `crates/agents/src/diplomacy.rs:34-53`.
- The thresholds that lift the continuous score from
  `crates/agents/src/diplomacy.rs:161-173` into qualitative
  `RelationKind` (`Alliance` / `Trade` / `Neutral` / `Rivalry` / `War`).
- A `treaty` substrate that emerges from sustained `Alliance` /
  `Rivalry` / `War` states — proposals, terms, and breakage, **all
  emergent**, never authored.
- Diagnostics: per-pair signal provenance, replay-bus events for
  every relation transition, entropy target from
  `crates/agents/src/diplomacy.rs:244-270`.
- Determinism, cap-table, and test minimums that match the
  `EMERGENCE_WIRING_PATCHPLAN` recipe.

**Out of scope (explicit non-goals).**

- **No scripted diplomacy.** No "you are an ally of X", no "you owe
  tribute to Y", no "you sign a non-aggression pact with Z because the
  designer said so". Every relation transition has a signal-level
  cause that is traceable to k-means distance, resource competition,
  or one of the wired emergence scalars.
- **No leader/monarch/face models.** The diplomacy substrate operates
  on emergent clusters, not individuals.
- **No narrative treaty tree.** Treaties are observed side-effects of
  sustained relation states, not authored content.
- **No full mod API expansion.** The `.civmod` /
  `mod.loaded.v1` / `wasmtime` ticks pipeline from the
  `civ-diplomacy` AGENTS.md status row is out of scope; this design
  feeds it, not the other way around.
- **No combat targeting rewrite.** `phase_tactics` /
  `last_tick_engagements` are inputs (via `GriefAccumulator`), not the
  rewrite target.

**Cross-references.**

- ADR-015 — k-means ideology clustering is the faction primitive.
- ADR-020 — the 11 dormant phases whose macros this design consumes.
- `EMERGENCE_WIRING_PATCHPLAN.md` §3.1–3.11 — phase signatures and
  DAG ordering.
- `crates/agents/src/diplomacy.rs` — the existing six-driver
  `DiplomacySignal` and `DiplomacyMatrix` substrate (unchanged).
- `crates/engine/src/faction_emergence.rs` — the existing
  k-means producer (unchanged).
- FR-3D matrix (`docs/traceability/fr-3d-matrix.md`) — the
  DIPLO sub-matrix this design unblocks.

---

## 2. Design constraints (why this shape)

### 2.1 Factions are emergent, so diplomacy must be emergent

ADR-015 establishes that faction membership is **derived from**
ideology vectors, not assigned by the designer. If diplomacy were
authored ("nation X allies with nation Y"), the system would have
two sources of truth — emergent factions vs scripted diplomatic ties
— and the user-visible relation would diverge from the underlying
cluster geometry the moment clusters shift.

Therefore every diplomacy signal that crosses a pair of clusters
**must be reproducible from the same inputs that produced the
clusters**: ideology vectors (k-means inputs), resource stocks
(economy outputs), grievance (combat outputs), and the wired macro
scalars (belief, unrest, cohesion, social mood, stratification,
institutions, economic focus).

### 2.2 Diplomacy must fit inside the 23-entry `PHASE_ORDER`

`EMERGENCE_WIRING_PATCHPLAN §1` fixes `phase_diplomacy` at slot 7
(between `phase_economy` and `phase_tactics`), **strictly before**
the 11 new macro phases. This is a non-negotiable DAG constraint:

- The macro phases `phase_belief`, `phase_unrest`, `phase_cohesion`,
  `phase_social_mood`, `phase_stratification`, `phase_institutions`,
  `phase_economic_focus` do not exist yet at slot 7, so diplomacy
  reads stale-allowed values from the **previous tick's**
  `state.belief` / `state.unrest` / `state.cohesion` /
  `state.society_mood` / `state.dispossessed_permille` /
  `state.economic_focus` — the same single-tick-lag pattern the
  emergence patch plan calls out as acceptable for slow-moving
  aggregates (see `EMERGENCE_WIRING_PATCHPLAN.md §3.5`).
- `phase_emergence` (slot 22) is the orchestrator and is the
  canonical writer of cluster centroids; its output from the
  **previous tick** is the input to `phase_diplomacy` on the current
  tick.
- `phase_tactics` (slot 8, immediately after `phase_diplomacy`)
  consumes `diplomacy_events` and `DiplomacyMatrix.relation(...)`
  to influence combat targeting. This is the only downstream
  consumer in the new pipeline.

### 2.3 Diplomacy must inherit the macro cap-table

`EMERGENCE_WIRING_PATCHPLAN §7` fixes the grep-able cap-table that
guards every bounded scalar writer. Diplomacy introduces **5 new
caps** (one per signal weight class plus a per-tick relation-flip
cap); all named consts live alongside the existing
`MAX_AWAKENING_*` / `MAX_RESEARCH_PER_TICK` / etc. so the review-time
guard works unchanged.

### 2.4 Diplomacy must be replay-deterministic

ADR-003 + the existing `test_determinism` regression require two
same-seed sims to remain byte-identical. Diplomacy drift is the
single largest non-determinism risk in this design because the
six-driver signal sum can be sensitive to the order of pair
updates. The fix is: **process every pair in canonical
`(min(cluster_id), max(cluster_id))` order** every tick, identical
to the existing `crates/agents/src/diplomacy.rs:153-159` key
contract.

---

## 3. Substrate: the existing six-driver `DiplomacySignal`

For continuity, the signal substrate is unchanged. We document the
contract here so the rest of this design can reference it without
re-explaining.

```text
DiplomacySignal (crates/agents/src/diplomacy.rs:33-53)
├── resource_competition : f32   — shared-resource pressure (push -)
├── trade_volume         : f32   — exchange intensity       (push +)
├── proximity            : f32   — proximity pressure       (push -)
├── combat_grievance     : f32   — accumulated grievance    (push -)
├── need_complementarity : f32   — surplus ↔ deficit fit    (push +)
└── scarcity_pressure    : f32   — energy scarcity         (push - if +)

Weights (crates/agents/src/diplomacy.rs:209-215):
  W_TRADE      = 0.08
  W_COMPETE    = 0.12
  W_BORDER     = 0.04
  W_GRIEVANCE  = 0.18
  W_COMPLEMENT = 0.06
  W_SCARCITY   = 0.10
  W_RELAX      = 0.01   (per-tick neutral relaxation)

Drift equation (crates/agents/src/diplomacy.rs:224-232):
  score(t+1) = ((score(t) + drift) * (1 - W_RELAX)).clamp(-1, 1)

Thresholds (crates/agents/src/diplomacy.rs:161-173):
  score ≥ +0.60  → Alliance
  score ≥ +0.20  → Trade
  -0.20 < score < +0.20 → Neutral
  score ≤ -0.20  → Rivalry
  score ≤ -0.60  → War

Entropy (crates/agents/src/diplomacy.rs:244-270):
  target operating range [1.5, 2.1]  (Shannon over 5 buckets)
```

This substrate is **the consumer of every signal in this design**.
Nothing in this document changes it.

---

## 4. The producer: emergent signals from k-means + wired phases

`phase_diplomacy` reads six sources and folds each into the matching
`DiplomacySignal` field. Each source is **already a real producer**
(existing engine code) or **lands as part of ADR-020** — none of
the six is a new wiring decision.

### 4.1 `resource_competition` — shared-resource pressure (CIV-007 §1.3, slot 1 of 6)

**Source.** `crates/economy` overlap map (existing; consumed by
`phase_diplomacy` slot 7 already at
`crates/engine/src/engine.rs:1281-1850`).

**Definition.**

```text
shared_res(a, b) = |resources_a ∩ resources_b|     (goods that both clusters draw from)
overlap(a, b)    = min(consume_a, consume_b) summed over shared_res
                  ─────────────────────────────────────────────────────
                       max(consume_a, consume_b) summed over shared_res

resource_competition(a, b) = overlap(a, b) * shared_res(a, b).len() / k_SHARE_CAP
```

Where `k_SHARE_CAP = 4` (a cluster sharing four or more goods with
its pair maxes out the signal at 1.0 — no per-cluster tuning
required). `consume_a` is the cluster's per-tick withdrawal from
each shared good as recorded in `state.faction_resources` /
cluster projection.

**Cap (new).** `MAX_RESOURCE_COMPETITION = 1.0` (signal is
already pre-clamped; const declared for grep parity with
`EMERGENCE_WIRING_PATCHPLAN §7`).

**Push direction.** Negative (resists cooperation).

### 4.2 `trade_volume` — exchange intensity (slot 2 of 6)

**Source.** `state.trade_routes` (existing) — projected to
cluster-pair keys by the existing
`crates/engine/src/emergence.rs:194-213` cluster projection.

**Definition.**

```text
trade_volume(a, b) = Σ flow(c)  for c in active_routes(this_tick)
                                where c.origin in cluster_a
                                  and c.dest   in cluster_b
trade_volume(a, b) = min(MAX_TRADE_VOLUME,
                         trade_volume(a, b) / TRADE_NORMALIZER)
```

Where `TRADE_NORMALIZER = 100` (one full trade-route cycle of 100
units of flow saturates the signal at 1.0 — keeps the signal
unit-agnostic across economic regimes).

**Cap (new).** `MAX_TRADE_VOLUME = 1.0` (pre-clamped; const for
grep parity).

**Push direction.** Positive (pulls toward `Trade` / `Alliance`).

### 4.3 `proximity` — spatial pressure (slot 3 of 6)

**Source.** Voronoi-cell overlap of cluster territory polygons
(existing projection in `crates/engine/src/cluster_geom.rs`,
projected onto `ClusterId`).

**Definition.**

```text
proximity(a, b) = min(1.0, shared_border_length(a, b) / BORDER_SATURATION)
```

Where `BORDER_SATURATION = 64` tiles (a 64-tile shared border —
approximately one edge of a 16×16 settlement footprint — maxes
the signal at 1.0). Below 16 tiles shared border the signal is
below 0.25 and contributes almost nothing — emergent pairs that
have never touched edges stay `Neutral`.

**Cap (new).** `MAX_PROXIMITY = 1.0` (pre-clamped; const for grep
parity).

**Push direction.** Negative (mildly; the `W_BORDER = 0.04` weight
in `crates/agents/src/diplomacy.rs:212` is deliberately small — see
§4.4 for the strong negative signal that does the real war-driving).

### 4.4 `combat_grievance` — accumulated grievance (slot 4 of 6)

**Source.** Existing `GriefAccumulator`
(`crates/agents/src/diplomacy.rs:55-109`) — fed by
`last_tick_engagements` at `crates/engine/src/engine.rs:1178-1211`
slot 8 (`phase_tactics`). **Cross-tick persistence** is what makes
grievance the strongest negative signal.

**Definition.**

```text
grievance(a, b) = GriefAccumulator.get(a, b)
                  ───────────────────────────────
                       GRIEVANCE_NORMALIZER

combat_grievance(a, b) = clamp(0.0, MAX_COMBAT_GRIEVANCE, normalized)
```

Where `GRIEVANCE_NORMALIZER = 0.20` (≈10 successive one-sided
engagements saturate the signal) and
`MAX_COMBAT_GRIEVANCE = 1.0`.

The decay is handled **inside** `GriefAccumulator` (`tick_decay` at
`crates/agents/src/diplomacy.rs:87-95`) — a per-tick
`(1 - decay_rate) = 0.97` multiplier with `decay_rate = 0.03`. This
is the cross-tick hysteresis that turns a single battle into a
multi-tick war: a pair that fought once will sit at
`combat_grievance ≈ 0.10` for ~30 ticks before it falls below the
0.02 noise floor (the `1e-5` zero-trip at
`crates/agents/src/diplomacy.rs:91-93`).

**Push direction.** Strongly negative — the `W_GRIEVANCE = 0.18`
weight is the single largest in
`crates/agents/src/diplomacy.rs:209-215`, and `decay_rate = 0.03` +
`engagement_weight = 0.02` match the existing
`crates/agents/src/diplomacy.rs:73-79` defaults.

### 4.5 `need_complementarity` — surplus/deficit match (slot 5 of 6)

**Source.** `state.faction_resources` (existing). Computed per
cluster via the same projection that powers §4.2.

**Definition.** For each good `g`:

```text
net_a(g) = stock_a(g) - target_a(g)
net_b(g) = stock_b(g) - target_b(g)
```

A good `g` contributes to complementarity when its signs differ
(one cluster has surplus, the other has deficit):

```text
complementarity(a, b) = Σ |net_a(g) - net_b(g)|
                        where sign(net_a(g)) ≠ sign(net_b(g))
                        ──────────────────────────────────────────
                                    NEED_COMPLEMENT_CAP

need_complementarity(a, b) = min(1.0, complementarity_raw)
```

Where `NEED_COMPLEMENT_CAP = 100` units (saturates the signal when
two clusters have ~100-unit divergence across matched surplus/
deficit goods). The cap is deliberately smaller than the
`MAX_NEED_COMPLEMENT = 1.0` outer clamp — the latter is a
defensive double-guard so the inner const can change without
breaking determinism.

**Push direction.** Positive (latent cooperation pull).

### 4.6 `scarcity_pressure` — energy-scarcity gradient (slot 6 of 6)

**Source.** `state.energy_budget_joules` /
`state.faction_resources` (existing). Two-sided: it is **the only
signal that can push relations in either direction** depending on
whether the pair is energy-poor or energy-mismatched.

**Definition.**

```text
energy_a = Σ energy stocks of cluster_a
energy_b = Σ energy stocks of cluster_b
need_a   = population_a * ENERGY_PER_CAPITA
need_b   = population_b * ENERGY_PER_CAPITA

ratio_a = energy_a / max(1, need_a)
ratio_b = energy_b / max(1, need_b)

both_scarce = (ratio_a < 1.0) and (ratio_b < 1.0)
one_surplus = ((ratio_a ≥ 1.5) and (ratio_b < 1.0))
              or
              ((ratio_b ≥ 1.5) and (ratio_a < 1.0))

scarcity_pressure(a, b) = +0.5  if one_surplus  (pulls toward cooperation)
                        = -0.5  if both_scarce (sharpens competition)
                        =  0.0  otherwise
```

The sign convention matches the existing comment at
`crates/agents/src/diplomacy.rs:49-52` — a negative
`scarcity_pressure` becomes a *positive* drift term in the
`apply_signal` equation (`crates/agents/src/diplomacy.rs:228`).

**Cap (new).** `MAX_SCARCITY_PRESSURE = 1.0` (defensive outer
clamp; the inner ±0.5 is the actual saturation point — leaving
headroom for follow-up refinements to use a continuous gradient
without breaking determinism).

**Push direction.** Bidirectional (only six-driver signal that
does so).

### 4.7 Summary of new consts

```text
MAX_RESOURCE_COMPETITION     = 1.0
MAX_TRADE_VOLUME             = 1.0
MAX_PROXIMITY                = 1.0
MAX_COMBAT_GRIEVANCE         = 1.0
MAX_NEED_COMPLEMENT          = 1.0
MAX_SCARCITY_PRESSURE        = 1.0
TRADE_NORMALIZER             = 100
BORDER_SATURATION            = 64
GRIEVANCE_NORMALIZER         = 0.20
NEED_COMPLEMENT_CAP          = 100
ENERGY_PER_CAPITA            = 10        (joules / person / tick, to be calibrated)
MAX_RELATION_FLIPS_PER_TICK  = 2         (see §6.3)
```

All declared in `crates/agents/src/diplomacy.rs` alongside the
existing `W_*` weights, **not** in `engine.rs` — this keeps the
diplomacy substrate self-contained and lets the producer be unit-
tested in isolation.

---

## 5. From signals to relations: emergent alliances/wars/treaties

### 5.1 The drift equation is already wired

The drift equation in `crates/agents/src/diplomacy.rs:224-232`
operates on every `(ClusterId, ClusterId)` pair. The thresholds in
`crates/agents/src/diplomacy.rs:161-173` lift the continuous score
into a qualitative `RelationKind`. **Nothing in this design changes
that equation** — we only feed it better inputs (§4) and reason
about its outputs more carefully (§5.2–5.5).

### 5.2 Alliances emerge from sustained positive score

The `Alliance` threshold is `score ≥ 0.60`. To reach it from
`Neutral`, a pair needs a combination of:

- `trade_volume` sustained for ~10 ticks (push +
  `0.08 × 1.0 = 0.08` per tick against a 0.01 relaxation pull),
  OR
- A negative `scarcity_pressure` sustained for ~30 ticks (one
  cluster consistently carrying the other's energy deficit), OR
- Strong complementarity on multiple goods for ~12 ticks, OR
- All three at lower magnitudes.

This is the **emergent alliance**: no player input, no script,
just the cumulative weight of macro signals across the wired
phases.

**Test minimum (ADR-011).** 3 tests:
`phase_diplomacy_emerges_alliance_on_sustained_trade`,
`phase_diplomacy_emerges_alliance_on_energy_surplus`,
`phase_diplomacy_emerges_alliance_on_complementarity`.

### 5.3 Wars emerge from sustained negative score

The `War` threshold is `score ≤ -0.60`. The dominant pathway is:

- 3+ cross-border engagements in a row feed
  `GriefAccumulator.add_engagement` at
  `crates/agents/src/diplomacy.rs:98-102`
  (`+0.02` per engagement), saturating `combat_grievance ≈ 0.10`
  within 5 engagements, and pushing
  `score` by `0.18 × 0.10 = 0.018` per tick **against** an
  `Alliance`-class opposite signal.
- Cross-tick hysteresis (the `decay_rate = 0.03` per tick) means
  the war signal persists for ~50 ticks after the last engagement.
- `resource_competition` (clusters fighting over the same food/
  timber tiles) feeds in continuously while the territory overlap
  holds.

**Test minimum (ADR-011).** 3 tests:
`phase_diplomacy_emerges_war_on_sustained_grievance`,
`phase_diplomacy_emerges_war_on_resource_competition`,
`phase_diplomacy_war_persists_after_engagement_stops` (decay
hysteresis).

### 5.4 Treaties: emergent side-effects of sustained `Alliance` / `Rivalry` / `War`

**Critical design point.** A treaty is **not** an entity. It is a
**transition record** observed by the replay bus when a `RelationKind`
enters or leaves `Alliance`, `Rivalry`, or `War`. The terms of the
treaty (if any) are derived from the per-pair signal provenance
stored on `RelationRecord` (see §6.1).

The five treaty subtypes:

| Subtype | Trigger | Terms (derived) |
|---------|---------|-----------------|
| `treaty.alliance.v1` | `Neutral → Alliance` | duration = current `relation_score / 0.04` ticks; signatories = pair cluster ids |
| `treaty.trade.v1` | `Neutral → Trade` | terms = `need_complementarity` snapshot (surplus/deficit per good); trade-route capacity ceiling = `trade_volume × 100` |
| `treaty.rivalry.v1` | `Neutral → Rivalry` | terms = `proximity × 64` shared-border tiles; resource-claim resolution = "first-claim-wins" (existing economy priority) |
| `treaty.war.v1` | `Neutral → War` (or `Rivalry → War`) | terms = `combat_grievance` snapshot; engagement limit = `2 × MAX_GRIEVANCE_NORMALIZED` per tick before `phase_tactics` de-prioritises further engagements |
| `treaty.broken.v1` | `Alliance → Trade` (or below), `War → Rivalry` (or below) | breakage cause = dominant signal at transition; per-good terms released |

**Why treaties are not entities.** If treaties were entities they
would need an authored lifecycle (propose / accept / sign / break).
That violates §1 ("no scripted diplomacy"). Treating them as
replay-bus observations of emergent relation transitions makes them
**observable from the outside** (UI, mod API, replay tools) without
making them a first-class simulated object.

**Replay-bus events.** 5 new `ReplayEvent` variants,
sibling to `record_belief` /
`record_cohesion` from `EMERGENCE_WIRING_PATCHPLAN §6`:

| Method | Tick stamp | Payload |
|--------|-----------|---------|
| `record_treaty(tick, subtype, pair, terms_json)` | yes | (subtype: enum, pair: (u32, u32), terms_json: String ≤ 256 bytes) |

Where `terms_json` is the compact serialised `terms` snapshot from
the table above — bounded so replay files stay compact (the
`civ-replay` size budget is `~80 KB / 1k ticks`).

**Test minimum (ADR-011).** 3 tests:
`phase_diplomacy_emits_treaty_alliance_v1_on_threshold_cross`,
`phase_diplomacy_emits_treaty_war_v1_on_threshold_cross`,
`phase_diplomacy_terms_json_bounded_under_256_bytes`.

### 5.5 Why "emergent" beats "scripted" here

A scripted alliance (designer-authored "nation X likes nation Y")
has three observable defects:

1. It **cannot adapt** to cluster shifts — when the k-means pass
   re-assigns half of nation X's population to nation Z (because a
   religion wave sweeps the cluster geometry), the alliance string
   still reads "X–Y" while the population no longer believes it.
2. It **diverges from the cluster geometry** that the simulation
   actually evolves — a UI showing the alliance as a ribbon line
   gives the user a wrong picture of who is trading with whom.
3. It **bypasses the FR-DIPLO entropy target** of `[1.5, 2.1]`
   from `crates/agents/src/diplomacy.rs:243-244` — authored
   alliances cluster at the `Alliance` bucket and depress entropy
   below the operating range.

The emergent pipeline keeps the alliance, the war, the treaty, and
the cluster geometry all derivable from the same k-means inputs +
wired phases. If clusters re-shape, relations re-shape. If
relations look "wrong" to the user, the user can inspect
`RelationRecord.terms_json` to see which signals drove the
transition.

---

## 6. Diagnostics and observability

### 6.1 Per-pair signal provenance

Extend `RelationRecord` (`crates/agents/src/diplomacy.rs:111-127`)
with one **bounded** diagnostic field:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationRecord {
    pub score: f32,
    pub samples: u32,
    /// Last-tick signal snapshot (LIFO of last 8 ticks, ring buffer).
    /// Lets the UI answer "why is X allied with Y?" without re-deriving.
    pub signal_history: [DiplomacySignal; 8],
    pub signal_history_head: u8,   // ring buffer cursor in [0, 8)
}
```

The ring buffer is bounded at 8 entries (≈80 bytes per pair) so
diplomacy memory stays O(pairs) and does not grow with tick count.
The ring is **deterministic**: every pair writes to slot
`head % 8` and increments, so two same-seed sims produce identical
ring contents.

### 6.2 Replay-bus relation events

Add 4 new `ReplayEvent` variants on `ReplayLog` (sibling to
`record_treaty` from §5.4 and the macro events from
`EMERGENCE_WIRING_PATCHPLAN §6`):

| Method | Tick stamp | Payload |
|--------|-----------|---------|
| `record_relation_score(tick, pair, score)` | yes | (pair: (u32, u32), score: f32) |
| `record_relation_kind_transition(tick, pair, before, after)` | yes | (pair, RelationKind before/after) |
| `record_diplomacy_entropy(tick, h)` | yes | (h: f32) |
| `record_diplomacy_event(tick, label)` | yes | (label: String ≤ 64 bytes) |

The `record_diplomacy_event` is the **only escape hatch** for
human-readable labelling of significant transitions ("First
contact", "Border skirmish escalated", "Trade pact observed"). It
is **opt-in** (default empty), only the mod dev / scenario harness
sets it, and the label is bounded so it cannot be used to smuggle
scripted diplomacy into the replay.

### 6.3 Per-tick relation-flip cap

Without a cap, a single high-magnitude signal swing could flip a
pair from `Alliance` to `Rivalry` in one tick — observable as
"diplomatic whiplash". The cap is:

```text
MAX_RELATION_FLIPS_PER_TICK = 2
```

Enforced inside `phase_diplomacy` (not in `apply_signal`) so it
counts across the whole phase, not per pair. If a tick would
produce 3+ transitions, the 3rd-and-beyond are **deferred** to
the next tick (the signal is still applied to `score`, but the
`RelationKind` transition is gated). This keeps the replay
readable without losing information — the deferred transitions
appear in the next tick's event feed.

### 6.4 Entropy target as a dashboard metric

`crates/agents/src/diplomacy.rs:244-270` already computes
`trust_entropy`. Wire it to the emergence dashboard (per
`EMERGENCE_DASHBOARD.md` pattern):

- Operating range: `[1.5, 2.1]` (documented at
  `crates/agents/src/diplomacy.rs:243-244`).
- Out-of-range alarms: `h < 1.0` (relations too clustered,
  likely scripted), `h > 2.2` (relations too random,
  likely missing signals).

---

## 7. Determinism, perf, and test minimums

### 7.1 Determinism guards

Three new guards, all matching `EMERGENCE_WIRING_PATCHPLAN §8`:

1. **Pair ordering.** Process pairs in canonical
   `(min, max)` order — identical to
   `crates/agents/src/diplomacy.rs:153-159`. Two same-seed sims
   produce identical pair iteration order.
2. **Ring buffer determinism.** `signal_history_head` is incremented
   deterministically; no per-tick RNG is consumed by the ring.
3. **Signal source determinism.** Each signal source in §4.1–4.6 is
   fed from a producer that already exists or lands as part of
   ADR-020 — both of which are seeded off
   `state.rng_seed ^ state.tick ^ agent_id` (per
   `EMERGENCE_WIRING_PATCHPLAN §8`).

### 7.2 Perf budget

`EMERGENCE_WIRING_PATCHPLAN §8` caps the 11 new phases at
`0.6–1.2 ms / tick` at 5,000-agent populations. `phase_diplomacy`
already exists; this design only adds the per-pair signal
producers, which cost:

- 6 vector reads from `state.faction_resources` per pair:
  O(6 × k_goods) ≈ O(60) per pair.
- 1 ring-buffer write per pair: O(1).
- 1 threshold check + (maybe) 1 treaty emission per pair: O(1).

At 5,000 agents / ~50 clusters / `50 × 49 / 2 = 1,225` pairs,
total cost ≈ `1,225 × 60 = ~74k` scalar ops/tick ≈ `0.15 ms` on
the same hardware profile as ADR-020. Stays well under the 4 ms
tick budget cap.

### 7.3 Test minimums (ADR-011)

| Test | Validates |
|------|-----------|
| `phase_diplomacy_emerges_alliance_on_sustained_trade` | §5.2 trade → Alliance |
| `phase_diplomacy_emerges_alliance_on_energy_surplus` | §5.2 one-surplus → Alliance |
| `phase_diplomacy_emerges_alliance_on_complementarity` | §5.2 complementarity → Alliance |
| `phase_diplomacy_emerges_war_on_sustained_grievance` | §5.3 grievance → War |
| `phase_diplomacy_emerges_war_on_resource_competition` | §5.3 resource comp → War |
| `phase_diplomacy_war_persists_after_engagement_stops` | §5.3 decay hysteresis |
| `phase_diplomacy_emits_treaty_alliance_v1_on_threshold_cross` | §5.4 treaty emission |
| `phase_diplomacy_emits_treaty_war_v1_on_threshold_cross` | §5.4 treaty emission |
| `phase_diplomacy_terms_json_bounded_under_256_bytes` | §5.4 replay budget |
| `phase_diplomacy_relation_flip_cap_defers_third_flip` | §6.3 flip cap |
| `phase_diplomacy_entropy_within_operating_range` | §6.4 entropy target |
| `phase_diplomacy_signal_history_ring_is_deterministic` | §7.1 #2 |

12 tests (4 above the ADR-011 3-test minimum per phase — the extra
tests cover the **interactions** between signals that are the
defining characteristic of an emergent system).

---

## 8. Phased landing plan

Mirrors `EMERGENCE_WIRING_PATCHPLAN §9` "mechanical recipe":

1. **Land the 6 signal producers** in
   `crates/agents/src/diplomacy.rs` (extend `RelationRecord`,
   add 6 producer fns, add ring buffer). Build GREEN.
2. **Wire `phase_diplomacy` to call the 6 producers** in canonical
   pair order. Build GREEN.
3. **Land the 5 treaty subtypes** as `ReplayEvent` variants + the
   `record_treaty` method on `ReplayLog`. Build GREEN.
4. **Land the 4 relation-event `record_*` methods** + the entropy
   dashboard hook. Build GREEN.
5. **Add the 12 tests** from §7.3.
6. **Verify** with `just civis-3d-verify` (full 3D workspace
   gate) and the existing `test_determinism` regression.
7. **Move this design to "Accepted"** in the FR-3D matrix DIPLO
   sub-matrix.

---

## 9. Cross-references (consolidated)

- [ADR-015](../adr/ADR-015-faction-emergence-via-k-means-ideology-clustering.md) — k-means faction primitive.
- [ADR-020](../adr/ADR-020-wire-dormant-emergence-phases.md) — wired emergence phases.
- [`EMERGENCE_WIRING_PATCHPLAN.md`](EMERGENCE_WIRING_PATCHPLAN.md) — phase signatures, DAG ordering, replay-bus pattern.
- [`crates/agents/src/diplomacy.rs`](../../crates/agents/src/diplomacy.rs) — existing six-driver `DiplomacySignal` substrate.
- [`crates/engine/src/faction_emergence.rs`](../../crates/engine/src/faction_emergence.rs) — k-means producer.
- [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md) — DIPLO sub-matrix.
- [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) — emergence charter.
- [`docs/design/warfare.md`](warfare.md) — `phase_tactics` / `last_tick_engagements` consumer.
- [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) — culture emergence (related substrate).
- [`docs/design/polities-markets.md`](polities-markets.md) — polity cluster concept.
- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — emergence charter (3-test minimum, FR-traceable).