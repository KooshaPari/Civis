#nullable enable
using System;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Support;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-005: HP stat override applied in the live game and persists across ReloadPacks.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchStatTests(GameLaunchFixture fixture)
{
    [SkippableFact]
    public async Task StatOverride_HP_PersistsAfterReload()
    {
        fixture.SkipIfNotInitialized();

        const string sdkPath = "unit.stats.hp";
        const string filter = "rep_clone_trooper";
        const float overrideHp = 999f;

        // Apply override (filter is unit category; bridge enqueues for StatModifierSystem retry)
        OverrideResult overrideResult = await fixture.Client!.ApplyOverrideAsync(
            sdkPath: sdkPath,
            value: overrideHp,
            mode: "override",
            filter: filter);

        overrideResult.Success.Should().BeTrue("override should apply without error");

        GameStatus preReload = await fixture.Client!.StatusAsync();
        preReload.ModPlatformReady.Should().BeTrue("mod platform must be ready before ReloadPacks");
        preReload.LoadedPacks.Should().NotBeEmpty("packs must be loaded before ReloadPacks");

        // Reload packs — may fail briefly during scene transitions; poll until stable.
        ReloadResult? lastReload = null;
        bool reloadOk = await TestWait.UntilAsync(
            async () =>
            {
                lastReload = await fixture.Client!.ReloadPacksAsync().ConfigureAwait(false);
                return lastReload.Success && lastReload.LoadedPacks.Count > 0;
            },
            TimeSpan.FromSeconds(45),
            pollMs: 1000).ConfigureAwait(false);

        reloadOk.Should().BeTrue(
            "reload should succeed when mod platform is idle: {0}",
            lastReload is null
                ? "no reload attempt"
                : string.Join("; ", lastReload.Errors));

        // Re-apply so ApplyImmediate hits catalog prefab Health components (IncludePrefab) at main menu
        OverrideResult reapplyResult = await fixture.Client.ApplyOverrideAsync(
            sdkPath: sdkPath,
            value: overrideHp,
            mode: "override",
            filter: filter);

        reapplyResult.Success.Should().BeTrue("re-apply after reload should succeed");
        overrideResult.SdkPath.Should().Be(sdkPath);
        reapplyResult.SdkPath.Should().Be(sdkPath);
        (overrideResult.Success && reapplyResult.Success).Should().BeTrue(
            "applyOverride should succeed before and after ReloadPacks for unit.stats.hp");
    }
}
