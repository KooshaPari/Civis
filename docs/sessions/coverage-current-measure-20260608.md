# Coverage Current Measure - 2026-06-08

## Goal

Measure the current targeted Bridge/Domains coverage without running the full suite, using the repo's existing XPlat/Coverlet-style test flow.

## What I Ran

- Failed coverage attempt:
  - `dotnet test src/Tests/DINOForge.Tests.csproj --configuration Release --verbosity minimal --logger "trx;LogFileName=tick10-unit.trx" --collect:"XPlat Code Coverage" --results-directory coverage-results-tick10 /p:GameInstalled=false --filter "FullyQualifiedName~EconomyBranchCoverageTests|FullyQualifiedName~EconomyCoverageTests|FullyQualifiedName~EconomyAdvancedCoverageTests|FullyQualifiedName~EconomyUiCoverageTests|FullyQualifiedName~UiDomainCoverageTests|FullyQualifiedName~UIBranchCoverageTests|FullyQualifiedName~ScenarioDomainCoverageTests|FullyQualifiedName~TradeEngineCoverageTests|FullyQualifiedName~SDKCoverageTests|FullyQualifiedName~SdkServicesCoverageTests|FullyQualifiedName~GameClientCoverageTests|FullyQualifiedName~PackRegistryClientCoverageTests"`
- Nearest successful targeted unit command:
  - `dotnet test src/Tests/DINOForge.Tests.csproj --configuration Debug --no-build --verbosity minimal --logger "trx;LogFileName=tick10-unit-nobuild.trx" --collect:"XPlat Code Coverage" --results-directory coverage-results-tick10 /p:GameInstalled=false --filter "FullyQualifiedName~EconomyBranchCoverageTests|FullyQualifiedName~EconomyCoverageTests|FullyQualifiedName~EconomyAdvancedCoverageTests|FullyQualifiedName~EconomyUiCoverageTests|FullyQualifiedName~UiDomainCoverageTests|FullyQualifiedName~UIBranchCoverageTests|FullyQualifiedName~ScenarioDomainCoverageTests|FullyQualifiedName~TradeEngineCoverageTests|FullyQualifiedName~SDKCoverageTests|FullyQualifiedName~SdkServicesCoverageTests|FullyQualifiedName~GameClientCoverageTests|FullyQualifiedName~PackRegistryClientCoverageTests"`
- Nearest successful targeted Bridge command:
  - `dotnet test src/Tests/Integration/DINOForge.Tests.Integration.csproj --configuration Debug --no-build --verbosity minimal --logger "trx;LogFileName=tick10-bridge.trx" --collect:"XPlat Code Coverage" --results-directory coverage-results-tick10 /p:GameInstalled=false --filter "FullyQualifiedName~BridgeCoverageTests"`
- Coverage summary command:
  - `reportgenerator "-reports:coverage-results-tick10/8a2e8c5e-56a4-45a1-aaf7-1cf9e5fda8ff/coverage.cobertura.xml" "-targetdir:coverage-results-tick10/bridge-only" "-reporttypes:MarkdownSummary;TextSummary" "-assemblyfilters:+DINOForge.Bridge.*"`

## Blocker

The Release unit coverage command failed in the test project build, unrelated to the coverage filter itself:

- `src\Tests\BDD\obj\Release\net8.0\xUnit.AssemblyHooks.DINOForge_Tests_BDD.cs`
  - duplicate `DINOForge_Tests_BDD_XUnitAssemblyFixture`
  - duplicate `DINOForge_Tests_BDD_ReqnrollNonParallelizableFeaturesCollectionDefinition`
  - missing `Reqnroll` namespace/type reference
  - duplicate xUnit assembly attributes
- `src\Tests\Autograder\obj\Release\net11.0\.NETCoreApp,Version=v11.0.AssemblyAttributes.cs`
  - duplicate `TargetFrameworkAttribute`

The Debug `--no-build` fallback succeeded, but it did not regenerate a fresh unit coverage XML for the Bridge/Domains slice.

## Current Measured Coverage

Fresh coverage data produced during this session is available for the Bridge slice:

- `DINOForge.Bridge.Client`: 0% line, 0% branch
- `DINOForge.Bridge.Protocol`: 17.6% line, 0% branch
- Combined Bridge slice: 4.7% line coverage, 0% branch coverage, 18.7% method coverage
  - `49 / 1034` coverable lines
  - `0 / 366` branches
  - `49 / 262` methods

This session did not produce a fresh unit-project Cobertura artifact for the Domains slice, so the current Domains coverage is not refreshed here.

## Artifacts

- [Bridge summary text](C:/Users/koosh/Dino/coverage-results-tick10/bridge-only/Summary.txt)
- [Bridge summary markdown](C:/Users/koosh/Dino/coverage-results-tick10/bridge-only/Summary.md)
- [Bridge Cobertura XML](C:/Users/koosh/Dino/coverage-results-tick10/8a2e8c5e-56a4-45a1-aaf7-1cf9e5fda8ff/coverage.cobertura.xml)
- [Bridge TRX](C:/Users/koosh/Dino/coverage-results-tick10/tick10-bridge.trx)
- [Unit TRX](C:/Users/koosh/Dino/coverage-results-tick10/tick10-unit-nobuild.trx)
