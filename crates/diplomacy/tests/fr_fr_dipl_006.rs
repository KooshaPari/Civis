//! FR-DIPL-006 tests — Espionage detection events.
//!
//! Detected espionage SHALL emit `diplomacy.espionage.detected.v1`.

use civ_diplomacy::{EspionageAction, EspionageConfig, EspionageEngine, SpyNetwork};

/// FR-DIPL-006: detected_emits_event — detection generates EspionageEvent::Detected.
#[test]
fn espionage_detected_emits_event() {
    let config = EspionageConfig {
        base_detection_chance: 1.0, // guaranteed detection
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");

    // Deploy with zero cover so detection chance = 1.0 * 1.0 * risk = risk > 0
    let mut net = SpyNetwork::new(1, 2, 0.5);
    net.cover = 0.0;
    eng.networks.push(net);

    // Execute any action — should always detect
    // detection_chance = 1.0 * (1.0 - 0.0) * 0.3 = 0.3
    // rng = 0.1 < 0.3 => detected
    let result = eng.execute(EspionageAction::GatherIntel, 0, || 0.1);
    assert!(result.is_ok());
    assert!(
        matches!(result.unwrap(), civ_diplomacy::SpyResult::Detected),
        "should be detected"
    );

    // Drain events and verify detection event was emitted
    let events = eng.drain_events();
    assert_eq!(events.len(), 1, "one detection event");
    match &events[0] {
        civ_diplomacy::EspionageEvent::Detected {
            source_faction,
            target_faction,
            action,
            ..
        } => {
            assert_eq!(*source_faction, 1);
            assert_eq!(*target_faction, 2);
            assert_eq!(*action, EspionageAction::GatherIntel);
        }
        other => panic!("expected Detected event, got {other:?}"),
    }
}

/// FR-DIPL-006: High cover prevents detection (no event emitted).
#[test]
fn espionage_high_cover_no_event() {
    let config = EspionageConfig {
        base_detection_chance: 0.50,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");
    eng.deploy(1, 2, 0.5).unwrap();
    // Cover is 1.0 by default; rng=1.0 > detection_chance
    let result = eng.execute(EspionageAction::GatherIntel, 0, || 1.0);
    assert!(result.is_ok());
    let events = eng.drain_events();
    assert!(events.is_empty(), "no detection event when not detected");
}

/// FR-DIPL-006: Multiple detections accumulate events.
#[test]
fn espionage_multiple_detections() {
    let config = EspionageConfig {
        base_detection_chance: 1.0,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");

    // Deploy two networks with zero cover
    for src in [1, 3] {
        let mut net = SpyNetwork::new(src, 2, 0.5);
        net.cover = 0.0;
        eng.networks.push(net);
    }

    let _ = eng.execute(EspionageAction::GatherIntel, 0, || 0.1);
    let _ = eng.execute(EspionageAction::Sabotage, 1, || 0.1);

    let events = eng.drain_events();
    assert_eq!(events.len(), 2, "two detection events");
    assert!(matches!(events[0], civ_diplomacy::EspionageEvent::Detected { .. }));
    assert!(matches!(events[1], civ_diplomacy::EspionageEvent::Detected { .. }));
}

/// FR-DIPL-006: Detection event includes tick information.
#[test]
fn espionage_event_fields_populated() {
    let config = EspionageConfig {
        base_detection_chance: 1.0,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");
    let mut net = SpyNetwork::new(10, 20, 0.5);
    net.cover = 0.0;
    eng.networks.push(net);

    let _ = eng.execute(EspionageAction::Sabotage, 0, || 0.5);
    let events = eng.drain_events();
    assert_eq!(events.len(), 1);
    if let civ_diplomacy::EspionageEvent::Detected {
        source_faction,
        target_faction,
        action,
        tick,
    } = &events[0]
    {
        assert_eq!(*source_faction, 10);
        assert_eq!(*target_faction, 20);
        assert_eq!(*action, EspionageAction::Sabotage);
        assert_eq!(*tick, 0); // tick not tracked per-call
    }
}
