# Pattern #112 — Direct DateTime.Now/UtcNow in Production Code

**Alias**: Pattern #100 (`detect_direct_datetime.py`)  
**Status**: Active gate (CI enforced)  
**Detection**: `scripts/ci/detect_direct_datetime.py` (382 lines, self-testing)  
**Allowlist**: `docs/qa/direct-datetime-allowlist.txt`

---

## The Smell

Production code reaches for `DateTime.Now` or `DateTime.UtcNow` directly:

```csharp
// BAD: wall-clock singleton, untestable, deadline-loop hazard.
public bool IsExpired(DateTime issuedAt, TimeSpan ttl)
{
    return DateTime.UtcNow - issuedAt > ttl;
}
```

---

## Why It's Bad

### 1. **Untestable**
- Unit tests that assert behavior at a specific instant must either:
  - `Thread.Sleep(...)` — slow, flaky, timing-dependent
  - Accept arbitrary clock skew — brittle tests
- **Fix**: Inject `TimeProvider` (C# 8+) so tests substitute `FakeTimeProvider`

### 2. **Deadline-Loop Hazard** ⚠️
- NTP step, DST transitions, or VM resume can jump the wall clock backward
- A `while (DateTime.UtcNow < deadline)` loop **never terminates** if the clock steps back

```csharp
// DANGEROUS: can spin forever on NTP step.
var deadline = DateTime.UtcNow.AddSeconds(5);
while (DateTime.UtcNow < deadline) { DoWork(); }  // BROKEN if clock steps back
```

- **Fix**: Use `Stopwatch.GetTimestamp()` (monotonic) or `TimeProvider.GetUtcNow()`

### 3. **NuGet API Surface**
- Code in `src/SDK/`, `src/Bridge/Client/`, `src/Bridge/Protocol/` is consumed by external integrators
- Baking the wall clock into public methods **denies consumers testability**

---

## The Healthy Pattern

**Dependency-inject time**:

```csharp
public sealed class MyService
{
    private readonly TimeProvider _time;

    // Constructor injection — consumers can pass FakeTimeProvider in tests.
    public MyService(TimeProvider time)
    {
        _time = time;
    }

    public bool IsExpired(DateTime issuedAt, TimeSpan ttl)
    {
        // Routes through injected TimeProvider — deterministic and testable.
        return _time.GetUtcNow() - issuedAt > ttl;
    }
}

// Unit test:
[Fact]
public void IsExpired_WithPastTimestamp_ReturnsFalse()
{
    var fakeTime = new FakeTimeProvider(new DateTimeOffset(2026, 4, 28, 0, 0, 0, TimeSpan.Zero));
    var service = new MyService(fakeTime);

    var issuedAt = fakeTime.GetUtcNow().DateTime - TimeSpan.FromSeconds(2);
    var ttl = TimeSpan.FromSeconds(5);

    // No Thread.Sleep, no clock-skew flakiness — deterministic.
    Assert.False(service.IsExpired(issuedAt, ttl));

    fakeTime.Advance(TimeSpan.FromSeconds(6)); // Fast-forward
    Assert.True(service.IsExpired(issuedAt, ttl));
}
```

---

## Classification & Severity

The gate classifies violations by scope and context:

| Severity | Scope | Example | Action |
|----------|-------|---------|--------|
| **HIGH** | `src/SDK/`, `src/Bridge/Client/`, `src/Bridge/Protocol/`, all of `src/Runtime/` | NuGet API surface, deadline loops | Migrate to injected `TimeProvider` |
| **HIGH+** | `src/Runtime/Bridge/` deadline loops | `while (DateTime.UtcNow < deadline)` | Migrate to `Stopwatch.GetTimestamp()` or `TimeProvider` |
| **MED** | `src/Tools/`, `src/Domains/` | CLI tools, library code | Migrate to `TimeProvider`; allowlist if cosmetic |
| **LOW** | Files matching `*Logger.cs`, `*Diagnostics.cs`, `*.Debug.cs` | Cosmetic logging timestamps | Safe to leave; tag with `// debug-log timestamp` |

---

## Migration Examples

### Example 1: Cache LoadedAt Timestamp

**Before** (untestable):
```csharp
public sealed class AssetService
{
    public DateTime LoadedAt { get; private set; }

    public void Reload(string path)
    {
        _assets = LoadFromDisk(path);
        LoadedAt = DateTime.UtcNow;  // ← HARD-CODED, untestable
    }
}

[Fact]
public void Reload_UpdatesLoadedAt()
{
    var svc = new AssetService();
    svc.Reload("test.dat");
    // Now what? Can't assert a specific time; can only check ">= now - epsilon"
    Assert.True(svc.LoadedAt > DateTime.UtcNow.AddSeconds(-10));  // ← Flaky!
}
```

**After** (testable):
```csharp
public sealed class AssetService
{
    private readonly TimeProvider _time;

    public DateTime LoadedAt { get; private set; }

    public AssetService(TimeProvider time)
    {
        _time = time;
    }

    public void Reload(string path)
    {
        _assets = LoadFromDisk(path);
        LoadedAt = _time.GetUtcNow().DateTime;  // ← INJECTED, testable
    }
}

[Fact]
public void Reload_UpdatesLoadedAt()
{
    var fakeTime = new FakeTimeProvider(
        new DateTimeOffset(2026, 4, 28, 12, 0, 0, TimeSpan.Zero)
    );
    var svc = new AssetService(fakeTime);

    svc.Reload("test.dat");

    // Deterministic assertion — no flakiness.
    Assert.Equal(
        fakeTime.GetUtcNow().DateTime,
        svc.LoadedAt
    );
}
```

---

### Example 2: Deadline Polling Loop

**Before** (deadline-loop hazard):
```csharp
public sealed class ResourceWaiter
{
    public bool WaitForResource(TimeSpan timeout, Func<bool> isReady)
    {
        var deadline = DateTime.UtcNow.AddSeconds(timeout.TotalSeconds);

        while (DateTime.UtcNow < deadline)  // ← BROKEN if clock steps back
        {
            if (isReady()) return true;
            Thread.Sleep(50);
        }
        return false;
    }
}
```

**After** (monotonic + safe):
```csharp
public sealed class ResourceWaiter
{
    public bool WaitForResource(TimeSpan timeout, Func<bool> isReady)
    {
        var stopwatch = Stopwatch.StartNew();

        while (stopwatch.Elapsed < timeout)  // ← SAFE: monotonic, not wall-clock
        {
            if (isReady()) return true;
            Thread.Sleep(50);
        }
        return false;
    }
}
```

---

### Example 3: Logging Timestamps (Safe to Leave)

**Allowed with tag**:
```csharp
public sealed class GameLauncher
{
    private static readonly string DebugLogPath = "logs/launch.txt";

    public void LogLaunchEvent(string msg)
    {
        // ✓ Cosmetic logging timestamp — no testability impact.
        File.AppendAllText(
            DebugLogPath,
            $"[{DateTime.UtcNow:u}] {msg}\n"  // debug-log timestamp
        );
    }
}
```

Tag the line with the trailing comment `// debug-log timestamp` to suppress the gate.

---

## Gate Configuration

### Allowlist Format

File: `docs/qa/direct-datetime-allowlist.txt`  
One entry per line; `#` for comments.

**Line-locked entry** (suppress a specific site):
```
HIGH|src/Runtime/Bridge/GameBridgeServer.cs|754
```

**File-wide entry** (suppress all sites in a file):
```
src/Tools/Cli/Commands/WatchCommand.cs
```

### Auto-Exemptions (No Allowlist Entry Needed)

- **Test files**: `src/Tests/**/*.cs` (tests are allowed to assert real-clock behavior)
- **Trailing comment**: `// TimeProvider-deferred (netstandard2.0)` (blocks migration)
- **Trailing comment**: `// debug-log timestamp` (cosmetic logging, safe)
- **Type decorator**: `[ExcludeFromCodeCoverage]` (diagnostic glue, exempt)
- **Allowlist file**: Named path suppresses all hits in that file

---

## CI Gate Behavior

```bash
# Run the gate locally:
python scripts/ci/detect_direct_datetime.py [--strict]

# Outputs:
#   - HIGH violations → exit 1 (CI fails)
#   - MED violations → exit 1 (CI fails)
#   - LOW violations → exit 0 (pass) [unless --strict]
#   - Allowlisted sites → not counted
```

**Threshold**: Current baseline = 82. Gate fails on count > 87 (drift gate, +5 tolerance).

---

## Related Issues

- **#285**: TimeProvider injection sweep (production code migration)
- **#300**: Extended Pattern #100 detection to `src/Runtime/` + `src/Tools/`
- **#281** (pending): `detect_unprotected_string_dict.py` (Pattern #99 detection)
- **Future Roslyn analyzer**: Compile-time enforcement of `TimeProvider` injection in NuGet API surfaces

---

## Further Reading

- **Microsoft Docs**: [System.TimeProvider](https://learn.microsoft.com/en-us/dotnet/api/system.timeprovider) (C# 8+)
- **NuGet**.org: [`TimeProvider.Testing`](https://www.nuget.org/packages/TimeProvider.Testing) for `FakeTimeProvider`
- **DINOForge CLAUDE.md**: Pattern #100 governance section
