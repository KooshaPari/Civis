//! Demographic simulation with age cohorts.
//!
//! The update rule combines a Leslie-style cohort transition with a logistic
//! crowding term: fertile cohorts generate births, cohorts age forward, and the
//! total population is capped by carrying capacity.

/// One age cohort in the population.
#[derive(Clone, Debug)]
pub struct AgeGroup {
    pub label: &'static str,
    pub count: u32,
    pub birth_rate: f32,
    pub death_rate: f32,
}

/// Demographic state for a settlement or region.
#[derive(Clone, Debug)]
pub struct Demographics {
    pub groups: Vec<AgeGroup>,
    pub carrying_capacity: u32,
}

const FOOD_DEATH_WEIGHT: f32 = 0.35;
const DISEASE_DEATH_WEIGHT: f32 = 0.45;
const CARRYING_CAPACITY_FOOD_SCALE: f32 = 10_000.0;

fn adjusted_death_rate(base: f32, food_per_capita: f32, disease_factor: f32, crowding: f32) -> f32 {
    let famine_penalty = (1.0 - food_per_capita).max(0.0) * FOOD_DEATH_WEIGHT;
    let disease_penalty = disease_factor.max(0.0) * DISEASE_DEATH_WEIGHT;
    let crowding_penalty = crowding.max(0.0) * 0.25;
    (base + famine_penalty + disease_penalty + crowding_penalty).clamp(0.0, 0.95)
}

fn logistic_birth_multiplier(population: u32, carrying_capacity: u32) -> f32 {
    if carrying_capacity == 0 {
        return 0.0;
    }

    let load = population as f32 / carrying_capacity as f32;
    (1.0 - load).clamp(0.0, 1.0)
}

fn round_population(value: f32) -> u32 {
    if value <= 0.0 {
        0
    } else if value >= u32::MAX as f32 {
        u32::MAX
    } else {
        value.round() as u32
    }
}

/// Advance demographic cohorts by one tick.
///
/// The model:
/// - aggregates births from fertile cohorts,
/// - ages surviving cohorts forward one band,
/// - raises death rates when food is scarce or disease is present,
/// - applies a logistic pressure term based on carrying capacity,
/// - caps the resulting population at carrying capacity.
pub fn tick_demographics(d: &mut Demographics, food_per_capita: f32, disease_factor: f32) {
    if d.groups.is_empty() {
        return;
    }

    let population = total_population(d);
    let capacity = d.carrying_capacity.max(1);
    let fertility_pressure = logistic_birth_multiplier(population, capacity);
    let crowding = (population as f32 / capacity as f32).clamp(0.0, 2.0);

    let mut next_groups = vec![0_u32; d.groups.len()];
    let mut births = 0.0_f32;

    for (idx, group) in d.groups.iter().enumerate() {
        let death_rate = adjusted_death_rate(
            group.death_rate,
            food_per_capita,
            disease_factor,
            crowding,
        );
        let survivors = (group.count as f32) * (1.0 - death_rate);
        let survivors = round_population(survivors);

        if idx + 1 < next_groups.len() {
            next_groups[idx + 1] = next_groups[idx + 1].saturating_add(survivors);
        } else {
            next_groups[idx] = next_groups[idx].saturating_add(survivors);
        }

        births += group.count as f32 * group.birth_rate;
    }

    let food_bonus = food_per_capita.max(0.0).min(1.5);
    let newborns = round_population(births * fertility_pressure * food_bonus);
    next_groups[0] = next_groups[0].saturating_add(newborns);

    let next_total: u32 = next_groups.iter().copied().sum();
    if next_total > capacity {
        let scale = capacity as f32 / next_total as f32;
        let mut resized_total = 0_u32;
        for count in &mut next_groups {
            *count = round_population(*count as f32 * scale);
            resized_total = resized_total.saturating_add(*count);
        }

        if resized_total > capacity {
            let mut excess = resized_total - capacity;
            for count in next_groups.iter_mut().rev() {
                if excess == 0 {
                    break;
                }
                let trim = (*count).min(excess);
                *count -= trim;
                excess -= trim;
            }
        }
    }

    for (group, count) in d.groups.iter_mut().zip(next_groups) {
        group.count = count;
    }
}

/// Sum all age cohorts.
pub fn total_population(d: &Demographics) -> u32 {
    d.groups.iter().map(|group| group.count).sum()
}

/// Convert food supply into a population carrying capacity.
pub fn carrying_capacity_from_food(food_supply: f32) -> u32 {
    if food_supply <= 0.0 {
        return 1;
    }

    let capacity = food_supply * CARRYING_CAPACITY_FOOD_SCALE;
    if capacity >= u32::MAX as f32 {
        u32::MAX
    } else {
        capacity.floor().max(1.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_demographics() -> Demographics {
        Demographics {
            carrying_capacity: 10_000,
            groups: vec![
                AgeGroup {
                    label: "children",
                    count: 2_000,
                    birth_rate: 0.0,
                    death_rate: 0.03,
                },
                AgeGroup {
                    label: "adults",
                    count: 5_000,
                    birth_rate: 0.18,
                    death_rate: 0.02,
                },
                AgeGroup {
                    label: "elders",
                    count: 1_000,
                    birth_rate: 0.0,
                    death_rate: 0.06,
                },
            ],
        }
    }

    #[test]
    fn total_population_sums_cohorts() {
        let d = sample_demographics();
        assert_eq!(total_population(&d), 8_000);
    }

    #[test]
    fn carrying_capacity_scales_with_food_supply() {
        assert_eq!(carrying_capacity_from_food(0.0), 1);
        assert_eq!(carrying_capacity_from_food(12.5), 125_000);
    }

    #[test]
    fn favorable_conditions_grow_population() {
        let mut d = sample_demographics();
        tick_demographics(&mut d, 1.25, 0.0);
        assert!(total_population(&d) > 8_000);
        assert!(total_population(&d) <= d.carrying_capacity);
    }

    #[test]
    fn famine_and_plague_reduce_population() {
        let mut d = Demographics {
            carrying_capacity: 10_000,
            groups: vec![
                AgeGroup {
                    label: "children",
                    count: 3_000,
                    birth_rate: 0.0,
                    death_rate: 0.04,
                },
                AgeGroup {
                    label: "adults",
                    count: 4_500,
                    birth_rate: 0.12,
                    death_rate: 0.03,
                },
                AgeGroup {
                    label: "elders",
                    count: 1_500,
                    birth_rate: 0.0,
                    death_rate: 0.08,
                },
            ],
        };

        let before = total_population(&d);
        tick_demographics(&mut d, 0.65, 1.25);
        let after = total_population(&d);

        assert!(after < before, "famine and plague should reduce population");
        assert!(after > 0);
    }
}
