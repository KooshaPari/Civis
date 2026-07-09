//! Cellular-automaton material simulation.
//!
//! Stub — the module is declared in `lib.rs` but no external crate imports from it
//! directly. Added so the crate compiles while the CA logic is developed.

/// Minimum-viable placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialCaState;

/// Stub — returns default state.
pub fn simulate(_dt: f64) -> MaterialCaState {
    MaterialCaState
}
