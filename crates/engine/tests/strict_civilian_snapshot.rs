use civ_agents::{Alignment, Civilian, Position3d, Velocity};
use civ_engine::strict_civilian_snapshot::CivilianSnapshot;
use civ_voxel::WorldCoord;
use hecs::World;

fn civilian() -> (Civilian, Position3d, Velocity) {
    (
        Civilian {
            id: 42,
            alignment: Alignment::Faction(2),
            age: 27,
        },
        Position3d {
            coord: WorldCoord {
                x: 10,
                y: 20,
                z: 30,
            },
        },
        Velocity { dx: 0.25, dy: -0.5 },
    )
}

#[test]
fn snapshot_preserves_entity_and_components() {
    let mut world = World::new();
    let entity = world.spawn(civilian());
    let snapshot = CivilianSnapshot::capture(&world).unwrap();
    let json = snapshot.to_json().unwrap();
    let decoded = CivilianSnapshot::from_json(&json).unwrap();
    let restored = decoded.restore().unwrap();
    assert_eq!(*restored.get::<&Civilian>(entity).unwrap(), civilian().0);
    assert_eq!(*restored.get::<&Position3d>(entity).unwrap(), civilian().1);
    assert_eq!(*restored.get::<&Velocity>(entity).unwrap(), civilian().2);
}

#[test]
fn unknown_component_is_rejected_without_mutating_source() {
    let mut world = World::new();
    let entity = world.spawn((civilian().0, civilian().1, civilian().2, 99_u64));
    assert!(CivilianSnapshot::capture(&world).is_err());
    assert_eq!(*world.get::<&u64>(entity).unwrap(), 99);
}

#[test]
fn external_alignment_and_duplicate_identity_are_rejected() {
    let mut world = World::new();
    let mut components = civilian();
    components.0.alignment = Alignment::OtherEntity(19);
    world.spawn(components);
    assert!(CivilianSnapshot::capture(&world).is_err());
    let mut world = World::new();
    world.spawn(civilian());
    let mut snapshot = CivilianSnapshot::capture(&world).unwrap();
    snapshot.entities.push(snapshot.entities[0].clone());
    assert!(snapshot.restore().is_err());
}

#[test]
fn empty_and_reused_entity_identity_roundtrip() {
    let mut world = World::new();
    assert_eq!(
        CivilianSnapshot::capture(&world)
            .unwrap()
            .restore()
            .unwrap()
            .len(),
        0
    );
    let old = world.spawn(civilian());
    world.despawn(old).unwrap();
    let replacement = world.spawn(civilian());
    assert_ne!(old, replacement);
    let restored = CivilianSnapshot::capture(&world)
        .unwrap()
        .restore()
        .unwrap();
    assert!(restored.get::<&Civilian>(replacement).is_ok());
    assert!(restored.get::<&Civilian>(old).is_err());
}

#[test]
fn oversized_identity_and_unknown_schema_are_rejected() {
    let mut world = World::new();
    world.spawn(civilian());
    let mut snapshot = CivilianSnapshot::capture(&world).unwrap();
    snapshot.entities[0].entity_bits = (1_u64 << 32) | 1_000_000;
    assert!(snapshot.restore().is_err());
    snapshot.version = 2;
    assert!(snapshot.restore().is_err());
    assert!(CivilianSnapshot::from_json(br#"{"version":1,"entities":[],"extra":true}"#).is_err());
}

#[test]
fn non_finite_velocity_is_rejected() {
    let mut world = World::new();
    let mut components = civilian();
    components.2.dx = f32::NAN;
    world.spawn(components);
    assert!(CivilianSnapshot::capture(&world).is_err());
}

#[test]
fn non_finite_optional_component_is_rejected() {
    let mut world = World::new();
    let components = civilian();
    let needs = civ_agents::Needs {
        food: f32::NAN,
        shelter: 0.0,
        safety: 0.0,
        belonging: 0.0,
        rest: 0.0,
        health: 0.0,
    };
    world.spawn((components.0, components.1, components.2, needs));
    assert!(CivilianSnapshot::capture(&world).is_err());
}

#[test]
fn authored_civilian_bundle_survives_snapshot() {
    use civ_agents::{spawn_civilian_at, ActorVisualKind};
    use rand::SeedableRng;
    let mut world = World::new();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
    let entity = spawn_civilian_at(
        &mut world,
        1000,
        Alignment::Faction(2),
        0.42,
        0.58,
        ActorVisualKind::Humanoid,
        &mut rng,
    );
    let snapshot = CivilianSnapshot::capture(&world).unwrap();
    let restored = snapshot.restore().unwrap();
    assert!(restored.get::<&Civilian>(entity).is_ok());
    assert_eq!(
        serde_json::to_value(&snapshot).unwrap(),
        serde_json::to_value(CivilianSnapshot::capture(&restored).unwrap()).unwrap()
    );
}

#[test]
fn nested_unknown_and_large_payload_are_rejected() {
    let mut world = World::new();
    world.spawn(civilian());
    let snapshot = CivilianSnapshot::capture(&world).unwrap();
    let mut json = serde_json::to_value(snapshot).unwrap();
    json["entities"][0]["civilian"]["unexpected"] = serde_json::json!(true);
    assert!(CivilianSnapshot::from_json(&serde_json::to_vec(&json).unwrap()).is_err());
    assert!(CivilianSnapshot::from_json(&vec![b' '; 16 * 1024 * 1024 + 1]).is_err());
    assert!(CivilianSnapshot::from_json(br#"{"version":1,"version":1,"entities":[]}"#).is_err());
    let text = String::from_utf8(snapshot_json_for_duplicate()).unwrap();
    let duplicated = text.replace("\"id\":42", "\"id\":42,\"id\":42");
    assert_ne!(text, duplicated);
    assert!(CivilianSnapshot::from_json(duplicated.as_bytes()).is_err());
}

fn snapshot_json_for_duplicate() -> Vec<u8> {
    let mut world = World::new();
    world.spawn(civilian());
    CivilianSnapshot::capture(&world)
        .unwrap()
        .to_json()
        .unwrap()
}

#[test]
fn duplicate_index_with_different_generations_and_unknown_entity_fail() {
    let mut world = World::new();
    world.spawn(civilian());
    let mut snapshot = CivilianSnapshot::capture(&world).unwrap();
    let mut duplicate = snapshot.entities[0].clone();
    duplicate.entity_bits += 1_u64 << 32;
    duplicate.civilian.id += 1;
    snapshot.entities.push(duplicate);
    assert!(snapshot.restore().is_err());
    let mut unsupported = World::new();
    unsupported.spawn((123_u64,));
    assert!(CivilianSnapshot::capture(&unsupported).is_err());
}

#[test]
fn oversized_dna_is_rejected_before_restore() {
    let mut world = World::new();
    world.spawn(civilian());
    let mut snapshot = CivilianSnapshot::capture(&world).unwrap();
    snapshot.entities[0].dna = Some(civ_genetics::Dna(vec![0; 65_537]));
    assert!(snapshot.restore().is_err());
}

#[test]
fn oversized_encoded_snapshot_is_rejected() {
    let mut world = World::new();
    world.spawn(civilian());
    let mut snapshot = CivilianSnapshot::capture(&world).unwrap();
    snapshot.entities.clear();
    for id in 0..300_u64 {
        let mut record = CivilianSnapshot::capture(&world)
            .unwrap()
            .entities
            .remove(0);
        record.entity_bits = (1_u64 << 32) | id;
        record.civilian.id = id;
        record.dna = Some(civ_genetics::Dna(vec![255; 65_536]));
        snapshot.entities.push(record);
    }
    assert!(snapshot.to_json().is_err());
}

#[test]
fn oversized_capture_is_rejected_without_mutating_world() {
    let mut world = World::new();
    for id in 0..70_u64 {
        let mut value = civilian();
        value.0.id = id;
        world.spawn((
            value.0,
            value.1,
            value.2,
            civ_genetics::Dna(vec![255; 65_536]),
        ));
    }
    assert!(CivilianSnapshot::capture(&world).is_err());
    assert_eq!(world.len(), 70);
}

#[test]
fn mixed_supported_archetypes_roundtrip() {
    let mut world = World::new();
    let first = world.spawn(civilian());
    let mut second_components = civilian();
    second_components.0.id = 43;
    let second = world.spawn((
        second_components.0,
        second_components.1,
        second_components.2,
        civ_genetics::Dna(vec![1, 2, 3]),
    ));
    let snapshot = CivilianSnapshot::capture(&world).unwrap();
    let restored = CivilianSnapshot::from_json(&serde_json::to_vec(&snapshot).unwrap())
        .unwrap()
        .restore()
        .unwrap();
    assert!(restored.get::<&Civilian>(first).is_ok());
    assert_eq!(
        restored.get::<&civ_genetics::Dna>(second).unwrap().0,
        vec![1, 2, 3]
    );
}
