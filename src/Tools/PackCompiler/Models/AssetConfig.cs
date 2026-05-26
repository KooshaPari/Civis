#nullable enable
using System;
using System.Collections.Generic;
using System.IO;

namespace DINOForge.Tools.PackCompiler.Models
{
    /// <summary>
    /// Asset pipeline configuration loaded from asset_pipeline.yaml.
    /// Defines all model imports, LOD targets, materials, and build outputs.
    /// </summary>
    public class AssetPipelineConfig
    {
        /// <summary>Default constructor for deserialization.</summary>
        public AssetPipelineConfig() { }

        /// <summary>Asset pipeline schema version (e.g., "1.0.0").</summary>
        public required string Version { get; init; }

        /// <summary>Pack identifier matching pack.yaml id.</summary>
        public required string PackId { get; init; }

        /// <summary>Target Unity version for generated asset bundles.</summary>
        public required string TargetUnityVersion { get; init; }

        /// <summary>Global asset import and optimization settings.</summary>
        public required AssetSettings AssetSettings { get; init; }

        /// <summary>Material definitions keyed by faction (e.g., republic, cis).</summary>
        public required Dictionary<string, MaterialDefinition> Materials { get; init; }

        /// <summary>Asset import phases (e.g., v0_8_1_infantry, v0_9_0_heroes).</summary>
        public required Dictionary<string, AssetPhase> Phases { get; init; }

        /// <summary>Build output and performance configuration.</summary>
        public required BuildConfig Build { get; init; }
    }

    /// <summary>Global asset pipeline settings</summary>
    public class AssetSettings
    {
        /// <summary>Base directory for source assets (GLB/FBX files).</summary>
        public required string BasePath { get; init; }

        /// <summary>Directory where bundled asset outputs are written.</summary>
        public required string OutputPath { get; init; }

        /// <summary>Subdirectory for material definitions (default: "materials").</summary>
        public string MaterialsPath { get; init; } = "materials";

        /// <summary>Texture quality: "low", "medium", or "high" (default: "high").</summary>
        public string TextureQuality { get; init; } = "high";  // low | medium | high

        /// <summary>LOD decimation strategy: "aggressive", "balanced", or "conservative" (default: "aggressive").</summary>
        public string LODStrategy { get; init; } = "aggressive";  // aggressive | balanced | conservative

        /// <summary>Computed path to imported assets subdirectory</summary>
        public string ImportedPath => Path.Combine(BasePath, "imported");

        /// <summary>Computed path to optimized assets subdirectory</summary>
        public string OptimizedPath => Path.Combine(BasePath, "optimized");

        /// <summary>Computed path to prefabs subdirectory</summary>
        public string PrefabsPath => Path.Combine(BasePath, "prefabs");
    }

    /// <summary>Material definition with faction colors</summary>
    public class MaterialDefinition
    {
        /// <summary>Faction identifier (e.g., "republic", "cis").</summary>
        public required string Faction { get; init; }

        /// <summary>Base material color in hex format #RRGGBB (e.g., "F5F5F5").</summary>
        public required string BaseColor { get; init; }  // #RRGGBB

        /// <summary>Emission/glow color in hex format #RRGGBB.</summary>
        public required string EmissionColor { get; init; }

        /// <summary>Emission intensity multiplier (0-2 typical range).</summary>
        public required float EmissionIntensity { get; init; }

        /// <summary>Surface roughness (0=glossy, 1=matte, default 0.5).</summary>
        public float Roughness { get; init; } = 0.5f;

        /// <summary>Metallic value (0=non-metal, 1=full metal, default 0).</summary>
        public float Metallic { get; init; } = 0f;
    }

    /// <summary>Asset import phase (v0.7.0, v0.8.0, etc.)</summary>
    public class AssetPhase
    {
        /// <summary>Human-readable phase description (e.g., "Clone Infantry Assets").</summary>
        public required string Description { get; init; }

        /// <summary>Collection of assets to import in this phase.</summary>
        public required List<AssetDefinition> Models { get; init; }
    }

    /// <summary>Individual asset definition</summary>
    public class AssetDefinition
    {
        /// <summary>Unique asset identifier (e.g., "sw-rep-clone-trooper").</summary>
        public required string Id { get; init; }

        /// <summary>Path to source file relative to BasePath (e.g., "infantry/clone.glb").</summary>
        public required string File { get; init; }

        /// <summary>Asset type classification (infantry, hero, heavy, vehicle, building, etc.).</summary>
        public required string Type { get; init; }  // infantry | hero | heavy | vehicle | building | etc.

        /// <summary>Faction identifier for material lookup.</summary>
        public required string Faction { get; init; }

        /// <summary>Target polygon count for LOD0 (optimization target).</summary>
        public required int PolyCountTarget { get; init; }

        /// <summary>Model scale multiplier (default 1.0).</summary>
        public float Scale { get; init; } = 1.0f;

        /// <summary>LOD generation configuration.</summary>
        public required LODDefinition LOD { get; init; }

        /// <summary>Material name for texture and color lookup.</summary>
        public required string Material { get; init; }

        /// <summary>Addressables catalog key for runtime asset loading.</summary>
        public required string AddressableKey { get; init; }

        /// <summary>Output prefab file path relative to OutputPath.</summary>
        public required string OutputPrefab { get; init; }

        /// <summary>Optional game definition auto-update configuration.</summary>
        public DefinitionUpdateConfig? UpdateDefinition { get; init; }

        /// <summary>Additional metadata (future extensibility).</summary>
        public Dictionary<string, object>? Metadata { get; init; }
    }

    /// <summary>LOD generation configuration</summary>
    public class LODDefinition
    {
        /// <summary>Whether LOD generation is enabled (default: true).</summary>
        public bool Enabled { get; init; } = true;

        /// <summary>LOD quality percentages (e.g., [100, 60, 30] for LOD0, LOD1, LOD2).</summary>
        public required List<int> Levels { get; init; }  // [100, 60, 30]

        /// <summary>Screen size transition thresholds (e.g., [100, 50, 20] in percentages).</summary>
        public List<int>? ScreenSizes { get; init; }  // [100, 50, 20]
    }

    /// <summary>Game definition auto-update configuration</summary>
    public class DefinitionUpdateConfig
    {
        /// <summary>Whether to auto-update game definitions after import (default: false).</summary>
        public bool Enabled { get; init; } = false;

        /// <summary>Path to game definition file (relative to pack root).</summary>
        public string? File { get; init; }

        /// <summary>Definition ID to update (e.g., unit or building ID).</summary>
        public string? Id { get; init; }

        /// <summary>Field to update in definition (typically "visual_asset").</summary>
        public string? Field { get; init; }  // visual_asset
    }

    /// <summary>Build output configuration</summary>
    public class BuildConfig
    {
        /// <summary>Directory for finalized asset bundle outputs.</summary>
        public required string OutputDirectory { get; init; }

        /// <summary>Directory for generated Addressables catalog entries.</summary>
        public required string AddressablesOutput { get; init; }

        /// <summary>Optional log file path for build process output.</summary>
        public string? LogFile { get; init; }

        /// <summary>Whether to generate an HTML report after build (default: false).</summary>
        public bool GenerateHtmlReport { get; init; } = false;

        /// <summary>Optional performance targets for validation.</summary>
        public PerformanceTargets? PerformanceTargets { get; init; }
    }

    /// <summary>Performance/timing targets</summary>
    public class PerformanceTargets
    {
        /// <summary>Maximum seconds for asset import phase (default: 5).</summary>
        public int ImportTimeSeconds { get; init; } = 5;

        /// <summary>Maximum seconds for LOD generation (default: 10).</summary>
        public int LODGenerationSeconds { get; init; } = 10;

        /// <summary>Maximum seconds for prefab generation (default: 2).</summary>
        public int PrefabGenerationSeconds { get; init; } = 2;

        /// <summary>Maximum seconds for total pipeline execution (default: 120).</summary>
        public int TotalPipelineSeconds { get; init; } = 120;
    }
}
