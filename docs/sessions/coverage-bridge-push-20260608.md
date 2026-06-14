# Bridge Coverage Push - 2026-06-08

## Goal

Add 3-5 small, focused Bridge tests that hit high-ROI public methods without duplicating the tick12/tick13 coverage work, then re-run Bridge-filtered Cobertura and record the delta.

## Files Changed

- `src/Tests/Bridge/CanonicalJsonEdgeCaseTests.cs`
  - Added a new focused Bridge test class covering:
    - canonical hashing for nested objects and arrays via `BridgeReceiptBuilder.ComputePayloadHash(...)`
    - scalar string canonicalization
    - finite float and decimal scalar canonicalization
    - the string `"Infinity"` rejection path
    - nested unsafe numeric handling with a BigInteger + `float.NaN` payload
- `src/Tests/DINOForge.Tests.csproj`
  - Excluded the nested `Perf` test project from the parent aggregate test project so focused Bridge runs do not fail on the benchmark subtree.
- `docs/sessions/coverage-bridge-push-20260608.md`
  - This session note.

## Validation

Targeted test command:

```powershell
dotnet test src/Tests/DINOForge.Tests.csproj --configuration Release --verbosity minimal --filter FullyQualifiedName~CanonicalJsonEdgeCaseTests /p:GameInstalled=false
```

Result:

- Passed: 6
- Failed: 0
- Skipped: 0

Bridge-filtered coverage command:

```powershell
dotnet-coverage collect dotnet test src/Tests/DINOForge.Tests.csproj --configuration Release --verbosity minimal --results-directory coverage-results-bridge-push /p:GameInstalled=false --filter 'FullyQualifiedName~Bridge' -f cobertura -o coverage-results-bridge-push/bridge-filtered.cobertura.xml
```

Bridge-only summary generated from the Cobertura artifact:

- Line coverage: 54.3% `606 / 1114`
- Branch coverage: 52.2% `260 / 498`
- Method coverage: 53.9% `103 / 191`

Test result:

- Passed: 195
- Skipped: 1
- Failed: 0

Artifacts:

- `coverage-results-bridge-push/bridge-filtered.cobertura.xml`
- `coverage-results-bridge-push/bridge-only-bridge/Summary.txt`

## Delta vs Tick13

Tick13 Bridge line coverage baseline:

- Line coverage: 52.0%

This push:

- Line coverage: 54.3%

Delta:

- Line coverage: `+2.3` points

## Notes

- The new tests intentionally avoid the tick12/tick13 receipt-verifier cases and focus on previously untested canonicalization edges.
- The aggregate test project needed the nested `Perf` project excluded before the focused Bridge run would build cleanly.
