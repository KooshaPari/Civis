//! Monte Carlo Tree Search lookahead (FR-AI-003).
//!
//! Provides a domain-agnostic MCTS engine. Callers supply a
//! [`GameState`] implementation and receive the best action.

use std::collections::HashMap;
use std::marker::PhantomData;

/// Stable identifier for an action.
pub type ActionId = String;

/// Configuration for an MCTS search.
#[derive(Debug, Clone)]
pub struct MctsConfig {
    /// Number of MCTS iterations.
    pub iterations: usize,
    /// Maximum depth for random simulations.
    pub max_sim_depth: usize,
    /// Exploration constant (c) for UCB1.
    pub exploration: f64,
    /// Optional seed for deterministic randomness.
    pub seed: Option<u64>,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            max_sim_depth: 10,
            exploration: std::f64::consts::SQRT_2,
            seed: None,
        }
    }
}

/// Trait that a game state must implement for MCTS.
pub trait GameState: Clone {
    /// Return all legal actions from this state.
    fn legal_actions(&self) -> Vec<ActionId>;
    /// Apply an action, producing a new state.
    fn apply_action(&self, action: &ActionId) -> Self;
    /// Returns true if no more actions are available.
    fn is_terminal(&self) -> bool;
    /// Terminal reward from the perspective of the player to move.
    fn reward(&self) -> Option<f64>;
    /// Choose a random legal action.
    fn random_action(&self, rng: &mut LinearRng) -> ActionId;
}

/// Deterministic linear congruential generator.
#[derive(Debug, Clone)]
pub struct LinearRng {
    state: u64,
}

impl LinearRng {
    /// Create a new RNG with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    /// Generate the next pseudo-random u64.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
    /// Generate a pseudo-random value in [0, bound).
    #[must_use]
    pub fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() as usize) % bound
    }
}

/// A single node in the MCTS tree.
#[derive(Debug)]
pub struct Node {
    /// The action that led to this node.
    /// The action that led to this node.
    pub action: ActionId,
    /// Total value accumulated from rollouts.
    /// Total value accumulated from rollouts.
    pub total_value: f64,
    /// Number of visits to this node.
    /// Number of visits to this node.
    pub visits: usize,
    /// Child nodes indexed by action.
    pub children: HashMap<ActionId, Node>,
    /// Whether this node represents a terminal state.
    pub is_terminal: bool,
}

impl Node {
    /// Create a new leaf node.
    pub fn new(action: ActionId, is_terminal: bool) -> Self {
        Self {
            action,
            total_value: 0.0,
            visits: 0,
            children: HashMap::new(),
            is_terminal,
        }
    }
    #[must_use]
    /// Mean action value.
    pub fn q_mean(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_value / self.visits as f64
        }
    }
    #[must_use]
    /// Upper Confidence Bound for Trees (UCB1).
    pub fn ucb1(&self, exploration: f64, parent_visits: usize) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        self.q_mean() + exploration * ((parent_visits as f64).ln() / self.visits as f64).sqrt()
    }
}

/// Monte Carlo Tree Search tree.
pub struct MctsTree<G: GameState> {
    root: Node,
    config: MctsConfig,
    _phantom: PhantomData<G>,
}

impl<G: GameState> MctsTree<G> {
    #[must_use]
    /// Create a new MCTS tree from a game state.
    pub fn new(state: &G, config: MctsConfig) -> Self {
        Self {
            root: Node::new(String::new(), state.is_terminal()),
            config,
            _phantom: PhantomData,
        }
    }
    #[must_use]
    /// Return a reference to the root node.
    pub fn root(&self) -> &Node {
        &self.root
    }
    /// Run the MCTS search iterations.
    /// Run the MCTS search iterations.
    pub fn search(&mut self, state: &G) {
        if state.is_terminal() {
            return;
        }
        let seed = self.config.seed.unwrap_or(42);
        let mut rng = LinearRng::new(seed);
        for _ in 0..self.config.iterations {
            self.run_one_iteration(state, &mut rng);
        }
    }
    #[must_use]
    /// Return the best action based on visit counts.
    pub fn best_action(&self) -> Option<ActionId> {
        self.root
            .children
            .iter()
            .max_by_key(|(_, n)| n.visits)
            .map(|(a, _)| a.clone())
    }
    #[must_use]
    /// Total iterations executed.
    pub fn iterations(&self) -> usize {
        self.root.visits
    }

    fn run_one_iteration(&mut self, state: &G, rng: &mut LinearRng) {
        let exploration = self.config.exploration;
        let max_sim_depth = self.config.max_sim_depth;
        let mut sim_state = state.clone();
        let mut path: Vec<ActionId> = Vec::new();
        Self::select(&mut self.root, exploration, &mut path, &mut sim_state);
        let action_applied = {
            let leaf = Self::node_at_mut(&mut self.root, &path);
            Self::expand_node(leaf, &mut sim_state, rng)
        };
        if let Some(action) = action_applied {
            path.push(action);
        }
        let reward = if let Some(r) = sim_state.reward() {
            r
        } else {
            Self::simulate_random(&sim_state, rng, max_sim_depth)
        };
        Self::backup(&mut self.root, &path, reward);
    }

    fn node_at_mut<'a>(root: &'a mut Node, path: &[ActionId]) -> &'a mut Node {
        let mut current = root;
        for action in path {
            current = current.children.get_mut(action).unwrap();
        }
        current
    }

    fn select(mut node: &mut Node, exploration: f64, path: &mut Vec<ActionId>, sim_state: &mut G) {
        loop {
            if node.is_terminal {
                break;
            }
            let legal = sim_state.legal_actions();
            if node.children.len() < legal.len() {
                break;
            }
            let pv = node.visits;
            let best = node
                .children
                .iter()
                .max_by(|a, b| {
                    a.1.ucb1(exploration, pv)
                        .partial_cmp(&b.1.ucb1(exploration, pv))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone());
            match best {
                Some(action) => {
                    *sim_state = sim_state.apply_action(&action);
                    path.push(action.clone());
                    node = node.children.get_mut(&action).unwrap();
                }
                None => break,
            }
        }
    }

    fn expand_node(leaf: &mut Node, sim_state: &mut G, rng: &mut LinearRng) -> Option<ActionId> {
        if leaf.is_terminal {
            return None;
        }
        let legal = sim_state.legal_actions();
        if legal.is_empty() {
            return None;
        }
        let action = legal[rng.next_usize(legal.len())].clone();
        *sim_state = sim_state.apply_action(&action);
        let is_term = sim_state.is_terminal();
        leaf.children
            .insert(action.clone(), Node::new(action.clone(), is_term));
        Some(action)
    }

    fn simulate_random(state: &G, rng: &mut LinearRng, max_depth: usize) -> f64 {
        let mut sim = state.clone();
        for _ in 0..max_depth {
            if let Some(r) = sim.reward() {
                return r;
            }
            let legal = sim.legal_actions();
            if legal.is_empty() {
                break;
            }
            let action = sim.random_action(rng);
            sim = sim.apply_action(&action);
        }
        sim.reward().unwrap_or(0.0)
    }

    fn backup(node: &mut Node, path: &[ActionId], reward: f64) {
        node.visits += 1;
        node.total_value += reward;
        if let Some((first, rest)) = path.split_first() {
            if let Some(child) = node.children.get_mut(first) {
                Self::backup(child, rest, reward);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone)]
    struct TestGame {
        outcome: u8,
    }
    impl TestGame {
        fn new() -> Self {
            Self { outcome: 0 }
        }
    }
    impl GameState for TestGame {
        fn legal_actions(&self) -> Vec<ActionId> {
            if self.outcome != 0 {
                return vec![];
            }
            vec!["win".into(), "lose".into(), "draw".into()]
        }
        fn apply_action(&self, action: &ActionId) -> Self {
            match action.as_str() {
                "win" => Self { outcome: 1 },
                "lose" => Self { outcome: 2 },
                "draw" => Self { outcome: 3 },
                _ => self.clone(),
            }
        }
        fn is_terminal(&self) -> bool {
            self.outcome != 0
        }
        fn reward(&self) -> Option<f64> {
            match self.outcome {
                0 => None,
                1 => Some(1.0),
                2 => Some(0.0),
                3 => Some(0.5),
                _ => None,
            }
        }
        fn random_action(&self, rng: &mut LinearRng) -> ActionId {
            let a = self.legal_actions();
            a[rng.next_usize(a.len())].clone()
        }
    }
    #[test]
    fn rng_determinism() {
        let mut a = LinearRng::new(123);
        let mut b = LinearRng::new(123);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
    #[test]
    fn node_q_mean_zero() {
        let n = Node::new("a".into(), false);
        assert!(n.q_mean().abs() < f64::EPSILON);
    }
    #[test]
    fn node_q_mean_with_visits() {
        let mut n = Node::new("a".into(), false);
        n.total_value = 3.0;
        n.visits = 6;
        assert!((n.q_mean() - 0.5).abs() < f64::EPSILON);
    }
    #[test]
    fn node_ucb1_zero_visits() {
        let n = Node::new("a".into(), false);
        assert!(n.ucb1(1.0, 10).is_infinite());
    }
    #[test]
    fn node_ucb1_with_visits() {
        let mut n = Node::new("a".into(), false);
        n.total_value = 1.0;
        n.visits = 1;
        assert!(n.ucb1(1.0, 10) > 1.0);
    }
    #[test]
    fn mcts_selects_winning() {
        let cfg = MctsConfig {
            iterations: 200,
            seed: Some(42),
            ..Default::default()
        };
        let mut tree = MctsTree::new(&TestGame::new(), cfg);
        tree.search(&TestGame::new());
        assert_eq!(tree.best_action(), Some("win".into()));
    }
    #[test]
    fn zero_iterations() {
        let tree = MctsTree::new(
            &TestGame::new(),
            MctsConfig {
                iterations: 0,
                ..Default::default()
            },
        );
        assert_eq!(tree.iterations(), 0);
        assert!(tree.best_action().is_none());
    }
    #[test]
    fn deterministic_seed() {
        let cfg = MctsConfig {
            iterations: 50,
            seed: Some(99),
            ..Default::default()
        };
        let mut t1 = MctsTree::new(&TestGame::new(), cfg.clone());
        let mut t2 = MctsTree::new(&TestGame::new(), cfg);
        let s = TestGame::new();
        t1.search(&s);
        t2.search(&s);
        assert_eq!(t1.best_action(), t2.best_action());
    }
    #[test]
    fn terminal_no_expansion() {
        let terminal = TestGame { outcome: 1 };
        let mut tree = MctsTree::new(&terminal, MctsConfig::default());
        tree.search(&terminal);
        assert_eq!(tree.iterations(), 0);
    }
    #[test]
    fn three_action_game() {
        let cfg = MctsConfig {
            iterations: 300,
            seed: Some(7),
            ..Default::default()
        };
        let mut tree = MctsTree::new(&TestGame::new(), cfg);
        tree.search(&TestGame::new());
        assert_eq!(tree.best_action(), Some("win".into()));
        assert_eq!(tree.root().children.len(), 3);
    }
    #[test]
    fn config_defaults() {
        let c = MctsConfig::default();
        assert_eq!(c.iterations, 100);
        assert_eq!(c.max_sim_depth, 10);
        assert!(c.seed.is_none());
        assert!((c.exploration - std::f64::consts::SQRT_2).abs() < f64::EPSILON);
    }
}
