//! CivLab Simulation Engine - Core Tick Loop with ECS
//!
//! This module provides the deterministic simulation loop with entity component system.

use hecs::World;
use rand::SeedableRng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Fixed;

/// Seeded RNG for reproducible simulation
pub type SimRng = ChaCha8Rng;

// ============================================================================
// COMPONENTS - Data attached to entities
// ============================================================================

/// Position on the hex grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Citizen entity component
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Citizen {
    pub age: u32,              // Age in years
    pub health: Fixed,          // Health 0.0 - 1.0
    pub ideology: Fixed,        // -1.0 (libertarian) to 1.0 (authoritarian)
    pub welfare: Fixed,        // 0.0 - 1.0
    pub job: Option<JobType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobType {
    Farmer,
    Warrior,
    Scholar,
    Trader,
    Priest,
    Admin,
    Unemployed,
}

/// Building entity component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub hp: Fixed,
    pub max_hp: Fixed,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Farm,
    Mine,
    Barracks,
    Temple,
    Market,
    House,
    CityCenter,
}

/// Resource storage component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Resources {
    pub food: Fixed,
    pub wood: Fixed,
    pub metal: Fixed,
    pub energy: Fixed,  // Joules
}

/// Production capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Production {
    pub output_type: ResourceType,
    pub rate: Fixed,  // Per tick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Wood,
    Metal,
    Energy,
}

/// Military unit component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitaryUnit {
    pub unit_type: UnitType,
    pub strength: Fixed,
    pub morale: Fixed,
    pub position: Position,
    pub faction_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Soldier,
    Archer,
    Knight,
    Scout,
}

// ============================================================================
// WORLD STATE
// ============================================================================

/// Global world state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub tick: u64,
    pub population: u64,
    pub energy_budget_joules: Fixed,
    pub rng_seed: u64,
    /// Faction ID -> faction name
    pub factions: HashMap<u32, String>,
    /// Faction ID -> treasury balance
    pub faction_treasury: HashMap<u32, Fixed>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 1_000_000,
            energy_budget_joules: Fixed::from_num(1_000_000_000_000i64),
            rng_seed: 42,
            factions: HashMap::from([
                (0, "Player".to_string()),
                (1, "AI Faction A".to_string()),
                (2, "AI Faction B".to_string()),
            ]),
            faction_treasury: HashMap::from([
                (0, Fixed::from_num(10_000)),
                (1, Fixed::from_num(8_000)),
                (2, Fixed::from_num(8_000)),
            ]),
        }
    }
}

/// Simulation engine combining state + ECS world
pub struct Simulation {
    pub state: WorldState,
    pub world: World,
    rng: SimRng,
}

impl Simulation {
    /// Create new simulation with default state
    pub fn new() -> Self {
        let rng = SimRng::seed_from_u64(42);
        let mut world = World::new();
        
        // Spawn initial entities
        Self::spawn_initial_entities(&mut world);
        
        Self {
            state: WorldState::default(),
            world,
            rng,
        }
    }
    
    /// Create simulation with custom seed
    pub fn with_seed(seed: u64) -> Self {
        let rng = SimRng::seed_from_u64(seed);
        let mut world = World::new();
        Self::spawn_initial_entities(&mut world);
        
        Self {
            state: WorldState {
                rng_seed: seed,
                ..Default::default()
            },
            world,
            rng,
        }
    }
    
    /// Spawn initial world entities
    fn spawn_initial_entities(world: &mut World) {
        // Create initial citizens
        for i in 0..100 {
            let citizen = Citizen {
                age: 20 + (i % 40),
                health: Fixed::from_num(1),
                ideology: Fixed::from_num((i as i64 % 20 - 10) as i32) / Fixed::from_num(10),
                welfare: Fixed::from_num(7) / Fixed::from_num(10),
                job: Some(JobType::Farmer),
            };
            let _ = world.spawn((citizen,));
        }
        
        // Create city center
        let city = Building {
            building_type: BuildingType::CityCenter,
            hp: Fixed::from_num(1000),
            max_hp: Fixed::from_num(1000),
            position: Position { x: 0, y: 0 },
        };
        let _ = world.spawn((city,));
        
        // Create farms
        for i in 0i32..5 {
            let farm = Building {
                building_type: BuildingType::Farm,
                hp: Fixed::from_num(200),
                max_hp: Fixed::from_num(200),
                position: Position { x: i - 2, y: 1 },
            };
            let _ = world.spawn((farm,));
        }
        
        // Create initial military
        for i in 0i32..10 {
            let soldier = MilitaryUnit {
                unit_type: UnitType::Soldier,
                strength: Fixed::from_num(10),
                morale: Fixed::from_num(1),
                position: Position { x: i, y: 0 },
                faction_id: 0,  // Player faction
            };
            let _ = world.spawn((soldier,));
        }
    }
    
    /// Get mutable reference to RNG
    pub fn rng_mut(&mut self) -> &mut SimRng {
        &mut self.rng
    }
    
    /// Advance simulation by one tick
    pub fn tick(&mut self) {
        self.state.tick += 1;
        
        // Run simulation phases
        self.phase_production();
        self.phase_citizen_lifecycle();
        self.phase_military();
        self.phase_economy();
    }
    
    /// Production phase - buildings produce resources
    fn phase_production(&mut self) {
        let mut production: HashMap<ResourceType, Fixed> = HashMap::new();
        production.insert(ResourceType::Food, Fixed::ZERO);
        production.insert(ResourceType::Wood, Fixed::ZERO);
        production.insert(ResourceType::Metal, Fixed::ZERO);
        
        // Collect production from buildings
        for (_, building) in self.world.query::<&Building>().iter() {
            match building.building_type {
                BuildingType::Farm => {
                    *production.get_mut(&ResourceType::Food).unwrap() += Fixed::from_num(10);
                }
                BuildingType::Mine => {
                    *production.get_mut(&ResourceType::Metal).unwrap() += Fixed::from_num(5);
                }
                _ => {}
            }
        }
        
        // Apply production to state (simplified - would go to resources in full impl)
        tracing::debug!("Tick {} production: food={:?}, metal={:?}", 
            self.state.tick,
            production.get(&ResourceType::Food),
            production.get(&ResourceType::Metal));
    }
    
    /// Citizen lifecycle phase
    fn phase_citizen_lifecycle(&mut self) {
        let mut births: u32 = 0;
        
        for (_, citizen) in self.world.query::<&mut Citizen>().iter() {
            // Age citizens
            citizen.age += 1;
            
            // Simple welfare decay/growth based on random
            let change = Fixed::from_num(self.rng.gen_range(-5..=5)) / Fixed::from_num(100);
            citizen.welfare = (citizen.welfare + change).clamp(Fixed::ZERO, Fixed::from_num(1));
        }
        
        // Births based on welfare
        if self.state.population > 0 && self.rng.gen_bool(0.001) {
            births = 1;
        }
        
        self.state.population += births as u64;
    }
    
    /// Military phase
    fn phase_military(&mut self) {
        for (_, unit) in self.world.query::<&mut MilitaryUnit>().iter() {
            // Morale recovery
            if unit.morale < Fixed::from_num(1) {
                unit.morale = (unit.morale + Fixed::from_num(1) / Fixed::from_num(100))
                    .min(Fixed::from_num(1));
            }
        }
    }
    
    /// Economy phase - energy consumption
    fn phase_economy(&mut self) {
        // Base energy consumption per citizen
        let consumption = Fixed::from_num(self.state.population) / Fixed::from_num(1000);
        self.state.energy_budget_joules = 
            (self.state.energy_budget_joules - consumption).max(Fixed::ZERO);
    }
    
    /// Get snapshot of current state
    pub fn snapshot(&self) -> SimulationSnapshot {
        let citizen_count = self.world.query::<&Citizen>().iter().count();
        let building_count = self.world.query::<&Building>().iter().count();
        let military_count = self.world.query::<&MilitaryUnit>().iter().count();
        
        SimulationSnapshot {
            tick: self.state.tick,
            population: self.state.population,
            citizen_count,
            building_count,
            military_count,
            energy_budget: self.state.energy_budget_joules,
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    pub tick: u64,
    pub population: u64,
    pub citizen_count: usize,
    pub building_count: usize,
    pub military_count: usize,
    pub energy_budget: Fixed,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new();
        assert_eq!(sim.state.tick, 0);
    }
    
    #[test]
    fn test_tick_advances() {
        let mut sim = Simulation::new();
        sim.tick();
        assert_eq!(sim.state.tick, 1);
    }
    
    #[test]
    fn test_initial_entities() {
        let sim = Simulation::new();
        let snapshot = sim.snapshot();
        assert!(snapshot.citizen_count > 0);
        assert!(snapshot.building_count > 0);
        assert!(snapshot.military_count > 0);
    }
    
    #[test]
    fn test_determinism() {
        let mut sim1 = Simulation::with_seed(12345);
        let mut sim2 = Simulation::with_seed(12345);
        
        for _ in 0..100 {
            sim1.tick();
            sim2.tick();
        }
        
        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(sim1.state.population, sim2.state.population);
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn position_creation() {
        let pos = Position { x: 5, y: 10 };
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn position_equality() {
        let pos1 = Position { x: 1, y: 2 };
        let pos2 = Position { x: 1, y: 2 };
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn position_clone() {
        let pos1 = Position { x: 3, y: 4 };
        let pos2 = pos1;
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn citizen_creation() {
        let cit = Citizen {
            age: 25,
            health: Fixed::from_num(1),
            ideology: Fixed::from_num(0),
            welfare: Fixed::from_num(1),
            job: Some(JobType::Farmer),
        };
        assert_eq!(cit.age, 25);
        assert_eq!(cit.job, Some(JobType::Farmer));
    }

    #[test]
    fn citizen_default_unemployed() {
        let cit = Citizen {
            age: 20,
            health: Fixed::from_num(1),
            ideology: Fixed::from_num(0),
            welfare: Fixed::from_num(1),
            job: None,
        };
        assert_eq!(cit.job, None);
    }

    #[test]
    fn building_creation() {
        let bld = Building {
            building_type: BuildingType::Farm,
            hp: Fixed::from_num(200),
            max_hp: Fixed::from_num(200),
            position: Position { x: 0, y: 0 },
        };
        assert_eq!(bld.building_type, BuildingType::Farm);
        assert_eq!(bld.hp, bld.max_hp);
    }

    #[test]
    fn building_damaged() {
        let bld = Building {
            building_type: BuildingType::Farm,
            hp: Fixed::from_num(100),
            max_hp: Fixed::from_num(200),
            position: Position { x: 0, y: 0 },
        };
        assert!(bld.hp < bld.max_hp);
    }

    #[test]
    fn military_unit_creation() {
        let unit = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 0, y: 0 },
            faction_id: 0,
        };
        assert_eq!(unit.unit_type, UnitType::Soldier);
        assert_eq!(unit.faction_id, 0);
    }

    #[test]
    fn resources_default_zero() {
        let res = Resources::default();
        assert_eq!(res.food, Fixed::ZERO);
        assert_eq!(res.wood, Fixed::ZERO);
        assert_eq!(res.metal, Fixed::ZERO);
        assert_eq!(res.energy, Fixed::ZERO);
    }

    #[test]
    fn production_creation() {
        let prod = Production {
            output_type: ResourceType::Food,
            rate: Fixed::from_num(10),
        };
        assert_eq!(prod.output_type, ResourceType::Food);
    }
}

#[cfg(test)]
mod simulation_lifecycle_tests {
    use super::*;

    #[test]
    fn simulation_new_initializes() {
        let sim = Simulation::new();
        assert_eq!(sim.state.tick, 0);
        assert!(sim.state.population > 0);
    }

    #[test]
    fn simulation_with_seed_uses_seed() {
        let sim = Simulation::with_seed(999);
        assert_eq!(sim.state.rng_seed, 999);
    }

    #[test]
    fn simulation_tick_increments() {
        let mut sim = Simulation::new();
        let initial_tick = sim.state.tick;
        sim.tick();
        assert_eq!(sim.state.tick, initial_tick + 1);
    }

    #[test]
    fn simulation_multiple_ticks() {
        let mut sim = Simulation::new();
        for _ in 0..5 {
            sim.tick();
        }
        assert_eq!(sim.state.tick, 5);
    }

    #[test]
    fn simulation_energy_decreases_with_tick() {
        let mut sim = Simulation::new();
        let initial_energy = sim.state.energy_budget_joules;
        sim.tick();
        assert!(sim.state.energy_budget_joules <= initial_energy);
    }

    #[test]
    fn simulation_population_tracked() {
        let sim = Simulation::new();
        assert!(sim.state.population > 0);
    }

    #[test]
    fn simulation_snapshot_captures_state() {
        let sim = Simulation::new();
        let snap = sim.snapshot();
        assert_eq!(snap.tick, sim.state.tick);
        assert_eq!(snap.population, sim.state.population);
    }

    #[test]
    fn simulation_snapshot_counts_entities() {
        let sim = Simulation::new();
        let snap = sim.snapshot();
        assert!(snap.citizen_count > 0);
        assert!(snap.building_count > 0);
        assert!(snap.military_count > 0);
    }

    #[test]
    fn simulation_deterministic_with_same_seed() {
        let mut sim1 = Simulation::with_seed(777);
        let mut sim2 = Simulation::with_seed(777);

        for _ in 0..10 {
            sim1.tick();
            sim2.tick();
        }

        assert_eq!(sim1.state.tick, sim2.state.tick);
    }

    #[test]
    fn simulation_rng_mut_returns_mutable() {
        let mut sim = Simulation::new();
        let _rng = sim.rng_mut();
        // Test that we can get a mutable reference
    }

    #[test]
    fn simulation_default_equals_new() {
        let sim1 = Simulation::new();
        let sim2 = Simulation::default();
        assert_eq!(sim1.state.tick, sim2.state.tick);
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn snapshot_is_cloneable() {
        let snap1 = SimulationSnapshot {
            tick: 10,
            population: 1000,
            citizen_count: 100,
            building_count: 20,
            military_count: 50,
            energy_budget: Fixed::from_num(5000),
        };
        let snap2 = snap1.clone();
        assert_eq!(snap1.tick, snap2.tick);
        assert_eq!(snap1.population, snap2.population);
    }
}
