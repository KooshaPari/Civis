using System;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.Tests.Support;

/// <summary>
/// Test-side polling helpers replacing Task.Delay-based synchronization.
/// Pattern #108 — sleep-based sync is fragile (slow CI flakes; fast machines waste wall-clock).
/// </summary>
public static class TestWait
{
    /// <summary>
    /// Poll predicate at <paramref name="pollMs"/> intervals until it returns true OR timeout elapses.
    /// Returns true if predicate succeeded; false if timed out.
    /// </summary>
    public static async Task<bool> UntilAsync(
        Func<bool> predicate,
        TimeSpan timeout,
        int pollMs = 50,
        CancellationToken ct = default)
    {
        if (predicate is null) throw new ArgumentNullException(nameof(predicate));
        var sw = System.Diagnostics.Stopwatch.StartNew();
        while (sw.Elapsed < timeout)
        {
            ct.ThrowIfCancellationRequested();
            if (predicate()) return true;
            try { await Task.Delay(pollMs, ct).ConfigureAwait(false); }
            catch (OperationCanceledException) { throw; }
        }
        return false;
    }

    /// <summary>
    /// Async predicate variant — supports awaiting bridge calls etc.
    /// </summary>
    public static async Task<bool> UntilAsync(
        Func<Task<bool>> predicate,
        TimeSpan timeout,
        int pollMs = 50,
        CancellationToken ct = default)
    {
        if (predicate is null) throw new ArgumentNullException(nameof(predicate));
        var sw = System.Diagnostics.Stopwatch.StartNew();
        while (sw.Elapsed < timeout)
        {
            ct.ThrowIfCancellationRequested();
            if (await predicate().ConfigureAwait(false)) return true;
            try { await Task.Delay(pollMs, ct).ConfigureAwait(false); }
            catch (OperationCanceledException) { throw; }
        }
        return false;
    }
}
