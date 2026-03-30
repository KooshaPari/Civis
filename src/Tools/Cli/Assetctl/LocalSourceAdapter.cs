#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Source adapter that reads assets from local pack registry directories.
/// </summary>
public sealed class LocalSourceAdapter : ISourceAdapter
{
    private readonly string _repositoryRoot;
    private readonly ILogger<LocalSourceAdapter>? _logger;
    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    /// <summary>
    /// Initializes a new instance of the <see cref="LocalSourceAdapter"/> class.
    /// </summary>
    /// <param name="repositoryRoot">Root path of the repository containing packs.</param>
    /// <param name="logger">Optional logger for diagnostics.</param>
    public LocalSourceAdapter(string repositoryRoot, ILogger<LocalSourceAdapter>? logger = null)
    {
        _repositoryRoot = repositoryRoot ?? throw new ArgumentNullException(nameof(repositoryRoot));
        _logger = logger;
    }

    /// <inheritdoc />
    public string SourceName => "local";

    /// <inheritdoc />
    public bool SupportsSearch => true;

    /// <inheritdoc />
    public bool SupportsGetById => true;

    /// <inheritdoc />
    public async Task<IEnumerable<AssetCandidate>> SearchAsync(string query, CancellationToken cancellationToken = default)
    {
        var results = new List<AssetCandidate>();

        try
        {
            IEnumerable<string> packDirs = GetPackDirectories();
            foreach (string packDir in packDirs)
            {
                cancellationToken.ThrowIfCancellationRequested();

                string registryPath = Path.Combine(packDir, "assets", "registry", "asset_index.json");
                if (!File.Exists(registryPath))
                {
                    continue;
                }

                try
                {
                    string json = await File.ReadAllTextAsync(registryPath, cancellationToken);
                    var entries = JsonSerializer.Deserialize<List<AssetIndexEntry>>(json, _jsonOptions);

                    if (entries is null)
                    {
                        continue;
                    }

                    string loweredQuery = query.ToLowerInvariant();
                    foreach (var entry in entries)
                    {
                        bool matches = MatchesQuery(entry, loweredQuery);
                        if (matches)
                        {
                            results.Add(ConvertToAssetCandidate(entry, packDir));
                        }
                    }
                }
                catch (Exception ex)
                {
                    _logger?.LogWarning(ex, "Failed to read registry at {Path}", registryPath);
                }
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to search assets in local registry");
        }

        return results;
    }

    /// <inheritdoc />
    public async Task<AssetCandidate?> GetByIdAsync(string id, CancellationToken cancellationToken = default)
    {
        try
        {
            IEnumerable<string> packDirs = GetPackDirectories();
            foreach (string packDir in packDirs)
            {
                cancellationToken.ThrowIfCancellationRequested();

                string registryPath = Path.Combine(packDir, "assets", "registry", "asset_index.json");
                if (!File.Exists(registryPath))
                {
                    continue;
                }

                string json = await File.ReadAllTextAsync(registryPath, cancellationToken);
                var entries = JsonSerializer.Deserialize<List<AssetIndexEntry>>(json, _jsonOptions);

                if (entries is null)
                {
                    continue;
                }

                var entry = entries.FirstOrDefault(e =>
                    string.Equals(e.AssetId, id, StringComparison.OrdinalIgnoreCase));

                if (entry is not null)
                {
                    return ConvertToAssetCandidate(entry, packDir);
                }
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to get asset {AssetId} from local registry", id);
        }

        return null;
    }

    private IEnumerable<string> GetPackDirectories()
    {
        string packsDir = Path.Combine(_repositoryRoot, "packs");
        if (!Directory.Exists(packsDir))
        {
            return Enumerable.Empty<string>();
        }

        return Directory.GetDirectories(packsDir)
            .Where(dir => File.Exists(Path.Combine(dir, "pack.yaml")) ||
                         File.Exists(Path.Combine(dir, "pack.json")));
    }

    private bool MatchesQuery(AssetIndexEntry entry, string query)
    {
        if (string.IsNullOrWhiteSpace(query))
        {
            return true;
        }

        // Check asset_id, canonical_name, source_platform
        bool matchesId = entry.AssetId.Contains(query, StringComparison.OrdinalIgnoreCase);
        bool matchesName = !string.IsNullOrEmpty(entry.CanonicalName) &&
                          entry.CanonicalName.Contains(query, StringComparison.OrdinalIgnoreCase);
        bool matchesSource = !string.IsNullOrEmpty(entry.SourcePlatform) &&
                            entry.SourcePlatform.Contains(query, StringComparison.OrdinalIgnoreCase);

        return matchesId || matchesName || matchesSource;
    }

    private AssetCandidate ConvertToAssetCandidate(AssetIndexEntry entry, string packDir)
    {
        string? packId = GetPackIdFromPath(packDir);
        List<string> tags = new();
        if (!string.IsNullOrEmpty(packId))
        {
            tags.Add(packId);
        }

        return new AssetCandidate
        {
            Source = !string.IsNullOrEmpty(entry.SourcePlatform) ? entry.SourcePlatform : "local",
            ExternalId = entry.AssetId,
            Title = entry.CanonicalName ?? entry.AssetId,
            AuthorName = entry.AuthorName,
            LicenseLabel = entry.LicenseLabel,
            SourceUrl = entry.SourceUrl ?? string.Empty,
            OriginalFormat = entry.OriginalFormat ?? "unknown",
            Category = entry.Category,
            Subtype = entry.Subtype,
            IpStatus = entry.IpStatus ?? "unknown_provenance",
            ProvenanceConfidence = 0.9, // Local assets have high provenance confidence
            Tags = tags.Count > 0 ? tags.ToArray() : null
        };
    }

    private string? GetPackIdFromPath(string packDir)
    {
        string fileName = Path.GetFileName(packDir);
        return fileName;
    }

    private sealed class AssetIndexEntry
    {
        public string AssetId { get; set; } = string.Empty;
        public string? CanonicalName { get; set; }
        public string? SourcePlatform { get; set; }
        public string? TechnicalStatus { get; set; }
        public string? IpStatus { get; set; }
        public bool ReleaseAllowed { get; set; }
        public string? AuthorName { get; set; }
        public string? LicenseLabel { get; set; }
        public string? SourceUrl { get; set; }
        public string? OriginalFormat { get; set; }
        public string? Category { get; set; }
        public string? Subtype { get; set; }
    }
}
