//! civ-server library — exposes the 3D-extension protocol bridge that consumers
//! (renderers, replay tools) use to convert `Simulation::last_tick_voxel_events`
//! into `civ-protocol-3d` frames.
//!
//! The eventual WebSocket bridge lives here too; for now this crate ships the
//! frame builders and a binary that prints determinism metrics.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod autosave;
pub mod jsonrpc;
pub mod saves;
pub mod subscription_filter;
pub mod voxel_frame_builder;
/// WebSocket bridge and health endpoint for streaming 3D protocol frames.
pub mod ws_bridge;

pub use autosave::{
    autosave_cadence_from_env, autosave_filename_for_tick, autosave_keep_from_env,
    run_autosave_once, spawn_autosave_loop, AutosaveContext, AutosaveResult, DEFAULT_AUTOSAVE_KEEP,
};
pub use jsonrpc::{
    dispatch_request, encode_response, error_code, forbidden_operator_role_error,
    parse_error_response, parse_replay_path, parse_request, parse_reset_seed, parse_role_param,
    parse_sim_command_action, resolve_replay_path, role_allows_operator_tick,
    set_sim_command_tick, snapshot_fields_from_sim, DispatchContext, DispatchEffect,
    DispatchPlan, EmergenceSampleFields, JsonRpcError, JsonRpcMethod, JsonRpcParseError,
    JsonRpcRequest, JsonRpcResponse, RequestId, SimCommandAction, SnapshotFields,
    JSONRPC_VERSION, OPERATOR_ROLE,
};
pub use saves::{
    list_saves, most_recent_save_path, save_archive_path, validate_production_slot, SaveListEntry,
};
pub use voxel_frame_builder::{build_voxel_delta_frame, VoxelFrameBuilderError};
pub use ws_bridge::{
    run_ws_bridge, spawn_ws_bridge, spawn_ws_bridge_with_config, TickBroadcastFormat,
    WsBridgeConfig,
};
