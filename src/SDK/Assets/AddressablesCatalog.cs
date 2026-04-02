using System;
using System.Collections.Generic;
using System.IO;
using Newtonsoft.Json;
using System.Diagnostics.CodeAnalysis;
using Newtonsoft.Json.Linq;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Parses Unity Addressables catalog.json to extract asset key to bundle mappings.
    /// The catalog format uses m_InternalIds (array of bundle/asset paths),
    /// m_KeyDataString, m_BucketDataString, and m_EntryDataString (Base64-encoded
    /// binary data) to map string keys to internal ID indices.
    /// </summary>
    [ExcludeFromCodeCoverage] // Requires real Unity catalog.json — integration tests only
    public sealed class AddressablesCatalog
    {
        private readonly Dictionary<string, string> _keyToBundleMap;
        private readonly List<string> _internalIds;
        private readonly List<string> _bundlePaths;

        /// <summary>All internal IDs from the catalog (both asset keys and bundle paths).</summary>
        public IReadOnlyList<string> InternalIds => _internalIds;

        /// <summary>All bundle file paths referenced in the catalog.</summary>
        public IReadOnlyList<string> BundlePaths => _bundlePaths;

        /// <summary>Mapping of asset keys to their containing bundle paths.</summary>
        public IReadOnlyDictionary<string, string> KeyToBundleMap => _keyToBundleMap;

        private AddressablesCatalog(List<string> internalIds, List<string> bundlePaths, Dictionary<string, string> keyToBundleMap)
        {
            _internalIds = internalIds;
            _bundlePaths = bundlePaths;
            _keyToBundleMap = keyToBundleMap;
        }

        /// <summary>
        /// Loads and parses an Addressables catalog.json file.
        /// </summary>
        /// <param name="catalogPath">Path to catalog.json.</param>
        /// <returns>Parsed catalog with asset key to bundle mappings.</returns>
        /// <exception cref="FileNotFoundException">Thrown when the catalog file does not exist.</exception>
        /// <exception cref="InvalidOperationException">Thrown when the catalog format is invalid.</exception>
        public static AddressablesCatalog Load(string catalogPath)
        {
            if (!File.Exists(catalogPath))
            {
                throw new FileNotFoundException("Addressables catalog not found.", catalogPath);
            }

            string json = File.ReadAllText(catalogPath);
            JObject catalog = JObject.Parse(json);

            // Extract m_InternalIds — contains both bundle paths and asset paths
            JArray? internalIdsArray = catalog["m_InternalIds"] as JArray;
            if (internalIdsArray == null)
            {
                throw new InvalidOperationException("Catalog missing m_InternalIds field.");
            }

            var internalIds = new List<string>();
            var bundlePaths = new List<string>();
            var bundleIndices = new HashSet<int>();

            for (int i = 0; i < internalIdsArray.Count; i++)
            {
                string id = internalIdsArray[i]?.ToString() ?? string.Empty;
                internalIds.Add(id);

                if (id.EndsWith(".bundle", StringComparison.OrdinalIgnoreCase))
                {
                    bundlePaths.Add(id);
                    bundleIndices.Add(i);
                }
            }

            // Build a simplified key-to-bundle map.
            // The catalog's binary-encoded bucket/entry/key data is complex to decode.
            // We use a practical approach: non-bundle internal IDs are asset keys,
            // and we map them to bundles based on entry data when possible,
            // falling back to a simple heuristic for the common case.
            var keyToBundleMap = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

            // Try to decode the entry data to get proper mappings
            string? entryDataString = catalog["m_EntryDataString"]?.ToString();
            if (!string.IsNullOrEmpty(entryDataString))
            {
                try
                {
                    byte[] entryData = Convert.FromBase64String(entryDataString);
                    ParseEntryData(entryData, internalIds, bundleIndices, keyToBundleMap);
                }
                catch (Exception)
                {
                    // If entry data parsing fails, fall back to assigning all
                    // non-bundle assets to the first/largest bundle (defaultlocalgroup).
                    string fallbackBundle = bundlePaths.Count > 0 ? bundlePaths[0] : string.Empty;
                    for (int i = 0; i < internalIds.Count; i++)
                    {
                        if (!bundleIndices.Contains(i) && !string.IsNullOrEmpty(fallbackBundle))
                        {
                            keyToBundleMap[internalIds[i]] = fallbackBundle;
                        }
                    }
                }
            }

            return new AddressablesCatalog(internalIds, bundlePaths, keyToBundleMap);
        }

        /// <summary>
        /// Parses the Addressables entry data to extract asset-to-bundle mappings.
        /// Entry data format: each entry is 28 bytes containing internal ID index,
        /// provider index, dependency key index, dep hash, data index, primary key index,
        /// and resource type index.
        /// </summary>
        private static void ParseEntryData(
            byte[] entryData,
            List<string> internalIds,
            HashSet<int> bundleIndices,
            Dictionary<string, string> keyToBundleMap)
        {
            // Addressables entry data format (Unity 2021.x):
            // First 4 bytes: entry count (int32)
            // Each entry: 7 x int32 = 28 bytes
            //   [0] internalId index
            //   [1] provider index
            //   [2] dependencyKey index
            //   [3] depHash
            //   [4] dataIndex
            //   [5] primaryKey index
            //   [6] resourceType index
            if (entryData.Length < 4)
                return;

            int count = BitConverter.ToInt32(entryData, 0);
            int entrySize = 28; // 7 ints * 4 bytes
            int headerSize = 4;

            if (entryData.Length < headerSize + count * entrySize)
                return;

            // First pass: identify which internal IDs are bundles (dependency keys)
            // and map asset entries to their dependency (bundle) keys.
            for (int i = 0; i < count; i++)
            {
                int offset = headerSize + i * entrySize;
                int internalIdIndex = BitConverter.ToInt32(entryData, offset);
                int depKeyIndex = BitConverter.ToInt32(entryData, offset + 8);

                // Skip if indices are out of range
                if (internalIdIndex < 0 || internalIdIndex >= internalIds.Count)
                    continue;

                // Skip bundle entries themselves
                if (bundleIndices.Contains(internalIdIndex))
                    continue;

                string assetKey = internalIds[internalIdIndex];

                // The dependency key index points to another entry whose internalId
                // is the bundle path. Find the bundle for this dependency.
                if (depKeyIndex >= 0 && depKeyIndex < internalIds.Count &&
                    bundleIndices.Contains(depKeyIndex))
                {
                    keyToBundleMap[assetKey] = internalIds[depKeyIndex];
                }
            }
        }

        /// <summary>
        /// Resolves an Addressables runtime path placeholder to a real filesystem path.
        /// Replaces {UnityEngine.AddressableAssets.Addressables.RuntimePath} with the
        /// actual StreamingAssets/aa directory.
        /// </summary>
        /// <param name="bundlePath">Bundle path from the catalog (may contain placeholders).</param>
        /// <param name="gameDir">Root game installation directory.</param>
        /// <returns>Resolved filesystem path to the bundle file.</returns>
        public static string ResolveBundlePath(string bundlePath, string gameDir)
        {
            const string runtimePathPlaceholder = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}";
            string streamingAssetsAa = Path.Combine(
                gameDir,
                "Diplomacy is Not an Option_Data",
                "StreamingAssets",
                "aa");

            if (bundlePath.Contains(runtimePathPlaceholder))
            {
                return bundlePath
                    .Replace(runtimePathPlaceholder, streamingAssetsAa)
                    .Replace('\\', Path.DirectorySeparatorChar);
            }

            return bundlePath;
        }
    }
}
