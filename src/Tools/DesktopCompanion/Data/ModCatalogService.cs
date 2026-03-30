using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Reads pack metadata from local folders or remote HTTP(S) catalog endpoints.
    /// Supports JSON catalog files with a simplified schema matching pack.yaml semantics.
    /// </summary>
    public sealed class ModCatalogService : IModCatalogService
    {
        private readonly ILogger<ModCatalogService>? _logger;
        private readonly HttpClient _httpClient;

        /// <summary>
        /// Initializes a new instance of <see cref="ModCatalogService"/>.
        /// </summary>
        /// <param name="logger">Optional logger for diagnostic output.</param>
        public ModCatalogService(ILogger<ModCatalogService>? logger = null)
        {
            _logger = logger;
            _httpClient = new HttpClient();
            _httpClient.Timeout = TimeSpan.FromSeconds(30);
        }

        /// <inheritdoc />
        public async Task<CatalogLoadResult> LoadCatalogAsync(string catalogUri)
        {
            if (string.IsNullOrWhiteSpace(catalogUri))
            {
                return new CatalogLoadResult
                {
                    SourceUri = catalogUri,
                    Errors = new[] { "Catalog URI is empty." }
                };
            }

            _logger?.LogInformation("Loading catalog from: {CatalogUri}", catalogUri);

            try
            {
                // Detect URI scheme
                if (catalogUri.StartsWith("http://", StringComparison.OrdinalIgnoreCase) ||
                    catalogUri.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
                {
                    return await LoadFromHttpAsync(catalogUri).ConfigureAwait(false);
                }

                // Handle file:// URIs or raw filesystem paths
                string path = catalogUri.StartsWith("file://", StringComparison.OrdinalIgnoreCase)
                    ? catalogUri.Substring(7).TrimStart('/')
                    : catalogUri;

                return LoadFromDirectory(path);
            }
            catch (Exception ex)
            {
                _logger?.LogError(ex, "Failed to load catalog from: {CatalogUri}", catalogUri);
                return new CatalogLoadResult
                {
                    SourceUri = catalogUri,
                    Errors = new[] { $"Failed to load catalog: {ex.Message}" }
                };
            }
        }

        private async Task<CatalogLoadResult> LoadFromHttpAsync(string url)
        {
            HttpResponseMessage response = await _httpClient.GetAsync(url).ConfigureAwait(false);
            response.EnsureSuccessStatusCode();

            string json = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
            return ParseJsonCatalog(json, url);
        }

        private CatalogLoadResult LoadFromDirectory(string directoryPath)
        {
            if (!Directory.Exists(directoryPath))
            {
                return new CatalogLoadResult
                {
                    SourceUri = directoryPath,
                    Errors = new[] { $"Directory not found: {directoryPath}" }
                };
            }

            List<CatalogEntry> entries = new List<CatalogEntry>();
            List<string> errors = new List<string>();

            foreach (string subDir in Directory.GetDirectories(directoryPath))
            {
                string dirName = Path.GetFileName(subDir);

                // Skip archived/hidden directories
                if (dirName.StartsWith("_") || dirName.StartsWith("."))
                    continue;

                string packYamlPath = Path.Combine(subDir, "pack.yaml");
                if (!File.Exists(packYamlPath))
                    continue;

                try
                {
                    string yaml = File.ReadAllText(packYamlPath);
                    CatalogEntry? entry = TryParseYamlPack(yaml, dirName);
                    if (entry != null)
                        entries.Add(entry);
                }
                catch (Exception ex)
                {
                    errors.Add($"Failed to parse {packYamlPath}: {ex.Message}");
                }
            }

            return new CatalogLoadResult
            {
                SourceUri = directoryPath,
                Entries = entries.AsReadOnly(),
                Errors = errors.AsReadOnly()
            };
        }

        private static CatalogEntry? TryParseYamlPack(string yaml, string fallbackId)
        {
            // Minimal YAML parse using YamlDotNet to extract fields
            try
            {
                var deserializer = new YamlDotNet.Serialization.DeserializerBuilder()
                    .WithNamingConvention(YamlDotNet.Serialization.NamingConventions.UnderscoredNamingConvention.Instance)
                    .IgnoreUnmatchedProperties()
                    .Build();

                object? raw = deserializer.Deserialize<object>(yaml);
                if (raw is not IDictionary<object, object> dict)
                    return null;

                string id = GetString(dict, "id") ?? fallbackId;
                string version = GetString(dict, "version") ?? "0.1.0";

                return new CatalogEntry
                {
                    Id = id,
                    Name = GetString(dict, "name") ?? id,
                    Version = version,
                    Author = GetString(dict, "author") ?? "",
                    Type = GetString(dict, "type") ?? "content",
                    Description = GetString(dict, "description"),
                    DependsOn = GetStringList(dict, "depends_on"),
                    ConflictsWith = GetStringList(dict, "conflicts_with"),
                    DownloadUrl = GetString(dict, "download_url") ?? "",
                    HomepageUrl = GetString(dict, "homepage_url"),
                    FileSizeBytes = GetLong(dict, "file_size_bytes")
                };
            }
            catch
            {
                return null;
            }
        }

        private CatalogLoadResult ParseJsonCatalog(string json, string sourceUri)
        {
            List<CatalogEntry> entries = new List<CatalogEntry>();
            List<string> errors = new List<string>();

            try
            {
                JObject? root = JObject.Parse(json);
                JArray? packs = root["packs"] as JArray;

                if (packs == null)
                {
                    return new CatalogLoadResult
                    {
                        SourceUri = sourceUri,
                        Errors = new[] { "Invalid catalog format: 'packs' array not found." }
                    };
                }

                foreach (JToken? token in packs)
                {
                    if (token is not JObject obj)
                        continue;

                    try
                    {
                        CatalogEntry entry = new CatalogEntry
                        {
                            Id = obj.Value<string>("id") ?? "",
                            Name = obj.Value<string>("name") ?? obj.Value<string>("id") ?? "",
                            Version = obj.Value<string>("version") ?? "0.1.0",
                            Author = obj.Value<string>("author") ?? "",
                            Type = obj.Value<string>("type") ?? "content",
                            Description = obj.Value<string>("description"),
                            DownloadUrl = obj.Value<string>("download_url") ?? "",
                            HomepageUrl = obj.Value<string>("homepage_url"),
                            FileSizeBytes = obj.Value<long?>("file_size_bytes") ?? 0,
                            DependsOn = ParseStringArray(obj["depends_on"]),
                            ConflictsWith = ParseStringArray(obj["conflicts_with"])
                        };

                        entries.Add(entry);
                    }
                    catch (Exception ex)
                    {
                        errors.Add($"Failed to parse catalog entry: {ex.Message}");
                    }
                }
            }
            catch (JsonException ex)
            {
                errors.Add($"JSON parse error: {ex.Message}");
            }

            return new CatalogLoadResult
            {
                SourceUri = sourceUri,
                Entries = entries.AsReadOnly(),
                Errors = errors.AsReadOnly()
            };
        }

        private static string? GetString(IDictionary<object, object> dict, string key)
        {
            if (dict.TryGetValue(key, out object? value))
                return value?.ToString();
            return null;
        }

        private static long GetLong(IDictionary<object, object> dict, string key)
        {
            if (dict.TryGetValue(key, out object? value) && long.TryParse(value?.ToString(), out long result))
                return result;
            return 0;
        }

        private static IReadOnlyList<string> GetStringList(IDictionary<object, object> dict, string key)
        {
            if (dict.TryGetValue(key, out object? value) && value is IList<object> list)
                return list.Select(o => o?.ToString() ?? "").ToList().AsReadOnly();
            return System.Array.Empty<string>();
        }

        private static IReadOnlyList<string> ParseStringArray(JToken? token)
        {
            if (token is JArray arr)
                return arr.Select(t => t.Value<string>() ?? "").ToList().AsReadOnly();
            return System.Array.Empty<string>();
        }
    }
}
