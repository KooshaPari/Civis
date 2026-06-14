# Bridge Autograder C# Project - 2026-06-08

## Scope

Added a C# autograder test project for Bridge acceptance-contract scoring:

- `src/Tests/Autograder/DINOForge.Tests.Autograder.csproj`
- `src/Tests/Autograder/BridgeAcceptanceEvidence.cs`
- `src/Tests/Autograder/BridgeAcceptanceScorerTests.cs`

This does not duplicate the tick14 Tier-1 Python autograder work. The existing tick14 path remains the focused MVP wire for `tier=mvp`, `pass=true`; this project adds a C# Bridge evidence scorer and threshold tests for the three acceptance tiers.

## Contract Coverage

The C# scorer asserts these tier requirements:

- Tier 1: canonical spec, focused test, and traceability row.
- Tier 2: Tier-1 prerequisite plus mutation, chaos, perf, and load threshold evidence.
- Tier 3: Tier-2 prerequisite plus Bridge coverage >= 75%, mutation >= 70%, 2+ chaos, 2+ perf, 2+ load, and 1+ BDD regression-suite evidence.

The Tier-2 mutation floor is encoded from `docs/specs/SPEC-TIER2-MATURE.md` as 85%. The Tier-3 thresholds match `docs/specs/SPEC-TIER3-ELITE.md`.

## Wiring

- Added `Tests\Autograder\DINOForge.Tests.Autograder.csproj` to `src/DINOForge.sln`.
- Updated `src/Tests/DINOForge.Tests.csproj` to exclude `Autograder\**`, preventing the parent test project from compiling the nested Autograder sources and generated outputs.

## Validation

Targeted build:

```powershell
dotnet build src/Tests/Autograder/DINOForge.Tests.Autograder.csproj -c Release -m:1 --disable-build-servers
```

Result:

- Build succeeded.
- 0 warnings.
- 0 errors.

Targeted test sanity check:

```powershell
dotnet test src/Tests/Autograder/DINOForge.Tests.Autograder.csproj -c Release --no-build --verbosity minimal
```

Result:

- Passed: 5
- Failed: 0
- Skipped: 0

## Notes

- The scorer is intentionally deterministic and evidence-driven; it does not run coverage, mutation, chaos, perf, load, or BDD jobs itself.
- Current Bridge coverage evidence found in existing session notes is below the Tier-3 75% floor, so the C# tests assert the contract threshold behavior rather than claiming the current repo already passes Tier 3.
