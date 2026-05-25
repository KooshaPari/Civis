//! Read-only spectator view for dashboards and protocol clients (FR-CIV-WEB-003, P-U1).
//!
//! Deterministic pin positions match `civ-watch` so web and Godot render the same layout
//! when attached to the same simulation seed/tick.

use serde::{Deserialize, Serialize};

use crate::engine::{BuildingType, Citizen, JobType, Simulation};

/// Normalised map pin for one civilian (0..1 plane coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CivPin {
    pub idx: u32,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub job: Option<JobLabel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobLabel {
    Farmer,
    Warrior,
    Scholar,
    Trader,
    Priest,
    Admin,
    Unemployed,
}

impl From<JobType> for JobLabel {
    fn from(value: JobType) -> Self {
        match value {
            JobType::Farmer => Self::Farmer,
            JobType::Warrior => Self::Warrior,
            JobType::Scholar => Self::Scholar,
            JobType::Trader => Self::Trader,
            JobType::Priest => Self::Priest,
            JobType::Admin => Self::Admin,
            JobType::Unemployed => Self::Unemployed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Faction {
    pub id: u32,
    pub color: [u8; 3],
    pub capital: [f32; 2],
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BuildingKind {
    Residential,
    Commercial,
    Industrial,
    Civic,
}

impl From<BuildingType> for BuildingKind {
    fn from(value: BuildingType) -> Self {
        match value {
            BuildingType::House | BuildingType::Farm => Self::Residential,
            BuildingType::Market | BuildingType::CityCenter => Self::Commercial,
            BuildingType::Mine | BuildingType::Barracks => Self::Industrial,
            BuildingType::Temple => Self::Civic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BuildingPin {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub kind: BuildingKind,
    pub era: u16,
    pub faction_id: u32,
}

/// Full spectator payload for JSON-RPC / SSE clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectatorView {
    pub civ_pins: Vec<CivPin>,
    pub factions: Vec<Faction>,
    pub buildings: Vec<BuildingPin>,
    pub is_day: bool,
}

impl Simulation {
    /// Build deterministic spectator pins (matches `civ-watch` layout).
    pub fn spectator_view(&self) -> SpectatorView {
        let tick = self.state.tick;
        let factions = factions_for_tick(tick);
        SpectatorView {
            civ_pins: civ_pins(self),
            factions: factions.clone(),
            buildings: buildings_for_factions(&factions, tick, self),
            is_day: {
                let phase = self.climate().day_phase;
                (0.25..0.75).contains(&phase)
            },
        }
    }
}

fn wrap01(v: f32) -> f32 {
    v - v.floor()
}

fn civ_pins(sim: &Simulation) -> Vec<CivPin> {
    sim.world
        .query::<&Citizen>()
        .iter()
        .take(256)
        .enumerate()
        .map(|(idx, (_, citizen))| {
            let seed = u64::from(idx as u32).wrapping_mul(2_654_435_761) ^ u64::from(citizen.age);
            let base_x = ((seed & 0xffff) as f32) / 65535.0;
            let base_y = (((seed >> 16) & 0xffff) as f32) / 65535.0;
            let angle = ((seed >> 32) as f32 / u32::MAX as f32) * std::f32::consts::TAU;
            let drift = 0.0015 + ((seed >> 48) as f32 / 65535.0) * 0.0025;
            let dx = angle.cos() * drift;
            let dy = angle.sin() * drift;
            let tick_phase = (sim.state.tick as f32) * 0.1;
            CivPin {
                idx: idx as u32,
                x: wrap01(base_x + dx * tick_phase),
                y: wrap01(base_y + dy * tick_phase),
                dx,
                dy,
                job: citizen.job.map(JobLabel::from),
            }
        })
        .collect()
}

fn factions_for_tick(tick: u64) -> Vec<Faction> {
    let territory_radius_t = 18.0 + (tick as f32) * 0.018;
    let capitals = [
        (0.22, 0.24, [214, 174, 110]),
        (0.76, 0.27, [112, 176, 122]),
        (0.27, 0.73, [103, 151, 214]),
        (0.72, 0.74, [184, 118, 196]),
    ];
    capitals
        .iter()
        .enumerate()
        .map(|(idx, (x, y, color))| Faction {
            id: idx as u32,
            color: *color,
            capital: [*x, *y],
            radius: territory_radius_t + idx as f32 * 2.75,
        })
        .collect()
}

fn buildings_for_factions(factions: &[Faction], tick: u64, sim: &Simulation) -> Vec<BuildingPin> {
    let kinds = [
        BuildingKind::Residential,
        BuildingKind::Commercial,
        BuildingKind::Industrial,
        BuildingKind::Civic,
    ];
    let mut pins = Vec::new();
    for faction in factions {
        for i in 0..3 {
            let idx = faction.id * 3 + i;
            let angle = (idx as f32) * 1.7 + (tick as f32) * 0.02;
            let dist = faction.radius * 0.35 + (i as f32) * 2.5;
            pins.push(BuildingPin {
                id: idx,
                x: wrap01(faction.capital[0] + angle.cos() * dist / 128.0),
                y: wrap01(faction.capital[1] + angle.sin() * dist / 128.0),
                kind: kinds[i as usize % kinds.len()],
                era: ((tick / 120) % 6) as u16,
                faction_id: faction.id,
            });
        }
    }
    for (idx, (_, building)) in sim.world.query::<&crate::Building>().iter().enumerate() {
        if building.building_type == crate::BuildingType::CityCenter {
            let (x, y) = crate::grid_to_norm(building.position);
            pins.push(BuildingPin {
                id: 9_000 + idx as u32,
                x,
                y,
                kind: BuildingKind::Civic,
                era: ((tick / 120) % 6) as u16,
                faction_id: 0,
            });
        }
    }
    pins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectator_view_has_pins_after_startup() {
        let sim = Simulation::with_seed(1);
        let view = sim.spectator_view();
        assert!(!view.civ_pins.is_empty());
        assert!(!view.factions.is_empty());
        assert!(!view.buildings.is_empty());
    }
}
