//! Tests for FR-CIV-3D-009
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-009: Scene Initialization Time
//! The 3D client completes scene construction within 2 seconds.
//! Engine-side: verify simulation can bootstrap quickly.

#[cfg(test)]
mod fr_fr_civ_3d_009 {
    use civ_engine::WorldState;

    /// Default WorldState initializes without expensive computation.
    #[test]
    fn default_init_is_cheap() {
        let start = std::time::Instant::now();
        let _ws = WorldState::default();
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "Default WorldState init should be <100ms, got {:?}",
            elapsed
        );
    }

    /// Step function completes quickly.
    #[test]
    fn single_step_is_fast() {
        let ws = WorldState::default();
        let start = std::time::Instant::now();
        let _next = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 10,
            "Single step should be <10ms, got {:?}",
            elapsed
        );
    }
}
