//! Tests for FR-CIV-EMERGENCE-002 — criticality indicator.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_002 {
    use civ_emergence_metrics::{criticality_indicator, CriticalityBands, CriticalityInputs};

    #[test]
    fn verify_fr_civ_emergence_002_basic() {
        // All inputs at band centres → indicator ~1.0
        let inputs = CriticalityInputs {
            branching_sigma: 0.95,  // centre of [0.85, 1.05]
            power_law_alpha: 1.7,   // centre of [1.4, 2.0]
            entropy_norm: 0.75,     // centre of [0.6, 0.9]
        };
        let value = criticality_indicator(inputs, &CriticalityBands::default());
        assert!(value > 0.8, "all-centre inputs should yield high indicator, got {value}");
    }

    #[test]
    fn zero_defaults_give_zero() {
        let inputs = CriticalityInputs {
            branching_sigma: 0.0,
            power_law_alpha: 0.0,
            entropy_norm: 0.0,
        };
        let value = criticality_indicator(inputs, &CriticalityBands::default());
        assert_eq!(value, 0.0);
    }

    #[test]
    fn out_of_band_input_reduces_indicator() {
        let centre = CriticalityInputs {
            branching_sigma: 0.95,
            power_law_alpha: 1.7,
            entropy_norm: 0.75,
        };
        let out = CriticalityInputs {
            branching_sigma: 0.5,  // way below band
            power_law_alpha: 1.7,
            entropy_norm: 0.75,
        };
        let v_centre = criticality_indicator(centre, &CriticalityBands::default());
        let v_out = criticality_indicator(out, &CriticalityBands::default());
        assert!(v_out < v_centre, "out-of-band should be lower");
    }
}
