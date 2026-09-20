//! Tests for FR-CIV-POLITY-007
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-007: StratBand ordering and shifting.

#[cfg(test)]
mod fr_fr_civ_polity_007 {
    /// Verify FR-CIV-POLITY-007: StratBand ranks are ordered.
    #[test]
    fn verify_fr_civ_polity_007_basic() {
        use civ_engine::StratBand;
        assert!(StratBand::Poor < StratBand::Middle);
        assert!(StratBand::Middle < StratBand::Rich);
        assert!(StratBand::Rich < StratBand::Elite);
    }

    /// Verify StratBand::shift promotes correctly.
    #[test]
    fn strat_band_shift_promotion() {
        use civ_engine::StratBand;
        assert_eq!(StratBand::Poor.shift(1), StratBand::Middle);
        assert_eq!(StratBand::Middle.shift(1), StratBand::Rich);
        assert_eq!(StratBand::Rich.shift(1), StratBand::Elite);
        // Clamps at Elite
        assert_eq!(StratBand::Elite.shift(1), StratBand::Elite);
    }

    /// Verify StratBand::shift demotes correctly.
    #[test]
    fn strat_band_shift_demotion() {
        use civ_engine::StratBand;
        assert_eq!(StratBand::Elite.shift(-1), StratBand::Rich);
        assert_eq!(StratBand::Rich.shift(-1), StratBand::Middle);
        assert_eq!(StratBand::Middle.shift(-1), StratBand::Poor);
        // Clamps at Poor
        assert_eq!(StratBand::Poor.shift(-1), StratBand::Poor);
    }
}
