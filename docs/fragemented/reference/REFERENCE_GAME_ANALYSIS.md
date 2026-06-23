---
title: "CivLab Reference Game Mechanical Analysis"
date: 2026-02-21
status: ACTIVE
owner: CIV Architecture and Game Design Team
version: 1.0.0
tags: [game-design, reference, mechanics, victoria3, dwarf-fortress, ck3, factorio, openttd, terra-nil, influence]
---

# CivLab Reference Game Mechanical Analysis

**Doc ID:** CIV-REF-ANALYSIS-001
**Version:** 1.0.0
**Status:** ACTIVE
**Date:** 2026-02-21
**Owner:** CIV Architecture and Game Design Team
**Related Specs:**
- `CIVLAB_GAME_DESIGN.md` — Core design document; inspirations table; design pillars
- `CIV-0100-economy-v1.md` — Economy module; market clearing; allocation regimes
- `CIV-0106-social-ideology-health-insurgency-v1.md` — Social, ideology, health, insurgency
- `CIV-0105-war-diplomacy-shadow-v1.md` — War, diplomacy, shadow networks, espionage
- `CIV-0103-institutions-timeseries-citizen-lifecycle-v1.md` — Institutions, citizen lifecycle
- `CIV-0102-climate-followup-v1.md` — Climate and resource dynamics
- `CIV-0107-joule-economy-system-v1.md` — Joule economy system

---

## Table of Contents

1. [Document Purpose and Methodology](#1-document-purpose-and-methodology)
2. [Victoria 3 — Population System](#2-victoria-3--population-system)
3. [Victoria 3 — Market System](#3-victoria-3--market-system)
4. [Dwarf Fortress — Fortress and History Simulation](#4-dwarf-fortress--fortress-and-history-simulation)
5. [Crusader Kings 3 — AI Decision Architecture](#5-crusader-kings-3--ai-decision-architecture)
6. [Factorio — Production Graph](#6-factorio--production-graph)
7. [OpenTTD — Transport and Logistics](#7-openttd--transport-and-logistics)
8. [Terra Nil — Environmental System](#8-terra-nil--environmental-system)
9. [Influence / Offworld Trading Company Analog — Covert Operations](#9-influence--offworld-trading-company-analog--covert-operations)
10. [Cross-Reference Design Contract Index](#10-cross-reference-design-contract-index)

---

## 1. Document Purpose and Methodology

### 1.1 Purpose

This document is an engineering-grade mechanical analysis of the reference games that inspired CivLab's design. For each game, it provides:

1. **Formal mechanical analysis** of the relevant subsystem — the actual algorithm, formula, or state machine that makes the mechanic work.
2. **CivLab analog** — which CivLab spec and FR IDs implement the equivalent system.
3. **Delta table** — explicit comparison of what CivLab keeps, drops, or extends from each reference.
4. **Design contracts** — binding statements of the form "CivLab MUST implement X analogous to Y in game Z." These are implementation obligations, not aspirational descriptions.

### 1.2 Methodology

Analysis is based on:
- Community-verified mechanical documentation (Victoria 3 wiki, DF wiki, CK3 wiki, Factorio wiki)
- Source-available codebases (OpenTTD is open source; Factorio has community-reverse-engineered mechanics)
- Design document reverse engineering from known inputs and outputs

Where exact game source is not available, mechanics are described at the level of publicly observable behavior with formulas inferred from player experimentation.

### 1.3 Notation

- **GDD:** Game Design Document (the reference game's designer intent)
- **CIV-XXXX:** CivLab specification document ID
- **FR-CIV-XXX-NNN:** CivLab functional requirement ID
- **CONTRACT-XXX:** A binding design contract that CivLab MUST satisfy
- **ADOPTS:** CivLab takes this mechanic nearly unchanged
- **EXTENDS:** CivLab takes the mechanic but adds capability
- **DROPS:** CivLab deliberately does not implement this mechanic
- **REPLACES:** CivLab implements a different mechanic that serves the same design goal

### 1.4 How to Read Design Contracts

Design contracts are written as formal obligations:

> **CONTRACT-ID: CivLab MUST implement [WHAT] analogous to [WHAT IN REFERENCE GAME], satisfying [INVARIANT].**

They are binding on implementors. Deviations require an ADR (Architecture Decision Record) documenting the rationale.

---

## 2. Victoria 3 — Population System

### 2.1 Reference Mechanic Summary

Victoria 3's population system is the most sophisticated agent-population model in commercial strategy games. Rather than individual citizens, Victoria 3 groups people into **Pop groups** (referred to as "pops") defined by the intersection of:

- **Culture** (e.g., British, French, African)
- **Religion** (e.g., Protestant, Catholic, Animist)
- **Job type / Strata** (e.g., Aristocrats, Capitalists, Laborers, Farmers, Clergymen)

Each pop group is treated as a single homogeneous unit for simulation purposes: one happiness score, one income, one political movement membership. This is Victoria 3's fundamental simplification — it trades individual agency for computational tractability with millions of pop units.

### 2.2 Formal Mechanical Analysis

#### Pop Type Classification

```
pop_type = f(culture, religion, job)

job → strata:
  nobility, capitalist, clergy → upper strata (aristocrat, capitalist, priest)
  shopkeeper, engineer, officer → middle strata (petit bourgeoisie, intellectual, officer)
  laborer, farmer, miner, soldier → lower strata (laborer, farmer, miner, soldier)
  slave → bottom strata

pop.size = count of people in this (culture, religion, job) group
pop.wealth = total_income_last_year / pop.size  (per-capita wealth)
```

#### Pop Needs Hierarchy

Victoria 3 implements a tiered needs model that maps to Maslow's hierarchy:

```
NEEDS TIERS (consumed in order; higher tiers only if lower satisfied):

Tier 1 — Basic Needs (all pops, survival):
  food, clothes, shelter (basic)
  satisfaction_threshold = 0.8  (must meet 80% of basic need to not decline)

Tier 2 — Standard Needs (lower/middle strata, quality of life):
  furniture, basic medicine, services
  satisfaction_threshold = 0.5

Tier 3 — Luxury Needs (upper strata, political stability):
  luxury food, luxury clothes, arts, fine medicine
  satisfaction_threshold = 0.3

Satisfaction formula per tier t:
  need_satisfied[t] = consumed[t] / demanded[t]
  if need_satisfied[t] < satisfaction_threshold[t]:
    pop.standard_of_living -= weight[t] * (threshold - satisfaction)
```

#### Political Movement Formation

Grievance accumulation drives movement formation:

```
pop.political_strength = pop.size * pop.literacy * pop.radicalism

pop.radicalism accumulates from:
  + standard_of_living declining (primary driver)
  + ideology mismatch with government (if pop is liberal but gov is autocratic: +2/month)
  + unemployment
  - ideology match with government: -1/month
  - standard_of_living rising: -radicalism * 0.05 (decay)

movement.members += pop if:
  pop.radicalism > movement.join_threshold (typically 30-50)
  AND pop.ideology aligns with movement goal

movement.clout = sum(member_pop.political_strength)
revolution triggered when movement.clout > country.total_political_strength * revolution_threshold
```

#### Market Participation

Pops are both producers and consumers in Victoria 3's market:

```
producer behavior:
  pop produces goods based on job assignment
  good_output = pop.size * productivity[job] * building_level

consumer behavior:
  pop demands goods based on pop.size * need_weights[strata]
  pop buys from cheapest available source (market price)
  if price > max_affordable_price: pop cannot satisfy that need tier
```

### 2.3 CivLab Analog

| Victoria 3 Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Pop group (culture × religion × job) | Individual citizen with `culture`, `ideology_vector`, `job` fields | CIV-0103 citizen lifecycle; FR-CIV-ACT-001 |
| Pop.size | Each citizen is weight 1; cohort aggregation for performance | CIV-0106 social cohesion; CIV-0103 |
| Pop needs tiers (basic/standard/luxury) | `NeedsTier` enum; citizen consumption basket | CIV-0106 health/welfare; CIVLAB_GAME_DESIGN Section 2.2 |
| Pop.radicalism → movement formation | Cohesion decay → insurgency propensity score | CIV-0106 insurgency model |
| Pop.standard_of_living | `citizen.happiness` (0–100 bounded scalar) | CIV-0106; FR-CIV-ACT-004 |
| Political movement → revolution | Insurgency cell formation → civil war trigger | CIV-0106 Section 4 |
| Market participation (producer + consumer) | Citizen produces in job; citizen consumes from market clearing output | CIV-0100 production + consumption phases |

### 2.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Grouped pops (culture × religion × job) | **REPLACES** with per-citizen tracking | Per-citizen tracking enables richer emergent behavior and CivLab's research mission (individual causal chains). Computational cost justified by Rust implementation and Phase 1/4 parallelism. |
| Strata system (upper/middle/lower) | **EXTENDS** with `social_class` field + institutional membership | CivLab adds institutional allegiance as a separate axis from economic class |
| Religion as pop identity axis | **EXTENDS** with full `ideology_vector` in R^d | CivLab replaces single-religion identity with a multi-axis ideology vector capturing economic, political, and spiritual dimensions simultaneously |
| Pop income (per-capita wealth) | **ADOPTS** with Joule-backed currency extension | CivLab income is denominated in Drachma (Joule-backed) not arbitrary currency |
| Needs tier satisfaction model | **ADOPTS** with energy layer added | CivLab adds energy consumption as a needs tier: energy_access is a requirement below food |
| Political movement formation | **EXTENDS** with shadow network integration | CivLab movements can be accelerated or suppressed by espionage operations (CIV-0105) |
| Pop migration on dissatisfaction | **ADOPTS** | Citizens migrate to cities with higher happiness; migration events emitted |
| Literacy multiplier on political power | **DROPS** | CivLab uses education skill level instead; literacy is not a separate attribute |

### 2.5 CivLab Differences

**Per-citizen vs. grouped pops:** This is CivLab's most consequential departure from Victoria 3. Victoria 3 uses pop groups because simulating millions of individuals in 2016-era game engines was computationally prohibitive. CivLab, written in Rust with deterministic tick architecture and SIMD-capable population batching, can simulate 500K–5M individual citizens within the 1-second tick budget.

Per-citizen simulation enables:
- Individual causal chains ("citizen X was happy, then their spouse died, then they joined the insurgency")
- Network effects through actual social networks (not statistical approximations)
- True information propagation (a rumor spreads through specific people, not statistical populations)
- Emergent family and kinship structures

**Energy (Joules) as a needs tier:** Victoria 3 does not model energy access as a citizen need. CivLab adds energy as a prerequisite tier below food: a citizen without energy access cannot heat their home, cannot work in a powered factory, and cannot access information networks. This models the reality that modern civilization's needs are energy-contingent.

### 2.6 Design Contracts

> **CONTRACT-C3-POP-001: CivLab MUST implement a citizen need satisfaction model with at least three tiers (survival, standard, luxury), where lower tier satisfaction is computed before higher tier satisfaction, and where unmet tier-N need reduces citizen happiness proportional to the tier weight and deficit magnitude.**

> **CONTRACT-C3-POP-002: CivLab MUST implement a grievance accumulation → insurgency threshold model where citizen grievance (cohesion decay) accumulates from material stress, ideology mismatch, and coercion, and where crossing a configurable threshold triggers a stochastic mobilization event (CIV-0106 Section 4).**

> **CONTRACT-C3-POP-003: CivLab MUST implement citizen market participation as both producer (job output) and consumer (basket purchase), such that a citizen who cannot satisfy basic needs at market price experiences happiness penalty per tick.**

> **CONTRACT-C3-POP-004: CivLab MUST model per-citizen ideology as a multi-axis vector in R^d (minimum d=3: economic, political, spiritual), not as a single scalar or categorical religion field. Ideology diffusion between citizens must be computed per the CIV-0106 ideology diffusion spec.**

---

## 3. Victoria 3 — Market System

### 3.1 Reference Mechanic Summary

Victoria 3's market system is a supply/demand clearing mechanism operating over a goods taxonomy with price discovery through iterative market clearing. It is the economic backbone of the entire game: all production decisions, pop satisfaction, and political outcomes are downstream of the market price vector.

### 3.2 Formal Mechanical Analysis

#### Goods Taxonomy

Victoria 3 defines approximately 50 goods organized into categories:

```
GOODS CATEGORIES (simplified):

RGO Output (raw materials):
  grain, fish, fruit, sugar, cotton, coal, iron, oil, sulfur, rubber

Industrial Goods:
  steel, glass, tools, wood, paper, fertilizer, chemicals, engines

Consumer Goods:
  food (processed), clothes, furniture, medicine, services, luxury food,
  luxury clothes, luxury furniture, automobiles, telephones

Military Goods:
  ammunition, artillery, tanks, ships

Services:
  services (generic; produced by service-sector pops)
```

#### Supply/Demand Clearing

Victoria 3 uses an iterative solver that computes market price from supply and demand:

```
MARKET CLEARING ALGORITHM (per good g, per country/market):

1. Aggregate supply:
   supply[g] = sum over all production buildings of (output[g] per building per week)

2. Aggregate demand:
   demand[g] = sum over all pops of (
     pop.size * need_weight[pop.strata][g] * (pop.wealth / price[g])
   )
   + sum over all buildings of (input_demand[g] per production building)

3. Compute market tension:
   tension[g] = (demand[g] - supply[g]) / supply[g]   # positive = shortage, negative = surplus

4. Update price:
   price[g] *= (1 + tension[g] * price_sensitivity[g])
   price[g] = clamp(price[g], price_floor[g], price_ceiling[g])

5. Iterate steps 2-4 N times (Victoria 3 uses ~5 iterations per week)
   Convergence: |tension| < 0.01 for all goods

6. Compute clearing price = price after final iteration
   Record price_history for trend analysis
```

#### Price Discovery

The market price that emerges from clearing is the signal that drives all production and investment decisions:

```
PRICE SIGNAL EFFECTS:

If price[g] > profitability_threshold[g]:
  → new buildings investing in producing g
  → capitalists hire more laborers
  → supply increases next iteration (lagged ~4 weeks)

If price[g] < profitability_threshold[g]:
  → buildings reduce output (unprofitable)
  → capitalists release laborers
  → supply decreases next iteration

Lagged adjustment prevents immediate equilibrium:
  capital_investment_lag = 4-8 weeks (time to build new buildings)
  This creates natural boom/bust cycles
```

#### Trade Routes (Export/Import)

When multiple countries share a trade agreement, their markets are partially integrated:

```
TRADE INTEGRATION:

merged_supply[g] = supply_A[g] + supply_B[g] * trade_route_efficiency(A, B)
merged_demand[g] = demand_A[g] + demand_B[g] * trade_route_efficiency(A, B)

trade_route_efficiency = f(distance, infrastructure, tariffs, political_relations)
  default: 0.5 (50% integration)
  with free trade agreement: 0.9
  with embargo: 0.0

EXPORT: if price_A[g] < price_B[g] - transport_cost:
  country A exports g to country B
  A's supply decreases; B's supply increases
  prices equalize toward:
    clearing_price = (supply_A * price_A + supply_B * price_B) / (supply_A + supply_B)
```

### 3.3 CivLab Analog

| Victoria 3 Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Goods taxonomy (~50 goods) | 9-good taxonomy (essentials, discretionary, capital, public, energy) | CIV-0100; CIV-0107 Joule economy |
| Supply/demand clearing (iterative) | `AllocationEngine` trait + market clearing sub-phase (Phase 3c) | CIV-0100 Section 3 |
| Price discovery (clearing price) | Price vector `P: GoodCategory → i64` (fixed-point Drachma per unit) | CIV-0100 conservation equation |
| Trade routes (partial market integration) | Trade route bilateral agreements between cities | CIVLAB_GAME_DESIGN Section 3.2 |
| Price sensitivity parameter | `price_sensitivity[g]` → `allocation_engine.price_elasticity[good]` | CIV-0100 AllocationEngine trait |
| Building production consuming input goods | District output → district input production chain | CIVLAB_GAME_DESIGN Section 3.1 |
| Export/import between nations | Nation-level trade routes; Joule-denominated exchange rates | CIV-0100; CIV-0105 trade sanctions |

### 3.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| ~50-good taxonomy | **REPLACES** with 9 abstract good categories | CivLab's research goal is cross-regime comparison; excessive granularity creates confounds without adding explanatory power. 9 categories cover the essential economic structure. |
| Iterative market solver | **ADOPTS** as `AllocationEngine` trait for market regime | Iterated clearing is computationally tractable and regime-agnostic |
| Lagged capital investment | **EXTENDS** with Joule-cost for district construction | CivLab adds energy constraint to building investment: constructing a factory costs Joules, not just gold. |
| Trade route efficiency function | **EXTENDS** with Joule transport cost | Every trade route in CivLab has a Joule cost per unit of goods transported. This ties trade viability to energy availability. |
| Price floor/ceiling clamp | **ADOPTS** | Prevents price instability; configurable per regime (planned economy has fixed prices = floor = ceiling) |
| Victoria 3's ~50-good capitalist market | Market regime is one of three pluggable `AllocationEngine` implementations | CivLab supports planned economy and Joule quota economy alongside the market engine; same conservation substrate for all |
| Currency (British Pounds, Marks, etc.) | **REPLACES** with Joule-backed Drachma | Joule backing ties monetary value to physical energy, enabling the climate/energy coupling that is core to CivLab's research agenda |

### 3.5 CivLab Differences

**9-good taxonomy vs. ~50 goods:** CivLab deliberately abstracts goods into 9 categories to maintain cross-regime comparability. In a planned economy, "steel" and "iron ore" are managed as distinct quotas, but in a Joule economy, they collapse into "industrial good" measured in embedded Joules. The 9-category system is the common denominator.

**Joule-backed currency:** Victoria 3's currencies are fiat (backed by national reputation and economic output). CivLab's Drachma is backed by the energy reserve (`money_supply <= energy_reserve * 2`). This creates a physical constraint on money creation that Victoria 3 lacks. The consequence: monetary policy and energy policy are coupled — expanding the money supply requires expanding energy production.

**Joule transport cost on trade:** Victoria 3 models trade efficiency abstractly (infrastructure level, tariffs). CivLab measures trade transport cost in Joules per unit-kilometer. This means a trade route that was profitable at high energy abundance becomes unprofitable during an energy crisis — a coupling that Victoria 3's abstraction misses.

### 3.6 Design Contracts

> **CONTRACT-C3-MKT-001: CivLab MUST implement market clearing as an iterative supply/demand solver over the 9-good taxonomy, where price converges to a clearing price satisfying |tension| < epsilon after N iterations. The solver MUST be pure (no side effects during iteration) and execute within the Phase 3c time budget.**

> **CONTRACT-C3-MKT-002: CivLab MUST implement trade routes as bilateral Joule-costed good transfers between cities, where trade profitability = (price_differential - joule_transport_cost_per_unit). Routes with negative profitability are not executed (no forced trade).**

> **CONTRACT-C3-MKT-003: CivLab MUST implement the market clearing engine as one pluggable AllocationEngine implementation. The same conservation equation, ledger structure, and metric computation MUST apply to planned economy and Joule quota allocations, enabling cross-regime comparison on identical accounting substrate.**

> **CONTRACT-C3-MKT-004: CivLab MUST implement lagged supply response to price signals — a new district takes multiple ticks to come online after investment is committed. This prevents instantaneous equilibrium and enables natural boom/bust cycle emergence.**

---

## 4. Dwarf Fortress — Fortress and History Simulation

### 4.1 Reference Mechanic Summary

Dwarf Fortress (Bay 12 Games) is the reference implementation of emergent complexity from simple agent rules. Its depth comes not from scripted events but from the interaction of needs simulation, job systems, stress accumulation, and social dynamics playing out over hundreds of in-game years. Every "fun" (DF slang for catastrophe) is the result of a causal chain that started with a specific dwarf's unmet need five years earlier.

### 4.2 Formal Mechanical Analysis

#### Need Simulation

Every dwarf in Dwarf Fortress has a needs vector:

```
DWARF NEEDS (evaluated periodically, not every tick):

Physical needs:
  hunger: decreases by 1/tick; reaches critical at -1000; dwarf seeks food
  thirst: similar model, faster depletion (hydration critical faster than food)
  sleep: decreases by 1/tick; critical at -2000; dwarf enters "tired" state

Social needs (personality-modulated):
  social_interaction: introverts need 0-2 interactions/month; extroverts need 10+
  need_to_help: some dwarves get stress relief from helping others
  need_for_solitude: some dwarves stress when in crowds
  need_for_artistic_expression: satisfied by musical instruments, crafts, etc.

Creature comforts (quality of environment):
  room_quality: size and value of bedroom; affects baseline happiness
  dining_quality: quality of food + dining table quality
  aesthetic_needs: engravings, statues, gardens in dwarf's daily path

Spiritual needs:
  religion_need: satisfied by temples, prayer, worship activities
  contemplation_need: satisfied by solitude + study
```

#### Job Assignment System

Dwarf Fortress uses a pull-based job queue:

```
JOB QUEUE MODEL:

1. Jobs are created by game state (hungry dwarf → "prepare food" job; empty bin → "store item" job)
2. Jobs sit in a priority queue ordered by: urgency[job_type] + time_in_queue
3. Idle dwarves scan the job queue for eligible jobs based on:
   - dwarf.labor_categories ∩ job.required_labor (set intersection; non-empty = eligible)
   - distance_to_job (nearby jobs preferred)
   - current_mood (if dwarf in bad mood: -50% efficiency, may refuse non-essential jobs)
4. Dwarf claims job (removes from queue); executes; deposits result

LABOR CATEGORIES (each independently toggled per dwarf):
  Mining, Woodcutting, Masonry, Carpentry, Crafts, Smithing, Farming, Cooking,
  Medicine, Combat, Hauling, Cleaning, ...

HAULING is special:
  All dwarves have hauling enabled by default.
  Hauling jobs fill idle time; skilled dwarves prefer to do skilled work.
  If too many dwarves haul and not enough cook, food crisis ensues.
  Player must balance labor category toggles manually or via Manager (Dwarf Fortress v50).
```

#### Stress Model

The Dwarf Fortress stress model is the most detailed mental health simulation in mainstream gaming:

```
STRESS ACCUMULATION MODEL:

dwarf.stress &isin; [-5000, 5000]
  Negative = content; Positive = stressed; > 5000 = breakdown

STRESS SOURCES (per event, scaled by personality):

Positive (reduce stress):
  + ate_tasty_meal: -200 * food_quality_multiplier
  + slept_in_good_bedroom: -100 * room_quality
  + watched_art_performance: -150
  + had_positive_social_interaction: -100 * extraversion
  + completed_job_well: -50
  + had_child: -500 (major positive for family-oriented dwarves)

Negative (increase stress):
  + witnessed_death_of_friend: +3000 (catastrophic; may trigger immediate breakdown)
  + slept_on_floor: +500
  + hungry_for_3_days: +1000
  + working_without_rest: +200/day
  + imprisoned_unfairly: +5000 (nearly always triggers breakdown)
  + lost_possession: +100 to +500 depending on how valued the item is

PERSONALITY MODULATION:
  neuroticism * stress_event_magnitude → actual_stress_change
  high_neuroticism dwarf: 1.5x normal stress events
  low_neuroticism dwarf: 0.5x normal stress events

BREAKDOWN BEHAVIORS (triggered at stress > 5000):
  if mood_type = "melancholy":
    dwarf stops working; sits alone; refuses food; eventually dies
  if mood_type = "berserk":
    dwarf attacks nearest creature; becomes combat entity
  if mood_type = "tantrum":
    dwarf destroys nearest furniture; may injure bystanders
  if mood_type = "oblivious":
    dwarf ignores all stress; works normally but at reduced efficiency

RECOVERY:
  treated_by_doctor: -1000 stress
  time_off_work + good_food + social_interaction: -50/day if conditions met
  legendary_craftwork (strange mood): reset stress to -2000
```

#### Emergent Storytelling

The interaction of needs, jobs, and stress creates emergent narratives without scripted events:

```
EMERGENT STORY CHAIN (example):

Tick 0: War → military dwarf Urist killed in battle
Tick 1: Urist's wife Momuz witnesses death → +3000 stress
Tick 3: Momuz too stressed to work; food preparation halted
Tick 5: Fortress food supply short → other dwarves hungry
Tick 8: Hungry dwarves accumulate stress → productivity falls
Tick 12: Momuz reaches 5000 stress → "berserk" mood
Tick 13: Momuz attacks nearest dwarf → injures Kadol (bystander)
Tick 15: Kadol hospitalized → loses leg → permanent disability
Tick 20: Disabled Kadol cannot fulfill job role → labor shortage continues
...
"Fun" emerged from simple rules applied for 20 ticks.
```

### 4.3 CivLab Analog

| DF Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Hunger/thirst/sleep needs | Citizen survival needs (food, water, rest tiers); energy access as additional need | CIV-0106 health/welfare; FR-CIV-ACT-001 |
| Social needs (personality-modulated) | Citizen social cohesion score; ideology-matching social interactions | CIV-0106 cohesion model |
| Job queue (pull-based, labor categories) | Citizen job assignment by skill + happiness optimization | CIV-0103 citizen lifecycle; FR-CIV-ACT-003 |
| Stress accumulation | Cohesion decay accumulation → `cohort_stress_score`; health burden | CIV-0106 Sections 1 and 3 |
| Stress → breakdown behavior → recovery | Insurgency propensity crossing mobilization threshold → cell formation; recovery via welfare | CIV-0106 Section 4 |
| Personality modifier on stress events | Ideology vector component modulates stress sensitivity | CIV-0106 ideology diffusion |
| Emergent storytelling from simulation depth | Per-citizen causal chain tracing (research mode observer); deterministic replay | CIVLAB_GAME_DESIGN design pillars |
| Dwarf death triggering cascade | Citizen death → social network grief propagation → cohesion decay in connected citizens | CIV-0106 + CIV-0103 social graph |

### 4.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Physical needs (hunger/thirst/sleep) | **ADOPTS** as needs tier model; adds energy access | Energy access is a physical survival need in modern civilization; DF's medieval setting omits it |
| Social needs with personality vectors | **EXTENDS** with ideology_vector replacing personality traits | CivLab uses multi-axis ideology to capture the politically relevant dimension of "who is this person" rather than DF's behavioral personality system |
| Pull-based job queue | **EXTENDS** with market-driven job selection | CivLab citizens choose jobs based on (expected_income + happiness_bonus); not just labor eligibility |
| Stress model (integer accumulation) | **ADOPTS** as `cohort_stress_score`; maps to `CivLab.cohesion_decay` | CivLab uses fixed-point Q16.16 for stress accumulation; same model, just represented as cohesion decay |
| Breakdown behavior taxonomy (melancholy/berserk/tantrum/oblivious) | **REPLACES** with political/collective outcomes (insurgency, migration, collective action) | Individual "going berserk" is a one-person story. CivLab scales the equivalent to collective political action. |
| Legendary craftwork (strange mood) | **DROPS** | No analog; CivLab's tech tree and district improvements serve the "rare positive breakthrough" role |
| Doctor treatment for stress recovery | **REPLACES** with welfare floor policy | CivLab's health/welfare intervention model is the systemic equivalent of DF's individual medical treatment |
| Personality-modulated stress sensitivity | **EXTENDS** with ideology × culture × class interaction | CivLab models stress sensitivity as a function of multiple overlapping identity dimensions |

### 4.5 CivLab Differences

**Individual vs. emergent collective:** Dwarf Fortress's stress system culminates in individual breakdowns. CivLab's equivalent stress accumulation culminates in collective political action: cells forming, insurgencies beginning, migration waves, religious revival. This reflects CivLab's focus on civilization-level phenomena rather than individual narratives.

**No crafting system:** Dwarf Fortress has a deep crafting/production system based on individual dwarf skill and workshop chains. CivLab abstracts this to district-level production chains. The individual citizen contributes labor to a district's production output, but CivLab does not track which specific citizen produced which specific artifact.

**Determinism guarantees:** Dwarf Fortress is famously difficult to reproduce. CivLab's Tier-1 determinism requirement (fixed-point arithmetic, seeded ChaCha20Rng, BTreeMap ordering) means every simulation is exactly reproducible — the opposite of DF's notorious emergence from system complexity.

### 4.6 Design Contracts

> **CONTRACT-DF-001: CivLab MUST implement a citizen stress accumulation model where stress increases from: material deprivation (unmet needs tiers), social trauma (death events in social network), coercion (enforcement_intensity), and ideology mismatch. Stress MUST decay via welfare delivery and positive social interaction. The accumulation-decay balance must be tunable via welfare floor and surge capacity parameters.**

> **CONTRACT-DF-002: CivLab MUST implement a mobilization threshold such that when a cohort's accumulated stress score (cohort_stress_score) crosses a configurable threshold, a stochastic mobilization event fires, with probability sampled from ChaCha20Rng. This corresponds to the DF "stress > 5000 → breakdown" model at the cohort scale.**

> **CONTRACT-DF-003: CivLab MUST implement social network grief propagation: when a citizen death event fires, stress increments are applied to all citizens within social_distance <= 2 of the deceased, scaled by relationship strength. This captures the DF "witnessed death of friend: +3000 stress" mechanic at network scale.**

> **CONTRACT-DF-004: CivLab MUST implement citizen job assignment as a pull model: idle citizens evaluate available jobs, score each by (expected_income + skill_match_bonus + happiness_modifier), and claim the highest-scoring available job. Job eligibility is gated by skill category match. This is the CivLab analog of DF's labor category toggle system.**

---

## 5. Crusader Kings 3 — AI Decision Architecture

### 5.1 Reference Mechanic Summary

Crusader Kings 3 (Paradox Interactive) is the canonical reference for character-driven AI decision-making in strategy games. Its AI is not a pathfinding algorithm or minimax tree search — it is a personality-modulated utility function that evaluates schemes and interactions based on the character's traits, goals, and circumstances. The key insight: AI behavior is emergent from trait + situation evaluation, not from scripted behavior trees.

### 5.2 Formal Mechanical Analysis

#### AI "Schemes" (Multi-Tick Operations)

CK3's "schemes" are covert multi-tick operations with variable outcomes:

```
SCHEME MODEL:

scheme = {
  type: seduce | murder | fabricate_hook | sway | claim_throne | ...
  initiator: character_id
  target: character_id
  progress: int [0, 100]  (accumulates per tick toward success threshold)
  success_threshold: int [typically 80-100]
  duration_remaining: int [ticks]
  secrecy: int [0-100]  (probability target does not discover scheme)
  agents: list[character_id]  (co-conspirators, increase progress/tick)
}

PER-TICK SCHEME PROGRESS:
  progress_gain = base_progress_per_tick[scheme_type]
                + sum(agent.scheme_power for agent in scheme.agents)
                - target.scheme_resistance (intrigue skill + spymaster skill)

  if progress >= success_threshold:
    scheme.outcome = SUCCESS
    execute_scheme_effect(scheme)

  if scheme_discovered():
    scheme.outcome = DISCOVERED
    execute_discovery_consequences(scheme)

  scheme_discovered_roll:
    discovery_chance = (1.0 - secrecy/100) * scheme_type_detection_base
    rolled_discovered = uniform(0, 1) < discovery_chance  [per tick]
```

#### Personality Traits → Behavior Modifiers

CK3 uses "personality traits" as multipliers on all decision utility calculations:

```
CHARACTER TRAITS (examples):
  brave: +30 martial skill; more likely to initiate wars; less likely to sue for peace
  craven: -30 martial; less likely to initiate wars; more likely to accept unfavorable peace
  greedy: +2 gold/month; more likely to demand concessions in negotiations
  generous: +2 opinion from most characters; less likely to demand concessions
  wrathful: likely to retaliate for slights; long memory of insults
  patient: less likely to initiate; waits for better opportunity

UTILITY FUNCTION STRUCTURE:
  evaluate_action(action, character):
    base_utility = action.base_weight
    + sum(trait_modifier[trait] for trait in character.traits if trait relevant to action)
    + situational_modifiers(action, character.current_situation)
    + goal_alignment(action, character.current_goals)

    return base_utility  [>0 = favorable; <0 = unfavorable]

  AI picks action with highest positive utility across all available options.
  Minimum threshold: utility must exceed 10 to be considered.
```

#### CK3 Stress System (Character)

CK3 has its own stress model for characters (not to be confused with the DF stress model):

```
CK3 CHARACTER STRESS:

character.stress &isin; [0, 3]  (levels: Serenity, Calm, Uneasy, Stressed, Crisis)

Stress increases from:
  + acting against personality traits (e.g., generous character forced to be cruel: +1 stress)
  + traumatic events (major war losses, family deaths: +1-2 stress)
  + holding a title above stress_threshold (too many realms to manage)

Stress decreases from:
  + indulging personality traits (e.g., scholarly character reading: -0.5 stress)
  + meditation lifestyle choice (-0.2/month)
  + physician treatment (if stressed or crisis level)

At Stress Level 3 (Crisis):
  character gains random negative trait (depression, haunted, etc.)
  character may abdicate if stress remains unresolved
  character may die (heart attack, suicide) at high stress + negative traits
```

#### Council and Vassal Relationship Management

```
VASSAL MANAGEMENT:

vassal.opinion_of_liege &isin; [-100, +100]
  opinion > 25: vassal is "content"
  opinion < -25: vassal is "disloyal"
  opinion < -50: vassal will join factions against liege
  opinion < -75: vassal will join independence wars

opinion CHANGES from:
  + gifts received: +opinion_per_gold_gift
  + being given titles: +20 to +40 depending on title tier
  - being revoked of titles: -50 to -80
  - liege acting against realm interests: -5 to -20
  + successful wars fought together: +10 to +30

COUNCIL POSITIONS (6 seats):
  chancellor: diplomacy skill; affects foreign policy options
  marshal: martial skill; affects army quality
  steward: stewardship; affects tax income
  spymaster: intrigue; affects scheme detection and execution
  court chaplain: learning; affects religious relations
  [player choose who fills each role; councillors use their skill on realm]

COUNCIL MECHANICS:
  councillors with low opinion of liege: perform duties poorly (malus)
  councillors with high opinion: perform well (+bonus to their domain)
  faction formation: vassals with shared grievance form factions
  faction demands: liege must accept or face war
```

### 5.3 CivLab Analog

| CK3 Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| AI schemes (multi-tick covert operations) | Covert operations in CIV-0105 shadow networks; multi-tick execution with success/discovery probability | CIV-0105; CIVLAB_GAME_DESIGN Section 4.4 |
| Personality traits → behavior modifiers | Nation personality archetypes + ideology vector; `behavior_modifier` coefficients | CIV-0400 AI spec (referenced); CIVLAB_GAME_DESIGN |
| Stress from acting against personality | Institutional ideology mismatch → governance_legitimacy decay | CIV-0103 institutions; CIV-0106 ideology diffusion |
| Character stress → breakdown → abdication | Government stability → institutional collapse → leader succession | CIV-0103 institution lifecycle; CIV-0106 |
| Vassal opinion management | Nation trust level (bilateral 0–100); CIV-0105 diplomacy | CIV-0105; CIVLAB_GAME_DESIGN Section 4.3 |
| Council positions with skill → outcome | Government institution officials with skill level → policy efficiency | CIV-0103 institution state; governance spec |
| Faction formation from shared grievance | Insurgency faction formation in CIV-0106; alliance formation in CIV-0105 | CIV-0106; CIV-0105 |
| Trait-modulated utility function | AI nation utility function with personality archetype modifier | CIV-0400 AI spec |

### 5.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Character-level personality traits | **REPLACES** with nation personality archetypes | CivLab's primary AI actors are nations, not individual characters. Nation archetypes (expansionist, isolationist, mercantile, militarist) serve the same behavior-modulation role. |
| Multi-tick schemes | **ADOPTS** as covert operations in CIV-0105 | Covert operations have the same structure: initiator, target, progress accumulation per tick, success/discovery outcome |
| CK3 character stress model | **EXTENDS** to institutional legitimacy | CivLab's institutions can suffer "legitimacy decay" from governance actions that contradict their founding ideology — the institutional analog of CK3's character stress |
| Council skill → outcome quality | **EXTENDS** with institution state model | CivLab tracks institution capacity and capability; institutions staffed by skilled governors perform better |
| Vassal opinion → faction → demands → war | **EXTENDS** to nation trust → alliance → ultimatum → war | CivLab's diplomacy follows the same causal chain at the nation level |
| Dynasty mechanics (inheritance, succession) | **DROPS** | CivLab has "institutional memory" that persists across individual leaders; no dynasty mechanics |
| Religious conversion as control tool | **REPLACES** with ideology diffusion and propaganda operations | CivLab's ideology diffusion and shadow network propaganda serves the same control function as CK3 religious conversion |
| Stress Level 3 → negative trait acquisition | **REPLACES** with institution collapse event | At sufficiently high institutional stress, institutions collapse rather than individual characters acquiring negative traits |

### 5.5 CivLab Differences

**Institutional memory vs. dynasty:** CK3's game is fundamentally about dynasties — the continuity of bloodlines through succession crises. CivLab deliberately removes this: institutions (not bloodlines) carry memory across leadership transitions. A government's ideology, its accumulated policies, and its international reputation persist through leadership change. This models modern states better than medieval ones.

**Nation as AI actor vs. character:** CK3's AI evaluates every decision from a specific character's perspective, including their personal ambitions (becoming emperor, murdering a rival, seducing a spouse). CivLab's AI actors are nations — collective entities without personal ambitions but with institutional ones (territorial expansion, trade dominance, ideological spread).

**Covert operations at institutional scale:** CK3's schemes are personal intrigue between characters. CivLab's covert operations are institutional intelligence operations — they involve multiple agents over multiple ticks and target nation-level assets (tech trees, production, leadership stability) rather than individual character relationships.

### 5.6 Design Contracts

> **CONTRACT-CK3-AI-001: CivLab's AI nation decision architecture MUST implement a utility function of the form `evaluate_action(action, nation) = base_weight + sum(personality_modifier[archetype] * trait_weight for relevant traits) + situational_modifiers`. The personality archetype MUST modulate the base utility of all strategic actions (war declaration, trade agreement, alliance formation, covert operation initiation). An AI nation with "expansionist" archetype MUST have systematically higher utility for territorial acquisition actions than a "mercantile" archetype nation.**

> **CONTRACT-CK3-AI-002: CivLab MUST implement covert operations as multi-tick schemes equivalent to CK3's scheme model: each operation has a progress accumulation function (influenced by attacker skill and target security), a success threshold, and a per-tick discovery probability. Operations MUST not resolve in a single tick.**

> **CONTRACT-CK3-AI-003: CivLab MUST implement a bilateral trust model between nations (0–100 scalar) where trust changes from: trade conducted (+), shared enemies (+), alliance membership (+), war (-), broken treaties (-50), espionage discovery (-30). Trust MUST drive faction formation and alliance eligibility.**

> **CONTRACT-CK3-AI-004: CivLab MUST implement governance legitimacy decay when institutional policies systematically contradict the ideology vector of the governed population. This is the institutional analog of CK3's "acting against personality traits → stress accumulation" mechanic.**

---

## 6. Factorio — Production Graph

### 6.1 Reference Mechanic Summary

Factorio (Wube Software) is the definitive reference for production graph optimization as gameplay. Its core mechanic is designing production chains (recipe graphs) that balance throughput across machines, handle bottlenecks, and maintain power generation/consumption balance. Every Factorio factory is, at its core, a directed acyclic graph (DAG) of recipes where the player's goal is to maximize throughput while respecting capacity constraints.

### 6.2 Formal Mechanical Analysis

#### Recipe Graph

Factorio's production is defined by a recipe graph:

```
RECIPE DEFINITION:
  recipe = {
    inputs: {item_type: quantity, ...}  (consumed per execution)
    outputs: {item_type: quantity, ...}  (produced per execution)
    energy_cost: float  (Joules consumed per execution)
    time: float  (seconds per execution)
  }

EXAMPLE RECIPE CHAIN (iron → steel → engine unit):
  iron_plate:
    inputs: {iron_ore: 1}
    outputs: {iron_plate: 1}
    time: 3.2s
    energy_cost: 90 kJ

  steel_plate:
    inputs: {iron_plate: 5}
    outputs: {steel_plate: 1}
    time: 17.5s
    energy_cost: 90 kJ

  engine_unit:
    inputs: {steel_plate: 1, iron_gear_wheel: 1, iron_pipe: 2}
    outputs: {engine_unit: 1}
    time: 10s
    energy_cost: 150 kJ

Recipe graph:
  iron_ore → [smelter] → iron_plate → [smelter] → steel_plate → [assembler] → engine_unit
                                    ↘ [assembler] → iron_gear_wheel ↗
                                    ↘ [assembler] → iron_pipe ↗
```

#### Throughput Analysis and Bottleneck Detection

```
THROUGHPUT MODEL:

machine_throughput = recipe.outputs / recipe.time  (per machine, per second)

For a target output rate T (items/second) of final product P:
  required_machines[P] = T / machine_throughput[P.recipe]
  required_input_rate[input] = T * recipe[P].inputs[input] / recipe[P].outputs[P]

Propagate requirements backward through recipe graph:
  for each precursor recipe R of input i:
    required_machines[R] = required_input_rate[i] / machine_throughput[R]

BOTTLENECK IDENTIFICATION:
  actual_output_rate[machine_type] = min(
    machine_throughput * machine_count,
    input_delivery_rate
  )

  if input_delivery_rate < machine_throughput * machine_count:
    bottleneck_input = the starved input material
    bottleneck_stage = the upstream stage not supplying fast enough

  Balanced factory: all stages operating at 100% utilization.
  Unbalanced factory: some stages idle (input-starved) or overproducing.
```

#### Logistic Network (Belt, Inserter, Train)

```
TRANSPORT MECHANICS:

Belt throughput:
  yellow_belt: 15 items/second/lane
  red_belt: 30 items/second/lane
  blue_belt: 45 items/second/lane

Belt capacity constraint:
  if belt_throughput < required_input_rate: belt is the bottleneck
  solution: upgrade belt tier or add parallel belt lanes

Inserter rate: 0.83 to 4 items/second depending on type
  inserter capacity is often the practical limit for high-throughput machines

Train throughput:
  train_load_time = 40 seconds (1 cargo wagon, 40 items/stack)
  items_per_trip = 40 * items_per_stack
  round_trip_time = 2 * distance / speed + load_unload_time
  throughput_per_train = items_per_trip / round_trip_time

NETWORK CAPACITY:
  A train network can be analyzed as a queuing system:
  utilization = demand_rate / (train_count * throughput_per_train)
  if utilization > 0.8: add trains or optimize routes
```

#### Power Production / Consumption Balance

```
POWER GRID MODEL:

supply_watts = sum(generator_capacity for generator in online_generators)
demand_watts = sum(machine_energy_cost / operating_time for machine in active_machines)
net_balance = supply_watts - demand_watts

if net_balance < 0 (power shortage):
  machines receive partial power: each machine gets (supply / demand) fraction
  machine_speed = base_speed * (supply / demand)  [linear derating]
  production output decreases proportionally

if net_balance > 0 (power surplus):
  accumulators charge (buffer capacity)
  excess over accumulator_capacity is wasted

STEAM POWER MODEL:
  boiler: consumes fuel (coal = 8 MJ/item); produces steam
  steam_engine: converts steam to electricity (900 kW each)
  fuel_consumption_rate = demand_watts / boiler_efficiency / fuel_energy_density

SOLAR POWER MODEL:
  solar_panel: produces 60 kW during day, 0 during night
  accumulator: stores 5 MJ; buffers day/night cycle
  solar_ratio_for_night_coverage: 0.84 panels per accumulator (well-known Factorio formula)
```

### 6.3 CivLab Analog

| Factorio Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Recipe graph (item DAG) | District production chain (district output → downstream district input) | CIVLAB_GAME_DESIGN Section 3.1; CIV-0100 production sub-phase |
| Throughput bottleneck analysis | Production capacity constraint; district output capped by input availability | CIV-0100 conservation equation; supply stress metric |
| Machine saturation = balanced factory | District utilization rate; labor saturation | CIV-0100 metrics |
| Belt/train as transport | Trade routes between cities; transport Joule cost | CIV-0100; CIV-0105 supply lines |
| Power production/consumption balance | Joule grid: total_produced vs. total_consumed per tick | CIVLAB_GAME_DESIGN Section 2.2 Phase 7 (Energy Accounting) |
| Power shortage → machine derating | Energy shortage → production efficiency penalty (-10% per 1M Joules short) | CIVLAB_GAME_DESIGN Phase 7; CIV-0102 climate |
| Accumulator buffering day/night cycle | Energy reserve management; battery/storage district output | CIV-0107 Joule economy |
| Solar panel intermittency | Renewable energy variability; demand-matching challenge | CIV-0102; CIVLAB_GAME_DESIGN Pillar 2 |

### 6.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Explicit recipe graph with precise quantities | **EXTENDS** with population-labor variable | CivLab's production function includes worker count as a variable: `output = f(workers, energy, materials, building_level)`. Factorio's machines run at fixed rates; CivLab's districts have variable staffing. |
| Belt/inserter physical logistics | **REPLACES** with trade route abstraction | Physical belt routing is gameplay friction at the individual item level. CivLab models logistics at the route/capacity level: trade route has max_throughput (constrained by transport Joule cost). |
| Machine saturation as factory balance metric | **EXTENDS** to district utilization as productivity metric | CivLab tracks district utilization as a policy-relevant metric: under-utilized districts waste investment; over-demanded districts create supply stress. |
| Power grid (exact kW balance) | **ADOPTS** as Joule grid (exact Joule balance per tick) | CivLab's energy accounting is isomorphic to Factorio's power grid: `supply - demand = net_balance`. The net_balance drives the same derating mechanic. |
| Solar intermittency / accumulator ratio | **ADOPTS** as renewable variability + energy reserve management | CivLab's renewable sources (solar, wind) have variable output per tick; energy reserves buffer variability. Players must solve the same day/night (or seasonal) coverage problem. |
| Train routing as throughput optimization | **DROPS** (no train routing puzzle) | CivLab abstracts logistics to route throughput; there is no visual train placement puzzle. The strategic tradeoff (which routes to build, which goods to prioritize) is preserved. |
| Inserter as throughput bottleneck | **DROPS** | No analog at CivLab's abstraction level. |
| Blueprint-based factory design | **DROPS** (no player factory layout) | CivLab does not have a spatial production layout. Districts are abstract regions, not tile-based factories. |

### 6.5 CivLab Differences

**Abstraction level:** Factorio operates at the individual machine/belt/inserter level. CivLab operates at the district level. A CivLab "factory district" is the abstracted equivalent of an entire Factorio factory section: it consumes inputs, produces outputs, requires workers (labor) and energy, and has a throughput capacity determined by building level and worker count.

**Human workers as variable:** Factorio's machines operate at constant speed (subject to power). CivLab's districts have variable output based on staffing level: a district with 1000 workers assigned produces proportionally more than the same district with 500 workers. This introduces the labor economics layer that Factorio lacks.

**Joule economy coupling:** Factorio separates power (electricity) from materials (items). They interact only through machine energy cost. In CivLab, the Joule is the meta-currency: every good embeds a Joule cost, every trade route has a Joule transport cost, and the money supply is backed by the energy reserve. The separation between "energy" and "materials" does not exist at the same level.

### 6.6 Design Contracts

> **CONTRACT-FAC-001: CivLab MUST model district production as a throughput-constrained function: `district.output[good] = min(capacity(workers, building_level), input_supply_rate / recipe.inputs[good]) * recipe.outputs[good]`. When any input is supply-constrained, district output falls proportionally. This is the CivLab equivalent of Factorio's input-starved machine running below capacity.**

> **CONTRACT-FAC-002: CivLab MUST implement a Joule grid balance check every tick: `net_joule_balance = total_produced - total_consumed`. If `net_joule_balance \< 0`, all production districts receive a proportional efficiency penalty: `district_efficiency = total_produced / total_consumed` (capped at 1.0). This is isomorphic to Factorio's power grid derating mechanic.**

> **CONTRACT-FAC-003: CivLab MUST implement renewable energy variability: solar and wind generation output varies per tick based on climate parameters (insolation, wind speed). Energy reserves must buffer variability. Players who over-rely on renewables without sufficient reserve storage experience production disruptions during low-generation ticks.**

> **CONTRACT-FAC-004: CivLab MUST track district utilization rate (actual output / theoretical max output). Under-utilized districts emit a reportable metric. The utilization rate is the primary signal for investment decisions: players should invest in increasing capacity of bottleneck districts and reduce over-investment in surplus districts.**

---

## 7. OpenTTD — Transport and Logistics

### 7.1 Reference Mechanic Summary

OpenTTD (open-source transport tycoon) is the canonical reference for transport network economics: route profitability as a function of cargo, distance, and time. Its freight and passenger demand generation, route capacity management, and network topology optimization make it the best reference for modeling trade route economics.

### 7.2 Formal Mechanical Analysis

#### Route Profitability

```
ROUTE INCOME FORMULA (OpenTTD):

income = cargo_payment_rate[cargo_type]
         * cargo_units
         * f(travel_time, cargo_type)

where:
  f(travel_time, cargo_type) = max(
    payment_floor[cargo_type],
    (base_payment[cargo_type] + penalty_per_day * max(0, travel_time - ideal_travel_time))
  )

CARGO TYPE PAYMENT CHARACTERISTICS:
  Passengers: high base_payment; steep penalty for slow delivery (time-sensitive)
  Mail: moderate base; moderate time penalty
  Food: moderate base; high time penalty (perishable)
  Grain: low base; low time penalty (not perishable)
  Ore/coal: low base; minimal time penalty (bulk commodity)

Example (passengers, 100 units, 200 tiles, 15 days travel vs. 10-day ideal):
  income = base_rate * 100 * (1 - penalty * (15 - 10))
         = 100 * 100 * (1 - 0.05 * 5)
         = 10,000 * 0.75
         = 7,500 units

Profit = income - operating_cost (fuel + staff + vehicle wear)
```

#### Network Capacity Constraints

```
CAPACITY MODEL:

vehicle_capacity = max_cargo_units per vehicle
vehicle_throughput = vehicle_capacity / round_trip_time

route_throughput = vehicle_throughput * vehicle_count_on_route

if route.demand > route.throughput:
  backlog forms (cargo waiting at station)
  station_rating decreases (affects future cargo generation)
  solution: add more vehicles

if route.throughput >> route.demand:
  vehicles running mostly empty
  profit per vehicle falls
  solution: reassign vehicles to busier routes or add cargo types

STATION RATINGS (0-100%):
  high_rating (>80%): generates more cargo + passengers
  low_rating (<40%): demand collapses; route becomes unprofitable
  rating_factors: transport_frequency, timeliness, max_waiting_cargo
```

#### Passenger and Freight Demand Generation

```
DEMAND GENERATION:

town_passenger_demand = town.population * passengers_per_capita
  (passengers per capita increases with town size non-linearly: larger cities have denser ridership)

industry_freight_demand = industry.production_rate
  (each industry type has base production rate; output must be collected or it piles up)

CARGO CHAIN (raw material → processing → consumer goods):
  iron_ore_mine → iron_ore → [steel_mill] → steel → [train] → engineering_supply
  farm → grain → [food_processor] → food → [truck] → town_consumption

  Breaking any link in the chain causes backed-up production at the upstream industry.
  Undelivered output reduces production rate (industry scales back if output not collected).
```

### 7.3 CivLab Analog

| OpenTTD Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Route income = f(distance, cargo type, travel time) | Trade route profitability = f(distance, good type, Joule transport cost) | CIV-0100; CIVLAB_GAME_DESIGN Section 3.2 |
| Vehicle capacity → route throughput | Trade route max_throughput (bounded by transport infrastructure) | CIV-0100 trade route model |
| Station rating → demand generation | Trade partner reputation + trade frequency → demand pull | CIV-0105 diplomacy; CIV-0100 |
| Cargo chain (raw → processed → consumer) | District production chain (farm → grain → food processor → citizens) | CIV-0100 production chain; CIVLAB_GAME_DESIGN Section 3.1 |
| Industry output backing up without collection | District surplus accumulation; market price falls if surplus unshipped | CIV-0100 conservation equation; price clearing |
| Network topology (hub-and-spoke vs. point-to-point) | City trade network topology; hub cities in CIVLAB_GAME_DESIGN Section 3.2 | CIVLAB_GAME_DESIGN |
| Passenger demand = f(city population) | Citizen labor mobility and migration drive "passenger demand" analog | CIV-0103 citizen migration; CIV-0106 |

### 7.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Route income formula (distance × cargo × time) | **EXTENDS** with Joule transport cost replacing time | CivLab's fundamental constraint is energy (Joules), not calendar time. Slow transport costs more Joules per unit; fast transport is physically more energy-intensive. |
| Vehicle types (train, truck, ship, plane) | **REPLACES** with infrastructure quality levels | CivLab abstracts vehicle types to route infrastructure quality (road, rail, port), which determines max_throughput and energy efficiency. |
| Station rating affecting demand | **ADOPTS** as trade partner reputation affecting trade volume | Consistent reliable trade builds trust (analog of high station rating); missed deliveries reduce trade volume (analog of low station rating) |
| Cargo time-sensitivity (perishable goods) | **EXTENDS** with Joule cost of refrigerated transport | Perishable goods cost more Joules per unit-distance (refrigeration) than durable goods. This maps OpenTTD's time penalty into Joule cost. |
| Visual route planning and vehicle management | **DROPS** | CivLab is not a transport tycoon; route management is strategic-level, not operational |
| Competition between transport companies | **DROPS** | CivLab has nation-level competition for trade routes, not company-level transport competition |
| Industry production halting if not collected | **ADOPTS** as district surplus driving price drops | If a district produces goods that aren't traded away, the surplus accumulates, market price falls, and production investment becomes less attractive |

### 7.5 CivLab Differences

**Joule transport cost replaces time penalty:** OpenTTD penalizes slow delivery through time-based income reduction. CivLab penalizes it through energy cost: faster transport (by air or high-speed rail equivalent) is more energy-intensive. This ties transport economics directly to the energy system, whereas OpenTTD's time penalty is an abstracted financial penalty.

**No vehicle management:** CivLab does not have individual vehicles to manage. Trade routes are capacity agreements: two cities agree to exchange goods at a rate bounded by their transport infrastructure. The "how" of transport is abstracted.

### 7.6 Design Contracts

> **CONTRACT-OTTD-001: CivLab MUST implement trade route profitability as: `profit = (price_differential[good] - joule_transport_cost[good, distance, infrastructure_quality]) * units_traded`. A route with negative profit is not executed (merchants do not voluntarily trade at a loss). This is the CivLab equivalent of OpenTTD's income minus operating cost calculation.**

> **CONTRACT-OTTD-002: CivLab MUST implement trade route throughput capacity: each trade route has a `max_throughput_per_tick` determined by infrastructure quality (road \< rail \< port). If trade demand exceeds route capacity, surplus accumulates at the producing city, driving down the local price via market clearing, and reducing production incentives. This captures OpenTTD's backlog mechanics.**

> **CONTRACT-OTTD-003: CivLab MUST implement good-type-specific transport cost modifiers: perishable goods (food, medicine) have higher Joule cost per unit-distance than durable goods (metals, tools). This captures OpenTTD's cargo type time-sensitivity in energy terms.**

---

## 8. Terra Nil — Environmental System

### 8.1 Reference Mechanic Summary

Terra Nil (Free Lives, 2023) inverts the factory-building genre: instead of building industry, you restore an industrial wasteland to a thriving ecosystem. Its core mechanic is **biome restoration through cascading environmental conditions**: toxic soil must be remediated before plants can grow; plants must establish before insects return; water systems must be restored for fish; diverse biomes must be established for complex ecosystems. Each stage depends on the previous, and investment in early stages pays dividends in later ones.

### 8.2 Formal Mechanical Analysis

#### Biome Restoration Mechanics

```
TILE STATE MODEL:
  tile.state &isin; {wasteland, remediated, grassland, forest, wetland, tundra, coast}
  tile.pollution &isin; [0.0, 1.0]
  tile.moisture &isin; [0.0, 1.0]
  tile.biodiversity &isin; [0.0, 1.0]  (count of distinct species present / total possible)

RESTORATION PIPELINE (linear prerequisite chain):
  wasteland (pollution > 0.5)
    → [apply detoxifier] → cost: 100 units, time: 1 season
  remediated (pollution < 0.1)
    → [irrigate] → cost: 50 units + water availability
  grassland (moisture 0.3-0.6, pollution < 0.1)
    → [plant trees or manage] → cost: 75 units + existing grass
  forest (moisture 0.5-0.8, temperature range, biodiversity > 0.3)
    → [establish wetland areas] → cost: 200 units + adjacent water
  wetland
    → [complete ecosystem] → biodiversity > 0.7 → RESTORATION COMPLETE

CASCADING CONDITIONS:
  forest.presence in adjacent tiles → moisture += 0.1/season (transpiration)
  wetland.presence → adjacent_tile.pollution -= 0.05/season (biofiltration)
  biodiversity > 0.5 → self-sustaining (no further investment needed)
  biodiversity < 0.1 → decline (pollution or invasive species; loses 0.02/season)
```

#### Non-Linear Returns on Early Investment

```
RESTORATION COST CURVE:

Detoxification phase:
  cost_per_tile_pollution_unit = high (1.0 → 0.5: expensive)
  investment units: 100 per tile
  Restoration ROI: low in isolation (tile is just remediated, not productive)

Grassland phase:
  cost_per_tile: 50 units
  ROI begins: grassland generates 5 biodiversity/season (moderate)

Forest/Wetland phase:
  cost_per_tile: 75-200 units
  ROI: high and accelerating
    - forests expand moisture to adjacent tiles (reduces future irrigation cost)
    - wetlands self-propagate if moisture > 0.6
    - biodiversity > 0.5: ecosystem becomes self-sustaining (zero maintenance cost)

OBSERVATION: Early investment (detoxification) has LOW direct ROI but is the prerequisite
for high-ROI later phases. Total ROI of early investment is HIGH when viewed across the
full restoration chain. Terra Nil's design insight: front-load high-cost/low-return work
to unlock the compounding returns of later phases.

Restoration cost as function of restoration % completed:
  cost_to_reach_10%:  expensive (pure detoxification, no return)
  cost_to_reach_50%:  moderate (grassland and forest established, ROI rising)
  cost_to_reach_80%:  cheaper per % point (ecosystem partly self-sustaining)
  cost_to_reach_100%: expensive again (last 20% requires rare biome conditions)

This creates a non-linear cost curve: ∩-shape (expensive at both ends; cheap in middle)
```

#### Resource Consumption for Restoration

```
RESOURCE ECONOMY:

Every restoration action consumes:
  - construction_materials (finite; must be collected from existing structures)
  - recyclers: special machines that convert demolished structures to materials

Player constraint: you start with X materials from the existing industrial ruins.
You must use structures economically — every piece of infrastructure you build
must eventually be demolished and recycled to fund the next phase.

This creates the "bootstrapping" dynamic:
  Phase 1: build detoxifiers (cost: 50 materials) → remediate 20 tiles
  Phase 2: demolish detoxifiers (recover 40 materials) → irrigate remediated tiles
  Phase 3: demolish irrigators (recover 35 materials) → plant forests
  ...
  Final phase: demolish all remaining structures → recycled into materials for final ecosystem
```

### 8.3 CivLab Analog

| Terra Nil Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Tile pollution level → remediation pipeline | CO2 concentration → climate remediation investment pipeline | CIV-0102 climate; CIVLAB_GAME_DESIGN Section 3.1 Joule economy |
| Cascading biome restoration (soil → plants → ecosystem) | Carbon capture investment → renewable transition → climate stabilization | CIV-0102; CIVLAB_GAME_DESIGN Phase 8 climate events |
| Non-linear early investment ROI | Renewable energy infrastructure: early investment is capital-intensive but reduces long-run climate event probability | CIVLAB_GAME_DESIGN Phase 8; CIV-0102 |
| Biodiversity > threshold → self-sustaining | Carbon-neutral energy mix → climate stabilization (no further climate event spiral) | CIVLAB_GAME_DESIGN Phase 8 probability curve |
| Moisture transpiration from forests → adjacent tiles | CO2 reduction in one nation reduces global CO2 concentration (shared climate system) | CIV-0102 global carbon budget |
| Construction/demolition cycle for materials | Dirty energy investment vs. renewable transition investment | CIV-0102; Joule economy regime tradeoffs |

### 8.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Tile-based spatial restoration | **REPLACES** with aggregate CO2/climate state per nation | CivLab is a nation-level simulation; spatial biome restoration at tile level is too granular. CO2 concentration and climate state are the equivalent aggregate variables. |
| Biome diversity as restoration metric | **REPLACES** with energy mix diversification | CivLab's "biodiversity" equivalent is the energy portfolio: a nation with diverse renewable energy mix is resilient; one dependent on coal is fragile. |
| Self-sustaining ecosystem at biodiversity > 0.5 | **ADOPTS** as climate stabilization threshold | CivLab's climate model has a CO2 threshold below which climate events cease: the analog of Terra Nil's self-sustaining ecosystem. |
| Demolition-funded bootstrapping | **EXTENDS** as stranded asset problem | CivLab models stranded assets: coal infrastructure invested in before the renewable transition has economic value that is lost when decommissioned. This is the CivLab analog of Terra Nil's structural demolition. |
| Non-linear restoration cost curve | **ADOPTS** explicitly: early climate investment has higher ROI than late | CivLab MUST implement this as a design invariant (CONTRACT-TERRA-001 below) |
| Invasive species / biodiversity decline | **REPLACES** with irreversible tipping point | If CO2 > 550 ppm in CivLab, climate events become self-reinforcing (50% per tick); this is the analog of Terra Nil's ecosystem decline below critical biodiversity. |
| Single-player construction game | CivLab's climate is a multi-player collective action problem | Terra Nil's single player controls all restoration. CivLab's climate is a multi-nation coordination problem: individual nations benefit from free-riding on others' climate investment. |

### 8.5 CivLab Differences

**Multi-nation collective action problem:** Terra Nil is a single-player game where the player controls all restoration activities. CivLab's climate system is shared across all nations: every nation's CO2 emissions contribute to the global carbon budget, but the costs of climate remediation fall on whichever nation invests. This creates a public goods problem (free-rider incentive) that has no analog in Terra Nil.

**Stranded asset economics:** Terra Nil's structures are demolished to fund the next phase — there is no economic cost to demolition. In CivLab, transitioning from coal to renewable energy requires not just building renewable capacity but also writing off the value of existing coal infrastructure (stranded assets). Nations with high coal investment face a larger economic hit from the renewable transition.

**Climate as probability distribution, not deterministic:** Terra Nil's restoration progress is deterministic (apply detoxifier → pollution drops by exactly X). CivLab's climate is probabilistic: high CO2 increases the probability of climate events per tick, but does not guarantee them. This creates risk management decisions rather than optimization decisions.

### 8.6 Design Contracts

> **CONTRACT-TERRA-001: CivLab MUST implement climate remediation with non-linear returns: the marginal ROI of reducing CO2 concentration from 550ppm to 450ppm MUST be higher than the ROI of reducing from 450ppm to 350ppm, and both MUST be higher than the ROI of reducing from 600ppm to 550ppm (where climate events are already self-reinforcing). This captures Terra Nil's design insight that early restoration investment unlocks compounding returns.**

> **CONTRACT-TERRA-002: CivLab MUST implement a climate stabilization threshold: at CO2 \< 350ppm, climate event probability drops to 0. This is the analog of Terra Nil's "biodiversity > 0.5 → self-sustaining" mechanic. Nations that invest in carbon capture and renewable transition can reach this threshold and permanently exit the climate risk spiral.**

> **CONTRACT-TERRA-003: CivLab MUST model the stranded asset problem: coal and fossil fuel districts have positive economic value (they produce Joules cheaply) but contribute to CO2 accumulation. Decommissioning them before end-of-life writes off their remaining economic value. The stranded asset cost is the economic barrier to rapid renewable transition, and MUST be reflected in the player's decision calculus.**

> **CONTRACT-TERRA-004: CivLab MUST implement the collective action problem for climate: each nation's CO2 contributes to a shared global carbon budget (not nation-specific). Nations have incentive to free-ride on others' climate investment. This creates diplomatic pressure mechanics (carbon trading disputes, climate war casus belli) absent from Terra Nil's single-player design.**

---

## 9. Influence / Offworld Trading Company Analog — Covert Operations

### 9.1 Reference Mechanic Summary

"Influence" in CivLab's design document refers to the covert operations and information asymmetry mechanics found in games like Offworld Trading Company (OTC) by Mohawk Games. OTC is a real-time economic strategy game where covert sabotage, market manipulation, and information advantage are the primary competitive tools rather than military force. Its black market, sabotage detection system, and information asymmetry mechanics provide the canonical design reference for CivLab's espionage layer.

OTC's key design insight: **information asymmetry is a competitive moat**. A player who knows where the next resource shortage will occur before their opponent can pre-position to profit from it. Covert operations create and defend information asymmetry.

### 9.2 Formal Mechanical Analysis

#### Black Market Mechanics

```
BLACK MARKET MODEL (OTC):

black_market_items = {
  "hack": disable enemy building for 30 seconds; cost: 3000 currency
  "emp": disable all enemy buildings in radius for 15 seconds; cost: 8000 currency
  "patent_steal": gain opponent's patent (technology) temporarily; cost: 5000 currency
  "carbon_taxes": force opponent to pay tax on all fossil fuel production; cost: 2000 currency
  "pirates": disrupt opponent's supply lines; cost: 4000 currency
}

PURCHASE MODEL:
  black_market.items rotate every 5 minutes (new options appear)
  item_price scales with match_time (cheaper early; expensive late)
  item_quantity is finite (if you buy last EMP, no more EMPs available)

COUNTER-INTEL:
  player can buy "bribe official" (reverse one black market item targeting you)
  or buy "security" (passive: X% chance any item targeting you is negated)
```

#### Sabotage Operations with Detection Risk

```
DETECTION PROBABILITY MODEL (generalized from OTC):

detection_chance = f(operation_complexity, target_security_investment, attacker_skill)

For a single sabotage operation:
  base_detection_chance = operation_type.base_detection[operation_type]
  modified_detection = base_detection_chance
                     * (1 + target.security_investment / security_normalization)
                     * (1 - attacker.espionage_skill / skill_normalization)
  detection_roll = uniform(0, 1) < modified_detection

If detected:
  diplomatic_penalty = -relationship_damage[operation_type]
  if relationship drops below war_threshold: casus_belli_available = True
  attacker.agent_captured = True (loses agent, cannot use same agent again)

If undetected:
  operation_effect applied silently
  target does not know which nation attacked (if multiple nations plausible)
```

#### Information Asymmetry as Competitive Advantage

```
INFORMATION STATE MODEL:

Each nation has an information state:
  visible_to_self[own_production] = TRUE (always)
  visible_to_self[own_reserves] = TRUE (always)
  visible_to_self[own_prices] = TRUE (always)
  visible_to_opponent[own_production] = FALSE (unless scout/intel operation)

INTELLIGENCE GATHERING:
  spy_mission(target_nation, information_type):
    success_chance = attacker.spy_budget * f(target.counter_intel_budget)
    if success: reveal target's [production_levels / army_positions / treasury / tech_progress]
    knowledge_decay: revealed information degrades in accuracy at rate_per_tick

INFORMATION VALUE:
  Knowing price_of_food[opponent] in advance of trade round:
    → can pre-buy cheap before price spikes
    → information rents: profit = (post-spike_price - current_price) * units_bought

  Knowing army_composition[opponent]:
    → tactical advantage in war planning
    → can avoid engaging unfavorable unit matchups

PROPAGANDA (information injection):
  Inject false information into opponent's population:
    target.citizen.belief_update(rumor, source=hidden)
    spread through social network with distortion coefficient

  Defensive: detect_propaganda_campaign[own_population] costs counter-intel budget
```

### 9.3 CivLab Analog

| OTC/Influence Concept | CivLab Equivalent | Spec Reference |
|---|---|---|
| Black market operations (buy/sell off-books) | Shadow network covert operations budget | CIV-0105; CIVLAB_GAME_DESIGN Section 4.4 |
| Sabotage with detection risk | CIV-0105 technology sabotage, production disruption operations | CIV-0105 shadow networks; CIVLAB_GAME_DESIGN Operations |
| Detection probability scaling with target security | `detection_probability = f(operation_complexity * target_security_investment)` | CIVLAB_GAME_DESIGN Section 4.4 (success chance mechanics) |
| Information asymmetry as competitive moat | Intel gathering reveals hidden state (army, treasury, production, tech) | CIV-0105; CIVLAB_GAME_DESIGN Information Gathering operation |
| Propaganda / false information injection | Rumor spreading via Phase 9 (information spread); CIV-0106 ideology diffusion | CIVLAB_GAME_DESIGN Phase 9; CIV-0106 |
| Counter-intelligence (passive defense) | Counter-intelligence budget reduces detection failure rate | CIV-0105; CIVLAB_GAME_DESIGN (counter-intelligence) |
| Patent steal (temporary technology transfer) | Technology sabotage (slow enemy research); inverse of steal | CIV-0105 technological sabotage |
| Market manipulation (know price before others) | Intelligence revealing opponent's market state enables pre-positioning | CIV-0105 intelligence + CIV-0100 market |
| Agent captured if detected | Spy lost on detection; no reuse; diplomatic damage | CIVLAB_GAME_DESIGN Section 4.4 failure consequences |

### 9.4 Delta Table

| Mechanic | CivLab Treatment | Rationale |
|----------|-----------------|-----------|
| Black market item rotation (5-minute refresh) | **DROPS** | CivLab does not have a rotating shop mechanic; covert operations are available at any time but constrained by budget and agent availability |
| Finite item quantities (last EMP bought by one player) | **REPLACES** with agent capacity limit | CivLab limits covert operations by the number of available agents and the spy budget. More agents (higher budget) = more concurrent operations. |
| Real-time sabotage (30-second disable) | **EXTENDS** to multi-tick operation with duration | CivLab operations take 1-8 weeks (measured in ticks) to execute, not 30 seconds. This reflects realistic espionage timescales. |
| OTC's information as exploitable commodity | **EXTENDS** with information social network propagation | CivLab's intelligence integrates with the Phase 9 information propagation system: revealed intelligence can be used to craft propaganda that spreads through the target nation's social network. |
| Detection probability model | **ADOPTS** explicitly with formula | See CONTRACT-INF-001 below; the formula is directly adopted |
| Market manipulation via advance intel | **EXTENDS** to strategic pre-positioning | CivLab's market is slower (week-scale ticks not second-scale); intelligence about trade routes, production levels, and resource reserves enables strategic pre-positioning over multiple ticks. |
| EMP disabling all enemy buildings | **REPLACES** with infrastructure sabotage (production district disruption) | CivLab's covert operations target specific districts (e.g., "sabotage enemy power grid" → reduce energy production 30% for 5 ticks). No instantaneous area-effect disables. |
| Carbon taxes as aggressive market action | **ADOPTS** as climate policy weaponization | Nations can push climate agreements that disproportionately burden fossil-fuel-dependent opponents. This is CivLab's equivalent of OTC's carbon tax black market item — using policy as economic warfare. |

### 9.5 CivLab Differences

**Timescale (real-time vs. tick-based):** OTC operates in real-time; a sabotage mission resolves in 30 seconds. CivLab's covert operations unfold over multiple ticks (weeks), with progress accumulation, detection risk each tick, and delayed revelation of results. This matches the realistic pace of intelligence operations.

**Multi-agent persistent network:** OTC's black market is transactional (buy one item, use it). CivLab has a persistent spy network: agents are recruited, trained, deployed on specific missions, and can be captured or "burned." The network is a strategic asset that requires long-term investment and maintenance.

**Social network integration:** OTC treats information as a binary (you have intel or you don't). CivLab's intelligence integrates with the population information propagation system (CIV-0106 Phase 9): discovered intelligence can be weaponized as propaganda that spreads through the target's social network with distortion, creating a richer and more realistic information operations layer.

**Institutional espionage vs. character intrigue:** CK3's espionage targets characters (assassinate the king). CivLab's espionage targets institutions (sabotage the research bureau, corrupt the treasury ministry). This keeps CivLab's covert operations at the institutional level consistent with the rest of the simulation.

### 9.6 Design Contracts

> **CONTRACT-INF-001: CivLab MUST implement covert operation detection probability as: `detection_probability = base_detection[operation_type] * (1 + target.security_investment / k1) * (1 - attacker.espionage_skill / k2)`, where k1 and k2 are tunable constants calibrated so that a maximally-defended target against a minimally-skilled attacker has > 80% detection probability, and a minimally-defended target against a maximally-skilled attacker has \< 10% detection probability. This formula MUST apply to all operation types, with operation-type-specific base_detection values.**

> **CONTRACT-INF-002: CivLab MUST implement covert operations as multi-tick schemes: each operation has a per-tick progress accumulation function. Operations MUST NOT resolve in a single tick. This ensures operations are interruptible (if the attacker's agent is detected mid-operation, the operation fails and consequences apply).**

> **CONTRACT-INF-003: CivLab MUST implement intelligence value decay: information revealed by a successful intelligence operation degrades in accuracy at a rate per tick. Information about a rapidly-changing state (army positions, trade prices) decays faster than information about stable state (technology level, government ideology). Players cannot stockpile intelligence indefinitely; they must act on it promptly.**

> **CONTRACT-INF-004: CivLab MUST implement a propaganda injection mechanic: a successful propaganda operation injects a "rumor" into the target nation's information propagation network (CIV-0106 Phase 9 information spread). The rumor propagates through the social network with the same distortion coefficient as natural rumors. The attacker's identity is hidden (the rumor appears to originate internally). Detection of propaganda as foreign-sourced has the same relationship damage as detection of any other espionage operation.**

> **CONTRACT-INF-005: CivLab MUST implement climate policy weaponization as a covert economic operation: a nation can lobby (openly or covertly) for international climate agreements with asymmetric cost structures that burden fossil-fuel-dependent opponents more than themselves. This is the strategic equivalent of OTC's carbon tax black market item — using international institutions as instruments of economic competition.**

---

## 10. Cross-Reference Design Contract Index

The following table indexes all design contracts from this document for quick lookup by implementors.

| Contract ID | Game Reference | CivLab Component | Primary Spec | Implementation Priority |
|-------------|---------------|-----------------|-------------|------------------------|
| CONTRACT-C3-POP-001 | Victoria 3 Populations | Citizen need satisfaction model | CIV-0106, CIV-0103 | P0 |
| CONTRACT-C3-POP-002 | Victoria 3 Populations | Grievance → insurgency threshold | CIV-0106 Section 4 | P0 |
| CONTRACT-C3-POP-003 | Victoria 3 Populations | Citizen market participation | CIV-0100, CIV-0106 | P0 |
| CONTRACT-C3-POP-004 | Victoria 3 Populations | Multi-axis ideology vector | CIV-0106 ideology diffusion | P1 |
| CONTRACT-C3-MKT-001 | Victoria 3 Markets | Market clearing solver | CIV-0100 AllocationEngine | P0 |
| CONTRACT-C3-MKT-002 | Victoria 3 Markets | Trade route Joule cost | CIV-0100, CIV-0105 | P0 |
| CONTRACT-C3-MKT-003 | Victoria 3 Markets | Regime-agnostic conservation substrate | CIV-0100 | P0 |
| CONTRACT-C3-MKT-004 | Victoria 3 Markets | Lagged supply response | CIV-0100 | P1 |
| CONTRACT-DF-001 | Dwarf Fortress | Citizen stress accumulation model | CIV-0106 | P0 |
| CONTRACT-DF-002 | Dwarf Fortress | Mobilization threshold + stochastic event | CIV-0106 Section 4 | P0 |
| CONTRACT-DF-003 | Dwarf Fortress | Social network grief propagation | CIV-0106, CIV-0103 | P1 |
| CONTRACT-DF-004 | Dwarf Fortress | Pull-based job assignment | CIV-0103 | P1 |
| CONTRACT-CK3-AI-001 | Crusader Kings 3 AI | Personality-modulated utility function | CIV-0400 AI | P1 |
| CONTRACT-CK3-AI-002 | Crusader Kings 3 AI | Multi-tick covert schemes | CIV-0105 | P0 |
| CONTRACT-CK3-AI-003 | Crusader Kings 3 AI | Bilateral trust model | CIV-0105 diplomacy | P0 |
| CONTRACT-CK3-AI-004 | Crusader Kings 3 AI | Governance legitimacy decay | CIV-0103, CIV-0106 | P1 |
| CONTRACT-FAC-001 | Factorio | Throughput-constrained production | CIV-0100 | P0 |
| CONTRACT-FAC-002 | Factorio | Joule grid derating mechanic | CIV-0100, CIV-0102 | P0 |
| CONTRACT-FAC-003 | Factorio | Renewable energy variability | CIV-0102 | P1 |
| CONTRACT-FAC-004 | Factorio | District utilization rate metric | CIV-0100 metrics | P1 |
| CONTRACT-OTTD-001 | OpenTTD | Trade route profitability formula | CIV-0100 | P0 |
| CONTRACT-OTTD-002 | OpenTTD | Trade route throughput capacity | CIV-0100 | P0 |
| CONTRACT-OTTD-003 | OpenTTD | Good-type transport cost modifiers | CIV-0100 | P1 |
| CONTRACT-TERRA-001 | Terra Nil | Non-linear climate remediation ROI | CIV-0102 | P0 |
| CONTRACT-TERRA-002 | Terra Nil | Climate stabilization threshold | CIV-0102 | P0 |
| CONTRACT-TERRA-003 | Terra Nil | Stranded asset problem | CIV-0100, CIV-0102 | P1 |
| CONTRACT-TERRA-004 | Terra Nil | Collective action climate problem | CIV-0102, CIV-0105 | P1 |
| CONTRACT-INF-001 | Influence/OTC | Detection probability formula | CIV-0105 | P0 |
| CONTRACT-INF-002 | Influence/OTC | Multi-tick operation scheme | CIV-0105 | P0 |
| CONTRACT-INF-003 | Influence/OTC | Intelligence value decay | CIV-0105 | P1 |
| CONTRACT-INF-004 | Influence/OTC | Propaganda injection | CIV-0105, CIV-0106 | P1 |
| CONTRACT-INF-005 | Influence/OTC | Climate policy weaponization | CIV-0102, CIV-0105 | P2 |

### 10.1 P0 Contracts Summary (Must Ship at v1.0)

The following contracts are P0 and block the v1.0 release of CivLab:

1. Citizen need satisfaction model (CONTRACT-C3-POP-001)
2. Grievance accumulation → insurgency (CONTRACT-C3-POP-002, CONTRACT-DF-001, CONTRACT-DF-002)
3. Market clearing solver over 9-good taxonomy (CONTRACT-C3-MKT-001)
4. Trade route Joule cost model (CONTRACT-C3-MKT-002)
5. Regime-agnostic conservation substrate (CONTRACT-C3-MKT-003)
6. Multi-tick covert operation scheme (CONTRACT-CK3-AI-002, CONTRACT-INF-002)
7. Bilateral nation trust model (CONTRACT-CK3-AI-003)
8. Throughput-constrained district production (CONTRACT-FAC-001)
9. Joule grid derating mechanic (CONTRACT-FAC-002)
10. Trade route profitability and throughput capacity (CONTRACT-OTTD-001, CONTRACT-OTTD-002)
11. Climate stabilization threshold + non-linear remediation ROI (CONTRACT-TERRA-001, CONTRACT-TERRA-002)
12. Covert operation detection probability formula (CONTRACT-INF-001)

### 10.2 Design Violation Protocol

If an implementation deviates from a design contract:
1. Create an ADR in `ADR.md` documenting the deviation, rationale, and acceptance criteria for the alternative.
2. Tag the ADR with the CONTRACT-ID being superseded.
3. Update this document's delta table for the relevant section.
4. Notify the CIV Architecture team before merging.

Deviations without ADR will be flagged in code review and rejected.

---

## Backmatter

### Decision Delta Summary (Cross-Game)

| Game | Most Important Adoption | Most Important Extension | Most Important Replacement |
|------|------------------------|--------------------------|---------------------------|
| Victoria 3 (Pop) | Needs tier satisfaction model | Per-citizen tracking replaces grouped pops | Religion axis → multi-axis ideology vector |
| Victoria 3 (Market) | Iterative supply/demand clearing | Joule-backed currency | ~50 goods → 9 abstract categories |
| Dwarf Fortress | Stress accumulation → threshold model | Collective political outcome replaces individual breakdown | Doctor treatment → welfare floor policy |
| Crusader Kings 3 | Multi-tick schemes for covert ops | Institutional memory replaces dynasty mechanics | Character traits → nation personality archetypes |
| Factorio | Throughput-constrained production model | Joule grid is the meta-constraint across all production | Visual factory layout → district abstraction |
| OpenTTD | Route profitability formula | Joule transport cost replaces calendar time penalty | Vehicle management → route capacity agreement |
| Terra Nil | Non-linear early investment ROI | Multi-nation collective action problem | Single-player optimization → multi-nation free-rider problem |
| Influence/OTC | Detection probability formula | Social network propaganda integration | Real-time items → multi-tick persistent spy network |

### Validation Commands

```bash
# Verify all P0 design contracts have corresponding test coverage
grep -r "CONTRACT-" crates/*/tests/ | sort | uniq
# Expected: all 12 P0 contract IDs appear in at least one test file

# Verify market clearing solver is regime-agnostic (runs on all three allocation engines)
cargo test --package crates/economy test_allocation_engine_market
cargo test --package crates/economy test_allocation_engine_planned
cargo test --package crates/economy test_allocation_engine_joule_quota

# Verify conservation invariant holds after market clearing
cargo test --package crates/economy test_conservation_invariant_all_regimes

# Verify insurgency propensity threshold model
cargo test --package crates/social test_mobilization_threshold_crossing

# Verify covert operation multi-tick progress accumulation
cargo test --package crates/diplomacy test_covert_op_progress_accumulation

# Verify Joule grid derating mechanics
cargo test --package crates/energy test_grid_derating_linear_with_shortage
```

### Residual Design Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Per-citizen simulation at 5M pop exceeds 1s tick budget | Performance miss → unable to simulate large scenarios | Profile Phase 1 and 4 (O(N) phases); parallelize with rayon; fall back to cohort simulation if N > 1M with opt-in |
| Multi-axis ideology diffusion produces degenerate equilibria (all citizens converge to same ideology) | Emergent complexity collapses | Tune damping coefficient and minimum diversity parameter; verify diversity is maintained in 100-run research mode |
| Collective action problem for climate makes game unsolvable (no Nash equilibrium for climate cooperation) | Game unwinnable on climate path | Ensure shadow network + diplomacy + international institution mechanics provide sufficient coordination tools |
| Non-linear climate remediation ROI is too subtle for players to discover | Players don't invest early enough; climate spiral inevitable | Tutorial scenario specifically teaching early climate investment ROI; research mode data export for studying the curve |

### Follow-up Review Date

This reference analysis is scheduled for review on **2026-08-21** (6 months after publication). Triggers for immediate update:
- New reference game identified by the design team
- Material change to any referenced CIV spec that affects a design contract
- Playtesting evidence that a design contract produces unfun or unrealistic outcomes

---

## 11. Formal Pseudocode Library

This section consolidates all formal pseudocode and formulas from the reference game analyses into a single reference for implementors. Each entry cross-references its source section.

### 11.1 Citizen Need Satisfaction (from Section 2.2 + CONTRACT-C3-POP-001)

```rust
// CivLab Rust pseudocode — citizen need satisfaction per tick
// Located in: crates/social/src/needs.rs

const NEEDS_TIERS: [NeedsTier; 4] = [
    NeedsTier { id: Energy,    weight: 1.5, threshold: 0.9 },  // Energy below food
    NeedsTier { id: Survival,  weight: 1.0, threshold: 0.8 },  // Food, water, shelter
    NeedsTier { id: Standard,  weight: 0.5, threshold: 0.5 },  // Medicine, furniture, services
    NeedsTier { id: Luxury,    weight: 0.2, threshold: 0.3 },  // Luxury food, arts, fine goods
];

fn compute_happiness_delta(citizen: &Citizen, consumption: &ConsumptionBasket) -> i64 {
    let mut delta: i64 = 0;

    // Process tiers in order; apply penalty for each unmet tier
    for tier in &NEEDS_TIERS {
        let demanded = citizen.needs_demand(tier.id);
        let consumed = consumption.get(tier.id);
        let satisfaction = if demanded > 0 { consumed as f64 / demanded as f64 } else { 1.0 };

        if satisfaction < tier.threshold {
            let deficit = tier.threshold - satisfaction;
            // Fixed-point: multiply by 1000 for precision, divide at output
            let penalty = (deficit * tier.weight * HAPPINESS_PENALTY_SCALE) as i64;
            delta -= penalty;
        } else {
            // Small positive reinforcement for well-met needs
            delta += (tier.weight * HAPPINESS_BONUS_PER_MET_TIER) as i64;
        }
    }

    // Additional modifiers
    delta += job_satisfaction_delta(citizen);
    delta += ideology_alignment_delta(citizen);  // +10 if match, -5 if mismatch
    delta += social_cohesion_delta(citizen);     // positive if high cohesion in region

    delta
}

fn job_satisfaction_delta(citizen: &Citizen) -> i64 {
    match citizen.current_job {
        Some(job) => JOB_HAPPINESS_TABLE[job.category] as i64,
        None => UNEMPLOYMENT_HAPPINESS_PENALTY,  // -30
    }
}
```

---

### 11.2 Market Clearing Solver (from Section 3.2 + CONTRACT-C3-MKT-001)

```rust
// CivLab Rust pseudocode — iterative market clearing
// Located in: crates/economy/src/market_clearing.rs

const CLEARING_ITERATIONS: usize = 5;
const CONVERGENCE_EPSILON: i64 = 100; // 0.1% in fixed-point (100/100000)

fn clear_market(
    supply: &BTreeMap<GoodCategory, i64>,    // units produced this tick
    demand: &BTreeMap<GoodCategory, i64>,    // units demanded this tick
    prices: &mut BTreeMap<GoodCategory, i64>, // price vector in fixed-point Drachma
    elasticity: &BTreeMap<GoodCategory, i64>, // price sensitivity per good
) -> MarketClearingResult {
    let mut converged = false;

    for iteration in 0..CLEARING_ITERATIONS {
        let mut max_tension: i64 = 0;

        for good in GoodCategory::all() {
            let s = supply.get(&good).copied().unwrap_or(0);
            let d = demand.get(&good).copied().unwrap_or(0);

            if s == 0 { continue; }  // No supply: price undefined

            // Tension = (demand - supply) / supply in fixed-point
            let tension = ((d - s) * FIXED_POINT_SCALE) / s;

            // Price update: price *= (1 + tension * elasticity)
            let price_change = (tension * elasticity[&good]) / FIXED_POINT_SCALE;
            let new_price = prices[&good] + (prices[&good] * price_change / FIXED_POINT_SCALE);

            // Clamp to floor/ceiling
            prices.insert(good, clamp(new_price, PRICE_FLOOR[good], PRICE_CEILING[good]));

            max_tension = max_tension.max(tension.abs());
        }

        if max_tension < CONVERGENCE_EPSILON {
            converged = true;
            break;
        }
    }

    MarketClearingResult {
        clearing_prices: prices.clone(),
        converged,
        iterations_used: if converged { CLEARING_ITERATIONS } else { CLEARING_ITERATIONS },
    }
}
```

---

### 11.3 Cohesion Decay Function (from Section 4.2 + CONTRACT-DF-001)

```rust
// CivLab Rust pseudocode — cohesion decay per tick
// Located in: crates/social/src/cohesion.rs
// Stored in fixed-point Q16.16 (scale = 65536)

const COHESION_SCALE: i64 = 65536;

struct CohesionDecayDrivers {
    material_stress_index: i64,      // 0..COHESION_SCALE: fraction of pop below needs threshold
    coercion_index: i64,             // from CIV-0105 enforcement_intensity
    shadow_capture_score: i64,       // from CIV-0105 institutional capture
    welfare_delivery_rate: i64,      // 0..COHESION_SCALE: fraction of needs met by welfare
    civic_participation_rate: i64,   // 0..COHESION_SCALE: fraction engaged in civic activities
}

// Decay coefficients (tunable via scenario parameters)
const ALPHA_MATERIAL: i64 = 2000;    // ~0.030 per unit stress
const ALPHA_COERCION: i64 = 3000;    // ~0.046 per unit coercion
const ALPHA_CAPTURE: i64 = 1500;     // ~0.023 per unit shadow capture
const BETA_WELFARE: i64 = 1000;      // ~0.015 reinforcement per unit welfare
const BETA_CIVIC: i64 = 800;         // ~0.012 reinforcement per unit civic

fn compute_cohesion_delta(
    cohesion: i64,   // current cohesion in Q16.16
    drivers: &CohesionDecayDrivers,
) -> i64 {
    // Decay term (negative contribution)
    let decay = (drivers.material_stress_index * ALPHA_MATERIAL
                + drivers.coercion_index * ALPHA_COERCION
                + drivers.shadow_capture_score * ALPHA_CAPTURE)
                / COHESION_SCALE;

    // Reinforcement term (positive contribution)
    let reinforcement = (drivers.welfare_delivery_rate * BETA_WELFARE
                        + drivers.civic_participation_rate * BETA_CIVIC)
                        / COHESION_SCALE;

    let delta = reinforcement - decay;
    // Return delta; caller clamps result to [0, COHESION_SCALE]
    delta
}
```

---

### 11.4 Insurgency Propensity (from Section 4.2 + CONTRACT-DF-002)

```rust
// CivLab Rust pseudocode — insurgency propensity computation
// Located in: crates/social/src/insurgency.rs

struct InsurgencyPropensityDrivers {
    cohesion: i64,                    // Q16.16; low cohesion → high propensity
    material_deprivation: i64,        // fraction of pop below survival needs threshold
    ideology_mismatch: i64,           // distance between regime ideology and pop ideology
    coercion_intensity: i64,          // enforcement from CIV-0105
    external_support: i64,            // shadow network external backing (from CIV-0105)
    historical_trauma: i64,           // accumulated historical grievance (slow-decaying)
}

// Coefficients
const W_COHESION: i64 = -4000;        // negative: high cohesion reduces propensity
const W_DEPRIVATION: i64 = 5000;
const W_IDEOLOGY: i64 = 3000;
const W_COERCION_SHORT: i64 = -2000; // coercion short-run reduces overt action
const W_COERCION_LONG: i64 = 1500;   // coercion long-run raises grievance (cohesion decay)
const W_EXTERNAL: i64 = 2000;
const W_TRAUMA: i64 = 1000;

fn compute_propensity(drivers: &InsurgencyPropensityDrivers) -> i64 {
    let raw = (drivers.cohesion * W_COHESION
              + drivers.material_deprivation * W_DEPRIVATION
              + drivers.ideology_mismatch * W_IDEOLOGY
              + drivers.coercion_intensity * W_COERCION_SHORT   // short-run suppression
              + drivers.coercion_intensity * W_COERCION_LONG    // long-run grievance
              + drivers.external_support * W_EXTERNAL
              + drivers.historical_trauma * W_TRAUMA)
              / FIXED_POINT_SCALE;

    // Clamp to [0, PROPENSITY_MAX]
    raw.clamp(0, PROPENSITY_MAX)
}

// Stochastic cell formation: fires in Phase 4 (stochastic events)
fn maybe_form_cell(
    propensity: i64,
    mobilization_threshold: i64,
    rng: &mut ChaCha20Rng,
) -> Option<InsurgencyCell> {
    if propensity < mobilization_threshold {
        return None;
    }

    // Probability of cell formation is proportional to excess propensity above threshold
    let excess = propensity - mobilization_threshold;
    let formation_probability = (excess * FORMATION_PROBABILITY_SCALE) / PROPENSITY_MAX;

    if rng.next_u64() % PROBABILITY_DENOMINATOR < formation_probability as u64 {
        Some(InsurgencyCell::new())
    } else {
        None
    }
}
```

---

### 11.5 AI Utility Function (from Section 5.2 + CONTRACT-CK3-AI-001)

```rust
// CivLab Rust pseudocode — AI nation utility function
// Located in: crates/ai/src/decision.rs

enum NationArchetype {
    Expansionist,   // Prefers territorial acquisition, war
    Mercantile,     // Prefers trade, economic growth
    Isolationist,   // Prefers stability, minimal foreign engagement
    Militarist,     // Prefers military buildup, deterrence
    Ideological,    // Prefers spreading ideology, alliances of shared values
}

struct ArchetypeModifiers {
    war_declaration_bonus: i64,     // additive bonus to utility of declaring war
    trade_agreement_bonus: i64,     // additive bonus to utility of trade agreements
    alliance_bonus: i64,            // additive bonus to utility of forming alliances
    covert_op_bonus: i64,           // additive bonus to utility of covert operations
    climate_investment_bonus: i64,  // additive bonus to climate investment utility
}

// Archetype modifier table (fixed at game start, modified by events over time)
const ARCHETYPE_TABLE: [(NationArchetype, ArchetypeModifiers); 5] = [
    (Expansionist,  ArchetypeModifiers { war_declaration_bonus: 30,  trade_agreement_bonus: -10, alliance_bonus: 10,  covert_op_bonus: 15,  climate_investment_bonus: -20 }),
    (Mercantile,    ArchetypeModifiers { war_declaration_bonus: -20, trade_agreement_bonus: 40,  alliance_bonus: 20,  covert_op_bonus: 10,  climate_investment_bonus: 15  }),
    (Isolationist,  ArchetypeModifiers { war_declaration_bonus: -30, trade_agreement_bonus: -5,  alliance_bonus: -20, covert_op_bonus: -10, climate_investment_bonus: 5   }),
    (Militarist,    ArchetypeModifiers { war_declaration_bonus: 40,  trade_agreement_bonus: -15, alliance_bonus: 5,   covert_op_bonus: 20,  climate_investment_bonus: -25 }),
    (Ideological,   ArchetypeModifiers { war_declaration_bonus: 0,   trade_agreement_bonus: 10,  alliance_bonus: 30,  covert_op_bonus: 25,  climate_investment_bonus: 20  }),
];

fn evaluate_action_utility(
    action: &StrategicAction,
    nation: &Nation,
    situation: &SituationalContext,
) -> i64 {
    let base_utility = action.base_weight(situation);
    let modifier = get_archetype_modifier(nation.archetype, action.action_type);
    let situational = situational_modifier(action, situation);

    let total = base_utility + modifier + situational;

    // Minimum threshold: actions below 10 utility are not considered
    if total < MIN_ACTION_UTILITY_THRESHOLD { return 0; }
    total
}

// AI picks the action with highest utility across all available options
fn ai_select_action(
    nation: &Nation,
    available_actions: &[StrategicAction],
    situation: &SituationalContext,
) -> Option<StrategicAction> {
    available_actions.iter()
        .map(|a| (a, evaluate_action_utility(a, nation, situation)))
        .filter(|(_, u)| *u > 0)
        .max_by_key(|(_, u)| *u)
        .map(|(a, _)| a.clone())
}
```

---

### 11.6 Covert Operation Detection Probability (from Section 9.2 + CONTRACT-INF-001)

```rust
// CivLab Rust pseudocode — covert operation detection probability
// Located in: crates/diplomacy/src/covert_ops.rs

// Base detection rates per operation type (probability = P * 1000 for fixed-point)
const BASE_DETECTION: [(OperationType, i64); 6] = [
    (IntelGathering,       300),   // 30% base
    (TechSabotage,         500),   // 50% base
    (ProductionDisruption, 450),   // 45% base
    (Assassination,        800),   // 80% base
    (Propaganda,           250),   // 25% base
    (InfrastructureSabotage, 600), // 60% base
];

// Calibration constants
// At max security (k1 = 1.0 security investment), detection should reach ~80-85%
// At max skill (k2 = 1.0 espionage skill), detection should reach ~5-10%
const K1: i64 = 1000;  // Security normalization (fixed-point 1.0)
const K2: i64 = 1000;  // Skill normalization (fixed-point 1.0)

fn compute_detection_probability(
    op_type: OperationType,
    target_security: i64,    // 0..K1
    attacker_skill: i64,     // 0..K2
) -> i64 {
    let base = BASE_DETECTION.iter()
        .find(|(t, _)| *t == op_type)
        .map(|(_, p)| *p)
        .unwrap_or(500);  // default 50% if unknown

    // formula: detection = base * (1 + security/K1) * (1 - skill/K2)
    // All in fixed-point; base is per-1000
    let security_factor = (K1 + target_security) ;  // 1000 + security
    let skill_factor = (K2 - attacker_skill);        // 1000 - skill

    let detection = base * security_factor / K1 * skill_factor / K2;
    // detection is now in per-1000 scale

    // Clamp: minimum 5% (1 in 20) even against perfectly skilled attacker
    //        maximum 95% (cannot guarantee detection)
    detection.clamp(50, 950)
}

fn tick_detection_roll(
    detection_probability: i64,  // 0..1000
    rng: &mut ChaCha20Rng,
) -> bool {
    (rng.next_u64() % 1000) < detection_probability as u64
}
```

---

### 11.7 Joule Grid Energy Balance (from Section 6.2 + CONTRACT-FAC-002)

```rust
// CivLab Rust pseudocode — Joule grid balance and derating
// Located in: crates/energy/src/grid.rs

struct EnergyGridState {
    produced: i64,    // total joules produced this tick (all districts, all sources)
    consumed: i64,    // total joules demanded this tick (all districts, all citizens)
    reserved: i64,    // joules in energy reserves (batteries, storage districts)
    reserve_capacity: i64,  // maximum reserve capacity
}

fn compute_energy_balance(grid: &EnergyGridState) -> EnergyBalanceResult {
    let net = grid.produced - grid.consumed;

    if net >= 0 {
        // Surplus: charge reserves; any excess over reserve_capacity is wasted
        let reserve_charge = net.min(grid.reserve_capacity - grid.reserved);
        let wasted = net - reserve_charge;
        EnergyBalanceResult {
            efficiency: FIXED_POINT_SCALE,  // 1.0: all districts run at full speed
            reserve_delta: reserve_charge,
            wasted,
            shortage: 0,
        }
    } else {
        // Shortage: draw from reserves first
        let shortage_magnitude = (-net).min(grid.reserved);  // draw from reserves
        let remaining_shortage = (-net) - shortage_magnitude;

        let efficiency = if grid.consumed > 0 {
            // Efficiency = (produced + drawn_from_reserves) / consumed
            // Linear derating: all districts receive proportional power
            (grid.produced + shortage_magnitude) * FIXED_POINT_SCALE / grid.consumed
        } else {
            FIXED_POINT_SCALE
        };

        EnergyBalanceResult {
            efficiency: efficiency.clamp(0, FIXED_POINT_SCALE),
            reserve_delta: -(shortage_magnitude as i64),
            wasted: 0,
            shortage: remaining_shortage.max(0),
        }
    }
}

// Apply derating to all production districts
fn apply_energy_derating(
    districts: &mut [ProductionDistrict],
    efficiency: i64,  // 0..FIXED_POINT_SCALE
) {
    for district in districts.iter_mut() {
        district.actual_output = district.theoretical_output * efficiency / FIXED_POINT_SCALE;
    }
}
```

---

### 11.8 Trade Route Profitability (from Section 7.2 + CONTRACT-OTTD-001)

```rust
// CivLab Rust pseudocode — trade route profitability calculation
// Located in: crates/economy/src/trade_routes.rs

struct TradeRoute {
    city_a: CityId,
    city_b: CityId,
    good: GoodCategory,
    infrastructure_quality: InfrastructureQuality,  // Road, Rail, Port, Air
    distance_units: i64,  // abstract distance units between cities
}

// Transport cost per unit-distance per good, by infrastructure quality
// In fixed-point Drachma per unit per distance unit
const TRANSPORT_COST_TABLE: [[i64; 4]; 9] = [
    // Road, Rail, Port, Air
    [50, 20, 15, 100],   // Essentials (food, medicine) — perishable surcharge on air
    [40, 15, 10, 80],    // Discretionary
    [60, 25, 20, 150],   // Capital goods (heavy; air expensive)
    [30, 10, 8,  60],    // Public services (intangible; lower transport cost)
    [80, 30, 25, 200],   // Energy (fuel; high weight)
    // ... other good categories ...
];

fn compute_trade_profitability(
    route: &TradeRoute,
    price_a: i64,    // price of good in city A (Drachma/unit, fixed-point)
    price_b: i64,    // price of good in city B (Drachma/unit, fixed-point)
    units: i64,      // units traded
) -> TradeProfitabilityResult {
    let price_differential = (price_b - price_a).max(0);  // only trade if B price > A price

    let transport_cost_per_unit_per_distance =
        TRANSPORT_COST_TABLE[route.good as usize][route.infrastructure_quality as usize];
    let total_transport_cost =
        transport_cost_per_unit_per_distance * route.distance_units * units / FIXED_POINT_SCALE;

    let gross_profit = price_differential * units / FIXED_POINT_SCALE;
    let net_profit = gross_profit - total_transport_cost;

    TradeProfitabilityResult {
        gross_profit,
        transport_cost: total_transport_cost,
        net_profit,
        // Route is only executed if net_profit > 0 (merchants do not trade at a loss)
        should_execute: net_profit > 0,
    }
}
```

---

### 11.9 Climate Remediation Non-Linear ROI (from Section 8.2 + CONTRACT-TERRA-001)

```rust
// CivLab Rust pseudocode — climate event probability and remediation ROI
// Located in: crates/climate/src/carbon.rs

// Climate event probability as function of CO2 ppm (per tick)
// Deliberately non-linear; matches CIVLAB_GAME_DESIGN Phase 8 probability curve
fn climate_event_probability_per_tick(co2_ppm: i64) -> i64 {
    // Returns probability in fixed-point per-1000 scale
    if co2_ppm < 350 {
        0      // Safe zone: no events
    } else if co2_ppm < 450 {
        // Linear: 0% → 1% per tick
        (co2_ppm - 350) * 10 / 100   // max 10 (1.0%)
    } else if co2_ppm < 550 {
        // Faster: 1% → 10% per tick
        10 + (co2_ppm - 450) * 90 / 100  // 10..100
    } else {
        // Catastrophe: 50% per tick
        500
    }
}

// ROI of reducing CO2 by 1 ppm at current level
// Higher ROI when reducing from dangerous zone than from safe zone
fn marginal_remediation_roi(current_co2_ppm: i64) -> i64 {
    let current_prob = climate_event_probability_per_tick(current_co2_ppm);
    let reduced_prob = climate_event_probability_per_tick(current_co2_ppm - 1);
    let prob_reduction = current_prob - reduced_prob;

    // ROI = probability_reduction * average_climate_event_damage / remediation_cost_per_ppm
    // Higher probability zones have steeper prob curves → higher ROI per ppm
    prob_reduction * AVG_CLIMATE_EVENT_DAMAGE / REMEDIATION_COST_PER_PPM
}

// Example: reducing from 560ppm → 559ppm has ROI ~50x higher than 360ppm → 359ppm
// This implements CONTRACT-TERRA-001: non-linear early investment ROI
```

---

## 12. Cross-Game Mechanic Interaction Map

Some of CivLab's most interesting emergent behaviors arise from the interaction between subsystems derived from different reference games. This section maps the key cross-subsystem interactions.

### 12.1 Victoria 3 Market × Dwarf Fortress Stress

When the market system (Section 3) clears at prices that citizens cannot afford (need satisfaction falls below tier threshold), the Dwarf Fortress stress model (Section 4) kicks in. This creates the economic → social chain:

```
Market Event: food_price spikes (shortage)
    ↓
V3 Market: need_satisfied[Survival] < threshold for farmer citizens
    ↓
DF Stress: cohesion_decay += material_stress driver
    ↓
CIV-0106: cohesion falls → insurgency propensity rises
    ↓
DF Stress threshold: if propensity > mobilization_threshold
    → stochastic cell_formation event fires
    ↓
CIV-0105: insurgency_risk sent to war/diplomacy system
    ↓
If unresolved: civil war trigger
```

**Design implication:** Players must prevent market failure (ensure food supply exceeds demand) not just for economic reasons but to prevent the social cascade into insurgency. This is the CivLab equivalent of the DF "food shortage → mass starvation → fortress collapse" doom spiral, but at civilization scale.

---

### 12.2 Factorio Production Graph × Terra Nil Climate

The Factorio production graph (Section 6) and Terra Nil climate remediation (Section 8) interact through the energy source selection:

```
Player decision: build coal factory districts (cheap, immediate Joules)
    ↓
FAC Production: Joule production increases; energy shortage resolved
    ↓
TERRA Climate: coal_production += CO2_emission_factor * joules_produced
    ↓
CO2 accumulation: global carbon budget ticks toward next threshold
    ↓
If CO2 > 450ppm: climate event probability jumps to 10% per tick
    ↓
Climate event: drought hits wheat regions (farm district output -50%)
    ↓
V3 Market: food supply drops; prices spike
    ↓
DF Stress: citizen stress accumulates from food insecurity
    ↓
[insurgency chain from 12.1]
```

**Design implication:** The energy production decision (coal vs. renewable) has a causal chain that spans every major subsystem. A player optimizing purely for immediate Joule production is inadvertently triggering a climate → food → social → political cascade. This is the CivLab version of Factorio's "build a nuclear plant before your coal supply runs out" challenge, but with civilization-wide stakes.

---

### 12.3 CK3 Covert Operations × Victoria 3 Ideology Diffusion

The CK3 covert operations system (Section 5) interacts with Victoria 3's ideology model (Section 2) through the propaganda mechanic:

```
Nation A's espionage operation: "Propaganda campaign against Nation B"
    ↓
INF Covert Ops: rumor injected into Nation B's information network
    ↓
V3/CIV-0106 Ideology Diffusion: rumor spreads through social graph
    (distortion coefficient: 60% accuracy → 80% accuracy → 50% accuracy as spreads)
    ↓
Citizens update belief state: ideology_vector shifts toward rumor content
    ↓
Ideology mismatch increases between citizens and current government
    ↓
DF Stress: cohesion_decay += ideology_mismatch driver
    ↓
If propensity crosses threshold: insurgency cell formed
    ↓
Civil war: Nation B weakened; Nation A's strategic position improves
```

**Design implication:** Information operations are a slower, cheaper alternative to military force. A nation that invests heavily in espionage can destabilize a rival through propaganda faster and at lower cost than a military campaign. This is the CivLab implementation of "influence" as a strategic resource — directly answering the design inspiration from the Influence / OTC reference (Section 9).

---

### 12.4 OpenTTD Trade Routes × Factorio Energy × Terra Nil Climate

```
Energy trade route: Nation A (solar surplus) → Nation B (energy deficit)
    ↓
OTTD Trade Route: Nation B receives Joules via trade route
    (route profitability: price_B - price_A > joule_transport_cost)
    ↓
FAC Energy: Nation B's energy shortage resolved without building coal
    ↓
TERRA Climate: Nation B's CO2 emissions do not increase
    (coal alternative avoided)
    ↓
Global carbon budget: reduced pressure toward next threshold
    ↓
TERRA Climate: climate event probability remains lower for all nations
    (shared global carbon budget)
    ↓
CONTRACT-TERRA-004: collective action benefit — Nation A and B both gain
    from the trade route even though only Nation B directly benefits from
    cheaper energy
```

**Design implication:** Energy trade is not just economically valuable — it is a climate coordination mechanism. Nations with renewable energy surplus have an incentive to build export capacity (it generates Drachma revenue) and the importing nation avoids coal emissions. This creates a natural alignment of economic incentives with climate goals, unlike purely coercive international climate agreements.

---

## 13. Implementation Phasing Guide

This section maps design contracts to suggested implementation phases, based on dependency ordering.

### 13.1 Phase 1: Foundation (Month 1–2)

These contracts establish the conservation substrate that all other systems depend on:

| Contract | Implementation | Dependency |
|----------|---------------|-----------|
| CONTRACT-C3-MKT-003 | Conservation equation; double-entry ledger | None |
| CONTRACT-FAC-001 | Throughput-constrained production function | Conservation substrate |
| CONTRACT-FAC-002 | Joule grid balance and derating | Production function |
| CONTRACT-C3-MKT-001 | Market clearing solver (3 iterations minimum) | Conservation substrate |

**Output:** A tick that produces goods, clears the market, and balances the Joule grid, with conservation invariant verified after each tick. No citizens, no politics, no climate.

### 13.2 Phase 2: Citizens and Social Dynamics (Month 3–4)

| Contract | Implementation | Dependency |
|----------|---------------|-----------|
| CONTRACT-C3-POP-001 | Citizen need satisfaction model | Production (Phase 1) |
| CONTRACT-DF-001 | Stress accumulation model | Needs satisfaction |
| CONTRACT-C3-POP-002 | Grievance → insurgency threshold | Stress model |
| CONTRACT-DF-002 | Stochastic cell formation | Threshold model |
| CONTRACT-OTTD-001 | Trade route profitability | Market clearing |
| CONTRACT-OTTD-002 | Trade route throughput capacity | Trade routes |

**Output:** Citizens who can be happy or stressed, with insurgency emerging from sustained stress. Trade between cities operational.

### 13.3 Phase 3: AI, Diplomacy, and Covert Operations (Month 5–6)

| Contract | Implementation | Dependency |
|----------|---------------|-----------|
| CONTRACT-CK3-AI-002 | Multi-tick covert operations | Social dynamics (Phase 2) |
| CONTRACT-CK3-AI-003 | Bilateral trust model | Diplomacy foundation |
| CONTRACT-INF-001 | Detection probability formula | Covert operations |
| CONTRACT-INF-002 | Multi-tick operation progress | Covert operations |
| CONTRACT-CK3-AI-001 | AI utility function with archetypes | All of Phase 2 |

**Output:** Nations that make strategic decisions and run espionage operations against each other.

### 13.4 Phase 4: Climate and Environmental Systems (Month 7–8)

| Contract | Implementation | Dependency |
|----------|---------------|-----------|
| CONTRACT-FAC-003 | Renewable energy variability | Energy grid (Phase 1) |
| CONTRACT-TERRA-001 | Non-linear climate remediation ROI | Climate model |
| CONTRACT-TERRA-002 | Climate stabilization threshold | Climate model |
| CONTRACT-TERRA-003 | Stranded asset economics | Production + climate |
| CONTRACT-TERRA-004 | Collective action climate problem | Diplomacy (Phase 3) |

**Output:** Full climate simulation with collective action dynamics, renewable energy variability, and stranded asset economics.

### 13.5 Phase 5: Polish and Extended Mechanics (Month 9–12)

| Contract | Implementation | Dependency |
|----------|---------------|-----------|
| CONTRACT-C3-POP-004 | Multi-axis ideology vector (R^d) | Social dynamics |
| CONTRACT-DF-003 | Social network grief propagation | Social network |
| CONTRACT-DF-004 | Pull-based job assignment | Citizens |
| CONTRACT-CK3-AI-004 | Governance legitimacy decay | Institutions + ideology |
| CONTRACT-INF-003 | Intelligence value decay | Covert ops |
| CONTRACT-INF-004 | Propaganda injection | Social network |
| CONTRACT-C3-MKT-004 | Lagged supply response | Market clearing |
| CONTRACT-OTTD-003 | Good-type transport cost modifiers | Trade routes |
| CONTRACT-FAC-004 | District utilization rate metric | Production |

**Output:** Full CivLab v1.0 with all design contracts satisfied.
