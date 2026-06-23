# CIV-0105: War, Diplomacy, and Shadow Networks Specification v1

**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Related Specs:**
- CIV-0001: Core Simulation Loop (determinism, tick architecture, quality bar)
- CIV-0103: Institutions, Time-Series, and Citizen Lifecycle (legitimacy coupling)
- CIV-0104: Minimal Constraint Set Theorem (coalition compatibility, enforcement bounds)

---

## 1. Executive Summary

This specification defines the **geopolitical simulation layer** for CivLab: a deterministic, fully audited system modeling three parallel channels through which polities interact:

1. **Overt Diplomacy** — treaty slots, influence capital, formal alliance/war states
2. **Trade and Sanctions** — formal resource flow network with interdiction mechanics
3. **Shadow Networks** — covert flows of finance, information, and materiel that persist under enforcement pressure

The system must be:
- **Deterministic per tick** — identical seed + state produces identical output
- **Bounded and conserved** — leakage non-negative, pressure bounded, coalition stability in `[0,1]`
- **Explicitly audited** — shadow influence is never silently applied; every flow is logged
- **Coupled to legitimacy** — enforcement intensity affects the legitimacy modifier with overreach detection

The three formal theorem chains from the research corpus (Sanctions Leakage Threshold, Authoritarian Backfire, and Coalition Stability) are the mathematical backbone of this system. Their threshold conditions — L₀, C₀, and the backfire point E* — are computed every tick and exposed as monitoring metrics.

### CIV Sim Integration Notes

This module operates in **Phase 2 (Policy Phase)** and **Phase 3 (Deterministic Transition)** of the CIV-0001 tick cycle. All RNG calls are confined to Phase 4 (Stochastic Event Phase) and must use `ChaCha20Rng` seeded from the policy bundle hash. No system clock. No float arithmetic in state transitions. All actor-pair evaluation is sorted by stable `ActorId`.

---

## 2. Diplomatic FSM

### 2.1 States

Each ordered pair `(actor_a, actor_b)` where `actor_a \< actor_b` (stable sort) holds exactly one `DiplomaticState` at any tick:

| State | Code | Description |
|---|---|---|
| `Cooperative` | 0 | Active trade, treaty obligations, mutual defense |
| `Strained` | 1 | Grievance accumulating; trade restricted; embassy downgraded |
| `Sanctioned` | 2 | Formal sanctions in force; coalition pressure applied |
| `Escalating` | 3 | Arms buildup, mobilization signaling, crisis window |
| `ActiveConflict` | 4 | Declared or undeclared war; battle mechanics live |
| `Deescalating` | 5 | Ceasefire in effect; legitimacy recovery window |
| `ColdWar` | 6 | No formal conflict; shadow proxy contest; no trade |
| `Alliance` | 7 | Mutual defense pact; shared intelligence; trade bonus |

### 2.2 Transition Triggers and Threshold Parameters

All transitions are deterministic given state + policy bundle. Thresholds are config-driven.

```
Transition Table (from → to : trigger condition)

Cooperative → Strained      : grievance_score >= STRAIN_THRESHOLD (default 0.35)
Cooperative → Alliance      : influence_capital >= ALLIANCE_COST AND ideology_alignment >= ALIGN_FLOOR
Strained    → Sanctioned    : coalition_size >= 1 AND pressure_score >= SANCTION_TRIGGER (0.55)
Strained    → Cooperative   : grievance_score < RECOVERY_FLOOR (0.15) for 3 consecutive ticks
Strained    → Escalating    : military_readiness_delta > READINESS_SPIKE (0.20)
Sanctioned  → Escalating    : scarcity_delta > SCARCITY_SPIKE (0.40) AND L0 > 1.0
Escalating  → ActiveConflict: casus_belli_valid AND war_payoff > peace_settlement_range
Escalating  → Deescalating  : negotiation_accepted (influence capital transfer)
ActiveConflict → Deescalating: war_termination_condition satisfied
Deescalating → Strained     : after CEASEFIRE_TICKS elapsed
Deescalating → Cooperative  : after CEASEFIRE_TICKS elapsed AND legitimacy_both >= LEGIT_FLOOR
Alliance    → Cooperative   : treaty_violation OR ideology_drift > DEFECT_THRESHOLD
Alliance    → Strained      : alliance_cost_imbalance > IMBALANCE_THRESHOLD
ColdWar     → Sanctioned    : shadow_exposure_event AND coalition_joins
```

### 2.3 Output Multipliers by State

Each state applies multipliers to downstream systems. These are exact `i64` fixed-point values (scaled by 1000):

| State | Trade Availability | Defense Spend Multiplier | Coalition Stability Bonus |
|---|---|---|---|
| Cooperative | 1000 | 900 | +50 |
| Strained | 650 | 1050 | +0 |
| Sanctioned | 200 | 1150 | -100 |
| Escalating | 400 | 1400 | -200 |
| ActiveConflict | 50 | 1800 | -350 |
| Deescalating | 350 | 1100 | +20 |
| ColdWar | 150 | 1300 | -200 |
| Alliance | 1200 | 850 | +150 |

All multipliers applied via integer multiplication then divided by 1000 to preserve fixed-point semantics.

### 2.4 Rust Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum DiplomaticState {
    Cooperative     = 0,
    Strained        = 1,
    Sanctioned      = 2,
    Escalating      = 3,
    ActiveConflict  = 4,
    Deescalating    = 5,
    ColdWar         = 6,
    Alliance        = 7,
}

impl DiplomaticState {
    /// Trade availability multiplier (fixed-point, divide by 1000).
    pub fn trade_availability_milli(&self) -> i64 {
        match self {
            Self::Cooperative    => 1000,
            Self::Strained       => 650,
            Self::Sanctioned     => 200,
            Self::Escalating     => 400,
            Self::ActiveConflict => 50,
            Self::Deescalating   => 350,
            Self::ColdWar        => 150,
            Self::Alliance       => 1200,
        }
    }

    /// Defense spend multiplier (fixed-point, divide by 1000).
    pub fn defense_spend_milli(&self) -> i64 {
        match self {
            Self::Cooperative    => 900,
            Self::Strained       => 1050,
            Self::Sanctioned     => 1150,
            Self::Escalating     => 1400,
            Self::ActiveConflict => 1800,
            Self::Deescalating   => 1100,
            Self::ColdWar        => 1300,
            Self::Alliance       => 850,
        }
    }

    /// Coalition stability bonus (signed integer, points).
    pub fn coalition_stability_bonus(&self) -> i32 {
        match self {
            Self::Cooperative    => 50,
            Self::Strained       => 0,
            Self::Sanctioned     => -100,
            Self::Escalating     => -200,
            Self::ActiveConflict => -350,
            Self::Deescalating   => 20,
            Self::ColdWar        => -200,
            Self::Alliance       => 150,
        }
    }
}
```

---

## 3. Conflict Mechanics

### 3.1 Casus Belli Validation

Before transitioning to `ActiveConflict`, the engine validates casus belli. Invalid casus belli blocks the transition and records a `CasusBelliRejected` event.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CasusBelli {
    TerritorialDispute { cell_id: u64 },
    SanctionBreakthrough { sanction_id: u64 },
    AllyDefense { ally_actor_id: u64 },
    ResourceCorridorSeizure { corridor_edge_id: u64 },
    ProxyConflict { shadow_node_id: u64 },
    RetaliationStrike { prior_incident_tick: u64 },
}

pub fn validate_casus_belli(
    cb: &CasusBelli,
    state: &WorldState,
    actor_a: ActorId,
    actor_b: ActorId,
) -> bool {
    match cb {
        CasusBelli::TerritorialDispute { cell_id } => {
            state.disputed_cells.contains(cell_id)
                && state.diplomatic_state(actor_a, actor_b) >= DiplomaticState::Escalating
        }
        CasusBelli::SanctionBreakthrough { sanction_id } => {
            state.sanctions.contains_key(sanction_id)
                && state.leakage_reproduction_number(actor_a, actor_b) > 100 // fixed-point > 1.0
        }
        CasusBelli::AllyDefense { ally_actor_id } => {
            state.alliance_exists(actor_a, *ally_actor_id)
                && state.diplomatic_state(*ally_actor_id, actor_b) == DiplomaticState::ActiveConflict
        }
        CasusBelli::ResourceCorridorSeizure { corridor_edge_id } => {
            state.corridor_owner(*corridor_edge_id) == actor_b
                && state.scarcity_delta(actor_a) > state.thresholds.scarcity_spike_milli
        }
        CasusBelli::ProxyConflict { shadow_node_id } => {
            state.shadow_node_owner(*shadow_node_id) == actor_b
                && state.shadow_exposure_confirmed(*shadow_node_id)
        }
        CasusBelli::RetaliationStrike { prior_incident_tick } => {
            state.tick.saturating_sub(*prior_incident_tick) <= state.thresholds.retaliation_window_ticks
        }
    }
}
```

### 3.2 Battle Mechanics

Battle resolution is **deterministic** (no RNG) in the Deterministic Transition Phase. Stochastic battle events (ambush, weather, morale shock) occur in Phase 4.

```
BattleOutcome = f(
    manpower_ratio,
    readiness_ratio,
    tech_ratio,
    morale_ratio,
    supply_sufficiency_index,
    terrain_modifier,
    intelligence_modifier
)
```

**Supply Sufficiency Index (SSI):**
```
SSI = (stockpile_weeks / target_weeks) * route_throughput_ratio * (1 - disruption_probability) * (1 - corruption_leakage_rate)
```

SSI dominates in prolonged conflict. When `SSI \< 0.5`, attrition doubles.

**Attrition Rate (per tick, fixed-point):**
```
attrition_milli = BASE_ATTRITION_MILLI
    + (1 - SSI_milli / 1000) * SUPPLY_ATTRITION_FACTOR
    + (1 - morale_milli / 1000) * MORALE_ATTRITION_FACTOR
    - tech_advantage_milli / TECH_DIVISOR
```

**Territorial Change Probability (per tick, fixed-point):**
```
territory_delta_milli = (manpower_ratio - 1000) * MANPOWER_WEIGHT
    + (readiness_ratio - 1000) * READINESS_WEIGHT
    + SSI_milli / SSI_DIVISOR
    - terrain_penalty_milli
```

### 3.3 Siege Mechanics

Infrastructure damage accumulates per tick of `ActiveConflict` in a theater:

```rust
pub struct SiegeDamage {
    pub energy_grid_damage_milli: i64,   // 0..1000; reduces grid capacity
    pub housing_stock_damage_milli: i64, // 0..1000; raises displacement
    pub transport_damage_milli: i64,     // 0..1000; reduces SSI
    pub industrial_damage_milli: i64,    // 0..1000; reduces defense production
}

pub fn compute_siege_damage(
    battle_intensity: i64, // 0..1000
    civilian_protection_policy: CivilianProtectionPolicy,
    tick: u64,
) -> SiegeDamage {
    let base = battle_intensity;
    let protection_factor = civilian_protection_policy.factor_milli(); // 500..1000
    SiegeDamage {
        energy_grid_damage_milli: (base * protection_factor / 1000).min(1000),
        housing_stock_damage_milli: (base * protection_factor / 1000 * 8 / 10).min(1000),
        transport_damage_milli: (base * 900 / 1000).min(1000),
        industrial_damage_milli: (base * 600 / 1000).min(1000),
    }
}
```

### 3.4 War Termination Conditions

A war terminates (transitions to `Deescalating`) when any of:

1. **Manpower exhaustion**: one actor's `manpower_pool \< EXHAUSTION_FLOOR`
2. **Legitimacy collapse**: one actor's `legitimacy \< COLLAPSE_FLOOR` for `COLLAPSE_TICKS` consecutive ticks
3. **Settlement accepted**: influence capital transfer clears bargaining range
4. **Coalition withdrawal**: sanctioning coalition dissolves (`C0 > COALITION_COLLAPSE_THRESHOLD` for attacker's coalition)
5. **SSI collapse**: attacker's `SSI \< SSI_COLLAPSE_FLOOR` for `SSI_COLLAPSE_TICKS` ticks

All termination checks are deterministic and happen in Phase 3.

---

## 4. Sanction Model

### 4.1 Formal Sanction Pressure Formula

Sanction pressure on target `j` from coalition `C`:

```
SanctionPressure(j, C, t) = σ(
    c1 * dependency(j ← C)
    + c2 * coalition_size(C)
    - c3 * substitutability(j)
    - c4 * leakage_throughput(j, t)
)
```

Where `σ` is a fixed-point sigmoid approximation (no floats):
```
σ(x) = 1000 / (1 + exp(-x/100))   // operating in milli-units
```

Parameters (default values, overridable in policy bundle):
- `c1 = 400` (dependency weight)
- `c2 = 200` (coalition size weight, per member)
- `c3 = 300` (substitutability weight)
- `c4 = 500` (leakage weight)

### 4.2 Leakage Formula

Leakage throughput `Λ` (black/gray import restoration) follows the **Sanctions Leakage Threshold Theorem**:

```
Λ(t+1) = Λ(t)
    + α * H(t) * (S(t) + η * ΔP(t)) * (1 + κ * Sel(t)) * (1 - Λ(t) / Λ_max)
    - β * (K(t) + ψ * E(t)) * G(t) * (1 - Sel(t)) * Λ(t)
```

Where (all fixed-point, scaled):
- `H(t)` = shadow network facilitation capacity (0..1000)
- `S(t)` = scarcity pressure inside target (0..1000)
- `ΔP(t)` = price wedge between shadow and official markets (0..1000)
- `Sel(t)` = enforcement selectivity/corruption (0..1000)
- `K(t)` = coalition interdiction budget (0..1000)
- `E(t)` = target internal enforcement intensity (0..1000)
- `G(t)` = governance integrity in target (0..1000)
- `Λ_max` = geographic/route capacity ceiling (config)

Parameters: `α = 12, η = 8, κ = 4, β = 15, ψ = 6` (defaults; all scaled by 100)

### 4.3 Leakage Reproduction Number L₀

The **Leakage Reproduction Number** is computed each tick and compared to the threshold of 1.0:

```
L0(t) = [α * H(t) * (S(t) + η * ΔP(t)) * (1 + κ * Sel(t))]
      / [β * (K(t) + ψ * E(t)) * G(t) * (1 - Sel(t))]
```

In fixed-point (scaled to avoid division by zero):
```rust
pub fn leakage_reproduction_number(p: &SanctionPressure) -> i64 {
    // All values 0..1000 (milli-units); result in milli-units (1000 = threshold)
    let numerator = p.shadow_facilitation_milli
        .saturating_mul(p.scarcity_milli + p.price_wedge_milli * 8 / 100)
        .saturating_mul(1000 + p.selectivity_milli * 4 / 100)
        / 1_000_000;
    let denominator = (p.coalition_interdiction_milli + p.enforcement_milli * 6 / 100)
        .saturating_mul(p.governance_integrity_milli)
        .saturating_mul(1000 - p.selectivity_milli)
        / 1_000_000;
    if denominator == 0 { return i64::MAX; }
    numerator * 1000 / denominator // result: 1000 = L0 of 1.0
}
```

**Invariant**: If `L0 > 1000` (i.e., L₀ > 1.0) for 3 consecutive ticks, a `sanctions.leakage_threshold_breached.v1` event is emitted.

### 4.4 Joint Constraint

The simulation enforces at all times:

1. `leakage >= 0` (enforced by saturation arithmetic)
2. `total_external_stress = pressure + leakage_reduction_effect` is bounded
3. Leakage cannot exceed `Λ_max` (config-bounded cap)
4. Enforcement cannot exceed `E_max(legitimacy, baseline_rights)` (legitimacy-bounded)

---

## 5. Shadow Network Model

### 5.1 Channel Types

Three distinct shadow channel types are modeled explicitly. All flows are audited — no silent application.

| Channel | Code | Description | Primary Metric |
|---|---|---|---|
| Financial | `F` | Black market capital flows, sanctions bypass, procurement leakage | `flow_amount_joules` |
| Information | `I` | Disinformation campaigns, espionage intelligence, propaganda | `influence_units` |
| Material | `M` | Smuggled goods, weapons, dual-use technology, energy re-exports | `flow_amount_joules` |

### 5.2 Shadow Flow Rates

Shadow capacity on each network edge evolves as:

```
c̃(e, t+1) = (1 - δ_s) * c̃(e, t) + η * ShadowGrowthRate(e, t)

ShadowGrowthRate(e, t) =
    α * S_i(t)
    + β * ΔP(t)
    - γ * E_i(t)
    + δ * CorruptionLevel_i(t)
```

Where `δ_s = 50` (decay per tick if scarcity eases, milli-units).

Actual flow through edge `e`:
```
FlowS(e, t) = c̃(e, t) * (1000 - interdiction_risk_milli(e, t)) / 1000
```

Interdiction risk depends on enforcement intensity, surveillance scope, and shadow network sophistication.

### 5.3 Detection Probability

Detection probability for shadow channel `c` at tick `t`:

```
P_detect(c, t) = σ(
    θ1 * enforcement_milli
    + θ2 * transparency_milli
    + θ3 * audit_intensity_milli
    - θ4 * network_sophistication_milli
    - θ5 * corruption_milli
)
```

Default parameters: `θ1 = 300, θ2 = 250, θ3 = 200, θ4 = 350, θ5 = 400`

When `P_detect > detection_threshold` (config, default 700 milli), a `shadow.network_signal.v1` event is emitted with the detected channel ID. The shadow node's `exposure_risk` is incremented.

### 5.4 Influence Application Mechanics

Shadow influence is **never silently applied**. Every tick, the system:

1. Computes shadow flow for each active channel
2. Applies influence via explicit `ShadowInfluenceApplication` records
3. Emits `shadow.network_signal.v1` for each application
4. Records the application in `shadow_flows` table with detection score

Shadow influence on formal institutions:
```
InfluencePressure = NodeInfluence * EdgeStrength * InstitutionSusceptibility / 1_000_000
```

If `InfluencePressure > capture_threshold`:
- Policy distortion applied (logged)
- Rent leakage increases
- Enforcement bias applied (logged)

Shadow network resource growth (fueling itself):
```
R_shadow(t+1) = R_shadow(t) + ν * ShadowFlow(t) - ExposureLoss(t)
```

Shadow profiteering from war:
```
R_shadow(t+1) += χ * DefenseSpend(t)  // war profiteering hook
```

### 5.5 Espionage and Sabotage

Espionage and sabotage actions are discrete events generated in Phase 4 (stochastic):

- **Espionage**: increases `intelligence_modifier` in battle resolution; reduces opponent's detection probability for 1d4 ticks
- **Sabotage**: increases `disruption_probability` on targeted corridor edges; reduces SSI
- **Disinformation**: increases `Di,t` (disinformation pressure) in coalition members; reduces `Effic_t`
- **False flag**: creates a synthetic `casus_belli` with `ProxyConflict` type; requires shadow node proof

All espionage/sabotage events are seeded from `ChaCha20Rng` using the policy bundle hash at tick start.

---

## 6. Coalition Stability Model

### 6.1 Formal Coalition Stability Score

For each coalition member `i`, per-member decay pressure:
```
Ψ(i, t) = α1 * Blowback(i,t) + α2 * Scarcity(i,t) + α3 * (1000 - Effic(t)) + α4 * Disinfo(i,t)
```

Per-member support:
```
Ω(i, t) = α5 * side_payments(i,t) + α6 * Legitimacy(i,t) + α7 * CommitmentPropensity(i,t)
```

Per-member stability ratio:
```
κ(i, t) = Ψ(i, t) / Ω(i, t)     // fixed-point: 1000 = threshold
```

**Coalition Stability Number C₀:**
```
C0(t) = (1 / |C|) * Σ_{i &isin; C} κ(i, t)
```

If `C0 \< 1000` (i.e., C₀ \< 1.0): coalition holds.
If `C0 > 1000` for sustained ticks: cascade exit begins.

### 6.2 Fatigue Dynamics

```
F(i, t+1) = F(i, t)
    + α1 * Blowback(i, t)
    + α2 * Scarcity(i, t)
    + α3 * (1000 - Effic(t))
    + α4 * Disinfo(i, t)
    - α5 * side_payments(i, t)
    - α6 * Legitimacy(i, t)
```

Default parameters: `α1..α6 = [12, 8, 10, 15, 20, 18]` (scaled by 100)

### 6.3 Disinformation Dynamics

```
D(i, t+1) = (1 - δ_D) * D(i, t)
    + β1 * ShadowSpend(t)
    + β2 * Polarization(i, t)
    - β3 * GovernanceIntegrity(i, t)
    - β4 * Transparency(i, t)
```

Where `δ_D = 50` (milli-units decay).

### 6.4 Perceived Effectiveness

```
Effic(t) = σ(
    γ1 * target_scarcity_delta(t)
    - γ2 * Λ(t)
    - γ3 * mean_disinfo(C, t)
)
```

Parameters: `γ1 = 400, γ2 = 600, γ3 = 350`

### 6.5 Cascade Exit Model

When member `i` exits:
1. Coalition interdiction `K(t)` drops by member's contribution
2. Leakage `Λ(t)` increases (less suppression)
3. `Effic(t)` falls (sanctions look ineffective)
4. Remaining members' fatigue rises
5. `C0(t)` rises further

This is a positive-feedback cascade, modeled iteratively in Phase 3. Maximum cascade depth per tick: 3 exits (prevents runaway in single tick).

---

## 7. Enforcement-Legitimacy Coupling

### 7.1 Mathematical Formula

Legitimacy `L(t)` evolution under enforcement pressure `E(t)`:

```
L(t+1) = L(t)
    + b1 * ServiceDelivery(t)
    - b2 * Scarcity(t)
    - b3 * Λ(t)             // visible leakage erodes trust
    - b4 * Φ(E(t), Sel(t))  // coercion injustice
```

Where the coercion injustice function `Φ`:
```
Φ(E, Sel) = E * (1000 + Sel * κ_sel) / 1000
```

With `&part;Φ/&part;E > 0` and `&part;Φ/&part;Sel > 0` (more enforcement hurts legitimacy; selective enforcement hurts disproportionately).

Parameters: `b1 = 15, b2 = 25, b3 = 10, b4 = 20, κ_sel = 500`

### 7.2 Legitimacy Modifier on Enforcement Capacity

Maximum safe enforcement is bounded by legitimacy and baseline rights:
```
E_max(t) = base_enforcement_cap * (L(t) / 1000) * (1 + baseline_rights_milli / 2000)
```

Enforcement that exceeds `E_max` triggers an `enforcement.overreach.v1` event and forces an automatic cap.

### 7.3 Overreach Detection

Enforcement overreach is detected when `E(t) > E*(t)`, the backfire threshold:

```
E*(t) = calibrated threshold where &part;Λ(t+k)/&part;E(t) > 0 for k &gt; 1
```

The backfire condition holds when:
```
b4 * &part;Φ/&part;E * (marginal legitimacy loss) > β * ψ * G * (1-Sel) * Λ   (suppression gain)
```

Simplified detection heuristic (computed each tick):
```
backfire_risk_milli = 1000 if (
    Sel > 300 AND G < 400 AND L < 500 AND S > 300
) else
    (Sel * G_inverse * L_inverse * S) / calibration_divisor
```

**Backfire Mechanic:**
When `backfire_risk_milli > 700`:
1. Enforcement increment is halved automatically
2. `enforcement.backfire_warning.v1` event emitted
3. If backfire_risk remains > 900 for 5 ticks, unrest spike is triggered in Phase 4

### 7.4 Overreach Spiral

The overreach spiral is a named attractor: `E ↑ → L ↓ → R ↑ → E ↑`.

Spiral entry condition: `backfire_risk_milli > 700` for `SPIRAL_ENTRY_TICKS` (default 5) consecutive ticks.
Spiral exit condition: `ServiceDelivery > RECOVERY_SERVICE_FLOOR` AND `Sel \< SEL_FLOOR` for 3 ticks.

Spiral state is tracked in `ConflictState.enforcement_spiral_ticks: u32`.

---

## 8. State Structs (Rust)

All structs derive `Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize`. No floats. Fixed-point integers scaled by 1000 unless otherwise noted.

```rust
use std::collections::BTreeMap;

pub type ActorId = u64;
pub type RunId = u64;
pub type Tick = u64;

/// Unique key for a diplomatic pair. Always actor_a < actor_b for canonical ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub struct ActorPairKey {
    pub actor_a: ActorId,
    pub actor_b: ActorId,
}

impl ActorPairKey {
    pub fn new(a: ActorId, b: ActorId) -> Self {
        if a <= b { Self { actor_a: a, actor_b: b } }
        else      { Self { actor_a: b, actor_b: a } }
    }
}

/// Complete diplomatic relationship between two actors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiplomaticRelation {
    pub pair: ActorPairKey,
    pub state: DiplomaticState,
    /// Accumulated grievance score (0..1000 milli-units).
    pub grievance_milli: i64,
    /// Influence capital (shared pool, non-negative).
    pub influence_capital: i64,
    /// Consecutive ticks in current state (for timeout transitions).
    pub state_ticks: u32,
    /// Sanction pressure score (0..1000 milli-units).
    pub pressure_score_milli: i64,
    /// Active treaty slot IDs.
    pub treaty_slots: Vec<u64>,
    /// Last validated casus belli (if any).
    pub casus_belli: Option<CasusBelli>,
}

/// Full sanction pressure record for a target polity under a coalition.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SanctionPressure {
    pub target_actor: ActorId,
    /// Coalition member actor IDs (sorted for determinism).
    pub coalition_members: Vec<ActorId>,
    /// Total pressure score (0..1000 milli-units).
    pub pressure_milli: i64,
    /// Leakage throughput Λ (0..Λ_max, joule-equivalent units).
    pub leakage_throughput: i64,
    /// Maximum leakage capacity Λ_max (config).
    pub leakage_max: i64,
    /// Shadow network facilitation H (0..1000).
    pub shadow_facilitation_milli: i64,
    /// Target scarcity S (0..1000).
    pub scarcity_milli: i64,
    /// Price wedge ΔP between shadow and official (0..1000).
    pub price_wedge_milli: i64,
    /// Enforcement selectivity/corruption Sel (0..1000).
    pub selectivity_milli: i64,
    /// Coalition interdiction budget K (0..1000).
    pub coalition_interdiction_milli: i64,
    /// Target internal enforcement E (0..1000).
    pub enforcement_milli: i64,
    /// Target governance integrity G (0..1000).
    pub governance_integrity_milli: i64,
    /// Computed L₀ (fixed-point; 1000 = threshold of 1.0).
    pub leakage_reproduction_number: i64,
    /// Ticks L₀ has exceeded 1000 consecutively.
    pub l0_breach_ticks: u32,
}

/// Per-member coalition membership record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CoalitionMembership {
    pub member_actor: ActorId,
    pub target_actor: ActorId,
    /// Blowback cost B_i (0..1000).
    pub blowback_milli: i64,
    /// Sanction fatigue F_i (0..1000).
    pub fatigue_milli: i64,
    /// Disinformation pressure D_i (0..1000).
    pub disinformation_milli: i64,
    /// Side-payment received s_i (joule-equivalent units).
    pub side_payments: i64,
    /// Domestic legitimacy L_i (0..1000).
    pub domestic_legitimacy_milli: i64,
    /// Commitment propensity H_i (0..1000; slow-moving cultural factor).
    pub commitment_propensity_milli: i64,
    /// Per-member stability ratio κ_i (fixed-point; 1000 = threshold).
    pub stability_ratio_milli: i64,
    /// Whether this member has exited the coalition.
    pub exited: bool,
    /// Tick of exit (0 if still active).
    pub exit_tick: Tick,
}

/// Shadow flow record for a single channel and tick.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShadowFlow {
    pub run_id: RunId,
    pub tick: Tick,
    pub channel: ShadowChannel,
    pub source_actor: ActorId,
    pub dest_actor: ActorId,
    /// Actual flow amount (joule-equivalent or influence units).
    pub flow_amount: i64,
    /// Shadow network capacity on this edge (0..capacity_max).
    pub edge_capacity: i64,
    /// Detection probability (0..1000 milli-units).
    pub detection_probability_milli: i64,
    /// Whether detection event was triggered this tick.
    pub detected: bool,
    /// Interdiction risk (0..1000).
    pub interdiction_risk_milli: i64,
    /// Corruption leakage fraction (0..1000).
    pub corruption_leakage_milli: i64,
}

/// Shadow channel type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub enum ShadowChannel {
    /// Black market capital flows, sanctions bypass, procurement leakage.
    Financial,
    /// Disinformation, espionage intelligence, propaganda.
    Information,
    /// Smuggled goods, weapons, dual-use technology, energy re-exports.
    Material,
}

/// Active conflict state for a pair of actors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConflictState {
    pub pair: ActorPairKey,
    pub tick_started: Tick,
    /// Validated casus belli.
    pub casus_belli: CasusBelli,
    /// Battle state per theater (sorted by theater_id for determinism).
    pub theaters: BTreeMap<u64, BattleState>,
    /// War termination condition (if met).
    pub termination: Option<WarTerminationCondition>,
    /// Consecutive ticks in enforcement overreach spiral.
    pub enforcement_spiral_ticks: u32,
    /// Consecutive ticks coalition stability C₀ > threshold.
    pub c0_breach_ticks: u32,
}

/// Battle state for a single theater.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BattleState {
    pub theater_id: u64,
    pub attacker: ActorId,
    pub defender: ActorId,
    /// Manpower (attacker, defender) in fixed-point units.
    pub manpower_attacker: i64,
    pub manpower_defender: i64,
    /// Readiness 0..1000.
    pub readiness_attacker_milli: i64,
    pub readiness_defender_milli: i64,
    /// Morale 0..1000.
    pub morale_attacker_milli: i64,
    pub morale_defender_milli: i64,
    /// Tech level (relative advantage, 0..2000; 1000 = parity).
    pub tech_ratio_milli: i64,
    /// Supply Sufficiency Index 0..1000.
    pub ssi_attacker_milli: i64,
    pub ssi_defender_milli: i64,
    /// Terrain modifier (0..1000; 1000 = neutral).
    pub terrain_modifier_milli: i64,
    /// Intelligence modifier (0..1000; 1000 = no advantage).
    pub intelligence_modifier_milli: i64,
    /// Accumulated siege damage.
    pub siege_damage: SiegeDamage,
    /// Attrition per tick (fixed-point units).
    pub attrition_rate_milli: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SiegeDamage {
    pub energy_grid_damage_milli: i64,
    pub housing_stock_damage_milli: i64,
    pub transport_damage_milli: i64,
    pub industrial_damage_milli: i64,
}

/// Trade availability for a polity after diplomatic and sanction adjustments.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TradeAvailability {
    pub actor: ActorId,
    /// Formal trade multiplier (0..1200 milli-units; 1000 = baseline).
    pub formal_trade_milli: i64,
    /// Shadow trade addition (joule-equivalent units).
    pub shadow_trade_joules: i64,
    /// Net effective trade (formal + shadow, post-leakage).
    pub net_effective_trade_joules: i64,
    /// Sanction interdiction active.
    pub sanctioned: bool,
    /// Embargo active (stronger than sanction).
    pub embargoed: bool,
}

/// War termination condition.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WarTerminationCondition {
    ManpowerExhaustion { exhausted_actor: ActorId },
    LegitimacyCollapse { collapsed_actor: ActorId },
    NegotiatedSettlement { influence_transferred: i64 },
    CoalitionWithdrawal,
    SsiCollapse { actor: ActorId },
}
```

---

## 9. Rust Module Layout

```
crates/
├── diplomacy/
│   └── src/
│       ├── lib.rs                  -- re-exports; module declarations
│       ├── state.rs                -- DiplomaticState enum + multiplier methods
│       ├── relation.rs             -- DiplomaticRelation + ActorPairKey
│       ├── transitions.rs          -- FSM transition logic (deterministic)
│       ├── casus_belli.rs          -- CasusBelli validation
│       ├── sanctions.rs            -- SanctionPressure, leakage model, L₀
│       ├── coalition.rs            -- CoalitionMembership, C₀, fatigue, disinformation
│       ├── trade.rs                -- TradeAvailability computation
│       ├── influence.rs            -- Influence capital mechanics, treaty slots
│       └── tests/
│           ├── determinism.rs      -- replay consistency tests
│           ├── leakage.rs          -- L₀ directionality tests
│           ├── coalition.rs        -- C₀ bounded stability tests
│           └── enforcement.rs      -- backfire region tests
│
└── conflict/
    └── src/
        ├── lib.rs                  -- re-exports; module declarations
        ├── state.rs                -- ConflictState, BattleState, SiegeDamage
        ├── battle.rs               -- deterministic battle resolution
        ├── siege.rs                -- infrastructure damage model
        ├── logistics.rs            -- SSI computation, stockpile model
        ├── termination.rs          -- war termination condition checks
        ├── shadow.rs               -- ShadowFlow, ShadowChannel, detection
        ├── enforcement.rs          -- E-L coupling, overreach, backfire meter
        └── tests/
            ├── determinism.rs
            ├── battle_bounds.rs
            ├── shadow_audit.rs
            └── enforcement_backfire.rs
```

Both crates are added to the workspace in `Cargo.toml`:
```toml
[workspace]
members = [
  "crates/engine",
  "crates/policy",
  "crates/metrics",
  "crates/io",
  "crates/server",
  "crates/diplomacy",   # new
  "crates/conflict",    # new
]
```

---

## 10. Event Contracts (JSON Schemas)

### 10.1 `diplomacy.state_changed.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "diplomacy.state_changed.v1",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "actor_a", "actor_b",
               "from_state", "to_state", "trigger", "pressure_score_milli",
               "grievance_milli", "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":           { "type": "string", "const": "diplomacy.state_changed.v1" },
    "version":              { "type": "string", "const": "1" },
    "run_id":               { "type": "integer", "minimum": 0 },
    "tick":                 { "type": "integer", "minimum": 0 },
    "actor_a":              { "type": "integer", "minimum": 0 },
    "actor_b":              { "type": "integer", "minimum": 0 },
    "from_state":           { "type": "string", "enum": ["Cooperative","Strained","Sanctioned","Escalating","ActiveConflict","Deescalating","ColdWar","Alliance"] },
    "to_state":             { "type": "string", "enum": ["Cooperative","Strained","Sanctioned","Escalating","ActiveConflict","Deescalating","ColdWar","Alliance"] },
    "trigger":              { "type": "string" },
    "pressure_score_milli": { "type": "integer", "minimum": 0, "maximum": 1000 },
    "grievance_milli":      { "type": "integer", "minimum": 0, "maximum": 1000 },
    "influence_capital":    { "type": "integer" },
    "state_hash":           { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 10.2 `sanctions.leakage_estimated.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "sanctions.leakage_estimated.v1",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "target_actor",
               "coalition_size", "leakage_throughput", "leakage_max",
               "leakage_reproduction_number", "l0_breach_ticks",
               "pressure_milli", "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                    { "type": "string", "const": "sanctions.leakage_estimated.v1" },
    "version":                       { "type": "string", "const": "1" },
    "run_id":                        { "type": "integer", "minimum": 0 },
    "tick":                          { "type": "integer", "minimum": 0 },
    "target_actor":                  { "type": "integer", "minimum": 0 },
    "coalition_size":                { "type": "integer", "minimum": 0 },
    "leakage_throughput":            { "type": "integer", "minimum": 0 },
    "leakage_max":                   { "type": "integer", "minimum": 0 },
    "leakage_reproduction_number":   { "type": "integer", "minimum": 0,
                                       "description": "Fixed-point; 1000 = L0 of 1.0" },
    "l0_breach_ticks":               { "type": "integer", "minimum": 0 },
    "pressure_milli":                { "type": "integer", "minimum": 0, "maximum": 1000 },
    "shadow_facilitation_milli":     { "type": "integer", "minimum": 0, "maximum": 1000 },
    "scarcity_milli":                { "type": "integer", "minimum": 0, "maximum": 1000 },
    "enforcement_milli":             { "type": "integer", "minimum": 0, "maximum": 1000 },
    "governance_integrity_milli":    { "type": "integer", "minimum": 0, "maximum": 1000 },
    "selectivity_milli":             { "type": "integer", "minimum": 0, "maximum": 1000 },
    "state_hash":                    { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 10.3 `shadow.network_signal.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "shadow.network_signal.v1",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "channel",
               "source_actor", "dest_actor", "flow_amount",
               "detection_probability_milli", "detected", "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                      { "type": "string", "const": "shadow.network_signal.v1" },
    "version":                         { "type": "string", "const": "1" },
    "run_id":                          { "type": "integer", "minimum": 0 },
    "tick":                            { "type": "integer", "minimum": 0 },
    "channel":                         { "type": "string", "enum": ["Financial","Information","Material"] },
    "source_actor":                    { "type": "integer", "minimum": 0 },
    "dest_actor":                      { "type": "integer", "minimum": 0 },
    "flow_amount":                     { "type": "integer", "minimum": 0 },
    "edge_capacity":                   { "type": "integer", "minimum": 0 },
    "detection_probability_milli":     { "type": "integer", "minimum": 0, "maximum": 1000 },
    "detected":                        { "type": "boolean" },
    "interdiction_risk_milli":         { "type": "integer", "minimum": 0, "maximum": 1000 },
    "corruption_leakage_milli":        { "type": "integer", "minimum": 0, "maximum": 1000 },
    "state_hash":                      { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 10.4 `conflict.battle_resolved.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "conflict.battle_resolved.v1",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "theater_id",
               "attacker", "defender", "attacker_attrition", "defender_attrition",
               "ssi_attacker_milli", "ssi_defender_milli", "territory_delta_milli",
               "siege_damage", "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":              { "type": "string", "const": "conflict.battle_resolved.v1" },
    "version":                 { "type": "string", "const": "1" },
    "run_id":                  { "type": "integer", "minimum": 0 },
    "tick":                    { "type": "integer", "minimum": 0 },
    "theater_id":              { "type": "integer", "minimum": 0 },
    "attacker":                { "type": "integer", "minimum": 0 },
    "defender":                { "type": "integer", "minimum": 0 },
    "attacker_attrition":      { "type": "integer", "minimum": 0 },
    "defender_attrition":      { "type": "integer", "minimum": 0 },
    "ssi_attacker_milli":      { "type": "integer", "minimum": 0, "maximum": 1000 },
    "ssi_defender_milli":      { "type": "integer", "minimum": 0, "maximum": 1000 },
    "territory_delta_milli":   { "type": "integer" },
    "morale_attacker_delta":   { "type": "integer" },
    "morale_defender_delta":   { "type": "integer" },
    "siege_damage": {
      "type": "object",
      "required": ["energy_grid_damage_milli","housing_stock_damage_milli",
                   "transport_damage_milli","industrial_damage_milli"],
      "properties": {
        "energy_grid_damage_milli":    { "type": "integer", "minimum": 0, "maximum": 1000 },
        "housing_stock_damage_milli":  { "type": "integer", "minimum": 0, "maximum": 1000 },
        "transport_damage_milli":      { "type": "integer", "minimum": 0, "maximum": 1000 },
        "industrial_damage_milli":     { "type": "integer", "minimum": 0, "maximum": 1000 }
      }
    },
    "state_hash":              { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 10.5 `conflict.war_declared.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "conflict.war_declared.v1",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "actor_a", "actor_b",
               "casus_belli_type", "casus_belli_valid", "prior_state", "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":        { "type": "string", "const": "conflict.war_declared.v1" },
    "version":           { "type": "string", "const": "1" },
    "run_id":            { "type": "integer", "minimum": 0 },
    "tick":              { "type": "integer", "minimum": 0 },
    "actor_a":           { "type": "integer", "minimum": 0 },
    "actor_b":           { "type": "integer", "minimum": 0 },
    "casus_belli_type":  { "type": "string", "enum": ["TerritorialDispute","SanctionBreakthrough","AllyDefense","ResourceCorridorSeizure","ProxyConflict","RetaliationStrike"] },
    "casus_belli_valid": { "type": "boolean" },
    "prior_state":       { "type": "string", "enum": ["Cooperative","Strained","Sanctioned","Escalating","ActiveConflict","Deescalating","ColdWar","Alliance"] },
    "influence_capital_at_declaration": { "type": "integer" },
    "state_hash":        { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 10.6 `coalition.member_defected.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "coalition.member_defected.v1",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "member_actor",
               "target_actor", "fatigue_milli", "disinformation_milli",
               "stability_ratio_milli", "remaining_members",
               "coalition_c0_milli", "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":            { "type": "string", "const": "coalition.member_defected.v1" },
    "version":               { "type": "string", "const": "1" },
    "run_id":                { "type": "integer", "minimum": 0 },
    "tick":                  { "type": "integer", "minimum": 0 },
    "member_actor":          { "type": "integer", "minimum": 0 },
    "target_actor":          { "type": "integer", "minimum": 0 },
    "fatigue_milli":         { "type": "integer", "minimum": 0, "maximum": 1000 },
    "disinformation_milli":  { "type": "integer", "minimum": 0, "maximum": 1000 },
    "stability_ratio_milli": { "type": "integer", "minimum": 0 },
    "remaining_members":     { "type": "integer", "minimum": 0 },
    "coalition_c0_milli":    { "type": "integer", "minimum": 0,
                               "description": "C0 at time of defection; 1000 = threshold" },
    "state_hash":            { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

---

## 11. Database Schema (DDL)

```sql
-- =========================================================
-- CIV-0105 Database Schema
-- All tables are append-only (no UPDATE/DELETE in sim paths).
-- All tick-keyed. run_id partitions separate simulation runs.
-- =========================================================

CREATE TABLE diplomatic_states (
    id                    BIGSERIAL PRIMARY KEY,
    run_id                BIGINT    NOT NULL,
    tick                  BIGINT    NOT NULL,
    actor_a               BIGINT    NOT NULL,
    actor_b               BIGINT    NOT NULL,
    state                 SMALLINT  NOT NULL,   -- DiplomaticState ordinal
    grievance_milli       INTEGER   NOT NULL DEFAULT 0 CHECK (grievance_milli BETWEEN 0 AND 1000),
    pressure_score_milli  INTEGER   NOT NULL DEFAULT 0 CHECK (pressure_score_milli BETWEEN 0 AND 1000),
    influence_capital     BIGINT    NOT NULL DEFAULT 0 CHECK (influence_capital >= 0),
    state_ticks           INTEGER   NOT NULL DEFAULT 0 CHECK (state_ticks >= 0),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT actor_order CHECK (actor_a < actor_b)
);

CREATE INDEX idx_diplomatic_states_run_tick ON diplomatic_states (run_id, tick);
CREATE INDEX idx_diplomatic_states_pair     ON diplomatic_states (run_id, actor_a, actor_b, tick DESC);

-- --------------------------------------------------------

CREATE TABLE sanction_records (
    id                            BIGSERIAL PRIMARY KEY,
    run_id                        BIGINT    NOT NULL,
    tick                          BIGINT    NOT NULL,
    target_actor                  BIGINT    NOT NULL,
    coalition_members             BIGINT[]  NOT NULL,
    pressure_milli                INTEGER   NOT NULL CHECK (pressure_milli BETWEEN 0 AND 1000),
    leakage_throughput            BIGINT    NOT NULL CHECK (leakage_throughput >= 0),
    leakage_max                   BIGINT    NOT NULL CHECK (leakage_max > 0),
    leakage_reproduction_number   INTEGER   NOT NULL CHECK (leakage_reproduction_number >= 0),
    l0_breach_ticks               INTEGER   NOT NULL DEFAULT 0,
    shadow_facilitation_milli     INTEGER   NOT NULL CHECK (shadow_facilitation_milli BETWEEN 0 AND 1000),
    scarcity_milli                INTEGER   NOT NULL CHECK (scarcity_milli BETWEEN 0 AND 1000),
    price_wedge_milli             INTEGER   NOT NULL CHECK (price_wedge_milli BETWEEN 0 AND 1000),
    selectivity_milli             INTEGER   NOT NULL CHECK (selectivity_milli BETWEEN 0 AND 1000),
    coalition_interdiction_milli  INTEGER   NOT NULL CHECK (coalition_interdiction_milli BETWEEN 0 AND 1000),
    enforcement_milli             INTEGER   NOT NULL CHECK (enforcement_milli BETWEEN 0 AND 1000),
    governance_integrity_milli    INTEGER   NOT NULL CHECK (governance_integrity_milli BETWEEN 0 AND 1000),
    created_at                    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sanction_records_run_tick   ON sanction_records (run_id, tick);
CREATE INDEX idx_sanction_records_target     ON sanction_records (run_id, target_actor, tick DESC);

-- --------------------------------------------------------

CREATE TABLE shadow_flows (
    id                           BIGSERIAL PRIMARY KEY,
    run_id                       BIGINT    NOT NULL,
    tick                         BIGINT    NOT NULL,
    channel                      SMALLINT  NOT NULL,  -- 0=Financial,1=Information,2=Material
    source_actor                 BIGINT    NOT NULL,
    dest_actor                   BIGINT    NOT NULL,
    flow_amount                  BIGINT    NOT NULL CHECK (flow_amount >= 0),
    edge_capacity                BIGINT    NOT NULL CHECK (edge_capacity >= 0),
    detection_probability_milli  INTEGER   NOT NULL CHECK (detection_probability_milli BETWEEN 0 AND 1000),
    detected                     BOOLEAN   NOT NULL DEFAULT FALSE,
    interdiction_risk_milli      INTEGER   NOT NULL CHECK (interdiction_risk_milli BETWEEN 0 AND 1000),
    corruption_leakage_milli     INTEGER   NOT NULL CHECK (corruption_leakage_milli BETWEEN 0 AND 1000),
    created_at                   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_shadow_flows_run_tick     ON shadow_flows (run_id, tick);
CREATE INDEX idx_shadow_flows_channel      ON shadow_flows (run_id, channel, tick DESC);
CREATE INDEX idx_shadow_flows_detection    ON shadow_flows (run_id, detected) WHERE detected = TRUE;

-- --------------------------------------------------------

CREATE TABLE conflict_history (
    id                     BIGSERIAL PRIMARY KEY,
    run_id                 BIGINT    NOT NULL,
    tick                   BIGINT    NOT NULL,
    actor_a                BIGINT    NOT NULL,
    actor_b                BIGINT    NOT NULL,
    theater_id             BIGINT    NOT NULL,
    casus_belli_type       SMALLINT  NOT NULL,
    attacker               BIGINT    NOT NULL,
    attacker_manpower      BIGINT    NOT NULL,
    defender_manpower      BIGINT    NOT NULL,
    attacker_attrition     BIGINT    NOT NULL,
    defender_attrition     BIGINT    NOT NULL,
    ssi_attacker_milli     INTEGER   NOT NULL CHECK (ssi_attacker_milli BETWEEN 0 AND 1000),
    ssi_defender_milli     INTEGER   NOT NULL CHECK (ssi_defender_milli BETWEEN 0 AND 1000),
    territory_delta_milli  INTEGER   NOT NULL,
    termination_type       SMALLINT,           -- NULL if conflict ongoing
    enforcement_spiral     BOOLEAN   NOT NULL DEFAULT FALSE,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT actor_order CHECK (actor_a < actor_b)
);

CREATE INDEX idx_conflict_history_run_tick ON conflict_history (run_id, tick);
CREATE INDEX idx_conflict_history_pair     ON conflict_history (run_id, actor_a, actor_b, tick DESC);
CREATE INDEX idx_conflict_history_active   ON conflict_history (run_id, termination_type)
    WHERE termination_type IS NULL;

-- --------------------------------------------------------

CREATE TABLE coalition_members (
    id                          BIGSERIAL PRIMARY KEY,
    run_id                      BIGINT    NOT NULL,
    tick                        BIGINT    NOT NULL,
    member_actor                BIGINT    NOT NULL,
    target_actor                BIGINT    NOT NULL,
    blowback_milli              INTEGER   NOT NULL CHECK (blowback_milli BETWEEN 0 AND 1000),
    fatigue_milli               INTEGER   NOT NULL CHECK (fatigue_milli BETWEEN 0 AND 1000),
    disinformation_milli        INTEGER   NOT NULL CHECK (disinformation_milli BETWEEN 0 AND 1000),
    side_payments               BIGINT    NOT NULL DEFAULT 0,
    domestic_legitimacy_milli   INTEGER   NOT NULL CHECK (domestic_legitimacy_milli BETWEEN 0 AND 1000),
    commitment_propensity_milli INTEGER   NOT NULL CHECK (commitment_propensity_milli BETWEEN 0 AND 1000),
    stability_ratio_milli       INTEGER   NOT NULL CHECK (stability_ratio_milli >= 0),
    exited                      BOOLEAN   NOT NULL DEFAULT FALSE,
    exit_tick                   BIGINT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_coalition_members_run_tick  ON coalition_members (run_id, tick);
CREATE INDEX idx_coalition_members_target    ON coalition_members (run_id, target_actor, tick DESC);
CREATE INDEX idx_coalition_members_active    ON coalition_members (run_id, member_actor)
    WHERE exited = FALSE;
```

---

## 12. Determinism Constraints

### 12.1 Actor-Pair Evaluation Ordering

All actor-pair transitions are evaluated in sorted order by `ActorPairKey` (which enforces `actor_a \< actor_b`). The full evaluation order for any tick is:

```rust
// In Phase 3 (Deterministic Transition):
let mut pairs: Vec<ActorPairKey> = state.diplomatic_relations.keys().cloned().collect();
pairs.sort(); // ActorPairKey implements Ord via (actor_a, actor_b) lexicographic order
for pair in &pairs {
    evaluate_transition(pair, state, control);
}
```

**Invariant**: No `HashMap` in critical paths. All relation maps are `BTreeMap \< ActorPairKey, DiplomaticRelation>`.

### 12.2 Shadow Flow Rounding

Shadow flows are integer-valued. Fractional results from multiplication are truncated toward zero (Rust default for integer division). This must be consistent across all platforms. No `f64` conversion permitted.

```rust
// Correct:
let flow = (capacity * (1000 - interdiction_risk_milli)) / 1000;

// Forbidden:
let flow = (capacity as f64 * (1.0 - interdiction_risk as f64 / 1000.0)) as i64;
```

### 12.3 Battle RNG Seeding

All stochastic battle events (Phase 4) seed from the policy bundle hash:

```rust
// Policy bundle hash is computed deterministically in Phase 2.
let seed = policy_bundle_hash ^ (tick << 32) ^ theater_id;
let mut rng = ChaCha20Rng::seed_from_u64(seed);
```

The policy bundle hash includes: all active policy parameters, the current tick, and the sorted actor ID list. This ensures that the same policy configuration always produces the same stochastic events.

### 12.4 Coalition Cascade Limit

Maximum 3 coalition exits per tick (prevents runaway cascade within a single deterministic phase). After 3 exits, remaining at-risk members are deferred to the next tick.

### 12.5 No System Time

No `SystemTime::now()` or `Instant::now()` in any simulation path. Simulation clock is `tick: u64` only. All timestamps in database records use the server wall clock only for audit/logging purposes; they have no effect on simulation logic.

---

## 13. Conservation Invariants

### 13.1 Leakage Non-Negativity

**Invariant**: `Λ(t) >= 0` at all times.

Enforced by:
1. `i64::saturating_sub` on all leakage decrements
2. `i64::max(0, ...)` guard at end of each tick's leakage update
3. Property test: `test_leakage_non_negative`

### 13.2 Sanction Pressure Bounded

**Invariant**: `pressure_milli &isin; [0, 1000]` at all times.

Enforced by:
1. Fixed-point sigmoid `σ` saturates at 1000
2. All pressure accumulators clipped to `[0, 1000]`
3. Property test: `test_pressure_bounded`

### 13.3 Coalition Stability in `[0, &infin;)`

**Invariant**: Per-member `κ(i,t) >= 0` (since `Ψ,Ω &gt; 0`).
**Invariant**: `C0(t) &gt; 0`.

C₀ is not bounded above (can exceed 1000 to signal coalition collapse), but individual inputs are bounded:
- `Ψ(i,t) &isin; [0, 4000]` (sum of four 0..1000 terms)
- `Ω(i,t) &isin; [0, 3000]` (sum of three 0..1000 terms)
- `κ(i,t) &isin; [0, 4000 / max(Ω,1)]` (bounded by input ranges)

Property test: `test_coalition_stability_bounded`

### 13.4 Enforcement Bounded by Legitimacy

**Invariant**: `E(t) <= E_max(L(t), baseline_rights)` at all times.

Enforced by:
1. Enforcement clipped to `E_max` computed before application
2. Overreach event emitted if clip was required
3. Property test: `test_enforcement_bounded_by_legitimacy`

### 13.5 Trade Availability Non-Negative

**Invariant**: `net_effective_trade_joules >= 0`.

Shadow trade addition is non-negative (flows are non-negative) and formal trade uses non-negative multiplier. Property test: `test_trade_availability_non_negative`.

---

## 14. Failure Modes

### 14.1 Shadow Network Detection Cascade

**Trigger**: Multiple `shadow.network_signal.v1` events in same tick with `detected = true`.

**Cascade behavior**: Each detection event raises `exposure_risk` on the shadow node. When `exposure_risk > EXPOSURE_THRESHOLD` (config, default 700), a `shadow.node_exposed.v1` event fires, reducing `InfluenceScore` and `ResourceBase` of that node by `EXPOSURE_PENALTY_MILLI` (default 400).

**Cascade limit**: Maximum 5 exposures per tick across all shadow nodes. Excess exposures are deferred.

**Recovery**: Shadow nodes recover at `recovery_rate_milli` per tick (default 20) when `exposure_risk \< 300`.

### 14.2 Enforcement Overreach Spiral

**Trigger**: `backfire_risk_milli > 700` for `SPIRAL_ENTRY_TICKS` (5) consecutive ticks.

**Spiral behavior**: Each tick in spiral, `Sel(t)` increases by `SEL_DRIFT_RATE` (30 milli-units), `G(t)` decreases by `INTEGRITY_DRAIN_RATE` (20 milli-units), and `L(t)` decreases by `LEGITIMACY_DRAIN_RATE` (15 milli-units). This compounds the backfire condition.

**Spiral exit**: `ServiceDelivery > RECOVERY_SERVICE_FLOOR` AND `Sel \< 300` for 3 consecutive ticks.

**Unrecoverable state**: If `L(t) < COLLAPSE_FLOOR` (100 milli-units) during spiral, `DiplomaticState::ActiveConflict` or `DiplomaticState::Deescalating` transitions may be forced by internal legitimacy collapse.

### 14.3 Coalition Dissolution

**Trigger**: `C0(t) > 1000` for `C0_BREACH_TICKS` (3) consecutive ticks.

**Dissolution behavior**: All members with `κ(i,t) > 1200` are marked `exited = true` (up to 3 per tick, cascade limit). Interdiction budget `K(t)` reduced by exiting members' contributions. Leakage `Λ(t)` rises. Effectiveness `Effic(t)` falls.

**Dissolution chain**: Coalition dissolution may trigger `DiplomaticState` transition from `Sanctioned → Escalating` for the target if `L0 > 1000` post-dissolution.

**Prevention mechanisms**: Side-payments, governance integrity improvement, humanitarian exceptions reducing target scarcity (which reduces `ΔP` and `S`).

### 14.4 Runaway Conflict Escalation

**Trigger**: All of the following simultaneously:
- `DiplomaticState::Escalating` for `ESCALATION_TICKS` (10) ticks
- `shadow_provocation_events > PROVOCATION_THRESHOLD` in last 5 ticks
- `scarcity_milli(actor_a) > 600` OR `scarcity_milli(actor_b) > 600`
- No negotiation accepted

**Escalation mechanic**: Automatic transition to `ActiveConflict` with `CasusBelli::RetaliationStrike` or `CasusBelli::ResourceCorridorSeizure` depending on which scarcity trigger fired.

**Runaway indicators**: War burden rising, legitimacy declining, SSI declining, enforcement spiral active — any combination produces compounding feedback.

---

## 15. Acceptance Test Suite

### 15.1 Replay Consistency

```rust
#[test]
fn test_diplomacy_replay_consistency() {
    let seed = 42u64;
    let state0 = DiplomacyTestFixture::initial_state();
    let control = DiplomacyTestFixture::policy_bundle();

    let (snapshot1, state1) = diplomacy_tick(&state0, &control, seed);
    let (snapshot2, state2) = diplomacy_tick(&state0, &control, seed);

    assert_eq!(snapshot1, snapshot2, "Diplomacy tick must be deterministic");
    assert_eq!(state1, state2, "State must be identical under same seed+control");
}

#[test]
fn test_conflict_replay_consistency() {
    let seed = 99u64;
    let state0 = ConflictTestFixture::active_war_state();
    let control = ConflictTestFixture::wartime_policy();

    let (snapshot1, state1) = conflict_tick(&state0, &control, seed);
    let (snapshot2, state2) = conflict_tick(&state0, &control, seed);

    assert_eq!(snapshot1, snapshot2, "Conflict tick must be deterministic");
    assert_eq!(state1, state2);
}
```

### 15.2 Leakage-Enforcement Directionality

```rust
#[test]
fn test_leakage_decreases_with_enforcement() {
    let base = SanctionPressure::test_default();

    // Higher enforcement → lower next-tick leakage (direct effect, short run)
    let low_enforce  = SanctionPressure { enforcement_milli: 200, ..base.clone() };
    let high_enforce = SanctionPressure { enforcement_milli: 800, ..base.clone() };

    let low_leakage_next  = compute_leakage_next_tick(&low_enforce);
    let high_leakage_next = compute_leakage_next_tick(&high_enforce);

    assert!(
        high_leakage_next < low_leakage_next,
        "Higher enforcement must reduce leakage in short run (direct effect)"
    );
}

#[test]
fn test_backfire_region_legitimacy_coupling() {
    // When selectivity is high and governance low, enforcement reduces legitimacy
    let p = EnforcementParams {
        enforcement_milli: 900,
        selectivity_milli: 700,
        governance_integrity_milli: 200,
        legitimacy_milli: 400,
        scarcity_milli: 500,
    };

    let legitimacy_after = compute_legitimacy_after_enforcement(&p);
    assert!(legitimacy_after < p.legitimacy_milli, "High selective enforcement must reduce legitimacy");

    let backfire_risk = compute_backfire_risk(&p);
    assert!(backfire_risk > 700, "Backfire risk must exceed warning threshold in this config");
}

#[test]
fn test_leakage_non_negative() {
    proptest!(|(
        h in 0i64..=1000,
        s in 0i64..=1000,
        dp in 0i64..=1000,
        sel in 0i64..=1000,
        k in 0i64..=1000,
        e in 0i64..=1000,
        g in 0i64..=1000,
        lambda_init in 0i64..=50000,
    )| {
        let p = SanctionPressure {
            shadow_facilitation_milli: h,
            scarcity_milli: s,
            price_wedge_milli: dp,
            selectivity_milli: sel,
            coalition_interdiction_milli: k,
            enforcement_milli: e,
            governance_integrity_milli: g,
            leakage_throughput: lambda_init,
            leakage_max: 100_000,
            ..SanctionPressure::default()
        };
        let next = compute_leakage_next_tick(&p);
        prop_assert!(next >= 0, "Leakage must remain non-negative");
    });
}
```

### 15.3 Coalition Bounded Stability

```rust
#[test]
fn test_coalition_stability_bounded() {
    proptest!(|(
        blowback in 0i64..=1000,
        fatigue in 0i64..=1000,
        disinfo in 0i64..=1000,
        side_payments in 0i64..=1000,
        legitimacy in 0i64..=1000,
        commitment in 0i64..=1000,
    )| {
        let member = CoalitionMembership {
            blowback_milli: blowback,
            fatigue_milli: fatigue,
            disinformation_milli: disinfo,
            side_payments,
            domestic_legitimacy_milli: legitimacy,
            commitment_propensity_milli: commitment,
            ..CoalitionMembership::default()
        };
        let psi = compute_decay_pressure(&member);
        let omega = compute_support(&member);
        let kappa = if omega == 0 { i64::MAX } else { psi * 1000 / omega };

        prop_assert!(psi >= 0, "Decay pressure must be non-negative");
        prop_assert!(omega >= 0, "Support must be non-negative");
        prop_assert!(kappa >= 0, "Stability ratio must be non-negative");
    });
}

#[test]
fn test_pressure_bounded() {
    proptest!(|(
        dependency in 0i64..=1000,
        coalition_size in 0usize..=20,
        substitutability in 0i64..=1000,
        leakage in 0i64..=100_000,
    )| {
        let pressure = compute_sanction_pressure(
            dependency,
            coalition_size as i64,
            substitutability,
            leakage,
        );
        prop_assert!(pressure >= 0, "Pressure must be non-negative");
        prop_assert!(pressure <= 1000, "Pressure must not exceed 1000 milli-units");
    });
}

#[test]
fn test_enforcement_bounded_by_legitimacy() {
    proptest!(|(
        requested_enforcement in 0i64..=2000,
        legitimacy in 0i64..=1000,
        baseline_rights in 0i64..=1000,
    )| {
        let e_max = compute_enforcement_max(legitimacy, baseline_rights);
        let actual = compute_clipped_enforcement(requested_enforcement, e_max);
        prop_assert!(actual <= e_max, "Enforcement must not exceed E_max");
        prop_assert!(actual >= 0, "Enforcement must be non-negative");
    });
}
```

### 15.4 Actor-Pair Ordering Determinism

```rust
#[test]
fn test_actor_pair_ordering_is_stable() {
    let pairs: Vec<ActorPairKey> = vec![
        ActorPairKey::new(5, 3),
        ActorPairKey::new(1, 9),
        ActorPairKey::new(2, 8),
        ActorPairKey::new(3, 5), // duplicate of (5,3); should be canonicalized
    ];
    let mut sorted = pairs.clone();
    sorted.sort();
    let sorted2 = { let mut p = pairs.clone(); p.sort(); p };
    assert_eq!(sorted, sorted2, "Sorting actor pairs must be deterministic");
    // Verify canonical ordering
    for pair in &sorted {
        assert!(pair.actor_a <= pair.actor_b, "actor_a must always <= actor_b");
    }
}
```

---

## 16. CIV Sim Integration Notes

### 16.1 Tick Phase Placement

| Phase | Diplomacy/Conflict Action |
|---|---|
| Phase 1: Command Intake | Accept player diplomatic commands (treaty offers, war declarations, sanction proposals) |
| Phase 2: Policy Phase | Evaluate policy bundle; compute `E_max`, `backfire_risk_milli`; validate casus belli |
| Phase 3: Deterministic Transition | Run FSM transitions (sorted by ActorPairKey); compute `L₀`, `C₀`; apply battle attrition; update leakage |
| Phase 4: Stochastic Events | Roll espionage/sabotage outcomes; detection events; coalition exit probability draws |
| Phase 5: Metrics Compute | Aggregate `war_burden`, `civilian_harm_index`, `shadow_share`, `backfire_risk` |
| Phase 6: Client Broadcast | Emit all events generated in phases 3–5 |

### 16.2 Coupling to CIV-0103 (Legitimacy)

The `legitimacy` field used in enforcement-legitimacy coupling (Section 7) is the same `Mood.legitimacy: i16` from the citizen lifecycle model (CIV-0103). The coupling formula:

```
legitimacy_delta_from_enforcement = -b4 * Φ(E, Sel)
```

feeds directly into the `Mood.legitimacy` component update in the social phase. The `enforcement.overreach.v1` event is consumed by the institution state machine (CIV-0103) to trigger `contested → captured` transitions.

### 16.3 Coupling to CIV-0104 (Coalition Constraint)

CIV-0104's "coalition-compatible external strategy" constraint maps to:
- `C0(t) < 1000` maintained by the coalition model (Section 6)
- `L0(t) < 1000` maintained by the sanction model (Section 4)
- Both are logged as metrics for the CIV-0104 scenario ablation suite

### 16.4 Resource Graph and Trade

The formal trade network capacity `c_e` and shadow trade network capacity `c̃_e` are defined over the same resource corridor graph defined in the economy crate. `TradeAvailability.formal_trade_milli` is derived by applying `DiplomaticState.trade_availability_milli()` to the base trade capacity. `TradeAvailability.shadow_trade_joules` is derived from the shadow flow model (Section 5).

### 16.5 Defense Spend Integration

`DiplomaticState.defense_spend_milli()` is the multiplier applied to the `defense_industry_investment_share` policy parameter in the economy crate's sector output model. During `ActiveConflict`, the mobilization model additionally diverts a configurable fraction of labor from productive sectors.

### 16.6 Event Log Integration

All six event types (Sections 10.1–10.6) are written to the existing event log via the `io` crate's `EventWriter` interface. Events include `state_hash` computed from the producing state (CIV-0001 invariant E3). All events are replay-verifiable.

### 16.7 Metrics Exported

The following metrics are exported to the `metrics` crate for dashboard consumption:

| Metric | Source | Description |
|---|---|---|
| `leakage_reproduction_number` | `SanctionPressure.leakage_reproduction_number` | L₀; threshold at 1000 |
| `coalition_stability_number` | `C0(t)` | C₀; threshold at 1000 |
| `backfire_risk_milli` | Enforcement coupling | Repression trap proximity |
| `war_burden_milli` | `ConflictState` | % output diverted to war |
| `civilian_harm_index` | `SiegeDamage` aggregate | Infrastructure damage stock |
| `shadow_share_milli` | Shadow flows | Shadow % of total flow |
| `enforcement_spiral_ticks` | `ConflictState` | Ticks in overreach spiral |
| `ssi_theater_minimum` | `BattleState` | Minimum SSI across active theaters |

---

## 17. Formal Theorem References

The following theorems from the research corpus are the mathematical foundations for this specification. Their threshold conditions must be computed and monitored every tick.

### T1: Sanctions Leakage Threshold Theorem (Section 4.3)

Sanctions remain effective iff `L₀(t) < 1.0` uniformly. When `L₀ > 1.0` for sustained periods, leakage grows toward `Λ_max` and sanctions effectiveness collapses.

**Plain meaning:** If scarcity and profit incentives outpace combined interdiction and honest enforcement, black markets will grow until they neutralize sanctions.

**Practical implication:** Governance integrity (`G`) and impartial enforcement (`1 - Sel`) are the primary dials. Raw enforcement intensity (`E`) alone is insufficient when `G*(1-Sel)` is low.

### T2: Authoritarian Enforcement Backfire Theorem (Section 7.3)

There exists an enforcement level `E*` such that for `E > E*`, enforcement reduces legitimacy enough to raise unrest and expand shadow network capacity, causing net leakage to increase over a finite horizon.

**Plain meaning:** You cannot brute-force away black markets under scarcity without risking collapse. You need integrity + legitimacy + services, not just enforcement.

**Practical implication:** The `backfire_risk_milli` meter (Section 17, Metrics) must be surfaced as a primary AI policy agent advisory output.

### T3: Coalition Sanctions Stability Theorem (Section 6)

If `C₀(t) < 1.0` uniformly, coalition holds. If `C₀(t) > 1.0` for sustained periods, exits cascade (positive feedback). Coalition collapse is the most common path to sanctions failure.

**Plain meaning:** Sanctions fail most often because coalitions fracture, not because interdiction is impossible.

**Practical implication:** Side-payments, governance integrity (resists disinformation), and humanitarian exceptions (reducing target scarcity, which reduces `ΔP` and fatigue) are the primary coalition-maintenance levers.

---

**Version History:**
- v1.0 (2026-02-21): Full expansion from 28-line stub to complete engineering-grade specification. Covers diplomatic FSM, conflict mechanics, sanction model with L₀, shadow network model with detection probability, coalition stability with C₀, enforcement-legitimacy coupling with backfire theorem, full Rust structs, module layout, JSON event schemas, SQL DDL, determinism constraints, conservation invariants, failure modes, and acceptance test suite.
- v1.1 (2026-02-21): Appended deep sections: Hidden Network Layer (full graph model, node/edge state, propagation algorithm), Espionage and Intelligence System (asset deployment, graded intel, counterintelligence), War Profiteering and Resource Extraction (war economy, arms trade, occupation ledger), Diplomacy Extended (formal treaty model, multilateral negotiations, international institutions), Long-Run Geopolitical Dynamics (hegemony cycle, alliance decay, ideological competition, phase diagram), Extended Event Taxonomy and DDL (8 new events, 5 new tables), Extended Test Suite (8 new stubs, shadow-state-takeover chaos scenario).

---

## 18. Hidden Network Layer — Deep Graph Model

### 18.1 Conceptual Foundation

Every polity operates two governance layers simultaneously:

1. **Formal State Layer** — public institutions, official policy, auditable records
2. **Shadow Network Layer** — covert influence graph of actors whose incentives may diverge from the formal state

The shadow network is not a single entity. It is a directed weighted graph whose nodes are power centers (not individuals) and whose edges carry resource flows, loyalty signals, and coercive obligations. The formal theorem for when shadow influence eclipses formal governance — the Shadow-State Capture Threshold Theorem — is the mathematical backbone of this section. Its capture reproduction number `R₀_capture` is monitored every tick alongside `L₀` and `C₀`.

### 18.2 Node State Vector

Each shadow network node represents a power cluster (intelligence faction, oligarch bloc, organized crime syndicate, foreign influence cell, military elite group, political machine, media conglomerate).

```rust
/// Six-dimensional ideology vector: (authority, market, equality, liberty, security, tradition).
/// Each axis: i32 in [-1000, 1000]. Negative = opposes axis, positive = supports.
pub type IdeologyVector = [i32; 6];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NodeType {
    IntelligenceFaction,
    MilitaryElite,
    CorporateOligarch,
    OrganizedCrime,
    MediaConglomerate,
    ForeignInfluenceCell,
    PoliticalMachine,
    IdeologicalCell,
}

/// State of a single shadow network node. All numeric fields are fixed-point i64.
/// Scaling: 0..1000 = 0.0..1.0 unless otherwise noted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkNode {
    pub node_id: u64,
    pub actor_id: ActorId,      // Which polity hosts this node
    pub node_type: NodeType,
    /// Ideology vector. Drives alliance formation and foreign policy alignment.
    pub ideology_vector: IdeologyVector,
    /// Aggregate influence over formal institutions (0..1000 milli-units).
    pub influence_score_milli: i64,
    /// Risk of being detected/exposed this tick (0..1000 milli-units).
    pub detection_risk_milli: i64,
    /// Controlled resources (joule-equivalent or influence units).
    pub resource_held: i64,
    /// Primary loyalty target (None = autonomous).
    pub loyalty_target: Option<u64>,  // node_id of patron node
    /// Legitimacy mask: how legitimate this node appears publicly (0..1000).
    pub legitimacy_mask_milli: i64,
    /// Capture intent: which formal institution is being targeted (institution_id or 0).
    pub capture_intent: u64,
    /// Exposure stock: cumulative detection events (0..1000; >700 triggers node_exposed).
    pub exposure_stock_milli: i64,
    /// Ticks since last exposure event.
    pub ticks_since_exposure: u32,
}
```

### 18.3 Edge Types and Mechanics

Edges are directed (`source_node_id → dest_node_id`) and typed. A node pair may carry multiple edge types simultaneously (e.g., a patronage relationship that also involves information sharing).

| Edge Type | Code | Resource Flow Direction | Mechanic |
|---|---|---|---|
| `Patronage` | 0 | Source → Dest (resources) | Source funds dest; dest votes with source on institutional decisions |
| `Bribery` | 1 | Source → Dest (resources) | Dest distorts a specific policy output in source's favor each tick |
| `Coercion` | 2 | Source → Dest (compliance) | Dest cannot defect for `coercion_duration_ticks`; costs both parties legitimacy |
| `Information` | 3 | Bidirectional (intelligence) | Both nodes share detection risk reductions and intelligence grades |
| `Blackmail` | 4 | Source → Dest (coercion escalation) | Dest loyalty_target set to source; detection of this edge exposes both |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub enum EdgeType {
    Patronage  = 0,
    Bribery    = 1,
    Coercion   = 2,
    Information = 3,
    Blackmail  = 4,
}

/// A single directed edge in the hidden influence network.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkEdge {
    pub edge_id: u64,
    pub source_node_id: u64,
    pub dest_node_id: u64,
    pub edge_type: EdgeType,
    /// Strength of the edge (0..1000 milli-units).
    pub strength_milli: i64,
    /// Resource flow per tick (joule-equivalent or influence units, non-negative).
    pub flow_per_tick: i64,
    /// Remaining coercion ticks (for Coercion/Blackmail edges; 0 = expired).
    pub coercion_duration_ticks: u32,
    /// Tick on which edge was created.
    pub created_tick: Tick,
    /// Whether this edge has been detected (triggers network.edge_created.v1 retroactively).
    pub detected: bool,
    /// Detection probability for this edge per tick (0..1000 milli-units).
    pub edge_detection_risk_milli: i64,
}
```

### 18.4 Full Hidden Network Container

```rust
use std::collections::BTreeMap;

/// The full hidden network graph for a single polity.
/// Stored in BTreeMap for determinism (sorted by node_id / edge_id).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HiddenNetwork {
    pub actor_id: ActorId,
    pub nodes: BTreeMap<u64, NetworkNode>,
    pub edges: BTreeMap<u64, NetworkEdge>,
    /// Spectral radius approximation of the adjacency matrix (fixed-point; 1000 = 1.0).
    /// Updated every SPECTRAL_UPDATE_INTERVAL ticks (default 10) using power iteration.
    pub spectral_radius_milli: i64,
    /// Aggregate shadow influence index (0..1000 milli-units).
    /// Derived from mean node influence_score weighted by resource_held.
    pub shadow_influence_index_milli: i64,
    /// Capture reproduction number R₀_capture (fixed-point; 1000 = threshold 1.0).
    pub capture_r0_milli: i64,
    /// Ticks R₀_capture has exceeded 1000 consecutively.
    pub r0_breach_ticks: u32,
}
```

### 18.5 Influence Propagation Algorithm

Influence propagates through the network each tick via a single-step diffusion pass (not iterative to convergence — determinism requires bounded computation). The algorithm runs in Phase 3 (Deterministic Transition).

```rust
/// Propagate influence one step through the hidden network.
/// All arithmetic is saturating i64. No floats.
pub fn propagate_influence(network: &mut HiddenNetwork, tick: Tick) {
    // Collect influence deltas before mutation (avoid order-dependence).
    let mut deltas: BTreeMap<u64, i64> = BTreeMap::new();

    for edge in network.edges.values() {
        if edge.detected && edge.edge_type == EdgeType::Bribery {
            // Detected bribery edges contribute negative legitimacy mask, not positive influence.
            continue;
        }
        let source = match network.nodes.get(&edge.source_node_id) {
            Some(n) => n,
            None => continue,
        };
        // Influence carried along edge = source_influence * edge_strength / 1000.
        let carried = source.influence_score_milli
            .saturating_mul(edge.strength_milli)
            / 1000;
        // Patronage and Information increase dest influence; Coercion and Blackmail do not.
        let delta = match edge.edge_type {
            EdgeType::Patronage | EdgeType::Information => carried / 10, // 10% per tick
            EdgeType::Bribery => carried / 20,                           //  5% per tick
            EdgeType::Coercion | EdgeType::Blackmail => 0,
        };
        *deltas.entry(edge.dest_node_id).or_insert(0) += delta;
    }

    // Apply deltas with saturation.
    for (node_id, delta) in &deltas {
        if let Some(node) = network.nodes.get_mut(node_id) {
            node.influence_score_milli =
                (node.influence_score_milli + delta).min(1000).max(0);
        }
    }

    // Update aggregate shadow influence index (resource-weighted mean).
    let total_resources: i64 = network.nodes.values()
        .map(|n| n.resource_held)
        .fold(0i64, |a, b| a.saturating_add(b));
    if total_resources > 0 {
        let weighted_influence: i64 = network.nodes.values()
            .map(|n| n.influence_score_milli.saturating_mul(n.resource_held))
            .fold(0i64, |a, b| a.saturating_add(b));
        network.shadow_influence_index_milli =
            (weighted_influence / total_resources).min(1000).max(0);
    }
}
```

### 18.6 Edge Creation and Destruction

Edges are created in Phase 4 (Stochastic) when shadow actors have sufficient resources and compatible ideology vectors. Edges are destroyed when coercion expires, when a node is exposed above the exposure threshold, or when the resource flow falls below a minimum.

```rust
/// Conditions under which a new edge may be created between two nodes.
pub fn can_create_edge(
    source: &NetworkNode,
    dest: &NetworkNode,
    edge_type: EdgeType,
    world_tick: Tick,
) -> bool {
    // Source must have resources to sustain the flow.
    if source.resource_held < MINIMUM_EDGE_RESOURCE_FLOOR {
        return false;
    }
    // Ideology alignment check: sum of absolute axis differences must be <= ALIGN_TOLERANCE.
    let ideology_distance: i32 = source.ideology_vector.iter()
        .zip(dest.ideology_vector.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();
    match edge_type {
        EdgeType::Patronage | EdgeType::Information => {
            ideology_distance <= PATRONAGE_ALIGN_TOLERANCE  // default 2000 (out of 6000 max)
        }
        EdgeType::Bribery => {
            // Bribery ignores ideology — purely transactional.
            true
        }
        EdgeType::Coercion | EdgeType::Blackmail => {
            // Coercion requires high source influence, ignores ideology.
            source.influence_score_milli >= COERCION_INFLUENCE_FLOOR  // default 400
        }
    }
}

/// Destroy an edge when any termination condition is met.
pub fn should_destroy_edge(edge: &NetworkEdge, source: &NetworkNode, dest: &NetworkNode) -> bool {
    // Expired coercion
    if (edge.edge_type == EdgeType::Coercion || edge.edge_type == EdgeType::Blackmail)
        && edge.coercion_duration_ticks == 0
    {
        return true;
    }
    // Source node exposed beyond recovery
    if source.exposure_stock_milli > EXPOSURE_DESTROY_THRESHOLD {  // default 850
        return true;
    }
    // Dest node captured (loyalty_target points to someone else with higher resource)
    if let Some(dest_loyalty) = dest.loyalty_target {
        if dest_loyalty != edge.source_node_id
            && dest.resource_held > LOYALTY_SWITCH_FLOOR  // default 5000
        {
            return true;
        }
    }
    false
}
```

### 18.7 Capture Reproduction Number (R₀_capture)

The capture reproduction number measures how self-sustaining shadow capture growth is. It is computed from the full node/edge state every tick and stored in `HiddenNetwork.capture_r0_milli`.

```rust
/// Compute the Shadow-State Capture Reproduction Number R₀_capture.
/// Result is fixed-point; 1000 = R₀ of 1.0 (threshold).
///
/// Formula:
///   R₀ = [α * ρ(A) * (R_base + ω * W_base) * O_base * (1 - G + κ * Sel_base)]
///       / [β * (1 - O_base) * G * (1 - Sel_base) + χ * Exposure_baseline]
///
/// All inputs 0..1000 milli-units. Parameters (defaults): α=12, ω=8, κ=5, β=15, χ=10.
pub fn compute_capture_r0(
    spectral_radius_milli: i64,     // ρ(A) * 1000
    rent_base_milli: i64,           // R_base
    war_opacity_milli: i64,         // W_base (emergency spending opacity)
    opacity_milli: i64,             // O_base (general transparency inverse)
    governance_integrity_milli: i64,// G
    selectivity_milli: i64,         // Sel_base
    exposure_baseline_milli: i64,   // χ * Exposure(0)
) -> i64 {
    const ALPHA: i64 = 12;
    const OMEGA: i64 = 8;
    const KAPPA: i64 = 5;
    const BETA: i64 = 15;

    let rent_plus_war = rent_base_milli
        .saturating_add(war_opacity_milli.saturating_mul(OMEGA) / 100);
    let integrity_term = (1000 - governance_integrity_milli)
        .saturating_add(selectivity_milli.saturating_mul(KAPPA) / 100)
        .min(2000);

    let numerator = ALPHA
        .saturating_mul(spectral_radius_milli) / 1000
        .saturating_mul(rent_plus_war) / 1000
        .saturating_mul(opacity_milli) / 1000
        .saturating_mul(integrity_term) / 1000;

    let denominator = {
        let transparency = 1000 - opacity_milli;
        let impartial = 1000 - selectivity_milli;
        BETA
            .saturating_mul(transparency) / 1000
            .saturating_mul(governance_integrity_milli) / 1000
            .saturating_mul(impartial) / 1000
            .saturating_add(exposure_baseline_milli)
    };

    if denominator <= 0 { return i64::MAX; }
    numerator.saturating_mul(1000) / denominator
}
```

**Invariant**: If `capture_r0_milli > 1000` for `R0_BREACH_TICKS` (default 3) consecutive ticks, a `shadow.capture_threshold_breached.v1` event is emitted and shadow node `resource_held` values increase by `CAPTURE_GROWTH_RATE_MILLI` (default 25 milli-units per tick) until the condition clears.

---

## 19. Espionage and Intelligence System

### 19.1 Intelligence Asset Deployment

Intelligence assets are discrete units assigned to collection missions. Asset deployment is decided in Phase 2 (Policy Phase); collection probability is resolved in Phase 4 (Stochastic).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub enum TargetType {
    /// Tactical: troop positions, supply routes, readiness levels.
    Military,
    /// Strategic: policy intentions, resource reserves, alliance commitments.
    Diplomatic,
    /// Structural: institution maps, hidden network topology, capture levels.
    Institutional,
    /// Economic: production output, trade volumes, sanctions leakage routes.
    Economic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub enum IntelligenceGrade {
    /// Tactical grade: improves `intelligence_modifier_milli` in battle resolution.
    Tactical,
    /// Strategic grade: improves diplomatic transition probabilities; reveals hidden
    /// treaty terms and sanction leakage routes.
    Strategic,
    /// Structural grade: partially reveals hidden network nodes/edges for target polity.
    Structural,
}

/// A single deployed intelligence asset.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IntelligenceAsset {
    pub asset_id: u64,
    pub run_id: RunId,
    pub owning_actor: ActorId,
    pub target_actor: ActorId,
    pub target_type: TargetType,
    pub grade: IntelligenceGrade,
    /// Base collection probability per tick (0..1000 milli-units).
    pub collection_probability_base_milli: i64,
    /// Current cover integrity (0..1000; 0 = blown).
    pub cover_integrity_milli: i64,
    /// Whether asset has been detected by target counterintelligence.
    pub blown: bool,
    /// Tick on which asset was deployed.
    pub deployed_tick: Tick,
    /// Tick on which cover was blown (0 if not blown).
    pub blown_tick: Tick,
    /// Ticks of remaining mission (0 = asset recalled).
    pub mission_ticks_remaining: u32,
}
```

### 19.2 Collection Probability by Target Type

Collection probability is a function of asset quality, target transparency, and active counterintelligence intensity. All values are fixed-point.

```
P_collect(asset, t) = σ(
    θ1 * asset.collection_probability_base_milli
    - θ2 * target.transparency_milli
    - θ3 * target.counterintelligence_intensity_milli
    + θ4 * asset.cover_integrity_milli
    - θ5 * target.governance_integrity_milli
)
```

Default parameters: `θ1 = 600, θ2 = 300, θ3 = 400, θ4 = 200, θ5 = 150`

**Collection probability by target type** (baseline adjustment to `collection_probability_base_milli`):

| Target Type | Base Milli | Primary Suppressor |
|---|---|---|
| Military | 550 | Counterintelligence intensity |
| Diplomatic | 450 | Governance integrity |
| Institutional | 300 | Transparency + shadow node detection |
| Economic | 600 | Trade opacity (inverse of trade openness) |

### 19.3 Intelligence Decay Rate

Collected intelligence becomes stale. Each grade decays at a different rate:

| Grade | Decay per Tick | Notes |
|---|---|---|
| Tactical | 150 milli-units | Battle positions change rapidly |
| Strategic | 60 milli-units | Policy intentions shift quarterly |
| Structural | 20 milli-units | Network topology is slow-moving |

Intelligence value is modeled as a stock `intel_value_milli` that decays multiplicatively:
```
intel_value_milli(t+1) = intel_value_milli(t) * (1000 - decay_rate) / 1000
```

When `intel_value_milli \< INTEL_STALE_THRESHOLD` (default 100), the intelligence record is marked stale and its modifiers cease.

### 19.4 Covert Operations

Covert operations are discrete actions that consume shadow network resources and produce targeted effects. They are resolved in Phase 4 using `ChaCha20Rng`.

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CovertOperationType {
    /// Sabotage: disrupts production or logistics in target polity.
    Sabotage {
        target_corridor_edge_id: u64,
        disruption_increase_milli: i64,   // added to corridor disruption_probability
        duration_ticks: u32,
    },
    /// Assassination: removes a specific shadow network node from the target polity.
    Assassination {
        target_node_id: u64,
        detection_risk_milli: i64,
    },
    /// Disinformation: injects ideology vector shift into target polity's population.
    Disinformation {
        target_actor: ActorId,
        ideology_axis: usize,             // 0..5 index into IdeologyVector
        shift_milli: i32,                 // signed; injected per tick for duration
        duration_ticks: u32,
    },
    /// ElectionInterference: increases disinformation pressure in coalition members;
    /// distorts sanction coalition stability C₀.
    ElectionInterference {
        target_actor: ActorId,
        c0_distortion_milli: i64,         // added to C₀ (raises instability)
        duration_ticks: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CovertOperation {
    pub operation_id: u64,
    pub run_id: RunId,
    pub initiating_actor: ActorId,
    pub operation_type: CovertOperationType,
    /// Resource cost (deducted from shadow network resource_held).
    pub resource_cost: i64,
    /// Base success probability (0..1000 milli-units).
    pub success_probability_milli: i64,
    /// Detection probability if operation is attempted (0..1000 milli-units).
    pub detection_probability_milli: i64,
    /// Tick on which operation was initiated.
    pub initiated_tick: Tick,
    /// Tick on which operation resolved.
    pub resolved_tick: Option<Tick>,
}

/// Result of resolving a covert operation in Phase 4.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CovertResult {
    pub operation_id: u64,
    pub succeeded: bool,
    pub detected: bool,
    /// Legitimacy damage applied to initiating actor if detected (0..500 milli-units).
    pub legitimacy_damage_if_detected_milli: i64,
    /// State hash at resolution tick.
    pub state_hash: String,
}
```

### 19.5 Counterintelligence Mechanics

Target polities run a counterintelligence (CI) system each tick. CI intensity determines the probability of detecting an active asset or covert operation.

```
P_detect_asset(asset, t) = σ(
    φ1 * target.counterintelligence_intensity_milli
    + φ2 * target.transparency_milli
    - φ3 * asset.cover_integrity_milli
    - φ4 * owning_actor.network_sophistication_milli
    + φ5 * asset.mission_ticks_elapsed   // longer missions accumulate risk
)
```

Default CI parameters: `φ1 = 400, φ2 = 200, φ3 = 350, φ4 = 300, φ5 = 10`

**Double-agent mechanics**: A detected asset is not immediately burned. With probability `DOUBLE_AGENT_PROBABILITY` (default 150 milli-units), the target CI converts the asset into a double agent. A double-agent asset:
- Continues to appear functional to the owning actor for `DOUBLE_AGENT_DURATION_TICKS` (default 5..15 from RNG)
- Feeds disinformation back to the owning actor (inverts collected intelligence grade effects)
- Triggers `espionage.double_agent_activated.v1` event (visible only to target)

**Blown cover consequences**:
1. Asset marked `blown = true`; `cover_integrity_milli` set to 0
2. Owning actor receives `espionage.asset_burned.v1` event (internal)
3. Target receives `espionage.operation_conducted.v1` event with `detected = true`
4. Legitimacy damage applied to owning actor's `legitimacy_milli` by `BLOWN_COVER_LEGITIMACY_DAMAGE` (default 30 milli-units per blown asset)
5. Shadow network `detection_risk_milli` of owning actor's nodes increases by `BLOW_DETECTION_CONTAGION` (default 50 milli-units) for 3 ticks

---

## 20. War Profiteering and Resource Extraction

### 20.1 War Economy Model

During `ActiveConflict`, resource flows are redirected toward military production. The war economy model tracks this diversion and the shadow network's extraction of a fraction of war-related transactions.

```rust
/// War economy state for a single actor during active conflict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WarEconomy {
    pub actor: ActorId,
    pub run_id: RunId,
    pub tick: Tick,
    /// Fraction of total economic output diverted to defense (0..1000 milli-units).
    pub mobilization_fraction_milli: i64,
    /// Total defense spend this tick (joule-equivalent units).
    pub defense_spend: i64,
    /// Production disruption due to labor diversion (0..1000 milli-units; reduces civilian output).
    pub civilian_output_suppression_milli: i64,
    /// War profiteering: fraction of defense_spend captured by shadow network (0..1000 milli-units).
    pub profiteering_fraction_milli: i64,
    /// Absolute amount extracted by shadow network this tick.
    pub shadow_extraction: i64,
    /// Occupation resource extraction from all occupied territories (joule-equivalent).
    pub occupation_extraction: i64,
}
```

Mobilization fraction evolves based on diplomatic state and policy:

```
mobilization_fraction_milli(t) = BASE_MOBILIZATION
    + (DiplomaticState.defense_spend_milli() - 1000) * MOBILIZATION_SENSITIVITY / 1000
    - max(0, ssi_theater_minimum_milli - SSI_FLOOR) * SUPPLY_DEMOBILIZATION
```

Civilian output suppression is linear in mobilization:
```
civilian_output_suppression_milli = mobilization_fraction_milli * SUPPRESSION_FACTOR / 1000
```
Default `SUPPRESSION_FACTOR = 600` (60% labor diversion causes ~60% output suppression in affected sectors).

### 20.2 War Profiteering Formula

Shadow networks extract a fraction `χ` of defense spending each tick. This is the direct hook referenced in Section 5.4:

```
shadow_extraction(t) = χ * defense_spend(t)
```

The profiteering fraction `χ` is not fixed — it grows with capture level:

```rust
/// Compute the war profiteering fraction (χ) given current shadow influence and capture.
/// Result: 0..1000 milli-units (fraction of defense_spend extracted by shadow network).
pub fn compute_profiteering_fraction(
    shadow_influence_index_milli: i64,
    capture_r0_milli: i64,
    governance_integrity_milli: i64,
) -> i64 {
    // Base profiteering driven by shadow influence.
    let base = shadow_influence_index_milli * 120 / 1000;  // up to 12% at full influence
    // Capture amplification: if R₀_capture > 1000, profiteering grows.
    let capture_amp = if capture_r0_milli > 1000 {
        (capture_r0_milli - 1000).min(1000) * 80 / 1000  // up to 8% additional
    } else {
        0
    };
    // Governance reduces profiteering.
    let governance_reduction = governance_integrity_milli * 100 / 1000;  // up to 10% reduction
    (base + capture_amp - governance_reduction).max(0).min(1000)
}
```

The extracted amount flows into the shadow network's `resource_held` for the nodes with `CaptureIntent` pointing at defense institutions:
```
R_shadow(t+1) += shadow_extraction(t)   // War profiteering hook (cf. Section 5.4)
```

### 20.3 Arms Trade and Sanctions Leakage via Materiel

Shadow networks facilitate arms transfers between polities, even under formal embargo. Arms transfers are a specialized `ShadowChannel::Material` flow with additional detection consequences.

```rust
/// A single arms transfer event through the shadow network.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArmsTransfer {
    pub transfer_id: u64,
    pub run_id: RunId,
    pub tick: Tick,
    pub source_actor: ActorId,
    pub dest_actor: ActorId,
    /// Materiel value (joule-equivalent; represents weapons system capacity).
    pub materiel_value: i64,
    /// Transfer channel: which shadow node facilitated this.
    pub facilitating_node_id: u64,
    /// Detection probability (0..1000 milli-units).
    pub detection_probability_milli: i64,
    /// Whether detected by sanctioning coalition this tick.
    pub detected_by_coalition: bool,
    /// Battle impact: increases dest `readiness_milli` by this amount for 3 ticks.
    pub readiness_boost_milli: i64,
    /// Sanctions leakage contribution: reduces effective sanction pressure by this milli-amount.
    pub sanction_leakage_contribution_milli: i64,
}
```

Arms transfer detection has stricter consequences than generic shadow channel detection:
- Detection probability multiplied by `ARMS_DETECTION_FACTOR` (default 1500 milli, i.e., 1.5× base)
- Detected arms transfer triggers `ColdWar → Sanctioned` transition for the source polity if coalition detects it
- Detected transfer adds `ARMS_DETECTION_COALITION_FATIGUE` (default 80 milli) to coalition member blowback

### 20.4 Occupation Economics

Occupied territories (cells in `ActiveConflict` controlled by attacker) contribute to the attacker's economy while incurring resistance cost and legitimacy drain.

```rust
/// Ledger entry for resource extraction from a single occupied territory cell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OccupationLedger {
    pub ledger_id: u64,
    pub run_id: RunId,
    pub tick: Tick,
    pub occupying_actor: ActorId,
    pub occupied_cell_id: u64,
    /// Gross resource extracted (joule-equivalent units per tick).
    pub gross_extraction: i64,
    /// Resistance cost: portion of extraction consumed by suppressing local resistance.
    pub resistance_cost: i64,
    /// Net extraction (gross - resistance_cost, non-negative).
    pub net_extraction: i64,
    /// Legitimacy drain applied to occupying actor this tick (0..1000 milli-units).
    pub legitimacy_drain_milli: i64,
    /// Occupation duration in ticks (longer occupation = higher resistance).
    pub occupation_duration_ticks: u32,
    /// Local population resistance intensity (0..1000 milli-units).
    pub resistance_intensity_milli: i64,
}

/// Compute net extraction from an occupied territory.
pub fn compute_occupation_net(
    gross_extraction: i64,
    occupation_duration_ticks: u32,
    initial_legitimacy_milli: i64,
    occupier_governance_milli: i64,
) -> OccupationLedger {
    // Resistance intensity grows with occupation duration (logistic approach to 1000).
    let duration_factor = (occupation_duration_ticks as i64).min(200) * 5; // 0..1000 over 200 ticks
    let resistance_milli = (500 + duration_factor / 2
        - initial_legitimacy_milli / 4
        - occupier_governance_milli / 4).clamp(0, 1000);

    // Resistance cost = gross * resistance_intensity / 1000.
    let resistance_cost = gross_extraction * resistance_milli / 1000;
    let net = (gross_extraction - resistance_cost).max(0);

    // Legitimacy drain on occupier: presence of occupation is globally visible.
    let legitimacy_drain = (resistance_milli / 10 + 20).min(80);  // 20..80 milli per tick

    OccupationLedger {
        ledger_id: 0, // assigned by engine
        run_id: 0,
        tick: 0,
        occupying_actor: 0,
        occupied_cell_id: 0,
        gross_extraction,
        resistance_cost,
        net_extraction: net,
        legitimacy_drain_milli: legitimacy_drain,
        occupation_duration_ticks,
        resistance_intensity_milli: resistance_milli,
    }
}
```

**Occupation conservation invariant**: `net_extraction >= 0` at all times (enforced by `max(0, ...)` guard). `legitimacy_drain_milli &isin; [20, 80]` (bounded by construction). Property test: `test_occupation_net_non_negative`.

---

## 21. Diplomacy Extended — Treaties and Multilateral Negotiations

### 21.1 Formal Treaty Model

Treaties are structured agreements with explicit terms, compliance monitoring, and breach detection. Each treaty occupies a **treaty slot** in the `DiplomaticRelation.treaty_slots` vector (maximum `MAX_TREATY_SLOTS` per pair, default 4).

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TreatyTermType {
    /// Resource sharing: actor_a transfers `amount_per_tick` joules to actor_b each tick.
    ResourceSharing { actor_a: ActorId, actor_b: ActorId, amount_per_tick: i64 },
    /// Non-aggression: both actors may not transition to Escalating/ActiveConflict.
    NonAggression { duration_ticks: u32 },
    /// Alliance obligation: actor_a must enter ActiveConflict if actor_b is attacked.
    AllianceObligation { obligated_actor: ActorId, protected_actor: ActorId },
    /// Trade terms: multiplier applied to actor_b's `formal_trade_milli` from actor_a.
    TradeTerm { from_actor: ActorId, to_actor: ActorId, multiplier_milli: i64 },
    /// Intelligence sharing: both actors share Tactical-grade intelligence automatically.
    IntelligenceSharing,
    /// Sanction participation: actor commits to joining coalition against a third party.
    SanctionParticipation { target_actor: ActorId, contribution_budget: i64 },
}

/// A single structured treaty between two polities.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Treaty {
    pub treaty_id: u64,
    pub run_id: RunId,
    pub pair: ActorPairKey,
    /// All terms of the treaty (sorted by term discriminant for determinism).
    pub terms: Vec<TreatyTermType>,
    /// Tick on which treaty was signed.
    pub signed_tick: Tick,
    /// Tick on which treaty expires (0 = indefinite).
    pub expiry_tick: Tick,
    /// Whether treaty is currently in force.
    pub in_force: bool,
    /// Ticks of compliance failure (by actor_a and actor_b respectively).
    pub compliance_failure_ticks_a: u32,
    pub compliance_failure_ticks_b: u32,
    /// Total breaches detected (>= BREACH_THRESHOLD triggers treaty.breached.v1).
    pub breach_count: u32,
}
```

### 21.2 Treaty Term Monitoring

Treaty compliance is checked every tick in Phase 3 (Deterministic Transition). Each term has an explicit compliance check.

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ComplianceCheck {
    pub treaty_id: u64,
    pub tick: Tick,
    pub term_index: u32,
    /// Whether actor_a is in compliance with this term.
    pub actor_a_compliant: bool,
    /// Whether actor_b is in compliance with this term.
    pub actor_b_compliant: bool,
    /// Compliance metric value (term-specific; 0 = full breach, 1000 = full compliance).
    pub compliance_score_milli: i64,
}

/// Check compliance for a single treaty term.
pub fn check_term_compliance(
    term: &TreatyTermType,
    state: &WorldState,
    pair: ActorPairKey,
    tick: Tick,
) -> ComplianceCheck {
    match term {
        TreatyTermType::ResourceSharing { actor_a, actor_b, amount_per_tick } => {
            let transferred = state.resource_transfer_this_tick(*actor_a, *actor_b);
            let score = (transferred * 1000 / amount_per_tick.max(1)).min(1000);
            ComplianceCheck {
                treaty_id: 0, tick, term_index: 0,
                actor_a_compliant: transferred >= *amount_per_tick,
                actor_b_compliant: true,
                compliance_score_milli: score,
            }
        }
        TreatyTermType::NonAggression { .. } => {
            let dip_state = state.diplomatic_state(pair.actor_a, pair.actor_b);
            let in_breach = dip_state == DiplomaticState::Escalating
                || dip_state == DiplomaticState::ActiveConflict;
            ComplianceCheck {
                treaty_id: 0, tick, term_index: 0,
                actor_a_compliant: !in_breach,
                actor_b_compliant: !in_breach,
                compliance_score_milli: if in_breach { 0 } else { 1000 },
            }
        }
        TreatyTermType::TradeTerm { from_actor, to_actor, multiplier_milli } => {
            let actual_milli = state.trade_availability_milli(*from_actor, *to_actor);
            let score = (actual_milli * 1000 / multiplier_milli.max(1)).min(1000);
            ComplianceCheck {
                treaty_id: 0, tick, term_index: 0,
                actor_a_compliant: actual_milli >= *multiplier_milli,
                actor_b_compliant: true,
                compliance_score_milli: score,
            }
        }
        _ => ComplianceCheck {
            treaty_id: 0, tick, term_index: 0,
            actor_a_compliant: true,
            actor_b_compliant: true,
            compliance_score_milli: 1000,
        }
    }
}
```

**Breach detection**: If any term's `compliance_score_milli \< BREACH_SCORE_THRESHOLD` (default 300) for `BREACH_GRACE_TICKS` (default 2) consecutive ticks, `breach_count` increments. When `breach_count >= TREATY_BREACH_THRESHOLD` (default 3), a `treaty.breached.v1` event is emitted and:
- `DiplomaticState` transitions toward `Strained` if currently `Cooperative` or `Alliance`
- `treaty_slots` entry for this treaty is freed after `BREACH_FALLOUT_TICKS` (default 5)

**Arbitration mechanics**: Either polity may invoke arbitration (influence capital cost: `ARBITRATION_COST`, default 500 influence units). Arbitration halts breach accumulation for `ARBITRATION_FREEZE_TICKS` (default 10) while a resolution is negotiated. Arbitration outcome is a Phase 4 stochastic event seeded from the policy bundle hash.

### 21.3 Multilateral Negotiations

Multilateral negotiations involve three or more polities and produce coalition treaties, trade frameworks, or collective security agreements.

```rust
/// A multilateral negotiation session (3+ polities).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MultilateralNegotiation {
    pub negotiation_id: u64,
    pub run_id: RunId,
    pub tick_started: Tick,
    /// Sorted list of participating actor IDs.
    pub participants: Vec<ActorId>,
    /// The issue under negotiation (determines payoff structure).
    pub issue: NegotiationIssue,
    /// Proposed terms (indexed by participant position).
    pub proposed_terms: Vec<TreatyTermType>,
    /// Per-participant acceptance score (0..1000; >= ACCEPT_THRESHOLD = binding commitment).
    pub acceptance_scores: BTreeMap<ActorId, i64>,
    /// Whether negotiation has concluded.
    pub concluded: bool,
    /// Outcome if concluded.
    pub outcome: Option<NegotiationOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NegotiationIssue {
    SanctionCoordination { target_actor: ActorId },
    TradeFramework,
    CollectiveSecurity,
    ResourceSharingPool,
    NuclearNonProliferation,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NegotiationOutcome {
    /// Minimum winning coalition formed: at least `mwc_size` participants agreed.
    MinimumWinningCoalition { mwc_size: usize, treaty_ids: Vec<u64> },
    /// Full consensus (all participants).
    Consensus { treaty_ids: Vec<u64> },
    /// Failure: insufficient acceptance.
    Failure { blocking_actors: Vec<ActorId> },
}
```

**Minimum Winning Coalition (MWC)**: For sanction coordination, MWC is defined as the smallest subset of participants whose combined interdiction budget exceeds `SANCTION_EFFECTIVE_THRESHOLD` (default 600 milli-units of target dependency coverage). For collective security, MWC requires a majority of participants by power score.

**Issue linkage**: A negotiation may link two issues (e.g., trade framework + collective security). Linked issues increase negotiation complexity (more terms) but also increase acceptance probability for participants whose interests span both issues.

### 21.4 International Institutions

Formal international institutions constrain bilateral diplomatic actions and provide legitimacy effects.

```
InstitutionConstraint(action, actor_a, actor_b) = {
    "allowed": action.compliance_with_institution_rules,
    "legitimacy_bonus": actor_a.institution_standing_milli * INSTITUTION_LEGIT_FACTOR / 1000,
    "coalition_contribution": action.coalition_alignment_score * INSTITUTION_COALITION_FACTOR / 1000
}
```

An actor's `institution_standing_milli` (0..1000) increases when:
- Treaties are signed and maintained in compliance (+5 milli per compliant tick)
- Multilateral negotiations are participated in (+20 milli per concluded negotiation)
- No breached treaties in the last `STANDING_CLEAN_TICKS` (default 20) ticks

An actor's `institution_standing_milli` decreases when:
- Treaties are breached (-50 milli per breach event)
- Unilateral war declarations without casus belli (-100 milli)
- Shadow covert operations are detected by coalition (+penalty to targeted polity; -30 milli to initiator)

**Legitimacy effect**: `legitimacy_bonus = institution_standing_milli * 8 / 1000` (up to +8 milli-units per tick legitimacy recovery from international goodwill).

---

## 22. Long-Run Geopolitical Dynamics

### 22.1 Hegemony Cycle Model

The simulation tracks a global power parity ratio `π_t = Power_C / Power_H` where `H` is the current hegemon (highest composite power actor) and `C` is the leading challenger.

```rust
/// Global hegemonic state computed each tick.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HegemonyState {
    pub run_id: RunId,
    pub tick: Tick,
    /// Current hegemon actor ID.
    pub hegemon_actor: ActorId,
    /// Challenger actor ID (second-highest composite power).
    pub challenger_actor: ActorId,
    /// Power parity ratio (fixed-point; 1000 = parity; < 1000 = hegemon dominant).
    pub parity_ratio_milli: i64,
    /// Transition Stress Index (fixed-point; 1000 = critical threshold).
    pub transition_stress_milli: i64,
    /// System war probability (fixed-point; 1000 = certainty).
    pub system_war_probability_milli: i64,
    /// Bloc fragmentation index (0..1000 milli-units; 1000 = fully fragmented).
    pub bloc_fragmentation_milli: i64,
    /// Whether a hegemonic transition is in progress.
    pub transition_in_progress: bool,
    /// Ticks parity_ratio has been above PARITY_CRITICAL_THRESHOLD (default 900).
    pub parity_breach_ticks: u32,
}

/// Composite power score for a single actor (fixed-point, no upper bound).
pub fn compute_composite_power(
    economic_output: i64,        // sector output sum (joule-equivalent)
    military_readiness_milli: i64,
    alliance_network_centrality_milli: i64,  // 0..1000
    institutional_cohesion_milli: i64,       // = (1000 - polarization - shadow_influence_index) / 2
    financial_fragility_milli: i64,          // 0..1000 (higher = more fragile)
) -> i64 {
    let material = economic_output;
    let military = material * military_readiness_milli / 1000;
    let network = military * (500 + alliance_network_centrality_milli / 2) / 1000;
    let cohesion_factor = (1000 - financial_fragility_milli / 2)
        .saturating_mul(institutional_cohesion_milli) / 1000;
    network * cohesion_factor / 1000
}
```

**Transition Stress Index**:
```
TS(t) = α1 * max(0, π_t - π0) / 1000
       + α2 * scarcity_world_milli / 1000
       + α3 * bloc_fragmentation_milli / 1000
       + α4 * financial_fragility_hegemon_milli / 1000
       + α5 * capture_r0_hegemon_milli / 1000
```

Default parameters: `α1 = 300, α2 = 200, α3 = 200, α4 = 150, α5 = 150`. Safe buffer `π0 = 700` (hegemon is safe until challenger reaches 70% of its power).

**System war probability** (fixed-point sigmoid):
```
system_war_probability_milli = σ(κ * TS_milli + λ * misperception_milli + μ * shadow_provocation_milli)
```
Parameters: `κ = 8, λ = 5, μ = 4`

**Hegemonic transition trigger**: When `parity_ratio_milli > PARITY_CRITICAL_THRESHOLD` (default 900) for `TRANSITION_TRIGGER_TICKS` (default 15) consecutive ticks AND `system_war_probability_milli > WAR_TRIGGER` (default 700), an `hegemony.transition_risk_updated.v1` event fires. The transition does not happen automatically — it depends on whether a war occurs or the challenger peacefully achieves dominant composite power.

### 22.2 Alliance Reliability Decay

Alliance commitments erode without active reinforcement. This is distinct from the coalition stability model (Section 6); it operates on longer time scales (decades of ticks).

```
alliance_reliability_milli(pair, t+1) = alliance_reliability_milli(pair, t)
    - δ_alliance * (1 - reinforcement_this_tick)
    + γ_alliance * side_payment_this_tick / SIDE_PAYMENT_SCALE
    + ideology_alignment_bonus(pair, t)
```

Where:
- `δ_alliance = 10` milli-units per tick decay when no reinforcement occurs
- `reinforcement_this_tick = 1` if any intelligence sharing, joint exercise event, or resource transfer occurred this tick
- `ideology_alignment_bonus = max(0, (1000 - ideology_distance(pair)) / 100)` (0..10 milli per tick)

When `alliance_reliability_milli \< ALLIANCE_DEFECT_THRESHOLD` (default 200) for `ALLIANCE_DEFECT_TICKS` (default 3) consecutive ticks, the alliance reliability penalty is applied to the `AllianceObligation` treaty term: the obligated actor may not respond even if the protected actor is attacked. This does not automatically trigger a treaty breach (an obligation unfulfilled during conflict does — see Section 21.2).

### 22.3 Ideological Competition and Foreign Policy Alignment

The ideology vectors carried by shadow network nodes (`NetworkNode.ideology_vector`) aggregate into a polity-level ideology distribution. This distribution influences foreign policy alignment in ways that bypass formal diplomatic states.

**Alignment score between two polities**:
```rust
/// Compute ideology alignment between two polities (0..1000; 1000 = perfect alignment).
/// Based on the mean ideology vectors of their respective shadow network nodes,
/// weighted by node influence_score.
pub fn compute_ideology_alignment(
    net_a: &HiddenNetwork,
    net_b: &HiddenNetwork,
) -> i64 {
    if net_a.nodes.is_empty() || net_b.nodes.is_empty() {
        return 500; // neutral alignment if no network data
    }

    let weighted_vec = |net: &HiddenNetwork| -> [i64; 6] {
        let total_influence: i64 = net.nodes.values()
            .map(|n| n.influence_score_milli)
            .fold(0i64, i64::saturating_add);
        if total_influence == 0 { return [0i64; 6]; }
        let mut out = [0i64; 6];
        for node in net.nodes.values() {
            for i in 0..6 {
                out[i] = out[i].saturating_add(
                    node.ideology_vector[i] as i64 * node.influence_score_milli / total_influence
                );
            }
        }
        out
    };

    let va = weighted_vec(net_a);
    let vb = weighted_vec(net_b);
    // L1 distance, normalized by max possible distance (6 axes × 2000 range each = 12000).
    let l1: i64 = va.iter().zip(vb.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();
    let distance_milli = (l1 * 1000 / 12000).min(1000);
    1000 - distance_milli
}
```

**Alignment effects on diplomatic transitions**:
- High ideology alignment (`> ALIGN_FLOOR`, default 700) is a prerequisite for `Cooperative → Alliance` transitions (cf. Section 2.2)
- Alignment below `ALIGN_CRISIS_FLOOR` (default 200) accelerates `Strained → Escalating` when combined with scarcity pressure
- Shadow network disinformation operations (Section 19.4 `ElectionInterference`) inject ideology vector shifts that temporarily reduce alignment

### 22.4 Long-Run Shadow Network Growth Model

Under sustained supercritical conditions (`capture_r0_milli > 1000`), shadow network aggregate `resource_held` grows according to:

```
R_shadow(t+1) = R_shadow(t)
    * (1 + capture_growth_rate_milli / 1000)
    + profiteering_extraction(t)
    + shadow_flow_total(t) * SHADOW_REINVESTMENT_RATE / 1000
    - ExposureLoss(t)
    - reform_seizure(t)
```

Where:
- `capture_growth_rate_milli = max(0, capture_r0_milli - 1000) * CAPTURE_GROWTH_SENSITIVITY / 1000`
- `CAPTURE_GROWTH_SENSITIVITY = 50` (default)
- `ExposureLoss(t) = exposure_events_this_tick * EXPOSURE_RESOURCE_PENALTY` (default 2000 joule-equivalent per event)
- `reform_seizure(t) = policy_reform_intensity * reform_seizure_rate` (from CIV-0103 governance reform actions)

**Shadow power eclipse condition**: Shadow network eclipses formal state power when:
```
shadow_influence_index_milli > ECLIPSE_THRESHOLD (default 850)
AND governance_integrity_milli < ECLIPSE_GOVERNANCE_FLOOR (default 150)
AND capture_r0_milli > ECLIPSE_R0_FLOOR (default 1500)
```

When all three conditions hold for `ECLIPSE_TICKS` (default 10) consecutive ticks, the polity enters the `ShadowStateTakeover` attractor (see Section 30: Chaos Scenario).

### 22.5 Phase Diagram: Legitimacy × Shadow Influence → Regime Classification

The following phase diagram maps the 2D space `(legitimacy_milli, shadow_influence_index_milli)` to regime types. Both axes are in `[0, 1000]`. Boundaries are deterministic thresholds checked each tick; regime is updated only on transition.

| Regime Type | Legitimacy Range | Shadow Influence Range | Description |
|---|---|---|---|
| `StableHybrid` | &gt; 600 | &lt; 300 | High-legitimacy, low-capture. Reform possible. |
| `WeakDemocracy` | 400..599 | &lt; 400 | Functional institutions; capture risk growing. |
| `OligarchicCapitalism` | 300..599 | 400..699 | Shadow networks dominant; policy distorted. |
| `MilitarizedSecurity` | 200..499 | 300..599 | Defense-driven enforcement; civil liberties eroded. |
| `CorruptBureaucracy` | 200..399 | 500..799 | High capture, moderate legitimacy collapse. |
| `ExternallyDestabilized` | &lt; 299 | &gt; 500 | Foreign influence cells dominant; sovereignty hollowed. |
| `ShadowStateTakeover` | &lt; 200 | &gt; 850 | Formal institutions captured; see Section 30. |
| `LegitimacyCollapse` | &lt; 100 | Any | Civil war threshold; couples to CIV-0103 revolt mechanics. |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub enum RegimeType {
    StableHybrid,
    WeakDemocracy,
    OligarchicCapitalism,
    MilitarizedSecurity,
    CorruptBureaucracy,
    ExternallyDestabilized,
    ShadowStateTakeover,
    LegitimacyCollapse,
}

pub fn classify_regime(
    legitimacy_milli: i64,
    shadow_influence_index_milli: i64,
) -> RegimeType {
    if legitimacy_milli <= 100 {
        return RegimeType::LegitimacyCollapse;
    }
    if shadow_influence_index_milli >= 850 && legitimacy_milli <= 200 {
        return RegimeType::ShadowStateTakeover;
    }
    if legitimacy_milli <= 299 && shadow_influence_index_milli >= 500 {
        return RegimeType::ExternallyDestabilized;
    }
    if legitimacy_milli <= 399 && shadow_influence_index_milli >= 500 {
        return RegimeType::CorruptBureaucracy;
    }
    if shadow_influence_index_milli >= 400 && shadow_influence_index_milli <= 699 {
        return RegimeType::OligarchicCapitalism;
    }
    if shadow_influence_index_milli >= 300 && shadow_influence_index_milli <= 599 {
        return RegimeType::MilitarizedSecurity;
    }
    if legitimacy_milli >= 400 && legitimacy_milli <= 599 {
        return RegimeType::WeakDemocracy;
    }
    RegimeType::StableHybrid
}
```

---

## 23. Extended Event Taxonomy and JSON Schemas

### 23.1 `espionage.operation_conducted.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "espionage.operation_conducted.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","initiating_actor","target_actor",
               "operation_type","succeeded","detected","detection_probability_milli",
               "resource_cost","legitimacy_damage_applied","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                  { "type": "string", "const": "espionage.operation_conducted.v1" },
    "version":                     { "type": "string", "const": "1" },
    "run_id":                      { "type": "integer", "minimum": 0 },
    "tick":                        { "type": "integer", "minimum": 0 },
    "initiating_actor":            { "type": "integer", "minimum": 0 },
    "target_actor":                { "type": "integer", "minimum": 0 },
    "operation_type":              { "type": "string",
                                     "enum": ["Sabotage","Assassination","Disinformation","ElectionInterference"] },
    "succeeded":                   { "type": "boolean" },
    "detected":                    { "type": "boolean" },
    "detection_probability_milli": { "type": "integer", "minimum": 0, "maximum": 1000 },
    "resource_cost":               { "type": "integer", "minimum": 0 },
    "legitimacy_damage_applied":   { "type": "integer", "minimum": 0 },
    "cover_blown_asset_id":        { "type": "integer", "minimum": 0 },
    "state_hash":                  { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.2 `network.edge_created.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "network.edge_created.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","actor_id","source_node_id","dest_node_id",
               "edge_type","strength_milli","flow_per_tick","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":        { "type": "string", "const": "network.edge_created.v1" },
    "version":           { "type": "string", "const": "1" },
    "run_id":            { "type": "integer", "minimum": 0 },
    "tick":              { "type": "integer", "minimum": 0 },
    "actor_id":          { "type": "integer", "minimum": 0 },
    "source_node_id":    { "type": "integer", "minimum": 0 },
    "dest_node_id":      { "type": "integer", "minimum": 0 },
    "edge_type":         { "type": "string",
                           "enum": ["Patronage","Bribery","Coercion","Information","Blackmail"] },
    "strength_milli":    { "type": "integer", "minimum": 0, "maximum": 1000 },
    "flow_per_tick":     { "type": "integer", "minimum": 0 },
    "coercion_ticks":    { "type": "integer", "minimum": 0 },
    "state_hash":        { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.3 `network.node_captured.v1`

Emitted when a shadow network node successfully captures a formal institution (its `InfluencePressure` exceeds `capture_threshold` for the targeted institution).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "network.node_captured.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","actor_id","node_id",
               "institution_id","influence_pressure_milli","capture_r0_milli","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                { "type": "string", "const": "network.node_captured.v1" },
    "version":                   { "type": "string", "const": "1" },
    "run_id":                    { "type": "integer", "minimum": 0 },
    "tick":                      { "type": "integer", "minimum": 0 },
    "actor_id":                  { "type": "integer", "minimum": 0 },
    "node_id":                   { "type": "integer", "minimum": 0 },
    "institution_id":            { "type": "integer", "minimum": 0 },
    "influence_pressure_milli":  { "type": "integer", "minimum": 0 },
    "capture_r0_milli":          { "type": "integer", "minimum": 0,
                                   "description": "R0_capture at capture tick; 1000 = threshold" },
    "policy_distortion_applied": { "type": "boolean" },
    "rent_leakage_increase":     { "type": "integer", "minimum": 0 },
    "state_hash":                { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.4 `treaty.signed.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "treaty.signed.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","treaty_id","actor_a","actor_b",
               "term_types","signed_tick","expiry_tick","influence_capital_transferred","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                    { "type": "string", "const": "treaty.signed.v1" },
    "version":                       { "type": "string", "const": "1" },
    "run_id":                        { "type": "integer", "minimum": 0 },
    "tick":                          { "type": "integer", "minimum": 0 },
    "treaty_id":                     { "type": "integer", "minimum": 0 },
    "actor_a":                       { "type": "integer", "minimum": 0 },
    "actor_b":                       { "type": "integer", "minimum": 0 },
    "term_types":                    { "type": "array",
                                       "items": { "type": "string",
                                         "enum": ["ResourceSharing","NonAggression","AllianceObligation",
                                                  "TradeTerm","IntelligenceSharing","SanctionParticipation"] }},
    "signed_tick":                   { "type": "integer", "minimum": 0 },
    "expiry_tick":                   { "type": "integer", "minimum": 0,
                                       "description": "0 = indefinite" },
    "influence_capital_transferred": { "type": "integer" },
    "multilateral_negotiation_id":   { "type": "integer", "minimum": 0 },
    "state_hash":                    { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.5 `treaty.breached.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "treaty.breached.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","treaty_id","actor_a","actor_b",
               "breaching_actor","breach_count","term_index","compliance_score_milli","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":              { "type": "string", "const": "treaty.breached.v1" },
    "version":                 { "type": "string", "const": "1" },
    "run_id":                  { "type": "integer", "minimum": 0 },
    "tick":                    { "type": "integer", "minimum": 0 },
    "treaty_id":               { "type": "integer", "minimum": 0 },
    "actor_a":                 { "type": "integer", "minimum": 0 },
    "actor_b":                 { "type": "integer", "minimum": 0 },
    "breaching_actor":         { "type": "integer", "minimum": 0 },
    "breach_count":            { "type": "integer", "minimum": 1 },
    "term_index":              { "type": "integer", "minimum": 0 },
    "compliance_score_milli":  { "type": "integer", "minimum": 0, "maximum": 1000 },
    "diplomatic_state_after":  { "type": "string",
                                 "enum": ["Cooperative","Strained","Sanctioned","Escalating",
                                          "ActiveConflict","Deescalating","ColdWar","Alliance"] },
    "state_hash":              { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.6 `war_economy.profiteering_recorded.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "war_economy.profiteering_recorded.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","actor","defense_spend",
               "profiteering_fraction_milli","shadow_extraction",
               "shadow_influence_index_milli","capture_r0_milli","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                    { "type": "string", "const": "war_economy.profiteering_recorded.v1" },
    "version":                       { "type": "string", "const": "1" },
    "run_id":                        { "type": "integer", "minimum": 0 },
    "tick":                          { "type": "integer", "minimum": 0 },
    "actor":                         { "type": "integer", "minimum": 0 },
    "defense_spend":                 { "type": "integer", "minimum": 0 },
    "profiteering_fraction_milli":   { "type": "integer", "minimum": 0, "maximum": 1000 },
    "shadow_extraction":             { "type": "integer", "minimum": 0 },
    "mobilization_fraction_milli":   { "type": "integer", "minimum": 0, "maximum": 1000 },
    "shadow_influence_index_milli":  { "type": "integer", "minimum": 0, "maximum": 1000 },
    "capture_r0_milli":              { "type": "integer", "minimum": 0,
                                       "description": "R0_capture at this tick; 1000 = threshold" },
    "state_hash":                    { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.7 `occupation.resource_extracted.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "occupation.resource_extracted.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","occupying_actor","occupied_cell_id",
               "gross_extraction","resistance_cost","net_extraction",
               "legitimacy_drain_milli","resistance_intensity_milli","occupation_duration_ticks",
               "state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                  { "type": "string", "const": "occupation.resource_extracted.v1" },
    "version":                     { "type": "string", "const": "1" },
    "run_id":                      { "type": "integer", "minimum": 0 },
    "tick":                        { "type": "integer", "minimum": 0 },
    "occupying_actor":             { "type": "integer", "minimum": 0 },
    "occupied_cell_id":            { "type": "integer", "minimum": 0 },
    "gross_extraction":            { "type": "integer", "minimum": 0 },
    "resistance_cost":             { "type": "integer", "minimum": 0 },
    "net_extraction":              { "type": "integer", "minimum": 0 },
    "legitimacy_drain_milli":      { "type": "integer", "minimum": 0, "maximum": 1000 },
    "resistance_intensity_milli":  { "type": "integer", "minimum": 0, "maximum": 1000 },
    "occupation_duration_ticks":   { "type": "integer", "minimum": 0 },
    "state_hash":                  { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

### 23.8 `hegemony.transition_risk_updated.v1`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "hegemony.transition_risk_updated.v1",
  "type": "object",
  "required": ["event_type","version","run_id","tick","hegemon_actor","challenger_actor",
               "parity_ratio_milli","transition_stress_milli",
               "system_war_probability_milli","bloc_fragmentation_milli",
               "parity_breach_ticks","state_hash"],
  "additionalProperties": false,
  "properties": {
    "event_type":                    { "type": "string", "const": "hegemony.transition_risk_updated.v1" },
    "version":                       { "type": "string", "const": "1" },
    "run_id":                        { "type": "integer", "minimum": 0 },
    "tick":                          { "type": "integer", "minimum": 0 },
    "hegemon_actor":                 { "type": "integer", "minimum": 0 },
    "challenger_actor":              { "type": "integer", "minimum": 0 },
    "parity_ratio_milli":            { "type": "integer", "minimum": 0,
                                       "description": "Fixed-point; 1000 = parity" },
    "transition_stress_milli":       { "type": "integer", "minimum": 0,
                                       "description": "Fixed-point; 1000 = critical threshold" },
    "system_war_probability_milli":  { "type": "integer", "minimum": 0, "maximum": 1000 },
    "bloc_fragmentation_milli":      { "type": "integer", "minimum": 0, "maximum": 1000 },
    "parity_breach_ticks":           { "type": "integer", "minimum": 0 },
    "transition_in_progress":        { "type": "boolean" },
    "c0_milli":                      { "type": "integer", "minimum": 0 },
    "l0_milli":                      { "type": "integer", "minimum": 0 },
    "capture_r0_milli":              { "type": "integer", "minimum": 0 },
    "state_hash":                    { "type": "string", "minLength": 64, "maxLength": 64 }
  }
}
```

---

## 24. Extended Database Schema (DDL)

```sql
-- =========================================================
-- CIV-0105 v1.1 Extended Tables
-- All tables are append-only (no UPDATE/DELETE in sim paths).
-- All tick-keyed. run_id partitions separate simulation runs.
-- =========================================================

-- Hidden network nodes (one row per node per tick snapshot, or on state change).
-- Stored on state change only for performance; full snapshot every SNAPSHOT_INTERVAL ticks.
CREATE TABLE hidden_network_nodes (
    id                         BIGSERIAL PRIMARY KEY,
    run_id                     BIGINT      NOT NULL,
    tick                       BIGINT      NOT NULL,
    actor_id                   BIGINT      NOT NULL,
    node_id                    BIGINT      NOT NULL,
    node_type                  SMALLINT    NOT NULL,   -- NodeType ordinal
    -- Ideology vector stored as 6 separate columns for query efficiency.
    ideology_authority         INTEGER     NOT NULL DEFAULT 0 CHECK (ideology_authority BETWEEN -1000 AND 1000),
    ideology_market            INTEGER     NOT NULL DEFAULT 0 CHECK (ideology_market BETWEEN -1000 AND 1000),
    ideology_equality          INTEGER     NOT NULL DEFAULT 0 CHECK (ideology_equality BETWEEN -1000 AND 1000),
    ideology_liberty           INTEGER     NOT NULL DEFAULT 0 CHECK (ideology_liberty BETWEEN -1000 AND 1000),
    ideology_security          INTEGER     NOT NULL DEFAULT 0 CHECK (ideology_security BETWEEN -1000 AND 1000),
    ideology_tradition         INTEGER     NOT NULL DEFAULT 0 CHECK (ideology_tradition BETWEEN -1000 AND 1000),
    influence_score_milli      INTEGER     NOT NULL CHECK (influence_score_milli BETWEEN 0 AND 1000),
    detection_risk_milli       INTEGER     NOT NULL CHECK (detection_risk_milli BETWEEN 0 AND 1000),
    resource_held              BIGINT      NOT NULL CHECK (resource_held >= 0),
    loyalty_target             BIGINT,                 -- NULL = autonomous
    legitimacy_mask_milli      INTEGER     NOT NULL CHECK (legitimacy_mask_milli BETWEEN 0 AND 1000),
    capture_intent             BIGINT      NOT NULL DEFAULT 0,
    exposure_stock_milli       INTEGER     NOT NULL CHECK (exposure_stock_milli BETWEEN 0 AND 1000),
    ticks_since_exposure       INTEGER     NOT NULL DEFAULT 0 CHECK (ticks_since_exposure >= 0),
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_hnn_run_tick       ON hidden_network_nodes (run_id, tick);
CREATE INDEX idx_hnn_actor_node     ON hidden_network_nodes (run_id, actor_id, node_id, tick DESC);
CREATE INDEX idx_hnn_high_influence ON hidden_network_nodes (run_id, influence_score_milli DESC)
    WHERE influence_score_milli >= 700;

-- --------------------------------------------------------

-- Hidden network edges (one row per edge per tick snapshot or on state change).
CREATE TABLE hidden_network_edges (
    id                         BIGSERIAL PRIMARY KEY,
    run_id                     BIGINT      NOT NULL,
    tick                       BIGINT      NOT NULL,
    actor_id                   BIGINT      NOT NULL,
    edge_id                    BIGINT      NOT NULL,
    source_node_id             BIGINT      NOT NULL,
    dest_node_id               BIGINT      NOT NULL,
    edge_type                  SMALLINT    NOT NULL,   -- EdgeType ordinal
    strength_milli             INTEGER     NOT NULL CHECK (strength_milli BETWEEN 0 AND 1000),
    flow_per_tick              BIGINT      NOT NULL CHECK (flow_per_tick >= 0),
    coercion_duration_ticks    INTEGER     NOT NULL DEFAULT 0 CHECK (coercion_duration_ticks >= 0),
    detected                   BOOLEAN     NOT NULL DEFAULT FALSE,
    edge_detection_risk_milli  INTEGER     NOT NULL CHECK (edge_detection_risk_milli BETWEEN 0 AND 1000),
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_hne_run_tick     ON hidden_network_edges (run_id, tick);
CREATE INDEX idx_hne_actor_edge   ON hidden_network_edges (run_id, actor_id, edge_id, tick DESC);
CREATE INDEX idx_hne_detected     ON hidden_network_edges (run_id, detected, tick DESC)
    WHERE detected = TRUE;
CREATE INDEX idx_hne_source_dest  ON hidden_network_edges (run_id, source_node_id, dest_node_id);

-- --------------------------------------------------------

-- Intelligence assets (one row per asset per tick, or on state change).
CREATE TABLE intelligence_assets (
    id                                BIGSERIAL PRIMARY KEY,
    run_id                            BIGINT      NOT NULL,
    tick                              BIGINT      NOT NULL,
    asset_id                          BIGINT      NOT NULL,
    owning_actor                      BIGINT      NOT NULL,
    target_actor                      BIGINT      NOT NULL,
    target_type                       SMALLINT    NOT NULL,   -- TargetType ordinal
    grade                             SMALLINT    NOT NULL,   -- IntelligenceGrade ordinal
    collection_probability_base_milli INTEGER     NOT NULL CHECK (collection_probability_base_milli BETWEEN 0 AND 1000),
    cover_integrity_milli             INTEGER     NOT NULL CHECK (cover_integrity_milli BETWEEN 0 AND 1000),
    blown                             BOOLEAN     NOT NULL DEFAULT FALSE,
    deployed_tick                     BIGINT      NOT NULL,
    blown_tick                        BIGINT      NOT NULL DEFAULT 0,
    mission_ticks_remaining           INTEGER     NOT NULL CHECK (mission_ticks_remaining >= 0),
    intel_value_milli                 INTEGER     NOT NULL DEFAULT 1000
                                                  CHECK (intel_value_milli BETWEEN 0 AND 1000),
    created_at                        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ia_run_tick      ON intelligence_assets (run_id, tick);
CREATE INDEX idx_ia_owning_actor  ON intelligence_assets (run_id, owning_actor, tick DESC);
CREATE INDEX idx_ia_blown         ON intelligence_assets (run_id, blown, tick DESC)
    WHERE blown = TRUE;
CREATE INDEX idx_ia_active        ON intelligence_assets (run_id, target_actor)
    WHERE blown = FALSE AND mission_ticks_remaining > 0;

-- --------------------------------------------------------

-- Treaties (one row per treaty per tick of state change; full snapshot on signing/breach).
CREATE TABLE treaties (
    id                           BIGSERIAL PRIMARY KEY,
    run_id                       BIGINT      NOT NULL,
    tick                         BIGINT      NOT NULL,
    treaty_id                    BIGINT      NOT NULL,
    actor_a                      BIGINT      NOT NULL,
    actor_b                      BIGINT      NOT NULL,
    -- Term types stored as sorted integer array for compact querying.
    term_types                   SMALLINT[]  NOT NULL,
    signed_tick                  BIGINT      NOT NULL,
    expiry_tick                  BIGINT      NOT NULL DEFAULT 0,  -- 0 = indefinite
    in_force                     BOOLEAN     NOT NULL DEFAULT TRUE,
    compliance_failure_ticks_a   INTEGER     NOT NULL DEFAULT 0 CHECK (compliance_failure_ticks_a >= 0),
    compliance_failure_ticks_b   INTEGER     NOT NULL DEFAULT 0 CHECK (compliance_failure_ticks_b >= 0),
    breach_count                 INTEGER     NOT NULL DEFAULT 0 CHECK (breach_count >= 0),
    influence_capital_at_signing BIGINT      NOT NULL DEFAULT 0,
    multilateral_negotiation_id  BIGINT,
    created_at                   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT treaty_actor_order CHECK (actor_a < actor_b)
);

CREATE INDEX idx_treaties_run_tick   ON treaties (run_id, tick);
CREATE INDEX idx_treaties_pair       ON treaties (run_id, actor_a, actor_b, tick DESC);
CREATE INDEX idx_treaties_in_force   ON treaties (run_id, in_force) WHERE in_force = TRUE;
CREATE INDEX idx_treaties_breached   ON treaties (run_id, breach_count) WHERE breach_count >= 3;

-- --------------------------------------------------------

-- Occupation ledger (one row per occupied cell per tick during occupation).
CREATE TABLE occupation_ledger (
    id                          BIGSERIAL PRIMARY KEY,
    run_id                      BIGINT      NOT NULL,
    tick                        BIGINT      NOT NULL,
    occupying_actor             BIGINT      NOT NULL,
    occupied_cell_id            BIGINT      NOT NULL,
    gross_extraction            BIGINT      NOT NULL CHECK (gross_extraction >= 0),
    resistance_cost             BIGINT      NOT NULL CHECK (resistance_cost >= 0),
    net_extraction              BIGINT      NOT NULL CHECK (net_extraction >= 0),
    legitimacy_drain_milli      INTEGER     NOT NULL CHECK (legitimacy_drain_milli BETWEEN 0 AND 1000),
    resistance_intensity_milli  INTEGER     NOT NULL CHECK (resistance_intensity_milli BETWEEN 0 AND 1000),
    occupation_duration_ticks   INTEGER     NOT NULL CHECK (occupation_duration_ticks >= 0),
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_occ_run_tick       ON occupation_ledger (run_id, tick);
CREATE INDEX idx_occ_actor          ON occupation_ledger (run_id, occupying_actor, tick DESC);
CREATE INDEX idx_occ_cell           ON occupation_ledger (run_id, occupied_cell_id, tick DESC);
CREATE INDEX idx_occ_high_resist    ON occupation_ledger (run_id, resistance_intensity_milli DESC)
    WHERE resistance_intensity_milli >= 700;
```

---

## 25. Extended Rust Module Layout

The following modules are added to the existing crate layout (Section 9):

```
crates/
├── diplomacy/
│   └── src/
│       ├── treaties.rs             -- Treaty, TreatyTermType, ComplianceCheck
│       ├── multilateral.rs         -- MultilateralNegotiation, NegotiationOutcome, MWC
│       ├── hegemony.rs             -- HegemonyState, compute_composite_power, parity cycle
│       └── tests/
│           ├── treaty_compliance.rs
│           ├── multilateral_stability.rs
│           └── hegemony_trajectory.rs
│
├── conflict/
│   └── src/
│       ├── war_economy.rs          -- WarEconomy, compute_profiteering_fraction
│       ├── arms_trade.rs           -- ArmsTransfer, detection consequences
│       ├── occupation.rs           -- OccupationLedger, compute_occupation_net
│       └── tests/
│           ├── war_profiteering.rs
│           ├── occupation_conservation.rs
│           └── arms_detection.rs
│
└── shadow/                         -- new crate
    └── src/
        ├── lib.rs                  -- re-exports; module declarations
        ├── node.rs                 -- NetworkNode, NodeType, IdeologyVector
        ├── edge.rs                 -- NetworkEdge, EdgeType, can_create_edge, should_destroy_edge
        ├── network.rs              -- HiddenNetwork, propagate_influence, spectral_radius
        ├── capture.rs              -- compute_capture_r0, shadow_power_eclipse_check
        ├── regime.rs               -- RegimeType, classify_regime, phase diagram
        ├── espionage.rs            -- IntelligenceAsset, CovertOperation, CovertResult
        ├── counterintelligence.rs  -- CI probability, double_agent_mechanics, blown_cover_consequences
        ├── ideology_alignment.rs   -- compute_ideology_alignment, alignment effects
        └── tests/
            ├── determinism.rs
            ├── influence_propagation.rs
            ├── espionage_bounds.rs
            ├── capture_r0.rs
            ├── regime_classification.rs
            └── shadow_takeover.rs
```

Add `crates/shadow` to workspace:
```toml
[workspace]
members = [
  "crates/engine",
  "crates/policy",
  "crates/metrics",
  "crates/io",
  "crates/server",
  "crates/diplomacy",
  "crates/conflict",
  "crates/shadow",      # new
]
```

---

## 26. Extended Metrics Exported

| Metric | Source | Description |
|---|---|---|
| `shadow_influence_index_milli` | `HiddenNetwork.shadow_influence_index_milli` | Aggregate shadow influence per polity |
| `capture_r0_milli` | `HiddenNetwork.capture_r0_milli` | R₀_capture; threshold at 1000 |
| `r0_breach_ticks` | `HiddenNetwork.r0_breach_ticks` | Ticks of supercritical capture growth |
| `regime_type` | `classify_regime()` | Current regime classification enum |
| `ideology_alignment_hegemon_challenger` | `compute_ideology_alignment()` | Alignment 0..1000 |
| `parity_ratio_milli` | `HegemonyState.parity_ratio_milli` | Power parity; 1000 = parity |
| `transition_stress_milli` | `HegemonyState.transition_stress_milli` | Geopolitical fragility |
| `system_war_probability_milli` | `HegemonyState.system_war_probability_milli` | System war risk |
| `total_war_profiteering` | `WarEconomy.shadow_extraction` sum | Total shadow extraction from defense |
| `occupation_legitimacy_drain_total` | `OccupationLedger.legitimacy_drain_milli` sum | Occupation legitimacy cost |
| `active_intelligence_assets` | `intelligence_assets` WHERE blown=FALSE | Deployed spy count |
| `treaty_compliance_mean` | `ComplianceCheck.compliance_score_milli` mean | Portfolio compliance health |
| `alliance_reliability_minimum` | min over all alliances | Weakest alliance link |

---

## 27. Extended Conservation Invariants

### 27.1 Network Influence Bounded

**Invariant**: `NetworkNode.influence_score_milli &isin; [0, 1000]` at all times.

Enforced by:
1. `propagate_influence()` applies `.min(1000).max(0)` after each delta
2. Initial node creation clamps to `[0, 1000]`
3. Property test: `test_influence_bounded`

### 27.2 Shadow Extraction Non-Negative and Bounded

**Invariant**: `shadow_extraction >= 0` AND `shadow_extraction <= defense_spend`.

Enforced by:
1. `profiteering_fraction_milli &isin; [0, 1000]` (bounded by `compute_profiteering_fraction`)
2. `shadow_extraction = defense_spend * profiteering_fraction_milli / 1000`
3. Integer truncation toward zero (defense_spend non-negative by invariant)
4. Property test: `test_war_profiteering_conservation`

### 27.3 Occupation Net Non-Negative

**Invariant**: `net_extraction >= 0` (resistance_cost cannot exceed gross_extraction).

Enforced by:
1. `net = (gross_extraction - resistance_cost).max(0)` in `compute_occupation_net`
2. `resistance_milli &isin; [0, 1000]` (clamped)
3. `resistance_cost = gross_extraction * resistance_milli / 1000 <= gross_extraction`
4. Property test: `test_occupation_net_non_negative`

### 27.4 Treaty Compliance Score Bounded

**Invariant**: `compliance_score_milli &isin; [0, 1000]` for all term compliance checks.

Enforced by:
1. All `check_term_compliance` implementations return `.min(1000)`
2. Division protected by `.max(1)` on denominator
3. Property test: `test_treaty_compliance_bounded`

### 27.5 Parity Ratio Non-Negative

**Invariant**: `parity_ratio_milli >= 0`.

Enforced by:
1. Composite power computation uses saturating arithmetic; power >= 0 by construction
2. Ratio computed as `challenger_power * 1000 / hegemon_power.max(1)`
3. Property test: `test_parity_non_negative`

---

## 28. Extended Determinism Constraints

### 28.1 Hidden Network Evaluation Order

All hidden network updates (edge creation/destruction, influence propagation, capture computation) are evaluated in sorted order by `actor_id`, then by `node_id` (for nodes) or `edge_id` (for edges). `BTreeMap` guarantees this without additional sort calls.

### 28.2 Intelligence Asset Resolution Order

Intelligence asset collection and CI detection are resolved in Phase 4. Assets are sorted by `(target_actor, asset_id)` before resolution. RNG is seeded per-asset from:
```rust
let asset_seed = policy_bundle_hash ^ (tick << 32) ^ asset_id ^ (target_actor << 16);
let mut rng = ChaCha20Rng::seed_from_u64(asset_seed);
```

### 28.3 Treaty Compliance Evaluation Order

Treaties are evaluated in sorted order by `treaty_id` within each tick. `treaty_id` is assigned monotonically at signing (guaranteed by the engine's ID allocator, which uses a `u64` counter starting at tick 0, monotonically incrementing).

### 28.4 Covert Operation Resolution Order

Covert operations are resolved in Phase 4 in `(initiating_actor, operation_id)` sorted order. Each operation consumes its own RNG stream seeded from:
```rust
let op_seed = policy_bundle_hash ^ (tick << 32) ^ operation_id;
let mut rng = ChaCha20Rng::seed_from_u64(op_seed);
```

---

## 29. Extended Test Suite

### 29.1 Hidden Network Influence Propagation Determinism

```rust
#[test]
fn test_influence_propagation_determinism() {
    let net0 = ShadowTestFixture::initial_hidden_network();
    let tick = 42u64;

    let mut net1 = net0.clone();
    propagate_influence(&mut net1, tick);

    let mut net2 = net0.clone();
    propagate_influence(&mut net2, tick);

    assert_eq!(net1, net2,
        "Influence propagation must be deterministic given same initial state and tick");
}
```

### 29.2 Espionage Detection Probability Bounds

```rust
#[test]
fn test_espionage_detection_probability_bounds() {
    proptest!(|(
        ci_intensity in 0i64..=1000,
        transparency in 0i64..=1000,
        cover_integrity in 0i64..=1000,
        sophistication in 0i64..=1000,
        mission_ticks in 0u32..=200,
    )| {
        let p_detect = compute_ci_detection_probability(
            ci_intensity, transparency, cover_integrity, sophistication, mission_ticks
        );
        prop_assert!(p_detect >= 0, "Detection probability must be non-negative");
        prop_assert!(p_detect <= 1000, "Detection probability must not exceed 1000 milli-units");
    });
}
```

### 29.3 War Profiteering Conservation

```rust
#[test]
fn test_war_profiteering_conservation() {
    proptest!(|(
        defense_spend in 0i64..=100_000_000,
        shadow_influence in 0i64..=1000,
        capture_r0 in 0i64..=3000,
        governance in 0i64..=1000,
    )| {
        let fraction = compute_profiteering_fraction(shadow_influence, capture_r0, governance);
        prop_assert!(fraction >= 0, "Profiteering fraction must be non-negative");
        prop_assert!(fraction <= 1000, "Profiteering fraction must not exceed 1000 milli-units");
        let extraction = defense_spend * fraction / 1000;
        prop_assert!(extraction >= 0, "Shadow extraction must be non-negative");
        prop_assert!(extraction <= defense_spend,
            "Shadow extraction must not exceed defense spend");
    });
}
```

### 29.4 Treaty Compliance Monitoring Bounds

```rust
#[test]
fn test_treaty_compliance_score_bounded() {
    proptest!(|(
        actual_transfer in 0i64..=10_000_000,
        required_transfer in 1i64..=10_000_000,
    )| {
        // Simulate ResourceSharing compliance check.
        let score = (actual_transfer * 1000 / required_transfer).min(1000);
        prop_assert!(score >= 0, "Compliance score must be non-negative");
        prop_assert!(score <= 1000, "Compliance score must not exceed 1000 milli-units");
    });
}

#[test]
fn test_treaty_breach_monotone() {
    // Breach count must only increase, never decrease, until treaty is terminated.
    let mut treaty = TreatyTestFixture::active_resource_sharing_treaty();
    let mut prev_breach_count = treaty.breach_count;
    for tick in 1u64..=30 {
        apply_compliance_failure(&mut treaty, tick);
        assert!(treaty.breach_count >= prev_breach_count,
            "Breach count must be monotonically non-decreasing");
        prev_breach_count = treaty.breach_count;
    }
}
```

### 29.5 Multilateral Coalition Stability

```rust
#[test]
fn test_multilateral_mwc_stability() {
    // A minimum winning coalition must not be empty and must have at least 2 members.
    let negotiation = MultilateralTestFixture::sanction_coordination_session(5);
    let outcome = resolve_multilateral_negotiation(&negotiation);
    match outcome {
        NegotiationOutcome::MinimumWinningCoalition { mwc_size, .. } => {
            assert!(mwc_size >= 2, "MWC must have at least 2 members");
            assert!(mwc_size <= negotiation.participants.len(),
                "MWC must not exceed total participants");
        }
        NegotiationOutcome::Consensus { .. } => {
            // Full consensus is acceptable — all participants agreed.
        }
        NegotiationOutcome::Failure { ref blocking_actors } => {
            assert!(!blocking_actors.is_empty(),
                "Failure must identify at least one blocking actor");
        }
    }
}
```

### 29.6 Occupation Legitimacy Drain

```rust
#[test]
fn test_occupation_legitimacy_drain_bounded() {
    proptest!(|(
        gross_extraction in 0i64..=1_000_000,
        duration_ticks in 0u32..=500,
        initial_legitimacy in 0i64..=1000,
        governance in 0i64..=1000,
    )| {
        let ledger = compute_occupation_net(
            gross_extraction, duration_ticks, initial_legitimacy, governance
        );
        prop_assert!(ledger.net_extraction >= 0, "Net extraction must be non-negative");
        prop_assert!(ledger.net_extraction <= gross_extraction,
            "Net extraction must not exceed gross extraction");
        prop_assert!(ledger.legitimacy_drain_milli >= 20,
            "Legitimacy drain must be at least 20 milli-units");
        prop_assert!(ledger.legitimacy_drain_milli <= 80,
            "Legitimacy drain must not exceed 80 milli-units");
        prop_assert!(ledger.resistance_intensity_milli >= 0,
            "Resistance must be non-negative");
        prop_assert!(ledger.resistance_intensity_milli <= 1000,
            "Resistance must not exceed 1000 milli-units");
    });
}
```

### 29.7 Hegemony Cycle Trajectory

```rust
#[test]
fn test_hegemony_parity_trajectory_monotone_when_challenger_grows() {
    // If challenger power grows each tick and hegemon is static, parity_ratio must increase.
    let mut hegemon_power = 1_000_000i64;
    let mut challenger_power = 500_000i64;
    let mut prev_parity = challenger_power * 1000 / hegemon_power;

    for _ in 0..50 {
        challenger_power = challenger_power.saturating_add(10_000);
        let parity = challenger_power * 1000 / hegemon_power;
        assert!(parity >= prev_parity,
            "Parity ratio must be non-decreasing when challenger grows and hegemon is static");
        prev_parity = parity;
    }
}

#[test]
fn test_transition_stress_bounded_below() {
    proptest!(|(
        parity_milli in 0i64..=2000,
        scarcity_milli in 0i64..=1000,
        fragmentation_milli in 0i64..=1000,
        financial_fragility_milli in 0i64..=1000,
        capture_r0_milli in 0i64..=3000,
    )| {
        let ts = compute_transition_stress(
            parity_milli, scarcity_milli, fragmentation_milli,
            financial_fragility_milli, capture_r0_milli
        );
        prop_assert!(ts >= 0, "Transition stress must be non-negative");
    });
}
```

### 29.8 Shadow Power Eclipse Conditions

```rust
#[test]
fn test_shadow_eclipse_requires_all_conditions() {
    // Eclipse must NOT trigger if any one condition is not met.
    let full_eclipse = (shadow_influence: 900, governance: 100, capture_r0: 1600);
    assert!(shadow_eclipse_check(900, 100, 1600));

    // Missing governance condition.
    assert!(!shadow_eclipse_check(900, 200, 1600),
        "Eclipse must not trigger if governance integrity is above floor");

    // Missing shadow influence condition.
    assert!(!shadow_eclipse_check(800, 100, 1600),
        "Eclipse must not trigger if shadow influence below threshold");

    // Missing capture R0 condition.
    assert!(!shadow_eclipse_check(900, 100, 1400),
        "Eclipse must not trigger if R0_capture below floor");
}
```

---

## 30. Chaos Scenario: Shadow State Takeover

### 30.1 Scenario Description

The **Shadow State Takeover** scenario is a fully-specified chaos test that validates the simulation correctly models the attractor state in which formal institutions are hollowed out and the hidden network achieves de facto governance dominance. This scenario exercises all six major subsystems added in v1.1: the hidden network layer, espionage system, war profiteering, occupation ledger, hegemony dynamics, and regime phase diagram.

The scenario targets a single polity (`actor_id = 1`) running for 200 ticks.

### 30.2 Initial Conditions

```rust
pub fn shadow_takeover_scenario_initial_state() -> WorldState {
    WorldState {
        tick: 0,
        // Diplomatic relations: single polity, no external conflict.
        diplomatic_relations: BTreeMap::new(),
        // Shadow network: pre-seeded with 4 nodes at moderate influence.
        hidden_network: HiddenNetwork {
            actor_id: 1,
            nodes: btreemap! {
                1 => NetworkNode {
                    node_id: 1, actor_id: 1,
                    node_type: NodeType::CorporateOligarch,
                    ideology_vector: [200, 400, -300, -100, 100, 50],
                    influence_score_milli: 400,
                    detection_risk_milli: 150,
                    resource_held: 500_000,
                    loyalty_target: None,
                    legitimacy_mask_milli: 700,
                    capture_intent: 1,  // targets procurement institution
                    exposure_stock_milli: 0,
                    ticks_since_exposure: 0,
                },
                2 => NetworkNode {
                    node_id: 2, actor_id: 1,
                    node_type: NodeType::IntelligenceFaction,
                    ideology_vector: [500, 0, -200, -400, 600, 200],
                    influence_score_milli: 350,
                    detection_risk_milli: 200,
                    resource_held: 300_000,
                    loyalty_target: Some(1),  // patronage from oligarch node
                    legitimacy_mask_milli: 500,
                    capture_intent: 2,  // targets judiciary
                    exposure_stock_milli: 0,
                    ticks_since_exposure: 0,
                },
                // ... two more nodes omitted for brevity, defined in test fixture
            },
            edges: btreemap! {
                1 => NetworkEdge {
                    edge_id: 1, source_node_id: 1, dest_node_id: 2,
                    edge_type: EdgeType::Patronage,
                    strength_milli: 600, flow_per_tick: 50_000,
                    coercion_duration_ticks: 0,
                    created_tick: 0, detected: false,
                    edge_detection_risk_milli: 80,
                },
            },
            spectral_radius_milli: 800,
            shadow_influence_index_milli: 375,
            capture_r0_milli: 1100,   // already above threshold at t=0
            r0_breach_ticks: 1,
        },
        // Macro conditions: moderate governance, moderate scarcity, active conflict ongoing.
        governance_integrity_milli: 350,
        legitimacy_milli: 480,
        scarcity_milli: 550,
        selectivity_milli: 400,
        opacity_milli: 600,
        war_opacity_milli: 700,   // ongoing conflict inflates war opacity
        rent_base_milli: 500,
        // No formal treaty in force.
        treaty_slots: vec![],
    }
}
```

### 30.3 Expected Trajectory

| Tick Range | Expected Behavior |
|---|---|
| 0..20 | `capture_r0_milli` remains > 1000; shadow nodes accumulate war profiteering; no eclipse yet |
| 20..60 | Two additional edges created (Bribery + Coercion); institution capture events fire for procurement and judiciary; policy distortion visible in rent leakage increase |
| 60..100 | `shadow_influence_index_milli` crosses 700; `governance_integrity_milli` falls below 250 via capture drain; `legitimacy_milli` falls to 300..400 range |
| 100..150 | First `shadow.capture_threshold_breached.v1` event cascade; regime classified as `CorruptBureaucracy` then `OligarchicCapitalism` |
| 150..180 | `shadow_influence_index_milli` crosses 850; `legitimacy_milli` falls below 200; `shadow_eclipse_check` returns true; `ShadowStateTakeover` regime classified |
| 180..200 | Formal institutions hollow: enforcement bias > 600, rent leakage > 400, `capture_r0_milli` > 2000; scenario ends with `ShadowStateTakeover` stable |

### 30.4 Test Assertions

```rust
#[test]
fn test_shadow_state_takeover_scenario() {
    let mut state = shadow_takeover_scenario_initial_state();
    let policy = ShadowTakeoverScenario::wartime_no_reform_policy();
    let seed = 77u64;

    let mut eclipse_reached_tick: Option<u64> = None;
    let mut regime_history: Vec<(Tick, RegimeType)> = Vec::new();

    for tick in 0..200u64 {
        let (snapshot, next_state) = full_tick(&state, &policy, seed ^ tick);
        state = next_state;

        let regime = classify_regime(
            state.legitimacy_milli,
            state.hidden_network.shadow_influence_index_milli,
        );
        regime_history.push((tick, regime));

        // Eclipse check.
        if shadow_eclipse_check(
            state.hidden_network.shadow_influence_index_milli,
            state.governance_integrity_milli,
            state.hidden_network.capture_r0_milli,
        ) && eclipse_reached_tick.is_none() {
            eclipse_reached_tick = Some(tick);
        }
    }

    // Assert eclipse is reached within the expected window.
    let eclipse_tick = eclipse_reached_tick.expect("Eclipse must be reached within 200 ticks");
    assert!(eclipse_tick >= 150, "Eclipse must not happen too early (before tick 150)");
    assert!(eclipse_tick <= 200, "Eclipse must occur within simulation window");

    // Assert final regime.
    let final_regime = regime_history.last().map(|(_, r)| *r).unwrap();
    assert_eq!(final_regime, RegimeType::ShadowStateTakeover,
        "Final regime must be ShadowStateTakeover under wartime no-reform conditions");

    // Assert capture_r0 monotone increase (no reform = no decay).
    let r0_values: Vec<i64> = snapshot_history.iter()
        .map(|s| s.hidden_network.capture_r0_milli)
        .collect();
    for window in r0_values.windows(2) {
        assert!(window[1] >= window[0] - 50,  // allow small tick-to-tick noise
            "R0_capture must trend non-decreasing without reform intervention");
    }

    // Assert no infinite resources (conservation check).
    let final_resource_total: i64 = state.hidden_network.nodes.values()
        .map(|n| n.resource_held)
        .fold(0i64, i64::saturating_add);
    assert!(final_resource_total > 0, "Shadow network must have non-zero resources at eclipse");
    assert!(final_resource_total < i64::MAX / 2,
        "Shadow network resources must not overflow i64::MAX/2");
}

#[test]
fn test_shadow_state_takeover_determinism() {
    // Same seed must produce identical trajectory.
    let state0 = shadow_takeover_scenario_initial_state();
    let policy = ShadowTakeoverScenario::wartime_no_reform_policy();
    let seed = 77u64;

    let trajectory1: Vec<WorldState> = run_scenario(&state0, &policy, seed, 200);
    let trajectory2: Vec<WorldState> = run_scenario(&state0, &policy, seed, 200);

    for (tick, (s1, s2)) in trajectory1.iter().zip(trajectory2.iter()).enumerate() {
        assert_eq!(s1, s2,
            "Scenario trajectory must be identical at tick {} given same seed", tick);
    }
}
```

### 30.5 Anti-Takeover Counterfactual

To validate that the scenario is not a fixed attractor regardless of policy, the test suite includes a counterfactual run with `reform_policy` (governance integrity restoration, transparency increase, anti-rent enforcement):

```rust
#[test]
fn test_shadow_state_takeover_prevented_by_reform() {
    let state0 = shadow_takeover_scenario_initial_state();
    let reform_policy = ShadowTakeoverScenario::aggressive_reform_policy();
    let seed = 77u64;

    let trajectory = run_scenario(&state0, &reform_policy, seed, 200);
    let final_state = trajectory.last().unwrap();

    let final_regime = classify_regime(
        final_state.legitimacy_milli,
        final_state.hidden_network.shadow_influence_index_milli,
    );

    assert_ne!(final_regime, RegimeType::ShadowStateTakeover,
        "Aggressive reform policy must prevent ShadowStateTakeover");

    // Under reform, capture R0 must eventually fall below threshold.
    let r0_at_end = final_state.hidden_network.capture_r0_milli;
    assert!(r0_at_end < 1200,
        "Reform must suppress capture R0 below supercritical regime (was {})", r0_at_end);
}
```

---

## 31. Extended Integration Notes

### 31.1 Hidden Network ↔ CIV-0103 Coupling

The `capture_intent` field of each `NetworkNode` references a `institution_id` from the CIV-0103 institution state machine. When a node achieves capture (`InfluencePressure > capture_threshold`):

1. CIV-0103's `InstitutionState` for that institution receives a `captured_by_shadow: true` flag
2. The institution's enforcement bias increases by `CAPTURE_ENFORCEMENT_BIAS` (default 200 milli-units)
3. Legitimacy delta from that institution's service delivery is reduced by `CAPTURE_SERVICE_PENALTY` (default 30%)
4. The `network.node_captured.v1` event is consumed by both CIV-0103 and CIV-0105 event logs

### 31.2 Hegemony ↔ Coalition Coupling

`HegemonyState.bloc_fragmentation_milli` is directly derived from the coalition model:

```
bloc_fragmentation_milli = max over all active coalitions of (
    coalition_members_exited / coalition_initial_size * 1000
)
```

Coalition collapse (`C₀ > 1000` for sustained ticks) directly contributes to hegemonic fragmentation.

### 31.3 War Profiteering ↔ Shadow Network Growth

War profiteering extraction feeds the shadow network's `resource_held` per the formula in Section 20.2. This creates a direct positive feedback:

```
Active conflict → defense_spend increases → profiteering_fraction increases
→ shadow_extraction increases → R_shadow increases
→ influence_score increases → capture_r0 increases
→ more institutional capture → more policy distortion toward more conflict
```

This feedback loop is the "pro-war shadow lobby" attractor identified in the research corpus (part_013, Section 6.1.C). The feedback is bounded by:
- `ExposureLoss` — detection events reduce R_shadow
- `reform_seizure` — governance reform actions reduce R_shadow
- `MOBILIZATION_FRACTION_MAX` (default 800 milli-units) caps defense spend

### 31.4 Espionage ↔ Battle Resolution

`IntelligenceAsset` grade effects on battle resolution are applied as modifiers to `BattleState.intelligence_modifier_milli`:

| Grade | Effect | Duration |
|---|---|---|
| Tactical | `intelligence_modifier_milli += 150` | 1 tick per collection success |
| Strategic | `intelligence_modifier_milli += 75`; exposes one treaty term | 3 ticks |
| Structural | Reveals 1 shadow node (reduces target `legitimacy_mask_milli` by 200) | Until stale |

Blown cover eliminates all intelligence modifiers from that asset immediately.

### 31.5 Treaty Compliance ↔ Diplomatic FSM

Treaty breaches interact with the FSM transition table (Section 2.2):

- `Alliance → Cooperative` transition trigger: `treaty_violation` — now specifically fires when an `AllianceObligation` term is breached
- `Cooperative → Strained` may fire even without grievance accumulation if `breach_count >= TREATY_BREACH_THRESHOLD` on any active treaty
- `IntelligenceSharing` treaty terms modify `detection_probability_milli` for both parties' assets by `-100` milli-units (shared counterintelligence reduces mutual surveillance costs)

---

## 32. Formal Theorem Chain — Complete Reference

With the v1.1 additions, the full theorem chain is:

| Theorem | Threshold Variable | Section | Failure Condition |
|---|---|---|---|
| T1: Sanctions Leakage Threshold | `L₀` | 4.3 | `L₀ > 1.0` → leakage grows to `Λ_max` |
| T2: Authoritarian Enforcement Backfire | `backfire_risk` | 7.3 | `E > E*` → enforcement raises leakage net |
| T3: Coalition Sanctions Stability | `C₀` | 6.1 | `C₀ > 1.0` → cascade exit, sanctions collapse |
| T4: Shadow-State Capture Threshold | `R₀_capture` | 18.7 | `R₀_capture > 1.0` → supercritical capture growth |
| T5: Order Stability (Hegemony) | `TS` (Transition Stress) | 22.1 | `TS > TS_crit` + two threshold breaches → war or fragmentation |

All five threshold variables are computed every tick and exported as metrics. The composite stability envelope is:

```
Stable global order requires simultaneously:
    L₀ < 1000   (sanctions effective)
    C₀ < 1000   (coalition holds)
    R₀_capture < 1000   (shadow capture contained)
    TS < 1000   (transition stress below critical)
    backfire_risk < 700   (enforcement not in backfire region)
```

Violation of any two simultaneously triggers instability events. Violation of three simultaneously is sufficient for `ShadowStateTakeover` or `SystemWar` basin entry depending on which three are violated.

---

**Version History:**
- v1.0 (2026-02-21): Full expansion from 28-line stub to complete engineering-grade specification.
- v1.1 (2026-02-21): Appended deep sections: Hidden Network Layer (full graph model, node/edge state, propagation algorithm), Espionage and Intelligence System (asset deployment, graded intel, counterintelligence), War Profiteering and Resource Extraction (war economy, arms trade, occupation ledger), Diplomacy Extended (formal treaty model, multilateral negotiations, international institutions), Long-Run Geopolitical Dynamics (hegemony cycle, alliance decay, ideological competition, phase diagram), Extended Event Taxonomy and DDL (8 new events, 5 new tables), Extended Test Suite (8 new stubs, shadow-state-takeover chaos scenario). Total: ~3,300 lines.
