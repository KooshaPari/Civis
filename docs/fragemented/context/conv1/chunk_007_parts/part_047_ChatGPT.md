### **ChatGPT**

CIVLAB v1 PHASE SCHEDULER + INTERFACE CONTRACTS
(MVE – Deterministic Update Order, Read/Write Sets, Cadences v1.0)
This is the “clockwork contract” that makes CivLab coherent, reproducible, and debuggable. Every module reads/writes a defined subset of state, at a defined cadence, in a fixed order. No hidden side effects.

0) Core rules
R0. Determinism
Given:

World seed package (WSP)

Genesis/intervention log (if Origin Mode)

Scenario config + policy regime config

Random seed(s)

Then the run must be exactly reproducible.
R1. Fixed phase order
No module may run out of order. All effects propagate through the pipeline.
R2. Explicit read/write sets
Each phase has:

Reads: which state variables it may consult

Writes: which state variables it may mutate

Emits: event types it logs

Cadence: how often it runs

R3. Derived vs stored
Prefer storing slow-moving core state; derive secondary diagnostics in the Metrics phase. If something is needed for future dynamics, store it; if it’s purely for UI, compute it in metrics.
R4. LOD gating
City/micro phases only run for active cells (player zoom, hotspots, theaters). Everything else stays macro.

1) Time scales and clocks
CivLab runs multiple clocks:

Weekly tick: twt\_wtw​ (core macro economy, demographics increments, local city updates)

Quarterly strategic tick: tqt\_qtq​ (diplomacy/war decisions, sanctions planning, financial policy turns, climate forcing step)

Yearly tick: tyt\_yty​ (slow structural updates: tech regime hazard, cultural drift, long demographic cohort smoothing)

Rule: weekly ticks always run; quarterly/yearly ticks trigger on week boundaries (e.g., week 13, 26, 39, 52).

2) State namespaces
To avoid confusion, state is partitioned:

Static: WSP.\* (never changes)

Global dynamic: G.\*

Regional dynamic: R[r].\*

City dynamic: C[c].\* (active only)

Micro incident dynamic: I[i].\* (event-driven only)

Networks: Net.\* (formal/hidden flows, alliances, influence graphs)

Policy controls: U[r].\* (current applied policy knobs; constrained)

3) Weekly Scheduler (core loop)
Phase W0 — Tick Setup
Cadence: weekly
Reads: clock, scenario config
Writes: internal scratch buffers, active-region/city sets
Emits: none
Purpose:

Determine active cities/districts (player focus, hot zones)

Determine whether quarterly/yearly phases will run this week

Phase W1 — Planetary & Exogenous Field Sampling
Cadence: weekly
Reads: WSP.maps.\*, G.forcing, region positions/topology
Writes: R[r].hazard\_baseline, R[r].resource\_availability, R[r].seasonal\_modifiers
Emits: none
Purpose:

Apply seasonal/geo modifiers (e.g., drought season) without running full climate

Phase W2 — Energy Supply & Extraction
Cadence: weekly
Reads: R[r].energy\_capacity, R[r].energy\_investments, WSP.energy\_ceiling, R[r].resource\_fields, R[r].damage\_index
Writes: R[r].E\_cap, R[r].E\_supply\_realized, R[r].fossil\_remaining, R[r].renewable\_share
Emits: EnergyCapacityChanged, ExtractionUsed
Purpose:

Realize usable energy given infrastructure + damage

Enforce hard ceilings (no free energy)

Phase W3 — Formal Trade/Resource Flows (Network Flow Solve)
Cadence: weekly (fast solve); quarterly for big treaty changes
Reads: Net.formal\_graph, R[r].E\_demand\_est, sanctions state, war disruption, capacities
Writes: R[r].E\_imported, R[r].critical\_imports\_received, Net.flow\_snapshot\_formal
Emits: TradeFlowUpdated
Purpose:

Compute deliverable energy/inputs via max-flow under interdiction and disruption

Phase W4 — Shadow/Leakage Flows (Black Market Layer)
Cadence: weekly
Reads: Net.shadow\_graph, enforcement, corruption, scarcity, sanctions interdictions, shadow influence index
Writes: R[r].E\_shadow\_imported, R[r].shadow\_flow\_share, Net.flow\_snapshot\_shadow
Emits: ShadowFlowUpdated, LeakageSpike
Purpose:

Compute leakage capacity and shadow rerouting

Feed back into effective resource availability

Phase W5 — Sector Production & Capital Update
Cadence: weekly
Reads: R[r].capital\_stocks, R[r].labor\_effective, R[r].E\_supply\_realized + imports, tech multipliers, damage index
Writes: R[r].sector\_output, R[r].capital\_stocks (depreciation + investment), R[r].E\_used, R[r].emissions\_week
Emits: SectorProduced, CapitalUpdated
Purpose:

Produce goods/services under energy constraint and damage

Update capital depreciation and investment allocation

Phase W6 — Allocation Engine (Regime Module)
Cadence: weekly
Reads: R[r].sector\_output, policy regime config, price signals / quota rules / plan rules, baseline rights bundle, energy quota rules
Writes: R[r].essentials\_delivered, R[r].discretionary\_delivered, R[r].prices\_or\_alloc\_weights, R[r].wage\_income\_aggregate, R[r].quota\_debits\_aggregate
Emits: EssentialsAllocated, DiscretionaryAllocated, PricesUpdated
Purpose:

Clear distribution via market, plan, joule, or hybrid composition

Must not mutate production or flows—only allocation and prices/weights

Hard constraint: Coupling lock enforced here (essentials cannot be denied due to metrics).

Phase W7 — Household/Needs & Health Update (Aggregate)
Cadence: weekly
Reads: essentials delivered, housing security proxy, healthcare coverage, pathogen pressure, climate damage, war harm
Writes: R[r].health\_index, R[r].stress\_index, R[r].sustain\_success\_rate, R[r].future\_pessimism
Emits: SustainFailure, HealthShock
Purpose:

Translate consumption shortfalls into health/stress dynamics

Produce sustain success rates (feeds legitimacy and fertility)

Phase W8 — Demography Increment (Cohort Update Lite)
Cadence: weekly
Reads: R[r].fertility\_drive, health/mortality modifiers, war deaths, migration flows (from W12)
Writes: R[r].cohort\_counts (light update), R[r].dependency\_ratio (derived or stored), R[r].births\_week, R[r].deaths\_week
Emits: Births, Deaths
Purpose:

Keep cohort structure evolving without expensive annual recalcs (those happen yearly)

Phase W9 — Finance Cycle Update (Credit/Defaults)
Cadence: weekly
Reads: output, unemployment proxy (optional), interest regime, debt stock, defaults hazard, asset price indices, scarcity/war shocks
Writes: R[r].debt\_stock, R[r].default\_rate, R[r].NPL, R[r].bank\_health, R[r].credit\_creation, R[r].asset\_price\_indices, R[r].financial\_fragility
Emits: CreditCrunch, DefaultWave, AssetBubble
Purpose:

Amplify shocks and inequality through credit/asset loops

Phase W10 — Governance Drift & Enforcement Update
Cadence: weekly
Reads: legitimacy trend, corruption, shadow capture pressure, fiscal strain, scarcity, war state
Writes: R[r].governance\_quality, R[r].corruption\_leakage, R[r].admin\_overhead, R[r].enforcement\_intensity, R[r].surveillance\_intensity (bounded), R[r].coupling\_lock (cannot flip in hybrid unless scenario explicitly allows constitutional break)
Emits: ReformEvent, CaptureEvent, EmergencyPowersInvoked
Purpose:

Drift institutions based on pressures; enforce constitutional caps

Phase W11 — Ideology, Polarization, Legitimacy Update
Cadence: weekly
Reads: sustain success, inequality, tyranny drivers, war harm, corruption scandals, media/shadow ops
Writes: R[r].legitimacy, R[r].ideology\_distribution, R[r].polarization, R[r].revolt\_hazard
Emits: ProtestRiskHigh, RadicalizationShift
Purpose:

Hearts-and-minds layer (Rebel Inc style) with multi-axis ideology

Phase W12 — Migration & Internal Movement
Cadence: weekly
Reads: cross-region utility differentials (discretionary, tyranny exposure, mobility), housing slack, war zones, climate damage
Writes: R[r].migration\_in/out, updates to R[r].cohort\_counts via deferred application (or at W8 next tick)
Emits: MigrationWave
Purpose:

Population moves, causing labor/innovation and political effects

Phase W13 — City Layer Update (Active Cities Only)
Cadence: weekly, only for active set
Reads: regional prices/alloc, energy deficits, housing/rent parameters, infrastructure capacity, local policies, war/disaster impacts
Writes: C[c].zoning\_state, C[c].infrastructure\_load, C[c].rent\_gradient, C[c].service\_coverage, C[c].local\_scarcity, C[c].local\_protest\_hazard, C[c].black\_market\_intensity
Emits: BlackoutLocal, HousingCrisisLocal, LocalStrike
Purpose:

Cities: Skylines-style causality and overlays

Phase W14 — Micro Incident Resolution (Event-driven)
Cadence: only when incidents exist
Reads: incident inputs (local ideology, enforcement, shortages, shock events)
Writes: incident outcomes that aggregate back into city/region deltas
Emits: Riot, Suppression, ScandalBreak, EpidemicFlare
Purpose:

High-resolution short runs for hotspots, then reaggregate

Phase W15 — Metrics, Logging, Snapshots
Cadence: weekly
Reads: all region states, networks, active cities, incidents
Writes: metrics frames, alert flags, replay log append
Emits: MetricsFrame, Alert
Purpose:

compute waste decomposition, tyranny, Pareto metrics, attractor monitors

export snapshot to UI clients

4) Quarterly Scheduler (strategic turns)
Quarterly phases run after weekly W15 on that boundary week.
Phase Q1 — Diplomacy & Treaty Decisions
Cadence: quarterly
Reads: alliance graph, influence capital, ideology alignment, trade dependencies, hegemon order pressure
Writes: Net.alliances, treaty set, influence spending
Emits: TreatySigned, AllianceFormed, GuaranteeIssued
Phase Q2 — Sanctions Planner & Coalition Formation
Cadence: quarterly
Reads: corridor graph, dependency matrices, leakage estimates, coalition fatigue, side-payment budget
Writes: interdiction set xex\_exe​, coalition membership, side-payments, sanction parameters
Emits: SanctionsImposed, CoalitionChanged
Phase Q3 — War Posture & Mobilization Decisions
Cadence: quarterly
Reads: parity index, threat perception, logistics SSI, domestic legitimacy, scarcity, shadow provocation signals
Writes: war state transitions, mobilization level, stockpile targets, theater allocations
Emits: CrisisEscalated, WarDeclared, Ceasefire
Phase Q4 — Climate Forcing & Disaster Realization Step
Cadence: quarterly
Reads: emissions accumulation, adaptation stock, climate volatility index
Writes: global forcing FFF, regional damage DrD\_rDr​, disaster events list
Emits: DisasterEvent, DamageUpdated

5) Yearly Scheduler (slow structural updates)
Phase Y1 — Tech Regime Hazard & GPT Diffusion
Cadence: yearly
Reads: knowledge stock, R&D share, openness, sanctions isolation, cultural innovation norm
Writes: tech regime label, GPT adoption fraction, efficiency multipliers, surveillance capability ceiling
Emits: TechRegimeShift, GPTAdoptionChange
Phase Y2 — Cultural Drift & Generational Imprints
Cadence: yearly
Reads: major shocks, war trauma, prosperity, repression, migration assimilation
Writes: cultural vector, cohort imprint modifiers
Emits: CulturalShift
Phase Y3 — Demography Full Reconciliation
Cadence: yearly
Reads: weekly cohort accumulation, mortality functions, fertility drivers
Writes: normalized cohort pyramid, median age, long-run fertility trend
Emits: DemographyReconciled
Phase Y4 — Hegemonic Order & Global Stability Update
Cadence: yearly
Reads: composite power, network centrality, order costs, fragility, scarcity world index
Writes: hegemon identity, order intensity, parity index, transition stress
Emits: OrderShift, HegemonChanged, SystemWarRiskHigh

6) Module Interface Contracts (read/write summaries)
Below are the “hard boundaries” that prevent spaghetti.
6.1 Energy Module

Reads: resource maps, damage, investments

Writes: E\_cap, E\_supply\_realized, fossil\_remaining, renewable\_share

Never writes: prices, legitimacy, ideology

6.2 Flow Network Solver (Formal + Shadow)

Reads: graphs, capacities, interdictions, disruption, enforcement/corruption (shadow)

Writes: deliverable imports, flow snapshots

Never writes: production directly (production reads flows)

6.3 Economy Production

Reads: energy delivered, labor, capital, tech multipliers, damage

Writes: sector outputs, emissions, capital depreciation

Never writes: allocations or prices

6.4 Allocation Engine (Regime)

Reads: outputs, policy rules, rights guarantees, quota laws

Writes: essentials/discretionary delivered, prices/weights, wage aggregates

Must enforce: coupling lock

6.5 Finance Module

Reads: output, scarcity, interest stance, debt/asset indices

Writes: credit creation, default rates, asset indices, bank health

Never writes: production outputs

6.6 Governance Drift

Reads: legitimacy, corruption pressure, shadow capture, scarcity, war

Writes: governance quality, admin overhead, enforcement/surveillance (bounded)

Cannot change: constitutional constraints unless scenario explicitly allows constitutional failure

6.7 Ideology/Legitimacy

Reads: lived outcomes (sustain, inequality, tyranny drivers), scandals, war harm

Writes: legitimacy, ideology distribution, polarization, revolt hazard

6.8 War/Diplomacy (Strategic)

Reads: parity, corridors, logistics, domestic constraints

Writes: war state, interdictions, mobilization, treaty sets

Does not write: day-to-day economy variables directly; it modifies disruptions/interdictions/commitments which then affect flows and economy.

7) Hotspot detection (what triggers city/micro)
A region/city becomes “active” if any triggers exceed threshold:

scarcity pressure rising fast

revolt hazard high

disaster event occurred

war theater present

major scandal exposure

sudden credit crunch

This keeps compute sane and makes the sim explainable.

8) Constitutional constraints enforcement points
Hard enforcement locations:

W6 Allocation Engine: prevents essentials denial due to metrics

W10 Governance Drift: clamps surveillance intensity to ceiling

Q phases: treaty/sanction/war actions must respect constitutional bans (e.g., no “rationing tied to loyalty” if coupling locked)

If a scenario intentionally models constitutional failure, it must be explicit and logged as a regime-change event.

9) What this enables immediately
With this scheduler:

causal traces (“why did tyranny rise?”)

stable replay

modular development and testing

clean separation between research engine and game clients

ability to publish methods section as-is

Next doc (my choice)
To make this fully executable (still no code), the next planning artifact should be:
Event Taxonomy + Causal Attribution Protocol
So every major outcome can be traced back through logged events and module deltas—this is crucial for both research papers and good gameplay (“why did my city riot?”).
Say “event taxonomy” and I’ll write it.

---

