# ADR-008: Wrap, Don't Handroll

**Status**: Accepted
**Date**: 2026-03-09
**Deciders**: kooshapari

## Context

DINOForge is developed entirely through vibecoding (agent-driven development with no human code review). This means:

- Agents produce more reliable output when integrating proven code than when generating novel implementations
- Handrolled code has higher defect rates and lacks community testing
- No human is available to catch subtle bugs in custom implementations
- Every handrolled component becomes a maintenance liability that agents handle poorly
- Maximizing feature coverage while minimizing risk is the priority

## Decision

**Always prefer existing libraries and tools over custom implementations.**

### Priority Order

1. **Direct use** of an existing library/tool as-is
2. **Thin wrapper / adapter** around an existing library to match our interfaces
3. **Composition** of multiple existing libraries
4. **Modified fork** of an existing library (last resort before handroll)
5. **Handroll** only when no alternative exists

### When Handrolling is Acceptable

- No existing solution covers the need (e.g. DINO-specific ECS glue code)
- Wrapping would be more complex than a simple implementation
- The scope is tiny and self-contained (&lt; 50 lines)
- The component is inherently game-specific (runtime hook layer)

### Concrete Dependency Preferences

| Capability | Preferred Package | Avoid |
|-----------|-------------------|-------|
| Schema validation | JsonSchema.Net / NJsonSchema | Custom validator |
| Semver / dependency resolution | Semver.NET, NuGet resolver patterns | Custom semver parser |
| Logging | Serilog / NLog / BepInEx logger | Custom logging framework |
| CLI tooling | System.CommandLine / Spectre.Console | Custom arg parsing |
| Config management | BepInEx ConfigurationManager | Custom config system |
| YAML parsing | YamlDotNet | Custom YAML parser |
| JSON serialization | System.Text.Json | Custom JSON handling |
| Diffing | DiffPlex | Custom diff engine |
| Testing | xUnit + FluentAssertions + Moq | Custom test framework |
| File watching | FileSystemWatcher | Custom polling |
| ECS access | Unity.Entities APIs (wrapped) | Raw reflection |

### Wrapper Pattern

When wrapping, use the adapter pattern:

```csharp
// Our interface (stable, owned by us)
public interface IPackValidator
{
    PackValidationResult Validate(PackManifest manifest);
}

// Wrapper around NJsonSchema (implementation detail, swappable)
internal class NJsonSchemaPackValidator : IPackValidator
{
    private readonly JsonSchema _schema;

    public PackValidationResult Validate(PackManifest manifest)
    {
        // Delegate to NJsonSchema, translate results to our types
    }
}
```

This gives us:
- Stable public API that agents code against
- Swappable implementation if the underlying library changes
- Battle-tested validation logic we didn't have to write or debug

## Consequences

- Agents search NuGet/GitHub before writing new code
- PR/commit reviews (automated) flag new files that could be library wrappers
- Dependency count will be higher than a handrolled project - this is intentional and correct
- Runtime layer (ECS glue) is the primary exception where handrolling is expected
- SDK and Tools layers should be almost entirely composed of wrapped dependencies
- Pack system should use zero handrolled parsers/validators
