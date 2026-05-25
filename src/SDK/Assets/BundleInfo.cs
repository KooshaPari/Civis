using System;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Information about a Unity asset bundle file.
    /// </summary>
    public sealed class BundleInfo
    {
        /// <summary>Full filesystem path to the bundle file.</summary>
        public string Path { get; }

        /// <summary>Bundle file name (without directory).</summary>
        public string Name { get; }

        /// <summary>File size in bytes.</summary>
        public long SizeBytes { get; }

        /// <summary>Number of assets contained in the bundle.</summary>
        public int AssetCount { get; }

        /// <summary>
        /// Initializes a new <see cref="BundleInfo"/> instance.
        /// </summary>
        public BundleInfo(string path, string name, long sizeBytes, int assetCount)
        {
            Path = path ?? throw new ArgumentNullException(nameof(path));
            Name = name ?? throw new ArgumentNullException(nameof(name));
            SizeBytes = sizeBytes;
            AssetCount = assetCount;
        }
    }
}
