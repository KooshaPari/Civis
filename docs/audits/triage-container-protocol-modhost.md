# Container-Binding Triage — Protocol / Mod-Host / Network

**Date:** 2026-09-29
**Scope:** the 72 rows of `docs/audits/container-bindings.json` whose `file` is under `crates/mod-host/` (34), `crates/protocol-3d/` (34), `crates/civis-mcp/` (2), plus `crates/engine/src/replay.rs` (2, the `.civreplay` serialization rows). This is the fourth lane of the four-way split; the three sibling reports own `crates/server/**`, `crates/engine/src/engine.rs` + `WorldState`/`Simulation`, and the sim/domain crates.
**Method:** For every in-scope ID the authoritative requirement sentence was read from its owning spec (`docs/specs/CIV-0200-client-protocol.md`, `docs/design/modding-platform.md`, `agileplus-specs/civ-017-civis-mcp-server/spec.md`, `docs/specs/CIV-0700-modding-api-spec.md`, `docs/specs/CIV-1000-save-load-persistence-spec.md`) — **not** from the JSON `requirement` field, which in this lane is a traceability-table artifact and is wrong for 30 of 72 rows. The tagged item was then read in source and the owning crate grepped for a function performing the required behavior. No `cargo build`/`cargo test` was run. No `.rs` file was modified.

## Verdict counts

| Verdict | Count |
|---|---|
| NOT-IMPLEMENTED | 6 |
| CONTAINER-ONLY (FALSE tag) | 61 |
| DATA-SHAPE-ONLY (tag legitimate) | 4 |
| IMPLEMENTED-BY-BEHAVIOR | 1 |
| **Total** | **72** |

**67 of 72 tags (93%) are FALSE** (CONTAINER-ONLY or NOT-IMPLEMENTED). Only 5 tags (7%) are legitimate. For comparison, the sibling lanes report 123/164 false (75%) in `triage-container-sim-domain.md` and 32/46 (70%) in `triage-container-engine-core.md`.

The 61 CONTAINER-ONLY rows collapse to **32 distinct requirement IDs** — 14 `FR-CIV-MOD-*` (26 rows), 15 `FR-CIV-PROTO-*` (33 rows), 2 `FR-CIV-MCP-*` (2 rows). Ten of those IDs carry 2-5 tags each, so the headline count overstates the number of distinct defects:

| Requirement ID | Tag rows | Why every one of them is false |
|---|---|---|
| `FR-CIV-PROTO-002` | 5 | Port 9876 is not honored (the bridge binds 3800); 5 tags on 4 data types. |
| `FR-CIV-PROTO-010` | 3 | The query API (`sim.query`, `agent_in_region`, `institution_ledger`) does not exist. |
| `FR-CIV-PROTO-012` | 4 | The Bevy client exists but the spec's `example_bevy_client` gate does not. |
| `FR-CIV-PROTO-005` | 3 | Filtering is real but by frame kind, not entity type/region; 3 tags on constants. |
| `FR-CIV-PROTO-013` | 3 | The Unreal C++ frame unpacker does not exist. |
| `FR-CIV-MOD-008` / `-010` | 3 each | UI-overlay / scenario-registration under one reading, signature-verify / action-validation under the other. |
| `FR-CIV-PROTO-003` / `-008` / `-011` / `-015` | 2 each | Handshake absent / `client_priority` absent / error struct lives in `civ-server` / web client is not a `protocol-3d` consumer. |
| `FR-CIV-MOD-002` … `-014` | 2 each | Namespace collision — see Defect 2 below. |
| Remaining 19 IDs | 1 each | Each is a single false tag on a single passive type or constant. |

---

## The defect, in one picture

There are **two** defects in this lane, and the second is worse than a mis-placed tag.

**Defect 1 — tags on passive containers.** The familiar pattern: `crates/protocol-3d/src/bundle.rs:15-17` stacks three requirement tags on a magic-byte constant.

```
15  // FR-CIV-PROTO-005   <- "Subscribe with filter; receive only requested entity types/regions"
16  // FR-CIV-PROTO-012   <- "Bevy client can connect, subscribe, and render agent positions"
17  // FR-CIV-PROTO-013   <- "Unreal plugin can unpack binary frames and update AActor transforms"
18  pub const FRAME3D_BUNDLE_MAGIC: &[u8; 4] = b"F3DB";
```

`b"F3DB"` filters nothing, connects to nothing, and unpacks nothing. The tags describe a transport, a Bevy plugin, and an Unreal C++ plugin, none of which is a 4-byte array.

**Defect 2 — the `FR-CIV-MOD-*` namespace is a namespace collision, and it is the reason most of this lane is unverifiable.** `docs/specs/CIV-0700-modding-api-spec.md:2356-2474` defines `FR-CIV-MOD-001` … `FR-CIV-MOD-015` as a completely different taxonomy:

| ID | CIV-0700 §15 title | `docs/design/modding-platform.md` §0 title |
|---|---|---|
| `FR-CIV-MOD-002` | CPU Budget Enforcement (terminate a mod callback exceeding 50 µs) | Material + reaction registration |
| `FR-CIV-MOD-003` | **API Version Compatibility Enforcement** (reject `api_version` outside `[current-1, current]`, return `IncompatibleApiVersion`) | Building / recipe / structure **grammar** registration |
| `FR-CIV-MOD-004` | **Mod Determinism Invariant** (identical state hash at every tick boundary) | Law / physics-constant extension |
| `FR-CIV-MOD-005` | **Non-Deterministic Instruction Rejection** | Species / genome primitive registration |
| `FR-CIV-MOD-006` | **Permission Enforcement** (`ERR_PERMISSION_DENIED`) | Biome / climate rule registration |
| `FR-CIV-MOD-007` | **Mod Fault Isolation** | Event hooks |
| `FR-CIV-MOD-008` | **Signature Verification** (Ed25519 before instantiation) | UI / overlay registration |
| `FR-CIV-MOD-009` | Scenario Registration and Init | Charter validator |
| `FR-CIV-MOD-010` | **Action Validation and Conservation** | Mod loading pipeline |
| `FR-CIV-MOD-011` | Custom Good Type Registration | Dependency + version + capability model |
| `FR-CIV-MOD-012` | **Mid-Simulation Mod Swap** (`sim.mod.swap()`) | Load ordering |
| `FR-CIV-MOD-014` | **Lua Script Parity** | Hot-reload |
| `FR-CIV-MOD-015` | **Mod Status Telemetry** | Stable mod API surface |

`docs/traceability/fr-civ-mod-003/fr-civ-mod-003-spec.md` is the tie-breaker and it is unambiguous: its `Implementing Code` list names **both** sources (`docs/design/modding-platform.md:29,192` **and** `docs/specs/CIV-0700-modding-api-spec.md:2372`). The generated traceability layer merged two incompatible FR families into one ID space. The in-source tags all follow the **design-doc** numbering, and the source code all follows the **CIV-0700** numbering, so **the two never meet**:

- `crates/mod-host/src/capability.rs:69-82` tags `ModCapabilitySet` with `FR-CIV-MOD-003` (a **grammar-registration** id). But `ModCapabilitySet::can_read_domain` / `can_emit_action` (lines 121, 133) is real, tested, deny-by-default **permission enforcement** — that is CIV-0700's **FR-CIV-MOD-006**, not `-003`.
- `crates/mod-host/src/signature.rs:26` `verify_wasm_signature` is real Ed25519 verification with a tamper test. It carries **no** FR tag; the ids `FR-CIV-MOD-008` (Signature Verification) and `FR-CIV-MOD-014` (Lua parity) are tagged on `HOST_IMPORT_MODULE` and `MOD_WASM_SIG_NAME` respectively, and neither is the behavior.
- `crates/mod-host/src/determinism.rs:8,35` tags the error enum and report struct with `FR-CIV-MOD-012` (a **load-ordering** id). The determinism scan is real and is CIV-0700's **FR-CIV-MOD-005** — and the code already says so, tagging the *functions* `FR-CIV-MOD-013` at lines 49 and 85, a **third** numbering that matches neither spec.

**Consequence for the audit:** for every `FR-CIV-MOD-*` row below I judge the tag **as written** — i.e. does the tagged symbol discharge the requirement sentence the cited authority states. A row can be a *correct behavior, wrong id*; that is still a false binding, and I say so in the reason column.

---

## §1 — `crates/protocol-3d` — client wire protocol (34 rows)

Crate wiring: `civ-protocol-3d` is a workspace member and is depended on by `civ-server`, `clients/bevy-ref`, and `clients/godot-ref/rust` — it **is** on a production path. That matters for the verdict: where the behavior exists, the tag is misplaced rather than fabricated.

### The JSON `requirement` field is corrupt for this file group

For 30 of 34 rows the `requirement` string in `container-bindings.json` is not a requirement. It is the **middle column of a markdown table row** that the detector's `load_spec_texts()` truncated at the `|`. Example, the raw stored value:

```
'FR-CIV-PROTO-005'  ->  ': Snapshot Filtering'      (3 identical rows)
'FR-CIV-PROTO-010'  ->  ''                          (3 identical rows, defined_by_spec: false)
'FR-CIV-MOD-016'    ->  'Conflict detection + resolution (ID collisions, law contradictions)'
```

`: Snapshot Filtering` is the tail of a `| :name: | FR-CIV-PROTO-005 | : Snapshot Filtering |`-shaped line, not a SHALL. **The `defined_by_spec: true` flag in the JSON is unreliable for this lane** — it is true for `FR-CIV-PROTO-005` only because the fragment `: Snapshot Filtering` happens to be ≥12 characters and not a table row. This is a defect in the detector, not just the tags, and it is why `FR-CIV-PROTO-010` is flagged `defined_by_spec: false` while `FR-CIV-PROTO-005` is not. Every verdict below is anchored to the real sentence at `docs/specs/CIV-0200-client-protocol.md:1124-1197`.

### All 34 `crates/protocol-3d` candidates, one row per tag (`lib.rs` 21 + `bundle.rs` 13)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `SCHEMA_VERSION` (lib.rs:43) | FR-CIV-PROTO-001 | CONTAINER-ONLY | `JsonRpcRequest`/`JsonRpcError` + `dispatch_request` — `crates/server/src/jsonrpc.rs:257`, `:1946` | CIV-0200:1124 | "All messages comply with JSON-RPC 2.0 spec (id, jsonrpc, method/result/error)" is enforced by the server's parse/dispatch; a `u32 = 0` wire-compat number is not, and its own doc comment ("clients refuse to attach on mismatch") describes a refusal path no client implements. |
| 2 | `SCHEMA_VERSION` (lib.rs:44) | FR-CIV-PROTO-015 | CONTAINER-ONLY | partial, unowned: `web/dashboard` WS consumer | CIV-0200:1194 | "Web client can connect, subscribe, and render in React/Vue"; the spec's gate `example_web_client` appears **only** inside `docs/fragmented/`. Second tag on the same const. |
| 3 | `BuildingProvenance` (lib.rs:49) | FR-CIV-PROTO-002 | CONTAINER-ONLY | `spawn_ws_bridge` — `crates/server/src/ws_bridge.rs:595` | CIV-0200:1129 | "Server accepts WebSocket connections on **port 9876**" — the bridge binds `127.0.0.1:3800` (`ws_bridge.rs:150`) and `git grep "9876"` returns **zero** hits repo-wide. A 2-variant enum is not a transport. |
| 4 | `BuildingProvenance` (lib.rs:50) | FR-CIV-PROTO-007 | CONTAINER-ONLY | `WsBridgeState.max_clients` — `ws_bridge.rs:128,1109` | CIV-0200:1154 | "Multiple clients connect simultaneously; commands don't interfere" is a concurrency property; only a 16-client admission cap exists, and no interference test does. |
| 5 | `WorldXZ` (lib.rs:60) | FR-CIV-PROTO-002 | CONTAINER-ONLY | same as row 3 | CIV-0200:1129 | Second `FR-CIV-PROTO-002`; a 2-field `f32` coordinate pair cannot be a WebSocket listener. |
| 6 | `WorldXZ` (lib.rs:61) | FR-CIV-PROTO-009 | CONTAINER-ONLY | `SubscriptionFilter::clear` — `subscription_filter.rs:146`, wired `ws_bridge.rs:882` | CIV-0200:1164 | "Client can unsubscribe from broadcasts" is genuinely implemented and on the live broadcast path — but the behavior is in `civ-server`, and `WorldXZ` is a coordinate. |
| 7 | `WorldXZ` (lib.rs:62) | FR-CIV-PROTO-010 | CONTAINER-ONLY | none | CIV-0200:1169 | "Research clients can query state (`agent_in_region`, `institution_ledger`)" — `agent_in_region` → **zero** hits; `institution_ledger` matches only a doc comment (`crates/economy/src/allocator.rs:164`). No `sim.query` method exists. **JSON flags this `defined_by_spec: false`**; the real sentence is at CIV-0200:1169, so the flag is a detector false-negative, not an absent requirement. |
| 8 | `BuildingKind3d` (lib.rs:84) | FR-CIV-PROTO-002 | CONTAINER-ONLY | same as row 3 | CIV-0200:1129 | Third `FR-CIV-PROTO-002`; a 6-variant building-class enum. |
| 9 | `BuildingKind3d` (lib.rs:85) | FR-CIV-PROTO-007 | CONTAINER-ONLY | same as row 4 | CIV-0200:1154 | Second `FR-CIV-PROTO-007`. |
| 10 | `BuildingKind3d` (lib.rs:86) | FR-CIV-PROTO-010 | CONTAINER-ONLY | none | CIV-0200:1169 | Third `FR-CIV-PROTO-010`; same absent query API. |
| 11 | `BuildingDiffEntry` (lib.rs:106) | FR-CIV-PROTO-002 | CONTAINER-ONLY | same as row 3 | CIV-0200:1129 | Fourth `FR-CIV-PROTO-002`; a 4-field building diff row. |
| 12 | `BuildingDiffEntry` (lib.rs:107) | FR-CIV-PROTO-010 | CONTAINER-ONLY | none | CIV-0200:1169 | Fourth `FR-CIV-PROTO-010`; second tag on `BuildingDiffEntry`. The named query surface (`agent_in_region`, `institution_ledger`) does not exist anywhere in the repo. |
| 13 | `BuildingDiffFrame` (lib.rs:122) | FR-CIV-PROTO-002 | CONTAINER-ONLY | same as row 3 | CIV-0200:1129 | Fifth `FR-CIV-PROTO-002`; a frame payload struct. |
| 14 | `CivilianNeeds3d` (lib.rs:140) | FR-CIV-PROTO-008 | CONTAINER-ONLY | none | CIV-0200:1159 | "Commands ordered by `client_priority`, then `tick_received`" — `git grep "client_priority"` across `crates/`, `web/`, `clients/` returns **exactly one** hit: the doc comment of the test that would have caught it (`crates/engine/tests/fr_fr_civ_core_016.rs:6`). No priority field, no sort, no tier. |
| 15 | `VoxelDeltaFrame` (lib.rs:408) | FR-CIV-PROTO-003 | CONTAINER-ONLY | none | CIV-0200:1134 | "Client sends handshake, receives current tick + seed + snapshot" — `JsonRpcMethod` (jsonrpc.rs:38-163) has **no** handshake variant and `git grep "handshake" -- crates/server/src` returns zero. A voxel batch is not a handshake. |
| 16 | `VoxelDeltaFrame` (lib.rs:409) | FR-CIV-PROTO-011 | CONTAINER-ONLY | `JsonRpcError { code, message, data }` — `jsonrpc.rs:257` | CIV-0200:1174 | "All errors return JSON-RPC error format with code, message, optional data" **is** implemented and that struct is the correct artifact for it — but it lives in `civ-server`. Nearest-miss placement. |
| 17 | `VoxelDeltaFrame` (lib.rs:410) | FR-CIV-PROTO-012 | CONTAINER-ONLY | partial, unowned: `clients/bevy-ref/src/bin/bevy_window.rs:71` | CIV-0200:1179 | "Bevy client can connect, subscribe, and render agent positions" is real at the client, but the spec's gate `example_bevy_client` does not exist. |
| 18 | `AgentAppearanceFrame` (lib.rs:421) | FR-CIV-PROTO-008 | CONTAINER-ONLY | none | CIV-0200:1159 | Second `FR-CIV-PROTO-008`; an appearance-update batch cannot order commands. |
| 19 | `FRAME3D_BINARY_MAGIC` (lib.rs:526) | FR-CIV-PROTO-003 | CONTAINER-ONLY | none | CIV-0200:1134 | Third tag on `b"F3D0"`; no handshake method exists to carry tick+seed+snapshot. |
| 20 | `FRAME3D_BINARY_MAGIC` (lib.rs:527) | FR-CIV-PROTO-004 | CONTAINER-ONLY | `is_frame3d_binary` — `lib.rs:536` | CIV-0200:1139 | "Commands accepted if resources sufficient; **rejected with reason** if not" — the only `"reason"` key in the server is on an *outcome* result (jsonrpc.rs:2132), not a command rejection. A 4-byte magic performs no acceptance decision. |
| 21 | `FRAME3D_BINARY_MAGIC` (lib.rs:528) | FR-CIV-PROTO-011 | CONTAINER-ONLY | `JsonRpcError` — `jsonrpc.rs:257` | CIV-0200:1174 | Second `FR-CIV-PROTO-011`; error formatting is a server concern. |
| 22 | `FRAME3D_BUNDLE_MAGIC` (bundle.rs:15) | FR-CIV-PROTO-005 | CONTAINER-ONLY | `SubscriptionFilter::filter_frames` — `subscription_filter.rs:167`, wired `ws_bridge.rs:882` | CIV-0200:1144 | "Subscribe with filter; receive only requested entity types/regions" — real, but by **frame kind**, not entity type/region, and `get_snapshot_for_session` (`engine.rs:3321`) returns the **full** snapshot with filtering deferred per its own doc comment. `b"F3DB"` filters nothing. |
| 23 | `FRAME3D_BUNDLE_MAGIC` (bundle.rs:16) | FR-CIV-PROTO-012 | CONTAINER-ONLY | same as row 16 | CIV-0200:1179 | Second `FR-CIV-PROTO-012`, on a magic constant. |
| 24 | `FRAME3D_BUNDLE_MAGIC` (bundle.rs:17) | FR-CIV-PROTO-013 | CONTAINER-ONLY | none | CIV-0200:1184 | "Unreal plugin can unpack binary frames and update AActor transforms" — `clients/unreal-show/` contains **no** C++ that unpacks `F3DB` (only a README mentions frames). |
| 25 | `DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL` (bundle.rs:28) | FR-CIV-PROTO-005 | CONTAINER-ONLY | same as row 21 | CIV-0200:1144 | Second `FR-CIV-PROTO-005`; a compression level is not a filter. |
| 26 | `DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL` (bundle.rs:29) | FR-CIV-PROTO-006 | DATA-SHAPE-ONLY | `encode_frame3d_bundle` / `decode_frame3d_bundle` — `bundle.rs:151,194` | CIV-0200:1149 | "Support binary frames with zstd compression; unpack without errors" — the tagged const **is** the spec's named data (a zstd level), so the tag is legitimate on shape. The behavior lives in the two codec functions; note the level is unvalidated (no clamp to zstd's valid range is asserted anywhere) and the spec's `use_binary_frames=true` opt-in has **zero** hits repo-wide. |
| 27 | `DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL` (bundle.rs:30) | FR-CIV-PROTO-012 | CONTAINER-ONLY | same as row 16 | CIV-0200:1179 | Third `FR-CIV-PROTO-012`. |
| 28 | `DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL` (bundle.rs:31) | FR-CIV-PROTO-013 | CONTAINER-ONLY | none | CIV-0200:1184 | Third `FR-CIV-PROTO-013`. |
| 29 | `Frame3dBundleFlags` (bundle.rs:37) | FR-CIV-PROTO-014 | CONTAINER-ONLY | none | CIV-0200:1189 | "Unity client can connect via WebSocket and render snapshots" — `git grep -l "Unity" -- clients/` returns `clients/unreal-show/README.md` only. **There is no Unity client in this repository.** The tag is on a newtype over one compression bit. |
| 30 | `Frame3dBundleEncodeOptions` (bundle.rs:65) | FR-CIV-PROTO-005 | CONTAINER-ONLY | same as row 21 | CIV-0200:1144 | Third `FR-CIV-PROTO-005`. |
| 31 | `Frame3dBundleEncodeOptions` (bundle.rs:66) | FR-CIV-PROTO-006 | DATA-SHAPE-ONLY | `encode_frame3d_bundle` — `bundle.rs:151` | CIV-0200:1149 | Second `FR-CIV-PROTO-006`. The struct is the correct data shape for the requirement (a `compress` flag plus a zstd level); the *behavior* — unpack without errors — is performed by `decode_frame3d_bundle`, which carries no tag. Legitimate on shape, silent on the behavior. |
| 32 | `Frame3dBundleEncodeOptions` (bundle.rs:67) | FR-CIV-PROTO-012 | CONTAINER-ONLY | same as row 16 | CIV-0200:1179 | Fourth `FR-CIV-PROTO-012`. |
| 33 | `Frame3dBundleEncodeOptions` (bundle.rs:68) | FR-CIV-PROTO-013 | CONTAINER-ONLY | none | CIV-0200:1184 | Fourth `FR-CIV-PROTO-013`. |
| 34 | `Frame3dBundleEncodeOptions` (bundle.rs:69) | FR-CIV-PROTO-015 | CONTAINER-ONLY | partial, unowned: `web/dashboard` WS consumer | CIV-0200:1194 | Second `FR-CIV-PROTO-015`; no React/Vue component in `web/dashboard/src` imports a `protocol-3d` type. |
| 35 | *(not a JSON candidate — see note)* | FR-CIV-PROTO-006 | **NOT IMPLEMENTED** | `decode_frame3d_binary` — `lib.rs:617` | CIV-0200:1149 | **Coverage note, not a tag row.** No source symbol carries `FR-CIV-PROTO-006` at its own definition site; both tags land on bundle-level constants. `decode_frame3d_binary` is the real `F3D0` unpacker and is exercised end-to-end via `civ-server`, but the spec sentence also requires the `use_binary_frames=true` opt-in, which has **zero** hits repo-wide, so the requirement is not satisfied end to end. |

**Row 35 is a coverage note, not one of the 34 candidates.** Rows 1–34 map 1:1 onto the 34 `crates/protocol-3d` JSON candidates (21 `lib.rs` + 13 `bundle.rs`).

---

## §2 — `crates/mod-host` — WASM mod host (34 rows)

Crate wiring: `civ-mod-host` **is** on a production path — `crates/engine/Cargo.toml`, `crates/server/Cargo.toml`, and `crates/watch/Cargo.toml` all depend on it. The mod platform is real, and several of the behaviors exist. The tags are what is broken.

**Systemic note (TEST-NO-CODE-REF):** all 21 `crates/mod-host/tests/fr_fr_civ_mod_0NN.rs` files are **shape/constructability tests, not requirement tests**. `fr_fr_civ_mod_003.rs` in full:

```rust
#[test]
fn world_domain_variants() {
    use civ_mod_host::WorldDomain;
    // Must compile - variants exist
    let _ = WorldDomain::Economy;
    let _ = WorldDomain::Military;
}
```

`fr_fr_civ_mod_000.rs` asserts `ModType::Policy == ModType::Policy`. None of them exercises a spec gate. **Do not count them as coverage.** (This mirrors finding S3 in the sim/domain lane.)

### `crates/mod-host/src/lib.rs` (9 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `ModType` (lib.rs:53) | FR-CIV-MOD-000 | DATA-SHAPE-ONLY | `load_manifest` — `lib.rs:994` | modding-platform.md:26, AC at :164 | "Mod manifest schema (RON primary / JSON parallel), versioned" is a schema requirement and `ModType` is a legitimate piece of that schema. **But the AC is violated**: the spec requires `manifest.ron` **and** an equivalent `manifest.json` to parse to identical in-memory structs; the loader reads only `manifest.toml` (`CIVMOD_MANIFEST_NAME`, lib.rs:208, used at lib.rs:494) — a third format — and no RON path exists. Weakest DATA-SHAPE-ONLY in this lane. |
| 2 | `ModMeta` (lib.rs:68) | FR-CIV-MOD-000 | DATA-SHAPE-ONLY | same | modding-platform.md:26 | Second `FR-CIV-MOD-000`, on the 20-field manifest struct itself. Same defect, same struct family. |
| 3 | `ModPermissions` (lib.rs:108) | FR-CIV-MOD-002 | CONTAINER-ONLY | none | modding-platform.md:28 (also **collides** with CIV-0700:2364) | Under the design-doc id this is "Material + reaction registration (extends `material.rs` + `laws`)" — `git grep "ReactionRegistrar\|LawRegistrar"` across `crates/**` returns **zero**. Under the CIV-0700 id it is "CPU Budget Enforcement … terminate any mod callback that exceeds 50 µs" — no epoch interruption, no fuel metering, no `ModTimeout` event exists anywhere. **A 10-boolean flag table with `#[serde(default)]` on every field is neither. Fails under both readings of the colliding id.** |
| 4 | `ManifestError` (lib.rs:169) | FR-CIV-MOD-017 | CONTAINER-ONLY | partial, unowned: `read_civmod_archive` / `load_civmod_archive` — `lib.rs:519` | modding-platform.md:43 | "Sharing format — Workshop-style bundle = seed + diff" requires a `.civmod` **content-addressed, signed** bundle with `.civmod-lock` and `.civmod-sig` members; `git grep "civmod-lock\|civmod-sig"` returns **zero** hits repo-wide. The repo ships a plain ZIP with neither. An `#[derive(Error)]` enum describing IO failure is not a distribution format. |
| 5 | `CIVMOD_MANIFEST_NAME` (lib.rs:207) | FR-CIV-MOD-020 | CONTAINER-ONLY | none | modding-platform.md:46 | "Save-game / mod compatibility + migration" — `git grep "min_save_schema\|max_save_schema\|save_schema"` across `crates/mod-host/**` and `crates/engine/src/save_bundle.rs` returns **zero**. No `compat` block, no mod-set recorded in any save, no migration path. The tag is on `pub const CIVMOD_MANIFEST_NAME: &str = "manifest.toml"` — the root path *inside* the ZIP. |
| 6 | `ModLoadedRecord` (lib.rs:211) | FR-CIV-MOD-019 | CONTAINER-ONLY | none | modding-platform.md:45 | "Mod test harness + lint (`civis mod validate`)" — `git grep "mod validate"` across `crates/**/src/**` and `scripts/**` returns **zero**; there is no `civis mod` subcommand. This is the `mod.loaded.v1` lifecycle record, and its own doc comment cites a **different** id (`FR-MOD-004`). Two ids on one symbol, neither of them a validator. |
| 7 | `ModRegistry` (lib.rs:254) | FR-CIV-TACTICS-070 | MIS-BOUND (was NOT-IMPLEMENTED/UNDEFINED) | none | **defined at `docs/traceability/fr-3d-matrix.md:160`** — "Remote mod fetch cache" (see §6; the `no spec definition` note in this row is superseded) | **MIS-BOUND, NOT UNDEFINED.** `ModRegistry` is a `Vec<LoadedMod>` with three phase-logging stubs whose own doc comments say "(WASM callbacks not invoked yet)". The requirement is real and is implemented by `civ-watch`'s `post_mods_fetch` / `list_remote_mods` (`crates/watch/src/mods_api.rs`, tested at `api_tests.rs:1286`). Wrong crate, wrong symbol. |
| 8 | `ModHost` (lib.rs:326) | FR-CIV-MOD-004 | NOT-IMPLEMENTED | none | modding-platform.md:30 — **collides with** CIV-0700:2380 | Design-doc id = "Law / physics-constant extension (extends `crates/laws`)": no `LawRegistrar`, no `constants` block with `[min,max]` clamps. CIV-0700 id = "Mod Determinism Invariant … identical state hash at every tick boundary": no cross-platform replay comparison exists. `ModHost` is a 5-field aggregate of registries and a policy mapper. |
| 9 | `ModHost` (lib.rs:327) | FR-CIV-MOD-020 | NOT-IMPLEMENTED | none | modding-platform.md:46 | Second `FR-CIV-MOD-020`, on the second field of `ModHost` — the same absent save-compatibility requirement as row 5, on a different line of the same struct. |

### `crates/mod-host/src/capability.rs` (3 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `WorldDomain` (capability.rs:20) | FR-CIV-MOD-003 | CONTAINER-ONLY | `ModCapabilitySet::can_read_domain` — `capability.rs:121` | modding-platform.md:29 (collides with CIV-0700:2372) | **The clearest mis-numbering in the lane.** The real behavior is deny-by-default domain/action gating with `ERR_PERMISSION_DENIED = -2` (capability.rs:6), emitting `mod.permission_violation.v1` (line 178) — that is **CIV-0700 FR-CIV-MOD-006, Permission Enforcement**, exactly. The id written is the design-doc's *grammar-registration* slot, whose registrar traits do not exist. `WorldDomain` is a 5-variant `#[repr(i32)]` tag enum. |
| 2 | `ModStatus` (capability.rs:52) | FR-CIV-MOD-004 | NOT-IMPLEMENTED | none | modding-platform.md:30 (collides with CIV-0700:2380) | A lifecycle enum with **no transition function** — constructed at lib.rs:339 and never mutated, so `Suspended` / `Faulted` / `Degraded` are unreachable states. CIV-0700's `FR-CIV-MOD-004` requires a *determinism invariant*; the design doc's requires law/constant extension. Neither is a status enum. |
| 3 | `ModCapabilitySet` (capability.rs:69) | FR-CIV-MOD-003 | CONTAINER-ONLY | `ModCapabilitySet::can_read_domain` / `can_emit_action` — `capability.rs:121,133` | modding-platform.md:29 (collides with CIV-0700:2372) | Second `FR-CIV-MOD-003`, on the struct that holds the bits those two methods read. The behavior is real and tested; the id is wrong (see row 1). |

### `crates/mod-host/src/hooks.rs` (5 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `ModHook` (hooks.rs:10) | FR-CIV-MOD-005 | CONTAINER-ONLY | none | modding-platform.md:31 (collides with CIV-0700:2388) | Design-doc id = "Species / genome primitive registration": no `GenomeRegistrar` / `GenomeCatalog` / `GenomeLocus` exists (`git grep` → zero across `crates/**`). CIV-0700 id = "Non-Deterministic Instruction Rejection": real, but at `scan_wasm_determinism` (determinism.rs:86), not here. `ModHook` is an 8-variant event enum. |
| 2 | `ModHook` (hooks.rs:11) | FR-CIV-MOD-006 | CONTAINER-ONLY | `ModCapabilitySet::can_emit_action` — `capability.rs:133` | modding-platform.md:32 (collides with CIV-0700:2396) | Design-doc id = "Biome / climate rule registration": no `BiomeRegistrar` / `BiomeRule` exists. CIV-0700 id = "Permission Enforcement": real at `capability.rs:121,133`. Second tag on `ModHook`; neither reading is a hook variant. |
| 3 | `HookResult` (hooks.rs:33) | FR-CIV-MOD-007 | CONTAINER-ONLY | partial, unowned: `ModHookEngine::execute` — `hooks.rs:95` | modding-platform.md:33 (collides with CIV-0700:2404) | Design-doc id = "Event hooks (read-only observers + **bounded reactors**)": `execute()` is a real priority-ordered dispatch, but the reactor half is absent — line 96 is literally `let _ = context; // available for future mod-guest calls`, so no handler ever runs. The capability gate the spec requires ("an observer that attempts a substrate write fails the capability check") does not exist in this file. CIV-0700's `FR-CIV-MOD-007` (fault isolation) is also unimplemented. `HookResult` is a 4-variant enum. |
| 4 | `ModHookEngine` (hooks.rs:58) | FR-CIV-MOD-005 | CONTAINER-ONLY | same as row 1 | modding-platform.md:31 | Second `FR-CIV-MOD-005`, on the engine that dispatches them. |
| 5 | `ModHookEngine` (hooks.rs:59) | FR-CIV-MOD-006 | CONTAINER-ONLY | same as row 2 | modding-platform.md:32 | Second `FR-CIV-MOD-006`, on the same struct. |

### `crates/mod-host/src/wasm_guest.rs` (5 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `HOST_IMPORT_MODULE` (wasm_guest.rs:11) | FR-CIV-MOD-008 | CONTAINER-ONLY | `validate_guest_imports` — `wasm_guest.rs:93` (the real "reject before instantiation" behavior) | modding-platform.md:34 (collides with CIV-0700:2412) | Design-doc id = "UI / overlay registration": no `OverlayRegistrar` / `OverlayDef` / `civlab-sdk::ui` exists. CIV-0700 id = "Signature Verification (Ed25519 before instantiation)": real at `signature.rs:26`, **not** here. The tag is on `pub const HOST_IMPORT_MODULE: &str = "civlab"`, a namespace string. |
| 2 | `HOST_CAPABILITY_IMPORTS` (wasm_guest.rs:15) | FR-CIV-MOD-009 | CONTAINER-ONLY | `validate_guest_imports` — `wasm_guest.rs:93` | modding-platform.md:35 (collides with CIV-0700:2420) | Design-doc id = "Charter validator — reject hardcoded-outcome mods": `git grep -i "charter" -- crates/mod-host/src/**` returns **zero**. CIV-0700 id = "Scenario Registration and Init (`ScenarioDescriptor` → `WorldState`)": no `ScenarioDescriptor` type exists. A 7-element `&[&str]` allowlist is neither. |
| 3 | `HOST_CAPABILITY_API_VERSION` (wasm_guest.rs:27) | FR-CIV-MOD-008 | CONTAINER-ONLY | same as row 1 | modding-platform.md:34 | Second `FR-CIV-MOD-008`, on `pub const … : i32 = 1`. Note: this const is the *natural* home for CIV-0700's `FR-CIV-MOD-003` (API Version Compatibility), and nothing compares it. |
| 4 | `HOST_GUEST_MEMORY_CAP` (wasm_guest.rs:31) | FR-CIV-MOD-009 | CONTAINER-ONLY | same as row 2 | modding-platform.md:35 | Second `FR-CIV-MOD-009`, on `pub const … : usize = 65_536`. A memory ceiling is not a charter validator. |
| 5 | `HostState` (wasm_guest.rs:35) | FR-CIV-MOD-008 | CONTAINER-ONLY | same as row 1 | modding-platform.md:34 | Third `FR-CIV-MOD-008`, on a 4-field per-instance store. |

### `crates/mod-host/src/determinism.rs` (2 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `DeterminismError` (determinism.rs:8) | FR-CIV-MOD-012 | CONTAINER-ONLY | `scan_wasm_determinism` / `scan_wasm_determinism_report` — `determinism.rs:86,50` | modding-platform.md:38 (collides with CIV-0700:2444) | Design-doc id = "Load ordering (topological + priority + deterministic tie-break)": `git grep "topolog"` in `crates/mod-host/src` returns **zero**; there is no dependency graph, no `priority` field on `ModMeta`, and `ModRegistry::register` is a bare `Vec::push` in call order. CIV-0700 id = "Mid-Simulation Mod Swap (`sim.mod.swap()`)": no such method exists in `JsonRpcMethod`. The two scan functions are real — and are tagged `FR-CIV-MOD-013`, a **third** numbering matching neither spec. |
| 2 | `DeterminismScanReport` (determinism.rs:35) | FR-CIV-MOD-012 | CONTAINER-ONLY | same as row 1 | modding-platform.md:38 | Second `FR-CIV-MOD-012`, on the report struct the two scan functions return. |

### `crates/mod-host/src/guest_state.rs` (5 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `MOD_GUEST_STATE_VERSION` (guest_state.rs:6) | FR-CIV-MOD-010 | CONTAINER-ONLY | partial, unowned: `load_manifest_dir` / `load_civmod_archive` — `lib.rs:492,519` | modding-platform.md:36 (collides with CIV-0700:2428) | Design-doc id = "Mod loading pipeline (discover → parse → validate → resolve → bind)": the real loader performs **only** parse + determinism-scan + signature-verify + register. There is no charter-validate, no dependency-resolve, no ordering, no law-merge, no capability-grant, no bind stage. CIV-0700 id = "Action Validation and Conservation (double-entry, `ModActionRejected` event)": `git grep "ModActionRejected"` returns **zero**. The tag is on `pub const … : u32 = 1`. |
| 2 | `ModGuestMemoryBlob` (guest_state.rs:10) | FR-CIV-MOD-010 | CONTAINER-ONLY | same as row 1 | modding-platform.md:36 | Second `FR-CIV-MOD-010`, on a byte-blob wrapper for guest linear memory. |
| 3 | `ModGuestStateSave` (guest_state.rs:20) | FR-CIV-MOD-010 | CONTAINER-ONLY | same as row 1 | modding-platform.md:36 | Third `FR-CIV-MOD-010`, on the save struct (`to_json` / `from_json` at :40,:45 are real and round-trip tested). |
| 4 | `GuestStateError` (guest_state.rs:55) | FR-CIV-MOD-011 | CONTAINER-ONLY | none | modding-platform.md:37 (collides with CIV-0700:2436) | Design-doc id = "Dependency + version + capability model (semver)": `ModDependencies.civlab_api` (lib.rs:101) is parsed but **never checked** — no comparison exists, and `ModMeta.api_version` is never compared against a host range, so CIV-0700's "reject at load time … `IncompatibleApiVersion`" cannot fire and no such error variant exists. CIV-0700 id = "Custom Good Type Registration": no `GoodType` anywhere. A 2-variant error enum whose real content is a JSON parse failure is neither. |
| 5 | `ModBrowserEntry` (guest_state.rs:66) | FR-CIV-TACTICS-062 | MIS-BOUND (was NOT-IMPLEMENTED/UNDEFINED) | none | **defined at `docs/traceability/fr-3d-matrix.md:152`** — "Mod catalog + runtime install" (see §6; the `no spec definition` note in this row is superseded) | **MIS-BOUND, NOT UNDEFINED.** A 7-field UI/RPC row whose own doc comment calls it a "mod browser **stub**". The requirement is real and is implemented by `civ-watch`'s `post_mods_install` (`crates/watch/src/mods_api.rs`, tested at `api_tests.rs:681`). Wrong crate, wrong symbol. |

### `policy_action.rs`, `signature.rs`, `float_data_flow.rs` (5 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `PolicyActionKind` (policy_action.rs:9) | FR-CIV-MOD-002 | CONTAINER-ONLY | `policy_action_to_emit_type` — `policy_action.rs:53` | modding-platform.md:28 (collides with CIV-0700:2364) | Second `FR-CIV-MOD-002`. The discriminant mapper is real and round-trip-tested, but it maps action ids; it registers no materials or reactions (design-doc id) and enforces no CPU budget (CIV-0700 id). |
| 2 | `PolicyActionKind` (policy_action.rs:10) | FR-CIV-MOD-015 | CONTAINER-ONLY | none | modding-platform.md:41 (collides with CIV-0700:2468) | Design-doc id = "Stable mod API surface (trait + ABI contract, semver'd)": the nine registrar traits the spec freezes (spec:380-382) do not exist, and there is no `SCHEMA_VERSION` for the mod API. CIV-0700 id = "Mod Status Telemetry (status/fault count/timeout count via the metrics endpoint)": no mod counters reach `crates/server/src/metrics.rs`. Second tag, same enum. |
| 3 | `MOD_WASM_SIG_NAME` (signature.rs:7) | FR-CIV-MOD-014 | CONTAINER-ONLY | `verify_wasm_signature` — `signature.rs:26` (real, Ed25519, tamper-tested) | modding-platform.md:40 (collides with CIV-0700:2460) | Design-doc id = "Hot-reload of mods (data-tier live, code-tier staged)": `ModHost::reload_mod` is an unload-then-reload of the same directory — no file watcher, no copy-on-write catalog swap, no staged code tier, and the spec's "conflict on reload keeps the prior version active" is not honored (a failed reload leaves the mod unloaded). CIV-0700 id = "Lua Script Parity": no Lua anywhere (`git grep -i "lua" -- crates/` → zero). The tag is on `pub const MOD_WASM_SIG_NAME: &str = "mod.wasm.sig"`. |
| 4 | `SignatureError` (signature.rs:11) | FR-CIV-MOD-014 | CONTAINER-ONLY | `verify_wasm_signature` — `signature.rs:26` | modding-platform.md:40 (collides with CIV-0700:2460) | Second `FR-CIV-MOD-014`, on the error enum. **The correct tag for this file is CIV-0700 `FR-CIV-MOD-008` (Signature Verification) and it is not written anywhere.** |
| 5 | `FloatContaminationSite` (float_data_flow.rs:7) | FR-CIV-MOD-016 | CONTAINER-ONLY | none | modding-platform.md:42 | "Conflict detection + resolution (ID collisions, law contradictions)": the spec's conflict table (spec:402-409) needs a post-merge id scan, `LawDb::validate` over the union, and a constant-clash priority rule; none exist. `FloatContaminationSite` is a 3-field diagnostic record for float→`action_emit` data flow — a different feature entirely, and correctly left untagged by any other id. |

---

## §3 — `crates/civis-mcp` — MCP JSON-RPC proxy (2 rows)

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `TOOL_NAMES` (lib.rs:54) | FR-CIV-MCP-002 | CONTAINER-ONLY | none | civ-017 spec:38-42 | "The MCP server SHALL register read-only HTTP tools for `/terrain`, `/snapshot` … and SHALL NOT call any mutating HTTP route (`/control/*`) without an explicit `--allow-mutations` flag" — `git grep "allow_mutations\|allow-mutations" -- crates/civis-mcp/**` returns **zero**. The list at :58-161 is 100+ tool names with **no** route kind attached, and it includes obviously-mutating tools (`civis_place_voxel`, `civis_god_action_earthquake`, `civis_spawn_civilian`, `civis_set_policy`, `sim_undo`, `sim_reset`) with no read-only gate. |
| 2 | `HARNESS_VERSION` (lib.rs:163) | FR-CIV-MCP-005 | CONTAINER-ONLY | none | civ-017 spec:49-51 | "Configuration SHALL be read from environment variables **only** (`CIVIS_MCP_CIV_SERVER_URL`, `CIVIS_MCP_CIV_WATCH_URL`, `CIVIS_MCP_AUTH_TOKEN`); no hardcoded URLs, ports, or secrets" — `git grep "CIVIS_MCP_CIV_SERVER_URL\|CIVIS_MCP_CIV_WATCH_URL\|CIVIS_MCP_AUTH_TOKEN" -- crates/civis-mcp/**` returns **zero**. The three named variables are never read. `HARNESS_VERSION` is `env!("CARGO_PKG_VERSION")`, a compile-time version string, not configuration. |

---

## §4 — `crates/engine/src/replay.rs` — `.civreplay` serialization (2 rows)

Picked up because the `CIV-1000` save/load chain is the substrate the mod-host guest-state and hash-chain rows depend on, and because `FR-SAVE-009` is the one in-lane row with a **fully implemented** requirement.

| # | Tagged symbol (file:line) | ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|---|---|
| 1 | `ReplayLog` (replay.rs:146) | FR-SAVE-009 | IMPLEMENTED-BY-BEHAVIOR | `HashChainState::advance` — `crates/engine/src/hash_chain.rs:58`; `ReplayLog::record_tick` — `replay.rs:371,383`; `check_integrity_after_replay_load` — `integrity.rs:128` | CIV-1000:2808 | "The BLAKE3 hash chain tail SHALL be serialized and restored on load, enabling the chain to continue unbroken from the saved tick" is genuinely discharged: `running_hash: Option<[u8; HASH_LEN]>` is `#[serde(default)]` (replay.rs:160) so it round-trips through `.civreplay`, and `integrity.rs` fails the load when the stored tail disagrees with the recomputed chain. The tag sits on the container, but the requirement is materially met and tested. |
| 2 | `ReplayLog` (replay.rs:146) | FR-CIV-RTS-015 | NOT-IMPLEMENTED | none | CIV-0300 §12.1 via `docs/specs/` (client-side prediction) | "Client-Side Prediction & Replay Correction — smooth unit movement interpolation … snap correction < 100 ms" is a **client** interpolation behavior. `ReplayLog` is a server-side event recorder. No prediction or smoothing code exists in `crates/`. **Out-of-lane id on an in-lane file**; recorded here only because it shares the tag line. |

---

## §5 — TEST-NO-CODE-REF rows

A **TEST-NO-CODE-REF** row is one where a test file exists and asserts something, but no source symbol implements the requirement. Flagged as requested:

| ID | Test file | What the test asserts | Why it is not evidence |
|---|---|---|---|
| FR-CIV-MOD-000 | `crates/mod-host/tests/fr_fr_civ_mod_000.rs` | `ModType::Policy == ModType::Policy`; `meta.id == "test-mod"` | Constructability only. The spec's AC (`modding-platform.md:164`) requires RON **and** JSON manifests parsing to an identical in-memory struct; the loader reads only `manifest.toml`. |
| FR-CIV-MOD-003 | `crates/mod-host/tests/fr_fr_civ_mod_003.rs` | `WorldDomain::Economy` exists; `allow_all` grants all | The test's own header says "Verifies capability set and world domain access control" but only checks `can_read_domain` true/false. It also references `civ_mod_host::CapabilitySet`, an alias that does not exist in `lib.rs` (the type is `ModCapabilitySet`) — the test cannot compile as written. |
| FR-CIV-MOD-012 | `crates/mod-host/tests/fr_fr_civ_mod_012.rs` | — | Tagged id is a load-ordering requirement; no ordering code exists for the test to cover. |
| FR-CIV-MOD-013 | `crates/mod-host/tests/fr_fr_civ_mod_013.rs` | `scan_wasm_determinism_report(b"")` returns a parse error | Asserts an **error** on empty input. Never asserts a rejection of a real non-deterministic opcode, which is the requirement. |
| FR-CIV-MCP-002 | (no `tests/` dir under `crates/civis-mcp`) | — | The spec's own FR-CIV-MCP-003 requires a contract test per tool; none exists. |
| FR-CIV-PROTO-010 | (none) | — | No query API, no test. |

**All 21 `crates/mod-host/tests/fr_fr_civ_mod_*.rs` files carry this defect class.** They were not individually tabled above because none of them changes a verdict — the underlying behavior is absent or mis-numbered regardless.

---

## §6 — UNDEFINED-ID

**SUPERSEDED — the two UNDEFINED-ID findings in this section were wrong, and have been re-adjudicated.** See the corrections below. The rows are retained so the error stays visible and traceable rather than being quietly deleted.

| ID | Tagged at | Status |
|---|---|---|
| `FR-CIV-TACTICS-070` | `crates/mod-host/src/lib.rs:254` (`ModRegistry`) | **MIS-BOUND, NOT UNDEFINED.** *Originally recorded here as UNDEFINED-ID on the strength of the detector's `defined_by_spec: false` and an empty `requirement` field — both of which this same report shows to be unreliable (see the next two rows).* It is in fact defined at `docs/traceability/fr-3d-matrix.md:160` as "Remote mod fetch cache", and discharged by `civ-watch`'s `post_mods_fetch` and `list_remote_mods` handlers (`crates/watch/src/mods_api.rs`, tested at `api_tests.rs:1286`). The tag is still wrong, but for a different reason: `ModRegistry` is a `Vec<LoadedMod>` in the wrong crate whose phase stubs say "WASM callbacks not invoked yet". |
| `FR-CIV-TACTICS-062` | `crates/mod-host/src/guest_state.rs:66` (`ModBrowserEntry`) | **MIS-BOUND, NOT UNDEFINED.** *Originally UNDEFINED-ID, same error.* Defined at `docs/traceability/fr-3d-matrix.md:152` as "Mod catalog + runtime install", discharged by `civ-watch`'s `post_mods_install` handler (`crates/watch/src/mods_api.rs`, tested at `api_tests.rs:681`). `ModBrowserEntry` is a 7-field stub row in the wrong crate and implements none of it. |
| `FR-CIV-PROTO-010` | `protocol-3d/src/lib.rs:62,86,107` (4 tag rows) | **Not undefined**, but the JSON marks it `defined_by_spec: false` with an empty `requirement` and the detector's extracted text is a table-row fragment. Real sentence recovered from CIV-0200:1169. The flag is a detector false-negative, not an absent requirement — and the requirement itself is still unimplemented (no `sim.query` method). |
| All 34 `FR-CIV-PROTO-*` rows | `protocol-3d/**` | **Not undefined, but the `requirement` field is corrupt** for 30 of them (table-row fragments such as `: Snapshot Filtering`). Any tool trusting that field is reading garbage. |

The lesson this section earns the hard way: `defined_by_spec: false` and an empty `requirement` are properties of the detector's index, not of the spec tree. Two ids were called UNDEFINED on that basis and both turned out to be defined in `docs/traceability/fr-3d-matrix.md` and implemented in a third crate. An UNDEFINED-ID verdict requires a search of the spec tree itself, not a field in generated JSON.

**Colliding (not undefined, but ambiguous) IDs:** `FR-CIV-MOD-002` through `FR-CIV-MOD-015` each have **two mutually incompatible definitions** (CIV-0700 §15 vs `modding-platform.md` §0). A third numbering (`FR-CIV-MOD-013`, used at `determinism.rs:49,85` and `tests/fr_fr_civ_mod_013.rs`) matches neither. These 14 ids cannot be resolved without a decision from the spec owners.

---

## Highest-confidence FALSE tags

Not judgment calls. The requirement text is explicit and the tagged artifact cannot satisfy it under any reading:

1. **`FR-CIV-MOD-002..015` (24 tag rows) — namespace collision.** The in-source numbering is the design-doc's; the source code implements the CIV-0700 numbering; the two never meet. `docs/traceability/fr-civ-mod-003/fr-civ-mod-003-spec.md` names both authorities for one id, confirming the merge.
2. **`FR-CIV-PROTO-012/013/014/015` (8 tag rows) — the named clients do not exist.** `example_bevy_client`, `example_unreal_client`, `example_unity_client`, `example_web_client` appear **only** inside `docs/fragmented/`. There is no Unity client in this repository at all (`git grep -l "Unity" -- clients/` → `unreal-show/README.md` only). Four FRs are tagged on a magic constant, a zstd default, a newtype, and an options struct.
3. **`FR-CIV-PROTO-008` (2 rows) — priority ordering does not exist.** `client_priority` has exactly one repo-wide hit: the doc comment of the test that would have caught it.
4. **`FR-CIV-MOD-009` (2 rows) — no charter validator.** `git grep -i charter -- crates/mod-host/src/**` → zero. A mod that hardcodes an outcome is not rejected because nothing checks.
5. **`FR-CIV-MOD-020` (3 rows) — no save/mod compatibility.** `save_schema`, `min_save_schema`, `max_save_schema` → zero hits.
6. **`FR-CIV-MOD-019` — no `civis mod validate`.** The CLI subcommand does not exist.
7. **`FR-CIV-MOD-017` — no `.civmod-lock`, no `.civmod-sig`.** The "content-addressed, signed" bundle spec is met by a plain unsigned ZIP.
8. **`FR-CIV-MCP-005` — the three mandated env vars are never read.** Zero hits.
9. **`FR-CIV-MCP-002` — no read-only gate and no `--allow-mutations` flag**, in a tool list that is majority-mutating.
10. **`FR-CIV-TACTICS-062` and `FR-CIV-TACTICS-070`** — **MIS-BOUND, NOT UNDEFINED** (corrected; this entry previously said UNDEFINED-ID, which was wrong). Both are defined in `docs/traceability/fr-3d-matrix.md` (`:152` and `:160`) and implemented in `civ-watch`, not in `mod-host`. The tags on `ModRegistry` and `ModBrowserEntry` are still false, but the ids are real.

## Recommended follow-up (read-only audit)

1. **Resolve the `FR-CIV-MOD-*` collision before any tag tooling runs again.** One of the two taxonomies must be retired or the two namespaces must be renamed (`FR-CIV-MODPLAT-*` vs `FR-CIV-MODSEC-*`). Until then, no automatic verdict on this namespace is trustworthy, and the `fr_fr_civ_mod_*.rs` tests are validating the wrong requirements.
2. **Fix the detector's `load_spec_texts()`.** It accepts a mid-table fragment (`": Snapshot Filtering"`) as a requirement sentence. It should reject any extracted string that begins with `:` or that came from a line containing more than one `|`. This mis-grades 30 of 34 rows in `protocol-3d` alone.
3. **Re-tag the behaviors that are real and currently untagged.** These exist, are tested, and carry no correct id: `ModCapabilitySet::can_read_domain`/`can_emit_action` (CIV-0700 `FR-CIV-MOD-006`), `verify_wasm_signature` (CIV-0700 `FR-CIV-MOD-008`), `scan_wasm_determinism` (CIV-0700 `FR-CIV-MOD-005`), `validate_guest_imports` (NFR-CIV-SEC-001, already documented in-file at wasm_guest.rs:75), and `SubscriptionFilter::filter_frames` (CIV-0200 `FR-CIV-PROTO-005`). Meanwhile the tags that *are* wrong point at constants.
4. **Delete or quarantine the 21 `crates/mod-host/tests/fr_fr_civ_mod_*.rs` stubs.** They inflate apparent coverage while asserting variant existence. `fr_fr_civ_mod_003.rs` additionally references a type alias (`CapabilitySet`) that does not exist, so it does not even compile.

No source file was modified by this audit. This markdown file is the only artifact created.
