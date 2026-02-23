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
}
