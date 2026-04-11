using System.Runtime.InteropServices;

namespace DINOForge.NativeInterop;

/// <summary>
/// P/Invoke wrapper for Zig LOD mesh decimation functions.
/// Interfaces with dinoforge_lod shared library (Windows: .dll, Linux: .so, macOS: .dylib).
/// </summary>
public static class ZigLodPipeline
{
    private const string LibName = "dinoforge_lod";

    /// <summary>
    /// Compute target LOD level based on vertex count and target reduction ratio.
    /// </summary>
    /// <param name="vertexCount">Current mesh vertex count.</param>
    /// <param name="targetRatio">Target polycount ratio (0.0-1.0, e.g., 0.5 = 50% reduction).</param>
    /// <returns>Target vertex count for the LOD level.</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint ComputeLodLevel(uint vertexCount, float targetRatio);

    /// <summary>
    /// Validate mesh geometry (vertex and triangle counts).
    /// </summary>
    /// <param name="vertexCount">Vertex count of the mesh.</param>
    /// <param name="triangleCount">Triangle count of the mesh.</param>
    /// <returns>True if mesh is valid; false otherwise.</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool ValidateMesh(uint vertexCount, uint triangleCount);

    /// <summary>
    /// Decimate mesh to target polycount percentage (simple proxy for Garfield-Heckbert).
    /// </summary>
    /// <param name="currentPolycount">Current triangle/polycount.</param>
    /// <param name="targetRatio">Target reduction ratio (0.0-1.0).</param>
    /// <returns>Target polycount after decimation.</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint DecimateToTarget(uint currentPolycount, float targetRatio);
}
