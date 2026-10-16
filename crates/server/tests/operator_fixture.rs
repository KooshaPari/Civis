//! Supplied-role fixtures for the two guarded inputs used by the playthrough.
//!
//! These dispatch tests prove the planned effect, not authenticated actor
//! entitlement or live mutation. The separate real-server fixture verifies
//! fresh-connection/header behavior and observes the authoritative state.

use civ_server::{
    dispatch_request, error_code, parse_request, DispatchContext, DispatchEffect, DispatchPlan,
    RequestId,
};
use serde_json::{json, Value};

fn context() -> DispatchContext {
    DispatchContext {
        tick: 21,
        population: None,
        snapshot: None,
        tile_probe: None,
        require_role: true,
        speed_multiplier: 0,
        connection_role: None,
        saves_dir: None,
        emergence: None,
        legends: None,
        researched: vec![],
        in_progress_tech: None,
        outcome_fields: None,
        last_tick_ms: 0.0,
        psyche_snapshot: None,
        sentience_events: None,
        religion_state: None,
    }
}

fn dispatch(method: &str, params: Value) -> DispatchPlan {
    let request = parse_request(
        &json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params}).to_string(),
    )
    .expect("valid fixture request");
    dispatch_request(request, context())
}

fn assert_denied(plan: DispatchPlan) {
    assert_eq!(plan.response.id, RequestId::Number(1));
    assert_eq!(
        plan.effect,
        DispatchEffect::None,
        "denial must plan no mutation"
    );
    assert!(
        plan.response.result.is_none(),
        "denial must not include a result"
    );
    let error = plan
        .response
        .error
        .expect("denial must return an RPC error");
    assert_eq!(error.code, error_code::FORBIDDEN);
    assert_eq!(error.code, -32003);
    assert_eq!(error.message, "Forbidden: operator role required");
    assert_eq!(error.data, Some(json!({"required_role":"operator"})));
}

#[test]
fn spawn_missing_or_viewer_role_plans_no_effect() {
    for role in [None, Some("viewer")] {
        let mut params = json!({"x":0.25, "y":0.75, "faction":1});
        if let Some(role) = role {
            params["role"] = json!(role);
        }
        assert_denied(dispatch("sim.spawn_civilian", params));
    }
}

#[test]
fn tick_missing_or_viewer_role_plans_no_effect() {
    for role in [None, Some("viewer")] {
        let mut params = json!({"action":"tick"});
        if let Some(role) = role {
            params["role"] = json!(role);
        }
        assert_denied(dispatch("sim.command", params));
    }
}

#[test]
fn spawn_explicit_operator_input_plans_civilian_spawn() {
    let plan = dispatch(
        "sim.spawn_civilian",
        json!({"x":0.25, "y":0.75, "faction":1, "role":"operator"}),
    );
    assert_eq!(plan.response.id, RequestId::Number(1));
    assert!(plan.response.error.is_none());
    assert_eq!(plan.response.result, Some(json!({"accepted":true})));
    match plan.effect {
        DispatchEffect::SpawnCivilian { x, y, faction, .. } => {
            assert_eq!(x, 0.25);
            assert_eq!(y, 0.75);
            assert_eq!(faction, 1);
        }
        effect => panic!("operator spawn must plan a civilian spawn, got {effect:?}"),
    }
}

#[test]
fn tick_explicit_operator_input_plans_one_tick() {
    let plan = dispatch("sim.command", json!({"action":"tick", "role":"operator"}));
    assert_eq!(plan.response.id, RequestId::Number(1));
    assert!(plan.response.error.is_none());
    assert_eq!(
        plan.response.result,
        Some(json!({"accepted":true, "tick":21}))
    );
    assert_eq!(plan.effect, DispatchEffect::AdvanceTick);
}
