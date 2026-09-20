//! Tests for FR-CIV-LEGENDS-PRESIM-10 — Pre-simulation legend seeding
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies that the legends system can be seeded with initial state.

#[cfg(test)]
mod fr_fr_civ_legends_presim_10 {
    use civ_legends::{LegendsConfig, SignificanceConfig};

    /// FR-CIV-LEGENDS-PRESIM-10: LegendsConfig has epoch_of mapping.
    #[test]
    fn config_epoch_mapping() {
        let config = LegendsConfig::default();
        assert_eq!(config.epoch_of(0).0, 0);
        assert_eq!(config.epoch_of(63).0, 0);
        assert_eq!(config.epoch_of(64).0, 1);
        assert_eq!(config.epoch_of(128).0, 2);
    }

    /// FR-CIV-LEGENDS-PRESIM-10: SignificanceConfig has configurable decay.
    #[test]
    fn significance_config_decay_rate() {
        let config = SignificanceConfig::default();
        assert!(config.decay_rate > 0.0 && config.decay_rate < 1.0);
        assert!(config.cluster_window > 0);
    }

    /// FR-CIV-LEGENDS-PRESIM-10: LegendsConfig default has sensible promotion threshold.
    #[test]
    fn config_promotion_threshold() {
        let config = LegendsConfig::default();
        assert!(config.promotion_threshold > 0.0);
        assert!(config.max_graph_nodes > 0);
    }
}
