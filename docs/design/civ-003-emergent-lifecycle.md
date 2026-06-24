# CIV-003: Emergent Citizen Lifecycle — Design Spec

> **Status:** Design (planner-only, 2026-06-10). No implementation code in this document.
> **Spec ID:** civ-003 | **Epic:** E2 | **Supersedes:** the FSM clauses in `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md` FR-CIV-ACTOR-001.
> **Governing canon:** `docs/guides/emergence-charter.md`, `docs/design/emergent-systems-spec.md`.
> **Traceability:** FR-CIV-LIFE-001/002/003 (needs/sickness/death), FR-CIV-GENETICS-001/002/010 (DNA mutation/recombination/speciation), FR-CIV-AGENTS-001 (wardrobe/tools diffusion), FR-CIV-ACTOR-001/002 (lifecycle — THIS document replaces the FSM reading).

---

## 0. Charter constraint

The Civis Emergence Charter forbids hardcoding life/sentience/psyche as authored state machines. This document specifies how `Born → Employed → Retired → Dead` (and the social roles in between) emerge from micro-level drivers rather than being scripted enum transitions. The FSM language in the original spec.md is preserved as an **observation vocabulary** — labels observers can apply to measured state — not as an implemented control structure.

---

## 1. Core emergence model

### 1.1 The observable lifecycle is a measurement, not a state field

There is no `LifecycleStage` enum on a citizen. Instead, each tick a read-only classifier function maps continuous state into a human-readable label for the dashboard and legends engine. The underlying state that drives behavior is entirely continuous:

| Continuous driver | Crate + field | What it does |
|---|---|---|
| Age in game-years | `Civilian::age: u16` (agents) | Increments each year-tick; governs need-decay scaling, physical capacity, mortality sensitivity |
| Need satisfaction vector | `civ_needs::Needs` (6 scalars `[0,1]`) | Drives health damage, activity choice, mood |
| Health integrity | `civ_needs::Health::integrity` | Monotonically links deprivation to death |
| Deprivation streak | `Health::deprivation_streak: u32` | Sickness onset gate after `HealthParams::sickness_onset_ticks` consecutive critical ticks |
| DNA-expressed cognition | `civ_genetics::sentience::cognition_score(dna, profile)` | Gates labor capacity tier and social role eligibility |
| Psyche maturity | `Psyche::maturity: f32` (`[0,1]`) in agents/psyche.rs | Plastic in youth (`1 − maturity * 0.8` in `nudge_temperament`), crystallized in elderhood |
| Social embedding | `SocialGraph::ties` (kinship, affinity, familiarity, trust up to `MAX_TIES=150`) | Determines partnering, care networks, mortality lags |
| Resource gradient | local voxel food/water stocks (`economy::stocks::Stocks`) | Determines whether needs can be satisfied at all |

### 1.2 Observable lifecycle labels (classifier output, not stored state)

```
fn classify_lifecycle(age: u16, health: &Health, maturity: f32, labor_capacity: f32) -> LifecycleLabel
```

| Label | Condition (all must hold) | Key behavioral difference |
|---|---|---|
| **Infant** | `age < 3` | Needs fully met by parents (safety delegated); no labor |
| **Child** | `age in [3, 14)` AND `maturity < 0.35` | Social play drives `social` need; physical capacity scales with age |
| **Adolescent** | `age in [14, 18)` OR `maturity in [0.35, 0.65)` | Apprentice labor slot; high temperament plasticity (`nudge_temperament`) |
| **Adult** | `maturity >= 0.65` AND `health.integrity > 0.3` AND `labor_capacity > 0.4` | Full labor capacity; partnering eligible when social graph has `Partner`-labeled tie |
| **Elder** | `age > age_threshold(dna)` OR `health.integrity < 0.3` with low deprivation | Reduced labor capacity; wisdom bonus to social influence |
| **Terminal** | `health.integrity < 0.1` OR `health.sick AND deprivation_streak > terminal_onset` | No labor; end-of-life care demand |
| **Dead** | `health.is_dead()` | ECS entity despawn trigger |

`age_threshold(dna)` is derived from a DNA byte slot (e.g. bytes 20–22 weighted by longevity profile), not a hardcoded constant. Species with high cognition (`cognition_score`) or high resource security age more slowly.

### 1.3 What produces each macro phenomenon

**Childhood / adolescence:** Emerges from low `maturity` (starts at 0.0 on `CivilianBundle::newborn_default`, grows each tick inversely proportional to `needs.any_critical(0.1)` — stress slows maturation) combined with `age`. There is no scripted "you are now a child" flag.

**Employment:** A citizen occupies a labor slot when three conditions converge: (a) `labor_capacity` > threshold (derived from `health.integrity * cognition_score * age_factor`), (b) a matching resource-gradient attractor exists nearby (food production, tool creation, shelter construction), (c) the social graph contains at least one `Cooperated` tie at that location. Employment is lost when capacity falls below threshold (illness, aging) or the resource gradient disappears (famine, collapse). The economy crate's `AllocationEngine` (specifically `PriorityTier`) assigns labor slots; the citizen does not hold a `job_id` field — it holds a `HomeAssignment::building_id` and the economy tick resolves what is produced.

**Partnering:** A citizen acquires a `Partner`-labeled tie (from `relation_label` in social.rs) when affinity >= 0.75, trust >= 0.35, familiarity >= 0.5, and both parties have `social` need below 0.6. Reproduction fires when two partnered citizens co-locate (`Position3d` within bonding radius) while both have `food > 0.5` AND `safety > 0.4` AND `health.integrity > 0.6` AND `age` within fertile range (a DNA-derived range, not a hardcoded [18,45]). This produces a `spawn_child_near` call — no scripted "couple formed" event.

**Mortality:** Death is a continuous gradient: `health.integrity` drains via `damage_per_critical * critical_count` per tick when needs are unmet (FR-CIV-LIFE-003). Genetics modulates baseline decay rates — a high-longevity DNA profile reduces `DecayRates::food` and `DecayRates::water` by a small factor derived from byte slots 23–25. Sickness (`Health::sick`) applies the additional `sickness_damage` term. There is no scripted "you die at age 70" check. Age increases mortality risk only by reducing the need-satisfaction capacity: older citizens move more slowly (lower `movement_speed_factor` equivalent) and have lower `regen`, so the same deprivation level that an adult recovers from kills an elder.

---

## 2. Bidirectional coupling to economy and society

The charter explicitly forbids siloed layers calling each other through API boundaries with no lag. The coupling mechanism here is **shared conserved gradients** with **explicit lags**.

### 2.1 Lifecycle → Economy (downward causation)

| Lifecycle signal | Economy effect | Mechanism (no direct call) |
|---|---|---|
| Population age distribution shifts toward elders | Labor supply falls | `AllocationEngine::allocate_by_priority` receives fewer eligible agents; `PriorityTier` items are unfilled, production drops |
| Mass infant cohort (baby boom) | Food/water demand spike | `Stocks` depletion rate rises because `Needs` vectors are summed across all living agents each economy tick |
| High mortality event (epidemic) | Labor + consumer demand both collapse | Fewer agents to fill slots AND fewer agents consuming; `EconomyState::energy_budget_joules` swing |
| Partnership rate rises | Housing demand gradient rises | `HomeAssignment` requests increase; shelter need pressure on existing stock; new construction attractor fires |

The tiered consumer demand that recently landed maps directly here: infants demand only Food+Water at low rate; adolescents add Shelter+Tools at medium rate; adults drive the full `GOODS` basket at maximum rate; elders downshift. This **tier is read from the age/maturity classifier output**, not hardcoded per-agent.

### 2.2 Economy → Lifecycle (upward causation with lag)

| Economy signal | Lifecycle effect | Lag mechanism |
|---|---|---|
| Food stocks depleted (`Stocks::get(Good::Food) == 0`) | `Needs::food` decay not offset → `deprivation_streak` grows | 1 tick lag (decay fires next tick after stock depleted) |
| Trade surplus restores stocks | Needs satisfy, `deprivation_streak` resets, sick clears when `integrity >= 0.95` | Recovery lag: `sickness_onset_ticks` (default 30) before health normalizes |
| Labor slot filled raises resource output | Shared stocks replenish, need pressure on neighborhood eases | Allocation lag: 1 economy `step()` cycle between production and stock availability |
| Famine → adult cohort dies → fewer producers | Positive feedback to further food deficit | Structural lag: 1 generational cycle (years) for child cohort to mature into labor |

### 2.3 Lifecycle → Society

| Signal | Social graph effect |
|---|---|
| Birth | `Interaction::Kin` applied to both parents' `SocialGraph`, setting `tie.kinship = 1.0` for the child |
| Shared deprivation streak in a cluster | Co-located agents accumulate `Coexisted` events → `familiarity` grows → `Interaction::Cooperated` when food-sharing occurs |
| High mortality cohort | `decay_social_graph` runs on surviving graph; dead agent's ties decay naturally (no tombstone needed — `last_seen` grows stale) |
| Partnering | Partner tie reaches `RelationLabel::Partner` threshold organically through repeated cooperation events |

### 2.4 Society → Lifecycle (downward causation)

| Social signal | Lifecycle effect |
|---|---|
| High `Needs::social` deprivation (no cooperated events for long streak) | `social` need falls critical → contributes to `deprivation_streak` → accelerates mortality risk |
| Dense cooperative cluster (many `CloseFriend`/`Family` ties) | Members share food (future `FR-CIV-LIFE-004`): a neighbor with `food > 0.7` satisfies a neighbor's `food` need by `0.1/tick` when co-located, draining the donor's stock |
| Cluster dissolution after conflict (Defected events dominate) | Safety need falls; residents seek new location → geographic dispersion; economy attractor loses labor |

---

## 3. Criticality knobs — edge of chaos

The following `LifecycleParams` struct (to be added in `crates/needs/src/lifecycle.rs` or as a field of `HealthParams`) concentrates the edge-of-chaos tuning surface. Default values target weak emergence (Class 4): sustained structure, not heat-death or explosion.

| Parameter | Type | Default | Effect on population | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `base_maturity_rate` | `f32` | `0.0008/tick` | Speed of childhood → adulthood | < 0.0003 | > 0.005 |
| `maturity_stress_penalty` | `f32` | `0.5` | Fraction of maturity rate lost per critical need | 0.0 (no stress effect) | 1.0 (stress fully halts maturation) |
| `longevity_dna_weight` | `f32` | `0.3` | Fraction of lifespan determined by genetics vs. needs | 0.0 (fully needs-determined) | 1.0 (fully genetics-determined) |
| `fertility_food_threshold` | `f32` | `0.5` | Minimum `Needs::food` for reproduction to fire | > 0.8 (very hard to breed) | < 0.2 (near-free breeding) |
| `fertility_safety_threshold` | `f32` | `0.4` | Minimum `Needs::safety` for reproduction | > 0.7 | < 0.1 |
| `care_share_rate` | `f32` | `0.1/tick` | Need-sharing rate between Partner-tied co-located agents | 0.0 (no mutual support) | 1.0 (full pooling) |
| `elder_labor_floor` | `f32` | `0.2` | Minimum labor capacity for elders even with health degraded | 0.0 (full drop-off) | 0.8 (no capacity loss) |
| `sickness_onset_ticks` | `u32` | `30` (existing `HealthParams`) | Lag before sickness fires; increases epidemic spread resistance | < 10 | > 200 |
| `damage_per_critical` | `f32` | `0.01` (existing) | Per-tick mortality pressure per unmet need | < 0.003 | > 0.05 |

All knobs are grouped in one `LifecycleParams` struct (not scattered across crates) and loaded from the scenario RON config. The emergence dashboard (§4) plots a real-time criticality indicator so the designer can see whether the system is heading toward heat-death or explosion before adjusting.

---

## 4. Observable emergence metrics for the dashboard

These metrics feed the **Emergence Dashboard** (`crates/engine/src/emergence.rs` expansion) and the legends engine. They are population aggregates computed cheaply from existing ECS queries, not new agent-level state.

| Metric | How to compute | Target signature (healthy population) | Failure mode |
|---|---|---|---|
| **Age distribution** | Histogram of `Civilian::age` bucketed to decade bands | Roughly log-normal or decaying exponential; no hard modal spike | Single-age-class spike = reproductive event driven by script, not pressure |
| **Cohort survival curve** | For each birth-year cohort, track `alive_count / born_count` over time | Gompertz-like: flat childhood, inflecting at elder transition | Flat line = mortality too low (no challenge); vertical drop early = over-tuned damage |
| **Maturity distribution** | Histogram of `Psyche::maturity` | Bimodal: peak near 0.2 (youth) and 0.85 (adults); smooth bridge | Missing youth peak = low birth rate; missing adult peak = high juvenile mortality |
| **Deprivation entropy** | Shannon entropy of `Health::deprivation_streak` values across population | High: population has diverse deprivation histories (edge of chaos) | Low: everyone deprived simultaneously (famine) or no one (no challenge) |
| **Partnership rate** | Count of `RelationLabel::Partner` ties / eligible adults | Tracks resource security; rises in prosperity, falls in famine | Monotone rise or fall = coupling to economy missing |
| **Labor fill ratio** | `filled_slots / total_slots` in `AllocationEngine` | Should oscillate with age distribution; lags baby boom by ~15 in-game years | Permanent under-fill = excessive mortality; permanent over-fill = insufficient demand |
| **Population structure count** | Number of distinct `ClusterId` groups with >5 members | Should grow then stabilize; power-law-ish cluster size distribution | Single cluster = no differentiation; all singletons = social collapse |
| **Mutual information: need↔labor** | Correlation between mean `Needs::food` and `labor_fill_ratio` with 1-tick lag | Moderate positive correlation (well-fed → more work → more food) | Near-zero = bidirectional coupling broken; near-1.0 = positive feedback loop dominates |

---

## 5. Phased implementation plan

This is a DAG-structured WBS. No code is written here; file paths and coupling points are identified for the implementing agent.

### Phase 0 — Prerequisite audit (no new structs)

| Task | File | Depends on | Agent effort |
|---|---|---|---|
| P0-A: Verify `Psyche::maturity` is written at birth to 0.0 and incremented per tick | `crates/agents/src/psyche.rs` | — | 2 tool calls |
| P0-B: Confirm `Civilian::age` increments on year-tick, not on every tick | `crates/agents/src/lib.rs` | — | 2 tool calls |
| P0-C: Map which `HealthParams` fields are scenario-configurable vs. hardcoded | `crates/needs/src/lib.rs` | — | 2 tool calls |
| P0-D: Audit `crates/economy/src/allocation.rs` for labor-slot eligibility hooks | `crates/economy/src/allocation.rs` | — | 3 tool calls |

### Phase 1 — LifecycleParams struct + classifier (no behavior change)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P1-A: Add `LifecycleParams` struct to `crates/needs/src/lifecycle.rs` (new file) | new `lifecycle.rs` | P0-C | All §3 knobs; default impl matching §3 table |
| P1-B: Add `fn classify_lifecycle(age, health, maturity, labor_capacity) -> LifecycleLabel` | same file | P1-A | Pure fn, no ECS; returns enum for dashboard/legends only |
| P1-C: Add `fn labor_capacity(age, health, dna, params) -> f32` | same file | P1-A | Uses `cognition_score` + `integrity` + age factor |
| P1-D: Add `fn age_threshold(dna, longevity_profile) -> u16` | same file | P1-A | DNA bytes 20–22 weighted avg, scaled to [50, 120] game-years |
| P1-E: xUnit tests: classify_lifecycle matches expected labels for boundary inputs | `crates/needs/src/lifecycle.rs` tests | P1-B, P1-C | 6 boundary cases; property test: no label appears for impossible state |

### Phase 2 — Maturity growth + need-stress penalty

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P2-A: Add `fn tick_maturity(psyche: &mut Psyche, health: &Health, params: &LifecycleParams)` | `crates/agents/src/psyche.rs` | P1-A | Increments `maturity` by `base_maturity_rate * (1 - stress)` where stress = fraction of critical needs × `maturity_stress_penalty` |
| P2-B: Call `tick_maturity` from `civ-engine` simulation phase, after needs tick | `crates/engine/src/engine.rs` | P2-A, P0-A | One-line integration |
| P2-C: Tests: maturity grows faster when needs are met; stress slows it; clamps at 1.0 | `crates/agents/src/psyche.rs` tests | P2-A | Deterministic under fixed seed |

### Phase 3 — Reproduction trigger

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P3-A: Add `fn should_reproduce(a: &Agent, b: &Agent, graph: &SocialGraph, params: &LifecycleParams) -> bool` | `crates/agents/src/social.rs` | P1-B, P1-C | Checks Partner tie label + food/safety thresholds + age range from `age_threshold` |
| P3-B: Add reproduction pass to simulation tick: query Partner-tied co-located adults, fire `spawn_child_near` when `should_reproduce` returns true | `crates/engine/src/engine.rs` | P3-A | Consumes seeded RNG for scatter; rate-limited to 1 birth per pair per in-game year |
| P3-C: Record `Interaction::Kin` on both parents' graphs and new child's graph | same | P3-B | Uses existing `apply_social_event` |
| P3-D: Tests: no reproduction when food < threshold; reproduction fires when all conditions met; child's kinship tie set correctly | `crates/engine/` integration test | P3-A–C | |

### Phase 4 — Labor capacity coupling to AllocationEngine

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P4-A: Thread `labor_capacity(age, health, dna, params)` into `AllocationEngine::allocate_by_priority` as an agent weight | `crates/economy/src/allocation.rs` | P1-C, P0-D | Agent with capacity 0.0 is ineligible; partial capacity reduces output unit count |
| P4-B: Emit `EconomyState` ledger entry when an agent's labor capacity drops below 0.4 (labor withdrawal event) | same | P4-A | Used by emergence dashboard MI metric |
| P4-C: Tests: elder with `integrity=0.25` contributes less output than adult with `integrity=0.9`; zero-capacity agent never fills a slot | `crates/economy/` tests | P4-A | |

### Phase 5 — Dashboard metrics

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P5-A: Add `fn compute_lifecycle_metrics(world: &World, economy: &EconomyState) -> LifecycleMetrics` | `crates/engine/src/emergence.rs` | P1-B, P4-A | Computes all §4 metrics in one pass over ECS |
| P5-B: Expose via `civis-cli census` subcommand | `crates/civis-cli/src/census.rs` | P5-A | JSON output for CI-accessible observation |
| P5-C: Tests: metrics stay bounded and coherent for populations of 10, 100, 1000 agents | `crates/engine/` tests | P5-A | |

### Phase 6 — Care-sharing (bidirectional social↔lifecycle coupling)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P6-A: Add `fn tick_care_share(world: &mut World, params: &LifecycleParams, rng: &mut ChaCha8Rng)` | `crates/agents/src/social.rs` or new `crates/agents/src/care.rs` | P3-C, Phase 2 complete | Co-located Partner/Family ties: transfer `care_share_rate` food need-satisfaction from donor (food > 0.7) to recipient (food < 0.5) |
| P6-B: Tests: infant with no food survives longer when partnered parent is nearby; donor's food need increases | integration test | P6-A | |

---

## 6. Test strategy summary

- **Unit tests** (property-based via `proptest`): each new pure function in `lifecycle.rs` has invariant tests (label boundaries, no invalid state reachable, maturity stays in [0,1]).
- **Integration tests** (hecs World with seeded RNG): a population of 20 agents run for 5000 ticks; assert age distribution is not monoclonal, at least one birth event fires, at least one death fires, labor fill ratio oscillates.
- **Emergence regression**: `cargo test -p civ-engine -- lifecycle_emergence` runs the 1000-agent 10000-tick scenario and asserts deprivation entropy > 0.5 (not heat-death) and population does not exceed 4× initial (not explosion). This runs in CI as a performance-gated test.
- **No determinism requirement** (per charter): tests assert statistical properties, not bit-identical outcomes.

---

## 7. What this spec does NOT include

- Any `enum LifecycleStage` stored on a citizen entity.
- Any `age >= 65 → retire` guard.
- Any scripted `employment_status` field toggled by a lifecycle event handler.
- Any LLM call in the lifecycle path.
- Any "ideology field" updated by a scripted `ideology_shift()` function (that is a separate domain spec — the social coupling here flows through `Psyche::beliefs` and `update_beliefs` which already respond to culture exposure through `SocialGraph` ties).

---

*Document authority: this spec supersedes the FSM interpretation of FR-CIV-ACTOR-001 in `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md`. The acceptance criteria in that spec are satisfied by the emergence classifier (§1.2) + the tests in §6, not by a hardcoded state machine.*
