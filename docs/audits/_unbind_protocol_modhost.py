"""Unbind the false container tags identified by the protocol/mod-host triage.

Every removal here is taken from a verdict in
docs/audits/triage-container-protocol-modhost.md, which cites the authoritative
requirement sentence and the real implementing symbol for each ID. Tags whose
verdict is DATA-SHAPE-ONLY or IMPLEMENTED-BY-BEHAVIOR are left alone.

The `FR-CIV-MOD-*` namespace is a genuine collision: CIV-0700 and
docs/design/modding-platform.md define the same IDs as different taxonomies.
Those tags are removed because under *neither* reading does the tagged symbol
discharge the requirement, which is the test the triage applied. The collision
itself is a spec-ownership decision and is escalated in the report rather than
silently resolved here.

The tool is idempotent: it writes a marker line naming the declaration, and a
second run refuses to touch a declaration it has already processed.
"""
import argparse
import pathlib
import re
import sys

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")

ID_RE = re.compile(r"\b(?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*\b")

# ---------------------------------------------------------------------------
# Verdict data, transcribed from the triage report.
#   removed: ID -> why the tagged symbol cannot discharge it
# ---------------------------------------------------------------------------

# ---- crates/protocol-3d/src/lib.rs and bundle.rs -------------------------
PROTO3D = {
    "FR-CIV-PROTO-001": "requirement is JSON-RPC 2.0 envelope compliance (id, jsonrpc, method/result/error), enforced by JsonRpcRequest/JsonRpcError and dispatch_request in crates/server/src/jsonrpc.rs; a u32 wire-compat number is not an envelope, and no client implements the refusal its doc comment describes (CIV-0200:1124)",
    "FR-CIV-PROTO-015": "requirement is a React/Vue web client that connects, subscribes, and renders; no component under web/dashboard/src imports a protocol-3d type, and the spec's example_web_client gate exists only inside docs/fragmented/ (CIV-0200:1194)",
    "FR-CIV-PROTO-002": "requirement is that the server accept WebSocket connections on port 9876; the bridge binds 127.0.0.1:3800 (ws_bridge.rs:150) and 9876 has zero hits repo-wide. A provenance/coordinate/frame enum is not a transport (CIV-0200:1129)",
    "FR-CIV-PROTO-007": "requirement is that multiple clients connect simultaneously without interfering; only a 16-client admission cap exists (ws_bridge.rs:128) and no interference test. A 2-variant provenance enum is not concurrency (CIV-0200:1154)",
    "FR-CIV-PROTO-009": "requirement is that a client can unsubscribe from broadcasts; that is genuinely implemented in civ-server at SubscriptionFilter::clear (subscription_filter.rs:146), but WorldXZ is a coordinate pair (CIV-0200:1164)",
    "FR-CIV-PROTO-010": "requirement is a state-query surface (agent_in_region, institution_ledger); agent_in_region has zero hits, institution_ledger matches only a doc comment, and no sim.query method exists (CIV-0200:1169)",
    "FR-CIV-PROTO-003": "requirement is a client handshake that returns current tick + seed + snapshot; JsonRpcMethod has no handshake variant and grep for handshake under crates/server/src returns zero (CIV-0200:1134)",
    "FR-CIV-PROTO-011": "requirement is the JSON-RPC error format {code, message, data}; that struct is JsonRpcError in crates/server/src/jsonrpc.rs:257. It is tagged here on a voxel delta frame (CIV-0200:1174)",
    "FR-CIV-PROTO-012": "requirement is a Bevy client that connects, subscribes, and renders agent positions under the example_bevy_client gate; the client exists at clients/bevy-ref but the gate does not, and the tag is on a voxel frame and a magic constant (CIV-0200:1179)",
    "FR-CIV-PROTO-008": "requirement is commands ordered by client_priority then tick_received; client_priority has exactly one repo-wide hit, the doc comment of the test that would have caught it. No priority field or sort exists (CIV-0200:1159)",
    "FR-CIV-PROTO-004": "requirement is a command rejected with a reason when resources are insufficient; the only reason key in the server is on an outcome result, not a command rejection (CIV-0200:1139)",
    "FR-CIV-PROTO-005": "requirement is subscription filtering by entity type/region; filtering is real but by frame kind at SubscriptionFilter::filter_frames, and get_snapshot_for_session returns the full snapshot. A magic constant and a zstd level filter nothing (CIV-0200:1144)",
    "FR-CIV-PROTO-013": "requirement is an Unreal plugin that unpacks binary frames and updates AActor transforms; clients/unreal-show contains no C++ that unpacks F3DB (CIV-0200:1184)",
    "FR-CIV-PROTO-014": "requirement is a Unity client connecting over WebSocket and rendering snapshots; there is no Unity client in this repository (clients/ holds bevy-ref, godot-ref, unreal-show only) (CIV-0200:1189)",
}

# ---- crates/mod-host ------------------------------------------------------
# Each reason notes the collision explicitly.
MOD = {
    "FR-CIV-MOD-002": "COLLIDING ID. modding-platform.md:28 = material + reaction registration; CIV-0700:2364 = CPU budget enforcement at 50us. No ReactionRegistrar/LawRegistrar exists and no fuel metering or epoch interruption exists. A boolean flag table is neither (policy_action.rs, lib.rs ModPermissions)",
    "FR-CIV-MOD-003": "COLLIDING ID. modding-platform.md:29 = building/recipe/structure grammar registration; CIV-0700:2372 = API version compatibility with IncompatibleApiVersion. The real behavior at ModCapabilitySet::can_read_domain/can_emit_action is deny-by-default permission gating, which is CIV-0700 FR-CIV-MOD-006, and WorldDomain is a 5-variant tag enum (capability.rs)",
    "FR-CIV-MOD-004": "COLLIDING ID. modding-platform.md:30 = law/physics-constant extension; CIV-0700:2380 = determinism invariant at every tick boundary. Neither exists. ModStatus is a lifecycle enum with no transition function, so its Faulted/Degraded states are unreachable; ModHost is a 5-field aggregate (capability.rs, lib.rs)",
    "FR-CIV-MOD-005": "COLLIDING ID. modding-platform.md:31 = species/genome primitive registration; CIV-0700:2388 = non-deterministic instruction rejection. No GenomeRegistrar exists; the real determinism scan is at scan_wasm_determinism (determinism.rs:86). ModHook is an 8-variant event enum and ModHookEngine just dispatches it (hooks.rs)",
    "FR-CIV-MOD-006": "COLLIDING ID. modding-platform.md:32 = biome/climate rule registration; CIV-0700:2396 = permission enforcement. No BiomeRegistrar exists; the real permission gate is at capability.rs:121,133. A hook variant and the engine struct are neither (hooks.rs)",
    "FR-CIV-MOD-007": "COLLIDING ID. modding-platform.md:33 = event hooks with bounded reactors; CIV-0700:2404 = mod fault isolation. ModHookEngine::execute is a real priority-ordered dispatch but no handler runs: hooks.rs:96 is `let _ = context; // available for future mod-guest calls`. No fault isolation and no capability-gated observer. HookResult is a 4-variant enum (hooks.rs)",
    "FR-CIV-MOD-008": "COLLIDING ID. modding-platform.md:34 = UI/overlay registration; CIV-0700:2412 = Ed25519 signature verification before instantiation. No OverlayRegistrar exists; the real signature check is verify_wasm_signature (signature.rs:26), which carries no tag. Tagged on an import-module string, an i32 version const, and a 4-field store (wasm_guest.rs)",
    "FR-CIV-MOD-009": "COLLIDING ID. modding-platform.md:35 = charter validator rejecting hardcoded-outcome mods; CIV-0700:2420 = scenario registration via ScenarioDescriptor. grep for charter under crates/mod-host/src returns zero and no ScenarioDescriptor exists. An import allowlist and a memory cap are neither (wasm_guest.rs)",
    "FR-CIV-MOD-010": "COLLIDING ID. modding-platform.md:36 = mod loading pipeline discover..bind; CIV-0700:2428 = action validation and conservation with ModActionRejected. The loader does parse + determinism-scan + signature-verify + register, with no charter-validate, dependency-resolve, ordering, law-merge, or bind stage; ModActionRejected has zero hits. Tagged on a u32 version const and two blob structs (guest_state.rs)",
    "FR-CIV-MOD-011": "COLLIDING ID. modding-platform.md:37 = dependency + version + capability model with semver; CIV-0700:2436 = custom good type registration. ModDependencies.civlab_api is parsed but never compared and ModMeta.api_version is never checked against a host range, so no IncompatibleApiVersion can fire. GuestStateError is a 2-variant enum whose real content is a JSON parse failure (guest_state.rs, lib.rs)",
    "FR-CIV-MOD-012": "COLLIDING ID. modding-platform.md:38 = load ordering (topological + priority + deterministic tie-break); CIV-0700:2444 = mid-simulation mod swap. No dependency graph exists, ModMeta has no priority field, ModRegistry::register is a bare Vec::push, and no sim.mod.swap method exists. DeterminismError/DeterminismScanReport are diagnostics, not an ordering mechanism (determinism.rs)",
    "FR-CIV-MOD-014": "COLLIDING ID. modding-platform.md:40 = hot-reload with staged code tier; CIV-0700:2460 = Lua script parity. ModHost::reload_mod is an unload-then-reload with no file watcher and no staged tier, and a failed reload leaves the mod unloaded rather than keeping the prior version; grep for lua under crates/ returns zero. Tagged on a filename const and an error enum (signature.rs)",
    "FR-CIV-MOD-015": "COLLIDING ID. modding-platform.md:41 = stable semver'd mod API surface; CIV-0700:2468 = mod status telemetry. The nine registrar traits the spec freezes do not exist, there is no mod-API SCHEMA_VERSION, and no mod counters reach crates/server/src/metrics.rs. PolicyActionKind is a discriminant enum (policy_action.rs)",
    "FR-CIV-MOD-016": "requirement is conflict detection and resolution (id collisions, law contradictions); the spec's conflict table needs a post-merge id scan, LawDb::validate over the union, and a constant-clash priority rule, none of which exist. FloatContaminationSite is a float data-flow diagnostic, a different feature (float_data_flow.rs)",
    "FR-CIV-MOD-017": "requirement is a Workshop-style content-addressed signed .civmod bundle with .civmod-lock and .civmod-sig members; both have zero hits repo-wide and the repo ships a plain ZIP. ManifestError is a thiserror enum describing IO failure (lib.rs)",
    "FR-CIV-MOD-019": "requirement is a mod test harness and lint at `civis mod validate`; that subcommand does not exist. ModLoadedRecord is a mod.loaded.v1 lifecycle record whose own doc comment cites a different id (lib.rs)",
    "FR-CIV-MOD-020": "requirement is save-game/mod compatibility and migration; save_schema, min_save_schema, and max_save_schema have zero hits across crates/mod-host and save_bundle.rs, so no compat block and no mod-set in any save. Tagged on the manifest filename const and a ModHost field (lib.rs)",
    # Defined in docs/traceability/fr-3d-matrix.md, but implemented in civ-watch, not here.
    "FR-CIV-TACTICS-062": "MIS-BOUND, NOT UNDEFINED. Defined at docs/traceability/fr-3d-matrix.md:152 as \"Mod catalog + runtime install\", discharged by civ-watch's post_mods_install handler (crates/watch/src/mods_api.rs, tested at api_tests.rs:681). ModBrowserEntry is a 7-field stub row in the wrong crate and implements none of it (mod-host/src/guest_state.rs:66)",
    "FR-CIV-TACTICS-070": "MIS-BOUND, NOT UNDEFINED. Defined at docs/traceability/fr-3d-matrix.md:160 as \"Remote mod fetch cache\", discharged by civ-watch's post_mods_fetch and list_remote_mods handlers (crates/watch/src/mods_api.rs, tested at api_tests.rs:1286). ModRegistry is a Vec<LoadedMod> in the wrong crate whose phase stubs say \"WASM callbacks not invoked yet\" (mod-host/src/lib.rs:254)",
}

# ---- crates/civis-mcp -----------------------------------------------------
MCP = {
    "FR-CIV-MCP-002": "requirement is read-only HTTP tools plus a --allow-mutations gate before any mutating /control/* route; allow_mutations and allow-mutations have zero hits under crates/civis-mcp. The list is 100+ tool names with no route kind attached and includes plainly mutating tools (civis_place_voxel, sim_undo, sim_reset) with no gate (civ-017 spec:38-42)",
    "FR-CIV-MCP-005": "requirement is configuration read only from CIVIS_MCP_CIV_SERVER_URL, CIVIS_MCP_CIV_WATCH_URL, and CIVIS_MCP_AUTH_TOKEN; all three have zero hits under crates/civis-mcp. HARNESS_VERSION is env!(\"CARGO_PKG_VERSION\"), a compile-time version string (civ-017 spec:49-51)",
}

# Tags on a declaration that the triage did not clear, so the tool leaves them be
# and can still tell "deliberately kept" from "missed by the triage".
# FR-CIV-PROTO-006 is the spec's own zstd data, so the constant and the options
# struct genuinely carry its shape. FR-SAVE-009 is materially discharged.
# FR-MOD-001..005 are the real mod-lifecycle ids for this crate.
# FR-CIV-TACTICS-047/049/053/057/061 belong to validate_guest_imports and the
# determinism scan, which are real and are examined in their own right.
KEEP = {
    "FR-CIV-PROTO-006", "FR-SAVE-009",
    "FR-MOD-001", "FR-MOD-002", "FR-MOD-003", "FR-MOD-004", "FR-MOD-005",
    "FR-CIV-TACTICS-047", "FR-CIV-TACTICS-049", "FR-CIV-TACTICS-053",
    "FR-CIV-TACTICS-057", "FR-CIV-TACTICS-061",
    "FR-CIV-MOD-000",
}

# ---- out-of-lane ids on in-lane files ------------------------------------
RTS = {
    "FR-CIV-RTS-015": "requirement is client-side prediction and replay correction (interpolation, snap under 100 ms), which is a client behavior; no prediction or smoothing code exists in crates/. ReplayLog is a server-side event recorder (CIV-0300 sec 12.1)",
}

SITES = [
    # (path, declaration, removed-dict, note-lines)
    ("crates/protocol-3d/src/lib.rs", "SCHEMA_VERSION", PROTO3D, [
        "The wire-protocol FRs are real requirements, and several of the behaviors they",
        "describe genuinely exist elsewhere in the production path (`civ-server`, the Bevy",
        "client, `SubscriptionFilter`). These tags are misplaced rather than fabricated, but",
        "a tag names the symbol that discharges the requirement, and none of these passive",
        "types and constants does.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "BuildingProvenance", PROTO3D, [
        "A 2-variant provenance enum cannot be a WebSocket listener on a specific port.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "WorldXZ", PROTO3D, [
        "A coordinate pair is not a subscription, a query surface, or a transport.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "BuildingKind3d", PROTO3D, [
        "A building-class enum is not a transport, a concurrency property, or a query API.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "BuildingDiffEntry", PROTO3D, [
        "A diff row is not a transport or a query surface.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "BuildingDiffFrame", PROTO3D, [
        "A frame payload struct is not a WebSocket listener.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "CivilianNeeds3d", PROTO3D, [
        "A per-civilian needs struct cannot order commands by client priority; no such",
        "priority field exists anywhere in the repository.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "VoxelDeltaFrame", PROTO3D, [
        "A voxel batch is not a handshake, a JSON-RPC error envelope, or a Bevy client.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "AgentAppearanceFrame", PROTO3D, [
        "An appearance-update batch cannot order commands by client priority.",
    ]),
    ("crates/protocol-3d/src/lib.rs", "FRAME3D_BINARY_MAGIC", PROTO3D, [
        "A four-byte magic performs no handshake, no resource-acceptance decision, and is",
        "not the server's error type.",
    ]),
    ("crates/protocol-3d/src/bundle.rs", "FRAME3D_BUNDLE_MAGIC", PROTO3D, [
        "`b\"F3DB\"` filters nothing, connects to nothing, and unpacks nothing.",
    ]),
    ("crates/protocol-3d/src/bundle.rs", "DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL", {
        k: v for k, v in PROTO3D.items() if k != "FR-CIV-PROTO-006"
    }, [
        "FR-CIV-PROTO-006 is deliberately kept on this constant: a zstd level is exactly",
        "the data the requirement names, so that one tag is legitimate on shape. The other",
        "three are not.",
    ]),
    ("crates/protocol-3d/src/bundle.rs", "Frame3dBundleFlags", PROTO3D, [
        "There is no Unity client in this repository; the tag is on a newtype over one",
        "compression bit.",
    ]),
    ("crates/protocol-3d/src/bundle.rs", "Frame3dBundleEncodeOptions", {
        k: v for k, v in PROTO3D.items() if k != "FR-CIV-PROTO-006"
    }, [
        "FR-CIV-PROTO-006 is kept here for the same reason as on the constant above.",
    ]),
    ("crates/mod-host/src/lib.rs", "ModPermissions", MOD, [
        "The FR-CIV-MOD-* namespace is a genuine collision: CIV-0700 and",
        "docs/design/modding-platform.md define the same ids as different taxonomies. Every",
        "tag below is removed because the tagged symbol fails under *both* readings, not",
        "because the collision has been resolved. Which taxonomy wins is a spec-ownership",
        "decision and is escalated in docs/audits/triage-container-protocol-modhost.md.",
    ]),
    ("crates/mod-host/src/lib.rs", "ModMeta", MOD, [
        "See the namespace-collision note above. FR-CIV-MOD-000 is deliberately retained:",
        "the manifest schema requirement is materially discharged by these two types, and",
        "the open item is the missing RON/JSON parser, not a mis-binding.",
    ]),
    ("crates/mod-host/src/lib.rs", "ManifestError", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/lib.rs", "CIVMOD_MANIFEST_NAME", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/lib.rs", "ModLoadedRecord", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/lib.rs", "ModRegistry", MOD, [
        "See the namespace-collision note above. FR-CIV-TACTICS-070 is NOT undefined: it is",
        "defined at docs/traceability/fr-3d-matrix.md:160 as \"Remote mod fetch cache\",",
        "implemented by civ-watch's post_mods_fetch and list_remote_mods handlers",
        "(crates/watch/src/mods_api.rs; behaviour tested at api_tests.rs:1286). The tag sits",
        "on a registry Vec in the wrong crate and is removed as a mis-binding, not as an",
        "undefined id.",
    ]),
    ("crates/mod-host/src/lib.rs", "ModHost", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/capability.rs", "WorldDomain", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/capability.rs", "ModStatus", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/capability.rs", "ModCapabilitySet", MOD, [
        "See the namespace-collision note above. The permission behavior implemented here",
        "is real, tested, and currently untagged; it belongs to CIV-0700 FR-CIV-MOD-006,",
        "which is recorded as follow-up rather than applied here, because the id space is",
        "still ambiguous.",
    ]),
    ("crates/mod-host/src/hooks.rs", "ModHook", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/hooks.rs", "HookResult", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/hooks.rs", "ModHookEngine", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/wasm_guest.rs", "HOST_IMPORT_MODULE", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/wasm_guest.rs", "HOST_CAPABILITY_IMPORTS", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/wasm_guest.rs", "HOST_CAPABILITY_API_VERSION", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/wasm_guest.rs", "HOST_GUEST_MEMORY_CAP", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/wasm_guest.rs", "HostState", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/determinism.rs", "DeterminismError", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/determinism.rs", "DeterminismScanReport", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/guest_state.rs", "MOD_GUEST_STATE_VERSION", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/guest_state.rs", "ModGuestMemoryBlob", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/guest_state.rs", "ModGuestStateSave", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/guest_state.rs", "GuestStateError", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/guest_state.rs", "ModBrowserEntry", MOD, [
        "See the namespace-collision note above. FR-CIV-TACTICS-062 is NOT undefined: it is",
        "defined at docs/traceability/fr-3d-matrix.md:152 as \"Mod catalog + runtime install\",",
        "implemented by civ-watch's post_mods_install handler",
        "(crates/watch/src/mods_api.rs; behaviour tested at api_tests.rs:681). The tag sits",
        "on a stub struct in the wrong crate and is removed as a mis-binding, not as an",
        "undefined id.",
    ]),
    ("crates/mod-host/src/policy_action.rs", "PolicyActionKind", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/signature.rs", "MOD_WASM_SIG_NAME", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/mod-host/src/signature.rs", "SignatureError", MOD, [
        "See the namespace-collision note above. The correct id for this file is CIV-0700",
        "FR-CIV-MOD-008 (Signature Verification); it is not written anywhere, and applying",
        "it is deferred until the collision is resolved.",
    ]),
    ("crates/mod-host/src/float_data_flow.rs", "FloatContaminationSite", MOD, [
        "See the namespace-collision note above.",
    ]),
    ("crates/civis-mcp/src/lib.rs", "TOOL_NAMES", MCP, [
        "The list has no route kind attached and no read-only gate, so it cannot discharge",
        "the requirement however the tools are actually dispatched.",
    ]),
    ("crates/civis-mcp/src/lib.rs", "HARNESS_VERSION", MCP, [
        "A compile-time version string is not environment-variable configuration.",
    ]),
    ("crates/engine/src/replay.rs", "ReplayLog", RTS, [
        "FR-SAVE-009 is deliberately kept on ReplayLog: the hash-chain tail really is",
        "serialized and restored, and that requirement is materially met. FR-CIV-RTS-015",
        "describes client-side prediction, which is a different subsystem entirely.",
    ]),
]


def is_tag_line(line: str) -> bool:
    s = line.strip()
    if not (s.startswith("//") and ID_RE.search(s)):
        return False
    # A line this tool wrote is bookkeeping, not a binding. Without this the
    # explanatory reasons would be re-counted as live tags on the next run.
    return "[unbound]" not in s


def is_doc_line(line: str) -> bool:
    s = line.strip()
    return s.startswith("///") or s.startswith("//!")


def is_attr_line(line: str) -> bool:
    return line.strip().startswith("#[")


def collect_ids(text: str):
    return set(ID_RE.findall(text))


def process(rel: str, decl: str, removed: dict, note: list, apply: bool):
    path = REPO / rel
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")

    # Find the declaration.
    decl_idx = None
    for i, line in enumerate(lines):
        stripped = line.strip()
        if not (stripped.startswith("pub ") or stripped.startswith("pub(")):
            continue
        if not re.match(r"pub\s+(struct|enum|const|static|type)\s", stripped):
            continue
        m = re.match(r"pub\s+(?:struct|enum|const|static|type)\s+([A-Za-z0-9_]+)", stripped)
        if m and m.group(1) == decl:
            decl_idx = i
            break
    if decl_idx is None:
        return f"  MISSING declaration {decl} in {rel}", 0

    def skippable(line: str) -> bool:
        return is_doc_line(line) or is_tag_line(line) or is_attr_line(line)

    tags_start = None
    i = decl_idx - 1
    while i >= 0 and skippable(lines[i]):
        if is_tag_line(lines[i]):
            tags_start = i
        i -= 1
    if tags_start is None:
        return f"  no tag block above {rel} :: {decl}", 0

    block = lines[tags_start:decl_idx]
    present = collect_ids("\n".join(block))
    removable = sorted(present & set(removed))
    unknown = sorted(present - set(removed) - KEEP)
    if unknown:
        return f"  WARNING {rel} :: {decl} has unclassified ids {unknown}", 0
    if not removable:
        return f"  nothing removable at {rel} :: {decl}", 0

    # Idempotence: the count is part of the marker, so a declaration that has
    # already been processed by this tool is recognised and skipped. Checking it
    # after classification keeps the marker honest about what it claims.
    marker = f"// The following {len(removable)} requirement tags were removed from {decl}."
    if any(marker in l for l in lines):
        return f"  already processed: {rel} :: {decl}", 0

    def strip_removed_ids(line: str) -> str:
        """Drop only the false ids from a tag line, keeping the legitimate ones.

        A line such as `// FR-CIV-RTS-015, FR-SAVE-009` mixes a false binding with
        a real one. Removing the whole line would discard the true tag; keeping it
        whole would leave the false one asserted. So the false ids are deleted
        individually and the separators tidied, and a line left with no ids at all
        disappears.
        """
        ids = collect_ids(line)
        if not ids:
            return line
        if not (ids & set(removed)):
            return line
        for bad in sorted(ids & set(removed)):
            line = re.sub(rf"\b{re.escape(bad)}\b", "", line)
        # Tidy the separators the deletions left behind, but only inside a comment
        # so the surrounding code cannot be touched.
        if line.lstrip().startswith("//"):
            body = re.sub(r",\s*(?=,)", "", line)
            body = re.sub(r"^\s*(//\s*)[,;]\s*", r"\1", body)
            body = re.sub(r"[,;]\s*$", "", body)
            body = re.sub(r"\(\s*\)|\[\s*\]", "", body)
            body = re.sub(r"[,;]{2,}", ",", body)
            line = body.rstrip()
            if not collect_ids(line):
                return ""
        return line

    # Every line this tool writes back must carry a bare-ID token at the start of
    # the comment body, so that a second run recognises it as already-processed
    # rather than as a fresh live tag. The tag name follows that token, which is
    # what keeps the reason text from being mistaken for a binding on a rerun.
    TOKEN = "  [unbound]"
    kept = []
    for line in block:
        stripped = strip_removed_ids(line)
        if stripped:
            kept.append(stripped)
    reason_lines = [f"//{TOKEN} {tag}: {removed[tag]}" for tag in removable]

    # Doc comments and attributes must sit immediately above the declaration.
    trailing_idx = {k for k, l in enumerate(kept) if is_doc_line(l) or is_attr_line(l)}
    split = min(trailing_idx) if trailing_idx else len(kept)
    leading, trailing = kept[:split], kept[split:]

    header = [
        f"// The following {len(removable)} requirement tags were removed from {decl}.",
        "// They are not discharged by this symbol. The tag named a requirement whose",
        "// behavior lives elsewhere, or a requirement with no implementation at all, so",
        "// leaving the tag here asserted coverage that this declaration does not provide.",
    ]
    header += [f"// {n}" for n in note]
    header += ["//", "// Removed, with the reason each cannot be discharged here:"]
    new_lines = lines[:tags_start] + header + reason_lines + leading + trailing + lines[decl_idx:]

    out = "\n".join(new_lines)
    if apply:
        path.write_text(out, encoding="utf-8", newline="\n")
    return f"  {rel} :: {decl}: removed {len(removable)} tag(s)", len(removable)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args()

    total = 0
    messages = []
    for rel, decl, removed, note in SITES:
        msg, n = process(rel, decl, removed, note, args.apply)
        messages.append(msg)
        total += n
    for m in messages:
        print(m)
    print(f"\ntotal removed: {total}")
    if not args.apply:
        print("(preview only; pass --apply to write)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
