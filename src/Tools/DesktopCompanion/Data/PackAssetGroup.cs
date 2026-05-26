using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Represents a pack and its discovered asset bundles.
    /// Used to group bundles by pack in the Asset Browser.
    /// </summary>
    public sealed partial class PackAssetGroup : ObservableObject
    {
        /// <summary>Pack ID from pack.yaml.</summary>
        public string PackId { get; init; } = "";

        /// <summary>Human-readable pack name.</summary>
        public string PackName { get; init; } = "";

        /// <summary>Pack version.</summary>
        public string PackVersion { get; init; } = "";

        /// <summary>Absolute path to the pack directory.</summary>
        public string PackPath { get; init; } = "";

        /// <summary>List of bundle entries discovered for this pack.</summary>
        public ObservableCollection<BundleEntry> Bundles { get; } = new ObservableCollection<BundleEntry>();

        /// <summary>Total size of all bundles in bytes.</summary>
        public long TotalSizeBytes => CalculateTotalSize();

        /// <summary>Human-readable total size.</summary>
        public string TotalSizeFormatted => FormatBytes(TotalSizeBytes);

        /// <summary>Count of bundles in this pack.</summary>
        public int BundleCount => Bundles.Count;

        /// <summary>Total count of assets across all bundles.</summary>
        public int TotalAssetCount => CalculateTotalAssetCount();

        private long CalculateTotalSize()
        {
            long total = 0;
            foreach (BundleEntry bundle in Bundles)
                total += bundle.SizeBytes;
            return total;
        }

        private int CalculateTotalAssetCount()
        {
            int total = 0;
            foreach (BundleEntry bundle in Bundles)
                total += bundle.AssetCount;
            return total;
        }

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
