# Warfare — Emergent Three-Layer Combat System (Design Spec)

**Status:** Design / planner spec. No implementation. Specs + acceptance criteria + WBS only.
**FR namespace:** `FR-CIV-WAR-*` (this doc) building on the shipped `FR-CIV-TACTICS-*` substrate in `crates/tactics`.
**Charter anchor:** [emergence-charter.md](../guides/emergence-charter.md) — *war is never scripted; it emerges from polity / resource / ideology conflict over the Layer-0 substrate.*
**Research anchors:** [empire-at-war-foc.md](../research/empire-at-war-foc.md) (galactic/strategic surface), [call-to-arms.md](../research/call-to-arms.md) (direct-control + on-map logistics tactical sublayer), [sim-misc.md](../research/sota-tech/sim-misc.md) (DF coarse pre-sim, SoS LOD aggregation).

---

## 0. Governing principle: war is an emergent measure, not an authored mode

There is **no `War` object, no `declare_war()` verb, no `faction.at_war: bool` flag** authored anywhere in Civis. Consistent with the charter (a "faction" is emergent cluster overlap, NOT `faction: u32`), *war* is a **named region of the emergent-relations state space**: a sustained, mutually-coercive interaction between two polity clusters, detected and labelled by an observer system, never set by fiat.

The three layers below are **readers/writers of pre-existing emergent state** (polity clusters, economy, agent psyche, culture/ideology). They add *resolution* and *rendering*, not new authored concepts:

| Layer | What it is | Reads from | Writes to | Game-feel analog |
|-------|-----------|-----------|-----------|------------------|
| **STRATEGIC** | Polity-cluster map; war as a detected relation; theaters | polity clusters, ideology fields, resource maps, trade graph | belief/intent fields, theater objectives, mobilization pressure | EAW:FoC galactic layer |
| **OPERATIONAL** | Army-scale maneuver, supply, logistics | strategic theaters, settlement stock (economy), terrain (planet/voxel) | unit positions, supply state, attrition, requisition demand | Call-to-Arms on-map logistics |
| **TACTICAL** | Per-soldier voxel-destructible combat, RTS + direct control | operational unit positions/supply, doctrine library, agent psyche | voxel `DamageEvent`s, agent strength/death, engagement stats | Men-of-War / CtA squad combat (**already shipped** in `crates/tactics`) |

The layers form a **closed feedback loop with the rest of the sim**: tactical outcomes → operational attrition → strategic war-exhaustion → polity/ideology drift → new or terminated wars. The doctrine GA evolves *across* this loop.

---

## 1. STRATEGIC layer — emergent diplomacy, war detection, theaters

### 1.1 Concept
A coarse, low-frequency layer over the polity-cluster substrate. It does not *decide* wars; it **continuously measures relations between clusters** and surfaces the subset that have crossed into sustained coercion. EAW:FoC's lesson (research §1, §7): the strategic map is the *primary surface where intent forms*, and corruption/influence are **outcomes of underlying rules**, not scripted conquest logic.

### 1.2 Inputs it reads (never authored, all emergent)
- **Polity clusters** (`crates/agents` / settlement co-location + kinship + culture overlap) — the membership graph that defines "sides." A cluster is a fuzzy, overlapping set, not an id.
- **Resource/economy maps** (`crates/economy`, `crates/needs`) — scarcity, surplus/deficit, contested resource voxels, blocked trade routes.
- **Ideology/culture fields** (`crates/agents` cultural drift) — belief-distance between clusters; in-group/out-group salience.
- **Border friction** — co-located, low-kinship, high-belief-distance populations sharing a resource frontier.

### 1.3 Emergence triggers (the war-onset measure) — `FR-CIV-WAR-001`
War **emerges** when a sustained-coercion measure between two clusters crosses a hysteresis band. Pseudocode (spec only):

```
tension(A,B) =
      w_res  * contested_resource_pressure(A,B)     // economy/needs deficit over shared resource
    + w_ideo * belief_distance(A,B) * contact_area  // cultural field
    + w_grv  * accumulated_grievance(A,B)           // memory of prior raids/seizures (agent psyche)
    - w_trade* trade_interdependence(A,B)            // economy graph: trade dampens war
    - w_kin  * kinship_overlap(A,B)

if tension(A,B) > ONSET_THRESHOLD for >= ONSET_DWELL ticks:
    emit WarObserved(A,B)            // a *label*, written to the relations/event log, not a flag set by fiat
if tension(A,B) < CEASE_THRESHOLD for >= CEASE_DWELL ticks (CEASE < ONSET → hysteresis):
    emit WarEnded(A,B)              // exhaustion/attrition/ideology-shift drove tension back down
```

- Thresholds/weights are **tunables in a war-config**, not gameplay scripts; real randomness welcome (charter: determinism not required) so identical tension does not always tip identically.
- **No central arbiter declares war.** `WarObserved` is produced by the same detector that labels trade relations, alliances, etc. — war is one band on a continuous relations spectrum.

### 1.4 Theaters — `FR-CIV-WAR-002`
A *theater* is an emergent spatial cluster of contested frontier (resource frontier + border friction + active engagements), found by spatial clustering (e.g. grid-binned DBSCAN over contested cells), **not** an authored map region. Each theater carries an emergent **objective vector** (control resource X, sever trade route Y, displace population Z) derived from *why* tension is high there — directly feeding operational maneuver targets. EAW analog: theaters are the "planets worth fighting over," but here discovered from the substrate.

### 1.5 Mobilization pressure — `FR-CIV-WAR-003`
The strategic layer writes a **mobilization-pressure field** back onto clusters (a belief/intent signal in agent psyche), which *raises the utility* of military-role adoption, requisition, and unit formation in the agent AI — it does **not** spawn armies. Armies form because agents, under pressure + available resources, choose military roles (charter: structures/roles self-organize). War-economy and conscription thus **emerge** (see §4).

### 1.6 Cadence & LOD
Strategic ticks are coarse (minutes of wall-clock / many sim-ticks), LOD-aggregated per `sim-misc.md` SoS discipline. Far-from-camera wars resolve **statistically** (abstracted operational/tactical rollups); near-camera wars get full operational + tactical resolution. DF coarse-pre-sim pattern (research §1): historical/off-screen wars run as cheap rollups logged to the Legends/event stream.

---

## 2. OPERATIONAL layer — army-scale logistics, supply, maneuver

### 2.1 Concept
Between strategic intent and per-soldier combat. Call-to-Arms is the reference (research §3–§5): **logistics is an on-map friction system experienced during the fight**, not a macro production number. Supply scarcity *creates* emergent battlefield shape. `crates/tactics` already ships the movement/maneuver primitives (`operational.rs` hook, `movement.rs` `OperationalMovementConfig`, `pathfinding.rs` A*/BFS, `formation.rs`, `MilitaryPhaseConfig`); this layer specifies the **logistics/supply/attrition model** that sits on top.

### 2.2 Inputs / outputs
- **Reads:** theater objective vectors (§1.4), unit positions + grid (existing `MilitaryUnitSample`), terrain passability (planet/voxel + `grid_obstacles.rs`), settlement stock & production (economy), road/desire-path network (charter architecture — roads form along desire-paths).
- **Writes:** unit target positions (into existing operational movement), **supply state per unit/formation**, **attrition events**, and **requisition demand** back into the economy.

### 2.3 Supply & logistics model — `FR-CIV-WAR-010`
Supply is **physical and on-map**, not abstract (CtA §4–§5):
- Each formation carries finite **consumables** (ammunition→combat capacity, rations→cohesion/health, materiel→repair). Modeled as voxel/economy stock moving along supply lines, NOT a global pool.
- **Supply lines** are paths along the emergent road/desire-path network from a source settlement (economy stock) to the formation. Cutting the path (terrain, voxel destruction, enemy interdiction in a theater) **starves** the formation — combat capacity and cohesion decay.
- **Resupply** is emergent local behavior (CtA dropped-kit/scavenging analog): starved formations forage from terrain/captured stock, or retreat toward supply — an operational maneuver pressure, not a scripted state.

### 2.4 Maneuver — `FR-CIV-WAR-011`
Operational movement (already in `movement.rs`) is **driven by theater objectives + supply gradients**: formations advance toward objectives only while supplied; otherwise the supply gradient dominates (advance-to-contact vs. fall-back-to-supply emerges from the same utility comparison). Flow-field / pooled pathing per `sim-misc.md` LOD aggregation for the statistical mass; A* for near-camera units.

### 2.5 Attrition & cohesion — `FR-CIV-WAR-012`
Operational attrition (starvation, exposure, terrain, fatigue) drains unit **strength** (existing `civ-engine` fixed-point strength the war bridge already mutates) and **cohesion** *before and between* tactical engagements. A broken-cohesion formation routs (operational withdrawal) rather than fighting — feeding refugee/displacement flows (§5). This is where most casualties occur in real war and must dominate the tactical layer in aggregate.

### 2.6 Bridge to tactical — `FR-CIV-WAR-013`
When two opposing formations close within engagement range in a theater under near-camera LOD, the operational layer **hands off** the contact to the tactical war bridge (existing `WarBridge::resolve_combat`, `MilitaryPhaseConfig`). Operational supply/cohesion state **parameterizes** the tactical engagement (low ammo → fewer shots; low cohesion → worse formation integrity / earlier rout). Far-from-camera contacts resolve via a **statistical combat rollup** (LOD), logged to the event stream, never spawning per-soldier voxels.

---

## 3. TACTICAL layer — per-soldier voxel-destructible combat (extends `crates/tactics`)

### 3.1 Concept — mostly shipped
This is the **already-implemented** core. The design here records how it plugs into the two layers above and the direct-control/RTS UX, **not** a rebuild. Reference is CtA (research §1–§2): *direct control is a mode of intervention in the same battlefield state, not a minigame; strategic intent persists while direct control temporarily overrides one agent's local decisions.*

Existing substrate in `crates/tactics` (read-only, do not duplicate):
- `war_bridge.rs` — `WarBridge::resolve_combat` per-soldier engagements with LOS gating → voxel `DamageEvent`s; `CombatEngagement`.
- `los.rs` / `fog_of_war.rs` — line-of-sight + per-faction fog (`FogOfWar`, `fog_vision_radius`).
- `formation.rs` / `movement.rs` / `pathfinding.rs` / `grid_obstacles.rs` — squad formations, movement, A*/BFS, obstacle gating.
- `doctrine_fitness.rs` + `Doctrine`/`DoctrineLibrary`/`evolve_doctrine` (`lib.rs`) — the doctrine GA.
- `DamageEvent` → the voxel CA (`crates/voxel`) applies destructible-terrain damage; mass-conserving per Layer-0 physics.

### 3.2 RTS + direct control — `FR-CIV-WAR-020`
Two co-existing control modes over the *same* tactical state (CtA continuity-of-agency):
- **RTS / command mode:** player issues squad-level orders (move-to, hold, approach-vector, supporting fire) — these set the same operational/tactical targets the AI would. Strategic intent (theater objective) persists as a default.
- **Direct control:** player embodies one soldier/vehicle crew; overrides *only that agent's* local decision policy temporarily. On release, the agent resumes utility/GOAP behavior under the standing order. No separate physics/state — same voxel battlefield.

### 3.3 Reads from psyche/agent state — `FR-CIV-WAR-021`
Per-soldier behavior reads **emergent agent psyche** (charter Layer-1): morale, fear, in-group loyalty, fatigue modulate fire discipline, rout threshold, and willingness to follow orders. Ideology/culture set who is even regarded as an enemy. Casualties and atrocities **write back grievance/trauma** into psyche → feeds strategic grievance (§1.3) and post-war culture.

### 3.4 Voxel destruction feedback — `FR-CIV-WAR-022`
`DamageEvent`s reshape terrain/structures via the voxel CA; the altered voxel field **changes LOS, cover, and passability** for subsequent ticks at *all three* layers (collapsed bridge cuts an operational supply line; cratered wall opens a tactical firing lane). Destruction is therefore a cross-layer coupling, not a cosmetic effect.

---

## 4. Doctrine-GA evolution across the loop

### 4.1 Existing mechanism
`evolve_doctrine` evolves a `DoctrineLibrary` (unit-composition genomes) scored by `score_doctrine_fitness` from `FactionEngagementStats` (engagement pressure + voxels removed). Shipped under `FR-CIV-TACTICS-023`.

### 4.2 Spec extension — close the loop with operational + strategic outcomes — `FR-CIV-WAR-030`
Doctrine fitness today reads *tactical* engagement stats only. Extend the fitness signal (spec; not yet coded) to include **operational + strategic outcomes** so doctrines that win battles but lose the war are selected against:
```
fitness(doctrine) =
      score_doctrine_fitness(...)                 // existing tactical term
    + k_terr  * net_theater_objective_gain        // operational/strategic: did this doctrine take objectives?
    + k_supl  * supply_efficiency                  // operational: low attrition / sustainable
    - k_loss  * own_attrition_and_routs            // operational: pyrrhic doctrines penalized
    - k_grv   * civilian_grievance_generated       // strategic blowback (§5) lowers long-run fitness
```
- Per-cluster libraries evolve **independently** → doctrine *diversity emerges* (different clusters under different terrain/economy/ideology evolve different ways of war). No global "correct" doctrine.
- Cultural diffusion (charter) can **copy** successful doctrines across contact networks (memetic spread) in addition to GA mutation — a second, emergent evolution channel.
- Real randomness welcome (charter); no determinism requirement on the GA.

---

## 5. Civilian impact — refugees, war-economy, reconstruction

War must visibly cost the civilian substrate (charter: everything emerges; war is not isolated from economy/population). All three are **emergent consequences**, not authored systems.

### 5.1 Refugees / displacement — `FR-CIV-WAR-040`
Theater activity + voxel destruction + routing formations (§2.5) raise local **danger fields**; civilian agents' existing needs/safety drives make flight the high-utility choice → emergent refugee flows along the road network, away from theaters toward safe/kin settlements. Receiving settlements face resource strain (economy) → can *re-trigger* §1.3 tension elsewhere (war spreads via displacement). No "refugee" entity type — just agents whose safety drive dominates.

### 5.2 War-economy — `FR-CIV-WAR-041`
Mobilization pressure (§1.5) + requisition demand (§2.2) reshape the **emergent market**: military materiel demand spikes, civilian production is crowded out, scarcity/inflation emerge from the existing market rules (charter: markets of varying types emerge). Conscription emerges as agents adopt military roles under pressure; labor shortages ripple through production. This is the existing economy responding to the war signal — no parallel "war economy" subsystem.

### 5.3 Reconstruction — `FR-CIV-WAR-042`
On `WarEnded` (§1.3), danger fields decay; surviving/returning agents rebuild via the **existing architecture/engineering emergence** (houses/roads rebuilt where needs+resources allow) over the voxel-scarred terrain. Reconstruction speed is an emergent function of remaining population, materiel, and grievance (high residual grievance → re-tension → fragile peace). Legends/event log (research §1) records the war's history for the player-facing feed.

---

## 6. Cross-layer state contract (read/write summary)

| Emergent substrate | STRATEGIC | OPERATIONAL | TACTICAL |
|--------------------|-----------|-------------|----------|
| Polity clusters (`agents`) | **R** membership/relations, **W** mobilization pressure | R sides | R faction id, **W** grievance |
| Economy/markets (`economy`,`needs`) | R scarcity/trade, **W** war-demand signal | **R/W** supply stock, requisition | — |
| Ideology/culture (`agents`) | **R** belief-distance, **W** in/out-group salience | — | **R** enemy-ID, **W** trauma/culture |
| Agent psyche (`agents`) | R grievance memory | R fatigue/cohesion | **R** morale/fear, **W** trauma |
| Terrain/voxel (`planet`,`voxel`) | R resource frontier | **R** passability, supply lines | **R** cover/LOS, **W** `DamageEvent` |
| Doctrine library (`tactics`) | — | R for composition | **R/W** GA evolve |

R = reads, W = writes, **bold** = primary owner of that coupling.

---

## 7. Acceptance criteria (system-level)

- **AC-WAR-1 (no authored war):** Codebase contains no `War` struct, `declare_war`, or boolean at-war flag set by fiat; war presence is derived from the tension measure (§1.3) and lives only in the relations/event log. *(grep-able invariant for review.)*
- **AC-WAR-2 (onset emerges):** With zero scripted intervention, two clusters under sustained contested-resource + belief-distance pressure produce a `WarObserved` event; lowering tension (trade restored / ideology converges / exhaustion) produces `WarEnded`, with hysteresis (no flapping).
- **AC-WAR-3 (theaters discovered):** Theaters are spatial clusters of contested cells with non-empty objective vectors; none are hand-placed.
- **AC-WAR-4 (logistics bites):** Cutting a formation's supply line measurably degrades its tactical combat capacity and can force operational withdrawal without any direct combat.
- **AC-WAR-5 (tactical reuse):** The tactical layer routes through existing `WarBridge`/`MilitaryPhaseConfig`/doctrine GA — no parallel combat implementation.
- **AC-WAR-6 (direct control continuity):** Direct control overrides exactly one agent's policy and restores standing-order AI on release; battlefield state is shared (same voxel world), not a separate mode.
- **AC-WAR-7 (doctrine loop):** Doctrine fitness incorporates operational/strategic outcomes (§4.2); a battle-winning, war-losing doctrine is selected against over generations; distinct clusters evolve distinct doctrines.
- **AC-WAR-8 (civilian cost):** Active theaters produce refugee flows, a war-economy market shift, and post-war reconstruction — all via existing agent/economy/architecture emergence, no new entity types.
- **AC-WAR-9 (LOD):** Far-from-camera wars resolve via statistical rollups logged to the event stream; near-camera wars get full operational+tactical resolution; switching LOD does not duplicate or lose war state.

---

## 8. Phased WBS + DAG (planner handoff — implementation NOT in this doc)

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|-----------|
| P1 Strategic detection | W-S1 | Tension measure + `WarObserved`/`WarEnded` hysteresis detector over polity clusters (`FR-CIV-WAR-001`) | polity-cluster + economy + ideology emergent state exists |
| P1 | W-S2 | Theater spatial-clustering + objective vectors (`FR-CIV-WAR-002`) | W-S1 |
| P1 | W-S3 | Mobilization-pressure field write-back into agent utility (`FR-CIV-WAR-003`) | W-S1 |
| P2 Operational | W-O1 | Supply/consumables + supply-line model over road network (`FR-CIV-WAR-010`) | W-S2; economy stock; road emergence |
| P2 | W-O2 | Objective+supply-gradient maneuver driver over existing `movement.rs` (`FR-CIV-WAR-011`) | W-S2, W-O1 |
| P2 | W-O3 | Attrition/cohesion + rout → operational withdrawal (`FR-CIV-WAR-012`) | W-O1 |
| P2 | W-O4 | Operational→tactical handoff + statistical-rollup LOD path (`FR-CIV-WAR-013`, `FR-CIV-WAR-...` LOD = AC-WAR-9) | W-O2, W-O3; existing `WarBridge` |
| P3 Tactical wiring | W-T1 | RTS command + direct-control intervention over shared tactical state (`FR-CIV-WAR-020`) | W-O4; existing tactics crate |
| P3 | W-T2 | Psyche-driven per-soldier modifiers + grievance/trauma write-back (`FR-CIV-WAR-021`) | W-T1; agent psyche |
| P3 | W-T3 | Cross-layer voxel-destruction coupling (LOS/cover/passability re-read) (`FR-CIV-WAR-022`) | W-T1; existing voxel CA |
| P4 Doctrine loop | W-D1 | Extend doctrine fitness with operational/strategic terms + memetic diffusion channel (`FR-CIV-WAR-030`) | W-O3, W-T2; existing `evolve_doctrine` |
| P5 Civilian | W-C1 | Refugee/displacement via safety-drive danger fields (`FR-CIV-WAR-040`) | W-O3, W-T3 |
| P5 | W-C2 | War-economy market shift + emergent conscription (`FR-CIV-WAR-041`) | W-S3; economy |
| P5 | W-C3 | Reconstruction + Legends/event-log war history (`FR-CIV-WAR-042`) | W-C1, W-C2; architecture emergence |

**DAG note:** P1→P2→P3 is the spine (strategic intent → operational maneuver → tactical resolution). P4 (doctrine) closes back from P2/P3 outcomes into P3 inputs. P5 (civilian) hangs off P2/P3 outputs and feeds back into P1 (displacement re-tension), closing the macro loop. No cycles within a phase; the cross-phase feedback is via the emergent substrate, not direct calls.

---

## 9. Cross-Project Reuse Opportunities (Phenotype org)
- **Tension/relations detector** (§1.3) is generic graph-relations labelling — candidate for a shared `phenotype-*` social-dynamics module if a second project needs emergent diplomacy.
- **LOD statistical-rollup pattern** (§1.6, AC-WAR-9) generalizes the `sim-misc.md` aggregation discipline — keep aligned with any shared LOD/agent-sim primitive.
- **Supply-graph over road network** (§2.3) reuses the emergent road/desire-path graph; do not fork a parallel pathing/graph layer (`crates/civ-traffic` / `pathfinding.rs` own routing).
- Tactical layer is **already** the shared substrate (`crates/tactics`); this spec is additive and must not duplicate it.
