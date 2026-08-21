//! Goal tree for multi-step AI planning (FR-AI-002).
//!
//! Extends FR-AI-001's utility-based goal selection with a hierarchical goal
//! tree that supports active evaluation, step-by-step execution, sub-goals,
//! and blocking conditions.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::goal::Need;
use crate::social_graph::AgentId;

/// Status of a goal returned by [`Goal::evaluate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoalStatus {
    /// Goal is actively being pursued.
    Active,
    /// Goal has been achieved.
    Completed,
    /// Goal has failed and cannot be recovered.
    Failed,
    /// Goal is blocked by a precondition that is not currently satisfied.
    Blocked,
}

/// Error type for goal tree operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum AIError {
    /// Goal execution encountered a resource shortage.
    #[error("insufficient resources: {0}")]
    InsufficientResources(String),

    /// Goal execution failed because a prerequisite was not met.
    #[error("dependency not met: {0}")]
    DependencyNotMet(String),

    /// Goal is in an invalid state for the requested operation.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// No viable path exists to complete the goal.
    #[error("no viable path: {0}")]
    NoViablePath(String),
}

/// 2D world position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

impl Position {
    /// Create a new position.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to `other`.
    #[must_use]
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Move toward `target` by at most `max_step` distance.
    /// Returns the new position (clamped so it never overshoots).
    #[must_use]
    pub fn move_toward(&self, target: &Self, max_step: f64) -> Self {
        let dist = self.distance_to(target);
        if dist <= max_step || dist < f64::EPSILON {
            return *target;
        }
        let ratio = max_step / dist;
        Self {
            x: self.x + (target.x - self.x) * ratio,
            y: self.y + (target.y - self.y) * ratio,
        }
    }
}

/// A simple resource inventory keyed by resource name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    /// Resource name -> quantity.
    pub stock: HashMap<String, f64>,
}

impl Resources {
    /// Create an empty inventory.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an inventory pre-populated with the given entries.
    #[must_use]
    pub fn from_pairs(pairs: &[(&str, f64)]) -> Self {
        Self {
            stock: pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect(),
        }
    }

    /// Get the quantity of a named resource. Returns `0.0` if absent.
    #[must_use]
    pub fn get(&self, name: &str) -> f64 {
        self.stock.get(name).copied().unwrap_or(0.0)
    }

    /// Add `amount` of a named resource.
    pub fn add(&mut self, name: &str, amount: f64) {
        *self.stock.entry(name.to_string()).or_insert(0.0) += amount;
    }

    /// Consume `amount` of a named resource. Returns `true` on success,
    /// `false` if the agent doesn't have enough.
    pub fn consume(&mut self, name: &str, amount: f64) -> bool {
        let current = self.get(name);
        if current + f64::EPSILON >= amount {
            let entry = self.stock.entry(name.to_string()).or_insert(0.0);
            *entry -= amount;
            true
        } else {
            false
        }
    }
}

/// An entity within the agent's perception range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearbyEntity {
    /// Unique entity identifier.
    pub id: u64,
    /// World position.
    pub position: Position,
    /// Entity kind label (e.g. "agent", "food_source", "shelter", "faction").
    pub kind: String,
    /// Faction id, if the entity belongs to one.
    pub faction_id: Option<u64>,
}

/// Sensory and state context passed to goals during evaluation and execution.
pub struct AgentContext {
    /// Unique agent identifier.
    pub agent_id: AgentId,
    /// Current world position.
    pub position: Position,
    /// Current resource inventory.
    pub resources: Resources,
    /// Entities within perception range.
    pub nearby_entities: Vec<NearbyEntity>,
    /// Current need vector (from FR-AI-001).
    pub needs: Vec<Need>,
    /// Relationship weights keyed by other agent id (positive = bond,
    /// negative = grudge).
    pub relationships: HashMap<AgentId, f64>,
    /// Id of the goal currently being executed, if any.
    pub current_goal: Option<String>,
    /// Current simulation tick.
    pub tick: u64,
}

impl AgentContext {
    /// Look up a specific need's urgency. Returns `0.0` if the need is absent.
    #[must_use]
    pub fn need_urgency(&self, target: Need) -> f32 {
        for need in &self.needs {
            if std::mem::discriminant(need) == std::mem::discriminant(&target) {
                return need.urgency();
            }
        }
        0.0
    }

    /// Total count of nearby agents (entities with kind "agent").
    #[must_use]
    pub fn nearby_agent_count(&self) -> usize {
        self.nearby_entities
            .iter()
            .filter(|e| e.kind == "agent")
            .count()
    }

    /// Find the nearest entity matching `kind`. Returns `None` if none found.
    #[must_use]
    pub fn nearest_entity(&self, kind: &str) -> Option<&NearbyEntity> {
        self.nearby_entities
            .iter()
            .filter(|e| e.kind == kind)
            .min_by(|a, b| {
                let da = self.position.distance_to(&a.position);
                let db = self.position.distance_to(&b.position);
                da.partial_cmp(&db).unwrap_or(Ordering::Equal)
            })
    }
}

/// A goal that can be evaluated and executed within an agent context.
///
/// This extends FR-AI-001's utility-based Goal trait with active evaluation
/// and execution, enabling multi-step planning through the GoalTree.
pub trait Goal: Send + Sync {
    /// Stable identifier for this goal.
    fn id(&self) -> &str;

    /// Evaluate this goal given the current agent context. Should be pure and
    /// cheap — called every tick for every goal in the tree.
    fn evaluate(&self, ctx: &AgentContext) -> GoalStatus;

    /// Execute one tick of this goal, mutating the agent context as needed.
    fn execute(&self, ctx: &mut AgentContext) -> Result<(), AIError>;

    /// Priority score — higher means more urgent.
    fn priority(&self) -> f64;
}

/// A hierarchical goal tree that manages a priority-ordered collection of
/// goals and sub-goals.
pub struct GoalTree {
    /// Top-level goals stored by id for O(1) lookup.
    goals: HashMap<String, Box<dyn Goal>>,
    /// Sub-goals keyed by parent goal id.
    sub_goals: HashMap<String, Vec<Box<dyn Goal>>>,
}

impl GoalTree {
    /// Create an empty goal tree.
    #[must_use]
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            sub_goals: HashMap::new(),
        }
    }

    /// Add a top-level goal. If a goal with the same id already exists it is
    /// replaced.
    pub fn add_goal(&mut self, goal: Box<dyn Goal>) {
        let id = goal.id().to_string();
        self.goals.insert(id, goal);
    }

    /// Add a sub-goal under `parent_id`. Returns `Err` if no parent with that
    /// id exists in the tree.
    pub fn add_sub_goal(&mut self, parent_id: &str, goal: Box<dyn Goal>) -> Result<(), AIError> {
        if !self.goals.contains_key(parent_id) {
            return Err(AIError::InvalidState(format!(
                "parent goal '{parent_id}' not found"
            )));
        }
        self.sub_goals
            .entry(parent_id.to_string())
            .or_default()
            .push(goal);
        Ok(())
    }

    /// Evaluate all top-level goals and return a reference to the
    /// highest-priority one whose status is `Active`.
    #[must_use]
    pub fn select_active<'a>(&'a self, ctx: &AgentContext) -> Option<&'a dyn Goal> {
        self.goals
            .iter()
            .filter(|(_, g)| matches!(g.evaluate(ctx), GoalStatus::Active))
            .max_by(|(id_a, g_a), (id_b, g_b)| {
                g_a.priority()
                    .partial_cmp(&g_b.priority())
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| id_a.cmp(id_b))
            })
            .map(|(_, g)| g.as_ref())
    }

    /// Number of top-level goals.
    #[must_use]
    pub fn len(&self) -> usize {
        self.goals.len()
    }

    /// Whether the tree is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }

    /// All top-level goal ids, in arbitrary order.
    #[must_use]
    pub fn goal_ids(&self) -> Vec<&str> {
        self.goals.keys().map(String::as_str).collect()
    }

    /// Look up a goal by id.
    #[must_use]
    pub fn get_goal(&self, id: &str) -> Option<&dyn Goal> {
        self.goals.get(id).map(|g| g.as_ref())
    }

    /// Look up sub-goals for a parent id.
    #[must_use]
    pub fn sub_goals_of(&self, parent_id: &str) -> Vec<&dyn Goal> {
        self.sub_goals
            .get(parent_id)
            .map(|v| v.iter().map(|g| g.as_ref()).collect())
            .unwrap_or_default()
    }

    /// Remove a goal (and all its sub-goals) from the tree.
    pub fn remove_goal(&mut self, id: &str) {
        self.goals.remove(id);
        self.sub_goals.remove(id);
    }

    /// Tick the goal tree: evaluate all goals, execute the best active one,
    /// and handle status transitions.
    ///
    /// Returns the id of a goal that completed or failed this tick, if any.
    pub fn tick(&mut self, ctx: &mut AgentContext) -> Option<String> {
        // Collect candidate ids and priorities.
        let mut candidates: Vec<(String, f64)> = self
            .goals
            .iter()
            .map(|(id, g)| (id.clone(), g.priority()))
            .collect();

        // Sort descending by priority (highest first), ties by id ascending.
        candidates.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        for (id, _) in &candidates {
            let status = match self.goals.get(id) {
                Some(g) => g.evaluate(ctx),
                None => continue,
            };

            match status {
                GoalStatus::Active => {
                    // Execute one step.
                    ctx.current_goal = Some(id.clone());
                    let exec_result = {
                        let goal = self.goals.get(id).unwrap();
                        goal.execute(ctx)
                    };

                    // Re-evaluate after execution to check for transition.
                    let new_status = match exec_result {
                        Ok(()) => match self.goals.get(id) {
                            Some(g) => g.evaluate(ctx),
                            None => GoalStatus::Failed,
                        },
                        Err(_) => GoalStatus::Failed,
                    };

                    match new_status {
                        GoalStatus::Completed | GoalStatus::Failed => {
                            if let Some(children) = self.sub_goals.remove(id) {
                                for child in children {
                                    let child_id = child.id().to_string();
                                    self.goals.insert(child_id, child);
                                }
                            }
                            self.goals.remove(id);
                            ctx.current_goal = None;
                            return Some(id.clone());
                        }
                        GoalStatus::Blocked => {
                            ctx.current_goal = None;
                        }
                        GoalStatus::Active => {
                            ctx.current_goal = None;
                        }
                    }
                    return None;
                }
                GoalStatus::Completed | GoalStatus::Failed => {
                    if let Some(children) = self.sub_goals.remove(id) {
                        for child in children {
                            let child_id = child.id().to_string();
                            self.goals.insert(child_id, child);
                        }
                    }
                    self.goals.remove(id);
                    ctx.current_goal = None;
                    return Some(id.clone());
                }
                GoalStatus::Blocked => {}
            }
        }

        None
    }
}

impl Default for GoalTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Concrete goal implementations
// ---------------------------------------------------------------------------

/// Distance threshold within which an agent is "at" a target.
const ARRIVAL_THRESHOLD: f64 = 0.5;
/// Base movement speed per tick.
const MOVE_SPEED: f64 = 1.0;
/// Hunger restored when the agent eats.
const FOOD_RESTORE: f32 = 0.3;
/// Safety restored when the agent reaches shelter.
const SHELTER_RESTORE: f32 = 0.4;

/// Agent searches for and moves toward the nearest food source.
#[derive(Debug)]
pub struct SeekFoodGoal {
    /// Hunger urgency threshold below which this goal is not pursued.
    pub hunger_threshold: f32,
}

impl Default for SeekFoodGoal {
    fn default() -> Self {
        Self {
            hunger_threshold: 0.3,
        }
    }
}

impl Goal for SeekFoodGoal {
    fn id(&self) -> &str {
        "seek_food"
    }

    fn evaluate(&self, ctx: &AgentContext) -> GoalStatus {
        let hunger = ctx.need_urgency(Need::Hunger(0.0));
        if hunger < self.hunger_threshold {
            return GoalStatus::Completed;
        }
        if ctx.nearest_entity("food_source").is_some() {
            GoalStatus::Active
        } else {
            GoalStatus::Blocked
        }
    }

    fn execute(&self, ctx: &mut AgentContext) -> Result<(), AIError> {
        let food = ctx
            .nearest_entity("food_source")
            .ok_or_else(|| AIError::NoViablePath("no food source in perception".into()))?
            .clone();

        let dist = ctx.position.distance_to(&food.position);
        if dist <= ARRIVAL_THRESHOLD {
            let hunger_before = ctx.need_urgency(Need::Hunger(0.0));
            let restored = (hunger_before - FOOD_RESTORE).max(0.0);
            if let Some(need) = ctx.needs.iter_mut().find(|n| matches!(n, Need::Hunger(_))) {
                *need = Need::Hunger(restored);
            }
        } else {
            ctx.position = ctx.position.move_toward(&food.position, MOVE_SPEED);
        }
        Ok(())
    }

    fn priority(&self) -> f64 {
        80.0
    }
}

/// Agent finds or builds shelter to satisfy the Safety need.
#[derive(Debug)]
pub struct SeekShelterGoal {
    /// Safety urgency threshold below which this goal is not pursued.
    pub safety_threshold: f32,
}

impl Default for SeekShelterGoal {
    fn default() -> Self {
        Self {
            safety_threshold: 0.4,
        }
    }
}

impl Goal for SeekShelterGoal {
    fn id(&self) -> &str {
        "seek_shelter"
    }

    fn evaluate(&self, ctx: &AgentContext) -> GoalStatus {
        let safety = ctx.need_urgency(Need::Safety(0.0));
        if safety < self.safety_threshold {
            return GoalStatus::Completed;
        }
        if ctx.nearest_entity("shelter").is_some() {
            GoalStatus::Active
        } else {
            GoalStatus::Blocked
        }
    }

    fn execute(&self, ctx: &mut AgentContext) -> Result<(), AIError> {
        let shelter = ctx
            .nearest_entity("shelter")
            .ok_or_else(|| AIError::NoViablePath("no shelter in perception".into()))?
            .clone();

        let dist = ctx.position.distance_to(&shelter.position);
        if dist <= ARRIVAL_THRESHOLD {
            let safety_before = ctx.need_urgency(Need::Safety(0.0));
            let restored = (safety_before - SHELTER_RESTORE).max(0.0);
            if let Some(need) = ctx.needs.iter_mut().find(|n| matches!(n, Need::Safety(_))) {
                *need = Need::Safety(restored);
            }
        } else {
            ctx.position = ctx.position.move_toward(&shelter.position, MOVE_SPEED);
        }
        Ok(())
    }

    fn priority(&self) -> f64 {
        90.0
    }
}

/// Agent seeks interaction with other agents.
#[derive(Debug)]
pub struct SocializeGoal {
    /// Social urgency threshold below which this goal is not pursued.
    pub social_threshold: f32,
}

impl Default for SocializeGoal {
    fn default() -> Self {
        Self {
            social_threshold: 0.3,
        }
    }
}

impl Goal for SocializeGoal {
    fn id(&self) -> &str {
        "socialize"
    }

    fn evaluate(&self, ctx: &AgentContext) -> GoalStatus {
        let social = ctx.need_urgency(Need::Social(0.0));
        if social < self.social_threshold {
            return GoalStatus::Completed;
        }
        if ctx.nearby_agent_count() > 0 {
            GoalStatus::Active
        } else {
            GoalStatus::Blocked
        }
    }

    fn execute(&self, ctx: &mut AgentContext) -> Result<(), AIError> {
        let target = ctx
            .nearby_entities
            .iter()
            .filter(|e| e.kind == "agent" && e.id != ctx.agent_id)
            .min_by(|a, b| {
                let da = ctx.position.distance_to(&a.position);
                let db = ctx.position.distance_to(&b.position);
                da.partial_cmp(&db).unwrap_or(Ordering::Equal)
            })
            .ok_or_else(|| AIError::NoViablePath("no other agent in perception".into()))?
            .clone();

        let dist = ctx.position.distance_to(&target.position);
        if dist <= ARRIVAL_THRESHOLD {
            let social_before = ctx.need_urgency(Need::Social(0.0));
            let restored = (social_before - 0.25).max(0.0);
            if let Some(need) = ctx.needs.iter_mut().find(|n| matches!(n, Need::Social(_))) {
                *need = Need::Social(restored);
            }
            *ctx.relationships.entry(target.id).or_insert(0.0) += 0.1;
        } else {
            ctx.position = ctx.position.move_toward(&target.position, MOVE_SPEED);
        }
        Ok(())
    }

    fn priority(&self) -> f64 {
        50.0
    }
}

/// Agent attempts to trade surplus resources with nearby factions.
#[derive(Debug)]
pub struct TradeGoal {
    /// Minimum food surplus required to attempt a trade.
    pub min_surplus: f64,
}

impl Default for TradeGoal {
    fn default() -> Self {
        Self { min_surplus: 2.0 }
    }
}

impl Goal for TradeGoal {
    fn id(&self) -> &str {
        "trade"
    }

    fn evaluate(&self, ctx: &AgentContext) -> GoalStatus {
        let food = ctx.resources.get("food");
        if food < self.min_surplus {
            return GoalStatus::Completed;
        }
        if ctx.nearby_entities.iter().any(|e| e.kind == "faction") {
            GoalStatus::Active
        } else {
            GoalStatus::Blocked
        }
    }

    fn execute(&self, ctx: &mut AgentContext) -> Result<(), AIError> {
        let faction = ctx
            .nearby_entities
            .iter()
            .find(|e| e.kind == "faction")
            .ok_or_else(|| AIError::NoViablePath("no faction in perception".into()))?
            .clone();

        let dist = ctx.position.distance_to(&faction.position);
        if dist <= ARRIVAL_THRESHOLD {
            if ctx.resources.consume("food", 1.0) {
                let trade_resource = ctx
                    .needs
                    .iter()
                    .filter(|n| !matches!(n, Need::Hunger(_)))
                    .max_by(|a, b| {
                        a.urgency()
                            .partial_cmp(&b.urgency())
                            .unwrap_or(Ordering::Equal)
                    })
                    .map(|n| match n {
                        Need::Rest(_) => "rest_materials",
                        Need::Safety(_) => "building_materials",
                        Need::Social(_) => "luxury_goods",
                        Need::Purpose(_) => "knowledge",
                        Need::Hunger(_) => unreachable!(),
                    })
                    .unwrap_or("building_materials");

                ctx.resources.add(trade_resource, 1.0);

                if let Some(fid) = faction.faction_id {
                    *ctx.relationships.entry(fid).or_insert(0.0) += 0.05;
                }
            }
        } else {
            ctx.position = ctx.position.move_toward(&faction.position, MOVE_SPEED);
        }
        Ok(())
    }

    fn priority(&self) -> f64 {
        40.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunger_urgency(ctx: &AgentContext) -> f32 {
        ctx.needs
            .iter()
            .find_map(|n| {
                if let Need::Hunger(u) = n {
                    Some(*u)
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    fn safety_urgency(ctx: &AgentContext) -> f32 {
        ctx.needs
            .iter()
            .find_map(|n| {
                if let Need::Safety(u) = n {
                    Some(*u)
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    fn social_urgency(ctx: &AgentContext) -> f32 {
        ctx.needs
            .iter()
            .find_map(|n| {
                if let Need::Social(u) = n {
                    Some(*u)
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    fn make_ctx(needs: Vec<Need>, nearby: Vec<NearbyEntity>) -> AgentContext {
        AgentContext {
            agent_id: 1,
            position: Position::new(0.0, 0.0),
            resources: Resources::from_pairs(&[("food", 5.0)]),
            nearby_entities: nearby,
            needs,
            relationships: HashMap::new(),
            current_goal: None,
            tick: 0,
        }
    }

    // -- Position tests ----------------------------------------------------

    #[test]
    fn position_distance_to_is_symmetric() {
        let a = Position::new(0.0, 0.0);
        let b = Position::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
        assert!((b.distance_to(&a) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn position_distance_to_self_is_zero() {
        let a = Position::new(7.0, -3.0);
        assert!((a.distance_to(&a)).abs() < 1e-10);
    }

    #[test]
    fn position_move_toward_clamps() {
        let origin = Position::new(0.0, 0.0);
        let far = Position::new(100.0, 0.0);
        let step = origin.move_toward(&far, 5.0);
        assert!((step.x - 5.0).abs() < 1e-10);
        assert!((step.y).abs() < 1e-10);
    }

    #[test]
    fn position_move_toward_arrives_when_close() {
        let a = Position::new(0.0, 0.0);
        let b = Position::new(0.3, 0.4); // dist = 0.5
        let step = a.move_toward(&b, 1.0);
        assert_eq!(step, b);
    }

    // -- Resources tests ---------------------------------------------------

    #[test]
    fn resources_get_default_zero() {
        let r = Resources::new();
        assert!((r.get("food") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn resources_add_and_get() {
        let mut r = Resources::new();
        r.add("food", 3.0);
        r.add("food", 2.0);
        assert!((r.get("food") - 5.0).abs() < 1e-10);
    }

    #[test]
    fn resources_consume_succeeds() {
        let mut r = Resources::from_pairs(&[("food", 5.0)]);
        assert!(r.consume("food", 3.0));
        assert!((r.get("food") - 2.0).abs() < 1e-10);
    }

    #[test]
    fn resources_consume_fails_insufficient() {
        let mut r = Resources::from_pairs(&[("food", 1.0)]);
        assert!(!r.consume("food", 2.0));
        assert!((r.get("food") - 1.0).abs() < 1e-10);
    }

    // -- SeekFoodGoal tests ------------------------------------------------

    #[test]
    fn seek_food_completed_when_not_hungry() {
        let ctx = make_ctx(vec![Need::Hunger(0.1)], vec![]);
        let goal = SeekFoodGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Completed);
    }

    #[test]
    fn seek_food_blocked_when_no_food_visible() {
        let ctx = make_ctx(vec![Need::Hunger(0.8)], vec![]);
        let goal = SeekFoodGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Blocked);
    }

    #[test]
    fn seek_food_active_when_hungry_and_food_visible() {
        let food = NearbyEntity {
            id: 10,
            position: Position::new(5.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let ctx = make_ctx(vec![Need::Hunger(0.8)], vec![food]);
        let goal = SeekFoodGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Active);
    }

    #[test]
    fn seek_food_moves_toward_food() {
        let food = NearbyEntity {
            id: 10,
            position: Position::new(5.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Hunger(0.8)], vec![food]);
        let goal = SeekFoodGoal::default();
        goal.execute(&mut ctx).unwrap();
        assert!(ctx.position.x > 0.0, "agent should have moved toward food");
    }

    #[test]
    fn seek_food_arrives_and_eats() {
        let food = NearbyEntity {
            id: 10,
            position: Position::new(0.3, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Hunger(0.9)], vec![food]);
        let goal = SeekFoodGoal::default();
        goal.execute(&mut ctx).unwrap();
        let new_hunger = hunger_urgency(&ctx);
        assert!(
            new_hunger < 0.9,
            "hunger should have decreased from 0.9, got {new_hunger}"
        );
    }

    // -- SeekShelterGoal tests ---------------------------------------------

    #[test]
    fn seek_shelter_completed_when_safe() {
        let ctx = make_ctx(vec![Need::Safety(0.1)], vec![]);
        let goal = SeekShelterGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Completed);
    }

    #[test]
    fn seek_shelter_blocked_when_no_shelter() {
        let ctx = make_ctx(vec![Need::Safety(0.8)], vec![]);
        let goal = SeekShelterGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Blocked);
    }

    #[test]
    fn seek_shelter_active_when_unsafe_and_shelter_visible() {
        let shelter = NearbyEntity {
            id: 20,
            position: Position::new(3.0, 4.0),
            kind: "shelter".into(),
            faction_id: None,
        };
        let ctx = make_ctx(vec![Need::Safety(0.8)], vec![shelter]);
        let goal = SeekShelterGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Active);
    }

    #[test]
    fn seek_shelter_arrives_and_safens() {
        let shelter = NearbyEntity {
            id: 20,
            position: Position::new(0.2, 0.0),
            kind: "shelter".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Safety(0.9)], vec![shelter]);
        let goal = SeekShelterGoal::default();
        goal.execute(&mut ctx).unwrap();
        let new_safety = safety_urgency(&ctx);
        assert!(
            new_safety < 0.9,
            "safety should have improved, got {new_safety}"
        );
    }

    // -- SocializeGoal tests -----------------------------------------------

    #[test]
    fn socialize_completed_when_not_lonely() {
        let ctx = make_ctx(vec![Need::Social(0.1)], vec![]);
        let goal = SocializeGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Completed);
    }

    #[test]
    fn socialize_blocked_when_alone() {
        let ctx = make_ctx(vec![Need::Social(0.8)], vec![]);
        let goal = SocializeGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Blocked);
    }

    #[test]
    fn socialize_active_when_lonely_and_agent_nearby() {
        let other = NearbyEntity {
            id: 2,
            position: Position::new(3.0, 0.0),
            kind: "agent".into(),
            faction_id: None,
        };
        let ctx = make_ctx(vec![Need::Social(0.8)], vec![other]);
        let goal = SocializeGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Active);
    }

    #[test]
    fn socialize_moves_toward_agent() {
        let other = NearbyEntity {
            id: 2,
            position: Position::new(5.0, 0.0),
            kind: "agent".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Social(0.8)], vec![other]);
        let goal = SocializeGoal::default();
        goal.execute(&mut ctx).unwrap();
        assert!(ctx.position.x > 0.0);
    }

    #[test]
    fn socialize_interacts_and_satisfies_need() {
        let other = NearbyEntity {
            id: 2,
            position: Position::new(0.1, 0.0),
            kind: "agent".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Social(0.9)], vec![other]);
        let goal = SocializeGoal::default();
        goal.execute(&mut ctx).unwrap();
        let new_social = social_urgency(&ctx);
        assert!(
            new_social < 0.9,
            "social need should have decreased, got {new_social}"
        );
        assert!(ctx.relationships.get(&2).copied().unwrap_or(0.0) > 0.0);
    }

    #[test]
    fn socialize_does_not_interact_with_self() {
        let self_entity = NearbyEntity {
            id: 1,
            position: Position::new(0.1, 0.0),
            kind: "agent".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Social(0.8)], vec![self_entity]);
        let goal = SocializeGoal::default();
        let result = goal.execute(&mut ctx);
        assert!(result.is_err(), "should fail with no other agent");
    }

    // -- TradeGoal tests ---------------------------------------------------

    #[test]
    fn trade_completed_when_insufficient_surplus() {
        let mut ctx = make_ctx(vec![Need::Hunger(0.3)], vec![]);
        ctx.resources = Resources::from_pairs(&[("food", 1.0)]);
        let goal = TradeGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Completed);
    }

    #[test]
    fn trade_blocked_when_no_faction() {
        let mut ctx = make_ctx(vec![Need::Hunger(0.3)], vec![]);
        ctx.resources = Resources::from_pairs(&[("food", 5.0)]);
        let goal = TradeGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Blocked);
    }

    #[test]
    fn trade_active_when_surplus_and_faction_nearby() {
        let faction = NearbyEntity {
            id: 30,
            position: Position::new(2.0, 0.0),
            kind: "faction".into(),
            faction_id: Some(100),
        };
        let mut ctx = make_ctx(vec![Need::Hunger(0.3)], vec![faction]);
        ctx.resources = Resources::from_pairs(&[("food", 5.0)]);
        let goal = TradeGoal::default();
        assert_eq!(goal.evaluate(&ctx), GoalStatus::Active);
    }

    #[test]
    fn trade_moves_toward_faction() {
        let faction = NearbyEntity {
            id: 30,
            position: Position::new(5.0, 0.0),
            kind: "faction".into(),
            faction_id: Some(100),
        };
        let mut ctx = make_ctx(vec![Need::Hunger(0.3)], vec![faction]);
        ctx.resources = Resources::from_pairs(&[("food", 5.0)]);
        let goal = TradeGoal::default();
        goal.execute(&mut ctx).unwrap();
        assert!(ctx.position.x > 0.0);
    }

    #[test]
    fn trade_arrives_and_exchanges() {
        let faction = NearbyEntity {
            id: 30,
            position: Position::new(0.2, 0.0),
            kind: "faction".into(),
            faction_id: Some(100),
        };
        let mut ctx = make_ctx(vec![Need::Hunger(0.3), Need::Safety(0.8)], vec![faction]);
        ctx.resources = Resources::from_pairs(&[("food", 5.0)]);
        let goal = TradeGoal::default();
        goal.execute(&mut ctx).unwrap();
        assert!((ctx.resources.get("food") - 4.0).abs() < 1e-10);
        assert!((ctx.resources.get("building_materials") - 1.0).abs() < 1e-10);
        assert!(ctx.relationships.get(&100).copied().unwrap_or(0.0) > 0.0);
    }

    // -- GoalTree tests ----------------------------------------------------

    #[test]
    fn goal_tree_empty_by_default() {
        let tree = GoalTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn goal_tree_add_and_select() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SocializeGoal::default()));
        tree.add_goal(Box::new(SeekFoodGoal::default()));
        assert_eq!(tree.len(), 2);

        let other = NearbyEntity {
            id: 2,
            position: Position::new(3.0, 0.0),
            kind: "agent".into(),
            faction_id: None,
        };
        let food = NearbyEntity {
            id: 10,
            position: Position::new(4.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let ctx = make_ctx(
            vec![Need::Hunger(0.8), Need::Social(0.8)],
            vec![other, food],
        );

        let selected = tree
            .select_active(&ctx)
            .expect("should have an active goal");
        assert_eq!(selected.id(), "seek_food");
    }

    #[test]
    fn goal_tree_add_sub_goal() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));
        tree.add_sub_goal("seek_food", Box::new(SocializeGoal::default()))
            .unwrap();
        let subs = tree.sub_goals_of("seek_food");
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].id(), "socialize");
    }

    #[test]
    fn goal_tree_add_sub_goal_fails_without_parent() {
        let mut tree = GoalTree::new();
        let result = tree.add_sub_goal("nonexistent", Box::new(SocializeGoal::default()));
        assert!(result.is_err());
    }

    #[test]
    fn goal_tree_tick_completes_unblocked_goal() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));

        let mut ctx = make_ctx(vec![Need::Hunger(0.1)], vec![]);
        let result = tree.tick(&mut ctx);
        assert_eq!(result.as_deref(), Some("seek_food"));
        assert!(tree.is_empty(), "completed goal should be removed");
    }

    #[test]
    fn goal_tree_tick_promotes_sub_goals() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));
        tree.add_sub_goal("seek_food", Box::new(SocializeGoal::default()))
            .unwrap();

        let mut ctx = make_ctx(vec![Need::Hunger(0.1)], vec![]);
        tree.tick(&mut ctx);
        assert_eq!(tree.len(), 1);
        assert!(tree.get_goal("socialize").is_some());
    }

    #[test]
    fn goal_tree_tick_executes_active_goal() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));

        let food = NearbyEntity {
            id: 10,
            position: Position::new(5.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let mut ctx = make_ctx(vec![Need::Hunger(0.8)], vec![food]);
        tree.tick(&mut ctx);
        assert!(ctx.position.x > 0.0);
    }

    #[test]
    fn goal_tree_remove_goal() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));
        tree.add_goal(Box::new(SocializeGoal::default()));
        tree.remove_goal("seek_food");
        assert_eq!(tree.len(), 1);
        assert!(tree.get_goal("seek_food").is_none());
    }

    #[test]
    fn goal_tree_goal_ids() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));
        tree.add_goal(Box::new(SocializeGoal::default()));
        let mut ids = tree.goal_ids();
        ids.sort();
        assert_eq!(ids, vec!["seek_food", "socialize"]);
    }

    // -- Priority ordering tests -------------------------------------------

    #[test]
    fn seek_shelter_has_highest_priority() {
        assert!(SeekShelterGoal::default().priority() > SeekFoodGoal::default().priority());
    }

    #[test]
    fn seek_food_has_higher_priority_than_socialize() {
        assert!(SeekFoodGoal::default().priority() > SocializeGoal::default().priority());
    }

    #[test]
    fn socialize_has_higher_priority_than_trade() {
        assert!(SocializeGoal::default().priority() > TradeGoal::default().priority());
    }

    // -- Integration: mixed needs ------------------------------------------

    #[test]
    fn tree_selects_shelter_when_safety_critical() {
        let mut tree = GoalTree::new();
        tree.add_goal(Box::new(SeekFoodGoal::default()));
        tree.add_goal(Box::new(SeekShelterGoal::default()));
        tree.add_goal(Box::new(SocializeGoal::default()));

        let shelter = NearbyEntity {
            id: 20,
            position: Position::new(2.0, 0.0),
            kind: "shelter".into(),
            faction_id: None,
        };
        let food = NearbyEntity {
            id: 10,
            position: Position::new(4.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let ctx = make_ctx(
            vec![Need::Hunger(0.5), Need::Safety(0.9), Need::Social(0.5)],
            vec![shelter, food],
        );

        let selected = tree.select_active(&ctx).expect("should have active goal");
        assert_eq!(selected.id(), "seek_shelter");
    }

    #[test]
    fn ctx_need_urgency_returns_zero_for_absent_need() {
        let ctx = make_ctx(vec![Need::Hunger(0.5)], vec![]);
        assert!((ctx.need_urgency(Need::Safety(0.0)) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ctx_nearby_agent_count() {
        let other = NearbyEntity {
            id: 2,
            position: Position::new(1.0, 0.0),
            kind: "agent".into(),
            faction_id: None,
        };
        let food = NearbyEntity {
            id: 10,
            position: Position::new(2.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let ctx = make_ctx(vec![], vec![other, food]);
        assert_eq!(ctx.nearby_agent_count(), 1);
    }

    #[test]
    fn ctx_nearest_entity() {
        let far_food = NearbyEntity {
            id: 10,
            position: Position::new(10.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let near_food = NearbyEntity {
            id: 11,
            position: Position::new(2.0, 0.0),
            kind: "food_source".into(),
            faction_id: None,
        };
        let ctx = make_ctx(vec![], vec![far_food, near_food]);
        let nearest = ctx.nearest_entity("food_source").unwrap();
        assert_eq!(nearest.id, 11);
    }
}
