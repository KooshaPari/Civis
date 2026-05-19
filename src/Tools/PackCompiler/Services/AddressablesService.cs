#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using DINOForge.Tools.PackCompiler.Models;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Service for generating Addressables catalog entries.
    /// Creates address mapping files for runtime asset loading.
    /// </summary>
    public class AddressablesService
    {
        /// <summary>
        /// Generate Addressables catalog for a set of processed assets.
        /// </summary>
        public async Task GenerateCatalogAsync(
            List<(OptimizedAsset asset, AssetDefinition definition)> assets,
            string outputPath)
        {
            await Task.Run(() =>
            {
                var catalogEntries = new List<string>();
                catalogEntries.Add("# Addressables Catalog for DINOForge Assets");
                catalogEntries.Add($"# Generated: {DateTime.UtcNow:O}");
                catalogEntries.Add("");

                foreach (var (asset, definition) in assets)
                {
                    var entry = BuildCatalogEntry(asset, definition);
                    catalogEntries.AddRange(entry);
                    catalogEntries.Add("");
                }

                Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");
                File.WriteAllLines(outputPath, catalogEntries, Encoding.UTF8);
            });
        }

        /// <summary>
        /// Generate a JSON catalog in YAML format (compatible with Addressables v1.21+).
        /// </summary>
        public async Task GenerateYamlCatalogAsync(
            List<(OptimizedAsset asset, AssetDefinition definition)> assets,
            string outputPath)
        {
            await Task.Run(() =>
            {
                var sb = new StringBuilder(4096);  // Capacity ~= YAML header (20 lines × 60 chars) + asset loop (N × 40 chars)

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

                foreach (var (asset, definition) in assets)
                {
                    string guid = Guid.NewGuid().ToString();
                    sb.AppendLine($"  - {{fileID: {guid}, type: AssetGroup}}");
                }

                sb.AppendLine("  m_BuildRemoteCatalog: 0");
                sb.AppendLine("  m_DisableCatalogUpdateOnStart: 0");
                sb.AppendLine("  m_UniqueBundleIds: 1");
                sb.AppendLine("  m_NonceNumericValue: 0");
                sb.AppendLine("  m_ShaderBundleNaming: 0");
                sb.AppendLine("  m_MonoScriptBundlesName: ");
                sb.AppendLine("  m_StripUnityVersionFromBundleNames: 0");
                sb.AppendLine("  m_DisableVisibleToOtherAssemblies: 0");
                sb.AppendLine("  m_CopyBundlesToPlatformCache: 0");
                sb.AppendLine("  m_DebugNames: 0");

                Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");
                File.WriteAllText(outputPath, sb.ToString(), Encoding.UTF8);
            });
        }

        /// <summary>
        /// Generate an asset group entry with LOD variants.
        /// </summary>
        public async Task GenerateAssetGroupAsync(
            OptimizedAsset asset,
            AssetDefinition definition,
            string outputDirectory)
        {
            await Task.Run(() =>
            {
                var groupFile = Path.Combine(outputDirectory, $"{asset.AssetId}_group.yaml");

                var sb = new StringBuilder(512);  // Capacity ~= 22 appends × 24 chars (YAML lines)
                sb.AppendLine("%YAML 1.1");
                sb.AppendLine("%TAG !u! tag:unity3d.com,2011:");
                sb.AppendLine("--- !u!114 &11400000");
                sb.AppendLine("AddressableAssetGroup:");
                sb.AppendLine("  m_ObjectHideFlags: 0");
                sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
                sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
                sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
                sb.AppendLine("  m_Name: " + asset.AssetId);
                sb.AppendLine("  m_Entries:");

                // Add LOD0
                sb.AppendLine("  - m_GUID: " + Guid.NewGuid().ToString());
                sb.AppendLine($"    m_Address: {definition.AddressableKey}_lod0");
                sb.AppendLine("    m_SerializedLabels: []");

                // Add LOD1
                sb.AppendLine("  - m_GUID: " + Guid.NewGuid().ToString());
                sb.AppendLine($"    m_Address: {definition.AddressableKey}_lod1");
                sb.AppendLine("    m_SerializedLabels: []");

                // Add LOD2
                sb.AppendLine("  - m_GUID: " + Guid.NewGuid().ToString());
                sb.AppendLine($"    m_Address: {definition.AddressableKey}_lod2");
                sb.AppendLine("    m_SerializedLabels: []");

                // Add main address (LOD0)
                sb.AppendLine("  - m_GUID: " + Guid.NewGuid().ToString());
                sb.AppendLine($"    m_Address: {definition.AddressableKey}");
                sb.AppendLine("    m_SerializedLabels: [\"lod-asset\", \"dino-forge\"]");

                Directory.CreateDirectory(outputDirectory);
                File.WriteAllText(groupFile, sb.ToString(), Encoding.UTF8);
            });
        }

        private List<string> BuildCatalogEntry(OptimizedAsset asset, AssetDefinition definition)
        {
            var lines = new List<string>();

            lines.Add($"[{definition.AddressableKey}]");
            lines.Add($"  asset_id: {asset.AssetId}");
            lines.Add($"  type: {definition.Type}");
            lines.Add($"  faction: {definition.Faction}");
            lines.Add($"  lod0_path: {definition.OutputPrefab}.lod0");
            lines.Add($"  lod1_path: {definition.OutputPrefab}.lod1");
            lines.Add($"  lod2_path: {definition.OutputPrefab}.lod2");
            lines.Add($"  lod0_polycount: {asset.LOD0.TriangleCount}");
            lines.Add($"  lod1_polycount: {asset.LOD1.TriangleCount}");
            lines.Add($"  lod2_polycount: {asset.LOD2.TriangleCount}");
            lines.Add($"  material: {definition.Material}");
            lines.Add($"  scale: {definition.Scale}");
            lines.Add($"  optimized_at: {asset.OptimizedAt:O}");

            if (asset.Skeleton != null)
            {
                lines.Add($"  skeleton: {asset.Skeleton.RootBone}");
                lines.Add($"  bone_count: {asset.Skeleton.BoneCount}");
            }

            return lines;
        }
    }
}
