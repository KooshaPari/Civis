# CIV-0106 Social, Ideology, Health, and Insurgency Spec v1

---

## Header

| Field              | Value                                                          |
|--------------------|----------------------------------------------------------------|
| Spec ID            | CIV-0106                                                       |
| Title              | Social, Ideology, Health, and Insurgency                       |
| Version            | 1.0.0                                                          |
| Status             | Draft                                                          |
| Date               | 2026-02-21                                                     |
| Authors            | CivLab                                                         |
| Related Specs      | CIV-0001 (Core Loop), CIV-0103 (Institutions/Lifecycle), CIV-0105 (War/Diplomacy/Shadow) |
| Crate              | `crates/social`                                                |
| Rust Edition       | 2021                                                           |
| Determinism Tier   | Tier-1 (fully deterministic, replay-stable)                   |

### CIV Sim Integration Notes (Overview)

This module runs inside the six-phase deterministic tick loop defined in CIV-0001. It owns:

- **Phase 2 (Policy Phase):** consumes intervention bundles, applies welfare floor, information integrity, and surge capacity adjustments.
- **Phase 3 (Deterministic Transition):** computes cohesion decay/reinforcement, ideology diffusion step, health burden update, and insurgency propensity recalculation.
- **Phase 4 (Stochastic Event Phase):** samples mobilization threshold crossings via ChaCha20Rng seeded from the canonical run seed.
- **Phase 5 (Metrics):** emits `social.state_updated.v1`, `health.welfare_updated.v1`, `insurgency.risk_updated.v1`, `ideology.diffusion_stepped.v1`, and `insurgency.cell_formed.v1` events.

Incoming coupling from other modules:

| Source Module  | Field                  | Consumed By                    |
|----------------|------------------------|--------------------------------|
| CIV-0105       | `enforcement_intensity`| Coercion index in insurgency propensity |
| CIV-0105       | `shadow_capture_score` | Institutional capture in cohesion decay |
| CIV-0103       | `citizen_lifecycle`    | Cohort stress drives health burden |
| CIV-0103       | `institution_state`    | Policy capacity multiplier for interventions |

Outgoing coupling to other modules:

| Destination    | Field                          | Purpose                             |
|----------------|--------------------------------|-------------------------------------|
| CIV-0105       | `insurgency_risk`              | Escalation trigger for conflict state |
| CIV-0103       | `cohort_stress_score`          | Feeds citizen lifecycle transitions |
| CIV-0001       | `social_legitimacy_modifier`   | Adjusts compliance rate calculation |

---

## Summary

This specification defines the computational models, state representations, database schemas, event contracts, Rust implementation structures, and acceptance criteria for the social dynamics subsystem of the CivLab simulation engine. The subsystem comprises four tightly coupled model components:

1. **Social Cohesion** — a bounded scalar field per region/cohort capturing collective trust and solidarity. Cohesion decays under material stress, coercion, and institutional capture; it is reinforced by service delivery, civic participation, and welfare floor maintenance.

2. **Ideology Diffusion** — a directed weighted graph over actor nodes where each node holds a multi-axis ideology vector in R^d. Per-tick diffusion propagates influence across edges weighted by contact rate and ideological similarity. Information integrity programs act as a damping coefficient on diffusion rates. Propaganda injects directed influence from state or shadow actors.

3. **Health and Welfare** — a compartmental model tracking cohort movement through health states (Healthy → Strained → Disabled → Deceased). Welfare coverage is a policy-controlled floor; surge capacity is a policy-controlled ceiling on health delivery speed. Health burden accumulates from material stress, unmet welfare needs, and epidemic shock events.

4. **Insurgency** — a propensity score per region computed from declared drivers with explicit coefficients. When propensity crosses a mobilization threshold, a stochastic cell formation event fires. Coercion raises short-term suppression but contributes to long-run propensity via cohesion decay feedback.

All models are Tier-1 deterministic: fixed-point arithmetic (i64 scaled), BTreeMap ordering, seeded ChaCha20Rng, and stable graph traversal. Every state transition emits a canonical event. Every intervention effect is declared and logged.

---

## 1. Social Cohesion Model

### 1.1 Field Definition

Cohesion is a bounded scalar field:

```
C(r, t) ∈ [0, 1]   for region r at tick t
```

Internally stored as `i64` in fixed-point Q16.16 (scale factor 65536). Values below 0 are clamped to 0; values above 65536 are clamped to 65536.

Cohesion is defined independently per (region_key, cohort_id) pair. The region aggregate is the population-weighted mean over cohorts.

### 1.2 Decay Function

Per tick, cohesion decays from the following drivers:

```
ΔC_decay(r, t) = -(
    α_stress  · stress(r, t)         +
    α_coerce  · coercion(r, t)        +
    α_capture · capture(r, t)         +
    α_polar   · polarization(r, t)    +
    α_health  · health_burden(r, t)
)
```

Where:
- `stress(r, t)` — normalized material stress score ∈ [0, 1] (from energy/food/income shortfall)
- `coercion(r, t)` — enforcement intensity from CIV-0105 ∈ [0, 1]
- `capture(r, t)` — institutional capture score from CIV-0105 shadow module ∈ [0, 1]
- `polarization(r, t)` — ideological polarization score ∈ [0, 1] (computed by ideology module)
- `health_burden(r, t)` — normalized health burden ∈ [0, 1]

Default coefficients (overridable via `PolicyBundle`):

| Coefficient   | Default |
|---------------|---------|
| `α_stress`    | 0.18    |
| `α_coerce`    | 0.14    |
| `α_capture`   | 0.20    |
| `α_polar`     | 0.10    |
| `α_health`    | 0.08    |

### 1.3 Reinforcement Function

Cohesion is reinforced each tick by:

```
ΔC_reinforce(r, t) = (
    β_welfare  · welfare_coverage(r, t)  +
    β_service  · service_delivery(r, t)  +
    β_civic    · civic_participation(r, t)+
    β_legitimacy · legitimacy(r, t)
) · (1 - C(r, t))
```

The `(1 - C(r, t))` term enforces a natural ceiling: reinforcement slows as cohesion approaches 1.0.

Default coefficients:

| Coefficient     | Default |
|-----------------|---------|
| `β_welfare`     | 0.15    |
| `β_service`     | 0.12    |
| `β_civic`       | 0.08    |
| `β_legitimacy`  | 0.10    |

### 1.4 Net Update

```
C(r, t+1) = clamp(C(r, t) + ΔC_reinforce - ΔC_decay, 0.0, 1.0)
```

### 1.5 Spatial Diffusion

Cohesion diffuses across adjacent regions via a spatial diffusion coefficient `κ_spatial`:

```
C_diff(r, t) = κ_spatial · Σ_{r' ∈ neighbors(r)} (C(r', t) - C(r, t)) / |neighbors(r)|
```

Default `κ_spatial = 0.05`. Neighbors are defined by the region adjacency graph (BTreeMap-keyed, stable ordering). Diffusion is applied after decay/reinforcement.

### 1.6 Polarization as Second-Order Effect

Polarization is not a primary field — it is derived from the variance of cohort-level cohesion within a region:

```
polarization(r, t) = Var_{cohorts c ∈ r}(C(c, t))
```

Where variance is computed over cohort cohesion values, scaled such that maximum inter-cohort variance maps to polarization = 1.0.

Polarization feeds back into cohesion decay (coefficient `α_polar`) creating a self-reinforcing divergence dynamic: as cohorts diverge, aggregate cohesion decays faster, which widens the divergence further.

---

## 2. Ideology Diffusion Model

### 2.1 Ideology Vector

Each node (actor or region) holds an ideology vector in R^6:

```
v = [v_market, v_state, v_liberty, v_equality, v_security, v_tradition]
```

All components are bounded ∈ [-1, 1]. Internally stored as i16 (scale: 32767 = 1.0). The full vector is stored in `IdeologyField`.

Axes:
- `v_market`: preference for market allocation vs. central planning (-1 = full central plan, +1 = full market)
- `v_state`: preference for state power vs. individual autonomy (-1 = minimal state, +1 = maximal state)
- `v_liberty`: preference for civil liberties vs. security controls
- `v_equality`: preference for redistributive equality vs. meritocratic hierarchy
- `v_security`: preference for stability and order vs. change tolerance
- `v_tradition`: preference for traditional norms vs. progressive change

### 2.2 Influence Graph

The ideology module operates over a directed weighted graph `G = (V, E)`:

- V: all ideological nodes (regions, cohorts, institutional actors, shadow networks)
- E: directed edges `(source, target, weight, contact_rate)`

Edge weight `w ∈ [0, 1]` encodes structural influence strength (e.g., media reach, social proximity, economic dependency). Contact rate `c ∈ [0, 1]` encodes frequency of interaction per tick. Both are stored as i16 fixed-point.

The effective influence of edge `(s → t)` per tick:

```
influence(s, t) = w(s,t) · c(s,t) · sim(v_s, v_t)
```

Where `sim(v_s, v_t)` is ideological similarity:

```
sim(v_s, v_t) = 1 - (||v_s - v_t||_2 / max_distance)
```

And `max_distance = sqrt(6 · 4) = sqrt(24)` (maximum possible L2 distance over 6 axes each spanning [-1,1]).

### 2.3 Diffusion Propagation Step

Per tick, for each target node `t`, compute the weighted mean of incoming source vectors:

```
Δv_t = η · Σ_{s: (s→t) ∈ E} influence(s, t) · (v_s - v_t)
```

Where `η` is the base diffusion rate (default: 0.04 per tick).

Apply information integrity damping:

```
Δv_t_damped = Δv_t · (1 - integrity_damping(t))
```

Where `integrity_damping(t) ∈ [0, 1]` is controlled by the information integrity intervention (default: 0.0, maximum damping: 0.80).

Update:

```
v_t(tick+1) = clamp(v_t(tick) + Δv_t_damped, -1.0, 1.0)  component-wise
```

### 2.4 Propaganda: Directed Influence Injection

Propaganda is modeled as a synthetic source node with a fixed ideology vector `v_prop` and a declaration of target nodes. At each tick:

```
Δv_target += propaganda_intensity · (v_prop - v_target)
```

Where `propaganda_intensity ∈ [0, 1]` is set by the actor controlling the propaganda channel (state, shadow network, or foreign actor). Propaganda injections are bounded by `max_propaganda_shift = 0.12` per tick per axis to prevent instantaneous opinion flipping.

Propaganda events are emitted as `ideology.diffusion_stepped.v1` with `source_type = "propaganda"` flag.

### 2.5 Ideological Distance Metric

Pairwise ideological distance used for coalition stability, insurgency alignment, and diplomatic coupling:

```
d(v_a, v_b) = ||v_a - v_b||_2 / max_distance   ∈ [0, 1]
```

Values > 0.7 indicate high ideological distance (potential for conflict or instability). Values < 0.2 indicate high alignment (coalition formation favorable).

### 2.6 Information Integrity Programs

The information integrity intervention sets `integrity_damping` for target nodes. Effect:

```
integrity_damping = clamp(base_integrity + Σ_programs program_strength, 0.0, 0.80)
```

Programs are declared in the `InterventionRecord` and have explicit duration in ticks. Upon expiry the damping contribution of that program drops to zero. Programs stack additively up to the cap.

---

## 3. Health and Welfare Model

### 3.1 Compartmental Health Model

Cohorts move through four health states:

```
Healthy (H) → Strained (S) → Disabled (D) → Deceased (X)
```

Transition rates per tick:

```
λ_HS = f(material_stress, welfare_gap, epidemic_shock)
λ_SD = g(health_burden, welfare_gap, age_factor)
λ_DX = h(health_burden, surge_capacity_deficit)
λ_SH = recovery_rate · welfare_coverage · surge_capacity
λ_DS = partial_recovery_rate · welfare_coverage
```

Where:
- `welfare_gap = max(0, welfare_floor - actual_welfare_coverage)`
- `surge_capacity_deficit = max(0, demand - surge_capacity_ceiling)`

### 3.2 Health Burden Accumulation

Aggregate health burden for cohort `c` at tick `t`:

```
burden(c, t) = burden(c, t-1) · (1 - recovery_rate)
             + λ_HS(c, t) · stress_weight
             + λ_SD(c, t) · disability_weight
             + epidemic_shock(c, t)
             - welfare_relief(c, t)
```

Where:
- `recovery_rate = base_recovery · surge_capacity · welfare_coverage`
- `stress_weight = 0.30`
- `disability_weight = 0.55`
- `epidemic_shock` is zero except during declared shock events
- `welfare_relief = β_welfare · welfare_coverage · (1 - burden)`

Burden is bounded ∈ [0, 1]. Internally stored as i64 fixed-point Q16.16.

### 3.3 Welfare Coverage

Welfare coverage `W(r, t)` is a policy-controlled variable in [0, 1]:

```
W(r, t) = clamp(welfare_floor_policy + delivery_capacity(r, t) - leakage(r, t), 0.0, 1.0)
```

Where:
- `welfare_floor_policy` is set by the `WelfareFloorAdjustment` intervention
- `delivery_capacity` is bounded by institutional capacity multiplier (from CIV-0103)
- `leakage` is drawn from shadow capture score (from CIV-0105): `leakage = shadow_capture · 0.25`

### 3.4 Surge Capacity

Public health surge capacity `Q(r, t)` sets the maximum health delivery rate:

```
Q(r, t) = base_capacity + surge_investment(r, t) · surge_multiplier - degradation(r, t)
```

Where:
- `base_capacity = 0.30` (default — sustains population without growth stress)
- `surge_investment` is set by the `SurgeCapacityIntervention`
- `surge_multiplier = 0.60`
- `degradation = conflict_intensity · 0.15 + coercion · 0.05`

Surge capacity is bounded ∈ [0, 1].

### 3.5 Lag and Diffusion Parameters

Health shocks propagate with:
- **Lag:** epidemic shocks apply with a 3-tick lag (configurable via `EpidemicShockParams.lag_ticks`)
- **Spatial diffusion coefficient:** `κ_health = 0.03` across region adjacency edges per tick
- **Intervention effect delay:** welfare floor changes take effect at `t + 1`; surge capacity changes take effect at `t + 2`

---

## 4. Insurgency Model

### 4.1 Insurgency Propensity

Insurgency propensity (risk) for region `r` at tick `t`:

```
risk(r, t) = clamp(
    γ_stress   · material_stress(r, t)       +
    γ_coerce   · coercion(r, t)              +
    γ_capture  · capture(r, t)               +
    γ_polar    · ideology_polarization(r, t)  +
    γ_cohesion · (1 - cohesion(r, t))        +
    γ_welfare  · welfare_gap(r, t)           +
    γ_legit    · (1 - legitimacy(r, t))      -
    γ_deter    · deterrence(r, t),
    0.0, 1.0
)
```

Default coefficients:

| Coefficient   | Default | Description                                      |
|---------------|---------|--------------------------------------------------|
| `γ_stress`    | 0.22    | Material deprivation driver                      |
| `γ_coerce`    | 0.18    | Coercion grievance driver                        |
| `γ_capture`   | 0.15    | Institutional illegitimacy driver                |
| `γ_polar`     | 0.12    | Ideological fragmentation driver                 |
| `γ_cohesion`  | 0.10    | Low solidarity driver                            |
| `γ_welfare`   | 0.14    | Unmet welfare need driver                        |
| `γ_legit`     | 0.16    | Legitimacy deficit driver                        |
| `γ_deter`     | 0.08    | State deterrence suppression                     |

All coefficients are overridable via `InsurgencyParams` in the `PolicyBundle`.

### 4.2 Mobilization Score

Mobilization score tracks the accumulation of insurgent capacity:

```
mobilization(r, t) = mobilization(r, t-1) + recruitment_rate(r, t) - attrition(r, t)
```

Where:
```
recruitment_rate(r, t) = risk(r, t) · population(r) · recruit_susceptibility(r, t)
attrition(r, t) = deterrence(r, t) · mobilization(r, t-1) · attrition_coefficient
```

`recruit_susceptibility` is highest among cohorts with stage `dissenting` (from CIV-0103 citizen lifecycle).

### 4.3 Mobilization Threshold and Cell Formation

When `mobilization(r, t) ≥ mobilization_threshold(r)`, a stochastic cell formation event fires:

```
p_cell = sigmoid((mobilization - threshold) / threshold_sensitivity)
```

Cell formation is sampled from `ChaCha20Rng` seeded by the canonical run seed. If fired, a `MobilizationCell` record is created and `insurgency.cell_formed.v1` is emitted.

Default `mobilization_threshold = 0.65`. Default `threshold_sensitivity = 0.10`.

Cell formation is non-destructive to mobilization score (cells persist as ongoing entities). Post-formation, active cells add a `γ_cell · cell_count` term to risk in subsequent ticks.

### 4.4 Legitimacy Dynamics

Legitimacy for region `r`:

```
legitimacy(r, t+1) = legitimacy(r, t)
    + δ_service · service_delivery(r, t)
    - δ_corrupt · corruption(r, t)
    - δ_coerce  · coercion_overreach(r, t)
    - δ_harm    · war_harm(r, t)
    + δ_amnesty · amnesty_applied(r, t)
```

Default coefficients:

| Coefficient    | Default |
|----------------|---------|
| `δ_service`    | 0.10    |
| `δ_corrupt`    | 0.14    |
| `δ_coerce`     | 0.18    |
| `δ_harm`       | 0.20    |
| `δ_amnesty`    | 0.12    |

Legitimacy is bounded ∈ [0, 1].

### 4.5 Tyranny Index

A composite governance pathology index:

```
T(r, t) = σ(
    w1 · survival_dependence(r, t)  +
    w2 · goodhart_pressure(r, t)    +
    w3 · admin_coercion(r, t)       +
    w4 · stratification_lock(r, t)  +
    w5 · scarcity_amplification(r,t)-
    w6 · baseline(r, t)             -
    w7 · governance_quality(r, t)
)
```

Where `σ` is the logistic sigmoid. Weights:

| Weight | Default | Description                                 |
|--------|---------|---------------------------------------------|
| `w1`   | 0.18    | Dependency on state survival allocation     |
| `w2`   | 0.14    | Metric-to-rights coupling pressure          |
| `w3`   | 0.20    | Administrative coercion intensity           |
| `w4`   | 0.16    | Mobility lock-in / stratification           |
| `w5`   | 0.12    | Scarcity amplification of control           |
| `w6`   | 0.10    | Baseline institutional quality offset       |
| `w7`   | 0.15    | Net governance quality offset               |

Tyranny index feeds back into legitimacy decay and insurgency propensity. High tyranny accelerates coercion-cohesion tradeoff deterioration.

---

## 5. Intervention Surfaces

### 5.1 Welfare Floor Adjustment

**Lever:** `WelfareFloorAdjustment { region_key, new_floor: f64, effective_tick: u64 }`

**Effect function:**
```
W_floor(r, t) = new_floor      for t ≥ effective_tick
welfare_coverage(r, t) = max(welfare_coverage(r, t), W_floor(r, t))
```

**Downstream effects:**
- Reduces `welfare_gap` → reduces health burden accumulation rate
- Reduces `γ_welfare` contribution to insurgency propensity
- Reinforces cohesion (β_welfare term)
- Requires institutional delivery capacity ≥ new_floor to take full effect; excess floor above capacity is partially absorbed (leakage applies)

**Event emitted:** `health.welfare_updated.v1` with `intervention_id` field.

### 5.2 Information Integrity and Civic Education Programs

**Lever:** `InformationIntegrityProgram { region_key, target_nodes: Vec<NodeId>, program_strength: f64, duration_ticks: u64 }`

**Effect function:**
```
integrity_damping(node, t) += program_strength   for t ∈ [start, start + duration_ticks)
ideology_diffusion_rate(node, t) *= (1 - integrity_damping(node, t))
```

**Downstream effects:**
- Slows ideology drift in target nodes
- Reduces propaganda effectiveness at target nodes by the same damping factor
- Civic education component (when `civic_education: true`): adds `β_civic` increment to cohesion reinforcement

**Event emitted:** `social.state_updated.v1` with `intervention_type = "information_integrity"`.

### 5.3 Public Health Surge Capacity

**Lever:** `SurgeCapacityIntervention { region_key, surge_investment: f64, duration_ticks: u64 }`

**Effect function:**
```
Q(r, t) = base_capacity + surge_investment · surge_multiplier   for t ∈ [start, start + duration_ticks)
recovery_rate(r, t) = base_recovery · Q(r, t) · welfare_coverage(r, t)
```

**Downstream effects:**
- Accelerates λ_SH and λ_DS recovery transitions
- Reduces health burden accumulation
- Reduces insurgency γ_welfare contribution indirectly via lower burden → lower stress

**Event emitted:** `health.welfare_updated.v1` with `intervention_type = "surge_capacity"`.

### 5.4 De-escalation and Amnesty

**Lever:** `AmnestyCampaign { region_key, amnesty_strength: f64, effective_ticks: u64 }`

**Effect function:**
```
mobilization(r, t) -= amnesty_strength · mobilization(r, t)   for t ∈ [start, start + effective_ticks)
legitimacy(r, t) += δ_amnesty · amnesty_strength
insurgency_risk(r, t) *= (1 - amnesty_strength · 0.30)
```

**Downstream effects:**
- Rolls back mobilization score (bounded — cannot go negative)
- Increases legitimacy (δ_amnesty)
- Reduces instantaneous risk via the `amnesty_applied` term in legitimacy dynamics
- Does NOT eliminate MobilizationCell records — those require dedicated demobilization events

**Event emitted:** `insurgency.risk_updated.v1` with `intervention_type = "amnesty"`.

### 5.5 Intervention Stack and Priority

Multiple simultaneous interventions are applied in a declared priority order within Phase 2:

1. Surge capacity (health infrastructure first)
2. Welfare floor (material baseline second)
3. Information integrity programs (ideology damping third)
4. Amnesty campaigns (de-escalation last, after all other state is updated)

If two interventions conflict on the same field, the higher-priority intervention's value is used for that field in that tick. Conflicts are logged as `intervention.conflict_detected.v1` (internal diagnostic event, not in public schema).

---

## 6. State Structs (Rust)

```rust
// crates/social/src/types.rs

use std::collections::BTreeMap;

/// Fixed-point scaling factor: Q16.16 (1.0 = 65536)
pub const FP_SCALE: i64 = 65_536;

/// Bounded fixed-point value in [0, FP_SCALE]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FpUnit(pub i64);

impl FpUnit {
    pub const ZERO: FpUnit = FpUnit(0);
    pub const ONE: FpUnit = FpUnit(FP_SCALE);

    pub fn clamp(self) -> Self {
        FpUnit(self.0.clamp(0, FP_SCALE))
    }

    pub fn as_f64(self) -> f64 {
        self.0 as f64 / FP_SCALE as f64
    }

    pub fn from_f64(v: f64) -> Self {
        FpUnit((v * FP_SCALE as f64) as i64)
    }
}

/// Ideology axis value in [-32767, 32767] (Q0.15 fixed-point: 32767 = 1.0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdeologyAxis(pub i16);

impl IdeologyAxis {
    pub const NEG_ONE: IdeologyAxis = IdeologyAxis(-32767);
    pub const ZERO: IdeologyAxis = IdeologyAxis(0);
    pub const POS_ONE: IdeologyAxis = IdeologyAxis(32767);

    pub fn as_f64(self) -> f64 {
        self.0 as f64 / 32767.0
    }

    pub fn from_f64(v: f64) -> Self {
        IdeologyAxis((v.clamp(-1.0, 1.0) * 32767.0) as i16)
    }
}

/// Six-axis ideology vector
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeologyVector {
    pub market:    IdeologyAxis, // -1 = central plan, +1 = free market
    pub state:     IdeologyAxis, // -1 = minimal state, +1 = maximal state
    pub liberty:   IdeologyAxis, // -1 = security priority, +1 = civil liberty
    pub equality:  IdeologyAxis, // -1 = meritocratic, +1 = redistributive
    pub security:  IdeologyAxis, // -1 = change tolerance, +1 = stability
    pub tradition: IdeologyAxis, // -1 = progressive, +1 = traditional
}

impl IdeologyVector {
    pub fn l2_distance(&self, other: &IdeologyVector) -> f64 {
        let axes = [
            (self.market.as_f64()    - other.market.as_f64()),
            (self.state.as_f64()     - other.state.as_f64()),
            (self.liberty.as_f64()   - other.liberty.as_f64()),
            (self.equality.as_f64()  - other.equality.as_f64()),
            (self.security.as_f64()  - other.security.as_f64()),
            (self.tradition.as_f64() - other.tradition.as_f64()),
        ];
        let sum_sq: f64 = axes.iter().map(|d| d * d).sum();
        sum_sq.sqrt()
    }

    /// Normalized similarity score ∈ [0, 1]; 1.0 = identical
    pub fn similarity(&self, other: &IdeologyVector) -> f64 {
        const MAX_DISTANCE: f64 = 4.899; // sqrt(24)
        1.0 - (self.l2_distance(other) / MAX_DISTANCE).clamp(0.0, 1.0)
    }
}

/// Per-(region, cohort) social state snapshot at one tick
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocialState {
    pub run_id:        u64,
    pub tick:          u64,
    pub region_key:    String,
    pub cohort_id:     String,
    pub cohesion:      FpUnit,      // [0, FP_SCALE]
    pub polarization:  FpUnit,      // [0, FP_SCALE]; derived from inter-cohort variance
    pub legitimacy:    FpUnit,      // [0, FP_SCALE]
    pub tyranny_index: FpUnit,      // [0, FP_SCALE]
    pub created_at:    u64,         // tick-derived timestamp; NO SystemTime
}

/// Per-region cohesion field (aggregate over cohorts)
#[derive(Debug, Clone)]
pub struct RegionCohesion {
    pub region_key:       String,
    pub cohort_cohesion:  BTreeMap<String, FpUnit>, // ordered by cohort_id for determinism
    pub aggregate:        FpUnit,                   // population-weighted mean
    pub polarization:     FpUnit,                   // inter-cohort variance
    pub neighbors:        Vec<String>,              // sorted for determinism
    pub spatial_diffusion_coeff: FpUnit,
}

impl RegionCohesion {
    /// Recompute aggregate and polarization from cohort_cohesion.
    /// Must be called after any cohort update. BTreeMap guarantees stable iteration order.
    pub fn recompute_aggregate(&mut self) {
        let values: Vec<f64> = self.cohort_cohesion.values().map(|v| v.as_f64()).collect();
        if values.is_empty() {
            self.aggregate = FpUnit::ZERO;
            self.polarization = FpUnit::ZERO;
            return;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        self.aggregate = FpUnit::from_f64(mean).clamp();
        self.polarization = FpUnit::from_f64(variance.clamp(0.0, 0.25) * 4.0).clamp(); // normalize: max variance 0.25 → 1.0
    }
}

/// A node in the ideology diffusion graph
#[derive(Debug, Clone)]
pub struct IdeologyField {
    pub node_id:            String,
    pub node_type:          IdeologyNodeType,
    pub vector:             IdeologyVector,
    pub integrity_damping:  FpUnit,  // [0, 0.80·FP_SCALE] set by interventions
    pub propaganda_target:  bool,
    pub tick_last_updated:  u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdeologyNodeType {
    Region,
    Cohort,
    Institution,
    ShadowNetwork,
    ForeignActor,
}

/// A directed edge in the ideology diffusion graph
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdeologyEdge {
    pub source_id:   String,
    pub target_id:   String,
    pub weight:      i16,       // Q0.15 fixed-point; [0, 32767]
    pub contact_rate: i16,      // Q0.15 fixed-point; [0, 32767]
}

impl IdeologyEdge {
    pub fn effective_influence(&self, source_vec: &IdeologyVector, target_vec: &IdeologyVector) -> f64 {
        let w = self.weight as f64 / 32767.0;
        let c = self.contact_rate as f64 / 32767.0;
        let sim = source_vec.similarity(target_vec);
        w * c * sim
    }
}

/// Health and welfare state for one cohort at one tick
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthWelfareState {
    pub run_id:            u64,
    pub tick:              u64,
    pub cohort_id:         String,
    pub region_key:        String,
    pub population_healthy:   i64,  // absolute population count
    pub population_strained:  i64,
    pub population_disabled:  i64,
    pub population_deceased:  i64,
    pub health_burden:     FpUnit,  // aggregate burden [0, FP_SCALE]
    pub welfare_coverage:  FpUnit,  // [0, FP_SCALE]
    pub surge_capacity:    FpUnit,  // [0, FP_SCALE]
    pub welfare_floor:     FpUnit,  // policy-set floor [0, FP_SCALE]
    pub created_at:        u64,
}

/// Insurgency state for one region at one tick
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsurgencyState {
    pub run_id:               u64,
    pub tick:                 u64,
    pub region_key:           String,
    pub risk:                 FpUnit,              // propensity score [0, FP_SCALE]
    pub mobilization_score:   FpUnit,              // accumulated mobilization [0, FP_SCALE]
    pub active_cell_count:    u32,
    pub legitimacy:           FpUnit,
    pub tyranny_index:        FpUnit,
    pub coercion_index:       FpUnit,              // from CIV-0105
    pub deterrence:           FpUnit,
    pub created_at:           u64,
}

/// A mobilized insurgent cell
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobilizationCell {
    pub cell_id:              u64,                 // monotonic within run
    pub run_id:               u64,
    pub tick_formed:          u64,
    pub region_key:           String,
    pub formation_risk:       FpUnit,              // risk at moment of formation
    pub formation_mobilization: FpUnit,
    pub ideology_vector:      IdeologyVector,      // ideological character of cell
    pub active:               bool,
}

/// A logged intervention record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterventionRecord {
    pub intervention_id:   u64,
    pub run_id:            u64,
    pub tick_applied:      u64,
    pub tick_expires:      Option<u64>,            // None = permanent
    pub region_key:        String,
    pub intervention_type: InterventionType,
    pub parameters:        BTreeMap<String, i64>,  // fixed-point parameter values
    pub effect_emitted:    bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterventionType {
    WelfareFloorAdjustment,
    InformationIntegrityProgram,
    SurgeCapacity,
    AmnestyCampaign,
    CivicEducation,
    PropagandaInjection,
}

/// Polarization metrics snapshot for a region
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolarizationMetrics {
    pub run_id:                   u64,
    pub tick:                     u64,
    pub region_key:               String,
    pub inter_cohort_variance:    FpUnit,           // cohesion variance
    pub ideology_dispersion:      FpUnit,           // mean pairwise ideological distance
    pub echo_chamber_index:       FpUnit,           // fraction of edges within homogeneous clusters
    pub polarization_velocity:    i64,              // signed rate of change (FpUnit/tick)
    pub created_at:               u64,
}
```

---

## 7. Rust Module Layout

The social module lives in `crates/social`. This crate must be added to the workspace `Cargo.toml`.

```
crates/social/
├── Cargo.toml
└── src/
    ├── lib.rs                  # Public API surface; re-exports all public types
    ├── types.rs                # All state structs (Section 6 above)
    ├── cohesion.rs             # Social cohesion decay, reinforcement, spatial diffusion
    ├── ideology/
    │   ├── mod.rs              # Ideology module public API
    │   ├── graph.rs            # IdeologyEdge graph, BTreeMap-keyed adjacency
    │   ├── diffusion.rs        # Per-tick diffusion propagation step
    │   ├── propaganda.rs       # Directed influence injection
    │   └── metrics.rs          # Polarization, distance, echo chamber index
    ├── health/
    │   ├── mod.rs              # Health module public API
    │   ├── compartmental.rs    # Healthy/Strained/Disabled/Deceased transitions
    │   ├── burden.rs           # Burden accumulation and recovery
    │   └── welfare.rs          # Welfare coverage and surge capacity
    ├── insurgency/
    │   ├── mod.rs              # Insurgency module public API
    │   ├── propensity.rs       # Propensity calculation with explicit coefficients
    │   ├── mobilization.rs     # Score accumulation and cell formation
    │   ├── legitimacy.rs       # Legitimacy dynamics
    │   └── tyranny.rs          # Tyranny index computation
    ├── interventions/
    │   ├── mod.rs              # Intervention application pipeline
    │   ├── welfare.rs          # WelfareFloorAdjustment + SurgeCapacity effects
    │   ├── information.rs      # InformationIntegrityProgram + CivicEducation
    │   └── amnesty.rs          # AmnestyCampaign de-escalation
    ├── events.rs               # Event struct definitions and emission helpers
    ├── params.rs               # All coefficient structs (SocialParams, InsurgencyParams, etc.)
    └── tick.rs                 # Top-level tick step function: social_tick(state, policy, rng)
```

`crates/social/Cargo.toml`:

```toml
[package]
name    = "social"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
rand        = { version = "0.8", features = ["std_rng"] }
rand_chacha = "0.3"
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"

[dev-dependencies]
proptest    = "1"
```

Top-level `Cargo.toml` workspace members update:

```toml
[workspace]
members = [
  "crates/engine",
  "crates/policy",
  "crates/metrics",
  "crates/io",
  "crates/server",
  "crates/social",           # ADD THIS LINE
]
```

Main entry point `crates/social/src/tick.rs`:

```rust
// crates/social/src/tick.rs

use rand_chacha::ChaCha20Rng;
use crate::types::*;
use crate::params::SocialTickParams;

/// Top-level tick step for the social module.
/// Called from the engine's Phase 3 (Deterministic Transition).
/// All mutation is deterministic given the same params and rng seed.
///
/// # Invariants
/// - rng is ChaCha20Rng seeded from canonical run seed; callers must not use SystemTime.
/// - All BTreeMap iterations are stable across platforms.
/// - Returns updated state; original state is not mutated in place.
pub fn social_tick(
    prev_state:  &SocialSnapshot,
    params:      &SocialTickParams,
    rng:         &mut ChaCha20Rng,
) -> SocialSnapshot {
    let mut next = prev_state.clone();

    // Phase order is fixed and declared:
    // 1. Apply active interventions (from policy bundle, pre-loaded by caller)
    crate::interventions::apply_all(&mut next, &params.interventions);

    // 2. Ideology diffusion step (deterministic, BTreeMap-ordered node traversal)
    crate::ideology::diffusion::step(&mut next.ideology_fields, &next.ideology_edges, &params.ideology);

    // 3. Cohesion decay and reinforcement (per region/cohort, BTreeMap-ordered)
    crate::cohesion::update(&mut next.region_cohesion, &next, &params.cohesion);

    // 4. Health/welfare compartmental transitions
    crate::health::compartmental::step(&mut next.health_welfare, &params.health);

    // 5. Health burden accumulation
    crate::health::burden::update(&mut next.health_welfare, &next, &params.health);

    // 6. Insurgency propensity and mobilization
    crate::insurgency::propensity::update(&mut next.insurgency, &next, &params.insurgency);
    crate::insurgency::mobilization::update(&mut next.insurgency, &params.insurgency, rng);

    // 7. Legitimacy and tyranny index
    crate::insurgency::legitimacy::update(&mut next.social_states, &next, &params.insurgency);
    crate::insurgency::tyranny::update(&mut next.social_states, &next, &params.insurgency);

    next
}
```

---

## 8. Event Contracts

All events conform to the CIV-0001 event envelope schema. The `payload` field contains the module-specific JSON object defined below.

### 8.1 `social.state_updated.v1`

```json
{
  "$schema": "https://civlab.internal/schemas/social.state_updated.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key", "cohort_id", "cohesion", "polarization", "legitimacy", "tyranny_index", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":     { "type": "string", "const": "social.state_updated.v1" },
    "version":        { "type": "string", "const": "1" },
    "run_id":         { "type": "integer", "minimum": 1 },
    "tick":           { "type": "integer", "minimum": 0 },
    "region_key":     { "type": "string", "minLength": 1 },
    "cohort_id":      { "type": "string", "minLength": 1 },
    "cohesion":       { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "polarization":   { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "legitimacy":     { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "tyranny_index":  { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "intervention_type": { "type": ["string", "null"] },
    "intervention_id":   { "type": ["integer", "null"] },
    "created_at":     { "type": "integer", "minimum": 0 }
  }
}
```

### 8.2 `health.welfare_updated.v1`

```json
{
  "$schema": "https://civlab.internal/schemas/health.welfare_updated.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "cohort_id", "region_key", "health_burden", "welfare_coverage", "surge_capacity", "population_healthy", "population_strained", "population_disabled", "population_deceased", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":            { "type": "string", "const": "health.welfare_updated.v1" },
    "version":               { "type": "string", "const": "1" },
    "run_id":                { "type": "integer", "minimum": 1 },
    "tick":                  { "type": "integer", "minimum": 0 },
    "cohort_id":             { "type": "string", "minLength": 1 },
    "region_key":            { "type": "string", "minLength": 1 },
    "health_burden":         { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "welfare_coverage":      { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "surge_capacity":        { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "welfare_floor":         { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "population_healthy":    { "type": "integer", "minimum": 0 },
    "population_strained":   { "type": "integer", "minimum": 0 },
    "population_disabled":   { "type": "integer", "minimum": 0 },
    "population_deceased":   { "type": "integer", "minimum": 0 },
    "intervention_type":     { "type": ["string", "null"] },
    "intervention_id":       { "type": ["integer", "null"] },
    "created_at":            { "type": "integer", "minimum": 0 }
  }
}
```

### 8.3 `insurgency.risk_updated.v1`

```json
{
  "$schema": "https://civlab.internal/schemas/insurgency.risk_updated.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key", "risk", "mobilization_score", "active_cell_count", "legitimacy", "coercion_index", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":          { "type": "string", "const": "insurgency.risk_updated.v1" },
    "version":             { "type": "string", "const": "1" },
    "run_id":              { "type": "integer", "minimum": 1 },
    "tick":                { "type": "integer", "minimum": 0 },
    "region_key":          { "type": "string", "minLength": 1 },
    "risk":                { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "mobilization_score":  { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "active_cell_count":   { "type": "integer", "minimum": 0 },
    "legitimacy":          { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "tyranny_index":       { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "coercion_index":      { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "deterrence":          { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "intervention_type":   { "type": ["string", "null"] },
    "intervention_id":     { "type": ["integer", "null"] },
    "created_at":          { "type": "integer", "minimum": 0 }
  }
}
```

### 8.4 `ideology.diffusion_stepped.v1`

```json
{
  "$schema": "https://civlab.internal/schemas/ideology.diffusion_stepped.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "node_id", "node_type", "vector_before", "vector_after", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":      { "type": "string", "const": "ideology.diffusion_stepped.v1" },
    "version":         { "type": "string", "const": "1" },
    "run_id":          { "type": "integer", "minimum": 1 },
    "tick":            { "type": "integer", "minimum": 0 },
    "node_id":         { "type": "string", "minLength": 1 },
    "node_type":       { "type": "string", "enum": ["Region", "Cohort", "Institution", "ShadowNetwork", "ForeignActor"] },
    "vector_before": {
      "type": "object",
      "required": ["market", "state", "liberty", "equality", "security", "tradition"],
      "additionalProperties": false,
      "properties": {
        "market":    { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "state":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "liberty":   { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "equality":  { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "security":  { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "tradition": { "type": "number", "minimum": -1.0, "maximum": 1.0 }
      }
    },
    "vector_after": {
      "type": "object",
      "required": ["market", "state", "liberty", "equality", "security", "tradition"],
      "additionalProperties": false,
      "properties": {
        "market":    { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "state":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "liberty":   { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "equality":  { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "security":  { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "tradition": { "type": "number", "minimum": -1.0, "maximum": 1.0 }
      }
    },
    "source_type":         { "type": ["string", "null"] },
    "integrity_damping":   { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at":          { "type": "integer", "minimum": 0 }
  }
}
```

### 8.5 `insurgency.cell_formed.v1`

```json
{
  "$schema": "https://civlab.internal/schemas/insurgency.cell_formed.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "cell_id", "region_key", "formation_risk", "formation_mobilization", "ideology_vector", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":             { "type": "string", "const": "insurgency.cell_formed.v1" },
    "version":                { "type": "string", "const": "1" },
    "run_id":                 { "type": "integer", "minimum": 1 },
    "tick":                   { "type": "integer", "minimum": 0 },
    "cell_id":                { "type": "integer", "minimum": 1 },
    "region_key":             { "type": "string", "minLength": 1 },
    "formation_risk":         { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "formation_mobilization": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "ideology_vector": {
      "type": "object",
      "required": ["market", "state", "liberty", "equality", "security", "tradition"],
      "additionalProperties": false,
      "properties": {
        "market":    { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "state":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "liberty":   { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "equality":  { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "security":  { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "tradition": { "type": "number", "minimum": -1.0, "maximum": 1.0 }
      }
    },
    "cell_ideology_distance_from_state": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at":             { "type": "integer", "minimum": 0 }
  }
}
```

---

## 9. Database Schema

All tables are append-only and tick-keyed. `run_id` + `tick` + entity key form the composite primary key. No UPDATE or DELETE is ever issued against these tables in normal operation.

```sql
-- social_state: Per (region, cohort) social dynamics snapshot per tick
CREATE TABLE IF NOT EXISTS social_state (
    id              BIGSERIAL   PRIMARY KEY,
    run_id          BIGINT      NOT NULL,
    tick            BIGINT      NOT NULL,
    region_key      TEXT        NOT NULL,
    cohort_id       TEXT        NOT NULL,
    cohesion        REAL        NOT NULL CHECK (cohesion >= 0.0 AND cohesion <= 1.0),
    polarization    REAL        NOT NULL CHECK (polarization >= 0.0 AND polarization <= 1.0),
    legitimacy      REAL        NOT NULL CHECK (legitimacy >= 0.0 AND legitimacy <= 1.0),
    tyranny_index   REAL        NOT NULL CHECK (tyranny_index >= 0.0 AND tyranny_index <= 1.0),
    created_at      BIGINT      NOT NULL,
    UNIQUE (run_id, tick, region_key, cohort_id)
);

CREATE INDEX idx_social_state_run_tick
    ON social_state (run_id, tick);

CREATE INDEX idx_social_state_region
    ON social_state (run_id, region_key, tick);

-- health_welfare_state: Per cohort health and welfare dynamics per tick
CREATE TABLE IF NOT EXISTS health_welfare_state (
    id                    BIGSERIAL   PRIMARY KEY,
    run_id                BIGINT      NOT NULL,
    tick                  BIGINT      NOT NULL,
    cohort_id             TEXT        NOT NULL,
    region_key            TEXT        NOT NULL,
    health_burden         REAL        NOT NULL CHECK (health_burden >= 0.0 AND health_burden <= 1.0),
    welfare_coverage      REAL        NOT NULL CHECK (welfare_coverage >= 0.0 AND welfare_coverage <= 1.0),
    welfare_floor         REAL        NOT NULL CHECK (welfare_floor >= 0.0 AND welfare_floor <= 1.0),
    surge_capacity        REAL        NOT NULL CHECK (surge_capacity >= 0.0 AND surge_capacity <= 1.0),
    population_healthy    BIGINT      NOT NULL CHECK (population_healthy >= 0),
    population_strained   BIGINT      NOT NULL CHECK (population_strained >= 0),
    population_disabled   BIGINT      NOT NULL CHECK (population_disabled >= 0),
    population_deceased   BIGINT      NOT NULL CHECK (population_deceased >= 0),
    created_at            BIGINT      NOT NULL,
    UNIQUE (run_id, tick, cohort_id)
);

CREATE INDEX idx_health_welfare_run_tick
    ON health_welfare_state (run_id, tick);

CREATE INDEX idx_health_welfare_cohort
    ON health_welfare_state (run_id, cohort_id, tick);

-- insurgency_state: Per region insurgency dynamics per tick
CREATE TABLE IF NOT EXISTS insurgency_state (
    id                  BIGSERIAL   PRIMARY KEY,
    run_id              BIGINT      NOT NULL,
    tick                BIGINT      NOT NULL,
    region_key          TEXT        NOT NULL,
    risk                REAL        NOT NULL CHECK (risk >= 0.0 AND risk <= 1.0),
    mobilization_score  REAL        NOT NULL CHECK (mobilization_score >= 0.0 AND mobilization_score <= 1.0),
    active_cell_count   INTEGER     NOT NULL CHECK (active_cell_count >= 0),
    legitimacy          REAL        NOT NULL CHECK (legitimacy >= 0.0 AND legitimacy <= 1.0),
    tyranny_index       REAL        NOT NULL CHECK (tyranny_index >= 0.0 AND tyranny_index <= 1.0),
    coercion_index      REAL        NOT NULL CHECK (coercion_index >= 0.0 AND coercion_index <= 1.0),
    deterrence          REAL        NOT NULL CHECK (deterrence >= 0.0 AND deterrence <= 1.0),
    created_at          BIGINT      NOT NULL,
    UNIQUE (run_id, tick, region_key)
);

CREATE INDEX idx_insurgency_run_tick
    ON insurgency_state (run_id, tick);

CREATE INDEX idx_insurgency_region
    ON insurgency_state (run_id, region_key, tick);

-- ideology_state: Per node ideology vector per tick (sparse; only emitted on change > threshold)
CREATE TABLE IF NOT EXISTS ideology_state (
    id                  BIGSERIAL   PRIMARY KEY,
    run_id              BIGINT      NOT NULL,
    tick                BIGINT      NOT NULL,
    node_id             TEXT        NOT NULL,
    node_type           TEXT        NOT NULL,
    v_market            REAL        NOT NULL CHECK (v_market >= -1.0 AND v_market <= 1.0),
    v_state             REAL        NOT NULL CHECK (v_state >= -1.0 AND v_state <= 1.0),
    v_liberty           REAL        NOT NULL CHECK (v_liberty >= -1.0 AND v_liberty <= 1.0),
    v_equality          REAL        NOT NULL CHECK (v_equality >= -1.0 AND v_equality <= 1.0),
    v_security          REAL        NOT NULL CHECK (v_security >= -1.0 AND v_security <= 1.0),
    v_tradition         REAL        NOT NULL CHECK (v_tradition >= -1.0 AND v_tradition <= 1.0),
    integrity_damping   REAL        NOT NULL CHECK (integrity_damping >= 0.0 AND integrity_damping <= 1.0),
    created_at          BIGINT      NOT NULL,
    UNIQUE (run_id, tick, node_id)
);

CREATE INDEX idx_ideology_run_tick
    ON ideology_state (run_id, tick);

-- mobilization_cells: One row per cell formation event; persistent across ticks
CREATE TABLE IF NOT EXISTS mobilization_cells (
    id                                BIGSERIAL   PRIMARY KEY,
    cell_id                           BIGINT      NOT NULL,
    run_id                            BIGINT      NOT NULL,
    tick_formed                       BIGINT      NOT NULL,
    region_key                        TEXT        NOT NULL,
    formation_risk                    REAL        NOT NULL CHECK (formation_risk >= 0.0 AND formation_risk <= 1.0),
    formation_mobilization            REAL        NOT NULL CHECK (formation_mobilization >= 0.0 AND formation_mobilization <= 1.0),
    v_market                          REAL        NOT NULL,
    v_state                           REAL        NOT NULL,
    v_liberty                         REAL        NOT NULL,
    v_equality                        REAL        NOT NULL,
    v_security                        REAL        NOT NULL,
    v_tradition                       REAL        NOT NULL,
    cell_ideology_distance_from_state REAL        NOT NULL,
    active                            BOOLEAN     NOT NULL DEFAULT TRUE,
    tick_deactivated                  BIGINT,
    created_at                        BIGINT      NOT NULL,
    UNIQUE (run_id, cell_id)
);

CREATE INDEX idx_cells_run_region
    ON mobilization_cells (run_id, region_key, active);
```

---

## 10. Coercion-Cohesion Tradeoff

### 10.1 Mathematical Model

Coercion produces a short-term compliance effect and a long-term cohesion cost. The net function is non-linear:

**Short-term compliance gain:**

```
compliance_gain(c, t) = ρ_coerce · coercion(r, t) · (1 - capture(r, t))
```

Where `ρ_coerce = 0.35`. High shadow capture reduces compliance return (coercion is diverted/corrupted).

**Long-term cohesion cost:**

Coercion contributes to cohesion decay (coefficient `α_coerce = 0.14`). This cost is deferred and cumulative. Additionally, coercion raises the tyranny index, which accelerates legitimacy decay, which further reduces cohesion reinforcement:

```
coercion_tyranny_feedback = coercion(r, t) · w3  (w3 = 0.20 in tyranny formula)
legitimacy_decay_from_tyranny = δ_coerce · coercion_overreach(r, t)
```

**Inflection Point:**

The inflection point occurs when the marginal compliance gain equals the marginal cohesion cost. Solving for the coercion level at which the net benefit is zero:

```
ρ_coerce · (1 - capture) = α_coerce + δ_coerce · coercion · ∂legitimacy/∂coercion
```

For default coefficients with `capture = 0` and `∂legitimacy/∂coercion = δ_coerce = 0.18`:

```
0.35 = 0.14 + 0.18 · coercion*
coercion* = (0.35 - 0.14) / 0.18 ≈ 0.78 / tick
```

At `coercion > coercion*`, increasing coercion yields negative net effect on stability. This is the Coercion Inflection Point. Above it, the model predicts accelerating instability even with continued compliance surface improvements.

### 10.2 Measurement Tyranny Coupling

Metric-to-rights coupling (the Goodhart Pressure term `w2`) activates when welfare or compliance metrics are used to gate access to basic rights (food, housing, medical). When active:

```
goodhart_pressure(r, t) = baseline_pressure + rights_gating_score(r, t) · goodhart_amplifier
```

Where `goodhart_amplifier = 1.4`. This non-linearly increases the tyranny index and feeds back into insurgency propensity through the `γ_capture` and `γ_legit` terms.

### 10.3 Scarcity-Amplification of Coercion

Under resource scarcity, coercion effectiveness per unit declines faster:

```
effective_coercion(r, t) = coercion(r, t) · (1 - scarcity_amplification(r, t) · 0.40)
```

Where `scarcity_amplification` captures the dynamic that coercive control under scarcity is more expensive per unit of compliance and more fragile. This is the `w5` term in the tyranny index.

---

## 11. Diffusion Ordering (Determinism)

### 11.1 Node Traversal Order

The ideology diffusion step iterates over nodes in lexicographic order of `node_id`. This is enforced by storing all node maps as `BTreeMap<String, IdeologyField>` (never `HashMap`). The iteration order is guaranteed by the BTreeMap specification.

```rust
// CORRECT — deterministic
for (node_id, field) in ideology_fields.iter() { ... }

// FORBIDDEN — non-deterministic
let mut fields: HashMap<String, IdeologyField> = ...;
for (node_id, field) in fields.iter() { ... }  // BANNED
```

### 11.2 Edge Processing Order

Edges are stored as `BTreeMap<(String, String), IdeologyEdge>` keyed by `(source_id, target_id)`. Incoming edges for a target node are collected by scanning all edges matching `target_id` in a single ordered pass.

Alternatively (for performance): adjacency list stored as `BTreeMap<String, Vec<IdeologyEdge>>` where `Vec<IdeologyEdge>` is sorted by `source_id` at insertion time. Sorting must be applied at any insert point to maintain the invariant.

### 11.3 Fixed-Step Integration

Cohesion and health dynamics use Euler integration with a fixed step size of 1 tick. No sub-tick integration is used. The order of operations within a tick is fixed and declared in `tick.rs` (Section 7 above). Partial updates mid-tick are not observable from outside the module.

### 11.4 Spatial Diffusion Order

Regional cohesion spatial diffusion iterates over regions in lexicographic `region_key` order. For each region, neighbor regions are sorted lexicographically. The diffusion delta computed in tick `t` is applied simultaneously after all deltas are computed — not iteratively (i.e., the update uses `prev_state` values for all neighbor reads, not the partially-updated `next_state`).

```rust
// CORRECT: collect all deltas first, then apply
let deltas: BTreeMap<String, FpUnit> = compute_all_deltas(&prev_cohesion);
for (region_key, delta) in deltas.iter() {
    next_cohesion.get_mut(region_key).unwrap().aggregate += *delta;
}

// FORBIDDEN: apply delta before reading neighbors
// (violates determinism: result depends on iteration order)
```

---

## 12. Conservation Invariants

### 12.1 Declared Invariants

**I-SOC-1: Cohesion Boundedness**
For all `(r, c, t)`: `cohesion(r, c, t) ∈ [0.0, 1.0]`. Enforced by `FpUnit::clamp()` after every update.

**I-SOC-2: Polarization Boundedness**
For all `(r, t)`: `polarization(r, t) ∈ [0.0, 1.0]`. Derived from clamped inter-cohort variance.

**I-INS-1: Insurgency Risk Boundedness**
For all `(r, t)`: `insurgency_risk(r, t) ∈ [0.0, 1.0]`. Clamped in propensity calculation.

**I-INS-2: Mobilization Score Boundedness**
For all `(r, t)`: `mobilization_score(r, t) ∈ [0.0, 1.0]`. Amnesty campaigns clamp to zero from below; natural ceiling is 1.0.

**I-IDE-1: Ideology Axis Boundedness**
For all `(node, axis, t)`: `ideology_axis ∈ [-1.0, 1.0]`. Enforced by `IdeologyAxis::from_f64` clamp on every write.

**I-IDE-2: Diffusion Stability**
Ideology diffusion converges: if all propaganda and intervention inputs are held constant, the vector field converges to a fixed point. This follows from the similarity-weighted update (influence is zero when identical; the update is a contraction).

**I-HLT-1: Population Conservation**
For all `(c, t)`: `Healthy + Strained + Disabled + Deceased = total_population(c)`. No simulation tick may change total population count (births/deaths require explicit lifecycle events from CIV-0103).

**I-HLT-2: Health Burden Boundedness**
For all `(c, t)`: `health_burden(c, t) ∈ [0.0, 1.0]`.

**I-HLT-3: Welfare Coverage Floor**
For all `(r, t)`: `welfare_coverage(r, t) ≥ welfare_floor_policy(r, t)` unless institutional capacity is insufficient (in which case a `welfare_floor_breach.v1` event is emitted and the deficit is logged).

**I-LEG-1: Legitimacy Boundedness**
For all `(r, t)`: `legitimacy(r, t) ∈ [0.0, 1.0]`.

**I-TYR-1: Tyranny Index Boundedness**
For all `(r, t)`: `tyranny_index(r, t) ∈ [0.0, 1.0]` (logistic sigmoid guarantees this).

### 12.2 Property Tests

```rust
// crates/social/src/tests/invariants.rs

use proptest::prelude::*;
use crate::types::*;

proptest! {
    #[test]
    fn prop_cohesion_always_bounded(
        initial in 0.0f64..=1.0,
        stress in 0.0f64..=1.0,
        coercion in 0.0f64..=1.0,
        welfare in 0.0f64..=1.0,
    ) {
        let c = run_cohesion_update(initial, stress, coercion, welfare);
        prop_assert!(c >= 0.0 && c <= 1.0, "cohesion out of bounds: {}", c);
    }

    #[test]
    fn prop_insurgency_risk_bounded(
        stress in 0.0f64..=1.0,
        coercion in 0.0f64..=1.0,
        capture in 0.0f64..=1.0,
        polarization in 0.0f64..=1.0,
        cohesion in 0.0f64..=1.0,
        welfare_gap in 0.0f64..=1.0,
        legitimacy in 0.0f64..=1.0,
        deterrence in 0.0f64..=1.0,
    ) {
        let risk = compute_insurgency_risk(
            stress, coercion, capture, polarization, cohesion, welfare_gap, legitimacy, deterrence,
            &InsurgencyParams::default()
        );
        prop_assert!(risk >= 0.0 && risk <= 1.0, "risk out of bounds: {}", risk);
    }

    #[test]
    fn prop_ideology_axis_bounded(axis in -1.0f64..=1.0, delta in -0.5f64..=0.5) {
        let updated = (axis + delta).clamp(-1.0, 1.0);
        prop_assert!(updated >= -1.0 && updated <= 1.0);
    }

    #[test]
    fn prop_health_population_conserved(
        healthy in 0i64..100_000,
        strained in 0i64..100_000,
        disabled in 0i64..100_000,
    ) {
        let total = healthy + strained + disabled;
        let (h2, s2, d2, x2) = run_compartmental_step(healthy, strained, disabled, 0, 0.5, 0.5);
        prop_assert_eq!(h2 + s2 + d2 + x2, total, "population not conserved");
    }

    #[test]
    fn prop_diffusion_converges(seed in 0u64..u64::MAX) {
        // Run 1000 ticks with no propaganda or interventions;
        // assert that ideology vector changes converge toward zero.
        let (final_delta_norm, _final_state) = run_isolated_diffusion(seed, 1000);
        prop_assert!(final_delta_norm < 1e-4, "diffusion did not converge: {}", final_delta_norm);
    }
}
```

---

## 13. Failure Modes

### FM-SOC-01: Cohesion Collapse
**Trigger:** Simultaneous high coercion + high shadow capture + welfare gap > 0.5.
**Behavior:** Cohesion decays below 0.15 within 30 ticks. Once polarization enters self-reinforcing regime, recovery requires a sustained welfare + legitimacy intervention combo.
**Detection:** Alert when `cohesion < 0.15` for any region across 3 consecutive ticks.
**Not mitigated by:** Amnesty alone (does not address material drivers). Propaganda alone (information integrity intervention cannot substitute for material welfare delivery).

### FM-SOC-02: Echo Chamber Lock-In
**Trigger:** Ideology diffusion graph becomes strongly clustered with low inter-cluster edge weights after 50+ ticks of high polarization.
**Behavior:** Each cluster's ideology vector drifts monotonically away from others. Ideological distance between clusters exceeds 0.7. Diffusion effectively ceases across cluster boundaries.
**Detection:** Monitor `echo_chamber_index > 0.80` in `PolarizationMetrics`.
**Mitigation:** Cross-cluster information integrity programs (targeting edges between clusters). Civic education programs in high-polarization cohorts.

### FM-HLT-01: Health System Saturation
**Trigger:** `surge_capacity < demand` for > 10 consecutive ticks with `health_burden > 0.75`.
**Behavior:** Transition rate λ_DX (Disabled → Deceased) spikes. Irreversible population loss begins. Welfare floor enforcement becomes impossible to meet (delivery capacity collapse).
**Detection:** Alert when `surge_capacity_deficit > 0.30` and `health_burden > 0.70`.
**Mitigation requires:** Surge capacity intervention AND welfare floor adjustment simultaneously. Neither alone is sufficient once saturation threshold is crossed.

### FM-INS-01: Mobilization Cascade
**Trigger:** `mobilization_score` exceeds threshold in ≥ 3 adjacent regions within 5 ticks.
**Behavior:** Cell formation probability becomes near-certain in all three regions. Active cell count growth feeds `γ_cell` term in subsequent ticks, creating self-sustaining insurgency.
**Detection:** Monitor cluster of threshold-crossing events within spatial and temporal window.
**Mitigation:** De-escalation (amnesty) + immediate legitimacy interventions. Coercion escalation is counterproductive (above inflection point).

### FM-INS-02: Legitimacy Death Spiral
**Trigger:** Legitimacy < 0.20 combined with coercion > 0.70.
**Behavior:** Every coercion action increases tyranny index, which decays legitimacy further, which increases insurgency risk, which triggers more coercion. Model enters a positive feedback loop.
**Detection:** Alert when `legitimacy < 0.25 AND coercion > 0.60` sustained for 5+ ticks.
**Mitigation:** Requires simultaneous: reduce coercion to below inflection point, deploy welfare floor + surge capacity, initiate amnesty campaign.

### FM-IDE-01: Propaganda Saturation
**Trigger:** Multiple propaganda injections target the same node at maximum intensity simultaneously.
**Behavior:** `propaganda_intensity` values stack additively before the `max_propaganda_shift` cap is applied per axis. If total injections exceed cap, excess is silently dropped (logged as `ideology.propaganda_capped.v1` internal diagnostic).
**Invariant:** Per-axis shift cannot exceed `max_propaganda_shift = 0.12` regardless of number of concurrent propaganda sources. This is enforced in `propaganda.rs`.

### FM-DET-01: Rng Seed Mismatch
**Trigger:** Cell formation uses `ChaCha20Rng` seeded from canonical run seed. If caller injects a different rng instance, replay diverges.
**Behavior:** Cell formation events fire at different ticks. Determinism guarantee broken.
**Detection:** Replay divergence check: re-run with same seed and assert identical `mobilization_cells` table.
**Prevention:** The `social_tick` function signature accepts `&mut ChaCha20Rng`; callers are responsible for passing the canonical rng. No internal rng creation is permitted.

---

## 14. Acceptance Test Suite

All test functions are in `crates/social/src/tests/`. Each test references the FR it validates.

```rust
// crates/social/src/tests/mod.rs
mod invariants;   // property tests (Section 12.2)
mod determinism;
mod cohesion;
mod ideology;
mod health;
mod insurgency;
mod interventions;
mod integration;

// ============================================================
// crates/social/src/tests/determinism.rs
// FR-SOC-DET-001: Identical inputs produce identical outputs
// ============================================================

/// FR-SOC-DET-001
#[test]
fn test_social_tick_is_deterministic() {
    // Run two instances of social_tick with the same seed and params.
    // Assert byte-for-byte identical output states.
    let params = SocialTickParams::default_test();
    let initial = SocialSnapshot::test_fixture_alpha();
    let seed = 0xDEADBEEF_CAFEBABE_u64;

    let mut rng1 = ChaCha20Rng::seed_from_u64(seed);
    let mut rng2 = ChaCha20Rng::seed_from_u64(seed);

    let state1 = social_tick(&initial, &params, &mut rng1);
    let state2 = social_tick(&initial, &params, &mut rng2);

    assert_eq!(state1, state2, "social_tick must be deterministic");
}

/// FR-SOC-DET-002: 100-tick replay produces identical trajectory
#[test]
fn test_100_tick_replay_identical() {
    let params = SocialTickParams::default_test();
    let initial = SocialSnapshot::test_fixture_alpha();
    let seed = 42_u64;

    let trace1 = run_social_for_n_ticks(&initial, &params, seed, 100);
    let trace2 = run_social_for_n_ticks(&initial, &params, seed, 100);

    assert_eq!(trace1, trace2, "100-tick replay must be identical");
}

// ============================================================
// crates/social/src/tests/cohesion.rs
// ============================================================

/// FR-SOC-COH-001: Cohesion decay under max coercion
#[test]
fn test_cohesion_decays_under_max_coercion() {
    let mut state = SocialSnapshot::with_cohesion(0.80);
    let params = SocialTickParams { coercion_index: 1.0, ..Default::default() };
    let initial_cohesion = state.region_cohesion["region_a"].aggregate.as_f64();

    advance_n_ticks(&mut state, &params, 10);

    let final_cohesion = state.region_cohesion["region_a"].aggregate.as_f64();
    assert!(final_cohesion < initial_cohesion, "cohesion must decay under coercion");
}

/// FR-SOC-COH-002: Cohesion reinforced by welfare floor increase
#[test]
fn test_cohesion_reinforced_by_welfare_floor() {
    let mut state = SocialSnapshot::with_cohesion(0.40);
    let params = SocialTickParams { welfare_coverage: 0.95, ..Default::default() };
    let initial = state.region_cohesion["region_a"].aggregate.as_f64();

    advance_n_ticks(&mut state, &params, 10);

    let final_cohesion = state.region_cohesion["region_a"].aggregate.as_f64();
    assert!(final_cohesion > initial, "cohesion must increase under high welfare");
}

/// FR-SOC-COH-003: Polarization feedback accelerates cohesion decay
#[test]
fn test_polarization_feedback_accelerates_decay() {
    let params_hi_polar = SocialTickParams { force_polarization: 0.9, ..Default::default() };
    let params_lo_polar = SocialTickParams { force_polarization: 0.1, ..Default::default() };

    let decay_hi = measure_cohesion_decay(&params_hi_polar, 20);
    let decay_lo = measure_cohesion_decay(&params_lo_polar, 20);

    assert!(decay_hi > decay_lo, "high polarization must accelerate cohesion decay");
}

/// FR-SOC-COH-004: Spatial diffusion propagates across adjacent regions
#[test]
fn test_spatial_diffusion_propagates() {
    let mut state = SocialSnapshot::with_region_cohesions(&[("region_a", 0.9), ("region_b", 0.1)]);
    // After diffusion, region_b should increase, region_a should decrease slightly
    advance_n_ticks(&mut state, &SocialTickParams::spatial_only(), 5);

    let a = state.region_cohesion["region_a"].aggregate.as_f64();
    let b = state.region_cohesion["region_b"].aggregate.as_f64();
    assert!(b > 0.1, "region_b cohesion must increase via diffusion");
    assert!(a < 0.9, "region_a cohesion must decrease via diffusion");
}

// ============================================================
// crates/social/src/tests/ideology.rs
// ============================================================

/// FR-SOC-IDE-001: Ideology diffusion step moves target toward source
#[test]
fn test_diffusion_moves_target_toward_source() {
    let source = IdeologyVector { market: IdeologyAxis::POS_ONE, ..IdeologyVector::neutral() };
    let target = IdeologyVector { market: IdeologyAxis::NEG_ONE, ..IdeologyVector::neutral() };
    let edge = IdeologyEdge::test_strong("source", "target");

    let updated = run_single_diffusion_step(&source, &target, &edge, 0.0);

    assert!(updated.market.as_f64() > target.market.as_f64(),
        "target market axis must move toward source");
}

/// FR-SOC-IDE-002: Information integrity damping reduces diffusion magnitude
#[test]
fn test_integrity_damping_reduces_diffusion() {
    let (updated_no_damp, updated_damped) = run_diffusion_with_and_without_damping(0.80);

    let delta_no_damp = (updated_no_damp.market.as_f64() - (-1.0)).abs();
    let delta_damped  = (updated_damped.market.as_f64()  - (-1.0)).abs();

    assert!(delta_damped < delta_no_damp, "damping must reduce ideology shift magnitude");
}

/// FR-SOC-IDE-003: Propaganda injection bounded by max_propaganda_shift per axis
#[test]
fn test_propaganda_shift_bounded() {
    let target = IdeologyVector::neutral();
    let propaganda = IdeologyVector { market: IdeologyAxis::POS_ONE, ..IdeologyVector::neutral() };
    let intensity = 1.0;

    let updated = apply_propaganda(&target, &propaganda, intensity);
    let shift = (updated.market.as_f64() - target.market.as_f64()).abs();
    assert!(shift <= 0.12 + 1e-9, "propaganda shift must not exceed max_propaganda_shift");
}

/// FR-SOC-IDE-004: Ideological distance metric is bounded [0,1]
#[test]
fn test_ideological_distance_bounded() {
    let a = IdeologyVector { market: IdeologyAxis::POS_ONE, state: IdeologyAxis::POS_ONE,
        liberty: IdeologyAxis::POS_ONE, equality: IdeologyAxis::POS_ONE,
        security: IdeologyAxis::POS_ONE, tradition: IdeologyAxis::POS_ONE };
    let b = IdeologyVector { market: IdeologyAxis::NEG_ONE, state: IdeologyAxis::NEG_ONE,
        liberty: IdeologyAxis::NEG_ONE, equality: IdeologyAxis::NEG_ONE,
        security: IdeologyAxis::NEG_ONE, tradition: IdeologyAxis::NEG_ONE };

    let d = a.l2_distance(&b) / 4.899; // normalize
    assert!(d >= 0.0 && d <= 1.0, "distance out of bounds: {}", d);
    assert!((d - 1.0).abs() < 1e-4, "max distance must equal 1.0");
}

// ============================================================
// crates/social/src/tests/health.rs
// ============================================================

/// FR-SOC-HLT-001: Population is conserved across compartmental transitions
#[test]
fn test_population_conserved() {
    let initial = HealthWelfareState::test_fixture_1000_population();
    let total = initial.population_healthy + initial.population_strained
              + initial.population_disabled + initial.population_deceased;

    let updated = run_compartmental_step(&initial, &HealthParams::default());
    let after = updated.population_healthy + updated.population_strained
              + updated.population_disabled + updated.population_deceased;

    assert_eq!(total, after, "population total must be conserved");
}

/// FR-SOC-HLT-002: Health burden increases under high material stress
#[test]
fn test_burden_increases_under_stress() {
    let initial = HealthWelfareState::test_fixture_zero_burden();
    let params = HealthParams { material_stress: 0.9, welfare_coverage: 0.1, ..Default::default() };

    let updated = run_burden_update(&initial, &params);
    assert!(updated.health_burden.as_f64() > 0.0, "burden must increase under high stress");
}

/// FR-SOC-HLT-003: Surge capacity intervention reduces burden accumulation rate
#[test]
fn test_surge_capacity_reduces_burden_rate() {
    let baseline_rate = measure_burden_accumulation_rate(SurgeCapacityLevel::Base);
    let surge_rate    = measure_burden_accumulation_rate(SurgeCapacityLevel::Max);
    assert!(surge_rate < baseline_rate, "surge capacity must reduce burden accumulation");
}

/// FR-SOC-HLT-004: Welfare floor breach emitted when capacity < floor
#[test]
fn test_welfare_floor_breach_emitted() {
    let params = SocialTickParams {
        welfare_floor_policy: 0.90,
        institutional_capacity: 0.30,
        ..Default::default()
    };
    let events = run_tick_capture_events(&params);
    let breach = events.iter().any(|e| e.event_type == "welfare_floor_breach.v1");
    assert!(breach, "welfare floor breach event must be emitted when capacity < floor");
}

// ============================================================
// crates/social/src/tests/insurgency.rs
// ============================================================

/// FR-SOC-INS-001: Insurgency risk increases under max coercion + max stress
#[test]
fn test_risk_increases_under_max_drivers() {
    let initial_risk = 0.20;
    let params = InsurgencyParams {
        material_stress: 1.0,
        coercion: 1.0,
        capture: 1.0,
        ideology_polarization: 1.0,
        ..Default::default()
    };
    let risk = compute_insurgency_risk_from_params(initial_risk, &params);
    assert!(risk > initial_risk, "risk must increase under max drivers");
    assert!(risk <= 1.0, "risk must not exceed 1.0");
}

/// FR-SOC-INS-002: Coercion inflection — above inflection point, marginal return is negative
#[test]
fn test_coercion_inflection_point() {
    // Below inflection: increasing coercion increases compliance
    let compliance_below = measure_net_compliance_effect(0.50);
    // Above inflection: increasing coercion decreases net stability
    let compliance_above = measure_net_compliance_effect(0.90);
    assert!(compliance_below > 0.0, "below inflection coercion should net positive");
    assert!(compliance_above < compliance_below, "above inflection: net benefit must fall");
}

/// FR-SOC-INS-003: Cell formation fires when mobilization crosses threshold
#[test]
fn test_cell_formation_fires_at_threshold() {
    let mut state = SocialSnapshot::with_mobilization(0.64);
    let params = SocialTickParams { recruit_rate_boost: 0.10, ..Default::default() };
    let mut rng = ChaCha20Rng::seed_from_u64(1);

    let events = advance_tick_capture_events(&mut state, &params, &mut rng);
    let cell_formed = events.iter().any(|e| e.event_type == "insurgency.cell_formed.v1");

    // With mobilization just above threshold, cell formation must fire with seeded rng
    assert!(cell_formed, "cell_formed event must fire when mobilization > threshold");
}

/// FR-SOC-INS-004: Amnesty campaign reduces mobilization score and risk
#[test]
fn test_amnesty_reduces_mobilization_and_risk() {
    let mut state = SocialSnapshot::with_mobilization(0.80);
    let amnesty = AmnestyCampaign { amnesty_strength: 0.50, ..Default::default() };

    let before_risk = state.insurgency["region_a"].risk.as_f64();
    let before_mob  = state.insurgency["region_a"].mobilization_score.as_f64();

    apply_amnesty(&mut state, &amnesty);

    let after_risk = state.insurgency["region_a"].risk.as_f64();
    let after_mob  = state.insurgency["region_a"].mobilization_score.as_f64();

    assert!(after_mob < before_mob, "amnesty must reduce mobilization");
    assert!(after_risk < before_risk, "amnesty must reduce risk");
    assert!(after_mob >= 0.0, "mobilization must not go negative");
}

/// FR-SOC-INS-005: Non-linear risk jump near mobilization threshold
#[test]
fn test_nonlinear_risk_jump_near_threshold() {
    let risk_at_0_60 = compute_risk_at_mobilization(0.60);
    let risk_at_0_65 = compute_risk_at_mobilization(0.65);
    let risk_at_0_75 = compute_risk_at_mobilization(0.75);

    let slope_before = (risk_at_0_65 - risk_at_0_60) / 0.05;
    let slope_after  = (risk_at_0_75 - risk_at_0_65) / 0.10;

    assert!(slope_after > slope_before * 1.5,
        "risk slope must be steeper above threshold than below");
}

// ============================================================
// crates/social/src/tests/interventions.rs
// ============================================================

/// FR-SOC-INT-001: Each intervention toggles output in declared direction
#[test]
fn test_welfare_floor_raises_coverage() {
    let before = measure_welfare_coverage(0.30);
    let after  = measure_welfare_coverage_with_intervention(WelfareFloorAdjustment { new_floor: 0.70 });
    assert!(after > before, "welfare floor intervention must raise coverage");
}

/// FR-SOC-INT-002: Information integrity program reduces ideology diffusion rate
#[test]
fn test_info_integrity_reduces_diffusion_rate() {
    let rate_without = measure_diffusion_magnitude_after_n_ticks(10, None);
    let rate_with    = measure_diffusion_magnitude_after_n_ticks(10, Some(info_program(0.60)));
    assert!(rate_with < rate_without, "info integrity must reduce diffusion rate");
}

/// FR-SOC-INT-003: Intervention effects are event-emitting
#[test]
fn test_interventions_emit_events() {
    let events = apply_all_interventions_and_capture();
    let welfare_event = events.iter().any(|e| e.event_type == "health.welfare_updated.v1");
    let social_event  = events.iter().any(|e| e.event_type == "social.state_updated.v1");
    let insurg_event  = events.iter().any(|e| e.event_type == "insurgency.risk_updated.v1");
    assert!(welfare_event && social_event && insurg_event,
        "all intervention types must emit their canonical events");
}

/// FR-SOC-INT-004: Expired interventions have no effect
#[test]
fn test_expired_intervention_has_no_effect() {
    let mut state = SocialSnapshot::default_test();
    let program = InformationIntegrityProgram { duration_ticks: 5, ..Default::default() };
    apply_program_and_expire(&mut state, &program, 6);

    let damping = state.ideology_fields["node_a"].integrity_damping.as_f64();
    assert!((damping - 0.0).abs() < 1e-6, "expired intervention must leave no damping");
}

// ============================================================
// crates/social/src/tests/integration.rs
// ============================================================

/// FR-SOC-INTG-001: Social module outputs couple correctly to insurgency in CIV-0105
#[test]
fn test_social_outputs_propagate_to_insurgency_coupling() {
    // Simulate: run social module for 20 ticks with rising risk.
    // Assert: outbound insurgency_risk field matches what CIV-0105 would receive.
    let trace = run_social_for_n_ticks(&SocialSnapshot::test_fixture_alpha(), &SocialTickParams::high_stress(), 99, 20);
    let final_risk = trace.last().unwrap().insurgency["region_a"].risk.as_f64();
    assert!(final_risk > 0.60, "high-stress 20-tick run must produce risk > 0.60 for coupling test");
}

/// FR-SOC-INTG-002: Coercion index from CIV-0105 raises insurgency risk
#[test]
fn test_external_coercion_raises_risk() {
    let low_coerce  = run_with_coercion_index(0.10, 10);
    let high_coerce = run_with_coercion_index(0.90, 10);
    assert!(high_coerce > low_coerce, "higher external coercion must produce higher risk");
}

/// FR-SOC-INTG-003: Citizen lifecycle dissenting stage raises recruit susceptibility
#[test]
fn test_dissenting_cohort_raises_recruitment() {
    let base_recruit   = measure_recruitment_rate(CitizenStage::Active);
    let dissent_recruit = measure_recruitment_rate(CitizenStage::Dissenting);
    assert!(dissent_recruit > base_recruit,
        "dissenting cohort must have higher recruitment susceptibility");
}
```

---

## 15. CIV Sim Integration Notes (Detailed)

### 15.1 Tick Phase Placement

The social module's `social_tick` function is invoked from the engine's Phase 3 (Deterministic Transition). The engine maintains a `SocialSnapshot` in the canonical world state alongside all other module snapshots.

Phase execution order within Phase 3 is fixed:

1. Institution state transitions (CIV-0103)
2. Citizen lifecycle transitions (CIV-0103)
3. Shadow flow resolution (CIV-0105)
4. **Social tick** (this module — CIV-0106)
5. Economic tick (CIV-0102)
6. Conflict state machine (CIV-0105)

The social tick reads `enforcement_intensity` and `shadow_capture_score` from the CIV-0105 output produced in step 3. It writes `insurgency_risk`, `cohort_stress_score`, and `social_legitimacy_modifier` which are consumed by step 6.

### 15.2 State Handoff Protocol

The engine passes cross-module coupling fields via the `WorldStateView` struct:

```rust
// crates/engine/src/world_state.rs (excerpt — add these fields)

pub struct WorldStateView {
    // ... existing fields ...

    // From CIV-0105 → CIV-0106
    pub enforcement_intensity:   BTreeMap<String, FpUnit>,  // keyed by region_key
    pub shadow_capture_score:    BTreeMap<String, FpUnit>,  // keyed by region_key

    // From CIV-0103 → CIV-0106
    pub citizen_lifecycle:       BTreeMap<(String, String), CitizenStage>,  // (region, cohort)
    pub institution_capacity:    BTreeMap<String, FpUnit>,  // keyed by institution_id

    // From CIV-0106 → CIV-0105 (written by social_tick)
    pub insurgency_risk:         BTreeMap<String, FpUnit>,
    pub social_legitimacy_mod:   BTreeMap<String, FpUnit>,
    pub cohort_stress_score:     BTreeMap<(String, String), FpUnit>,
}
```

### 15.3 Event Emission in Phase 5

During Phase 5 (Metrics), the engine calls `social::events::emit_all(&snapshot, &prev_snapshot, &mut event_sink)`. This function:

1. Iterates over all regions in lexicographic order (BTreeMap)
2. For each region, emits `social.state_updated.v1` for each cohort that changed
3. Emits `health.welfare_updated.v1` for each cohort whose health state changed
4. Emits `insurgency.risk_updated.v1` for each region whose risk changed
5. Emits `ideology.diffusion_stepped.v1` for each node whose vector changed by > `ideology_emit_threshold` (default: 0.005)
6. Emits `insurgency.cell_formed.v1` for all cells formed this tick

Events that are identical to the previous tick (no change) are NOT emitted to reduce write volume. The exception is canonical runs (tagged `canonical: true`) which emit all events every tick for full replay fidelity.

### 15.4 Policy Bundle Consumption

Policy bundles from Phase 2 are unpacked into intervention records before Phase 3. The `PolicyBundle` from `crates/policy` must include:

```rust
pub struct SocialPolicyBundle {
    pub welfare_floor_adjustments: Vec<WelfareFloorAdjustment>,
    pub info_integrity_programs:   Vec<InformationIntegrityProgram>,
    pub surge_capacity_interventions: Vec<SurgeCapacityIntervention>,
    pub amnesty_campaigns:         Vec<AmnestyCampaign>,
    pub propaganda_injections:     Vec<PropagandaInjection>,
}
```

These are consumed by `social::interventions::apply_all` at the start of each social tick.

### 15.5 Persistence Protocol

After Phase 5, the engine writes all emitted events and the complete `SocialSnapshot` to the persistence layer (`crates/io`). The following tables receive writes:

- `social_state` — all cohort social snapshots
- `health_welfare_state` — all cohort health snapshots
- `insurgency_state` — all region insurgency snapshots
- `ideology_state` — all changed node ideology snapshots (or all if canonical run)
- `mobilization_cells` — new cell records only; existing cells updated for `active` status changes

All writes are append-only. The run_id + tick combination is guaranteed unique by the engine's tick counter (monotonic i64, no SystemTime).

### 15.6 Canonical Run Determinism Guarantee

For a canonical run `(run_id, seed)`:
1. All BTreeMap orderings are lexicographic (stable across platforms)
2. ChaCha20Rng is initialized once per run from `seed`; the same rng instance is threaded through all ticks in order
3. No floating-point operations are used in any invariant-critical path; all computations use i64 fixed-point Q16.16
4. No `HashMap`, `HashSet`, or any structure with unspecified iteration order is used in state that affects outputs
5. No `SystemTime`, `Instant`, or wall-clock value is ever read within the tick pipeline
6. Spatial diffusion uses the two-pass algorithm (collect deltas, then apply) to prevent order dependency

A replay validation test (`test_100_tick_replay_identical`) is required to pass before any merge to main that touches `crates/social`.

---

## Appendix A: Default Coefficient Summary

| Parameter                | Default | Range     | Override Via                |
|--------------------------|---------|-----------|-----------------------------|
| `α_stress`               | 0.18    | [0, 1]    | `CohesionParams`            |
| `α_coerce`               | 0.14    | [0, 1]    | `CohesionParams`            |
| `α_capture`              | 0.20    | [0, 1]    | `CohesionParams`            |
| `α_polar`                | 0.10    | [0, 1]    | `CohesionParams`            |
| `α_health`               | 0.08    | [0, 1]    | `CohesionParams`            |
| `β_welfare`              | 0.15    | [0, 1]    | `CohesionParams`            |
| `β_service`              | 0.12    | [0, 1]    | `CohesionParams`            |
| `β_civic`                | 0.08    | [0, 1]    | `CohesionParams`            |
| `β_legitimacy`           | 0.10    | [0, 1]    | `CohesionParams`            |
| `κ_spatial`              | 0.05    | [0, 0.5]  | `CohesionParams`            |
| `η` (diffusion rate)     | 0.04    | [0, 0.25] | `IdeologyParams`            |
| `max_propaganda_shift`   | 0.12    | [0, 0.5]  | `IdeologyParams`            |
| `max_integrity_damping`  | 0.80    | [0, 1]    | `IdeologyParams`            |
| `κ_health`               | 0.03    | [0, 0.25] | `HealthParams`              |
| `base_capacity`          | 0.30    | [0, 1]    | `HealthParams`              |
| `surge_multiplier`       | 0.60    | [0, 2]    | `HealthParams`              |
| `γ_stress`               | 0.22    | [0, 1]    | `InsurgencyParams`          |
| `γ_coerce`               | 0.18    | [0, 1]    | `InsurgencyParams`          |
| `γ_capture`              | 0.15    | [0, 1]    | `InsurgencyParams`          |
| `γ_polar`                | 0.12    | [0, 1]    | `InsurgencyParams`          |
| `γ_cohesion`             | 0.10    | [0, 1]    | `InsurgencyParams`          |
| `γ_welfare`              | 0.14    | [0, 1]    | `InsurgencyParams`          |
| `γ_legit`                | 0.16    | [0, 1]    | `InsurgencyParams`          |
| `γ_deter`                | 0.08    | [0, 1]    | `InsurgencyParams`          |
| `mobilization_threshold` | 0.65    | [0, 1]    | `InsurgencyParams`          |
| `threshold_sensitivity`  | 0.10    | [0.01, 1] | `InsurgencyParams`          |
| `ρ_coerce` (compliance)  | 0.35    | [0, 1]    | `InsurgencyParams`          |
| `coercion_inflection`    | 0.78    | derived   | (computed, not overridable) |
| `δ_service`              | 0.10    | [0, 1]    | `InsurgencyParams`          |
| `δ_corrupt`              | 0.14    | [0, 1]    | `InsurgencyParams`          |
| `δ_coerce`               | 0.18    | [0, 1]    | `InsurgencyParams`          |
| `δ_harm`                 | 0.20    | [0, 1]    | `InsurgencyParams`          |
| `δ_amnesty`              | 0.12    | [0, 1]    | `InsurgencyParams`          |
| `goodhart_amplifier`     | 1.40    | [1, 3]    | `TyrannyParams`             |

---

## Appendix B: FR Traceability

| FR ID              | Description                                                   | Section  | Test                                              |
|--------------------|---------------------------------------------------------------|----------|---------------------------------------------------|
| FR-SOC-DET-001     | Identical inputs produce identical outputs                    | 11       | `test_social_tick_is_deterministic`               |
| FR-SOC-DET-002     | 100-tick replay is identical                                  | 11       | `test_100_tick_replay_identical`                  |
| FR-SOC-COH-001     | Cohesion decays under coercion                                | 1.2      | `test_cohesion_decays_under_max_coercion`         |
| FR-SOC-COH-002     | Cohesion reinforced by welfare                                | 1.3      | `test_cohesion_reinforced_by_welfare_floor`       |
| FR-SOC-COH-003     | Polarization feedback accelerates decay                       | 1.6      | `test_polarization_feedback_accelerates_decay`    |
| FR-SOC-COH-004     | Spatial diffusion propagates between regions                  | 1.5      | `test_spatial_diffusion_propagates`               |
| FR-SOC-IDE-001     | Diffusion moves target toward source                          | 2.3      | `test_diffusion_moves_target_toward_source`       |
| FR-SOC-IDE-002     | Integrity damping reduces diffusion magnitude                 | 2.6      | `test_integrity_damping_reduces_diffusion`        |
| FR-SOC-IDE-003     | Propaganda shift bounded by max                               | 2.4      | `test_propaganda_shift_bounded`                   |
| FR-SOC-IDE-004     | Ideological distance bounded [0,1]                            | 2.5      | `test_ideological_distance_bounded`               |
| FR-SOC-HLT-001     | Population conserved across compartmental transitions         | 3.1      | `test_population_conserved`                       |
| FR-SOC-HLT-002     | Health burden increases under stress                          | 3.2      | `test_burden_increases_under_stress`              |
| FR-SOC-HLT-003     | Surge capacity reduces burden rate                            | 3.4      | `test_surge_capacity_reduces_burden_rate`         |
| FR-SOC-HLT-004     | Welfare floor breach emitted when unmet                       | 5.1      | `test_welfare_floor_breach_emitted`               |
| FR-SOC-INS-001     | Risk increases under max drivers                              | 4.1      | `test_risk_increases_under_max_drivers`           |
| FR-SOC-INS-002     | Coercion inflection point holds                               | 10.1     | `test_coercion_inflection_point`                  |
| FR-SOC-INS-003     | Cell formation fires at threshold                             | 4.3      | `test_cell_formation_fires_at_threshold`          |
| FR-SOC-INS-004     | Amnesty reduces mobilization and risk                         | 5.4      | `test_amnesty_reduces_mobilization_and_risk`      |
| FR-SOC-INS-005     | Non-linear risk jump near threshold                           | 4.3      | `test_nonlinear_risk_jump_near_threshold`         |
| FR-SOC-INT-001     | Welfare floor raises coverage                                 | 5.1      | `test_welfare_floor_raises_coverage`              |
| FR-SOC-INT-002     | Info integrity reduces diffusion rate                         | 5.2      | `test_info_integrity_reduces_diffusion_rate`      |
| FR-SOC-INT-003     | Interventions emit events                                     | 5.5      | `test_interventions_emit_events`                  |
| FR-SOC-INT-004     | Expired interventions have no effect                          | 5.5      | `test_expired_intervention_has_no_effect`         |
| FR-SOC-INTG-001    | Social outputs propagate to CIV-0105 coupling                 | 15.1     | `test_social_outputs_propagate_to_insurgency_coupling` |
| FR-SOC-INTG-002    | External coercion raises insurgency risk                      | 15.2     | `test_external_coercion_raises_risk`              |
| FR-SOC-INTG-003    | Dissenting cohort raises recruitment                          | 4.2      | `test_dissenting_cohort_raises_recruitment`       |

---

## 16. Multi-Faction Political Economy



### 16.1 Faction Taxonomy

The political economy module defines six canonical ideological factions. Each faction is a named cluster in the eight-dimensional extended ideology space (see Section 17 for the extended vector). Faction identity is stable; what changes is faction size (membership) and the internal policy preference weights held by that faction's membership.

| Faction ID | Name | Primary Ideology Axes | Signature Preference |
|------------|------|-----------------------|----------------------|
| `technocratic` | Technocrats | high `v_technocracy`, high `v_state`, low `v_tradition` | Expert-led central planning, metrics-driven governance |
| `socialist` | Socialists | high `v_equality`, high `v_state`, low `v_market` | Redistribution, welfare maximalism, collective ownership |
| `libertarian` | Libertarians | high `v_market`, high `v_liberty`, low `v_state` | Market primacy, civil liberties, minimal state |
| `nationalist` | Nationalists | high `v_tradition`, high `v_militarism`, low `v_cosmopolitan` | Sovereignty maximalism, cultural continuity, border control |
| `religious` | Religious Conservatives | high `v_tradition`, high `v_theocracy`, low `v_liberty` | Moral order, religious institution primacy, social cohesion via faith |
| `green` | Greens | high `v_ecology`, high `v_equality`, low `v_militarism` | Environmental constraint, anti-growth, redistributive welfare |

Each faction is persistent across the simulation run. Factions do not dissolve — they shrink to near-zero membership when their preferred conditions are absent.

### 16.2 Faction State Struct

```rust
// crates/social/src/faction/types.rs

use std::collections::BTreeMap;
use crate::types::{FpUnit, FP_SCALE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactionId {
    Technocratic,
    Socialist,
    Libertarian,
    Nationalist,
    Religious,
    Green,
}

/// Snapshot of one faction in one region at one tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionState {
    pub run_id:          u64,
    pub tick:            u64,
    pub region_key:      String,
    pub faction_id:      FactionId,
    /// Fraction of region population affiliated with this faction [0, FP_SCALE].
    pub membership:      FpUnit,
    /// Strength of policy pressure exerted this tick [0, FP_SCALE].
    pub policy_pressure: FpUnit,
    /// Internal cohesion of faction membership [0, FP_SCALE].
    /// Low internal cohesion reduces effective pressure.
    pub internal_cohesion: FpUnit,
    pub created_at:      u64,
}

/// Per-faction policy preference vector.
/// Maps each policy axis to a preferred direction and intensity.
/// Stored as i16 signed fixed-point (Q0.15): 32767 = maximum preference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionPreference {
    pub faction_id:          FactionId,
    /// Redistribution preference: +1 = maximum redistribution, -1 = none.
    pub redistribution:      i16,
    /// State authority preference: +1 = maximum state, -1 = minimal state.
    pub authority:           i16,
    /// Tradition preference: +1 = maximum traditional, -1 = progressive.
    pub tradition:           i16,
    /// Cosmopolitan preference: +1 = open borders / global, -1 = isolationist.
    pub cosmopolitan:        i16,
    /// Ecology preference: +1 = zero-growth constraint, -1 = extraction first.
    pub ecology:             i16,
    /// Theocracy preference: +1 = religious law primacy, -1 = secular.
    pub theocracy:           i16,
    /// Technocracy preference: +1 = expert rule, -1 = populist/democratic.
    pub technocracy:         i16,
    /// Militarism preference: +1 = military spending priority, -1 = demilitarize.
    pub militarism:          i16,
}

impl FactionPreference {
    /// Default preference vectors calibrated to each faction identity.
    pub fn defaults() -> BTreeMap<FactionId, FactionPreference> {
        let mut m = BTreeMap::new();
        m.insert(FactionId::Technocratic, FactionPreference {
            faction_id: FactionId::Technocratic,
            redistribution: 8000, authority: 24000, tradition: -8000,
            cosmopolitan: 16000, ecology: 8000, theocracy: -16000,
            technocracy: 32767, militarism: 0,
        });
        m.insert(FactionId::Socialist, FactionPreference {
            faction_id: FactionId::Socialist,
            redistribution: 32767, authority: 20000, tradition: -12000,
            cosmopolitan: 16000, ecology: 16000, theocracy: -8000,
            technocracy: 8000, militarism: -8000,
        });
        m.insert(FactionId::Libertarian, FactionPreference {
            faction_id: FactionId::Libertarian,
            redistribution: -32767, authority: -32767, tradition: 0,
            cosmopolitan: 24000, ecology: 0, theocracy: -24000,
            technocracy: 0, militarism: -16000,
        });
        m.insert(FactionId::Nationalist, FactionPreference {
            faction_id: FactionId::Nationalist,
            redistribution: 0, authority: 20000, tradition: 32767,
            cosmopolitan: -32767, ecology: -8000, theocracy: 8000,
            technocracy: -8000, militarism: 28000,
        });
        m.insert(FactionId::Religious, FactionPreference {
            faction_id: FactionId::Religious,
            redistribution: 8000, authority: 16000, tradition: 32767,
            cosmopolitan: -16000, ecology: 0, theocracy: 32767,
            technocracy: -16000, militarism: 8000,
        });
        m.insert(FactionId::Green, FactionPreference {
            faction_id: FactionId::Green,
            redistribution: 24000, authority: 8000, tradition: -8000,
            cosmopolitan: 24000, ecology: 32767, theocracy: -8000,
            technocracy: 8000, militarism: -24000,
        });
        m
    }
}

/// Aggregate policy pressure across all factions in a region.
/// Represents the net vector of demand on the policy DSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyPressure {
    pub region_key:      String,
    pub tick:            u64,
    pub redistribution:  i64,   // weighted sum, signed, FP_SCALE units
    pub authority:       i64,
    pub tradition:       i64,
    pub cosmopolitan:    i64,
    pub ecology:         i64,
    pub theocracy:       i64,
    pub technocracy:     i64,
    pub militarism:      i64,
    /// Total effective pressure magnitude [0, FP_SCALE].
    pub total_magnitude: FpUnit,
}
```

### 16.3 Faction Membership Dynamics

Citizen ideology vectors (Section 17) determine faction affiliation via nearest-centroid assignment. Each tick:

1. Compute ideological distance from each citizen cohort's mean ideology vector to each faction's centroid vector.
2. Assign cohort members to the faction with minimum distance.
3. Update `FactionState.membership` as the fraction of regional population assigned to each faction.

Membership shift rate is bounded by a maximum drift coefficient `ζ_faction = 0.08` per tick to prevent instantaneous swings:

```
membership(f, r, t+1) = membership(f, r, t)
    + clamp(raw_membership(f, r, t) - membership(f, r, t), -ζ_faction, +ζ_faction)
```

This creates inertia: faction sizes change gradually even when underlying ideology shifts sharply.

**Faction internal cohesion** decays when member ideology variance is high:

```
internal_cohesion(f, r, t+1) = 1 - ideology_variance_within_faction(f, r, t)
```

Where `ideology_variance_within_faction` is the mean pairwise L2 distance between cohort ideology vectors assigned to faction `f`, normalized to [0, 1].

### 16.4 Preference Aggregation Function

The effective policy pressure exerted by all factions in a region is a membership-weighted sum of faction preference vectors:

```
PolicyPressure(axis, r, t) = Σ_{f ∈ factions} membership(f, r, t) · internal_cohesion(f, r, t) · preference(f, axis)
```

Each axis of `PolicyPressure` is an i64 in Q16.16 fixed-point. The `total_magnitude` is:

```
total_magnitude(r, t) = ||PolicyPressure(r, t)||_2 / max_possible_magnitude
```

Where `max_possible_magnitude` is the L2 norm when all axes are at maximum (all factions unified at peak preference, which equals `sqrt(8) · FP_SCALE`). Normalized to [0, 1] and stored as `FpUnit`.

```rust
// crates/social/src/faction/aggregation.rs

use crate::faction::types::{FactionId, FactionPreference, FactionState, PolicyPressure};
use crate::types::{FpUnit, FP_SCALE};
use std::collections::BTreeMap;

/// Aggregate faction preferences into net policy pressure for a region.
/// Input: faction states (membership, cohesion) and preference vectors.
/// Output: PolicyPressure with signed weighted sums per axis.
pub fn aggregate_faction_pressure(
    faction_states: &BTreeMap<FactionId, FactionState>,
    preferences:    &BTreeMap<FactionId, FactionPreference>,
) -> PolicyPressure {
    let mut redistribution: i64 = 0;
    let mut authority:       i64 = 0;
    let mut tradition:       i64 = 0;
    let mut cosmopolitan:    i64 = 0;
    let mut ecology:         i64 = 0;
    let mut theocracy:       i64 = 0;
    let mut technocracy:     i64 = 0;
    let mut militarism:      i64 = 0;

    let region_key = faction_states
        .values()
        .next()
        .map(|s| s.region_key.clone())
        .unwrap_or_default();
    let tick = faction_states
        .values()
        .next()
        .map(|s| s.tick)
        .unwrap_or(0);

    // BTreeMap iteration order is lexicographic by FactionId — deterministic.
    for (fid, state) in faction_states.iter() {
        let pref = match preferences.get(fid) {
            Some(p) => p,
            None => continue,
        };
        // weight = membership * internal_cohesion (both FP_SCALE-scaled i64)
        let weight: i64 = (state.membership.0 * state.internal_cohesion.0) / FP_SCALE;

        redistribution += weight * pref.redistribution as i64 / 32767;
        authority      += weight * pref.authority      as i64 / 32767;
        tradition      += weight * pref.tradition      as i64 / 32767;
        cosmopolitan   += weight * pref.cosmopolitan   as i64 / 32767;
        ecology        += weight * pref.ecology        as i64 / 32767;
        theocracy      += weight * pref.theocracy      as i64 / 32767;
        technocracy    += weight * pref.technocracy    as i64 / 32767;
        militarism     += weight * pref.militarism     as i64 / 32767;
    }

    // Compute L2 magnitude and normalize.
    let sum_sq: i64 = [redistribution, authority, tradition, cosmopolitan,
                        ecology, theocracy, technocracy, militarism]
        .iter()
        .map(|v| (v / FP_SCALE).pow(2))
        .sum();
    let magnitude_raw = (sum_sq as f64).sqrt();
    // max possible: sqrt(8) * 1.0 in normalized units = 2.828
    let total_magnitude = FpUnit::from_f64((magnitude_raw / (FP_SCALE as f64 * 2.828)).clamp(0.0, 1.0));

    PolicyPressure {
        region_key, tick,
        redistribution, authority, tradition, cosmopolitan,
        ecology, theocracy, technocracy, militarism,
        total_magnitude,
    }
}
```

### 16.5 Coalition Government Mechanics

When institutions include a democratic governance type (from CIV-0103), factions must form a governing coalition. Coalition mechanics determine which bundle of factions has sufficient combined membership to govern, and what policy compromise results.

**Minimum Winning Coalition (MWC):**

A coalition C is winning if:

```
Σ_{f ∈ C} membership(f, r, t) ≥ coalition_threshold   (default: 0.50)
```

Among all winning coalitions, the MWC is the smallest by total membership exceeding the threshold. This is the coalition that minimizes internal heterogeneity (ideological distance between members).

**Coalition formation algorithm** (deterministic, BTreeMap-ordered):

1. Sort factions by membership descending (ties broken by FactionId lexicographic order).
2. Greedily add factions from largest to smallest until coalition_threshold is met.
3. If no single faction exceeds threshold, the two closest factions (by ideology distance) form the nucleus, then grow.
4. Coalition is stable if mean pairwise ideology distance among members < `coalition_distance_threshold = 0.45`.
5. If distance exceeds threshold, the coalition is unstable; instability adds `+0.08` to insurgency propensity per region per tick.

**Policy compromise formula:**

The governing coalition's effective policy vector is the membership-weighted centroid of member faction preferences:

```
coalition_policy(axis) = Σ_{f ∈ C} membership(f) · preference(f, axis) / Σ_{f ∈ C} membership(f)
```

This centroid becomes the `effective_policy_vector` that constrains the `policy.evaluate()` function in the Policy DSL.

```rust
// crates/social/src/faction/coalition.rs

use crate::faction::types::{FactionId, FactionPreference, FactionState};
use crate::types::FpUnit;
use std::collections::BTreeMap;

pub const COALITION_THRESHOLD: f64 = 0.50;
pub const COALITION_DISTANCE_THRESHOLD: f64 = 0.45;

#[derive(Debug, Clone)]
pub struct Coalition {
    pub members:              Vec<FactionId>,   // sorted by FactionId for determinism
    pub combined_membership:  FpUnit,
    pub is_stable:            bool,
    pub mean_ideology_distance: FpUnit,
    /// Weighted centroid of member preferences per axis.
    pub effective_policy:     EffectivePolicyVector,
}

#[derive(Debug, Clone)]
pub struct EffectivePolicyVector {
    pub redistribution: i64,
    pub authority:      i64,
    pub tradition:      i64,
    pub cosmopolitan:   i64,
    pub ecology:        i64,
    pub theocracy:      i64,
    pub technocracy:    i64,
    pub militarism:     i64,
}

/// Compute the minimum winning coalition given faction states and preferences.
/// Deterministic: uses BTreeMap ordering, no randomness.
pub fn compute_mwc(
    faction_states: &BTreeMap<FactionId, FactionState>,
    preferences:    &BTreeMap<FactionId, FactionPreference>,
    ideology_distance_fn: impl Fn(FactionId, FactionId) -> f64,
) -> Coalition {
    // Step 1: collect factions sorted by membership desc, ties by FactionId asc.
    let mut ranked: Vec<(FactionId, f64)> = faction_states
        .iter()
        .map(|(fid, s)| (*fid, s.membership.as_f64()))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()
        .then(a.0.cmp(&b.0)));

    // Step 2: greedy accumulation.
    let mut members: Vec<FactionId> = Vec::new();
    let mut total_mem = 0.0f64;
    for (fid, mem) in &ranked {
        members.push(*fid);
        total_mem += mem;
        if total_mem >= COALITION_THRESHOLD {
            break;
        }
    }
    members.sort(); // stable deterministic order

    // Step 3: compute stability and effective policy.
    let mean_dist = compute_mean_ideology_distance(&members, &ideology_distance_fn);
    let is_stable = mean_dist < COALITION_DISTANCE_THRESHOLD;

    let eff = compute_effective_policy(&members, faction_states, preferences);
    let combined = FpUnit::from_f64(total_mem.clamp(0.0, 1.0));
    let mean_dist_fp = FpUnit::from_f64(mean_dist.clamp(0.0, 1.0));

    Coalition {
        members,
        combined_membership: combined,
        is_stable,
        mean_ideology_distance: mean_dist_fp,
        effective_policy: eff,
    }
}

fn compute_mean_ideology_distance(
    members: &[FactionId],
    dist_fn: &impl Fn(FactionId, FactionId) -> f64,
) -> f64 {
    if members.len() < 2 { return 0.0; }
    let mut total = 0.0f64;
    let mut count = 0u64;
    for i in 0..members.len() {
        for j in (i+1)..members.len() {
            total += dist_fn(members[i], members[j]);
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { total / count as f64 }
}

fn compute_effective_policy(
    members:        &[FactionId],
    states:         &BTreeMap<FactionId, FactionState>,
    preferences:    &BTreeMap<FactionId, FactionPreference>,
) -> EffectivePolicyVector {
    use crate::types::FP_SCALE;
    let (mut w_sum, mut r, mut a, mut t, mut co, mut ec, mut th, mut tc, mut mi)
        = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
    for fid in members.iter() {
        let mem = states.get(fid).map(|s| s.membership.0).unwrap_or(0);
        let coh = states.get(fid).map(|s| s.internal_cohesion.0).unwrap_or(FP_SCALE);
        let w = mem * coh / FP_SCALE;
        w_sum += w;
        if let Some(pref) = preferences.get(fid) {
            r  += w * pref.redistribution as i64 / 32767;
            a  += w * pref.authority      as i64 / 32767;
            t  += w * pref.tradition      as i64 / 32767;
            co += w * pref.cosmopolitan   as i64 / 32767;
            ec += w * pref.ecology        as i64 / 32767;
            th += w * pref.theocracy      as i64 / 32767;
            tc += w * pref.technocracy    as i64 / 32767;
            mi += w * pref.militarism     as i64 / 32767;
        }
    }
    let d = if w_sum == 0 { 1 } else { w_sum };
    EffectivePolicyVector {
        redistribution: r / d, authority: a / d, tradition: t / d,
        cosmopolitan: co / d, ecology: ec / d, theocracy: th / d,
        technocracy: tc / d, militarism: mi / d,
    }
}
```

### 16.6 Election Model

When governance institutions include `ElectoralSystem` (CIV-0103), faction membership determines electoral outcomes per tick-cycle equal to `election_period_ticks` (default: 48 ticks = one simulated year at weekly ticks).

**Electoral outcome rule:**

1. At tick `t` where `t mod election_period_ticks == 0`, compute coalition using the MWC algorithm.
2. The coalition's `effective_policy_vector` replaces the current `active_policy_vector` in the `PolicyBundle`.
3. The transition emits `faction.coalition.formed.v1` event.
4. Non-coalition factions enter opposition; opposition pressure = their combined membership × their mean preference deviation from the coalition policy.
5. Opposition pressure feeds back into insurgency propensity (`γ_capture` channel) and ideology diffusion (opposition factions become propaganda sources targeting coalition ideology).

**Electoral legitimacy:**

```
electoral_legitimacy(r, t) = coalition_combined_membership(r, t)
    · coalition_stability_factor(r, t)
    · (1 - abstention_rate(r, t))
```

Where `abstention_rate = 1 - Σ_{f} membership(f) · faction_turnout(f)` and `faction_turnout` defaults to `0.75` for all factions (overridable via `ElectionParams`).

Electoral legitimacy directly adds to the `δ_service` term in legitimacy dynamics (Section 4.4) when the most recent election was fair (no `shadow_capture_score > 0.5`).

### 16.7 Module Layout Extension

The faction subsystem extends the social crate:

```
crates/social/src/faction/
├── mod.rs          # Public API: FactionState, FactionPreference, Coalition
├── types.rs        # FactionId enum, FactionState, FactionPreference, PolicyPressure
├── membership.rs   # Membership dynamics: nearest-centroid assignment, drift cap
├── aggregation.rs  # aggregate_faction_pressure()
├── coalition.rs    # compute_mwc(), Coalition, EffectivePolicyVector
└── election.rs     # Electoral cycle, legitimacy computation, policy bundle update
```

---

## 17. Compartmental Social Dynamics (Civic Engagement Model)

### 17.1 Conceptual Mapping

Inspired by SIR epidemic compartmental models (from Plague Inc. / academic social contagion literature), civic engagement is modeled as population movement through three states:

```
Engaged (E) → Alienated (A) → Rebellious (R)
```

With recovery paths:

```
Rebellious (R) → Alienated (A) → Engaged (E)
```

This is analogous to SIR with:
- `Engaged` = Susceptible (integrated, participatory)
- `Alienated` = Infected (withdrawn, distrustful)
- `Rebellious` = Recovered in SIR — but here it is an active harmful state, not recovery

The reversal path (R → A → E) represents de-escalation programs, welfare improvement, and legitimacy restoration.

### 17.2 State Definitions

```
E(r, t)  — fraction of regional population that is civically engaged   ∈ [0, 1]
A(r, t)  — fraction that is alienated (withdrawn, distrustful)         ∈ [0, 1]
R(r, t)  — fraction that is rebellious (active opposition, mobilized)  ∈ [0, 1]

E + A + R = 1.0  at all times (population conservation)
```

Internally stored as i64 FP_SCALE fractions. Population counts derived by multiplying by `region_population`.

### 17.3 Transition Rates

**Forward transitions (deterioration):**

```
λ_EA(r, t) = α_stress · material_stress(r, t)
           + α_coerce · coercion(r, t)
           + α_welfare_gap · welfare_gap(r, t)
           + α_capture · capture(r, t)
           - α_service · service_delivery(r, t)
```

Rate at which Engaged population moves to Alienated. Bounded ∈ [0, 0.15] per tick (biological ceiling: no more than 15% of engaged population can alienate in one tick).

```
λ_AR(r, t) = β_social · A(r, t) · contact_rate(r, t)
           + β_cell   · active_cell_count(r, t) · cell_influence_weight
           + β_polar  · polarization(r, t)
           - β_legit  · legitimacy(r, t)
```

Rate at which Alienated population mobilizes to Rebellious. The `A(r, t)` term creates the SIR-analog: alienation spreads through social contact proportional to current alienated fraction (like infection spreading proportional to infected fraction). Bounded ∈ [0, 0.10] per tick.

**Reverse transitions (recovery):**

```
μ_AE(r, t) = γ_welfare · welfare_coverage(r, t)
           + γ_service · service_delivery(r, t)
           + γ_legit   · legitimacy(r, t)
           - γ_polar   · polarization(r, t)
```

Rate at which Alienated population re-engages. Bounded ∈ [0, 0.12] per tick.

```
μ_RA(r, t) = δ_amnesty · amnesty_applied(r, t)
           + δ_welfare  · welfare_coverage(r, t) · 0.40
           + δ_legit    · legitimacy(r, t)        · 0.30
```

Rate at which Rebellious population de-escalates to Alienated. Bounded ∈ [0, 0.06] per tick. Recovery from rebellion is deliberately slow: the `0.40` and `0.30` dampening factors reflect that welfare improvements do not immediately convert rebels.

### 17.4 Discrete Update Rule

```
ΔE = -λ_EA · E + μ_AE · A
ΔA = +λ_EA · E - μ_AE · A - λ_AR · A + μ_RA · R
ΔR = +λ_AR · A - μ_RA · R

E(t+1) = clamp(E(t) + ΔE, 0, 1)
A(t+1) = clamp(A(t) + ΔA, 0, 1)
R(t+1) = clamp(R(t) + ΔR, 0, 1)
```

After clamping, renormalize so E + A + R = 1.0:

```
total = E(t+1) + A(t+1) + R(t+1)
E(t+1) /= total; A(t+1) /= total; R(t+1) /= total
```

This ensures exact conservation despite floating-point edge cases. In fixed-point: distribute residual rounding error to the largest compartment.

### 17.5 R₀ Analog for Civic Rebellion

The basic reproduction number analog R₀ for civic rebellion is the expected number of additional alienated persons one rebellious cell contact mobilizes before de-escalation:

```
R₀_civic(r, t) = (β_social · contact_rate(r, t) · A(r, t)) / μ_RA(r, t)
```

When `R₀_civic > 1.0`, the Rebellious compartment is self-sustaining (rebellion grows without external input). When `R₀_civic < 1.0`, rebellion decays without intervention.

The R₀ threshold `R₀_civic = 1.0` is the **civic rebellion threshold**. The system monitors this per region per tick. Crossing from below to above emits `civic.rebellion_threshold_crossed.v1` (diagnostic event).

Default regime analysis:
- High welfare (`welfare_coverage = 0.80`), low polarization: `R₀_civic ≈ 0.4` (sub-critical, rebellion decays)
- Low welfare (`welfare_coverage = 0.20`), high cells (`active_cell_count ≥ 3`): `R₀_civic ≈ 1.8` (super-critical, rebellion self-sustaining)
- Amnesty campaign at max strength brings `μ_RA` up enough to suppress `R₀_civic` below 1.0 within 8–12 ticks

### 17.6 Spatial Spread

Contact rate `contact_rate(r, t)` is modulated by regional cohesion:

```
contact_rate(r, t) = base_contact_rate · (1 - cohesion(r, t) · κ_cohesion_contact)
                   + Σ_{r' ∈ neighbors(r)} κ_spatial_contact · A(r', t)
```

Where:
- `base_contact_rate = 0.25` — baseline within-region contact
- `κ_cohesion_contact = 0.40` — high cohesion suppresses alienation spread (social trust reduces contagion)
- `κ_spatial_contact = 0.05` — cross-region contagion (alienation spreads to neighbors)

Spatial spread is computed in a two-pass algorithm identical to Section 11.4: collect all cross-region contributions first using `prev_state` values, then apply simultaneously.

### 17.7 Rust Structs

```rust
// crates/social/src/civic/types.rs

use crate::types::{FpUnit, FP_SCALE};

/// Civic engagement compartmental state for one region at one tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CivicCompartments {
    pub run_id:     u64,
    pub tick:       u64,
    pub region_key: String,
    /// Engaged fraction [0, FP_SCALE]. E + A + R = FP_SCALE.
    pub engaged:    FpUnit,
    /// Alienated fraction [0, FP_SCALE].
    pub alienated:  FpUnit,
    /// Rebellious fraction [0, FP_SCALE].
    pub rebellious: FpUnit,
    /// Transition rates computed this tick (for event emission).
    pub lambda_ea:  FpUnit,
    pub lambda_ar:  FpUnit,
    pub mu_ae:      FpUnit,
    pub mu_ra:      FpUnit,
    /// R₀ analog [0, FP_SCALE·4] (can exceed 1.0; stored without upper clamp).
    pub r0_civic:   i64,
    pub created_at: u64,
}

impl CivicCompartments {
    /// Verify E + A + R = FP_SCALE (within rounding tolerance of ±2 units).
    pub fn assert_conserved(&self) {
        let total = self.engaged.0 + self.alienated.0 + self.rebellious.0;
        assert!(
            (total - FP_SCALE).abs() <= 2,
            "CivicCompartments conservation violated: total={}", total
        );
    }
}

/// Parameters controlling civic compartmental dynamics.
#[derive(Debug, Clone)]
pub struct CivicParams {
    pub alpha_stress:      FpUnit,   // default: 0.12
    pub alpha_coerce:      FpUnit,   // default: 0.10
    pub alpha_welfare_gap: FpUnit,   // default: 0.08
    pub alpha_capture:     FpUnit,   // default: 0.07
    pub alpha_service:     FpUnit,   // default: 0.09
    pub beta_social:       FpUnit,   // default: 0.18
    pub beta_cell:         FpUnit,   // default: 0.06
    pub beta_polar:        FpUnit,   // default: 0.10
    pub beta_legit:        FpUnit,   // default: 0.12
    pub gamma_welfare:     FpUnit,   // default: 0.14
    pub gamma_service:     FpUnit,   // default: 0.10
    pub gamma_legit:       FpUnit,   // default: 0.08
    pub gamma_polar:       FpUnit,   // default: 0.06
    pub delta_amnesty:     FpUnit,   // default: 0.10
    pub delta_welfare:     FpUnit,   // default: 0.05 (× 0.40 dampening)
    pub delta_legit:       FpUnit,   // default: 0.05 (× 0.30 dampening)
    pub base_contact_rate: FpUnit,   // default: 0.25
    pub kappa_cohesion_contact:  FpUnit,  // default: 0.40
    pub kappa_spatial_contact:   FpUnit,  // default: 0.05
    pub max_lambda_ea:     FpUnit,   // default: 0.15
    pub max_lambda_ar:     FpUnit,   // default: 0.10
    pub max_mu_ae:         FpUnit,   // default: 0.12
    pub max_mu_ra:         FpUnit,   // default: 0.06
}

impl Default for CivicParams {
    fn default() -> Self {
        CivicParams {
            alpha_stress:      FpUnit::from_f64(0.12),
            alpha_coerce:      FpUnit::from_f64(0.10),
            alpha_welfare_gap: FpUnit::from_f64(0.08),
            alpha_capture:     FpUnit::from_f64(0.07),
            alpha_service:     FpUnit::from_f64(0.09),
            beta_social:       FpUnit::from_f64(0.18),
            beta_cell:         FpUnit::from_f64(0.06),
            beta_polar:        FpUnit::from_f64(0.10),
            beta_legit:        FpUnit::from_f64(0.12),
            gamma_welfare:     FpUnit::from_f64(0.14),
            gamma_service:     FpUnit::from_f64(0.10),
            gamma_legit:       FpUnit::from_f64(0.08),
            gamma_polar:       FpUnit::from_f64(0.06),
            delta_amnesty:     FpUnit::from_f64(0.10),
            delta_welfare:     FpUnit::from_f64(0.05),
            delta_legit:       FpUnit::from_f64(0.05),
            base_contact_rate: FpUnit::from_f64(0.25),
            kappa_cohesion_contact: FpUnit::from_f64(0.40),
            kappa_spatial_contact:  FpUnit::from_f64(0.05),
            max_lambda_ea:     FpUnit::from_f64(0.15),
            max_lambda_ar:     FpUnit::from_f64(0.10),
            max_mu_ae:         FpUnit::from_f64(0.12),
            max_mu_ra:         FpUnit::from_f64(0.06),
        }
    }
}
```

### 17.8 Civic State Transition Step

```rust
// crates/social/src/civic/compartmental.rs

use crate::civic::types::{CivicCompartments, CivicParams};
use crate::types::{FpUnit, FP_SCALE};

/// Single-tick update of civic compartmental state for one region.
/// All inputs are normalized [0,1] as f64 for readability; result stored as FpUnit.
pub fn civic_step(
    prev:           &CivicCompartments,
    material_stress: f64,
    coercion:        f64,
    welfare_gap:     f64,
    capture:         f64,
    service_delivery: f64,
    welfare_coverage: f64,
    legitimacy:       f64,
    polarization:     f64,
    amnesty_applied:  f64,
    active_cells:     u32,
    neighbor_alienated: f64,  // mean alienated fraction of neighbor regions
    params:          &CivicParams,
) -> CivicCompartments {
    let e = prev.engaged.as_f64();
    let a = prev.alienated.as_f64();
    let r = prev.rebellious.as_f64();

    let cell_inf = active_cells as f64 * params.beta_cell.as_f64();

    // Contact rate with spatial spread from neighbors.
    let contact_rate = (params.base_contact_rate.as_f64()
        * (1.0 - (prev.engaged.as_f64() * params.kappa_cohesion_contact.as_f64())))
        + params.kappa_spatial_contact.as_f64() * neighbor_alienated;
    let contact_rate = contact_rate.clamp(0.0, 1.0);

    // Forward transition rates.
    let lambda_ea_raw = params.alpha_stress.as_f64()      * material_stress
        + params.alpha_coerce.as_f64()     * coercion
        + params.alpha_welfare_gap.as_f64()* welfare_gap
        + params.alpha_capture.as_f64()    * capture
        - params.alpha_service.as_f64()    * service_delivery;
    let lambda_ea = lambda_ea_raw.clamp(0.0, params.max_lambda_ea.as_f64());

    let lambda_ar_raw = params.beta_social.as_f64() * a * contact_rate
        + cell_inf
        + params.beta_polar.as_f64() * polarization
        - params.beta_legit.as_f64() * legitimacy;
    let lambda_ar = lambda_ar_raw.clamp(0.0, params.max_lambda_ar.as_f64());

    // Recovery rates.
    let mu_ae_raw = params.gamma_welfare.as_f64() * welfare_coverage
        + params.gamma_service.as_f64() * service_delivery
        + params.gamma_legit.as_f64()   * legitimacy
        - params.gamma_polar.as_f64()   * polarization;
    let mu_ae = mu_ae_raw.clamp(0.0, params.max_mu_ae.as_f64());

    let mu_ra_raw = params.delta_amnesty.as_f64() * amnesty_applied
        + params.delta_welfare.as_f64() * welfare_coverage * 0.40
        + params.delta_legit.as_f64()   * legitimacy        * 0.30;
    let mu_ra = mu_ra_raw.clamp(0.0, params.max_mu_ra.as_f64());

    // Euler step.
    let delta_e = -lambda_ea * e + mu_ae * a;
    let delta_a =  lambda_ea * e - mu_ae * a - lambda_ar * a + mu_ra * r;
    let delta_r =  lambda_ar * a - mu_ra * r;

    let e2 = (e + delta_e).clamp(0.0, 1.0);
    let a2 = (a + delta_a).clamp(0.0, 1.0);
    let r2 = (r + delta_r).clamp(0.0, 1.0);

    // Renormalize for conservation.
    let total = e2 + a2 + r2;
    let (e_n, a_n, r_n) = if total > 0.0 {
        (e2 / total, a2 / total, r2 / total)
    } else {
        (1.0, 0.0, 0.0)
    };

    // R₀ analog (stored as i64 without upper clamp; can exceed FP_SCALE).
    let r0 = if mu_ra > 1e-9 {
        ((params.beta_social.as_f64() * contact_rate * a_n / mu_ra) * FP_SCALE as f64) as i64
    } else {
        i64::MAX / 2
    };

    CivicCompartments {
        run_id:     prev.run_id,
        tick:       prev.tick + 1,
        region_key: prev.region_key.clone(),
        engaged:    FpUnit::from_f64(e_n).clamp(),
        alienated:  FpUnit::from_f64(a_n).clamp(),
        rebellious: FpUnit::from_f64(r_n).clamp(),
        lambda_ea:  FpUnit::from_f64(lambda_ea).clamp(),
        lambda_ar:  FpUnit::from_f64(lambda_ar).clamp(),
        mu_ae:      FpUnit::from_f64(mu_ae).clamp(),
        mu_ra:      FpUnit::from_f64(mu_ra).clamp(),
        r0_civic:   r0,
        created_at: prev.tick + 1,
    }
}
```

---

## 18. Ideology Vector System (Extended — Eight Axes)

### 18.1 Extended Vector Definition

The base ideology vector defined in Section 2.1 uses six axes. The extended system used by the faction layer, hidden network actors, and radicalization attractor adds two additional axes:

```
v_extended = [
    v_market,       // -1 = central plan,        +1 = free market
    v_state,        // -1 = minimal state,        +1 = maximal state
    v_liberty,      // -1 = security priority,    +1 = civil liberty
    v_equality,     // -1 = meritocratic,         +1 = redistributive
    v_security,     // -1 = change tolerance,     +1 = stability preference
    v_tradition,    // -1 = progressive,           +1 = traditional
    v_ecology,      // -1 = extraction priority,   +1 = ecological constraint  [NEW]
    v_cosmopolitan, // -1 = isolationist,          +1 = cosmopolitan/globalist  [NEW]
]
```

All components bounded ∈ [-1, 1], stored as i16 (Q0.15 fixed-point: 32767 = 1.0).

The extended vector is used for:
- Faction centroid computation (Section 16.2)
- Hidden network actor ideology seeding (Section 18.5)
- Radicalization attractor computation (Section 18.4)

The six-axis `IdeologyVector` from Section 2 remains the primary type for diffusion and all existing events. The extended eight-axis type is `ExtendedIdeologyVector` and is only used in the faction and hidden network subsystems.

### 18.2 Distance Metric (Eight-Axis)

The L2 distance over eight axes:

```
d8(v_a, v_b) = ||v_a - v_b||_2
```

Maximum possible distance over eight axes each spanning [-1, 1]:

```
max_distance_8 = sqrt(8 · 4) = sqrt(32) ≈ 5.657
```

Normalized distance:

```
d8_norm(v_a, v_b) = d8(v_a, v_b) / 5.657   ∈ [0, 1]
```

```rust
// crates/social/src/ideology/extended.rs

/// Eight-axis extended ideology vector for faction and hidden network use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedIdeologyVector(pub [i16; 8]);

impl ExtendedIdeologyVector {
    pub const AXES: usize = 8;
    pub const MAX_DISTANCE: f64 = 5.6569; // sqrt(32)

    pub fn as_f64_array(&self) -> [f64; 8] {
        self.0.map(|v| v as f64 / 32767.0)
    }

    /// L2 distance, not normalized.
    pub fn l2_distance(&self, other: &Self) -> f64 {
        let a = self.as_f64_array();
        let b = other.as_f64_array();
        let sum_sq: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();
        sum_sq.sqrt()
    }

    /// Normalized similarity ∈ [0, 1]; 1.0 = identical.
    pub fn similarity(&self, other: &Self) -> f64 {
        1.0 - (self.l2_distance(other) / Self::MAX_DISTANCE).clamp(0.0, 1.0)
    }

    /// Normalized distance ∈ [0, 1]; 1.0 = maximally distant.
    pub fn normalized_distance(&self, other: &Self) -> f64 {
        (self.l2_distance(other) / Self::MAX_DISTANCE).clamp(0.0, 1.0)
    }

    /// Clamp all components to [-32767, 32767].
    pub fn clamped(&self) -> Self {
        ExtendedIdeologyVector(self.0.map(|v| v.clamp(-32767, 32767)))
    }

    /// Weighted mean of two vectors with weight w for self, (1-w) for other.
    pub fn weighted_mean(&self, other: &Self, w: f64) -> Self {
        let w = w.clamp(0.0, 1.0);
        let a = self.as_f64_array();
        let b = other.as_f64_array();
        let result: [i16; 8] = std::array::from_fn(|i| {
            ((w * a[i] + (1.0 - w) * b[i]) * 32767.0).clamp(-32767.0, 32767.0) as i16
        });
        ExtendedIdeologyVector(result)
    }
}
```

### 18.3 Diffusion on Social Graph (Extended)

The same diffusion step from Section 2.3 applies to the extended vector in hidden network nodes. The extended diffusion iterates over all eight axes independently:

```
Δv_t[i] = η_ext · Σ_{s: (s→t) ∈ E} influence(s, t) · (v_s[i] - v_t[i])
```

Where `η_ext = 0.03` (slightly slower than the six-axis diffusion rate of 0.04, because hidden network actors have structural inertia).

Information integrity damping applies identically:

```
Δv_t_damped[i] = Δv_t[i] · (1 - integrity_damping(t))
```

Crucially, the extended diffusion operates on a separate sub-graph: the **hidden network influence graph**. This graph has nodes for shadow actors, media conglomerates, religious institutions, and foreign actors. It does NOT directly overlap with the citizen/region diffusion graph from Section 2.2 — the coupling is one-directional: hidden network node ideology vectors influence (but are not directly influenced by) citizen cohort ideology via `IdeologyEdge` connections that originate from hidden network nodes and target cohort nodes.

### 18.4 Radicalization Attractor

Under certain conditions, ideology vectors do not converge to a mean — they diverge toward extremes. The radicalization attractor is a state where the diffusion update amplifies rather than dampens ideological extremism.

**Attractor activation condition:**

```
radicalization_active(node, t) = (legitimacy(r, t) < legit_threshold)
    AND (polarization(r, t) > polar_threshold)
    AND (welfare_gap(r, t) > welfare_threshold)
```

Default thresholds:
- `legit_threshold = 0.25`
- `polar_threshold = 0.65`
- `welfare_threshold = 0.35`

**When attractor is active**, the diffusion update is modified:

```
Δv_t_attractor[i] = Δv_t[i] + sign(v_t[i]) · radicalization_boost · |v_t[i]|
```

Where `radicalization_boost = 0.02` per tick per axis. This creates a positive feedback: nodes that already lean in a direction are pushed further in that direction.

The attractor effect is bounded: `radicalization_boost` is applied only when `|v_t[i]| > 0.40`, preventing radicalization from generating opinions from neutrality.

**Detection and events:**

When any node's ideology vector norm exceeds `radicalization_threshold = 0.80` (i.e., strongly positioned across most axes), and the vector velocity (change per tick) exceeds `velocity_threshold = 0.04`, emit `ideology.radicalization_detected.v1`.

**Attractor stability analysis:**

In the radicalization-active regime, the fixed points are the extremes of ideology space (each axis at ±1.0). The system converges to the nearest extreme. The rate of convergence is approximately:

```
convergence_ticks ≈ 1 / (η_ext + radicalization_boost) ≈ 1/0.05 = 20 ticks
```

At default parameters, a node enters extreme radicalization within 20 ticks of continuous attractor activation.

### 18.5 Media and Propaganda as Diffusion Modifier

Media control by a hidden network actor modifies the effective diffusion rate for all citizen nodes within the actor's reach:

```
effective_η(node, t) = η_base · (1 + media_control(actor, t) · media_amplify_coeff)
                                · (1 - integrity_damping(node, t))
```

Where `media_amplify_coeff = 0.60`. High media control can nearly double the effective diffusion rate, rapidly shifting citizen ideology toward the media-controlling actor's ideology vector.

The `information_integrity` parameter from Section 2.6 directly counters media amplification. At `integrity_damping = 0.80` (maximum), even full media control is reduced to `effective_η ≈ η_base · (1 + 0.60) · 0.20 = η_base · 0.32` — still above baseline but significantly dampened.

```rust
// crates/social/src/ideology/radicalization.rs

use crate::ideology::extended::ExtendedIdeologyVector;
use crate::types::FpUnit;

pub const RADICALIZATION_BOOST: f64 = 0.02;
pub const RADICALIZATION_AXIS_THRESHOLD: f64 = 0.40;
pub const RADICALIZATION_DETECTION_NORM: f64 = 0.80;
pub const RADICALIZATION_VELOCITY_THRESHOLD: f64 = 0.04;

/// Apply radicalization attractor to ideology delta.
/// Only modifies axes where |v_current| > RADICALIZATION_AXIS_THRESHOLD.
/// Returns the modified delta array.
pub fn apply_radicalization_attractor(
    v_current:  &ExtendedIdeologyVector,
    delta:      &mut [f64; 8],
    is_active:  bool,
) {
    if !is_active { return; }
    let current = v_current.as_f64_array();
    for i in 0..8 {
        if current[i].abs() > RADICALIZATION_AXIS_THRESHOLD {
            let sign = if current[i] > 0.0 { 1.0 } else { -1.0 };
            delta[i] += sign * RADICALIZATION_BOOST * current[i].abs();
        }
    }
}

/// Check whether radicalization is detectable (for event emission).
pub fn is_radicalization_detectable(
    v_current: &ExtendedIdeologyVector,
    v_prev:    &ExtendedIdeologyVector,
) -> bool {
    let current = v_current.as_f64_array();
    let prev    = v_prev.as_f64_array();

    // Compute norm (mean absolute value across axes).
    let norm: f64 = current.iter().map(|v| v.abs()).sum::<f64>() / 8.0;

    // Compute velocity (mean absolute change per tick).
    let velocity: f64 = current.iter().zip(prev.iter())
        .map(|(c, p)| (c - p).abs())
        .sum::<f64>() / 8.0;

    norm > RADICALIZATION_DETECTION_NORM && velocity > RADICALIZATION_VELOCITY_THRESHOLD
}

/// Check all three attractor activation conditions.
pub fn radicalization_attractor_active(
    legitimacy:   f64,
    polarization: f64,
    welfare_gap:  f64,
) -> bool {
    const LEGIT_THRESHOLD:   f64 = 0.25;
    const POLAR_THRESHOLD:   f64 = 0.65;
    const WELFARE_THRESHOLD: f64 = 0.35;
    legitimacy < LEGIT_THRESHOLD
        && polarization > POLAR_THRESHOLD
        && welfare_gap > WELFARE_THRESHOLD
}
```

---

## 19. Public Health System Model (Extended)

### 19.1 Health System Capacity Accounting

The health system capacity is tracked as three interacting components:

```
health_capacity(r, t) = doctor_capacity(r, t)
                      + hospital_capacity(r, t)
                      + supply_chain_capacity(r, t)
```

Each expressed as a fraction [0, 1] of ideal service capacity for the regional population. The aggregate `Q(r, t)` from Section 3.4 is the mean of these three:

```
Q(r, t) = (doctor_capacity + hospital_capacity + supply_chain_capacity) / 3
```

The three sub-capacities degrade and recover independently:

| Sub-capacity | Primary degradation driver | Recovery lever |
|---|---|---|
| `doctor_capacity` | War harm (`δ_harm` coercion), emigration | Medical training programs (+0.04/tick) |
| `hospital_capacity` | Conflict damage, maintenance deficit | Capital investment (surge capacity intervention) |
| `supply_chain_capacity` | Shadow capture (leakage), conflict disruption | Anti-corruption + supply chain investment |

```rust
// crates/social/src/health/capacity.rs

use crate::types::FpUnit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSystemCapacity {
    pub run_id:                  u64,
    pub tick:                    u64,
    pub region_key:              String,
    /// Doctor and medical personnel availability [0, FP_SCALE].
    pub doctor_capacity:         FpUnit,
    /// Hospital bed and facility availability [0, FP_SCALE].
    pub hospital_capacity:       FpUnit,
    /// Medical supply chain integrity [0, FP_SCALE].
    pub supply_chain_capacity:   FpUnit,
    /// Aggregate capacity Q = mean of three sub-capacities [0, FP_SCALE].
    pub aggregate_capacity:      FpUnit,
    /// Utilization rate: demand / capacity. Values > 1.0 indicate saturation.
    /// Stored as i64; 65536 = 1.0; can exceed 65536 when over-capacity.
    pub utilization_rate:        i64,
    pub created_at:              u64,
}

impl HealthSystemCapacity {
    pub fn recompute_aggregate(&mut self) {
        let sum = self.doctor_capacity.0 + self.hospital_capacity.0
                + self.supply_chain_capacity.0;
        self.aggregate_capacity = crate::types::FpUnit(sum / 3).clamp();
    }
}
```

### 19.2 Disease Burden Accumulation (Extended)

Beyond the base burden model in Section 3.2, the extended model adds chronic disease accumulation and long-term health trajectory:

**Chronic burden accumulation:**

```
chronic_burden(c, t) = chronic_burden(c, t-1) · persistence_rate
    + acute_to_chronic_conversion · burden(c, t)
    - chronic_recovery · welfare_coverage · Q(r, t)
```

Where:
- `persistence_rate = 0.95` — chronic burden is slow to clear (5% natural decay per tick)
- `acute_to_chronic_conversion = 0.10` — 10% of acute burden converts to chronic per tick
- `chronic_recovery = 0.05` — maximum chronic recovery rate with full welfare + full capacity

**Life expectancy effect:**

```
life_expectancy_modifier(c, t) = 1.0
    - chronic_burden_weight · chronic_burden(c, t)
    - disability_weight      · (population_disabled / population_total)
    - welfare_gap_weight     · welfare_gap(r, t)
```

Where:
- `chronic_burden_weight = 0.35`
- `disability_weight = 0.25`
- `welfare_gap_weight = 0.20`

The `life_expectancy_modifier` feeds back into CIV-0103 citizen lifecycle: reduced modifier shortens expected productive lifespan of cohort members.

**Labor productivity feedback:**

```
productivity_modifier(c, t) = 1.0
    - 0.20 · (population_strained / population_healthy)
    - 0.55 · (population_disabled / population_healthy)
    - 0.30 · chronic_burden(c, t)
```

This is the outgoing coupling to the economic module (CIV-0102): sick cohorts produce fewer joules per tick. The `productivity_modifier` is written to `WorldStateView.cohort_productivity_modifier` at the end of Phase 3.

```rust
// crates/social/src/health/burden.rs (extended)

use crate::types::{FpUnit, FP_SCALE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedDiseaseBurden {
    pub cohort_id:                String,
    pub region_key:               String,
    pub tick:                     u64,
    /// Acute burden (existing, from Section 3.2) [0, FP_SCALE].
    pub acute_burden:             FpUnit,
    /// Chronic burden (new) [0, FP_SCALE].
    pub chronic_burden:           FpUnit,
    /// Combined burden = acute + chronic · 0.60 (chronic weighted lower) [0, FP_SCALE].
    pub combined_burden:          FpUnit,
    /// Life expectancy modifier [0, FP_SCALE]; 1.0 = no reduction.
    pub life_expectancy_modifier: FpUnit,
    /// Labor productivity modifier [0, FP_SCALE]; 1.0 = full productivity.
    pub productivity_modifier:    FpUnit,
}

impl ExtendedDiseaseBurden {
    pub fn recompute_combined(&mut self) {
        let combined_raw = self.acute_burden.as_f64()
            + self.chronic_burden.as_f64() * 0.60;
        self.combined_burden = FpUnit::from_f64(combined_raw.clamp(0.0, 1.0)).clamp();
    }
}
```

### 19.3 Epidemic Model (Stochastic)

The epidemic model extends the compartmental health system with a stochastic spread layer. Unlike the deterministic burden accumulation, epidemic spread uses ChaCha20Rng (canonical seed) for event sampling.

**Epidemic state per region:**

```
Susceptible (S) → Exposed (E) → Infected (I) → Recovered (Im) or Deceased (X)
```

This is a SEIR model layered on top of the existing H/S/D/X compartments:

| SEIR state | Maps to H/S/D/X |
|---|---|
| Susceptible | Healthy (H) |
| Exposed | Strained (S) — pre-symptomatic |
| Infected | Disabled (D) — symptomatic and incapacitated |
| Recovered | Healthy (H) with immunity flag |
| Deceased | Deceased (X) |

Epidemic spread is triggered by an `epidemic.outbreak_detected.v1` event (either stochastic emergence or cross-region import). Once triggered, the SEIR dynamics layer on top of the existing compartmental transitions for the duration of the epidemic.

**Transmission rate:**

```
β_epidemic(r, t) = base_transmission · population_density_factor(r)
                 · (1 - vaccination_coverage(r, t))
                 · (1 - health_system_barrier(r, t))
```

Where `health_system_barrier = Q(r, t) · 0.40` — health system capacity reduces transmission (contact tracing, isolation support).

**Emergence probability (stochastic):**

Each tick, epidemic emergence is sampled:

```
p_emergence(r, t) = epidemic_seed_rate
                  · health_burden(r, t)
                  · (1 - welfare_coverage(r, t))
                  · environmental_exposure(r, t)
```

Where `epidemic_seed_rate = 0.002` per tick per region (approximately one emergence event per 500 tick-regions). Sample from `ChaCha20Rng`: if `rng.gen::<f64>() < p_emergence`, trigger outbreak.

**Cross-region transmission:**

For each adjacent region pair, if one is in outbreak state:

```
p_cross_import(r1 → r2, t) = cross_transmission_rate · infected_fraction(r1, t)
                            · connectivity(r1, r2)
```

Where `cross_transmission_rate = 0.04` and `connectivity` is a region-pair property (defaults to 0.50 for all adjacent pairs; 0.0 for non-adjacent).

```rust
// crates/social/src/health/epidemic.rs

use crate::types::{FpUnit, FP_SCALE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpidemicStatus {
    Clear,
    Outbreak { tick_started: u64 },
    Contained { tick_ended: u64 },
}

#[derive(Debug, Clone)]
pub struct EpidemicState {
    pub run_id:              u64,
    pub tick:                u64,
    pub region_key:          String,
    pub status:              EpidemicStatus,
    /// SEIR compartment fractions [0, FP_SCALE].
    pub susceptible:         FpUnit,
    pub exposed:             FpUnit,
    pub infected:            FpUnit,
    pub recovered_immune:    FpUnit,
    /// Transmission rate β [0, FP_SCALE].
    pub transmission_rate:   FpUnit,
    /// Effective reproduction number R_e (FP_SCALE = 1.0, unbounded above).
    pub r_effective:         i64,
    /// Duration in ticks if in outbreak.
    pub outbreak_duration:   u32,
    pub created_at:          u64,
}

impl EpidemicState {
    /// Check if epidemic is self-sustaining (R_e > FP_SCALE).
    pub fn is_self_sustaining(&self) -> bool {
        self.r_effective > FP_SCALE
    }
}

/// Parameters for the epidemic model.
#[derive(Debug, Clone)]
pub struct EpidemicParams {
    pub base_transmission:        f64,   // default: 0.08 per tick
    pub epidemic_seed_rate:       f64,   // default: 0.002
    pub cross_transmission_rate:  f64,   // default: 0.04
    pub incubation_ticks:         u32,   // default: 3 (E → I transition lag)
    pub recovery_ticks:           u32,   // default: 8 (I → Im baseline)
    pub case_fatality_rate:       f64,   // default: 0.02 (modulated by Q)
    pub health_barrier_coeff:     f64,   // default: 0.40
    pub vaccination_coverage:     f64,   // default: 0.0 (set by intervention)
}

impl Default for EpidemicParams {
    fn default() -> Self {
        EpidemicParams {
            base_transmission:       0.08,
            epidemic_seed_rate:      0.002,
            cross_transmission_rate: 0.04,
            incubation_ticks:        3,
            recovery_ticks:          8,
            case_fatality_rate:      0.02,
            health_barrier_coeff:    0.40,
            vaccination_coverage:    0.0,
        }
    }
}
```

### 19.4 Surge Capacity: Emergency Expansion

Surge capacity emergency expansion is a special intervention that temporarily doubles health system capacity at a joule cost:

```
Q_emergency(r, t) = min(Q(r, t) · surge_multiplier_emergency, Q_max_emergency)
```

Where:
- `surge_multiplier_emergency = 2.0` — doubles capacity
- `Q_max_emergency = 0.85` — cannot exceed 85% even with emergency surge (structural ceiling)
- Duration: `emergency_surge_duration_ticks = 12` (default)
- Joule cost: `emergency_surge_cost = population(r) · joule_per_person_surge` (flows to CIV-0102 as energy demand)

The effectiveness curve is not linear: surge capacity reduces the `λ_DX` (Disabled → Deceased) transition rate following a diminishing returns curve:

```
λ_DX_effective(r, t) = λ_DX_base · max(0, 1 - Q(r, t) · surge_effectiveness_curve(Q))
```

Where `surge_effectiveness_curve(Q) = 1 - exp(-Q · 3.0)` — a saturating exponential. At `Q = 0.30` (base), effectiveness is `1 - exp(-0.9) ≈ 0.59`. At `Q = 0.85` (emergency max), effectiveness is `1 - exp(-2.55) ≈ 0.92`.

### 19.5 Long-Term Health Trajectory

At the run level, each region accumulates a `health_trajectory_score` that summarizes long-term health trends:

```
health_trajectory_score(r, t) = EMA(combined_burden(r, τ), τ ≤ t, window=24)
```

Where EMA is the exponential moving average over a 24-tick window. This smooths out short-term shocks and captures sustained degradation or improvement.

When `health_trajectory_score > 0.60` for 24 consecutive ticks, emit `health.trajectory_critical.v1` — a trigger for emergency policy review.

---

## 20. Insurgency Cell System (Extended)

### 20.1 Cell Lifecycle

Beyond the simple `MobilizationCell` formation in Section 4.3, the extended cell system tracks full cell lifecycle: formation, growth, operational capability, degradation, and dissolution.

**Lifecycle states:**

```
Nascent → Active → Operational → Degraded → Dissolved
```

| State | Definition | Duration |
|---|---|---|
| Nascent | Cell formed; recruiting; no operational capability | 3–8 ticks |
| Active | Sufficient members to conduct operations | ongoing |
| Operational | Has resources AND safe zones; can execute campaigns | ongoing |
| Degraded | Lost resources or members; capability reduced | ongoing |
| Dissolved | Membership < dissolution_threshold OR dismantled | terminal |

Transitions:

```
Nascent → Active:      member_count ≥ nascent_threshold (default: 15)
Active → Operational:  resource_held ≥ op_resource_threshold
                       AND safe_zone_count ≥ 1
                       AND trained_members ≥ op_trained_threshold
Operational → Degraded: successful counterinsurgency op OR resource drop
Degraded → Dissolved:   member_count < dissolution_threshold (default: 5)
                         OR amnesty campaign absorbs remaining members
Active → Dissolved:     amnesty campaign at high strength (amnesty_strength ≥ 0.70)
```

### 20.2 Cell Growth Dynamics

Per tick, each active cell recruits and loses members:

```
Δmembers(cell, t) = recruit_rate(cell, t) - attrition_rate(cell, t) - detection_loss(cell, t)

recruit_rate(cell, t) = base_recruit · R(r, t) · (1 - ideology_distance_penalty)
    · (1 - saturation_factor)

ideology_distance_penalty = normalized_distance(cell.ideology, regional_mean_ideology) · 0.50

saturation_factor = member_count / max_cell_size   (max_cell_size = 500)

attrition_rate(cell, t) = natural_attrition · member_count
    + counterinsurgency_pressure(r, t) · attrition_multiplier

detection_loss(cell, t) = detection_probability(cell, t) · member_count · detection_capture_rate
```

Where:
- `base_recruit = 0.08` per tick per unit of R(r, t)
- `natural_attrition = 0.02` per tick (desertion, death, capture by chance)
- `attrition_multiplier = 0.15` (counterinsurgency multiplier)
- `detection_capture_rate = 0.30` (fraction of detected members captured/neutralized)

**Resource accumulation:**

```
resource_held(cell, t) = resource_held(cell, t-1)
    + resource_acquisition(cell, t)
    - resource_consumption(cell, t)
    - resource_seized(cell, t)

resource_acquisition = shadow_network_support(r, t) · acquisition_rate
    + taxation_of_population(r, t) · extortion_rate · R(r, t) · member_count
```

Resources are tracked in joule-equivalent units (coupling to CIV-0102 energy accounting).

### 20.3 Cell Network Structure

Cells are nodes in a covert coordination network. The network evolves as cells form, grow, and dissolve.

```rust
// crates/social/src/insurgency/cell_network.rs

use crate::types::{FpUnit, FP_SCALE};
use crate::ideology::extended::ExtendedIdeologyVector;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellLifecycleState {
    Nascent,
    Active,
    Operational,
    Degraded,
    Dissolved,
}

#[derive(Debug, Clone)]
pub struct InsurgencyCell {
    pub cell_id:              u64,
    pub run_id:               u64,
    pub tick_formed:          u64,
    pub region_key:           String,
    pub lifecycle_state:      CellLifecycleState,
    /// Member count (integer; not FpUnit).
    pub member_count:         u32,
    pub trained_members:      u32,
    /// Resources held in joule-equivalent units (fixed-point i64).
    pub resource_held:        i64,
    /// Number of safe zones controlled [0, 5].
    pub safe_zone_count:      u8,
    /// Ideology of the cell (seeds from regional mobilized cohort mean).
    pub ideology:             ExtendedIdeologyVector,
    /// Operational capability score [0, FP_SCALE].
    pub operational_capacity: FpUnit,
    /// Detection risk accumulated [0, FP_SCALE].
    pub detection_risk:       FpUnit,
    pub tick_last_op:         Option<u64>,
    pub tick_dissolved:       Option<u64>,
}

impl InsurgencyCell {
    /// Compute operational capacity from current state.
    /// op_capacity = (resource · trained_fraction · safe_zone_factor) normalized.
    pub fn recompute_operational_capacity(&mut self) {
        if self.member_count == 0 {
            self.operational_capacity = FpUnit::ZERO;
            return;
        }
        let resource_factor = (self.resource_held as f64 / 1_000_000.0).clamp(0.0, 1.0);
        let trained_fraction = self.trained_members as f64 / self.member_count as f64;
        let safe_zone_factor = (self.safe_zone_count as f64 / 3.0).clamp(0.0, 1.0);
        let raw = resource_factor * trained_fraction * safe_zone_factor;
        self.operational_capacity = FpUnit::from_f64(raw.clamp(0.0, 1.0)).clamp();
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.lifecycle_state, CellLifecycleState::Dissolved)
    }
}

/// Coordination edge between two cells.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellCoordinationEdge {
    pub cell_a:             u64,
    pub cell_b:             u64,
    /// Coordination strength [0, 32767] Q0.15 fixed-point.
    pub coordination:       i16,
    /// Ideological alignment [0, 32767].
    pub ideological_alignment: i16,
    pub tick_established:   u64,
}

/// The full insurgent cell network for a run.
pub struct CellNetwork {
    /// All cells, keyed by cell_id. BTreeMap for determinism.
    pub cells:        BTreeMap<u64, InsurgencyCell>,
    /// Coordination edges, keyed by (min_id, max_id). BTreeMap for determinism.
    pub edges:        BTreeMap<(u64, u64), CellCoordinationEdge>,
    /// Next cell_id (monotonic counter).
    pub next_cell_id: u64,
}

impl CellNetwork {
    pub fn new() -> Self {
        CellNetwork {
            cells:        BTreeMap::new(),
            edges:        BTreeMap::new(),
            next_cell_id: 1,
        }
    }

    pub fn active_cell_count(&self) -> usize {
        self.cells.values().filter(|c| c.is_active()).count()
    }

    pub fn active_cells_in_region(&self, region_key: &str) -> Vec<&InsurgencyCell> {
        self.cells.values()
            .filter(|c| c.is_active() && c.region_key == region_key)
            .collect()
    }

    /// Establish coordination edge between two cells if they share region
    /// or are ideologically aligned. Deterministic: sorts ids before inserting.
    pub fn try_establish_edge(
        &mut self,
        cell_a_id: u64,
        cell_b_id: u64,
        tick: u64,
    ) {
        let (min_id, max_id) = if cell_a_id < cell_b_id {
            (cell_a_id, cell_b_id)
        } else {
            (cell_b_id, cell_a_id)
        };
        if self.edges.contains_key(&(min_id, max_id)) { return; }

        let align = if let (Some(ca), Some(cb)) = (self.cells.get(&min_id), self.cells.get(&max_id)) {
            (ca.ideology.similarity(&cb.ideology) * 32767.0) as i16
        } else { return; };

        self.edges.insert((min_id, max_id), CellCoordinationEdge {
            cell_a: min_id, cell_b: max_id,
            coordination: (align / 2).clamp(0, 32767),
            ideological_alignment: align,
            tick_established: tick,
        });
    }
}
```

### 20.4 Operational Capability Formula

Operational capacity determines the attack capacity of a cell for executing campaigns:

```
attack_capacity(cell, t) = operational_capacity(cell, t)
    · network_coordination_bonus(cell, t)
    · safe_zone_coverage(cell, t)

network_coordination_bonus = 1.0
    + Σ_{edges adjacent to cell} (coordination_strength · 0.10)
    (capped at 2.0 — network at most doubles individual capacity)

safe_zone_coverage = safe_zone_count / max_safe_zones   (max_safe_zones = 5)
```

Campaigns consume resources and expose the cell to detection:

```
resource_cost_per_campaign = op_resource_base · campaign_scale
detection_risk_increase_per_campaign = campaign_scale · 0.15
```

### 20.5 Counterinsurgency Mechanics

Counterinsurgency (COIN) operations are directed interventions that target cells for detection, disruption, or hearts-and-minds programs.

**Detection probability:**

```
p_detect(cell, t) = base_detection_rate
    · (1 + intelligence_investment(r, t) · intelligence_multiplier)
    · detection_risk(cell, t)
    · (1 - cell_concealment_factor(cell, t))
```

Where:
- `base_detection_rate = 0.05` per tick per cell
- `intelligence_multiplier = 2.0`
- `cell_concealment_factor = safe_zone_count / 5 · 0.60` (safe zones provide concealment)

Detection does not immediately dissolve a cell — it triggers a disruption event that reduces operational capacity and increases attrition for the next 6 ticks.

**Disruption operation outcome:**

```
disruption_damage_members = floor(detection_capture_rate · detected_members)
disruption_damage_resources = resource_seized_fraction · resource_held
disruption_detection_risk_reduction = disruption_strength · 0.20
```

If disruption reduces `member_count < dissolution_threshold`, the cell transitions to `Dissolved`.

**Hearts-and-minds (HaM) programs:**

HaM programs target the Rebellious compartment directly, offering amnesty and welfare improvements to convert Rebellious citizens into Alienated (not engaging in cell support):

```
ham_effectiveness(r, t) = ham_investment(r, t)
    · (1 - ideology_distance(state_ideology, rebel_ideology) · 0.60)
    · legitimacy(r, t)

Δ R→A via HaM = ham_effectiveness · R(r, t) · ham_conversion_rate
```

Where `ham_conversion_rate = 0.08` per tick per unit of HaM effectiveness.

```rust
// crates/social/src/insurgency/counterinsurgency.rs

use crate::types::FpUnit;

#[derive(Debug, Clone)]
pub struct CounterinsurgencyOp {
    pub op_id:         u64,
    pub run_id:        u64,
    pub tick:          u64,
    pub region_key:    String,
    pub op_type:       CoinOpType,
    pub target_cell_id: Option<u64>,   // None = region-wide
    pub strength:      FpUnit,          // [0, FP_SCALE]
    pub outcome:       CoinOpOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoinOpType {
    IntelligenceGathering,
    DisruptionStrike,
    HeartsAndMinds,
    AmnestyOffer,
    SafeZoneElimination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoinOpOutcome {
    Pending,
    Success { members_captured: u32, resources_seized: i64 },
    PartialSuccess { detection_risk_added: i64 },
    Failure { blowback_legitimacy_loss: i64 },
}

/// Compute detection probability for a cell given current intelligence investment.
/// Returns probability as f64 ∈ [0, 1].
pub fn detection_probability(
    base_detection_rate:     f64,
    intelligence_investment: f64,
    cell_detection_risk:     f64,
    safe_zone_count:         u8,
) -> f64 {
    const INTELLIGENCE_MULTIPLIER: f64 = 2.0;
    const CONCEALMENT_PER_ZONE: f64 = 0.12;

    let concealment = (safe_zone_count as f64 * CONCEALMENT_PER_ZONE).clamp(0.0, 0.60);
    let p = base_detection_rate
        * (1.0 + intelligence_investment * INTELLIGENCE_MULTIPLIER)
        * cell_detection_risk
        * (1.0 - concealment);
    p.clamp(0.0, 1.0)
}
```

### 20.6 Insurgency-Legitimacy Feedback

When a cell executes a successful operation (attack, disruption, coercion of population), it reduces government legitimacy:

```
Δlegitimacy_from_op = -op_impact · (1 - government_response_effectiveness)
op_impact = attack_capacity(cell, t) · campaign_scale · legitimacy_damage_coeff
legitimacy_damage_coeff = 0.08
```

Failed operations (detected, disrupted before completion) reduce the cell's operational capacity without legitimacy damage:

```
operational_capacity(cell, t+1) -= failed_op_damage · 0.25
```

The feedback loop:

```
Successful op → legitimacy drops → insurgency propensity rises → R₀_civic rises →
more recruits for cell → cell grows → next op is larger
```

Counterbalance (when government responds effectively):

```
Effective COIN → cell disrupted → op_capacity drops → op fails → no legitimacy damage →
legitimacy stable → insurgency propensity stable → R₀_civic < 1.0 → rebellion decays
```

---

## 21. Extended Event Taxonomy and DDL

### 21.1 New Event Schemas

All eight new events conform to the CIV-0001 event envelope. JSON Schema definitions follow.

#### 21.1.1 `faction.membership_shifted.v1`

Emitted when any faction's membership in a region changes by more than `faction_emit_threshold = 0.02` in a single tick.

```json
{
  "$schema": "https://civlab.internal/schemas/faction.membership_shifted.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key", "faction_id",
               "membership_before", "membership_after", "internal_cohesion", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":          { "type": "string", "const": "faction.membership_shifted.v1" },
    "version":             { "type": "string", "const": "1" },
    "run_id":              { "type": "integer", "minimum": 1 },
    "tick":                { "type": "integer", "minimum": 0 },
    "region_key":          { "type": "string", "minLength": 1 },
    "faction_id":          { "type": "string",
                             "enum": ["technocratic","socialist","libertarian",
                                      "nationalist","religious","green"] },
    "membership_before":   { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "membership_after":    { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "delta":               { "type": "number", "minimum": -1.0, "maximum": 1.0 },
    "internal_cohesion":   { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "policy_pressure_magnitude": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at":          { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.2 `coalition.formed.v1`

Emitted at every election tick when a new governing coalition is established.

```json
{
  "$schema": "https://civlab.internal/schemas/coalition.formed.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key",
               "member_faction_ids", "combined_membership", "is_stable",
               "mean_ideology_distance", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":             { "type": "string", "const": "coalition.formed.v1" },
    "version":                { "type": "string", "const": "1" },
    "run_id":                 { "type": "integer", "minimum": 1 },
    "tick":                   { "type": "integer", "minimum": 0 },
    "region_key":             { "type": "string", "minLength": 1 },
    "member_faction_ids":     { "type": "array",
                                "items": { "type": "string" },
                                "minItems": 1 },
    "combined_membership":    { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "is_stable":              { "type": "boolean" },
    "mean_ideology_distance": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "electoral_legitimacy":   { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "effective_policy": {
      "type": "object",
      "required": ["redistribution","authority","tradition","cosmopolitan",
                   "ecology","theocracy","technocracy","militarism"],
      "additionalProperties": false,
      "properties": {
        "redistribution": { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "authority":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "tradition":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "cosmopolitan":   { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "ecology":        { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "theocracy":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "technocracy":    { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "militarism":     { "type": "number", "minimum": -1.0, "maximum": 1.0 }
      }
    },
    "created_at": { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.3 `civic.state_transitioned.v1`

Emitted when any civic compartment (E/A/R) changes by more than `civic_emit_threshold = 0.03` in a region.

```json
{
  "$schema": "https://civlab.internal/schemas/civic.state_transitioned.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key",
               "engaged", "alienated", "rebellious", "r0_civic", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":   { "type": "string", "const": "civic.state_transitioned.v1" },
    "version":      { "type": "string", "const": "1" },
    "run_id":       { "type": "integer", "minimum": 1 },
    "tick":         { "type": "integer", "minimum": 0 },
    "region_key":   { "type": "string", "minLength": 1 },
    "engaged":      { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "alienated":    { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "rebellious":   { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "lambda_ea":    { "type": "number", "minimum": 0.0 },
    "lambda_ar":    { "type": "number", "minimum": 0.0 },
    "mu_ae":        { "type": "number", "minimum": 0.0 },
    "mu_ra":        { "type": "number", "minimum": 0.0 },
    "r0_civic":     { "type": "number", "minimum": 0.0 },
    "r0_supercritical": { "type": "boolean" },
    "created_at":   { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.4 `ideology.radicalization_detected.v1`

Emitted when a node crosses the radicalization detection threshold (Section 18.4).

```json
{
  "$schema": "https://civlab.internal/schemas/ideology.radicalization_detected.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "node_id", "node_type",
               "ideology_norm", "velocity", "attractor_active", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":       { "type": "string", "const": "ideology.radicalization_detected.v1" },
    "version":          { "type": "string", "const": "1" },
    "run_id":           { "type": "integer", "minimum": 1 },
    "tick":             { "type": "integer", "minimum": 0 },
    "node_id":          { "type": "string", "minLength": 1 },
    "node_type":        { "type": "string" },
    "ideology_norm":    { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "velocity":         { "type": "number", "minimum": 0.0 },
    "attractor_active": { "type": "boolean" },
    "vector_snapshot": {
      "type": "object",
      "required": ["market","state","liberty","equality","security","tradition",
                   "ecology","cosmopolitan"],
      "additionalProperties": false,
      "properties": {
        "market":       { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "state":        { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "liberty":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "equality":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "security":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "tradition":    { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "ecology":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "cosmopolitan": { "type": "number", "minimum": -1.0, "maximum": 1.0 }
      }
    },
    "created_at": { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.5 `epidemic.outbreak_detected.v1`

Emitted when a region transitions from `EpidemicStatus::Clear` to `EpidemicStatus::Outbreak`.

```json
{
  "$schema": "https://civlab.internal/schemas/epidemic.outbreak_detected.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key",
               "origin", "susceptible", "exposed", "infected",
               "transmission_rate", "r_effective", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":        { "type": "string", "const": "epidemic.outbreak_detected.v1" },
    "version":           { "type": "string", "const": "1" },
    "run_id":            { "type": "integer", "minimum": 1 },
    "tick":              { "type": "integer", "minimum": 0 },
    "region_key":        { "type": "string", "minLength": 1 },
    "origin":            { "type": "string",
                           "enum": ["stochastic_emergence", "cross_region_import"] },
    "source_region_key": { "type": ["string", "null"] },
    "susceptible":       { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "exposed":           { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "infected":          { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "transmission_rate": { "type": "number", "minimum": 0.0 },
    "r_effective":       { "type": "number", "minimum": 0.0 },
    "health_system_capacity": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at":        { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.6 `insurgency.cell_formed.v1` (Extended)

The existing schema (Section 8.5) is extended with lifecycle and network fields. The existing schema remains backward compatible; new fields are optional.

```json
{
  "$schema": "https://civlab.internal/schemas/insurgency.cell_formed.v2.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "cell_id", "region_key",
               "formation_risk", "formation_mobilization", "ideology_vector",
               "lifecycle_state", "member_count", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":             { "type": "string", "const": "insurgency.cell_formed.v1" },
    "version":                { "type": "string", "const": "2" },
    "run_id":                 { "type": "integer", "minimum": 1 },
    "tick":                   { "type": "integer", "minimum": 0 },
    "cell_id":                { "type": "integer", "minimum": 1 },
    "region_key":             { "type": "string", "minLength": 1 },
    "formation_risk":         { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "formation_mobilization": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "lifecycle_state":        { "type": "string",
                                "enum": ["Nascent","Active","Operational","Degraded","Dissolved"] },
    "member_count":           { "type": "integer", "minimum": 0 },
    "ideology_vector": {
      "type": "object",
      "required": ["market","state","liberty","equality","security","tradition",
                   "ecology","cosmopolitan"],
      "additionalProperties": false,
      "properties": {
        "market":       { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "state":        { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "liberty":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "equality":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "security":     { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "tradition":    { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "ecology":      { "type": "number", "minimum": -1.0, "maximum": 1.0 },
        "cosmopolitan": { "type": "number", "minimum": -1.0, "maximum": 1.0 }
      }
    },
    "cell_ideology_distance_from_state": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at": { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.7 `insurgency.operation_executed.v1`

Emitted when an Operational cell executes a campaign.

```json
{
  "$schema": "https://civlab.internal/schemas/insurgency.operation_executed.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "cell_id", "region_key",
               "op_type", "attack_capacity", "legitimacy_damage",
               "resource_consumed", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":          { "type": "string", "const": "insurgency.operation_executed.v1" },
    "version":             { "type": "string", "const": "1" },
    "run_id":              { "type": "integer", "minimum": 1 },
    "tick":                { "type": "integer", "minimum": 0 },
    "cell_id":             { "type": "integer", "minimum": 1 },
    "region_key":          { "type": "string", "minLength": 1 },
    "op_type":             { "type": "string" },
    "attack_capacity":     { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "legitimacy_damage":   { "type": "number", "minimum": 0.0 },
    "resource_consumed":   { "type": "integer", "minimum": 0 },
    "detection_risk_after": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at":          { "type": "integer", "minimum": 0 }
  }
}
```

#### 21.1.8 `counterinsurgency.operation_result.v1`

Emitted after each COIN operation resolves.

```json
{
  "$schema": "https://civlab.internal/schemas/counterinsurgency.operation_result.v1.json",
  "type": "object",
  "required": ["event_type", "version", "run_id", "tick", "region_key",
               "op_type", "outcome", "created_at"],
  "additionalProperties": false,
  "properties": {
    "event_type":           { "type": "string", "const": "counterinsurgency.operation_result.v1" },
    "version":              { "type": "string", "const": "1" },
    "run_id":               { "type": "integer", "minimum": 1 },
    "tick":                 { "type": "integer", "minimum": 0 },
    "region_key":           { "type": "string", "minLength": 1 },
    "op_type":              { "type": "string",
                              "enum": ["IntelligenceGathering","DisruptionStrike",
                                       "HeartsAndMinds","AmnestyOffer","SafeZoneElimination"] },
    "target_cell_id":       { "type": ["integer", "null"] },
    "outcome":              { "type": "string",
                              "enum": ["Success","PartialSuccess","Failure"] },
    "members_captured":     { "type": "integer", "minimum": 0 },
    "resources_seized":     { "type": "integer", "minimum": 0 },
    "legitimacy_change":    { "type": "number" },
    "blowback_risk":        { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "created_at":           { "type": "integer", "minimum": 0 }
  }
}
```

### 21.2 Extended SQL Tables

Five new tables, all append-only and tick-keyed, conforming to the persistence protocol in Section 15.5.

```sql
-- faction_states: Per-faction snapshot per region per tick
CREATE TABLE IF NOT EXISTS faction_states (
    id                      BIGSERIAL   PRIMARY KEY,
    run_id                  BIGINT      NOT NULL,
    tick                    BIGINT      NOT NULL,
    region_key              TEXT        NOT NULL,
    faction_id              TEXT        NOT NULL CHECK (faction_id IN (
                                'technocratic','socialist','libertarian',
                                'nationalist','religious','green')),
    membership              REAL        NOT NULL CHECK (membership >= 0.0 AND membership <= 1.0),
    policy_pressure         REAL        NOT NULL CHECK (policy_pressure >= 0.0 AND policy_pressure <= 1.0),
    internal_cohesion       REAL        NOT NULL CHECK (internal_cohesion >= 0.0 AND internal_cohesion <= 1.0),
    created_at              BIGINT      NOT NULL,
    UNIQUE (run_id, tick, region_key, faction_id)
);

CREATE INDEX idx_faction_states_run_tick
    ON faction_states (run_id, tick);
CREATE INDEX idx_faction_states_region_faction
    ON faction_states (run_id, region_key, faction_id, tick);

-- civic_compartments: Per-region civic engagement compartmental state per tick
CREATE TABLE IF NOT EXISTS civic_compartments (
    id              BIGSERIAL   PRIMARY KEY,
    run_id          BIGINT      NOT NULL,
    tick            BIGINT      NOT NULL,
    region_key      TEXT        NOT NULL,
    engaged         REAL        NOT NULL CHECK (engaged >= 0.0 AND engaged <= 1.0),
    alienated       REAL        NOT NULL CHECK (alienated >= 0.0 AND alienated <= 1.0),
    rebellious      REAL        NOT NULL CHECK (rebellious >= 0.0 AND rebellious <= 1.0),
    lambda_ea       REAL        NOT NULL CHECK (lambda_ea >= 0.0),
    lambda_ar       REAL        NOT NULL CHECK (lambda_ar >= 0.0),
    mu_ae           REAL        NOT NULL CHECK (mu_ae >= 0.0),
    mu_ra           REAL        NOT NULL CHECK (mu_ra >= 0.0),
    r0_civic        REAL        NOT NULL CHECK (r0_civic >= 0.0),
    created_at      BIGINT      NOT NULL,
    UNIQUE (run_id, tick, region_key),
    CONSTRAINT civic_conservation CHECK (
        ABS(engaged + alienated + rebellious - 1.0) < 0.01
    )
);

CREATE INDEX idx_civic_compartments_run_tick
    ON civic_compartments (run_id, tick);
CREATE INDEX idx_civic_compartments_region
    ON civic_compartments (run_id, region_key, tick);

-- ideology_vectors_extended: Extended 8-axis ideology per node per tick (sparse)
CREATE TABLE IF NOT EXISTS ideology_vectors_extended (
    id              BIGSERIAL   PRIMARY KEY,
    run_id          BIGINT      NOT NULL,
    tick            BIGINT      NOT NULL,
    node_id         TEXT        NOT NULL,
    node_type       TEXT        NOT NULL,
    v_market        REAL        NOT NULL CHECK (v_market >= -1.0 AND v_market <= 1.0),
    v_state         REAL        NOT NULL CHECK (v_state >= -1.0 AND v_state <= 1.0),
    v_liberty       REAL        NOT NULL CHECK (v_liberty >= -1.0 AND v_liberty <= 1.0),
    v_equality      REAL        NOT NULL CHECK (v_equality >= -1.0 AND v_equality <= 1.0),
    v_security      REAL        NOT NULL CHECK (v_security >= -1.0 AND v_security <= 1.0),
    v_tradition     REAL        NOT NULL CHECK (v_tradition >= -1.0 AND v_tradition <= 1.0),
    v_ecology       REAL        NOT NULL CHECK (v_ecology >= -1.0 AND v_ecology <= 1.0),
    v_cosmopolitan  REAL        NOT NULL CHECK (v_cosmopolitan >= -1.0 AND v_cosmopolitan <= 1.0),
    ideology_norm   REAL        NOT NULL CHECK (ideology_norm >= 0.0 AND ideology_norm <= 1.0),
    radicalization_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      BIGINT      NOT NULL,
    UNIQUE (run_id, tick, node_id)
);

CREATE INDEX idx_ideology_vectors_ext_run_tick
    ON ideology_vectors_extended (run_id, tick);
CREATE INDEX idx_ideology_vectors_ext_node
    ON ideology_vectors_extended (run_id, node_id, tick);

-- epidemic_states: Per-region epidemic SEIR state per tick (sparse; only during outbreak)
CREATE TABLE IF NOT EXISTS epidemic_states (
    id                    BIGSERIAL   PRIMARY KEY,
    run_id                BIGINT      NOT NULL,
    tick                  BIGINT      NOT NULL,
    region_key            TEXT        NOT NULL,
    status                TEXT        NOT NULL CHECK (status IN ('Clear','Outbreak','Contained')),
    susceptible           REAL        NOT NULL CHECK (susceptible >= 0.0 AND susceptible <= 1.0),
    exposed               REAL        NOT NULL CHECK (exposed >= 0.0 AND exposed <= 1.0),
    infected              REAL        NOT NULL CHECK (infected >= 0.0 AND infected <= 1.0),
    recovered_immune      REAL        NOT NULL CHECK (recovered_immune >= 0.0 AND recovered_immune <= 1.0),
    transmission_rate     REAL        NOT NULL CHECK (transmission_rate >= 0.0),
    r_effective           REAL        NOT NULL CHECK (r_effective >= 0.0),
    outbreak_duration     INTEGER     NOT NULL DEFAULT 0,
    health_system_capacity REAL       NOT NULL CHECK (health_system_capacity >= 0.0 AND health_system_capacity <= 1.0),
    created_at            BIGINT      NOT NULL,
    UNIQUE (run_id, tick, region_key)
);

CREATE INDEX idx_epidemic_states_run_tick
    ON epidemic_states (run_id, tick);
CREATE INDEX idx_epidemic_states_region
    ON epidemic_states (run_id, region_key, tick);

-- insurgency_cells_extended: Full lifecycle cell tracking (extends mobilization_cells)
CREATE TABLE IF NOT EXISTS insurgency_cells_extended (
    id                      BIGSERIAL   PRIMARY KEY,
    cell_id                 BIGINT      NOT NULL,
    run_id                  BIGINT      NOT NULL,
    tick_snapshot           BIGINT      NOT NULL,
    region_key              TEXT        NOT NULL,
    lifecycle_state         TEXT        NOT NULL CHECK (lifecycle_state IN (
                                'Nascent','Active','Operational','Degraded','Dissolved')),
    member_count            INTEGER     NOT NULL CHECK (member_count >= 0),
    trained_members         INTEGER     NOT NULL CHECK (trained_members >= 0),
    resource_held           BIGINT      NOT NULL DEFAULT 0,
    safe_zone_count         SMALLINT    NOT NULL CHECK (safe_zone_count >= 0 AND safe_zone_count <= 5),
    operational_capacity    REAL        NOT NULL CHECK (operational_capacity >= 0.0 AND operational_capacity <= 1.0),
    detection_risk          REAL        NOT NULL CHECK (detection_risk >= 0.0 AND detection_risk <= 1.0),
    v_market                REAL        NOT NULL,
    v_state                 REAL        NOT NULL,
    v_liberty               REAL        NOT NULL,
    v_equality              REAL        NOT NULL,
    v_security              REAL        NOT NULL,
    v_tradition             REAL        NOT NULL,
    v_ecology               REAL        NOT NULL,
    v_cosmopolitan          REAL        NOT NULL,
    tick_formed             BIGINT      NOT NULL,
    tick_last_op            BIGINT,
    tick_dissolved          BIGINT,
    created_at              BIGINT      NOT NULL,
    UNIQUE (run_id, cell_id, tick_snapshot)
);

CREATE INDEX idx_cells_ext_run_region_tick
    ON insurgency_cells_extended (run_id, region_key, tick_snapshot);
CREATE INDEX idx_cells_ext_lifecycle
    ON insurgency_cells_extended (run_id, lifecycle_state, tick_snapshot);
```

---

## 22. Cross-Module Integration

### 22.1 Faction Pressure to Policy DSL

The `PolicyPressure` computed by the faction aggregation function (Section 16.4) is translated into constraints on the `policy.evaluate()` function in the Policy DSL (`crates/policy`).

**Constraint generation:**

For each policy axis where `|PolicyPressure(axis)| > pressure_constraint_threshold` (default: 0.30 × FP_SCALE):

```rust
// crates/policy/src/faction_constraints.rs

pub struct FactionPolicyConstraint {
    pub axis:        PolicyAxis,
    pub direction:   i8,   // +1 = faction demands higher, -1 = lower
    pub pressure:    FpUnit,
    pub coalition_backed: bool,  // true if the constraint comes from governing coalition
}
```

Coalition-backed constraints carry veto weight: if `policy.evaluate()` produces a score that violates a coalition-backed constraint, the policy bundle is flagged as `coalition_incompatible = true` and the engine logs it as `policy.coalition_tension.v1` (internal diagnostic).

**Non-coalition faction pressure** is advisory: it modifies the `insurgency_propensity` input for the faction's member regions proportional to how far the enacted policy deviates from faction preference:

```
faction_grievance(f, r, t) = Σ_axis |enacted_policy(axis) - preference(f, axis)| / 8
faction_propensity_contribution(f, r, t) = faction_grievance · membership(f, r, t) · γ_faction
```

Where `γ_faction = 0.06`. This feeds into the `γ_capture` channel of insurgency propensity (Section 4.1) as political alienation.

### 22.2 Insurgency to Diplomacy (CIV-0105 Coupling)

The `CellNetwork` state is exposed to the diplomacy and war module (CIV-0105) via `WorldStateView`:

```rust
// Added to WorldStateView (crates/engine/src/world_state.rs)
pub insurgency_active_cells: BTreeMap<String, u32>,         // region_key → active cell count
pub insurgency_op_capacity:  BTreeMap<String, FpUnit>,       // region_key → aggregate op capacity
pub insurgency_r0_civic:     BTreeMap<String, i64>,          // region_key → R₀ analog
```

**Diplomatic leverage reduction:**

When a region has `active_cell_count ≥ 3` OR `R₀_civic > 1.5 × FP_SCALE`, the region's diplomatic leverage in external negotiations is reduced:

```
diplomatic_leverage(r, t) = base_leverage(r, t)
    · (1 - insurgency_leverage_penalty(r, t))

insurgency_leverage_penalty = clamp(
    0.10 · active_cell_count(r, t)
    + 0.20 · max(0, r0_civic(r, t) - FP_SCALE) / FP_SCALE,
    0.0, 0.50
)
```

Maximum penalty: 50% leverage reduction. This models that foreign counterparts perceive internal instability as weakening the negotiating position.

**Foreign exploitation:**

Foreign actors (from CIV-0105 shadow module) may covertly support insurgent cells:

```
foreign_cell_support(r, t) = foreign_actor_hostility(r, t)
    · foreign_support_intensity
    · (1 - counterintelligence_effectiveness(r, t))
```

This adds to `resource_acquisition` for cells whose ideology is aligned with the foreign actor (ideological distance < 0.35).

### 22.3 Ideology to Hidden Network (CIV-0105 Coupling)

The extended ideology vectors of citizen cohorts seed the initial ideology vectors of hidden network actors during world initialization:

```
hidden_actor_ideology = weighted_mean(
    cohort_ideology vectors in actor's region of influence,
    weights = cohort_population_fractions
)
```

This creates ideological coherence between the population and the power structures that emerge from it. Hidden network actors then evolve their ideology independently via the extended diffusion step (Section 18.3), potentially diverging from the population over time — especially under radicalization attractor conditions.

The coupling is one-directional at initialization. During the run, hidden actor ideology influences population via `IdeologyEdge` connections (Section 2.2), but population ideology does not directly update hidden actor ideology — only through the social contact graph edges.

**Divergence monitoring:**

The mean ideological distance between hidden network actors and the regional population is computed each tick:

```
hidden_population_divergence(r, t) = mean_{actor in r} d8_norm(actor.ideology, regional_mean_ideology(r, t))
```

When `hidden_population_divergence > 0.55`, emit `ideology.hidden_divergence_alert.v1` (diagnostic). This models a detachment of power structures from the populations they govern — a precursor to legitimacy collapse.

### 22.4 Health Burden to Labor Supply (CIV-0102 Coupling)

The `productivity_modifier` from Section 19.2 is written to `WorldStateView` for consumption by the economic module:

```rust
// Added to WorldStateView
pub cohort_productivity_modifier: BTreeMap<(String, String), FpUnit>,  // (region, cohort) → modifier
```

The economic module (CIV-0102) multiplies the joule production rate of each cohort by its `productivity_modifier`:

```
joule_output(cohort, t) = base_joule_output(cohort, t) · productivity_modifier(cohort, t)
```

Under epidemic conditions (`EpidemicStatus::Outbreak` AND `infected_fraction > 0.15`), a separate epidemic productivity shock applies:

```
epidemic_productivity_shock = 1.0 - (infected_fraction · epidemic_labor_coefficient)
epidemic_labor_coefficient = 0.80  (sick workers produce 80% fewer joules)
```

The combined modifier:

```
effective_productivity = productivity_modifier · epidemic_productivity_shock
```

This is the primary feedback loop between health and economic output. Sustained epidemic + high chronic burden can reduce regional joule production by 40–60% at default parameters.

### 22.5 Full Integration Event Routing Table

| Event | Emitting Module | Consuming Modules | Routing Channel |
|---|---|---|---|
| `faction.membership_shifted.v1` | CIV-0106 faction | CIV-0103 institutions | `WorldStateView.faction_membership` |
| `coalition.formed.v1` | CIV-0106 faction | CIV-0103 policy, CIV-0105 diplomacy | `WorldStateView.active_coalition` |
| `civic.state_transitioned.v1` | CIV-0106 civic | CIV-0105 insurgency coupling | `WorldStateView.civic_compartments` |
| `ideology.radicalization_detected.v1` | CIV-0106 ideology | CIV-0105 shadow network | `WorldStateView.radicalization_alerts` |
| `epidemic.outbreak_detected.v1` | CIV-0106 health | CIV-0102 economy, CIV-0103 citizen | `WorldStateView.epidemic_states` |
| `insurgency.operation_executed.v1` | CIV-0106 insurgency | CIV-0105 diplomacy/war | `WorldStateView.insurgency_ops` |
| `counterinsurgency.operation_result.v1` | CIV-0106 insurgency | CIV-0103 institutions | `WorldStateView.coin_ops` |
| `health.trajectory_critical.v1` | CIV-0106 health | CIV-0001 engine alert | `WorldStateView.health_alerts` |

All cross-module state is passed exclusively through `WorldStateView` — direct crate dependencies between `crates/social` and other domain crates are forbidden (enforced by `tach.toml` boundary rules).

---

## 23. Extended Test Suite

The following ten additional tests extend the acceptance suite from Section 14. All tests reside in `crates/social/src/tests/` in the appropriate submodule.

```rust
// ============================================================
// crates/social/src/tests/faction.rs
// ============================================================

/// FR-SOC-FAC-001: Faction preference aggregation is membership-weighted
/// and conserved across total membership sum.
#[test]
fn test_faction_pressure_conservation() {
    // Setup: six factions each with uniform membership (1/6 each).
    // Assertion: the PolicyPressure magnitude must equal the membership-weighted
    // mean of faction preference norms (within fixed-point rounding).
    let states    = FactionState::uniform_test_fixture();
    let prefs     = FactionPreference::defaults();
    let pressure  = aggregate_faction_pressure(&states, &prefs);

    // Conservation: total membership in states sums to 1.0 (uniform 1/6 each).
    let total_mem: f64 = states.values().map(|s| s.membership.as_f64()).sum();
    assert!((total_mem - 1.0).abs() < 1e-4,
        "total faction membership must sum to 1.0, got {}", total_mem);

    // Pressure magnitude must be > 0 and <= 1.
    assert!(pressure.total_magnitude.as_f64() > 0.0,
        "pressure magnitude must be positive");
    assert!(pressure.total_magnitude.as_f64() <= 1.0,
        "pressure magnitude must not exceed 1.0");
}

/// FR-SOC-FAC-002: Faction membership drift is bounded by ζ_faction per tick.
#[test]
fn test_faction_membership_drift_bounded() {
    const ZETA_FACTION: f64 = 0.08;
    let mut states = FactionState::test_fixture_shifted_preference();
    let initial: std::collections::BTreeMap<FactionId, f64> = states
        .iter()
        .map(|(fid, s)| (*fid, s.membership.as_f64()))
        .collect();

    // Simulate one membership update step with a large ideology shift.
    apply_membership_update(&mut states, &IdeologySnapshot::large_shift_fixture());

    for (fid, state) in states.iter() {
        let before = initial[fid];
        let after  = state.membership.as_f64();
        let delta  = (after - before).abs();
        assert!(delta <= ZETA_FACTION + 1e-6,
            "faction {:?} membership drift {} exceeds ζ_faction {}", fid, delta, ZETA_FACTION);
    }
}

// ============================================================
// crates/social/src/tests/civic.rs
// ============================================================

/// FR-SOC-CIV-001: Civic compartmental R₀ is bounded correctly.
/// R₀ > 1.0 iff λ_AR · A > μ_RA (self-sustaining condition).
#[test]
fn test_r0_civic_bounds_and_criticality() {
    // Sub-critical scenario: high welfare, low alienated fraction, active amnesty.
    let subcrit = civic_step_with_params(
        CivicStepParams {
            material_stress: 0.10, coercion: 0.10, welfare_gap: 0.05,
            welfare_coverage: 0.85, legitimacy: 0.80, polarization: 0.10,
            amnesty_applied: 0.50, active_cells: 0,
            initial_alienated: 0.10, ..Default::default()
        }
    );
    assert!(subcrit.r0_civic < FP_SCALE,
        "sub-critical R₀ must be < 1.0 (FP_SCALE), got {}", subcrit.r0_civic);

    // Super-critical scenario: low welfare, high alienated, active cells.
    let supercrit = civic_step_with_params(
        CivicStepParams {
            material_stress: 0.80, coercion: 0.70, welfare_gap: 0.60,
            welfare_coverage: 0.15, legitimacy: 0.20, polarization: 0.75,
            amnesty_applied: 0.0, active_cells: 4,
            initial_alienated: 0.40, ..Default::default()
        }
    );
    assert!(supercrit.r0_civic > FP_SCALE,
        "super-critical R₀ must be > 1.0 (FP_SCALE), got {}", supercrit.r0_civic);
}

/// FR-SOC-CIV-002: Civic compartment population is conserved (E+A+R=1.0).
#[test]
fn test_civic_compartment_conservation() {
    let params_varied = vec![
        CivicStepParams { material_stress: 0.5, coercion: 0.4, ..Default::default() },
        CivicStepParams { welfare_coverage: 0.9, legitimacy: 0.8, ..Default::default() },
        CivicStepParams { active_cells: 6, polarization: 0.9, ..Default::default() },
    ];
    for p in params_varied {
        let result = civic_step_with_params(p);
        result.assert_conserved();
    }
}

// ============================================================
// crates/social/src/tests/ideology.rs (extended)
// ============================================================

/// FR-SOC-IDE-005: Ideology diffusion is deterministic — same seed, same graph,
/// same output across repeated calls.
#[test]
fn test_ideology_diffusion_determinism() {
    let graph  = IdeologyGraph::test_fixture_complex();
    let params = IdeologyParams::default();

    let result_1 = run_extended_diffusion_n_ticks(&graph, &params, 50);
    let result_2 = run_extended_diffusion_n_ticks(&graph, &params, 50);

    for (node_id, vec1) in result_1.iter() {
        let vec2 = &result_2[node_id];
        assert_eq!(vec1.0, vec2.0,
            "ideology diffusion must be deterministic for node {}", node_id);
    }
}

/// FR-SOC-IDE-006: Radicalization attractor only activates under all three
/// conditions simultaneously; one condition alone is insufficient.
#[test]
fn test_radicalization_attractor_requires_all_conditions() {
    // Only low legitimacy — not sufficient.
    assert!(!radicalization_attractor_active(0.20, 0.50, 0.25),
        "low legitimacy alone must not activate attractor");
    // Only high polarization — not sufficient.
    assert!(!radicalization_attractor_active(0.50, 0.80, 0.25),
        "high polarization alone must not activate attractor");
    // Only high welfare gap — not sufficient.
    assert!(!radicalization_attractor_active(0.50, 0.50, 0.50),
        "high welfare gap alone must not activate attractor");
    // All three — sufficient.
    assert!(radicalization_attractor_active(0.20, 0.80, 0.50),
        "all three conditions must activate attractor");
}

// ============================================================
// crates/social/src/tests/health.rs (extended)
// ============================================================

/// FR-SOC-HLT-005: Epidemic containment reduces R_effective below 1.0
/// when health system capacity exceeds containment threshold.
#[test]
fn test_epidemic_containment_effectiveness() {
    let params_low_q = EpidemicParams { vaccination_coverage: 0.0, ..Default::default() };
    let params_hi_q  = EpidemicParams { vaccination_coverage: 0.60, ..Default::default() };

    let r_eff_low = compute_r_effective(0.80, 0.0, &params_low_q);
    let r_eff_hi  = compute_r_effective(0.80, 0.85, &params_hi_q);

    assert!(r_eff_low > 1.0,
        "R_effective must exceed 1.0 without vaccination or capacity");
    assert!(r_eff_hi < 1.0,
        "R_effective must drop below 1.0 with high vaccination + capacity");
}

// ============================================================
// crates/social/src/tests/insurgency.rs (extended)
// ============================================================

/// FR-SOC-INS-006: Insurgency cell lifecycle transitions follow declared rules.
/// Nascent → Active requires member_count >= nascent_threshold.
#[test]
fn test_cell_lifecycle_nascent_to_active() {
    const NASCENT_THRESHOLD: u32 = 15;
    let mut cell = InsurgencyCell::test_nascent_fixture();
    assert_eq!(cell.lifecycle_state, CellLifecycleState::Nascent);

    // Below threshold — must stay Nascent.
    cell.member_count = NASCENT_THRESHOLD - 1;
    apply_lifecycle_transition(&mut cell);
    assert_eq!(cell.lifecycle_state, CellLifecycleState::Nascent,
        "cell must remain Nascent below member threshold");

    // At threshold — must transition to Active.
    cell.member_count = NASCENT_THRESHOLD;
    apply_lifecycle_transition(&mut cell);
    assert_eq!(cell.lifecycle_state, CellLifecycleState::Active,
        "cell must transition to Active at member threshold");
}

/// FR-SOC-INS-007: Counterinsurgency detection probability is monotonically
/// increasing in intelligence_investment and cell detection_risk.
#[test]
fn test_coin_detection_probability_monotone() {
    let p_low_intel  = detection_probability(0.05, 0.10, 0.50, 0);
    let p_high_intel = detection_probability(0.05, 0.90, 0.50, 0);
    assert!(p_high_intel > p_low_intel,
        "detection probability must increase with intelligence investment");

    let p_low_risk  = detection_probability(0.05, 0.50, 0.10, 0);
    let p_high_risk = detection_probability(0.05, 0.50, 0.90, 0);
    assert!(p_high_risk > p_low_risk,
        "detection probability must increase with cell detection risk");

    // Safe zones must reduce detection probability.
    let p_no_zones   = detection_probability(0.05, 0.50, 0.50, 0);
    let p_with_zones = detection_probability(0.05, 0.50, 0.50, 4);
    assert!(p_with_zones < p_no_zones,
        "safe zones must reduce detection probability");
}

// ============================================================
// crates/social/src/tests/integration.rs (extended)
// ============================================================

/// FR-SOC-INTG-004: Faction coalition stability feeds back into insurgency propensity.
/// An unstable coalition (mean ideology distance > COALITION_DISTANCE_THRESHOLD)
/// adds the declared insurgency propensity contribution.
#[test]
fn test_unstable_coalition_raises_insurgency_propensity() {
    let stable_propensity   = run_with_coalition_stability(true,  20);
    let unstable_propensity = run_with_coalition_stability(false, 20);

    assert!(unstable_propensity > stable_propensity,
        "unstable coalition must produce higher insurgency propensity");

    // The delta must be attributable to the declared +0.08 per tick contribution.
    let propensity_delta = unstable_propensity - stable_propensity;
    assert!(propensity_delta >= 0.05,
        "propensity delta {} below expected minimum contribution", propensity_delta);
}

/// FR-SOC-INTG-005: Cross-module health-labor feedback: epidemic reduces joule output.
/// Infected fraction > 0.15 must reduce productivity_modifier by at least
/// epidemic_labor_coefficient · infected_fraction.
#[test]
fn test_epidemic_reduces_labor_productivity() {
    let no_epidemic_modifier = compute_productivity_modifier(
        ProductivityInput { infected_fraction: 0.0, chronic_burden: 0.1,
                            strained_fraction: 0.05, disabled_fraction: 0.02 }
    );
    let epidemic_modifier = compute_productivity_modifier(
        ProductivityInput { infected_fraction: 0.25, chronic_burden: 0.1,
                            strained_fraction: 0.05, disabled_fraction: 0.02 }
    );
    assert!(epidemic_modifier < no_epidemic_modifier,
        "epidemic must reduce labor productivity modifier");

    // Expected reduction floor: 0.25 · 0.80 = 0.20.
    let reduction = no_epidemic_modifier - epidemic_modifier;
    assert!(reduction >= 0.15,
        "epidemic productivity reduction {} below expected floor", reduction);
}

/// FR-SOC-INTG-006: Radicalization attractor stability — under sustained
/// radicalization conditions (50 ticks), ideology norm exceeds 0.80.
#[test]
fn test_radicalization_attractor_drives_to_extreme() {
    // Start from a moderately positioned node (norm ≈ 0.40).
    let initial = ExtendedIdeologyVector([
        13000, -6000, 8000, -10000, 14000, 7000, -5000, 9000
    ]);
    let conditions = RadicalizationConditions {
        legitimacy:   0.15,
        polarization: 0.80,
        welfare_gap:  0.55,
    };
    let final_vec = run_radicalization_for_n_ticks(initial, conditions, 50);
    let norm: f64 = final_vec.as_f64_array().iter().map(|v| v.abs()).sum::<f64>() / 8.0;

    assert!(norm > 0.80,
        "ideology norm must exceed 0.80 after 50 ticks of radicalization, got {}", norm);
}

/// FR-SOC-INTG-007: Civic recovery path — sustained welfare + legitimacy improvement
/// drives Rebellious fraction to near zero within 30 ticks from R(t=0)=0.30.
#[test]
fn test_civic_recovery_path_succeeds() {
    let initial = CivicCompartments::test_fixture_high_rebellion(0.30);
    let recovery_params = CivicParams {
        // High welfare and legitimacy, active amnesty.
        gamma_welfare: FpUnit::from_f64(0.20),
        gamma_legit:   FpUnit::from_f64(0.15),
        delta_amnesty: FpUnit::from_f64(0.18),
        ..Default::default()
    };
    let driver_inputs = CivicDrivers {
        welfare_coverage: 0.85, legitimacy: 0.80,
        amnesty_applied: 0.80, material_stress: 0.10,
        coercion: 0.05, active_cells: 0, ..Default::default()
    };
    let final_state = run_civic_for_n_ticks(&initial, &recovery_params, driver_inputs, 30);

    assert!(final_state.rebellious.as_f64() < 0.05,
        "rebellious fraction must drop below 0.05 after 30 ticks of recovery, got {}",
        final_state.rebellious.as_f64());
}
```

---

## Appendix C: Extended FR Traceability

| FR ID | Description | Section | Test |
|---|---|---|---|
| FR-SOC-FAC-001 | Faction pressure aggregation conserved | 16.4 | `test_faction_pressure_conservation` |
| FR-SOC-FAC-002 | Faction membership drift bounded by ζ | 16.3 | `test_faction_membership_drift_bounded` |
| FR-SOC-CIV-001 | Civic R₀ correctly identifies criticality | 17.5 | `test_r0_civic_bounds_and_criticality` |
| FR-SOC-CIV-002 | Civic compartment population conserved | 17.4 | `test_civic_compartment_conservation` |
| FR-SOC-IDE-005 | Extended diffusion is deterministic | 18.3 | `test_ideology_diffusion_determinism` |
| FR-SOC-IDE-006 | Radicalization attractor requires all three conditions | 18.4 | `test_radicalization_attractor_requires_all_conditions` |
| FR-SOC-HLT-005 | Epidemic containment reduces R_effective below 1.0 | 19.3 | `test_epidemic_containment_effectiveness` |
| FR-SOC-INS-006 | Cell lifecycle transition rules enforced | 20.1 | `test_cell_lifecycle_nascent_to_active` |
| FR-SOC-INS-007 | COIN detection probability is monotone | 20.5 | `test_coin_detection_probability_monotone` |
| FR-SOC-INTG-004 | Unstable coalition raises insurgency propensity | 22.1 | `test_unstable_coalition_raises_insurgency_propensity` |
| FR-SOC-INTG-005 | Epidemic reduces labor productivity (economy coupling) | 22.4 | `test_epidemic_reduces_labor_productivity` |
| FR-SOC-INTG-006 | Radicalization attractor drives ideology norm > 0.80 | 18.4 | `test_radicalization_attractor_drives_to_extreme` |
| FR-SOC-INTG-007 | Civic recovery path: rebellion decays under sustained welfare | 17.3 | `test_civic_recovery_path_succeeds` |

---

## Appendix D: Extended Default Coefficient Summary

| Parameter | Default | Range | Override Via | Section |
|---|---|---|---|---|
| `ζ_faction` | 0.08 | [0.01, 0.30] | `FactionParams` | 16.3 |
| `coalition_threshold` | 0.50 | [0.33, 0.67] | `ElectionParams` | 16.5 |
| `coalition_distance_threshold` | 0.45 | [0.20, 0.80] | `ElectionParams` | 16.5 |
| `election_period_ticks` | 48 | [12, 200] | `ElectionParams` | 16.6 |
| `faction_turnout` | 0.75 | [0.30, 1.0] | `ElectionParams` | 16.6 |
| `γ_faction` | 0.06 | [0, 0.20] | `InsurgencyParams` | 22.1 |
| `α_stress` (civic) | 0.12 | [0, 0.30] | `CivicParams` | 17.3 |
| `β_social` (civic) | 0.18 | [0, 0.40] | `CivicParams` | 17.3 |
| `μ_RA max` | 0.06 | [0, 0.20] | `CivicParams` | 17.3 |
| `max_lambda_EA` | 0.15 | [0, 0.30] | `CivicParams` | 17.3 |
| `base_contact_rate` | 0.25 | [0.05, 0.60] | `CivicParams` | 17.6 |
| `κ_cohesion_contact` | 0.40 | [0, 0.80] | `CivicParams` | 17.6 |
| `κ_spatial_contact` | 0.05 | [0, 0.20] | `CivicParams` | 17.6 |
| `η_ext` (extended diffusion) | 0.03 | [0, 0.15] | `IdeologyParams` | 18.3 |
| `radicalization_boost` | 0.02 | [0, 0.10] | `IdeologyParams` | 18.4 |
| `radicalization_axis_threshold` | 0.40 | [0.20, 0.80] | `IdeologyParams` | 18.4 |
| `media_amplify_coeff` | 0.60 | [0, 1.50] | `IdeologyParams` | 18.5 |
| `persistence_rate` (chronic) | 0.95 | [0.80, 0.99] | `HealthParams` | 19.2 |
| `acute_to_chronic_conversion` | 0.10 | [0.02, 0.30] | `HealthParams` | 19.2 |
| `base_transmission` | 0.08 | [0.02, 0.30] | `EpidemicParams` | 19.3 |
| `epidemic_seed_rate` | 0.002 | [0.0005, 0.01] | `EpidemicParams` | 19.3 |
| `case_fatality_rate` | 0.02 | [0.001, 0.20] | `EpidemicParams` | 19.3 |
| `surge_multiplier_emergency` | 2.0 | [1.0, 3.0] | `HealthParams` | 19.4 |
| `Q_max_emergency` | 0.85 | [0.60, 1.0] | `HealthParams` | 19.4 |
| `nascent_threshold` | 15 | [5, 50] | `InsurgencyParams` | 20.1 |
| `dissolution_threshold` | 5 | [1, 20] | `InsurgencyParams` | 20.1 |
| `base_recruit` | 0.08 | [0.02, 0.25] | `InsurgencyParams` | 20.2 |
| `natural_attrition` | 0.02 | [0.005, 0.10] | `InsurgencyParams` | 20.2 |
| `base_detection_rate` | 0.05 | [0.01, 0.30] | `CoinParams` | 20.5 |
| `intelligence_multiplier` | 2.0 | [1.0, 5.0] | `CoinParams` | 20.5 |
| `legitimacy_damage_coeff` | 0.08 | [0.02, 0.25] | `InsurgencyParams` | 20.6 |
| `insurgency_leverage_penalty_cap` | 0.50 | [0.10, 0.80] | `DiplomacyParams` | 22.2 |
| `hidden_divergence_alert_threshold` | 0.55 | [0.30, 0.80] | `IdeologyParams` | 22.3 |
| `pressure_constraint_threshold` | 0.30 | [0.10, 0.60] | `FactionParams` | 22.1 |
| `epidemic_labor_coefficient` | 0.80 | [0.30, 1.0] | `HealthParams` | 22.4 |
| `faction_emit_threshold` | 0.02 | [0.005, 0.10] | `EventParams` | 21.1.1 |
| `civic_emit_threshold` | 0.03 | [0.005, 0.10] | `EventParams` | 21.1.3 |
| `ideology_emit_threshold_ext` | 0.005 | [0.001, 0.02] | `EventParams` | 18.3 |
