# Emergence Tests Plan — Proving the 11 Dormant Phases *Emerge*

> Companion spec for [ADR-020](../adr/ADR-020-wire-dormant-emergence-phases.md)
> and [EMERGENCE_WIRING_PATCHPLAN](EMERGENCE_WIRING_PATCHPLAN.md).
> Read-only: no code changes here. The implementer writes the tests
> in a follow-up engine PR (or per-phase PR group) once the call sites
> in `Simulation::tick` land and the `state.*` fields are promoted.

**Scope.** Design the unit/integration tests that prove each of the 11
dormant emergence phases (plus the `phase_emergence` orchestrator)
**actually emerges** — not just that the call executes. The bar is the
**emergence quality** of the output (power-law shape, Shannon entropy,
6-connectivity structure count, Zipf/Mandelbrot tails, branching
ratio `σ̄_W`, boundedness), per the charter and ADR-011. Hardcoded
"value is X" assertions are an anti-pattern: they pass when the phase
is wired to a constant. Power-law / entropy / structure-count /
fixed-point invariants **fail** when a phase is hardcoded.

**Source-of-truth citations.**

- `crates/engine/src/engine.rs:55-68` — `PHASE_ORDER` (current 12 entries).
- `crates/engine/src/engine.rs:1183-1211` — `Simulation::tick` (does not
  yet call any of the 11 new phases).
- `crates/engine/src/emergence.rs:159-171` — `phase_emergence` orchestrator
  (dead outside `#[cfg(test)]` until wiring lands).
- `crates/engine/src/emergence_metrics.rs:66` — `EMERGENCE_SAMPLE_INTERVAL = 50`.
- `crates/engine/src/emergence_metrics.rs:149-220` — `EmergenceSample` shape
  (entropy, structure_count, power_law_alpha, branching_sigma, novelty_rate).
- `crates/legends/src/graph.rs:139-142` — `SagaGraph::node_count / edge_count`.
- `crates/civ-emergence-metrics/src/power_law.rs` — `PowerLawFit` (α).
- `crates/civ-emergence-metrics/src/shannon.rs` — `ShannonEntropy`.
- `crates/civ-emergence-metrics/src/structure.rs` — `StructureCount` (CC).
- `crates/civ-emergence-metrics/src/branching.rs` — `BranchingLedger`,
  `rolling_mean_sigma`, `classify_regime`, `SIGMA_SUBCRITICAL = 0.85`,
  `SIGMA_SUPERCRITICAL = 0.95`.

---

## 0. Test-design philosophy

Every test in this plan asserts an **emergent property** rather than a
hardcoded value. The four families of property:

| Family | Property | What it rules out |
|--------|----------|-------------------|
| **Power-law** | rank-frequency distribution over cluster sizes, agent counts, or activity streams fits `P(r) ∝ r^{-α}` with `α ∈ [1.5, 3.5]` | uniform output (no structure), single-tower monopoly, hardcoded constant |
| **Entropy** | Shannon entropy of a categorical histogram (cluster labels, belief lanes, voxel material) is in `(0.1, log₂ N)` (i.e., diversified, not collapsed) | all-one-bin output (no emergence), perfectly uniform (no structure) |
| **Structure count** | 6-connectivity component count on a binary mask is `> 1` and bounded above by a sensible multiple of expected structures | single mega-cluster (monoculture), no clusters (failure to form), unbounded proliferation (feedback run-away) |
| **Boundedness / determinism** | per-tick scalar step `≤ MAX_*`, two same-seed sims byte-equal across `N` ticks | feedback explosion (the "edge of chaos" risk in ADR-020 §4), non-determinism (charter §3.1) |

The 3-test ADR-011 minimum is **happy / boundary / decay** (smoke + cap +
negative). This plan adds **emergence-quality** tests (power-law / entropy
/ structure-count) on top so that a phase wired to a constant *fails* one
of them. A future engineer who tries to land a stub `fn phase_X(&mut self) {}`
body — pass-through with no emergent behaviour — will see at least one
emergence-quality test fail and either implement the real body or justify
why a different test covers it.

---

## 1. Total test budget

| # | Phase | Smoke (happy) | Boundary / cap | Decay (negative) | Emergence-quality | **Phase total** |
|---|-------|---------------|----------------|------------------|-------------------|-----------------|
| 1 | `phase_life` | 1 | 1 | 1 | 2 | **5** |
| 2 | `phase_research` | 1 | 1 | 1 | 2 | **5** |
| 3 | `phase_tech` | 1 | 1 | 1 | 1 | **4** |
| 4 | `phase_belief` | 1 | 1 | 1 | 2 | **5** |
| 5 | `phase_unrest` | 1 | 1 | 1 | 2 | **5** |
| 6 | `phase_cohesion` | 1 | 1 | 1 | 2 | **5** |
| 7 | `phase_social_mood` | 1 | 1 | 1 | 2 | **5** |
| 8 | `phase_stratification` | 1 | 1 | 1 | 2 | **5** |
| 9 | `phase_institutions` | 1 | 1 | 1 | 1 | **4** |
| 10 | `phase_economic_focus` | 1 | 1 | 1 | 2 | **5** |
| 11 | `phase_emergence` (orchestrator) | 1 | 1 | 1 | 3 | **6** |
| **Subtotal** | | 11 | 11 | 11 | **21** | **54** |
| Cross-cutting (determinism + perf guard) | | | | | | **3** |
| **Grand total** | | | | | | **57 tests** |

Per-phase design is detailed in §3. Cross-cutting tests in §4.

---

## 2. Test placement

Each phase's tests live as `#[cfg(test)] mod tests` next to the
implementation — matching the existing convention at
`crates/engine/src/emergence.rs:730-1098`,
`crates/engine/src/emergence_metrics.rs:1241+`, and
`crates/engine/src/engine.rs:3362+`. Integration tests (cross-phase
emergence-quality + replay determinism) live in a new file
`crates/engine/tests/emergence_wiring.rs`. The 57-test total splits
roughly **33 unit + 24 integration**.

Test file paths:

- `crates/engine/src/emergence.rs` — extend existing `mod tests` with
  the `phase_emergence` orchestrator tests (smoke + emergence-quality).
- `crates/engine/src/emergence_life.rs` *(new — implementer adds when
  landing)* — `phase_life` tests.
- `crates/engine/src/emergence_research.rs` *(new)* — `phase_research`,
  `phase_tech`.
- `crates/engine/src/emergence_psyche.rs` *(new)* — `phase_belief`,
  `phase_unrest`, `phase_cohesion`, `phase_social_mood`.
- `crates/engine/src/emergence_macro.rs` *(new)* — `phase_stratification`,
  `phase_institutions`, `phase_economic_focus`.
- `crates/engine/tests/emergence_wiring.rs` *(new)* — cross-phase
  determinism, replay byte-equality, emergence-quality invariants.

(Implementer is free to fold new modules into existing files if the
reviewer prefers; the function names are the contract.)

---

## 3. Per-phase test design

Each phase has a **DAG contract** (what it reads + writes), a
**boundedness contract** (the `MAX_*` cap), and an
**emergence-quality property** (the new test). Every test must
**fail when the body is `let _ = ();`** (a pass-through stub).

### 3.1 `phase_life` — settlement commons (slot 12)

**Source.** EMERGENCE_WIRING_PATCHPLAN §3.1. Reads `population`,
`ClusterMember`, `Needs`; writes `cluster_stocks`, `last_settlement_count`,
`last_life_deaths`. Const cap: `CLUSTER_FOOD_PRODUCTION_PER_MEMBER = 1`,
`CLUSTER_FOOD_CONSUMPTION_PER_MEMBER = 1`.

**Tests** (`crates/engine/src/emergence_life.rs`):

1. **`phase_life_commits_two_plus_clusters`** *(smoke)*.
   Spawn 6 civilians with `Position3d` clustered as `(0,0,0)` x3 and
   `(100,0,0)` x3. Run 1 tick. Assert
   `sim.settlement_count() >= 2` and `sim.cluster_stocks().len() >= 2`
   (`SETTLEMENT_MIN_MEMBERS = 2` at `engine.rs:2828`).
   *Fails-on-stub:* `cluster_stocks` is empty.

2. **`phase_life_drops_singleton_clusters`** *(boundary)*.
   Spawn 3 civilians each in their own non-touching cluster
   (`(0,0,0)`, `(200,0,0)`, `(400,0,0)`). Run 1 tick. Assert
   `sim.cluster_stocks().is_empty()` (the `emergence.rs:198` `size < 2`
   filter).
   *Fails-on-stub:* singletons committed.

3. **`phase_life_food_commons_bounded`** *(decay / boundedness)*.
   Spawn a 10-member cluster at `(0,0,0)`. Run 1,000 ticks. Assert
   `cluster_stocks[0].food` absolute value `≤ 10 * 1_000` (the matched
   rate cap; no runaway even if other phases misbehave).
   *Fails-on-stub:* food value is hardcoded constant or unbounded.

4. **`phase_life_cluster_size_distribution_is_zipfian`** *(emergence-quality)*.
   Spawn 50 civilians at hand-placed positions forming 5 clusters of
   sizes 1, 2, 5, 12, 30 (Zipf-like). Run 1 tick, then call
   `sim.sample_emergence()`. Pull `cluster_member_counts` from the
   ECS, feed to `civ_emergence_metrics::power_law::PowerLawFit`, assert
   `0.5 < alpha < 3.5` *and* `r_squared > 0.7`. A pure pass-through
   stub produces alpha = 0 (sentinel) or undefined.
   *Fails-on-stub:* fit not run, alpha = 0 sentinel.

5. **`phase_life_settlement_count_loglinear_in_population`** *(emergence-quality)*.
   For populations in `[10, 50, 200, 1000]` with random co-location,
   run 1 tick each, record `sim.settlement_count()`. Assert the
   4-point series has Spearman `ρ > 0.7` with `log(population)`.
   Tests that **clustering scales sub-linearly with population**
   (the emergence signature; pure random collisions would give
   `count ∝ sqrt(N)`, which is *not* a power-law with `α ≈ 2` but
   still sub-linear — the Spearman check accepts both regimes while
   ruling out the `count = N` "no clustering" stub).
   *Fails-on-stub:* count equals `population` (no clustering).

---

### 3.2 `phase_research` — research progress (slot 13)

**Source.** PATCHPLAN §3.2. Reads `population`, `belief`, `cohesion`,
`economy_state`; writes `state.research_progress` capped by
`MAX_RESEARCH_PER_TICK = 5_000`.

**Tests** (`crates/engine/src/emergence_research.rs`):

1. **`phase_research_increments_progress`** *(smoke)*.
   Spawn 100 civilians. Run 50 ticks. Assert
   `sim.research_cache().researched.is_empty()` (still researching —
   no tier 1 yet at 5_000 × 50 = 250k and the tier-1 threshold is
   higher; this verifies the producer, not the consumer) and
   `sim.research_progress() > 0`.
   *Fails-on-stub:* `research_progress == 0`.

2. **`phase_research_clamps_at_max`** *(boundary)*.
   Spawn 1_000_000 civilians + populate `state.belief = u64::MAX / 4`,
   `state.cohesion = u64::MAX / 4`. Run 1 tick. Assert
   `sim.research_progress() ≤ MAX_RESEARCH_PER_TICK`.
   *Fails-on-stub:* unbounded accumulation.

3. **`phase_research_zero_inputs_no_op`** *(decay)*.
   Empty world (no civilians). Run 10 ticks. Assert
   `sim.research_progress() == 0`. Mirrors the existing
   `sentience_research_bonus` zero-agent guard.
   *Fails-on-stub:* non-zero from constants.

4. **`phase_research_progress_trajectory_is_power_law`** *(emergence-quality)*.
   Spawn 200 civilians with random `Position3d`, run 1000 ticks,
   sample `sim.research_progress()` every 100 ticks. Fit a log-log
   line over ticks `[200, 1000]` (skipping the burn-in). Assert
   the slope is in `[0.5, 1.2]` (sub-linear acceleration is the
   expected signature: research accelerates with population,
   knowledge, and tier but each tier has diminishing returns).
   A stub that returns `tick * constant` has slope `1.0` *only* by
   coincidence — and the constant would fail test 2.
   *Fails-on-stub:* linear, in-spec, but test 2 catches it.

5. **`phase_research_rate_varies_with_belief`** *(emergence-quality)*.
   Two sims: A with `state.belief = 1_000_000`, B with
   `state.belief = 0`, both with 200 civilians. Run 200 ticks.
   Assert `sim_A.research_progress() > sim_B.research_progress() * 1.05`
   (the belief contribution is real). Documents the upward-causation
   signature: faith contributes to discovery.
   *Fails-on-stub:* identical progress.

---

### 3.3 `phase_tech` — tier-derived unlocks (slot 14)

**Source.** PATCHPLAN §3.3. Reads `research_progress`; writes
`state.tech_unlocks` via `tech_unlocks_for_tier(tier)`.

**Tests** (`crates/engine/src/emergence_research.rs`):

1. **`phase_tech_sets_irrigation_at_tier_1`** *(smoke)*.
   Pre-populate `sim.research_cache_mut().researched.push("irrigation".into())`.
   Run 1 tick. Assert
   `(sim.tech_unlocks() & TECH_IRRIGATION) != 0`
   (const at `engine.rs:2015`).
   *Fails-on-stub:* `tech_unlocks == 0`.

2. **`phase_tech_idempotent`** *(boundary)*.
   Same as test 1 but run 100 ticks. Assert
   `sim.tech_unlocks()` is unchanged after the first tick.
   *Fails-on-stub:* monotonic accumulation.

3. **`phase_tech_no_progress_no_unlocks`** *(decay)*.
   Empty sim, run 10 ticks. Assert `sim.tech_unlocks() == 0`.
   *Fails-on-stub:* unlock from constant.

4. **`phase_tech_unlock_order_matches_tier_thresholds`** *(emergence-quality)*.
   Push 6 distinct tech names into `research_cache.researched`, run
   1 tick. Assert `sim.tech_unlocks() == TECH_IRRIGATION | TECH_STORAGE
   | TECH_METALLURGY | TECH_WRITING | TECH_SANITATION | TECH_GUNPOWDER`
   (all 6 bits set in tier order — the deterministic bitmask
   contract from `tech_unlocks_for_tier` at `engine.rs:2023-2044`).
   A stub that uses an unordered mask fails this; a stub that
   hardcodes 6 missing any one fails this; a stub that hardcodes
   all 6 but in wrong tier order passes (so the contract test is
   sharp, not a smoke test).
   *Fails-on-stub:* bitmask missing or scrambled.

---

### 3.4 `phase_belief` — faith reserve (slot 15)

**Source.** PATCHPLAN §3.4. Reads `emergence.last_sentience`,
`unrest` (stale-allowed), `population`; writes `state.belief` capped
by `MAX_BELIEF_PER_TICK = 200` + `MAX_AWAKENING_BELIEF_PER_TICK = 50`.

**Tests** (`crates/engine/src/emergence_psyche.rs`):

1. **`phase_belief_increases_on_awakening`** *(smoke)*.
   Lower `sim.emergence.sentience_threshold = SentienceThreshold::new(0.05)`.
   Run 1 tick. Assert `sim.belief() > 0` (awakenings mint belief via
   `awakening_belief_gain`, which is referenced at `emergence.rs:544`).
   *Fails-on-stub:* `belief == 0`.

2. **`phase_belief_clamps_at_cap`** *(boundary)*.
   Spam `sim.add_belief(u64::MAX / 2)` then run 1 tick.
   Assert `sim.belief() ≤ pre + MAX_BELIEF_PER_TICK` (saturating).
   *Fails-on-stub:* overflow panic or unbounded.

3. **`phase_belief_zero_awakenings_no_op`** *(decay)*.
   Empty world, run 10 ticks. Assert `sim.belief() == 0`.
   *Fails-on-stub:* belief accrues from nothing.

4. **`phase_belief_trajectory_saturates_not_explodes`** *(emergence-quality)*.
   200 civilians with `sentience_threshold = 0.05`. Run 5000 ticks.
   Sample `sim.belief()` every 100 ticks. Fit a logarithmic
   curve `belief(t) = a * log(1 + t/b)`. Assert `a < 100_000`
   (the saturation scale; the `MAX_BELIEF_PER_TICK` cap + saturating
   `add_belief` give a logarithmic curve). A linear stub fails;
   a step-function stub fails; a hardcoded constant fails tests 1
   and 3.
   *Fails-on-stub:* linear / step / constant.

5. **`phase_belief_cap_grep_guard`** *(emergence-quality, ADR-020 §4)*.
   Run `rg -n 'MAX_(BELIEF|AWAKENING)' crates/engine/src/engine.rs` from
   the test process via `std::process::Command`. Assert `≥ 2 hits`.
   This is the **cap-const grep guard** from ADR-020 §4: a phase that
   adds a new bounded writer without naming its cap is rejected on
   review; the test enforces that contract mechanically.
   *Fails-on-stub:* the cap const never gets written.

---

### 3.5 `phase_unrest` — societal unrest (slot 16)

**Source.** PATCHPLAN §3.5. Reads `food_price`, `non-food prices`,
`energy_budget`, mean `-Psyche.mood.valence`, `dispossessed_permille`
(stale-allowed); writes `state.unrest` via the seven `*_unrest` legs.

**Tests** (`crates/engine/src/emergence_psyche.rs`):

1. **`phase_unrest_rises_on_food_scarcity`** *(smoke)*.
   Set `sim.market_state` food price to `5_000` (5× baseline
   `FOOD_SCARCITY_BASELINE = 1_000` at `engine.rs:2012`). Run 1 tick.
   Assert `sim.unrest() > 0`.
   *Fails-on-stub:* `unrest == 0`.

2. **`phase_unrest_clamps_at_cap`** *(boundary)*.
   Spam the food price to `i64::MAX / 4`, energy budget to `Fixed::ZERO`,
   `dispossessed_permille = 1000`. Run 1 tick. Assert
   `sim.unrest() - pre ≤ 200` (ADR-020 §3.5 cap table "net rise is
   `O(<200)` per tick in worst case"). *Fails-on-stub:* unbounded rise.

3. **`phase_unrest_decays_when_abundant`** *(decay)*.
   Food price `≤ baseline`, energy budget positive, no misery.
   Pre-set `sim.state.unrest = 1_000`. Run 50 ticks. Assert
   `sim.unrest() < 1_000` (strict decay path).
   *Fails-on-stub:* constant or rising.

4. **`phase_unrest_dynamics_is_bounded_random_walk`** *(emergence-quality)*.
   200 civilians with random `Psyche.mood.valence ∈ [-1, 1]`, fixed
   food price at baseline, energy budget mid-range. Run 5_000 ticks,
   sample `sim.unrest()` every 50 ticks. Compute mean and standard
   deviation over the latter half. Assert:
   - `mean(unrest) < 100` (long-run bounded — the cap wins);
   - `std(unrest) < 50` (bounded fluctuation — feedback explosion
     would give `std ∝ sqrt(t)`);
   - the autocorrelation at lag-50 is `> 0.0` (memory persists but
     doesn't diverge).
   A linear stub produces `mean ∝ t`; a step stub produces
   `std = 0`; a hardcoded constant produces `mean = const` and
   `std = 0`.
   *Fails-on-stub:* linear, step, or constant.

5. **`phase_unrest_correlates_with_agent_misery`** *(emergence-quality)*.
   Two sims with identical non-ECS state but divergent
   `Psyche.mood.valence` (A: all 0.5, B: all -0.5). Run 200 ticks.
   Assert `unrest_B > unrest_A + 50` (the upward-causation leg
   `agent_misery_unrest` is real and sign-correct).
   *Fails-on-stub:* identical unrest.

---

### 3.6 `phase_cohesion` — social fabric (slot 17)

**Source.** PATCHPLAN §3.6. Reads `state.belief`, `state.unrest`,
`avg_faction_kinship(world)`, `micro_cohesion_delta(world)`; writes
`state.cohesion` with `MICRO_BIND_CAP = 12` / `MICRO_FRAY_CAP = 18`.

**Tests** (`crates/engine/src/emergence_psyche.rs`):

1. **`phase_cohesion_rises_with_belief`** *(smoke)*.
   Set `state.belief = 1_000_000`. Run 1 tick. Assert
   `state.cohesion > 0` (the `belief / COHESION_BELIEF_DIVISOR = 200`
   leg at `engine.rs:2690` fires).
   *Fails-on-stub:* cohesion stays at 0.

2. **`phase_cohesion_frays_with_unrest`** *(boundary)*.
   Set `state.unrest = 1_000_000`. Run 1 tick. Assert
   `state.cohesion == 0` or `state.cohesion` strictly decreases vs
   pre (the `unrest / COHESION_UNREST_DIVISOR = 50` fray leg wins).
   *Fails-on-stub:* cohesion rises.

3. **`phase_cohesion_micro_delta_capped`** *(decay)*.
   1_000 civilians all with `Psyche.beliefs[0] = 0.5` (perfect
   consensus). Run 1 tick. Assert `micro_cohesion_delta(&world)
   ≤ MICRO_BIND_CAP = 12` (the cap wins — perfect consensus does
   not produce +∞ cohesion).
   *Fails-on-stub:* uncapped rise.

4. **`phase_cohesion_varies_with_consensus_polynomial`** *(emergence-quality)*.
   For `n ∈ {10, 50, 200}` civilians with `Psyche.beliefs[0]` drawn
   from `Uniform(0.0, 1.0)`, run 1 tick and record
   `micro_cohesion_delta(&world)`. Across the three runs, assert
   the empirical variance of the deltas is `> 1.0` (non-constant)
   and `≤ 30²` (bounded). Documents that consensus matters but is
   saturated.
   *Fails-on-stub:* delta is constant.

5. **`phase_cohesion_upward_causation_from_kinship`** *(emergence-quality)*.
   Two sims with identical state but divergent `SocialGraph.ties`
   (A: high kinship, B: low kinship, both in same world). Run 200
   ticks. Assert `state.cohesion_A > state.cohesion_B + 100`
   (the `avg_faction_kinship` upward-causation leg is real).
   *Fails-on-stub:* identical cohesion.

---

### 3.7 `phase_social_mood` — mean mood aggregate (slot 18)

**Source.** PATCHPLAN §3.7. Reads mean `Psyche.mood.valence`,
`state.cohesion`; writes `state.society_mood` clamped to `[-1, 1]`
with per-tick step `MAX_MOOD_STEP_PER_TICK = 0.05`.

**Tests** (`crates/engine/src/emergence_psyche.rs`):

1. **`phase_social_mood_rises_on_positive_mean`** *(smoke)*.
   100 civilians with `Psyche.mood.valence = 0.5`. Run 1 tick.
   Assert `sim.society_mood() > 0.0`.
   *Fails-on-stub:* `society_mood == 0`.

2. **`phase_social_mood_clamps_at_one`** *(boundary)*.
   100 civilians with `Psyche.mood.valence = 1.0`. Run 5_000 ticks.
   Assert `sim.society_mood() ≤ 1.0` and `≥ 1.0 - 1e-6`
   (asymptote from below — the per-tick step cap prevents
   instantaneous saturation).
   *Fails-on-stub:* `society_mood > 1.0` (no clamp) or `== 1.0`
   on tick 1 (no step cap).

3. **`phase_social_mood_step_bounded`** *(decay)*.
   Pre-set `sim.state.society_mood = -1.0`. 100 civilians with
   `val = 1.0`. Run 1 tick. Assert
   `|society_mood - (-1.0)| ≤ MAX_MOOD_STEP_PER_TICK + 1e-6`.
   *Fails-on-stub:* instant jump to 1.0.

4. **`phase_social_mood_distribution_is_bounded_normal_like`** *(emergence-quality)*.
   500 civilians with `val ~ Normal(0.0, 0.3)`. Run 2_000 ticks.
   Sample `sim.society_mood()` every 50 ticks after burn-in (first
   500 ticks). Compute empirical mean and standard deviation.
   Assert `|mean| < 0.1` (the random walk is unbiased) and
   `std < 0.05` (the `MAX_MOOD_STEP_PER_TICK` cap bounds the
   fluctuation; the per-tick cap of 0.05 over 100 samples gives
   a stationary band of width ≈ `0.05 * sqrt(N_eff) ≈ 0.5` but the
   mean reverts, so `std` is bounded by the cap, not by N).
   *Fails-on-stub:* `mean != 0` (biased) or `std > 0.1` (cap missing).

5. **`phase_social_mood_anti_correlates_with_unrest`** *(emergence-quality)*.
   Two sims: A with no misery, B with high misery. Run 200 ticks.
   Assert `society_mood_A > society_mood_B` (the upward-causation
   from `Psyche.mood.valence` to `society_mood` is sign-correct and
   magnitudes are non-trivial).
   *Fails-on-stub:* identical moods.

---

### 3.8 `phase_stratification` — dispossessed share (slot 19)

**Source.** PATCHPLAN §3.8. Reads `treasury_spread`, `state.cohesion`,
`state.economic_focus`, `state.unrest`; writes `state.dispossessed_permille`
via `dispossession_step` (max 5 permille per tick internal cap,
`MAX_STRAT_STEP_PER_TICK = 50` outer cap).

**Tests** (`crates/engine/src/emergence_macro.rs`):

1. **`phase_stratification_rises_on_inequality`** *(smoke)*.
   Set `state.faction_treasury = {0: 10_000, 1: 0}` (spread = 10_000,
   so `from_inequality = 50` at `engine.rs:2478`). Run 50 ticks.
   Assert `sim.dispossessed_permille() > 0`.
   *Fails-on-stub:* `dispossessed_permille == 0`.

2. **`phase_stratification_decays_with_cohesion`** *(boundary)*.
   Pre-set `state.cohesion = 1_000_000` (cohesion erosion term is
   `cohesion / 5_000 = 200`, more than cancels the inequality term).
   Run 100 ticks from `dispossessed_permille = 1000`. Assert
   `sim.dispossessed_permille() < 1000`.
   *Fails-on-stub:* monotonically increasing.

3. **`phase_stratification_clamps_at_1000`** *(decay)*.
   Spam `treasury_spread = i64::MAX`. Run 5_000 ticks. Assert
   `sim.dispossessed_permille() ≤ 1000`.
   *Fails-on-stub:* overflow or no clamp.

4. **`phase_stratification_step_is_linear_then_saturates`** *(emergence-quality)*.
   With high spread and zero cohesion, run 100 ticks. Assert the
   per-tick step is exactly `MAX_STEP = 5` (the inner cap from
   `dispossession_step` at `engine.rs:2507`) until saturation
   at `1000`. The trajectory is **piecewise linear → plateau**;
   a hardcoded constant fails; a quadratic stub fails; the
   specific `5 permille/tick` slope is verified by
   `dispossessed_permille(t) ≈ min(5*t, 1000)`.
   *Fails-on-stub:* any non-linear path to 1000.

5. **`phase_stratification_emerges_power_law_over_faction_wealth`** *(emergence-quality)*.
   Spawn 8 factions with treasury drawn from a Zipf distribution
   (`t_i = 1_000_000 / i`). Run 200 ticks. Rank factions by
   `faction_treasury`, fit `PowerLawFit` over the rank-frequency
   distribution. Assert `1.0 < alpha < 2.0` (the emergent wealth
   rank-frequency exhibits a Zipf-like tail). Documents that
   inequality emerges with power-law shape, not as a uniform
   gradient.
   *Fails-on-stub:* alpha = 0 (sentinel).

---

### 3.9 `phase_institutions` — temple + garrison (slot 20)

**Source.** PATCHPLAN §3.9. Reads `belief`, `unrest`,
`dispossessed_permille`, `population`; writes `state.temple_level`,
`state.garrison_level` capped at `MAX_INSTITUTION_LEVEL = 5` with
`MAX_INSTITUTION_RISE_PER_TICK = 1`.

**Tests** (`crates/engine/src/emergence_macro.rs`):

1. **`phase_institutions_temple_rises_with_belief`** *(smoke)*.
   Set `state.belief = 1_000_000` (target = `1_000_000 / 200_000 = 5`
   at `institution_target_level(belief, per_level=200_000)`). Run
   10 ticks. Assert `sim.temple_level() >= 5` (saturated in 5 ticks
   at the per-tick rise of 1).
   *Fails-on-stub:* `temple_level == 0`.

2. **`phase_institutions_garrison_rises_with_unrest`** *(boundary)*.
   Set `state.unrest = 1_000_000`, `state.dispossessed_permille = 0`.
   Run 10 ticks. Assert `sim.garrison_level() >= 5`.
   *Fails-on-stub:* `garrison_level == 0`.

3. **`phase_institutions_step_bounded`** *(decay)*.
   Both drivers maxed. Run 100 ticks. Assert
   `sim.temple_level() ≤ MAX_INSTITUTION_LEVEL = 5` and
   `sim.garrison_level() ≤ 5`.
   *Fails-on-stub:* overflow.

4. **`phase_institutions_temple_garrison_anticorrelated_in_history`** *(emergence-quality)*.
   Two sims, A with high belief / low unrest (temple > garrison),
   B with low belief / high unrest (garrison > temple). Run 200
   ticks. For each, compute the **trajectory correlation**
   `corr(temple_t, garrison_t)`. Assert `corr_A < 0` and
   `corr_B < 0` (temple and garrison are **anti-correlated** in
   any single history — a society doesn't simultaneously build
   temples and garrisons from the same signals; this is the
   emergence signature).
   *Fails-on-stub:* both rise together (no signal differentiation).

---

### 3.10 `phase_economic_focus` — 5-way label (slot 21)

**Source.** PATCHPLAN §3.10. Reads `food`, `research_tier`, `belief`,
`treasury_total`; writes `state.economic_focus: EconomicFocus` via
`candidate_economic_focus` (`engine.rs:2341-2364`).

**Tests** (`crates/engine/src/emergence_macro.rs`):

1. **`phase_economic_focus_picks_agrarian_on_food`** *(smoke)*.
   Set `food = 1_000_000`, `research_tier = 0`, `belief = 0`,
   `treasury_total = 0`. Run 1 tick. Assert
   `sim.economic_focus() == EconomicFocus::Agrarian`
   (the `agr = food` leg wins).
   *Fails-on-stub:* any other label.

2. **`phase_economic_focus_picks_industrial_on_tier`** *(boundary)*.
   `food = 0`, `research_tier = 5`, `belief = 0`, `treasury_total = 0`.
   Run 1 tick. Assert
   `sim.economic_focus() == EconomicFocus::Industrial`
   (the `ind = tier * 50_000 = 250_000` leg wins).
   *Fails-on-stub:* any other label.

3. **`phase_economic_focus_picks_sacred_on_belief`** *(decay)*.
   `food = 0`, `tier = 0`, `belief = 1_000_000`, `treasury_total = 0`.
   Run 1 tick. Assert
   `sim.economic_focus() == EconomicFocus::Sacred`
   (the `sac = belief / 4 = 250_000` leg wins).
   *Fails-on-stub:* any other label.

4. **`phase_economic_focus_label_distribution_is_diversified`** *(emergence-quality)*.
   20 sims with seeds `1..=20`, run 500 ticks each, record final
   `sim.economic_focus()`. Build the label histogram. Compute
   Shannon entropy over the 5 labels. Assert
   `entropy > 0.5` bits (diversified; a stub that always returns
   `Balanced` gives `entropy = 0`).
   *Fails-on-stub:* single-label collapse.

5. **`phase_economic_focus_two_pass_settles_within_two_ticks`** *(emergence-quality)*.
   Pre-set `state.economic_focus = Balanced`. Set the four
   candidate inputs so the result must be `Industrial`. Run 2
   ticks. Assert `sim.economic_focus() == Industrial` after tick 2
   (the two-pass implementation settles in 2 ticks; a single-pass
   stub with stale inputs settles in 1 but is wrong on tick 2 —
   this test fires against that stub by checking the **settled**
   value at tick 2, not the tick-1 transient).
   *Fails-on-stub:* settles to wrong label.

---

### 3.11 `phase_emergence` — orchestrator (slot 22)

**Source.** PATCHPLAN §3.11. Runs the seven sub-phases
(`emergence_ensure_genomes`, `emergence_culture`, `emergence_social`,
`emergence_psyche`, `emergence_genetics_sentience`, `emergence_legends`,
`emergence_civ_ai`) and calls `apply_awakening_coupling`.

**Tests** (`crates/engine/src/emergence.rs` — extend existing
`mod tests`):

1. **`phase_emergence_invoked_by_tick`** *(smoke)*.
   `Simulation::with_seed(42).tick()` then assert
   `!sim.emergence_feed().is_empty() ||
    sim.legends_graph().node_count() > 0` (the orchestrator
   produces observable side-effects in one tick).
   *Fails-on-stub:* orchestrator never called.

2. **`phase_emergence_awakening_coupling_within_cap`** *(boundary)*.
   Set `sim.emergence.sentience_threshold = SentienceThreshold::new(0.05)`.
   Run 1 tick. Assert
   `sim.belief() ≤ MAX_AWAKENING_BELIEF_PER_TICK + sim.belief_pre` and
   `sim.cohesion_increment ≤ MAX_AWAKENING_COHESION_PER_TICK = 10`
   (the awakening pulse is bounded — feedback-explosion guard).
   *Fails-on-stub:* unbounded rise.

3. **`phase_emergence_no_op_when_world_empty`** *(decay)*.
   Empty world. Run 1 tick. Assert `sim.emergence_feed().is_empty()`
   *and* `sim.last_sentience().is_empty()` (no false awakenings
   from an empty Dna pool).
   *Fails-on-stub:* false-positive sentience.

4. **`phase_emergence_saga_graph_grows_sublinearly`** *(emergence-quality)*.
   100 civilians, run 1_000 ticks. Sample `sim.legends_graph().node_count()`
   every 100 ticks. Fit a log-log line over ticks `[200, 1000]`.
   Assert slope `∈ [0.5, 1.5]` (the saga graph accumulates but
   doesn't explode; slope `= 1` means linear; slope `> 1.5` means
   super-linear / runaway; slope `< 0.5` means stagnant).
   A no-op stub produces slope `= 0`; a runaway stub produces
   slope `> 2`.
   *Fails-on-stub:* slope 0 or > 2.

5. **`phase_emergence_cluster_culture_entropy_rises_then_stabilises`** *(emergence-quality)*.
   200 civilians in 4 hand-placed clusters, run 1_000 ticks.
   Sample `sim.cluster_cultures()` every 100 ticks, compute
   Shannon entropy over cluster ids weighted by `traits[0]`.
   Assert:
   - entropy at tick 200 `> entropy at tick 0` (cultures diverge);
   - entropy at tick 1000 `< entropy at tick 200 * 1.5` (cultures
     stabilise, don't diverge forever — the `drift_populations`
     retention of `0.85` at `emergence.rs:234` gives bounded drift).
   A no-op stub gives constant entropy (fail test 1); a runaway
   stub gives monotonically increasing entropy (fail test 2).
   *Fails-on-stub:* constant or monotonically increasing.

6. **`phase_emergence_social_graph_degree_distribution_is_heavy_tailed`** *(emergence-quality)*.
   200 civilians with default `cluster_by_colocation` membership,
   run 500 ticks. Walk each `SocialGraph.ties`, build a degree
   histogram (ties per agent). Fit `PowerLawFit`. Assert
   `1.5 < alpha < 3.5` and `r_squared > 0.6`. A no-op stub gives
   `alpha = 0` sentinel; a uniform stub gives `alpha → ∞`.
   *Fails-on-stub:* 0 or ∞.

---

## 4. Cross-cutting tests (integration)

**File:** `crates/engine/tests/emergence_wiring.rs` *(new)*.

7. **`emergence_wiring_determinism_same_seed`** — Two `Simulation::with_seed(N)`
   for `N ∈ {1, 7, 42}`, run 1_000 ticks. Compare full state via
   `sim.snapshot()` and `sim.last_emergence_sample()`. Assert byte-equality
   across all fields. Detects RNG divergence, ordering changes, and
   non-deterministic HashMap iteration in any of the 11 phases.
   *Fails-on-stub:* per-tick RNG use is non-deterministic, or any
   HashMap→Vec iteration breaks determinism.

8. **`emergence_wiring_perf_budget`** — 1_000 civilians, run 200 ticks,
   measure mean + p99 wall-clock per tick via
   `std::time::Instant`. Assert `p99 ≤ 4.0 ms` (ADR-010 budget).
   *Fails-on-stub:* per-tick cost > 4 ms (catches the
   "scan the world 11 times" implementation mistake).

9. **`emergence_wiring_cap_grep_guard`** — Run
   `rg -n 'MAX_(AWAKENING|MISERY|COHESION|RESEARCH|INSTITUTION|MOOD|STRAT|BELIEF|FOCUS)'`
   from the test process. Assert `≥ 8 hits` (per ADR-020 §4).
   *Fails-on-stub:* a phase was added without naming its cap.

---

## 5. Test-by-test count summary

| Phase / cross-cutting | Smoke | Boundary | Decay | Emergence | **Total** |
|------------------------|-------|----------|-------|-----------|-----------|
| 3.1 `phase_life` | 1 | 1 | 1 | 2 | **5** |
| 3.2 `phase_research` | 1 | 1 | 1 | 2 | **5** |
| 3.3 `phase_tech` | 1 | 1 | 1 | 1 | **4** |
| 3.4 `phase_belief` | 1 | 1 | 1 | 2 | **5** |
| 3.5 `phase_unrest` | 1 | 1 | 1 | 2 | **5** |
| 3.6 `phase_cohesion` | 1 | 1 | 1 | 2 | **5** |
| 3.7 `phase_social_mood` | 1 | 1 | 1 | 2 | **5** |
| 3.8 `phase_stratification` | 1 | 1 | 1 | 2 | **5** |
| 3.9 `phase_institutions` | 1 | 1 | 1 | 1 | **4** |
| 3.10 `phase_economic_focus` | 1 | 1 | 1 | 2 | **5** |
| 3.11 `phase_emergence` | 1 | 1 | 1 | 3 | **6** |
| 4.x cross-cutting | 1 | 1 | 1 | — | **3** |
| **TOTAL** | **12** | **12** | **12** | **21** | **57** |

Per the charter + ADR-011 (3-test minimum per phase), the 11 phases
already have **33 tests** at smoke/boundary/decay depth. This plan
adds **21 emergence-quality tests** and **3 cross-cutting tests**
(57 total) so that:
- A pass-through stub (`fn phase_X(&mut self) {}`) fails at least one
  smoke test **and** at least one emergence-quality test.
- A hardcoded-constant stub fails the emergence-quality tests.
- A feedback-explosion stub fails the cap / determinism tests.
- A non-deterministic stub fails the determinism test.

The 21 emergence-quality tests cover all four emergence property
families (power-law, entropy, structure-count, boundedness) and all
five emergence-quality crates in `civ-emergence-metrics`
(`power_law`, `shannon`, `structure`, `branching`, `novelty`).

---

## 6. CI integration

Once the wiring PR lands, `just civis-3d-verify` runs
`agent-smoke` + `check` + `test` + `clippy` + `fmt`; the new tests
ride the existing `cargo test -p civ-engine` invocation. The
cross-cutting integration tests in `crates/engine/tests/emergence_wiring.rs`
take ~3 s on the regression corpus (1_000 ticks × 3 seeds for
determinism, 200 ticks for perf); budget accounted for in the
existing `tick_budget.rs` envelope.

A new just recipe:

```just
# Emergence-quality gate — re-runs the 21 emergence tests + the
# 3 cross-cutting tests in isolation so a CI failure surfaces
# "emergence is broken" rather than the standard "test failed".
civis-3d-emergence-tests:
    cargo test -p civ-engine --lib emergence::
    cargo test -p civ-engine --test emergence_wiring
```

This recipe is the per-PR review gate; it runs in < 30 s and
replaces no existing gate.

---

## 7. Cross-references

- ADR-020 — the wiring decision.
- EMERGENCE_WIRING_PATCHPLAN — the per-phase mechanical recipe.
- ADR-011 — N-Series Emergence Coupling (3-test minimum, named cap,
  shared gradient).
- ADR-018 — Emergence Systems Bidirectional Coupling.
- ADR-010 — CA tick-budget guard (4 ms cap; warning path).
- ADR-003 / ADR-determinism-dropped — replay determinism.
- `crates/civ-emergence-metrics/` — the `PowerLawFit`, `ShannonEntropy`,
  `StructureCount`, `BranchingLedger` crates that power the
  emergence-quality tests.
- `docs/design/emergence-dashboard.md` — the dashboard tile contract
  the integration tests verify.
- `docs/reports/EMERGENCE_AUDIT.md` — the audit gap #1 / #6 this
  test plan closes (read-side coverage of the macro-web emergence
  layers).

---

## 8. Out-of-scope (deferred to follow-up PRs)

- **`phase_chronicle`** — chronicle dedupe / golden-age lines (deferred
  per EMERGENCE_WIRING_PATCHPLAN §5). Tests land in the chronicle
  PR.
- **`phase_disasters`** — disaster-driven unrest & belief (ADR-020
  §"Consequences" — "if `phase_disasters` is wired later"). Tests
  land in the disasters PR.
- **`phase_religion`** — `religion::spread_religion` hook (currently a
  TODO marker; the `MAX_BELIEF_PER_TICK` cap absorbs the zero input).
  Tests land in the religion PR.
- **Mod-host emergence tests** — the `mod-host` tests at
  `crates/mod-host/tests/` cover WASM ticking but not emergence
  coupling from a mod. Deferred to the mod-capability-enforcement
  PR (out of scope per the `AGENTS.md` "Do not implement full
  CIV-0700" line).

---

## 9. Open questions for the implementer

1. **Where does `phase_life`'s `last_life_deaths` field live?**
   `engine.rs:418` declares `last_life_deaths: u32` already but it's
   never read or written. The test design assumes it counts
   settlement-famine deaths; if the implementer wants a different
   meaning, tests 1 and 5 need to be re-scoped.
2. **Two-pass `phase_economic_focus` semantics.** The patch plan
   describes a settle pass after `phase_institutions`. Test 5 in §3.10
   asserts settle-by-tick-2; if the implementer keeps the
   pre-pass and settle-pass on the *same* tick (no lag), the test
   should be tightened to tick-1. Adjust as implemented.
3. **Awakening coupling magnitude.** The test in §3.11 test 2
   assumes `MAX_AWAKENING_BELIEF_PER_TICK = 50` (per
   `emergence.rs:537` docstring) and
   `MAX_AWAKENING_COHESION_PER_TICK = 10` (per `emergence.rs:2713`).
   If the implementer changes these caps, the test budget
   constants must update alongside.
4. **Determinism test runtime.** The 1_000-tick, 3-seed determinism
   test takes ~5 s on the regression corpus. Acceptable for
   `cargo test --release` but slow for debug. Consider
   `#[ignore]` on the determinism test and gate it on
   `civis-3d-emergence-tests` only.
5. **The `phase_emergence` orchestrator test 4 (saga graph sub-linear).**
   The legend/saga ingestion rate depends on the **death rate**
   (which depends on `phase_citizen_lifecycle` and food production).
   If the determinism seed produces a stable death rate, the slope
   is well-defined; if the death rate is noisy, the test should
   use **Spearman correlation** rather than log-log slope. The
   implementer should run the test against their wiring PR and
   adjust the assertion if the slope is too noisy for a
   least-squares fit.

---

## 10. Mechanical recipe for the implementer

1. **Land the wiring PR** per EMERGENCE_WIRING_PATCHPLAN §1-9
   (the 23-entry `PHASE_ORDER`, the 11 `WorldState` fields, the
   11 `phase_*` method stubs, the 12 new call sites in
   `Simulation::tick`, the 11 accessors, the 8 `ReplayLog::record_*`
   methods, the 3 new consts).
2. **Build is GREEN.** All existing tests pass.
3. **Land the 57 tests in §3 + §4** as a single follow-up PR (or
   per-phase group if the reviewer prefers). Each test file uses
   the existing `mod tests` convention (unit) or
   `tests/<name>.rs` (integration).
4. **Run `just civis-3d-emergence-tests`** — must be GREEN.
5. **Run `just civis-3d-verify`** — must be GREEN (existing gates
   still pass; FR matrix still resolves).
6. **Update `docs/traceability/fr-3d-matrix.md`** to map each new
   test to the FR-3D row it covers (audit §6 closure).
7. **Move ADR-020 to Accepted** once §6 is GREEN.