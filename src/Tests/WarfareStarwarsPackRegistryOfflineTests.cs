#nullable enable
using System;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Offline characterization for warfare-starwars pack content (pack.yaml validation + SDK registry).
/// Does not launch the game or use the bridge.
/// </summary>
[Trait("Category", "AssetSwap")]
[Trait("Category", "WarfareStarwars")]
public sealed class WarfareStarwarsPackRegistryOfflineTests
{
    private const string PackId = "warfare-starwars";
    private const string CloneTrooperUnitId = "rep_clone_trooper";
    private const string CloneTrooperVisualAsset = "sw-clone-trooper-republic";

    private static readonly string PackRoot = Path.Combine(GetRepoRoot(), "packs", PackId);

    /// <summary>
    /// GL-004 prerequisite: clone trooper unit id is declared in pack YAML, pack.yaml validates,
    /// and <see cref="ContentLoader"/> registers the unit without a live game.
    /// </summary>
    [Fact]
    public void WarfareStarwarsPack_RegistersRepCloneTrooperUnitId_WithoutGame()
    {
        string packYaml = Path.Combine(PackRoot, "pack.yaml");
        string unitsYaml = Path.Combine(PackRoot, "units", "republic_units.yaml");

        File.Exists(packYaml).Should().BeTrue("warfare-starwars pack.yaml must exist");
        File.Exists(unitsYaml).Should().BeTrue("republic unit definitions must exist");

        var packLoader = new PackLoader();
        PackManifest manifest = packLoader.LoadFromFile(packYaml);
        manifest.Id.Should().Be(PackId, "pack validation must deserialize warfare-starwars id");

        File.ReadAllText(unitsYaml).Should().Contain(
            $"id: {CloneTrooperUnitId}",
            "pack source must declare the clone trooper unit id used by Phase2 entity queries");

        var registries = new RegistryManager();
        var loader = new ContentLoader(registries);

        loader.LoadPack(PackRoot);
        loader.LastLoadErrorCount.Should().Be(0,
            "warfare-starwars should load without errors: {0}",
            string.Join("; ", loader.LastLoadErrors));

        registries.Units.Contains(CloneTrooperUnitId).Should().BeTrue(
            "ContentLoader must register rep_clone_trooper in the unit registry");

        var trooper = registries.Units.Get(CloneTrooperUnitId);
        trooper.Should().NotBeNull();
        trooper!.VisualAsset.Should().Be(CloneTrooperVisualAsset,
            "visual_asset must match bundle key for asset swap registration");
    }

    private static string GetRepoRoot()
    {
        string? current = Path.GetDirectoryName(typeof(WarfareStarwarsPackRegistryOfflineTests).Assembly.Location);
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "global.json")))
            {
                return current;
            }

            current = Path.GetDirectoryName(current);
        }

        current = Directory.GetCurrentDirectory();
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "global.json")))
            {
                return current;
            }

            current = Path.GetDirectoryName(current);
        }

        throw new DirectoryNotFoundException("Could not locate repo root (global.json)");
    }
}
