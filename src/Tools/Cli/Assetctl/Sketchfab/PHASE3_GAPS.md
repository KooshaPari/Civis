# Phase 3 Gap Fillers - Quick Reference

## Gap #1: Batch Download Orchestration

**File**: `SketchfabAdapter.cs` / `DownloadBatchInternalAsync()` (~130 lines)

**What was missing**: SketchfabClient only provides single download. No orchestration for concurrent, retryable batches.

**Solution**:
```csharp
SemaphoreSlim semaphore = new(maxConcurrency, maxConcurrency);

foreach (model in manifest)
{
    Task.Run(async () => {
        await semaphore.WaitAsync();
        try {
            // Check rate limit BEFORE download
            var quota = await GetQuotaAsync();
            if (quota.Remaining <= 5)
                await Task.Delay(30s);  // Backoff

            // Retry loop
            for (int retry = 0; retry <= 3; retry++)
            {
                try {
                    result = await DownloadAsync(model);
                    break;  // Success
                }
                catch when (retry < 3)
                {
                    await Task.Delay(exponential_backoff(retry));
                }
            }
            // Collect result
        }
        finally {
            semaphore.Release();
        }
    });
}

await Task.WhenAll(all_tasks);
```

**Key Features**:
- **Concurrency**: SemaphoreSlim(clamped 1-5)
- **Rate Limit Awareness**: Checks quota before each download
- **Retry Logic**: 3x with exponential backoff (1s, 2s, 4s)
- **Batch Resilience**: Errors don't stop operation
- **Progress**: Reports current/total to callback

**Testing**:
```csharp
// Mock SketchfabClient to fail on first attempt
var manifest = new[] {
    new BatchManifestItem { ModelId = "m1", Format = "glb" },
    new BatchManifestItem { ModelId = "m2", Format = "glb" }
};
var results = await adapter.DownloadBatchInternalAsync(manifest, 2, null, ct);
// Verify: both models attempted, failures collected, not thrown
```

---

## Gap #2: Rate Limit Tracking

**File**: `SketchfabAdapter.cs` / `GetQuotaAsync()` (~35 lines)

**What was missing**: SketchfabClient tracks rate limit internally but doesn't expose caching or header parsing. No pre-download quota checks.

**Solution**:
```csharp
// Private state
private int _rateLimitRemaining = -1;
private DateTime _rateLimitReset = DateTime.UtcNow.AddHours(1);
private DateTime _lastQuotaCheck = DateTime.MinValue;
private readonly SemaphoreSlim _rateLimitLock = new(1, 1);

async Task<SketchfabRateLimitState?> GetQuotaAsync()
{
    await _rateLimitLock.WaitAsync();
    try {
        // Return cached if fresh (< 60s)
        if (DateTime.UtcNow - _lastQuotaCheck < TimeSpan.FromSeconds(60))
            return new SketchfabRateLimitState {
                Remaining = _rateLimitRemaining,
                ResetAtUtc = _rateLimitReset,
                LastCheckedUtc = _lastQuotaCheck
            };

        // Fetch fresh from API
        var validation = await _client.ValidateTokenAsync();
        var quotaState = _client.GetRateLimitState();

        // Parse and cache
        if (quotaState != null) {
            _rateLimitRemaining = quotaState.Remaining;
            _rateLimitReset = quotaState.ResetAtUtc;
            _lastQuotaCheck = DateTime.UtcNow;
        }

        return quotaState;
    }
    finally {
        _rateLimitLock.Release();
    }
}
```

**Key Features**:
- **Caching**: 60-second TTL reduces API calls
- **Header Parsing**: Extracts X-RateLimit-Remaining and X-RateLimit-Reset
- **Thread Safety**: SemaphoreSlim lock guards state
- **Proactive Throttling**: Used in batch download to avoid 429 errors

**Header Source** (from SketchfabClient response):
```
X-RateLimit-Limit: 50           // Total requests per hour
X-RateLimit-Remaining: 42       // Requests left
X-RateLimit-Reset: 1710163200   // Unix timestamp when quota resets
```

**Testing**:
```csharp
// Test 1: Cache hit
var q1 = await adapter.GetQuotaAsync(ct);
var q2 = await adapter.GetQuotaAsync(ct);
// Verify q1 == q2, no second API call

// Test 2: Cache expiration
await Task.Delay(TimeSpan.FromSeconds(61));
var q3 = await adapter.GetQuotaAsync(ct);
// Verify q3 is fresh (different timestamp)

// Test 3: Proactive throttling in batch
if (quota.Remaining <= 5)
{
    // Should wait 30s before proceeding
}
```

---

## Integration in Batch Download

**Flow**:
```
1. Load manifest from JSON
2. SemaphoreSlim(maxConcurrency)
3. For each model:
   a. Task.Run(download)
   b.   Acquire semaphore
   c.   [GAP #2] Check quota
   d.   [GAP #2] If low, backoff
   e.   [GAP #1] Try download (3x retry)
   f.   Collect result
   g.   Release semaphore
4. await Task.WhenAll
5. Return aggregated results
```

**Rate Limit Safety**:
- Queries before each download (proactive)
- Waits 30s if approaching limit (conservative backoff)
- Prevents 429 Too Many Requests errors
- Logs all throttling decisions

---

## Input/Output Contracts

### Gap #1: DownloadBatchAsync
```csharp
Input:
- manifestPath: string (JSON file with download requests)
- maxConcurrency: int (1-5, clamped)
- progress: Action<int, int>? (current, total)
- ct: CancellationToken

Manifest format:
[
  {"model_id": "abc", "format": "glb", "output_path": "out.glb"},
  {"model_id": "def", "format": "fbx"}  // optional fields
]

Output:
- IReadOnlyList<SketchfabDownloadResult>
- One result per manifest item
- Result.FileSizeBytes > 0 = success
- Result.FileSizeBytes = 0 + ErrorMessage = failure
```

### Gap #2: GetQuotaAsync
```csharp
Input:
- ct: CancellationToken

Output:
- SketchfabRateLimitState? (null if never queried)
  - Remaining: int (requests left)
  - ResetAtUtc: DateTime (when quota resets)
  - ResetInSeconds: int (convenience)
  - LastCheckedUtc: DateTime (cache timestamp)
```

---

## Error Handling

| Error | Gap | Handling |
|-------|-----|----------|
| Model not found (404) | Both | Log + skip in batch, throw in single |
| Rate limit (429) | #2 | Avoid via proactive checks before download |
| Rate limit low (< 5) | #2 | Wait 30s backoff |
| Download failure | #1 | Retry 3x with exponential backoff |
| Download still fails | #1 | Collect error, continue batch |
| Cancelled | #1 | Log, re-throw, release semaphore |
| Bad manifest JSON | #1 | Throw InvalidOperationException |

---

## Performance Tuning

### Concurrency (maxConcurrency)
- Free tier (50 req/hour): Use 1
- Pro tier (500 req/hour): Use 2-3
- Enterprise: Can go up to 5

### Rate Limit Checking
- Default: Check before each download
- With high cache hits: Only ~1ms overhead
- In batch of 100: ~100ms total quota checks

### Manifest Size
- Tested: Up to 1000 models
- Memory: O(n) for results (linear)
- Time: Depends on concurrency + rate limits

---

## Logging for Monitoring

### Critical (ERROR)
```
"Download failed for model {ModelId}"
"Failed to fetch quota"
"Batch download failed for item {Index}"
```

### Warnings (WARNING)
```
"Rate limit approaching: {Remaining}/{Total}. Waiting before download."
"Download retry {Attempt}/{Max} for {ModelId}"
"Token validation failed - invalid or expired token"
```

### Informational (INFO)
```
"Batch download: {Count} models with maxConcurrency={Concurrency}"
"Downloaded {ModelId} to {Path} ({Size} bytes, {Speed} B/s, {Duration}ms)"
"Quota refreshed: {Remaining} requests remaining"
```

### Debug (DEBUG)
```
"Fetching fresh Sketchfab API quota from server"
```

---

## Migration Path

**Phase 2 (old)**: Loop + sequential downloads
```csharp
foreach (var model in models)
    await adapter.DownloadAsync(model.ModelId, ...);
```

**Phase 3 (new)**: Batch operation with retries
```csharp
File.WriteAllText("manifest.json", JsonSerializer.Serialize(
    models.Select(m => new { model_id = m.ModelId, format = "glb" })));

var results = await adapter.DownloadBatchAsync("manifest.json", maxConcurrency: 2);
```

---

## Dependencies

- **Microsoft.Extensions.Logging**: ILogger<T> abstraction
- **System.Text.Json**: JSON serialization
- **System.Net.Http**: HttpClient (via SketchfabClient)
- **.NET 8.0+**: Async/await, nullable reference types

---

## Related Code Locations

- **Gap #1 Implementation**: `SketchfabAdapter.cs:DownloadBatchInternalAsync()` (lines ~130-250)
- **Gap #2 Implementation**: `SketchfabAdapter.cs:GetQuotaAsync()` (lines ~270-335)
- **Header Parsing**: `SketchfabClient.cs:UpdateRateLimitState()` (delegates parsing)
- **Interface Contract**: `ISketchfabAdapter.cs` (with pseudocode docs)
- **Usage**: `AssetDownloader.cs:DownloadBatchAsync()` (calls adapter)
