# TECH_PROGRESSION: Emergent Tech / Research Progression — Possibility-Space Design

> **Status:** Design spec (docs-only, planner stance). Owner: Research Lead.
> **Branch:** `research/tech-progression-design`. **Companion PRs:** wired
> `phase_research` + `phase_tech` slots in `Simulation::tick` (ADR-020, PR
> #732); companion spec [`docs/design/tech-engineering.md`](tech-engineering.md)
> (the *abstract* emergent model — charter-level, not engine-shaped).
> **Engine state at design time:** `phase_research` and `phase_tech` are
> **invoked** from `Simulation::tick` (engine.rs:1204-1205) but their method
> bodies are still empty stubs in this branch — the methods need to be
> filled in by an implementation PR. This document is the **shape of those
> method bodies**: a possibility-space the phases traverse, not a hand-coded
> tech tree.
> **Governing constraint:**
> [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) —
> *technology is not a fixed tree*. Only physical / environmental / genomic
> laws are authored; the *order and timing* of discovery emerge. This spec
> extends the charter into the engine-shaped runtime that the wired
> `phase_research` + `phase_tech` slots expect.
> **Companion docs:**
> [`docs/design/tech-engineering.md`](tech-engineering.md) (charter-level
> emergence model — same author intent, more abstract),
> [`docs/design/EMERGENCE_WIRING_PATCHPLAN.md`](EMERGENCE_WIRING_PATCHPLAN.md)
> (DAG / cap table for these phases — same source of truth for inputs/outputs/caps),
> [`docs/adr/ADR-020-wire-dormant-emergence-phases.md`](../adr/ADR-020-wire-dormant-emergence-phases.md)
> (the slot the phases occupy in `Simulation::tick`),
> [`docs/adr/ADR-011-n-series-emergence-coupling-architecture.md`](../adr/)
> (the shared-gradient coupling doctrine).
> **Existing crates this spec maps onto (extend, never duplicate):**
> `crates/laws` (the physics/materials laws DB + validator — the only
> authority on what is *physically possible*),
> `crates/research` (`TechCard`, `LlmClient`, `validate`, `ResearchCache`,
> `LlmEvent`, `ReplayMode`, `run_research_cycle` — the validator + LLM lane
> + replay-safe cache),
> `crates/diffusion` (Bass/Rogers S-curve — adoption spread within a
> population),
> `crates/needs` (per-agent needs — the need-pressure signal),
> `crates/economy` (`Stocks`, `surplus`/`deficit`,
> `comparative_advantage` — resource-availability signal),
> `crates/agents` (Civilian + Psyche — the cognitive-capacity signal;
> ClusterMember — the population unit; KnowledgeSet carrier).
>
> **Engine shape this spec drives:** the 6-bit `tech_unlocks_for_tier`
> ladder (`engine.rs:2026-2056` — `TECH_IRRIGATION`…`TECH_GUNPOWDER`) is
> the *observation surface* (downward causation reads it), not the
> *invention mechanism*. The phases produce research progress + bitmask
> changes; the mechanism of *what gets researched* is the emergent
> candidate-sourcing pass this spec describes.

---

## 0. Thesis

**Technology in Civis is a *measured pattern*, not a tree the player climbs.**

A population invents a technique when three independent pressures
coincide — **NEED** (an unmet need or persistent economic deficit the
technique would relieve), **RESOURCE** (the material inputs are locally
available or tradeable), and **KNOWLEDGE** (the prerequisite techniques /
laws are already known to that population). The wired `phase_research`
turns that pressure triple into `state.research_progress`; the wired
`phase_tech` turns accumulated progress into `state.tech_unlocks` — a
**bitmask**, not a sequence.

Every candidate technique must validate against the authored `civ-laws`
DB through `civ-research::validate` *unchanged* — the laws DB is the
only authority on what is *physically possible*; the *order and timing*
of discovery is emergent. The `TechCard` shape (id, era-rank, inputs,
energy_cost, byproducts, dependencies) is the **schema of a candidate**;
the actual catalogue is *open-ended* (canonical seed cards + LLM-proposed
cards in Hybrid/Free saves).

> **Eras are labels we read off, never gates** — `era_label` of a
> population is a percentile over the era-rank of *adopted* techniques.
> A civ does not "advance to the Bronze Age"; we *observe* that its
> adopted frontier has crossed the bronze threshold and we name the
> epoch accordingly. No code path may *gate* a capability on
> `era_label`; capability is governed by known+adopted techniques
> (`crates/research` + `crates/diffusion`).

This spec covers, in order:

1. **The possibility-space** — the `TechCard` schema + the authored
   seed catalogue + the LLM proposal lane (Hybrid/Free only) — §1.
2. **The three pressures** — `need × resource × knowledge`, the
   multiplicative gate — §2.
3. **`phase_research`** — pressure → progress, the research tick — §3.
4. **`phase_tech`** — progress → unlocks, the tech-tick (candidate
   sourcing + validation + insertion) — §4.
5. **Adoption dynamics** — Bass/Rogers S-curve diffusion of a known
   technique through a population, modulated by substrate state — §5.
6. **Era labels** — the percentile read-off over adopted techniques —
   §6.
7. **The full per-tick DAG** — what each phase reads / writes, the
   shared-gradient contract, the per-tick caps — §7.
8. **Crate mapping + FR catalogue + WBS** — what to build, where, in
   what order — §8-§10.

---

## 1. The possibility-space (no fixed tree)

### 1.1 What "tech" is, here

A *technique* is a `TechCard` (`crates/research/src/lib.rs:28-43`):

```rust
pub struct TechCard {
    pub id: String,            // stable id
    pub era: u16,              // era-rank label (sortable, not a gate)
    pub inputs: Vec<String>,   // resource IDs consumed per application
    pub energy_cost: u64,      // per-application cost
    pub byproducts: Vec<String>,
    pub dependencies: Vec<String>, // law IDs (in civ-laws DB)
}
```

`era` is an **era-rank label** (the era-rank of the dominant
dependency, per `crates/laws::Law::era_min`). It is a *sortable
difficulty rank*, not a gate. A population can hold a high-rank
technique without holding lower-rank ones (charter: alt-paths welcome).

A *KnowledgeSet* is a per-population set of `TechCard.id`s that the
population has discovered (not necessarily adopted). Knowledge is
**per-population, partial, and losable** — severing contact +
extinguishing the last holder removes a technique from the regional
frontier (§3 of `tech-engineering.md`).

A *technology* in the engine state sense is `state.tech_unlocks: u64`
(`engine.rs:2026-2056`) — a 6-bit observation bitmask derived each
tick from `research_tier()` (the count of completed techs in
`research_cache.researched`). The bitmask is a **coarse, downward-facing
observation surface** for game logic that wants to know "is metallurgy
known here?" — it is *not* the invention mechanism. The mechanism is
the three-pressure trigger (§2); the bitmask is the *consequence*.

### 1.2 What the possibility-space is

The **possibility-space** of a population at a tick is the set of
`TechCard`s it *could* invent *right now* — i.e. the candidates whose:

- declared `dependencies` are all known to the population
  (`knowledge_fraction = 1.0`),
- declared `inputs` are locally obtainable
  (`resource_availability > 0`), **and**
- unmet need or economic deficit exists that the technique would
  relieve (`need_pressure > 0`).

The set is **not enumerated** anywhere — it is *derived on demand*
from the live laws DB + the population's `KnowledgeSet` + the live
`Stocks` + the live `Needs`. A population with no metallurgy knowledge
has no candidates that depend on metallurgy; a population with no iron
ore has no candidates that require it as input. **The candidate set
shrinks and grows with the substrate** — that is what "no fixed tree"
means in runtime.

### 1.3 Where candidates come from (three lanes, in priority order)

The three lanes, in priority order — same as `tech-engineering.md §2.2`:

1. **Adjacent-possible canonical cards** — seed `TechCard`s in RON
   (mirroring the `civ-laws` RON convention) whose `dependencies` are
   *one step* beyond the population's `KnowledgeSet`. This is the
   replay-safe backbone: every `Canonical` save draws only from this
   lane. Backed by `crates/laws::DEFAULT_LAW_RON` /
   `MOD_LAW_FILENAME` plus a new `crates/research::seed_cards.ron` that
   lists the canonical techniques (one per authored
   `civ-laws::Law::id`).
2. **Diffused cards** — a neighbour already knows the card; the
   population can *adopt by learning* (cross-population spread, §5.2)
   rather than invent from scratch. The card is already in some other
   population's `KnowledgeSet`; the candidate surface is the symmetric
   difference of the two sets after the spread tick.
3. **LLM-proposed novel cards** — when need is high but no canonical /
   diffused candidate fits, the AI worker pool proposes a *new*
   `TechCard` (`crates/research::run_research_cycle`). Gated to
   `Hybrid` / `Free` saves; `Canonical` saves never invoke this lane
   (per `crates/research::replay_advance_llm_event`).

### 1.4 Acceptance — the `validate` gate is the only authority

Every candidate (canonical, diffused, or LLM) passes through
`crates/research::validate(card, &LawDb)` (lib.rs:200-220) **unchanged**:

- declared `dependencies` must exist in the `LawDb`
  (`RejectReason::UnknownDependency`);
- each dependency's `era_min` must be `≤` the card's `era`
  (`RejectReason::DependencyEraGated`);
- the card must have effects (`RejectReason::NoEffects`: not both
  `inputs` *and* `byproducts` empty).

On `Accept`, the card's `id` is added to the population's
`KnowledgeSet` and a fresh `AdoptionState` (f ≈ ε, e.g. `1e-3`) is
seeded for it. On `Reject`, the candidate is discarded; for LLM cards,
the rejection is logged for the dev-assist balance analyst
(`ai-rnd.md` §3).

**No technique exists that the laws DB does not permit.** This is the
charter's "model the rule, not the outcome" applied to invention.

### Acceptance criteria — possibility-space

- **AC-PS1**: The candidate surface of a population is a strict function
  of `(KnowledgeSet, Stocks, LawDb)` — no global pool, no global
  enumeration, no global tier ladder that is read for *gating* purposes
  (the 6-bit `tech_unlocks_for_tier` is a *read-off*, not a
  gate-generator — see §6 and §7.4).
- **AC-PS2**: A population with `knowledge_fraction < 1.0` for a card
  has that card absent from its candidate set; only by *learning* the
  prerequisite (diffusion) does the card enter the candidate set.
- **AC-PS3**: Every accepted card passed `civ-research::validate`
  against the *current* `LawDb`; there is no bypass path.
- **AC-PS4**: A `Canonical` save never emits an `LlmEvent`; the
  candidate surface draws from canonical + diffused lanes only (per
  existing `crates/research::replay_advance_llm_event`).
- **AC-PS5**: A `Hybrid` / `Free` save may invoke the LLM lane; the
  proposal is recorded as an `LlmEvent` with hash-keyed cache
  (`prompt_hash + input_snapshot_hash + model_id + model_version`) so
  replay is reproducible.

---

## 2. The three pressures — need × resource × knowledge

### 2.1 Invention-readiness score (the multiplicative gate)

A population becomes *ready to invent* a candidate card `C` when a
**multiplicative** gate clears. Multiplicative (not additive) so that
**any** missing factor blocks invention — you cannot invent steelmaking
with no need, no ore, or no prerequisite knowledge:

```
readiness(pop, C) =
    need_pressure(pop, C)        // [0,1]
  * resource_availability(pop, C) // [0,1]
  * knowledge_fraction(pop, C)   // [0,1]

invention_attempt(pop, C) per tick with hazard proportional to:
    base_rate * readiness(pop, C) * cognitive_capacity(pop)
```

- **`need_pressure(pop, C)`** ∈ [0, 1]: a scalar derived from
  `crates/needs` deficits + `crates/economy::deficit()` / scarcity.
  "How badly would inventing `C` help?"
  - High when agents in the population are starving / cold / unsafe
    (`crates/needs::Needs::food`, `.water`, `.safety`) **or** the
    economy shows a persistent `deficit()` / scarcity in a good `C`
    would produce (`C.outputs`, where `C` is a *production* technique).
  - This is the "necessity is the mother of invention" term.
- **`resource_availability(pop, C)`** ∈ [0, 1]: fraction of
  `C.inputs` locally obtainable — over `civ-economy::Stocks` + voxel
  material presence. A technique whose inputs are absent cannot be
  invented locally — but **can arrive by diffusion** (§5), which is
  how resource-poor regions still acquire tech.
- **`knowledge_fraction(pop, C)`** ∈ [0, 1]: fraction of
  `C.dependencies` (which are law IDs in `civ-laws::LawDb`) already
  known to the population — i.e. the laws (and prerequisite cards
  derived from those laws) are in the population's `KnowledgeSet`.
  At `1.0`, all prerequisites are known; below `1.0`, `C` is "not yet
  conceivable here."
- **`cognitive_capacity(pop)`** ∈ [0, 1]: an emergent scalar derived
  from genomic / sentience traits (`crates/species`),
  `civ-agents::Psyche::maturity`, population size, free labour
  surplus (`civ-economy`). A pre-sentient lineage ≈ 0. *Why this
  matters:* two populations with identical (need, resource,
  knowledge) but different cognitive capacity invent at different
  expected rates.

**No global timer.** A population with high need, abundant ore, and
metallurgy knowledge invents iron tools *early*; a sheltered,
resource-poor one may never invent them. *Variety-that-makes-sense*,
per the charter.

### 2.2 Candidate sourcing per tick — the algorithm `phase_tech` runs

The wired `phase_tech` (slot 14 in `PHASE_ORDER`) executes the
candidate-sourcing pass each tick:

1. **Build the candidate surface.** Walk the canonical seed cards in
   `crates/research::seed_cards.ron` (lane 1) whose `dependencies` are
   all in the population's `KnowledgeSet`. Append diffused cards
   (lane 2) reachable from neighbour populations' `KnowledgeSet`s via
   the contact network (`crates/agents::SocialGraph`). Optionally
   enqueue an LLM proposal request (lane 3) gated to
   `mode != Canonical` when the surface is empty but need is high.
2. **For each candidate `C` in the surface**, compute
   `readiness(pop, C)`. Reject `C` if any factor is `0.0`.
3. **Draw a per-candidate Bernoulli trial** at hazard
   `base_rate * readiness(pop, C) * cognitive_capacity(pop)`. On
   success, call `civ-research::validate(C, &law_db)`. On `Accept`,
   insert `C.id` into the population's `KnowledgeSet` and seed
   `AdoptionState(C.id) = (f = 1e-3, params = ...)`. On `Reject`, log
   + discard.
4. **Per-tick cap.** The total accepted-this-tick count is bounded by
   `MAX_RESEARCH_PER_TICK = 5_000` (in the *progress* units of
   `phase_research`; the *count* of accepted cards is bounded by a
   `MAX_TECH_UNLOCKED_PER_TICK = 1` per population — see §7.5) to
   prevent invention spikes; otherwise the system is on the
   edge-of-chaos envelope (ADR-018).

**Why per-tick cap matters.** A population with high need + abundant
resources + full prerequisites *would* invent every candidate on the
surface per tick under a naive loop. Capping to one accepted card per
tick per population keeps emergence *gradual* — exactly the charter
intent (variety that makes sense, not scripted).

### 2.3 Knowledge vs. capability — adoption is separate from invention

A card in a population's `KnowledgeSet` is *known*; adoption is a
separate `AdoptionState.f ∈ [0, 1]` advanced by `civ-diffusion::advance`
each tick (see §5). The split matters because:

- A technique crosses the knowledge frontier by *invention* (§2.1-2.2)
  or by *diffusion-learning* (§5.2); both are sharp events.
- Adoption is *gradual* — a known technique may take hundreds of ticks
  to saturate within a population. The classical Bass/Rogers S-curve
  gives ~50% adoption at `t* = ln((p+q)/p) / q` ticks after the
  innovation spark (with default `p = 0.03, q = 0.38` this is `~10.6`
  ticks, but `p` and `q` are *modulated* by substrate, not constant —
  §5.1).
- A population may *know* a technique and keep `f ≈ 0` if it cannot
  source the inputs (§5.1 resource gating) — knowledge-before-capability,
  realistic and emergent.

### Acceptance criteria — three pressures

- **AC-T1**: `readiness` returns `0.0` if any of (need, resource,
  knowledge) is `0.0` (multiplicative gate). Two populations with the
  same need/resource/knowledge but different cognitive capacity invent
  at different expected rates.
- **AC-T2**: A population with `knowledge_fraction < 1.0` for `C`
  never adds `C.id` to its `KnowledgeSet` by invention (only by
  diffusion-learning, which still requires the prerequisite to be
  learnable next).
- **AC-T3**: Removing all need pressure halts net new invention even
  with abundant resources + knowledge (no invention-for-its-own-sake).
- **AC-T4**: The per-tick accepted count per population is bounded by
  `MAX_TECH_UNLOCKED_PER_TICK = 1`; the per-tick accepted count across
  *all* populations is bounded by `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL = 8`
  to keep the emergence dashboard's chaos metric on the
  edge-of-chaos envelope.

---

## 3. `phase_research` — pressure → progress (the research tick)

### 3.1 Slot in `Simulation::tick`

`phase_research` runs in slot 13 of `PHASE_ORDER` (engine.rs:1204, after
`phase_life` and before `phase_tech`). Per ADR-020 §2 table:

- **Reads (inputs from earlier phases):** `state.population` (post-life),
  `state.belief` (stale-allowed — runs *before* `phase_belief`, so the
  cap on belief-flavoured input is small; see ADR-020 "Note on
  belief/cohesion latency"), `state.cohesion` (stale-allowed — runs
  *before* `phase_cohesion`), `economy_state.research_funding` (post-
  `phase_economy`), `world` (ECS) for `civilian_count`,
  `mean_psyche_maturity`, `mean_psyche_valence`.
- **Writes (outputs to later phases):** `state.research_progress: u64`
  (`saturating_add`, capped by `MAX_RESEARCH_PER_TICK = 5_000` per
  tick), `research_cache.in_progress` (advances the queued card's
  progress), `research_cache.researched` (when a card completes).

### 3.2 The per-tick accumulation

```
delta =
    base_research_floor(pop)
  + belief_contribution(belief)
  + cohesion_contribution(cohesion)
  + sentience_research_bonus(world)        // civ-genetics, civ-agents::Psyche
  + scarcity_research_pressure(economy_state) // cives when deficit in C.outputs is high
  + funding_research_pressure(economy_state.research_funding)

state.research_progress =
    state.research_progress.saturating_add(delta.min(MAX_RESEARCH_PER_TICK))
```

Each contribution is a small per-tick increment so the system stays
on the edge-of-chaos envelope. `delta` is non-negative, integer, and
clamped per tick — no overflow, no per-tick leap, no hidden RNG.

The **base_research_floor** is a function of `state.population` — a
non-zero floor so even a small population slowly accumulates
progress. The other contributions modulate the floor up or down
(belief, cohesion, sentience, scarcity) — and importantly, the
*scarcity* leg is a *pressure*: when there is a persistent
`deficit()` in a good `C` would produce, that pressure *raises* the
research rate, which is the upward-causation feedback that
*closes the loop* (need → research → invention → supply → less need,
modulated by adoption lag).

### 3.3 Research funding and `phase_economy`

`economy_state.research_funding` is set by `phase_economy` (slot 6,
before `phase_research` runs in slot 13) as a fraction of the
`drained_joules`. A scenario can re-tune the fraction; the default
is a small percentage of the energy budget (TODO: pin a default
in the engine-side `Policy` — see §8.3 follow-up).

The fraction is the *downward-causation* leg: a pol pol pol can
*choose* to invest more in research (e.g. a "Scientific" policy
signal from `phase_policy` slot 5) by raising the research-funding
fraction. The *upward-causation* leg is the scarcity pressure in
§3.2 — a starving pol that *needs* agriculture research will
accumulate progress faster even at low funding.

### 3.4 Per-tick cap and bounded runaway

`MAX_RESEARCH_PER_TICK = 5_000` (ADR-020 §3 table) is the per-tick cap
on `research_progress` rise. It exists because research should not
leap a tier per tick under any combination of inputs. Combined with
`RESEARCH_TIER_MAX = 6` (the maximum `research_tier()` value — beyond
which `tech_unlocks_for_tier` is saturated), the system has a
finite, well-defined upper bound on progress accumulation.

The cap is named and grep-able per ADR-011 (shared-gradient
doctrine):

```text
rg -n 'MAX_(RESEARCH|RESEARCH_TIER|TECH_UNLOCKED)' crates/engine/src/engine.rs
```

must return ≥ 4 hits (one per bounded writer / const).

### 3.5 The replay story

`phase_research` is **deterministic** by construction — all inputs
are integer / saturating-add, no LLM call happens in this phase
(the LLM lane is lane 3 of §1.3 and is invoked from `phase_tech`
slot 14, not from `phase_research`). Two same-seed sims with the
same `(population, belief, cohesion, economy_state, world)` are
byte-identical after a `phase_research` call. The existing
`phase_research_*` tests in `EMERGENCE_TESTS_PLAN.md` (3-test
minimum per ADR-011) cover happy / boundary / decay paths.

### Acceptance criteria — `phase_research`

- **AC-R1**: `phase_research` advances `state.research_progress` by
  `min(MAX_RESEARCH_PER_TICK, delta)`, where `delta ≥ 0` and is
  computed purely from the inputs above. No hidden RNG. No external
  call. No wall-clock.
- **AC-R2**: `state.research_progress` is non-overflowing
  (`saturating_add`); the cap `MAX_RESEARCH_PER_TICK = 5_000` is
  enforced even when the natural `delta` would be larger.
- **AC-R3**: When `state.research_progress` crosses a tier boundary
  (the threshold that bumps `research_tier()` by 1 — pinned in
  §4.1), the corresponding candidate is sourced, validated, and on
  `Accept` is appended to `research_cache.researched` and
  `state.tech_unlocks` is updated by `phase_tech` in the same tick.
- **AC-R4**: When all input factors (population, belief, cohesion,
  funding, sentience) are zero, `delta = 0` and the phase is a
  no-op — no spurious progress (decay path).
- **AC-R5**: Two same-seed sims with identical inputs produce
  byte-identical `state.research_progress` and
  `research_cache.researched` after `phase_research` (replay-safe).

---

## 4. `phase_tech` — progress → unlocks (the tech tick)

### 4.1 Slot in `Simulation::tick` and tier ladder

`phase_tech` runs in slot 14 of `PHASE_ORDER` (engine.rs:1205, after
`phase_research` and before `phase_belief`). It is the
**sourcing + validation + insertion** pass.

- **Reads:** `state.research_progress` (post-`phase_research`),
  `state.population` (for `MAX_TECH_UNLOCKED_PER_TICK`),
  `economy_state` (for the *sourcing* pass — `deficit()` to seed
  `need_pressure`), `world` (ECS) for the cognitive-capacity
  signal, `law_db` (the in-scope `civ-laws::LawDb` —
  `crates/engine/src/scenario.rs` / `crates/research::seed_cards.ron`
  is the canonical seed).
- **Writes:** `state.tech_unlocks: u64` (bitmask via
  `tech_unlocks_for_tier(research_tier())`, which is **idempotent** —
  re-derived each tick, no per-tick rise of the bitmask itself;
  engine.rs:2035-2056); `state.research_progress` (consumed on tier
  boundary crossing); `research_cache.researched` (the canonical
  history of completed techs); `KnowledgeSet` per population (the
  new card).

The **tier ladder** is the 6-step bitmask: tier 0 = nothing, tier 1
= `TECH_IRRIGATION`, tier 2 = `TECH_STORAGE`, tier 3 =
`TECH_METALLURGY`, tier 4 = `TECH_WRITING`, tier 5 =
`TECH_SANITATION`, tier 6 = `TECH_GUNPOWDER` (engine.rs:2026-2056).
`research_tier()` (engine.rs:1043-1045) is a computed accessor over
`research_cache.researched.len()`, not a stored field — it is
**re-derived** each tick, so the bitmask is always coherent with
the cache.

The per-tier `research_progress` threshold is **per-card cost** in
`crates/research::seed_cards.ron` (a new RON file in the same
`include_str!` style as `crates/laws/laws/default.ron`). The default
costs are a monotone ladder — `cost(0) = 0`, `cost(1) = 5_000`,
`cost(2) = 12_000`, `cost(3) = 25_000`, `cost(4) = 50_000`,
`cost(5) = 100_000`, `cost(6) = 200_000` — so each tier takes
noticeably more accumulated progress than the last, but no tier
takes *infinite* progress (the system is bounded at `tier = 6`).

### 4.2 The per-tick algorithm

```
fn phase_tech(&mut self) {
    // 1. Re-derive the coarse bitmask (idempotent — no per-tick cap).
    let tier = self.research_tier();
    self.state.tech_unlocks = tech_unlocks_for_tier(tier);

    // 2. Boundary-crossing pass: did a tier boundary just fire this tick?
    let prev_tier = previous_tier(self);  // tracked in last_tick_tech_tier
    if tier > prev_tier {
        // 3. Sourcing + validation + insertion per new tier.
        for new_tier in (prev_tier + 1)..=tier {
            if let Some(card) = source_canonical_card(new_tier) {
                // The candidate surface is the canonical seed card for
                // this tier; a single card per tier. (LLM lane and
                // diffused lane are *additive* — see §1.3.)
                let law_db = self.law_db();
                match civ_research::validate(&card, &law_db) {
                    ValidationOutcome::Accept => {
                        // 4. Insert into KnowledgeSet + per-pop AdoptionState
                        for pop in self.populations() {
                            pop.knowledge_set.insert(card.id.clone());
                            pop.adoption_state.insert(
                                card.id.clone(),
                                AdoptionState::new(1e-3, default_diffusion_params())
                            );
                        }
                        self.research_cache_mut()
                            .researched
                            .push(card.id.clone());
                        // 5. Optional: emit chronicle line ("technological breakthrough")
                        //    — deferred to phase_chronicle per ADR-020 §3.3.
                    }
                    ValidationOutcome::Reject(reason) => {
                        // The card is rejected. Log + do not insert.
                        tracing::warn!(card = %card.id, ?reason, "tier card rejected by validate");
                    }
                }
            }
        }
    }

    // 6. Track the previous tier for next-tick boundary detection.
    self.last_tick_tech_tier = tier;

    // 7. (Future) LLM lane + diffused lane — additive, behind
    //    `mode != Canonical` gate and contact-network spread
    //    (see §5.2 and §1.3). Out of scope for v1 of the phase.
}
```

The boundary-crossing detector is the only way `state.tech_unlocks`
*changes* — re-deriving the bitmask from `research_tier()` is
idempotent, but the *transition* from `prev_tier` to `tier` is the
event that triggers sourcing + validation + insertion.

### 4.3 Why the bitmask is idempotent and the tier is a read-off

The 6-bit bitmask is a **read-off** over `research_tier()`, not a
sequence the player "climbs." A pol pol pol whose
`research_cache.researched` has 4 entries has `tech_unlocks =
TECH_IRRIGATION | TECH_STORAGE | TECH_METALLURGY | TECH_WRITING`,
regardless of *how* those 4 entries were obtained (invention in 4
different populations; 1 invention + 3 diffused; etc.). The bitmask
is a **measured pattern** over the world's tech frontier, not a
shared global state.

This is the same stance as `tech-engineering.md` §6 ("Eras are read
off, never set") and `era_label` (the percentile over adopted
techniques' era-rank) — eras and tech-unlocks are *both*
read-offs, *both* per-population, *both* emerge.

### 4.4 Per-tick cap and bounded runaway

The per-tick accepted count of new `KnowledgeSet` inserts is bounded
by `MAX_TECH_UNLOCKED_PER_TICK = 1` per population and
`MAX_TECH_UNLOCKED_PER_TICK_GLOBAL = 8` across the world. These
caps exist for the same reason as `MAX_RESEARCH_PER_TICK`:
preventing invention spikes that push the emergence dashboard
(per `docs/design/EMERGENCE_DASHBOARD.md`) off the
edge-of-chaos envelope.

In practice, the canonical 6-tier ladder bounds the *world's* max
accepts-per-tick to ≤ 6 (one per tier, ≤ 6 tiers), so the global
cap of 8 is the right envelope (allows up to 2 LLM-proposed cards
per tick in addition to a single canonical).

### 4.5 The replay story

`phase_tech` is **deterministic** in `Canonical` mode — the
candidate surface is the canonical seed cards, the validate gate is
pure, the tier ladder is fixed, the bitmask is re-derived from a
pure function. In `Hybrid` / `Free` mode, the LLM lane records an
`LlmEvent` (per `crates/research::replay_advance_llm_event`); the
*cache hit* is the replay contract — a cache miss refuses to
advance. The same `phase_tech_*` 3-test minimum applies (happy /
boundary / decay per ADR-011).

### Acceptance criteria — `phase_tech`

- **AC-T6**: `state.tech_unlocks` is a pure function of
  `research_tier()` (via `tech_unlocks_for_tier`), re-derived each
  tick. No per-tick mutation of the bitmask itself; the bitmask
  changes only when `research_tier()` changes.
- **AC-T7**: When `research_tier()` crosses a tier boundary, exactly
  one new canonical card is sourced for that tier and passed through
  `civ-research::validate`. On `Accept`, the card enters every
  population's `KnowledgeSet` and `research_cache.researched`. On
  `Reject`, the card is logged + discarded (no insert).
- **AC-T8**: A `Canonical` save never invokes the LLM lane;
  `phase_tech` sources only from canonical + diffused candidates.
- **AC-T9**: Two same-seed sims with identical inputs produce
  byte-identical `state.tech_unlocks`, `state.research_progress`,
  and `research_cache.researched` after `phase_tech` (replay-safe).
- **AC-T10**: `MAX_TECH_UNLOCKED_PER_TICK = 1` per population and
  `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL = 8` are enforced; the existing
  phantom-target test at `engine.rs:4656` (`sim.phase_tech()`) compiles
  (it is *not* a no-op once the method body is filled — see §10 WBS).

---

## 5. Adoption dynamics — Bass/Rogers S-curve per `(pop, C)`

### 5.1 Intra-population adoption

Each `(pop, C)` holds an `AdoptionState { f: f32 ∈ [0, 1], params:
DiffusionParams { p, q } }`. The `f` is the *adopted fraction* —
how much of the population actually *uses* `C` (not merely aware).
Advanced each tick by `civ-diffusion::advance(f, params)`:

```
f'(t) = (p + q · f) · (1 − f)
```

`civ-diffusion` is a pure math crate (`crates/diffusion/src/lib.rs:60-62`):
no RNG, no LLM, no I/O. It exposes `advance(f, params) -> f` and
`trajectory(f0, params, ticks) -> Vec<f32>` — the canonical
Bass-model S-curve.

**`p` and `q` are emergent, not constant.** `DiffusionParams` per
`(pop, C)` are *modulated* by substrate state — the meta-analysis
defaults `p ≈ 0.03, q ≈ 0.38` are only a starting prior:

- `p` ↑ with `need_pressure(pop, C)` (urgent tech catches on faster —
  the *upward-causation* leg from §2.1 back into §5.1: the same
  pressure that *invented* `C` also *spreads* it).
- `p` ↑ with `resource_availability(pop, C)` (can't adopt what you
  can't supply — if `C.inputs` are scarce, the *practitioners* are
  few, so spontaneous uptake is rare; this is **knowledge-before-
  capability** in action).
- `q` ↑ with population density / contact-network connectivity (more
  neighbours to imitate).
- `q` ↑ with cultural openness (an emergent ideology metric,
  `crates/agents::CultureProfile::openness` — modifiable per
  population; not stored on a global enum).
- `q` ↓ with cultural conservatism / taboo (emergent, also from
  `CultureProfile`) — modelling tech that stalls despite being known.

This makes adoption **gradual and uneven** — visible tech (tools,
wardrobe, architecture) propagates across a civ rather than
snap-upgrading, exactly the stated purpose of `crates/diffusion`.

### 5.2 Inter-population spread — knowledge transmission

A technique crosses from population A to neighbour B along the
**contact / kinship / trade network** — the same graph used for
culture / language drift (`crates/agents::social.rs`,
`crates/agents::culture.rs`). When `A.adoption[C]` is high (above an
emergence spread threshold, e.g. `f ≥ 0.4`) and an A↔B contact
edge exists, B *learns* `C` (adds it to `B.KnowledgeSet` with
`f ≈ ε`) — provided B can satisfy `C.inputs` *or* import them via
`civ-economy` trade.

Spread rate along an edge scales with **contact intensity** —
trade volume from `civ-economy::propose_trade` / `apply_trade`,
migration, shared culture. This is why **trade routes are tech
vectors**: comparative-advantage trade moves goods *and* carries
techniques. A pol that imports iron ore from a neighbour whose
`TECH_METALLURGY` is widely adopted will, over time, learn
metallurgy itself — not because of a timer, but because the
*substrate pressure* (incoming iron + knowledge in traders'
heads + neighbour who knows the technique) clears the
three-pressure gate.

**Resource gating on adoption (not on learning):** B may *know* `C`
yet keep `f ≈ 0` if it cannot source `C.inputs` — knowledge
outruns capability until trade or local extraction catches up.
Realistic and emergent.

### 5.3 Per-tick cap and bounded runaway

`civ-diffusion::advance` is monotone non-decreasing under non-negative
`(p, q)` and saturates at `1.0` (existing `civ-diffusion` tests
`tick_increase_matches_closed_form`, `saturation_produces_zero_increase`,
`trajectories_are_monotone_nondecreasing`). No per-tick cap is
required at the diffusion math layer — the cap is the substrate
input (the `(p, q)` params are bounded per-population in the
*modulator* that derives them, not in `advance` itself).

The modulator's bounds are pinned: `p ∈ [0.0, 0.5]`, `q ∈ [0.0,
0.8]`. Outside these envelopes, the params are clamped. The
empirically-validated Bass/Rogers meta-analysis range is
`p ∈ [0.0, 0.1]`, `q ∈ [0.0, 0.6]`; the engine envelope is wider
on purpose to allow extreme-but-bounded cases (urgent
adoption-of-desperation, viral-adoption).

### Acceptance criteria — adoption

- **AC-A1**: `AdoptionState.f` is monotone non-decreasing under
  non-negative `(p, q)` (inherits `civ-diffusion` FR-CIV-DIFFUSION-003)
  and saturates at ≤ 1.0.
- **AC-A2**: Higher `need_pressure` ⇒ higher effective `p` ⇒ faster
  early adoption for the same `q`. The two pressures (invention +
  adoption) share a single substrate signal — no double-counting.
- **AC-A3**: A technique known by A spreads to a *contacting*
  neighbour B but not to an isolated population C with no path to A.
  `crates/agents::SocialGraph` is the contact graph; BFS over it
  bounds the spread.
- **AC-A4**: B can hold `C` in `KnowledgeSet` with `f ≈ 0` while
  `C.inputs` are unavailable, then `f` rises once trade supplies the
  inputs (knowledge-before-capability).
- **AC-A5**: Cutting the A↔B contact edge halts further
  cross-population spread along it; existing knowledge is retained
  by both A and B (the edge is a *transmission channel*, not a
  *membership* relation).

---

## 6. Era labels — read-off over adopted techniques (no gate)

### 6.1 Era-rank of a technique

Each canonical `TechCard` carries an `era: u16` (lib.rs:32) — the
era-rank label, derived from the dominant dependency's
`civ-laws::Law::era_min`. The default seed ranks are pinned:
`tech_irrigation = 1`, `tech_storage = 2`, `tech_metallurgy = 3`,
`tech_writing = 4`, `tech_sanitation = 5`, `tech_gunpowder = 6`.
Fictional / LLM-proposed cards use the era-rank of their dominant
dependency; the rank is **sortable but not gating**.

### 6.2 Era label of a population (a percentile read-off)

```
era_label(pop) =
    percentile(  // e.g. p70
        [t.era for t in pop.knowledge_set if pop.adoption[t.id].f >= 0.5],
        0.70,
    )
```

An era transition is the event "`era_label(pop)` crosses an integer
boundary." Transitions are emitted to the **legends / event log**
(`docs/design/legends-engine.md`, `docs/research/ai-rnd.md` §1.1) —
never used to *unlock* anything. Unlocking is governed by knowledge
(§2) + laws (§1.4), not by era.

Because the label is driven by the *adoption* `f` values (§5),
**era advance is gradual and population-specific** — a civ "enters
the Iron Age" when iron tools have *diffused widely enough*, not
when the first smith invents them. Regions age at different rates;
a world holds multiple coexisting eras simultaneously (a
steam-power core trading with a stone-tool periphery) — emergent,
not scripted.

### 6.3 Regression

If adoption collapses (knowledge loss via §3 of
`tech-engineering.md`, depopulation, resource exhaustion cutting
`f` back toward 0), `era_label` *falls* — an emergent dark age,
surfaced to legends as a measured regression. The bitmask
`state.tech_unlocks` is **not** affected by regression — once a
tech is in `research_cache.researched`, it stays there. Only the
*adoption* `f` and the derived `era_label` fall.

### Acceptance criteria — era labels

- **AC-EL1**: No code path *gates* a capability on `era_label`;
  era is purely descriptive / observational. The compiler-enforced
  test is a grep: `rg 'era_label' crates/ | grep -v '///' | grep -v
  '//'` must show *only* read sites, no write sites outside
  `phase_emergence` (which writes the label, not a gate).
- **AC-EL2**: `era_label(pop)` rises only as adopted-technique `f`
  values rise (driven by `civ-diffusion`), not on invention alone.
- **AC-EL3**: Two contacting populations can hold different
  `era_label`s simultaneously (uneven aging) and the world reports
  both.
- **AC-EL4**: A modelled adoption collapse lowers `era_label`
  (regression is representable and emitted to legends).

---

## 7. The full per-tick DAG (what each phase reads / writes)

### 7.1 Shared-gradient contract (per ADR-011 + ADR-018)

Every coupling is a **shared gradient**, not an API call. The
producer writes a value that the consumer reads off the same
`Simulation`. Every per-tick delta has a `const` cap nearby. The
cap is the invariant that keeps the system on the edge of chaos.

For tech progression specifically:

| Phase | Shared gradient read | Shared gradient written | Cap (`const`) | Per-tick cap name |
|---|---|---|---|---|
| `phase_research` (slot 13) | `state.population`, `state.belief` (stale-allowed), `state.cohesion` (stale-allowed), `economy_state.research_funding`, `mean_psyche_maturity`, `mean_psyche_valence` | `state.research_progress: u64` | `MAX_RESEARCH_PER_TICK = 5_000` | `MAX_RESEARCH_PER_TICK` |
| `phase_tech` (slot 14) | `state.research_progress` (post-`phase_research`), `state.population` (cap), `economy_state` (sourcing), `law_db` (validation), `world` (cognitive capacity), `last_tick_tech_tier` (boundary detection) | `state.tech_unlocks: u64` (idempotent, re-derived from `research_tier()`), `research_cache.researched` (per-card append on boundary), per-population `KnowledgeSet` (insert on boundary) | `MAX_TECH_UNLOCKED_PER_TICK = 1` per population; `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL = 8`; tier max `RESEARCH_TIER_MAX = 6` | `MAX_TECH_UNLOCKED_PER_TICK`, `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL`, `RESEARCH_TIER_MAX` |

### 7.2 What `phase_research` reads (in DAG order)

```
phase_research reads:
    state.population                 // from phase_life (slot 12)
    state.belief                     // stale — prior tick's phase_belief output
    state.cohesion                   // stale — prior tick's phase_cohesion output
    economy_state.research_funding   // from phase_economy (slot 6)
    world (ECS):
        mean_psyche_maturity         // from phase_emergence (slot 22) prior tick
        mean_psyche_valence          // ditto
    state.economic_focus             // from phase_economic_focus settle (slot 21) prior tick
```

The stale-allowed read on `state.belief` and `state.cohesion`
introduces a single-tick lag in the upward-causation feedback;
this lag is bounded (≤ 1 tick = `O(1)`) and named (ADR-020 §2
"Note on belief/cohesion latency").

### 7.3 What `phase_tech` reads (in DAG order)

```
phase_tech reads:
    state.research_progress          // from phase_research (slot 13) this tick
    state.population                 // from phase_life (slot 12) this tick
    economy_state                    // from phase_economy (slot 6) — for deficit() sourcing
    law_db                           // the in-scope civ-laws::LawDb
    last_tick_tech_tier              // self, for boundary detection
    world (ECS):
        per-population cognitive_capacity  // aggregate of Psyche.maturity
```

`phase_tech` runs *immediately* after `phase_research`, so
`state.research_progress` is the freshest possible. No staleness.

### 7.4 What downstream consumers read

| Consumer phase (slot) | Reads from `phase_research` | Reads from `phase_tech` |
|---|---|---|
| `phase_belief` (15) | — | `state.tech_unlocks` (downward causation on belief via `awakening_belief_gain` if sentience cross threshold) |
| `phase_unrest` (16) | — | `state.tech_unlocks` (sanitation, storage, irrigation each modulate unrest via `commodity_unrest_delta`) |
| `phase_cohesion` (17) | — | `state.tech_unlocks` (writing → legibility → cohesion pulse) |
| `phase_economic_focus_pre` (18) | — | `state.tech_unlocks` (metallurgy → industrial; writing → mercantile; irrigation → agrarian) |
| `phase_stratification` (19) | — | `state.tech_unlocks` (gunpowder shifts the dispossession envelope) |
| `phase_institutions` (20) | — | `state.tech_unlocks` (writing → bureaucracy → garrison_level gate) |
| `phase_economic_focus` (21) | — | `state.tech_unlocks` (settle pass) |
| `phase_emergence` (22) | `state.research_progress` (record as macro scalar) | `state.tech_unlocks` (record as macro scalar; feed legends) |
| `phase_diffusion` (23) | — | `state.tech_unlocks` → `target_era` for wardrobe/tools propagation (existing path, engine.rs:1550-1575) |

The **downward-causation** legs are the key insight: the 6-bit
bitmask is the *interface* that downstream phases consume. The
bitmask is **idempotent** (re-derived from `research_tier()`), so
it is always coherent with the cache, and the downstream phases
do not need to track *which tier* fired — only *which bits are
set*. The `phase_diffusion` leg in particular uses
`state.research_tier()` (engine.rs:1043-1045) as the `target_era`
for wardrobe + tools era propagation — exactly the existing
contract, but now with the emergent source.

### 7.5 Per-tick cap audit (the grep)

The per-tick caps are the *invariants* that keep the system on the
edge-of-chaos. They are named, grep-able, and reviewed on every
PR:

```text
rg -n 'MAX_(RESEARCH|TECH_UNLOCKED|RESEARCH_TIER)' crates/engine/src/engine.rs
```

must return ≥ 4 hits (one per bounded writer / const):

- `MAX_RESEARCH_PER_TICK = 5_000` (const, near the phase — new)
- `MAX_TECH_UNLOCKED_PER_TICK = 1` (const, near the phase — new)
- `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL = 8` (const, near the phase — new)
- `RESEARCH_TIER_MAX = 6` (const, near `tech_unlocks_for_tier` — new)

A future PR that adds a new shared-gradient writer to the tech
progression phases must add its cap to this set (per ADR-011
shared-gradient doctrine: **No new coupling is accepted on review
without a named cap and a 3-test minimum**).

### 7.6 Determinism guard (per ADR-003)

All inputs to both phases are integer / saturating-add / pure
function of substrate state. No RNG is consumed in `phase_research`
or in the canonical-lane pass of `phase_tech`. The LLM lane
(lane 3 of §1.3) is the *only* non-deterministic surface, and
it is gated to `Hybrid` / `Free` saves with cache-hash replay
support per `crates/research::replay_advance_llm_event`. Two
same-seed sims with the same save mode produce byte-identical
`state.research_progress`, `state.tech_unlocks`, and
`research_cache.researched`.

---

## 8. Crate mapping (extend, never duplicate)

| Capability | Crate | New surface (additive) |
|---|---|---|
| Technique record + validation | `crates/research` | `seed_cards.ron` (canonical seed cards, mirror `civ-laws` RON convention); `KnowledgeSet` type (per-population set of `TechCard.id`s); `AdoptionState` type (per-`(pop, C)` `f` + `DiffusionParams`) |
| Physics/material authority | `crates/laws` | none — used as-is as the validator authority; seed canonical era-rank `era_min`s in the existing RON DB |
| Per-technique adoption spread | `crates/diffusion` | emergent-`DiffusionParams` modulation hook (params derived from need/density/culture); no math change — `advance` is unchanged |
| Need pressure | `crates/needs` | read-only consumer; expose aggregate deficit per population (`mean_needs_deficit(pop)`); no new fields |
| Resource availability + trade vectors | `crates/economy` | read-only consumer of `Stocks` / `deficit` / `comparative_advantage` / `propose_trade`; no new fields |
| Cognitive capacity / knowledge carriers | `crates/agents` | derive `cognitive_capacity` from `Civilian` + `Psyche` aggregates; `KnowledgeSet` membership carried on `Civilian` (or a `KnowledgeCarrier` component if the per-civilian set is too fine-grained) |
| Orchestration (per-pop knowledge/adoption/era state) | `crates/engine` | `phase_research` + `phase_tech` method bodies (filling the empty stubs at engine.rs:1204-1205); `state.research_progress` + `state.tech_unlocks` are already declared in `WorldState` (per the existing engine shape); per-population `KnowledgeSet` + `AdoptionState` storage on `Simulation` (BTreeMap by cluster id); `last_tick_tech_tier` field for boundary detection |
| AI proposal lane | `crates/ai` (per `ai-rnd.md`) + `crates/research` | async novel-card proposals behind the existing `LlmClient` / `AiProvider` port; **out of scope for v1 of the phases** — wired behind a `mode != Canonical` feature flag |
| Narration of inventions/era transitions | legends / event log | emit `invention`, `era_transition`, `regression` events to the saga graph (legends-engine.md, ai-rnd.md §1.1) — **deferred to `phase_chronicle`** per ADR-020 §3.3 |

> **Cross-project reuse:** the `readiness = need × resource ×
> knowledge` gate + the diffusion-params-from-substrate modulation are
> generic "emergent capability unlock" primitives — candidate for a
> shared Phenotype substrate alongside the `civ-ai` extraction (flag
> per the reuse protocol; confirm destination before extracting).

---

## 9. FR catalogue — `FR-CIV-TECH-*`

> The existing FR namespace `FR-CIV-RESEARCH-*` lives in
> `crates/research/` (lib.rs:9, FR-CIV-RESEARCH-000…033) and covers
> the validator + cache + replay layer. This spec adds the
> engine-shaped runtime FR namespace `FR-CIV-TECH-*` covering
> `phase_research` + `phase_tech` and the adoption / era mechanics.
> The two namespaces are orthogonal: `RESEARCH` is the validator /
> LLM lane / cache; `TECH` is the engine phase that drives the
> candidate sourcing + acceptance + diffusion + era read-off.

| FR | Requirement | Verifies | Maps to AC |
|---|---|---|---|
| **FR-CIV-TECH-001** | Invention readiness is the multiplicative product of need, resource, and knowledge factors; any zero factor blocks invention. | `readiness` unit tests | AC-T1, AC-T3 |
| **FR-CIV-TECH-002** | A technique is added to a population's `KnowledgeSet` only after passing `civ-research::validate` against the current `LawDb`. | validation-gate test | AC-PS3 |
| **FR-CIV-TECH-003** | Invention requires `knowledge_fraction == 1.0` (all prerequisites known) for from-scratch invention. | prerequisite-gate test | AC-T2, AC-PS2 |
| **FR-CIV-TECH-004** | `cognitive_capacity` modulates invention hazard (pre-sentient ⇒ ~0). | hazard-scaling test | AC-T1 |
| **FR-CIV-TECH-005** | Knowledge is per-population and partial; no global tech-level field exists. | architecture / grep | AC-PS1 |
| **FR-CIV-TECH-006** | Knowledge loss is representable: extinguishing the last holder + severing contact removes a technique from a region's frontier. | knowledge-loss test | AC-A3, AC-A5 |
| **FR-CIV-TECH-007** | Canonical saves never emit `LlmEvent`s; `phase_tech` sources only from canonical + diffused candidates. | replay-mode test | AC-PS4, AC-T8 |
| **FR-CIV-TECH-008** | LLM-proposed cards are validated by the same `validate`; rejects never enter any `KnowledgeSet`. | LLM-gate test | AC-PS5 |
| **FR-CIV-TECH-009** | `phase_research` advances `state.research_progress` by `min(MAX_RESEARCH_PER_TICK, delta)` where `delta ≥ 0` is a pure function of `(population, belief, cohesion, economy_state, world)`. | progress-cap test | AC-R1, AC-R2 |
| **FR-CIV-TECH-010** | `state.tech_unlocks` is a pure function of `research_tier()` via `tech_unlocks_for_tier`, re-derived each tick. | bitmask-derivation test | AC-T6 |
| **FR-CIV-TECH-011** | When `research_tier()` crosses a tier boundary, exactly one new canonical card is sourced for that tier and passed through `civ-research::validate`. | tier-boundary test | AC-T7 |
| **FR-CIV-TECH-012** | Per-`(pop, C)` adoption advances via `civ-diffusion::advance`; monotone non-decreasing, saturates ≤ 1.0. | diffusion-integration test | AC-A1 |
| **FR-CIV-TECH-013** | `DiffusionParams` are modulated by emergent substrate (need ↑p; density/openness ↑q; conservatism ↓q; resource ↑p). | param-modulation test | AC-A2 |
| **FR-CIV-TECH-014** | A technique spreads across a contact edge to a neighbour but not to an isolated population. | cross-population test | AC-A3, AC-A5 |
| **FR-CIV-TECH-015** | Knowledge can precede capability: a population holds a technique with `f≈0` until trade/extraction supplies inputs. | knowledge-before-capability test | AC-A4 |
| **FR-CIV-TECH-016** | `era_label(pop)` is a percentile read-off over adopted techniques' era-rank; no capability is gated on it. | era-derivation test | AC-EL1, AC-EL2 |
| **FR-CIV-TECH-017** | Multiple eras coexist across contacting populations (uneven aging). | multi-era world test | AC-EL3 |
| **FR-CIV-TECH-018** | Adoption collapse lowers `era_label`; regression is representable. | regression test | AC-EL4 |
| **FR-CIV-TECH-019** | Two same-seed sims with identical inputs produce byte-identical `state.research_progress`, `state.tech_unlocks`, and `research_cache.researched` after `phase_research` + `phase_tech` (replay-safe in `Canonical` mode). | replay test | AC-R5, AC-T9 |
| **FR-CIV-TECH-020** | Per-tick cap audit: `MAX_RESEARCH_PER_TICK = 5_000`, `MAX_TECH_UNLOCKED_PER_TICK = 1`, `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL = 8`, `RESEARCH_TIER_MAX = 6` are all declared and enforced. | cap-audit test (grep + runtime) | AC-R2, §7.5 |

---

## 10. Phased WBS (DAG)

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| **P0 — Research-state landing (engine shape)** | **T0** | Fill in the empty `phase_research` and `phase_tech` method bodies (currently stub at `engine.rs:1204-1205`) per §3 + §4. Promote `state.research_progress: u64` and `state.tech_unlocks: u64` to explicit `WorldState` fields (already referenced by `apply_replay_research` at `engine.rs:946-953` and by the existing `tech_unlocks_for_tier` ladder). Add `last_tick_tech_tier: u64` to `Simulation` for boundary detection. | — |
| P0 | T0.1 | Declare the four new `const` caps: `MAX_RESEARCH_PER_TICK`, `MAX_TECH_UNLOCKED_PER_TICK`, `MAX_TECH_UNLOCKED_PER_TICK_GLOBAL`, `RESEARCH_TIER_MAX` (per §7.5). | — |
| P0 | T0.2 | Implement `phase_research` body (§3.2): integer accumulation, no RNG, no LLM, no wall-clock. | T0, T0.1 |
| P0 | T0.3 | Implement `phase_tech` body (§4.2): boundary detection, canonical sourcing, `validate`, `KnowledgeSet` insert, idempotent bitmask re-derivation. | T0, T0.1 |
| P0 | T0.4 | The existing phantom-target test at `engine.rs:4656` (`sim.phase_tech()`) compiles and passes. | T0.3 |
| **P1 — Possibility-space** | **T1** | Add `crates/research::seed_cards.ron` — canonical seed cards in RON, mirroring the `civ-laws` RON convention. Six cards in v1, one per tier (`tech_irrigation`, `tech_storage`, `tech_metallurgy`, `tech_writing`, `tech_sanitation`, `tech_gunpowder`), each with a `cost` field and `dependencies` referencing `LawDb` ids. | T0 |
| P1 | T2 | Add `KnowledgeSet` and `AdoptionState` types in `crates/research` (per §8 mapping). | T0 |
| P1 | T3 | Add `phase_tech` candidate sourcing for the canonical lane (v1): walks `seed_cards.ron`, filters by `dependencies ∈ knowledge_set`, applies §2.1 `readiness` gate, draws Bernoulli trial, calls `validate`. | T1, T2 |
| P1 | T4 | Per-population `KnowledgeSet` + `AdoptionState` storage on `Simulation` (BTreeMap by cluster id) — the engine-side carrier. | T2 |
| **P2 — Diffusion / Adoption** | **T5** | `civ-diffusion` integration: per-`(pop, C)` `AdoptionState` advanced by `civ-diffusion::advance(f, params)` each tick. | T4 |
| P2 | T6 | `DiffusionParams` modulation hook: `p` ↑ with `need_pressure` + `resource_availability`; `q` ↑ with density / openness; `q` ↓ with conservatism. (Per §5.1.) | T5, `civ-agents::CultureProfile::openness` |
| P2 | T7 | Inter-population spread: technique crosses contact edges when `A.adoption[C].f ≥ 0.4`. Spread rate scales with contact intensity. | T5, `civ-agents::SocialGraph` |
| P2 | T8 | `era_label(pop)` percentile read-off over adopted techniques. Emit `era_transition` events to the saga graph. | T5, `civ-legends` |
| **P3 — LLM lane (gated)** | **T9** | LLM proposal path: when canonical + diffused lanes return no candidates and need is high, enqueue an `AiTask` per `ai-rnd.md §4.3`. Gated to `mode != Canonical`. Replay-cached via `LlmEvent::cache_key`. | T3, `civ-ai` P1 (`ai-rnd.md`) |
| **P4 — Dev-assist / observability** | **T10** | Add tech-progression metric streams to the emergence dashboard: per-tick `delta(research_progress)`, `delta(tech_unlocks)`, mean `f` per `(pop, C)`, era-label distribution. Per-tick cap alarms when `MAX_*` triggers. | T0, T5, T8 |
| P4 | T11 | Update `EMERGENCE_TESTS_PLAN.md` with the 3-test minimum per phase (happy / boundary / decay), per ADR-011. | T0 |

**Aggressive agent-effort:** P0 ≈ 1 impl subagent / ~10 min wall (the
method bodies are the smallest concrete deliverable); P1 ≈ 2-3
parallel subagents / ~15 min (seed RON, types, sourcing, storage);
P2 ≈ 2-3 subagents / ~15 min (diffusion integration, modulation
hook, contact-edge spread, era read-off); P3 ≈ 1-2 subagents / ~10
min (LLM lane, gated); P4 ≈ 1-2 subagents / ~10 min (dashboard +
tests).

---

## 11. Summary of the two headline models

**Emergent progression (invention):** a population invents a
technique when **need × resource × knowledge** clears a
multiplicative gate, scaled by emergent cognitive capacity. The
candidate is sourced from three lanes (canonical seed cards,
diffused-from-neighbour, LLM-proposed) and **law-validated** by
`civ-research::validate` against the authored `civ-laws` DB before
it can exist. The wired `phase_research` turns the pressure triple
into `state.research_progress`; the wired `phase_tech` turns
accumulated progress into a 6-bit `state.tech_unlocks` ladder via
the existing `tech_unlocks_for_tier` function — *idempotent*,
re-derived each tick, never *gated* on. No global tech tree —
knowledge is per-population, partial, transmissible, and losable.

**Adoption dynamics (spread + eras):** a known technique spreads
via the Bass/Rogers S-curve (`civ-diffusion`) — intra-population
adoption with substrate-modulated `p` / `q`, and inter-population
transmission along the contact / trade network (trade routes are
tech vectors). **Eras are read off, not gated**: `era_label(pop)`
is a percentile over the era-rank of *adopted* techniques, so a
civ ages gradually as tech *diffuses widely*, not when it is first
invented. Multiple eras coexist, and adoption collapse produces
emergent dark-age regression. Capability is always governed by
known+adopted techniques and the laws DB — never by the era label
or the tier itself; the 6-bit bitmask is the *observation surface*
for downstream phases, not a *gating* surface.
