# Disaster Emergence — From Physics Fields to Civilizational Response

> **Status:** Design (Planner, 2026-06-23). No implementation code in this PR.
> **Branch:** `research/disaster-emergence-design`
> **Scope:** A coherent design for natural disasters (earthquakes, floods, storms, volcanism, plague) that **emerge** from the physics substrate and **feed back** into civilization emergence: migration, collapse, myth-formation.
> **Doctrinal anchors (do not relitigate):**
> - [`docs/design/PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md) — six continuous fields T, M, E, F, P, B; emergent layers couple *only* through the substrate.
> - [`docs/design/PHYSICS_INTEGRATION_PLAN.md`](PHYSICS_INTEGRATION_PLAN.md) — 40-cell coupling matrix, 14 silo'd call-sites, phased sprint plan.
> - [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) — emergence-default principle; outcomes are measured patterns, not scripted enums.
> - [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — hardcode only physics + environment + genome; everything else emerges.
> - [`EMERGENCE_COUPLING_AUDIT.txt`](../../EMERGENCE_COUPLING_AUDIT.txt) — current phase DAG; `phase_disasters` at engine.rs:1275 currently fires only on `temp_c_fp + precip_mm_fp` (wildfire) and `tide + latitude` (quake), with no feedback to substrate fields.

---

## 1. Thesis — A disaster is a phase transition in a field gradient

A flood is not a flag. A plague is not a damage event. They are **phase transitions** in the substrate's continuous fields, arising when a gradient crosses a physical threshold:

- **Earthquake** — a discontinuity in the **tectonic-stress** field (a derivative of `F`, the material flux field) crosses a lithostatic-yield threshold at a fault zone.
- **Flood** — the **moisture (M) gradient** at a coast or river-basin exceeds a drainage-carrying capacity, computed as the local maximum of `M` over a small watershed.
- **Storm** — a coupled instability in the **T–M–E** fields where the local gradient `‖∇T‖ · ‖∇M‖` exceeds a latent-heat-driven threshold and the storm-intensity field (already in `WeatherCell.storm_intensity_fp`) crosses a critical value.
- **Volcanism** — sustained **|E|** injection at a subduction margin (where the lithosphere gradient ‖∇F‖ is large and `E` builds) crosses a magma-chamber venting threshold.
- **Plague** — a disease vector's reproduction rate, modulated by **M (humidity)**, **T (temperature)**, and the **P (population pressure)** gradient, exceeds an immune-naïve contact threshold; the substrate is the *agent's body*, not the terrain.

The simulation already owns the fields. The design's only job is to make the disaster *threshold* a function of the field state, not a hardcoded spawn, and let the resulting damage write back to the substrate (M, F, P, B) so that downstream emergence phases read the new field state, not a notification.

```
[Field state] ── threshold check ──► [disaster onset] ── damage write ──► [new field state]
                                              │
                                              └───► [RawSimEvent onto watch bus]
                                                          │
                                                          ├──► legends graph
                                                          ├──► chronicle
                                                          ├──► emergence feed
                                                          └──► civ-ai narrator
```

This is the substrate-first style the substrate doc §2.3 calls "downward causation": agents do not subscribe to disasters, they read the *changed fields* and act.

---

## 2. The five disaster kinds — physics triggers

The five kinds map onto five *physical* trigger conditions. Each is computed from the substrate (or, in the current codebase, the substrate's existing scalar projections: `WeatherCell`, `Climate`, voxel material counts) and produces a `DisasterEvent` with onset site + radius + magnitude. There are no hand-tuned probability tables.

### 2.1 Earthquake — `tectonic_stress > lithostatic_yield(at plate boundary)`

| Field read | Source today | Trigger |
|------------|--------------|---------|
| `tectonic_stress` (derived from `‖∇F‖` at depth) | `crates/planet/src/geology.rs:1` (region latitudes; future `GeologyMap.plate_boundary` field, see §6.1) | `‖∇F‖_p > YIELD` AND `p` is on a plate boundary |
| Lithostatic yield | `material.density · g · depth` (CA density, not hand-tuned) | fixed-point: `YIELD_FP = 2_500_000` (2.5 MPa·m scale) |

Magnitude `M_q` follows a Gutenberg-Richter-like scaling clipped to the local stress integral:

```
M_q = clamp( (tectonic_stress - YIELD) / YIELD, 0.05, 1.0 )   // normalized 0..1
```

The current `phase_disasters` (engine.rs:1275–1283) **already** has a quake onset at `tide_offset.abs() ≥ 0.9` AND `|latitude_fp| ≥ 40_000`. The design promotes that scalar trigger to a substrate-derived one: replace `tide_offset` with a moving average of `‖∇F‖` at the cell and treat the latitude floor as a *proxy* for "near a plate boundary" until `GeologyMap` carries explicit plate geometry. This is the substrate doc S5 closure.

**Damage write** (downward causation):
- `set(F, p, -debris_mass, …)` — voxel rubble, written as `STONE`/`GRAVEL` over radius (the current `apply_disaster` (disasters.rs:172-183) already does this; preserve as is).
- `set(P, p, -casualties, …)` — agents within radius (per the existing `hit_agents` call at disasters.rs:177-181).
- **New**: `set(M, p, +groundwater_pulse, …)` — quakes open fissures; moisture intrudes into surface cells for 1–2 ticks, raising local M and increasing the chance of an *aftershock flood* (compositional coupling).
- **New**: `set(E, p, +seismic_heat, …)` — frictional heating at the fault; small but persistent for the quake's duration. This is the hook for thermal-driven chemistry in the CA (e.g. `STONE → STONE+HEAT` percolation).

### 2.2 Flood — `‖∇M‖ over watershed > drainage_capacity`

| Field read | Source today | Trigger |
|------------|--------------|---------|
| `‖∇M‖` (moisture gradient) | implicit in `WeatherCell.precip_mm_fp` (today's scalar; future substrate `M`) | `sum_{p ∈ watershed(p*)} (precip_mm_fp) > DRAINAGE_FP` |
| Watershed mask | future substrate field `W: Grid<u8>` (watershed id per cell) | `DRAINAGE_FP(p*) = 3_000 + 200·(terrain_slope(p*))` |
| Topography | existing `GeologyMap` (`biome_at_normalized` proxy; future `elevation: Grid<f32>`) | flat cells saturate faster |

The current `phase_disasters` does **not** have a flood onset. Today's `Flood` kind exists in `DisasterKind` (disasters.rs:23) but is only triggerable via `invoke_divine_disaster` (player input). The design adds an *emergent* flood path:

```
for cell in weather_grid:
    watershed_sum = sum precip_mm_fp over cells in same W watershed
    if watershed_sum > DRAINAGE_FP and cell.precip_mm_fp >= FLOOD_PRECIP_FP:
        onset_sites.push(cell)
```

`FLOOD_PRECIP_FP` = 1_200 (1.2 mm fixed-point per cell per tick, sustained over a watershed). This is a *physical* threshold, not a probability table.

**Damage write**:
- `set(M, p, +flood_mass, …)` over a larger radius (the existing disasters.rs:161-171 writes `WATER` voxels; that is the voxel projection of the substrate `M` write).
- `set(P, p, -casualties, …)` (existing hit_agents at disasters.rs:165-170).
- `set(B, p, -topsoil_loss, …)` — flood water strips topsoil biomass; affects `phase_life` next tick.
- **New**: `set(F, p, +silt_deposition, …)` — downstream of the flood, silt material flux in the floodplain (a delayed positive write; silt is the substrate's M + F signal, future M write).

### 2.3 Storm — `‖∇T‖·‖∇M‖ > latent_heat_threshold AND storm_intensity_fp > critical`

| Field read | Source today | Trigger |
|------------|--------------|---------|
| `‖∇T‖·‖∇M‖` | `WeatherCell.temp_c_fp` neighbours; future substrate T + M | `gradient_product > LHT_FP` |
| `storm_intensity_fp` | `WeatherCell.storm_intensity_fp` (already computed at weather.rs:153) | `> 1_500` (existing) |
| Latitude | `WeatherCell.latitude_fp` | `|lat| ≤ 60_000` (mid-latitude storms; the existing weather-kind threshold is unconditional) |

The current `WeatherCell.storm_intensity_fp` is already a substrate-derived scalar (`weather.rs:152-154`): `(day_heat_fp.abs() + pressure_wave_fp.abs())/2 + moisture - |temp|/4`. The design's contribution is the *coupling*: storm onset is gated on `‖∇T‖·‖∇M‖`, so a storm in a homogeneous warm cell does not form (no moisture contrast), but a cold-warm front in a wet cell does. This is the storm-front trigger the substrate doc §3.1 expects from "climate × T,M,E".

**Damage write**:
- `set(M, p, +heavy_precip, …)` — voxel WATER/ICE over a wide radius (the existing `Storm` arm at disasters.rs:196-206, kept).
- `set(E, p, +lightning_heat, …)` — discrete high-E writes at strike sites (the "E spikes" the lightning does to the field).
- `set(P, p, -casualties, …)` (existing hit_agents at disasters.rs:202-205).
- **New**: `set(F, p, +debris_flux, …)` — wind-driven material flux (a transient write the fluid CA can capture; reads at the next `phase_voxel` pass).

### 2.4 Volcanism — `|E|_deep + ‖∇F‖_vertical > vent_threshold AND cell on subduction margin`

| Field read | Source today | Trigger |
|------------|--------------|---------|
| `‖∇F‖_vertical` | future substrate `F` at depth; today no depth field | `> 5_000` (vertical material flux gradient) |
| `|E|_deep` | future substrate `E` at depth | `> 8_000_000` (deep-earth energy) |
| Subduction margin | future `GeologyMap.subduction_zones: Vec<(RegionId, RegionId)>` | plate adjacency, not latitude |

The current `DisasterKind` enum (disasters.rs:18-32) has no `Volcanism` variant; it has `Meteor`, `Flood`, `Quake`, `Wildfire`, `Storm`, `Plague`. The design **adds** `Volcanism`. It is the disaster most clearly *only* explainable by substrate state: there is no scalar trigger that captures "is this cell on a subduction margin with a deep energy buildup?" The substrate has to.

**Damage write**:
- `set(E, p, +lava_enthalpy, …)` — large positive E write (LAVA voxel, the existing `Meteor` arm at disasters.rs:140-159 sets the precedent).
- `set(F, p, +ash_flux, …)` — vertical ash material flux; future CA reaction `ASH + M → MUD`.
- `set(M, p, -evaporation_loss, …)` — boiling-off of surface moisture near the vent.
- `set(T, p, -surface_temp, …)` — long-term cooling over the ejecta blanket (volcanic winter, see §3.4).
- `set(P, p, -casualties, …)` (radius = current `Meteor` radius).

### 2.5 Plague — `R0(T, M) · contact_rate(P) > herd_immunity AND susceptible_fraction > threshold`

| Field read | Source today | Trigger |
|------------|--------------|---------|
| `T` (local temperature) | future substrate T; today `WeatherCell.temp_c_fp` | `15_000 ≤ T ≤ 30_000` (FP °C, i.e. 15–30 °C) |
| `M` (local humidity) | future substrate M; today `WeatherCell.precip_mm_fp` | `M ≥ 600` (humid enough for vector survival) |
| `P` (population pressure gradient) | future substrate P; today ECS `ClusterMember` count | `‖∇P‖ > 0` (agents in contact) |
| Herd immunity | tracked per-`DnaClass` in `crates/genetics/src/sentience.rs` | fraction of population with prior exposure |

Plague is the only disaster that is *not* primarily a terrain effect. The damage is on the substrate field `P` (population pressure). The current `Plague` arm (disasters.rs:208-215) hits agents in `radius*2` with low damage (0.05/0.10/0.18/0.06). The design reframes: the trigger is `R0·contact_rate > 1`, where:

```
R0(T, M) = R0_base · bell(T) · bell(M)            // bell = exp(-((x-μ)/σ)²)
contact_rate = |∇P|_local · density_coefficient   // agents per cell × movement
susceptible_fraction = 1 - prior_exposure[DNA_class]
trigger if R0 * contact_rate > 1 AND susceptible_fraction > 0.3
```

The agent's "prior exposure" is a *new* ECS component (`ImmuneMemory { dna_class_hash: u64, exposure_epoch: u64 }`) introduced in `civ-genetics`. After a plague event, survivors get `ImmuneMemory` and the local `R0` for that DNA class drops. This is the substrate doc §2.3 "downward causation" applied to epidemiology: the disease doesn't *say* who is immune, it writes to the substrate and the substrate's `P` field reflects the surviving immune population.

**Damage write**:
- `set(P, p, -mortality, …)` — population pressure drops; this is the field projection of the existing `hit_agents` call (disasters.rs:209-214), kept.
- `set(B, p, -0, …)` — plague does not directly affect biomass (no debris to decompose), but the *next* tick's `phase_life` reads `P` and may run a regrowth in vacant cells.
- **New**: `set(legend_severity, …)` — plagues promote chronicling because they are slow and spatially diffuse (see §3.5).

### 2.6 Summary table — five triggers, one substrate

| Disaster | Primary field | Trigger expression | Damage fields written |
|----------|---------------|--------------------|------------------------|
| Earthquake | `‖∇F‖` at depth | `‖∇F‖ > YIELD ∧ on_boundary` | F, P, M (fissure), E (friction) |
| Flood | `Σ M` over watershed | `watershed_precip > DRAINAGE` | M, P, B (topsoil), F (silt delayed) |
| Storm | `‖∇T‖·‖∇M‖` + intensity | `gradient_product > LHT ∧ intensity > 1_500` | M, E (lightning), P, F (debris) |
| Volcanism | `‖∇F‖_vert + |E|_deep` | `> vent_threshold ∧ on_subduction` | E, F (ash), M (evap), T (cooling), P |
| Plague | `T, M, ‖∇P‖, susceptible_fraction` | `R0·contact > 1 ∧ sus > 0.3` | P, legend_severity (via legends) |

All five *read* substrate fields and *write* substrate fields. None call an inter-layer API. This is the substrate contract preserved.

---

## 3. Downstream coupling — how disasters feed civilization emergence

The substrate doc §3.1 enumerates 12 emergent layers. Disasters touch **all of them** in measurable ways, and the design specifies the channel for each. The pattern: disaster → field write → field gradient → emergent layer reads gradient → emergent behavior. There is no `disasters.bump(religion)`.

### 3.1 Migration (faction × P, B)

The primary migration mechanism today is the `phase_life` flow (engine.rs:2261+). With disasters, the migration pressure is now a function of *field-derived* hostility, not a hand-fed `group_size`:

- `‖∇M‖_p` rises near a flood onset → agents in high-Moisture cells read the gradient and write `set(P, p, -outflow, FactionLayer)` (substrate doc §3.2 L3×P write).
- `‖∇T‖_p` falls in a volcanic-winter cell → agents in low-T cells read the cold gradient and write `set(P, p, -outflow, FactionLayer)`.
- Migration is then read by the next layer: `phase_emergence` sees the new `P` distribution and updates `cluster_cultures` (emergence.rs:191+) — culture follows the migrants.

The carrier is `set(P, p, …)`; the channel is the substrate. The faction's territory choice (the 14-dim ideology at `cluster_with_gradients` per substrate doc §2.3 L3) now includes the disaster's gradient as a 5th axis alongside T, M, B, P.

### 3.2 Collapse (economy × B, T, P)

Disaster-induced collapse is the failure of the `B` field to sustain `P` (carrying capacity). The mechanism:

- Earthquake writes `set(F, p, -rubble)` and `set(B, p, -0)`. Carrying capacity drops because `K(p) = f(M, T)` is now bounded by the reduced `B`.
- The `phase_economy` (engine.rs:2553+) reads the new `K` via `carrying_capacity()` (audit §2.0) and the *price* of food spikes.
- A spike in `market_state.prices["food"]` raises `unrest` (audit §4.0 UNREST HUB: `market_food_price → unrest_delta`).
- A sustained unrest spike writes `set(P, p, -starvation, LifeLayer)` (L1×P write, substrate doc §2.3).
- Collapse = `P → 0` over multiple ticks, not a single death event.

This is the substrate doc M2 closure: `cluster_stocks → economy` was a dead silo; with disaster writes, the new `B` value at affected cells is the *signal* the economy reads.

### 3.3 Migration-Collapse coupling (compositional)

A flood → migration → collapse in the *receiving* polity is the canonical compositional disaster scenario. The coupling:

1. **t = 0**: flood onset at watershed `W_A`. `set(M, p, +flood_mass, …)` over `W_A`.
2. **t + 1**: agents in `W_A` read `‖∇M‖` and write `set(P, p, -outflow, FactionLayer)`. `P` drops in `W_A`, rises in neighboring `W_B`.
3. **t + 2**: `W_B` now has more agents than its `K(p) = f(M, T, B)` can support (the new `P` is locally a population spike). `carrying_capacity` for `W_B` is exceeded.
4. **t + 3**: `phase_economy` reads `P > K` in `W_B`, raises food price, raises unrest. `phase_unrest` (engine.rs:1806+) raises the global `unrest` scalar.
5. **t + 5–10**: `unrest → inequality_unrest → dispossessed_permille → garrison_drain`. If `cohesion` is low, the polity's `temple_level` cannot stabilize; if `belief` is low, the cohesion_damp is absent. The receiver collapses; the sender's `P` does *not* recover because the flood's `M` write persists.

This is edge-of-chaos: each step is a substrate read + write, the lag is the 1-tick substrate evolve (substrate doc §4.1), and the system can recover or amplify depending on initial parameters.

### 3.4 Myth-formation (religion × T, M, B, E, P)

This is the deepest coupling. The current `trigger_disaster` already does the simplest myth-formation: `add_belief(DISASTER_FAITH_GAIN = 50)` (disasters.rs:39-40). The design extends this through the substrate:

- `hardship = ‖∇T‖_p + ‖∇B‖_p` (the substrate doc §4.4 example) replaces the hand-fed `hardship` argument to `emerge_belief` (religion.rs:35-67). A post-volcanic-winter cell has high `‖∇T‖`; a post-flood cell has high `‖∇B‖`; the belief that emerges is conditioned on *which* field was disturbed.
- **Disaster-typed beliefs**: the `BeliefConcept` enum (religion.rs:3-10) currently has `NaturalAgent`, `MoralOverseer`, `Afterlife`, `Taboo`, `Ritual`. The design adds *emergent* belief tags derived from the disaster kind:
  - **Earthquake → `Taboo { action: "forbidden_ground" }` or `NaturalAgent { domain: "earth" }`** (the belief concept carries the disaster's field signature).
  - **Flood → `Taboo { action: "forbidden_lowland" }` or `Ritual { cost: 0.3, kind: "rain_dance" }`** (the ritual cost is `μ·M·σ(T_opt−T)` from the substrate regrowth formula).
  - **Storm → `MoralOverseer { domain: "sky" }` or `Ritual { kind: "wind_charm" }`**.
  - **Volcanism → `Taboo { action: "forbidden_vent" }` or `NaturalAgent { domain: "fire" }`**.
  - **Plague → `Taboo { action: "forbidden_contact" }` or `MoralOverseer { domain: "pestilence" }`**.
- **Myth-formation lag**: a single disaster event triggers a *single* belief today (one `add_belief(50)`). A *myth* is a chain of beliefs across time and population, so the design adds:
  - **`belief_promotion`**: when a disaster-generated belief is observed by 10+ agents across 5+ ticks, it is *promoted* into a `ClusterCulture` (emergence.rs:191+). The promotion is the same significance-threshold machinery that legends uses (legends/src/graph.rs).
  - **`myth_anchor`**: the *site* of the disaster becomes a named place in the legends graph (`EntityKind::Settlement` or a new `EntityKind::DisasterSite`). Subsequent events at the same site chain via `LegendEdge::CausedBy` (model.rs:160). This is the "Battle of Ash Valley" pattern from substrate doc §6.
  - **`chronicle_emission`**: each disaster emits a `RawSimEvent { kind: EventKind::Disaster, … }` (legends/model.rs:31), which the existing `emergence_legends` (emergence.rs:548-611) can ingest with one new line:
    ```rust
    for disaster in self.last_disasters().to_vec() {
        let raw = RawSimEvent::new(tick, EventKind::Disaster, SourceCrate::Engine, disaster.magnitude)
            .with_participant(SourceCrate::Engine, SimRuntimeId(disaster.id), Role::Cause);
        let _ = self.emergence_ingest_legend(raw);
    }
    ```
    The chain `Disaster → CausedBy → Famine → CausedBy → Migration → CausedBy → CulturalSpeciation` is then visible in `saga_of(disaster_site)` (legends/src/query.rs).

### 3.5 Disaster severity and chronicling (legends × raw_magnitude)

The substrate doc §3.2 says legends × all fields = 0 cells (pure event sink). But the *trigger* of a disaster, by being a `RawSimEvent`, IS the path legends uses. The design adds:

- `raw_magnitude = M_q` for earthquakes, `watershed_precip / DRAINAGE` for floods, `‖∇T‖·‖∇M‖ / LHT` for storms, `vent_pressure / vent_threshold` for volcanism, `R0·contact - 1` for plagues. All are normalized 0..1.
- The legends engine's significance function (legends/src/model.rs:84, `Role::weight`) is unchanged; disasters are events with magnitude and one or two participants (the disaster site + the affected cluster).
- A *plague* is more chronicle-worthy than a *quake* of the same `magnitude` because plagues unfold over 50–200 ticks and the `Participant` list grows; a quake is over in 1 tick. The `raw_magnitude` for plagues is *time-integrated* (sum of weekly case-fatality) so the saga entry captures the chronic phase.

This is the substrate's "downward causation" applied to chronicling: the disaster doesn't know it is myth-worthy; the legend graph measures it.

### 3.6 Diplomacy effects (faction relations × P, B)

- Two polities sharing a `W` watershed (a flood-affected region) experience a `‖∇P‖` spike as one polity receives the other's migrants. The diplomacy layer (substrate doc L10×P read) reads the gradient and biases the `diplomacy_conflict_threshold` toward war (audit §4.0: `‖∇P‖ → treaty tension`).
- Resource wars after a flood: the receiver polity's `B` drops (it cannot feed the new arrivals). Its `economic_focus` shifts to Agrarian (audit §4.0: `B → economic_focus`). The sender polity's `B` may recover (its population is gone); the receiver's `cohesion` and `dispossessed_permille` change. Diplomacy sees the gradient.
- Plague-induced xenophobia: a polity with high plague mortality writes `set(legend_severity, high, …)` and the *rumor* layer (legends/src/rumor.rs) propagates "outsiders bring disease" through the social graph. The next diplomacy signal between that polity and its neighbors is biased by the rumor, *not* by an explicit `phase_diplomacy` rule — the rumor is a write to the social graph, which `phase_emergence_social` already updates.

### 3.7 Faction emergence (14-dim ideology cluster)

The substrate doc §2.3 L3 specifies `cluster_with_gradients(ideologies, fields, k)` with a 14-dim vector. Disasters add the *temporal* axis:

- A cluster that survived a flood 50 ticks ago has its ideology vector biased toward `M` (avoidance) and away from `B` (distrust of the local food economy).
- A cluster that survived a volcano has its vector biased toward `T` (cold tolerance) and `E` (fire ritual centrality).
- The 14-dim drift is per-tick; the disaster's effect is permanent in the cultural record (cluster cultures are read-and-decayed, not reset). The `cluster_cultures` BTreeMap (emergence.rs:65) is the substrate-projection of these vectors.

---

## 4. Phase ordering — where each disaster hook sits in `Simulation::tick`

The audit (EMERGENCE_COUPLING_AUDIT.txt §1) lists the current phase order. The design fits each disaster kind into the existing order with the substrate as the only connective tissue:

```
[tick n]
 1  phase_production           engine.rs:2224    — uses prior-tick B
 2  phase_citizen_lifecycle    engine.rs:2261    — births/deaths
 3  phase_military             engine.rs:2334
 4  phase_economy              engine.rs:2553    — reads B, M gradients
 5  phase_planet               engine.rs:1522    — climate.advance()
 6  phase_diplomacy            engine.rs:2444    — every 500 ticks
 7  phase_tactics              engine.rs:1614
 8  phase_voxel                engine.rs:1682    — CA reaction table
 9  phase_compact              engine.rs:1744
10  phase_buildings            engine.rs:1972
11  phase_diffusion            engine.rs:2006
12  phase_disasters            disasters.rs:70   ──► THIS DOC
13  phase_life                 engine.rs:2042
14  phase_emergence            emergence.rs:135
15-24 ... (research, belief, unrest, cohesion, ...)
```

**`phase_disasters` runs after `phase_voxel` and before `phase_life`.** This is intentional:
- After `phase_voxel`, the CA has applied reactions and the substrate fields F and E are post-step.
- After `phase_disasters`, the substrate fields M, F, P, B have the disaster's writes.
- Before `phase_life`, the agents in the loop see the post-disaster field state (their `set(P, p, ...)` for births/deaths uses the new carrying capacity).

**The substrate doc S5 gap** (audit §6 D5: `phase_emergence` not in `PHASE_ORDER`) is *not* the responsibility of this doc. But the disaster `RawSimEvent` emission must land in `phase_emergence.legends` *after* `phase_disasters`, which is the case once S5 is closed.

### 4.1 Internal phase_disasters order

The current `phase_disasters` collects onset sites first, then triggers (disasters.rs:80-110). The design preserves this pattern and extends it to five kinds:

```rust
pub fn phase_disasters(&mut self) {
    let wildfire_sites = wildfire_onset_sites(&self.weather_grid, self.research_tier());
    let quake_sites    = quake_onset_sites(&self.weather_grid, &self.geology, &self.substrate);
    let flood_sites    = flood_onset_sites(&self.weather_grid, &self.watershed_map);
    let storm_sites    = storm_onset_sites(&self.weather_grid, &self.substrate);
    let volcano_sites  = volcano_onset_sites(&self.geology, &self.substrate);
    let plague_sites   = plague_onset_sites(&self.weather_grid, &self.population, &self.immune);

    // Sort all sites by deterministic key (region, cell) for replay stability.
    let mut all: Vec<(DisasterKind, WorldCoord, f32)> = ...;
    all.sort_by_key(|(k, c, _)| (c.x, c.y, c.z, std::mem::discriminant(k)));

    for (kind, pos, magnitude) in all {
        trigger_disaster(self, kind, pos, magnitude);  // existing apply_disaster + add_belief
        // NEW: emit RawSimEvent to the watch bus for legends ingest
        self.emit_disaster_event(kind, pos, magnitude);
    }
}
```

The sort is the same determinism pattern as the audit's `BTreeMap` / `sort_unstable` discipline. The `Discriminant` sort key keeps enum variant order stable.

### 4.2 Downward-causation budget per tick

The substrate doc §5.2 (explosion tripwire) bounds mass and energy writes. The design's disaster writes must respect this:

| Field | Per-tick budget per cell | Disaster contribution |
|-------|--------------------------|------------------------|
| F | 5% of `|F|_cell` per tick | quake: 10% (one-shot); flood: 2% (silt); storm: 1% (debris) |
| M | 10% of `|M|_cell` per tick | flood: 30% (saturated event, allowed by `trigger_disaster`'s voxel write path) |
| E | 5% of `|E|_cell` per tick | volcano: 20% (allowed; one-shot) |
| P | 10% of `|P|_cell` per tick | plague: 30% (one-shot; lifetime 50 ticks) |
| B | 2% of `|B|_cell` per tick | flood topsoil: 1% |

These are inside the substrate's explosion tripwire envelope. The substrate's `set()` rate-limit table (substrate doc §1.3) is the choke-point; disasters use `WriteSource::Disaster` which has its own rate-limit column.

---

## 5. Replay determinism and substrate-first compatibility

The substrate doc §2.3 mandates that emergent layer writes be a pure function of substrate state. The design preserves this:

- **Disaster onset**: a pure function of `(weather_grid, geology, substrate, immune_memory, tick)`. No layer-internal RNG.
- **Disaster damage**: a pure function of `(kind, onset_site, magnitude, agent_ecs_snapshot)`. The voxel write is deterministic (existing `apply_disaster` at disasters.rs:136-217 is deterministic). The `add_belief(50)` is constant.
- **`emit_disaster_event`**: a pure function of the disaster's `(kind, pos, magnitude, tick)`. The legends worker (legends/src/worker.rs) is already on the watch bus and is deterministic.
- **Replay**: `replay_log` records the `DisasterEvent`s in sorted order (per §4.1 sort key). A replay with the same `rng_seed` and substrate state produces the same events.

The substrate doc §7.3 specifies the substrate hash as part of the seeded RNG. The disaster trigger's deterministic input includes the substrate hash, so a substrate perturbation (e.g. `auto_tune.rs` changing `D_T`) shifts the disaster distribution *in a reproducible way* — the same perturbation, the same disaster distribution. This is the design's commitment to the substrate-first style.

---

## 6. Migration plan (out of scope for this PR — but spelled out)

The substrate doc §3.2 sprint plan already addresses most of the substrate plumbing. The disaster-specific work is layered on top:

### 6.1 Sprint 5 — disaster substrate first wiring (proposed)

| # | Task | Edit site | DoD |
|---|------|-----------|-----|
| 5.1 | Add `Substrate.set(F, p, …, DisasterSource)` writes in `apply_disaster` | `crates/engine/src/disasters.rs:136-217` | quake writes `set(F, p, -debris, DisasterSource::Quake)` |
| 5.2 | Add `M`, `E`, `B` writes for each disaster kind (per §2 table) | same | unit test: post-quake `M[p]` rises |
| 5.3 | Replace tide-based quake trigger with `‖∇F‖` at plate boundary | `crates/engine/src/disasters.rs:84-103` | `quake_onset_sites` reads substrate; old `tide_offset` trigger preserved as fallback |
| 5.4 | Add `flood_onset_sites` (watershed-sum) and `storm_onset_sites` (‖∇T‖·‖∇M‖) | new fns in `disasters.rs` | unit tests on synthetic weather |
| 5.5 | Add `Volcanism` variant to `DisasterKind` and `volcano_onset_sites` | new | 1 unit test using `GeologyMap.subduction_zones` (proposed in 6.2) |
| 5.6 | Add `Plague` trigger: `R0(T, M)·‖∇P‖ > 1 ∧ sus > 0.3` | new | unit test: synthetic climate + cluster → plague |
| 5.7 | Add `ImmuneMemory` ECS component | `crates/genetics/src/lib.rs` | test: post-plague exposure drops R0 |
| 5.8 | Add `emit_disaster_event` to `phase_disasters` | `crates/engine/src/disasters.rs` | saga graph has disaster event next tick |

### 6.2 Sprint 6 — substrate-side enablers (proposed, depends on Sprint 0–4 of substrate doc)

| # | Task | Edit site | DoD |
|---|------|-----------|-----|
| 6.1 | Add `GeologyMap.plate_boundary: Grid<bool>` (or region adjacency) | `crates/planet/src/geology.rs` | 1-bit per cell, derived from PlanetConfig |
| 6.2 | Add `GeologyMap.subduction_zones: Vec<(RegionId, RegionId)>` | same | list of (high-stress, low-stress) pairs |
| 6.3 | Add `Watershed: Grid<u8>` to substrate (per §2.2) | `crates/physics-substrate/src/grid.rs` | flood trigger can read it |
| 6.4 | Add `‖∇F‖_vertical` projection (top vs bottom voxel cell) | `crates/physics-substrate/src/operators.rs` | volcanism trigger has a depth-aware gradient |
| 6.5 | Add `ImmuneMemory` to ECS schema + serde | `crates/genetics/src/lib.rs` | survives save/load |

### 6.3 Sprint 7 — myth-formation and chronicling (proposed)

| # | Task | Edit site | DoD |
|---|------|-----------|-----|
| 7.1 | Add `belief_promotion` (10+ agents × 5+ ticks → `ClusterCulture` entry) | `crates/engine/src/religion.rs` | post-quake cluster has `NaturalAgent { domain: "earth" }` belief |
| 7.2 | Add `myth_anchor`: disaster site becomes named `EntityKind::DisasterSite` | `crates/legends/src/model.rs` + `crates/legends/src/graph.rs` | `saga_of(disaster_site)` returns the causal chain |
| 7.3 | Add `disaster_typed_belief_concept(kind, fields) -> BeliefConcept` | `crates/engine/src/religion.rs` | flood produces `Ritual { kind: "rain_dance" }`, plague produces `Taboo { action: "forbidden_contact" }` |
| 7.4 | Plague chronicle emission: time-integrated `raw_magnitude` | `crates/engine/src/disasters.rs` | saga entry for plague spans 50–200 ticks, not 1 |

### 6.4 Acceptance criteria (full disaster emergence done when)

- All five disaster kinds have substrate-derived onsets (no hand-coded probability).
- All five disaster kinds write to substrate fields (F, M, E, P, B) on onset.
- `phase_disasters` emits a `RawSimEvent` for each disaster, ingested by `phase_emergence.legends`.
- Replay of a 10k-tick scenario with a flood + a quake + a plague produces identical `replay_log` and identical `saga_of(disaster_site)` output.
- `substrate_conservation` invariant holds for 100k ticks across all 4 reference scenarios.
- Plague's `ImmuneMemory` drops `R0` for exposed DNA classes and prevents an *immediate* second plague in the same population.
- Flood → migration → collapse in a neighboring polity is observable in the diaspora's `dispossessed_permille` and `cohesion` scalars (audit §3.1) within 10 ticks of the flood.
- Myth-formation: a polity hit by an earthquake develops a `NaturalAgent { domain: "earth" }` belief within 50 ticks; the belief is *promoted* (10+ agents × 5+ ticks) and becomes part of `cluster_cultures`.

---

## 7. Open questions for review

1. **Plate-boundary geometry.** `GeologyMap` is a 1-D latitude band model today (`geology.rs:284-292`). The substrate doc and this design assume a 2-D field. Sprint 6.1 is the migration to a 2-D `Grid<bool>`. Until then, the quake trigger falls back to the existing `|latitude_fp| ≥ 40_000` proxy, which is *not* emergent (it's a scalar in the script). Acceptable for v0.5; not acceptable for the doctrine.
2. **`ImmuneMemory` granularity.** Per-DNA-class hash, per-lineage, or per-cluster? The substrate doc S5 closure suggests per-DNA-class (cheap, replay-stable). Per-lineage (the existing `lineage_id` in `crates/genetics/src/sentience.rs`) is finer-grained but explosion-prone (a 1000-lineage polity has 1000 `ImmuneMemory` entries). **Recommendation:** per-DNA-class hash, with a per-cluster aggregate for the trigger.
3. **Plague realism vs spectacle.** `R0(T, M) = 2..5` in real epidemiology; a 1-tick trigger is "everyone sick at once." The design's 50–200 tick duration (driven by `‖∇P‖`-modulated contact rate) is plausible but should be tuned by a domain expert, not a code reviewer. **Action:** route `R0_base` and `duration_ticks` through `scenarios/*.yaml`.
4. **Myth-formation threshold.** 10+ agents × 5+ ticks is a guess. The legends engine's `PROMOTION_THRESHOLD` (audit §6 D3) is 0.5 significance. Should the *belief* promotion be the same threshold? **Recommendation:** use the same significance machinery; myths are events with high significance, same as battles.
5. **Determinism vs emergence.** The substrate doc §7.3 says replay determinism is preserved by hashing the substrate. The design's disaster trigger is a pure function of `(weather_grid, geology, substrate, immune_memory, tick)`. A `DisasterEvent` is then a `RawSimEvent` with the same hash. If the substrate's `D_T` is auto-tuned (substrate doc §4.2), the disaster distribution *changes between runs with different auto-tune seeds*. **Recommendation:** auto-tune runs are replay-equivalent only when the auto-tune seed is fixed; this is consistent with the substrate doc.
6. **`add_belief(50)` constant vs field-derived.** The current `trigger_disaster` (disasters.rs:39-40) hardcodes `DISASTER_FAITH_GAIN = 50`. The design reframes this as a function of the disaster's `magnitude` and the local `‖∇T‖ + ‖∇B‖` (the substrate doc §4.4 hardship formula). **Recommendation:** deprecate the constant in favor of `disaster_faith_gain(kind, magnitude, field_gradients) -> u64`.
7. **Volcanism and existing `Meteor` overlap.** The `Meteor` kind (disasters.rs:140-159) is essentially a one-shot volcanism (LAVA + GRAVEL + AIR ring). Is the new `Volcanism` a *separate* kind or a *sustained* version of `Meteor`? **Recommendation:** keep them distinct — `Meteor` is a script event (divine intervention), `Volcanism` is emergent (substrate onset). They share the `LAVA` damage write but differ in trigger and chronicling.

---

## 8. References

- [`docs/design/PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md) — substrate doctrine, 40-cell matrix, lags, failure modes.
- [`docs/design/PHYSICS_INTEGRATION_PLAN.md`](PHYSICS_INTEGRATION_PLAN.md) — phased sprint plan, 14 silo'd call-sites, 40-cell matrix mapping.
- [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) — emergence-default principle, two-layer content model, MVP+ scale targets.
- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only physics + environment + genome are authored; everything else emerges.
- [`EMERGENCE_COUPLING_AUDIT.txt`](../../EMERGENCE_COUPLING_AUDIT.txt) — current phase DAG; `phase_disasters` at engine.rs:1275–1283; fires only on `temp_c_fp + precip_mm_fp` and `tide + latitude`.
- [`MULTIPOLITY_DESIGN.txt`](../../MULTIPOLITY_DESIGN.txt) — per-faction shadow state; disaster damage should ultimately hit `PolityMacroState.belief`, `.unrest`, `.cohesion` (not the global scalars).
- [`crates/engine/src/disasters.rs`](../../crates/engine/src/disasters.rs) — current `phase_disasters`, `apply_disaster`, `trigger_disaster`, `invoke_divine_disaster`; `DISASTER_FAITH_GAIN = 50` at line 39.
- [`crates/planet/src/geology.rs`](../../crates/planet/src/geology.rs) — `BiomeKind`, `GeologyMap`, latitude-band model.
- [`crates/planet/src/weather.rs`](../../crates/planet/src/weather.rs) — `WeatherCell`, `compute_weather`, `storm_intensity_fp` already substrate-derived (line 152–154).
- [`crates/climate/src/lib.rs`](../../crates/climate/src/lib.rs) — `ClimateState`, CO₂ forcing, sea-level response.
- [`crates/engine/src/religion.rs`](../../crates/engine/src/religion.rs) — `BeliefConcept`, `Religion`, `emerge_belief(hardship, group_size, bias)` (silo'd call S1 in substrate doc).
- [`crates/engine/src/emergence.rs`](../../crates/engine/src/emergence.rs) — `phase_emergence` at 159–171; `emergence_legends` at 548–611.
- [`crates/legends/src/model.rs`](../../crates/legends/src/model.rs) — `EventKind::Disaster` (line 31), `EntityKind::Disaster` (line 62), `RawSimEvent` (line 181).
- Gutenberg, B. & Richter, C. F. (1954). *Seismicity of the Earth and Associated Phenomena*. (Magnitude-frequency scaling for earthquakes.)
- Anderson, R. M. & May, R. M. (1992). *Infectious Diseases of Humans*. (R0(T, M) shape for plague.)
- Tainter, J. (1988). *The Collapse of Complex Societies*. (Collapse as a function of `B`-`P` gap, not a single event.)
- Campbell, D. T. (1974). "Downward causation" in hierarchically organized biological systems. (Doctrinal anchor for the substrate contract.)
- Tilly, C. (1985). "War Making and State Making as Organized Crime." (Disasters as a driver of state formation; supports the myth-formation lag in §3.4.)

---

*End of design — implementation agents: do not couple disasters to emergent layers via API; route every write through `PhysicsFields::set(...)` per substrate doc §2.3.*
