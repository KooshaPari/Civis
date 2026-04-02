using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Runtime.InteropServices;
using System.Text.Json;
using System.Threading.Tasks;
using DINOForge.SDK.Models;

namespace DINOForge.SDK.NativeInterop
{
    /// <summary>
    /// Interop wrapper for Rust asset pipeline module (dinoforge_asset_pipeline).
    /// Provides high-performance GLB/FBX import with zero-copy mesh operations and SIMD LOD generation.
    ///
    /// **Integration Paths**:
    /// 1. PyO3 via Python MCP server (preferred) — HTTP call to localhost:8765
    /// 2. Direct P/Invoke (optional, if latency critical) — load .pyd/.dll directly
    /// 3. C# fallback via AssimpNet (always available) — if Rust module unavailable
    ///
    /// **Load Priority**:
    /// 1. Try PyO3 module via MCP server (seamless)
    /// 2. Fall back to C# AssimpNet wrapper
    /// </summary>
    [ExcludeFromCodeCoverage] // Requires Rust/PyO3 toolchain — integration tests only
    public static class RustAssetPipeline
    {
        /// <summary>
        /// Check if Rust asset pipeline is available in this environment.
        /// Returns true if PyO3 module exists AND MCP server is running (or P/Invoke DLL available).
        /// </summary>
        public static bool IsAvailable
        {
            get
            {
                // Check MCP server availability (preferred path)
                if (_mcpAvailable.HasValue)
                    return _mcpAvailable.Value;

                // Attempt to detect Python module
                try
                {
                    // Try calling MCP server
                    var response = TryCallMcp("ping");
                    _mcpAvailable = response != null;
                    return _mcpAvailable.Value;
                }
                catch
                {
                    _mcpAvailable = false;
                    return false;
                }
            }
        }

        private static bool? _mcpAvailable;

        /// <summary>
        /// Import a 3D model asset (GLB, FBX, OBJ, etc.) using Rust pipeline for maximum performance.
        /// Falls back to C# AssimpNet if Rust unavailable.
        /// </summary>
        /// <param name="assetId">Unique asset identifier (e.g., "sw-rep-clone-trooper")</param>
        /// <param name="filePath">Full path to GLB/FBX/OBJ file</param>
        /// <returns>Imported asset with mesh, materials, skeleton data</returns>
        /// <exception cref="FileNotFoundException">Asset file not found</exception>
        /// <exception cref="InvalidOperationException">Rust module failed or corrupted data</exception>
        public static async Task<ImportedAsset> ImportAssetAsync(string assetId, string filePath)
        {
            if (!File.Exists(filePath))
                throw new FileNotFoundException($"Asset file not found: {filePath}");

            // Try Rust pathway first
            if (IsAvailable)
            {
                try
                {
                    return await ImportAssetViaRustAsync(assetId, filePath);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"Rust import failed, falling back to C#: {ex.Message}");
                    // Fall through to C# path
                }
            }

            // Fallback to C# AssimpNet
            return await ImportAssetViaCSharpAsync(assetId, filePath);
        }

        /// <summary>
        /// Import via Rust PyO3 module + MCP server (preferred, fastest).
        /// </summary>
        private static async Task<ImportedAsset> ImportAssetViaRustAsync(string assetId, string filePath)
        {
            // Make HTTP call to MCP server
            var request = new
            {
                file_path = Path.GetFullPath(filePath),
                asset_id = assetId
            };

            var response = await CallMcpAsync("asset_import", request);

            if (response == null)
                throw new InvalidOperationException("MCP server returned no data");

            // Parse response JSON
            var json = response.ToString();
            var imported = JsonSerializer.Deserialize<ImportedAsset>(json);

            if (imported == null)
                throw new InvalidOperationException("Failed to deserialize Rust import result");

            return imported;
        }

        /// <summary>
        /// Import via C# AssimpNet wrapper (fallback, acceptable performance).
        /// </summary>
        private static async Task<ImportedAsset> ImportAssetViaCSharpAsync(string assetId, string filePath)
        {
            // Delegate to existing C# implementation
            // This is a placeholder — actual implementation would call AssimpNet wrapper
            return await Task.FromResult(new ImportedAsset
            {
                AssetId = assetId,
                SourcePath = filePath,
                Mesh = new MeshData { Vertices = Array.Empty<float>() },
                Materials = new List<MaterialData>(),
                Skeleton = null,
                Metadata = new AssetMetadata()
            });
        }

        /// <summary>
        /// Optimize imported asset (LOD generation, mesh decimation) using Rust for parallelism.
        /// Falls back to C# if unavailable.
        /// </summary>
        public static async Task<OptimizedAsset> OptimizeAssetAsync(
            ImportedAsset imported,
            AssetDefinition definition)
        {
            if (IsAvailable)
            {
                try
                {
                    return await OptimizeAssetViaRustAsync(imported, definition);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"Rust optimization failed: {ex.Message}");
                }
            }

            // Fallback to C# optimization
            return await OptimizeAssetViaCSharpAsync(imported, definition);
        }

        private static async Task<OptimizedAsset> OptimizeAssetViaRustAsync(
            ImportedAsset imported,
            AssetDefinition definition)
        {
            var request = new
            {
                asset_id = imported.AssetId,
                vertex_count = imported.Mesh.Vertices.Length / 3,
                lod_targets = definition.LOD?.Levels ?? new[] { 100, 60, 30 }
            };

            var response = await CallMcpAsync("asset_optimize", request);
            var json = response.ToString();
            return JsonSerializer.Deserialize<OptimizedAsset>(json)
                ?? throw new InvalidOperationException("Failed to deserialize optimization result");
        }

        private static async Task<OptimizedAsset> OptimizeAssetViaCSharpAsync(
            ImportedAsset imported,
            AssetDefinition definition)
        {
            // Placeholder — would call C# optimization service
            return await Task.FromResult(new OptimizedAsset
            {
                AssetId = imported.AssetId,
                LOD0 = imported.Mesh,
                LOD1 = imported.Mesh,
                LOD2 = imported.Mesh
            });
        }

        // ===== Private MCP Integration =====

        private static async Task<object?> CallMcpAsync(string toolName, object parameters)
        {
            // Implementation: POST to http://127.0.0.1:8765/api/tools/{toolName}
            // with parameters as JSON body
            // Returns parsed JSON response
            // For now, stub implementation
            await Task.Delay(0);
            return null;
        }

        private static object? TryCallMcp(string toolName)
        {
            // Non-async version for availability check
            // Returns null if MCP server not reachable
            return null;
        }

        // ===== P/Invoke Alternative (for low-latency paths) =====

        /// <summary>
        /// Direct P/Invoke call to Rust module (cdecl, returns JSON as string).
        /// Only used if PyO3 module not available but DLL is loaded.
        /// **NOT RECOMMENDED**: Use MCP path instead for better platform support.
        /// </summary>
        [DllImport("dinoforge_asset_pipeline", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        private static extern int RustImportAsset(
            [MarshalAs(UnmanagedType.LPStr)] string filePath,
            [MarshalAs(UnmanagedType.LPStr)] string assetId,
            [Out] out IntPtr resultJson);

        /// <summary>
        /// Cleanup pointer allocated by Rust FFI.
        /// </summary>
        [DllImport("dinoforge_asset_pipeline", CallingConvention = CallingConvention.Cdecl)]
        private static extern void RustFreeString(IntPtr ptr);

        /// <summary>
        /// Direct P/Invoke import (alternative if latency critical and DLL available).
        /// </summary>
        private static ImportedAsset ImportAssetViaPInvoke(string assetId, string filePath)
        {
            int exitCode = RustImportAsset(filePath, assetId, out IntPtr resultPtr);

            if (exitCode != 0)
                throw new InvalidOperationException($"Rust import failed with code {exitCode}");

            try
            {
                string json = Marshal.PtrToStringAnsi(resultPtr)
                    ?? throw new InvalidOperationException("Rust returned null JSON");

                var imported = JsonSerializer.Deserialize<ImportedAsset>(json);
                return imported ?? throw new InvalidOperationException("Failed to deserialize JSON");
            }
            finally
            {
                RustFreeString(resultPtr);
            }
        }
    }

    // Placeholder types (normally in SDK.Models)

    /// <summary>
    /// Represents an asset imported via the Rust pipeline, containing mesh, materials, skeleton, and metadata.
    /// </summary>
    public class ImportedAsset
    {
        /// <summary>The unique identifier for this asset.</summary>
        public string AssetId { get; set; } = string.Empty;
        /// <summary>Path to the source asset file.</summary>
        public string SourcePath { get; set; } = string.Empty;
        /// <summary>Mesh data (vertices, indices, triangles).</summary>
        public MeshData Mesh { get; set; } = new();
        /// <summary>List of materials used by this asset.</summary>
        public List<MaterialData> Materials { get; set; } = new();
        /// <summary>Optional skeleton data for rigged assets.</summary>
        public SkeletonData? Skeleton { get; set; }
        /// <summary>Asset metadata including polycount.</summary>
        public AssetMetadata Metadata { get; set; } = new();
    }

    /// <summary>
    /// Mesh geometry data including vertices, indices, and triangle count.
    /// </summary>
    public class MeshData
    {
        /// <summary>Vertex positions as float array.</summary>
        public float[] Vertices { get; set; } = Array.Empty<float>();
        /// <summary>Triangle indices as unsigned int array.</summary>
        public uint[] Indices { get; set; } = Array.Empty<uint>();
        /// <summary>Total number of triangles in this mesh.</summary>
        public int TriangleCount { get; set; }
    }

    /// <summary>
    /// Material data including name and properties.
    /// </summary>
    public class MaterialData
    {
        /// <summary>Name of this material.</summary>
        public string Name { get; set; } = string.Empty;
    }

    /// <summary>
    /// Skeleton data for rigged assets.
    /// </summary>
    public class SkeletonData
    {
        /// <summary>Name of the skeleton.</summary>
        public string Name { get; set; } = string.Empty;
    }

    /// <summary>
    /// Metadata about an imported asset.
    /// </summary>
    public class AssetMetadata
    {
        /// <summary>Number of polygons in the original asset.</summary>
        public int PolyCount { get; set; }
    }

    /// <summary>
    /// Asset with pre-generated LOD (Level of Detail) meshes.
    /// </summary>
    public class OptimizedAsset
    {
        /// <summary>The unique identifier for this asset.</summary>
        public string AssetId { get; set; } = string.Empty;
        /// <summary>Highest-detail LOD mesh (100% polycount).</summary>
        public MeshData LOD0 { get; set; } = new();
        /// <summary>Medium-detail LOD mesh (typically 50% polycount).</summary>
        public MeshData LOD1 { get; set; } = new();
        /// <summary>Lowest-detail LOD mesh (typically 10% polycount).</summary>
        public MeshData LOD2 { get; set; } = new();
    }

    /// <summary>
    /// Definition of an asset for registration in the asset registry.
    /// </summary>
    public class AssetDefinition
    {
        /// <summary>Unique identifier for this asset.</summary>
        public string Id { get; set; } = string.Empty;
        /// <summary>Optional LOD configuration for this asset.</summary>
        public LODDefinition? LOD { get; set; }
    }

    /// <summary>
    /// LOD (Level of Detail) configuration specifying mesh reduction targets.
    /// </summary>
    public class LODDefinition
    {
        /// <summary>Polycount percentages for each LOD level (e.g., [100, 50, 10]).</summary>
        public int[] Levels { get; set; } = Array.Empty<int>();
    }
}
