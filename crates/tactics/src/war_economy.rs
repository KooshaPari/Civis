//! FR-CIV-WARFARE-003 — Sustained war drains cluster economy; attrition reduces population.

/// Fraction of treasury drained per tick during active war.
pub const WAR_ECONOMY_DRAIN_RATE: f32 = 0.02;
/// Population loss per unit of estimated casualties.
pub const CASUALTIES_PER_ENERGY_UNIT: f32 = 0.001;

/// Output of one war-economy drain computation.
#[derive(Debug, Clone, PartialEq)]
pub struct WarEconomyDrain {
    /// Treasury units drained this tick.
    pub treasury_drain: i64,
    /// Personnel lost this tick.
    pub population_loss: u32,
    /// True when the treasury has fallen below 10% of its starting value.
    pub economically_exhausted: bool,
}

/// Compute the war-economy drain for one tick.
///
/// Returns zero drain when `at_war` is false. Both `treasury_drain` and
/// `population_loss` are monotonic in their respective inputs.
pub fn compute_war_economy_drain(
    treasury: i64,
    estimated_casualties: u32,
    at_war: bool,
) -> WarEconomyDrain {
    if !at_war {
        return WarEconomyDrain {
            treasury_drain: 0,
            population_loss: 0,
            economically_exhausted: false,
        };
    }
    let treasury_drain = (treasury as f32 * WAR_ECONOMY_DRAIN_RATE) as i64;
    let population_loss = (estimated_casualties as f32 * CASUALTIES_PER_ENERGY_UNIT) as u32;
    let remaining = treasury - treasury_drain;
    let exhaustion_floor = (treasury / 10).max(10);
    let economically_exhausted = remaining <= exhaustion_floor;
    WarEconomyDrain {
        treasury_drain,
        population_loss,
        economically_exhausted,
    }
}

/// Apply a drain result to a treasury balance, clamping to zero.
pub fn apply_war_drain(treasury: i64, drain: &WarEconomyDrain) -> i64 {
    (treasury - drain.treasury_drain).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn war_drains_economy() {
        let drain = compute_war_economy_drain(10_000, 50, true);
        assert!(drain.treasury_drain > 0);
    }

    #[test]
    fn peace_does_not_drain_economy() {
        let drain = compute_war_economy_drain(10_000, 0, false);
        assert_eq!(drain.treasury_drain, 0);
        assert_eq!(drain.population_loss, 0);
    }

    #[test]
    fn treasury_clamps_to_zero() {
        let drain = WarEconomyDrain {
            treasury_drain: 100_000,
            population_loss: 0,
            economically_exhausted: false,
        };
        let new_treasury = apply_war_drain(50, &drain);
        assert_eq!(new_treasury, 0);
    }

    #[test]
    fn casualty_to_pop_connection() {
        let drain = compute_war_economy_drain(100_000, 1000, true);
        assert!(drain.population_loss > 0);
    }

    #[test]
    fn economically_exhausted_flag_not_set_in_peace() {
        assert!(!compute_war_economy_drain(10_000, 0, false).economically_exhausted);
    }

    #[test]
    fn drain_is_deterministic() {
        let a = compute_war_economy_drain(50_000, 200, true);
        let b = compute_war_economy_drain(50_000, 200, true);
        assert_eq!(a, b);
    }

    #[test]
    fn apply_drain_reduces_treasury() {
        let drain = compute_war_economy_drain(10_000, 0, true);
        let remaining = apply_war_drain(10_000, &drain);
        assert!(remaining < 10_000);
    }
}
