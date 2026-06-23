# CIV-0700: Modding and Plugin API — Full Specification

**Spec ID:** CIV-0700
**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team
**Related Specs:**
- CIV-0001: Core Simulation Loop (deterministic tick architecture, phase scheduler)
- CIV-0100: Economy Module (AllocationEngine trait, conservation invariants)
- CIV-0103: Institutions & Governance (policy authority, policy application phase)
- CIV-0104: Minimal Constraint Set Theorem (invariant enforcement)

---

## Executive Summary

CivLab's modding and plugin API is a **first-class extension surface** enabling three distinct user groups — researchers adding novel economic models, players constructing new unit and building archetypes, and AI engineers embedding custom policy algorithms — to extend simulation behavior without modifying engine source code.

The API is architected around four principles:

1. **Isolation by default.** All mod code runs inside a WebAssembly (WASM) sandbox with a hard memory ceiling, a hard per-tick CPU budget, and zero access to the host filesystem or network stack. A misbehaving mod cannot corrupt engine state or slow the simulation below its tick budget.

2. **Determinism as a first-class invariant.** The core simulation guarantee (identical seed + identical input events → identical state at every tick) extends fully into the mod layer. Non-deterministic instructions are rejected at load time. Host-provided randomness flows through a seeded, deterministic interface. Mods cannot call `std::time` or any platform clock.

3. **Typed, versioned contracts.** Mod types declare an API compatibility version. The host rejects mods at load time if their declared API version does not satisfy the host's accepted range. API evolution follows semantic versioning; breaking changes increment the major API version.

4. **Research-grade introspection.** Mod actions are recorded in the replay log alongside native engine actions. Mod state is observable via the existing metrics and telemetry subsystems. A mod can be swapped in or out mid-simulation (at a tick boundary) for controlled experiments.

This specification defines the WASM sandbox architecture, each mod type's trait contract, the `civlab-sdk` guest-side library, mod loading lifecycle, the `.civmod` distribution format, the optional Lua scripting path, functional requirements, and testing mandates.

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [Mod Types and Extension Points](#2-mod-types-and-extension-points)
3. [WASM Sandbox Architecture](#3-wasm-sandbox-architecture)
4. [Mod Manifest Format](#4-mod-manifest-format)
5. [PolicyMod API](#5-policymod-api)
6. [EconomicMod API](#6-economicmod-api)
7. [EventMod API](#7-eventmod-api)
8. [ScenarioMod API](#8-scenariomod-api)
9. [civlab-sdk — Guest-Side Rust Library](#9-civlab-sdk--guest-side-rust-library)
10. [Mod Loading and Lifecycle](#10-mod-loading-and-lifecycle)
11. [Mod Distribution Format](#11-mod-distribution-format)
12. [Scenario Scripting Alternative — Lua](#12-scenario-scripting-alternative--lua)
13. [Research Mod Examples](#13-research-mod-examples)
14. [Security and Isolation](#14-security-and-isolation)
15. [Functional Requirements](#15-functional-requirements)
16. [Testing and CI](#16-testing-and-ci)

---

## 1. Design Philosophy

### 1.1 Who the Mod System Serves

The modding API is not a cosmetic extension layer. It is a **scientific instrument** for simulation research and a **creativity layer** for scenario construction. Three user groups are explicitly in scope:

**Researchers** — economists, political scientists, climate scientists, and AI researchers using CivLab as an experimental platform. They add new economic models (e.g., carbon pricing mechanisms, alternative allocation algorithms, experimental tax structures), new climate event types, and custom victory conditions measuring research-specific outcomes. Their primary need is access to typed, auditable state slices and the ability to inject deterministic policy actions into the tick pipeline.

**Players** — scenario builders and game designers who extend the simulation with new building archetypes, unit types, technology paths, and map configurations. They need a composable vocabulary of named entity types, construction requirements, and production chains without touching engine internals.

**AI Researchers** — engineers embedding custom reinforcement learning policies, rule-based agents, or hybrid decision algorithms. They need a zero-latency callback per tick, access to the full observable state available to native AI agents, and a well-typed action space identical to the native AI action space. WASM overhead per tick must be bounded and predictable.

### 1.2 Non-Goals

The following are explicitly outside scope for the v1 mod API:

- **Rendering extensions.** Mods cannot inject geometry, shaders, or audio assets into the Bevy/Unreal client layer. That surface is covered by the client protocol spec (CIV-0200).
- **Network access from mods.** Mods are pure computation. External data sources must be pre-baked into the mod bundle at build time.
- **Multithreaded mods.** Each mod instance executes on a single thread managed by the host. Thread spawn is not a permitted syscall. Parallelism is achieved by the host loading multiple mod instances.
- **Shared mutable state between mods.** Mods observe world state through read-only host-provided views. Mods do not communicate directly. Inter-mod coordination happens via the host's action queue.

### 1.3 Security Model

The threat model assumes mod code is **untrusted**. Community mods may contain bugs, intentional exploits, or adversarial behavior. The sandbox must enforce:

- **Memory isolation:** A mod cannot read or write host memory outside its own WASM linear memory region.
- **No syscalls:** The WASM host does not export any syscall-adjacent functions (open, read, write, socket, exec, etc.). The only imported functions are civlab host functions defined in this spec.
- **CPU budget enforcement:** A mod that enters an infinite loop or expensive computation is terminated by epoch interruption after 50 µs. The tick scheduler receives a `ModTimeoutError`; the mod's actions for that tick are discarded.
- **Determinism enforcement:** Non-deterministic WASM instructions (platform-specific float operations, any instruction with undefined behavior across platforms) are detected at module load time and cause load rejection.
- **Signature verification:** Mod WASM binaries are signed with the author's Ed25519 key. The host verifies the signature before instantiation. Unsigned mods can only be loaded in development mode (compile flag `--features mod-dev`).

### 1.4 Determinism Contract

CivLab's core invariant (CIV-0001 §Determinism Invariants) states:

> Given identical seed and identical input event sequence, the simulation produces identical state at every tick, on any host platform.

This invariant extends into the mod layer without qualification. Every mod is subject to the same constraints:

- **No wall-clock time.** `std::time::SystemTime`, `std::time::Instant`, and any WASM `clock_time_get` equivalent are not exported by the host. A mod that imports these will fail validation at load time.
- **No platform RNG.** `rand::thread_rng()`, `OsRng`, and equivalent are not available. The host provides a seeded `ChaCha20Rng` instance per tick, with the seed derived deterministically from the tick number and simulation seed.
- **Deterministic iteration.** Mods receiving collection views receive them in deterministic order (BTreeMap key order, matching the engine's own collection discipline from CIV-0001 §Determinism Invariants).
- **Fixed-point arithmetic.** All monetary, energy, and resource quantities are `i64` fixed-point values. Mods must not use `f64` arithmetic on quantities that affect world state. The `civlab-sdk` re-exports the `fixed` crate for safe fixed-point operations.

### 1.5 Versioning Strategy

The mod API uses a two-level version scheme:

- **API version** — an integer (e.g., `1`). Incremented on any breaking change to the host ABI, trait definitions, or permitted action types. A mod declares `api_version = "1"` in its manifest; the host accepts mods declaring any version within its supported range.
- **SDK version** — a semantic version for the `civlab-sdk` crate (e.g., `1.0.0`). Follows semver; patch and minor increments are backward-compatible.

The host maintains a compatibility window of `[current_api_version - 1, current_api_version]`. Mods compiled against API version 0 can run on a v1 host during the transition window. API version 0 support is dropped when the host increments to API version 2.

---

## 2. Mod Types and Extension Points

CivLab defines four distinct mod types. Each targets a specific phase in the tick pipeline and has a distinct trait contract.

### 2.1 Mod Type Summary

| Mod Type | Phase Hook | Extension Point | Primary Use Case |
|---|---|---|---|
| `PolicyMod` | Phase 3a (Policy Application) | Tax rates, transfers, policy params | Custom economic policies, AI agents |
| `EconomicMod` | Phase 3b/3c (Production, Market Clear) | Production formulas, market allocators, custom goods | New economic models, research experiments |
| `EventMod` | Phase 4 (Stochastic Events) | Custom event triggers and handlers | New disaster types, geopolitical events |
| `ScenarioMod` | Simulation init + Phase 5 (Victory Check) | Starting conditions, victory conditions, map generation | Custom scenarios, tournaments, research setups |

### 2.2 PolicyMod

A `PolicyMod` receives a typed, read-only view of the current world state and returns a list of `PolicyAction` values to be applied by the host. It is invoked once per tick during Phase 3a, before production and market clearing. It is also invoked on receipt of `SimEvent` values from Phase 4.

**Intended use cases:**
- Implement a carbon tax with household rebate mechanism
- Implement a Pigouvian subsidy for renewable energy producers
- Implement an AI-driven adaptive tax rate controller
- Implement a sanctions regime triggered by diplomatic events

### 2.3 EconomicMod

An `EconomicMod` overrides or supplements the host's production, market clearing, and consumption calculations. It is invoked during Phase 3b (Production) and Phase 3c (Market Clear). Unlike `PolicyMod`, an `EconomicMod` replaces default behavior rather than layering on top of it — a scenario must explicitly assign an `EconomicMod` as the active allocator for a given sector or nation.

**Intended use cases:**
- Replace the default price-auction market clearing with a central planning allocator
- Introduce a new energy production technology (e.g., fusion reactor) with a custom output formula
- Add a new custom good type (e.g., carbon credits) with custom production and consumption rules
- Model a dual-economy scenario (market sector + planned sector coexisting)

### 2.4 EventMod

An `EventMod` registers one or more event handlers that fire based on triggers: tick intervals, threshold crossings, or pattern matches on incoming event streams. It can generate new `SimEvent` values that enter the standard event pipeline, and it can return `SimAction` values that modify world state.

**Intended use cases:**
- New climate disaster type (e.g., megadrought) with custom trigger conditions and economic impact
- Custom diplomatic event (e.g., trade pact formation based on research-defined conditions)
- Population migration trigger based on climate + economic thresholds

### 2.5 ScenarioMod

A `ScenarioMod` defines a complete simulation starting configuration and optionally overrides victory condition evaluation. It is executed once at simulation initialization to construct the initial `WorldState` and is called each tick during Phase 5 (Victory Check) if it registers a custom victory condition.

**Intended use cases:**
- Cold War scenario: two superpowers with specific military and economic starting conditions
- Climate crisis scenario: high-baseline CO2, stressed agricultural systems, refugee pressure
- Research tournament: standardized starting conditions for comparing AI agent performance
- Custom map: archipelago geography with maritime trade routes

---

## 3. WASM Sandbox Architecture

### 3.1 Runtime

The mod host uses **wasmtime 26.x** (Rust crate `wasmtime`). wasmtime provides:

- Component model support (used for ABI type-checking at load time)
- Epoch-based interruption (used for CPU budget enforcement)
- `StoreLimitsBuilder` (used for memory budget enforcement)
- Fuel-based metering (optional, for research instrumentation of mod computational cost)

wasmtime is the sole supported WASM runtime. Alternative runtimes (wasmer, wasm3, WAVM) are not supported. This is a deliberate constraint: multi-runtime support would require duplicating sandbox validation logic and accepting divergent behavior in edge cases.

### 3.2 Memory Limits

Each mod instance is allocated a private WASM linear memory with the following limits:

| Parameter | Value | Enforcement Mechanism |
|---|---|---|
| Initial memory | 1 MB | wasmtime default page allocation |
| Maximum memory | 64 MB | `StoreLimitsBuilder::memory_size(67_108_864)` |
| Table size | 8 192 entries | `StoreLimitsBuilder::table_elements(8192)` |
| Instance count | 1 per mod | One `Instance` per mod `id`; no sub-instantiation |

A mod that attempts to grow its memory beyond 64 MB receives a WASM `memory.grow` failure (returns `-1`). The mod must handle this case; the host does not terminate the mod for a failed `memory.grow`. However, if the mod subsequently traps due to out-of-memory, the host catches the trap, logs `ModTrap { mod_id, tick, trap_type: OOM }`, and discards the mod's actions for that tick.

### 3.3 CPU Budget

CPU budget is enforced via **wasmtime epoch interruption**:

- The host runtime increments a shared epoch counter at a fixed wall-clock rate (1 epoch per 10 µs).
- Each mod `Store` is configured with `epoch_deadline = 5` (5 epochs × 10 µs = 50 µs).
- When a mod's callback is entered, the host calls `store.set_epoch_deadline(5)`.
- If the mod does not return within 5 epochs, wasmtime raises an `InterruptTrap`.
- The host catches the `InterruptTrap`, logs `ModTimeout { mod_id, tick, callback }`, and discards that tick's actions.

The 50 µs budget is calibrated against the tick budget defined in CIV-0001 §Performance Targets. The tick scheduler allocates Phase 3a a 2 ms budget across all policy mods. With the default mod limit of 16 concurrently loaded `PolicyMod` instances, each instance receives approximately 125 µs of the phase budget. The 50 µs per-callback limit is intentionally conservative to leave headroom for host-side action validation and dispatch.

```
Tick budget (Phase 3a): 2 000 µs total
Max policy mods loaded: 16
Per-mod allocation:      125 µs (2000 / 16)
Per-callback hard limit: 50 µs (enforced by epoch)
Remaining for host:      75 µs per mod (action validation, dispatch)
```

Mods that consistently approach the 50 µs limit receive a warning in the mod performance log. A mod that hits the limit on more than 10 consecutive ticks is flagged as `ModStatus::Degraded` and the engine emits a `ModDegradedEvent` that the client UI can surface.

### 3.4 Permitted and Denied Host Functions

The host exports a fixed set of functions to mod WASM instances. No other imports are satisfied; a mod that imports an undeclared function will fail at instantiation with `ModLoadError::UnsatisfiedImport`.

**Permitted imports (module `civlab`):**

| Function Signature | Description |
|---|---|
| `log_i64(msg_ptr: i32, msg_len: i32, value: i64)` | Debug logging; output captured to mod log, not stdout |
| `rng_next_u64() -> u64` | Next value from tick-scoped seeded ChaCha20Rng |
| `rng_next_range(lo: i64, hi: i64) -> i64` | Bounded random integer in `[lo, hi)` |
| `fixed_mul(a: i64, b: i64, scale: i32) -> i64` | Fixed-point multiplication: `(a * b) >> scale` |
| `fixed_div(a: i64, b: i64, scale: i32) -> i64` | Fixed-point division: `(a << scale) / b` |
| `world_read(query_ptr: i32, query_len: i32, out_ptr: i32, out_cap: i32) -> i32` | Read world state slice; returns bytes written |
| `action_emit(action_ptr: i32, action_len: i32) -> i32` | Emit a serialized `ModAction`; returns 0 on accept, error code on reject |
| `panic_abort(msg_ptr: i32, msg_len: i32)` | Mod-initiated abort; host logs message and marks mod faulted |

**Denied syscall categories (any import from these modules is rejected):**

| Module | Reason |
|---|---|
| `wasi_snapshot_preview1` | Filesystem, network, environment, clock access |
| `wasi_preview2` | Same; newer WASI interface |
| `env` (non-civlab) | Uncontrolled host environment access |
| `pthread` | Thread spawn |
| Any unlisted module | Fail-closed: unknown import modules are rejected |

### 3.5 Determinism Enforcement at Load Time

After WASM binary validation, the host performs a determinism scan of the module's instruction stream before instantiation. The following instructions are rejected:

| Rejected Instruction | Reason |
|---|---|
| `f32.nearest`, `f64.nearest` | Platform-dependent rounding semantics |
| `f32.sqrt`, `f64.sqrt` | Bit-exact results not guaranteed across all IEEE 754 implementations |
| Any `f32`/`f64` instruction operating on values derived from host state | Float contamination of deterministic integer pipeline |
| `memory.atomic.*` | Not relevant in single-threaded WASM but rejected to prevent future misuse |

Note: `f32`/`f64` instructions are permitted for internal mod computations (e.g., display formatting, intermediate ML model inference) provided the results are not emitted as `ModAction` values affecting `i64` world-state fields. The scan enforces this by tracing data flow from `action_emit` call sites backward through the instruction graph. If any `f64` value reaches an `action_emit` argument without an explicit `i64.trunc` cast, the load is rejected with `ModLoadError::FloatContamination`.

---

## 4. Mod Manifest Format

Every mod bundle must include a `mod.toml` manifest at the bundle root. The manifest is parsed and validated before the WASM binary is loaded.

### 4.1 Full Manifest Schema

```toml
# mod.toml — CivLab Mod Manifest
# All fields required unless marked optional.

[mod]
# Unique mod identifier. Reverse-domain format recommended.
# Must match [a-z][a-z0-9-]{0,63}
id = "custom-carbon-tax"

# Human-readable display name.
name = "Carbon Tax Policy"

# Mod version (semver).
version = "1.0.0"

# CivLab API major version this mod targets.
# Must be an integer. Host accepts [current-1, current].
api_version = "1"

# One of: "policy", "economic", "event", "scenario"
mod_type = "policy"

# Author or organization name.
author = "CivLab Research Team"

# Short description (max 256 characters).
description = "Implements a carbon tax with household rebate mechanism. \
               Tax rate is dynamic based on atmospheric CO2 concentration."

# Optional: URL to documentation or source repository.
# homepage = "https://example.com/carbon-tax-mod"

# Optional: SPDX license identifier.
# license = "MIT"

[dependencies]
# Core API version range (semver range syntax).
# This is always required.
civlab-api = ">=1.0.0, <2.0.0"

# Optional: declare other mods this mod depends on.
# [dependencies.mods]
# "base-climate-events" = ">=1.0.0"

[permissions]
# Declare read permissions for each world-state domain.
# Host enforces: world_read() calls for undeclared domains are rejected.
read_economy   = true
read_climate   = true
read_military  = false
read_diplomacy = false
read_citizens  = false

# Declare write permissions (which action types are permitted).
# Host enforces: action_emit() calls for undeclared action types are rejected.
write_policy   = true   # SetTaxRate, SetPolicyParam
write_economy  = false  # Direct production/ledger modification
write_events   = false  # TriggerEvent (EventMod only)
write_scenario = false  # Scenario modification (ScenarioMod only)

# Optional: declare TransferFunds permission explicitly.
# Requires both read_economy and write_policy to be true.
transfer_funds = true

[runtime]
# Optional: override default runtime limits (cannot exceed host maximums).
# memory_mb = 32      # Default: 64 (host max: 64)
# cpu_us    = 30      # Per-callback limit in µs. Default: 50 (host max: 50)
```

### 4.2 Manifest Validation Rules

The host applies the following validation rules before WASM loading:

| Rule | Error if violated |
|---|---|
| `id` matches `[a-z][a-z0-9-]{0,63}` | `ManifestError::InvalidId` |
| `api_version` is an integer within host's accepted range | `ManifestError::IncompatibleApiVersion` |
| `mod_type` is one of the four permitted values | `ManifestError::UnknownModType` |
| `description` is at most 256 bytes | `ManifestError::DescriptionTooLong` |
| `civlab-api` version range is parseable semver | `ManifestError::InvalidDependencyRange` |
| `civlab-api` range overlaps host's current API version | `ManifestError::UnsatisfiedDependency` |
| No permission field requests more than the mod type allows | `ManifestError::PermissionExceedsModType` |
| `runtime.memory_mb` does not exceed 64 | `ManifestError::MemoryLimitExceedsMax` |
| `runtime.cpu_us` does not exceed 50 | `ManifestError::CpuLimitExceedsMax` |

### 4.3 Permission Enforcement at Runtime

Permissions declared in the manifest are compiled into a `ModCapabilitySet` stored in the host's `ModRegistry`. The `world_read()` host function checks the query's domain tag against the capability set before executing. The `action_emit()` host function checks the action's type discriminant against the capability set before enqueuing.

Violations are not silently ignored. A permission violation causes:

1. The specific call returns error code `ERR_PERMISSION_DENIED (2)`.
2. A `ModPermissionViolation { mod_id, tick, call, domain }` event is appended to the mod event log.
3. After 5 permission violations in a single tick, the mod is flagged `ModStatus::Suspended` and receives no further callbacks for that tick.
4. The suspension is lifted at the start of the next tick.

Repeated suspension across 10 consecutive ticks promotes the mod to `ModStatus::Faulted`, which requires explicit operator intervention to clear (`sim mod reset-fault \<mod_id\>`).

---

## 5. PolicyMod API

### 5.1 Trait Definition

```rust
// civlab-sdk/src/policy.rs
// This trait is compiled into the WASM guest via civlab-sdk.
// The host binds to the exported C ABI entry points generated
// by the #[civlab_mod] proc macro.

/// Primary trait for policy mods.
///
/// Implementors define custom policy algorithms that inject
/// PolicyAction values into the engine's Phase 3a pipeline.
///
/// # Determinism
///
/// All methods must be deterministic. Do not use std::time,
/// OsRng, or any non-civlab-sdk randomness source.
///
/// # Error Handling
///
/// Methods return Vec<PolicyAction>. An empty Vec is valid and
/// means "no actions this tick/event". Methods must not panic;
/// panics are caught by the host as ModTrap::GuestPanic and the
/// mod is faulted after 3 consecutive panics.
pub trait PolicyMod: Send + 'static {
    /// Called once per tick during Phase 3a (Policy Application).
    ///
    /// The host provides a PolicyContext containing read-only views
    /// of world state slices the mod declared read permission for.
    /// Attempting to access an undeclared domain in ctx panics
    /// in debug builds and returns a zero-value view in release builds.
    fn on_tick(&mut self, ctx: &PolicyContext) -> Vec<PolicyAction>;

    /// Called when a SimEvent is dispatched to this mod.
    ///
    /// Events are delivered to all loaded PolicyMod instances that
    /// declared interest in the event type (via ModMetadata::subscribed_events).
    /// The host may call on_event zero or more times per tick, after on_tick.
    fn on_event(&mut self, event: &SimEvent) -> Vec<PolicyAction>;

    /// Returns static metadata about this mod instance.
    ///
    /// Called once during mod_init(). The returned ModMetadata is
    /// stored in the host ModRegistry and not re-queried per tick.
    fn metadata(&self) -> ModMetadata;
}
```

### 5.2 PolicyContext

```rust
// civlab-sdk/src/policy.rs

/// Read-only view of world state provided to PolicyMod::on_tick.
///
/// Fields are only populated if the mod declared the corresponding
/// read permission in its manifest. Accessing an unpopulated field
/// returns a zero-value sentinel (e.g., EconomyView::default()).
#[derive(Debug)]
pub struct PolicyContext {
    /// Current simulation tick number.
    /// Monotonically increasing; starts at 0.
    pub tick: u64,

    /// Read-only view of the nation this mod is bound to.
    /// A mod may be bound to a specific nation (scenario assignment)
    /// or to the global policy layer (affects all nations).
    pub nation: NationView,

    /// Read-only snapshot of the economy state for this tick.
    /// Populated only if read_economy = true in manifest.
    pub economy: EconomyView,

    /// Read-only snapshot of the climate state for this tick.
    /// Populated only if read_climate = true in manifest.
    pub climate: ClimateView,

    /// Read-only snapshot of the diplomatic state for this tick.
    /// Populated only if read_diplomacy = true in manifest.
    pub diplomacy: DiplomacyView,

    /// Read-only snapshot of citizen welfare metrics for this tick.
    /// Populated only if read_citizens = true in manifest.
    pub citizens: CitizenView,
}

/// Snapshot of nation-level state visible to policy mods.
#[derive(Debug, Default, Clone)]
pub struct NationView {
    pub id: NationId,
    pub name: String,
    pub population: i64,
    pub territory_hex_count: i32,
    pub government_type: GovernmentType,
    pub policy_params: BTreeMap<String, i64>,
}

/// Snapshot of economy state visible to policy mods.
/// All monetary values are i64 fixed-point (scale: 1 unit = 1_000 milli-units).
#[derive(Debug, Default, Clone)]
pub struct EconomyView {
    pub gdp_millijoules: i64,
    pub tax_revenue_millijoules: i64,
    pub fiscal_balance_millijoules: i64,
    pub inflation_rate_bps: i64,         // basis points (1 bps = 0.01%)
    pub gini_coefficient_bps: i64,       // basis points (0 = perfect equality)
    pub goods: BTreeMap<GoodType, GoodMarketView>,
    pub sectors: BTreeMap<SectorId, SectorView>,
}

/// Per-good market state snapshot.
#[derive(Debug, Default, Clone)]
pub struct GoodMarketView {
    pub supply: i64,
    pub demand: i64,
    pub price_millijoules: i64,
    pub tax_rate_bps: i64,
    pub subsidy_rate_bps: i64,
}

/// Climate state snapshot visible to policy mods.
#[derive(Debug, Default, Clone)]
pub struct ClimateView {
    pub co2_ppm_milliunits: i64,         // CO2 concentration in milli-ppm
    pub global_temp_delta_millk: i64,    // Temperature delta from baseline in milli-Kelvin
    pub sea_level_mm: i64,               // Sea level in millimeters above baseline
    pub renewable_share_bps: i64,        // Renewable energy share in basis points
    pub carbon_budget_remaining_mt: i64, // Remaining carbon budget in megatons
}
```

### 5.3 PolicyAction

```rust
// civlab-sdk/src/policy.rs

/// Actions a PolicyMod can emit.
///
/// Actions are validated by the host before application.
/// Invalid actions (e.g., SetTaxRate with rate_bps > 10_000) are
/// logged and discarded; they do not cause the mod to fault.
///
/// All action values use the same fixed-point scale as the
/// corresponding world-state fields they modify.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PolicyAction {
    /// Set the tax rate for a specific good category.
    ///
    /// rate_bps: basis points (0..=10_000). Values outside range
    /// are clamped by host (not rejected; a warning is logged).
    SetTaxRate {
        good: GoodType,
        rate_bps: i64,
    },

    /// Set a subsidy rate for a specific good category.
    ///
    /// rate_bps: basis points (0..=5_000). Values above 5_000 are
    /// clamped; the subsidy cap is a hard policy invariant.
    SetSubsidyRate {
        good: GoodType,
        rate_bps: i64,
    },

    /// Transfer funds from one actor to another.
    ///
    /// Requires transfer_funds = true in manifest.
    /// Host validates: from actor exists, has sufficient balance,
    /// and the transfer does not violate conservation invariants.
    /// A failed transfer is logged and skipped; subsequent actions
    /// in the same Vec are still applied.
    TransferFunds {
        from: ActorId,
        to: ActorId,
        amount_millijoules: i64,
    },

    /// Trigger a named event to be injected into Phase 4.
    ///
    /// Requires write_events = true in manifest.
    /// event_type must be registered in mod metadata or be a
    /// built-in event type (see SimEvent::BuiltIn variants).
    /// payload is an opaque byte blob passed through to EventMod handlers.
    TriggerEvent {
        event_type: String,
        payload: Vec<u8>,
    },

    /// Set a named policy parameter on the mod's bound nation.
    ///
    /// Parameters are stored in NationView::policy_params and
    /// are accessible to other mods (read-only) and to the engine's
    /// metric reporting system.
    SetPolicyParam {
        key: String,
        value: i64,
    },

    /// Set a nominal interest rate for the bound nation's fiscal system.
    ///
    /// rate_bps: basis points (0..=3_000). Affects CivLab's fiscal
    /// balance simulation if interest rates are enabled in scenario.
    SetInterestRate {
        rate_bps: i64,
    },
}
```

### 5.4 ModMetadata

```rust
// civlab-sdk/src/common.rs

/// Static metadata returned by PolicyMod::metadata() during init.
#[derive(Debug, Clone)]
pub struct ModMetadata {
    /// Mod identifier. Must match the id field in mod.toml.
    pub id: String,

    /// Display name for UI and logging.
    pub name: String,

    /// Semver version of this mod.
    pub version: String,

    /// Set of SimEvent type strings this mod wants on_event called for.
    /// Glob patterns supported: "climate.*", "diplomacy.war.*".
    /// Empty set means on_event is never called (reduces host overhead).
    pub subscribed_events: Vec<String>,

    /// If true, this mod's on_tick is called even during fast-forward
    /// (simulation running at >10x real-time speed).
    /// If false, on_tick is skipped during fast-forward and only called
    /// at normal speed. Default: true.
    pub run_during_fast_forward: bool,
}
```

---

## 6. EconomicMod API

### 6.1 Trait Definition

```rust
// civlab-sdk/src/economic.rs

/// Trait for economic mods that override or supplement the host's
/// production, market clearing, and consumption calculations.
///
/// Unlike PolicyMod (which layers actions on top of the default engine),
/// EconomicMod replaces default behavior for the sectors/goods it is
/// registered for. The scenario configuration determines which sectors
/// and goods route through which EconomicMod.
///
/// An EconomicMod must be purely functional: given the same context,
/// it must return the same result. No internal mutable state may
/// influence the result (mutable state is allowed for caching, but
/// cache misses must produce identical results to cache hits).
pub trait EconomicMod: Send + 'static {
    /// Called during Phase 3b (Production) for each sector this mod
    /// is registered as the production handler for.
    ///
    /// Returns a ProductionResult specifying output quantities.
    /// The host applies conservation checks: output cannot exceed
    /// input resources + declared production efficiency. A
    /// ProductionResult that violates conservation is rejected with
    /// a logged ModConservationViolation event; the default production
    /// formula is used as fallback for that sector on that tick.
    fn on_production(
        &self,
        ctx: &ProductionContext,
    ) -> ProductionResult;

    /// Called during Phase 3c (Market Clearing) for each market
    /// this mod is registered as the allocator for.
    ///
    /// Returns a MarketResult specifying allocation decisions.
    /// The host validates: total allocated quantity does not exceed
    /// available supply. Over-allocation is rejected with a logged
    /// ModConservationViolation; the default market clearing runs
    /// as fallback for that market on that tick.
    fn on_market_clear(
        &self,
        ctx: &MarketContext,
    ) -> MarketResult;

    /// Called during Phase 3c (Consumption) for each consumption
    /// category this mod is registered as the consumption handler for.
    ///
    /// Returns a ConsumptionResult specifying actual consumption amounts
    /// (may be less than requested if supply is insufficient).
    fn on_consumption(
        &self,
        ctx: &ConsumptionContext,
    ) -> ConsumptionResult;

    /// Returns static metadata about this mod instance.
    fn metadata(&self) -> ModMetadata;

    /// Returns the set of sectors this mod handles production for.
    /// Called once during mod_init(). Empty means no production override.
    fn handled_production_sectors(&self) -> Vec<SectorId>;

    /// Returns the set of goods this mod handles market clearing for.
    /// Called once during mod_init(). Empty means no market clearing override.
    fn handled_market_goods(&self) -> Vec<GoodType>;

    /// Returns the custom GoodType definitions introduced by this mod.
    /// Host registers these types in the global GoodType registry.
    /// Empty for mods that do not introduce new good types.
    fn custom_good_types(&self) -> Vec<CustomGoodDescriptor>;
}
```

### 6.2 Production Contexts and Results

```rust
// civlab-sdk/src/economic.rs

/// Context provided to EconomicMod::on_production.
#[derive(Debug)]
pub struct ProductionContext {
    pub tick: u64,
    pub sector: SectorView,

    /// Available input resources for this production cycle.
    /// Keys: GoodType. Values: available quantity (fixed-point i64).
    pub inputs: BTreeMap<GoodType, i64>,

    /// Current capacity utilization rate in basis points (0..=10_000).
    pub capacity_utilization_bps: i64,

    /// Climate conditions affecting production (e.g., solar irradiance).
    /// Only populated if read_climate = true in manifest.
    pub climate: ClimateView,

    /// Seeded RNG for stochastic production (e.g., crop yield variance).
    /// Use this instead of any external RNG source.
    pub rng: ModRng,
}

/// Result of a production computation.
#[derive(Debug, Default)]
pub struct ProductionResult {
    /// Output quantities by GoodType.
    /// The host will verify: sum(output_joules) <= sum(input_joules) * efficiency.
    pub outputs: BTreeMap<GoodType, i64>,

    /// Input quantities consumed (must not exceed ctx.inputs values).
    pub inputs_consumed: BTreeMap<GoodType, i64>,

    /// Waste/loss quantities (e.g., heat loss, emissions).
    /// Included in conservation accounting; must balance with inputs_consumed.
    pub waste: BTreeMap<GoodType, i64>,

    /// Optional: emissions produced (CO2-equivalent in milligrams).
    /// Passed to the climate module for CO2 accounting.
    pub co2_emissions_mg: i64,
}

/// Context provided to EconomicMod::on_market_clear.
#[derive(Debug)]
pub struct MarketContext {
    pub tick: u64,
    pub good: GoodType,

    /// Total available supply this tick.
    pub supply: i64,

    /// All pending demand bids, sorted by willingness-to-pay (descending).
    pub bids: Vec<MarketBid>,

    /// Current price from last tick (starting reference price).
    pub last_price_millijoules: i64,
}

#[derive(Debug, Clone)]
pub struct MarketBid {
    pub actor: ActorId,
    pub quantity: i64,
    pub max_price_millijoules: i64,     // Maximum price this actor will pay
    pub priority: AllocationPriority,   // Used in planned-economy allocators
}

/// Result of market clearing for one good.
#[derive(Debug, Default)]
pub struct MarketResult {
    /// Per-actor allocation decisions.
    pub allocations: Vec<MarketAllocation>,

    /// Clearing price (may differ from last_price; used for price updates).
    pub clearing_price_millijoules: i64,

    /// Quantity unallocated (shortage). Must equal supply - sum(allocations.quantity).
    pub unmet_demand: i64,
}

#[derive(Debug, Clone)]
pub struct MarketAllocation {
    pub actor: ActorId,
    pub quantity: i64,
    pub price_millijoules: i64,
}

/// Descriptor for a new custom GoodType introduced by this mod.
#[derive(Debug, Clone)]
pub struct CustomGoodDescriptor {
    /// Unique identifier for this good type.
    /// Must be unique across all loaded mods. Collision causes ModLoadError::GoodTypeConflict.
    pub id: String,

    /// Human-readable name for UI display.
    pub name: String,

    /// Joule-equivalent per unit (for conservation accounting).
    /// If None, this good is tracked in quantity only (not joule-equivalent balanced).
    pub joule_equivalent: Option<i64>,

    /// If true, this good participates in the carbon accounting ledger.
    pub carbon_tracked: bool,
}
```

---

## 7. EventMod API

### 7.1 Trait Definition

```rust
// civlab-sdk/src/event.rs

/// Trait for mods that register custom event triggers and handlers.
///
/// EventMod instances are called during Phase 4 (Stochastic Events).
/// They can generate new events, respond to existing events, and
/// return SimAction values that modify world state.
pub trait EventMod: Send + 'static {
    /// Called once during mod_init() to register this mod's event handlers.
    ///
    /// The host stores the returned handlers and evaluates their triggers
    /// each tick. This method is not called per-tick; handler registration
    /// is static for the mod's lifetime.
    fn register_handlers(&self) -> Vec<EventHandler>;

    /// Called when a CustomEvent addressed to this mod fires.
    ///
    /// A CustomEvent is fired when:
    /// - A trigger in one of this mod's registered EventHandlers evaluates to true.
    /// - Another mod emits a TriggerEvent action naming an event type this mod
    ///   declared interest in via register_handlers().
    ///
    /// Returns a Vec<SimAction> to be applied by the host.
    fn on_custom_event(&mut self, event: &CustomEvent) -> Vec<SimAction>;

    /// Returns static metadata about this mod instance.
    fn metadata(&self) -> ModMetadata;
}
```

### 7.2 Event Handler Registration

```rust
// civlab-sdk/src/event.rs

/// A registered event handler: a trigger condition paired with a handler.
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// The name of this handler (used in logs and telemetry).
    pub name: String,

    /// Condition that must be true for the handler to fire.
    pub trigger: EventTrigger,

    /// The name of the CustomEvent type that will be generated when
    /// this trigger fires. This name is passed to on_custom_event().
    pub fires_event: String,
}

/// Trigger conditions for EventHandler.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum EventTrigger {
    /// Fire every N ticks.
    EveryNTicks { n: u64 },

    /// Fire once at a specific tick.
    AtTick { tick: u64 },

    /// Fire when a named world-state metric crosses a threshold.
    ThresholdCrossing {
        metric: String,
        threshold: i64,
        direction: ThresholdDirection,
    },

    /// Fire when an event matching a glob pattern is dispatched.
    OnEventPattern { pattern: String },

    /// Fire with probability p per tick (deterministic: uses mod's RNG).
    /// p_per_million: probability in millionths (e.g., 1_000 = 0.1%).
    Probabilistic { p_per_million: i64 },

    /// Logical AND of multiple triggers.
    All { triggers: Vec<EventTrigger> },

    /// Logical OR of multiple triggers.
    Any { triggers: Vec<EventTrigger> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdDirection {
    RisingAbove,
    FallingBelow,
}

/// A custom event dispatched to an EventMod's on_custom_event().
#[derive(Debug)]
pub struct CustomEvent {
    pub tick: u64,
    pub event_name: String,

    /// World state at the time the trigger fired.
    pub world_snapshot: WorldSnapshot,

    /// Optional payload from a TriggerEvent action.
    pub payload: Vec<u8>,

    /// Source RNG for this event (deterministic, tick-scoped).
    pub rng: ModRng,
}
```

### 7.3 SimAction (EventMod Actions)

```rust
// civlab-sdk/src/event.rs

/// Actions an EventMod can return from on_custom_event().
///
/// This is a superset of PolicyAction: EventMod has additional
/// action types for modifying physical world state (climate, territory).
/// All actions require corresponding write permissions in the manifest.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SimAction {
    /// All PolicyAction variants are available to EventMod.
    Policy(PolicyAction),

    /// Apply a climate perturbation.
    /// Requires write_climate = true (future permission; not in v1).
    /// Reserved for v2 API.
    Climate(ClimateAction),

    /// Spawn a world event (e.g., natural disaster) at a location.
    SpawnWorldEvent {
        event_class: WorldEventClass,
        location: HexCoord,
        severity: i64,         // Severity in basis points (0..=10_000)
        duration_ticks: u32,
    },

    /// Modify a named counter in the research data export.
    /// These counters appear in simulation metrics but do not affect gameplay.
    RecordMetric {
        key: String,
        value: i64,
        aggregation: MetricAggregation,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldEventClass {
    Drought,
    Flood,
    Hurricane,
    Earthquake,
    HeatWave,
    /// Custom event class defined by this mod.
    Custom(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricAggregation {
    Sum,
    Max,
    Min,
    LastValue,
}
```

---

## 8. ScenarioMod API

### 8.1 Trait Definition

```rust
// civlab-sdk/src/scenario.rs

/// Trait for mods that define complete simulation scenarios.
///
/// A ScenarioMod is used once: it constructs the initial WorldState
/// at simulation start. After init, it may optionally participate in
/// victory condition checking each tick.
pub trait ScenarioMod: Send + 'static {
    /// Build and return the initial scenario configuration.
    ///
    /// Called once before Tick 0. The host applies the returned
    /// ScenarioDescriptor to construct the initial WorldState.
    fn build_scenario(&self) -> ScenarioDescriptor;

    /// Optional: called each tick during Phase 5 (Victory Check).
    ///
    /// If this returns Some(VictoryResult), the simulation ends.
    /// Return None to continue. Only called if
    /// ScenarioDescriptor::custom_victory_condition is true.
    fn check_victory(&self, world: &WorldView) -> Option<VictoryResult>;

    /// Optional: generate a custom hex map for this scenario.
    ///
    /// Called once before build_scenario() if
    /// ScenarioDescriptor::custom_map_generation is true.
    /// The returned MapDescriptor is used as the world map.
    fn generate_map(&self, seed: u64) -> MapDescriptor;

    /// Returns static metadata about this mod instance.
    fn metadata(&self) -> ModMetadata;
}
```

### 8.2 ScenarioDescriptor

```rust
// civlab-sdk/src/scenario.rs

/// Complete description of a simulation starting configuration.
#[derive(Debug, Clone)]
pub struct ScenarioDescriptor {
    /// Scenario display name.
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// Starting nations. At least 2 required.
    pub nations: Vec<NationDescriptor>,

    /// Climate parameters at simulation start.
    pub climate_initial: ClimateInitial,

    /// Economic parameters at simulation start.
    pub economy_initial: EconomyInitial,

    /// Global event probability multipliers.
    /// Key: event class name. Value: multiplier in basis points (10_000 = 1.0x).
    pub event_probability_modifiers: BTreeMap<String, i64>,

    /// If true, check_victory() is called each tick.
    pub custom_victory_condition: bool,

    /// If true, generate_map() is called to build the world map.
    pub custom_map_generation: bool,

    /// Maximum tick count. Simulation ends at this tick if no victory.
    /// None means no tick limit.
    pub max_ticks: Option<u64>,
}

/// Starting configuration for one nation.
#[derive(Debug, Clone)]
pub struct NationDescriptor {
    pub id: NationId,
    pub name: String,
    pub government_type: GovernmentType,
    pub initial_population: i64,
    pub initial_cities: Vec<CityDescriptor>,
    pub initial_treasury_millijoules: i64,
    pub initial_policy_params: BTreeMap<String, i64>,
    pub territory_hexes: Vec<HexCoord>,
    pub ai_controller: AiController,
}

#[derive(Debug, Clone)]
pub enum AiController {
    /// Native engine AI (default).
    Native { difficulty: i32 },
    /// Player-controlled (waits for player input events).
    Player,
    /// Controlled by a loaded PolicyMod with the given mod id.
    PolicyMod { mod_id: String },
}

/// Victory condition result.
#[derive(Debug, Clone)]
pub struct VictoryResult {
    pub winner: Option<NationId>,
    pub condition_description: String,
    pub final_metrics: BTreeMap<String, i64>,
}

/// Custom map descriptor.
#[derive(Debug, Clone)]
pub struct MapDescriptor {
    /// Hex grid dimensions.
    pub width: u32,
    pub height: u32,

    /// Per-hex terrain types. Length must equal width * height.
    pub terrain: Vec<TerrainType>,

    /// Resource deposits by hex coordinate.
    pub resources: BTreeMap<HexCoord, Vec<ResourceDeposit>>,

    /// Named geographic regions.
    pub regions: Vec<GeographicRegion>,
}
```

---

## 9. civlab-sdk — Guest-Side Rust Library

### 9.1 Overview

`civlab-sdk` is a Rust crate published to the CivLab internal crate registry. It compiles to `wasm32-unknown-unknown` and provides everything a mod author needs to implement the trait contracts defined in this spec: trait definitions, view types, action types, the `#[civlab_mod]` proc macro, fixed-point math, and the seeded RNG interface.

Mod authors do not interact with raw WASM ABI functions directly. The SDK abstracts the ABI into idiomatic Rust.

### 9.2 Crate Layout

```
civlab-sdk/
  Cargo.toml
  src/
    lib.rs           -- Re-exports all public types; feature flags
    common.rs        -- ModMetadata, ActorId, NationId, GoodType, SectorId, HexCoord
    policy.rs        -- PolicyMod trait, PolicyContext, PolicyAction
    economic.rs      -- EconomicMod trait, ProductionContext, MarketContext, ConsumptionContext
    event.rs         -- EventMod trait, EventHandler, EventTrigger, SimAction
    scenario.rs      -- ScenarioMod trait, ScenarioDescriptor, VictoryResult, MapDescriptor
    rng.rs           -- ModRng: deterministic ChaCha20Rng wrapper
    math.rs          -- Fixed-point math utilities (wraps host fixed_mul/fixed_div)
    abi/
      mod.rs         -- Raw C ABI bindings (generated; do not edit manually)
      host_fns.rs    -- Extern "C" declarations for civlab host imports
      dispatch.rs    -- C ABI entry points that dispatch into trait impls
    macros/          -- Separate proc-macro crate: civlab-sdk-macros
      src/
        lib.rs       -- #[civlab_mod] derive and registration macro
```

### 9.3 The `#[civlab_mod]` Proc Macro

The `#[civlab_mod]` attribute macro performs three transformations on an annotated struct:

1. **ABI entry point generation.** It generates `#[no_mangle] extern "C"` functions that wasmtime calls:
   - `mod_init() -> i32` — called once; calls `ModMetadata::metadata()`, registers with host.
   - `mod_on_tick(ctx_ptr: i32, ctx_len: i32) -> i32` — deserializes `PolicyContext`, calls `on_tick()`, serializes actions, writes to output buffer, returns output length.
   - `mod_on_event(event_ptr: i32, event_len: i32) -> i32` — same pattern for `on_event()`.
   - `mod_alloc(size: i32) -> i32` — simple bump allocator for host-to-mod data passing.
   - `mod_free(ptr: i32, size: i32)` — corresponding dealloc.

2. **Serialization wiring.** Context structs arriving from the host are encoded in `postcard` binary format (compact, no_std compatible). The macro generates deserialization calls. Action vecs are serialized in `postcard` format before being passed to `action_emit()`.

3. **Panic handler installation.** The macro sets a global panic handler that calls `panic_abort()` with the panic message.

**Usage:**

```rust
// my_mod/src/lib.rs
use civlab_sdk::prelude::*;

#[civlab_mod]
pub struct CarbonTaxMod {
    accumulated_revenue_millijoules: i64,
}

impl PolicyMod for CarbonTaxMod {
    fn on_tick(&mut self, ctx: &PolicyContext) -> Vec<PolicyAction> {
        // Implementation in Section 13.1
        vec![]
    }

    fn on_event(&mut self, _event: &SimEvent) -> Vec<PolicyAction> {
        vec![]
    }

    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "custom-carbon-tax".into(),
            name: "Carbon Tax Policy".into(),
            version: "1.0.0".into(),
            subscribed_events: vec!["climate.*".into()],
            run_during_fast_forward: true,
        }
    }
}
```

The `#[civlab_mod]` macro requires exactly one `impl PolicyMod` (or `EconomicMod`, `EventMod`, `ScenarioMod`) block in scope. Multiple trait implementations on the same struct are a compile error (a mod cannot be both a `PolicyMod` and an `EconomicMod` in the same WASM binary; use separate `.civmod` files).

### 9.4 Fixed-Point Math

All monetary, energy, and resource quantities in the SDK use `i64` fixed-point arithmetic with a scale of `1 unit = 1_000_000 micro-units` (6 decimal places). The `civlab_sdk::math` module provides:

```rust
// civlab-sdk/src/math.rs

/// Fixed-point scale: 1 unit = SCALE micro-units.
pub const SCALE: i64 = 1_000_000;

/// Multiply two fixed-point values: (a * b) / SCALE.
/// Uses the host's fixed_mul() for guaranteed bit-exact results.
#[inline]
pub fn fp_mul(a: i64, b: i64) -> i64 {
    unsafe { host_fns::fixed_mul(a, b, 20) }  // 2^20 &asymp; 1_000_000
}

/// Divide two fixed-point values: (a * SCALE) / b.
/// Uses the host's fixed_div() for guaranteed bit-exact results.
/// Panics (via panic_abort) if b == 0.
#[inline]
pub fn fp_div(a: i64, b: i64) -> i64 {
    if b == 0 {
        panic!("fixed-point division by zero");
    }
    unsafe { host_fns::fixed_div(a, b, 20) }
}

/// Convert an integer to fixed-point.
#[inline]
pub fn to_fp(n: i64) -> i64 { n * SCALE }

/// Convert fixed-point to integer (truncating).
#[inline]
pub fn from_fp(n: i64) -> i64 { n / SCALE }

/// Clamp a fixed-point value to [lo, hi].
#[inline]
pub fn fp_clamp(v: i64, lo: i64, hi: i64) -> i64 {
    v.max(lo).min(hi)
}

/// Compute percentage of a fixed-point value.
/// pct_bps: percentage in basis points (100 bps = 1%).
#[inline]
pub fn apply_rate_bps(value: i64, rate_bps: i64) -> i64 {
    fp_mul(value, rate_bps) / 10_000
}
```

The `fixed` crate is also re-exported for mods that need arbitrary-precision fixed-point types:

```rust
pub use fixed::{FixedI64, FixedI128, types::extra::*};
```

### 9.5 Seeded RNG Interface

```rust
// civlab-sdk/src/rng.rs

/// Deterministic RNG provided to mods per tick.
///
/// Wraps the host's rng_next_u64() import. Seed is derived by the host
/// from the global simulation seed and the current tick number:
///     rng_seed = ChaCha20(global_seed, tick_number, mod_id_hash)
///
/// This ensures two mods with different IDs have independent RNG streams
/// even within the same tick, while the global seed still determines all
/// outcomes deterministically.
pub struct ModRng {
    _private: (),  // Opaque; construction only via SDK internals
}

impl ModRng {
    /// Generate the next uniform u64.
    pub fn next_u64(&mut self) -> u64 {
        unsafe { host_fns::rng_next_u64() }
    }

    /// Generate a uniform i64 in [lo, hi).
    /// Panics if lo >= hi.
    pub fn next_range(&mut self, lo: i64, hi: i64) -> i64 {
        assert!(lo < hi, "ModRng::next_range: lo must be < hi");
        unsafe { host_fns::rng_next_range(lo, hi) }
    }

    /// Generate a uniform bool with probability p_per_million / 1_000_000.
    pub fn next_bool_p(&mut self, p_per_million: i64) -> bool {
        let threshold = (u64::MAX / 1_000_000) * (p_per_million as u64);
        self.next_u64() < threshold
    }

    /// Shuffle a slice in place using Fisher-Yates algorithm.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let n = slice.len();
        for i in (1..n).rev() {
            let j = self.next_range(0, (i + 1) as i64) as usize;
            slice.swap(i, j);
        }
    }
}
```

### 9.6 Build Instructions

Mods are compiled to WASM using standard Rust tooling:

```bash
# Install the WASM target once
rustup target add wasm32-unknown-unknown

# Build a release mod binary
cargo build \
  --target wasm32-unknown-unknown \
  --release \
  --package my-carbon-tax-mod

# Output: target/wasm32-unknown-unknown/release/my_carbon_tax_mod.wasm

# Optional: optimize with wasm-opt (recommended for distribution)
wasm-opt -O3 -o mod.wasm \
  target/wasm32-unknown-unknown/release/my_carbon_tax_mod.wasm

# Package into .civmod bundle
civlab pack \
  --manifest mod.toml \
  --wasm mod.wasm \
  --output custom-carbon-tax-1.0.0.civmod \
  --sign ~/.civlab/keys/my-ed25519.key
```

**Cargo.toml configuration for a mod:**

```toml
[package]
name = "my-carbon-tax-mod"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]   # Required: produces a .wasm shared library

[dependencies]
civlab-sdk = { version = "1.0", registry = "civlab" }

[profile.release]
opt-level = "z"           # Minimize binary size
lto = true
codegen-units = 1
panic = "abort"           # Required: no unwinding in WASM

[target.wasm32-unknown-unknown.dependencies]
# No std dependencies; civlab-sdk is no_std compatible
```

### 9.7 SDK Feature Flags

| Feature | Default | Description |
|---|---|---|
| `policy` | enabled | Includes `PolicyMod` trait and associated types |
| `economic` | enabled | Includes `EconomicMod` trait and associated types |
| `event` | enabled | Includes `EventMod` trait and associated types |
| `scenario` | enabled | Includes `ScenarioMod` trait and associated types |
| `fixed-math` | enabled | Includes `civlab_sdk::math` fixed-point utilities |
| `rng` | enabled | Includes `ModRng` |
| `serde-derive` | disabled | Enables `#[derive(Serialize, Deserialize)]` on SDK types for external tooling |
| `dev-mock` | disabled | Includes mock host implementations for unit testing outside WASM (see Section 16.1) |

Mods should enable only the features they use to minimize binary size:

```toml
[dependencies]
civlab-sdk = { version = "1.0", registry = "civlab", default-features = false, features = ["policy", "rng"] }
```

---

## 10. Mod Loading and Lifecycle

### 10.1 Loading Sequence

The following sequence describes the complete lifecycle of a mod from bundle submission to first tick callback.

```
Step 1: Bundle submission
  sim.mod.load(mod_id: String, civmod_bytes: Vec<u8>) -> Result<(), ModLoadError>

Step 2: Bundle unpacking
  Host unzips .civmod archive
  Extracts: mod.toml, mod.wasm, optional assets/
  Assets stored in read-only mod asset store (keyed by mod_id)

Step 3: Manifest parsing and validation
  Host parses mod.toml per Section 4 rules
  All manifest validation rules applied
  ModCapabilitySet constructed from [permissions] section
  On validation failure: return ModLoadError::Manifest(ManifestError)

Step 4: Signature verification
  Host reads mod.wasm.sig (Ed25519 signature file in bundle)
  Verifies signature against mod.wasm content and author public key
  Author public key retrieved from mod registry or bundle-embedded cert
  On failure (unless mod-dev feature): return ModLoadError::InvalidSignature

Step 5: WASM binary validation
  wasmtime engine validates WASM binary structure (magic, sections, types)
  On malformed binary: return ModLoadError::InvalidWasm(WasmError)

Step 6: Determinism scan
  Host performs static analysis of WASM instruction stream
  Rejects non-deterministic float operations as defined in Section 3.5
  Checks all extern imports are satisfied by civlab host function set
  On determinism violation: return ModLoadError::NonDeterministicInstruction(InstrName)
  On unsatisfied import: return ModLoadError::UnsatisfiedImport(ImportName)

Step 7: WASM instantiation
  wasmtime creates Engine, Module, Store with limits from Section 3.2
  Epoch interruption configured per Section 3.3
  Host functions linked into the Store's Linker
  ModCapabilitySet stored in Store::data() for runtime permission checks
  Instance created via Linker::instantiate()
  On instantiation failure: return ModLoadError::InstantiationFailed(WasmError)

Step 8: mod_init() call
  Host calls exported mod_init() -> i32
  Mod registers itself (metadata, handler list, custom goods)
  Host receives ModMetadata via shared buffer
  Host stores metadata in ModRegistry
  Return value 0 = success; non-zero = mod-defined error code
  On non-zero return: return ModLoadError::InitFailed(code)

Step 9: Mod registered
  ModRegistry::insert(mod_id, ModInstance { instance, store, metadata, capability_set })
  Mod status set to ModStatus::Active
  LoadSuccess event emitted to client event stream
  Mod will receive tick callbacks starting at next tick boundary
```

### 10.2 Tick Callback Sequence

Each tick, the host invokes active mods in the following order within Phase 3a:

```
For each active PolicyMod (in deterministic load order):
  1. store.set_epoch_deadline(5)          -- Reset CPU budget
  2. Serialize PolicyContext into mod's input buffer
     (via mod_alloc() to obtain guest pointer)
  3. Call mod_on_tick(ctx_ptr, ctx_len) -> out_len
  4. If InterruptTrap: log ModTimeout, skip this mod's actions
  5. If GuestTrap (non-interrupt): log ModTrap, increment fault counter
     After 3 consecutive faults: set ModStatus::Faulted
  6. If out_len >= 0: read out_len bytes from guest output buffer
  7. Deserialize Vec<PolicyAction> from output bytes
  8. For each PolicyAction:
     a. Check action type against ModCapabilitySet
     b. Validate action parameters (ranges, actor existence)
     c. If valid: enqueue in ActionQueue for Phase 3a application
     d. If invalid: log ModActionRejected event, continue
```

### 10.3 Mid-Simulation Mod Swap

For research use cases, a mod can be swapped at a tick boundary without stopping the simulation:

```
sim.mod.swap(old_mod_id: String, new_mod_bytes: Vec<u8>) -> Result<(), ModLoadError>
```

The swap is atomic at a tick boundary:
1. At the start of a tick, if a swap is pending, `old_mod_id` is unloaded (its store is dropped, memory freed).
2. The new mod goes through the full load sequence (Steps 2-9 above).
3. The new mod's first callback is the current tick's `on_tick()`.

Swap is primarily intended for A/B experiments where a researcher wants to compare two versions of a policy algorithm under identical scenario conditions. The simulation seed and tick state are identical; only the mod changes.

### 10.4 Unloading

```
sim.mod.unload(mod_id: String) -> Result<(), ModUnloadError>
```

Unloading is deferred to the next tick boundary (same as swap). At unload:
1. `ModStatus::Unloading` is set immediately (no further callbacks queued).
2. At tick boundary: Store is dropped (all mod memory freed by wasmtime).
3. ModRegistry removes the entry.
4. `ModUnloadedEvent` emitted.
5. Any pending actions from the last tick already in the ActionQueue are still applied (the unload does not retroactively discard queued actions).

### 10.5 Mod Status State Machine

```
              load()         mod_init() succeeds
 (not loaded) --------> Loading -----------------> Active
                                                      |
                          fault_count >= 3            |  timeout > 10 consecutive
                         /                            v
                    Faulted <---------- Degraded <----+
                        |
    operator: reset-fault
                        |
                        v
                     Active

             unload()
 Active ----------------> Unloading --------> (not loaded)
```

| Status | Callbacks | Operator Action Required |
|---|---|---|
| `Active` | Receives tick callbacks | None |
| `Degraded` | Receives tick callbacks (with logged warnings) | None (self-recovers if timeouts stop) |
| `Faulted` | No callbacks | `sim mod reset-fault \<mod_id\>` |
| `Unloading` | No callbacks | None |

---

## 11. Mod Distribution Format

### 11.1 `.civmod` Bundle Structure

A `.civmod` file is a ZIP archive (deflate compressed) with the following structure:

```
custom-carbon-tax-1.0.0.civmod (ZIP)
  mod.toml              -- Manifest (Section 4; required)
  mod.wasm              -- WASM binary (required)
  mod.wasm.sig          -- Ed25519 signature of mod.wasm (required except in dev mode)
  assets/               -- Optional asset directory
    icons/
      mod-icon-64.png   -- 64x64 PNG mod icon for UI (optional)
    data/
      tax-schedule.bin  -- Pre-baked data files (read-only; accessible via world_read asset query)
  CHANGELOG.md          -- Optional version history
  LICENSE               -- Optional license text
```

Constraints:
- Total bundle size limit: 16 MB (enforced at `sim.mod.load()` call).
- `mod.wasm` size limit: 8 MB (the remaining 8 MB is for assets and metadata).
- Assets are immutable at runtime; mods read them via `world_read()` with query type `AssetQuery { mod_id, path }`.
- The ZIP must not contain absolute paths or path traversal components (`../`).

### 11.2 Signature Scheme

Mod signing uses Ed25519 with the following protocol:

```
Private key:  ~/.civlab/keys/<author>.ed25519.pem  (mod author's machine; never distributed)
Public key:   ~/.civlab/keys/<author>.ed25519.pub   (distributed via mod registry)
Signature:    Ed25519(private_key, SHA3-256(mod.wasm))

Signing (mod author):
  civlab sign \
    --key ~/.civlab/keys/my-ed25519.key \
    --input mod.wasm \
    --output mod.wasm.sig

Verification (host, at load time):
  1. Retrieve author's public key from mod registry by author identity in mod.toml
  2. Compute SHA3-256(mod.wasm)
  3. Ed25519::verify(public_key, hash, signature)
  4. On failure: ModLoadError::InvalidSignature
```

**Development mode bypass:** When the host engine is compiled with `--features mod-dev` (never in production builds), signature verification is skipped. A warning is logged to the mod audit log for every unsigned mod loaded. Production simulation servers do not compile with `mod-dev`.

### 11.3 Mod Registry Protocol

The CivLab mod registry exposes an HTTP API for discovery and download:

```
Base URL: https://mods.civlab.io/v1/

GET  /mods                    -- List all published mods (paginated)
GET  /mods/{mod_id}           -- Get mod metadata and version list
GET  /mods/{mod_id}/{version} -- Download .civmod bundle
POST /mods                    -- Publish a new mod (authenticated)
GET  /authors/{author_id}/pubkey  -- Retrieve author's Ed25519 public key
```

The registry is read-only from the simulation engine's perspective. The engine never POSTs to the registry; bundle download and publication are CLI operations.

**CLI download:**
```bash
civlab mod install custom-carbon-tax@1.0.0
# Downloads to ~/.civlab/mods/custom-carbon-tax-1.0.0.civmod
# Verifies signature using registry-provided public key
# Loads into running simulation: sim.mod.load("custom-carbon-tax", bundle_bytes)
```

### 11.4 Versioning and Compatibility

The host enforces compatibility at load time via the manifest's `civlab-api` version range. The compatibility rules follow semver:

| Host API Version | Accepted Mod api_version | Policy |
|---|---|---|
| 1 | `1` | Current stable |
| 1 | `0` | Transition window (deprecated; removed in host v2) |
| 2 | `2` | Current stable |
| 2 | `1` | Transition window |

Mods must re-declare their `api_version` when targeting a new major API version. A re-compilation against the new `civlab-sdk` major version is required (ABI is not compatible across API major versions).

---

## 12. Scenario Scripting Alternative — Lua

### 12.1 When to Use Lua

The WASM mod system is powerful but has friction: WASM compilation, `wasm32-unknown-unknown` toolchain, binary signing. For research teams that need to quickly iterate on parameter adjustments or simple policy variants, CivLab provides an alternative scripting path using **Lua 5.4** via the `mlua` Rust crate.

The Lua path is appropriate when:
- The script is under 10 KB of Lua source.
- The researcher needs to adjust parameters without a full compilation cycle.
- The policy logic can be expressed in the Lua API subset (no custom goods, no custom events).
- Development speed is more important than maximum isolation (Lua sandbox is less strict than WASM).

The Lua path is **not appropriate** when:
- The mod introduces new good types, new event types, or custom map generation.
- The mod will be distributed to other users (distribution requires WASM + signature).
- The mod is used in a published research result (WASM provides stronger reproducibility guarantees).

### 12.2 Lua Sandbox

The Lua environment is sandboxed via `mlua`'s `SafeLua` mode with the following configuration:

- **Standard libraries available:** `math`, `string`, `table` (limited subset).
- **Standard libraries denied:** `io`, `os`, `package`, `require`, `debug`, `coroutine`, `utf8`, `load`, `loadfile`, `dofile`.
- **No `require`:** All civlab bindings are pre-loaded into the global environment. Scripts cannot load external Lua modules.
- **Script size limit:** 10 240 bytes (10 KB). Scripts exceeding this limit are rejected at load time.
- **Per-call time limit:** 50 µs, enforced via Lua debug hooks (same budget as WASM).
- **Memory limit:** 4 MB (smaller than WASM; Lua scripts are intended to be simple).
- **Determinism:** Lua's `math.random` is replaced with a civlab-provided deterministic PRNG. `os.time`, `os.clock`, and `os.date` are not available.

### 12.3 Lua Policy API

The following globals are pre-loaded for Lua policy scripts:

```lua
-- civlab.lua (pre-loaded by mlua host; not a file, injected into Lua VM)

-- World state access (mirrors PolicyContext fields)
civlab.tick()                         -- Returns current tick (integer)
civlab.economy.gdp()                  -- Returns GDP in millijoules (integer)
civlab.economy.tax_revenue()          -- Returns current tick tax revenue
civlab.economy.good_price(good)       -- Returns price for named good (string key)
civlab.economy.good_supply(good)      -- Returns supply for named good
civlab.economy.good_tax_rate(good)    -- Returns current tax rate in bps
civlab.climate.co2_ppm()              -- Returns CO2 in milli-ppm
civlab.climate.temp_delta_mk()        -- Returns temp delta in milli-Kelvin
civlab.climate.renewable_share_bps()  -- Returns renewable energy share in bps

-- Action emission
civlab.action.set_tax_rate(good, rate_bps)       -- Emit SetTaxRate action
civlab.action.set_subsidy_rate(good, rate_bps)   -- Emit SetSubsidyRate action
civlab.action.transfer_funds(from, to, amount)   -- Emit TransferFunds action
civlab.action.set_policy_param(key, value)        -- Emit SetPolicyParam action

-- Fixed-point math helpers (host-backed, deterministic)
civlab.math.fp_mul(a, b)    -- Fixed-point multiply
civlab.math.fp_div(a, b)    -- Fixed-point divide
civlab.math.bps(value, bps) -- Apply basis-point rate: value * bps / 10000

-- Deterministic RNG
civlab.rng.next_int(lo, hi)   -- Uniform integer in [lo, hi)
civlab.rng.next_bool(p_bps)   -- Boolean with probability p_bps / 10000

-- Logging
civlab.log(msg)  -- Appends to mod log; not visible on stdout
```

### 12.4 Lua Script Structure

```lua
-- Example: carbon-tax-simple.lua
-- A simple carbon tax Lua script for rapid research iteration.

-- on_tick is the required entry point for policy Lua scripts.
-- Called once per tick with the world state pre-loaded into civlab.*
function on_tick()
    local co2 = civlab.climate.co2_ppm()
    local baseline_co2 = 280000  -- 280 ppm in milli-ppm

    -- Tax rate increases linearly with CO2 above baseline
    local excess_co2 = math.max(0, co2 - baseline_co2)
    local tax_rate_bps = civlab.math.bps(excess_co2, 50)  -- 0.5% per 100 milli-ppm excess
    tax_rate_bps = math.min(tax_rate_bps, 2000)            -- Cap at 20%

    civlab.action.set_tax_rate("fossil_fuel", tax_rate_bps)

    -- Rebate: transfer 80% of carbon tax revenue to citizens
    local revenue = civlab.economy.tax_revenue()
    local rebate = civlab.math.bps(revenue, 8000)  -- 80%
    if rebate > 0 then
        civlab.action.transfer_funds("treasury", "citizens_fund", rebate)
    end
end

-- on_event is optional; called for subscribed events
function on_event(event_type, payload)
    if event_type == "climate.co2_spike" then
        civlab.log("CO2 spike detected; applying emergency carbon levy")
        civlab.action.set_tax_rate("fossil_fuel", 3000)  -- Emergency 30% rate
    end
end
```

### 12.5 Lua Script Manifest

Lua scripts use a simplified manifest:

```toml
# lua-mod.toml
[mod]
id = "carbon-tax-simple"
name = "Simple Carbon Tax (Lua)"
version = "1.0.0"
api_version = "1"
mod_type = "policy"
script_type = "lua"          # Distinguishes from WASM mods
script_file = "policy.lua"   # Relative to manifest location
author = "Research Team"
description = "Carbon tax Lua script for rapid iteration."

[permissions]
read_economy = true
read_climate = true
write_policy = true
transfer_funds = true
```

---

## 13. Research Mod Examples

This section provides four complete, buildable mod examples. All examples use the `civlab-sdk` WASM path. Each is also included in the repository under `examples/mods/` and is run as part of the CI suite (see Section 16.4).

### 13.1 Example 1: Carbon Tax Policy (PolicyMod)

**File:** `examples/mods/carbon-tax/src/lib.rs`

```rust
//! Carbon Tax Policy Mod
//!
//! Implements a dynamic carbon tax where the rate scales linearly with
//! atmospheric CO2 concentration above the pre-industrial baseline of 280 ppm.
//! 80% of collected revenue is redistributed to a citizen rebate fund.
//! 20% is retained in the national treasury for clean energy investment.
//!
//! Tax rate formula:
//!   excess_co2_ppm = max(0, current_co2_ppm - 280)
//!   rate_bps = clamp(excess_co2_ppm * RATE_SLOPE_BPS_PER_PPM, 0, MAX_TAX_RATE_BPS)

use civlab_sdk::prelude::*;

/// Rate slope: 0.5 bps per milli-ppm of excess CO2.
const RATE_SLOPE_BPS_PER_MILLI_PPM: i64 = 50;

/// Maximum carbon tax rate: 25% (2500 bps).
const MAX_TAX_RATE_BPS: i64 = 2_500;

/// Rebate fraction: 80% of revenue returned to citizens.
const REBATE_FRACTION_BPS: i64 = 8_000;

/// Pre-industrial CO2 baseline in milli-ppm (280 ppm * 1000).
const BASELINE_CO2_MILLI_PPM: i64 = 280_000;

#[civlab_mod]
pub struct CarbonTaxMod {
    /// Accumulated rebate distributed this simulation run (for metrics).
    total_rebate_distributed_millijoules: i64,
}

impl PolicyMod for CarbonTaxMod {
    fn on_tick(&mut self, ctx: &PolicyContext) -> Vec<PolicyAction> {
        let mut actions = Vec::new();

        // Compute dynamic carbon tax rate based on current CO2.
        let co2 = ctx.climate.co2_ppm_milliunits;
        let excess_co2 = (co2 - BASELINE_CO2_MILLI_PPM).max(0);

        // Apply slope and clamp to max rate.
        let rate_bps = fp_clamp(
            excess_co2 * RATE_SLOPE_BPS_PER_MILLI_PPM / 1_000,
            0,
            MAX_TAX_RATE_BPS,
        );

        // Set carbon tax on fossil fuels.
        actions.push(PolicyAction::SetTaxRate {
            good: GoodType::FossilFuel,
            rate_bps,
        });

        // Record current rate as a policy parameter for metrics.
        actions.push(PolicyAction::SetPolicyParam {
            key: "carbon_tax_rate_bps".into(),
            value: rate_bps,
        });

        // Distribute rebate: 80% of prior-tick carbon tax revenue to citizens.
        let prior_revenue = ctx.economy.tax_revenue_millijoules;
        if prior_revenue > 0 {
            let rebate = apply_rate_bps(prior_revenue, REBATE_FRACTION_BPS);
            self.total_rebate_distributed_millijoules += rebate;

            actions.push(PolicyAction::TransferFunds {
                from: ActorId::Treasury(ctx.nation.id),
                to: ActorId::CitizensFund(ctx.nation.id),
                amount_millijoules: rebate,
            });

            actions.push(PolicyAction::SetPolicyParam {
                key: "total_rebate_distributed".into(),
                value: self.total_rebate_distributed_millijoules,
            });
        }

        actions
    }

    fn on_event(&mut self, event: &SimEvent) -> Vec<PolicyAction> {
        match event {
            SimEvent::Climate(ClimateEvent::Co2Spike { .. }) => {
                // Emergency rate on CO2 spikes: force maximum rate.
                vec![PolicyAction::SetTaxRate {
                    good: GoodType::FossilFuel,
                    rate_bps: MAX_TAX_RATE_BPS,
                }]
            }
            _ => vec![],
        }
    }

    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "custom-carbon-tax".into(),
            name: "Carbon Tax Policy".into(),
            version: "1.0.0".into(),
            subscribed_events: vec!["climate.co2_spike".into()],
            run_during_fast_forward: true,
        }
    }
}
```

### 13.2 Example 2: Custom Market Allocator (EconomicMod)

**File:** `examples/mods/planned-allocator/src/lib.rs`

```rust
//! Planned Economy Allocator
//!
//! Replaces the default price-auction market clearing with a priority-queue
//! allocator that allocates scarce goods by declared social priority rather
//! than willingness-to-pay. Priority order: Essential > Industrial > Luxury.
//! Within each priority class, allocation is proportional to quantity requested.
//!
//! Used for research comparing planned allocation outcomes against price-based
//! clearing under identical scarcity conditions.

use civlab_sdk::prelude::*;

#[civlab_mod]
pub struct PlannedAllocatorMod;

impl EconomicMod for PlannedAllocatorMod {
    fn on_production(&self, ctx: &ProductionContext) -> ProductionResult {
        // Planned allocator does not override production.
        ProductionResult {
            outputs: ctx.sector.default_outputs.clone(),
            inputs_consumed: ctx.inputs.clone(),
            waste: BTreeMap::new(),
            co2_emissions_mg: ctx.sector.default_co2_emissions_mg,
        }
    }

    fn on_market_clear(&self, ctx: &MarketContext) -> MarketResult {
        let mut allocations = Vec::new();
        let mut remaining_supply = ctx.supply;

        // Group bids by priority level (high priority = lower u8 value = allocated first).
        let mut priority_groups: BTreeMap<u8, Vec<&MarketBid>> = BTreeMap::new();
        for bid in &ctx.bids {
            priority_groups
                .entry(bid.priority.as_u8())
                .or_default()
                .push(bid);
        }

        // Allocate from highest to lowest priority.
        for (_level, group) in priority_groups.iter() {
            if remaining_supply == 0 {
                break;
            }
            let total_group_demand: i64 = group.iter().map(|b| b.quantity).sum();
            let group_supply = remaining_supply.min(total_group_demand);

            for bid in group {
                if remaining_supply == 0 { break; }
                // Proportional within priority class.
                let share = fp_div(fp_mul(group_supply, bid.quantity), total_group_demand);
                let allocated = share.min(remaining_supply);
                remaining_supply -= allocated;
                allocations.push(MarketAllocation {
                    actor: bid.actor,
                    quantity: allocated,
                    // Administered price: 90% of last clearing price.
                    price_millijoules: apply_rate_bps(ctx.last_price_millijoules, 9_000),
                });
            }
        }

        let total_demanded: i64 = ctx.bids.iter().map(|b| b.quantity).sum();
        let total_allocated: i64 = allocations.iter().map(|a| a.quantity).sum();

        MarketResult {
            allocations,
            clearing_price_millijoules: apply_rate_bps(ctx.last_price_millijoules, 9_000),
            unmet_demand: total_demanded - total_allocated,
        }
    }

    fn on_consumption(&self, ctx: &ConsumptionContext) -> ConsumptionResult {
        ConsumptionResult::default_from(ctx)
    }

    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "planned-allocator".into(),
            name: "Planned Economy Market Allocator".into(),
            version: "1.0.0".into(),
            subscribed_events: vec![],
            run_during_fast_forward: true,
        }
    }

    fn handled_production_sectors(&self) -> Vec<SectorId> { vec![] }
    fn handled_market_goods(&self) -> Vec<GoodType> { GoodType::all_builtin() }
    fn custom_good_types(&self) -> Vec<CustomGoodDescriptor> { vec![] }
}
```

### 13.3 Example 3: Solar Energy Production Formula (EconomicMod)

**File:** `examples/mods/solar-energy/src/lib.rs`

```rust
//! Solar PV Production Formula
//!
//! Overrides the flat-rate production formula for the SolarPV sector with
//! a capacity-factor model accounting for season, climate, and weather variance:
//!
//!   output_joules = installed_capacity_wp
//!                   * capacity_factor_bps / 10_000
//!                   * hours_per_tick
//!                   * joules_per_watt_hour
//!
//! Enables research on renewable intermittency effects under different policies.

use civlab_sdk::prelude::*;

const TICKS_PER_YEAR: u64 = 365;
const BASE_CAPACITY_FACTOR_BPS: i64 = 2_500;  // 25% at equator, clear sky, summer peak
const JOULES_PER_WATT_HOUR: i64 = 3_600;
const HOURS_PER_TICK: i64 = 24;               // 1 tick = 1 simulated day

#[civlab_mod]
pub struct SolarEnergyMod;

impl EconomicMod for SolarEnergyMod {
    fn on_production(&self, ctx: &ProductionContext) -> ProductionResult {
        let mut rng = ctx.rng;

        // Season factor: sinusoidal model using integer cosine approximation.
        let day_of_year = (ctx.tick % TICKS_PER_YEAR) as i64;
        let season_bps = season_factor_bps(day_of_year);

        // Climate factor: +0.5% per degree of warming.
        let temp_k = ctx.climate.global_temp_delta_millk / 1_000;
        let climate_bps = 10_000 + temp_k * 50;

        // Weather factor: uniform +/-20% random variation.
        let weather_bps = 10_000 + rng.next_range(-2_000, 2_000);

        // Composite capacity factor.
        let cf_bps = apply_rate_bps(
            apply_rate_bps(
                apply_rate_bps(BASE_CAPACITY_FACTOR_BPS, season_bps),
                climate_bps,
            ),
            weather_bps,
        );

        let installed_wp = ctx.sector.installed_capacity_units;
        let output_wh = apply_rate_bps(installed_wp * HOURS_PER_TICK, cf_bps);
        let output_joules = output_wh * JOULES_PER_WATT_HOUR;

        let mut outputs = BTreeMap::new();
        outputs.insert(GoodType::Electricity, output_joules);

        let mut inputs_consumed = BTreeMap::new();
        inputs_consumed.insert(GoodType::Sunlight, installed_wp);

        ProductionResult {
            outputs,
            inputs_consumed,
            waste: BTreeMap::new(),
            co2_emissions_mg: 0,
        }
    }

    fn on_market_clear(&self, ctx: &MarketContext) -> MarketResult {
        MarketResult::default_pass_through(ctx)
    }

    fn on_consumption(&self, ctx: &ConsumptionContext) -> ConsumptionResult {
        ConsumptionResult::default_from(ctx)
    }

    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "solar-energy".into(),
            name: "Solar PV Production Formula".into(),
            version: "1.0.0".into(),
            subscribed_events: vec![],
            run_during_fast_forward: true,
        }
    }

    fn handled_production_sectors(&self) -> Vec<SectorId> { vec![SectorId::SolarPV] }
    fn handled_market_goods(&self) -> Vec<GoodType> { vec![] }
    fn custom_good_types(&self) -> Vec<CustomGoodDescriptor> {
        vec![CustomGoodDescriptor {
            id: "sunlight".into(),
            name: "Solar Irradiance".into(),
            joule_equivalent: None,
            carbon_tracked: false,
        }]
    }
}

/// Integer cosine approximation returning [-10_000, 10_000].
/// No f64 used; lookup table derived from standard cos values.
fn integer_cos_bps(deg: i64) -> i64 {
    const TABLE: [i16; 91] = [
        10000, 9998, 9994, 9986, 9976, 9962, 9945, 9925, 9903, 9877,
        9848, 9816, 9781, 9744, 9703, 9659, 9613, 9563, 9511, 9455,
        9397, 9336, 9272, 9205, 9135, 9063, 8988, 8910, 8829, 8746,
        8660, 8572, 8480, 8387, 8290, 8192, 8090, 7986, 7880, 7771,
        7660, 7547, 7431, 7314, 7193, 7071, 6947, 6820, 6691, 6561,
        6428, 6293, 6157, 6018, 5878, 5736, 5592, 5446, 5299, 5150,
        5000, 4848, 4695, 4540, 4384, 4226, 4067, 3907, 3746, 3584,
        3420, 3256, 3090, 2924, 2756, 2588, 2419, 2250, 2079, 1908,
        1736, 1564, 1392, 1219, 1045,  872,  698,  523,  349,  175, 0,
    ];
    let d = (deg.abs() % 360) as usize;
    let (idx, sign): (usize, i64) = match d {
        0..=90   => (d,       1),
        91..=180 => (180 - d, -1),
        181..=270 => (d - 180, -1),
        _         => (360 - d, 1),
    };
    TABLE[idx] as i64 * sign
}

fn season_factor_bps(day_of_year: i64) -> i64 {
    // Peak summer (day 182) = 10_000 bps (1.0x); peak winter = 5_000 bps (0.5x).
    let angle = ((day_of_year - 182).abs() * 360) / 365;
    let cos = integer_cos_bps(angle);  // [-10_000, 10_000]
    7_500 + cos / 2  // Maps to [5_000, 10_000]
}
```

### 13.4 Example 4: Custom Victory Condition — 90% Renewable Energy (ScenarioMod)

**File:** `examples/mods/renewable-victory/src/lib.rs`

```rust
//! Renewable Transition Race — ScenarioMod
//!
//! Defines a two-nation scenario with a custom victory condition:
//! first nation to sustain >= 90% renewable electricity for 10
//! consecutive ticks wins.
//!
//! Starting conditions are calibrated for 2030 baseline:
//! - CO2: 420 ppm
//! - Global temperature delta: +1.2 C
//! - Nation 1 (Industria): 25% renewable baseline, market democracy
//! - Nation 2 (Solaria): 50% renewable baseline, green technocracy

use civlab_sdk::prelude::*;

const VICTORY_THRESHOLD_BPS: i64 = 9_000;    // 90%
const CONSECUTIVE_REQUIRED: u32  = 10;

#[civlab_mod]
pub struct RenewableVictoryMod;

impl ScenarioMod for RenewableVictoryMod {
    fn build_scenario(&self) -> ScenarioDescriptor {
        ScenarioDescriptor {
            name: "Renewable Transition Race".into(),
            description: "First nation to sustain 90% renewable electricity \
                           for 10 consecutive ticks wins. 2030 baseline.".into(),
            nations: vec![
                NationDescriptor {
                    id: NationId(1),
                    name: "Industria".into(),
                    government_type: GovernmentType::MarketDemocracy,
                    initial_population: 80_000_000,
                    initial_cities: vec![
                        CityDescriptor {
                            name: "Capital".into(),
                            population: 5_000_000,
                            hex: HexCoord { q: 10, r: 10 },
                        },
                    ],
                    initial_treasury_millijoules: 500_000_000_000,
                    initial_policy_params: {
                        let mut p = BTreeMap::new();
                        p.insert("renewable_share_bps".into(), 2_500);
                        p.insert("consecutive_renewable_ticks".into(), 0);
                        p
                    },
                    territory_hexes: hex_region(HexCoord { q: 8, r: 8 }, 8),
                    ai_controller: AiController::Native { difficulty: 3 },
                },
                NationDescriptor {
                    id: NationId(2),
                    name: "Solaria".into(),
                    government_type: GovernmentType::GreenTechnocracy,
                    initial_population: 40_000_000,
                    initial_cities: vec![
                        CityDescriptor {
                            name: "Sunport".into(),
                            population: 3_000_000,
                            hex: HexCoord { q: 30, r: 10 },
                        },
                    ],
                    initial_treasury_millijoules: 300_000_000_000,
                    initial_policy_params: {
                        let mut p = BTreeMap::new();
                        p.insert("renewable_share_bps".into(), 5_000);
                        p.insert("consecutive_renewable_ticks".into(), 0);
                        p
                    },
                    territory_hexes: hex_region(HexCoord { q: 28, r: 8 }, 6),
                    ai_controller: AiController::Native { difficulty: 3 },
                },
            ],
            climate_initial: ClimateInitial {
                co2_ppm_milliunits: 420_000,
                global_temp_delta_millk: 1_200,
                sea_level_mm: 200,
            },
            economy_initial: EconomyInitial::default(),
            event_probability_modifiers: {
                let mut m = BTreeMap::new();
                m.insert("climate.heatwave".into(), 15_000);  // 1.5x
                m
            },
            custom_victory_condition: true,
            custom_map_generation: false,
            max_ticks: Some(3_650),
        }
    }

    fn check_victory(&self, world: &WorldView) -> Option<VictoryResult> {
        // Read consecutive tick counters persisted via policy_params.
        for nation in world.nations() {
            let consecutive = nation
                .policy_params
                .get("consecutive_renewable_ticks")
                .copied()
                .unwrap_or(0);
            if consecutive >= CONSECUTIVE_REQUIRED as i64 {
                return Some(VictoryResult {
                    winner: Some(nation.id),
                    condition_description: format!(
                        "{} sustained {}%+ renewable energy for {} consecutive ticks.",
                        nation.name,
                        VICTORY_THRESHOLD_BPS / 100,
                        CONSECUTIVE_REQUIRED,
                    ),
                    final_metrics: {
                        let mut m = BTreeMap::new();
                        m.insert("winning_tick".into(), world.tick as i64);
                        m.insert("winning_renewable_bps".into(),
                            nation.policy_params.get("renewable_share_bps").copied().unwrap_or(0));
                        m
                    },
                });
            }
        }
        None
    }

    fn generate_map(&self, _seed: u64) -> MapDescriptor {
        MapDescriptor::default()
    }

    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "renewable-victory".into(),
            name: "Renewable Transition Race".into(),
            version: "1.0.0".into(),
            subscribed_events: vec![],
            run_during_fast_forward: true,
        }
    }
}

/// Generate a hex disc of given radius around center.
fn hex_region(center: HexCoord, radius: i32) -> Vec<HexCoord> {
    let mut hexes = Vec::new();
    for q in -radius..=radius {
        let r_lo = (-radius).max(-q - radius);
        let r_hi = radius.min(-q + radius);
        for r in r_lo..=r_hi {
            hexes.push(HexCoord { q: center.q + q, r: center.r + r });
        }
    }
    hexes
}
```

The victory condition's consecutive tick tracking requires a companion `PolicyMod` loaded alongside this `ScenarioMod` that reads each nation's `renewable_share_bps` parameter each tick and increments or resets `consecutive_renewable_ticks` via `SetPolicyParam` actions. The full companion mod is in `examples/mods/renewable-victory/companion-policy/`.

---

## 14. Security and Isolation

### 14.1 Memory Isolation

wasmtime's WASM linear memory model provides hard memory isolation between mod instances and between each mod and the host:

- Each mod instance has its own `Memory` object (a contiguous byte region in the host process address space).
- WASM memory accesses are bounds-checked by wasmtime (or, for AOT-compiled WASM, by hardware trap via guard pages). An out-of-bounds memory access in WASM is caught as a `Trap::MemoryOutOfBounds` and surfaced to the host, never reaching arbitrary host memory.
- Mods cannot address each other's memories. There is no shared memory between mod instances (WASM shared memory is disabled in the engine's `wasmtime::Config`).
- The host's Rust data structures are entirely outside WASM linear memory. The only bridge is the host functions exported via `Linker`, which the host controls completely.

### 14.2 No Syscalls

The WASM sandbox provides zero syscall access:

- The host's `Linker` satisfies only the `civlab` import module. Any import from `wasi_snapshot_preview1`, `wasi_preview2`, `env`, or any other module causes `ModLoadError::UnsatisfiedImport` at instantiation.
- WASI is explicitly disabled in the wasmtime `Store` configuration (`wasi: false`).
- There is no mechanism for a mod to escalate from WASM computation to host filesystem, network, or process control. The attack surface is limited to the civlab host functions defined in Section 3.4.

### 14.3 CPU Budget Enforcement

Epoch interruption is the primary mechanism for CPU budget enforcement. Implementation details:

```rust
// In the host engine (sim/src/mod_host.rs):

// Engine configured with epoch interruption enabled.
let mut engine_config = wasmtime::Config::new();
engine_config.epoch_interruption(true);
let engine = wasmtime::Engine::new(&engine_config)?;

// Background thread increments epoch every 10 µs.
let epoch_engine = engine.clone();
std::thread::spawn(move || {
    loop {
        std::thread::sleep(std::time::Duration::from_micros(10));
        epoch_engine.increment_epoch();
    }
});

// Per-mod store configured with deadline = 5 epochs = 50 µs.
let mut store = wasmtime::Store::new(&engine, mod_capability_set);
store.set_epoch_deadline(5);
store.epoch_deadline_trap();   // Raises Trap::Interrupt on deadline exceeded

// Before each mod callback:
store.set_epoch_deadline(5);   // Reset to fresh 50 µs budget
let result = mod_on_tick.call(&mut store, (ctx_ptr, ctx_len));

match result {
    Ok(out_len) => { /* process actions */ }
    Err(e) if e.is::<wasmtime::Trap>() => {
        match e.downcast::<wasmtime::Trap>()? {
            Trap::Interrupt => {
                log_mod_timeout(mod_id, tick);
                // Discard actions; continue tick with other mods
            }
            other => {
                log_mod_trap(mod_id, tick, other);
                increment_fault_counter(mod_id);
            }
        }
    }
    Err(e) => { return Err(e.into()); }
}
```

### 14.4 Memory Budget Enforcement

```rust
// wasmtime StoreLimitsBuilder enforces memory ceiling.
use wasmtime::ResourceLimiter;

struct ModStoreLimits {
    memory_limit_bytes: usize,
    table_limit_elements: u32,
}

impl ResourceLimiter for ModStoreLimits {
    fn memory_growing(&mut self, _current: usize, desired: usize, _max: Option<usize>)
        -> Result<bool>
    {
        // Allow growth up to the configured limit.
        Ok(desired <= self.memory_limit_bytes)
    }

    fn table_growing(&mut self, _current: u32, desired: u32, _max: Option<u32>)
        -> Result<bool>
    {
        Ok(desired <= self.table_limit_elements)
    }
}

// Applied to each mod store:
store.limiter(|state| &mut state.resource_limiter);
// Where state.resource_limiter = ModStoreLimits {
//     memory_limit_bytes: 64 * 1024 * 1024,  // 64 MB
//     table_limit_elements: 8_192,
// };
```

### 14.5 Determinism Enforcement

The determinism scan (Section 3.5) is implemented as a post-validation pass over the WASM binary's code section. The scanner uses the `wasmparser` crate to iterate instructions and applies the following rules:

| Check | Implementation |
|---|---|
| Float contamination of actions | Data-flow backward trace from all `action_emit` call sites; any `f32`/`f64` value reaching an argument is flagged |
| Non-deterministic float ops | Instruction opcode allowlist: `f32.nearest`, `f64.nearest`, `f32.sqrt`, `f64.sqrt` are denied |
| Platform-undefined operations | `i32.clz`, `i64.clz` on zero input are flagged (behavior defined in WASM spec but historically varied; conservatively flagged) |
| Atomic memory ops | All `memory.atomic.*` and `memory.fence` instructions are denied |
| Import validation | All imports cross-referenced against permitted function set; any unknown import denied |

The scan runs synchronously before instantiation and adds approximately 2-5 ms to the load time for a typical mod binary.

### 14.6 Action Validation

Before applying mod actions to world state, the host validates each action:

| Action Type | Validation |
|---|---|
| `SetTaxRate` | `good` must be a registered `GoodType`; `rate_bps` in `[0, 10_000]` (clamped with warning at `>5_000`) |
| `SetSubsidyRate` | `good` must be registered; `rate_bps` in `[0, 5_000]` (hard cap; values above rejected) |
| `TransferFunds` | `from` actor must exist and have `amount` available; `to` actor must exist; `amount > 0`; conservation check: sum of all transfers must not exceed treasury balance |
| `TriggerEvent` | `event_type` must be registered in mod metadata or be a builtin; `payload` size limit 4 KB |
| `SetPolicyParam` | `key` must be alphanumeric+underscore, max 64 chars; `value` any i64 |
| `SetInterestRate` | `rate_bps` in `[0, 3_000]` |

Rejected actions are logged as `ModActionRejected { mod_id, tick, action_type, reason }` and do not cause the mod to fault. Mods are expected to emit only valid actions, but rejection is not a fault condition since the host's world state may have changed since the context was serialized.

### 14.7 Mod Audit Log

All mod activity is recorded in the mod audit log, a separate append-only log distinct from the main simulation event log:

```
mod-audit.log format (NDJSON):
{"ts":"2026-02-21T10:00:00Z","event":"ModLoaded","mod_id":"carbon-tax","tick":0}
{"ts":"...","event":"ModTimeout","mod_id":"slow-mod","tick":42,"callback":"on_tick"}
{"ts":"...","event":"ModActionRejected","mod_id":"carbon-tax","tick":50,
 "action":"TransferFunds","reason":"InsufficientBalance"}
{"ts":"...","event":"ModPermissionViolation","mod_id":"rogue-mod","tick":100,
 "call":"world_read","domain":"military"}
{"ts":"...","event":"ModFaulted","mod_id":"bad-mod","tick":200,"fault_count":3}
{"ts":"...","event":"ModUnloaded","mod_id":"carbon-tax","tick":500}
```

The audit log is written by the host, not by mods. Mods cannot influence what appears in the audit log. The audit log is used for:
- Post-hoc debugging of mod behavior
- Research reproducibility (audit log is included in `.civreplay` archives)
- Security incident investigation

---

## 15. Functional Requirements

The following functional requirements (FRs) govern the implementation of the modding API. Each FR is expressed as a SHALL statement and includes a test reference.

### FR-CIV-MOD-001: WASM Memory Isolation

**SHALL** ensure that a mod's WASM instance cannot read or write host memory or another mod instance's memory.

**Test:** `tests/mod_security/test_memory_isolation.rs` — loads two mod instances; verifies that writes in mod A's linear memory are not visible in mod B's linear memory or host Rust data structures.

**Acceptance:** Test passes with zero cross-memory access detected across 1000 simulated tick iterations.

### FR-CIV-MOD-002: CPU Budget Enforcement

**SHALL** terminate any mod callback that exceeds 50 µs wall-clock time via epoch interruption, log a `ModTimeout` event, and discard the timed-out callback's actions without faulting the mod.

**Test:** `tests/mod_security/test_cpu_budget.rs` — loads a deliberately infinite-loop mod; verifies that the simulation tick completes within 100 µs of the epoch deadline, that `ModTimeout` is logged, and that the mod remains in `Active` status.

**Acceptance:** Infinite-loop mod does not prevent tick completion within 2× the timeout budget.

### FR-CIV-MOD-003: API Version Compatibility Enforcement

**SHALL** reject at load time any mod whose declared `api_version` does not fall within the host's accepted version range `[current - 1, current]`, returning `ModLoadError::IncompatibleApiVersion`.

**Test:** `tests/mod_load/test_api_version.rs` — attempts to load mods declaring api_version 0 (accepted during transition), 1 (current; accepted), and 2 (future; rejected on a v1 host).

**Acceptance:** Version 0 and 1 load successfully; version 2 returns `IncompatibleApiVersion`.

### FR-CIV-MOD-004: Mod Determinism Invariant

**SHALL** produce identical simulation state at every tick boundary when the same mod is loaded in a simulation with the same seed and input event sequence, on any supported host platform.

**Test:** `tests/mod_determinism/test_cross_platform_replay.rs` — runs a 1000-tick simulation with the carbon tax mod on two separate wasmtime engine configurations (one AOT, one interpreter); compares state hashes at every tick boundary.

**Acceptance:** State hashes match at all 1000 tick boundaries across both configurations.

### FR-CIV-MOD-005: Non-Deterministic Instruction Rejection

**SHALL** reject at load time any WASM binary containing instructions classified as non-deterministic per Section 3.5, returning `ModLoadError::NonDeterministicInstruction`.

**Test:** `tests/mod_load/test_nondeterministic_rejection.rs` — builds test WASM binaries containing each rejected instruction type; verifies each is rejected with the correct error.

**Acceptance:** All 6 non-deterministic instruction categories are detected and rejected.

### FR-CIV-MOD-006: Permission Enforcement

**SHALL** enforce read and write permissions declared in the mod manifest at runtime, returning error code `ERR_PERMISSION_DENIED` from `world_read()` or `action_emit()` for undeclared domains/action types, and logging a `ModPermissionViolation` event.

**Test:** `tests/mod_security/test_permission_enforcement.rs` — loads a mod that attempts to read the military domain without declaring `read_military = true`; verifies the call is rejected and the violation is logged.

**Acceptance:** Undeclared domain access returns ERR_PERMISSION_DENIED; violation is present in audit log.

### FR-CIV-MOD-007: Mod Fault Isolation

**SHALL** isolate mod faults (panics, traps, repeated timeouts) such that a faulting mod does not affect the simulation state, other mod instances, or simulation tick completion.

**Test:** `tests/mod_security/test_fault_isolation.rs` — loads a panic-on-tick mod alongside a well-behaved mod; verifies that the panicking mod reaches `Faulted` status after 3 consecutive faults, the well-behaved mod continues to operate, and simulation state remains consistent.

**Acceptance:** Panic mod is faulted in <= 3 ticks; simulation continues with correct state from the well-behaved mod.

### FR-CIV-MOD-008: Signature Verification

**SHALL** verify the Ed25519 signature of the mod WASM binary before instantiation in non-development builds, rejecting unsigned or tampered mods with `ModLoadError::InvalidSignature`.

**Test:** `tests/mod_load/test_signature_verification.rs` — attempts to load a mod with a valid signature, a mod with a tampered WASM binary (signature mismatch), and an unsigned mod; verifies only the validly signed mod loads.

**Acceptance:** Tampered and unsigned mods return `InvalidSignature`; validly signed mod loads successfully.

### FR-CIV-MOD-009: Scenario Registration and Init

**SHALL** allow a `ScenarioMod` to register a complete `ScenarioDescriptor` that the host uses to construct the initial `WorldState`, including all nations, cities, climate parameters, and economic parameters.

**Test:** `tests/mod_integration/test_scenario_init.rs` — loads the renewable-victory ScenarioMod; verifies that the resulting WorldState contains exactly 2 nations with the correct starting populations, treasury values, and climate parameters.

**Acceptance:** WorldState at Tick 0 matches ScenarioDescriptor fields exactly.

### FR-CIV-MOD-010: Action Validation and Conservation

**SHALL** validate all mod-emitted actions against the conservation invariants defined in CIV-0100 §Conservation, rejecting actions that would violate double-entry balance, and logging a `ModActionRejected` event for each rejected action.

**Test:** `tests/mod_integration/test_action_conservation.rs` — loads a mod that emits a `TransferFunds` action with `amount` exceeding the treasury balance; verifies the action is rejected, the treasury balance is unchanged, and a rejection event is logged.

**Acceptance:** Over-balance transfer is rejected; conservation invariant holds after tick.

### FR-CIV-MOD-011: Custom Good Type Registration

**SHALL** allow an `EconomicMod` to register custom `GoodType` definitions via `custom_good_types()` during `mod_init()`, making the new types available in subsequent tick contexts and market state.

**Test:** `tests/mod_integration/test_custom_good_registration.rs` — loads the solar energy mod; verifies that `GoodType::Custom("sunlight")` is present in the `EconomyView` returned to subsequent mods after init.

**Acceptance:** Custom good type is present and queryable from EconomyView on Tick 1.

### FR-CIV-MOD-012: Mid-Simulation Mod Swap

**SHALL** allow swapping a loaded mod with a new mod binary at a tick boundary via `sim.mod.swap()`, with the new mod receiving its first tick callback in the same tick the swap completes, and the old mod receiving no further callbacks.

**Test:** `tests/mod_integration/test_mod_swap.rs` — runs 100 ticks with mod-v1; swaps to mod-v2 at tick 100; verifies mod-v2 receives its first callback on tick 100 and mod-v1 receives no callbacks after tick 99.

**Acceptance:** Swap is atomic at tick boundary; no callback overlap.

### FR-CIV-MOD-013: Mod Replay Inclusion

**SHALL** include all mod actions and mod events in the `.civreplay` event log, ensuring that a replay from the event log reproduces the identical simulation state at every tick boundary.

**Test:** `tests/mod_determinism/test_replay_with_mods.rs` — runs a 500-tick simulation with three mods loaded; saves the replay; replays from the event log; compares state hashes at every tick boundary.

**Acceptance:** State hashes match at all 500 tick boundaries between live run and replay.

### FR-CIV-MOD-014: Lua Script Parity

**SHALL** ensure that a Lua script implementing the same policy logic as a WASM PolicyMod produces numerically identical outcomes on every tick when given the same inputs, within the constraints of the Lua API subset.

**Test:** `tests/lua/test_lua_wasm_parity.rs` — runs a 100-tick simulation with the carbon-tax WASM mod; runs the same simulation with the carbon-tax Lua script; compares tax rate and rebate transfer amounts at every tick.

**Acceptance:** Tax rates and rebate amounts are identical at all 100 ticks.

### FR-CIV-MOD-015: Mod Status Telemetry

**SHALL** expose each loaded mod's current status (`Active`, `Degraded`, `Faulted`, `Unloading`), fault count, timeout count, and last action count via the simulation metrics endpoint, accessible to the client via the protocol defined in CIV-0200.

**Test:** `tests/mod_integration/test_mod_telemetry.rs` — loads a mod that times out on odd ticks; queries the metrics endpoint after 10 ticks; verifies timeout count = 5, status = `Degraded`, fault count = 0.

**Acceptance:** Telemetry matches observed mod behavior exactly.

---

## 16. Testing and CI

### 16.1 SDK Unit Tests — Mock Host

The `civlab-sdk` crate's `dev-mock` feature provides a complete mock implementation of all host functions, enabling mod unit tests to run natively (not in WASM) with `cargo test`:

```rust
// In mod unit tests (native target, not wasm32):
#[cfg(test)]
mod tests {
    use civlab_sdk::dev_mock::*;  // Requires dev-mock feature

    #[test]
    fn carbon_tax_rate_scales_with_co2() {
        let mut mod_instance = CarbonTaxMod {
            total_rebate_distributed_millijoules: 0,
        };

        // Build a mock PolicyContext with high CO2.
        let ctx = MockPolicyContextBuilder::new()
            .tick(100)
            .climate(MockClimateView {
                co2_ppm_milliunits: 420_000,   // 420 ppm
                renewable_share_bps: 3_000,
                ..Default::default()
            })
            .economy(MockEconomyView {
                tax_revenue_millijoules: 1_000_000,
                ..Default::default()
            })
            .build();

        let actions = mod_instance.on_tick(&ctx);

        // With 420 ppm CO2: excess = 140 ppm; rate = 140 * 50 / 1000 = 7 bps.
        // Wait: excess_co2 = (420_000 - 280_000) = 140_000 milli-ppm
        //        rate = 140_000 * 50 / 1_000 = 7_000 bps => clamped to 2_500
        let set_tax = actions.iter().find(|a| matches!(a,
            PolicyAction::SetTaxRate { good: GoodType::FossilFuel, .. }
        ));
        assert!(set_tax.is_some());
        if let Some(PolicyAction::SetTaxRate { rate_bps, .. }) = set_tax {
            assert_eq!(*rate_bps, 2_500, "Rate should be clamped to MAX_TAX_RATE_BPS");
        }
    }

    #[test]
    fn carbon_tax_distributes_rebate() {
        let mut m = CarbonTaxMod { total_rebate_distributed_millijoules: 0 };
        let ctx = MockPolicyContextBuilder::new()
            .economy(MockEconomyView {
                tax_revenue_millijoules: 10_000_000,  // 10M millijoules revenue
                ..Default::default()
            })
            .build();

        let actions = m.on_tick(&ctx);

        let transfer = actions.iter().find(|a| matches!(a, PolicyAction::TransferFunds { .. }));
        assert!(transfer.is_some(), "Expected a TransferFunds action");
        if let Some(PolicyAction::TransferFunds { amount_millijoules, .. }) = transfer {
            // 80% of 10_000_000 = 8_000_000
            assert_eq!(*amount_millijoules, 8_000_000);
        }
    }

    #[test]
    fn zero_revenue_produces_no_transfer() {
        let mut m = CarbonTaxMod { total_rebate_distributed_millijoules: 0 };
        let ctx = MockPolicyContextBuilder::new()
            .economy(MockEconomyView {
                tax_revenue_millijoules: 0,
                ..Default::default()
            })
            .build();

        let actions = m.on_tick(&ctx);
        let transfer = actions.iter().find(|a| matches!(a, PolicyAction::TransferFunds { .. }));
        assert!(transfer.is_none(), "No transfer expected with zero revenue");
    }
}
```

The mock host records all `action_emit()` calls, provides configurable world state, and implements deterministic RNG seeded from a fixed test seed.

### 16.2 Integration Tests — Load into Sim

Integration tests load compiled WASM mod binaries into a real (non-mocked) simulation instance:

```rust
// tests/mod_integration/test_carbon_tax_integration.rs

#[tokio::test]
async fn carbon_tax_actions_applied_correctly() {
    // Build the mod binary (pre-built by CI; loaded from test fixtures).
    let wasm_bytes = include_bytes!("../../fixtures/carbon-tax.wasm");

    // Start a headless simulation with a deterministic seed.
    let mut sim = Simulation::new(SimConfig {
        seed: 0xDEADBEEF,
        scenario: ScenarioDescriptor::test_minimal(),
        ..Default::default()
    });

    // Load the mod.
    sim.mod_host().load("carbon-tax", wasm_bytes).await
        .expect("mod should load successfully");

    // Run 10 ticks.
    for tick in 0..10 {
        sim.step().await.expect("tick should succeed");

        let state = sim.current_state();
        let fossil_tax = state.economy
            .goods[&GoodType::FossilFuel]
            .tax_rate_bps;

        // At tick 0 with test_minimal scenario CO2 = 350 ppm:
        // excess = 70 ppm; rate = 70_000 * 50 / 1_000 = 3_500 bps => clamped to 2_500
        assert_eq!(fossil_tax, 2_500,
            "Carbon tax rate should be clamped at MAX on tick {}", tick);
    }

    // Verify mod is still Active (no faults).
    let mod_status = sim.mod_host().status("carbon-tax");
    assert_eq!(mod_status, ModStatus::Active);
}
```

### 16.3 Fuzz Targets

Three `cargo-fuzz` fuzz targets are maintained:

**Target 1: WASM binary parsing**

```rust
// fuzz/fuzz_targets/fuzz_wasm_load.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to load arbitrary bytes as a WASM mod.
    // Must not panic, crash, or hang. Should return a ModLoadError on invalid input.
    let _ = civlab_sim::mod_host::validate_wasm_binary(data);
});
```

**Target 2: Action deserialization**

```rust
// fuzz/fuzz_targets/fuzz_action_deserialize.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use civlab_sdk::PolicyAction;

fuzz_target!(|data: &[u8]| {
    // Attempt to deserialize arbitrary bytes as a Vec<PolicyAction>.
    // Must not panic. Invalid postcard encoding should return Err.
    let _ = postcard::from_bytes::<Vec<PolicyAction>>(data);
});
```

**Target 3: Manifest parsing**

```rust
// fuzz/fuzz_targets/fuzz_manifest_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = civlab_sim::mod_host::parse_manifest(s);
    }
});
```

Fuzz targets run in CI for 60 seconds per target per PR using `cargo fuzz run --jobs 4 -- -max_total_time=60`.

### 16.4 Example Mods in CI

All four example mods from Section 13 are built and run in CI on every PR. The CI pipeline:

```yaml
# .github/workflows/mod-ci.yml (excerpt)

jobs:
  build-example-mods:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - name: Install wasm-opt
        run: cargo install wasm-opt
      - name: Build example mods
        run: |
          for mod_dir in examples/mods/*/; do
            (cd "$mod_dir" && \
             cargo build --target wasm32-unknown-unknown --release && \
             wasm-opt -O3 \
               -o target/wasm32-unknown-unknown/release/mod_optimized.wasm \
               target/wasm32-unknown-unknown/release/*.wasm)
          done

  test-example-mods:
    needs: build-example-mods
    runs-on: ubuntu-latest
    steps:
      - name: Run mod integration tests
        run: cargo test --test mod_integration -- --test-threads=1
        # Sequential to avoid simulation state interference between tests.

  mod-determinism:
    needs: build-example-mods
    runs-on: ubuntu-latest
    steps:
      - name: Run determinism tests
        run: cargo test --test mod_determinism -- --test-threads=1

  mod-security:
    needs: build-example-mods
    runs-on: ubuntu-latest
    steps:
      - name: Run security tests
        run: cargo test --test mod_security

  fuzz-mods:
    runs-on: ubuntu-latest
    steps:
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Fuzz WASM loader
        run: cargo fuzz run fuzz_wasm_load -- -max_total_time=60
      - name: Fuzz action deserializer
        run: cargo fuzz run fuzz_action_deserialize -- -max_total_time=60
      - name: Fuzz manifest parser
        run: cargo fuzz run fuzz_manifest_parse -- -max_total_time=60
```

### 16.5 Test Coverage Targets

| Test Category | Target Coverage | Current Status |
|---|---|---|
| `civlab-sdk` unit tests (via mock host) | 100% line coverage | Required before v1.0 release |
| Mod loading pipeline (Steps 1-9) | 100% branch coverage | Required before v1.0 release |
| Action validation (all action types) | 100% path coverage | Required before v1.0 release |
| Security isolation (memory, CPU, syscall) | 100% scenario coverage | Required before v1.0 release |
| Example mod integration tests | Pass on every PR | Required before v1.0 release |
| Fuzz targets | No crashes in 60s/target | Required before v1.0 release |
| Determinism (cross-platform replay) | 100% tick match | Required before v1.0 release |

Coverage is measured using `cargo llvm-cov` and enforced by the CI quality gate.

### 16.6 Regression Baselines

The mod CI pipeline maintains regression baselines for performance:

| Metric | Baseline | Alert Threshold |
|---|---|---|
| WASM load time (carbon-tax mod, ~20 KB) | <= 8 ms | > 20 ms |
| Determinism scan time (carbon-tax mod) | <= 3 ms | > 10 ms |
| `on_tick` callback time (carbon-tax, mock world) | <= 5 µs | > 30 µs |
| Memory used per mod instance (carbon-tax) | <= 2 MB | > 16 MB |
| WASM binary size (carbon-tax, wasm-opt'd) | <= 50 KB | > 200 KB |

Performance regression alerts trigger a PR build failure and require explicit sign-off before merging.

---

## Appendix A: Error Type Reference

```rust
// civlab-sim/src/mod_host/errors.rs

#[derive(Debug, thiserror::Error)]
pub enum ModLoadError {
    #[error("manifest error: {0}")]
    Manifest(ManifestError),

    #[error("invalid WASM binary: {0}")]
    InvalidWasm(#[from] wasmtime::Error),

    #[error("non-deterministic instruction: {instr}")]
    NonDeterministicInstruction { instr: String },

    #[error("float contamination: f64 value reaches action_emit at {site}")]
    FloatContamination { site: String },

    #[error("unsatisfied WASM import: {module}::{name}")]
    UnsatisfiedImport { module: String, name: String },

    #[error("invalid signature: {reason}")]
    InvalidSignature { reason: String },

    #[error("mod_init returned error code: {code}")]
    InitFailed { code: i32 },

    #[error("WASM instantiation failed: {0}")]
    InstantiationFailed(wasmtime::Error),

    #[error("bundle too large: {size_bytes} bytes (limit: {limit_bytes})")]
    BundleTooLarge { size_bytes: usize, limit_bytes: usize },

    #[error("good type conflict: {good_id} already registered by {owner_mod_id}")]
    GoodTypeConflict { good_id: String, owner_mod_id: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("invalid mod id: {id:?}")]
    InvalidId { id: String },

    #[error("incompatible api_version: declared {declared}, host accepts [{min}, {max}]")]
    IncompatibleApiVersion { declared: u32, min: u32, max: u32 },

    #[error("unknown mod type: {mod_type:?}")]
    UnknownModType { mod_type: String },

    #[error("description too long: {len} bytes (limit: 256)")]
    DescriptionTooLong { len: usize },

    #[error("invalid dependency version range: {range:?}")]
    InvalidDependencyRange { range: String },

    #[error("unsatisfied dependency: {dep} {range} not satisfied by host {version}")]
    UnsatisfiedDependency { dep: String, range: String, version: String },

    #[error("permission {permission:?} is not valid for mod type {mod_type:?}")]
    PermissionExceedsModType { permission: String, mod_type: String },

    #[error("memory limit {requested_mb}MB exceeds host maximum {max_mb}MB")]
    MemoryLimitExceedsMax { requested_mb: u32, max_mb: u32 },

    #[error("cpu limit {requested_us}µs exceeds host maximum {max_us}µs")]
    CpuLimitExceedsMax { requested_us: u32, max_us: u32 },
}
```

---

## Appendix B: Host Function ABI Reference

Complete ABI for all host functions exported to WASM mods (module: `civlab`):

| Function | Signature | Return | Description |
|---|---|---|---|
| `log_i64` | `(msg_ptr: i32, msg_len: i32, value: i64)` | `void` | Append formatted log entry to mod log |
| `rng_next_u64` | `()` | `i64` | Next u64 from tick-scoped ChaCha20Rng (reinterpreted as i64) |
| `rng_next_range` | `(lo: i64, hi: i64)` | `i64` | Uniform i64 in [lo, hi) |
| `fixed_mul` | `(a: i64, b: i64, scale: i32)` | `i64` | `(a * b) >> scale` |
| `fixed_div` | `(a: i64, b: i64, scale: i32)` | `i64` | `(a << scale) / b` |
| `world_read` | `(query_ptr: i32, query_len: i32, out_ptr: i32, out_cap: i32)` | `i32` | Read world state; returns bytes written or negative error code |
| `action_emit` | `(action_ptr: i32, action_len: i32)` | `i32` | Emit serialized action; returns 0 on accept, negative error code on reject |
| `panic_abort` | `(msg_ptr: i32, msg_len: i32)` | `void` | Mod-initiated abort; host logs and faults mod |

**Error codes returned by `world_read` and `action_emit`:**

| Code | Constant | Meaning |
|---|---|---|
| `0` | `OK` | Success |
| `-1` | `ERR_INVALID_QUERY` | Malformed query or action payload |
| `-2` | `ERR_PERMISSION_DENIED` | Domain or action type not in mod capability set |
| `-3` | `ERR_BUFFER_TOO_SMALL` | `out_cap` insufficient for response; retry with larger buffer |
| `-4` | `ERR_ACTION_REJECTED` | Action failed validation (conservation, range, actor existence) |
| `-5` | `ERR_INTERNAL` | Host internal error; mod should call `panic_abort` |

---

## Appendix C: Glossary

| Term | Definition |
|---|---|
| `.civmod` | ZIP bundle format for mod distribution: manifest + WASM binary + assets |
| `api_version` | Integer major version of the CivLab mod API declared in mod manifest |
| `civlab-sdk` | Rust guest-side library for writing CivLab mods (targets `wasm32-unknown-unknown`) |
| Epoch interruption | wasmtime mechanism for terminating long-running WASM callbacks by incrementing a shared counter |
| `EconomicMod` | Mod type that overrides production formulas and/or market clearing algorithms |
| `EventMod` | Mod type that registers event triggers and handlers for Phase 4 processing |
| Fixed-point | Integer representation of fractional values using a fixed implicit decimal scale (1 unit = 1_000_000 micro-units) |
| `ModCapabilitySet` | Host-side data structure encoding the permissions declared in a mod's manifest |
| `ModRng` | SDK wrapper around the host's deterministic per-tick ChaCha20Rng interface |
| `#[civlab_mod]` | Rust proc macro that generates WASM ABI entry points for a mod struct |
| `PolicyMod` | Mod type that emits policy actions during Phase 3a (Policy Application) |
| `postcard` | Binary serialization format used for host-guest data exchange (compact, no_std) |
| `ScenarioMod` | Mod type that defines initial world state and custom victory conditions |
| WASM | WebAssembly — the bytecode format and execution model used for mod sandboxing |
| `wasmtime` | Rust WASM runtime library (version 26.x) used as the mod execution host |

---

*End of CIV-0700 — Modding and Plugin API Specification*

*Related specs: CIV-0001 (Core Simulation Loop), CIV-0100 (Economy Module), CIV-0103 (Institutions & Governance), CIV-0200 (Client Protocol)*
