#nullable enable
namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Configuration for Sketchfab API integration.
/// Loaded from appsettings.json and environment variables.
/// </summary>
public sealed class SketchfabConfiguration
{
    /// <summary>
    /// Sketchfab API base URL (default: https://api.sketchfab.com/v3)
    /// </summary>
    public string ApiBaseUrl { get; set; } = "https://api.sketchfab.com/v3";

    /// <summary>
    /// Rate limit delay in milliseconds between requests.
    /// </summary>
    public int RateLimitDelayMs { get; set; } = 2000;

    /// <summary>
    /// Default number of search results to return.
    /// </summary>
    public int DefaultSearchLimit { get; set; } = 5;

    /// <summary>
    /// Maximum number of retries for failed requests.
    /// </summary>
    public int MaxRetries { get; set; } = 3;

    /// <summary>
    /// HTTP request timeout in seconds.
    /// </summary>
    public int HttpTimeoutSeconds { get; set; } = 30;
}

/// <summary>
/// Configuration for asset pipeline operations.
/// </summary>
public sealed class AssetPipelineConfiguration
{
    /// <summary>
    /// Directory where downloaded models are stored.
    /// </summary>
    public string DownloadDir { get; set; } = "packs/warfare-starwars/assets/raw";

    /// <summary>
    /// Directory for caching API responses and metadata.
    /// </summary>
    public string CacheDir { get; set; } = ".cache/sketchfab";

    /// <summary>
    /// Temporary directory for downloads before moving to final location.
    /// </summary>
    public string TempDir { get; set; } = "/tmp/asset-downloads";

    /// <summary>
    /// Enable SHA256 integrity checks on downloaded files.
    /// </summary>
    public bool IntegrityCheckEnabled { get; set; } = true;

    /// <summary>
    /// Automatically generate SHA256 hashes for downloaded files.
    /// </summary>
    public bool GenerateHashes { get; set; } = true;

    /// <summary>
    /// Automatically generate asset_manifest.json.
    /// </summary>
    public bool AutoManifest { get; set; } = true;
}
