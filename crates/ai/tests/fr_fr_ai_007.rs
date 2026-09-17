//! FR-AI-007 — MCTS computation time SHALL be capped at a fraction of the
//! 100 ms tick budget.

use civ_ai::{MctsConfig, MctsGameState, MctsTree};

/// A game that takes a while to explore - many branches at each depth.
#[derive(Clone)]
struct SlowGame {
    depth: usize,
}

impl SlowGame {
    fn start() -> Self {
        Self { depth: 0 }
    }
}

impl MctsGameState for SlowGame {
    fn legal_actions(&self) -> Vec<civ_ai::mcts::ActionId> {
        if self.depth >= 4 {
            return vec![];
        }
        vec!["a".into(), "b".into(), "c".into(), "d".into()]
    }

    fn apply_action(&self, _action: &civ_ai::mcts::ActionId) -> Self {
        Self {
            depth: self.depth + 1,
        }
    }

    fn is_terminal(&self) -> bool {
        self.depth >= 4
    }

    fn reward(&self) -> Option<f64> {
        if self.is_terminal() {
            Some(0.5)
        } else {
            None
        }
    }

    fn random_action(&self, rng: &mut civ_ai::mcts::LinearRng) -> civ_ai::mcts::ActionId {
        let a = self.legal_actions();
        a[rng.next_usize(a.len())].clone()
    }
}

#[test]
fn time_capped_within_budget() {
    // Ask for many iterations but cap at 20ms.
    let cfg = MctsConfig {
        iterations: 100_000,
        max_sim_depth: 10,
        exploration: std::f64::consts::SQRT_2,
        seed: Some(42),
        time_budget_ms: Some(20),
    };

    let start = std::time::Instant::now();
    let mut tree = MctsTree::new(&SlowGame::start(), cfg);
    tree.search(&SlowGame::start());
    let elapsed = start.elapsed();

    // Should have stopped well before 100_000 iterations due to time budget.
    assert!(tree.iterations() < 100_000,
        "expected early termination, ran {} iterations", tree.iterations());
    // Should respect the 20ms budget (with some tolerance for scheduling).
    assert!(elapsed.as_millis() < 500,
        "search took {:?}, expected < 500ms (20ms budget + overhead)", elapsed);
}

#[test]
fn no_budget_runs_all_iterations() {
    let cfg = MctsConfig {
        iterations: 200,
        max_sim_depth: 3,
        exploration: std::f64::consts::SQRT_2,
        seed: Some(42),
        time_budget_ms: None,
    };

    let mut tree = MctsTree::new(&SlowGame::start(), cfg);
    tree.search(&SlowGame::start());

    // Without a time budget, all 200 iterations should run.
    assert_eq!(tree.iterations(), 200);
}
