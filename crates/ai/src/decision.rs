//! Utility-based decision making for population agents.
//!
//! Implements FR-AI-001: Each agent evaluates available actions based on
//! utility scores influenced by needs, mood, goals, and social context.

use crate::goal::{Goal, Need};
use crate::mood::MoodState;
use std::collections::HashMap;

/// A candidate action an agent can take.
#[derive(Debug, Clone)]
pub struct Action {
    /// Unique identifier for the action.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Base utility score before modifiers.
    pub base_utility: f64,
    /// Resource cost to perform this action.
    pub resource_cost: f64,
    /// Risk factor (0.0 = safe, 1.0 = very risky).
    pub risk_factor: f64,
}

/// Weight factors for utility calculation.
#[derive(Debug, Clone, Default)]
pub struct UtilityWeights {
    /// Weight for need satisfaction contribution.
    pub need_satisfaction: f64,
    /// Weight for mood alignment contribution.
    pub mood_alignment: f64,
    /// Weight for goal progress contribution.
    pub goal_progress: f64,
    /// Weight for social influence contribution.
    pub social_influence: f64,
    /// Weight for risk aversion penalty.
    pub risk_aversion: f64,
}

/// Result of utility evaluation for an action.
#[derive(Debug, Clone)]
pub struct UtilityScore {
    /// ID of the evaluated action.
    pub action_id: String,
    /// Total utility score.
    pub total: f64,
    /// Breakdown of score components.
    pub breakdown: HashMap<String, f64>,
}

/// Evaluate utility of all candidate actions for an agent.
pub fn evaluate_actions(
    actions: &[Action],
    weights: &UtilityWeights,
    needs: &[Need],
    mood: &MoodState,
    goals: &[Box<dyn Goal>],
    social_pressure: f64,
) -> Vec<UtilityScore> {
    actions.iter().map(|action| {
        let mut breakdown = HashMap::new();
        
        // Need satisfaction: average of unmet needs this action addresses
        let need_score = weights.need_satisfaction * action.base_utility;
        breakdown.insert("needs".to_string(), need_score);
        
        // Mood alignment: use valence to determine mood factor
        // Positive valence (Happy/Content) boosts utility, negative (Anxious/Angry) reduces it
        let mood_factor = match mood.valence {
            v if v > 0.5 => 1.2,    // Elated
            v if v > 0.1 => 1.1,    // Content
            v if v > -0.1 => 1.0,   // Neutral
            v if v > -0.5 => 0.8,   // Displeased
            _ => 0.6,               // Miserable
        };
        let mood_score = weights.mood_alignment * action.base_utility * mood_factor;
        breakdown.insert("mood".to_string(), mood_score);
        
        // Goal progress: how much this action advances highest-priority goal
        let goal_score = if let Some(top_goal) = goals.first() {
            weights.goal_progress * top_goal.utility(needs) as f64
        } else {
            0.0
        };
        breakdown.insert("goals".to_string(), goal_score);
        
        // Social influence: peer pressure factor
        let social_score = weights.social_influence * social_pressure;
        breakdown.insert("social".to_string(), social_score);
        
        // Risk penalty
        let risk_penalty = weights.risk_aversion * action.risk_factor * action.resource_cost;
        breakdown.insert("risk".to_string(), -risk_penalty);
        
        let total: f64 = breakdown.values().sum();
        
        UtilityScore {
            action_id: action.id.clone(),
            total,
            breakdown,
        }
    }).collect()
}

/// Select the best action from scored candidates.
pub fn select_best(scores: &[UtilityScore]) -> Option<&UtilityScore> {
    scores.iter().max_by(|a, b| a.total.partial_cmp(&b.total).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mood::MoodState;

    fn test_action(id: &str, utility: f64, cost: f64, risk: f64) -> Action {
        Action {
            id: id.to_string(),
            name: format!("Action {}", id),
            base_utility: utility,
            resource_cost: cost,
            risk_factor: risk,
        }
    }

    #[test]
    fn test_evaluate_single_action() {
        let actions = vec![test_action("work", 0.8, 0.2, 0.1)];
        let weights = UtilityWeights {
            need_satisfaction: 1.0,
            mood_alignment: 1.0,
            goal_progress: 1.0,
            social_influence: 1.0,
            risk_aversion: 1.0,
        };
        let needs = vec![Need::Hunger(0.5)];
        let goals: Vec<Box<dyn Goal>> = vec![];
        
        let scores = evaluate_actions(&actions, &weights, &needs, &MoodState::neutral(), &goals, 0.0);
        assert_eq!(scores.len(), 1);
        assert!(scores[0].total > 0.0);
    }

    #[test]
    fn test_select_best_action() {
        let scores = vec![
            UtilityScore { action_id: "a".into(), total: 0.5, breakdown: HashMap::new() },
            UtilityScore { action_id: "b".into(), total: 0.9, breakdown: HashMap::new() },
            UtilityScore { action_id: "c".into(), total: 0.3, breakdown: HashMap::new() },
        ];
        let best = select_best(&scores).unwrap();
        assert_eq!(best.action_id, "b");
    }

    #[test]
    fn test_mood_affects_utility() {
        let actions = vec![test_action("socialize", 0.5, 0.1, 0.0)];
        let weights = UtilityWeights { mood_alignment: 1.0, ..Default::default() };
        let needs = vec![];
        let goals: Vec<Box<dyn Goal>> = vec![];
        
        let mut happy_mood = MoodState::neutral();
        happy_mood.valence = 0.8;
        
        let mut sad_mood = MoodState::neutral();
        sad_mood.valence = -0.8;
        
        let happy_scores = evaluate_actions(&actions, &weights, &needs, &happy_mood, &goals, 0.0);
        let sad_scores = evaluate_actions(&actions, &weights, &needs, &sad_mood, &goals, 0.0);
        
        assert!(happy_scores[0].total > sad_scores[0].total);
    }

    #[test]
    fn test_risk_penalty() {
        let safe = test_action("safe_action", 0.5, 0.1, 0.0);
        let risky = test_action("risky_action", 0.5, 0.1, 0.9);
        let weights = UtilityWeights { risk_aversion: 1.0, ..Default::default() };
        let needs = vec![];
        let goals: Vec<Box<dyn Goal>> = vec![];
        
        let scores = evaluate_actions(&[safe, risky], &weights, &needs, &MoodState::neutral(), &goals, 0.0);
        assert!(scores[0].total > scores[1].total);
    }

    #[test]
    fn test_social_influence() {
        let actions = vec![test_action("follow_peers", 0.5, 0.1, 0.0)];
        let weights = UtilityWeights { social_influence: 1.0, ..Default::default() };
        let needs = vec![];
        let goals: Vec<Box<dyn Goal>> = vec![];
        
        let no_pressure = evaluate_actions(&actions, &weights, &needs, &MoodState::neutral(), &goals, 0.0);
        let high_pressure = evaluate_actions(&actions, &weights, &needs, &MoodState::neutral(), &goals, 1.0);
        
        assert!(high_pressure[0].total > no_pressure[0].total);
    }

    #[test]
    fn test_empty_actions() {
        let scores = evaluate_actions(&[], &UtilityWeights::default(), &[], &MoodState::neutral(), &[], 0.0);
        assert!(scores.is_empty());
    }
}
