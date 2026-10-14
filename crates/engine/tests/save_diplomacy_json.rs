//! Regression for the tuple-key save failure; this does not certify ECS persistence.
use civ_engine::{CivSaveBundle, DiplomacyKind, Simulation};

#[test]
fn seed42_tick0_diplomacy_archive_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut sim = Simulation::with_seed(42);
    assert_eq!(sim.state.tick, 0);
    let before = dir.path().join("slot-1.civsave.zst");
    CivSaveBundle::save_archive(&before, &sim).expect("pre-diplomacy control saves");
    let pristine = CivSaveBundle::load_archive(&before).expect("control loads");
    assert_eq!(pristine.faction_relations, sim.faction_relations);

    for kind in [
        DiplomacyKind::TradeAgreement,
        DiplomacyKind::Conflict,
        DiplomacyKind::Conflict,
    ] {
        sim.apply_player_diplomacy_action(0, 1, kind)
            .expect("valid pair");
        let path = dir.path().join("slot-2.civsave.zst");
        CivSaveBundle::save_archive(&path, &sim).expect("populated diplomacy saves");
        let restored = CivSaveBundle::load_archive(&path).expect("populated diplomacy loads");
        assert_eq!(restored.state.tick, 0);
        assert_eq!(restored.faction_relations, sim.faction_relations);
        assert_eq!(restored.state.faction_relations, sim.faction_relations);
    }
}
