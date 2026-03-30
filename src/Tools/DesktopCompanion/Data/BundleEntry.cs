using System.Collections.Generic;
using CommunityToolkit.Mvvm.ComponentModel;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Represents a single asset bundle file discovered in a pack's assets/bundles directory.
    /// </summary>
    public sealed partial class BundleEntry : ObservableObject
    {
        /// <summary>Bundle file name (without path).</summary>
        public string FileName { get; init; } = "";

        /// <summary>Size in bytes.</summary>
        public long SizeBytes { get; init; }

        /// <summary>Human-readable size (e.g., "1.3 MB").</summary>
        public string SizeFormatted => FormatBytes(SizeBytes);

        /// <summary>Number of assets in this bundle (estimated from manifest or 0 if unknown).</summary>
        public int AssetCount { get; init; }

        /// <summary>Manifest file path (if present).</summary>
        public string? ManifestPath { get; init; }

        /// <summary>Full path to the bundle file.</summary>
        public string? FullPath { get; init; }

        private static string FormatBytes(long bytes)
        {
            const long kb = 1024;
            const long mb = kb * 1024;
            const long gb = mb * 1024;

            return bytes switch
            {
                >= gb => $"{bytes / (double)gb:F2} GB",
                >= mb => $"{bytes / (double)mb:F2} MB",
                >= kb => $"{bytes / (double)kb:F2} KB",
                _ => $"{bytes} B"
            };
        }
    }
}
