# DINOForge iter-148 Code Review

**Date**: 2026-05-28  
**Scope**: Top 10 substantive commits from the iter-148 session (2026-05-28)  
**Reviewer**: Claude Sonnet 4.6 (automated analysis)

---

## Summary Table

| File:Line | Finding | Severity |
|-----------|---------|---------|
| `src/Runtime/Telemetry/MetricsCollector.cs:86-98` | Race condition: `MetricEntry` mutable fields mutated via `AddOrUpdate` update delegate — non-atomic read-modify-write on `CounterValue` | HIGH |
| `src/Runtime/Telemetry/MetricsCollector.cs:86-98` | Pattern #111: bare `catch {}` swallows all exceptions in hot-path metric recording | HIGH |
| `src/Runtime/Settings/PackSettingsStore.cs:54` | Wrong base path: `AppDomain.CurrentDomain.BaseDirectory` points to assembly directory, not BepInEx root — settings file ends up in wrong location on deploy | HIGH |
| `src/SDK/Signing/PackVerifier.cs:94` | Pattern #124: `PackVerifier` is a non-sealed public class with no virtual members in NuGet-published `DINOForge.SDK` — violates unsealing governance | HIGH |
| `src/SDK/Signing/PackVerifier.cs:157-191` | Reflection-based RSA key import silently returns `null` on netstandard2.0 — all verification attempts return `UnknownAuthor` or `TamperedSignatureMismatch` on BepInEx Mono; security feature is non-functional at runtime | HIGH |
| `src/Tools/Cli/Commands/PackDiffCommand.cs:80-82` | Path traversal: `packA`/`packB` args from CLI are used directly in `Path.Combine(repoRoot, "packs", packA)` with no canonicalization check — `../../../etc` traverses outside packs/ | HIGH |
| `src/Runtime/Localization/L10n.cs:45` | `_strings.Clear()` in `LoadLocale` runs before I/O succeeds — if locale file missing, dictionary is cleared but `_loadedLocales` check at line 39 prevents reloading en-US on second call, leaving UI with no strings | MED |
| `src/Runtime/Localization/L10n.cs:63` | `string jsonPath = null` — nullable annotation disabled for file (`#nullable enable` absent) causes potential NullRef on `File.ReadAllText(jsonPath)` path even though guarded; compiler won't warn | MED |
| `src/Runtime/Settings/PackSettingsStore.cs:234-239` | Pattern #109: `GetJsonOptions()` allocates a new `JsonSerializerOptions` on every call (called from inside `lock` in hot path `Set()`) — should be `static readonly` | MED |
| `src/Runtime/Conflicts/ConflictResolutionStore.cs:135-151` | Hand-rolled JSON serializer violates CLAUDE.md "wrap, don't handroll" principle — `Newtonsoft.Json` is already a dependency of the Runtime project; no reason to hand-roll | MED |
| `src/Runtime/Updates/UpdateChecker.cs:278` | Pattern #112: `DateTime.UtcNow` used directly for throttle deadline comparison — `TimeProvider` not threaded through, making 24h throttle untestable | MED |
| `src/Runtime/Updates/UpdateChecker.cs:111` | `Uri.EscapeDataString` applied to `repoOwner` and `repoName` — valid GitHub owner names never need escaping but malformed input could construct an unexpected URL path (low risk but cosmetically inconsistent with the safe-URL-validator pattern added in commit `92a3552e`) | LOW |
| `src/Runtime/Bridge/AssetSwapSystem.cs:101-105` | Pattern #111: bare `catch {}` inside `FindBestEntityManager` silently swallows `CalculateEntityCount()` failures; the outer entity-count comparison then uses `bestCount = -1 → 0` fallback silently | LOW |
| `src/Runtime/Bridge/AssetSwapSystem.cs:171` | Pattern #111: `catch { /* best-effort */ }` on telemetry `IncrementCounter` call — acceptable per MetricsCollector design but uses unlabelled catch | LOW |
| `src/SDK/Patching/PackContentBuilder.cs:50` | `__item_0`, `__item_1` sequential fallback IDs for items with no `id` field will collide when two files in the same section both have un-IDed items — `sectionDict.Count` resets per file but the dictionary is shared across files | MED |
| `src/Tests/Integration/SmokeTests.cs:60` | `schemaValidator: null` in `ContentLoader` constructor disables schema validation for smoke tests — catches YAML parse errors but not schema constraint violations | LOW |
| `src/Tools/Cli/Commands/RegistryCommand.cs:257` | Pattern #109: inline `new JsonSerializerOptions { ... }` in `RegistryCommand` — should use `CliJsonOptions.Default` per Pattern #109 governance | MED |
| `src/Runtime/Localization/L10n.cs:174` | O(n) `locales.Contains(locale)` called for every file in the i18n directory — irrelevant for a handful of locales but violates the "no O(n²)" guideline for unbounded iteration; `HashSet` should be used | LOW |

**Totals: 6 HIGH · 7 MED · 5 LOW**

---

## Top 10 Most Concerning Issues (Detailed Analysis)

### 1. `MetricsCollector` Race Condition on `MetricEntry` Mutation (HIGH)

**File**: `src/Runtime/Telemetry/MetricsCollector.cs:86-98`

`ConcurrentDictionary.AddOrUpdate` provides atomic entry-level operations, but the update delegate mutates a shared `MetricEntry` object's mutable properties (`CounterValue++`, `TotalDurationMs += ...`) without locking. Two concurrent `IncrementCounter("x")` calls can both read `existing.CounterValue = 5`, both increment to `6`, and both write `6` — losing an increment. `ConcurrentDictionary` ensures the dictionary structure is safe, but the *values* are plain reference objects with no atomicity guarantee on their fields.

The pattern is also used for `RecordValue` and `RecordDuration`:
```csharp
// Line 90-93 — non-atomic RMW on shared MetricEntry
existing.CounterValue++;   // READ (stale), MODIFY, WRITE — not atomic
return existing;
```

**Fix**: Use `Interlocked.Increment(ref entry._counterValue)` on a `long` backing field, or replace the shared-object mutation with a proper immutable-update pattern (return a NEW `MetricEntry` from the update delegate).

---

### 2. `PackSettingsStore` Writes to Wrong Directory (HIGH)

**File**: `src/Runtime/Settings/PackSettingsStore.cs:54`

```csharp
_settingsPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "dinoforge-pack-settings.json");
```

In BepInEx context `AppDomain.CurrentDomain.BaseDirectory` resolves to the Unity game root directory (e.g. `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`), **not** the BepInEx folder. The file will be written alongside the game executable, not under BepInEx where other DINOForge persistence files live. This is inconsistent with `ProfileManager` (which correctly takes `profilesDir` as a constructor parameter) and `ConflictResolutionStore` (which takes `bepInExDirectory`).

The singleton's default constructor gives no opportunity to inject the correct path before `Load()` is called on line 55 — the path is frozen at construction time with the wrong base.

**Fix**: The singleton factory should accept the BepInEx root path and cache it, or `PackSettingsStore` should follow the same constructor-injection pattern as `ProfileManager`.

---

### 3. `PackVerifier` Verification is Silently Non-Functional on BepInEx Mono (HIGH)

**File**: `src/SDK/Signing/PackVerifier.cs:157-191`

`LoadRsaPublicKey` attempts to call `ImportSubjectPublicKeyInfo` via reflection:
```csharp
var method = rsa.GetType().GetMethod("ImportSubjectPublicKeyInfo", ...);
if (method != null)
    method.Invoke(rsa, new object[] { keyBytes, null });
else
    return null; // netstandard2.0 fallback: return null
```

`ImportSubjectPublicKeyInfo` does not exist on .NET Framework/Mono (the BepInEx runtime). The reflection probe returns `null`, and the method returns `null` (no key loaded). Since `_trustedAuthors` can never be populated via `LoadTrustedKeys` on BepInEx Mono, `Verify()` will always fall through to the `pack.publickey` embedded-key check — which calls `LoadRsaPublicKey` again and gets `null` — then returns `TamperedSignatureMismatch`.

**Effect**: Every signed pack will appear tampered on the actual game runtime even when legitimate. The security feature was likely developed and tested on .NET 8 (Tools tier) but not validated against the netstandard2.0 BepInEx target.

**Fix**: Implement a netstandard2.0-compatible key import path using `RSAParameters` and `FromXmlString` / `ImportParameters`, or add an explicit `#if NETSTANDARD2_0` compile guard that returns `Unsigned` rather than a false `TamperedSignatureMismatch`.

---

### 4. Path Traversal in `PackDiffCommand` (HIGH)

**File**: `src/Tools/Cli/Commands/PackDiffCommand.cs:80-82`

```csharp
var packPathA = Path.Combine(repoRoot, "packs", packA);
var packPathB = Path.Combine(repoRoot, "packs", packB);
if (!Directory.Exists(packPathA))
    throw new InvalidOperationException($"Pack not found: {packPathA}");
```

`packA` and `packB` are raw CLI arguments. `Path.Combine` does **not** sanitize `..` components: `Path.Combine("C:/repo/packs", "../..", "etc")` yields `"C:/repo/packs/../../etc"` which resolves outside the packs directory. An attacker running `dinoforge pack diff ../../secrets/file legit-pack` could read arbitrary files from the filesystem.

The existence check (`Directory.Exists`) does not prevent traversal — it only rejects paths that don't exist, not paths outside `packs/`.

**Fix**:
```csharp
string resolvedA = Path.GetFullPath(packPathA);
string resolvedB = Path.GetFullPath(packPathB);
string packsBase = Path.GetFullPath(Path.Combine(repoRoot, "packs"));
if (!resolvedA.StartsWith(packsBase + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
    throw new SecurityException($"Pack path escapes packs directory: {packA}");
```

---

### 5. `PackVerifier` Violates Pattern #124 (Unsealed Public Class) (HIGH)

**File**: `src/SDK/Signing/PackVerifier.cs:94`

```csharp
public class PackVerifier   // should be: public sealed class PackVerifier
```

`PackVerifier` is a non-sealed public class in `DINOForge.SDK` (NuGet-published). It has no `virtual`/`abstract` members and no documented subclassing contract. Per Pattern #124 governance all new public types in SDK/ should default to `sealed`. This was introduced in commit `4fd2d3d6` — caught by the Roslyn DF1013 analyzer but not fixed at authoring time.

**Fix**: Add `sealed` modifier. Note `PackVerificationResult` (line 47) is the same issue.

---

### 6. `L10n.LoadLocale` Clears State Before Confirming I/O Success (MED)

**File**: `src/Runtime/Localization/L10n.cs:45`

```csharp
public static void LoadLocale(string locale)
{
    if (_loadedLocales.Contains(locale)) { _currentLocale = locale; return; }
    _strings.Clear();   // ← strings wiped HERE
    // ... file search ...
    if (jsonPath == null && locale != "en-US")
    {
        LoadLocale("en-US");   // recursive fallback
        return;
    }
    // ... if jsonPath == null AND locale == "en-US", strings remain empty
```

If `locale == "en-US"` AND the en-US JSON file is absent from both search paths, `_strings` is cleared and never re-populated. All subsequent `T()` calls return the key string as-is (acceptable degraded behavior), but `_currentLocale` is then set to `"en-US"` and `_loadedLocales.Add("en-US")` records it as successfully loaded. A second call to `LoadLocale("en-US")` will short-circuit on the `_loadedLocales.Contains` check and never retry.

The fix is to clear `_strings` only after the file is successfully parsed, or to skip `_loadedLocales.Add` when the file was missing.

---

### 7. `MetricsCollector` Pattern #111: Bare Catch in Hot Path (HIGH)

**File**: `src/Runtime/Telemetry/MetricsCollector.cs:96-98`

```csharp
catch
{
    // Best-effort: never throw on metric recording
}
```

Bare `catch {}` (no exception type, no logging) in `IncrementCounter`, `RecordValue`, and `RecordDuration`. While the best-effort intent is stated, these methods sit in the hot path of `AssetSwapSystem.OnUpdate()` (called every frame). A silent failure — e.g., from `string.Intern` on a garbage key — produces zero diagnostic output. Per Pattern #111, bare catches must carry a `// safe-swallow: <reason>` marker AND the exception should be logged at least to `DebugLog.Write` at low priority.

---

### 8. `PackContentBuilder` Sequential Fallback ID Collision (MED)

**File**: `src/SDK/Patching/PackContentBuilder.cs:219`

```csharp
itemId = $"__item_{sectionDict.Count}";
```

`sectionDict.Count` is the dictionary's current size when this item is added. If two YAML files both contribute items without `id` fields to the same section, the second file's items start counting from whatever `sectionDict.Count` is after the first file, which prevents collision — BUT if the first file puts 3 items in and the second file's first item tries `__item_3`, that slot may already exist from a different file's fourth item if files are processed in certain orders with different item counts.

More critically: if `itemId` collides with a legitimate `id` from another item in the same section (e.g., an item genuinely named `__item_0`), the last writer wins silently. The YAML convention of using `__item_` as auto-generated keys is undocumented and fragile.

---

### 9. `PackSettingsStore.GetJsonOptions()` Pattern #109 Violation (MED)

**File**: `src/Runtime/Settings/PackSettingsStore.cs:234-239`

```csharp
private static JsonSerializerOptions GetJsonOptions()
{
    return new JsonSerializerOptions   // ← allocates on every call
    {
        WriteIndented = true,
        PropertyNameCaseInsensitive = false,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };
}
```

`GetJsonOptions()` is called from `Get<T>()` (inside a `lock`, potentially per-frame), `Set()`, `Load()`, and `Save()`. Each call allocates a new `JsonSerializerOptions` object. `JsonSerializerOptions` initialization is non-trivial (caches converter lookups internally). This is a Pattern #109 violation (inline `JsonSerializerOptions` construction) and a Pattern #121 violation (unnecessary allocation in hot path).

**Fix**:
```csharp
private static readonly JsonSerializerOptions JsonOptions = new JsonSerializerOptions
{
    WriteIndented = true,
    PropertyNameCaseInsensitive = false,
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
};
```

---

### 10. `ConflictResolutionStore` Hand-Rolled JSON Violates "Wrap, Don't Handroll" (MED)

**File**: `src/Runtime/Conflicts/ConflictResolutionStore.cs:135-228`

94 lines of hand-rolled JSON serializer/parser for a flat `{ "key": "value" }` dictionary. The Runtime project already depends on `Newtonsoft.Json` (imported by `Plugin.cs`, `ModPlatform.cs`, `ProfileManager.cs`). The rationale in the comment is:

> *"Minimal hand-rolled JSON serialiser — avoids pulling in System.Text.Json for a netstandard2.0 target"*

But `Newtonsoft.Json` is already present — it is used throughout Runtime. The correct fix is:
```csharp
string json = JsonConvert.SerializeObject(_resolutions, Formatting.None);
// ...
var parsed = JsonConvert.DeserializeObject<Dictionary<string, string>>(json);
```

This is a CLAUDE.md violation ("NEVER handroll what a library already solves"). The hand-rolled parser also misses Unicode escape sequences (`\uXXXX`) which would silently corrupt pack IDs containing non-ASCII characters.

---

## Patterns NOT Followed (Should Be)

### Pattern #109 — Inline `JsonSerializerOptions`
Violated in:
- `src/Runtime/Settings/PackSettingsStore.cs:234` (allocates per call)
- `src/Tools/Cli/Commands/RegistryCommand.cs:257` (inline in method body)

Both should delegate to `SDK.Json.JsonOptions.Default` or a project-local static holder.

### Pattern #112 — Direct `DateTime.UtcNow`
Violated in:
- `src/Runtime/Updates/UpdateChecker.cs:278` (`ShouldCheck` uses `DateTime.UtcNow` for 24h throttle comparison — untestable without mocking)
- `src/Runtime/Telemetry/MetricsCollector.cs:69` (`_lastDumpUtc = DateTime.UtcNow` in field initializer — static init side effect)

### Pattern #124 — Unsealed Public Classes in SDK
Violated in:
- `src/SDK/Signing/PackVerifier.cs:94` (`public class PackVerifier`)
- `src/SDK/Signing/PackVerifier.cs:47` (`public class PackVerificationResult`)

### Pattern #99 — Unprotected `Dictionary<string, T>` Without `StringComparer`
`ConvertYamlMapping` in `PackDiffCommand.cs:308` creates `new Dictionary<string, object?>()` without `StringComparer.Ordinal`. Pack IDs are schema-driven / case-sensitive; the missing comparer creates implicit-default smell.

### Pattern #116 — Sync-over-Async in `PackSettingsStore`
`Save()` (line 209) is a synchronous file write called from `Set()` which is called while holding `_settingsLock`. If called from a UI update thread, this is a blocking disk write under lock — a Pattern #116 concern in the Unity main-thread context.

### "Wrap, Don't Handroll" (CLAUDE.md)
`ConflictResolutionStore.SerializeJson` / `ParseJson` — 94 lines of custom JSON that `Newtonsoft.Json` replaces in two lines.

---

## Refactor Recommendations

1. **`MetricsCollector`**: Replace mutable `MetricEntry` fields with `Interlocked` operations or switch to an immutable-update pattern in the `AddOrUpdate` delegate. Add at least a bare `DebugLog.Write` in the catch blocks.

2. **`PackSettingsStore`**: Follow `ProfileManager`'s pattern — accept `bepInExDirectory` as a constructor parameter. Make `GetJsonOptions()` a `static readonly` field.

3. **`PackVerifier`**: Add a netstandard2.0 fallback key-import path using `RSAParameters` / `FromXmlString`. Seal `PackVerifier` and `PackVerificationResult`. Consider returning `Unsigned` (not `TamperedSignatureMismatch`) when key import is unsupported, to avoid false positives.

4. **`PackDiffCommand`**: Add canonicalization + containment check on `packA`/`packB` args before calling `Directory.Exists`.

5. **`ConflictResolutionStore`**: Replace hand-rolled JSON with `JsonConvert.SerializeObject` / `DeserializeObject`.

6. **`L10n`**: Move `_strings.Clear()` to after successful file parse; only call `_loadedLocales.Add` when load succeeded.

7. **`UpdateChecker`**: Thread `TimeProvider` through `ShouldCheck` for testability.
