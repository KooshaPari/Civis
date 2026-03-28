# CIV-0107 Joule Economy System Spec v1

**status**: draft
**date**: 2026-02-21
**source**: ChatGPT Conversation 6996c2ff-a110-8320-bd45-e50b5181badb
**scope**: Globalist technocratic economy based entirely on joule (energy) metric as foundation for work measurement, retirement, and acquisition.

---

## Summary

The Joule Economy System is a distinct allocation regime for CivLab that replaces monetary exchange with physical energy accounting. All human work generates joules (energy output, measured or proxied). Citizens accumulate joules in a personal energy ledger. Consumption of goods/services debits joules. A retirement pool threshold determines when citizens can exit mandatory work. Acquisition (obtaining goods) is structured through embedded energy labels and quota systems rather than price signals. The regime can operate optimally (low measurement tyranny, strong baseline rights) or dystopically (high surveillance, tight quotas coupled to rights access). This spec defines the mechanics, state variables, invariants, and integration points with the broader CivLab simulation.

---

## Core Premise

The joule economy is grounded in a single principle: **all human activity has an embedded energy cost and output**. This reframes economics as a physics problem rather than a financial one:

- **Work output** is measured in joules (direct energy produced, or proxied via output value × energy intensity).
- **Consumption cost** is the embedded energy in goods/services.
- **Wealth** is a personal joule balance (accumulated lifetime work energy).
- **Money** is optional or secondary; joules are the primary exchange medium.
- **Rights** (baseline essentials) are decoupled from score/quota, or coupled (dystopian mode).

This differs radically from capitalism (price signals) and planned economies (administrative allocation) by making physical constraint explicit at every transaction level.

---

## Citizen Energy Ledger

### Structure

Each citizen `i` maintains:

```
EnergyLedger_i = {
  lifetime_work_accumulated: J &gt; 0,  // total joules earned
  current_balance: J,                 // available joules this period
  discretionary_allocation: J,        // joules available for non-essential consumption
  baseline_fulfillment: bool,         // essentials met this tick?
  quota_remaining: J,                 // joules available to spend (consumption cap)
  audit_exposure: [0,1],              // probability of quota audit this tick
  retirement_status: enum,            // ACTIVE | RETIRED | SEMI_RETIRED
}
```

### Earning Work

Citizens earn joules by working. Work output is measured by:

```
JoulesEarned_i(t) = output_i(t) * energy_intensity(work_type) + baseline_discretionary_bonus

Where:
- output_i(t): units of work performed (hours, tasks, goods produced)
- energy_intensity: joules per unit (varies by sector)
  - Food production: high
  - Care/education: medium (partly subsidized)
  - Creative/research: low (optional energy allocation)
- baseline_discretionary_bonus: fixed weekly allocation ensuring minimum survival + small surplus
```

### Baseline Provision (Rights Layer)

Essentials are **not** purchased from the personal joule balance. They are provided universally:

```
BaslineProvision_i = {
  food_energy: F_min,         // enough for nutrition
  housing_energy: H_min,      // climate-controlled shelter
  healthcare_energy: HC_min,  // access to basic medical services
  utilities_energy: U_min,    // water, sanitation, basic connectivity
}

SustainCost_i = F_min + H_min + HC_min + U_min
```

**Constitutional rule (hybrid optimistic mode)**: Baseline provision is **never** withheld based on energy quota compliance, productivity score, or behavioral metrics. It is unconditional.

In dystopian mode: Baseline is conditional on maintaining quota compliance or passing surveillance audits.

### Retirement Pool

Citizens accumulate lifetime joule credit toward retirement:

```
RetirementCredit_i = cumulative_lifetime_work_j from j=start to t

RetirementThreshold = T_retire
  (configurable per scenario; typically 30-40 years of median-output work)

RetirementEligibility_i:
  IF RetirementCredit_i >= T_retire:
    citizen may exit mandatory work
    receives retirement transfer = median_living_cost_j × years_retired
    (funded from tax/allocation pool)
    OR remains partially employed (semi-retired)
```

**Mechanics**:
- Retirement is not a one-time payment; it is a continuous pension in joules/year.
- The pension is funded via a public retirement reserve (percent of total output, like Social Security).
- Retired citizens maintain baseline rights access but cannot accumulate additional joule wealth (or accumulate slowly).
- Family/generational transfer of retirement credits: optional per scenario (varies regime to regime).

---

## Work Classification & Measurement

Work types are classified by energy output and difficulty. The system is **flexible** to support multiple measurement schemes.

### Scheme A: Direct Energy Measurement

Suitable for **energy production, food, manufacturing**:

```
JoulesEarned = actual_physical_energy_produced
Example: Solar panel installation → 500 kWh = 1.8e9 joules earned
```

### Scheme B: Embedded Energy in Output

Suitable for **all goods/services**:

```
JoulesEarned = units_produced × energy_per_unit
Example: Engineer designs bridge; bridge requires 1e12 joules to build
  → engineer receives fraction: 1e12 / engineer_fraction = allocated joules
```

### Scheme C: Labor-Hours × Sector Energy Intensity

Suitable for **generic labor when output is hard to measure**:

```
JoulesEarned = hours_worked × energy_intensity_factor[sector]

Sector factors (joules/hour baseline, varies scenario):
- Food production: 2e7 J/hr
- Manufacturing: 1e7 J/hr
- Care/education: 5e6 J/hr (subsidized; true cost higher, gap covered by tax pool)
- Creative/research: 2e6 J/hr (optional; society funds exploration pool)
- Administrative: 1e6 J/hr (overhead burden; disincentivized)
```

### Adjustment Mechanisms

- **Skills/seniority**: multiplier on base rate (1.2x–3x)
- **Hazard/difficulty**: premium (1.5x–2x)
- **Undersupply bonus**: sectors with labor shortage get +10–20% until supply recovers
- **Cooperative production**: output split fairly among contributors (no single-person capture)

### Anti-Gaming Rules

To prevent Goodhart-style collapse:

1. **Output validation**: audit that claimed joule output matches physical reality (food weight, energy meters, auditor samples).
2. **Sector intensity review**: energy factors reviewed every 5–10 years; outdated factors are adjusted.
3. **Metric rotation**: primary measurement rotates to prevent convergence on easily gamed proxies.
4. **Diversity requirement**: citizens cannot earn >70% from a single sector/employer (encourages varied contribution).

---

## Acquisition / Commodity System

### Embedded Energy Labels

Every consumable good/service carries a label:

```
GoodLabel = {
  name: string,
  embedded_energy: J,          // total joules to produce + transport
  baseline_flag: bool,         // is this a rights-guaranteed item?
  energy_per_unit: J / unit,   // for bulk goods
  carbon_footprint: CO2e,      // optional; used in some scenarios
  sector: string,              // food, housing, energy, health, education, discretionary
  durability: years,           // how long before replacement needed
}

Example:
  Loaf of bread: 2e6 J, baseline=true, 0.5 J/gram
  Vacation trip: 1e9 J, baseline=false, carbon footprint high
  Healthcare visit: 5e7 J, baseline=true, complex energy chain
```

### Consumption / Quota Debit

When a citizen consumes a good:

```
quota_remaining_i -= good.embedded_energy

If quota_remaining_i < 0:
  transaction fails (or triggers black market / barter, in future scenario)
  audit exposure increases
  stress increases
```

### Baseline Essentials vs Discretionary

**Baseline essentials** (food, housing, healthcare, utilities):
- Provided universally, not debited from personal quota
- Allocated by rights authority, not market
- Example: 2000 kcal/day food, 15 sqm heated shelter, 1 doctor visit/month

**Discretionary** (luxury, travel, entertainment, excess consumption):
- Debited from personal quota
- Subject to availability (energy supply constraint)
- Can be traded/allocated via quota markets (if enabled)

### Acquisition Mechanisms

#### Option 1: Fixed Allocation (Planned Mode)

Goods distributed by authority based on:
- Family size / needs
- Regional availability
- Rationing during scarcity

Citizens receive allocation vouchers (abstract, not stored):
```
allocation_i = {
  food: 2000 kcal,
  clothing: 1 set/season,
  entertainment: 1 event/month,
  discretionary_joules: 1e7 / month,
}
```

#### Option 2: Energy Quota Market (Joule Optimistic)

Citizens can:
- Spend personal quota on any good they choose
- Trade quota with neighbors (bounded: not > 10% per transaction, lifetime limits)
- Sale quota at discount if strapped for cash (discouraged via progressive surcharge)

**Market clearing**:
```
Price_j(t) = f(demand_j, embedded_energy_j, scarcity_pressure, supply_capacity)

For scarce goods: price spikes (rationing by quota wealth)
For abundant goods: price near energy cost (minimal markup)
```

#### Option 3: Hybrid (Hybrid Regime)

- Baseline essentials: fixed allocation + freemarket option (citizen chooses provider)
- Discretionary: pure quota market with anti-monopoly / price controls
- Trading: bounded; progressive surcharge to prevent quota hoarding

---

## Governance & Control Mechanisms

### Baseline Strength (Constitutional Anchor)

**Baseline strength B &isin; [0,1]** defines:

```
B = ratio of unconditional rights to total output

Low B (0.2):   Baseline covers only minimal survival; hunger risk high
Medium B (0.5): Baseline covers essentials + small discretionary buffer
High B (0.8):  Baseline covers essentials + moderate creative freedom
```

**Effect on tyranny**:
- High B → survival decoupled from quota compliance → low forced compliance → low measurement tyranny
- Low B → survival depends on quota earnings → high compliance pressure → measurement tyranny risk

### Measurement Intensity (Surveillance)

**Surveillance scope Σ &isin; [0,1]** controls:

```
Σ = how much of citizen energy behavior is observed

Low Σ (0.2):  Only transaction-level observation (what is bought); work not tracked
High Σ (0.8): Biometric tracking, continuous audit, work-hour monitoring, supply-chain verification
```

**Measurement waste**:
```
MeasurementWaste = 0.05 * Σ * total_output
  (5% of output → audit overhead, compliance friction, metric gaming effort)
```

### Coupling Lock (Anti-Tyranny Constitutional Rule)

**Most critical guardrail**:

```
Constitutional rule: energy_quota_compliance cannot spill into rights access.

IF coupling_enabled:
  survival_access = f(quota_balance, audit_record, behavior_score)
  tyranny_index += 0.3  (automatic penalty)
  measurement_waste *= 2  (people hide consumption, evade audits)
ELSE (coupling forbidden):
  survival_access = unconditional (baseline strength B only)
  tyranny_index unaffected by quota status
```

### Enforcement & Audit

**Audit rate A &isin; [0, 1]**: probability citizen is audited per tick.

```
Audit outcome:
  - Check quota claim vs actual consumption
  - If discrepancy > threshold: fine, warn, or escalate to enforcement
  - Corruption leakage C &isin; [0,1]: fraction of fines diverted to official pockets

Enforcement severity E &isin; [0,1]:
  - Low E: warning, education
  - High E: quota penalty, credit freeze, mobility restriction

Legitimate enforcement is necessary for system integrity.
Illegitimate enforcement (selective targeting, corruption) drives black markets.
```

### Quota Expiry & Hoarding Prevention

```
QuotaExpiry = X weeks (typically 8–26 weeks, ~2–6 months)

Rule: joules earned expire if not used
  - Prevents indefinite accumulation → wealth stratification
  - Forces circulation → prevents stagnation
  - Encourages conservation (use it or lose it) but not extraction-like

Carryover allowance: up to 20% of next-period quota can roll over
  (provides buffer without creating permanent stratification)

Alternative: "joule inflation" where unused joules lose 10% per month
  (encourages spending on investment, not hoarding)
```

### Progressive Surcharge on Quota Trading

To prevent quota-wealthy citizens from buying up others' quotas:

```
SurchargeRate = 5% + 10% * (quotas_purchased_lifetime / median_quota)

If a citizen buys >30% of total quota volume in a year:
  Surcharge jumps to 50%
  Trading cap triggered: max 10% of monthly volume per person

Effect: quota wealth still matters, but cannot concentrate indefinitely
```

---

## Failure Modes & Edge Cases

### Joule-Optimistic (Low-Tyranny) Failure Modes

#### Mode 1: "Measurement Creep"

Over time, measurement scope Σ expands via routine pressure:
- Officials claim better auditing catches cheating
- Surveillance expands to "ensure fairness"
- Coupling pressure rises during scarcity

**Prevention**:
- Constitutional cap on Σ (e.g., max 0.6 forever)
- Metric sunset clauses (every 5 yrs, re-justify surveillance scope)
- Public transparency dashboard showing Σ trend
- Automatic alerts if Σ increases while tyranny index > threshold

#### Mode 2: "Work Intensity Spiral"

Citizens pressure themselves to work more hours to accumulate quota (discretionary access). Health / stress / creativity decline.

**Prevention**:
- Generous baseline B (&gt;0.5) → no survival pressure
- Energy intensity factors capped at reasonable hourly effort
- Mandatory vacation / rest periods (non-deductible from quota)
- Psychological stress metric monitored; alerts if trending up

#### Mode 3: "Black Market Emergence"

If quota scarcity too high, underground barter / grey markets emerge. This is not necessarily bad (resilience), but can hide inequality and enable exploitation.

**Response**:
- Monitor black market size (revealed via energy imbalance audits)
- If black market > 10% of official economy: re-examine quota cap / baseline strength
- Don't criminalize entirely; allow "informal economy" with light audit

### Joule-Dystopian (Score-State) Failure Modes

#### Mode 1: "Compliance Trap"

Coupling enabled → survival depends on quota compliance. Citizens optimize for visible metrics, not welfare:
- Suppress energy-intensive activities (art, exploration, play)
- Seek low-energy work even if mismatched to skills
- Report fake compliance
- Mobility crushed (can't switch sectors without quota loss)

**Measurement**: Discretionary realization crashes. Innovation stagnates. Legitimacy drops.

#### Mode 2: "Corruption Capture"

Officials skim fines, inflate audit results. Citizens must bribe auditors to avoid quota penalties.

**Feedback**:
- Corruption leakage C grows → misallocated energy → sustain failures → further coercion
- Governance quality decays → reform pressure builds or tyranny solidifies

#### Mode 3: "Scarcity Authoritarian Drift"

Under climate/resource shock, baseline erodes. Officials couple quota to access, tighten Σ, increase enforcement.

**Trajectory**:
```
scarcity_pressure ↑ → baseline weakens → coupling enabled → surveillance ↑ → tyranny ↑ → legitimacy ↓
```

### Edge Case: Generational Wealth Accumulators

Citizens who have high energy-intensity skills (STEM, leadership) accumulate faster. Without redistribution, inequality grows.

**Solutions**:
- Progressive taxation on joule earnings (top earners pay 30–50% tax)
- Or: mandatory profit-sharing in high-value sectors
- Or: time-based reset (everyone's quota resets fairly; no inheritance)
- Track intergenerational mobility; alert if Gini > threshold

---

## CIV Sim Integration Notes

### Existing CIV Model Dependencies

The joule economy plugs into the CivLab architecture at these points:

#### 1. Allocation Module Interface

In `TECHNICAL_SPEC.md` / engine phases, the joule economy is an **allocation rule**:

```rust
trait AllocationEngine {
  fn allocate(&self, world: &WorldState, control: &PolicyParams) -> AllocationResult;
}

impl AllocationEngine for JouleAllocator {
  // implements the quota debit, baseline provision, work accrual logic
}
```

Sits alongside:
- `MarketAllocator` (capitalist)
- `PlanAllocator` (communist)
- `HybridAllocator` (three-layer constitutional)

#### 2. Metrics System Integration

Extend `metric_snapshots` in `DATA_MODEL_DB_SPEC.md`:

```sql
-- New columns in metric_snapshots:
- energy_waste_ratio: waste / total_output (total includes measurement overhead)
- measurement_overhead: joules spent on auditing / compliance
- quota_utilization: average % of quotas consumed per citizen
- quota_trading_volume: joules exchanged in quota markets
- baseline_integrity: % of citizens meeting baseline essentials
- quota_hoarding_index: Gini of joule balances (inequality in energy wealth)
- measurement_creep_trajectory: trend in surveillance scope Σ
- coupling_violations: count of citizens denied baseline due to quota status
- black_market_proxy: (unsustained production - official consumption)
```

#### 3. Events (EVENT_TAXONOMY.md)

Add new event types:

```
joule.work_earned.v1
  actor_id, work_type, joules_earned, energy_intensity_applied, tick

joule.quota_debited.v1
  actor_id, good_type, embedded_energy, quota_remaining, tick

joule.audit_triggered.v1
  actor_id, audit_result (pass/fail), fine_amount, corruption_leakage, tick

joule.retirement_eligible.v1
  actor_id, lifetime_credit, threshold, pension_amount, tick

joule.baseline_denied.v1  [only in coupling-enabled mode]
  actor_id, reason (quota_failed | compliance_score), tick

joule.quota_expired.v1
  actor_id, joules_lost, carryover_amount, tick

joule.scarcity_shock.v1
  region, energy_supply_loss, quota_reduction, enforcement_pressure_delta, tick
```

#### 4. Data Model Additions

Extend entities:

```sql
-- citizen ledger table:
citizen_joule_ledger(citizen_id, tick, lifetime_work_accumulated, current_balance,
  quota_remaining, discretionary_allocation, retirement_credit, audit_exposure_score,
  baseline_met_flag, retirement_status)

-- goods/services table:
goods_joule_label(good_id, embedded_energy, baseline_flag, sector, energy_per_unit,
  carbon_footprint, durability_years)

-- work transaction table:
joule_work_earned(citizen_id, tick, work_type, output_amount, energy_intensity,
  joules_earned, sector_multiplier, difficulty_premium)

-- quota transaction table:
joule_quota_transaction(citizen_id, tick, transaction_type, good_id, joules_debited,
  quota_remaining, market_price_if_traded, counterparty_id)

-- audit log:
joule_audit_log(audit_id, citizen_id, tick, audit_type, quota_claimed,
  actual_consumption, discrepancy, fine_amount, corruption_diverted, enforcement_action)
```

### Event Taxonomy Extensions

These events feed the experiment framework and dashboards:

| Event Type | Trigger | Payload | Use in Metrics |
|-----------|---------|---------|-----------------|
| `joule.work_earned.v1` | Every work action | actor, work_type, joules | Total output, productivity growth |
| `joule.quota_debited.v1` | Every consumption | actor, good, energy_debited | Waste ratio, quota utilization |
| `joule.audit_triggered.v1` | Random or targeted | actor, audit_result, fine | Measurement waste, corruption leakage |
| `joule.retirement_eligible.v1` | Threshold crossed | actor, lifetime_credit, pension | Retirement pool stress, intergenerational transfers |
| `joule.baseline_denied.v1` | (coupling mode only) | actor, reason | Tyranny spike, legitimacy decay |
| `joule.quota_expired.v1` | End of expiry period | actor, joules_lost | Circulation rate, inflation proxy |
| `joule.scarcity_shock.v1` | Climate / resource event | region, supply_loss, enforcement_pressure | Governance drift, tipping points |

### DB Schema Implications

Key invariants to enforce:

1. **Double-entry ledger**: every joule debit has a source (work earned, trade, transfer, etc.)
2. **Quota expiry**: expired joules automatically zeroed per schedule
3. **Baseline provision**: always executed before quota debits (baseline gets priority)
4. **Retirement pension**: continuous draw from pool, pool funded from tax/allocation
5. **Audit consistency**: audit findings immutable; fines / penalties append-only

Indexes needed:
- `citizen_id, tick` for time-series per citizen
- `tick, audit_type` for audit rollup
- `good_id, embedded_energy` for label lookups
- `actor_id, retirement_status` for pension calculations

---

## Hybrid Constitution Integration

The joule economy is **only compatible** with the hybrid constitutional blueprint if:

1. **Baseline decoupling** (B &gt; 0.4): essentials unconditional
2. **Coupling forbidden**: constitutional hard-cap on Σ, explicit ban on energy_score → rights
3. **Sunset clauses**: energy measurement factors / surveillance scope reviewed 5-yearly
4. **Transparency**: public dashboards show Σ, measurement waste, enforcement statistics
5. **Anti-hoarding**: progressive surcharge + expiry prevent quota wealth stratification

If these are violated, the system drifts toward **Joule Score-State** (dystopian), where:
- Survival depends on quota compliance
- Measurement tyranny rises
- Innovation suppressed
- Legitimacy collapses

---

## Open Questions

1. **Work Valuation Under Uncertainty**: How do you fairly assign joules to creative work (art, research, volunteering) where output is inherently uncertain or delayed?
   - Solution sketches: lottery funding pools, mandatory R&D allocation, subjective expert panels (corrupting risk)

2. **Intergenerational Transfer**: Do children inherit retirement credits from parents? Does this create hereditary energy classes?
   - Scenarios: no inheritance (reset each generation); partial inheritance (50%); full inheritance (de facto aristocracy)

3. **Quota Expiry Harshness**: Does "use it or lose it" create perverse incentives (binge consumption before expiry)?
   - Tension: prevent hoarding vs prevent waste; carryover %/mechanisms vary scenario

4. **Trading Markets Under Scarcity**: When quotas are tight (climate scenarios), quota markets become lotteries for the quota-poor. Is this more fair than price rationing, or just a different form of inequality?
   - Research: compare outcomes in climate forcing scenarios across allocation mechanisms

5. **Measurement Creep Inevitability**: Is scope expansion (Σ increasing over time) mathematically inevitable once energy accounting exists?
   - Meta-question for attractor analysis: can Σ remain stable < 0.6 over 100+ years?

6. **Black Market Emergence Trigger**: Under what quota scarcity levels do grey markets predictably emerge?
   - Empirical: test in parameter sweeps; find bifurcation points

7. **Corruption-Legitimacy Feedback**: Does small corruption leakage (5–10%) remain stable, or does it spiral?
   - Model: corruption → baseline erosion → enforcement tightens → corruption incentive rises

8. **Citizen Acceptance**: Do people psychologically accept "joule-based" fairness the same way they accept monetary exchange?
   - Requires cognitive framing research (outside of sim); hypothesis: transparency helps but does not eliminate resentment

---

## Experiment Hooks

The joule economy enables these novel experiments:

### A) Optimal Baseline Strength (Pareto Sweep)

Vary B from 0.2 to 0.8. For each B:
- Run 50 Monte Carlo seeds over 50 years
- Measure: tyranny, discretionary realization, waste, innovation, legitimacy stability
- Result: Pareto frontier of B vs outcomes

**Hypothesis**: B &gt; 0.5 prevents measurement tyranny without killing innovation.

### B) Measurement Scope Creep Dynamics

Enable governance drift + measurement creep. Track Σ over 100 years.

**Question**: Does Σ stabilize below constitutional cap, or inevitably creep?

### C) Quota-Market Fairness Under Scarcity

Compare quota market vs price market vs fixed allocation under identical climate shocks.

**Metric**: intergenerational mobility, stress distribution, black market size, legitimacy trajectory

### D) Integration with Hybrid Constitution

Run Scenario 7 (target hybrid) with joule layer enabled vs disabled.

**Question**: Does joule accounting reduce waste without raising tyranny, vs baseline market/plan?

### E) Retirement Pool Sustainability

Vary retirement threshold T_retire (30yr, 40yr, 50yr) and pension multiplier (1x, 1.5x, 2x cost of living).

**Metric**: intergenerational pension adequacy, tax burden, legitimacy stability

### F) Corruption Feedback Loops

Vary corruption leakage C; add feedback where corruption erodes baseline. Track collapse risk.

**Hypothesis**: C > 15% leads to cascade toward state failure

---

## References

- **Core discussion**: Conversation 6996c2ff (lines 10–220, 3700–5000, 6000–7500)
- **Metric framework**: CIV-0100 Economy, CIVLAB Metric Specification Document
- **Constitutional guardrails**: Hybrid Constitutional Blueprint section
- **Governance drift model**: Institutional Evolution Model section
- **Climate forcing integration**: Climate & Resource Depletion Dynamics spec
- **Multi-regime comparison**: Scenario Catalog (Scenarios 5–6: Joule regimes)

---

## Next Steps

1. Formalize work valuation models for non-standard sectors (creative, care, volunteering)
2. Define joule-denominated financial instruments (bonds, credit, redistribution mechanisms) for future expansion
3. Build Joule Optimistic vs Joule Score-State scenario pair for launch
4. Integrate with climate layer: model how resource scarcity affects energy intensity factors and baseline costs
5. Run calibration sweep: fit joule intensity factors to historical energy data (if applicable)

