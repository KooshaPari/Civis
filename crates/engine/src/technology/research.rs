//! Emergent research rate calculation (FR-CIV-TECH).
//!
//! Research progress per tick is a function of population size, food surplus,
//! and the number of active institutions. The formula is intentionally simple
//! and fully deterministic — no RNG required.

/// Calculate research points generated per simulation tick.
///
/// # Parameters
/// - `population`: Current settlement population.
/// - `food_surplus`: Net food units above subsistence (may be negative).
/// - `institution_count`: Number of active scholarly/institutional buildings.
///
/// # Returns
/// Research points per tick as an `f32`. Returns `0.0` for degenerate inputs.
#[must_use]
pub fn calculate_research_rate(
    population: u32,
    food_surplus: f32,
    institution_count: u32,
) -> f32 {
    const BASE_RATE: f32 = 1.0;
    const SURPLUS_SCALE: f32 = 0.1;
    const INST_MULTIPLIER: f32 = 0.5;

    let pop_factor = (f32::from(u16::MAX).min(population as f32) + 1.0).ln();
    let surplus_factor = 1.0 + (food_surplus * SURPLUS_SCALE).max(-0.9);
    let inst_factor = 1.0 + institution_count as f32 * INST_MULTIPLIER;

    BASE_RATE * pop_factor * surplus_factor * inst_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_rate_scales_with_population() {
        let low = calculate_research_rate(100, 0.0, 0);
        let high = calculate_research_rate(10_000, 0.0, 0);
        assert!(
            high > low,
            "research rate must increase with population: low={low}, high={high}"
        );
    }

    #[test]
    fn research_rate_scales_with_food_surplus() {
        let deficit = calculate_research_rate(1_000, -5.0, 0);
        let surplus = calculate_research_rate(1_000, 10.0, 0);
        assert!(
            surplus > deficit,
            "surplus should yield higher rate: deficit={deficit}, surplus={surplus}"
        );
    }

    #[test]
    fn research_rate_scales_with_institutions() {
        let none = calculate_research_rate(1_000, 0.0, 0);
        let many = calculate_research_rate(1_000, 0.0, 5);
        assert!(
            many > none,
            "more institutions -> higher rate: none={none}, many={many}"
        );
    }

    #[test]
    fn research_rate_zero_population_is_non_negative() {
        let rate = calculate_research_rate(0, 0.0, 0);
        assert!(rate >= 0.0, "rate must be non-negative for zero pop");
    }
}
