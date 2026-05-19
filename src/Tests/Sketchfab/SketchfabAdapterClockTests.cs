#nullable enable
using System;
using System.Net.Http;
using System.Reflection;
using DINOForge.Tools.Cli.Assetctl.Sketchfab;
using FluentAssertions;
using Microsoft.Extensions.Logging.Abstractions;
using Xunit;

namespace DINOForge.Tests.Sketchfab;

/// <summary>
/// Verifies SketchfabAdapter respects an injected <see cref="TimeProvider"/> for
/// quota-cache TTL and rate-limit reset arithmetic (Pattern #82 / Task #216 Phase 1).
///
/// Test approach: a tiny inline <see cref="FixedTimeProvider"/> rather than pulling in
/// Microsoft.Extensions.Time.Testing — keeps scope tight per dispatch instructions.
///
/// Cache-hit cases are exercised by seeding the adapter's private cache fields via
/// reflection; this lets us validate the time-comparison branch without standing up
/// an HttpClient/SketchfabClient mock.
/// </summary>
public class SketchfabAdapterClockTests
{
    private static readonly DateTime BaseUtc = new(2026, 4, 25, 12, 0, 0, DateTimeKind.Utc);

    /// <summary>
    /// Tiny manually-advanced TimeProvider — avoids adding Microsoft.Extensions.Time.Testing.
    /// </summary>
    private sealed class FixedTimeProvider : TimeProvider
    {
        private DateTimeOffset _now;
        public FixedTimeProvider(DateTimeOffset now) => _now = now;
        public override DateTimeOffset GetUtcNow() => _now;
        public void Advance(TimeSpan delta) => _now = _now.Add(delta);
    }

    private static SketchfabAdapter NewAdapter(TimeProvider clock)
    {
        // Internal ctor (string apiToken, HttpClient httpClient, SketchfabClientOptions? options)
        // lets us avoid spinning up a real network client.
        var ctor = typeof(SketchfabClient).GetConstructor(
            BindingFlags.Instance | BindingFlags.NonPublic,
            binder: null,
            types: new[] { typeof(string), typeof(HttpClient), typeof(SketchfabClientOptions) },
            modifiers: null);
        ctor.Should().NotBeNull("internal SketchfabClient ctor should remain available for tests");
        var http = new HttpClient();
        var client = (SketchfabClient)ctor!.Invoke(new object?[] { "test-token", http, null });

        return new SketchfabAdapter(client, NullLogger<SketchfabAdapter>.Instance, clock);
    }

    private static void SeedCache(SketchfabAdapter adapter, int remaining, DateTime resetAtUtc, DateTime lastCheckUtc)
    {
        var t = typeof(SketchfabAdapter);
        var flags = BindingFlags.Instance | BindingFlags.NonPublic;
        t.GetField("_rateLimitRemaining", flags)!.SetValue(adapter, remaining);
        t.GetField("_rateLimitReset", flags)!.SetValue(adapter, resetAtUtc);
        t.GetField("_lastQuotaCheck", flags)!.SetValue(adapter, lastCheckUtc);
    }

    [Fact]
    public async System.Threading.Tasks.Task GetQuotaAsync_ReturnsCachedState_WhenWithinTtl()
    {
        // Arrange: clock at T0, seed cache as if checked just now.
        var clock = new FixedTimeProvider(BaseUtc);
        var adapter = NewAdapter(clock);
        var resetAt = BaseUtc.AddMinutes(15);
        SeedCache(adapter, remaining: 42, resetAtUtc: resetAt, lastCheckUtc: BaseUtc);

        // Advance 30s — still inside the 60s TTL.
        clock.Advance(TimeSpan.FromSeconds(30));

        // Act
        var quota = await adapter.GetQuotaAsync();

        // Assert: returned cached value (no network call needed since we got Remaining=42 back).
        quota.Should().NotBeNull();
        quota!.Remaining.Should().Be(42);
        quota.ResetAtUtc.Should().Be(resetAt);
        quota.LastCheckedUtc.Should().Be(BaseUtc);
    }

    [Fact]
    public void Constructor_SeedsRateLimitReset_FromInjectedTimeProvider()
    {
        // The +1h reset window initializer should use the injected clock, not wall-clock.
        var clock = new FixedTimeProvider(BaseUtc);
        var adapter = NewAdapter(clock);

        var actualReset = (DateTime)typeof(SketchfabAdapter)
            .GetField("_rateLimitReset", BindingFlags.Instance | BindingFlags.NonPublic)!
            .GetValue(adapter)!;

        actualReset.Should().Be(BaseUtc.AddHours(1));
    }

    [Fact]
    public async System.Threading.Tasks.Task GetQuotaAsync_ResetInSeconds_ComputedFromInjectedClock()
    {
        // Arrange: cache says reset at T0+10min, clock at T0+2min (still inside 60s TTL? -> need 2min < 60s? no).
        // We want a fresh cache (within TTL) at T+30s, with resetAt at T+10min => ResetInSeconds ~570.
        var clock = new FixedTimeProvider(BaseUtc);
        var adapter = NewAdapter(clock);
        var resetAt = BaseUtc.AddMinutes(10);
        SeedCache(adapter, remaining: 7, resetAtUtc: resetAt, lastCheckUtc: BaseUtc);

        clock.Advance(TimeSpan.FromSeconds(30)); // now=T+30s; ttl freshness OK; reset is in 9m30s = 570s.

        var quota = await adapter.GetQuotaAsync();

        quota.Should().NotBeNull();
        quota!.Remaining.Should().Be(7);
        // 10min - 30s = 570s
        quota.ResetInSeconds.Should().Be(570);
    }
}
