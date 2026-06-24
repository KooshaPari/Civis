# Criticality Re-Verification — `engine.rs` tick loop

**Date:** 2026-06-16  
**Method:** Read-only static audit (`git show main:crates/engine/src/engine.rs` + satellite `phase_*` in `disasters.rs`, `emergence.rs`, `emergence_metrics.rs`). No `cargo`, no source edits.  
**Ref audited:** `main` @ `eacd43613` (`feat(engine): add PolityMacroState map with unrest dual-write`, 2026-06-16)  
**Workspace note:** Checked-out branch `feat/emergence-batch-25` @ `647a3a07` is **ahead** of `main` for the three prior fixes and for N1/N2/M1 couplings. This report states what is **on `main` today**.

**Prior audit baseline:** `EMERGENCE_COUPLING_AUDIT.txt` §5 ([U1]–[U6]).

---

## Executive summary

| Prior flag | On `main` @ eacd4361 | Verdict |
|------------|----------------------|---------|
| **[U5] `cluster_stocks` food** | `phase_settlement_consumption` drains `1 × members` after `phase_life` adds `1 × members` | **BOUNDED** ✓ |
| **`faction_unrest`** | `unrest_delta(shadow)` accrues; **no** proportional decay | **NOT BOUNDED** ✗ |
| **[FC-3] cohesion × commercial × metal/build** | `phase_buildings` allocates on cohesion-driven commercial signal; **no** wood/metal affordability gate or debit | **NOT BOUNDED** ✗ |

**Couplings N1 / N2 / M1-A / M1-C:** **absent on `main`**. Present only on `feat/emergence-batch-25` (see §4).

**Overall:** **NOT CLEAN** on `main`. One of three claimed fixes verified; two remain open. Six legacy unbounded accumulators from the prior audit still apply, plus FC-3 building-graph growth.

---

## 1. Prior three flags — re-verification

### [U5] `cluster_stocks` food — **BOUNDED** ✓

| Item | Detail |
|------|--------|
| **Field** | `Simulation.cluster_stocks[cluster_id].food` |
| **Feeding term** | `phase_life` §6: `stock.add(Food, size × 1)` per tick |
| **Sink / bound** | `phase_settlement_consumption`: `consumption = size × FOOD_PER_MEMBER_PER_TICK` (1), same tick, immediately after life |
| **Mechanism** | Matched production/consumption → net integrator zero at steady state; stock floored at 0 via `saturating_sub` |
| **Residual risk** | Low: consumption re-queries `ClusterMember` while production uses in-phase `cluster_sizes`; counts should match after §5 writes. N1 (not on main) would feed stocks into market supply without adding a new accumulator. |

### `faction_unrest` — **NOT BOUNDED** ✗

| Item | Detail |
|------|--------|
| **Field** | `WorldState.faction_unrest[id]` |
| **Feeding term** | `phase_faction_unrest`: `delta = faction_unrest_delta_from_shadow(shadow)` → mirrors `unrest_delta` (+1…+50/tick under scarcity shadow above baseline) |
| **Missing bound** | No `FACTION_UNREST_DECAY_DIVISOR` proportional decay (present on `feat/emergence-batch-25` only) |
| **Recommended fix** | `*entry = entry.saturating_sub(*entry / FACTION_UNREST_DECAY_DIVISOR)` after accrual (÷200, matching global unrest equilibrium pattern) |

### [FC-3] cohesion × commercial build × metal — **NOT BOUNDED** ✗

| Item | Detail |
|------|--------|
| **Fields** | `state.cohesion`, `building_graph` (parcel growth), optionally `state.resources.metal` |
| **Feeding loop** | `phase_cohesion` accrues cohesion (÷500 decay only, no ceiling) → `building_demand_signals` commercial `= (cohesion / 1e6).clamp(0,1)` → `phase_buildings` calls `allocator.allocate` whenever any signal > 0.5, on cadence ≥ 4 ticks |
| **Missing bound** | No `building_materials_affordable` pre-check; no `building_material_cost` wood/metal debit (both on `feat/emergence-batch-25`) |
| **Runaway mode** | High cohesion → perpetual commercial/civic/residential/industrial parcel allocation → **unbounded `building_graph` growth** (metal stockpile never consulted on `main`) |
| **Recommended fix** | Gate allocation on affordable `(wood, metal)`; debit stockpiles per allocated parcel (`BUILDING_WOOD_PER_PARCEL=10`, `BUILDING_METAL_PER_PARCEL=5`) |

---

## 2. Full `phase_*` scan — `tick()` order on `main`

Phases invoked from `Simulation::tick_with_emergence_source` (lines ~1460–1487). Status: **CLEAN** = no unbounded scalar accumulator or uncapped positive-feedback loop in this phase; **FLAG** = issue listed.

| # | Phase | Location | Accumulators / feedback | Status |
|---|--------|----------|-------------------------|--------|
| 1 | `phase_production` | `engine.rs` | `state.resources.{food,wood,metal,energy}` += building yields; bounded by building count × yield factors | **CLEAN** (throughput-limited) |
| 2 | `phase_citizen_lifecycle` | `engine.rs` | Birth/death; `population` saturating; birth chance ∈ (0,1] via overcrowding + `food_scarcity_birth_factor` | **CLEAN** |
| 3 | `phase_military` | `engine.rs` | Morale recovery; combat → `pending_damage`; unit HP floored at despawn | **CLEAN** |
| 4 | `phase_economy` | `engine.rs` | Energy budget clamped ≥ 0; `market_state.apply_pressure` ±8/tick; `tick_trade_routes` uses capped multipliers (trade 2×, unrest trade ≥0.5, cohesion trade ≤1.5) | **FLAG** — see [R1] treasury |
| 5 | `phase_planet` | `engine.rs` | Climate/weather recompute; tide offset on registered coastal columns | **CLEAN** |
| 6 | `phase_diplomacy` | `engine.rs` | Every 500 ticks: trade +100 or conflict −50 treasury; threshold terms capped (`BELIEF_PEACE_CAP`, `DIPLOMACY_MIN_CONFLICT_THRESHOLD`) | **FLAG** — see [R1] |
| 7 | `phase_tactics` | `engine.rs` | Doctrine evolve every 64 ticks; damage queue drained | **CLEAN** |
| 8 | `phase_voxel` | `engine.rs` | Drains dirty-event queue | **CLEAN** |
| 9 | `phase_compact` | `engine.rs` | Periodic voxel compact | **CLEAN** |
| 10 | `phase_buildings` | `engine.rs` | Cohesion/unrest/population → demand signals; **unbounded parcel allocation** | **FLAG** — FC-3 |
| 11 | `phase_diffusion` | `engine.rs` | Wardrobe/tools era propagation with LOD | **CLEAN** |
| 12 | `phase_disasters` | `disasters.rs` | Discrete wildfire/quake triggers from weather | **CLEAN** |
| 13 | `phase_life` | `engine.rs` | Needs/death; clustering; `cluster_stocks` production | **CLEAN** (with consumption sink) |
| 14 | `phase_settlement_consumption` | `engine.rs` | Drains cluster food 1×members | **CLEAN** (pairs with §13) |
| 15 | `phase_emergence` | `emergence.rs` | Culture drift, social graph, psyche; `CultureProfile` traits drift in bounded simplex | **CLEAN** (micro silo on main — no macro feedback) |
| 16 | `phase_research` | `engine.rs` | `research_progress += pop/1000 + cohesion bonus (≤+50%) + sentience (≤+50)` | **FLAG** — [U3] |
| 17 | `phase_tech` | `engine.rs` | OR-only `tech_unlocks` bitmask | **CLEAN** |
| 18 | `phase_belief` | `engine.rs` | `belief += pop/2000 + temple_level`; ÷500 decay | **FLAG** — [U4] no ceiling |
| 19 | `phase_unrest` | `engine.rs` | Multi-driver delta; per-driver caps on components; **no global ceiling** | **FLAG** — [U1] |
| 20 | `phase_faction_unrest` | `engine.rs` | Per-faction shadow unrest | **FLAG** — see §1 |
| 21 | `phase_cohesion` | `engine.rs` | `cohesion += bind − fray`; ÷500 decay; **no ceiling** | **FLAG** — [U2] |
| 22 | `phase_social_mood` | `engine.rs` | Psyche valence += uplift ≤ 0.02/tick; clamped [-1, 1] | **CLEAN** |
| 23 | `phase_stratification` | `engine.rs` | `dispossessed_permille` sticky step, cap 1000 | **CLEAN** |
| 24 | `phase_institutions` | `engine.rs` | Temple/garrison levels ≤ `MAX_INSTITUTION_LEVEL` (5); one step/tick; poorest treasury upkeep floored at 0 | **CLEAN** |
| 25 | `phase_economic_focus` | `engine.rs` | `focus_pressure` cap 10; hysteresis threshold 5 | **CLEAN** |
| 26 | `phase_chronicle` | `engine.rs` | `chronicle` vec capped `CHRONICLE_MAX_LEN` (200) | **CLEAN** |
| — | `sample_emergence` | `emergence_metrics.rs` | 50-tick boundary metrics sampler; no state integration | **CLEAN** |

---

## 3. Remaining unbounded accumulators & feedback loops on `main`

| ID | Field | Feeding term | Bounding fix (recommended) |
|----|-------|--------------|----------------------------|
| **[U1]** | `state.unrest` | `phase_unrest`: food price + energy blackout + agent misery (≤30) + overcrowding (≤30) + inequality (≤25) + dispossession (≤25) − garrison; rise damped via research/cohesion but **no ceiling** | Hard cap or stronger proportional decay on total unrest |
| **[U2]** | `state.cohesion` | `phase_cohesion`: `cohesion_delta(belief, unrest)` + ÷500 decay only | Ceiling on cohesion scalar, or cap bind term |
| **[U3]** | `state.research_progress` | `phase_research`: linear in population + capped bonuses; **accumulator never decays** | Tier-only consumption of progress, or decay/divisor |
| **[U4]** | `state.belief` | `phase_unrest`: `faith_from_hardship = unrest / 100` (uncapped inflow); `add_belief` has no max | Cap hardship faith/tick or belief ceiling |
| **[U5]** | `cluster_stocks` food | — | **FIXED on main** (§1) |
| **[U6]** | `faction_treasury` spread | `phase_diplomacy` trade +100/500t; `tick_trade_routes` profit transfer; richest accumulates without bound → inequality unrest (≤25/tick) | Treasury soft cap, progressive tax, or diminishing trade bonus |
| **[FC-3]** | `building_graph` | Cohesion → commercial signal → allocate parcels | **Material gate + debit** (on batch-25, not main) |
| **[R1]** | `faction_unrest` | Scarcity shadow → `unrest_delta` | **Proportional decay** (on batch-25, not main) |

### Closed / bounded cross-phase loops (no new flag)

- **Misery → unrest → belief → diplomacy threshold:** stabilising arm; threshold capped at `2 × DIPLOMACY_BASE_CONFLICT_THRESHOLD`.
- **Cohesion → unrest damp / trade boost / research bonus:** effect saturates via divisors/caps; **scalar cohesion still unbounded** ([U2]).
- **Belief → temple (≤5) → belief:** institution cap prevents runaway level.
- **Research tier → carrying capacity → food price → births:** birth factor ≤ 1.0 (scarcity only damps).
- **Market prices:** `apply_pressure` ±8/tick; floor price 1.

---

## 4. Recent couplings (N1, N2, M1-A, M1-C) — `main` vs batch-25

| Coupling | On `main`? | Risk if merged (batch-25 static read) |
|----------|------------|--------------------------------------|
| **N1** settlement `cluster_stocks` → market food supply (`SETTLEMENT_FOOD_MARKET_WEIGHT=2`, scaled demand/supply) | **No** | **Low.** Stocks bounded (§1); adds supply term to already ±8 capped price step. Possible soft loop: abundant stocks → lower price → less unrest → (indirect) — damped. |
| **N2** `CultureProfile` similarity → `diplomacy_culture_threshold_bias` (≤ `CULTURE_PEACE_SPAN=3000`) | **No** | **Low.** Threshold shift capped; one-tick culture lag; no new accumulator. |
| **M1-A** `micro_cohesion_delta(Psyche.beliefs[0])` → `phase_cohesion` (bind ≤12, fray ≤18/tick) | **No** | **Low.** Per-tick delta capped; feeds [U2] scalar which still lacks ceiling. |
| **M1-C** `micro_social_trust_permille(SocialGraph)` → `society_trade_factor` (micro cap 250‰, combined cap 750‰) | **No** | **Low.** Trade multiplier capped at 1.75×; feeds [U6] treasury path already flagged. |

**N3** (settlement overlap → diplomacy pair selection) is on batch-25 only; replaces rotating faction-id pair pick. No new scalar accumulator; pair selection is deterministic from cluster layout.

---

## 5. Verdict

### On `main` @ `eacd4361`

**NOT CLEAN.**

- **Verified bounded:** [U5] cluster food (production/consumption sink).
- **Still open:** `faction_unrest` integrator, [FC-3] building expansion, plus legacy [U1]–[U4], [U6].
- **N1 / N2 / M1-A / M1-C:** not wired on `main`; no coupling-induced regressions on this ref.

### On `feat/emergence-batch-25` @ `647a3a07` (informational)

All three prior fixes appear present (`FACTION_UNREST_DECAY`, `building_materials_affordable` + debit, named `CLUSTER_FOOD_*` constants + N1/N2/M1). A merge re-verify should confirm the three flags **CLOSED** and re-run this phase scan with N1/N2/M1 paths active (treasury and cohesion scalars remain the dominant long-horizon risks).

---

## 6. Tick-order DAG (reference)

```text
production → citizen_lifecycle → military → economy → planet
  → diplomacy → tactics → voxel → compact → buildings → diffusion
  → disasters → life → settlement_consumption → emergence
  → research → tech → belief → unrest → faction_unrest → cohesion
  → social_mood → stratification → institutions → economic_focus → chronicle
  → sample_emergence (50-tick boundary)
```

---

*Generated by read-only static audit. No tests executed.*
