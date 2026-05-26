using System;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Information about a single asset within a Unity asset bundle.
    /// </summary>
    public sealed class AssetInfo
    {
        /// <summary>Asset name or path within the bundle.</summary>
        public string Name { get; }

        /// <summary>Unity type name (e.g. "Texture2D", "Mesh", "GameObject").</summary>
        public string TypeName { get; }

        /// <summary>Unique path ID of the asset within the bundle.</summary>
        public long PathId { get; }

        /// <summary>Approximate size of the asset in bytes.</summary>
        public long SizeBytes { get; }

        /// <summary>
        /// Initializes a new <see cref="AssetInfo"/> instance.
        /// </summary>
        public AssetInfo(string name, string typeName, long pathId, long sizeBytes)
        {
            Name = name ?? throw new ArgumentNullException(nameof(name));
            TypeName = typeName ?? throw new ArgumentNullException(nameof(typeName));
            PathId = pathId;
            SizeBytes = sizeBytes;
        }
    }
}
