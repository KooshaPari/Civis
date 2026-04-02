#nullable enable
using System;
using System.IO;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-003: Phase 1 bundle patch written to disk before entity load (by frame 5).
/// GL-004: Phase 2 RenderMesh swap applied to clone trooper entities.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchAssetSwapTests(GameLaunchFixture fixture)
{
    private const string PatchedBundleDir = "BepInEx/dinoforge_patched_bundles";

    /// <summary>
    /// Verifies that AssetSwapSystem phase 1 writes patched bundles to disk
    /// shortly after pack load (does not require RenderMesh entities to exist).
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task Phase1_PatchedBundleExistsOnDisk_BeforeEntityLoad()
    {
        string? gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        gamePath.Should().NotBeNull("DINO_GAME_PATH must be set");

        string patchDir = Path.Combine(Path.GetDirectoryName(gamePath!)!, PatchedBundleDir);

        // Allow up to 5 seconds for phase 1 to write the patch (it runs on first OnUpdate)
        bool patchExists = await WaitForConditionAsync(
            () => Directory.Exists(patchDir) && Directory.GetFiles(patchDir, "*.bundle").Length > 0,
            timeoutMs: 5_000);

        patchExists.Should().BeTrue(
            $"AssetSwapSystem phase 1 should write patched bundles to {PatchedBundleDir} " +
            $"within the first few frames");
    }

    /// <summary>
    /// Verifies that AssetSwapSystem phase 2 has populated entities in the ECS world
    /// once the warfare-starwars pack is loaded.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task Phase2_CloneTrooper_EntityRegistered()
    {
        QueryResult result =
            await fixture.Client!.QueryEntitiesAsync(category: "rep_clone_trooper");

        result.Entities.Should().NotBeEmpty(
            "clone trooper entities should exist after warfare-starwars is loaded");

        foreach (EntityInfo entity in result.Entities)
        {
            entity.Components.Should().NotBeEmpty(
                "phase 2 should have populated components on the entity");
        }
    }

    private static async Task<bool> WaitForConditionAsync(Func<bool> condition, int timeoutMs)
    {
        System.Diagnostics.Stopwatch sw = System.Diagnostics.Stopwatch.StartNew();
        while (sw.ElapsedMilliseconds < timeoutMs)
        {
            if (condition()) return true;
            await Task.Delay(250).ConfigureAwait(false);
        }
        return false;
    }
}
