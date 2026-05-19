# Roslyn Analyzers (Tier 1)

**Status**: 9 Tier 1 analyzers wired into SDK + Bridge + Runtime + Domains projects.  
**Compile-Time Enforcement**: All rules compile-time detected; 1 CodeFix (DF0096) with IDE light-bulb auto-fix.

## Overview

DINOForge uses Roslyn analyzers to enforce critical Pattern Catalog rules at compile time. This eliminates runtime surprises and catches violations before CI/CD. Tier 1 analyzers are high-confidence patterns with minimal false positives.

## Analyzer Table

| ID | Pattern | Title | Severity | Suppression |
|---|---------|-------|----------|-------------|
| **DF0096** | #96 | LogError discards stack trace | **Error** | `#pragma warning disable DF0096` |
| **DF0097** | #97 | TCS sync continuation hazard | **Warning** | `#pragma warning disable DF0097` |
| **DF0099** | #99 | Unprotected `Dictionary<string,T>` | **Warning** | `#pragma warning disable DF0099` |
| **DF0102** | #102 | Process.Start orphan handle | **Error** | `#pragma warning disable DF0102` |
| **DF0111** | #111 | Silent exception swallowing | **Warning** | `#pragma warning disable DF0111` |
| **DF0114** | #114 | CancellationToken not threaded | **Warning** | `#pragma warning disable DF0114` |
| **DF0117** | #117 | StringBuilder no capacity | **Warning** | `#pragma warning disable DF0117` |
| **DF0120** | #120 | JsonDeserialize no options | **Error** | `#pragma warning disable DF0120` |
| **DF0123** | #123 | Public mutable collections | **Warning** | `#pragma warning disable DF0123` |

**CodeFix**: DF0096 has IDE light-bulb auto-fix to upgrade `Debug.LogError(msg)` → `Debug.LogError(ex, msg)`.

---

## DF0096: LogError Discards Stack Trace

**Pattern**: Runtime logging layer loses exception context when `Debug.LogError()` or `LogError()` called without exception parameter.

### What Fires

```csharp
// ❌ FIRES: No stack trace
catch (Exception ex)
{
    Debug.LogError("Operation failed");  // DF0096
}

// ❌ FIRES: Message-only logging
LogError("Something went wrong");  // DF0096 (in Plugin.cs, ModPlatform.cs)
```

### What Doesn't Fire

```csharp
// ✓ OK: Exception passed
catch (Exception ex)
{
    Debug.LogError(ex, "Operation failed");  // OK
}

// ✓ OK: Exception context in message
LogWarning($"Failed: {ex.Message}");  // OK (but Pattern #54 prefers structured logging)
```

### CodeFix

IDE light-bulb appears on hover. Click → auto-converts to exception-aware form:
- `Debug.LogError(msg)` → `Debug.LogError(ex, msg)`
- `LogError(msg)` → `LogError(ex, msg)`

---

## DF0097: TCS Sync Continuation Hazard

**Pattern**: `TaskCompletionSource` sync continuation on UI/synchronous context risks deadlock or stale state.

### What Fires

```csharp
// ❌ FIRES: Sync continuation via .Result
var tcs = new TaskCompletionSource<T>();
var result = tcs.Task.Result;  // DF0097

// ❌ FIRES: ContinueWith no ConfigureAwait
Task.Run(async () => await SomeTask())
    .ContinueWith(t => HandleResult(t.Result));  // DF0097
```

### What Doesn't Fire

```csharp
// ✓ OK: ConfigureAwait(false)
var result = await tcs.Task.ConfigureAwait(false);  // OK

// ✓ OK: MainThreadDispatcher wrapper
MainThreadDispatcher.Instance.Enqueue(() => HandleResult(tcs.Task.Result));  // OK
```

---

## DF0099: Unprotected Dictionary<string, T>

**Pattern**: User-sourced keys in `Dictionary<string, T>` bypass `StringComparer.Ordinal` enforcement, risking case-sensitivity bugs.

### What Fires

```csharp
// ❌ FIRES: Default case-sensitive comparer
var dict = new Dictionary<string, int>();
dict["FactionName"] = 1;  // DF0099 (key case matters)

// ❌ FIRES: Implicit equality
if (dict.ContainsKey(factionInput)) { }  // DF0099 if factionInput is user-sourced
```

### What Doesn't Fire

```csharp
// ✓ OK: StringComparer.Ordinal declared
var dict = new Dictionary<string, int>(StringComparer.Ordinal);
dict["FactionName"] = 1;  // OK

// ✓ OK: StringComparer.OrdinalIgnoreCase (if intentional)
var dict = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);  // OK
```

---

## DF0102: Process.Start Orphan Handle Leakage

**Pattern**: `Process.Start()` result not disposed leaves process handle open, risking resource exhaustion.

### What Fires

```csharp
// ❌ FIRES: No dispose
var p = Process.Start("cmd.exe");
p.WaitForExit();  // DF0102

// ❌ FIRES: Not wrapped in using
Process.Start(new ProcessStartInfo { ... });  // DF0102
```

### What Doesn't Fire

```csharp
// ✓ OK: Using statement
using var p = Process.Start("cmd.exe");
p.WaitForExit();  // OK

// ✓ OK: Explicit dispose
var p = Process.Start("cmd.exe");
try { p.WaitForExit(); }
finally { p.Dispose(); }  // OK
```

---

## DF0111: Silent Exception Swallowing

**Pattern**: Bare `catch {}` or `catch (Exception)` without logging or rethrow erases failure context.

### What Fires

```csharp
// ❌ FIRES: Bare catch
try { await LoadConfig(); }
catch { }  // DF0111

// ❌ FIRES: Exception ignored
try { ValidateInput(data); }
catch (Exception) { }  // DF0111 (not marked // safe-swallow)
```

### What Doesn't Fire

```csharp
// ✓ OK: Logged
try { await LoadConfig(); }
catch (Exception ex) { _logger.LogWarning(ex, "Config load failed"); }  // OK

// ✓ OK: Marked safe-swallow
try { file.Delete(); }
catch { }  // safe-swallow: temp file, tolerate missing  // OK (marked inline)

// ✓ OK: Test cleanup (marked)
try { CleanupTestFixture(); }
catch { }  // test-cleanup-ok  // OK
```

---

## DF0114: CancellationToken Not Threaded

**Pattern**: Async methods accept `CancellationToken` parameter but don't propagate to async operations, breaking cancellation semantics.

### What Fires

```csharp
// ❌ FIRES: CT parameter declared but not used
public async Task<Data> FetchAsync(CancellationToken ct)
{
    var result = await _http.GetAsync(url);  // DF0114: ct ignored
    return result;
}

// ❌ FIRES: CT passed to wrong operation
public async Task<Data> FetchAsync(CancellationToken ct)
{
    return await _http.GetAsync(url, ct: CancellationToken.None);  // DF0114: wrong token
}
```

### What Doesn't Fire

```csharp
// ✓ OK: CT threaded through
public async Task<Data> FetchAsync(CancellationToken ct)
{
    var result = await _http.GetAsync(url, cancellationToken: ct);  // OK
    return result;
}

// ✓ OK: CT used in timeout
public async Task<Data> FetchAsync(CancellationToken ct)
{
    using var cts = CancellationTokenSource.CreateLinkedTokenSource(ct);
    cts.CancelAfter(TimeSpan.FromSeconds(30));
    return await _http.GetAsync(url, cancellationToken: cts.Token);  // OK
}
```

---

## DF0117: StringBuilder No Capacity

**Pattern**: `StringBuilder` without initial capacity forces repeated buffer re-allocation.

### What Fires

```csharp
// ❌ FIRES: No capacity hint
var sb = new StringBuilder();
for (int i = 0; i < 10000; i++)
    sb.Append($"Line {i}: data\n");  // DF0117: re-allocs on every ~16 appends

// ❌ FIRES: Undersized capacity
var sb = new StringBuilder(5);  // DF0117: will re-alloc immediately
sb.Append("This is a much longer string");
```

### What Doesn't Fire

```csharp
// ✓ OK: Sized capacity
var sb = new StringBuilder(16000);  // OK (pre-sized)
for (int i = 0; i < 10000; i++)
    sb.Append($"Line {i}: data\n");

// ✓ OK: Known small strings
var sb = new StringBuilder();
sb.Append("a");
sb.Append("b");  // OK (pattern-match exception for trivial cases)
```

---

## DF0120: JsonDeserialize No Options

**Pattern**: `JsonSerializer.Deserialize()` without explicit `JsonSerializerOptions` uses non-deterministic defaults, risking case-sensitivity and encoding bugs.

### What Fires

```csharp
// ❌ FIRES: Bare deserialize
var config = JsonSerializer.Deserialize<ConfigModel>(json);  // DF0120

// ❌ FIRES: No options
JsonSerializer.Deserialize<PackManifest>(File.ReadAllText(path));  // DF0120
```

### What Doesn't Fire

```csharp
// ✓ OK: Options passed
var config = JsonSerializer.Deserialize<ConfigModel>(
    json,
    CliJsonOptions.Default);  // OK

// ✓ OK: Inline options
var config = JsonSerializer.Deserialize<ConfigModel>(
    json,
    new JsonSerializerOptions { PropertyNameCaseInsensitive = true });  // OK
```

---

## DF0123: Public Mutable Collections

**Pattern**: Public properties returning mutable collections (List, Dictionary, etc.) allow external code to modify internal state.

### What Fires

```csharp
// ❌ FIRES: Public List property
public class FactionRegistry
{
    public List<Faction> Factions { get; set; }  // DF0123
}

// ❌ FIRES: Returned from method
public Dictionary<string, Unit> GetUnits()
{
    return _units;  // DF0123: caller can modify internal dict
}
```

### What Doesn't Fire

```csharp
// ✓ OK: IReadOnlyList exposed
public class FactionRegistry
{
    public IReadOnlyList<Faction> Factions => _factions;  // OK
}

// ✓ OK: Copy returned
public List<Faction> GetFactionsCopy()
{
    return new List<Faction>(_factions);  // OK (copy, not reference)
}

// ✓ OK: Sealed/internal class (not NuGet API)
internal class InternalRegistry
{
    public List<Faction> Factions { get; set; }  // OK (internal)
}
```

---

## Configuration & Enforcement

### Enabling Analyzers

Analyzers are bundled in the **DINOForge.Analyzers** NuGet package and auto-enabled in:
- `src/SDK/` (public API surface)
- `src/Bridge/` (JSON-RPC protocol)
- `src/Runtime/` (BepInEx plugin)
- `src/Domains/` (Warfare, Economy, Scenario, UI plugins)

### Suppression

For deliberate violations, use `#pragma`:

```csharp
#pragma warning disable DF0111
try { TryDeleteTempFile(); }
catch { }  // Known-safe: temp cleanup
#pragma warning restore DF0111
```

Or inline comment:

```csharp
try { TryDeleteTempFile(); }
catch { }  // safe-swallow: temp cleanup (for DF0111 detection script)
```

### CI Enforcement

All Tier 1 analyzers are **build-failure** (`error` severity) or **warning-enforced** (in CI) via `dotnet build -c Release` gate.

---

## Next Steps

Tier 2 analyzers (DF0201+) under development:
- **DF0201**: Async void event handlers
- **DF0202**: Sync-over-async in hot paths
- **DF0203**: Unvalidated enum deserialization

See [Pattern Catalog](/quality/pattern-catalog) for full governance.
