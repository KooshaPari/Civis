# Test Isolation Policy (Pattern #93)

This is the policy enforced by `scripts/ci/detect_global_state_tests.py`
(task #257) and the matching workflow `.github/workflows/test-isolation.yml`.

## Why this exists

xUnit runs **test classes in parallel by default**. Methods within one
class run sequentially, but two different classes can interleave on
separate threads, sharing the same process and therefore the same:

- environment variables (`Environment.SetEnvironmentVariable`),
- working directory (`Directory.SetCurrentDirectory`),
- static fields (any `static` member, especially `static readonly` paths
  computed at class init),
- named pipes,
- TCP ports.

If two test classes both touch `Environment.SetEnvironmentVariable("X")`,
the value seen by either class is non-deterministic — tests pass on a dev
machine and fail intermittently in CI, or vice versa. This is **Pattern
#93: Order-Dependent / Process-Global State**.

## The four-collection registry

`src/Tests/Collections.cs` exposes four constants. Each maps to a
`[CollectionDefinition(..., DisableParallelization = true)]` and forces
serial execution of all member tests:

| Constant                         | Use when a test...                        |
|----------------------------------|-------------------------------------------|
| `Collections.EnvVarMutation`     | calls `Environment.SetEnvironmentVariable` |
| `Collections.WorkingDirectory`   | calls `Directory.SetCurrentDirectory`     |
| `Collections.BridgePipe`         | binds the dinoforge MCP named pipe        |
| `Collections.NetworkPort`        | binds a fixed (non-ephemeral) TCP port    |

(Two pre-existing collections — `Collections.AssetSwapRegistry` and
`Collections.UiAutomation` — are also surfaced for completeness.)

### Usage

```csharp
[Collection(Collections.EnvVarMutation)]
public class MyEnvTest : IDisposable
{
    private readonly string? _snapshot;

    public MyEnvTest()
    {
        // Snapshot in ctor — serialized via the collection.
        _snapshot = Environment.GetEnvironmentVariable("DINOFORGE_TEST_FOO");
    }

    public void Dispose()
    {
        // Restore in Dispose — even if the test threw.
        Environment.SetEnvironmentVariable("DINOFORGE_TEST_FOO", _snapshot);
    }

    [Fact]
    public void MyTest()
    {
        Environment.SetEnvironmentVariable("DINOFORGE_TEST_FOO", "value");
        // ... assertions ...
    }
}
```

## Required env-var prefix

Tests that set environment variables MUST use the `DINOFORGE_TEST_`
prefix. **Production env-var names** (`DINOFORGE_RESOLVER_PATH`,
`DINOFORGE_HOME`, etc.) leak into the same process that hosts production
code under test — corrupting later tests, causing CI flake, and hiding
real bugs in the production path.

| Severity | Pattern                                                             | Example                                              |
|----------|---------------------------------------------------------------------|------------------------------------------------------|
| **HIGH** | `Environment.SetEnvironmentVariable("DINOFORGE_<not TEST_>...")` | `Environment.SetEnvironmentVariable("DINOFORGE_RESOLVER_PATH", "/tmp/foo")` |
| OK       | `Environment.SetEnvironmentVariable("DINOFORGE_TEST_<...>")`     | `Environment.SetEnvironmentVariable("DINOFORGE_TEST_FOO", "/tmp/foo")` |

## Snapshot-in-ctor pattern

Tests that mutate any process-global state MUST snapshot the original
value in their ctor and restore it in `Dispose()`. This handles three
edge cases:

1. The variable was set externally (parent shell, prior test run).
2. The test throws mid-run.
3. The variable is consumed by another test in the same collection.

```csharp
public MyTest()
{
    _snapshot = Environment.GetEnvironmentVariable(Var);
}

public void Dispose()
{
    Environment.SetEnvironmentVariable(Var, _snapshot);
}
```

`Environment.SetEnvironmentVariable(name, null)` clears the variable;
restoring `null` does the right thing.

## Class-static path discipline

A class-level `static readonly string` initialised from
`Path.Combine(Path.GetTempPath(), "literal")` is shared by EVERY instance
of the class — including those running in parallel under different test
classes that import the same fixture. Use a `Guid.NewGuid()` (or
`Path.GetRandomFileName()`) discriminator:

```csharp
// BAD: two parallel runs collide on the same path.
private static readonly string Shared =
    Path.Combine(Path.GetTempPath(), "dinoforge-fixtures");

// GOOD: each test class instance gets a unique directory.
private static readonly string Unique =
    Path.Combine(Path.GetTempPath(), "df-" + Guid.NewGuid());
```

## What the gate detects

| Rule key                            | Severity | Description                                          |
|-------------------------------------|----------|------------------------------------------------------|
| `raw_prod_env_var`                  | HIGH     | `SetEnvironmentVariable("DINOFORGE_<not TEST_>...")` |
| `env_mutation_without_collection`   | MED      | `SetEnvironmentVariable` in a class with no `[Collection]` |
| `set_current_directory`             | MED      | Any `Directory.SetCurrentDirectory` call             |
| `class_static_path_no_guid`         | LOW      | `static (readonly)? string` field with `Path.Combine`/`GetTempPath` and no Guid |
| `hardcoded_temp_path`               | LOW      | `Path.Combine(Path.GetTempPath(), "literal")` with no Guid in scope |

## Allowlist

`docs/qa/global-state-allowlist.txt` accepts one entry per line. Two
forms are recognised:

- A full key: `rule|fqn|line|file` — line-locked.
- A bare FQN: `Some.Namespace.SomeClass` — suppresses every rule for
  every line in that class.

`#`-comments and blanks are ignored. The line-locked form is preferred
because moving code requires a fresh allowlist refresh — that's a
feature, not a bug.

## Cross-references

- Detection script: `scripts/ci/detect_global_state_tests.py`
- Workflow: `.github/workflows/test-isolation.yml`
- Reference impl: `src/Tests/EnvVarMutationCollection.cs`,
  `src/Tests/AssetSwapRegistryCollection.cs`
- Companion patterns: #91 (Tautological Tests), #92 (Audit Ledger Decay)
- Tasks: #93/#255/#256/#257
