//! FR-CIV-MCP-001 / 003 — civis-mcp tool surface and request contract.
//!
//! Requirement text (`agileplus-specs/civ-017-civis-mcp-server/spec.md:33-45`):
//!
//! - FR-CIV-MCP-001: "The `civis-mcp` crate SHALL expose an MCP server binary
//!   (`civis-mcp`) that registers a tool for every JSON-RPC method listed in
//!   `docs/api/jsonrpc-surface.md` (14 methods today) and SHALL forward each
//!   tool call to `civ-server` over the existing WebSocket surface."
//! - FR-CIV-MCP-003: "Each tool SHALL be covered by a contract test asserting
//!   the request envelope shape, the response shape, and the error mapping
//!   (e.g. JSON-RPC -32601 for unknown methods)."
//!
//! Replaces placeholders (`crates/civis-mcp/tests/fr_fr_civ_mcp_001.rs` and
//! `_003.rs`) whose entire body was `assert_eq!(SCHEMA_VERSION, 0); let _ =
//! AiConfig::default();` — a check on `civ-ai` types, in the wrong crate
//! entirely, and identical in both files.
//!
//! ## Measured state of 001
//!
//! The requirement's "(14 methods today)" is a stale parenthetical: the surface
//! doc now declares `## Method catalog (42)`. The registered tool set has grown
//! past the requirement's floor, which is what the count assertion below pins.
//!
//! ## What 003 can and cannot assert headlessly
//!
//! The **request envelope** is built locally by `build_rpc_request`, so it is
//! asserted directly. The **response shape** and the **error mapping**
//! (`-32601` for an unknown method) are produced by `civ-server` on the far side
//! of a WebSocket, so they cannot be observed without a live server; the
//! `dispatch_rpc_method` path opens a real connection. Those are recorded as
//! requiring an integration harness rather than faked here.

use civis_mcp::{build_rpc_request, tool_names};

/// JSON-RPC methods the MCP layer is required to expose a tool for, with the
/// tool name that carries them. Taken from `docs/api/jsonrpc-surface.md`; a
/// representative cross-section of every method family (health, sim read,
/// sim write, save).
const REQUIRED_METHOD_TOOLS: [(&str, &str); 6] = [
    ("health", "civis_health"),
    ("sim.snapshot", "civis_snapshot"),
    ("sim.emergence", "civis_emergence"),
    ("sim.command", "civis_sim_command"),
    ("sim.reset", "civis_reset"),
    ("save.slot", "civis_save_slot"),
];

// ---------------------------------------------------------------------------
// FR-CIV-MCP-001 — a tool per JSON-RPC method
// ---------------------------------------------------------------------------

/// Covers FR-CIV-MCP-001.
///
/// Every method in the representative cross-section must have a registered
/// tool, or that method is unreachable from an MCP client.
#[test]
fn fr_civ_mcp_001_representative_methods_have_registered_tools() {
    let names = tool_names();
    assert!(!names.is_empty(), "the MCP server must register tools");

    for (method, tool) in REQUIRED_METHOD_TOOLS {
        assert!(
            names.iter().any(|n| n == tool),
            "JSON-RPC method `{method}` has no registered tool `{tool}`; \
             registered tools: {names:?}"
        );
    }
}

/// Covers FR-CIV-MCP-001.
///
/// The registered set must clear the requirement's own stated floor of 14
/// methods. A regression that dropped the surface below that would break the
/// contract the requirement codifies.
#[test]
fn fr_civ_mcp_001_tool_surface_meets_the_required_floor() {
    let names = tool_names();
    assert!(
        names.len() >= 14,
        "the spec records a floor of 14 methods; only {} tools are registered: {names:?}",
        names.len()
    );
}

/// Covers FR-CIV-MCP-001.
///
/// `tool_names` documents itself as sorted lexicographic and the router keys on
/// the name, so the surface must be sorted and free of duplicates. A duplicate
/// would make tool dispatch ambiguous.
#[test]
fn fr_civ_mcp_001_tool_names_are_sorted_and_unique() {
    let names = tool_names();

    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "tool_names must be sorted lexicographically");

    let mut deduped = names.clone();
    deduped.dedup();
    assert_eq!(
        names.len(),
        deduped.len(),
        "tool names must be unique; duplicates make dispatch ambiguous: {names:?}"
    );
}

/// Covers FR-CIV-MCP-001.
///
/// Every registered tool name must be a valid MCP identifier so clients can
/// address it: non-empty, lowercase ASCII letters/digits/underscore only, and
/// not starting with a digit or underscore.
///
/// The surface deliberately spans two prefixes — `civis_*` for the tools that
/// forward to `civ-server`, and `sim_*` for the simulation-oriented group — so
/// this asserts name *validity*, not a single namespace. (An earlier version of
/// this test wrongly required every forwarding tool to be `civis_`-prefixed,
/// which the real surface does not do.)
#[test]
fn fr_civ_mcp_001_tool_names_are_valid_mcp_identifiers() {
    let names = tool_names();
    assert!(!names.is_empty(), "premise: the server registers tools");

    for name in &names {
        assert!(!name.is_empty(), "a tool name must not be empty");
        assert!(
            !name.starts_with(|c: char| c.is_ascii_digit() || c == '_'),
            "tool `{name}` must not start with a digit or underscore"
        );
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "tool `{name}` contains characters outside [a-z0-9_], which MCP \
             clients may not be able to address"
        );
    }

    // The two known prefixes must both be present; if one vanishes, tools have
    // been renamed out from under clients.
    assert!(
        names.iter().any(|n| n.starts_with("civis_")),
        "expected at least one `civis_`-prefixed tool: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.starts_with("sim_")),
        "expected at least one `sim_`-prefixed tool: {names:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-MCP-003 — request envelope contract
// ---------------------------------------------------------------------------

/// Covers FR-CIV-MCP-003.
///
/// The request envelope must be JSON-RPC 2.0 with a pinned id, the requested
/// method, and the caller's params verbatim. `civ-server` rejects anything else.
#[test]
fn fr_civ_mcp_003_request_envelope_shape_is_jsonrpc_2() {
    let frame = build_rpc_request("sim.snapshot", serde_json::json!({ "tick": 7 }));

    let parsed: serde_json::Value =
        serde_json::from_str(&frame).expect("the envelope must be valid JSON");

    assert_eq!(
        parsed["jsonrpc"], "2.0",
        "the envelope must declare JSON-RPC 2.0, got {parsed}"
    );
    assert_eq!(
        parsed["id"], 1,
        "the envelope id is pinned to 1 so one inflight call suffices, got {parsed}"
    );
    assert_eq!(
        parsed["method"], "sim.snapshot",
        "the method must be forwarded verbatim, got {parsed}"
    );
    assert_eq!(
        parsed["params"]["tick"], 7,
        "params must be forwarded verbatim, got {parsed}"
    );
}

/// Covers FR-CIV-MCP-003.
///
/// The envelope must be built for any method and any param shape, including an
/// empty object, since most read-only methods take `{}` or omit params.
#[test]
fn fr_civ_mcp_003_envelope_holds_for_arbitrary_methods_and_params() {
    for (method, params) in [
        ("health", serde_json::json!({})),
        ("sim.reset", serde_json::json!({ "seed": 42u64 })),
        (
            "sim.place_voxel",
            serde_json::json!({ "x": -3i64, "y": 0i64, "z": 9i64, "material": 6u16 }),
        ),
    ] {
        let parsed: serde_json::Value =
            serde_json::from_str(&build_rpc_request(method, params.clone()))
                .unwrap_or_else(|err| panic!("envelope for {method} is not valid JSON: {err}"));

        assert_eq!(parsed["jsonrpc"], "2.0", "method {method}");
        assert_eq!(parsed["method"], method, "method must round-trip");
        assert_eq!(
            parsed["params"], params,
            "params for {method} must round-trip unchanged"
        );
        assert_eq!(parsed["id"], 1, "id must be pinned for {method}");
    }
}

/// Covers FR-CIV-MCP-003.
///
/// A method name is passed through rather than validated locally, so an unknown
/// method still produces a well-formed envelope for the server to reject. That
/// is what makes the server's `-32601` mapping reachable; asserting the
/// envelope here documents that the client does not pre-filter.
///
/// The `-32601` response itself is produced by `civ-server` across a WebSocket
/// and needs a live server to observe, so it is not asserted here.
#[test]
fn fr_civ_mcp_003_unknown_method_still_forms_a_valid_envelope() {
    let parsed: serde_json::Value =
        serde_json::from_str(&build_rpc_request("sim.does_not_exist", serde_json::json!({})))
            .expect("an unknown method must still yield a parseable envelope");

    assert_eq!(
        parsed["method"], "sim.does_not_exist",
        "an unknown method must be forwarded so the server can answer -32601"
    );
    assert_eq!(
        parsed["jsonrpc"], "2.0",
        "the envelope must stay valid JSON-RPC even for an unknown method"
    );
    // The error mapping itself is server-side:
    // TODO(FR-CIV-MCP-003): cover the -32601 response shape with a harness that
    // runs a real civ-server, and assert the response error object there.
}
