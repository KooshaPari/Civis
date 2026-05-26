#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Tools.PackCompiler.Models;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Service for validating imported and optimized assets against configuration targets.
    /// Checks polycount targets, scale ranges, material references, LOD sanity, and skeleton integrity.
    /// </summary>
    public class AssetValidationService
    {
        private const float MinScaleFactor = 0.1f;
        private const float MaxScaleFactor = 10f;
        private const int MinPolyCount = 100;
        private const int MaxPolyCount = 500000;

        /// <summary>
        /// Validate an imported asset against its definition config.
        /// Returns validation result with errors and warnings.
        /// </summary>
        public ValidationResult ValidateImportedAsset(ImportedAsset asset, AssetDefinition definition)
        {
            var errors = new List<string>();
            var warnings = new List<string>();

            // Check: Asset ID matches
            if (asset.AssetId != definition.Id)
            {
                errors.Add($"Asset ID mismatch: expected '{definition.Id}', got '{asset.AssetId}'");
            }

            // Check: Mesh exists and is non-empty
            if (asset.Mesh == null)
            {
                errors.Add("Asset has no mesh data");
                return new ValidationResult { IsValid = false, Errors = errors };
            }

            if (asset.Mesh.VertexCount == 0 || asset.Mesh.Indices.Length == 0)
            {
                errors.Add("Mesh is empty (no vertices or indices)");
                return new ValidationResult { IsValid = false, Errors = errors };
            }

            // Check: Polycount bounds
            int actualPolyCount = asset.Mesh.TriangleCount;
            if (actualPolyCount < MinPolyCount)
            {
                errors.Add($"Polycount too low: {actualPolyCount} (minimum: {MinPolyCount})");
            }
            if (actualPolyCount > MaxPolyCount)
            {
                errors.Add($"Polycount exceeds limit: {actualPolyCount} (maximum: {MaxPolyCount})");
            }

            // Check: Polycount reasonably close to target (within 50% variance)
            int targetPolyCount = definition.PolyCountTarget;
            float variance = Math.Abs((actualPolyCount - targetPolyCount) / (float)targetPolyCount);
            if (variance > 0.5f)
            {
                warnings.Add($"Polycount variance: {actualPolyCount} vs target {targetPolyCount} ({variance * 100:F1}% off)");
            }

            // Check: Scale factor in valid range
            if (definition.Scale < MinScaleFactor || definition.Scale > MaxScaleFactor)
            {
                errors.Add($"Scale out of range: {definition.Scale} (valid: {MinScaleFactor}-{MaxScaleFactor})");
            }

            // Check: Materials
            if (asset.Materials.Count == 0 && definition.Material != "none")
            {
                warnings.Add("Asset has no materials but material is expected in config");
            }

            foreach (var material in asset.Materials)
            {
                if (material.IsFallbackMaterial)
                {
                    warnings.Add(
                        $"Material '{material.Name}' is placeholder URP Lit metadata and may render flat until authored shader settings are imported."
                    );
                }

                if (material.MaterialWarnings.Count > 0)
                {
                    warnings.AddRange(material.MaterialWarnings);
                }
            }

            // Check: Material reference exists
            if (string.IsNullOrEmpty(definition.Material) || definition.Material == "default")
            {
                warnings.Add("Material reference is not specified; will use default");
            }

            // Check: Normals and UVs for rendering
            if (asset.Mesh.Normals == null || asset.Mesh.Normals.Length == 0)
            {
                warnings.Add("Asset has no normals; they will be computed at runtime");
            }

            if (asset.Mesh.UVs == null || asset.Mesh.UVs.Length == 0)
            {
                warnings.Add("Asset has no UV coordinates; textures will not map correctly");
            }

            // Check: Skeleton integrity if rigged
            if (definition.Type.Contains("hero") && asset.Skeleton == null && asset.Mesh.BoneWeights == null)
            {
                warnings.Add("Asset is hero unit but has no skeleton/bone data");
            }

            if (asset.Skeleton != null)
            {
                ValidateSkeleton(asset.Skeleton, errors, warnings);
            }

            // Check: Bounds calculated
            if (asset.Mesh.Bounds == null || asset.Mesh.Bounds.Value.Min == null)
            {
                errors.Add("Asset bounds not calculated");
            }

            return new ValidationResult
            {
                IsValid = errors.Count == 0,
                Errors = errors,
                Warnings = warnings,
                ActualPolyCount = actualPolyCount,
                TargetPolyCount = targetPolyCount
            };
        }

        /// <summary>
        /// Validate an optimized asset before prefab generation.
        /// Checks LOD mesh sanity, screen size thresholds, and optimization quality.
        /// </summary>
        public ValidationResult ValidateOptimizedAsset(OptimizedAsset asset, AssetDefinition definition)
        {
            var errors = new List<string>();
            var warnings = new List<string>();

            // Check: All LOD variants exist
            if (asset.LOD0 == null || asset.LOD0.VertexCount == 0)
            {
                errors.Add("LOD0 (full detail) is missing or empty");
            }

            if (asset.LOD1 == null || asset.LOD1.VertexCount == 0)
            {
                errors.Add("LOD1 (medium detail) is missing or empty");
            }

            if (asset.LOD2 == null || asset.LOD2.VertexCount == 0)
            {
                errors.Add("LOD2 (low detail) is missing or empty");
            }

            if (errors.Count > 0)
            {
                return new ValidationResult { IsValid = false, Errors = errors };
            }

            // Check: LOD progression is sensible (each should be smaller than previous)
            // Null-forgiving: LOD0/1/2 were validated non-null above; compiler can't track through early-return
            int lod0Poly = asset.LOD0!.TriangleCount;
            int lod1Poly = asset.LOD1!.TriangleCount;
            int lod2Poly = asset.LOD2!.TriangleCount;

            if (lod1Poly >= lod0Poly)
            {
                errors.Add($"LOD1 ({lod1Poly}) should be smaller than LOD0 ({lod0Poly})");
            }

            if (lod2Poly >= lod1Poly)
            {
                errors.Add($"LOD2 ({lod2Poly}) should be smaller than LOD1 ({lod1Poly})");
            }

            // Check: LOD percentages are reasonable
            float lod1Percent = asset.Metadata.LOD1Quality;
            float lod2Percent = asset.Metadata.LOD2Quality;

            if (lod1Percent < 40f || lod1Percent > 80f)
            {
                warnings.Add($"LOD1 quality {lod1Percent:F1}% outside typical range (40-80%)");
            }

            if (lod2Percent < 20f || lod2Percent > 50f)
            {
                warnings.Add($"LOD2 quality {lod2Percent:F1}% outside typical range (20-50%)");
            }

            // Check: Screen size thresholds make sense
            if (asset.ScreenSizes.LOD0Max <= asset.ScreenSizes.LOD0Min)
            {
                errors.Add($"LOD0 screen size range invalid: max ({asset.ScreenSizes.LOD0Max}) <= min ({asset.ScreenSizes.LOD0Min})");
            }

            if (asset.ScreenSizes.LOD1Max <= asset.ScreenSizes.LOD1Min)
            {
                errors.Add($"LOD1 screen size range invalid: max ({asset.ScreenSizes.LOD1Max}) <= min ({asset.ScreenSizes.LOD1Min})");
            }

            // Check: Screen size ranges don't overlap incorrectly
            if (asset.ScreenSizes.LOD0Min != asset.ScreenSizes.LOD1Max)
            {
                warnings.Add("LOD0Min and LOD1Max should match for smooth transitions");
            }

            if (asset.ScreenSizes.LOD1Min != asset.ScreenSizes.LOD2Min)
            {
                warnings.Add("LOD1Min and LOD2Min should match for smooth transitions");
            }

            // Check: Optimization time reasonable
            if (asset.Metadata.OptimizationTimeSeconds > 60)
            {
                warnings.Add($"Optimization took {asset.Metadata.OptimizationTimeSeconds}s (longer than typical)");
            }

            // Check: Skeleton preserved if original was rigged
            if (asset.Skeleton == null && definition.Type.Contains("hero"))
            {
                warnings.Add("Skeleton was lost during optimization for hero unit");
            }

            return new ValidationResult
            {
                IsValid = errors.Count == 0,
                Errors = errors,
                Warnings = warnings,
                ActualPolyCount = lod0Poly,
                TargetPolyCount = definition.PolyCountTarget
            };
        }

        /// <summary>
        /// Validate asset pipeline configuration for completeness and consistency.
        /// </summary>
        public ValidationResult ValidateConfiguration(AssetPipelineConfig config)
        {
            var errors = new List<string>();
            var warnings = new List<string>();

            // Check: Required fields
            if (string.IsNullOrWhiteSpace(config.Version))
            {
                errors.Add("Version not specified in config");
            }

            if (string.IsNullOrWhiteSpace(config.PackId))
            {
                errors.Add("PackId not specified in config");
            }

            if (config.AssetSettings == null)
            {
                errors.Add("AssetSettings is null");
                return new ValidationResult { IsValid = false, Errors = errors };
            }

            // Check: Output paths are defined
            if (string.IsNullOrWhiteSpace(config.AssetSettings.OutputPath))
            {
                errors.Add("OutputPath not specified");
            }

            if (config.Build == null)
            {
                errors.Add("BuildConfig is null");
            }
            else
            {
                if (string.IsNullOrWhiteSpace(config.Build.OutputDirectory))
                {
                    errors.Add("Build.OutputDirectory not specified");
                }

                if (string.IsNullOrWhiteSpace(config.Build.AddressablesOutput))
                {
                    errors.Add("Build.AddressablesOutput not specified");
                }
            }

            // Check: Materials defined
            if (config.Materials == null || config.Materials.Count == 0)
            {
                warnings.Add("No materials defined in config");
            }

            // Check: At least one phase with assets
            if (config.Phases == null || config.Phases.Count == 0)
            {
                errors.Add("No asset phases defined");
            }
            else
            {
                int totalAssets = 0;
                foreach (var phase in config.Phases.Values)
                {
                    if (phase.Models == null || phase.Models.Count == 0)
                    {
                        warnings.Add($"Phase '{phase.Description}' has no models");
                    }
                    else
                    {
                        totalAssets += phase.Models.Count;

                        // Validate each asset definition
                        foreach (var asset in phase.Models)
                        {
                            ValidateAssetDefinition(asset, config, errors, warnings);
                        }
                    }
                }

                if (totalAssets == 0)
                {
                    errors.Add("No assets defined across all phases");
                }
            }

            return new ValidationResult
            {
                IsValid = errors.Count == 0,
                Errors = errors,
                Warnings = warnings
            };
        }

        private void ValidateSkeleton(SkeletonData skeleton, List<string> errors, List<string> warnings)
        {
            if (skeleton.Bones.Count == 0)
            {
                errors.Add("Skeleton has no bones");
                return;
            }

            if (string.IsNullOrWhiteSpace(skeleton.RootBone))
            {
                errors.Add("Skeleton root bone not specified");
            }

            var boneNames = new HashSet<string>(skeleton.Bones.Select(b => b.Name));

            // Check: Root bone exists
            if (!string.IsNullOrWhiteSpace(skeleton.RootBone) && !boneNames.Contains(skeleton.RootBone))
            {
                errors.Add($"Root bone '{skeleton.RootBone}' not found in bone list");
            }

            // Check: Parent references are valid
            foreach (var bone in skeleton.Bones)
            {
                if (!string.IsNullOrWhiteSpace(bone.ParentName) && !boneNames.Contains(bone.ParentName))
                {
                    warnings.Add($"Bone '{bone.Name}' references non-existent parent '{bone.ParentName}'");
                }

                // Check: Quaternion is normalized
                if (bone.LocalRotation.Length == 4)
                {
                    float magnitude = (float)Math.Sqrt(
                        bone.LocalRotation[0] * bone.LocalRotation[0] +
                        bone.LocalRotation[1] * bone.LocalRotation[1] +
                        bone.LocalRotation[2] * bone.LocalRotation[2] +
                        bone.LocalRotation[3] * bone.LocalRotation[3]
                    );

                    if (Math.Abs(magnitude - 1.0f) > 0.01f)
                    {
                        warnings.Add($"Bone '{bone.Name}' quaternion not normalized (magnitude: {magnitude:F3})");
                    }
                }
            }
        }

        private void ValidateAssetDefinition(AssetDefinition asset, AssetPipelineConfig config, List<string> errors, List<string> warnings)
        {
            // Check: Required fields
            if (string.IsNullOrWhiteSpace(asset.Id))
            {
                errors.Add("Asset definition missing Id");
                return;
            }

            if (string.IsNullOrWhiteSpace(asset.File))
            {
                errors.Add($"Asset '{asset.Id}' missing File path");
            }

            if (string.IsNullOrWhiteSpace(asset.Faction))
            {
                errors.Add($"Asset '{asset.Id}' missing Faction");
            }

            // Check: Material reference exists
            if (!string.IsNullOrWhiteSpace(asset.Material) && asset.Material != "default" && !config.Materials.ContainsKey(asset.Material))
            {
                errors.Add($"Asset '{asset.Id}' references non-existent material '{asset.Material}'");
            }

            // Check: LOD config
            if (asset.LOD == null)
            {
                errors.Add($"Asset '{asset.Id}' missing LOD definition");
            }
            else if (asset.LOD.Levels == null || asset.LOD.Levels.Count == 0)
            {
                errors.Add($"Asset '{asset.Id}' LOD levels not specified");
            }
            else
            {
                // Check: LOD levels are descending
                for (int i = 1; i < asset.LOD.Levels.Count; i++)
                {
                    if (asset.LOD.Levels[i] >= asset.LOD.Levels[i - 1])
                    {
                        errors.Add($"Asset '{asset.Id}' LOD levels not in descending order");
                        break;
                    }
                }
            }

            // Check: Addressable key
            if (string.IsNullOrWhiteSpace(asset.AddressableKey))
            {
                errors.Add($"Asset '{asset.Id}' missing AddressableKey");
            }

            // Check: Output prefab
            if (string.IsNullOrWhiteSpace(asset.OutputPrefab))
            {
                errors.Add($"Asset '{asset.Id}' missing OutputPrefab path");
            }

        }
    }

    /// <summary>
    /// Result of asset validation.
    /// </summary>
    public class ValidationResult
    {
        public bool IsValid { get; set; }
        public List<string> Errors { get; init; } = new();
        public List<string> Warnings { get; init; } = new();
        public int ActualPolyCount { get; set; }
        public int TargetPolyCount { get; set; }

        public override string ToString()
        {
            var lines = new List<string>();
            lines.Add($"Valid: {IsValid}");

            if (ActualPolyCount > 0)
            {
                lines.Add($"Polycount: {ActualPolyCount} (target: {TargetPolyCount})");
            }

            if (Errors.Count > 0)
            {
                lines.Add($"\nErrors ({Errors.Count}):");
                lines.AddRange(Errors.Select(e => $"  ✗ {e}"));
            }

            if (Warnings.Count > 0)
            {
                lines.Add($"\nWarnings ({Warnings.Count}):");
                lines.AddRange(Warnings.Select(w => $"  ⚠ {w}"));
            }

            return string.Join("\n", lines);
        }
    }
}
