# CivLab Data Model and Database Specification

**Spec ID:** SPEC-DATA-MODEL-CIV-001
**Version:** 1.0.0
**Status:** ACTIVE
**Date:** 2026-02-21
**Owner:** CIV Architecture & Engine Team

**Related Specs:**
- `CIV-0001-core-simulation-loop.md` — Deterministic tick architecture, ECS World struct, .civreplay format
- `CIV-0100-economy-v1.md` — Economy module, double-entry ledger, market clearing, conservation invariants
- `CIV-0102-climate-resource-dynamics.md` — Climate state, tipping points, energy scarcity coupling
- `CIV-0103-institutions-governance.md` — Institution state, legitimacy, capture mechanics
- `CIV-0105-war-diplomacy.md` — War records, mobilization, sanctions
- `CIV-0107-joule-economy-system.md` — Citizen joule ledger, quota mechanics

---

## Table of Contents

1. Data Architecture Overview
   - 1.1 Storage Tier Responsibilities
   - 1.2 Storage Hierarchy and Data Flow
   - 1.3 In-Memory ECS World
   - 1.4 SQLite Embedded Store
   - 1.5 PostgreSQL Multi-User Research Mode
   - 1.6 .civreplay Binary Format
2. SQLite Schema DDL — Full Table Definitions
   - 2.1 `schema_versions` — Migration tracking
   - 2.2 `runs` — Simulation run metadata
   - 2.3 `snapshots` — Per-tick state snapshots
   - 2.4 `events` — Event log
   - 2.5 `nations` — Nation state per tick
   - 2.6 `cities` — City state per tick
   - 2.7 `citizens` — Citizen records per tick
   - 2.8 `ledger_transfers` — Double-entry bookkeeping
   - 2.9 `markets` — Market clearing per good per tick
   - 2.10 `climate_state` — Climate state per tick
   - 2.11 `institutions` — Institution state per tick
   - 2.12 `wars` — Conflict records
   - 2.13 `research_runs` — Research and scenario metadata
   - 2.14 `replay_events` — Compressed event stream
   - 2.15 `metrics_timeseries` — Aggregated metrics
   - 2.16 `rng_seeds` — RNG seed log for replay auditability
3. Indexes and Performance
   - 3.1 SQLite Index DDL
   - 3.2 SQLite PRAGMA Settings
   - 3.3 PostgreSQL Partitioning Strategy
   - 3.4 Query Plan Annotations
4. Rust Type Definitions and DDL Mapping
   - 4.1 Core Simulation Structs
   - 4.2 SQLx Type Mappings
   - 4.3 Custom Type Encodings
   - 4.4 Diesel Schema Macros (alternative)
5. Data Lifecycle and Retention
   - 5.1 Hot Window Policy
   - 5.2 Snapshot Policy
   - 5.3 Pruning and Archival
   - 5.4 Export Formats
6. Scenario and Parameter Schema
   - 6.1 JSON Schema Definition
   - 6.2 Validation Rules
   - 6.3 Scenario Registry
7. Conservation and Integrity Invariants
   - 7.1 Ledger Conservation Trigger
   - 7.2 BLAKE3 State Hash Chain
   - 7.3 Unique and Not-Null Constraints Summary
   - 7.4 Foreign Key Cascade Behavior
8. Migration Strategy
   - 8.1 `schema_versions` Table
   - 8.2 Migration File Conventions
   - 8.3 Backward-Compatible Migration Rules
9. Research Query Patterns
   - 9.1 Average Happiness by Class Over Time
   - 9.2 GDP Trajectory per Nation
   - 9.3 Market Price Volatility
   - 9.4 War Frequency Distribution
   - 9.5 Energy Balance vs. Stability Correlation
   - 9.6 Citizen Migration Flows
   - 9.7 Institution Legitimacy Decay
   - 9.8 Climate Shock Correlation with Economic Disruption
   - 9.9 Gini Coefficient Trajectory
   - 9.10 Tipping Point Activation Timeline
10. Test Harness
    - 10.1 Property Tests
    - 10.2 Round-Trip Tests
    - 10.3 Performance Benchmarks
    - 10.4 Test Fixtures

---

## 1. Data Architecture Overview

### 1.1 Storage Tier Responsibilities

CivLab uses three distinct storage tiers with strict separation of concerns. No tier substitutes for another. No silent fallback from one tier to another is permitted.

| Tier | Technology | Scope | Access Pattern | Notes |
|------|-----------|-------|---------------|-------|
| Hot simulation state | In-memory ECS `World` struct | Current tick + rollback buffer (64 ticks) | Direct struct field access, O(1) | Zero I/O; deterministic; dropped on process exit |
| Run results and history | SQLite (embedded) | Per-run database file | SQL reads/writes via SQLx | One file per run; WAL mode; portable |
| Multi-user research | PostgreSQL (optional) | Shared research database | SQL via SQLx; partitioned tables | Partitioned by tick range; RLS per user |
| Replay archive | `.civreplay` binary | Full event stream + seed | Sequential read/write; mmap | BLAKE3-chained; compressed with zstd |
| Scenario library | JSON files on disk | Scenario definitions | File read at load time; validated | Immutable after scenario is started |

### 1.2 Storage Hierarchy and Data Flow

```
Simulation Engine (Rust, in-process)
        │
        ├── ECS World (in-memory)
        │     ├── NationComponent map  (BTreeMap<NationId, NationState>)
        │     ├── CityComponent map    (BTreeMap<CityId, CityState>)
        │     ├── CitizenComponent map (BTreeMap<CitizenId, CitizenRecord>)
        │     ├── LedgerState          (BTreeMap<Currency, i64> per actor)
        │     ├── MarketState          (BTreeMap<Good, MarketClearing>)
        │     ├── ClimateState         (single struct, updated per tick)
        │     └── InstitutionMap       (BTreeMap<InstId, InstitutionState>)
        │
        ├── Tick boundary → SQLite write path
        │     ├── events INSERT (every event emitted during tick)
        │     ├── nations INSERT (full nation state snapshot per tick)
        │     ├── cities INSERT (full city state snapshot per tick)
        │     ├── ledger_transfers INSERT (all transfers per tick)
        │     ├── markets INSERT (market clearing per good per tick)
        │     ├── climate_state INSERT (climate snapshot per tick)
        │     ├── institutions INSERT (institution state per tick)
        │     └── snapshots INSERT (state hash + msgpack blob every 100 ticks)
        │
        ├── Replay event append → .civreplay file
        │     └── All raw events, compressed, BLAKE3-chained
        │
        └── Pruning job (background)
              ├── Citizens older than 500 ticks → compressed to .civreplay
              └── SQLite VACUUM after large prune
```

### 1.3 In-Memory ECS World

The simulation engine maintains a single `World` struct that is the authoritative source of truth for the current tick. This is not persisted between process restarts. The `World` is a pure in-memory ECS (Entity-Component-System) store with the following properties:

- All collections use `BTreeMap` for deterministic iteration order (required for conservation invariants and BLAKE3 hash computation).
- No `HashMap` is used in simulation-critical paths. Hash maps produce non-deterministic iteration order across platforms and seeds, which would break replay.
- All numeric quantities are `i64` fixed-point integers. No `f32` or `f64` is used for economic values. The unit is millijoules (mJ) for energy/wealth, and milliunits for population-derived quantities.
- The `World` carries a rollback buffer of the last 64 tick states to support client-side rewinding without database reads.

```rust
// In-memory ECS World — canonical definition (see src/sim/world.rs)
pub struct World {
    pub tick: u64,
    pub run_id: Uuid,
    pub rng: ChaCha20Rng,
    pub nations: BTreeMap<NationId, NationState>,
    pub cities: BTreeMap<CityId, CityState>,
    pub citizens: BTreeMap<CitizenId, CitizenRecord>,
    pub ledger: LedgerState,
    pub markets: BTreeMap<Good, MarketClearing>,
    pub climate: ClimateState,
    pub institutions: BTreeMap<InstId, InstitutionState>,
    pub wars: BTreeMap<WarId, WarRecord>,
    pub event_log: Vec<SimEvent>,        // flushed to SQLite at tick boundary
    pub rollback_buf: RingBuf<WorldSnap, 64>,
}
```

### 1.4 SQLite Embedded Store

Each simulation run produces exactly one SQLite database file: `runs/{run_id}.db`. This file is the durable record of everything that occurred in that run. It is written at tick boundaries, not mid-tick.

Key SQLite configuration (applied at connection open time, enforced by the `DbConn::open` wrapper — see Section 3.2):

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA cache_size = -65536;      -- 64 MiB (negative = KiB units)
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 8589934592;   -- 8 GiB
PRAGMA wal_autocheckpoint = 1000;
```

SQLite is the default storage backend and requires no external services. It is appropriate for single-user simulation, research batch runs, and scenario development.

### 1.5 PostgreSQL Multi-User Research Mode

When multiple researchers run scenarios concurrently against a shared dataset, the system can be configured to use PostgreSQL. This is controlled by the `CIVLAB_DB_URL` environment variable. If it starts with `postgres://`, the PostgreSQL path is used. If it starts with `sqlite://`, the SQLite path is used.

PostgreSQL-specific features used:
- `RANGE` partitioning on `tick` for the `snapshots`, `events`, `citizens`, and `metrics_timeseries` tables.
- Row-Level Security (RLS) keyed on `user_id` for `research_runs`.
- `pg_partman` for automatic partition creation.
- `BRIN` indexes on `tick` columns within each partition.

The schema DDL in Section 2 is written in a SQLite-compatible dialect. PostgreSQL-specific extensions are called out in `[POSTGRES ONLY]` annotations.

### 1.6 .civreplay Binary Format

The `.civreplay` file is a binary append-only log of all raw simulation events. It is the primary archival format and the authoritative source for full replay.

**File layout:**

```
+------------------+
|  Header (256 B)  |
|  magic: b"CIVR"  |
|  version: u16    |
|  run_id: [u8;16] |  -- UUID bytes
|  seed: u64       |
|  tick_count: u64 |
|  reserved: [u8]  |
+------------------+
|  Frame 0         |
|  frame_len: u32  |  -- LE
|  tick: u64       |  -- LE
|  seq: u64        |  -- LE
|  hash_prev: [u8;32] | -- BLAKE3 of previous frame bytes
|  payload_len: u32|
|  payload: [u8]   |  -- zstd-compressed MessagePack event bytes
|  hash_self: [u8;32] | -- BLAKE3 of (frame_len..payload end)
+------------------+
|  Frame 1         |
|  ...             |
+------------------+
```

The hash chain ensures that any tampering with the replay file is detectable: `frame[n].hash_prev` must equal `frame[n-1].hash_self`. The hash of the empty bytes is used for `frame[0].hash_prev`.

---

## 2. SQLite Schema DDL — Full Table Definitions

All DDL is valid SQLite 3.42+. PostgreSQL-specific additions are annotated. Foreign key enforcement requires `PRAGMA foreign_keys = ON` at connection time.

### 2.1 `schema_versions` — Migration Tracking

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
    version         INTEGER     NOT NULL PRIMARY KEY,
    description     TEXT        NOT NULL,
    applied_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    checksum        TEXT        NOT NULL  -- SHA-256 of the migration file content
);

-- Seed with initial migration
INSERT OR IGNORE INTO schema_versions (version, description, checksum)
VALUES (1, 'initial schema', 'placeholder-replaced-by-migration-tooling');
```

### 2.2 `runs` — Simulation Run Metadata

The `runs` table is the root entity. Every other table references a `run_id`. A run is immutable once its `status` reaches `completed` or `failed`.

```sql
CREATE TABLE IF NOT EXISTS runs (
    run_id          TEXT        NOT NULL PRIMARY KEY,  -- UUID v4, stored as TEXT
    scenario_id     TEXT        NOT NULL,              -- references scenario JSON (not FK; scenarios live on disk)
    seed            INTEGER     NOT NULL,              -- u64 ChaCha20Rng seed
    start_tick      INTEGER     NOT NULL DEFAULT 0,
    end_tick        INTEGER,                           -- NULL while running
    status          TEXT        NOT NULL DEFAULT 'running'
                                CHECK (status IN ('running', 'completed', 'failed', 'paused', 'archived')),
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    params          TEXT        NOT NULL DEFAULT '{}', -- JSON: scenario parameter overrides
    tick_duration_ms INTEGER    NOT NULL DEFAULT 100,  -- wall-clock ms per simulation tick
    version         TEXT        NOT NULL DEFAULT '1.0.0', -- engine version that produced this run
    notes           TEXT                               -- free-text researcher annotation
);

CREATE INDEX IF NOT EXISTS idx_runs_scenario_id  ON runs (scenario_id);
CREATE INDEX IF NOT EXISTS idx_runs_status       ON runs (status);
CREATE INDEX IF NOT EXISTS idx_runs_created_at   ON runs (created_at);
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SimRun {
    pub run_id: String,           // Uuid serialized as hyphenated string
    pub scenario_id: String,
    pub seed: i64,                // u64 stored as i64 (SQLite INTEGER is signed 64-bit)
    pub start_tick: i64,
    pub end_tick: Option<i64>,
    pub status: RunStatus,
    pub created_at: String,       // ISO-8601 UTC
    pub updated_at: String,
    pub params: serde_json::Value,
    pub tick_duration_ms: i64,
    pub version: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Paused,
    Archived,
}
```

### 2.3 `snapshots` — Per-Tick State Snapshots

Full world snapshots are written every 100 ticks. Delta snapshots are written every tick (containing only changed components). The `is_full` flag distinguishes them.

```sql
CREATE TABLE IF NOT EXISTS snapshots (
    run_id          TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick            INTEGER     NOT NULL,
    is_full         INTEGER     NOT NULL DEFAULT 0 CHECK (is_full IN (0, 1)),
    state_hash      BLOB        NOT NULL,  -- BLAKE3 32-byte digest of canonical world bytes
    snapshot_bytes  BLOB        NOT NULL,  -- zstd-compressed MessagePack of full/delta world
    size_bytes      INTEGER     NOT NULL,  -- uncompressed size in bytes
    compressed_size INTEGER     NOT NULL,  -- compressed size in bytes
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (run_id, tick)
);

-- [POSTGRES ONLY] Partition by tick range (1000-tick windows):
-- CREATE TABLE snapshots (...) PARTITION BY RANGE (tick);
-- CREATE TABLE snapshots_tick_0    PARTITION OF snapshots FOR VALUES FROM (0)    TO (1000);
-- CREATE TABLE snapshots_tick_1000 PARTITION OF snapshots FOR VALUES FROM (1000) TO (2000);
-- ... managed by pg_partman

CREATE INDEX IF NOT EXISTS idx_snapshots_run_tick_full
    ON snapshots (run_id, tick)
    WHERE is_full = 1;
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Snapshot {
    pub run_id: String,
    pub tick: i64,
    pub is_full: bool,
    pub state_hash: Vec<u8>,       // 32 bytes, BLAKE3 digest
    pub snapshot_bytes: Vec<u8>,   // zstd-compressed msgpack
    pub size_bytes: i64,
    pub compressed_size: i64,
    pub created_at: String,
}
```

**Hash computation contract:**

The `state_hash` is computed as `blake3::hash(&canonical_bytes)` where `canonical_bytes` is the deterministic MessagePack serialization of the `World` struct with all maps sorted by key. The hash of tick `n+1` must be derivable from the hash of tick `n` plus the events applied during tick `n+1`. This chain is verified by the integrity checker (see Section 7.2).

### 2.4 `events` — Event Log

Every simulation event emitted during tick execution is appended to this table at the tick boundary. This is the primary audit trail and replay source for incremental replay (without reading full snapshots).

```sql
CREATE TABLE IF NOT EXISTS events (
    run_id          TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick            INTEGER     NOT NULL,
    event_id        TEXT        NOT NULL,   -- UUID v4
    event_type      TEXT        NOT NULL,   -- e.g. 'economy.ledger_transfer', 'war.declaration'
    payload         TEXT        NOT NULL,   -- JSON payload; schema defined per event_type
    seq             INTEGER     NOT NULL,   -- monotonically increasing within (run_id, tick)
    parent_event_id TEXT,                   -- NULL for root events; UUID for derived events
    phase           TEXT        NOT NULL DEFAULT 'unknown',
                                            -- tick phase: 'policy','production','market',
                                            --             'ledger','stochastic','climate'
    actor_id        TEXT,                   -- UUID of primary actor (nation, city, citizen, inst)
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (run_id, tick, event_id),
    UNIQUE (run_id, seq)  -- global sequence uniqueness per run
);

CREATE INDEX IF NOT EXISTS idx_events_run_tick
    ON events (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_events_run_type
    ON events (run_id, event_type);
CREATE INDEX IF NOT EXISTS idx_events_actor
    ON events (run_id, actor_id)
    WHERE actor_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_parent
    ON events (run_id, parent_event_id)
    WHERE parent_event_id IS NOT NULL;
```

**Event type taxonomy:**

| Prefix | Domain | Examples |
|--------|--------|---------|
| `economy.*` | Economy module | `economy.ledger_transfer`, `economy.market_cleared`, `economy.conservation_verified` |
| `climate.*` | Climate module | `climate.temp_updated`, `climate.tipping_point_activated`, `climate.sea_level_updated` |
| `war.*` | War module | `war.declaration`, `war.battle_resolved`, `war.peace_treaty` |
| `institution.*` | Institutions | `institution.policy_changed`, `institution.capture_level_changed`, `institution.legitimacy_updated` |
| `nation.*` | Nation | `nation.ideology_drift`, `nation.stability_updated`, `nation.population_updated` |
| `city.*` | City | `city.migration_flow`, `city.energy_balance_updated`, `city.food_balance_updated` |
| `citizen.*` | Citizen | `citizen.class_transition`, `citizen.employment_changed`, `citizen.happiness_updated` |
| `research.*` | Research events | `research.scenario_started`, `research.parameter_sweep_completed` |
| `sim.*` | Simulation control | `sim.tick_completed`, `sim.run_started`, `sim.run_completed`, `sim.snapshot_written` |

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SimEvent {
    pub run_id: String,
    pub tick: i64,
    pub event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub seq: i64,
    pub parent_event_id: Option<String>,
    pub phase: String,
    pub actor_id: Option<String>,
    pub created_at: String,
}
```

### 2.5 `nations` — Nation State Per Tick

One row per nation per tick. This is a full state snapshot of every nation at every persisted tick. Only ticks in the hot window (last 1000 ticks) have full rows; older ticks are pruned unless the run is a research run.

```sql
CREATE TABLE IF NOT EXISTS nations (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    nation_id           TEXT        NOT NULL,   -- UUID v4
    name                TEXT        NOT NULL,
    ideology_vector     TEXT        NOT NULL,   -- JSON array of 8 REAL values, each in [-1.0, 1.0]
                                                -- [planned_vs_market, authoritarian_vs_liberal,
                                                --  isolationist_vs_globalist, secular_vs_theocratic,
                                                --  militarist_vs_pacifist, ecoconservative_vs_extractivist,
                                                --  centralist_vs_federalist, technocratic_vs_traditionalist]
    stability           INTEGER     NOT NULL,   -- 0..10000 (fixed-point, divide by 100 for 0.00..100.00)
    legitimacy          INTEGER     NOT NULL,   -- 0..10000
    population_total    INTEGER     NOT NULL,   -- total population (integer count)
    population_growth   INTEGER     NOT NULL DEFAULT 0,  -- net change this tick (may be negative)
    gdp_millijoules     INTEGER     NOT NULL DEFAULT 0,  -- total economic output in mJ this tick
    energy_surplus_mj   INTEGER     NOT NULL DEFAULT 0,  -- net energy balance (positive = surplus)
    food_surplus_mu     INTEGER     NOT NULL DEFAULT 0,  -- net food balance in milliunits
    gini_coefficient    INTEGER     NOT NULL DEFAULT 0,  -- 0..10000 (x100 for 2 decimal places)
    at_war              INTEGER     NOT NULL DEFAULT 0 CHECK (at_war IN (0, 1)),
    capital_city_id     TEXT,                            -- UUID of capital city (NULL if no cities)

    PRIMARY KEY (run_id, tick, nation_id)
);

CREATE INDEX IF NOT EXISTS idx_nations_run_tick
    ON nations (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_nations_run_nation
    ON nations (run_id, nation_id);

-- [POSTGRES ONLY] RANGE partition on tick
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NationState {
    pub run_id: String,
    pub tick: i64,
    pub nation_id: String,
    pub name: String,
    pub ideology_vector: Vec<f64>,   // deserialized from JSON; length always 8
    pub stability: i64,              // 0..10000 fixed-point
    pub legitimacy: i64,
    pub population_total: i64,
    pub population_growth: i64,
    pub gdp_millijoules: i64,
    pub energy_surplus_mj: i64,
    pub food_surplus_mu: i64,
    pub gini_coefficient: i64,
    pub at_war: bool,
    pub capital_city_id: Option<String>,
}
```

### 2.6 `cities` — City State Per Tick

One row per city per tick. Cities are sub-entities of nations. All city-level economic accounting rolls up to the nation.

```sql
CREATE TABLE IF NOT EXISTS cities (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    city_id             TEXT        NOT NULL,   -- UUID v4
    nation_id           TEXT        NOT NULL,   -- UUID v4; references nations.nation_id
    name                TEXT        NOT NULL,
    position_x          INTEGER     NOT NULL,   -- grid x coordinate (integer tile)
    position_y          INTEGER     NOT NULL,   -- grid y coordinate (integer tile)
    population          INTEGER     NOT NULL,
    energy_balance_mj   INTEGER     NOT NULL,   -- net energy (positive = surplus, negative = deficit)
    food_balance_mu     INTEGER     NOT NULL,   -- net food in milliunits
    housing_capacity    INTEGER     NOT NULL,   -- maximum population this city can support
    employed_count      INTEGER     NOT NULL DEFAULT 0,
    unemployed_count    INTEGER     NOT NULL DEFAULT 0,
    happiness_avg       INTEGER     NOT NULL DEFAULT 5000,  -- 0..10000
    infrastructure_level INTEGER   NOT NULL DEFAULT 0,     -- 0..100 (integer percent)
    is_capital          INTEGER     NOT NULL DEFAULT 0 CHECK (is_capital IN (0, 1)),
    under_siege         INTEGER     NOT NULL DEFAULT 0 CHECK (under_siege IN (0, 1)),

    PRIMARY KEY (run_id, tick, city_id)
);

CREATE INDEX IF NOT EXISTS idx_cities_run_tick
    ON cities (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_cities_run_nation
    ON cities (run_id, nation_id, tick);
CREATE INDEX IF NOT EXISTS idx_cities_run_city
    ON cities (run_id, city_id);
```

### 2.7 `citizens` — Citizen Records Per Tick

The highest-volume table. One row per citizen per tick. For large simulations (>100k citizens), only every 10th tick is persisted unless the run is tagged as a research run. Rows older than 500 ticks are pruned to .civreplay unless the run is a research run (see Section 5.3).

```sql
CREATE TABLE IF NOT EXISTS citizens (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    citizen_id          TEXT        NOT NULL,   -- UUID v4
    city_id             TEXT        NOT NULL,   -- UUID v4
    nation_id           TEXT        NOT NULL,   -- UUID v4; denormalized for query efficiency
    happiness           INTEGER     NOT NULL,   -- 0..10000
    wealth_mj           INTEGER     NOT NULL,   -- net wealth in millijoules
    class_enum          TEXT        NOT NULL
                        CHECK (class_enum IN (
                            'subsistence', 'working', 'middle', 'professional',
                            'capitalist', 'elite', 'lumpenproletariat'
                        )),
    employment_status   TEXT        NOT NULL
                        CHECK (employment_status IN (
                            'employed', 'unemployed', 'self_employed',
                            'retired', 'student', 'disabled'
                        )),
    age_ticks           INTEGER     NOT NULL,   -- age in simulation ticks
    joule_quota_mj      INTEGER     NOT NULL DEFAULT 0,  -- current joule quota balance (CIV-0107)
    dissatisfaction     INTEGER     NOT NULL DEFAULT 0,  -- 0..10000; input to political instability
    migration_intent    INTEGER     NOT NULL DEFAULT 0 CHECK (migration_intent IN (0, 1)),

    PRIMARY KEY (run_id, tick, citizen_id)
);

CREATE INDEX IF NOT EXISTS idx_citizens_run_tick
    ON citizens (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_citizens_run_city_tick
    ON citizens (run_id, city_id, tick);
CREATE INDEX IF NOT EXISTS idx_citizens_run_nation_tick
    ON citizens (run_id, nation_id, tick);
CREATE INDEX IF NOT EXISTS idx_citizens_class_tick
    ON citizens (run_id, tick, class_enum);

-- Partial index for citizens with migration intent (common filter)
CREATE INDEX IF NOT EXISTS idx_citizens_migration_intent
    ON citizens (run_id, tick, city_id)
    WHERE migration_intent = 1;

-- [POSTGRES ONLY] Partition by tick, 1000-tick windows
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CitizenRecord {
    pub run_id: String,
    pub tick: i64,
    pub citizen_id: String,
    pub city_id: String,
    pub nation_id: String,
    pub happiness: i64,
    pub wealth_mj: i64,
    pub class_enum: CitizenClass,
    pub employment_status: EmploymentStatus,
    pub age_ticks: i64,
    pub joule_quota_mj: i64,
    pub dissatisfaction: i64,
    pub migration_intent: bool,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum CitizenClass {
    Subsistence,
    Working,
    Middle,
    Professional,
    Capitalist,
    Elite,
    Lumpenproletariat,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum EmploymentStatus {
    Employed,
    Unemployed,
    SelfEmployed,
    Retired,
    Student,
    Disabled,
}
```

### 2.8 `ledger_transfers` — Double-Entry Bookkeeping

Every resource transfer is recorded as a pair of ledger entries: one debit and one credit. The conservation invariant (Section 7.1) enforces that debits and credits net to zero for every currency per tick. This table is append-only; no updates or deletions are permitted.

```sql
CREATE TABLE IF NOT EXISTS ledger_transfers (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    transfer_id         TEXT        NOT NULL,   -- UUID v4; unique per transfer
    from_actor_id       TEXT        NOT NULL,   -- UUID of debited actor (nation, city, institution)
    from_actor_type     TEXT        NOT NULL
                        CHECK (from_actor_type IN ('nation', 'city', 'citizen', 'institution', 'market', 'void')),
    to_actor_id         TEXT        NOT NULL,   -- UUID of credited actor
    to_actor_type       TEXT        NOT NULL
                        CHECK (to_actor_type IN ('nation', 'city', 'citizen', 'institution', 'market', 'void')),
    amount_mj           INTEGER     NOT NULL CHECK (amount_mj >= 0),  -- always positive; direction implied by from/to
    currency_enum       TEXT        NOT NULL
                        CHECK (currency_enum IN ('joule', 'fiat', 'quota', 'labor_credit', 'carbon_credit')),
    transfer_type       TEXT        NOT NULL,   -- e.g. 'wage', 'tax', 'trade', 'subsidy', 'war_reparation'
    event_id            TEXT,                  -- references events.event_id that triggered this transfer
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (run_id, tick, transfer_id)
);

CREATE INDEX IF NOT EXISTS idx_ledger_run_tick
    ON ledger_transfers (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_ledger_run_from_actor
    ON ledger_transfers (run_id, from_actor_id, tick);
CREATE INDEX IF NOT EXISTS idx_ledger_run_to_actor
    ON ledger_transfers (run_id, to_actor_id, tick);
CREATE INDEX IF NOT EXISTS idx_ledger_currency_type
    ON ledger_transfers (run_id, tick, currency_enum, transfer_type);
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LedgerTransfer {
    pub run_id: String,
    pub tick: i64,
    pub transfer_id: String,
    pub from_actor_id: String,
    pub from_actor_type: ActorType,
    pub to_actor_id: String,
    pub to_actor_type: ActorType,
    pub amount_mj: i64,         // always >= 0; enforced by DB CHECK and Rust type invariant
    pub currency_enum: Currency,
    pub transfer_type: String,
    pub event_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Currency {
    Joule,
    Fiat,
    Quota,
    LaborCredit,
    CarbonCredit,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ActorType {
    Nation,
    City,
    Citizen,
    Institution,
    Market,
    Void,   // used for creation events (from Void) and destruction events (to Void)
}
```

### 2.9 `markets` — Market Clearing Per Good Per Tick

One row per (good, city) pair per tick. Records the outcome of the market clearing algorithm for that good in that city at that tick. This table is the primary source for price signal analysis.

```sql
CREATE TABLE IF NOT EXISTS markets (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    good_enum           TEXT        NOT NULL
                        CHECK (good_enum IN (
                            'energy', 'food', 'housing', 'medicine',
                            'capital_goods', 'consumer_goods', 'labor', 'carbon_credit'
                        )),
    city_id             TEXT        NOT NULL,   -- UUID v4; market is always city-scoped
    clearing_price_mj   INTEGER     NOT NULL,   -- price in millijoules per unit
    bid_volume          INTEGER     NOT NULL,   -- total demanded quantity
    ask_volume          INTEGER     NOT NULL,   -- total supplied quantity
    cleared_volume      INTEGER     NOT NULL,   -- quantity actually exchanged
    unmet_demand        INTEGER     NOT NULL DEFAULT 0,  -- bid_volume - cleared_volume (>=0)
    unmet_supply        INTEGER     NOT NULL DEFAULT 0,  -- ask_volume - cleared_volume (>=0)
    price_floor_active  INTEGER     NOT NULL DEFAULT 0 CHECK (price_floor_active IN (0, 1)),
    price_ceiling_active INTEGER   NOT NULL DEFAULT 0 CHECK (price_ceiling_active IN (0, 1)),
    regime              TEXT        NOT NULL DEFAULT 'market'
                        CHECK (regime IN ('market', 'planned', 'joule', 'hybrid')),

    PRIMARY KEY (run_id, tick, good_enum, city_id)
);

CREATE INDEX IF NOT EXISTS idx_markets_run_tick
    ON markets (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_markets_run_good
    ON markets (run_id, good_enum, tick);
CREATE INDEX IF NOT EXISTS idx_markets_run_city
    ON markets (run_id, city_id, tick);

-- Partial index for ticks with unmet demand (supply stress indicator)
CREATE INDEX IF NOT EXISTS idx_markets_unmet_demand
    ON markets (run_id, tick, good_enum)
    WHERE unmet_demand > 0;
```

### 2.10 `climate_state` — Climate State Per Tick

One row per tick for the global climate. Climate is not city-scoped or nation-scoped; it is a single global state. Local effects (sea level rise affecting coastal cities, drought affecting specific biomes) are derived from this table in the query layer.

```sql
CREATE TABLE IF NOT EXISTS climate_state (
    run_id                  TEXT    NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                    INTEGER NOT NULL,
    global_temp_offset_mc   INTEGER NOT NULL,  -- millicelsius above pre-industrial baseline
    co2_ppm_mc              INTEGER NOT NULL,  -- CO2 in milli-ppm (divide by 1000 for ppm)
    sea_level_rise_mm       INTEGER NOT NULL,  -- sea level rise in millimeters
    ocean_acidification_mu  INTEGER NOT NULL,  -- pH drop * 1000000 (microunits)
    arctic_ice_pct          INTEGER NOT NULL,  -- % of baseline ice coverage * 100
    active_tipping_points   TEXT    NOT NULL DEFAULT '[]',  -- JSON array of TippingPoint enum strings
    extreme_weather_count   INTEGER NOT NULL DEFAULT 0,     -- number of extreme weather events this tick
    renewable_capacity_pct  INTEGER NOT NULL DEFAULT 0,     -- % of global energy from renewables * 100

    PRIMARY KEY (run_id, tick)
);

CREATE INDEX IF NOT EXISTS idx_climate_run_tick
    ON climate_state (run_id, tick);
```

**Tipping point enum values (used in `active_tipping_points` JSON array):**

```
'west_antarctic_ice_sheet_collapse'
'greenland_ice_sheet_collapse'
'amazon_dieback'
'permafrost_methane_release'
'atlantic_circulation_collapse'
'coral_reef_die_off'
'boreal_forest_dieback'
'monsoon_disruption'
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClimateStateRow {
    pub run_id: String,
    pub tick: i64,
    pub global_temp_offset_mc: i64,   // millicelsius; divide by 1000 for °C
    pub co2_ppm_mc: i64,              // milli-ppm; divide by 1000 for ppm
    pub sea_level_rise_mm: i64,
    pub ocean_acidification_mu: i64,
    pub arctic_ice_pct: i64,          // * 100 fixed-point
    pub active_tipping_points: Vec<TippingPoint>,  // deserialized from JSON
    pub extreme_weather_count: i64,
    pub renewable_capacity_pct: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TippingPoint {
    WestAntarcticIceSheetCollapse,
    GreenlandIceSheetCollapse,
    AmazonDieback,
    PermafrostMethaneRelease,
    AtlanticCirculationCollapse,
    CoralReefDieOff,
    BorealForestDieback,
    MonsoonDisruption,
}
```

### 2.11 `institutions` — Institution State Per Tick

Institutions are formal organizations within nations: central banks, planning bureaus, regulatory agencies, courts, military commands, etc. They have budgets, legitimacy, and capture levels.

```sql
CREATE TABLE IF NOT EXISTS institutions (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    inst_id             TEXT        NOT NULL,   -- UUID v4
    inst_type           TEXT        NOT NULL
                        CHECK (inst_type IN (
                            'central_bank', 'planning_bureau', 'regulatory_agency',
                            'court', 'military_command', 'trade_union', 'religious_body',
                            'media_organization', 'environmental_agency', 'taxation_authority'
                        )),
    nation_id           TEXT        NOT NULL,   -- UUID v4; owning nation
    name                TEXT        NOT NULL,
    capture_level       INTEGER     NOT NULL DEFAULT 0,  -- 0..10000; 0=not captured, 10000=fully captured
    legitimacy          INTEGER     NOT NULL DEFAULT 5000,  -- 0..10000
    budget_mj           INTEGER     NOT NULL DEFAULT 0,    -- operating budget in millijoules
    budget_spent_mj     INTEGER     NOT NULL DEFAULT 0,    -- amount spent this tick
    policy_vector       TEXT        NOT NULL DEFAULT '{}', -- JSON: active policy settings
    autonomy_level      INTEGER     NOT NULL DEFAULT 5000, -- 0..10000; 0=captured, 10000=fully autonomous
    effectiveness       INTEGER     NOT NULL DEFAULT 5000, -- 0..10000

    PRIMARY KEY (run_id, tick, inst_id)
);

CREATE INDEX IF NOT EXISTS idx_institutions_run_tick
    ON institutions (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_institutions_run_nation
    ON institutions (run_id, nation_id, tick);
CREATE INDEX IF NOT EXISTS idx_institutions_type
    ON institutions (run_id, inst_type, tick);
```

### 2.12 `wars` — Conflict Records

Wars span multiple ticks. One row per war (not per tick). The war record is created when a war declaration event fires and updated when it ends. Battles within a war are tracked via the events table (`war.battle_resolved`).

```sql
CREATE TABLE IF NOT EXISTS wars (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    war_id              TEXT        NOT NULL,   -- UUID v4
    attacker_nation_id  TEXT        NOT NULL,   -- UUID v4
    defender_nation_id  TEXT        NOT NULL,   -- UUID v4
    start_tick          INTEGER     NOT NULL,
    end_tick            INTEGER,               -- NULL while ongoing
    outcome             TEXT                   -- NULL while ongoing
                        CHECK (outcome IS NULL OR outcome IN (
                            'attacker_victory', 'defender_victory', 'stalemate',
                            'peace_treaty', 'white_peace', 'annexation'
                        )),
    casualties_attacker INTEGER     NOT NULL DEFAULT 0,
    casualties_defender INTEGER     NOT NULL DEFAULT 0,
    territory_exchanged TEXT        NOT NULL DEFAULT '[]',  -- JSON array of city_ids that changed hands
    war_score_attacker  INTEGER     NOT NULL DEFAULT 0,     -- 0..10000
    war_score_defender  INTEGER     NOT NULL DEFAULT 0,

    PRIMARY KEY (run_id, war_id)
);

CREATE INDEX IF NOT EXISTS idx_wars_run_nations
    ON wars (run_id, attacker_nation_id, defender_nation_id);
CREATE INDEX IF NOT EXISTS idx_wars_active
    ON wars (run_id, start_tick)
    WHERE end_tick IS NULL;
```

### 2.13 `research_runs` — Research and Scenario Metadata

Supplements `runs` with research-specific metadata. Created when a run is tagged as a research run. Not all runs have a research_runs row.

```sql
CREATE TABLE IF NOT EXISTS research_runs (
    run_id              TEXT        NOT NULL PRIMARY KEY REFERENCES runs(run_id) ON DELETE CASCADE,
    scenario_json       TEXT        NOT NULL,   -- full scenario JSON at time of run
    parameter_set_json  TEXT        NOT NULL,   -- full parameter sweep entry (if part of sweep)
    user_id             TEXT        NOT NULL,   -- researcher identifier (not authenticated in SQLite mode)
    tags                TEXT        NOT NULL DEFAULT '[]',  -- JSON array of string tags
    sweep_id            TEXT,                  -- UUID of parameter sweep batch this run belongs to
    notes               TEXT,
    is_canonical        INTEGER     NOT NULL DEFAULT 0 CHECK (is_canonical IN (0, 1)),
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_research_runs_user
    ON research_runs (user_id);
CREATE INDEX IF NOT EXISTS idx_research_runs_sweep
    ON research_runs (sweep_id)
    WHERE sweep_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_research_runs_canonical
    ON research_runs (is_canonical)
    WHERE is_canonical = 1;
```

### 2.14 `replay_events` — Compressed Event Stream for Inline Storage

This table holds a compressed replica of the event stream for runs where the full .civreplay file has been archived but fast seek access is still needed. It is not written during active runs; it is populated by the archival process.

```sql
CREATE TABLE IF NOT EXISTS replay_events (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    seq                 INTEGER     NOT NULL,   -- global sequence number within run
    tick                INTEGER     NOT NULL,
    event_bytes         BLOB        NOT NULL,   -- zstd-compressed MessagePack of SimEvent
    hash_prev           BLOB        NOT NULL,   -- BLAKE3 of previous frame (32 bytes)
    hash_self           BLOB        NOT NULL,   -- BLAKE3 of this frame (32 bytes)

    PRIMARY KEY (run_id, seq)
);

CREATE INDEX IF NOT EXISTS idx_replay_run_tick
    ON replay_events (run_id, tick);
```

### 2.15 `metrics_timeseries` — Aggregated Metrics

Pre-aggregated metrics written at the end of each tick. Redundant with data in nations/cities/citizens but cached here for fast time-series queries without expensive per-tick joins. The `entity_scope` identifies what the metric applies to.

```sql
CREATE TABLE IF NOT EXISTS metrics_timeseries (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    metric_name         TEXT        NOT NULL,
    metric_value        INTEGER     NOT NULL,   -- always integer; fixed-point where needed
    entity_scope        TEXT        NOT NULL,   -- 'global', UUID of nation/city/institution, or 'class:working'
    unit                TEXT        NOT NULL DEFAULT 'raw',  -- 'raw', 'millijoules', 'permille', 'count'

    PRIMARY KEY (run_id, tick, metric_name, entity_scope)
);

CREATE INDEX IF NOT EXISTS idx_metrics_run_name
    ON metrics_timeseries (run_id, metric_name, tick);
CREATE INDEX IF NOT EXISTS idx_metrics_run_scope
    ON metrics_timeseries (run_id, entity_scope, metric_name, tick);
```

**Standard metric names:**

| Metric Name | Unit | Entity Scope | Description |
|------------|------|-------------|-------------|
| `gdp` | `millijoules` | nation UUID | Total economic output |
| `happiness_avg` | `permille` | nation UUID / `class:X` | Population average happiness |
| `gini` | `permille` | nation UUID | Gini coefficient |
| `stability` | `permille` | nation UUID | Political stability |
| `legitimacy` | `permille` | nation UUID / institution UUID | Legitimacy score |
| `energy_balance` | `millijoules` | nation UUID / city UUID | Net energy balance |
| `co2_ppm` | `raw` | `global` | CO2 concentration |
| `temp_offset` | `millicelsius` | `global` | Temperature offset |
| `unemployment_rate` | `permille` | nation UUID / city UUID | Unemployment rate |
| `war_casualties` | `count` | `global` / nation UUID | Cumulative war casualties |
| `market_stress` | `permille` | `global` / city UUID / `good:X` | Market supply stress |
| `institution_capture` | `permille` | institution UUID | Institutional capture level |

### 2.16 `rng_seeds` — RNG Seed Log for Replay Auditability

Every use of the random number generator is logged with its seed, call index, and the simulation phase in which it occurred. This enables full deterministic replay verification: any external verifier can recompute the RNG sequence and confirm it matches the recorded seed log.

```sql
CREATE TABLE IF NOT EXISTS rng_seeds (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    phase_enum          TEXT        NOT NULL
                        CHECK (phase_enum IN (
                            'stochastic_events', 'migration', 'war_resolution',
                            'climate_perturbation', 'citizen_behavior', 'institution_drift'
                        )),
    seed_u64            INTEGER     NOT NULL,   -- u64 stored as i64 (bit-cast)
    call_index          INTEGER     NOT NULL,   -- monotonically increasing within (run_id, tick, phase_enum)
    call_site           TEXT        NOT NULL,   -- source location: "module::function:line"
    output_u64          INTEGER     NOT NULL,   -- the value returned by the RNG at this call

    PRIMARY KEY (run_id, tick, phase_enum, call_index)
);

CREATE INDEX IF NOT EXISTS idx_rng_run_tick
    ON rng_seeds (run_id, tick);
```

---

## 3. Indexes and Performance

### 3.1 SQLite Index DDL Summary

All indexes are created with `CREATE INDEX IF NOT EXISTS` in the migration scripts. The following table summarizes the rationale for each index group.

| Index | Table | Columns | Purpose |
|-------|-------|---------|---------|
| `idx_runs_scenario_id` | runs | scenario_id | Filter runs by scenario |
| `idx_snapshots_run_tick_full` | snapshots | (run_id, tick) WHERE is_full=1 | Fast lookup of full snapshots for rollback |
| `idx_events_run_tick` | events | (run_id, tick) | Per-tick event fetch for replay |
| `idx_events_run_type` | events | (run_id, event_type) | Filter events by type across all ticks |
| `idx_events_actor` | events | (run_id, actor_id) WHERE NOT NULL | All events for a specific actor |
| `idx_nations_run_tick` | nations | (run_id, tick) | Per-tick nation state fetch |
| `idx_nations_run_nation` | nations | (run_id, nation_id) | Full history of one nation |
| `idx_cities_run_nation` | cities | (run_id, nation_id, tick) | All cities of a nation at tick |
| `idx_citizens_run_city_tick` | citizens | (run_id, city_id, tick) | Citizens in a city at tick |
| `idx_citizens_class_tick` | citizens | (run_id, tick, class_enum) | Class distribution per tick |
| `idx_citizens_migration_intent` | citizens | (run_id, tick, city_id) WHERE intent=1 | Migration flow queries |
| `idx_ledger_run_from_actor` | ledger_transfers | (run_id, from_actor_id, tick) | Debits by actor |
| `idx_ledger_currency_type` | ledger_transfers | (run_id, tick, currency_enum, transfer_type) | Transfers by currency and type |
| `idx_markets_run_good` | markets | (run_id, good_enum, tick) | Price history for a good |
| `idx_markets_unmet_demand` | markets | (run_id, tick, good_enum) WHERE >0 | Supply stress events |
| `idx_institutions_run_nation` | institutions | (run_id, nation_id, tick) | Institutions in a nation |
| `idx_metrics_run_name` | metrics_timeseries | (run_id, metric_name, tick) | Time series for a metric |

### 3.2 SQLite PRAGMA Settings

These PRAGMAs are applied by the `DbConn::open` wrapper every time a connection is opened. They are not stored in the database; they must be re-applied on each connection.

```rust
// src/db/conn.rs
pub async fn open(path: &Path) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .pragma("cache_size", "-65536")    // 64 MiB
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "8589934592") // 8 GiB
        .pragma("wal_autocheckpoint", "1000")
        .pragma("optimize", "0x10002");    // analyze on close

    SqlitePool::connect_with(options).await.map_err(DbError::Connection)
}
```

**PRAGMA rationale:**

| PRAGMA | Value | Rationale |
|--------|-------|-----------|
| `journal_mode=WAL` | WAL | Concurrent readers during simulation write; no journal contention |
| `synchronous=NORMAL` | NORMAL | Durable enough (survives OS crash); faster than FULL |
| `foreign_keys=ON` | ON | Enforce referential integrity; off by default in SQLite |
| `cache_size=-65536` | 64 MiB | Large page cache reduces I/O on repeated queries |
| `temp_store=MEMORY` | MEMORY | Sorting and grouping operations use RAM, not temp files |
| `mmap_size=8GiB` | 8589934592 | Memory-mapped I/O for read-heavy analytical queries |
| `wal_autocheckpoint=1000` | 1000 | Checkpoint WAL after 1000 pages to prevent unbounded growth |

### 3.3 PostgreSQL Partitioning Strategy

When PostgreSQL is the backend, the following tables use `RANGE` partitioning on `tick`:

```sql
-- snapshots: 1000-tick partitions
CREATE TABLE snapshots (
    -- ... same columns ...
) PARTITION BY RANGE (tick);

-- Managed by pg_partman; template:
CREATE TABLE snapshots_p0000 PARTITION OF snapshots
    FOR VALUES FROM (0) TO (1000);
CREATE TABLE snapshots_p1000 PARTITION OF snapshots
    FOR VALUES FROM (1000) TO (2000);
-- ... created dynamically by pg_partman as simulation progresses

-- BRIN index within each partition (tick is nearly monotonic within partition)
CREATE INDEX snapshots_p0000_tick_brin ON snapshots_p0000 USING BRIN (tick);

-- Same pattern for: events, citizens, metrics_timeseries
-- nations and cities are smaller; no partitioning needed
-- ledger_transfers: partitioned by tick if run has >10M transfers
```

**Partition maintenance:**

```sql
-- pg_partman configuration (managed table)
SELECT partman.create_parent(
    p_parent_table => 'public.snapshots',
    p_control => 'tick',
    p_type => 'range',
    p_interval => '1000',
    p_premake => 4
);
```

### 3.4 Query Plan Annotations

Key queries and their expected query plans:

**Q1: Fetch all nation states at tick T for run R**
```sql
EXPLAIN QUERY PLAN
SELECT * FROM nations WHERE run_id = ? AND tick = ?;
-- Expected: SEARCH nations USING INDEX idx_nations_run_tick (run_id=? AND tick=?)
-- Cardinality: O(num_nations), typically 4-20 rows
-- Expected wall time: <1ms
```

**Q2: Time series of GDP for a specific nation**
```sql
EXPLAIN QUERY PLAN
SELECT tick, gdp_millijoules FROM nations
WHERE run_id = ? AND nation_id = ?
ORDER BY tick ASC;
-- Expected: SEARCH nations USING INDEX idx_nations_run_nation (run_id=? AND nation_id=?)
-- Full index scan for one nation; cardinality = num_ticks
-- Expected wall time: <10ms for 10k ticks
```

**Q3: All events in tick range [T1, T2] for run R**
```sql
EXPLAIN QUERY PLAN
SELECT * FROM events WHERE run_id = ? AND tick BETWEEN ? AND ?
ORDER BY seq ASC;
-- Expected: SEARCH events USING INDEX idx_events_run_tick (run_id=? AND tick>? AND tick<?)
-- Expected wall time: <5ms for 100-tick window
```

**Q4: Market price history for energy in city C**
```sql
EXPLAIN QUERY PLAN
SELECT tick, clearing_price_mj FROM markets
WHERE run_id = ? AND good_enum = 'energy' AND city_id = ?
ORDER BY tick ASC;
-- Expected: SEARCH markets USING INDEX PRIMARY KEY (run_id=? AND tick=? AND good_enum=? AND city_id=?)
-- or: SEARCH markets USING INDEX idx_markets_run_good for range scans
```

---

## 4. Rust Type Definitions and DDL Mapping

### 4.1 Core Simulation Structs

The following Rust structs are the canonical definitions. All SQL DDL is derived from these structs; the structs are the source of truth.

```rust
// src/sim/types.rs — canonical type definitions

use std::collections::BTreeMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Fixed-point millijoule value. Never f64 in simulation-critical paths.
pub type Mj = i64;

/// Fixed-point permille value (0..10000 = 0.00%..100.00%).
pub type Permille = i64;

/// Simulation tick counter.
pub type Tick = u64;

/// Nation identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NationId(pub Uuid);

/// City identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CityId(pub Uuid);

/// Citizen identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CitizenId(pub Uuid);

/// Institution identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InstId(pub Uuid);

/// War identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WarId(pub Uuid);

/// The complete in-memory world state for one tick.
/// Serialized to msgpack for snapshots. All maps use BTreeMap for determinism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnap {
    pub tick: Tick,
    pub run_id: Uuid,
    pub nations: BTreeMap<NationId, NationState>,
    pub cities: BTreeMap<CityId, CityState>,
    pub citizens: BTreeMap<CitizenId, CitizenRecord>,
    pub ledger: LedgerState,
    pub markets: BTreeMap<(Good, CityId), MarketClearing>,
    pub climate: ClimateState,
    pub institutions: BTreeMap<InstId, InstitutionState>,
    pub wars: BTreeMap<WarId, WarRecord>,
}

/// Nation state at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationState {
    pub nation_id: NationId,
    pub name: String,
    /// Ideology vector: 8 dimensions, each in [-1.0, 1.0].
    /// Dimensions: [planned_vs_market, authoritarian_vs_liberal,
    ///   isolationist_vs_globalist, secular_vs_theocratic,
    ///   militarist_vs_pacifist, ecoconservative_vs_extractivist,
    ///   centralist_vs_federalist, technocratic_vs_traditionalist]
    pub ideology_vector: [f64; 8],
    pub stability: Permille,
    pub legitimacy: Permille,
    pub population_total: i64,
    pub population_growth: i64,
    pub gdp_millijoules: Mj,
    pub energy_surplus_mj: Mj,
    pub food_surplus_mu: i64,
    pub gini_coefficient: Permille,
    pub at_war: bool,
    pub capital_city_id: Option<CityId>,
}

/// City state at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityState {
    pub city_id: CityId,
    pub nation_id: NationId,
    pub name: String,
    pub position: (i32, i32),
    pub population: i64,
    pub energy_balance_mj: Mj,
    pub food_balance_mu: i64,
    pub housing_capacity: i64,
    pub employed_count: i64,
    pub unemployed_count: i64,
    pub happiness_avg: Permille,
    pub infrastructure_level: i64,
    pub is_capital: bool,
    pub under_siege: bool,
}

/// Individual citizen record at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizenRecord {
    pub citizen_id: CitizenId,
    pub city_id: CityId,
    pub nation_id: NationId,
    pub happiness: Permille,
    pub wealth_mj: Mj,
    pub class: CitizenClass,
    pub employment_status: EmploymentStatus,
    pub age_ticks: u64,
    pub joule_quota_mj: Mj,
    pub dissatisfaction: Permille,
    pub migration_intent: bool,
}

/// Double-entry ledger state.
/// Invariant: for each currency, sum of all balances across all actors is conserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerState {
    pub balances: BTreeMap<(Uuid, Currency), Mj>,
    pub transfers: Vec<LedgerTransfer>,
}

/// Single ledger transfer (double-entry: one debit, one credit per transfer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerTransfer {
    pub transfer_id: Uuid,
    pub from_actor_id: Uuid,
    pub from_actor_type: ActorType,
    pub to_actor_id: Uuid,
    pub to_actor_type: ActorType,
    pub amount_mj: Mj,
    pub currency: Currency,
    pub transfer_type: String,
    pub event_id: Option<Uuid>,
}

/// Market clearing result for one good in one city at one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketClearing {
    pub good: Good,
    pub city_id: CityId,
    pub clearing_price_mj: Mj,
    pub bid_volume: i64,
    pub ask_volume: i64,
    pub cleared_volume: i64,
    pub unmet_demand: i64,
    pub unmet_supply: i64,
    pub price_floor_active: bool,
    pub price_ceiling_active: bool,
    pub regime: AllocationRegime,
}

/// Climate state at a single tick. Global (not city-scoped).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateState {
    pub global_temp_offset_mc: i64,
    pub co2_ppm_mc: i64,
    pub sea_level_rise_mm: i64,
    pub ocean_acidification_mu: i64,
    pub arctic_ice_pct: Permille,
    pub active_tipping_points: Vec<TippingPoint>,
    pub extreme_weather_count: i64,
    pub renewable_capacity_pct: Permille,
}

/// Institution state at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionState {
    pub inst_id: InstId,
    pub inst_type: InstitutionType,
    pub nation_id: NationId,
    pub name: String,
    pub capture_level: Permille,
    pub legitimacy: Permille,
    pub budget_mj: Mj,
    pub budget_spent_mj: Mj,
    pub policy_vector: serde_json::Value,
    pub autonomy_level: Permille,
    pub effectiveness: Permille,
}

/// War record. Spans multiple ticks; not tick-scoped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarRecord {
    pub war_id: WarId,
    pub attacker_nation_id: NationId,
    pub defender_nation_id: NationId,
    pub start_tick: Tick,
    pub end_tick: Option<Tick>,
    pub outcome: Option<WarOutcome>,
    pub casualties_attacker: i64,
    pub casualties_defender: i64,
    pub territory_exchanged: Vec<CityId>,
    pub war_score_attacker: Permille,
    pub war_score_defender: Permille,
}
```

### 4.2 SQLx Type Mappings

The project uses `sqlx` 0.8 with the `sqlite` feature flag. All database operations use typed queries with compile-time SQL verification via `sqlx::query!` and `sqlx::query_as!`.

```rust
// Cargo.toml
// sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "uuid", "json", "macros"] }
// uuid = { version = "1", features = ["v4"] }
// rmp-serde = "1"      -- MessagePack serialization
// blake3 = "1"
// zstd = "0.13"

// src/db/queries.rs — example typed queries

pub async fn insert_nation_state(
    pool: &SqlitePool,
    run_id: &str,
    tick: i64,
    state: &NationState,
) -> Result<(), sqlx::Error> {
    let ideology_json = serde_json::to_string(&state.ideology_vector)
        .expect("ideology_vector serialization infallible for [f64; 8]");
    sqlx::query!(
        r#"
        INSERT INTO nations (
            run_id, tick, nation_id, name, ideology_vector,
            stability, legitimacy, population_total, population_growth,
            gdp_millijoules, energy_surplus_mj, food_surplus_mu,
            gini_coefficient, at_war, capital_city_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        run_id,
        tick,
        state.nation_id.0.to_string(),
        state.name,
        ideology_json,
        state.stability,
        state.legitimacy,
        state.population_total,
        state.population_growth,
        state.gdp_millijoules,
        state.energy_surplus_mj,
        state.food_surplus_mu,
        state.gini_coefficient,
        state.at_war as i64,
        state.capital_city_id.map(|id| id.0.to_string()),
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Bulk insert citizen records using a transaction for performance.
/// 1M rows target: < 5 seconds (see Section 10.3).
pub async fn bulk_insert_citizens(
    pool: &SqlitePool,
    run_id: &str,
    tick: i64,
    citizens: &[CitizenRecord],
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;
    for citizen in citizens {
        sqlx::query!(
            r#"
            INSERT INTO citizens (
                run_id, tick, citizen_id, city_id, nation_id,
                happiness, wealth_mj, class_enum, employment_status,
                age_ticks, joule_quota_mj, dissatisfaction, migration_intent
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            run_id,
            tick,
            citizen.citizen_id.0.to_string(),
            citizen.city_id.0.to_string(),
            citizen.nation_id.0.to_string(),
            citizen.happiness,
            citizen.wealth_mj,
            citizen.class.as_str(),
            citizen.employment_status.as_str(),
            citizen.age_ticks as i64,
            citizen.joule_quota_mj,
            citizen.dissatisfaction,
            citizen.migration_intent as i64,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(citizens.len())
}
```

### 4.3 Custom Type Encodings

**BLAKE3 hash (32 bytes) ↔ BLOB(32):**

```rust
pub fn blake3_to_blob(hash: &blake3::Hash) -> Vec<u8> {
    hash.as_bytes().to_vec()
}

pub fn blake3_from_blob(blob: &[u8]) -> Result<blake3::Hash, DbError> {
    let bytes: [u8; 32] = blob.try_into()
        .map_err(|_| DbError::InvalidHash(format!("expected 32 bytes, got {}", blob.len())))?;
    Ok(blake3::Hash::from_bytes(bytes))
}
```

**ChaCha20Rng seed (u64) ↔ INTEGER (i64 bit-cast):**

```rust
// u64 → i64 via bit-cast. Round-trip safe. SQLite INTEGER is signed 64-bit.
pub fn seed_to_db(seed: u64) -> i64 { i64::from_ne_bytes(seed.to_ne_bytes()) }
pub fn seed_from_db(stored: i64) -> u64 { u64::from_ne_bytes(stored.to_ne_bytes()) }
```

**Ideology vector ([f64; 8]) ↔ JSON TEXT:**

```rust
pub fn ideology_to_json(v: &[f64; 8]) -> String {
    serde_json::to_string(v).expect("f64 array serialization is infallible")
}
pub fn ideology_from_json(s: &str) -> Result<[f64; 8], DbError> {
    let vec: Vec<f64> = serde_json::from_str(s)
        .map_err(|e| DbError::ParseError(format!("ideology JSON: {e}")))?;
    vec.try_into()
        .map_err(|_| DbError::ParseError("ideology_vector must have exactly 8 elements".into()))
}
```

**Permille (i64, 0..10000) ↔ INTEGER:** Stored directly. Display layer divides by 100 for percent display.

**Millijoule (i64) ↔ INTEGER:** Stored directly. Display layer divides by 1000 for joule display.

**UUID ↔ TEXT:** `uuid::Uuid::to_string()` → hyphenated lowercase. Parse with `uuid::Uuid::parse_str()`.

**Boolean ↔ INTEGER:** `true` = 1, `false` = 0. SQLite has no native BOOLEAN type.

**Vec<TippingPoint> ↔ JSON TEXT:** Serialized as JSON array of snake_case strings via serde.

**serde_json::Value ↔ TEXT:** Stored as compact JSON string. Parsed on read.

### 4.4 Diesel Schema Macros (Reference)

Provided for projects using Diesel as an alternative to SQLx. Not the active backend.

```rust
// src/db/diesel_schema.rs (reference only)

diesel::table! {
    runs (run_id) {
        run_id -> Text,
        scenario_id -> Text,
        seed -> BigInt,
        start_tick -> BigInt,
        end_tick -> Nullable<BigInt>,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        params -> Text,
        tick_duration_ms -> BigInt,
        version -> Text,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    nations (run_id, tick, nation_id) {
        run_id -> Text,
        tick -> BigInt,
        nation_id -> Text,
        name -> Text,
        ideology_vector -> Text,
        stability -> BigInt,
        legitimacy -> BigInt,
        population_total -> BigInt,
        population_growth -> BigInt,
        gdp_millijoules -> BigInt,
        energy_surplus_mj -> BigInt,
        food_surplus_mu -> BigInt,
        gini_coefficient -> BigInt,
        at_war -> Integer,
        capital_city_id -> Nullable<Text>,
    }
}

diesel::table! {
    citizens (run_id, tick, citizen_id) {
        run_id -> Text,
        tick -> BigInt,
        citizen_id -> Text,
        city_id -> Text,
        nation_id -> Text,
        happiness -> BigInt,
        wealth_mj -> BigInt,
        class_enum -> Text,
        employment_status -> Text,
        age_ticks -> BigInt,
        joule_quota_mj -> BigInt,
        dissatisfaction -> BigInt,
        migration_intent -> Integer,
    }
}
```

---

## 5. Data Lifecycle and Retention

### 5.1 Hot Window Policy

The "hot window" is the set of ticks with full per-entity rows in SQLite and available for fast query without replay.

| Layer | Retention Window | Eviction Policy |
|-------|-----------------|----------------|
| In-memory rollback buffer | Last 64 ticks | Ring buffer; oldest evicted on push |
| SQLite citizens rows | Last 500 ticks | Pruning job after each 100-tick batch |
| SQLite nations/cities/institutions/markets | Last 1000 ticks | Pruning job |
| SQLite full snapshots (every 100 ticks) | Indefinite | Never pruned; zstd compressed |
| SQLite delta snapshots (every tick) | Last 200 ticks | Pruned when full snapshot covers range |
| .civreplay archive | All ticks, all events | Never pruned; zstd per-frame |

**The hot window is transparent to queries.** All queries target SQLite. If a tick is outside the hot window, the query layer returns an error directing the caller to use the replay/export path instead. No silent fallback to replay occurs.

### 5.2 Snapshot Policy

**Full snapshots (every 100 ticks):**

```rust
// Written when tick % 100 == 0
pub const FULL_SNAPSHOT_INTERVAL: u64 = 100;
pub const FULL_SNAPSHOT_COMPRESSION_LEVEL: i32 = 6;
```

**Delta snapshots (every tick, configurable):**

```rust
// Written every tick by default; configurable via scenario.persist_every_n_ticks
pub const DELTA_SNAPSHOT_COMPRESSION_LEVEL: i32 = 3;
```

Delta snapshot encoding: a `WorldDelta` struct containing only the entities whose state hash changed since the last snapshot. Components are keyed by entity ID; values are full new state (not diffs within the struct). This keeps deserialization simple (no patch-apply logic) at the cost of slightly larger delta blobs.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDelta {
    pub tick: Tick,
    pub run_id: Uuid,
    pub changed_nations: BTreeMap<NationId, NationState>,
    pub changed_cities: BTreeMap<CityId, CityState>,
    pub changed_citizens: BTreeMap<CitizenId, CitizenRecord>,
    pub removed_citizens: Vec<CitizenId>,   // died this tick
    pub ledger_delta: LedgerDelta,
    pub changed_markets: BTreeMap<(Good, CityId), MarketClearing>,
    pub climate: ClimateState,              // always included (small struct)
    pub changed_institutions: BTreeMap<InstId, InstitutionState>,
    pub war_updates: Vec<WarUpdate>,
}
```

**Snapshot policy table:**

| Tick | Full Snapshot | Delta Snapshot | Notes |
|------|-------------|---------------|-------|
| 0 | YES | NO | Initial state; always full |
| 1..99 | NO | YES | Delta only |
| 100 | YES | NO | Full snapshot replaces delta at 100-tick boundary |
| 101..199 | NO | YES | Delta only |
| 200 | YES | NO | Full snapshot |
| ... | ... | ... | Pattern repeats |

### 5.3 Pruning and Archival

Pruning is a background `tokio::task` scheduled after each batch of 100 ticks completes. It does not block tick execution.

**Citizen pruning:**

```rust
pub const CITIZEN_RETENTION_TICKS: i64 = 500;

pub async fn prune_old_citizen_rows(
    pool: &SqlitePool,
    run_id: &str,
    current_tick: i64,
    is_research_run: bool,
) -> Result<PruneStats, DbError> {
    if is_research_run {
        return Ok(PruneStats::skipped("research_run_retention_override"));
    }
    let cutoff = current_tick - CITIZEN_RETENTION_TICKS;
    if cutoff <= 0 { return Ok(PruneStats::skipped("no_rows_old_enough")); }

    // Archive to .civreplay before deleting
    archive_citizens_to_civreplay(pool, run_id, cutoff).await?;

    let deleted = sqlx::query!(
        "DELETE FROM citizens WHERE run_id = ? AND tick < ?",
        run_id, cutoff
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?
    .rows_affected();

    // VACUUM only for large prune operations (>100k rows)
    if deleted > 100_000 {
        sqlx::query("VACUUM").execute(pool).await.map_err(DbError::Sqlx)?;
    }

    Ok(PruneStats { rows_deleted: deleted, cutoff_tick: cutoff, skipped_reason: None })
}
```

**Nation/city/institution/market pruning (1000-tick retention):**

```rust
pub const WORLD_STATE_RETENTION_TICKS: i64 = 1000;

pub async fn prune_world_state_rows(
    pool: &SqlitePool,
    run_id: &str,
    current_tick: i64,
    is_research_run: bool,
) -> Result<PruneStats, DbError> {
    if is_research_run { return Ok(PruneStats::skipped("research_run")); }
    let cutoff = current_tick - WORLD_STATE_RETENTION_TICKS;
    if cutoff <= 0 { return Ok(PruneStats::skipped("no_rows_old_enough")); }

    // Prune in a single transaction
    let mut tx = pool.begin().await.map_err(DbError::Sqlx)?;
    let mut total_deleted = 0u64;

    for table in &["nations", "cities", "institutions"] {
        let rows = sqlx::query(&format!(
            "DELETE FROM {} WHERE run_id = ? AND tick < ?", table
        ))
        .bind(run_id)
        .bind(cutoff)
        .execute(&mut *tx)
        .await
        .map_err(DbError::Sqlx)?
        .rows_affected();
        total_deleted += rows;
    }

    // Markets: prune but keep unmet demand rows for research reference
    let market_rows = sqlx::query!(
        "DELETE FROM markets WHERE run_id = ? AND tick < ? AND unmet_demand = 0",
        run_id, cutoff
    )
    .execute(&mut *tx)
    .await
    .map_err(DbError::Sqlx)?
    .rows_affected();
    total_deleted += market_rows;

    tx.commit().await.map_err(DbError::Sqlx)?;
    Ok(PruneStats { rows_deleted: total_deleted, cutoff_tick: cutoff, skipped_reason: None })
}
```

### 5.4 Export Formats

**CSV export:**

One `.csv` file per table. Column names match SQL column names exactly. Produced by `civ export csv --run-id <UUID> --output-dir <path>`.

**Parquet export (via arrow2):**

One `.parquet` file per table. Schema mirrors SQL schema. Column types: Int64 for all INTEGER columns, Utf8 for TEXT, Binary for BLOB. Compression: ZSTD level 4. Row group size: 65536. Produced by `civ export parquet --run-id <UUID> --output-dir <path>`.

Parquet files are the recommended format for research analysis in Python (pandas/polars) or DuckDB:

```python
# Research analysis example (Python/DuckDB)
import duckdb

conn = duckdb.connect()
conn.execute("CREATE VIEW nations AS SELECT * FROM read_parquet('nations.parquet')")
conn.execute("CREATE VIEW citizens AS SELECT * FROM read_parquet('citizens.parquet')")

result = conn.execute("""
    SELECT n.tick, n.nation_id, n.name,
           AVG(c.happiness) AS avg_happiness,
           COUNT(*) AS citizen_count
    FROM citizens c
    JOIN nations n ON c.nation_id = n.nation_id AND c.tick = n.tick
    GROUP BY n.tick, n.nation_id, n.name
    ORDER BY n.tick, n.nation_id
""").fetchdf()
```

**.civreplay export:**

The `.civreplay` file is produced continuously during simulation (see Section 1.6). It can also be produced post-hoc from the SQLite event log via `civ export civreplay --run-id <UUID> --output <path>`.

---

## 6. Scenario and Parameter Schema

### 6.1 JSON Schema Definition

```json
{
  "$schema": "https://civlab.dev/schemas/scenario/v1.json",
  "scenario_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "name": "Two Superpowers: Joule vs. Market",
  "description": "Comparative study of joule-technocracy and market-capitalism under climate stress.",
  "version": "1.0.0",
  "initial_nations": [
    {
      "nation_id": "11111111-1111-1111-1111-111111111111",
      "name": "Joule Republic",
      "ideology_vector": [-0.8, 0.2, 0.0, -0.5, -0.3, 0.6, 0.4, 0.9],
      "initial_stability": 7500,
      "initial_legitimacy": 8000,
      "initial_population": 50000000,
      "capital_position": {"x": 10, "y": 15},
      "allocation_regime": "joule"
    },
    {
      "nation_id": "22222222-2222-2222-2222-222222222222",
      "name": "Free Market Federation",
      "ideology_vector": [0.9, 0.7, 0.5, -0.2, 0.1, -0.4, -0.3, -0.2],
      "initial_stability": 6500,
      "initial_legitimacy": 7000,
      "initial_population": 60000000,
      "capital_position": {"x": 30, "y": 15},
      "allocation_regime": "market"
    }
  ],
  "initial_cities": [
    {
      "city_id": "aaaa0001-0000-0000-0000-000000000001",
      "nation_id": "11111111-1111-1111-1111-111111111111",
      "name": "Joulesburg",
      "position": {"x": 10, "y": 15},
      "initial_population": 5000000,
      "is_capital": true,
      "initial_infrastructure_level": 75
    }
  ],
  "initial_institutions": [
    {
      "inst_id": "bbbb0001-0000-0000-0000-000000000001",
      "inst_type": "planning_bureau",
      "nation_id": "11111111-1111-1111-1111-111111111111",
      "name": "Joule Allocation Bureau",
      "initial_legitimacy": 8500,
      "initial_budget_mj": 100000000000
    }
  ],
  "climate_config": {
    "initial_temp_offset_mc": 1200,
    "initial_co2_ppm_mc": 420000,
    "initial_sea_level_rise_mm": 200,
    "warming_rate_mc_per_1000_ticks": 50,
    "tipping_points_enabled": true,
    "tipping_point_thresholds": {
      "west_antarctic_ice_sheet_collapse": 15000,
      "amazon_dieback": 20000
    }
  },
  "economy_config": {
    "global_energy_supply_mj_per_tick": 500000000000,
    "food_base_production_mu_per_capita": 3500,
    "market_clearing_algorithm": "walrasian_tatonnement",
    "max_price_adjustment_per_tick_permille": 100,
    "joule_quota_base_mj_per_citizen": 1000000
  },
  "world_config": {
    "grid_width": 50,
    "grid_height": 40,
    "max_cities_per_nation": 20
  },
  "rng_seed": 4242424242424242,
  "tick_limit": 10000,
  "persist_every_n_ticks": 10,
  "research_run": true,
  "win_conditions": [
    {
      "type": "metric_threshold",
      "metric": "happiness_avg",
      "entity_scope": "global",
      "threshold": 8000,
      "sustained_ticks": 500,
      "winner": "highest"
    },
    {
      "type": "tick_limit",
      "tick": 10000,
      "winner": "evaluate_metrics"
    }
  ],
  "tags": ["comparative", "climate_stress", "two_nations", "joule_vs_market"]
}
```

### 6.2 Validation Rules

All scenario files are validated at load time. Failures are hard errors.

```rust
// src/scenario/validator.rs

pub fn validate(scenario: &Scenario) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // R1: scenario_id must be a valid UUID v4
    if Uuid::parse_str(&scenario.scenario_id).is_err() {
        errors.push(ValidationError::InvalidUuid("scenario_id".into()));
    }

    // R2: ideology_vector must have exactly 8 elements, each in [-1.0, 1.0]
    for nation in &scenario.initial_nations {
        if nation.ideology_vector.len() != 8 {
            errors.push(ValidationError::IdeologyVectorLength(nation.nation_id.clone()));
        }
        for &v in &nation.ideology_vector {
            if !(-1.0f64..=1.0).contains(&v) {
                errors.push(ValidationError::IdeologyVectorRange(nation.nation_id.clone(), v));
            }
        }
    }

    // R3: all city nation_ids must reference a declared nation
    let nation_ids: std::collections::HashSet<_> =
        scenario.initial_nations.iter().map(|n| &n.nation_id).collect();
    for city in &scenario.initial_cities {
        if !nation_ids.contains(&city.nation_id) {
            errors.push(ValidationError::OrphanCity(city.city_id.clone()));
        }
    }

    // R4: all institution nation_ids must reference a declared nation
    for inst in &scenario.initial_institutions {
        if !nation_ids.contains(&inst.nation_id) {
            errors.push(ValidationError::OrphanInstitution(inst.inst_id.clone()));
        }
    }

    // R5: rng_seed must be non-zero
    if scenario.rng_seed == 0 {
        errors.push(ValidationError::ZeroSeed);
    }

    // R6: tick_limit must be >= 100
    if scenario.tick_limit < 100 {
        errors.push(ValidationError::TickLimitTooSmall(scenario.tick_limit));
    }

    // R7: at least one nation and one city required
    if scenario.initial_nations.is_empty() { errors.push(ValidationError::NoNations); }
    if scenario.initial_cities.is_empty() { errors.push(ValidationError::NoCities); }

    // R8: each capital_position must map to an initial_city in that nation
    for nation in &scenario.initial_nations {
        let has_capital = scenario.initial_cities.iter().any(|c|
            c.nation_id == nation.nation_id && c.is_capital
        );
        if !has_capital {
            errors.push(ValidationError::MissingCapitalCity(nation.nation_id.clone()));
        }
    }

    // R9: win_conditions must have at least one tick_limit or metric_threshold entry
    if scenario.win_conditions.is_empty() {
        errors.push(ValidationError::NoWinConditions);
    }

    // R10: persist_every_n_ticks must be >= 1
    if scenario.persist_every_n_ticks < 1 {
        errors.push(ValidationError::InvalidPersistInterval);
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### 6.3 Scenario Registry

The scenario registry file at `scenarios/index.json` is updated by the CLI when scenarios are added or archived.

```json
{
  "version": "1",
  "updated_at": "2026-02-21T00:00:00Z",
  "scenarios": [
    {
      "scenario_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "name": "Two Superpowers: Joule vs. Market",
      "file": "two-superpowers-joule-market.json",
      "tags": ["comparative", "climate_stress", "two_nations"],
      "created_at": "2026-02-21T00:00:00Z",
      "status": "active"
    },
    {
      "scenario_id": "00000000-0000-0000-0000-000000000001",
      "name": "Minimal Smoke Test",
      "file": "minimal.json",
      "tags": ["test", "ci"],
      "created_at": "2026-02-21T00:00:00Z",
      "status": "active"
    }
  ]
}
```

---

## 7. Conservation and Integrity Invariants

### 7.1 Ledger Conservation

The primary invariant: for every currency, the net sum of all Void-originated creation events minus all Void-destined destruction events per tick is zero. Peer-to-peer transfers are zero-sum by construction.

**In-process verification (runs before DB write):**

```rust
// src/sim/conservation.rs

pub fn verify_tick_conservation(transfers: &[LedgerTransfer]) -> Result<(), ConservationError> {
    let mut net: BTreeMap<Currency, i64> = BTreeMap::new();
    for t in transfers {
        match (t.from_actor_type, t.to_actor_type) {
            (ActorType::Void, _) => { *net.entry(t.currency).or_default() += t.amount_mj; }
            (_, ActorType::Void) => { *net.entry(t.currency).or_default() -= t.amount_mj; }
            _ => { /* peer transfer: zero net effect on system totals */ }
        }
    }
    for (currency, net_flow) in &net {
        if *net_flow != 0 {
            return Err(ConservationError::NonZeroNetFlow {
                currency: *currency,
                net_flow: *net_flow,
            });
        }
    }
    Ok(())
}
```

**SQL trigger (defense-in-depth, SQLite):**

```sql
CREATE TRIGGER IF NOT EXISTS trg_ledger_conservation_check
AFTER INSERT ON ledger_transfers
BEGIN
    SELECT RAISE(ABORT, 'CONSERVATION_VIOLATION: non-zero net Void flow for this currency/tick')
    WHERE EXISTS (
        SELECT 1
        FROM (
            SELECT
                SUM(CASE
                    WHEN from_actor_type = 'void' THEN  amount_mj
                    WHEN to_actor_type   = 'void' THEN -amount_mj
                    ELSE 0
                END) AS net_flow
            FROM ledger_transfers
            WHERE run_id = NEW.run_id
              AND tick   = NEW.tick
              AND currency_enum = NEW.currency_enum
        ) sub
        WHERE sub.net_flow <> 0
    );
END;
```

### 7.2 BLAKE3 State Hash Chain

Every full snapshot carries a `state_hash` = BLAKE3(canonical_msgpack(WorldSnap)). The canonical bytes are produced by serializing the `WorldSnap` with all BTreeMap keys sorted (guaranteed by BTreeMap ordering), all fields in declaration order, via `rmp_serde::to_vec_named`.

The integrity checker (see `src/sim/integrity.rs`) verifies that for each pair of consecutive full snapshots (tick A, tick B):

1. Load full snapshot at tick A.
2. Fetch all events in (A, B] from the `events` table.
3. Apply events tick-by-tick using the pure state transition function.
4. Compute BLAKE3 of the resulting WorldSnap at tick B.
5. Compare to stored `state_hash` at tick B.
6. If they differ, report a `HashViolation` for tick B.

This check is O(n_events * tick_span). For a 10k-tick run with 100 events/tick, verifying the full chain takes ~100k event applications. Expected wall time: < 60s for a typical research run on modern hardware.

```rust
// Integrity check entry point
pub async fn verify_full_chain(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<HashChainReport, IntegrityError> {
    let full_snapshots = fetch_full_snapshots_ordered(pool, run_id).await?;
    let mut violations = Vec::new();

    for window in full_snapshots.windows(2) {
        let (tick_a, _hash_a) = &window[0];
        let (tick_b, hash_b_stored) = &window[1];

        let world_a = load_and_decompress_snapshot(pool, run_id, *tick_a).await?;
        let events_ab = fetch_events_in_range(pool, run_id, *tick_a, *tick_b).await?;
        let world_b = apply_events_deterministic(world_a, &events_ab)?;

        let bytes_b = rmp_serde::to_vec_named(&world_b)
            .map_err(|e| IntegrityError::Serialize(e.to_string()))?;
        let computed = blake3::hash(&bytes_b);

        if computed.as_bytes() != hash_b_stored.as_slice() {
            violations.push(HashViolation { tick: *tick_b });
        }
    }

    Ok(HashChainReport {
        run_id: run_id.to_string(),
        snapshots_checked: full_snapshots.len(),
        violations,
    })
}
```

### 7.3 Unique and Not-Null Constraints Summary

| Table | Primary Key | Additional UNIQUE | Critical NOT NULL |
|-------|------------|-------------------|------------------|
| `runs` | `(run_id)` | — | run_id, scenario_id, seed, status |
| `snapshots` | `(run_id, tick)` | — | state_hash, snapshot_bytes |
| `events` | `(run_id, tick, event_id)` | `(run_id, seq)` | event_type, payload, seq, phase |
| `nations` | `(run_id, tick, nation_id)` | — | name, ideology_vector, stability, legitimacy, population_total |
| `cities` | `(run_id, tick, city_id)` | — | nation_id, name, position_x, position_y, population |
| `citizens` | `(run_id, tick, citizen_id)` | — | city_id, nation_id, happiness, wealth_mj, class_enum, employment_status, age_ticks |
| `ledger_transfers` | `(run_id, tick, transfer_id)` | — | from_actor_id, to_actor_id, amount_mj, currency_enum, transfer_type |
| `markets` | `(run_id, tick, good_enum, city_id)` | — | clearing_price_mj, bid_volume, ask_volume, cleared_volume |
| `climate_state` | `(run_id, tick)` | — | global_temp_offset_mc, co2_ppm_mc, sea_level_rise_mm |
| `institutions` | `(run_id, tick, inst_id)` | — | inst_type, nation_id, name |
| `wars` | `(run_id, war_id)` | — | attacker_nation_id, defender_nation_id, start_tick |
| `rng_seeds` | `(run_id, tick, phase_enum, call_index)` | — | seed_u64, call_site, output_u64 |

### 7.4 Foreign Key Cascade Behavior

All child tables reference `runs(run_id) ON DELETE CASCADE`. No other cascade relationships. Cross-entity references (nation_id in cities, city_id in citizens) are soft references validated in-process, not enforced by the database.

```sql
-- Safe run deletion (removes all child data atomically):
PRAGMA foreign_keys = ON;
DELETE FROM runs WHERE run_id = ? AND status IN ('archived', 'failed');
-- Cascades automatically to all 15 child tables.
```

---

## 8. Migration Strategy

### 8.1 `schema_versions` Table

Migration state is tracked in `schema_versions`. The migration runner reads this on startup and runs any unapplied migrations in version order.

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
    version         INTEGER     NOT NULL PRIMARY KEY,
    description     TEXT        NOT NULL,
    applied_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    checksum        TEXT        NOT NULL  -- hex(SHA-256) of migration SQL file content
);
```

### 8.2 Migration Runner

```rust
// src/db/migrations.rs

const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, description: "initial schema: runs, snapshots, events",
        sql: include_str!("../../migrations/001_initial.sql") },
    Migration { version: 2, description: "nations, cities tables",
        sql: include_str!("../../migrations/002_nation_city.sql") },
    Migration { version: 3, description: "citizens, ledger_transfers, markets",
        sql: include_str!("../../migrations/003_economy.sql") },
    Migration { version: 4, description: "climate_state, institutions, wars",
        sql: include_str!("../../migrations/004_world.sql") },
    Migration { version: 5, description: "research_runs, replay_events, metrics_timeseries, rng_seeds",
        sql: include_str!("../../migrations/005_research.sql") },
    Migration { version: 6, description: "schema_versions self-bootstrap + all indexes",
        sql: include_str!("../../migrations/006_indexes.sql") },
];

pub async fn run_pending(pool: &SqlitePool) -> Result<Vec<u32>, MigrationError> {
    // Bootstrap schema_versions if it doesn't exist yet
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_versions (
            version INTEGER NOT NULL PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            checksum TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await
    .map_err(MigrationError::Db)?;

    let applied: Vec<i64> = sqlx::query_scalar!(
        "SELECT version FROM schema_versions ORDER BY version ASC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let applied_set: std::collections::HashSet<i64> = applied.into_iter().collect();
    let mut ran = Vec::new();

    for migration in MIGRATIONS {
        if applied_set.contains(&(migration.version as i64)) { continue; }

        let checksum = format!("{:x}", sha256_of(migration.sql.as_bytes()));
        let mut tx = pool.begin().await.map_err(MigrationError::Db)?;

        // Execute each statement in the migration file separately
        for statement in migration.sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            sqlx::query(statement).execute(&mut *tx).await
                .map_err(|e| MigrationError::Execute(migration.version, e))?;
        }

        sqlx::query!(
            "INSERT INTO schema_versions (version, description, checksum) VALUES (?, ?, ?)",
            migration.version as i64, migration.description, checksum
        )
        .execute(&mut *tx)
        .await
        .map_err(MigrationError::Db)?;

        tx.commit().await.map_err(MigrationError::Db)?;
        ran.push(migration.version);
    }

    Ok(ran)
}
```

### 8.3 Migration File Conventions

Files are at `migrations/NNN_description.sql`. Embedded in binary via `include_str!`. Naming: zero-padded 3-digit version, snake_case description.

```
migrations/
  001_initial.sql               -- runs, snapshots, events
  002_nation_city.sql           -- nations, cities
  003_economy.sql               -- citizens, ledger_transfers, markets
  004_world.sql                 -- climate_state, institutions, wars
  005_research.sql              -- research_runs, replay_events, metrics_timeseries, rng_seeds
  006_indexes.sql               -- all indexes (separated for clarity)
  007_add_notes_column.sql      -- example additive migration
```

**Permitted migration operations:**

| Operation | Permitted |
|-----------|-----------|
| `ADD COLUMN ... DEFAULT ...` | YES |
| `CREATE TABLE IF NOT EXISTS` | YES |
| `CREATE INDEX IF NOT EXISTS` | YES |
| `INSERT OR IGNORE` | YES |
| `DROP COLUMN` | NO — mark deprecated in comment |
| `DROP TABLE` | NO — retain for at least one major version |
| `ALTER COLUMN` | NO — create new column, migrate data, deprecate old |
| Tightening CHECK constraint | NO — breaks existing data |
| Removing CHECK constraint | YES with a new table copy migration |

---

## 9. Research Query Patterns

All queries target SQLite. PostgreSQL variants use the same SQL plus window function extensions.

### 9.1 Average Happiness by Class Over Time

```sql
SELECT
    c.tick,
    c.class_enum,
    COUNT(*)                    AS citizen_count,
    AVG(c.happiness)            AS avg_happiness,
    AVG(c.wealth_mj)            AS avg_wealth_mj,
    AVG(c.dissatisfaction)      AS avg_dissatisfaction
FROM citizens c
WHERE c.run_id = :run_id
  AND c.nation_id = :nation_id
  AND c.tick BETWEEN :tick_start AND :tick_end
GROUP BY c.tick, c.class_enum
ORDER BY c.tick ASC, c.class_enum ASC;
```

Index used: `idx_citizens_run_nation_tick` + `idx_citizens_class_tick`. Expected wall time: <1s for 100-tick window.

### 9.2 GDP Trajectory per Nation

```sql
SELECT
    n.tick,
    n.nation_id,
    n.name                                              AS nation_name,
    n.gdp_millijoules                                   AS gdp_mj,
    n.gdp_millijoules / NULLIF(n.population_total, 0)  AS gdp_per_capita_mj,
    n.gini_coefficient,
    n.energy_surplus_mj,
    n.stability,
    n.legitimacy
FROM nations n
WHERE n.run_id = :run_id
  AND n.tick BETWEEN :tick_start AND :tick_end
ORDER BY n.tick ASC, n.nation_id ASC;
```

### 9.3 Market Price Volatility

```sql
SELECT
    m.good_enum,
    m.city_id,
    COUNT(*)                                                AS observations,
    MIN(m.clearing_price_mj)                               AS price_min,
    MAX(m.clearing_price_mj)                               AS price_max,
    AVG(m.clearing_price_mj)                               AS price_avg,
    AVG(m.clearing_price_mj * m.clearing_price_mj)
        - AVG(m.clearing_price_mj) * AVG(m.clearing_price_mj)  AS price_variance,
    AVG(m.unmet_demand)                                    AS avg_unmet_demand,
    SUM(CASE WHEN m.price_floor_active = 1 THEN 1 ELSE 0 END)   AS ticks_floor_active,
    SUM(CASE WHEN m.price_ceiling_active = 1 THEN 1 ELSE 0 END) AS ticks_ceiling_active
FROM markets m
WHERE m.run_id = :run_id
  AND m.tick BETWEEN :tick_start AND :tick_end
GROUP BY m.good_enum, m.city_id
ORDER BY price_variance DESC;
```

### 9.4 War Frequency Distribution

```sql
SELECT
    w.outcome,
    COUNT(*)                                                AS war_count,
    AVG(COALESCE(w.end_tick, :current_tick) - w.start_tick) AS avg_duration_ticks,
    SUM(w.casualties_attacker + w.casualties_defender)     AS total_casualties
FROM wars w
WHERE w.run_id = :run_id
GROUP BY w.outcome
ORDER BY war_count DESC;
```

### 9.5 Energy Balance vs. Stability Correlation (Pearson r)

```sql
SELECT
    n.nation_id,
    n.name,
    COUNT(*) AS sample_size,
    (COUNT(*) * SUM(n.energy_surplus_mj * n.stability)
     - SUM(n.energy_surplus_mj) * SUM(n.stability))
    / NULLIF(SQRT(
        (COUNT(*) * SUM(n.energy_surplus_mj * n.energy_surplus_mj)
         - SUM(n.energy_surplus_mj) * SUM(n.energy_surplus_mj))
        * (COUNT(*) * SUM(n.stability * n.stability)
           - SUM(n.stability) * SUM(n.stability))
    ), 0) AS pearson_r
FROM nations n
WHERE n.run_id = :run_id
  AND n.tick BETWEEN :tick_start AND :tick_end
GROUP BY n.nation_id, n.name
ORDER BY ABS(pearson_r) DESC;
```

### 9.6 Citizen Migration Flows

```sql
SELECT
    c.tick,
    c.city_id,
    COUNT(*)                                    AS total_citizens,
    SUM(c.migration_intent)                     AS migration_intent_count,
    CAST(SUM(c.migration_intent) AS REAL)
        / NULLIF(COUNT(*), 0) * 100.0           AS migration_intent_pct
FROM citizens c
WHERE c.run_id = :run_id
  AND c.tick BETWEEN :tick_start AND :tick_end
GROUP BY c.tick, c.city_id
ORDER BY c.tick ASC, migration_intent_pct DESC;
```

### 9.7 Institution Legitimacy Decay

```sql
SELECT
    i.tick,
    i.inst_id,
    i.name,
    i.inst_type,
    i.legitimacy,
    i.capture_level,
    i.autonomy_level,
    i.legitimacy - LAG(i.legitimacy, 1) OVER (
        PARTITION BY i.inst_id ORDER BY i.tick
    ) AS legitimacy_delta_per_tick
FROM institutions i
WHERE i.run_id = :run_id
  AND i.nation_id = :nation_id
  AND i.tick BETWEEN :tick_start AND :tick_end
ORDER BY i.inst_id, i.tick ASC;

-- Note: requires SQLite 3.25+ for LAG() window function.
```

### 9.8 Climate Shock vs. Economic Disruption

```sql
WITH climate_shocks AS (
    SELECT e.tick, json_extract(e.payload, '$.tipping_point') AS tipping_point
    FROM events e
    WHERE e.run_id = :run_id AND e.event_type = 'climate.tipping_point_activated'
)
SELECT
    cs.tipping_point,
    cs.tick AS shock_tick,
    AVG(n_pre.gdp_millijoules)  AS avg_gdp_pre_50ticks,
    AVG(n_post.gdp_millijoules) AS avg_gdp_post_50ticks,
    AVG(n_post.gdp_millijoules) - AVG(n_pre.gdp_millijoules) AS gdp_delta
FROM climate_shocks cs
JOIN nations n_pre  ON n_pre.run_id  = :run_id AND n_pre.tick  BETWEEN cs.tick - 50 AND cs.tick - 1
JOIN nations n_post ON n_post.run_id = :run_id AND n_post.tick BETWEEN cs.tick + 1  AND cs.tick + 50
GROUP BY cs.tipping_point, cs.tick
ORDER BY ABS(gdp_delta) DESC;
```

### 9.9 Gini Coefficient Trajectory with Moving Average

```sql
SELECT
    n.tick,
    n.nation_id,
    n.name,
    n.gini_coefficient,
    AVG(n.gini_coefficient) OVER (
        PARTITION BY n.nation_id
        ORDER BY n.tick
        ROWS BETWEEN 99 PRECEDING AND CURRENT ROW
    ) AS gini_ma_100ticks,
    n.stability,
    n.gdp_millijoules
FROM nations n
WHERE n.run_id = :run_id
  AND n.tick BETWEEN :tick_start AND :tick_end
ORDER BY n.nation_id, n.tick ASC;
```

### 9.10 Tipping Point Activation Timeline

```sql
SELECT
    json_extract(e.payload, '$.tipping_point') AS tipping_point,
    MIN(e.tick)                                AS first_activation_tick,
    cl.global_temp_offset_mc / 1000.0         AS temp_c_at_activation,
    cl.co2_ppm_mc / 1000.0                    AS co2_ppm_at_activation
FROM events e
JOIN climate_state cl ON cl.run_id = e.run_id AND cl.tick = e.tick
WHERE e.run_id = :run_id
  AND e.event_type = 'climate.tipping_point_activated'
GROUP BY json_extract(e.payload, '$.tipping_point')
ORDER BY first_activation_tick ASC;
```

---

## 10. Test Harness

### 10.1 Property Tests

Property tests verify the conservation and integrity invariants across arbitrary simulation states. They use `proptest` for property-based testing.

```toml
# Cargo.toml (dev dependencies)
[dev-dependencies]
proptest = "1"
tokio-test = "0.4"
tempfile = "3"
```

**Conservation invariant property test:**

```rust
// tests/property/conservation.rs

use proptest::prelude::*;
use civ_sim::sim::conservation::verify_tick_conservation;
use civ_sim::sim::types::{LedgerTransfer, ActorType, Currency};
use uuid::Uuid;

proptest! {
    /// Property: Any set of balanced creation/destruction pairs satisfies conservation.
    #[test]
    fn prop_balanced_void_transfers_conserve(
        amounts in prop::collection::vec(1i64..1_000_000_000i64, 0..100),
        currencies in prop::collection::vec(0u8..5, 0..100),
    ) {
        let transfers: Vec<LedgerTransfer> = amounts.iter().zip(currencies.iter())
            .flat_map(|(&amount, &cur_idx)| {
                let currency = match cur_idx % 5 {
                    0 => Currency::Joule,
                    1 => Currency::Fiat,
                    2 => Currency::Quota,
                    3 => Currency::LaborCredit,
                    _ => Currency::CarbonCredit,
                };
                let actor = Uuid::new_v4();
                // Create a matched creation + destruction pair
                vec![
                    LedgerTransfer {
                        transfer_id: Uuid::new_v4(),
                        from_actor_id: Uuid::nil(),
                        from_actor_type: ActorType::Void,
                        to_actor_id: actor,
                        to_actor_type: ActorType::Nation,
                        amount_mj: amount,
                        currency,
                        transfer_type: "creation".into(),
                        event_id: None,
                    },
                    LedgerTransfer {
                        transfer_id: Uuid::new_v4(),
                        from_actor_id: actor,
                        from_actor_type: ActorType::Nation,
                        to_actor_id: Uuid::nil(),
                        to_actor_type: ActorType::Void,
                        amount_mj: amount,
                        currency,
                        transfer_type: "destruction".into(),
                        event_id: None,
                    },
                ]
            })
            .collect();

        prop_assert!(verify_tick_conservation(&transfers).is_ok(),
            "Balanced creation/destruction pairs must satisfy conservation");
    }

    /// Property: Unbalanced Void transfers always violate conservation.
    #[test]
    fn prop_unbalanced_void_transfers_violate(
        amount in 1i64..1_000_000_000i64,
    ) {
        let transfer = LedgerTransfer {
            transfer_id: Uuid::new_v4(),
            from_actor_id: Uuid::nil(),
            from_actor_type: ActorType::Void,
            to_actor_id: Uuid::new_v4(),
            to_actor_type: ActorType::Nation,
            amount_mj: amount,
            currency: Currency::Joule,
            transfer_type: "creation_unmatched".into(),
            event_id: None,
        };
        prop_assert!(verify_tick_conservation(&[transfer]).is_err(),
            "Unmatched creation must violate conservation");
    }

    /// Property: Peer-to-peer transfers never affect conservation.
    #[test]
    fn prop_peer_transfers_do_not_affect_conservation(
        amounts in prop::collection::vec(1i64..1_000_000_000i64, 0..200),
    ) {
        let transfers: Vec<LedgerTransfer> = amounts.iter().map(|&amount| {
            LedgerTransfer {
                transfer_id: Uuid::new_v4(),
                from_actor_id: Uuid::new_v4(),
                from_actor_type: ActorType::Nation,
                to_actor_id: Uuid::new_v4(),
                to_actor_type: ActorType::City,
                amount_mj: amount,
                currency: Currency::Joule,
                transfer_type: "transfer".into(),
                event_id: None,
            }
        }).collect();
        prop_assert!(verify_tick_conservation(&transfers).is_ok(),
            "Peer transfers must always satisfy conservation (zero net effect)");
    }
}
```

**Ideology vector property test:**

```rust
proptest! {
    /// Property: ideology_to_json → ideology_from_json round-trips for all valid vectors.
    #[test]
    fn prop_ideology_roundtrip(
        values in prop::array::uniform8(-1.0f64..=1.0f64)
    ) {
        use civ_sim::db::encoding::{ideology_to_json, ideology_from_json};
        let json = ideology_to_json(&values);
        let recovered = ideology_from_json(&json).expect("roundtrip should succeed");
        for (a, b) in values.iter().zip(recovered.iter()) {
            prop_assert!((a - b).abs() < 1e-12,
                "ideology_vector roundtrip must be lossless for valid values");
        }
    }

    /// Property: seed_to_db → seed_from_db round-trips for all u64 values.
    #[test]
    fn prop_seed_roundtrip(seed: u64) {
        use civ_sim::db::encoding::{seed_to_db, seed_from_db};
        prop_assert_eq!(seed_from_db(seed_to_db(seed)), seed,
            "u64 seed bit-cast to i64 and back must be lossless");
    }
}
```

### 10.2 Round-Trip Tests

Round-trip tests verify that serialization → storage → retrieval → deserialization produces an identical struct. These use a real in-memory SQLite database.

```rust
// tests/integration/roundtrip.rs

use civ_sim::db::{conn::open_in_memory, migrations::run_pending};
use civ_sim::db::queries::*;
use civ_sim::sim::types::*;
use uuid::Uuid;

async fn setup_db() -> sqlx::SqlitePool {
    let pool = open_in_memory().await.expect("in-memory SQLite must open");
    run_pending(&pool).await.expect("migrations must succeed");
    pool
}

#[tokio::test]
async fn test_run_roundtrip() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();

    // Insert
    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test-scenario', 12345, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    // Retrieve
    let row = sqlx::query!(
        "SELECT run_id, scenario_id, seed, status FROM runs WHERE run_id = ?",
        run_id
    )
    .fetch_one(&pool).await.unwrap();

    assert_eq!(row.run_id, run_id);
    assert_eq!(row.seed, 12345i64);
    assert_eq!(row.status, "running");
}

#[tokio::test]
async fn test_nation_state_roundtrip() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();
    let nation_id = Uuid::new_v4();

    // Setup run
    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test', 1, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    let original = NationState {
        nation_id: NationId(nation_id),
        name: "Test Nation".to_string(),
        ideology_vector: [0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8],
        stability: 7500,
        legitimacy: 6000,
        population_total: 10_000_000,
        population_growth: 1234,
        gdp_millijoules: 999_000_000,
        energy_surplus_mj: -50_000,
        food_surplus_mu: 100_000,
        gini_coefficient: 3500,
        at_war: false,
        capital_city_id: None,
    };

    insert_nation_state(&pool, &run_id, 42, &original).await.unwrap();

    let fetched = fetch_nations_at_tick(&pool, &run_id, 42).await.unwrap();
    assert_eq!(fetched.len(), 1);
    let n = &fetched[0];

    assert_eq!(n.nation_id, original.nation_id);
    assert_eq!(n.name, original.name);
    assert_eq!(n.stability, original.stability);
    assert_eq!(n.gdp_millijoules, original.gdp_millijoules);
    assert_eq!(n.at_war, original.at_war);

    for (a, b) in n.ideology_vector.iter().zip(original.ideology_vector.iter()) {
        assert!((a - b).abs() < 1e-12, "ideology_vector must round-trip losslessly");
    }
}

#[tokio::test]
async fn test_snapshot_roundtrip() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test', 1, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    let world = WorldSnap {
        tick: 100,
        run_id: Uuid::parse_str(&run_id).unwrap(),
        nations: Default::default(),
        cities: Default::default(),
        citizens: Default::default(),
        ledger: LedgerState { balances: Default::default(), transfers: vec![] },
        markets: Default::default(),
        climate: ClimateState {
            global_temp_offset_mc: 1500,
            co2_ppm_mc: 450_000,
            sea_level_rise_mm: 300,
            ocean_acidification_mu: 150_000,
            arctic_ice_pct: 6500,
            active_tipping_points: vec![],
            extreme_weather_count: 0,
            renewable_capacity_pct: 2500,
        },
        institutions: Default::default(),
        wars: Default::default(),
    };

    use civ_sim::sim::persistence::write_snapshot;
    write_snapshot(&pool, &world, true, &run_id).await.unwrap();

    let row = sqlx::query!(
        "SELECT tick, is_full, size_bytes, compressed_size FROM snapshots WHERE run_id = ? AND tick = 100",
        run_id
    )
    .fetch_one(&pool).await.unwrap();

    assert_eq!(row.tick, 100i64);
    assert_eq!(row.is_full, 1i64);
    assert!(row.size_bytes > 0);
    assert!(row.compressed_size > 0);
    assert!(row.compressed_size <= row.size_bytes);
}

#[tokio::test]
async fn test_conservation_trigger_fires() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test', 1, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    // Insert an unmatched Void-origin transfer (conservation violation)
    // Expect the trigger to raise ABORT
    let result = sqlx::query!(
        r#"
        INSERT INTO ledger_transfers
            (run_id, tick, transfer_id, from_actor_id, from_actor_type,
             to_actor_id, to_actor_type, amount_mj, currency_enum, transfer_type)
        VALUES (?, 1, ?, '00000000-0000-0000-0000-000000000000', 'void',
                ?, 'nation', 1000000, 'joule', 'unmatched_creation')
        "#,
        run_id,
        Uuid::new_v4().to_string(),
        Uuid::new_v4().to_string()
    )
    .execute(&pool).await;

    // The trigger should have rejected this insert
    assert!(result.is_err(), "Unmatched Void transfer should trigger conservation violation");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("CONSERVATION_VIOLATION") || err_msg.contains("constraint"),
        "Error should mention conservation: {}", err_msg);
}
```

### 10.3 Performance Benchmarks

Performance benchmarks use Criterion.rs for statistical rigor. They run against a real SQLite database file (not in-memory) to measure realistic I/O performance.

```toml
# Cargo.toml
[[bench]]
name = "db_performance"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
```

```rust
// benches/db_performance.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use civ_sim::db::{conn::open, migrations::run_pending};
use civ_sim::db::queries::bulk_insert_citizens;
use civ_sim::sim::types::*;
use uuid::Uuid;
use std::path::PathBuf;
use tempfile::tempdir;

fn make_citizen(run_id_str: &str, city_id: Uuid, nation_id: Uuid, tick: i64) -> CitizenRecord {
    CitizenRecord {
        citizen_id: CitizenId(Uuid::new_v4()),
        city_id: CityId(city_id),
        nation_id: NationId(nation_id),
        happiness: 6000,
        wealth_mj: 500_000,
        class: CitizenClass::Working,
        employment_status: EmploymentStatus::Employed,
        age_ticks: 500,
        joule_quota_mj: 1_000_000,
        dissatisfaction: 2000,
        migration_intent: false,
    }
}

/// Benchmark: Insert 1M citizen records.
/// Target: < 5 seconds total (200k rows/sec sustained write rate).
fn bench_bulk_citizen_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("bench.db");

    let pool = rt.block_on(async {
        let pool = open(&db_path).await.unwrap();
        run_pending(&pool).await.unwrap();

        // Setup a run row
        sqlx::query!(
            "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
             VALUES ('bench-run', 'bench', 1, 'running', '{}', '1.0.0')"
        )
        .execute(&pool).await.unwrap();

        pool
    });

    let nation_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut group = c.benchmark_group("citizen_insert");
    group.throughput(Throughput::Elements(1_000_000));
    group.sample_size(3);  // large benchmark; 3 samples sufficient

    group.bench_function("1M_citizens", |b| {
        let mut tick = 0i64;
        b.iter(|| {
            tick += 1;
            let citizens: Vec<CitizenRecord> = (0..1_000_000)
                .map(|_| make_citizen("bench-run", city_id, nation_id, tick))
                .collect();
            rt.block_on(bulk_insert_citizens(&pool, "bench-run", tick, &citizens))
                .expect("bulk insert must succeed");
        });
    });

    group.finish();
}

/// Benchmark: Query last 100 ticks of nation state.
/// Target: < 100ms for run with 10k ticks, 10 nations.
fn bench_query_last_100_ticks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("bench_query.db");

    let (pool, max_tick) = rt.block_on(async {
        let pool = open(&db_path).await.unwrap();
        run_pending(&pool).await.unwrap();

        sqlx::query!(
            "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
             VALUES ('qbench', 'bench', 1, 'running', '{}', '1.0.0')"
        )
        .execute(&pool).await.unwrap();

        // Insert 10k ticks × 10 nations
        let nation_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
        let ideology_json = "[0.1,-0.2,0.3,-0.4,0.5,-0.6,0.7,-0.8]";
        for tick in 0i64..10_000 {
            for nation_id in &nation_ids {
                sqlx::query!(
                    r#"INSERT INTO nations
                       (run_id, tick, nation_id, name, ideology_vector,
                        stability, legitimacy, population_total, population_growth,
                        gdp_millijoules, energy_surplus_mj, food_surplus_mu, gini_coefficient, at_war)
                       VALUES ('qbench', ?, ?, 'Nation', ?, 7000, 6000, 5000000, 100, 1000000, 50000, 30000, 3500, 0)"#,
                    tick, nation_id.to_string(), ideology_json
                )
                .execute(&pool).await.unwrap();
            }
        }
        (pool, 10_000i64)
    });

    let mut group = c.benchmark_group("nation_query");

    group.bench_function("last_100_ticks_10_nations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _rows = sqlx::query!(
                    "SELECT tick, nation_id, gdp_millijoules, stability FROM nations
                     WHERE run_id = 'qbench' AND tick BETWEEN ? AND ?
                     ORDER BY tick ASC",
                    max_tick - 100,
                    max_tick
                )
                .fetch_all(&pool).await.unwrap();
            });
        });
    });

    group.finish();
}

/// Benchmark: Market clearing query across 1000 ticks × 8 goods × 20 cities.
/// Target: < 500ms.
fn bench_market_price_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("bench_market.db");

    let pool = rt.block_on(async {
        let pool = open(&db_path).await.unwrap();
        run_pending(&pool).await.unwrap();

        sqlx::query!(
            "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
             VALUES ('mbench', 'bench', 1, 'running', '{}', '1.0.0')"
        )
        .execute(&pool).await.unwrap();

        let goods = ["energy", "food", "housing", "medicine",
                     "capital_goods", "consumer_goods", "labor", "carbon_credit"];
        let city_ids: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();

        for tick in 0i64..1_000 {
            for good in &goods {
                for city_id in &city_ids {
                    sqlx::query!(
                        r#"INSERT INTO markets
                           (run_id, tick, good_enum, city_id,
                            clearing_price_mj, bid_volume, ask_volume, cleared_volume,
                            unmet_demand, unmet_supply, regime)
                           VALUES ('mbench', ?, ?, ?, 500000, 10000, 9500, 9500, 500, 0, 'market')"#,
                        tick, good, city_id.to_string()
                    )
                    .execute(&pool).await.unwrap();
                }
            }
        }
        pool
    });

    let mut group = c.benchmark_group("market_query");

    group.bench_function("price_history_energy_all_cities_1000ticks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _rows = sqlx::query!(
                    "SELECT tick, city_id, clearing_price_mj, unmet_demand
                     FROM markets
                     WHERE run_id = 'mbench' AND good_enum = 'energy'
                     ORDER BY tick ASC"
                )
                .fetch_all(&pool).await.unwrap();
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_bulk_citizen_insert,
    bench_query_last_100_ticks,
    bench_market_price_query
);
criterion_main!(benches);
```

**Performance targets and current baselines:**

| Benchmark | Target | Expected at |
|-----------|--------|-------------|
| Bulk insert 1M citizen rows | < 5s | NVMe SSD, SQLite WAL |
| Query last 100 ticks (10 nations) | < 100ms | After index warm-up |
| Market price history (8 goods × 20 cities × 1000 ticks) | < 500ms | With idx_markets_run_good |
| Full hash chain verification (10k-tick run) | < 60s | Single-threaded replay |
| Parquet export (full run, 10k ticks) | < 30s | NVMe, arrow2 zstd |
| Snapshot write (full, compressed) | < 50ms | Per tick boundary |

### 10.4 Test Fixtures

Three SQL fixture files provide standard test datasets. They are loaded by integration tests and can be loaded manually for development.

**`tests/fixtures/minimal_run.sql`**

A minimal valid run with 2 nations, 2 cities, 10 citizens, 10 ticks. Used for unit tests and fast CI.

```sql
-- tests/fixtures/minimal_run.sql
-- Minimal run fixture: 2 nations, 2 cities, 10 citizens, 10 ticks
-- Run ID: 00000000-0000-0000-0000-000000000001

INSERT INTO runs (run_id, scenario_id, seed, start_tick, end_tick, status, params, version)
VALUES ('00000000-0000-0000-0000-000000000001', 'minimal-scenario', 42, 0, 9, 'completed', '{}', '1.0.0');

INSERT INTO nations (run_id, tick, nation_id, name, ideology_vector, stability, legitimacy,
                     population_total, population_growth, gdp_millijoules, energy_surplus_mj,
                     food_surplus_mu, gini_coefficient, at_war)
VALUES
  ('00000000-0000-0000-0000-000000000001', 0,
   'aaaaaaaa-0000-0000-0000-000000000001', 'Nation Alpha',
   '[0.5,-0.3,0.1,-0.2,0.0,0.4,-0.1,0.2]',
   7000, 6500, 1000000, 500, 500000000, 10000, 50000, 3200, 0),

  ('00000000-0000-0000-0000-000000000001', 0,
   'bbbbbbbb-0000-0000-0000-000000000001', 'Nation Beta',
   '[-0.4,0.6,-0.2,0.1,-0.3,-0.5,0.3,-0.1]',
   6000, 7000, 800000, 300, 400000000, -5000, 40000, 4100, 0);

INSERT INTO cities (run_id, tick, city_id, nation_id, name, position_x, position_y,
                    population, energy_balance_mj, food_balance_mu, housing_capacity,
                    employed_count, unemployed_count, happiness_avg, infrastructure_level, is_capital)
VALUES
  ('00000000-0000-0000-0000-000000000001', 0,
   'cccccccc-0000-0000-0000-000000000001', 'aaaaaaaa-0000-0000-0000-000000000001',
   'Alpha Capital', 10, 10, 800000, 10000, 50000, 1000000, 600000, 50000, 6500, 70, 1),

  ('00000000-0000-0000-0000-000000000001', 0,
   'dddddddd-0000-0000-0000-000000000001', 'bbbbbbbb-0000-0000-0000-000000000001',
   'Beta Capital', 30, 10, 650000, -5000, 40000, 900000, 480000, 60000, 5800, 55, 1);

INSERT INTO climate_state (run_id, tick, global_temp_offset_mc, co2_ppm_mc, sea_level_rise_mm,
                            ocean_acidification_mu, arctic_ice_pct, active_tipping_points,
                            extreme_weather_count, renewable_capacity_pct)
VALUES ('00000000-0000-0000-0000-000000000001', 0,
        1200, 420000, 200, 150000, 7500, '[]', 0, 2000);

-- Additional ticks 1-9 follow same pattern (omitted for brevity in this header comment)
-- Full fixture file contains all 10 ticks.
```

**`tests/fixtures/full_scenario.sql`**

A full research scenario with 5 nations, 20 cities, 50k citizens, 1000 ticks, including war and climate events. Used for integration and performance tests. File size: ~50 MB. Generated by `civ fixtures generate --preset full`.

**`tests/fixtures/stress_test_run.sql`**

Extreme-scale fixture: 10 nations, 100 cities, 1M citizens at tick 0 only. Used for write-path performance tests. Generated by `civ fixtures generate --preset stress`.

**Fixture loading in tests:**

```rust
// tests/common/fixtures.rs

pub async fn load_fixture(pool: &SqlitePool, fixture_name: &str) {
    let fixture_sql = match fixture_name {
        "minimal_run" => include_str!("../fixtures/minimal_run.sql"),
        "full_scenario" => include_str!("../fixtures/full_scenario.sql"),
        "stress_test_run" => include_str!("../fixtures/stress_test_run.sql"),
        _ => panic!("Unknown fixture: {}", fixture_name),
    };

    // Execute each statement
    for stmt in fixture_sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        sqlx::query(stmt).execute(pool).await
            .unwrap_or_else(|e| panic!("Fixture statement failed: {e}\nSQL: {stmt}"));
    }
}

// Usage in tests:
// let pool = setup_db().await;
// load_fixture(&pool, "minimal_run").await;
// let nations = fetch_nations_at_tick(&pool, "00000000-...", 0).await.unwrap();
// assert_eq!(nations.len(), 2);
```

---

## Appendix A — Enum Value Sets

### A.1 `runs.status`

| Value | Meaning |
|-------|---------|
| `running` | Simulation is actively executing |
| `completed` | Simulation reached tick_limit or a win condition |
| `failed` | Simulation aborted due to an error (conservation violation, OOM, etc.) |
| `paused` | Simulation is suspended; can be resumed |
| `archived` | Run data has been archived to .civreplay; SQLite rows pruned |

### A.2 `citizens.class_enum`

| Value | Description |
|-------|-------------|
| `subsistence` | At or below basic survival threshold; no discretionary resources |
| `working` | Basic needs met; employed in primary/secondary sector |
| `middle` | Comfortable; discretionary income; stable employment |
| `professional` | Knowledge work; high income; low dissatisfaction typically |
| `capitalist` | Owns means of production; income from capital |
| `elite` | Ruling class; high wealth; high political influence |
| `lumpenproletariat` | Chronically unemployed; disconnected from productive economy |

### A.3 `citizens.employment_status`

| Value | Description |
|-------|-------------|
| `employed` | Working for a wage or salary |
| `unemployed` | Seeking work; not employed |
| `self_employed` | Operating own business or farm |
| `retired` | No longer in labor force due to age |
| `student` | In education; not in labor force |
| `disabled` | Unable to work |

### A.4 `ledger_transfers.currency_enum`

| Value | Description | Unit |
|-------|-------------|------|
| `joule` | Physical energy currency (joule economy regimes) | Millijoules (mJ) |
| `fiat` | Government-issued currency (market regimes) | Millicredits |
| `quota` | Planned allocation quota (planned economy regimes) | Quota units |
| `labor_credit` | Labor-time certificates | Milliminutes of labor |
| `carbon_credit` | Carbon emission permits | Millitons CO2e |

### A.5 `markets.good_enum`

| Value | Description | Physical nature |
|-------|-------------|----------------|
| `energy` | Electrical/thermal energy | Physical (joules) |
| `food` | Agricultural and processed food | Physical (calories) |
| `housing` | Residential units | Physical (unit-months) |
| `medicine` | Healthcare goods and services | Physical/service |
| `capital_goods` | Industrial equipment | Physical |
| `consumer_goods` | Non-essential manufactured goods | Physical |
| `labor` | Work-hours | Service |
| `carbon_credit` | Emission permits | Financial |

### A.6 `institutions.inst_type`

| Value | Description |
|-------|-------------|
| `central_bank` | Monetary policy, money supply control |
| `planning_bureau` | Resource allocation planning in planned/joule economies |
| `regulatory_agency` | Market regulation, standards enforcement |
| `court` | Legal adjudication, contract enforcement |
| `military_command` | Armed forces coordination |
| `trade_union` | Labor collective bargaining |
| `religious_body` | Cultural/moral authority; legitimacy source |
| `media_organization` | Information production; narrative control |
| `environmental_agency` | Environmental regulation; climate policy |
| `taxation_authority` | Revenue collection; fiscal enforcement |

### A.7 `climate_state.active_tipping_points` (JSON array values)

| Value | Description | Activation threshold (approx.) |
|-------|-------------|-------------------------------|
| `west_antarctic_ice_sheet_collapse` | WAIS destabilization; multi-meter SLR | +1.5°C |
| `greenland_ice_sheet_collapse` | GIS melt; 7m SLR over centuries | +1.5°C |
| `amazon_dieback` | Rainforest transition to savanna | +2.0°C |
| `permafrost_methane_release` | Siberian/arctic CH4 emissions | +1.5°C |
| `atlantic_circulation_collapse` | AMOC shutdown; European cooling | +2.0°C |
| `coral_reef_die_off` | Mass bleaching; fishery collapse | +1.5°C |
| `boreal_forest_dieback` | Taiga to grassland transition | +3.0°C |
| `monsoon_disruption` | ITCZ shift; Asian monsoon failure | +2.5°C |

### A.8 `rng_seeds.phase_enum`

| Value | Description | Module |
|-------|-------------|--------|
| `stochastic_events` | Random event selection and outcome | `sim::events` |
| `migration` | Citizen migration destination sampling | `sim::city` |
| `war_resolution` | Battle outcome dice rolls | `sim::war` |
| `climate_perturbation` | Weather variability and tipping point rolls | `sim::climate` |
| `citizen_behavior` | Individual citizen decision sampling | `sim::citizen` |
| `institution_drift` | Capture and legitimacy random walk | `sim::institution` |

---

## Appendix B — Cross-Reference Table

| Entity | SQL Table | Rust Struct | In-memory ECS Component | .civreplay presence |
|--------|-----------|-------------|------------------------|---------------------|
| Simulation Run | `runs` | `SimRun` | — (metadata only) | Header |
| State Snapshot | `snapshots` | `Snapshot` | `WorldSnap` / `WorldDelta` | Per-frame |
| Simulation Event | `events` | `SimEvent` | `Vec<SimEvent>` in `World.event_log` | Every event |
| Nation | `nations` | `NationState` | `BTreeMap<NationId, NationState>` | Via events |
| City | `cities` | `CityState` | `BTreeMap<CityId, CityState>` | Via events |
| Citizen | `citizens` | `CitizenRecord` | `BTreeMap<CitizenId, CitizenRecord>` | Via events (sampled) |
| Ledger Transfer | `ledger_transfers` | `LedgerTransfer` | `LedgerState.transfers` | Via events |
| Market Clearing | `markets` | `MarketClearing` | `BTreeMap<(Good, CityId), MarketClearing>` | Via events |
| Climate | `climate_state` | `ClimateStateRow` / `ClimateState` | `World.climate` | Via events |
| Institution | `institutions` | `InstitutionState` | `BTreeMap<InstId, InstitutionState>` | Via events |
| War | `wars` | `WarRecord` | `BTreeMap<WarId, WarRecord>` | Via events |
| Research Run | `research_runs` | `ResearchRun` | — (metadata) | Header |
| Replay Event | `replay_events` | `ReplayEvent` | — (archive) | Primary |
| Metric | `metrics_timeseries` | `MetricRow` | — (derived) | No |
| RNG Seed | `rng_seeds` | `RngSeedRow` | `World.rng` (live state) | No |
| Schema Version | `schema_versions` | — (migration tooling) | — | No |

---

## Appendix C — SQLite File Size Estimates

Reference estimates for typical simulation runs at various scales. All estimates assume zstd compression for snapshot_bytes.

| Scale | Nations | Cities | Citizens | Ticks | SQLite Size | .civreplay Size |
|-------|---------|--------|---------|-------|-------------|----------------|
| Minimal | 2 | 4 | 10k | 1k | ~50 MB | ~5 MB |
| Small | 4 | 16 | 100k | 5k | ~2 GB | ~200 MB |
| Medium | 8 | 40 | 500k | 10k | ~15 GB | ~1 GB |
| Large | 16 | 100 | 2M | 20k | ~80 GB | ~5 GB |
| Research batch (10 runs) | 4 | 20 | 200k | 10k each | ~60 GB | ~5 GB total |

**Notes:**
- Citizens table dominates storage at scale. Enable pruning (500-tick retention) for runs beyond "Small".
- .civreplay is ~10x smaller than SQLite due to event-only format (no per-tick full rows).
- For research batches with parameter sweeps, use the parquet export path for analysis. SQLite files grow proportionally; consider PostgreSQL for concurrent multi-user research.

---

## Appendix D — Connection Pool Configuration

### D.1 SQLite Connection Pool (SQLx)

```rust
// src/db/conn.rs

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use std::path::Path;

pub async fn open(path: &Path) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .pragma("cache_size", "-65536")
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "8589934592")
        .pragma("wal_autocheckpoint", "1000");

    // SQLite with WAL supports 1 writer + N readers concurrently.
    // max_connections = 1 writer + up to 7 readers for typical research workload.
    SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect_with(options)
        .await
        .map_err(DbError::Connection)
}

pub async fn open_in_memory() -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .journal_mode(SqliteJournalMode::Memory)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(1)   // in-memory DB is connection-local in SQLite
        .connect_with(options)
        .await
        .map_err(DbError::Connection)
}
```

### D.2 PostgreSQL Connection Pool (SQLx, multi-user mode)

```rust
// src/db/pg_conn.rs (multi-user research mode)

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

pub async fn open_pg(database_url: &str) -> Result<PgPool, DbError> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(database_url)
        .await
        .map_err(DbError::Connection)
}
```

---

*End of CivLab Data Model and Database Specification*

**Spec ID:** SPEC-DATA-MODEL-CIV-001 | **Version:** 1.0.0 | **Date:** 2026-02-21
