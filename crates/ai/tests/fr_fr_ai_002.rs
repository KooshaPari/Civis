//! FR-AI-002 — MCTS SHALL be used for multi-step lookahead planning
//! beyond depth 1.

use civ_ai::{MctsConfig, MctsGameState, MctsTree};

/// A simple game where the agent must choose "right" twice to win.
/// Depth 1 greedy picks "left" which looks good short-term but loses.
/// MCTS with depth > 1 finds the two-step "right" path.
#[derive(Clone)]
struct DepthGame {
    path: Vec<String>,
}

impl DepthGame {
    fn start() -> Self {
        Self { path: vec![] }
    }
}

impl MctsGameState for DepthGame {
    fn legal_actions(&self) -> Vec<civ_ai::mcts::ActionId> {
        match self.path.len() {
            0 => vec!["left".into(), "right".into()],
            1 => vec!["continue".into(), "stop".into()],
            _ => vec![],
        }
    }

    fn apply_action(&self, action: &civ_ai::mcts::ActionId) -> Self {
        let mut p = self.path.clone();
        p.push(action.clone());
        Self { path: p }
    }

    fn is_terminal(&self) -> bool {
        self.path.len() >= 2
    }

    fn reward(&self) -> Option<f64> {
        if !self.is_terminal() {
            return None;
        }
        // "right" then "continue" = 1.0, everything else < 1.0.
        if self.path.get(0).map_or(false, |a| a == "right")
            && self.path.get(1).map_or(false, |a| a == "continue")
        {
            Some(1.0)
        } else {
            Some(0.0)
        }
    }

    fn random_action(&self, rng: &mut civ_ai::mcts::LinearRng) -> civ_ai::mcts::ActionId {
        let a = self.legal_actions();
        a[rng.next_usize(a.len())].clone()
    }
}

#[test]
fn lookahead_depth_gt_1() {
    let cfg = MctsConfig {
        iterations: 500,
        max_sim_depth: 5,
        seed: Some(42),
        ..Default::default()
    };
    let mut tree = MctsTree::new(&DepthGame::start(), cfg);
    tree.search(&DepthGame::start());

    // MCTS must look ahead and find "right" as the best first move.
    let best = tree.best_action();
    assert_eq!(best, Some("right".into()),
        "MCTS with depth > 1 should find the two-step winning path starting with 'right'");
}

#[test]
fn tree_expands_beyond_root() {
    let cfg = MctsConfig {
        iterations: 1000,
        max_sim_depth: 10,
        seed: Some(7),
        ..Default::default()
    };
    let mut tree = MctsTree::new(&DepthGame::start(), cfg);
    tree.search(&DepthGame::start());

    // After 1000 iterations the tree should have expanded beyond the root.
    let root = tree.root();
    assert!(!root.children.is_empty(), "root must have children after search");
    assert!(root.visits >= 1000, "root should have been visited many times");
}
