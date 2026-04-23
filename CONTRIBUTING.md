# Contributing to DINOForge

Thank you for your interest in contributing to DINOForge! This document provides guidelines for contributing code, documentation, content packs, and domain plugins to our agent-driven mod platform for **Diplomacy is Not an Option (DINO)**.

## Welcome

DINOForge is a **mod operating system**, not a single mod. It provides:

- **Runtime**: BepInEx-compatible ECS plugin for DINO with component mapping
- **SDK**: Public mod API with typed registries, schemas, and pack loading
- **Domain Plugins**: Extensible systems (Warfare, Economy, Scenario, UI)
- **Pack System**: Declarative YAML content with validation, dependency resolution, and hot-reload
- **Tooling**: CLI tools for pack compilation, asset pipelines, and debugging
- **MCP Bridge**: Game automation and testing via Claude Code integration

We believe modding should be declarative, composable, and agent-friendly. Every contribution — whether code, packs, tests, or documentation — helps us build a more robust ecosystem.

**Key Design Principles**:
- **Wrap, don't handroll** — Always prefer existing libraries over custom implementations
- **Declarative before imperative** — YAML/JSON manifests over C# patches
- **Framework before content** — Platform stability takes priority
- **Agent-first** — Optimized for autonomous agent-driven development
- **Schema-driven** — Schemas validate before runtime

See [CLAUDE.md](./CLAUDE.md) for the full agent governance model and technical rules.

## System Requirements

### Required

- **.NET 11 Preview SDK** (`11.0.100-preview.2.26159.112`) — For building tools and CLI
  - [Download .NET 11](https://dotnet.microsoft.com/download/dotnet/11.0)
  - Verify: `dotnet --version` should show `11.0.100-preview.2.26159.112`
- **.NET 8.0 SDK** — For SDK/domain library builds (netstandard2.0 compat)
  - [Download .NET 8.0](https://dotnet.microsoft.com/download/dotnet/8.0)
- **Git** 2.20+
- **Windows** (DINO and BepInEx are Windows-only; Linux/macOS via WSL supported)

### Optional but Recommended

- **Diplomacy is Not an Option** (Steam) — For in-game mod testing
- **Visual Studio Code** or **Visual Studio 2022** — For C# development
- **BepInEx 5.4.x** — For testing mods in-game
- **pre-commit** — For automatic code checks (`pip install pre-commit && pre-commit install`)

## Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/KooshaPari/Dino.git
cd Dino
```

### 2. Verify .NET Version

```bash
# Check your SDK versions
dotnet --version
# Should include: 11.0.100-preview.2.26159.112

# List all SDKs
dotnet --info
```

If .NET 11 preview is missing, [download it here](https://dotnet.microsoft.com/download/dotnet/11.0).

### 3. Build the Solution

```bash
# Build everything (excluding Runtime which requires game DLLs)
dotnet build src/DINOForge.CI.sln

# Or build the full solution if you have DINO installed
dotnet build src/DINOForge.sln
```

### 4. Run Tests

All tests must pass before submitting a PR.

```bash
# Run all tests (CI solution — no game dependencies)
dotnet test src/DINOForge.CI.sln --verbosity normal

# Run tests with coverage
dotnet test src/DINOForge.CI.sln /p:CollectCoverage=true /p:CoverageFormat=opencover

# Run tests from full solution (requires game DLLs)
dotnet test src/DINOForge.sln --verbosity normal

# Run specific test category
dotnet test src/DINOForge.CI.sln --filter "Category=Unit"
```

### 5. Code Formatting

DINOForge uses `dotnet format` to enforce consistent style.

```bash
# Check formatting (fails if changes needed)
dotnet format src/DINOForge.CI.sln --verify-no-changes

# Auto-fix formatting
dotnet format src/DINOForge.CI.sln
```

### 6. (Optional) Install Game and BepInEx

If you want to test mods in-game:

1. Install [Diplomacy is Not an Option](https://store.steampowered.com/app/1272320/) from Steam
2. Install **BepInEx 5.4.x** with ECS support from [GitHub releases](https://github.com/BepInEx/BepInEx/releases)
3. Extract to your game directory
4. Set `single-instance=0` in `Diplomacy is Not an Option_Data/boot.config`
5. Verify the structure:
   ```
   G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\
     winhttp.dll                          (BepInEx loader)
     Diplomacy is Not an Option.exe
     doorstop_config.ini
     Diplomacy is Not an Option_Data/
       boot.config                        (must contain: single-instance=0)
     BepInEx/
       plugins/
       ecs_plugins/                       (where DINOForge.Runtime.dll goes)
       config/
       dinoforge_packs/                   (pack directory)
       dinoforge_debug.log                (runtime logs)
   ```
6. Update `Directory.Build.props` with your DINO installation path if deploying locally

## Build Commands Reference

```bash
# Build CI solution (no game dependencies)
dotnet build src/DINOForge.CI.sln

# Build full solution (requires DINO DLLs)
dotnet build src/DINOForge.sln

# Test
dotnet test src/DINOForge.CI.sln --verbosity normal

# Format (check)
dotnet format src/DINOForge.CI.sln --verify-no-changes

# Format (fix)
dotnet format src/DINOForge.CI.sln

# Validate all packs
dotnet run --project src/Tools/PackCompiler -- validate packs/

# Build a specific pack
dotnet run --project src/Tools/PackCompiler -- build packs/<pack-id>
```

## Architecture Overview

### Layered Design

```
Packs (user-created mods)
  ↓
Domain Plugins (Warfare, Economy, Scenario, UI)
  ↓
SDK (registries, content loaders, validators, asset services)
  ↓
Runtime (BepInEx plugin, ECS bridge, component mapping)
  ↓
Game (DINO Unity ECS)
```

### Directory Structure

```
src/
  Runtime/              # BepInEx plugin bootstrap, ECS detection, hooks
    Bridge/             #   Component mapping, stat modifiers, entity queries
  SDK/                  # Public mod API
    Assets/             #   Addressables catalog, asset loaders
    Dependencies/       #   Pack dependency resolver
    Models/             #   Content data structures
    Registry/           #   Generic extensible registry system
    Validation/         #   Schema validation (NJsonSchema)
  Domains/
    Warfare/            #   Factions, doctrines, combat, balance, waves
    Economy/            #   Rates, trade, production (planned)
    Scenario/           #   Scripting, conditions (planned)
    UI/                 #   HUD injection, menus (planned)
  Tools/
    Cli/                #   dinoforge CLI tool
    PackCompiler/       #   Pack validation and build
    DumpTools/          #   Entity analysis (Spectre.Console)
    Installer/          #   BepInEx + DINOForge installer
  Tests/                #   xUnit + FluentAssertions
    Integration/        #   Integration tests
  Templates/            #   Content generators
packs/                  # Official and example packs
schemas/                # JSON/YAML schema definitions (17 schemas)
```

## Code Style & Conventions

DINOForge uses modern C# idioms enforced via `.editorconfig` and `dotnet format`.

### C# Style Rules

- **Language version**: C# 12+ with nullable reference types enabled
- **Async/await**: Prefer `async/await` over raw `Task`s
- **XML documentation**: Required on all public APIs
- **Naming conventions**:
  - `PascalCase` for types, methods, properties
  - `camelCase` for parameters and local variables
  - `_camelCase` for private fields
- **Type inference**: No `var` for non-obvious types
  ```csharp
  // Bad: type not immediately clear
  var result = registry.Register(unit);

  // Good: type explicit
  bool registered = registry.Register(unit);
  ```
- **Immutability**: Prefer immutable data models via `record` or `init` properties
  ```csharp
  // Good: immutable record
  public record Unit(string Id, string Name, int MaxHealth);

  // Also good: init properties
  public class Unit
  {
      public string Id { get; init; }
      public string Name { get; init; }
  }
  ```
- **Design patterns**:
  - **Registry pattern** for all extensible domains (typed registries, conflict detection)
  - **Dependency injection** for services and bridges
  - **Thin adapters** around external libraries (wrap, don't fork)
  - **Composition over inheritance** (interfaces + composition > deep hierarchies)

### Example Public API

```csharp
/// <summary>
/// Registers a new unit with the game engine.
/// </summary>
/// <param name="unitId">Unique identifier for the unit type.</param>
/// <param name="config">Configuration specifying unit stats and abilities.</param>
/// <returns>True if registration succeeded; false if ID already exists.</returns>
/// <exception cref="ArgumentNullException">Thrown if unitId or config is null.</exception>
/// <remarks>
/// Units must have unique IDs. Attempting to register a duplicate ID returns false
/// without modifying the existing entry.
/// </remarks>
public bool RegisterUnit(string unitId, UnitConfig config)
{
    ArgumentNullException.ThrowIfNull(unitId);
    ArgumentNullException.ThrowIfNull(config);

    return _units.TryAdd(unitId, config);
}
```

### Formatting & Linting

Formatting is enforced in CI. Run locally before committing:

```bash
# Check formatting (fails if changes needed)
dotnet format src/DINOForge.sln --verify-no-changes

# Apply fixes
dotnet format src/DINOForge.sln
```

## Conventional Commits and SemVer

DINOForge uses Conventional Commits for release intent and Semantic Versioning for public releases.

### Commit format

Use:

```text
type(scope): summary
```

Accepted top-level types are:

- `feat`
- `fix`
- `docs`
- `test`
- `perf`
- `refactor`
- `chore`
- `ci`
- `build`
- `style`
- `revert`

### Version bump policy

While DINOForge is still in `0.x`, use the stricter KooshaPari rule set:

- `fix` / `perf` / `revert` normally map to a patch release
- `feat` maps to a minor release
- Breaking public contract changes still require an explicit changelog callout and should bump the minor version while the project remains in `0.x`
- After `1.0.0`, breaking public contract changes must bump the major version

Pre-release identifiers should use one of:

- `alpha.N`
- `beta.N`
- `rc.N`

## Keep a Changelog Rules

`CHANGELOG.md` is not optional release decoration. It is part of the release contract.

Required rules:

- keep `## [Unreleased]` at the top
- add all non-doc, non-meta changes to `CHANGELOG.md`
- use standard Keep a Changelog headings:
  - `Added`
  - `Changed`
  - `Deprecated`
  - `Removed`
  - `Fixed`
  - `Security`
- move `Unreleased` entries into a dated version section before tagging
- keep `VERSION` synchronized with the latest released changelog section

## Release Governance

Before cutting a release:

1. finalize `CHANGELOG.md`
2. ensure `VERSION` matches the release target
3. verify CI, formatting, and coverage are green
4. verify release artifacts and checksums
5. tag using `vX.Y.Z` or `vX.Y.Z-rc.N`

The canonical release playbook is in [RELEASING.md](./RELEASING.md).

## Shared KooshaPari Semantics

This repo follows the shared KooshaPari semantics for release signals, coverage, ownership, and governance docs. The shared contract and Dino-specific exceptions are documented in [docs/reference/kooshapari-project-semantics.md](./docs/reference/kooshapari-project-semantics.md).

## Creating a Content Pack

Packs are the primary way to extend DINOForge. They use declarative YAML manifests.

### Step 1: Create Pack Directory

```bash
mkdir -p packs/my-awesome-pack/{factions,units,buildings,doctrines}
```

### Step 2: Create `pack.yaml` Manifest

```yaml
id: my-awesome-pack
name: My Awesome Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: Your Name
description: A brief description of what this pack adds
type: content                    # content | balance | ruleset | total_conversion | utility
depends_on: []                   # List pack IDs this depends on
conflicts_with: []               # List pack IDs this conflicts with

loads:
  factions:
    - my-faction.yaml
  units:
    - my-unit.yaml
  buildings:
    - my-building.yaml
  doctrines: []
  waves: []
```

### Step 3: Add Content Files

Create content in the corresponding subdirectories. Each file should be valid JSON/YAML matching the appropriate schema.

Example `factions/my-faction.yaml`:
```yaml
id: my-faction
name: My Faction
description: A custom faction
color: "#FF0000"
units: []
buildings: []
```

### Step 4: Validate Your Pack

```bash
# Validate a single pack
dotnet run --project src/Tools/PackCompiler -- validate packs/my-awesome-pack

# Validate all packs
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

### Step 5: Test in Game (Optional)

1. Copy/symlink `packs/my-awesome-pack` to your DINO game directory's `packs/` folder
2. Run the game and load your pack
3. Check the mod menu (F10) for errors

See [Creating Packs](./docs/guide/creating-packs.md) for detailed schema documentation.

## Adding a New Domain Plugin

Domain plugins extend DINOForge with new gameplay systems (e.g., Warfare, Economy).

### Step 1: Create Project Structure

```bash
mkdir -p src/Domains/MyDomain
cd src/Domains/MyDomain
dotnet new classlib -f net8.0
```

### Step 2: Implement IDomainPlugin Interface

```csharp
using DINOForge.SDK;

namespace DINOForge.Domains.MyDomain;

public class MyDomainPlugin : IDomainPlugin
{
    public string Id => "my-domain";
    public string Name => "My Domain";
    public string Version => "0.1.0";

    /// <summary>Registers domain content with the SDK registries.</summary>
    public void RegisterSchemas(ISchemaRegistry schemas) { }

    /// <summary>Initializes domain systems in the ECS world.</summary>
    public void InitializeEcsWorld(World world) { }

    /// <summary>Called when a pack is loaded.</summary>
    public void OnPackLoaded(PackManifest pack) { }
}
```

### Step 3: Add to Runtime Loading

Register your plugin in `src/Runtime/Plugins/PluginLoader.cs`.

### Step 4: Add Tests

Create integration tests in `src/Tests/Domains/MyDomainTests.cs`.

See [src/Domains/Warfare](./src/Domains/Warfare) for a complete example.

## Submitting Issues

### Bug Reports

Use the **Bug Report** template. Include:

- Clear description of the problem
- Steps to reproduce (with specific commands/inputs)
- Expected vs. actual behavior
- DINOForge and DINO game versions
- Component affected (Runtime, SDK, PackCompiler, etc.)
- Error logs or stack trace (if applicable)

### Feature Requests

Use the **Feature Request** template. Include:

- Problem statement: What gap or pain point does this fill?
- Proposed solution: How should it work?
- Alternative approaches you've considered
- **Agent Move Class**: Which legal operation category? (see [CLAUDE.md](./CLAUDE.md) for options)

### Pack Feedback

Use the **Pack Feedback** template to report issues with specific packs (balance, content quality, bugs).

## Pull Request Process

### Before Submitting

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```

2. **Make your changes** and commit with clear messages:
   ```bash
   git commit -m "feat: add new unit registry

   Adds support for registering custom units with the SDK.
   Includes schema validation and conflict detection."
   ```

3. **Run the full test suite**:
   ```bash
   dotnet test src/DINOForge.CI.sln --verbosity normal
   ```

4. **Check code formatting**:
   ```bash
   dotnet format src/DINOForge.CI.sln --verify-no-changes
   ```

5. **Validate any packs you modified**:
   ```bash
   dotnet run --project src/Tools/PackCompiler -- validate packs/
   ```

### Submitting the PR

1. Push your branch and create a pull request against `main`
2. Fill out the PR template (see [.github/pull_request_template.md](./.github/pull_request_template.md))
3. Link any related issues: `Closes #123`

### PR Requirements

All of the following must pass before merging:

- ✅ All tests pass (`dotnet test src/DINOForge.CI.sln`)
- ✅ Code formatting passes (`dotnet format --verify-no-changes`)
- ✅ No new compiler warnings
- ✅ Pack validation passes (if applicable)
- ✅ Documentation updated (if public API changed)
- ✅ CHANGELOG.md updated with your changes

### Checklist for Contributors

Before marking as ready for review:

- [ ] Tests added/updated for new functionality
- [ ] XML doc comments added to public APIs
- [ ] No hardcoded content IDs in engine glue
- [ ] Schemas validated (if modified)
- [ ] Registry pattern used (if adding extensibility)
- [ ] Dependencies declared in pack manifests
- [ ] CHANGELOG.md updated
- [ ] README.md updated (if structure/commands changed)

## Changelog Conventions

Update `CHANGELOG.md` using [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [0.2.0] - 2026-03-15

### Added
- New feature description

### Changed
- Updated behavior description

### Fixed
- Bug fix description

### Deprecated
- Deprecated feature description
```

Always update CHANGELOG when:
- Adding features
- Fixing bugs
- Changing public APIs
- Deprecating functionality
- Releasing a new version

## ADR Process (Architectural Decision Records)

For significant architecture changes or design decisions:

1. Create a new file: `docs/adr/ADR-NNN-title-slug.md`
2. Use the [ADR template](./docs/adr/template.md)
3. Include context, decision, consequences, alternatives considered
4. Link from relevant architectural documentation
5. Get review from maintainers before implementation

See [docs/adr/](./docs/adr/) for examples.

## Legal Move Classes

All contributions should map to one of these **legal move classes**:

- **`create schema`** — New data shape definition (add JSON schema)
- **`extend registry`** — Add entries to existing registry
- **`add content pack`** — New pack with manifest
- **`patch mapping`** — Update vanilla-to-mod component mapping
- **`write validator`** — New validation rule
- **`add test fixture`** — New test case or test data
- **`add debug view`** — New diagnostic overlay or inspector view
- **`add migration`** — Version compatibility migration
- **`add compatibility rule`** — Cross-pack conflict rule
- **`add documentation manifest`** — Update docs or guides

Reframe your work to fit one of these categories. See [CLAUDE.md](./CLAUDE.md) for details.

## Testing Requirements

DINOForge maintains **1,000+ tests** with **95%+ code coverage**. We use **BDD + TDD + SDD**:

- **BDD** (Behavior-Driven Development): Test acceptance criteria first
- **TDD** (Test-Driven Development): All public APIs require unit tests
- **SDD** (Spec-Driven Development): Schemas validate before runtime
- **Property-based testing**: Validate invariants across random inputs (FsCheck)

### Test Categories

Tests are organized by category and tagged with attributes:

| Category | Attribute | Purpose | Example |
|----------|-----------|---------|---------|
| **Unit** | `[Category("Unit")]` | Single function/class in isolation | Registry.Register() |
| **Integration** | `[Category("Integration")]` | Component interactions, I/O, databases | ContentLoader + Registry |
| **Property** | `[Category("Property")]` | Invariants across random inputs | Dependency resolution never cycles |
| **GameAutomation** | `[Category("GameAutomation")]` | Tests against live/mocked game | Asset swap, stat modifiers |

### Writing Tests

Use **xUnit** + **FluentAssertions** + **Moq**:

```csharp
using Xunit;
using FluentAssertions;

public class UnitRegistryTests
{
    private readonly UnitRegistry _registry = new();

    [Fact]
    [Category("Unit")]
    public void Register_WithUniqueId_Succeeds()
    {
        // Arrange
        var unit = new Unit("warrior", "Warrior", maxHealth: 50);

        // Act
        bool registered = _registry.Register(unit);

        // Assert
        registered.Should().BeTrue();
        _registry.Get("warrior").Should().NotBeNull();
        _registry.Get("warrior")!.MaxHealth.Should().Be(50);
    }

    [Fact]
    [Category("Unit")]
    public void Register_WithDuplicateId_FailsSilently()
    {
        // Arrange
        var unit1 = new Unit("warrior", "Warrior", maxHealth: 50);
        var unit2 = new Unit("warrior", "Different", maxHealth: 60);
        _registry.Register(unit1);

        // Act
        bool registered = _registry.Register(unit2);

        // Assert
        registered.Should().BeFalse();
        _registry.Get("warrior")!.MaxHealth.Should().Be(50);  // Original unchanged
    }

    [Theory]
    [Category("Property")]
    [InlineData("unit-1")]
    [InlineData("unit-2")]
    [InlineData("long-unit-name-with-dashes")]
    public void Register_WithValidId_NeverThrows(string unitId)
    {
        // Arrange
        var unit = new Unit(unitId, "Name", maxHealth: 50);

        // Act & Assert
        var action = () => _registry.Register(unit);
        action.Should().NotThrow();
    }
}
```

### Coverage Requirements

- **New public APIs**: 95%+ coverage required
- **Modified code**: Coverage must not decrease
- **Exclusions**: Model classes (records), auto-generated code, ECS glue

Measure coverage locally:

```bash
dotnet test src/DINOForge.CI.sln /p:CollectCoverage=true /p:CoverageFormat=opencover
# Look for: src/Tests/coverage.xml
```

GitHub will block merge if coverage drops below the baseline.

### Testing Best Practices

1. **Arrange-Act-Assert (AAA)** — Organize every test with clear setup, action, assertion phases
2. **One assertion per behavior** — Test one thing per test (multiple assertions on same concept is OK)
3. **Meaningful names** — Test name = "should X when Y given Z"
4. **No test interdependencies** — Tests must run in any order
5. **Avoid mocking registries** — Mock external services (HTTP, files), not domain logic
6. **Use fixtures for reusable setup** — Don't repeat Arrange logic

## Code of Conduct

We're committed to providing a welcoming and inclusive environment. All contributors must abide by our [Code of Conduct](./CODE_OF_CONDUCT.md).

Examples of unacceptable behavior:
- Harassment or discrimination
- Intentional disruption
- Trolling or bad-faith engagement
- Publishing others' private information

Report violations to the maintainers.

## Questions?

- **Documentation**: Check [docs/](./docs/)
- **Issues**: Search [existing issues](https://github.com/KooshaPari/Dino/issues)
- **Architecture**: Read [CLAUDE.md](./CLAUDE.md)
- **Discussions**: Start a [GitHub Discussion](https://github.com/KooshaPari/Dino/discussions)

Thank you for contributing to DINOForge!
