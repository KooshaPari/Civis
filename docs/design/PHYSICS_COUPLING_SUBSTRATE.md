# Physics Coupling Substrate

> **Doctrine (recap, do not relitigate):** All emergent layers — life, language,
> faction, religion, trade, architecture, economy, climate — must couple
> **bidirectionally** through an effectively-real shared physics substrate via
> **continuous field gradients and conserved resources** (downward causation).
> Inter-layer `API` calls between silos are *prohibited* as a coupling mechanism.
>
> Today the substrate exists (planet, climate, voxel) but the emergent layers
> still read **abstract scalars** (`hardship`, `group_size`, `contact_pressure`,
> `agent_detection_bias`) plumbed by the engine. This document is the gap-closer.

**Scope of this document:** research/design only. No code changes in this PR.
See `§7 Integration Plan` for the staged work that follows.

---

## 1. Current state — what `crates/` actually does

### 1.1 Substrate crates (the physics is there)

| Crate | Field it owns | Source |
|-------|---------------|--------|
| `civ-planet` | `GeologyMap` (iron / silicate / water / ice plate heights) — geological relief, tectonics | `crates/planet/src/geology.rs:1` |
| `civ-planet` | Orbital / axial context that climate reads | `crates/planet/src/lib.rs` |
| `civ-climate` | Energy balance, advection, weather, climate cell state | `crates/climate/src/lib.rs:1` |
| `civ-voxel` | `MaterialId` CA grid, fluid CA, pairwise `ReactionRule` table, worldgen | `crates/voxel/src/{material,reactions,fluid_ca,worldgen}.rs` |
| `civ-diffusion` | Bass/Rogers S-curve adoption math — **pure math, no field I/O** | `crates/diffusion/src/lib.rs:1` |

Concrete reaction rules already exist in voxel (`crates/voxel/src/reactions.rs:36-119`):
fire+oil, lava+water→stone+steam, acid+stone→air, water+ice→ice, gunpowder+fire, etc.
These are **the downward causation channel** today — and they only run inside the
voxel CA, not anywhere else.

### 1.2 Emergent layer crates (the silo)

| Crate | What it does today | How it couples (current) |
|-------|--------------------|--------------------------|
| `civ-genetics` | DNA, sentience thresholds, divergence spawning | pure logic + RNG, no field I/O |
| `civ-species` | Species expression from DNA | pure logic |
| `civ-needs` | Per-agent needs vectors | `civ_needs::Needs` consumed by `civ-agents` |
| `civ-agents` (psyche, culture, social, civ-ai) | Civilian state machines, mood, beliefs, clusters | `phase_emergence` reads `hardship`, `group_size`, `contact_pressure` from engine — **abstract, not field-derived** |
| `civ-laws` | Law books, policy multipliers | pure rules; no field reads |
| `civ-diplomacy` | Treaties, reputation, war declarations | scalar trade balance, no field reads |
| `civ-tactics` | Combat resolution, war_bridge | reads civilian + voxel fire/material, but no continuous gradient |
| `civ-economy` | Extraction, chains, market, allocation, institution | reads material counts per tile, writes ledger entries |
| `civ-legends` | Saga graph, role assignment, name refs | pure event-sourcing layer |
| `civ-research` | Tech tree, firepass unlocker | cost multipliers; no field reads |
| `civ-ai` | LLM-driven civ-ai decisions | reads aggregate state only |
| `engine::religion` | `Religion`, `Belief`, `emerge_belief(hardship, group_size, agent_detection_bias)` | **abstract** (`crates/engine/src/religion.rs:42-67`) |
| `engine::language` | `LanguageState`, `tick_language(lang, contact_pressure)` | **abstract** (`crates/engine/src/language.rs:48-60`) |
| `engine::faction_emergence` | `cluster_into_factions(ideologies, k)` over 8-dim `AgentIdeology` | abstract ideology vector, no field reads (`crates/engine/src/faction_emergence.rs:23-72`) |

The `phase_emergence` orchestrator (`crates/engine/src/emergence.rs:1-80`) calls
these in a fixed order: **genetics → culture → social → psyche → legends ingest →
civ-ai naming**. The order is itself a form of API coupling: each phase is invoked
by name, with hand-shaped arguments. The substrate never sees the result.

### 1.3 The gap, stated precisely

A famine is a moisture gradient, a heatwave is a temperature gradient, a
battlefield is a material-flux gradient, a trade route is a population-pressure
gradient. Today none of those gradients are the channel — each emergent layer
gets a hand-fed scalar. The result is **theater**: the simulation *looks*
coupled (a fire triggers a "hardship bump") but the bumps do not feed back into
the substrate as field deformations, so the substrate is decorative.

---

## 2. The shared physics substrate

Six continuous field arrays, all on the same world grid (a chunked 3D lattice
indexed by `IVec3`, the same lattice used by `civ-voxel` and the
`GeologyMap` plate grid). All six are conserved (Lagrangian) up to sources and
sinks; all six are the **only** channel through which emergent layers may
communicate with each other.

| Symbol | Field | Units | Owner today | Conserved? |
|--------|-------|-------|-------------|------------|
| **T** | Temperature | K (clamped `[0, 6000]`) | `civ-climate` | Energy-conserving, with radiative sink |
| **M** | Moisture / water | kg/m³-equivalent | `civ-climate` (precip) + `civ-voxel` (WATER/ICE/STEAM) | Mass-conserving |
| **E** | Energy / joules | J-equivalent | `civ-climate` (radiative balance) + `civ-voxel` (fire exo/endo) | Energy-conserving |
| **F** | Material flux | kg-equivalent per cell per tick | `civ-voxel` (fluid CA + reaction table) | Mass-conserving |
| **P** | Population pressure | agents / m² (or biomass proxy) | `civ-agents` (hecs) | Advects with agents |
| **B** | Biomass / resources | kg-equivalent | `civ-economy::extraction` writes; `civ-agents` reads | Mass-conserving |

The substrate layer (`civ-physics-substrate`, proposed, see §7) holds a single
struct `PhysicsFields { T, M, E, F, P, B }` plus the linear-algebra primitives
needed to read/write at a point, sample a gradient, or integrate a flux.

### 2.1 What "shared" means concretely

For any layer X, the **only** way to influence layer Y is:

1. X writes a value to one or more of {T, M, E, F, P, B} at a point `p`.
2. The substrate applies one tick of field evolution (advection, diffusion,
   reaction, sink/source, conservation enforcement).
3. Y reads one or more of {T, M, E, F, P, B} at `p` (or its gradient ∇ field).

There is no `religion.bump(faction)`, no `economy.notify(language)`. If a
religion changes the food market, it does so by **driving people to a place**
(writing P), or by **wasting biomass on ritual fires** (writing E via FIRE
spawn, then F via oxidation products). The market then sees B fall. The market
does not subscribe to religion.

### 2.2 Field evolution operators (the substrate's job)

For each field, the substrate exposes an `evolve(fields, dt)` that does:

- **Advection** — upwind scheme on T, M, F, P (mass transport).
- **Diffusion** — explicit Laplacian on T, M, E (heat, water, energy).
- **Reaction** — applies the `REACTIONS` table on F (voxel material CA) and
  pumps enthalpy into E.
- **Population dispersal** — `P` follows a gradient of B (food-seeking) and is
  bounded by carrying-capacity `K(p) = f(M, T)`.
- **Biomass regrowth** — `B` regrows at rate `μ · M · σ(T_opt − T)` clipped to
  `B_max(p)`; agents harvest it.
- **Conservation clamps** — total `Σ T`, `Σ M`, `Σ E`, `Σ F`, `Σ P`, `Σ B` are
  reported in invariants; if any field's total changes by more than a budget
  per tick the engine reports a leak.

### 2.3 The downward-causation contract

- **Upward causation (bottom-up):** T, M, E, F evolve from physics first.
  Climate, geology, and voxel reactions run before any agent step. This is
  already true in `phase_climate → phase_voxel → phase_agents`.
- **Downward causation (top-down):** agents, factions, religions, economies,
  and architectures *can* write back to T, M, E, F, P, B — but **only through
  the field's own units and operators**. A faction cannot teleport water; it
  can only move a population (write P) whose needs and labor modify E and M
  (draft animals drink, irrigation moves M, ritual fire spawns FIRE voxels
  which feeds the CA which writes E).

The shared substrate is the *only* downward-causation channel. The "downward"
part is enforced by making every layer's write go through `PhysicsFields::set(...)`
which type-checks units and rate-limits writes (e.g. one faction can write at
most N joules per tick — no thermonuclear factions).

---

## 3. Coupling matrix (layer × field, R/W)

Legend: **R** = layer reads field; **W** = layer writes field (through the
substrate's typed setters). A blank cell means the layer has no business
touching that field. Cells marked `·` mean read-only consumers (e.g. UI).

**Coupled-fields count** = non-blank cells with at least one of {R, W} active
*and* that read or write being non-trivial (i.e. not just "any layer can read
temperature for weather"). See §3.2 for the precise counting rule.

### 3.1 The matrix

| Layer (emergent)        | T (temp) | M (moisture) | E (energy) | F (material flux) | P (population) | B (biomass) |
|-------------------------|:--------:|:------------:|:----------:|:-----------------:|:--------------:|:------------:|
| **life** (species, genetics, needs)        | R/W | R | R | R | W | R/W |
| **language** (drift, split, borrow)        | ·   | ·   | ·   | ·   | R | ·   |
| **faction** (ideology clusters, territory) | R   | R   | R   | R   | R/W | R   |
| **religion** (beliefs, rituals, taboo)     | R   | R   | R/W | R/W | R   | R/W |
| **trade** (chains, market, allocation)     | ·   | R/W | R   | R/W | R   | R/W |
| **architecture** (buildings, infra)        | R/W | R/W | R/W | R/W | R   | R   |
| **economy** (institution, allocator)       | R   | R   | R   | R   | R   | R/W |
| **climate** (energy balance, advection)    | R/W | R/W | R/W | R   | ·   | R   |
| **tactics** (combat, war_bridge)           | R/W | R   | R/W | R/W | R/W | R   |
| **diplomacy** (treaties, reputation)       | ·   | ·   | ·   | ·   | R   | R   |
| **laws** (policy multipliers)              | ·   | ·   | ·   | ·   | R   | R   |
| **legends** (event sourcing)              | ·   | ·   | ·   | ·   | ·   | ·   |

### 3.2 Counting rule (for the PR report)

A **coupled-field** is one cell (layer, field) where the layer either:

- **Reads the field as a continuous gradient** ∇field (not just an "is it raining?" boolean) **and** that read materially influences the layer's emergent behavior, **OR**
- **Writes the field** through the substrate's typed setters (changing the
  field's value at one or more cells).

Reads that are pure UI (e.g. legends showing "Battle of Ash Valley" with the
ambient T as flavor text) do not count. Boolean/derived reads that just gate a
behavior on "is temperature > 0?" without consuming a gradient also do not
count.

Applying that rule:

- **climate × T,M,E** = 3 (full R/W)
- **life × T,M,E,F,B** = 5 (life writes biomass via growth+death, reads T,M,E,F as gradients; writes P via birth/death)
- **faction × T,M,E,F,P,B** = 6 (faction reads gradients to choose territory centroid and writes P via migration)
- **religion × T,M,E,F,B** = 5 (rituals write E via fire and F via burnt offerings; taboos read M and B gradients to choose which actions to forbid; hardship = |∇T| + |∇B|)
- **trade × M,F,B** = 3 (trade writes biomass and material flux, reads moisture for transport feasibility)
- **architecture × T,M,E,F** = 4 (buildings shade T, drain M, vent E, redirect F via channels)
- **economy × T,M,E,F,B** = 5 (institutions read gradients to set tax zones, allocator writes B via harvest)
- **tactics × T,E,F,P** = 4 (fire spreads F, casualties write P, fires write E)
- **language × P** = 1 (drift rate modulated by population pressure, drift is fundamentally a contact-pressure phenomenon, which is a P gradient)
- **diplomacy × P,B** = 2 (treaty triggers on |∇P| and resource competition reads B)
- **laws × P,B** = 2 (policy multipliers modulate harvest and migration ceilings)
- **legends** = 0 (pure event-sourcing sink, no field coupling by doctrine)

**Total coupled-fields = 40.**

(Rough sanity check: with 12 layers × 6 fields = 72 cells, ~56% are coupled.
The matrix is dense — that's the point; the substrate is the *connective tissue*.)

---

## 4. Lags and feedback — putting the system at edge-of-chaos (Wolfram Class 4)

The doctrine of an emergent sandbox is to be **at the edge of chaos** (Langton
1989; Kauffman 1993; Wolfram Class 4 — complex, persistent, propagating
structures). Three tuning knobs and three delay stages do it.

### 4.1 Three delay stages per tick

The substrate's `evolve(fields, dt)` is run in **three operator-split stages**
per tick, and emergent layers are scheduled between them:

```
[tick n]
  1. SUBSTRATE-PHYSICS    evolve T,M,E,F (advection, diffusion, reaction)   dt_phys
  2. SUBSTRATE-ECOLOGY    evolve P,B given current T,M,E,F                    dt_eco
  3. EMERGENT LAYERS      life, language, faction, religion, trade,
                          architecture, economy, climate-feedback,
                          tactics, diplomacy, laws                          (writes back to substrate)
  4. SUBSTRATE-LOG        append event log + invariants + legends ingest
```

The interleave is the **lag** that produces edge-of-chaos: agents see the
physics that exists *now*, but their writes only affect physics that exists
*next tick*. This is the same lag pattern that produces Wolfram Class 4 in
elementary CA (rule 110, etc.) — the substrate's evolution rule and the
agents' influence rule are different operators applied in alternation.

### 4.2 Three tuning knobs

Define the following dimensionless parameters on the substrate:

- **α (diffusivity ratio)** = D_T / (D_M + D_E). Below 0.01, heat doesn't
  spread — agents in a cold cell stay cold forever (frozen). Above 10, the
  world homogenizes (heat-death). Target α ∈ [0.3, 2.0].
- **β (write-rate ratio)** = max layer write rate / substrate sink rate. Below
  0.001, emergent layers are theater (writes evaporate before the next read).
  Above 10, emergent layers dominate and physics becomes a slave (the "explosion"
  failure mode). Target β ∈ [0.05, 0.5].
- **γ (memory ratio)** = reaction timescale / tick. Below 0.01, materials
  teleport (fire spreads everywhere in one tick — explosive). Above 100,
  nothing propagates (theater again). Target γ ∈ [0.1, 10].

The substrate's `evolve()` exposes these and reports the current values in
`Metrics`. A "Class-4 zone" diagnostic in `civ-engine/src/invariants.rs`
asserts that all three ratios are in range and trips an alarm otherwise.

### 4.3 Why this is edge-of-chaos

- If α is too low, agents in bad cells can't escape — the simulation collapses
  to a frozen attractor.
- If α is too high, gradients vanish and the simulation collapses to a
  heat-death attractor (every cell averages out).
- If β is too low, agents can't influence the world — theater.
- If β is too high, a single agent event can set the world on fire — explosion
  (literal fire if it's the E channel).
- If γ is too low, materials propagate across the map per tick — explosive.
- If γ is too high, materials don't propagate — theater.

The three lag stages and three ratio parameters together define a 3D parameter
space; the Class-4 zone is a 3D box in that space. The substrate's
`invariants` module asserts we are inside it. We **do not** tune by hand — we
provide the assertion and a parameter auto-tuner that runs once at sim init
from scenario YAML.

### 4.4 Downward causation, in practice

Consider a religion that mandates a "rain dance" ritual every 100 ticks.
The rule, in the substrate-first style:

1. **Read:** `hardship = |∇T|_p + |∇B|_p` (sampled around the temple site).
   This replaces today's `emerge_belief(hardship: f32, ...)` `hardship` arg,
   which is hand-fed.
2. **Decide:** if `hardship > 0.7`, perform the ritual this tick.
3. **Write:** the ritual spawns 1000 FIRE voxels in a 3×3×3 block. This is
   the only write.
4. **Substrate step:** the FIRE voxels react with adjacent M (water) and F
   (biomass). The CA consumes FIRE, releases STEAM and E (heat) and ash.
5. **Next layer's read:** the climate layer sees E go up and M go down.
   Precipitation adjusts (more evaporation). The economy layer sees B go
   down (ash-fall fertilization actually *raises* regrowth in adjacent
   cells after a delay). The faction layer sees a P gradient shift (the
   ritual attracts spectators).

No `religion.notify(faction)`. No `religion.bump(hardship)`. Just field
writes that propagate through physics.

---

## 5. Failure modes and tripwires

The invariants module in `civ-engine/src/invariants.rs` enforces five
tripwires. The first three are the divergence attractors; the last two are
the "stuck" attractors.

### 5.1 Heat-death (over-diffusion)

**Symptom:** world-averaged `σ(T)` collapses toward 0; gradients vanish; agents
stop making interesting decisions because all cells are equivalent.
**Detection:** `σ(T) < 0.01 K` for >1000 ticks; `max(|∇T|) < 0.001 K/cell`.
**Tripwire:** clamp `D_T *= 0.5` per tick until recovered. **Root cause:**
α too high. **Auto-tuner:** lowers `D_T`, raises `D_M` slightly.

### 5.2 Explosion (over-write or runaway reaction)

**Symptom:** `Σ E` grows by >10% per tick for >10 ticks; or `Σ F` shows
net material creation (violates mass conservation); or a single reaction
cascades to >50% of the map in one tick.
**Detection:** invariant `le_ΣE_per_tick ≤ ε_E`; `le_ΣF_per_tick ≤ ε_F`
(0.5% mass budget per tick); CA cascade depth ≤ 32 cells/tick.
**Tripwire:** reduce all layer write rates by 50% for 50 ticks; quarantine
the offending reaction in the `REACTIONS` table.
**Root cause:** β too high or γ too low.

### 5.3 Theater (under-coupling)

**Symptom:** writes from emergent layers have negligible effect on field
values (|Δ field| < float epsilon for 1000 ticks); legends log is empty;
emergence feed events stop.
**Detection:** `emission_lag = mean over layers of (write_rate_observed /
write_rate_intended)`. If `emission_lag < 0.001` for 500 ticks, trip.
**Tripwire:** scale layer write rates by 10× and tighten the substrate
type-check so writes can't go below 1e-6 of a unit.
**Root cause:** β too low (writes evaporating) or the layer is bypassing
the substrate (calling another layer's API directly — a doctrine
violation; the call-site detector in §6 catches this).

### 5.4 Frozen (under-diffusion + over-memory)

**Symptom:** `σ(P)` collapses; no agents move; no events; simulation
appears stuck on the world map.
**Detection:** `max(|∇P|) < 1e-6 agents/cell` for 1000 ticks; `Σ births < 1`
in 1000 ticks.
**Tripwire:** raise `D_P` (population dispersal) by 2× per tick until
recovered.
**Root cause:** α too low, γ too high.

### 5.5 Runaway (positive-feedback loop)

**Symptom:** a feedback loop forms between two layers (e.g. religion →
FIRE → climate dry → hardship → religion); oscillation grows.
**Detection:** FFT of the integrated `|∇B|` over time; if peak amplitude
>10× baseline, trip.
**Tripwire:** identify the dominant cycle via cross-correlation between
layer write timeseries; insert a randomized 1–5 tick refractory in the
dominant layer's write.
**Root cause:** a missing lag or a too-strong coupling constant.

---

## 6. Doctrinal enforcement (catching silo-API regressions)

The doctrine is "no API calls between emergent layers; only the substrate".
This must be **enforced**, not just encouraged. The plan is:

1. A new crate `civ-physics-substrate` (see §7.2) holds `PhysicsFields` and
   the only public function to mutate it: `PhysicsFields::set(field, point,
   delta)`. Layer crates depend on `civ-physics-substrate`; they do not
   depend on each other for "write" semantics.
2. A `deny.toml`-style lint check in `scripts/ci/coupling_audit.sh` greps
   `crates/` for `use civ_agents` inside `crates/economy`, `use civ_religion`
   inside `crates/faction`, etc. — all "inter-layer" `use` statements
   (i.e. one emergent layer importing another emergent layer's types) are
   flagged. Climate/planet/voxel are the only "downward" targets; everything
   else is an emergent layer.
3. The audit allows `civ-agents → civ-needs` (lifecycle sub-coupling within
   the same logical layer) and the same crate's own sub-modules. The
   dependency graph among emergent layers is required to be a DAG with
   `civ-physics-substrate` as the only sink.
4. An `invariant` in `civ-engine/src/invariants.rs` (`emission_lag`) tracks
   whether each layer's writes are actually being read by the substrate;
   sustained near-zero emission trips the theater tripwire (§5.3).

---

## 7. Integration plan

Phased, doc-then-implementation. **This PR is docs only**; the implementation
work is laid out for the next 3 sprints.

### 7.1 Sprint 1 — substrate crate skeleton (no behavior change)

- Add `crates/physics-substrate/Cargo.toml` and `src/lib.rs`.
- Define `pub struct PhysicsFields { pub T, M, E, F, P, B: Vec<f32> }` on a
  shared grid descriptor.
- Implement `pub fn evolve(fields: &mut PhysicsFields, dt: f32) -> EvolveReport`
  with empty operators (identity for now).
- Wire `phase_substrate` into `Simulation::tick` between `phase_climate` and
  `phase_agents`, dispatching to `evolve()`.
- **No emergent layer changes yet.** The new phase is a no-op that runs the
  invariant checks against the current planet/climate/voxel state projected
  into `PhysicsFields`. The first integration test asserts `Σ T`, `Σ M`, `Σ E`,
  `Σ F` are conserved within a tick (sum of substrate reads of existing
  state = sum of writes).
- New invariant: `civ-engine/src/invariants.rs::substrate_conservation()`.
  Reuses `civ_voxel::FluidCa::conservation_report()` shape.

### 7.2 Sprint 2 — first real coupling (life + climate)

- Wire `civ-needs` and `civ-species` to read T and M gradients (and write P
  and B through births/deaths/harvest).
- Replace `civ-climate`'s advection to use `PhysicsFields` as backing store
  for T and M.
- Add `civ-economy::extraction` writes to B (it already does — formalize the
  contract: it writes to `PhysicsFields::B` via the substrate, not its own
  `Ledger`).
- Coupling-matrix cells now active: life×{T,M,F,P,B}, climate×{T,M,E,B},
  economy×B (12 cells of 40). This is the first measurable "real" coupling.

### 7.3 Sprint 3 — emergent layers read gradients

- Replace `engine::religion::emerge_belief(hardship, ...)`'s `hardship`
  argument with a sampled value from `|∇T| + |∇B|` at the agent's position.
  Old signature becomes a deprecated wrapper for replay compatibility.
- Replace `engine::language::tick_language(lang, contact_pressure)`'s
  `contact_pressure` with `|∇P|` at the cluster's centroid.
- Replace `engine::faction_emergence::cluster_into_factions`'s ideology
  weights with a 14-dim vector = (current 8-dim ideology) ⊕ (∇T, ∇M, ∇B,
  ∇P) sampled at the agent. This makes territory preference an emergent
  property of gradients, not a hand-tuned heuristic.
- Add the coupling audit script `scripts/ci/coupling_audit.sh`.
- Add the auto-tuner `crates/physics-substrate/src/auto_tune.rs` that
  initializes α, β, γ from a target `Class-4` zone and a scenario
  perturbation analysis (run once at sim init).

### 7.4 Sprint 4 — downward causation (architecture, tactics, religion writes)

- Allow `engine::religion::spread_religion` to spawn FIRE voxels (Ritual
  cost) through the substrate.
- Allow `civ-tactics` to write back P (casualties) and E (battlefield
  fires) through the substrate.
- Allow `architecture` (new module) to write T (building shade), M
  (drainage), E (heat venting), F (channeled flow).
- All 40 cells of the matrix active.

### 7.5 Sprint 5 — invariants + auto-tuner + dashboard

- Implement the five tripwires from §5 in `civ-engine/src/invariants.rs`.
- Surface them in `civ-watch`'s dashboard (substrate health panel).
- Run the auto-tuner on the standard 4 scenarios (Genesis, Stress,
  Famine, Ice) and capture the converged α, β, γ; commit those as scenario
  defaults in `scenarios/*.yaml`.

### 7.6 Acceptance criteria (full integration done when all are green)

- [ ] All 12 layers listed in §3.1 are wired to `civ-physics-substrate`.
- [ ] `coupling_audit.sh` finds zero cross-layer `use` statements.
- [ ] `substrate_conservation()` invariant holds for 100k ticks across
      all 4 reference scenarios.
- [ ] All five tripwires in §5 fire correctly in their targeted failure
      scenarios (each failure scenario is a unit test).
- [ ] α ∈ [0.3, 2.0], β ∈ [0.05, 0.5], γ ∈ [0.1, 10] for all reference
      scenarios at tick 10k.
- [ ] `emission_lag` > 0.001 for every layer in the standard 10k-tick run.
- [ ] Replay determinism preserved (every change is a function of the
      substrate state at tick t, not of layer-internal RNG).

---

## 8. Why this is the *right* substrate (and not just another API)

The substrate is right because:

1. **It is conserved.** Every write is checked against a conservation
   invariant; runaway is impossible by construction.
2. **It is continuous.** Reads are gradients, not booleans. This is what
   makes the system "edge-of-chaos" — booleans collapse to discrete
   automata (Class 1 or 2); continuous gradients with delays are Class 4.
3. **It is shared.** All emergent layers read the same six arrays, so any
   correlation they discover is real (a religion that wants moisture-poor
   terrain and a faction that wants the same terrain *will* both be there
   because they both read ∇M — not because someone wired them together).
4. **It is downward-causal.** Writes from emergent layers go through typed
   setters with rate limits; there is no path for a layer to *bypass* the
   physics and stomp the world.
5. **It is observable.** `Metrics::substrate_health()` reports α, β, γ,
   conservation budgets, and per-layer emission lag. We can *see* the
   system is at edge-of-chaos, not just hope it is.

The substrate is not an API. It is a **field of conserved resources** that
every layer lives inside. Coupling *is* the substrate.

---

## 9. Open questions for review

1. **Field count.** Six is enough for the doctrine; is it the right number
   for our scenarios? Trade and architecture may want a 7th (`U` — "use" /
   "habitation intensity") that distinguishes "biomass is here" from
   "agents are using the biomass". Pro: cleaner read for economy. Con: more
   invariants to track.
2. **Reaction table ownership.** Today `civ-voxel` owns `REACTIONS`. Should
   the substrate own it (so climate, life, and architecture can all declare
   reactions)? The principle says yes; the implementation cost is moving
   ~120 lines of constants.
3. **Replay determinism.** Bass/Rogers diffusion in `civ-diffusion` is
   already deterministic. The new substrate must be too — every step is a
   pure function of `(fields_t, writes_from_layers, dt)`. The integration
   plan in §7.1 calls this out; flag if a non-deterministic write is
   proposed anywhere.
4. **L5 visual pass interaction.** The `Frame3d` / 16³ mesh features in
   the L5 plan read the substrate's T, M, F directly. The substrate
   becoming the source of truth for *visual* fields is a happy side
   effect, not a goal; do not let the L5 plan drive substrate design.

---

## 10. References

- Wolfram, S. (2002). *A New Kind of Science*. Class 4 elementary CA.
- Langton, C. (1989). Computation at the edge of chaos. *Physica D*.
- Kauffman, S. (1993). *The Origins of Order*. NK models, tunable
  criticality.
- Rogers, E. (1962). *Diffusion of Innovations*. (S-curve adoption —
  already in `civ-diffusion`.)
- Bass, F. (1969). A new product growth model for consumer durables.
  *Management Science*. (Already in `civ-diffusion`.)
- Campbell, D. T. (1974). "Downward causation" in hierarchically
  organized biological systems. *Studies in the Philosophy of Biology*.
  (Doctrinal anchor for the substrate contract in §2.3.)
- Price, G. (1995). The nature of selection in *The Major Transitions in
  Evolution*. (Used in §4.4 for the ritual-fire example.)
- Civis codebase: `crates/planet`, `crates/climate`, `crates/voxel`,
  `crates/diffusion`, `crates/agents`, `crates/economy`, `crates/laws`,
  `crates/diplomacy`, `crates/tactics`, `crates/needs`, `crates/species`,
  `crates/genetics`, `crates/legends`, `crates/research`, `crates/ai`,
  `crates/engine/src/{emergence,religion,language,faction_emergence}.rs`.
