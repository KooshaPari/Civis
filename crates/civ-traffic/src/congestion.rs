//! Path congestion model for traffic edge costs.
//!
//! FR-CIV-PATH-CONGEST: edge cost rises with concurrent users and relaxes
//! when they leave.

use serde::{Deserialize, Serialize};

/// Congestion state for a single traversed edge.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathCongestion {
    /// Baseline traversal cost with no load.
    pub base_cost: f32,
    /// Number of concurrent users currently occupying the edge.
    pub users: u32,
    /// Additional cost applied per concurrent user.
    pub per_user_penalty: f32,
}

impl PathCongestion {
    /// Build a congestion model with a baseline cost and per-user penalty.
    #[must_use]
    pub const fn new(base_cost: f32, per_user_penalty: f32) -> Self {
        Self {
            base_cost,
            users: 0,
            per_user_penalty,
        }
    }

    /// Register an entrant onto the edge.
    pub fn enter(&mut self) -> u32 {
        self.users = self.users.saturating_add(1);
        self.users
    }

    /// Register an exit from the edge.
    pub fn leave(&mut self) -> u32 {
        self.users = self.users.saturating_sub(1);
        self.users
    }

    /// Current traversal cost for the edge.
    #[must_use]
    pub fn cost(self) -> f32 {
        self.base_cost + self.per_user_penalty * self.users as f32
    }
}

#[cfg(test)]
mod tests {
    use super::PathCongestion;

    /// FR-CIV-PATH-CONGEST — cost rises under load and decays after users leave.
    #[test]
    fn cost_rises_under_load_and_decays_after() {
        let mut congestion = PathCongestion::new(10.0, 2.5);
        let idle = congestion.cost();

        congestion.enter();
        let one_user = congestion.cost();

        congestion.enter();
        let two_users = congestion.cost();

        congestion.leave();
        let after_leave = congestion.cost();

        assert!(one_user > idle);
        assert!(two_users > one_user);
        assert!(after_leave < two_users);
        assert_eq!(after_leave, one_user);
    }
}
