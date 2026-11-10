use civ_agents::{spawn_civilian_at, ActorVisualKind, Alignment};
use civ_engine::{strict_seed_world_snapshot::SeedWorldSnapshot, Simulation};
use rand::SeedableRng;

#[test]
fn seeded_world_and_authored_spawn_roundtrip() {
    let mut sim = Simulation::with_seed(42);
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(87);
    let entity = spawn_civilian_at(
        &mut sim.world,
        900_000,
        Alignment::Faction(2),
        0.42,
        0.58,
        ActorVisualKind::Humanoid,
        &mut rng,
    );
    let before = SeedWorldSnapshot::capture(&sim.world).unwrap();
    let bytes = before.to_json().unwrap();
    let restored = SeedWorldSnapshot::from_json(&bytes)
        .unwrap()
        .restore()
        .unwrap();
    assert!(restored.contains(entity));
    let after = SeedWorldSnapshot::capture(&restored).unwrap();
    assert_eq!(after.to_json().unwrap(), bytes);
}

#[test]
fn unknown_world_component_fails_without_mutation() {
    let mut sim = Simulation::with_seed(42);
    let before = sim.world.len();
    let entity = sim.world.spawn((23_u64,));
    assert!(SeedWorldSnapshot::capture(&sim.world).is_err());
    assert_eq!(sim.world.len(), before + 1);
    assert_eq!(*sim.world.get::<&u64>(entity).unwrap(), 23);
}

#[test]
fn duplicate_cross_kind_identity_is_rejected() {
    let sim = Simulation::with_seed(42);
    let snapshot = SeedWorldSnapshot::capture(&sim.world).unwrap();
    let bytes = snapshot.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let source = format!("\"entity_bits\":{}", value["standalone"][0]["entity_bits"]);
    let duplicate = format!("\"entity_bits\":{}", value["civilians"][0]["entity_bits"]);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains(&source));
    let changed = text.replacen(&source, &duplicate, 1);
    let error = SeedWorldSnapshot::from_json(changed.as_bytes()).unwrap_err();
    assert_eq!(error, "duplicate or oversized identity");
}

#[test]
fn nested_unknown_and_duplicate_json_keys_fail() {
    let sim = Simulation::with_seed(42);
    let snapshot = SeedWorldSnapshot::capture(&sim.world).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&snapshot.to_json().unwrap()).unwrap();
    value["standalone"][0]["unexpected"] = serde_json::json!(true);
    assert!(SeedWorldSnapshot::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(SeedWorldSnapshot::from_json(
        br#"{"version":1,"version":1,"civilians":[],"standalone":[]}"#
    )
    .is_err());
}
