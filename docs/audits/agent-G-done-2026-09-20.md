agent-G done (62 IDs)

Summary:
- tagged: 0 (no source-code tagging was required — every orphan
  source already carries an explicit `/// FR-XYZ` doc comment
  matching the slice `code_refs`)
- wrote: 56 substantial intent specs (FR-CIV-ARCH-C-001..004,
  FR-CIV-CLIMATE-1..4, FR-CIV-DIPLO-003-01..07 + 003-006,
  FR-CIV-FAMINE-001, FR-CIV-INSTITUTIONS-001, FR-CIV-REL-007,
  FR-EMG-001..025 except 011)
- wrote + deleted stub: 6 (FR-NFR-R-01..06 generic reliability
  placeholders — Option B per bias rule for FR-NFR-R prefixes)
- wrote + deleted stub: 13 (FR-NFR-CIV-DET-001/002 and
  FR-NFR-CIV-PERF-002..008, 900..902 — generic placeholders; stub
  tests were degenerate `assert!(ws.tick == 0)` asserts with no
  real coverage)
- deferred: 0
- deleted: 18 orphan stub tests total
- failed: 0

Verification:
- `cargo check -p civ-engine --tests` → exit 0
- `cargo check -p civ-build -p civ-planet -p civ-diplomacy -p emergence-oracle`
  → exit 0 (16m 42s, only pre-existing warnings, none introduced
  by these changes)
- `docs/audits/code-only-deleted.md` records every deleted stub
  file by ID so future audit runs do not rediscover the same
  orphans.

Per-ID commits: 6 (one per epic group, plus the deletion commit).
Branch: next-P3-G. Not pushed.
