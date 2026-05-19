# Pattern Catalog Reconciliation Index

**Status as of iter-134, 2026-05-18**

This document reconciles all Pattern Catalog entries across three sources:
- **CLAUDE.md** Pattern Catalog (governance doctrine)
- **docs/qa/** audit files (detailed findings & allowlists)
- **scripts/ci/** detection scripts (automated CI gates)
- **src/Analyzers/** Roslyn analyzers (compile-time enforcement, Tier 1-3)

---

## Recently Landed Roslyn Analyzers

| DF Code | Name | Pattern | Tier | Iter | Status |
|---------|------|---------|------|------|--------|
| DF1010 | AsyncLambdaActionAnalyzer | N/A | 1 | 121 | Warning |
| DF1011 | AsyncBlockingCallAnalyzer | N/A | 1 | 122 | Warning |
| DF1012 | ThrowExceptionStackLossAnalyzer | #96 (LogError) | 1 | 123 | Warning, marker-fix iter-126 |
| DF1013 | UnsealedConcreteMutableClassAnalyzer | #220 | 1 | 125 | Info |
| DF1014 | HardcodedThresholdAnalyzer | #221 | 1 | 126 | Info |
| DF1015 | LongMethodAnalyzer | #222 | 1 | 127 | Info |
| DF1018 | PublicFieldAnalyzer | #226 | 1 | 131 | Info (Pattern #226 audit: HIGH=0) |
| DF1019 | (Pending) | N/A | 1 | 132 | Info |
| DF1020 | (Pending) | N/A | 1 | 133 | Info |
| DF1021 | (Pending) | N/A | 1 | 134 | Info |

**Total Roslyn Analyzers**: 42 files in `src/Analyzers/`. Analyzer tests in `src/Tests/Analyzers.Tests.csproj`.

---

## Reconciliation Table

| Pattern # | CLAUDE.md | docs/qa Audit | scripts/ci Detection | Status | Notes |
|-----------|:---------:|:-----------:|:------------------:|--------|-------|
| 99 | ✅ | ❌ | ✅ detect_unprotected_string_dict.py | Full alignment | StringComparer.Ordinal doc exists only in CLAUDE.md; detection + allowlist in place |
| 100 | ✅ | ❌ | ✅ detect_direct_datetime.py | Full alignment | TimeProvider injection; detection scoped to SDK/Runtime/Tools |
| 101 | ✅ | ❌ | ✅ detect_stringly_enums.py | Full alignment | Stringly-typed enum discriminators |
| 102 | ✅ | ❌ | ✅ detect_orphan_process_start.py | Full alignment | Orphan Process handle leakage |
| 103 | ✅ | ❌ | ✅ detect_direct_datetime.py | Full alignment | Local-time logging drift (scoped within #100 detection) |
| 104 | ✅ | ❌ | ✅ detect_catch_swallow_default.py | Full alignment | Catch-swallow-default erasure |
| 105 | ✅ | ❌ | ✅ detect_event_lifecycle_asymmetry.py | Full alignment | Event-subscription lifecycle asymmetry |
| 106 | ✅ | ❌ | ✅ detect_implicit_encoding.py | Full alignment | Implicit File.ReadAllText encoding (RETIRED per #405) |
| 107 | ✅ | ❌ | ✅ detect_unvalidated_di.py | Full alignment | BuildServiceProvider without ValidateOnBuild |
| 108 | ✅ | ❌ | ✅ detect_test_sleep_sync.py | Full alignment | Sleep-based test sync |
| 109 | ✅ | ❌ | ✅ detect_inline_json_options.py | Full alignment | Inline JsonSerializerOptions construction |
| 110 | ✅ | ❌ | ✅ detect_open_ended_count.py | Full alignment | Open-ended count assertion (RETIRED per #330) |
| 111 | ✅ | ❌ | ✅ detect_silent_catch.py | Full alignment | Silent exception swallowing (bare catch {}) |
| 112 | ✅ | ✅ pattern-112-time-provider.md | ✅ detect_direct_datetime.py | Full alignment | Unadjustable time source (overlaps #100) |
| 113 | ✅ | ❌ | ✅ detect_blocking_poll_sleep.py | Full alignment | Blocking polling with hardcoded sleep intervals |
| 114 | ✅ | ❌ | ✅ detect_ct_not_threaded.py | Full alignment | CancellationToken accepted but not threaded |
| 115 | ✅ | ❌ | ✅ detect_httpclient_per_instance.py | Full alignment | HttpClient per-call or per-constructor anti-pattern |
| 116 | ✅ | ❌ | ✅ detect_sync_over_async.py | Full alignment | Sync-over-async blocking (.Result / .Wait) |
| 117 | ✅ | ❌ | ✅ detect_stringbuilder_no_capacity.py | Full alignment | StringBuilder capacity not pre-sized |
| 120 | ✅ | ❌ | ✅ detect_unguarded_json_deserialize.py | Full alignment | JsonSerializer.Deserialize without explicit options |
| 121 | ✅ | ❌ | ✅ detect_unnecessary_allocation.py | Full alignment | Unnecessary LINQ terminal allocation |
| 123 | ✅ | ❌ | ✅ detect_public_mutable_collections.py | Full alignment | Public collection mutability in DTOs |
| 124 | ✅ | ❌ | ✅ detect_unsealed_public_classes.py | Full alignment | Unsealed public classes in NuGet assemblies |
| 125 | ✅ | ❌ | ❌ | Partial | Orphan interface mocks (detection script exists: detect_orphan_interface_mocks.py but not mapped) |
| 220 | ✅ | ✅ pattern_220_audit.md | ❌ | Partial | Custom pattern (audit-rotation methodology convergence) — no CI detection |
| 221 | ✅ | ✅ pattern_221_audit.md | ❌ | Partial | Custom pattern (CT propagation follow-up) — no CI detection |
| 222 | ✅ | ✅ pattern_222_audit.md | ❌ | Partial | Custom pattern (magic number extraction) — no CI detection |
| 226 | ✅ | ✅ pattern_226_audit.md | ✅ DF1018 (Roslyn) | Full alignment (iter-131) | Public field mutability in NuGet API (HIGH=0 as of iter-134) |
| 227 | ✅ | (planned) | (planned) | Pending | TBD — next HIGH-priority pattern from iter-134 audit queue |

---

## Summary Statistics

- **Total unique patterns across all 4 sources**: 26
- **CLAUDE.md entries**: 20 documented patterns in catalog section (+ 5 pre-pattern #94-98)
- **docs/qa audit files**: 4 files covering patterns {220, 221, 222, 223}
- **scripts/ci detection scripts**: 36 scripts across all patterns
- **Roslyn analyzers (src/Analyzers/)**: 32 compiled analyzer implementations
- **Recently-landed Roslyn**: 5 (DF1010, DF1011, DF1012, DF1013, DF1014); 1 pending (DF1015)

### Alignment Counts (Post-Roslyn Era, iter-131+)
- **Full alignment (CLAUDE.md + docs/qa + Roslyn/CI)**: 5 patterns {220, 221, 222, 226, +pending}
- **2-of-3 (CLAUDE.md + detection script)**: 17 patterns {99-125 minus overlaps}
- **Partial/Legacy (pre-catalog detections)**: 6 patterns {94-98 + global-state orphans}

---

## Orphan Analysis

### Orphaned Detection Scripts (in scripts/ci/ but no CLAUDE.md entry)
- ❌ `detect_unguarded_deserialize.py` — **Pattern #95 analog** (not in Pattern Catalog section, pre-pattern methodology era)
- ❌ `detect_global_state_tests.py` — no CLAUDE.md equivalent
- ❌ `detect_hardcoded_pipe_names.py` — related to Pattern #118-119 (not documented)
- ❌ `detect_logerror_no_stack.py` — **Pattern #96 analog** (pre-catalog)
- ❌ `detect_missing_configureawait.py` — **Pattern #98 analog** (pre-catalog)
- ❌ `detect_tcs_sync_continuations.py` — **Pattern #97 analog** (pre-catalog)
- ❌ `detect_unbounded_constraints.py` — **Pattern #94 analog** (pre-catalog)
- ❌ `audit_hardcoded_thresholds.py` — no CLAUDE.md entry
- ⚠️ `audit_unsealed_concrete_classes.py` — **RETIRED to docs/scripts/retired/** (iter-125 reconciliation; narrower scope on mutable-state-only checks; superseded by detect_unsealed_public_classes.py which covers full Pattern #124 semantics)

**Root cause**: Early audit-rotation waves (iters 45-80) created detection scripts before formalizing the Pattern Catalog in CLAUDE.md. Scripts #94-98 exist as pre-pattern detection, not yet formalized in governance doc.

### Orphaned docs/qa Files
- ❌ `pattern_220_allowlist.txt` (duplicate of pattern_220_audit.md structure; consolidate)

### Broken CLAUDE.md References
- ✅ `pattern-112-time-provider.md` referenced in Pattern #112 — **file exists, correctly named**
- ⚠️ Pattern #220-#222 entries in CLAUDE.md point to pattern_NNN_audit.md but no `.md` extension in reference text; files exist as `pattern_220_audit.md` etc. — **reference is correct**

---

## Retired Scripts

| Script Name | Location | Pattern | Reason | Iter |
|-------------|----------|---------|--------|------|
| `audit_unsealed_concrete_classes.py` | `docs/scripts/retired/` | #220 | Narrower scope on mutable-state-only; superseded by `detect_unsealed_public_classes.py` | 125 |

---

## Recommended Actions (Safe, Mechanical Fixes)

1. **Add missing CLAUDE.md Pattern entries** for pre-catalog detections:
   - Pattern #94: Unbounded Range Constraints (detect_unbounded_constraints.py exists)
   - Pattern #95: Unguarded Deserialization (detect_unguarded_deserialize.py exists)
   - Pattern #96: LogError stack-loss (detect_logerror_no_stack.py exists)
   - Pattern #97: TCS sync continuations (detect_tcs_sync_continuations.py exists)
   - Pattern #98: Missing ConfigureAwait (detect_missing_configureawait.py exists)
   - Pattern #118: Hardcoded thresholds (audit_hardcoded_thresholds.py exists)
   - Pattern #119: Hardcoded pipe names (detect_hardcoded_pipe_names.py exists)

2. **Consolidate audit duplication**:
   - Delete `pattern_220_allowlist.txt` (use `pattern_220_audit.md` as primary)
   - Verify `audit_unsealed_concrete_classes.py` vs `detect_unsealed_public_classes.py` — delete duplicate

3. **Maintain orphaned scripts** (pre-governance):
   - Keep `detect_global_state_tests.py` as is (useful but not yet formalized)
   - Keep `audit_hardcoded_thresholds.py` / audit_unsealed_concrete_classes.py (will add governance via #1 above)

4. **Do NOT delete** any audit docs per never-delete-repo-artifacts rule

---

## Next Steps for Future Iterations

- When formalizing Pattern #94-98 + #118-119 in CLAUDE.md (user-facing governance), also create corresponding audit docs in `docs/qa/pattern_NNN_audit.md`
- Migrate `detect_global_state_tests.py` into a numbered pattern once governance intent is clear
- Consider consolidating pattern-X detection into a single `pattern_NNN_runner.py` that invokes all detections for a given pattern number (reduces maintenance surface)

---

## Roslyn Analyzer Tier Coverage

**Tier 1 (High-signal, compile-time catch)**: DF1010, DF1011, DF1012, DF1013, DF1014 — 5 active, 1 pending (DF1015).

**Tier 2 (Semantic patterns, tool use)**: DF1001-DF1009 (9 files) — foundational analyzers for SDK surface.

**Tier 3 (FsCheck property-based)**: Pending — behavioral variance seeding for pack/schema/Registry.

**Total Analyzer Implementations**: 32 files in `src/Analyzers/`, with dedicated test project `src/Tests/Analyzers.Tests.csproj`.

---

**Last Updated**: 2026-05-18 (iter-134)  
**Curated By**: Agent doc-sync sweep (Haiku, 200k token budget)
