# Requirement-ID provenance corrections (2026-09-29)

Adversarial review of the traceability matrix found that a set of
`crates/render` modules advertised implementations of requirement IDs that
**no authoritative spec defines**. This document records what was found, what
was changed, and how recurrence is prevented.

## The defect

`docs/traceability/TRACEABILITY_MATRIX.md` contains hand-written tables
claiming implementations for nine requirement families. Those tables cite
three source spec files:

| Cited source spec | Exists? |
|---|---|
| `docs/specs/CIV-0300-ui-ux.md` | **no** |
| `docs/specs/CIV-0600-2d-assets.md` | **no** |
| `docs/specs/CIV-0601-3d-assets.md` | **no** |

The real files are `CIV-0300-rts-ui-ux-spec.md`, `CIV-0600-2d-asset-pipeline-spec.md`,
and `CIV-0601-3d-asset-transition-and-agentic-gen-spec.md`. None of the real
files defines `FR-UX-*`, `FR-ASSET-*`, `FR-AUD-*`, or `FR-PERF-003`; the real
specs number their requirements `FR-CIV-RTS-*`, `FR-CIV-GEO-*`, `FR-CIV-ASSET-*`,
`FR-CIV-3D-*`, and `FR-CIV-AUDIO-*`.

Each `crates/render` module doc header quoted a normative sentence
("The UI SHALL render the hex map …", "LOD transitions SHALL be visually
seamless …") that existed **only in the traceability matrix itself**. The
source and the audit were citing each other.

### Why this is worse than a missing link

`FR-UX-001` through `FR-UX-005` are *real* IDs in
`docs/models/civ-sim/USER_SPEC.md`, where they mean something entirely
different:

| ID | `USER_SPEC.md` meaning (the bound spec) | what the render crate claimed |
|---|---|---|
| `FR-UX-001` | Every chart must display its provenance badge | hex map renderer at 60 fps |
| `FR-UX-002` | Exported artifacts must include assumption disclosure | RTS camera pan/zoom/select |
| `FR-UX-003` | Run IDs must be stable, unique, human-readable | timeline scrubber rewind |
| `FR-UX-004` | Replay references must be self-contained | single-frame LOD transition |
| `FR-UX-005` | Tick state hashes must be visible in timeline | event-derived UI state |

Because the matrix bound these IDs to `USER_SPEC.md`, five rows were marked
`COVERED` on the strength of code that implements none of the cited text. That
is false coverage, which is worse than an admitted gap.

## Scope

| Measure | Count |
|---|---|
| Requirement IDs tagged in `crates/**/*.rs` | 1089 |
| IDs defined by at least one authoritative spec | 900 |
| **Source-tagged IDs with no authoritative spec** | **371** |
| …of which exist only in `docs/traceability/**` scaffolding (no requirement text) | 291 |
| Render-crate IDs in this specific defect | 9 |

Of the 371, **291** appear nowhere except unfilled `SPEC-TEMPLATE` scaffolding
under `docs/traceability/`; the remainder are defined by specs outside the
narrow roots the first classifier used. Both groups are unbacked, and both are
now reported by the new tooling.

## Corrections applied

| Module | Removed ID(s) | Genuine binding now |
|---|---|---|
| `render::hex_map` | `FR-UX-001` | none (no ID describes hex-map rendering) |
| `render::camera` | `FR-UX-002` | none |
| `render::timeline` | `FR-UX-003` | none |
| `render::lod` | `FR-UX-004` | none |
| `render::state` | `FR-UX-005` | none |
| `render::atlas` | `FR-ASSET-001..003` | none (`FR-CIV-ASSET-011` is a 30 s build-time gate, not atlas packing) |
| `render::gltf` | `FR-ASSET-004` | none |
| `render::frame` | `FR-PERF-003` | none (`FR-PERF-002` is an entity-count constraint, deliberately not claimed) |
| `render::audio` | `FR-AUD-001..003` | **`FR-CIV-AUDIO-004`** (adaptive score from mood stems) — the one audio requirement this code genuinely implements |

IDs were **removed rather than rebound** wherever no authoritative requirement
describes the behavior. Rebinding to a merely-adjacent ID would repeat the
original sin with better paperwork.

### Tests

The five `crates/render/tests/fr_fr_ux_00N.rs` files and four
`fr_fr_asset_00N.rs` files were renamed so they no longer claim credit for
IDs they do not implement. **Their assertions are real and were kept** — a
genuine behavioral test of the code, simply no longer mislabelled:

| Old name | New name |
|---|---|
| `fr_fr_ux_001.rs` | `hex_map_draw_list.rs` |
| `fr_fr_ux_002.rs` | `rts_camera_controls.rs` |
| `fr_fr_ux_003.rs` | `timeline_scrubber_rewind.rs` |
| `fr_fr_ux_004.rs` | `lod_seamless_transition.rs` |
| `fr_fr_ux_005.rs` | `state_from_events_only.rs` |
| `fr_fr_asset_001.rs` | `atlas_svg_rasterised.rs` |
| `fr_fr_asset_002.rs` | `atlas_packed_per_lod.rs` |
| `fr_fr_asset_003.rs` | `atlas_build_events.rs` |
| `fr_fr_asset_004.rs` | `gltf_lazy_loaded.rs` |

Each file header now carries a provenance note explaining the change.
`cargo test -p civ-render`: 41 lib + 9 integration + 4 doc-tests pass.

## `FR-SAVE-009` — a genuine gap, now closed

The same review found a real miss in the opposite direction: `FR-SAVE-009`
("the BLAKE3 hash chain tail SHALL be serialized and restored on load,
enabling the chain to continue unbroken") **was** implemented and tested at
the `civreplay` codec level, but carried no source tag, so the matrix recorded
it as `SPEC-ONLY`.

- Tagged `ReplayLog::running_hash` in `crates/engine/src/replay.rs`.
- Added two behavioral tests in `crates/engine/src/save_bundle.rs` that
  exercise the *whole* bundle path (tar + zstd), not just the codec:
  - `fr_save_009_hash_chain_tail_survives_bundle_roundtrip` — asserts the tail
    is restored, re-verifies against its own events, and that continuing the
    chain after load equals an uninterrupted run.
  - `fr_save_009_tampered_chain_tail_is_rejected_on_load` — corrupts the
    embedded tail and asserts the load fails, proving the tail is load-bearing.
- **Mutation-checked**: neutering `hash_chain_root()` makes the positive test
  fail, so it is not vacuously green.

`FR-SAVE-008`'s recorded deferral reason was wrong and is corrected: the
ledger claimed it needs `ChaCha20Rng::from_state` introspection, but
`rand_chacha 0.3` is already a dependency. The actual gap is that the engine
uses `ChaCha8Rng` and persists only `rng_seed: u64`, never capturing stream
position or a word count.

## Prevention

- `docs/audits/_verify_bindings.py` — flags (a) `EPIC-PAIRING-FALSE`, where
  source claims `(FR-XXX-NNN, CIV-NNNN)` but that epic defines no such ID;
  (b) `UNDEFINED-TAG`, IDs with no authoritative spec; (c) `MATRIX-MISBINDING`,
  rows whose cited `path:line` does not contain the row's ID.
- `docs/audits/_detect_id_collisions.py` — flags IDs defined by more than one
  spec file, and phantom source IDs.

Both are read-only and are intended to run in CI.
