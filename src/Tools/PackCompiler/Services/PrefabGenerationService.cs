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
    /// Service for generating Unity prefabs from optimized assets.
    /// Creates prefab YAML files with LOD group, mesh renderers, and materials.
    /// </summary>
    public class PrefabGenerationService
    {
        /// <summary>
        /// Generate a prefab from an optimized asset.
        /// Creates a .prefab file with LOD group, mesh renderers, and material assignments.
        /// </summary>
        public async Task GeneratePrefabAsync(OptimizedAsset asset, AssetDefinition definition, string outputPath)
        {
            await Task.Run(() =>
            {
                var prefab = BuildPrefabYaml(asset, definition);

                Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");
                File.WriteAllText(outputPath, prefab, Encoding.UTF8);
            });
        }

        /// <summary>
        /// Build prefab YAML content for the optimized asset.
        /// Includes LOD group, mesh filter/renderer, materials, and skeleton (if rigged).
        /// </summary>
        private string BuildPrefabYaml(OptimizedAsset asset, AssetDefinition definition)
        {
            var sb = new StringBuilder(512);  // Capacity ~= 17 appends × 30 chars (prefab YAML lines)

            // Prefab header
            sb.AppendLine("%YAML 1.1");
            sb.AppendLine("%TAG !u! tag:unity3d.com,2011:");
            sb.AppendLine("--- !u!1 &" + GenerateGUID());
            sb.AppendLine("GameObject:");
            sb.AppendLine("  m_ObjectHideFlags: 0");
            sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
            sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
            sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
            sb.AppendLine("  serializedVersion: 6");
            sb.AppendLine("  m_Component:");
            sb.AppendLine("  - component: {fileID: " + GenerateFileID() + "}");
            sb.AppendLine("  - component: {fileID: " + GenerateFileID() + "}");

            // Add renderer components
            if (asset.LOD0 != null)
            {
                sb.AppendLine("  - component: {fileID: " + GenerateFileID() + "}");
            }

            // Add LOD group if multiple LODs
            if (asset.LOD1 != null && asset.LOD2 != null)
            {
                sb.AppendLine("  - component: {fileID: " + GenerateFileID() + "}");
            }

            sb.AppendLine("  m_Layer: 0");
            sb.AppendLine($"  m_Name: {asset.AssetId}");
            sb.AppendLine("  m_TagString: Untagged");
            sb.AppendLine("  m_Icon: {fileID: 0}");
            sb.AppendLine("  m_NavMeshLayer: 0");
            sb.AppendLine("  m_StaticEditorFlags: 0");
            sb.AppendLine("  m_IsActive: 1");
            sb.AppendLine("--- !u!4 &" + GenerateGUID());

            // Transform component
            sb.AppendLine("Transform:");
            sb.AppendLine("  m_ObjectHideFlags: 1");
            sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
            sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
            sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
            sb.AppendLine("  m_GameObject: {fileID: 0}");
            sb.AppendLine("  m_LocalRotation: {x: 0, y: 0, z: 0, w: 1}");
            sb.AppendLine("  m_LocalPosition: {x: 0, y: 0, z: 0}");
            sb.AppendLine($"  m_LocalScale: {{x: {definition.Scale}, y: {definition.Scale}, z: {definition.Scale}}}");
            sb.AppendLine("  m_ConstrainProportionsScale: 0");
            sb.AppendLine("  m_Children: []");
            sb.AppendLine("  m_Father: {fileID: 0}");
            sb.AppendLine("  m_RootOrder: 0");
            sb.AppendLine("  m_LocalEulerAnglesHint: {x: 0, y: 0, z: 0}");

            // Mesh filter and renderer (LOD0)
            sb.AppendLine("--- !u!33 &" + GenerateGUID());
            sb.AppendLine("MeshFilter:");
            sb.AppendLine("  m_ObjectHideFlags: 1");
            sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
            sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
            sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
            sb.AppendLine("  m_GameObject: {fileID: 0}");
            sb.AppendLine($"  m_Mesh: {{fileID: {GenerateMeshFileID(asset.LOD0!)}, guid: {GenerateGUIDString()}, type: 3}}");

            sb.AppendLine("--- !u!23 &" + GenerateGUID());
            sb.AppendLine("MeshRenderer:");
            sb.AppendLine("  m_ObjectHideFlags: 1");
            sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
            sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
            sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
            sb.AppendLine("  m_GameObject: {fileID: 0}");
            sb.AppendLine("  m_Enabled: 1");
            sb.AppendLine("  m_CastShadows: 1");
            sb.AppendLine("  m_ReceiveShadows: 1");
            sb.AppendLine("  m_DynamicOccludee: 1");
            sb.AppendLine("  m_StaticShadowCaster: 0");
            sb.AppendLine("  m_MotionVectors: 1");
            sb.AppendLine("  m_LightProbeUsage: 1");
            sb.AppendLine("  m_ReflectionProbeUsage: 1");
            sb.AppendLine("  m_RayTracingMode: 2");
            sb.AppendLine("  m_RayTraceProcedural: 0");
            sb.AppendLine("  m_RenderingLayerMask: 1");
            sb.AppendLine("  m_RendererPriority: 0");
            sb.AppendLine("  m_Materials:");

            // Material assignments
            foreach (var material in asset.Materials)
            {
                sb.AppendLine($"  - {{fileID: {GenerateMaterialFileID(material)}, guid: {GenerateGUIDString()}, type: 2}}");
            }

            sb.AppendLine("  m_StaticBatchInfo:");
            sb.AppendLine("    firstBatch: 0");
            sb.AppendLine("    batchCount: 0");
            sb.AppendLine("  m_StaticBatchRoot: {fileID: 0}");
            sb.AppendLine("  m_ProbeAnchor: {fileID: 0}");
            sb.AppendLine("  m_LightProbeVolumeOverride: {fileID: 0}");
            sb.AppendLine("  m_ScaleInLightmap: 1");
            sb.AppendLine("  m_ReceiveGI: 3");
            sb.AppendLine("  m_PreserveUVs: 0");
            sb.AppendLine("  m_IgnoreNormalsForChartDetection: 0");
            sb.AppendLine("  m_ImportantGI: 0");
            sb.AppendLine("  m_StitchLightmapSeams: 1");
            sb.AppendLine("  m_SelectedEditorRenderState: 3");
            sb.AppendLine("  m_MinimumChartSize: 4");
            sb.AppendLine("  m_AutoUVMaxDistance: 0.5");
            sb.AppendLine("  m_AutoUVMaxAngle: 89");
            sb.AppendLine("  m_LightmapParameters: {fileID: 0}");
            sb.AppendLine("  m_SortingLayerID: 0");
            sb.AppendLine("  m_SortingOrder: 0");
            sb.AppendLine("  m_AdditionalVertexStreams: {fileID: 0}");

            // LOD group (if applicable)
            if (asset.LOD1 != null && asset.LOD2 != null)
            {
                sb.AppendLine("--- !u!198 &" + GenerateGUID());
                sb.AppendLine("LODGroup:");
                sb.AppendLine("  m_ObjectHideFlags: 1");
                sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
                sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
                sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
                sb.AppendLine("  m_GameObject: {fileID: 0}");
                sb.AppendLine("  m_Enabled: 1");
                sb.AppendLine("  serializedVersion: 2");
                sb.AppendLine("  m_LocalReferencePoint: {x: 0, y: 0, z: 0}");
                sb.AppendLine("  m_Size: 1");
                sb.AppendLine("  m_LODs:");

                // LOD0
                sb.AppendLine($"  - screenRelativeTransitionHeight: {(asset.ScreenSizes.LOD0Min / 100f):F2}");
                sb.AppendLine("    fadeTransitionWidth: 0.5");
                sb.AppendLine("    renderers:");
                sb.AppendLine("    - {fileID: 0}");

                // LOD1
                sb.AppendLine($"  - screenRelativeTransitionHeight: {(asset.ScreenSizes.LOD1Min / 100f):F2}");
                sb.AppendLine("    fadeTransitionWidth: 0.5");
                sb.AppendLine("    renderers:");
                sb.AppendLine("    - {fileID: 0}");

                // LOD2
                sb.AppendLine("  - screenRelativeTransitionHeight: 0.01");
                sb.AppendLine("    fadeTransitionWidth: 0.5");
                sb.AppendLine("    renderers:");
                sb.AppendLine("    - {fileID: 0}");

                sb.AppendLine("  m_AnimateCrossFading: 1");
                sb.AppendLine("  m_AnimationCrossFadingTransitionTime: 0.5");
                sb.AppendLine("  m_LODData: []");
            }

            // Animator component if rigged
            if (asset.Skeleton != null)
            {
                sb.AppendLine("--- !u!95 &" + GenerateGUID());
                sb.AppendLine("Animator:");
                sb.AppendLine("  serializedVersion: 5");
                sb.AppendLine("  m_ObjectHideFlags: 1");
                sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
                sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
                sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
                sb.AppendLine("  m_GameObject: {fileID: 0}");
                sb.AppendLine("  m_Enabled: 1");
                sb.AppendLine("  m_Avatar: {fileID: 0}");
                sb.AppendLine("  m_Controller: {fileID: 0}");
                sb.AppendLine("  m_CullingMode: 0");
                sb.AppendLine("  m_UpdateMode: 0");
                sb.AppendLine("  m_ApplyRootMotion: 0");
                sb.AppendLine("  m_LinearVelocityBlending: 0");
                sb.AppendLine("  m_WarningMessage: ''");
                sb.AppendLine("  m_HasTransformHierarchy: 1");
                sb.AppendLine("  m_AllowConstantClipSamplingOptimization: 1");
                sb.AppendLine("  m_KeepAnimatorStateOnDisable: 0");
                sb.AppendLine("  m_WriteDefaultValuesOnDisable: 0");
            }

            sb.AppendLine("--- !u!1027 &" + GenerateGUID());
            sb.AppendLine("AssetMetadata:");
            sb.AppendLine($"  assetId: {asset.AssetId}");
            sb.AppendLine($"  version: {(int)asset.OptimizedAt.Ticks}");
            sb.AppendLine($"  optimizedAt: {asset.OptimizedAt:O}");
            sb.AppendLine($"  lod0PolyCount: {asset.LOD0!.TriangleCount}");
            sb.AppendLine($"  lod1PolyCount: {asset.LOD1!.TriangleCount}");
            sb.AppendLine($"  lod2PolyCount: {asset.LOD2!.TriangleCount}");

            return sb.ToString();
        }

        private long _fileIdCounter = 100000;

        private long GenerateFileID()
        {
            // Use deterministic IDs based on counter for reproducible builds
            return _fileIdCounter++;
        }

        private string GenerateGUID() => Guid.NewGuid().ToString("N").Substring(0, 16);
        private string GenerateGUIDString() => Guid.NewGuid().ToString();
        private long GenerateMeshFileID(MeshData mesh) => mesh.VertexCount.GetHashCode();
        private long GenerateMaterialFileID(MaterialData material) => material.Name.GetHashCode();
    }
}
