# Coverage Rerun - 2026-06-08

## Goal

Re-measure targeted Bridge coverage after the tick12 bridge test bump and the BDD/Autograder Release-build unblock, using the same Bridge-filtered Cobertura path and documenting the delta versus the tick11 baseline.

## Baseline

Tick11 baseline from `docs/sessions/coverage-current-measure-20260608.md`:

- Line coverage: 4.7%
- Branch coverage: 0%
- Method coverage: 18.7%

## Measurement

Command run:

```powershell
dotnet-coverage collect dotnet test src/Tests/DINOForge.Tests.csproj --configuration Release --verbosity minimal --results-directory coverage-results-bridge-rerun /p:GameInstalled=false --filter 'FullyQualifiedName~Bridge' -f cobertura -o coverage-results-bridge-rerun/bridge-filtered.cobertura.xml
```

Bridge-only summary generated from the fresh Cobertura XML:

- Line coverage: 52.0% `580 / 1114`
- Branch coverage: 47.3% `236 / 498`
- Method coverage: 52.3% `100 / 191`

Test result:

- Passed: 188
- Skipped: 1
- Failed: 0

Artifacts:

- `coverage-results-bridge-rerun/bridge-filtered.cobertura.xml`
- `coverage-results-bridge-rerun/bridge-only/Summary.txt`
- `coverage-results-bridge-rerun/bridge-only/Summary.md`

## Delta vs Tick11

- Line coverage: `+47.3` points
- Branch coverage: `+47.3` points
- Method coverage: `+33.6` points

## Blockers

- No build blocker remained in this rerun; the Release test project built and executed successfully after the BDD/Autograder generated-file fix.
- One test remained skipped by design: `DINOForge.Tests.MockGameBridgeServerTests.MockGameBridgeServer_StrictMode_AcceptsValidReceipt`.
- The measurement is bridge-filtered only; it does not refresh the broader Domains slice.
