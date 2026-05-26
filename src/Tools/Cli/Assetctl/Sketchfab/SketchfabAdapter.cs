#nullable enable
using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;
using DINOForge.Tools.Cli.Json;
using Microsoft.Extensions.Logging;

namespace DINOForge.Tools.Cli.Assetctl.Sketchfab;

/// <summary>
/// Concrete implementation of ISketchfabAdapter.
/// Wraps SketchfabClient and adds higher-level operations including:
/// - Batch download orchestration with concurrency control and retry logic
/// - Rate limit tracking with header parsing and proactive throttling
/// - Comprehensive error handling and logging
/// </summary>
public sealed class SketchfabAdapter : ISketchfabAdapter
{
    private readonly SketchfabClient _client;
    private readonly ILogger<SketchfabAdapter> _logger;
    private readonly TimeProvider _timeProvider;

    // Rate limit tracking (Gap Filler #1: Quota tracking)
    private int _rateLimitRemaining = -1;
    private DateTime _rateLimitReset;
    private DateTime _lastQuotaCheck = DateTime.MinValue;
    private readonly SemaphoreSlim _rateLimitLock = new(1, 1);

    /// <summary>
    /// Initializes a new Sketchfab adapter.
    /// </summary>
    public SketchfabAdapter(SketchfabClient client, ILogger<SketchfabAdapter> logger, TimeProvider? timeProvider = null)
    {
        _client = client ?? throw new ArgumentNullException(nameof(client));
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        _timeProvider = timeProvider ?? TimeProvider.System;
        _rateLimitReset = _timeProvider.GetUtcNow().UtcDateTime.AddHours(1);
    }

    public async Task<IReadOnlyList<SketchfabModelInfo>> SearchAsync(
        string query,
        int limit = 10,
        string? license = null,
        CancellationToken ct = default)
    {
        ValidateInput(query, nameof(query));

        _logger.LogInformation("Searching Sketchfab for: {Query} (limit: {Limit}, license: {License})",
            query, limit, license ?? "any");

        var filters = new SketchfabSearchFilters
        {
            Limit = Math.Min(Math.Max(limit, 1), 40),  // Clamp 1-40
            License = license
        };

        try
        {
            var results = await _client.SearchModelsAsync(query, filters, ct).ConfigureAwait(false);
            _logger.LogInformation("Found {Count} results for query '{Query}'", results.Count, query);
            return results;
        }
        catch (OperationCanceledException)
        {
            _logger.LogWarning("Search cancelled for query '{Query}'", query);
            throw;
        }
        catch (SketchfabApiException ex)
        {
            _logger.LogError(ex, "Search failed for query '{Query}'", query);
            throw;
        }
    }

    public async Task<SketchfabDownloadResult> DownloadAsync(
        string modelId,
        string format = "glb",
        string? outputPath = null,
        CancellationToken ct = default)
    {
        ValidateInput(modelId, nameof(modelId));
        ValidateInput(format, nameof(format));

        _logger.LogInformation("Downloading model {ModelId} in format {Format}", modelId, format);

        outputPath ??= Path.Combine(Path.GetTempPath(), $"sketchfab_{modelId}.{format}");

        // Ensure parent directory exists
        var outputDir = Path.GetDirectoryName(outputPath);
        if (!string.IsNullOrEmpty(outputDir) && !Directory.Exists(outputDir))
        {
            try
            {
                Directory.CreateDirectory(outputDir);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Failed to create output directory {OutputDir}", outputDir);
                throw new InvalidOperationException($"Cannot create output directory: {outputDir}", ex);
            }
        }

        try
        {
            var result = await _client.DownloadModelAsync(modelId, format, outputPath, ct)
                .ConfigureAwait(false);

            _logger.LogInformation(
                "Downloaded {ModelId} to {Path} ({Size} bytes, {Speed} B/s, {Duration}ms)",
                modelId, result.FilePath, result.FileSizeBytes, result.SpeedBytesPerSecond, result.DurationMs);

            return result;
        }
        catch (OperationCanceledException)
        {
            _logger.LogWarning("Download cancelled for model {ModelId}", modelId);
            throw;
        }
        catch (SketchfabModelNotFoundException ex)
        {
            _logger.LogError(ex, "Model not found: {ModelId}", modelId);
            throw;
        }
        catch (SketchfabApiException ex)
        {
            _logger.LogError(ex, "Download failed for model {ModelId}", modelId);
            throw;
        }
    }

    public async Task<IReadOnlyList<SketchfabDownloadResult>> DownloadBatchAsync(
        string manifestPath,
        int maxConcurrency = 1,
        Action<int, int>? progress = null,
        CancellationToken ct = default)
    {
        ValidateInput(manifestPath, nameof(manifestPath));

        if (!File.Exists(manifestPath))
            throw new FileNotFoundException($"Manifest not found: {manifestPath}");

        _logger.LogInformation("Reading batch manifest from {ManifestPath}", manifestPath);

        try
        {
            var json = await File.ReadAllTextAsync(manifestPath, ct).ConfigureAwait(false);
            var manifest = JsonSerializer.Deserialize<BatchManifestItem[]>(json, CliJsonOptions.AssetManifest)
                ?? throw new InvalidOperationException("Failed to deserialize manifest");

            if (manifest.Length == 0)
            {
                _logger.LogWarning("Batch manifest is empty");
                return new List<SketchfabDownloadResult>();
            }

            return await DownloadBatchInternalAsync(manifest, maxConcurrency, progress, ct)
                .ConfigureAwait(false);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Batch download failed: {ManifestPath}", manifestPath);
            throw;
        }
    }

    /// <summary>
    /// Internal batch download with advanced orchestration and rate limiting.
    /// Gap Filler #1: Batch orchestration with SemaphoreSlim, retry logic, and rate limit checking.
    /// </summary>
    private async Task<IReadOnlyList<SketchfabDownloadResult>> DownloadBatchInternalAsync(
        BatchManifestItem[] manifest,
        int maxConcurrency,
        Action<int, int>? progress,
        CancellationToken ct)
    {
        maxConcurrency = Math.Max(1, Math.Min(maxConcurrency, 5)); // Clamp 1-5

        _logger.LogInformation(
            "Batch download: {Count} models with maxConcurrency={Concurrency}",
            manifest.Length, maxConcurrency);

        var semaphore = new SemaphoreSlim(maxConcurrency, maxConcurrency);
        var results = new List<SketchfabDownloadResult>();
        var resultsLock = new object();
        var tasks = new List<Task>();
        var sw = Stopwatch.StartNew();

        for (int i = 0; i < manifest.Length; i++)
        {
            var item = manifest[i];
            var index = i;

            var task = Task.Run(async () =>
            {
                await semaphore.WaitAsync(ct).ConfigureAwait(false);
                try
                {
                    // Gap Filler: Check rate limit before download
                    var quota = await GetQuotaAsync(ct).ConfigureAwait(false);
                    if (quota != null && quota.Remaining <= 5)
                    {
                        _logger.LogWarning(
                            "Rate limit approaching: {Remaining}/{Total}. Waiting before download.",
                            quota.Remaining, quota.ResetAtUtc);

                        // Exponential backoff: wait 30 seconds then retry
                        await Task.Delay(TimeSpan.FromSeconds(30), ct).ConfigureAwait(false);
                    }

                    // Attempt download with retry logic
                    SketchfabDownloadResult? result = null;
                    Exception? lastException = null;
                    const int maxRetries = 3;

                    for (int retry = 0; retry <= maxRetries; retry++)
                    {
                        try
                        {
                            result = await DownloadAsync(
                                item.ModelId,
                                item.Format ?? "glb",
                                item.OutputPath,
                                ct).ConfigureAwait(false);

                            break; // Success, exit retry loop
                        }
                        catch (SketchfabApiException ex) when (retry < maxRetries)
                        {
                            lastException = ex;
                            var backoffMs = (long)Math.Pow(2, retry) * 1000; // 1s, 2s, 4s
                            _logger.LogWarning(
                                "Download retry {Attempt}/{Max} for {ModelId} after {Backoff}ms: {Message}",
                                retry + 1, maxRetries, item.ModelId, backoffMs, ex.Message);

                            await Task.Delay((int)backoffMs, ct).ConfigureAwait(false);
                        }
                    }

                    if (result != null)
                    {
                        lock (resultsLock)
                        {
                            results.Add(result);
                        }
                        progress?.Invoke(index + 1, manifest.Length);
                    }
                    else if (lastException != null)
                    {
                        throw new SketchfabApiException(
                            $"Failed to download {item.ModelId} after {maxRetries} retries: {lastException.Message}",
                            lastException);
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Batch download failed for item {Index}: {ModelId}", index, item.ModelId);
                    // Don't throw - collect error and continue with other items
                    lock (resultsLock)
                    {
                        results.Add(new SketchfabDownloadResult
                        {
                            FilePath = string.Empty,
                            Sha256 = string.Empty,
                            DurationMs = 0,
                            FileSizeBytes = 0
                        });
                    }
                }
                finally
                {
                    semaphore.Release();
                }
            }, ct);

            tasks.Add(task);
        }

        await Task.WhenAll(tasks).ConfigureAwait(false);
        sw.Stop();

        _logger.LogInformation(
            "Batch download completed: {Count} models in {Duration}ms",
            manifest.Length, sw.ElapsedMilliseconds);

        return results;
    }

    public async Task<SketchfabRateLimitState?> GetQuotaAsync(CancellationToken ct = default)
    {
        await _rateLimitLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            // Return cached quota if fresh (< 1 minute old)
            if (_rateLimitRemaining >= 0 && _timeProvider.GetUtcNow().UtcDateTime - _lastQuotaCheck < TimeSpan.FromSeconds(60))
            {
                return new SketchfabRateLimitState
                {
                    Remaining = _rateLimitRemaining,
                    ResetAtUtc = _rateLimitReset,
                    ResetInSeconds = (int)Math.Max(0, (_rateLimitReset - _timeProvider.GetUtcNow().UtcDateTime).TotalSeconds),
                    LastCheckedUtc = _lastQuotaCheck
                };
            }

            _logger.LogDebug("Fetching fresh Sketchfab API quota from server");

            try
            {
                // Make a validation request to get rate limit headers
                var validation = await _client.ValidateTokenAsync(ct).ConfigureAwait(false);
                var quotaState = _client.GetRateLimitState();

                if (quotaState != null)
                {
                    // Update cached state
                    _rateLimitRemaining = quotaState.Remaining;
                    _rateLimitReset = quotaState.ResetAtUtc;
                    _lastQuotaCheck = _timeProvider.GetUtcNow().UtcDateTime;

                    _logger.LogInformation(
                        "Quota refreshed: {Remaining} requests remaining, resets in {ResetIn}s",
                        quotaState.Remaining, quotaState.ResetInSeconds);

                    return quotaState;
                }

                return null;
            }
            catch (SketchfabApiException ex)
            {
                _logger.LogError(ex, "Failed to fetch quota");
                throw;
            }
        }
        finally
        {
            _rateLimitLock.Release();
        }
    }

    public async Task<SketchfabTokenValidation> ValidateTokenAsync(CancellationToken ct = default)
    {
        _logger.LogInformation("Validating Sketchfab API token");

        try
        {
            var validation = await _client.ValidateTokenAsync(ct).ConfigureAwait(false);
            _logger.LogInformation(
                "Token validation: IsValid={IsValid}, Plan={Plan}, Remaining={Remaining}, DailyLimit={DailyLimit}",
                validation.IsValid, validation.Plan, validation.RemainingQuota, validation.DailyLimit);
            return validation;
        }
        catch (OperationCanceledException)
        {
            _logger.LogWarning("Token validation cancelled");
            throw;
        }
        catch (SketchfabAuthenticationException ex)
        {
            _logger.LogWarning(ex, "Token validation failed - invalid or expired token");
            throw;
        }
        catch (SketchfabApiException ex)
        {
            _logger.LogError(ex, "Token validation error");
            throw;
        }
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// <summary>
    /// Validates that input is not null or empty.
    /// </summary>
    private static void ValidateInput(string? input, string paramName)
    {
        if (string.IsNullOrWhiteSpace(input))
            throw new ArgumentException($"{paramName} cannot be null or empty", paramName);
    }

    /// <summary>
    /// Batch manifest item structure for deserialization.
    /// </summary>
    private sealed class BatchManifestItem
    {
        [JsonPropertyName("model_id")]
        public string ModelId { get; set; } = string.Empty;

        [JsonPropertyName("format")]
        public string? Format { get; set; }

        [JsonPropertyName("output_path")]
        public string? OutputPath { get; set; }
    }
}
