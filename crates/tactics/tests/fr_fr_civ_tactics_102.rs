//! Tests for FR-CIV-TACTICS-102
//!
//! Epic: FR-CIV-TACTICS
//! Status: SPEC-ONLY
//!
//! FR-CIV-TACTICS-102: A* pathfinding returns a valid ordered path.

use civ_tactics::astar_path;

#[cfg(test)]
mod fr_fr_civ_tactics_102 {
    use super::*;

    /// FR-CIV-TACTICS-102: A* finds a direct path in open terrain.
    #[test]
    fn verify_fr_civ_tactics_102_basic() {
        let path = astar_path((0, 0), (3, 0), 100);
        assert!(path.is_some(), "A* should find path in open terrain");
        let p = path.unwrap();
        assert_eq!(p.first(), Some(&(0, 0)), "path starts at origin");
        assert_eq!(p.last(), Some(&(3, 0)), "path ends at goal");
    }

    /// FR-CIV-TACTICS-102: A* returns None when start equals goal.
    #[test]
    fn astar_none_at_goal() {
        assert!(astar_path((5, 5), (5, 5), 100).is_none());
    }
}
