# CIV-0400: AI / NPC Behavior Specification v1

**Spec ID:** CIV-0400
**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team
**Related Specs:**
- CIV-0001: Core Simulation Loop (deterministic tick architecture, ChaCha20Rng, WebSocket command interface)
- CIV-0100: Economy v1 (resource flows, allocation regimes, conservation invariants)
- CIV-0103: Institutions, Time-Series, and Citizen Lifecycle (institutional FSM, legitimacy model)
- CIV-0105: War, Diplomacy, and Shadow Networks (diplomatic FSM, treaty system, covert operations)
- CIVLAB_GAME_DESIGN.md (game pillars, victory conditions, difficulty system)
- FUNCTIONAL_REQUIREMENTS.md FR-CIV-RTS-014 (Faction AI Behavior & Decision Making)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [AI Architecture Philosophy](#2-ai-architecture-philosophy)
3. [Nation AI — Strategic Layer](#3-nation-ai--strategic-layer)
4. [Nation AI — Personality Matrix](#4-nation-ai--personality-matrix)
5. [Nation AI — Memory and Learning](#5-nation-ai--memory-and-learning)
6. [Military AI — Operational Layer](#6-military-ai--operational-layer)
7. [Military AI — Tactical Layer](#7-military-ai--tactical-layer)
8. [Diplomatic AI](#8-diplomatic-ai)
9. [City-Level AI — Development Planner](#9-city-level-ai--development-planner)
10. [Difficulty Scaling System](#10-difficulty-scaling-system)
11. [MCTS Implementation (Difficulty 4–5)](#11-mcts-implementation-difficulty-45)
12. [Espionage AI](#12-espionage-ai)
13. [AI Observability and Debugging](#13-ai-observability-and-debugging)
14. [Modding API for AI](#14-modding-api-for-ai)
15. [FR Traceability](#15-fr-traceability)
16. [Acceptance Criteria](#16-acceptance-criteria)

---

## 1. Executive Summary

CivLab requires AI-controlled factions that are simultaneously:

- **Deterministic** — given the same simulation seed and policy bundle, AI nations make identical decisions across all platforms, all runs, and all client configurations. This is non-negotiable because CivLab is a research platform; cross-run reproducibility is a core guarantee.
- **Fair** — AI factions operate exclusively through the same WebSocket command interface as human players. No privileged simulation access. No hidden reads of simulation internals. The AI submits `{"action_type": "declare_war", ...}` just as a player does.
- **Moddable** — every utility weight, personality threshold, and decision heuristic is externally configurable via YAML. Custom AI modules can be registered as Rust trait implementations or Python callback hooks.
- **Scalable** — AI complexity scales with difficulty level. At Novice, the AI runs fast heuristics with shallow memory. At Legendary, it runs bounded MCTS lookahead with full memory depth.
- **Comprehensible** — every AI decision emits an `ai.decision.v1` event with full utility scores for the top five considered options, enabling post-game analysis, debugging, and academic study.

The AI system is organized into three granularity levels that execute at different frequencies:

| Level | Name | Scope | Frequency |
|---|---|---|---|
| Strategic | Nation AI | Whole nation — wars, treaties, research, economy | Every tick |
| Operational | Military AI | Army group — movement, mission selection, supply | Every tick |
| Tactical | Unit AI | Individual unit — combat targeting, formation | Real-time during battle phase |

All three levels are seeded from the same `ChaCha20Rng` instance initialized from the scenario seed, ensuring reproducibility. Neural networks are explicitly excluded from the core AI loop; see Section 2.3 for the full rationale.

---

## 2. AI Architecture Philosophy

### 2.1 Hybrid Rule-Based Utility Scoring with Optional MCTS Lookahead

The core decision mechanism is **utility theory**: for each possible action `a` in the action space `A`, a scalar utility score `U(a, s)` is computed given the current state `s`. The action with the highest utility score is selected. This is fast, predictable, and fully auditable.

```
action* = argmax_{a in A} U(a, s)
```

Utility functions are composed of weighted sub-scores:

```
U(a, s) = Σ_i [ weight_i(personality) × factor_i(a, s) ]
```

Where `weight_i` is drawn from the nation's personality parameter vector (Section 4) and `factor_i` is a normalized factor function that maps a specific aspect of the state to `[-1.0, 1.0]`.

**All factor functions use `i32` arithmetic scaled by 1000** (fixed-point) to preserve determinism. No `f64` appears in any utility computation. Division results are rounded toward zero.

At difficulty levels 4 and 5, a bounded **Monte Carlo Tree Search** layer wraps the utility scorer. Instead of picking the single greedy best action from the current state, MCTS evaluates sequences of actions over a limited lookahead horizon, using the utility scorer as the rollout policy. See Section 11 for full MCTS specification.

### 2.2 Fair Play Guarantee — AI Through the Command Interface

AI factions submit all actions through `AiCommandSubmitter`, which serializes to the same WebSocket JSON format used by human clients:

```rust
pub trait AiCommandSubmitter {
    fn submit(&self, nation_id: EntityId, action: AiAction) -> Result<(), SubmitError>;
}

pub struct WebSocketAiSubmitter {
    endpoint: WebSocketEndpoint,
    rate_governor: ApmGovernor,
}

impl AiCommandSubmitter for WebSocketAiSubmitter {
    fn submit(&self, nation_id: EntityId, action: AiAction) -> Result<(), SubmitError> {
        self.rate_governor.check_and_consume()?;
        let payload = action.to_command_json(nation_id);
        self.endpoint.send(payload)
    }
}
```

The AI reads game state only from the same event stream that human clients receive. It does not call into simulation internals. This means:

- Fog of war applies to the AI exactly as to humans (except at Difficulty 5 where "perfect intel" is disclosed as a named bonus).
- AI actions are subject to the same validation rules as player actions. An invalid action is rejected with the same error code.
- Mods that restrict player actions automatically restrict AI actions.

This constraint also means the AI is naturally moddable via command interception — a mod can intercept `AiAction` before submission, modify it, or replace it entirely.

### 2.3 No Neural Networks in Core AI — Determinism Requirement

Neural network inference is explicitly forbidden in the core AI decision path. Reasons:

1. **Platform non-determinism.** Floating-point operations in neural network forward passes produce bit-different results across CPU architectures, SIMD instruction sets, and BLAS library versions. This breaks the research mode guarantee that the same seed produces identical results on different machines.
2. **Opacity.** Neural network decisions cannot be explained by a human-readable utility score breakdown. The `ai.decision.v1` event would be meaningless.
3. **Non-moddability.** Mod authors cannot override a neural network weight by editing a YAML file.
4. **Complexity cost.** Core AI must run on every tick for every AI nation. Neural inference at that scale requires GPU infrastructure that the headless core cannot assume.

Neural networks MAY be used in experimental modding scenarios where the mod explicitly acknowledges the above limitations, disables determinism verification for the session, and tags the replay as non-reproducible.

### 2.4 Deterministic RNG — ChaCha20Rng Seeding

All AI random decisions (tie-breaking between equal-utility actions, stochastic personality drift, espionage operation outcomes) use a `ChaCha20Rng` seeded from:

```rust
fn ai_rng_for_tick(scenario_seed: u64, nation_id: EntityId, tick: TickNumber) -> ChaCha20Rng {
    let mut hasher = SipHasher13::new();
    hasher.write_u64(scenario_seed);
    hasher.write_u64(nation_id.raw());
    hasher.write_u64(tick.0 as u64);
    // Domain separator to avoid cross-subsystem collisions
    hasher.write_u64(0xAI_DOMAIN_TAG);
    let seed_bytes: [u8; 32] = derive_chacha_seed(hasher.finish());
    ChaCha20Rng::from_seed(seed_bytes)
}
```

This ensures:
- Different nations get different RNG sequences on the same tick.
- The same nation on the same tick always gets the same RNG sequence.
- The AI RNG is isolated from the demographic and economic RNG (different domain tags).

The `0xAI_DOMAIN_TAG` constant is `0xA1_A1_A1_A1_A1_A1_A1_A1u64`.

### 2.5 Action Rate Governor (APM Limiter)

To prevent superhuman micro, AI action rates are capped by an `ApmGovernor`:

```rust
pub struct ApmGovernor {
    max_actions_per_minute: u32,
    token_bucket: TokenBucket,
}

impl ApmGovernor {
    pub fn check_and_consume(&mut self) -> Result<(), RateLimitError> {
        if self.token_bucket.try_consume(1) {
            Ok(())
        } else {
            Err(RateLimitError::BucketEmpty)
        }
    }
}
```

APM limits per difficulty level are specified in Section 10. Actions that are rate-limited are queued and retried on subsequent ticks, not dropped.

---

## 3. Nation AI — Strategic Layer

### 3.1 NationAIState — Core Data Structure

```rust
/// Top-level AI state for a single AI-controlled nation.
/// Persisted across ticks; serializable for save/load and replay annotation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NationAIState {
    /// Owning nation identifier.
    pub nation_id: EntityId,

    /// Stable personality archetype, seeded from nation_id hash at game start.
    /// May drift over time; see Section 4.3.
    pub personality: NationPersonality,

    /// Current personality parameter vector (may differ from archetype defaults
    /// if drift has accumulated).
    pub params: PersonalityParams,

    /// Priority queue of active strategic goals, ordered by priority descending.
    pub goals: BinaryHeap<Reverse<StrategicGoal>>,

    /// Threat assessment per neighbor nation. Updated every tick.
    pub threat_model: BTreeMap<EntityId, ThreatScore>,

    /// Opportunity assessment per neighbor nation. Updated every tick.
    pub opportunity_model: BTreeMap<EntityId, OpportunityScore>,

    /// Rolling memory of past events and outcomes. Bounded by difficulty level.
    pub memory: AIMemory,

    /// Tick of last strategic re-evaluation (full goal recompute).
    pub last_full_eval_tick: TickNumber,

    /// Current difficulty level (copied from scenario config; used to select
    /// planning horizon and memory depth).
    pub difficulty: DifficultyLevel,
}

/// Priority-ordered strategic goal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StrategicGoal {
    ExpandTerritory   { target_district: EntityId, priority: i32 },
    SecureResources   { resource_type: GoodType,   priority: i32 },
    FormAlliance      { target_nation: EntityId,   priority: i32 },
    DevelopEconomy    { sector: EconomicSector,    priority: i32 },
    SuppressInsurgency{ district: EntityId,        priority: i32 },
    ResearchTech      { tech_id: TechId,           priority: i32 },
    NegotiatePeace    { target_nation: EntityId,   priority: i32 },
    BuildMilitary     { unit_type: UnitType,       priority: i32 },
    LaunchEspionage   { target_nation: EntityId,
                        operation: EspionageOperation, priority: i32 },
    ManageCarbon      { target_ppm: i32,           priority: i32 },
}

impl StrategicGoal {
    pub fn priority(&self) -> i32 {
        match self {
            Self::ExpandTerritory    { priority, .. } => *priority,
            Self::SecureResources    { priority, .. } => *priority,
            Self::FormAlliance       { priority, .. } => *priority,
            Self::DevelopEconomy     { priority, .. } => *priority,
            Self::SuppressInsurgency { priority, .. } => *priority,
            Self::ResearchTech       { priority, .. } => *priority,
            Self::NegotiatePeace     { priority, .. } => *priority,
            Self::BuildMilitary      { priority, .. } => *priority,
            Self::LaunchEspionage    { priority, .. } => *priority,
            Self::ManageCarbon       { priority, .. } => *priority,
        }
    }
}
```

### 3.2 Strategic Evaluation Cycle

The strategic layer runs once per tick for each AI nation. The evaluation cycle:

```
1. UPDATE_OBSERVATIONS     — read latest event stream snapshot
2. UPDATE_THREAT_MODEL     — recompute ThreatScore for each neighbor
3. UPDATE_OPPORTUNITY_MODEL— recompute OpportunityScore for each neighbor
4. RECOMPUTE_GOALS         — rebuild goal priority queue from observations
   (full recompute every N ticks; incremental otherwise; N = planning_horizon)
5. ENUMERATE_ACTIONS       — generate candidate actions from top-3 goals
6. SCORE_ACTIONS           — apply utility functions to each candidate action
7. APPLY_APM_GOVERNOR      — filter to rate-limited budget for this tick
8. SUBMIT_ACTIONS          — submit surviving actions via AiCommandSubmitter
9. EMIT_DECISION_EVENT     — emit ai.decision.v1 with full score breakdown
10. UPDATE_MEMORY          — append tick observations to AIMemory
```

Steps 1–4 are read-only (no side effects). Steps 5–8 produce actions. Steps 9–10 are observability and bookkeeping.

### 3.3 Threat Scoring

```rust
/// ThreatScore represents how much nation `other` threatens the AI nation.
/// All fields are i32 in range [0, 1000] (fixed-point, divide by 1000 for [0.0, 1.0]).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThreatScore {
    pub military_ratio:      i32,  // their_strength / our_strength * 1000, clamped [0,2000]
    pub territorial_pressure:i32,  // adjacent_districts_controlled_by_them / total_border_districts * 1000
    pub diplomatic_hostility:i32,  // derived from DiplomaticState (ActiveConflict=1000, Alliance=0)
    pub espionage_exposure:  i32,  // detected_ops_last_50_ticks / 50 * 1000
    pub composite:           i32,  // weighted sum, see formula below
}

/// Composite threat formula (all factors fixed-point /1000):
///
///   composite = (
///       military_ratio      × WEIGHT_MILITARY       +
///       territorial_pressure× WEIGHT_TERRITORY      +
///       diplomatic_hostility× WEIGHT_HOSTILITY      +
///       espionage_exposure  × WEIGHT_ESPIONAGE
///   ) / 1000
///
/// Default weights (moddable via personality.yaml):
///   WEIGHT_MILITARY  = 500
///   WEIGHT_TERRITORY = 250
///   WEIGHT_HOSTILITY = 200
///   WEIGHT_ESPIONAGE = 50
///
/// composite range: [0, 1000]
/// threat is "high" if composite > 600
/// threat is "critical" if composite > 850

fn compute_threat_score(
    ai_state: &NationAIState,
    other: EntityId,
    game_state: &GameStateSnapshot,
) -> ThreatScore {
    let military_ratio = {
        let our = game_state.military_strength(ai_state.nation_id);
        let their = game_state.military_strength(other);
        if our == 0 { 1000 } else { (their * 1000 / our).min(2000) }
    };
    let territorial_pressure = {
        let border_districts = game_state.border_districts(ai_state.nation_id, other);
        let theirs = border_districts.iter().filter(|d| d.controller == other).count() as i32;
        let total = border_districts.len().max(1) as i32;
        theirs * 1000 / total
    };
    let diplomatic_hostility = game_state
        .diplomatic_state(ai_state.nation_id, other)
        .threat_contribution_milli();
    let espionage_exposure = {
        let ops = ai_state.memory.detected_espionage_ops(other, 50);
        (ops as i32 * 1000 / 50).min(1000)
    };
    let p = &ai_state.params;
    let composite = (
        military_ratio       * p.threat_weight_military   +
        territorial_pressure * p.threat_weight_territory  +
        diplomatic_hostility * p.threat_weight_hostility  +
        espionage_exposure   * p.threat_weight_espionage
    ) / 1000;
    ThreatScore { military_ratio, territorial_pressure, diplomatic_hostility,
                  espionage_exposure, composite }
}
```

### 3.4 Utility Functions — Full Specifications

Each utility function returns an `i32` in `[-1000, 1000]` representing the desirability of the action. Positive means "do this", negative means "avoid this".

#### 3.4.1 `declare_war_utility`

```
declare_war_utility(target) =
    clamp(
        threat_score(target).composite           × W_threat          +
        territorial_gain_estimate(target)         × W_territory       +
        strength_ratio_advantage(target)          × W_strength        +
        diplomatic_cost_penalty(target)           × W_diplo_cost      +
        timing_modifier(target)                   × W_timing          +
        personality_war_bias(personality)         × W_personality,
        -1000, 1000
    )
```

Factor definitions:

| Factor | Formula | Range |
|---|---|---|
| `threat_score(target).composite` | Section 3.3 composite | [0, 1000] |
| `territorial_gain_estimate` | districts_takeable × avg_district_value / max_value × 1000 | [0, 1000] |
| `strength_ratio_advantage` | (our_strength - their_strength) / max(our_strength, 1) × 1000 | [-1000, 1000] |
| `diplomatic_cost_penalty` | −(alliance_retaliation_count × 200 + global_reputation_loss × 100) | [-1000, 0] |
| `timing_modifier` | +300 if they_are_at_war_elsewhere; +200 if their_supply_low; −200 if we_are_at_war | [-500, 500] |
| `personality_war_bias` | from personality params table (Section 4.1) | [-500, 500] |

Default weights: `W_threat=300, W_territory=200, W_strength=250, W_diplo_cost=150, W_timing=50, W_personality=50`. Sum = 1000.

**War is declared only if** `declare_war_utility(target) > war_utility_threshold` AND `casus_belli_valid(target)`. The threshold is `params.war_utility_threshold` (personality-dependent; default 600).

#### 3.4.2 `sign_treaty_utility`

```
sign_treaty_utility(partner, treaty_type) =
    clamp(
        relationship_score(partner)               × W_relationship    +
        economic_benefit_estimate(partner, treaty)× W_economic        +
        security_benefit_estimate(partner, treaty)× W_security        +
        sovereignty_cost_penalty(treaty)          × W_sovereignty     +
        personality_alliance_bias(personality)    × W_personality,
        -1000, 1000
    )
```

Factor definitions:

| Factor | Formula | Range |
|---|---|---|
| `relationship_score(partner)` | diplomatic_relation.score / 1000 × 1000 (pass-through) | [-1000, 1000] |
| `economic_benefit_estimate` | projected_trade_gain_per_tick × planning_horizon / max_economic_value × 1000 | [0, 1000] |
| `security_benefit_estimate` | partner_military_strength / (total_threat_strength + 1) × 1000 | [0, 1000] |
| `sovereignty_cost_penalty` | −treaty_obligation_count × 150 | [-1000, 0] |
| `personality_alliance_bias` | from personality params (Section 4.1) | [-500, 500] |

Default weights: `W_relationship=250, W_economic=300, W_security=250, W_sovereignty=150, W_personality=50`.

**Treaty is offered when** `sign_treaty_utility > treaty_offer_threshold` (default 400) AND no active conflict with partner.

#### 3.4.3 `build_structure_utility`

```
build_structure_utility(district, structure_type) =
    clamp(
        production_gain_normalized(district, structure_type)   × W_production  +
        defense_value_normalized(structure_type)               × W_defense      +
        citizen_happiness_gain_normalized(structure_type)      × W_happiness    +
        joule_cost_penalty(structure_type, current_reserves)   × W_joule_cost   +
        urgency_modifier(district)                             × W_urgency,
        -1000, 1000
    )
```

Factor definitions:

| Factor | Formula | Range |
|---|---|---|
| `production_gain_normalized` | (output_per_tick × planning_horizon) / max_possible_gain × 1000 | [0, 1000] |
| `defense_value_normalized` | structure_defense_rating / max_defense_rating × 1000 | [0, 1000] |
| `citizen_happiness_gain_normalized` | happiness_delta_per_citizen × pop / max_happiness_impact × 1000 | [-500, 1000] |
| `joule_cost_penalty` | −joule_cost / current_joule_reserves × 1000, clamped [-1000, 0] | [-1000, 0] |
| `urgency_modifier` | +500 if district_under_threat; +200 if happiness \< 40; +300 if supply_critical | [0, 500] |

Default weights: `W_production=350, W_defense=200, W_happiness=200, W_joule_cost=150, W_urgency=100`.

#### 3.4.4 `set_policy_utility`

```
set_policy_utility(policy) =
    clamp(
        legitimacy_effect(policy)    × W_legitimacy    +
        economic_effect(policy)      × W_economic      +
        happiness_effect(policy)     × W_happiness     +
        implementation_risk(policy)  × W_risk,
        -1000, 1000
    )
```

| Factor | Formula | Range |
|---|---|---|
| `legitimacy_effect` | predicted_legitimacy_delta / 1.0 × 1000 | [-1000, 1000] |
| `economic_effect` | predicted_gdp_delta_per_tick × planning_horizon / max_gdp × 1000 | [-1000, 1000] |
| `happiness_effect` | predicted_happiness_delta × population / max_happiness_impact × 1000 | [-1000, 1000] |
| `implementation_risk` | −policy.failure_probability × severity_factor × 1000 | [-1000, 0] |

The AI will not enact a policy if `implementation_risk < −700` unless the `params.risk_tolerance > 700`.

Default weights: `W_legitimacy=300, W_economic=350, W_happiness=250, W_risk=100`.

#### 3.4.5 `launch_espionage_utility`

```
launch_espionage_utility(target, operation) =
    clamp(
        intelligence_gain_normalized(operation)  × W_intel          +
        disruption_value_normalized(operation)   × W_disruption     +
        detection_risk_penalty(operation, target)× W_detection      +
        joule_cost_penalty(operation)            × W_cost,
        -1000, 1000
    )
```

| Factor | Formula | Range |
|---|---|---|
| `intelligence_gain_normalized` | bits_of_intel × operation_reliability / max_intel_value × 1000 | [0, 1000] |
| `disruption_value_normalized` | disruption_magnitude / max_disruption × 1000 | [0, 1000] |
| `detection_risk_penalty` | −detection_probability × consequence_severity × 1000 | [-1000, 0] |
| `joule_cost_penalty` | −joule_cost / current_reserves × 1000 | [-1000, 0] |

Default weights: `W_intel=300, W_disruption=350, W_detection=250, W_cost=100`.

### 3.5 Goal Recomputation Algorithm

Full goal recomputation runs every `planning_horizon` ticks. Incremental updates run every tick. The algorithm:

```rust
fn recompute_goals(state: &NationAIState, snapshot: &GameStateSnapshot) -> BinaryHeap<Reverse<StrategicGoal>> {
    let mut goals = BinaryHeap::new();

    // 1. Existential threats first
    for (nation_id, threat) in &state.threat_model {
        if threat.composite > CRITICAL_THREAT_THRESHOLD {
            // Immediately prioritize military buildup or peace negotiation
            if state.params.war_aggression_bias > 600 {
                goals.push(Reverse(StrategicGoal::BuildMilitary {
                    unit_type: best_counter_unit(snapshot, *nation_id),
                    priority: 1000,
                }));
            } else {
                goals.push(Reverse(StrategicGoal::NegotiatePeace {
                    target_nation: *nation_id,
                    priority: 900,
                }));
            }
        }
    }

    // 2. Resource scarcity
    for good_type in GoodType::all() {
        let scarcity = snapshot.supply_stress(state.nation_id, good_type);
        if scarcity > SCARCITY_ALERT_THRESHOLD {
            goals.push(Reverse(StrategicGoal::SecureResources {
                resource_type: good_type,
                priority: 500 + (scarcity - SCARCITY_ALERT_THRESHOLD).min(500),
            }));
        }
    }

    // 3. Personality-driven expansion
    if state.params.expansion_bias > 500 {
        if let Some(best_target) = find_best_expansion_target(state, snapshot) {
            goals.push(Reverse(StrategicGoal::ExpandTerritory {
                target_district: best_target,
                priority: state.params.expansion_bias / 2,
            }));
        }
    }

    // 4. Happiness / legitimacy maintenance
    let happiness = snapshot.avg_citizen_happiness(state.nation_id);
    if happiness < HAPPINESS_ALERT_THRESHOLD {
        goals.push(Reverse(StrategicGoal::DevelopEconomy {
            sector: EconomicSector::ConsumerGoods,
            priority: (HAPPINESS_ALERT_THRESHOLD - happiness) * 5,
        }));
    }

    // 5. Research if tech gap detected
    if let Some(priority_tech) = identify_tech_gap(state, snapshot) {
        goals.push(Reverse(StrategicGoal::ResearchTech {
            tech_id: priority_tech,
            priority: 300,
        }));
    }

    goals
}
```

Constants: `CRITICAL_THREAT_THRESHOLD = 850`, `SCARCITY_ALERT_THRESHOLD = 700`, `HAPPINESS_ALERT_THRESHOLD = 45`.

---

## 4. Nation AI — Personality Matrix

### 4.1 Personality Archetypes — Parameter Vectors

Each nation has a `NationPersonality` that seeds a `PersonalityParams` vector at game start. Parameters are `i32` in `[0, 1000]`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NationPersonality {
    Aggressive,
    Expansionist,
    Isolationist,
    Mercantile,
    Diplomatic,
}

/// Full parameter vector. All values i32 in [0, 1000].
/// Higher values = stronger expression of that trait.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonalityParams {
    // War and aggression
    pub war_aggression_bias:         i32,  // base desire to declare war
    pub war_utility_threshold:       i32,  // min utility to actually declare
    pub diplomatic_patience:         i32,  // ticks before escalating grievance
    pub defensive_priority:          i32,  // weight on defensive military build

    // Expansion
    pub expansion_bias:              i32,  // weight on territorial expansion
    pub colonization_weight:         i32,  // preference for unclaimed territory
    pub internal_dev_weight:         i32,  // preference for building vs expanding

    // Trade and economy
    pub trade_weight:                i32,  // value placed on trade income
    pub economic_sector_bias:        EconomicSectorBias, // which sectors to prioritize

    // Diplomacy
    pub alliance_seeking_weight:     i32,  // desire to form alliances
    pub treaty_offer_threshold:      i32,  // min utility to offer treaty
    pub soft_power_weight:           i32,  // value of ideological influence

    // Risk
    pub risk_tolerance:              i32,  // willingness to take risky actions
    pub espionage_aggressiveness:    i32,  // frequency of covert ops

    // Threat weights (for ThreatScore computation)
    pub threat_weight_military:      i32,
    pub threat_weight_territory:     i32,
    pub threat_weight_hostility:     i32,
    pub threat_weight_espionage:     i32,

    // Personality drift accumulators (updated by drift engine, Section 4.3)
    pub drift_toward_aggressive:     i32,  // accumulated drift pressure
    pub drift_toward_mercantile:     i32,
}
```

### 4.2 Archetype Default Parameter Tables

| Parameter | Aggressive | Expansionist | Isolationist | Mercantile | Diplomatic |
|---|---|---|---|---|---|
| `war_aggression_bias` | 800 | 450 | 100 | 150 | 50 |
| `war_utility_threshold` | 400 | 550 | 900 | 750 | 950 |
| `diplomatic_patience` | 100 | 300 | 600 | 500 | 800 |
| `defensive_priority` | 600 | 400 | 900 | 300 | 400 |
| `expansion_bias` | 700 | 900 | 50 | 400 | 250 |
| `colonization_weight` | 400 | 800 | 50 | 350 | 200 |
| `internal_dev_weight` | 200 | 300 | 900 | 600 | 700 |
| `trade_weight` | 200 | 350 | 150 | 950 | 600 |
| `alliance_seeking_weight` | 200 | 400 | 50 | 600 | 950 |
| `treaty_offer_threshold` | 700 | 500 | 800 | 400 | 200 |
| `soft_power_weight` | 100 | 200 | 50 | 400 | 900 |
| `risk_tolerance` | 800 | 600 | 200 | 450 | 300 |
| `espionage_aggressiveness` | 700 | 500 | 200 | 400 | 300 |
| `threat_weight_military` | 600 | 450 | 700 | 400 | 350 |
| `threat_weight_territory` | 300 | 400 | 250 | 200 | 200 |
| `threat_weight_hostility` | 70 | 100 | 40 | 300 | 400 |
| `threat_weight_espionage` | 30 | 50 | 10 | 100 | 50 |

**Personality seeding from nation ID:**

```rust
fn seed_personality(nation_id: EntityId, scenario_seed: u64) -> NationPersonality {
    let mut hasher = SipHasher13::new();
    hasher.write_u64(nation_id.raw());
    hasher.write_u64(scenario_seed);
    hasher.write_u64(0xPERS_DOMAIN_TAG);
    let hash = hasher.finish();
    match hash % 5 {
        0 => NationPersonality::Aggressive,
        1 => NationPersonality::Expansionist,
        2 => NationPersonality::Isolationist,
        3 => NationPersonality::Mercantile,
        _ => NationPersonality::Diplomatic,
    }
}
```

For historical scenario nations (e.g., Napoleonic Wars), the scenario YAML explicitly sets `leader_personality`, overriding the hash-derived value.

### 4.3 Personality Drift

Personality is not static. Accumulated experience causes parameter drift over time. The drift engine runs once every 25 ticks (once per in-game quarter).

```rust
fn apply_personality_drift(params: &mut PersonalityParams, snapshot: &NationSnapshot) {
    // Legitimacy loss → drift toward aggressive
    // Low legitimacy means the government is unstable; aggressive action may seem appealing
    if snapshot.legitimacy < 3000 {  // < 0.30 (fixed-point /10000)
        let pressure = (3000 - snapshot.legitimacy) / 30;  // [0, 100]
        params.drift_toward_aggressive += pressure;
        if params.drift_toward_aggressive > DRIFT_THRESHOLD {
            apply_drift_step(params, NationPersonality::Aggressive, DRIFT_STEP_SIZE);
            params.drift_toward_aggressive = 0;
        }
    }

    // High prosperity → drift toward mercantile
    // Wealthy nations find profit more attractive than war
    if snapshot.gdp_per_capita > HIGH_PROSPERITY_THRESHOLD {
        let pressure = (snapshot.gdp_per_capita - HIGH_PROSPERITY_THRESHOLD) / 1000;
        params.drift_toward_mercantile += pressure.min(50) as i32;
        if params.drift_toward_mercantile > DRIFT_THRESHOLD {
            apply_drift_step(params, NationPersonality::Mercantile, DRIFT_STEP_SIZE);
            params.drift_toward_mercantile = 0;
        }
    }

    // Prolonged war → drift toward aggressive (war normalizes)
    if snapshot.ticks_at_war > 100 {
        params.war_aggression_bias = (params.war_aggression_bias + 5).min(1000);
        params.diplomatic_patience = (params.diplomatic_patience - 5).max(0);
    }

    // Long peace → drift toward diplomatic or mercantile
    if snapshot.ticks_since_last_war > 200 {
        params.war_aggression_bias = (params.war_aggression_bias - 3).max(0);
        params.trade_weight = (params.trade_weight + 2).min(1000);
    }
}

/// DRIFT_THRESHOLD = 500 (accumulated pressure before a step change occurs)
/// DRIFT_STEP_SIZE = 30 (parameter change per drift step)
/// HIGH_PROSPERITY_THRESHOLD = 5000 (GDP per capita, fixed-point units)
```

Personality drift is capped: parameters cannot drift more than 400 units from their archetype defaults. This preserves nation character while allowing meaningful evolution.

---

## 5. Nation AI — Memory and Learning

### 5.1 AIMemory — Data Structure

```rust
/// Rolling memory for AI decision making. Bounded by difficulty level.
/// All collections are BTreeMap/VecDeque for deterministic iteration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIMemory {
    /// Log of treaty violations by other nations.
    /// Informs alliance reluctance and diplomatic distrust.
    pub betrayal_log: Vec<BetrayalRecord>,

    /// Rolling window of battle outcomes (last N battles, N = memory_depth).
    pub battle_outcomes: VecDeque<BattleOutcome>,

    /// Rolling economic snapshots for trend analysis (last M ticks).
    pub economic_trends: VecDeque<EconomicSnapshot>,

    /// Estimated reliability of each ally: [0, 1000] fixed-point.
    pub ally_reliability: BTreeMap<EntityId, i32>,

    /// Detected espionage operations per source nation, per tick.
    pub detected_ops: BTreeMap<EntityId, VecDeque<TickNumber>>,

    /// Maximum entries in battle_outcomes (set from difficulty level).
    pub max_battle_memory: usize,

    /// Maximum entries in economic_trends (set from difficulty level).
    pub max_economic_memory: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BetrayalRecord {
    pub betrayer: EntityId,
    pub tick: TickNumber,
    pub betrayal_type: BetrayalType,
    /// Relationship penalty applied (negative i32).
    pub penalty_applied: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BetrayalType {
    TreatyBroken      { treaty_id: TreatyId },
    AllianceAbandoned { conflict_id: EntityId },
    EspionageCaught   { operation: EspionageOperation },
    TradeContractBreach,
    UnprovockedAttack,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BattleOutcome {
    pub tick: TickNumber,
    pub opponent: EntityId,
    pub our_initial_strength: i32,
    pub their_initial_strength: i32,
    pub result: BattleResult,
    pub our_losses: i32,
    pub their_losses: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BattleResult { Victory, Defeat, Draw, Retreat }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EconomicSnapshot {
    pub tick: TickNumber,
    pub gdp: i64,
    pub treasury: i64,
    pub joule_reserves: i64,
    pub avg_happiness: i32,
    pub legitimacy: i32,
    pub supply_stress: i32,
}
```

### 5.2 Betrayal Memory and Relationship Penalties

When a treaty violation is detected (event `treaty.violated.v1`):

```rust
fn process_betrayal(memory: &mut AIMemory, betrayer: EntityId, betrayal_type: BetrayalType, tick: TickNumber) {
    let base_penalty = match betrayal_type {
        BetrayalType::TreatyBroken { .. }         => -200,
        BetrayalType::AllianceAbandoned { .. }    => -300,
        BetrayalType::EspionageCaught { .. }      => -150,
        BetrayalType::TradeContractBreach         => -100,
        BetrayalType::UnprovockedAttack           => -400,
    };

    memory.betrayal_log.push(BetrayalRecord {
        betrayer,
        tick,
        betrayal_type: betrayal_type.clone(),
        penalty_applied: base_penalty,
    });

    // Permanent relationship score adjustment (applied to DiplomaticRelation)
    // This persists even after the record decays from memory
    // The penalty is halved by memory decay (see Section 5.4) but a floor of
    // base_penalty / 4 persists indefinitely.
    memory.ally_reliability
        .entry(betrayer)
        .and_modify(|r| *r = (*r + base_penalty).max(-1000))
        .or_insert(500 + base_penalty);
}
```

### 5.3 Battle Outcome Learning — War Threshold Adjustment

After each battle, the AI adjusts its war-readiness based on recent win/loss ratio:

```rust
fn apply_battle_learning(params: &mut PersonalityParams, memory: &AIMemory) {
    if memory.battle_outcomes.is_empty() { return; }

    let recent: Vec<_> = memory.battle_outcomes.iter()
        .rev()
        .take(10)
        .collect();

    let wins = recent.iter().filter(|b| b.result == BattleResult::Victory).count() as i32;
    let losses = recent.iter().filter(|b| b.result == BattleResult::Defeat).count() as i32;
    let total = recent.len() as i32;

    // Win ratio in [0, 1000]
    let win_ratio = wins * 1000 / total;

    // If winning consistently: raise aggression slightly (wars are going well)
    if win_ratio > 700 {
        params.war_aggression_bias = (params.war_aggression_bias + 10).min(1000);
        params.war_utility_threshold = (params.war_utility_threshold - 10).max(200);
    }

    // If losing consistently: reduce aggression, lower war threshold (more cautious)
    if win_ratio < 300 {
        params.war_aggression_bias = (params.war_aggression_bias - 20).max(0);
        params.war_utility_threshold = (params.war_utility_threshold + 30).min(1000);
    }

    // Heavy losses regardless of outcome: adjust defensive posture
    let avg_loss_ratio = recent.iter()
        .map(|b| if b.our_initial_strength > 0 {
            b.our_losses * 1000 / b.our_initial_strength
        } else { 0 })
        .sum::<i32>() / total;

    if avg_loss_ratio > 400 {
        params.defensive_priority = (params.defensive_priority + 50).min(1000);
    }
}
```

### 5.4 Economic Trend Extrapolation

The AI uses linear regression over the last 20 economic snapshots to project future resource availability. This is computed in fixed-point arithmetic.

```rust
fn extrapolate_gdp_trend(memory: &AIMemory, horizon: i32) -> i64 {
    let snapshots: Vec<_> = memory.economic_trends.iter()
        .rev()
        .take(20)
        .collect();

    if snapshots.len() < 2 { return snapshots.first().map(|s| s.gdp).unwrap_or(0); }

    // Linear regression: y = mx + b, where x = tick offset, y = gdp
    // Computed in integer arithmetic; slope in units of gdp_change_per_tick
    let n = snapshots.len() as i64;
    let x_mean = (n - 1) / 2;  // Assumes evenly spaced ticks (one per entry)
    let y_mean: i64 = snapshots.iter().map(|s| s.gdp).sum::<i64>() / n;

    let mut num: i64 = 0;
    let mut den: i64 = 0;
    for (i, s) in snapshots.iter().enumerate() {
        let x = i as i64 - x_mean;
        let y = s.gdp - y_mean;
        num += x * y;
        den += x * x;
    }

    let slope = if den == 0 { 0 } else { num / den };
    y_mean + slope * horizon as i64
}
```

This projection is used in goal recomputation: if GDP is trending downward, `DevelopEconomy` goals are prioritized. If trending upward, expansion goals receive more weight.

### 5.5 Memory Decay

Events in memory are weighted by recency. The decay function applies an exponential weight when computing aggregate scores from memory:

```
weight(event_at_tick_T, current_tick) = exp(-(current_tick - T) / HALF_LIFE_TICKS)
```

Where `HALF_LIFE_TICKS = 100` (events from 100 ticks ago have half the weight of current events).

In integer arithmetic, decay is approximated as:

```rust
fn decay_weight(age_ticks: u32, half_life: u32) -> i32 {
    // Returns weight in [0, 1000]
    // Approximation: weight = 1000 >> (age_ticks / half_life)
    // (bit-shifts approximate halving per half-life period)
    let shifts = age_ticks / half_life;
    if shifts >= 10 { return 1; }  // Floor at ~1/1024
    1000 >> shifts
}
```

The betrayal log applies decay when computing effective relationship scores, but the permanent floor (base_penalty / 4) always persists regardless of decay weight. This models the difference between day-to-day diplomatic warmth and deep-seated distrust.

---

## 6. Military AI — Operational Layer

### 6.1 Army Group Management

The operational layer aggregates individual units into army groups and assigns each group a mission. Army groups form dynamically: units within 3 hexes of each other and belonging to the same nation are grouped together.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArmyGroup {
    pub id: ArmyGroupId,
    pub nation_id: EntityId,
    pub units: Vec<EntityId>,
    pub centroid_hex: HexCoord,
    pub mission: ArmyMission,
    pub supply_status: SupplyStatus,
    pub effective_strength: i32,  // [0, 1000] relative to nominal strength
    pub morale: i32,              // [0, 1000]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ArmyMission {
    /// Advance toward target district. Attack on contact.
    Attack  { target: EntityId },
    /// Hold current position. Fortify. Engage if attacked.
    Defend  { anchor_hex: HexCoord },
    /// Fast strike on high-value target. Withdraw immediately after.
    Raid    { target: EntityId, withdrawal_hex: HexCoord },
    /// Interdict supply routes between two points.
    Blockade{ route_from: EntityId, route_to: EntityId },
    /// Hold captured territory against insurgency.
    Garrison{ district: EntityId },
    /// Retreat to safe position and regroup.
    Retreat { destination_hex: HexCoord },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SupplyStatus {
    pub current_supply_level: i32,   // [0, 1000], 1000 = fully supplied
    pub ticks_without_supply: u32,
    pub supply_decay_rate: i32,      // units per tick (from difficulty config)
}
```

### 6.2 Mission Assignment Algorithm

```rust
fn assign_mission(
    group: &ArmyGroup,
    threat_model: &BTreeMap<EntityId, ThreatScore>,
    snapshot: &GameStateSnapshot,
    params: &PersonalityParams,
    rng: &mut ChaCha20Rng,
) -> ArmyMission {
    let our_strength = group.effective_strength;
    let supply_ok = group.supply_status.current_supply_level > 300;

    // Critical supply → retreat to resupply first
    if group.supply_status.current_supply_level < SUPPLY_RETREAT_THRESHOLD {
        let safe_hex = find_nearest_supply_depot(group, snapshot);
        return ArmyMission::Retreat { destination_hex: safe_hex };
    }

    // Find the highest-priority threat adjacent to this army group
    let adjacent_threats = snapshot.nations_adjacent_to_group(group);

    for threat_nation in adjacent_threats.iter().sorted_by_key(|n| {
        -threat_model.get(n).map(|t| t.composite).unwrap_or(0)
    }) {
        let threat = threat_model.get(threat_nation).unwrap_or(&ThreatScore::default());
        let enemy_strength = snapshot.military_strength(*threat_nation);
        let strength_ratio = if enemy_strength > 0 {
            our_strength * 1000 / enemy_strength
        } else { 2000 };

        // Attack: we're stronger and aggressive
        if strength_ratio > 1200 && supply_ok && params.war_aggression_bias > 400 {
            if let Some(target) = find_weakest_adjacent_district(threat_nation, snapshot) {
                return ArmyMission::Attack { target };
            }
        }

        // Raid: we're comparable in strength; personality is aggressive; target is valuable
        if strength_ratio > 800 && strength_ratio < 1200
            && params.risk_tolerance > 500
            && supply_ok
        {
            if let Some((target, withdrawal)) = find_raid_target(threat_nation, snapshot, group) {
                return ArmyMission::Raid { target, withdrawal_hex: withdrawal };
            }
        }

        // Defend: we're weaker; entrench
        if strength_ratio < 800 || !supply_ok {
            return ArmyMission::Defend { anchor_hex: group.centroid_hex };
        }

        // Blockade: enemy relies heavily on supply route; we can cut it
        if threat.composite > 600 && strength_ratio > 600 {
            if let Some((from, to)) = find_critical_supply_route(threat_nation, snapshot) {
                return ArmyMission::Blockade { route_from: from, route_to: to };
            }
        }
    }

    // No active threat → garrison captured districts or hold position
    if let Some(contested_district) = find_contested_district(group, snapshot) {
        ArmyMission::Garrison { district: contested_district }
    } else {
        ArmyMission::Defend { anchor_hex: group.centroid_hex }
    }
}

const SUPPLY_RETREAT_THRESHOLD: i32 = 300;  // 30% supply level
```

### 6.3 Pathfinding — A* on Hex Grid

Army group movement uses A* pathfinding on the hex grid with terrain cost weights:

```rust
/// Terrain movement cost multipliers (base cost = 1000 per hex).
/// All costs are i32 fixed-point multiplied by 1000.
fn terrain_cost(hex: HexCoord, snapshot: &GameStateSnapshot) -> i32 {
    let base = match snapshot.terrain(hex) {
        Terrain::Plains   => 1000,
        Terrain::Road     =>  500,   // Roads halve movement cost
        Terrain::Forest   => 2000,
        Terrain::Hills    => 3000,
        Terrain::Mountains=> 5000,
        Terrain::River    => 3000,   // River crossing penalty
        Terrain::Marsh    => 4000,
        Terrain::Desert   => 2500,
    };

    // Zone of control: each adjacent enemy unit adds penalty
    let zoc_penalty = snapshot.adjacent_enemy_units(hex).len() as i32 * 10_000;

    base + zoc_penalty
}

/// A* heuristic: Euclidean distance on hex grid (scaled to match terrain costs).
fn hex_heuristic(from: HexCoord, to: HexCoord) -> i32 {
    hex_distance(from, to) * 1000  // Base cost per hex with no terrain penalty
}
```

The pathfinder returns the minimum-cost path from the army group centroid to the mission target. Path is recalculated every tick to account for changing enemy positions and zone of control.

### 6.4 Supply Line Awareness

Supply lines connect army groups to their nation's supply depots. Supply is tracked as a continuous resource in `[0, 1000]`:

```rust
fn update_supply_status(group: &mut ArmyGroup, snapshot: &GameStateSnapshot) {
    let supply_path = find_supply_path(group, snapshot);

    match supply_path {
        Some(path) if path.is_not_interdicted(snapshot) => {
            // Supply route open: replenish supply
            let replenish_rate = snapshot.supply_replenish_rate(group.nation_id);
            group.supply_status.current_supply_level =
                (group.supply_status.current_supply_level + replenish_rate).min(1000);
            group.supply_status.ticks_without_supply = 0;
        }
        _ => {
            // Supply route cut or no path: drain supply
            group.supply_status.ticks_without_supply += 1;
            let drain = group.supply_status.supply_decay_rate
                * group.supply_status.ticks_without_supply as i32;
            group.supply_status.current_supply_level =
                (group.supply_status.current_supply_level - drain).max(0);

            // Effective strength degrades as supply drops
            group.effective_strength = compute_effective_strength(
                group.effective_strength,
                group.supply_status.current_supply_level,
            );
        }
    }
}

fn compute_effective_strength(nominal: i32, supply: i32) -> i32 {
    // At 100% supply: full strength
    // At 50% supply: 75% strength (supply has diminishing effect)
    // At 0% supply: 30% strength (starving army)
    let supply_factor = 300 + (supply * 700 / 1000);
    nominal * supply_factor / 1000
}
```

### 6.5 Retreat Decision Logic

```rust
fn should_retreat(group: &ArmyGroup, snapshot: &GameStateSnapshot) -> bool {
    let initial_strength = snapshot.initial_strength(group.id);
    let strength_ratio = if initial_strength > 0 {
        group.effective_strength * 1000 / initial_strength
    } else { 0 };

    let supply_critical = group.supply_status.current_supply_level < 300;

    let surrounded = snapshot.enemy_zone_of_control_hexes()
        .iter()
        .filter(|hex| hex_distance(**hex, group.centroid_hex) <= 1)
        .count() >= 4;  // surrounded on 4+ sides

    // Retreat conditions:
    strength_ratio < 400           // lost > 60% of initial strength
        || supply_critical          // < 30% supply
        || surrounded               // no escape route
}
```

---

## 7. Military AI — Tactical Layer

### 7.1 Tactical AI Scope

The tactical layer controls individual units during the battle resolution phase (CIV-0001 Phase 6: Diplomacy and Conflict). It runs in real-time during battle ticks, not once-per-simulation-tick.

Tactical AI is simpler than operational AI by design: battles are resolved quickly and the state space is bounded to the battle hex area.

### 7.2 Unit Targeting Priority

```rust
fn select_target(unit: &Unit, battle_state: &BattleState) -> Option<EntityId> {
    let enemies = battle_state.enemy_units_in_range(unit);
    if enemies.is_empty() { return None; }

    // Priority-ordered target selection:
    // 1. Enemy siege equipment (threatens our fortifications)
    // 2. Weakest enemy unit (fastest kill, improve numbers advantage)
    // 3. Highest-threat enemy unit (most dangerous to us)
    // 4. Nearest enemy unit (default)

    if let Some(siege) = enemies.iter()
        .find(|e| e.unit_type == UnitType::SiegeEquipment)
    {
        return Some(siege.id);
    }

    enemies.iter()
        .min_by_key(|e| e.current_hp)
        .map(|e| e.id)
}
```

### 7.3 Formation Logic

Units maintain formation relative to their army group centroid. Formation type is selected by mission:

| Mission | Formation | Description |
|---|---|---|
| Attack | Wedge | Units concentrate forward, cavalry on flanks |
| Defend | Line | Units spread across defensive perimeter |
| Raid | Column | Fast-moving column, minimal flank coverage |
| Garrison | Perimeter | Units distributed around district border |
| Retreat | Rearguard | Strongest units at rear, vulnerable at front |

Formation adjusts every battle tick by moving units toward their formation position relative to centroid.

### 7.4 Morale Management

```rust
fn update_unit_morale(unit: &mut Unit, battle_state: &BattleState) {
    let base = unit.morale;

    // Morale penalties
    let ally_rout_penalty = battle_state.allies_routed_this_tick() * 50;
    let outnumbered_penalty = if battle_state.enemy_count() > battle_state.ally_count() * 2 {
        100
    } else { 0 };
    let supply_penalty = if battle_state.army_supply_level < 300 { 100 } else { 0 };

    // Morale bonuses
    let terrain_bonus = battle_state.defender_terrain_bonus(unit.hex) / 2;
    let kill_bonus = battle_state.kills_by_unit(unit.id) * 20;
    let hero_bonus = if battle_state.hero_unit_alive { 50 } else { 0 };

    unit.morale = (base
        - ally_rout_penalty - outnumbered_penalty - supply_penalty
        + terrain_bonus + kill_bonus + hero_bonus
    ).clamp(0, 1000);

    // Rout check: morale below 200 → unit routs with probability
    if unit.morale < 200 {
        let rout_chance = (200 - unit.morale) * 5;  // [0, 1000]
        // Rout is rolled in Phase 4 (stochastic) using ChaCha20Rng
        unit.rout_probability = rout_chance;
    }
}
```

---

## 8. Diplomatic AI

### 8.1 Diplomatic Relation Model

```rust
/// Full diplomatic relation between two nations. Ordered by nation_id (a < b).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiplomaticRelation {
    pub nation_a: EntityId,
    pub nation_b: EntityId,
    /// Relationship score: [-1000, 1000]. -1000 = total enmity, 1000 = deep alliance.
    pub score: i32,
    /// Active treaties between these nations.
    pub treaties: Vec<TreatyId>,
    /// Tick of last war between these nations. None if never at war.
    pub last_war_tick: Option<TickNumber>,
    /// Tick of last betrayal event (treaty break, espionage caught).
    pub last_betrayal_tick: Option<TickNumber>,
    /// Trade volume over last 100 ticks (in joules equivalent).
    pub trade_volume_last_100_ticks: i64,
    /// Nations that are enemies of both: common enemies strengthen bonds.
    pub shared_enemies: Vec<EntityId>,
    /// Current diplomatic FSM state (from CIV-0105).
    pub diplomatic_state: DiplomaticState,
    /// Grievance accumulator (feeds DiplomaticState transitions).
    pub grievance_score: i32,  // [0, 1000]
}
```

### 8.2 Relationship Score Update — Per-Tick Formula

```rust
fn update_relationship_score(
    relation: &mut DiplomaticRelation,
    events: &[DiplomaticEvent],
    snapshot: &GameStateSnapshot,
) {
    for event in events {
        let delta = match event {
            DiplomaticEvent::TradeTick { joule_volume } => {
                // +1 per 100J of trade per tick; capped at +5/tick
                (*joule_volume / 100).min(5) as i32
            }
            DiplomaticEvent::SharedEnemyEngaged { .. } => 5,
            DiplomaticEvent::AllianceYearPassed => 2,
            DiplomaticEvent::WarDeclaredOn { .. } => -20,
            DiplomaticEvent::WarTick { .. } => -10,
            DiplomaticEvent::PromiseBroken { .. } => -50,
            DiplomaticEvent::EspionageCaught { .. } => -30,
            DiplomaticEvent::GiftSent { value } => (*value / 1000).min(20) as i32,
            DiplomaticEvent::InsultPublic { .. } => -15,
            DiplomaticEvent::TerritoryViolated { .. } => -25,
        };
        relation.score = (relation.score + delta).clamp(-1000, 1000);
    }

    // Passive decay toward neutral (600 = neutral for this scale)
    // Relations drift toward neutral over time if no events
    // (50% decay of distance from neutral every 500 ticks)
    let neutral = 0;
    let distance_from_neutral = relation.score - neutral;
    relation.score = neutral + distance_from_neutral * 999 / 1000;

    // Shared enemies boost
    let shared_count = snapshot.shared_enemies(relation.nation_a, relation.nation_b).len() as i32;
    if shared_count > 0 {
        relation.score = (relation.score + shared_count * 3).min(1000);
    }
}
```

### 8.3 Treaty Offer Decision

The AI evaluates whether to offer a treaty each full evaluation cycle:

```rust
fn should_offer_treaty(
    relation: &DiplomaticRelation,
    treaty_type: TreatyType,
    params: &PersonalityParams,
    memory: &AIMemory,
    current_tick: TickNumber,
) -> bool {
    // Must have positive relationship
    if relation.score < TREATY_OFFER_SCORE_FLOOR { return false; }

    // Must not have recent conflict
    if let Some(war_tick) = relation.last_war_tick {
        if current_tick.0 - war_tick.0 < RECENT_CONFLICT_TICKS { return false; }
    }

    // Must not have recent betrayal
    if let Some(betrayal_tick) = relation.last_betrayal_tick {
        if current_tick.0 - betrayal_tick.0 < BETRAYAL_COOLDOWN_TICKS { return false; }
    }

    // Compute utility for this treaty type
    let utility = sign_treaty_utility_for_type(relation, treaty_type, params);
    utility > params.treaty_offer_threshold
}

const TREATY_OFFER_SCORE_FLOOR: i32 = 300;
const RECENT_CONFLICT_TICKS: u64 = 200;
const BETRAYAL_COOLDOWN_TICKS: u64 = 150;
```

### 8.4 Alliance Formation Rules

Alliance types and their minimum requirements:

| Alliance Type | Min Relationship Score | Additional Condition | Mutual Benefit |
|---|---|---|---|
| Non-Aggression Pact | 150 | No active war | Neither attacks the other |
| Trade Agreement | 250 | Trade volume > 10K J/100t | Tariff reduction |
| Military Cooperation | 400 | Shared enemy exists | Intel sharing |
| Mutual Defense Pact | 600 | No recent war \< 500 ticks | Auto-join defensive war |
| Economic Alliance | 400 | Trade volume > 50K J/100t | Joint production |
| Full Alliance | 750 | No recent war \< 1000 ticks | Full military + economic |

```rust
fn propose_alliance(
    relation: &DiplomaticRelation,
    memory: &AIMemory,
    snapshot: &GameStateSnapshot,
    current_tick: TickNumber,
) -> Option<TreatyType> {
    // Find the highest-tier alliance the relationship score supports
    let alliance_types = [
        (TreatyType::FullAlliance,          750, 1000),
        (TreatyType::MutualDefensePact,     600,  500),
        (TreatyType::MilitaryCooperation,   400,    0),
        (TreatyType::EconomicAlliance,      400,    0),
        (TreatyType::TradeAgreement,        250,    0),
        (TreatyType::NonAggressionPact,     150,    0),
    ];

    for (treaty_type, score_req, war_cooldown) in &alliance_types {
        if relation.score >= *score_req {
            let war_ok = match relation.last_war_tick {
                None => true,
                Some(t) => current_tick.0 - t.0 >= *war_cooldown,
            };
            if war_ok {
                // Check additional conditions
                if additional_conditions_met(relation, treaty_type, snapshot) {
                    return Some(*treaty_type);
                }
            }
        }
    }
    None
}
```

### 8.5 War Declaration Decision Integration

War declaration integrates diplomatic state, utility scoring, and casus belli validation:

```rust
fn evaluate_war_declaration(
    ai_state: &NationAIState,
    target: EntityId,
    snapshot: &GameStateSnapshot,
    current_tick: TickNumber,
) -> Option<CasusBelli> {
    // Step 1: Check if war is strategically desirable
    let utility = declare_war_utility(ai_state, target, snapshot);
    if utility <= ai_state.params.war_utility_threshold { return None; }

    // Step 2: Verify we have a valid casus belli
    let available_cb = snapshot.available_casus_belli(ai_state.nation_id, target);
    if available_cb.is_empty() { return None; }

    // Step 3: Select the casus belli with highest war support
    let best_cb = available_cb.iter()
        .max_by_key(|cb| cb.war_support_base)
        .cloned()?;

    // Step 4: Verify population support is adequate
    let projected_support = compute_war_support(ai_state, &best_cb, snapshot);
    if projected_support < MIN_WAR_SUPPORT { return None; }

    // Step 5: Check that we're not already over-extended
    let active_wars = snapshot.active_wars(ai_state.nation_id).len();
    if active_wars >= ai_state.params.max_simultaneous_wars as usize { return None; }

    Some(best_cb)
}

const MIN_WAR_SUPPORT: i32 = 300;  // 30% population support minimum
```

### 8.6 Peace Negotiation AI

The AI evaluates peace offers using a settlement range calculation:

```rust
fn evaluate_peace_offer(
    ai_state: &NationAIState,
    offer: &PeaceTerms,
    opponent: EntityId,
    snapshot: &GameStateSnapshot,
) -> PeaceDecision {
    // Compute the value of continuing war vs. accepting peace
    let war_continuation_value = estimate_war_continuation_value(ai_state, opponent, snapshot);
    let peace_value = evaluate_peace_terms_value(offer, ai_state, snapshot);

    // Accept if peace value exceeds war continuation + peace_preference bias
    let peace_bias = 1000 - ai_state.params.war_aggression_bias;
    if peace_value + peace_bias > war_continuation_value {
        PeaceDecision::Accept
    } else if offer.is_improvable() {
        // Propose counter-terms
        let counter = generate_counter_terms(ai_state, offer, snapshot);
        PeaceDecision::Counter(counter)
    } else {
        PeaceDecision::Reject
    }
}
```

---

## 9. City-Level AI — Development Planner

### 9.1 District Development Priority Queue

The development planner runs once per tick for each AI-controlled district, selecting the highest-priority development action:

```rust
fn compute_development_score(
    district: &District,
    structure_type: StructureType,
    snapshot: &GameStateSnapshot,
    planning_horizon: i32,
) -> i32 {
    // resource_output_gain: additional output per tick over planning horizon
    let resource_gain = estimate_resource_gain(district, structure_type, planning_horizon);
    let resource_gain_normalized = resource_gain * 1000 / MAX_RESOURCE_GAIN;

    // population_growth_potential: structures that enable more housing or food
    let pop_growth = estimate_pop_growth(district, structure_type, planning_horizon);
    let pop_growth_normalized = pop_growth * 1000 / MAX_POP_GROWTH;

    // strategic_value: proximity to borders, resource hotspots, trade routes
    let strategic = compute_strategic_value(district, structure_type, snapshot);

    // construction_cost_normalized: joule cost relative to current reserves
    let cost_penalty = {
        let cost = structure_type.joule_cost();
        let reserves = snapshot.joule_reserves(district.nation_id);
        if reserves == 0 { -1000 } else { -(cost * 1000 / reserves).min(1000) }
    };

    let score =
        resource_gain_normalized * 350 / 1000
        + pop_growth_normalized  * 200 / 1000
        + strategic              * 250 / 1000
        + cost_penalty           * 200 / 1000;

    score.clamp(-1000, 1000)
}
```

### 9.2 Building Selection — Greedy Policy

The AI selects buildings using a greedy policy: build whichever structure maximizes `resource_output_gain / joule_cost` over the next `planning_horizon` ticks:

```rust
fn select_next_building(
    district: &District,
    snapshot: &GameStateSnapshot,
    params: &PersonalityParams,
    planning_horizon: i32,
) -> Option<StructureType> {
    let available = snapshot.buildable_structures(district);
    let reserves = snapshot.joule_reserves(district.nation_id);

    available.iter()
        .filter(|s| s.joule_cost() <= reserves)  // Can we afford it?
        .map(|s| {
            let gain = estimate_resource_gain(district, *s, planning_horizon);
            let cost = s.joule_cost().max(1);
            let roi = gain * 1000 / cost;
            (s, roi)
        })
        .max_by_key(|(_, roi)| *roi)
        .map(|(s, _)| *s)
}
```

### 9.3 Population Assignment — Optimization

The AI assigns citizens to jobs each tick to maximize total productivity. For small districts (< 1000 citizens), it uses a greedy assignment. For larger districts, it uses the Hungarian algorithm for optimal assignment.

```rust
fn assign_population_to_jobs(
    district: &District,
    snapshot: &GameStateSnapshot,
) -> Vec<(CitizenId, JobSlotId)> {
    let citizens = snapshot.unemployed_or_reassignable_citizens(district.id);
    let jobs = snapshot.open_job_slots(district.id);

    if citizens.len() <= 1000 {
        // Greedy: assign each citizen to highest-productivity job available
        greedy_job_assignment(&citizens, &jobs, snapshot)
    } else {
        // Hungarian algorithm for optimal assignment (O(n^3))
        hungarian_job_assignment(&citizens, &jobs, snapshot)
    }
}

fn citizen_job_productivity(citizen: &Citizen, job: &JobSlot) -> i32 {
    let skill_match = citizen.skill_level(job.required_skill) * 1000
        / MAX_SKILL_LEVEL;
    let happiness_bonus = citizen.happiness / 10;
    skill_match + happiness_bonus
}
```

### 9.4 Healthcare and Education Investment

```rust
fn evaluate_welfare_investment(
    district: &District,
    snapshot: &GameStateSnapshot,
) -> WelfareDecision {
    let avg_happiness = snapshot.avg_citizen_happiness(district.id);
    let healthcare_coverage = snapshot.healthcare_coverage(district.id);
    let education_coverage = snapshot.education_coverage(district.id);

    // If happiness critically low: prioritize welfare above all else
    if avg_happiness < HAPPINESS_CRITICAL_THRESHOLD {
        return WelfareDecision::BuildWelfareStructure {
            priority: StructureType::Hospital,
        };
    }

    // If healthcare below 60%: build hospital
    if healthcare_coverage < 600 {
        return WelfareDecision::BuildWelfareStructure {
            priority: StructureType::Hospital,
        };
    }

    // If education below 50%: build school
    if education_coverage < 500 {
        return WelfareDecision::BuildWelfareStructure {
            priority: StructureType::School,
        };
    }

    WelfareDecision::NoImmediateAction
}

const HAPPINESS_CRITICAL_THRESHOLD: i32 = 40;  // Below 40/100: crisis
const HAPPINESS_ALERT_THRESHOLD: i32 = 50;      // Below 50/100: alert
```

---

## 10. Difficulty Scaling System

### 10.1 Difficulty Level Definitions

| Level | Name | AI APM | Memory Depth | Planning Horizon | MCTS | Cheats |
|---|---|---|---|---|---|---|
| 1 | Novice | 5/min | 10 ticks | 5 ticks | No | None |
| 2 | Standard | 15/min | 25 ticks | 15 ticks | No | None |
| 3 | Advanced | 30/min | 50 ticks | 30 ticks | No | None |
| 4 | Expert | 60/min | 100 ticks | 50 ticks | Yes (depth=50) | +10% resources |
| 5 | Legendary | 120/min | 200 ticks | 100 ticks | Yes (depth=100) | +25% resources, perfect intel |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum DifficultyLevel {
    Novice    = 1,
    Standard  = 2,
    Advanced  = 3,
    Expert    = 4,
    Legendary = 5,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DifficultyConfig {
    pub level: DifficultyLevel,
    pub max_apm: u32,
    pub memory_depth_ticks: usize,
    pub planning_horizon_ticks: i32,
    pub mcts_enabled: bool,
    pub mcts_depth_ticks: i32,
    pub mcts_compute_budget_ms: u64,
    pub resource_bonus_milli: i32,       // +10% = 100 (fixed-point /1000)
    pub perfect_intel: bool,
    pub supply_decay_rate: i32,          // Per tick without supply
    pub evaluation_frequency_ticks: u32, // Full re-eval period
}

impl DifficultyLevel {
    pub fn config(&self) -> DifficultyConfig {
        match self {
            Self::Novice => DifficultyConfig {
                level: *self,
                max_apm: 5,
                memory_depth_ticks: 10,
                planning_horizon_ticks: 5,
                mcts_enabled: false,
                mcts_depth_ticks: 0,
                mcts_compute_budget_ms: 0,
                resource_bonus_milli: 0,
                perfect_intel: false,
                supply_decay_rate: 15,
                evaluation_frequency_ticks: 25,
            },
            Self::Standard => DifficultyConfig {
                level: *self,
                max_apm: 15,
                memory_depth_ticks: 25,
                planning_horizon_ticks: 15,
                mcts_enabled: false,
                mcts_depth_ticks: 0,
                mcts_compute_budget_ms: 0,
                resource_bonus_milli: 0,
                perfect_intel: false,
                supply_decay_rate: 20,
                evaluation_frequency_ticks: 15,
            },
            Self::Advanced => DifficultyConfig {
                level: *self,
                max_apm: 30,
                memory_depth_ticks: 50,
                planning_horizon_ticks: 30,
                mcts_enabled: false,
                mcts_depth_ticks: 0,
                mcts_compute_budget_ms: 0,
                resource_bonus_milli: 0,
                perfect_intel: false,
                supply_decay_rate: 25,
                evaluation_frequency_ticks: 10,
            },
            Self::Expert => DifficultyConfig {
                level: *self,
                max_apm: 60,
                memory_depth_ticks: 100,
                planning_horizon_ticks: 50,
                mcts_enabled: true,
                mcts_depth_ticks: 50,
                mcts_compute_budget_ms: 100,
                resource_bonus_milli: 100,  // +10%
                perfect_intel: false,
                supply_decay_rate: 30,
                evaluation_frequency_ticks: 5,
            },
            Self::Legendary => DifficultyConfig {
                level: *self,
                max_apm: 120,
                memory_depth_ticks: 200,
                planning_horizon_ticks: 100,
                mcts_enabled: true,
                mcts_depth_ticks: 100,
                mcts_compute_budget_ms: 200,
                resource_bonus_milli: 250,  // +25%
                perfect_intel: true,
                supply_decay_rate: 35,
                evaluation_frequency_ticks: 3,
            },
        }
    }
}
```

### 10.2 APM Governor Implementation

```rust
pub struct TokenBucket {
    max_tokens: u32,
    current_tokens: u32,
    refill_rate_per_tick: u32,  // tokens added per simulation tick
    ticks_per_minute: u32,      // 25 ticks/minute (1 tick = ~2 weeks, but in real-time 10Hz mode)
}

impl TokenBucket {
    pub fn new(max_apm: u32, ticks_per_minute: u32) -> Self {
        let refill = max_apm / ticks_per_minute;
        Self {
            max_tokens: max_apm,
            current_tokens: max_apm / 2,  // Start at half-full
            refill_rate_per_tick: refill.max(1),
            ticks_per_minute,
        }
    }

    pub fn tick_refill(&mut self) {
        self.current_tokens = (self.current_tokens + self.refill_rate_per_tick)
            .min(self.max_tokens);
    }

    pub fn try_consume(&mut self, n: u32) -> bool {
        if self.current_tokens >= n {
            self.current_tokens -= n;
            true
        } else {
            false
        }
    }
}
```

### 10.3 Difficulty Cheats — Disclosure and Implementation

At difficulty levels 4 and 5, the AI receives resource bonuses framed as "experienced leadership" bonuses. These are fully disclosed in the scenario description and the AI thought panel.

```rust
fn apply_difficulty_bonus(
    nation_resources: &mut NationResources,
    config: &DifficultyConfig,
) {
    if config.resource_bonus_milli > 0 {
        // Apply +10% or +25% to all resource production this tick
        // This is applied as a multiplier to production output, not a cheat credit
        let factor = 1000 + config.resource_bonus_milli;
        nation_resources.apply_production_multiplier(factor);
        // Emit observability event so player can see the bonus
        nation_resources.log_leadership_bonus(config.resource_bonus_milli);
    }
}

fn apply_perfect_intel(
    ai_state: &mut NationAIState,
    snapshot: &GameStateSnapshot,
    config: &DifficultyConfig,
) {
    if config.perfect_intel {
        // At Legendary: AI sees all nations' exact military strength,
        // economic state, and diplomatic relations
        // (normally occluded by fog of war)
        ai_state.threat_model = compute_perfect_threat_model(snapshot);
        ai_state.opportunity_model = compute_perfect_opportunity_model(snapshot);
    }
}
```

### 10.4 Fog of War at Standard Difficulty

At difficulties 1–4, the AI's knowledge of other nations is limited:

- Military strength: known within &plusmn;20% (based on intelligence operations)
- Economic state: known within &plusmn;30% without spy assets
- Diplomatic state: fully known (public information)
- Espionage operations: known only if the AI has infiltration assets

The AI's threat model at standard difficulties uses estimates, not exact values:

```rust
fn estimate_military_strength(
    target: EntityId,
    ai_state: &NationAIState,
    snapshot: &GameStateSnapshot,
) -> i32 {
    let true_strength = snapshot.military_strength(target);
    if ai_state.difficulty == DifficultyLevel::Legendary {
        return true_strength;
    }

    // Estimate with noise based on intelligence coverage
    let intel_coverage = ai_state.memory.intelligence_coverage(target);
    let noise_range = (1000 - intel_coverage) * 2 / 10;  // &plusmn;0-20% noise
    let noise = ai_state.rng.gen_range(-noise_range..=noise_range);
    (true_strength + true_strength * noise / 1000).max(0)
}
```

---

## 11. MCTS Implementation (Difficulty 4–5)

### 11.1 Overview

At difficulty levels 4 and 5, the AI uses Monte Carlo Tree Search (MCTS) with UCB1 selection to evaluate action sequences over a lookahead horizon. MCTS is bounded by a wall-clock compute budget (`mcts_compute_budget_ms`) and runs in a separate thread to avoid blocking the main simulation tick.

### 11.2 MCTS Node and Tree

```rust
/// A single node in the MCTS search tree.
#[derive(Debug)]
pub struct MCTSNode {
    /// Snapshot of simulation state at this node.
    pub state: SimulationStateSnapshot,
    /// Action taken to reach this node from parent. None for root.
    pub action: Option<AiAction>,
    /// Number of times this node has been visited in simulation.
    pub visits: u32,
    /// Total accumulated reward across all simulations through this node.
    pub total_reward: i64,
    /// Child nodes (expanded lazily).
    pub children: Vec<Box<MCTSNode>>,
    /// Whether this node has been fully expanded.
    pub fully_expanded: bool,
}

impl MCTSNode {
    /// UCB1 selection score for child nodes.
    /// C = exploration constant (sqrt(2) approximated as 1414/1000 in fixed-point).
    pub fn ucb1_score(&self, parent_visits: u32) -> i64 {
        if self.visits == 0 {
            return i64::MAX;  // Unvisited nodes have infinite priority
        }
        let exploitation = self.total_reward / self.visits as i64;
        let exploration = {
            // C * sqrt(ln(parent_visits) / visits)
            // Approximated with integer sqrt
            let ln_parent = integer_ln(parent_visits as i64);
            let ratio = ln_parent * 1000 / self.visits as i64;
            let sqrt_ratio = integer_sqrt(ratio);
            1414 * sqrt_ratio / 1000  // C = sqrt(2) &asymp; 1.414
        };
        exploitation + exploration
    }
}
```

### 11.3 MCTS Search Algorithm

```rust
/// Run MCTS search from the current state and return the best immediate action.
pub fn mcts_search(
    root_state: SimulationStateSnapshot,
    nation_id: EntityId,
    config: &DifficultyConfig,
    params: &PersonalityParams,
    rng: &mut ChaCha20Rng,
) -> AiAction {
    let deadline = Instant::now() + Duration::from_millis(config.mcts_compute_budget_ms);
    let mut root = MCTSNode {
        state: root_state,
        action: None,
        visits: 0,
        total_reward: 0,
        children: Vec::new(),
        fully_expanded: false,
    };

    // MCTS loop: run until compute budget exhausted
    while Instant::now() < deadline {
        // 1. Selection: descend tree using UCB1 until a leaf or unexpanded node
        let path = select_node(&root);

        // 2. Expansion: add a child node for an untried action
        let leaf = expand_node(&mut root, &path, nation_id, params, rng);

        // 3. Simulation: fast rollout from leaf using utility-scored random policy
        let reward = simulate_rollout(
            &leaf.state,
            nation_id,
            config.mcts_depth_ticks,
            params,
            rng,
        );

        // 4. Backpropagation: update visit counts and rewards up the path
        backpropagate(&mut root, &path, reward);
    }

    // Return the action of the most-visited child of root
    root.children.iter()
        .max_by_key(|c| c.visits)
        .and_then(|c| c.action.clone())
        .unwrap_or_else(|| best_greedy_action(&root.state, nation_id, params))
}

/// Fast rollout simulation for MCTS.
/// Uses the utility scorer as a policy (pick highest-utility action each step).
/// Runs for `depth` ticks and returns aggregate reward.
fn simulate_rollout(
    state: &SimulationStateSnapshot,
    nation_id: EntityId,
    depth: i32,
    params: &PersonalityParams,
    rng: &mut ChaCha20Rng,
) -> i64 {
    let mut current_state = state.clone();
    let mut total_reward: i64 = 0;
    let mut discount = 1000i64;  // Discount factor: 0.95^t, approximated as 950/1000 per step

    for _ in 0..depth {
        // Pick action using utility scorer (fast, no recursion)
        let action = best_greedy_action(&current_state, nation_id, params);

        // Advance state by one tick using fast simulation approximation
        current_state = fast_simulate_tick(current_state, nation_id, &action);

        // Compute reward for this state
        let reward = compute_state_reward(&current_state, nation_id, params);
        total_reward += reward * discount / 1000;
        discount = discount * 950 / 1000;  // 5% discount per tick
    }

    total_reward
}
```

### 11.4 Reward Function

The MCTS reward function evaluates how good a simulation state is for the AI nation. Weights are personality-dependent:

```rust
fn compute_state_reward(
    state: &SimulationStateSnapshot,
    nation_id: EntityId,
    params: &PersonalityParams,
) -> i64 {
    let territory = state.district_count(nation_id) as i64;
    let gdp = state.gdp(nation_id);
    let army_strength = state.military_strength(nation_id) as i64;
    let alliance_count = state.alliance_count(nation_id) as i64;
    let legitimacy = state.legitimacy(nation_id) as i64;

    // Normalize each factor to [0, 1000] range
    let territory_score = territory * 1000 / state.total_districts() as i64;
    let gdp_score = gdp * 1000 / state.total_gdp().max(1);
    let strength_score = army_strength * 1000 / state.max_military_strength().max(1) as i64;
    let alliance_score = alliance_count * 1000 / state.nation_count() as i64;
    let legitimacy_score = legitimacy;  // Already [0, 10000], divide later

    // Personality-weighted reward
    (
        territory_score  * params.expansion_bias as i64           / 1000 +
        gdp_score        * params.trade_weight as i64             / 1000 +
        strength_score   * params.defensive_priority as i64       / 1000 +
        alliance_score   * params.alliance_seeking_weight as i64  / 1000 +
        legitimacy_score * 500i64                                  / 10000  // fixed-point correction
    )
}
```

### 11.5 MCTS Thread Management

MCTS runs on a dedicated thread pool to avoid blocking the simulation tick:

```rust
pub struct MCTSScheduler {
    thread_pool: ThreadPool,
    pending: HashMap<EntityId, JoinHandle<AiAction>>,
    config: DifficultyConfig,
}

impl MCTSScheduler {
    pub fn schedule(&mut self, nation_id: EntityId, state: SimulationStateSnapshot,
                    params: PersonalityParams, rng_seed: u64) {
        let config = self.config.clone();
        let handle = self.thread_pool.spawn(move || {
            let mut rng = ChaCha20Rng::from_seed(derive_chacha_seed(rng_seed));
            mcts_search(state, nation_id, &config, &params, &mut rng)
        });
        self.pending.insert(nation_id, handle);
    }

    pub fn collect(&mut self, nation_id: EntityId) -> Option<AiAction> {
        self.pending.remove(&nation_id).and_then(|h| h.join().ok())
    }
}
```

MCTS results from tick T are applied at tick T+1. If MCTS is still running when the next tick arrives, the result from the previous cycle is used and a new search is scheduled.

---

## 12. Espionage AI

### 12.1 Espionage Decision Cycle

The espionage AI runs every tick as part of the Strategic Layer evaluation. It manages:

1. **Operation selection**: choose which covert operation to launch this tick
2. **Asset placement**: decide where to embed intelligence assets
3. **Asset rotation**: rotate assets approaching detection risk threshold
4. **Counter-intelligence**: detect and neutralize foreign operations
5. **Shadow network expansion**: extend network into target institutions

### 12.2 Operation Selection

```rust
fn select_espionage_operation(
    ai_state: &NationAIState,
    targets: &[EntityId],
    snapshot: &GameStateSnapshot,
) -> Option<(EntityId, EspionageOperation)> {
    let budget = snapshot.espionage_budget(ai_state.nation_id);
    if budget < MIN_OPERATION_BUDGET { return None; }

    let mut best_utility = ai_state.params.espionage_launch_threshold;
    let mut best_operation = None;

    for target in targets {
        for operation in EspionageOperation::all() {
            if operation.joule_cost() > budget { continue; }

            let utility = compute_espionage_utility(ai_state, *target, operation, snapshot);
            if utility > best_utility {
                best_utility = utility;
                best_operation = Some((*target, operation));
            }
        }
    }

    best_operation
}

fn compute_espionage_utility(
    ai_state: &NationAIState,
    target: EntityId,
    operation: EspionageOperation,
    snapshot: &GameStateSnapshot,
) -> i32 {
    let intel_gain = operation.intel_gain() * 1000 / MAX_INTEL_GAIN;
    let disruption = operation.disruption_value() * 1000 / MAX_DISRUPTION;

    let detection_prob = estimate_detection_probability(operation, target, snapshot);
    let consequence = operation.consequence_severity();
    let detection_risk = -(detection_prob * consequence / 1000);

    let cost = operation.joule_cost();
    let reserves = snapshot.joule_reserves(ai_state.nation_id).max(1);
    let cost_penalty = -(cost * 1000 / reserves as i32).min(1000);

    let utility =
        intel_gain    * ai_state.params.espionage_intel_weight          / 1000 +
        disruption    * ai_state.params.espionage_disruption_weight     / 1000 +
        detection_risk* ai_state.params.espionage_risk_weight           / 1000 +
        cost_penalty  * ai_state.params.espionage_cost_weight           / 1000;

    utility.clamp(-1000, 1000)
}

const MIN_OPERATION_BUDGET: i64 = 10_000;  // 10K joules minimum to consider any op
```

### 12.3 Available Espionage Operations

| Operation | Intel Gain | Disruption Value | Base Detection Chance | Joule Cost |
|---|---|---|---|---|
| `IntelligenceGather` | 800 | 0 | 200 (20%) | 10K |
| `TechSabotage` | 100 | 700 | 400 (40%) | 50K |
| `TradeSabotage` | 50 | 500 | 300 (30%) | 30K |
| `PropagandaCampaign` | 200 | 600 | 250 (25%) | 20K |
| `AssetRecruit` | 400 | 100 | 350 (35%) | 25K |
| `CovertAssassination` | 50 | 1000 | 800 (80%) | 500K |
| `ShadowNetworkExpand` | 300 | 200 | 400 (40%) | 40K |
| `CounterIntelSweep` | 600 | 0 | 0 (self-operation) | 15K |

### 12.4 Asset Management

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EspionageAsset {
    pub asset_id: AssetId,
    pub owner_nation: EntityId,
    pub host_nation: EntityId,
    pub host_institution: EntityId,  // Which institution this asset is embedded in
    pub asset_type: AssetType,
    pub detection_score: i32,  // [0, 1000]: higher = closer to being caught
    pub intelligence_yield: i32,  // [0, 1000]: intel produced per tick
    pub ticks_active: u32,
}

fn manage_assets(
    ai_state: &NationAIState,
    assets: &mut Vec<EspionageAsset>,
    snapshot: &GameStateSnapshot,
) {
    for asset in assets.iter_mut() {
        // Rotate high-risk assets
        if asset.detection_score > ASSET_ROTATE_THRESHOLD {
            // Queue asset rotation (new placement in lower-scrutiny institution)
            ai_state.pending_actions.push(AiAction::RotateAsset {
                asset_id: asset.asset_id,
                new_institution: find_low_scrutiny_institution(asset.host_nation, snapshot),
            });
        }

        // Assets in captured institutions have higher yield but higher detection
        if snapshot.institution_state(asset.host_institution) == InstitutionState::Captured {
            asset.intelligence_yield = (asset.intelligence_yield + 200).min(1000);
            asset.detection_score = (asset.detection_score + 100).min(1000);
        }
    }
}

const ASSET_ROTATE_THRESHOLD: i32 = 700;
```

### 12.5 Counter-Intelligence

```rust
fn evaluate_counter_intel(
    ai_state: &NationAIState,
    snapshot: &GameStateSnapshot,
) -> bool {
    // Trigger counter-intel sweep if:
    // 1. Own institutions show signs of foreign infiltration
    // 2. Detection rate of our assets by foreign nations is high
    // 3. Unexplained productivity drops in key institutions

    let infiltration_signs = snapshot.infiltration_indicators(ai_state.nation_id);
    let our_asset_detection_rate = ai_state.memory.asset_detection_rate_last_50_ticks();

    infiltration_signs > INFILTRATION_ALERT_THRESHOLD
        || our_asset_detection_rate > OWN_DETECTION_ALERT_THRESHOLD
}

const INFILTRATION_ALERT_THRESHOLD: i32 = 300;
const OWN_DETECTION_ALERT_THRESHOLD: i32 = 300;
```

### 12.6 Shadow Network Expansion (CIV-0105 Integration)

The AI expands its shadow network into target nations when the capture score of a target institution exceeds the `shadow_capture_threshold`:

```rust
fn should_expand_shadow_network(
    target_institution: EntityId,
    snapshot: &GameStateSnapshot,
    params: &PersonalityParams,
) -> bool {
    let capture_score = snapshot.institution_capture_score(target_institution);
    let institution_value = snapshot.institution_strategic_value(target_institution);

    // Expand if institution is capture-vulnerable AND strategically valuable
    capture_score > params.shadow_capture_threshold
        && institution_value > params.shadow_min_value
}
```

---

## 13. AI Observability and Debugging

### 13.1 AI Decision Event — `ai.decision.v1`

Every AI action submission is preceded by an `ai.decision.v1` event containing the full decision context:

```json
{
  "event_id": "e-2026-02-21-AI-00456",
  "event_type": "ai.decision.v1",
  "tick_number": 1234,
  "payload": {
    "nation_id": "gaul",
    "decision_type": "strategic",
    "top_candidates": [
      {
        "action": "declare_war",
        "target": "rome",
        "utility_score": 720,
        "utility_breakdown": {
          "threat_score": 250,
          "territorial_gain": 180,
          "strength_ratio": 200,
          "diplomatic_cost": -120,
          "timing_modifier": 110,
          "personality_bias": 100
        }
      },
      {
        "action": "form_alliance",
        "target": "hispania",
        "utility_score": 480,
        "utility_breakdown": {
          "relationship_score": 150,
          "economic_benefit": 120,
          "security_benefit": 180,
          "sovereignty_cost": -80,
          "personality_bias": 110
        }
      },
      {
        "action": "build_structure",
        "district": "lugdunum",
        "structure": "barracks",
        "utility_score": 340
      },
      {
        "action": "launch_espionage",
        "target": "rome",
        "operation": "intelligence_gather",
        "utility_score": 290
      },
      {
        "action": "set_policy",
        "policy": "war_economy",
        "utility_score": 270
      }
    ],
    "chosen_action": "declare_war",
    "reason_codes": ["THREAT_CRITICAL", "STRENGTH_ADVANTAGE", "TIMING_FAVORABLE"],
    "current_goals": ["ExpandTerritory:gallia_cisalpina", "SecureResources:iron"],
    "personality": "Aggressive",
    "personality_drift": {"toward_mercantile": 45, "toward_aggressive": 0},
    "mcts_used": false,
    "planning_horizon": 30
  }
}
```

### 13.2 AI Thought Panel — Debug UI

In debug mode (`civlab --debug-ai`), each AI nation exposes a thought panel in the UI showing:

```
AI Thought Panel: GAUL (Aggressive)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Current Goals (priority order):
  1. ExpandTerritory: Gallia Cisalpina    [priority: 850]
  2. SecureResources: Iron               [priority: 620]
  3. BuildMilitary: Heavy Infantry       [priority: 450]

Threat Model:
  ROME      [composite: 780 / CRITICAL] military_ratio: 820, territorial: 700
  HISPANIA  [composite: 210 / LOW     ] military_ratio: 300, territorial: 120

Opportunity Model:
  GAUL_SOUTH [takeable_value: 650, weakness: 720]

Personality: Aggressive
  war_aggression_bias: 800  |  expansion_bias: 700
  trade_weight: 200         |  alliance_seeking: 200
  risk_tolerance: 800       |  diplomatic_patience: 100
  drift: +45 toward mercantile (prosperity-driven)

Memory Summary:
  Betrayals: ROME broke trade pact (tick 950, penalty -200)
  Battle W/L: 3W / 1L (last 4 battles)
  GDP trend: +1.2% / tick (20-tick regression)

Last Decision (tick 1234):
  Chose: DECLARE_WAR on ROME (utility 720)
  Reason: THREAT_CRITICAL + STRENGTH_ADVANTAGE + TIMING_FAVORABLE

APM Budget: 42/60 actions remaining this minute
```

### 13.3 Replay Annotation

`.civreplay` files include AI decision logs for post-game analysis:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplayAIAnnotation {
    pub tick: TickNumber,
    pub nation_id: EntityId,
    pub decision: AiDecisionRecord,
    pub state_hash: [u8; 32],  // SHA256 of NationAIState for integrity verification
}

pub struct CivReplay {
    pub scenario: ScenarioConfig,
    pub player_commands: Vec<TimestampedCommand>,
    pub ai_annotations: Vec<ReplayAIAnnotation>,
    pub tick_events: Vec<TickEventLog>,
}
```

Post-game, players can open the replay and view any AI nation's decision rationale on any tick. This serves both educational (understand why the AI acted as it did) and research (analyze AI decision patterns) purposes.

### 13.4 AI Performance Metrics

The simulation exposes AI performance metrics for benchmarking and modding:

```rust
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AIPerformanceMetrics {
    pub avg_strategic_eval_ms: f32,
    pub avg_mcts_iterations_per_search: u32,
    pub avg_utility_candidates_evaluated: u32,
    pub decisions_rate_limited_pct: f32,    // % of decisions that hit APM limit
    pub memory_lookup_avg_ns: u64,
    pub goal_recompute_avg_ms: f32,
}
```

These are emitted every 100 ticks as `ai.performance.v1` events and are visible in the research mode metrics dashboard.

---

## 14. Modding API for AI

### 14.1 YAML Personality Definition

Mod authors can define custom personality archetypes in YAML:

```yaml
# my_mod/ai/personalities/zealot.yaml

name: "Zealot"
description: "Fanatically devoted to ideological purity. Will sacrifice economic efficiency for ideological conformity."
version: "1.0"
author: "my_mod_author"

parameters:
  war_aggression_bias:          600
  war_utility_threshold:        450
  diplomatic_patience:          200
  defensive_priority:           700
  expansion_bias:               500
  colonization_weight:          300
  internal_dev_weight:          400
  trade_weight:                 100    # Trade is ideologically suspect
  alliance_seeking_weight:      200    # Only with ideologically aligned nations
  treaty_offer_threshold:       650
  soft_power_weight:            800    # Aggressively spread ideology
  risk_tolerance:               750
  espionage_aggressiveness:     500

  # Custom: only ally with nations whose ideology score > 60
  alliance_ideology_filter:     600

  # Threat weights
  threat_weight_military:       400
  threat_weight_territory:      200
  threat_weight_hostility:      300   # Ideological hostility matters more
  threat_weight_espionage:      100

drift:
  # Zealots drift toward Aggressive under any legitimacy loss
  legitimacy_drift_multiplier:  1.5
  # Zealots resist prosperity drift
  prosperity_drift_resistance:  0.3

# Custom utility modifier: bonus to declare_war on ideologically opposed nations
custom_utility_hooks:
  declare_war_modifier: |
    if target.ideology_alignment < 0.3:
      return +200  # Strongly favor war against ideological enemies
    else:
      return 0
```

### 14.2 Custom AI Module — Rust Trait

Mod authors can replace the entire AI decision engine for a nation by implementing the `NationAI` trait:

```rust
/// Trait for pluggable nation AI implementations.
/// Implement this to completely replace the default utility-based AI
/// for specific nations or scenarios.
pub trait NationAI: Send + Sync {
    /// Called once per tick. Return the list of actions to submit this tick.
    /// Must not block for more than `budget_ms` milliseconds.
    fn decide_actions(
        &self,
        state: &NationAIState,
        snapshot: &SimulationStateSnapshot,
        budget_ms: u64,
    ) -> Vec<AiAction>;

    /// Called after actions are submitted. Update internal state if needed.
    fn post_tick_update(
        &mut self,
        state: &NationAIState,
        outcome: &TickOutcome,
    );

    /// Human-readable name for debug panels.
    fn name(&self) -> &'static str;

    /// Whether this AI guarantees determinism.
    /// If false, replay verification is disabled for nations using this AI.
    fn is_deterministic(&self) -> bool { true }
}

/// Registration: call this in mod initialization.
pub fn register_nation_ai(nation_id: EntityId, ai: Box<dyn NationAI>);
```

Example skeleton:

```rust
pub struct MyCustomAI {
    rng: ChaCha20Rng,
}

impl NationAI for MyCustomAI {
    fn decide_actions(
        &self,
        state: &NationAIState,
        snapshot: &SimulationStateSnapshot,
        _budget_ms: u64,
    ) -> Vec<AiAction> {
        // Your custom decision logic here
        vec![]
    }

    fn post_tick_update(&mut self, _state: &NationAIState, _outcome: &TickOutcome) {}

    fn name(&self) -> &'static str { "MyCustomAI" }
}
```

### 14.3 Python Scripting — Callback Hooks

The Python scripting API (available in research mode and sandboxed play) provides callback hooks for AI behavior:

```python
import civlab.ai

# Hook: called every time the AI considers declaring war
@civlab.ai.register_handler("on_war_consideration")
def my_war_handler(nation_id: str, target_id: str, utility_score: int, state: dict) -> int:
    """
    Modify the utility score for war consideration.
    Return the adjusted utility score (int in [-1000, 1000]).
    Return None to use the default score.
    """
    # Example: penalize wars against allied nations
    if state["relations"][target_id]["score"] > 600:
        return utility_score - 500  # Strong penalty
    return utility_score

# Hook: called when AI builds a goal list
@civlab.ai.register_handler("on_goal_recompute")
def my_goal_handler(nation_id: str, goals: list, state: dict) -> list:
    """
    Modify or replace the goal list.
    Return the modified goals list.
    """
    # Example: force research goal to always be present
    goals.append({
        "type": "ResearchTech",
        "tech_id": "industrial_revolution",
        "priority": 500
    })
    return goals

# Hook: called after AI submits actions
@civlab.ai.register_handler("on_action_submitted")
def on_action(nation_id: str, action: dict, tick: int):
    """Post-submission observer. Cannot modify the action."""
    print(f"[Tick {tick}] {nation_id} submitted: {action['action_type']}")
```

Available hook types:

| Hook Name | When Called | Can Modify |
|---|---|---|
| `on_war_consideration` | AI evaluates war declaration | Utility score |
| `on_treaty_consideration` | AI evaluates treaty offer | Utility score |
| `on_goal_recompute` | AI rebuilds goal list | Goal list |
| `on_mission_assignment` | Operational AI assigns mission | Mission type |
| `on_espionage_selection` | AI selects covert operation | Operation choice |
| `on_action_submitted` | Action submitted to engine | Read-only |
| `on_threat_model_updated` | Threat model recomputed | Threat scores |
| `on_personality_drift` | Drift engine fires | Drift magnitude |

### 14.4 Utility Override — Per-Action Type

Mod authors can override the utility function for any action type without replacing the full AI:

```rust
/// Register a custom utility modifier for a specific action type.
/// The modifier receives the default utility score and may return a new score.
pub fn register_utility_modifier(
    action_type: AiActionType,
    modifier: Box<dyn Fn(i32, &NationAIState, &SimulationStateSnapshot) -> i32 + Send + Sync>,
);

// Usage:
register_utility_modifier(
    AiActionType::DeclareWar,
    Box::new(|default_utility, ai_state, snapshot| {
        // Example: reduce war utility during winter ticks
        if snapshot.is_winter_tick() {
            (default_utility * 800 / 1000).max(-1000)
        } else {
            default_utility
        }
    }),
);
```

### 14.5 Custom Scenario AI Configuration

Scenario YAML files can specify per-nation AI configurations:

```yaml
# scenarios/my_scenario.yaml

nations:
  rome:
    name: "Roman Republic"
    ai_module: "default"          # Use standard utility AI
    ai_personality: "Aggressive"  # Override hash-derived personality
    ai_params_override:           # Override specific parameters
      war_aggression_bias: 900    # Very aggressive
      expansion_bias: 950

  carthage:
    name: "Carthage"
    ai_module: "my_mod:carthage_ai"  # Use custom Rust trait implementation
    ai_personality: "Mercantile"

  sparta:
    name: "Sparta"
    ai_module: "default"
    ai_personality: "custom:zealot"  # Use custom YAML personality
```

---

## 15. FR Traceability

### 15.1 FR-CIV-RTS-014 — Faction AI Behavior and Decision Making

**Requirement Text:** AI factions issue military commands autonomously per tick based on faction policy. AI evaluates: threat (military balance vs player), opportunity (undefended enemies), resource state (can afford new units?). AI decisions SHALL be deterministic given policy and state.

| Acceptance Criterion | Addressed By | Section |
|---|---|---|
| AI policy: {threat_tolerance, opportunity_threshold, resource_spending_rate, preferred_tactics} | `PersonalityParams` struct with all equivalent parameters | §3.1, §4.1 |
| Threat eval: compare own vs player military strength; if ratio \< threat_tolerance, raise alert | `compute_threat_score()`, `ThreatScore.military_ratio` | §3.3 |
| Defense: if threatened and structures \< threshold, spawn defensive units | `assign_mission()` Defend branch; goal `BuildMilitary` | §6.2, §3.5 |
| Attack: if opportunity and resources available, queue attack order | `assign_mission()` Attack branch; `select_next_building()` resource check | §6.2 |
| Diplomacy: if heavily outnumbered, may propose peace/trade, form alliances | `evaluate_war_declaration()`, `propose_alliance()`, `evaluate_peace_offer()` | §8.3, §8.4, §8.6 |
| Determinism: identical faction state + policy + seed → identical decisions | `ai_rng_for_tick()` ChaCha20Rng seeding; no f64; BTreeMap iteration | §2.4 |

### 15.2 Related Requirements

| FR ID | Description | Addressed By | Section |
|---|---|---|---|
| FR-CIV-RTS-013 | Unit experience and leveling | Morale management, `update_unit_morale()` | §7.4 |
| FR-CIV-RTS-012 | Real-time and turn-based modes | APM governor, tick-rate-independent design | §2.5, §10.2 |
| FR-CIV-RTS-015 | Client-side prediction | AI uses same WebSocket interface as clients | §2.2 |

### 15.3 CIV-0105 Integration Points

The AI integrates with the war/diplomacy spec (CIV-0105) at these precise points:

| CIV-0105 Concept | AI Integration |
|---|---|
| `DiplomaticState` FSM | AI reads state to compute `diplomatic_hostility` in ThreatScore |
| Transition `Escalating → ActiveConflict` | Triggered by AI `declare_war` action submission |
| Treaty system | AI submits `sign_treaty` actions; treaty violations update `AIMemory.betrayal_log` |
| Shadow network `capture_score` | Espionage AI reads to decide `ShadowNetworkExpand` |
| `coalition_stability` | AI reads to calibrate alliance-seeking weight |
| `grievance_score` | AI monitors to anticipate diplomatic state transitions |

### 15.4 CIV-0100 Integration Points

The AI integrates with the economy spec (CIV-0100) at these points:

| CIV-0100 Concept | AI Integration |
|---|---|
| Conservation equation | AI reads `supply_stress` to detect scarcity; triggers `SecureResources` goal |
| Joule reserves | `joule_cost_penalty` in utility functions; APM governor budget check |
| GDP trend | `extrapolate_gdp_trend()` uses economic snapshots for planning |
| Allocation regime | AI `set_policy_utility` evaluates policy effects on economic output |
| Double-entry ledger | AI cannot bypass; all transactions go through standard command interface |

### 15.5 CIV-0103 Integration Points

The AI integrates with the institutions spec (CIV-0103) at these points:

| CIV-0103 Concept | AI Integration |
|---|---|
| `InstitutionState` FSM | AI reads institution states to assess governance health; `SuppressInsurgency` goal triggered by `Collapsed` institutions |
| Legitimacy model | Personality drift triggered by legitimacy drops below 0.30 |
| Citizen lifecycle `dissenting` | High dissent triggers `DevelopEconomy` or `SuppressInsurgency` goal |
| Capture vulnerability | Espionage AI targets institutions with high `capture_score` |
| Policy capacity multiplier | AI factors policy capacity into `set_policy_utility` |

### 15.6 Game Pillar Alignment

| Design Pillar | AI Implementation |
|---|---|
| Pillar 1: Determinism & Replay | ChaCha20Rng seeding; no f64; BTreeMap iteration; `ai.decision.v1` events in replay |
| Pillar 3: Emergent Complexity | AI acts as genuine player; complex behaviors emerge from goal interaction |
| Pillar 4: Multi-Layer Play | Three AI granularity levels map exactly to Strategic/Operational/Tactical zoom levels |
| Pillar 5: Modding & Extensibility | YAML personality, Rust trait, Python hooks, utility override API |

---

## 16. Acceptance Criteria

The following criteria must pass before CIV-0400 is considered implemented:

### 16.1 Determinism

- [ ] Run scenario `test_ai_determinism` with seed `0xDEADBEEF` twice. Event logs for `ai.decision.v1` must be byte-identical.
- [ ] Run on Linux x86_64 and macOS ARM64 with same seed; AI decision logs must be identical.
- [ ] Verify no `f64` appears in any utility computation path (CI check: `grep -r "f64" crates/ai/`).
- [ ] Verify `ChaCha20Rng` is the only RNG used in AI code paths.

### 16.2 Fair Play

- [ ] AI nations submit all actions via `WebSocketAiSubmitter`. No direct calls to simulation internals from AI code.
- [ ] Fog of war at difficulty 1–4: AI threat model uses estimates, not true values. Verify with test `test_fog_of_war_ai`.
- [ ] At difficulty 5 ("perfect intel"), log `ai.leadership_bonus.v1` event disclosing the bonus to player.

### 16.3 Personality and Behavior

- [ ] Aggressive AI declares war within 50 ticks when strength_ratio > 1.2 and casus_belli valid.
- [ ] Diplomatic AI proposes treaty when relationship_score > 300 and no recent conflict.
- [ ] Isolationist AI does not initiate offensive wars for 200 ticks in `test_isolationist_peace`.
- [ ] Mercantile AI establishes at least 3 trade agreements before tick 100 in `test_mercantile_trade`.
- [ ] Personality drift: Aggressive AI drifts toward Mercantile after 200 ticks of high prosperity.

### 16.4 Memory

- [ ] Betrayal penalty applied immediately to `ally_reliability` after `treaty.violated.v1` event.
- [ ] Battle learning: war_utility_threshold increases after 3 consecutive defeats.
- [ ] Economic extrapolation: AI prioritizes `DevelopEconomy` when `extrapolate_gdp_trend` returns negative slope.
- [ ] Memory respects `max_battle_memory` and `max_economic_memory` bounds from difficulty config.

### 16.5 Military Operational AI

- [ ] Army groups form correctly: units within 3 hexes of same nation grouped together.
- [ ] Supply decay: army at 0 supply reduces effective_strength to &lt; 300 within 5 ticks.
- [ ] Retreat triggered when `effective_strength \< 0.4 × initial_strength`.
- [ ] A* pathfinding respects terrain costs and zone of control penalties.

### 16.6 Difficulty Scaling

- [ ] APM governor enforces rate limits: Novice AI submits &lt; 5 actions/minute in `test_apm_novice`.
- [ ] MCTS enabled only at difficulty 4–5: test `mcts_disabled_at_difficulty_3`.
- [ ] Resource bonus: at Legendary, AI production output is 25% higher than at Advanced on same map.
- [ ] `ai.leadership_bonus.v1` event emitted every tick at difficulty 4–5.

### 16.7 Espionage AI

- [ ] Espionage operation selected only when `launch_espionage_utility > espionage_launch_threshold`.
- [ ] Asset rotation triggered when `detection_score > 700`.
- [ ] Counter-intel sweep triggered when `infiltration_indicators > 300`.
- [ ] Shadow network expansion attempted when target institution `capture_score > shadow_capture_threshold`.

### 16.8 Observability

- [ ] `ai.decision.v1` event emitted on every AI action submission with top-5 candidates and utility breakdown.
- [ ] AI thought panel visible in `civlab --debug-ai` mode.
- [ ] `.civreplay` files contain full `ReplayAIAnnotation` sequence; post-game AI review playback works.
- [ ] `ai.performance.v1` metrics emitted every 100 ticks in research mode.

### 16.9 Modding API

- [ ] Custom YAML personality loaded and applied: `civlab --mod my_mod test_personality zealot`.
- [ ] Custom Rust `NationAI` trait implementation registered and called: `test_custom_ai_trait`.
- [ ] Python callback hook `on_war_consideration` fires and modifies utility score: `test_python_hook`.
- [ ] `register_utility_modifier` override applied and reflected in `ai.decision.v1` utility breakdown.

### 16.10 MCTS

- [ ] MCTS terminates within `mcts_compute_budget_ms` on all tested hardware.
- [ ] MCTS output is deterministic: same root state + rng_seed → same best action.
- [ ] MCTS reward function produces non-trivial variation across personality archetypes.
- [ ] MCTS thread does not block simulation tick: tick time \< 50ms overhead when MCTS active.

---

*End of CIV-0400: AI / NPC Behavior Specification v1*
