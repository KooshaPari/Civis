chore(P2): agent-E done (22 IDs tagged, 22 IDs implemented, 13 IDs deferred)

Slice: docs/audits/P2-impl-slice/P2-agent-E.md (35 IDs).
Defer ledger: docs/audits/spec-only-deferred.md.

Tagged / implemented (in this round, on top of next-P2-E):

  FR-CIV-ASSET-QUAL-001      asset-pipeline/src/validate.rs (3 rules + 4 tests)
  FR-CIV-GODOT-ATTACH-001    godot-ref/rust/src/attach.rs::set_speed_rpc_payload
  FR-CIV-GODOT-ATTACH-002    godot-ref/rust/src/attach.rs::terrain_url_for
  FR-CIV-GODOT-ATTACH-003    godot-ref/rust/src/attach.rs::snapshot_throttle_ms_for
  FR-CIV-GODOT-ATTACH-004    godot-ref/rust/src/attach.rs::attach_mode_of
  FR-CIV-MIGRATION-001..005  emergence-migration/src/lib.rs (FR tags + helpers)
  FR-CIV-SOCIAL-002-IDEOLOGY social/src/ideology.rs (blend_toward + alignment_f32 + 2 tests)
  FR-SAVE-007                save-db::SaveDbError::HashMismatch
  FR-SAVE-014                save-db::SaveDbError::TooOldFormat
  FR-SAVE-015                save-db::SaveDbError::FutureFormat
  FR-SAVE-020                save-db::evict_autosaves (tag + new ring-cap test)
  FR-SAVE-021                save-db::SAVE_RPC_METHODS constant
  FR-SAVE-022                save-db::SAVE_EVENT_NAMES + 3 formatters
  FR-SAVE-023                save-db::failed_step on load-failed event
  FR-SAVE-024                save-db::pending_commands_marker
  FR-SAVE-025                save-db::paginate_by_tick_desc (helper + test)
  NFR-CIV-LEGENDS-LOUD-03    legends::decay::prune_below + worker::drain tracing::warn
  NFR-CIV-SEC-001            mod-host::validate_guest_imports + 3 tests

Verification:
  cargo check -p civ-asset-pipeline        -> ok
  cargo check -p civ-mod-host              -> ok
  cargo check -p civ-save-db               -> ok
  cargo test  -p civ-asset-pipeline        ->  4/4 pass
  cargo test  -p civ-mod-host (nfr_civ_sec_001*) ->  3/3 pass
  cargo test  -p civ-save-db               -> 21/21 pass
  cargo test  -p civ-social                -> 14/14 pass
  cargo test  -p civ-legends               -> 36/36 pass
  cargo test  -p civ-emergence-migration   ->  9/9  pass
  cargo test  -p civis-godot-rust          -> 12/12 pass (4 new attach tests)
  cargo check  --workspace --tests         -> ok (no new errors; pre-existing warnings only)

Deferred (see docs/audits/spec-only-deferred.md for full reasoning):
  FR-SAVE-006, FR-SAVE-008..013, FR-SAVE-016..019 (12 IDs)
  NFR-CIV-SEC-002, NFR-CIV-SEC-003, NFR-CIV-SEC-004 (3 IDs)

Counts: tagged 22 / wrote 22 / deferred 13 / deleted 0 / failed 0.
