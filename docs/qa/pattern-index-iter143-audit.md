# Pattern Index Reconciliation Audit — Iter-143 Wave 2

**Audit Date**: 2026-05-19
**Scope**: Cross-check of `docs/qa/PATTERN_INDEX.md` against iter-143 (wave 1 + wave 2) landed changes.
**Purpose**: Identify required updates to PATTERN_INDEX.md so a future commit agent can apply them mechanically.
**Note**: This audit is read-only. PATTERN_INDEX.md itself is NOT modified.

---

## 1. Governance Hooks Section

No proposed updates. (Iter-143 hardening on hooks is governance-doc-only; no new hook landed.)

---

## 2. Recently Landed Roslyn Analyzers Table

### Existing rows requiring update

| DF Code | Current Status | Proposed Status | Reason |
|---|---|---|---|
| DF0096 | `Warning — formalized (task #269); interpolation + concat + marker recognition + 19 tests` (iter 144) | `Warning — Tier 1 formalized iter-143 wave 2 (task #269); interpolation + concat + marker recognition + 19 tests` (iter 143) | Iter column previously labeled 144; iter-143 wave 2 is the actual landing iteration. Add "Tier 1" qualifier to make Tier explicit. |

### New rows to ADD to the "Recently Landed Roslyn Analyzers" table

| DF Code | Name | Pattern | Tier | Iter | Status |
|---------|------|---------|------|------|--------|
| DF0116 | SyncOverAsyncAnalyzer | #116 (sync-over-async) | 1 | 143 | Warning — marker recognition gap fixed iter-143 wave 2; `// sync-over-async-unavoidable: <reason>` and `// sync-over-async-ok: <reason>` now suppress correctly |

### "Total Roslyn Analyzers" line

| Current Text | Proposed Text | Reason |
|---|---|---|
| `**Total Roslyn Analyzers**: 42 files in src/Analyzers/. Analyzer tests in src/Tests/Analyzers.Tests.csproj.` | `**Total Roslyn Analyzers**: 43 files in src/Analyzers/ (iter-143 added DF0116 marker recognition; DF0096 Tier 1 formalization). Analyzer tests in src/Tests/Analyzers.Tests.csproj.` | DF0096 + DF0116 land in iter-143 wave 2. |

---

## 3. Reconciliation Table — Existing Pattern Rows

| Pattern | Current Status | Proposed Status | Reason |
|---|---|---|---|
| 96 | `Partial (no CLAUDE.md entry yet) — LogError stack-trace loss; Roslyn formalized iter-144 (task #269); 19 firing tests` | `RETIRED (iter-143 wave 2) — 46 violations across repo → 0. DF0096 Roslyn Tier 1 formalized; Python detector (detect_logerror_no_stack.py) parity with analyzer. 19 firing tests. CLAUDE.md entry still pending (governance doctrine to be added next sweep).` | Iter-143 wave 2 cleared all 46 violations and formalized DF0096 as Tier 1. |
| 99 | `Full alignment — StringComparer.Ordinal doc exists only in CLAUDE.md; detection + allowlist in place` | `Full alignment (iter-143 wave 2 follow-ups #540 + #541) — PackDependencyResolver migrated `StringComparer.OrdinalIgnoreCase` → `Ordinal`; Registry migrated to `Ordinal`. Detection + allowlist in place.` | Iter-143 wave 2 closed two follow-up call sites that previously held OrdinalIgnoreCase. |
| 222 | `Partial — Custom pattern (method body > 60 lines) — no CI detection` | `Partial (iter-143 wave 2: #538 characterization tests + NativeMenuInjector decomp landed, 302 → 63 lines via 6 helpers). Custom pattern — no CI detection (DF1015 Roslyn analyzer covers compile-time enforcement).` | Iter-143 wave 2 task #538 decomposed NativeMenuInjector into 6 helpers + added characterization tests. |
| 232 | `Full alignment (closure) — Unbounded append-only file logging without rotation (HIGH=26 baseline)` | `RETIRED (iter-143 wave 2) — 3 HIGH → 0. WriteDebug methods got 100MB rotation guard + BepInEx fallback. Detection script (detect_unbounded_log_append.py) remains as drift gate.` | Iter-143 wave 2 fix: WriteDebug rotation + BepInEx fallback closes the unbounded-append vector. |

### Pattern #530 — NEW row to ADD to Reconciliation Table

```
| 530 | ✅ | ✅ pattern_530_audit.md (pending) | ❌ (MSBuild Warning Code DF0530) | Full alignment (iter-143 wave 2 NEW) | MSBuild deploy target silent no-op under multi-TFM project (HIGH=1 in Runtime.csproj baseline, fixed via WarnDeployWrongTFM target with DF0530 code) |
```

### Pattern #235 — NEW row to ADD to Reconciliation Table

```
| 235 | ✅ | ✅ pattern_235_audit.md (pending) | ❌ (grep-based detection in PR review) | Full alignment (iter-143 wave 1) | BepInEx plugin GraphicRaycaster without EventSystem guard (HIGH=1 in DFCanvas.cs baseline, fixed via EventSystem.current ensure block before raycaster add) |
```

---

## 4. Iter-142 Audit Closeouts Section

No structural changes proposed. Section is iter-142 specific.

**Recommendation**: Add new "Iter-143 Audit Closeouts" section after iter-142 closeouts:

```markdown
## Iter-143 Audit Closeouts

| Finding | Reference | Status | Notes |
|---------|-----------|--------|-------|
| Pattern #96 RETIRED | DF0096 Tier 1 + detect_logerror_no_stack.py | LANDED iter-143 wave 2 | 46 → 0 violations; 19 firing tests |
| Pattern #232 RETIRED | WriteDebug 100MB rotation + BepInEx fallback | LANDED iter-143 wave 2 | 3 HIGH → 0; rotation guard prevents 3.3GB log incident recurrence |
| Pattern #222 partial closure | #538 NativeMenuInjector decomp | LANDED iter-143 wave 2 | 302 → 63 lines via 6 helpers; characterization tests added |
| Pattern #99 follow-ups | #540 PackDependencyResolver + #541 Registry | LANDED iter-143 wave 2 | StringComparer.OrdinalIgnoreCase → Ordinal across two NuGet-published call sites |
| Pattern #530 NEW | DINOForge.Runtime.csproj WarnDeployWrongTFM | LANDED iter-143 wave 2 | New MSBuild silent-no-op pattern; DF0530 warning added |
| Pattern #235 NEW | src/Runtime/UI/DFCanvas.cs EventSystem guard | LANDED iter-143 wave 1 | BepInEx UI overlay no longer kills native mouse clicks |
| DF0096 Tier 1 formalization | src/Analyzers/LogErrorStackTraceAnalyzer.cs | LANDED iter-143 wave 2 | Interpolation + concat + marker recognition + 19 tests passing |
| DF0116 marker recognition fix | src/Analyzers/SyncOverAsyncAnalyzer.cs | LANDED iter-143 wave 2 | // sync-over-async-unavoidable / -ok markers now correctly suppress |

Cross-reference: `docs/sessions/iter143_session_retrospective.md` (pending), `project_iter143_session_retrospective.md` (memory).
```

---

## 5. Summary Statistics Section

| Current Text | Proposed Text | Reason |
|---|---|---|
| `**Total unique patterns across all 4 sources**: 39 (patterns 99-125 + 220-234)` | `**Total unique patterns across all 4 sources**: 41 (patterns 99-125 + 220-235 + 530)` | Adds #235 and #530 (iter-143). |
| `**CLAUDE.md entries**: 27 documented patterns in catalog section (+ 5 pre-pattern #94-98)` | `**CLAUDE.md entries**: 29 documented patterns in catalog section (+ 5 pre-pattern #94-98)` | CLAUDE.md gained #235 and #530 per iter-143 wave 1/2 sweep. |
| `**docs/qa audit files**: 15 files covering patterns {220-234} (iter-130+)` | `**docs/qa audit files**: 15+ files covering patterns {220-235, 530} (iter-130+); pattern_235_audit.md and pattern_530_audit.md pending` | Adds pending audit files. |
| `**Roslyn analyzers (src/Analyzers/)**: 42 compiled analyzer implementations (Tier 1-3)` | `**Roslyn analyzers (src/Analyzers/)**: 43 compiled analyzer implementations (Tier 1-3); DF0116 marker fix iter-143 wave 2` | Reflects DF0116 addition; DF0096 promoted to Tier 1. |
| `**Recently-landed Roslyn**: 7 (DF1010-DF1015 + pending DF1021)` | `**Recently-landed Roslyn**: 9 (DF1010-DF1015 + DF0096 + DF0116 + pending DF1021)` | Two analyzer landings in iter-143. |

### Alignment Counts subsection

| Current Text | Proposed Text | Reason |
|---|---|---|
| `**Full alignment (CLAUDE.md + docs/qa + Roslyn/CI)**: 3 patterns {226 (Roslyn DF1018), 234 (DeployPacks exclusion), pending}` | `**Full alignment (CLAUDE.md + docs/qa + Roslyn/CI)**: 5 patterns {96 (RETIRED, DF0096 Tier 1 + detect_logerror_no_stack.py), 226 (DF1018), 232 (RETIRED, detect_unbounded_log_append.py), 234 (DeployPacks exclusion), 530 (DF0530 MSBuild warning)}` | Patterns #96 + #232 retired with full triangle (CLAUDE.md + audit + detector/analyzer); #530 newly fully aligned. |
| `**Tier 2 Audit-Only (CLAUDE.md + docs/qa, no CI detection)**: 12 patterns {220-225, 227-233}` | `**Tier 2 Audit-Only (CLAUDE.md + docs/qa, no CI detection)**: 11 patterns {220-225, 227-231, 233, 235}` | #232 promoted to full alignment; #235 added as Tier 2. |

---

## 6. Roslyn Analyzer Tier Coverage Section

| Current Text | Proposed Text | Reason |
|---|---|---|
| `**Tier 1 (High-signal, compile-time catch)**: DF1010, DF1011, DF1012, DF1013, DF1014 — 5 active, 1 pending (DF1015).` | `**Tier 1 (High-signal, compile-time catch)**: DF0096, DF0116, DF1010, DF1011, DF1012, DF1013, DF1014, DF1015 — 8 active (DF0096 + DF0116 promoted iter-143 wave 2).` | DF0096 and DF0116 promoted to Tier 1 in iter-143 wave 2; DF1015 also active. |
| `**Total Analyzer Implementations**: 32 files in src/Analyzers/, with dedicated test project src/Tests/Analyzers.Tests.csproj.` | `**Total Analyzer Implementations**: 43 files in src/Analyzers/, with dedicated test project src/Tests/Analyzers.Tests.csproj.` | Number was stale at 32; current actual count is 43 per iter-143 wave 2. |

---

## 7. Footer

| Current Text | Proposed Text | Reason |
|---|---|---|
| `**Last Updated**: 2026-05-18 (iter-142)` | `**Last Updated**: 2026-05-19 (iter-143 wave 2)` | Audit date roll-forward. |
| `**Curated By**: Agent doc-sync sweep (Haiku, 200k token budget)` | `**Curated By**: Agent doc-sync sweep (Haiku, 200k token budget) — iter-143 audit by separate audit agent` | Reflects multi-wave authorship. |
| `**Governance Hooks Status**: 2 LIVE (block-git-stash, guard-git-worktree), 1 queued (block-no-verify). Settings.json wiring planned for v0.26.0.` | (unchanged) | No hook landed iter-143; status preserved. |

---

## Summary — Proposed Changes for Future Commit Agent

The following changes should be applied to `docs/qa/PATTERN_INDEX.md`:

### Edits to existing content

1. **DF0096 row** (Recently Landed Roslyn Analyzers table): Iter `144` → `143`; add "Tier 1" qualifier to Status.
2. **Total Roslyn Analyzers line**: `42 files` → `43 files`, add iter-143 note.
3. **Pattern #96 reconciliation row**: Status `Partial` → `RETIRED (iter-143 wave 2)`; update Notes (46 → 0 violations).
4. **Pattern #99 reconciliation row**: Append note about #540 + #541 follow-ups (PackDependencyResolver + Registry OrdinalIgnoreCase → Ordinal).
5. **Pattern #222 reconciliation row**: Append note about #538 characterization tests + NativeMenuInjector decomp (302 → 63 LOC).
6. **Pattern #232 reconciliation row**: Status `Full alignment (closure)` → `RETIRED (iter-143 wave 2)`; update Notes (3 HIGH → 0, rotation guard + BepInEx fallback).
7. **Summary Statistics block** (5 lines): patterns count 39 → 41; CLAUDE.md entries 27 → 29; audit files note pending #235/#530; analyzer count 42 → 43; recently-landed 7 → 9.
8. **Alignment Counts subsection**: Full alignment 3 → 5 patterns; Tier 2 Audit-Only 12 → 11 patterns (reflect #96, #232 promotion + #235 addition).
9. **Roslyn Analyzer Tier Coverage section**: Tier 1 active 5 → 8 (add DF0096, DF0116, DF1015); Total Analyzer Implementations 32 → 43.
10. **Footer Last Updated**: `2026-05-18 (iter-142)` → `2026-05-19 (iter-143 wave 2)`.

### New content to insert

11. **NEW row in Recently Landed Roslyn Analyzers table** for `DF0116 | SyncOverAsyncAnalyzer | #116 | 1 | 143 | Warning — marker recognition gap fixed iter-143 wave 2`.
12. **NEW row in Reconciliation Table** for Pattern #530 (MSBuild deploy target silent no-op under multi-TFM project).
13. **NEW row in Reconciliation Table** for Pattern #235 (BepInEx plugin GraphicRaycaster without EventSystem guard).
14. **NEW section** "Iter-143 Audit Closeouts" inserted after the existing "Iter-142 Audit Closeouts" section (8 rows: #96, #232, #222, #99, #530, #235, DF0096, DF0116 closures).

### Out-of-scope (do NOT add in this sweep)

- Creation of `docs/qa/pattern_235_audit.md` and `docs/qa/pattern_530_audit.md` (separate task; referenced as "pending" in this audit).
- Modifications to CLAUDE.md (separate doc, separate sweep).
- Modifications to detection scripts under `scripts/ci/` (separate task).

---

**Audit Output Verified**: This file is the only artifact produced. PATTERN_INDEX.md remains untouched, per instructions.

**Audit Authored By**: Haiku audit agent (iter-143 wave 2 retrospective sweep)
**Triggering Context**: Codex spark timeout at 600s on equivalent task; re-run via Haiku for reliability.
