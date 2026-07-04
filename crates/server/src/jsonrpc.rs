//! JSON-RPC 2.0 request/response types for the CIV-0200 WebSocket protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;

use crate::saves::{list_saves, parse_slot_name_params};

/// JSON-RPC 2.0 version string required on every message.
pub const JSONRPC_VERSION: &str = "2.0";

/// Standard JSON-RPC error codes (https://www.jsonrpc.org/specification#error_object).
pub mod error_code {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Caller lacks permission for the requested action (CIV-0200 application range).
    pub const FORBIDDEN: i32 = -32003;
}

/// Role required for privileged `sim.command` actions when role enforcement is enabled.
pub const OPERATOR_ROLE: &str = "operator";

/// Supported JSON-RPC methods (CIV-0200 stub surface).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonRpcMethod {
    /// Server liveness / tick probe (`health`).
    Health,
    /// Simulation command intake (`sim.command`).
    SimCommand,
    /// Simulation status query (`sim.status`).
    SimStatus,
    /// Full simulation snapshot (`sim.snapshot`).
    SimSnapshot,
    /// Persist the in-memory replay log (`sim.save_replay`).
    SimSaveReplay,
    /// Replace the bridge simulation from a `.civreplay` file (`sim.load_replay`).
    SimLoadReplay,
    /// Replace the bridge simulation with a fresh seeded instance (`sim.reset`).
    SimReset,
    /// Load a named preset scenario and start a fresh simulation (`sim.load_scenario`).
    SimLoadScenario,
    /// Update `Simulation::economy_policy` on the bridge (`sim.set_policy`).
    SimSetPolicy,
    /// Simulation tick speed multiplier (`sim.set_speed`).
    SimSetSpeed,
    /// Read simulation tick speed multiplier (`sim.get_speed`).
    SimGetSpeed,
    /// Spawn one civilian at normalized map coords (`sim.spawn_civilian`, FR-CIV-UX-002).
    SimSpawnCivilian,
    /// Spawn civilian, vehicle, or airport (`sim.spawn_entity`, FR-CIV-UX-006).
    SimSpawnEntity,
    /// Write one voxel (`sim.place_voxel`, FR-CIV-UX-003).
    SimPlaceVoxel,
    /// Tactical voxel damage (`sim.damage`, FR-CIV-TACTICS / P-U1).
    SimDamage,
    /// Persist simulation to a production slot (`save.slot`, CIV-1000 §13).
    SaveSlot,
    /// Load simulation from a production slot (`save.load`, CIV-1000 §13).
    LoadSlot,
    /// List saves in the bridge `saves/` directory (`save.list`, CIV-1000 §13).
    SaveList,
    /// Read the latest civ-emergence-metrics sample
    /// (`sim.emergence`, stacked on PR #350; FR dashboard).
    SimEmergence,
    /// Inspect terrain + faction at a tile coordinate.
    /// (`sim.inspect_tile`, FR tile-inspector).
    SimInspectTile,
    /// Client-initiated diplomacy action (propose_treaty / declare_war / offer_trade). (sim.diplomacy_action, FR-CIV-CLIENT-006).
    SimDiplomacyAction,
    /// Queue a research tech on the simulation (`sim.queue_research`, FR-CIV-SERVER-003).
    SimQueueResearch,
    /// Read the current tech research state (`sim.tech_state`, FR-CIV-SERVER-003).
    SimTechState,
    /// Opt-in tick broadcast filter (`sim.subscribe`, CIV-0200).
    SimGodAction,
    SimPerf,
    SimSubscribe,
    /// Clear per-connection tick broadcast filter (`sim.unsubscribe`).
    SimUnsubscribe,
    /// Replace per-connection tick broadcast filter `(`sim.update_subscription`).`
    SimUpdateSubscription,
    /// Query current game outcome state (`sim.outcome`, FR-CIV-GAME-001).
    SimOutcome,
}

impl JsonRpcMethod {
    /// Wire name for this method.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Health => "health",
            Self::SimCommand => "sim.command",
            Self::SimStatus => "sim.status",
            Self::SimSnapshot => "sim.snapshot",
            Self::SimSaveReplay => "sim.save_replay",
            Self::SimLoadReplay => "sim.load_replay",
            Self::SimReset => "sim.reset",
            Self::SimLoadScenario => "sim.load_scenario",
            Self::SimSetPolicy => "sim.set_policy",
            Self::SimSetSpeed => "sim.set_speed",
            Self::SimGetSpeed => "sim.get_speed",
            Self::SimSpawnCivilian => "sim.spawn_civilian",
            Self::SimSpawnEntity => "sim.spawn_entity",
            Self::SimPlaceVoxel => "sim.place_voxel",
            Self::SimDamage => "sim.damage",
            Self::SaveSlot => "save.slot",
            Self::LoadSlot => "save.load",
            Self::SaveList => "save.list",
            Self::SimEmergence => "sim.emergence",
            Self::SimInspectTile => "sim.inspect_tile",
            Self::SimGodAction => "sim.god_action",
            Self::SimPerf => "sim.perf",
            Self::SimDiplomacyAction => "sim.diplomacy_action",
            Self::SimQueueResearch => "sim.queue_research",
            Self::SimTechState => "sim.tech_state",
            Self::SimSubscribe => "sim.subscribe",
            Self::SimUnsubscribe => "sim.unsubscribe",
            Self::SimUpdateSubscription => "sim.update_subscription",
            Self::SimOutcome => "sim.outcome",
        }
    }

    /// Parse a method name from the wire.
    pub fn parse_name(name: &str) -> Option<Self> {
        match name {
            "health" => Some(Self::Health),
            "sim.command" => Some(Self::SimCommand),
            "sim.status" => Some(Self::SimStatus),
            "sim.snapshot" => Some(Self::SimSnapshot),
            "sim.save_replay" => Some(Self::SimSaveReplay),
            "sim.load_replay" => Some(Self::SimLoadReplay),
            "sim.reset" => Some(Self::SimReset),
            "sim.load_scenario" => Some(Self::SimLoadScenario),
            "sim.set_policy" => Some(Self::SimSetPolicy),
            "sim.set_speed" => Some(Self::SimSetSpeed),
            "sim.get_speed" => Some(Self::SimGetSpeed),
            "sim.spawn_civilian" => Some(Self::SimSpawnCivilian),
            "sim.spawn_entity" => Some(Self::SimSpawnEntity),
            "sim.place_voxel" => Some(Self::SimPlaceVoxel),
            "sim.damage" => Some(Self::SimDamage),
            "save.slot" => Some(Self::SaveSlot),
            "save.load" => Some(Self::LoadSlot),
            "save.list" => Some(Self::SaveList),
            "sim.emergence" => Some(Self::SimEmergence),
            "sim.inspect_tile" => Some(Self::SimInspectTile),
            "sim.god_action" => Some(Self::SimGodAction),
            "sim.perf" => Some(Self::SimPerf),
            "sim.diplomacy_action" => Some(Self::SimDiplomacyAction),
            "sim.queue_research" => Some(Self::SimQueueResearch),
            "sim.tech_state" => Some(Self::SimTechState),
            "sim.subscribe" => Some(Self::SimSubscribe),
            "sim.unsubscribe" => Some(Self::SimUnsubscribe),
            "sim.update_subscription" => Some(Self::SimUpdateSubscription),
            "sim.outcome" => Some(Self::SimOutcome),
            _ => None,
        }
    }
}

/// JSON-RPC request `id` (string, number, or null per spec).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// Numeric id.
    Number(i64),
    /// String id.
    String(String),
    /// Explicit null id (notifications; rejected for calls in this scaffold).
    Null,
}

/// A validated JSON-RPC 2.0 request (single object, not a batch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcRequest {
    /// Always `"2.0"`.
    pub jsonrpc: String,
    /// Correlates the response.
    pub id: RequestId,
    /// Dispatched method.
    pub method: JsonRpcMethod,
    /// Optional method parameters.
    pub params: Option<Value>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// JSON-RPC error code.
    pub code: i32,
    /// Short description.
    pub message: String,
    /// Optional structured detail.
    pub data: Option<Value>,
}

/// A JSON-RPC 2.0 response (success or error, never both).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Always `"2.0"`.
    pub jsonrpc: String,
    /// Echoes the request id.
    pub id: RequestId,
    /// Present on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Present on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Build a success response.
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Build an error response.
    pub fn failure(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// Errors produced while parsing or validating an inbound request.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum JsonRpcParseError {
    /// Body is not valid JSON.
    #[error("Parse error")]
    Parse,
    /// Request object failed structural validation.
    #[error("{message}")]
    InvalidRequest {
        /// Human-readable reason.
        message: &'static str,
    },
    /// `method` is not registered.
    #[error("Method not found: {method}")]
    MethodNotFound {
        /// Wire method name.
        method: String,
    },
}

impl JsonRpcParseError {
    /// JSON-RPC error code for this failure.
    pub const fn code(&self) -> i32 {
        match self {
            Self::Parse => error_code::PARSE_ERROR,
            Self::InvalidRequest { .. } => error_code::INVALID_REQUEST,
            Self::MethodNotFound { .. } => error_code::METHOD_NOT_FOUND,
        }
    }

    /// Short error message for the wire.
    pub fn message(&self) -> String {
        match self {
            Self::Parse => "Parse error".to_owned(),
            Self::InvalidRequest { message } => (*message).to_owned(),
            Self::MethodNotFound { method } => format!("Method not found: {method}"),
        }
    }

    /// Convert to a JSON-RPC error object.
    pub fn into_error(self) -> JsonRpcError {
        JsonRpcError {
            code: self.code(),
            message: self.message(),
            data: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

/// Parse and validate a single JSON-RPC 2.0 request from a WebSocket text frame.
pub fn parse_request(raw: &str) -> Result<JsonRpcRequest, JsonRpcParseError> {
    let value: Value = serde_json::from_str(raw).map_err(|_| JsonRpcParseError::Parse)?;

    if value.is_array() {
        return Err(JsonRpcParseError::InvalidRequest {
            message: "Batch requests are not supported",
        });
    }

    let raw: RawRequest = serde_json::from_value(value).map_err(|_| JsonRpcParseError::Parse)?;

    let jsonrpc = raw.jsonrpc.as_deref().unwrap_or_default();
    if jsonrpc != JSONRPC_VERSION {
        return Err(JsonRpcParseError::InvalidRequest {
            message: "Invalid Request",
        });
    }

    let id = raw
        .id
        .ok_or(JsonRpcParseError::InvalidRequest {
            message: "Invalid Request",
        })
        .and_then(parse_request_id)?;

    let method_name = raw.method.ok_or(JsonRpcParseError::InvalidRequest {
        message: "Invalid Request",
    })?;

    let method =
        JsonRpcMethod::parse_name(&method_name).ok_or(JsonRpcParseError::MethodNotFound {
            method: method_name,
        })?;

    Ok(JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_owned(),
        id,
        method,
        params: raw.params,
    })
}

/// Recognized `sim.command` actions (minimal CIV-0200 surface).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimCommandAction {
    /// No-op command intake (stub).
    Noop,
    /// Advance the simulation by one tick.
    Tick,
}

/// Read `role` from JSON-RPC params (connect / first-message stub).
pub fn parse_role_param(params: Option<&Value>) -> Option<String> {
    params
        .and_then(|p| p.get("role"))
        .and_then(|r| r.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// Whether operator-only RPCs are permitted under optional role enforcement.
pub fn role_allows_operator(
    require_role: bool,
    params: Option<&Value>,
    connection_role: Option<&str>,
) -> bool {
    if !require_role {
        return true;
    }
    let param_role = parse_role_param(params);
    let effective = param_role.as_deref().or(connection_role);
    effective == Some(OPERATOR_ROLE)
}

/// Whether `sim.command` tick is permitted under optional role enforcement.
pub fn role_allows_operator_tick(
    require_role: bool,
    params: Option<&Value>,
    connection_role: Option<&str>,
) -> bool {
    role_allows_operator(require_role, params, connection_role)
}

/// JSON-RPC error when operator role is missing.
pub fn forbidden_operator_role_error() -> JsonRpcError {
    JsonRpcError {
        code: error_code::FORBIDDEN,
        message: "Forbidden: operator role required".to_owned(),
        data: Some(serde_json::json!({ "required_role": OPERATOR_ROLE })),
    }
}

/// Parse `sim.command` params into a known action.
pub fn parse_sim_command_action(params: Option<&Value>) -> Option<SimCommandAction> {
    match params
        .and_then(|p| p.get("action"))
        .and_then(|a| a.as_str())?
    {
        "noop" => Some(SimCommandAction::Noop),
        "tick" => Some(SimCommandAction::Tick),
        _ => None,
    }
}

/// Institution row on `sim.snapshot` (read-only, from `civ-economy` ledger).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstitutionSnapshot {
    /// Institution id.
    pub id: u32,
    /// `market` or `treasury`.
    pub kind: String,
    /// Joule balance.
    pub balance_joules: i64,
}

/// Snapshot fields from `Simulation::snapshot()` for read-only RPC handlers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotFields {
    /// Engine tick at snapshot time.
    pub tick: u64,
    /// World population.
    pub population: u64,
    /// Count of building entities in the ECS world.
    pub building_count: usize,
    /// Fixed-point energy budget as f64 (omitted on the wire when unset).
    pub energy_budget: Option<f64>,
    /// Per-good clearing prices in cents from `Simulation::market_state`.
    pub market_prices: BTreeMap<String, i64>,
    /// Latest hash-chain root as lowercase hex (omitted on the wire when unset).
    pub hash_chain_root: Option<String>,
    /// Tick speed multiplier from the bridge (`AppState::speed_multiplier`).
    pub speed_multiplier: u32,
    /// Deterministic pins for spectator clients (web, Godot).
    pub spectator: Option<civ_engine::SpectatorView>,
    /// Institution balances (`civ-economy` stub ledger).
    pub institutions: Vec<InstitutionSnapshot>,
    /// Military unit pins for spectator renderers (from ECS `MilitaryUnit`).
    pub military_units: Vec<MilitaryPinSnapshot>,
    /// Damage events applied on the last tick (normalized centers).
    pub damage_events: Vec<DamagePulseSnapshot>,
    /// Count of damage events on the last tick.
    pub damage_events_count: u32,
    /// Voxels removed by tactical damage on the last tick.
    pub voxel_damage_removed_this_tick: u32,
    /// Loaded mods for mod-browser UI (FR-CIV-TACTICS-054).
    pub mods: Vec<civ_mod_host::ModBrowserEntry>,
    /// `mod.loaded.v1` replay-bus JSON from the most recent tick (scenario load).
    pub mod_lifecycle: Vec<String>,
    /// `session.saved.v1` replay-bus JSON from the most recent tick (slot/autosave).
    pub session_saved: Vec<String>,
    /// `mod.permission_violation.v1` replay-bus JSON from the most recent tick.
    pub mod_permission_violations: Vec<String>,
    /// Deterministic planet climate from `phase_planet` (FR-CIV-PLANET-010).
    pub climate: civ_engine::Climate,
    /// Most recent civ-emergence-metrics sample (FR-CIV-EMERG-003
    /// surface). `None` on a fresh sim before the first 50-tick
    /// sample boundary; the bridge copies the same value here as
    /// `DispatchContext::emergence` so dashboard consumers can poll
    /// either `sim.emergence` or `sim.snapshot.emergence` without
    /// missing the dashboard block.
    pub emergence: Option<EmergenceSampleFields>,
    /// Fully-researched techs (FR-CIV-SERVER-003).
    pub researched: Vec<String>,
    /// Currently-researching tech, if any (FR-CIV-SERVER-003).
    pub in_progress_tech: Option<String>,
}

/// Tactical damage pulse for `sim.snapshot` (normalized map coords).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamagePulseSnapshot {
    /// Normalized map X.
    pub x: f32,
    /// Normalized map Y.
    pub y: f32,
    /// Attacking unit pin id when damage came from military contact.
    pub unit_a: Option<u64>,
    /// Defending unit pin id.
    pub unit_b: Option<u64>,
}

/// Military pin row for `sim.snapshot` (matches civ-watch wire shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitaryPinSnapshot {
    /// Stable pin id for clients.
    pub id: u64,
    /// Normalized map X.
    pub x: f32,
    /// Normalized map Y.
    pub y: f32,
    /// Wire label (`Vehicle` maps from `Knight`).
    pub unit_type: String,
    /// Owning faction id.
    pub faction: u32,
    /// Unit strength (float for JSON clients).
    pub strength: f32,
}

/// Build the JSON-RPC result object for `sim.snapshot`.
pub fn snapshot_result_json(fields: &SnapshotFields) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("tick".to_owned(), serde_json::json!(fields.tick));
    obj.insert(
        "population".to_owned(),
        serde_json::json!(fields.population),
    );
    obj.insert(
        "building_count".to_owned(),
        serde_json::json!(fields.building_count),
    );
    if let Some(energy_budget) = fields.energy_budget {
        obj.insert("energy_budget".to_owned(), serde_json::json!(energy_budget));
    }
    obj.insert(
        "market_prices".to_owned(),
        serde_json::json!(fields.market_prices),
    );
    if let Some(hash_chain_root) = &fields.hash_chain_root {
        obj.insert(
            "hash_chain_root".to_owned(),
            serde_json::json!(hash_chain_root),
        );
    }
    obj.insert(
        "speed_multiplier".to_owned(),
        serde_json::json!(fields.speed_multiplier),
    );
    if let Some(spec) = &fields.spectator {
        obj.insert(
            "civ_pins".to_owned(),
            serde_json::to_value(&spec.civ_pins).unwrap_or(Value::Null),
        );
        obj.insert(
            "factions".to_owned(),
            serde_json::to_value(&spec.factions).unwrap_or(Value::Null),
        );
        obj.insert(
            "buildings".to_owned(),
            serde_json::to_value(&spec.buildings).unwrap_or(Value::Null),
        );
        obj.insert("is_day".to_owned(), serde_json::json!(spec.is_day));
    }
    if !fields.institutions.is_empty() {
        obj.insert(
            "institutions".to_owned(),
            serde_json::to_value(&fields.institutions).unwrap_or(Value::Null),
        );
    }
    if !fields.military_units.is_empty() {
        obj.insert(
            "military_units".to_owned(),
            serde_json::to_value(&fields.military_units).unwrap_or(Value::Null),
        );
    }
    obj.insert(
        "damage_events_count".to_owned(),
        serde_json::json!(fields.damage_events_count),
    );
    obj.insert(
        "voxel_damage_removed_this_tick".to_owned(),
        serde_json::json!(fields.voxel_damage_removed_this_tick),
    );
    if !fields.damage_events.is_empty() {
        obj.insert(
            "damage_events".to_owned(),
            serde_json::to_value(&fields.damage_events).unwrap_or(Value::Null),
        );
    }
    if !fields.mods.is_empty() {
        obj.insert(
            "mods".to_owned(),
            serde_json::to_value(&fields.mods).unwrap_or(Value::Null),
        );
    }
    if !fields.mod_lifecycle.is_empty() {
        obj.insert(
            "mod_lifecycle".to_owned(),
            serde_json::to_value(&fields.mod_lifecycle).unwrap_or(Value::Null),
        );
    }
    if !fields.session_saved.is_empty() {
        obj.insert(
            "session_saved".to_owned(),
            serde_json::to_value(&fields.session_saved).unwrap_or(Value::Null),
        );
    }
    if !fields.mod_permission_violations.is_empty() {
        obj.insert(
            "mod_permission_violations".to_owned(),
            serde_json::to_value(&fields.mod_permission_violations).unwrap_or(Value::Null),
        );
    }
    obj.insert(
        "climate".to_owned(),
        serde_json::to_value(fields.climate).unwrap_or(Value::Null),
    );
    if let Some(emergence) = &fields.emergence {
        // `sim.snapshot.emergence` block (FR-CIV-EMERG-003). The five
        // dashboard fields are hoisted to a nested `dashboard` object
        // so dashboard clients can read either the flat wire shape
        // (`result["cluster_entropy"]`) or the nested one
        // (`result["dashboard"]["cluster_entropy"]`). The flat hoist
        // keeps PR #350 dashboard clients working; the nested
        // location is the canonical "all five tiles" handle.
        let mut emergence_obj = serde_json::Map::new();
        emergence_obj.insert("tick".to_owned(), serde_json::json!(emergence.tick));
        emergence_obj.insert(
            "entropy_bits".to_owned(),
            serde_json::json!(emergence.entropy_bits),
        );
        emergence_obj.insert(
            "entropy_norm".to_owned(),
            serde_json::json!(emergence.entropy_norm),
        );
        if let Some(count) = emergence.structure_count {
            emergence_obj.insert("structure_count".to_owned(), serde_json::json!(count));
        }
        if let Some(largest) = emergence.structure_largest {
            emergence_obj.insert("structure_largest".to_owned(), serde_json::json!(largest));
        }
        if let Some(foreground) = emergence.structure_foreground {
            emergence_obj.insert(
                "structure_foreground".to_owned(),
                serde_json::json!(foreground),
            );
        }
        emergence_obj.insert(
            "histogram_total".to_owned(),
            serde_json::json!(emergence.histogram_total),
        );
        emergence_obj.insert(
            "histogram_populated_bins".to_owned(),
            serde_json::json!(emergence.histogram_populated_bins),
        );
        emergence_obj.insert(
            "sample_dur_us".to_owned(),
            serde_json::json!(emergence.sample_dur_us),
        );
        if let Some(dashboard) = &emergence.dashboard {
            emergence_obj.insert(
                "cluster_entropy".to_owned(),
                serde_json::json!(dashboard.cluster_entropy),
            );
            emergence_obj.insert(
                "ideology_homophily".to_owned(),
                serde_json::json!(dashboard.ideology_homophily),
            );
            emergence_obj.insert(
                "sentience_fraction".to_owned(),
                serde_json::json!(dashboard.sentience_fraction),
            );
            emergence_obj.insert(
                "psyche_stability".to_owned(),
                serde_json::json!(dashboard.psyche_stability),
            );
            emergence_obj.insert(
                "diplomacy_tension".to_owned(),
                serde_json::json!(dashboard.diplomacy_tension),
            );
            emergence_obj.insert(
                "dashboard".to_owned(),
                serde_json::to_value(dashboard).unwrap_or(Value::Null),
            );
        }
        emergence_obj.insert(
            "power_law_alpha".to_owned(),
            serde_json::json!(emergence.power_law_alpha),
        );
        emergence_obj.insert(
            "novelty_rate".to_owned(),
            serde_json::json!(emergence.novelty_rate),
        );
        emergence_obj.insert(
            "mi_material_faction_norm".to_owned(),
            emergence
                .mi_material_faction_norm
                .map_or(Value::Null, |v| serde_json::json!(v)),
        );
        obj.insert("emergence".to_owned(), Value::Object(emergence_obj));
    }
    Value::Object(obj)
}

/// Map ECS military units to spectator pins.
pub fn military_pins_from_sim(sim: &civ_engine::Simulation) -> Vec<MilitaryPinSnapshot> {
    use civ_engine::{grid_to_norm, spawn::military_pin_id, unit_type_label, MilitaryUnit};
    sim.world
        .query::<&MilitaryUnit>()
        .iter()
        .enumerate()
        .map(|(idx, (entity, unit))| {
            let (x, y) = grid_to_norm(unit.position);
            MilitaryPinSnapshot {
                id: military_pin_id(entity, idx),
                x,
                y,
                unit_type: unit_type_label(unit.unit_type).to_string(),
                faction: unit.faction_id,
                strength: unit.hp.to_f64() as f32,
            }
        })
        .collect()
}

/// Map live simulation institution ledger to wire rows.
pub fn institutions_from_sim(sim: &civ_engine::Simulation) -> Vec<InstitutionSnapshot> {
    use civ_economy::{InstitutionKind, InstitutionLedger};
    let ledger = if sim.economy_state.institutions.accounts.is_empty() {
        InstitutionLedger::with_defaults()
    } else {
        sim.economy_state.institutions.clone()
    };
    ledger
        .accounts
        .values()
        .map(|account| InstitutionSnapshot {
            id: account.id,
            kind: match account.kind {
                InstitutionKind::Market => "market".to_owned(),
                InstitutionKind::Treasury => "treasury".to_owned(),
            },
            balance_joules: account.balance_joules,
        })
        .collect()
}

/// Build snapshot RPC fields from a live simulation (includes [`SpectatorView`]).
pub fn snapshot_fields_from_sim(
    sim: &civ_engine::Simulation,
    speed_multiplier: u32,
) -> SnapshotFields {
    let snap = sim.snapshot();
    SnapshotFields {
        tick: snap.tick,
        population: snap.population,
        building_count: snap.building_count,
        energy_budget: Some(snap.energy_budget.to_f64()),
        market_prices: snap.market_prices.clone(),
        hash_chain_root: sim
            .hash_chain_root()
            .map(|root| civ_engine::hash_hex(&root)),
        speed_multiplier,
        spectator: Some(sim.spectator_view()),
        institutions: institutions_from_sim(sim),
        military_units: military_pins_from_sim(sim),
        damage_events: sim
            .last_tick_combat_pulses()
            .iter()
            .map(|pulse| DamagePulseSnapshot {
                x: pulse.x,
                y: pulse.y,
                unit_a: pulse.unit_a,
                unit_b: pulse.unit_b,
            })
            .collect(),
        damage_events_count: sim.last_tick_combat_pulses().len() as u32,
        voxel_damage_removed_this_tick: sim.last_tick_voxel_damage_count() as u32,
        mods: sim.mod_browser_entries(),
        mod_lifecycle: sim.replay_log().mod_loaded_bus_at_tick(sim.state.tick),
        session_saved: sim.replay_log().session_saved_bus_at_tick(sim.state.tick),
        mod_permission_violations: sim
            .replay_log()
            .mod_permission_violation_bus_at_tick(sim.state.tick),
        climate: *sim.climate(),
        emergence: sim.last_emergence_sample().map(EmergenceSampleFields::from),
        researched: vec![],
        in_progress_tech: None,
    }
}

/// Precomputed outcome for `sim.outcome` (FR-CIV-GAME-001).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutcomeFields {
    pub tag: String,
    pub reason: String,
    pub tick: u64,
}

/// Tick and optional snapshot fields passed into dispatch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DispatchContext {
    /// Current bridge tick (may lag until the next broadcast).
    pub tick: u64,
    /// `snapshot().population` when the handler could read the simulation.
    pub population: Option<u64>,
    /// Full snapshot when the handler could read the simulation.
    pub snapshot: Option<SnapshotFields>,
    /// When true, privileged `sim.command` actions require the operator role.
    pub require_role: bool,
    /// Current tick speed multiplier from the bridge (`AppState::speed_multiplier`).
    pub speed_multiplier: u32,
    /// Role established on connect (header) or first JSON-RPC message params.
    pub connection_role: Option<String>,
    /// Bridge save directory for `save.*` handlers.
    pub saves_dir: Option<PathBuf>,
    /// Latest civ-emergence-metrics sample (PR #350 stack). `None` on a
    /// fresh simulation before the first 50-tick sample boundary.
    pub emergence: Option<EmergenceSampleFields>,
    /// Fully-researched techs from `ResearchCache` (FR-CIV-SERVER-003).
    pub researched: Vec<String>,
    /// Currently-researching tech name, if any (FR-CIV-SERVER-003).
    pub in_progress_tech: Option<String>,
    /// Precomputed game outcome for `sim.outcome` handler (FR-CIV-GAME-001).
    pub outcome_fields: Option<OutcomeFields>,
    /// Server-reported last tick wall-clock duration (ms) for sim.perf (FR-CIV-PERF-001).
    pub last_tick_ms: f64,
}

/// JSON-RPC view of [`civ_engine::emergence_metrics::EmergenceSample`].
///
/// Mirrors the engine type but keeps a fixed, transport-friendly
/// representation (no `f32` precision surprises, all `Option`s for the
/// "no dense chunks yet" boot state). Emitted by the `sim.emergence`
/// JSON-RPC method and embedded in `sim.snapshot` for dashboard
/// consumers that poll the snapshot stream instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergenceSampleFields {
    /// Engine tick the sample was taken at.
    pub tick: u64,
    /// Shannon entropy (bits) over the live material histogram.
    pub entropy_bits: f32,
    /// Normalised Shannon entropy (`0..=1`).
    pub entropy_norm: f32,
    /// 6-connectivity component count on the first dense chunk.
    pub structure_count: Option<u32>,
    /// Size (in voxels) of the largest component in the sampled chunk.
    pub structure_largest: Option<u32>,
    /// Number of foreground voxels in the sampled chunk.
    pub structure_foreground: Option<u32>,
    /// Total number of voxels accumulated into the histogram.
    pub histogram_total: u64,
    /// Number of populated bins in the material histogram.
    pub histogram_populated_bins: u32,
    /// Wall-clock duration of the sample, in microseconds.
    pub sample_dur_us: u64,
    /// Five-tile dashboard block (FR-CIV-EMERG-001/003). Field is
    /// `None` only on the (rare) path where the engine sample is
    /// absent and we still want to emit a non-`null` JSON object;
    /// the wire shape is `cluster_entropy`, `ideology_homophily`,
    /// `sentience_fraction`, `psyche_stability`, `diplomacy_tension`.
    pub dashboard: Option<DashboardBlock>,
    /// Rolling-mean branching ratio `σ̄_W` (charter §3.6).
    pub branching_sigma: f32,
    /// Normalised edge-of-chaos score for `branching_sigma`.
    pub branching_sigma_score: f32,
    /// Rolling window `W` for `branching_sigma`.
    pub branching_window: u32,
    /// Monotonic count of closed avalanches.
    pub avalanches_closed: u64,
    /// Charter regime label for `branching_sigma`.
    pub branching_regime: String,
    /// Power-law exponent α for the cluster-size distribution (charter §3.5).
    pub power_law_alpha: f32,
    /// Novelty rate: novel config fingerprints per window per civilian (charter §3.4).
    pub novelty_rate: f32,
    /// Normalised mutual information between material and faction distributions.
    pub mi_material_faction_norm: Option<f32>,
}

/// Wire-friendly mirror of
/// [`civ_emergence_metrics::dashboard::EmergenceDashboard`] for the
/// `sim.emergence` / `sim.snapshot.emergence` JSON-RPC surface
/// (FR-CIV-EMERG-003). The struct re-exports the five f32 fields as a
/// nested object so dashboard clients can read either the flat
/// `result["cluster_entropy"]` shape *or* the nested
/// `result["dashboard"]["cluster_entropy"]` shape — both are emitted
/// by the dispatch for backwards compatibility with PR #350 consumers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardBlock {
    /// Normalised Shannon entropy over per-cluster population sizes.
    pub cluster_entropy: f32,
    /// Homophily index for the ideology distribution.
    pub ideology_homophily: f32,
    /// Fraction of agents that have crossed the sentience threshold.
    pub sentience_fraction: f32,
    /// Population stability of mood valence.
    pub psyche_stability: f32,
    /// Mean absolute tension across the recent diplomacy events.
    pub diplomacy_tension: f32,
}

impl From<civ_emergence_metrics::dashboard::EmergenceDashboard> for DashboardBlock {
    fn from(d: civ_emergence_metrics::dashboard::EmergenceDashboard) -> Self {
        Self {
            cluster_entropy: d.cluster_entropy,
            ideology_homophily: d.ideology_homophily,
            sentience_fraction: d.sentience_fraction,
            psyche_stability: d.psyche_stability,
            diplomacy_tension: d.diplomacy_tension,
        }
    }
}

impl From<civ_engine::emergence_metrics::EmergenceSample> for EmergenceSampleFields {
    fn from(s: civ_engine::emergence_metrics::EmergenceSample) -> Self {
        Self {
            tick: s.tick,
            entropy_bits: s.entropy_bits,
            entropy_norm: s.entropy_norm,
            structure_count: s.structure_count,
            structure_largest: s.structure_largest,
            structure_foreground: s.structure_foreground,
            histogram_total: s.histogram_total,
            histogram_populated_bins: s.histogram_populated_bins,
            sample_dur_us: s.sample_dur_us,
            dashboard: Some(DashboardBlock::from(s.dashboard)),
            branching_sigma: s.branching_sigma,
            branching_sigma_score: s.branching_sigma_score,
            branching_window: s.branching_window,
            avalanches_closed: s.avalanches_closed,
            branching_regime: s.branching_regime.label().to_string(),
            power_law_alpha: s.power_law_alpha,
            novelty_rate: s.novelty_rate,
            mi_material_faction_norm: s.mi_material_faction_norm,
        }
    }
}

/// Side effect the WebSocket bridge must apply after building the wire response.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DispatchEffect {
    /// No simulation mutation.
    #[default]
    None,
    /// Advance the simulation by one tick (`sim.command` tick).
    AdvanceTick,
    /// Write the replay log to disk (`sim.save_replay`).
    SaveReplay {
        /// Filesystem path from request params.
        path: String,
    },
    /// Replace the bridge simulation from a replay file (`sim.load_replay`).
    LoadReplay {
        /// Filesystem path from request params.
        path: String,
    },
    /// Replace the bridge simulation with `Simulation::with_seed` (`sim.reset`).
    ResetSimulation {
        /// RNG/world seed from request params.
        seed: u64,
    },
    /// Load a named preset and start fresh (`sim.load_scenario`).
    LoadScenario {
        /// Name of the preset (e.g. "three-race-balanced").
        preset: String,
        /// RNG seed for this run.
        seed: u64,
    },
    /// Update `Simulation::economy_policy` (`sim.set_policy`).
    SetPolicy {
        /// Validated scarcity multiplier.
        scarcity_multiplier: f64,
        /// Optional base consumption override (joules).
        base_consumption_joules: Option<u64>,
    },
    /// Store tick speed multiplier on the bridge (`sim.set_speed`).
    SetSpeed {
        /// Validated multiplier (mirrors civ-watch `/control/speed`).
        multiplier: u32,
    },
    /// Spawn one civilian (`sim.spawn_civilian`).
    SpawnCivilian {
        /// Normalized X in `[0, 1]` (maps to world X).
        x: f32,
        /// Normalized Y in `[0, 1]` (maps to world Z).
        y: f32,
        /// Faction id.
        faction: u32,
        /// Entity id seed (bridge tick xor constant).
        entity_seq: u64,
    },
    /// Spawn civilian, vehicle, or airport (`sim.spawn_entity`, FR-CIV-UX-006).
    SpawnEntity {
        /// Palette kind from request params.
        kind: SpawnEntityKind,
        /// Normalized map X.
        x: f32,
        /// Normalized map Y.
        y: f32,
        /// Owning faction id.
        faction: u32,
        /// Entity id seed for civilians only.
        entity_seq: u64,
    },
    /// Write one voxel (`sim.place_voxel`).
    GodAction { action: String, x: Option<f32>, y: Option<f32>, target_faction: Option<u32>, magnitude: Option<f32> },
    PlaceVoxel {
        /// World X coordinate.
        x: i64,
        /// World Y coordinate.
        y: i64,
        /// World Z coordinate.
        z: i64,
        /// Material id (0–255).
        material: u16,
    },
    /// Queue tactical voxel damage (`sim.damage`).
    ApplyDamage {
        /// Damage event applied on next tactics phase (replay-logged).
        event: civ_engine::DamageEvent,
    },
    /// Write a production slot archive (`save.slot`).
    SaveSlot {
        /// Validated slot stem (`slot-1` … `slot-5`).
        slot_name: String,
    },
    /// Replace bridge simulation from a slot archive (`save.load`).
    LoadSlot {
        /// Validated slot stem (`slot-1` … `slot-5`).
        slot_name: String,
    },
}

/// Outcome of dispatching a JSON-RPC request.
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchPlan {
    /// Wire response (result `tick` is filled after advance/load when applicable).
    pub response: JsonRpcResponse,
    /// Optional bridge-side simulation work.
    pub effect: DispatchEffect,
}

/// Build a JSON-RPC error response for a parse/validation failure.
///
/// When the raw body is valid JSON, echoes the request `id` when present; otherwise uses `null`.
pub fn parse_error_response(raw: &str, err: JsonRpcParseError) -> JsonRpcResponse {
    let id = serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|v| v.get("id").cloned())
        .and_then(|v| parse_request_id(v).ok())
        .unwrap_or(RequestId::Null);
    JsonRpcResponse::failure(id, err.into_error())
}

/// Serialize a JSON-RPC response for a WebSocket text frame.
pub fn encode_response(response: &JsonRpcResponse) -> String {
    serde_json::to_string(response).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#
            .to_owned()
    })
}

/// Parse `sim.load_scenario` params: `{ "preset": String, "seed"?: u64 }`.
pub fn parse_load_scenario_params(
    params: Option<&Value>,
) -> Result<(String, u64), JsonRpcError> {
    let p = params.ok_or_else(|| JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: r#"Invalid params: expected object with "preset""#.to_owned(),
        data: None,
    })?;
    let preset = p
        .get("preset")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: r#"Invalid params: expected string "preset""#.to_owned(),
            data: None,
        })?;
    let seed = p
        .get("seed")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(42)
        });
    Ok((preset, seed))
}

/// Parse `sim.reset` `seed` param (optional; defaults to 0 when absent).
pub fn parse_reset_seed(params: Option<&Value>) -> Result<u64, JsonRpcError> {
    // If seed key is present it must be a valid u64; if absent or params is
    // None the seed defaults to 0 so callers can omit it entirely.
    match params.and_then(|p| p.get("seed")) {
        None => Ok(0),
        Some(v) => v.as_u64().ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected unsigned integer \"seed\"".to_owned(),
            data: None,
        }),
    }
}

/// Parsed `sim.set_policy` params.
#[derive(Debug, Clone, PartialEq)]
pub struct SetPolicyParams {
    /// Economy scarcity multiplier (must be >= 0).
    pub scarcity_multiplier: f64,
    /// When set, replaces `base_consumption_joules` on the simulation policy.
    pub base_consumption_joules: Option<u64>,
}

/// Parse and validate `sim.set_policy` params.
pub fn parse_set_policy_params(params: Option<&Value>) -> Result<SetPolicyParams, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: "Invalid params: expected object with \"scarcity_multiplier\"".to_owned(),
        data: None,
    })?;

    let scarcity_multiplier = params
        .get("scarcity_multiplier")
        .and_then(|v| v.as_f64())
        .ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected number \"scarcity_multiplier\"".to_owned(),
            data: None,
        })?;

    if scarcity_multiplier < 0.0
        || scarcity_multiplier.is_nan()
        || scarcity_multiplier.is_infinite()
    {
        return Err(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: \"scarcity_multiplier\" must be >= 0".to_owned(),
            data: None,
        });
    }

    let base_consumption_joules =
        match params.get("base_consumption_joules") {
            None => None,
            Some(v) => Some(
                v.as_u64().ok_or(JsonRpcError {
                    code: error_code::INVALID_PARAMS,
                    message:
                        "Invalid params: expected unsigned integer \"base_consumption_joules\""
                            .to_owned(),
                    data: None,
                })?,
            ),
        };

    Ok(SetPolicyParams {
        scarcity_multiplier,
        base_consumption_joules,
    })
}

/// Speed multipliers accepted by `sim.set_speed` (mirrors civ-watch `/control/speed`).
pub const ALLOWED_SPEED_MULTIPLIERS: [u32; 5] = [0, 1, 2, 4, 8];

/// Parse and validate `sim.set_speed` params.
pub fn parse_set_speed_params(params: Option<&Value>) -> Result<u32, JsonRpcError> {
    let multiplier = params
        .and_then(|p| p.get("multiplier"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected unsigned integer \"multiplier\"".to_owned(),
            data: None,
        })?;

    if !ALLOWED_SPEED_MULTIPLIERS.contains(&multiplier) {
        return Err(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: \"multiplier\" must be 0, 1, 2, 4, or 8".to_owned(),
            data: None,
        });
    }

    Ok(multiplier)
}

/// Parse `sim.save_replay` / `sim.load_replay` `path` param.
///
/// The returned path is **strictly relative**: it must not be absolute,
/// must not start with a path separator, must not contain any `..`
/// parent segments, prefix components (e.g. `C:`), or root directory
/// components. This blocks arbitrary-file-read attacks where a client
/// sends `/etc/passwd` or `../../something`. Containment under the
/// bridge's replay base directory is enforced separately at the
/// consumer via [`resolve_replay_path`].
pub fn parse_replay_path(params: Option<&Value>) -> Result<String, JsonRpcError> {
    let path_str = params
        .and_then(|p| p.get("path"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .filter(|path_str| {
            // Reject absolute paths (Unix-style starting with / or Windows-style with drive letter)
            let path = std::path::Path::new(path_str);
            path.is_relative()
                && !path_str.starts_with('/')
                && !path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
        })
        .ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected relative replay path without parent segments"
                .to_owned(),
            data: None,
        })?;
    let path = std::path::Path::new(path_str);
    if !path.is_relative()
        || path_str.starts_with('/')
        || path_str.starts_with('\\')
        || path
            .components()
            .any(|c| matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
            ))
    {
        return Err(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: format!(
                "Invalid params: replay path {path_str:?} must be a relative path with no \"..\", absolute, or prefix components"
            ),
            data: None,
        });
    }
    Ok(path_str.to_owned())
}

/// Resolve a validated relative replay path under `base_dir` and verify
/// the canonicalized result is still contained inside the canonicalized
/// base directory. This defeats symlink escape: `..` and absolute paths
/// are rejected earlier by [`parse_replay_path`], but a relative path
/// like `replays/../../etc/passwd` (after symlink resolution) could
/// otherwise still escape if a parent of `base_dir` is itself a
/// symlink. Canonicalizing both sides and checking containment closes
/// that gap.
pub fn resolve_replay_path(
    base_dir: &std::path::Path,
    relative: &str,
) -> Result<std::path::PathBuf, String> {
    let base_canonical = std::fs::canonicalize(base_dir)
        .map_err(|err| format!("replay base dir {:?} not accessible: {err}", base_dir))?;
    let joined = base_canonical.join(relative);
    // Canonicalize the joined path; if the file does not yet exist
    // (e.g. `sim.save_replay` is creating it), canonicalize the deepest
    // existing ancestor and re-append the final filename. This still
    // defeats symlink escape because every existing parent must resolve
    // to a path under `base_canonical` for the containment check to
    // pass.
    let canonical = match std::fs::canonicalize(&joined) {
        Ok(path) => path,
        Err(_) => {
            let mut existing = joined.clone();
            let mut tail_components: Vec<std::path::PathBuf> = Vec::new();
            loop {
                let parent = existing
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .ok_or_else(|| {
                        format!("replay path {:?} has no existing ancestor", joined)
                    })?;
                if parent == existing {
                    return Err(format!(
                        "replay path {:?} cannot be resolved relative to base {:?}",
                        joined, base_canonical
                    ));
                }
                if let Some(name) = existing.file_name() {
                    tail_components.push(std::path::PathBuf::from(name));
                }
                match std::fs::canonicalize(parent) {
                    Ok(canonical_parent) => {
                        let mut result = canonical_parent;
                        for component in tail_components.iter().rev() {
                            result.push(component);
                        }
                        break result;
                    }
                    Err(_) => {
                        existing = parent.to_path_buf();
                    }
                }
            }
        }
    };
    if !canonical.starts_with(&base_canonical) {
        return Err(format!(
            "replay path {:?} escapes replay base directory {:?}",
            canonical, base_canonical
        ));
    }
    Ok(canonical)
}

/// Dispatch a validated request (CIV-0200 stub handlers).
pub fn dispatch_request(req: JsonRpcRequest, ctx: DispatchContext) -> DispatchPlan {
    match req.method {
        JsonRpcMethod::Health => DispatchPlan {
            response: JsonRpcResponse::success(req.id, serde_json::json!({ "tick": ctx.tick })),
            effect: DispatchEffect::None,
        },
        JsonRpcMethod::SimStatus => {
            let result = match ctx.population {
                Some(population) => {
                    serde_json::json!({ "tick": ctx.tick, "population": population })
                }
                None => serde_json::json!({ "tick": ctx.tick }),
            };
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, result),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimSnapshot => {
            let result = match ctx.snapshot.as_ref() {
                Some(fields) => snapshot_result_json(fields),
                None => serde_json::json!({
                    "tick": ctx.tick,
                    "speed_multiplier": ctx.speed_multiplier,
                }),
            };
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, result),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimSaveReplay => match parse_replay_path(req.params.as_ref()) {
            Ok(path) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "saved": true, "path": path }),
                ),
                effect: DispatchEffect::SaveReplay { path },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimLoadReplay => match parse_replay_path(req.params.as_ref()) {
            Ok(path) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "loaded": true, "tick": ctx.tick }),
                ),
                effect: DispatchEffect::LoadReplay { path },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimReset => match parse_reset_seed(req.params.as_ref()) {
            Ok(seed) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "seed": seed, "tick": 0 }),
                ),
                effect: DispatchEffect::ResetSimulation { seed },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimLoadScenario => match parse_load_scenario_params(req.params.as_ref()) {
            Ok((preset, seed)) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "preset": preset, "seed": seed, "tick": 0 }),
                ),
                effect: DispatchEffect::LoadScenario { preset, seed },
            },
            Err(e) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, e),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimSetPolicy => match parse_set_policy_params(req.params.as_ref()) {
            Ok(policy) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({
                        "updated": true,
                        "scarcity_multiplier": policy.scarcity_multiplier,
                    }),
                ),
                effect: DispatchEffect::SetPolicy {
                    scarcity_multiplier: policy.scarcity_multiplier,
                    base_consumption_joules: policy.base_consumption_joules,
                },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimSetSpeed => match parse_set_speed_params(req.params.as_ref()) {
            Ok(multiplier) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "accepted": true, "multiplier": multiplier }),
                ),
                effect: DispatchEffect::SetSpeed { multiplier },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimGetSpeed => DispatchPlan {
            response: JsonRpcResponse::success(
                req.id,
                serde_json::json!({ "multiplier": ctx.speed_multiplier }),
            ),
            effect: DispatchEffect::None,
        },
        JsonRpcMethod::SimEmergence => {
            // Latest emergence-metrics sample (civ-emergence-metrics
            // via `crates/engine::emergence_metrics`). Returns `null`
            // for the no-sample-yet state (ticks 0..49 on a fresh sim)
            // so dashboard clients can disambiguate "no data" from
            // "entropy is exactly zero".
            let result = match ctx.emergence.as_ref() {
                Some(sample) => serde_json::to_value(sample).unwrap_or(serde_json::json!({
                    "tick": ctx.tick,
                })),
                None => serde_json::json!({ "tick": ctx.tick, "sample": serde_json::Value::Null }),
            };
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, result),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimInspectTile => {
            let x = req.params.as_ref().and_then(|p| p.get("x").and_then(|v| v.as_i64())).unwrap_or(0);
            let y = req.params.as_ref().and_then(|p| p.get("y").and_then(|v| v.as_i64())).unwrap_or(0);
            let result = serde_json::json!({ "x": x, "y": y, "stub": true });
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, result),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimDiplomacyAction => {
            let action = req.params.as_ref()
                .and_then(|p| p.get("action"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let target = req.params.as_ref()
                .and_then(|p| p.get("target_faction"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let result = serde_json::json!({
                "action": action,
                "target_faction": target,
                "stub": true,
                "tick": ctx.tick,
            });
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, result),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimSpawnCivilian => {
            if !role_allows_operator(
                ctx.require_role,
                req.params.as_ref(),
                ctx.connection_role.as_deref(),
            ) {
                DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, forbidden_operator_role_error()),
                    effect: DispatchEffect::None,
                }
            } else {
                match parse_spawn_civilian_params(req.params.as_ref()) {
                    Ok((x, y, faction)) => DispatchPlan {
                        response: JsonRpcResponse::success(
                            req.id,
                            serde_json::json!({ "accepted": true }),
                        ),
                        effect: DispatchEffect::SpawnCivilian {
                            x,
                            y,
                            faction,
                            entity_seq: ctx.tick.wrapping_add(1) ^ 0x00c0_ffee,
                        },
                    },
                    Err(error) => DispatchPlan {
                        response: JsonRpcResponse::failure(req.id, error),
                        effect: DispatchEffect::None,
                    },
                }
            }
        }
        JsonRpcMethod::SimSpawnEntity => {
            if !role_allows_operator(
                ctx.require_role,
                req.params.as_ref(),
                ctx.connection_role.as_deref(),
            ) {
                DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, forbidden_operator_role_error()),
                    effect: DispatchEffect::None,
                }
            } else {
                match parse_spawn_entity_params(req.params.as_ref()) {
                    Ok((kind, x, y, faction)) => DispatchPlan {
                        response: JsonRpcResponse::success(
                            req.id,
                            serde_json::json!({ "accepted": true, "kind": kind.wire_label() }),
                        ),
                        effect: DispatchEffect::SpawnEntity {
                            kind,
                            x,
                            y,
                            faction,
                            entity_seq: ctx.tick.wrapping_add(1) ^ 0x00c0_ffee,
                        },
                    },
                    Err(error) => DispatchPlan {
                        response: JsonRpcResponse::failure(req.id, error),
                        effect: DispatchEffect::None,
                    },
                }
            }
        }
        JsonRpcMethod::SimPlaceVoxel => {
            if !role_allows_operator(
                ctx.require_role,
                req.params.as_ref(),
                ctx.connection_role.as_deref(),
            ) {
                DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, forbidden_operator_role_error()),
                    effect: DispatchEffect::None,
                }
            } else {
                match parse_place_voxel_params(req.params.as_ref()) {
                    Ok((x, y, z, material)) => DispatchPlan {
                        response: JsonRpcResponse::success(
                            req.id,
                            serde_json::json!({ "accepted": true }),
                        ),
                        effect: DispatchEffect::PlaceVoxel { x, y, z, material },
                    },
                    Err(error) => DispatchPlan {
                        response: JsonRpcResponse::failure(req.id, error),
                        effect: DispatchEffect::None,
                    },
                }
            }
        }
        JsonRpcMethod::SimDamage => {
            if !role_allows_operator(
                ctx.require_role,
                req.params.as_ref(),
                ctx.connection_role.as_deref(),
            ) {
                DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, forbidden_operator_role_error()),
                    effect: DispatchEffect::None,
                }
            } else {
                match parse_damage_params(req.params.as_ref()) {
                    Ok(event) => DispatchPlan {
                        response: JsonRpcResponse::success(
                            req.id,
                            serde_json::json!({ "accepted": true }),
                        ),
                        effect: DispatchEffect::ApplyDamage { event },
                    },
                    Err(error) => DispatchPlan {
                        response: JsonRpcResponse::failure(req.id, error),
                        effect: DispatchEffect::None,
                    },
                }
            }
        }
        JsonRpcMethod::SimCommand => match parse_sim_command_action(req.params.as_ref()) {
            Some(SimCommandAction::Noop) => DispatchPlan {
                response: JsonRpcResponse::success(req.id, serde_json::json!({ "accepted": true })),
                effect: DispatchEffect::None,
            },
            Some(SimCommandAction::Tick) => {
                if !role_allows_operator_tick(
                    ctx.require_role,
                    req.params.as_ref(),
                    ctx.connection_role.as_deref(),
                ) {
                    DispatchPlan {
                        response: JsonRpcResponse::failure(req.id, forbidden_operator_role_error()),
                        effect: DispatchEffect::None,
                    }
                } else {
                    DispatchPlan {
                        response: JsonRpcResponse::success(
                            req.id,
                            serde_json::json!({ "accepted": true, "tick": ctx.tick }),
                        ),
                        effect: DispatchEffect::AdvanceTick,
                    }
                }
            }
            None => DispatchPlan {
                response: JsonRpcResponse::failure(
                    req.id,
                    JsonRpcError {
                        code: error_code::METHOD_NOT_FOUND,
                        message: "Method not found".to_owned(),
                        data: None,
                    },
                ),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SaveSlot => match parse_slot_name_params(req.params.as_ref()) {
            Ok(slot_name) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "saved": true, "slot_name": slot_name }),
                ),
                effect: DispatchEffect::SaveSlot { slot_name },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::LoadSlot => match parse_slot_name_params(req.params.as_ref()) {
            Ok(slot_name) => DispatchPlan {
                response: JsonRpcResponse::success(
                    req.id,
                    serde_json::json!({ "loaded": true, "slot_name": slot_name }),
                ),
                effect: DispatchEffect::LoadSlot { slot_name },
            },
            Err(error) => DispatchPlan {
                response: JsonRpcResponse::failure(req.id, error),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SaveList => match ctx.saves_dir.as_deref() {
            Some(dir) => match list_saves(dir) {
                Ok(entries) => DispatchPlan {
                    response: JsonRpcResponse::success(
                        req.id,
                        serde_json::to_value(entries).unwrap_or(Value::Array(vec![])),
                    ),
                    effect: DispatchEffect::None,
                },
                Err(error) => DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, error),
                    effect: DispatchEffect::None,
                },
            },
            None => DispatchPlan {
                response: JsonRpcResponse::failure(
                    req.id,
                    JsonRpcError {
                        code: error_code::INTERNAL_ERROR,
                        message: "Save directory unavailable".to_owned(),
                        data: None,
                    },
                ),
                effect: DispatchEffect::None,
            },
        },
        JsonRpcMethod::SimQueueResearch => {
            const KNOWN_TECHS: &[&str] = &[
                "pottery", "masonry", "writing", "iron_working", "currency",
                "mathematics", "gunpowder", "printing", "banking",
                "steam_power", "electricity", "railroad",
            ];
            let tech = req.params.as_ref()
                .and_then(|p| p.get("tech"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if tech.is_empty() {
                DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, JsonRpcError {
                        code: error_code::INVALID_PARAMS,
                        message: "Missing or empty \"tech\" parameter".to_owned(),
                        data: None,
                    }),
                    effect: DispatchEffect::None,
                }
            } else if !KNOWN_TECHS.contains(&tech) {
                DispatchPlan {
                    response: JsonRpcResponse::failure(req.id, JsonRpcError {
                        code: error_code::INVALID_PARAMS,
                        message: format!("Unknown tech \"{tech}\"; valid: {}", KNOWN_TECHS.join(", ")),
                        data: None,
                    }),
                    effect: DispatchEffect::None,
                }
            } else {
                DispatchPlan {
                    response: JsonRpcResponse::success(req.id, serde_json::json!({
                        "queued": tech,
                        "tick": ctx.tick,
                    })),
                    effect: DispatchEffect::None,
                }
            }
        }
        JsonRpcMethod::SimTechState => DispatchPlan {
            response: JsonRpcResponse::success(req.id, serde_json::json!({
                "available": [
                    "pottery", "masonry", "writing", "iron_working", "currency",
                    "mathematics", "gunpowder", "printing", "banking",
                    "steam_power", "electricity", "railroad",
                ],
                "researched": ctx.researched,
                "in_progress": ctx.in_progress_tech,
                "tick": ctx.tick,
            })),
            effect: DispatchEffect::None,
        },
        JsonRpcMethod::SimOutcome => {
            use civ_engine::check_outcome;
            let outcome_result = {
                let sim = ctx.snapshot.as_ref().map(|_| ()).is_some();
                // We only have a ctx snapshot; real outcome check needs the live sim.
                // The ws_bridge populates outcome_fields before dispatch (see below).
                ctx.outcome_fields.clone().unwrap_or_else(|| crate::jsonrpc::OutcomeFields {
                    tag: "ongoing".to_owned(),
                    reason: String::new(),
                    tick: ctx.tick,
                })
            };
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, serde_json::json!({
                    "outcome": outcome_result.tag,
                    "reason": outcome_result.reason,
                    "tick": outcome_result.tick,
                })),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimGodAction => {
            let params = req.params.as_ref().and_then(|p| p.as_object())
                .ok_or("sim.god_action requires params object");
            match params {
                Err(msg) => DispatchPlan { response: JsonRpcResponse::failure(req.id, JsonRpcError { code: error_code::INVALID_PARAMS, message: msg.to_owned(), data: None }), effect: DispatchEffect::None },
                Ok(p) => DispatchPlan {
                    response: JsonRpcResponse::success(req.id, serde_json::json!({ "ok": true, "tick": ctx.tick })),
                    effect: DispatchEffect::GodAction {
                        action: p.get("action").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
                        x: p.get("x").and_then(|v| v.as_f64()).map(|f| f as f32),
                        y: p.get("y").and_then(|v| v.as_f64()).map(|f| f as f32),
                        target_faction: p.get("target_faction").and_then(|v| v.as_u64()).map(|f| f as u32),
                        magnitude: p.get("magnitude").and_then(|v| v.as_f64()).map(|f| f as f32),
                    },
                },
            }
        }
        JsonRpcMethod::SimPerf => {
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, serde_json::json!({
                    "tick": ctx.tick,
                    "last_tick_ms": ctx.last_tick_ms,
                    "agent_count": ctx.population.unwrap_or(0) as u32,
                    "ca_steps": 1u32,
                        "stub": true,
                    })),
                    effect: DispatchEffect::None,
                }
        }
        JsonRpcMethod::SimTechState => {
            DispatchPlan {
                response: JsonRpcResponse::success(req.id, serde_json::json!({
                    "available": ["pottery", "masonry", "writing", "iron_working",
                                  "currency", "mathematics", "gunpowder", "printing",
                                  "banking", "steam_power", "electricity", "railroad"],
                    "researched": [],
                    "in_progress": null,
                    "tick": ctx.tick,
                    "stub": true,
                })),
                effect: DispatchEffect::None,
            }
        }
        JsonRpcMethod::SimSubscribe
        | JsonRpcMethod::SimUpdateSubscription
        | JsonRpcMethod::SimUnsubscribe => DispatchPlan {
            response: JsonRpcResponse::failure(
                req.id,
                JsonRpcError {
                    code: error_code::INTERNAL_ERROR,
                    message: "sim.subscribe/unsubscribe require an active WebSocket connection"
                        .to_owned(),
                    data: None,
                },
            ),
            effect: DispatchEffect::None,
        },
    }
}

/// Palette entity for `sim.spawn_entity` (FR-CIV-UX-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnEntityKind {
    /// Spawn via `civ-agents` civilian helper.
    Civilian,
    /// Spawn military unit (`Knight` → wire label `Vehicle`).
    Vehicle,
    /// Spawn civic hub building (`CityCenter`).
    Airport,
    /// Spawn harbor / trade port (`Market`).
    Port,
    /// Spawn hangar / barracks (`Barracks`).
    Hangar,
}

impl SpawnEntityKind {
    /// JSON `kind` string echoed in RPC results.
    pub fn wire_label(self) -> &'static str {
        match self {
            Self::Civilian => "civilian",
            Self::Vehicle => "vehicle",
            Self::Airport => "airport",
            Self::Port => "port",
            Self::Hangar => "hangar",
        }
    }
}

/// Parse `sim.spawn_entity` params: `{ "kind", "x", "y", "faction"? }`.
pub fn parse_spawn_entity_params(
    params: Option<&Value>,
) -> Result<(SpawnEntityKind, f32, f32, u32), JsonRpcError> {
    let p = params.ok_or(JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: "Missing params".to_owned(),
        data: None,
    })?;
    let kind = match p.get("kind").and_then(|v| v.as_str()) {
        Some("civilian") => SpawnEntityKind::Civilian,
        Some("vehicle") => SpawnEntityKind::Vehicle,
        Some("airport") => SpawnEntityKind::Airport,
        Some("port") => SpawnEntityKind::Port,
        Some("hangar") => SpawnEntityKind::Hangar,
        _ => {
            return Err(JsonRpcError {
                code: error_code::INVALID_PARAMS,
                message: "kind must be civilian, vehicle, airport, port, or hangar".to_owned(),
                data: None,
            });
        }
    };
    let (x, y, faction) = parse_spawn_civilian_params(Some(p))?;
    Ok((kind, x, y, faction))
}

/// Parse `sim.spawn_civilian` params: `{ "x", "y", "faction" }` (normalized coords).
pub fn parse_spawn_civilian_params(
    params: Option<&Value>,
) -> Result<(f32, f32, u32), JsonRpcError> {
    let p = params.ok_or(JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: "Missing params".to_owned(),
        data: None,
    })?;
    let x = p
        .get("x")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .ok_or(invalid_params("x"))?;
    let y = p
        .get("y")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .ok_or(invalid_params("y"))?;
    let faction = p
        .get("faction")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0);
    Ok((x, y, faction))
}

/// Parse `sim.damage` params: `{ "x", "y", "z", "radius", "energy"? }` (fixed-point world coords).
pub fn parse_damage_params(
    params: Option<&Value>,
) -> Result<civ_engine::DamageEvent, JsonRpcError> {
    let p = params.ok_or(JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: "Missing params".to_owned(),
        data: None,
    })?;
    let x = p
        .get("x")
        .and_then(|v| v.as_i64())
        .ok_or(invalid_params("x"))?;
    let y = p
        .get("y")
        .and_then(|v| v.as_i64())
        .ok_or(invalid_params("y"))?;
    let z = p
        .get("z")
        .and_then(|v| v.as_i64())
        .ok_or(invalid_params("z"))?;
    let radius = p
        .get("radius")
        .and_then(|v| v.as_u64())
        .map(|v| v.clamp(1, 32) as u8)
        .unwrap_or(8);
    let energy = p
        .get("energy")
        .and_then(|v| v.as_u64())
        .map(|v| v.min(u32::MAX as u64) as u32)
        .unwrap_or(1000);
    Ok(civ_engine::DamageEvent {
        center: civ_voxel::WorldCoord { x, y, z },
        radius_voxels: radius,
        energy,
    })
}

/// Parse `sim.place_voxel` params: `{ "x", "y", "z", "material" }`.
pub fn parse_place_voxel_params(
    params: Option<&Value>,
) -> Result<(i64, i64, i64, u16), JsonRpcError> {
    let p = params.ok_or(JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: "Missing params".to_owned(),
        data: None,
    })?;
    let x = p
        .get("x")
        .and_then(|v| v.as_i64())
        .ok_or(invalid_params("x"))?;
    let y = p
        .get("y")
        .and_then(|v| v.as_i64())
        .ok_or(invalid_params("y"))?;
    let z = p
        .get("z")
        .and_then(|v| v.as_i64())
        .ok_or(invalid_params("z"))?;
    let material = p
        .get("material")
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(0);
    Ok((x, y, z, material))
}

fn invalid_params(field: &str) -> JsonRpcError {
    JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: format!("Invalid params: missing or invalid `{field}`"),
        data: None,
    }
}

/// Set `result.entity_id` after `sim.spawn_civilian`.
pub fn set_spawn_civilian_result(response: &mut JsonRpcResponse, entity_id: u32) {
    if let Some(result) = response.result.as_mut() {
        if let Some(obj) = result.as_object_mut() {
            obj.insert("entity_id".to_owned(), serde_json::json!(entity_id));
            obj.insert("ok".to_owned(), serde_json::json!(true));
        }
    }
}

/// Set `result.tick` on a successful `sim.command` tick response.
pub fn set_sim_command_tick(response: &mut JsonRpcResponse, tick: u64) {
    if let Some(result) = response.result.as_mut() {
        if let Some(obj) = result.as_object_mut() {
            obj.insert("tick".to_owned(), serde_json::json!(tick));
        }
    }
}

fn parse_request_id(value: Value) -> Result<RequestId, JsonRpcParseError> {
    match value {
        Value::Null => Ok(RequestId::Null),
        Value::Number(n) => {
            n.as_i64()
                .map(RequestId::Number)
                .ok_or(JsonRpcParseError::InvalidRequest {
                    message: "Invalid Request",
                })
        }
        Value::String(s) => Ok(RequestId::String(s)),
        _ => Err(JsonRpcParseError::InvalidRequest {
            message: "Invalid Request",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_health_request() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"health","params":{}}"#)
            .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::Health);
        assert_eq!(req.id, RequestId::Number(1));
    }

    #[test]
    fn parse_sim_command_request() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":"abc","method":"sim.command","params":{"action":"noop"}}"#,
        )
        .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimCommand);
        assert_eq!(req.id, RequestId::String("abc".to_owned()));
    }

    #[test]
    fn parse_request_rejects_fractional_or_out_of_range_ids() {
        for raw in [
            r#"{"jsonrpc":"2.0","id":1.5,"method":"health"}"#,
            r#"{"jsonrpc":"2.0","id":9223372036854775808,"method":"health"}"#,
        ] {
            let err = parse_request(raw).expect_err("id should be rejected");
            assert_eq!(err.code(), error_code::INVALID_REQUEST);
        }
    }

    #[test]
    fn dispatch_sim_command_tick_plans_advance() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 7,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::AdvanceTick);
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "accepted": true, "tick": 7 }))
        );
    }

    #[test]
    fn parse_sim_save_replay_request() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":5,"method":"sim.save_replay","params":{"path":"/tmp/x.civreplay"}}"#,
        )
        .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimSaveReplay);
    }

    #[test]
    fn dispatch_sim_save_replay_plans_save_effect() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":6,"method":"sim.save_replay","params":{"path":"replay.civreplay"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(
            plan.effect,
            DispatchEffect::SaveReplay {
                path: "replay.civreplay".to_owned()
            }
        );
    }

    #[test]
    fn parse_sim_reset_request() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":10,"method":"sim.reset","params":{"seed":4242}}"#,
        )
        .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimReset);
    }

    #[test]
    fn dispatch_sim_reset_plans_reset_effect() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":11,"method":"sim.reset","params":{"seed":99}}"#)
                .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 42,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::ResetSimulation { seed: 99 });
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "seed": 99, "tick": 0 }))
        );
    }

    #[test]
    fn dispatch_sim_reset_rejects_missing_seed() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":12,"method":"sim.reset","params":{}}"#)
            .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 3,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::INVALID_PARAMS)
        );
    }

    #[test]
    fn dispatch_sim_load_replay_rejects_missing_path() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":8,"method":"sim.load_replay","params":{}}"#)
                .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 3,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::INVALID_PARAMS)
        );
    }

    #[test]
    fn dispatch_sim_command_tick_requires_operator_when_enforced() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: None,
                require_role: true,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::FORBIDDEN)
        );
    }

    #[test]
    fn dispatch_sim_command_tick_accepts_operator_in_params() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick","role":"operator"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: None,
                require_role: true,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::AdvanceTick);
    }

    #[test]
    fn dispatch_sim_command_tick_accepts_operator_from_connection_role() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: None,
                require_role: true,
                speed_multiplier: 1,
                connection_role: Some(OPERATOR_ROLE.to_owned()),
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::AdvanceTick);
    }

    #[test]
    fn parse_error_on_invalid_json() {
        let err = parse_request("{not json").unwrap_err();
        assert_eq!(err, JsonRpcParseError::Parse);
        assert_eq!(err.code(), error_code::PARSE_ERROR);
    }

    #[test]
    fn parse_error_on_wrong_jsonrpc_version() {
        let err = parse_request(r#"{"jsonrpc":"1.0","id":1,"method":"health"}"#).unwrap_err();
        assert!(matches!(err, JsonRpcParseError::InvalidRequest { .. }));
        assert_eq!(err.code(), error_code::INVALID_REQUEST);
    }

    #[test]
    fn parse_error_on_unknown_method() {
        let err = parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"sim.unknown"}"#).unwrap_err();
        assert!(matches!(err, JsonRpcParseError::MethodNotFound { .. }));
        assert_eq!(err.code(), error_code::METHOD_NOT_FOUND);
    }

    #[test]
    fn parse_error_on_batch_array() {
        let err = parse_request(r#"[{"jsonrpc":"2.0","id":1,"method":"health"}]"#).unwrap_err();
        assert!(matches!(err, JsonRpcParseError::InvalidRequest { .. }));
    }

    #[test]
    fn parse_error_response_invalid_json_uses_null_id() {
        let resp = parse_error_response("{not json", JsonRpcParseError::Parse);
        assert_eq!(resp.id, RequestId::Null);
        assert_eq!(
            resp.error.as_ref().map(|e| e.code),
            Some(error_code::PARSE_ERROR)
        );
    }

    #[test]
    fn parse_error_response_preserves_request_id() {
        let resp = parse_error_response(
            r#"{"jsonrpc":"2.0","id":"req-1","method":"sim.unknown"}"#,
            JsonRpcParseError::MethodNotFound {
                method: "sim.unknown".to_owned(),
            },
        );
        assert_eq!(resp.id, RequestId::String("req-1".to_owned()));
        assert_eq!(
            resp.error.as_ref().map(|e| e.code),
            Some(error_code::METHOD_NOT_FOUND)
        );
    }

    #[test]
    fn response_success_and_failure_shape() {
        use serde_json::json;
        let ok = JsonRpcResponse::success(RequestId::Null, json!({"v":1}));
        assert!(ok.result.is_some());
        assert!(ok.error.is_none());
        assert_eq!(ok.jsonrpc, JSONRPC_VERSION);
        let err = JsonRpcError {
            code: -32603,
            message: "boom".to_owned(),
            data: None,
        };
        let fail = JsonRpcResponse::failure(RequestId::Null, err);
        assert!(fail.result.is_none());
        assert!(fail.error.is_some());
        assert_eq!(fail.jsonrpc, ok.jsonrpc);
    }

    #[test]
    fn parse_error_codes_and_messages() {
        assert_eq!(JsonRpcParseError::Parse.code(), error_code::PARSE_ERROR);
        let inv = JsonRpcParseError::InvalidRequest { message: "bad" };
        assert_eq!(inv.code(), error_code::INVALID_REQUEST);
        let mnf = JsonRpcParseError::MethodNotFound {
            method: "sim.nope".to_owned(),
        };
        assert_eq!(mnf.code(), error_code::METHOD_NOT_FOUND);
        assert_eq!(JsonRpcParseError::Parse.message(), "Parse error");
        assert!(mnf.message().contains("sim.nope"));
        let e = JsonRpcParseError::Parse.into_error();
        assert_eq!(e.code, error_code::PARSE_ERROR);
        assert_eq!(e.message, "Parse error");
    }

    #[test]
    fn dispatch_sim_status_includes_population_when_available() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":3,"method":"sim.status","params":{}}"#)
            .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 42,
                population: Some(1_000_000),
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "tick": 42, "population": 1_000_000 }))
        );
    }

    #[test]
    fn dispatch_sim_status_omits_population_without_snapshot() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":4,"method":"sim.status"}"#).expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.response.result, Some(serde_json::json!({ "tick": 1 })));
    }

    /// `sim.emergence` returns the cached [`EmergenceSampleFields`] when
    /// the bridge has already populated [`DispatchContext::emergence`]
    /// (PR #350 stack). The wire contract is a flat JSON object with
    /// `tick`, `entropy_bits`, `entropy_norm`, and the structure bin
    /// counts — exactly the shape the dashboard tiles consume.
    #[test]
    fn dispatch_sim_emergence_returns_cached_sample() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":50,"method":"sim.emergence","params":{}}"#)
                .expect("parse");
        let sample = EmergenceSampleFields {
            tick: 50,
            entropy_bits: 0.3472,
            entropy_norm: 0.0820,
            structure_count: Some(3),
            structure_largest: Some(2048),
            structure_foreground: Some(4096),
            histogram_total: 4096,
            histogram_populated_bins: 4,
            sample_dur_us: 17,
            dashboard: Some(DashboardBlock {
                cluster_entropy: 0.97,
                ideology_homophily: 0.6,
                sentience_fraction: 0.4,
                psyche_stability: 0.8,
                diplomacy_tension: 0.1,
            }),
            branching_sigma: 0.95,
            branching_sigma_score: 0.71,
            branching_window: 10,
            avalanches_closed: 4,
            branching_regime: "Edge of chaos (target)".to_string(),
            power_law_alpha: 0.0,
            novelty_rate: 0.0,
            mi_material_faction_norm: None,
        };
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 50,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: Some(sample),
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        let result = plan.response.result.expect("result");
        assert_eq!(result["tick"], 50);
        // f32 round-trips through JSON as a number close to the
        // original; we assert within a small absolute tolerance
        // rather than exact equality.
        let entropy_bits = result["entropy_bits"].as_f64().expect("f64");
        assert!(
            (entropy_bits - 0.3472).abs() < 1e-5,
            "entropy_bits: {entropy_bits}"
        );
        let entropy_norm = result["entropy_norm"].as_f64().expect("f64");
        assert!(
            (entropy_norm - 0.0820).abs() < 1e-5,
            "entropy_norm: {entropy_norm}"
        );
        assert_eq!(result["structure_count"], 3);
        assert_eq!(result["structure_largest"], 2048);
        assert_eq!(result["structure_foreground"], 4096);
        assert_eq!(result["histogram_total"], 4096);
        assert_eq!(result["histogram_populated_bins"], 4);
        assert_eq!(result["sample_dur_us"], 17);
    }

    /// FR-CIV-EMERG-003: `sim.emergence` JSON-RPC response includes
    /// the five-tile `dashboard` block with the canonical field
    /// names. The wire shape is documented in
    /// `EmergenceSampleFields::dashboard`.
    #[test]
    fn emerg_emerg_003_sim_emergence_returns_dashboard_block() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":52,"method":"sim.emergence","params":{}}"#)
                .expect("parse");
        let sample = EmergenceSampleFields {
            tick: 100,
            entropy_bits: 0.5,
            entropy_norm: 0.25,
            structure_count: Some(2),
            structure_largest: Some(2048),
            structure_foreground: Some(2048),
            histogram_total: 4096,
            histogram_populated_bins: 5,
            sample_dur_us: 21,
            dashboard: Some(DashboardBlock {
                cluster_entropy: 0.97,
                ideology_homophily: 0.5,
                sentience_fraction: 0.4,
                psyche_stability: 0.8,
                diplomacy_tension: 0.1,
            }),
            branching_sigma: 0.95,
            branching_sigma_score: 0.71,
            branching_window: 10,
            avalanches_closed: 4,
            branching_regime: "Edge of chaos (target)".to_string(),
            power_law_alpha: 0.0,
            novelty_rate: 0.0,
            mi_material_faction_norm: None,
        };
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 100,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: Some(sample),
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        let result = plan.response.result.expect("result");
        let dashboard = &result["dashboard"];
        assert!(dashboard.is_object(), "dashboard must be an object");
        // f32 round-trips through JSON with a small precision loss;
        // we assert the parsed f64 is close to the input rather than
        // bit-equal so the test survives cross-platform JSON
        // number formatting.
        for (key, expected) in [
            ("cluster_entropy", 0.97_f64),
            ("ideology_homophily", 0.5),
            ("sentience_fraction", 0.4),
            ("psyche_stability", 0.8),
            ("diplomacy_tension", 0.1),
        ] {
            let value = dashboard[key].as_f64().unwrap_or(f64::NAN);
            assert!(
                (value - expected).abs() < 1e-5,
                "{key}: got {value}, expected {expected}"
            );
        }
    }

    /// `sim.emergence` on a fresh sim (ticks 0..49, no sample yet)
    /// returns `{ "tick": N, "sample": null }` so dashboard clients can
    /// distinguish "no data" from "entropy is exactly zero".
    #[test]
    fn dispatch_sim_emergence_returns_null_sample_before_first_boundary() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":51,"method":"sim.emergence"}"#).expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 12,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "tick": 12, "sample": serde_json::Value::Null }))
        );
    }

    #[test]
    fn parse_sim_snapshot_request() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":13,"method":"sim.snapshot","params":{}}"#)
            .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimSnapshot);
        assert_eq!(req.id, RequestId::Number(13));
    }

    #[test]
    fn snapshot_fields_from_sim_includes_spectator_pins() {
        let sim = civ_engine::Simulation::with_seed(7);
        let fields = snapshot_fields_from_sim(&sim, 1);
        assert!(fields
            .spectator
            .as_ref()
            .is_some_and(|s| !s.civ_pins.is_empty()));
        let json = snapshot_result_json(&fields);
        assert!(json.get("civ_pins").and_then(|v| v.as_array()).is_some());
    }

    #[test]
    fn snapshot_fields_from_sim_includes_damage_after_tick() {
        use civ_engine::DamageEvent;
        use civ_voxel::{MaterialId, WorldCoord};

        let mut sim = civ_engine::Simulation::with_seed(9);

        // Pre-fill solid voxels at and around the damage center so that
        // `apply_damage` has material to remove (an empty world returns 0).
        let cx = civ_voxel::FIXED_SCALE / 2;
        let cy = 0i64;
        let cz = civ_voxel::FIXED_SCALE / 4;
        for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                for dz in -1i64..=1 {
                    sim.push_voxel_write(
                        WorldCoord {
                            x: cx + dx,
                            y: cy + dy,
                            z: cz + dz,
                        },
                        MaterialId(1),
                    );
                }
            }
        }

        sim.push_damage(DamageEvent {
            center: WorldCoord {
                x: cx,
                y: cy,
                z: cz,
            },
            radius_voxels: 1,
            energy: 4,
        });
        sim.tick();
        let fields = snapshot_fields_from_sim(&sim, 1);
        assert_eq!(fields.damage_events_count, 1);
        assert_eq!(fields.damage_events.len(), 1);
        assert!(fields.voxel_damage_removed_this_tick > 0);
    }

    #[test]
    fn parse_spawn_civilian_params_accepts_normalized_coords() {
        let params = serde_json::json!({ "x": 0.25, "y": 0.75, "faction": 2 });
        let (x, y, faction) = parse_spawn_civilian_params(Some(&params)).expect("spawn params");
        assert!((x - 0.25).abs() < f32::EPSILON);
        assert!((y - 0.75).abs() < f32::EPSILON);
        assert_eq!(faction, 2);
    }

    #[test]
    fn parse_place_voxel_params_reads_coords() {
        let params = serde_json::json!({ "x": 1, "y": 2, "z": 3, "material": 4 });
        let (x, y, z, material) = parse_place_voxel_params(Some(&params)).expect("place params");
        assert_eq!((x, y, z, material), (1, 2, 3, 4));
    }

    #[test]
    fn parse_damage_params_reads_world_coords() {
        let params = serde_json::json!({ "x": 10, "y": 20, "z": 30, "radius": 4, "energy": 500 });
        let event = parse_damage_params(Some(&params)).expect("damage");
        assert_eq!(event.center.x, 10);
        assert_eq!(event.radius_voxels, 4);
        assert_eq!(event.energy, 500);
    }

    #[test]
    fn dispatch_sim_damage_schedules_apply() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":11,"method":"sim.damage","params":{"x":1,"y":2,"z":3,"radius":8}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 3,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert!(matches!(plan.effect, DispatchEffect::ApplyDamage { .. }));
    }

    #[test]
    fn parse_spawn_entity_params_accepts_palette_kinds() {
        let params = serde_json::json!({ "kind": "vehicle", "x": 0.1, "y": 0.9, "faction": 1 });
        let (kind, x, y, faction) = parse_spawn_entity_params(Some(&params)).expect("spawn entity");
        assert_eq!(kind, SpawnEntityKind::Vehicle);
        assert!((x - 0.1).abs() < f32::EPSILON);
        assert!((y - 0.9).abs() < f32::EPSILON);
        assert_eq!(faction, 1);

        let hangar = serde_json::json!({ "kind": "hangar", "x": 0.5, "y": 0.5 });
        let (kind, ..) = parse_spawn_entity_params(Some(&hangar)).expect("hangar");
        assert_eq!(kind, SpawnEntityKind::Hangar);
    }

    #[test]
    fn dispatch_sim_spawn_entity_schedules_vehicle() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":10,"method":"sim.spawn_entity","params":{"kind":"vehicle","x":0.4,"y":0.6,"faction":2}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 4,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert!(matches!(
            plan.effect,
            DispatchEffect::SpawnEntity {
                kind: SpawnEntityKind::Vehicle,
                ..
            }
        ));
    }

    #[test]
    fn snapshot_fields_include_military_pins_when_present() {
        let mut sim = civ_engine::Simulation::with_seed(3);
        civ_engine::spawn_military_at(&mut sim.world, 1, 0.25, 0.75, civ_engine::UnitType::Knight);
        let fields = snapshot_fields_from_sim(&sim, 1);
        assert!(
            fields
                .military_units
                .iter()
                .any(|unit| unit.unit_type == "Vehicle"),
            "expected spawned Knight pin as Vehicle"
        );
        let json = snapshot_result_json(&fields);
        assert!(json
            .get("military_units")
            .and_then(|v| v.as_array())
            .is_some());
    }

    #[test]
    fn dispatch_sim_spawn_civilian_schedules_effect() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":9,"method":"sim.spawn_civilian","params":{"x":0.5,"y":0.5,"faction":0}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 10,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert!(matches!(
            plan.effect,
            DispatchEffect::SpawnCivilian { faction: 0, .. }
        ));
    }

    #[test]
    fn snapshot_fields_from_sim_includes_institutions() {
        let sim = civ_engine::Simulation::with_seed(7);
        let fields = snapshot_fields_from_sim(&sim, 1);
        assert!(!fields.institutions.is_empty());
        let json = snapshot_result_json(&fields);
        let rows = json
            .get("institutions")
            .and_then(|v| v.as_array())
            .expect("institutions array");
        assert!(rows
            .iter()
            .any(|r| r.get("kind").and_then(|k| k.as_str()) == Some("market")));
        assert!(rows
            .iter()
            .any(|r| r.get("kind").and_then(|k| k.as_str()) == Some("treasury")));
    }

    #[test]
    fn dispatch_sim_snapshot_includes_snapshot_fields() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":14,"method":"sim.snapshot","params":{}}"#)
            .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 99,
                population: None,
                snapshot: Some(SnapshotFields {
                    tick: 42,
                    population: 1_000_000,
                    building_count: 7,
                    energy_budget: Some(1_000_000.0),
                    market_prices: BTreeMap::from([
                        ("food".to_string(), 1_000),
                        ("energy".to_string(), 1_250),
                    ]),
                    hash_chain_root: Some(
                        "a3f7c2b1e9d045f8a3f7c2b1e9d045f8a3f7c2b1e9d045f8a3f7c2b1e9d045f8"
                            .to_string(),
                    ),
                    speed_multiplier: 1,
                    spectator: None,
                    institutions: vec![],
                    military_units: vec![],
                    damage_events: vec![],
                    damage_events_count: 0,
                    voxel_damage_removed_this_tick: 0,
                    mods: vec![],
                    mod_lifecycle: vec![],
                    session_saved: vec![],
                    mod_permission_violations: vec![],
                    climate: civ_engine::Climate {
                        tick: 42,
                        day_phase: 0.0,
                        year_phase: 0.0,
                        moon_phase: 0.0,
                        tide_offset: 0.0,
                    },
                    emergence: None,
                    researched: vec![],
                    in_progress_tech: None,
                }),
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({
                "tick": 42,
                "population": 1_000_000,
                "building_count": 7,
                "energy_budget": 1_000_000.0,
                "market_prices": {
                    "food": 1_000,
                    "energy": 1_250,
                },
                "hash_chain_root":
                    "a3f7c2b1e9d045f8a3f7c2b1e9d045f8a3f7c2b1e9d045f8a3f7c2b1e9d045f8",
                "speed_multiplier": 1,
                "damage_events_count": 0,
                "voxel_damage_removed_this_tick": 0,
                "climate": {
                    "tick": 42,
                    "day_phase": 0.0,
                    "year_phase": 0.0,
                    "moon_phase": 0.0,
                    "tide_offset": 0.0,
                },
            }))
        );
    }

    #[test]
    fn dispatch_sim_snapshot_omits_hash_chain_root_when_unset() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":14,"method":"sim.snapshot","params":{}}"#)
            .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: Some(SnapshotFields {
                    tick: 1,
                    population: 500,
                    building_count: 2,
                    energy_budget: Some(100.0),
                    market_prices: BTreeMap::from([("food".to_string(), 1_000)]),
                    hash_chain_root: None,
                    speed_multiplier: 1,
                    spectator: None,
                    institutions: vec![],
                    military_units: vec![],
                    damage_events: vec![],
                    damage_events_count: 0,
                    voxel_damage_removed_this_tick: 0,
                    mods: vec![],
                    mod_lifecycle: vec![],
                    session_saved: vec![],
                    mod_permission_violations: vec![],
                    climate: civ_engine::Climate {
                        tick: 1,
                        day_phase: 0.0,
                        year_phase: 0.0,
                        moon_phase: 0.0,
                        tide_offset: 0.0,
                    },
                    emergence: None,
                    researched: vec![],
                    in_progress_tech: None,
                }),
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({
                "tick": 1,
                "population": 500,
                "building_count": 2,
                "energy_budget": 100.0,
                "market_prices": {
                    "food": 1_000,
                },
                "speed_multiplier": 1,
                "damage_events_count": 0,
                "voxel_damage_removed_this_tick": 0,
                "climate": {
                    "tick": 1,
                    "day_phase": 0.0,
                    "year_phase": 0.0,
                    "moon_phase": 0.0,
                    "tide_offset": 0.0,
                },
            }))
        );
        assert!(plan
            .response
            .result
            .as_ref()
            .and_then(|v| v.get("hash_chain_root"))
            .is_none());
    }

    #[test]
    fn dispatch_sim_snapshot_omits_energy_budget_when_unset() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":15,"method":"sim.snapshot","params":{}}"#)
            .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: Some(SnapshotFields {
                    tick: 1,
                    population: 500,
                    building_count: 2,
                    energy_budget: None,
                    market_prices: BTreeMap::from([
                        ("food".to_string(), 1_000),
                        ("energy".to_string(), 1_000),
                    ]),
                    hash_chain_root: None,
                    speed_multiplier: 1,
                    spectator: None,
                    institutions: vec![],
                    military_units: vec![],
                    damage_events: vec![],
                    damage_events_count: 0,
                    voxel_damage_removed_this_tick: 0,
                    mods: vec![],
                    mod_lifecycle: vec![],
                    session_saved: vec![],
                    mod_permission_violations: vec![],
                    climate: civ_engine::Climate {
                        tick: 1,
                        day_phase: 0.0,
                        year_phase: 0.0,
                        moon_phase: 0.0,
                        tide_offset: 0.0,
                    },
                    emergence: None,
                    researched: vec![],
                    in_progress_tech: None,
                }),
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({
                "tick": 1,
                "population": 500,
                "building_count": 2,
                "market_prices": {
                    "food": 1_000,
                    "energy": 1_000,
                },
                "speed_multiplier": 1,
                "damage_events_count": 0,
                "voxel_damage_removed_this_tick": 0,
                "climate": {
                    "tick": 1,
                    "day_phase": 0.0,
                    "year_phase": 0.0,
                    "moon_phase": 0.0,
                    "tide_offset": 0.0,
                },
            }))
        );
    }

    #[test]
    fn dispatch_sim_snapshot_falls_back_to_bridge_tick_without_snapshot() {
        let req = parse_request(r#"{"jsonrpc":"2.0","id":16,"method":"sim.snapshot","params":{}}"#)
            .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 5,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "tick": 5, "speed_multiplier": 1 }))
        );
    }

    #[test]
    fn parse_sim_set_policy_request() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":17,"method":"sim.set_policy","params":{"scarcity_multiplier":0.0}}"#,
        )
        .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimSetPolicy);
    }

    #[test]
    fn dispatch_sim_set_policy_plans_set_policy_effect() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":18,"method":"sim.set_policy","params":{"scarcity_multiplier":2.5,"base_consumption_joules":1000}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(
            plan.effect,
            DispatchEffect::SetPolicy {
                scarcity_multiplier: 2.5,
                base_consumption_joules: Some(1000),
            }
        );
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "updated": true, "scarcity_multiplier": 2.5 }))
        );
    }

    #[test]
    fn dispatch_sim_set_policy_rejects_negative_scarcity() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":19,"method":"sim.set_policy","params":{"scarcity_multiplier":-1.0}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::INVALID_PARAMS)
        );
    }

    #[test]
    fn dispatch_sim_set_policy_rejects_missing_scarcity() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":20,"method":"sim.set_policy","params":{}}"#)
                .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::INVALID_PARAMS)
        );
    }

    #[test]
    fn parse_set_policy_params_rejects_nan_scarcity() {
        let params = serde_json::json!({ "scarcity_multiplier": f64::NAN });
        let err = parse_set_policy_params(Some(&params)).unwrap_err();
        assert_eq!(err.code, error_code::INVALID_PARAMS);
    }

    #[test]
    fn parse_set_policy_params_rejects_infinite_scarcity() {
        for value in [f64::INFINITY, f64::NEG_INFINITY] {
            let params = serde_json::json!({ "scarcity_multiplier": value });
            let err = parse_set_policy_params(Some(&params)).unwrap_err();
            assert_eq!(err.code, error_code::INVALID_PARAMS, "value {value}");
        }
    }

    #[test]
    fn parse_set_policy_params_rejects_non_numeric_scarcity() {
        let params = serde_json::json!({ "scarcity_multiplier": "1.5" });
        let err = parse_set_policy_params(Some(&params)).unwrap_err();
        assert_eq!(err.code, error_code::INVALID_PARAMS);
    }

    #[test]
    fn parse_set_policy_params_rejects_invalid_base_consumption_joules() {
        let params = serde_json::json!({
            "scarcity_multiplier": 1.0,
            "base_consumption_joules": -1
        });
        let err = parse_set_policy_params(Some(&params)).unwrap_err();
        assert_eq!(err.code, error_code::INVALID_PARAMS);
    }

    #[test]
    fn parse_set_policy_params_accepts_zero_scarcity_without_base_consumption() {
        let params = serde_json::json!({ "scarcity_multiplier": 0.0 });
        let parsed = parse_set_policy_params(Some(&params)).expect("parse");
        assert_eq!(parsed.scarcity_multiplier, 0.0);
        assert_eq!(parsed.base_consumption_joules, None);
    }

    #[test]
    fn dispatch_sim_command_noop_allowed_without_operator_when_enforced() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":4,"method":"sim.command","params":{"action":"noop"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: true,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert!(plan.response.error.is_none());
    }

    #[test]
    fn role_allows_operator_tick_rejects_non_operator_role() {
        assert!(!role_allows_operator_tick(
            true,
            Some(&serde_json::json!({ "action": "tick", "role": "viewer" })),
            None,
        ));
    }

    #[test]
    fn role_allows_operator_tick_param_role_overrides_connection_role() {
        assert!(!role_allows_operator_tick(
            true,
            Some(&serde_json::json!({ "action": "tick", "role": "viewer" })),
            Some(OPERATOR_ROLE),
        ));
        assert!(role_allows_operator_tick(
            true,
            Some(&serde_json::json!({ "action": "tick", "role": "operator" })),
            Some("viewer"),
        ));
    }

    #[test]
    fn forbidden_operator_role_error_includes_required_role_data() {
        let err = forbidden_operator_role_error();
        assert_eq!(err.code, error_code::FORBIDDEN);
        assert_eq!(
            err.data,
            Some(serde_json::json!({ "required_role": OPERATOR_ROLE }))
        );
    }

    #[test]
    fn parse_sim_set_speed_request() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":21,"method":"sim.set_speed","params":{"multiplier":2}}"#,
        )
        .expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimSetSpeed);
    }

    #[test]
    fn dispatch_sim_set_speed_plans_set_speed_effect() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":22,"method":"sim.set_speed","params":{"multiplier":4}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::SetSpeed { multiplier: 4 });
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "accepted": true, "multiplier": 4 }))
        );
    }

    #[test]
    fn dispatch_sim_set_speed_rejects_invalid_multiplier() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":23,"method":"sim.set_speed","params":{"multiplier":3}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::INVALID_PARAMS)
        );
    }

    #[test]
    fn parse_sim_get_speed_request() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":24,"method":"sim.get_speed"}"#).expect("parse");
        assert_eq!(req.method, JsonRpcMethod::SimGetSpeed);
    }

    #[test]
    fn dispatch_sim_get_speed_returns_multiplier_from_context() {
        let req =
            parse_request(r#"{"jsonrpc":"2.0","id":25,"method":"sim.get_speed"}"#).expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 4,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "multiplier": 4 }))
        );
    }

    /// Covers FR-CIV-SAVE-001.
    /// Covers FR-CIV-SAVE-002.
    /// Covers FR-CIV-TACTICS-066.
    /// FR-CIV-TACTICS-066 — `save.slot` is a JSON-RPC method dispatched on
    /// the server bridge; the planner emits a `DispatchEffect::SaveSlot`
    /// with the slot name from the request params.
    #[test]
    fn dispatch_save_slot_plans_save_effect_fr_save_002() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":30,"method":"save.slot","params":{"slot_name":"slot-2"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 1,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(
            plan.effect,
            DispatchEffect::SaveSlot {
                slot_name: "slot-2".to_owned()
            }
        );
    }

    /// Covers FR-CIV-SAVE-001.
    /// Covers FR-CIV-SAVE-002.
    /// Covers FR-CIV-TACTICS-066.
    #[test]
    fn dispatch_save_slot_rejects_invalid_slot_fr_save_002() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":31,"method":"save.slot","params":{"slot_name":"slot-9"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.error.as_ref().map(|e| e.code),
            Some(error_code::INVALID_PARAMS)
        );
    }

    #[test]
    fn parse_reset_seed_extracts_u64() {
        use serde_json::json;
        assert_eq!(parse_reset_seed(Some(&json!({"seed": 42}))).unwrap(), 42);
        // seed is optional — None params and missing key both default to 0.
        assert_eq!(parse_reset_seed(None).unwrap(), 0);
        assert_eq!(parse_reset_seed(Some(&json!({}))).unwrap(), 0);
        // A present but non-integer seed is still an error.
        assert!(parse_reset_seed(Some(&json!({"seed": "x"}))).is_err());
    }

    #[test]
    fn parse_set_speed_params_validates_multiplier() {
        use serde_json::json;
        assert_eq!(
            parse_set_speed_params(Some(&json!({"multiplier": 2}))).unwrap(),
            2
        );
        assert_eq!(
            parse_set_speed_params(Some(&json!({"multiplier": 0}))).unwrap(),
            0
        );
        assert!(parse_set_speed_params(Some(&json!({"multiplier": 3}))).is_err());
        assert!(parse_set_speed_params(None).is_err());
        assert!(parse_set_speed_params(Some(&json!({"multiplier": "x"}))).is_err());
    }

    #[test]
    fn parse_role_param_reads_nonempty_role() {
        use serde_json::json;
        assert_eq!(
            parse_role_param(Some(&json!({"role":"operator"}))),
            Some("operator".to_string())
        );
        assert_eq!(parse_role_param(Some(&json!({"role":""}))), None);
        assert_eq!(parse_role_param(None), None);
        assert_eq!(parse_role_param(Some(&json!({}))), None);
    }

    #[test]
    fn role_allows_operator_enforces_when_required() {
        use serde_json::json;
        assert!(role_allows_operator(false, None, None));
        assert!(role_allows_operator(
            true,
            Some(&json!({"role":"operator"})),
            None
        ));
        assert!(role_allows_operator(true, None, Some("operator")));
        assert!(!role_allows_operator(true, None, None));
        assert!(!role_allows_operator(
            true,
            Some(&json!({"role":"viewer"})),
            None
        ));
    }

    #[test]
    fn parse_name_maps_all_known_methods() {
        assert_eq!(
            JsonRpcMethod::parse_name("health"),
            Some(JsonRpcMethod::Health)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.command"),
            Some(JsonRpcMethod::SimCommand)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.status"),
            Some(JsonRpcMethod::SimStatus)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.reset"),
            Some(JsonRpcMethod::SimReset)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.set_speed"),
            Some(JsonRpcMethod::SimSetSpeed)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("save.slot"),
            Some(JsonRpcMethod::SaveSlot)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("save.load"),
            Some(JsonRpcMethod::LoadSlot)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("save.list"),
            Some(JsonRpcMethod::SaveList)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.emergence"),
            Some(JsonRpcMethod::SimEmergence)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.snapshot"),
            Some(JsonRpcMethod::SimSnapshot)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.spawn_entity"),
            Some(JsonRpcMethod::SimSpawnEntity)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.subscribe"),
            Some(JsonRpcMethod::SimSubscribe)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.unsubscribe"),
            Some(JsonRpcMethod::SimUnsubscribe)
        );
        assert_eq!(
            JsonRpcMethod::parse_name("sim.update_subscription"),
            Some(JsonRpcMethod::SimUpdateSubscription)
        );
        assert_eq!(JsonRpcMethod::parse_name("unknown.method"), None);
        assert_eq!(JsonRpcMethod::parse_name(""), None);
    }

    #[test]
    fn parse_replay_path_extracts_nonempty() {
        use serde_json::json;
        assert_eq!(
            parse_replay_path(Some(&json!({"path":"saves/x.civreplay"}))).unwrap(),
            "saves/x.civreplay"
        );
        assert!(parse_replay_path(None).is_err());
        assert!(parse_replay_path(Some(&json!({}))).is_err());
        assert!(parse_replay_path(Some(&json!({"path":""}))).is_err());
        assert!(parse_replay_path(Some(&json!({"path":"/tmp/x.civreplay"}))).is_err());
        assert!(parse_replay_path(Some(&json!({"path":"../x.civreplay"}))).is_err());
    }

    #[test]
    fn parse_replay_path_rejects_absolute_paths() {
        use serde_json::json;
        // Linux/macOS absolute path.
        assert!(parse_replay_path(Some(&json!({"path":"/etc/passwd"}))).is_err());
        // Windows-style absolute paths with drive prefix.
        assert!(
            parse_replay_path(Some(&json!({"path":"C:\\Windows\\System32\\drivers\\etc\\hosts"})))
                .is_err()
        );
        // Backslash-leading UNC-style path is rejected.
        assert!(parse_replay_path(Some(&json!({"path":"\\\\server\\share\\file"}))).is_err());
        // Just the root.
        assert!(parse_replay_path(Some(&json!({"path":"/"}))).is_err());
    }

    #[test]
    fn parse_replay_path_rejects_parent_segments() {
        use serde_json::json;
        // Single `..` segment.
        assert!(parse_replay_path(Some(&json!({"path":"../etc/passwd"}))).is_err());
        // Embedded `..` segment.
        assert!(parse_replay_path(Some(&json!({"path":"replays/../../etc/passwd"}))).is_err());
        // Trailing `..`.
        assert!(parse_replay_path(Some(&json!({"path":"replays/.."}))).is_err());
        // `..` as the entire (non-empty) path.
        assert!(parse_replay_path(Some(&json!({"path":".."}))).is_err());
    }

    #[test]
    fn parse_replay_path_rejects_windows_drive_prefix() {
        use serde_json::json;
        // `C:` alone or with relative path is rejected because of the
        // Prefix component.
        assert!(parse_replay_path(Some(&json!({"path":"C:replays/x.civreplay"}))).is_err());
        assert!(parse_replay_path(Some(&json!({"path":"D:\\foo"}))).is_err());
    }

    #[test]
    fn parse_replay_path_accepts_safe_relative_paths() {
        use serde_json::json;
        assert_eq!(
            parse_replay_path(Some(&json!({"path":"replays/x.civreplay"}))).unwrap(),
            "replays/x.civreplay"
        );
        assert_eq!(
            parse_replay_path(Some(&json!({"path":"a/b/c/x.civreplay"}))).unwrap(),
            "a/b/c/x.civreplay"
        );
        // A single filename (no separator) is fine.
        assert_eq!(
            parse_replay_path(Some(&json!({"path":"x.civreplay"}))).unwrap(),
            "x.civreplay"
        );
    }

    #[test]
    fn resolve_replay_path_contains_existing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().join("replays");
        std::fs::create_dir_all(&base).expect("create base");
        let file = base.join("x.civreplay");
        std::fs::write(&file, b"replay-bytes").expect("write file");
        let resolved = resolve_replay_path(&base, "x.civreplay").expect("resolve");
        // Canonicalize both before comparing (symlink-resolved on macOS).
        assert_eq!(
            std::fs::canonicalize(&resolved).unwrap(),
            std::fs::canonicalize(&file).unwrap()
        );
    }

    #[test]
    fn resolve_replay_path_resolves_new_file_under_base() {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().join("replays");
        std::fs::create_dir_all(&base).expect("create base");
        // File does not exist yet — `save_replay` will create it.
        let resolved = resolve_replay_path(&base, "new.civreplay").expect("resolve");
        let base_canonical = std::fs::canonicalize(&base).unwrap();
        assert!(resolved.starts_with(&base_canonical));
        assert!(resolved.ends_with("new.civreplay"));
    }

    #[test]
    fn resolve_replay_path_rejects_escape_via_symlink() {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().join("replays");
        std::fs::create_dir_all(&base).expect("create base");
        // Create an out-of-base target file.
        let outside = dir.path().join("outside.civreplay");
        std::fs::write(&outside, b"out-of-base").expect("write outside");
        // Inside the base, plant a symlink that points outside.
        let link = base.join("escape.civreplay");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside, &link).expect("symlink");
        // The relative path resolves via the symlink to a file outside
        // the canonicalized base — containment check must reject it.
        let result = resolve_replay_path(&base, "escape.civreplay");
        assert!(result.is_err(), "expected escape rejection, got {result:?}");
    }

    #[test]
    fn parse_sim_command_action_maps_noop_tick() {
        use serde_json::json;
        assert!(matches!(
            parse_sim_command_action(Some(&json!({"action":"noop"}))),
            Some(SimCommandAction::Noop)
        ));
        assert!(matches!(
            parse_sim_command_action(Some(&json!({"action":"tick"}))),
            Some(SimCommandAction::Tick)
        ));
        assert!(parse_sim_command_action(Some(&json!({"action":"bogus"}))).is_none());
        assert!(parse_sim_command_action(Some(&json!({}))).is_none());
        assert!(parse_sim_command_action(None).is_none());
    }

    #[test]
    fn spawn_entity_kind_wire_labels() {
        assert_eq!(SpawnEntityKind::Civilian.wire_label(), "civilian");
        assert_eq!(SpawnEntityKind::Vehicle.wire_label(), "vehicle");
        assert_eq!(SpawnEntityKind::Airport.wire_label(), "airport");
        assert_eq!(SpawnEntityKind::Port.wire_label(), "port");
        assert_eq!(SpawnEntityKind::Hangar.wire_label(), "hangar");
    }

    #[test]
    fn set_sim_command_tick_inserts_tick_into_object_result() {
        use serde_json::json;
        let mut resp = JsonRpcResponse::success(RequestId::Null, json!({"status":"ok"}));
        set_sim_command_tick(&mut resp, 99);
        assert_eq!(
            resp.result.as_ref().unwrap().get("tick").unwrap(),
            &json!(99)
        );
        let mut resp2 = JsonRpcResponse::success(RequestId::Null, json!("plain-string"));
        set_sim_command_tick(&mut resp2, 5);
        assert_eq!(resp2.result.as_ref().unwrap(), &json!("plain-string"));
    }

    /// Criticality metrics are carried through From<EmergenceSample> for EmergenceSampleFields.
    #[test]
    fn emergence_sample_fields_from_carries_criticality_metrics() {
        use civ_emergence_metrics::branching::BranchingRegime;
        use civ_emergence_metrics::dashboard::EmergenceDashboard;
        use civ_engine::emergence_metrics::EmergenceSample;

        let sample = EmergenceSample {
            tick: 1,
            entropy_bits: 0.0,
            entropy_norm: 0.0,
            structure_count: None,
            structure_largest: None,
            structure_foreground: None,
            histogram_total: 0,
            histogram_populated_bins: 0,
            sample_dur_us: 0,
            dashboard: EmergenceDashboard::default(),
            branching_sigma: 0.0,
            branching_sigma_score: 0.0,
            branching_window: 0,
            avalanches_closed: 0,
            branching_regime: BranchingRegime::HeatDeath,
            power_law_alpha: 1.95,
            novelty_rate: 0.008,
            mi_material_faction_norm: Some(0.33),
        };
        let fields = EmergenceSampleFields::from(sample);
        assert!(
            (fields.power_law_alpha - 1.95).abs() < 1e-6,
            "power_law_alpha mismatch"
        );
        assert!(
            (fields.novelty_rate - 0.008).abs() < 1e-6,
            "novelty_rate mismatch"
        );
        assert_eq!(fields.mi_material_faction_norm, Some(0.33));
    }

    /// snapshot_result_json includes power_law_alpha, novelty_rate, mi_material_faction_norm
    /// in the emergence block.
    #[test]
    fn snapshot_result_json_emergence_contains_criticality_keys() {
        let emergence = EmergenceSampleFields {
            tick: 5,
            entropy_bits: 0.1,
            entropy_norm: 0.05,
            structure_count: None,
            structure_largest: None,
            structure_foreground: None,
            histogram_total: 100,
            histogram_populated_bins: 3,
            sample_dur_us: 10,
            dashboard: None,
            branching_sigma: 1.0,
            branching_sigma_score: 0.5,
            branching_window: 5,
            avalanches_closed: 2,
            branching_regime: "Edge of chaos (target)".to_string(),
            power_law_alpha: 2.1,
            novelty_rate: 0.005,
            mi_material_faction_norm: Some(0.42),
        };
        let fields = SnapshotFields {
            tick: 5,
            population: 0,
            building_count: 0,
            energy_budget: None,
            market_prices: Default::default(),
            hash_chain_root: None,
            speed_multiplier: 1,
            spectator: None,
            institutions: vec![],
            military_units: vec![],
            damage_events: vec![],
            damage_events_count: 0,
            voxel_damage_removed_this_tick: 0,
            mods: vec![],
            mod_lifecycle: vec![],
            session_saved: vec![],
            mod_permission_violations: vec![],
            climate: civ_engine::Climate {
                tick: 5,
                day_phase: 0.0,
                year_phase: 0.0,
                moon_phase: 0.0,
                tide_offset: 0.0,
            },
            emergence: Some(emergence),
            researched: vec![],
            in_progress_tech: None,
        };
        let json = snapshot_result_json(&fields);
        let emerg = json.get("emergence").expect("emergence block");
        assert!(
            emerg.get("power_law_alpha").is_some(),
            "missing power_law_alpha"
        );
        assert!(emerg.get("novelty_rate").is_some(), "missing novelty_rate");
        assert!(
            emerg.get("mi_material_faction_norm").is_some(),
            "missing mi_material_faction_norm"
        );
        let mi = emerg["mi_material_faction_norm"]
            .as_f64()
            .expect("mi_material_faction_norm f64");
        assert!((mi - 0.42).abs() < 1e-5);
    }
    /// FR-CIV-SERVER-003 — sim.queue_research with a valid tech accepts the request.
    #[test]
    fn dispatch_queue_research_valid_tech_accepted() {
        use serde_json::json;
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":1,"method":"sim.queue_research","params":{"tech":"pottery"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 10,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        let res = plan.response.result.expect("result");
        assert_eq!(res["queued"], "pottery");
    }

    /// FR-CIV-SERVER-003 — sim.queue_research with an unknown tech returns INVALID_PARAMS.
    #[test]
    fn dispatch_queue_research_unknown_tech_rejected() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.queue_research","params":{"tech":"unobtainium"}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 0,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(plan.response.error.as_ref().map(|e| e.code), Some(error_code::INVALID_PARAMS));
    }

    /// FR-CIV-SERVER-003 — sim.tech_state returns available list and stub flag.
    #[test]
    fn dispatch_tech_state_returns_available_list() {
        let req = parse_request(
            r#"{"jsonrpc":"2.0","id":3,"method":"sim.tech_state","params":{}}"#,
        )
        .expect("parse");
        let plan = dispatch_request(
            req,
            DispatchContext {
                tick: 42,
                population: None,
                snapshot: None,
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
                saves_dir: None,
                emergence: None,
                researched: vec![],
                in_progress_tech: None,
                last_tick_ms: 0.0,
                outcome_fields: None,
            },
        );
        assert!(plan.response.result.is_some(), "expected success");
        let result = plan.response.result.unwrap();
        let available = result.get("available").expect("available field");
        assert!(available.is_array(), "available should be an array");
        assert!(available.as_array().unwrap().len() > 0, "available should be non-empty");
    }
}
