# Fuzz Corpus

Seed inputs for SharpFuzz coverage-guided fuzzing targets.

## Directories

- `yaml/` — pack.yaml fragments (valid, malformed, unicode, edge cases)
- `json/` — JSON schema fragments (valid, malformed, deeply nested)
- `semver/` — semver boundary strings (valid boundaries, invalid inputs)

## Fuzz Targets

| Target | File | Description |
|--------|------|-------------|
| `YamlFuzzTarget` | `FuzzTargets/YamlFuzzTarget.cs` | YamlDotNet parser via arbitrary YAML input |
| `JsonFuzzTarget` | `FuzzTargets/JsonFuzzTarget.cs` | System.Text.Json parser via arbitrary JSON input |
| `SemverFuzzTarget` | `FuzzTargets/SemverFuzzTarget.cs` | CompatibilityChecker version range parsing |
| `PackManifestFuzzTarget` | `FuzzTargets/PackManifestFuzzTarget.cs` | Full PackManifest deserialization pipeline |

## Running Fuzz Targets

Fuzz targets are tagged with `[Trait("Category", "Fuzz")]`. To run smoke tests:

```bash
dotnet test src/DINOForge.CI.sln --filter "Category=Fuzz" --configuration Release
```

To run full coverage-guided fuzzing (requires AFL++ or libFuzzer via SharpFuzz tooling):

```bash
# 1. Instrument the assembly
sharpfuzz src/Tests/bin/Release/net8.0/DINOForge.Tests.dll

# 2. Run with AFL++
afl-fuzz -i src/Tests/FuzzCorpus/yaml -o /tmp/fuzz-out -- \
    dotnet DINOForge.Tests.dll DINOForge.Tests.FuzzTargets.YamlFuzzTarget::Fuzz @@
```

## Policy

Any newly discovered crash-inducing input MUST be added here as a regression fixture
before the PR is merged. Name the file after the crash type (e.g., `crash-null-ref-001.yaml`).
