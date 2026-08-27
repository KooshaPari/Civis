//! Building Tiers System
//!
//! Provides a tiered building progression system for the Civis Bevy godgame.
//! Buildings start at Primitive tier and can be upgraded through six tiers
//! (Primitive → Basic → Advanced → Sophisticated → Monumental → Legendary)
//! provided the settlement meets technology and population requirements.

// ---------------------------------------------------------------------------
// BuildingTier
// ---------------------------------------------------------------------------

/// Six-tier building progression system.
///
/// Each variant carries an ordinal value representing its tier level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildingTier {
    Primitive = 0,
    Basic = 1,
    Advanced = 2,
    Sophisticated = 3,
    Monumental = 4,
    Legendary = 5,
}

impl BuildingTier {
    /// Returns the ordinal (0-based) of this tier.
    pub fn ordinal(&self) -> u32 {
        *self as u32
    }

    /// Try to convert a `u32` ordinal into a `BuildingTier`.
    pub fn from_ordinal(value: u32) -> Option<Self> {
        match value {
            0 => Some(BuildingTier::Primitive),
            1 => Some(BuildingTier::Basic),
            2 => Some(BuildingTier::Advanced),
            3 => Some(BuildingTier::Sophisticated),
            4 => Some(BuildingTier::Monumental),
            5 => Some(BuildingTier::Legendary),
            _ => None,
        }
    }

    /// The next higher tier, if one exists.
    pub fn next(&self) -> Option<Self> {
        Self::from_ordinal(self.ordinal() + 1)
    }

    /// The previous (lower) tier, if one exists.
    pub fn prev(&self) -> Option<Self> {
        self.ordinal().checked_sub(1).and_then(Self::from_ordinal)
    }

    /// All tiers in order, from Primitive to Legendary.
    pub fn all() -> &'static [BuildingTier] {
        &[
            BuildingTier::Primitive,
            BuildingTier::Basic,
            BuildingTier::Advanced,
            BuildingTier::Sophisticated,
            BuildingTier::Monumental,
            BuildingTier::Legendary,
        ]
    }
}

// ---------------------------------------------------------------------------
// BuildingTierConfig
// ---------------------------------------------------------------------------

/// Unlock requirements for a specific building tier.
///
/// A settlement must meet **all** four requirements before buildings of this
/// tier can be created or upgraded to.
#[derive(Debug, Clone)]
pub struct BuildingTierConfig {
    /// Minimum technology level the settlement must have reached.
    pub tech_level: u32,
    /// Minimum population required.
    pub population_min: u32,
    /// One-time resource cost (abstract units) to build / upgrade.
    pub resource_cost: u32,
    /// Per-tick maintenance cost multiplier applied to condition decay.
    pub maintenance_cost: f32,
}

impl BuildingTierConfig {
    /// Create a new config with the given values.
    pub fn new(tech_level: u32, population_min: u32, resource_cost: u32, maintenance_cost: f32) -> Self {
        Self {
            tech_level,
            population_min,
            resource_cost,
            maintenance_cost,
        }
    }
}

// ---------------------------------------------------------------------------
// Building
// ---------------------------------------------------------------------------

/// A single building placed in a settlement.
#[derive(Debug, Clone)]
pub struct Building {
    /// Unique building identifier.
    pub id: u32,
    /// Current tier of this building.
    pub tier: BuildingTier,
    /// The settlement this building belongs to.
    pub settlement_id: u32,
    /// Whether the building is currently operational.
    pub operational: bool,
    /// Age in ticks since construction / last upgrade.
    pub age: u32,
    /// Condition factor in the range `[0.0, 1.0]`. 1.0 = pristine.
    pub condition: f32,
}

impl Building {
    /// Create a new building at the given tier with pristine condition.
    pub fn new(id: u32, tier: BuildingTier, settlement_id: u32) -> Self {
        Self {
            id,
            tier,
            settlement_id,
            operational: true,
            age: 0,
            condition: 1.0,
        }
    }

    /// Create a new building with a custom starting condition.
    pub fn with_condition(id: u32, tier: BuildingTier, settlement_id: u32, condition: f32) -> Self {
        Self {
            id,
            tier,
            settlement_id,
            operational: true,
            age: 0,
            condition: condition.clamp(0.0, 1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// UpgradeError
// ---------------------------------------------------------------------------

/// Errors that can occur when attempting to upgrade a building.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeError {
    /// Settlement's technology level is too low.
    InsufficientTech,
    /// Settlement's population is too low.
    InsufficientPopulation,
    /// Insufficient resources to pay the upgrade cost.
    InsufficientResources,
    /// The building is already at the maximum tier (Legendary).
    AlreadyMaxTier,
    /// No building with the given id was found.
    BuildingNotFound,
}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeError::InsufficientTech => write!(f, "insufficient technology level"),
            UpgradeError::InsufficientPopulation => write!(f, "insufficient population"),
            UpgradeError::InsufficientResources => write!(f, "insufficient resources"),
            UpgradeError::AlreadyMaxTier => write!(f, "already at maximum tier"),
            UpgradeError::BuildingNotFound => write!(f, "building not found"),
        }
    }
}

impl std::error::Error for UpgradeError {}

// ---------------------------------------------------------------------------
// BuildingTierEngine
// ---------------------------------------------------------------------------

/// Engine that manages building tiers, upgrades, and per-tick simulation.
pub struct BuildingTierEngine {
    /// All active buildings.
    pub buildings: Vec<Building>,
    /// Tier configs, indexed by tier ordinal (index 0 = Primitive, … 5 = Legendary).
    pub configs: Vec<BuildingTierConfig>,
    /// Auto-incrementing id counter for new buildings.
    next_id: u32,
}

impl BuildingTierEngine {
    /// Create a new engine with the given default tier config.
    ///
    /// `default_config` is replicated for every tier, then scaled up by tier
    /// to give progressively harder requirements.
    pub fn new(default_config: BuildingTierConfig) -> Self {
        let configs: Vec<BuildingTierConfig> = BuildingTier::all()
            .iter()
            .enumerate()
            .map(|(idx, _tier)| BuildingTierConfig {
                tech_level: default_config.tech_level + (idx as u32) * 2,
                population_min: default_config.population_min + (idx as u32) * 10,
                resource_cost: default_config.resource_cost + (idx as u32) * 50,
                maintenance_cost: default_config.maintenance_cost + (idx as f32) * 0.05,
            })
            .collect();

        Self {
            buildings: Vec::new(),
            configs,
            next_id: 1,
        }
    }

    /// Create a new engine with explicit per-tier configs.
    pub fn with_configs(configs: Vec<BuildingTierConfig>) -> Self {
        assert_eq!(configs.len(), 6, "exactly 6 tier configs required");
        Self {
            buildings: Vec::new(),
            configs,
            next_id: 1,
        }
    }

    /// Spawn a new building in the given settlement at the specified tier.
    /// Returns the new building's id.
    pub fn spawn(&mut self, tier: BuildingTier, settlement_id: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.buildings.push(Building::new(id, tier, settlement_id));
        id
    }

    /// Attempt to upgrade the building with the given id to the next tier.
    ///
    /// Returns `Ok(())` on success, or an `UpgradeError` explaining why the
    /// upgrade cannot proceed.
    pub fn upgrade(
        &mut self,
        building_id: u32,
        current_tech: u32,
        current_population: u32,
        available_resources: &mut u32,
    ) -> Result<(), UpgradeError> {
        let idx = self
            .buildings
            .iter()
            .position(|b| b.id == building_id)
            .ok_or(UpgradeError::BuildingNotFound)?;

        let current_tier = self.buildings[idx].tier;
        let next_tier = current_tier.next().ok_or(UpgradeError::AlreadyMaxTier)?;

        let req = &self.configs[next_tier.ordinal() as usize];

        if current_tech < req.tech_level {
            return Err(UpgradeError::InsufficientTech);
        }
        if current_population < req.population_min {
            return Err(UpgradeError::InsufficientPopulation);
        }
        if *available_resources < req.resource_cost {
            return Err(UpgradeError::InsufficientResources);
        }

        *available_resources -= req.resource_cost;
        self.buildings[idx].tier = next_tier;
        self.buildings[idx].age = 0;
        self.buildings[idx].condition = 1.0;

        Ok(())
    }

    /// Downgrade a building by one tier. The building's age is reset to 0 and
    /// condition restored to 1.0.
    ///
    /// Returns `Ok(())` on success, or `BuildingNotFound` / `AlreadyMaxTier`
    /// (used for Primitive tier) errors.
    pub fn downgrade(&mut self, building_id: u32) -> Result<(), UpgradeError> {
        let idx = self
            .buildings
            .iter()
            .position(|b| b.id == building_id)
            .ok_or(UpgradeError::BuildingNotFound)?;

        let prev_tier = self.buildings[idx].tier.prev().ok_or(UpgradeError::AlreadyMaxTier)?;

        self.buildings[idx].tier = prev_tier;
        self.buildings[idx].age = 0;
        self.buildings[idx].condition = 1.0;

        Ok(())
    }

    /// Advance the simulation by `dt` ticks. Each building's age is incremented
    /// and its condition decays according to the formula:
    ///
    /// ```text
    /// condition -= maintenance_cost * dt
    /// ```
    ///
    /// Condition is clamped to `[0.0, 1.0]`. Buildings at 0.0 condition become
    /// non-operational.
    pub fn tick(&mut self, dt: f32) {
        for building in &mut self.buildings {
            if !building.operational {
                continue;
            }

            building.age += 1;

            let maintenance = self.configs[building.tier.ordinal() as usize].maintenance_cost;
            building.condition = (building.condition - maintenance * dt).clamp(0.0, 1.0);

            if building.condition <= 0.0 {
                building.operational = false;
            }
        }
    }

    /// Return the list of tiers that the given settlement is eligible to
    /// upgrade to, considering technology level and population.
    ///
    /// This checks each tier above `Primitive` to see if its requirements
    /// are met. Useful for UI display of available upgrades.
    pub fn available_upgrades(
        &self,
        settlement_id: u32,
        current_tech: u32,
        current_population: u32,
    ) -> Vec<BuildingTier> {
        BuildingTier::all()
            .iter()
            .filter(|tier| tier.ordinal() > 0)
            .filter(|tier| {
                let req = &self.configs[tier.ordinal() as usize];
                current_tech >= req.tech_level && current_population >= req.population_min
            })
            .copied()
            .collect()
    }

    /// Calculate the total capacity of all operational buildings in the given
    /// settlement. Capacity is defined as the sum of `(tier_ordinal + 1)` for
    /// each operational building.
    pub fn total_capacity(&self, settlement_id: u32) -> u32 {
        self.buildings
            .iter()
            .filter(|b| b.settlement_id == settlement_id && b.operational)
            .map(|b| b.tier.ordinal() + 1)
            .sum()
    }

    /// Find a building by its id.
    pub fn get_building(&self, building_id: u32) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == building_id)
    }

    /// Find a building by its id (mutable).
    pub fn get_building_mut(&mut self, building_id: u32) -> Option<&mut Building> {
        self.buildings.iter_mut().find(|b| b.id == building_id)
    }

    /// Remove all buildings in a settlement.
    pub fn clear_settlement(&mut self, settlement_id: u32) {
        self.buildings.retain(|b| b.settlement_id != settlement_id);
    }

    /// Get the config for a specific tier.
    pub fn config_for(&self, tier: BuildingTier) -> &BuildingTierConfig {
        &self.configs[tier.ordinal() as usize]
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create an engine with sensible defaults.
    fn test_engine() -> BuildingTierEngine {
        BuildingTierEngine::new(BuildingTierConfig::new(1, 10, 100, 0.01))
    }

    #[test]
    fn test_tier_ordinals() {
        assert_eq!(BuildingTier::Primitive.ordinal(), 0);
        assert_eq!(BuildingTier::Basic.ordinal(), 1);
        assert_eq!(BuildingTier::Advanced.ordinal(), 2);
        assert_eq!(BuildingTier::Sophisticated.ordinal(), 3);
        assert_eq!(BuildingTier::Monumental.ordinal(), 4);
        assert_eq!(BuildingTier::Legendary.ordinal(), 5);
    }

    #[test]
    fn test_tier_next_prev() {
        assert_eq!(BuildingTier::Primitive.next(), Some(BuildingTier::Basic));
        assert_eq!(BuildingTier::Legendary.next(), None);
        assert_eq!(BuildingTier::Basic.prev(), Some(BuildingTier::Primitive));
        assert_eq!(BuildingTier::Primitive.prev(), None);
    }

    #[test]
    fn test_tier_from_ordinal() {
        assert_eq!(BuildingTier::from_ordinal(0), Some(BuildingTier::Primitive));
        assert_eq!(BuildingTier::from_ordinal(5), Some(BuildingTier::Legendary));
        assert_eq!(BuildingTier::from_ordinal(6), None);
    }

    #[test]
    fn test_upgrade_progression() {
        let mut engine = test_engine();
        let id = engine.spawn(BuildingTier::Primitive, 1);
        let mut resources: u32 = 1000;

        // Upgrade Primitive -> Basic (needs tech >= 3, pop >= 20, cost >= 150)
        let result = engine.upgrade(id, 10, 50, &mut resources);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Basic);

        // Upgrade Basic -> Advanced
        let result = engine.upgrade(id, 10, 50, &mut resources);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Advanced);

        // Upgrade Advanced -> Sophisticated
        let result = engine.upgrade(id, 10, 50, &mut resources);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Sophisticated);

        // Upgrade Sophisticated -> Monumental
        let result = engine.upgrade(id, 10, 50, &mut resources);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Monumental);

        // Upgrade Monumental -> Legendary
        let result = engine.upgrade(id, 10, 50, &mut resources);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Legendary);

        // Already at max tier
        let result = engine.upgrade(id, 10, 50, &mut resources);
        assert_eq!(result, Err(UpgradeError::AlreadyMaxTier));
    }

    #[test]
    fn test_upgrade_insufficient_tech() {
        let mut engine = test_engine();
        let id = engine.spawn(BuildingTier::Primitive, 1);
        let mut resources: u32 = 1000;

        // Primitive -> Basic requires tech >= 3
        let result = engine.upgrade(id, 1, 100, &mut resources);
        assert_eq!(result, Err(UpgradeError::InsufficientTech));
    }

    #[test]
    fn test_upgrade_insufficient_population() {
        let mut engine = test_engine();
        let id = engine.spawn(BuildingTier::Primitive, 1);
        let mut resources: u32 = 1000;

        // Primitive -> Basic requires pop >= 20
        let result = engine.upgrade(id, 10, 5, &mut resources);
        assert_eq!(result, Err(UpgradeError::InsufficientPopulation));
    }

    #[test]
    fn test_upgrade_insufficient_resources() {
        let mut engine = test_engine();
        let id = engine.spawn(BuildingTier::Primitive, 1);
        let mut resources: u32 = 10;

        // Primitive -> Basic requires cost >= 150
        let result = engine.upgrade(id, 10, 100, &mut resources);
        assert_eq!(result, Err(UpgradeError::InsufficientResources));
    }

    #[test]
    fn test_downgrade() {
        let mut engine = test_engine();
        let id = engine.spawn(BuildingTier::Advanced, 1);

        let result = engine.downgrade(id);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Basic);

        let result = engine.downgrade(id);
        assert!(result.is_ok());
        assert_eq!(engine.get_building(id).unwrap().tier, BuildingTier::Primitive);

        // Cannot downgrade below Primitive
        let result = engine.downgrade(id);
        assert_eq!(result, Err(UpgradeError::AlreadyMaxTier));
    }

    #[test]
    fn test_condition_decay() {
        let mut engine = BuildingTierEngine::new(BuildingTierConfig::new(1, 10, 100, 0.1));
        let id = engine.spawn(BuildingTier::Primitive, 1);

        let building = engine.get_building(id).unwrap();
        assert!((building.condition - 1.0).abs() < f32::EPSILON);

        // Tick once: condition should decrease by 0.1 * dt
        engine.tick(1.0);
        let building = engine.get_building(id).unwrap();
        assert!((building.condition - 0.9).abs() < 0.001);

        // Tick several more times
        for _ in 0..9 {
            engine.tick(1.0);
        }
        let building = engine.get_building(id).unwrap();
        assert!((building.condition - 0.0).abs() < 0.001);
        assert!(!building.operational);
    }

    #[test]
    fn test_condition_clamped() {
        let mut engine = BuildingTierEngine::new(BuildingTierConfig::new(1, 10, 100, 10.0));
        let id = engine.spawn(BuildingTier::Primitive, 1);

        // Huge decay should clamp to 0.0, not go negative
        engine.tick(1.0);
        let building = engine.get_building(id).unwrap();
        assert!(building.condition >= 0.0);
        assert!(building.condition <= 1.0);
    }

    #[test]
    fn test_total_capacity() {
        let mut engine = test_engine();
        // Primitive.ordinal() + 1 = 1
        engine.spawn(BuildingTier::Primitive, 1);
        // Advanced.ordinal() + 1 = 3
        engine.spawn(BuildingTier::Advanced, 1);
        // Monumental.ordinal() + 1 = 5
        engine.spawn(BuildingTier::Monumental, 1);

        // Different settlement should not be counted
        engine.spawn(BuildingTier::Legendary, 2);

        let capacity = engine.total_capacity(1);
        assert_eq!(capacity, 1 + 3 + 5);
    }

    #[test]
    fn test_available_upgrades() {
        let engine = test_engine();
        // tech=10, pop=50 — should see all tiers whose config is met
        let upgrades = engine.available_upgrades(1, 10, 50);

        // With defaults (start at 1, increment by 2 tech & 10 pop per tier):
        // Primitive(0): tech>=1, pop>=10  — already at this tier, skipped
        // Basic(1):      tech>=3, pop>=20  — met
        // Advanced(2):   tech>=5, pop>=30  — met
        // Sophisticated(3): tech>=7, pop>=40 — met
        // Monumental(4): tech>=9, pop>=50  — met
        // Legendary(5):  tech>=11, pop>=60 — NOT met
        assert_eq!(upgrades.len(), 4);
        assert!(upgrades.contains(&BuildingTier::Basic));
        assert!(upgrades.contains(&BuildingTier::Advanced));
        assert!(upgrades.contains(&BuildingTier::Sophisticated));
        assert!(upgrades.contains(&BuildingTier::Monumental));
        assert!(!upgrades.contains(&BuildingTier::Legendary));
    }

    #[test]
    fn test_upgrade_resets_age_and_condition() {
        let mut engine = BuildingTierEngine::new(BuildingTierConfig::new(1, 10, 100, 0.1));
        let id = engine.spawn(BuildingTier::Primitive, 1);
        let mut resources: u32 = 1000;

        // Age the building
        engine.tick(1.0);
        engine.tick(1.0);
        let b = engine.get_building(id).unwrap();
        assert_eq!(b.age, 2);
        assert!(b.condition < 1.0);

        // Upgrade should reset age and condition
        engine.upgrade(id, 10, 50, &mut resources).unwrap();
        let b = engine.get_building(id).unwrap();
        assert_eq!(b.age, 0);
        assert!((b.condition - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_building_not_found() {
        let mut engine = test_engine();
        let mut resources: u32 = 1000;
        assert_eq!(
            engine.upgrade(9999, 10, 50, &mut resources),
            Err(UpgradeError::BuildingNotFound)
        );
        assert_eq!(
            engine.downgrade(9999),
            Err(UpgradeError::BuildingNotFound)
        );
    }
}
