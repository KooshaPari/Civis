using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace DINOForge.Runtime
{
    /// <summary>
    /// P/Invoke wrapper for Rust asset pipeline (dinoforge_asset_pipeline.dll/so).
    /// Provides high-performance asset import and LOD optimization via FFI.
    ///
    /// **Integration Strategy:**
    /// - Primary: P/Invoke calls to Rust DLL (when available)
    /// - Fallback: Graceful degradation with error reporting (when DLL missing)
    /// - All operations marshal to/from JSON for cross-language compatibility
    ///
    /// **Safety:**
    /// - DllNotFoundException is caught and logged, not propagated
    /// - All returned pointers are cleaned up via RustFreeString
    /// - Input validation before P/Invoke calls
    /// </summary>
    public class RustAssetPipelineInterop
    {
        /// <summary>
        /// Check if Rust asset pipeline DLL is available and loadable.
        /// </summary>
        public static bool IsAvailable
        {
            get
            {
                if (_availability.HasValue)
                    return _availability.Value;

                try
                {
                    // Attempt a trivial P/Invoke call to validate DLL loading
                    RustGetVersion(out IntPtr versionPtr);
                    if (versionPtr != IntPtr.Zero)
                    {
                        RustFreeString(versionPtr);
                        _availability = true;
                    }
                    else
                    {
                        _availability = false;
                    }
                }
                catch (DllNotFoundException)
                {
                    _availability = false;
                }
                catch (EntryPointNotFoundException)
                {
                    _availability = false;
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"[RustAssetPipeline] Availability check failed: {ex.GetType().Name}: {ex.Message}");
                    _availability = false;
                }

                return _availability.Value;
            }
        }

        private static bool? _availability;

        /// <summary>
        /// Import a 3D model asset (GLB, FBX, OBJ, etc.) using Rust pipeline.
        /// Performs mesh validation, material extraction, and skeleton import.
        /// </summary>
        /// <param name="glbPath">Full path to GLB/FBX/OBJ file</param>
        /// <param name="assetId">Unique asset identifier (e.g., "sw-rep-clone-trooper")</param>
        /// <returns>Imported asset data with mesh, materials, skeleton</returns>
        /// <exception cref="ArgumentNullException">Path or assetId is null</exception>
        /// <exception cref="FileNotFoundException">Asset file does not exist</exception>
        /// <exception cref="InvalidOperationException">Import failed or Rust DLL unavailable</exception>
        public static AssetData ImportAsset(string glbPath, string assetId)
        {
            if (glbPath == null)
                throw new ArgumentNullException(nameof(glbPath), "Asset path cannot be null");

            if (assetId == null)
                throw new ArgumentNullException(nameof(assetId), "Asset ID cannot be null");

            if (!File.Exists(glbPath))
                throw new FileNotFoundException($"Asset file not found: {glbPath}", glbPath);

            if (!IsAvailable)
                throw new InvalidOperationException(
                    "Rust asset pipeline unavailable. Ensure dinoforge_asset_pipeline.dll is in PATH or current directory.");

            try
            {
                string fullPath = Path.GetFullPath(glbPath);
                int exitCode = RustImportAsset(fullPath, assetId, out IntPtr resultPtr);

                if (exitCode != 0)
                    throw new InvalidOperationException($"Rust import failed with exit code {exitCode}");

                if (resultPtr == IntPtr.Zero)
                    throw new InvalidOperationException("Rust returned null result");

                try
                {
                    string json = Marshal.PtrToStringAnsi(resultPtr)
                        ?? throw new InvalidOperationException("Failed to marshal Rust result string");

                    var imported = JsonConvert.DeserializeObject<AssetData>(json);
                    return imported ?? throw new InvalidOperationException("Failed to deserialize import result JSON");
                }
                finally
                {
                    RustFreeString(resultPtr);
                }
            }
            catch (DllNotFoundException dex)
            {
                throw new InvalidOperationException(
                    "Rust asset pipeline DLL not found. Ensure dinoforge_asset_pipeline.dll/so is deployed.", dex);
            }
            catch (EntryPointNotFoundException epex)
            {
                throw new InvalidOperationException(
                    "Rust asset pipeline function not found. DLL version mismatch?", epex);
            }
        }

        /// <summary>
        /// Optimize an imported asset by generating LOD (Level of Detail) variants.
        /// Uses SIMD-accelerated mesh decimation for performance.
        /// </summary>
        /// <param name="importedAsset">Asset data to optimize</param>
        /// <param name="lodTargets">LOD polycount targets as percentages (e.g., [100, 60, 30])</param>
        /// <returns>Array of optimized LOD variants [LOD0, LOD1, LOD2, ...]</returns>
        /// <exception cref="ArgumentNullException">Asset or lodTargets is null</exception>
        /// <exception cref="ArgumentException">lodTargets is empty or contains invalid values</exception>
        /// <exception cref="InvalidOperationException">Optimization failed or Rust DLL unavailable</exception>
        public static AssetData[] OptimizeAsset(AssetData importedAsset, int[] lodTargets)
        {
            if (importedAsset == null)
                throw new ArgumentNullException(nameof(importedAsset), "Asset cannot be null");

            if (lodTargets == null)
                throw new ArgumentNullException(nameof(lodTargets), "LOD targets cannot be null");

            if (lodTargets.Length == 0)
                throw new ArgumentException("LOD targets array cannot be empty", nameof(lodTargets));

            foreach (var target in lodTargets)
            {
                if (target <= 0 || target > 100)
                    throw new ArgumentException($"LOD target must be between 1 and 100, got {target}", nameof(lodTargets));
            }

            if (!IsAvailable)
                throw new InvalidOperationException(
                    "Rust asset pipeline unavailable. Ensure dinoforge_asset_pipeline.dll is in PATH or current directory.");

            try
            {
                // Serialize optimization request
                var optimizeRequest = new
                {
                    asset_id = importedAsset.AssetId,
                    vertex_count = importedAsset.VertexCount,
                    lod_targets = lodTargets
                };

                string requestJson = JsonConvert.SerializeObject(optimizeRequest);

                int exitCode = RustOptimizeAsset(requestJson, out IntPtr resultPtr);

                if (exitCode != 0)
                    throw new InvalidOperationException($"Rust optimization failed with exit code {exitCode}");

                if (resultPtr == IntPtr.Zero)
                    throw new InvalidOperationException("Rust returned null result");

                try
                {
                    string json = Marshal.PtrToStringAnsi(resultPtr)
                        ?? throw new InvalidOperationException("Failed to marshal Rust result string");

                    var optimized = JsonConvert.DeserializeObject<AssetData[]>(json);
                    return optimized ?? throw new InvalidOperationException("Failed to deserialize optimization result JSON");
                }
                finally
                {
                    RustFreeString(resultPtr);
                }
            }
            catch (DllNotFoundException dex)
            {
                throw new InvalidOperationException(
                    "Rust asset pipeline DLL not found. Ensure dinoforge_asset_pipeline.dll/so is deployed.", dex);
            }
            catch (EntryPointNotFoundException epex)
            {
                throw new InvalidOperationException(
                    "Rust asset pipeline function not found. DLL version mismatch?", epex);
            }
        }

        // ===== P/Invoke Bindings =====

        /// <summary>
        /// P/Invoke: Check Rust module version and availability.
        /// Returns version string as allocated C string pointer.
        /// </summary>
        [DllImport("dinoforge_asset_pipeline", CallingConvention = CallingConvention.Cdecl, SetLastError = true)]
        private static extern int RustGetVersion(out IntPtr versionPtr);

        /// <summary>
        /// P/Invoke: Import 3D asset from GLB/FBX/OBJ.
        /// Returns JSON as C string pointer, allocated by Rust (must be freed with RustFreeString).
        /// Exit codes: 0 = success, non-zero = error
        /// </summary>
        [DllImport("dinoforge_asset_pipeline", CallingConvention = CallingConvention.Cdecl, SetLastError = true,
            CharSet = CharSet.Ansi)]
        private static extern int RustImportAsset(
            [MarshalAs(UnmanagedType.LPStr)] string filePath,
            [MarshalAs(UnmanagedType.LPStr)] string assetId,
            out IntPtr resultJson);

        /// <summary>
        /// P/Invoke: Optimize asset by generating LOD variants.
        /// Input: JSON request with asset_id, vertex_count, lod_targets.
        /// Returns JSON array of optimized assets as C string pointer.
        /// Exit codes: 0 = success, non-zero = error
        /// </summary>
        [DllImport("dinoforge_asset_pipeline", CallingConvention = CallingConvention.Cdecl, SetLastError = true,
            CharSet = CharSet.Ansi)]
        private static extern int RustOptimizeAsset(
            [MarshalAs(UnmanagedType.LPStr)] string requestJson,
            out IntPtr resultJson);

        /// <summary>
        /// P/Invoke: Free a C string allocated by Rust.
        /// Call after each RustImportAsset or RustOptimizeAsset to clean up.
        /// </summary>
        [DllImport("dinoforge_asset_pipeline", CallingConvention = CallingConvention.Cdecl)]
        private static extern void RustFreeString(IntPtr ptr);
    }

    /// <summary>
    /// JSON-serializable asset data model for P/Invoke marshaling.
    /// Matches Rust serde structures for zero-copy FFI.
    /// </summary>
    public class AssetData
    {
        [JsonProperty("asset_id")]
        public string AssetId { get; set; } = string.Empty;

        [JsonProperty("source_path")]
        public string SourcePath { get; set; } = string.Empty;

        [JsonProperty("vertex_count")]
        public int VertexCount { get; set; }

        [JsonProperty("triangle_count")]
        public int TriangleCount { get; set; }

        [JsonProperty("bone_count")]
        public int BoneCount { get; set; }

        [JsonProperty("material_count")]
        public int MaterialCount { get; set; }

        [JsonProperty("bounds_min")]
        public float[] BoundsMin { get; set; } = Array.Empty<float>();

        [JsonProperty("bounds_max")]
        public float[] BoundsMax { get; set; } = Array.Empty<float>();

        [JsonProperty("mesh_data")]
        public byte[] MeshData { get; set; } = Array.Empty<byte>();

        [JsonProperty("metadata")]
        public AssetMetadataJson Metadata { get; set; } = new();
    }

    /// <summary>
    /// Asset metadata for JSON serialization.
    /// </summary>
    public class AssetMetadataJson
    {
        [JsonProperty("format")]
        public string Format { get; set; } = string.Empty;

        [JsonProperty("version")]
        public string Version { get; set; } = string.Empty;

        [JsonProperty("author")]
        public string Author { get; set; } = string.Empty;

        [JsonProperty("license")]
        public string License { get; set; } = string.Empty;

        [JsonProperty("optimize_timestamp")]
        public long OptimizeTimestamp { get; set; }
    }
}
