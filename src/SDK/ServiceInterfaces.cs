#nullable enable
using System.Collections.Generic;

namespace DINOForge.SDK
{
    public interface IFileDiscoveryService { }
    public interface ISchemaResolverService { }
    public interface IRegistryImportService { }
    public interface IPackReloadService { }
    public interface IContentDiscoveryService { }
    public interface ISchemaValidator { }
    public interface IPackRootResolver { }

    public sealed class BundleInfo
    {
        public string Path { get; }
        public string Name { get; }
        public long SizeBytes { get; }
        public int AssetCount { get; }

        public BundleInfo(string path, string name, long sizeBytes, int assetCount)
        {
            Path = path;
            Name = name;
            SizeBytes = sizeBytes;
            AssetCount = assetCount;
        }
    }

    public sealed class AssetInfo
    {
        public string Name { get; }
        public string TypeName { get; }
        public long PathId { get; }
        public long SizeBytes { get; }

        public AssetInfo(string name, string typeName, long pathId, long sizeBytes)
        {
            Name = name;
            TypeName = typeName;
            PathId = pathId;
            SizeBytes = sizeBytes;
        }
    }

    public sealed class AssetValidationResult
    {
        public bool IsValid { get; }
        public string UnityVersion { get; }
        public IReadOnlyList<string> Errors { get; }
        public IReadOnlyList<AssetInfo> Assets { get; }

        public AssetValidationResult(bool isValid, string unityVersion, IReadOnlyList<string>? errors = null, IReadOnlyList<AssetInfo>? assets = null)
        {
            IsValid = isValid;
            UnityVersion = unityVersion;
            Errors = errors ?? System.Array.Empty<string>();
            Assets = assets ?? System.Array.Empty<AssetInfo>();
        }

        public static AssetValidationResult Failure(params string[] errors)
            => new AssetValidationResult(false, "", errors);
    }

    public sealed class HotReloadResult
    {
        public bool IsSuccess { get; }
        public IReadOnlyList<string> Errors { get; }

        public HotReloadResult(bool success, IReadOnlyList<string>? errors = null)
        {
            IsSuccess = success;
            Errors = errors ?? System.Array.Empty<string>();
        }
    }

    public sealed class DependencyResult
    {
        public bool IsSuccess { get; }
        public IReadOnlyList<PackManifest> LoadOrder { get; }
        public IReadOnlyList<string> Errors { get; }

        private DependencyResult(bool isSuccess, IReadOnlyList<PackManifest>? loadOrder, IReadOnlyList<string>? errors)
        {
            IsSuccess = isSuccess;
            LoadOrder = loadOrder ?? System.Array.Empty<PackManifest>();
            Errors = errors ?? System.Array.Empty<string>();
        }

        public static DependencyResult Success(IReadOnlyList<PackManifest> loadOrder)
            => new DependencyResult(true, loadOrder, null);

        public static DependencyResult Failure(IReadOnlyList<string> errors)
            => new DependencyResult(false, null, errors);
    }
}
