#nullable enable
namespace DINOForge.Tools.Cli.Assetctl.Sketchfab;

/// <summary>
/// Abstraction for Sketchfab API operations (search, download, quota).
/// Provides a clean boundary for DI and testability.
///
/// Implementations MUST handle:
/// 1. Search: Delegate to SketchfabClient.SearchModelsAsync with filter building
/// 2. Single Download: Delegate to SketchfabClient.DownloadModelAsync with output dir creation
/// 3. Batch Download: Implement SemaphoreSlim-based orchestration, retry logic, and rate limit checking
/// 4. Quota: Cache rate limit state from response headers, parse X-RateLimit-* headers
/// 5. Token Validation: Delegate to SketchfabClient.ValidateTokenAsync
///
/// Gap Fillers (Phase 3):
/// - Gap #1: Batch orchestration with concurrency control, exponential backoff retry (3x), and proactive rate limit checking
/// - Gap #2: Rate limit tracking with header parsing, caching (60s TTL), and pre-download quota checks
/// </summary>
public interface ISketchfabAdapter
{
    /// <summary>
    /// Searches Sketchfab for models matching the query.
    /// </summary>
    /// <param name="query">Search keywords (e.g., "clone trooper")</param>
    /// <param name="limit">Maximum result count (clamped 1-40)</param>
    /// <param name="license">License filter CSV (e.g., "cc0,cc-by")</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>List of model search results</returns>
    /// <exception cref="ArgumentException">Thrown if query is null/empty</exception>
    /// <exception cref="SketchfabApiException">Thrown on API errors (401, 400, 5xx)</exception>
    /// <exception cref="OperationCanceledException">Thrown if cancellation requested</exception>
    Task<IReadOnlyList<SketchfabModelInfo>> SearchAsync(
        string query,
        int limit = 10,
        string? license = null,
        CancellationToken ct = default);

    /// <summary>
    /// Downloads a single model from Sketchfab in the specified format.
    /// </summary>
    /// <remarks>
    /// Automatically creates parent directories for outputPath if needed.
    /// Logs download progress and completion with speed metrics.
    /// </remarks>
    /// <param name="modelId">Sketchfab model UID (non-empty, alphanumeric + dash)</param>
    /// <param name="format">Download format: glb (default), fbx, usdz, zip, etc.</param>
    /// <param name="outputPath">Local file path to save downloaded model (will create parent dirs)</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Download result with file path, size, SHA256 hash, and timing</returns>
    /// <exception cref="ArgumentException">Thrown if modelId or format is null/empty</exception>
    /// <exception cref="InvalidOperationException">Thrown if output directory cannot be created</exception>
    /// <exception cref="SketchfabModelNotFoundException">Thrown if model does not exist (404)</exception>
    /// <exception cref="SketchfabApiException">Thrown on other API errors</exception>
    /// <exception cref="OperationCanceledException">Thrown if cancellation requested</exception>
    Task<SketchfabDownloadResult> DownloadAsync(
        string modelId,
        string format = "glb",
        string? outputPath = null,
        CancellationToken ct = default);

    /// <summary>
    /// Batch downloads multiple models from a manifest file with advanced orchestration.
    ///
    /// Implementation (Gap Filler #1: Batch Orchestration):
    /// 1. Load manifest file (JSON array of {model_id, format?, output_path?})
    /// 2. Create SemaphoreSlim(maxConcurrency, clamped 1-5)
    /// 3. For each model:
    ///    a. Acquire semaphore slot
    ///    b. Check rate limit (GetQuotaAsync) - if remaining <= 5, wait 30s backoff
    ///    c. Download with retry: up to 3 attempts with exponential backoff (1s, 2s, 4s)
    ///    d. Collect results (success/failure)
    ///    e. Report progress callback
    ///    f. Release semaphore
    /// 4. Wait for all tasks (Task.WhenAll)
    /// 5. Return aggregated results (succeeded + failed)
    /// 6. Log batch summary (total, duration, rate limit state)
    /// </summary>
    /// <remarks>
    /// Manifest file format:
    /// [
    ///   {"model_id": "abc123", "format": "glb", "output_path": "/path/to/output.glb"},
    ///   {"model_id": "def456", "format": "glb"}
    /// ]
    ///
    /// - Empty manifests return empty result list (no error)
    /// - Failed downloads do not stop batch (failure collected in results)
    /// - Rate limit checking is performed before each download to avoid 429 responses
    /// - maxConcurrency is clamped to 1-5 to respect API rate limits
    /// </remarks>
    /// <param name="manifestPath">Path to JSON manifest with array of download requests</param>
    /// <param name="maxConcurrency">Maximum concurrent downloads (1-5, default: 1)</param>
    /// <param name="progress">Optional progress callback (current, total)</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>List of download results (one per manifest item)</returns>
    /// <exception cref="ArgumentException">Thrown if manifestPath is null/empty</exception>
    /// <exception cref="FileNotFoundException">Thrown if manifest file does not exist</exception>
    /// <exception cref="InvalidOperationException">Thrown if manifest cannot be deserialized</exception>
    /// <exception cref="OperationCanceledException">Thrown if cancellation requested</exception>
    Task<IReadOnlyList<SketchfabDownloadResult>> DownloadBatchAsync(
        string manifestPath,
        int maxConcurrency = 1,
        Action<int, int>? progress = null,
        CancellationToken ct = default);

    /// <summary>
    /// Gets the current API quota and rate limit state.
    ///
    /// Implementation (Gap Filler #2: Rate Limit Tracking):
    /// 1. Check cached quota if fresh (< 1 minute old), return cached
    /// 2. Otherwise, call SketchfabClient.ValidateTokenAsync to get fresh X-RateLimit-* headers
    /// 3. Parse headers:
    ///    - X-RateLimit-Remaining: requests left in quota
    ///    - X-RateLimit-Reset: Unix timestamp when quota resets
    /// 4. Update cache with parsed values and current timestamp
    /// 5. Return QuotaInfo with remaining, total, reset time, cache timestamp
    ///
    /// Thread-safe caching via SemaphoreSlim lock.
    /// </summary>
    /// <remarks>
    /// Used before batch downloads to check if approaching rate limit.
    /// If remaining <= 5, callers should implement backoff (e.g., 30s delay).
    /// Cache has 60-second TTL to avoid excessive API calls.
    /// </remarks>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Rate limit state (remaining, reset time) or null if not yet fetched</returns>
    /// <exception cref="SketchfabApiException">Thrown on API errors</exception>
    /// <exception cref="OperationCanceledException">Thrown if cancellation requested</exception>
    Task<SketchfabRateLimitState?> GetQuotaAsync(CancellationToken ct = default);

    /// <summary>
    /// Validates the API token by making a test request.
    /// </summary>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Token validation result with plan info and remaining quota</returns>
    /// <exception cref="SketchfabAuthenticationException">Thrown if token is invalid</exception>
    /// <exception cref="SketchfabApiException">Thrown on other API errors</exception>
    /// <exception cref="OperationCanceledException">Thrown if cancellation requested</exception>
    Task<SketchfabTokenValidation> ValidateTokenAsync(CancellationToken ct = default);
}
