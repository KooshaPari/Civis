# Quality-Grind Findings (2026-06-14, post-consolidation steady-state)

## Coverage breadth: STRONG (not the gap)
- ~3969 test methods, all 14 surveyed feature areas covered, 0 untested core classes (orchestrator-direct survey).
- The L6 gaps are DEPTH on regression-prone paths, not missing coverage.

## DF0114 (CancellationToken not threaded) — LARGELY FALSE-POSITIVE
- Inspected EconomyContentLoader.cs: ALL 6 flagged lines (71,72,73,119,179,239) ALREADY thread `cancellationToken` into the inner await (e.g. `SafeFileIO.ReadTextAsync(file, cancellationToken).ConfigureAwait(false)`).
- The DF0114 (Pattern #114) detector cannot trace ct through `.ConfigureAwait(false)` chains → over-flags compliant code.
- **Implication: the ~1298 DF0114 "warnings" are mostly NOT real debt.** Do NOT bulk-churn them. The real fix is improving the DF0114 detector precision (Roslyn analyzer), a separate task — OR suppress with documented markers where confirmed-compliant.

## Real DEPTH backlog (curated, deliberate — not per-tick-forced)
1. ThemeApplier SRP extraction from MainMenuThemer.cs (1200+ lines, source of 2 session regressions) — real maintainability win, but high-risk; needs careful staged refactor + tests.
2. LoadingScreenController field-rename guard — net8.0 reflection test impossible (UI/* excluded); needs a source-level/Roslyn check or a netstandard2.0-targeted test.
3. DF0111 silent-catch (1513) — needs case-by-case judgment (log vs //safe-swallow vs remove), NOT bulk; many may also be analyzer-conservative.
4. CS8602 null-deref in Tests/GameLaunch (120) — add null guards; low-risk but test-only code.

## Lane reality (memory)
- forge→Fireworks: reliable SINGLE-SHOT, flaky MULTI-TURN (streaming body-decode). Orchestrator-direct is the workhorse for substantive work.

## Steady-state stance
Consolidation L0-L6 COMPLETE, PR #286 mergeable→main. Remaining quality is a deliberate curated set, not a warning-count chase. Land real improvements when a lane is healthy; don't manufacture churn or chase analyzer false-positives.
