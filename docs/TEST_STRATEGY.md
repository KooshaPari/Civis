# DINOForge Test Strategy

**Version**: 1.0
**Last Updated**: 2026-03-30
**Status**: Active

## Test Philosophy

DINOForge employs a comprehensive testing strategy centered on **behavior-driven development (BDD)** with schema-driven validation (SDD) and test-driven development (TDD) practices. The goal is to ensure reliability, maintainability, and confidence in automated game mod orchestration.

### Core Principles
1. **Fast feedback** - Tests complete in <1 second per layer
2. **Isolated testing** - No external dependencies for unit tests
3. **Clear intent** - Test names describe behavior, not implementation
4. **Comprehensive coverage** - Critical paths 100%, edge cases 85%+
5. **Regression prevention** - All bug fixes have test cases
6. **Continuous validation** - CI gates prevent regressions

## Test Pyramid

```
                    ▲
                   ╱│╲     E2E (9%)
                  ╱ │ ╲    Real game scenarios
                 ╱  │  ╲
                ╱───┼───╲  Integration (22%)
               ╱    │    ╲ Multi-component flows
              ╱     │     ╲
             ╱──────┼──────╲ Unit (63%)
            ╱       │       ╲ Fast, isolated
           ╱────────┼────────╲ Property (6%)
          ╱         │         ╲ Randomized inputs
         └──────────┼──────────┘
                    │
           1,749 Total Tests
           ~13 seconds total
```

### Test Distribution Target
- **Unit Tests**: 63% (fast, isolated, specific)
- **Integration**: 22% (component interactions)
- **E2E**: 9% (user workflows, game scenarios)
- **Property-Based**: 6% (invariant checking)

## Test Layers

### Layer 1: Unit Tests (1,100 tests)

**Purpose**: Validate individual components in isolation

**Scope**:
- Individual classes and methods
- No external I/O (filesystem, network, database)
- Focused on business logic
- Edge cases and boundary conditions

**Examples**:
```csharp
[Fact]
public void Registry_Add_WithDuplicateKey_ThrowsConflictException()
{
    var registry = new Registry<UnitDefinition>();
    registry.Add("unit-1", new UnitDefinition { Id = "unit-1" });

    var action = () => registry.Add("unit-1", new UnitDefinition { Id = "unit-1" });

    action.Should().Throw<RegistryConflictException>();
}

[Theory]
[InlineData(0)]
[InlineData(-1)]
[InlineData(int.MinValue)]
public void StatModifier_WithNegativeValue_ClipsToZero(int value)
{
    var modifier = new StatModifier("health", value);

    modifier.Value.Should().Be(0);
}
```

**Tools**:
- xUnit.net (test framework)
- FluentAssertions (readable assertions)
- Moq (mocking)

**Coverage Target**: 90%+ per subsystem

---

### Layer 2: Integration Tests (380 tests)

**Purpose**: Validate interactions between multiple components

**Scope**:
- Component integration
- Registry population from YAML
- Pack loading and dependency resolution
- ECS Bridge entity queries
- State transitions

**Examples**:
```csharp
[Fact]
public void ContentLoader_LoadPack_RegistersAllContent()
{
    var registryMgr = new RegistryManager();
    var loader = new ContentLoader(registryMgr);

    loader.LoadPack("packs/example-balance");

    registryMgr.Units.Count.Should().BeGreaterThan(0);
    registryMgr.Buildings.Count.Should().BeGreaterThan(0);
}

[Fact]
public void PackDependencyResolver_WithCircularDependency_DetectsAndThrows()
{
    var packA = new PackManifest { Id = "pack-a", DependsOn = new[] { "pack-b" } };
    var packB = new PackManifest { Id = "pack-b", DependsOn = new[] { "pack-a" } };
    var resolver = new PackDependencyResolver();

    var action = () => resolver.Resolve(new[] { packA, packB });

    action.Should().Throw<CircularDependencyException>();
}
```

**Test Fixtures**:
- `GameFixture` - Real ECS world + asset loading
- `PackFixture` - Pre-built test packs
- `RegistryFixture` - Populated registries

**Coverage Target**: 85%+ per subsystem

---

### Layer 3: Property-Based Tests (100 tests)

**Purpose**: Validate invariants and properties under randomized inputs

**Scope**:
- Registry operations with random IDs
- SemVer version string parsing
- Pack loading with random content
- Stat modifiers with random values
- Conflict detection with random pack graphs

**Examples**:
```csharp
[Property]
public void Registry_AddThenRemove_ReturnsEmpty(string[] keys)
{
    var registry = new Registry<UnitDefinition>();
    var definitions = keys.Select(k => new UnitDefinition { Id = k }).ToList();

    foreach (var def in definitions)
        registry.Add(def.Id, def);

    foreach (var def in definitions)
        registry.Remove(def.Id);

    registry.Count.Should().Be(0);
}

[Property]
public void SemVer_Parse_Then_ToString_Preserves(string version)
{
    Assume.That(IsValidSemVer(version));

    var parsed = SemVer.Parse(version);

    parsed.ToString().Should().Be(version);
}
```

**Tools**:
- FsCheck.Xunit (property-based testing)
- Arbitrary generators for domain objects

**Coverage Target**: 80%+ per subsystem

---

### Layer 4: End-to-End Tests (150 tests)

**Purpose**: Validate complete user workflows and game scenarios

**Scope**:
- Game launch + pack loading + visual verification
- User menu interactions
- Mod installation flow
- Save game + load game + verify mods
- Multi-pack interaction scenarios

**Examples**:
```csharp
[Fact]
public async Task GameLaunch_WithModPack_ModsAreActive()
{
    // Launch game with mod pack
    var process = await MCP.LaunchGameAsync(new[] { "warfare-starwars" });

    // Wait for world ready
    await MCP.WaitForWorldAsync(timeout: TimeSpan.FromSeconds(10));

    // Verify mods loaded
    var status = await MCP.GetStatusAsync();
    status.LoadedPacks.Should().Contain("warfare-starwars");

    // Screenshot + VLM validation
    var screenshot = await MCP.ScreenshotAsync();
    var analysis = await VLM.AnalyzeAsync(screenshot);
    analysis.DetectedUI.Should().Contain("Clone Trooper");
}

[Fact]
public async Task UserWorkflow_InstallModsThenPlayGame_Succeeds()
{
    // 1. Run installer
    var result = await Installer.InstallAsync(
        gamePath: "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option",
        mods: new[] { "warfare-starwars", "economy-balanced" }
    );
    result.IsSuccessful.Should().BeTrue();

    // 2. Launch game
    var gameProcess = await MCP.LaunchGameAsync();

    // 3. Verify mods active
    var status = await MCP.GetStatusAsync();
    status.LoadedPacks.Should().HaveCount(2);

    // 4. Take screenshot
    var screenshot = await MCP.ScreenshotAsync();
    screenshot.Should().NotBeNull();
}
```

**Tools**:
- MCP Bridge (game automation)
- OmniParser (UI element detection)
- VLM (visual validation)
- Playwright (companion UI)

**Coverage Target**: 80%+ per workflow

---

## Test Naming Conventions

### Format
```csharp
[Fact/Theory]
public void <ClassName>_<MethodName>_<Scenario>_<ExpectedBehavior>()
{
    // Arrange
    // Act
    // Assert
}
```

### Examples
```csharp
public void Registry_Add_WithDuplicateKey_ThrowsConflictException()
public void ContentLoader_LoadPack_WithMissingDependency_ThrowsError()
public void GameClient_PingAsync_WhenDisconnected_ReturnsTimeout()
public void PackDependencyResolver_Resolve_WithCircularDependency_DetectsConflict()
```

## Test Organization

### File Structure
```
src/Tests/
├── GapClosure_*.cs              # Gap-closure focused tests
├── *Tests.cs                    # Component-specific tests
├── PropertyTests/
│   ├── RegistryPropertyTests.cs
│   ├── SemVerPropertyTests.cs
│   └── YamlFuzzTests.cs
├── Integration/
│   ├── Fixtures/
│   ├── Tests/
│   └── Assertions/
├── GameLaunch/
│   ├── GameLaunchSmokeTests.cs
│   ├── GameLaunchPackTests.cs
│   └── GameLaunchFixture.cs
├── UiAutomation/
│   ├── CompanionLaunchTests.cs
│   ├── CompanionNavigationTests.cs
│   └── CompanionFixture.cs
└── Benchmarks/
    ├── ContentLoaderBenchmarks.cs
    └── RegistryBenchmarks.cs
```

### Grouping Strategy
- **By domain**: SDK tests, Domains tests, Tools tests
- **By responsibility**: ContentLoader, Registry, Validation
- **By test type**: Unit, Integration, E2E, Property
- **By concern**: Coverage gaps, Regressions, Performance

## Assertion Best Practices

### FluentAssertions Patterns
```csharp
// Equality
result.Should().Be(expected);
result.Should().NotBeNull();
result.Should().BeEmpty();

// Collections
items.Should().HaveCount(5);
items.Should().Contain(x => x.Name == "Unit");
items.Should().BeInAscendingOrder();

// Exceptions
action.Should().Throw<ArgumentNullException>();
action.Should().NotThrow();

// Strings
name.Should().StartWith("unit-");
name.Should().Match("*Clone*");

// Complex assertions
result.Should()
    .HaveCount(3)
    .And.AllSatisfy(x => x.IsValid.Should().BeTrue())
    .And.NotContainNulls();
```

### Multi-Assertion Tests
```csharp
[Fact]
public void UnitDefinition_CreateCloneTrooper_HasCorrectStats()
{
    var unit = UnitDefinition.CreateCloneTrooper();

    unit.Should()
        .NotBeNull()
        .And.Subject.Health.Should().Be(100)
        .And.Subject.Attack.Should().Be(45)
        .And.Subject.Defense.Should().Be(30);

    unit.Visuals.ModelPath.Should().Contain("clone-trooper");
}
```

## Mocking Strategy

### When to Mock
- External dependencies (filesystem, network, database)
- Time-dependent operations
- Random or non-deterministic code
- Slow operations (>100ms)

### When NOT to Mock
- Pure domain logic
- In-memory collections
- Value objects
- Small, fast operations

### Mock Examples
```csharp
// Mock IGameClient for testing UI
var mockClient = new Mock<IGameClient>();
mockClient
    .Setup(x => x.GetStatusAsync(It.IsAny<CancellationToken>()))
    .ReturnsAsync(new GameStatus { LoadedPacks = new[] { "pack-1" } });

// Mock IFileSystem for testing content loading
var mockFS = new Mock<IFileDiscoveryService>();
mockFS
    .Setup(x => x.GetFiles("packs", "*.yaml", SearchOption.AllDirectories))
    .Returns(new[] { "packs/unit.yaml", "packs/building.yaml" });

// Don't mock: registry operations, stat calculations
var registry = new Registry<UnitDefinition>();  // Real object
registry.Add("unit-1", unit);
```

## Test Fixtures

### GameFixture (Integration Tests)
```csharp
public class GameFixture : IAsyncLifetime
{
    private GameClient _client;
    private RegistryManager _registry;

    public async Task InitializeAsync()
    {
        _client = new GameClient();
        await _client.ConnectAsync();
        _registry = new RegistryManager();
    }

    public async Task DisposeAsync()
    {
        _client?.Disconnect();
        _client?.Dispose();
    }

    public GameClient Client => _client;
    public RegistryManager Registry => _registry;
}

// Usage
[Collection("Game")]
public class GameLaunchPackTests
{
    private readonly GameFixture _fixture;

    [Fact]
    public async Task LoadPack_VerifiesContent()
    {
        // Test using _fixture.Client, _fixture.Registry
    }
}
```

### PackFixture (Content Tests)
```csharp
public class PackFixture
{
    public PackManifest CreateExampleBalance()
    {
        return new PackManifest
        {
            Id = "example-balance",
            Name = "Example Balance Pack",
            Version = "0.1.0",
            FrameworkVersion = ">=0.1.0",
            Author = "Test",
            Type = PackType.Balance
        };
    }

    public string[] GetExamplePackPaths()
    {
        return new[]
        {
            "packs/example-balance",
            "packs/warfare-starwars",
            "packs/economy-balanced"
        };
    }
}
```

## Coverage Targets

### By Subsystem
| Subsystem | Line | Branch | Method |
|-----------|------|--------|--------|
| Protocol | 100% | 100% | 100% |
| SDK | 85% | 75% | 95% |
| Bridge.Client | 85% | 75% | 95% |
| Domains | 90% | 80% | 95% |
| Tools | 85% | 75% | 93% |

### By Path Type
| Path Type | Target | Validation |
|-----------|--------|------------|
| Happy path | 100% | Always tested |
| Error paths | 90% | Exception handling |
| Edge cases | 80% | Boundary conditions |
| Recovery | 80% | State restoration |

## Continuous Integration

### Test Execution in CI
```yaml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-dotnet@v4
      with:
        dotnet-version: '11.0.100-preview.2'

    - run: dotnet build src/DINOForge.sln
    - run: dotnet test src/Tests --no-build /p:CollectCoverage=true

    - run: |
        if [ $(coverage) -lt 75 ]; then exit 1; fi
```

### Coverage Gates
- **Minimum coverage**: 75% (hard gate)
- **Target coverage**: 85%+ (goal)
- **Test pass rate**: 100% (required)
- **Build time**: <5 minutes (acceptable)

## Regression Testing

### Regression Test Suite
- All bug fixes have test cases
- Previous failures never recur
- Test count: ~100 (dedicated regression tests)

### Running Regression Tests
```bash
# Run all regression tests
dotnet test src/Tests -k "Regression"

# Run regression tests for specific component
dotnet test src/Tests -k "Registry AND Regression"
```

## Performance Testing

### Benchmarks
- ContentLoader: <1s for 100 units
- Registry operations: <5ms each
- Pack loading: <2s complete
- ECS queries: <100ms per query

### Running Benchmarks
```bash
dotnet run --project src/Tests/Benchmarks -- -f '*'
```

## Mutation Testing (Planned)

### Strategy
1. Use Stryker.NET to generate mutants
2. Run full test suite against each mutant
3. Track mutation kill rate
4. Identify weak assertions
5. Strengthen test expectations

### Goal: 75%+ mutation kill rate

## Test Maintenance

### Adding New Tests
1. Create test method in appropriate file
2. Follow naming convention
3. Include Arrange/Act/Assert comments
4. Run `dotnet test` locally
5. Verify coverage doesn't decrease
6. Submit PR with test

### Fixing Failing Tests
1. Check if test or code is wrong
2. If code: fix code, update test if needed
3. If test: fix test logic
4. Never skip flaky tests - investigate root cause
5. Document issue if environmental

### Refactoring Tests
- Keep tests readable over DRY
- Extract helpers for common patterns
- Use fixtures for repeated setup
- Avoid test interdependencies

## Tools & Dependencies

### Testing Frameworks
- **xUnit.net** - Test runner
- **FluentAssertions** - Assertions
- **Moq** - Mocking
- **FsCheck.Xunit** - Property testing
- **BenchmarkDotNet** - Performance benchmarking

### Coverage Tools
- **Coverlet** - Coverage collection
- **ReportGenerator** - Coverage reports
- **Stryker.NET** - Mutation testing (planned)

### CI Tools
- **GitHub Actions** - CI/CD
- **Codecov** - Coverage tracking
- **DotCover** - IDE integration

## References

- [xUnit Documentation](https://xunit.net/)
- [FluentAssertions Guide](https://fluentassertions.com/)
- [Property-Based Testing](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Test Pyramid](https://martinfowler.com/bliki/TestPyramid.html)

---

*Last updated: 2026-03-30*
*Next review: After mutation testing implementation*
