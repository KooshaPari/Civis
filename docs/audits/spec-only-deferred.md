# Spec-only IDs deferred by agent-E (P2)

This ledger records every ID from the P2-agent-E slice that we chose to
**defer** rather than write a source impl for in this round, with a one-line
reason. The full slice is in
[`P2-agent-E.md`](P2-impl-slice/P2-agent-E.md); the deferred entries below
are the ones that did **not** get a `// FR-XYZ-NNN` tag in source.

Bias: the P2 task spec caps each implementation at ≤20 lines per function
and forbids new dependencies. Anything that would exceed that budget — or
require a new crate (e.g. `blake3`, `rand_chacha`) or a multi-thousand-line
new module — is deferred here for a future agent whose scope allows it.

## Deferred — P2 agent-E

| ID | Epic | Reason |
|---|---|---|
| FR-SAVE-006 | FR-SAVE | BLAKE3 integrity hash for save blobs. Requires the `blake3` crate (new dependency). Defer to a follow-up that wires `blake3` into `civ-save-db`. |
| FR-SAVE-008 | FR-SAVE | ChaCha20Rng state serialization (20 u32 words + sub-block word count). Requires `rand_chacha::ChaCha20Rng::from_state` introspection on the engine side. Out of scope for a save-DB-only patch. |
| FR-SAVE-009 | FR-SAVE | BLAKE3 hash chain tail serialization. Same `blake3` dependency issue as FR-SAVE-006. |
| FR-SAVE-010 | FR-SAVE | AI state serialization (personality, goals, memory, MCTS cache, threat models). Multi-module change in `civ-ai`; not appropriate for a save-DB-only patch. |
| FR-SAVE-011 | FR-SAVE | WASM `ModStateSave` trait + per-mod state on save/load. Requires extending `civ-mod-host` and a host-side state registry. |
| FR-SAVE-012 | FR-SAVE | "Unknown mod ID → warn + skip" semantics on load. Coupled to FR-SAVE-011 — defer together so the warn-on-skip path has an actual mod registry to consult. |
| FR-SAVE-013 | FR-SAVE | Save format version + N-2..N-1 migration. Requires a `MigrateFn` table and per-version transformers; medium module, deferred. |
| FR-SAVE-016 | FR-SAVE | QuickSave ≤50ms SLO. Performance target — needs the QuickSave path to actually exist in the engine before we can measure against it. |
| FR-SAVE-017 | FR-SAVE | SlotSave ≤500ms SLO. Same as FR-SAVE-016 — performance budget, not a feature. |
| FR-SAVE-018 | FR-SAVE | Load ≤1,000ms SLO. Same as FR-SAVE-016 — performance budget, not a feature. |
| FR-SAVE-019 | FR-SAVE | Save integrity verification ≤200ms SLO. Performance budget; depends on FR-SAVE-006's BLAKE3 path existing. |
| NFR-CIV-SEC-002 | NFR-CIV-SEC | "No secret material in committed config" — this is a CI-infra concern (a `trufflehog`/`gitleaks` step in `.github/workflows`). Out of Rust scope. |
| NFR-CIV-SEC-003 | NFR-CIV-SEC | "Tests run in an isolated network namespace" — CI-infra concern (Docker `--network=none` etc.). |
| NFR-CIV-SEC-004 | NFR-CIV-SEC | "Dependency audit + Bandit/Semgrep pass on every PR" — CI-infra concern. |

## Implemented in P2 agent-E (for completeness)

The following IDs in the P2-agent-E slice **were** implemented (tagged and
tested). They are listed here only so a reader can sanity-check the
inverse — every ID not in the deferred table above should appear in a
`git log` commit on `next-P2-E`:

- FR-CIV-ASSET-QUAL-001 — `crates/asset-pipeline/src/validate.rs`
- FR-CIV-GODOT-ATTACH-001..004 — `clients/godot-ref/rust/src/attach.rs`
- FR-CIV-MIGRATION-001..005 — `crates/emergence-migration/src/lib.rs`
- FR-CIV-SOCIAL-002-IDEOLOGY — `crates/social/src/ideology.rs`
- FR-SAVE-007, FR-SAVE-014, FR-SAVE-015, FR-SAVE-021, FR-SAVE-022,
  FR-SAVE-023, FR-SAVE-024, FR-SAVE-025 — `crates/save-db/src/lib.rs`
- FR-SAVE-020 — `crates/save-db/src/lib.rs` (`evict_autosaves`)
- NFR-CIV-LEGENDS-LOUD-03 — `crates/legends/src/{decay.rs,worker.rs}`
- NFR-CIV-SEC-001 — `crates/mod-host/src/wasm_guest.rs`

## Counts

Deferred: **13** IDs.
Implemented: **22** IDs (covering 17 unique spec IDs + 5 social/migration
batch tags).
