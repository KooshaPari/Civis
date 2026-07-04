# CIV-007 Emergent Diplomacy — Design Specification

**Status:** DESIGN  
**Branch:** wip/civ007-design  
**Depends on:** CIV-005 (factions/alignment), CIV-006 (tactics/casualties), CIV-002 (joule economy, tiered allocation)  
**Engine scaffold:** `phase_diplomacy` in `crates/engine/src/engine.rs`; `DiplomacyMatrix` + `DiplomacySignal` + `RelationRecord` in `crates/agents/src/diplomacy.rs`

---

## 0. Charter Constraint

The Civis Emergence Charter forbids hardcoding diplomacy as a rule table or finite-state machine. This spec makes inter-faction relations a **continuous field shaped by gradients and lags**, not a scripted transition graph. The only hardcoded floor is the physics of resource scarcity — itself a physical/environmental law the Charter permits — and the mathematical form of the drift equation. What emerges (who allies, who wars, how long peace lasts, whether trade is the dominant mode) is entirely a consequence of initial conditions and the live gradient landscape.

---

## 1. Emergence Model

### 1.1 Core Abstraction

Each ordered pair of factions `(A, B)` holds a single continuous **relation score** `r ∈ [-1.0, 1.0]`, stored in `RelationRecord.score` inside `DiplomacyMatrix`. This score is **not set** by any rule; it drifts each tick under a weighted sum of micro-driver signals. The qualitative state (Alliance / Trade / Neutral / Rivalry / War, per `RelationKind`) is merely a read-off threshold on the continuous score, not a primary variable.

### 1.2 Micro-Driver Signals

All six signals below map directly to `DiplomacySignal` fields (existing or extended):

| Signal | Field | Direction | Physical grounding |
|---|---|---|---|
| Resource competition | `resource_competition: f32` | negative | Two factions drawing on the same scarce gradient (food, metal, energy) push the score down. Derived from `faction_resources` overlap and `cluster_stocks` shortfalls. |
| Trade volume | `trade_volume: f32` | positive | Measured from `WorldState::trade_routes` — sum of `TradeRoute.volume` crossing the faction boundary. Active trade raises the score. |
| Border friction | `proximity: f32` | mildly negative | Normalized spatial overlap of civilian populations (from `Position3d` coordinates and `ClusterMember.cluster` assignments). Close neighbors accumulate low-level irritation without conflict. |
| Accumulated combat grievance | `combat_grievance: f32` (new field) | strongly negative | Derived from `last_tick_engagements`: per-tick accumulation of casualties where `shooter_faction != target_faction`. Decays with a configurable half-life. |
| Need-overlap benefit | `need_complementarity: f32` (new field) | positive | Measures how much faction A's surplus good (high stock in `faction_resources`) matches faction B's deficit (low stock). High complementarity implies latent trade benefit, pushing toward cooperation. |
| Energy scarcity pressure | `scarcity_pressure: f32` (new field) | negative when both scarce, positive when one can supply | Derived from `economy_state.energy_budget_joules` per faction relative to `tiered_demand`. When both factions are energy-scarce, competition sharpens. When one is surplus, resource-dependency creates asymmetric pull toward cooperation. |

### 1.3 Drift Equation

The `apply_signal` call in `DiplomacyMatrix` already implements a weighted sum:

```
drift = trade_volume    × W_trade
      - resource_competition × W_compete
      - proximity        × W_border
      - combat_grievance × W_grievance
      + need_complementarity × W_complement
      - scarcity_pressure × W_scarcity

r(t+1) = clamp(r(t) + drift, -1.0, 1.0)
```

Current weights in `apply_signal`: `trade=0.08, competition=0.12, proximity=0.04`.  
Spec adds: `grievance=0.18, complement=0.06, scarcity=0.10`.  
(These are criticality knobs — see Section 3.)

### 1.4 Why This Produces Macro Diplomacy Without a Script

- **Alliance formation** emerges when two factions have sustained positive trade volume AND high need-complementarity AND no combat history — the score accumulates past the `0.60` threshold.
- **War** emerges when resource competition is high AND combat grievances have accumulated AND scarcity pressure is mutual — score falls below `-0.60`.
- **Fragile peace** emerges when trade and combat grievance roughly cancel — the score hovers in the `[-0.20, 0.20]` Neutral band, drifting with shocks.
- **Alliance breakdown** emerges naturally when trade routes are severed (famine, energy embargo) and the trade term drops — the score drifts down without any "defect" rule.

No transition triggers, no FSM states, no scripted casus belli.

---

## 2. Coupling

### 2.1 Economy → Diplomacy (downward causation)

**Trade volume signal.** `phase_diplomacy` reads `WorldState::trade_routes` and sums `TradeRoute.volume` per faction pair. This is already populated by `tick_trade_routes`. When a route's `from_faction` has insufficient stock (`available <= Fixed::ZERO`), the volume contribution drops to zero — an economic event (resource depletion) directly damps the positive trade signal.

**Scarcity pressure signal.** `economy_state.energy_budget_joules` (synced each tick in `phase_economy`) provides the per-faction energy level. The ratio of current budget to `tiered_demand` for the Subsistence tier — already computed in `tiered_demand()` via live `LifeNeeds` pressure — gives a scarcity index. When both factions are subsistence-stressed (Subsistence tier fill fraction < 0.5), scarcity pressure is set high; when one can export surplus, the asymmetry produces a complementarity pull.

**Embargo feedback.** When `r < -0.60` (War), the implementer should suppress the corresponding `TradeRoute.volume` to zero — this closes the feedback loop: war prevents trade, which removes the positive signal, making recovery slower.

### 2.2 Tactics → Diplomacy → Tactics (grievance feedback loop)

`last_tick_engagements: Vec<CombatEngagement>` (field on `Simulation`) is populated each `phase_tactics` tick from the war bridge. Each `CombatEngagement` carries `shooter_faction` and `target_faction`. The grievance accumulator in `phase_diplomacy` iterates this slice, adds a weight-per-engagement to the running `combat_grievance` for the `(shooter_faction, target_faction)` pair, then applies exponential decay:

```
grievance(t+1) = grievance(t) × (1 - decay_rate) + engagements_this_tick × engagement_weight
```

This grievance feeds into the `apply_signal` call, pushing `r` down. As `r` falls further into War territory, the chance of new engagements rises (factions with War relation are militarily active), producing the positive feedback loop:

> **war → casualties → grievance ↑ → r falls → more war**

The loop has a natural brake: high casualties deplete `MilitaryUnit` entities and reduce `state.population`, eventually reducing the engagement rate. This is the war-exhaustion mechanism — it is not hardcoded, it emerges from the carrying capacity of the population.

### 2.3 Emergent Factions / Alignment → Diplomacy

Factions in the engine are already emergent: `faction_alignment()` reads live `Alignment::Faction` values from `AgentCivilian` components. Civilians with `Alignment::Faction(id)` contribute to that faction's spatial footprint and its cluster memberships (`ClusterMember.cluster`). The `proximity` signal in `DiplomacySignal` is computed from the spatial overlap of per-faction cluster populations: when two faction populations share settlement clusters (as determined by `cluster_by_colocation`), proximity is high.

Because alignment is emergent (civilians drift alignment based on needs, culture, and psyche — task #108), the faction membership of clusters can shift over time, causing the proximity and resource_competition signals to evolve without explicit programming.

### 2.4 Diplomacy → Economy (upward causation)

When `r >= 0.20` (Trade or Alliance), a `DiplomacyKind::TradeAgreement` event is emitted and the existing treasury boost in `phase_diplomacy` is the placeholder for a proper mechanism: the implementer should instead ensure that the faction pair's `TradeRoute.volume` is permitted to be non-zero (unblocked) and optionally scaled by `r`. When `r < -0.20` (Rivalry or War), trade routes between these factions are suppressed (volume zeroed), starving both factions of complementary goods — a feedback that worsens their economic state, increasing scarcity pressure, which then feeds back into the relation score.

---

## 3. Criticality Knobs

The following scalar parameters govern where the system sits on the order-chaos axis. Values outside the suggested ranges tend toward degenerate attractors.

| Knob | Field location | Suggested range | Heat-death risk (too low) | Explosion risk (too high) |
|---|---|---|---|---|
| `W_grievance` | drift equation weight | 0.10 – 0.25 | Wars never start; factions stuck in Neutral | Cascade into total war from any skirmish |
| `W_trade` | drift equation weight | 0.05 – 0.12 | No economic incentive to cooperate | Single trade route locks all factions into Alliance permanently |
| grievance decay rate | new field `GriefParams.decay` | 0.005 – 0.03 per tick | Old wounds never heal; permanent war | Grievances reset too fast; no memory of conflict |
| `W_compete` | drift equation weight | 0.08 – 0.18 | Sharing resources has no diplomatic cost | Peaceful coexistence impossible under any scarcity |
| `W_scarcity` | drift equation weight | 0.05 – 0.15 | Energy crises have no diplomatic effect | Energy shock instantly causes all wars simultaneously |
| `W_complement` | drift equation weight | 0.03 – 0.10 | No incentive for specialization-based alliance | All factions converge to identical specialization preventing divergence |
| Alliance threshold | `DiplomacyMatrix::relation_kind` cutoff at `0.60` | 0.50 – 0.75 | Too easy to ally; permanent-peace attractor | Too hard; no alliances form; war/neutral dominant |
| War threshold | `DiplomacyMatrix::relation_kind` cutoff at `-0.60` | -0.75 – -0.45 | Too easy to fall into war | Too hard; total war only under extreme conditions |

**Criticality target:** the system should exhibit a power-law war-size distribution (many small skirmishes, few large wars), not a bimodal "all at peace or all at war" distribution. See Section 4 for the measurement.

---

## 4. Observable Metrics

All of these should be computed by `phase_diplomacy` and exposed on `Simulation` (as accessor methods reading from the `DiplomacyMatrix` snapshot):

### 4.1 Alliance-Network Structure

Treat factions with `RelationKind::Alliance` or `RelationKind::Trade` as edges in an undirected graph. Track:

- **Largest connected component size** — approaches `N_factions` in heat-death, stays small in healthy rivalry.
- **Clustering coefficient** — high = tight alliance blocs; near zero = only bilateral pairings.
- **Modularity** — detects whether the graph has split into opposing blocs (precursor to bloc war).

### 4.2 War Frequency Distribution

Maintain a histogram of how many consecutive ticks a pair spends in `RelationKind::War` per episode. A healthy distribution is approximately power-law (slope ~1.5–2.5). A bimodal distribution (many micro-wars + a few permanent wars, or no micro-wars at all) indicates a criticality failure.

Derived from the sequence of `DiplomacyOutcome` results returned by `apply_signal`: count transitions into and out of War per pair, and measure episode durations.

### 4.3 Trust-Matrix Entropy

The `DiplomacyMatrix` stores `RelationRecord.score` for each pair. The distribution of scores across all pairs is a probability-like measure when normalized. Compute Shannon entropy over the 5 `RelationKind` buckets:

```
H = -sum_k [ p_k * log2(p_k) ]   where p_k = fraction of pairs in RelationKind k
```

- `H ≈ 0`: all pairs in same state (all-peace or all-war — degenerate attractors)
- `H ≈ log2(5) ≈ 2.32`: maximum diversity; all states equally populated
- **Target operating range:** `H ∈ [1.5, 2.1]`

### 4.4 Grievance Half-Life

Track the per-pair `combat_grievance` over time. Measure the empirical half-life of grievance decay. If the half-life is shorter than the average war episode duration, wars self-terminate naturally. If it is longer, wars require external shocks to end (supply exhaustion, etc.).

---

## 5. Phased Implementation Plan

### Phase A — Signal Plumbing (no new structs, extend existing)

**Depends on:** existing `phase_diplomacy` stub, `last_tick_engagements`, `cluster_stocks`, `faction_resources`

| Task | What to touch | Notes |
|---|---|---|
| A1: Extend `DiplomacySignal` | `crates/agents/src/diplomacy.rs` | Add `combat_grievance: f32`, `need_complementarity: f32`, `scarcity_pressure: f32` fields |
| A2: Compute combat grievance | `phase_diplomacy` in `engine.rs` | Iterate `self.last_tick_engagements`, group by `(shooter_faction, target_faction)`, apply decay formula |
| A3: Compute need complementarity | `phase_diplomacy` | For each faction pair, compare surplus goods in `faction_resources` (A high food + B low food = positive complementarity) |
| A4: Compute scarcity pressure | `phase_diplomacy` | Use `economy_state.energy_budget_joules` per faction vs Subsistence tier demand from `tiered_demand()` |
| A5: Compute proximity signal | `phase_diplomacy` | Iterate ECS for `ClusterMember` + `AgentCivilian.alignment` to count faction co-location |

Phase A deliverable: `phase_diplomacy` populates a complete `DiplomacySignal` for every faction pair and calls `DiplomacyMatrix::apply_signal`. The matrix is stored on `Simulation` (new field: `diplomacy_matrix: DiplomacyMatrix`).

### Phase B — Grievance Memory and Trade Suppression

**Depends on:** Phase A

| Task | What to touch |
|---|---|
| B1: Add grievance accumulator | New struct `GriefAccumulator { pairs: BTreeMap<(u32,u32), f32> }` on `Simulation` or inside `DiplomacyMatrix` |
| B2: Trade route suppression | Modify `tick_trade_routes` to skip routes where `diplomacy_matrix.relation(a,b) == War` |
| B3: Treasury replacement | Replace the fixed `+100`/`-50` treasury adjustments in `phase_diplomacy` with `trade_route` volume effects — the economy coupling is now through routes, not direct treasury magic |

### Phase C — Faction→Economy Feedback

**Depends on:** Phase B

| Task | What to touch |
|---|---|
| C1: Alliance trade bonus | When `r >= 0.60`, boost `TradeRoute.volume` for existing routes between those factions by an alliance multiplier |
| C2: War penalty on EconomyState | When `r <= -0.60`, reduce `energy_budget_joules` allocation to the war-fighting faction by the defense-spend multiplier pattern from CIV-0105 spec |
| C3: Scarcity cascade test | Write a test: reduce `faction_resources.food` on faction A to zero, verify `scarcity_pressure` rises, verify `r(A,B)` drifts negative within N ticks |

### Phase D — Emit DiplomacyEvents and Wire Metrics

**Depends on:** Phase B, Phase C

| Task | What to touch |
|---|---|
| D1: Emit on `RelationKind` transitions | When `DiplomacyOutcome.before != after`, push a `DiplomacyEvent` with the appropriate `DiplomacyKind` (map: `War→Conflict`, `Alliance/Trade→TradeAgreement`, others→Peace as placeholder) |
| D2: Implement alliance-network metric | New method `Simulation::diplomacy_alliance_graph_stats()` returning component sizes + H |
| D3: Implement trust entropy | New method `Simulation::diplomacy_trust_entropy() -> f32` |
| D4: Expose grievance half-life | Track in `GriefAccumulator`, expose via accessor |

### Phase E — Test Strategy

**No implementation code; equips the engineer writing tests.**

All tests live in `crates/engine/tests/` and `crates/agents/src/diplomacy.rs` (existing test module).

| Test | What to assert |
|---|---|
| `diplomacy_war_emerges_from_competition` | Seed two factions with identical resource demand for a scarce shared good; assert `r < -0.60` within 50 ticks without any explicit combat |
| `diplomacy_trade_drives_alliance` | Seed two factions with complementary goods and high trade volume; assert `r > 0.60` within 30 ticks |
| `grievance_loop_amplifies_war` | Inject synthetic `CombatEngagement` events; assert score falls faster than without engagements; assert score recovers after engagements stop (decay) |
| `alliance_collapses_on_trade_disruption` | Bring two factions to Alliance; zero out their `TradeRoute.volume`; assert `r` drifts toward Neutral within 100 ticks |
| `trust_entropy_in_healthy_range` | Run simulation for 500 ticks with diverse resource distributions; assert `H ∈ [1.0, 2.3]` |
| `war_suppresses_trade_routes` | Bring two factions to War; assert their `TradeRoute.volume` has been zeroed |
| `determinism` | Run two simulations with same seed; assert `DiplomacyMatrix` snapshots are identical after N ticks |
| `score_never_escapes_clamped_range` | Proptest: arbitrary signal sequences; assert `r ∈ [-1.0, 1.0]` throughout |

---

## 6. What the Implementer Must NOT Do

- Do not add a `DiplomaticState` enum or FSM transition table to the engine. `RelationKind` (already in `diplomacy.rs`) is a read-only projection, not a primary variable.
- Do not hardcode "faction A declares war on faction B at tick T" anywhere.
- Do not consult `WorldState::factions` HashMap directly to decide who allies; only the gradient signals and the continuous score should determine outcomes.
- Do not skip the decay term on `combat_grievance`. Without decay, the first war permanently prevents future alliances (heat-death of diplomacy).
- Do not run `phase_diplomacy` on the existing `tick % 500 == 0` cadence for the gradient updates — that was a placeholder. Gradient updates should run every tick (or every N ticks with N ≤ 10) to maintain continuous dynamics. The existing 500-tick cadence was a stub that emits a random coin-flip, not an emergence mechanism.

---

## 7. Cross-Reference to Existing Scaffolding

| Existing item | Role in this spec |
|---|---|
| `DiplomacyMatrix` (`crates/agents/src/diplomacy.rs`) | Primary relation store; already implements `apply_signal`, `RelationKind` read-off, symmetric key |
| `DiplomacySignal` | Signal struct to extend with grievance, complementarity, scarcity fields |
| `RelationRecord.score` | The continuous relation variable; already clamped to `[-1.0, 1.0]` |
| `RelationKind::{Alliance, Trade, Neutral, Rivalry, War}` | Macro-diplomacy labels derived from score; already defined |
| `DiplomacyOutcome.{before, after}` | Transition detection for event emission in Phase D |
| `last_tick_engagements: Vec<CombatEngagement>` on `Simulation` | Grievance input from tactics |
| `CombatEngagement.{shooter_faction, target_faction}` | Faction attribution for grievance |
| `WorldState::trade_routes: Vec<TradeRoute>` | Trade volume signal source |
| `TradeRoute.{from_faction, to_faction, volume}` | Trade signal computation |
| `WorldState::faction_resources: HashMap<u32, Resources>` | Need-complementarity signal |
| `Resources.{food, wood, metal, energy}` | Per-good surplus/deficit |
| `economy_state.energy_budget_joules` | Scarcity pressure signal |
| `tiered_demand()` method on `Simulation` | Subsistence demand baseline for scarcity ratio |
| `ClusterMember.cluster`, `AgentCivilian.alignment` (ECS) | Proximity signal via faction co-location |
| `DiplomacyEvent` + `DiplomacyKind` | Event bus output (extend from current stub) |
| `phase_diplomacy` stub in `engine.rs` (lines 1908-1948) | Entry point to replace with gradient computation |
| `diplomacy_events: Vec<DiplomacyEvent>` on `Simulation` | Output collector; already cleared at tick start |
