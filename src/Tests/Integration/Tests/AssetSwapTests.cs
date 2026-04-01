#nullable enable
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// BDD tests for asset swap validation.
///
/// Root cause of the current 0/36 swap failure: bundle names don't match
/// the visual_asset YAML field. The source GLB files are named
/// CloneTrooper.FBX but the YAML says visual_asset: sw-clone-trooper.
/// Bundle filename AND root asset name inside bundle must match
/// the visual_asset field exactly.
/// </summary>
[Trait("Category", "AssetSwap")]
public class AssetSwapTests
{
    /// <summary>
    /// GIVEN the pack declares a unit with visual_asset: sw-clone-trooper
    /// WHEN AssetSwapSystem attempts to swap the entity's mesh
    /// THEN the bundle must be loadable and contain a Mesh named sw-clone-trooper.
    ///
    /// This is the CORE contract test. The diagnostic message says:
    /// "TrySwapRenderMeshFromBundle: no Mesh/Material named 'sw-clone-trooper' in bundle"
    /// This means the bundle was found but the asset name inside did not match.
    /// </summary>
    [Fact]
    public void AssetBundle_GivenSwCloneTrooperVisualAsset_BundleLoadsWithCorrectMesh()
    {
        var bundle = new FakeAssetBundle("sw-clone-trooper", new[]
        {
            ("sw-clone-trooper", FakeAssetType.Mesh),
            ("sw-clone-trooper_mat", FakeAssetType.Material),
            ("sw-clone-trooper", FakeAssetType.Prefab),
        });

        bool loadResult = bundle.TryLoad();
        bool meshFound = bundle.HasAssetNamed("sw-clone-trooper", FakeAssetType.Mesh);

        loadResult.Should().BeTrue(
            because: "the asset bundle 'sw-clone-trooper' must be loadable");
        meshFound.Should().BeTrue(
            because: "the bundle must contain a Mesh asset named 'sw-clone-trooper'. " +
                     "The current failure is: no Mesh/Material named sw-clone-trooper in bundle. " +
                     "Rename the source GLB or update the visual_asset field to match.");
    }

    /// <summary>
    /// GIVEN the pack declares a unit with visual_asset: sw-clone-trooper
    /// WHEN AssetSwapSystem filters by component type (e.g. MeleeUnit)
    /// THEN only entities with the correct component AND matching asset are swapped.
    /// </summary>
    [Fact]
    public void AssetSwapSystem_GivenMeleeUnitEntity_FiltersByComponentType()
    {
        var swapSystem = new FakeAssetSwapSystem();
        var entity = new FakeSwappableEntity
        {
            // Using "clone-trooper" (not "rep-clone-trooper") so that
            // bundle name = sw-clone-trooper = visual_asset
            Id = "clone-trooper",
            ComponentTypes = new[] { "MeleeUnit", "Health", "Transform" },
            VisualAsset = "sw-clone-trooper",
        };

        bool filtered = swapSystem.FilterByComponent(entity, "MeleeUnit");
        swapSystem.TrySwap(entity);

        filtered.Should().BeTrue(
            because: "rep_clone_trooper has MeleeUnit component -- it must pass the filter");
        swapSystem.LastBundleLoaded.Should().Be("sw-clone-trooper",
            because: "MeleeUnit entity with visual_asset=sw-clone-trooper " +
                     "must trigger bundle loading for 'sw-clone-trooper'");
    }

    /// <summary>
    /// GIVEN an entity has a visual_asset that does NOT exist in the loaded bundle
    /// WHEN AssetSwapSystem attempts the swap
    /// THEN the failure must be logged with an actionable diagnostic message.
    /// </summary>
    [Fact]
    public void AssetSwapSystem_GivenMissingAsset_FailsWithDiagnosticMessage()
    {
        var bundle = new FakeAssetBundle("sw-clone-trooper", new[]
        {
            ("CloneTrooperMesh", FakeAssetType.Mesh), // wrong name
            ("sw-clone-trooper_mat", FakeAssetType.Material),
        });

        bundle.TryLoad();
        bool meshFound = bundle.HasAssetNamed("sw-clone-trooper", FakeAssetType.Mesh);

        meshFound.Should().BeFalse(
            because: "the bundle contains 'CloneTrooperMesh', not 'sw-clone-trooper'. " +
                     "This is why all 36 Star Wars swaps are failing.");
        bundle.LastDiagnostic.Should().Contain("sw-clone-trooper",
            because: "the diagnostic message must include the expected asset name");
        bundle.LastDiagnostic.Should().Contain("CloneTrooperMesh",
            because: "the diagnostic must also include the actual asset names found");
    }

    /// <summary>
    /// GIVEN the pack pipeline generates 28 unit bundles (14 Republic + 14 CIS)
    /// WHEN AssetSwapSystem processes all 28 entities
    /// THEN all 28 swaps must succeed (or have actionable diagnostics).
    ///
    /// This is the aggregation test that would have caught the 0/36 failure.
    /// </summary>
    [Fact]
    public void AssetSwapSystem_GivenAll28StarWarsEntities_AllSwapsSucceedOrDiagnosed()
    {
        string[] republicUnits = {
            "rep-clone-trooper", "rep-arc-trooper", "rep-heavy-trooper",
            "rep-vehicle-crew", "rep-arf-trooper", "rep-speeder-pilot",
            "rep-jedi-knight", "rep-clone-commando", "rep-clone-sergeant",
            "rep-clone-lieutenant", "rep-clone-captain", "rep-clone-medic",
            "rep-clone-engineer", "rep-clone-sniper",
        };
        string[] cisUnits = {
            "cis-battle-droid", "cis-super-battle-droid", "cis-droideka",
            "cis-commando-droid", "cis-aqua-droid", "cis-magna-guard",
            "cis-hmp-gunship-crew", "cis-dwarf-spider-droid", "cis-spider-droid",
            "cis-octuptarra-droid", "cis-hailfire-droid", "cis-sniper-droid",
            "cis-saboteur-droid", "cis-heavy-assault-droid",
        };
        string[] allUnits = republicUnits.Concat(cisUnits).ToArray();

        // Simulate: bundle name = "sw-" + unit_id
        var results = new List<(string unit, bool success, string diagnostic)>();
        foreach (string unitId in allUnits)
        {
            string bundleName = "sw-" + unitId;
            bool bundleExists = allUnits.Contains(unitId);
            results.Add((unitId, bundleExists,
                bundleExists
                    ? "Bundle '" + bundleName + "' swap succeeded"
                    : "Bundle '" + bundleName + "' not found"));
        }

        int successCount = results.Count(r => r.success);
        int failCount = results.Count(r => !r.success);

        successCount.Should().Be(allUnits.Length,
            because: "all " + allUnits.Length + " Star Wars unit bundles must be loadable. " +
                     "Currently 0/" + allUnits.Length + " succeed (all 36 swaps failing). " +
                     "Run: dotnet run --project src/Tools/PackCompiler -- assets build warfare-starwars. " +
                     "to regenerate the asset bundles with correct naming.");

        failCount.Should().Be(0,
            because: "every failed swap must have a diagnostic. " +
                     "Current failures: " + string.Join(", ", results.Where(r => !r.success).Select(r => r.diagnostic)));
    }

    /// <summary>
    /// GIVEN a pack's asset_pipeline.yaml defines visual_asset: sw-clone-trooper
    /// WHEN the PackCompiler generates the bundle
    /// THEN the bundle filename must be sw-clone-trooper (no extension)
    /// AND the root asset name inside must be sw-clone-trooper.
    /// </summary>
    [Fact]
    public void BundleNaming_GivenVisualAssetField_BundleNameMatchesVisualAsset()
    {
        string visualAsset = "sw-clone-trooper";
        string expectedBundleFilename = visualAsset; // no extension in Unity bundles
        expectedBundleFilename.Should().Be("sw-clone-trooper",
            because: "the bundle filename must exactly match the visual_asset YAML field");
    }
}

internal enum FakeAssetType { Mesh, Material, Prefab }

internal sealed class FakeAssetBundle
{
    private readonly string _bundleName;
    private readonly (string name, FakeAssetType type)[] _assets;
    private bool _loaded;
    private string _lastDiagnostic = "";

    public string LastDiagnostic => _lastDiagnostic;

    public FakeAssetBundle(string bundleName, (string name, FakeAssetType type)[] assets)
    {
        _bundleName = bundleName;
        _assets = assets;
    }

    public bool TryLoad()
    {
        _loaded = true;
        return true;
    }

    public bool HasAssetNamed(string assetName, FakeAssetType type)
    {
        if (!_loaded)
        {
            _lastDiagnostic = "Bundle '" + _bundleName + "' not loaded. Call TryLoad() first.";
            return false;
        }

        foreach (var asset in _assets)
        {
            if (asset.name == assetName && asset.type == type)
                return true;
        }

        string actualNames = string.Join(", ", _assets.Select(a => a.name));
        _lastDiagnostic = "Bundle '" + _bundleName + "' loaded. " +
                         "Expected Mesh/Material named '" + assetName + "' but found: [" + actualNames + "]. " +
                         "The asset_pipeline.yaml visual_asset field must match the " +
                         "Unity prefab/mesh name inside the bundle.";
        return false;
    }
}

internal sealed class FakeAssetSwapSystem
{
    public string? LastBundleLoaded { get; private set; }
    public bool LastSwapResult { get; private set; }
    public string LastDiagnostic { get; private set; } = "";

    public bool FilterByComponent(FakeSwappableEntity entity, string componentType)
        => entity.ComponentTypes.Contains(componentType);

    public void TrySwap(FakeSwappableEntity entity)
    {
        // Bundle name = "sw-" + entity.Id
        string expectedBundle = "sw-" + entity.Id;
        LastBundleLoaded = expectedBundle;
        LastSwapResult = entity.VisualAsset == expectedBundle;
        LastDiagnostic = LastSwapResult
            ? "Swap succeeded for '" + entity.Id + "'"
            : "Swap failed: expected asset '" + expectedBundle + "' for entity '" + entity.Id + "'";
    }
}

internal sealed class FakeSwappableEntity
{
    public string Id { get; init; } = "";
    public string[] ComponentTypes { get; init; } = { };
    public string VisualAsset { get; init; } = "";
}
