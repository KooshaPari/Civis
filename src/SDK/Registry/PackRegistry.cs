using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK.Json;

namespace DINOForge.SDK.Registry
{
    using TimeProvider = System.TimeProvider;
    // ---------------------------------------------------------------------------
    // Data transfer objects — mirror the registry.json schema exactly.
    // ---------------------------------------------------------------------------

    /// <summary>
    /// A single pack entry as it appears in the remote pack registry.
    /// </summary>
    public sealed class RegistryPackEntry
    {
        /// <summary>Unique kebab-case identifier (e.g. "warfare-modern").</summary>
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;

        /// <summary>Human-readable display name.</summary>
        [JsonPropertyName("name")]
        public string Name { get; set; } = string.Empty;

        /// <summary>Author or organisation name.</summary>
        [JsonPropertyName("author")]
        public string Author { get; set; } = string.Empty;

        /// <summary>Semantic version of the pack (e.g. "0.3.0").</summary>
        [JsonPropertyName("version")]
        public string Version { get; set; } = string.Empty;

        /// <summary>Pack category: content | balance | ruleset | total_conversion | utility.</summary>
        [JsonPropertyName("type")]
        public string Type { get; set; } = string.Empty;

        /// <summary>Short description shown in the installer UI.</summary>
        [JsonPropertyName("description")]
        public string Description { get; set; } = string.Empty;

        /// <summary>Searchable tag list.</summary>
        [JsonPropertyName("tags")]
        public List<string> Tags { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>URL of the pack's source repository.</summary>
        [JsonPropertyName("repo")]
        public string Repo { get; set; } = string.Empty;

        /// <summary>Direct download URL for the pack artifact (.zip).</summary>
        [JsonPropertyName("download_url")]
        public string DownloadUrl { get; set; } = string.Empty;

        /// <summary>Relative path within the repository where the pack manifest lives.</summary>
        [JsonPropertyName("pack_path")]
        public string PackPath { get; set; } = string.Empty;

        /// <summary>SemVer range expressing required DINOForge framework version.</summary>
        [JsonPropertyName("framework_version")]
        public string FrameworkVersion { get; set; } = string.Empty;

        /// <summary>True if reviewed and verified by the DINOForge team.</summary>
        [JsonPropertyName("verified")]
        public bool Verified { get; set; }

        /// <summary>True if the pack should appear in the featured section of the installer.</summary>
        [JsonPropertyName("featured")]
        public bool Featured { get; set; }

        /// <summary>Pack IDs that are mutually exclusive with this pack.</summary>
        [JsonPropertyName("conflicts_with")]
        public List<string> ConflictsWith { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Pack IDs that must be installed before this pack.</summary>
        [JsonPropertyName("depends_on")]
        public List<string> DependsOn { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List
    }

    /// <summary>Root document deserialized from the remote registry.json.</summary>
    internal sealed class RegistryDocument
    {
        [JsonPropertyName("version")]
        public string Version { get; set; } = string.Empty;

        [JsonPropertyName("updated")]
        public string Updated { get; set; } = string.Empty;

        [JsonPropertyName("packs")]
        public List<RegistryPackEntry> Packs { get; set; } = new List<RegistryPackEntry>(); // public-mutable-ok: JSON deserializer requires mutable List
    }

    // ---------------------------------------------------------------------------
    // Filter object
    // ---------------------------------------------------------------------------

    /// <summary>
    /// Criteria used to filter pack registry results.
    /// Null / empty values are treated as "no constraint" (match all).
    /// </summary>
    public sealed class PackRegistryFilter
    {
        /// <summary>
        /// When set, only packs that include ALL of these tags are returned.
        /// </summary>
        public IReadOnlyList<string>? Tags { get; set; }

        /// <summary>
        /// When set, only packs whose <c>type</c> field matches are returned
        /// (e.g. "balance", "total_conversion").
        /// </summary>
        public string? Type { get; set; }

        /// <summary>
        /// When <c>true</c>, only packs with <c>verified = true</c> are returned.
        /// When <c>false</c> or <c>null</c>, no constraint is applied.
        /// </summary>
        public bool? Verified { get; set; }

        /// <summary>
        /// When <c>true</c>, only packs with <c>featured = true</c> are returned.
        /// When <c>false</c> or <c>null</c>, no constraint is applied.
        /// </summary>
        public bool? Featured { get; set; }
    }

    // ---------------------------------------------------------------------------
    // Client
    // ---------------------------------------------------------------------------

    /// <summary>
    /// Fetches and caches the remote DINOForge pack registry, exposing search
    /// and filtering helpers for the installer and SDK consumers.
    /// </summary>
    /// <remarks>
    /// The client is safe to share across threads.  Results are cached in memory
    /// for <see cref="CacheDuration"/> (default: 1 hour) before the next network
    /// request is issued.
    /// </remarks>
    public sealed class PackRegistryClient : IDisposable
    {
        // ------------------------------------------------------------------
        // Singleton
        // ------------------------------------------------------------------

        private static readonly Lazy<PackRegistryClient> _instance =
            new Lazy<PackRegistryClient>(() => new PackRegistryClient(), LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>Shared singleton instance backed by the default registry URL.</summary>
        public static PackRegistryClient Instance => _instance.Value;

        // ------------------------------------------------------------------
        // Configuration
        // ------------------------------------------------------------------

        /// <summary>Default remote URL for the pack registry.</summary>
        public const string DefaultRegistryUrl = "https://kooshapari.github.io/Dino/registry.json";

        /// <summary>How long to keep fetched results before re-requesting the registry.</summary>
        public TimeSpan CacheDuration { get; set; } = TimeSpan.FromHours(1);

        private static readonly HttpClient SharedHttp = new();

        private readonly string _registryUrl;
        private readonly HttpClient _http;
        private readonly bool _ownsHttpClient;

        // ------------------------------------------------------------------
        // Cache
        // ------------------------------------------------------------------

        private readonly SemaphoreSlim _lock = new SemaphoreSlim(1, 1);
        private List<RegistryPackEntry>? _cache;
        private DateTime _cacheExpiry = DateTime.MinValue;
        private readonly TimeProvider _timeProvider;

        // ------------------------------------------------------------------
        // Constructors
        // ------------------------------------------------------------------

        /// <summary>
        /// Creates a client pointing at <see cref="DefaultRegistryUrl"/>.
        /// </summary>
        public PackRegistryClient()
            : this(DefaultRegistryUrl) { }

        /// <summary>
        /// Creates a client pointing at a custom registry URL.
        /// Useful for private or self-hosted registries.
        /// </summary>
        /// <param name="registryUrl">Full URL of the registry.json endpoint.</param>
        /// <param name="timeProvider">Optional TimeProvider for cache expiry checks (defaults to System).</param>
        public PackRegistryClient(string registryUrl, TimeProvider? timeProvider = null)
            : this(registryUrl, SharedHttp, timeProvider) { }

        /// <summary>
        /// Creates a client with an explicit <see cref="HttpClient"/> instance.
        /// Use this overload when you need to configure proxy, timeouts, or auth headers.
        /// </summary>
        /// <param name="registryUrl">Full URL of the registry.json endpoint.</param>
        /// <param name="httpClient">Pre-configured HttpClient to use for requests.</param>
        /// <param name="timeProvider">Optional TimeProvider for cache expiry checks (defaults to System).</param>
        public PackRegistryClient(string registryUrl, HttpClient httpClient, TimeProvider? timeProvider = null)
        {
            _registryUrl = registryUrl ?? throw new ArgumentNullException(nameof(registryUrl));
            _http = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
            _ownsHttpClient = !ReferenceEquals(httpClient, SharedHttp);
            _timeProvider = timeProvider ?? TimeProvider.System;
        }

        // ------------------------------------------------------------------
        // Core fetch
        // ------------------------------------------------------------------

        /// <summary>
        /// Returns all packs from the remote registry, using the in-memory cache
        /// when the cached data is still fresh.
        /// </summary>
        /// <param name="cancellationToken">Optional cancellation token.</param>
        /// <returns>Read-only list of all registry pack entries.</returns>
        public async Task<IReadOnlyList<RegistryPackEntry>> GetAllPacksAsync(
            CancellationToken cancellationToken = default)
        {
            await _lock.WaitAsync(cancellationToken);
            try
            {
                if (_cache != null && _timeProvider.GetUtcNow().UtcDateTime < _cacheExpiry)
                    return _cache.AsReadOnly();

                using HttpRequestMessage request = new(HttpMethod.Get, _registryUrl);
                using HttpResponseMessage response = await _http
                    .SendAsync(request, HttpCompletionOption.ResponseHeadersRead, cancellationToken);
                response.EnsureSuccessStatusCode();
                string json = await Task.Run(() => response.Content.ReadAsStringAsync().GetAwaiter().GetResult(), cancellationToken);

                RegistryDocument? doc = JsonSerializer.Deserialize<RegistryDocument>(json, JsonOptions.Default);

                _cache = doc?.Packs ?? new List<RegistryPackEntry>();
                _cacheExpiry = _timeProvider.GetUtcNow().UtcDateTime.Add(CacheDuration);

                return _cache.AsReadOnly();
            }
            finally
            {
                _lock.Release();
            }
        }

        // ------------------------------------------------------------------
        // Filtered search
        // ------------------------------------------------------------------

        /// <summary>
        /// Returns packs matching all constraints in <paramref name="filter"/>.
        /// A null filter (or a filter with all properties null) returns all packs.
        /// </summary>
        /// <param name="filter">Filtering criteria.  May be null.</param>
        /// <param name="cancellationToken">Optional cancellation token.</param>
        public async Task<IReadOnlyList<RegistryPackEntry>> SearchAsync(
            PackRegistryFilter? filter,
            CancellationToken cancellationToken = default)
        {
            IReadOnlyList<RegistryPackEntry> all = await GetAllPacksAsync(cancellationToken);

            if (filter == null)
                return all;

            IEnumerable<RegistryPackEntry> results = all;

            if (filter.Tags != null && filter.Tags.Count > 0)
            {
                results = results.Where(p =>
                    filter.Tags.All(t => p.Tags.Contains(t, StringComparer.OrdinalIgnoreCase)));
            }

            if (!string.IsNullOrWhiteSpace(filter.Type))
            {
                results = results.Where(p =>
                    string.Equals(p.Type, filter.Type, StringComparison.OrdinalIgnoreCase));
            }

            if (filter.Verified.HasValue)
            {
                results = results.Where(p => p.Verified == filter.Verified.Value);
            }

            if (filter.Featured.HasValue)
            {
                results = results.Where(p => p.Featured == filter.Featured.Value);
            }

            return results.ToList().AsReadOnly();
        }

        // ------------------------------------------------------------------
        // Convenience helpers
        // ------------------------------------------------------------------

        /// <summary>Returns all packs tagged with <paramref name="tag"/>.</summary>
        public Task<IReadOnlyList<RegistryPackEntry>> GetByTagAsync(
            string tag,
            CancellationToken cancellationToken = default) =>
            SearchAsync(new PackRegistryFilter { Tags = new[] { tag } }, cancellationToken);

        /// <summary>Returns all packs of a given <paramref name="type"/>.</summary>
        public Task<IReadOnlyList<RegistryPackEntry>> GetByTypeAsync(
            string type,
            CancellationToken cancellationToken = default) =>
            SearchAsync(new PackRegistryFilter { Type = type }, cancellationToken);

        /// <summary>Returns all verified packs.</summary>
        public Task<IReadOnlyList<RegistryPackEntry>> GetVerifiedAsync(
            CancellationToken cancellationToken = default) =>
            SearchAsync(new PackRegistryFilter { Verified = true }, cancellationToken);

        /// <summary>Returns all featured packs.</summary>
        public Task<IReadOnlyList<RegistryPackEntry>> GetFeaturedAsync(
            CancellationToken cancellationToken = default) =>
            SearchAsync(new PackRegistryFilter { Featured = true }, cancellationToken);

        /// <summary>
        /// Looks up a single pack by its exact <paramref name="id"/>.
        /// Returns null when not found.
        /// </summary>
        public async Task<RegistryPackEntry?> GetByIdAsync(
            string id,
            CancellationToken cancellationToken = default)
        {
            IReadOnlyList<RegistryPackEntry> all = await GetAllPacksAsync(cancellationToken);
            return all.FirstOrDefault(p => string.Equals(p.Id, id, StringComparison.OrdinalIgnoreCase));
        }

        /// <summary>
        /// Invalidates the in-memory cache, forcing the next call to re-fetch
        /// from the remote registry.
        /// </summary>
        // sync-over-async-unavoidable: public API contract (SDK-shipped)
        public void InvalidateCache()
        {
            // sync-over-async-unavoidable: SemaphoreSlim acquire in sync public API; non-async by contract
            _lock.Wait();
            try
            {
                _cache = null;
                _cacheExpiry = DateTime.MinValue;
            }
            finally
            {
                _lock.Release();
            }
        }

        /// <summary>
        /// Releases the cached HTTP client and synchronization primitive.
        /// </summary>
        public void Dispose()
        {
            _lock.Dispose();
            if (_ownsHttpClient)
            {
                _http.Dispose();
            }
        }
    }
}
