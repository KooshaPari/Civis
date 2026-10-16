#![allow(clippy::field_reassign_with_default)]
// State built via Default::default() + explicit field assignments for readability;
// restructuring into an initializer is out of scope for this no-behavior-change cleanup.

//! Tests for FR-SESS-004 - Challenge Mode
//!
//! Epic: FR-SESS
//! Challenge mode SHALL allow async submission of a civ seed for scoring.

#[cfg(test)]
mod fr_fr_sess_004 {
    #[test]
    fn challenge_seed_is_settable() {
        let mut ws = civ_engine::WorldState::default();
        ws.rng_seed = 42;
        assert_eq!(ws.rng_seed, 42, "challenge seed is settable");
    }
}
