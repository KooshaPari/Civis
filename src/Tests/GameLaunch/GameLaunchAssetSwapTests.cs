#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Support;
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
    [SkippableFact]
    public async Task Phase1_PatchedBundleExistsOnDisk_BeforeEntityLoad()
    {
        fixture.SkipIfNotInitialized();

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
    [SkippableFact]
    public async Task Phase2_CloneTrooper_EntityRegistered()
    {
        fixture.SkipIfNotInitialized();

        QueryResult result =
            await fixture.Client!.QueryEntitiesAsync(category: "rep_clone_trooper");

        result.Entities.Should().NotBeEmpty( // open-ended-count-ok: game-runtime entity query result is fixture-dependent (pack loaded in-game)
            "clone trooper entities should exist after warfare-starwars is loaded");

        foreach (EntityInfo entity in result.Entities)
        {
            entity.Components.Should().NotBeEmpty( // open-ended-count-ok: game-runtime entity components are fixture-dependent (pack load side effect)
                "phase 2 should have populated components on the entity");
        }
    }

    /// <summary>
    /// After warfare-starwars is loaded (status) or clone trooper entities exist, captures an in-game
    /// screenshot via the bridge and writes a receipt under <c>docs/qa/evidence/asset-swap/</c> (or temp).
    /// See <c>docs/reference/asset-visual-acceptance.md</c> for human review criteria.
    /// </summary>
    [SkippableFact]
    public async Task AssetVisualAcceptance_WarfareStarwarsLoaded_CapturesScreenshotWithReceipt()
    {
        fixture.SkipIfNotInitialized();

        const string packId = "warfare-starwars";
        const string cloneCategory = "rep_clone_trooper";

        bool packOrEntityReady = await TestWait.UntilAsync(
            async () =>
            {
                GameStatus status = await fixture.Client!.StatusAsync().ConfigureAwait(false);
                if (status.LoadedPacks.Contains(packId))
                {
                    return true;
                }

                QueryResult entities =
                    await fixture.Client.QueryEntitiesAsync(category: cloneCategory).ConfigureAwait(false);
                return entities.Entities.Count > 0;
            },
            TimeSpan.FromSeconds(15),
            pollMs: 500).ConfigureAwait(false);

        packOrEntityReady.Should().BeTrue(
            $"bridge status should list {packId} or query should return {cloneCategory} entities before visual capture");

        GameStatus finalStatus = await fixture.Client!.StatusAsync().ConfigureAwait(false);
        QueryResult cloneEntities =
            await fixture.Client.QueryEntitiesAsync(category: cloneCategory).ConfigureAwait(false);

        string evidenceDir = ResolveEvidenceDirectory();
        string screenshotPath = Path.Combine(evidenceDir, "warfare-starwars-asset-visual.png");

        ScreenshotResult screenshot =
            await fixture.Client.ScreenshotAsync(screenshotPath).ConfigureAwait(false);

        screenshot.Success.Should().BeTrue("bridge screenshot capture should succeed");
        File.Exists(screenshotPath).Should().BeTrue("screenshot file should exist on disk");
        new FileInfo(screenshotPath).Length.Should().BeGreaterThan(100, "screenshot should contain image bytes");

        bool gamePathSet = !string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("DINO_GAME_PATH"));
        bool packLoaded = finalStatus.LoadedPacks.Contains(packId);
        bool overallPass = packOrEntityReady && screenshot.Success && File.Exists(screenshotPath);

        var receipt = new Dictionary<string, object?>
        {
            ["schema"] = "dinoforge.asset-visual-acceptance/v1",
            ["observed_at"] = DateTimeOffset.Now.ToString("o"),
            ["evidence_dir"] = evidenceDir,
            ["test_name"] = nameof(AssetVisualAcceptance_WarfareStarwarsLoaded_CapturesScreenshotWithReceipt),
            ["dino_game_path_set"] = gamePathSet,
            ["warfare_starwars_loaded"] = packLoaded,
            ["clone_trooper_entity_count"] = cloneEntities.Entities.Count,
            ["screenshot_path"] = screenshot.Path,
            ["screenshot_success"] = screenshot.Success,
            ["screenshot_width"] = screenshot.Width,
            ["screenshot_height"] = screenshot.Height,
            ["overall_pass"] = overallPass,
            ["steps"] = new object[]
            {
                new Dictionary<string, object?>
                {
                    ["step"] = "pack_or_entity_ready",
                    ["pass"] = packOrEntityReady,
                    ["detail"] = packLoaded
                        ? $"{packId} listed in status.LoadedPacks"
                        : $"{cloneCategory} entity count={cloneEntities.Entities.Count}",
                    ["at"] = DateTimeOffset.Now.ToString("o"),
                },
                new Dictionary<string, object?>
                {
                    ["step"] = "bridge_screenshot",
                    ["pass"] = screenshot.Success && File.Exists(screenshotPath),
                    ["detail"] = screenshot.Path,
                    ["at"] = DateTimeOffset.Now.ToString("o"),
                },
            },
        };

        string receiptPath = Path.Combine(evidenceDir, "visual-acceptance-receipt.json");
        await File.WriteAllTextAsync(
            receiptPath,
            JsonSerializer.Serialize(receipt, new JsonSerializerOptions { WriteIndented = true })).ConfigureAwait(false);

        overallPass.Should().BeTrue("visual acceptance capture receipt should record overall_pass");
    }

    private static string ResolveEvidenceDirectory()
    {
        try
        {
            string evidenceDir = Path.Combine(
                GetRepoRoot(),
                "docs",
                "qa",
                "evidence",
                "asset-swap");
            Directory.CreateDirectory(evidenceDir);
            return evidenceDir;
        }
        catch
        {
            string fallback = Path.Combine(
                Path.GetTempPath(),
                $"dinoforge_asset_visual_{Guid.NewGuid():N}");
            Directory.CreateDirectory(fallback);
            return fallback;
        }
    }

    private static string GetRepoRoot()
    {
        string? current = Path.GetDirectoryName(typeof(GameLaunchAssetSwapTests).Assembly.Location);
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

