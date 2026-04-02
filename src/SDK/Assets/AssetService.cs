using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using AssetsTools.NET;
using System.Diagnostics.CodeAnalysis;
using AssetsTools.NET.Extra;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Service for reading and inspecting Unity asset bundles.
    /// Wraps AssetsTools.NET for DINOForge mod development.
    /// </summary>
    [ExcludeFromCodeCoverage] // Requires AssetsTools.NET native runtime — integration tests only
    public class AssetService : IDisposable
    {
        /// <summary>Expected Unity version for DINO (2021.3.45f2).</summary>
        public const string ExpectedUnityVersion = "2021.3";

        private readonly string _gameDir;
        private readonly AssetsManager _assetsManager;
        private readonly string _streamingAssetsDir;
        private bool _disposed;

        /// <summary>
        /// Initializes the asset service for the given game installation directory.
        /// </summary>
        /// <param name="gameDir">Root game installation directory (e.g. G:\SteamLibrary\...\Diplomacy is Not an Option).</param>
        /// <exception cref="ArgumentNullException">Thrown when <paramref name="gameDir"/> is null.</exception>
        public AssetService(string gameDir)
        {
            _gameDir = gameDir ?? throw new ArgumentNullException(nameof(gameDir));
            _assetsManager = new AssetsManager();
            _streamingAssetsDir = Path.Combine(
                gameDir,
                "Diplomacy is Not an Option_Data",
                "StreamingAssets",
                "aa",
                "StandaloneWindows64");
        }

        /// <summary>
        /// Lists all asset bundles in the game's Addressables directory.
        /// </summary>
        /// <returns>List of bundle information; empty if the directory does not exist.</returns>
        public IReadOnlyList<BundleInfo> ListBundles()
        {
            if (!Directory.Exists(_streamingAssetsDir))
            {
                return Array.Empty<BundleInfo>();
            }

            string[] bundleFiles = Directory.GetFiles(_streamingAssetsDir, "*.bundle");
            var results = new List<BundleInfo>(bundleFiles.Length);

            foreach (string bundlePath in bundleFiles)
            {
                FileInfo fileInfo = new FileInfo(bundlePath);
                int assetCount = 0;

                try
                {
                    assetCount = CountAssetsInBundle(bundlePath);
                }
                catch (Exception)
                {
                    // Bundle may be corrupt or locked; report 0 assets
                }

                results.Add(new BundleInfo(
                    path: bundlePath,
                    name: fileInfo.Name,
                    sizeBytes: fileInfo.Length,
                    assetCount: assetCount));
            }

            return results;
        }

        /// <summary>
        /// Lists assets within a specific bundle file.
        /// </summary>
        /// <param name="bundlePath">Full path to the .bundle file.</param>
        /// <returns>List of asset information from the bundle.</returns>
        /// <exception cref="FileNotFoundException">Thrown when the bundle file does not exist.</exception>
        public IReadOnlyList<AssetInfo> ListAssets(string bundlePath)
        {
            if (!File.Exists(bundlePath))
            {
                throw new FileNotFoundException("Bundle file not found.", bundlePath);
            }

            var results = new List<AssetInfo>();

            BundleFileInstance bundleInst = _assetsManager.LoadBundleFile(bundlePath);
            try
            {
                AssetsFileInstance assetsInst = _assetsManager.LoadAssetsFileFromBundle(bundleInst, 0);
                AssetsFile assetsFile = assetsInst.file;

                foreach (AssetFileInfo info in assetsFile.AssetInfos)
                {
                    string typeName = GetTypeName(assetsFile, info);
                    string assetName = TryGetAssetName(assetsInst, info);

                    results.Add(new AssetInfo(
                        name: assetName,
                        typeName: typeName,
                        pathId: info.PathId,
                        sizeBytes: info.ByteSize));
                }
            }
            finally
            {
                _assetsManager.UnloadAll(true);
            }

            return results;
        }

        /// <summary>
        /// Reads the Addressables catalog and returns asset key to bundle path mappings.
        /// </summary>
        /// <returns>Dictionary mapping asset keys to bundle file paths.</returns>
        public IReadOnlyDictionary<string, string> ReadCatalog()
        {
            string catalogPath = Path.Combine(
                _gameDir,
                "Diplomacy is Not an Option_Data",
                "StreamingAssets",
                "aa",
                "catalog.json");

            if (!File.Exists(catalogPath))
            {
                return new Dictionary<string, string>();
            }

            AddressablesCatalog catalog = AddressablesCatalog.Load(catalogPath);
            return catalog.KeyToBundleMap;
        }

        /// <summary>
        /// Extracts a specific asset's raw data from a bundle.
        /// </summary>
        /// <param name="bundlePath">Full path to the .bundle file.</param>
        /// <param name="assetName">Name or path ID of the asset to extract.</param>
        /// <returns>Raw asset data bytes, or null if not found.</returns>
        public byte[]? ExtractAsset(string bundlePath, string assetName)
        {
            if (!File.Exists(bundlePath))
            {
                return null;
            }

            BundleFileInstance bundleInst = _assetsManager.LoadBundleFile(bundlePath);
            try
            {
                AssetsFileInstance assetsInst = _assetsManager.LoadAssetsFileFromBundle(bundleInst, 0);
                AssetsFile assetsFile = assetsInst.file;

                // Try to find by path ID first (if assetName is numeric)
                if (long.TryParse(assetName, out long pathId))
                {
                    AssetFileInfo? info = assetsFile.AssetInfos.FirstOrDefault(a => a.PathId == pathId);
                    if (info != null)
                    {
                        return ReadAssetBytes(assetsFile, info);
                    }
                }

                // Try to find by name
                foreach (AssetFileInfo info in assetsFile.AssetInfos)
                {
                    string name = TryGetAssetName(assetsInst, info);
                    if (string.Equals(name, assetName, StringComparison.OrdinalIgnoreCase))
                    {
                        return ReadAssetBytes(assetsFile, info);
                    }
                }
            }
            finally
            {
                _assetsManager.UnloadAll(true);
            }

            return null;
        }

        /// <summary>
        /// Validates that a mod's asset bundle is compatible with the game.
        /// Checks Unity version compatibility and bundle integrity.
        /// </summary>
        /// <param name="modBundlePath">Path to the mod's asset bundle file.</param>
        /// <returns>Validation result with errors and asset listing.</returns>
        public AssetValidationResult ValidateModBundle(string modBundlePath)
        {
            if (!File.Exists(modBundlePath))
            {
                return AssetValidationResult.Failure(
                    new[] { $"Bundle file not found: {modBundlePath}" });
            }

            var errors = new List<string>();
            var assets = new List<AssetInfo>();
            string unityVersion = "unknown";

            try
            {
                BundleFileInstance bundleInst = _assetsManager.LoadBundleFile(modBundlePath);
                try
                {
                    AssetsFileInstance assetsInst = _assetsManager.LoadAssetsFileFromBundle(bundleInst, 0);
                    AssetsFile assetsFile = assetsInst.file;

                    // Check Unity version
                    unityVersion = assetsFile.Metadata.UnityVersion ?? "unknown";
                    if (!unityVersion.StartsWith(ExpectedUnityVersion, StringComparison.Ordinal))
                    {
                        errors.Add(
                            $"Unity version mismatch: bundle built with '{unityVersion}', " +
                            $"game expects '{ExpectedUnityVersion}.x'.");
                    }

                    // Enumerate assets
                    foreach (AssetFileInfo info in assetsFile.AssetInfos)
                    {
                        string typeName = GetTypeName(assetsFile, info);
                        string assetName = TryGetAssetName(assetsInst, info);

                        assets.Add(new AssetInfo(
                            name: assetName,
                            typeName: typeName,
                            pathId: info.PathId,
                            sizeBytes: info.ByteSize));
                    }

                    if (assets.Count == 0)
                    {
                        errors.Add("Bundle contains no assets.");
                    }
                }
                finally
                {
                    _assetsManager.UnloadAll(true);
                }
            }
            catch (Exception ex)
            {
                errors.Add($"Failed to read bundle: {ex.Message}");
            }

            return new AssetValidationResult(
                isValid: errors.Count == 0,
                unityVersion: unityVersion,
                errors: errors,
                assets: assets);
        }

        /// <summary>
        /// Gets the type name for an asset entry using the type tree or class ID mapping.
        /// </summary>
        private static string GetTypeName(AssetsFile assetsFile, AssetFileInfo info)
        {
            int typeIdOrIndex = info.TypeIdOrIndex;

            // In format >= 16, TypeIdOrIndex is an index into TypeTreeTypes
            if (typeIdOrIndex >= 0 && typeIdOrIndex < assetsFile.Metadata.TypeTreeTypes.Count)
            {
                TypeTreeType typeTreeType = assetsFile.Metadata.TypeTreeTypes[typeIdOrIndex];
                int classId = typeTreeType.TypeId;
                return GetClassIdName(classId);
            }

            // Fall back to using TypeId directly as a class ID
            return GetClassIdName(info.TypeId);
        }

        /// <summary>
        /// Attempts to read the m_Name field from an asset using AssetsManager.GetBaseField.
        /// Falls back to the path ID as a string.
        /// </summary>
        private string TryGetAssetName(AssetsFileInstance assetsInst, AssetFileInfo info)
        {
            try
            {
                AssetTypeValueField baseField = _assetsManager.GetBaseField(assetsInst, info);
                AssetTypeValueField nameField = baseField["m_Name"];
                if (nameField != null && !nameField.IsDummy)
                {
                    string name = nameField.AsString;
                    if (!string.IsNullOrEmpty(name))
                    {
                        return name;
                    }
                }
            }
            catch (Exception)
            {
                // Not all asset types have m_Name; fall through
            }

            return $"PathID_{info.PathId}";
        }

        /// <summary>
        /// Reads raw bytes for an asset from its file position.
        /// </summary>
        private static byte[] ReadAssetBytes(AssetsFile assetsFile, AssetFileInfo info)
        {
            assetsFile.Reader.Position = info.ByteOffset;
            return assetsFile.Reader.ReadBytes((int)info.ByteSize);
        }

        /// <summary>
        /// Counts assets in a bundle without fully loading all data.
        /// </summary>
        private int CountAssetsInBundle(string bundlePath)
        {
            BundleFileInstance bundleInst = _assetsManager.LoadBundleFile(bundlePath);
            try
            {
                AssetsFileInstance assetsInst = _assetsManager.LoadAssetsFileFromBundle(bundleInst, 0);
                return assetsInst.file.AssetInfos.Count;
            }
            finally
            {
                _assetsManager.UnloadAll(true);
            }
        }

        /// <summary>
        /// Maps well-known Unity class IDs to human-readable type names.
        /// </summary>
        private static string GetClassIdName(int classId)
        {
            switch (classId)
            {
                case 1: return "GameObject";
                case 2: return "Component";
                case 3: return "LevelGameManager";
                case 4: return "Transform";
                case 5: return "TimeManager";
                case 20: return "Camera";
                case 21: return "Material";
                case 23: return "MeshRenderer";
                case 25: return "Renderer";
                case 28: return "Texture2D";
                case 33: return "MeshFilter";
                case 43: return "Mesh";
                case 48: return "Shader";
                case 49: return "TextAsset";
                case 54: return "Rigidbody";
                case 65: return "BoxCollider";
                case 72: return "ComputeShader";
                case 74: return "AnimationClip";
                case 83: return "AudioClip";
                case 84: return "AudioListener";
                case 89: return "Cubemap";
                case 91: return "AnimatorController";
                case 95: return "Animator";
                case 102: return "SortingGroup";
                case 104: return "RenderTexture";
                case 108: return "Light";
                case 111: return "Animation";
                case 114: return "MonoBehaviour";
                case 115: return "MonoScript";
                case 128: return "Font";
                case 134: return "PhysicMaterial";
                case 136: return "SphereCollider";
                case 137: return "CapsuleCollider";
                case 143: return "SkinnedMeshRenderer";
                case 152: return "MovieTexture";
                case 156: return "Terrain";
                case 157: return "LightmapSettings";
                case 187: return "VideoClip";
                case 188: return "VideoPlayer";
                case 198: return "ParticleSystem";
                case 199: return "ParticleSystemRenderer";
                case 205: return "LODGroup";
                case 212: return "SpriteRenderer";
                case 213: return "Sprite";
                case 222: return "CanvasRenderer";
                case 223: return "Canvas";
                case 224: return "RectTransform";
                case 225: return "CanvasGroup";
                case 226: return "BillboardAsset";
                case 227: return "BillboardRenderer";
                case 228: return "SpeedTreeWindAsset";
                case 229: return "AnchoredJoint2D";
                case 236: return "SortingLayer";
                case 258: return "LightProbeGroup";
                case 271: return "SpriteAtlas";
                case 290: return "AssetBundle";
                default: return $"ClassID_{classId}";
            }
        }

        /// <summary>
        /// Replaces an asset's raw bytes in a bundle file, writing a new bundle to outputPath.
        /// Used by AssetSwapSystem to inject mod assets over vanilla ones at runtime.
        /// The source bundle is read-only; the patched result is written to <paramref name="outputPath"/>.
        /// </summary>
        /// <param name="bundlePath">Source bundle to read from.</param>
        /// <param name="assetName">Name or PathID (numeric string) of the asset to replace.</param>
        /// <param name="newData">New raw bytes to write for the asset.</param>
        /// <param name="outputPath">Destination path for the modified bundle.</param>
        /// <returns>True if the replacement succeeded; false on any error.</returns>
        public bool ReplaceAsset(string bundlePath, string assetName, byte[] newData, string outputPath)
        {
            if (!File.Exists(bundlePath))
            {
                LogWarning($"ReplaceAsset: source bundle not found: {bundlePath}");
                return false;
            }

            if (newData == null || newData.Length == 0)
            {
                LogWarning($"ReplaceAsset: newData is null or empty for asset '{assetName}'");
                return false;
            }

            try
            {
                // Use a dedicated manager so we don't pollute the shared instance.
                AssetsManager localManager = new AssetsManager();
                try
                {
                    BundleFileInstance bundleInst = localManager.LoadBundleFile(bundlePath, unpackIfPacked: false);

                    // Decompress the bundle in-memory so we can modify individual assets.
                    using (MemoryStream unpackedStream = new MemoryStream())
                    {
                        AssetsFileWriter unpackWriter = new AssetsFileWriter(unpackedStream);
                        bundleInst.file.Unpack(unpackWriter);
                        unpackedStream.Position = 0;

                        // Re-load from the unpacked stream.
                        BundleFileInstance unpackedBundle = localManager.LoadBundleFile(unpackedStream, bundlePath);
                        AssetsFileInstance assetsInst = localManager.LoadAssetsFileFromBundle(unpackedBundle, 0);
                        AssetsFile assetsFile = assetsInst.file;

                        // Find the target asset by PathID or by name.
                        AssetFileInfo? targetInfo = null;

                        if (long.TryParse(assetName, out long parsedPathId))
                        {
                            targetInfo = assetsFile.AssetInfos.FirstOrDefault(a => a.PathId == parsedPathId);
                        }

                        if (targetInfo == null)
                        {
                            foreach (AssetFileInfo info in assetsFile.AssetInfos)
                            {
                                string name = TryGetAssetName(assetsInst, info);
                                if (string.Equals(name, assetName, StringComparison.OrdinalIgnoreCase))
                                {
                                    targetInfo = info;
                                    break;
                                }
                            }
                        }

                        if (targetInfo == null)
                        {
                            LogWarning($"ReplaceAsset: asset '{assetName}' not found in bundle '{bundlePath}'");
                            return false;
                        }

                        // Replace the raw data on the AssetFileInfo.
                        targetInfo.SetNewData(newData);

                        // Write the modified assets file back into the bundle directory entry.
                        using (MemoryStream assetsStream = new MemoryStream())
                        {
                            AssetsFileWriter assetsWriter = new AssetsFileWriter(assetsStream);
                            assetsFile.Write(assetsWriter, 0);
                            assetsStream.Position = 0;
                            byte[] patchedAssetsBytes = assetsStream.ToArray();

                            // Replace the bundle directory entry with the patched assets bytes.
                            AssetBundleDirectoryInfo dirEntry =
                                unpackedBundle.file.BlockAndDirInfo.DirectoryInfos[0];
                            dirEntry.SetNewData(patchedAssetsBytes);
                        }

                        // Ensure output directory exists.
                        string? outputDir = Path.GetDirectoryName(outputPath);
                        if (!string.IsNullOrEmpty(outputDir) && !Directory.Exists(outputDir))
                        {
                            Directory.CreateDirectory(outputDir);
                        }

                        // Write the final patched bundle to disk.
                        using (FileStream outputStream = File.Create(outputPath))
                        {
                            AssetsFileWriter bundleWriter = new AssetsFileWriter(outputStream);
                            unpackedBundle.file.Write(bundleWriter, 0);
                        }
                    }
                }
                finally
                {
                    localManager.UnloadAll(true);
                }

                return true;
            }
            catch (Exception ex)
            {
                LogWarning($"ReplaceAsset: failed to patch '{assetName}' in '{bundlePath}': {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Lists all asset bundles in the game's Addressables directory that contain
        /// at least one asset whose type name matches <paramref name="typeName"/>.
        /// </summary>
        /// <param name="typeName">Unity type name to search for (e.g. "Texture2D", "Mesh").</param>
        /// <returns>Bundles that contain at least one asset of the requested type.</returns>
        public IReadOnlyList<BundleInfo> FindBundlesWithType(string typeName)
        {
            IReadOnlyList<BundleInfo> allBundles = ListBundles();
            var matching = new List<BundleInfo>();

            foreach (BundleInfo bundle in allBundles)
            {
                try
                {
                    IReadOnlyList<AssetInfo> assets = ListAssets(bundle.Path);
                    bool hasType = assets.Any(a =>
                        string.Equals(a.TypeName, typeName, StringComparison.OrdinalIgnoreCase));
                    if (hasType)
                    {
                        matching.Add(bundle);
                    }
                }
                catch (Exception ex)
                {
                    LogWarning($"FindBundlesWithType: could not read bundle '{bundle.Name}': {ex.Message}");
                }
            }

            return matching;
        }

        /// <summary>
        /// Writes a warning message to a local log file (best-effort; never throws).
        /// </summary>
        private static void LogWarning(string message)
        {
            try
            {
                string logPath = Path.Combine(
                    Path.GetTempPath(), "dinoforge_assetsvc.log");
                File.AppendAllText(logPath, $"[{DateTime.UtcNow:u}] WARN {message}\n");
            }
            catch { }
        }

        /// <inheritdoc/>
        public void Dispose()
        {
            if (!_disposed)
            {
                _assetsManager.UnloadAll(true);
                _disposed = true;
            }
        }
    }
}
