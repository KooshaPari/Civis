//! Isolated save witness; not a production save format.
use civ_engine::strict_seed_world_snapshot::SeedWorldSnapshot;
use hecs::{AllocatorImage, World};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

const LIMIT: usize = 16 * 1024 * 1024;
const DOMAIN: &[u8] = b"civis-world-save-witness-v1\0";

struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "serialization limit",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut out = Bounded(Vec::new());
    serde_json::to_writer(&mut out, value).map_err(|e| e.to_string())?;
    Ok(out.0)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    version: u32,
    world: Vec<u8>,
    allocator: AllocatorImage,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    payload: Vec<u8>,
    digest: String,
}

fn digest(bytes: &[u8]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(DOMAIN);
    hash.update(bytes);
    hash.finalize().to_hex().to_string()
}

/// Capture the supported ECS subset and its allocator under exclusive world access.
pub fn capture(world: &mut World) -> Result<Vec<u8>, String> {
    let payload = Payload {
        version: 1,
        world: SeedWorldSnapshot::capture(world)?.to_json()?,
        allocator: world.allocator_image().map_err(str::to_owned)?,
    };
    let payload = encode(&payload)?;
    let envelope = Envelope {
        version: 1,
        digest: digest(&payload),
        payload,
    };
    let bytes = encode(&envelope)?;
    Ok(bytes)
}

/// Validate and build a separate world; the caller can swap it in after success.
pub fn restore(bytes: &[u8]) -> Result<World, String> {
    if bytes.len() > LIMIT {
        return Err("envelope too large".into());
    }
    let envelope: Envelope = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if envelope.version != 1
        || encode(&envelope)? != bytes
        || digest(&envelope.payload) != envelope.digest
    {
        return Err("invalid envelope".into());
    }
    let payload: Payload = serde_json::from_slice(&envelope.payload).map_err(|e| e.to_string())?;
    if payload.version != 1 || encode(&payload)? != envelope.payload {
        return Err("invalid payload".into());
    }
    let mut world = SeedWorldSnapshot::from_json(&payload.world)?.restore()?;
    world
        .restore_allocator_image(&payload.allocator)
        .map_err(str::to_owned)?;
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_engine::Simulation;

    #[test]
    fn restored_world_keeps_components_and_future_ids() {
        let mut source = Simulation::with_seed(42).world;
        let ids: Vec<_> = source.iter().map(|e| e.entity()).take(4).collect();
        source.despawn(ids[1]).unwrap();
        source.despawn(ids[3]).unwrap();
        let reserved: Vec<_> = (0..3).map(|_| source.reserve_entity()).collect();
        let bytes = capture(&mut source).unwrap();
        let mut restored = restore(&bytes).unwrap();
        assert_eq!(
            SeedWorldSnapshot::capture(&source)
                .unwrap()
                .to_json()
                .unwrap(),
            SeedWorldSnapshot::capture(&restored)
                .unwrap()
                .to_json()
                .unwrap()
        );
        assert_eq!(source.reserve_entity(), restored.reserve_entity());
        source.flush();
        restored.flush();
        for id in reserved {
            assert!(source.contains(id) && restored.contains(id));
        }
        assert_eq!(source.spawn(()), restored.spawn(()));
    }

    #[test]
    fn rejects_tampering_and_preserves_caller_world() {
        let mut world = Simulation::with_seed(42).world;
        let before = SeedWorldSnapshot::capture(&world)
            .unwrap()
            .to_json()
            .unwrap();
        let bytes = capture(&mut world).unwrap();
        let mut envelope: Envelope = serde_json::from_slice(&bytes).unwrap();
        envelope.payload[0] ^= 1;
        assert!(restore(&serde_json::to_vec(&envelope).unwrap()).is_err());
        assert_eq!(
            before,
            SeedWorldSnapshot::capture(&world)
                .unwrap()
                .to_json()
                .unwrap()
        );
        let mut envelope: Envelope = serde_json::from_slice(&bytes).unwrap();
        envelope.version = 2;
        assert!(restore(&serde_json::to_vec(&envelope).unwrap()).is_err());
    }

    #[test]
    fn rejects_integrity_valid_but_invalid_allocator() {
        let mut world = Simulation::with_seed(42).world;
        let before = SeedWorldSnapshot::capture(&world)
            .unwrap()
            .to_json()
            .unwrap();
        let bytes = capture(&mut world).unwrap();
        let mut envelope: Envelope = serde_json::from_slice(&bytes).unwrap();
        let mut payload: serde_json::Value = serde_json::from_slice(&envelope.payload).unwrap();
        payload["allocator"]["generations"][0] = 0.into();
        // Use the typed encoder to preserve the canonical field ordering.
        let payload: Payload = serde_json::from_value(payload).unwrap();
        envelope.payload = serde_json::to_vec(&payload).unwrap();
        envelope.digest = digest(&envelope.payload);
        assert!(restore(&serde_json::to_vec(&envelope).unwrap()).is_err());
        assert_eq!(
            before,
            SeedWorldSnapshot::capture(&world)
                .unwrap()
                .to_json()
                .unwrap()
        );
    }

    #[test]
    fn rejects_oversized_reservations_before_allocator_copy() {
        let mut world = Simulation::with_seed(42).world;
        let _reservations = world.reserve_entities(1_000_001).count();
        assert!(capture(&mut world).is_err());
    }

    #[test]
    fn rejects_resealed_invalid_reservation_cursor() {
        let mut world = Simulation::with_seed(42).world;
        let bytes = capture(&mut world).unwrap();
        let mut envelope: Envelope = serde_json::from_slice(&bytes).unwrap();
        let mut payload: serde_json::Value = serde_json::from_slice(&envelope.payload).unwrap();
        payload["allocator"]["cursor"] = (-1_000_001i64).into();
        let payload: Payload = serde_json::from_value(payload).unwrap();
        envelope.payload = serde_json::to_vec(&payload).unwrap();
        envelope.digest = digest(&envelope.payload);
        assert!(restore(&serde_json::to_vec(&envelope).unwrap()).is_err());
    }

    #[test]
    fn seeded_tick_replays_after_world_restore() {
        let mut source = Simulation::with_seed(42);
        let mut replay = Simulation::with_seed(42);
        let bytes = capture(&mut source.world).unwrap();
        replay.world = restore(&bytes).unwrap();
        source.tick();
        replay.tick();
        assert_eq!(
            SeedWorldSnapshot::capture(&source.world)
                .unwrap()
                .to_json()
                .unwrap(),
            SeedWorldSnapshot::capture(&replay.world)
                .unwrap()
                .to_json()
                .unwrap()
        );
    }
}
