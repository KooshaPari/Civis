#nullable enable
using System;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.SDK.Concurrency;

using TimeProvider = System.TimeProvider;

/// <summary>
/// Retry-with-exponential-backoff polling utility for async operations.
/// Patterns #113 (polling primitives) and #108 (sleep-based test sync).
/// Replaces hand-rolled loops with consistent backoff semantics.
/// </summary>
public static class PollingHelper
{
    /// <summary>
    /// Poll a non-null/non-default predicate with exponential backoff until success, timeout expires, or cancellation.
    /// </summary>
    /// <typeparam name="T">Reference type (class) to poll for.</typeparam>
    /// <param name="probe">Predicate that returns non-null T on success.</param>
    /// <param name="timeout">Maximum time to poll before returning null.</param>
    /// <param name="initialDelay">First retry delay. Default 50ms.</param>
    /// <param name="backoffFactor">Exponential backoff multiplier. Default 2.0.</param>
    /// <param name="maxDelay">Cap on retry delay. Default 800ms.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <param name="timeProvider">Source of current time for deadline calculations. Defaults to <see cref="System.TimeProvider.System"/>.</param>
    /// <returns>Non-null T if probe succeeds, null if timeout or cancellation.</returns>
    /// <remarks>
    /// Loop: probe → if non-null return; else delay → double delay (capped); respect ct + timeout.
    /// Supports both fire-and-forget fire-and-forget probes and stateful re-queries.
    /// Example: await PollingHelper.RetryUntilAsync(() => GetSessionIdIfReady(), timeout: TimeSpan.FromSeconds(5)).ConfigureAwait(false);
    /// </remarks>
    public static async Task<T?> RetryUntilAsync<T>(
        Func<T?> probe,
        TimeSpan timeout,
        TimeSpan initialDelay = default,
        double backoffFactor = 2.0,
        TimeSpan maxDelay = default,
        CancellationToken ct = default,
        TimeProvider? timeProvider = null) where T : class
    {
        if (probe is null)
            throw new ArgumentNullException(nameof(probe));

        // Normalize defaults per docstring
        timeProvider ??= TimeProvider.System;
        TimeSpan currentDelay = initialDelay == default ? TimeSpan.FromMilliseconds(50) : initialDelay;
        TimeSpan actualMaxDelay = maxDelay == default ? TimeSpan.FromMilliseconds(800) : maxDelay;
        DateTime deadline = timeProvider.GetUtcNow().UtcDateTime.Add(timeout);

        while (true)
        {
            // Probe first (non-null = success)
            T? result = probe();
            if (result is not null)
                return result;

            // Check timeout/cancellation before sleeping
            ct.ThrowIfCancellationRequested();
            if (timeProvider.GetUtcNow().UtcDateTime >= deadline)
                return null;

            // Delay with cancellation support
            TimeSpan remaining = deadline - timeProvider.GetUtcNow().UtcDateTime;
            TimeSpan delayTime = TimeSpan.FromMilliseconds(Math.Min(currentDelay.TotalMilliseconds, remaining.TotalMilliseconds));
            if (delayTime > TimeSpan.Zero)
                await Task.Delay(delayTime, ct);

            // Exponential backoff (capped at actualMaxDelay)
            double nextDelayMs = currentDelay.TotalMilliseconds * backoffFactor;
            currentDelay = TimeSpan.FromMilliseconds(Math.Min(nextDelayMs, actualMaxDelay.TotalMilliseconds));
        }
    }

    /// <summary>
    /// Poll a boolean predicate with exponential backoff until it returns true, timeout expires, or cancellation.
    /// </summary>
    /// <param name="probe">Predicate that returns true on success.</param>
    /// <param name="timeout">Maximum time to poll before returning false.</param>
    /// <param name="initialDelay">First retry delay. Default 50ms.</param>
    /// <param name="backoffFactor">Exponential backoff multiplier. Default 2.0.</param>
    /// <param name="maxDelay">Cap on retry delay. Default 800ms.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <param name="timeProvider">Source of current time for deadline calculations. Defaults to <see cref="System.TimeProvider.System"/>.</param>
    /// <returns>True if probe succeeds, false if timeout or cancellation.</returns>
    /// <remarks>
    /// Loop: probe → if true return; else delay → double delay (capped); respect ct + timeout.
    /// Example: await PollingHelper.RetryUntilTrueAsync(() => process.HasExited, timeout: TimeSpan.FromSeconds(10)).ConfigureAwait(false);
    /// </remarks>
    public static async Task<bool> RetryUntilTrueAsync(
        Func<bool> probe,
        TimeSpan timeout,
        TimeSpan initialDelay = default,
        double backoffFactor = 2.0,
        TimeSpan maxDelay = default,
        CancellationToken ct = default,
        TimeProvider? timeProvider = null)
    {
        if (probe is null)
            throw new ArgumentNullException(nameof(probe));

        // Normalize defaults per docstring
        timeProvider ??= TimeProvider.System;
        TimeSpan currentDelay = initialDelay == default ? TimeSpan.FromMilliseconds(50) : initialDelay;
        TimeSpan actualMaxDelay = maxDelay == default ? TimeSpan.FromMilliseconds(800) : maxDelay;
        DateTime deadline = timeProvider.GetUtcNow().UtcDateTime.Add(timeout);

        while (true)
        {
            // Probe first (true = success)
            if (probe())
                return true;

            // Check timeout/cancellation before sleeping
            ct.ThrowIfCancellationRequested();
            if (timeProvider.GetUtcNow().UtcDateTime >= deadline)
                return false;

            // Delay with cancellation support
            TimeSpan remaining = deadline - timeProvider.GetUtcNow().UtcDateTime;
            TimeSpan delayTime = TimeSpan.FromMilliseconds(Math.Min(currentDelay.TotalMilliseconds, remaining.TotalMilliseconds));
            if (delayTime > TimeSpan.Zero)
                await Task.Delay(delayTime, ct);

            // Exponential backoff (capped at actualMaxDelay)
            double nextDelayMs = currentDelay.TotalMilliseconds * backoffFactor;
            currentDelay = TimeSpan.FromMilliseconds(Math.Min(nextDelayMs, actualMaxDelay.TotalMilliseconds));
        }
    }
}
