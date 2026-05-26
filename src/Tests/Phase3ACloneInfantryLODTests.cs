using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;
using Xunit.Sdk;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tests
{
    /// <summary>
    /// Phase 3A: Clone Infantry LOD Optimization Tests
    ///
    /// Tests for importing Republic Clone trooper specialist units and generating LOD variants.
    /// Covers 5 Clone infantry specialist units: Sharpshooter, Heavy, Medic, ARF Trooper, Militia.
    ///
    /// Expected outcomes:
    /// - 5 Clone infantry units with raw GLB files present
    /// - LOD targets configured: 100%, 60%, 30% per unit (15 variants total)
    /// - Material definitions: Republic faction colors (blue, #4488FF)
    /// - Configuration validation: All units have required fields
    /// - Asset references: Ready for visual_asset field injection in republic_units.yaml
    /// </summary>
    public class Phase3ACloneInfantryLODTests
    {
        private static readonly string RepoRoot = GetRepoRoot();
        private string AssetPipelineYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/asset_pipeline.yaml");
        private string RepublicUnitsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/units/republic_units.yaml");
        private string RawAssetsPath => Path.Combine(RepoRoot, "packs/warfare-starwars/assets/raw");

        private static string GetRepoRoot()
        {
            var currentDir = AppContext.BaseDirectory;
            while (currentDir != null && !Directory.Exists(Path.Combine(currentDir, "packs")))
            {
                currentDir = Path.GetDirectoryName(currentDir);
            }
            return currentDir ?? throw new InvalidOperationException("Could not find repo root");
        }

        // ── Phase 3A Configuration & Inventory ───────────────────────────────

        [Fact]
        public void Phase3A_Configuration_Exists()
        {
            File.Exists(AssetPipelineYamlPath).Should().BeTrue(
                $"Asset pipeline configuration must exist at {AssetPipelineYamlPath}");
        }

        [Fact]
        public void Phase3A_Configuration_ContainsV0_8_1_Infantry()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain("v0_8_1_infantry");
            yaml.Should().Contain("Clone infantry variants");
        }

        [Fact]
        public void Phase3A_Configuration_Has5CloneUnits()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);

            // Parse YAML to count Phase 3A models
            var deserializer = YamlLoader.Deserializer;

            // Extract just the v0_8_1_infantry section
            var phaseStart = yaml.IndexOf("v0_8_1_infantry:");
            phaseStart.Should().BeGreaterThan(-1, "v0_8_1_infantry phase must exist");

            var phaseEnd = yaml.IndexOf("\n  v0_9_0_cis_droids:", phaseStart);
            if (phaseEnd == -1) phaseEnd = yaml.IndexOf("\n  v0_9_0_future:", phaseStart);
            if (phaseEnd == -1) phaseEnd = yaml.Length;

            var phaseYaml = yaml.Substring(phaseStart, phaseEnd - phaseStart);

            // Count model entries (lines starting with "- id:")
            var modelCount = phaseYaml.Split(new[] { "\n      - id:" }, StringSplitOptions.None).Length - 1;
            modelCount.Should().BeGreaterThanOrEqualTo(5, "Phase 3A should have at least 5 Clone infantry units");
        }

        // ── Raw Assets (Conditional) ────────────────────────────────────────

        [Fact]
        public void Phase3A_Raw_Assets_ConditionalCheck()
        {
            // Check if raw assets directory exists and has content
            // Skip if assets not downloaded (CI/clean checkout)
            if (!Directory.Exists(RawAssetsPath))
            {
                return; // Assets not present - skip
            }

            // Check if any GLB files exist
            var hasAssets = Directory.GetFiles(RawAssetsPath, "*.glb", SearchOption.AllDirectories).Any();
            if (!hasAssets)
            {
                return; // No GLB files - skip
            }

            // If we get here, assets exist - run the check
            var expectedUnits = new[]
            {
                "rep_clone_sharpshooter_sketchfab_001",
                "rep_clone_heavy_sketchfab_001",
                "rep_clone_medic_sketchfab_001",
                "rep_arf_trooper_sketchfab_001",
                "rep_clone_militia_sketchfab_001"
            };

            int foundCount = 0;
            foreach (var unitDir in expectedUnits)
            {
                var unitDirPath = Path.Combine(RawAssetsPath, unitDir);
                if (Directory.Exists(unitDirPath))
                {
                    var hasGlb = Directory.GetFiles(unitDirPath, "*.glb").Any();
                    if (hasGlb) foundCount++;
                }
            }

            // At least verify we found some units if assets exist
            foundCount.Should().BeGreaterThan(0, "should find at least some clone infantry assets");
        }

        [Fact]
        public void Phase3A_RawAssets_Size_ConditionalCheck()
        {
            // Skip if assets not present
            if (!Directory.Exists(RawAssetsPath))
            {
                return;
            }

            var glbFiles = Directory.GetFiles(RawAssetsPath, "*.glb", SearchOption.AllDirectories);
            if (glbFiles.Length == 0)
            {
                return;
            }

            // If we have assets, verify they're reasonable size
            long totalSize = glbFiles.Sum(f => new FileInfo(f).Length);
            totalSize.Should().BeGreaterThan(0, "GLB files should not be empty");
        }

        // ── LOD Configuration Validation ──────────────────────────────────────

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_CloneUnit_ConfiguredCorrectly(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);

            // Verify unit exists in config
            yaml.Should().Contain($"- id: {unitId}", $"Unit {unitId} must be defined in asset_pipeline.yaml");

            // Verify LOD configuration
            var unitSection = ExtractUnitSection(yaml, unitId);
            unitSection.Should().Contain("enabled: true", $"{unitId} must have LOD enabled");
            unitSection.Should().Contain("levels: [100, 60, 30]", $"{unitId} must have LOD levels [100, 60, 30]");
            unitSection.Should().Contain("screen_sizes: [100, 50, 20]", $"{unitId} must have screen sizes [100, 50, 20]");

            // Verify material assignment (Republic faction)
            unitSection.Should().Contain("material: republic", $"{unitId} must use Republic material");
            unitSection.Should().Contain("faction: republic", $"{unitId} must be Republic faction");
        }

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_CloneUnit_HasAddressableKey(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("addressable_key:", $"{unitId} must have addressable_key defined");
            unitSection.Should().Contain("sw-", $"{unitId} addressable key should follow sw- naming convention");
        }

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_CloneUnit_HasOutputPrefab(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("output_prefab:", $"{unitId} must have output_prefab defined");
            unitSection.Should().Contain(".prefab", $"{unitId} output_prefab should end with .prefab");
        }

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_CloneUnit_ReferencesRawGlbPath_InConfiguration(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("file: raw/", $"{unitId} must point at a raw asset path in config");
            unitSection.Should().Contain("model.glb", $"{unitId} must target a GLB model in config");
            unitSection.Should().NotContain("TODO", $"{unitId} raw asset reference should be production-like config");
        }

        [Fact]
        public void Phase3A_CloneUnits_UseDistinctRawGlbPaths()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitIds = new[]
            {
                "rep_clone_sharpshooter",
                "rep_clone_heavy",
                "rep_clone_medic",
                "rep_arf_trooper",
                "rep_clone_militia"
            };

            var rawPaths = unitIds
                .Select(unitId => ExtractConfigValue(ExtractUnitSection(yaml, unitId), "file"))
                .ToList();

            rawPaths.Should().OnlyHaveUniqueItems("each Phase 3A clone unit should map to a distinct source GLB path");
            rawPaths.Should().OnlyContain(path => path.StartsWith("raw/", StringComparison.Ordinal), "Phase 3A source assets should remain under the raw asset tree");
            rawPaths.Should().OnlyContain(path => path.EndsWith("model.glb", StringComparison.OrdinalIgnoreCase), "Phase 3A units should still reference GLB sources");
        }

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_CloneUnit_HasDefinitionUpdate(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("update_definition:", $"{unitId} must have update_definition");
            unitSection.Should().Contain("enabled: true", $"{unitId} update_definition must be enabled");
            unitSection.Should().Contain("file: units/republic_units.yaml", $"{unitId} must update units/republic_units.yaml");
            unitSection.Should().Contain("field: visual_asset", $"{unitId} must inject visual_asset field");
        }

        // ── Republic Material Definition ───────────────────────────────────────

        [Fact]
        public void Phase3A_RepublicMaterial_Defined()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);

            yaml.Should().Contain("republic:", "Republic material must be defined");
            yaml.Should().Contain("faction: republic", "Republic material must specify faction");
            yaml.Should().Contain("base_color: \"#4488FF\"", "Republic base color should be #4488FF (blue)");
            yaml.Should().Contain("emission_color: \"#2244FF\"", "Republic emission should be #2244FF");
            yaml.Should().Contain("emission_intensity: 1.5", "Republic emission intensity should be 1.5");
        }

        // ── Republic Units Definition ─────────────────────────────────────────

        [Fact]
        public void Phase3A_RepublicUnits_Yaml_Exists()
        {
            File.Exists(RepublicUnitsYamlPath).Should().BeTrue(
                $"Republic units definition must exist at {RepublicUnitsYamlPath}");
        }

        [Fact]
        public void Phase3A_RepublicUnits_Has5PlusUnits()
        {
            var yaml = File.ReadAllText(RepublicUnitsYamlPath, System.Text.Encoding.UTF8);

            // Count units (lines starting with "- id:")
            var unitLines = yaml.Split('\n').Where(l => l.TrimStart().StartsWith("- id:")).ToList();
            unitLines.Count.Should().BeGreaterThanOrEqualTo(5, "Republic units should have at least 5 base Clone infantry units (may have additional variants)");
        }

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_RepublicUnit_Defined(string unitId)
        {
            var yaml = File.ReadAllText(RepublicUnitsYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"- id: {unitId}", $"Unit {unitId} must be defined in republic_units.yaml");
        }

        [Theory]
        [InlineData("rep_clone_sharpshooter")]
        [InlineData("rep_clone_heavy")]
        [InlineData("rep_clone_medic")]
        [InlineData("rep_arf_trooper")]
        [InlineData("rep_clone_militia")]
        public void Phase3A_RepublicUnit_IsRepublicFaction(string unitId)
        {
            var yaml = File.ReadAllText(RepublicUnitsYamlPath, System.Text.Encoding.UTF8);
            var unitStart = yaml.IndexOf($"- id: {unitId}");

            if (unitStart == -1)
                Assert.Fail($"Unit {unitId} not found in YAML");

            // Find next unit (- id:) or end of file
            var nextUnit = yaml.IndexOf("\n- id:", unitStart + 1);
            if (nextUnit == -1) nextUnit = yaml.Length;

            var unitSection = yaml.Substring(unitStart, nextUnit - unitStart);
            unitSection.Should().Contain("faction_id: republic", $"{unitId} must belong to Republic faction");
        }

        // ── Helper Methods ───────────────────────────────────────────────────

        private string ExtractUnitSection(string yaml, string unitId)
        {
            var start = yaml.IndexOf($"- id: {unitId}");
            if (start == -1) Assert.Fail($"Unit {unitId} not found in YAML");

            // Find the next unit or end of phases
            var nextUnit = yaml.IndexOf("\n      - id:", start + 1);
            if (nextUnit == -1) nextUnit = yaml.IndexOf("\n  v0_", start + 1);
            if (nextUnit == -1) nextUnit = yaml.Length;

            return yaml.Substring(start, nextUnit - start);
        }

        private static string ExtractConfigValue(string unitSection, string key)
        {
            var prefix = $"{key}: ";
            var line = unitSection
                .Split('\n')
                .Select(line => line.Trim())
                .FirstOrDefault(line => line.StartsWith(prefix, StringComparison.Ordinal));

            line.Should().NotBeNull($"expected config key '{key}' to exist in unit section");
            return line![prefix.Length..].Trim();
        }
    }
}

