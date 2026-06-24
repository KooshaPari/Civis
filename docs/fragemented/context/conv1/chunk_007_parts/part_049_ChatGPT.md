### **ChatGPT**

Good.
This is the layer that makes the entire system explainable instead of opaque.
Without this, you get:

“Why did this collapse?”

“Why did tyranny spike?”

“Why did sanctions fail?”

“Why did this hegemon fall?”

With this, you get:

Causal graphs

Reproducible attribution

Clear player feedback

Publishable decomposition

CIVLAB EVENT TAXONOMY + CAUSAL ATTRIBUTION PROTOCOL
(MVE – Explainability & Research Traceability Spec v1.0)

I. Purpose

Every meaningful state change must be attributable.

Every macro shift must be decomposable into contributing factors.

Every collapse must have a traceable cause chain.

Every player-visible event must map to logged state deltas.

This supports:

Research reproducibility

Player transparency

AI policy debugging

Attractor analysis

II. Event Model Structure
Every event follows this schema:

\`\`\`
CodeEvent {  event\_id  timestamp  phase\_id  region\_id  city\_id (optional)  micro\_scope (optional)  event\_type  trigger\_conditions  primary\_state\_deltas  secondary\_state\_deltas  attribution\_weights  parent\_event\_ids}
\`\`\`

III. Event Categories (Taxonomy)
Events are grouped into top-level domains.

1) Resource & Energy Events

EnergyCapacityChanged

EnergyDeficitTriggered

FossilDepletionThreshold

RenewableMilestone

BlackoutLocal

SupplyChainDisruption

ExtractionSpike

These feed into:

Scarcity pressure

Production drops

War logistics shifts

2) Climate & Disaster Events

DisasterEvent

DamageEscalation

AdaptationThresholdReached

ClimateTippingSignal

These feed into:

Capital destruction

Health decline

Migration surge

Scarcity spike

3) Economic Events

SectorProductionShift

RentExtractionSpike

WageCompression

HousingCrisisLocal

CommodityPriceShock

These feed into:

Inequality

Discretionary decline

Protest hazard

4) Finance Events

CreditExpansionWave

AssetBubbleDetected

DefaultWave

CreditCrunch

CapitalFlight

These feed into:

Unemployment proxy

Inequality

Governance strain

5) Governance & Institutional Events

ReformEvent

CaptureEvent

CorruptionLeakIncrease

EmergencyPowersInvoked

OversightStrengthened

These feed into:

Governance quality

Enforcement intensity

Tyranny risk

6) Ideology & Social Events

RadicalizationShift

PolarizationIncrease

ProtestRiskHigh

Riot

Suppression

CulturalShift

GenerationalImprint

These feed into:

Legitimacy

Stability risk

War appetite

7) War & Diplomacy Events

CrisisEscalated

WarDeclared

Ceasefire

TreatySigned

SanctionsImposed

CoalitionChanged

MajorBattleOutcome

CorridorSeized

These feed into:

Logistics disruption

Scarcity

Legitimacy

Hegemonic shifts

8) Shadow & Black Market Events

ShadowFlowSpike

ExposureScandal

InfluenceNetworkShift

CovertOperation

EnforcementLeakageDetected

These feed into:

Corruption

Sanction effectiveness

Legitimacy shifts

9) Demographic Events

FertilityCollapseSignal

MigrationWave

AgingThresholdCrossed

FiscalStrainEscalation

These feed into:

Labor supply

Budget stress

Political stability

10) Tech & Regime Events

GPTAdoptionMilestone

TechRegimeShift

AutomationShock

SurveillanceCapabilityIncreased

OrbitalCapabilityUnlocked

These feed into:

Productivity

Inequality

Tyranny temptation

11) Hegemonic Cycle Events

ParityThresholdReached

OrderInstabilityRising

SystemWarTriggered

HegemonChanged

GlobalOrderReset

These feed into:

Trade norms

Sanction patterns

War frequency

IV. Causal Attribution Framework
Every macro variable update must store a contribution vector.
Example:
ΔTr,t=w1ΔSr,t+w2ΔΣr,t+w3ΔIr,t+w4ΔCoupr,t+w5ΔWarStater,t\\Delta T\_{r,t} =
w\_1 \\Delta S\_{r,t}
+ w\_2 \\Delta \\Sigma\_{r,t}
+ w\_3 \\Delta I\_{r,t}
+ w\_4 \\Delta Coup\_{r,t}
+ w\_5 \\Delta WarState\_{r,t}ΔTr,t​=w1​ΔSr,t​+w2​ΔΣr,t​+w3​ΔIr,t​+w4​ΔCoupr,t​+w5​ΔWarStater,t​
When tyranny increases:

\`\`\`
CodeTyrannyIncreaseEvent {  attribution\_weights = {    scarcity: 0.41,    enforcement: 0.28,    inequality: 0.19,    war: 0.12  }}
\`\`\`

These weights must sum to 1 for explainability.

V. Parent-Child Event Graph
Events can trigger cascades.
Example chain:
DisasterEvent
→ EnergyDeficitTriggered
→ SectorProductionShift
→ WageCompression
→ ProtestRiskHigh
→ Riot
→ EmergencyPowersInvoked
→ TyrannyIncrease
Each event stores parent\_event\_ids.
This creates a directed acyclic event graph.

VI. Event Severity & Persistence
Each event has:

Severity (magnitude of delta)

Duration (one tick vs multi-tick)

Persistence tag (temporary vs structural)

Structural events modify baselines:

TechRegimeShift

CaptureEvent

FertilityCollapseSignal

Temporary events decay.

VII. Causal Decomposition Outputs
At any time, you must be able to query:

What caused current Scarcity?

What caused current Tyranny?

What caused current Inequality?

What caused OrderInstability?

What caused Migration spike?

Why did sanctions fail?

Why did this hegemon fall?

All answered by traversing event graph backwards with weight aggregation.

VIII. Player-Facing Explanation System
UI panel: “Why?”
Click any metric.
See:

Immediate contributors (last 10 ticks)

Structural contributors (baseline drift)

External shocks

Shadow interference

Example:
Tyranny: 0.52
Primary drivers:

Scarcity from corridor disruption (33%)

Emergency war mobilization (24%)

Enforcement expansion (19%)

Inequality spike from finance (14%)

Cultural tolerance shift (10%)

IX. Research Mode Attribution
Exportable artifacts:

Event graph JSON

Time-weighted attribution tables

Module contribution matrices

Collapse root-cause tree

Attractor classification metadata

This makes your work publishable.

X. Performance Considerations
To prevent log explosion:

Aggregate minor events into composite weekly deltas.

Store only significant deltas above threshold.

Compress event chains periodically into summarized nodes.

XI. Testing & Validation
You must verify:

Every major macro shift has at least one event root.

No metric changes without an event record.

Event graph remains acyclic.

Attribution weights sum correctly.

This ensures scientific integrity.

XII. What This Completes
You now have:
Planet → Biosphere → Civilization
Economy → Energy → Scarcity
War → Sanctions → Logistics
Finance → Bubbles → Crises
Shadow State → Capture → Leakage
Demography → Aging → Fiscal Stress
Tech Regimes → Automation → Surveillance
Hegemony → Parity → System War
All causally linked and explainable.

Now we are at the true endgame architecture stage.
There are only two meaningful directions left:

Formal mathematical stability proofs & attractor classification framework

Hard-nosed feasibility audit (what must be cut to ship in 5 years)

Which do you want?

---

