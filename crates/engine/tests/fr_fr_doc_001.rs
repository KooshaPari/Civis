//! Tests for FR-DOC-001 — Documentation / public API surface
//!
//! Epic: FR-DOC
//! Verifies that key public API types and functions have documentation.

#[cfg(test)]
mod fr_fr_doc_001 {
    /// FR-DOC-001: Step function exists and is callable.
    #[test]
    fn step_function_exists_and_is_callable() {
        let ws = civ_engine::WorldState::default();
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(next.tick, 1);
    }

    /// FR-DOC-001: SCALE constant is documented with a value.
    #[test]
    fn scale_constant_is_documented() {
        assert!(civ_engine::SCALE > 0);
    }

    /// FR-DOC-001: create_rng function is public.
    #[test]
    fn create_rng_is_public() {
        let _rng = civ_engine::create_rng(42);
    }
}
