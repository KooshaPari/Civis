# L1 Build-Gate Findings (2026-06-13)

## The full-sln build is NOT the right gate
`dotnet build src/DINOForge.sln -c Release` (GameInstalled=false) → **13 errors**, but NONE from the 99 dirty files:
- **12× MSB3030** "Could not copy Unity.*.dll ... not found" in `DINOForge.Tests.Integration.csproj` — Unity game DLLs only exist when the game is installed. Pure **GameInstalled=false environmental gap**, not code.
- **1× CS1997** in `src/Tests/UiAutomation/CompanionFixture.cs:45` — `return Task.CompletedTask;` inside an `async Task InitializeAsync()` (must be bare `return;`). **PRE-EXISTING bug** (file is NOT in my dirty set, committed already). `UiAutomation` project is **NOT in CI.NoRuntime.sln** and NOT in the push gate → doesn't block CI/consolidation.

## Correct gate = CI.NoRuntime.sln
`src/DINOForge.CI.NoRuntime.sln` is the CI-safe build target: excludes the Unity-ref Integration project and the broken UiAutomation project. This is what isolates whether the 99 dirty files compile. (The push hook uses `dotnet test src/Tests/DINOForge.Tests.csproj` — also excludes Integration/UiAutomation — which is why the earlier HEAD push passed green.)

## Side-task surfaced for the DAG (L6 / quality)
- **FIX CS1997 in CompanionFixture.cs:45** — real pre-existing bug; `return Task.CompletedTask;` → `return;` (method is `async Task`). Small, safe, leaves UiAutomation buildable. Add to backlog.
- Consider adding UiAutomation to a CI lane so such breaks are caught (currently invisible to CI).

## CRITICAL: net8.0 regression gate MUST be --no-incremental
The cursors merge added a call to `TryApplyEnvironmentThemeForScene` (Plugin.cs:510) that is defined NOWHERE → CS0103 on net8.0. My post-merge `dotnet build Runtime net8.0` gate PASSED (false green) because it was INCREMENTAL and didn't recompile Plugin.cs. The lefthook test-integration step does a clean build → caught it. **Always run the net8.0 regression gate with `--no-incremental`** or it gives stale passes. Three merge regressions total caught by the net8.0/integration gate: (1) loadingscreen TrackColor→ProgressTrackColor, (2) warfare-modern pack resurrection conflict, (3) cursors orphan TryApplyEnvironmentThemeForScene. CI.NoRuntime.sln (netstandard2.0/net11) misses ALL net8.0-specific ones.

## lefthook commit-msg hook bug on Windows (merge commits) — ROOT CAUSE + FIX
The `conventional` commit-msg hook crashed (`sh -c: line N: syntax error: unexpected end of file` / `syntax error near unexpected token '(' `) on the L2 merge commit, blocking it. ROOT CAUSE: **lefthook on Windows runs hooks via `cmd.exe -> sh -c`, and cmd.exe PRE-PARSES `(`, `)`, `|` before sh sees them** — so ANY inline hook with a grouped regex or `case ... |` pattern gets mangled. The original `$1` inline hook had this latent but only tripped on the multi-line merge message path. FIX: moved the validator to **`scripts/check-commit-msg.sh`** and the hook is just `run: sh scripts/check-commit-msg.sh {1}` — no `(`/`)`/`|` in the inline command → cmd.exe can't corrupt it. Committed `020d6b73`. (forge spent ~10min circling this — it diagnosed the cmd.exe layer correctly but kept re-writing a `case ... |` that re-triggers the bug; the script-file approach is the durable fix.)

## Dirty-set gate status
- Restored 8 regenerable artifacts (coverage/.coverage/coverage.json/test-results.xml/packages.lock.json) — not committed (churn).
- 99 real dirty remain (23 src .cs + new test suites + tooling + docs + .agileplus). Gating via CI.NoRuntime.sln now.
