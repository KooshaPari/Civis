#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Microsoft.Data.Sqlite;
using Microsoft.Extensions.Logging;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// SQLite-backed persistent catalog store for asset library browser.
/// Stores asset metadata including id, name, faction, type, source_url, status, provenance, pack_id, created_at.
/// </summary>
public sealed class AssetCatalogStore : IDisposable
{
    private readonly string _connectionString;
    private readonly string _dbPath;
    private readonly ILogger<AssetCatalogStore>? _logger;
    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true
    };

    /// <summary>
    /// Initializes a new instance of the <see cref="AssetCatalogStore"/> class.
    /// </summary>
    /// <param name="dbPath">Path to the SQLite database file.</param>
    /// <param name="logger">Optional logger for diagnostics.</param>
    public AssetCatalogStore(string dbPath, ILogger<AssetCatalogStore>? logger = null)
    {
        _dbPath = dbPath ?? throw new ArgumentNullException(nameof(dbPath));
        _logger = logger;
        _connectionString = $"Data Source={dbPath}";
        InitializeDatabase();
    }

    /// <summary>
    /// Adds a new asset to the catalog store.
    /// </summary>
    /// <param name="asset">The asset catalog entry to add.</param>
    /// <returns>True if the asset was added successfully; otherwise, false if it already exists.</returns>
    public bool AddAsset(AssetCatalogEntry asset)
    {
        if (asset is null)
        {
            throw new ArgumentNullException(nameof(asset));
        }

        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            cmd.CommandText = @"
                INSERT OR IGNORE INTO assets (asset_id, name, faction, type, source_url, status, provenance, pack_id, metadata_json, created_at)
                VALUES (@assetId, @name, @faction, @type, @sourceUrl, @status, @provenance, @packId, @metadataJson, @createdAt)";

            cmd.Parameters.AddWithValue("@assetId", asset.AssetId);
            cmd.Parameters.AddWithValue("@name", asset.Name ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@faction", asset.Faction ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@type", asset.Type ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@sourceUrl", asset.SourceUrl ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@status", asset.Status ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@provenance", asset.Provenance ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@packId", asset.PackId ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@metadataJson", asset.MetadataJson ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@createdAt", asset.CreatedAt.ToString("O"));

            int rowsAffected = cmd.ExecuteNonQuery();
            bool inserted = rowsAffected > 0;

            if (inserted)
            {
                _logger?.LogInformation("Added asset {AssetId} to catalog store", asset.AssetId);
            }

            return inserted;
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to add asset {AssetId} to catalog store", asset.AssetId);
            return false;
        }
    }

    /// <summary>
    /// Retrieves all assets from the catalog store.
    /// </summary>
    /// <returns>A list of all asset catalog entries.</returns>
    public IReadOnlyList<AssetCatalogEntry> GetAssets()
    {
        var results = new List<AssetCatalogEntry>();

        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            cmd.CommandText = "SELECT * FROM assets ORDER BY created_at DESC";

            using var reader = cmd.ExecuteReader();
            while (reader.Read())
            {
                results.Add(ReadAssetEntry(reader));
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to retrieve assets from catalog store");
        }

        return results;
    }

    /// <summary>
    /// Searches assets with optional filters.
    /// </summary>
    /// <param name="query">Optional search query for name or title.</param>
    /// <param name="faction">Optional faction filter.</param>
    /// <param name="type">Optional type filter.</param>
    /// <param name="status">Optional status filter.</param>
    /// <returns>A list of matching asset catalog entries.</returns>
    public IReadOnlyList<AssetCatalogEntry> SearchAssets(string? query, string? faction, string? type, string? status)
    {
        var results = new List<AssetCatalogEntry>();

        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            var conditions = new List<string>();
            var parameters = new List<SqliteParameter>();

            if (!string.IsNullOrWhiteSpace(query))
            {
                conditions.Add("(name LIKE @query OR asset_id LIKE @query)");
                cmd.Parameters.AddWithValue("@query", $"%{query}%");
            }

            if (!string.IsNullOrWhiteSpace(faction))
            {
                conditions.Add("faction = @faction");
                cmd.Parameters.AddWithValue("@faction", faction);
            }

            if (!string.IsNullOrWhiteSpace(type))
            {
                conditions.Add("type = @type");
                cmd.Parameters.AddWithValue("@type", type);
            }

            if (!string.IsNullOrWhiteSpace(status))
            {
                conditions.Add("status = @status");
                cmd.Parameters.AddWithValue("@status", status);
            }

            string whereClause = conditions.Count > 0 ? $"WHERE {string.Join(" AND ", conditions)}" : "";
            cmd.CommandText = $"SELECT * FROM assets {whereClause} ORDER BY created_at DESC";

            using var reader = cmd.ExecuteReader();
            while (reader.Read())
            {
                results.Add(ReadAssetEntry(reader));
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to search assets in catalog store");
        }

        return results;
    }

    /// <summary>
    /// Gets assets by pack ID.
    /// </summary>
    /// <param name="packId">The pack identifier.</param>
    /// <returns>A list of asset catalog entries for the specified pack.</returns>
    public IReadOnlyList<AssetCatalogEntry> GetByPack(string packId)
    {
        var results = new List<AssetCatalogEntry>();

        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            cmd.CommandText = "SELECT * FROM assets WHERE pack_id = @packId ORDER BY created_at DESC";
            cmd.Parameters.AddWithValue("@packId", packId);

            using var reader = cmd.ExecuteReader();
            while (reader.Read())
            {
                results.Add(ReadAssetEntry(reader));
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to get assets by pack {PackId}", packId);
        }

        return results;
    }

    /// <summary>
    /// Gets a single asset by ID.
    /// </summary>
    /// <param name="assetId">The asset identifier.</param>
    /// <returns>The asset catalog entry if found; otherwise, null.</returns>
    public AssetCatalogEntry? GetById(string assetId)
    {
        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            cmd.CommandText = "SELECT * FROM assets WHERE asset_id = @assetId";
            cmd.Parameters.AddWithValue("@assetId", assetId);

            using var reader = cmd.ExecuteReader();
            if (reader.Read())
            {
                return ReadAssetEntry(reader);
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to get asset {AssetId}", assetId);
        }

        return null;
    }

    /// <summary>
    /// Updates an existing asset in the catalog store.
    /// </summary>
    /// <param name="asset">The asset catalog entry with updated values.</param>
    /// <returns>True if the asset was updated successfully; otherwise, false.</returns>
    public bool UpdateAsset(AssetCatalogEntry asset)
    {
        if (asset is null)
        {
            throw new ArgumentNullException(nameof(asset));
        }

        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            cmd.CommandText = @"
                UPDATE assets SET
                    name = @name,
                    faction = @faction,
                    type = @type,
                    source_url = @sourceUrl,
                    status = @status,
                    provenance = @provenance,
                    pack_id = @packId,
                    metadata_json = @metadataJson
                WHERE asset_id = @assetId";

            cmd.Parameters.AddWithValue("@assetId", asset.AssetId);
            cmd.Parameters.AddWithValue("@name", asset.Name ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@faction", asset.Faction ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@type", asset.Type ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@sourceUrl", asset.SourceUrl ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@status", asset.Status ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@provenance", asset.Provenance ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@packId", asset.PackId ?? (object)DBNull.Value);
            cmd.Parameters.AddWithValue("@metadataJson", asset.MetadataJson ?? (object)DBNull.Value);

            int rowsAffected = cmd.ExecuteNonQuery();
            return rowsAffected > 0;
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to update asset {AssetId}", asset.AssetId);
            return false;
        }
    }

    /// <summary>
    /// Gets catalog statistics.
    /// </summary>
    /// <returns>A dictionary of statistics by category.</returns>
    public AssetCatalogStats GetStats()
    {
        var stats = new AssetCatalogStats();

        try
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            // Total count
            using (var cmd = connection.CreateCommand())
            {
                cmd.CommandText = "SELECT COUNT(*) FROM assets";
                stats.TotalCount = Convert.ToInt32(cmd.ExecuteScalar());
            }

            // Count by faction
            using (var cmd = connection.CreateCommand())
            {
                cmd.CommandText = "SELECT faction, COUNT(*) FROM assets WHERE faction IS NOT NULL GROUP BY faction";
                using var reader = cmd.ExecuteReader();
                while (reader.Read())
                {
                    string? faction = reader.GetString(0);
                    if (!string.IsNullOrEmpty(faction))
                    {
                        stats.CountByFaction[faction] = reader.GetInt32(1);
                    }
                }
            }

            // Count by type
            using (var cmd = connection.CreateCommand())
            {
                cmd.CommandText = "SELECT type, COUNT(*) FROM assets WHERE type IS NOT NULL GROUP BY type";
                using var reader = cmd.ExecuteReader();
                while (reader.Read())
                {
                    string? type = reader.GetString(0);
                    if (!string.IsNullOrEmpty(type))
                    {
                        stats.CountByType[type] = reader.GetInt32(1);
                    }
                }
            }

            // Count by status
            using (var cmd = connection.CreateCommand())
            {
                cmd.CommandText = "SELECT status, COUNT(*) FROM assets WHERE status IS NOT NULL GROUP BY status";
                using var reader = cmd.ExecuteReader();
                while (reader.Read())
                {
                    string? status = reader.GetString(0);
                    if (!string.IsNullOrEmpty(status))
                    {
                        stats.CountByStatus[status] = reader.GetInt32(1);
                    }
                }
            }

            // Count by pack
            using (var cmd = connection.CreateCommand())
            {
                cmd.CommandText = "SELECT pack_id, COUNT(*) FROM assets WHERE pack_id IS NOT NULL GROUP BY pack_id";
                using var reader = cmd.ExecuteReader();
                while (reader.Read())
                {
                    string? packId = reader.GetString(0);
                    if (!string.IsNullOrEmpty(packId))
                    {
                        stats.CountByPack[packId] = reader.GetInt32(1);
                    }
                }
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to get catalog statistics");
        }

        return stats;
    }

    /// <summary>
    /// Exports all assets to a JSON file for portability.
    /// </summary>
    /// <param name="exportPath">Path to the export JSON file.</param>
    /// <returns>True if export was successful; otherwise, false.</returns>
    public bool ExportToJson(string exportPath)
    {
        try
        {
            var assets = GetAssets();
            string json = JsonSerializer.Serialize(new { assets }, _jsonOptions);
            File.WriteAllText(exportPath, json);
            _logger?.LogInformation("Exported {Count} assets to {Path}", assets.Count, exportPath);
            return true;
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to export assets to JSON");
            return false;
        }
    }

    /// <summary>
    /// Imports assets from a JSON export file.
    /// </summary>
    /// <param name="importPath">Path to the import JSON file.</param>
    /// <returns>Number of assets imported.</returns>
    public int ImportFromJson(string importPath)
    {
        int count = 0;

        try
        {
            string json = File.ReadAllText(importPath);
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            if (root.TryGetProperty("assets", out var assetsElement))
            {
                foreach (var assetElement in assetsElement.EnumerateArray())
                {
                    var asset = JsonSerializer.Deserialize<AssetCatalogEntry>(assetElement.GetRawText(), _jsonOptions);
                    if (asset is not null && AddAsset(asset))
                    {
                        count++;
                    }
                }
            }

            _logger?.LogInformation("Imported {Count} assets from {Path}", count, importPath);
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to import assets from JSON");
        }

        return count;
    }

    private void InitializeDatabase()
    {
        try
        {
            // Ensure directory exists
            string? directory = Path.GetDirectoryName(_dbPath);
            if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
            {
                Directory.CreateDirectory(directory);
            }

            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            using var cmd = connection.CreateCommand();
            cmd.CommandText = @"
                CREATE TABLE IF NOT EXISTS assets (
                    asset_id TEXT PRIMARY KEY,
                    name TEXT,
                    faction TEXT,
                    type TEXT,
                    source_url TEXT,
                    status TEXT,
                    provenance TEXT,
                    pack_id TEXT,
                    metadata_json TEXT,
                    created_at TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_assets_faction ON assets(faction);
                CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(type);
                CREATE INDEX IF NOT EXISTS idx_assets_status ON assets(status);
                CREATE INDEX IF NOT EXISTS idx_assets_pack_id ON assets(pack_id);
            ";
            cmd.ExecuteNonQuery();

            _logger?.LogInformation("Initialized catalog store database at {Path}", _dbPath);
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Failed to initialize catalog store database at {Path}", _dbPath);
            throw;
        }
    }

    private static AssetCatalogEntry ReadAssetEntry(SqliteDataReader reader)
    {
        return new AssetCatalogEntry
        {
            AssetId = reader.GetString(reader.GetOrdinal("asset_id")),
            Name = reader.IsDBNull(reader.GetOrdinal("name")) ? null : reader.GetString(reader.GetOrdinal("name")),
            Faction = reader.IsDBNull(reader.GetOrdinal("faction")) ? null : reader.GetString(reader.GetOrdinal("faction")),
            Type = reader.IsDBNull(reader.GetOrdinal("type")) ? null : reader.GetString(reader.GetOrdinal("type")),
            SourceUrl = reader.IsDBNull(reader.GetOrdinal("source_url")) ? null : reader.GetString(reader.GetOrdinal("source_url")),
            Status = reader.IsDBNull(reader.GetOrdinal("status")) ? null : reader.GetString(reader.GetOrdinal("status")),
            Provenance = reader.IsDBNull(reader.GetOrdinal("provenance")) ? null : reader.GetString(reader.GetOrdinal("provenance")),
            PackId = reader.IsDBNull(reader.GetOrdinal("pack_id")) ? null : reader.GetString(reader.GetOrdinal("pack_id")),
            MetadataJson = reader.IsDBNull(reader.GetOrdinal("metadata_json")) ? null : reader.GetString(reader.GetOrdinal("metadata_json")),
            CreatedAt = DateTime.Parse(reader.GetString(reader.GetOrdinal("created_at")))
        };
    }

    /// <summary>
    /// Disposes of the catalog store resources.
    /// </summary>
    public void Dispose()
    {
        // SQLite connections are managed per-call, no persistent resources to dispose
    }
}

/// <summary>
/// Represents an asset catalog entry stored in the SQLite catalog.
/// </summary>
public sealed class AssetCatalogEntry
{
    /// <summary>
    /// Stable asset identifier.
    /// </summary>
    public string AssetId { get; set; } = string.Empty;

    /// <summary>
    /// Human-readable name or title.
    /// </summary>
    public string? Name { get; set; }

    /// <summary>
    /// Faction classification (e.g., republic, cis, neutral).
    /// </summary>
    public string? Faction { get; set; }

    /// <summary>
    /// Asset type (e.g., unit, vehicle, building, prop).
    /// </summary>
    public string? Type { get; set; }

    /// <summary>
    /// Source URL where the asset was discovered.
    /// </summary>
    public string? SourceUrl { get; set; }

    /// <summary>
    /// Technical or IP status of the asset.
    /// </summary>
    public string? Status { get; set; }

    /// <summary>
    /// Provenance information for the asset.
    /// </summary>
    public string? Provenance { get; set; }

    /// <summary>
    /// Associated pack identifier.
    /// </summary>
    public string? PackId { get; set; }

    /// <summary>
    /// Additional metadata as JSON string.
    /// </summary>
    public string? MetadataJson { get; set; }

    /// <summary>
    /// Timestamp when the asset was added to the catalog.
    /// </summary>
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
}

/// <summary>
/// Statistics about the asset catalog.
/// </summary>
public sealed class AssetCatalogStats
{
    /// <summary>
    /// Total number of assets in the catalog.
    /// </summary>
    public int TotalCount { get; set; }

    /// <summary>
    /// Asset count by faction.
    /// </summary>
    public Dictionary<string, int> CountByFaction { get; set; } = new();

    /// <summary>
    /// Asset count by type.
    /// </summary>
    public Dictionary<string, int> CountByType { get; set; } = new();

    /// <summary>
    /// Asset count by status.
    /// </summary>
    public Dictionary<string, int> CountByStatus { get; set; } = new();

    /// <summary>
    /// Asset count by pack.
    /// </summary>
    public Dictionary<string, int> CountByPack { get; set; } = new();
}
