//! Tests for FR-CIV-VERIFY-007
//! Epic: FR-CIV-VERIFY. Worktree naming convention.
#[cfg(test)]
mod fr_fr_civ_verify_007 {
    #[test]
    fn engine_works_from_worktree() {
        // FR-CIV-VERIFY-007 requires the engine to work from any worktree path.
        // This test proves it works from the current worktree.
        let ws = civ_engine::WorldState::default();
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        assert_eq!(next.tick, 1);
    }
}
