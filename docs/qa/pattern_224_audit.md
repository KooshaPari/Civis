# Pattern #224 Audit: Undisposed IDisposable Fields

**Date**: 2026-05-18  
**Audit Tool**: `scripts/ci/audit_undisposed_idisposable_fields.py`  
**Tool LOC**: 115 (regex-based, no external deps)  

## Summary

**Total Violations**: 5  
**Tier Classification**: LOW (< 20)  
**Promotion**: Fix as touched (no new catalog entry needed)  

## Violations by Disposable Type

| Type | Count |
|------|-------|
| `HttpClient` | 2 |
| `ManualResetEventSlim` | 1 |
| `SemaphoreSlim` | 2 |

## Directory Heat-map

| Directory | Count |
|-----------|-------|
| `SDK/` | 2 |
| `Tools/` | 2 |
| `Runtime/` | 1 |

## Top 15 Violations (Full List)

| # | File | Line | Class | Field | Type |
|---|------|------|-------|-------|------|
| 1 | `Runtime\Plugin.cs` | 414 | `Plugin` | `_backgroundPollStopEvent` | `ManualResetEventSlim` |
| 2 | `SDK\Registry\PackRegistry.cs` | 168 | `PackRegistryClient` | `_http` | `HttpClient` |
| 3 | `SDK\Registry\PackRegistry.cs` | 174 | `PackRegistryClient` | `_lock` | `SemaphoreSlim` |
| 4 | `Tools\Cli\Assetctl\Sketchfab\SketchfabAdapter.cs` | 26 | `SketchfabAdapter` | `_rateLimitLock` | `SemaphoreSlim` |
| 5 | `Tools\DesktopCompanion\Data\ModCatalogService.cs` | 26 | `ModCatalogService` | `_httpClient` | `HttpClient` |

## Remediation Notes

### High Priority (Immediate)
1. **Runtime\Plugin.cs:414** — `ManualResetEventSlim` is long-lived background thread coordination. Wrap `Plugin` in `IDisposable` + dispose in `OnDestroy()`.
2. **SDK\Registry\PackRegistry.cs:168/174** — `PackRegistryClient` (likely short-lived NuGet client). Implement `IDisposable`, dispose in finalizer + public Dispose method.
3. **Tools\DesktopCompanion\Data\ModCatalogService.cs:26** — Service likely singleton; implement `IAsyncDisposable` and dispose via app shutdown hook.

### Medium Priority
4. **Tools\Cli\Assetctl\Sketchfab\SketchfabAdapter.cs:26** — CLI tool, process-scoped. Implement `IDisposable` + use in `using` statement at main entry point.

## Governance

- No new Pattern Catalog entry needed (LOW tier, 5 violations, isolated to specific service classes).
- Fix in situ during next feature branch or by touching those files.
- Add inline `// idisposable-ok: <reason>` for any intentional exceptions (e.g., shared static HttpClient that outlives class scope).
- Consider `ServiceCollection.AddHttpClient<T>()` in SDK/DI setup for centralized pooling.

## Detection Script

```python
# scripts/ci/audit_undisposed_idisposable_fields.py
# 115 LOC regex-based scanner for undisposed IDisposable fields
# Exempts: MonoBehaviour, ComponentSystemBase, SystemBase, structs, marked fields
# Output: CSV (file, line, class, type, fieldname) + summary
```

---

**Next Audit Cycle**: 2026-06-15 (during Q2 sprint close)
