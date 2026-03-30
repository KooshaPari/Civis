using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.Extensions.Logging;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Service for checking whether installed packs have available updates in a catalog.
    /// Performs version-by-version comparison using semantic versioning rules.
    /// </summary>
    public sealed class UpdateCheckService
    {
        private readonly ILogger<UpdateCheckService>? _logger;

        /// <summary>
        /// Initializes a new instance of <see cref="UpdateCheckService"/>.
        /// </summary>
        /// <param name="logger">Optional logger for diagnostic output.</param>
        public UpdateCheckService(ILogger<UpdateCheckService>? logger = null)
        {
            _logger = logger;
        }

        /// <summary>
        /// Compares installed packs against catalog entries and returns update information.
        /// </summary>
        /// <param name="installedPacks">Packs currently installed on the system.</param>
        /// <param name="catalogEntries">Pack entries available in the catalog.</param>
        /// <returns>A collection of update info for packs with newer versions available.</returns>
        public IReadOnlyList<UpdateInfo> CheckForUpdates(
            IReadOnlyList<PackViewModel> installedPacks,
            IReadOnlyList<CatalogEntry> catalogEntries)
        {
            if (installedPacks == null || catalogEntries == null)
            {
                _logger?.LogWarning("Update check received null arguments");
                return Array.Empty<UpdateInfo>();
            }

            _logger?.LogInformation(
                "Checking {InstalledCount} installed packs against {CatalogCount} catalog entries",
                installedPacks.Count,
                catalogEntries.Count);

            List<UpdateInfo> updates = new List<UpdateInfo>();

            // Build lookup for installed packs by ID
            Dictionary<string, PackViewModel> installedById = installedPacks
                .ToDictionary(p => p.Id, StringComparer.OrdinalIgnoreCase);

            foreach (CatalogEntry catalogEntry in catalogEntries)
            {
                if (!installedById.TryGetValue(catalogEntry.Id, out PackViewModel? installed))
                    continue;

                if (IsNewerVersion(catalogEntry.Version, installed.Version))
                {
                    updates.Add(new UpdateInfo
                    {
                        PackId = catalogEntry.Id,
                        PackName = catalogEntry.Name,
                        CurrentVersion = installed.Version,
                        AvailableVersion = catalogEntry.Version,
                        DownloadUrl = catalogEntry.DownloadUrl,
                        Description = catalogEntry.Description
                    });

                    _logger?.LogInformation(
                        "Update available for {PackId}: {Current} -> {Available}",
                        catalogEntry.Id,
                        installed.Version,
                        catalogEntry.Version);
                }
            }

            return updates.AsReadOnly();
        }

        /// <summary>
        /// Determines whether version <paramref name="newVersion"/> is newer than <paramref name="currentVersion"/>.
        /// Uses simple semantic versioning comparison (major.minor.patch).
        /// </summary>
        /// <param name="newVersion">The potential newer version string.</param>
        /// <param name="currentVersion">The current installed version string.</param>
        /// <returns>True if newVersion is strictly greater than currentVersion.</returns>
        public static bool IsNewerVersion(string newVersion, string currentVersion)
        {
            if (string.IsNullOrEmpty(newVersion) || string.IsNullOrEmpty(currentVersion))
                return false;

            int[] newParts = ParseVersion(newVersion);
            int[] currentParts = ParseVersion(currentVersion);

            for (int i = 0; i < 3; i++)
            {
                if (newParts[i] > currentParts[i])
                    return true;
                if (newParts[i] < currentParts[i])
                    return false;
            }

            return false;
        }

        /// <summary>
        /// Parses a semantic version string (e.g. "1.2.3") into an integer array.
        /// Missing components default to 0.
        /// </summary>
        private static int[] ParseVersion(string version)
        {
            string[] parts = version.TrimStart('v', 'V').Split('.');
            int[] result = new int[3];

            for (int i = 0; i < Math.Min(parts.Length, 3); i++)
            {
                if (int.TryParse(parts[i], out int component))
                    result[i] = component;
            }

            return result;
        }
    }

    /// <summary>
    /// Represents available update information for a single pack.
    /// </summary>
    public sealed class UpdateInfo
    {
        /// <summary>Unique pack identifier.</summary>
        public string PackId { get; init; } = "";

        /// <summary>Human-readable pack name.</summary>
        public string PackName { get; init; } = "";

        /// <summary>The currently installed version.</summary>
        public string CurrentVersion { get; init; } = "";

        /// <summary>The version available in the catalog.</summary>
        public string AvailableVersion { get; init; } = "";

        /// <summary>URL to download the updated pack archive.</summary>
        public string DownloadUrl { get; init; } = "";

        /// <summary>Optional description of the update.</summary>
        public string? Description { get; init; }
    }
}
