//! Compatibility path for the deterministic material cellular automaton.
//!
//! The implementation lives in [`crate::fluid_ca`]. Keeping this module as a
//! re-export preserves the historical module path without maintaining a
//! second, divergent CA state model.

pub use crate::fluid_ca::*;

#[cfg(test)]
mod tests {
    use super::CaGrid;

    #[test]
    fn compatibility_path_uses_deterministic_ca_grid() {
        let grid = CaGrid::new([2, 2, 2]);
        assert_eq!(grid.cells.len(), 8);
        assert!(grid.dirty_chunks.is_empty());
    }
}
