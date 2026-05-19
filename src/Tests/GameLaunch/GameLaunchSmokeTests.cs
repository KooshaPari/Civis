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
    [Fact]
    public async Task Bridge_IsHealthy_AfterBootstrap()
    {
        if (!fixture.IsInitialized)
        {
            return;  // Skip test when game is not available
        }

        GameStatus status = await fixture.Client!.StatusAsync();
        status.WorldReady.Should().BeTrue("DINOForge plugin should report world ready after BepInEx bootstrap");
        status.EntityCount.Should().BeGreaterThan(0, "the ECS world should have spawned entities");
    }

    [Fact]
    public async Task Bridge_Ping_RoundTripUnder100Ms()
    {
        if (!fixture.IsInitialized) return;

        Stopwatch sw = Stopwatch.StartNew();
        await fixture.Client!.PingAsync();
        sw.Stop();

        sw.ElapsedMilliseconds.Should().BeLessThan(100,
            "bridge should be local-process latency, not network latency");
    }
}
