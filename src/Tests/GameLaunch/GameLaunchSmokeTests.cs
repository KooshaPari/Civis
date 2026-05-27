#nullable enable
using System.Diagnostics;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-001: BepInEx bootstraps DINOForge and the bridge is healthy.
/// Prerequisite: <see cref="GameLaunchFixture"/> has already waited for healthy ping.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchSmokeTests(GameLaunchFixture fixture)
{
    [SkippableFact]
    public async Task Bridge_IsHealthy_AfterBootstrap()
    {
        fixture.SkipIfNotInitialized();

        GameStatus status = await fixture.Client!.StatusAsync();
        status.WorldReady.Should().BeTrue("DINOForge plugin should report world ready after BepInEx bootstrap");
        status.EntityCount.Should().BeGreaterThan(0, "the ECS world should have spawned entities");
    }

    [SkippableFact]
    public async Task Bridge_Ping_RoundTripUnderOneSecond()
    {
        fixture.SkipIfNotInitialized();

        Stopwatch sw = Stopwatch.StartNew();
        await fixture.Client!.PingAsync();
        sw.Stop();

        sw.ElapsedMilliseconds.Should().BeLessThan(1000,
            "bridge round-trip over named pipe; allow headroom for self-hosted runners and loaded game");
    }
}
