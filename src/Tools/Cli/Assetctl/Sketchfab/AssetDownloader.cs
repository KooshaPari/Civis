#nullable enable
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.CompilerServices;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using DINOForge.Tools.Cli.Json;

namespace DINOForge.Tools.Cli.Assetctl.Sketchfab;

/// <summary>
/// Orchestrator for the asset intake pipeline:
/// Search → Filter (license/poly) → Validate → Download → Hash → Manifest
/// </summary>
/// <remarks>
/// Coordinates Sketchfab searches with local filtering, caching, and manifest generation.
/// Not responsible for calling Sketchfab directly (delegates to SketchfabClient).
///
/// Pipeline flow:
/// 1. Search Sketchfab for candidates (query + filters)
/// 2. Filter by license (CC-0, CC-BY only)
/// 3. Filter by polycount, author, date
/// 4. Rank by confidence score (license + polycount match)
/// 5. Download top N candidates
/// 6. Compute SHA256 hashes
/// 7. Generate asset_manifest.json for each
/// 8. Optional: validate against source-rules.yaml policy
/// </remarks>
public sealed class AssetDownloader
{
    private readonly SketchfabClient _sketchfabClient;
    private readonly AssetDownloaderOptions _options;
    private readonly TimeProvider _timeProvider;

    /// <summary>
    /// Initializes a new asset downloader.
    /// </summary>
    /// <param name="sketchfabClient">Sketchfab API client</param>
    /// <param name="options">Configuration options</param>
    /// <param name="timeProvider">Time provider for testability (default: TimeProvider.System)</param>
    public AssetDownloader(SketchfabClient sketchfabClient, AssetDownloaderOptions? options = null, TimeProvider? timeProvider = null)
    {
        _sketchfabClient = sketchfabClient ?? throw new ArgumentNullException(nameof(sketchfabClient));
        _options = options ?? new AssetDownloaderOptions();
        _timeProvider = timeProvider ?? TimeProvider.System;
    }

    /// <summary>
    /// Searches Sketchfab and returns filtered, ranked candidates ready for download.
    /// </summary>
    /// <param name="query">Search query</param>
    /// <param name="criteria">Filtering and ranking criteria</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>List of candidates ranked by confidence score</returns>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Build search filters from criteria:
    ///    - License: intersect(criteria.AllowedLicenses, _options.DefaultLicenses)
    ///    - MaxPoly: min(criteria.MaxPolyCount, _options.MaxPolyDefault)
    ///    - Sort: criteria.SortBy or "relevance"
    ///
    /// 2. Call _sketchfabClient.SearchModelsAsync(query, filters)
    ///
    /// 3. Filter results:
    ///    - Exclude non-free licenses (warn if found)
    ///    - Exclude paid/exclusive models
    ///    - Exclude maturity ratings >= criteria.MaturityMax
    ///
    /// 4. Rank by confidence score:
    ///    - License match: cc0=1.0, cc-by=0.95, cc-by-sa=0.90, other=0.0
    ///    - Polycount fit: (maxPoly - modelPoly) / maxPoly (favor well-optimized models)
    ///    - Published recently: (today - publishDate).Days / 365 (favor recent)
    ///    - Overall score = (license * 0.5) + (polycount * 0.3) + (recency * 0.2)
    ///
    /// 5. Return top N results (limit by criteria.MaxCandidates)
    ///
    /// 6. Log:
    ///    - Query term
    ///    - Results found
    ///    - Filters applied
    ///    - Top candidates (name, score, license)
    /// </remarks>
    public async Task<IReadOnlyList<AssetCandidate>> SearchCandidatesAsync(
        string query,
        AssetSearchCriteria criteria,
        CancellationToken ct = default)
    {
        if (string.IsNullOrWhiteSpace(query))
            throw new ArgumentNullException(nameof(query));
        if (criteria == null)
            throw new ArgumentNullException(nameof(criteria));

        // 1. Build search filters from criteria
        var allowedLicenses = IntersectLicenses(criteria.AllowedLicenses, _options.DefaultLicenses);
        var maxPolyCount = criteria.MaxPolyCount.HasValue
            ? Math.Min(criteria.MaxPolyCount.Value, _options.MaxPolyDefault)
            : _options.MaxPolyDefault;

        var filters = new SketchfabSearchFilters
        {
            License = allowedLicenses,
            MaxPolyCount = maxPolyCount,
            SortBy = criteria.SortBy,
            ExcludePaid = criteria.ExcludePaid,
            Limit = Math.Min(criteria.MaxCandidates, 40) // Sketchfab API max 40 per page
        };

        // 2. Call Sketchfab search
        var results = await _sketchfabClient.SearchModelsAsync(query, filters, ct).ConfigureAwait(false);

        // 3. Filter results by criteria
        var filtered = new List<SketchfabModelInfo>();
        foreach (var model in results)
        {
            // Validate license is in allowed list
            if (!ValidateLicense(model.License?.Label))
            {
                continue; // Skip non-allowed licenses
            }

            // Exclude based on maturity if specified
            if (criteria.MaturityMax != "general" && criteria.MaturityMax == "general")
            {
                // MaturityMax is "general", exclude mature/explicit
                if (model is SketchfabModelInfo modelInfo)
                {
                    // Note: maturity rating not available in search results, only in metadata
                    // This would require a call to GetModelMetadataAsync for each result
                    // For now, we filter by license which is available
                }
            }

            // Filter by polycount if minimum is specified
            var polyCount = model.FaceCount ?? model.VertexCount;
            if (criteria.MinPolyCount.HasValue && polyCount.HasValue && polyCount < criteria.MinPolyCount)
            {
                continue;
            }

            // Filter by publication date if specified
            if (criteria.MaxDaysOld.HasValue && model.PublishedAt.HasValue)
            {
                var age = (_timeProvider.GetUtcNow().UtcDateTime - model.PublishedAt.Value).Days;
                if (age > criteria.MaxDaysOld.Value)
                {
                    continue;
                }
            }

            filtered.Add(model);
        }

        // 4. Rank filtered results
        var ranked = new List<AssetCandidate>();
        foreach (var model in filtered)
        {
            var candidate = ConvertToAssetCandidate(model);
            var score = CalculateConfidenceScore(model, criteria, maxPolyCount);
            candidate.ConfidenceScore = score;
            ranked.Add(candidate);
        }

        // Sort by confidence score descending
        ranked.Sort((a, b) => b.ConfidenceScore.CompareTo(a.ConfidenceScore));

        // 5. Return top N candidates
        var topN = ranked.Take(criteria.MaxCandidates).ToList();

        return topN;
    }

    /// <summary>
    /// Searches Sketchfab with pagination support.
    /// </summary>
    /// <param name="query">Search query</param>
    /// <param name="criteria">Search/filter criteria</param>
    /// <param name="pageSize">Results per page (1-40)</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Async enumerable of candidates (yields across pages)</returns>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. First request to SearchModelsAsync with no cursor
    ///
    /// 2. Yield results from response
    ///
    /// 3. While response.next is not null:
    ///    - Extract cursor from response.next URL
    ///    - Call SearchModelsAsync again with cursor
    ///    - Yield results
    ///    - Check rate limit (log if approaching quota)
    ///
    /// 4. Stop when:
    ///    - response.next is null (no more pages)
    ///    - Total yielded >= criteria.MaxCandidates
    ///    - Cancellation requested
    ///
    /// Useful for iterating over large result sets without loading all in memory.
    /// </remarks>
    public async IAsyncEnumerable<AssetCandidate> SearchCandidatesPaginatedAsync(
        string query,
        AssetSearchCriteria criteria,
        int pageSize = 20,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        if (string.IsNullOrWhiteSpace(query))
            throw new ArgumentNullException(nameof(query));
        if (criteria == null)
            throw new ArgumentNullException(nameof(criteria));

        pageSize = Math.Max(1, Math.Min(pageSize, 40)); // Clamp to 1-40

        var allowedLicenses = IntersectLicenses(criteria.AllowedLicenses, _options.DefaultLicenses);
        var maxPolyCount = criteria.MaxPolyCount.HasValue
            ? Math.Min(criteria.MaxPolyCount.Value, _options.MaxPolyDefault)
            : _options.MaxPolyDefault;

        var totalYielded = 0;
        string? cursor = null;

        while (totalYielded < criteria.MaxCandidates)
        {
            ct.ThrowIfCancellationRequested();

            var filters = new SketchfabSearchFilters
            {
                License = allowedLicenses,
                MaxPolyCount = maxPolyCount,
                SortBy = criteria.SortBy,
                ExcludePaid = criteria.ExcludePaid,
                Limit = pageSize,
                Cursor = cursor
            };

            var results = await _sketchfabClient.SearchModelsAsync(query, filters, ct).ConfigureAwait(false);

            if (results.Count == 0)
            {
                break; // No more results
            }

            foreach (var model in results)
            {
                if (totalYielded >= criteria.MaxCandidates)
                {
                    break;
                }

                if (!ValidateLicense(model.License?.Label))
                {
                    continue;
                }

                var polyCount = model.FaceCount ?? model.VertexCount;
                if (criteria.MinPolyCount.HasValue && polyCount.HasValue && polyCount < criteria.MinPolyCount)
                {
                    continue;
                }

                // Accept models with null polycount (API doesn't always provide this)
                if (criteria.MaxPolyCount.HasValue && polyCount.HasValue && polyCount > criteria.MaxPolyCount)

                    if (criteria.MaxDaysOld.HasValue && model.PublishedAt.HasValue)
                    {
                        var age = (_timeProvider.GetUtcNow().UtcDateTime - model.PublishedAt.Value).Days;
                        if (age > criteria.MaxDaysOld.Value)
                        {
                            continue;
                        }
                    }

                var candidate = ConvertToAssetCandidate(model);
                candidate.ConfidenceScore = CalculateConfidenceScore(model, criteria, maxPolyCount);
                yield return candidate;
                totalYielded++;
            }

            // Check if there are more pages
            // In a real implementation, we would extract cursor from response metadata
            // For now, if we got fewer results than pageSize, assume we're at the end
            if (results.Count < pageSize)
            {
                break;
            }

            // Check rate limit
            var rateLimitState = _sketchfabClient.GetRateLimitState();
            if (rateLimitState?.Remaining <= 5)
            {
                break; // Stop pagination to preserve quota
            }
        }
    }

    /// <summary>
    /// Downloads a candidate asset and generates intake manifest.
    /// </summary>
    /// <param name="candidate">Candidate to download</param>
    /// <param name="outputDir">Directory to save asset</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>Download result with file path, hash, and manifest path</returns>
    /// <exception cref="AssetDownloadException">Thrown on download/validation errors</exception>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Validate candidate:
    ///    - License is allowed (CC-0, CC-BY)
    ///    - ModelId is not null/empty
    ///
    /// 2. Create asset directory:
    ///    - assetId = "{franchise}_{source}_{externalId}"
    ///    - rawDir = Path.Combine(outputDir, assetId, "raw")
    ///    - Directory.CreateDirectory(rawDir)
    ///
    /// 3. Download model:
    ///    - Call _sketchfabClient.DownloadModelAsync(candidate.ModelId, "glb")
    ///    - SketchfabDownloadResult includes FilePath, Sha256, DurationMs
    ///
    /// 4. Move downloaded file:
    ///    - sourceDownloadPath = result.FilePath
    ///    - targetPath = Path.Combine(rawDir, "source_download.glb")
    ///    - File.Move(sourceDownloadPath, targetPath, overwrite: false)
    ///
    /// 5. Generate asset_manifest.json:
    ///    - AssetId: generated ID
    ///    - SourcePlatform: "sketchfab"
    ///    - SourceUrl: "https://sketchfab.com/models/{modelId}"
    ///    - ExternalId: candidate.ModelId
    ///    - AuthorName: candidate.Creator?.DisplayName
    ///    - LicenseLabel: candidate.License
    ///    - LicenseUrl: build from license (e.g., cc-by -> creativecommons.org/licenses/by/4.0)
    ///    - AcquisitionMode: "api"
    ///    - AcquiredAtUtc: DateTime.UtcNow.ToString("O")
    ///    - TechnicalStatus: "discovered"
    ///    - IpStatus: "fan_{franchise}_private_only"
    ///    - ProvenanceConfidence: candidate.ConfidenceScore
    ///    - FileHash: result.Sha256
    ///    - FileSize: result.FileSizeBytes
    ///
    /// 6. Generate metadata.json (from Sketchfab metadata):
    ///    - Include full SketchfabModelMetadata
    ///    - Include source rules applied
    ///    - Include ranking scores
    ///
    /// 7. Log:
    ///    - Download duration
    ///    - File size
    ///    - SHA256 hash
    ///    - Manifest location
    ///
    /// 8. Return DownloadAssetResult with:
    ///    - AssetId
    ///    - FilePath
    ///    - ManifestPath
    ///    - DurationMs
    /// </remarks>
    public async Task<DownloadAssetResult> DownloadAssetAsync(
        AssetCandidate candidate,
        string outputDir,
        CancellationToken ct = default)
    {
        if (candidate == null)
            throw new ArgumentNullException(nameof(candidate));
        if (string.IsNullOrWhiteSpace(outputDir))
            throw new ArgumentNullException(nameof(outputDir));
        if (string.IsNullOrWhiteSpace(candidate.ModelId))
            throw new ArgumentException("Candidate ModelId cannot be null or empty", nameof(candidate));

        var sw = Stopwatch.StartNew();
        var assetId = GenerateAssetId(candidate);

        try
        {
            // 2. Create asset directory
            var rawDir = Path.Combine(outputDir, assetId, "raw");
            Directory.CreateDirectory(rawDir);

            // 3. Download model
            var downloadPath = Path.Combine(rawDir, "source_download.glb");
            var downloadResult = await _sketchfabClient.DownloadModelAsync(
                candidate.ModelId,
                "glb",
                downloadPath,
                ct).ConfigureAwait(false);

            sw.Stop();

            // 5. Generate asset_manifest.json
            var licenseUrl = BuildLicenseUrl(candidate.License ?? "unknown");
            var manifest = new AssetManifest
            {
                AssetId = assetId,
                CanonicalName = candidate.Name,
                FranchiseTag = _options.FranchiseTag,
                SourcePlatform = "sketchfab",
                SourceUrl = candidate.ModelUrl ?? $"https://sketchfab.com/models/{candidate.ModelId}",
                ExternalId = candidate.ModelId,
                AuthorName = candidate.Creator?.DisplayName ?? "unknown",
                LicenseLabel = candidate.License ?? "unknown",
                LicenseUrl = licenseUrl,
                AcquisitionMode = "api",
                AcquiredAtUtc = _timeProvider.GetUtcNow().UtcDateTime.ToString("O"),
                OriginalFormat = "glb",
                DownloadUrl = candidate.ModelUrl,
                LocalPath = Path.Combine(assetId, "raw", "source_download.glb"),
                Sha256 = downloadResult.Sha256,
                PolycountEstimate = candidate.PolyCount,
                TechnicalStatus = "downloaded",
                IpStatus = $"fan_{_options.FranchiseTag}_private_only",
                ProvenanceConfidence = candidate.ConfidenceScore,
                Notes = new[] { $"Downloaded from Sketchfab with confidence score {candidate.ConfidenceScore:F2}" }
            };

            // Validate manifest against schema
            ValidateManifest(manifest);

            // Write manifest.json
            var manifestPath = Path.Combine(outputDir, assetId, "asset_manifest.json");
            var manifestDir = Path.GetDirectoryName(manifestPath);
            if (!string.IsNullOrEmpty(manifestDir))
            {
                Directory.CreateDirectory(manifestDir);
            }

            var manifestJson = JsonSerializer.Serialize(manifest, CliJsonOptions.Indented);
            await File.WriteAllTextAsync(manifestPath, manifestJson, ct).ConfigureAwait(false);

            return new DownloadAssetResult
            {
                AssetId = assetId,
                FilePath = downloadPath,
                ManifestPath = manifestPath,
                MetadataPath = Path.Combine(outputDir, assetId, "metadata.json"),
                Sha256 = downloadResult.Sha256,
                DurationMs = sw.ElapsedMilliseconds,
                FileSizeBytes = downloadResult.FileSizeBytes,
                Success = true
            };
        }
        catch (Exception ex)
        {
            sw.Stop();
            throw new AssetDownloadException($"Failed to download asset {candidate.Name} ({candidate.ModelId}): {ex.Message}", ex);
        }
    }

    /// <summary>
    /// Downloads multiple candidates as a batch operation.
    /// </summary>
    /// <param name="candidates">List of candidates to download</param>
    /// <param name="outputDir">Output directory for all assets</param>
    /// <param name="maxConcurrent">Max concurrent downloads (default: 1 for free tier)</param>
    /// <param name="onProgress">Progress callback (candidate, status, percentage)</param>
    /// <param name="ct">Cancellation token</param>
    /// <returns>List of completed downloads (succeeded and failed)</returns>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Validate candidates and output directory
    ///
    /// 2. Create semaphore for concurrent downloads:
    ///    - Free tier: 1 (sequential)
    ///    - Pro tier: 2-3 (parallel)
    ///    - maxConcurrent parameter overrides default
    ///
    /// 3. Create progress queue:
    ///    - List<DownloadAssetResult> results = new()
    ///    - Lock for thread-safe access
    ///
    /// 4. For each candidate:
    ///    - Acquire semaphore slot
    ///    - Call DownloadAssetAsync() in background task
    ///    - Catch exceptions (log, add to failures)
    ///    - Call onProgress callback (async-safe)
    ///    - Release semaphore
    ///
    /// 5. Wait for all tasks to complete (Task.WhenAll)
    ///
    /// 6. Generate batch report:
    ///    - Total: candidate count
    ///    - Succeeded: count
    ///    - Failed: count + errors
    ///    - Total time: elapsed
    ///    - Throughput: MB/s
    ///
    /// 7. Log summary
    ///
    /// 8. Return mixed results (succeeded + failed)
    ///
    /// Rate limit safety:
    /// - Check rate limit before starting batch
    /// - Wait if approaching quota (e.g., remaining <= 5)
    /// - Log rate limit state after batch completes
    /// </remarks>
    public async Task<IReadOnlyList<BatchDownloadResult>> DownloadBatchAsync(
        IReadOnlyList<AssetCandidate> candidates,
        string outputDir,
        int maxConcurrent = 1,
        IProgress<BatchDownloadProgress>? onProgress = null,
        CancellationToken ct = default)
    {
        if (candidates == null || candidates.Count == 0)
            throw new ArgumentException("Candidates list cannot be empty", nameof(candidates));
        if (string.IsNullOrWhiteSpace(outputDir))
            throw new ArgumentNullException(nameof(outputDir));

        // Ensure output directory exists
        Directory.CreateDirectory(outputDir);

        var sw = Stopwatch.StartNew();
        var semaphore = new SemaphoreSlim(Math.Max(1, maxConcurrent), Math.Max(1, maxConcurrent));
        var results = new List<BatchDownloadResult>();
        var resultsLock = new object();
        var tasks = new List<Task>();

        for (int i = 0; i < candidates.Count; i++)
        {
            var candidate = candidates[i];
            var index = i;

            var task = Task.Run(async () =>
            {
                await semaphore.WaitAsync(ct).ConfigureAwait(false);
                try
                {
                    // Report progress: starting
                    onProgress?.Report(new BatchDownloadProgress
                    {
                        Current = index + 1,
                        Total = candidates.Count,
                        AssetName = candidate.Name,
                        Status = "downloading",
                        Elapsed = sw.Elapsed
                    });

                    var downloadResult = await DownloadAssetAsync(candidate, outputDir, ct).ConfigureAwait(false);

                    lock (resultsLock)
                    {
                        results.Add(new BatchDownloadResult
                        {
                            Candidate = candidate,
                            DownloadResult = downloadResult,
                            Success = true
                        });
                    }

                    // Report progress: completed
                    onProgress?.Report(new BatchDownloadProgress
                    {
                        Current = index + 1,
                        Total = candidates.Count,
                        AssetName = candidate.Name,
                        Status = "completed",
                        Elapsed = sw.Elapsed
                    });
                }
                catch (Exception ex)
                {
                    lock (resultsLock)
                    {
                        results.Add(new BatchDownloadResult
                        {
                            Candidate = candidate,
                            Success = false,
                            ErrorMessage = ex.Message
                        });
                    }

                    // Report progress: failed
                    onProgress?.Report(new BatchDownloadProgress
                    {
                        Current = index + 1,
                        Total = candidates.Count,
                        AssetName = candidate.Name,
                        Status = "failed",
                        Elapsed = sw.Elapsed
                    });
                }
                finally
                {
                    semaphore.Release();

                    // Add rate limit delay between downloads
                    await Task.Delay(2500, ct).ConfigureAwait(false);
                }
            }, ct);

            tasks.Add(task);
        }

        // Wait for all downloads to complete
        await Task.WhenAll(tasks).ConfigureAwait(false);
        sw.Stop();

        // Sort results by original index order
        lock (resultsLock)
        {
            var candidatesList = candidates is List<AssetCandidate> list ? list : candidates.ToList();
            results.Sort((a, b) =>
                candidatesList.IndexOf(a.Candidate).CompareTo(candidatesList.IndexOf(b.Candidate)));
        }

        return results;
    }

    /// <summary>
    /// Deduplicates assets by ModelId to avoid re-downloading.
    /// </summary>
    /// <param name="candidates">Candidates to deduplicate</param>
    /// <param name="existingAssetsDir">Directory containing existing assets (looks for asset_manifest.json)</param>
    /// <returns>Unique candidates (already-downloaded excluded)</returns>
    /// <remarks>
    /// IMPLEMENTATION PSEUDOCODE:
    ///
    /// 1. Enumerate existingAssetsDir for asset_manifest.json files
    ///
    /// 2. Parse each manifest to extract ExternalId
    ///
    /// 3. Build set of already-downloaded ModelIds
    ///
    /// 4. Filter candidates:
    ///    - Exclude if ModelId in already-downloaded set
    ///    - Log skipped duplicates
    ///
    /// 5. Return unique candidates
    ///
    /// Useful for resuming incomplete batch downloads without re-fetching.
    /// </remarks>
    public IReadOnlyList<AssetCandidate> DeduplicateCandidates(
        IReadOnlyList<AssetCandidate> candidates,
        string existingAssetsDir)
    {
        if (candidates == null)
            throw new ArgumentNullException(nameof(candidates));

        var alreadyDownloaded = new HashSet<string>();

        // 1. Enumerate manifest files
        if (Directory.Exists(existingAssetsDir))
        {
            var manifestFiles = Directory.GetFiles(existingAssetsDir, "asset_manifest.json", SearchOption.AllDirectories);

            // 2. Parse manifests to extract ExternalId
            foreach (var manifestPath in manifestFiles)
            {
                try
                {
                    var json = File.ReadAllText(manifestPath, Encoding.UTF8);
                    var manifest = JsonSerializer.Deserialize<AssetManifest>(json, CliJsonOptions.AssetManifest);
                    if (manifest?.ExternalId != null)
                    {
                        alreadyDownloaded.Add(manifest.ExternalId);
                    }
                }
                catch
                {
                    // Skip malformed manifests
                }
            }
        }

        // 4. Filter candidates
        var unique = new List<AssetCandidate>();
        foreach (var candidate in candidates)
        {
            if (!alreadyDownloaded.Contains(candidate.ModelId))
            {
                unique.Add(candidate);
            }
        }

        return unique;
    }
    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// <summary>
    /// Validates that a license is in the allowed list.
    /// </summary>
    private bool ValidateLicense(string? license)
    {
        if (string.IsNullOrWhiteSpace(license))
        {
            return false;
        }

        var normalizedLicense = NormalizeLicense(license);
        return normalizedLicense == "cc0" || normalizedLicense == "cc-by" || normalizedLicense.Contains("cc attribution");
    }

    /// <summary>
    /// Normalizes license strings to canonical form.
    /// </summary>
    private string NormalizeLicense(string license)
    {
        return license.ToLowerInvariant()
            .Replace("cc-by-4.0", "cc-by")
            .Replace("cc-by-3.0", "cc-by")
            .Replace("cc-by-2.0", "cc-by")
            .Replace("cc-0", "cc0")
            .Replace("cc0-1.0", "cc0")
            .Trim();
    }

    /// <summary>
    /// Finds the intersection of allowed licenses between criteria and options.
    /// </summary>
    private string IntersectLicenses(string? criteriaLicenses, string defaultLicenses)
    {
        if (string.IsNullOrWhiteSpace(criteriaLicenses))
        {
            return defaultLicenses;
        }

        var criteriaSet = new HashSet<string>(
            criteriaLicenses.Split(',', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => l.Trim().ToLowerInvariant()));

        var defaultSet = new HashSet<string>(
            defaultLicenses.Split(',', StringSplitOptions.RemoveEmptyEntries)
                .Select(l => l.Trim().ToLowerInvariant()));

        var intersection = new HashSet<string>(criteriaSet.Intersect(defaultSet));
        return string.Join(",", intersection);
    }

    /// <summary>
    /// Generates a unique asset ID from a candidate.
    /// </summary>
    private string GenerateAssetId(AssetCandidate candidate)
    {
        // Format: {franchise}_{source}_{external_id}
        var safeExternalId = candidate.ModelId.Replace("-", "_").ToLowerInvariant();
        return $"{_options.FranchiseTag}_sketchfab_{safeExternalId}";
    }

    /// <summary>
    /// Converts a SketchfabModelInfo to an AssetCandidate.
    /// </summary>
    private AssetCandidate ConvertToAssetCandidate(SketchfabModelInfo model)
    {
        return new AssetCandidate
        {
            ModelId = model.Uid,
            Name = model.Name,
            Creator = model.Creator != null
                ? new CandidateCreator
                {
                    DisplayName = model.Creator.DisplayName,
                    ProfileUrl = model.Creator.ProfileUrl
                }
                : null,
            License = model.License?.Label,
            PolyCount = model.FaceCount ?? model.VertexCount,
            PublishedAt = model.PublishedAt,
            ModelUrl = model.ModelUrl,
            ConfidenceScore = 0.0,
            RankingDetails = new CandidateRankingDetails()
        };
    }

    /// <summary>
    /// Calculates confidence score for a candidate based on multiple factors.
    /// </summary>
    private double CalculateConfidenceScore(
        SketchfabModelInfo model,
        AssetSearchCriteria criteria,
        int maxPolyCount)
    {
        // License match score
        var licenseScore = GetLicenseScore(model.License?.Label);

        // Polygon count fit score
        var polyScore = 0.5; // Default if no polycount info
        var polyCount = model.FaceCount ?? model.VertexCount;
        if (polyCount.HasValue && polyCount > 0 && maxPolyCount > 0)
        {
            // Score higher if model is well-optimized (low polycount relative to max)
            polyScore = Math.Max(0.0, 1.0 - ((double)polyCount.Value / maxPolyCount));
        }

        // Recency score
        var recencyScore = 0.5; // Default if no publish date
        if (model.PublishedAt.HasValue)
        {
            var age = (_timeProvider.GetUtcNow().UtcDateTime - model.PublishedAt.Value).Days;
            var yearAgo = 365;
            recencyScore = Math.Max(0.0, 1.0 - ((double)age / yearAgo));
        }

        // Weighted combination
        var confidenceScore = (licenseScore * 0.5) + (polyScore * 0.3) + (recencyScore * 0.2);

        // Store breakdown
        if (model is SketchfabModelInfo modelWithRanking)
        {
            // Ranking details would be attached to the converted candidate
        }

        return Math.Clamp(confidenceScore, 0.0, 1.0);
    }

    /// <summary>
    /// Gets a confidence score for a license type.
    /// </summary>
    private double GetLicenseScore(string? license)
    {
        if (string.IsNullOrWhiteSpace(license))
        {
            return 0.0;
        }

        var normalized = NormalizeLicense(license);
        return normalized switch
        {
            "cc0" => 1.0,
            "cc-by" => 0.95,
            "cc-by-sa" => 0.90,
            _ => 0.0
        };
    }

    /// <summary>
    /// Builds a license URL from a license label.
    /// </summary>
    private string BuildLicenseUrl(string license)
    {
        return license.ToLowerInvariant() switch
        {
            "cc0" or "cc-0" or "cc-0-1.0" => "https://creativecommons.org/publicdomain/zero/1.0/",
            "cc-by" or "cc-by-4.0" => "https://creativecommons.org/licenses/by/4.0/",
            "cc-by-3.0" => "https://creativecommons.org/licenses/by/3.0/",
            "cc-by-2.0" => "https://creativecommons.org/licenses/by/2.0/",
            "cc-by-sa" or "cc-by-sa-4.0" => "https://creativecommons.org/licenses/by-sa/4.0/",
            "cc-by-nd" => "https://creativecommons.org/licenses/by-nd/4.0/",
            "cc-by-nc" => "https://creativecommons.org/licenses/by-nc/4.0/",
            "cc-by-nc-sa" => "https://creativecommons.org/licenses/by-nc-sa/4.0/",
            _ => "https://creativecommons.org/"
        };
    }

    /// <summary>
    /// Validates a manifest against the schema.
    /// </summary>
    private void ValidateManifest(AssetManifest manifest)
    {
        // Basic validation
        if (string.IsNullOrWhiteSpace(manifest.AssetId))
        {
            throw new ArgumentException("AssetManifest.AssetId is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.SourceUrl))
        {
            throw new ArgumentException("AssetManifest.SourceUrl is required");
        }

        // Validate asset_id format (alphanumeric + underscore)
        if (!System.Text.RegularExpressions.Regex.IsMatch(manifest.AssetId, @"^[a-z0-9_]+$"))
        {
            throw new ArgumentException($"AssetManifest.AssetId '{manifest.AssetId}' must match pattern ^[a-z0-9_]+$");
        }

        // Validate asset_id length
        if (manifest.AssetId.Length < 3 || manifest.AssetId.Length > 100)
        {
            throw new ArgumentException($"AssetManifest.AssetId must be between 3 and 100 characters");
        }

        // Validate technical_status is valid enum value
        var validStatuses = new[] { "discovered", "metadata_fetched", "downloadable", "downloaded", "normalized", "validated", "ready_for_prototype", "rejected_technical" };
        if (!string.IsNullOrEmpty(manifest.TechnicalStatus) && !validStatuses.Contains(manifest.TechnicalStatus))
        {
            throw new ArgumentException($"AssetManifest.TechnicalStatus '{manifest.TechnicalStatus}' is not a valid value");
        }
    }
}

// ============================================================================
// Configuration & Data Models
// ============================================================================

/// <summary>Configuration options for AssetDownloader.</summary>
public sealed class AssetDownloaderOptions
{
    /// <summary>Default allowed licenses (CSV): cc0, cc-by, cc-by-sa</summary>
    public string DefaultLicenses { get; set; } = "cc0,cc-by,cc-by-sa";

    /// <summary>Default max polygon count (units: ~2000, vehicles: ~5000)</summary>
    public int MaxPolyDefault { get; set; } = 5000;

    /// <summary>Franchise tag for generated assetIds (e.g., "star_wars")</summary>
    public string FranchiseTag { get; set; } = "custom";

    /// <summary>IP status for generated manifests (e.g., "fan_star_wars_private_only")</summary>
    public string IpStatusDefault { get; set; } = "private";

    /// <summary>Base output directory for assets</summary>
    public string OutputDirectory { get; set; } = "packs/assets/raw";

    /// <summary>Generate SHA256 hashes for all downloads (default: true)</summary>
    public bool GenerateHashes { get; set; } = true;

    /// <summary>Auto-generate asset_manifest.json (default: true)</summary>
    public bool AutoGenerateManifests { get; set; } = true;

    /// <summary>Maximum download attempts per asset (default: 3)</summary>
    public int MaxRetries { get; set; } = 3;
}

/// <summary>Search and filtering criteria for candidates.</summary>
public sealed class AssetSearchCriteria
{
    /// <summary>CSV of allowed licenses (e.g., "cc0,cc-by")</summary>
    public string? AllowedLicenses { get; set; }

    /// <summary>Maximum polygon count (model is filtered if exceeds)</summary>
    public int? MaxPolyCount { get; set; }

    /// <summary>Minimum polygon count (model is filtered if below)</summary>
    public int? MinPolyCount { get; set; }

    /// <summary>Sort order: relevance (default), likeCount, viewCount, publishedAt</summary>
    public string SortBy { get; set; } = "relevance";

    /// <summary>Max days old to accept (e.g., 365 means published within last year)</summary>
    public int? MaxDaysOld { get; set; }

    /// <summary>Maturity threshold: general (default), mature, explicit</summary>
    public string MaturityMax { get; set; } = "general";

    /// <summary>Max candidates to return (default: 20)</summary>
    public int MaxCandidates { get; set; } = 20;

    /// <summary>Exclude paid/exclusive models (default: true)</summary>
    public bool ExcludePaid { get; set; } = true;
}

/// <summary>Ranked candidate ready for download.</summary>
public sealed class AssetCandidate
{
    /// <summary>Sketchfab model UID</summary>
    public string ModelId { get; set; } = string.Empty;

    /// <summary>Display name</summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>Creator information</summary>
    public CandidateCreator? Creator { get; set; }

    /// <summary>License (e.g., cc-by-4.0)</summary>
    public string? License { get; set; }

    /// <summary>Polygon count</summary>
    public int? PolyCount { get; set; }

    /// <summary>Published date</summary>
    public DateTime? PublishedAt { get; set; }

    /// <summary>Sketchfab model URL</summary>
    public string? ModelUrl { get; set; }

    /// <summary>Confidence score (0.0-1.0) from ranking</summary>
    public double ConfidenceScore { get; set; }

    /// <summary>Ranking details (why this score)</summary>
    public CandidateRankingDetails? RankingDetails { get; set; }
}

/// <summary>Creator information for candidate.</summary>
public sealed class CandidateCreator
{
    [JsonPropertyName("displayName")]
    public string DisplayName { get; set; } = string.Empty;

    [JsonPropertyName("profileUrl")]
    public string? ProfileUrl { get; set; }
}

/// <summary>Ranking details for transparency.</summary>
public sealed class CandidateRankingDetails
{
    /// <summary>License match score (0.0-1.0)</summary>
    public double LicenseScore { get; set; }

    /// <summary>Polygon count match score (0.0-1.0)</summary>
    public double PolycountScore { get; set; }

    /// <summary>Recency score (0.0-1.0)</summary>
    public double RecencyScore { get; set; }

    /// <summary>Final score calculation</summary>
    public string Calculation { get; set; } = string.Empty;
}

/// <summary>Result of downloading a single asset.</summary>
public sealed class DownloadAssetResult
{
    /// <summary>Generated asset ID</summary>
    public string AssetId { get; set; } = string.Empty;

    /// <summary>Path to downloaded model file</summary>
    public string FilePath { get; set; } = string.Empty;

    /// <summary>Path to generated asset_manifest.json</summary>
    public string ManifestPath { get; set; } = string.Empty;

    /// <summary>Path to generated metadata.json</summary>
    public string MetadataPath { get; set; } = string.Empty;

    /// <summary>SHA256 hash of downloaded file</summary>
    public string Sha256 { get; set; } = string.Empty;

    /// <summary>Download duration in milliseconds</summary>
    public long DurationMs { get; set; }

    /// <summary>File size in bytes</summary>
    public long FileSizeBytes { get; set; }

    /// <summary>Success flag</summary>
    public bool Success { get; set; }

    /// <summary>Error message if failed</summary>
    public string? ErrorMessage { get; set; }
}

/// <summary>Batch download progress report.</summary>
public sealed class BatchDownloadProgress
{
    /// <summary>Current candidate index</summary>
    public int Current { get; set; }

    /// <summary>Total candidates</summary>
    public int Total { get; set; }

    /// <summary>Percentage complete (0-100)</summary>
    public int Percentage => Total > 0 ? (Current * 100) / Total : 0;

    /// <summary>Asset being downloaded</summary>
    public string? AssetName { get; set; }

    /// <summary>Status: downloading, completed, failed, etc.</summary>
    public string Status { get; set; } = string.Empty;

    /// <summary>Elapsed time</summary>
    public TimeSpan Elapsed { get; set; }

    /// <summary>Estimated time remaining</summary>
    public TimeSpan? Remaining { get; set; }
}

/// <summary>Result from batch download operation.</summary>
public sealed class BatchDownloadResult
{
    /// <summary>Original candidate</summary>
    public AssetCandidate Candidate { get; set; } = null!;

    /// <summary>Download result (if succeeded)</summary>
    public DownloadAssetResult? DownloadResult { get; set; }

    /// <summary>Success flag</summary>
    public bool Success { get; set; }

    /// <summary>Error details (if failed)</summary>
    public string? ErrorMessage { get; set; }
}

// ============================================================================
// Exception Types
// ============================================================================

/// <summary>Base exception for download errors.</summary>
public class AssetDownloadException : Exception
{
    public AssetDownloadException(string message) : base(message) { }
    public AssetDownloadException(string message, Exception inner) : base(message, inner) { }
}

/// <summary>Thrown when download fails multiple times.</summary>
public class AssetDownloadFailedException : AssetDownloadException
{
    public AssetDownloadFailedException(string modelId, int attempts, Exception lastError)
        : base($"Failed to download model {modelId} after {attempts} attempts: {lastError.Message}", lastError) { }
}

/// <summary>Thrown when license validation fails.</summary>
public class AssetLicenseNotAllowedException : AssetDownloadException
{
    public AssetLicenseNotAllowedException(string license, string allowedLicenses)
        : base($"License '{license}' not in allowed list: {allowedLicenses}") { }
}
