#nullable enable
using System;
using System.Net.Http;
using System.Reflection;
using DINOForge.Tools.Cli.Assetctl.Sketchfab;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Sketchfab;

/// <summary>
/// Verifies SketchfabClient respects an injected <see cref="TimeProvider"/> for
/// rate-limit timestamps and ResetInSeconds arithmetic (Pattern #82 / Task #216 Phase 2).
///
/// Mirrors the SketchfabAdapterClockTests pattern: FixedTimeProvider + reflection-seeded
/// private rate-limit state to validate the time-comparison branches without standing up
/// a live HTTP endpoint.
/// </summary>
public class SketchfabClientClockTests
{
    private static readonly DateTime BaseUtc = new(2026, 4, 25, 12, 0, 0, DateTimeKind.Utc);

    /// <summary>
    /// Tiny manually-advanced TimeProvider — same shape as SketchfabAdapterClockTests.
    /// </summary>
    private sealed class FixedTimeProvider : TimeProvider
    {
        private DateTimeOffset _now;
        public FixedTimeProvider(DateTimeOffset now) => _now = now;
        public override DateTimeOffset GetUtcNow() => _now;
        public void Advance(TimeSpan delta) => _now = _now.Add(delta);
    }

    private static SketchfabClient NewClient(TimeProvider clock)
    {
        // Use the 4-arg internal ctor (string, HttpClient, SketchfabClientOptions?, TimeProvider?)
        // added in #216 Phase 2 — lets us inject a clock without standing up real HTTP.
        var ctor = typeof(SketchfabClient).GetConstructor(
            BindingFlags.Instance | BindingFlags.NonPublic,
            binder: null,
            types: new[] { typeof(string), typeof(HttpClient), typeof(SketchfabClientOptions), typeof(TimeProvider) },
            modifiers: null);
        ctor.Should().NotBeNull("internal SketchfabClient ctor with TimeProvider should be available for tests");
        var http = new HttpClient();
        return (SketchfabClient)ctor!.Invoke(new object?[] { "test-token", http, null, clock });
    }

    private static void SeedRateLimit(SketchfabClient client, int remaining, long resetUnix, DateTime lastCheckUtc)
    {
        var t = typeof(SketchfabClient);
        var flags = BindingFlags.Instance | BindingFlags.NonPublic;
        t.GetField("_rateLimitRemaining", flags)!.SetValue(client, remaining);
        t.GetField("_rateLimitResetUnix", flags)!.SetValue(client, resetUnix);
        t.GetField("_lastRateLimitCheck", flags)!.SetValue(client, lastCheckUtc);
    }

    [Fact]
    public void GetRateLimitState_ResetInSeconds_ComputedFromInjectedClock()
    {
        // Arrange: clock at T0, reset 10 minutes after T0 (Unix ts).
        var clock = new FixedTimeProvider(BaseUtc);
        var client = NewClient(clock);
        var resetAt = BaseUtc.AddMinutes(10);
        var resetUnix = (long)(resetAt - DateTime.UnixEpoch).TotalSeconds;
        SeedRateLimit(client, remaining: 17, resetUnix: resetUnix, lastCheckUtc: BaseUtc);

        // Advance 30s — reset is now 9m30s away (570s).
        clock.Advance(TimeSpan.FromSeconds(30));

        // Act
        var state = client.GetRateLimitState();

        // Assert
        state.Should().NotBeNull();
        state!.Remaining.Should().Be(17);
        state.ResetInSeconds.Should().Be(570);
        state.LastCheckedUtc.Should().Be(BaseUtc);
    }

    [Fact]
    public void GetRateLimitState_ResetInSeconds_ClampedToZero_WhenResetInPast()
    {
        // Arrange: clock at T0, reset already 5 minutes ago (Unix ts).
        var clock = new FixedTimeProvider(BaseUtc);
        var client = NewClient(clock);
        var resetAt = BaseUtc.AddMinutes(-5);
        var resetUnix = (long)(resetAt - DateTime.UnixEpoch).TotalSeconds;
        SeedRateLimit(client, remaining: 0, resetUnix: resetUnix, lastCheckUtc: BaseUtc.AddMinutes(-6));

        // Act
        var state = client.GetRateLimitState();

        // Assert: Math.Max(0, ...) clamps negative to zero — proves arithmetic uses injected clock,
        // not wall-clock (which would yield a very large positive number).
        state.Should().NotBeNull();
        state!.ResetInSeconds.Should().Be(0);
    }

    [Fact]
    public void UpdateRateLimitState_SetsLastCheck_FromInjectedClock()
    {
        // Arrange: clock at T0; create a fake response with rate-limit headers.
        var clock = new FixedTimeProvider(BaseUtc);
        var client = NewClient(clock);

        var response = new HttpResponseMessage(System.Net.HttpStatusCode.OK);
        response.Headers.Add("X-RateLimit-Remaining", "42");
        response.Headers.Add("X-RateLimit-Reset", "1893456000");

        // Advance 90s before invoking UpdateRateLimitState — _lastRateLimitCheck should record T+90s.
        clock.Advance(TimeSpan.FromSeconds(90));

        // Act: invoke the private UpdateRateLimitState via reflection.
        var update = typeof(SketchfabClient).GetMethod(
            "UpdateRateLimitState",
            BindingFlags.Instance | BindingFlags.NonPublic);
        update.Should().NotBeNull();
        update!.Invoke(client, new object?[] { response });

        // Assert
        var state = client.GetRateLimitState();
        state.Should().NotBeNull();
        state!.Remaining.Should().Be(42);
        state.LastCheckedUtc.Should().Be(BaseUtc.AddSeconds(90));
    }
}
