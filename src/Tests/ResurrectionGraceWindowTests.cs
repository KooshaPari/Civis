namespace DINOForge.Tests;

using System;
using FluentAssertions;
using Xunit;

/// <summary>
/// Characterization tests for the FailureMode B fix (iter-149, 2026-05-29):
/// the resurrection-fallback grace window must be latched in a STATIC deadline so that a
/// loop restart RESUMES the in-progress window instead of resetting it.
///
/// The production logic lives in <c>DINOForge.Runtime.Plugin.ResurrectionFallbackLoop</c>
/// (netstandard2.0, Unity-coupled, not file-linkable). This test replicates the pure
/// deadline-latch state machine to lock in the invariant that previously regressed:
/// with a LOCAL timer, every loop re-entry reset the window so the grace never elapsed and
/// TryResurrect was detected-but-never-executed.
/// </summary>
public class ResurrectionGraceWindowTests
{
    private const int GraceWindowMs = 4000;

    /// <summary>Mirror of Plugin's static latch (the field that survives loop restarts).</summary>
    private sealed class GraceLatch
    {
        private DateTime _deadlineUtc = DateTime.MinValue; // MinValue = disarmed

        /// <summary>Replicates IsGraceWindowElapsed: arms once, then returns true after deadline.</summary>
        public bool IsElapsed(DateTime now, int graceWindowMs, out bool justArmed)
        {
            justArmed = false;
            if (_deadlineUtc == DateTime.MinValue)
            {
                _deadlineUtc = now.AddMilliseconds(graceWindowMs);
                justArmed = true;
                return false;
            }
            return now >= _deadlineUtc;
        }

        public void Reset() => _deadlineUtc = DateTime.MinValue;

        public void Rearm(DateTime now, int graceWindowMs) => _deadlineUtc = now.AddMilliseconds(graceWindowMs);

        public bool IsArmed => _deadlineUtc != DateTime.MinValue;
    }

    [Fact]
    public void GraceWindow_ArmsOnFirstObservation_DoesNotFireImmediately()
    {
        GraceLatch latch = new();
        DateTime t0 = new(2026, 5, 29, 12, 0, 0, DateTimeKind.Utc);

        bool elapsed = latch.IsElapsed(t0, GraceWindowMs, out bool justArmed);

        elapsed.Should().BeFalse("the window was only just armed");
        justArmed.Should().BeTrue();
        latch.IsArmed.Should().BeTrue();
    }

    [Fact]
    public void GraceWindow_PersistsAcrossLoopRestart_FiresAfterDeadline()
    {
        // This is the core FailureMode B regression: a NEW loop pass (simulating a thread/loop
        // restart) must HONOR the deadline armed by the previous pass — not reset it.
        GraceLatch latch = new();
        DateTime t0 = new(2026, 5, 29, 12, 0, 0, DateTimeKind.Utc);

        // Pass 1 (loop instance A): arms the deadline.
        latch.IsElapsed(t0, GraceWindowMs, out bool armed1).Should().BeFalse();
        armed1.Should().BeTrue();

        // Simulate the loop restarting many times BEFORE the grace window elapses.
        // With the OLD local-variable bug this would re-arm every pass and never fire.
        for (int i = 1; i <= 5; i++)
        {
            DateTime mid = t0.AddMilliseconds(i * 500); // 0.5s..2.5s — still inside 4s window
            latch.IsElapsed(mid, GraceWindowMs, out bool armedN).Should().BeFalse($"still within grace at +{i * 500}ms");
            armedN.Should().BeFalse("the deadline was already armed by pass 1 and must persist");
        }

        // Now a pass occurs AFTER the original deadline — it must fire.
        DateTime afterDeadline = t0.AddMilliseconds(GraceWindowMs + 1);
        latch.IsElapsed(afterDeadline, GraceWindowMs, out _).Should().BeTrue("the persisted deadline has elapsed");
    }

    [Fact]
    public void GraceWindow_Reset_DisarmsForFreshNeed()
    {
        GraceLatch latch = new();
        DateTime t0 = new(2026, 5, 29, 12, 0, 0, DateTimeKind.Utc);

        latch.IsElapsed(t0, GraceWindowMs, out _);
        latch.IsArmed.Should().BeTrue();

        latch.Reset(); // need cleared (revive succeeded or no need observed)
        latch.IsArmed.Should().BeFalse();

        // A fresh need re-arms cleanly.
        latch.IsElapsed(t0.AddSeconds(30), GraceWindowMs, out bool reArmed).Should().BeFalse();
        reArmed.Should().BeTrue();
    }

    [Fact]
    public void GraceWindow_Rearm_AfterPartialRevive_ExtendsDeadline()
    {
        // When TryResurrect runs but the driver is not yet live, the need is RETAINED and the
        // deadline RE-ARMED for another attempt (never silently dropped).
        GraceLatch latch = new();
        DateTime t0 = new(2026, 5, 29, 12, 0, 0, DateTimeKind.Utc);

        latch.IsElapsed(t0, GraceWindowMs, out _);
        DateTime fired = t0.AddMilliseconds(GraceWindowMs + 1);
        latch.IsElapsed(fired, GraceWindowMs, out _).Should().BeTrue();

        // Partial revive -> re-arm from the fire time.
        latch.Rearm(fired, GraceWindowMs);

        // Immediately after re-arm it must NOT fire again.
        latch.IsElapsed(fired.AddMilliseconds(100), GraceWindowMs, out _).Should().BeFalse();
        // But it fires again after the new window elapses.
        latch.IsElapsed(fired.AddMilliseconds(GraceWindowMs + 1), GraceWindowMs, out _).Should().BeTrue();
    }
}
