using System;
using System.IO;
using System.Text;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Aviation and Anti-Air Tests for the warfare-starwars pack.
    ///
    /// Validates aerial unit configuration (V-19 Torrent, Tri-Fighter),
    /// anti-air building configuration (Skyshield Generator, Vulture Nest),
    /// faction aerial counts, and asset pipeline coverage for Phase 5 units.
    /// </summary>
    public class AviationStarWarsTests
    {
        private static readonly string RepoRoot = GetRepoRoot();

        private string RepublicUnitsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/units/republic_units.yaml");
        private string CISUnitsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/units/cis_units.yaml");
        private string RepublicBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/buildings/republic_buildings.yaml");
        private string CISBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/buildings/cis_buildings.yaml");
        private string AssetPipelineYamlPath => Path.Combine(RepoRoot, "packs/warfare-starwars/asset_pipeline.yaml");

        private static string GetRepoRoot()
        {
            var dir = AppContext.BaseDirectory;
            while (dir != null && !Directory.Exists(Path.Combine(dir, "packs")))
                dir = Path.GetDirectoryName(dir);
            return dir ?? throw new InvalidOperationException("Could not find repo root");
        }

        // ── Group 1: Aerial Unit Configuration ────────────────────────────────

        [Theory]
        [InlineData("rep_v19_torrent", "republic")]
        [InlineData("cis_tri_fighter", "cis")]
        [InlineData("rep_laat_gunship", "republic")]
        [InlineData("cis_nantex_fighter", "cis")]
        public void AerialUnit_ExistsInYaml(string unitId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicUnitsYamlPath : CISUnitsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            yaml.Should().Contain($"id: {unitId}",
                $"Aerial unit '{unitId}' must be defined in {faction} units YAML");
        }

        [Theory]
        [InlineData("rep_v19_torrent", "republic")]
        [InlineData("cis_tri_fighter", "cis")]
        [InlineData("rep_laat_gunship", "republic")]
        [InlineData("cis_nantex_fighter", "cis")]
        public void AerialUnit_HasAerialBehaviorTag(string unitId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicUnitsYamlPath : CISUnitsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            // Extract the block for this unit and confirm it has Aerial in behavior_tags
            int idIndex = yaml.IndexOf($"id: {unitId}");
            idIndex.Should().BeGreaterThan(-1,
                $"Unit '{unitId}' must exist in YAML before checking behavior_tags");

            int nextUnit = yaml.IndexOf("\n- id:", idIndex + 1);
            var unitBlock = nextUnit > -1
                ? yaml.Substring(idIndex, nextUnit - idIndex)
                : yaml.Substring(idIndex);

            unitBlock.Should().Contain("Aerial",
                $"Aerial unit '{unitId}' must have 'Aerial' in its behavior_tags");
        }

        [Theory]
        [InlineData("rep_v19_torrent", "republic")]
        [InlineData("cis_tri_fighter", "cis")]
        [InlineData("rep_laat_gunship", "republic")]
        [InlineData("cis_nantex_fighter", "cis")]
        public void AerialUnit_HasAerialBlockWithCruiseAltitude(string unitId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicUnitsYamlPath : CISUnitsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {unitId}");
            idIndex.Should().BeGreaterThan(-1);

            int nextUnit = yaml.IndexOf("\n- id:", idIndex + 1);
            var unitBlock = nextUnit > -1
                ? yaml.Substring(idIndex, nextUnit - idIndex)
                : yaml.Substring(idIndex);

            unitBlock.Should().Contain("aerial:",
                $"Aerial unit '{unitId}' must have an 'aerial:' configuration block");
            unitBlock.Should().Contain("cruise_altitude:",
                $"Aerial unit '{unitId}' must define 'cruise_altitude:' in its aerial block");

            var altLine = Array.Find(
                unitBlock.Split('\n'),
                l => l.TrimStart().StartsWith("cruise_altitude:"));
            altLine.Should().NotBeNull($"Could not find cruise_altitude line for '{unitId}'");

            var altValueStr = altLine!.Split(':')[1].Trim();
            double.TryParse(altValueStr, System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double altValue).Should().BeTrue(
                $"cruise_altitude value for '{unitId}' must be a valid number, got '{altValueStr}'");

            altValue.Should().BeGreaterThan(0,
                $"Aerial unit '{unitId}' cruise_altitude must be greater than 0");
        }

        // ── Group 2: Aerial Properties Validation ─────────────────────────────

        [Theory]
        [InlineData("rep_v19_torrent", "republic", 20.0)]
        [InlineData("cis_tri_fighter", "cis", 18.0)]
        public void AerialUnit_CruiseAltitudeInValidRange(string unitId, string faction, double expectedAltitude)
        {
            var yamlPath = faction == "republic" ? RepublicUnitsYamlPath : CISUnitsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {unitId}");
            int nextUnit = yaml.IndexOf("\n- id:", idIndex + 1);
            var unitBlock = nextUnit > -1
                ? yaml.Substring(idIndex, nextUnit - idIndex)
                : yaml.Substring(idIndex);

            var altLine = Array.Find(
                unitBlock.Split('\n'),
                l => l.TrimStart().StartsWith("cruise_altitude:"));

            var altValueStr = altLine!.Split(':')[1].Trim();
            double.TryParse(altValueStr, System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double altValue);

            altValue.Should().BeInRange(5.0, 40.0,
                $"Aerial unit '{unitId}' cruise_altitude {altValue} must be between 5 and 40");
            altValue.Should().Be(expectedAltitude,
                $"Aerial unit '{unitId}' cruise_altitude must match expected value {expectedAltitude}");
        }

        [Theory]
        [InlineData("rep_v19_torrent", "republic", 8.0, 5.0)]
        [InlineData("cis_tri_fighter", "cis", 10.0, 6.0)]
        public void AerialUnit_AscendAndDescendSpeedsArePositive(string unitId, string faction,
            double expectedAscend, double expectedDescend)
        {
            var yamlPath = faction == "republic" ? RepublicUnitsYamlPath : CISUnitsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {unitId}");
            int nextUnit = yaml.IndexOf("\n- id:", idIndex + 1);
            var unitBlock = nextUnit > -1
                ? yaml.Substring(idIndex, nextUnit - idIndex)
                : yaml.Substring(idIndex);

            var lines = unitBlock.Split('\n');

            var ascendLine = Array.Find(lines, l => l.TrimStart().StartsWith("ascend_speed:"));
            ascendLine.Should().NotBeNull($"'{unitId}' must define ascend_speed");
            double.TryParse(ascendLine!.Split(':')[1].Trim(), System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double ascendValue);
            ascendValue.Should().BeGreaterThan(0,
                $"'{unitId}' ascend_speed must be greater than 0");
            ascendValue.Should().Be(expectedAscend,
                $"'{unitId}' ascend_speed must match expected value {expectedAscend}");

            var descendLine = Array.Find(lines, l => l.TrimStart().StartsWith("descend_speed:"));
            descendLine.Should().NotBeNull($"'{unitId}' must define descend_speed");
            double.TryParse(descendLine!.Split(':')[1].Trim(), System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double descendValue);
            descendValue.Should().BeGreaterThan(0,
                $"'{unitId}' descend_speed must be greater than 0");
            descendValue.Should().Be(expectedDescend,
                $"'{unitId}' descend_speed must match expected value {expectedDescend}");
        }

        // ── Group 3: Anti-Air Building Configuration ──────────────────────────

        [Theory]
        [InlineData("rep_skyshield_generator", "republic")]
        [InlineData("cis_vulture_nest", "cis")]
        public void AntiAirBuilding_ExistsInYaml(string buildingId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicBuildingsYamlPath : CISBuildingsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            yaml.Should().Contain($"id: {buildingId}",
                $"Anti-air building '{buildingId}' must be defined in {faction} buildings YAML");
        }

        [Theory]
        [InlineData("rep_skyshield_generator", "republic")]
        [InlineData("cis_vulture_nest", "cis")]
        public void AntiAirBuilding_HasAntiAirDefenseTag(string buildingId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicBuildingsYamlPath : CISBuildingsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {buildingId}");
            idIndex.Should().BeGreaterThan(-1,
                $"Building '{buildingId}' must exist before checking defense_tags");

            int nextBuilding = yaml.IndexOf("\n- id:", idIndex + 1);
            var buildingBlock = nextBuilding > -1
                ? yaml.Substring(idIndex, nextBuilding - idIndex)
                : yaml.Substring(idIndex);

            buildingBlock.Should().Contain("AntiAir",
                $"Building '{buildingId}' must have 'AntiAir' in its defense_tags");
        }

        [Theory]
        [InlineData("rep_skyshield_generator", "republic")]
        [InlineData("cis_vulture_nest", "cis")]
        public void AntiAirBuilding_HasAntiAirBlockWithRange(string buildingId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicBuildingsYamlPath : CISBuildingsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {buildingId}");
            int nextBuilding = yaml.IndexOf("\n- id:", idIndex + 1);
            var buildingBlock = nextBuilding > -1
                ? yaml.Substring(idIndex, nextBuilding - idIndex)
                : yaml.Substring(idIndex);

            buildingBlock.Should().Contain("anti_air:",
                $"Building '{buildingId}' must have an 'anti_air:' configuration block");
            buildingBlock.Should().Contain("range:",
                $"Building '{buildingId}' must define 'range:' in its anti_air block");

            var rangeLine = Array.Find(
                buildingBlock.Split('\n'),
                l => l.TrimStart().StartsWith("range:"));
            rangeLine.Should().NotBeNull($"Could not find range line for '{buildingId}'");

            double.TryParse(rangeLine!.Split(':')[1].Trim(), System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double rangeValue).Should().BeTrue(
                $"range value for '{buildingId}' must be a valid number");
            rangeValue.Should().BeGreaterThan(0,
                $"Building '{buildingId}' anti_air range must be greater than 0");
        }

        // ── Group 4: Anti-Air Building Parameters ─────────────────────────────

        [Theory]
        [InlineData("rep_skyshield_generator", "republic", 40.0, 1.4)]
        [InlineData("cis_vulture_nest", "cis", 38.0, 1.5)]
        public void AntiAirBuilding_RangeAndDamageBonusInValidRanges(string buildingId, string faction,
            double expectedRange, double expectedDamageBonus)
        {
            var yamlPath = faction == "republic" ? RepublicBuildingsYamlPath : CISBuildingsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {buildingId}");
            int nextBuilding = yaml.IndexOf("\n- id:", idIndex + 1);
            var buildingBlock = nextBuilding > -1
                ? yaml.Substring(idIndex, nextBuilding - idIndex)
                : yaml.Substring(idIndex);

            var lines = buildingBlock.Split('\n');

            var rangeLine = Array.Find(lines, l => l.TrimStart().StartsWith("range:"));
            rangeLine.Should().NotBeNull($"'{buildingId}' must define range");
            double.TryParse(rangeLine!.Split(':')[1].Trim(), System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double rangeValue);
            rangeValue.Should().BeInRange(10.0, 100.0,
                $"'{buildingId}' anti_air range {rangeValue} must be between 10 and 100");
            rangeValue.Should().Be(expectedRange,
                $"'{buildingId}' range must match expected value {expectedRange}");

            var damageLine = Array.Find(lines, l => l.TrimStart().StartsWith("damage_bonus:"));
            damageLine.Should().NotBeNull($"'{buildingId}' must define damage_bonus");
            double.TryParse(damageLine!.Split(':')[1].Trim(), System.Globalization.NumberStyles.Any,
                System.Globalization.CultureInfo.InvariantCulture, out double damageBonusValue);
            damageBonusValue.Should().BeInRange(1.0, 3.0,
                $"'{buildingId}' damage_bonus {damageBonusValue} must be between 1.0 and 3.0");
            damageBonusValue.Should().Be(expectedDamageBonus,
                $"'{buildingId}' damage_bonus must match expected value {expectedDamageBonus}");
        }

        // ── Group 5: Faction Aerial Counts ────────────────────────────────────

        [Fact]
        public void Republic_HasAtLeastOneAerialUnit()
        {
            var yaml = File.ReadAllText(RepublicUnitsYamlPath, System.Text.Encoding.UTF8);
            var lines = yaml.Split('\n');

            int aerialUnitCount = 0;
            bool inBehaviorTags = false;

            foreach (var line in lines)
            {
                if (line.TrimStart().StartsWith("- id:"))
                    inBehaviorTags = false;

                if (line.TrimStart().StartsWith("behavior_tags:"))
                    inBehaviorTags = true;

                if (inBehaviorTags && line.Contains("Aerial"))
                {
                    aerialUnitCount++;
                    inBehaviorTags = false;
                }
            }

            aerialUnitCount.Should().BeGreaterThanOrEqualTo(1,
                "Republic faction must have at least 1 aerial unit with 'Aerial' in behavior_tags");
        }

        [Fact]
        public void CIS_HasAtLeastOneAerialUnit()
        {
            var yaml = File.ReadAllText(CISUnitsYamlPath, System.Text.Encoding.UTF8);
            var lines = yaml.Split('\n');

            int aerialUnitCount = 0;
            bool inBehaviorTags = false;

            foreach (var line in lines)
            {
                if (line.TrimStart().StartsWith("- id:"))
                    inBehaviorTags = false;

                if (line.TrimStart().StartsWith("behavior_tags:"))
                    inBehaviorTags = true;

                if (inBehaviorTags && line.Contains("Aerial"))
                {
                    aerialUnitCount++;
                    inBehaviorTags = false;
                }
            }

            aerialUnitCount.Should().BeGreaterThanOrEqualTo(1,
                "CIS faction must have at least 1 aerial unit with 'Aerial' in behavior_tags");
        }

        // ── Group 6: Asset Pipeline Aerial Section ────────────────────────────

        [Fact]
        public void AssetPipeline_ContainsPhase5UnitsSection()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            yaml.Should().Contain("v1_2_0_units_phase5:",
                "asset_pipeline.yaml must contain the 'v1_2_0_units_phase5:' section");
        }

        [Fact]
        public void AssetPipeline_Phase5UnitsSection_HasAtLeast8Models()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_2_0_units_phase5:");
            start.Should().BeGreaterThan(-1,
                "'v1_2_0_units_phase5:' section must exist in asset_pipeline.yaml");

            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            end.Should().BeGreaterThan(start,
                "'v0_9_0_future:' section must follow 'v1_2_0_units_phase5:' in asset_pipeline.yaml");

            var section = yaml.Substring(start, end - start);
            int modelCount = section.Split(new[] { "- id: " }, StringSplitOptions.None).Length - 1;

            modelCount.Should().BeGreaterThanOrEqualTo(8,
                $"v1_2_0_units_phase5 section must configure at least 8 unit models, found {modelCount}");
        }

        [Fact]
        public void AssetPipeline_Phase5UnitsSection_IncludesAerialUnits()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_2_0_units_phase5:");
            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            var section = yaml.Substring(start, end - start);

            section.Should().Contain("v19_torrent",
                "v1_2_0_units_phase5 must include the Republic V-19 Torrent aerial unit model");
            section.Should().Contain("cis_tri_fighter",
                "v1_2_0_units_phase5 must include the CIS Tri-Fighter aerial unit model");
        }

        [Fact]
        public void AssetPipeline_Phase5UnitsSection_AerialUnitsHaveAerialType()
        {
            var yaml = File.ReadAllText(AssetPipelineYamlPath, System.Text.Encoding.UTF8);
            int start = yaml.IndexOf("v1_2_0_units_phase5:");
            int end = yaml.IndexOf("v0_9_0_future:", start + 1);
            var section = yaml.Substring(start, end - start);

            // Find the tri_fighter block and verify it uses type: aerial
            int triStart = section.IndexOf("cis_tri_fighter");
            triStart.Should().BeGreaterThan(-1, "cis_tri_fighter entry must exist in phase5 section");

            int triEnd = section.IndexOf("- id:", triStart + 1);
            var triBlock = triEnd > -1
                ? section.Substring(triStart, triEnd - triStart)
                : section.Substring(triStart);

            triBlock.Should().Contain("type: aerial",
                "The cis_tri_fighter model entry must have 'type: aerial' in the asset pipeline");
        }

        // ── Group 7: Aircraft Production Buildings (Airport / Hangar) ──────────

        [Theory]
        [InlineData("rep_gunship_bay", "republic")]
        [InlineData("cis_droid_hangar", "cis")]
        public void AircraftBuilding_ExistsInYaml(string buildingId, string faction)
        {
            var yamlPath = faction == "republic" ? RepublicBuildingsYamlPath : CISBuildingsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            yaml.Should().Contain($"id: {buildingId}",
                $"Aircraft production building '{buildingId}' must be defined in {faction} buildings YAML");
        }

        [Theory]
        [InlineData("rep_gunship_bay", "republic", "rep_laat_gunship")]
        [InlineData("rep_gunship_bay", "republic", "rep_v19_torrent")]
        [InlineData("cis_droid_hangar", "cis", "cis_tri_fighter")]
        [InlineData("cis_droid_hangar", "cis", "cis_nantex_fighter")]
        public void AircraftBuilding_ProducesAerialUnit(string buildingId, string faction, string producedUnitId)
        {
            var yamlPath = faction == "republic" ? RepublicBuildingsYamlPath : CISBuildingsYamlPath;
            var yaml = File.ReadAllText(yamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {buildingId}");
            idIndex.Should().BeGreaterThan(-1,
                $"Building '{buildingId}' must exist before checking production");

            int nextBuilding = yaml.IndexOf("\n- id:", idIndex + 1);
            var buildingBlock = nextBuilding > -1
                ? yaml.Substring(idIndex, nextBuilding - idIndex)
                : yaml.Substring(idIndex);

            buildingBlock.Should().Contain($"{producedUnitId}:",
                $"Aircraft building '{buildingId}' must list '{producedUnitId}' in its production block " +
                "so the airport produces the aerial unit");
        }

        // ── Group 8: Generic warfare-aerial Airport Buildings + Airplanes ─────

        private string AerialBuildingsYamlPath => Path.Combine(RepoRoot, "packs/warfare-aerial/buildings/airfield_buildings.yaml");
        private string AerialUnitsYamlPath => Path.Combine(RepoRoot, "packs/warfare-aerial/units/aerial_units.yaml");

        [Theory]
        [InlineData("airport")]
        [InlineData("hangar")]
        [InlineData("landing_pad")]
        public void GenericAirport_ExistsInYaml(string buildingId)
        {
            var yaml = File.ReadAllText(AerialBuildingsYamlPath, Encoding.UTF8);
            yaml.Should().Contain($"id: {buildingId}",
                $"Generic airport building '{buildingId}' must be defined in warfare-aerial buildings YAML");
        }

        [Theory]
        [InlineData("airport", "aerial_fighter_plane")]
        [InlineData("airport", "aerial_interceptor_plane")]
        [InlineData("hangar", "aerial_bomber_plane")]
        [InlineData("landing_pad", "aerial_interceptor_plane")]
        public void GenericAirport_ProducesAirplane(string buildingId, string producedUnitId)
        {
            var yaml = File.ReadAllText(AerialBuildingsYamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {buildingId}");
            idIndex.Should().BeGreaterThan(-1);

            int nextBuilding = yaml.IndexOf("\n- id:", idIndex + 1);
            var block = nextBuilding > -1
                ? yaml.Substring(idIndex, nextBuilding - idIndex)
                : yaml.Substring(idIndex);

            block.Should().Contain($"{producedUnitId}:",
                $"Airport building '{buildingId}' must produce '{producedUnitId}'");
        }

        [Theory]
        [InlineData("aerial_fighter_plane")]
        [InlineData("aerial_bomber_plane")]
        [InlineData("aerial_interceptor_plane")]
        public void GenericAirplane_HasAerialBlockAndTag(string unitId)
        {
            var yaml = File.ReadAllText(AerialUnitsYamlPath, Encoding.UTF8);

            int idIndex = yaml.IndexOf($"id: {unitId}");
            idIndex.Should().BeGreaterThan(-1,
                $"Airplane unit '{unitId}' must be defined in warfare-aerial units YAML");

            int nextUnit = yaml.IndexOf("\n- id:", idIndex + 1);
            var block = nextUnit > -1
                ? yaml.Substring(idIndex, nextUnit - idIndex)
                : yaml.Substring(idIndex);

            block.Should().Contain("Aerial",
                $"Airplane '{unitId}' must carry the 'Aerial' behavior_tag so AerialUnitMapper attaches AerialUnitComponent");
            block.Should().Contain("cruise_altitude:",
                $"Airplane '{unitId}' must define a cruise_altitude so the Aviation systems fly it");
        }
    }
}
