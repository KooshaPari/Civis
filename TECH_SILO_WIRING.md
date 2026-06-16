# TECH_SILO_WIRING — `TECH_STORAGE` & `TECH_METALLURGY`

**Audit scope:** `crates/engine/src/engine.rs` (`tech_unlocks` bitmask)  
**Date:** 2026-06-16  
**Mode:** read-only (no source edits, no `cargo`, no commit)

## Verdict

Both bits are **set-only silos**. Grep across the entire repo finds **zero gameplay reads** of `TECH_STORAGE` or `TECH_METALLURGY` outside tests and documentation. They are OR'd into `WorldState.tech_unlocks` at research tiers 2 and 3 but never influence any phase outcome.

Four sibling bits **are** wired: `TECH_IRRIGATION`, `TECH_WRITING`, `TECH_SANITATION`, `TECH_GUNPOWDER`.

---

## Bit map & tier gates

| Bit | Constant | Value | Unlocked at | Set by |
|-----|----------|-------|-------------|--------|
| 0 | `TECH_IRRIGATION` | `1 << 0` | `research_tier >= 1` | `tech_unlocks_for_tier` → `phase_tech` |
| 1 | `TECH_STORAGE` | `1 << 1` | `research_tier >= 2` | same |
| 2 | `TECH_METALLURGY` | `1 << 2` | `research_tier >= 3` | same |
| 3 | `TECH_WRITING` | `1 << 3` | `research_tier >= 4` | same |
| 4 | `TECH_SANITATION` | `1 << 4` | `research_tier >= 5` | same |
| 5 | `TECH_GUNPOWDER` | `1 << 5` | `research_tier >= 6` | same |

Setter (only production path):

```2805:2825:crates/engine/src/engine.rs
fn tech_unlocks_for_tier(research_tier: u64) -> u64 {
    let mut bits = 0u64;
    if research_tier >= 1 {
        bits |= TECH_IRRIGATION;
    }
    if research_tier >= 2 {
        bits |= TECH_STORAGE;
    }
    if research_tier >= 3 {
        bits |= TECH_METALLURGY;
    }
    // ... WRITING, SANITATION, GUNPOWDER ...
    bits
}
```

```1786:1788:crates/engine/src/engine.rs
fn phase_tech(&mut self) {
    self.state.tech_unlocks |= tech_unlocks_for_tier(self.research_tier());
}
```

---

## Grep inventory: SET vs READ

### `TECH_STORAGE` (`1 << 1`)

| Site | File:line | Role |
|------|-----------|------|
| const definition | `engine.rs:2798` | declare |
| `bits \|= TECH_STORAGE` | `engine.rs:2811` | **SET** (tier ≥ 2) |
| `tier3 & TECH_STORAGE` | `engine.rs:4891` | test assert |
| `has_tech(TECH_STORAGE)` | `engine.rs:4905,4909` | test assert |
| `tier6 & TECH_STORAGE` | `engine.rs:4927` | test assert |

**Gameplay reads:** none.

### `TECH_METALLURGY` (`1 << 2`)

| Site | File:line | Role |
|------|-----------|------|
| const definition | `engine.rs:2799` | declare |
| `bits \|= TECH_METALLURGY` | `engine.rs:2814` | **SET** (tier ≥ 3) |
| `tier3 & TECH_METALLURGY` | `engine.rs:4892` | test assert |
| `tier6 & TECH_METALLURGY` | `engine.rs:4928` | test assert |

**Gameplay reads:** none.

### `tech_unlocks` field (non-test)

| Site | Role |
|------|------|
| `WorldState.tech_unlocks` (`engine.rs:347`) | persisted state |
| `phase_tech` (`engine.rs:1787`) | **SET** (OR tier mask) |
| `carrying_capacity` (`engine.rs:1202,1205`) | **READ** IRRIGATION, SANITATION |
| `phase_research` (`engine.rs:1776`) | **READ** WRITING |
| `phase_military` (`engine.rs:2385`) | **READ** GUNPOWDER |
| `phase_chronicle` (`engine.rs:1978–1986`) | **READ** (narration dedup only) |
| `has_tech` / `tech_unlocks()` (`engine.rs:1282–1289`) | accessors (unused by phases for STORAGE/METALLURGY) |

### Wired tech pattern (reference)

| Bit | Phase fn | Field / effect |
|-----|----------|----------------|
| `TECH_IRRIGATION` | `carrying_capacity` | `cap` +200k → feeds `phase_economy` `apply_pressure("food", …)` supply |
| `TECH_WRITING` | `phase_research` | `research_progress` +1/tick contribution |
| `TECH_SANITATION` | `carrying_capacity` | `cap` +300k |
| `TECH_GUNPOWDER` | `phase_military` | `morale_recovery` +0.01/tick |

---

## Tick order constraint

`phase_tech` runs **after** `phase_economy`, `phase_production`, and `phase_buildings` in the same tick (`engine.rs:1425–1441`). A tier crossing that sets a new bit takes effect on the **next** tick — same as existing wired bits.

---

## `TECH_STORAGE` — optimal minimal coupling

### Intended semantics

Granaries and preservation blunt staple-market swings and reduce stock loss — **food price volatility** and (future) **spoilage**.

### Why not `phase_unrest` alone?

`research_unrest_mitigation` (`engine.rs:3080–3092`) already documents “advanced food logistics (storage, distribution)” but keys only on `research_tier`, not `TECH_STORAGE`. Wiring STORAGE there would couple to **unrest**, not **prices**. Price volatility has a dedicated, unused hook in `phase_economy`.

### Recommended hook (single site, engine-only)

| Item | Value |
|------|-------|
| **Phase fn** | `Simulation::phase_economy` |
| **Field** | `self.market_state.prices["food"]` (via `MarketState::prices` / `apply_pressure`) |
| **Mechanism** | After `market_state.step(tick)` and `market_state.apply_pressure("food", demand, supply)`, if `self.state.tech_unlocks & TECH_STORAGE != 0`, halve the **net food price delta** for this tick (pull current price halfway back toward pre-phase price). |
| **Effect** | Dampens both sources of food price movement per tick: (1) `MarketState::step` deterministic drift (`crates/economy/src/market.rs:30–45`, ±1..+13 on one good), (2) `apply_pressure` imbalance cap (`MAX_DELTA = 8`, `market.rs:53–60`). |
| **Coupling chain** | `phase_research` → `phase_tech` sets bit → next tick `phase_economy` → calmer `food` clearing price → `phase_unrest` / `phase_citizen_lifecycle` see stabler scarcity signal |

**Sketch (implementer handoff, not applied):**

```
food_before = market_state.prices["food"]
market_state.step(tick)
damp_food_delta_if_storage(food_before)   // new 3-line helper
market_state.apply_pressure("food", demand, supply)
damp_food_delta_if_storage(food_before)
```

Constants: divisor `2` (halve volatility); no new `WorldState` fields.

### Optional second hook (only if spoilage is added)

There is **no spoilage drain** on `state.resources.food` today. If a per-tick stock decay is introduced later:

| Phase fn | Field | Mechanism |
|----------|-------|-----------|
| `phase_production` | `self.state.resources.food` | Apply `food_out *= spoilage_factor` where `spoilage_factor < 1` when surplus; multiply retention by e.g. `1.25` when `TECH_STORAGE` is set |

Defer until spoilage exists; price damp in `phase_economy` is the minimal fix for the current codebase.

---

## `TECH_METALLURGY` — optimal minimal coupling

### Intended semantics

Smelting and metalworking raise mine output and/or pull industrial construction demand.

### Recommended hook (single site)

| Item | Value |
|------|-------|
| **Phase fn** | `Simulation::phase_production` |
| **Field** | `metal_out` → `self.state.resources.metal` |
| **Mechanism** | After `metal_out = metal * yield_factor` and the `EconomicFocus::Industrial` focus branch, if `self.state.tech_unlocks & TECH_METALLURGY != 0`, multiply `metal_out` by `11/10` (+10%). Mirrors existing `focus_bonus` pattern (`engine.rs:2286–2294`). |
| **Coupling chain** | `phase_tech` sets bit → `phase_production` raises `resources.metal` → `phase_economic_focus` / trade routes / faction stocks see more metal |

**Why `phase_production` over `building_demand_signals`:**

- `building_demand_signals.industrial` (`engine.rs:3118`) is already tier-driven; adding metallurgy there requires threading `tech_unlocks` into `phase_buildings` and the pure helper — two call sites.
- `metal_out` is a **one-line conditional** beside an existing tech-adjacent bonus (`EconomicFocus::Industrial`), consistent with how `TECH_WRITING` adds a flat research increment.

### Alternate hook (if construction pull is preferred over yield)

| Phase fn | Field | Mechanism |
|----------|-------|-----------|
| `phase_buildings` → `building_demand_signals` | `DemandSignals.industrial` | Add `+0.2` (clamped) when `TECH_METALLURGY` set; extend helper signature with `tech_unlocks: u64` |

Use this only when industrial parcel growth should lead mine output, not follow it.

---

## Cross-references

| Doc | Note |
|-----|------|
| `EMERGENCE_COUPLING_AUDIT.txt` §D5, §M3 | Confirms dead silo; same fix vectors |
| `docs/traceability/emergent-systems-tracelinks.md` | Open gap row for STORAGE/METALLURGY gameplay |

---

## Suggested tests (future implementation lane)

| Bit | Test idea |
|-----|-----------|
| `TECH_STORAGE` | Two `phase_economy` runs with identical demand/supply; assert smaller `\|Δfood_price\|` when `tech_unlocks \|= TECH_STORAGE` |
| `TECH_METALLURGY` | Fixed mine count; assert `resources.metal` delta higher with bit set vs cleared (other inputs equal) |

---

## Implementation DAG (when approved)

```
T1  Add storage_price_damp helper + wire in phase_economy     (depends: none)
T2  Add metallurgy metal_out multiplier in phase_production   (depends: none)
T3  Unit tests for T1, T2                                     (depends: T1, T2)
T4  Refresh emergent-systems-tracelinks.md matrix row         (depends: T3)
```

T1 and T2 are independent (~2 small edits in `engine.rs`, ~6 tool calls wall clock).
