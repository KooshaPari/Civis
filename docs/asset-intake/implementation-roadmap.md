# Sketchfab Asset Intake Implementation Roadmap

## Executive Summary

This document outlines the phased implementation plan for Sketchfab API integration into DINOForge's asset intake pipeline. The implementation is broken into 5 sprints with clear deliverables, test requirements, and agent responsibilities.

**Target Completion**: 4-5 weeks (assuming 1 sprint per week)
**Lead Agent**: Assigned via task system
**Related Docs**:
- `docs/SKETCHFAB_API_SETUP.md` — API token setup and security
- `docs/asset-intake/SKETCHFAB_CLI_COMMANDS.md` — CLI command specifications
- `src/Tools/Cli/Assetctl/Sketchfab/SketchfabClient.cs` — Client interface (pseudocode)
- `src/Tools/Cli/Assetctl/Sketchfab/AssetDownloader.cs` — Orchestrator interface (pseudocode)

---

## Phase 1: Foundation & Dependency Setup (Sprint 1)

### Goals
- Add required NuGet packages
- Set up project structure
- Configure logging/observability
- Write base test fixtures

### Deliverables

#### 1.1 Update `.csproj` Dependencies

**File**: `src/Tools/Cli/DINOForge.Tools.Cli.csproj`

Add packages:
```xml
<PackageReference Include="System.Net.Http" Version="4.3.4" />
<PackageReference Include="System.Text.Json" Version="8.0.0" />
<PackageReference Include="System.IO" Version="4.3.0" />
<PackageReference Include="Serilog" Version="3.0.1" />
<PackageReference Include="Serilog.Sinks.Console" Version="5.0.0" />
<!-- Already have System.CommandLine, Spectre.Console from main CLI -->
```

**Rationale** (per CLAUDE.md "wrap, don't handroll"):
- `System.Net.Http`: Use standard HttpClient (proven, battle-tested)
- `System.Text.Json`: Built-in, no external parser needed for JSON
- `Serilog`: Industry standard logging (DINOForge may already use)

#### 1.2 Create Directory Structure

```bash
src/Tools/Cli/Assetctl/Sketchfab/
├── SketchfabClient.cs              # ✅ Created (pseudocode)
├── AssetDownloader.cs              # ✅ Created (pseudocode)
├── SketchfabHttpHandler.cs         # NEW: Rate limit handling
├── SketchfabLogging.cs             # NEW: Structured logging
└── Extensions/
    ├── SketchfabClientExtensions.cs # NEW: Helper methods
    └── RateLimitExtensions.cs        # NEW: Rate limit utilities
```

#### 1.3 Implement `SketchfabHttpHandler.cs`

**Purpose**: Intercept HTTP responses, extract rate limit headers, implement retry logic

**Pseudocode**:
```csharp
public sealed class SketchfabRateLimitHandler : DelegatingHandler
{
    private readonly SketchfabClientOptions _options;
    private readonly SemaphoreSlim _rateLimitSemaphore;

    public SketchfabRateLimitHandler(SketchfabClientOptions options)
    {
        _options = options;
        _rateLimitSemaphore = new SemaphoreSlim(1, 1);
    }

    protected override async Task<HttpResponseMessage> SendAsync(
        HttpRequestMessage request, CancellationToken ct)
    {
        // 1. Check rate limit state before request
        //    - If remaining <= _options.ThrottleThreshold, wait
        //
        // 2. Send request (delegate to inner handler)
        //
        // 3. Extract X-RateLimit-* headers from response
        //    - Update internal rate limit state
        //
        // 4. Handle 429 Too Many Requests:
        //    - Read Retry-After header
        //    - Wait (exponential backoff)
        //    - Retry (up to MaxRetries)
        //    - Log all retries
        //
        // 5. Return response (caller decides success/failure)

        throw new NotImplementedException();
    }
}
```

#### 1.4 Implement `SketchfabLogging.cs`

**Purpose**: Structured logging for HTTP requests, rate limits, downloads

**Example Usage**:
```csharp
_logger.LogSearchStarted(query, filters);
_logger.LogRateLimitApproaching(remaining: 5, limit: 50);
_logger.LogDownloadProgress(modelId, bytesDownloaded, totalBytes);
_logger.LogDownloadCompleted(modelId, durationMs, sha256);
```

#### 1.5 Create Test Fixtures

**File**: `src/Tests/Cli/Assetctl/Sketchfab/Fixtures/SketchfabResponseFixtures.cs`

Mock API responses for unit testing:
```csharp
public static class SketchfabResponseFixtures
{
    public static string GetSearchResultsMock() => """
    {
      "results": [
        {
          "uid": "a1b2c3d4e5f6",
          "name": "Clone Trooper Phase 1",
          "creator": { "displayName": "test_artist" },
          "license": "cc-by-4.0",
          "vertexCount": 45000,
          "faceCount": 22500,
          "publishedAt": "2024-06-15T10:30:00Z",
          "modelUrl": "https://sketchfab.com/models/a1b2c3d4e5f6"
        }
      ],
      "next": null
    }
    """;

    public static string GetModelMetadataMock(string modelId) => """...""";
    public static string GetRateLimitExceeded() => """...""";
    // ... more fixtures
}
```

**File**: `src/Tests/Cli/Assetctl/Sketchfab/Fixtures/MockHttpMessageHandler.cs`

```csharp
public sealed class MockHttpMessageHandler : HttpMessageHandler
{
    private readonly Dictionary<string, string> _responses = new();

    public void SetResponse(string url, string content, HttpStatusCode status = HttpStatusCode.OK)
    {
        _responses[url] = content;
    }

    protected override Task<HttpResponseMessage> SendAsync(...)
    {
        // Return mock response from _responses dict
        throw new NotImplementedException();
    }
}
```

#### 1.6 Configuration & Constants

**File**: `src/Tools/Cli/Assetctl/Sketchfab/SketchfabConstants.cs`

```csharp
public static class SketchfabConstants
{
    public const string ApiBaseUrl = "https://api.sketchfab.com/v3";
    public const string ApiVersion = "3";

    // Limits per plan
    public const int FreeTierDailyLimit = 50;
    public const int ProTierDailyLimit = 500;

    // Backoff strategy
    public const int MaxRetries = 5;
    public const int InitialBackoffMs = 1000;
    public const int MaxBackoffMs = 120000; // 2 minutes

    // License validation
    public static readonly string[] AllowedLicenses = { "cc0", "cc-by", "cc-by-sa" };

    // etc.
}
```

### Success Criteria (Sprint 1)
- [ ] All dependencies added to `.csproj`
- [ ] Directory structure created
- [ ] `SketchfabHttpHandler` compiles (pseudocode OK, no implementation)
- [ ] Test fixtures created (mock responses)
- [ ] Constants defined
- [ ] All 5+ new test files created (empty, ready for tests)
- [ ] `dotnet build` succeeds with no warnings (except NotImplementedExceptions)

### Testing (Sprint 1)
- Unit tests verify mock HTTP handler behavior
- Test fixtures validated (JSON well-formed)
- Rate limit header parsing tests

---

## Phase 2: Core Client Implementation (Sprint 2)

### Goals
- Implement `SketchfabClient` methods (search, metadata, download)
- Implement rate limiting with exponential backoff
- Implement error handling (401, 404, 429, 5xx)
- Write comprehensive unit tests (mock HTTP)

### Deliverables

#### 2.1 Implement `SketchfabClient.SearchModelsAsync()`

```csharp
public async Task<IReadOnlyList<SketchfabModelInfo>> SearchModelsAsync(
    string query,
    SketchfabSearchFilters? filters = null,
    CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Build query URL: /search?query=...&license=...&face_count=...&limit=...
    // 2. Send GET request
    // 3. Parse response JSON
    // 4. Update rate limit state
    // 5. Return results

    throw new NotImplementedException();
}
```

**Tests** (4-5 test cases):
- [ ] Valid query returns results
- [ ] License filter applied correctly
- [ ] Polycount filter works
- [ ] Empty results handled gracefully
- [ ] Rate limit headers extracted

#### 2.2 Implement `SketchfabClient.GetModelMetadataAsync()`

```csharp
public async Task<SketchfabModelMetadata> GetModelMetadataAsync(
    string modelId,
    CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Build URL: /models/{modelId}
    // 2. Send GET request
    // 3. Parse response JSON
    // 4. Validate required fields
    // 5. Return metadata

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Valid model ID returns metadata
- [ ] 404 Not Found throws SketchfabModelNotFoundException
- [ ] Creator/license info parsed correctly
- [ ] URL validation (downloadUrl format)

#### 2.3 Implement `SketchfabClient.DownloadModelAsync()`

**Most Complex** - implements streaming + hashing:

```csharp
public async Task<SketchfabDownloadResult> DownloadModelAsync(
    string modelId,
    string format = "glb",
    string? outputPath = null,
    CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Fetch metadata (includes downloadUrl)
    // 2. Validate format available
    // 3. Create temp file
    // 4. Stream download with:
    //    - SHA256 hash computation during stream
    //    - Progress logging (every 10% or 5MB)
    //    - Timeout handling
    // 5. Move temp file to final location
    // 6. Return SketchfabDownloadResult

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Mock download completes successfully
- [ ] SHA256 hash computed correctly (compare to known value)
- [ ] File saved to correct path
- [ ] Progress callback invoked
- [ ] Temp file cleanup on error
- [ ] Timeout behavior

#### 2.4 Implement Rate Limiting & Retry Logic

**Key Feature**: Exponential backoff on 429

```csharp
// In SketchfabHttpHandler.SendAsync()
if (response.StatusCode == HttpStatusCode.TooManyRequests)
{
    var retryAfter = ExtractRetryAfter(response);
    for (int attempt = 0; attempt < _options.MaxRetries; attempt++)
    {
        var backoffMs = CalculateBackoffMs(attempt, retryAfter);
        _logger.LogRateLimitBackoff(backoffMs, attempt);
        await Task.Delay(backoffMs, ct);
        response = await base.SendAsync(request, ct);
        if (response.IsSuccessStatusCode) return response;
    }
}
```

**Tests**:
- [ ] 429 triggers backoff
- [ ] Exponential backoff calculation correct
- [ ] Retry-After header respected
- [ ] Max retries limit enforced
- [ ] Logging includes backoff duration

#### 2.5 Implement Error Handling

```csharp
// In SketchfabClient methods
if (response.StatusCode == HttpStatusCode.Unauthorized)
    throw new SketchfabAuthenticationException("Invalid API token");

if (response.StatusCode == HttpStatusCode.NotFound)
    throw new SketchfabModelNotFoundException(modelId);

if (!response.IsSuccessStatusCode)
    throw new SketchfabApiException($"HTTP {response.StatusCode}: {content}");
```

**Tests**:
- [ ] 401 → SketchfabAuthenticationException
- [ ] 404 → SketchfabModelNotFoundException
- [ ] 400 → SketchfabValidationException
- [ ] 5xx → SketchfabServerException
- [ ] Error messages are informative

#### 2.6 Implement `ValidateTokenAsync()`

```csharp
public async Task<SketchfabTokenValidation> ValidateTokenAsync(CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Call GET /models (with limit=1)
    // 2. Check response status:
    //    - 200 OK → token valid
    //    - 401 → token invalid
    // 3. Extract rate limit headers → determine plan
    // 4. Return SketchfabTokenValidation

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Valid token returns IsValid=true
- [ ] Invalid token returns IsValid=false
- [ ] Plan detection (Free vs Pro)
- [ ] Quota extraction

### Success Criteria (Sprint 2)
- [ ] All 4 main methods implemented (not stubbed)
- [ ] 25+ unit tests pass (mock HTTP, no real API calls)
- [ ] All exception types thrown correctly
- [ ] Rate limiting works with exponential backoff
- [ ] Code compiles with no warnings
- [ ] Test coverage >= 80% for SketchfabClient

### Testing (Sprint 2)
```bash
dotnet test src/Tests/Cli/ -k "SketchfabClient" --logger "console;verbosity=detailed"
```

---

## Phase 3: Orchestrator & Batch Operations (Sprint 3)

### Goals
- Implement `AssetDownloader` search and batch operations
- Implement candidate ranking/filtering
- Implement deduplication
- Write integration tests with mocked Sketchfab

### Deliverables

#### 3.1 Implement `AssetDownloader.SearchCandidatesAsync()`

```csharp
public async Task<IReadOnlyList<AssetCandidate>> SearchCandidatesAsync(
    string query,
    AssetSearchCriteria criteria,
    CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Translate criteria to SketchfabSearchFilters
    // 2. Call _sketchfabClient.SearchModelsAsync()
    // 3. Filter results:
    //    - Validate license
    //    - Filter by polycount
    //    - Filter by maturity rating
    // 4. Rank by confidence score:
    //    - license_score = LicenseToScore(license)
    //    - polycount_score = (maxPoly - modelPoly) / maxPoly
    //    - recency_score = 1.0 - (age_days / 365)
    //    - final = (license * 0.5) + (polycount * 0.3) + (recency * 0.2)
    // 5. Sort by confidence_score descending
    // 6. Limit results to criteria.MaxCandidates
    // 7. Return ranked list

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Query returns candidates
- [ ] License filter enforced (block non-allowed)
- [ ] Polycount filter applied
- [ ] Confidence score calculation correct
- [ ] Results ranked by score
- [ ] Max candidates limit respected

#### 3.2 Implement Pagination Support

```csharp
public async IAsyncEnumerable<AssetCandidate> SearchCandidatesPaginatedAsync(
    string query,
    AssetSearchCriteria criteria,
    int pageSize = 20)
{
    // IMPLEMENTATION:
    // 1. Loop through pages using cursor
    // 2. Yield results as they arrive
    // 3. Stop when: no next cursor OR reached MaxCandidates
    // 4. Check rate limit between pages
    // 5. Log page progress

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Yields across multiple pages
- [ ] Stops at MaxCandidates
- [ ] Rate limit checked between pages
- [ ] Cursor pagination works

#### 3.3 Implement `AssetDownloader.DownloadAssetAsync()`

```csharp
public async Task<DownloadAssetResult> DownloadAssetAsync(
    AssetCandidate candidate,
    string outputDir,
    CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Validate candidate (license, modelId)
    // 2. Generate assetId: "{franchise}_{source}_{externalId}"
    // 3. Create directories: {outputDir}/{assetId}/raw
    // 4. Call _sketchfabClient.DownloadModelAsync()
    // 5. Generate asset_manifest.json:
    //    - Load from candidate + metadata
    //    - Set TechnicalStatus = "discovered"
    //    - Set AcquisitionMode = "api"
    //    - Include FileHash, FileSize
    // 6. Generate metadata.json:
    //    - Include full Sketchfab metadata
    //    - Include ranking details
    //    - Include source rules
    // 7. Return DownloadAssetResult

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Manifest generated with all required fields
    - [ ] asset_id, source_url, license_label, author_name
    - [ ] acquired_at_utc, technical_status, ip_status
- [ ] Metadata JSON includes Sketchfab fields
- [ ] SHA256 hash stored in manifest
- [ ] Directory structure created correctly

#### 3.4 Implement `AssetDownloader.DownloadBatchAsync()`

**Most Complex** - manages concurrent downloads:

```csharp
public async Task<IReadOnlyList<BatchDownloadResult>> DownloadBatchAsync(
    IReadOnlyList<AssetCandidate> candidates,
    string outputDir,
    int maxConcurrent = 1,
    IProgress<BatchDownloadProgress>? onProgress = null,
    CancellationToken ct = default)
{
    // IMPLEMENTATION:
    // 1. Create semaphore for concurrency: new SemaphoreSlim(maxConcurrent)
    // 2. Check rate limit before batch
    // 3. For each candidate:
    //    - Acquire semaphore
    //    - Launch DownloadAssetAsync() in Task.Run()
    //    - Report progress via onProgress callback
    //    - Catch exceptions, add to failures
    // 4. Task.WhenAll() for all downloads
    // 5. Generate batch report
    // 6. Log summary
    // 7. Return mixed results

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Concurrent downloads respect maxConcurrent limit
- [ ] Progress callback invoked for each download
- [ ] Failed downloads reported (not thrown)
- [ ] Rate limit checked
- [ ] Summary stats calculated (total time, MB/s)

#### 3.5 Implement Deduplication

```csharp
public IReadOnlyList<AssetCandidate> DeduplicateCandidates(
    IReadOnlyList<AssetCandidate> candidates,
    string existingAssetsDir)
{
    // IMPLEMENTATION:
    // 1. Enumerate {existingAssetsDir}/**/asset_manifest.json
    // 2. Parse manifests, extract ExternalId
    // 3. Build set of existing modelIds
    // 4. Filter candidates: exclude if ModelId in set
    // 5. Log skipped count
    // 6. Return unique candidates

    throw new NotImplementedException();
}
```

**Tests**:
- [ ] Existing assets detected
- [ ] Duplicates excluded
- [ ] Logging includes skipped count

#### 3.6 License Validation Helper

```csharp
private static bool IsLicenseAllowed(string license, string allowedCsv)
{
    // Split allowedCsv, normalize, compare
    var allowed = allowedCsv.Split(',').Select(s => s.Trim().ToLower());
    var normalized = license.ToLower();
    return allowed.Contains(normalized) || allowed.Contains("*");
}
```

**Tests**:
- [ ] CC-0 accepted if in allow list
- [ ] CC-BY accepted if in allow list
- [ ] CC-BY-NC rejected (commercial restriction)
- [ ] Wildcard "*" accepts all

### Success Criteria (Sprint 3)
- [ ] All AssetDownloader methods implemented
- [ ] 20+ integration tests (with mocked Sketchfab)
- [ ] Ranking algorithm produces expected scores
- [ ] Batch download respects concurrency limits
- [ ] Error handling doesn't break batch
- [ ] Test coverage >= 75% for AssetDownloader

### Testing (Sprint 3)
```bash
dotnet test src/Tests/ -k "AssetDownloader" --logger "console;verbosity=detailed"
```

---

## Phase 4: CLI Commands & Integration (Sprint 4)

### Goals
- Implement all 5 CLI commands in `AssetctlCommand.cs`
- Integrate with environment variables (`.env`)
- Implement progress displays (Spectre.Console)
- Implement error reporting

### Deliverables

#### 4.1 Implement Commands in `AssetctlCommand.cs`

Add these 5 methods (following existing pattern):

1. `CreateSearchSketchfabCommand()` — search-sketchfab
2. `CreateDownloadSketchfabCommand()` — download-sketchfab
3. `CreateDownloadBatchSketchfabCommand()` — download-batch-sketchfab
4. `CreateValidateSketchfabTokenCommand()` — validate-sketchfab-token
5. `CreateSketchfabQuotaCommand()` — sketchfab-quota

**Key Implementation Details**:
- Read `.env` via `Environment.GetEnvironmentVariable()`
- Instantiate `SketchfabClient` and `AssetDownloader`
- Handle exceptions and format output (text/JSON)
- Use Spectre.Console for progress bars and tables
- Log all operations

#### 4.2 Environment Variable Binding

**Create**: `src/Tools/Cli/Assetctl/Sketchfab/SketchfabConfig.cs`

```csharp
public sealed class SketchfabConfig
{
    public static SketchfabConfig LoadFromEnvironment()
    {
        var token = Environment.GetEnvironmentVariable("SKETCHFAB_API_TOKEN")
            ?? throw new InvalidOperationException("SKETCHFAB_API_TOKEN not set");

        var baseUrl = Environment.GetEnvironmentVariable("SKETCHFAB_API_BASE_URL")
            ?? "https://api.sketchfab.com/v3";

        var maxConcurrent = int.TryParse(
            Environment.GetEnvironmentVariable("SKETCHFAB_CONCURRENT_REQUESTS"),
            out var value) ? value : 1;

        // ... load other env vars

        return new SketchfabConfig { Token = token, BaseUrl = baseUrl, ... };
    }

    public string Token { get; set; }
    public string BaseUrl { get; set; }
    public int MaxConcurrentRequests { get; set; }
    // ... other properties
}
```

#### 4.3 Implement Progress Display

**Use Spectre.Console** for rich output:

```csharp
// In search-sketchfab command
var table = new Table()
    .Border(TableBorder.Rounded)
    .Title("[bold]Sketchfab Search Results[/]");
table.AddColumn("Name");
table.AddColumn("License");
table.AddColumn("Poly Count");
table.AddColumn("Score");

foreach (var candidate in candidates)
{
    table.AddRow(
        candidate.Name,
        candidate.License ?? "unknown",
        candidate.PolyCount?.ToString("N0") ?? "unknown",
        candidate.ConfidenceScore.ToString("0.00"));
}

AnsiConsole.Write(table);
```

#### 4.4 Implement Progress Bar for Batch

```csharp
// In download-batch-sketchfab command
var progress = AnsiConsole.Progress()
    .Start(ctx =>
    {
        var task = ctx.AddTask("[green]Downloading assets...[/]", maxValue: candidates.Count);

        var onProgress = new Progress<BatchDownloadProgress>(update =>
        {
            task.Value = update.Current;
            task.Description = $"[green]{update.AssetName}[/]";
        });

        var results = await downloader.DownloadBatchAsync(candidates, outputDir, onProgress: onProgress);
    });
```

#### 4.5 Error Handling & Output Formatting

**Pattern** (follow existing assetctl):
```csharp
if (!string.Equals(outputFormat, "json", StringComparison.OrdinalIgnoreCase))
{
    // Text output
    AnsiConsole.Write(new Panel("Error message")
        .BorderStyle(new Style(foreground: ConsoleColor.Red)));
}
else
{
    // JSON output
    Console.WriteLine(JsonSerializer.Serialize(new
    {
        success = false,
        error = "error_code",
        message = "User-friendly message"
    }));
}
```

#### 4.6 Rate Limit Warnings

Display rate limit state after each operation:

```csharp
var rateLimitState = sketchfabClient.GetRateLimitState();
if (rateLimitState?.Remaining < 5)
{
    AnsiConsole.MarkupLine("[red]⚠️  WARNING: Rate limit quota nearly exhausted![/]");
    AnsiConsole.MarkupLine($"[red]Remaining: {rateLimitState.Remaining} requests[/]");
    AnsiConsole.MarkupLine($"[red]Reset at: {rateLimitState.ResetAtUtc:g} UTC[/]");
}
```

#### 4.7 Integration Tests

**File**: `src/Tests/Cli/Commands/AssetctlSketchfabCommandTests.cs`

```csharp
[Fact]
public async Task SearchSketchfabCommand_ValidQuery_ReturnsFormattedResults()
{
    // Mock SketchfabClient
    // Mock AssetDownloader
    // Execute: assetctl search-sketchfab "clone" --limit 5 --format json
    // Assert: JSON output with results
}

[Fact]
public async Task DownloadBatchSketchfabCommand_DisplaysProgress()
{
    // Execute: assetctl download-batch-sketchfab "clone" --limit 3 --max-concurrent 1
    // Assert: Progress bar displayed, final report shown
}

[Fact]
public async Task ValidateSketchfabTokenCommand_InvalidToken_DisplaysError()
{
    // Mock SketchfabClient.ValidateTokenAsync() → throws SketchfabAuthenticationException
    // Execute: assetctl validate-sketchfab-token
    // Assert: Error message displayed clearly
}
```

### Success Criteria (Sprint 4)
- [ ] All 5 commands implemented and wired up
- [ ] Commands parse arguments correctly
- [ ] Environment variables read from `.env`
- [ ] Text output formatted with Spectre.Console
- [ ] JSON output valid and parseable
- [ ] Rate limit warnings displayed
- [ ] Error messages helpful
- [ ] 15+ integration tests pass
- [ ] `assetctl help` shows all commands

### Testing (Sprint 4)
```bash
# Manual testing
dotnet run --project src/Tools/Cli -- assetctl search-sketchfab "clone" --format json

# Unit tests
dotnet test src/Tests/Cli/Commands/ -k "Sketchfab" --logger "console"
```

---

## Phase 5: Documentation, Testing, Release Prep (Sprint 5)

### Goals
- Complete all documentation
- Add comprehensive test coverage
- Performance testing
- Release to stable branch

### Deliverables

#### 5.1 API Documentation

- [ ] Update `docs/SKETCHFAB_API_SETUP.md` with examples
- [ ] Add API response examples
- [ ] Document rate limit strategy
- [ ] Create troubleshooting guide
- [ ] Add security best practices

#### 5.2 Test Coverage

**Target**: >= 80% overall coverage

```bash
# Generate coverage report
dotnet test src/Tests/ /p:CollectCoverage=true /p:CoverageFormat=opencover

# Identify gaps
dotnet reportgenerator -reports:"coverage.opencover.xml" -targetdir:"coverage_report"
```

**Missing Test Areas**:
- [ ] Timeout scenarios
- [ ] Corrupt download recovery
- [ ] Manifest validation edge cases
- [ ] Concurrency stress tests
- [ ] Large batch downloads (>50 models)

#### 5.3 Performance Testing

**Benchmark**: Download 20 models in &lt; 10 minutes

```csharp
[Benchmark]
public async Task DownloadBatch_20Models()
{
    var candidates = GetMock20Candidates();
    var results = await downloader.DownloadBatchAsync(candidates, outputDir);
    Assert.Equal(20, results.Count(r => r.Success));
}
```

#### 5.4 Integration Test with Real API (Optional)

**Only if token provided**:
```bash
# Set token and run real API tests
export SKETCHFAB_API_TOKEN=your_token
dotnet test -k "IntegrationReal" --filter "Category=SketchfabIntegration"
```

**What to test**:
- [ ] Real search returns results
- [ ] Real download succeeds
- [ ] Rate limit handling works
- [ ] Manifest generation correct

#### 5.5 Example Workflows

**Create**: `docs/asset-intake/SKETCHFAB_WORKFLOWS.md`

Include these workflows:

```bash
# Workflow 1: Search for assets
assetctl search-sketchfab "B1 battle droid" \
  --license cc-by,cc0 \
  --max-poly 2000 \
  --limit 10 \
  --format json > candidates.json

# Workflow 2: Download top candidate
assetctl download-sketchfab sketchfab:model_id \
  --output packs/warfare-starwars/assets/raw/ \
  --franchise star_wars

# Workflow 3: Batch download
assetctl download-batch-sketchfab "clone trooper" \
  --output packs/warfare-starwars/assets/raw/ \
  --limit 10 \
  --max-poly 3000 \
  --max-concurrent 1

# Workflow 4: Validate before pipeline
assetctl validate-sketchfab-token --format json
assetctl sketchfab-quota
```

#### 5.6 CHANGELOG & VERSION

**Update**: `CHANGELOG.md`

```markdown
## [1.1.0] - 2026-03-31

### Added
- **Sketchfab API Integration**: Full support for searching, downloading, and ingesting 3D models
  - `assetctl search-sketchfab` — Query Sketchfab with filtering (license, polycount)
  - `assetctl download-sketchfab` — Download individual models with manifest generation
  - `assetctl download-batch-sketchfab` — Batch operations with progress tracking
  - `assetctl validate-sketchfab-token` — Validate API credentials
  - `assetctl sketchfab-quota` — Monitor API rate limit quota
- Sketchfab client with automatic rate limiting and exponential backoff
- Asset deduplication to prevent re-downloading
- Confidence scoring for candidate ranking
- Complete API documentation and troubleshooting guide

### Documentation
- `docs/SKETCHFAB_API_SETUP.md` — Setup guide (OAuth vs token, credentials, quotas)
- `docs/asset-intake/SKETCHFAB_CLI_COMMANDS.md` — Command reference
- `.env.example` — Environment configuration template

### Dependencies
- Added Serilog for structured logging
- (No external API client library; using standard HttpClient)

### Testing
- 100+ new unit & integration tests
- Mock HTTP responses for offline testing
- Rate limit scenario testing
- Batch download concurrency tests
```

**Update**: `VERSION` file
```
1.1.0
```

#### 5.7 README.md Updates

Add to main README:

```markdown
### Asset Intake Pipeline

DINOForge includes an automated asset intake pipeline for discovering and ingesting 3D models from Sketchfab.

```bash
# Search for models
assetctl search-sketchfab "clone trooper" --license cc-by,cc0

# Download batch
assetctl download-batch-sketchfab "star wars infantry" --limit 5 --output packs/warfare-starwars/assets/raw/
```

See `docs/SKETCHFAB_API_SETUP.md` for full setup instructions.
```

#### 5.8 Migration Guide (Future Agents)

**Create**: `docs/asset-intake/MIGRATION_GUIDE.md`

Cover:
- How to move existing manually-downloaded assets to new pipeline
- How to re-intake previously skipped assets
- How to handle license updates (if Sketchfab changes)

### Success Criteria (Sprint 5)
- [ ] All documentation complete and proofread
- [ ] Test coverage >= 80%
- [ ] Performance benchmark passes
- [ ] CHANGELOG updated
- [ ] README updated
- [ ] VERSION incremented
- [ ] No compiler warnings
- [ ] All tests pass (unit + integration)
- [ ] Code reviewed by at least 1 agent

### Testing (Sprint 5)
```bash
# Full test suite
dotnet test src/Tests/ --logger "console;verbosity=detailed"

# Coverage report
dotnet test /p:CollectCoverage=true /p:CoverageFormat=opencover

# Manual E2E
assetctl search-sketchfab "test" --format json | jq
assetctl validate-sketchfab-token --format json | jq
```

---

## Post-Implementation: Maintenance & Extensions

### Monitoring

Agents should set up monitoring for:
- [ ] API token expiration (alert 30 days before)
- [ ] Rate limit quota exhaustion (daily check)
- [ ] Download failures (log to central system)
- [ ] Manifest validation failures

### Future Enhancements (Post-v1.1)

1. **Incremental/Resume Downloads** (v1.2)
   - Store partial download state
   - Resume on network interruption
   - Checksum verification

2. **OAuth Support** (v1.3)
   - Allow user-initiated downloads (not service account)
   - Delegate permissions per user

3. **Advanced Filtering** (v1.2)
   - YAML-based filter rules in `manifests/`
   - Per-pack source policies
   - Blacklist/whitelist creators

4. **Asset Caching** (v1.2)
   - Local SQLite DB of model metadata
   - Avoid duplicate API calls for same query
   - Cache invalidation strategy

5. **License Audit** (v1.3)
   - Periodic validation of downloaded asset licenses
   - Flag if license changed on Sketchfab
   - Generate compliance reports (CSV, PDF)

6. **Integration with Other Sources** (v1.4)
   - BlendSwap API support
   - ModDB integration
   - Generic URL downloader (for direct model links)

---

## Risk Mitigation

### Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Sketchfab API changes | Low | High | Monitor API status, version pinning in docs, test fixtures |
| Rate limit exhaustion | Medium | Medium | Proactive throttling, alert at 20% quota, stagger batch downloads |
| License non-compliance | Medium | High | Validate license on every download, audit tool, human review |
| Network failures during download | Medium | Medium | Implement retry + resume, store partial state |
| Large batch memory usage | Low | Medium | Stream processing, limit concurrent downloads to 2-3 |
| Slow downloads (>100MB models) | Medium | Low | Add timeout handling, progress logging, resume support |

---

## Sprint Schedule

| Sprint | Phase | Duration | Key Deliverable |
|--------|-------|----------|-----------------|
| 1 | Foundation | Week 1 | Dependencies, structure, fixtures |
| 2 | Client | Week 2 | SketchfabClient with rate limiting |
| 3 | Orchestrator | Week 3 | AssetDownloader + batch operations |
| 4 | CLI Commands | Week 4 | All 5 CLI commands + integration |
| 5 | Polish & Release | Week 5 | Tests, docs, CHANGELOG, release |

**Total**: 5 weeks (can be compressed if agents work in parallel)

---

## Agent Responsibilities

### Recommended Team Structure

| Role | Responsibility | Sprint |
|------|-----------------|--------|
| **Infra Agent** | Dependency setup, project structure | Sprint 1 |
| **Client Agent** | SketchfabClient implementation | Sprint 2 |
| **Core Agent** | AssetDownloader implementation | Sprint 3 |
| **CLI Agent** | Command integration | Sprint 4 |
| **QA/Docs Agent** | Testing, documentation, release prep | Sprint 5 |

### Handoff Protocol

Each sprint should produce:
- [ ] Feature branch with prefix `feat/sketchfab-{phase}`
- [ ] PR with test results + coverage report
- [ ] Documentation updates in PR
- [ ] ✅ Code review approval before merge to main

### Communication

- Daily standup: What was done, blockers, next steps
- Weekly sync: Sprint review, sprint planning
- Async: GitHub issues + PR discussions

---

**Status**: Ready for implementation
**Last Updated**: 2026-03-11
**Maintained By**: DINOForge Agents
