using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Phase 4: Building Structures LOD Optimization Tests
    ///
    /// Tests for importing building GLB files and generating LOD variants.
    /// Covers 10 buildings: 5 Republic, 5 CIS.
    /// </summary>
    public class Phase4BuildingLODTests
    {
        private static readonly string RepoRoot = GetRepoRoot();
        private string AssetPipelineYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/asset_pipeline.yaml");
        private string RepublicBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/buildings/republic_buildings.yaml");
        private string CISBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/buildings/cis_buildings.yaml");

        private static string GetRepoRoot()
        {
            var currentDir = AppContext.BaseDirectory;
            while (currentDir != null && !Directory.Exists(Path.Combine(currentDir, "packs")))
            {
                currentDir = Path.GetDirectoryName(currentDir);
            }
            return currentDir ?? throw new InvalidOperationException("Could not find repo root");
        }

        // ── Building configuration validation ──────────────────────────────

        [Fact]
        public void Phase4_Configuration_Exists()
        {
            File.Exists(AssetPipelineYamlPath).Should().BeTrue(
                $"Asset pipeline configuration must exist at {AssetPipelineYamlPath}");
        }

        [Fact]
        public void Phase4_Configuration_ContainsV1_0_0_Buildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain("v1_0_0_buildings");
            yaml.Should().Contain("Phase 4: Building structures LOD optimization");
        }

        [Fact]
        public void Phase4_RepublicBuildingsDefinition_Exists()
        {
            File.Exists(RepublicBuildingsYamlPath).Should().BeTrue(
                $"Republic buildings definition must exist at {RepublicBuildingsYamlPath}");
        }

        [Fact]
        public void Phase4_CISBuildingsDefinition_Exists()
        {
            File.Exists(CISBuildingsYamlPath).Should().BeTrue(
                $"CIS buildings definition must exist at {CISBuildingsYamlPath}");
        }

        // ── Building model configuration tests ──────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks")]
        [InlineData("rep_weapons_factory")]
        [InlineData("rep_vehicle_bay")]
        [InlineData("rep_guard_tower")]
        [InlineData("rep_shield_generator")]
        public void Phase4_RepublicBuilding_ConfiguredCorrectly(string modelId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"- id: {modelId}")
                .And.Contain($"faction: republic")
                .And.Contain("type: building");
        }

        [Theory]
        [InlineData("cis_droid_factory")]
        [InlineData("cis_assembly_line")]
        [InlineData("cis_heavy_foundry")]
        [InlineData("cis_sentry_turret")]
        [InlineData("cis_ray_shield")]
        public void Phase4_CISBuilding_ConfiguredCorrectly(string modelId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"- id: {modelId}")
                .And.Contain("faction: cis")
                .And.Contain("type: building");
        }

        // ── LOD configuration tests ────────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks")]
        [InlineData("rep_weapons_factory")]
        [InlineData("rep_vehicle_bay")]
        [InlineData("rep_guard_tower")]
        [InlineData("rep_shield_generator")]
        [InlineData("cis_droid_factory")]
        [InlineData("cis_assembly_line")]
        [InlineData("cis_heavy_foundry")]
        [InlineData("cis_sentry_turret")]
        [InlineData("cis_ray_shield")]
        public void Phase4_Building_HasLODConfiguration(string buildingId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            startIdx.Should().BeGreaterThan(-1, $"Building {buildingId} not found in configuration");

            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);

            // Verify LOD configuration
            section.Should().Contain("lod:")
                .And.Contain("levels: [100, 80, 50]")
                .And.Contain("screen_sizes: [200, 100, 50]")
                .And.Contain("enabled: true");
        }

        // ── Addressables configuration tests ───────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks", "sw-clone-barracks")]
        [InlineData("rep_weapons_factory", "sw-weapons-factory")]
        [InlineData("rep_vehicle_bay", "sw-vehicle-bay")]
        [InlineData("rep_guard_tower", "sw-guard-tower")]
        [InlineData("rep_shield_generator", "sw-shield-generator")]
        [InlineData("cis_droid_factory", "sw-droid-factory")]
        [InlineData("cis_assembly_line", "sw-assembly-line")]
        [InlineData("cis_heavy_foundry", "sw-heavy-foundry")]
        [InlineData("cis_sentry_turret", "sw-sentry-turret")]
        [InlineData("cis_ray_shield", "sw-ray-shield")]
        public void Phase4_Building_HasAddressableKey(string buildingId, string expectedKey)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain($"addressable_key: {expectedKey}");
        }

        // ── Prefab output path tests ───────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks")]
        [InlineData("rep_weapons_factory")]
        [InlineData("rep_vehicle_bay")]
        [InlineData("rep_guard_tower")]
        [InlineData("rep_shield_generator")]
        [InlineData("cis_droid_factory")]
        [InlineData("cis_assembly_line")]
        [InlineData("cis_heavy_foundry")]
        [InlineData("cis_sentry_turret")]
        [InlineData("cis_ray_shield")]
        public void Phase4_Building_HasPrefabOutputPath(string buildingId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain("output_prefab: prefabs/")
                .And.Contain(".prefab");
        }

        // ── Definition update configuration tests ──────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks", "republic_buildings.yaml")]
        [InlineData("rep_weapons_factory", "republic_buildings.yaml")]
        [InlineData("rep_vehicle_bay", "republic_buildings.yaml")]
        [InlineData("rep_guard_tower", "republic_buildings.yaml")]
        [InlineData("rep_shield_generator", "republic_buildings.yaml")]
        [InlineData("cis_droid_factory", "cis_buildings.yaml")]
        [InlineData("cis_assembly_line", "cis_buildings.yaml")]
        [InlineData("cis_heavy_foundry", "cis_buildings.yaml")]
        [InlineData("cis_sentry_turret", "cis_buildings.yaml")]
        [InlineData("cis_ray_shield", "cis_buildings.yaml")]
        public void Phase4_Building_HasDefinitionUpdate(string buildingId, string expectedFile)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            startIdx.Should().BeGreaterThan(-1, $"Building {buildingId} should be in asset pipeline");

            var endIdx = yaml.IndexOf("\n      - id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.IndexOf("\n  v1_1_0_buildings_expansion:", startIdx);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain("update_definition:")
                .And.Contain("enabled: true")
                .And.Contain($"file: buildings/{expectedFile}")
                .And.Contain("field: visual_asset");
        }

        // ── Building definition tests ──────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_facility")]
        [InlineData("rep_weapons_factory")]
        [InlineData("rep_vehicle_bay")]
        [InlineData("rep_guard_tower")]
        [InlineData("rep_shield_generator")]
        public void Phase4_RepublicBuilding_DefinedInYAML(string buildingId)
        {
            var yaml = File.ReadAllText(RepublicBuildingsYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"id: {buildingId}");
        }

        [Theory]
        [InlineData("cis_droid_factory")]
        [InlineData("cis_assembly_line")]
        [InlineData("cis_heavy_foundry")]
        [InlineData("cis_sentry_turret")]
        [InlineData("cis_ray_shield")]
        public void Phase4_CISBuilding_DefinedInYAML(string buildingId)
        {
            var yaml = File.ReadAllText(CISBuildingsYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"id: {buildingId}");
        }

        [Theory]
        [InlineData("rep_clone_facility", "barracks")]
        [InlineData("rep_weapons_factory", "barracks")]
        [InlineData("rep_vehicle_bay", "barracks")]
        [InlineData("rep_guard_tower", "defense")]
        [InlineData("rep_shield_generator", "defense")]
        public void Phase4_RepublicBuilding_HasCorrectType(string buildingId, string expectedType)
        {
            var yaml = File.ReadAllText(RepublicBuildingsYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            startIdx.Should().BeGreaterThan(-1);

            var endIdx = yaml.IndexOf("\n- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain($"building_type: {expectedType}");
        }

        [Theory]
        [InlineData("cis_droid_factory", "barracks")]
        [InlineData("cis_assembly_line", "barracks")]
        [InlineData("cis_heavy_foundry", "barracks")]
        [InlineData("cis_sentry_turret", "defense")]
        [InlineData("cis_ray_shield", "defense")]
        public void Phase4_CISBuilding_HasCorrectType(string buildingId, string expectedType)
        {
            var yaml = File.ReadAllText(CISBuildingsYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            startIdx.Should().BeGreaterThan(-1);

            var endIdx = yaml.IndexOf("\n- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain($"building_type: {expectedType}");
        }

        // ── Coverage and completeness tests ────────────────────────────────

        [Fact]
        public void Phase4_Configuration_Has10Buildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var section = yaml.Substring(yaml.IndexOf("v1_0_0_buildings:"),
                yaml.IndexOf("v1_1_0_buildings_expansion:") - yaml.IndexOf("v1_0_0_buildings:"));

            var count = section.Split(new[] { "- id: " }, StringSplitOptions.None).Length - 1;
            count.Should().BeGreaterThanOrEqualTo(10, "Phase 4 should configure at least 10 buildings");
        }

        [Fact]
        public void Phase4_Configuration_Has5RepublicBuildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var section = yaml.Substring(yaml.IndexOf("v1_0_0_buildings:"),
                yaml.IndexOf("v1_1_0_buildings_expansion:") - yaml.IndexOf("v1_0_0_buildings:"));

            var republicCount = section.Split(new[] { "faction: republic" }, StringSplitOptions.None).Length - 1;
            republicCount.Should().BeGreaterThanOrEqualTo(5, "Phase 4 should configure at least 5 Republic buildings");
        }

        [Fact]
        public void Phase4_Configuration_Has5CISBuildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var section = yaml.Substring(yaml.IndexOf("v1_0_0_buildings:"),
                yaml.IndexOf("v1_1_0_buildings_expansion:") - yaml.IndexOf("v1_0_0_buildings:"));

            var cisCount = section.Split(new[] { "faction: cis" }, StringSplitOptions.None).Length - 1;
            cisCount.Should().BeGreaterThanOrEqualTo(5, "Phase 4 should configure at least 5 CIS buildings");
        }

        // ── Polycount targets ──────────────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks", 45000)]
        [InlineData("rep_weapons_factory", 38000)]
        [InlineData("rep_vehicle_bay", 52000)]
        [InlineData("rep_guard_tower", 28000)]
        [InlineData("rep_shield_generator", 35000)]
        [InlineData("cis_droid_factory", 42000)]
        [InlineData("cis_assembly_line", 36000)]
        [InlineData("cis_heavy_foundry", 48000)]
        [InlineData("cis_sentry_turret", 22000)]
        [InlineData("cis_ray_shield", 32000)]
        public void Phase4_Building_HasReasonablePolycount(string buildingId, int expectedPolycount)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain($"polycount_target: {expectedPolycount}");
        }

        // ── Material assignment ────────────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks")]
        [InlineData("rep_weapons_factory")]
        [InlineData("rep_vehicle_bay")]
        [InlineData("rep_guard_tower")]
        [InlineData("rep_shield_generator")]
        public void Phase4_RepublicBuilding_HasRepublicMaterial(string buildingId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain("material: republic");
        }

        [Theory]
        [InlineData("cis_droid_factory")]
        [InlineData("cis_assembly_line")]
        [InlineData("cis_heavy_foundry")]
        [InlineData("cis_sentry_turret")]
        [InlineData("cis_ray_shield")]
        public void Phase4_CISBuilding_HasCISMaterial(string buildingId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain("material: cis");
        }

        // ── LOD screen size distances ──────────────────────────────────────

        [Fact]
        public void Phase4_Buildings_UseConsistentScreenSizes()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var section = yaml.Substring(yaml.IndexOf("v1_0_0_buildings:"),
                yaml.IndexOf("v1_1_0_buildings_expansion:") - yaml.IndexOf("v1_0_0_buildings:"));

            // All entries should have a valid screen_sizes array
            var screenSizeLines = section.Split('\n')
                .Where(l => l.Contains("screen_sizes:"))
                .ToList();

            screenSizeLines.Should().AllSatisfy(line =>
                (line.Contains("[200, 100, 50]") || line.Contains("[100, 50, 20]")).Should().BeTrue(
                    $"screen_sizes should be [200, 100, 50] or [100, 50, 20], got: {line.Trim()}")
            );
        }

        // ── Metadata and documentation ────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_barracks")]
        [InlineData("rep_weapons_factory")]
        [InlineData("rep_vehicle_bay")]
        [InlineData("rep_guard_tower")]
        [InlineData("rep_shield_generator")]
        [InlineData("cis_droid_factory")]
        [InlineData("cis_assembly_line")]
        [InlineData("cis_heavy_foundry")]
        [InlineData("cis_sentry_turret")]
        [InlineData("cis_ray_shield")]
        public void Phase4_Building_HasMetadataNote(string buildingId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var startIdx = yaml.IndexOf($"- id: {buildingId}");
            var endIdx = yaml.IndexOf("- id: ", startIdx + 1);
            if (endIdx == -1) endIdx = yaml.Length;

            var section = yaml.Substring(startIdx, endIdx - startIdx);
            section.Should().Contain("priority: high")
                .And.Contain("note:");
        }
    }
}

