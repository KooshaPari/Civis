# Civis pass 2 — world/save identity and commit boundary

Date 2026-09-29. Original frozen source remains `b3cd62a7394878cc64d024fbfcfd398b8bb88bf1`. This pass also inspects concurrent main `54d5758970249c8d1f24688ea45920b530e77299`; findings are revision-bound and do not silently rebase the original assessment.

## C-F05 — one UI action can target server and local simulation

At concurrent main, `clients/bevy-ref/src/save_load_ui.rs` blob `ab33270ad1c15cc6800fe8cd200874c443d32929` was read through `apply_save_slot_action`.

For SaveAs:
1. if `server_bridge` exists, the UI sends RPC `save.slot`;
2. it then requires a local `SimState`;
3. if local state exists, it also calls local `save_to_slot`.

For Load:
1. if `server_bridge` exists, it sends RPC `save.slot` with action load;
2. if local `SimState` exists, it also calls local `load_from_slot`;
3. the loaded local simulation replaces `sim.0` and the local UI transitions to Playing.

The code does not make these paths mutually exclusive. Therefore the architecture cannot assume a Save/Load button has one authoritative target merely because a server bridge is connected.

This is a source-observed dual-dispatch condition. Whether normal plugin/resource composition can actually provide both resources simultaneously must still be traced. If mutually exclusive by construction elsewhere, that invariant needs explicit enforcement/test. If both can coexist, the UI action can mutate/persist two different simulations under one user gesture.

### Required decision

Define one authority mode per session/action:

- LOCAL: local SimState is authoritative; server bridge cannot also mutate a different world.
- REMOTE: server world is authoritative; local state is a projection/cache and local save/load is disabled or explicitly separate.
- MIRRORED/DUAL: only if intentionally supported, with shared world identity and a synchronization/commit contract. It must not arise accidentally from optional resources.

REC-CIVIS-WORLD-IDENTITY should carry this as a first-class invariant.

## C-F06 — autosave filename tick can diverge from written tick

At concurrent main, `crates/server/src/autosave.rs` blob `3bb2d1141a5ba158458ebd49744e16b1d2567e62` was read.

`run_autosave_once`:
1. acquires the simulation mutex and reads `tick_for_filename`;
2. releases the mutex;
3. constructs `autosave-{tick_for_filename}`;
4. later reacquires the mutex;
5. writes the archive;
6. reads `tick_at_write`;
7. records DB/event metadata using `tick_at_write`.

The source comment says the filename tick is "the same one" as the archive/metadata tick, but the two lock acquisitions permit the simulation to advance between them. Under contention, the filename may identify tick A while the actual archive, DB row and event identify tick B.

This is a concrete TOCTOU identity race in source semantics, not a native reproduction. The fix direction is architectural: capture the authoritative snapshot/tick once under the same consistency boundary used to serialize, then derive filename and metadata from that snapshot. Do not merely compare after the fact and rename without defining crash behavior.

Negative control: force a tick between the two acquisitions. The grader must reject a save whose path tick disagrees with metadata/manifest/world tick unless the naming contract explicitly stops encoding tick identity.

## C-F07 — save metadata index is post-file and can diverge

The inspected server autosave writes the archive before `SaveDb::record_autosave`. Watch save_handler likewise writes an archive, then records metadata. If metadata insertion fails, the archive can exist without the index/event path completing. Conversely eviction can update DB state and then fail to remove the file, leaving an orphan archive.

This may be an acceptable recoverable design if the filesystem is canonical and the DB is rebuildable metadata. That authority has not been established. The contract must say which is authoritative and how reconciliation works after crash/DB failure.

## C-F08 — concurrent main contains explicit compile-recovery stubs/TODOs

`crates/engine/src/engine.rs` blob `b5fabb65b3f8982c7feb5fdf913d3e8ed32efa13` contains explicit "Local stubs for removed upstream types", "forward-declared placeholders so the engine compiles", and TODOs for removed/renamed APIs. This is direct evidence of transition debt. It does not mean the entire engine is a stub; the imported validation report says the workspace build passed and thousands of tests executed at a nearby exact candidate. But any mature-contract mapping through these placeholder types must classify them as transitional until their semantics are reconciled.

Do not turn compile recovery into product completion credit.

## Save-integrity update

The concurrent v5+ manifest work materially improves corruption detection and mutation coverage. It still does not by itself establish:
- authoritative world identity across local/server modes;
- atomic replacement of the prior accepted slot;
- metadata-selected legacy/current classification;
- DB/filesystem reconciliation;
- exhaustive authoritative-state ownership;
- authenticity against a writer who can recompute unkeyed hashes.

Those are separate criteria, not reasons to discount the new integrity work.

## Next experiments / trace work

1. Trace Bevy plugin composition to prove whether local SimState + ServerBridge can coexist. If yes, reproduce dual save/load and compare world IDs/ticks.
2. Add a forced scheduler/tick barrier between autosave lock acquisitions and observe filename/archive/DB/event identities.
3. Fault after archive write but before DB insert; restart and specify/reconcile orphan behavior.
4. Fault after DB eviction before file deletion; restart and reconcile.
5. Trace save_archive destination replacement mechanics and filesystem guarantees; test prior-slot survival on write interruption.
6. Enumerate the 22 mirrored state fields plus sidecars/ECS/queues/guest state and classify canonical owner vs projection.
7. Audit compile-recovery stubs against accepted emergence semantics before using their tests as requirement evidence.

No native execution was performed by this recovery worker. These findings remain blocking design questions, not completed fixes.
