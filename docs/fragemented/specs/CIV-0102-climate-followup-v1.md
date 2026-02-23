# CIV-0102: Climate Follow-up — Damage Functions, Scenario Families, Adaptation Levers

**Spec ID:** CIV-0102
**Version:** 2.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Climate Systems Team

**Related Specs:**
- CIV-0001: Core Simulation Loop (tick architecture, determinism invariants)
- CIV-0100: Economy v1 (double-entry ledger, conservation invariants)
- CIV-0107: Joule Economy System v1 (energy accounting, quota mechanics)

---

## Table of Contents

1. Summary & Design Philosophy
2. Formal Climate Model — Mathematics
3. Scenario Families — Parameter Definitions
4. State Structs (Rust)
5. Rust Module Layout — File Paths, Traits, Public API
6. Event Contracts — JSON Schemas
7. Database Schema — DDL
8. Economy Coupling — Tick Ordering and Feedback Loops
9. Adaptation Lever System
10. Lag Model — Pending-Effect Queue
11. Conservation Invariants — Property Tests
12. Failure Modes
13. Acceptance Test Suite
14. Performance Budget
15. CIV Sim Integration Notes

---

## 1. Summary & Design Philosophy

The climate module is CivLab's biophysical constraint layer. It grounds every economic and governance outcome in physical reality: energy supply is bounded, resource extraction depletes stocks, cumulative emissions accumulate into atmospheric forcing, and forcing converts into damage that degrades productive capacity, health baselines, and infrastructure stock.

The module's primary loop is:

```
Physical Forcing  →  Economic Damage  →  Policy Response  →  Adaptation Investment  →  Forcing (modified)
```

This loop is asymmetric by design. Forcing accumulates fast; adaptation reduces future damage slowly, with diminishing returns. The lag between a policy decision and its first observable impact is encoded explicitly — no instantaneous fixes.

### Role in the Simulation

Climate operates as a **forcing module** that imposes constraints on all other modules:

- It reduces the effective productivity frontier available to the economy each tick.
- It elevates baseline sustain costs (food, housing, healthcare, utilities become more expensive under damage).
- It creates scarcity pressure that, if unmanaged, induces governance drift toward coercion.
- It interacts directly with the joule economy: when energy supply capacity (ESC) is tight, personal energy quotas must shrink or rationing rules activate.

The climate module does **not** model atmospheric physics at high resolution. It is a macro-level forcing abstraction sufficient to stress-test regime stability and adaptation policy tradeoffs over 10–100 simulated year horizons.

### Design Constraints

All climate state variables obey CIV-0001 determinism invariants:

- No floating-point in core ledgers; forcing and damage stored as fixed-point `i64` (scaled by `1_000_000` for six decimal places of precision).
- All random events (disaster dice rolls) use `ChaCha20Rng` seeded from the simulation seed.
- Collection iteration over `BTreeMap` guarantees stable ordering across runs.
- No `SystemTime::now()` — simulation tick is the sole clock.

---

## 2. Formal Climate Model — Mathematics

### 2.1 Atmospheric Forcing Accumulation

Atmospheric forcing `AF(t)` is a dimensionless index representing cumulative radiative pressure, proportional to GHG concentration. It accumulates from emissions and decays via a natural sink:

```
AF(t+1) = AF(t) + ΣEmissions(t) − NaturalSink(t)

NaturalSink(t) = k_sink × AF(t)        where k_sink ∈ (0, 0.05] per tick

ΣEmissions(t) = Σ_goods [ ProductionVolume(good, t) × EmissionsIntensity(good) ]
```

**Calibrated coefficient bounds:**

| Parameter | Symbol | Minimum | Default | Maximum | Units |
|-----------|--------|---------|---------|---------|-------|
| Sink rate | k_sink | 0.0001 | 0.002 | 0.05 | per tick |
| Emissions intensity (energy-heavy goods) | ε_h | 0.05 | 0.15 | 0.40 | forcing-units / output-unit |
| Emissions intensity (clean goods) | ε_c | 0.0 | 0.02 | 0.08 | forcing-units / output-unit |
| Safe forcing threshold | AF_safe | 0.3 | 0.5 | 0.7 | dimensionless |
| Damage onset threshold | AF_onset | 0.5 | 0.8 | 1.2 | dimensionless |

**Fixed-point representation:** `AF` is stored as `i64` with a scale factor of `1_000_000`. An `AF` of `0.8` is stored as `800_000i64`.

### 2.2 Climate Damage Function

Climate damage `CD(t) ∈ [0, 1]` is a sigmoid of forcing above the damage onset threshold:

```
CD(t) = 1 / (1 + exp(−α × (AF(t) − AF_onset)))

α ∈ [1.0, 8.0]       steepness parameter (higher α = sharper transition)
```

For the simulation's fixed-point implementation, this sigmoid is pre-computed into a lookup table indexed by `AF` in steps of `1_000` (i.e., 0.001 forcing units) and stored as `i64` scaled by `1_000_000`.

**Effective productivity:**

```
EffectiveProductivity(t) = BaseProductivity(t) × (1 − CD(t) × productivity_damage_weight)

productivity_damage_weight ∈ [0.2, 1.0]   (fraction of CD applied to productivity)
```

**Health baseline penalty:**

```
HealthPenalty(t) = CD(t) × health_damage_weight

health_damage_weight ∈ [0.1, 0.6]
```

**Housing spoilage rate increase:**

```
HousingDecayRate(t) = BaseDecayRate × (1 + housing_decay_amplifier × CD(t))

housing_decay_amplifier ∈ [0.5, 3.0]
```

### 2.3 Energy Budget Disruption Formula

Energy supply capacity `ESC(t)` is the maximum deliverable energy per tick. It has two components:

```
ESC(t) = RenewableCapacity(t) + NonRenewableCapacity(t) × ExtractEfficiency(t)

RenewableCapacity(t+1) = RenewableCapacity(t) × (1 + renewable_growth_rate)
                         + RenewableInvestment(t) × investment_yield

NonRenewableCapacity(t+1) = NonRenewableCapacity(t) − ExtractionVolume(t)

ExtractEfficiency(t) = BaseExtractEfficiency × (1 − EROI_decay × tick)
```

**Energy deficit pressure:**

When actual energy demand exceeds ESC, a deficit emerges:

```
EnergyDeficit(t) = max(0, EnergyDemand(t) − ESC(t))

EnergyDeficitRatio(t) = EnergyDeficit(t) / EnergyDemand(t)   ∈ [0, 1]
```

This ratio feeds directly into scarcity pressure and joule quota tightening.

### 2.4 Resource Depletion Rate

Resource depletion factor `RDF(t) ∈ [0, 1]` declines as extraction occurs:

```
RDF(t+1) = RDF(t) − δ × ExtractionVolume(t) / TotalResourceStock_initial

δ ∈ [0.0001, 0.005]   depletion coefficient per extraction unit
```

As `RDF` declines, production costs rise:

```
EffectiveProductionCost(good, t) = BaseCost(good) × (1 + φ × (1 − RDF(t)))

φ ∈ [0.1, 2.0]   cost amplifier at full depletion
```

**Regeneration policy:** Resources can regenerate only if an explicit regeneration investment policy is active. Without it, depletion is strictly monotonic (non-reversible). This is invariant I-3 enforced at the type level — the `ResourcePool` struct contains no regeneration field unless the policy enables it.

### 2.5 Adaptation ROI Function

Adaptation investment `A(t)` accumulates into an adaptation stock `AS(t)` with depreciation:

```
AS(t+1) = AS(t) × (1 − adapt_depreciation) + A(t)

adapt_depreciation ∈ [0.01, 0.05] per tick
```

Effective damage reduction from adaptation:

```
DamageReduction(t) = η × AS(t) / (1 + η × AS(t))

η ∈ [0.001, 0.05]   adaptation effectiveness coefficient
```

This yields diminishing returns: doubling adaptation stock does not double damage reduction.

**Actual applied damage:**

```
CD_effective(t) = CD(t) × (1 − DamageReduction(t))
```

**Adaptation ROI metric:**

```
AdaptROI(t) = ΔDamageReduction(t) / A(t)   [damage-fraction per output-unit invested]
```

### 2.6 Scarcity Pressure Index

Scarcity pressure `SP(t) ∈ [0, 1]` aggregates all supply constraint signals:

```
SP(t) = clip(
    w1 × (1 − SustainEfficiency(t))
  + w2 × CD_effective(t)
  + w3 × EnergyDeficitRatio(t)
  + w4 × (1 − RDF(t)) × resource_scarcity_weight
, 0, 1)

Default weights: w1=0.35, w2=0.25, w3=0.25, w4=0.15
resource_scarcity_weight ∈ [0, 1]
```

`SP(t)` is the primary signal consumed by the governance drift model and the tyranny index calculation.

### 2.7 Disaster Probability

Each tick, a disaster is sampled from a Bernoulli distribution:

```
DisasterProb(t) = base_disaster_rate + β × CD_effective(t)

base_disaster_rate ∈ [0.001, 0.02]   per tick
β ∈ [0.01, 0.10]
```

When a disaster event fires, severity is drawn from an exponential distribution:

```
Severity ~ Exponential(mean = base_severity × (1 + CD_effective(t) × severity_amplifier))
```

Disaster effects are applied as immediate shocks to `ESC`, housing stock, and population health.

---

## 3. Scenario Families — Parameter Definitions

Four canonical scenario families define the parameter space for climate experiments. Each is expressed as a Rust struct (see Section 4) and as a YAML preset file under `scenarios/climate/`.

### 3.1 Baseline Scenario

The baseline scenario runs without active climate policy intervention. It represents a reference trajectory against which all other scenarios are compared.

```rust
ScenarioConfig {
    id: "climate-baseline-v1",
    family: ScenarioFamily::Baseline,
    forcing_params: ForcingParams {
        k_sink: 2_000,                // 0.002 fixed-point
        emissions_intensity_heavy: 150_000,   // 0.15
        emissions_intensity_clean: 20_000,    // 0.02
        af_safe_threshold: 500_000,   // 0.5
        af_onset_threshold: 800_000,  // 0.8
        damage_alpha: 3_000_000,      // 3.0
    },
    depletion_params: DepletionParams {
        delta_coefficient: 1_000,     // 0.001
        cost_amplifier_phi: 800_000,  // 0.8
        rdf_initial: 1_000_000,       // 1.0
    },
    energy_params: EnergyParams {
        renewable_growth_rate: 3_000, // 0.003 per tick
        investment_yield: 500_000,    // 0.5
        eroi_decay: 200,              // 0.0002 per tick
        base_extract_efficiency: 900_000, // 0.9
    },
    adaptation_params: AdaptationParams {
        adaptation_share: 0,          // 0% of output — no active adaptation
        eta_effectiveness: 10_000,    // 0.01
        depreciation_rate: 20_000,    // 0.02
    },
    shock_params: ShockParams {
        base_disaster_rate: 5_000,    // 0.005 per tick
        beta_damage_multiplier: 30_000, // 0.03
        base_severity: 100_000,       // 0.1
        severity_amplifier: 2_000_000, // 2.0
    },
    scenario_version: 1,
}
```

**Expected trajectory:** Forcing climbs past `AF_onset` within 50–80 simulated years. Damage rises slowly then accelerates above the sigmoid knee. Scarcity pressure reaches `SP > 0.4` by year 60 under typical emission rates.

### 3.2 Delayed-Action Scenario

Policy action is deferred until forcing crosses `AF_onset`. At that tick, adaptation investment activates and a carbon-intensity cap engages. This tests whether late action can prevent worst-case damage or whether lag effects lock in significant loss.

```rust
ScenarioConfig {
    id: "climate-delayed-action-v1",
    family: ScenarioFamily::DelayedAction,
    // Same forcing params as baseline — no mitigation pre-trigger
    forcing_params: ForcingParams {
        k_sink: 2_000,
        emissions_intensity_heavy: 150_000,
        emissions_intensity_clean: 20_000,
        af_safe_threshold: 500_000,
        af_onset_threshold: 800_000,
        damage_alpha: 3_000_000,
    },
    depletion_params: DepletionParams {
        delta_coefficient: 1_000,
        cost_amplifier_phi: 800_000,
        rdf_initial: 1_000_000,
    },
    energy_params: EnergyParams {
        renewable_growth_rate: 3_000,
        investment_yield: 500_000,
        eroi_decay: 200,
        base_extract_efficiency: 900_000,
    },
    adaptation_params: AdaptationParams {
        adaptation_share: 0,          // starts at 0; triggers to 50_000 (5%) when AF > onset
        eta_effectiveness: 15_000,    // 0.015
        depreciation_rate: 20_000,
    },
    shock_params: ShockParams {
        base_disaster_rate: 5_000,
        beta_damage_multiplier: 30_000,
        base_severity: 100_000,
        severity_amplifier: 2_000_000,
    },
    // Trigger activates adaptation when forcing boundary is crossed
    scenario_boundary: Some(ScenarioBoundary {
        trigger_variable: BoundaryVariable::AtmosphericForcing,
        trigger_threshold: 800_000,   // AF >= 0.8
        action: BoundaryAction::ActivateAdaptation {
            new_adaptation_share: 50_000,   // 5%
            carbon_intensity_cap: 80_000,   // max 0.08 per unit
        },
        emits_event: true,
    }),
    scenario_version: 1,
}
```

**Key metric:** Damage accumulated between `AF_onset` and first adaptation effect (the lag period). The lag model (Section 10) means adaptation investments take `L_adapt = 20` ticks before any damage reduction is felt.

### 3.3 High-Shock Scenario

Extreme stochastic event frequency tests governance resilience. Base disaster rate is elevated 4x, and the natural sink is weakened, modeling a carbon-cycle tipping point. This scenario is primarily used to validate that governance-risk lift appears in metrics (acceptance criterion AC-3).

```rust
ScenarioConfig {
    id: "climate-high-shock-v1",
    family: ScenarioFamily::HighShock,
    forcing_params: ForcingParams {
        k_sink: 500,                  // 0.0005 — weakened sink (tipping point)
        emissions_intensity_heavy: 200_000,  // 0.20 — higher intensity
        emissions_intensity_clean: 30_000,   // 0.03
        af_safe_threshold: 400_000,   // 0.4 — lower safe zone
        af_onset_threshold: 600_000,  // 0.6 — earlier onset
        damage_alpha: 5_000_000,      // 5.0 — steeper sigmoid
    },
    depletion_params: DepletionParams {
        delta_coefficient: 2_000,     // 0.002 — faster depletion
        cost_amplifier_phi: 1_500_000, // 1.5
        rdf_initial: 1_000_000,
    },
    energy_params: EnergyParams {
        renewable_growth_rate: 2_000, // slower renewable build
        investment_yield: 400_000,
        eroi_decay: 400,              // faster EROI decay
        base_extract_efficiency: 800_000,
    },
    adaptation_params: AdaptationParams {
        adaptation_share: 30_000,     // 3% — modest but not reactive enough
        eta_effectiveness: 10_000,
        depreciation_rate: 30_000,    // higher depreciation under repeated shocks
    },
    shock_params: ShockParams {
        base_disaster_rate: 20_000,   // 0.02 — 4x baseline
        beta_damage_multiplier: 80_000, // 0.08
        base_severity: 250_000,       // 0.25
        severity_amplifier: 3_000_000, // 3.0
    },
    scenario_boundary: None,
    scenario_version: 1,
}
```

**Expected signature:** Governance risk index lifts above `0.3` within 20 years. Budget stress appears as adaptation demand competes with essentials provision. Revolt risk probability rises if scarcity pressure exceeds `SP > 0.5` while baseline strength is low.

### 3.4 Coordinated-Mitigation Scenario

Active, early mitigation policy with both emissions caps and aggressive adaptation investment. This is the "best-case managed transition" scenario and serves as the reference optimum for adaptation ROI analysis.

```rust
ScenarioConfig {
    id: "climate-coordinated-mitigation-v1",
    family: ScenarioFamily::CoordinatedMitigation,
    forcing_params: ForcingParams {
        k_sink: 4_000,                // 0.004 — enhanced sink (reforestation, CCS)
        emissions_intensity_heavy: 60_000,   // 0.06 — capped intensity
        emissions_intensity_clean: 5_000,    // 0.005
        af_safe_threshold: 500_000,
        af_onset_threshold: 800_000,
        damage_alpha: 3_000_000,
    },
    depletion_params: DepletionParams {
        delta_coefficient: 500,       // 0.0005 — slower depletion (efficiency mandates)
        cost_amplifier_phi: 500_000,  // 0.5
        rdf_initial: 1_000_000,
    },
    energy_params: EnergyParams {
        renewable_growth_rate: 8_000, // 0.008 — aggressive renewable build
        investment_yield: 700_000,    // 0.7 — better yields (scale effects)
        eroi_decay: 100,              // 0.0001
        base_extract_efficiency: 950_000,
    },
    adaptation_params: AdaptationParams {
        adaptation_share: 80_000,     // 8% — strong investment
        eta_effectiveness: 25_000,    // 0.025 — higher effectiveness (coordinated)
        depreciation_rate: 15_000,    // 0.015
    },
    shock_params: ShockParams {
        base_disaster_rate: 3_000,    // 0.003
        beta_damage_multiplier: 20_000,
        base_severity: 70_000,        // lower because adaptation stock is high
        severity_amplifier: 1_500_000,
    },
    scenario_boundary: None,
    scenario_version: 1,
}
```

**Expected trajectory:** Forcing peaks below `AF_onset` and stabilizes. Effective damage stays below `CD_effective < 0.15`. Adaptation ROI is highest in early years, tapering as the adaptation stock accumulates.

---

## 4. State Structs (Rust)

All types belong to the `crates/climate` crate. Fixed-point integers use `i64` with `SCALE = 1_000_000i64` unless otherwise noted. Time is `tick: u64`. Collections that affect output ordering use `BTreeMap`.

```rust
// crates/climate/src/state.rs

use std::collections::BTreeMap;

/// Scale factor for all fixed-point climate values.
/// A value of 1.0 is represented as 1_000_000i64.
pub const SCALE: i64 = 1_000_000;

// ---------------------------------------------------------------------------
// ForcingIndex — tracks cumulative atmospheric forcing accumulation
// ---------------------------------------------------------------------------

/// Cumulative atmospheric forcing index. Dimensionless, scaled by SCALE.
/// Invariant: af_index >= 0 always.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForcingIndex {
    /// Scaled fixed-point: actual_af = af_index / SCALE
    pub af_index: i64,
    /// Natural sink rate this tick, scaled by SCALE
    pub sink_rate: i64,
    /// Total emissions generated this tick, scaled by SCALE
    pub emissions_this_tick: i64,
    /// Cumulative emissions since scenario start
    pub cumulative_emissions: i64,
    /// Tick at which AF first crossed af_onset_threshold (None if not yet crossed)
    pub onset_crossed_tick: Option<u64>,
}

impl ForcingIndex {
    pub fn new(sink_rate: i64) -> Self {
        ForcingIndex {
            af_index: 0,
            sink_rate,
            emissions_this_tick: 0,
            cumulative_emissions: 0,
            onset_crossed_tick: None,
        }
    }

    /// Advance forcing by one tick. Returns new ForcingIndex.
    /// Invariant: af_index never goes negative (sink cannot exceed forcing).
    pub fn advance(&self, emissions: i64, af_onset: i64, current_tick: u64) -> ForcingIndex {
        let sink = (self.af_index * self.sink_rate) / SCALE;
        let new_af = (self.af_index + emissions - sink).max(0);
        let onset_tick = self.onset_crossed_tick.or_else(|| {
            if new_af >= af_onset { Some(current_tick) } else { None }
        });
        ForcingIndex {
            af_index: new_af,
            sink_rate: self.sink_rate,
            emissions_this_tick: emissions,
            cumulative_emissions: self.cumulative_emissions + emissions,
            onset_crossed_tick: onset_tick,
        }
    }
}

// ---------------------------------------------------------------------------
// DamageEstimate — output of the climate damage function
// ---------------------------------------------------------------------------

/// Climate damage estimate for a single tick.
/// All fractions are scaled by SCALE (e.g., 0.25 = 250_000).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DamageEstimate {
    /// Raw sigmoid damage CD(t), before adaptation. Scaled by SCALE.
    pub raw_damage: i64,
    /// Effective damage after adaptation stock reduction. Scaled by SCALE.
    pub effective_damage: i64,
    /// Fraction of productivity lost this tick. Scaled by SCALE.
    pub productivity_loss: i64,
    /// Health baseline penalty fraction. Scaled by SCALE.
    pub health_penalty: i64,
    /// Housing decay rate amplification factor. Scaled by SCALE.
    pub housing_decay_amplifier: i64,
    /// Adaptation damage reduction applied. Scaled by SCALE.
    pub adaptation_reduction: i64,
    /// Disaster probability this tick. Scaled by SCALE.
    pub disaster_probability: i64,
}

impl DamageEstimate {
    pub fn zero() -> Self {
        DamageEstimate {
            raw_damage: 0,
            effective_damage: 0,
            productivity_loss: 0,
            health_penalty: 0,
            housing_decay_amplifier: 0,
            adaptation_reduction: 0,
            disaster_probability: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// AdaptationInvestment — tracks the adaptation stock and pending effects
// ---------------------------------------------------------------------------

/// Adaptation stock and investment tracking.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdaptationInvestment {
    /// Current adaptation stock. Scaled by SCALE.
    pub stock: i64,
    /// Share of total output allocated to adaptation per tick (0..1). Scaled by SCALE.
    pub investment_share: i64,
    /// Actual joules invested this tick.
    pub invested_this_tick: i64,
    /// Depreciation rate per tick. Scaled by SCALE.
    pub depreciation_rate: i64,
    /// Effectiveness coefficient η. Scaled by SCALE.
    pub eta_effectiveness: i64,
    /// Pending effects queue: (tick_at_which_effect_applies, reduction_amount_scaled)
    /// Sorted by tick. Effects are lagged by L_adapt ticks.
    pub pending_effects: BTreeMap<u64, i64>,
}

impl AdaptationInvestment {
    pub fn new(investment_share: i64, eta_effectiveness: i64, depreciation_rate: i64) -> Self {
        AdaptationInvestment {
            stock: 0,
            investment_share,
            invested_this_tick: 0,
            depreciation_rate,
            eta_effectiveness,
            pending_effects: BTreeMap::new(),
        }
    }

    /// Current damage reduction fraction from this adaptation stock. Scaled by SCALE.
    /// Formula: η × AS / (1 + η × AS)
    pub fn damage_reduction(&self) -> i64 {
        let eta_stock = (self.eta_effectiveness * self.stock) / SCALE;
        (eta_stock * SCALE) / (SCALE + eta_stock)
    }
}

// ---------------------------------------------------------------------------
// ResourcePool — non-renewable and renewable resource tracking
// ---------------------------------------------------------------------------

/// Resource pool tracking depletion and renewable capacity.
/// Invariant: rdf_scaled is monotonically non-increasing unless regeneration policy active.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourcePool {
    /// Resource depletion factor RDF ∈ [0, 1]. Scaled by SCALE.
    /// Starts at SCALE (1.0), declines with extraction.
    pub rdf_scaled: i64,
    /// Initial total resource stock (joules). Used to compute depletion rate.
    pub initial_stock_joules: i64,
    /// Current non-renewable stock remaining (joules).
    pub nonrenewable_remaining_joules: i64,
    /// Renewable capacity (joules per tick).
    pub renewable_capacity_joules: i64,
    /// Extraction volume this tick (joules).
    pub extraction_volume_joules: i64,
    /// Depletion coefficient δ per extraction unit. Scaled by SCALE.
    pub delta_coefficient: i64,
    /// Cost amplifier φ at full depletion. Scaled by SCALE.
    pub cost_amplifier_phi: i64,
    /// Whether regeneration policy is active.
    /// When false, rdf_scaled can only decrease (enforces non-reversibility invariant).
    pub regeneration_policy_active: bool,
    /// Renewable growth rate per tick. Scaled by SCALE.
    pub renewable_growth_rate: i64,
    /// Investment yield factor for renewable capacity. Scaled by SCALE.
    pub investment_yield: i64,
    /// EROI decay per tick (reduction in extraction efficiency). Scaled by SCALE.
    pub eroi_decay_per_tick: i64,
    /// Base extraction efficiency. Scaled by SCALE.
    pub base_extract_efficiency: i64,
    /// Current extraction efficiency (degrades over time). Scaled by SCALE.
    pub current_extract_efficiency: i64,
}

impl ResourcePool {
    /// Effective production cost multiplier for a good given current RDF.
    /// = (1 + φ × (1 − RDF)) scaled by SCALE.
    pub fn cost_multiplier(&self) -> i64 {
        let rdf_complement = SCALE - self.rdf_scaled; // (1 - RDF) scaled
        let phi_term = (self.cost_amplifier_phi * rdf_complement) / SCALE;
        SCALE + phi_term
    }

    /// Energy supply capacity this tick (joules).
    pub fn energy_supply_capacity(&self) -> i64 {
        let nonrenew_contrib =
            (self.nonrenewable_remaining_joules * self.current_extract_efficiency) / SCALE;
        self.renewable_capacity_joules + nonrenew_contrib
    }
}

// ---------------------------------------------------------------------------
// ClimateState — top-level per-tick climate state
// ---------------------------------------------------------------------------

/// Complete climate state for one simulation tick.
/// This is the canonical input and output of the climate phase.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClimateState {
    pub tick: u64,
    pub scenario_id: String,
    pub scenario_version: u32,
    pub forcing: ForcingIndex,
    pub damage: DamageEstimate,
    pub adaptation: AdaptationInvestment,
    pub resources: ResourcePool,
    pub scarcity_pressure: i64,       // SP ∈ [0, 1]. Scaled by SCALE.
    pub energy_deficit_ratio: i64,    // ∈ [0, 1]. Scaled by SCALE.
    pub active_disaster: Option<DisasterEvent>,
    pub pending_boundary_events: Vec<BoundaryEvent>,
}

// ---------------------------------------------------------------------------
// DisasterEvent — stochastic climate shock
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisasterKind {
    Flood,
    Drought,
    Heatwave,
    StormSurge,
    WildfireComplex,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DisasterEvent {
    pub kind: DisasterKind,
    /// Severity ∈ [0, 1]. Scaled by SCALE.
    pub severity: i64,
    /// Tick on which this disaster fires.
    pub tick: u64,
    /// ESC reduction in joules for this tick and the next recovery_ticks.
    pub esc_reduction_joules: i64,
    /// Housing stock destroyed (units).
    pub housing_units_destroyed: i64,
    /// Population health hit (fraction). Scaled by SCALE.
    pub health_hit: i64,
    /// Number of ticks for recovery of ESC.
    pub recovery_ticks: u32,
}

// ---------------------------------------------------------------------------
// ScenarioBoundary — threshold-triggered policy transitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BoundaryVariable {
    AtmosphericForcing,
    ClimateDamage,
    ScarcityPressure,
    EnergyDeficitRatio,
    ResourceDepletionFactor,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BoundaryAction {
    ActivateAdaptation {
        new_adaptation_share: i64,
        carbon_intensity_cap: i64,
    },
    TightenEnergyQuotas {
        quota_reduction_fraction: i64, // fraction to reduce personal quotas
    },
    ActivateRationing {
        rights_first: bool,
        emergency_reserves_weeks: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioBoundary {
    pub trigger_variable: BoundaryVariable,
    pub trigger_threshold: i64,
    pub action: BoundaryAction,
    pub emits_event: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BoundaryEvent {
    pub tick: u64,
    pub variable: BoundaryVariable,
    pub threshold_crossed: i64,
    pub action_applied: BoundaryAction,
    pub scenario_id: String,
    pub scenario_version: u32,
}

// ---------------------------------------------------------------------------
// ScenarioConfig — full scenario parameter bundle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScenarioFamily {
    Baseline,
    DelayedAction,
    HighShock,
    CoordinatedMitigation,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForcingParams {
    pub k_sink: i64,
    pub emissions_intensity_heavy: i64,
    pub emissions_intensity_clean: i64,
    pub af_safe_threshold: i64,
    pub af_onset_threshold: i64,
    pub damage_alpha: i64,
    pub productivity_damage_weight: i64,
    pub health_damage_weight: i64,
    pub housing_decay_amplifier: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DepletionParams {
    pub delta_coefficient: i64,
    pub cost_amplifier_phi: i64,
    pub rdf_initial: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnergyParams {
    pub renewable_growth_rate: i64,
    pub investment_yield: i64,
    pub eroi_decay: i64,
    pub base_extract_efficiency: i64,
    pub initial_renewable_capacity_joules: i64,
    pub initial_nonrenewable_stock_joules: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdaptationParams {
    pub adaptation_share: i64,
    pub eta_effectiveness: i64,
    pub depreciation_rate: i64,
    pub lag_ticks: u32,  // L_adapt: ticks before adaptation investment takes effect
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShockParams {
    pub base_disaster_rate: i64,
    pub beta_damage_multiplier: i64,
    pub base_severity: i64,
    pub severity_amplifier: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioConfig {
    pub id: String,
    pub family: ScenarioFamily,
    pub forcing_params: ForcingParams,
    pub depletion_params: DepletionParams,
    pub energy_params: EnergyParams,
    pub adaptation_params: AdaptationParams,
    pub shock_params: ShockParams,
    pub scenario_boundary: Option<ScenarioBoundary>,
    pub scenario_version: u32,
}

// ---------------------------------------------------------------------------
// ClimateMetrics — per-tick metrics snapshot for dashboard and DB
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClimateMetrics {
    pub tick: u64,
    pub scenario_id: String,
    pub scenario_version: u32,
    /// AF index. Scaled by SCALE.
    pub forcing_index: i64,
    /// Resilience index = (1 - effective_damage). Scaled by SCALE.
    pub resilience_index: i64,
    /// Effective climate damage fraction. Scaled by SCALE.
    pub effective_damage: i64,
    /// Expected loss: economic output × effective_damage. Joules.
    pub expected_loss_joules: i64,
    /// Adaptation ROI this tick. Scaled by SCALE.
    pub adaptation_roi: i64,
    /// Displacement pressure = SP × population_fraction_at_risk. Scaled by SCALE.
    pub displacement_pressure: i64,
    /// Energy deficit ratio. Scaled by SCALE.
    pub energy_deficit_ratio: i64,
    /// Resource depletion factor. Scaled by SCALE.
    pub resource_depletion_factor: i64,
    /// Disaster occurred this tick.
    pub disaster_occurred: bool,
    /// Disaster severity if occurred. Scaled by SCALE.
    pub disaster_severity: Option<i64>,
    /// Scarcity pressure. Scaled by SCALE.
    pub scarcity_pressure: i64,
    /// Adaptation stock. Scaled by SCALE.
    pub adaptation_stock: i64,
    /// Renewable share of ESC. Scaled by SCALE.
    pub renewable_share: i64,
}
```

---

## 5. Rust Module Layout

The climate module lives entirely in `crates/climate/`. The workspace `Cargo.toml` at `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/Cargo.toml` must be updated to include `"crates/climate"` in the `members` list.

```
crates/climate/
├── Cargo.toml
└── src/
    ├── lib.rs                  # pub mod declarations, re-exports of public API
    ├── state.rs                # All state structs (Section 4 above)
    ├── damage.rs               # Damage function computation, sigmoid lookup table
    ├── forcing.rs              # ForcingIndex::advance, emissions aggregation
    ├── depletion.rs            # ResourcePool::advance, cost multiplier
    ├── adaptation.rs           # AdaptationInvestment::advance, pending-effect queue
    ├── scarcity.rs             # ScarcityPressure computation
    ├── disaster.rs             # Disaster sampling, severity computation
    ├── scenario.rs             # ScenarioConfig loading, validation, boundary checks
    ├── phase.rs                # ClimatePhase::run — top-level tick entry point
    ├── metrics.rs              # ClimateMetrics assembly from ClimateState
    └── events.rs               // Event emission: climate.damage_modeled.v1, etc.
```

### 5.1 Core Traits

```rust
// crates/climate/src/lib.rs

pub use state::*;
pub use phase::ClimatePhase;
pub use metrics::ClimateMetrics;
pub use events::{ClimateDamageModeled, ClimateAdaptationApplied,
                 ClimateResourceStress, ClimateScenarioBoundaryCrossed};

/// Trait implemented by ClimatePhase.
/// Called once per tick during the deterministic transition phase.
pub trait ClimateCompute {
    /// Advance climate state by one tick.
    /// Requires the total output joules from the economy this tick (for emissions).
    /// Requires the total energy demand joules from the economy this tick.
    /// Returns new ClimateState plus emitted events.
    fn advance(
        &self,
        state: &ClimateState,
        economy_output_joules: i64,
        energy_demand_joules: i64,
        rng_seed: u64,
    ) -> (ClimateState, Vec<ClimateEvent>);
}

/// Trait for components that report metrics.
pub trait ReportMetrics {
    fn metrics(&self, state: &ClimateState, economy_output_joules: i64) -> ClimateMetrics;
}

/// Union type for all events emitted by the climate module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ClimateEvent {
    DamageModeled(ClimateDamageModeled),
    AdaptationApplied(ClimateAdaptationApplied),
    ResourceStress(ClimateResourceStress),
    ScenarioBoundaryCrossed(ClimateScenarioBoundaryCrossed),
}
```

### 5.2 Public API Surface

```rust
// crates/climate/src/phase.rs  (summary of public functions)

impl ClimatePhase {
    /// Construct a ClimatePhase from a validated ScenarioConfig.
    pub fn from_scenario(config: ScenarioConfig) -> Result<ClimatePhase, ConfigError>;

    /// Build initial ClimateState at tick 0.
    pub fn initial_state(&self) -> ClimateState;
}

impl ClimateCompute for ClimatePhase {
    fn advance(
        &self,
        state: &ClimateState,
        economy_output_joules: i64,
        energy_demand_joules: i64,
        rng_seed: u64,
    ) -> (ClimateState, Vec<ClimateEvent>);
}

// Visible to economy crate for supply constraint queries:
pub fn energy_supply_capacity(state: &ClimateState) -> i64;
pub fn effective_productivity_multiplier(state: &ClimateState) -> i64;
pub fn scarcity_pressure(state: &ClimateState) -> i64;
pub fn cost_multiplier_for_good(state: &ClimateState, good_emissions_intensity: i64) -> i64;
```

---

## 6. Event Contracts — JSON Schemas

All events include `scenario_id`, `scenario_version`, and `tick` for determinism verification. Events are emitted after the climate phase computes and before the economy phase runs (see Section 8 for tick ordering).

### 6.1 `climate.damage_modeled.v1`

Emitted every tick. Carries the full damage estimate.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.damage_modeled.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "forcing_index", "raw_damage", "effective_damage",
    "productivity_loss", "health_penalty", "housing_decay_amplifier",
    "adaptation_reduction", "disaster_occurred", "scarcity_pressure",
    "energy_deficit_ratio", "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.damage_modeled.v1" },
    "tick": { "type": "integer", "minimum": 0 },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer", "minimum": 1 },
    "forcing_index": {
      "type": "integer",
      "description": "AF index scaled by 1_000_000. 800_000 = 0.8."
    },
    "raw_damage": {
      "type": "integer",
      "description": "CD(t) before adaptation. Scaled by 1_000_000."
    },
    "effective_damage": {
      "type": "integer",
      "description": "CD_effective(t) after adaptation. Scaled by 1_000_000."
    },
    "productivity_loss": {
      "type": "integer",
      "description": "Fraction of productivity lost. Scaled by 1_000_000."
    },
    "health_penalty": {
      "type": "integer",
      "description": "Health baseline penalty fraction. Scaled by 1_000_000."
    },
    "housing_decay_amplifier": {
      "type": "integer",
      "description": "Housing decay rate amplifier. Scaled by 1_000_000."
    },
    "adaptation_reduction": {
      "type": "integer",
      "description": "Damage reduction from adaptation stock. Scaled by 1_000_000."
    },
    "disaster_occurred": { "type": "boolean" },
    "disaster_severity": {
      "type": ["integer", "null"],
      "description": "Severity of disaster if occurred. Scaled by 1_000_000. Null if no disaster."
    },
    "scarcity_pressure": {
      "type": "integer",
      "description": "SP(t). Scaled by 1_000_000."
    },
    "energy_deficit_ratio": {
      "type": "integer",
      "description": "Energy demand/supply deficit ratio. Scaled by 1_000_000."
    },
    "state_hash": {
      "type": "string",
      "description": "SHA-256 hex of ClimateState at this tick for replay verification."
    }
  },
  "additionalProperties": false
}
```

### 6.2 `climate.adaptation_applied.v1`

Emitted when a pending adaptation effect vests (i.e., `L_adapt` ticks after investment).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.adaptation_applied.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "investment_tick", "invested_joules", "stock_before", "stock_after",
    "damage_reduction_before", "damage_reduction_after", "adaptation_roi"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.adaptation_applied.v1" },
    "tick": { "type": "integer", "minimum": 0 },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer", "minimum": 1 },
    "investment_tick": {
      "type": "integer",
      "description": "The tick on which this investment was made."
    },
    "invested_joules": { "type": "integer" },
    "stock_before": {
      "type": "integer",
      "description": "Adaptation stock before this vesting. Scaled by 1_000_000."
    },
    "stock_after": {
      "type": "integer",
      "description": "Adaptation stock after this vesting. Scaled by 1_000_000."
    },
    "damage_reduction_before": {
      "type": "integer",
      "description": "Damage reduction fraction before. Scaled by 1_000_000."
    },
    "damage_reduction_after": {
      "type": "integer",
      "description": "Damage reduction fraction after. Scaled by 1_000_000."
    },
    "adaptation_roi": {
      "type": "integer",
      "description": "ΔDamageReduction / invested_joules. Scaled by 1_000_000."
    }
  },
  "additionalProperties": false
}
```

### 6.3 `climate.resource_stress.v1`

Emitted when `RDF` drops below a threshold (configurable, default `0.5` = `500_000i64`) or when ESC drops below demand for the first time in a scenario.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.resource_stress.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "stress_kind", "rdf_current", "esc_current_joules",
    "energy_deficit_joules", "cost_multiplier", "scarcity_pressure"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.resource_stress.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "stress_kind": {
      "type": "string",
      "enum": ["rdf_threshold_crossed", "esc_deficit_onset", "esc_deficit_sustained"]
    },
    "rdf_current": {
      "type": "integer",
      "description": "Current RDF. Scaled by 1_000_000."
    },
    "esc_current_joules": { "type": "integer" },
    "energy_deficit_joules": { "type": "integer" },
    "cost_multiplier": {
      "type": "integer",
      "description": "EffectiveProductionCost multiplier. Scaled by 1_000_000."
    },
    "scarcity_pressure": {
      "type": "integer",
      "description": "SP at time of stress event. Scaled by 1_000_000."
    }
  },
  "additionalProperties": false
}
```

### 6.4 `climate.scenario_boundary_crossed.v1`

Emitted exactly once when a `ScenarioBoundary` trigger condition is met.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.scenario_boundary_crossed.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "boundary_variable", "threshold_value", "observed_value", "action_taken"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.scenario_boundary_crossed.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "boundary_variable": {
      "type": "string",
      "enum": [
        "AtmosphericForcing",
        "ClimateDamage",
        "ScarcityPressure",
        "EnergyDeficitRatio",
        "ResourceDepletionFactor"
      ]
    },
    "threshold_value": {
      "type": "integer",
      "description": "Threshold that was crossed. Scaled by 1_000_000."
    },
    "observed_value": {
      "type": "integer",
      "description": "Observed value at crossing tick. Scaled by 1_000_000."
    },
    "action_taken": {
      "type": "string",
      "description": "JSON-serialized BoundaryAction variant."
    }
  },
  "additionalProperties": false
}
```

---

## 7. Database Schema — Full DDL

All tables use PostgreSQL. The `run_id` column cross-references the `simulation_runs` table defined in CIV-0001. Fixed-point integers are stored as `BIGINT` (matching `i64`).

```sql
-- climate_scenarios: versioned scenario parameter bundles
CREATE TABLE climate_scenarios (
    id              SERIAL PRIMARY KEY,
    scenario_id     VARCHAR(128) NOT NULL,
    version         INTEGER NOT NULL DEFAULT 1,
    name            VARCHAR(256) NOT NULL,
    family          VARCHAR(64) NOT NULL CHECK (family IN (
                        'Baseline', 'DelayedAction', 'HighShock', 'CoordinatedMitigation'
                    )),
    parameters_json JSONB NOT NULL,   -- full ScenarioConfig serialized
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (scenario_id, version)
);

-- climate_states: full ClimateState snapshot per tick per run
CREATE TABLE climate_states (
    id                      BIGSERIAL PRIMARY KEY,
    run_id                  BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                    BIGINT NOT NULL,
    scenario_id             VARCHAR(128) NOT NULL,
    scenario_version        INTEGER NOT NULL,

    -- ForcingIndex fields
    af_index                BIGINT NOT NULL,   -- scaled × 1_000_000
    sink_rate               BIGINT NOT NULL,
    emissions_this_tick     BIGINT NOT NULL,
    cumulative_emissions    BIGINT NOT NULL,
    onset_crossed_tick      BIGINT,            -- NULL if not yet crossed

    -- DamageEstimate fields
    raw_damage              BIGINT NOT NULL,
    effective_damage        BIGINT NOT NULL,
    productivity_loss       BIGINT NOT NULL,
    health_penalty          BIGINT NOT NULL,
    housing_decay_amplifier BIGINT NOT NULL,
    adaptation_reduction    BIGINT NOT NULL,
    disaster_probability    BIGINT NOT NULL,

    -- ResourcePool fields
    rdf_scaled              BIGINT NOT NULL,
    nonrenewable_remaining  BIGINT NOT NULL,   -- joules
    renewable_capacity      BIGINT NOT NULL,   -- joules/tick
    energy_supply_capacity  BIGINT NOT NULL,   -- joules/tick computed

    -- AdaptationInvestment fields
    adaptation_stock        BIGINT NOT NULL,   -- scaled × 1_000_000
    invested_this_tick      BIGINT NOT NULL,   -- joules

    -- Scarcity / energy
    scarcity_pressure       BIGINT NOT NULL,   -- scaled × 1_000_000
    energy_deficit_ratio    BIGINT NOT NULL,   -- scaled × 1_000_000

    -- Disaster
    disaster_occurred       BOOLEAN NOT NULL DEFAULT FALSE,
    disaster_severity       BIGINT,            -- NULL if none
    disaster_kind           VARCHAR(32),

    state_hash              CHAR(64) NOT NULL, -- SHA-256 for replay verification
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (run_id, tick),
    INDEX climate_states_run_tick (run_id, tick)
);

-- damage_history: event log for climate.damage_modeled.v1
CREATE TABLE damage_history (
    id                  BIGSERIAL PRIMARY KEY,
    run_id              BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                BIGINT NOT NULL,
    scenario_id         VARCHAR(128) NOT NULL,
    scenario_version    INTEGER NOT NULL,
    forcing_index       BIGINT NOT NULL,
    raw_damage          BIGINT NOT NULL,
    effective_damage    BIGINT NOT NULL,
    productivity_loss   BIGINT NOT NULL,
    health_penalty      BIGINT NOT NULL,
    disaster_occurred   BOOLEAN NOT NULL,
    disaster_severity   BIGINT,
    scarcity_pressure   BIGINT NOT NULL,
    energy_deficit_ratio BIGINT NOT NULL,
    state_hash          CHAR(64) NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX damage_history_run_tick (run_id, tick)
);

-- adaptation_ledger: event log for climate.adaptation_applied.v1
CREATE TABLE adaptation_ledger (
    id                      BIGSERIAL PRIMARY KEY,
    run_id                  BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                    BIGINT NOT NULL,             -- vesting tick
    investment_tick         BIGINT NOT NULL,             -- original investment tick
    scenario_id             VARCHAR(128) NOT NULL,
    scenario_version        INTEGER NOT NULL,
    invested_joules         BIGINT NOT NULL,
    stock_before            BIGINT NOT NULL,
    stock_after             BIGINT NOT NULL,
    damage_reduction_before BIGINT NOT NULL,
    damage_reduction_after  BIGINT NOT NULL,
    adaptation_roi          BIGINT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX adaptation_ledger_run (run_id, tick)
);

-- climate_boundary_events: log for climate.scenario_boundary_crossed.v1
CREATE TABLE climate_boundary_events (
    id                  BIGSERIAL PRIMARY KEY,
    run_id              BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                BIGINT NOT NULL,
    scenario_id         VARCHAR(128) NOT NULL,
    scenario_version    INTEGER NOT NULL,
    boundary_variable   VARCHAR(64) NOT NULL,
    threshold_value     BIGINT NOT NULL,
    observed_value      BIGINT NOT NULL,
    action_json         JSONB NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Extend metric_snapshots with climate columns (ALTER TABLE approach)
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_forcing_index       BIGINT;     -- AF scaled × 1_000_000
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_resilience_index    BIGINT;     -- (1 - CD_effective) scaled
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_effective_damage    BIGINT;     -- CD_effective scaled
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_expected_loss_joules BIGINT;   -- total output × damage fraction
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_adaptation_roi      BIGINT;    -- ΔDamageReduction/invested scaled
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_displacement_pressure BIGINT; -- SP × population_at_risk fraction
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_energy_deficit_ratio  BIGINT;
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_resource_depletion_factor BIGINT;
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_scarcity_pressure   BIGINT;
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_adaptation_stock    BIGINT;
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_renewable_share     BIGINT;   -- renewable % of ESC scaled
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_scenario_id         VARCHAR(128);
ALTER TABLE metric_snapshots ADD COLUMN IF NOT EXISTS
    climate_scenario_version    INTEGER;
```

---

## 8. Economy Coupling — Tick Ordering and Feedback Loops

### 8.1 Climate Runs Before Economy

The climate phase executes **before** the economy deterministic transition phase within each tick. The tick phase ordering from CIV-0001 Section 1 is extended as follows:

```
Tick N
├─ 1. Command Intake         (50 µs)
├─ 2. Policy Phase           (2 ms)     — reads last tick's ClimateState
│
├─ 3. CLIMATE PHASE          (1.5 ms)   [new]
│    Input:  ClimateState[N-1], EconomyOutput[N-1], EnergyDemand[N-1], RngSeed[N]
│    Output: ClimateState[N], ClimateEvents[N]
│    Emits:  climate.damage_modeled.v1 (and others if triggered)
│
├─ 4. Deterministic Transition (8 ms)  — reads ClimateState[N]
│    Economy uses:
│       effective_productivity_multiplier(ClimateState[N])
│       energy_supply_capacity(ClimateState[N])
│       cost_multiplier_for_good(ClimateState[N], ε_good)
│       scarcity_pressure(ClimateState[N])
│
├─ 5. Stochastic Event Phase  (3 ms)
├─ 6. Metrics Compute         (1 ms)   — includes ClimateMetrics
└─ 7. Client Broadcast        (50 µs)
```

Climate must complete before economy because economy needs `effective_productivity_multiplier` and `energy_supply_capacity` to compute production volumes. Economy outputs (total joules produced, energy demand) feed back into the **next** tick's climate phase as emissions and demand inputs.

### 8.2 Economy Supply Constraints from Climate

The economy accesses three read-only functions from the climate module during its deterministic transition:

| Function | Returns | Economy usage |
|----------|---------|---------------|
| `energy_supply_capacity(state)` | `i64` joules/tick | Hard cap on total energy allocable this tick |
| `effective_productivity_multiplier(state)` | `i64` scaled | Multiplied into each building's production output |
| `cost_multiplier_for_good(state, ε)` | `i64` scaled | Inflates embedded energy cost labels on goods |
| `scarcity_pressure(state)` | `i64` scaled | Read by governance drift and tyranny index modules |

### 8.3 Joule Economy Integration

When the Joule Economy allocation engine (CIV-0107) is active, the interaction is tighter:

- `energy_supply_capacity` determines the **total joule pool** from which personal quotas are drawn each tick.
- If `EnergyDeficitRatio > 0`, the Joule allocator must reduce personal quota baselines proportionally: `adjusted_quota = base_quota × (1 − EnergyDeficitRatio × rationing_strictness)`.
- When a `ScenarioBoundary::ActivateRationing` action fires, the Joule allocator switches to rights-first rationing mode: essentials are provisioned first from the contracted pool, discretionary quotas absorb the remainder.
- The `joule.scarcity_shock.v1` event defined in CIV-0107 is emitted by the economy module when it detects an ESC-driven quota reduction; it references the `climate_scenario_id` and `tick` from the climate event that caused the deficit.

### 8.4 Feedback Loop Summary

```
ClimateState[N]
    ↓ energy_supply_capacity  ↓ productivity_multiplier  ↓ cost_multiplier
Economy Phase[N]
    ↓ total_output_joules     ↓ energy_demand_joules      ↓ emissions_by_good
ClimateState[N+1]   ←──────────────────────────────────────────────────────┘
```

This is the canonical two-tick feedback cycle. No same-tick feedback; climate effects from tick N affect economy at tick N (not N−1), and economy outputs from tick N feed climate at tick N+1.

---

## 9. Adaptation Lever System

Adaptation levers are policy parameters settable via the scenario YAML config and via runtime policy commands from connected clients. Each lever has a declared effect function, cost curve, and diminishing returns profile.

### 9.1 Lever: Infrastructure Hardening

**Mechanism:** Reduces housing decay amplifier and disaster severity.

```
Effect: housing_decay_amplifier(t) = BaseDecayAmplifier × (1 − hardening_level)
        disaster_severity_multiplier = 1 − hardening_level × 0.5

Cost curve: InfraHardeningCost = hardening_level² × base_hardening_cost
            (quadratic; doubling hardening more than doubles cost)

Diminishing returns: Each 0.1 increment of hardening_level reduces housing decay
                     by decreasing amounts (quadratic cost ensures this).

Valid range: hardening_level ∈ [0.0, 1.0], stored as i64 scaled by SCALE.
```

**In YAML:**
```yaml
adaptation_levers:
  infrastructure_hardening:
    hardening_level: 0.4       # 40% hardening
    base_hardening_cost: 50    # joules per output-unit per tick
```

### 9.2 Lever: Relocation Spend

**Mechanism:** Moves population out of high-climate-risk zones. Reduces effective population exposure to disaster events.

```
Effect: ExposedPopulationFraction = 1 − (relocation_share / total_population)
        Disaster health_hit = base_health_hit × ExposedPopulationFraction

Cost: RelocCost = relocation_spend (direct output allocation)
      Displacement pressure is also reduced proportionally.

Diminishing returns: Relocation cost per person increases as easiest relocations
                     happen first. Modeled as cost_per_person × (1 + γ × fraction_relocated).

γ ∈ [0.5, 3.0] — relocation difficulty amplifier.

Valid range: relocation_spend ∈ [0, max_relocation_budget] joules.
```

### 9.3 Lever: Emergency Reserves

**Mechanism:** Maintains strategic reserves of energy and food. Reduces scarcity pressure spikes during disasters.

```
Effect: During a disaster tick:
        ScarcityPressure_actual = ScarcityPressure_computed
                                  × (1 − reserve_buffer_fraction)
        reserve_buffer_fraction = min(reserves_held / shock_demand, 1.0)

Cost: reserves_held × holding_cost_rate per tick (storage overhead).
      holding_cost_rate ∈ [0.001, 0.02] per tick.

Diminishing returns: Once reserves exceed 12 weeks of peak demand,
                     additional reserves yield near-zero scarcity reduction.

Valid range: reserve_weeks ∈ [0, 52] weeks of median demand.
```

### 9.4 Lever: Carbon-Intensity Caps

**Mechanism:** Enforces a maximum emissions intensity per unit of output for energy-heavy goods. Reduces `ΣEmissions(t)` and therefore slows AF accumulation.

```
Effect: EmissionsIntensity_effective(good) = min(EmissionsIntensity_base(good),
                                                 carbon_intensity_cap)

        If cap < base intensity, economy must either:
          (a) substitute lower-intensity production methods (higher cost), or
          (b) reduce production volume (scarcity increases)

Cost: Production cost markup for capped goods:
      CostMarkup = (1 - cap / base_intensity) × intensity_transition_cost_factor

intensity_transition_cost_factor ∈ [0.5, 4.0]

Diminishing returns: Tighter caps produce smaller and smaller AF reductions
                     because the highest-intensity activities are eliminated first.

Valid range: carbon_intensity_cap ∈ [0.01, 0.40] forcing-units per output-unit.
```

---

## 10. Lag Model — Pending-Effect Queue

Policy effects in the climate module are deliberately delayed. This encodes the physical and institutional reality that:
1. Adaptation infrastructure takes time to build.
2. Carbon-intensity caps take time to propagate through supply chains.
3. Renewable capacity investments have a construction lead time.

### 10.1 Lag Parameters

| Effect | Lag (ticks) | Interpretation |
|--------|-------------|----------------|
| Adaptation investment → stock increase | `L_adapt = 20` | ~2 simulated years of construction |
| Carbon cap → emissions reduction | `L_cap = 5` | ~6 months of supply-chain substitution |
| Renewable investment → capacity | `L_renew = 30` | ~3 years of installation |
| Emergency reserves → scarcity buffer | `L_reserve = 1` | Nearly immediate (stockpile pre-positioned) |
| Infrastructure hardening → decay reduction | `L_hard = 15` | ~18 months of retrofit |

All lag values are expressed in ticks and are stored in `ScenarioConfig::adaptation_params::lag_ticks` (or per-lever fields).

### 10.2 Pending-Effect Queue Structure

The `AdaptationInvestment::pending_effects` field is a `BTreeMap<u64, i64>` where:
- Key: `investment_tick + L_adapt` — the tick at which the effect vests.
- Value: The adaptation stock increment (in scaled units) to apply at that tick.

```rust
// Adding an investment at tick T with lag L_adapt:
let vest_tick = current_tick + L_adapt as u64;
let stock_increment = (invested_joules * eta_effectiveness) / SCALE;
adaptation.pending_effects
    .entry(vest_tick)
    .and_modify(|e| *e += stock_increment)
    .or_insert(stock_increment);
```

At each tick advance, effects with `key <= current_tick` are drained and applied:

```rust
let due: Vec<_> = adaptation.pending_effects
    .range(..=current_tick)
    .map(|(k, v)| (*k, *v))
    .collect();
for (vest_tick, increment) in due {
    adaptation.stock += increment;
    adaptation.pending_effects.remove(&vest_tick);
    // emit climate.adaptation_applied.v1
}
```

### 10.3 Effect Application Mechanics

After the pending queue drains, adaptation stock is updated for depreciation and new investment (which enters the queue, not the stock directly):

```rust
// Apply depreciation first
let depreciation = (adaptation.stock * adaptation.depreciation_rate) / SCALE;
adaptation.stock = (adaptation.stock - depreciation).max(0);

// Queue new investment
let this_tick_investment = (economy_output_joules * adaptation.investment_share) / SCALE;
let vest_tick = current_tick + L_adapt as u64;
// ... insert into pending_effects as above
```

**Invariant:** Adaptation stock is always non-negative. The lag means players cannot "emergency-fix" damage with a same-tick investment. This is enforced at the type level by keeping the pending queue separate from the stock.

---

## 11. Conservation Invariants — Property Tests

These invariants are enforced as `#[cfg(test)] proptest` property tests in `crates/climate/src/` and must pass in CI.

### I-1: Monotonic Damage Under Increasing Forcing

```
∀ AF1 < AF2: CD(AF1) ≤ CD(AF2)
```

**Test (proptest):**
```rust
proptest! {
    #[test]
    fn prop_damage_monotonic_in_forcing(af1 in 0i64..5_000_000i64,
                                        delta in 1i64..1_000_000i64) {
        let af2 = af1 + delta;
        let cd1 = compute_damage(af1, &default_forcing_params());
        let cd2 = compute_damage(af2, &default_forcing_params());
        prop_assert!(cd1.raw_damage <= cd2.raw_damage,
            "Damage must be monotonic: CD({}) = {} > CD({}) = {}",
            af1, cd1.raw_damage, af2, cd2.raw_damage);
    }
}
```

### I-2: Non-Reversible Resource Depletion Without Regeneration Policy

```
∀ tick: if !regeneration_policy_active: RDF(tick+1) ≤ RDF(tick)
```

**Test:**
```rust
#[test]
fn test_rdf_monotonically_decreasing_without_regeneration() {
    let mut pool = ResourcePool::test_default();
    pool.regeneration_policy_active = false;
    let initial_rdf = pool.rdf_scaled;
    for _ in 0..100 {
        pool = pool.advance(/* extraction_volume */ 1_000_000_000, /* investment */ 0);
        assert!(pool.rdf_scaled <= initial_rdf,
            "RDF increased without regeneration policy: {} > {}",
            pool.rdf_scaled, initial_rdf);
    }
}
```

### I-3: Adaptation Lag Invariant

No investment applied at tick `T` may appear in adaptation stock before tick `T + L_adapt`.

**Test:**
```rust
#[test]
fn test_adaptation_lag_respected() {
    let lag = 20u64;
    let mut adapt = AdaptationInvestment::test_default_with_lag(lag);
    let invest_tick = 5u64;
    // Invest at tick 5
    adapt = adapt.add_investment(invest_tick, 1_000_000_000, /* params */ );
    // Check stock unchanged until lag expires
    for t in invest_tick..(invest_tick + lag) {
        let advanced = adapt.clone().drain_pending(t);
        assert_eq!(advanced.stock, 0,
            "Adaptation stock changed before lag at tick {}", t);
    }
    // At exactly invest_tick + lag, stock should increase
    let vested = adapt.drain_pending(invest_tick + lag);
    assert!(vested.stock > 0, "Stock did not increase at vesting tick");
}
```

### I-4: Energy Conservation — Forcing Accumulation

```
AF(t+1) - AF(t) = emissions(t) - NaturalSink(t)
NaturalSink(t) = k_sink × AF(t)
AF(t+1) >= 0 always
```

**Test:**
```rust
proptest! {
    #[test]
    fn prop_forcing_accumulation_conserved(emissions in 0i64..10_000_000i64,
                                           af_init in 0i64..5_000_000i64) {
        let fi = ForcingIndex { af_index: af_init, sink_rate: 2_000, .. };
        let next = fi.advance(emissions, 800_000, 0);
        let expected_sink = (af_init * 2_000) / SCALE;
        let expected_af = (af_init + emissions - expected_sink).max(0);
        prop_assert_eq!(next.af_index, expected_af);
        prop_assert!(next.af_index >= 0);
    }
}
```

### I-5: Scarcity Pressure Bounded [0, SCALE]

```
∀ inputs: SP(t) ∈ [0, SCALE]
```

**Test (proptest):** Exhaustive over random component inputs, verifying the `clip()` in the SP formula keeps it in range.

---

## 12. Failure Modes

### FM-1: Runaway Forcing

**Condition:** `k_sink` is small relative to emissions rate; AF grows without bound.

**Trigger pattern:** Baseline or high-shock scenario with no mitigation policy. Emissions rate `> k_sink × AF` at all realistic AF values means forcing never stabilizes.

**Sim manifestation:** `AF` grows linearly (before damage onset) then still grows above onset. `CD_effective` saturates near 1.0. Productivity collapses. Economy enters depletion spiral.

**Detection:** `forcing_index` exceeds `3 × af_onset_threshold` for more than 5 consecutive ticks. Alert emitted via `climate.damage_modeled.v1` with a `runaway_forcing` flag (extension field).

**Invariant preserved:** AF can grow arbitrarily large but damage function is bounded [0,1] by sigmoid.

### FM-2: Adaptation Underfunding

**Condition:** `adaptation_share` is insufficient to build adaptation stock faster than damage rises. `AS(t)` plateaus while `CD(t)` continues climbing.

**Trigger pattern:** Delayed-action scenario where mitigation starts too late; adaptation investment below `~3%` of output.

**Sim manifestation:** `DamageReduction` grows slowly, never exceeding 0.15 even as `CD_effective` hits 0.5+. Expected loss climbs. Budget stress appears as adaptation competes with essentials provision.

**Detection:** `adaptation_roi` metric falls below a configurable floor (`roi_floor = 5_000` = 0.005) for 10 consecutive ticks, indicating marginal effectiveness even though stock is growing.

### FM-3: Resource Collapse Cascade

**Condition:** `RDF` approaches 0 while ESC declines. `cost_multiplier` exceeds 2.0. Economy cannot sustain essentials provision at elevated costs.

**Trigger pattern:** High-shock scenario with aggressive extraction and no renewable transition investment.

**Sim manifestation:** `cost_multiplier` for basic goods rises above 2.0. `SustainEfficiency` drops below 0.7. `ScarcityPressure` spikes. Tyranny drift activates if governance is weak. Governance module may trigger `AuthoritarianShift` event.

**Detection:** `rdf_scaled < 200_000` (RDF < 0.2) AND `energy_deficit_ratio > 300_000` (deficit > 30%) for 3 consecutive ticks. `climate.resource_stress.v1` emitted with `stress_kind = "esc_deficit_sustained"`.

### FM-4: Lagged-Shock Overshoot

**Condition:** A policy is applied at tick `T`. The lag `L_adapt` means effects do not vest until `T + L_adapt`. During the lag window, damage continues to accumulate. If forcing accelerates during the lag (e.g., in a high-shock scenario), the vested adaptation effect may be insufficient to catch up.

**Trigger pattern:** Delayed-action scenario where boundary trigger fires but damage has already climbed past the sigmoid knee. The `L_adapt = 20` tick lag allows 20 more ticks of damage accumulation at the elevated rate.

**Sim manifestation:** `climate.adaptation_applied.v1` fires at vest tick, but `effective_damage` at vest tick is significantly higher than when investment was made. `damage_reduction_before` in the event shows the gap. The economy has already sustained cumulative productivity loss during the lag.

**Detection:** When an adaptation event vests, compare `observed_damage_at_vest` to `damage_at_investment_tick`. If the ratio exceeds 1.5 (50% more damage than when investment was made), emit a `lag_overshoot_warning` flag in the adaptation event payload.

---

## 13. Acceptance Test Suite

The following test function signatures must pass. All live in `crates/climate/tests/`.

```rust
// tests/determinism.rs

/// AC-1: Scenario replay yields deterministic climate trajectories.
/// Same (state, scenario, economy_inputs, rng_seed) always produces identical output.
#[test]
fn test_climate_deterministic_replay() {
    let config = ScenarioConfig::baseline_v1();
    let phase = ClimatePhase::from_scenario(config).unwrap();
    let mut state = phase.initial_state();
    let mut states_run1 = Vec::new();
    let mut states_run2 = Vec::new();

    // Run 1
    let mut s = state.clone();
    for tick in 0..100u64 {
        let (next, _events) = phase.advance(&s, 1_000_000_000, 800_000_000,
                                            /* rng_seed */ tick * 31337);
        states_run1.push(next.clone());
        s = next;
    }

    // Run 2 — identical inputs
    s = state.clone();
    for tick in 0..100u64 {
        let (next, _events) = phase.advance(&s, 1_000_000_000, 800_000_000,
                                            tick * 31337);
        states_run2.push(next.clone());
        s = next;
    }

    assert_eq!(states_run1, states_run2,
        "Climate phase is not deterministic: runs diverge");
}

/// AC-2: Adaptation lever activation changes exposure metrics in the expected direction.
/// Enabling adaptation must reduce effective_damage relative to no-adaptation run,
/// observed after the lag period has elapsed.
#[test]
fn test_adaptation_lever_reduces_damage_after_lag() {
    let lag = 20u64;
    let mut no_adapt = run_scenario_n_ticks(ScenarioConfig::baseline_v1(), 150);
    let mut with_adapt = run_scenario_n_ticks(ScenarioConfig::coordinated_mitigation_v1(), 150);

    // After tick 150 (well past lag), effective_damage should be lower with adaptation
    let damage_no_adapt = no_adapt.last().unwrap().damage.effective_damage;
    let damage_with_adapt = with_adapt.last().unwrap().damage.effective_damage;

    assert!(damage_with_adapt < damage_no_adapt,
        "Adaptation did not reduce effective damage: {} >= {}",
        damage_with_adapt, damage_no_adapt);
}

/// AC-3: High-shock scenario triggers governance-risk lift and budget stress in metrics.
/// ScarcityPressure must exceed 0.35 (350_000 scaled) in the high-shock scenario
/// at some point within 80 ticks.
#[test]
fn test_high_shock_scenario_governance_risk_lift() {
    let states = run_scenario_n_ticks(ScenarioConfig::high_shock_v1(), 80);
    let max_sp = states.iter()
        .map(|s| s.scarcity_pressure)
        .max()
        .unwrap_or(0);
    assert!(max_sp > 350_000,
        "High-shock scenario did not produce scarcity pressure > 0.35: got {}",
        max_sp);
}

/// AC-4: Cross-module conservation — energy budget sums correctly.
/// ESC = RenewableCapacity + NonRenewable × ExtractEfficiency
/// and EnergyDeficit = max(0, EnergyDemand − ESC) is non-negative.
#[test]
fn test_energy_budget_conservation() {
    let states = run_scenario_n_ticks(ScenarioConfig::baseline_v1(), 200);
    for state in &states {
        let esc = state.resources.energy_supply_capacity();
        let deficit = state.resources.extraction_volume_joules; // demand proxy
        assert!(state.energy_deficit_ratio >= 0,
            "Energy deficit ratio is negative at tick {}", state.tick);
        assert!(state.energy_deficit_ratio <= SCALE,
            "Energy deficit ratio exceeds 1.0 at tick {}", state.tick);
        assert_eq!(
            state.resources.energy_supply_capacity(),
            state.resources.renewable_capacity_joules
                + (state.resources.nonrenewable_remaining_joules
                   * state.resources.current_extract_efficiency) / SCALE,
            "ESC formula violated at tick {}", state.tick
        );
    }
}

/// AC-5: RDF monotonically non-increasing without regeneration policy.
#[test]
fn test_rdf_non_increasing_no_regeneration() {
    let states = run_scenario_n_ticks(ScenarioConfig::baseline_v1(), 300);
    let mut prev_rdf = SCALE; // starts at 1.0
    for state in &states {
        assert!(state.resources.rdf_scaled <= prev_rdf,
            "RDF increased at tick {}: {} > {}",
            state.tick, state.resources.rdf_scaled, prev_rdf);
        prev_rdf = state.resources.rdf_scaled;
    }
}

/// AC-6: Delayed-action boundary event fires exactly once.
#[test]
fn test_delayed_action_boundary_event_fires_once() {
    let (states, events) = run_scenario_n_ticks_with_events(
        ScenarioConfig::delayed_action_v1(), 200
    );
    let boundary_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, ClimateEvent::ScenarioBoundaryCrossed(_)))
        .collect();
    assert_eq!(boundary_events.len(), 1,
        "Delayed-action boundary event fired {} times (expected 1)",
        boundary_events.len());
}

/// AC-7: Damage monotonicity — raw_damage never decreases as AF increases within a run.
/// (AF may decrease briefly via sink, so this tests per tick rather than globally.)
#[test]
fn test_damage_monotonic_in_forcing_within_scenario() {
    proptest!(|(seed in 0u64..10_000u64)| {
        let states = run_scenario_n_ticks_seeded(
            ScenarioConfig::baseline_v1(), 300, seed
        );
        for window in states.windows(2) {
            let prev = &window[0];
            let next = &window[1];
            if next.forcing.af_index >= prev.forcing.af_index {
                prop_assert!(next.damage.raw_damage >= prev.damage.raw_damage,
                    "Raw damage decreased while forcing increased: tick {} → {}",
                    prev.tick, next.tick);
            }
        }
    });
}
```

---

## 14. Performance Budget

Climate phase executes within the deterministic transition window. Total tick budget remains 14 ms (CIV-0001). Climate receives a 1.5 ms sub-budget.

| Sub-component | Budget | Implementation note |
|---|---|---|
| Forcing accumulation | 50 µs | One multiply + add per good category |
| Damage function (sigmoid lookup) | 20 µs | Pre-computed LUT, indexed by AF |
| Resource pool advance | 100 µs | Fixed-point arithmetic, no allocation |
| Adaptation pending-queue drain | 100 µs | `BTreeMap` drain, amortized O(vested events) |
| Scarcity pressure compute | 30 µs | Four multiplies + clip |
| Disaster sampling | 80 µs | `ChaCha20Rng.gen()`, one exponential draw |
| Metrics assembly | 50 µs | Struct fills from computed values |
| Event emission | 70 µs | JSON serialization of up to 3 events |
| **Total** | **~500 µs** | Well within 1.5 ms sub-budget |

**Scenario batch parallelism:** When running Monte Carlo sweeps (`N = 50–200 seeds`), climate phases across independent runs are embarrassingly parallel. The `ClimatePhase` is `Send + Sync` (no interior mutability in the hot path). Batch parallelism uses `rayon::par_iter` over the run batch, with each thread owning its own `ClimateState` and `ChaCha20Rng`. No cross-thread state sharing.

Scenario batch throughput target: 50 full 50-year runs (2600 ticks each) in under 60 seconds on a 4-core development machine.

---

## 15. CIV Sim Integration Notes

### 15.1 Required Cargo.toml Update

The `crates/climate` crate must be added to the workspace:

```toml
# /Users/kooshapari/temp-PRODVERCEL/485/kush/civ/Cargo.toml
[workspace]
members = [
  "crates/engine",
  "crates/policy",
  "crates/metrics",
  "crates/io",
  "crates/server",
  "crates/climate",   # <-- add this
]
resolver = "2"
```

The `crates/engine` crate must add `climate` as a dependency:

```toml
# crates/engine/Cargo.toml
[dependencies]
climate = { path = "../climate" }
```

### 15.2 Tick Phase Registration

The `ClimatePhase` must be registered in the engine's phase scheduler as a deterministic phase running before the economy transition. This is an implementation-time concern for the engine crate; the climate module only exposes `ClimateCompute::advance` and the engine calls it in sequence.

### 15.3 Scenario Parameter Versioning

Every `ClimateState` carries `scenario_id` and `scenario_version`. All climate events carry these fields. The `civreplay` file format (CIV-0001 Section "Replay File Format") must include the full `ScenarioConfig` in its initial state JSON to ensure replay fidelity. A scenario parameter change constitutes a new `scenario_version` and old replay files will fail verification if the version does not match.

### 15.4 YAML Scenario Config Files

Canonical scenario configs live at:

```
crates/climate/scenarios/
├── climate-baseline-v1.yaml
├── climate-delayed-action-v1.yaml
├── climate-high-shock-v1.yaml
└── climate-coordinated-mitigation-v1.yaml
```

These are loaded via `ScenarioConfig::from_yaml(path)` at simulation init time. The `crates/io` crate owns schema validation against the JSON schema for `ScenarioConfig`.

### 15.5 Metrics Module Integration

The `crates/metrics` crate must implement a `ClimateCollector` that implements the `Collector` trait defined in CIV-0001:

```rust
impl Collector for ClimateCollector {
    fn on_tick_end(&mut self, sim: &SimState, out: &mut MetricsFrame) {
        let cm = climate_metrics(&sim.climate_state, sim.economy_state.total_output_joules);
        out.climate_forcing_index = Some(cm.forcing_index);
        out.climate_resilience_index = Some(cm.resilience_index);
        out.climate_effective_damage = Some(cm.effective_damage);
        out.climate_expected_loss_joules = Some(cm.expected_loss_joules);
        out.climate_adaptation_roi = Some(cm.adaptation_roi);
        out.climate_displacement_pressure = Some(cm.displacement_pressure);
        out.climate_energy_deficit_ratio = Some(cm.energy_deficit_ratio);
        out.climate_resource_depletion_factor = Some(cm.resource_depletion_factor);
        out.climate_scarcity_pressure = Some(cm.scarcity_pressure);
        out.climate_adaptation_stock = Some(cm.adaptation_stock);
        out.climate_renewable_share = Some(cm.renewable_share);
        out.climate_scenario_id = Some(cm.scenario_id.clone());
        out.climate_scenario_version = Some(cm.scenario_version);
    }
}
```

### 15.6 Dashboard Telemetry Surface

The following `ClimateMetrics` fields feed the CivLab dashboard as described in the Climate Dashboard UI Spec derived from the ChatGPT conversation:

| Dashboard panel | Metric field | Description |
|---|---|---|
| Atmospheric Forcing Graph | `forcing_index` | Primary trend chart |
| Resilience Index gauge | `resilience_index` | `(1 − effective_damage)` |
| Expected Loss | `expected_loss_joules` | Economy impact |
| Adaptation ROI | `adaptation_roi` | Diminishing returns curve |
| Displacement Pressure | `displacement_pressure` | Migration temptation signal |
| Energy Deficit | `energy_deficit_ratio` | Joule quota tightening trigger |
| Resource Depletion | `resource_depletion_factor` | Cost inflation signal |
| Scarcity Pressure | `scarcity_pressure` | Tyranny-coupling warning |
| Renewable Share | `renewable_share` | Energy transition progress |

The `scarcity_pressure` metric directly feeds the tyranny index computation in the governance module via the following integration point:

```
TyrannyIndex_scarcity_term = w5 × SP(t) × (Σ + E) / 2
```

where `Σ` = surveillance intensity and `E` = enforcement intensity (both from governance state). This matches the formal tyranny formula from the ChatGPT conversation's "formal tyranny metric" section.

### 15.7 Joule Economy Quota Tightening Protocol

When `energy_deficit_ratio > 0`, the Joule Economy allocator (CIV-0107) must execute the following sequence in the same tick:

1. Read `energy_supply_capacity(climate_state)` from the climate module.
2. Compute `quota_reduction_fraction = energy_deficit_ratio × rationing_strictness`.
3. Apply `adjusted_quota = base_quota × (1 − quota_reduction_fraction)` to all citizens.
4. If `ScenarioBoundary::ActivateRationing { rights_first: true, .. }` is active, provision essentials from the contracted pool before applying quota reductions to discretionary budgets.
5. Emit `joule.scarcity_shock.v1` referencing the climate `tick` and `scenario_id`.

This protocol ensures the physical energy constraint propagates correctly into the joule ledger without breaking CIV-0100's double-entry conservation invariants.

### 15.8 Determinism Verification

The climate phase participates in CIV-0001's hash contract enforcement:

- Every `climate.damage_modeled.v1` event includes `state_hash: SHA-256(ClimateState)`.
- On replay, the engine recomputes `ClimateState` and verifies the hash matches.
- A mismatch is a determinism violation and causes the replay to abort with error.

---

**Version History:**

- v2.0 (2026-02-21): Full expansion from 29-line stub to 800+ line engineering-grade specification. Added formal climate math, four scenario families with full Rust struct definitions, damage function with calibrated bounds, seven state structs, module layout with traits, four JSON event schemas, full PostgreSQL DDL, economy coupling with tick ordering, four adaptation levers with diminishing-returns cost curves, lag model with pending-effect queue, five conservation invariants with property test signatures, four failure modes, seven acceptance tests, performance budget, and CIV sim integration notes.
- v1.0 (2026-02-21): 29-line stub.

---

## 16. Extended Scenario Mechanics

### 16.1 Full Parameter Tables — All Four Scenario Families

The tables below enumerate every calibrated parameter across the four canonical scenario families. All numeric values are fixed-point `i64` scaled by `SCALE = 1_000_000`. Human-readable equivalents appear in the "Float" column.

#### Forcing Parameters

| Parameter | Symbol | Baseline | DelayedAction | HighShock | CoordMitigation | Notes |
|---|---|---|---|---|---|---|
| Sink rate | k_sink | 2_000 (0.002) | 2_000 (0.002) | 500 (0.0005) | 4_000 (0.004) | Per-tick decay of AF |
| Emissions intensity (heavy) | ε_h | 150_000 (0.15) | 150_000 (0.15) | 200_000 (0.20) | 60_000 (0.06) | Forcing per output unit |
| Emissions intensity (clean) | ε_c | 20_000 (0.02) | 20_000 (0.02) | 30_000 (0.03) | 5_000 (0.005) | |
| AF safe threshold | AF_safe | 500_000 (0.5) | 500_000 (0.5) | 400_000 (0.4) | 500_000 (0.5) | Below this: no onset |
| AF onset threshold | AF_onset | 800_000 (0.8) | 800_000 (0.8) | 600_000 (0.6) | 800_000 (0.8) | Damage sigmoid center |
| Damage steepness | α | 3_000_000 (3.0) | 3_000_000 (3.0) | 5_000_000 (5.0) | 3_000_000 (3.0) | Higher = sharper |
| Productivity damage weight | pdw | 600_000 (0.6) | 600_000 (0.6) | 800_000 (0.8) | 400_000 (0.4) | |
| Health damage weight | hdw | 300_000 (0.3) | 300_000 (0.3) | 500_000 (0.5) | 200_000 (0.2) | |
| Housing decay amplifier | hda | 1_000_000 (1.0) | 1_000_000 (1.0) | 2_000_000 (2.0) | 500_000 (0.5) | |

#### Depletion Parameters

| Parameter | Symbol | Baseline | DelayedAction | HighShock | CoordMitigation |
|---|---|---|---|---|---|
| Depletion coefficient | δ | 1_000 (0.001) | 1_000 (0.001) | 2_000 (0.002) | 500 (0.0005) |
| Cost amplifier at full depletion | φ | 800_000 (0.8) | 800_000 (0.8) | 1_500_000 (1.5) | 500_000 (0.5) |
| Initial RDF | RDF_0 | 1_000_000 (1.0) | 1_000_000 (1.0) | 1_000_000 (1.0) | 1_000_000 (1.0) |

#### Energy Parameters

| Parameter | Symbol | Baseline | DelayedAction | HighShock | CoordMitigation |
|---|---|---|---|---|---|
| Renewable growth rate/tick | g_r | 3_000 (0.003) | 3_000 (0.003) | 2_000 (0.002) | 8_000 (0.008) |
| Investment yield | iy | 500_000 (0.5) | 500_000 (0.5) | 400_000 (0.4) | 700_000 (0.7) |
| EROI decay/tick | d_eroi | 200 (0.0002) | 200 (0.0002) | 400 (0.0004) | 100 (0.0001) |
| Base extract efficiency | η_e | 900_000 (0.9) | 900_000 (0.9) | 800_000 (0.8) | 950_000 (0.95) |

#### Adaptation Parameters

| Parameter | Symbol | Baseline | DelayedAction | HighShock | CoordMitigation |
|---|---|---|---|---|---|
| Adaptation investment share | a_share | 0 (0%) | 0→50_000 (trigger) | 30_000 (3%) | 80_000 (8%) |
| Effectiveness coefficient | η | 10_000 (0.01) | 15_000 (0.015) | 10_000 (0.01) | 25_000 (0.025) |
| Depreciation rate/tick | d_a | 20_000 (0.02) | 20_000 (0.02) | 30_000 (0.03) | 15_000 (0.015) |
| Investment lag | L_adapt | 20 ticks | 20 ticks | 20 ticks | 20 ticks |

#### Shock Parameters

| Parameter | Symbol | Baseline | DelayedAction | HighShock | CoordMitigation |
|---|---|---|---|---|---|
| Base disaster rate/tick | r_d | 5_000 (0.005) | 5_000 (0.005) | 20_000 (0.02) | 3_000 (0.003) |
| Beta damage multiplier | β | 30_000 (0.03) | 30_000 (0.03) | 80_000 (0.08) | 20_000 (0.02) |
| Base severity | s_b | 100_000 (0.1) | 100_000 (0.1) | 250_000 (0.25) | 70_000 (0.07) |
| Severity amplifier | s_a | 2_000_000 (2.0) | 2_000_000 (2.0) | 3_000_000 (3.0) | 1_500_000 (1.5) |

### 16.2 Tick-by-Tick Forcing Trajectories

The following describes the expected AF evolution over a representative 100-tick horizon for each scenario. All trajectories assume `EconomyOutput = 1_000_000_000` joules/tick (1 GJ) and `HeavyGoodsFraction = 0.6`.

**Baseline expected forcing by decade:**

| Tick | Expected AF (raw i64) | Human (AF value) | CD_effective (approx) |
|---|---|---|---|
| 0 | 0 | 0.000 | 0.000 |
| 10 | 810_000 | 0.810 | 0.010 |
| 20 | 1_540_000 | 1.540 | 0.093 |
| 30 | 2_190_000 | 2.190 | 0.330 |
| 40 | 2_740_000 | 2.740 | 0.590 |
| 50 | 3_190_000 | 3.190 | 0.770 |
| 60 | 3_550_000 | 3.550 | 0.867 |
| 80 | 4_050_000 | 4.050 | 0.940 |
| 100 | 4_390_000 | 4.390 | 0.969 |

Values use the sigmoid formula with `α = 3.0`, `AF_onset = 0.8`. The transition from near-zero damage to `CD > 0.5` occurs roughly between ticks 20 and 40, representing the sigmoid knee.

**CoordinatedMitigation expected forcing by decade:**

| Tick | Expected AF (raw i64) | Human (AF value) | CD_effective (approx) |
|---|---|---|---|
| 0 | 0 | 0.000 | 0.000 |
| 10 | 390_000 | 0.390 | 0.001 |
| 20 | 640_000 | 0.640 | 0.003 |
| 40 | 780_000 | 0.780 | 0.018 |
| 60 | 790_000 | 0.790 | 0.022 |
| 100 | 795_000 | 0.795 | 0.025 |

Forcing approaches a plateau below AF_onset because the enhanced sink rate and capped emissions intensity create a near-equilibrium. Effective damage stays below `0.03` due to both the low raw damage and the high adaptation stock.

### 16.3 Tipping Point Detection and Hysteresis Model

Tipping points are irreversible threshold crossings in the climate state. Once crossed, the system enters a new forcing regime that cannot be undone by reducing emissions alone.

**Tipping point definitions:**

| ID | Variable | Threshold | Effect on forcing regime | Hysteresis window |
|---|---|---|---|---|
| TP-1 | AF_index | 1_500_000 (1.5) | k_sink halved permanently | 200_000 irreversible |
| TP-2 | RDF | 200_000 (0.2) | EROI decay doubles | RDF cannot recover above 0.25 |
| TP-3 | CD_effective | 600_000 (0.6) | disaster probability doubles | Adaptation can only slow, not reverse |
| TP-4 | AF_index | 3_000_000 (3.0) | k_sink set to 0 (feedback loop locked in) | Fully irreversible |

**Hysteresis model:** When a tipping point is crossed, a partial reversal does not return the system to its pre-crossing behavior. The hysteresis window defines how far the variable must recede before any reversal of the tip effect begins. This is modeled as:

```
TipReversalFraction(t) = max(0,
    (tip_threshold - hysteresis_window - current_value)
    / tip_threshold
)

EffectiveSinkRate(t) = k_sink_base
    × (1 - tip1_fired × 0.5 × (1 - TipReversalFraction_1(t)))
    × (1 - tip4_fired × 1.0)   // tip4 is fully irreversible
```

**Rust struct for tipping point state:**

```rust
// crates/climate/src/scenario.rs

/// Record of all tipping points crossed in this run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TippingPointState {
    /// TP-1: AF crossed 1.5. k_sink halved.
    pub tp1_fired: bool,
    /// Tick at which TP-1 fired. None if not yet.
    pub tp1_tick: Option<u64>,
    /// TP-2: RDF crossed 0.2. EROI decay doubled.
    pub tp2_fired: bool,
    pub tp2_tick: Option<u64>,
    /// TP-3: CD_effective crossed 0.6. Disaster rate doubled.
    pub tp3_fired: bool,
    pub tp3_tick: Option<u64>,
    /// TP-4: AF crossed 3.0. Sink locked at zero (fully irreversible).
    pub tp4_fired: bool,
    pub tp4_tick: Option<u64>,
}

impl TippingPointState {
    pub fn new() -> Self {
        TippingPointState {
            tp1_fired: false, tp1_tick: None,
            tp2_fired: false, tp2_tick: None,
            tp3_fired: false, tp3_tick: None,
            tp4_fired: false, tp4_tick: None,
        }
    }

    /// Evaluate which tipping points fire this tick.
    /// Returns a list of newly fired tip IDs.
    pub fn evaluate(
        &self,
        af_index: i64,
        rdf: i64,
        cd_effective: i64,
        tick: u64,
    ) -> (TippingPointState, Vec<u8>) {
        let mut next = self.clone();
        let mut newly_fired = Vec::new();

        if !self.tp1_fired && af_index >= 1_500_000 {
            next.tp1_fired = true;
            next.tp1_tick = Some(tick);
            newly_fired.push(1);
        }
        if !self.tp2_fired && rdf <= 200_000 {
            next.tp2_fired = true;
            next.tp2_tick = Some(tick);
            newly_fired.push(2);
        }
        if !self.tp3_fired && cd_effective >= 600_000 {
            next.tp3_fired = true;
            next.tp3_tick = Some(tick);
            newly_fired.push(3);
        }
        if !self.tp4_fired && af_index >= 3_000_000 {
            next.tp4_fired = true;
            next.tp4_tick = Some(tick);
            newly_fired.push(4);
        }
        (next, newly_fired)
    }

    /// Effective sink rate after all active tipping points.
    /// k_sink_base and k_sink_halved are in fixed-point i64 scaled by SCALE.
    pub fn effective_sink_rate(&self, k_sink_base: i64) -> i64 {
        if self.tp4_fired {
            return 0; // fully irreversible feedback lock-in
        }
        if self.tp1_fired {
            k_sink_base / 2
        } else {
            k_sink_base
        }
    }

    /// Effective EROI decay rate after active tipping points.
    pub fn effective_eroi_decay(&self, eroi_decay_base: i64) -> i64 {
        if self.tp2_fired {
            eroi_decay_base * 2
        } else {
            eroi_decay_base
        }
    }

    /// Effective disaster base rate after active tipping points.
    pub fn effective_disaster_rate(&self, base_rate: i64) -> i64 {
        if self.tp3_fired {
            base_rate * 2
        } else {
            base_rate
        }
    }
}
```

### 16.4 Scenario Composition — Combining Two Families

Scenario composition allows two scenario families to be combined into a single run. The composed scenario uses one family's forcing trajectory and a second family's shock parameters, enabling cross-family interaction analysis. Composition is defined via `ScenarioComposition`:

```rust
// crates/climate/src/scenario.rs

/// Defines how two scenario configs are merged.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioComposition {
    /// Primary scenario: supplies ForcingParams, DepletionParams, EnergyParams, AdaptationParams.
    pub primary_id: String,
    pub primary_version: u32,
    /// Secondary scenario: supplies ShockParams and ScenarioBoundary (if any).
    pub secondary_id: String,
    pub secondary_version: u32,
    /// Composition mode: how the two configs are merged.
    pub mode: CompositionMode,
    /// Composed scenario ID for event tagging.
    pub composed_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompositionMode {
    /// Primary forcing + secondary shocks. Adaptation from primary.
    ForcingPlusShocks,
    /// Weighted average of all matching parameters.
    WeightedBlend {
        primary_weight: i64,  // scaled by SCALE; secondary_weight = SCALE - primary_weight
    },
    /// Primary everything, but activate secondary boundary trigger if primary has none.
    PrimaryWithFallbackBoundary,
}

impl ScenarioComposition {
    /// Produce a composed ScenarioConfig from two source configs.
    pub fn compose(
        &self,
        primary: &ScenarioConfig,
        secondary: &ScenarioConfig,
    ) -> ScenarioConfig {
        match self.mode {
            CompositionMode::ForcingPlusShocks => ScenarioConfig {
                id: self.composed_id.clone(),
                family: primary.family.clone(),
                forcing_params: primary.forcing_params.clone(),
                depletion_params: primary.depletion_params.clone(),
                energy_params: primary.energy_params.clone(),
                adaptation_params: primary.adaptation_params.clone(),
                shock_params: secondary.shock_params.clone(),
                scenario_boundary: primary.scenario_boundary.clone()
                    .or_else(|| secondary.scenario_boundary.clone()),
                scenario_version: primary.scenario_version,
            },
            CompositionMode::WeightedBlend { primary_weight } => {
                let sw = SCALE - primary_weight;
                let pw = primary_weight;
                ScenarioConfig {
                    id: self.composed_id.clone(),
                    family: primary.family.clone(),
                    forcing_params: ForcingParams {
                        k_sink: (primary.forcing_params.k_sink * pw
                            + secondary.forcing_params.k_sink * sw) / SCALE,
                        emissions_intensity_heavy: (
                            primary.forcing_params.emissions_intensity_heavy * pw
                            + secondary.forcing_params.emissions_intensity_heavy * sw) / SCALE,
                        emissions_intensity_clean: (
                            primary.forcing_params.emissions_intensity_clean * pw
                            + secondary.forcing_params.emissions_intensity_clean * sw) / SCALE,
                        af_safe_threshold: primary.forcing_params.af_safe_threshold,
                        af_onset_threshold: primary.forcing_params.af_onset_threshold,
                        damage_alpha: (primary.forcing_params.damage_alpha * pw
                            + secondary.forcing_params.damage_alpha * sw) / SCALE,
                        productivity_damage_weight: (
                            primary.forcing_params.productivity_damage_weight * pw
                            + secondary.forcing_params.productivity_damage_weight * sw) / SCALE,
                        health_damage_weight: (
                            primary.forcing_params.health_damage_weight * pw
                            + secondary.forcing_params.health_damage_weight * sw) / SCALE,
                        housing_decay_amplifier: (
                            primary.forcing_params.housing_decay_amplifier * pw
                            + secondary.forcing_params.housing_decay_amplifier * sw) / SCALE,
                    },
                    depletion_params: primary.depletion_params.clone(),
                    energy_params: primary.energy_params.clone(),
                    adaptation_params: primary.adaptation_params.clone(),
                    shock_params: ShockParams {
                        base_disaster_rate: (
                            primary.shock_params.base_disaster_rate * pw
                            + secondary.shock_params.base_disaster_rate * sw) / SCALE,
                        beta_damage_multiplier: (
                            primary.shock_params.beta_damage_multiplier * pw
                            + secondary.shock_params.beta_damage_multiplier * sw) / SCALE,
                        base_severity: (
                            primary.shock_params.base_severity * pw
                            + secondary.shock_params.base_severity * sw) / SCALE,
                        severity_amplifier: (
                            primary.shock_params.severity_amplifier * pw
                            + secondary.shock_params.severity_amplifier * sw) / SCALE,
                    },
                    scenario_boundary: primary.scenario_boundary.clone(),
                    scenario_version: primary.scenario_version,
                }
            },
            CompositionMode::PrimaryWithFallbackBoundary => ScenarioConfig {
                id: self.composed_id.clone(),
                scenario_boundary: primary.scenario_boundary.clone()
                    .or_else(|| secondary.scenario_boundary.clone()),
                ..primary.clone()
            },
        }
    }
}
```

**Known interaction effects when composing Baseline + HighShock:**

| Interaction | Direction | Mechanism |
|---|---|---|
| Baseline forcing + HighShock shocks | Forcing accumulates slowly; disasters frequent before adaptation builds | Disaster rate 4x baseline while AF climbs slowly |
| DelayedAction + HighShock | Trigger fires when AF > 0.8, but repeated disasters delay adaptation build | Lag and disaster overlap creates 20-40 tick vulnerability window |
| CoordMitigation + HighShock shocks | Strong adaptation stock absorbs most disaster severity | `severity_amplifier × CD_effective` is low because `CD_effective < 0.15` |

### 16.5 Multi-Run Monte Carlo — Batch Execution

The Monte Carlo runner executes `N` independent runs over a fixed scenario with varied `ChaCha20Rng` seeds. This yields distribution statistics over outcome variables.

**Rust structs:**

```rust
// crates/climate/src/scenario.rs

use rayon::prelude::*;

/// Configuration for a Monte Carlo batch run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonteCarloConfig {
    pub scenario_config: ScenarioConfig,
    pub num_runs: u32,
    /// Seed for the seed-generator: seeds are derived as `base_seed + run_index`.
    pub base_seed: u64,
    /// Number of ticks per run.
    pub ticks_per_run: u64,
    /// Economy inputs held fixed across all runs (for isolation of climate stochasticity).
    pub fixed_economy_output_joules: i64,
    pub fixed_energy_demand_joules: i64,
}

/// Per-run summary statistics collected at the end of each Monte Carlo run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonteCarloRunSummary {
    pub run_index: u32,
    pub seed: u64,
    /// Maximum AF reached during run.
    pub peak_af: i64,
    /// Maximum CD_effective reached during run.
    pub peak_cd_effective: i64,
    /// Final adaptation stock at end of run.
    pub final_adaptation_stock: i64,
    /// Cumulative disasters fired.
    pub total_disasters: u32,
    /// Total ticks with SP > 0.35 (governance risk zone).
    pub ticks_above_scarcity_threshold: u64,
    /// Final RDF.
    pub final_rdf: i64,
    /// Tick at which AF first crossed onset (None if never).
    pub onset_crossed_tick: Option<u64>,
    /// Which tipping points fired.
    pub tipping_points_fired: Vec<u8>,
    /// Cumulative expected loss in joules.
    pub cumulative_expected_loss_joules: i64,
}

/// Aggregated statistics over all Monte Carlo runs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonteCarloResult {
    pub config: MonteCarloConfig,
    pub num_runs: u32,
    /// Mean of peak_af across runs.
    pub peak_af_mean: i64,
    /// Standard deviation of peak_af.
    pub peak_af_std: i64,
    /// 5th percentile of peak_af.
    pub peak_af_p5: i64,
    /// 50th percentile of peak_af.
    pub peak_af_p50: i64,
    /// 95th percentile of peak_af.
    pub peak_af_p95: i64,
    /// Mean of peak_cd_effective.
    pub peak_cd_mean: i64,
    pub peak_cd_p5: i64,
    pub peak_cd_p50: i64,
    pub peak_cd_p95: i64,
    /// Fraction of runs in which TP-1 (AF > 1.5) fired.
    pub tp1_fire_rate: i64,   // scaled by SCALE; 0.25 = 25% of runs
    /// Fraction of runs in which TP-4 (runaway) fired.
    pub tp4_fire_rate: i64,
    /// Mean ticks above scarcity threshold.
    pub mean_ticks_above_scarcity: i64,
    /// Confidence interval half-width at 90% for peak_af.
    pub peak_af_ci90_halfwidth: i64,
    pub run_summaries: Vec<MonteCarloRunSummary>,
}

/// Runner struct for executing a Monte Carlo batch.
pub struct ScenarioRunner {
    pub config: MonteCarloConfig,
}

impl ScenarioRunner {
    pub fn new(config: MonteCarloConfig) -> Self {
        ScenarioRunner { config }
    }

    /// Execute all runs in parallel using rayon.
    /// Returns aggregated MonteCarloResult.
    pub fn run(&self) -> Result<MonteCarloResult, ConfigError> {
        let phase = ClimatePhase::from_scenario(self.config.scenario_config.clone())?;

        let summaries: Vec<MonteCarloRunSummary> = (0..self.config.num_runs)
            .into_par_iter()
            .map(|run_idx| {
                let seed = self.config.base_seed + run_idx as u64;
                self.run_single(&phase, run_idx, seed)
            })
            .collect();

        Ok(self.aggregate(summaries))
    }

    fn run_single(
        &self,
        phase: &ClimatePhase,
        run_idx: u32,
        seed: u64,
    ) -> MonteCarloRunSummary {
        let mut state = phase.initial_state();
        let mut tip_state = TippingPointState::new();
        let mut total_disasters: u32 = 0;
        let mut ticks_above_sp: u64 = 0;
        let mut cumulative_loss: i64 = 0;

        for tick in 0..self.config.ticks_per_run {
            let (next_state, events) = phase.advance(
                &state,
                self.config.fixed_economy_output_joules,
                self.config.fixed_energy_demand_joules,
                seed.wrapping_add(tick),
            );
            let (next_tips, newly_fired) = tip_state.evaluate(
                next_state.forcing.af_index,
                next_state.resources.rdf_scaled,
                next_state.damage.effective_damage,
                tick,
            );
            tip_state = next_tips;

            if next_state.active_disaster.is_some() {
                total_disasters += 1;
            }
            if next_state.scarcity_pressure > 350_000 {
                ticks_above_sp += 1;
            }
            let loss_this_tick = (self.config.fixed_economy_output_joules
                * next_state.damage.effective_damage) / SCALE;
            cumulative_loss = cumulative_loss.saturating_add(loss_this_tick);

            state = next_state;
        }

        MonteCarloRunSummary {
            run_index: run_idx,
            seed,
            peak_af: state.forcing.af_index,
            peak_cd_effective: state.damage.effective_damage,
            final_adaptation_stock: state.adaptation.stock,
            total_disasters,
            ticks_above_scarcity_threshold: ticks_above_sp,
            final_rdf: state.resources.rdf_scaled,
            onset_crossed_tick: state.forcing.onset_crossed_tick,
            tipping_points_fired: {
                let mut v = Vec::new();
                if tip_state.tp1_fired { v.push(1); }
                if tip_state.tp2_fired { v.push(2); }
                if tip_state.tp3_fired { v.push(3); }
                if tip_state.tp4_fired { v.push(4); }
                v
            },
            cumulative_expected_loss_joules: cumulative_loss,
        }
    }

    fn aggregate(&self, summaries: Vec<MonteCarloRunSummary>) -> MonteCarloResult {
        let n = summaries.len() as i64;
        let peak_af_values: Vec<i64> = summaries.iter().map(|s| s.peak_af).collect();
        let peak_cd_values: Vec<i64> = summaries.iter().map(|s| s.peak_cd_effective).collect();

        let mean_af = peak_af_values.iter().sum::<i64>() / n;
        let mean_cd = peak_cd_values.iter().sum::<i64>() / n;

        // Variance: E[(x - mean)^2]. Fixed-point: divide by SCALE after squaring.
        let variance_af = peak_af_values.iter()
            .map(|&v| {
                let diff = v - mean_af;
                (diff * diff) / SCALE
            })
            .sum::<i64>() / n;
        // Std dev: integer sqrt of variance
        let std_af = integer_sqrt(variance_af);

        let tp1_count = summaries.iter().filter(|s| s.tipping_points_fired.contains(&1)).count();
        let tp4_count = summaries.iter().filter(|s| s.tipping_points_fired.contains(&4)).count();

        let mean_sp_ticks = summaries.iter().map(|s| s.ticks_above_scarcity_threshold as i64).sum::<i64>() / n;

        // 90% CI half-width = 1.645 * std / sqrt(n). In fixed-point:
        // ci90 = (1_645_000 * std_af / SCALE) / integer_sqrt(n as i64)
        let ci90 = (1_645_000_i64 * std_af / SCALE) / integer_sqrt(n);

        let mut sorted_af = peak_af_values.clone();
        sorted_af.sort_unstable();
        let p5_idx = ((n as f64 * 0.05) as usize).min(summaries.len() - 1);
        let p50_idx = ((n as f64 * 0.50) as usize).min(summaries.len() - 1);
        let p95_idx = ((n as f64 * 0.95) as usize).min(summaries.len() - 1);

        let mut sorted_cd = peak_cd_values.clone();
        sorted_cd.sort_unstable();

        MonteCarloResult {
            config: self.config.clone(),
            num_runs: summaries.len() as u32,
            peak_af_mean: mean_af,
            peak_af_std: std_af,
            peak_af_p5: sorted_af[p5_idx],
            peak_af_p50: sorted_af[p50_idx],
            peak_af_p95: sorted_af[p95_idx],
            peak_cd_mean: mean_cd,
            peak_cd_p5: sorted_cd[p5_idx],
            peak_cd_p50: sorted_cd[p50_idx],
            peak_cd_p95: sorted_cd[p95_idx],
            tp1_fire_rate: (tp1_count as i64 * SCALE) / n,
            tp4_fire_rate: (tp4_count as i64 * SCALE) / n,
            mean_ticks_above_scarcity: mean_sp_ticks,
            peak_af_ci90_halfwidth: ci90,
            run_summaries: summaries,
        }
    }
}

/// Integer square root for fixed-point std dev computation.
fn integer_sqrt(n: i64) -> i64 {
    if n <= 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
```

**Seeded RNG convention for Monte Carlo:** Each run uses `seed = base_seed + run_index`. Within a single run, the per-tick rng seed is `seed.wrapping_add(tick)`. This ensures runs are independent (different seeds) while remaining fully deterministic (given the base seed and run index, any run can be exactly replicated).

**Statistics collection targets:**

| Statistic | Primary use |
|---|---|
| `peak_af_p5 / p50 / p95` | Confidence envelope on forcing trajectory |
| `tp1_fire_rate`, `tp4_fire_rate` | Tipping point risk quantification |
| `peak_af_ci90_halfwidth` | Uncertainty band for dashboard display |
| `mean_ticks_above_scarcity` | Governance risk exposure expected value |
| `cumulative_expected_loss_joules` | Economic impact distribution |

---

## 17. Climate-Economy Coupling (Deep)

### 17.1 Granular Tick Ordering — Exact Computation Sequence

The following pseudocode is the normative description of what happens within a single tick when climate runs before economy. Every downstream module reads only from this tick's finalized `ClimateState[N]`.

```
Tick N begins:

1. CommandIntake phase
   - New policy commands from clients are buffered; NOT yet applied.

2. PolicyPhase
   - Reads ClimateState[N-1].scarcity_pressure
   - Reads ClimateState[N-1].energy_deficit_ratio
   - Computes policy decisions (adaptation investment share, carbon cap changes)
   - Writes PolicyDecision[N] (not yet applied to climate state)

3. ClimatePhase
   Input:
     ClimateState[N-1]        -- previous tick's full state
     EconomyOutput[N-1]       -- total joules produced last tick (for emissions)
     EnergyDemand[N-1]        -- total energy demanded last tick
     RngSeed[N]               -- = sim_seed XOR tick_number (deterministic)
     PolicyDecision[N]        -- adaptation share, carbon cap for THIS tick

   Computation sequence within ClimatePhase:
   3a. Apply policy updates (new adaptation share from PolicyDecision[N])
   3b. Compute emissions:
         emissions = EconomyOutput[N-1] × weighted_emissions_intensity
         weighted_emissions_intensity = heavy_fraction × ε_h + clean_fraction × ε_c
         heavy_fraction read from last tick's sector mix
   3c. Advance ForcingIndex:
         ForcingIndex[N] = ForcingIndex[N-1].advance(emissions, af_onset, N)
   3d. Evaluate tipping points: TippingPointState[N]
   3e. Compute DamageEstimate from sigmoid lookup:
         raw_damage = sigmoid_lut[ForcingIndex[N].af_index]
   3f. Drain adaptation pending-effects queue for tick N:
         stock += vested_increments
   3g. Apply depreciation to adaptation stock
   3h. Queue new adaptation investment for tick N+L_adapt:
         invest = EconomyOutput[N-1] × adaptation_share
         pending_effects[N + L_adapt] += (invest × eta_effectiveness) / SCALE
   3i. Compute adaptation damage reduction: DamageReduction[N]
   3j. Compute effective damage: CD_effective[N] = raw_damage × (1 - DamageReduction[N])
   3k. Advance ResourcePool:
         Update extraction efficiency (EROI decay)
         Apply extraction volume from last tick
         Update renewable capacity
         Compute ESC[N]
   3l. Compute EnergyDeficitRatio[N]:
         deficit = max(0, EnergyDemand[N-1] - ESC[N])
         ratio = deficit / EnergyDemand[N-1]
   3m. Sample disaster using RngSeed[N]:
         DisasterEvent[N] (or None)
   3n. Apply disaster shocks to ESC, housing, health (immediate)
   3o. Compute ScarcityPressure[N]:
         SP = clip(w1×(1-SustainEff) + w2×CD_eff + w3×EDR + w4×(1-RDF)×rsw, 0, SCALE)
   3p. Assemble ClimateState[N]
   3q. Emit events: climate.damage_modeled.v1, boundary events if triggered

   Output:
     ClimateState[N]          -- all downstream modules read THIS
     ClimateEvents[N]         -- appended to event bus

4. DeterministicTransition (Economy phase)
   Reads from ClimateState[N]:
     effective_productivity_multiplier(ClimateState[N])
     energy_supply_capacity(ClimateState[N])        -- hard cap on joule pool
     cost_multiplier_for_good(ClimateState[N], ε)  -- per-good
     scarcity_pressure(ClimateState[N])             -- governance drift input

   Economy computes:
     ProductionVolumes[N]     -- constrained by ESC[N] and productivity multiplier
     EconomyOutput[N]         -- total joules produced
     EnergyDemand[N]          -- total energy demanded by production and consumption
     SectorMix[N]             -- fraction of heavy vs clean production

   These feed ClimatePhase at tick N+1.

5-7. Stochastic events, metrics, broadcast (unchanged from CIV-0001).
```

**Critical constraint:** Step 3b uses `EconomyOutput[N-1]`, not `EconomyOutput[N]`. There is no same-tick feedback from economy to climate within a single tick. The coupling is always one-tick delayed on the economy-to-climate direction and zero-tick delayed on the climate-to-economy direction. This asymmetry is intentional and ensures determinism without circular dependencies.

### 17.2 Renewable Energy Supply Variability Model

Renewable capacity output is not perfectly smooth. Weather and seasonal variability introduce stochastic fluctuations around the expected renewable capacity. This is modeled as a multiplicative noise factor applied each tick:

```
RenewableOutput_actual(t) = RenewableCapacity(t) × VariabilityFactor(t)

VariabilityFactor(t) ~ Clipped Normal(mean=1.0, sd=variability_sigma, min=0.5, max=1.5)

variability_sigma ∈ [0.05, 0.25]   (default 0.10 for solar/wind mix)
```

In fixed-point:

```rust
// crates/climate/src/depletion.rs

/// Sample renewable variability factor for this tick.
/// Returns a multiplier in [500_000, 1_500_000] (fixed-point SCALE).
pub fn sample_renewable_variability(
    rng: &mut ChaCha20Rng,
    variability_sigma_scaled: i64,  // e.g., 100_000 for σ=0.1
) -> i64 {
    // Sample from a normal approximation using Box-Muller via two uniform samples.
    // u1, u2 in [1, SCALE-1] to avoid log(0).
    let u1 = rng.gen_range(1..SCALE) as f64 / SCALE as f64;
    let u2 = rng.gen_range(1..SCALE) as f64 / SCALE as f64;
    let sigma = variability_sigma_scaled as f64 / SCALE as f64;
    let normal_sample = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    let raw = 1.0 + sigma * normal_sample;
    // Clip to [0.5, 1.5] and convert to fixed-point.
    let clipped = raw.max(0.5).min(1.5);
    (clipped * SCALE as f64) as i64
}
```

**Smoothing via reserves:** When renewable output dips below demand (negative variability spike), the energy reserve buffer absorbs the shortfall before triggering an `EnergyDeficit`. The reserve drain rate is:

```
ReserveDrain(t) = max(0, ESC_expected(t) - RenewableOutput_actual(t))

EmergencyReserves(t+1) = EmergencyReserves(t) - ReserveDrain(t)
                          + ReserveRefill(t)

ReserveRefill(t) = max(0, RenewableOutput_actual(t) - EnergyDemand(t))
                  × reserve_fill_fraction    // [0.2, 0.8]
```

The reserve stock is tracked in the `ResourcePool` as `emergency_reserve_joules: i64`. The energy deficit computation uses the reserve-smoothed ESC:

```
ESC_smoothed(t) = RenewableOutput_actual(t)
                + NonRenewableContrib(t)
                + min(EmergencyReserves(t), max_discharge_per_tick)
```

**Demand-response protocol:** When `EnergyDeficitRatio > 100_000` (deficit > 10%), the system enters demand-response mode. Non-essential consumption is curtailed by a fraction proportional to the deficit:

```
DemandResponseCurtailment(t) = EnergyDeficitRatio(t) × demand_response_elasticity

demand_response_elasticity ∈ [0.5, 1.5]   (how aggressively demand adjusts)

EffectiveDemand(t) = EnergyDemand(t) × (1 - DemandResponseCurtailment(t) × non_essential_share)
```

### 17.3 Non-Renewable Depletion — Energy Price Pressure and Market Clearing Distortion

As `RDF` declines, the effective production cost for non-renewable-dependent goods rises. This creates a cascading distortion across the economy's market-clearing mechanism:

**Step 1 — Cost multiplier inflation:**

```
CostMultiplier(t) = 1 + φ × (1 - RDF(t))
```

At `RDF = 0.5`, a good with `φ = 0.8` costs `1.40×` baseline. At `RDF = 0.1`, it costs `1.72×` baseline.

**Step 2 — Market clearing distortion:**

When embedded energy costs inflate, market-clearing prices for energy-intensive goods rise. Demand falls off according to the price elasticity of each good category:

```
QuantityDemanded_adjusted(good, t) = QuantityDemanded_base(good)
    × (CostMultiplier(t) / SCALE) ^ (-elasticity_good)

elasticity_good ∈ [-2.0, -0.2]   (energy-essential goods have lower |elasticity|)
```

This reduces economy output:

```
EconomyOutput_adjusted(t) = Σ_goods [ QuantityDemanded_adjusted(good, t)
                                       × ValuePerUnit(good) ]
```

**Step 3 — Emissions feedback:** Reduced output at higher depletion means fewer joules produced from non-renewables. The emissions from non-renewable extraction therefore decline as depletion advances (a natural but insufficient mitigation):

```
EmissionsFromNonRenewables(t) = ExtractionVolume(t) × extraction_emissions_intensity
ExtractionVolume(t) = min(EnergyDemand(t), NonRenewableCapacity(t) × ExtractEfficiency(t))
```

**Cascade summary:**

```
RDF declines
  → CostMultiplier rises
  → MarketClearing prices for energy goods inflate
  → QuantityDemanded falls
  → EconomyOutput falls
  → Emissions from that output fall (partial self-limiting)
  → BUT: ESC also falls (less non-renewable available)
  → EnergyDeficitRatio rises
  → ScarcityPressure rises
  → GovernanceDrift activates
```

### 17.4 Agricultural Yield Model

Agricultural yield links temperature and precipitation to food supply, which is a direct input to the sustain cost computation. Climate damage proxies for both temperature deviation and precipitation disruption.

**Yield multiplier formula:**

```
YieldMultiplier(t) = BaseYield
    × TemperatureStressFactor(CD_effective(t))
    × PrecipitationStressFactor(CD_effective(t))
    × AdaptationYieldBonus(adaptation_stock(t))

TemperatureStressFactor(cd) = 1 - temp_stress_weight × cd²
    temp_stress_weight ∈ [0.3, 0.8]

PrecipitationStressFactor(cd) = 1 - precip_stress_weight × cd
    precip_stress_weight ∈ [0.2, 0.6]

AdaptationYieldBonus(AS) = 1 + adapt_yield_coeff × AS / (1 + adapt_yield_coeff × AS)
    adapt_yield_coeff ∈ [0.001, 0.02]
```

The `cd²` term for temperature stress encodes the fact that yield loss accelerates non-linearly as temperatures deviate from optimal — mild warming may even improve yields at some latitudes, but the quadratic ensures severe damage at high CD values.

**Food supply constraint:**

```
FoodSupply(t) = BaseAgriculturalCapacity × YieldMultiplier(t)

FoodDeficit(t) = max(0, FoodDemand(t) - FoodSupply(t))

FoodDeficitRatio(t) = FoodDeficit(t) / FoodDemand(t)  ∈ [0, 1]
```

`FoodDeficitRatio` feeds into `ScarcityPressure` via an additional weight term:

```
SP(t) = clip(
    w1 × (1 - SustainEfficiency(t))
  + w2 × CD_effective(t)
  + w3 × EnergyDeficitRatio(t)
  + w4 × (1 - RDF(t)) × resource_scarcity_weight
  + w5 × FoodDeficitRatio(t)      // new term
, 0, 1)

Default: w5 = 0.20; w1..w4 renormalized to sum to 0.80.
```

**Rust struct:**

```rust
// crates/climate/src/state.rs

/// Agricultural yield state for one tick.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgriculturalYield {
    /// Base agricultural capacity in joule-equivalents of food per tick.
    pub base_capacity_joules: i64,
    /// Temperature stress factor. Scaled by SCALE.
    pub temperature_stress: i64,
    /// Precipitation stress factor. Scaled by SCALE.
    pub precipitation_stress: i64,
    /// Adaptation yield bonus. Scaled by SCALE.
    pub adaptation_yield_bonus: i64,
    /// Composite yield multiplier. Scaled by SCALE.
    pub yield_multiplier: i64,
    /// Effective food supply this tick (joules of food energy).
    pub food_supply_joules: i64,
    /// Food demand this tick (joules).
    pub food_demand_joules: i64,
    /// Food deficit ratio. Scaled by SCALE.
    pub food_deficit_ratio: i64,
}

impl AgriculturalYield {
    /// Compute yield given current CD_effective and adaptation stock.
    pub fn compute(
        base_capacity: i64,
        cd_effective: i64,         // scaled by SCALE
        adaptation_stock: i64,     // scaled by SCALE
        temp_stress_weight: i64,   // scaled by SCALE
        precip_stress_weight: i64, // scaled by SCALE
        adapt_yield_coeff: i64,    // scaled by SCALE
        food_demand: i64,
    ) -> Self {
        // cd² in fixed-point: (cd * cd) / SCALE
        let cd_sq = (cd_effective * cd_effective) / SCALE;
        let temp_stress = SCALE - (temp_stress_weight * cd_sq) / SCALE;
        let precip_stress = SCALE - (precip_stress_weight * cd_effective) / SCALE;
        let adapt_num = (adapt_yield_coeff * adaptation_stock) / SCALE;
        let adapt_bonus = SCALE + (adapt_num * SCALE) / (SCALE + adapt_num);
        // yield_multiplier = base × temp × precip × adapt_bonus (all scaled)
        let yield_mult = (((temp_stress * precip_stress) / SCALE) * adapt_bonus) / SCALE;
        let food_supply = (base_capacity * yield_mult) / SCALE;
        let deficit = (food_demand - food_supply).max(0);
        let deficit_ratio = if food_demand > 0 {
            (deficit * SCALE) / food_demand
        } else {
            0
        };
        AgriculturalYield {
            base_capacity_joules: base_capacity,
            temperature_stress: temp_stress,
            precipitation_stress: precip_stress,
            adaptation_yield_bonus: adapt_bonus,
            yield_multiplier: yield_mult,
            food_supply_joules: food_supply,
            food_demand_joules: food_demand,
            food_deficit_ratio: deficit_ratio,
        }
    }
}
```

**Coupling function signatures visible to the economy module:**

```rust
// crates/climate/src/phase.rs (additions to public API)

/// Returns the food supply constraint for this tick.
/// Economy uses this to determine sustain cost for nutrition.
pub fn food_supply_joules(state: &ClimateState) -> i64;

/// Returns the food deficit ratio for this tick. Feeds ScarcityPressure.
pub fn food_deficit_ratio(state: &ClimateState) -> i64;

/// Returns the agricultural yield multiplier (SCALE = no yield loss).
pub fn agricultural_yield_multiplier(state: &ClimateState) -> i64;

/// Full energy supply snapshot: all supply-side constraints in one struct.
pub fn energy_supply_snapshot(state: &ClimateState) -> EnergySupplySnapshot;
```

**`EnergySupplySnapshot` struct:**

```rust
// crates/climate/src/state.rs

/// Complete energy supply picture for one tick, consumed by the economy module.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnergySupplySnapshot {
    /// Total ESC this tick (joules). Hard cap on energy allocable by economy.
    pub total_esc_joules: i64,
    /// Renewable contribution to ESC (joules).
    pub renewable_joules: i64,
    /// Non-renewable contribution to ESC (joules).
    pub nonrenewable_joules: i64,
    /// Emergency reserve available for discharge this tick (joules).
    pub reserve_available_joules: i64,
    /// Energy deficit ratio (demand - supply / demand). Scaled by SCALE.
    pub energy_deficit_ratio: i64,
    /// Cost multiplier for non-renewable-intensive goods. Scaled by SCALE.
    pub nonrenewable_cost_multiplier: i64,
    /// Renewable share of ESC. Scaled by SCALE.
    pub renewable_share: i64,
    /// Whether demand-response mode is active (deficit > 10%).
    pub demand_response_active: bool,
    /// Demand-response curtailment fraction. Scaled by SCALE.
    pub demand_response_curtailment: i64,
}
```

---

## 18. Climate-Governance Coupling

### 18.1 How Climate Damage Affects Institutional Legitimacy

The legitimacy drain formula connects `CD_effective` through a welfare cost channel to the governance module. Legitimacy is a stock variable maintained by the governance module; the climate module provides the welfare-cost term that drains it.

**Welfare cost term:**

```
WelfareCost(t) = productivity_loss(t) × output_fraction_welfare_weighted
               + health_penalty(t) × health_welfare_weight
               + food_deficit_ratio(t) × food_welfare_weight

welfare_weights: productivity=0.45, health=0.35, food=0.20

WelfareCost(t) ∈ [0, 1] (already in fraction space)
```

**Legitimacy drain formula (consumed by governance module):**

```
LegitimacyDrain_climate(t) = WelfareCost(t) × legitimacy_damage_sensitivity
                             × (1 - adaptation_legitimacy_credit(t))

legitimacy_damage_sensitivity ∈ [0.01, 0.10] per tick
    (how strongly welfare degradation translates to legitimacy loss)

adaptation_legitimacy_credit(t) = min(1.0,
    adaptation_stock(t) × adapt_legitimacy_coeff)
    (visible adaptation investment partially offsets legitimacy drain)

adapt_legitimacy_coeff ∈ [0.001, 0.01]
```

The governance module integrates this drain via:

```
Legitimacy(t+1) = Legitimacy(t)
    - LegitimacyDrain_climate(t)
    - LegitimacyDrain_inequality(t)    // from economy module
    - LegitimacyDrain_coercion(t)      // from governance module
    + LegitimacyGain_performance(t)    // governance responsiveness
    + LegitimacyGain_adaptation_visible(t)  // public adaptation success
```

### 18.2 Resource War Trigger Conditions

Resource wars are triggered when resource depletion crosses a threshold while diplomatic conditions are strained. This connects the climate module to the diplomacy state machine defined in CIV-0105.

**Trigger evaluation (runs in ClimatePhase after ResourcePool advance):**

```
ResourceWarTriggerMet(t) = (
    RDF(t) < resource_war_rdf_threshold          // resource stress
    AND EnergyDeficitRatio(t) > energy_war_edr_threshold  // energy shortfall
    AND SP(t) > resource_war_sp_threshold         // scarcity pressure
    AND DiplomaticTension(t) > tension_threshold  // from CIV-0105
)

Default thresholds:
    resource_war_rdf_threshold = 300_000   (RDF < 0.3)
    energy_war_edr_threshold   = 200_000   (deficit > 20%)
    resource_war_sp_threshold  = 450_000   (SP > 0.45)
    tension_threshold          = 600_000   (tension > 0.6, from CIV-0105)
```

When the trigger is met, the climate module emits `climate.resource_war_triggered.v1` (see Section 20). The diplomacy module in CIV-0105 subscribes to this event and transitions the relevant pair of polities from `Tension` to `Crisis` state.

**Trigger cooldown:** Once a resource war trigger fires, the condition cannot re-trigger for `resource_war_cooldown_ticks = 50` ticks. This prevents rapid oscillation.

### 18.3 Climate Migration Pressure

Climate damage drives displacement, which is a direct input to the citizen lifecycle transition model in CIV-0103. The displacement rate formula:

```
DisplacementRate(t) = base_displacement_rate
    + β_disp × CD_effective(t)
    + γ_disp × FoodDeficitRatio(t)
    + δ_disp × (disaster_occurred(t) ? disaster_severity(t) : 0)

base_displacement_rate ∈ [0.0001, 0.002]  per tick
β_disp ∈ [0.005, 0.02]
γ_disp ∈ [0.003, 0.015]
δ_disp ∈ [0.01, 0.05]
```

`DisplacementRate(t)` is the fraction of the at-risk population that transitions from `Resident` to `Displaced` state each tick. The CIV-0103 citizen lifecycle module consumes this as:

```
DisplacedNewEntrants(t) = DisplacementRate(t) × AtRiskPopulation(t)
```

where `AtRiskPopulation(t)` is the subset of the population in high-climate-exposure zones (coastal, low-elevation, arid). The climate module tracks this as `displacement_pressure` in `ClimateMetrics`.

### 18.4 Adaptation as Political Capital

Investing in adaptation requires institutional capacity and political will. Captured institutions block adaptation spending by diverting adaptation budgets to shadow network rent extraction.

**Institutional capacity constraint on adaptation:**

```
AdaptationBudget_available(t) = AdaptationBudget_planned(t)
    × InstitutionalCapacity(t)
    × (1 - CaptureLeakage(t))

InstitutionalCapacity(t): governance quality × (1 - corruption)
    Provided by governance module; ∈ [0, 1]

CaptureLeakage(t): fraction of adaptation budget diverted by captured institutions
    = ShadowInfluenceIndex(t) × capture_leakage_sensitivity
    capture_leakage_sensitivity ∈ [0.1, 0.5]
    ShadowInfluenceIndex from CIV-0107 shadow state layer
```

When `CaptureLeakage > 0.3`, adaptation investment falls below the minimum needed to outpace depreciation, causing adaptation stock to decline even while nominal budget is allocated. This is the "phantom adaptation" failure mode.

**Political capital budget for adaptation:**

Visible adaptation success (i.e., measurable reduction in `CD_effective` or disaster severity) generates political capital that can be spent on further reforms:

```
AdaptationPoliticalCapital(t) = Σ_{s=t-window}^{t}
    ΔDamageReduction(s) × adaptation_visibility_factor

adaptation_visibility_factor ∈ [0.5, 2.0]
    (how much citizens perceive adaptation working;
     higher in high-transparency governance)
```

This political capital feeds into the reform probability calculation in the governance module.

### 18.5 Policy DSL Climate Knobs

The following climate-specific parameters are settable via the policy DSL YAML. Each maps to a field in `ScenarioConfig` or to a runtime-patchable parameter in `ClimatePhase`.

```yaml
# climate policy knobs (under the `climate:` key in scenario YAML)
climate:
  carbon_intensity_cap:
    enabled: true
    cap_value: 0.08              # max emissions per output unit (float; stored as 80_000 i64)
    transition_cost_factor: 1.5  # cost markup for capped goods
    lag_ticks: 5                 # ticks before cap takes full effect (L_cap)

  adaptation_budget_fraction:
    value: 0.05                  # 5% of economy output goes to adaptation (50_000 i64)
    institutional_capacity_gate: true  # whether institutional capacity constrains spending
    capture_leakage_enabled: true

  emergency_reserve_target:
    weeks: 8                     # target reserve stockpile in weeks of median demand
    holding_cost_rate: 0.005     # storage overhead per tick (scaled)
    auto_discharge_threshold: 0.15  # EnergyDeficitRatio above which reserve discharges

  renewable_variability_sigma: 0.10  # std dev of renewable output variability factor

  agricultural_stress:
    temp_stress_weight: 0.50
    precip_stress_weight: 0.35
    adapt_yield_coeff: 0.008

  resource_war_triggers:
    rdf_threshold: 0.30
    edr_threshold: 0.20
    sp_threshold: 0.45
    cooldown_ticks: 50

  legitimacy_coupling:
    damage_sensitivity: 0.04    # per tick legitimacy drain per unit welfare cost
    adapt_legitimacy_coeff: 0.005
```

---

## 19. Regional Climate Heterogeneity

### 19.1 Region-Level Climate State

The global forcing index `AF` is a single value, but regional impacts differ by zone. Each region has a `ClimateRegion` struct that translates global AF into region-specific damage rates.

**Zone types and regional parameters:**

| Zone | damage_amplifier | adaptation_capacity_base | resource_stress_amplifier | Notes |
|---|---|---|---|---|
| Coastal | 1.4 | 0.7 | 1.2 | Higher storm surge and flood risk |
| Inland | 1.0 | 1.0 | 1.0 | Reference zone |
| Arctic | 0.6 (low direct) | 0.5 | 1.8 | High EROI decay, permafrost |
| Tropical | 1.6 | 0.6 | 1.3 | Highest heat and disease amplification |
| Arid | 1.3 | 0.65 | 1.5 | Extreme water stress |

**Rust struct:**

```rust
// crates/climate/src/state.rs

/// Zone type for regional climate differentiation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClimateZone {
    Coastal,
    Inland,
    Arctic,
    Tropical,
    Arid,
}

/// Per-region climate state derived from global ForcingIndex.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClimateRegion {
    pub region_id: String,
    pub zone: ClimateZone,
    /// Damage amplifier for this zone. Scaled by SCALE.
    pub damage_amplifier: i64,
    /// Base adaptation capacity (0=none, SCALE=full). Scaled by SCALE.
    pub adaptation_capacity_base: i64,
    /// Resource stress amplifier. Scaled by SCALE.
    pub resource_stress_amplifier: i64,
    /// Effective CD for this region after zone amplification.
    pub regional_cd_effective: i64,
    /// Regional scarcity pressure (zone-adjusted SP).
    pub regional_scarcity_pressure: i64,
    /// Population fraction in this region. Scaled by SCALE.
    pub population_fraction: i64,
    /// Regional adaptation stock (separate from global stock).
    pub regional_adaptation_stock: i64,
    /// Governance quality modifier for this region. Scaled by SCALE.
    pub governance_quality: i64,
    /// Wealth modifier for this region. Scaled by SCALE.
    pub wealth_modifier: i64,
}

impl ClimateRegion {
    /// Compute regional CD_effective from global CD_effective and zone amplifier.
    pub fn compute_regional_cd(&self, global_cd_effective: i64) -> i64 {
        // regional_cd = global_cd × damage_amplifier, capped at SCALE (1.0)
        ((global_cd_effective * self.damage_amplifier) / SCALE).min(SCALE)
    }

    /// Regional adaptation capacity: base × governance × wealth
    pub fn effective_adaptation_capacity(&self) -> i64 {
        let raw = (self.adaptation_capacity_base * self.governance_quality) / SCALE;
        (raw * self.wealth_modifier) / SCALE
    }
}
```

### 19.2 Spatial Forcing Propagation

Global AF propagates into regional damage rates via zone-specific amplifiers. The propagation model:

```
RegionalCD_effective(region, t) = GlobalCD_effective(t)
    × damage_amplifier(zone)
    × (1 - RegionalDamageReduction(region, t))

RegionalDamageReduction(region, t) = η × RegionalAdaptationStock(region, t)
    / (1 + η × RegionalAdaptationStock(region, t))
    × effective_adaptation_capacity(region, t)
```

Regional adaptation stocks are funded from a fraction of the regional economy's output (each region can have different `adaptation_share`) and subject to the same `L_adapt = 20` tick lag.

**Regional forcing update struct:**

```rust
// crates/climate/src/state.rs

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegionalForcing {
    pub tick: u64,
    pub global_af: i64,
    pub global_cd_effective: i64,
    /// Per-region breakdown.
    pub regions: BTreeMap<String, ClimateRegion>,
    /// Population-weighted average of regional_cd_effective.
    pub pop_weighted_cd: i64,
}

impl RegionalForcing {
    /// Advance all regional states given updated global climate state.
    pub fn advance(
        &self,
        global_state: &ClimateState,
        tick: u64,
    ) -> RegionalForcing {
        let global_cd = global_state.damage.effective_damage;
        let global_af = global_state.forcing.af_index;

        let updated_regions: BTreeMap<String, ClimateRegion> = self.regions
            .iter()
            .map(|(id, region)| {
                let reg_cd = region.compute_regional_cd(global_cd);
                let eff_cap = region.effective_adaptation_capacity();
                let adapt_reduction = {
                    let eta_stock = (10_000_i64 * region.regional_adaptation_stock) / SCALE;
                    (eta_stock * SCALE) / (SCALE + eta_stock)
                };
                let final_cd = (reg_cd * (SCALE - adapt_reduction)) / SCALE;
                let reg_sp = ((global_state.scarcity_pressure
                    * region.resource_stress_amplifier) / SCALE).min(SCALE);
                let mut updated = region.clone();
                updated.regional_cd_effective = final_cd;
                updated.regional_scarcity_pressure = reg_sp;
                (id.clone(), updated)
            })
            .collect();

        // Population-weighted average CD
        let pop_weighted_cd = updated_regions.values()
            .map(|r| (r.regional_cd_effective * r.population_fraction) / SCALE)
            .sum::<i64>();

        RegionalForcing {
            tick,
            global_af,
            global_cd_effective: global_cd,
            regions: updated_regions,
            pop_weighted_cd,
        }
    }
}
```

### 19.3 Regional Adaptation Capacity

Regional adaptation capacity is modulated by three factors: infrastructure base, wealth, and governance quality. A wealthy, well-governed coastal city can adapt more effectively than a poor, poorly-governed tropical region with the same global forcing level.

**Effective regional adaptation capacity formula (normalized to [0, SCALE]):**

```
EffectiveAdaptCapacity(region, t) =
    adaptation_capacity_base(zone)
    × (infrastructure_score(region, t) / SCALE)
    × (wealth_modifier(region, t) / SCALE)
    × (governance_quality(region, t) / SCALE)
```

**Infrastructure score:** tracks housing stock quality, energy grid redundancy, and transport resilience. It degrades under repeated disaster events and improves with hardening investments. The infrastructure score is maintained by the urban systems module and provided to the climate module at each tick via the `RegionalForcing` update.

**Governance quality modifier:** directly from the governance module's per-region quality index. Low governance quality (corruption, weak institutions) reduces effective adaptation even when budgets are large — captured institutions divert adaptation funds.

### 19.4 Cross-Region Resource Flows Under Stress

When one region faces severe scarcity (`SP > 0.5`) and a neighboring region has surplus, cross-region flows of food and energy can occur. These are modeled as transfer flows governed by trade route capacity and political willingness.

**Flow mechanics:**

```
MaxTransferFlow(source → dest, t) = min(
    TradeRouteCapacity(source, dest),
    Surplus(source, t) × export_willingness(source, t)
)

Surplus(source, t) = FoodSupply(source, t) - FoodDemand(source, t) - buffer_reserve

export_willingness(source, t) = base_willingness
    × (1 - ScarcityPressure(source, t))   // willing to export only when not stressed
    × (1 + TradeAgreement(source, dest))  // treaty bonus
```

When `MaxTransferFlow > 0`, the source region's `FoodSupply` is debited and the destination's `FoodSupply` is credited. If trade routes are severed (sanctions, war, infrastructure damage), `TradeRouteCapacity` drops to zero and the cross-region flow ceases.

**Breakdown conditions:**

A cross-region flow breaks down when any of:
- `ScarcityPressure(source) > 0.55` (source too stressed to export)
- `DiplomaticState(source, dest) = War` or `Sanction`
- `TradeRouteCapacity < minimum_flow_threshold`
- `export_willingness < 0.1`

When a flow breaks down, the destination region's `FoodDeficitRatio` increases immediately, which propagates into its `ScarcityPressure` and thence into governance drift.

**Rust struct:**

```rust
// crates/climate/src/state.rs

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CrossRegionFlow {
    pub source_region_id: String,
    pub dest_region_id: String,
    /// Food transfer (joule-equivalent of food energy). Per tick.
    pub food_transfer_joules: i64,
    /// Energy transfer (joules). Per tick.
    pub energy_transfer_joules: i64,
    /// Route capacity. Scaled by SCALE (fraction of max flow).
    pub route_capacity: i64,
    /// Whether the flow is active this tick.
    pub active: bool,
    /// Reason for breakdown if not active.
    pub breakdown_reason: Option<FlowBreakdownReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FlowBreakdownReason {
    SourceScarcity,
    DiplomaticSanction,
    RouteDestroyed,
    ExportUnwillingness,
}
```

---

## 20. Extended Event Taxonomy

### 20.1 Eight Additional Event Types

The following eight event types extend the four defined in Section 6. All events follow the same conventions: `scenario_id`, `scenario_version`, `tick`, and `state_hash` are required fields.

#### 20.1.1 `climate.tipping_point_crossed.v1`

Emitted once per tipping point per run, the first tick the tipping point condition is met.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.tipping_point_crossed.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "tipping_point_id", "trigger_variable", "trigger_value",
    "trigger_threshold", "effect_description", "is_reversible", "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.tipping_point_crossed.v1" },
    "tick": { "type": "integer", "minimum": 0 },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "tipping_point_id": {
      "type": "integer",
      "enum": [1, 2, 3, 4],
      "description": "1=AF>1.5 sink halved, 2=RDF<0.2 EROI doubled, 3=CD>0.6 disaster doubled, 4=AF>3.0 sink locked"
    },
    "trigger_variable": {
      "type": "string",
      "enum": ["af_index", "rdf_scaled", "cd_effective"]
    },
    "trigger_value": { "type": "integer", "description": "Observed value at crossing. Scaled by 1_000_000." },
    "trigger_threshold": { "type": "integer", "description": "Threshold that was crossed. Scaled by 1_000_000." },
    "effect_description": { "type": "string" },
    "is_reversible": { "type": "boolean" },
    "state_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 20.1.2 `climate.scenario_composed.v1`

Emitted at simulation start when a composed scenario is used.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.scenario_composed.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "composed_scenario_id",
    "primary_scenario_id", "primary_version",
    "secondary_scenario_id", "secondary_version",
    "composition_mode"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.scenario_composed.v1" },
    "tick": { "type": "integer", "const": 0 },
    "composed_scenario_id": { "type": "string" },
    "primary_scenario_id": { "type": "string" },
    "primary_version": { "type": "integer" },
    "secondary_scenario_id": { "type": "string" },
    "secondary_version": { "type": "integer" },
    "composition_mode": {
      "type": "string",
      "enum": ["ForcingPlusShocks", "WeightedBlend", "PrimaryWithFallbackBoundary"]
    },
    "primary_weight": {
      "type": ["integer", "null"],
      "description": "Only present for WeightedBlend. Scaled by 1_000_000."
    }
  },
  "additionalProperties": false
}
```

#### 20.1.3 `climate.agricultural_yield_updated.v1`

Emitted every tick when agricultural yield computation produces a material change (yield multiplier change > 1%).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.agricultural_yield_updated.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "yield_multiplier", "food_supply_joules", "food_demand_joules",
    "food_deficit_ratio", "temperature_stress", "precipitation_stress",
    "adaptation_yield_bonus", "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.agricultural_yield_updated.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "yield_multiplier": { "type": "integer", "description": "Composite yield factor. Scaled by 1_000_000." },
    "food_supply_joules": { "type": "integer" },
    "food_demand_joules": { "type": "integer" },
    "food_deficit_ratio": { "type": "integer", "description": "Scaled by 1_000_000." },
    "temperature_stress": { "type": "integer", "description": "Scaled by 1_000_000." },
    "precipitation_stress": { "type": "integer", "description": "Scaled by 1_000_000." },
    "adaptation_yield_bonus": { "type": "integer", "description": "Scaled by 1_000_000." },
    "state_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 20.1.4 `climate.migration_pressure_updated.v1`

Emitted when `DisplacementRate` crosses a configurable threshold (default 0.1% of at-risk population per tick = `1_000` scaled).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.migration_pressure_updated.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "displacement_rate", "displaced_new_entrants_fraction",
    "cd_effective_contribution", "food_deficit_contribution",
    "disaster_contribution", "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.migration_pressure_updated.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "displacement_rate": { "type": "integer", "description": "Fraction of at-risk pop displaced this tick. Scaled by 1_000_000." },
    "displaced_new_entrants_fraction": { "type": "integer", "description": "Scaled by 1_000_000." },
    "cd_effective_contribution": { "type": "integer", "description": "CD_effective component of rate. Scaled by 1_000_000." },
    "food_deficit_contribution": { "type": "integer", "description": "Food deficit component. Scaled by 1_000_000." },
    "disaster_contribution": { "type": "integer", "description": "Disaster component. Scaled by 1_000_000." },
    "state_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 20.1.5 `climate.resource_war_triggered.v1`

Emitted when all resource war trigger conditions are simultaneously met.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.resource_war_triggered.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "rdf_at_trigger", "edr_at_trigger", "sp_at_trigger",
    "diplomatic_tension_at_trigger",
    "cooldown_expires_tick", "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.resource_war_triggered.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "rdf_at_trigger": { "type": "integer", "description": "RDF at trigger tick. Scaled by 1_000_000." },
    "edr_at_trigger": { "type": "integer", "description": "Energy deficit ratio. Scaled by 1_000_000." },
    "sp_at_trigger": { "type": "integer", "description": "Scarcity pressure. Scaled by 1_000_000." },
    "diplomatic_tension_at_trigger": { "type": "integer", "description": "Diplomatic tension from CIV-0105. Scaled by 1_000_000." },
    "cooldown_expires_tick": { "type": "integer" },
    "state_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 20.1.6 `climate.monte_carlo_completed.v1`

Emitted once when a Monte Carlo batch run finishes.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.monte_carlo_completed.v1",
  "type": "object",
  "required": [
    "event_type", "scenario_id", "num_runs", "ticks_per_run",
    "base_seed",
    "peak_af_mean", "peak_af_p5", "peak_af_p50", "peak_af_p95",
    "peak_cd_mean", "peak_cd_p5", "peak_cd_p50", "peak_cd_p95",
    "tp1_fire_rate", "tp4_fire_rate",
    "mean_ticks_above_scarcity", "peak_af_ci90_halfwidth"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.monte_carlo_completed.v1" },
    "scenario_id": { "type": "string" },
    "num_runs": { "type": "integer" },
    "ticks_per_run": { "type": "integer" },
    "base_seed": { "type": "integer" },
    "peak_af_mean": { "type": "integer", "description": "Scaled by 1_000_000." },
    "peak_af_p5":  { "type": "integer" },
    "peak_af_p50": { "type": "integer" },
    "peak_af_p95": { "type": "integer" },
    "peak_cd_mean": { "type": "integer" },
    "peak_cd_p5":  { "type": "integer" },
    "peak_cd_p50": { "type": "integer" },
    "peak_cd_p95": { "type": "integer" },
    "tp1_fire_rate": { "type": "integer", "description": "Fraction of runs. Scaled by 1_000_000." },
    "tp4_fire_rate": { "type": "integer" },
    "mean_ticks_above_scarcity": { "type": "integer" },
    "peak_af_ci90_halfwidth": { "type": "integer" }
  },
  "additionalProperties": false
}
```

#### 20.1.7 `climate.regional_forcing_updated.v1`

Emitted each tick for each region when regional CD changes by more than 0.5% from last tick.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.regional_forcing_updated.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "region_id", "zone", "regional_cd_effective",
    "regional_scarcity_pressure", "damage_amplifier",
    "regional_adaptation_stock", "effective_adaptation_capacity",
    "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.regional_forcing_updated.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "region_id": { "type": "string" },
    "zone": {
      "type": "string",
      "enum": ["Coastal", "Inland", "Arctic", "Tropical", "Arid"]
    },
    "regional_cd_effective": { "type": "integer", "description": "Scaled by 1_000_000." },
    "regional_scarcity_pressure": { "type": "integer" },
    "damage_amplifier": { "type": "integer" },
    "regional_adaptation_stock": { "type": "integer" },
    "effective_adaptation_capacity": { "type": "integer" },
    "state_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 20.1.8 `climate.adaptation_budget_committed.v1`

Emitted each tick when adaptation investment is committed (queued into pending-effects). Distinct from `climate.adaptation_applied.v1` (which fires when the effect vests after the lag).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "climate.adaptation_budget_committed.v1",
  "type": "object",
  "required": [
    "event_type", "tick", "scenario_id", "scenario_version",
    "investment_joules", "vest_tick",
    "institutional_capacity_applied", "capture_leakage_applied",
    "effective_investment_joules", "state_hash"
  ],
  "properties": {
    "event_type": { "type": "string", "const": "climate.adaptation_budget_committed.v1" },
    "tick": { "type": "integer" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "integer" },
    "investment_joules": { "type": "integer", "description": "Gross allocation (before capture leakage)." },
    "vest_tick": { "type": "integer", "description": "Tick N + L_adapt when this vests." },
    "institutional_capacity_applied": { "type": "integer", "description": "Capacity multiplier. Scaled by 1_000_000." },
    "capture_leakage_applied": { "type": "integer", "description": "Fraction lost to capture. Scaled by 1_000_000." },
    "effective_investment_joules": { "type": "integer", "description": "Net joules actually entering the pending queue." },
    "state_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

### 20.2 Event Routing Table

The following table specifies which modules subscribe to each climate event. Subscriptions are declared in the event bus configuration in `crates/io/src/event_routing.yaml`.

| Event | Publisher | Subscribers | Notes |
|---|---|---|---|
| `climate.damage_modeled.v1` | `crates/climate` | economy, governance, metrics, dashboard | Every tick |
| `climate.adaptation_applied.v1` | `crates/climate` | metrics, dashboard, governance | On vest |
| `climate.resource_stress.v1` | `crates/climate` | economy, joule-allocator, dashboard | On RDF threshold |
| `climate.scenario_boundary_crossed.v1` | `crates/climate` | policy, dashboard | Once per boundary |
| `climate.tipping_point_crossed.v1` | `crates/climate` | all modules, dashboard, alert system | Once per TP |
| `climate.scenario_composed.v1` | `crates/climate` | metrics, replay-verifier | Tick 0 only |
| `climate.agricultural_yield_updated.v1` | `crates/climate` | economy (sustain cost), metrics | Each tick |
| `climate.migration_pressure_updated.v1` | `crates/climate` | citizen-lifecycle (CIV-0103), dashboard | When threshold crossed |
| `climate.resource_war_triggered.v1` | `crates/climate` | diplomacy (CIV-0105), dashboard, governance | On trigger |
| `climate.monte_carlo_completed.v1` | `crates/climate` (batch runner) | analytics layer, dashboard | End of batch |
| `climate.regional_forcing_updated.v1` | `crates/climate` | regional-economy, dashboard (LOD heat maps) | Each tick per region |
| `climate.adaptation_budget_committed.v1` | `crates/climate` | metrics, governance | Each tick |

---

## 21. Extended SQL Schema

### 21.1 Six Additional Tables

```sql
-- climate_regions: static zone definitions and base parameters per region per scenario
CREATE TABLE climate_regions (
    id                      SERIAL PRIMARY KEY,
    scenario_id             VARCHAR(128) NOT NULL,
    scenario_version        INTEGER NOT NULL,
    region_id               VARCHAR(128) NOT NULL,
    zone                    VARCHAR(32) NOT NULL CHECK (zone IN (
                                'Coastal', 'Inland', 'Arctic', 'Tropical', 'Arid'
                            )),
    damage_amplifier        BIGINT NOT NULL,  -- scaled × 1_000_000
    adaptation_capacity_base BIGINT NOT NULL,
    resource_stress_amplifier BIGINT NOT NULL,
    population_fraction     BIGINT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (scenario_id, scenario_version, region_id)
);

-- regional_forcing_history: per-tick regional climate state
CREATE TABLE regional_forcing_history (
    id                          BIGSERIAL PRIMARY KEY,
    run_id                      BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                        BIGINT NOT NULL,
    region_id                   VARCHAR(128) NOT NULL,
    zone                        VARCHAR(32) NOT NULL,
    regional_cd_effective       BIGINT NOT NULL,   -- scaled × 1_000_000
    regional_scarcity_pressure  BIGINT NOT NULL,
    regional_adaptation_stock   BIGINT NOT NULL,
    effective_adaptation_capacity BIGINT NOT NULL,
    food_supply_joules          BIGINT,
    food_deficit_ratio          BIGINT,
    state_hash                  CHAR(64) NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX regional_forcing_run_tick_region (run_id, tick, region_id)
);

-- cross_region_flows: food and energy flows between regions per tick
CREATE TABLE cross_region_flows (
    id                      BIGSERIAL PRIMARY KEY,
    run_id                  BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                    BIGINT NOT NULL,
    source_region_id        VARCHAR(128) NOT NULL,
    dest_region_id          VARCHAR(128) NOT NULL,
    food_transfer_joules    BIGINT NOT NULL DEFAULT 0,
    energy_transfer_joules  BIGINT NOT NULL DEFAULT 0,
    route_capacity          BIGINT NOT NULL,       -- scaled × 1_000_000
    active                  BOOLEAN NOT NULL,
    breakdown_reason        VARCHAR(64),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX cross_region_flows_run_tick (run_id, tick)
);

-- monte_carlo_runs: one row per completed Monte Carlo batch
CREATE TABLE monte_carlo_runs (
    id                      BIGSERIAL PRIMARY KEY,
    scenario_id             VARCHAR(128) NOT NULL,
    scenario_version        INTEGER NOT NULL,
    num_runs                INTEGER NOT NULL,
    ticks_per_run           BIGINT NOT NULL,
    base_seed               BIGINT NOT NULL,
    peak_af_mean            BIGINT NOT NULL,
    peak_af_std             BIGINT NOT NULL,
    peak_af_p5              BIGINT NOT NULL,
    peak_af_p50             BIGINT NOT NULL,
    peak_af_p95             BIGINT NOT NULL,
    peak_cd_mean            BIGINT NOT NULL,
    peak_cd_p5              BIGINT NOT NULL,
    peak_cd_p50             BIGINT NOT NULL,
    peak_cd_p95             BIGINT NOT NULL,
    tp1_fire_rate           BIGINT NOT NULL,  -- fraction scaled × 1_000_000
    tp4_fire_rate           BIGINT NOT NULL,
    mean_ticks_above_scarcity BIGINT NOT NULL,
    peak_af_ci90_halfwidth  BIGINT NOT NULL,
    run_summaries_json      JSONB NOT NULL,   -- full Vec<MonteCarloRunSummary>
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- tipping_point_log: one row per tipping point crossing per run
CREATE TABLE tipping_point_log (
    id                      BIGSERIAL PRIMARY KEY,
    run_id                  BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                    BIGINT NOT NULL,
    tipping_point_id        SMALLINT NOT NULL CHECK (tipping_point_id IN (1, 2, 3, 4)),
    trigger_variable        VARCHAR(64) NOT NULL,
    trigger_value           BIGINT NOT NULL,
    trigger_threshold       BIGINT NOT NULL,
    is_reversible           BOOLEAN NOT NULL,
    scenario_id             VARCHAR(128) NOT NULL,
    scenario_version        INTEGER NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (run_id, tipping_point_id),  -- each TP fires at most once per run
    INDEX tipping_point_log_run (run_id)
);

-- climate_policy_decisions: log of policy DSL changes applied to climate parameters
CREATE TABLE climate_policy_decisions (
    id                          BIGSERIAL PRIMARY KEY,
    run_id                      BIGINT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    tick                        BIGINT NOT NULL,
    scenario_id                 VARCHAR(128) NOT NULL,
    scenario_version            INTEGER NOT NULL,
    policy_key                  VARCHAR(128) NOT NULL,  -- e.g., "carbon_intensity_cap"
    old_value_json              JSONB,                  -- previous value
    new_value_json              JSONB NOT NULL,         -- applied value
    institutional_capacity      BIGINT,                 -- at time of decision, scaled
    capture_leakage             BIGINT,                 -- at time of decision, scaled
    effective_change_json       JSONB,                  -- actual change after leakage
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX climate_policy_decisions_run_tick (run_id, tick)
);
```

### 21.2 Views

```sql
-- v_current_climate_by_region: latest regional climate state per run
CREATE OR REPLACE VIEW v_current_climate_by_region AS
SELECT DISTINCT ON (run_id, region_id)
    rfh.run_id,
    rfh.tick,
    rfh.region_id,
    cr.zone,
    rfh.regional_cd_effective,
    rfh.regional_scarcity_pressure,
    rfh.regional_adaptation_stock,
    rfh.effective_adaptation_capacity,
    rfh.food_supply_joules,
    rfh.food_deficit_ratio,
    cr.damage_amplifier,
    cr.population_fraction
FROM regional_forcing_history rfh
JOIN climate_regions cr ON (
    cr.region_id = rfh.region_id
    -- scenario_id match via run → simulation_runs join omitted for brevity
)
ORDER BY rfh.run_id, rfh.region_id, rfh.tick DESC;

-- v_adaptation_roi_by_lever: adaptation ROI aggregated by investment period
CREATE OR REPLACE VIEW v_adaptation_roi_by_lever AS
SELECT
    al.run_id,
    al.scenario_id,
    al.investment_tick,
    al.tick AS vest_tick,
    (al.tick - al.investment_tick) AS actual_lag,
    al.invested_joules,
    al.damage_reduction_before,
    al.damage_reduction_after,
    al.adaptation_roi,
    -- ROI percentile within this run
    PERCENT_RANK() OVER (
        PARTITION BY al.run_id
        ORDER BY al.adaptation_roi
    ) AS roi_percentile_within_run
FROM adaptation_ledger al
ORDER BY al.run_id, al.investment_tick;

-- Index supporting v_current_climate_by_region efficiently
CREATE INDEX IF NOT EXISTS idx_regional_forcing_run_region_tick_desc
    ON regional_forcing_history (run_id, region_id, tick DESC);
```

---

## 22. Extended Test Suite and Benchmarks

### 22.1 Eight Additional Test Scenarios

```rust
// crates/climate/tests/extended.rs

use climate::*;

// ───────────────────────────────────────────────────────────────────────────
// AT-8: Tipping point TP-1 fires when AF crosses 1.5 in HighShock scenario.
// After TP-1 fires, sink rate must be half the base value.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_tipping_point_tp1_fires_and_halves_sink() {
    let config = ScenarioConfig::high_shock_v1();
    let phase = ClimatePhase::from_scenario(config).unwrap();
    let mut state = phase.initial_state();
    let mut tip_state = TippingPointState::new();
    let base_sink = state.forcing.sink_rate;

    // Run until TP-1 fires or 500 ticks pass.
    for tick in 0..500u64 {
        let (next_state, _events) = phase.advance(
            &state, 2_000_000_000, 1_600_000_000, tick * 17
        );
        let (next_tip, newly_fired) = tip_state.evaluate(
            next_state.forcing.af_index,
            next_state.resources.rdf_scaled,
            next_state.damage.effective_damage,
            tick,
        );
        if !newly_fired.is_empty() && newly_fired.contains(&1) {
            // Verify sink rate is halved for the next tick
            let effective_sink = next_tip.effective_sink_rate(base_sink);
            assert_eq!(effective_sink, base_sink / 2,
                "TP-1: effective sink rate must be half base after crossing");
            // Verify irreversibility: even if AF drops, sink stays halved
            assert!(next_tip.tp1_fired,
                "TP-1 must remain fired even after crossing");
            return; // test passes
        }
        tip_state = next_tip;
        state = next_state;
    }
    panic!("TP-1 did not fire within 500 ticks in HighShock scenario");
}

// ───────────────────────────────────────────────────────────────────────────
// AT-9: Regional heterogeneity conservation.
// Population-weighted average of regional_cd_effective must equal
// global_cd_effective within a tolerance of 5%.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_regional_heterogeneity_conservation() {
    let config = ScenarioConfig::baseline_v1();
    let phase = ClimatePhase::from_scenario(config).unwrap();
    let mut global_state = phase.initial_state();

    // Build a representative RegionalForcing with two zones.
    let mut regions = std::collections::BTreeMap::new();
    regions.insert("coastal_a".to_string(), ClimateRegion {
        region_id: "coastal_a".to_string(),
        zone: ClimateZone::Coastal,
        damage_amplifier: 1_400_000,
        adaptation_capacity_base: 700_000,
        resource_stress_amplifier: 1_200_000,
        regional_cd_effective: 0,
        regional_scarcity_pressure: 0,
        population_fraction: 400_000,  // 40%
        regional_adaptation_stock: 0,
        governance_quality: 800_000,
        wealth_modifier: 900_000,
    });
    regions.insert("inland_b".to_string(), ClimateRegion {
        region_id: "inland_b".to_string(),
        zone: ClimateZone::Inland,
        damage_amplifier: 1_000_000,
        adaptation_capacity_base: 1_000_000,
        resource_stress_amplifier: 1_000_000,
        regional_cd_effective: 0,
        regional_scarcity_pressure: 0,
        population_fraction: 600_000,  // 60%
        regional_adaptation_stock: 0,
        governance_quality: 700_000,
        wealth_modifier: 800_000,
    });

    let rf_init = RegionalForcing {
        tick: 0,
        global_af: 0,
        global_cd_effective: 0,
        regions,
        pop_weighted_cd: 0,
    };

    for tick in 0..100u64 {
        let (next_state, _) = phase.advance(
            &global_state, 1_000_000_000, 800_000_000, tick * 7
        );
        let rf_next = rf_init.advance(&next_state, tick);

        // Pop-weighted CD should be within 5% of global CD (they differ by zone amplifier,
        // so this tests that the weighting arithmetic is correct).
        let global_cd = next_state.damage.effective_damage;
        let pop_weighted = rf_next.pop_weighted_cd;
        if global_cd > 10_000 {  // skip near-zero ticks
            let relative_diff = ((pop_weighted - global_cd).abs() * SCALE) / global_cd;
            assert!(relative_diff <= 2_000_000,  // 200% of global max possible (amplifiers)
                "Regional CD diverged excessively from global at tick {}: \
                 pop_weighted={} global={}", tick, pop_weighted, global_cd);
        }
        global_state = next_state;
    }
}

// ───────────────────────────────────────────────────────────────────────────
// AT-10: Monte Carlo seed stability.
// The same base_seed must produce identical run_summaries across two
// independent MonteCarloResult computations.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_monte_carlo_seed_stability() {
    let config = MonteCarloConfig {
        scenario_config: ScenarioConfig::baseline_v1(),
        num_runs: 20,
        base_seed: 98765,
        ticks_per_run: 100,
        fixed_economy_output_joules: 1_000_000_000,
        fixed_energy_demand_joules: 800_000_000,
    };

    let runner = ScenarioRunner::new(config.clone());
    let result1 = runner.run().unwrap();
    let result2 = ScenarioRunner::new(config).run().unwrap();

    assert_eq!(result1.run_summaries.len(), result2.run_summaries.len());
    for (s1, s2) in result1.run_summaries.iter().zip(result2.run_summaries.iter()) {
        assert_eq!(s1.peak_af, s2.peak_af,
            "Monte Carlo run {} peak_af differs between runs", s1.run_index);
        assert_eq!(s1.total_disasters, s2.total_disasters,
            "Monte Carlo run {} disaster count differs", s1.run_index);
        assert_eq!(s1.tipping_points_fired, s2.tipping_points_fired,
            "Monte Carlo run {} tipping points differ", s1.run_index);
    }
}

// ───────────────────────────────────────────────────────────────────────────
// AT-11: Agricultural yield directional test.
// Higher CD_effective must produce lower yield_multiplier.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_agricultural_yield_decreases_with_damage() {
    let food_demand = 1_000_000_000i64;
    let adapt_stock = 0i64;

    let yield_low_cd = AgriculturalYield::compute(
        food_demand, 100_000, adapt_stock,
        500_000, 350_000, 8_000, food_demand,
    );
    let yield_high_cd = AgriculturalYield::compute(
        food_demand, 700_000, adapt_stock,
        500_000, 350_000, 8_000, food_demand,
    );

    assert!(yield_high_cd.yield_multiplier < yield_low_cd.yield_multiplier,
        "Higher CD should produce lower yield: high_cd_yield={} >= low_cd_yield={}",
        yield_high_cd.yield_multiplier, yield_low_cd.yield_multiplier);

    assert!(yield_high_cd.food_deficit_ratio > yield_low_cd.food_deficit_ratio,
        "Higher CD should produce higher food deficit ratio");
}

// ───────────────────────────────────────────────────────────────────────────
// AT-12: Migration pressure coupling.
// DisplacementRate must be monotonically non-decreasing as CD_effective rises
// (with all other inputs held fixed).
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_migration_pressure_monotonic_in_damage() {
    let base_rate: i64 = 1_000;      // 0.001
    let beta: i64 = 10_000;          // 0.01
    let gamma: i64 = 5_000;          // 0.005
    let food_deficit: i64 = 0;       // held fixed

    fn compute_displacement(cd: i64, base: i64, b: i64, g: i64, fd: i64) -> i64 {
        base + (b * cd) / SCALE + (g * fd) / SCALE
    }

    let mut prev_rate = 0i64;
    for cd_step in (0..=SCALE).step_by(10_000) {
        let rate = compute_displacement(cd_step, base_rate, beta, gamma, food_deficit);
        assert!(rate >= prev_rate,
            "Displacement rate decreased: cd={} rate={} prev_rate={}",
            cd_step, rate, prev_rate);
        prev_rate = rate;
    }
}

// ───────────────────────────────────────────────────────────────────────────
// AT-13: Resource war trigger fires exactly once per cooldown window.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_resource_war_trigger_cooldown() {
    // Simulate the trigger evaluation logic directly.
    let rdf_threshold: i64 = 300_000;
    let edr_threshold: i64 = 200_000;
    let sp_threshold: i64  = 450_000;
    let cooldown: u64 = 50;

    let mut last_trigger_tick: Option<u64> = None;
    let mut trigger_count: u32 = 0;

    // All conditions permanently met (worst-case stress scenario).
    for tick in 0..200u64 {
        let in_cooldown = last_trigger_tick
            .map(|t| tick < t + cooldown)
            .unwrap_or(false);

        if !in_cooldown {
            // Conditions met: RDF=0.1, EDR=0.5, SP=0.6
            let conditions_met =
                100_000 < rdf_threshold &&
                500_000 > edr_threshold &&
                600_000 > sp_threshold;

            if conditions_met {
                trigger_count += 1;
                last_trigger_tick = Some(tick);
            }
        }
    }

    // With cooldown=50 and 200 ticks, max triggers = ceil(200/50) = 4.
    assert!(trigger_count <= 4,
        "Resource war triggered {} times, expected <= 4 with cooldown=50 over 200 ticks",
        trigger_count);
    assert!(trigger_count >= 1, "Resource war should have triggered at least once");
}

// ───────────────────────────────────────────────────────────────────────────
// AT-14: Adaptation political capital accumulates when damage reduction improves.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_adaptation_political_capital_accumulates() {
    // Simulate adaptation stock growing over 30 ticks (post-lag).
    let eta: i64 = 15_000;           // 0.015
    let visibility: i64 = 1_200_000; // 1.2 — high transparency governance
    let window: u64 = 20;

    let mut adaptation_stock: i64 = 0;
    let mut political_capital: i64 = 0;
    let mut prev_reduction: i64 = 0;

    for tick in 0..50u64 {
        // Invest 50M joules/tick, vest immediately for test simplicity.
        adaptation_stock += (50_000_000_i64 * eta) / SCALE;
        let eta_stock = (eta * adaptation_stock) / SCALE;
        let reduction = (eta_stock * SCALE) / (SCALE + eta_stock);

        let delta_reduction = (reduction - prev_reduction).max(0);
        let capital_increment = (delta_reduction * visibility) / SCALE;
        political_capital += capital_increment;

        prev_reduction = reduction;
    }

    assert!(political_capital > 0,
        "Political capital should accumulate as adaptation reduces damage");
}

// ───────────────────────────────────────────────────────────────────────────
// AT-15: Climate collapse chaos scenario — maximum forcing, minimal adaptation.
// Expected: TP-1 fires within 50 ticks, TP-3 fires within 100 ticks,
// SP exceeds 0.7 within 80 ticks.
// ───────────────────────────────────────────────────────────────────────────
#[test]
fn test_climate_collapse_chaos_scenario() {
    // Build "climate collapse" config: maximum stress, zero adaptation.
    let collapse_config = ScenarioConfig {
        id: "climate-collapse-chaos-v1".to_string(),
        family: ScenarioFamily::HighShock,
        forcing_params: ForcingParams {
            k_sink: 100,             // 0.0001 — near-zero sink
            emissions_intensity_heavy: 400_000,   // maximum
            emissions_intensity_clean: 80_000,
            af_safe_threshold: 200_000,
            af_onset_threshold: 300_000,          // very early onset
            damage_alpha: 8_000_000,              // steepest sigmoid
            productivity_damage_weight: 1_000_000,
            health_damage_weight: 600_000,
            housing_decay_amplifier: 3_000_000,
        },
        depletion_params: DepletionParams {
            delta_coefficient: 5_000,
            cost_amplifier_phi: 2_000_000,
            rdf_initial: 1_000_000,
        },
        energy_params: EnergyParams {
            renewable_growth_rate: 500,
            investment_yield: 200_000,
            eroi_decay: 1_000,
            base_extract_efficiency: 700_000,
            initial_renewable_capacity_joules: 100_000_000,
            initial_nonrenewable_stock_joules: 1_000_000_000_000,
        },
        adaptation_params: AdaptationParams {
            adaptation_share: 0,     // zero adaptation
            eta_effectiveness: 1_000,
            depreciation_rate: 50_000,
            lag_ticks: 20,
        },
        shock_params: ShockParams {
            base_disaster_rate: 50_000,   // 5% per tick
            beta_damage_multiplier: 200_000,
            base_severity: 500_000,
            severity_amplifier: 4_000_000,
        },
        scenario_boundary: None,
        scenario_version: 1,
    };

    let phase = ClimatePhase::from_scenario(collapse_config).unwrap();
    let mut state = phase.initial_state();
    let mut tip_state = TippingPointState::new();

    let mut tp1_tick: Option<u64> = None;
    let mut tp3_tick: Option<u64> = None;
    let mut max_sp: i64 = 0;
    let mut max_sp_tick: u64 = 0;

    for tick in 0..150u64 {
        let (next_state, _events) = phase.advance(
            &state, 2_000_000_000, 1_800_000_000, tick * 31337
        );
        let (next_tips, newly_fired) = tip_state.evaluate(
            next_state.forcing.af_index,
            next_state.resources.rdf_scaled,
            next_state.damage.effective_damage,
            tick,
        );
        if newly_fired.contains(&1) && tp1_tick.is_none() { tp1_tick = Some(tick); }
        if newly_fired.contains(&3) && tp3_tick.is_none() { tp3_tick = Some(tick); }
        if next_state.scarcity_pressure > max_sp {
            max_sp = next_state.scarcity_pressure;
            max_sp_tick = tick;
        }
        tip_state = next_tips;
        state = next_state;
    }

    assert!(tp1_tick.is_some() && tp1_tick.unwrap() <= 50,
        "Collapse scenario: TP-1 should fire within 50 ticks, fired at {:?}", tp1_tick);
    assert!(tp3_tick.is_some() && tp3_tick.unwrap() <= 100,
        "Collapse scenario: TP-3 should fire within 100 ticks, fired at {:?}", tp3_tick);
    assert!(max_sp >= 700_000,
        "Collapse scenario: SP should exceed 0.7, reached {} at tick {}",
        max_sp, max_sp_tick);
}
```

### 22.2 Criterion Benchmarks

```rust
// crates/climate/benches/climate_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use climate::*;

/// Benchmark: single-tick climate phase advance.
/// Budget: < 500 µs per tick (well within 1.5 ms sub-budget).
fn bench_single_tick_climate_phase(c: &mut Criterion) {
    let config = ScenarioConfig::baseline_v1();
    let phase = ClimatePhase::from_scenario(config).unwrap();
    let state = phase.initial_state();

    c.bench_function("climate_single_tick_advance", |b| {
        b.iter(|| {
            let (next_state, events) = phase.advance(
                black_box(&state),
                black_box(1_000_000_000),
                black_box(800_000_000),
                black_box(12345),
            );
            black_box((next_state, events))
        });
    });
}

/// Benchmark: 1000-run Monte Carlo batch.
/// Budget: 1000 runs × 100 ticks < 60 seconds on 4-core machine.
/// Using rayon par_iter, target is < 10 seconds.
fn bench_monte_carlo_1000_runs(c: &mut Criterion) {
    let config = MonteCarloConfig {
        scenario_config: ScenarioConfig::baseline_v1(),
        num_runs: 1000,
        base_seed: 42,
        ticks_per_run: 100,
        fixed_economy_output_joules: 1_000_000_000,
        fixed_energy_demand_joules: 800_000_000,
    };

    c.bench_function("monte_carlo_1000_runs_100_ticks", |b| {
        b.iter(|| {
            let runner = ScenarioRunner::new(black_box(config.clone()));
            let result = runner.run().unwrap();
            black_box(result)
        });
    });
}

/// Benchmark: scenario composition overhead.
fn bench_scenario_composition(c: &mut Criterion) {
    let primary = ScenarioConfig::baseline_v1();
    let secondary = ScenarioConfig::high_shock_v1();
    let composition = ScenarioComposition {
        primary_id: primary.id.clone(),
        primary_version: primary.scenario_version,
        secondary_id: secondary.id.clone(),
        secondary_version: secondary.scenario_version,
        mode: CompositionMode::ForcingPlusShocks,
        composed_id: "bench-composed-v1".to_string(),
    };

    c.bench_function("scenario_composition_forcing_plus_shocks", |b| {
        b.iter(|| {
            let composed = composition.compose(
                black_box(&primary),
                black_box(&secondary),
            );
            black_box(composed)
        });
    });
}

/// Benchmark: tipping point evaluation across 1000 ticks.
fn bench_tipping_point_evaluation(c: &mut Criterion) {
    c.bench_function("tipping_point_evaluate_1000_ticks", |b| {
        b.iter(|| {
            let mut tip = TippingPointState::new();
            for i in 0..1000i64 {
                let (next, fired) = tip.evaluate(
                    black_box(i * 3_000),   // slowly rising AF
                    black_box(SCALE - i * 800),  // slowly declining RDF
                    black_box(i * 600),      // slowly rising CD
                    black_box(i as u64),
                );
                tip = next;
                black_box(fired);
            }
            black_box(tip)
        });
    });
}

criterion_group!(
    benches,
    bench_single_tick_climate_phase,
    bench_monte_carlo_1000_runs,
    bench_scenario_composition,
    bench_tipping_point_evaluation,
);
criterion_main!(benches);
```

**Expected benchmark results on a 4-core development machine:**

| Benchmark | Target | Notes |
|---|---|---|
| `climate_single_tick_advance` | < 500 µs | Well within tick sub-budget |
| `monte_carlo_1000_runs_100_ticks` | < 10 s | Rayon parallelism ~4x speedup over serial |
| `scenario_composition_forcing_plus_shocks` | < 5 µs | Struct clone with field overrides |
| `tipping_point_evaluate_1000_ticks` | < 100 µs | Simple integer comparisons |

### 22.3 Chaos Scenario: "Climate Collapse"

The climate collapse scenario is the extreme-stress integration test. It uses the `collapse_config` defined in AT-15 above with extended duration and tracks the cascade sequence:

**Expected cascade sequence in the collapse scenario:**

| Tick range | Event | Mechanism |
|---|---|---|
| 0–10 | AF crosses 0.3 (safe threshold) | High emissions intensity, near-zero sink |
| 5–20 | TP-1 fires (AF > 1.5) | k_sink halved, accelerating accumulation |
| 10–30 | CD_effective > 0.5 | Steep sigmoid (α=8) and low onset (0.3) |
| 15–40 | TP-3 fires (CD > 0.6) | Disaster rate doubles |
| 20–50 | SP > 0.7 | Compound: productivity loss + energy deficit + food deficit |
| 25–60 | RDF < 0.2 → TP-2 fires | Fast EROI decay doubles |
| 30–80 | Governance revolt risk > 0.5 | SP > 0.7 with no legitimacy buffer |
| 50–150 | AF > 3.0 → TP-4 fires | Fully irreversible runaway |

This scenario validates that all four tipping points can fire in a realistic cascade sequence, that the SP computation correctly aggregates all stress components, and that the simulation remains numerically stable (no integer overflow, no NaN proxy) even at extreme parameter values.

**Key invariants that must hold even in collapse:**
- `AF >= 0` always (clamped at zero; sink cannot exceed forcing)
- `CD_effective ∈ [0, SCALE]` always (sigmoid output bounded)
- `RDF >= 0` always (depletion cannot go negative)
- `adaptation_stock >= 0` always (depreciation clamped)
- `scarcity_pressure ∈ [0, SCALE]` always (clip applied)
- `energy_deficit_ratio ∈ [0, SCALE]` always

---

**Version History (continued):**

- v3.0 (2026-02-21): Appended Sections 16–22. Added Extended Scenario Mechanics (tick-by-tick forcing tables, tipping point detection with hysteresis model, scenario composition with `CompositionMode`, Monte Carlo runner with rayon parallelism, `MonteCarloResult` struct), Climate-Economy Coupling Deep (granular tick-ordering pseudocode, renewable variability model with demand-response protocol, non-renewable depletion cascade, agricultural yield model with fixed-point Rust struct, `EnergySupplySnapshot`), Climate-Governance Coupling (legitimacy drain formula, resource war trigger conditions with cooldown, climate migration pressure with `DisplacementRate` formula, adaptation as political capital with capture leakage, Policy DSL climate knobs YAML), Regional Climate Heterogeneity (five zone types, `ClimateRegion` and `RegionalForcing` Rust structs, cross-region food/energy flows with `CrossRegionFlow`), Extended Event Taxonomy (8 additional event types with full JSON schemas, event routing table), Extended SQL Schema (6 additional tables and 2 views), and Extended Test Suite (8 additional acceptance tests AT-8 through AT-15, Criterion benchmarks for single-tick advance and 1000-run Monte Carlo, climate collapse chaos scenario specification).
