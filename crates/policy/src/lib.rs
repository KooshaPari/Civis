use civ_engine::Fixed;

#[derive(Debug, Clone, Copy)]
pub struct PolicyInput {
    pub base_consumption_joules: f64,
    pub scarcity_multiplier: f64,
}

pub fn effective_consumption(input: PolicyInput) -> Fixed {
    let result = input.base_consumption_joules * input.scarcity_multiplier.max(0.0);
    Fixed::from_num(result as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumption_non_negative() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 10.0,
            scarcity_multiplier: -1.0,
        });
        assert_eq!(c, Fixed::ZERO);
    }

    #[test]
    fn consumption_zero_multiplier() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 100.0,
            scarcity_multiplier: 0.0,
        });
        assert_eq!(c, Fixed::ZERO);
    }

    #[test]
    fn consumption_identity_multiplier() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 50.0,
            scarcity_multiplier: 1.0,
        });
        assert_eq!(c, Fixed::from_num(50i64));
    }

    #[test]
    fn consumption_scales_with_multiplier() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 100.0,
            scarcity_multiplier: 2.0,
        });
        assert_eq!(c, Fixed::from_num(200i64));
    }

    #[test]
    fn consumption_fractional_multiplier() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 100.0,
            scarcity_multiplier: 0.5,
        });
        assert_eq!(c, Fixed::from_num(50i64));
    }

    #[test]
    fn consumption_clamps_negative_multiplier() {
        let c1 = effective_consumption(PolicyInput {
            base_consumption_joules: 50.0,
            scarcity_multiplier: -10.0,
        });
        let c2 = effective_consumption(PolicyInput {
            base_consumption_joules: 50.0,
            scarcity_multiplier: -0.5,
        });
        assert_eq!(c1, c2);
        assert_eq!(c1, Fixed::ZERO);
    }

    #[test]
    fn policy_input_clone() {
        let input = PolicyInput {
            base_consumption_joules: 75.5,
            scarcity_multiplier: 1.5,
        };
        let cloned = input;
        
        assert_eq!(input.base_consumption_joules, cloned.base_consumption_joules);
        assert_eq!(input.scarcity_multiplier, cloned.scarcity_multiplier);
    }

    #[test]
    fn consumption_with_large_values() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 1e9,
            scarcity_multiplier: 2.0,
        });
        let expected = Fixed::from_num(2_000_000_000i64);
        assert_eq!(c, expected);
    }
}
