# TODO / FIXME / HACK / XXX Inventory — `src/`

**Generated**: 2026-06-10
**Scope**: `src/**/*.cs`
**Pattern**: `\b(TODO|FIXME|HACK|XXX)\b`
**Total matches**: 44 across 14 files

## Subsystem Summary

| Subsystem   | Files | Matches | Notes |
|-------------|-------|---------|-------|
| Analyzers   | 1     | 3       | Marker tokens referenced in analyzer rules/docs (intentional) |
| Runtime     | 5     | 7       | Bridge, native-menu/label/hud adapters; deferred seams |
| SDK         | 3     | 19      | Asset CDN catalog + cache, YAML schema converter — v0.26.0 backlog |
| Domains     | 1     | 1       | UI validator follow-up |
| Tools       | 1     | 8       | Cache CLI command — v0.26.0 backlog |
| Tests       | 4     | 6       | Analyzers-self-tests + LOD fixture content guards |
| **Total**   | **14**| **44**  | |

## Inventory Table

Columns: `file:line` | marker | text (truncated) | action

### Analyzers (`src/Analyzers/`)

| file:line | marker | text | action |
|-----------|--------|------|--------|
| `src/Analyzers/SilentCatchAnalyzer.cs:91` | TODO | `// is annotated solely with a TODO/FIXME/XXX/HACK comment. These are abandoned` | **KEEP** — analyzer rule documentation |
| `src/Analyzers/SilentCatchAnalyzer.cs:93` | TODO | `` // `// test-cleanup-ok` (per HasSafeSwallowComment), not `// TODO`. `` | **KEEP** — analyzer rule documentation |
| `src/Analyzers/SilentCatchAnalyzer.cs:188` | TODO | `// (TODO/FIXME/XXX/HACK). Distinct from `// safe-swallow:` which is the explicit opt-out.` | **KEEP** — analyzer rule documentation |

### Runtime (`src/Bridge/`, `src/Runtime/`)

| file:line | marker | text | action |
|-----------|--------|------|--------|
| `src/Runtime/UI/Adapters/NativeMenuHostAdapter.cs:97` | TODO | `TODO(#NNN-followup-iter145, M11.5 / WI-004a): when NativeMainMenuModMenu.CanUseNativeScreen` | **TRACK** — milestone WI-004a backlog |
| `src/Runtime/UI/Adapters/NativeMenuHostAdapter.cs:109` | TODO | `TODO(#NNN-followup-iter145, M11.5 / WI-004a): mirror Register's wire-up — drop the screen from` | **TRACK** — milestone WI-004a backlog |
| `src/Runtime/UI/Adapters/NativeLabelGuardAdapter.cs:41` | TODO | `TODO(#NNN-followup-iter145): when UiGridHarmonyPatch is refactored for multi-label support,` | **TRACK** — iter145 follow-up, depends on UiGridHarmonyPatch refactor |
| `src/Runtime/UI/Adapters/NativeLabelGuardAdapter.cs:53` | TODO | `TODO(#NNN-followup-iter145): ModsButtonTextPatch is currently single-label, install-once.` | **TRACK** — iter145 follow-up |
| `src/Runtime/UI/Adapters/ModButtonInjectorAdapter.cs:168` | TODO | `TODO(#NNN-followup-iter145, M11.5 / WI-004b): when NativeMenuInjector routes through this seam,` | **TRACK** — milestone WI-004b backlog |
| `src/Runtime/UI/Adapters/HudElementRendererAdapter.cs:181` | TODO | `drain on canvas-ready (see TODO(#NNN-followup-iter145): canvas-ready drain).` | **TRACK** — iter145 follow-up |

### SDK (`src/SDK/`)

| file:line | marker | text | action |
|-----------|--------|------|--------|
| `src/SDK/Validation/YamlSchemaConverter.cs:26` | XXX | `See: https://github.com/aaubry/YamlDotNet/issues/XXX (Windows CLR static init ordering)` | **KEEP** — upstream issue link, not an XXX-marker in the code-fix sense |
| `src/SDK/Assets/AssetCdnCatalog.cs:124` | TODO | `TODO (v0.26.0): Parse YAML manifest file and populate _assets dictionary.` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:132` | TODO | `TODO: Parse YAML, extract pack_id, pack_version, asset_cdn config` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:168` | TODO | `TODO (v0.26.0): Validate URL structure (https://, no path traversal)` | **IMPLEMENT** — v0.26.0 (security-relevant) |
| `src/SDK/Assets/AssetCdnCatalog.cs:185` | TODO | `TODO (v0.26.0): Resolve BepInEx directory from RuntimeContext` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:229` | TODO | `TODO (v0.26.0): Scan dinoforge_pack_cache directory and SQLite assets.db` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:239` | TODO | `TotalCacheBytes = 0L,  // TODO: scan filesystem` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:241` | TODO | `CachedAssetCount = 0,  // TODO: count from assets.db` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:243` | TODO | `EstimatedHitRate = 0.0,  // TODO: calculate from access logs` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCatalog.cs:244` | TODO | `OldestEntryAge = TimeSpan.Zero,  // TODO: from cache metadata` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCache.cs:36` | TODO | `TODO (v0.26.0): Replace with SQLite assets.db for persistent cache metadata` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCache.cs:42` | TODO | `TODO: Move to SQLite assets.db` | **IMPLEMENT** — v0.26.0 (duplicate of :36) |
| `src/SDK/Assets/AssetCdnCache.cs:104` | TODO | `TODO (v0.26.0): Load existing cache metadata from assets.db` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCache.cs:119` | TODO | `TODO (v0.26.0): Implement actual HTTP download using HttpClient.` | **IMPLEMENT** — v0.26.0 (critical path) |
| `src/SDK/Assets/AssetCdnCache.cs:161` | TODO | `TODO (v0.26.0): Verify SHA256 on disk matches expected` | **IMPLEMENT** — v0.26.0 (integrity check) |
| `src/SDK/Assets/AssetCdnCache.cs:174` | TODO | `TODO (v0.26.0): Download from cdnUrl using HttpClient` | **IMPLEMENT** — v0.26.0 (duplicate of :119) |
| `src/SDK/Assets/AssetCdnCache.cs:245` | TODO | `TODO (v0.26.0): Delete cache directory recursively using safe Windows API.` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCache.cs:275` | TODO | `TODO (v0.26.0): Implement full eviction logic.` | **IMPLEMENT** — v0.26.0 |
| `src/SDK/Assets/AssetCdnCache.cs:292` | TODO | `TODO: Implement eviction` | **IMPLEMENT** — v0.26.0 (duplicate of :275) |
| `src/SDK/Assets/AssetCdnCache.cs:321` | TODO | `TODO (v0.26.0): Integrate into download + cache write flow.` | **IMPLEMENT** — v0.26.0 |

### Domains (`src/Domains/`)

| file:line | marker | text | action |
|-----------|--------|------|--------|
| `src/Domains/UI/UIValidator.cs:69` | TODO | `TODO(#844-followup): cross-cutting theme reference checks` | **TRACK** — issue #844 follow-up |

### Tools (`src/Tools/`)

| file:line | marker | text | action |
|-----------|--------|------|--------|
| `src/Tools/Cli/Commands/CacheCommand.cs:82` | TODO | `TODO (v0.26.0): Scan BepInEx/dinoforge_pack_cache directory` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:127` | TODO | `TODO: Read oldest/newest from assets.db` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:179` | TODO | `TODO (v0.26.0): Scan all packs, identify least-recently-used assets` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:212` | TODO | `TODO: Check access time from assets.db; only delete if unused > threshold` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:243` | TODO | `TODO: Read user input and confirm` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:285` | TODO | `TODO (v0.26.0): Implement full prefetch flow` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:341` | TODO | `TODO (v0.26.0): Delete entire BepInEx/dinoforge_pack_cache/{pack_id}/` | **IMPLEMENT** — v0.26.0 |
| `src/Tools/Cli/Commands/CacheCommand.cs:361` | TODO | `TODO: Read user input` | **IMPLEMENT** — v0.26.0 (duplicate of :243) |

### Tests (`src/Tests/`)

| file:line | marker | text | action |
|-----------|--------|------|--------|
| `src/Tests/Analyzers/SilentCatchAnalyzerTests.cs:375` | TODO | `Gap D documentation: `// TODO` placeholder comment inside catch — currently NOT flagged` | **KEEP** — test-fixture documentation, intentional |
| `src/Tests/Analyzers/SilentCatchAnalyzerTests.cs:377` | TODO | `any trivia). Documented here to prevent future regression: a `catch { /* TODO */ }`` | **KEEP** — test-fixture documentation, intentional |
| `src/Tests/Analyzers/SilentCatchAnalyzerTests.cs:390` | TODO | `catch (Exception) { /* TODO: handle */ }` | **KEEP** — test input fixture (asserts analyzer ignores `// TODO` markers) |
| `src/Tests/Analyzers/ImplicitEncodingAnalyzerTests.cs:200` | XXX | `Analyzer is scoped to `File.XXX` calls — unrelated types are exempt.` | **KEEP** — analyzer scope documentation (`XXX` is a wildcard, not a marker) |
| `src/Tests/Phase3BDroidLODTests.cs:151` | TODO | `unitSection.Should().NotContain("TODO", $"{unitId} raw asset reference should be production-like config");` | **KEEP** — assertion string fixture, not a marker |
| `src/Tests/Phase3ACloneInfantryLODTests.cs:217` | TODO | `unitSection.Should().NotContain("TODO", $"{unitId} raw asset reference should be production-like config");` | **KEEP** — assertion string fixture, not a marker |

## Action Legend

- **KEEP** — Token appears intentionally (analyzer rule docs, test fixtures, or `XXX` wildcard/URL). Do not act.
- **TRACK** — Defers real work to a tracked milestone/issue. Convert to a bead if not already tracked.
- **IMPLEMENT** — Concrete deferred work. The bulk of `SDK/Assets/`, `Tools/Cli/Commands/CacheCommand.cs` items target v0.26.0.

## Hot Spots (concentration of deferred work)

1. **`src/SDK/Assets/AssetCdnCatalog.cs`** — 9 TODO lines, v0.26.0 manifest parse / cache stats
2. **`src/SDK/Assets/AssetCdnCache.cs`** — 10 TODO lines, v0.26.0 SQLite + HttpClient + eviction
3. **`src/Tools/Cli/Commands/CacheCommand.cs`** — 8 TODO lines, mirrors SDK cache gaps for v0.26.0
4. **`src/Runtime/UI/Adapters/*`** — 6 TODO lines, iter145 / M11.5 (WI-004a / WI-004b) native-UI seam

The SDK + Tools cache surface is the largest coherent block of deferred work and is consistently tagged `v0.26.0`. The Runtime UI TODOs are individually tracked follow-ups with explicit issue/milestone tags.
