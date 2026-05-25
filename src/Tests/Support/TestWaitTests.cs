using System;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Support;

[CollectionDefinition("TestWait", DisableParallelization = true)]
public class TestWaitCollection;

[Collection("TestWait")]
[Trait("Category", "Support")]
public class TestWaitTests
{
    /// <summary>Advances virtual time for deadline checks without relying on wall-clock Task.Delay under load.</summary>
    private sealed class SteppingTimeProvider : TimeProvider
    {
        private DateTimeOffset _now;

        public SteppingTimeProvider(DateTimeOffset start) => _now = start;

        public override DateTimeOffset GetUtcNow() => _now;

        public void Advance(TimeSpan step) => _now = _now.Add(step);
    }

    [Fact]
    public async Task UntilAsync_PredicateTrueImmediately_ReturnsTrue()
    {
        var sw = Stopwatch.StartNew();
        var result = await TestWait.UntilAsync(
            () => true,
            TimeSpan.FromSeconds(2),
            pollMs: 25);
        sw.Stop();

        result.Should().BeTrue();
        sw.ElapsedMilliseconds.Should().BeLessThan(500,
            "predicate true on first eval — must not wait a full poll interval");
    }

    [Fact]
    public async Task UntilAsync_PredicateNeverTrue_ReturnsFalseAfterTimeout()
    {
        var sw = Stopwatch.StartNew();
        var result = await TestWait.UntilAsync(
            () => false,
            TimeSpan.FromMilliseconds(200),
            pollMs: 25);
        sw.Stop();

        result.Should().BeFalse();
        sw.ElapsedMilliseconds.Should().BeGreaterOrEqualTo(150,
            "must spin until at least the timeout window before returning false");
    }

    [Fact]
    public async Task UntilAsync_PredicateBecomesTrue_ReturnsTrueWithinTimeout()
    {
        var clock = new SteppingTimeProvider(DateTimeOffset.UtcNow);
        var becomesTrueAt = clock.GetUtcNow().AddMilliseconds(150);
        var pollCount = 0;

        bool Probe()
        {
            pollCount++;
            clock.Advance(TimeSpan.FromMilliseconds(50));
            return clock.GetUtcNow() >= becomesTrueAt;
        }

        var result = await TestWait.UntilAsync(
            Probe,
            TimeSpan.FromSeconds(2),
            pollMs: 25,
            timeProvider: clock);

        result.Should().BeTrue();
        pollCount.Should().BeGreaterOrEqualTo(3,
            "should have polled until the virtual trigger time");
        pollCount.Should().BeLessThan(50,
            "should NOT spin until full virtual timeout when predicate flips");
    }

    [Fact]
    public async Task UntilAsync_AsyncPredicate_Works()
    {
        var counter = 0;
        var result = await TestWait.UntilAsync(
            async () =>
            {
                await Task.CompletedTask.ConfigureAwait(false);
                return Interlocked.Increment(ref counter) >= 3;
            },
            TimeSpan.FromSeconds(2),
            pollMs: 10);

        result.Should().BeTrue();
        counter.Should().BeGreaterOrEqualTo(3);
    }

    [Fact]
    public async Task UntilAsync_NullPredicate_Throws()
    {
        Func<Task> act = () => TestWait.UntilAsync(
            (Func<bool>)null!,
            TimeSpan.FromMilliseconds(50));
        await act.Should().ThrowAsync<ArgumentNullException>();
    }

    [Fact]
    public async Task UntilAsync_CancellationRequested_Throws()
    {
        using var cts = new CancellationTokenSource();
        cts.CancelAfter(TimeSpan.FromMilliseconds(100));

        Func<Task> act = () => TestWait.UntilAsync(
            () => false,
            TimeSpan.FromSeconds(2),
            pollMs: 25,
            ct: cts.Token);

        await act.Should().ThrowAsync<OperationCanceledException>();
    }
}
