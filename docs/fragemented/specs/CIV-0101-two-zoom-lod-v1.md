# CIV-0101 Two-Zoom LOD Spec v1

## Context
Defines dual-resolution simulation views for Civ-Sim: macro governance view and micro population/economy view, with deterministic aggregation and drill-down.

## Invariants
1. Macro aggregates are deterministic functions of micro state at the same tick.
2. Zoom transitions do not mutate simulation truth state.
3. Aggregation error bounds are explicit and versioned.
4. Drill-down and roll-up preserve identity mapping of regions/cohorts.

## Interfaces
- `metrics.compute_micro(state, context) -> micro_snapshot`
- `metrics.aggregate_macro(micro_snapshot, schema) -> macro_snapshot`
- `io.render_zoom(run_id, tick, zoom_level, filters) -> view_model`

## Data Model Additions
1. Add `lod_snapshots(run_id, tick, zoom_level, region_key, payload_json, checksum, created_at)`.
2. Add `lod_mappings(version, macro_key, micro_selector, created_at)`.
3. Add `lod_error_bounds(version, metric, p50_error, p95_error, created_at)`.

## Event Contracts
1. Add `metrics.lod_snapshot.v1` for each zoom snapshot emit.
2. Add `metrics.lod_mapping_changed.v1` when aggregation maps version bump.
3. Add `metrics.lod_error_reported.v1` for validation runs.

## Acceptance Checks
1. Macro totals equal micro sums within configured error bounds.
2. Replay of zoom exports is byte-stable for identical inputs.
3. Changing zoom level does not alter `tick_states` checksum.
4. Benchmark scenario renders both zoom levels within SLA budget.

## Determinism Notes
- Version and pin aggregation mappings per run.
- Keep roll-up computations pure and order-stable.

---

# Section 2: RTS Command Layer & Zoom-Level Command Validity

## Context
Real-time strategy (RTS) gameplay operates on a command interface overlaid on the LOD system. Commands issued by player are valid only at certain zoom levels. Command execution produces state changes that propagate through both LOD levels.

## RTS Zoom Layer Definitions

### Zoom Level 1: Strategic (Region-Level Commands)
**Focus:** Nation-wide strategy, regional resource flows, coalition management
**Data Contract:** Region aggregates, diplomatic relations, military unit counts per region, trade volume
**Valid Commands:**
- Diplomacy: `propose_alliance(target_region)`, `declare_war(target_region)`, `offer_trade(goods, quantities)`, `request_peace()`
- Regional control: `set_regional_policy(policy_bundle)`, `allocate_resources_to_region(region, amounts)`
- Military: `move_army_group(army_id, target_region)`, `garrison_region(troops)`, `form_coalition(member_regions)`
- Trade: `establish_trade_route(source_region, dest_region, goods)`, `embargo_region(target_region)`

**Data Size:** ~0.5 KB per region per snapshot (includes: population, GDP, military_count, diplomatic_status, trade_flows)
**Update Frequency:** 100 ms minimum; fewer updates if state stable (coalescing)
**Latency Requirement:** Command ACK within 50 ms; execution within 1 second

### Zoom Level 2: Tactical (District-Level Commands)
**Focus:** City management, district economy, local military operations, structure placement
**Data Contract:** District details (structures, units, citizen cohorts, resource stocks, job assignments), local trade, institution states
**Valid Commands:**
- Structure: `build_structure(structure_type, location)`, `repair_structure(structure_id)`, `garrison_structure(troops)`, `demolish_structure(structure_id)`
- Military: `move_unit(unit_id, hex_location)`, `attack_target(unit_id, target_unit_or_structure)`, `form_group(unit_ids, formation)`, `resupply_unit(unit_id, depot_id)`
- Economy: `set_production_quota(good_type, quota)`, `set_tax_rate(rate)`, `issue_subsidy(cohort_id, amount)`, `establish_market(good_type)`
- Citizen management: `reassign_jobs(cohort_id, new_job)`, `issue_conscription(count)`, `offer_migration_incentive(target_district)`
- Diplomacy: `send_emissary(target_district)`, `request_support(ally_district)`, `propose_trade_agreement(target_district, terms)`

**Data Size:** ~2 KB per district per snapshot (structures, units, cohorts, production state, institution details)
**Update Frequency:** 50 ms (more frequent due to tactical changes); coalesce minor updates
**Latency Requirement:** Command ACK within 50 ms; execution within 100 ms (tactical immediacy)

### Zoom Level 3: Simulation (Citizen-Level Details)
**Focus:** Research, deep analysis, emergent behavior observation
**Data Contract:** Individual citizen states (job, welfare, ideology, stress, kinship), event details, institution leadership, skill progression
**Valid Commands:** (Research/analysis mode, not real-time gameplay)
- Query: `get_citizen(citizen_id)`, `list_citizens_by_cohort(cohort_id)`, `trace_causal_chain(event_id)`
- Analysis: `compute_welfare_distribution()`, `analyze_migration_drivers()`, `simulate_what_if_policy(policy_delta)`
- Visualization: `export_citizen_genealogy()`, `plot_institution_lifecycle()`, `show_causal_graph(event_id)`

**Data Size:** ~0.1 KB per citizen (10k citizens &asymp; 1 MB); full export ~10 MB for complete city state
**Update Frequency:** On-demand (not real-time streaming)
**Latency Requirement:** Query completion within 1 second; deep analysis within 10 seconds

## RTS Command Data Contracts

### Command Request Format (JSON-RPC)
```json
{
  "jsonrpc": "2.0",
  "method": "execute_command",
  "params": {
    "zoom_level": 2,
    "command": {
      "type": "move_unit",
      "unit_id": 42,
      "target": {"x": 150, "y": 200}
    }
  },
  "id": 1001
}
```

### Command Response Format
```json
{
  "jsonrpc": "2.0",
  "result": {
    "command_id": "cmd-unique-id-9876",
    "status": "queued",
    "tick_execution": 1005,
    "error": null
  },
  "id": 1001
}
```

### Command Validation Rules

| Condition | Validation | Response |
|-----------|-----------|----------|
| Zoom level valid for command type | Unit move only valid at zoom &gt; 2 | Reject if zoom 1 and command = move_unit |
| Unit/structure/actor exists | Entity ID must exist in state | Error: "unit_id 999 not found" |
| Player owns entity | Unit must belong to player faction | Error: "unauthorized: unit owned by faction_B" |
| Command is feasible | Move distance &lt; unit range; build cost &lt; treasury | Error: "insufficient treasury (need 100, have 50)" |
| Map location valid | Hex within map bounds, passable for unit type | Error: "location (999, 999) out of bounds" |
| Cooldown respected | Some commands have cooldown (e.g., diplomacy proposal every 10 ticks) | Warn: "diplomacy proposal on cooldown, 3 ticks remaining" |

## RTS State Update & Broadcast

### Server-Pushed State Updates (Zoom-Specific)

**Zoom Level 1 Update (100 ms cadence):**
```json
{
  "type": "state_update",
  "zoom_level": 1,
  "tick": 5000,
  "data": {
    "regions": [
      {
        "region_id": "R1",
        "population": 12500,
        "military_strength": 250,
        "diplomatic_status": {"R2": "cooperative", "R3": "sanctioned"},
        "trade_balance": +150,
        "gdp": 5000
      },
      ...
    ]
  }
}
```

**Zoom Level 2 Update (50 ms cadence):**
```json
{
  "type": "state_update",
  "zoom_level": 2,
  "tick": 5000,
  "data": {
    "districts": [
      {
        "district_id": "D1",
        "structures": [
          {"structure_id": 1, "type": "barracks", "hp": 100, "garrison": 50},
          {"structure_id": 2, "type": "farm", "production_rate": 15}
        ],
        "units": [
          {"unit_id": 42, "type": "soldier", "x": 100, "y": 150, "hp": 45, "morale": 0.8}
        ],
        "cohorts": [
          {"cohort_id": "C1", "job": "farmer", "count": 500, "avg_welfare": 0.7}
        ],
        "resource_stock": {"food": 1200, "metal": 450, "wood": 800}
      },
      ...
    ]
  }
}
```

### Command Execution Events (Per-Tick Phase)

When a command executes (tick N + offset), server broadcasts execution event:
```json
{
  "type": "event",
  "event_type": "military.unit_moved.v1",
  "tick": 5001,
  "correlation_id": "cmd-unique-id-9876",
  "data": {
    "unit_id": 42,
    "origin": {"x": 100, "y": 150},
    "destination": {"x": 150, "y": 200},
    "distance_traveled": 50,
    "terrain_effects": "no effect",
    "enemies_encountered": []
  }
}
```

## Client-Side Prediction Model

### Motivation
Network latency (50-200 ms) makes RTS feel sluggish if client waits for server ACK before moving unit. Solution: **client predicts** unit position locally; server confirms authoritative position periodically.

### Prediction Algorithm

1. **Command Issued (t=0):** Player right-clicks; client immediately moves unit locally
   - `predicted_position = current_position + (target - current) × prediction_speed`
   - `prediction_speed = unit.speed × terrain_multiplier` (estimate, using local knowledge)
   - Unit animates smoothly from current to target

2. **Server Update Arrives (t=50-100 ms):** Server broadcasts authoritative state
   - Server: `actual_position = resolve_movement_physics(unit, terrain, obstacles)`
   - Client receives `authoritative_position`

3. **Reconciliation:**
   - **Delta \< 1 hex:** Smooth transition over 50 ms (unit drifts to correct position)
   - **Delta &gt; 1 hex:** Snap to authoritative position (visible but quick; indicates desync)
   - **Delta > 5 hex:** Log warning and request full state resync (indicates serious issue)

### Determinism & Validation

Client prediction is **deterministic** given:
- Start position, target, unit speed, terrain map, RNG seed for tie-breaking pathfinding
- Identical start conditions → identical predicted path
- Server validates: predicted path vs actual path; major divergences logged as sync errors

### Acceptable Prediction Error

| Scenario | Error Budget | Correction Strategy |
|----------|--------------|-------------------|
| WiFi low latency (50 ms) | &plusmn;1 hex | Smooth transition |
| WiFi medium latency (100 ms) | &plusmn;2 hex | Smooth transition with slight desync |
| Cellular high latency (200+ ms) | &plusmn;3 hex | Snap correction visible but acceptable |

## Command Queuing & Execution Phases

### Phase 1: Policy (Tick N, ~5 ms)
- Faction policies evaluated
- Allocators resolve economy (market clearing, joule allocation)
- Transfers executed
- **RTS Impact:** No unit/structure changes

### Phase 2: Movement (Tick N, ~5 ms)
- Units move per movement orders queued in prior tick(s)
- Collision detection applied
- Vision updates
- **RTS Impact:** Units may reach destinations, trigger proximity alerts

### Phase 3: Combat (Tick N, ~5 ms)
- Attack orders resolved
- Damage applied, casualties calculated
- Morale updates
- **RTS Impact:** Unit HP/morale changes, units may be destroyed or routed

### Phase 4: Events & Cascades (Tick N, ~5 ms)
- Supply depletion triggers resupply requests
- Low morale triggers routing
- Migration decisions trigger migrations
- Dissent triggers protest events
- **RTS Impact:** Cascading failures may propagate (e.g., supply → low morale → routing → combat loss)

### Command Execution Timing

Commands issued at tick N are enqueued and executed per following schedule:
- **Tick N+0:** Command validation (queued, not yet executed)
- **Tick N+1:** Phase 1-4 execution; command triggers (e.g., move_unit enters movement phase)
- **Tick N+2:** Cascading effects (e.g., unit reaches destination, triggers proximity event)

**Example:** Player issues `move_unit(42, target)` at tick 5000.
- Tick 5000: Command queued, validated
- Tick 5001: Movement phase executes; unit travels per speed
- Tick 5002: Unit reaches destination (if reachable in 1 tick); proximity alert if enemies near

## Determinism & Replay of RTS Commands

### Requirement
Given identical:
1. Scenario spec (map, initial state)
2. Command sequence (same command_ids, order, timing)
3. Random seed

Replay should produce: identical unit states, identical combat outcomes, identical unit deaths.

### Validation
```rust
#[test]
fn test_rts_command_replay_determinism() {
    let scenario = load_scenario("island-skirmish");
    let commands = vec![
        move_unit(5, target_a),
        attack_unit(5, enemy_b),
        form_group(vec![5, 10, 15], "wedge"),
    ];

    // Run A
    let (state_a, events_a) = execute_commands(&scenario, &commands, seed=42);

    // Run B (identical)
    let (state_b, events_b) = execute_commands(&scenario, &commands, seed=42);

    // Assertions
    assert_eq!(state_a.units, state_b.units);  // Byte-identical
    assert_eq!(events_a.len(), events_b.len());
    for (e_a, e_b) in events_a.iter().zip(events_b.iter()) {
        assert_eq!(e_a, e_b);  // Same event order & content
    }
}
```

## RTS Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Command latency (ACK) | < 50 ms (p95) | Over WiFi; includes network roundtrip |
| State update frequency (zoom 1) | 100 ms | Regional updates; lower frequency OK |
| State update frequency (zoom 2) | 50 ms | Tactical updates; critical for responsiveness |
| Tick rate (wall-clock) | &gt; 10 Hz | 100 ms per tick; maintains smoothness |
| Concurrent players | &gt; 100 | Supported server load per instance |
| Unit count per faction | &lt; 1000 | Practical limit before performance degrades |
| Memory footprint | < 500 MB | For medium scenario (5 regions, 50 districts) |

## Examples

### Example 1: Unit Movement with Local Prediction

**Tick 5000:**
1. Player right-clicks hex (150, 200); client sends `move_unit(42, (150, 200))`
2. Client predicts: unit moves 10 hexes/sec × 0.1 sec = 1 hex progress = (101, 150)
3. Server receives command; enqueues for execution

**Tick 5001:**
1. Server executes movement: unit travels from (100, 150) to (150, 200) in 5 ticks (flat terrain, speed=10)
2. Unit advances 2 hexes: (102, 152)
3. Server broadcasts: `state_update(unit_42: position=(102, 152))`
4. Client receives update; compares to predicted (103, 151): delta=1 hex; smooth transition over 50 ms

**Tick 5002-5004:** Unit continues movement, predictions drift &plusmn;1 hex each tick, server updates correct position

**Tick 5005:**
1. Unit reaches destination (150, 200)
2. Server broadcasts: `unit_reached_destination(42, (150, 200))`
3. Proximity check: enemy scout detected at (151, 200), distance=1 hex
4. Broadcast: `proximity_alert(unit_42, enemies_near)`

### Example 2: Attack Command with Combat Resolution

**Tick 5100:**
1. Player selects group of 3 soldiers (IDs: 1, 2, 3) and right-clicks enemy unit 99
2. Client sends: `attack_group([1, 2, 3], target=99)`
3. Server validates: units exist, player owns them, target exists and is enemy; queues for execution

**Tick 5101:**
1. Combat phase resolves:
   - Unit 1 vs Unit 99: 1 attacks strength 25 - 99 armor 5 = 20 damage (+/- RNG 10%)
   - Unit 2 vs Unit 99: similar
   - Unit 3 attacks different enemy: etc.
2. Unit 99 takes cumulative damage; HP 50 → 15 (after 3 attacks averaging 12 damage)
3. Unit 99 morale drops 15% due to damage; morale now 0.65 (was 0.80)
4. Server broadcasts: `military.unit_combat.v1` event with damage details
5. Client receives event; updates unit 99 HP bar, highlights damage
6. Unit 99 counter-attacks (enemy units also attack in combat phase)

**Tick 5102:**
1. Unit 99 takes another hit; HP 15 → 3
2. Morale drops below routing threshold (0.2); unit 99 routes (flees)
3. Server removes unit 99 from combat; marks as "routing" (unavailable for 20 ticks)
4. Broadcast: `military.unit_routed.v1` event; client shows unit 99 fleeing toward nearest friendly structure

---

# Section 3: Aggregation Error Bounds & LOD Versioning

## Schema Versioning

LOD snapshots include schema_version to enable backward compatibility:
```json
{
  "lod_schema_version": "1.0",
  "zoom_level": 2,
  "tick": 5000,
  "data": { ... }
}
```

Schema changes (adding fields, renaming) increment version:
- v1.0: Initial schema (release candidate)
- v1.1: Add district_morale field (additive, backward-compatible)
- v2.0: Rename structure.hp → structure.health_points (breaking change)

Client must handle version mismatches:
- v1.x ← v1.0: Ignore new fields, use old fields (forward-compatible)
- v2.x ← v1.0: Error; cannot parse (requires schema upgrade)

## Aggregation Error Bounds

Macro aggregates are deterministic but may have bounded rounding errors. Errors are documented and monitored:

| Metric | Aggregation Method | Error Bound (p95) | Notes |
|--------|-------------------|------------------|-------|
| Population | Sum of district cohorts | &plusmn;0 (exact) | No rounding if integer counts |
| Food stocks | Sum of district stocks | &plusmn;0.01 units | Fixed-point accumulation; negligible |
| GDP | Sum of district production × price | &plusmn;1.0 units | Price aggregation may have &plusmn;0.5% error per price |
| Gini (inequality) | Weighted average of district Ginis | &plusmn;0.02 (0.02 on [0,1] scale) | Aggregation of cohort distributions introduces approximation |
| Legitimacy | Weighted average of institution legitimacy | &plusmn;0.01 | Different institutions have different weights |

Error bounds are verified in acceptance test:
```rust
#[test]
fn test_aggregation_error_bounds() {
    let scenario = load_scenario("temperate-city");
    let state = execute_ticks(&scenario, 1000);

    let micro = compute_micro(&state);
    let macro = aggregate_macro(&micro);

    // Check bounds
    assert!((macro.population - micro.total_population).abs() < 1);
    assert!((macro.gini - micro.weighted_gini).abs() < 0.02);
    assert!((macro.gdp - micro.total_production).abs() < 1.0);
}
```

---

# Section 4: Full LOD Streaming Architecture

## 4.1 Overview

This section defines the complete LOD streaming architecture: how the server computes LOD-appropriate snapshots, how clients negotiate their preferred LOD, and how the system selects and transitions between LOD levels at runtime.

LOD is not only about visual resolution — it directly controls how much simulation data is transmitted per tick. The system has three discrete LOD levels mapped to the three zoom levels defined in Section 2. Each LOD level has a defined data contract, server-side computation cost, and transmission bandwidth budget.

## 4.2 LOD Level Definitions

### LOD Level 0: Full Citizen Resolution (L0)

**Zoom mapping:** Zoom 3 (Simulation / Research mode)
**Scope:** Individual citizen entities with full attribute set
**Active when:** Camera zoom factor >= Z_CITIZEN threshold; entity count in viewport <= 500
**Server computation:** Full ECS query over `CitizenComponent` for every entity in viewport

```rust
pub struct L0CitizenSnapshot {
    pub citizen_id: EntityId,
    pub position: HexCoord,
    pub job: JobType,
    pub welfare: f32,
    pub ideology: IdeologyVector,
    pub stress: f32,
    pub health: HealthStatus,
    pub age: u32,
    pub kinship_group_id: Option<GroupId>,
    pub institution_membership: Vec<InstitutionId>,
    pub tick: u64,
}
```

**Data size:** ~0.1 KB per citizen; 500 citizens = ~50 KB per snapshot
**Transmission:** On-demand only (no real-time streaming); query response model
**Latency budget:** Response within 1 second for viewport queries

### LOD Level 1: District Aggregates (L1)

**Zoom mapping:** Zoom 2 (Tactical / City view)
**Scope:** Districts as aggregated entities; structures, units, cohort summaries
**Active when:** Camera zoom factor in tactical range (Z_DISTRICT to Z_CITIZEN)
**Server computation:** Aggregate `CitizenCohort` components; enumerate `StructureComponent` and `UnitComponent` per district

```rust
pub struct L1DistrictSnapshot {
    pub district_id: DistrictId,
    pub tick: u64,
    pub population: u32,
    pub cohorts: Vec<CohortSummary>,           // job, count, avg_welfare
    pub structures: Vec<StructureSummary>,     // id, type, hp, garrison
    pub units: Vec<UnitSummary>,               // id, type, position, hp, morale
    pub resource_stock: ResourceStock,
    pub production_state: ProductionSummary,
    pub institution_summary: Vec<InstitutionSummary>,
    pub local_event_count: u32,               // events this tick in district
}

pub struct CohortSummary {
    pub cohort_id: CohortId,
    pub job: JobType,
    pub count: u32,
    pub avg_welfare: f32,
    pub avg_ideology: f32,  // scalar projection on primary axis
}
```

**Data size:** ~2 KB per district per snapshot
**Transmission:** 50 ms cadence (streamed, real-time)
**Latency budget:** State update visible to client within 100 ms of tick

### LOD Level 2: Nation Outlines (L2)

**Zoom mapping:** Zoom 1 (Strategic / World map)
**Scope:** Nations (regions) as high-level aggregates; no unit or citizen detail
**Active when:** Camera zoom factor <= Z_REGION threshold
**Server computation:** Aggregate over all districts in each region

```rust
pub struct L2RegionSnapshot {
    pub region_id: RegionId,
    pub tick: u64,
    pub population: u64,
    pub gdp: u64,
    pub military_strength: u32,       // total unit power
    pub stability: f32,               // 0.0–1.0
    pub legitimacy: f32,              // 0.0–1.0
    pub dominant_institution: Option<InstitutionId>,
    pub diplomatic_relations: HashMap<RegionId, DiplomaticStatus>,
    pub trade_balance: i64,
    pub resource_surplus: Vec<(GoodType, i64)>,  // positive = surplus, negative = deficit
    pub threat_level: f32,            // 0.0–1.0; influences war risk indicator
}
```

**Data size:** ~0.5 KB per region per snapshot
**Transmission:** 100 ms cadence (streamed, real-time)
**Latency budget:** State update visible to client within 200 ms of tick

## 4.3 Automatic LOD Selection

The client determines the active LOD level using three inputs:

1. **Camera zoom factor** — primary selector
2. **Entity count in viewport** — adaptive fallback (high entity density → raise LOD level)
3. **Client FPS** — performance-driven adaptation (FPS \< 30 → raise LOD level by 1)

### LOD Selection Algorithm

```rust
pub fn select_lod_level(
    camera_zoom: f32,
    entities_in_viewport: u32,
    client_fps: f32,
) -> LODLevel {
    // Zoom-based primary selection
    let zoom_lod = if camera_zoom >= Z_CITIZEN_THRESHOLD {
        LODLevel::L0
    } else if camera_zoom >= Z_DISTRICT_THRESHOLD {
        LODLevel::L1
    } else {
        LODLevel::L2
    };

    // Entity count override: too many entities at L0 → fall back to L1
    let entity_lod = if entities_in_viewport > MAX_L0_ENTITIES {
        LODLevel::L1.max(zoom_lod)
    } else {
        zoom_lod
    };

    // FPS adaptive override: poor performance → increase LOD (less detail)
    if client_fps < FPS_LOD_FALLBACK_THRESHOLD {
        entity_lod.raise_one()  // L0 → L1; L1 → L2; L2 stays L2
    } else {
        entity_lod
    }
}

// Constants
const Z_CITIZEN_THRESHOLD: f32 = 4.0;   // Camera zoom factor for citizen-level
const Z_DISTRICT_THRESHOLD: f32 = 1.5;  // Camera zoom factor for district-level
const MAX_L0_ENTITIES: u32 = 500;        // Max entities before L0 becomes L1
const FPS_LOD_FALLBACK_THRESHOLD: f32 = 30.0; // FPS below this triggers LOD raise
```

### LOD Hysteresis

To prevent LOD thrashing at boundaries, hysteresis is applied:
- LOD raises immediately (performance/bandwidth priority)
- LOD lowers only after 10 consecutive ticks at the same zoom without triggering the raise condition

```rust
pub struct LODHysteresisState {
    pub current_lod: LODLevel,
    pub stable_ticks_at_current: u32,
    pub pending_lower: Option<LODLevel>,
}

impl LODHysteresisState {
    const HYSTERESIS_TICKS: u32 = 10;

    pub fn update(&mut self, desired_lod: LODLevel) -> LODLevel {
        if desired_lod > self.current_lod {
            // Raise immediately
            self.current_lod = desired_lod;
            self.stable_ticks_at_current = 0;
            self.pending_lower = None;
        } else if desired_lod < self.current_lod {
            // Lower only after hysteresis period
            self.stable_ticks_at_current += 1;
            if self.stable_ticks_at_current >= Self::HYSTERESIS_TICKS {
                self.current_lod = desired_lod;
                self.stable_ticks_at_current = 0;
            }
        } else {
            self.stable_ticks_at_current += 1;
        }
        self.current_lod
    }
}
```

## 4.4 Server-Side LOD Computation

The server computes LOD snapshots as part of Phase 6: Client Broadcast (see CIV-0001). LOD computation is isolated from simulation state mutation — it is a read-only projection of state.

### SnapshotFilter

Each client subscription includes a `SnapshotFilter` that tells the server exactly what to include in broadcasts:

```rust
pub struct SnapshotFilter {
    pub lod_level: LODLevel,
    pub viewport: HexRect,
    pub subscribed_entity_ids: Vec<EntityId>,  // explicit entity subscriptions (always included)
    pub max_entities: u32,                      // client-declared entity budget
    pub include_events: bool,                   // whether to include event list
    pub event_filter: Option<EventTypeFilter>,  // which event types to include
}

pub struct HexRect {
    pub center: HexCoord,
    pub radius: u32,  // hex radius (Manhattan distance)
}

impl HexRect {
    pub fn contains(&self, hex: HexCoord) -> bool {
        hex_manhattan_distance(self.center, hex) <= self.radius
    }
}
```

### Server LOD Computation per Client

```rust
fn compute_client_snapshot(
    world: &World,
    filter: &SnapshotFilter,
    tick: u64,
) -> ClientSnapshot {
    match filter.lod_level {
        LODLevel::L2 => compute_l2_snapshot(world, &filter.viewport, tick),
        LODLevel::L1 => compute_l1_snapshot(world, &filter.viewport, tick),
        LODLevel::L0 => compute_l0_snapshot(
            world,
            &filter.viewport,
            &filter.subscribed_entity_ids,
            filter.max_entities,
            tick,
        ),
    }
}

fn compute_l2_snapshot(world: &World, viewport: &HexRect, tick: u64) -> ClientSnapshot {
    // Query all regions with centroid in viewport
    let regions: Vec<L2RegionSnapshot> = world
        .query::<&RegionComponent>()
        .iter(world)
        .filter(|r| viewport.contains(r.centroid))
        .map(|r| aggregate_region_to_l2(world, r, tick))
        .collect();

    ClientSnapshot::L2 { regions, tick }
}

fn compute_l1_snapshot(world: &World, viewport: &HexRect, tick: u64) -> ClientSnapshot {
    // Query all districts with any hex in viewport
    let districts: Vec<L1DistrictSnapshot> = world
        .query::<&DistrictComponent>()
        .iter(world)
        .filter(|d| viewport.intersects(&d.hex_bounds))
        .map(|d| aggregate_district_to_l1(world, d, tick))
        .collect();

    ClientSnapshot::L1 { districts, tick }
}

fn compute_l0_snapshot(
    world: &World,
    viewport: &HexRect,
    explicit_ids: &[EntityId],
    max_entities: u32,
    tick: u64,
) -> ClientSnapshot {
    // Combine viewport citizens + explicit subscriptions, up to budget
    let mut citizens: Vec<L0CitizenSnapshot> = world
        .query::<(&CitizenComponent, &PositionComponent)>()
        .iter(world)
        .filter(|(_, pos)| viewport.contains(pos.hex))
        .take(max_entities as usize)
        .map(|(c, pos)| project_citizen_to_l0(c, pos, tick))
        .collect();

    // Add explicit subscriptions (not subject to viewport or count limit)
    for &entity_id in explicit_ids {
        if let Some((citizen, pos)) = world.get_components::<(&CitizenComponent, &PositionComponent)>(entity_id) {
            citizens.push(project_citizen_to_l0(citizen, pos, tick));
        }
    }

    ClientSnapshot::L0 { citizens, tick }
}
```

### LOD Computation Budgets (Server-Side)

| LOD Level | Max Entities | Compute Target | Notes |
|-----------|-------------|----------------|-------|
| L2 | All regions in viewport (~10–50) | < 1 ms | Simple aggregation |
| L1 | All districts in viewport (~20–200) | < 3 ms | Moderate aggregation |
| L0 | Up to 500 citizens | < 10 ms | Full component projection |

These budgets are enforced via `max_entities` in `SnapshotFilter` and pre-tick timing checks.

## 4.5 Client-Server LOD Negotiation Protocol

Clients negotiate their LOD preference at subscription time and whenever their camera changes significantly.

### Initial Subscription

```json
{
  "jsonrpc": "2.0",
  "method": "sim.subscribe",
  "params": {
    "lod_level": 1,
    "viewport": { "center": {"q": 10, "r": 5}, "radius": 20 },
    "max_entities": 200,
    "include_events": true,
    "event_filter": ["war.*", "disaster.*", "economy.*", "tech.*"]
  },
  "id": 1
}
```

Server response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscribed": true,
    "effective_lod": 1,
    "viewport_accepted": true,
    "broadcast_cadence_ms": 50,
    "initial_snapshot": { ... }
  },
  "id": 1
}
```

### LOD Update (on camera change)

Client sends viewport update when camera moves beyond threshold:

```json
{
  "jsonrpc": "2.0",
  "method": "sim.update_subscription",
  "params": {
    "lod_level": 2,
    "viewport": { "center": {"q": 20, "r": 8}, "radius": 50 },
    "max_entities": 50
  },
  "id": 42
}
```

**Threshold for sending update:** Camera center moves > 5 hexes OR zoom factor changes by > 0.5.

Smaller movements are handled client-side by the renderer (interpolation / culling) without triggering a re-subscription.

### Server LOD Override

If the server is under load, it may override the client's requested LOD to a higher (less detailed) level:

```json
{
  "jsonrpc": "2.0",
  "method": "sim.lod_override",
  "params": {
    "effective_lod": 2,
    "reason": "server_load",
    "restore_tick": 15000
  }
}
```

Client must accept the override and update its rendering accordingly. The override is temporary; `restore_tick` indicates when the client may re-request its preferred LOD.

---

# Section 5: RTS Client-Side Prediction — Full Specification

## 5.1 Prediction Motivation and Scope

Client-side prediction applies to all commands that have a visible effect within 1–3 ticks of issuance. The goal is to eliminate the perceived 50–200 ms lag between player input and visual response on screen.

**Scope of prediction:**
- `move_unit` — unit position prediction (primary use case)
- `attack_target` — unit attack animation starts immediately
- `build_structure` — construction animation starts immediately (ghost building shown)
- `demolish_structure` — structure begins fade immediately

**Out of scope for prediction:**
- Diplomatic commands (no immediate visual; wait for server)
- Economy commands (`set_tax_rate`, etc.) — no immediate visual
- Administrative commands (`reassign_jobs`, etc.)

## 5.2 Command Sequence Numbering

Every command issued by a client carries a monotonically increasing `sequence_number`:

```rust
pub struct RtsCommand {
    pub sequence_number: u64,       // monotonic, per-client
    pub client_id: ClientId,
    pub tick_issued: u64,           // tick at time of issuance (client's known tick)
    pub command: CommandPayload,
    pub predicted_outcome: Option<PredictedOutcome>,  // client's prediction, optional
}
```

The server includes ACK information in snapshots:

```rust
pub struct TickBroadcast {
    pub tick: u64,
    pub snapshot: ClientSnapshot,
    pub events: Vec<SimulationEvent>,
    pub acked_sequences: Vec<(ClientId, u64)>,  // last processed seq per client
    pub rejected_sequences: Vec<(ClientId, u64, RejectionReason)>,
}
```

The client uses `acked_sequences` to determine which predictions have been server-confirmed and which are still unresolved.

## 5.3 MoveUnit Prediction in Full Detail

### Step 1: Command Issuance

```
Player input: right-click on hex (150, 200) at wall-clock t=0, simulation tick=5000
Client state: unit_42 at position (100, 150), speed=10 hexes/tick on flat terrain
```

The client immediately:
1. Assigns sequence_number = 1001 to this command
2. Records prediction: `predicted_positions[1001] = [(tick=5001, (102, 152)), (tick=5002, (104, 154)), ...]`
3. Begins animating unit_42 toward (150, 200) at local prediction speed
4. Sends command to server: `{seq: 1001, command: move_unit(42, (150, 200))}`

### Step 2: Pathfinding for Prediction

Client runs A* on the local hex grid to compute the predicted path:

```rust
fn predict_move_path(
    unit: &UnitSnapshot,
    target: HexCoord,
    terrain_map: &TerrainMap,
) -> Vec<(u64, HexCoord)> {
    let path = astar_hex(unit.position, target, terrain_map, unit.unit_type);
    let speed = unit.speed_hexes_per_tick(terrain_map, &path);

    path.iter()
        .enumerate()
        .map(|(step, &hex)| {
            let tick_offset = (step as f32 / speed).ceil() as u64;
            (unit.last_known_tick + tick_offset, hex)
        })
        .collect()
}
```

The A* algorithm on the client MUST use the same hex cost function as the server (terrain penalties are public knowledge, encoded in the map state). This ensures client and server converge on the same path.

### Step 3: Server Reconciliation

When the server snapshot arrives containing `acked_sequences = [(client_1, 1001)]`:

```rust
fn reconcile_movement(
    prediction_log: &PredictionLog,
    server_snapshot: &ClientSnapshot,
    acked_seq: u64,
) -> ReconciliationResult {
    let prediction = prediction_log.get(acked_seq);
    let server_pos = server_snapshot.unit_position(prediction.entity_id);
    let predicted_pos = prediction.position_at_tick(server_snapshot.tick);

    let delta = hex_manhattan_distance(predicted_pos, server_pos);

    match delta {
        0 => ReconciliationResult::Perfect,
        1..=1 => ReconciliationResult::SmoothCorrect {
            from: predicted_pos,
            to: server_pos,
            duration_ms: 50,
        },
        2..=5 => ReconciliationResult::SnapCorrect {
            to: server_pos,
            visual_effect: SnapEffect::BriefFlash,
        },
        _ => ReconciliationResult::HardDesync {
            to: server_pos,
            request_full_sync: true,
        },
    }
}
```

### Step 4: Correction Rendering

| Delta | Action | Visual |
|-------|--------|--------|
| 0 hexes | No correction | Unit continues smoothly |
| 1 hex | Smooth interpolation over 50 ms | Nearly imperceptible drift |
| 2–5 hexes | Hard snap + brief positional flash | Visible but quick (< 100 ms) |
| > 5 hexes | Hard snap + full sync request | Unit teleports; may show "syncing..." indicator |

## 5.4 Prediction Rollback

If the server rejects a command (illegal move, insufficient resources, unauthorized), the client receives a rejection notification:

```json
{
  "type": "command_rejected",
  "sequence_number": 1001,
  "reason": "terrain_impassable",
  "detail": "hex (120, 160) is water; unit type INFANTRY cannot cross"
}
```

The client rolls back the prediction:

```rust
fn rollback_prediction(
    entity_id: EntityId,
    rejected_seq: u64,
    prediction_log: &mut PredictionLog,
    render_state: &mut RenderState,
) {
    let prediction = prediction_log.remove(rejected_seq);
    let rollback_position = prediction.position_before_command;

    // Snap entity back to pre-command position
    render_state.snap_entity_to(entity_id, rollback_position);

    // Show error indicator (red flash on entity)
    render_state.show_error_indicator(entity_id, RejectionEffect::RedFlash);

    // Remove any pending predictions that depended on this one
    prediction_log.remove_dependent_on(rejected_seq);
}
```

## 5.5 Multi-Command Prediction Queue

Players often issue multiple commands in rapid succession (move unit A, then attack with unit B, then garrison unit C). The prediction system maintains a queue of unresolved predictions:

```rust
pub struct PredictionQueue {
    pub pending: VecDeque<PendingPrediction>,
    pub max_unresolved: u32,  // = 50; reject new commands if queue full
}

pub struct PendingPrediction {
    pub sequence_number: u64,
    pub issued_tick: u64,
    pub entity_id: EntityId,
    pub predicted_state: PredictedEntityState,
    pub depends_on: Option<u64>,  // sequence_number of prerequisite command
}
```

**Dependency tracking:** If command 1002 (attack target) depends on command 1001 (move into range), and command 1001 is rolled back, command 1002 is also automatically rolled back.

## 5.6 BuildStructure Prediction

Build commands show a "ghost" structure immediately:

```rust
fn predict_build_structure(
    command: &BuildStructureCommand,
    render_state: &mut RenderState,
) {
    // Show ghost at target location immediately
    render_state.add_ghost_structure(
        command.structure_type,
        command.location,
        GhostStyle::TranslucentBlue,  // "pending construction"
    );
    // Ghost persists until server ACK or rejection
}

fn confirm_build_structure(
    command_seq: u64,
    server_structure_id: StructureId,
    render_state: &mut RenderState,
) {
    // Replace ghost with real structure
    render_state.materialize_ghost(command_seq, server_structure_id);
}

fn rollback_build_structure(
    command_seq: u64,
    render_state: &mut RenderState,
) {
    render_state.remove_ghost(command_seq);
    render_state.show_error_indicator_at(command_seq, RejectionEffect::RedFlash);
}
```

---

# Section 6: Entity Culling Budget

## 6.1 Overview

The culling system ensures that only entities the client will actually render are processed. Culling happens client-side before any rendering or game logic update. The culling budget is a hard ceiling on entities processed per frame.

## 6.2 Viewport Culling

Only entities within the camera viewport plus a 3-tile margin are considered for rendering. The margin ensures entities at the viewport edge do not pop in/out abruptly.

```
Camera viewport: HexRect { center, radius }
Culling frustum: HexRect { center, radius + 3 }
```

All entities outside the culling frustum are immediately skipped. This is the cheapest possible cull — O(1) per entity using AABB intersection.

### Hex AABB Intersection

For a hex at coordinate (q, r) and a viewport rect with center (cq, cr) and radius R:

```rust
fn hex_in_viewport(hex: HexCoord, viewport: &HexRect, margin: u32) -> bool {
    let effective_radius = viewport.radius + margin;
    hex_manhattan_distance(hex, viewport.center) <= effective_radius
}

fn hex_manhattan_distance(a: HexCoord, b: HexCoord) -> u32 {
    // Axial coordinate hex distance
    ((a.q - b.q).abs() + (a.r - b.r).abs() + (a.q + a.r - b.q - b.r).abs()) as u32 / 2
}
```

## 6.3 Culling Tiers

Entities that pass viewport culling are assigned to one of three rendering tiers:

| Tier | Distance from Camera Center | Rendering Mode | Notes |
|------|----------------------------|----------------|-------|
| Visible | 0 – radius | Full render | All components rendered |
| Near-visible | radius – radius+3 | Simplified render | Sprites only; no UI overlay, no health bars |
| Distant | > radius+3 | Aggregate only | Not rendered individually; contributes to district aggregate indicator |

```rust
pub enum CullingTier {
    Visible,
    NearVisible,
    Distant,
}

pub fn assign_culling_tier(
    entity_hex: HexCoord,
    viewport: &HexRect,
) -> CullingTier {
    let dist = hex_manhattan_distance(entity_hex, viewport.center);
    if dist <= viewport.radius {
        CullingTier::Visible
    } else if dist <= viewport.radius + 3 {
        CullingTier::NearVisible
    } else {
        CullingTier::Distant
    }
}
```

## 6.4 Budget Enforcement

**Maximum renderable entities:** `MAX_RENDER_ENTITIES = 10_000`

If visible entity count exceeds this budget, entities are prioritized:

```
Priority order (highest to lowest):
  1. Player's own units (always rendered)
  2. Enemy units (within viewport)
  3. Friendly NPC units
  4. Structures (player-owned first)
  5. Neutral structures
  6. Resources / terrain decorations
```

```rust
pub fn apply_render_budget(
    mut visible_entities: Vec<(EntityId, RenderPriority)>,
    max: usize,
) -> Vec<EntityId> {
    if visible_entities.len() <= max {
        return visible_entities.into_iter().map(|(id, _)| id).collect();
    }

    // Sort by priority (highest first), then by entity_id for determinism
    visible_entities.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
    });

    visible_entities
        .into_iter()
        .take(max)
        .map(|(id, _)| id)
        .collect()
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub enum RenderPriority {
    TerrainDecoration = 0,
    NeutralStructure = 1,
    Resource = 2,
    NpcStructure = 3,
    PlayerStructure = 4,
    NpcUnit = 5,
    EnemyUnit = 6,
    PlayerUnit = 7,
}
```

## 6.5 Occlusion: Painter's Algorithm for 2D

CivLab uses a 2D isometric (or top-down hex) renderer. Occlusion between buildings and units behind them is handled via painter's algorithm (back-to-front rendering order):

```rust
pub fn sort_entities_for_painter(
    entities: &mut Vec<(EntityId, HexCoord, RenderLayer)>,
) {
    // Sort by: render layer first (ground < buildings < units < UI)
    // Within same layer: sort by row (r coordinate) descending for isometric
    entities.sort_by(|a, b| {
        a.2.cmp(&b.2).then_with(|| b.1.r.cmp(&a.1.r))
    });
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub enum RenderLayer {
    Ground = 0,
    TerrainFeature = 1,
    BuildingBase = 2,
    BuildingTop = 3,
    Unit = 4,
    Projectile = 5,
    UI = 6,
}
```

Buildings in the same hex as units sort above units in the render order, achieving natural occlusion without GPU z-buffer.

## 6.6 Frustum Culling Math Summary

Full viewport frustum check per entity:

```
Input: entity_hex (q, r), camera_center_hex (cq, cr), camera_radius R
Effective culling radius: R_eff = R + 3  (margin)

Step 1: Axial distance
  dq = |q - cq|
  dr = |r - cr|
  ds = |q + r - cq - cr|
  d_hex = (dq + dr + ds) / 2

Step 2: In frustum?
  in_frustum = (d_hex <= R_eff)

Step 3: Tier assignment
  if d_hex <= R:         Visible
  else if d_hex <= R+3:  NearVisible
  else:                  Distant (culled)
```

Cost: 6 integer subtractions + 3 abs + 2 comparisons per entity. For 10,000 entities: ~60,000 operations → < 0.1 ms on modern hardware.

---

# Section 7: Server Authoritative Correction Protocol

## 7.1 Tick Execution and Command Conflict Resolution

The server processes all commands from all clients in each tick's Command Intake phase (Phase 1, CIV-0001). When multiple clients issue commands that affect the same entity, conflicts are resolved deterministically.

### Conflict Resolution Rules

```
Priority 1: Human player commands (higher priority than AI)
Priority 2: Earlier sequence_number (tie-break within same priority)
Priority 3: Lower client_id (final tie-break for identical timestamps)
```

```rust
pub fn resolve_command_conflicts(
    mut commands: Vec<RtsCommand>,
) -> Vec<RtsCommand> {
    // Group by affected entity
    let mut by_entity: HashMap<EntityId, Vec<RtsCommand>> = HashMap::new();
    for cmd in commands {
        let entity_id = cmd.primary_affected_entity();
        by_entity.entry(entity_id).or_default().push(cmd);
    }

    let mut resolved = Vec::new();
    for (_, mut entity_cmds) in by_entity {
        // Sort by priority rules
        entity_cmds.sort_by(|a, b| {
            b.client_priority()
                .cmp(&a.client_priority())
                .then_with(|| a.sequence_number.cmp(&b.sequence_number))
                .then_with(|| a.client_id.cmp(&b.client_id))
        });

        // Accept highest priority command; reject others
        resolved.push(entity_cmds.remove(0));
        for rejected_cmd in entity_cmds {
            resolved.push(RtsCommand {
                status: CommandStatus::Rejected(RejectionReason::ConflictLostPriority),
                ..rejected_cmd
            });
        }
    }

    resolved
}
```

**Both clients are notified:** The winning client receives confirmation; the losing client receives a rejection with reason `ConflictLostPriority`.

## 7.2 Snapshot ACK Protocol

Every server broadcast includes the last acknowledged sequence number per client. This allows clients to prune their prediction queues.

```rust
pub struct TickBroadcastHeader {
    pub tick: u64,
    pub server_state_hash: u64,              // hash of full authoritative state
    pub acked_sequences: Vec<ClientAck>,     // one entry per subscribed client
    pub rejected_sequences: Vec<ClientReject>,
}

pub struct ClientAck {
    pub client_id: ClientId,
    pub last_acked_sequence: u64,            // all seqs <= this are confirmed
}

pub struct ClientReject {
    pub client_id: ClientId,
    pub sequence_number: u64,
    pub reason: RejectionReason,
    pub detail: String,
}
```

Client prediction pruning on receiving ACK:

```rust
fn prune_prediction_queue(
    queue: &mut PredictionQueue,
    acked_seq: u64,
    rejected: &[ClientReject],
    my_client_id: ClientId,
) {
    // Remove all confirmed predictions
    queue.pending.retain(|p| p.sequence_number > acked_seq);

    // Roll back rejected predictions
    for reject in rejected {
        if reject.client_id == my_client_id {
            queue.rollback_prediction(reject.sequence_number, reject.reason);
        }
    }
}
```

## 7.3 Desync Detection

Each client independently computes a rolling hash of its local state. The server includes its authoritative state hash in every broadcast. The client compares hashes and requests a full sync on mismatch.

### Hash Computation (Client)

```rust
fn compute_local_state_hash(
    units: &[UnitSnapshot],
    structures: &[StructureSnapshot],
    cohorts: &[CohortSummary],
) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();

    // Hash must be deterministic: sort by entity ID before hashing
    let mut sorted_units = units.to_vec();
    sorted_units.sort_by_key(|u| u.unit_id);
    for unit in &sorted_units {
        unit.hash(&mut hasher);
    }

    let mut sorted_structures = structures.to_vec();
    sorted_structures.sort_by_key(|s| s.structure_id);
    for structure in &sorted_structures {
        structure.hash(&mut hasher);
    }

    // Cohorts are stable-sorted by cohort_id
    let mut sorted_cohorts = cohorts.to_vec();
    sorted_cohorts.sort_by_key(|c| c.cohort_id);
    for cohort in &sorted_cohorts {
        cohort.hash(&mut hasher);
    }

    hasher.finish()
}
```

### Hash Comparison and Full Sync Request

```rust
fn check_for_desync(
    local_hash: u64,
    server_hash: u64,
    client_id: ClientId,
    ws: &WebSocketConnection,
) {
    if local_hash != server_hash {
        warn!("State hash mismatch: local={:#x}, server={:#x}", local_hash, server_hash);

        // Request full state resync
        ws.send_json(&json!({
            "jsonrpc": "2.0",
            "method": "sim.request_full_sync",
            "params": {
                "client_id": client_id,
                "last_known_tick": current_tick(),
                "reason": "hash_mismatch"
            },
            "id": next_rpc_id()
        }));

        // Show sync indicator in UI
        ui_state.show_sync_indicator("Synchronizing...");
    }
}
```

### Server Full Sync Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "type": "full_sync",
    "tick": 5100,
    "full_snapshot": {
      "lod_level": 1,
      "districts": [ ... ],
      "events_since_last_sync": [ ... ]
    },
    "state_hash": "0xdeadbeef12345678"
  },
  "id": 99
}
```

The client applies the full snapshot, discards all pending predictions (they were based on now-invalid state), and resumes normal operation.

## 7.4 State Hash Implementation Notes

The hash is computed over the **rendered state** (the LOD snapshot), not over the full simulation state. This is because:
1. The client only has access to the LOD snapshot, not the full server state
2. The LOD snapshot is deterministic given the same server state → the hash is a valid desync detector for the data the client actually uses

**Hash stability requirement:** The hash must produce identical output for identical inputs regardless of platform (x86/ARM, little-endian/big-endian). Use a platform-independent hasher (FNV-1a or xxHash) in production; the `DefaultHasher` above is acceptable for testing only.

```rust
use fnv::FnvHasher;

fn compute_stable_state_hash(snapshot: &ClientSnapshot) -> u64 {
    let mut hasher = FnvHasher::default();
    // ... (same logic as above, using FNV-1a)
    hasher.finish()
}
```

---

# Section 8: Benchmark Targets for LOD System

## 8.1 Server-Side Benchmarks

All benchmarks measured on reference hardware: 8-core modern server CPU, 16 GB RAM, NVMe storage.

| Operation | Target | Method | Notes |
|-----------|--------|--------|-------|
| L2 snapshot compute | < 1 ms | Criterion.rs benchmark | 50 regions in viewport |
| L1 snapshot compute | < 3 ms | Criterion.rs benchmark | 200 districts in viewport |
| L0 snapshot compute | < 10 ms | Criterion.rs benchmark | 1,000 entities in viewport |
| Snapshot filter application | < 0.5 ms per client | Criterion.rs | 100 concurrent clients |
| Command conflict resolution | < 0.1 ms | Per-tick timing | 100 commands from 10 clients |
| State hash computation | < 0.2 ms | Criterion.rs | L1 snapshot, 200 districts |
| Full sync serialization | < 50 ms | End-to-end test | Full L1 snapshot, JSON |

## 8.2 Client-Side Benchmarks

| Operation | Target | Method | Notes |
|-----------|--------|--------|-------|
| Client prediction update | < 1 ms per command | Profiler | A* pathfinding on 100-node graph |
| Prediction reconciliation | < 0.5 ms per tick | Profiler | 50 pending predictions |
| State hash comparison | < 0.1 ms | Profiler | L1 snapshot |
| Viewport culling pass | < 0.1 ms | Profiler | 10,000 entities |
| Painter's algorithm sort | < 0.5 ms | Profiler | 2,000 visible entities |
| LOD level selection | < 0.01 ms | Profiler | Simple math, no allocation |
| Ambient audio update on camera move | < 1 ms | Profiler | 8 audio layer volume updates |

## 8.3 Network Benchmarks

| Metric | Target | Notes |
|--------|--------|-------|
| L2 broadcast size | < 5 KB per tick per client | 10 regions × 0.5 KB |
| L1 broadcast size | < 40 KB per tick per client | 20 districts × 2 KB |
| L0 broadcast size | < 60 KB per on-demand response | 500 citizens × 0.1 KB |
| Full sync payload | < 500 KB | L1 full viewport; compressed |
| Snapshot compression ratio | > 3:1 | Using LZ4 or zstd |

**Compression:** L1/L2 snapshots are JSON by default. For the binary frame path, snapshots are encoded as MessagePack + LZ4, achieving 3–5x compression over JSON.

## 8.4 End-to-End Latency Targets

| Scenario | Target | Measurement |
|----------|--------|-------------|
| Command ACK (client → server → client) | < 50 ms (p95) | Over localhost; < 100 ms over WiFi |
| Full sync receive | < 100 ms | From request to full snapshot received |
| Full sync render | < 200 ms | From sync received to full render complete |
| LOD transition visual | < 250 ms | Camera zoom change to new LOD fully rendered |
| Prediction rollback render | < 100 ms | Rejection to entity in correct position |

## 8.5 Acceptance Test Suite

```rust
#[cfg(test)]
mod lod_benchmarks {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};

    fn bench_l2_snapshot(c: &mut Criterion) {
        let world = build_benchmark_world(50, 0, 0);  // 50 regions, no districts, no citizens
        let viewport = HexRect { center: HexCoord::ZERO, radius: 100 };

        c.bench_function("l2_snapshot_50_regions", |b| {
            b.iter(|| {
                compute_l2_snapshot(&world, &viewport, 0)
            })
        });
    }

    fn bench_l1_snapshot(c: &mut Criterion) {
        let world = build_benchmark_world(10, 200, 0);  // 10 regions, 200 districts
        let viewport = HexRect { center: HexCoord::ZERO, radius: 50 };

        c.bench_function("l1_snapshot_200_districts", |b| {
            b.iter(|| {
                compute_l1_snapshot(&world, &viewport, 0)
            })
        });
    }

    fn bench_l0_snapshot(c: &mut Criterion) {
        let world = build_benchmark_world(1, 1, 1000);  // 1 region, 1 district, 1000 citizens
        let viewport = HexRect { center: HexCoord::ZERO, radius: 20 };

        c.bench_function("l0_snapshot_1000_citizens", |b| {
            b.iter(|| {
                compute_l0_snapshot(&world, &viewport, &[], 1000, 0)
            })
        });
    }

    fn bench_prediction_update(c: &mut Criterion) {
        let terrain = build_flat_terrain_map(200, 200);
        let unit = UnitSnapshot {
            unit_id: EntityId(1),
            position: HexCoord { q: 0, r: 0 },
            speed: 10.0,
            unit_type: UnitType::Infantry,
            last_known_tick: 0,
        };
        let target = HexCoord { q: 50, r: 30 };

        c.bench_function("predict_move_path_100node", |b| {
            b.iter(|| {
                predict_move_path(&unit, target, &terrain)
            })
        });
    }

    fn bench_viewport_culling(c: &mut Criterion) {
        let entities: Vec<HexCoord> = (0..10_000)
            .map(|i| HexCoord { q: i % 200, r: i / 200 })
            .collect();
        let viewport = HexRect { center: HexCoord { q: 100, r: 25 }, radius: 30 };

        c.bench_function("viewport_culling_10k_entities", |b| {
            b.iter(|| {
                entities.iter()
                    .filter(|&&hex| hex_in_viewport(hex, &viewport, 3))
                    .count()
            })
        });
    }

    fn bench_full_sync(c: &mut Criterion) {
        let world = build_benchmark_world(10, 100, 0);
        let filter = SnapshotFilter {
            lod_level: LODLevel::L1,
            viewport: HexRect { center: HexCoord::ZERO, radius: 100 },
            subscribed_entity_ids: vec![],
            max_entities: 500,
            include_events: true,
            event_filter: None,
        };

        c.bench_function("full_l1_snapshot_serialize", |b| {
            b.iter(|| {
                let snapshot = compute_client_snapshot(&world, &filter, 0);
                serde_json::to_string(&snapshot).unwrap()
            })
        });
    }

    criterion_group!(
        benches,
        bench_l2_snapshot,
        bench_l1_snapshot,
        bench_l0_snapshot,
        bench_prediction_update,
        bench_viewport_culling,
        bench_full_sync,
    );
    criterion_main!(benches);
}
```

## 8.6 Performance Monitoring in Production

At runtime, the server records per-tick LOD computation times and exposes them via metrics:

```
civ_lod_compute_ms{level="L2"} histogram
civ_lod_compute_ms{level="L1"} histogram
civ_lod_compute_ms{level="L0"} histogram
civ_snapshot_filter_ms histogram
civ_snapshot_broadcast_bytes{client_id, lod_level} histogram
civ_desync_events_total counter
civ_full_sync_requests_total counter
civ_prediction_rollback_total counter
```

Alerts fire when p95 of any LOD compute metric exceeds 1.5× its target for 60 consecutive seconds.

---

# Section 9: FR-CIV-GEO-010 Traceability

This section explicitly maps the CIV-0101 spec to FR-CIV-GEO-010 acceptance criteria.

| FR-CIV-GEO-010 Criterion | Fulfilled By | Section |
|--------------------------|-------------|---------|
| Zoom level 1: {region_id, aggregated_population, aggregated_resources, military_unit_count, dominant_institution, diplomatic_status} | `L2RegionSnapshot` struct | Section 4.2 |
| Zoom level 2: {district_id, population_cohorts, resource_stocks, structures, military_units, citizen_morale} | `L1DistrictSnapshot` struct | Section 4.2 |
| Zoom level 3 (research mode): {citizen_id, job, welfare, ideology, location, stress_score} | `L0CitizenSnapshot` struct | Section 4.2 |
| Data size: zoom 1 ~0.5 KB per region | L2 target \< 0.5 KB per region | Section 4.2, Section 8.3 |
| Data size: zoom 2 ~2 KB per district | L1 target ~2 KB per district | Section 4.2, Section 8.3 |
| Schema versioning: `lod_snapshots.schema_version` | Schema version in Section 3 | Section 3 |
| Client rejects mismatched versions | Desync detection hash comparison | Section 7.3 |
| `metrics.lod_snapshot.v1` emitted per zoom transition | SnapshotFilter update → new snapshot broadcast | Section 4.5 |
| Determinism: identical state → identical LOD snapshot | Pure read-only projection from deterministic ECS state | Section 4.4 |

---

# Section 10: Implementation Checklist

- [ ] `LODLevel` enum defined with L0/L1/L2 variants
- [ ] `L0CitizenSnapshot`, `L1DistrictSnapshot`, `L2RegionSnapshot` structs defined
- [ ] `SnapshotFilter` struct defined with all fields
- [ ] `compute_l0_snapshot`, `compute_l1_snapshot`, `compute_l2_snapshot` functions implemented
- [ ] `select_lod_level` function with zoom factor, entity count, FPS inputs
- [ ] `LODHysteresisState` implemented; prevents LOD thrashing at zoom boundaries
- [ ] `sim.subscribe` RPC accepts `SnapshotFilter` parameters
- [ ] `sim.update_subscription` RPC for camera movement updates
- [ ] Server LOD override message `sim.lod_override` sent under load
- [ ] `RtsCommand.sequence_number` field present and monotonic
- [ ] `TickBroadcastHeader.acked_sequences` populated each tick
- [ ] `TickBroadcastHeader.rejected_sequences` populated for rejected commands
- [ ] `predict_move_path` uses same A* cost function as server
- [ ] Prediction reconciliation handles 0/1/2–5/>5 hex delta cases
- [ ] `rollback_prediction` removes prediction and shows error indicator
- [ ] `PredictionQueue.depends_on` chain rollback implemented
- [ ] `predict_build_structure` / `confirm_build_structure` / `rollback_build_structure` implemented
- [ ] Viewport culling using hex Manhattan distance; margin = 3
- [ ] `CullingTier` assignment per entity; `MAX_RENDER_ENTITIES = 10_000`
- [ ] `apply_render_budget` with priority sort (player units > enemy > friendly > structures > decorations)
- [ ] Painter's algorithm sort by `RenderLayer` + `r` coordinate
- [ ] Conflict resolution: player > AI, then by sequence_number, then client_id
- [ ] State hash computed using stable hasher (FNV-1a in production)
- [ ] `sim.request_full_sync` RPC implemented on client and server
- [ ] Full sync response includes compressed L1 snapshot
- [ ] Criterion.rs benchmarks exist for all 6 benchmark scenarios
- [ ] Production metrics emitted: `civ_lod_compute_ms`, `civ_desync_events_total`, etc.
- [ ] All FR-CIV-GEO-010 acceptance criteria mapped in Section 9
