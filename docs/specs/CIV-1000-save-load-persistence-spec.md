# CIV-1000: Save, Load, and Persistence System — Full Specification

**Spec ID:** CIV-1000
**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team
**Related Specs:**
- CIV-0001: Core Simulation Loop (deterministic tick architecture, ChaCha20Rng, BLAKE3 hash chain)
- CIV-0100 through CIV-0107: All simulation subsystems (Economy, Climate, Institutions, Citizens, War, Social, Joule Economy)
- CIV-0200: Client Protocol (JSON-RPC WebSocket interface)
- CIV-0400: AI / NPC Behavior Specification (MCTS trees, personality drift, AI memory)
- CIV-0700: Modding API (WASM mod lifecycle, mod state protocol)
- CIV-0900: Session Management (session.save / session.load delegation)

---

## Table of Contents

1. [Overview and Design Goals](#1-overview-and-design-goals)
2. [Save File Format](#2-save-file-format)
3. [SimStateSnapshot Schema](#3-simstatesnapshot-schema)
4. [AiStateSnapshot Schema](#4-aistatesnapshot-schema)
5. [Mod State Serialization](#5-mod-state-serialization)
6. [Save Operations — Rust API](#6-save-operations--rust-api)
7. [Load and Resume Sequence](#7-load-and-resume-sequence)
8. [Migration System](#8-migration-system)
9. [Database Schema](#9-database-schema)
10. [QuickSave Ring Buffer](#10-quicksave-ring-buffer)
11. [AutoSave Configuration](#11-autosave-configuration)
12. [Performance Targets](#12-performance-targets)
13. [JSON-RPC Methods](#13-json-rpc-methods)
14. [Events](#14-events)
15. [FR Traceability](#15-fr-traceability)
16. [Integration Points](#16-integration-points)
17. [Acceptance Criteria](#17-acceptance-criteria)

---

## 1. Overview and Design Goals

### 1.1 Purpose

The CivLab save/load system provides complete, deterministic, resumable snapshots of all simulation state. A save file is not a "checkpoint hint" — it is a **full re-entry point** into the simulation timeline. Given only the save file and the original scenario seed, the engine must produce tick-for-tick identical output from tick N+1 onward as it would have produced without any interruption.

This guarantee is the foundation of CivLab's research-grade audit trail. Researchers must be able to:

- Pause a 100,000-tick run at any point, ship the save file to a collaborator, and resume with identical results.
- Bisect a specific outcome by loading autosaves and replaying forward to the divergence point.
- Compare two branches of play from a common save point and analyze outcome divergence.

### 1.2 Three Save Types

| Save Type | Storage | Count Limit | Trigger | Persistence |
|-----------|---------|-------------|---------|-------------|
| **QuickSave** | In-memory ring buffer | 5 slots | Player command or keyboard shortcut | Session lifetime only; lost on crash |
| **SlotSave** | SQLite or PostgreSQL `save_slots` table + compressed file | Unlimited (user-named) | Explicit player or RPC command | Durable; survives restart |
| **AutoSave** | SQLite or PostgreSQL `autosaves` table + compressed file | 10 slots (ring) | Every N ticks (default 100) | Durable; ring-evicts oldest |

QuickSave prioritizes speed: serialization to memory only, no DB write, no compression. SlotSave and AutoSave use zstd level 3 compression and write to the configured storage backend.

### 1.3 Determinism Contract (D1-D7 Extension)

The core simulation already enforces rules D1–D7 (per CIV-0001). The save/load system extends these with two additional invariants:

**D8 — Save Completeness:** A save file must contain 100% of simulation state. No state may live outside the save envelope. Any state that affects future tick computation must be serialized.

**D9 — Resume Fidelity:** After loading a save at tick N, the computation of tick N+1 must be bit-for-bit identical to what it would have been in an uninterrupted run. This means:

- The ChaCha20Rng stream position is exactly restored.
- All component storage in bevy_ecs is exactly restored.
- All AI MCTS trees, personality parameters, and memory buffers are exactly restored.
- All mod-registered state is exactly restored.
- The BLAKE3 hash chain tail is exactly restored so the chain continues unbroken.

Violation of D9 is a critical engine bug. The test suite includes a "save-resume parity harness" that enforces this property at CI time.

### 1.4 State Inventory

All of the following must be captured in every save:

| Subsystem | Spec | State Volume | Notes |
|-----------|------|--------------|-------|
| Hex Grid | CIV-0001 | Medium | Terrain, ownership, improvements per hex |
| Joule Economy | CIV-0107 | Medium | District production/consumption, energy debt |
| Climate | CIV-0102 | Medium | CO2 grid, temperature grid, damage grid |
| Institutions | CIV-0103 | Small-Medium | FSM states, capture scores, legitimacy |
| Citizens | CIV-0103 | Large | Citizen vectors per district (dominant cost) |
| War / Diplomacy | CIV-0105 | Small | Relation matrix, treaty ledger, war state |
| Social / Ideology | CIV-0106 | Small-Medium | Ideology vectors, stress, insurgency FSM |
| RNG State | CIV-0001 | 80 bytes | ChaCha20Rng word stream |
| Tick Counter | CIV-0001 | 8 bytes | Monotonic u64 |
| BLAKE3 Hash Chain | CIV-0001 | 32 bytes | Chain tail hash |
| AI State (per nation) | CIV-0400 | Medium | Personality, memory, MCTS cache, goals |
| Mod State (per mod) | CIV-0700 | Variable | Opaque blobs, mod-controlled |
| Command Ledger | CIV-0001 | Small | Pending buffered commands at save point |

### 1.5 Non-Goals

- **Incremental/delta saves:** All saves are full snapshots. Delta compression is a future optimization; it adds merge complexity incompatible with D9 guarantees at this stage.
- **Cross-engine-version resume without migration:** Saves from engine version X are loadable in version X+1 only via the migration system (Section 8). Attempts to load an unknown format version fail loudly, not silently.
- **Client state persistence:** Client UI state (camera position, selected unit, opened panels) is not part of the simulation save. It is the client's responsibility to restore its own presentation state.
- **Network synchronization of saves:** The save system targets single-server persistence. Multi-server save reconciliation is out of scope for v1.

---

## 2. Save File Format

### 2.1 Container Types

CivLab supports two physical save representations:

**Single-file archive** (preferred): `.civsave.zst` — a zstd-compressed tar archive containing all save components. Used for SlotSave and AutoSave. Default.

**Uncompressed folder** (debug/inspection): `.civsave/` directory — raw components side-by-side. Used for development, migration testing, and human inspection. Not used in production by default.

QuickSaves bypass both: they hold `SimStateSnapshot` directly in a heap-allocated `Vec<u8>` (MessagePack-serialized, not compressed) inside the `QuickSaveRing`.

### 2.2 Archive Layout

```
my-save.civsave.zst
└── (zstd decompressed tar)
    ├── header.bin          # Magic bytes + binary header fields
    ├── metadata.json       # Human-readable sidecar, BLAKE3 of full body
    ├── state.bin           # MessagePack: SimStateSnapshot
    ├── ai_state.bin        # MessagePack: Vec<AiNationSnapshot>
    └── mod_state.bin       # MessagePack: HashMap<ModId, ModBlob>
```

ASCII diagram of the binary layout inside the archive:

```
┌─────────────────────────────────────────────────────────────┐
│  header.bin                                                  │
│  ┌──────────┬──────────┬──────────────┬────────────────┐    │
│  │  magic   │  fmt_ver │  eng_ver     │  created_at    │    │
│  │ "CIV1"   │  u16 LE  │  semver str  │  i64 unix ms   │    │
│  │  4 bytes │  2 bytes │  ≤ 32 bytes  │  8 bytes       │    │
│  └──────────┴──────────┴──────────────┴────────────────┘    │
│  ┌────────────────┬──────────────────┬────────────────────┐  │
│  │  seed_hi       │  seed_lo         │  tick              │  │
│  │  u64 LE        │  u64 LE          │  u64 LE            │  │
│  └────────────────┴──────────────────┴────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  blake3_hash   (32 bytes)                              │  │
│  │  hash covers: state.bin || ai_state.bin || mod_state.bin│  │
│  └────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  metadata.json  (UTF-8, human-readable)                      │
│  {                                                           │
│    "spec_id": "CIV-1000",                                    │
│    "format_version": 1,                                      │
│    "engine_version": "0.4.1",                                │
│    "scenario_name": "...",                                   │
│    "tick": 12500,                                            │
│    "wall_clock_duration_ms": 1250000,                        │
│    "nation_count": 6,                                        │
│    "citizen_count": 2847,                                    │
│    "active_mods": ["eco-v2", "climate-patch-1"],             │
│    "blake3_hash": "a3f9...<hex>",                            │
│    "created_at": "2026-02-21T14:30:00Z"                      │
│  }                                                           │
├─────────────────────────────────────────────────────────────┤
│  state.bin      (MessagePack encoded SimStateSnapshot)       │
│  ai_state.bin   (MessagePack encoded Vec<AiNationSnapshot>)  │
│  mod_state.bin  (MessagePack encoded HashMap<ModId,ModBlob>) │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 Magic Bytes and Header

The first 4 bytes of `header.bin` are always the ASCII literal `CIV1` (`0x43 0x49 0x56 0x31`). Any file that does not begin with these bytes is rejected immediately by the loader — no further parsing is attempted.

**Header field layout (all integers little-endian):**

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 4 | `[u8; 4]` | `magic` — must be `[0x43, 0x49, 0x56, 0x31]` |
| 4 | 2 | `u16` | `save_format_version` — bumped on breaking schema changes |
| 6 | 1 | `u8` | `engine_version_len` — byte length of engine version string |
| 7 | ≤32 | `[u8]` | `engine_version` — UTF-8 semver, zero-padded to declared len |
| 39 | 8 | `i64` | `created_at_unix_ms` — UTC milliseconds since epoch |
| 47 | 8 | `u64` | `seed_hi` — upper 64 bits of 128-bit scenario seed |
| 55 | 8 | `u64` | `seed_lo` — lower 64 bits of 128-bit scenario seed |
| 63 | 8 | `u64` | `tick` — simulation tick at time of save |
| 71 | 32 | `[u8; 32]` | `blake3_hash` — BLAKE3 of `state.bin || ai_state.bin || mod_state.bin` |

Total fixed header size: **103 bytes**.

### 2.4 BLAKE3 Hash Chain

The hash stored in `header.bin` covers the body content only (`state.bin || ai_state.bin || mod_state.bin`, concatenated in that order). It does not cover the header itself or `metadata.json` (metadata is derivative).

The hash chain integrity check:

```rust
let mut hasher = blake3::Hasher::new();
hasher.update(&state_bytes);
hasher.update(&ai_state_bytes);
hasher.update(&mod_state_bytes);
let computed = hasher.finalize();
assert_eq!(computed.as_bytes(), &header.blake3_hash, "Save integrity check failed");
```

Separately, the running simulation BLAKE3 hash chain (per CIV-0001) has its tail stored inside `state.bin` as `TickSnapshot::chain_tail: [u8; 32]`. This is the accumulating per-tick hash that continues after load — it is distinct from the save integrity hash above.

### 2.5 Serialization Format: MessagePack via `rmp-serde`

All `.bin` files inside the archive use **MessagePack** encoding via the `rmp-serde` crate with the following settings:

```rust
// Encoding
let bytes = rmp_serde::to_vec_named(&snapshot)?;

// Decoding
let snapshot: SimStateSnapshot = rmp_serde::from_slice(&bytes)?;
```

`to_vec_named` uses field names rather than array indices. This adds ~15-20% size overhead vs array encoding but provides a stable, debuggable format where individual fields can be inspected with msgpack tooling. Field names act as an additional compatibility signal during migration.

Rationale for MessagePack over bincode:
- MessagePack is self-describing: field names are present in the byte stream
- Better tooling support for inspection and migration scripting
- Cross-language compatibility (Python/JS migration scripts can read saves)
- bincode's implicit field ordering creates fragile schema coupling

### 2.6 Compression

SlotSave and AutoSave archives are compressed with **zstd level 3**:

```rust
use zstd::stream::encode_all;
let compressed = encode_all(tar_bytes.as_slice(), 3)?;
```

Level 3 is chosen to balance compression ratio (~60-75% size reduction for typical saves) against encode latency (~40-80ms for a 10MB uncompressed save on modern hardware). Levels 6+ were benchmarked and found to exceed the 500ms SlotSave budget without meaningful ratio improvement for the binary-heavy `.bin` payloads.

Decompression uses `zstd::stream::decode_all`. No streaming decompression is used in v1; the full archive is buffered in memory before decompression. Large-scenario saves (>200MB compressed) are not expected in the v1 use-case envelope.

---

## 3. SimStateSnapshot Schema

### 3.1 Top-Level Struct

```rust
use serde::{Deserialize, Serialize};

/// Complete serializable snapshot of all simulation state.
/// Every field that participates in tick computation must appear here.
/// Violation of this invariant breaks determinism rule D8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimStateSnapshot {
    /// Format version; must match CURRENT_SAVE_FORMAT_VERSION or be migrated before use.
    pub save_format_version: u16,

    /// Scenario seed (upper 64 bits).
    pub seed_hi: u64,

    /// Scenario seed (lower 64 bits).
    pub seed_lo: u64,

    /// Monotonic tick counter. Continues from this value after load; never reset.
    pub tick: u64,

    /// Scenario configuration (immutable after start; stored for audit, not replay).
    pub scenario_config: ScenarioConfigSnapshot,

    /// RNG state — must be restored exactly to guarantee D9.
    pub rng: RngSnapshot,

    /// BLAKE3 hash chain tail — the accumulating chain continues from this after load.
    pub tick_chain: TickChainSnapshot,

    /// Hex grid: terrain, ownership, improvements.
    pub hex_grid: HexGridSnapshot,

    /// Joule Economy subsystem.
    pub economy: EconomySnapshot,

    /// Climate subsystem.
    pub climate: ClimateSnapshot,

    /// Institutions subsystem.
    pub institutions: InstitutionsSnapshot,

    /// Citizens subsystem.
    pub citizens: CitizensSnapshot,

    /// War and Diplomacy subsystem.
    pub diplomacy: DiplomacySnapshot,

    /// Social, Ideology, and Insurgency subsystem.
    pub social: SocialSnapshot,

    /// Pending command buffer at the save point (commands received but not yet processed).
    pub pending_commands: Vec<SerializedCommand>,
}
```

### 3.2 RNG Snapshot

ChaCha20Rng from the `rand_chacha` crate maintains internal state as 20 words (4 bytes each = 80 bytes total). The snapshot preserves these exactly.

```rust
/// Exact serializable state of the ChaCha20Rng instance.
/// Restoring this struct bit-for-bit into a new ChaCha20Rng instance
/// produces the identical pseudo-random stream from the saved position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngSnapshot {
    /// The 20 u32 words of ChaCha20 internal state.
    /// Word layout: [key (8), counter (2), nonce (3), constants (4), block_used (1), output_buf (2)]
    /// Stored as-is from ChaCha20Rng::get_seed() equivalent extraction.
    pub words: [u32; 20],

    /// Words consumed from the current block (0..=63).
    /// Required to resume at the exact stream position.
    pub words_used: u8,
}

impl RngSnapshot {
    /// Restore a ChaCha20Rng from this snapshot.
    /// The resulting RNG will produce the identical stream from the saved position.
    pub fn restore(&self) -> rand_chacha::ChaCha20Rng {
        // Implementation uses ChaCha20Rng::from_seed with the serialized word state
        // and advances the internal block counter to `words_used`.
        // See civ-engine/src/rng/restore.rs for full implementation.
        unimplemented!("see rng/restore.rs")
    }
}
```

### 3.3 Tick Chain Snapshot

```rust
/// Snapshot of the running BLAKE3 hash chain maintained by CIV-0001.
/// After load, the chain continues from `chain_tail` as if no interruption occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickChainSnapshot {
    /// Tick at which this chain tail was computed. Must equal SimStateSnapshot::tick.
    pub at_tick: u64,

    /// The 32-byte BLAKE3 hash output at `at_tick`.
    /// Next tick computes: blake3(chain_tail || tick_state_bytes) to produce its hash.
    pub chain_tail: [u8; 32],
}
```

### 3.4 Hex Grid Snapshot

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexGridSnapshot {
    /// Grid width in hex columns.
    pub width: u16,
    /// Grid height in hex rows.
    pub height: u16,

    /// Per-hex data stored in row-major order: index = row * width + col.
    /// Length must equal width * height.
    pub hexes: Vec<HexSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexSnapshot {
    /// Terrain type. Stored as u8 discriminant for compact encoding.
    pub terrain: u8,

    /// Owning nation ID, or u16::MAX if unowned.
    pub owner_nation: u16,

    /// Improvement type installed, or 0 for none.
    pub improvement: u8,

    /// Improvement build progress (0-100 fixed point, i.e. 0..=10000 where 10000 = 100%).
    pub improvement_progress: u16,

    /// Elevation in decameters above sea level (i16 allows negative for ocean floor).
    pub elevation_dm: i16,

    /// Current pollution level (parts per million, stored as u32).
    pub pollution_ppm: u32,

    /// Habitability score (0-1 as FixedI32<U16> raw i32).
    pub habitability_raw: i32,
}
```

### 3.5 Economy Snapshot

```rust
/// Serialized state of the Joule Economy subsystem (CIV-0107).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomySnapshot {
    /// Per-district economic state. Keyed by district ID (u32).
    pub districts: Vec<DistrictEconomySnapshot>,

    /// Per-nation aggregate economic state. Keyed by nation ID (u16).
    pub nations: Vec<NationEconomySnapshot>,

    /// Global energy market clearing price (MilliCredits per KiloJoule), as raw i64.
    pub energy_price_mc_per_kj: i64,

    /// Global commodity market snapshot.
    pub commodity_markets: Vec<CommodityMarketSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictEconomySnapshot {
    pub district_id: u32,
    pub nation_id: u16,

    /// Energy produced this tick (KiloJoules as raw i64 newtype).
    pub energy_produced_kj: i64,
    /// Energy consumed this tick (KiloJoules as raw i64 newtype).
    pub energy_consumed_kj: i64,
    /// Accumulated energy debt (KiloJoules, negative = deficit).
    pub energy_debt_kj: i64,

    /// Treasury balance (MilliCredits as raw i64 newtype).
    pub treasury_mc: i64,
    /// Tax collection rate (FixedI32<U16> as raw i32).
    pub tax_rate_raw: i32,
    /// Production allocation vector (one entry per sector, sum = 1.0 as FixedI32<U16> raw i32).
    pub allocation_vector: Vec<i32>,

    /// Current production regime: 0=Command, 1=Mixed, 2=Market.
    pub regime: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationEconomySnapshot {
    pub nation_id: u16,
    /// National treasury (MilliCredits as raw i64).
    pub treasury_mc: i64,
    /// Debt outstanding (MilliCredits as raw i64).
    pub debt_mc: i64,
    /// Credit rating (0-1000, integer basis points).
    pub credit_rating_bp: u16,
    /// GDP proxy: total production value last tick (MilliCredits).
    pub gdp_proxy_mc: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityMarketSnapshot {
    /// Commodity type ID.
    pub commodity_id: u16,
    /// Global supply (MilliCredits equivalent).
    pub supply_mc: i64,
    /// Global demand (MilliCredits equivalent).
    pub demand_mc: i64,
    /// Clearing price (MilliCredits per unit, raw i64).
    pub price_mc: i64,
}
```

### 3.6 Climate Snapshot

```rust
/// Serialized state of the Climate subsystem (CIV-0102).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateSnapshot {
    /// CO2 concentration grid (ppm * 100 as u32, grid resolution = hex grid).
    /// Length must equal HexGridSnapshot::width * height.
    pub co2_grid_ppm_x100: Vec<u32>,

    /// Surface temperature grid (degrees Celsius * 100 as i32).
    /// Allows -100.00°C to +327.67°C in 0.01°C increments.
    pub temp_grid_c_x100: Vec<i32>,

    /// Cumulative damage grid per hex (0-1 as FixedI32<U16> raw i32).
    pub damage_grid_raw: Vec<i32>,

    /// Global mean CO2 (ppm * 100).
    pub global_co2_ppm_x100: u32,

    /// Global mean surface temperature (°C * 100).
    pub global_temp_c_x100: i32,

    /// Sea level relative to baseline (centimeters, signed; negative = below baseline).
    pub sea_level_cm: i32,

    /// Active climate event count (used for FSM tracking).
    pub active_event_count: u16,

    /// Per-nation emissions this tick (KiloJoules * emissions factor, as raw i64 per nation).
    pub nation_emissions: Vec<NationEmissionsSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationEmissionsSnapshot {
    pub nation_id: u16,
    pub co2_emitted_kg_x1000: i64,
    pub cumulative_co2_kg_x1000: i64,
}
```

### 3.7 Institutions Snapshot

```rust
/// Serialized state of the Institutions subsystem (CIV-0103).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionsSnapshot {
    /// Per-nation institution state.
    pub nations: Vec<NationInstitutionsSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationInstitutionsSnapshot {
    pub nation_id: u16,

    /// Active institutions by type ID.
    pub active_institutions: Vec<InstitutionSnapshot>,

    /// Legitimacy score (0-1 as FixedI32<U16> raw i32).
    pub legitimacy_raw: i32,

    /// Capacity score (0-1 as FixedI32<U16> raw i32).
    pub capacity_raw: i32,

    /// Fiscal extraction rate (0-1 as FixedI32<U16> raw i32).
    pub extraction_raw: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionSnapshot {
    /// Institution type discriminant (maps to InstitutionType enum).
    pub institution_type: u16,

    /// FSM state discriminant.
    pub fsm_state: u8,

    /// Capture score by faction (faction_id → capture_score as raw i32).
    pub capture_scores: Vec<(u16, i32)>,

    /// Strength score (0-1 as FixedI32<U16> raw i32).
    pub strength_raw: i32,

    /// Corruption score (0-1 as FixedI32<U16> raw i32).
    pub corruption_raw: i32,

    /// Ticks since last reform.
    pub ticks_since_reform: u32,
}
```

### 3.8 Citizens Snapshot

Citizens are the dominant save-size contributor. A 10,000-citizen scenario produces approximately 60-70MB of uncompressed citizen data before fixed-point compression. Citizens are stored in a flat vector per district with a varint-compressed representation.

```rust
/// Serialized state of the Citizens subsystem (CIV-0103).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizensSnapshot {
    /// Per-district citizen collections.
    pub districts: Vec<DistrictCitizensSnapshot>,

    /// Total citizen count across all districts. Used for sanity check on load.
    pub total_citizen_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictCitizensSnapshot {
    pub district_id: u32,
    pub nation_id: u16,

    /// Citizen records for this district.
    /// Stored as a MessagePack-encoded flat array for compact representation.
    pub citizens: Vec<CitizenRecord>,

    /// Aggregate statistics (pre-computed; redundant but validated on load for fast audit).
    pub aggregate_loyalty_raw: i32,       // mean, FixedI32<U16>
    pub aggregate_wellbeing_raw: i32,     // mean, FixedI32<U16>
    pub aggregate_productivity_raw: i32,  // mean, FixedI32<U16>
    pub unrest_index_raw: i32,            // district-level, FixedI32<U16>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizenRecord {
    /// Citizen ID (unique within session; monotonically assigned).
    pub citizen_id: u32,

    /// Occupation sector (maps to Sector enum).
    pub sector: u8,

    /// Loyalty to current government (FixedI32<U16> raw i32).
    pub loyalty_raw: i32,

    /// Wellbeing composite (FixedI32<U16> raw i32).
    pub wellbeing_raw: i32,

    /// Productivity (FixedI32<U16> raw i32).
    pub productivity_raw: i32,

    /// Ideology vector: [8 i16 values] representing position in 8D ideology space.
    /// Each dimension is FixedI16 in range [-1.0, 1.0] stored as i16 * 32768.
    pub ideology_vec: [i16; 8],

    /// Health (0-1 as u16 * 65535).
    pub health_u16: u16,

    /// Age in simulation years (u16, max 150).
    pub age_years: u16,

    /// Education level (0-10).
    pub education: u8,

    /// Insurgency participation flag (0=non-participant, 1=passive, 2=active).
    pub insurgency_role: u8,
}
```

### 3.9 Diplomacy Snapshot

```rust
/// Serialized state of the War and Diplomacy subsystem (CIV-0105).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomacySnapshot {
    /// NxN relation matrix. Index = nation_a * nation_count + nation_b.
    /// Stores the DiplomaticRelation between every pair of nations.
    pub relation_matrix: Vec<DiplomaticRelationRecord>,

    /// Total nation count (N); used to interpret the flat matrix.
    pub nation_count: u16,

    /// Active treaties.
    pub treaties: Vec<TreatySnapshot>,

    /// Active wars.
    pub wars: Vec<WarSnapshot>,

    /// Shadow network state (covert operations, spy placement, infiltration levels).
    pub shadow_networks: Vec<ShadowNetworkSnapshot>,

    /// Diplomatic event log (last 50 events for AI context).
    pub recent_events: Vec<DiplomaticEventRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticRelationRecord {
    pub nation_a: u16,
    pub nation_b: u16,
    /// Relation score (-1000 to +1000 integer; -1000 = maximum hostility).
    pub score: i16,
    /// FSM state (maps to DiplomacyState enum).
    pub state: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatySnapshot {
    pub treaty_id: u32,
    pub treaty_type: u8,
    pub signatories: Vec<u16>,
    pub signed_at_tick: u64,
    pub expires_at_tick: Option<u64>,
    /// Treaty terms encoded as CBOR (variable structure per treaty type).
    pub terms_cbor: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarSnapshot {
    pub war_id: u32,
    pub attacker: u16,
    pub defender: u16,
    pub started_at_tick: u64,
    /// War weariness per nation (0-1 as FixedI32<U16> raw i32).
    pub weariness: Vec<(u16, i32)>,
    /// Territorial control changes since war start (hex_index → controlling_nation).
    pub territorial_changes: Vec<(u32, u16)>,
    /// Pending peace terms (if any).
    pub pending_peace_terms: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowNetworkSnapshot {
    pub owning_nation: u16,
    /// Spy placements: (target_nation, hex_index, infiltration_level 0-100).
    pub spy_placements: Vec<(u16, u32, u8)>,
    /// Covert operation FSM states: (operation_id, fsm_state, progress 0-100).
    pub active_operations: Vec<(u32, u8, u8)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticEventRecord {
    pub at_tick: u64,
    pub event_type: u8,
    pub actor_nation: u16,
    pub target_nation: Option<u16>,
    pub summary_hash: u64, // FNV-1a hash of event description (for dedup/reference)
}
```

### 3.10 Social Snapshot

```rust
/// Serialized state of the Social, Ideology, and Insurgency subsystem (CIV-0106).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSnapshot {
    /// Per-nation social state.
    pub nations: Vec<NationSocialSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationSocialSnapshot {
    pub nation_id: u16,

    /// Mean ideology vector for the nation (8D, same encoding as CitizenRecord).
    pub mean_ideology_vec: [i16; 8],

    /// Social stress index (0-1 as FixedI32<U16> raw i32).
    pub stress_raw: i32,

    /// Active insurgency FSM state (0=None, 1=Latent, 2=Active, 3=Civil War).
    pub insurgency_fsm_state: u8,

    /// Insurgency strength (0-1 as FixedI32<U16> raw i32). Meaningful only if state >= 1.
    pub insurgency_strength_raw: i32,

    /// Active factions: (faction_id, faction_ideology_vec, faction_strength_raw).
    pub factions: Vec<FactionSnapshot>,

    /// Suppression level (0-1 as FixedI32<U16> raw i32).
    pub suppression_raw: i32,

    /// Propaganda effectiveness (0-1 as FixedI32<U16> raw i32).
    pub propaganda_effectiveness_raw: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSnapshot {
    pub faction_id: u16,
    pub ideology_vec: [i16; 8],
    pub strength_raw: i32,
    pub ticks_active: u64,
    pub allied_with_insurgency: bool,
    pub controlling_districts: Vec<u32>,
}
```

### 3.11 Scenario Config Snapshot

```rust
/// Immutable scenario configuration captured in the save for audit purposes.
/// This is NOT used for simulation restoration (config is already loaded from scenario file).
/// It is stored to detect config drift between save and load contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfigSnapshot {
    pub scenario_id: String,
    pub scenario_version: String,
    pub scenario_hash: [u8; 32], // BLAKE3 of the scenario JSON/YAML at load time
    pub hex_grid_width: u16,
    pub hex_grid_height: u16,
    pub nation_count: u16,
    pub initial_citizen_count: u32,
    pub tick_duration_ms: u32,   // 100 in all current configs
    pub max_ticks: Option<u64>,
}
```

### 3.12 Serialized Command

```rust
/// A command buffered but not yet processed at the save point.
/// After load, these commands are re-injected into the command buffer
/// before tick N+1 executes, preserving the exact command queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCommand {
    /// Raw JSON-RPC command bytes (the exact bytes received from the client).
    pub raw_json: Vec<u8>,
    /// Client session ID that issued the command.
    pub client_session_id: String,
    /// Monotonic command sequence number (for ordering).
    pub sequence: u64,
    /// Tick at which this command was received.
    pub received_at_tick: u64,
}
```

---

## 4. AiStateSnapshot Schema

### 4.1 Top-Level AI Snapshot

```rust
/// Complete serializable snapshot of all AI nation state.
/// One `AiNationSnapshot` per AI-controlled nation. Human-controlled nations
/// have no entry here; their "AI" is the human player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiStateSnapshot {
    /// Format version for the AI state blob (independent of main save version).
    pub ai_state_version: u16,

    /// Per-nation AI snapshots.
    pub nations: Vec<AiNationSnapshot>,
}
```

### 4.2 Per-Nation AI Snapshot

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiNationSnapshot {
    pub nation_id: u16,

    /// Personality parameter matrix (CIV-0400 Section 4).
    pub personality: PersonalityParams,

    /// Current strategic goals (ordered by priority).
    pub goals: Vec<StrategicGoal>,

    /// AI memory: betrayal records, battle outcomes, economic snapshots.
    pub memory: AiMemory,

    /// Cached MCTS result from the last MCTS evaluation (if difficulty >= 4).
    /// None if AI runs heuristic-only (difficulty < 4) or MCTS was not yet run.
    pub mcts_result_cache: Option<MctsResultCache>,

    /// Threat model: assessed threat score per other nation.
    pub threat_model: Vec<ThreatEntry>,

    /// Diplomatic relationship scores maintained by the AI (separate from DiplomacySnapshot
    /// which records the engine's authoritative state; this is the AI's internal model).
    pub ai_relation_scores: Vec<AiRelationEntry>,

    /// Ticks until next strategic reassessment.
    pub ticks_until_reassess: u32,

    /// Current difficulty level (affects AI behavior).
    pub difficulty: u8,
}
```

### 4.3 Personality Parameters

```rust
/// 18-parameter personality matrix for AI nation behavior.
/// All values stored as i32 (scaled by 1000; 1000 = 1.0 normalized).
/// See CIV-0400 Section 4 for full semantics of each parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityParams {
    /// Aggressiveness (0-1000): tendency toward military action.
    pub aggressiveness: i32,
    /// Expansionism (0-1000): priority on territorial growth.
    pub expansionism: i32,
    /// Diplomacy (0-1000): preference for negotiated resolution.
    pub diplomacy: i32,
    /// Economic focus (0-1000): priority on economic growth vs military.
    pub economic_focus: i32,
    /// Risk tolerance (0-1000): willingness to take high-variance actions.
    pub risk_tolerance: i32,
    /// Grudge memory (0-1000): persistence of betrayal memory decay.
    pub grudge_memory: i32,
    /// Alliance loyalty (0-1000): tendency to honor agreements.
    pub alliance_loyalty: i32,
    /// Isolationism (0-1000): preference for non-intervention.
    pub isolationism: i32,
    /// Technological focus (0-1000): priority on research/improvement.
    pub technological_focus: i32,
    /// Environmental concern (0-1000): weight on climate damage in decisions.
    pub environmental_concern: i32,
    /// Institutional strength preference (0-1000).
    pub institutional_preference: i32,
    /// Social control preference (0-1000): preference for suppression vs accommodation.
    pub social_control_preference: i32,
    /// Trade preference (0-1000): weight on trade treaty formation.
    pub trade_preference: i32,
    /// Covert operations tendency (0-1000).
    pub covert_tendency: i32,
    /// Reactivity (0-1000): speed of response to external changes.
    pub reactivity: i32,
    /// Long-term planning (0-1000): depth of horizon in planning.
    pub planning_horizon: i32,
    /// Propaganda use (0-1000): tendency to use social manipulation.
    pub propaganda_use: i32,
    /// Personality drift rate (0-1000): rate at which personality evolves with events.
    pub drift_rate: i32,
}
```

### 4.4 Strategic Goals

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicGoal {
    /// Goal type discriminant (maps to StrategicGoalType enum).
    pub goal_type: u16,
    /// Target nation (if applicable), or u16::MAX.
    pub target_nation: u16,
    /// Target district (if applicable), or u32::MAX.
    pub target_district: u32,
    /// Priority weight (0-1 as FixedI32<U16> raw i32).
    pub priority_raw: i32,
    /// Tick at which goal was set.
    pub set_at_tick: u64,
    /// Optional deadline tick (None = indefinite).
    pub deadline_tick: Option<u64>,
    /// Goal-specific parameters (CBOR encoded, goal-type specific).
    pub params_cbor: Vec<u8>,
}
```

### 4.5 AI Memory

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMemory {
    /// Betrayal records: treaties broken by other nations.
    pub betrayals: Vec<BetrayalRecord>,

    /// Battle outcomes (last 20 significant engagements).
    pub battle_outcomes: Vec<BattleOutcomeRecord>,

    /// Economic snapshots: own nation's economic history (last 100 ticks).
    pub economic_snapshots: Vec<EconomicMemoryEntry>,

    /// Diplomatic overtures made and their outcomes (last 50).
    pub diplomatic_outcomes: Vec<DiplomaticOutcomeRecord>,

    /// Accumulated threat assessments (persistent, not just current tick).
    pub threat_history: Vec<ThreatHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetrayalRecord {
    pub perpetrator_nation: u16,
    pub at_tick: u64,
    pub treaty_id: u32,
    pub betrayal_severity: u8, // 0=minor, 1=moderate, 2=severe
    pub memory_decay_raw: i32, // how much this record has faded (FixedI32<U16>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleOutcomeRecord {
    pub at_tick: u64,
    pub opponent_nation: u16,
    pub outcome: u8, // 0=defeat, 1=draw, 2=victory
    pub casualty_ratio_raw: i32, // own:opponent, FixedI32<U16>
    pub territory_delta: i16, // hex count gained (+) or lost (-)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicMemoryEntry {
    pub at_tick: u64,
    pub treasury_mc: i64,
    pub gdp_proxy_mc: i64,
    pub energy_balance_kj: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticOutcomeRecord {
    pub at_tick: u64,
    pub target_nation: u16,
    pub overture_type: u8,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatHistoryEntry {
    pub at_tick: u64,
    pub threat_nation: u16,
    pub threat_score_raw: i32, // FixedI32<U16>
}
```

### 4.6 MCTS Result Cache

```rust
/// Cached result of the last MCTS evaluation.
/// On resume, the AI uses this as its starting point for the next MCTS iteration
/// rather than starting from scratch, maintaining continuity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MctsResultCache {
    /// Tick at which this MCTS result was computed.
    pub computed_at_tick: u64,

    /// The winning action from the last MCTS evaluation.
    pub best_action: SerializedMctsAction,

    /// Utility score of the best action (FixedI32<U16> raw i32).
    pub best_utility_raw: i32,

    /// MCTS tree statistics (iteration count, depth reached).
    pub iterations: u32,
    pub max_depth: u8,

    /// Top-5 alternatives considered (for AI observability after resume).
    pub alternatives: Vec<MctsAlternative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMctsAction {
    /// Action type discriminant.
    pub action_type: u16,
    /// Action parameters (CBOR encoded).
    pub params_cbor: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MctsAlternative {
    pub action: SerializedMctsAction,
    pub utility_raw: i32,
    pub visit_count: u32,
}
```

### 4.7 Threat and Relation Entries

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEntry {
    pub nation_id: u16,
    pub threat_score_raw: i32,  // FixedI32<U16>
    pub last_updated_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRelationEntry {
    pub nation_id: u16,
    /// AI's internal diplomatic score (not the engine's authoritative score).
    pub ai_score: i16,  // -1000 to +1000
}
```

---

## 5. Mod State Serialization

### 5.1 ModStateSave Trait

All WASM mods that maintain persistent state must implement the `ModStateSave` trait on their host-side state proxy. The trait is defined in the `civlab-host` crate and called by the save system during snapshot construction.

```rust
/// Trait that mod host-side proxies implement to participate in save/load.
/// Called once per save for each registered mod.
pub trait ModStateSave: Send + Sync {
    /// Return the mod's unique identifier. Must be stable across versions.
    fn mod_id(&self) -> &str;

    /// Return the mod's current semantic version string.
    fn mod_version(&self) -> &str;

    /// Return the schema version of the serialized blob format.
    /// Bump this when the blob layout changes incompatibly.
    fn schema_version(&self) -> u16;

    /// Serialize all mod state to a byte blob.
    /// The format of the blob is entirely mod-controlled.
    /// The blob will be prefixed with a standard header by the save system.
    fn serialize(&self) -> Result<Vec<u8>, ModSaveError>;

    /// Restore mod state from a previously serialized blob.
    /// Called during load BEFORE the first resumed tick executes.
    fn deserialize(&mut self, bytes: &[u8]) -> Result<(), ModLoadError>;
}
```

### 5.2 Mod Save Registry

```rust
use std::collections::HashMap;

/// Registry of all mod save handlers.
/// Populated at engine startup when mods are loaded.
pub struct ModSaveRegistry {
    handlers: HashMap<String, Box<dyn ModStateSave>>,
}

impl ModSaveRegistry {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// Register a mod's save handler. Called by the mod loader at startup.
    pub fn register(&mut self, handler: Box<dyn ModStateSave>) {
        let id = handler.mod_id().to_string();
        self.handlers.insert(id, handler);
    }

    /// Serialize all mod state for inclusion in a save.
    pub fn serialize_all(&self) -> Result<HashMap<String, ModBlob>, ModSaveError> {
        let mut result = HashMap::new();
        for (id, handler) in &self.handlers {
            let payload = handler.serialize()?;
            let blob = ModBlob {
                mod_id: id.clone(),
                mod_version: handler.mod_version().to_string(),
                schema_version: handler.schema_version(),
                payload,
            };
            result.insert(id.clone(), blob);
        }
        Ok(result)
    }

    /// Restore all mod state from a save.
    /// Unknown mod IDs in the save are warned and skipped.
    pub fn deserialize_all(
        &mut self,
        blobs: HashMap<String, ModBlob>,
    ) -> Result<Vec<String>, ModLoadError> {
        let mut skipped = Vec::new();
        for (id, blob) in blobs {
            match self.handlers.get_mut(&id) {
                Some(handler) => {
                    handler.deserialize(&blob.payload)?;
                }
                None => {
                    tracing::warn!(mod_id = %id, "Mod not loaded; skipping saved mod state");
                    skipped.push(id);
                }
            }
        }
        Ok(skipped)
    }
}
```

### 5.3 Mod Blob Format

```rust
/// Serializable wrapper for a single mod's state blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModBlob {
    /// Stable mod identifier (e.g., "com.example.eco-extension").
    pub mod_id: String,

    /// Mod version at time of save (semver string).
    pub mod_version: String,

    /// Schema version of the payload format.
    pub schema_version: u16,

    /// Opaque payload bytes produced by ModStateSave::serialize().
    pub payload: Vec<u8>,
}
```

### 5.4 Unknown Mod Policy

When loading a save that references a mod that is not currently loaded:

1. **Warn** via `tracing::warn!` with `mod_id` and `schema_version`.
2. **Skip** the blob — do not fail the load.
3. **Record** the skipped mod in `LoadedSave::skipped_mods: Vec<String>`.
4. **Notify** the caller via `LoadedSave` so it can surface the warning to the player.

This is the only non-fatal skip in the load sequence. All other errors (hash mismatch, corrupt state.bin, unknown format version) are hard failures.

---

## 6. Save Operations — Rust API

### 6.1 Core Types

```rust
pub type SaveResult = Result<SaveMetadata, SaveError>;

/// Metadata returned after a successful save operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Unique save ID (UUID v4).
    pub save_id: String,
    /// Slot name (empty string for QuickSave and AutoSave).
    pub slot_name: String,
    /// Save type discriminant.
    pub save_type: SaveType,
    /// Simulation tick at save point.
    pub tick: u64,
    /// Compressed size in bytes (0 for QuickSave, which is not compressed).
    pub size_bytes: usize,
    /// Wall-clock time to complete the save operation (milliseconds).
    pub duration_ms: u64,
    /// BLAKE3 hash of body (hex string).
    pub state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveType {
    Quick,
    Slot,
    Auto,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),
    #[error("Compression failed: {0}")]
    Compression(String),
    #[error("Database write failed: {0}")]
    Database(String),
    #[error("Mod save failed for mod {mod_id}: {reason}")]
    ModSave { mod_id: String, reason: String },
    #[error("QuickSave ring overflow: max_quicksave_mb limit reached")]
    QuickSaveMemoryLimit,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 6.2 QuickSave

```rust
/// Save to the in-memory QuickSave ring buffer.
/// No compression. No DB write. Lowest latency path.
///
/// # Performance target: < 50ms for 1000-citizen scenario
pub fn save_quick(
    state: &SimState,
    ai_states: &[AiNationState],
    mod_registry: &ModSaveRegistry,
    ring: &mut QuickSaveRing,
) -> SaveResult {
    let start = std::time::Instant::now();

    // 1. Serialize all state to MessagePack
    let sim_snapshot = extract_sim_snapshot(state)?;
    let ai_snapshot = extract_ai_snapshot(ai_states)?;
    let mod_blobs = mod_registry.serialize_all()
        .map_err(|e| SaveError::ModSave { mod_id: e.mod_id, reason: e.reason })?;

    let state_bytes = rmp_serde::to_vec_named(&sim_snapshot)?;
    let ai_bytes = rmp_serde::to_vec_named(&ai_snapshot)?;
    let mod_bytes = rmp_serde::to_vec_named(&mod_blobs)?;

    // 2. Compute BLAKE3 integrity hash
    let mut hasher = blake3::Hasher::new();
    hasher.update(&state_bytes);
    hasher.update(&ai_bytes);
    hasher.update(&mod_bytes);
    let hash = hasher.finalize();

    // 3. Check memory ceiling
    let total_bytes = state_bytes.len() + ai_bytes.len() + mod_bytes.len();
    ring.check_capacity(total_bytes)?;

    // 4. Build QuickSave record
    let save_id = uuid::Uuid::new_v4().to_string();
    let qs = QuickSave {
        save_id: save_id.clone(),
        tick: state.tick,
        state_bytes,
        ai_bytes,
        mod_bytes,
        blake3_hash: *hash.as_bytes(),
        created_at: chrono::Utc::now(),
    };

    // 5. Push to ring (evicts oldest if full)
    ring.push(qs);

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(SaveMetadata {
        save_id,
        slot_name: String::new(),
        save_type: SaveType::Quick,
        tick: state.tick,
        size_bytes: total_bytes,
        duration_ms,
        state_hash: hash.to_hex().to_string(),
    })
}
```

### 6.3 SlotSave

```rust
/// Save to a named slot in the persistent storage backend (SQLite or PostgreSQL).
/// Compresses to zstd level 3. Writes metadata to DB.
///
/// # Performance target: < 500ms including compression and DB write
pub fn save_slot(
    state: &SimState,
    ai_states: &[AiNationState],
    mod_registry: &ModSaveRegistry,
    slot_name: &str,
    session_id: &str,
    db: &Db,
    file_store: &FileStore,
) -> SaveResult {
    let start = std::time::Instant::now();

    // 1. Extract snapshots
    let sim_snapshot = extract_sim_snapshot(state)?;
    let ai_snapshot = extract_ai_snapshot(ai_states)?;
    let mod_blobs = mod_registry.serialize_all()
        .map_err(|e| SaveError::ModSave { mod_id: e.mod_id, reason: e.reason })?;

    // 2. Serialize
    let state_bytes = rmp_serde::to_vec_named(&sim_snapshot)?;
    let ai_bytes = rmp_serde::to_vec_named(&ai_snapshot)?;
    let mod_bytes = rmp_serde::to_vec_named(&mod_blobs)?;

    // 3. Hash
    let mut hasher = blake3::Hasher::new();
    hasher.update(&state_bytes);
    hasher.update(&ai_bytes);
    hasher.update(&mod_bytes);
    let hash = hasher.finalize();
    let hash_hex = hash.to_hex().to_string();

    // 4. Build tar archive in memory
    let tar_bytes = build_tar_archive(
        &state.config,
        state.tick,
        session_id,
        slot_name,
        &hash,
        &state_bytes,
        &ai_bytes,
        &mod_bytes,
    )?;

    // 5. Compress with zstd level 3
    let compressed = zstd::encode_all(tar_bytes.as_slice(), 3)
        .map_err(|e| SaveError::Compression(e.to_string()))?;
    let compressed_size = compressed.len();

    // 6. Write to file store
    let save_id = uuid::Uuid::new_v4().to_string();
    let filename = format!("slot_{}_{}.civsave.zst", slug(slot_name), save_id);
    file_store.write(&filename, compressed)?;

    // 7. Write metadata to DB
    db.upsert_save_slot(SaveSlotRecord {
        id: save_id.clone(),
        session_id: session_id.to_string(),
        slot_name: slot_name.to_string(),
        tick: state.tick as i64,
        save_format_version: CURRENT_SAVE_FORMAT_VERSION as i16,
        state_hash: hash.as_bytes().to_vec(),
        file_path: filename,
        metadata_json: build_metadata_json(state, session_id, slot_name, &hash_hex),
        created_at: chrono::Utc::now(),
    })?;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(SaveMetadata {
        save_id,
        slot_name: slot_name.to_string(),
        save_type: SaveType::Slot,
        tick: state.tick,
        size_bytes: compressed_size,
        duration_ms,
        state_hash: hash_hex,
    })
}
```

### 6.4 AutoSave

```rust
/// Save to the autosave ring in the persistent storage backend.
/// Naming: autosave_{session_id}_{tick:010}.civsave.zst
/// Evicts the oldest autosave when the ring is full.
///
/// # Performance target: < 500ms (same as SlotSave)
pub fn save_auto(
    state: &SimState,
    ai_states: &[AiNationState],
    mod_registry: &ModSaveRegistry,
    session_id: &str,
    config: &AutoSaveConfig,
    db: &Db,
    file_store: &FileStore,
) -> SaveResult {
    // AutoSave uses the same serialization path as SlotSave
    let result = save_slot_internal(
        state,
        ai_states,
        mod_registry,
        &format!("autosave_{:010}", state.tick),
        session_id,
        db,
        file_store,
        SaveType::Auto,
    )?;

    // Enforce ring size: evict oldest autosave if over limit
    db.evict_old_autosaves(session_id, config.max_slots as i64)?;

    Ok(result)
}
```

### 6.5 Load

```rust
/// Load a save by ID. Verifies integrity, deserializes all state, returns LoadedSave.
///
/// # Errors
/// Hard failures: hash mismatch, unknown format version (without migration), corrupt bytes.
/// Soft failures: unknown mods (skipped, recorded in LoadedSave::skipped_mods).
pub fn load_save(
    save_id: &str,
    db: &Db,
    file_store: &FileStore,
    migration_registry: &MigrationRegistry,
) -> Result<LoadedSave, LoadError> {
    // 1. Fetch record from DB
    let record = db.get_save_slot(save_id)?
        .ok_or(LoadError::NotFound(save_id.to_string()))?;

    // 2. Read compressed archive
    let compressed = file_store.read(&record.file_path)?;

    // 3. Decompress
    let tar_bytes = zstd::decode_all(compressed.as_slice())
        .map_err(|e| LoadError::Decompression(e.to_string()))?;

    // 4. Extract archive components
    let components = extract_tar_components(&tar_bytes)?;

    // 5. Parse header and verify magic bytes
    let header = parse_header(&components.header_bytes)?;

    // 6. Verify BLAKE3 integrity
    let mut hasher = blake3::Hasher::new();
    hasher.update(&components.state_bytes);
    hasher.update(&components.ai_bytes);
    hasher.update(&components.mod_bytes);
    let computed_hash = hasher.finalize();
    if computed_hash.as_bytes() != &header.blake3_hash {
        return Err(LoadError::HashMismatch {
            expected: hex::encode(&header.blake3_hash),
            computed: computed_hash.to_hex().to_string(),
        });
    }

    // 7. Apply migrations if needed
    let (state_bytes, ai_bytes, mod_bytes) = if header.save_format_version < CURRENT_SAVE_FORMAT_VERSION {
        migration_registry.migrate(
            header.save_format_version,
            components.state_bytes,
            components.ai_bytes,
            components.mod_bytes,
        )?
    } else if header.save_format_version > CURRENT_SAVE_FORMAT_VERSION {
        return Err(LoadError::FutureVersion(header.save_format_version));
    } else {
        (components.state_bytes, components.ai_bytes, components.mod_bytes)
    };

    // 8. Deserialize
    let sim_snapshot: SimStateSnapshot = rmp_serde::from_slice(&state_bytes)
        .map_err(|e| LoadError::Deserialization(format!("state: {}", e)))?;
    let ai_snapshot: AiStateSnapshot = rmp_serde::from_slice(&ai_bytes)
        .map_err(|e| LoadError::Deserialization(format!("ai_state: {}", e)))?;
    let mod_blobs: HashMap<String, ModBlob> = rmp_serde::from_slice(&mod_bytes)
        .map_err(|e| LoadError::Deserialization(format!("mod_state: {}", e)))?;

    Ok(LoadedSave {
        save_id: save_id.to_string(),
        header,
        sim_snapshot,
        ai_snapshot,
        mod_blobs,
        skipped_mods: Vec::new(), // populated by apply_to_sim
    })
}
```

### 6.6 Verify (Hash Check Without Full Deserialize)

```rust
/// Verify a save file's integrity without fully deserializing state.
/// Reads header + recomputes BLAKE3. Fast path for health checks and UI.
///
/// # Performance: < 200ms for any save size (reads and hashes raw bytes only)
pub fn verify_save(
    save_id: &str,
    db: &Db,
    file_store: &FileStore,
) -> Result<SaveVerifyResult, VerifyError> {
    let record = db.get_save_slot(save_id)?
        .ok_or(VerifyError::NotFound(save_id.to_string()))?;

    let compressed = file_store.read(&record.file_path)?;
    let tar_bytes = zstd::decode_all(compressed.as_slice())
        .map_err(|e| VerifyError::Decompression(e.to_string()))?;
    let components = extract_tar_components(&tar_bytes)?;
    let header = parse_header(&components.header_bytes)?;

    let mut hasher = blake3::Hasher::new();
    hasher.update(&components.state_bytes);
    hasher.update(&components.ai_bytes);
    hasher.update(&components.mod_bytes);
    let computed = hasher.finalize();

    let hash_match = computed.as_bytes() == &header.blake3_hash;

    Ok(SaveVerifyResult {
        save_id: save_id.to_string(),
        valid: hash_match,
        hash_match,
        format_version: header.save_format_version,
        tick: header.tick,
        created_at_unix_ms: header.created_at_unix_ms,
        state_hash_hex: computed.to_hex().to_string(),
    })
}
```

---

## 7. Load and Resume Sequence

### 7.1 Full Load Sequence

The load sequence is strictly ordered. Any failure in steps 1-8 is a hard error that aborts the load. Steps 9-12 are restoration into live simulation objects.

```
Load Sequence
─────────────────────────────────────────────────────────────
Step 1:  Fetch save record from DB (or QuickSave ring)
Step 2:  Read compressed archive bytes from file store
Step 3:  Decompress (zstd::decode_all)
Step 4:  Extract tar components (header.bin, state.bin, ai_state.bin, mod_state.bin)
Step 5:  Parse header — verify magic bytes [CIV1], reject unknown
Step 6:  Verify BLAKE3 hash — abort on mismatch (hard fail)
Step 7:  Check format version:
         - version == current: proceed
         - version < current: apply migration chain (Section 8)
         - version > current: fail with FutureVersion error
Step 8:  Deserialize SimStateSnapshot from state.bin (rmp_serde)
Step 9:  Deserialize AiStateSnapshot from ai_state.bin (rmp_serde)
Step 10: Deserialize mod blobs from mod_state.bin (rmp_serde)
Step 11: Restore all mod state via ModSaveRegistry::deserialize_all
         - Unknown mods: warn + skip (non-fatal)
Step 12: Restore ChaCha20Rng from RngSnapshot::words + words_used
Step 13: Restore BLAKE3 chain tail from TickChainSnapshot::chain_tail
Step 14: Rebuild bevy_ecs World from SimStateSnapshot subsystems
Step 15: Re-inject pending_commands into command buffer
Step 16: Resume tick loop starting at tick N+1
─────────────────────────────────────────────────────────────
```

### 7.2 Determinism Guarantee

After completing the load sequence, the simulation is in a state that is guaranteed to be bit-for-bit identical to the pre-save state at tick N with respect to all future computation. This guarantee holds because:

1. **RNG**: ChaCha20Rng is restored to its exact stream position via `RngSnapshot::words` and `words_used`. The next call to any RNG-consuming function will produce the identical value it would have produced in an uninterrupted run.

2. **BLAKE3 chain**: The `chain_tail` from `TickChainSnapshot` is the exact 32-byte accumulation through tick N. Tick N+1's hash computation uses this tail as its input, maintaining an unbroken chain.

3. **ECS state**: All bevy_ecs components are restored from their serialized representations. Fixed-point numerics (i64 newtype wrappers for KiloJoules, MilliCredits; FixedI32<U16> for rates) are restored without floating-point rounding because they are stored as their raw integer values.

4. **AI state**: MCTS trees, personality parameters, memory, and goals are fully restored. The AI's next decision will be computed identically because all inputs to that decision — random seed position, personality, memory, threat model, scenario state — are identical.

5. **Command queue**: `pending_commands` are re-injected before tick N+1 executes, ensuring that any commands buffered at save time are processed in the same order and at the same point in the tick sequence.

### 7.3 RNG Restoration Implementation

```rust
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

/// Restore ChaCha20Rng from a saved snapshot.
/// The resulting instance produces the identical stream as the saved instance.
pub fn restore_rng(snapshot: &RngSnapshot) -> ChaCha20Rng {
    // ChaCha20Rng internals: the 20-word state maps to:
    //   words[0..4]   = constants ("expa", "nd 3", "2-by", "te k")
    //   words[4..12]  = key (256-bit seed)
    //   words[12..14] = counter (64-bit)
    //   words[14..16] = nonce
    //   words[16..20] = (implementation specific: partial output buffer position)
    //
    // We use the rand_chacha internal serialization via its `Rng` trait.
    // The snapshot was captured via equivalent extraction during save.
    //
    // Concrete implementation is in civ-engine/src/rng/restore.rs and uses
    // the `rand_chacha::ChaCha20Rng` `from_seed` + manual word injection
    // (pending stabilization of rand_chacha's serialize feature).
    //
    // Until rand_chacha exposes stable serialization, the engine saves
    // the seed + stream position (block index + words used) and reconstructs
    // by fast-forwarding from the seed. Fast-forward at 1 billion words/sec
    // is < 1ms even for 10^9 consumed words.
    let seed = extract_seed_from_words(&snapshot.words);
    let mut rng = ChaCha20Rng::from_seed(seed);
    // Fast-forward to the saved stream position
    let block_count = stream_position_from_words(&snapshot.words);
    rng.set_stream(block_count);
    // Discard `words_used` words to align sub-block position
    discard_words(&mut rng, snapshot.words_used as usize);
    rng
}
```

### 7.4 bevy_ecs World Restoration

The bevy_ecs World is not serialized directly. Instead, each subsystem snapshot is applied to a freshly constructed World:

```rust
pub fn restore_world(snapshot: &SimStateSnapshot) -> Result<bevy_ecs::world::World, LoadError> {
    let mut world = bevy_ecs::world::World::new();

    // Restore hex grid entities
    restore_hex_entities(&mut world, &snapshot.hex_grid)?;

    // Restore economy components
    restore_economy_components(&mut world, &snapshot.economy)?;

    // Restore climate resources
    restore_climate_resources(&mut world, &snapshot.climate)?;

    // Restore institution entities
    restore_institution_entities(&mut world, &snapshot.institutions)?;

    // Restore citizen entities (dominant operation; may spawn 10k+ entities)
    restore_citizen_entities(&mut world, &snapshot.citizens)?;

    // Restore diplomacy resources and entities
    restore_diplomacy_state(&mut world, &snapshot.diplomacy)?;

    // Restore social state
    restore_social_state(&mut world, &snapshot.social)?;

    Ok(world)
}
```

Entity IDs are reassigned during restoration (bevy_ecs entity IDs are not stable across World instances). All internal cross-references that use entity IDs are resolved during restoration via the index maps provided by each restore function.

---

## 8. Migration System

### 8.1 Design

Save format versions are monotonically increasing `u16` values. Breaking schema changes — adding required fields, removing fields, changing field types, restructuring nested structs — require a version bump and a corresponding migration function.

The migration registry holds a directed chain of migration functions from version N to version N+1. Loading a save of version V on an engine that understands version C applies migrations V→(V+1)→...→C sequentially. Each migration function operates on raw bytes (typically: deserialize from old schema, transform, serialize to new schema).

### 8.2 MigrationFn Type and Registry

```rust
/// A migration function transforms the raw bytes of a save component
/// from format version `from` to format version `to` (= from + 1).
/// Operates on (state_bytes, ai_bytes, mod_bytes) as a tuple.
pub type MigrationFn = fn(
    state: Vec<u8>,
    ai: Vec<u8>,
    mods: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), MigrationError>;

pub struct MigrationRegistry {
    /// Map from source version to migration function.
    /// Key = from_version; value = (to_version, fn).
    migrations: std::collections::BTreeMap<u16, (u16, MigrationFn)>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        let mut r = Self { migrations: std::collections::BTreeMap::new() };
        // Register all known migrations
        r.register(1, 2, migrate_v1_to_v2);
        r.register(2, 3, migrate_v2_to_v3);
        // Add new migrations here as format evolves
        r
    }

    pub fn register(&mut self, from: u16, to: u16, f: MigrationFn) {
        assert_eq!(to, from + 1, "Migrations must be sequential (from+1)");
        self.migrations.insert(from, (to, f));
    }

    /// Apply all needed migrations to bring bytes from `from_version` to current.
    pub fn migrate(
        &self,
        from_version: u16,
        mut state: Vec<u8>,
        mut ai: Vec<u8>,
        mut mods: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), MigrationError> {
        let mut current = from_version;
        while current < CURRENT_SAVE_FORMAT_VERSION {
            let (next, f) = self.migrations.get(&current)
                .ok_or(MigrationError::MissingMigration { from: current })?;
            tracing::info!(from = current, to = next, "Applying save migration");
            (state, ai, mods) = f(state, ai, mods)?;
            current = *next;
        }
        Ok((state, ai, mods))
    }
}
```

### 8.3 Backward Compatibility Policy

The engine supports loading saves from the **current version and the two preceding versions** (N-2 policy). For example:

| Engine Format Version | Readable Save Versions |
|-----------------------|------------------------|
| v1 | v1 |
| v2 | v1, v2 |
| v3 | v1, v2, v3 |
| v4 | v2, v3, v4 |
| v5 | v3, v4, v5 |

Saves older than N-2 are rejected with a hard error:

```
LoadError::TooOld {
    save_version: 1,
    minimum_supported: 3,
    current_version: 5,
}
```

Users must upgrade saves proactively if running old saves forward.

### 8.4 Example Migration: v1 → v2 (Adding insurgency_fsm_state)

Between v1 and v2, `NationSocialSnapshot` gained a new required field `insurgency_fsm_state: u8` (default 0 = None). The migration function adds this field with its default value:

```rust
pub fn migrate_v1_to_v2(
    state: Vec<u8>,
    ai: Vec<u8>,
    mods: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), MigrationError> {
    // Deserialize using the v1 schema (a fork of the struct with the old layout)
    let mut snapshot: SimStateSnapshotV1 = rmp_serde::from_slice(&state)
        .map_err(|e| MigrationError::Deserialization { version: 1, reason: e.to_string() })?;

    // Upgrade SocialSnapshot: add insurgency_fsm_state = 0 to each nation
    for nation in &mut snapshot.social.nations {
        // SimStateSnapshotV1::NationSocialSnapshotV1 lacks insurgency_fsm_state
        // The v2 struct adds it; we convert via a From impl
    }

    // Serialize using the v2 schema
    let v2: SimStateSnapshot = snapshot.into(); // From<SimStateSnapshotV1> converts with defaults
    let new_state = rmp_serde::to_vec_named(&v2)
        .map_err(|e| MigrationError::Serialization { version: 2, reason: e.to_string() })?;

    // AI and mod bytes are unaffected by this change
    Ok((new_state, ai, mods))
}
```

### 8.5 Migration Table

| From Version | To Version | Change Description | Affected Components |
|-------------|-----------|--------------------|--------------------|
| v1 | v2 | Add `insurgency_fsm_state` to `NationSocialSnapshot` | state.bin |
| v2 | v3 | Add `propaganda_effectiveness_raw` to `NationSocialSnapshot` | state.bin |
| Future | +1 | Add new AI goal type (extend `StrategicGoal::goal_type` range) | ai_state.bin |

Migration scripts are stored in `civ-engine/src/persistence/migrations/` with one file per version pair.

---

## 9. Database Schema

### 9.1 SQLite / PostgreSQL Compatibility

All DDL uses standard SQL-99 constructs compatible with both SQLite (via sqlx) and PostgreSQL. Type differences are handled by the ORM layer:

| Concept | SQLite | PostgreSQL |
|---------|--------|------------|
| UUIDs | `TEXT` | `UUID` |
| Timestamps | `TEXT` (ISO-8601) | `TIMESTAMPTZ` |
| JSON | `TEXT` | `JSONB` |
| Binary blobs | `BLOB` | `BYTEA` |
| Auto-increment | `INTEGER PRIMARY KEY` | `SERIAL` or `UUID` default |

The DDL below uses PostgreSQL types; the SQLite migration uses the equivalent SQLite types via the sqlx migration system.

### 9.2 save_slots Table

```sql
-- Named save slots. Each row is a player-named or API-named save.
CREATE TABLE save_slots (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id          UUID            NOT NULL,
    slot_name           TEXT            NOT NULL,
    tick                BIGINT          NOT NULL CHECK (tick >= 0),
    save_format_version SMALLINT        NOT NULL CHECK (save_format_version > 0),
    state_hash          BYTEA           NOT NULL CHECK (length(state_hash) = 32),
    file_path           TEXT            NOT NULL,
    metadata_json       JSONB           NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    -- Ensure slot names are unique within a session
    CONSTRAINT uq_session_slot UNIQUE (session_id, slot_name)
);

CREATE INDEX idx_save_slots_session_id    ON save_slots (session_id);
CREATE INDEX idx_save_slots_created_at    ON save_slots (created_at DESC);
CREATE INDEX idx_save_slots_tick          ON save_slots (tick);

-- Trigger: update updated_at on row update
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_save_slots_updated_at
    BEFORE UPDATE ON save_slots
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 9.3 autosaves Table

```sql
-- AutoSave ring: at most max_autosave_slots rows per session_id.
-- Eviction is handled by application code (save_auto) after each insert.
CREATE TABLE autosaves (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id          UUID            NOT NULL,
    tick                BIGINT          NOT NULL CHECK (tick >= 0),
    save_format_version SMALLINT        NOT NULL CHECK (save_format_version > 0),
    state_hash          BYTEA           NOT NULL CHECK (length(state_hash) = 32),
    file_path           TEXT            NOT NULL,
    metadata_json       JSONB           NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_autosaves_session_id   ON autosaves (session_id);
CREATE INDEX idx_autosaves_tick         ON autosaves (session_id, tick DESC);
CREATE INDEX idx_autosaves_created_at   ON autosaves (session_id, created_at DESC);
```

### 9.4 Autosave Eviction Function

```sql
-- Evict oldest autosaves for a session beyond the configured ring size.
-- Called by application after each autosave insert.
-- Returns the file_paths of evicted records (for file store cleanup).
CREATE OR REPLACE FUNCTION evict_old_autosaves(
    p_session_id    UUID,
    p_max_slots     INTEGER
) RETURNS TABLE(evicted_file_path TEXT) AS $$
BEGIN
    RETURN QUERY
    WITH ranked AS (
        SELECT id, file_path,
               ROW_NUMBER() OVER (PARTITION BY session_id ORDER BY tick DESC) AS rn
        FROM autosaves
        WHERE session_id = p_session_id
    ),
    to_evict AS (
        DELETE FROM autosaves
        WHERE id IN (SELECT id FROM ranked WHERE rn > p_max_slots)
        RETURNING file_path
    )
    SELECT file_path FROM to_evict;
END;
$$ LANGUAGE plpgsql;
```

### 9.5 save_migrations Table

```sql
-- Audit log of migration operations applied to saves.
CREATE TABLE save_migrations (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    from_version    SMALLINT        NOT NULL,
    to_version      SMALLINT        NOT NULL,
    applied_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    affected_saves  INTEGER         NOT NULL DEFAULT 0,
    duration_ms     INTEGER,
    initiated_by    TEXT            -- 'auto_load', 'manual', 'batch_upgrade'
);

CREATE INDEX idx_save_migrations_from_version ON save_migrations (from_version);
CREATE INDEX idx_save_migrations_applied_at   ON save_migrations (applied_at DESC);
```

### 9.6 sessions Table (Relevant Columns)

The `sessions` table (owned by CIV-0900) contains foreign keys referenced by save tables. The relevant columns for save integration:

```sql
-- Excerpt from sessions table (full definition in CIV-0900)
CREATE TABLE sessions (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    scenario_id     TEXT            NOT NULL,
    seed_hi         BIGINT          NOT NULL,
    seed_lo         BIGINT          NOT NULL,
    current_tick    BIGINT          NOT NULL DEFAULT 0,
    status          TEXT            NOT NULL DEFAULT 'running',
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    last_save_at    TIMESTAMPTZ,
    last_save_id    UUID            REFERENCES save_slots(id)
);
```

---

## 10. QuickSave Ring Buffer

### 10.1 Ring Struct

```rust
/// In-memory circular buffer of QuickSave records.
/// Capacity: QUICK_SAVE_RING_SIZE (default 5).
/// Oldest entry is evicted when the ring is full.
pub struct QuickSaveRing {
    slots: [Option<QuickSave>; QUICK_SAVE_RING_SIZE],
    head: usize,
    total_bytes: usize,
    max_bytes: usize,
}

pub const QUICK_SAVE_RING_SIZE: usize = 5;

impl QuickSaveRing {
    pub fn new(max_mb: usize) -> Self {
        Self {
            slots: [const { None }; QUICK_SAVE_RING_SIZE],
            head: 0,
            total_bytes: 0,
            max_bytes: max_mb * 1024 * 1024,
        }
    }

    /// Check if `incoming_bytes` would exceed the memory ceiling.
    pub fn check_capacity(&self, incoming_bytes: usize) -> Result<(), SaveError> {
        if self.total_bytes + incoming_bytes > self.max_bytes {
            Err(SaveError::QuickSaveMemoryLimit)
        } else {
            Ok(())
        }
    }

    /// Push a new QuickSave into the ring, evicting the oldest if needed.
    pub fn push(&mut self, save: QuickSave) {
        let size = save.state_bytes.len() + save.ai_bytes.len() + save.mod_bytes.len();

        // Evict the slot we are about to overwrite
        if let Some(ref old) = self.slots[self.head] {
            let old_size = old.state_bytes.len() + old.ai_bytes.len() + old.mod_bytes.len();
            self.total_bytes = self.total_bytes.saturating_sub(old_size);
        }

        self.slots[self.head] = Some(save);
        self.total_bytes += size;
        self.head = (self.head + 1) % QUICK_SAVE_RING_SIZE;
    }

    /// Retrieve a QuickSave by save_id. O(N) scan over at most 5 entries.
    pub fn get(&self, save_id: &str) -> Option<&QuickSave> {
        self.slots.iter().flatten().find(|s| s.save_id == save_id)
    }

    /// List all QuickSaves in insertion order (oldest first).
    pub fn list(&self) -> Vec<&QuickSave> {
        // Walk from head (oldest) to head-1 (newest) wrapping around
        let mut result = Vec::with_capacity(QUICK_SAVE_RING_SIZE);
        for i in 0..QUICK_SAVE_RING_SIZE {
            let idx = (self.head + i) % QUICK_SAVE_RING_SIZE;
            if let Some(ref s) = self.slots[idx] {
                result.push(s);
            }
        }
        result
    }

    /// Current memory usage in bytes.
    pub fn used_bytes(&self) -> usize {
        self.total_bytes
    }
}
```

### 10.2 QuickSave Record

```rust
/// A single in-memory quick save.
#[derive(Debug, Clone)]
pub struct QuickSave {
    pub save_id: String,
    pub tick: u64,
    pub state_bytes: Vec<u8>,     // MessagePack-encoded SimStateSnapshot
    pub ai_bytes: Vec<u8>,        // MessagePack-encoded AiStateSnapshot
    pub mod_bytes: Vec<u8>,       // MessagePack-encoded HashMap<String, ModBlob>
    pub blake3_hash: [u8; 32],
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

### 10.3 Memory Size Estimates

| Scenario Scale | Citizens | Estimated Uncompressed | Estimated Compressed (zstd 3) |
|---------------|----------|----------------------|-------------------------------|
| Tiny | 100 | ~2 MB | ~400 KB |
| Small | 500 | ~8 MB | ~1.5 MB |
| Medium | 1,000 | ~18 MB | ~3.5 MB |
| Large | 5,000 | ~50 MB | ~9 MB |
| Extra-Large | 10,000 | ~80 MB | ~14 MB |

QuickSave holds uncompressed bytes. With 5 slots and a medium scenario (18 MB each), peak QuickSave memory usage is ~90 MB. The default 500 MB ceiling accommodates extra-large scenarios (5 × 80 MB = 400 MB) comfortably.

---

## 11. AutoSave Configuration

### 11.1 AutoSaveConfig Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    /// Save every N simulation ticks. Default: 100 ticks (~10 seconds of sim time).
    /// Set to 0 to disable autosave.
    pub interval_ticks: u64,

    /// Maximum number of autosave slots to retain per session (ring size).
    /// Default: 10. Range: 1-50.
    pub max_slots: u8,

    /// Whether to compress autosaves with zstd. Default: true.
    /// Set to false for maximum write throughput at cost of ~60-75% larger files.
    pub compress: bool,

    /// zstd compression level for autosaves. Default: 3. Range: 1-22.
    pub compress_level: u8,

    /// If true, write autosave asynchronously (does not block tick loop).
    /// Default: true. If false, tick loop pauses during autosave write.
    pub async_write: bool,

    /// Maximum wall-clock time allowed for an autosave write before logging a warning.
    /// Default: 500ms.
    pub warn_threshold_ms: u64,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            interval_ticks: 100,
            max_slots: 10,
            compress: true,
            compress_level: 3,
            async_write: true,
            warn_threshold_ms: 500,
        }
    }
}
```

### 11.2 AutoSave Naming

AutoSave filenames follow this pattern:

```
autosave_{session_id}_{tick:010}.civsave.zst
```

Example:
```
autosave_550e8400-e29b-41d4-a716-446655440000_0000012500.civsave.zst
```

The zero-padded 10-digit tick ensures lexicographic sort order matches chronological order, enabling efficient identification of the most recent autosave without DB queries.

### 11.3 Integration with Tick Loop

The tick loop in CIV-0001 calls `should_autosave(tick, config)` at the end of each tick:

```rust
// In the tick loop (civ-engine/src/simulation/tick_loop.rs)
if autosave_config.interval_ticks > 0 && tick % autosave_config.interval_ticks == 0 {
    if autosave_config.async_write {
        // Snapshot state synchronously (fast), write asynchronously
        let snapshot = capture_snapshot_sync(&world, &ai_states, &mod_registry);
        let config_clone = autosave_config.clone();
        tokio::spawn(async move {
            if let Err(e) = write_autosave_async(snapshot, config_clone).await {
                tracing::error!(error = %e, tick = tick, "AutoSave write failed");
                // Emit session.save_failed.v1 event
            }
        });
    } else {
        save_auto(&sim_state, &ai_states, &mod_registry, session_id, &autosave_config, &db, &file_store)
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "AutoSave failed");
            });
    }
}
```

### 11.4 Session End Cleanup

On clean session end, the session manager retains the last `max_slots` autosaves and deletes the rest:

```rust
pub async fn cleanup_session_autosaves(
    session_id: &str,
    keep_count: u8,
    db: &Db,
    file_store: &FileStore,
) -> Result<usize, CleanupError> {
    let evicted_paths = db.evict_old_autosaves(session_id, keep_count as i64).await?;
    let mut deleted = 0;
    for path in evicted_paths {
        file_store.delete(&path)?;
        deleted += 1;
    }
    Ok(deleted)
}
```

---

## 12. Performance Targets

### 12.1 Latency Budgets

| Operation | Scenario Scale | Target Latency | Measurement Point |
|-----------|---------------|----------------|-------------------|
| QuickSave (serialization only) | 100 citizens | < 5 ms | wall clock, sync |
| QuickSave (serialization only) | 1,000 citizens | < 50 ms | wall clock, sync |
| QuickSave (serialization only) | 10,000 citizens | < 400 ms | wall clock, sync |
| SlotSave (serialize + compress + DB write) | 1,000 citizens | < 500 ms | wall clock, sync |
| SlotSave (serialize + compress + DB write) | 10,000 citizens | < 3,000 ms | wall clock, sync |
| AutoSave (async path) | 1,000 citizens | < 50 ms (sync phase only) | tick loop blocking time |
| Load + deserialize + World restore | 1,000 citizens | < 1,000 ms | wall clock |
| Load + deserialize + World restore | 10,000 citizens | < 5,000 ms | wall clock |
| Verify (hash check only) | any | < 200 ms | wall clock |

### 12.2 Size Estimates (Detailed)

| Component | 100 Citizens | 1,000 Citizens | 10,000 Citizens |
|-----------|-------------|----------------|-----------------|
| HexGridSnapshot | ~200 KB | ~200 KB | ~200 KB |
| EconomySnapshot | ~50 KB | ~150 KB | ~1 MB |
| ClimateSnapshot | ~300 KB | ~300 KB | ~300 KB |
| InstitutionsSnapshot | ~20 KB | ~80 KB | ~500 KB |
| CitizensSnapshot | ~1 MB | ~10 MB | ~70 MB |
| DiplomacySnapshot | ~30 KB | ~30 KB | ~30 KB |
| SocialSnapshot | ~20 KB | ~50 KB | ~300 KB |
| RNG + TickChain | < 1 KB | < 1 KB | < 1 KB |
| AiStateSnapshot | ~100 KB | ~100 KB | ~100 KB |
| ModBlobs | variable | variable | variable |
| **Total (uncompressed)** | **~2 MB** | **~11 MB** | **~72 MB** |
| **Total (zstd level 3)** | **~400 KB** | **~2 MB** | **~12 MB** |

### 12.3 Benchmarking

Save operation benchmarks are implemented in `civ-engine/benches/persistence_bench.rs` using `criterion`. CI runs these benchmarks on every PR that touches the persistence module and fails if any target is exceeded by more than 20%.

```rust
// civ-engine/benches/persistence_bench.rs (excerpt)
fn bench_quicksave_1000_citizens(c: &mut Criterion) {
    let state = SimState::fixture_1000_citizens();
    let ai_states = AiNationState::fixtures(6);
    let mod_registry = ModSaveRegistry::empty();
    let mut ring = QuickSaveRing::new(500);

    c.bench_function("quicksave_1000_citizens", |b| {
        b.iter(|| {
            save_quick(
                black_box(&state),
                black_box(&ai_states),
                black_box(&mod_registry),
                black_box(&mut ring),
            ).unwrap()
        })
    });
}
```

---

## 13. JSON-RPC Methods

All save/load operations are exposed via the existing JSON-RPC WebSocket interface (CIV-0200). Save methods are namespaced under `save.*`. They operate on the currently active session; session context is carried by the WebSocket connection's session token.

### 13.1 save.quick

Trigger a QuickSave into the in-memory ring.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "save.quick",
  "params": {}
}
```

**Response (success):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "save_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "tick": 12500,
    "duration_ms": 38,
    "ring_slot": 2,
    "ring_used_bytes": 54000000
  }
}
```

**Response (error — memory limit):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32001,
    "message": "QuickSave memory limit exceeded",
    "data": {
      "current_bytes": 524288000,
      "max_bytes": 524288000,
      "incoming_bytes": 18000000
    }
  }
}
```

**JSON Schema — Request params:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveQuickParams",
  "type": "object",
  "properties": {},
  "additionalProperties": false
}
```

**JSON Schema — Result:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveQuickResult",
  "type": "object",
  "required": ["save_id", "tick", "duration_ms", "ring_slot", "ring_used_bytes"],
  "properties": {
    "save_id":          { "type": "string", "format": "uuid" },
    "tick":             { "type": "integer", "minimum": 0 },
    "duration_ms":      { "type": "integer", "minimum": 0 },
    "ring_slot":        { "type": "integer", "minimum": 0, "maximum": 4 },
    "ring_used_bytes":  { "type": "integer", "minimum": 0 }
  },
  "additionalProperties": false
}
```

### 13.2 save.slot

Save to a named persistent slot. Creates or overwrites the named slot.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-002",
  "method": "save.slot",
  "params": {
    "slot_name": "before-war-with-rome"
  }
}
```

**Response (success):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-002",
  "result": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
    "slot_name": "before-war-with-rome",
    "tick": 12500,
    "size_bytes": 2097152,
    "duration_ms": 310,
    "state_hash": "a3f9b210e44cc4891b5671e02b2c3d555a3f9b210e44cc4891b5671e02b2c3d5"
  }
}
```

**JSON Schema — Request params:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveSlotParams",
  "type": "object",
  "required": ["slot_name"],
  "properties": {
    "slot_name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[a-zA-Z0-9_\\-\\.]+$",
      "description": "Alphanumeric, dashes, underscores, dots. No whitespace."
    }
  },
  "additionalProperties": false
}
```

**JSON Schema — Result:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveSlotResult",
  "type": "object",
  "required": ["save_id", "slot_name", "tick", "size_bytes", "duration_ms", "state_hash"],
  "properties": {
    "save_id":      { "type": "string", "format": "uuid" },
    "slot_name":    { "type": "string" },
    "tick":         { "type": "integer", "minimum": 0 },
    "size_bytes":   { "type": "integer", "minimum": 0 },
    "duration_ms":  { "type": "integer", "minimum": 0 },
    "state_hash":   { "type": "string", "pattern": "^[0-9a-f]{64}$" }
  },
  "additionalProperties": false
}
```

### 13.3 save.list

List all available saves for the current session (quick saves, slot saves, and autosaves).

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-003",
  "method": "save.list",
  "params": {
    "include_quick": true,
    "include_auto": true,
    "limit": 50,
    "offset": 0
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-003",
  "result": {
    "saves": [
      {
        "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
        "save_type": "slot",
        "slot_name": "before-war-with-rome",
        "tick": 12500,
        "size_bytes": 2097152,
        "created_at": "2026-02-21T14:30:00Z",
        "format_version": 1
      },
      {
        "save_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
        "save_type": "auto",
        "slot_name": "autosave_0000012400",
        "tick": 12400,
        "size_bytes": 2031616,
        "created_at": "2026-02-21T14:28:20Z",
        "format_version": 1
      },
      {
        "save_id": "c81d2a3b-1122-4bcd-9ef0-aabbccdd1122",
        "save_type": "quick",
        "slot_name": null,
        "tick": 12490,
        "size_bytes": 11000000,
        "created_at": "2026-02-21T14:29:50Z",
        "format_version": 1
      }
    ],
    "total_count": 3,
    "has_more": false
  }
}
```

**JSON Schema — Request params:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveListParams",
  "type": "object",
  "properties": {
    "include_quick":  { "type": "boolean", "default": true },
    "include_auto":   { "type": "boolean", "default": true },
    "limit":          { "type": "integer", "minimum": 1, "maximum": 200, "default": 50 },
    "offset":         { "type": "integer", "minimum": 0, "default": 0 }
  },
  "additionalProperties": false
}
```

**JSON Schema — SaveEntry (item in `saves` array):**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveEntry",
  "type": "object",
  "required": ["save_id", "save_type", "tick", "size_bytes", "created_at", "format_version"],
  "properties": {
    "save_id":        { "type": "string", "format": "uuid" },
    "save_type":      { "type": "string", "enum": ["quick", "slot", "auto"] },
    "slot_name":      { "type": ["string", "null"] },
    "tick":           { "type": "integer", "minimum": 0 },
    "size_bytes":     { "type": "integer", "minimum": 0 },
    "created_at":     { "type": "string", "format": "date-time" },
    "format_version": { "type": "integer", "minimum": 1 }
  },
  "additionalProperties": false
}
```

### 13.4 save.load

Load a save by ID and resume the session from that save point. The current tick loop is paused, state is replaced, and the tick loop resumes from tick N+1.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "method": "save.load",
  "params": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555"
  }
}
```

**Response (success):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "result": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
    "resumed_at_tick": 12500,
    "duration_ms": 820,
    "skipped_mods": [],
    "migration_applied": false
  }
}
```

**Response (with skipped mods — non-fatal):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "result": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
    "resumed_at_tick": 12500,
    "duration_ms": 820,
    "skipped_mods": ["com.example.old-mod"],
    "migration_applied": true
  }
}
```

**Response (error — hash mismatch):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "error": {
    "code": -32002,
    "message": "Save integrity check failed: BLAKE3 hash mismatch",
    "data": {
      "expected_hash": "a3f9b210...",
      "computed_hash": "deadbeef..."
    }
  }
}
```

**JSON Schema — Request params:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveLoadParams",
  "type": "object",
  "required": ["save_id"],
  "properties": {
    "save_id": { "type": "string", "format": "uuid" }
  },
  "additionalProperties": false
}
```

**JSON Schema — Result:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveLoadResult",
  "type": "object",
  "required": ["save_id", "resumed_at_tick", "duration_ms", "skipped_mods", "migration_applied"],
  "properties": {
    "save_id":            { "type": "string", "format": "uuid" },
    "resumed_at_tick":    { "type": "integer", "minimum": 0 },
    "duration_ms":        { "type": "integer", "minimum": 0 },
    "skipped_mods":       { "type": "array", "items": { "type": "string" } },
    "migration_applied":  { "type": "boolean" }
  },
  "additionalProperties": false
}
```

### 13.5 save.delete

Delete a named slot save or autosave. QuickSaves cannot be individually deleted via RPC; they are managed by the ring buffer.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-005",
  "method": "save.delete",
  "params": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555"
  }
}
```

**Response (success):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-005",
  "result": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
    "deleted": true
  }
}
```

**JSON Schema — Request params:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveDeleteParams",
  "type": "object",
  "required": ["save_id"],
  "properties": {
    "save_id": { "type": "string", "format": "uuid" }
  },
  "additionalProperties": false
}
```

### 13.6 save.verify

Verify a save's integrity without loading it. Recomputes BLAKE3 and checks header validity.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-006",
  "method": "save.verify",
  "params": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-006",
  "result": {
    "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
    "valid": true,
    "hash_match": true,
    "format_version": 1,
    "tick": 12500,
    "created_at": "2026-02-21T14:30:00Z",
    "state_hash": "a3f9b210e44cc4891b5671e02b2c3d555a3f9b210e44cc4891b5671e02b2c3d5",
    "duration_ms": 95
  }
}
```

**JSON Schema — Result:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SaveVerifyResult",
  "type": "object",
  "required": ["save_id", "valid", "hash_match", "format_version", "tick", "created_at", "state_hash", "duration_ms"],
  "properties": {
    "save_id":        { "type": "string", "format": "uuid" },
    "valid":          { "type": "boolean" },
    "hash_match":     { "type": "boolean" },
    "format_version": { "type": "integer", "minimum": 1 },
    "tick":           { "type": "integer", "minimum": 0 },
    "created_at":     { "type": "string", "format": "date-time" },
    "state_hash":     { "type": "string", "pattern": "^[0-9a-f]{64}$" },
    "duration_ms":    { "type": "integer", "minimum": 0 }
  },
  "additionalProperties": false
}
```

### 13.7 Error Codes

| Code | Constant | Meaning |
|------|----------|---------|
| -32001 | `SAVE_MEMORY_LIMIT` | QuickSave memory ceiling exceeded |
| -32002 | `SAVE_HASH_MISMATCH` | BLAKE3 verification failed |
| -32003 | `SAVE_NOT_FOUND` | No save with given ID |
| -32004 | `SAVE_FORMAT_TOO_OLD` | Save version predates N-2 support window |
| -32005 | `SAVE_FORMAT_TOO_NEW` | Save version is from a future engine |
| -32006 | `SAVE_CORRUPT` | Deserialization failed (not a hash error) |
| -32007 | `SAVE_MIGRATION_FAILED` | Migration chain could not complete |
| -32008 | `SAVE_DB_ERROR` | Database write/read failure |
| -32009 | `SAVE_IO_ERROR` | File store I/O failure |
| -32010 | `SAVE_SESSION_INACTIVE` | Cannot save: no active simulation session |

---

## 14. Events

The save/load system emits events on the standard event bus (CIV-0001 event protocol). All events are broadcast to connected clients via the WebSocket event stream.

### 14.1 session.saved.v1

Emitted after every successful save (QuickSave, SlotSave, or AutoSave).

**JSON Schema:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SessionSavedV1",
  "description": "Emitted after a successful save operation of any type.",
  "type": "object",
  "required": ["event", "version", "session_id", "save_id", "save_type", "tick", "duration_ms"],
  "properties": {
    "event":        { "const": "session.saved.v1" },
    "version":      { "const": 1 },
    "session_id":   { "type": "string", "format": "uuid" },
    "save_id":      { "type": "string", "format": "uuid" },
    "save_type":    { "type": "string", "enum": ["quick", "slot", "auto"] },
    "slot_name":    { "type": ["string", "null"],
                      "description": "Null for quick saves." },
    "tick":         { "type": "integer", "minimum": 0 },
    "size_bytes":   { "type": "integer", "minimum": 0,
                      "description": "Compressed bytes for slot/auto; uncompressed for quick." },
    "duration_ms":  { "type": "integer", "minimum": 0 },
    "state_hash":   { "type": "string", "pattern": "^[0-9a-f]{64}$" }
  },
  "additionalProperties": false
}
```

**Example payload:**
```json
{
  "event": "session.saved.v1",
  "version": 1,
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
  "save_type": "slot",
  "slot_name": "before-war-with-rome",
  "tick": 12500,
  "size_bytes": 2097152,
  "duration_ms": 310,
  "state_hash": "a3f9b210e44cc4891b5671e02b2c3d555a3f9b210e44cc4891b5671e02b2c3d5"
}
```

### 14.2 session.loaded.v1

Emitted after a save is successfully loaded and the tick loop has resumed.

**JSON Schema:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SessionLoadedV1",
  "description": "Emitted after a save is loaded and simulation resumes.",
  "type": "object",
  "required": ["event", "version", "session_id", "save_id", "resumed_at_tick", "duration_ms"],
  "properties": {
    "event":              { "const": "session.loaded.v1" },
    "version":            { "const": 1 },
    "session_id":         { "type": "string", "format": "uuid" },
    "save_id":            { "type": "string", "format": "uuid" },
    "resumed_at_tick":    { "type": "integer", "minimum": 0 },
    "duration_ms":        { "type": "integer", "minimum": 0 },
    "migration_applied":  { "type": "boolean" },
    "skipped_mods":       {
      "type": "array",
      "items": { "type": "string" },
      "description": "Mod IDs present in save but not loaded in current engine instance."
    }
  },
  "additionalProperties": false
}
```

**Example payload:**
```json
{
  "event": "session.loaded.v1",
  "version": 1,
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "save_id": "a3f9b210-44cc-4891-b567-1e02b2c3d555",
  "resumed_at_tick": 12500,
  "duration_ms": 820,
  "migration_applied": false,
  "skipped_mods": []
}
```

### 14.3 session.save_failed.v1

Emitted when a save operation fails for any reason (hash error, I/O error, mod serialization error). The simulation continues running; the failed save does not affect simulation state.

**JSON Schema:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SessionSaveFailedV1",
  "description": "Emitted when a save operation fails. Simulation continues.",
  "type": "object",
  "required": ["event", "version", "session_id", "save_type", "tick", "reason", "error_code"],
  "properties": {
    "event":        { "const": "session.save_failed.v1" },
    "version":      { "const": 1 },
    "session_id":   { "type": "string", "format": "uuid" },
    "save_type":    { "type": "string", "enum": ["quick", "slot", "auto"] },
    "slot_name":    { "type": ["string", "null"] },
    "tick":         { "type": "integer", "minimum": 0 },
    "reason":       { "type": "string",
                      "description": "Human-readable error description." },
    "error_code":   { "type": "string",
                      "description": "Machine-readable error code (e.g. SAVE_IO_ERROR)." }
  },
  "additionalProperties": false
}
```

**Example payload:**
```json
{
  "event": "session.save_failed.v1",
  "version": 1,
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "save_type": "auto",
  "slot_name": null,
  "tick": 12600,
  "reason": "File store write failed: disk full",
  "error_code": "SAVE_IO_ERROR"
}
```

### 14.4 session.load_failed.v1

Emitted when a load operation fails. The current session state is unchanged (load is atomic: either it fully succeeds and replaces state, or it fails and leaves state untouched).

**JSON Schema:**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SessionLoadFailedV1",
  "description": "Emitted when a load operation fails. Session state unchanged.",
  "type": "object",
  "required": ["event", "version", "session_id", "save_id", "reason", "error_code"],
  "properties": {
    "event":        { "const": "session.load_failed.v1" },
    "version":      { "const": 1 },
    "session_id":   { "type": "string", "format": "uuid" },
    "save_id":      { "type": "string", "format": "uuid" },
    "reason":       { "type": "string" },
    "error_code":   { "type": "string" }
  },
  "additionalProperties": false
}
```

---

## 15. FR Traceability

### 15.1 Functional Requirements

All FRs use the `FR-SAVE-NNN` namespace. Each FR maps to one or more acceptance criteria defined in Section 17.

| FR ID | SHALL Statement | Priority | Related Sections |
|-------|----------------|----------|-----------------|
| FR-SAVE-001 | The engine SHALL support QuickSave to an in-memory ring buffer of at most 5 slots without performing any I/O or DB writes. | MUST | §6.2, §10 |
| FR-SAVE-002 | The engine SHALL support SlotSave to a user-named persistent slot in the configured storage backend (SQLite or PostgreSQL). | MUST | §6.3, §9.2 |
| FR-SAVE-003 | The engine SHALL support AutoSave triggered automatically every N ticks (configurable; default 100) with a configurable ring of at most 50 autosave slots. | MUST | §6.4, §11 |
| FR-SAVE-004 | Every save SHALL include a complete snapshot of all simulation state such that no state required for tick computation is absent from the save file (D8 invariant). | MUST | §1.3, §3 |
| FR-SAVE-005 | After loading a save at tick N, tick N+1 SHALL be bit-for-bit identical to what it would have been in an uninterrupted run (D9 Resume Fidelity invariant). | MUST | §1.3, §7.2 |
| FR-SAVE-006 | The save file format SHALL include a BLAKE3 integrity hash covering all serialized state, computed at save time and verified at load time before any deserialization begins. | MUST | §2.4, §6.6 |
| FR-SAVE-007 | Loading a save with a BLAKE3 hash mismatch SHALL fail immediately with a hard error; the current session state SHALL remain unchanged. | MUST | §7.1 Step 6 |
| FR-SAVE-008 | The ChaCha20Rng state SHALL be serialized to 20 u32 words plus a sub-block word count and SHALL be exactly restored on load, guaranteeing RNG stream continuity. | MUST | §3.2, §7.3 |
| FR-SAVE-009 | The BLAKE3 hash chain tail SHALL be serialized and restored on load, enabling the chain to continue unbroken from the saved tick. | MUST | §3.3, §7.1 |
| FR-SAVE-010 | All AI state — personality parameters, strategic goals, memory buffers, MCTS result cache, and threat models — SHALL be serialized per-nation and fully restored on load. | MUST | §4 |
| FR-SAVE-011 | WASM mods that implement the `ModStateSave` trait SHALL have their state included in every save. On load, known mods SHALL have their state restored before tick N+1 executes. | MUST | §5 |
| FR-SAVE-012 | Unknown mod IDs present in a save but not loaded in the current engine instance SHALL produce a warning log entry and be skipped without failing the load operation. | MUST | §5.4 |
| FR-SAVE-013 | The save format version SHALL be stored in the binary header. The engine SHALL apply migration functions to bring saves from versions N-2 through N-1 to the current format version N on load. | MUST | §2.3, §8 |
| FR-SAVE-014 | Saves with a format version older than N-2 (the minimum supported version) SHALL be rejected with a hard error identifying the save version and the minimum supported version. | MUST | §8.3 |
| FR-SAVE-015 | Saves with a format version newer than the current engine version SHALL be rejected with a hard error. | MUST | §8.3 |
| FR-SAVE-016 | QuickSave latency SHALL be at most 50ms for a scenario with 1,000 citizens, measured as wall-clock time from save invocation to ring push completion. | MUST | §12.1 |
| FR-SAVE-017 | SlotSave latency SHALL be at most 500ms for a scenario with 1,000 citizens, including serialization, zstd compression, and DB metadata write. | MUST | §12.1 |
| FR-SAVE-018 | Load latency SHALL be at most 1,000ms for a scenario with 1,000 citizens, including decompression, deserialization, migration (if needed), and World restoration. | MUST | §12.1 |
| FR-SAVE-019 | Save integrity verification (hash check without full deserialization) SHALL complete in at most 200ms for any save size. | MUST | §6.6, §12.1 |
| FR-SAVE-020 | The autosave ring SHALL retain at most `max_slots` autosaves per session. Insertion of a new autosave that would exceed the limit SHALL evict the oldest autosave and delete its backing file. | MUST | §9.4, §11.3 |
| FR-SAVE-021 | All save and load operations SHALL be exposed via JSON-RPC methods (`save.quick`, `save.slot`, `save.list`, `save.load`, `save.delete`, `save.verify`) conforming to the schemas defined in Section 13. | MUST | §13 |
| FR-SAVE-022 | All save operation completions and failures SHALL emit events on the event bus (`session.saved.v1`, `session.loaded.v1`, `session.save_failed.v1`, `session.load_failed.v1`) conforming to the schemas in Section 14. | MUST | §14 |
| FR-SAVE-023 | Load operations SHALL be atomic: if any step in the load sequence fails, the current session state SHALL remain unchanged and a `session.load_failed.v1` event SHALL be emitted. | MUST | §7.1 |
| FR-SAVE-024 | Pending commands buffered at save time SHALL be serialized in `SimStateSnapshot::pending_commands` and re-injected into the command buffer before tick N+1 executes on resume. | MUST | §3.12, §7.1 Step 15 |
| FR-SAVE-025 | The `save.list` RPC method SHALL return metadata for all available saves (quick, slot, auto) ordered by tick descending, supporting pagination via `limit` and `offset`. | SHOULD | §13.3 |

---

## 16. Integration Points

### 16.1 CIV-0001 — Core Simulation Loop

The save system hooks into the tick loop at two points:

**AutoSave hook (end of tick):**
```
Tick N complete
├─ ... (normal tick phases)
└─ PostTickHook: if tick % autosave_interval == 0
   └─ capture_snapshot_sync(&world)     ← synchronous, fast
       └─ tokio::spawn(write_autosave)  ← async, non-blocking
```

The snapshot capture (extracting all component data from the bevy_ecs World into plain Rust structs) runs synchronously to avoid TOCTOU races with the next tick. The actual file I/O and compression run asynchronously so they do not block the tick loop.

**QuickSave hook (command handler):**
The `save.quick` JSON-RPC command is processed in the Command Intake phase of the next tick after it arrives. This ensures the save captures a consistent post-phase-N state.

**State ownership boundary:**
The core simulation owns the `SimState` (wrapping the bevy_ecs World, the ChaCha20Rng, and the tick counter). The persistence layer receives an immutable reference to `SimState` during serialization. The persistence layer has no write access to `SimState`; state restoration on load goes through the session manager, which reconstructs `SimState` from the loaded `SimStateSnapshot` and replaces the old World.

### 16.2 CIV-0900 — Session Management

Session management (CIV-0900) is the caller of the persistence API. The delegation chain is:

```
Client (WebSocket JSON-RPC)
    ↓  save.slot {slot_name}
Session Manager (CIV-0900)
    ↓  session.save_slot(slot_name)
Persistence Layer (CIV-1000)
    ↓  save_slot(state, ai_states, mod_registry, slot_name, session_id, db, file_store)
    ↓  → SaveMetadata
Session Manager
    ↓  updates sessions.last_save_id, sessions.last_save_at
    ↓  emits session.saved.v1 event
Client receives event
```

The session manager is responsible for:
- Routing incoming `save.*` RPC requests to the persistence layer
- Updating session metadata in the DB after a successful save
- Emitting save/load events on the event bus
- Exposing save listing via the session's WebSocket connection

The persistence layer is responsible for:
- All serialization, compression, hashing, and file I/O
- DB writes to `save_slots` and `autosaves` tables
- Migration on load
- Mod state coordination

### 16.3 CIV-0700 — Modding API

The persistence layer coordinates with the mod system through the `ModSaveRegistry`. The integration contract is:

**At mod load time** (CIV-0700 mod lifecycle):
```rust
// In the mod loader (civ-engine/src/mods/loader.rs)
let handler = create_mod_save_handler(&mod_manifest, wasm_instance);
mod_save_registry.register(handler);
```

**At save time:**
```rust
let mod_blobs = mod_save_registry.serialize_all()?;
// mod_blobs included in save archive as mod_state.bin
```

**At load time:**
```rust
let skipped = mod_save_registry.deserialize_all(mod_blobs)?;
// skipped contains mod IDs that were in the save but are not loaded
```

**WASM boundary:**
Mod state serialization crosses the WASM boundary. The host calls the mod's exported `__civlab_save` function, which serializes state inside the WASM sandbox and returns a byte pointer. The host copies the bytes out and includes them in the save. On load, the host calls `__civlab_load` with the saved bytes, and the mod restores its internal state. This preserves sandbox isolation — the host never directly reads mod internal memory.

### 16.4 CIV-0400 — AI / NPC Behavior

The AI state serialization (Section 4) is coordinated with the AI system:

```rust
// In the AI coordinator (civ-engine/src/ai/coordinator.rs)
pub fn extract_ai_snapshot(ai_states: &[AiNationState]) -> AiStateSnapshot {
    AiStateSnapshot {
        ai_state_version: CURRENT_AI_STATE_VERSION,
        nations: ai_states.iter().map(|s| s.to_snapshot()).collect(),
    }
}

pub fn restore_from_snapshot(snapshot: AiStateSnapshot) -> Vec<AiNationState> {
    snapshot.nations.into_iter().map(AiNationState::from_snapshot).collect()
}
```

The MCTS result cache (`MctsResultCache`) is captured from the AI coordinator's last-evaluated result. On resume, the AI uses the cached result as a warm start rather than running from scratch, maintaining behavioral continuity across save/load.

Personality drift parameters are included in `PersonalityParams` so that personality evolution (which accrues over many ticks) is preserved exactly.

---

## 17. Acceptance Criteria

Acceptance criteria are grouped by functional requirement. All criteria must pass for the CIV-1000 implementation to be considered complete.

### 17.1 Save Completeness (FR-SAVE-004, FR-SAVE-005)

**AC-1000-01**: Given a simulation running for 500 ticks from seed S, when a SlotSave is taken at tick 250, and the simulation is reset and resumed from that save, then for every tick from 251 to 500 the BLAKE3 hash of the tick state SHALL be identical to the hash produced in the original uninterrupted run.

**AC-1000-02**: The save-resume parity harness (`civ-engine/tests/determinism/save_resume_parity.rs`) SHALL pass for all three scenario scales (tiny/medium/large) in CI on every PR.

**AC-1000-03**: Manually removing any single field from `SimStateSnapshot` and re-running the parity test SHALL cause the test to fail, demonstrating completeness of the schema.

### 17.2 BLAKE3 Integrity (FR-SAVE-006, FR-SAVE-007)

**AC-1000-04**: When a save file's `state.bin` is modified by flipping any single byte, `verify_save` SHALL return `hash_match: false` and `load_save` SHALL return `LoadError::HashMismatch`.

**AC-1000-05**: When `load_save` encounters a hash mismatch, the active session's `SimState` SHALL be unchanged (verified by asserting that tick N and the RNG state are identical before and after the failed load attempt).

### 17.3 RNG Continuity (FR-SAVE-008)

**AC-1000-06**: Given a ChaCha20Rng at an arbitrary stream position P (after consuming N values), when `RngSnapshot` is captured and restored, the restored RNG SHALL produce the identical sequence of values as the original RNG from position P onward.

**AC-1000-07**: The RNG restoration test SHALL exercise positions at: block boundary 0, mid-block (word 32 of 64), and after 10^6 consumed values.

### 17.4 AI State Continuity (FR-SAVE-010)

**AC-1000-08**: After loading a save, an AI nation's personality parameters SHALL be identical to those at the save point (verified by asserting `PersonalityParams` field-by-field equality).

**AC-1000-09**: After loading a save, the AI's strategic goal queue SHALL be in the same order with the same priorities as at the save point.

**AC-1000-10**: After loading a save, the AI's memory (betrayal records, battle outcomes) SHALL contain the identical entries as at the save point.

### 17.5 Mod State (FR-SAVE-011, FR-SAVE-012)

**AC-1000-11**: A test mod implementing `ModStateSave` with a counter state incremented each tick SHALL, after save and load, have the counter restored to the exact value at save time.

**AC-1000-12**: When a save contains a mod blob for `com.example.unknown-mod` and that mod is not loaded, `load_save` SHALL succeed, `LoadedSave::skipped_mods` SHALL contain `"com.example.unknown-mod"`, and a `tracing::warn!` log entry SHALL be emitted.

### 17.6 Migration (FR-SAVE-013, FR-SAVE-014, FR-SAVE-015)

**AC-1000-13**: A save created at format version N-1 SHALL be loadable by an engine at format version N after applying the v(N-1)→vN migration function, producing a `SimStateSnapshot` that passes all parity checks.

**AC-1000-14**: A save with format version older than N-2 SHALL cause `load_save` to return `LoadError::TooOld` without deserializing any state.

**AC-1000-15**: A save with format version greater than the current engine version SHALL cause `load_save` to return `LoadError::FutureVersion` without deserializing any state.

### 17.7 Performance (FR-SAVE-016, FR-SAVE-017, FR-SAVE-018, FR-SAVE-019)

**AC-1000-16**: The `criterion` benchmark `quicksave_1000_citizens` SHALL measure a mean latency of ≤ 50ms on the CI hardware tier (4-core x86-64, 8GB RAM). CI SHALL fail the PR if the mean exceeds 60ms (20% tolerance).

**AC-1000-17**: The `criterion` benchmark `slotsave_1000_citizens` SHALL measure a mean latency of ≤ 500ms. CI SHALL fail the PR if the mean exceeds 600ms.

**AC-1000-18**: The `criterion` benchmark `load_1000_citizens` SHALL measure a mean latency of ≤ 1,000ms. CI SHALL fail the PR if the mean exceeds 1,200ms.

**AC-1000-19**: `verify_save` SHALL complete in ≤ 200ms for a 10,000-citizen save (largest supported scale).

### 17.8 JSON-RPC Contract (FR-SAVE-021)

**AC-1000-20**: Each of the six `save.*` RPC methods SHALL be exercised by integration tests that validate both successful and error response shapes against the JSON Schemas defined in Section 13.

**AC-1000-21**: `save.list` SHALL return results sorted by tick descending. When called with `limit: 2, offset: 1`, it SHALL return the second and third most recent saves.

### 17.9 Events (FR-SAVE-022)

**AC-1000-22**: After a successful `save.slot` operation, a `session.saved.v1` event SHALL be broadcast to all connected clients within 100ms of the save completing.

**AC-1000-23**: The event payload for `session.saved.v1` SHALL validate against the JSON Schema in Section 14.1. Automated contract tests SHALL assert this on every emitted event.

**AC-1000-24**: When an autosave I/O write fails, a `session.save_failed.v1` event SHALL be emitted with `error_code: "SAVE_IO_ERROR"` and the simulation SHALL continue running (tick loop not interrupted).

### 17.10 Load Atomicity (FR-SAVE-023)

**AC-1000-25**: A load operation that fails at step 8 (deserialization failure) SHALL leave the active session's tick counter, RNG state, and World unchanged, verified by asserting equality of all three before and after the failed load.

**AC-1000-26**: Fault injection tests SHALL simulate failure at each of steps 5-11 in the load sequence (Section 7.1) and verify that in each case the session state is unchanged and `session.load_failed.v1` is emitted.

---

*End of CIV-1000 Save, Load, and Persistence System Specification*

*Spec ID: CIV-1000 | Version: 1.0 | Status: SPECIFICATION | Date: 2026-02-21*
