# WorldState Dead-Silo Audit

**Scope:** `crates/engine/src/engine.rs` (`WorldState` + all `Simulation::phase_*` hooks invoked from `Simulation::tick_with_emergence_source`, including `disasters.rs::phase_disasters` and `emergence.rs::phase_emergence`).

**Method:** For each `WorldState` field, identify every tick-phase **writer** and every **reader in a different phase**. A field is a **dead silo** when at least one phase writes it and **no other phase reads it**. Reads via helper accessors (`research_tier()`, `belief()`, `faction_relation()`, etc.) count as reads. Same-phase read/write (intra-phase bookkeeping) does not count as cross-phase consumption.

**Tick phase order (actual call sequence):**

```
tick_increment → production → citizen_lifecycle → military → economy
  → planet → diplomacy → tactics → voxel → compact → buildings → diffusion
  → disasters → life → settlement_consumption → emergence
  → research → tech → belief → unrest → cohesion → social_mood
  → stratification → institutions → economic_focus → chronicle
```

(`tick_trade_routes` runs inside `phase_economy`.)

---

## Summary

| Field | Writer phase(s) | Cross-phase readers | Verdict |
|-------|-----------------|---------------------|---------|
| `focus_pressure` | `economic_focus` | *(none)* | Legitimate intra-phase hysteresis |
| `chronicle` | `chronicle` | *(none)* | Legitimate display-only narrative log |
| `chronicle_tech_seen` | `chronicle` | *(none)* | Legitimate intra-phase dedup state |
| `chronicle_age` | `chronicle` | *(none)* | Legitimate intra-phase dedup state |
| `faction_resources` | `economy` / `tick_trade_routes` | *(none)* | **Should be consumed** |
| `resources.wood` | `production` | *(none)* | **Should be consumed** |
| `resources.metal` | `production` | *(none)* | **Should be consumed** |
| `resources.energy` | `production` | *(none)* | **Should be consumed** |

**8 dead silos** across **8 logical fields** (4 bookkeeping/display, 4 simulation gaps).

All other phase-written `WorldState` fields have at least one cross-phase reader and are **not** dead silos.

---

## Dead silos (detail)

### 1. `focus_pressure`

- **Written by:** `phase_economic_focus` — increments when `candidate_economic_focus` disagrees with `economic_focus`; resets on switch or when candidate matches.
- **Read by:** only `phase_economic_focus` (hysteresis counter toward `FOCUS_PRESSURE_THRESHOLD = 5`).
- **External exposure:** tests only; not in `SimulationSnapshot`, watch snapshot, or web dashboard.
- **Verdict:** **Legitimate intra-phase state.** Hysteresis belongs in the tick loop but does not need cross-phase coupling. Consider moving off `WorldState` onto `Simulation` (not required for correctness).

---

### 2. `chronicle`

- **Written by:** `phase_chronicle` — appends tech-breakthrough and golden/dark-age lines; capped at `CHRONICLE_MAX_LEN` (256).
- **Read by:** only `phase_chronicle` (length cap / drain). Exposed via `Simulation::chronicle()` for callers; **no phase consumes it**.
- **External exposure:** serialized in `.civsave`; no server JSON-RPC or web panel wiring found.
- **Verdict:** **Legitimate display-only.** FR-CIV-0100 narrative output for future HUD / legends UI. No simulation feedback expected unless a future `legends` phase reads it.

---

### 3. `chronicle_tech_seen`

- **Written by:** `phase_chronicle` — snapshot of `tech_unlocks` after logging new bits.
- **Read by:** only `phase_chronicle` (dedup: `tech_unlocks != chronicle_tech_seen`).
- **Verdict:** **Legitimate intra-phase dedup state.** Could live on `Simulation` instead of persisted `WorldState`.

---

### 4. `chronicle_age`

- **Written by:** `phase_chronicle` — last recorded era (0 normal / 1 golden / 2 dark).
- **Read by:** only `phase_chronicle` (dedup golden/dark-age lines).
- **Verdict:** **Legitimate intra-phase dedup state.** Same relocation note as `chronicle_tech_seen`.

---

### 5. `faction_resources`

- **Written by:** `tick_trade_routes` (inside `phase_economy`) — debits exporter, credits importer per active `trade_routes` entry.
- **Read by:** only `tick_trade_routes` (stock checks, arbitrage multiplier, margin calc). **No other phase reads faction stockpiles.**
- **Indirect coupling:** trade **profit** updates `faction_treasury`, which *is* read by `phase_unrest`, `phase_stratification`, `phase_economic_focus`, and `phase_diplomacy`. Goods stocks themselves are invisible outside economy.
- **External exposure:** `.civsave` only; not in watch `EconomySnapshot`.
- **Verdict:** **Should be consumed.** Documented as unbounded accumulator **N4** in `UNBOUNDED_ACCUMULATORS.md`.
- **Recommended consumers:**
  - **`phase_production`** — faction-scoped yields into `faction_resources[*]` instead of (or in addition to) global `resources`.
  - **`phase_military`** — metal/energy upkeep per unit, debiting owning faction's stock.
  - **`phase_settlement_consumption` / `phase_citizen_lifecycle`** — food draw from faction or cluster stocks (align with cluster_stocks → faction pipeline).
  - **`phase_diplomacy`** — conflict could seize/deplete exporter stocks, not only treasury.

---

### 6. `resources.wood`

- **Written by:** `phase_production` — `wood * yield_factor` (currently always zero: no building type emits wood in the production loop).
- **Read by:** only `phase_production`. **`resources.food` is the counterexample:** read by `phase_citizen_lifecycle` and `phase_economic_focus`.
- **External exposure:** watch `EconomySnapshot.resources.wood`, web `ResourceBar` — **display-only**.
- **Verdict:** **Should be consumed.** Accumulator with no sink (latent until wood-producing buildings are added).
- **Recommended consumers:**
  - **`phase_buildings`** — construction cost per `Allocator::allocate` parcel.
  - **`phase_diffusion`** — wardrobe era propagation cost (thematic fit).
  - **`phase_production`** — Industrial/Mercantile focus could trade wood → treasury or food.

---

### 7. `resources.metal`

- **Written by:** `phase_production` — `Mine` buildings add metal each tick (`metal_out * yield_factor`; Industrial focus bonus applies to metal).
- **Read by:** only `phase_production`.
- **External exposure:** watch snapshot + web dashboard (display-only).
- **Verdict:** **Should be consumed.** Unbounded accumulator (see `UNBOUNDED_ACCUMULATORS.md` macro resource table).
- **Recommended consumers:**
  - **`phase_military`** — weapon/armor upkeep, scaled by `garrison_level` or unit count.
  - **`phase_buildings`** — civic/industrial parcel metal cost.
  - **`phase_institutions`** — garrison level-up could require metal stockpile threshold.

---

### 8. `resources.energy`

- **Written by:** `phase_production` — `CityCenter` buildings add energy each tick.
- **Read by:** only `phase_production`. **`energy_budget_joules` is separate** and *is* consumed by `phase_economy` (policy drain) and read by `phase_unrest`; the global `resources.energy` scalar never feeds that path.
- **External exposure:** watch snapshot + web dashboard (display-only).
- **Verdict:** **Should be consumed.** Duplicate energy accounting: joule budget vs. resource stock with no link.
- **Recommended consumers:**
  - **`phase_economy`** — optional top-up: `energy_budget_joules += f(resources.energy)` or unified single energy field.
  - **`phase_production`** — energy-intensive Industrial focus could **debit** stock for bonus output (trade-off, not pure accumulation).

---

## Fields written by phases but **not** dead silos (control group)

These are written during the tick and **are** read by at least one other phase:

| Field | Writer(s) | Cross-phase reader(s) |
|-------|-----------|------------------------|
| `tick` | `tick_increment`, replay helpers | virtually all phases |
| `population` | `citizen_lifecycle`, `life` | `research`, `belief`, `unrest`, `economy`, `buildings`, `citizen_lifecycle` |
| `research_progress` | `research` | `tech`, `unrest`, `buildings`, `production`, `economic_focus`, `disasters` (via `research_tier()`) |
| `belief` | `belief`, `unrest`, `disasters` (`try_invoke_divine_power`) | `cohesion`, `institutions`, `economic_focus`, `diplomacy`, `chronicle` |
| `unrest` | `unrest` | `cohesion`, `buildings`, `economy` (trade factor), `diplomacy`, `institutions`, `chronicle` |
| `cohesion` | `cohesion` | `research`, `unrest`, `stratification`, `social_mood`, `military`, `buildings`, `economy`, `diplomacy`, `chronicle` |
| `tech_unlocks` | `tech` | `research`, `military`, `chronicle`, `carrying_capacity()` → `unrest`/`economy` |
| `dispossessed_permille` | `stratification` | `unrest` |
| `temple_level` | `institutions` | `belief` |
| `garrison_level` | `institutions` | `unrest` |
| `economic_focus` | `economic_focus` | `production` (prior tick's focus; runs earlier in same tick) |
| `energy_budget_joules` | `economy` | `unrest` |
| `faction_treasury` | `diplomacy`, `institutions`, `economy`/trade | `unrest`, `stratification`, `economic_focus`, `diplomacy`, `economy` |
| `faction_relations` | `diplomacy` | `economy` (trade factor), `diplomacy` |
| `resources.food` | `production` | `citizen_lifecycle`, `economic_focus` |

---

## Fields never written by any tick phase (out of scope for “dead silo”, noted for completeness)

| Field | Phase readers | Role |
|-------|---------------|------|
| `rng_seed` | `tactics`, `emergence` | Scenario seed; immutable per run |
| `factions` | `diplomacy` | Static registry (names/IDs) |
| `trade_routes` | `economy` / `tick_trade_routes` | Scenario-authored topology; routes never mutate in tick |

These are **configuration silos**, not emergence outputs. They are read but intentionally not phase-written.

---

## Priority remediation (simulation gaps only)

1. **Unify or sink macro resources** (`resources.wood`, `resources.metal`, `resources.energy`) — mirror the existing `resources.food` → `citizen_lifecycle` pattern.
2. **Wire `faction_resources` into production/consumption** — close the faction goods loop so trade stocks affect gameplay beyond treasury margins (N4).
3. **Keep chronicle/focus_pressure/chronicle_* on WorldState or migrate to `Simulation`** — low priority; behavior is correct for their intended roles.

---

*Audit date: 2026-06-16. Read-only static analysis; no source changes, builds, or commits.*
