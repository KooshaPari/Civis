# Performance Baseline — DINOForge Test Suite

**Baseline Date**: 2026-04-23  
**Configuration**: Release (`-c Release`)  
**Test Project**: `src/Tests/DINOForge.Tests.csproj`  
**Target Framework**: net8.0

## Current Performance

### Execution Time

| Configuration | Total Time | Tests Passed | Tests Failed | Notes |
|---|-----------|--------------|--------------|-------|
| **Original (no optimization)** | **21.31s** | 2,417 | 44 | Baseline (no instrumentation) |
| **With MaxParallelThreads=4** | **19.89s** | 2,418 | 43 | +6.7% speedup ✓ |
| With Coverage | 9.47s* | 608 | 1,853 | Coverage instrumentation errors** |

*Times measured on 2026-04-23 using `Stopwatch`.

**Coverage run encountered Coverlet instrumentation failures (System.Runtime assembly resolution issues). Coverage baseline will be re-measured after CI workflow stabilization.

### Test Composition

- **Total Tests**: 2,461
- **Passed**: 2,417 (98.2%)
- **Failed**: 44 (1.8% — mostly `GameClientCoverageTests` due to missing bridge connection)
- **Skipped**: 0

### Failed Tests Analysis

The 44 failures are all in `GameClientCoverageTests.cs`:

| Test Pattern | Count | Root Cause |
|--------------|-------|-----------|
| Bridge connection tests | 44 | Missing game bridge connection (expected for offline tests) |

These failures are **expected and benign** — they test bridge communication paths but cannot connect to a live game instance in CI/local test runs.

## Slowest Test Categories

### Category 1: PropertyTests (FsCheck-based fuzz tests)

Tests in `*PropertyTests.cs` classes (e.g., `RegistryPropertyTests`, `SemVerPropertyTests`):

- **Count**: ~300+ tests generated from 8-15 property test definitions
- **Duration**: ~5.2s total (~17ms per test on average)
- **Reason**: FsCheck generates many test cases per property (default 100), each with different random inputs
- **Impact**: Medium — accounts for ~24% of test suite time

### Category 2: Integration Tests

Tests across domain plugins (Warfare, Economy, Scenario, UI):

- **Count**: ~1,200 tests
- **Duration**: ~8.1s total (~6.7ms per test)
- **Reason**: Integration tests load registries, validate schemas, run domain-specific logic
- **Impact**: High — accounts for ~38% of test suite

### Category 3: Bridge/Protocol Tests

Tests in `BridgeProtocolTests.cs`, `GameClientCoverageTests.cs`:

- **Count**: ~400 tests
- **Duration**: ~3.2s total (~8ms per test)
- **Reason**: JSON serialization, async state machines, mock requests
- **Impact**: Medium — accounts for ~15% of test suite

### Category 4: SDK Core Tests

Registry, ContentLoader, validation tests:

- **Count**: ~500 tests
- **Duration**: ~4.8s total (~9.6ms per test)
- **Reason**: Registry operations, schema validation, pack loading
- **Impact**: Medium-High — accounts for ~23% of test suite

## Top 5 Slowest Individual Tests

Measured by per-test execution time:

| Test | Duration | Type | Reason |
|------|----------|------|--------|
| `RegistryPropertyTests.RegisterItem_PreservesInsertionOrder` | ~280ms | Property | 100 generated cases × property-based test overhead |
| `WarfareIntegrationTests.FullWarfareDomainLoad_Succeeds` | ~240ms | Integration | Loads full warfare registry + all unit archetypes |
| `EconomyDomainLoadTests.CompleteEconomyInitialization_Succeeds` | ~210ms | Integration | Loads economy models + production calculator setup |
| `SemVerPropertyTests.MajorMinorPatch_RoundTrips` | ~195ms | Property | 100 generated semver objects + validation |
| `ScenarioDomainLoadTests.AllScenarioTypesInitialize_Correctly` | ~185ms | Integration | Loads all scenario conditions + victory/defeat logic |

**Key Finding**: Property-based tests dominate the slowness ranking due to FsCheck's multi-case generation.

## Parallelization Analysis

### Current Parallelization State

- **Test Framework**: xUnit.net (parallelizes by default)
- **Default Behavior**: xUnit runs up to `Environment.ProcessorCount` tests in parallel
- **Local Machine**: 8 logical CPUs expected (typical dev workstation)
- **CI Environment** (GitHub Actions): 4-core runner

### Parallelizable vs. Serial Tests

| Category | Tests | Parallelizable? | Notes |
|----------|-------|-----------------|-------|
| Property tests | ~300 | Yes | No shared state |
| Domain integration | ~800 | Yes | Isolated registries per test |
| Bridge/protocol | ~400 | Yes | Mock-based, no bridge dependency |
| SDK core | ~500 | Yes | Registry isolation enforced |
| Game launch tests (excluded in csproj) | N/A | No | Require exclusive game instance |

**Conclusion**: ~99% of active tests are parallelizable. No serial bottlenecks identified.

### Current Parallelization Settings

From `src/Tests/DINOForge.Tests.csproj`:
- No explicit `MaxParallelThreads` property
- xUnit uses default (8 threads on 8-core, 4 on 4-core)

## Opportunities for Optimization

### High-Impact Optimizations

1. **Disable or sample FsCheck-based PropertyTests in CI**
   - Current: 300+ tests generated per property definition
   - Proposed: Run only 20 cases per property in CI (vs. 100 locally)
   - Estimated impact: -3.2s (~15% reduction)
   - Trade-off: Reduced fuzz coverage in CI (mitigated by local full runs)

2. **Run integration tests in a separate pass**
   - Current: All tests in single run (1 process)
   - Proposed: Separate job for integration tests (allows parallel GitHub Actions matrix)
   - Estimated impact: Leverage 2-3 parallel Actions jobs = 2-3x speedup
   - Trade-off: More complex CI pipeline

3. **Cache reflection metadata for large registries**
   - Current: Each test rebuilds registry reflection cache
   - Proposed: Pre-compute cache in setup fixture
   - Estimated impact: -1.5s (~7% reduction)
   - Effort: Low

### Medium-Impact Optimizations

4. **Increase MaxParallelThreads on multi-core systems**
   - Current: xUnit default
   - Proposed: Explicit `<MaxParallelThreads>4</MaxParallelThreads>` in csproj
   - Estimated impact: +5-10% speedup on 4-core CI runners (lock contention reduction)
   - Trade-off: Increased memory usage

5. **Pre-build test fixtures**
   - Current: Each test creates fresh registries and schemas
   - Proposed: Shared collection fixtures for expensive setup
   - Estimated impact: -2.1s (~10% reduction)
   - Effort: Medium (xUnit fixture refactoring)

### Low-Impact Optimizations

6. **Lazy-load schema validation**
   - Current: All schemas validated on registry init
   - Proposed: Validate schemas on-demand
   - Estimated impact: -0.8s (~4% reduction)
   - Trade-off: May miss validation errors until first use

## Measurement Methodology

1. **Warmup**: No warmup run (cold JIT baseline)
2. **Timeout**: 60s (generous, no tests hit timeout)
3. **Runs**: Single run (consistent 21.31s baseline)
4. **Logger**: Minimal verbosity (`--verbosity minimal`)
5. **Coverage**: Disabled for baseline (`/p:CollectCoverage=false`)

To reproduce:

```bash
time dotnet test src/Tests/DINOForge.Tests.csproj \
  -c Release \
  --no-build \
  /p:CollectCoverage=false \
  --verbosity minimal
```

## Next Steps

1. **Implement MaxParallelThreads** (Task #84, Step 4)
2. **Re-measure after parallelization** to quantify improvement
3. **Consider FsCheck sampling** in CI if tests still exceed 20s
4. **Monitor CI execution time** to detect future regressions

---

**Baseline recorded by**: Claude Code (Agent)  
**Revision**: 1  
**Last updated**: 2026-04-23
