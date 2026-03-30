using System.Collections.Generic;
using CommunityToolkit.Mvvm.ComponentModel;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Represents a pack entry from a remote or local catalog for display in the Browse page.
    /// This is a flat DTO for UI binding and does not reference Runtime types.
    /// </summary>
    public sealed partial class CatalogEntry : ObservableObject
    {
        /// <summary>Unique pack identifier.</summary>
        public string Id { get; init; } = "";

        /// <summary>Human-readable pack name.</summary>
        public string Name { get; init; } = "";

        /// <summary>Semantic version string (e.g. "0.2.0").</summary>
        public string Version { get; init; } = "0.1.0";

        /// <summary>Pack author or organization.</summary>
        public string Author { get; init; } = "";

        /// <summary>Pack type: content, balance, ruleset, total_conversion, utility.</summary>
        public string Type { get; init; } = "content";

        /// <summary>Optional description of pack purpose.</summary>
        public string? Description { get; init; }

        /// <summary>List of pack IDs this pack depends on.</summary>
        public IReadOnlyList<string> DependsOn { get; init; } = System.Array.Empty<string>();

        /// <summary>List of pack IDs that conflict with this pack.</summary>
        public IReadOnlyList<string> ConflictsWith { get; init; } = System.Array.Empty<string>();

        /// <summary>Download URL for the pack archive (ZIP).</summary>
        public string DownloadUrl { get; init; } = "";

        /// <summary>File size in bytes, if known.</summary>
        public long FileSizeBytes { get; init; }

        /// <summary>Homepage or documentation URL.</summary>
        public string? HomepageUrl { get; init; }

        /// <summary>Whether this pack is already installed locally.</summary>
        [ObservableProperty]
        private bool _isInstalled;

        /// <summary>Whether an update is available for this pack.</summary>
        [ObservableProperty]
        private bool _hasUpdate;

        /// <summary>The installed version, if applicable.</summary>
        public string InstalledVersion { get; init; } = "";
    }
}
