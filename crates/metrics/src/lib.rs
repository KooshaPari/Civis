use civ_engine::Fixed;

#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    pub waste_joules: f64,
    pub surplus_joules: f64,
    pub tyranny_index: f64,
    pub legitimacy_index: f64,
}

pub fn compute(energy_budget_joules: Fixed, consumption_joules: Fixed) -> Metrics {
    let energy_f64 = energy_budget_joules.to_f64();
    let consumption_f64 = consumption_joules.to_f64();
    
    let waste = (consumption_f64 * 0.1).max(0.0);
    let surplus = (energy_f64 - consumption_f64).max(0.0);
    let tyranny = (consumption_f64 / (energy_f64 + 1.0)).min(1.0);
    let legitimacy = (1.0 - tyranny).max(0.0);

    Metrics {
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
    fn compute_with_zero_energy_budget() {
        let metrics = compute(Fixed::ZERO, Fixed::ZERO);
        assert_eq!(metrics.waste_joules, 0.0);
        assert_eq!(metrics.surplus_joules, 0.0);
        assert_eq!(metrics.tyranny_index, 0.0);
        assert_eq!(metrics.legitimacy_index, 1.0);
    }

    #[test]
    fn compute_with_excess_consumption() {
        let energy = Fixed::from_num(100i64);
        let consumption = Fixed::from_num(50i64);
        let metrics = compute(energy, consumption);
        
        assert!(metrics.waste_joules > 0.0);
        assert!(metrics.surplus_joules > 0.0);
        assert!(metrics.tyranny_index > 0.0);
        assert!(metrics.legitimacy_index > 0.0);
    }

    #[test]
    fn compute_waste_is_ten_percent_of_consumption() {
        let energy = Fixed::from_num(1000i64);
        let consumption = Fixed::from_num(100i64);
        let metrics = compute(energy, consumption);
        
        let expected_waste = 100.0 * 0.1;
        assert!((metrics.waste_joules - expected_waste).abs() < 0.0001);
    }

    #[test]
    fn compute_surplus_never_negative() {
        let energy = Fixed::from_num(100i64);
        let consumption = Fixed::from_num(200i64);
        let metrics = compute(energy, consumption);
        
        assert!(metrics.surplus_joules >= 0.0);
    }

    #[test]
    fn compute_tyranny_index_bounded() {
        let energy = Fixed::from_num(1000i64);
        let consumption = Fixed::from_num(5000i64);
        let metrics = compute(energy, consumption);
        
        assert!(metrics.tyranny_index >= 0.0);
        assert!(metrics.tyranny_index <= 1.0);
    }

    #[test]
    fn compute_legitimacy_is_one_minus_tyranny() {
        let energy = Fixed::from_num(500i64);
        let consumption = Fixed::from_num(250i64);
        let metrics = compute(energy, consumption);
        
        let sum = metrics.tyranny_index + metrics.legitimacy_index;
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn compute_legitimacy_never_negative() {
        let energy = Fixed::from_num(10i64);
        let consumption = Fixed::from_num(1000i64);
        let metrics = compute(energy, consumption);
        
        assert!(metrics.legitimacy_index >= 0.0);
    }

    #[test]
    fn metrics_default_is_zero() {
        let m = Metrics::default();
        assert_eq!(m.waste_joules, 0.0);
        assert_eq!(m.surplus_joules, 0.0);
        assert_eq!(m.tyranny_index, 0.0);
        assert_eq!(m.legitimacy_index, 0.0);
    }

    #[test]
    fn metrics_clone_preserves_values() {
        let original = compute(Fixed::from_num(100i64), Fixed::from_num(50i64));
        let cloned = original;
        
        assert_eq!(original.waste_joules, cloned.waste_joules);
        assert_eq!(original.surplus_joules, cloned.surplus_joules);
        assert_eq!(original.tyranny_index, cloned.tyranny_index);
        assert_eq!(original.legitimacy_index, cloned.legitimacy_index);
    }
}
