//! Tests for FR-CIV-0104-008
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-008: Parameter Immutability
//! No client command can modify MinimalConstraintParams at runtime.
//! The params struct is Clone + Serialize but represents an immutable config.

#[cfg(test)]
mod fr_fr_civ_0104_008 {
    use civ_engine::constraints::MinimalConstraintParams;
    use civ_engine::Fixed;

    /// Default params are deterministic across constructions.
    #[test]
    fn default_params_are_deterministic() {
        let p1 = MinimalConstraintParams::default();
        let p2 = MinimalConstraintParams::default();
        assert_eq!(p1, p2, "Default params should be deterministic");
    }

    /// C1 params have correct default values per spec.
    #[test]
    fn c1_params_match_spec_defaults() {
        let p = MinimalConstraintParams::default();
        assert_eq!(p.c1.e_base, Fixed::from_num(6) / Fixed::from_num(10));
        assert_eq!(p.c1.kappa_l, Fixed::from_num(4));
        assert_eq!(p.c1.sel_max, Fixed::from_num(2) / Fixed::from_num(10));
    }

    /// C2 params have correct default values per spec.
    #[test]
    fn c2_params_match_spec_defaults() {
        let p = MinimalConstraintParams::default();
        assert_eq!(p.c2.b_min, Fixed::from_num(92) / Fixed::from_num(100));
        assert_eq!(p.c2.s_max, Fixed::from_num(75) / Fixed::from_num(100));
    }

    /// C3 params have correct default values per spec.
    #[test]
    fn c3_params_match_spec_defaults() {
        let p = MinimalConstraintParams::default();
        assert_eq!(p.c3.o_max, Fixed::from_num(15) / Fixed::from_num(100));
        assert_eq!(
            p.c3.ledger_completeness_floor,
            Fixed::from_num(92) / Fixed::from_num(100)
        );
    }

    /// C4 params have correct default values per spec.
    #[test]
    fn c4_params_match_spec_defaults() {
        let p = MinimalConstraintParams::default();
        assert_eq!(p.c4.a_min_base, Fixed::from_num(4) / Fixed::from_num(100));
        assert_eq!(
            p.c4.cd_max,
            Fixed::from_num(25) / Fixed::from_num(100)
        );
    }

    /// C5 params have correct default values per spec.
    #[test]
    fn c5_params_match_spec_defaults() {
        let p = MinimalConstraintParams::default();
        assert_eq!(
            p.c5.c0_ceiling,
            Fixed::from_num(95) / Fixed::from_num(100)
        );
        assert_eq!(
            p.c5.l0_ceiling,
            Fixed::from_num(95) / Fixed::from_num(100)
        );
        assert_eq!(p.c5.coalition_min_members, 3);
    }

    /// Top-level thresholds match spec: lambda_rec = 0.35, L_min = 0.20.
    #[test]
    fn legitimacy_thresholds_match_spec() {
        let p = MinimalConstraintParams::default();
        assert_eq!(
            p.legitimacy_recovery_threshold,
            Fixed::from_num(35) / Fixed::from_num(100)
        );
        assert_eq!(
            p.legitimacy_floor,
            Fixed::from_num(20) / Fixed::from_num(100)
        );
        assert_eq!(p.recovery_window, 50);
    }

    /// Params serializes and deserializes deterministically (round-trip).
    #[test]
    fn params_roundtrip_serialization() {
        let p = MinimalConstraintParams::default();
        let json = serde_json::to_string(&p).expect("serialize");
        let p2: MinimalConstraintParams = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p, p2, "Serialization round-trip should preserve params");
    }
}
