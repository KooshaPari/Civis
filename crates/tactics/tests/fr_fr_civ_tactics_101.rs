//! Tests for FR-CIV-TACTICS-101
//!
//! Epic: FR-CIV-TACTICS
//! Status: SPEC-ONLY
//!
//! FR-CIV-TACTICS-101: Pathfinding BFS next step toward a target.

use civ_tactics::bfs_next_step;

#[cfg(test)]
mod fr_fr_civ_tactics_101 {
    use super::*;

    /// FR-CIV-TACTICS-101: BFS moves one step toward target.
    #[test]
    fn verify_fr_civ_tactics_101_basic() {
        let step = bfs_next_step((0, 0), (5, 0), 100);
        assert!(step.is_some(), "BFS should find a step toward (5,0)");
        let (x, y) = step.unwrap();
        assert_eq!(y, 0, "should move along x axis");
        assert!(x > 0, "should move in positive x direction");
    }

    /// FR-CIV-TACTICS-101: BFS returns None when already at goal.
    #[test]
    fn bfs_returns_none_at_goal() {
        assert!(bfs_next_step((3, 3), (3, 3), 100).is_none());
    }
}
