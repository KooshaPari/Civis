//! Snapshot synthesis from simulation state.

use civ_agents::{drift_toward_home, Civilian as AgentCivilian, Needs, Position3d, Velocity};
use civ_engine::{Citizen, DiplomacyKind, Simulation};
use civ_laws::{LawDb, LawKind};
use civ_server::build_voxel_delta_frame;
use civ_voxel::WorldCoord;
use tracing::warn;

use crate::app::{
    Building, BuildingKind, CivPin, DamagePulse, DiplomacyPulse, DisasterEvent, EconomySnapshot,
    Faction, FactionTreasury, GameEvent, HousingStats, InstitutionRow, MilitaryPin,
    PopulationPulse, ProductionRates, ResourceSnapshot, Road, RoadKind, SampleCivilian, Snapshot,
    TechNode, TradeRoute, TradeTickSummary, WeatherSnapshot,
};

pub(crate) fn make_snapshot(
    sim: &Simulation,
    military: &[MilitaryPin],
    damage_events: &[DamagePulse],
    trade: &TradeTickSummary,
    speed: u8,
    laws: &LawDb,
    current_era: u16,
) -> Snapshot {
    let voxel_events = sim.last_tick_voxel_events();
    let sample_civilians = sample_civilians(sim);
    let civ_pins = civ_pins(sim);
    let factions = factions(sim.state.tick);
    let mut buildings = buildings(&factions, sim.state.tick);
    merge_authoring_buildings(&mut buildings, sim);
    let housing_stats = housing_snapshot(sim, &mut buildings);
    let roads = roads(&buildings);
    let trade_routes = trade_routes(&factions, sim.state.tick);
    let economy = economy_snapshot(sim, &factions, &trade.balances);
    let birth_events: Vec<PopulationPulse> = sim
        .last_births()
        .iter()
        .map(|event| PopulationPulse {
            tick: event.tick,
            entity_id: event.entity_id,
            x: event.x,
            y: event.y,
        })
        .collect::<Vec<PopulationPulse>>();
    let death_events: Vec<PopulationPulse> = sim
        .last_deaths()
        .iter()
        .map(|event| PopulationPulse {
            tick: event.tick,
            entity_id: event.entity_id,
            x: event.x,
            y: event.y,
        })
        .collect::<Vec<PopulationPulse>>();
    let diplomacy_events: Vec<DiplomacyPulse> = sim
        .diplomacy_events()
        .iter()
        .map(|event| DiplomacyPulse {
            tick: event.tick,
            faction_a: event.faction_a,
            faction_b: event.faction_b,
            kind: event.kind,
        })
        .collect();
    let tech_nodes = tech_tree(laws, current_era);
    let disaster_events = disaster_events(sim.state.tick, &factions, &buildings);
    let events = game_events(
        sim,
        &birth_events,
        &death_events,
        &diplomacy_events,
        &disaster_events,
        &buildings,
        &tech_nodes,
    );
    let _ = build_voxel_delta_frame(sim.state.tick, voxel_events, sim.voxel()).map_err(|err| {
        warn!(?err, "voxel frame build failed for current tick");
    });
    let climate = sim.climate();
    let is_day = climate.day_phase >= 0.25 && climate.day_phase < 0.75;
    let weather = weather_snapshot(sim.state.tick, climate.year_phase);
    let tick_dt_ms = 100u32 / u32::from(speed.max(1));

    Snapshot {
        tick: sim.state.tick,
        tick_dt_ms,
        current_era,
        population: sim.state.population,
        voxel_dirty_count: events.len(),
        voxel_chunk_count: sim.voxel().chunk_count(),
        sample_civilians,
        civ_pins,
        factions,
        buildings,
        housing_stats,
        roads,
        trade_routes,
        economy,
        trade_volume_this_tick: trade.volume,
        births_this_tick: birth_events.len() as u32,
        deaths_this_tick: death_events.len() as u32,
        diplomacy_events,
        military_units: military.to_vec(),
        damage_events: damage_events.to_vec(),
        damage_events_count: damage_events.len() as u32,
        disaster_events,
        birth_events,
        death_events,
        tech_tree: tech_nodes,
        events,
        is_day,
        weather,
        speed,
        mods: sim.mod_browser_entries(),
    }
}

pub(crate) fn weather_snapshot(tick: u64, year_phase: f32) -> WeatherSnapshot {
    let season = season_from_year_phase(year_phase);
    let temperature = temperature_from_year_phase(year_phase);
    let precipitation = precipitation_from_weather(&season, temperature);
    let wind_speed = 2.5
        + (year_phase * std::f32::consts::TAU).sin().abs() * 2.0
        + (tick as f32 * 0.000_01).sin().abs() * 0.5
        + season_wind_bias(&season);

    WeatherSnapshot {
        season,
        temperature,
        wind_speed,
        precipitation,
    }
}

pub(crate) fn season_from_year_phase(year_phase: f32) -> String {
    match year_phase {
        phase if phase < 0.25 => "Spring".to_string(),
        phase if phase < 0.5 => "Summer".to_string(),
        phase if phase < 0.75 => "Autumn".to_string(),
        _ => "Winter".to_string(),
    }
}

pub(crate) fn temperature_from_year_phase(year_phase: f32) -> f32 {
    11.0 + (std::f32::consts::TAU * (year_phase - 0.25)).sin() * 17.0
}

pub(crate) fn precipitation_from_weather(season: &str, temperature: f32) -> String {
    match season {
        "Winter" if temperature <= 0.0 => "snow".to_string(),
        "Winter" => "none".to_string(),
        "Spring" | "Autumn" if temperature < 12.0 => "rain".to_string(),
        "Summer" if temperature < 14.0 => "rain".to_string(),
        _ => "none".to_string(),
    }
}

pub(crate) fn season_wind_bias(season: &str) -> f32 {
    match season {
        "Spring" => 1.0,
        "Summer" => 0.4,
        "Autumn" => 1.2,
        "Winter" => 1.6,
        _ => 0.0,
    }
}

pub(crate) fn game_events(
    sim: &Simulation,
    births_this_tick: &[PopulationPulse],
    deaths_this_tick: &[PopulationPulse],
    diplomacy_events: &[DiplomacyPulse],
    disaster_events: &[DisasterEvent],
    buildings: &[Building],
    tech_tree: &[TechNode],
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    let tick = sim.state.tick;

    for birth in births_this_tick {
        let faction_id = faction_for_point(birth.x, birth.y);
        events.push(GameEvent {
            tick: birth.tick,
            kind: "birth".to_string(),
            message: match faction_id {
                Some(id) => format!("A new citizen was born in Faction {id}"),
                None => "A new citizen was born".to_string(),
            },
            faction_id,
        });
    }

    for _death in deaths_this_tick {
        events.push(GameEvent {
            tick,
            kind: "death".to_string(),
            message: "A citizen died".to_string(),
            faction_id: None,
        });
    }

    for disaster in disaster_events {
        events.push(GameEvent {
            tick: disaster.tick,
            kind: "disaster".to_string(),
            message: format!(
                "{} at ({:.2}, {:.2})",
                disaster.kind, disaster.x, disaster.y
            ),
            faction_id: None,
        });
    }

    for diplomacy in diplomacy_events {
        let kind = match diplomacy.kind {
            DiplomacyKind::TradeAgreement => "trade",
            DiplomacyKind::Conflict => "conflict",
            DiplomacyKind::Peace => "peace",
        };
        let message = match diplomacy.kind {
            DiplomacyKind::TradeAgreement => format!(
                "Trade Agreement between Faction {} and Faction {}",
                diplomacy.faction_a, diplomacy.faction_b
            ),
            DiplomacyKind::Conflict => format!(
                "Conflict between Faction {} and Faction {}",
                diplomacy.faction_a, diplomacy.faction_b
            ),
            DiplomacyKind::Peace => format!(
                "Peace declared between Faction {} and Faction {}",
                diplomacy.faction_a, diplomacy.faction_b
            ),
        };
        events.push(GameEvent {
            tick: diplomacy.tick,
            kind: kind.to_string(),
            message,
            faction_id: Some(diplomacy.faction_a),
        });
    }

    for node in tech_tree
        .iter()
        .filter(|node| node.unlocked && node.era_min == (sim.state.tick / 600) as u16)
    {
        events.push(GameEvent {
            tick,
            kind: "tech".to_string(),
            message: format!(
                "Era {} reached: {} technology unlocked",
                node.era_min, node.id
            ),
            faction_id: None,
        });
    }

    let mut mod_buses = sim.replay_log().mod_loaded_bus_at_tick(tick);
    if tick <= 1 {
        for bus in sim.replay_log().mod_loaded_bus_at_tick(0) {
            if !mod_buses.iter().any(|existing| existing == &bus) {
                mod_buses.push(bus.clone());
            }
        }
    }
    for bus in &mod_buses {
        let message = serde_json::from_str::<serde_json::Value>(bus)
            .ok()
            .and_then(|value| {
                value
                    .get("mod_name")
                    .and_then(|name| name.as_str())
                    .map(|name| format!("Mod loaded: {name}"))
            })
            .unwrap_or_else(|| "Mod loaded".to_string());
        events.push(GameEvent {
            tick,
            kind: "mod.loaded".to_string(),
            message,
            faction_id: None,
        });
    }

    for bus in sim.replay_log().session_saved_bus_at_tick(tick) {
        let message = serde_json::from_str::<serde_json::Value>(&bus)
            .ok()
            .and_then(|value| {
                value
                    .get("slot")
                    .and_then(|slot| slot.as_str())
                    .map(|slot| format!("Game saved to {slot}"))
            })
            .unwrap_or_else(|| "Game saved".to_string());
        events.push(GameEvent {
            tick,
            kind: "session.saved".to_string(),
            message,
            faction_id: None,
        });
    }

    for bus in sim.replay_log().mod_permission_violation_bus_at_tick(tick) {
        let message = serde_json::from_str::<serde_json::Value>(&bus)
            .ok()
            .and_then(|value| {
                let mod_id = value.get("mod_id").and_then(|id| id.as_str())?;
                let call = value.get("call").and_then(|call| call.as_str())?;
                Some(format!("Mod {mod_id} denied: {call}"))
            })
            .unwrap_or_else(|| "Mod permission denied".to_string());
        events.push(GameEvent {
            tick,
            kind: "mod.permission_violation".to_string(),
            message,
            faction_id: None,
        });
    }

    for building in buildings {
        if matches!(building.kind, BuildingKind::Residential) {
            events.push(GameEvent {
                tick,
                kind: "building".to_string(),
                message: format!(
                    "New Residential building in Faction {}",
                    building.faction_id
                ),
                faction_id: Some(building.faction_id),
            });
        }
    }

    events.sort_by(|a, b| {
        a.tick
            .cmp(&b.tick)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.message.cmp(&b.message))
    });
    events
        .into_iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

pub(crate) fn disaster_events(
    tick: u64,
    factions: &[Faction],
    buildings: &[Building],
) -> Vec<DisasterEvent> {
    if tick == 0 || tick % 1000 != 0 {
        return Vec::new();
    }
    let roll = hash01(tick as f32 * 0.017);
    if roll < 0.25 {
        return vec![DisasterEvent {
            tick,
            kind: "Earthquake".to_string(),
            x: hash01(tick as f32 * 0.11) * 0.8 + 0.1,
            y: hash01(tick as f32 * 0.19 + 3.0) * 0.8 + 0.1,
            radius: 0.18,
            severity: 0.55,
        }];
    }
    if roll < 0.5 {
        let (x, y) = buildings
            .iter()
            .find(|building| building.kind == BuildingKind::Residential)
            .map(|building| (building.x, building.y))
            .unwrap_or_else(|| {
                (
                    hash01(tick as f32 * 0.07) * 0.8 + 0.1,
                    hash01(tick as f32 * 0.13 + 9.0) * 0.8 + 0.1,
                )
            });
        return vec![DisasterEvent {
            tick,
            kind: "Wildfire".to_string(),
            x,
            y,
            radius: 0.12,
            severity: 0.7,
        }];
    }
    if roll < 0.75 {
        let center = factions
            .first()
            .map(|faction| faction.capital)
            .unwrap_or([0.5, 0.5]);
        return vec![DisasterEvent {
            tick,
            kind: "Flood".to_string(),
            x: center[0],
            y: center[1],
            radius: 0.22,
            severity: 0.6,
        }];
    }
    vec![DisasterEvent {
        tick,
        kind: "Plague".to_string(),
        x: 0.5,
        y: 0.5,
        radius: 0.26,
        severity: 0.1,
    }]
}

pub(crate) fn hash01(value: f32) -> f32 {
    let hashed = (value * 12.9898).sin() * 43_758.547;
    hashed - hashed.floor()
}

pub(crate) fn faction_for_point(x: f32, y: f32) -> Option<u32> {
    factions(0)
        .into_iter()
        .min_by(|a, b| {
            let da = (x - a.capital[0]).powi(2) + (y - a.capital[1]).powi(2);
            let db = (x - b.capital[0]).powi(2) + (y - b.capital[1]).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|faction| faction.id)
}

pub(crate) fn tech_tree(db: &LawDb, current_era: u16) -> Vec<TechNode> {
    let mut nodes = db
        .laws
        .iter()
        .map(|law| TechNode {
            id: law.id.clone(),
            kind: match law.kind {
                LawKind::Conservation => "Conservation".to_string(),
                LawKind::Material => "Material".to_string(),
                LawKind::FictionalExtension => "FictionalExtension".to_string(),
            },
            era_min: law.era_min,
            unlocked: current_era >= law.era_min,
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.era_min.cmp(&b.era_min).then_with(|| a.id.cmp(&b.id)));
    nodes
}

pub(crate) fn economy_snapshot(
    sim: &Simulation,
    factions: &[Faction],
    trade_balances_this_tick: &std::collections::HashMap<u32, f64>,
) -> EconomySnapshot {
    let energy_budget = sim.state.energy_budget_joules.to_f64();
    let resources = &sim.state.resources;
    let faction_treasury = factions
        .iter()
        .map(|faction| {
            let name = sim
                .state
                .factions
                .get(&faction.id)
                .cloned()
                .unwrap_or_else(|| format!("Faction {}", faction.id));
            let balance = sim
                .state
                .faction_treasury
                .get(&faction.id)
                .map(|value| value.to_f64())
                .unwrap_or(0.0);
            FactionTreasury {
                id: faction.id,
                name,
                balance,
                trade_balance: *trade_balances_this_tick.get(&faction.id).unwrap_or(&0.0),
            }
        })
        .collect();

    let mut food_per_tick = 0.0;
    let wood_per_tick = 0.0;
    let mut metal_per_tick = 0.0;
    for (_, building) in sim.world.query::<&civ_engine::Building>().iter() {
        match building.building_type {
            civ_engine::BuildingType::Farm => food_per_tick += 10.0,
            civ_engine::BuildingType::Mine => metal_per_tick += 5.0,
            _ => {}
        }
    }

    let institutions = civ_server::jsonrpc::institutions_from_sim(sim)
        .into_iter()
        .map(|row| InstitutionRow {
            id: row.id,
            kind: row.kind.to_string(),
            balance_joules: row.balance_joules,
        })
        .collect();

    EconomySnapshot {
        energy_budget,
        faction_treasury,
        production_rates: ProductionRates {
            food_per_tick,
            wood_per_tick,
            metal_per_tick,
            energy_per_tick: energy_budget / 1000.0,
        },
        institutions,
        resources: ResourceSnapshot {
            food: resources.food.to_f64(),
            wood: resources.wood.to_f64(),
            metal: resources.metal.to_f64(),
            energy: resources.energy.to_f64(),
        },
    }
}

pub(crate) fn sample_civilians(sim: &Simulation) -> Vec<SampleCivilian> {
    sim.world
        .query::<&Citizen>()
        .iter()
        .take(8)
        .map(|(_, citizen)| SampleCivilian {
            age: citizen.age,
            health: citizen.health.to_f64(),
            ideology: citizen.ideology.to_f64(),
            welfare: citizen.welfare.to_f64(),
            job: None,
        })
        .collect()
}

pub(crate) fn civ_pins(sim: &Simulation) -> Vec<CivPin> {
    let mut pins = Vec::new();
    for (idx, (_, (_civilian, pos, vel))) in sim
        .world
        .query::<(&AgentCivilian, &Position3d, &Velocity)>()
        .iter()
        .enumerate()
    {
        let x = normalize_world_coord(pos.coord.x);
        let y = normalize_world_coord(pos.coord.z);
        pins.push(CivPin {
            idx: idx as u32,
            x,
            y,
            dx: vel.dx,
            dy: vel.dy,
            job: None,
        });
    }
    pins.sort_by_key(|pin| pin.idx);
    pins
}

pub(crate) fn assign_and_drift_housing(sim: &mut Simulation, buildings: &[Building]) {
    let world = &mut sim.world;
    let homes: Vec<_> = buildings
        .iter()
        .filter(|building| {
            matches!(building.kind, BuildingKind::Residential) && building.capacity > 0
        })
        .collect();
    let mut occupancy: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
    let mut home_lookup = std::collections::BTreeMap::new();
    for building in &homes {
        home_lookup.insert(building.id, (building.x, building.y));
    }

    for (_, (_civilian, pos, vel, needs)) in
        world.query_mut::<(&AgentCivilian, &Position3d, &mut Velocity, &Needs)>()
    {
        if needs.shelter <= 0.5 {
            continue;
        }
        let mut selected = None;
        for building in homes.iter() {
            let used = occupancy.get(&building.id).copied().unwrap_or(0);
            if used < building.capacity {
                selected = Some(*building);
                break;
            }
        }
        if let Some(home) = selected {
            occupancy
                .entry(home.id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            let home_pos = Position3d {
                coord: WorldCoord {
                    x: (home.x * civ_voxel::FIXED_SCALE as f32) as i64,
                    y: 0,
                    z: (home.y * civ_voxel::FIXED_SCALE as f32) as i64,
                },
            };
            let drifted = drift_toward_home(pos, &home_pos, *vel, needs.shelter);
            vel.dx = drifted.dx;
            vel.dy = drifted.dy;
        }
    }
}

pub(crate) fn factions(tick: u64) -> Vec<Faction> {
    let base_radius = 0.05 + (tick as f32 * 0.000_02).min(0.12);
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
            radius: (base_radius + idx as f32 * 0.018).min(0.3),
        })
        .collect()
}

pub(crate) fn normalize_world_coord(coord: i64) -> f32 {
    (coord as f32 / civ_voxel::FIXED_SCALE as f32).clamp(0.0, 1.0)
}

pub(crate) fn buildings(factions: &[Faction], tick: u64) -> Vec<Building> {
    let kinds = [
        BuildingKind::Residential,
        BuildingKind::Commercial,
        BuildingKind::Industrial,
        BuildingKind::Civic,
    ];
    let mut buildings = Vec::new();
    for faction in factions {
        for i in 0..3 {
            let idx = faction.id * 3 + i;
            let seed = u64::from(idx)
                .wrapping_mul(1_103_515_245)
                .wrapping_add(tick / 120);
            let x = wrap01(faction.capital[0] + noise_offset(seed, 0));
            let y = wrap01(faction.capital[1] + noise_offset(seed, 1));
            buildings.push(Building {
                id: idx,
                x,
                y,
                kind: kinds[(idx as usize) % kinds.len()].clone(),
                era: ((tick / 600) % 6) as u8,
                faction_id: faction.id,
                occupants: 0,
                capacity: match kinds[(idx as usize) % kinds.len()] {
                    BuildingKind::Residential => 4,
                    _ => 0,
                },
            });
        }
    }
    buildings
}

/// Placed airports / ports / hangars from ECS authoring (FR-CIV-UX-006).
pub(crate) fn merge_authoring_buildings(buildings: &mut Vec<Building>, sim: &Simulation) {
    use civ_engine::{grid_to_norm, BuildingType};

    for (idx, (_, building)) in sim
        .world
        .query::<&civ_engine::Building>()
        .iter()
        .enumerate()
    {
        let (x, y) = grid_to_norm(building.position);
        let (kind, id_base) = match building.building_type {
            BuildingType::CityCenter => (BuildingKind::Civic, 9_000_u32),
            BuildingType::Market => (BuildingKind::Commercial, 9_100_u32),
            BuildingType::Barracks => (BuildingKind::Industrial, 9_200_u32),
            _ => continue,
        };
        buildings.push(Building {
            id: id_base + idx as u32,
            x,
            y,
            kind,
            era: ((sim.state.tick / 600) % 6) as u8,
            faction_id: 0,
            occupants: 0,
            capacity: 0,
        });
    }
}

pub(crate) fn housing_snapshot(sim: &Simulation, buildings: &mut [Building]) -> HousingStats {
    let needy_count = sim
        .world
        .query::<(&AgentCivilian, &Needs)>()
        .iter()
        .filter(|(_, (_, needs))| needs.shelter > 0.5)
        .count() as u32;
    let total_capacity = buildings.iter().map(|building| building.capacity).sum();
    let occupied = needy_count.min(total_capacity);
    let homeless = needy_count.saturating_sub(total_capacity);
    let mut remaining = occupied;
    for building in buildings.iter_mut() {
        if building.capacity == 0 {
            building.occupants = 0;
            continue;
        }
        let assigned = remaining.min(building.capacity);
        building.occupants = assigned;
        remaining = remaining.saturating_sub(assigned);
    }
    let vacancy_rate = if total_capacity == 0 {
        0.0
    } else {
        (total_capacity.saturating_sub(occupied)) as f32 / total_capacity as f32
    };

    HousingStats {
        total_capacity,
        occupied,
        homeless,
        vacancy_rate,
    }
}

pub(crate) fn roads(buildings: &[Building]) -> Vec<Road> {
    let mut roads = Vec::new();
    let mut by_faction: std::collections::BTreeMap<u32, Vec<&Building>> =
        std::collections::BTreeMap::new();
    for building in buildings {
        by_faction
            .entry(building.faction_id)
            .or_default()
            .push(building);
    }

    for faction_buildings in by_faction.values_mut() {
        faction_buildings.sort_by_key(|building| building.id);
        for pair in faction_buildings.windows(2) {
            let from = pair[0];
            let to = pair[1];
            let distance = ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt();
            let kind = if distance < 0.03 {
                RoadKind::Trail
            } else if distance < 0.06 {
                RoadKind::Dirt
            } else if distance < 0.10 {
                RoadKind::Paved
            } else {
                RoadKind::Highway
            };
            let width = match kind {
                RoadKind::Trail => 0.2,
                RoadKind::Dirt => 0.4,
                RoadKind::Paved => 0.6,
                RoadKind::Highway => 1.0,
            };
            roads.push(Road {
                from: [from.x, from.y],
                to: [to.x, to.y],
                width,
                kind,
            });
        }
    }

    roads
}

pub(crate) fn trade_routes(factions: &[Faction], tick: u64) -> Vec<TradeRoute> {
    let goods = ["grain", "timber", "ore", "cloth", "salt", "tools"];
    let mut routes = Vec::new();
    for (idx, from) in factions.iter().enumerate() {
        for to in factions.iter().skip(idx + 1) {
            let goods_idx = ((tick / 180) as usize + idx + to.id as usize) % goods.len();
            let volume = 8.0 + (((tick / 30) as f32 + from.id as f32 + to.id as f32) % 16.0);
            routes.push(TradeRoute {
                from_faction: from.id,
                to_faction: to.id,
                goods: goods[goods_idx].to_string(),
                volume,
            });
        }
    }
    routes
}

pub(crate) fn apply_trade_routes(
    sim: &mut Simulation,
    factions: &[Faction],
    tick: u64,
) -> (f64, std::collections::HashMap<u32, f64>) {
    let routes = trade_routes(factions, tick);
    let diplomacy = sim
        .diplomacy_events()
        .iter()
        .map(|event| {
            (
                (
                    event.faction_a.min(event.faction_b),
                    event.faction_a.max(event.faction_b),
                ),
                event.kind,
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut trade_volume_this_tick = 0.0;
    let mut trade_balances = std::collections::HashMap::new();
    for route in routes {
        let key = (
            route.from_faction.min(route.to_faction),
            route.from_faction.max(route.to_faction),
        );
        let Some(kind) = diplomacy.get(&key).copied() else {
            continue;
        };
        if !matches!(kind, DiplomacyKind::Peace | DiplomacyKind::TradeAgreement) {
            continue;
        }

        let resource = route_resource(&route.goods);
        let supply = resource_amount(&sim.state.resources, resource);
        let demand = resource_demand(&sim.state.resources, resource);
        let trade_price = 1.0 + (demand - supply) * 0.1;
        let quantity = f64::from(route.volume) * 0.5;
        let treasury_delta = f64::from(route.volume) * trade_price;

        adjust_resource(&mut sim.state.resources, resource, -quantity);
        adjust_treasury(
            &mut sim.state.faction_treasury,
            route.from_faction,
            treasury_delta,
        );
        *trade_balances.entry(route.from_faction).or_insert(0.0) += treasury_delta;
        adjust_resource(&mut sim.state.resources, resource, quantity);
        adjust_treasury(
            &mut sim.state.faction_treasury,
            route.to_faction,
            -treasury_delta,
        );
        *trade_balances.entry(route.to_faction).or_insert(0.0) -= treasury_delta;
        trade_volume_this_tick += f64::from(route.volume);
    }

    (trade_volume_this_tick, trade_balances)
}

pub(crate) fn route_resource(goods: &str) -> civ_engine::ResourceType {
    match goods {
        "grain" => civ_engine::ResourceType::Food,
        "timber" => civ_engine::ResourceType::Wood,
        "ore" | "tools" => civ_engine::ResourceType::Metal,
        "cloth" | "salt" => civ_engine::ResourceType::Energy,
        _ => civ_engine::ResourceType::Food,
    }
}

pub(crate) fn resource_amount(
    resources: &civ_engine::Resources,
    resource: civ_engine::ResourceType,
) -> f64 {
    match resource {
        civ_engine::ResourceType::Food => resources.food.to_f64(),
        civ_engine::ResourceType::Wood => resources.wood.to_f64(),
        civ_engine::ResourceType::Metal => resources.metal.to_f64(),
        civ_engine::ResourceType::Energy => resources.energy.to_f64(),
    }
}

pub(crate) fn resource_demand(
    resources: &civ_engine::Resources,
    resource: civ_engine::ResourceType,
) -> f64 {
    (1000.0 - resource_amount(resources, resource)).max(0.0)
}

pub(crate) fn fixed_from_f64(value: f64) -> civ_engine::Fixed {
    civ_engine::Fixed::from_raw((value * civ_engine::SCALE as f64).round() as i64)
}

pub(crate) fn adjust_resource(
    resources: &mut civ_engine::Resources,
    resource: civ_engine::ResourceType,
    delta: f64,
) {
    let delta = fixed_from_f64(delta);
    match resource {
        civ_engine::ResourceType::Food => resources.food += delta,
        civ_engine::ResourceType::Wood => resources.wood += delta,
        civ_engine::ResourceType::Metal => resources.metal += delta,
        civ_engine::ResourceType::Energy => resources.energy += delta,
    }
}

pub(crate) fn adjust_treasury(
    treasury: &mut std::collections::HashMap<u32, civ_engine::Fixed>,
    faction_id: u32,
    delta: f64,
) {
    if let Some(balance) = treasury.get_mut(&faction_id) {
        *balance += fixed_from_f64(delta);
    }
}

pub(crate) fn noise_offset(seed: u64, lane: u64) -> f32 {
    let mixed = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(lane.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    let unit = ((mixed >> 40) as f32) / ((1u64 << 24) as f32);
    (unit - 0.5) * 0.10
}

pub(crate) fn wrap01(value: f32) -> f32 {
    value.rem_euclid(1.0)
}
