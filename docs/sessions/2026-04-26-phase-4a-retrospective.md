# Phase 4a bridge HMAC retrospective (2026-04-26)

## Summary
Phase 4a (server-side bridge HMAC + receipt emission) landed in iteration 41. 8/8 HMAC tests pass + 55/55 unit + 51/51 integration tests pass. Zero regressions on the bridge surface. This retrospective captures empirical learnings for Phase 4b/4c.

## What landed
- `src/Runtime/Bridge/SessionHmac.cs` — 256-bit per-session key, IDisposable, scrubs key on dispose, exposes `ComputeHmac(timestamp, frame, stateHash)` using HMAC-SHA256.
- `src/Bridge/Protocol/BridgeReceipt.cs` — DTO with snake_case wire keys (`session_id`, `timestamp`, `world_frame`, `state_sha256`, `hmac`).
- GameBridgeServer modifications:
  - Added `connect` JSON-RPC method handling handshake (returns `session_id` + `session_key_b64`).
  - `SerializeSuccess`/`SerializeError` now attach `BridgeReceipt` to every response except connect.
  - Canonicalizer (`CanonicalizeJson` + `CanonicalizeToken`) — sorted keys, no whitespace, RFC-8785-shaped per spec section 6.
- 8 HMAC tests covering uniqueness, determinism, tamper detection, base64 round-trip, JSON serialization (see `BridgeHmacTests` for exact methods: `SessionHmac_GeneratesUniqueKeyPerInstance`, `SessionHmac_HmacIsDeterministicForSameInputs`, `SessionHmac_HmacChangesWhenInputChanges`, `SessionHmac_DifferentKeysProduceDifferentHmacs`, `SessionHmac_KeyMaterialB64_RoundTrips`, `BridgeReceipt_RoundTripsThroughJsonSerialization`, `JsonRpcResponse_BridgeReceiptSerializesUnderSnakeCaseKey`, `JsonRpcResponse_OmitsBridgeReceiptWhenNull`).

## Key design choices that worked
1. **HMAC-SHA256 over canonical JSON** rather than over raw JSON — prevents whitespace-induced false negatives.
2. **Per-session ephemeral key** (256-bit, `RandomNumberGenerator.Fill`) — no key persistence, no rotation logic, simple lifecycle.
3. **Connect handshake transmits key once** in plaintext over local pipe — acceptable for our threat model (in-process tampering, not network).
4. **Receipt attached as sibling field** in `JsonRpcResponse` rather than wrapping the whole envelope — backward-compatible: clients ignoring `bridge_receipt` field continue to work.
5. **Connect response excludes its own receipt** (chicken-and-egg) — clean special case rather than bootstrap-key-derivation gymnastics. `JsonRpcResponse_OmitsBridgeReceiptWhenNull` guards this.
6. **`long microseconds` for world_frame** rather than DOTS `Time.ElapsedTime` ticks — fits in long, monotonic, NaN-guarded.

## Surprises
- **`JsonRpcResponse` needed an optional sibling field**, not a wrapper. Original spec was ambiguous; implementation chose sibling for backward compat.
- **`_payloadHasher` field needed thread-safety consideration** — currently shares a SHA256 instance across threads. Phase 4b should verify this; `SHA256.HashData()` (static) avoids the issue if .NET 5+.
- **`CanonicalizeToken` handles 6+ JToken types** — Object, Array, Property, Integer, Float, String, Boolean, Null. Edge cases: empty arrays/objects, integer overflow, floating-point precision (no normalization → potential cross-impl drift on doubles).
- **`BridgeHmacTests`** uncovered a bug in initial implementation where `_startTime` (used for diagnostic uptime) accidentally affected receipt timing. `SessionHmac_HmacIsDeterministicForSameInputs` caught it. Validates the 8-test approach.

## Phase 4b implementation contract (DO NOT VARY)
1. **Canonical JSON must be byte-identical** between server (`GameBridgeServer.CanonicalizeJson`) and client (new `Bridge.Client.CanonicalJson`). Any divergence → all HMACs fail. Strongly recommend extracting to a shared netstandard library if not already.
2. **Receipt order**: server attaches receipt AFTER serializing the result payload. Client recomputes by hashing the result payload (excluding `bridge_receipt` field) and comparing.
3. **Frame monotonicity**: client tracks `_lastFrame` per `session_id`. Frame=0 is sentinel for connect; otherwise must increase strictly.
4. **VerificationMode default**: `WarnOnly` in Phase 4b. Phase 4c flips to `Strict`.
5. **Missing receipt + Strict** → throw `GameClientException("hmac_invalid_receipt_missing")`. Missing receipt + WarnOnly → log + continue.

## Phase 4b open questions
1. Should the client cache `session_keys` in memory only, or persist for reconnect-with-same-session?
   - Recommended: in-memory only. Reconnect generates new `session_id`.
2. Should the verifier tolerate clock skew between server and client?
   - Recommended: no. Server stamps timestamp; client doesn't compare to local time. HMAC validates the timestamp wasn't tampered with — that's enough.
3. Should the receipt include a sequence number in addition to `world_frame`?
   - Recommended: `world_frame` is sufficient. Sequence-number adds wire size for redundant info.

## Migration path for Phase 4c
Phase 4c flips the default from `WarnOnly` to `Strict`. Two pre-conditions:
1. All existing tests pass with `VerificationMode.Strict` (verify by running test suite with default flipped).
2. Bundle aggregator (separate task) collects `bridge_receipts` during runs, includes in proof bundle, gets cosign-signed at bundle time.

The flip is one line: `public VerificationMode HmacVerificationMode { get; set; } = VerificationMode.Strict;`. Should land in a single PR with a CHANGELOG note.

## Cross-references
- Spec: `docs/design/2026-04-25-bridge-hmac-phase4.md`
- Phase 1+2+3 closures: TRUTH_TABLE.md updates #71-#75.
- Phase 4a closure: TRUTH_TABLE.md update #79.
- Open follow-up: bundle aggregator (gather `bridge_receipts` during test runs, attach to proof bundle).
