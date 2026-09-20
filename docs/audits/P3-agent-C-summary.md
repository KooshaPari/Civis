agent-C tagged 0 / wrote 14 / deferred 0 / deleted 10 / failed 0 IDs.

Per-ID commits:
- 1178da98 FR-CIV-AGGRESSION-001 spec
- 9bb7eab3 FR-CIV-BEVY-028 spec
- 18f18259 FR-CIV-BEVY-034 spec
- cb778f48 FR-CIV-BEVY-035 spec
- cf9ada30 FR-CIV-BEVY-036 spec
- 48b890d0 FR-CIV-CONTENT-001 spec
- 9371a336 FR-CIV-ECON-FOCUS-001 spec
- f9ba8501 FR-CIV-GENETICS-SEED-001 spec
- 3a85bccb FR-CIV-GENETICS-SEED-002 spec
- 61589381 FR-CIV-GENETICS-SEED-003 spec
- 0b07dd84 FR-CIV-NEEDS-DECAY-01 spec
- 1bd14012 FR-CIV-UNREST-002 spec
- 8088945b FR-MUSIC-001 spec
- f26da9ea FR-VIEWPORT-001 spec
- 6de3a915 chore: delete 10 FR-NFR-CIV-* stub tests + stub spec dirs

`cargo build --workspace --tests` exits 0 (only pre-existing warnings
in `civ-economy` about unused imports/constants; nothing from P3
changes).

Wrote (Option A, 14 specs):
14 intent.md docs at `docs/traceability/<id-lower>/<id-lower>-intent.md`
covering the source-code-tagged IDs. Each spec references the source
line(s), explains the contract, and lists tests. The source files
already carry `// FR-CIV-*` comments so the audit can now pick them up
as COVERED next sweep.

Deleted (Option B, 10 orphan stubs):
Per bias on generic FR-NFR-CIV-* prefixes without substance:
- FR-NFR-CIV-LEGENDS-LOUD-03
- FR-NFR-CIV-SCALE-001/002/003/004
- FR-NFR-CIV-SCALE-900/901/902/910/920
Removed 10 stub test files at `crates/engine/tests/fr_nfr_civ_*.rs`
(`WorldState::default(); assert!(ws.tick == 0)` placeholders) and
10 stub spec dirs at `docs/traceability/nfr-civ-*/`. Documented in
`docs/audits/code-only-deleted.md` to prevent future re-add.
