# RND-011: MCTS for Game AI -- Implementation Approach for CivLab Nation AI

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**MCTS is feasible for CivLab's difficulty 4-5 AI** with the following key design decisions:

1. **Paranoid MCTS** (not max-n or coalition) for multi-nation scenarios: treats all opponents
   as adversarial, producing robust/conservative play appropriate for a strategy game AI.
2. **Node-count bounded** (not time-bounded) for determinism: budget of N=5000-10000 nodes
   per decision, yielding consistent behavior regardless of hardware speed.
3. **Compressed state representation** (~10KB per MCTS node): extract only nation-relevant
   state (economy summary, military units, diplomatic relations, resource levels) -- not the
   full ECS World.
4. **Action space pruning** via utility-scored pre-selection: limit to top-K=20 candidate
   actions per node (from potentially 1000+ legal actions) using a fast heuristic evaluator.
5. **Simplified rollout policy**: 10-tick lookahead using a lightweight "fast-forward" model
   that approximates the full simulation with ~5 key equations (GDP growth, military strength
   delta, food balance, research progress, diplomatic tension).
6. **No parallelization for determinism**: single-threaded MCTS. The 100ms budget is met by
   limiting node count, not by parallelizing.

---

## Research Findings

### 1. MCTS Fundamentals for Strategy Games

#### 1.1 Standard MCTS (Single-Player / Two-Player)

The classic MCTS algorithm has four phases per iteration:
1. **Selection**: Traverse tree from root, picking children via UCB1 (or variant).
2. **Expansion**: Add a new child node for an unexplored action.
3. **Rollout (Simulation)**: Play random/heuristic moves from the new node to a terminal
   state or depth limit.
4. **Backpropagation**: Update win/visit statistics from the new node back to root.

UCB1 selection formula:
```
UCB1(child) = Q(child)/N(child) + C * sqrt(ln(N(parent)) / N(child))
```
Where Q = total reward, N = visit count, C = exploration constant (typically sqrt(2)).

#### 1.2 Multi-Player MCTS Variants

CivLab has 2-8 nations. Standard 2-player MCTS (minimax assumption) doesn't generalize
directly. Three main approaches:

| Variant | Description | Pros | Cons |
|---------|-------------|------|------|
| **Max-n** | Each node stores per-player rewards. Selection maximizes current player's reward. | Theoretically optimal for n-player games. | Assumes opponents play optimally for themselves. In practice, opponents may form coalitions or play suboptimally, making max-n overfit. |
| **Paranoid** | All opponents modeled as a single adversary trying to minimize the AI's reward. Reduces to 2-player minimax. | Conservative, robust play. Simple implementation (just negate reward for opponent turns). Good when opponents are threatening. | May miss cooperative/exploitative opportunities. Overly defensive in games with natural alliances. |
| **Coalition** | Dynamically models alliances. Opponents split into "with us" and "against us" groups. | More realistic for diplomacy-heavy games. | Complex implementation. Coalition detection is itself a hard problem. Unstable coalitions cause tree inconsistency. |

**Recommendation: Paranoid MCTS.**

Rationale:
- CivLab's AI difficulties 4-5 should play **competitively**, not exploitatively. Paranoid
  assumption produces an AI that defends well and doesn't take foolish risks.
- Coalition dynamics in CivLab are handled by the diplomacy system (RND-TBD), not by MCTS
  search. The AI's diplomatic decisions (ally, trade, declare war) are actions in the MCTS
  tree, not structural assumptions.
- Max-n is theoretically better but requires accurate modeling of each opponent's utility
  function. In practice, the AI doesn't know opponents' goals precisely enough for max-n
  to outperform paranoid.
- Implementation simplicity: paranoid MCTS is identical to 2-player MCTS with opponent turns
  interleaved.

Research backing: Nijssen's thesis on multi-player MCTS found that paranoid MCTS performs
comparably to max-n in most multi-player games, and significantly better when opponents have
hidden information or are modeled imprecisely.

#### 1.3 UCB1 vs PUCT

| Algorithm | Formula | Use Case |
|-----------|---------|----------|
| **UCB1** | `Q/N + C * sqrt(ln(N_parent) / N)` | No prior knowledge about action quality |
| **PUCT** | `Q/N + C * P(a) * sqrt(N_parent) / (1 + N)` | With prior probability P(a) from a heuristic/network |

**Recommendation: PUCT** (Polynomial UCT variant, as used in AlphaGo/AlphaZero).

Rationale:
- CivLab's action space is large (100-1000+ actions per turn). With UCB1, the algorithm must
  visit every action at least once before focusing -- with 1000 actions, the first 1000
  iterations are wasted on uniform exploration.
- PUCT uses a **prior probability** `P(a)` for each action, which biases exploration toward
  promising actions immediately. The prior comes from our utility-scoring heuristic (see
  Section 3), not from a neural network.
- PUCT with a good heuristic prior dramatically improves search efficiency in large action
  spaces. AlphaGo showed this; it applies equally to strategy games.

Modified PUCT for CivLab:
```
PUCT(a) = Q(a)/N(a) + C_puct * P(a) * sqrt(N_parent) / (1 + N(a))
```
Where:
- `P(a)` = prior probability from heuristic utility scoring (normalized to sum to 1.0)
- `C_puct` = exploration constant, tunable (start with 1.5, tune via self-play)
- `Q(a)` = average reward (fixed-point, see RND-003)
- `N(a)` = visit count for action a
- `N_parent` = visit count for parent node

### 2. State Representation

#### 2.1 The Full State Problem

CivLab's full simulation state includes:
- All entities in the ECS World (10k-100k entities with multiple components each)
- Terrain map (hex grid with per-tile data)
- Diplomatic relations (N x N matrix)
- Technology trees (per nation)
- Event queues, RNG state, tick counter

**Estimated size:** 1-10MB for a mid-game state. Copying this per MCTS node is infeasible
for 10k nodes.

#### 2.2 Compressed Nation State

For MCTS lookahead, the AI doesn't need full simulation fidelity. It needs to estimate
the **relative advantage** of different strategic choices. A compressed representation:

```rust
/// Compressed state for MCTS node. ~10KB total.
#[derive(Clone, Debug)]
pub struct MctsState {
    /// Which nation is making the decision
    pub acting_nation: NationId,

    /// Per-nation summaries (2-8 nations)
    pub nations: Vec<NationSummary>,

    /// Simplified diplomatic relations
    pub relations: Vec<(NationId, NationId, RelationScore)>,

    /// Current game tick
    pub tick: u64,

    /// Deterministic RNG state for rollouts
    pub rng_seed: u64,
}

/// Summary of a single nation's state. ~1KB per nation.
#[derive(Clone, Debug)]
pub struct NationSummary {
    pub id: NationId,
    pub population: i64,
    pub gdp: i64,                    // milli-credits
    pub food_balance: i64,           // kJ surplus/deficit per tick
    pub military_strength: i64,      // aggregate military power score
    pub research_progress: i64,      // total research points
    pub territory_size: i32,         // number of controlled hexes
    pub happiness: i32,              // fixed-point (Ratio bits)
    pub strategic_resources: [i64; 8], // key resource stockpiles
}

/// Diplomatic relation score between two nations.
/// Negative = hostile, positive = friendly.
pub type RelationScore = i32;
```

**Size analysis:**
- `NationSummary`: ~(8 + 8 + 8 + 8 + 8 + 8 + 4 + 4 + 64) = ~120 bytes per nation
- 8 nations: ~960 bytes
- Relations: 8*7/2 = 28 pairs * 12 bytes = ~336 bytes
- Overhead: ~200 bytes
- **Total: ~1.5 KB per MCTS node** (much better than the 10KB estimate)

At 10,000 nodes: ~15 MB total. Acceptable.

#### 2.3 State Extraction

```rust
/// Extract compressed MCTS state from the full ECS World.
/// Called once at the start of each AI decision.
pub fn extract_mcts_state(
    world: &World,
    acting_nation: NationId,
    tick: u64,
    rng_seed: u64,
) -> MctsState {
    let mut nations = Vec::new();

    // Query all nation entities and their summary components
    let mut query = world.query::<(
        &Nation,
        &Economy,
        &Military,
        &Research,
        &Territory,
        &Happiness,
        &ResourceStockpile,
    )>();

    for (nation, economy, military, research, territory, happiness, resources) in query.iter(world) {
        nations.push(NationSummary {
            id: nation.id,
            population: economy.population,
            gdp: economy.gdp,
            food_balance: economy.food_balance_per_tick,
            military_strength: military.aggregate_strength(),
            research_progress: research.total_points,
            territory_size: territory.hex_count,
            happiness: happiness.score.to_bits(),
            strategic_resources: resources.summarize(),
        });
    }

    // Sort nations by ID for determinism
    nations.sort_by_key(|n| n.id);

    // Extract diplomatic relations
    let relations = extract_relations(world);

    MctsState { acting_nation, nations, relations, tick, rng_seed }
}
```

### 3. Action Space and Pruning

#### 3.1 Action Types in CivLab

A nation's available actions per decision point include:

| Category | Example Actions | Cardinality |
|----------|-----------------|-------------|
| Military | Move unit, attack, recruit, fortify | ~50-500 (depends on army size) |
| Economic | Build improvement, set tax rate, trade | ~20-100 |
| Diplomatic | Propose alliance, declare war, trade deal | ~10-50 |
| Research | Choose tech, prioritize branch | ~5-20 |
| Domestic | Set policy, assign governor, event response | ~10-30 |
| **Total** | | **~100-700 per turn** |

With compound actions (multiple orders per turn), the space grows combinatorially.

#### 3.2 Pruning Strategy: Utility-Scored Pre-Selection

**Do not expand all actions in the MCTS tree.** Instead:

1. **Generate all legal actions** for the acting nation.
2. **Score each action** with a fast heuristic utility function (~1us per action).
3. **Select top-K actions** (K=15-25) to include in the MCTS tree.
4. The remaining actions are **pruned** -- never explored by MCTS.

This is conceptually similar to PUCT's prior probability but applied as a hard cutoff
rather than a soft bias.

```rust
/// Score an action using a fast heuristic. Higher = more promising.
/// This is NOT the MCTS reward -- it's a prior estimate for action selection.
pub fn score_action(state: &MctsState, action: &Action) -> i64 {
    match action {
        Action::Attack { target, strength } => {
            // Heuristic: attack value = expected damage - expected loss
            let target_defense = state.nation_summary(target.nation).military_strength;
            let advantage = *strength as i64 - target_defense;
            advantage * 100 // Scale to make comparable with other actions
        }
        Action::BuildImprovement { hex, improvement_type } => {
            // Heuristic: build value = expected yield increase
            improvement_type.expected_yield_increase() * 50
        }
        Action::DeclareWar { target } => {
            // Heuristic: war value based on relative power
            let us = state.nation_summary(state.acting_nation);
            let them = state.nation_summary(*target);
            (us.military_strength - them.military_strength) * 80
        }
        Action::Research { tech } => {
            // Heuristic: research value = tech's strategic weight
            tech.strategic_value() * 60
        }
        // ... other action types
        _ => 0, // Default: neutral priority
    }
}

/// Select top-K actions for MCTS expansion.
pub fn select_candidate_actions(
    state: &MctsState,
    all_actions: &[Action],
    k: usize,
) -> Vec<(Action, i64)> {
    let mut scored: Vec<_> = all_actions.iter()
        .map(|a| (a.clone(), score_action(state, a)))
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Take top K
    scored.truncate(k);
    scored
}
```

#### 3.3 Prior Probability for PUCT

Convert utility scores to probabilities for PUCT:

```rust
/// Convert raw utility scores to PUCT prior probabilities.
/// Uses softmax-like normalization (integer approximation).
pub fn scores_to_priors(scored_actions: &[(Action, i64)]) -> Vec<(Action, i32)> {
    if scored_actions.is_empty() {
        return Vec::new();
    }

    // Shift scores so minimum is 0 (avoid negative in "softmax")
    let min_score = scored_actions.iter().map(|(_, s)| *s).min().unwrap_or(0);
    let shifted: Vec<i64> = scored_actions.iter().map(|(_, s)| s - min_score + 1).collect();

    // Normalize to sum to 1000 (fixed-point probability, 0.1% resolution)
    let total: i64 = shifted.iter().sum();
    scored_actions.iter()
        .zip(shifted.iter())
        .map(|((action, _), &s)| {
            let prior = (s * 1000 / total) as i32; // 0-1000 range
            (action.clone(), prior.max(1)) // minimum prior of 1 (0.1%)
        })
        .collect()
}
```

### 4. Rollout Policy

#### 4.1 The Rollout Problem

Standard MCTS rollouts play to a terminal state using random moves. For CivLab:
- A game can last 1000+ ticks.
- Each tick involves the full simulation (ECS systems, climate, economy, military...).
- Running even 10 ticks of the full simulation per MCTS node is too expensive (10ms per
  tick * 10 ticks * 10,000 nodes = 1000 seconds).

#### 4.2 Fast-Forward Approximation

Instead of running the full ECS simulation, use a **simplified mathematical model** that
approximates 10 ticks of game progression in ~1-10us:

```rust
/// Fast-forward the compressed state by `ticks` steps.
/// This is a simplified model -- NOT the full simulation.
/// Accuracy is secondary to speed; the MCTS statistics average out errors.
pub fn fast_forward(state: &mut MctsState, ticks: u32, rng: &mut DeterministicRng) {
    for _ in 0..ticks {
        for nation in &mut state.nations {
            fast_forward_nation(nation, &state.relations, rng);
        }
        fast_forward_relations(&mut state.relations, rng);
        state.tick += 1;
    }
}

/// Simplified per-nation tick. ~5 key equations.
fn fast_forward_nation(
    nation: &mut NationSummary,
    relations: &[(NationId, NationId, RelationScore)],
    rng: &mut DeterministicRng,
) {
    // 1. Population growth: logistic model
    //    pop_delta = growth_rate * pop * (1 - pop/carrying_capacity)
    let carrying_capacity = nation.territory_size as i64 * 1000; // rough estimate
    let growth_rate: i64 = 5; // 0.5% per tick (scaled by 1000)
    let pop_delta = growth_rate * nation.population / 1000
        * (carrying_capacity - nation.population) / carrying_capacity;
    nation.population = (nation.population + pop_delta).max(0);

    // 2. GDP growth: proportional to population and research
    let gdp_growth_rate = 10 + nation.research_progress / 10000; // base 1% + tech bonus
    nation.gdp += nation.gdp * gdp_growth_rate / 1000;

    // 3. Food balance: territory * base_yield - population * consumption_rate
    let food_production = nation.territory_size as i64 * 500; // kJ per hex per tick
    let food_consumption = nation.population * 3; // 3 kJ per person per tick
    nation.food_balance = food_production - food_consumption;

    // 4. Military: slow decay if not at war, slow growth from GDP
    let military_budget = nation.gdp / 100; // 10% of GDP
    nation.military_strength += military_budget / 1000 - nation.military_strength / 500;

    // 5. Happiness: function of food balance and military safety
    let food_factor = (nation.food_balance / 100).clamp(-100, 100);
    nation.happiness = (nation.happiness as i64 + food_factor).clamp(0, 1000) as i32;
}

/// Simplified diplomatic evolution.
fn fast_forward_relations(
    relations: &mut [(NationId, NationId, RelationScore)],
    rng: &mut DeterministicRng,
) {
    for (_, _, score) in relations.iter_mut() {
        // Relations drift toward neutral with small random perturbation
        let drift = -(*score / 100); // mean reversion
        let noise = rng.next_range(-5, 5);
        *score = (*score + drift + noise).clamp(-1000, 1000);
    }
}
```

#### 4.3 Rollout Evaluation

After fast-forwarding, evaluate the resulting state:

```rust
/// Evaluate the state from the perspective of the acting nation.
/// Returns a score in [0, 1000] where 1000 = winning, 0 = losing.
pub fn evaluate_state(state: &MctsState) -> i32 {
    let us = state.nation_summary(state.acting_nation);
    let max_gdp = state.nations.iter().map(|n| n.gdp).max().unwrap_or(1);
    let max_mil = state.nations.iter().map(|n| n.military_strength).max().unwrap_or(1);
    let max_pop = state.nations.iter().map(|n| n.population).max().unwrap_or(1);

    // Weighted relative standing
    let gdp_score = (us.gdp * 300 / max_gdp.max(1)) as i32;        // 0-300
    let mil_score = (us.military_strength * 300 / max_mil.max(1)) as i32; // 0-300
    let pop_score = (us.population * 200 / max_pop.max(1)) as i32;  // 0-200
    let hap_score = us.happiness / 5;                                  // 0-200

    (gdp_score + mil_score + pop_score + hap_score).clamp(0, 1000)
}
```

### 5. Deterministic Bounding

#### 5.1 Why Not Wall-Clock Time

```rust
// BAD: Non-deterministic -- varies by hardware speed
while start.elapsed() < Duration::from_millis(100) {
    mcts.iterate();
}

// GOOD: Deterministic -- always same number of iterations
for _ in 0..NODE_BUDGET {
    mcts.iterate();
}
```

Using `std::time::Instant` makes the MCTS output depend on CPU speed. A fast machine
explores more nodes and makes better decisions than a slow machine -- breaking determinism
and creating unfair multiplayer.

#### 5.2 Node Budget Calibration

Target: complete MCTS within ~100ms on the reference hardware (mid-range 2024 CPU, single
thread).

**Per-node cost estimate:**
- State copy: ~1.5KB memcpy = ~100ns
- Action generation + scoring: ~5-20us (20 actions * 1us each)
- PUCT selection: ~1us (scan 20 children)
- Rollout (10 ticks fast-forward): ~10-50us
- Backpropagation: ~0.5us
- **Total per node: ~20-70us**

**Budget calculation:**
- 100ms / 50us average = ~2,000 nodes minimum
- 100ms / 20us average = ~5,000 nodes maximum
- **Recommended budget: 5,000 nodes** for difficulty 4
- **Recommended budget: 10,000 nodes** for difficulty 5

At 10,000 nodes * 50us = 500ms -- this exceeds 100ms on average hardware. Options:
1. Accept 200-500ms AI turns at difficulty 5 (strategy games are turn-based, players wait).
2. Reduce rollout depth from 10 to 5 ticks.
3. Reduce K from 20 to 10 candidate actions.
4. Optimize fast-forward model.

**Recommendation:** Start with 5,000 nodes, profile, and tune.

### 6. Parallelization Analysis

#### 6.1 Parallelization Approaches

| Method | Description | Deterministic? | Speedup |
|--------|-------------|---------------|---------|
| **Root parallelization** | Run N independent MCTS trees, merge statistics. | Yes (if merge is deterministic) | ~linear in N |
| **Leaf parallelization** | Parallelize rollouts at leaf nodes. | Yes (each rollout independent) | ~linear in batch size |
| **Tree parallelization** | Multiple threads traverse/expand same tree with locks or virtual loss. | NO -- thread scheduling affects exploration order. | Best theoretical speedup but non-deterministic. |

#### 6.2 Recommendation: No Parallelization

For CivLab, **do not parallelize MCTS**. Rationale:

1. **Determinism is the top priority.** Tree parallelization is inherently non-deterministic.
   Root parallelization can be deterministic but doubles memory usage.
2. **Budget is node-count-based, not time-based.** Parallelism doesn't help with a fixed
   node budget -- it just finishes faster. Since we're not time-bounded, there's no benefit.
3. **Simplicity.** Single-threaded MCTS is dramatically easier to debug, test, and reason
   about.
4. **MCTS runs on AI turn, not every frame.** A 200ms AI decision in a turn-based game is
   imperceptible. Parallelism solves a non-problem.

If performance becomes an issue in the future (e.g., real-time mode), root parallelization
with deterministic merging is the safe upgrade path.

### 7. Deterministic RNG

MCTS rollouts require randomness for the rollout policy. This must be deterministic:

```rust
/// Deterministic PRNG for MCTS rollouts.
/// Uses xorshift64 for speed and simplicity.
/// Seeded per MCTS search from the game's master RNG.
#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) } // Avoid zero state
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Random value in [min, max] inclusive.
    pub fn next_range(&mut self, min: i64, max: i64) -> i64 {
        let range = (max - min + 1) as u64;
        min + (self.next() % range) as i64
    }
}
```

---

## Decision

Implement **Paranoid MCTS with PUCT selection, node-count bounding, compressed state,
utility-scored action pruning, and simplified rollout model.** Single-threaded, fully
deterministic. No neural network -- heuristic priors only.

---

## Implementation Contract

### Data Structures

```rust
/// MCTS tree node.
#[derive(Debug)]
pub struct MctsNode {
    /// The action that led to this node (None for root).
    pub action: Option<Action>,

    /// Visit count.
    pub visits: u32,

    /// Total reward accumulated (integer, 0-1000 scale per visit).
    pub total_reward: i64,

    /// Prior probability from heuristic (0-1000 scale, see scores_to_priors).
    pub prior: i32,

    /// Children (expanded actions).
    pub children: Vec<MctsNode>,

    /// Compressed game state at this node.
    /// Only stored for expanded nodes. None for leaf/unexpanded.
    pub state: Option<MctsState>,

    /// Whether this is the acting nation's turn (for paranoid negation).
    pub is_acting_turn: bool,
}
```

### Core MCTS Loop

```rust
/// Top-level MCTS search. Returns the best action.
pub fn mcts_search(
    initial_state: MctsState,
    node_budget: u32,
    rollout_depth: u32,
    top_k_actions: usize,
    c_puct: i32, // exploration constant, 0-1000 scale (1500 = 1.5)
) -> Action {
    let mut root = MctsNode::new_root(initial_state.clone());

    // Expand root with candidate actions
    let actions = generate_actions(&initial_state);
    let candidates = select_candidate_actions(&initial_state, &actions, top_k_actions);
    let priors = scores_to_priors(&candidates);
    root.expand(priors, &initial_state);

    // Main MCTS loop: fixed node count for determinism
    let mut rng = DeterministicRng::new(initial_state.rng_seed);

    for _ in 0..node_budget {
        // Selection: traverse tree using PUCT
        let path = select_leaf(&mut root, c_puct);

        // Expansion: add children to leaf if not terminal
        let leaf = follow_path_mut(&mut root, &path);
        if leaf.visits > 0 && leaf.children.is_empty() {
            let leaf_state = leaf.state.as_ref().unwrap();
            let leaf_actions = generate_actions(leaf_state);
            let candidates = select_candidate_actions(leaf_state, &leaf_actions, top_k_actions);
            let priors = scores_to_priors(&candidates);
            leaf.expand(priors, leaf_state);
        }

        // Rollout: fast-forward from leaf state
        let leaf = follow_path(&root, &path);
        let mut rollout_state = leaf.state.as_ref().unwrap().clone();
        fast_forward(&mut rollout_state, rollout_depth, &mut rng);
        let reward = evaluate_state(&rollout_state);

        // Backpropagation: update statistics along the path
        backpropagate(&mut root, &path, reward);
    }

    // Select best action: highest visit count (most robust)
    root.best_child_action()
}
```

### PUCT Selection

```rust
/// Select a leaf node using PUCT. Returns path of child indices.
fn select_leaf(root: &MctsNode, c_puct: i32) -> Vec<usize> {
    let mut path = Vec::new();
    let mut node = root;

    while !node.children.is_empty() {
        let parent_visits = node.visits;
        let sqrt_parent = integer_sqrt(parent_visits as i64);

        let best_idx = node.children.iter().enumerate()
            .max_by_key(|(_, child)| {
                if child.visits == 0 {
                    // Unvisited: high priority, biased by prior
                    return i64::MAX / 2 + child.prior as i64;
                }

                // Q value: average reward (0-1000 scale)
                let q = child.total_reward / child.visits as i64;

                // Paranoid: negate reward for opponent turns
                let q_adjusted = if node.is_acting_turn { q } else { 1000 - q };

                // PUCT exploration term
                let exploration = c_puct as i64 * child.prior as i64
                    * sqrt_parent / (1000 * (1 + child.visits as i64));

                q_adjusted + exploration
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        path.push(best_idx);
        node = &node.children[best_idx];
    }

    path
}

/// Integer square root (floor). Deterministic, no f64.
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

### Backpropagation

```rust
/// Backpropagate reward along the path from leaf to root.
fn backpropagate(root: &mut MctsNode, path: &[usize], reward: i32) {
    root.visits += 1;
    root.total_reward += reward as i64;

    let mut node = root;
    for &idx in path {
        node = &mut node.children[idx];
        node.visits += 1;
        node.total_reward += reward as i64;
    }
}
```

### Best Action Selection

```rust
impl MctsNode {
    /// Select the action with the highest visit count (most robust selection).
    /// Ties broken by total reward (deterministic due to sorted comparison).
    pub fn best_child_action(&self) -> Action {
        self.children.iter()
            .max_by_key(|child| (child.visits, child.total_reward))
            .expect("MCTS root has no children")
            .action.clone()
            .expect("Root child has no action")
    }
}
```

### Integration with ECS

```rust
use bevy_ecs::prelude::*;

/// System that runs MCTS AI for nations at difficulty 4-5.
/// Called once per nation per AI decision point.
pub fn run_mcts_ai(
    world: &World,
    nation_id: NationId,
    difficulty: u8,
    tick: u64,
    rng: &mut DeterministicRng,
) -> Vec<Action> {
    // Difficulty-based parameters
    let (node_budget, rollout_depth, top_k) = match difficulty {
        4 => (5_000, 8, 15),
        5 => (10_000, 10, 20),
        _ => unreachable!("MCTS only for difficulty 4-5"),
    };

    // Extract compressed state from ECS
    let mcts_state = extract_mcts_state(world, nation_id, tick, rng.next());

    // Run MCTS
    let best_action = mcts_search(
        mcts_state,
        node_budget,
        rollout_depth,
        top_k,
        1500, // C_puct = 1.5
    );

    vec![best_action]
}
```

### Configuration Constants

```rust
/// MCTS configuration. All values are deterministic (no time-based bounds).
pub struct MctsConfig {
    /// Maximum nodes to expand per search.
    pub node_budget: u32,

    /// Number of ticks to fast-forward in rollouts.
    pub rollout_depth: u32,

    /// Number of candidate actions to consider per node.
    pub top_k_actions: usize,

    /// PUCT exploration constant (0-1000 scale; 1500 = 1.5).
    pub c_puct: i32,
}

impl MctsConfig {
    pub fn difficulty_4() -> Self {
        Self { node_budget: 5_000, rollout_depth: 8, top_k_actions: 15, c_puct: 1500 }
    }

    pub fn difficulty_5() -> Self {
        Self { node_budget: 10_000, rollout_depth: 10, top_k_actions: 20, c_puct: 1500 }
    }
}
```

### Memory Management

```rust
/// MCTS tree memory estimation:
/// - MctsNode: ~100 bytes + children Vec + Option<MctsState>
/// - MctsState: ~1.5 KB
/// - Average children per node: ~10 (after pruning)
///
/// At 10,000 nodes:
/// - Nodes: 10,000 * 100 bytes = 1 MB
/// - States: 10,000 * 1.5 KB = 15 MB (worst case; leaf nodes can drop state)
/// - Total: ~16 MB
///
/// Optimization: only store state at unexpanded leaf nodes (not internal nodes,
/// since internal node states can be reconstructed by replaying actions from root).
/// This reduces state storage to ~branching_factor * leaf_count * 1.5 KB.
///
/// For 10,000 nodes with branching factor 15:
/// - ~667 leaf nodes * 1.5 KB = ~1 MB
/// - Total with optimization: ~2 MB
///
/// MCTS memory is allocated at search start and freed at search end.
/// No persistent state between AI decisions.
```

---

## Open Questions Remaining

1. **Fast-forward model accuracy:** The simplified 5-equation model is a rough approximation.
   How much does MCTS quality degrade compared to using the full simulation? Need to validate
   by playing MCTS-with-approximate-rollout vs MCTS-with-full-rollout (expensive but
   informative one-time test). If approximate rollout is too inaccurate, consider:
   - Increasing rollout depth to compensate
   - Adding more equations (trade, climate effects)
   - Using a trained value function instead of rollouts (requires training infrastructure)

2. **C_puct tuning:** The exploration constant 1.5 is a starting point. Optimal value depends
   on the game dynamics and heuristic quality. Tune via self-play experiments. Consider
   different C_puct values for different game phases (early game: explore more; late game:
   exploit more).

3. **Action representation:** The `Action` type needs a full spec. How are compound actions
   (e.g., "build improvement AND move unit AND change tax rate") represented? Options:
   - Single-action per MCTS decision: nation makes one action per tick. Simple but slow.
   - Action sequence per turn: MCTS searches over sequences of 3-5 actions. Combinatorial
     explosion. Needs aggressive pruning.
   - Factored MCTS: separate MCTS trees for military, economic, diplomatic decisions. Results
     merged. Avoids combinatorics but misses cross-domain synergies.

4. **Multi-nation turn order:** In a single tick, nations act in order. The MCTS tree must
   model this: after our nation's action, the next nation acts, then the next, etc. With 8
   nations, depth-1 in the tree = 8 sequential actions. This makes the tree very deep with
   few visits per node. Mitigation: collapse opponent turns (assume opponents take their
   highest-utility action without MCTS search) and only search the acting nation's decisions.

5. **Difficulty 1-3 AI:** Lower difficulties use simpler AI (heuristic-only, no MCTS).
   Ensure the heuristic scoring function (used for MCTS priors) is independently useful as
   a standalone AI for difficulties 1-3. This avoids maintaining two completely separate AI
   codebases.

6. **Progressive widening:** For very large action spaces (>100 candidate actions even after
   pruning), consider progressive widening: start with K=5 actions and gradually widen to
   K=20 as the node gets more visits. This focuses early search on the most promising actions.

---

## References

- [Monte Carlo Tree Search (Wikipedia)](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search)
- [Nijssen -- Monte-Carlo Tree Search for Multi-Player Games (PhD thesis)](https://project.dke.maastrichtuniversity.nl/games/files/phd/Nijssen_thesis.pdf)
- [Parallel Monte-Carlo Tree Search (Winands et al.)](https://dke.maastrichtuniversity.nl/m.winands/documents/multithreadedMCTS2.pdf)
- [MCTS review: recent modifications and applications (Springer)](https://link.springer.com/article/10.1007/s10462-022-10228-y)
- [AlphaGo/AlphaZero PUCT formula](https://www.chessprogramming.org/UCT)
- [Memory Bounded MCTS (AAAI)](https://cdn.aaai.org/ojs/12932/12932-52-16449-1-2-20201228.pdf)
- [Parametric Action Pre-Selection for MCTS in RTS Games](https://ceur-ws.org/Vol-2719/paper11.pdf)
- [Tabletop Games MCTS framework](https://tabletopgames.ai/wiki/agents/MCTS.html)
- [zxqfl/mcts -- Generic parallel MCTS in Rust](https://github.com/zxqfl/mcts)
