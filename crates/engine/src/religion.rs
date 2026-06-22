use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeliefConcept {
    NaturalAgent,
    MoralOverseer,
    Afterlife,
    Taboo { action: String },
    Ritual { cost: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Belief {
    pub concept: BeliefConcept,
    pub strength: f32,
    pub social_spread: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Religion {
    pub beliefs: Vec<Belief>,
    pub cohesion: f32,
    pub member_count: u32,
    pub age_ticks: u64,
}

fn clamp01(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn belief_strength(hardship: f32, group_size: u32, agent_detection_bias: f32) -> f32 {
    let hardship = clamp01(hardship);
    let group_factor = clamp01(group_size as f32 / 100.0);
    let bias = clamp01(agent_detection_bias);
    clamp01(hardship * 0.7 + group_factor * 0.2 + bias * 0.1)
}

pub fn emerge_belief(hardship: f32, group_size: u32, agent_detection_bias: f32) -> Option<Belief> {
    let hardship = clamp01(hardship);
    let bias = clamp01(agent_detection_bias);

    let concept = if group_size > 50 {
        Some(BeliefConcept::MoralOverseer)
    } else if hardship > 0.7 {
        if bias >= 0.5 {
            Some(BeliefConcept::NaturalAgent)
        } else {
            Some(BeliefConcept::Taboo {
                action: "forbidden_contact".to_string(),
            })
        }
    } else {
        None
    }?;

    let social_spread = clamp01((group_size as f32 / 100.0) * 0.6 + hardship * 0.3 + bias * 0.1);

    Some(Belief {
        concept,
        strength: belief_strength(hardship, group_size, bias),
        social_spread,
    })
}

pub fn spread_religion(rel: &mut Religion, nearby_agents: u32, tick: u64) {
    rel.age_ticks = tick;

    if rel.beliefs.is_empty() {
        return;
    }

    let nearby_factor =
        clamp01(nearby_agents as f32 / (rel.member_count.max(1) as f32 + nearby_agents as f32));
    let ritual_load: f32 = rel
        .beliefs
        .iter()
        .filter_map(|belief| match &belief.concept {
            BeliefConcept::Ritual { cost } => {
                Some(cost.max(0.0) * belief.strength * belief.social_spread)
            }
            _ => None,
        })
        .sum();

    let shared_ritual_performance = clamp01(ritual_load * nearby_factor);
    let cohesion_gain = shared_ritual_performance * 0.2;
    rel.cohesion = clamp01(rel.cohesion + cohesion_gain);

    for belief in &mut rel.beliefs {
        let spread_gain = nearby_factor * 0.05 + shared_ritual_performance * 0.1;
        belief.social_spread = clamp01(belief.social_spread + spread_gain);
        belief.strength = clamp01(belief.strength + shared_ritual_performance * 0.03);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emerge_belief_uses_high_hardship_agent_detection_bias_for_natural_agent() {
        let belief = emerge_belief(0.9, 12, 0.8).expect("belief should emerge");
        assert_eq!(belief.concept, BeliefConcept::NaturalAgent);
        assert!(belief.strength > 0.0);
        assert!(belief.social_spread > 0.0);
    }

    #[test]
    fn emerge_belief_uses_high_hardship_low_bias_for_taboo() {
        let belief = emerge_belief(0.95, 12, 0.2).expect("belief should emerge");
        assert_eq!(
            belief.concept,
            BeliefConcept::Taboo {
                action: "forbidden_contact".to_string(),
            }
        );
    }

    #[test]
    fn emerge_belief_promotes_large_groups_to_moral_overseer() {
        let belief = emerge_belief(0.2, 51, 0.1).expect("belief should emerge");
        assert_eq!(belief.concept, BeliefConcept::MoralOverseer);
    }

    #[test]
    fn spread_religion_grows_cohesion_from_shared_ritual_performance() {
        let mut religion = Religion {
            beliefs: vec![
                Belief {
                    concept: BeliefConcept::Ritual { cost: 0.8 },
                    strength: 0.9,
                    social_spread: 0.7,
                },
                Belief {
                    concept: BeliefConcept::NaturalAgent,
                    strength: 0.5,
                    social_spread: 0.4,
                },
            ],
            cohesion: 0.3,
            member_count: 40,
            age_ticks: 2,
        };

        spread_religion(&mut religion, 30, 99);

        assert!(religion.cohesion > 0.3);
        assert_eq!(religion.age_ticks, 99);
        assert!(religion
            .beliefs
            .iter()
            .all(|belief| belief.social_spread >= 0.4));
    }
}
