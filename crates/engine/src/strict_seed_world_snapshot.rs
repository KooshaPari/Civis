//! Opt-in snapshot of the initial seed-world ECS archetypes.

use std::collections::BTreeSet;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::engine::{Building, Citizen, MilitaryUnit};
use crate::strict_civilian_snapshot::{
    capture_civilian, unique_json, BoundedJson, CivilianRecord, CivilianSnapshot, MAX_ENTITIES,
    MAX_ENTITY_INDEX, MAX_JSON_BYTES,
};

#[derive(Debug, Clone, Serialize)]
pub struct SeedWorldSnapshot {
    version: u32,
    civilians: Vec<CivilianRecord>,
    standalone: Vec<StandaloneRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWire {
    version: u32,
    civilians: Vec<CivilianRecord>,
    standalone: Vec<StandaloneRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StandaloneRecord {
    entity_bits: u64,
    component: StandaloneComponent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum StandaloneComponent {
    Citizen(Citizen),
    Building(Building),
    Military(MilitaryUnit),
}

impl SeedWorldSnapshot {
    pub fn capture(world: &World) -> Result<Self, String> {
        if world.len() as usize > MAX_ENTITIES {
            return Err("too many entities".into());
        }
        let mut civilians = Vec::new();
        let mut standalone = Vec::new();
        let mut budget = BoundedJson(Vec::new());
        for entity in world.iter() {
            if entity.has::<civ_agents::Civilian>() {
                let record = capture_civilian(entity)?;
                serde_json::to_writer(&mut budget, &record).map_err(|e| e.to_string())?;
                civilians.push(record);
                continue;
            }
            if entity.len() != 1 {
                return Err("unsupported ECS component combination".into());
            }
            let component = if let Some(x) = entity.get::<&Citizen>() {
                StandaloneComponent::Citizen(*x)
            } else if let Some(x) = entity.get::<&Building>() {
                StandaloneComponent::Building(*x)
            } else if let Some(x) = entity.get::<&MilitaryUnit>() {
                StandaloneComponent::Military((*x).clone())
            } else {
                return Err("unsupported standalone ECS component".into());
            };
            let record = StandaloneRecord {
                entity_bits: entity.entity().to_bits().get(),
                component,
            };
            serde_json::to_writer(&mut budget, &record).map_err(|e| e.to_string())?;
            standalone.push(record);
        }
        civilians.sort_by_key(|record| record.entity_bits);
        standalone.sort_by_key(|record| record.entity_bits);
        let snapshot = Self {
            version: 1,
            civilians,
            standalone,
        };
        snapshot.to_json()?;
        Ok(snapshot)
    }

    pub fn to_json(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = BoundedJson(Vec::new());
        serde_json::to_writer(&mut bytes, self).map_err(|e| e.to_string())?;
        Ok(bytes.0)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_JSON_BYTES {
            return Err("snapshot JSON exceeds size limit".into());
        }
        let value = serde_json::from_slice::<unique_json::UniqueValue>(bytes)
            .map_err(|e| e.to_string())?
            .0;
        let wire: SnapshotWire =
            serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        let snapshot = Self {
            version: wire.version,
            civilians: wire.civilians,
            standalone: wire.standalone,
        };
        if snapshot.to_json()? != bytes {
            return Err("unknown or noncanonical snapshot fields".into());
        }
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.civilians.len().saturating_add(self.standalone.len()) > MAX_ENTITIES
        {
            return Err("unsupported snapshot version or size".into());
        }
        CivilianSnapshot {
            version: 1,
            entities: self.civilians.clone(),
        }
        .validate()?;
        let mut ids = BTreeSet::new();
        for bits in self
            .civilians
            .iter()
            .map(|r| r.entity_bits)
            .chain(self.standalone.iter().map(|r| r.entity_bits))
        {
            let entity = Entity::from_bits(bits).ok_or("invalid entity identity")?;
            if entity.id() >= MAX_ENTITY_INDEX || !ids.insert(entity.id()) {
                return Err("duplicate or oversized identity".into());
            }
        }
        Ok(())
    }

    pub fn restore(&self) -> Result<World, String> {
        self.validate()?;
        let mut world = CivilianSnapshot {
            version: 1,
            entities: self.civilians.clone(),
        }
        .restore()?;
        for record in &self.standalone {
            let entity = Entity::from_bits(record.entity_bits).ok_or("invalid entity identity")?;
            match &record.component {
                StandaloneComponent::Citizen(x) => world.spawn_at(entity, (*x,)),
                StandaloneComponent::Building(x) => world.spawn_at(entity, (*x,)),
                StandaloneComponent::Military(x) => world.spawn_at(entity, (x.clone(),)),
            }
        }
        Ok(world)
    }
}
