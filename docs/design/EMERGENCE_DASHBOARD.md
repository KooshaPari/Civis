# Emergence Dashboard — Tuning Criticality at the Edge of Chaos

> **Audience:** emergent-systems / DX / sim-tuning maintainers.
> **Scope:** design of the `EmergenceDashboard` surface (what it shows, what it
> means, how to use it to drive the sim toward — and hold it at — the
> edge of chaos).
> **Status:** design, **not implementation**. Code references point at the
> authoritative crates; metric names match the public API.
> **Sister doc:** `docs/design/emergence-dashboard.md` (the metric
> *definitions*, FR-CIV-EMERG-001/002/003). This document is the **tuning
> manual** that sits on top of it.

---

## 1. Why a tuning dashboard?

The Civis simulation is built out of loosely-coupled phases
(`phase_culture`, `phase_diffusion`, `phase_economy`, `phase_diplomacy`,
`phase_production`, `phase_emergence_events_close`, …) whose aggregate
behaviour is not predictable from any one phase. When the per-phase
constants are tuned in isolation the run typically lands in one of three
basins:

1. **Frozen** — institutions lock, structure_count plateaus, novelty_rate
   collapses to ~0, power-law exponent steepens (heavy tail disappears).
   The replay looks like a stalling empire.
2. **Turbulent** — coupling floods the run, novelty_rate saturates at
   1.0, structure_count explodes, MI between adjacent phases blows up,
   branching ratio leaves the 0.9–1.0 band. The replay looks like
   white noise.
3. **Edge of chaos** — the regime Langton identified for cellular
   automata, Kauffman for random Boolean networks, and Bak for
   self-organised criticality. The metrics *covary*: structure_count and
   novelty_rate both rise together, the power-law exponent sits in the
   heavy-tail band (1.5 ≤ α ≤ 2.5), entropy and MI oscillate without
   pinning, and the branching ratio hovers around 1.0.

The dashboard's job is to **make basin (3) visible** and to give the
tuner a per-metric set of knobs that bias the run back toward it.

---

## 2. The five criticality metrics

The dashboard is a thin projection over
`civ_emergence_metrics::dashboard::EmergenceDashboard`
(`crates/civ-emergence-metrics/src/dashboard.rs:36-65`). Each field maps
to a single number the tuner is supposed to read and react to:

| Metric                | Source (raw)                                          | Range  | What it tells you                                |
|-----------------------|-------------------------------------------------------|--------|--------------------------------------------------|
| `power_law_alpha`     | `sample_snapshot::EmergenceSampleSnapshot::power_law_alpha` (`crates/civ-emergence-metrics/src/sample_snapshot.rs`) | f32    | Heavy-tail exponent over the size distribution of emergent clusters. α ∈ **[1.5, 2.5]** is the SOC band. |
| `shannon_entropy`     | `resource_entropy` field on the same snapshot         | [0,1]  | Normalised Shannon entropy over resource-bucket distribution. 1.0 = uniform across buckets, 0.0 = one bucket owns everything. |
| `structure_count`     | `structure_count` field on the same snapshot         | u32    | Number of *emergent* structures (settlements, factions, trade routes, institutions) at sample time. |
| `novelty_score`       | `novelty_rate` field on the same snapshot             | [0,1]  | Fraction of new configuration hashes the novelty window has *not* seen before. 0.0 = totally stuck, 1.0 = totally novel. |
| `coupling_strength`   | `mutual_information` over phase signal vectors        | [0,1]  | Normalised mutual information between paired phase outputs (resource ↔ culture, culture ↔ diplomacy, …). |

> **`coupling_strength` derivation.** The current engine wires a
> discrete per-pair `phase_a → phase_b` MI estimator
> (`crates/civ-emergence-metrics/src/mutual_information.rs`). The
> dashboard folds the *active* pair list into a single scalar (mean over
> active pairs) and a per-pair vector (see §6). The active pair list is
> curated by the engine's `EmergenceConfig` — see
> `crates/engine/src/emergence_metrics.rs:1-200` for the wiring surface
> the dashboard reads from.

### 2.1 Target bands (the "edge of chaos" envelope)

| Metric              | Frozen                              | Edge of chaos                          | Turbulent                          |
|---------------------|-------------------------------------|----------------------------------------|------------------------------------|
| `power_law_alpha`   | > 2.5 (steep — no heavy tail)       | **1.5 ≤ α ≤ 2.5**                      | < 1.5 (very heavy / divergent)     |
| `shannon_entropy`   | < 0.3 (one bucket dominates)        | **0.5 ≤ H ≤ 0.85**                     | > 0.9 (white noise across buckets) |
| `structure_count`   | monotonic plateau                    | **growing, with power-law distribution** | monotonic super-linear (runaway)   |
| `novelty_score`     | < 0.05 (no new configurations)      | **0.1 ≤ ν ≤ 0.4** (steady novelty)     | > 0.6 (saturation — nothing repeats) |
| `coupling_strength` | < 0.1 (phases are independent)      | **0.2 ≤ MI ≤ 0.6** (informative coupling) | > 0.7 (phases drive each other chaotically) |

The bands are **heuristic**, not derived. They were chosen so that a
known-frozen regression run and a known-turbulent regression run fall
outside on opposite sides. They should be re-fit as we accumulate
labeled runs (see §8).

---

## 3. Phase → metric wiring

This is the part that most design readers actually need: **which engine
phase writes which metric, and when does the dashboard refresh?**

### 3.1 Phase emit map

| Engine phase (`crates/engine/src/engine.rs`) | Metrics touched                                          | Why this phase owns the metric                                                                                  |
|---------------------------------------------|----------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------|
| `phase_culture`                             | `shannon_entropy`, `coupling_strength(culture ↔ diplomacy)` | Culture-traits / N2 affinity directly shift the resource-bucket distribution and the peace-bonus signal.        |
| `phase_diffusion`                           | `shannon_entropy`, `coupling_strength(resource ↔ culture)` | The diffusion PDE moves mass between resource buckets; bucket histogram is read here.                          |
| `phase_economy` / `phase_production`        | `structure_count`, `coupling_strength(resource ↔ production)` | Buildings, institutions, and emergent structures are born/die in this phase.                                   |
| `phase_diplomacy`                           | `coupling_strength(diplomacy ↔ culture)`, `coupling_strength(diplomacy ↔ trade)`, `structure_count` (factions, routes) | Faction merge/split, trade route birth/decay, peace-bonus signal all live here.                                  |
| `phase_emergence_events_close`              | **`novelty_score`** (the rolling-window hash check)       | The only phase that owns the novelty window. This is where `EmergenceState.novelty_window_new` is incremented. |
| `phase_life`                                | **`power_law_alpha`** (cluster-size histogram)           | Settlements are born and clustered in `phase_life`; the size distribution is read at sample time.              |
| `sample_emergence` (sampler, post-tick)     | All five (snapshot assembly)                              | The sampler pulls the current state into `EmergenceSampleSnapshot` (throttled — see §3.2) and the dashboard is rebuilt from it. |

The five metrics are **read-only** with respect to the phases — they
are an observation surface, not a control loop. The control loop is the
*knob* set (§4), and the dashboard is the per-knob signal that drives
human (or scripted) knob changes.

### 3.2 Sample cadence

`sample_emergence` runs at a fixed cadence (default: every 200 ticks,
configurable via `EmergenceConfig::sample_period_ticks`). The dashboard
inherits that cadence — the WebSocket frame, the JSON-RPC
`sim.emergence` reply, and the `sim.snapshot.emergence` block all
reflect the **most recent sample**, not the most recent tick. This
matters for tuning: pushing the period down does not give the
dashboard more information, it just costs more. Pushing it up blurs
transient excursions (e.g., a war spike) and should only be done when
the run is in steady state.

The novelty window rolls independently of the sample period; its
period is `EmergenceConfig::novelty_window_ticks` and defaults to
5× the sample period. This is intentional — novelty needs a longer
memory than instantaneous structure counts.

---

## 4. The criticality knobs

This is the **tuning interface**. Every knob below is currently a
`const` or `pub const` in `crates/engine/src/engine.rs` (or the
relevant module); the dashboard's job is to *expose* the *direction*
each knob pushes a metric, not to set the values. The values stay in
the engine so determinism is preserved.

### 4.1 Knob → metric push table

The arrow shows the direction the metric moves when the knob is
**increased**. (↓ = "knob up makes metric down", ↑ = opposite.)

| Knob (file:line)                                       | `power_law_alpha` | `shannon_entropy` | `structure_count` | `novelty_score` | `coupling_strength` |
|--------------------------------------------------------|:-----------------:|:-----------------:|:-----------------:|:---------------:|:-------------------:|
| `MAX_INSTITUTION_LEVEL` (`engine.rs:2484`)             | ↑ (lock-in steepens tail) | ↓ (institutions concentrate) | ↓ (cap kills structures) | ↓ (lock-in kills novelty) | ↓ (less cross-phase drive) |
| `SETTLEMENT_CLUSTER_RADIUS_FP` (`engine.rs:2830`)      | ↓ (wider clusters → heavier tail) | ↑ (more diverse cluster sizes) | ↑ | ↑ (more settlement types) | ↑ (more cross-cluster signal) |
| `SETTLEMENT_MIN_MEMBERS` (`engine.rs:2828`)            | ↑ (only large clusters count) | ↓ (fewer, larger clusters) | ↓ | ↓ | ↓ |
| `COHESION_BELIEF_DIVISOR` (`engine.rs:2690`)           | ↑ | ↓ | ↓ | ↓ | ↑ (stronger belief ↔ cohesion coupling) |
| `COHESION_UNREST_DIVISOR` (`engine.rs:2692`)           | ↓ | ↑ | ↑ | ↑ | ↑ |
| `TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD` (`engine.rs:3120`) | ↑ | ↓ | ↓ | ↓ | ↑ |
| `TRADE_ROUTE_MIN_RELATION` (`engine.rs:3122`)          | ↑ | ↓ | ↓ | ↓ | ↑ |
| `MAX_TRADE_ROUTES` (`engine.rs:3124`)                 | ↑ | ↓ | ↓ (hard cap) | ↓ | ↓ |
| `TRADE_ROUTE_UNUSED_DECAY_TICKS` (`engine.rs:3126`)    | ↓ | ↑ | ↑ | ↑ | ↓ |
| `DIPLOMACY_BASE_CONFLICT_THRESHOLD` (`engine.rs:2812`) | ↑ (peace → fewer merges → bigger tail exponent) | ↓ | ↓ | ↓ | ↑ |
| `DIPLOMACY_COMPETITION_DRIFT` (`engine.rs:2820`)       | ↓ | ↑ | ↑ | ↑ | ↑ |
| `DIPLOMACY_TRADE_DRIFT` (`engine.rs:2818`)             | ↓ | ↑ | ↑ | ↑ | ↑ |
| `DIPLOmacyMinConflictThreshold` (`engine.rs:2844`)     | ↑ | ↓ | ↓ | ↓ | ↑ |
| `FACTION_RELATION_DECAY_FACTOR` (`engine.rs:2816`)     | ↑ (faster decay → memoryless → steeper tail) | ↓ | ↓ | ↓ | ↓ |
| `RELIGIOUS_UNITY_PEACE_CAP` (`engine.rs:2278`)         | ↑ | ↓ | ↓ | ↓ | ↑ |
| `LANGUAGE_INTELLIGIBILITY_PEACE_CAP` (`engine.rs:2303`)| ↑ | ↓ | ↓ | ↓ | ↑ |
| `CULTURE_PEACE_SPAN` (`engine.rs:2826`)                | ↑ | ↓ | ↓ | ↓ | ↑ |
| `N12_AFFINITY_BIAS_SCALE` (`engine.rs:2268`)           | ↓ | ↑ | ↑ | ↑ | ↑ |
| `DIFFUSION` knobs (`crates/civ-diffusion`, see `DiffusionParams` in `engine.rs:413`) | varies | ↑ (higher `k` → more uniform) | varies | ↑ (new gradients) | ↑ |
| `WAVE_*` knobs (heat-field, if enabled, `engine.rs:1538`+) | varies | ↑ | varies | ↑ | ↑ |

> **Reading the table.** Increasing *most* knobs that **constrain**
> the system (caps, thresholds, divisors) pushes the run toward
> **frozen**. Increasing knobs that **inject variance** (drift factors,
> decay, diffusion `k`, lower birth thresholds) pushes toward
> **turbulent**. The art is finding the narrow band where the run
> is neither.

### 4.2 Phase-pair coupling knobs (the MI surface)

`coupling_strength` is itself a knob, not just a metric. Each
active pair has a coupling coefficient in `EmergenceConfig`:

| Phase pair (signal)                      | Knob field (proposed)             | What it controls                                         |
|------------------------------------------|-----------------------------------|----------------------------------------------------------|
| resource ↔ culture                       | `k_resource_culture`              | How strongly resource entropy writes back into the culture trait pool. |
| culture ↔ diplomacy                      | `k_culture_diplomacy`             | How strongly cultural similarity feeds peace bonus (and the inverse for hostility). |
| diplomacy ↔ trade                        | `k_diplomacy_trade`               | Trade-route birth rate per unit of pairwise relation.   |
| belief ↔ cohesion                        | `k_belief_cohesion`               | Cohesion-bonus signal per unit of belief.                |
| unrest ↔ trade                           | `k_unrest_trade`                  | Unrest penalty on trade-route birth.                    |

The `mutual_information` estimator reads these as *scaling factors*
on the signal — they are the **direct controls** of `coupling_strength`
on the dashboard. The dashboard reports the per-pair MI estimate and
the configured `k` next to each other so a tuner can see "MI is at
0.85, configured `k` is 0.4" and know to either lower the knob
or accept the runaway.

---

## 5. The tuning loop

A human (or scripted) tuner uses the dashboard in a four-step loop:

```
            ┌────────────────────────────────────┐
            │  1. OBSERVE: read all 5 metrics    │
            │     from sim.emergence             │
            └──────────────┬─────────────────────┘
                           │
                           ▼
            ┌────────────────────────────────────┐
            │  2. CLASSIFY: which basin?         │
            │     frozen / edge / turbulent      │
            │     (use the bands in §2.1)        │
            └──────────────┬─────────────────────┘
                           │
                           ▼
            ┌────────────────────────────────────┐
            │  3. PICK KNOB: use §4.1 table      │
            │     to pick the smallest knob      │
            │     change that flips the right    │
            │     arrow direction                │
            └──────────────┬─────────────────────┘
                           │
                           ▼
            ┌────────────────────────────────────┐
            │  4. SET & WAIT: change one knob,   │
            │     run ≥ 2 sample windows,        │
            │     re-observe                     │
            └──────────────┬─────────────────────┘
                           │
                           └──────► back to (1)
```

**The cardinal rule:** change **one knob at a time**. The
phase-pair coupling is non-linear, and the per-metric push
directions in §4.1 assume the *other* knobs are pinned. Flipping
two at once is the standard way to overshoot into the opposite
basin.

**The second rule:** wait at least 2 sample windows. The novelty
window is 5× the sample period by default, so 2 sample windows is
not actually enough for the novelty score to reflect the change;
the rule of thumb is *at least one novelty window*, i.e. 5 sample
periods, before drawing conclusions.

---

## 6. Dashboard layout (what the panel shows)

The dashboard is the same `EmergenceDashboard` struct
(`crates/civ-emergence-metrics/src/dashboard.rs:36`) projected onto
the web UI (`web/...`) and onto the Godot / Bevy / Unreal clients
(via the JSON-RPC `sim.emergence` payload). The panel has:

1. **Five headline numbers** (the five metrics, colour-coded by basin).
2. **A sparkline per metric** over the last *N* sample windows
   (default N = 20). The novelty sparkline gets a longer history
   than the others because its window is longer.
3. **A basin badge** ("FROZEN" / "EDGE" / "TURBULENT") derived from
   the per-metric bands in §2.1, with a *vote*: the dashboard
   reports how many of the five metrics are inside the edge band.
   A score of 5/5 is the goal; 3/5 is "close, keep nudging";
   1/5 or below means the run is far from the edge.
4. **A "next-knob" hint** computed from §4.1: the highest-leverage
   knob the tuner hasn't already pulled in the current direction.
   The hint is a hint, not an authority — the tuner can ignore it.
5. **A per-pair MI table** for the active coupling pairs (§4.2),
   showing the observed MI and the configured `k` side by side.

The "no data" affordance (sample hasn't fired yet, or the
novelty window is still warming up) is rendered as a dashed
gauge with the message "warming up — wait one novelty window".

---

## 7. Worked example: a run that froze

Scenario: a run in steady state. Sample at tick 4000 reports:

```
power_law_alpha   = 3.1   (above the 2.5 cap → frozen-leaning)
shannon_entropy   = 0.21  (below 0.3 → frozen-leaning)
structure_count   = 142   (flat for 12 sample windows)
novelty_score     = 0.01  (below 0.05 → frozen-leaning)
coupling_strength = 0.08  (below 0.1 → phases are independent)
```

**Basin vote: 5/5 metrics outside the edge band → "FROZEN".**

**Tuning step (using §4.1):** the tuner's goal is to *loosen* the
system without flipping it turbulent. The most leveraged single
knob is `SETTLEMENT_CLUSTER_RADIUS_FP` (raising it widens
settlements, which pushes `power_law_alpha` ↓, `shannon_entropy`
↑, `structure_count` ↑, `novelty_score` ↑, `coupling_strength` ↑ —
all five arrows in the right direction). Lower it from 6 to 9
(50% increase), wait one novelty window, re-sample.

If the next sample reports `power_law_alpha` = 2.1, `shannon_entropy`
= 0.71, `novelty_score` = 0.18, `coupling_strength` = 0.35, the
run is in the edge band on 4/5. The remaining metric
(`structure_count` is probably still growing) is allowed to keep
moving — do not react to it for at least another novelty window.

**Anti-pattern to flag:** the temptation to also lower
`MAX_INSTITUTION_LEVEL` "while we're at it". That knob's arrows
all point the same way as the radius knob, so doubling up pushes
*twice as hard* — past the edge, into turbulence. One knob at a
time.

---

## 8. Open questions / future work

1. **Band calibration.** The §2.1 bands are heuristic. We need a
   labeled-run set (frozen / edge / turbulent, hand-classified
   from the existing regression corpus) and a fit. Until then, the
   bands are a starting point, not a target.
2. **Per-pair MI variance.** The `coupling_strength` scalar is a
   mean over active pairs; it can hide a single high-MI pair
   (the run is being driven by one coupling) or a single low-MI
   pair (a coupling knob is dead). The per-pair table is the
   fix, but the variance metric is not yet wired.
3. **Closed-loop tuning.** The "next-knob" hint in §6 is a static
   table lookup. A PID-style controller over the 5 metrics
   reading from §4.1 is the natural next step, *but it must live
   outside the engine* — the engine is deterministic and a
   closed loop breaks replay. The right place is the watch
   service, gated by an explicit `mod-dev` flag (see the
   `do not (agents)` list in `AGENTS.md`).
4. **Live knob edit.** The knobs in §4.1 are currently `const`.
   To make the dashboard truly interactive, the watch service
   needs a `sim.config.patch` JSON-RPC method that updates the
   `Simulation`'s tuning struct at runtime *without*
   invalidating the replay hash for already-recorded ticks
   (i.e. new ticks use the new values; replayed ticks use the
   old). That's a replay-format change and needs its own ADR.

---

## 9. References

- `crates/civ-emergence-metrics/src/dashboard.rs` — `EmergenceDashboard`
  projection.
- `crates/civ-emergence-metrics/src/sample_snapshot.rs` —
  `EmergenceSampleSnapshot` raw fields.
- `crates/civ-emergence-metrics/src/power_law.rs` — α estimator.
- `crates/civ-emergence-metrics/src/shannon.rs` — entropy over
  resource buckets.
- `crates/civ-emergence-metrics/src/mutual_information.rs` — per-pair
  MI estimator.
- `crates/civ-emergence-metrics/src/branching.rs` — branching ratio
  (driver of `structure_count` dynamics).
- `crates/engine/src/engine.rs` — `Simulation` and the phase order.
- `crates/engine/src/emergence.rs` — `EmergenceState` (the state
  the dashboard reads).
- `crates/engine/src/emergence_metrics.rs` — sampler / wiring
  (the integration point the dashboard is built on top of).
- `crates/server/src/jsonrpc.rs` — `sim.emergence` /
  `sim.snapshot.emergence` / `emergence.dashboard` JSON-RPC
  surface.
- `docs/design/emergence-dashboard.md` — metric *definitions*
  (FR-CIV-EMERG-001/002/003).
- `docs/design/EMERGENCE_WIRING_PATCHPLAN.md` — engine-side
  wiring plan.
- `ADR.md` (and the `EMERGENCE_COUPLING_AUDIT.txt` /
  `CRITICALITY_REVERIFY.md` / `CRITICALITY_REVERIFY_2.md` notes
  in the repo root) — the design history that produced the
  §4.1 knob map.
