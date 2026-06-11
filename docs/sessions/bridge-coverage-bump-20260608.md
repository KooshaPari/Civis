# Bridge Coverage Bump - 2026-06-08

## Goal

Add the smallest safe Bridge unit-test bump that improves bridge-assembly coverage without duplicating the tick10 integration bridge tests.

## Files Changed

- `src/Tests/Bridge/BridgeReceiptVerifierUnitTests.cs`
  - Added direct coverage for `BridgeReceiptVerifier.ConstantTimeHexEquals(...)`.
  - Exercises the helper's null, length-mismatch, odd-length, invalid-hex, and case-insensitive success paths.
- `src/Tests/DINOForge.Tests.csproj`
  - Excluded `src/Tests/BridgeSmoke/**` from the parent xUnit project so the Bridge unit suite no longer picks up the separate MSTest smoke project during targeted runs.

## Rationale

- The tick10 bridge report measured only the integration slice and left `DINOForge.Bridge.Client` at 0% line coverage.
- The bridge unit suite already covered most public bridge APIs, but the constant-time hex comparison helper in `BridgeReceiptVerifier` had no direct unit coverage.
- The `BridgeSmoke` folder is a separate MSTest project; leaving it in the parent xUnit glob caused unrelated compile failures during focused Bridge test runs.

## Validation

Command run:

```powershell
dotnet-coverage collect dotnet test src/Tests/DINOForge.Tests.csproj --configuration Debug --verbosity minimal --results-directory coverage-results-bridge-native /p:GameInstalled=false --filter 'FullyQualifiedName~Bridge' -f cobertura -o coverage-results-bridge-native/bridge-unit.cobertura.xml
```

Fresh coverage summary for `DINOForge.Bridge.*`:

- Line coverage: 52.0% (748 / 1436)
- Branch coverage: 46.8% (239 / 510)
- Method coverage: 52.3% (100 / 191)

Test result:

- Passed: 188
- Skipped: 1
- Failed: 0

Cobertura artifact:

- `coverage-results-bridge-native/bridge-unit.cobertura.xml`

## Measured Delta

Baseline from `docs/sessions/coverage-current-measure-20260608.md`:

- Line coverage: 4.7%
- Branch coverage: 0%

Fresh bridge unit run:

- Line coverage: 52.0%
- Branch coverage: 46.8%

Delta:

- Line coverage: +47.3 points
- Branch coverage: +46.8 points

## Notes

- This is a bridge-unit-suite measurement, not the tick10 integration-only bridge slice.
- The bridge client still has remaining low-coverage surfaces, especially `GameProcessManager`, but the overall bridge assembly is now well past the 20% target.
