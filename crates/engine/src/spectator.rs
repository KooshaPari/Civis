//! Read-only spectator view for dashboards and protocol clients (FR-CIV-WEB-003, P-U1).
//!
//! Deterministic pin positions match `civ-watch` so web and Godot render the same layout
//! when attached to the same simulation seed/tick.

use serde::{Deserialize, Serialize};

use civ_agents::{Civilian, Position3d, Velocity};
use civ_voxel::FIXED_SCALE;

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

/// Pins from agent ECS positions (matches civ-watch); includes `spawn_civilian_at` spawns.
fn civ_pins(sim: &Simulation) -> Vec<CivPin> {
    let mut pins: Vec<CivPin> = sim
        .world
        .query::<(&Civilian, &Position3d, &Velocity)>()
        .iter()
        .map(|(entity, (civilian, pos, vel))| {
            let x = (pos.coord.x as f32 / FIXED_SCALE as f32).clamp(0.0, 1.0);
            let y = (pos.coord.z as f32 / FIXED_SCALE as f32).clamp(0.0, 1.0);
            let job = sim
                .world
                .get::<&Citizen>(entity)
                .ok()
                .and_then(|citizen| citizen.job)
                .map(JobLabel::from);
            CivPin {
                idx: civilian.id as u32,
                x,
                y,
                dx: vel.dx,
                dy: vel.dy,
                job,
            }
        })
        .collect();
    pins.sort_by_key(|p| p.idx);
    pins.truncate(256);
    pins
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
        let (x, y) = crate::grid_to_norm(building.position);
        match building.building_type {
            crate::BuildingType::CityCenter => pins.push(BuildingPin {
                id: 9_000 + idx as u32,
                x,
                y,
                kind: BuildingKind::Civic,
                era: ((tick / 120) % 6) as u16,
                faction_id: 0,
            }),
            crate::BuildingType::Market => pins.push(BuildingPin {
                id: 9_100 + idx as u32,
                x,
                y,
                kind: BuildingKind::Commercial,
                era: ((tick / 120) % 6) as u16,
                faction_id: 0,
            }),
            crate::BuildingType::Barracks => pins.push(BuildingPin {
                id: 9_200 + idx as u32,
                x,
                y,
                kind: BuildingKind::Industrial,
                era: ((tick / 120) % 6) as u16,
                faction_id: 0,
            }),
            _ => {}
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

    #[test]
    fn civ_pins_include_job_when_citizen_component_present() {
        let sim = Simulation::with_seed(1);
        let view = sim.spectator_view();
        assert!(
            view.civ_pins.iter().any(|p| p.job.is_some()),
            "expected pins with job from Citizen on agent entities, got {:?}",
            view.civ_pins
        );
        assert!(
            view.civ_pins
                .iter()
                .any(|p| p.idx == 10_003 && p.job == Some(JobLabel::Farmer)),
            "civilian id 10003 maps to Farmer via job_type_for_civilian_id"
        );
    }

    #[test]
    fn civ_pins_reflect_spawned_agent_coordinates() {
        use civ_agents::spawn_civilian_at;

        let mut sim = Simulation::with_seed(9);
        let mut rng = sim.rng_mut().clone();
        let _ = spawn_civilian_at(&mut sim.world, 42_007, 1, 0.4, 0.6, &mut rng);
        *sim.rng_mut() = rng;
        crate::engine::attach_citizen_to_agents(&mut sim.world);
        let pins = civ_pins(&sim);
        assert_eq!(
            pins.iter().find(|p| p.idx == 42_007).and_then(|p| p.job),
            Some(JobLabel::Farmer),
            "42_007 % 7 == 0 → Farmer"
        );
        assert!(
            pins.iter()
                .any(|p| (p.x - 0.4).abs() < 0.02 && (p.y - 0.6).abs() < 0.02),
            "expected spawn at norm (0.4, 0.6), got {pins:?}"
        );
    }
}
