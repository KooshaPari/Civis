using System;
using System.Text;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Order;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for StringBuilder capacity hint optimization.
/// Measures allocation overhead when pre-sizing StringBuilder for 30-line YAML output.
/// Context: AddressablesService.GenerateYamlCatalogAsync uses new StringBuilder(4096) to reduce allocations.
/// </summary>
[MemoryDiagnoser]
[Orderer(SummaryOrderPolicy.FastestToSlowest)]
[RankColumn]
[SimpleJob(warmupCount: 3, iterationCount: 5)]
public class StringBuilderAllocationBenchmarks
{
    /// <summary>
    /// Baseline: StringBuilder with default capacity (~16 chars).
    /// Simulates 30-line YAML generation, each line ~40 chars (typical Addressables catalog output).
    /// Total ~1200 chars + expansion overhead.
    /// </summary>
    [Benchmark]
    public string Baseline_DefaultCapacity()
    {
        var sb = new StringBuilder();

        // Addressables YAML header (20 lines)
        sb.AppendLine("%YAML 1.1");
        sb.AppendLine("%TAG !u! tag:unity3d.com,2011:");
        sb.AppendLine("--- !u!114 &11400000");
        sb.AppendLine("AddressableAssetSettings:");
        sb.AppendLine("  m_ObjectHideFlags: 0");
        sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
        sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
        sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
        sb.AppendLine("  m_Name: AddressableAssetSettings");
        sb.AppendLine("  m_EditorClassIdentifier: ");
        sb.AppendLine("  m_DefaultGroup: Default Local Group");
        sb.AppendLine("  m_CertificateHandlerType:");
        sb.AppendLine("    m_AssemblyTypeName: ");
        sb.AppendLine("  m_LogResourceManagerExceptions: 1");
        sb.AppendLine("  m_EnableJsonSerialization: 0");
        sb.AppendLine("  m_MaxConcurrentWebRequests: 3");
        sb.AppendLine("  m_CatalogImpl: AddressableAssetsCatalogProvider");
        sb.AppendLine("  m_GroupAssets:");

        // Simulated asset group entries (10 items, each ~50 chars)
        for (int i = 0; i < 10; i++)
        {
            string guid = Guid.NewGuid().ToString();
            sb.AppendLine($"  - {{fileID: {guid}, type: AssetGroup}}");
        }

        sb.AppendLine("  m_BuildRemoteCatalog: 0");
        sb.AppendLine("  m_DisableCatalogUpdateOnStart: 0");

        return sb.ToString();
    }

    /// <summary>
    /// Optimized: StringBuilder with pre-allocated capacity (4096 chars).
    /// Avoids internal buffer resizing during the 30-line append sequence.
    /// </summary>
    [Benchmark]
    public string Optimized_HintedCapacity()
    {
        var sb = new StringBuilder(4096);

        // Addressables YAML header (20 lines)
        sb.AppendLine("%YAML 1.1");
        sb.AppendLine("%TAG !u! tag:unity3d.com,2011:");
        sb.AppendLine("--- !u!114 &11400000");
        sb.AppendLine("AddressableAssetSettings:");
        sb.AppendLine("  m_ObjectHideFlags: 0");
        sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
        sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
        sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
        sb.AppendLine("  m_Name: AddressableAssetSettings");
        sb.AppendLine("  m_EditorClassIdentifier: ");
        sb.AppendLine("  m_DefaultGroup: Default Local Group");
        sb.AppendLine("  m_CertificateHandlerType:");
        sb.AppendLine("    m_AssemblyTypeName: ");
        sb.AppendLine("  m_LogResourceManagerExceptions: 1");
        sb.AppendLine("  m_EnableJsonSerialization: 0");
        sb.AppendLine("  m_MaxConcurrentWebRequests: 3");
        sb.AppendLine("  m_CatalogImpl: AddressableAssetsCatalogProvider");
        sb.AppendLine("  m_GroupAssets:");

        // Simulated asset group entries (10 items, each ~50 chars)
        for (int i = 0; i < 10; i++)
        {
            string guid = Guid.NewGuid().ToString();
            sb.AppendLine($"  - {{fileID: {guid}, type: AssetGroup}}");
        }

        sb.AppendLine("  m_BuildRemoteCatalog: 0");
        sb.AppendLine("  m_DisableCatalogUpdateOnStart: 0");

        return sb.ToString();
    }
}
