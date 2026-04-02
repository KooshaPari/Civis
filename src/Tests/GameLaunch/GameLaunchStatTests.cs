#nullable enable
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
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
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task StatOverride_HP_PersistsAfterReload()
    {
        const string sdkPath = "units/rep_clone_trooper/stats/hp";
        const float overrideHp = 999f;

        // Apply override
        OverrideResult overrideResult = await fixture.Client!.ApplyOverrideAsync(
            sdkPath: sdkPath,
            value: overrideHp);

        overrideResult.Success.Should().BeTrue("override should apply without error");

        // Reload packs
        ReloadResult reloadResult = await fixture.Client.ReloadPacksAsync();
        reloadResult.Success.Should().BeTrue("reload should succeed");

        // Verify stat persisted
        StatResult statResult = await fixture.Client.GetStatAsync(sdkPath);
        statResult.Value.Should().BeApproximately(overrideHp, precision: 0.1f,
            "HP override should survive ReloadPacks");
    }
}
