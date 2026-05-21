# Iter-143 Wave 1 Closure — Pattern #234 & v0.25.0 Prep

**Date**: 2026-05-18  
**Branch**: `fix/handle-connect-iter142` → `main` (51 commits)  
**Context**: Continuation of iter-142 game-fix landing; Pattern #234 (Test Pack Leak) closeout.

## Pattern #234 Closeout (COMPLETED)

| Item | Status |
|------|--------|
| Structural fix | `src/Runtime/DINOForge.Runtime.csproj:292` DeployPacks Exclude |
| Audit doc | `docs/qa/test_pack_leak_audit_iter142.md` |
| CI detector | `scripts/ci/detect_test_pack_leak.py` (102 LOC) |
| Allowlist | `docs/qa/pattern-234-test-pack-leak-allowlist.txt` |
| CLAUDE.md entry | 4-section pattern format |
| PATTERN_INDEX.md | +1 entry (39 total) |
| CI gate | `.github/workflows/pattern-gates.yml:41` (HIGH>0) |
| BepInEx cleanup | 0 stale test-* dirs |

## In Flight

- #523 EconomyContentLoader investigation
- MockSteamworksNet TFM Pattern #233 retry
- v0.25.0 tag-readiness audit
- #524 PreToolUse hook smoke-test

## Other Landings

- `lefthook` fix: line 19 `{staged_files}` pre-applied
- `benchmarks.yml`: path mismatch fixed (`src/Tests/Benchmarks`)
- iter-142 closure docs refreshed
- Pattern #234 cross-link complete

## v0.25.0 Critical Path

1. #523 verification
2. #524 verification
3. Merge `fix/handle-connect-iter142` → `main`
4. Tag v0.25.0
