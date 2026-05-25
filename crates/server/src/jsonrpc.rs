//! JSON-RPC 2.0 request/response types for the CIV-0200 WebSocket protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

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
            Self::SimSetPolicy => "sim.set_policy",
            Self::SimSetSpeed => "sim.set_speed",
            Self::SimGetSpeed => "sim.get_speed",
            Self::SimSpawnCivilian => "sim.spawn_civilian",
            Self::SimSpawnEntity => "sim.spawn_entity",
            Self::SimPlaceVoxel => "sim.place_voxel",
            Self::SimDamage => "sim.damage",
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
            "sim.set_policy" => Some(Self::SimSetPolicy),
            "sim.set_speed" => Some(Self::SimSetSpeed),
            "sim.get_speed" => Some(Self::SimGetSpeed),
            "sim.spawn_civilian" => Some(Self::SimSpawnCivilian),
            "sim.spawn_entity" => Some(Self::SimSpawnEntity),
            "sim.place_voxel" => Some(Self::SimPlaceVoxel),
            "sim.damage" => Some(Self::SimDamage),
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(default)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InstitutionSnapshot {
    /// Institution id.
    pub id: u32,
    /// `market` or `treasury`.
    pub kind: &'static str,
    /// Joule balance.
    pub balance_joules: i64,
}

/// Snapshot fields from `Simulation::snapshot()` for read-only RPC handlers.
#[derive(Debug, Clone, PartialEq)]
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
}

/// Military pin row for `sim.snapshot` (matches civ-watch wire shape).
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    Value::Object(obj)
}

/// Map ECS military units to spectator pins.
pub fn military_pins_from_sim(sim: &civ_engine::Simulation) -> Vec<MilitaryPinSnapshot> {
    use civ_engine::{grid_to_norm, unit_type_label, MilitaryUnit};
    sim.world
        .query::<&MilitaryUnit>()
        .iter()
        .enumerate()
        .map(|(idx, (entity, unit))| {
            let (x, y) = grid_to_norm(unit.position);
            MilitaryPinSnapshot {
                id: entity.to_bits().get() ^ u64::from(idx as u32),
                x,
                y,
                unit_type: unit_type_label(unit.unit_type).to_string(),
                faction: unit.faction_id,
                strength: unit.strength.to_f64() as f32,
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
                InstitutionKind::Market => "market",
                InstitutionKind::Treasury => "treasury",
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
    }
}

/// Tick and optional snapshot fields passed into dispatch.
#[derive(Debug, Clone, PartialEq)]
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

/// Parse `sim.reset` `seed` param.
pub fn parse_reset_seed(params: Option<&Value>) -> Result<u64, JsonRpcError> {
    params
        .and_then(|p| p.get("seed"))
        .and_then(|v| v.as_u64())
        .ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected unsigned integer \"seed\"".to_owned(),
            data: None,
        })
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
pub fn parse_replay_path(params: Option<&Value>) -> Result<String, JsonRpcError> {
    let path = params
        .and_then(|p| p.get("path"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected non-empty string \"path\"".to_owned(),
            data: None,
        })?;
    Ok(path.to_owned())
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
}

impl SpawnEntityKind {
    /// JSON `kind` string echoed in RPC results.
    pub fn wire_label(self) -> &'static str {
        match self {
            Self::Civilian => "civilian",
            Self::Vehicle => "vehicle",
            Self::Airport => "airport",
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
        _ => {
            return Err(JsonRpcError {
                code: error_code::INVALID_PARAMS,
                message: "kind must be civilian, vehicle, or airport".to_owned(),
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
            },
        );
        assert_eq!(plan.response.result, Some(serde_json::json!({ "tick": 1 })));
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
                }),
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
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
                }),
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
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
                }),
                require_role: false,
                speed_multiplier: 1,
                connection_role: None,
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
            },
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert_eq!(
            plan.response.result,
            Some(serde_json::json!({ "multiplier": 4 }))
        );
    }
}
