use std::collections::HashMap;

/// Unique identifier for an agent in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

/// Snapshot of the simulation world at a given tick.
#[derive(Debug, Clone)]
pub struct WorldState {
    pub tick: u64,
    pub self_joules: i64,
    pub neighbors: Vec<AgentId>,
    pub market_price_cents: HashMap<String, i64>,
}

/// Action an agent can take on a tick.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Work,
    Trade { good: String, quantity: i64 },
    Rest,
    Move { x: i32, y: i32 },
}

/// A strategy that decides what an agent does given the current world state.
pub trait DecisionPolicy: Send + Sync {
    fn decide(&self, world: &WorldState) -> Action;
    fn name(&self) -> &'static str;
}

/// Policy that returns a constant Rest action (placeholder / bottom-line).
#[derive(Debug, Clone)]
pub struct RandomPolicy {
    pub seed: u64,
}

impl DecisionPolicy for RandomPolicy {
    fn decide(&self, _world: &WorldState) -> Action {
        Action::Rest
    }

    fn name(&self) -> &'static str {
        "random"
    }
}

/// Greedy policy that picks the good with the best market price.
#[derive(Debug, Clone)]
pub struct GreedyPolicy;

impl DecisionPolicy for GreedyPolicy {
    fn decide(&self, world: &WorldState) -> Action {
        let best = world
            .market_price_cents
            .iter()
            .max_by_key(|(_, &price)| price)
            .map(|(good, _)| good.clone());

        match best {
            Some(good) => Action::Trade {
                good,
                quantity: 1,
            },
            None => Action::Rest,
        }
    }

    fn name(&self) -> &'static str {
        "greedy"
    }
}

/// Lookup table of named decision policies.
pub struct PolicyRegistry {
    policies: HashMap<String, Box<dyn DecisionPolicy>>,
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    pub fn register(&mut self, policy: Box<dyn DecisionPolicy>) {
        let name = policy.name().to_string();
        self.policies.insert(name, policy);
    }

    pub fn decide(&self, name: &str, world: &WorldState) -> Option<Action> {
        self.policies.get(name).map(|p| p.decide(world))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn random_policy_returns_action() {
        let policy = RandomPolicy { seed: 42 };
        let world = WorldState {
            tick: 0,
            self_joules: 100,
            neighbors: vec![],
            market_price_cents: HashMap::new(),
        };
        let action = policy.decide(&world);
        // RandomPolicy always returns Rest.
        assert_eq!(action, Action::Rest);
        assert_eq!(policy.name(), "random");
    }

    #[test]
    fn greedy_picks_max_price() {
        let policy = GreedyPolicy;
        let mut prices = HashMap::new();
        prices.insert("wheat".to_string(), 10);
        prices.insert("iron".to_string(), 50);
        prices.insert("wood".to_string(), 25);
        let world = WorldState {
            tick: 1,
            self_joules: 200,
            neighbors: vec![],
            market_price_cents: prices,
        };
        let action = policy.decide(&world);
        assert_eq!(
            action,
            Action::Trade {
                good: "iron".to_string(),
                quantity: 1
            }
        );
        assert_eq!(policy.name(), "greedy");
    }

    #[test]
    fn greedy_empty_market_returns_rest() {
        let policy = GreedyPolicy;
        let world = WorldState {
            tick: 1,
            self_joules: 200,
            neighbors: vec![],
            market_price_cents: HashMap::new(),
        };
        assert_eq!(policy.decide(&world), Action::Rest);
    }

    #[test]
    fn registry_lookup() {
        let mut registry = PolicyRegistry::new();
        registry.register(Box::new(RandomPolicy { seed: 0 }));
        registry.register(Box::new(GreedyPolicy));

        let mut prices = HashMap::new();
        prices.insert("oil".to_string(), 99);
        let world = WorldState {
            tick: 0,
            self_joules: 100,
            neighbors: vec![],
            market_price_cents: prices,
        };

        let action = registry.decide("greedy", &world);
        assert_eq!(
            action,
            Some(Action::Trade {
                good: "oil".to_string(),
                quantity: 1
            })
        );

        let action = registry.decide("random", &world);
        assert_eq!(action, Some(Action::Rest));
    }

    #[test]
    fn registry_unknown_returns_none() {
        let registry = PolicyRegistry::new();
        let world = WorldState {
            tick: 0,
            self_joules: 100,
            neighbors: vec![],
            market_price_cents: HashMap::new(),
        };
        assert_eq!(registry.decide("nonexistent", &world), None);
    }
}
