//! CivLab Deterministic Simulation Engine
//!
//! Uses fixed-point arithmetic for deterministic simulation results.
//! Uses i64 with scaling for deterministic calculations.
//!
//! ## Modules
//!
//! - `engine` - Full ECS-based simulation with tick loop
//! - `step` - Simple step function for basic simulation
//! - `policy` - Policy/consumption calculations
//! - `metrics` - Tyranny/legitimacy metrics
//! - `io` - File I/O utilities

pub mod engine;
pub mod io;
pub mod metrics;
pub mod policy;
pub mod replay;

pub use engine::{
    Building, BuildingType, Citizen, JobType, MilitaryUnit, Position, Production, ResourceType,
    Resources, Simulation, SimulationSnapshot, UnitType, WorldState,
};

pub use civ_planet::{Climate, MoonConfig, PlanetConfig};
pub use civ_tactics::{apply_damage, DamageEvent};
pub use metrics::{compute, Metrics};
pub use policy::{effective_consumption, PolicyInput};
pub use replay::{ReplayError, ReplayEvent, ReplayLog};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Fixed-point type: i64 with 18 decimal places of precision
/// Stored as raw i64, divided by 10^18 for actual value
/// This ensures deterministic simulation across platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Fixed {
    /// Raw value scaled by 10^18
    pub raw: i64,
}

pub const SCALE: i64 = 1_000_000; // 10^6 (easier to work with)

impl Fixed {
    pub const ZERO: Fixed = Fixed { raw: 0 };
    pub const ONE: Fixed = Fixed { raw: SCALE };

    pub fn from_num<T: TryInto<i128>>(n: T) -> Self {
        let scaled = n.try_into().unwrap_or(0) * SCALE as i128;
        Fixed { raw: scaled as i64 }
    }

    pub fn from_raw(raw: i64) -> Self {
        Fixed { raw }
    }

    pub fn to_f64(self) -> f64 {
        self.raw as f64 / SCALE as f64
    }

    pub fn saturating_add(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw.saturating_add(other.raw),
        }
    }

    pub fn saturating_sub(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw.saturating_sub(other.raw),
        }
    }

    pub fn clamp(self, min: Fixed, max: Fixed) -> Fixed {
        Fixed {
            raw: self.raw.clamp(min.raw, max.raw),
        }
    }
}

impl std::ops::Add for Fixed {
    type Output = Fixed;
    fn add(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw + other.raw,
        }
    }
}

impl std::ops::Sub for Fixed {
    type Output = Fixed;
    fn sub(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw - other.raw,
        }
    }
}

impl std::ops::Mul for Fixed {
    type Output = Fixed;
    fn mul(self, other: Fixed) -> Fixed {
        // Multiply and divide by scale to maintain precision
        let result = (self.raw as i128) * (other.raw as i128) / SCALE as i128;
        Fixed { raw: result as i64 }
    }
}

impl std::ops::Div for Fixed {
    type Output = Fixed;
    fn div(self, other: Fixed) -> Fixed {
        let result = (self.raw as i128 * SCALE as i128) / (other.raw.max(1) as i128);
        Fixed { raw: result as i64 }
    }
}

impl std::ops::AddAssign for Fixed {
    fn add_assign(&mut self, other: Fixed) {
        self.raw += other.raw;
    }
}

impl std::ops::SubAssign for Fixed {
    fn sub_assign(&mut self, other: Fixed) {
        self.raw -= other.raw;
    }
}

impl serde::Serialize for Fixed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(self.to_f64())
    }
}

impl<'de> serde::Deserialize<'de> for Fixed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let f = f64::deserialize(deserializer)?;
        Ok(Fixed::from_num((f * SCALE as f64) as i64))
    }
}

/// Seeded RNG for deterministic simulation
pub type SimRng = ChaCha8Rng;

/// Create a seeded RNG from world state
pub fn create_rng(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}

/// Advance simulation by one tick (simple API)
pub fn step(mut state: WorldState, consumption_joules: Fixed) -> WorldState {
    state.tick += 1;
    let result = state
        .energy_budget_joules
        .saturating_sub(consumption_joules);
    state.energy_budget_joules = if result.raw < 0 { Fixed::ZERO } else { result };
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn step_advances_tick() {
        let s = WorldState::default();
        let n = step(s, Fixed::from_num(100));
        assert_eq!(n.tick, 1);
    }

    #[test]
    fn step_decreases_energy() {
        let s = WorldState::default();
        // Initial energy is 1_000_000_000_000, subtract 1000 = 999_999_999_000
        let expected = Fixed::from_num(1_000_000_000_000i64) - Fixed::from_num(1000i64);
        let n = step(s, Fixed::from_num(1000));
        assert_eq!(n.energy_budget_joules, expected);
    }

    #[test]
    fn step_energy_floor_at_zero() {
        let s = WorldState {
            energy_budget_joules: Fixed::from_num(50),
            ..WorldState::default()
        };
        let n = step(s, Fixed::from_num(100));
        assert_eq!(n.energy_budget_joules, Fixed::ZERO);
    }

    #[test]
    fn determinism_same_seed_same_output() {
        let s1 = WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: Fixed::from_num(1000),
            rng_seed: 12345,
            factions: HashMap::new(),
            faction_treasury: HashMap::new(),
        };
        let s2 = WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: Fixed::from_num(1000),
            rng_seed: 12345,
            factions: HashMap::new(),
            faction_treasury: HashMap::new(),
        };

        let r1 = step(s1, Fixed::from_num(10));
        let r2 = step(s2, Fixed::from_num(10));

        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }
}
