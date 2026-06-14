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

## Dirty-set gate status
- Restored 8 regenerable artifacts (coverage/.coverage/coverage.json/test-results.xml/packages.lock.json) — not committed (churn).
- 99 real dirty remain (23 src .cs + new test suites + tooling + docs + .agileplus). Gating via CI.NoRuntime.sln now.
