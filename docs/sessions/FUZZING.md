# Fuzzing and Randomized Testing

## Current Status

DINOForge does not currently ship a dedicated libFuzzer, AFL, or OSS-Fuzz integration.

The repository does include randomized invariant testing through FsCheck property-based tests in [src/Tests/PropertyTests.cs](src/Tests/PropertyTests.cs). Those tests exercise parser and domain behavior across generated inputs rather than a fixed set of example cases.

Current randomized coverage includes:

- semantic-version range matching invariants in `CompatibilityChecker`
- pack dependency resolution ordering and cycle detection
- warfare balance invariants such as non-negative power and damage clamping
- registry behavior invariants such as register/get/contains consistency

## How To Run It

Run the property-based test suite through the normal test entrypoints:

```bash
dotnet test src/DINOForge.CI.sln --verbosity normal
dotnet test src/DINOForge.sln --verbosity normal
```

To inspect the current randomized tests directly, review [src/Tests/PropertyTests.cs](src/Tests/PropertyTests.cs).

## Scope and Gaps

The current approach is useful for invariant validation, but it is not a substitute for coverage-guided fuzzing.

Known gaps:

- no coverage-guided native or managed fuzz target
- no continuous corpus management
- no external fuzzing service integration
- no dedicated fuzz harnesses for CLI argument parsing or YAML/JSON content ingestion

## Near-Term Candidates

Plausible next targets for stronger fuzz coverage are:

- pack manifest and schema-driven content loading
- dependency graph parsing and cycle detection edge cases
- CLI command parsing and option validation
- asset metadata readers that accept untrusted pack input

Any future fuzz harness should be checked into the repository with a documented invocation and regression workflow.
