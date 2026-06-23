# RND-015: Victoria 3 / Dwarf Fortress Simulation Patterns -- Academic Literature and Open Implementation References

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-delta

---

## Executive Summary

This document surveys simulation design patterns from three primary game references (Victoria 3, Dwarf Fortress, OpenTTD) and four foundational academic models (Sugarscape, Schelling, SIR, Hammond-Axelrod) to derive concrete design contracts for CivLab. Each source is annotated with the specific CivLab system it informs, the tunable parameters it implies, and the mathematical formulations that should be implemented. The result is a reference library of simulation patterns that CivLab's designers and implementers can draw from, with explicit contracts mapping academic theory to CivLab's domain.

---

## Research Findings

### Part I: Game Reference Systems

---

### 1. Victoria 3 -- Population and Economy Simulation

**Source:** Paradox Interactive dev diaries (2021-2025), Paradox Wiki, Mikael Andersson's GDC/Gamasutra deep dive on V3 economy.

#### 1.1 Pop System

Victoria 3's fundamental simulation unit is the "Pop" (population group). Unlike Dwarf Fortress's individual-level simulation, V3 aggregates people into groups sharing the same:
- **State** (geographic region)
- **Culture**
- **Religion**
- **Profession** (Aristocrats, Capitalists, Bureaucrats, Officers, Shopkeepers, Machinists, Laborers, Peasants, etc.)

**Key mechanics:**

| Mechanic | Description | CivLab Analog |
|----------|-------------|---------------|
| Pop growth | Birth/death rates affected by Standard of Living, healthcare laws, literacy | Population growth per cell/district |
| Migration | Pops move between states based on economic opportunity differential | Inter-cell population movement |
| Profession change | Pops shift profession based on available employment in buildings | Workforce reallocation |
| Radicalization | Pops become radical when their Standard of Living drops below expectations | Unrest / ideology shift |
| Loyalty | Pops become loyal when Standard of Living exceeds expectations | Stability bonus |
| Qualifications | Pops gain qualifications (literacy, skills) over time, enabling higher-tier employment | Technology workforce requirements |

**Standard of Living (SoL)** is the central Pop welfare metric. It is computed from:
- Wealth (income minus expenses)
- Goods consumption (which goods the Pop can afford, weighted by cultural preference)
- Political rights (laws granting or restricting franchise, education, labor rights)

**CivLab contract:** CivLab's citizen satisfaction model (CIV-0102) should derive from SoL: a weighted composite of consumption fulfillment, political freedom, and economic opportunity. The weights must be tunable per scenario.

#### 1.2 Market System

V3 uses a **closed-market equilibrium** model:

1. **Buildings** produce goods (Sell Orders) and consume goods (Buy Orders).
2. **Pops** consume goods based on wealth tier and cultural preferences (Buy Orders).
3. **Price** is set by the ratio of total Buy Orders to total Sell Orders for each good.
4. **Price range:** Base price +/- 75%. At 50% oversupply, price hits the floor. At 50% undersupply, price hits the ceiling.
5. **Market clearing:** When supply < demand, goods are rationed proportionally across all buyers.
6. **Trade routes** connect markets. Goods flow along trade routes, generating Buy/Sell orders in both markets.

**Price formula (simplified):**

```
price_ratio = buy_orders / sell_orders
if price_ratio <= 0.5:
    price = base_price * 0.25     # floor
elif price_ratio >= 2.0:
    price = base_price * 1.75     # ceiling
else:
    price = base_price * lerp(0.25, 1.75, (price_ratio - 0.5) / 1.5)
```

**Substitution:** V3 handles substitutable goods (e.g., Grain vs Fruit for food). When a preferred good is expensive, Pops substitute cheaper alternatives, reducing demand for the expensive good and increasing demand for the substitute.

**CivLab contract:** CivLab's economy (CIV-0201) should implement a similar Buy/Sell order market. Tunable parameters:
- `price_elasticity_range`: float (default 0.75, meaning +/- 75% of base price)
- `oversupply_threshold`: float (default 0.5, supply exceeds demand by this ratio to hit floor)
- `undersupply_threshold`: float (default 0.5, demand exceeds supply by this ratio to hit ceiling)
- `substitution_coefficient`: float per good-pair (how readily good A substitutes for good B)

#### 1.3 AI Decision-Making

V3's AI uses a **weighted utility system** for strategic decisions:
- Each possible action (build a factory, declare war, pass a law) is scored by a utility function.
- The utility function considers: economic impact, political feasibility, military strength, cultural alignment.
- The AI picks the highest-utility action, with some randomization for variety.
- Interest Groups (political factions) influence which actions the AI considers viable.

**CivLab contract:** CivLab's nation AI (CIV-0301) should use a weighted utility scorer with pluggable evaluation functions. The MCTS approach (RND-011) is for tactical decisions; the V3-style utility scorer is for strategic long-term planning.

---

### 2. Dwarf Fortress -- Individual Agent Simulation

**Source:** Dwarf Fortress Wiki (dwarffortresswiki.org), Tarn Adams' "Simulation Principles from Dwarf Fortress" (Game AI Pro 2, Chapter 41), Steam community discussions.

#### 2.1 Need Satisfaction System

DF's need system is the most detailed individual-agent welfare model in any game. Each dwarf has 30+ distinct needs, each with a personality-weighted priority level.

**Need categories (partial list):**

| Need | Personality Trait Driver | Satisfaction Activity |
|------|-------------------------|----------------------|
| Alcohol | Immoderation | Drink at tavern |
| Prayer | Religiosity | Pray at temple |
| Social interaction | Gregariousness | Socialize at tavern/meeting hall |
| Creativity | Creativity | Create art, craft items |
| Martial training | Martial prowess | Spar, train in barracks |
| Romance | Romance value | Seek partner |
| Nature/Animals | Nature appreciation | Visit pastures, observe animals |
| Learning | Intellectual curiosity | Read books, attend lectures |
| Acquisition | Greed | Acquire wealth objects |
| Merriment | Fun-seeking | Attend parties, performances |
| Introspection | Introspection value | Meditate |

**Need fulfillment mechanics:**

1. Each need has an internal counter ranging from 400 (Unfettered) to -100,000+ (Badly distracted).
2. When a need is satisfied, its counter resets to 400 regardless of prior value.
3. Unsatisfied needs decay over time, passing through thresholds: Unfettered (400-300) -> Level-headed (299-200) -> Untroubled (199-100) -> Not distracted (99 to -999) -> Unfocused (-1000 to -9999) -> Distracted (-10,000 to -99,999) -> Badly distracted (-100,000+).
4. Need weights are personality-driven: proposed weights are 1, 2, 5, 10 per need level (higher level = more impact on focus).

**Focus formula:**

```
numerator = sum over all needs:
    Unfettered:      6.00 * weight
    Level-headed:    5.33 * weight
    Untroubled:      4.67 * weight
    Not distracted:  4.00 * weight
    Unfocused:       3.33 * weight
    Distracted:      2.67 * weight
    Badly distracted: 2.00 * weight

denominator = 4.0 * total_need_count

focus_ratio = floor(numerator) / denominator

Focus levels:
    >= 1.40: Very focused     (+50% skill bonus)
    >= 1.20: Quite focused
    >= 1.01: Focused
    == 1.00: Untroubled       (baseline)
    >= 0.81: Unfocused
    >= 0.61: Distracted
    <  0.61: Badly distracted (-50% skill penalty)
```

**CivLab contract:** CivLab does not simulate individual dwarves, but the need-satisfaction model maps to **district-level citizen satisfaction** in CivLab. Each district has a population with aggregate need fulfillment scores. The focus formula maps to **district productivity modifier**: a well-satisfied district produces more; an unsatisfied district produces less and generates unrest.

Tunable parameters:
- `need_decay_rate`: float per tick (how fast unsatisfied needs decay)
- `need_weights`: dict mapping need -> weight (personality distribution per culture)
- `focus_to_productivity_curve`: piecewise linear mapping from focus ratio to productivity modifier
- `focus_to_unrest_threshold`: float (below this focus ratio, district generates unrest events)

#### 2.2 Stress System

DF's stress system operates on two timescales:

**Short-term stress:** Range -100,000 to 100,000. Directly modified by thoughts (happy events subtract, unhappy events add). Maps to visible mood indicators (Ecstatic to Miserable).

**Long-term stress:** Range -50,000 to 120,000. Accumulates gradually from short-term stress. Status thresholds:
- Stressed: +25,000
- Haggard: +50,000
- Harrowed: +100,000

**Rates:**
- Maximum long-term stress increase: 20,160 per year (under constant misery)
- Maximum long-term stress decrease: 43,564 per year (under constant happiness)
- Recovery is ~2x faster than accumulation, but still takes years.

**Personality modifiers:**
- **Bravery:** Controls stress accumulation rate from combat/death events.
- **Stress vulnerability:** Determines the effective threshold capacity before breakdown.
- **Anxiety propensity:** Controls natural dissipation rate.

**Breakdown cascade:** Harrowed dwarves who witness death or receive additional stress triggers enter **insanity** (permanent, removes dwarf from useful labor). This is the primary "losing is fun" cascade mechanic.

**CivLab contract:** Map to **district morale** with two timescales:
- `district_mood`: short-term, event-driven, high volatility (analogous to DF short-term stress)
- `district_stability`: long-term, slow-moving, represents accumulated civic health (analogous to DF long-term stress)
- Tunable: `mood_to_stability_transfer_rate`, `stability_recovery_rate`, `stability_collapse_threshold`
- At `stability_collapse_threshold`, district enters crisis state (analogous to DF insanity cascade): production halts, emigration spikes, revolutionary events trigger.

#### 2.3 Tarn Adams' Four Simulation Principles (Game AI Pro 2)

From Chapter 41 of Game AI Pro 2:

1. **Base simulation on reality.** Use real-world analogues as design references. When the simulation produces unrealistic results, the real-world reference tells you what is wrong. Example: V3 rain shadows on mountains producing realistic biome distribution.

2. **Embrace emergent behavior.** Do not script high-level outcomes. Define low-level rules and let macro behavior emerge. Example: DF's fortress tantrum spirals emerge from individual stress mechanics, not from a scripted "tantrum spiral" event.

3. **Make the simulation inspectable.** Every value should be visible to the player (or at least to the developer). Opaque simulations are impossible to debug and frustrating to players.

4. **Iterate on the simulation, not the content.** Build systems that generate content procedurally. Invest in simulation depth rather than hand-crafted scenarios.

**CivLab contract:** These principles should be adopted as design axioms:
- All simulation parameters must be exposed in the scenario editor and debug overlay.
- No scripted macro-events; all events emerge from agent-level or district-level rule execution.
- Prefer simulation depth (more interacting systems) over content breadth (more hand-crafted scenarios).
- Use real-world references (academic models below) for parameter calibration.

---

### 3. OpenTTD -- Transport Network Simulation

**Source:** OpenTTD Wiki (wiki.openttd.org), OpenTTD source code (GitHub, C++).

#### 3.1 Pathfinding Architecture

OpenTTD has evolved through four pathfinding systems:
1. **OPF (Old Pathfinder):** Removed due to bugs.
2. **NTP (New Train Pathfinding):** Basic A* for trains only.
3. **NPF (New Global Pathfinding):** A* for all vehicle types. Correct but slow for large maps.
4. **YAPF (Yet Another Pathfinder):** Current default. Optimized A* with caching, templated C++ for type-specific cost functions.

**YAPF design patterns:**
- **A* with infrastructure-aware cost function.** The cost of traversing a tile includes: distance, slope penalty, curve penalty, signal penalty, station penalty, depot penalty, and infrastructure maintenance cost.
- **Penalty table:** Configurable penalties per obstacle type. Example penalties (from OpenTTD settings):
  - Rail station penalty: configurable (default varies by pathfinder)
  - Slope penalty: higher cost for uphill traversal
  - Curve penalty: cost for changing direction (discourages zigzag routes)
  - Signal penalty: cost for wrong-way signals or red signals
  - Depot reverse penalty: cost for reversing in a depot

- **Segment caching:** YAPF caches the cost of previously-computed path segments to avoid recomputation. Cache invalidation on infrastructure change (track built/demolished).

**Route profitability (simplified):**
```
profit = revenue_per_unit * units_transported -
         distance_cost * distance -
         vehicle_running_cost * time -
         infrastructure_maintenance * route_tiles
```

Revenue per unit depends on: cargo type, distance transported, and time in transit (cargo loses value the longer it takes to deliver, modeled as a decay curve).

**CivLab contract:** CivLab's trade route system (CIV-0205) should use A* pathfinding with a multi-factor cost function. Tunable parameters:
- `terrain_cost_table`: dict mapping terrain type -> traversal cost
- `slope_penalty`: float (cost multiplier for elevation change)
- `infrastructure_bonus`: float (roads/rails reduce traversal cost by this factor)
- `cargo_time_decay_rate`: float (how fast cargo value decays with transit time)
- `route_maintenance_cost_per_tile`: float

---

### Part II: Academic Simulation Models

---

### 4. Epstein & Axtell (1996) -- Sugarscape: Growing Artificial Societies

**Citation:** Epstein, J.M. & Axtell, R.L. (1996). *Growing Artificial Societies: Social Science from the Bottom Up.* MIT Press. Part of the 2050 Project (Santa Fe Institute + World Resources Institute + Brookings Institution).

#### Model Description

Sugarscape is an agent-based model on a 2D grid with a single renewable resource ("sugar," later extended with "spice"):

- **Agents** are born with: vision range (how far they can see resources), metabolism (how much sugar they consume per tick), speed, and initial sugar endowment.
- **Resource landscape:** Sugar grows back at a configurable regrowth rate in fixed geographic patterns (two "sugar mountains" at opposite corners).
- **Agent rules:** Each tick, agents look in their vision range, move to the richest unoccupied cell, and consume their metabolism amount. If sugar reserves hit 0, the agent dies.
- **Emergence:** Wealth inequality emerges naturally from heterogeneous vision/metabolism. Migration waves follow resource depletion. When a second resource (spice) is introduced and agents can trade, an economic market emerges with price discovery.

**Key extensions through chapters:**
1. Basic movement + resource consumption -> wealth distribution
2. Reproduction (sexual, genetic inheritance of vision/metabolism) -> population dynamics
3. Cultural transmission (tag copying between neighbors) -> cultural clustering
4. Combat (agents can attack neighbors for resources) -> territorial behavior
5. Trade (agents exchange sugar for spice at bilateral prices) -> market emergence
6. Disease transmission (SIR-like model between neighboring agents) -> epidemic dynamics

#### CivLab Design Contracts Derived

| Sugarscape Concept | CivLab System | Contract |
|-------------------|---------------|----------|
| Heterogeneous resource landscape | Resource distribution (CIV-0103) | Resources must be geographically concentrated, not uniform. At least 2 distinct resource types with non-overlapping peaks. |
| Agent vision range | Tech-level scouting range (CIV-0301) | Higher technology increases the effective "vision" of the nation AI when evaluating expansion/settlement targets. |
| Resource regrowth rate | Renewable resource model (CIV-0104) | All renewable resources (food, timber) have a configurable regrowth rate per cell. Over-extraction should be possible (depleting faster than regrowth). |
| Wealth inequality emergence | Income distribution (CIV-0202) | The economy should produce Gini coefficient > 0 without explicit inequality rules. Inequality should emerge from agent heterogeneity and resource geography. |
| Trade price discovery | Market price model (CIV-0201) | Bilateral trade prices should emerge from supply/demand ratios, not be fixed. The V3-style market model implements this. |
| Cultural tag transmission | Ideology diffusion (CIV-0106) | Cultural/ideology values should propagate between neighboring cells via contact. See Schelling model below for the homophily coefficient. |

**Tunable parameters:**
- `resource_regrowth_rate`: float per cell type per tick
- `resource_peak_concentration`: float (how concentrated resources are at peak cells vs background)
- `agent_vision_by_tech_level`: dict mapping tech tier -> scouting range in cells

---

### 5. Schelling (1971) -- Segregation and Neighborhood Homophily

**Citation:** Schelling, T.C. (1971). "Dynamic Models of Segregation." *Journal of Mathematical Sociology*, 1(2), 143-186.

#### Model Description

Schelling's segregation model demonstrates that mild individual preferences for similar neighbors produce strong aggregate segregation:

- **Grid:** 2D grid, each cell occupied by one of two agent types (or empty).
- **Satisfaction rule:** An agent is "satisfied" if at least a fraction *t* of its 8 neighbors (Moore neighborhood) are of the same type.
- **Movement rule:** Unsatisfied agents relocate to a random empty cell.
- **Key finding:** Even with *t* as low as 0.30 (agents tolerate up to 70% different neighbors), the grid quickly segregates into large homogeneous clusters. The segregation outcome far exceeds what individual preferences would predict.

**Mathematical formulation:**

```
For agent at position (x,y) with type A:
    neighbors = cells in Moore neighborhood (8 adjacent cells)
    same_type_count = count of neighbors with type A
    total_neighbor_count = count of occupied neighbors

    similarity_ratio = same_type_count / total_neighbor_count

    satisfied = similarity_ratio >= t

    if not satisfied:
        relocate to random empty cell
```

**Parameter sensitivity:**
- *t* = 0.30: mild clustering, some mixing
- *t* = 0.50: strong segregation, clear boundaries
- *t* = 0.75: extreme segregation, nearly zero mixing
- *t* = 1.00: complete segregation (agents only happy surrounded by identical type)

**Extensions (modern research):**
- Continuous homophily preferences (not binary satisfied/unsatisfied)
- Multiple agent types (not just 2)
- Heterogeneous thresholds (different agents have different *t*)
- Network-based (not just grid neighborhoods)

#### CivLab Design Contract: Ideology Neighborhood Diffusion

CivLab's ideology system (CIV-0106) models how ideological positions spread between neighboring districts. The Schelling model directly applies:

**Contract:**
```
ideology_homophily_coefficient: float  # analogous to Schelling's 't'
    Range: [0.0, 1.0]
    Default: 0.35
    Must be tunable per scenario.

    If a district's ideology differs from > (1 - homophily_coefficient)
    fraction of its neighbors, the district experiences:
    1. Ideological pressure (drift toward neighbor majority)
    2. Internal friction (reduced stability)
    3. Migration pressure (population movement toward ideologically aligned districts)

    Higher coefficient = stronger clustering tendency = more ideological
    balkanization. Lower coefficient = more mixing = more ideological diversity.
```

**R_0 for ideology spread (from SIR model, Section 7 below):**
CivLab's `R0_civic` formula should be calibrated against Schelling dynamics. If `R0_civic > 1`, the ideology spreads; if `R0_civic < 1`, it fades. The Schelling coefficient determines the "contact rate" in the SIR analogy: higher homophily = higher effective contact rate = higher R0.

---

### 6. SIR Compartmental Model -- Epidemic / Ideology Diffusion

**Citation:** Kermack, W.O. & McKendrick, A.G. (1927). "A Contribution to the Mathematical Theory of Epidemics." *Proceedings of the Royal Society A*, 115(772), 700-721.

#### Model Description

The SIR model divides a population into three compartments:
- **S (Susceptible):** Not yet exposed.
- **I (Infected/Adopting):** Currently spreading the ideology/disease.
- **R (Recovered/Committed):** No longer actively spreading (either immune or fully committed).

**Differential equations:**

```
dS/dt = -beta * S * I / N
dI/dt = beta * S * I / N - gamma * I
dR/dt = gamma * I

where:
    N = S + I + R (total population)
    beta = transmission rate (contact rate * transmission probability per contact)
    gamma = recovery rate (1 / duration of infectious/active-spreading period)
    R0 = beta / gamma (basic reproduction number)
```

**R0 interpretation:**
- R0 > 1: epidemic grows (ideology spreads)
- R0 = 1: endemic equilibrium
- R0 < 1: epidemic dies out (ideology fades)

#### CivLab Application: R0_civic Formula

CivLab's ideology diffusion (CIV-0106) uses a SIR-inspired model where:
- **S** = population not yet exposed to an ideology
- **I** = population actively proselytizing (recently converted, enthusiastic)
- **R** = population committed but no longer actively spreading (long-term adherents)

**R0_civic formula:**

```
R0_civic = (contact_rate * conversion_probability) / fade_rate

where:
    contact_rate = f(population_density, communication_technology, schelling_homophily)
    conversion_probability = f(ideology_appeal, current_satisfaction, propaganda_investment)
    fade_rate = f(ideology_stability, counter_propaganda, time_since_conversion)
```

**Validation against epidemiology:**
- Real-world R0 values: measles ~12-18, influenza ~1.5-2.0, COVID-19 ~2.5-3.5.
- CivLab ideology R0 should range: 0.5 (fringe ideology, barely spreads) to 5.0 (revolutionary ideology in crisis conditions).
- At R0_civic > 3.0, ideology spreads explosively (revolution scenario).
- At R0_civic ~1.0, ideology reaches endemic equilibrium (stable minority).

**Tunable parameters:**
- `base_contact_rate`: float (how many neighbors a district influences per tick)
- `technology_contact_multiplier`: float per tech level (printing press, radio, internet each increase contact rate)
- `ideology_appeal_by_satisfaction`: piecewise linear curve (low satisfaction -> high appeal for revolutionary ideologies)
- `fade_rate_base`: float (how fast active spreading decays)
- `counter_propaganda_effectiveness`: float (government investment reduces R0_civic)

---

### 7. Hammond & Axelrod (2006) -- Evolution of Ethnocentrism

**Citation:** Hammond, R.A. & Axelrod, R. (2006). "The Evolution of Ethnocentrism." *Journal of Conflict Resolution*, 50(6), 926-936.

#### Model Description

An evolutionary agent-based model studying in-group favoritism:

- **Grid:** 2D toroidal grid.
- **Agents:** Each has an arbitrary "tag" (one of 4+ possible values) and two behavioral genes:
  - Cooperate with same-tag agents? (yes/no)
  - Cooperate with different-tag agents? (yes/no)
- This produces 4 strategy types:
  - **Ethnocentric:** Cooperate with same, defect with different.
  - **Humanitarian:** Cooperate with all.
  - **Selfish:** Defect with all.
  - **Traitorous:** Cooperate with different, defect with same.

- **Interaction:** Agents play one-shot Prisoner's Dilemma with all neighbors.
- **Reproduction:** Agents with positive payoff reproduce (clone with mutation) into adjacent empty cells.
- **Death:** Random death probability per tick.

**Key result:** After transient period, population distribution stabilizes at:
- Ethnocentric: ~75%
- Humanitarian: ~15%
- Selfish: ~8%
- Traitorous: ~2%

This is robust across parameter variations (doubling/halving lattice size, cycle count, tag count, cooperation cost).

**Mechanism:** Ethnocentrics form cooperating clusters that out-compete free-riders (selfish agents) in neighboring territory. Humanitarians survive as second-most-common because they also cooperate within clusters but waste cooperation on out-group defectors.

#### CivLab Design Contract: Faction Dynamics

| Hammond-Axelrod Concept | CivLab System | Contract |
|------------------------|---------------|----------|
| Tag-based cooperation | Faction alliance tendency | Nations sharing cultural/ideological tags should have higher cooperation probability. |
| Ethnocentric dominance | Default diplomatic posture | Without player intervention, AI nations should tend toward ethnocentric behavior (cooperate with similar, defect with different). |
| Humanitarian minority | Diplomatic AI variation | ~15% of AI nations should exhibit humanitarian behavior (cooperate broadly), creating natural alliance partners for diverse player strategies. |
| Selfish minority | Aggressive AI nations | ~8% of AI nations should be aggressive toward all (defect universally). |
| Cluster competition | Border dynamics | Cooperative faction clusters should expand at the expense of isolated selfish nations. |
| Tag mutation | Cultural drift | Over time, cultural tags should mutate, creating new faction alignments. |

**Tunable parameters:**
- `ethnocentric_tendency_weight`: float (how strongly cultural similarity affects cooperation decisions)
- `faction_strategy_distribution`: dict mapping strategy -> initial probability (default: {ethnocentric: 0.75, humanitarian: 0.15, selfish: 0.08, traitorous: 0.02})
- `cultural_mutation_rate`: float per tick (probability of tag change)
- `cooperation_cost`: float (cost paid by cooperator, benefit received by partner -- Prisoner's Dilemma payoff matrix)
- `cooperation_benefit_ratio`: float (ratio of partner's benefit to cooperator's cost; default ~3.0 following Axelrod)

---

### Part III: Cross-Reference and Synthesis

---

### 8. Annotated Bibliography

| # | Citation | Year | Key Contribution | CivLab System |
|---|----------|------|-----------------|---------------|
| 1 | Epstein & Axtell, *Growing Artificial Societies* | 1996 | Sugarscape: resource heterogeneity, trade emergence, wealth inequality | CIV-0103 (resources), CIV-0201 (market), CIV-0202 (inequality) |
| 2 | Schelling, "Dynamic Models of Segregation" | 1971 | Neighborhood homophily -> macro segregation | CIV-0106 (ideology diffusion) |
| 3 | Kermack & McKendrick, "Mathematical Theory of Epidemics" | 1927 | SIR compartmental model, R0 | CIV-0106 (R0_civic formula) |
| 4 | Hammond & Axelrod, "Evolution of Ethnocentrism" | 2006 | Tag-based cooperation, faction dynamics emergence | CIV-0301 (nation AI), CIV-0302 (diplomacy) |
| 5 | Adams, "Simulation Principles from Dwarf Fortress" | 2015 | 4 design principles: reality-based, emergent, inspectable, system-over-content | All CivLab systems (design axioms) |
| 6 | Paradox Interactive, Victoria 3 Dev Diaries | 2021-25 | Pop system, market system, utility-based AI | CIV-0102 (satisfaction), CIV-0201 (market), CIV-0301 (AI) |
| 7 | Bay 12 Games, Dwarf Fortress Wiki | 2006-26 | Need satisfaction, stress system, focus formula | CIV-0102 (district satisfaction), CIV-0105 (morale) |
| 8 | OpenTTD Project, Source Code and Wiki | 2004-26 | YAPF pathfinding, route profitability, infrastructure cost | CIV-0205 (trade routes) |
| 9 | Andersson, "Deep Dive: Modeling the global economy in Victoria 3" | 2022 | Closed-market equilibrium, buy/sell orders, substitution | CIV-0201 (market) |
| 10 | Hatna & Benenson, "The Schelling Model of Ethnic Residential Dynamics" | 2012 | Extended Schelling model beyond binary segregation patterns | CIV-0106 (multi-ideology diffusion) |

---

### 9. CivLab Design Contracts Summary

#### Contract 1: Resource Geography (Sugarscape-derived)

```yaml
contract_id: SIM-C001
source: Epstein & Axtell 1996 (Sugarscape)
civlab_system: CIV-0103
requirement: >
  Resource distribution must be geographically concentrated, not uniform.
  At least 2 distinct resource types with non-overlapping peak regions.
  Over-extraction (consumption > regrowth) must be possible.
parameters:
  resource_regrowth_rate:
    type: float
    range: [0.001, 1.0]
    per: cell_type
    description: "Fraction of max capacity regrown per tick"
  resource_peak_concentration:
    type: float
    range: [1.0, 20.0]
    description: "Ratio of peak cell yield to background cell yield"
```

#### Contract 2: Market Price Discovery (V3-derived)

```yaml
contract_id: SIM-C002
source: Victoria 3 market system
civlab_system: CIV-0201
requirement: >
  Goods prices emerge from aggregate buy/sell orders.
  Price range: base_price * [1 - price_elasticity, 1 + price_elasticity].
  Market clearing rations goods proportionally when supply < demand.
  Substitution reduces demand for expensive goods.
parameters:
  price_elasticity_range:
    type: float
    default: 0.75
  oversupply_floor_ratio:
    type: float
    default: 0.5
  substitution_coefficient:
    type: float
    per: good_pair
    range: [0.0, 1.0]
```

#### Contract 3: Ideology Neighborhood Diffusion (Schelling + SIR)

```yaml
contract_id: SIM-C003
source: Schelling 1971 + Kermack-McKendrick 1927
civlab_system: CIV-0106
requirement: >
  Ideology spreads between neighboring districts via SIR dynamics.
  R0_civic determines whether ideology grows or fades.
  Schelling homophily coefficient controls contact rate.
  Must be tunable per scenario.
parameters:
  ideology_homophily_coefficient:
    type: float
    default: 0.35
    range: [0.0, 1.0]
    description: "Schelling 't' -- higher = stronger clustering"
  r0_civic_range:
    type: [float, float]
    default: [0.5, 5.0]
    description: "Achievable R0 range for ideology spread"
  base_contact_rate:
    type: float
    default: 2.0
  technology_contact_multiplier:
    type: dict
    description: "tech_tier -> multiplier (e.g., printing_press: 1.5, radio: 2.0, internet: 3.0)"
```

#### Contract 4: Faction Cooperation Dynamics (Hammond-Axelrod-derived)

```yaml
contract_id: SIM-C004
source: Hammond & Axelrod 2006
civlab_system: CIV-0301, CIV-0302
requirement: >
  AI nation diplomacy uses tag-based cooperation.
  Without player intervention, ethnocentric behavior should dominate (~75%).
  Cultural similarity increases cooperation probability.
  Cultural tags mutate over time, creating new alignments.
parameters:
  ethnocentric_tendency_weight:
    type: float
    default: 0.75
    range: [0.0, 1.0]
  faction_strategy_initial_distribution:
    type: dict
    default:
      ethnocentric: 0.75
      humanitarian: 0.15
      selfish: 0.08
      traitorous: 0.02
  cultural_mutation_rate:
    type: float
    default: 0.01
    description: "Probability of cultural tag change per tick per nation"
  cooperation_benefit_ratio:
    type: float
    default: 3.0
    description: "Prisoner's Dilemma: benefit/cost ratio"
```

#### Contract 5: District Satisfaction and Morale (DF-derived)

```yaml
contract_id: SIM-C005
source: Dwarf Fortress need/stress system
civlab_system: CIV-0102, CIV-0105
requirement: >
  District satisfaction is a composite of weighted need fulfillment scores.
  Two timescales: short-term mood (volatile) and long-term stability (slow).
  Focus/productivity modifier derived from satisfaction.
  Stability collapse triggers crisis cascade.
parameters:
  need_decay_rate:
    type: float
    default: 0.01
    description: "Per-tick decay for unsatisfied needs"
  focus_to_productivity_curve:
    type: piecewise_linear
    default: [[0.6, -0.5], [0.8, -0.1], [1.0, 0.0], [1.2, 0.1], [1.4, 0.5]]
    description: "[focus_ratio, productivity_modifier] pairs"
  stability_recovery_rate:
    type: float
    default: 0.005
    description: "Max long-term stability recovery per tick under good mood"
  stability_collapse_threshold:
    type: float
    default: -0.5
    description: "Normalized stability below which crisis cascade triggers"
```

#### Contract 6: Trade Route Pathfinding (OpenTTD-derived)

```yaml
contract_id: SIM-C006
source: OpenTTD YAPF pathfinding
civlab_system: CIV-0205
requirement: >
  Trade routes use A* pathfinding with infrastructure-aware cost function.
  Cost includes: distance, terrain penalty, slope penalty, infrastructure bonus.
  Route profitability considers cargo value decay over transit time.
parameters:
  terrain_cost_table:
    type: dict
    description: "terrain_type -> base traversal cost"
    default:
      plains: 1.0
      forest: 1.5
      hills: 2.0
      mountains: 4.0
      water: 3.0
      desert: 2.5
  slope_penalty:
    type: float
    default: 1.5
    description: "Multiplier for elevation change per cell"
  infrastructure_bonus:
    type: dict
    default:
      road: 0.5
      railroad: 0.25
      highway: 0.3
    description: "Multiplier applied to base terrain cost when infrastructure exists"
  cargo_time_decay_rate:
    type: float
    default: 0.02
    description: "Fraction of cargo value lost per tick of transit time"
```

---

## Decision

1. **Adopt V3-style market model** for CivLab economy (buy/sell orders, price elasticity, substitution).
2. **Adopt DF-inspired dual-timescale satisfaction model** for district welfare (mood + stability).
3. **Implement Schelling homophily coefficient** as the core parameter for ideology neighborhood diffusion.
4. **Use SIR-derived R0_civic** for ideology spread dynamics, calibrated to range [0.5, 5.0].
5. **Apply Hammond-Axelrod faction strategy distribution** as default AI diplomatic posture distribution.
6. **Use YAPF-inspired A* with infrastructure costs** for trade route pathfinding.
7. **All parameters must be tunable per scenario** and exposed in the scenario editor.
8. **Adams' four principles** (reality-based, emergent, inspectable, system-over-content) adopted as design axioms.

---

## Open Questions Remaining

1. **Pop granularity:** V3 uses culture/profession/state aggregation. CivLab currently uses district-level aggregation. Should CivLab model individual profession groups within districts (finer granularity, closer to V3), or keep district-level aggregation (simpler, sufficient for MVP)? Recommend district-level for MVP, profession-groups as post-MVP enhancement.

2. **Multi-ideology Schelling:** The classic Schelling model uses 2 types. CivLab has 4+ ideologies. Hatna & Benenson (2012) extended Schelling to multi-type settings and found that 3+ types produce more complex boundary patterns. The homophily coefficient may need to be a matrix (per ideology-pair), not a scalar.

3. **Market simulation tick rate:** V3 runs its market simulation daily (in-game). CivLab's tick rate is not yet finalized. The market model's stability depends on tick rate -- too infrequent causes oscillations, too frequent is computationally expensive. Recommend matching the main simulation tick rate and testing for oscillation.

4. **Sugarscape wealth distribution calibration:** The Gini coefficient that emerges from CivLab's economy should be compared against real-world reference values (e.g., USA ~0.40, Nordic countries ~0.27, pre-industrial societies ~0.45). If CivLab produces unrealistic inequality levels, adjust `resource_peak_concentration` and `substitution_coefficient`.

5. **OpenTTD pathfinding scale:** OpenTTD's YAPF works on grids of ~1000x1000 tiles. CivLab's hex grid may be larger. If A* performance is insufficient, consider hierarchical A* (HPA*) or JPS+ for the hex grid. This is an implementation concern, not a design concern.

---

## Sources

### Game References

- Victoria 3 Dev Diary #1 (Pops): https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-1-pops.1476573/
- Victoria 3 Market Wiki: https://vic3.paradoxwikis.com/Market
- Deep Dive: Modeling the global economy in Victoria 3: https://www.gamedeveloper.com/design/deep-dive-modeling-the-global-economy-in-victoria-3
- Victoria 3 Dev Diary #37 (Market Expansion): https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-37-market-expansion
- Dwarf Fortress Wiki - Need system: https://dwarffortresswiki.org/index.php/DF2014:Need
- Dwarf Fortress Wiki - Stress system: https://dwarffortresswiki.org/Stress
- Dwarf Fortress Wiki - Keeping dwarves unstressed: https://dwarffortresswiki.org/index.php/DF2014:Keeping_your_dwarves_unstressed
- Tarn Adams, "Simulation Principles from Dwarf Fortress," Game AI Pro 2, Ch. 41 (2015): http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf
- OpenTTD Pathfinding documentation: https://wiki.openttd.org/en/Archive/Source/OpenTTDDevBlackBook/Simulation/Pathfinding
- OpenTTD YAPF documentation: https://wiki.openttd.org/en/Archive/Manual/Yet%20Another%20Pathfinder

### Academic References

- Epstein, J.M. & Axtell, R.L. (1996). *Growing Artificial Societies: Social Science from the Bottom Up.* MIT Press: https://mitpress.mit.edu/9780262550253/growing-artificial-societies/
- Schelling, T.C. (1971). "Dynamic Models of Segregation." *Journal of Mathematical Sociology*, 1(2): http://nifty.stanford.edu/2014/mccown-schelling-model-segregation/
- Kermack, W.O. & McKendrick, A.G. (1927). "A Contribution to the Mathematical Theory of Epidemics." *Proc. Royal Society A*, 115(772): https://en.wikipedia.org/wiki/Compartmental_models_in_epidemiology
- Hammond, R.A. & Axelrod, R. (2006). "The Evolution of Ethnocentrism." *Journal of Conflict Resolution*, 50(6): https://journals.sagepub.com/doi/10.1177/0022002706293470
- Hatna, E. & Benenson, I. (2012). "The Schelling Model of Ethnic Residential Dynamics." *JASSS*, 15(1): https://jasss.soc.surrey.ac.uk/15/1/6.html
- Mesa Schelling Model implementation: https://mesa.readthedocs.io/latest/examples/basic/schelling.html
- Evolution of ethnocentrism model (CoMSES): https://www.comses.net/codebases/2942/releases/1.1.0/
