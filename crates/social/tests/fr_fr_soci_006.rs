//! FR-SOCI-006 — Health crisis event and labor productivity reduction.
//!
//! A health crisis SHALL emit `social.health.crisis.v1` and reduce
//! labor productivity.

use civ_social::{compute_health, HealthConfig, HealthInputs, MAX_HEALTH_BP};
use civ_social::events::EventType;

#[test]
fn crisis_emits_event_reduces_labor() {
    let inputs = HealthInputs {
        food_bp: 0,
        water_bp: 0,
        medical_bp: 0,
    };
    let cfg = HealthConfig::default();

    // First tick: crisis starts => event emitted
    let result = compute_health(inputs, &cfg, false, 10);
    assert!(result.crisis);
    assert!(result.crisis_event.is_some());
    let event = result.crisis_event.unwrap();
    assert_eq!(event.event_type, EventType::HealthCrisis);
    assert_eq!(event.tick, 10);

    // Labor productivity is reduced
    assert_eq!(result.labor_factor_bp, cfg.crisis_labor_factor_bp);
    assert!(result.labor_factor_bp < MAX_HEALTH_BP);
}

#[test]
fn no_duplicate_events_while_crisis_ongoing() {
    let inputs = HealthInputs {
        food_bp: 0,
        water_bp: 0,
        medical_bp: 0,
    };
    let cfg = HealthConfig::default();

    // First tick: crisis starts
    let result = compute_health(inputs, &cfg, false, 10);
    assert!(result.crisis_event.is_some());

    // Second tick: crisis already active => no new event
    let result = compute_health(inputs, &cfg, true, 11);
    assert!(result.crisis);
    assert!(result.crisis_event.is_none());
    assert_eq!(result.labor_factor_bp, cfg.crisis_labor_factor_bp);
}

#[test]
fn labor_restored_when_crisis_ends() {
    let inputs = HealthInputs {
        food_bp: 1000,
        water_bp: 1000,
        medical_bp: 1000,
    };
    let cfg = HealthConfig::default();

    // Crisis was active but now index is above threshold
    let result = compute_health(inputs, &cfg, true, 20);
    assert!(!result.crisis);
    assert_eq!(result.labor_factor_bp, MAX_HEALTH_BP);
}

#[test]
fn custom_crisis_labor_factor() {
    let inputs = HealthInputs {
        food_bp: 0,
        water_bp: 0,
        medical_bp: 0,
    };
    let cfg = HealthConfig {
        crisis_labor_factor_bp: 300, // 30% productivity
        ..Default::default()
    };

    let result = compute_health(inputs, &cfg, false, 1);
    assert!(result.crisis);
    assert_eq!(result.labor_factor_bp, 300);
}

#[test]
fn event_serializes() {
    use civ_social::Event;
    let event = Event {
        event_type: EventType::HealthCrisis,
        tick: 42,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("HealthCrisis"));
    assert!(json.contains("42"));
}
