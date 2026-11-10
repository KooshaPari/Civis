//! Strict, opt-in snapshot of a bounded civilian ECS subset.

use std::{
    any::TypeId,
    collections::BTreeSet,
    io::{self, Write},
};

use civ_agents::{
    ActorVisual, Alignment, Civilian, LodTier, Needs, Position3d, Tools, Velocity, Wardrobe,
};
use civ_genetics::Dna;
use civ_needs::{Health, Needs as LifeNeeds};
use hecs::{Entity, EntityBuilder, World};
use serde::{Deserialize, Serialize};

use crate::engine::Citizen;

pub(crate) mod unique_json {
    use std::fmt;

    use serde::de::{Error, MapAccess, SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer};
    use serde_json::{Map, Number, Value};

    pub struct UniqueValue(pub Value);

    impl<'de> Deserialize<'de> for UniqueValue {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct UniqueVisitor;
            impl<'de> Visitor<'de> for UniqueVisitor {
                type Value = UniqueValue;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("JSON without duplicate keys")
                }
                fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::Bool(v)))
                }
                fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::Number(v.into())))
                }
                fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::Number(v.into())))
                }
                fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::Number(
                        Number::from_f64(v).ok_or_else(|| E::custom("invalid number"))?,
                    )))
                }
                fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::String(v.to_owned())))
                }
                fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::String(v)))
                }
                fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::Null))
                }
                fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
                    Ok(UniqueValue(Value::Null))
                }
                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                    let mut values = Vec::new();
                    while let Some(value) = seq.next_element::<UniqueValue>()? {
                        values.push(value.0);
                    }
                    Ok(UniqueValue(Value::Array(values)))
                }
                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut values = Map::new();
                    while let Some(key) = map.next_key::<String>()? {
                        if values.contains_key(&key) {
                            return Err(A::Error::custom("duplicate JSON key"));
                        }
                        values.insert(key, map.next_value::<UniqueValue>()?.0);
                    }
                    Ok(UniqueValue(Value::Object(values)))
                }
            }
            deserializer.deserialize_any(UniqueVisitor)
        }
    }
}

pub(crate) const MAX_ENTITIES: usize = 100_000;
pub(crate) const MAX_ENTITY_INDEX: u32 = 1_000_000;
pub(crate) const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;
const MAX_DNA_BYTES: usize = 65_536;

pub(crate) struct BoundedJson(pub(crate) Vec<u8>);

impl Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_JSON_BYTES.saturating_sub(self.0.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "snapshot JSON exceeds size limit",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CivilianSnapshot {
    pub version: u32,
    pub entities: Vec<CivilianRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWire {
    version: u32,
    entities: Vec<CivilianRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CivilianRecord {
    pub entity_bits: u64,
    pub civilian: Civilian,
    pub position: Position3d,
    pub velocity: Velocity,
    pub wardrobe: Option<Wardrobe>,
    pub tools: Option<Tools>,
    pub needs: Option<Needs>,
    pub lod: Option<LodTier>,
    pub dna: Option<Dna>,
    pub life_needs: Option<LifeNeeds>,
    pub health: Option<Health>,
    pub visual: Option<ActorVisual>,
    pub citizen: Option<Citizen>,
}

fn supported(id: TypeId) -> bool {
    [
        TypeId::of::<Civilian>(),
        TypeId::of::<Position3d>(),
        TypeId::of::<Velocity>(),
        TypeId::of::<Wardrobe>(),
        TypeId::of::<Tools>(),
        TypeId::of::<Needs>(),
        TypeId::of::<LodTier>(),
        TypeId::of::<Dna>(),
        TypeId::of::<LifeNeeds>(),
        TypeId::of::<Health>(),
        TypeId::of::<ActorVisual>(),
        TypeId::of::<Citizen>(),
    ]
    .contains(&id)
}

pub(crate) fn capture_civilian(entity: hecs::EntityRef<'_>) -> Result<CivilianRecord, String> {
    if entity.component_types().any(|id| !supported(id)) {
        return Err("unsupported ECS component".into());
    }
    let civilian = (*entity.get::<&Civilian>().ok_or("missing civilian")?).clone();
    let position = *entity.get::<&Position3d>().ok_or("missing position")?;
    let velocity = *entity.get::<&Velocity>().ok_or("missing velocity")?;
    if entity
        .get::<&Dna>()
        .is_some_and(|dna| dna.0.len() > MAX_DNA_BYTES)
    {
        return Err("DNA exceeds size limit".into());
    }
    Ok(CivilianRecord {
        entity_bits: entity.entity().to_bits().get(),
        civilian,
        position,
        velocity,
        wardrobe: entity.get::<&Wardrobe>().map(|x| *x),
        tools: entity.get::<&Tools>().map(|x| *x),
        needs: entity.get::<&Needs>().map(|x| *x),
        lod: entity.get::<&LodTier>().map(|x| *x),
        dna: entity.get::<&Dna>().map(|x| (*x).clone()),
        life_needs: entity.get::<&LifeNeeds>().map(|x| *x),
        health: entity.get::<&Health>().map(|x| *x),
        visual: entity.get::<&ActorVisual>().map(|x| *x),
        citizen: entity.get::<&Citizen>().map(|x| *x),
    })
}

impl CivilianSnapshot {
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
            entities: wire.entities,
        };
        if snapshot.to_json()? != bytes {
            return Err("snapshot contains unknown or noncanonical fields".into());
        }
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn capture(world: &World) -> Result<Self, String> {
        if world.len() as usize > MAX_ENTITIES {
            return Err("too many entities".into());
        }
        let mut entities = Vec::new();
        let mut budget = BoundedJson(Vec::new());
        for entity in world.iter() {
            let record = capture_civilian(entity)?;
            serde_json::to_writer(&mut budget, &record).map_err(|e| e.to_string())?;
            entities.push(record);
        }
        entities.sort_by_key(|entry| entry.entity_bits);
        let snapshot = Self {
            version: 1,
            entities,
        };
        snapshot.to_json()?;
        Ok(snapshot)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.entities.len() > MAX_ENTITIES {
            return Err("unsupported snapshot version or size".into());
        }
        let mut ids = BTreeSet::new();
        let mut civilian_ids = BTreeSet::new();
        for entry in &self.entities {
            let entity = Entity::from_bits(entry.entity_bits).ok_or("invalid entity identity")?;
            if entity.id() >= MAX_ENTITY_INDEX
                || !ids.insert(entity.id())
                || !civilian_ids.insert(entry.civilian.id)
            {
                return Err("duplicate or oversized identity".into());
            }
            if matches!(entry.civilian.alignment, Alignment::OtherEntity(_)) {
                return Err("external entity alignment unsupported".into());
            }
            if entry
                .dna
                .as_ref()
                .is_some_and(|dna| dna.0.len() > MAX_DNA_BYTES)
            {
                return Err("DNA exceeds size limit".into());
            }
            if !entry.velocity.dx.is_finite() || !entry.velocity.dy.is_finite() {
                return Err("non-finite velocity".into());
            }
            if let Some(x) = entry.needs {
                if [x.food, x.shelter, x.safety, x.belonging, x.rest, x.health]
                    .iter()
                    .any(|v| !v.is_finite())
                {
                    return Err("non-finite needs".into());
                }
            }
            if let Some(x) = entry.life_needs {
                if [x.food, x.water, x.rest, x.safety, x.social, x.health]
                    .iter()
                    .any(|v| !v.is_finite())
                {
                    return Err("non-finite life needs".into());
                }
            }
            if let Some(x) = entry.health {
                if !x.integrity.is_finite() || !x.morale.is_finite() {
                    return Err("non-finite health".into());
                }
            }
        }
        Ok(())
    }

    pub fn restore(&self) -> Result<World, String> {
        self.validate()?;
        let mut world = World::new();
        for entry in &self.entities {
            let mut builder = EntityBuilder::new();
            builder
                .add(entry.civilian.clone())
                .add(entry.position)
                .add(entry.velocity);
            if let Some(x) = entry.wardrobe {
                builder.add(x);
            }
            if let Some(x) = entry.tools {
                builder.add(x);
            }
            if let Some(x) = entry.needs {
                builder.add(x);
            }
            if let Some(x) = entry.lod {
                builder.add(x);
            }
            if let Some(x) = &entry.dna {
                builder.add(x.clone());
            }
            if let Some(x) = entry.life_needs {
                builder.add(x);
            }
            if let Some(x) = entry.health {
                builder.add(x);
            }
            if let Some(x) = entry.visual {
                builder.add(x);
            }
            if let Some(x) = entry.citizen {
                builder.add(x);
            }
            world.spawn_at(
                Entity::from_bits(entry.entity_bits).ok_or("invalid entity identity")?,
                builder.build(),
            );
        }
        Ok(world)
    }
}
