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
        settlement_count: sim.settlement_count(),
        cluster_stocks: sim.cluster_stocks().clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::default_law_db;

    #[test]
    fn season_from_year_phase_partitions_the_year() {
        assert_eq!(season_from_year_phase(0.0), "Spring");
        assert_eq!(season_from_year_phase(0.24), "Spring");
        assert_eq!(season_from_year_phase(0.25), "Summer");
        assert_eq!(season_from_year_phase(0.49), "Summer");
        assert_eq!(season_from_year_phase(0.5), "Autumn");
        assert_eq!(season_from_year_phase(0.74), "Autumn");
        assert_eq!(season_from_year_phase(0.75), "Winter");
        assert_eq!(season_from_year_phase(1.0), "Winter");
    }

    #[test]
    fn season_wind_bias_is_defined_per_season_and_zero_otherwise() {
        assert_eq!(season_wind_bias("Spring"), 1.0);
        assert_eq!(season_wind_bias("Summer"), 0.4);
        assert_eq!(season_wind_bias("Autumn"), 1.2);
        assert_eq!(season_wind_bias("Winter"), 1.6);
        assert_eq!(season_wind_bias("nonsense"), 0.0);
    }

    #[test]
    fn precipitation_matches_season_and_temperature() {
        assert_eq!(precipitation_from_weather("Winter", -2.0), "snow");
        assert_eq!(precipitation_from_weather("Winter", 3.0), "none");
        assert_eq!(precipitation_from_weather("Spring", 5.0), "rain");
        assert_eq!(precipitation_from_weather("Summer", 30.0), "none");
    }

    #[test]
    fn hash01_is_always_in_unit_interval() {
        for v in [0.0_f32, 1.0, -3.5, 12.9898, 100.0, -0.0001] {
            let h = hash01(v);
            assert!((0.0..1.0).contains(&h), "hash01({v}) = {h} out of [0,1)");
        }
    }

    #[test]
    fn wrap01_maps_any_real_into_unit_interval() {
        for v in [0.0_f32, 0.5, 1.0, 1.25, -0.25, -3.75, 42.6] {
            let w = wrap01(v);
            assert!((0.0..1.0).contains(&w), "wrap01({v}) = {w} out of [0,1)");
        }
    }

    #[test]
    fn noise_offset_is_deterministic_and_bounded() {
        // Same (seed, lane) -> same output; magnitude within +/-0.05.
        assert_eq!(noise_offset(42, 0), noise_offset(42, 0));
        assert_ne!(noise_offset(42, 0), noise_offset(43, 0));
        for (s, l) in [(0_u64, 0_u64), (1, 1), (999, 7), (u64::MAX, 3)] {
            let n = noise_offset(s, l);
            assert!(
                n.abs() <= 0.05 + f32::EPSILON,
                "noise_offset({s},{l}) = {n}"
            );
        }
    }

    #[test]
    fn temperature_from_year_phase_stays_in_physical_band() {
        // 11 +/- 17 -> roughly [-6, 28] across the year.
        for i in 0..=100 {
            let t = temperature_from_year_phase(i as f32 / 100.0);
            assert!((-6.5..=28.5).contains(&t), "temp {t} out of band");
        }
    }

    #[test]
    fn faction_for_point_always_resolves_to_a_faction() {
        // factions(0) is non-empty, so every point maps to a nearest faction.
        assert!(faction_for_point(0.5, 0.5).is_some());
        assert!(faction_for_point(0.0, 0.0).is_some());
        assert!(faction_for_point(1.0, 1.0).is_some());
    }

    #[test]
    fn snapshot_pure_weather_resource_and_coord_helpers() {
        assert_eq!(season_from_year_phase(0.0), "Spring");
        assert_eq!(season_from_year_phase(0.3), "Summer");
        assert_eq!(season_from_year_phase(0.6), "Autumn");
        assert_eq!(season_from_year_phase(0.9), "Winter");

        assert_eq!(season_wind_bias("Spring"), 1.0);
        assert_eq!(season_wind_bias("Summer"), 0.4);
        assert_eq!(season_wind_bias("Autumn"), 1.2);
        assert_eq!(season_wind_bias("Winter"), 1.6);
        assert_eq!(season_wind_bias("Nonsense"), 0.0);

        assert_eq!(precipitation_from_weather("Winter", -5.0), "snow");
        assert_eq!(precipitation_from_weather("Winter", 5.0), "none");
        assert_eq!(precipitation_from_weather("Spring", 5.0), "rain");
        assert_eq!(precipitation_from_weather("Summer", 10.0), "rain");
        assert_eq!(precipitation_from_weather("Summer", 30.0), "none");

        for v in [0.0_f32, 1.0, 42.5, -3.0] {
            assert!((0.0..1.0).contains(&hash01(v)), "hash01({v}) out of [0,1)");
        }

        assert_eq!(normalize_world_coord(0), 0.0);
        assert_eq!(normalize_world_coord(i64::MAX), 1.0);
        assert_eq!(normalize_world_coord(-100), 0.0);

        use civ_engine::ResourceType;
        assert_eq!(route_resource("grain"), ResourceType::Food);
        assert_eq!(route_resource("timber"), ResourceType::Wood);
        assert_eq!(route_resource("ore"), ResourceType::Metal);
        assert_eq!(route_resource("tools"), ResourceType::Metal);
        assert_eq!(route_resource("cloth"), ResourceType::Energy);
        assert_eq!(route_resource("unknown"), ResourceType::Food);
    }

    #[test]
    fn factions_are_four_stable_with_ascending_ids() {
        let f = factions(0);
        assert_eq!(f.len(), 4);
        assert_eq!(f.iter().map(|x| x.id).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert!(f.iter().all(|x| x.radius > 0.0 && x.radius <= 0.3));
        assert_eq!(f[0].capital, [0.22, 0.24]);
        assert!(factions(100_000)[0].radius >= factions(0)[0].radius);
    }

    #[test]
    fn faction_for_point_picks_nearest_capital() {
        assert_eq!(faction_for_point(0.22, 0.24), Some(0));
        assert_eq!(faction_for_point(0.76, 0.27), Some(1));
        assert_eq!(faction_for_point(0.27, 0.73), Some(2));
        assert_eq!(faction_for_point(0.72, 0.74), Some(3));
        assert!(faction_for_point(0.5, 0.5).is_some());
    }

    #[test]
    fn buildings_three_per_faction_with_valid_coords() {
        let f = factions(0);
        let b = buildings(&f, 0);
        assert_eq!(b.len(), f.len() * 3);
        assert!(b
            .iter()
            .all(|x| (0.0..1.0).contains(&x.x) && (0.0..1.0).contains(&x.y)));
        assert!(b.iter().all(|x| x.faction_id < 4));
        assert!(b.iter().all(|x| x.occupants == 0));
        assert!(b
            .iter()
            .all(|x| matches!(x.kind, BuildingKind::Residential) == (x.capacity == 4)));
    }

    #[test]
    fn roads_connect_same_faction_buildings() {
        let f = factions(0);
        let b = buildings(&f, 0);
        let r = roads(&b);
        assert!(!r.is_empty());
        let faction_at = |x: f32, y: f32| -> Option<u32> {
            b.iter()
                .find(|building| building.x == x && building.y == y)
                .map(|building| building.faction_id)
        };
        for road in &r {
            let from_faction = faction_at(road.from[0], road.from[1])
                .expect("road.from references a known building");
            let to_faction =
                faction_at(road.to[0], road.to[1]).expect("road.to references a known building");
            assert_eq!(
                from_faction, to_faction,
                "road endpoints must belong to the same faction"
            );
        }
    }

    #[test]
    fn wrap01_wraps_into_unit_interval() {
        assert_eq!(wrap01(0.0), 0.0);
        assert_eq!(wrap01(0.5), 0.5);
        assert_eq!(wrap01(1.0), 0.0);
        assert_eq!(wrap01(1.25), 0.25);
        let w = wrap01(-0.25);
        assert!((w - 0.75).abs() < 1e-6);
        assert!((0.0..1.0).contains(&wrap01(7.3)));
        assert!((0.0..1.0).contains(&wrap01(-7.3)));
    }

    #[test]
    fn noise_offset_is_bounded_and_deterministic() {
        for (s, l) in [(0u64, 0u64), (1, 0), (42, 3), (u64::MAX, 7)] {
            let n = noise_offset(s, l);
            // `unit` is in [0.0, 1.0) so `(unit - 0.5) * 0.10` is in [-0.05, 0.05);
            // the lower bound is inclusive (seed=lane=0 hashes to exactly -0.05).
            assert!((-0.05..0.05).contains(&n), "noise {n} out of band");
        }
        assert_eq!(noise_offset(123, 4), noise_offset(123, 4));
    }

    #[test]
    fn resource_demand_never_negative() {
        let r = civ_engine::Resources::default();
        for res in [
            civ_engine::ResourceType::Food,
            civ_engine::ResourceType::Wood,
            civ_engine::ResourceType::Metal,
            civ_engine::ResourceType::Energy,
        ] {
            assert!(resource_demand(&r, res) >= 0.0);
        }
    }

    #[test]
    fn trade_routes_one_per_faction_pair() {
        let f = factions(0);
        let routes = trade_routes(&f, 0);
        assert_eq!(routes.len(), f.len() * (f.len() - 1) / 2);
        let goods = ["grain", "timber", "ore", "cloth", "salt", "tools"];
        assert!(routes.iter().all(|r| goods.contains(&r.goods.as_str())));
        assert!(routes.iter().all(|r| r.from_faction != r.to_faction));
        assert!(routes.iter().all(|r| r.volume >= 8.0 && r.volume < 24.0));
    }

    #[test]
    fn disaster_events_are_gated_to_nonzero_kiloticks() {
        let f = factions(0);
        let b = buildings(&f, 0);
        assert!(disaster_events(0, &f, &b).is_empty());
        assert!(disaster_events(1, &f, &b).is_empty());
        assert!(disaster_events(999, &f, &b).is_empty());
        assert!(disaster_events(1500, &f, &b).is_empty());
        assert_eq!(disaster_events(1000, &f, &b).len(), 1);
    }

    #[test]
    fn disaster_event_is_well_formed_when_fired() {
        let f = factions(0);
        let b = buildings(&f, 0);
        let kinds = ["Earthquake", "Wildfire", "Flood", "Plague"];
        for k in 1..=8u64 {
            let tick = k * 1000;
            let events = disaster_events(tick, &f, &b);
            assert_eq!(
                events.len(),
                1,
                "tick {tick} must fire exactly one disaster"
            );
            let e = &events[0];
            assert_eq!(e.tick, tick);
            assert!(
                kinds.contains(&e.kind.as_str()),
                "unexpected kind {}",
                e.kind
            );
            assert!(
                e.x > 0.0 && e.x < 1.0 && e.y > 0.0 && e.y < 1.0,
                "coords out of map: {},{}",
                e.x,
                e.y
            );
            assert!(e.radius > 0.0);
            assert!((0.0..=1.0).contains(&e.severity));
        }
    }

    #[test]
    fn game_events_empty_inputs_yield_no_lifecycle_events() {
        let sim = Simulation::with_seed(7);
        let events = game_events(&sim, &[], &[], &[], &[], &[], &[]);
        assert!(events
            .iter()
            .all(|e| !["birth", "death", "disaster"].contains(&e.kind.as_str())));
    }

    #[test]
    fn game_events_emits_one_birth_and_one_death() {
        let sim = Simulation::with_seed(7);
        let births = vec![PopulationPulse {
            tick: 5,
            entity_id: 1,
            x: 0.22,
            y: 0.24,
        }];
        let deaths = vec![PopulationPulse {
            tick: 6,
            entity_id: 2,
            x: 0.5,
            y: 0.5,
        }];
        let events = game_events(&sim, &births, &deaths, &[], &[], &[], &[]);
        assert_eq!(events.iter().filter(|e| e.kind == "birth").count(), 1);
        assert_eq!(events.iter().filter(|e| e.kind == "death").count(), 1);
        let birth = events.iter().find(|e| e.kind == "birth").unwrap();
        assert!(birth.faction_id.is_some());
    }

    #[test]
    fn game_events_emits_one_disaster_event() {
        let sim = Simulation::with_seed(7);
        let disasters = vec![DisasterEvent {
            tick: 1000,
            kind: "Earthquake".to_string(),
            x: 0.5,
            y: 0.5,
            radius: 0.18,
            severity: 0.55,
        }];
        let events = game_events(&sim, &[], &[], &[], &disasters, &[], &[]);
        assert_eq!(events.iter().filter(|e| e.kind == "disaster").count(), 1);
    }

    #[test]
    fn tech_tree_maps_every_law_and_sorts_by_era() {
        let db = default_law_db();
        let nodes = tech_tree(&db, 0);
        assert_eq!(nodes.len(), db.laws.len());
        assert!(nodes.windows(2).all(|w| w[0].era_min <= w[1].era_min));
        let kinds = ["Conservation", "Material", "FictionalExtension"];
        assert!(nodes.iter().all(|n| kinds.contains(&n.kind.as_str())));
    }

    #[test]
    fn tech_tree_unlocks_follow_current_era() {
        let db = default_law_db();
        let at0 = tech_tree(&db, 0);
        assert!(at0.iter().all(|n| n.unlocked == (n.era_min == 0)));
        let at_max = tech_tree(&db, u16::MAX);
        assert!(at_max.iter().all(|n| n.unlocked));
        let era5 = tech_tree(&db, 5);
        assert!(era5.iter().all(|n| n.unlocked == (5u16 >= n.era_min)));
    }

    #[test]
    fn sample_civilians_caps_at_eight() {
        let sim = Simulation::with_seed(7);
        let sample = sample_civilians(&sim);
        assert!(sample.len() <= 8, "sample_civilians must take at most 8");
    }

    #[test]
    fn civ_pins_are_sorted_by_idx_and_in_bounds() {
        let sim = Simulation::with_seed(7);
        let pins = civ_pins(&sim);
        assert!(pins.windows(2).all(|w| w[0].idx <= w[1].idx));
        assert!(pins
            .iter()
            .all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y)));
    }

    #[test]
    fn economy_snapshot_mirrors_factions_and_rates() {
        use std::collections::HashMap;
        let sim = Simulation::with_seed(7);
        let factions = factions(0);
        let balances: HashMap<u32, f64> = HashMap::new();
        let econ = economy_snapshot(&sim, &factions, &balances);

        assert_eq!(econ.faction_treasury.len(), factions.len());
        assert!(econ
            .faction_treasury
            .iter()
            .zip(factions.iter())
            .all(|(t, f)| t.id == f.id));
        assert!(econ.faction_treasury.iter().all(|t| t.trade_balance == 0.0));
        assert_eq!(econ.production_rates.wood_per_tick, 0.0);
        assert!((econ.production_rates.energy_per_tick * 1000.0 - econ.energy_budget).abs() < 1e-6);
        assert!(econ.production_rates.food_per_tick >= 0.0);
        assert!(econ.production_rates.metal_per_tick >= 0.0);
        assert!(
            econ.resources.food.is_finite()
                && econ.resources.wood.is_finite()
                && econ.resources.metal.is_finite()
                && econ.resources.energy.is_finite()
        );
    }

    #[test]
    fn housing_snapshot_invariants_hold() {
        let sim = Simulation::with_seed(7);
        let factions = factions(0);
        let mut blds = buildings(&factions, 0);
        let total_cap: u32 = blds.iter().map(|b| b.capacity).sum();
        let stats = housing_snapshot(&sim, &mut blds);
        // capacity is the sum of building capacities:
        assert_eq!(stats.total_capacity, total_cap);
        // occupied never exceeds capacity:
        assert!(stats.occupied <= stats.total_capacity);
        // occupants assigned across capacity'd buildings sum to `occupied`:
        let assigned: u32 = blds
            .iter()
            .filter(|b| b.capacity > 0)
            .map(|b| b.occupants)
            .sum();
        assert_eq!(assigned, stats.occupied);
        // zero-capacity buildings hold no occupants:
        assert!(blds
            .iter()
            .filter(|b| b.capacity == 0)
            .all(|b| b.occupants == 0));
        // vacancy_rate is a valid fraction (and 0.0 only-if no capacity):
        assert!((0.0..=1.0).contains(&stats.vacancy_rate));
        if stats.total_capacity == 0 {
            assert_eq!(stats.vacancy_rate, 0.0);
        }
    }

    #[test]
    fn resource_amount_demand_and_adjust_cover_every_resource_arm() {
        use civ_engine::{ResourceType, Resources};
        let mut res = Resources {
            food: fixed_from_f64(100.0),
            wood: fixed_from_f64(200.0),
            metal: fixed_from_f64(300.0),
            energy: fixed_from_f64(400.0),
        };
        let arms = [
            (ResourceType::Food, 100.0),
            (ResourceType::Wood, 200.0),
            (ResourceType::Metal, 300.0),
            (ResourceType::Energy, 400.0),
        ];
        for (r, want) in arms {
            assert!(
                (resource_amount(&res, r) - want).abs() < 1e-6,
                "amount {r:?}"
            );
            // demand = max(0, 1000 - amount); all of these are < 1000 so it's positive.
            assert!(
                (resource_demand(&res, r) - (1000.0 - want)).abs() < 1e-6,
                "demand {r:?}"
            );
        }

        // adjust_resource hits each arm; +50 then re-read.
        for (r, want) in arms {
            adjust_resource(&mut res, r, 50.0);
            assert!(
                (resource_amount(&res, r) - (want + 50.0)).abs() < 1e-6,
                "adjusted {r:?}"
            );
        }

        // demand clamps to 0 once amount exceeds the 1000 cap.
        adjust_resource(&mut res, ResourceType::Food, 5000.0);
        assert_eq!(resource_demand(&res, ResourceType::Food), 0.0);
    }

    #[test]
    fn adjust_treasury_updates_present_faction_and_ignores_absent() {
        use std::collections::HashMap;
        let mut treasury: HashMap<u32, civ_engine::Fixed> = HashMap::new();
        treasury.insert(1, fixed_from_f64(100.0));

        adjust_treasury(&mut treasury, 1, 25.0);
        assert!((treasury[&1].to_f64() - 125.0).abs() < 1e-6);

        // An absent faction is a silent no-op (no insert).
        adjust_treasury(&mut treasury, 99, 50.0);
        assert!(!treasury.contains_key(&99));
    }

    #[test]
    fn tech_tree_maps_every_law_kind_sorts_by_era_and_flags_unlocked() {
        // One law per LawKind at ascending eras so every match arm is hit.
        let db = LawDb::load_ron(
            r#"(
                version: 0,
                laws: [
                    (id: "mass_conservation", kind: Conservation, era_min: 0,
                     inputs: [], outputs: [], losses: [], dependencies: []),
                    (id: "steel", kind: Material, era_min: 4,
                     inputs: [], outputs: [], losses: [], dependencies: []),
                    (id: "fusion_power", kind: FictionalExtension, era_min: 9,
                     inputs: [], outputs: [], losses: [], dependencies: []),
                ],
            )"#,
        )
        .expect("valid law db");

        let nodes = tech_tree(&db, 5);
        assert_eq!(nodes.len(), 3);
        // Sorted by era_min ascending.
        assert_eq!(
            nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
            ["mass_conservation", "steel", "fusion_power"]
        );
        assert_eq!(
            nodes.iter().map(|n| n.kind.as_str()).collect::<Vec<_>>(),
            ["Conservation", "Material", "FictionalExtension"]
        );
        // current_era 5 unlocks eras 0 and 4 but not 9.
        assert_eq!(
            nodes.iter().map(|n| n.unlocked).collect::<Vec<_>>(),
            [true, true, false]
        );
    }

    #[test]
    fn roads_classify_kind_and_width_by_distance() {
        use crate::app::{Building, BuildingKind, RoadKind};
        // 5 same-faction buildings on the x-axis with consecutive gaps chosen to
        // land one in each distance bucket: 0.02 (Trail), 0.05 (Dirt),
        // 0.08 (Paved), 0.15 (Highway). roads() sorts by id and walks windows(2).
        let xs = [0.0_f32, 0.02, 0.07, 0.15, 0.30];
        let b: Vec<Building> = xs
            .iter()
            .enumerate()
            .map(|(i, &x)| Building {
                id: i as u32,
                x,
                y: 0.0,
                kind: BuildingKind::Residential,
                era: 0,
                faction_id: 0,
                occupants: 0,
                capacity: 0,
            })
            .collect();
        let r = roads(&b);
        assert_eq!(r.len(), 4);
        assert!(matches!(r[0].kind, RoadKind::Trail) && (r[0].width - 0.2).abs() < 1e-6);
        assert!(matches!(r[1].kind, RoadKind::Dirt) && (r[1].width - 0.4).abs() < 1e-6);
        assert!(matches!(r[2].kind, RoadKind::Paved) && (r[2].width - 0.6).abs() < 1e-6);
        assert!(matches!(r[3].kind, RoadKind::Highway) && (r[3].width - 1.0).abs() < 1e-6);
    }
}
