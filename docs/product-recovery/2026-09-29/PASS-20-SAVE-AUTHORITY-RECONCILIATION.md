# Civis pass 20 — filesystem save authority vs metadata index

Date 2026-09-30. Concurrent source `54d5758970249c8d1f24688ea45920b530e77299`.

## Source-observed authority split

`crates/save-db` describes itself as a **session-scoped SQLite metadata index for save files**. Records contain session/slot/tick/file_path/size/time; the DB does not contain the authoritative simulation payload.

`crates/watch/src/saves_api.rs::list_saves_handler`:
1. attempts to read SaveDb metadata into a name map;
2. if the DB read fails, logs a warning and continues with `None`;
3. independently scans the save directory;
4. emits actual filesystem saves and overlays DB metadata when a matching row exists.

This is strong source evidence that the filesystem save artifact is canonical enough to remain discoverable without the DB, while SaveDb is an index/projection.

## Inconsistency windows

Server autosave:
1. writes archive;
2. reads size;
3. inserts SaveDb row;
4. records replay-bus event;
5. asks SaveDb to evict rows;
6. removes returned files; removal failures are warnings only.

Therefore:
- archive can exist with no DB row if metadata insert fails;
- DB eviction can remove a row while backing file remains orphaned if unlink fails;
- filename tick and write tick already have a separate TOCTOU race.

The product needs **reconciliation**, not a distributed transaction that pretends filesystem + DB are one atomic store.

## Preferred authority model

- Immutable save-generation directory/archive + outer manifest = canonical durable artifact.
- CURRENT/slot pointer = canonical selected generation for a named slot.
- SaveDb = rebuildable/searchable metadata projection over canonical artifacts.
- On startup/listing, reconcile:
  - canonical artifact missing from DB -> index it;
  - DB row points to missing artifact -> mark/remove stale row;
  - orphan old generations -> retain according to rollback/GC policy;
  - corrupt/incomplete staged generation -> quarantine, never promote.

Autosave-ring policy should choose canonical artifacts first, then update the index, not delete the index row and merely hope file deletion succeeds.

## Prototype status

The test-only immutable-generation/CURRENT-pointer prototype now exercises:
- failed staged candidate does not replace current generation;
- successful commit switches pointer while preserving prior generation.

It deliberately does not claim the current `remove CURRENT; rename pending` implementation is crash-atomic on every supported filesystem. Production implementation must use a platform-qualified atomic replace/durable-write primitive or a small transactional metadata store.
