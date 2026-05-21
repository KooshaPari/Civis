# Pattern Catalog Reconciliation Index

**Status as of iter-144, 2026-05-20** (previously iter-142, 2026-05-18)

This document reconciles all Pattern Catalog entries across four sources:
- **CLAUDE.md** Pattern Catalog (governance doctrine)
- **docs/qa/** audit files (detailed findings & allowlists)
- **scripts/ci/** detection scripts (automated CI gates)
- **src/Analyzers/** Roslyn analyzers (compile-time enforcement, Tier 1-3)
- **PreToolUse Hooks** (structural enforcement via scripts/hooks/)

---

## Governance Hooks (Iter-141/142 Hardening)

| Hook | Rule | Trigger Incident | Status |
|---|---|---|---|
| `scripts/hooks/block-git-stash.ps1` (76 LOC) | feedback_stash_auto_route_to_branch.md | Iter-141: 3 concurrent stashes nearly lost | LIVE (tested, awaiting .claude/settings.json wiring) |
| `scripts/hooks/guard-git-worktree.ps1` (100 LOC) | feedback_worktree_boundary.md | Iter-142: `git worktree remove --force` bypassed safety | LIVE (tested, awaiting .claude/settings.json wiring) |
| `scripts/hooks/block-no-verify.ps1` (pending) | feedback_no_verify_forbidden.md | Iter-142: Preventive (no incident yet) | QUEUED for v0.26.0 |

**Configuration Status**: Hook files exist and are tested. `.claude/settings.json` **PreToolUse** wiring not yet implemented (v0.26.0 roadmap). See `governance_hardening_iter142.md` for full context.

---

## Recently Landed Roslyn Analyzers

| DF Code | Name | Pattern | Tier | Iter | Status |
|---------|------|---------|------|------|--------|
| DF1010 | AsyncLambdaActionAnalyzer | N/A | 1 | 121 | Warning |
| DF1011 | AsyncBlockingCallAnalyzer | N/A | 1 | 122 | Warning |
| DF0096 | LogErrorStackTraceAnalyzer | #96 (LogError) | 1 | 144 | Warning — formalized (task #269); interpolation + concat + marker recognition + 19 tests |
| DF1012 | ThrowExceptionStackLossAnalyzer | #96 (LogError) | 1 | 123 | Warning, marker-fix iter-126 |
| DF1013 | UnsealedConcreteMutableClassAnalyzer | #220 | 1 | 125 | Info |
| DF1014 | HardcodedThresholdAnalyzer | #221 | 1 | 126 | Info |
| DF1015 | LongMethodAnalyzer | #222 | 1 | 127 | Info |
| DF1018 | PublicFieldAnalyzer | #226 | 1 | 131 | Info (Pattern #226 audit: HIGH=0) |
| DF1019 | (Pending) | N/A | 1 | 132 | Info |
| DF1020 | (Pending) | N/A | 1 | 133 | Info |
| DF1021 | (Pending) | N/A | 1 | 134 | Info |

**Total Roslyn Analyzers**: 44 files in `src/Analyzers/` (Tier 1 = 16 + Tier 2 = 28 per #585 audit). Analyzer tests in `src/Tests/Analyzers.Tests.csproj`.

---

## Reconciliation Table

| Pattern # | CLAUDE.md | docs/qa Audit | scripts/ci Detection | Status | Notes |
|-----------|:---------:|:-----------:|:------------------:|--------|-------|
| 96 | ❌ (pre-catalog) | ❌ | ✅ detect_logerror_no_stack.py + Roslyn DF0096 (Tier 1) | Partial (no CLAUDE.md entry yet) | LogError stack-trace loss; Roslyn formalized iter-144 (task #269); 19 firing tests |
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
| 221 | ✅ | ✅ pattern_221_audit.md | ❌ | Partial | Custom pattern (hardcoded numeric thresholds) — no CI detection |
| 222 | ✅ | ✅ pattern_222_audit.md | ❌ | Partial | Custom pattern (method body > 60 lines) — no CI detection |
| 223 | ✅ | ✅ pattern_223_audit.md | ❌ | Partial | Custom pattern (iter-139) — no CI detection |
| 224 | ✅ | ✅ pattern_224_audit.md | ❌ | Partial | Custom pattern (iter-140) — no CI detection |
| 225 | ✅ | ✅ pattern_225_audit.md | ❌ | Partial | Custom pattern (iter-141) — no CI detection |
| 226 | ✅ | ✅ pattern_226_audit.md + pattern_226_event_exemptions.md | ✅ DF1018 (Roslyn) | Full alignment (iter-131) | Public field mutability in NuGet API (HIGH=0 as of iter-142) |
| 227 | ✅ | ✅ pattern_227_audit.md | ❌ | Partial | Custom pattern (iter-141) — no CI detection |
| 228 | ✅ | ✅ pattern_228_audit.md | ❌ | Partial | Custom pattern (iter-142) — no CI detection |
| 229 | ✅ | ✅ pattern_229_audit.md | ❌ | Partial | Custom pattern (iter-142) — no CI detection |
| 230 | ✅ | ✅ pattern_230_audit.md | ❌ | Partial | Custom pattern (iter-142) — no CI detection |
| 231 | ✅ | ✅ pattern_231_audit.md | ✅ detect_static_init_side_effect.py | Full alignment | Static constructor / field initializer with I/O side effect (HIGH=11 in NuGet surface) |
| 232 | ✅ | ✅ pattern_232_audit.md | ✅ detect_unbounded_log_append.py | Full alignment (closure) | Unbounded append-only file logging without rotation (HIGH=26 baseline) |
| 233 | ✅ | ✅ pattern_233_audit.md | ✅ detect_bepinex_plugin_tfm.py | Full alignment (closure) | TFM/SDK migration stale obj/ cache + BepInEx plugin multi-target (HIGH=0 baseline) |
| 234 | ✅ | ✅ test_pack_leak_audit_iter142.md | ✅ detect_test_pack_leak.py | Full alignment (iter-142) | Test fixture IDs leaking into deployed packs (MSBuild DeployPacks exclusion landed) |
| 235 | ✅ | ❌ | ❌ | LIVE (iter-143) | BepInEx plugin GraphicRaycaster without EventSystem guard; EventSystem ensure block landed in src/Runtime/UI/DFCanvas.cs (iter-143) — pairs with Pattern #231 static-init discipline |
| 530 | ✅ | ❌ | ❌ | LIVE (iter-143) | MSBuild deploy target silent no-op under multi-TFM project; WarnDeployWrongTFM guardrail target (Warning DF0530) landed in src/Runtime/DINOForge.Runtime.csproj (iter-143) — pairs with Pattern #233 stale obj/ cache |

---

## Iter-144 Progress Notes (2026-05-20)

| Pattern | Activity | File:Line / Ref | Status |
|---------|----------|-----------------|--------|
| #96 (LogError stack-trace loss) | ModPlatform.cs:253 reformatted to Pattern #96-compliant logging (full exception object passed, not `.Message`) — commit `30b29705` | `src/Runtime/ModPlatform.cs:253` | Code site cleaned; Roslyn DF0096 already enforces compile-time (iter-144 task #269) |
| #108 (Sleep-based test sync) | PackFileWatcher debounce relaxed — commit `9bc88f9c` | PackFileWatcher (SDK) | Debounce interval relaxed for test stability; not a regression — detection threshold unchanged |
| #231 (Static-init I/O side effect) | Referenced via task #505 follow-up; no new code touched in iter-144 (work landed earlier) | `src/Analyzers/...` | Stable; HIGH=11 in NuGet surface (baseline from iter-142) |
| #99, #117, #123 | No iter-144 code touches; baseline holds | — | Unchanged |

**Iter-144 recommended detector runs (next session)**:
- `scripts/ci/detect_logerror_no_stack.py` — verify ModPlatform.cs:253 fix did not introduce regressions elsewhere
- `scripts/ci/detect_test_sleep_sync.py` — confirm PackFileWatcher debounce change did not push test sleep counts over threshold
- `scripts/ci/detect_static_init_side_effect.py` — re-baseline HIGH count (still 11 expected)

---

## Iter-142 Audit Closeouts

| Finding | Reference | Status | Notes |
|---------|-----------|--------|-------|
| HiddenDesktopBackend wiring | isolation_layer.py (814 LOC) | DEFERRED v0.26.0 | NOT WIRED — dead code in current isolation_layer.py |
| Lefthook scope | scripts/hooks/lefthook | OPEN (still open per iter-144 audit) | Hardcoded sln path, fix = `{staged_files}` glob requires user authorization; no iter-143/144 closure recorded |
| TIER 1 deploy spec | src/Runtime/DINOForge.Runtime.csproj | RESOLVED (iter-143) | WarnDeployWrongTFM guardrail (Pattern #530) landed in iter-143 — DF0530 warning fires on non-netstandard2.0 TFM leaves when DeployToGame=true |
| IL2026 root cause | Newtonsoft.Json v13 transitive | OPEN (still open per iter-144 audit) | SDK serialization; 3 resolution options documented; no iter-143/144 closure recorded |
| CHANGELOG iter-142 entry | CHANGELOG.md | LANDED | 1-line addendum on isolation layer status |

Cross-reference: `docs/sessions/iter-142-DECISIONS-SYNTHESIS.md`, `docs/qa/*_iter142.md` (5 audit reports).

---

## Summary Statistics (Iter-144)

- **Total unique patterns across all 4 sources**: 41 (patterns 99-125 + 220-235 + 530)
- **CLAUDE.md entries**: 29 documented patterns in catalog section (+ 5 pre-pattern #94-98)
- **docs/qa audit files**: 15 files covering patterns {220-234} (iter-130+)
- **scripts/ci detection scripts**: 36 scripts across legacy patterns (99-125)
- **Roslyn analyzers (src/Analyzers/)**: 44 compiled analyzer implementations across Tier 1 (16) + Tier 2 (28) per #585 audit
- **Recently-landed Roslyn (Tier 1)**: 9 (DF1010-DF1015 + DF1018 + DF0096 + pending DF1019-DF1021)
- **Tier 3 (FsCheck property-based)**: 159 properties per ac455d audit (behavioral variance seeding for pack/schema/Registry)
- **Governance Hooks**: 2 LIVE + 1 queued for v0.26.0

### Alignment Counts (Post-Roslyn Era, iter-144)
- **Full alignment (CLAUDE.md + docs/qa + Roslyn/CI)**: 3 patterns {226 (Roslyn DF1018), 234 (DeployPacks exclusion), pending}
- **Tier 2 Audit-Only (CLAUDE.md + docs/qa, no CI detection)**: 12 patterns {220-225, 227-233} (custom patterns from iter-139+)
- **Legacy 2-of-3 (CLAUDE.md + detection script)**: 17 patterns {99-125 minus overlaps}
- **Partial/Pre-catalog (no CLAUDE.md yet)**: 5 patterns {94-98}
- **Iter-143 LIVE (CLAUDE.md only, code-site fix)**: 2 patterns {235 (EventSystem guard), 530 (WarnDeployWrongTFM guardrail)}

---

## Orphan Analysis

### Orphaned Detection Scripts (in scripts/ci/ but no CLAUDE.md entry)
- ❌ `detect_unguarded_deserialize.py` — **Pattern #95 analog** (not in Pattern Catalog section, pre-pattern methodology era)
- ❌ `detect_global_state_tests.py` — no CLAUDE.md equivalent
- ❌ `detect_hardcoded_pipe_names.py` — related to Pattern #118-119 (not documented)
- ✅ `detect_logerror_no_stack.py` — **Pattern #96** (Python detector); paired with Roslyn DF0096 (formalized iter-144, task #269)
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

**Tier 1 (High-signal, compile-time catch)**: 16 analyzers including DF0096, DF1010-DF1015, DF1018 active; DF1019-DF1021 pending.

**Tier 2 (Semantic patterns, tool use)**: 28 analyzers (DF1001-DF1009 foundational + extensions) — covers SDK surface.

**Tier 3 (FsCheck property-based)**: 159 properties per ac455d audit — behavioral variance seeding for pack/schema/Registry landed.

**Total Analyzer Implementations**: 44 files in `src/Analyzers/` (Tier 1 + Tier 2 per #585 audit), with dedicated test project `src/Tests/Analyzers.Tests.csproj`.

---

**Last Updated**: 2026-05-20 (iter-144) — previously iter-142 (2026-05-18)  
**Curated By**: Agent doc-sync sweep (iter-144 doc gardener)  
**Governance Hooks Status**: 2 LIVE (block-git-stash, guard-git-worktree), 1 queued (block-no-verify). Settings.json wiring planned for v0.26.0.
