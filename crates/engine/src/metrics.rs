//! Metrics module - simulation metrics calculation

use crate::Fixed;

#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    pub waste_joules: f64,
    pub surplus_joules: f64,
    pub tyranny_index: f64,
    pub legitimacy_index: f64,
}

pub fn compute(energy_budget_joules: f64, consumption_joules: f64) -> Metrics {
    let waste = (consumption_joules * 0.1).max(0.0);
    let surplus = (energy_budget_joules - consumption_joules).max(0.0);
    let tyranny = (consumption_joules / (energy_budget_joules + 1.0)).min(1.0);
    let legitimacy = (1.0 - tyranny).max(0.0);

    Metrics {
        waste_joules: waste,
        surplus_joules: surplus,
        tyranny_index: tyranny,
        legitimacy_index: legitimacy,
    }
}

/// Fixed-point metrics for deterministic replay and cross-platform simulation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MetricsFixed {
    pub waste_joules: Fixed,
    pub surplus_joules: Fixed,
    pub tyranny_index: Fixed,
    pub legitimacy_index: Fixed,
}

/// Same formulas as [`compute`], using fixed-point arithmetic.
pub fn compute_fixed(energy_budget_joules: Fixed, consumption_joules: Fixed) -> MetricsFixed {
    let tenth = Fixed::from_num(1) / Fixed::from_num(10);
    let waste = (consumption_joules * tenth).max(Fixed::ZERO);
    let surplus = (energy_budget_joules - consumption_joules).max(Fixed::ZERO);
    let denominator = energy_budget_joules + Fixed::ONE;
    let tyranny = (consumption_joules / denominator).min(Fixed::ONE);
    let legitimacy = (Fixed::ONE - tyranny).max(Fixed::ZERO);

    MetricsFixed {
        waste_joules: waste,
        surplus_joules: surplus,
        tyranny_index: tyranny,
        legitimacy_index: legitimacy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_basic() {
        let m = compute(1000.0, 500.0);
        assert_eq!(m.waste_joules, 50.0);
        assert_eq!(m.surplus_joules, 500.0);
    }

    #[test]
    fn test_tyranny_index() {
        let m = compute(100.0, 100.0);
        assert!(m.tyranny_index > 0.9);
    }

    #[test]
    fn compute_fixed_matches_float_within_six_decimals() {
        const EPS: f64 = 1e-6;
        let cases = [(1000.0, 500.0), (100.0, 100.0)];

        for (budget, consumption) in cases {
            let float_m = compute(budget, consumption);
            let fixed_m = compute_fixed(
                Fixed::from_num(budget as i64),
                Fixed::from_num(consumption as i64),
            );

            assert!(
                (float_m.waste_joules - fixed_m.waste_joules.to_f64()).abs() < EPS,
                "waste: float={}, fixed={}",
                float_m.waste_joules,
                fixed_m.waste_joules.to_f64()
            );
            assert!(
                (float_m.surplus_joules - fixed_m.surplus_joules.to_f64()).abs() < EPS,
                "surplus: float={}, fixed={}",
                float_m.surplus_joules,
                fixed_m.surplus_joules.to_f64()
            );
            assert!(
                (float_m.tyranny_index - fixed_m.tyranny_index.to_f64()).abs() < EPS,
                "tyranny: float={}, fixed={}",
                float_m.tyranny_index,
                fixed_m.tyranny_index.to_f64()
            );
            assert!(
                (float_m.legitimacy_index - fixed_m.legitimacy_index.to_f64()).abs() < EPS,
                "legitimacy: float={}, fixed={}",
                float_m.legitimacy_index,
                fixed_m.legitimacy_index.to_f64()
            );
        }
    }
}
