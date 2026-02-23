# CIV-0103: Institutions, Time-Series, and Citizen Lifecycle v1

**Spec ID:** CIV-0103
**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team
**Related Specs:**
- CIV-0001: Core Simulation Loop — Deterministic Tick Architecture
- CIV-0107: Joule Economy System v1
- CIV-0100: Economy v1 (forthcoming)

---

## 1. Summary

This specification defines three tightly coupled subsystems of the CivLab simulation engine:

**Institutions** are governance actors with internal legitimacy dynamics, capture vulnerability, and policy output multipliers. They are not static parameters; they drift, get captured, reform, and collapse in response to systemic pressures including scarcity, corruption, citizen grievance, and coalition support. An institution in state `captured` produces fundamentally different policy outputs (enforcement intensity, compliance rates, policy capacity) than the same institution in state `stable` or `reforming`. Institutional dynamics feed directly into the tyranny index computed by the metrics engine.

**Citizen Lifecycle** is an energy-ledger state machine. Every citizen moves through lifecycle stages (`active`, `strained`, `dissenting`, `migrating`, `retired`) driven by transitions that depend on their joule balance, welfare access, coercion index, and mobility constraints. The retirement pool is a deterministic threshold on cumulative lifetime joule credit, not an age or wealth target. Citizens in `retired` status draw from the public retirement reserve at a per-tick pension rate. Citizens in `dissenting` or `migrating` states exert direct upward pressure on institutional instability and insurgency risk.

**Time-Series** is an append-only, tick-keyed write pattern. Every institutional state change and every lifecycle transition is recorded as an immutable row in a time-series table. No row is ever mutated. Replay of any run from tick 0 reproduces the identical series deterministically. Retention of all rows for canonical runs is required; aggregation windows are layered on top for dashboard queries.

These three subsystems are tightly integrated: institutional capture raises coercion index which drives citizen lifecycle transitions toward `dissenting`; mass dissent raises reform pressure which can trigger institutional FSM transitions; citizen retirement pool sustainability depends on the total joule output of the `active` population which is itself influenced by institutional policy capacity multipliers.

---

## 2. CIV Sim Integration Notes

### 2.1 Phase Position in Tick Pipeline

Institutional evaluation runs in **Phase 2 (Policy Phase)** of the tick pipeline defined in CIV-0001. The institutional FSM transition evaluation reads current legitimacy and capture scores — both produced as outputs of **Phase 5 (Metrics Compute)** from the prior tick — and emits control signals (policy capacity multiplier, compliance rate, insurgency modifier) that are consumed by the allocation engines in Phase 3.

Citizen lifecycle transitions are evaluated in **Phase 3 (Deterministic Transition)** after allocation has settled for the current tick, because transitions depend on energy allocation outcomes that are resolved by the allocation engine in the same tick.

Time-series records are written during **Phase 5 (Metrics Compute)** immediately after institutional and lifecycle states are finalized for the tick. All writes within a single tick are batched and committed atomically.

### 2.2 Determinism Requirements

All transition evaluations must conform to CIV-0001 invariants:
- No floating-point in institutional state variables; legitimacy and capture scores are stored as `i32` scaled by `10_000` (4 decimal places of fixed-point, range `[0, 10_000]` representing `[0.0, 1.0]`).
- Institution IDs and cohort IDs are iterated in sorted order (BTreeMap).
- Transition thresholds are loaded from policy bundle configuration via serde; no runtime mutation.
- Pending-effect queue uses tick-indexed BTreeMap keyed by `(apply_at_tick, institution_id)`.

### 2.3 Workspace Crate Assignment

New crates required (add to `Cargo.toml` workspace members):
- `crates/institutions` — institutional FSM, legitimacy model, capture dynamics, propagation lag queue
- `crates/citizens` — citizen lifecycle FSM, cohort state, retirement pool, joule ledger interface

`crates/institutions` depends on `crates/policy` (for AllocationEngine trait) and `crates/metrics`.
`crates/citizens` depends on `crates/institutions` (for coercion index) and the joule ledger types from `crates/policy`.

---

## 3. Institutional Finite State Machine

### 3.1 States

An institution occupies exactly one of five states at any tick. The state determines all output multipliers consumed by other engine phases.

```rust
/// Formal states for institutional FSM.
/// Stored as u8 in serialized form for compact time-series rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum InstitutionState {
    /// Normal operation. Policy capacity at full value.
    /// Legitimacy in [legitimacy_stable_floor, 1.0].
    Stable = 0,

    /// Active legitimacy erosion or rising capture score.
    /// Policy capacity reduced; compliance rate degraded.
    /// Legitimacy in [legitimacy_contested_floor, legitimacy_stable_floor).
    Contested = 1,

    /// Dominant capture by private interests.
    /// Policy capacity severely reduced; enforcement selective.
    /// Capture score in [capture_threshold, 1.0].
    Captured = 2,

    /// Active reform underway. Capture score falling.
    /// Transition state; policy capacity partially restored each tick.
    Reforming = 3,

    /// Institution non-functional. All multipliers zero.
    /// Must be re-instantiated via governance event to recover.
    Collapsed = 4,
}
```

State descriptions:

| State | Legitimacy Range | Capture Score Range | Policy Capacity | Compliance Rate | Insurgency Modifier |
|---|---|---|---|---|---|
| `Stable` | `[floor_stable, 1.0]` | `[0.0, capture_low)` | `1.0` | `base_compliance` | `1.0` |
| `Contested` | `[floor_contested, floor_stable)` | any | `0.65 – 0.85` | `base * 0.80` | `1.20` |
| `Captured` | any | `[capture_threshold, 1.0]` | `0.40 – 0.60` | `selective` | `1.50` |
| `Reforming` | rising | falling | `0.55 + tick_delta` | recovering | `1.10` |
| `Collapsed` | `< floor_contested` | any | `0.0` | `0.0` | `2.00` |

Threshold parameters are loaded from the policy bundle YAML. Representative defaults:
- `legitimacy_stable_floor`: `0.60`
- `legitimacy_contested_floor`: `0.35`
- `capture_threshold`: `0.65`
- `capture_low`: `0.20`

### 3.2 Transition Table

```
Stable ──(legitimacy < floor_stable)──────────────────────────► Contested
Stable ──(capture_score >= capture_threshold)─────────────────► Captured
Contested ──(legitimacy < floor_contested)────────────────────► Collapsed
Contested ──(capture_score >= capture_threshold)──────────────► Captured
Contested ──(reform_pressure >= reform_trigger
             AND legitimacy recovering for N ticks)────────────► Reforming
Captured ──(reform_pressure > capture_reform_trigger
            AND legitimacy rising)─────────────────────────────► Reforming
Captured ──(legitimacy < collapse_floor)──────────────────────► Collapsed
Reforming ──(capture_score < capture_low
             AND legitimacy >= floor_stable)───────────────────► Stable
Reforming ──(reform_fails: legitimacy drops during reform)─────► Contested
Collapsed ──(governance_event: reinstatement)─────────────────► Reforming
```

All transitions are evaluated once per tick after legitimacy and capture scores are updated. Evaluation order is deterministic: institutions sorted ascending by `institution_id`. No institution's transition may depend on another institution's transition within the same tick.

### 3.3 Transition Trigger Parameters

```rust
/// Policy-bundle parameters governing institutional FSM transitions.
/// Loaded from YAML via serde. All values fixed-point i32 scaled 1/10_000.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct InstitutionTransitionParams {
    /// Legitimacy floor below which Stable → Contested fires.
    pub legitimacy_stable_floor: i32,         // default 6_000 (= 0.60)
    /// Legitimacy floor below which Contested → Collapsed fires.
    pub legitimacy_contested_floor: i32,      // default 3_500 (= 0.35)
    /// Absolute floor triggering Captured → Collapsed.
    pub legitimacy_collapse_floor: i32,       // default 1_500 (= 0.15)
    /// Capture score above which Stable/Contested → Captured fires.
    pub capture_threshold: i32,               // default 6_500 (= 0.65)
    /// Capture score below which Reforming → Stable is permitted.
    pub capture_low: i32,                     // default 2_000 (= 0.20)
    /// Reform pressure above which contested/captured reforms may start.
    pub reform_trigger: i32,                  // default 5_000 (= 0.50)
    /// Higher reform pressure required to break out of Captured.
    pub capture_reform_trigger: i32,          // default 7_500 (= 0.75)
    /// Number of consecutive ticks legitimacy must be rising before Reforming begins.
    pub reform_recovery_ticks: u32,           // default 3
    /// Propagation lag in ticks before institutional state change takes effect on outputs.
    pub propagation_lag_ticks: u32,           // default 5
}
```

### 3.4 Output Multipliers

Each state emits three multipliers consumed by the allocation engine and the metrics engine:

```rust
/// Output signals produced by an institution for the current tick.
/// These are consumed by the policy engine and metrics engine.
#[derive(Debug, Clone, Copy)]
pub struct InstitutionOutputs {
    /// Multiplicative factor on policy actions this institution can execute.
    /// Range [0.0, 1.0]. Collapsed = 0.0, Stable = 1.0.
    pub policy_capacity: f32,

    /// Rate at which the governed population complies with this institution's mandates.
    /// Range [0.0, 1.0]. Captured institutions enforce selectively, not uniformly.
    pub compliance_rate: f32,

    /// Multiplicative factor applied to insurgency risk metric.
    /// > 1.0 amplifies insurgency risk when institution is unstable or captured.
    pub insurgency_modifier: f32,
}

impl InstitutionOutputs {
    /// Compute outputs from current state. All arithmetic is f32 here because
    /// these are read-only multipliers consumed downstream; they are NOT stored
    /// as time-series data (only the source legitimacy/capture scores are stored).
    pub fn from_state(
        state: InstitutionState,
        legitimacy: f32,
        capture_score: f32,
    ) -> Self {
        match state {
            InstitutionState::Stable => Self {
                policy_capacity: 1.0,
                compliance_rate: 0.90 + 0.10 * legitimacy,
                insurgency_modifier: 1.0,
            },
            InstitutionState::Contested => Self {
                policy_capacity: 0.65 + 0.20 * legitimacy,
                compliance_rate: 0.60 + 0.20 * legitimacy,
                insurgency_modifier: 1.20 + 0.30 * (1.0 - legitimacy),
            },
            InstitutionState::Captured => Self {
                policy_capacity: 0.40 + 0.20 * (1.0 - capture_score),
                compliance_rate: 0.30 + 0.30 * (1.0 - capture_score), // selective
                insurgency_modifier: 1.50 + 0.50 * capture_score,
            },
            InstitutionState::Reforming => Self {
                policy_capacity: 0.55 + 0.30 * (1.0 - capture_score),
                compliance_rate: 0.55 + 0.25 * legitimacy,
                insurgency_modifier: 1.10,
            },
            InstitutionState::Collapsed => Self {
                policy_capacity: 0.0,
                compliance_rate: 0.0,
                insurgency_modifier: 2.00,
            },
        }
    }
}
```

---

## 4. Citizen Lifecycle Finite State Machine

### 4.1 Lifecycle Stages

```rust
/// Lifecycle stage for a citizen cohort.
/// Transitions are monotonic within a tick: a cohort may advance at most one stage
/// per tick and may not revert within the same tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum LifecycleStage {
    /// Normal participation. Earns joules, fulfills obligations, contributes to output.
    Active = 0,

    /// Elevated stress. Joule balance declining. Baseline still met but discretionary eroded.
    Strained = 1,

    /// Active dissent. Low joule security, high coercion exposure.
    /// Exerts reform pressure on institutions. May organize.
    Dissenting = 2,

    /// Mobility initiated. Consuming relocation joule budget.
    /// Reduced contribution to regional output. May exit region.
    Migrating = 3,

    /// Lifetime joule credit >= retirement threshold.
    /// Draws pension from retirement pool each tick.
    /// No longer obligated to work; no longer counted in labor supply.
    Retired = 4,
}
```

### 4.2 Transition Drivers

Lifecycle stage transitions are driven by four primary signals. All signals are normalized `[0.0, 1.0]` (or dimensionless ratios). Fixed-point arithmetic applies: signals stored as `i32` scaled `1/10_000`.

| Signal | Symbol | Description | Source |
|---|---|---|---|
| Energy Security | `ES_i` | `quota_balance_i / sustain_cost_i`. Below 1.0 = unmet needs. | Joule ledger (CIV-0107) |
| Welfare Access | `WA_i` | Fraction of baseline bundle received this tick. | Allocation engine |
| Coercion Index | `CI_i` | Experienced tyranny signal for cohort i. | Tyranny metric (metrics crate) |
| Mobility Constraints | `MC_i` | Inverse of migration freedom: quotas, enforcement, border barriers. | Regional policy params |

### 4.3 Transition Rules

```
Active ──(ES_i < energy_strain_threshold)────────────────────► Strained
Active ──(retirement_credit_i >= T_retire)───────────────────► Retired
Strained ──(ES_i < energy_crisis_threshold
            OR WA_i < welfare_crisis_threshold)──────────────► Dissenting
Strained ──(ES_i >= energy_recovery_threshold
            AND WA_i >= welfare_recovery_threshold)───────────► Active
Strained ──(retirement_credit_i >= T_retire)─────────────────► Retired
Dissenting ──(CI_i > coercion_flight_threshold
              AND MC_i < mobility_barrier_threshold)──────────► Migrating
Dissenting ──(CI_i < coercion_relief_threshold
              AND ES_i >= energy_recovery_threshold)──────────► Strained
Migrating ──(region_exit_completed)──────────────────────────► Active (new region)
Migrating ──(migration_blocked: MC_i surges)─────────────────► Dissenting
Retired ──(retirement_pool_insolvent)────────────────────────► Strained (forced return)
```

No transition may skip a stage within a single tick. `Active → Dissenting` is not a valid single-tick transition; it must pass through `Strained`. This enforces the monotonic invariant per-tick.

### 4.4 Transition Parameters

```rust
/// Policy-bundle parameters governing citizen lifecycle transitions.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LifecycleTransitionParams {
    /// ES_i below this value triggers Active → Strained.
    pub energy_strain_threshold: i32,         // default 8_500 (= 0.85)
    /// ES_i below this value triggers Strained → Dissenting.
    pub energy_crisis_threshold: i32,         // default 5_000 (= 0.50)
    /// WA_i below this value triggers Strained → Dissenting.
    pub welfare_crisis_threshold: i32,        // default 4_000 (= 0.40)
    /// ES_i above this value for recovery transition back toward Active.
    pub energy_recovery_threshold: i32,       // default 9_000 (= 0.90)
    /// WA_i above this value for recovery transition back toward Active.
    pub welfare_recovery_threshold: i32,      // default 7_500 (= 0.75)
    /// CI_i above this value triggers Dissenting → Migrating.
    pub coercion_flight_threshold: i32,       // default 7_000 (= 0.70)
    /// MC_i below this value permits migration (low barriers).
    pub mobility_barrier_threshold: i32,      // default 3_000 (= 0.30)
    /// CI_i below this value permits relief: Dissenting → Strained.
    pub coercion_relief_threshold: i32,       // default 3_500 (= 0.35)
    /// Cumulative lifetime joule credit required for retirement eligibility.
    /// Typically represents 30–40 years of median output.
    /// Stored as i64 joules; not fixed-point (raw joule count).
    pub retirement_threshold_joules: i64,     // default 1_000_000_000_000 (1 TJ)
}
```

---

## 5. State Structs

### 5.1 Institution

```rust
use std::collections::BTreeMap;

/// Full state of a single institution at a point in time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Institution {
    /// Stable unique identifier. Never reused within a run.
    pub id: u64,
    /// Human-readable name for display and logging.
    pub name: String,
    /// Type tag (e.g., "parliament", "central_bank", "labor_board", "energy_authority").
    pub institution_type: InstitutionType,
    /// Current FSM state.
    pub state: InstitutionState,
    /// Legitimacy score in fixed-point [0, 10_000] representing [0.0, 1.0].
    pub legitimacy: i32,
    /// Capture score in fixed-point [0, 10_000].
    pub capture_score: i32,
    /// Tick at which this institution entered its current state.
    pub state_entered_tick: u64,
    /// Running count of consecutive ticks with legitimacy rising (for reform trigger).
    pub legitimacy_rising_ticks: u32,
    /// Pending effects queue: (apply_at_tick → effect_payload).
    /// Effects are queued when a state transition occurs and applied after propagation lag.
    pub pending_effects: BTreeMap<u64, PendingEffect>,
}

/// Category of institution. Determines which policy domains the institution governs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InstitutionType {
    RightsAuthority,
    MarketRegulator,
    EnergyAccountingAuthority,
    GovernanceIntegrityCouncil,
    MetricReviewBoard,
    LaborBoard,
    CentralBank,
    SecurityApparatus,
}

/// An effect pending application at a future tick after propagation lag.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingEffect {
    pub originating_tick: u64,
    pub effect_type: EffectType,
    pub magnitude: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EffectType {
    PolicyCapacityDelta,
    ComplianceRateDelta,
    InsurgencyModifierDelta,
    BaselineStrengthDelta,
    SurveillanceScopeDelta,
}
```

### 5.2 Legitimacy Metrics

```rust
/// Decomposed legitimacy score for a single institution at a single tick.
/// All components are fixed-point i32 scaled 1/10_000.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LegitimacyMetrics {
    pub institution_id: u64,
    pub tick: u64,
    /// Combined legitimacy score: weighted sum of components, clamped [0, 10_000].
    pub score: i32,
    /// Contribution from enforcement intensity being within acceptable bounds.
    /// High enforcement without constitutional grounding reduces this.
    pub enforcement_alignment: i32,
    /// Contribution from coalition breadth (fraction of polity represented).
    pub coalition_support: i32,
    /// Contribution from transfer fairness (equitable baseline distribution).
    pub transfer_fairness: i32,
    /// Reduction from external pressure (sanctions, geopolitical coercion).
    pub external_pressure_penalty: i32,
    /// Reduction from observed corruption leakage this tick.
    pub corruption_penalty: i32,
    /// Reduction from scarcity amplification (high scarcity + enforcement = legitimacy drag).
    pub scarcity_drag: i32,
}
```

### 5.3 Capture Score

```rust
/// Capture score components for a single institution at a single tick.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureScore {
    pub institution_id: u64,
    pub tick: u64,
    /// Total capture score [0, 10_000].
    pub score: i32,
    /// Accumulated rent concentration influence.
    pub rent_pressure: i32,
    /// Shadow network influence (from shadow state model).
    pub shadow_influence: i32,
    /// Elite coalition capture accumulation.
    pub elite_capture: i32,
    /// Detection probability this tick (function of oversight, governance quality).
    pub detection_probability: i32,
    /// Reduction from active oversight and independent audit actions.
    pub oversight_reduction: i32,
}
```

### 5.4 Citizen Cohort

```rust
/// Aggregated cohort of citizens sharing the same lifecycle stage and region.
/// Cohorts are defined by (region_id, skill_quintile, age_band).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CitizenCohort {
    pub cohort_id: u64,
    pub region_id: u32,
    pub skill_quintile: u8,   // 0–4
    pub age_band: AgeBand,
    /// Number of citizens in this cohort this tick.
    pub population: u32,
    /// Current lifecycle stage.
    pub stage: LifecycleStage,
    /// Tick at which this cohort entered its current stage.
    pub stage_entered_tick: u64,
    /// Aggregate energy security score [0, 10_000].
    pub energy_security: i32,
    /// Aggregate welfare access score [0, 10_000].
    pub welfare_access: i32,
    /// Aggregate coercion index [0, 10_000].
    pub coercion_index: i32,
    /// Aggregate mobility constraint score [0, 10_000].
    pub mobility_constraint: i32,
    /// Cumulative lifetime joule credit (raw joules, i64).
    pub lifetime_joule_credit: i64,
    /// Joule credit earned this tick (raw joules, i64).
    pub joules_earned_this_tick: i64,
    /// Stress score [0, 10_000]; high stress accelerates stage transitions.
    pub stress_score: i32,
    /// Running count of consecutive ticks in this stage (for hysteresis).
    pub ticks_in_stage: u32,
}

/// Broad age categories for cohort stratification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgeBand {
    Youth,       // 0–24
    Working,     // 25–54
    LateCareer,  // 55–64
    Elder,       // 65+
}
```

### 5.5 Cohort Metrics

```rust
/// Per-tick metrics snapshot for a citizen cohort. Written to time-series.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CohortMetrics {
    pub run_id: u64,
    pub tick: u64,
    pub cohort_id: u64,
    pub stage: LifecycleStage,
    pub population: u32,
    pub stress_score: i32,
    pub energy_security: i32,
    pub welfare_access: i32,
    pub coercion_index: i32,
    /// Dissenting cohort fraction exerts reform pressure on institutions.
    pub reform_pressure_contribution: i32,
    /// Joules produced this tick by this cohort (raw joules).
    pub joules_produced: i64,
    /// Joules consumed from retirement pool this tick (retired cohorts only).
    pub pension_draw_joules: i64,
}
```

### 5.6 Retirement Pool

```rust
/// Global retirement pool state at a single tick.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetirementPool {
    pub run_id: u64,
    pub tick: u64,
    /// Total joules held in the retirement reserve (raw joules, i64).
    pub reserve_joules: i64,
    /// Count of citizens currently in Retired stage drawing pension.
    pub retired_population: u32,
    /// Pension rate per retired citizen per tick (raw joules).
    pub pension_rate_per_tick: i64,
    /// Total pension disbursed this tick (raw joules).
    pub total_disbursement_this_tick: i64,
    /// Government subsidy added to pool this tick (raw joules).
    pub government_subsidy_this_tick: i64,
    /// Fraction of total production contributed to pool this tick.
    pub contribution_rate: i32,   // fixed-point [0, 10_000]
    /// True if pool is solvent for this tick (reserve > 0 after disbursement).
    pub is_solvent: bool,
    /// Solvency runway in ticks at current drain rate (0 = insolvent now).
    pub solvency_runway_ticks: u32,
}
```

### 5.7 Time-Series Record

```rust
/// Generic append-only time-series record. One variant per record type.
/// Written once per tick per entity; never mutated after write.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TimeSeriesRecord {
    InstitutionState {
        run_id: u64,
        tick: u64,
        institution_id: u64,
        state: InstitutionState,
        legitimacy: i32,
        capture_score: i32,
    },
    CitizenLifecycle {
        run_id: u64,
        tick: u64,
        cohort_id: u64,
        stage: LifecycleStage,
        stress_score: i32,
        population: u32,
        joules_produced: i64,
    },
    LegitimacyUpdate {
        run_id: u64,
        tick: u64,
        institution_id: u64,
        metrics: LegitimacyMetrics,
    },
    CaptureUpdate {
        run_id: u64,
        tick: u64,
        institution_id: u64,
        score: CaptureScore,
    },
    RetirementPoolSnapshot {
        run_id: u64,
        tick: u64,
        pool: RetirementPool,
    },
}
```

---

## 6. Rust Module Layout

```
crates/institutions/
├── Cargo.toml
└── src/
    ├── lib.rs                  # Public API surface; re-exports
    ├── fsm.rs                  # InstitutionState enum, transition evaluation
    ├── institution.rs          # Institution struct, PendingEffect, EffectType
    ├── legitimacy.rs           # LegitimacyMetrics, legitimacy computation
    ├── capture.rs              # CaptureScore, capture accumulation, detection
    ├── propagation.rs          # Pending-effect queue, apply_pending_effects()
    ├── outputs.rs              # InstitutionOutputs, from_state()
    ├── params.rs               # InstitutionTransitionParams serde struct
    └── tests/
        ├── fsm_tests.rs        # Deterministic state chain tests
        ├── capture_tests.rs    # Capture accumulation and detection tests
        └── propagation_tests.rs

crates/citizens/
├── Cargo.toml
└── src/
    ├── lib.rs                  # Public API surface; re-exports
    ├── lifecycle.rs            # LifecycleStage enum, transition evaluation
    ├── cohort.rs               # CitizenCohort struct, cohort update logic
    ├── metrics.rs              # CohortMetrics assembly
    ├── retirement.rs           # RetirementPool, pension computation, solvency
    ├── ledger.rs               # Joule ledger interface (reads from policy crate types)
    ├── params.rs               # LifecycleTransitionParams serde struct
    └── tests/
        ├── lifecycle_tests.rs  # Threshold-governed transitions
        ├── retirement_tests.rs # Pool conservation invariants
        └── monotonic_tests.rs  # Monotonic per-tick transition invariant
```

---

## 7. Event Contracts

All events are emitted to the event bus (CIV-0001 Phase 6 broadcast) and appended to the event log. Events include the `state_hash` of the producing state per CIV-0001 E3.

### 7.1 `institution.state_changed.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "institution.state_changed.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "institution_id", "institution_name", "from_state", "to_state",
               "legitimacy", "capture_score", "trigger"],
  "properties": {
    "event_type": { "const": "institution.state_changed.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string", "description": "Hex SHA-256 of producing world state" },
    "institution_id": { "type": "integer", "minimum": 0 },
    "institution_name": { "type": "string" },
    "from_state": {
      "type": "string",
      "enum": ["Stable", "Contested", "Captured", "Reforming", "Collapsed"]
    },
    "to_state": {
      "type": "string",
      "enum": ["Stable", "Contested", "Captured", "Reforming", "Collapsed"]
    },
    "legitimacy": {
      "type": "integer",
      "minimum": 0,
      "maximum": 10000,
      "description": "Fixed-point legitimacy score at transition; divide by 10000 for float"
    },
    "capture_score": {
      "type": "integer",
      "minimum": 0,
      "maximum": 10000
    },
    "trigger": {
      "type": "string",
      "enum": [
        "legitimacy_below_stable_floor",
        "legitimacy_below_contested_floor",
        "legitimacy_below_collapse_floor",
        "capture_exceeded_threshold",
        "reform_pressure_triggered",
        "reform_completed",
        "reform_failed",
        "governance_event_reinstatement"
      ]
    },
    "propagation_apply_tick": {
      "type": "integer",
      "description": "Tick at which output multiplier changes take effect after propagation lag"
    }
  },
  "additionalProperties": false
}
```

### 7.2 `citizen.lifecycle_transitioned.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "citizen.lifecycle_transitioned.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "cohort_id", "region_id", "from_stage", "to_stage",
               "population_affected", "trigger", "energy_security",
               "welfare_access", "coercion_index"],
  "properties": {
    "event_type": { "const": "citizen.lifecycle_transitioned.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "cohort_id": { "type": "integer", "minimum": 0 },
    "region_id": { "type": "integer", "minimum": 0 },
    "from_stage": {
      "type": "string",
      "enum": ["Active", "Strained", "Dissenting", "Migrating", "Retired"]
    },
    "to_stage": {
      "type": "string",
      "enum": ["Active", "Strained", "Dissenting", "Migrating", "Retired"]
    },
    "population_affected": {
      "type": "integer",
      "minimum": 0,
      "description": "Number of cohort members undergoing this transition this tick"
    },
    "trigger": {
      "type": "string",
      "enum": [
        "energy_strain",
        "energy_crisis",
        "welfare_crisis",
        "energy_recovery",
        "welfare_recovery",
        "coercion_flight",
        "migration_blocked",
        "region_exit_completed",
        "retirement_threshold_reached",
        "retirement_pool_insolvent"
      ]
    },
    "energy_security": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "welfare_access": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "coercion_index": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "mobility_constraint": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "reform_pressure_delta": {
      "type": "integer",
      "description": "Change in regional reform pressure from this transition event (fixed-point)"
    }
  },
  "additionalProperties": false
}
```

### 7.3 `institution.legitimacy_updated.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "institution.legitimacy_updated.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "institution_id",
               "legitimacy_score", "enforcement_alignment", "coalition_support",
               "transfer_fairness", "external_pressure_penalty",
               "corruption_penalty", "scarcity_drag"],
  "properties": {
    "event_type": { "const": "institution.legitimacy_updated.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "institution_id": { "type": "integer", "minimum": 0 },
    "legitimacy_score": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "enforcement_alignment": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "coalition_support": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "transfer_fairness": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "external_pressure_penalty": { "type": "integer", "minimum": -10000, "maximum": 0 },
    "corruption_penalty": { "type": "integer", "minimum": -10000, "maximum": 0 },
    "scarcity_drag": { "type": "integer", "minimum": -10000, "maximum": 0 },
    "delta_from_prior_tick": {
      "type": "integer",
      "description": "Signed delta from prior tick legitimacy score (positive = recovering)"
    }
  },
  "additionalProperties": false
}
```

### 7.4 `citizen.retirement_threshold_crossed.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "citizen.retirement_threshold_crossed.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "cohort_id",
               "region_id", "citizen_count", "lifetime_joule_credit",
               "retirement_threshold_joules", "pension_rate_per_tick",
               "pool_reserve_after_joules", "pool_is_solvent"],
  "properties": {
    "event_type": { "const": "citizen.retirement_threshold_crossed.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "cohort_id": { "type": "integer", "minimum": 0 },
    "region_id": { "type": "integer", "minimum": 0 },
    "citizen_count": { "type": "integer", "minimum": 1 },
    "lifetime_joule_credit": {
      "type": "integer",
      "description": "Raw joules; lifetime_joule_credit >= retirement_threshold_joules"
    },
    "retirement_threshold_joules": { "type": "integer", "minimum": 0 },
    "pension_rate_per_tick": {
      "type": "integer",
      "description": "Joules per tick drawn from retirement pool per citizen"
    },
    "pool_reserve_after_joules": {
      "type": "integer",
      "description": "Pool reserve after absorbing this cohort's pension obligation"
    },
    "pool_is_solvent": { "type": "boolean" },
    "maintenance_tier": {
      "type": "string",
      "enum": ["full", "reduced", "minimum"],
      "description": "Retirement maintenance tier granted based on pool solvency"
    }
  },
  "additionalProperties": false
}
```

---

## 8. Database Schema (SQL DDL)

All time-series tables use a `(run_id, tick, <entity_id>)` composite primary key. No row is ever updated. Deletion is only permitted by retention policy operations on non-canonical runs.

```sql
-- ============================================================
-- INSTITUTION STATE TIME-SERIES
-- One row per (run, tick, institution). Append-only.
-- ============================================================
CREATE TABLE institution_states (
    run_id              BIGINT      NOT NULL,
    tick                BIGINT      NOT NULL,
    institution_id      BIGINT      NOT NULL,
    institution_type    TEXT        NOT NULL,
    state               SMALLINT    NOT NULL,   -- InstitutionState repr(u8)
    legitimacy          INT         NOT NULL,   -- fixed-point [0, 10_000]
    capture_score       INT         NOT NULL,   -- fixed-point [0, 10_000]
    policy_capacity_fp  INT         NOT NULL,   -- fixed-point [0, 10_000]
    compliance_rate_fp  INT         NOT NULL,   -- fixed-point [0, 10_000]
    insurgency_mod_fp   INT         NOT NULL,   -- fixed-point [0, 20_000] (can exceed 1.0)
    ticks_in_state      INT         NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, institution_id)
);

CREATE INDEX idx_inst_states_run_state
    ON institution_states (run_id, state, tick);

CREATE INDEX idx_inst_states_run_inst
    ON institution_states (run_id, institution_id, tick);

-- ============================================================
-- CITIZEN LIFECYCLE TIME-SERIES
-- One row per (run, tick, cohort). Append-only.
-- ============================================================
CREATE TABLE citizen_lifecycle (
    run_id              BIGINT      NOT NULL,
    tick                BIGINT      NOT NULL,
    cohort_id           BIGINT      NOT NULL,
    region_id           INT         NOT NULL,
    skill_quintile      SMALLINT    NOT NULL,
    age_band            SMALLINT    NOT NULL,
    stage               SMALLINT    NOT NULL,   -- LifecycleStage repr(u8)
    population          INT         NOT NULL,
    stress_score        INT         NOT NULL,   -- fixed-point [0, 10_000]
    energy_security     INT         NOT NULL,   -- fixed-point [0, 10_000]
    welfare_access      INT         NOT NULL,   -- fixed-point [0, 10_000]
    coercion_index      INT         NOT NULL,   -- fixed-point [0, 10_000]
    mobility_constraint INT         NOT NULL,   -- fixed-point [0, 10_000]
    joules_produced     BIGINT      NOT NULL,   -- raw joules
    pension_draw_joules BIGINT      NOT NULL,   -- raw joules; 0 if not Retired
    reform_pressure_fp  INT         NOT NULL,   -- fixed-point contribution [0, 10_000]
    lifetime_joule_credit BIGINT    NOT NULL,   -- raw joules; cumulative
    ticks_in_stage      INT         NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, cohort_id)
);

CREATE INDEX idx_lifecycle_run_stage
    ON citizen_lifecycle (run_id, stage, tick);

CREATE INDEX idx_lifecycle_run_region
    ON citizen_lifecycle (run_id, region_id, tick);

CREATE INDEX idx_lifecycle_dissenting
    ON citizen_lifecycle (run_id, tick)
    WHERE stage = 2;   -- Dissenting = 2

-- ============================================================
-- LEGITIMACY HISTORY
-- Decomposed legitimacy components per institution per tick.
-- ============================================================
CREATE TABLE legitimacy_history (
    run_id                   BIGINT      NOT NULL,
    tick                     BIGINT      NOT NULL,
    institution_id           BIGINT      NOT NULL,
    legitimacy_score         INT         NOT NULL,
    enforcement_alignment    INT         NOT NULL,
    coalition_support        INT         NOT NULL,
    transfer_fairness        INT         NOT NULL,
    external_pressure_penalty INT        NOT NULL,   -- negative value
    corruption_penalty       INT         NOT NULL,   -- negative value
    scarcity_drag            INT         NOT NULL,   -- negative value
    delta_from_prior_tick    INT         NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, institution_id)
);

CREATE INDEX idx_legit_run_inst
    ON legitimacy_history (run_id, institution_id, tick);

-- ============================================================
-- CAPTURE EVENTS
-- One row per (run, tick, institution) when capture score changes.
-- ============================================================
CREATE TABLE capture_events (
    run_id               BIGINT      NOT NULL,
    tick                 BIGINT      NOT NULL,
    institution_id       BIGINT      NOT NULL,
    capture_score        INT         NOT NULL,
    rent_pressure        INT         NOT NULL,
    shadow_influence     INT         NOT NULL,
    elite_capture        INT         NOT NULL,
    detection_probability INT        NOT NULL,
    oversight_reduction  INT         NOT NULL,
    delta_from_prior_tick INT        NOT NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, institution_id)
);

-- ============================================================
-- RETIREMENT POOL SNAPSHOTS
-- One row per (run, tick). Append-only.
-- ============================================================
CREATE TABLE retirement_pool_snapshots (
    run_id                      BIGINT      NOT NULL,
    tick                        BIGINT      NOT NULL,
    reserve_joules              BIGINT      NOT NULL,
    retired_population          INT         NOT NULL,
    pension_rate_per_tick       BIGINT      NOT NULL,
    total_disbursement_joules   BIGINT      NOT NULL,
    government_subsidy_joules   BIGINT      NOT NULL,
    contribution_rate_fp        INT         NOT NULL,   -- fixed-point [0, 10_000]
    is_solvent                  BOOLEAN     NOT NULL,
    solvency_runway_ticks       INT         NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick)
);

-- ============================================================
-- AGGREGATION VIEWS (read-only; no mutation of base tables)
-- ============================================================

-- Institutional state at 10-tick aggregation windows.
CREATE VIEW institution_states_10tick AS
SELECT
    run_id,
    (tick / 10) * 10 AS window_start_tick,
    institution_id,
    -- last state in window (ORDER BY tick DESC LIMIT 1 per group)
    (ARRAY_AGG(state ORDER BY tick DESC))[1]          AS final_state,
    MIN(legitimacy)                                    AS min_legitimacy,
    MAX(legitimacy)                                    AS max_legitimacy,
    AVG(legitimacy)::INT                               AS avg_legitimacy,
    MIN(capture_score)                                 AS min_capture,
    MAX(capture_score)                                 AS max_capture
FROM institution_states
GROUP BY run_id, (tick / 10) * 10, institution_id;

-- Lifecycle population by stage and region per tick.
CREATE VIEW lifecycle_stage_summary AS
SELECT
    run_id,
    tick,
    region_id,
    stage,
    SUM(population)::BIGINT                            AS total_population,
    AVG(stress_score)::INT                             AS avg_stress,
    AVG(energy_security)::INT                          AS avg_energy_security,
    AVG(coercion_index)::INT                           AS avg_coercion,
    SUM(joules_produced)                               AS total_joules_produced,
    SUM(pension_draw_joules)                           AS total_pension_draw
FROM citizen_lifecycle
GROUP BY run_id, tick, region_id, stage;

-- ============================================================
-- RETENTION POLICY
-- Canonical runs: retain all rows indefinitely.
-- Non-canonical runs: eligible for purge after 30 days.
-- ============================================================
ALTER TABLE institution_states       ADD COLUMN is_canonical_run BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE citizen_lifecycle        ADD COLUMN is_canonical_run BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE legitimacy_history       ADD COLUMN is_canonical_run BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE capture_events           ADD COLUMN is_canonical_run BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE retirement_pool_snapshots ADD COLUMN is_canonical_run BOOLEAN NOT NULL DEFAULT FALSE;

-- Purge non-canonical runs older than 30 days (run via scheduled job):
-- DELETE FROM institution_states
-- WHERE NOT is_canonical_run AND created_at < NOW() - INTERVAL '30 days';
-- (repeat for each table)
```

---

## 9. Legitimacy Model (Mathematical Formulation)

Legitimacy for institution `m` at tick `t` is a weighted linear combination of five input signals, clamped to `[0.0, 1.0]`, then stored as fixed-point:

```
L_{m,t} = clamp(
    w_EA * EA_{m,t}
  + w_CS * CS_{m,t}
  + w_TF * TF_{m,t}
  - w_EP * EP_{m,t}
  - w_CR * CR_{m,t}
  - w_SD * SD_{m,t},
  0.0, 1.0
)
```

### Input Signal Definitions

**Enforcement Alignment (EA)**: How closely enforcement intensity matches constitutional boundaries. High enforcement capacity without legitimate mandate reduces EA.

```
EA_{m,t} = 1 - |enforcement_intensity_{m,t} - constitutional_norm_{m}| / constitutional_range
```

Coefficient range: `w_EA ∈ [0.15, 0.30]`. Default: `0.25`.

**Coalition Support (CS)**: Breadth of active social coalition supporting the institution. Derived from ideology aggregation layer — fraction of population with `StatePreference > 0.5` or `TrustInInstitutions > 0.6`.

```
CS_{m,t} = fraction_of_polity_supporting_m(t)
```

Coalition support is attenuated by polarization: if Gini of ideology distribution > 0.6, CS is multiplied by `(1 - polarization_penalty)`.

Coefficient range: `w_CS ∈ [0.20, 0.35]`. Default: `0.30`.

**Transfer Fairness (TF)**: Degree to which the institution allocates resources equitably. Measured as `1 - Gini(allocation_per_cohort)` for the domains under institution's governance.

```
TF_{m,t} = 1 - Gini(allocation_vector_{m,t})
```

Coefficient range: `w_TF ∈ [0.15, 0.25]`. Default: `0.20`.

**External Pressure Penalty (EP)**: Sanctions, geopolitical coercion, or external interference reduce legitimacy by reducing the institution's apparent effectiveness.

```
EP_{m,t} = sanctions_intensity_{t} * external_interference_factor_{m}
```

Coefficient range: `w_EP ∈ [0.05, 0.15]`. Default: `0.10`.

**Corruption Penalty (CR)**: Direct corruption leakage from institution outputs reduces legitimacy.

```
CR_{m,t} = corruption_leakage_rate_{m,t} * enforcement_visibility_{m,t}
```

Visibility term: legitimate enforcement makes corruption more visible and thus more damaging to legitimacy when discovered.

Coefficient range: `w_CR ∈ [0.10, 0.25]`. Default: `0.15`.

**Scarcity Drag (SD)**: Elevated scarcity pressure with tight enforcement creates legitimacy drag because the institution is seen as coercive rather than protective.

```
SD_{m,t} = scarcity_pressure_{r,t} * enforcement_intensity_{m,t} * (1 - baseline_strength_{r,t})
```

If baseline is strong, enforcement under scarcity is seen as protective, not punitive. Coefficient range: `w_SD ∈ [0.05, 0.15]`. Default: `0.10`.

### Coefficient Bounds Summary

| Coefficient | Min | Max | Default | Sum (all defaults = 1.10; normalized post-clamping) |
|---|---|---|---|---|
| `w_EA` | 0.15 | 0.30 | 0.25 | — |
| `w_CS` | 0.20 | 0.35 | 0.30 | — |
| `w_TF` | 0.15 | 0.25 | 0.20 | — |
| `w_EP` | 0.05 | 0.15 | 0.10 | — |
| `w_CR` | 0.10 | 0.25 | 0.15 | — |
| `w_SD` | 0.05 | 0.15 | 0.10 | — |

Coefficients are loaded from policy bundle YAML. Enforcement: sum of positive weights must be `>= 0.60` and `<= 0.90`; sum of penalty weights must be `>= 0.20` and `<= 0.40`.

---

## 10. Capture Dynamics

### 10.1 Capture Score Accumulation

Capture score for institution `m` accumulates via shadow influence and rent concentration pressure each tick:

```
C_{m,t+1} = C_{m,t}
           + alpha_rent * RentConcentration_{r,t}   * Susceptibility_{m,t}
           + alpha_shadow * ShadowInfluence_{m,t}    * Susceptibility_{m,t}
           - beta_oversight * Oversight_{m,t}
           - beta_detection * DetectionEvent_{m,t}
```

**Susceptibility** is the inverse of governance quality, modulated by institution type:

```
Susceptibility_{m,t} = (1 - GovernanceQuality_{r,t}) * type_susceptibility_factor[m.institution_type]
```

Type susceptibility factors (default):
- `SecurityApparatus`: 0.80 (high capture risk)
- `MarketRegulator`: 0.70
- `CentralBank`: 0.65
- `LaborBoard`: 0.60
- `EnergyAccountingAuthority`: 0.55
- `RightsAuthority`: 0.45
- `GovernanceIntegrityCouncil`: 0.40
- `MetricReviewBoard`: 0.35

### 10.2 Detection Probability

Detection of capture reduces capture score by triggering a reform event:

```
P(detection_{m,t}) = sigma(oversight_{m,t} + transparency_{r,t} - capture_score_{m,t})
```

Where `sigma` is the standard logistic function. If detection fires:
- `DetectionEvent_{m,t} = 1`
- `capture_score` penalized by `detection_penalty` parameter (default: `0.25` of current score)
- Event `institution.state_changed.v1` emitted with trigger `capture_reform_triggered`

### 10.3 Reform Trigger Conditions

Capture to Reforming transition fires when ALL of the following hold:
1. `capture_score_{m,t} >= capture_threshold`
2. `reform_pressure_{r,t} > capture_reform_trigger` (from dissenting citizen cohorts)
3. `legitimacy_{m,t}` has risen for at least `reform_recovery_ticks` consecutive ticks
4. A detection event fired within the last `detection_window_ticks` (default: 10) ticks

Reform halts and reverts to Captured if `legitimacy_{m,t}` drops during the Reforming phase for more than `reform_failure_ticks` (default: 3) consecutive ticks.

---

## 11. Time-Series Architecture

### 11.1 Write Pattern

All time-series writes are append-only. The write sequence per tick is:

```
1. Evaluate institutional FSM transitions (read prior tick legitimacy/capture)
2. Compute LegitimacyMetrics and CaptureScore for all institutions
3. Evaluate citizen lifecycle transitions for all cohorts
4. Compute CohortMetrics for all cohorts
5. Update RetirementPool
6. Batch all rows: institution_states, citizen_lifecycle, legitimacy_history,
   capture_events, retirement_pool_snapshots
7. Commit batch atomically (single transaction per tick)
8. Emit events to event bus
```

No row from step 6 may reference data from a tick higher than its own tick column. No row is inserted or updated outside of step 7.

### 11.2 Tick-Keyed Primary Key Design

Primary key structure: `(run_id BIGINT, tick BIGINT, <entity_id> BIGINT)`.

- `run_id` partitions all data by simulation run. Multiple runs of the same scenario produce independent rows.
- `tick` is monotonically increasing within a run. There are no gaps (every tick produces a row for every live entity).
- `<entity_id>` (institution_id or cohort_id) is stable within a run. Entity creation and destruction produce sentinel rows (state = `Collapsed`, population = 0) to maintain completeness of the series.

### 11.3 Replay-Safe Read Patterns

Any query that reads time-series data MUST filter by `(run_id, tick)`. Queries MUST NOT join across runs. Replay verification computes a rolling hash of `(tick, institution_id, state, legitimacy, capture_score)` ordered by `(tick ASC, institution_id ASC)` and compares against the reference hash stored in the run metadata table.

```sql
-- Replay verification query pattern (reference implementation):
SELECT
    MD5(STRING_AGG(
        tick::TEXT || '|' || institution_id::TEXT || '|'
        || state::TEXT || '|' || legitimacy::TEXT || '|' || capture_score::TEXT,
        ',' ORDER BY tick ASC, institution_id ASC
    )) AS replay_hash
FROM institution_states
WHERE run_id = $1;
```

### 11.4 Aggregation Windows

Raw tick-level data is the source of truth. Aggregations are views or materialized views only; they never replace raw rows.

| Window | Purpose | Table | Refresh |
|---|---|---|---|
| 10-tick | Dashboard time series (smoothed) | `institution_states_10tick` (view) | On query |
| 100-tick | Long-run trend lines | Materialized view (refresh hourly) | Scheduled |
| Stage summary | Per-region lifecycle breakdown | `lifecycle_stage_summary` (view) | On query |

### 11.5 Retention Policies

- **Canonical runs**: All rows retained indefinitely. `is_canonical_run = TRUE`.
- **Research / parameter sweep runs**: Retained for 30 days. Purged by scheduled job.
- **Test runs**: Deleted immediately after test assertion (cleaned up in test teardown).

---

## 12. Retirement Pool Mechanics

### 12.1 Retirement Threshold

The retirement threshold `T_retire` is the cumulative lifetime joule credit a citizen cohort must reach before becoming eligible for the `Retired` lifecycle stage. It is not an age or wealth condition; it is a physical energy production threshold.

```
RetirementEligible(cohort, t) = (lifetime_joule_credit_{cohort,t} >= T_retire)
```

Default `T_retire = 1_000_000_000_000` joules (1 TJ), representing approximately 30–40 years of median-output work at `2e7 J/hr * 40 hr/week * 52 weeks * 30 years = 1.248e12 J` (scenario-calibrated). The threshold is stored in the policy bundle and does not change within a run.

### 12.2 Maintenance Tier Allocation

Upon crossing `T_retire`, a cohort is assigned a maintenance tier based on pool solvency at the crossing tick:

| Pool Solvency State | Maintenance Tier | Pension Rate |
|---|---|---|
| `solvency_runway_ticks >= 520` (10 years) | `full` | `median_sustain_cost_j * 1.0` per tick |
| `solvency_runway_ticks in [104, 520)` (2–10 years) | `reduced` | `median_sustain_cost_j * 0.75` per tick |
| `solvency_runway_ticks < 104` (< 2 years) | `minimum` | `median_sustain_cost_j * 0.50` per tick |

Maintenance tier is re-evaluated each tick and may degrade if pool solvency deteriorates. Degradation from `full` to `reduced` emits a `citizen.retirement_threshold_crossed.v1` event with updated tier.

### 12.3 Pension Draw and Pool Funding

Each tick:

```
total_disbursement_{t} = pension_rate_per_tier * retired_population_{t}

government_subsidy_{t} = max(
    0,
    BaseSubsidyShare * TotalOutput_{t}
) + scarcity_supplement_{t}

contribution_inflow_{t} = ContributionRate * TotalActiveOutput_{t}

reserve_joules_{t+1} = reserve_joules_{t}
                      + government_subsidy_{t}
                      + contribution_inflow_{t}
                      - total_disbursement_{t}
```

`ContributionRate` is a policy bundle parameter (default: `0.08`, i.e., 8% of active cohort output). `BaseSubsidyShare` is a policy bundle parameter (default: `0.02`).

Pool insolvency occurs when `reserve_joules_{t+1} <= 0`. On insolvency, disbursement is prorated:

```
actual_disbursement_{t} = min(total_disbursement_{t}, reserve_joules_{t})
prorated_rate_{t} = actual_disbursement_{t} / retired_population_{t}
```

If `prorated_rate_{t} < minimum_tier_pension`, the insolvency event triggers `retirement_pool_insolvent` transition, forcing affected cohorts from `Retired` back to `Strained`.

### 12.4 Post-Retirement Usage Rules

Citizens in `Retired` status:
- Maintain full baseline provision (unconditional; not debitable from retirement pension).
- May not accumulate new joule wealth beyond the pension draw (no additional work earnings credited to lifetime joule credit).
- May engage in voluntary contribution (creative, care, mentoring) which earns joules at `0.25x` the standard intensity factor, credited to the retirement pool rather than personal ledger.
- Generational transfer of retirement credit: disabled by default; configurable per scenario.

### 12.5 Government Subsidy Formula

```
government_subsidy_{t} = max(0,
    BaseSubsidyShare * TotalOutput_{t}
    + SolvencyBonus * max(0, TargetRunway - solvency_runway_ticks_{t})
    + ScarcityPressureSupplementRate * (1 - BaselineStrength_{r,t}) * ScarcityPressure_{r,t}
)
```

The scarcity supplement increases subsidy under scarcity conditions when baseline strength is low, to prevent pool insolvency from forcing retirees into forced labor during supply shocks.

---

## 13. Propagation Lag

### 13.1 Pending-Effect Queue

Institutional state changes do not take effect on downstream policy outputs immediately. There is a configurable propagation lag of `propagation_lag_ticks` (default: 5 ticks) between a state transition and the change in output multipliers delivered to the allocation engine.

The pending-effect queue is a `BTreeMap<u64, PendingEffect>` keyed by `apply_at_tick`:

```rust
/// Apply all pending effects for the current tick.
/// Called during Phase 2 (Policy Phase) of the tick pipeline.
pub fn apply_pending_effects(
    institution: &mut Institution,
    current_tick: u64,
    outputs: &mut InstitutionOutputs,
) {
    // Collect keys to apply (all keys <= current_tick)
    let apply_keys: Vec<u64> = institution
        .pending_effects
        .range(..=current_tick)
        .map(|(k, _)| *k)
        .collect();

    for key in apply_keys {
        if let Some(effect) = institution.pending_effects.remove(&key) {
            apply_single_effect(&effect, outputs);
        }
    }
}
```

### 13.2 Delay Parameter

When a state transition fires at tick `T`, the effect on output multipliers is enqueued with `apply_at_tick = T + propagation_lag_ticks`. During the lag period, the institution continues emitting the multipliers from its prior state. This models real-world institutional inertia: a government that collapses doesn't immediately lose all enforcement capacity in the same week.

### 13.3 Effect Application to Economy and Policy Modules

After `apply_pending_effects` runs, the `InstitutionOutputs` struct for each institution is passed to:
- **Allocation engine** (`crates/policy`): `policy_capacity` gates the magnitude of policy interventions the institution can execute this tick.
- **Tyranny metric** (`crates/metrics`): `insurgency_modifier` feeds directly into the insurgency risk computation.
- **Compliance tracking** (`crates/metrics`): `compliance_rate` determines the fraction of cohorts that comply with institutional mandates vs. evade.

---

## 14. Conservation Invariants

The following invariants must hold at every tick and are enforced by property tests.

### INV-1: Legitimacy Bounded

```
For all (run_id, tick, institution_id):
    0 <= legitimacy_history.legitimacy_score <= 10_000
    0 <= institution_states.legitimacy <= 10_000
```

### INV-2: Capture Score Bounded

```
For all (run_id, tick, institution_id):
    0 <= capture_events.capture_score <= 10_000
    0 <= institution_states.capture_score <= 10_000
```

### INV-3: Lifecycle Transitions Monotonic Per Tick

Within a single tick, no cohort may advance more than one stage. No cohort may revert a stage within the same tick. Specifically, if cohort C is at stage S at the start of tick T's transition evaluation, it may only transition to stage S+1 or S-1 (bounded by [0,4]) and may not transition back within the same tick's evaluation loop.

```
For all cohort transitions within tick T:
    |to_stage - from_stage| <= 1
    to_stage in {from_stage - 1, from_stage, from_stage + 1}
```

Exception: `Active → Retired` is permitted in one tick when `retirement_credit >= T_retire` because retirement is not a stress-driven transition but a threshold crossing.

### INV-4: Append-Only Time-Series

```
For all time-series tables:
    No UPDATE or DELETE on rows where run_id matches a canonical run.
    Row count for (run_id, entity_id) is monotonically increasing in tick.
    For canonical runs: MAX(tick) per (run_id, entity_id) = final_tick_of_run.
```

### INV-5: Retirement Pool Non-Negative Reserve (Post-Proration)

```
For all (run_id, tick):
    reserve_joules >= 0 AFTER proration of disbursement.
    (reserve may reach 0 but never go negative after proration is applied)
```

### INV-6: Pending-Effect Queue Monotonic Application

```
For all institutions:
    Effects are applied in apply_at_tick order.
    No effect is applied twice.
    No effect is applied before apply_at_tick.
```

---

## 15. Acceptance Test Suite

All tests are in `crates/institutions/src/tests/` and `crates/citizens/src/tests/`. All tests must pass with `cargo test --workspace`.

```rust
// ============================================================
// INSTITUTIONAL TESTS
// ============================================================

#[cfg(test)]
mod fsm_tests {
    use crate::fsm::*;
    use crate::institution::*;
    use crate::params::*;

    /// Given a known sequence of (legitimacy, capture_score) inputs,
    /// the institutional state chain must be deterministic and reproducible.
    #[test]
    fn test_deterministic_institutional_state_chain() {
        // Build two identical institutions with identical input sequences.
        // Assert state chains match exactly.
        let params = InstitutionTransitionParams::default();
        let mut inst_a = Institution::new_test(1, InstitutionState::Stable);
        let mut inst_b = Institution::new_test(1, InstitutionState::Stable);

        let inputs: Vec<(i32, i32)> = vec![
            (9_000, 1_000), // Stable
            (5_500, 1_500), // → Contested
            (3_000, 2_000), // → Collapsed
        ];

        for (legitimacy, capture) in &inputs {
            let state_a = evaluate_transition(&mut inst_a, *legitimacy, *capture, &params, 0);
            let state_b = evaluate_transition(&mut inst_b, *legitimacy, *capture, &params, 0);
            assert_eq!(state_a, state_b, "States must match for identical inputs");
        }
    }

    /// Transition from Stable → Contested fires when legitimacy drops below floor.
    #[test]
    fn test_stable_to_contested_threshold() {
        let params = InstitutionTransitionParams::default();
        let mut inst = Institution::new_test(1, InstitutionState::Stable);

        // At threshold - 1 (still stable)
        let state = evaluate_transition(&mut inst, params.legitimacy_stable_floor + 1, 1_000, &params, 0);
        assert_eq!(state, InstitutionState::Stable);

        // At threshold exactly (transition fires: legitimacy < floor, strictly less)
        let state = evaluate_transition(&mut inst, params.legitimacy_stable_floor, 1_000, &params, 1);
        assert_eq!(state, InstitutionState::Contested);
    }

    /// Transition from Contested → Collapsed fires at contested floor.
    #[test]
    fn test_contested_to_collapsed_threshold() {
        let params = InstitutionTransitionParams::default();
        let mut inst = Institution::new_test(1, InstitutionState::Contested);

        let state = evaluate_transition(&mut inst, params.legitimacy_contested_floor - 1, 1_500, &params, 0);
        assert_eq!(state, InstitutionState::Collapsed);
    }

    /// Capture score exceeding threshold triggers Stable → Captured.
    #[test]
    fn test_capture_score_triggers_transition() {
        let params = InstitutionTransitionParams::default();
        let mut inst = Institution::new_test(1, InstitutionState::Stable);

        // Just below threshold: still Stable
        let state = evaluate_transition(&mut inst, 8_000, params.capture_threshold - 1, &params, 0);
        assert_eq!(state, InstitutionState::Stable);

        // At threshold: → Captured
        let state = evaluate_transition(&mut inst, 8_000, params.capture_threshold, &params, 1);
        assert_eq!(state, InstitutionState::Captured);
    }

    /// Reforming → Stable fires only when both capture_score < capture_low
    /// AND legitimacy >= stable_floor.
    #[test]
    fn test_reforming_to_stable_requires_both_conditions() {
        let params = InstitutionTransitionParams::default();
        let mut inst = Institution::new_test(1, InstitutionState::Reforming);

        // Low capture but legitimacy still below floor: not yet Stable
        let state = evaluate_transition(&mut inst, params.legitimacy_stable_floor - 1, params.capture_low - 1, &params, 0);
        assert_ne!(state, InstitutionState::Stable);

        // Both conditions met
        let state = evaluate_transition(&mut inst, params.legitimacy_stable_floor + 100, params.capture_low - 1, &params, 1);
        assert_eq!(state, InstitutionState::Stable);
    }

    /// Propagation lag: output multipliers do not change until apply_at_tick.
    #[test]
    fn test_propagation_lag_delays_output_change() {
        let params = InstitutionTransitionParams::default();
        let mut inst = Institution::new_test(1, InstitutionState::Stable);
        let lag = params.propagation_lag_ticks as u64;

        // Trigger transition at tick 10
        evaluate_transition(&mut inst, 5_000, 1_000, &params, 10);
        // Before apply_at_tick, outputs still reflect prior state
        let mut outputs = InstitutionOutputs::from_state(InstitutionState::Stable, 0.9, 0.1);
        apply_pending_effects(&mut inst, 10 + lag - 1, &mut outputs);
        assert!(outputs.policy_capacity > 0.9, "Lag: outputs unchanged before apply tick");

        // At apply_at_tick, outputs change
        apply_pending_effects(&mut inst, 10 + lag, &mut outputs);
        // (specific assertion depends on prior and new state)
    }
}

// ============================================================
// CITIZEN LIFECYCLE TESTS
// ============================================================

#[cfg(test)]
mod lifecycle_tests {
    use crate::lifecycle::*;
    use crate::cohort::*;
    use crate::params::*;

    /// Lifecycle transitions respect declared thresholds in policy bundle.
    #[test]
    fn test_active_to_strained_at_energy_strain_threshold() {
        let params = LifecycleTransitionParams::default();
        let mut cohort = CitizenCohort::new_test(1, LifecycleStage::Active);

        cohort.energy_security = params.energy_strain_threshold + 1;
        let stage = evaluate_lifecycle_transition(&mut cohort, &params, 0);
        assert_eq!(stage, LifecycleStage::Active);

        cohort.energy_security = params.energy_strain_threshold - 1;
        let stage = evaluate_lifecycle_transition(&mut cohort, &params, 1);
        assert_eq!(stage, LifecycleStage::Strained);
    }

    /// Monotonic invariant: no cohort may advance more than one stage per tick.
    #[test]
    fn test_monotonic_stage_advance_per_tick() {
        let params = LifecycleTransitionParams::default();
        let mut cohort = CitizenCohort::new_test(1, LifecycleStage::Active);

        // Even with crisis-level inputs, cannot skip Strained
        cohort.energy_security = 0;  // Deep crisis
        cohort.welfare_access = 0;
        cohort.coercion_index = 10_000;
        let stage = evaluate_lifecycle_transition(&mut cohort, &params, 0);
        assert!(
            (stage as u8) <= (LifecycleStage::Active as u8) + 1,
            "Must not advance more than one stage: got {:?}", stage
        );
    }

    /// Active → Retired fires when lifetime_joule_credit >= retirement_threshold.
    #[test]
    fn test_retirement_threshold_triggers_stage() {
        let params = LifecycleTransitionParams::default();
        let mut cohort = CitizenCohort::new_test(1, LifecycleStage::Active);
        cohort.energy_security = 9_500; // Healthy — no stress transitions

        cohort.lifetime_joule_credit = params.retirement_threshold_joules - 1;
        let stage = evaluate_lifecycle_transition(&mut cohort, &params, 0);
        assert_eq!(stage, LifecycleStage::Active);

        cohort.lifetime_joule_credit = params.retirement_threshold_joules;
        let stage = evaluate_lifecycle_transition(&mut cohort, &params, 1);
        assert_eq!(stage, LifecycleStage::Retired);
    }

    /// Dissenting cohorts contribute reform pressure proportional to population.
    #[test]
    fn test_dissenting_cohort_emits_reform_pressure() {
        let params = LifecycleTransitionParams::default();
        let mut cohort = CitizenCohort::new_test(1, LifecycleStage::Dissenting);
        cohort.population = 1000;
        let metrics = assemble_cohort_metrics(&cohort, 0, 0);
        assert!(metrics.reform_pressure_contribution > 0,
            "Dissenting cohort must emit positive reform pressure");
    }
}

// ============================================================
// RETIREMENT POOL TESTS
// ============================================================

#[cfg(test)]
mod retirement_tests {
    use crate::retirement::*;

    /// Pool reserve must never go negative after proration is applied.
    #[test]
    fn test_retirement_pool_reserve_non_negative_after_proration() {
        let mut pool = RetirementPool {
            run_id: 1,
            tick: 0,
            reserve_joules: 1_000,
            retired_population: 100,
            pension_rate_per_tick: 100, // would need 10_000 but only 1_000 available
            total_disbursement_this_tick: 0,
            government_subsidy_this_tick: 0,
            contribution_rate: 800, // 8%
            is_solvent: true,
            solvency_runway_ticks: 0,
        };

        advance_pool_one_tick(&mut pool, 0, 5_000, 100_000);
        assert!(pool.reserve_joules >= 0, "Reserve must never go negative after proration");
    }

    /// Total joules in = total joules out + reserve change (conservation).
    #[test]
    fn test_retirement_pool_joule_conservation() {
        let initial_reserve: i64 = 1_000_000;
        let mut pool = RetirementPool {
            run_id: 1,
            tick: 0,
            reserve_joules: initial_reserve,
            retired_population: 10,
            pension_rate_per_tick: 1_000,
            total_disbursement_this_tick: 0,
            government_subsidy_this_tick: 0,
            contribution_rate: 800,
            is_solvent: true,
            solvency_runway_ticks: 100,
        };

        let subsidy = 5_000i64;
        let inflow = 20_000i64;
        advance_pool_one_tick(&mut pool, subsidy, inflow, 1_000_000);

        let expected_disbursement = i64::min(10 * 1_000, initial_reserve + subsidy + inflow);
        let expected_reserve = initial_reserve + subsidy + inflow - pool.total_disbursement_this_tick;
        assert_eq!(pool.reserve_joules, expected_reserve,
            "Pool joule conservation violated");
    }
}

// ============================================================
// TIME-SERIES TESTS
// ============================================================

#[cfg(test)]
mod timeseries_tests {
    use crate::timeseries::*;

    /// Time-series writer rejects any attempt to update an existing row.
    #[test]
    fn test_timeseries_append_only_rejects_update() {
        let mut store = InMemoryTimeSeriesStore::new();
        let record = make_test_institution_record(1, 0, 42);
        store.append(record.clone()).expect("First write must succeed");

        // Attempt to write same (run_id, tick, institution_id) again
        let result = store.append(record);
        assert!(result.is_err(), "Duplicate key must be rejected");
    }

    /// Deterministic replay reproduces identical hash from time-series records.
    #[test]
    fn test_deterministic_replay_hash() {
        let records_run1 = simulate_n_ticks(10, 12345);
        let records_run2 = simulate_n_ticks(10, 12345);
        assert_eq!(
            compute_replay_hash(&records_run1),
            compute_replay_hash(&records_run2),
            "Identical seed must produce identical time-series hash"
        );
    }
}
```

---

## 16. CIV Sim Integration Notes

### 16.1 Policy Phase Signal Flow

```
Tick N-1 (Metrics Compute) produces:
  → legitimacy_{inst,N-1}
  → capture_score_{inst,N-1}
  → dissenting_population_fraction_{r,N-1}  (reform pressure)

Tick N (Policy Phase, Phase 2) consumes:
  → evaluates institutional FSM transitions
  → applies pending effects whose apply_at_tick <= N
  → emits InstitutionOutputs{policy_capacity, compliance_rate, insurgency_modifier}
  → these signals are passed to Phase 3 (Deterministic Transition)

Tick N (Deterministic Transition, Phase 3) consumes:
  → InstitutionOutputs from all institutions
  → evaluates citizen lifecycle transitions for all cohorts
  → updates retirement pool
  → all transitions produce TimeSeriesRecord entries (not yet written)

Tick N (Metrics Compute, Phase 5) writes:
  → commits all TimeSeriesRecord entries from Phase 3
  → emits events: institution.state_changed.v1, citizen.lifecycle_transitioned.v1, etc.
```

### 16.2 Interaction With Tyranny Metric

The tyranny index computation in `crates/metrics/src/tyranny.rs` reads:
- `InstitutionOutputs.insurgency_modifier` for each institution (amplifies tyranny when institutions are captured or collapsed)
- `citizen_lifecycle` aggregate for dissenting + migrating fraction (signals failed legitimacy)
- `retirement_pool_snapshots.is_solvent` (insolvency drives strained cohort back from Retired, which spikes dissent)

### 16.3 Interaction With Joule Economy (CIV-0107)

The joule economy spec (CIV-0107) defines `EnergyLedger_i` with `lifetime_work_accumulated` and `retirement_status`. This spec maps those fields as:
- `lifetime_work_accumulated` → `CitizenCohort.lifetime_joule_credit` (raw joule sum)
- `retirement_status` → `CitizenCohort.stage` (Active/Retired/etc.)
- `quota_remaining` → drives `energy_security` signal for lifecycle transitions
- `baseline_fulfillment` → drives `welfare_access` signal for lifecycle transitions

The retirement pool in this spec implements the pension mechanics described in CIV-0107 §Retirement Pool: the pool is funded from `ContributionRate * TotalActiveOutput`, and pensions are paid continuously at `pension_rate_per_tier` joules per tick rather than as a lump sum.

### 16.4 Coalition Dynamics

The `coalition_support` component of legitimacy is computed from the ideology aggregation layer (not yet fully specified; pending CIV-0104). Provisional interface:

```rust
/// Trait implemented by the ideology layer, consumed by institutions/legitimacy.rs.
pub trait CoalitionSupport {
    /// Returns fraction of polity [0.0, 1.0] supporting institution m at tick t.
    fn coalition_fraction(&self, institution_id: u64, tick: u64) -> f32;
}
```

Until CIV-0104 is available, coalition support is a constant `0.60` (default policy bundle value) plus a stochastic term seeded from the run RNG.

### 16.5 Snapshot Serialization

Institutional state and cohort state are included in the tick snapshot (CIV-0001 §State Snapshot Protocol) under `world.institutions` and `world.cohorts` respectively. The snapshot format for institutions extends the existing:

```json
{
  "world": {
    "institutions": [
      {
        "id": 5001,
        "name": "Energy Accounting Authority",
        "type": "EnergyAccountingAuthority",
        "state": "Stable",
        "legitimacy": 0.82,
        "capture_score": 0.12,
        "policy_capacity": 1.0,
        "compliance_rate": 0.90,
        "insurgency_modifier": 1.0
      }
    ],
    "cohorts": [
      {
        "id": 101,
        "region_id": 1,
        "stage": "Active",
        "population": 5000,
        "stress_score": 0.15,
        "energy_security": 0.92,
        "lifetime_joule_credit_tj": 0.45,
        "joules_produced_this_tick": 120000000
      }
    ]
  }
}
```

---

## Appendix A: Invariant Summary Table

| ID | Invariant | Enforcement | Test |
|---|---|---|---|
| INV-1 | Legitimacy bounded `[0, 10_000]` | Clamp in compute | `test_legitimacy_bounded` |
| INV-2 | Capture score bounded `[0, 10_000]` | Clamp in accumulate | `test_capture_bounded` |
| INV-3 | Lifecycle transitions monotonic per tick | Max-one-stage check | `test_monotonic_stage_advance_per_tick` |
| INV-4 | Append-only time-series | Unique PK constraint | `test_timeseries_append_only_rejects_update` |
| INV-5 | Retirement pool non-negative after proration | Proration logic | `test_retirement_pool_reserve_non_negative_after_proration` |
| INV-6 | Pending-effect queue monotonic application | BTreeMap iteration | `test_propagation_lag_delays_output_change` |
| INV-7 | Deterministic replay | Same seed → same hash | `test_deterministic_replay_hash` |
| INV-8 | BTreeMap iteration order for institutions | Struct field type | Clippy: `hashmap_in_critical_path` |
| INV-9 | No system time in transition evaluation | No `SystemTime::now()` | Clippy: `system_time_in_sim` |
| INV-10 | No floating-point in stored state | All stored as `i32`/`i64` | Clippy: `floating_point_in_sim` |

---

## Appendix B: Policy Bundle YAML Fragment

```yaml
institutions:
  transition_params:
    legitimacy_stable_floor: 0.60
    legitimacy_contested_floor: 0.35
    legitimacy_collapse_floor: 0.15
    capture_threshold: 0.65
    capture_low: 0.20
    reform_trigger: 0.50
    capture_reform_trigger: 0.75
    reform_recovery_ticks: 3
    propagation_lag_ticks: 5
    detection_window_ticks: 10
    reform_failure_ticks: 3
  legitimacy_weights:
    enforcement_alignment: 0.25
    coalition_support: 0.30
    transfer_fairness: 0.20
    external_pressure_penalty: 0.10
    corruption_penalty: 0.15
    scarcity_drag: 0.10

citizens:
  lifecycle_params:
    energy_strain_threshold: 0.85
    energy_crisis_threshold: 0.50
    welfare_crisis_threshold: 0.40
    energy_recovery_threshold: 0.90
    welfare_recovery_threshold: 0.75
    coercion_flight_threshold: 0.70
    mobility_barrier_threshold: 0.30
    coercion_relief_threshold: 0.35
    retirement_threshold_joules: 1_000_000_000_000  # 1 TJ

retirement_pool:
  contribution_rate: 0.08
  base_subsidy_share: 0.02
  scarcity_pressure_supplement_rate: 0.01
  target_solvency_runway_ticks: 520   # 10 sim-years at weekly ticks
  full_tier_pension_multiplier: 1.00
  reduced_tier_pension_multiplier: 0.75
  minimum_tier_pension_multiplier: 0.50

timeseries:
  canonical_retention: indefinite
  noncanonical_retention_days: 30
  aggregation_windows: [10, 100]
```

---

**Version History:**
- v1.0 (2026-02-21): Full expansion from 33-line stub to complete engineering-grade spec. Covers institutional FSM, citizen lifecycle FSM, legitimacy model, capture dynamics, retirement pool mechanics, time-series architecture, propagation lag, conservation invariants, acceptance test suite, and full database DDL.
- v1.1 (2026-02-21): Extended with: full citizen state vector, productivity curve, energy debt mechanics, multi-generation dynamics, institution formation/dissolution, extended time-series architecture (partitioning, incremental aggregation, replay protocol), shadow institution mechanics, 6 additional events, 4 additional SQL tables, 8 additional test stubs.

---

## 17. Extended Citizen State Model

### 17.1 Full Citizen State Vector

The `CitizenCohort` struct defined in §5.4 carries only the core lifecycle signals. The full citizen state vector required for productivity modeling, generational dynamics, shadow state interaction, and ideology linkage (CIV-0106) extends that struct with the following additional fields.

All new fixed-point fields use the same `i32` scaled by `10_000` convention (`[0, 10_000]` represents `[0.0, 1.0]`). Joule fields remain `i64` raw joules.

```rust
/// Extended citizen cohort state including health dynamics, energy debt,
/// social trust, alienation, and ideology linkage.
/// This struct augments CitizenCohort for cohorts where the full simulation
/// depth is enabled. It is stored alongside CitizenCohort in the cohort registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CitizenHealthState {
    pub cohort_id: u64,
    /// Physical and mental health stock. Fixed-point [0, 10_000].
    /// Decays with age, stress, and inadequate welfare. Recovers with
    /// welfare coverage and public health surge capacity investment.
    pub health: i32,
    /// Accumulated psychological and economic stress. Fixed-point [0, 10_000].
    /// High stress accelerates health decay and lifecycle transitions.
    /// Driven by energy deficit, coercion exposure, and social instability.
    pub stress: i32,
    /// Current tick joule balance for this cohort: joules_earned - joules_owed
    /// this tick. Positive = surplus, negative = deficit drawing on credit.
    /// Raw joules, i64.
    pub joule_balance: i64,
    /// Accumulated energy debt: joules borrowed from future credit that have
    /// not been repaid. Grows when joule_balance < 0 and credit is available.
    /// Triggers default cascade when it exceeds joule_debt_default_threshold.
    /// Raw joules, i64. Always >= 0.
    pub joule_debt: i64,
    /// Social trust level. Fixed-point [0, 10_000].
    /// Represents trust in institutions, neighbors, and the social contract.
    /// Feeds into coalition_support computation for institutional legitimacy.
    /// Linked to CIV-0106 ideology layer via TrustInInstitutions signal.
    pub social_trust: i32,
    /// Alienation index. Fixed-point [0, 10_000].
    /// Rises when energy debt is high, coercion is elevated, or welfare fails.
    /// High alienation accelerates Dissenting → Migrating transition and
    /// contributes to rebellion risk tracked by CIV-0106.
    pub alienation_index: i32,
    /// Ideology vector index into the ideology layer (CIV-0106).
    /// This is a foreign key: the full ideology vector lives in the ideology
    /// crate. This field enables the join between citizen cohort state and
    /// ideological profile without duplicating the 8-dimensional vector here.
    pub ideology_vector_id: u64,
}
```

### 17.2 Productivity Curve

Joule output per tick for a cohort is a function of age band, health, and stress. The productivity curve is deterministic and fixed-point-safe: it produces an `i64` joule output per citizen per tick, which is then scaled by cohort population.

**Formula:**

```
joule_output_per_citizen_per_tick = base_rate
    × age_factor(age_band)
    × health_factor(health)
    × stress_factor(stress)
    × skill_factor(skill_quintile)
    × policy_capacity_factor(institution_outputs)
```

All factors are dimensionless multipliers normalized around `1.0`. Computation is done in integer arithmetic using scaled intermediate values.

```rust
/// Productivity curve parameters. Loaded from policy bundle YAML.
/// All factors stored as i32 fixed-point scaled 1/10_000 (i.e., 10_000 = 1.0).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProductivityCurveParams {
    /// Base joule production rate per citizen per tick (raw joules, i64).
    /// Represents median peak-age, full-health, zero-stress output.
    /// Default: 20_000_000_000 (20 GJ/tick at weekly tick cadence, ~1e12/yr).
    pub base_rate_joules_per_tick: i64,

    /// Age factor by band. Fixed-point [0, 10_000].
    /// Youth: skill investment phase, output reduced.
    /// Working: peak output.
    /// LateCareer: moderate decline.
    /// Elder: post-retirement maintenance only (output = 0 for Retired stage).
    pub age_factor_youth: i32,        // default 4_500 (= 0.45)
    pub age_factor_working: i32,      // default 10_000 (= 1.00)
    pub age_factor_late_career: i32,  // default 8_200 (= 0.82)
    pub age_factor_elder: i32,        // default 2_500 (= 0.25; only pre-retirement)

    /// Health factor: maps health stock to output multiplier.
    /// health_factor = (health / 10_000)^health_exponent, computed in fixed-point.
    /// Default exponent: 0.6 (stored as 6_000 / 10_000).
    pub health_exponent_fp: i32,      // default 6_000

    /// Stress penalty factor: linear attenuation at high stress.
    /// stress_factor = 1.0 - stress_drag_coeff * (stress / 10_000).
    /// Default drag: 0.40 (stored as 4_000 / 10_000).
    pub stress_drag_coeff_fp: i32,    // default 4_000

    /// Skill quintile multipliers. Fixed-point [0, 10_000].
    /// Quintile 0 (lowest): reduced output.
    /// Quintile 4 (highest): elevated output.
    pub skill_factor: [i32; 5],       // default [6_000, 8_000, 10_000, 12_500, 16_000]
}

/// Compute joule output for a single tick for a cohort.
/// All intermediate arithmetic uses i64 to avoid overflow.
/// Returns joules produced by the entire cohort this tick.
pub fn compute_cohort_joule_output(
    cohort: &CitizenCohort,
    health_state: &CitizenHealthState,
    params: &ProductivityCurveParams,
    institution_policy_capacity_fp: i32,  // [0, 10_000]
) -> i64 {
    if cohort.stage == LifecycleStage::Retired {
        return 0;
    }

    let age_factor = match cohort.age_band {
        AgeBand::Youth      => params.age_factor_youth as i64,
        AgeBand::Working    => params.age_factor_working as i64,
        AgeBand::LateCareer => params.age_factor_late_career as i64,
        AgeBand::Elder      => params.age_factor_elder as i64,
    };

    // health_factor = health^exponent approximated as linear in [0,1] range
    // for fixed-point: health_factor_fp = health * health_exponent_fp / 10_000
    // (simplified monotone proxy; full power function requires lookup table)
    let health_fp = health_state.health as i64;
    let health_factor_fp = (health_fp * params.health_exponent_fp as i64) / 10_000
        + (10_000 - params.health_exponent_fp as i64) * 10_000 / 10_000;
    // Clamp to [0, 10_000]
    let health_factor_fp = health_factor_fp.clamp(0, 10_000);

    // stress_factor = 10_000 - stress_drag * stress
    let stress_fp = health_state.stress as i64;
    let stress_factor_fp = (10_000
        - (params.stress_drag_coeff_fp as i64 * stress_fp) / 10_000)
        .clamp(0, 10_000);

    let skill_factor_fp = params.skill_factor[cohort.skill_quintile as usize] as i64;
    let policy_factor_fp = institution_policy_capacity_fp as i64;

    // Chain multiply: each factor is in [0, 10_000] units of 1/10_000
    // To avoid 5x overflow: compute step by step, divide by 10_000 at each step
    let base = params.base_rate_joules_per_tick;
    let after_age     = base * age_factor / 10_000;
    let after_health  = after_age * health_factor_fp / 10_000;
    let after_stress  = after_health * stress_factor_fp / 10_000;
    let after_skill   = after_stress * skill_factor_fp / 10_000;
    let after_policy  = after_skill * policy_factor_fp / 10_000;

    // Scale by population
    after_policy * cohort.population as i64
}
```

### 17.3 Health Dynamics

Health evolves each tick according to welfare coverage, public health surge capacity, and stress. Transitions are deterministic given inputs.

```
health_{t+1} = clamp(
    health_t
    + welfare_recovery_rate * welfare_access_t
    + surge_capacity_bonus_t
    - age_decay_rate(age_band)
    - stress_health_drag * stress_t
    - shock_damage_t,
    0, 10_000
)
```

**Parameters:**

| Parameter | Description | Default (fixed-point) |
|---|---|---|
| `welfare_recovery_rate` | Health gained per tick at full welfare access | `80` (= 0.0080/tick) |
| `age_decay_rate_youth` | Health decay per tick for Youth band | `10` (= 0.001/tick) |
| `age_decay_rate_working` | Health decay per tick for Working band | `20` |
| `age_decay_rate_late_career` | Health decay per tick for LateCareer band | `60` |
| `age_decay_rate_elder` | Health decay per tick for Elder band | `120` |
| `stress_health_drag` | Health lost per tick per unit stress | `40` (coeff, applied as `40 * stress / 10_000`) |
| `surge_capacity_bonus` | Additional recovery when public health capacity > 0.7 | `50` (conditional) |

Surge capacity is a regional public health parameter from CIV-0100 (Economy) representing hospital/clinic throughput. When surge capacity exceeds `surge_threshold_fp` (default `7_000`), the `surge_capacity_bonus` is added to recovery.

```rust
/// Health state update for one tick.
pub fn update_health(
    health_state: &mut CitizenHealthState,
    cohort: &CitizenCohort,
    surge_capacity_fp: i32,
    shock_damage_fp: i32,
    params: &HealthDynamicsParams,
) {
    let welfare_gain = (params.welfare_recovery_rate as i64
        * cohort.welfare_access as i64) / 10_000;

    let age_decay = match cohort.age_band {
        AgeBand::Youth      => params.age_decay_youth as i64,
        AgeBand::Working    => params.age_decay_working as i64,
        AgeBand::LateCareer => params.age_decay_late_career as i64,
        AgeBand::Elder      => params.age_decay_elder as i64,
    };

    let stress_drag = (params.stress_health_drag as i64
        * health_state.stress as i64) / 10_000;

    let surge_bonus = if surge_capacity_fp >= params.surge_threshold_fp {
        params.surge_capacity_bonus as i64
    } else {
        0
    };

    let new_health = health_state.health as i64
        + welfare_gain
        + surge_bonus
        - age_decay
        - stress_drag
        - shock_damage_fp as i64;

    health_state.health = new_health.clamp(0, 10_000) as i32;
}
```

### 17.4 Energy Debt Mechanics at Citizen Level

When a cohort's `joule_balance` for a tick is negative (joules owed exceed joules earned), the deficit is first covered by drawing on available energy credit (issued by the government subsidy mechanism). If credit is exhausted, debt accumulates in `joule_debt`.

**Debt accumulation:**

```
if joule_balance_t < 0:
    available_credit = government_credit_pool_t / cohort.population
    covered = min(-joule_balance_t, available_credit)
    joule_debt_{t+1} = joule_debt_t + (-joule_balance_t - covered)
else:
    joule_debt_{t+1} = max(0, joule_debt_t - joule_balance_t * debt_repayment_rate)
```

**Default trigger:** When `joule_debt > joule_debt_default_threshold` (policy parameter, default `5e10` joules = 50 GJ), a `citizen.debt_defaulted.v1` event fires. Default effects:

1. `stress_score` increases by `debt_default_stress_spike` (default `2_500`, = +0.25).
2. `alienation_index` increases by `debt_default_alienation_spike` (default `1_500`).
3. `welfare_access` is reduced by `debt_default_welfare_penalty` for the following tick.
4. If stress after spike >= `energy_crisis_threshold`, lifecycle transition to `Dissenting` is eligible that tick.

**Debt collection effects on stress:**

Each tick that `joule_debt > 0`, a continuous stress increment applies:

```
debt_stress_pressure = debt_stress_base_rate * (joule_debt / joule_debt_default_threshold)
stress_{t+1} = clamp(stress_t + debt_stress_pressure, 0, 10_000)
```

Default `debt_stress_base_rate`: `50` fixed-point units per tick (= 0.005/tick at full debt threshold).

```rust
/// Parameters governing energy debt mechanics.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EnergyDebtParams {
    /// Joule threshold above which debt default event fires.
    pub joule_debt_default_threshold: i64,           // default 50_000_000_000 (50 GJ)
    /// Fraction of surplus joules applied to debt repayment per tick.
    /// Fixed-point [0, 10_000]. Default 3_000 (= 30% of surplus goes to repayment).
    pub debt_repayment_rate_fp: i32,                 // default 3_000
    /// Stress spike on default event. Fixed-point delta [0, 10_000].
    pub debt_default_stress_spike: i32,              // default 2_500
    /// Alienation spike on default event. Fixed-point delta [0, 10_000].
    pub debt_default_alienation_spike: i32,          // default 1_500
    /// Welfare access penalty for the tick following a default. Fixed-point.
    pub debt_default_welfare_penalty: i32,           // default 2_000
    /// Continuous stress pressure coefficient from outstanding debt.
    /// stress_delta = base_rate * (joule_debt / threshold). Fixed-point.
    pub debt_stress_base_rate: i32,                  // default 50
}

/// Advance debt state one tick. Returns true if default event fires this tick.
pub fn advance_debt(
    health_state: &mut CitizenHealthState,
    joule_balance: i64,
    government_credit_per_citizen: i64,
    params: &EnergyDebtParams,
) -> bool {
    if joule_balance < 0 {
        let deficit = -joule_balance;
        let covered = deficit.min(government_credit_per_citizen);
        health_state.joule_debt = health_state.joule_debt.saturating_add(deficit - covered);
    } else {
        let repayment = (joule_balance
            * params.debt_repayment_rate_fp as i64) / 10_000;
        health_state.joule_debt = health_state.joule_debt.saturating_sub(repayment).max(0);
    }

    // Continuous stress from outstanding debt
    if health_state.joule_debt > 0 {
        let pressure = (params.debt_stress_base_rate as i64
            * health_state.joule_debt) / params.joule_debt_default_threshold;
        health_state.stress = (health_state.stress as i64 + pressure)
            .clamp(0, 10_000) as i32;
    }

    // Default event
    if health_state.joule_debt >= params.joule_debt_default_threshold {
        health_state.stress = (health_state.stress as i64
            + params.debt_default_stress_spike as i64)
            .clamp(0, 10_000) as i32;
        health_state.alienation_index = (health_state.alienation_index as i64
            + params.debt_default_alienation_spike as i64)
            .clamp(0, 10_000) as i32;
        return true;
    }
    false
}
```

### 17.5 Alienation and Social Trust Update

Alienation and social trust are updated each tick based on experienced coercion, energy security, and welfare access. They feed into the ideology vector via CIV-0106 and into institutional legitimacy via the `coalition_support` component.

```
social_trust_{t+1} = clamp(
    social_trust_t
    + trust_recovery_rate * welfare_access_t
    - trust_coercion_drag * coercion_index_t
    - trust_alienation_drag * alienation_index_t,
    0, 10_000
)

alienation_{t+1} = clamp(
    alienation_t
    + alienation_debt_pressure * (joule_debt / threshold)
    + alienation_coercion_rate * coercion_index_t
    - alienation_recovery_rate * energy_security_t,
    0, 10_000
)
```

These are computed in `crates/citizens/src/health.rs` after the energy debt advance.

---

## 18. Multi-Generation Dynamics

### 18.1 Birth Rate Model

Birth rate is not a fixed demographic constant. It responds to welfare security, energy surplus, and institutional stability. A cohort with stable energy security and strong institutional legitimacy produces children at a higher rate than one experiencing scarcity or coercion.

**Birth rate formula:**

```
birth_rate_weekly_t = base_birth_rate
    × welfare_security_factor(welfare_access_t)
    × energy_surplus_factor(energy_security_t)
    × institutional_stability_factor(avg_legitimacy_t)
    × (1 - alienation_drag * alienation_index_t / 10_000)
```

Factors are clamped to `[0.5×base, 2.0×base]` to prevent unrealistic extremes.

```rust
/// Demographics parameters for birth and death rate computation.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CohortDemographics {
    pub cohort_id: u64,
    pub region_id: u32,
    pub age_band: AgeBand,
    /// Tick at which this cohort's population was last revised by birth/death.
    pub last_revised_tick: u64,

    /// Base weekly birth rate per 1000 citizens in Working age band.
    /// Fixed-point [0, 10_000] representing births per citizen per tick.
    /// Default: 18 per 1000 per 52 ticks = ~18/52000 per tick ≈ 3 per 10_000.
    pub base_birth_rate_fp: i32,               // default 3

    /// Welfare security factor: multiplier at full welfare access.
    /// Range [0.5×base, 2.0×base]. Default midpoint at welfare_access = 7_500.
    pub welfare_birth_multiplier_at_full: i32, // default 12_000 (= 1.20)

    /// Energy surplus factor: multiplier when energy_security > threshold.
    pub energy_surplus_birth_bonus_fp: i32,    // default 1_500 (= +0.15 at ES > 0.90)

    /// Alienation drag on birth rate. Fixed-point coefficient.
    pub alienation_birth_drag_fp: i32,         // default 3_000 (= 0.30 at max alienation)

    /// Base weekly death rate per citizen in each age band.
    /// Fixed-point. Age-stratified.
    pub death_rate_youth_fp: i32,              // default 2  (= 0.0002/tick)
    pub death_rate_working_fp: i32,            // default 3
    pub death_rate_late_career_fp: i32,        // default 12
    pub death_rate_elder_fp: i32,              // default 80
    /// Additional death rate from low health stock.
    pub health_death_multiplier_fp: i32,       // default 5_000 (= 0.50 at health=0)
}

/// Compute net population change for a cohort this tick.
/// Returns (births, deaths) as raw citizen counts.
pub fn compute_demographic_change(
    cohort: &CitizenCohort,
    health_state: &CitizenHealthState,
    avg_legitimacy_fp: i32,
    params: &CohortDemographics,
) -> (u32, u32) {
    // Only Working band generates births in this model.
    let births = if cohort.age_band == AgeBand::Working {
        let welfare_factor = 10_000i64
            + (params.welfare_birth_multiplier_at_full as i64 - 10_000)
            * cohort.welfare_access as i64 / 10_000;
        let energy_bonus = if cohort.energy_security > 9_000 {
            params.energy_surplus_birth_bonus_fp as i64
        } else { 0 };
        let alien_drag = params.alienation_birth_drag_fp as i64
            * health_state.alienation_index as i64 / 10_000;
        let legit_factor = 5_000i64 + avg_legitimacy_fp as i64 / 2; // [5000, 10000]
        let rate = (params.base_birth_rate_fp as i64
            * welfare_factor / 10_000
            * legit_factor / 10_000)
            + energy_bonus
            - alien_drag;
        let rate = rate.clamp(0, params.base_birth_rate_fp as i64 * 2);
        (cohort.population as i64 * rate / 10_000).max(0) as u32
    } else { 0 };

    // Deaths: age-stratified, modulated by health
    let base_death_rate = match cohort.age_band {
        AgeBand::Youth      => params.death_rate_youth_fp as i64,
        AgeBand::Working    => params.death_rate_working_fp as i64,
        AgeBand::LateCareer => params.death_rate_late_career_fp as i64,
        AgeBand::Elder      => params.death_rate_elder_fp as i64,
    };
    // Health penalty: at health=0 death rate multiplied by (1 + health_death_multiplier)
    let health_penalty = params.health_death_multiplier_fp as i64
        * (10_000 - health_state.health as i64) / 10_000;
    let effective_death_rate = base_death_rate + health_penalty / 10_000;
    let deaths = (cohort.population as i64 * effective_death_rate / 10_000).max(0) as u32;

    (births, deaths)
}
```

### 18.2 Youth Cohort: Skill Development Phase

Youth cohorts (AgeBand::Youth) do not produce joules at standard rates. Instead, joule investment flows into them in the form of educational spending, which accumulates human capital that will raise their `skill_quintile` when they age into the Working band.

**Human capital accumulation:**

```
human_capital_{t+1} = human_capital_t
    + education_investment_joules_t * education_efficiency_fp / 10_000
    - human_capital_decay_rate
```

At band transition (Youth → Working), `skill_quintile` is assigned based on percentile rank of `human_capital` across the regional youth cohort population. This makes education investment a direct determinant of future productivity.

```rust
/// Human capital state for a youth cohort.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct YouthHumanCapital {
    pub cohort_id: u64,
    /// Accumulated human capital units (dimensionless index).
    /// Scaled as i64 to allow joule-denominated investment flows.
    pub human_capital: i64,
    /// Total joules invested in education for this cohort this tick.
    pub education_investment_joules: i64,
    /// Education system efficiency. Fixed-point [0, 10_000].
    /// Determined by institutional policy capacity of the education authority.
    pub education_efficiency_fp: i32,
    /// Tick when this youth cohort transitions to Working band.
    pub working_transition_tick: u64,
}
```

### 18.3 Elder Cohort: Institutional Memory Contribution

Post-retirement Elder cohorts (AgeBand::Elder, LifecycleStage::Retired) do not produce joules but contribute institutional memory, which acts as a governance quality modifier. Elders who engaged in governance, administration, or civic leadership accumulate `institutional_memory_credit` that reduces governance quality decay.

**Governance quality modifier from elder institutional memory:**

```
governance_quality_modifier_t = min(
    elder_memory_cap,
    elder_cohort_population_t * institutional_memory_credit_per_elder
        * institutional_alignment_fp / 10_000
)
```

This modifier is added to the base governance quality computed in the metrics engine. It represents the value of accumulated experience, precedent knowledge, and institutional norms transmitted by active elders. The modifier degrades as elder cohort population declines (mortality).

```rust
/// Elder institutional memory state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ElderInstitutionalMemory {
    pub cohort_id: u64,
    /// Accumulated institutional memory credit per elder citizen (fixed-point i32).
    /// Higher for cohorts that spent more ticks in Active governance roles.
    pub institutional_memory_credit_per_elder: i32,  // [0, 10_000]
    /// Alignment of this elder cohort's norms with current institutions.
    /// Low alignment reduces effective contribution (cultural friction).
    pub institutional_alignment_fp: i32,             // [0, 10_000]
    /// Maximum contribution cap. Fixed-point governance quality delta.
    pub elder_memory_cap_fp: i32,                    // default 500 (= 0.05 max)
}
```

### 18.4 Generational Joule Wealth Transfer

Upon citizen death (cohort member mortality), accumulated `lifetime_joule_credit` above the maintenance floor may be transferred to successor cohorts (designated beneficiary cohorts, typically Youth or Working bands in the same region). This is the inheritance mechanic.

Inheritance is disabled by default. When enabled via policy bundle, it introduces wealth concentration dynamics that increase inequality over time unless progressive inheritance tax is applied.

**Transfer formula:**

```
transferable_joules = max(0,
    lifetime_joule_credit_per_deceased - minimum_floor_joules
)
inheritance_tax_joules = transferable_joules * inheritance_tax_rate_fp / 10_000
net_transfer_joules = transferable_joules - inheritance_tax_joules

beneficiary_joule_credit += net_transfer_joules * beneficiary_fraction_fp / 10_000
retirement_pool_contribution += inheritance_tax_joules
```

The wealth concentration index tracks the Gini of `lifetime_joule_credit` across all living cohorts and is used as an inequality signal in the legitimacy computation.

```rust
/// Generational transfer record. Written to time-series on each mortality event
/// that produces a non-zero transfer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationalTransfer {
    pub run_id: u64,
    pub tick: u64,
    /// Source cohort (deceased members).
    pub source_cohort_id: u64,
    /// Destination cohort (beneficiaries).
    pub beneficiary_cohort_id: u64,
    /// Total joules transferred (after inheritance tax). Raw joules.
    pub net_transfer_joules: i64,
    /// Inheritance tax collected to retirement pool. Raw joules.
    pub tax_collected_joules: i64,
    /// Number of citizens whose deaths generated this transfer.
    pub deceased_count: u32,
    /// Wealth concentration index (Gini × 10_000) at transfer tick.
    pub wealth_concentration_index: i32,
}

/// Parameters controlling generational transfer.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenerationalTransferParams {
    /// Whether inheritance is enabled at all.
    pub inheritance_enabled: bool,               // default false
    /// Minimum joule credit floor: amount retained as societal commons, not transferable.
    /// Raw joules. Default: same as retirement_threshold_joules (100% clawback option).
    pub minimum_floor_joules: i64,               // default 0 (if enabled)
    /// Fraction of deceased's transferable credit sent to beneficiary cohort.
    /// Fixed-point [0, 10_000]. Remainder goes to retirement pool.
    pub beneficiary_fraction_fp: i32,            // default 8_000 (= 80%)
    /// Inheritance tax rate on transferable amount. Fixed-point [0, 10_000].
    pub inheritance_tax_rate_fp: i32,            // default 2_500 (= 25%)
}
```

---

## 19. Institution Formation and Dissolution

### 19.1 Institution Creation Conditions

New institutions may be created during a run via governance events. Creation requires:

1. **Coalition support threshold**: At least `creation_coalition_threshold` fraction of the polity (as measured by coalition support aggregation) must actively endorse the new institution.
2. **Resource commitment**: The founding coalition must commit `charter_resource_joules` from the public energy budget as the institution's initial operating capital.
3. **Charter ratification**: A `governance_event` of type `institution_charter_ratified` must be recorded, which acts as the creation trigger.
4. **No duplicate domain**: No existing non-collapsed institution may hold the same `InstitutionType` domain without a merger protocol (§19.2).

When all conditions are met, the institution enters the `Reforming` state at creation tick with `legitimacy = creation_initial_legitimacy_fp` (default `7_000`) and `capture_score = 0`.

```rust
/// Charter document recorded at institution creation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstitutionCharter {
    pub institution_id: u64,
    pub run_id: u64,
    /// Tick at which the charter was ratified and institution created.
    pub ratified_tick: u64,
    /// Type of institution created.
    pub institution_type: InstitutionType,
    /// Name of the new institution.
    pub name: String,
    /// Coalition support fraction at ratification. Fixed-point [0, 10_000].
    pub coalition_support_at_ratification: i32,
    /// Joules committed as initial operating capital.
    pub charter_resource_joules: i64,
    /// Initial legitimacy score. Fixed-point [0, 10_000].
    pub initial_legitimacy_fp: i32,
    /// Tick at which the charter expires (0 = no expiry).
    /// If expired without renewal, institution enters Collapsed state.
    pub charter_expiry_tick: u64,
    /// Enabling institutions: IDs of institutions that must be Stable
    /// for this institution to operate at full policy capacity.
    pub dependency_ids: Vec<u64>,
}
```

### 19.2 Institution Merger and Split Mechanics

**Merger trigger conditions:**

A merger between institutions A and B fires when:
- Both institutions have `InstitutionType` in overlapping policy domains (defined by type taxonomy).
- At least one institution is in `Contested` or `Reforming` state.
- `reform_pressure` exceeds `merger_reform_threshold` for at least `merger_window_ticks` (default 5) consecutive ticks.
- A `governance_event` of type `institution_merger_proposed` has been queued.

**Merger mechanics:**

```
merged_legitimacy = weighted_avg(A.legitimacy, B.legitimacy,
                                  weights = [A.coalition_support, B.coalition_support])
merged_capture    = max(A.capture_score, B.capture_score)   // conservative: worst of both
merged_state      = Reforming (always; merger is a reform event)
```

Liabilities (pending effects, outstanding capture) transfer to the merged institution. Assets (joule reserve, pending positive effects) are summed and assigned to the new institution ID. The old institution IDs enter `Collapsed` state (sentinel rows in time-series) and are never reused.

**Split trigger conditions:**

A split fires when:
- A `Captured` or `Stable` institution has `capture_score` asymmetrically concentrated in one policy domain (split threshold: concentration > `capture_domain_split_threshold`, default `0.70`).
- A `governance_event` of type `institution_split_proposed` has been queued.

The original institution retains the primary domain; the new institution takes the secondary domain. Capture score is split proportionally. Legitimacy is divided by `split_legitimacy_penalty_factor` (default `0.85`) applied to both successor institutions.

### 19.3 Institution Type Taxonomy

The extended `InstitutionType` enum covers all governance domain categories. Each type carries a `capability_multiplier` array that modulates which policy levers the institution amplifies.

```rust
/// Extended institution type taxonomy.
/// Each variant represents a distinct governance domain.
/// Type-specific capability multipliers are loaded from the policy bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
         serde::Serialize, serde::Deserialize)]
pub enum InstitutionType {
    /// Legislative/executive governance. High coalition support sensitivity.
    Government,
    /// Judicial branch. High enforcement alignment weight.
    Judiciary,
    /// Military and internal security. High capture risk. High enforcement capacity.
    Military,
    /// Civil society organizations, NGOs, community bodies. Low capture risk.
    CivilSociety,
    /// Religious institutions. Ideology vector influence; variable legitimacy dynamics.
    Religious,
    /// Economic regulatory bodies. Market rule enforcement. High rent pressure exposure.
    EconomicRegulatory,
    /// Original types preserved for compatibility:
    RightsAuthority,
    MarketRegulator,
    EnergyAccountingAuthority,
    GovernanceIntegrityCouncil,
    MetricReviewBoard,
    LaborBoard,
    CentralBank,
    SecurityApparatus,
}

/// Capability multiplier set for an institution type.
/// Determines which policy outputs the institution amplifies beyond the base
/// computed from institutional state.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct InstitutionTypeCapability {
    pub institution_type: InstitutionType,
    /// Multiplier on policy_capacity for this type. Fixed-point [0, 20_000].
    pub policy_capacity_multiplier_fp: i32,   // default 10_000 (= 1.0 neutral)
    /// Multiplier on enforcement output. Fixed-point [0, 20_000].
    pub enforcement_multiplier_fp: i32,       // varies by type
    /// Multiplier on legitimacy accumulation rate. Fixed-point [0, 20_000].
    pub legitimacy_accumulation_fp: i32,      // varies by type
    /// Type-specific capture susceptibility baseline override.
    /// If set, overrides the global `type_susceptibility_factor` from §10.1.
    pub capture_susceptibility_override_fp: Option<i32>,
}
```

**Default capability multipliers by type:**

| Type | Policy Capacity | Enforcement | Legitimacy Accum. | Capture Susceptibility |
|---|---|---|---|---|
| `Government` | 1.20 | 0.80 | 1.10 | 0.55 |
| `Judiciary` | 0.90 | 1.30 | 1.20 | 0.40 |
| `Military` | 0.70 | 1.80 | 0.70 | 0.80 |
| `CivilSociety` | 0.60 | 0.30 | 1.40 | 0.25 |
| `Religious` | 0.50 | 0.20 | 1.30 | 0.35 |
| `EconomicRegulatory` | 1.10 | 1.10 | 0.90 | 0.70 |

### 19.4 Cross-Institution Dependency Graph

Institutions form a directed dependency graph: some institutions require others to be operational (non-Collapsed, non-Captured) in order to function at full `policy_capacity`. This is the `dependency_ids` field in `InstitutionCharter`.

**Dependency effects on policy capacity:**

```
effective_policy_capacity = base_policy_capacity
    × PRODUCT(dependency_availability(dep_id) for dep_id in dependency_ids)

dependency_availability(dep_id) =
    IF institution[dep_id].state == Collapsed: 0.0
    IF institution[dep_id].state == Captured:  0.5
    IF institution[dep_id].state == Contested: 0.75
    ELSE: 1.0
```

The dependency graph is acyclic (DAC). Cycles are rejected at charter ratification time.

```rust
/// A single dependency edge in the cross-institution dependency graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstitutionDependency {
    pub dependent_institution_id: u64,
    pub required_institution_id: u64,
    /// Minimum state of the required institution for full availability.
    /// If required institution's state is worse than this, availability < 1.0.
    pub minimum_required_state: InstitutionState,
    /// Availability when required institution is exactly at minimum_required_state.
    /// Fixed-point [0, 10_000]. Collapsed always → 0 regardless.
    pub availability_at_minimum_fp: i32,    // default 7_500
}

/// Evaluate effective policy capacity accounting for all dependency availabilities.
pub fn effective_policy_capacity(
    institution_id: u64,
    base_capacity_fp: i32,
    dependencies: &[InstitutionDependency],
    institution_states: &std::collections::BTreeMap<u64, InstitutionState>,
) -> i32 {
    let mut capacity = base_capacity_fp as i64;

    for dep in dependencies {
        if dep.dependent_institution_id != institution_id { continue; }
        let req_state = institution_states
            .get(&dep.required_institution_id)
            .copied()
            .unwrap_or(InstitutionState::Collapsed);

        let avail = match req_state {
            InstitutionState::Collapsed  => 0i64,
            InstitutionState::Captured   => 5_000,
            InstitutionState::Contested  => 7_500,
            InstitutionState::Reforming  => 8_500,
            InstitutionState::Stable     => 10_000,
        };
        capacity = capacity * avail / 10_000;
    }

    capacity.clamp(0, 10_000) as i32
}
```

---

## 20. Time-Series Extended Architecture

### 20.1 Partitioning Strategy

The base schema uses `(run_id, tick, entity_id)` as the composite primary key. For runs exceeding 10,000 ticks with more than 20 institutions and 100 cohorts, query performance requires explicit range partitioning on `tick`.

```sql
-- ============================================================
-- PARTITIONED INSTITUTION STATES (for large runs)
-- Partition by tick ranges: [0, 1000), [1000, 2000), etc.
-- ============================================================
CREATE TABLE institution_states_partitioned (
    run_id              BIGINT      NOT NULL,
    tick                BIGINT      NOT NULL,
    institution_id      BIGINT      NOT NULL,
    institution_type    TEXT        NOT NULL,
    state               SMALLINT    NOT NULL,
    legitimacy          INT         NOT NULL,
    capture_score       INT         NOT NULL,
    policy_capacity_fp  INT         NOT NULL,
    compliance_rate_fp  INT         NOT NULL,
    insurgency_mod_fp   INT         NOT NULL,
    ticks_in_state      INT         NOT NULL,
    is_canonical_run    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (tick);

-- Create partitions at 1000-tick intervals.
-- In practice, generate these programmatically at run start.
CREATE TABLE institution_states_p0000
    PARTITION OF institution_states_partitioned
    FOR VALUES FROM (0) TO (1000);

CREATE TABLE institution_states_p1000
    PARTITION OF institution_states_partitioned
    FOR VALUES FROM (1000) TO (2000);

-- Index on each partition automatically inherits from parent:
CREATE INDEX idx_inst_part_run_state
    ON institution_states_partitioned (run_id, state, tick);

CREATE INDEX idx_inst_part_run_inst
    ON institution_states_partitioned (run_id, institution_id, tick);

-- ============================================================
-- PARTITIONED CITIZEN LIFECYCLE (same strategy)
-- ============================================================
CREATE TABLE citizen_lifecycle_partitioned (
    run_id              BIGINT      NOT NULL,
    tick                BIGINT      NOT NULL,
    cohort_id           BIGINT      NOT NULL,
    region_id           INT         NOT NULL,
    skill_quintile      SMALLINT    NOT NULL,
    age_band            SMALLINT    NOT NULL,
    stage               SMALLINT    NOT NULL,
    population          INT         NOT NULL,
    stress_score        INT         NOT NULL,
    energy_security     INT         NOT NULL,
    welfare_access      INT         NOT NULL,
    coercion_index      INT         NOT NULL,
    mobility_constraint INT         NOT NULL,
    joules_produced     BIGINT      NOT NULL,
    pension_draw_joules BIGINT      NOT NULL,
    reform_pressure_fp  INT         NOT NULL,
    lifetime_joule_credit BIGINT    NOT NULL,
    ticks_in_stage      INT         NOT NULL,
    -- Extended fields from §17:
    health              INT         NOT NULL DEFAULT 10_000,
    stress_extended     INT         NOT NULL DEFAULT 0,
    joule_balance       BIGINT      NOT NULL DEFAULT 0,
    joule_debt          BIGINT      NOT NULL DEFAULT 0,
    social_trust        INT         NOT NULL DEFAULT 7_000,
    alienation_index    INT         NOT NULL DEFAULT 0,
    is_canonical_run    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (tick);
```

### 20.2 Incremental Aggregation via Materialized Views

Rather than recomputing aggregations from full table scans, incremental materialized views are updated each tick by appending only the new tick's data. The pattern uses a `last_aggregated_tick` registry table.

```sql
-- ============================================================
-- AGGREGATION REGISTRY: tracks which ticks have been aggregated.
-- ============================================================
CREATE TABLE aggregation_registry (
    run_id                  BIGINT      NOT NULL,
    table_name              TEXT        NOT NULL,
    last_aggregated_tick    BIGINT      NOT NULL DEFAULT 0,
    last_aggregated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, table_name)
);

-- ============================================================
-- 100-TICK MATERIALIZED VIEW: incremental append pattern.
-- This view is REFRESHED (not rebuilt) by appending rows for
-- new 100-tick windows after each batch of 100 ticks.
-- ============================================================
CREATE MATERIALIZED VIEW institution_states_100tick AS
SELECT
    run_id,
    (tick / 100) * 100                            AS window_start_tick,
    institution_id,
    (ARRAY_AGG(state ORDER BY tick DESC))[1]      AS final_state,
    MIN(legitimacy)                                AS min_legitimacy,
    MAX(legitimacy)                                AS max_legitimacy,
    (SUM(legitimacy) / COUNT(*))::INT             AS avg_legitimacy,
    MIN(capture_score)                             AS min_capture,
    MAX(capture_score)                             AS max_capture,
    COUNT(DISTINCT state)                          AS state_transitions_in_window
FROM institution_states
GROUP BY run_id, (tick / 100) * 100, institution_id
WITH NO DATA;

CREATE UNIQUE INDEX idx_inst_100tick_pk
    ON institution_states_100tick (run_id, window_start_tick, institution_id);

-- Incremental refresh function: only recomputes windows containing new ticks.
CREATE OR REPLACE FUNCTION refresh_institution_100tick_incremental(
    p_run_id BIGINT,
    p_from_tick BIGINT
) RETURNS VOID AS $$
DECLARE
    v_window_start BIGINT;
BEGIN
    v_window_start := (p_from_tick / 100) * 100;
    -- Delete stale windows (partial windows may have been materialized earlier).
    DELETE FROM institution_states_100tick
    WHERE run_id = p_run_id
      AND window_start_tick >= v_window_start;
    -- Reinsert from source.
    INSERT INTO institution_states_100tick
    SELECT
        run_id,
        (tick / 100) * 100,
        institution_id,
        (ARRAY_AGG(state ORDER BY tick DESC))[1],
        MIN(legitimacy), MAX(legitimacy),
        (SUM(legitimacy) / COUNT(*))::INT,
        MIN(capture_score), MAX(capture_score),
        COUNT(DISTINCT state)
    FROM institution_states
    WHERE run_id = p_run_id
      AND tick >= v_window_start
    GROUP BY run_id, (tick / 100) * 100, institution_id;

    UPDATE aggregation_registry
    SET last_aggregated_tick = p_from_tick,
        last_aggregated_at = NOW()
    WHERE run_id = p_run_id
      AND table_name = 'institution_states_100tick';
END;
$$ LANGUAGE plpgsql;
```

### 20.3 Retention Tiers

Three retention tiers manage memory and storage across the full run lifecycle:

| Tier | Tick Range | Storage | Access Pattern | Eviction |
|---|---|---|---|---|
| Hot | Last 100 ticks | In-memory `BTreeMap` in engine | Read/write every tick | Spill to Warm on tick 101+ |
| Warm | Ticks 101–10,000 | PostgreSQL tables (unpartitioned for small runs, partitioned for large) | Dashboard queries, scenario comparison | Never evicted for canonical runs |
| Cold | Tick 10,001+ | Parquet export (one file per 1,000-tick window per table) | Archival queries, export to analytics | Retention per policy |

**Hot tier management in Rust:**

```rust
/// In-memory hot tier for recent institutional states.
/// Holds the last `hot_tier_depth` ticks of institutional state for all institutions.
pub struct HotTierInstitutionCache {
    /// Circular buffer keyed by (tick mod hot_tier_depth, institution_id).
    /// BTreeMap ensures deterministic iteration order.
    pub cache: std::collections::BTreeMap<(u64, u64), TimeSeriesRecord>,
    pub hot_tier_depth: u64,    // default 100
    pub current_tick: u64,
}

impl HotTierInstitutionCache {
    /// Insert record for current tick. Evicts stale records > hot_tier_depth old.
    pub fn insert(&mut self, record: TimeSeriesRecord, tick: u64) {
        self.current_tick = tick;
        let evict_before_tick = tick.saturating_sub(self.hot_tier_depth);
        // Evict all records older than hot_tier_depth
        self.cache.retain(|(t, _), _| *t >= evict_before_tick);
        // Insert new record (using tick as key component)
        if let TimeSeriesRecord::InstitutionState { institution_id, .. } = &record {
            self.cache.insert((tick, *institution_id), record);
        }
    }

    /// Flush all records with tick < flush_before_tick to the warm tier writer.
    pub fn flush_to_warm(
        &mut self,
        flush_before_tick: u64,
        warm_writer: &mut dyn WarmTierWriter,
    ) {
        let to_flush: Vec<_> = self.cache
            .range(..(flush_before_tick, 0))
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        for (key, record) in to_flush {
            warm_writer.write(record);
            self.cache.remove(&key);
        }
    }
}

/// Trait implemented by the warm tier database writer.
pub trait WarmTierWriter: Send {
    fn write(&mut self, record: TimeSeriesRecord);
    fn commit_batch(&mut self);
}
```

### 20.4 Replay Protocol

Given `run_id` and `seed`, the full institutional state chain is reconstructable from the event log without re-executing the simulation. The replay protocol:

1. Load run metadata: `seed`, `policy_bundle_hash`, `initial_institution_states`.
2. Read event log in tick-ascending order for `run_id`.
3. For each tick, apply events in `(tick ASC, event_sequence_number ASC)` order.
4. After applying all events for a tick, compute the expected `state_hash` and compare against the stored `state_hash` in the event payload.
5. If any `state_hash` mismatch is detected, replay halts and emits `ReplayIntegrityError`.

```sql
-- ============================================================
-- REPLAY QUERY: reconstruct institutional state chain for a run.
-- ============================================================
-- Step 1: Fetch event log ordered for replay.
SELECT
    tick,
    event_sequence_number,
    event_type,
    event_payload
FROM event_log
WHERE run_id = $1
ORDER BY tick ASC, event_sequence_number ASC;

-- Step 2: Verify replay hash at each tick window.
-- The following query computes the rolling hash for ticks [start_tick, end_tick].
SELECT
    MD5(STRING_AGG(
        tick::TEXT || '|'
        || institution_id::TEXT || '|'
        || state::TEXT || '|'
        || legitimacy::TEXT || '|'
        || capture_score::TEXT,
        ',' ORDER BY tick ASC, institution_id ASC
    )) AS replay_hash
FROM institution_states
WHERE run_id = $1
  AND tick BETWEEN $2 AND $3;

-- Step 3: Fetch run metadata for initial conditions.
SELECT seed, policy_bundle_hash, initial_tick_snapshot
FROM run_metadata
WHERE run_id = $1;
```

### 20.5 Cross-Run Comparison Queries

These queries support diffing two runs' institutional trajectories for scenario comparison.

```sql
-- ============================================================
-- CROSS-RUN TRAJECTORY DIFF
-- Compare institutional state between two runs at every tick.
-- Returns ticks where state diverges.
-- ============================================================
SELECT
    a.tick,
    a.institution_id,
    a.state      AS run_a_state,
    b.state      AS run_b_state,
    a.legitimacy AS run_a_legitimacy,
    b.legitimacy AS run_b_legitimacy,
    (b.legitimacy - a.legitimacy) AS legitimacy_delta,
    a.capture_score AS run_a_capture,
    b.capture_score AS run_b_capture,
    (b.capture_score - a.capture_score) AS capture_delta
FROM institution_states a
JOIN institution_states b
    ON  a.tick           = b.tick
    AND a.institution_id = b.institution_id
WHERE a.run_id = $1  -- run A
  AND b.run_id = $2  -- run B
  AND (a.state != b.state OR ABS(a.legitimacy - b.legitimacy) > $3)
ORDER BY a.tick ASC, a.institution_id ASC;

-- ============================================================
-- CROSS-RUN LIFECYCLE TRAJECTORY DIFF
-- Compare cohort stage distributions between two runs.
-- ============================================================
SELECT
    a.tick,
    a.region_id,
    a.stage,
    SUM(a.population) AS run_a_population,
    SUM(b.population) AS run_b_population,
    (SUM(b.population) - SUM(a.population)) AS population_delta
FROM citizen_lifecycle a
JOIN citizen_lifecycle b
    ON  a.tick      = b.tick
    AND a.cohort_id = b.cohort_id
WHERE a.run_id = $1
  AND b.run_id = $2
GROUP BY a.tick, a.region_id, a.stage
HAVING ABS(SUM(b.population) - SUM(a.population)) > $4
ORDER BY a.tick ASC, a.region_id ASC, a.stage ASC;
```

---

## 21. Shadow Institution Mechanics

### 21.1 Parallel Shadow Institutions

Shadow institutions are informal power structures that emerge in parallel to formal institutions when the formal system enters `Captured` or `Collapsed` states. They are modeled explicitly as first-class simulation entities, distinct from the CIV-0105 shadow network graph (which models covert actor nodes). Shadow institutions represent institutionalized informal power — stable enough to have observable behavior but not publicly chartered.

Shadow institutions are invisible to standard metrics queries until their `detection_risk` crosses the `shadow_detection_threshold`, at which point they become visible as a governance event and contribute to the reform pressure computation.

```rust
/// A shadow institution operating parallel to the formal governance layer.
/// Shadow institutions emerge when formal institutions fail.
/// They capture resource flows, coerce compliance, and resist reform.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShadowInstitution {
    pub shadow_id: u64,
    pub run_id: u64,
    /// Tick at which this shadow institution emerged.
    pub emerged_tick: u64,
    /// Region in which this shadow institution operates.
    pub region_id: u32,
    /// The formal institution whose failure created the vacuum this shadow fills.
    pub displaced_formal_institution_id: u64,
    /// Type mirrors the formal taxonomy but with "shadow" semantics.
    pub institution_type: InstitutionType,

    /// Influence score: how much policy and resource flow this shadow controls.
    /// Fixed-point [0, 10_000]. Grows as formal institutions weaken.
    pub influence_score: i32,

    /// Detection risk: probability that formal oversight uncovers this institution.
    /// Fixed-point [0, 10_000]. Computed each tick from transparency and audit intensity.
    pub detection_risk: i32,

    /// Overlap with the formal institution it displaces.
    /// Measures how much the shadow institution's policy domain overlaps with
    /// the formal institution's chartered domain. Fixed-point [0, 10_000].
    /// High overlap → shadow is a near-replacement; low overlap → shadow fills gaps.
    pub formal_institution_overlap: i32,

    /// Resource base: joules extracted per tick by this shadow institution.
    /// Subtracted from the formal economy as a leakage term.
    pub resource_extraction_joules_per_tick: i64,

    /// Coercion level applied by this shadow institution. Fixed-point [0, 10_000].
    /// Feeds into coercion_index for cohorts in its region.
    pub coercion_level: i32,

    /// State: whether this shadow institution is active, transitioning, or dissolved.
    pub shadow_state: ShadowInstitutionState,
}

/// State of a shadow institution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ShadowInstitutionState {
    /// Operating covertly. Influence growing or stable.
    Covert,
    /// Detection event fired; partially exposed. Influence temporarily reduced.
    Exposed,
    /// Undergoing formalization attempt (see §21.3).
    Transitioning,
    /// Dissolved: either detected and reformed, or displaced by formal recovery.
    Dissolved,
}
```

### 21.2 Influence Score Dynamics

Shadow institution influence grows when formal institutional capacity is low and the governance vacuum creates resource and enforcement opportunities.

```
influence_{t+1} = clamp(
    influence_t
    + alpha_shadow * (1 - formal_policy_capacity_t) * (1 - detection_risk_t)
    + beta_scarcity * scarcity_pressure_t
    - gamma_reform * reform_pressure_t
    - delta_detection * detection_event_t,
    0, 10_000
)
```

**Detection risk computation each tick:**

```
detection_risk_t = sigma(
    transparency_level_t
    + audit_intensity_t
    + whistleblower_probability_t
    - influence_t / 10_000 * opacity_shield_t
)
```

Where `sigma` is the standard logistic function. High influence itself reduces detection risk via opacity effects (the shadow institution invests in concealment).

```rust
/// Parameters for shadow institution dynamics.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ShadowInstitutionParams {
    /// Growth rate of influence when formal capacity is zero. Fixed-point.
    pub alpha_shadow_growth_fp: i32,       // default 500 (= 0.05/tick max)
    /// Scarcity amplification of shadow influence growth. Fixed-point.
    pub beta_scarcity_fp: i32,             // default 300 (= 0.03/tick max)
    /// Reform pressure dampening on shadow influence. Fixed-point.
    pub gamma_reform_fp: i32,              // default 400
    /// Detection event penalty on influence. Fixed-point.
    pub delta_detection_fp: i32,           // default 3_000 (= 0.30 per detection)
    /// Opacity shield factor: shadow reduces own detection as influence grows.
    pub opacity_shield_fp: i32,            // default 6_000 (= 0.60 max shield)
    /// Detection risk threshold above which shadow_emerged event fires.
    pub shadow_detection_threshold: i32,   // default 7_500
    /// Resource extraction rate at full influence. Raw joules per tick.
    pub max_resource_extraction_joules: i64,   // default 1_000_000_000_000 (1 TJ/tick)
}

/// Compute detection risk for a shadow institution this tick.
pub fn compute_shadow_detection_risk(
    shadow: &ShadowInstitution,
    transparency_fp: i32,
    audit_intensity_fp: i32,
    whistleblower_prob_fp: i32,
) -> i32 {
    let raw_exposure = transparency_fp as i64
        + audit_intensity_fp as i64
        + whistleblower_prob_fp as i64;
    let opacity_shield = (shadow.opacity_shield_protection(shadow.influence_score));
    let net = raw_exposure - opacity_shield as i64;
    // Logistic: sigmoid((net - 5000) / 2000) mapped to [0, 10_000]
    let x = (net - 5_000) * 10_000 / 20_000;
    logistic_fixed_point(x as i32)
}

fn logistic_fixed_point(x: i32) -> i32 {
    // Approximation of sigma(x/10000) → [0, 10_000]
    // Uses piecewise linear approximation for fixed-point safety.
    if x < -10_000 { return 500; }
    if x >  10_000 { return 9_500; }
    5_000 + x / 2
}

impl ShadowInstitution {
    fn opacity_shield_protection(&self, influence: i32) -> i32 {
        // opacity shield scales with influence
        (6_000i64 * influence as i64 / 10_000) as i32
    }
}
```

### 21.3 Interaction with CIV-0105 Shadow Network Graph

CIV-0105 defines the shadow network as a directed influence graph of covert actor nodes. Shadow institutions in this spec interact with that graph as follows:

- Shadow institutions are registered as **institutional nodes** in the CIV-0105 influence graph upon emergence. They receive a node ID in the shadow network and participate in influence diffusion.
- The `shadow_influence` component of the `CaptureScore` struct (§5.3) is the aggregate influence flowing from all connected shadow nodes (including shadow institutions) toward the formal institution.
- Shadow institution `influence_score` maps directly to the CIV-0105 node weight for that institution node.
- Edge weights between shadow institution nodes and formal institution nodes in the CIV-0105 graph are proportional to `formal_institution_overlap`.

This bidirectional relationship ensures that shadow institutions contribute to formal institution capture dynamics and vice versa.

### 21.4 Shadow-to-Formal Transition

A shadow institution may formalize through three pathways:

**1. Revolutionary Formalization:** When `influence_score >= 9_000` and formal institution is `Collapsed` for at least `revolution_tick_threshold` (default 20) consecutive ticks, the shadow institution may petition for charter ratification. If reform pressure is below `reform_pressure_threshold`, this fires as a `revolution_event` rather than a legitimacy-building process, creating a new institution with low initial legitimacy (default `3_500`).

**2. Election Capture:** When `influence_score >= 6_500` and a governance event `election_held` fires, shadow institution influence is added to the relevant candidates' coalition support, potentially resulting in formal institutional control. Legitimacy is moderate (default `5_500`) if election integrity is high; higher if election integrity is compromised.

**3. Coup:** When `influence_score >= 8_000` AND `military_institution_state == Captured` AND `legitimacy_of_formal_institution < 2_000`, a coup event fires. The shadow institution replaces the formal institution with `legitimacy = 2_000` and `capture_score = 8_000` (starts captured by definition).

```rust
/// Record of a shadow-to-formal transition event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShadowToFormalTransition {
    pub run_id: u64,
    pub tick: u64,
    pub shadow_id: u64,
    pub new_formal_institution_id: u64,
    pub transition_pathway: TransitionPathway,
    /// Legitimacy of the newly formalized institution. Fixed-point [0, 10_000].
    pub initial_legitimacy_fp: i32,
    /// Capture score of the newly formalized institution (inherited from shadow).
    pub initial_capture_score_fp: i32,
    /// Whether the old formal institution was dissolved in this transition.
    pub old_institution_dissolved: bool,
    pub old_institution_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransitionPathway {
    RevolutionaryFormalization,
    ElectionCapture,
    Coup,
}
```

---

## 22. Extended Event Taxonomy and DDL

### 22.1 Six Additional Event Schemas

#### `institution.created.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "institution.created.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "institution_id", "institution_name", "institution_type",
               "coalition_support_at_ratification", "charter_resource_joules",
               "initial_legitimacy", "charter_expiry_tick"],
  "properties": {
    "event_type": { "const": "institution.created.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "institution_id": { "type": "integer", "minimum": 0 },
    "institution_name": { "type": "string" },
    "institution_type": {
      "type": "string",
      "enum": ["Government", "Judiciary", "Military", "CivilSociety", "Religious",
               "EconomicRegulatory", "RightsAuthority", "MarketRegulator",
               "EnergyAccountingAuthority", "GovernanceIntegrityCouncil",
               "MetricReviewBoard", "LaborBoard", "CentralBank", "SecurityApparatus"]
    },
    "coalition_support_at_ratification": {
      "type": "integer", "minimum": 0, "maximum": 10000
    },
    "charter_resource_joules": {
      "type": "integer", "minimum": 0,
      "description": "Raw joules committed as initial operating capital"
    },
    "initial_legitimacy": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "charter_expiry_tick": { "type": "integer", "minimum": 0 },
    "dependency_institution_ids": {
      "type": "array", "items": { "type": "integer", "minimum": 0 }
    }
  },
  "additionalProperties": false
}
```

#### `institution.dissolved.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "institution.dissolved.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "institution_id", "dissolution_cause",
               "final_legitimacy", "final_capture_score", "ticks_in_operation"],
  "properties": {
    "event_type": { "const": "institution.dissolved.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "institution_id": { "type": "integer", "minimum": 0 },
    "dissolution_cause": {
      "type": "string",
      "enum": [
        "legitimacy_collapsed",
        "merger_absorbed",
        "coup_displaced",
        "charter_expired",
        "governance_event_dissolution",
        "dependency_cascade_collapse"
      ]
    },
    "final_legitimacy": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "final_capture_score": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "ticks_in_operation": { "type": "integer", "minimum": 0 },
    "successor_institution_id": {
      "type": ["integer", "null"],
      "description": "If dissolved via merger, the ID of the merged successor"
    }
  },
  "additionalProperties": false
}
```

#### `citizen.debt_defaulted.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "citizen.debt_defaulted.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "cohort_id", "region_id", "population_affected",
               "joule_debt_at_default", "stress_spike", "alienation_spike",
               "welfare_penalty_next_tick"],
  "properties": {
    "event_type": { "const": "citizen.debt_defaulted.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "cohort_id": { "type": "integer", "minimum": 0 },
    "region_id": { "type": "integer", "minimum": 0 },
    "population_affected": { "type": "integer", "minimum": 0 },
    "joule_debt_at_default": {
      "type": "integer",
      "description": "Raw joules of accumulated debt that triggered default"
    },
    "stress_spike": {
      "type": "integer", "minimum": 0, "maximum": 10000,
      "description": "Fixed-point increase to stress score on default"
    },
    "alienation_spike": {
      "type": "integer", "minimum": 0, "maximum": 10000
    },
    "welfare_penalty_next_tick": {
      "type": "integer", "minimum": 0, "maximum": 10000,
      "description": "Fixed-point reduction in welfare_access for following tick"
    },
    "lifecycle_transition_triggered": {
      "type": ["string", "null"],
      "enum": ["Dissenting", null],
      "description": "If stress spike crossed energy_crisis_threshold, transition stage"
    }
  },
  "additionalProperties": false
}
```

#### `citizen.generational_transfer.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "citizen.generational_transfer.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "source_cohort_id", "beneficiary_cohort_id",
               "net_transfer_joules", "tax_collected_joules",
               "deceased_count", "wealth_concentration_index"],
  "properties": {
    "event_type": { "const": "citizen.generational_transfer.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "source_cohort_id": { "type": "integer", "minimum": 0 },
    "beneficiary_cohort_id": { "type": "integer", "minimum": 0 },
    "net_transfer_joules": {
      "type": "integer",
      "description": "Raw joules transferred after inheritance tax"
    },
    "tax_collected_joules": {
      "type": "integer", "minimum": 0,
      "description": "Raw joules collected as inheritance tax, credited to retirement pool"
    },
    "deceased_count": { "type": "integer", "minimum": 1 },
    "wealth_concentration_index": {
      "type": "integer", "minimum": 0, "maximum": 10000,
      "description": "Gini × 10_000 of lifetime_joule_credit at this tick"
    },
    "inheritance_tax_rate_fp": {
      "type": "integer", "minimum": 0, "maximum": 10000,
      "description": "Inheritance tax rate applied (fixed-point)"
    }
  },
  "additionalProperties": false
}
```

#### `institution.shadow_emerged.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "institution.shadow_emerged.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "shadow_id", "region_id", "displaced_formal_institution_id",
               "institution_type", "initial_influence_score",
               "detection_risk", "formal_institution_overlap"],
  "properties": {
    "event_type": { "const": "institution.shadow_emerged.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "shadow_id": { "type": "integer", "minimum": 0 },
    "region_id": { "type": "integer", "minimum": 0 },
    "displaced_formal_institution_id": { "type": "integer", "minimum": 0 },
    "institution_type": { "type": "string" },
    "initial_influence_score": {
      "type": "integer", "minimum": 0, "maximum": 10000
    },
    "detection_risk": { "type": "integer", "minimum": 0, "maximum": 10000 },
    "formal_institution_overlap": {
      "type": "integer", "minimum": 0, "maximum": 10000
    },
    "trigger": {
      "type": "string",
      "enum": [
        "formal_institution_collapsed",
        "formal_institution_captured_vacuum",
        "scarcity_crisis_vacuum",
        "external_actor_seeded"
      ]
    }
  },
  "additionalProperties": false
}
```

#### `cohort.demographic_shifted.v1`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "cohort.demographic_shifted.v1",
  "type": "object",
  "required": ["event_type", "version", "tick", "run_id", "state_hash",
               "cohort_id", "region_id", "births_this_tick", "deaths_this_tick",
               "population_before", "population_after", "age_band"],
  "properties": {
    "event_type": { "const": "cohort.demographic_shifted.v1" },
    "version": { "const": 1 },
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "integer", "minimum": 0 },
    "state_hash": { "type": "string" },
    "cohort_id": { "type": "integer", "minimum": 0 },
    "region_id": { "type": "integer", "minimum": 0 },
    "age_band": {
      "type": "string",
      "enum": ["Youth", "Working", "LateCareer", "Elder"]
    },
    "births_this_tick": { "type": "integer", "minimum": 0 },
    "deaths_this_tick": { "type": "integer", "minimum": 0 },
    "population_before": { "type": "integer", "minimum": 0 },
    "population_after": { "type": "integer", "minimum": 0 },
    "generational_transfer_fired": {
      "type": "boolean",
      "description": "True if deaths triggered a generational_transfer event this tick"
    },
    "wealth_concentration_index_delta": {
      "type": "integer",
      "description": "Change in Gini × 10_000 resulting from this demographic shift"
    }
  },
  "additionalProperties": false
}
```

### 22.2 Four Additional SQL Tables

```sql
-- ============================================================
-- INSTITUTION CHARTERS
-- One row per institution creation event. Append-only.
-- ============================================================
CREATE TABLE institution_charters (
    run_id                          BIGINT      NOT NULL,
    institution_id                  BIGINT      NOT NULL,
    ratified_tick                   BIGINT      NOT NULL,
    institution_type                TEXT        NOT NULL,
    name                            TEXT        NOT NULL,
    coalition_support_at_ratification INT       NOT NULL,  -- fixed-point [0, 10_000]
    charter_resource_joules         BIGINT      NOT NULL,
    initial_legitimacy_fp           INT         NOT NULL,  -- fixed-point [0, 10_000]
    charter_expiry_tick             BIGINT      NOT NULL,  -- 0 = no expiry
    dependency_institution_ids      BIGINT[]    NOT NULL DEFAULT '{}',
    dissolved_tick                  BIGINT,                -- NULL until dissolved
    dissolution_cause               TEXT,
    successor_institution_id        BIGINT,
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, institution_id)
);

CREATE INDEX idx_charters_run_type
    ON institution_charters (run_id, institution_type, ratified_tick);

CREATE INDEX idx_charters_expiry
    ON institution_charters (run_id, charter_expiry_tick)
    WHERE charter_expiry_tick > 0;

-- ============================================================
-- GENERATIONAL TRANSFERS
-- One row per mortality-driven wealth transfer event.
-- ============================================================
CREATE TABLE generational_transfers (
    run_id                      BIGINT      NOT NULL,
    tick                        BIGINT      NOT NULL,
    transfer_sequence           INT         NOT NULL,  -- ordering within tick
    source_cohort_id            BIGINT      NOT NULL,
    beneficiary_cohort_id       BIGINT      NOT NULL,
    net_transfer_joules         BIGINT      NOT NULL,
    tax_collected_joules        BIGINT      NOT NULL,
    deceased_count              INT         NOT NULL,
    wealth_concentration_index  INT         NOT NULL,  -- Gini × 10_000
    inheritance_tax_rate_fp     INT         NOT NULL,
    is_canonical_run            BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, transfer_sequence)
);

CREATE INDEX idx_gen_transfers_run_source
    ON generational_transfers (run_id, source_cohort_id, tick);

CREATE INDEX idx_gen_transfers_run_beneficiary
    ON generational_transfers (run_id, beneficiary_cohort_id, tick);

-- Conservation check view: total joules in = total joules transferred + total tax.
CREATE VIEW generational_transfer_conservation AS
SELECT
    run_id,
    SUM(net_transfer_joules + tax_collected_joules) AS total_pre_tax_joules,
    SUM(net_transfer_joules)                         AS total_transferred_joules,
    SUM(tax_collected_joules)                        AS total_tax_joules
FROM generational_transfers
GROUP BY run_id;

-- ============================================================
-- SHADOW INSTITUTIONS
-- One row per tick per active shadow institution. Append-only.
-- ============================================================
CREATE TABLE shadow_institutions (
    run_id                              BIGINT      NOT NULL,
    tick                                BIGINT      NOT NULL,
    shadow_id                           BIGINT      NOT NULL,
    region_id                           INT         NOT NULL,
    displaced_formal_institution_id     BIGINT      NOT NULL,
    institution_type                    TEXT        NOT NULL,
    shadow_state                        SMALLINT    NOT NULL,
    -- 0=Covert, 1=Exposed, 2=Transitioning, 3=Dissolved
    influence_score                     INT         NOT NULL,  -- fixed-point [0, 10_000]
    detection_risk                      INT         NOT NULL,  -- fixed-point [0, 10_000]
    formal_institution_overlap          INT         NOT NULL,  -- fixed-point [0, 10_000]
    resource_extraction_joules          BIGINT      NOT NULL,
    coercion_level                      INT         NOT NULL,  -- fixed-point [0, 10_000]
    is_canonical_run                    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at                          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, shadow_id)
);

CREATE INDEX idx_shadow_run_region
    ON shadow_institutions (run_id, region_id, tick);

CREATE INDEX idx_shadow_active
    ON shadow_institutions (run_id, tick)
    WHERE shadow_state < 3;  -- not Dissolved

-- View: current active shadow institutions per run (latest tick per shadow_id).
CREATE VIEW active_shadow_institutions AS
SELECT DISTINCT ON (run_id, shadow_id)
    run_id, tick, shadow_id, region_id,
    displaced_formal_institution_id, institution_type,
    shadow_state, influence_score, detection_risk,
    formal_institution_overlap, resource_extraction_joules, coercion_level
FROM shadow_institutions
WHERE shadow_state < 3
ORDER BY run_id, shadow_id, tick DESC;

-- ============================================================
-- COHORT DEMOGRAPHICS
-- One row per (run, tick, cohort) for demographic tracking.
-- Append-only. Complements citizen_lifecycle with birth/death flow data.
-- ============================================================
CREATE TABLE cohort_demographics (
    run_id                          BIGINT      NOT NULL,
    tick                            BIGINT      NOT NULL,
    cohort_id                       BIGINT      NOT NULL,
    region_id                       INT         NOT NULL,
    age_band                        SMALLINT    NOT NULL,
    population                      INT         NOT NULL,
    births_this_tick                INT         NOT NULL,
    deaths_this_tick                INT         NOT NULL,
    net_change                      INT         NOT NULL,  -- births - deaths
    birth_rate_fp                   INT         NOT NULL,  -- effective rate this tick
    death_rate_fp                   INT         NOT NULL,  -- effective rate this tick
    generational_transfer_fired     BOOLEAN     NOT NULL DEFAULT FALSE,
    wealth_concentration_index      INT         NOT NULL,  -- Gini × 10_000
    human_capital                   BIGINT,                -- Youth cohorts only; NULL otherwise
    education_investment_joules     BIGINT,                -- Youth cohorts only
    institutional_memory_credit     INT,                   -- Elder cohorts only; NULL otherwise
    is_canonical_run                BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (run_id, tick, cohort_id)
);

CREATE INDEX idx_cohort_demo_run_region
    ON cohort_demographics (run_id, region_id, tick);

CREATE INDEX idx_cohort_demo_youth
    ON cohort_demographics (run_id, tick)
    WHERE age_band = 0;  -- Youth = 0

CREATE INDEX idx_cohort_demo_elder
    ON cohort_demographics (run_id, tick)
    WHERE age_band = 3;  -- Elder = 3
```

---

## 23. Extended Test Suite

### 23.1 Eight Additional Test Stubs

The following tests are added to the existing test suites in `crates/citizens/src/tests/` and `crates/institutions/src/tests/`.

```rust
// ============================================================
// GENERATIONAL WEALTH TRANSFER CONSERVATION
// crates/citizens/src/tests/generational_tests.rs
// ============================================================

#[cfg(test)]
mod generational_tests {
    use crate::cohort::*;
    use crate::demographics::*;

    /// Total joules before transfer = net_transfer_joules + tax_collected_joules.
    /// Tests that no joules are created or destroyed by the transfer mechanics.
    #[test]
    fn test_generational_transfer_joule_conservation() {
        let params = GenerationalTransferParams {
            inheritance_enabled: true,
            minimum_floor_joules: 0,
            beneficiary_fraction_fp: 8_000,
            inheritance_tax_rate_fp: 2_500,
        };

        // Cohort with 100 deceased, each with 1e12 joules lifetime credit.
        let deceased_joule_credit_per_citizen: i64 = 1_000_000_000_000;
        let deceased_count: u32 = 100;
        let total_transferable = deceased_joule_credit_per_citizen * deceased_count as i64;

        let transfer = compute_generational_transfer(
            1,    // source_cohort_id
            2,    // beneficiary_cohort_id
            deceased_count,
            deceased_joule_credit_per_citizen,
            &params,
        );

        let pre_tax = transfer.net_transfer_joules + transfer.tax_collected_joules;
        assert_eq!(
            pre_tax, total_transferable,
            "Pre-tax joules must equal transferable joules: conservation violated"
        );

        let expected_tax = total_transferable
            * params.inheritance_tax_rate_fp as i64 / 10_000;
        assert_eq!(
            transfer.tax_collected_joules, expected_tax,
            "Tax collected must match inheritance_tax_rate_fp"
        );

        let expected_net = (total_transferable - expected_tax)
            * params.beneficiary_fraction_fp as i64 / 10_000;
        assert_eq!(
            transfer.net_transfer_joules, expected_net,
            "Net transfer must match (total - tax) * beneficiary_fraction"
        );
    }

    /// Wealth concentration index rises monotonically when inheritance is concentrated.
    #[test]
    fn test_wealth_concentration_rises_with_concentrated_inheritance() {
        // Scenario: one high-wealth cohort dies, transfers to one youth cohort.
        // Gini should increase vs baseline equal distribution.
        let initial_gini = compute_lifetime_joule_gini(&[
            1_000_000_000_000i64, // cohort A
            1_000_000_000_000i64, // cohort B
            1_000_000_000_000i64, // cohort C
        ]);

        // After transfer: cohort A (youth beneficiary) gains 5x the others.
        let post_gini = compute_lifetime_joule_gini(&[
            6_000_000_000_000i64, // cohort A (received inheritance)
            1_000_000_000_000i64,
            1_000_000_000_000i64,
        ]);

        assert!(
            post_gini > initial_gini,
            "Gini must rise after concentrated generational transfer: {} vs {}",
            post_gini, initial_gini
        );
    }
}

// ============================================================
// SHADOW INSTITUTION EMERGENCE THRESHOLD
// crates/institutions/src/tests/shadow_tests.rs
// ============================================================

#[cfg(test)]
mod shadow_tests {
    use crate::shadow::*;
    use crate::institution::*;

    /// Shadow institution only emerges when formal institution is Collapsed or Captured.
    #[test]
    fn test_shadow_emergence_requires_formal_failure() {
        let params = ShadowInstitutionParams::default();

        // Formal institution is Stable: shadow should NOT emerge.
        let result = evaluate_shadow_emergence_condition(
            InstitutionState::Stable,
            0,    // ticks_in_failure_state
            5_000, // scarcity_pressure_fp
            &params,
        );
        assert!(!result, "Shadow must not emerge when formal institution is Stable");

        // Formal institution Collapsed for > threshold ticks: shadow SHOULD emerge.
        let result = evaluate_shadow_emergence_condition(
            InstitutionState::Collapsed,
            params.min_failure_ticks_before_emergence + 1,
            5_000,
            &params,
        );
        assert!(result, "Shadow must emerge after formal institution collapses for threshold ticks");
    }

    /// Shadow influence grows faster when formal policy capacity is lower.
    #[test]
    fn test_shadow_influence_inversely_proportional_to_formal_capacity() {
        let params = ShadowInstitutionParams::default();
        let mut shadow_low_formal = ShadowInstitution::new_test(1, 1, 1);
        let mut shadow_high_formal = ShadowInstitution::new_test(2, 1, 1);

        shadow_low_formal.influence_score = 3_000;
        shadow_high_formal.influence_score = 3_000;

        // Low formal capacity: shadow grows quickly.
        let delta_low = compute_shadow_influence_delta(
            &shadow_low_formal,
            1_000, // formal_policy_capacity_fp (low)
            3_000, // reform_pressure_fp
            false, // detection_event
            &params,
        );

        // High formal capacity: shadow grows slowly.
        let delta_high = compute_shadow_influence_delta(
            &shadow_high_formal,
            9_000, // formal_policy_capacity_fp (high)
            3_000,
            false,
            &params,
        );

        assert!(
            delta_low > delta_high,
            "Shadow influence must grow faster under low formal capacity: {} vs {}",
            delta_low, delta_high
        );
    }

    /// Detection event reduces shadow influence by delta_detection_fp.
    #[test]
    fn test_shadow_detection_event_reduces_influence() {
        let params = ShadowInstitutionParams::default();
        let mut shadow = ShadowInstitution::new_test(1, 1, 1);
        shadow.influence_score = 7_000;

        let delta_with_detection = compute_shadow_influence_delta(
            &shadow,
            5_000, // formal_policy_capacity_fp
            3_000, // reform_pressure_fp
            true,  // detection_event = true
            &params,
        );

        let delta_without_detection = compute_shadow_influence_delta(
            &shadow,
            5_000,
            3_000,
            false,  // detection_event = false
            &params,
        );

        assert!(
            delta_with_detection < delta_without_detection,
            "Detection event must reduce net shadow influence delta"
        );
    }
}

// ============================================================
// INSTITUTION FORMATION AND DISSOLUTION DETERMINISM
// crates/institutions/src/tests/formation_tests.rs
// ============================================================

#[cfg(test)]
mod formation_tests {
    use crate::institution::*;
    use crate::charter::*;

    /// Institution formation with identical charter inputs produces identical
    /// initial state across two independent runs.
    #[test]
    fn test_institution_formation_is_deterministic() {
        let charter_a = InstitutionCharter {
            institution_id: 5001,
            run_id: 1,
            ratified_tick: 100,
            institution_type: InstitutionType::Government,
            name: "Test Government A".to_string(),
            coalition_support_at_ratification: 7_500,
            charter_resource_joules: 10_000_000_000_000,
            initial_legitimacy_fp: 7_000,
            charter_expiry_tick: 0,
            dependency_ids: vec![],
        };

        let mut charter_b = charter_a.clone();
        charter_b.run_id = 2;

        let inst_a = Institution::from_charter(&charter_a);
        let inst_b = Institution::from_charter(&charter_b);

        assert_eq!(inst_a.state, inst_b.state, "Initial states must match");
        assert_eq!(inst_a.legitimacy, inst_b.legitimacy, "Initial legitimacy must match");
        assert_eq!(inst_a.capture_score, inst_b.capture_score, "Initial capture must match");
    }

    /// Dissolution of an institution via legitimacy collapse produces a Collapsed
    /// sentinel row and fires institution.dissolved.v1 event.
    #[test]
    fn test_institution_dissolution_via_legitimacy_collapse() {
        let params = InstitutionTransitionParams::default();
        let mut inst = Institution::new_test(5001, InstitutionState::Contested);

        // Push legitimacy below contested floor to trigger collapse.
        let state = evaluate_transition(
            &mut inst,
            params.legitimacy_contested_floor - 1,
            1_000,
            &params,
            0,
        );
        assert_eq!(state, InstitutionState::Collapsed,
            "Institution must collapse when legitimacy drops below contested floor");

        // After collapse, policy_capacity must be zero.
        let outputs = InstitutionOutputs::from_state(
            InstitutionState::Collapsed,
            (params.legitimacy_contested_floor - 1) as f32 / 10_000.0,
            0.1,
        );
        assert_eq!(outputs.policy_capacity, 0.0,
            "Collapsed institution must have zero policy_capacity");
    }
}

// ============================================================
// DEBT DEFAULT CASCADE
// crates/citizens/src/tests/debt_tests.rs
// ============================================================

#[cfg(test)]
mod debt_tests {
    use crate::cohort::*;
    use crate::health::*;

    /// Debt default fires when joule_debt >= joule_debt_default_threshold.
    #[test]
    fn test_debt_default_fires_at_threshold() {
        let params = EnergyDebtParams {
            joule_debt_default_threshold: 50_000_000_000,
            debt_repayment_rate_fp: 3_000,
            debt_default_stress_spike: 2_500,
            debt_default_alienation_spike: 1_500,
            debt_default_welfare_penalty: 2_000,
            debt_stress_base_rate: 50,
        };
        let mut health_state = CitizenHealthState::new_test(1);

        // Set debt just below threshold: no default.
        health_state.joule_debt = params.joule_debt_default_threshold - 1;
        let fired = advance_debt(&mut health_state, 0, 0, &params);
        assert!(!fired, "Default must not fire below threshold");

        // Set debt to exactly threshold: default fires.
        health_state.joule_debt = params.joule_debt_default_threshold;
        let fired = advance_debt(&mut health_state, 0, 0, &params);
        assert!(fired, "Default must fire at threshold");
    }

    /// Debt stress pressure increases monotonically with outstanding debt.
    #[test]
    fn test_debt_stress_pressure_monotone_with_debt() {
        let params = EnergyDebtParams {
            joule_debt_default_threshold: 50_000_000_000,
            debt_repayment_rate_fp: 3_000,
            debt_default_stress_spike: 2_500,
            debt_default_alienation_spike: 1_500,
            debt_default_welfare_penalty: 2_000,
            debt_stress_base_rate: 50,
        };

        let mut hs_low = CitizenHealthState::new_test(1);
        let mut hs_high = CitizenHealthState::new_test(2);

        hs_low.stress = 0;
        hs_high.stress = 0;
        hs_low.joule_debt = 10_000_000_000;  // low debt
        hs_high.joule_debt = 40_000_000_000; // high debt

        advance_debt(&mut hs_low,  0, 0, &params);
        advance_debt(&mut hs_high, 0, 0, &params);

        assert!(
            hs_high.stress > hs_low.stress,
            "Higher debt must produce higher stress pressure: {} vs {}",
            hs_high.stress, hs_low.stress
        );
    }

    /// Debt repayment reduces outstanding debt when joule_balance is positive.
    #[test]
    fn test_debt_repayment_reduces_debt_on_positive_balance() {
        let params = EnergyDebtParams {
            joule_debt_default_threshold: 50_000_000_000,
            debt_repayment_rate_fp: 5_000, // 50% of surplus goes to repayment
            debt_default_stress_spike: 2_500,
            debt_default_alienation_spike: 1_500,
            debt_default_welfare_penalty: 2_000,
            debt_stress_base_rate: 0, // disable continuous pressure for this test
        };
        let mut hs = CitizenHealthState::new_test(1);
        hs.joule_debt = 20_000_000_000;

        let surplus = 10_000_000_000i64; // positive joule_balance
        advance_debt(&mut hs, surplus, 0, &params);

        let expected_repayment = surplus * 5_000 / 10_000;
        let expected_debt = 20_000_000_000 - expected_repayment;
        assert_eq!(
            hs.joule_debt, expected_debt,
            "Debt must be reduced by repayment_rate * surplus"
        );
    }
}

// ============================================================
// MULTI-GENERATION DEMOGRAPHIC STABILITY
// crates/citizens/src/tests/demographic_tests.rs
// ============================================================

#[cfg(test)]
mod demographic_tests {
    use crate::cohort::*;
    use crate::demographics::*;

    /// Under stable conditions (high welfare, energy security, moderate legitimacy),
    /// birth rate >= death rate for Working age band.
    #[test]
    fn test_stable_conditions_produce_positive_net_population_change() {
        let params = CohortDemographics {
            cohort_id: 1,
            region_id: 1,
            age_band: AgeBand::Working,
            last_revised_tick: 0,
            base_birth_rate_fp: 3,
            welfare_birth_multiplier_at_full: 12_000,
            energy_surplus_birth_bonus_fp: 1_500,
            alienation_birth_drag_fp: 3_000,
            death_rate_youth_fp: 2,
            death_rate_working_fp: 3,
            death_rate_late_career_fp: 12,
            death_rate_elder_fp: 80,
            health_death_multiplier_fp: 5_000,
        };

        let cohort = CitizenCohort {
            cohort_id: 1,
            region_id: 1,
            skill_quintile: 2,
            age_band: AgeBand::Working,
            population: 10_000,
            stage: LifecycleStage::Active,
            stage_entered_tick: 0,
            energy_security: 9_500,
            welfare_access: 9_000,
            coercion_index: 1_000,
            mobility_constraint: 2_000,
            lifetime_joule_credit: 500_000_000_000,
            joules_earned_this_tick: 20_000_000_000,
            stress_score: 1_000,
            ticks_in_stage: 10,
        };

        let health_state = CitizenHealthState {
            cohort_id: 1,
            health: 9_000,
            stress: 1_000,
            joule_balance: 5_000_000_000,
            joule_debt: 0,
            social_trust: 8_000,
            alienation_index: 500,
            ideology_vector_id: 0,
        };

        let avg_legitimacy_fp = 7_500;
        let (births, deaths) = compute_demographic_change(
            &cohort, &health_state, avg_legitimacy_fp, &params
        );

        assert!(
            births as i64 >= deaths as i64,
            "Stable conditions must produce births >= deaths: {} births, {} deaths",
            births, deaths
        );
    }

    /// Population change is bounded: neither births nor deaths can exceed population.
    #[test]
    fn test_demographic_change_bounded_by_population() {
        let params = CohortDemographics {
            cohort_id: 1,
            region_id: 1,
            age_band: AgeBand::Elder,
            last_revised_tick: 0,
            base_birth_rate_fp: 3,
            welfare_birth_multiplier_at_full: 12_000,
            energy_surplus_birth_bonus_fp: 1_500,
            alienation_birth_drag_fp: 3_000,
            death_rate_youth_fp: 2,
            death_rate_working_fp: 3,
            death_rate_late_career_fp: 12,
            death_rate_elder_fp: 9_000, // extreme death rate for Elder in crisis
            health_death_multiplier_fp: 5_000,
        };

        let cohort = CitizenCohort {
            cohort_id: 1,
            region_id: 1,
            skill_quintile: 1,
            age_band: AgeBand::Elder,
            population: 100,
            stage: LifecycleStage::Retired,
            stage_entered_tick: 0,
            energy_security: 1_000,
            welfare_access: 1_000,
            coercion_index: 9_000,
            mobility_constraint: 8_000,
            lifetime_joule_credit: 1_000_000_000_000,
            joules_earned_this_tick: 0,
            stress_score: 9_000,
            ticks_in_stage: 50,
        };

        let health_state = CitizenHealthState {
            cohort_id: 1,
            health: 500,
            stress: 9_000,
            joule_balance: -5_000_000_000,
            joule_debt: 45_000_000_000,
            social_trust: 500,
            alienation_index: 9_000,
            ideology_vector_id: 0,
        };

        let (births, deaths) = compute_demographic_change(
            &cohort, &health_state, 2_000, &params
        );

        assert!(
            deaths <= cohort.population,
            "Deaths ({}) must not exceed population ({})",
            deaths, cohort.population
        );
        assert_eq!(births, 0, "Elder cohort must produce zero births");
    }
}

// ============================================================
// CROSS-RUN INSTITUTIONAL TRAJECTORY DIFF
// crates/institutions/src/tests/timeseries_tests.rs (extended)
// ============================================================

#[cfg(test)]
mod cross_run_diff_tests {
    use crate::timeseries::*;

    /// Two runs with identical seeds produce identical institutional trajectories
    /// (cross-run diff returns zero diverging ticks).
    #[test]
    fn test_identical_seed_produces_zero_diff() {
        let run_a = simulate_n_ticks(50, 42_000);
        let run_b = simulate_n_ticks(50, 42_000);

        let diff = compute_institutional_trajectory_diff(&run_a, &run_b, 0);
        assert_eq!(
            diff.diverging_tick_count, 0,
            "Identical seeds must produce zero diverging ticks"
        );
    }

    /// Two runs with different seeds produce at least one diverging tick
    /// (with high probability; verify stochastic behavior is present).
    #[test]
    fn test_different_seeds_produce_divergence() {
        let run_a = simulate_n_ticks(50, 12_345);
        let run_b = simulate_n_ticks(50, 99_999);

        let diff = compute_institutional_trajectory_diff(&run_a, &run_b, 0);
        assert!(
            diff.diverging_tick_count > 0,
            "Different seeds must produce at least one diverging tick"
        );
    }
}
```

### 23.2 Scenario: Institutional Collapse Cascade

This scenario test exercises the full cross-institution dependency chain: one institution captured, legitimacy drain propagating to dependent institutions, chain reaction destabilization.

```rust
// crates/institutions/src/tests/scenario_tests.rs

#[cfg(test)]
mod scenario_tests {
    use crate::institution::*;
    use crate::fsm::*;
    use crate::charter::*;
    use crate::propagation::*;
    use crate::params::*;

    /// Scenario: "Institutional Collapse Cascade"
    ///
    /// Setup:
    ///   Institution A (Government, Stable) — depended upon by B and C.
    ///   Institution B (Judiciary, Stable) — depends on A.
    ///   Institution C (EconomicRegulatory, Stable) — depends on A and B.
    ///
    /// Sequence:
    ///   Tick 1–5: Institution A capture_score rises to capture_threshold.
    ///   Tick 6: A transitions Stable → Captured. Propagation lag = 2 ticks.
    ///   Tick 8: A's Captured outputs take effect. B's effective_policy_capacity reduced to 50%.
    ///   Tick 8–12: B legitimacy declines due to reduced capacity (inability to enforce).
    ///   Tick 13: B transitions Contested → Collapsed.
    ///   Tick 13: C, depending on both A and B, now has both at failure states.
    ///             C effective_policy_capacity = 0.5 × 0.0 = 0.0.
    ///   Tick 14: C transitions Contested → Collapsed via legitimacy_below_contested_floor.
    ///
    /// Assertions:
    ///   - A capture triggers B effective_policy_capacity reduction.
    ///   - B collapse fires institution.dissolved.v1 for B.
    ///   - C collapse fires after B collapses (chain reaction demonstrable).
    ///   - Insurgency modifier at tick 14 >= 2.0 × 2.0 = 4.0 (amplified by two Collapsed).
    #[test]
    fn test_institutional_collapse_cascade() {
        let params = InstitutionTransitionParams::default();
        let mut institutions: std::collections::BTreeMap<u64, Institution> =
            std::collections::BTreeMap::new();

        // Create institution A (Government).
        let mut inst_a = Institution::new_test(1001, InstitutionState::Stable);
        inst_a.legitimacy = 8_000;
        inst_a.capture_score = 0;

        // Create institution B (Judiciary), depends on A.
        let mut inst_b = Institution::new_test(1002, InstitutionState::Stable);
        inst_b.legitimacy = 8_500;
        inst_b.capture_score = 0;

        // Create institution C (EconomicRegulatory), depends on A and B.
        let mut inst_c = Institution::new_test(1003, InstitutionState::Stable);
        inst_c.legitimacy = 7_500;
        inst_c.capture_score = 0;

        let dependencies = vec![
            InstitutionDependency {
                dependent_institution_id: 1002,
                required_institution_id: 1001,
                minimum_required_state: InstitutionState::Contested,
                availability_at_minimum_fp: 7_500,
            },
            InstitutionDependency {
                dependent_institution_id: 1003,
                required_institution_id: 1001,
                minimum_required_state: InstitutionState::Contested,
                availability_at_minimum_fp: 7_500,
            },
            InstitutionDependency {
                dependent_institution_id: 1003,
                required_institution_id: 1002,
                minimum_required_state: InstitutionState::Contested,
                availability_at_minimum_fp: 7_500,
            },
        ];

        // Phase 1: Drive A to Captured (ticks 1–6).
        for tick in 1u64..=5 {
            inst_a.capture_score = tick as i32 * 1_400; // rises to 7_000 at tick 5
            let state = evaluate_transition(
                &mut inst_a, inst_a.legitimacy, inst_a.capture_score, &params, tick
            );
            if tick < 5 { assert_eq!(state, InstitutionState::Stable); }
        }
        let state_a = evaluate_transition(
            &mut inst_a, 8_000, params.capture_threshold, &params, 6
        );
        assert_eq!(state_a, InstitutionState::Captured,
            "A must be Captured at tick 6");

        // Phase 2: After propagation lag (2 ticks), B capacity is reduced.
        let a_state_map: std::collections::BTreeMap<u64, InstitutionState> =
            [(1001, InstitutionState::Captured),
             (1002, InstitutionState::Stable),
             (1003, InstitutionState::Stable)]
            .into_iter().collect();

        let b_base_capacity = InstitutionOutputs::from_state(
            InstitutionState::Stable,
            8_500.0 / 10_000.0,
            0.0,
        ).policy_capacity;
        let b_effective_capacity_fp = effective_policy_capacity(
            1002,
            (b_base_capacity * 10_000.0) as i32,
            &dependencies,
            &a_state_map,
        );
        assert!(
            b_effective_capacity_fp < (b_base_capacity * 10_000.0) as i32,
            "B effective capacity must be reduced when A is Captured"
        );

        // Phase 3: B collapses (simulate legitimacy drain over ticks 8–13).
        for tick in 8u64..=13 {
            let legitimacy_drain = (tick - 7) as i32 * 900;
            let b_legitimacy = (8_500 - legitimacy_drain).max(0);
            let state_b = evaluate_transition(
                &mut inst_b, b_legitimacy, 1_000, &params, tick
            );
            if tick == 13 {
                assert_eq!(state_b, InstitutionState::Collapsed,
                    "B must collapse at tick 13 after sustained legitimacy drain");
            }
        }

        // Phase 4: C now depends on two failed institutions.
        let collapsed_map: std::collections::BTreeMap<u64, InstitutionState> =
            [(1001, InstitutionState::Captured),
             (1002, InstitutionState::Collapsed),
             (1003, InstitutionState::Stable)]
            .into_iter().collect();

        let c_base_capacity = 10_000i32;
        let c_effective_capacity_fp = effective_policy_capacity(
            1003, c_base_capacity, &dependencies, &collapsed_map
        );
        assert_eq!(
            c_effective_capacity_fp, 0,
            "C effective capacity must be 0 when B is Collapsed (dependency chain)"
        );

        // Phase 5: C collapses at tick 14.
        let state_c = evaluate_transition(
            &mut inst_c,
            params.legitimacy_contested_floor - 1,
            1_000,
            &params,
            14,
        );
        assert_eq!(state_c, InstitutionState::Collapsed,
            "C must collapse at tick 14 via cascade");

        // Phase 6: Insurgency modifier is amplified by two Collapsed institutions.
        let outputs_a = InstitutionOutputs::from_state(InstitutionState::Captured, 0.8, 0.7);
        let outputs_b = InstitutionOutputs::from_state(InstitutionState::Collapsed, 0.1, 0.1);
        let outputs_c = InstitutionOutputs::from_state(InstitutionState::Collapsed, 0.1, 0.1);

        let combined_insurgency = outputs_a.insurgency_modifier
            * outputs_b.insurgency_modifier
            * outputs_c.insurgency_modifier;

        assert!(
            combined_insurgency >= 4.0,
            "Combined insurgency modifier must >= 4.0 in collapse cascade: got {}",
            combined_insurgency
        );
    }
}
```

---

## 24. Extended Module Layout

The new crates and modules required to support §§17–23 are:

```
crates/institutions/
└── src/
    ├── lib.rs
    ├── fsm.rs
    ├── institution.rs
    ├── legitimacy.rs
    ├── capture.rs
    ├── propagation.rs
    ├── outputs.rs
    ├── params.rs
    ├── charter.rs              # NEW: InstitutionCharter, formation/dissolution
    ├── dependency.rs           # NEW: InstitutionDependency, effective_policy_capacity
    ├── shadow.rs               # NEW: ShadowInstitution, ShadowToFormalTransition
    ├── timeseries_extended.rs  # NEW: HotTierInstitutionCache, WarmTierWriter, replay
    └── tests/
        ├── fsm_tests.rs
        ├── capture_tests.rs
        ├── propagation_tests.rs
        ├── formation_tests.rs       # NEW
        ├── shadow_tests.rs          # NEW
        ├── scenario_tests.rs        # NEW
        └── cross_run_diff_tests.rs  # NEW

crates/citizens/
└── src/
    ├── lib.rs
    ├── lifecycle.rs
    ├── cohort.rs
    ├── metrics.rs
    ├── retirement.rs
    ├── ledger.rs
    ├── params.rs
    ├── health.rs               # NEW: CitizenHealthState, update_health, EnergyDebtParams
    ├── productivity.rs         # NEW: ProductivityCurveParams, compute_cohort_joule_output
    ├── demographics.rs         # NEW: CohortDemographics, compute_demographic_change
    ├── generational.rs         # NEW: GenerationalTransfer, GenerationalTransferParams
    └── tests/
        ├── lifecycle_tests.rs
        ├── retirement_tests.rs
        ├── monotonic_tests.rs
        ├── debt_tests.rs            # NEW
        ├── demographic_tests.rs     # NEW
        └── generational_tests.rs    # NEW
```

---

## 25. Extended Conservation Invariants

The following invariants supplement §14 and are enforced by the new test stubs.

### INV-11: Generational Transfer Joule Conservation

```
For all generational_transfer events in a canonical run:
    net_transfer_joules + tax_collected_joules
    == sum(lifetime_joule_credit_per_deceased - minimum_floor_joules
           for each deceased citizen)
```

No joules are created or destroyed by the transfer mechanics. The inheritance tax is fully credited to the retirement pool in the same tick.

### INV-12: Shadow Institution Influence Bounded

```
For all (run_id, tick, shadow_id):
    0 <= shadow_institutions.influence_score <= 10_000
    0 <= shadow_institutions.detection_risk <= 10_000
    0 <= shadow_institutions.coercion_level <= 10_000
```

### INV-13: Effective Policy Capacity Non-Increasing Under Dependency Failure

```
For any institution I with dependency D:
    IF D.state worsens (Stable → Contested → Captured → Collapsed),
    THEN effective_policy_capacity(I) is non-increasing.
```

No dependency failure may improve a dependent institution's effective capacity.

### INV-14: Cohort Population Non-Negative

```
For all (run_id, tick, cohort_id):
    cohort_demographics.population_after >= 0
    deaths_this_tick <= population_before
    population_after = population_before + births_this_tick - deaths_this_tick
```

### INV-15: Debt Non-Negative

```
For all (run_id, tick, cohort_id):
    CitizenHealthState.joule_debt >= 0
    (debt can reach 0 via repayment but never go negative)
```

### INV-16: Shadow Emergence Requires Formal Failure

```
For any shadow_institution with emerged_tick T:
    institution_states[displaced_formal_institution_id, T-1].state
    IN {Captured, Collapsed}
```

A shadow institution may only emerge in the shadow of a formally failed institution.

---

## Appendix C: Extended Policy Bundle YAML Fragment

```yaml
# Extended policy bundle additions for §§17–23.
# Appended to the base policy bundle defined in Appendix B.

citizen_health:
  welfare_recovery_rate: 80           # fixed-point units per tick at full welfare
  age_decay_youth: 10
  age_decay_working: 20
  age_decay_late_career: 60
  age_decay_elder: 120
  stress_health_drag: 40              # coefficient, applied as 40 * stress / 10_000
  surge_threshold_fp: 7000            # surge capacity level triggering bonus
  surge_capacity_bonus: 50

productivity_curve:
  base_rate_joules_per_tick: 20000000000   # 20 GJ/tick per citizen
  age_factor_youth: 4500
  age_factor_working: 10000
  age_factor_late_career: 8200
  age_factor_elder: 2500
  health_exponent_fp: 6000
  stress_drag_coeff_fp: 4000
  skill_factors: [6000, 8000, 10000, 12500, 16000]

energy_debt:
  joule_debt_default_threshold: 50000000000   # 50 GJ
  debt_repayment_rate_fp: 3000                # 30% of surplus
  debt_default_stress_spike: 2500
  debt_default_alienation_spike: 1500
  debt_default_welfare_penalty: 2000
  debt_stress_base_rate: 50

demographics:
  base_birth_rate_fp: 3               # births per 10_000 citizens per tick
  welfare_birth_multiplier_at_full: 12000
  energy_surplus_birth_bonus_fp: 1500
  alienation_birth_drag_fp: 3000
  death_rate_youth_fp: 2
  death_rate_working_fp: 3
  death_rate_late_career_fp: 12
  death_rate_elder_fp: 80
  health_death_multiplier_fp: 5000

generational_transfer:
  inheritance_enabled: false
  minimum_floor_joules: 0
  beneficiary_fraction_fp: 8000       # 80% to beneficiary cohort
  inheritance_tax_rate_fp: 2500       # 25% to retirement pool

institution_formation:
  creation_coalition_threshold_fp: 6000    # 60% coalition support required
  creation_initial_legitimacy_fp: 7000
  merger_reform_threshold_fp: 5000
  merger_window_ticks: 5
  capture_domain_split_threshold_fp: 7000
  split_legitimacy_penalty_factor_fp: 8500  # 0.85 multiplier on both successors

shadow_institutions:
  alpha_shadow_growth_fp: 500
  beta_scarcity_fp: 300
  gamma_reform_fp: 400
  delta_detection_fp: 3000
  opacity_shield_fp: 6000
  shadow_detection_threshold: 7500
  max_resource_extraction_joules: 1000000000000   # 1 TJ/tick at full influence
  min_failure_ticks_before_emergence: 10           # formal must fail for 10 ticks

timeseries:
  hot_tier_depth_ticks: 100
  warm_tier_partition_size_ticks: 1000
  cold_tier_threshold_tick: 10000
  incremental_aggregation_window: 100
  cross_run_comparison_tolerance_fp: 200   # diff threshold for trajectory comparison
```

---

**Version History (continued):**
- v1.1 (2026-02-21): Appended §§17–25. Adds extended citizen state vector (health, stress, joule_balance, joule_debt, social_trust, alienation_index, ideology_vector_id), productivity curve with age×health×stress formula, health dynamics model, energy debt mechanics and default cascade, multi-generation dynamics (birth rate, youth human capital, elder institutional memory, generational wealth transfer), institution formation/dissolution with charter ratification, merger/split mechanics, extended InstitutionType taxonomy with capability multipliers, cross-institution dependency graph, time-series extended architecture (partitioning, incremental aggregation, hot/warm/cold retention tiers, replay protocol, cross-run comparison queries), shadow institution mechanics with influence dynamics and shadow-to-formal transition pathways, 6 additional event schemas, 4 additional SQL tables, 8 additional test stubs, collapse cascade scenario, extended module layout, 6 new conservation invariants, and extended policy bundle YAML fragment.
