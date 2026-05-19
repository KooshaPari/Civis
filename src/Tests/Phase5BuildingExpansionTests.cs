using System;
using System.IO;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Phase 5: Building Expansion Tests
    ///
    /// Validates the 12 remaining buildings (6 Republic + 6 CIS) that complete
    /// 100% building coverage for the warfare-starwars pack.
    /// </summary>
    public class Phase5BuildingExpansionTests
    {
        private static readonly string RepoRoot = GetRepoRoot();
        private string AssetPipelineYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/asset_pipeline.yaml");
        private string RepublicBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/buildings/republic_buildings.yaml");
        private string CISBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/buildings/cis_buildings.yaml");
        private string PrefabsDir => Path.Combine(RepoRoot, "packs/warfare-starwars/assets/prefabs");

        private static string GetRepoRoot()
        {
            var dir = AppContext.BaseDirectory;
            while (dir != null && !Directory.Exists(Path.Combine(dir, "packs")))
                dir = Path.GetDirectoryName(dir);
            return dir ?? throw new InvalidOperationException("Could not find repo root");
        }

        // ── Pipeline configuration ─────────────────────────────────────────

        [Fact]
        public void Phase5_Configuration_ContainsExpansionSection()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain("v1_1_0_buildings_expansion",
                "Phase 5 expansion section must exist in asset_pipeline.yaml");
        }

        [Fact]
        public void Phase5_Configuration_Has12ExpansionBuildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_1_0_buildings_expansion:");
            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            var section = yaml.Substring(start, end - start);

            var count = section.Split(new[] { "- id: " }, StringSplitOptions.None).Length - 1;
            count.Should().Be(12, "Phase 5 should configure exactly 12 expansion buildings");
        }

        [Fact]
        public void Phase5_Configuration_Has6RepublicExpansionBuildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_1_0_buildings_expansion:");
            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            var section = yaml.Substring(start, end - start);

            section.Split(new[] { "faction: republic" }, StringSplitOptions.None).Length.Should().Be(7,
                "Phase 5 should have 6 Republic expansion buildings");
        }

        [Fact]
        public void Phase5_Configuration_Has6CISExpansionBuildings()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_1_0_buildings_expansion:");
            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            var section = yaml.Substring(start, end - start);

            section.Split(new[] { "faction: cis" }, StringSplitOptions.None).Length.Should().Be(7,
                "Phase 5 should have 6 CIS expansion buildings");
        }

        [Fact]
        public void Phase5_ExpansionBuildings_UseConsistentScreenSizes()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_1_0_buildings_expansion:");
            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            var section = yaml.Substring(start, end - start);

            var lines = section.Split('\n');
            var screenSizeLines = System.Array.FindAll(lines, l => l.Contains("screen_sizes:"));
            screenSizeLines.Should().AllSatisfy(line =>
                line.Should().Contain("[200, 100, 50]",
                    "all Phase 5 expansion buildings should use [200, 100, 50] screen sizes"));
        }

        // ── Republic expansion: visual_asset injection ─────────────────────

        [Theory]
        [InlineData("rep_command_center", "sw-rep-command-center")]
        [InlineData("rep_supply_station", "sw-rep-supply-depot")]
        [InlineData("rep_tibanna_refinery", "sw-tibanna-refinery")]
        [InlineData("rep_research_lab", "sw-rep-research-lab")]
        [InlineData("rep_blast_wall", "sw-blast-wall")]
        [InlineData("rep_skyshield_generator", "sw-skyshield-generator")]
        public void Phase5_RepublicExpansion_HasVisualAsset(string buildingId, string expectedKey)
        {
            var yaml = File.ReadAllText(RepublicBuildingsYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"id: {buildingId}",
                $"Republic building '{buildingId}' must be defined");
            yaml.Should().Contain($"visual_asset: {expectedKey}",
                $"Republic building '{buildingId}' must have visual_asset: {expectedKey}");
        }

        // ── CIS expansion: visual_asset injection ──────────────────────────

        [Theory]
        [InlineData("cis_tactical_center", "sw-cis-command-center")]
        [InlineData("cis_mining_facility", "sw-mining-facility")]
        [InlineData("cis_processing_plant", "sw-processing-plant")]
        [InlineData("cis_tech_union_lab", "sw-tech-union-lab")]
        [InlineData("cis_durasteel_barrier", "sw-durasteel-barrier")]
        [InlineData("cis_vulture_nest", "sw-vulture-nest")]
        public void Phase5_CISExpansion_HasVisualAsset(string buildingId, string expectedKey)
        {
            var yaml = File.ReadAllText(CISBuildingsYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain($"id: {buildingId}",
                $"CIS building '{buildingId}' must be defined");
            yaml.Should().Contain($"visual_asset: {expectedKey}",
                $"CIS building '{buildingId}' must have visual_asset: {expectedKey}");
        }

        // ── Prefab existence ───────────────────────────────────────────────

        [Theory]
        [InlineData("Command_Center_Republic.prefab")]
        [InlineData("Supply_Station_Republic.prefab")]
        [InlineData("Tibanna_Refinery_Republic.prefab")]
        [InlineData("Research_Lab_Republic.prefab")]
        [InlineData("Blast_Wall_Republic.prefab")]
        [InlineData("Skyshield_Generator_Republic.prefab")]
        [InlineData("Tactical_Center_CIS.prefab")]
        [InlineData("Mining_Facility_CIS.prefab")]
        [InlineData("Processing_Plant_CIS.prefab")]
        [InlineData("Tech_Union_Lab_CIS.prefab")]
        [InlineData("Durasteel_Barrier_CIS.prefab")]
        [InlineData("Vulture_Nest_CIS.prefab")]
        public void Phase5_ExpansionBuilding_PrefabExists(string prefabName)
        {
            var path = Path.Combine(PrefabsDir, prefabName);
            File.Exists(path).Should().BeTrue(
                $"Prefab '{prefabName}' must exist in assets/prefabs/");
        }

        // ── Total building coverage ────────────────────────────────────────

        [Fact]
        public void Phase5_RepublicBuildings_AllHaveVisualAssets()
        {
            var yaml = File.ReadAllText(RepublicBuildingsYamlPath, System.Text.Encoding.UTF8);
            var lines = yaml.Split('\n');
            string? currentId = null;
            bool hasVisual = false;

            foreach (var line in lines)
            {
                if (line.TrimStart().StartsWith("- id: "))
                {
                    if (currentId != null)
                        hasVisual.Should().BeTrue($"Republic building '{currentId}' must have visual_asset");
                    currentId = line.Trim().Substring("- id: ".Length).Trim();
                    hasVisual = false;
                }
                if (line.Contains("visual_asset:"))
                    hasVisual = true;
            }
            if (currentId != null)
                hasVisual.Should().BeTrue($"Republic building '{currentId}' must have visual_asset");
        }

        [Fact]
        public void Phase5_CISBuildings_AllHaveVisualAssets()
        {
            var yaml = File.ReadAllText(CISBuildingsYamlPath, System.Text.Encoding.UTF8);
            var lines = yaml.Split('\n');
            string? currentId = null;
            bool hasVisual = false;

            foreach (var line in lines)
            {
                if (line.TrimStart().StartsWith("- id: "))
                {
                    if (currentId != null)
                        hasVisual.Should().BeTrue($"CIS building '{currentId}' must have visual_asset");
                    currentId = line.Trim().Substring("- id: ".Length).Trim();
                    hasVisual = false;
                }
                if (line.Contains("visual_asset:"))
                    hasVisual = true;
            }
            if (currentId != null)
                hasVisual.Should().BeTrue($"CIS building '{currentId}' must have visual_asset");
        }
    }
}

