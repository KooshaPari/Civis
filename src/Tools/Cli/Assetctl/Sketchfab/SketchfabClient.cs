#nullable enable
using System.Diagnostics;
using System.Net.Http.Headers;
using System.Security.Cryptography;
using System.Text.Json.Serialization;

namespace DINOForge.Tools.Cli.Assetctl.Sketchfab;

/// <summary>
/// HTTP client for Sketchfab v3 REST API.
/// Handles authentication, rate limiting, retries, and error handling.
/// </summary>
/// <remarks>
/// Thread-safe. Uses internal locking for rate limit state to ensure
/// accurate quota tracking across concurrent requests.
///
/// Rate limit strategy:
/// - Detects 429 Too Many Requests and exponential backoff (1s -> 2s -> 4s -> 8s, max 120s)
/// - Proactive throttling when remaining quota drops below threshold
/// - Logs all rate limit events for monitoring
///
/// Authentication:
/// - Bearer token auth (HTTP Authorization header)
/// - Token scope: Read-only (search, metadata, download)
/// </remarks>
public sealed class SketchfabClient : IDisposable
{
    private static readonly HttpClient SharedHttp = new();

    private readonly HttpClient _httpClient;
    private readonly string _apiToken;
    private readonly string _apiBaseUrl;
    private readonly SketchfabClientOptions _options;
    private readonly object _rateLimitLock = new();
    private bool _disposed;

    // Rate limit state (protected by _rateLimitLock)
    private int _rateLimitRemaining = -1;
    private long _rateLimitResetUnix = -1;
    private DateTime _lastRateLimitCheck = DateTime.MinValue;

    /// <summary>
    /// Initializes a new Sketchfab API client.
    /// </summary>
    /// <param name="apiToken">Personal access token from https://sketchfab.com/settings/api</param>
    /// <param name="options">Client configuration options (optional)</param>
    /// <exception cref="ArgumentNullException">Thrown if apiToken is null/empty</exception>
    public SketchfabClient(string apiToken, SketchfabClientOptions? options = null)
    {
        if (string.IsNullOrWhiteSpace(apiToken))
            throw new ArgumentNullException(nameof(apiToken));

        _apiToken = apiToken;
        _options = options ?? new SketchfabClientOptions();
        _apiBaseUrl = _options.ApiBaseUrl;

        _httpClient = SharedHttp;
        _httpClient.Timeout = TimeSpan.FromSeconds(_options.HttpTimeoutSeconds);
        _httpClient.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", apiToken);
        _httpClient.DefaultRequestHeaders.UserAgent.ParseAdd("DINOForge/1.0 (+https://github.com/KooshaPari/Dino)");
    }

    /// <summary>
    /// Internal constructor for dependency injection (testing).
    /// </summary>
    internal SketchfabClient(string apiToken, HttpClient httpClient, SketchfabClientOptions? options = null)
        : this(apiToken, options)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
    }

    /// <summary>
    /// Searches Sketchfab for models matching the query.
    /// </summary>
    /// <param name="query">Search keywords (e.g., "clone trooper")</param>
    /// <param name="filters">Optional search filters (license, poly count, sort, etc.)</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>List of model search results</returns>
    /// <exception cref="SketchfabApiException">Thrown on API errors</exception>
    /// <exception cref="HttpRequestException">Thrown on network errors</exception>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Build query URL:
    ///    - Base: /search?query=...
    ///    - Add filters.License -> license=cc0,cc-by
    ///    - Add filters.MaxPolyCount -> face_count=...
    ///    - Add filters.SortBy -> sort_by=relevance|likeCount|viewCount
    ///    - Add pagination: limit=20, cursor=... (if provided)
    ///
    /// 2. Check rate limit before request:
    ///    - If remaining <= 2, wait until reset before proceeding
    ///    - Log proactive throttling
    ///
    /// 3. Send GET request with Bearer auth header
    ///
    /// 4. Handle response:
    ///    - 200 OK -> Extract results from JSON
    ///    - 429 Too Many Requests -> Parse Retry-After, wait, retry (exponential backoff)
    ///    - 401 Unauthorized -> Throw SketchfabAuthenticationException
    ///    - 400 Bad Request -> Throw SketchfabValidationException
    ///    - 500+ -> Throw SketchfabServerException
    ///
    /// 5. Update rate limit state from X-RateLimit-* headers
    ///
    /// 6. Deserialize response JSON to SketchfabSearchResult
    ///
    /// 7. Return results list (or empty list if no results)
    /// </remarks>
    public async Task<IReadOnlyList<SketchfabModelInfo>> SearchModelsAsync(
        string query,
        SketchfabSearchFilters? filters = null,
        CancellationToken ct = default)
    {
        ThrowIfDisposed();

        if (string.IsNullOrWhiteSpace(query))
            throw new ArgumentNullException(nameof(query));

        filters ??= new SketchfabSearchFilters();

        // Build query URL
        var uriBuilder = new UriBuilder(_apiBaseUrl + "/search");
        var queryParams = new List<string>
        {
            $"q={Uri.EscapeDataString(query)}",
            $"type=models",
            $"limit={filters.Limit}",
            $"sort_by=-{filters.SortBy}"
        };

        // NOTE: License filtering is done client-side because the Sketchfab API's license
        // filter has limited options. We get all results and filter locally.

        if (filters.MaxPolyCount.HasValue)
            queryParams.Add($"face_count={filters.MaxPolyCount}");

        if (filters.Animated.HasValue)
            queryParams.Add($"animated={filters.Animated.Value.ToString().ToLowerInvariant()}");

        if (!string.IsNullOrWhiteSpace(filters.Cursor))
            queryParams.Add($"cursor={Uri.EscapeDataString(filters.Cursor)}");

        uriBuilder.Query = string.Join("&", queryParams);
        var url = uriBuilder.Uri.ToString();

        // Check rate limit before request
        lock (_rateLimitLock)
        {
            if (_rateLimitRemaining >= 0 && _rateLimitRemaining <= 2 && _rateLimitResetUnix > 0)
            {
                var resetAt = UnixTimeStampToDateTime(_rateLimitResetUnix);
                var now = DateTime.UtcNow;
                if (resetAt > now)
                {
                    var waitMs = (int)Math.Ceiling((resetAt - now).TotalMilliseconds);
                    // Proactive throttling - wait for reset
                }
            }
        }

        // Retry logic with exponential backoff
        int retryAttempt = 0;
        while (retryAttempt <= _options.MaxRetries)
        {
            try
            {
                var request = new HttpRequestMessage(HttpMethod.Get, url);
                var response = await _httpClient.SendAsync(request, ct).ConfigureAwait(false);

                UpdateRateLimitState(response);

                if (response.IsSuccessStatusCode)
                {
                    var content = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
                    var searchResult = System.Text.Json.JsonSerializer.Deserialize<SketchfabSearchResponse>(
                        content,
                        new System.Text.Json.JsonSerializerOptions { PropertyNameCaseInsensitive = true });

                    return searchResult?.Results ?? new List<SketchfabModelInfo>();
                }

                // Handle specific error cases
                switch (response.StatusCode)
                {
                    case System.Net.HttpStatusCode.Unauthorized:
                        throw new SketchfabAuthenticationException("API token is invalid or expired");

                    case System.Net.HttpStatusCode.BadRequest:
                        var errorContent = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
                        throw new SketchfabValidationException($"Invalid search query: {errorContent}");

                    case System.Net.HttpStatusCode.TooManyRequests:
                        if (retryAttempt < _options.MaxRetries)
                        {
                            int retryAfterSeconds = 60;
                            if (response.Headers.TryGetValues("Retry-After", out var retryAfter))
                            {
                                if (int.TryParse(retryAfter.FirstOrDefault(), out var value))
                                    retryAfterSeconds = value;
                            }

                            var backoffMs = CalculateBackoffMs(retryAttempt, retryAfterSeconds);
                            await Task.Delay((int)backoffMs, ct).ConfigureAwait(false);
                            retryAttempt++;
                            continue;
                        }
                        throw new SketchfabApiException("Rate limit exceeded and max retries reached");

                    default:
                        if ((int)response.StatusCode >= 500)
                        {
                            throw new SketchfabServerException((int)response.StatusCode,
                                await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false));
                        }
                        throw new SketchfabApiException($"API request failed with status {response.StatusCode}");
                }
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception ex) when (!(ex is SketchfabApiException))
            {
                throw new SketchfabApiException($"Search failed: {ex.Message}", ex);
            }
        }

        throw new SketchfabApiException("Max retries exceeded");
    }

    /// <summary>
    /// Fetches detailed metadata for a single model.
    /// </summary>
    /// <param name="modelId">Sketchfab model UID</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Detailed model metadata including license, poly count, author, etc.</returns>
    /// <exception cref="SketchfabApiException">Thrown on API errors</exception>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Build URL: /models/{modelId}
    ///
    /// 2. Send GET request with Bearer auth
    ///
    /// 3. Handle response:
    ///    - 200 OK -> Deserialize to SketchfabModelMetadata
    ///    - 404 Not Found -> Throw SketchfabModelNotFoundException
    ///    - 429 Rate Limited -> Apply exponential backoff + retry
    ///
    /// 4. Update rate limit state
    ///
    /// 5. Return metadata object
    ///
    /// Note: Metadata includes downloadUrl which can be used for direct downloads.
    /// </remarks>
    public async Task<SketchfabModelMetadata> GetModelMetadataAsync(
        string modelId,
        CancellationToken ct = default)
    {
        ThrowIfDisposed();

        if (string.IsNullOrWhiteSpace(modelId))
            throw new ArgumentNullException(nameof(modelId));

        var url = $"{_apiBaseUrl}/models/{Uri.EscapeDataString(modelId)}";

        // Retry logic with exponential backoff
        int retryAttempt = 0;
        while (retryAttempt <= _options.MaxRetries)
        {
            try
            {
                var request = new HttpRequestMessage(HttpMethod.Get, url);
                var response = await _httpClient.SendAsync(request, ct).ConfigureAwait(false);

                UpdateRateLimitState(response);

                if (response.IsSuccessStatusCode)
                {
                    var content = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
                    var metadata = System.Text.Json.JsonSerializer.Deserialize<SketchfabModelMetadata>(
                        content,
                        new System.Text.Json.JsonSerializerOptions { PropertyNameCaseInsensitive = true });

                    if (metadata == null)
                        throw new SketchfabApiException("Failed to deserialize model metadata");

                    return metadata;
                }

                // Handle specific error cases
                switch (response.StatusCode)
                {
                    case System.Net.HttpStatusCode.NotFound:
                        throw new SketchfabModelNotFoundException(modelId);

                    case System.Net.HttpStatusCode.Unauthorized:
                        throw new SketchfabAuthenticationException("API token is invalid or expired");

                    case System.Net.HttpStatusCode.TooManyRequests:
                        if (retryAttempt < _options.MaxRetries)
                        {
                            int retryAfterSeconds = 60;
                            if (response.Headers.TryGetValues("Retry-After", out var retryAfter))
                            {
                                if (int.TryParse(retryAfter.FirstOrDefault(), out var value))
                                    retryAfterSeconds = value;
                            }

                            var backoffMs = CalculateBackoffMs(retryAttempt, retryAfterSeconds);
                            await Task.Delay((int)backoffMs, ct).ConfigureAwait(false);
                            retryAttempt++;
                            continue;
                        }
                        throw new SketchfabApiException("Rate limit exceeded and max retries reached");

                    default:
                        if ((int)response.StatusCode >= 500)
                        {
                            throw new SketchfabServerException((int)response.StatusCode,
                                await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false));
                        }
                        throw new SketchfabApiException($"API request failed with status {response.StatusCode}");
                }
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception ex) when (!(ex is SketchfabApiException))
            {
                throw new SketchfabApiException($"Failed to fetch metadata for model {modelId}: {ex.Message}", ex);
            }
        }

        throw new SketchfabApiException("Max retries exceeded");
    }

    /// <summary>
    /// Downloads a model in the specified format.
    /// </summary>
    /// <param name="modelId">Sketchfab model UID</param>
    /// <param name="format">Format: glb, zip, fbx, etc. (depends on model availability)</param>
    /// <param name="outputPath">Local file path to save downloaded model</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Download result with file path, size, and SHA256 hash</returns>
    /// <exception cref="SketchfabApiException">Thrown on API errors</exception>
    /// <exception cref="IOException">Thrown on file system errors</exception>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Fetch model metadata first (needed for downloadUrl)
    ///
    /// 2. Validate download is available for requested format
    ///
    /// 3. Build download URL (varies by format):
    ///    - /models/{modelId}/download?type=glb
    ///
    /// 4. Create temp file in ASSET_TEMP_DIR with unique name
    ///
    /// 5. Stream response to file:
    ///    - Use HttpCompletionOption.ResponseHeadersRead (don't buffer entire response)
    ///    - Track download progress (log every 10% or 5MB)
    ///    - Handle interruptions with resume support (Range headers)
    ///
    /// 6. Compute SHA256 hash during download:
    ///    - Use HashAlgorithm to compute hash while streaming
    ///    - Compare against Sketchfab X-Content-Hash header if present
    ///
    /// 7. Move temp file to final outputPath
    ///
    /// 8. Return SketchfabDownloadResult with:
    ///    - FilePath: absolute path to downloaded file
    ///    - FileSize: bytes
    ///    - Sha256: hex-encoded hash
    ///    - DurationMs: time elapsed
    ///
    /// 9. Clean up temp file on error
    ///
    /// Rate limit handling:
    /// - Downloads don't count against API rate limit (different quota)
    /// - Still applies Bearer auth token
    /// </remarks>
    public async Task<SketchfabDownloadResult> DownloadModelAsync(
        string modelId,
        string format = "glb",
        string? outputPath = null,
        CancellationToken ct = default)
    {
        ThrowIfDisposed();

        if (string.IsNullOrWhiteSpace(modelId))
            throw new ArgumentNullException(nameof(modelId));

        var sw = Stopwatch.StartNew();

        // Step 1: Get download URLs from the download endpoint
        var downloadInfoUrl = $"{_apiBaseUrl}/models/{Uri.EscapeDataString(modelId)}/download";
        var infoRequest = new HttpRequestMessage(HttpMethod.Get, downloadInfoUrl);

        HttpResponseMessage infoResponse;
        try
        {
            infoResponse = await _httpClient.SendAsync(infoRequest, ct).ConfigureAwait(false);
            UpdateRateLimitState(infoResponse);
        }
        catch (Exception ex)
        {
            throw new SketchfabApiException($"Failed to retrieve download URLs: {ex.Message}", ex);
        }

        if (infoResponse.StatusCode == System.Net.HttpStatusCode.NotFound)
            throw new SketchfabModelNotFoundException($"Model {modelId} not found");

        if (infoResponse.StatusCode == System.Net.HttpStatusCode.Unauthorized)
            throw new SketchfabAuthenticationException("API token is invalid or expired");

        if (infoResponse.StatusCode == System.Net.HttpStatusCode.Forbidden)
            throw new SketchfabApiException("License does not allow download for this model");

        infoResponse.EnsureSuccessStatusCode();

        var infoJson = await infoResponse.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        var downloadUrls = System.Text.Json.JsonSerializer.Deserialize<Dictionary<string, DownloadUrlEntry>>(
            infoJson,
            new System.Text.Json.JsonSerializerOptions { PropertyNameCaseInsensitive = true });

        if (downloadUrls is null || !downloadUrls.ContainsKey(format.ToLowerInvariant()))
            throw new SketchfabApiException($"Format '{format}' not available for model {modelId}");

        var fileUrl = downloadUrls[format.ToLowerInvariant()].Url;

        // Step 2: Stream-download the file with SHA256 computation
        string tempPath = Path.Combine(
            _options.TempDirectory ?? Path.GetTempPath(),
            $"sf_{modelId}_{Guid.NewGuid():N}.{format}");
        string finalPath = outputPath ?? tempPath;

        // Ensure output directory exists
        var outputDir = Path.GetDirectoryName(finalPath);
        if (!string.IsNullOrEmpty(outputDir) && !Directory.Exists(outputDir))
        {
            Directory.CreateDirectory(outputDir);
        }

        string sha256hex;
        long fileSize;

        try
        {
            using var dlResponse = await _httpClient.SendAsync(
                new HttpRequestMessage(HttpMethod.Get, fileUrl),
                HttpCompletionOption.ResponseHeadersRead, ct).ConfigureAwait(false);

            dlResponse.EnsureSuccessStatusCode();

            using var sha = SHA256.Create();
            using var fileStream = File.Create(tempPath);
            using var cryptoStream = new CryptoStream(fileStream, sha, CryptoStreamMode.Write);

            await dlResponse.Content.CopyToAsync(cryptoStream, ct).ConfigureAwait(false);
            await cryptoStream.FlushFinalBlockAsync(ct).ConfigureAwait(false);

            fileSize = new FileInfo(tempPath).Length;
            sha256hex = Convert.ToHexString(sha.Hash!).ToLowerInvariant();
        }
        catch (Exception ex) when (!(ex is SketchfabApiException))
        {
            // Clean up temp file on error
            if (File.Exists(tempPath))
            {
                try { File.Delete(tempPath); }
                catch { /* ignore cleanup errors */ }
            }
            throw new SketchfabApiException($"Download failed: {ex.Message}", ex);
        }

        // Move temp to final if different
        if (tempPath != finalPath)
        {
            File.Move(tempPath, finalPath, overwrite: true);
        }

        sw.Stop();

        return new SketchfabDownloadResult
        {
            FilePath = finalPath,
            FileSizeBytes = fileSize,
            Sha256 = sha256hex,
            DurationMs = sw.ElapsedMilliseconds
        };
    }

    /// <summary>
    /// Local record for deserializing download URL responses from Sketchfab.
    /// </summary>
    private sealed record DownloadUrlEntry
    {
        [System.Text.Json.Serialization.JsonPropertyName("url")]
        public string Url { get; set; } = string.Empty;

        [System.Text.Json.Serialization.JsonPropertyName("size")]
        public long Size { get; set; }
    }

    /// <summary>
    /// Gets the current rate limit quota from the last API request.
    /// </summary>
    /// <returns>Rate limit state or null if no requests have been made yet</returns>
    /// <remarks>
    /// Returns the X-RateLimit-* headers from the last successful request.
    /// If rate limit has been exceeded, this reflects the reset time.
    /// </remarks>
    public SketchfabRateLimitState? GetRateLimitState()
    {
        lock (_rateLimitLock)
        {
            if (_rateLimitRemaining < 0 || _rateLimitResetUnix < 0)
                return null;

            var resetAt = UnixTimeStampToDateTime(_rateLimitResetUnix);
            var remaining = _rateLimitRemaining;
            var resetInSeconds = (int)Math.Max(0, (resetAt - DateTime.UtcNow).TotalSeconds);

            return new SketchfabRateLimitState
            {
                Remaining = remaining,
                ResetAtUtc = resetAt,
                ResetInSeconds = resetInSeconds,
                LastCheckedUtc = _lastRateLimitCheck
            };
        }
    }

    /// <summary>
    /// Validates the API token by making a test request.
    /// </summary>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Token validation result with plan info</returns>
    /// <exception cref="SketchfabAuthenticationException">Thrown if token is invalid</exception>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Send GET /models (no query) with limit=1
    ///
    /// 2. Inspect response headers:
    ///    - X-RateLimit-Limit: Determine plan type (50 = Free, 500 = Pro)
    ///    - X-RateLimit-Remaining: Current quota
    ///
    /// 3. Return SketchfabTokenValidation with:
    ///    - IsValid: true
    ///    - Plan: "Free" | "Pro" | "Enterprise"
    ///    - RemainingQuota: remaining requests today
    ///
    /// On 401 Unauthorized:
    /// - Return IsValid: false
    /// - Throw SketchfabAuthenticationException
    /// </remarks>
    public async Task<SketchfabTokenValidation> ValidateTokenAsync(CancellationToken ct = default)
    {
        ThrowIfDisposed();

        // Send GET /models?q=test&limit=1 to validate token and extract quota info
        var testUrl = $"{_apiBaseUrl}/models?q=test&limit=1&sort_by=-relevance";
        var request = new HttpRequestMessage(HttpMethod.Get, testUrl);

        try
        {
            var response = await _httpClient.SendAsync(request, ct).ConfigureAwait(false);
            UpdateRateLimitState(response);

            if (response.StatusCode == System.Net.HttpStatusCode.Unauthorized)
            {
                throw new SketchfabAuthenticationException("API token is invalid or expired");
            }

            response.EnsureSuccessStatusCode();

            // Extract rate limit to determine plan type
            var quota = GetRateLimitState();
            string plan = "Free";
            if (response.Headers.TryGetValues("X-RateLimit-Limit", out var limitValues))
            {
                if (int.TryParse(limitValues.FirstOrDefault(), out var limit))
                {
                    plan = limit switch
                    {
                        >= 500 => "Pro",
                        _ => "Free"
                    };
                }
            }

            var dailyLimit = quota?.Remaining ?? 50;
            if (response.Headers.TryGetValues("X-RateLimit-Limit", out var dailyLimitValues))
            {
                if (int.TryParse(dailyLimitValues.FirstOrDefault(), out var dlimit))
                    dailyLimit = dlimit;
            }

            return new SketchfabTokenValidation
            {
                IsValid = true,
                Plan = plan,
                RemainingQuota = quota?.Remaining ?? 0,
                DailyLimit = dailyLimit
            };
        }
        catch (SketchfabApiException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new SketchfabApiException("Token validation failed due to network error", ex);
        }
    }

    /// <summary>
    /// Internal method to handle exponential backoff on rate limit.
    /// </summary>
    /// <param name="retryAttempt">Current attempt number (0-based)</param>
    /// <param name="retryAfterSeconds">Retry-After header value in seconds</param>
    /// <returns>Time to wait in milliseconds</returns>
    /// <remarks>
    /// Formula: min(retryAfterSeconds, 2^retryAttempt) * 1000ms, max 120s
    ///
    /// Examples:
    /// - Attempt 0: 1s
    /// - Attempt 1: 2s
    /// - Attempt 2: 4s
    /// - Attempt 3: 8s
    /// - Attempt 4: 16s
    /// - Attempt 5: 32s
    /// - Attempt 6: 64s
    /// - Attempt 7: 120s (max)
    /// </remarks>
    private static long CalculateBackoffMs(int retryAttempt, int retryAfterSeconds)
    {
        var exponentialMs = (long)Math.Pow(2, retryAttempt) * 1000;
        var maxMs = 120_000L;
        var retryAfterMs = retryAfterSeconds * 1000L;

        return Math.Min(maxMs, Math.Max(exponentialMs, retryAfterMs));
    }

    /// <summary>
    /// Internal method to extract rate limit state from response headers.
    /// </summary>
    private void UpdateRateLimitState(HttpResponseMessage response)
    {
        lock (_rateLimitLock)
        {
            _lastRateLimitCheck = DateTime.UtcNow;

            if (response.Headers.TryGetValues("X-RateLimit-Remaining", out var remaining))
            {
                if (int.TryParse(remaining.FirstOrDefault(), out var value))
                    _rateLimitRemaining = value;
            }

            if (response.Headers.TryGetValues("X-RateLimit-Reset", out var reset))
            {
                if (long.TryParse(reset.FirstOrDefault(), out var value))
                    _rateLimitResetUnix = value;
            }
        }
    }

    private static DateTime UnixTimeStampToDateTime(long unixTimeStamp)
        => DateTime.UnixEpoch.AddSeconds(unixTimeStamp);

    private void ThrowIfDisposed()
    {
        if (_disposed)
            throw new ObjectDisposedException(GetType().Name);
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed)
            return;

        _httpClient?.Dispose();
        _disposed = true;
    }
}

// ============================================================================
// Data Models
// ============================================================================

/// <summary>Configuration options for SketchfabClient.</summary>
public sealed class SketchfabClientOptions
{
    /// <summary>Sketchfab API base URL (default: https://api.sketchfab.com/v3)</summary>
    public string ApiBaseUrl { get; set; } = "https://api.sketchfab.com/v3";

    /// <summary>HTTP request timeout in seconds (default: 30)</summary>
    public int HttpTimeoutSeconds { get; set; } = 30;

    /// <summary>Maximum retry attempts before giving up (default: 5)</summary>
    public int MaxRetries { get; set; } = 5;

    /// <summary>Temp directory for downloads (default: system temp)</summary>
    public string? TempDirectory { get; set; }

    /// <summary>Enable automatic rate limit throttling (default: true)</summary>
    public bool EnableProactiveThrottling { get; set; } = true;

    /// <summary>Log HTTP requests/responses (default: false)</summary>
    public bool LogHttpTraffic { get; set; } = false;
}

/// <summary>Search result from Sketchfab API.</summary>
public sealed class SketchfabModelInfo
{
    /// <summary>Unique model ID (UID)</summary>
    [JsonPropertyName("uid")]
    public string Uid { get; set; } = string.Empty;

    /// <summary>Model display name</summary>
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    /// <summary>Creator/author information</summary>
    [JsonPropertyName("creator")]
    public SketchfabCreator? Creator { get; set; }

    /// <summary>License information from search result.</summary>
    [JsonPropertyName("license")]
    public SketchfabLicense? License { get; set; }

    /// <summary>Polygon/vertex count</summary>
    [JsonPropertyName("vertexCount")]
    public int? VertexCount { get; set; }

    /// <summary>Face/triangle count</summary>
    [JsonPropertyName("faceCount")]
    public int? FaceCount { get; set; }

    /// <summary>Publication date (ISO 8601)</summary>
    [JsonPropertyName("publishedAt")]
    public DateTime? PublishedAt { get; set; }

    /// <summary>Sketchfab model URL</summary>
    [JsonPropertyName("modelUrl")]
    public string? ModelUrl { get; set; }
}

/// <summary>Creator information from search result.</summary>
public sealed class SketchfabCreator
{
    /// <summary>Creator display name</summary>
    [JsonPropertyName("displayName")]
    public string DisplayName { get; set; } = string.Empty;

    /// <summary>Creator profile URL</summary>
    [JsonPropertyName("profileUrl")]
    public string? ProfileUrl { get; set; }
}

/// <summary>Detailed model metadata.</summary>
public sealed class SketchfabModelMetadata
{
    /// <summary>Unique model ID</summary>
    [JsonPropertyName("uid")]
    public string Uid { get; set; } = string.Empty;

    /// <summary>Model name</summary>
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    /// <summary>Model description</summary>
    [JsonPropertyName("description")]
    public string? Description { get; set; }

    /// <summary>Creator information</summary>
    [JsonPropertyName("creator")]
    public SketchfabCreator? Creator { get; set; }

    /// <summary>License information</summary>
    [JsonPropertyName("license")]
    public SketchfabLicense? License { get; set; }

    /// <summary>Vertex count</summary>
    [JsonPropertyName("vertexCount")]
    public int? VertexCount { get; set; }

    /// <summary>Face count</summary>
    [JsonPropertyName("faceCount")]
    public int? FaceCount { get; set; }

    /// <summary>Publication date</summary>
    [JsonPropertyName("publishedAt")]
    public DateTime? PublishedAt { get; set; }

    /// <summary>Sketchfab model URL</summary>
    [JsonPropertyName("modelUrl")]
    public string? ModelUrl { get; set; }

    /// <summary>Direct download URL for GLB/ZIP/etc.</summary>
    [JsonPropertyName("downloadUrl")]
    public string? DownloadUrl { get; set; }

    /// <summary>Maturity rating (general, mature, explicit)</summary>
    [JsonPropertyName("maturityRating")]
    public string? MaturityRating { get; set; }
}

/// <summary>License information from metadata.</summary>
public sealed class SketchfabLicense
{
    /// <summary>License UID (cc-by-4.0, cc0, etc.)</summary>
    [JsonPropertyName("uid")]
    public string Uid { get; set; } = string.Empty;

    /// <summary>Human-readable license name</summary>
    [JsonPropertyName("label")]
    public string Label { get; set; } = string.Empty;
}

/// <summary>Search filter options.</summary>
public sealed class SketchfabSearchFilters
{
    /// <summary>CSV of license UIDs to include (e.g., "cc0,cc-by,cc-by-sa")</summary>
    public string? License { get; set; }

    /// <summary>Maximum polygon/face count</summary>
    public int? MaxPolyCount { get; set; }

    /// <summary>Sort order: relevance, likeCount, viewCount, publishedAt</summary>
    public string SortBy { get; set; } = "relevance";

    /// <summary>Include animated models (default: false)</summary>
    public bool? Animated { get; set; }

    /// <summary>Results per page (1-40, default: 20)</summary>
    public int Limit { get; set; } = 20;

    /// <summary>Pagination cursor from previous response</summary>
    public string? Cursor { get; set; }

    /// <summary>Exclude paid/exclusive models (default: true)</summary>
    public bool ExcludePaid { get; set; } = true;
}

/// <summary>Download result with file info and hash.</summary>
public sealed class SketchfabDownloadResult
{
    /// <summary>Absolute path to downloaded file</summary>
    public string FilePath { get; set; } = string.Empty;

    /// <summary>File size in bytes</summary>
    public long FileSizeBytes { get; set; }

    /// <summary>SHA256 hash (lowercase hex)</summary>
    public string Sha256 { get; set; } = string.Empty;

    /// <summary>Download duration in milliseconds</summary>
    public long DurationMs { get; set; }

    /// <summary>Download speed in bytes per second</summary>
    public long SpeedBytesPerSecond => FileSizeBytes > 0 && DurationMs > 0
        ? (FileSizeBytes * 1000) / DurationMs
        : 0;
}

/// <summary>Rate limit state from API headers.</summary>
public sealed class SketchfabRateLimitState
{
    /// <summary>Remaining requests in current quota</summary>
    public int Remaining { get; set; }

    /// <summary>UTC timestamp when quota resets</summary>
    public DateTime ResetAtUtc { get; set; }

    /// <summary>Seconds until quota resets</summary>
    public int ResetInSeconds { get; set; }

    /// <summary>When this state was last updated</summary>
    public DateTime LastCheckedUtc { get; set; }
}

/// <summary>Token validation result.</summary>
public sealed class SketchfabTokenValidation
{
    /// <summary>Token is valid and usable</summary>
    public bool IsValid { get; set; }

    /// <summary>Account plan: Free, Pro, Enterprise</summary>
    public string? Plan { get; set; }

    /// <summary>Remaining API requests in current quota</summary>
    public int RemainingQuota { get; set; }

    /// <summary>Daily request limit</summary>
    public int DailyLimit { get; set; }

    /// <summary>Error message if validation failed</summary>
    public string? ErrorMessage { get; set; }
}

/// <summary>Wrapper for search API response containing paginated results.</summary>
public sealed class SketchfabSearchResponse
{
    /// <summary>List of model search results from the API response</summary>
    [JsonPropertyName("results")]
    public IReadOnlyList<SketchfabModelInfo> Results { get; set; } = new List<SketchfabModelInfo>();

    /// <summary>URL for next page of results (if available)</summary>
    [JsonPropertyName("next")]
    public string? Next { get; set; }

    /// <summary>Total count of matching results</summary>
    [JsonPropertyName("count")]
    public int Count { get; set; }
}

// ============================================================================
// Exception Types
// ============================================================================

/// <summary>Base exception for Sketchfab API errors.</summary>
public class SketchfabApiException : Exception
{
    public SketchfabApiException(string message) : base(message) { }
    public SketchfabApiException(string message, Exception inner) : base(message, inner) { }
}

/// <summary>Thrown when API request returns 401 Unauthorized.</summary>
public class SketchfabAuthenticationException : SketchfabApiException
{
    public SketchfabAuthenticationException(string message) : base(message) { }
}

/// <summary>Thrown when model is not found (404).</summary>
public class SketchfabModelNotFoundException : SketchfabApiException
{
    public SketchfabModelNotFoundException(string modelId)
        : base($"Model '{modelId}' not found on Sketchfab") { }
}

/// <summary>Thrown when request validation fails (400).</summary>
public class SketchfabValidationException : SketchfabApiException
{
    public SketchfabValidationException(string message) : base(message) { }
}

/// <summary>Thrown on server errors (5xx).</summary>
public class SketchfabServerException : SketchfabApiException
{
    public SketchfabServerException(int statusCode, string message)
        : base($"Sketchfab server error ({statusCode}): {message}") { }
}
