//! Background simulation worker (~10 Hz).

use std::sync::atomic::Ordering;
use std::time::Duration;

use civ_agents::{spawn_civilian_at, tick_movement};
use civ_engine::{DiplomacyKind, Simulation};
use civ_tactics::DamageEvent;
use civ_voxel::{MaterialId, WorldCoord};

use crate::app::{AppState, DamagePulse, MilitaryPin, TradeTickSummary};
use crate::snapshot::{
    apply_trade_routes, assign_and_drift_housing, buildings, factions, make_snapshot, noise_offset,
};
use crate::terrain::Terrain;

pub(crate) async fn simulation_worker(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        let speed = state.speed.load(Ordering::Relaxed);
        if speed == 0 {
            continue;
        }
        let snapshot = {
            let mut sim = state.sim.lock().await;
            let mut military = state.military.lock().await;
            let mut damage_events = Vec::new();
            let mut trade = TradeTickSummary::default();
            for _ in 0..speed {
                sim.tick();
                if sim.state.tick > 0 && sim.state.tick % 600 == 0 {
                    state
                        .target_era
                        .store(((sim.state.tick / 600).min(5)) as u16, Ordering::Relaxed);
                }
                let terrain = state.terrain.clone();
                let factions = factions(sim.state.tick);
                let buildings = buildings(&factions, sim.state.tick);
                assign_and_drift_housing(&mut sim, &buildings);
                let mut rng = sim.rng_mut().clone();
                tick_movement(&mut sim.world, 128, &mut rng, |x, y| {
                    terrain.is_walkable(x, y)
                });
                *sim.rng_mut() = rng;
                damage_events = tick_military(&mut sim, &terrain, &mut military);
                let tick = sim.state.tick;
                let (trade_volume, trade_balances) = apply_trade_routes(&mut sim, &factions, tick);
                trade.volume += trade_volume;
                for (faction_id, balance) in trade_balances {
                    *trade.balances.entry(faction_id).or_insert(0.0) += balance;
                }
                for event in &damage_events {
                    sim.push_damage(DamageEvent {
                        center: WorldCoord {
                            x: (event.x * civ_voxel::FIXED_SCALE as f32) as i64,
                            y: 0,
                            z: (event.y * civ_voxel::FIXED_SCALE as f32) as i64,
                        },
                        radius_voxels: 1,
                        energy: 8,
                    });
                }
            }
            let current_era = state.target_era.load(Ordering::Relaxed);
            make_snapshot(
                &sim,
                &military,
                &damage_events,
                &trade,
                speed,
                &state.laws,
                current_era,
            )
        };
        *state.latest.write().await = Some(snapshot.clone());
        let _ = state.tx.send(snapshot);
    }
}

pub(crate) fn seed_voxels(sim: &mut Simulation) {
    // Seed a tiny block of voxels so the chunk store is non-empty before any
    // user interaction. Eventually the procedural terrain will be written into
    // the sim's voxel store too.
    for x in 0..8 {
        sim.voxel_mut().write(
            WorldCoord {
                x: i64::from(x) * 1_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
    }
}

pub(crate) fn seed_civilians(sim: &mut Simulation, terrain: &Terrain) {
    let mut spawned = 0_u64;
    let mut x = 0.11_f32;
    let mut y = 0.19_f32;
    while spawned < 32 {
        if terrain.is_walkable(x, y) {
            let id = 10_000 + spawned;
            let mut rng = sim.rng_mut().clone();
            let _ = spawn_civilian_at(
                &mut sim.world,
                id,
                civ_agents::Alignment::Faction((spawned % 4) as u32),
                x,
                y,
                civ_agents::ActorVisualKind::Humanoid,
                &mut rng,
            );
            *sim.rng_mut() = rng;
            spawned += 1;
        }
        x = (x + 0.071).fract();
        y = (y + 0.113).fract();
    }
}

pub(crate) fn seed_military(sim: &mut Simulation, terrain: &Terrain, units: &mut Vec<MilitaryPin>) {
    let factions = factions(sim.state.tick);
    let mut next_id = 1_000_000_000_u64;
    for faction in factions {
        for _ in 0..5 {
            let seed = next_id ^ (u64::from(faction.id) << 32);
            units.push(MilitaryPin {
                id: next_id,
                x: (faction.capital[0] + noise_offset(seed, 0)).clamp(0.01, 0.99),
                y: (faction.capital[1] + noise_offset(seed, 1)).clamp(0.01, 0.99),
                unit_type: "Soldier".to_string(),
                faction: faction.id,
                strength: 1.0,
            });
            next_id += 1;
        }
    }
    let _ = terrain;
}

pub(crate) fn tick_military(
    sim: &mut Simulation,
    _terrain: &Terrain,
    units: &mut [MilitaryPin],
) -> Vec<DamagePulse> {
    let factions = factions(sim.state.tick);
    let conflict_factions: Vec<u32> = sim
        .diplomacy_events()
        .iter()
        .filter(|event| matches!(event.kind, DiplomacyKind::Conflict))
        .flat_map(|event| [event.faction_a, event.faction_b])
        .collect();
    if conflict_factions.is_empty() {
        return Vec::new();
    }

    let mut damage_events = Vec::new();
    for unit in units.iter_mut() {
        if !conflict_factions.contains(&unit.faction) {
            continue;
        }
        if let Some(target) = factions
            .iter()
            .filter(|faction| faction.id != unit.faction && conflict_factions.contains(&faction.id))
            .min_by(|a, b| {
                let da = (unit.x - a.capital[0]).powi(2) + (unit.y - a.capital[1]).powi(2);
                let db = (unit.x - b.capital[0]).powi(2) + (unit.y - b.capital[1]).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            let dx = target.capital[0] - unit.x;
            let dy = target.capital[1] - unit.y;
            let dist = (dx * dx + dy * dy).sqrt().max(0.0001);
            let seed = unit.id ^ (u64::from(unit.faction) << 32) ^ sim.state.tick;
            unit.x = (unit.x + dx / dist * 0.01 + noise_offset(seed, 0) * 0.5).clamp(0.0, 1.0);
            unit.y = (unit.y + dy / dist * 0.01 + noise_offset(seed, 1) * 0.5).clamp(0.0, 1.0);
        }
    }

    for i in 0..units.len() {
        for j in (i + 1)..units.len() {
            if units[i].faction == units[j].faction {
                continue;
            }
            if !conflict_factions.contains(&units[i].faction)
                || !conflict_factions.contains(&units[j].faction)
            {
                continue;
            }
            let dx = units[i].x - units[j].x;
            let dy = units[i].y - units[j].y;
            if dx * dx + dy * dy <= 0.05 * 0.05 {
                damage_events.push(DamagePulse {
                    x: (units[i].x + units[j].x) * 0.5,
                    y: (units[i].y + units[j].y) * 0.5,
                    unit_a: Some(units[i].id),
                    unit_b: Some(units[j].id),
                });
                units[i].strength = (units[i].strength - 0.05).max(0.0);
                units[j].strength = (units[j].strength - 0.05).max(0.0);
            }
        }
    }
    damage_events
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `seed_voxels` writes its starter block without panicking.
    #[test]
    fn seed_voxels_runs() {
        let mut sim = Simulation::with_seed(1);
        seed_voxels(&mut sim);
    }

    /// `seed_military` spawns 5 soldiers per faction, all at full strength with
    /// in-bounds coordinates.
    #[test]
    fn seed_military_spawns_five_per_faction() {
        let mut sim = Simulation::with_seed(42);
        let terrain = Terrain::generate(42);
        let mut units = Vec::new();
        seed_military(&mut sim, &terrain, &mut units);

        let faction_count = factions(sim.state.tick).len();
        assert_eq!(units.len(), faction_count * 5);
        assert!(units.iter().all(|u| u.unit_type == "Soldier"));
        assert!(units.iter().all(|u| (u.strength - 1.0).abs() < f32::EPSILON));
        assert!(
            units
                .iter()
                .all(|u| (0.01..=0.99).contains(&u.x) && (0.01..=0.99).contains(&u.y)),
            "unit coordinates must be clamped in-bounds"
        );
    }

    /// With no Conflict diplomacy events (a fresh, un-ticked sim) `tick_military`
    /// is a no-op: no damage pulses and unit strengths are untouched.
    #[test]
    fn tick_military_is_inert_without_conflict() {
        let mut sim = Simulation::with_seed(42);
        let terrain = Terrain::generate(42);
        let mut units = Vec::new();
        seed_military(&mut sim, &terrain, &mut units);

        let pulses = tick_military(&mut sim, &terrain, &mut units);
        assert!(pulses.is_empty(), "no conflict => no damage pulses");
        assert!(
            units.iter().all(|u| (u.strength - 1.0).abs() < f32::EPSILON),
            "strengths unchanged when there is no conflict"
        );
    }
}
