using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tests
{
    /// <summary>
    /// Phase 7: Visual Asset → ECS → Addressables Runtime Integration Tests
    ///
    /// Validates the full chain:
    ///   YAML definition files → ContentLoader/Registry → VisualAsset field populated
    ///   → Addressables key resolves in addressables.yaml catalog
    ///
    /// This ensures that every unit and building with a visual_asset field in the
    /// warfare-starwars pack will be loadable at runtime via Unity Addressables.
    /// </summary>
    public class Phase7VisualAssetIntegrationTests
    {
        private static readonly string RepoRoot = GetRepoRoot();
        private static readonly string PackRoot = Path.Combine(RepoRoot, "packs/warfare-starwars");
        private static readonly string AddressablesPath = Path.Combine(PackRoot, "addressables.yaml");

        // Uses YamlLoader.Deserializer for centralized YAML parsing

        // ── Helpers ────────────────────────────────────────────────────────────

        private static string GetRepoRoot()
        {
            string dir = AppContext.BaseDirectory;
            while (dir != null && !Directory.Exists(Path.Combine(dir, "packs")))
                dir = Path.GetDirectoryName(dir)!;
            return dir ?? throw new InvalidOperationException("Could not find repo root (no 'packs' directory found in ancestors)");
        }

        private HashSet<string> LoadCatalogKeys()
        {
            string content = File.ReadAllText(AddressablesPath);
            var keys = new HashSet<string>(StringComparer.Ordinal);
            foreach (Match m in Regex.Matches(content, @"^  - key: (\S+)", RegexOptions.Multiline))
                keys.Add(m.Groups[1].Value);
            return keys;
        }

        private List<UnitDefinition> LoadUnits(string yamlPath)
        {
            string content = File.ReadAllText(yamlPath);
            return YamlLoader.Deserializer.Deserialize<List<UnitDefinition>>(content) ?? new List<UnitDefinition>();
        }

        private List<BuildingDefinition> LoadBuildings(string yamlPath)
        {
            string content = File.ReadAllText(yamlPath);
            return YamlLoader.Deserializer.Deserialize<List<BuildingDefinition>>(content) ?? new List<BuildingDefinition>();
        }

        // ── Catalog presence ────────────────────────────────────────────────────

        [Fact]
        public void Addressables_CatalogExists()
        {
            File.Exists(AddressablesPath).Should().BeTrue("addressables.yaml must exist for runtime loading");
        }

        [Fact]
        public void Addressables_CatalogHasAtLeast50Entries()
        {
            var keys = LoadCatalogKeys();
            keys.Should().HaveCountGreaterThanOrEqualTo(50, "catalog should cover all units and buildings");
        }

        [Fact]
        public void Addressables_CatalogVersionIs11()
        {
            string content = File.ReadAllText(AddressablesPath);
            content.Should().Contain("version: \"1.1\"", "catalog should be at version 1.1");
        }

        // ── UnitDefinition deserialization ─────────────────────────────────────

        [Fact]
        public void Units_RepublicUnits_DeserializeVisualAsset()
        {
            var units = LoadUnits(Path.Combine(PackRoot, "units/republic_units.yaml"));
            units.Should().NotBeEmpty();
            units.Should().AllSatisfy(u =>
                u.VisualAsset.Should().NotBeNullOrEmpty(
                    $"Republic unit '{u.Id}' must have a visual_asset field"));
        }

        [Fact]
        public void Units_CisUnits_DeserializeVisualAsset()
        {
            var units = LoadUnits(Path.Combine(PackRoot, "units/cis_units.yaml"));
            units.Should().NotBeEmpty();
            units.Should().AllSatisfy(u =>
                u.VisualAsset.Should().NotBeNullOrEmpty(
                    $"CIS unit '{u.Id}' must have a visual_asset field"));
        }

        // ── BuildingDefinition deserialization ─────────────────────────────────

        [Fact]
        public void Buildings_RepublicBuildings_DeserializeVisualAsset()
        {
            var buildings = LoadBuildings(Path.Combine(PackRoot, "buildings/republic_buildings.yaml"));
            buildings.Should().NotBeEmpty();
            buildings.Should().AllSatisfy(b =>
                b.VisualAsset.Should().NotBeNullOrEmpty(
                    $"Republic building '{b.Id}' must have a visual_asset field"));
        }

        [Fact]
        public void Buildings_CisBuildings_DeserializeVisualAsset()
        {
            var buildings = LoadBuildings(Path.Combine(PackRoot, "buildings/cis_buildings.yaml"));
            buildings.Should().NotBeEmpty();
            buildings.Should().AllSatisfy(b =>
                b.VisualAsset.Should().NotBeNullOrEmpty(
                    $"CIS building '{b.Id}' must have a visual_asset field"));
        }

        // ── Addressables key resolution (full chain) ───────────────────────────

        [Fact]
        public void Chain_RepublicUnits_AllVisualAssetsResolveInCatalog()
        {
            var keys = LoadCatalogKeys();
            var units = LoadUnits(Path.Combine(PackRoot, "units/republic_units.yaml"));

            var missing = units
                .Where(u => !string.IsNullOrEmpty(u.VisualAsset) && !keys.Contains(u.VisualAsset!))
                .Select(u => $"{u.Id} => {u.VisualAsset}")
                .ToList();

            missing.Should().BeEmpty(
                $"all Republic unit visual_asset keys must exist in addressables.yaml, missing: {string.Join(", ", missing)}");
        }

        [Fact]
        public void Chain_CisUnits_AllVisualAssetsResolveInCatalog()
        {
            var keys = LoadCatalogKeys();
            var units = LoadUnits(Path.Combine(PackRoot, "units/cis_units.yaml"));

            var missing = units
                .Where(u => !string.IsNullOrEmpty(u.VisualAsset) && !keys.Contains(u.VisualAsset!))
                .Select(u => $"{u.Id} => {u.VisualAsset}")
                .ToList();

            missing.Should().BeEmpty(
                $"all CIS unit visual_asset keys must exist in addressables.yaml, missing: {string.Join(", ", missing)}");
        }

        [Fact]
        public void Chain_RepublicBuildings_AllVisualAssetsResolveInCatalog()
        {
            var keys = LoadCatalogKeys();
            var buildings = LoadBuildings(Path.Combine(PackRoot, "buildings/republic_buildings.yaml"));

            var missing = buildings
                .Where(b => !string.IsNullOrEmpty(b.VisualAsset) && !keys.Contains(b.VisualAsset!))
                .Select(b => $"{b.Id} => {b.VisualAsset}")
                .ToList();

            missing.Should().BeEmpty(
                $"all Republic building visual_asset keys must exist in addressables.yaml, missing: {string.Join(", ", missing)}");
        }

        [Fact]
        public void Chain_CisBuildings_AllVisualAssetsResolveInCatalog()
        {
            var keys = LoadCatalogKeys();
            var buildings = LoadBuildings(Path.Combine(PackRoot, "buildings/cis_buildings.yaml"));

            var missing = buildings
                .Where(b => !string.IsNullOrEmpty(b.VisualAsset) && !keys.Contains(b.VisualAsset!))
                .Select(b => $"{b.Id} => {b.VisualAsset}")
                .ToList();

            missing.Should().BeEmpty(
                $"all CIS building visual_asset keys must exist in addressables.yaml, missing: {string.Join(", ", missing)}");
        }

        [Fact]
        public void Chain_AllDefinitions_TotalVisualAssetCoverage()
        {
            var keys = LoadCatalogKeys();

            var allUnits = LoadUnits(Path.Combine(PackRoot, "units/republic_units.yaml"))
                .Concat(LoadUnits(Path.Combine(PackRoot, "units/cis_units.yaml")))
                .ToList();
            var allBuildings = LoadBuildings(Path.Combine(PackRoot, "buildings/republic_buildings.yaml"))
                .Concat(LoadBuildings(Path.Combine(PackRoot, "buildings/cis_buildings.yaml")))
                .ToList();

            int unitsWithAsset = allUnits.Count(u => !string.IsNullOrEmpty(u.VisualAsset));
            int buildingsWithAsset = allBuildings.Count(b => !string.IsNullOrEmpty(b.VisualAsset));
            int totalWithAsset = unitsWithAsset + buildingsWithAsset;

            unitsWithAsset.Should().Be(allUnits.Count,
                "every unit must have a visual_asset");
            buildingsWithAsset.Should().Be(allBuildings.Count,
                "every building must have a visual_asset");
            totalWithAsset.Should().BeGreaterThanOrEqualTo(50,
                "at least 50 definitions (28 units + 22 buildings) must have visual_asset keys");
        }

        // ── ContentLoader registry integration ─────────────────────────────────

        [Fact]
        public void Registry_StarWarsPack_LoadsAndUnitsHaveVisualAsset()
        {
            var registries = new RegistryManager();
            var loader = new ContentLoader(registries);

            loader.LoadPack(PackRoot);
            loader.LastLoadErrorCount.Should().Be(0,
                "warfare-starwars pack should load without errors. Errors: {0}",
                string.Join("; ", loader.LastLoadErrors));

            // Verify key units are registered
            registries.Units.Contains("rep_clone_trooper").Should().BeTrue(
                "rep_clone_trooper should be in unit registry after loading warfare-starwars pack");
            registries.Units.Contains("cis_b1_battle_droid").Should().BeTrue(
                "cis_b1_battle_droid should be in unit registry after loading warfare-starwars pack");

            // Verify visual_asset is populated on deserialized unit data
            var trooper = registries.Units.Get("rep_clone_trooper");
            trooper.Should().NotBeNull();
            trooper!.VisualAsset.Should().NotBeNullOrEmpty(
                "rep_clone_trooper must have visual_asset after ContentLoader deserialization");
        }

        [Fact]
        public void Registry_StarWarsPack_BuildingsHaveVisualAsset()
        {
            var registries = new RegistryManager();
            var loader = new ContentLoader(registries);

            loader.LoadPack(PackRoot);

            registries.Buildings.Contains("rep_clone_facility").Should().BeTrue(
                "rep_clone_facility should be in building registry after loading warfare-starwars pack");
            registries.Buildings.Contains("cis_droid_factory").Should().BeTrue(
                "cis_droid_factory should be in building registry after loading warfare-starwars pack");

            var barracks = registries.Buildings.Get("rep_clone_facility");
            barracks.Should().NotBeNull();
            barracks!.VisualAsset.Should().NotBeNullOrEmpty(
                "rep_clone_facility must have visual_asset after ContentLoader deserialization");
        }
    }
}
