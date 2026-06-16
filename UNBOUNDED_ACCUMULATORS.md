# Unbounded Accumulators Audit — `crates/engine/src/engine.rs`

**Date:** 2026-06-16  
**Scope:** READ-ONLY re-scan of every `phase_*` implementation in `engine.rs` (plus `tick()` preamble and `tick_trade_routes`, which `phase_economy` calls).  
**Charter:** Coupling audit flags U1–U5 in `EMERGENCE_COUPLING_AUDIT.txt`.  
**Method:** Trace each phase for integer/`Fixed` running sums that grow monotonically without a hard cap, proportional decay, or consumption sink in the same tick DAG.

**Tick order (production → … → chronicle):** `Simulation::tick_with_emergence_source` (`engine.rs` ~1405–1445).  
**Related phases outside this file (not re-scanned here):** `phase_disasters` (`disasters.rs`), `phase_emergence` (`emergence.rs`).

---

## Summary

| ID | Field | Phase(s) | Drain / cap today? | Risk |
|----|-------|----------|-------------------|------|
| U1 | `WorldState.unrest` | `phase_unrest` | Partial: food abundance only (`unrest_delta` −10/tick); garrison shaves rise | **Unbounded** under sustained multi-driver hardship |
| U2 | `WorldState.cohesion` | `phase_cohesion` | Proportional decay `÷500`; floor 0 | **Unbounded** if belief bind persistently exceeds unrest fray |
| U3 | `WorldState.research_progress` | `phase_research` | None | **Pure integrator** |
| U4 | `WorldState.belief` (hardship path) | `phase_unrest` → `add_belief` | Belief has `÷500` decay in `phase_belief`, but hardship inflow scales with standing unrest | **Uncapped inflow**; equilibrium drifts up with unrest |
| U5 | `Simulation.cluster_stocks` (Food) | `phase_life` | None | **Pure integrator** (dead silo) |
| N1 | `WorldState.resources.{food,wood,metal,energy}` | `phase_production` | Read in `phase_citizen_lifecycle`; never decremented in any phase | **Pure integrator** |
| N2 | `Simulation.market_state.prices` | `phase_economy` | `apply_pressure` ±8/tick; `step()` always adds +1..+13/tick; floor 1, no ceiling | **Upward drift** without asymptotic cap |
| N3 | `WorldState.faction_treasury[*]` | `phase_diplomacy`, `tick_trade_routes` | Conflict −50; institution upkeep on poorest; trade debits importer | **Net-positive** trade/diplomacy can grow without ceiling |
| N4 | `WorldState.faction_resources[*]` | `tick_trade_routes` (via `phase_economy`) | Exporter debited; per-good clamp at 0 on withdraw | **Importer stocks** accumulate without cap |

**Intentionally monotonic (not coupling risks):** `WorldState.tick` (time index), `Simulation.next_civilian_id` (ID generator), `WorldState.tech_unlocks` (OR-only bitmask, finite tier gates).

**Bounded / equilibrium fields (not flagged):** `dispossessed_permille`, `temple_level`, `garrison_level`, `focus_pressure`, `chronicle`, `energy_budget_joules` (policy drain), `population` (birth/death sinks), ECS `Psyche.mood.valence` (clamped).

---

## Per-phase scan (`engine.rs`)

### `tick_with_emergence_source` (preamble)

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.tick` | +1/tick | — | N/A (clock) |

---

### `phase_production`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.resources.food` | `+= food_out` | None in tick loop | **N1:** Consume on births/needs or cap stock at `carrying_capacity()` equivalent |
| `state.resources.wood` | `+= …` | None | **N1:** Tie to building upkeep or spoilage decay `stock -= stock/500` |
| `state.resources.metal` | `+= …` | None | **N1:** Sink into maintenance/construction spend each tick |
| `state.resources.energy` | `+= …` | None (global; joules drained separately) | **N1:** Merge with `energy_budget_joules` drain or cap global energy stock |

---

### `phase_citizen_lifecycle`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.population` | births `saturating_add` | deaths `saturating_sub` | Bounded by agents (not flagged) |
| `next_civilian_id` | +1 per birth | — | ID space (not flagged) |
| `AgentCivilian.age` | +1/tick per entity | despawn on death | Per-entity (not flagged) |

No WorldState running-sum integrator beyond population (balanced).

---

### `phase_military`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `MilitaryUnit.morale` | recovery toward 1 | capped at 1 | Bounded |
| `pending_damage` | push engagements | drained in `phase_tactics` | Ephemeral buffer |
| `last_tick_engagements`, `last_tick_combat_pulses` | push | cleared next tick preamble | Ephemeral |

---

### `phase_economy` (+ `tick_trade_routes`)

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.energy_budget_joules` | synced from economy | `drain_energy_budget` via policy allocation | **Bounded** (floor 0) |
| `market_state.prices[*]` | `step()` +1..+13; `apply_pressure` ±8 | floor 1 only | **N2:** Cap prices at e.g. `10 × FOOD_SCARCITY_BASELINE` or add mean-reversion decay |
| `faction_treasury[from]` | `+= profit` (trade), diplomacy +100 | conflict −50; upkeep (institutions) | **N3:** Wealth tax / inflation sink `treasury -= treasury/1000` or cap per-faction treasury |
| `faction_treasury[to]` | diplomacy +100 | `−= profit` (can go negative) | **N3:** Floor at 0 + same cap as above |
| `faction_resources[to]` | `+= quantity` (trade) | exporter debited | **N4:** Consumption sink `adjust_resource(..., -population/1000)` or stock cap per good |

---

### `phase_planet`

Recomputes `climate`, `weather_grid`; updates coastal voxel Y. No scalar WorldState accumulators.

---

### `phase_diplomacy` (cadence: `tick % 500 == 0`)

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `faction_treasury[a,b]` | +100 on trade agreement | −50 on conflict | **N3:** Same treasury cap/decay as economy phase |
| `faction_relations` | `apply_signal` | `decay_faction_relations(0.98)` | Scores clamped in matrix (bounded) |

---

### `phase_tactics`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `last_tick_voxel_damage_count` | += damage | reset 0 at phase start | Ephemeral |
| `last_tick_combat_pulses` | push | cleared next tick | Ephemeral |
| `faction_doctrines[].score` | reassigned each evolve cadence | evolved, not integrated | Not a running sum |

---

### `phase_voxel`

Drains voxel dirty queue into `last_tick_voxel_events` (ephemeral).

---

### `phase_voxel_ca`

Clears and rebuilds `last_tick_abiogenesis_sites` (ephemeral).

---

### `phase_compact`

Voxel compaction only; no accumulators.

---

### `phase_buildings`

Allocates parcels on cadence; no scalar state integrators.

---

### `phase_diffusion`

Updates cohort wardrobe/tools; sets `last_cohort_stats` (display). No unbounded sums.

---

### `phase_life`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `cluster_stocks[cluster].food` | `stock.add(Food, size)` per member | None | **U5:** Spoilage `stock.add(Food, -stock.get(Food)/100)` or wire cluster food into `faction_resources` / agent needs |
| `state.population` | — | `saturating_sub(dead)` | Death sink present |

---

### `phase_research`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.research_progress` | `saturating_add(contribution)` | None | **U3:** Cap at `100_000 × MAX_RESEARCH_TIER` or spend progress on projects `progress -= spend` |

---

### `phase_tech`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.tech_unlocks` | `\|=` tier bits | never cleared | **Bounded** (6 tier bits) |

---

### `phase_belief`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.belief` | `+= worship + temple_level` | `−= belief/500` | Soft equilibrium only; **cap:** `belief.min(MAX_BELIEF)` e.g. `500_000` |

---

### `phase_unrest`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.unrest` | stacked drivers (food, energy, misery, overcrowding, inequality, dispossession) minus garrison | decay only via food abundance (−10 max) | **U1:** Hard cap `unrest.min(10_000)` or proportional decay `unrest -= unrest/200` every tick |
| `state.belief` (via `add_belief`) | `+= unrest/100` | see `phase_belief` | **U4:** Cap hardship inflow `min(unrest/100, 50)` per tick |

---

### `phase_cohesion`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.cohesion` | `+= cohesion_delta(belief, unrest)` | `−= cohesion/500` | **U2:** Hard cap `cohesion.min(1_000_000)` or tie decay to unrest `−= unrest/100` |

---

### `phase_social_mood`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `Psyche.mood.valence` | uplift ≤ +0.02 | clamped `[-1, 1]` | **Bounded** |

---

### `phase_stratification`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.dispossessed_permille` | step toward target | target eroded by cohesion; clamp `[0, 1000]` | **Bounded** |

---

### `phase_institutions`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.temple_level` | step toward target | target from belief; cap `MAX_INSTITUTION_LEVEL` (5) | **Bounded** |
| `state.garrison_level` | step toward target | cap 5 | **Bounded** |
| `faction_treasury[poorest]` | — | upkeep `(temple+garrison)×10`, floor 0 | Sink present (spread driver for N3) |

---

### `phase_economic_focus`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.focus_pressure` | +1 when candidate ≠ focus | cap 10; reset on switch | **Bounded** |
| `state.economic_focus` | enum switch | — | Not an accumulator |

---

### `phase_chronicle`

| Field | Grows? | Drain? | Fix |
|-------|--------|--------|-----|
| `state.chronicle` | push on thresholds | drain to `CHRONICLE_MAX_LEN` (200) | **Bounded** |
| `chronicle_tech_seen`, `chronicle_age` | dedup state | — | Not growing unbounded |

---

## U1–U5 mapping (coupling audit)

| Flag | Field | Confirm |
|------|-------|---------|
| **U1** | `WorldState.unrest` | Confirmed — no ceiling; multi-driver rise can exceed single-path food decay |
| **U2** | `WorldState.cohesion` | Confirmed — proportional decay only; no upper asymptote |
| **U3** | `WorldState.research_progress` | Confirmed — monotonic `saturating_add`; tier effects saturate but scalar does not |
| **U4** | `unrest → belief` | Confirmed — `phase_unrest` `add_belief(unrest/100)` uncapped per tick |
| **U5** | `Simulation.cluster_stocks` | Confirmed — `phase_life` `stock.add(Food, size)` with no downstream consumer |

## Additional integrators (N1–N4)

Not in original U1–U5 list but same failure mode in `engine.rs` phases:

- **N1** Global `WorldState.resources` — production-only inflow.
- **N2** `MarketState.prices` — `step()` net-positive drift without ceiling.
- **N3** Per-faction `faction_treasury` — net wealth accumulation via trade arbitrage and diplomacy trade bonus.
- **N4** Per-faction `faction_resources` — importer-side stock growth without cap.

---

## Phases with no unbounded accumulator

`phase_planet`, `phase_compact`, `phase_buildings`, `phase_diffusion`, `phase_voxel`, `phase_voxel_ca`, `phase_social_mood`, `phase_stratification`, `phase_tech`, `phase_chronicle` (bounded), `phase_institutions` (levels bounded), `phase_economic_focus` (pressure bounded), `phase_military` (morale capped), `phase_citizen_lifecycle` (population balanced).

---

## Recommended priority (bounding fixes)

1. **U5 / N1** — Wire cluster/global food into consumption or add spoilage (strongest dead integrators).
2. **U1 / U4** — Cap unrest and hardship→belief inflow (closes misery→faith→cohesion slow drift).
3. **U3** — Cap or spend `research_progress` (scalar drift vs saturated tier effects).
4. **U2** — Hard cap or unrest-coupled decay on cohesion.
5. **N2 / N3** — Price ceiling and treasury decay (closes economy→unrest→trade loops).

One-line pattern: **`field = field.saturating_add(inflow).min(CAP)`** or **`field -= field/DECAY_DIVISOR`** or **`field -= consumption(demand)`** each tick.
