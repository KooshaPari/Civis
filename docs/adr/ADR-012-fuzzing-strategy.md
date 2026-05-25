# ADR-012 — Fuzzing and Property-Based Testing Strategy

**Date**: 2026-03-14
**Status**: Accepted
**Deciders**: kooshapari
**Related**: ADR-001 (Agent-Driven Dev), ADR-008 (Wrap Don't Handroll)

---

## Context

DINOForge processes complex, user-authored content at multiple ingestion points:

- **YAML pack manifests** (`pack.yaml`, `units.yaml`, `factions.yaml`, etc.) parsed by YamlDotNet
- **JSON schema validation** by NJsonSchema
- **Semver version strings** parsed and compared by SemVersion / custom logic
- **ECS component mappings** resolved by `PackStatMappings` string lookup
- **Stat modifier expressions** evaluated at runtime
- **ContentLoader** pipeline (discover → validate → register → apply)

Any of these paths can receive adversarial or malformed input from a third-party pack. A crash or silent data corruption in these paths can corrupt game state or cause BepInEx to unload the plugin.

### Current State (Audit 2026-03-14)

- **FsCheck.Xunit 3.3.2** is integrated in `src/Tests/DINOForge.Tests.csproj` ✓
- **`PropertyTests.cs`** (566 lines) covers 4 domains with ~14 properties:
  - `CompatibilityCheckerPropertyTests` — 5 properties (wildcard/range matching, monotonicity)
  - `RegistryPropertyTests` — 4 properties (insert/retrieve/conflict)
  - `DependencyResolverPropertyTests` — 3 properties (resolution correctness)
  - `BalanceCalculatorPropertyTests` — 2 properties (invariant checks)
- **No SharpFuzz** — no coverage-guided/crash-detection fuzzing
- **No persistent corpus** — no replay database for discovered edge cases
- **Uncovered serialization paths**: ContentLoader, PackManifest parser, schema validator, stat modifier, AssetSwapRegistry

---

## Options Considered

### Option A — FsCheck expansion only
Extend FsCheck to cover all remaining serialization/registry paths (~30+ properties total).
Pros: Already integrated, zero new tooling.
Cons: FsCheck generates random valid data; may miss crash-inducing byte-level edge cases in parsers.

### Option B — SharpFuzz only (coverage-guided)
Add `SharpFuzz` NuGet and run libFuzzer on YAML/JSON/semver parse functions.
Pros: Better at finding parser crashes from malformed byte sequences.
Cons: Requires separate runner infrastructure; harder to run in standard `dotnet test`.

### Option C — FsCheck + SharpFuzz (Chosen)
Expand FsCheck for logic correctness (30+ properties), add SharpFuzz for parser crash-detection (4 targets), build persistent corpus, add nightly CI gate.

Pros: Full coverage spectrum — logic correctness via properties, crash detection via coverage-guided fuzzing.
Cons: Slightly more setup; SharpFuzz targets run separately from unit test suite.

---

## Decision

**Adopt Option C**: FsCheck expansion + SharpFuzz parser targets + persistent corpus.

FsCheck covers the "is this semantically correct?" dimension. SharpFuzz covers the "does this crash?" dimension. These are complementary and together provide the full testing surface for a data-driven platform.

---

## Implementation Plan

### Phase 1 — FsCheck Expansion (10 new domains)

Extend `PropertyTests.cs` or add `PropertyTests.Extended.cs`:

| Domain | Properties |
|--------|-----------|
| ContentLoader YAML→object round-trip | 3 |
| PackLoader manifest edge cases | 2 |
| Schema validation boundary conditions | 3 |
| Stat modifier combinatorial correctness | 3 |
| AssetSwapRegistry concurrent Register | 2 |
| Entity query generation (vanilla_mapping strings) | 2 |
| DependencyResolver cycle detection | 2 (extend existing) |
| CompatibilityChecker monotonicity | 2 (extend existing) |
| TradeEngine balance invariants | 3 |
| ScenarioRunner condition evaluation | 2 |
| **Total new** | **~24 new properties** |

All FsCheck tests tagged `[Category("Property")]`.

### Phase 2 — SharpFuzz Targets

```
src/Tests/FuzzTargets/
├── YamlFuzzTarget.cs          # YamlDotNet pack.yaml deserialization
├── JsonFuzzTarget.cs          # NJsonSchema schema validation
├── SemverFuzzTarget.cs        # Semver.NET string parsing
└── PackManifestFuzzTarget.cs  # Full PackManifest round-trip
```

Each target implements:
```csharp
[SharpFuzz.Fuzz]
public static void Fuzz(ReadOnlySpan<byte> data)
{
    try { /* parse data */ }
    catch (Exception e) when (e is not OutOfMemoryException) { /* swallow expected */ }
}
```

Runner: `scripts/run-fuzz.sh` (libFuzzer via AFL++ or libFuzzer on Linux CI).

### Phase 3 — Corpus

```
src/Tests/FuzzCorpus/
├── yaml/          # Interesting pack.yaml fragments (malformed, boundary, unicode)
├── json/          # Interesting schema JSON fragments
├── semver/        # Boundary semver strings (0.0.0, 999.999.999, pre-release, build metadata)
└── README.md      # Corpus maintenance policy
```

Corpus is committed to git. Newly discovered crash-inducing inputs are added as regression fixtures.

### Phase 4 — CI

- `dotnet test --filter "Category=Property"` in PR gate (fast, <30s)
- `dotnet test --filter "Category=Fuzz"` in nightly workflow (longer run budget)
- SharpFuzz targets run in `.github/workflows/fuzz.yml` on nightly schedule against main

---

## Consequences

**Positive**:
- All serialization paths have crash regression coverage
- Logic correctness verified across 30+ properties
- Third-party pack authors get guaranteed safety guarantees for their YAML

**Negative**:
- SharpFuzz requires Linux CI runner (not Windows) — isolate in separate workflow
- Corpus maintenance is an ongoing responsibility

**Risks**:

| Risk | Likelihood | Mitigation |
|------|------------|-----------|
| SharpFuzz corpus grows too large for git | Low | Limit to 1KB inputs; .gitattributes exclude binary blobs |
| FsCheck generators produce unrealistic data | Medium | Use constrained custom generators (see existing VersionGenerators pattern) |
| False positives in property tests from flaky generators | Low | Set fixed seed for nightly runs; random seed for PR gate |
