# Coupling Recheck — New `tick()` Feedback Loops

**Date:** 2026-06-16  
**Scope:** READ-ONLY re-scan of `crates/engine/src/engine.rs` after three recent couplings: per-faction unrest (`phase_faction_unrest`), wood/metal building consumption (`phase_buildings`), and M1-A ideology → cohesion (spec + wiring check).  
**Baseline:** `UNBOUNDED_ACCUMULATORS.md`, `EMERGENCE_COUPLING_AUDIT.txt` (U1–U5).  
**Method:** Trace `tick_with_emergence_source` phase DAG for monotonic integrators and closed positive-feedback paths introduced or strengthened by the new edges. No `cargo` run.

---

## Executive summary

| Verdict | Count |
|---------|-------|
| **New unbounded integrator** | 1 (`faction_unrest`) |
| **New closed loops that amplify an existing unbounded field** | 2 (faction unrest ↔ treasury scarcity; cohesion/commercial building ↔ metal stock when cohesion is already drifting up) |
| **New coupling with sink only (depletion, not runaway)** | 1 (wood debit with zero wood production) |
| **M1-A in tree** | **Not wired** — `phase_cohesion` still uses macro `cohesion_delta(belief, unrest)` only; risks below are **pre-merge** from `M1_IMPL_SPEC.md` |

The two landed code paths (`faction_unrest`, building material debit) do **not** introduce a brand-new *unbounded* scalar beyond `faction_unrest` itself. They **do** add new feedback arms into diplomacy and construction that can sustain or accelerate pre-existing U1/U2 drift when scarcity persists.

---

## Tick context (relevant phases)

```
… phase_buildings (#10)     ← reads prior-tick cohesion & unrest; debits wood/metal
… phase_emergence (#14)     ← updates Psyche.beliefs[0] (M1-A input, not yet consumed)
… phase_unrest (#18)        ← global unrest integrator (U1)
… phase_faction_unrest (#19)← NEW per-faction integrator
… phase_cohesion (#20)      ← cohesion integrator (U2); M1-A not added yet
… phase_diplomacy (#6, cadence 500) ← pair_unrest → war threshold
```

`phase_buildings` runs **before** unrest/cohesion phases in the same tick, so demand signals lag one tick — this dampens same-tick oscillation but does not cap long-run integrators.

---

## New coupling inventory

| Coupling | Write site | Read site(s) | Drain / cap today |
|----------|------------|--------------|-------------------|
| Per-faction unrest | `phase_faction_unrest` → `faction_unrest[id]` | `phase_diplomacy` (`pair_unrest`) | Decay only when faction wealth shadow at baseline (`unrest_delta` −10); **no proportional decay, no ceiling** |
| Wood/metal construction | `phase_buildings` → `resources.{wood,metal}` | `building_materials_affordable` gate | `saturating_sub` on spend; wood inflow **0**/tick (`phase_production` sets `wood = Fixed::ZERO`) |
| M1-A ideology → cohesion | *Not in `engine.rs`* | Spec: `phase_cohesion` + `micro_cohesion_delta(&world)` | Spec caps per-tick bind/fray at +12/−18; `cohesion` still has ÷500 decay only |

---

## Risks — field, feeding term, bounding fix

### FC-1 — `WorldState.faction_unrest` (NEW integrator)

| Item | Detail |
|------|--------|
| **Field** | `WorldState.faction_unrest: HashMap<u32, u64>` |
| **Feeding term** | `faction_unrest_delta_from_shadow(faction_wealth_scarcity_shadow(treasury, faction_resources))` → reuses `unrest_delta`: **+1…+50/tick** when below comfort (`TREASURY_COMFORT` + food shadow), **−10/tick** only when wealthy |
| **Loop path** | Sustained treasury/food shortfall → `faction_unrest` ↑ every tick with no ceiling → `pair_unrest` in `diplomacy_conflict_threshold(belief+cohesion, pair_unrest)` erodes war tolerance (capped at `UNREST_WAR_CAP` = 8 000 currency) → conflict −50 treasury / 500 ticks → absolute wealth ↓ → scarcity shadow stays high → `faction_unrest` keeps integrating |
| **Why positive-feedback** | Integrator has **no** `faction_unrest / N` decay and **no** `min(cap)`; per-tick rise is capped but standing stock is not. Diplomacy war erosion is capped, but the unrest scalar itself is not. |
| **Bounding fix** | Add proportional decay each tick: `*entry -= *entry / FACTION_UNREST_DECAY_DIVISOR` (e.g. 200), mirroring `cohesion`; or hard cap `min(10_000)`; or decay toward 0 when shadow ≤ baseline (not just −10/tick delta). |

---

### FC-2 — `WorldState.faction_unrest` × `faction_treasury` (diplomacy scarcity spiral)

| Item | Detail |
|------|--------|
| **Field** | `faction_treasury[*]` (feeds FC-1 shadow) and `faction_unrest` |
| **Feeding term** | `pair_unrest` → `diplomacy_conflict_threshold` war term `(pair_unrest / UNREST_WAR_DIVISOR).min(UNREST_WAR_CAP)` |
| **Loop path** | High `faction_unrest` → lower conflict threshold → more `DiplomacyKind::Conflict` when disparity ≥ threshold → both parties `−50` treasury → both parties drop below wealth comfort → both accrue `faction_unrest` (+1…+50/tick) in parallel |
| **Why positive-feedback** | Conflict does not narrow disparity (symmetric debit), but **does** depress absolute wealth, keeping scarcity shadow elevated and preventing the −10 decay arm from engaging. Institution upkeep on poorest faction (`phase_institutions`) adds the same absolute drain. |
| **Bounding fix** | Floor treasury spread effects separately from absolute level (unrest from *relative* dispossession only); add post-conflict unrest decay `−= unrest/100`; cap conflict frequency per pair; or route conflict cost only to higher-treasury party to avoid mutual impoverishment loops. |

---

### FC-3 — `WorldState.cohesion` × `resources.metal` (commercial build demand)

| Item | Detail |
|------|--------|
| **Field** | `WorldState.cohesion` (U2) and `WorldState.resources.metal` |
| **Feeding term** | `building_demand_signals` → `commercial: (cohesion / 1_000_000).clamp(0, 1)` → `building_parcel_count` → `building_material_cost` (5 metal / parcel) |
| **Loop path** | High cohesion → commercial signal > 0.5 → extra parcel on cadence → metal debit → *no* parcel→ECS-mine feedback (`phase_production` counts `Building` entities, not parcel graph) → metal only recovers via existing mines → **no return path into cohesion** in the same chain |
| **Why listed** | Not a cohesion runaway by itself; **amplifies U2** when cohesion is already net-positive (belief bind + low unrest fray): construction spends metal without adding production capacity that would fund calmer wealth shadows. Metal can still integrate (N1) if mine output < civic+commercial+industrial demand. |
| **Bounding fix** | Cap parcels per cadence (e.g. `min(pending, 2)`); add `metal -= metal/500` spoilage; wire parcels to mine `Building` spawns so supply scales with demand; or gate commercial demand on `metal` stock `(metal/1000).clamp(0,1)`. |

---

### FC-4 — `WorldState.unrest` × `resources.{wood,metal}` (civic build demand)

| Item | Detail |
|------|--------|
| **Field** | `WorldState.unrest` (U1) and construction stocks |
| **Feeding term** | `civic: (unrest / 500).clamp(0, 1)` in `building_demand_signals` |
| **Loop path** | High unrest → civic parcel demand → wood/metal debit → construction does **not** reduce unrest (no read-back in `phase_unrest` / garrison) |
| **Why listed** | **Not** a positive-feedback loop on unrest (no term feeds unrest from building). It is a **one-way pump** that drains stocks while U1 unrest remains unbounded under multi-driver hardship. |
| **Bounding fix** | Tie civic completion to `garrison_level` target or `unrest` decay (−k per parcel); cap civic signal at 0.5 unless `metal`/`wood` surplus; add maintenance upkeep on parcels each tick. |

---

### FC-5 — `WorldState.resources.wood` (production/consumption mismatch)

| Item | Detail |
|------|--------|
| **Field** | `WorldState.resources.wood` |
| **Feeding term** | `phase_production`: `wood = Fixed::ZERO` then `resources.wood += wood * yield_factor` (always **0**); `phase_buildings`: `−BUILDING_WOOD_PER_PARCEL × parcels` |
| **Loop path** | Monotonic depletion → `building_materials_affordable` false → construction halts. **No upward spiral** — negative bounded at 0. |
| **Why listed** | Pre-existing N1; building consumption **accelerates depletion** and can strand civic/commercial/industrial demand signals high while materials are gone (stall, not runaway). |
| **Bounding fix** | Add wood production (lumber `BuildingType`) or faction trade in wood; or remove wood gate until production exists. |

---

### FC-6 — `WorldState.cohesion` (M1-A **pre-merge** risk)

| Item | Detail |
|------|--------|
| **Field** | `WorldState.cohesion` |
| **Feeding term** *(when M1-A lands)* | `micro_cohesion_delta(world)` → up to **+12/tick** bind from low `Psyche.beliefs[0]` variance (`M1_IMPL_SPEC.md` §4) **added to** `cohesion_delta(belief, unrest)` |
| **Loop path** | `phase_emergence` homophily converges `beliefs[0]` → persistent `+12` micro bind → cohesion ↑ → `cohesion_unrest_damp` shrinks unrest rise → `agent_misery_unrest` ↓ via `phase_social_mood` uplift → lower unrest fray on cohesion → `cohesion_research_bonus` / `cohesion_trade_factor` / diplomacy `belief+cohesion` peace arm strengthen → **macro belief** also fed by `unrest/100` (U4) adding bind via `belief/200` |
| **Why positive-feedback** | Per-tick micro bind is capped, but **standing `cohesion` is not** (U2). Combined macro+micro bind can exceed `cohesion/500` decay indefinitely when unrest is damped low. |
| **Bounding fix** | Land M1-A with `cohesion.min(1_000_000)`; or `micro_bind *= (1 - cohesion/1_000_000)` diminishing returns; or raise decay to `−= max(cohesion/500, unrest/100)`; keep `MICRO_BIND_CAP` ≤ macro fray cap as spec intends. |

---

### FC-7 — `WorldState.belief` (M1-A **pre-merge**, indirect)

| Item | Detail |
|------|--------|
| **Field** | `WorldState.belief` |
| **Feeding term** | M1-A does **not** write `belief`; indirect via cohesion → diplomacy `belief().saturating_add(cohesion())` peace term |
| **Loop path** | Higher cohesion (from FC-6) inflates effective peace input to `diplomacy_conflict_threshold` without increasing `belief` decay load on the belief scalar itself |
| **Why positive-feedback** | Peace bonus capped at `BELIEF_PEACE_CAP`, so diplomacy output bounded; **cohesion integrator** still the weak point. |
| **Bounding fix** | Pass `belief` and `cohesion` separately to `diplomacy_conflict_threshold` with independent caps; do not `saturating_add` into one peace numerator. |

---

## Couplings checked — bounded / no new runaway

| Path | Why safe |
|------|----------|
| `faction_unrest` → diplomacy war term | War erosion capped at `UNREST_WAR_CAP`; threshold floored at `DIPLOMACY_MIN_CONFLICT_THRESHOLD` |
| `pair_unrest` vs global `unrest` in diplomacy | Global `unrest` no longer drives war threshold directly; **decouples** U1 from diplomacy — does not create new growth term |
| `cohesion` → `cohesion_unrest_damp` | Divisor capped at 10× damp; rise floored at 1 — negative feedback on U1 |
| `cohesion` → `cohesion_trade_factor` | Boost capped at +50% (`COHESION_TRADE_CAP_PERMILLE`) |
| `research_tier` → `building_cadence` | Cadence floored at 4 ticks; speeds attempts but gated by affordability |
| Building parcels → `phase_production` | **No edge** — parcel graph does not spawn `Building` ECS entities; no production positive loop |

---

## Pre-existing integrators still active (unchanged by this recheck)

| ID | Field | Notes |
|----|-------|-------|
| U1 | `unrest` | Multi-driver rise; food-only decay path |
| U2 | `cohesion` | ÷500 decay only; commercial building adds new spend sink when cohesion high |
| U3 | `research_progress` | Pure integrator |
| U4 | `belief` ← `unrest/100` | Hardship inflow uncapped per tick |
| N1 | `resources.{food,metal,energy}` | Wood now drains faster; metal gains construction sink (partial relief) |

---

## M1-A implementation status

`grep micro_cohesion_delta crates/engine` → **no matches**.

Current `phase_cohesion` (`engine.rs` ~1878–1888):

```rust
let delta = cohesion_delta(self.state.belief, self.state.unrest);
self.state.cohesion = (self.state.cohesion as i64 + delta).max(0) as u64;
// … cohesion / COHESION_DECAY_DIVISOR
```

Per `M1_IMPL_SPEC.md` §5.2, expected:

```rust
let delta = cohesion_delta(self.state.belief, self.state.unrest)
    + micro_cohesion_delta(&self.world);
```

**FC-6 and FC-7 apply when that edit lands**, not to today's binary.

---

## Recommended fix priority (new edges only)

1. **FC-1** — Cap or decay `faction_unrest` (highest leverage; new unbounded field).
2. **FC-2** — Break symmetric conflict impoverishment or add post-conflict unrest relief.
3. **FC-5** — Restore wood inflow or disable wood gate until production exists.
4. **FC-6** — Ship M1-A with cohesion hard cap or diminishing micro_bind.
5. **FC-3** — Cap parcels per cadence or link parcels to mine output.

---

## Files referenced

- `crates/engine/src/engine.rs` — `tick_with_emergence_source`, `phase_faction_unrest`, `phase_buildings`, `phase_cohesion`, `phase_diplomacy`, `faction_wealth_scarcity_shadow`, `building_demand_signals`
- `M1_IMPL_SPEC.md` — M1-A handoff (ideology consensus → cohesion delta)
- `UNBOUNDED_ACCUMULATORS.md` — U1–U5 baseline audit
