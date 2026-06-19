# Civis Emergent Vehicles, Transport & Logistics — Design Spec

> **Status:** Design spec (docs-only, 2026-05-30). Owner: Design R&D Lead.
> **Stance:** PLANNER — specs, architecture, acceptance criteria, schemas, and brief pseudocode only.
> Contains **no implementation code**; it equips engineer/codex agents to build.
> **Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only
> physical/environmental/genomic laws are authored; everything else EMERGES. Vehicles are tools that
> civilizations *produce when tech + resource thresholds are met* (and the user may place); they are
> **never a fixed roster handed to a faction**.
> **Companions (do NOT duplicate):**
> - [`docs/research/sota-tech/roads-lanes.md`](../research/sota-tech/roads-lanes.md) — the Node→Segment→
>   Lane→LaneConnection routing model vehicles ride on. This spec consumes that graph; it does not redesign it.
> - [`docs/research/game-rnd.md`](../research/game-rnd.md) §1.3 — names **min-cost-flow trade routing over
>   the `civ-traffic` lane graph** and **comparative advantage** as the logistics drivers. This spec is the
>   concrete design of that "NEXT" item.
> - `crates/civ-traffic/src/lib.rs` + `lane.rs` — the existing `TrafficGraph`, `RoadKind` ladder,
>   `VehicleKind`/`Vehicle`, and Tier-2 `LaneGraph` this spec **extends, never replaces**.
> - Markets spec (`civ-economy`, tâtonnement/CDA) — owns *prices*; this spec owns *physical movement of
>   goods*. Coupling defined in §7, not duplicated.
> - Warfare logistics (FR-CIV-WAR) — owns *combat*; this spec owns *the supply line that feeds it* (§8).

---

## 0. Scope & the one-sentence thesis

**A vehicle is an emergent capability: when a settlement's accumulated tech traits + local material/energy
stock cross a kind's threshold, that settlement *can* build vehicles of that kind, which then move agents
and goods faster along the lane graph and raise per-trip carrying capacity.** Logistics is the layer that
*routes goods* (surplus→deficit) over that graph at minimum cost, using vehicles as the per-edge capacity +
speed modifier. Nothing here hardcodes *which* civ gets *which* vehicle; the substrate decides.

In scope: vehicle progression (foot→near-future), lane-graph coupling + speed model, freight/supply chains,
passenger movement, the min-cost-flow load/route optimizer, and the market + warfare couplings.
Out of scope (owned elsewhere, referenced only): road geometry/promotion (roads-lanes.md), price formation
(markets), combat resolution (warfare), crowd micro-movement (crowds.md).

---

## 1. Charter alignment (binding constraints)

| Charter rule | How this spec obeys it |
|---|---|
| Authored = only physical/material/energy/genomic law | We author the **physics of a vehicle** (mass, draft power, rolling resistance, water/rail/air medium, energy cost) and the **threshold rule**. We do NOT author a per-civ vehicle list or a tech tree of named units. |
| Tools/vehicles EMERGE at tech+resource thresholds | A `VehicleKind` becomes *buildable* in a locale when `TechProfile ⊇ kind.requires_traits` **and** local `MaterialStock ⊇ kind.requires_materials` **and** the energy budget covers `kind.build_cost`. No global "unlock button". |
| Civ-driven AND user-placeable | Both channels feed the **same** `TrafficGraph.vehicles` (mirroring the existing dual-authoring of roads via `InfraProvenance`). User placement still checks the locale capability gate — the user can place what the locale *could* build, not arbitrary anachronisms (configurable sandbox override flag). |
| Determinism NOT required | Vehicle build success, route choice tie-breaks, breakdowns may use real randomness/floats. Ordered containers (`BTreeMap`/sorted `Vec`) are kept only for save-stability, not replay-identity. |
| Markets/polities/architecture emerge | Logistics reads emergent surplus/deficit + emergent road graph; it never assumes a polity owns the roads. A min-cost-flow can cross "borders" that are themselves just emergent cluster overlaps. |

**Design test (every addition):** *can this emerge from a threshold over the substrate instead of being an
enum a civ is handed?* If yes, model the threshold. The `VehicleKind` enum is a **catalog of physically-
distinct movement archetypes** (like the materials DB), not a per-civ roster.

---

## 2. Vehicle progression (emergent, era-banded)

The existing crate ships only `Cart`/`Wagon`. This spec defines the **full archetype catalog** as a data
table. Each archetype is gated by *capability requirements*, not by a hardcoded era number alone — the
`unlock_era` field becomes a *derived hint for UI/legends*, while the real gate is `requires_traits +
requires_materials + medium availability`. Eras below are the *typical* emergence band, not a script.

| Archetype | Medium | Typical era band | Emergence gate (traits ∧ materials ∧ medium) | Capacity (goods-units) | Base speed mult | Notes |
|---|---|---|---|---|---|---|
| **Foot/porter** | Land (any walkable) | 0 (always) | none — baseline | 1 | 1.0 | Not a `Vehicle` row; the no-vehicle default. Sets the floor every other kind beats. |
| **Pack animal** | Land | 0–1 | `animal-domestication` ∧ livestock stock ∧ trail+ | 2 | 1.15 | Needs no road, only trail; first off-road freight multiplier. |
| **Hand cart** (`Cart`) | Land | 1 | `wheel` ∧ wood ∧ trail+ | 4 | 1.3 | Existing. |
| **Draft wagon** (`Wagon`) | Land | 2 | `wheel` ∧ `harness` ∧ livestock ∧ road+ | 12 | 1.7 | Existing. Road-bound for full speed. |
| **Riverboat / raft** | Water | 1–2 | `boatbuilding` ∧ wood ∧ navigable-water edge | 20 | 1.5 (downstream bonus) | Uses **water lanes** (§3.3); current adds/subtracts. |
| **Sailing ship** | Water | 3–4 | `sail` ∧ `rope` ∧ deep-water | 60 | 2.2 (wind-coupled) | Speed coupled to `crates/planet` wind; bulk inter-coast freight. |
| **Coach / stagecoach** | Land | 4 | `suspension` ∧ `road` ∧ livestock | 8 | 2.4 | Passenger-optimized (low capacity, high speed, comfort). |
| **Steam locomotive** | Rail | 5 | `steam` ∧ `iron-rail` ∧ rail edge | 240 | 4.0 | Requires **rail lane class** (§3.4); enormous bulk capacity. |
| **Steamship** | Water | 5 | `steam` ∧ `iron-hull` ∧ deep-water | 300 | 3.0 | Wind-independent bulk water freight. |
| **Truck** | Land | 6 | `internal-combustion` ∧ refined-fuel ∧ road+ | 30 | 5.0 | Door-to-door land freight; fuel-consuming. |
| **Cargo rail (diesel/electric)** | Rail | 6–7 | `diesel`\|`electrification` ∧ rail | 600 | 6.0 | Backbone bulk corridor. |
| **Cargo ship (container)** | Water | 7 | `containerization` ∧ steel-hull ∧ port | 2000 | 4.0 | Ocean bulk; needs **port node** capability. |
| **Aircraft (cargo/passenger)** | Air | 7–8 | `aviation` ∧ alloy ∧ refined-fuel ∧ airfield node | 40 / passengers | 12.0 | Uses **air lanes** (great-circle off-graph, §3.5); high speed, low capacity, high energy. |
| **Near-future (maglev / drone / autonomous)** | Rail/Air/Land | 8 | `superconductor`\|`autonomy` ∧ advanced-alloy ∧ energy-grid | varies | 8–15 | Open-ended top band; same data shape, new traits. |

**Schema (extends the existing `VehicleKind`):**

```text
VehicleArchetype {
    id: VehicleKind,                 // enum grown from the table above (catalog, not roster)
    medium: Medium { Land, Water, Rail, Air },
    requires_traits: TraitSet,       // emergent tech traits the locale must have accumulated
    requires_materials: MaterialSet, // from crates/laws materials DB
    build_cost: Energy,              // joules; charged against locale energy budget
    capacity: u32,                   // goods-units per trip
    base_speed_mult: f32,            // multiplies on top of lane speed (see §4)
    medium_coupling: MediumCoupling, // wind/current/grade/fuel terms (§4)
    upkeep: Upkeep,                  // per-tick fuel/feed + wear (§5)
    era_hint: u16,                   // UI/legends only; NOT the gate
}
```

> **Charter note:** `requires_traits` reference *emergent* tech traits (themselves measured clusters of
> accumulated knowledge in the agent/culture layer), so the "tech tree" is emergent, not authored. The
> archetype table is the *physics catalog*; whether a given world ever produces steam locomotives depends
> entirely on whether some lineage accumulates the `steam` + `iron-rail` traits.

**Acceptance criteria — progression (FR-CIV-VEHICLE-001..009):**
- **FR-CIV-VEHICLE-001** — An archetype is buildable in a locale **iff** all three gates (traits ∧ materials
  ∧ medium-available) pass; missing any one yields a *named* failure (`needs: steam; iron-rail`) per the
  loud-failure stance, never a silent no-op.
- **FR-CIV-VEHICLE-002** — Removing a required material from a locale's stock makes new builds of that kind
  fail while existing instances persist (capability is per-build, not retroactive).
- **FR-CIV-VEHICLE-003** — The catalog is additive/forward-only: new archetypes (near-future) slot in by
  adding a row + traits, with zero change to existing routing code.
- **FR-CIV-VEHICLE-004** — User placement honors the same capability gate by default; a sandbox flag
  (`allow_anachronism`) bypasses it and tags the instance `InfraProvenance::UserPlaced` + `anachronistic`.
- **FR-CIV-VEHICLE-005** — `era_hint` is never read by routing/build logic (assert: deleting it changes no
  sim outcome), proving emergence is gated by capability, not era.

---

## 3. How vehicles use the lane graph

Vehicles ride the **Tier-2 lane graph** (`LaneGraph` in `lane.rs`), not the scalar edge graph directly. The
key change this spec asks for is **lane-class ↔ vehicle-medium compatibility**: a lane advertises which
media it carries; a vehicle may only traverse lanes whose class admits its `Medium`.

### 3.1 Lane-class extension
`LaneClass` today = `{Trail, Road, Highway}` (all land). Extend with the media required by the catalog:

```text
LaneClass { Trail, Road, Highway, Water, Rail, Air }
```

- `lanes_for(segment, road)` (existing) keeps emitting land lanes from the `RoadKind` ladder.
- **Water lanes** are emitted from navigable-water edges (`crates/planet` hydrology marks them) — they are
  *not* on the `RoadKind` ladder; they exist wherever water is deep/wide enough, no promotion needed.
- **Rail lanes** are a *placed/grown overlay*: rail is a new `RoadKind`-parallel infra kind (or a
  `RailSegment` companion graph) requiring the `iron-rail`/`steel-rail` material to lay; promotion ladder
  `None → Tramway → MainLine`. Emergent rail grows along heavy repeated freight corridors (desire-path of
  *freight*, not feet); user may place it.
- **Air lanes** are not stamped infra; they are great-circle edges between **airfield/port capability nodes**
  (§3.5), generated on demand.

### 3.2 Medium ↔ lane compatibility (the routing gate)
`LaneConnection` routing (existing `route_lanes`) gains a **class filter**: a vehicle's router considers only
lanes whose `LaneClass.medium() == vehicle.medium` (with `Land` covering Trail/Road/Highway). This is the
TM:PE "policy annotation on a lane-connection" pattern (roads-lanes.md §2): we add `allowed_media` to the
connection and filter, rather than fork the router.

### 3.3 Water transport
Water lanes carry a **flow vector** (from hydrology). Downstream travel adds speed, upstream subtracts;
boats below a power threshold (raft) cannot go upstream at all. Bridges/fords are land lanes crossing water
lanes at a shared node — the node's `LaneConnection` set decides whether a land vehicle can cross.

### 3.4 Rail transport
Rail is **capacity-dominant, branch-sparse**: high `capacity`, very high `base_speed_mult`, but only where
rail lanes exist. The min-cost-flow (§6) naturally prefers rail for bulk long-haul because per-unit cost
collapses at high capacity — emergent corridor formation, no scripted "build rail here".

### 3.5 Capability nodes (ports / airfields / depots)
Some media require a **transfer node** to enter/leave: ships need a **port** node (water↔land transfer),
aircraft need an **airfield**. These are emergent structures (architecture layer) tagged with a capability
in the lane graph. A node without the capability simply offers no land↔water/air `LaneConnection`, so the
router can't transfer there. This is how multimodal supply chains arise (§6.3) without authoring them.

**Acceptance criteria — lane coupling (FR-CIV-VEHICLE-010..019):**
- **FR-CIV-VEHICLE-010** — A vehicle routes only over lanes admitting its medium; a land truck given a
  water-only path returns *no route* (loud, not a silent land fallback).
- **FR-CIV-VEHICLE-011** — Water-lane traversal speed = `base × current_term(direction)`; a raft upstream of
  its power threshold yields no traversal.
- **FR-CIV-VEHICLE-012** — Rail lanes are a forward-only overlay; absence of rail leaves all existing
  land/water routing unchanged (regression guard mirrors `scalar_speed_graph_stays_intact`).
- **FR-CIV-VEHICLE-013** — Multimodal transfer (e.g. truck→ship) is possible **iff** a shared node carries
  the port capability; otherwise the route does not exist.
- **FR-CIV-VEHICLE-014** — Adding a `LaneClass` variant does not change land-only outcomes (additive proof).

---

## 4. Speed coupling model

A trip's effective speed on a lane is a **product of independent multipliers**, so each layer (road, vehicle,
medium, load, congestion) stays orthogonal and emergent:

```text
effective_speed(lane, vehicle, load, t)
  = base_walk_speed
  × lane.speed_mult                      // existing RoadKind/RoadClass multiplier
  × vehicle.base_speed_mult              // archetype (§2 table)
  × medium_coupling(vehicle, lane, t)    // wind (sail), current (boat), grade (rail/road), 1.0 default
  × load_factor(load / vehicle.capacity) // laden vehicles slow; ∈ (load_min, 1.0]
  × congestion(lane, t)                  // ∈ (0,1]; from lane occupancy (roads-lanes.md lookahead)
```

- **`lane.speed_mult`** reuses the existing `RoadKind::speed_multiplier` / `speed_for_lane` — no new road
  speed data; this spec only *multiplies into* it (keeps the "thin shim over per-lane cost" invariant from
  roads-lanes.md §2).
- **`medium_coupling`** is where physics enters: sail ships read `crates/planet` wind, boats read hydrology
  current, rail/road read terrain grade (steep grade penalizes wagons/trucks heavily, rail moderately).
- **`load_factor`** ties capacity to speed so the optimizer (§6) faces a real laden-vs-empty trade-off.
- **`congestion`** comes from lane occupancy with multi-node **lookahead** (explicitly inheriting the C:S2
  anti-bug from roads-lanes.md §3 — reserve target lane N segments early).

**Acceptance criteria — speed (FR-CIV-VEHICLE-020..024):**
- **FR-CIV-VEHICLE-020** — With all couplings at 1.0 and no load, `effective_speed` reduces exactly to
  `base_walk × lane.speed_mult × vehicle.base_speed_mult` (orthogonality proof).
- **FR-CIV-VEHICLE-021** — A fully-laden vehicle is strictly slower than the same vehicle empty on the same
  lane (`load_factor` monotone decreasing).
- **FR-CIV-VEHICLE-022** — A sailing ship's speed varies with planet wind direction; downwind > crosswind >
  upwind (medium coupling wired to `crates/planet`).
- **FR-CIV-VEHICLE-023** — Congestion can only slow, never speed up (`congestion ∈ (0,1]`).
- **FR-CIV-VEHICLE-024** — The existing scalar `speed_multiplier_at` life-sim path is unchanged when no
  vehicle is involved (foot baseline preserved).

---

## 5. Vehicle lifecycle, upkeep & wear

- **Build:** a locale with capability (§2) spends `build_cost` energy + materials to mint a `Vehicle` row.
- **Upkeep per tick:** fuel/feed draw (animals eat, trucks burn fuel from `crates/laws` energy model);
  failure to supply upkeep → vehicle idles/abandons (loud, not silent) — a starving wagon's livestock dies,
  a fuel-starved truck stops where it stands and becomes a routing obstacle.
- **Wear:** each trip accumulates wear scaled by load + grade; past a threshold the vehicle breaks down
  (needs repair materials) — feeds back as emergent demand for repair goods.
- **Provenance:** `Vehicle.provenance` (existing) distinguishes civ-built vs user-placed for renderer/legends;
  the economy treats both identically (mirrors road provenance).

**AC (FR-CIV-VEHICLE-030..033):** build charges energy+materials (030); unsupplied upkeep idles the vehicle
with a named cause (031); wear accumulates per laden trip and breakdown blocks routing until repaired (032);
both provenances are economically identical (033).

---

## 6. Logistics: the load/route optimizer (min-cost-flow)

This is the heart of the spec and the concrete realization of game-rnd §1.3's "min-cost-flow trade routing
over the `civ-traffic` lane graph."

### 6.1 The flow network
Build a flow network *derived from* the lane graph each logistics tick (or on dirty-region change):

```text
Sources  = locales with surplus of good g           (supply = surplus quantity)
Sinks    = locales with deficit of good g            (demand = deficit quantity)
Arcs     = lane-graph edges admitting some vehicle, with:
             capacity(arc) = Σ (available vehicle capacity routed over it)   // throughput/tick
             cost(arc)     = lane_length / effective_speed(arc, best_vehicle) // time-cost per unit
                           + energy_cost(arc, vehicle)                        // joules per unit (laws)
                           + toll/penalty(arc)                                // emergent (polity/road class)
Transfer arcs at capability nodes (§3.5) carry a handling cost (load/unload).
```

Solve **min-cost-flow** per good (the `pathfinding` crate provides it; for 20mi² scale, precompute long-haul
with `fast_paths` contraction hierarchies per roads-lanes.md). Flow value = how much of good `g` actually
moves surplus→deficit this tick; the solution *assigns vehicles to lanes* and *quantities to carry*.

> **Comparative advantage (charter-named driver):** a locale becomes a *source* of `g` not by decree but
> because its local production cost of `g` (from `crates/laws` energy/material costs) is below the network's
> delivered price — surplus emerges, the flow then exports it. The optimizer never picks *what* to produce;
> it only moves what surplus/deficit already exist.

### 6.2 Load assignment (vehicles ↔ flow)
Given the flow on each arc, assign concrete vehicle trips:
- Greedily fill highest-capacity available vehicle on the arc first (rail/ship before trucks) — emerges as
  "bulk goes by rail/ship, last-mile by truck/cart" with no rule saying so.
- Remaining flow below a vehicle's capacity → smaller vehicles or pack animals (last-mile).
- Unassignable flow (no vehicle/medium) is reported as an **unmet demand** signal (loud) — feeds price up
  in the market (§7) and demand for new vehicle builds (§2).

### 6.3 Multimodal supply chains
Because transfer arcs only exist at capability nodes, the min-cost-flow *naturally* produces chains like
`farm —cart→ river port —barge→ coastal port —ship→ city`, choosing the cheapest medium per leg. No supply
chain is authored; it is the optimal flow over the emergent multimodal graph. Intermediate stockpile nodes
(warehouses/depots — architecture layer) appear as transshipment nodes with storage capacity.

### 6.4 Tick cadence & LOD
- Near-camera / active regions: full per-good min-cost-flow each logistics tick.
- Far regions: statistical/aggregated flow (LOD-tiered, per charter scale target) — solve coarsely over a
  contracted graph, no per-vehicle assignment.
- Dirty-region driven: only re-solve goods whose surplus/deficit or graph changed (mirrors voxel dirty-queue
  pattern; avoids the per-frame full-network re-solve cost that sank C:S2).

**Acceptance criteria — logistics (FR-CIV-VEHICLE-040..049):**
- **FR-CIV-VEHICLE-040** — Goods flow from surplus to deficit along the min-cost path; with two routes the
  cheaper (faster × lower-energy) carries more flow.
- **FR-CIV-VEHICLE-041** — Per-good min-cost-flow respects arc capacity = summed assigned vehicle capacity;
  excess demand over capacity surfaces as unmet demand, not silent loss.
- **FR-CIV-VEHICLE-042** — Bulk long-haul prefers high-capacity media (rail/ship) emergently (no explicit
  mode rule), verifiable by removing rail and observing cost/flow shift to road.
- **FR-CIV-VEHICLE-043** — Multimodal chains form only through capability nodes; deleting a port reroutes or
  raises cost, never teleports goods across a missing transfer.
- **FR-CIV-VEHICLE-044** — Comparative advantage: a locale with lower local production cost of `g` becomes a
  net exporter of `g` once delivered cost < local price elsewhere (driver is cost, not assignment).
- **FR-CIV-VEHICLE-045** — Unassignable flow (no compatible vehicle) is reported as a named unmet-demand
  signal consumed by §7 and §2.
- **FR-CIV-VEHICLE-046** — Far-region LOD solve produces aggregate flow without per-vehicle assignment and
  does not change near-region results (LOD isolation).
- **FR-CIV-VEHICLE-047** — Re-solve is dirty-region scoped: an unchanged good/region is not recomputed
  (perf-correctness guard).

---

## 7. Passenger movement

Passengers (agents traveling for needs/work/migration/trade, not goods) share the **same lane graph + speed
model** but route as a **per-agent shortest-effective-time** path (reuse the existing `route_lanes` over the
medium-filtered lane graph + §4 speed), not min-cost-flow. Differences from freight:
- Passenger vehicles (coach, passenger rail, aircraft, ferry) optimize **time + comfort**, low capacity.
- An agent boards a vehicle when one is available on its path and the time saved exceeds boarding overhead
  (an emergent utility decision in the agent backbone — crowds.md `big-brain`), else walks. No scheduling
  authority; ride-sharing/commuting patterns emerge from co-located agents choosing the same vehicle.
- Mass movement (migration, refugee flows, army marches) reuses the *freight* min-cost-flow with agents as
  the "good" when moving populations in bulk (e.g. evacuation), bridging to §8.

**AC (FR-CIV-VEHICLE-050..053):** agents path over the medium-filtered lane graph by effective time (050);
an agent boards iff time-saved > boarding overhead (emergent, not scheduled) (051); passenger vehicles favor
speed/comfort over capacity (052); bulk population moves reuse the freight flow with population as cargo (053).

---

## 8. Coupling to markets and warfare logistics

### 8.1 Markets (`civ-economy`)
- **Delivered price = local price + transport cost.** The min-cost-flow's arc cost (§6.1) is the transport
  term; the market's tâtonnement/CDA (game-rnd §1.3) sets the *local* price. Together they yield the
  *delivered* price that drives where surplus flows — closing the comparative-advantage loop.
- **Unmet demand (§6.2) raises the local price** (excess demand term in tâtonnement), which raises the
  delivered price ceiling, which makes longer/more-expensive routes (or new vehicle builds) worthwhile —
  emergent market-driven infrastructure growth.
- **Vehicle build cost** (energy+materials) is itself priced by the market; a steel shortage makes rail
  expensive, biasing flow back to road/ship — no authored balance, pure cost propagation.
- **Separation of concerns:** markets own price; logistics owns movement; they couple *only* through the
  transport-cost term and the unmet-demand signal. Neither duplicates the other.

### 8.2 Warfare logistics (FR-CIV-WAR)
- **Supply lines are min-cost-flow with the army as a moving sink.** A force in the field is a deficit node
  for food/fuel/munitions; the same optimizer (§6) routes supply to it over the lane graph. Cut the roads
  (destroyed bridge, blockaded port — §3.5) and the flow's cost spikes or the route vanishes → the army
  goes unsupplied (loud unmet-demand) → combat effectiveness degrades. **Attrition emerges from broken
  logistics, not a scripted supply stat.**
- **Strategic mobility** uses §4 speed: rail/truck/ship let forces redeploy fast; pre-rail armies crawl at
  wagon/foot speed — emergent operational tempo by era.
- **Interdiction** is a first-class emergent tactic: targeting capability nodes (ports/depots/rail junctions)
  or high-flow arcs is how an attacker collapses an enemy's supply min-cost-flow. The warfare spec owns the
  *decision* to interdict; this spec owns the *flow consequence*.
- **Bulk troop movement** reuses §7's population-as-cargo bridge.

**AC (FR-CIV-VEHICLE-060..064):** delivered price = local price + transport cost (060); unmet demand raises
local price and can justify new routes/vehicles (061); a field army is a moving deficit node supplied by the
same min-cost-flow (062); cutting roads/nodes degrades army supply via the flow, not a scripted stat (063);
interdicting a capability node measurably reduces enemy delivered supply (064).

---

## 9. Crate boundaries & reuse (no duplication)

| Concern | Owning crate | This spec adds |
|---|---|---|
| Road promotion + scalar graph | `civ-traffic` (`lib.rs`) | nothing — consumed as-is |
| Lane graph + routing | `civ-traffic` (`lane.rs`) | `LaneClass` media variants + medium filter on `LaneConnection`; rail/water/air lane emitters |
| Vehicle catalog + instances | `civ-traffic` | full `VehicleArchetype` table + capability gate (extends `VehicleKind`/`Vehicle`) |
| Min-cost-flow solver | **`pathfinding`** crate (+`fast_paths`) | the flow-network *builder* + load assignment over the lane graph |
| Prices / surplus / deficit | `civ-economy` | reads surplus/deficit; exposes transport-cost + unmet-demand hooks |
| Hydrology / wind / grade | `crates/planet` | medium-coupling reads these; authored physics only |
| Energy / material costs | `crates/laws` | build cost, upkeep, per-unit energy cost reads these |
| Combat | warfare crate (FR-CIV-WAR) | army-as-sink + interdiction *flow consequence* only |
| History / legends | `crates/watch` + legends engine | emit vehicle-build / trade-route / supply-cut events (no new event bus) |

**Cross-project reuse opportunity (Phenotype):** the **lane-media filter + min-cost-flow-over-a-lane-graph**
logistics layer is generic enough to extract as a shared `phenotype-logistics` module reusable by any
Phenotype sim with a transport graph (e.g. WorldSphereMod). Flag for the user before extraction; build it
inside `civ-traffic` first, extract once a second consumer appears (abstraction-at-2-uses rule).

---

## 10. Phased WBS (DAG)

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| P1 Catalog | V1 | `VehicleArchetype` table + capability gate (traits∧materials∧medium) extending `VehicleKind` | — |
| P1 Catalog | V2 | Vehicle lifecycle: build cost, upkeep, wear, breakdown | V1 |
| P2 Lanes | L1 | `LaneClass` media variants (Water/Rail/Air) + medium filter on routing | — |
| P2 Lanes | L2 | Water-lane emitter from hydrology; rail overlay graph; air/great-circle | L1 |
| P2 Lanes | L3 | Capability nodes (port/airfield/depot) + transfer arcs | L1, L2 |
| P3 Speed | S1 | Product speed model (load_factor, medium_coupling, congestion lookahead) | V1, L1 |
| P4 Logistics | F1 | Flow-network builder from lane graph (sources/sinks/arcs/costs) | L3, S1 |
| P4 Logistics | F2 | Min-cost-flow per good via `pathfinding` + load assignment | F1 |
| P4 Logistics | F3 | Multimodal chains + LOD/dirty-region tick cadence | F2, L3 |
| P5 Passengers | P1 | Per-agent passenger routing + emergent boarding | S1 |
| P6 Coupling | C1 | Market coupling (delivered price, unmet-demand → price) | F2 |
| P6 Coupling | C2 | Warfare coupling (army-as-sink, interdiction consequence) | F2, C1 |
| P6 Coupling | C3 | Legends/watch events for builds, routes, supply cuts | V2, F2 |

**DAG:** V1→V2; L1→{L2,L3}; {V1,L1}→S1; {L3,S1}→F1→F2→F3; S1→P1; F2→C1→C2; {V2,F2}→C3. No cycles.

---

## 11. Open design questions (for follow-up specs, not blockers)

1. **Rail as new `RoadKind` vs companion `RailSegment` graph** — companion graph keeps the foot/road ladder
   clean but duplicates promotion logic; lean companion-graph, revisit if duplication >5%.
2. **Air-lane congestion** — do great-circle air lanes need congestion at all at 20mi² scale, or only
   airfield-node throughput limits? Likely node-only.
3. **Storage/warehouse nodes** — owned by architecture layer; this spec assumes transshipment storage exists
   as a node capability. Needs a thin contract with the architecture spec.
4. **Per-good vs multi-commodity flow** — start per-good (simpler, parallelizable); revisit multi-commodity
   min-cost-flow if shared-capacity contention (one truck, many goods) proves material.

---

## 12. Requirements traceability summary

New FR family: **FR-CIV-VEHICLE-***. Backfills the existing `FR-CIV-INFRA-040` (vehicle tech-gate) and
`crates/civ-traffic` `VehicleKind`/`Vehicle` into the fuller catalog model. Couples to `FR-CIV-WAR-*`
(warfare logistics) and the markets FR family (delivered price). To be registered in
[`docs/reference/FR_TRACKER.md`](../reference/FR_TRACKER.md) under a new *Vehicles, Transport & Logistics*
section and linked to AgilePlus Epic *Emergent Logistics* with Story-level breakdown matching §10's WBS.
