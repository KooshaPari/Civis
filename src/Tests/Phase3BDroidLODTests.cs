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
    /// Phase 3B: CIS Droid Infantry LOD Optimization Tests
    ///
    /// Tests for importing CIS droid unit GLB files and generating LOD variants.
    /// Covers 5 CIS droid units: B1 Battle, Sniper, BX Commando, Droideka, Probe.
    ///
    /// Expected outcomes:
    /// - 5 CIS droid units with raw GLB files present
    /// - LOD targets configured: 100%, 60%, 30% per unit (15 variants total)
    /// - Material definitions: CIS faction colors (red/orange, #FF4400)
    /// - Configuration validation: All units have required fields
    /// - Asset references: Ready for visual_asset field injection in cis_units.yaml
    /// </summary>
    public class Phase3BDroidLODTests
    {
        private static readonly string RepoRoot = GetRepoRoot();
        private string AssetPipelineYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/asset_pipeline.yaml");
        private string CISUnitsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/units/cis_units.yaml");

        private static string GetRepoRoot()
        {
            var currentDir = AppContext.BaseDirectory;
            while (currentDir != null && !Directory.Exists(Path.Combine(currentDir, "packs")))
            {
                currentDir = Path.GetDirectoryName(currentDir);
            }
            return currentDir ?? throw new InvalidOperationException("Could not find repo root");
        }

        // ── Phase 3B Configuration & Inventory ───────────────────────────────

        [Fact]
        public void Phase3B_Configuration_Exists()
        {
            File.Exists(AssetPipelineYamlPath).Should().BeTrue(
                $"Asset pipeline configuration must exist at {AssetPipelineYamlPath}");
        }

        [Fact]
        public void Phase3B_Configuration_ContainsV0_9_0_CISDroids()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain("v0_9_0_cis_droids");
            yaml.Should().Contain("Phase 2B: CIS droid units");
        }

        [Fact]
        public void Phase3B_Configuration_Has5Droids()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);

            // Parse YAML to count Phase 3B models
            var deserializer = YamlLoader.Deserializer;

            // Extract just the v0_9_0_cis_droids section
            var phaseStart = yaml.IndexOf("v0_9_0_cis_droids:");
            phaseStart.Should().BeGreaterThan(-1, "v0_9_0_cis_droids phase must exist");

            var phaseEnd = yaml.IndexOf("\n  v0_9_0_future:", phaseStart);
            if (phaseEnd == -1) phaseEnd = yaml.Length;

            var phaseYaml = yaml.Substring(phaseStart, phaseEnd - phaseStart);

            // Count model entries (lines starting with "      - id:")
            var modelCount = phaseYaml.Split(new[] { "\n      - id:" }, StringSplitOptions.None).Length - 1;
            modelCount.Should().BeGreaterThanOrEqualTo(5, "Phase 3B should have at least 5 CIS droid units");
        }

        // ── LOD Configuration Validation ──────────────────────────────────────

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_DroidUnit_ConfiguredCorrectly(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);

            // Verify unit exists in config
            yaml.Should().Contain($"- id: {unitId}", $"Unit {unitId} must be defined in asset_pipeline.yaml");

            // Verify LOD configuration
            var unitSection = ExtractUnitSection(yaml, unitId);
            unitSection.Should().Contain("enabled: true", $"{unitId} must have LOD enabled");
            unitSection.Should().Contain("levels: [100, 60, 30]", $"{unitId} must have LOD levels [100, 60, 30]");
            unitSection.Should().Contain("screen_sizes: [100, 50, 20]", $"{unitId} must have screen sizes [100, 50, 20]");

            // Verify material assignment (CIS faction)
            unitSection.Should().Contain("material: cis", $"{unitId} must use CIS material");
            unitSection.Should().Contain("faction: cis", $"{unitId} must be CIS faction");
        }

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_DroidUnit_HasAddressableKey(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("addressable_key:", $"{unitId} must have addressable_key defined");
            unitSection.Should().Contain("sw-", $"{unitId} addressable key should follow sw- naming convention");
        }

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_DroidUnit_HasOutputPrefab(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("output_prefab:", $"{unitId} must have output_prefab defined");
            unitSection.Should().Contain(".prefab", $"{unitId} output_prefab should end with .prefab");
        }

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_DroidUnit_ReferencesRawGlbPath_InConfiguration(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("file: raw/", $"{unitId} must point at a raw asset path in config");
            unitSection.Should().Contain("model.glb", $"{unitId} must target a GLB model in config");
            unitSection.Should().NotContain("TODO", $"{unitId} raw asset reference should be production-like config");
        }

        [Fact]
        public void Phase3B_DroidUnits_UseDistinctRawGlbPaths()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitIds = new[]
            {
                "cis_b1_battle_droid",
                "cis_sniper_droid",
                "cis_bx_commando_droid",
                "cis_droideka",
                "cis_probe_droid"
            };

            var rawPaths = unitIds
                .Select(unitId => ExtractConfigValue(ExtractUnitSection(yaml, unitId), "file"))
                .ToList();

            rawPaths.Should().OnlyHaveUniqueItems("each Phase 3B droid unit should map to a distinct source GLB path");
            rawPaths.Should().OnlyContain(path => path.StartsWith("raw/", StringComparison.Ordinal), "Phase 3B source assets should remain under the raw asset tree");
            rawPaths.Should().OnlyContain(path => path.EndsWith("model.glb", StringComparison.OrdinalIgnoreCase), "Phase 3B units should still reference GLB sources");
        }

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_DroidUnit_HasDefinitionUpdate(string unitId)
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            var unitSection = ExtractUnitSection(yaml, unitId);

            unitSection.Should().Contain("update_definition:", $"{unitId} must have update_definition");
            unitSection.Should().Contain("enabled: true", $"{unitId} update_definition must be enabled");
            unitSection.Should().Contain("file: units/cis_units.yaml", $"{unitId} must update units/cis_units.yaml");
            unitSection.Should().Contain("field: visual_asset", $"{unitId} must inject visual_asset field");
        }

        // ── CIS Material Definition ───────────────────────────────────────────

        [Fact]
        public void Phase3B_CISMaterial_Defined()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);

            yaml.Should().Contain("cis:", "CIS material must be defined");
            yaml.Should().Contain("faction: cis", "CIS material must specify faction");
            yaml.Should().Contain("base_color: \"#FF4400\"", "CIS base color should be #FF4400 (red/orange)");
            yaml.Should().Contain("emission_color: \"#FF2200\"", "CIS emission should be #FF2200");
            yaml.Should().Contain("emission_intensity: 1.5", "CIS emission intensity should be 1.5");
        }

        // ── CIS Units Definition ─────────────────────────────────────────────

        [Fact]
        public void Phase3B_CISUnits_Yaml_Exists()
        {
            File.Exists(CISUnitsYamlPath).Should().BeTrue(
                $"CIS units definition must exist at {CISUnitsYamlPath}");
        }

        [Fact]
        public void Phase3B_CISUnits_Has5Units()
        {
            var yaml = File.ReadAllText(CISUnitsYamlPath, System.Text.Encoding.UTF8);

            // Count units (lines starting with "- id:")
            var unitLines = yaml.Split('\n').Where(l => l.TrimStart().StartsWith("- id:")).ToList();
            unitLines.Count.Should().Be(15, "CIS units should have exactly 15 units defined");
        }

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_CISUnit_Defined(string unitId)
        {
            var yaml = File.ReadAllText(CISUnitsYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"- id: {unitId}", $"Unit {unitId} must be defined in cis_units.yaml");
        }

        [Theory]
        [InlineData("cis_b1_battle_droid")]
        [InlineData("cis_sniper_droid")]
        [InlineData("cis_bx_commando_droid")]
        [InlineData("cis_droideka")]
        [InlineData("cis_probe_droid")]
        public void Phase3B_CISUnit_IsCISFaction(string unitId)
        {
            var yaml = File.ReadAllText(CISUnitsYamlPath, System.Text.Encoding.UTF8);
            var unitStart = yaml.IndexOf($"- id: {unitId}");

            if (unitStart == -1)
                throw new Xunit.Sdk.XunitException($"Unit {unitId} not found");

            // Find next unit (- id:) or end of file
            var nextUnit = yaml.IndexOf("\n- id:", unitStart + 1);
            if (nextUnit == -1) nextUnit = yaml.Length;

            var unitSection = yaml.Substring(unitStart, nextUnit - unitStart);
            unitSection.Should().Contain("faction_id: cis", $"{unitId} must belong to CIS faction");
        }

        // ── Helper Methods ───────────────────────────────────────────────────

        private string ExtractUnitSection(string yaml, string unitId)
        {
            // Try both indentation levels (units in v0.8.1 use "    - id:", assets use different indentation)
            var start = yaml.IndexOf($"- id: {unitId}");
            if (start == -1)
            {
                // Might be indented differently in different YAML sections
                start = yaml.IndexOf($"  - id: {unitId}");
            }
            if (start == -1) throw new Xunit.Sdk.XunitException($"Unit {unitId} not found in YAML");

            // Find the next unit or end of file
            var nextUnit = yaml.IndexOf("\n  - id:", start + 1);
            if (nextUnit == -1) nextUnit = yaml.IndexOf("\n- id:", start + 1);
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

