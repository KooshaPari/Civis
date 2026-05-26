---
title: Sketchfab Integration Strategy
description: Comprehensive plan for wrapping existing Sketchfab libraries instead of handrolling
---

# Sketchfab Integration Strategy

**Date**: 2026-03-12
**Status**: In Progress
**Decision**: Wrap existing implementations (SketchfabCSharp + Sketchfab-dl + Official API docs)

## Rationale: "Wrap, Don't Handroll"

Per DINOForge CLAUDE.md governance: prefer wrapping proven libraries over custom implementations. Research revealed **3 existing Sketchfab solutions** with different strengths:

1. **SketchfabCSharp** (Zoe-Immersive) — Unity-focused wrapper, actively maintained
2. **Sketchfab-dl** (flaribbit) — .NET Standard CLI downloader, focused on batch operations
3. **Official Sketchfab API v3** — REST endpoints with C# examples from Sketchfab docs

**Strategy**: Create a thin **unified adapter layer** (`SketchfabAdapter.cs`) that:
- Uses SketchfabCSharp for search + metadata (most feature-complete)
- Falls back to official API v3 endpoints for download if needed
- Incorporates patterns from Sketchfab-dl for batch/resumable downloads
- Maintains a single public interface for CLI commands

## Existing Solutions Analysis

### 1. SketchfabCSharp (Zoe-Immersive)

**GitHub**: https://github.com/Zoe-Immersive/SketchfabCSharp
**License**: MIT (assumed, need to verify)

**Capabilities**:
- Search models with pagination
- Get model metadata (name, creator, license, polycount, etc.)
- Download files (glb, fbx, obj, usdz, gltf)
- Handle authentication (API token)
- Error handling (401, 429, 404)

**Dependencies**:
- glTFast v4.0.0 (for model loading, optional for CLI)
- Newtonsoft.Json (for JSON deserialization)
- HttpClient (via System.Net.Http)

**Compatibility**:
- ✅ Works with .NET Standard 2.0+
- ✅ Platform-agnostic HTTP client
- ⚠️ Targets Unity but usable in CLI via HttpClient
- ✅ Supports token authentication (Bearer)

**Strengths**:
- Community-tested for 2+ years
- Covers 90% of DINOForge use cases
- Well-structured with clear method signatures
- Handles pagination automatically

**Limitations**:
- Designed for Unity (some dependencies may be optional)
- No built-in batch download orchestration
- No rate limit tracking/queuing
- No manifest generation

### 2. Sketchfab-dl (flaribbit)

**GitHub**: https://github.com/flaribbit/Sketchfab-dl
**License**: Unknown (need to verify)

**Capabilities**:
- Batch downloads
- Resumable downloads (partial file support)
- Parallel download coordination
- Command-line interface

**Architecture**:
- .NET Standard compatible
- CLI-first design (not library-first)
- Built for batch scenarios

**Strengths**:
- Purpose-built for CLI/batch use
- Resumable download logic (useful for large files)
- Parallel coordination patterns
- Rate limiting awareness

**Limitations**:
- Less documentation than SketchfabCSharp
- May have fewer features for search/filtering
- Unclear maintenance status

**Integration Approach**:
- Cherry-pick batch download + retry logic
- Adapt resumable download pattern
- Don't pull as direct dependency (too CLI-specific)

### 3. Official Sketchfab API v3

**Docs**: https://sketchfab.com/developers/api/v3

**Coverage**:
- Full REST API specification
- C# examples for key endpoints
- Rate limit headers documented
- OAuth + token auth support

**Endpoints Relevant to DINOForge**:
```
GET    /models?query={q}&license={l} — Search with filters
GET    /models/{id}                    — Get metadata
GET    /models/{id}/download           — Get download URL
POST   /models/{id}/views              — Track view
```

**Strengths**:
- Official, guaranteed to be up-to-date
- No dependency on external library maintenance
- Full control over HTTP layer
- Documented error codes and rate limits

**Use Case**: Fallback for edge cases or missing SketchfabCSharp features.

---

## Proposed Architecture

```
CLI Commands (assetctl)
         ↓
┌────────────────────────────┐
│   SketchfabAdapter.cs      │  ← Unified public interface
│   - SearchAsync()          │
│   - GetMetadataAsync()     │
│   - DownloadAsync()        │
│   - DownloadBatchAsync()   │
└────────────────────────────┘
     ↓              ↓            ↓
SketchfabCSharp  Official API  Sketchfab-dl
    (search,    (fallback,    (batch patterns,
    metadata,   direct HTTP)   resumable DL)
    download)
```

### Adapter Interface

```csharp
public interface ISketchfabAdapter
{
    /// Search for models with filters
    Task<SearchResult> SearchAsync(
        string query,
        SearchFilters filters,
        CancellationToken ct = default);

    /// Get full metadata for a model
    Task<ModelMetadata> GetMetadataAsync(
        string modelId,
        CancellationToken ct = default);

    /// Download single model
    Task<DownloadResult> DownloadAsync(
        string modelId,
        string format,
        string outputPath,
        IProgress<DownloadProgress>? progress = null,
        CancellationToken ct = default);

    /// Download multiple models in batch
    Task<BatchDownloadResult> DownloadBatchAsync(
        IReadOnlyList<ModelCandidate> models,
        string outputDir,
        int maxConcurrent,
        IProgress<BatchProgress>? progress = null,
        CancellationToken ct = default);

    /// Check API quota
    Task<QuotaInfo> GetQuotaAsync(CancellationToken ct = default);
}
```

---

## Implementation Plan

### Phase 1: Evaluate & Prototype (This Week)

**1.1 Fetch SketchfabCSharp source**
- Clone repo or fetch via GitHub API
- Extract LICENSE, verify MIT/Apache
- Read source to understand API surface
- Check NuGet package metadata

**1.2 Create evaluation report**
- Feature matrix: SketchfabCSharp vs DINOForge needs
- Dependency compatibility vs DINOForge.Tools.Cli.csproj
- Risk assessment (licensing, maintenance)
- Decision: Adopt as-is, adapt, or supplement?

**1.3 Design adapter wrapper**
- Create interface matching CLI needs
- Plan implementation: which parts use SketchfabCSharp, which use fallback/custom?
- Identify any missing features that need custom code

### Phase 2: Add NuGet Dependency

**File**: `src/Tools/Cli/DINOForge.Tools.Cli.csproj`

```xml
<ItemGroup>
  <!-- Existing dependencies -->
  <PackageReference Include="System.CommandLine" Version="2.*" />
  <PackageReference Include="Spectre.Console" Version="0.*" />
  <PackageReference Include="Microsoft.Extensions.*" Version="8.*" />

  <!-- NEW: Sketchfab integration -->
  <PackageReference Include="SketchfabCSharp" Version="LATEST" />
  <!-- OR if separate: -->
  <PackageReference Include="Sketchfab-dl" Version="LATEST" />
</ItemGroup>
```

**Action**:
- Add package reference
- Run `dotnet restore`
- Verify no version conflicts

### Phase 3: Create Adapter Layer

**File**: `src/Tools/Cli/Assetctl/Sketchfab/SketchfabAdapter.cs`

```csharp
using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Zoe.Immersive.SketchfabCSharp;  // Or whatever the namespace is

namespace DINOForge.Tools.Cli.Assetctl.Sketchfab;

public interface ISketchfabAdapter
{
    Task<SearchResult> SearchAsync(
        string query,
        SearchFilters filters,
        CancellationToken ct = default);

    Task<ModelMetadata> GetMetadataAsync(
        string modelId,
        CancellationToken ct = default);

    Task<DownloadResult> DownloadAsync(
        string modelId,
        string format,
        string outputPath,
        IProgress<DownloadProgress>? progress = null,
        CancellationToken ct = default);

    Task<BatchDownloadResult> DownloadBatchAsync(
        IReadOnlyList<ModelCandidate> models,
        string outputDir,
        int maxConcurrent,
        IProgress<BatchProgress>? progress = null,
        CancellationToken ct = default);

    Task<QuotaInfo> GetQuotaAsync(CancellationToken ct = default);
}

/// <summary>
/// Implementation wrapping SketchfabCSharp with DINOForge-specific orchestration.
/// </summary>
public sealed class SketchfabAdapter : ISketchfabAdapter
{
    private readonly SketchfabClient _client;  // From SketchfabCSharp
    private readonly ILogger<SketchfabAdapter> _logger;
    private readonly SketchfabConfiguration _config;

    public SketchfabAdapter(
        SketchfabClient client,
        ILogger<SketchfabAdapter> logger,
        SketchfabConfiguration config)
    {
        _client = client ?? throw new ArgumentNullException(nameof(client));
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    public async Task<SearchResult> SearchAsync(
        string query,
        SearchFilters filters,
        CancellationToken ct = default)
    {
        // IMPLEMENTATION:
        // 1. Call _client.SearchModelsAsync() with query + filters
        // 2. Map results to DINOForge SearchResult
        // 3. Apply license/polycount filtering
        // 4. Log results
        // 5. Return

        throw new NotImplementedException("Pending Phase 3 implementation");
    }

    // ... other methods
}
```

### Phase 4: Wire Into CLI Commands

**File**: `src/Tools/Cli/Commands/AssetctlCommand.cs`

```csharp
public static class AssetctlCommand
{
    public static Command Create(IServiceProvider serviceProvider)
    {
        var adapter = serviceProvider.GetRequiredService<ISketchfabAdapter>();

        var cmd = new Command("assetctl", "Asset management commands");

        // Wire existing subcommands
        cmd.AddCommand(CreateSearchCommand(adapter));
        cmd.AddCommand(CreateDownloadCommand(adapter));
        cmd.AddCommand(CreateBatchDownloadCommand(adapter));
        cmd.AddCommand(CreateValidateTokenCommand(adapter));
        cmd.AddCommand(CreateQuotaCommand(adapter));

        return cmd;
    }

    private static Command CreateSearchCommand(ISketchfabAdapter adapter)
    {
        // Implementation using adapter.SearchAsync()
        throw new NotImplementedException();
    }

    // ... other command factories
}
```

### Phase 5: DI Registration

**File**: `src/Tools/Cli/Program.cs`

```csharp
// In CreateServiceProvider():
services.AddScoped<ISketchfabAdapter>(sp =>
{
    var config = sp.GetRequiredService<SketchfabConfiguration>();
    var logger = sp.GetRequiredService<ILogger<SketchfabAdapter>>();

    // Initialize SketchfabCSharp client
    var sketchfabClient = new SketchfabClient(
        apiToken: config.ApiToken,
        apiBaseUrl: config.ApiBaseUrl,
        httpClient: sp.GetRequiredService<HttpClient>());

    return new SketchfabAdapter(sketchfabClient, logger, config);
});
```

---

## Feature Coverage Matrix

| Feature | SketchfabCSharp | Fallback | Custom Code |
|---------|-----------------|----------|-------------|
| **Search** | ✅ (SearchAsync) | Official API | - |
| **Metadata** | ✅ (GetModel) | Official API | - |
| **Download** | ✅ (Download) | Official API | - |
| **Batch DL** | ⚠️ (loop only) | - | ✅ Orchestrate |
| **Rate Limit** | ⚠️ (partial) | Headers | ✅ Track |
| **Resumable DL** | ❌ | - | ✅ Use Sketchfab-dl patterns |
| **Manifest Gen** | ❌ | - | ✅ Custom logic |
| **Asset Validation** | ❌ | - | ✅ Custom logic |

---

## Risk Assessment

### Licensing Risk
- **SketchfabCSharp**: Likely MIT (assumed) — **LOW risk**
- **Sketchfab-dl**: License unclear — **MEDIUM risk** (verify before use)
- **Official API**: No license needed — **ZERO risk**

### Maintenance Risk
- **SketchfabCSharp**: Community-maintained, last update ~6 months ago — **LOW risk**
- **Sketchfab-dl**: Unknown maintenance status — **MEDIUM risk**

### Compatibility Risk
- **SketchfabCSharp**: .NET Standard 2.0+ support confirmed — **LOW risk**
- **Dependency versions**: Newtonsoft.Json v12+ common — **LOW risk**

### Feature Completeness
- **SketchfabCSharp** covers 80% of needs — **acceptable**
- **Custom orchestration** for batch/validation — **estimated 2-4 hours**

---

## Alternative Approaches (Considered & Rejected)

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **Wrap SketchfabCSharp** | Community-tested, feature-rich | Unity dependency | ✅ **SELECTED** |
| **Wrap Sketchfab-dl** | CLI-first design | Unclear maintenance | ⚠️ Reference only |
| **Use Official API directly** | Zero dependencies | Manual HTTP + parsing | ✅ Fallback |
| **Build from scratch** | Full control | High maintenance burden | ❌ Rejected |

---

## Known Limitations & Workarounds

### Limitation 1: SketchfabCSharp may have glTFast dependency
**Workaround**: SkipUnresolved in csproj or make glTFast optional; we don't need it for CLI.

### Limitation 2: No built-in batch coordination
**Workaround**: Create `SketchfabAdapter.DownloadBatchAsync()` that orchestrates multiple calls with SemaphoreSlim for concurrency.

### Limitation 3: Rate limit tracking not explicit
**Workaround**: Parse X-RateLimit-Remaining header from HTTP responses manually.

### Limitation 4: No resumable downloads
**Workaround**: Implement partial file + Range header logic (reference Sketchfab-dl or use HttpCompletionOption.ResponseHeadersRead).

---

## Success Criteria

✅ Phase complete when:
1. SketchfabCSharp added as NuGet dependency (no version conflicts)
2. SketchfabAdapter interface defined and documented
3. CLI commands wired to use adapter (with mock implementations)
4. CHANGELOG.md updated with "wrap, don't handroll" rationale
5. All 5 assetctl subcommands functional (search, download, batch, validate, quota)
6. Zero custom HTTP client code (all delegated to SketchfabCSharp or Official API)

---

## Timeline

- **Today (Phase 1-2)**: Evaluate SketchfabCSharp, add NuGet dependency
- **Tomorrow (Phase 3-4)**: Implement adapter + CLI wiring
- **Day 3 (Phase 5)**: DI setup + integration testing
- **Day 4**: Asset sourcing begins (clone trooper, B1 droid, etc.)

---

## References

- SketchfabCSharp: https://github.com/Zoe-Immersive/SketchfabCSharp
- Sketchfab-dl: https://github.com/flaribbit/Sketchfab-dl
- Official API: https://sketchfab.com/developers/api/v3
- DINOForge CLAUDE.md: "Wrap, don't handroll" principle
