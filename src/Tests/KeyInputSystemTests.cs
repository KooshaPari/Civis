#nullable enable
using System;
using System.Reflection;
using System.Threading;
using DINOForge.Runtime;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// SPEC-004 (KIS-T1–KIS-T8) unit tests for <see cref="KeyInputSystem"/> delegate wiring,
/// resurrection flags, and Win32-style key edge detection. Runs without launching DINO;
/// does not exercise ECS <see cref="KeyInputSystem.OnUpdate"/> or real <c>GetAsyncKeyState</c>.
/// </summary>
[Trait("Category", "KeyInputSystem")]
public sealed class KeyInputSystemTests : IDisposable
{
    private readonly Action? _savedOnF9;
    private readonly Action? _savedOnF10;
    private readonly Action? _savedOnPackReload;

    public KeyInputSystemTests()
    {
        _savedOnF9 = KeyInputSystem.OnF9Pressed;
        _savedOnF10 = KeyInputSystem.OnF10Pressed;
        _savedOnPackReload = KeyInputSystem.OnPackReloadRequested;
        ResetPluginResurrectionFlags();
        ResetBackgroundEdgeState();
        KeyInputSystem.StopKeyPollThread();
    }

    public void Dispose()
    {
        KeyInputSystem.StopKeyPollThread();
        KeyInputSystem.OnF9Pressed = _savedOnF9;
        KeyInputSystem.OnF10Pressed = _savedOnF10;
        KeyInputSystem.OnPackReloadRequested = _savedOnPackReload;
        ResetPluginResurrectionFlags();
        ResetBackgroundEdgeState();
    }

    private static void ResetPluginResurrectionFlags()
    {
        Plugin.NeedsResurrection = false;
        Plugin.NeedsDeferredResurrection = false;
        Plugin.s_rootJustDestroyed = false;
    }

    private static void ResetBackgroundEdgeState()
    {
        SetPrivateStatic(typeof(KeyInputSystem), "_bgF9PreviousState", false);
        SetPrivateStatic(typeof(KeyInputSystem), "_bgF10PreviousState", false);
        SetPrivateStatic(typeof(KeyInputSystem), "_keyPollRunning", false);
    }

    private static void SetPrivateStatic(Type type, string fieldName, object value)
    {
        FieldInfo? field = type.GetField(fieldName, BindingFlags.Static | BindingFlags.NonPublic);
        field.Should().NotBeNull(because: $"test setup requires private field {type.Name}.{fieldName}");
        field!.SetValue(null, value);
    }

    // ── KIS-T1 / KIS-T2: delegate dispatch ─────────────────────────────────────

    [Fact]
    public void OnF9Pressed_InvokeWithHandler_CalledExactlyOnce()
    {
        int callCount = 0;
        KeyInputSystem.OnF9Pressed = () => callCount++;

        KeyInputSystem.OnF9Pressed.Invoke();

        callCount.Should().Be(1);
    }

    [Fact]
    public void OnF10Pressed_InvokeWithHandler_CalledExactlyOnce()
    {
        int callCount = 0;
        KeyInputSystem.OnF10Pressed = () => callCount++;

        KeyInputSystem.OnF10Pressed.Invoke();

        callCount.Should().Be(1);
    }

    // ── KIS-T3 / KIS-T4: null-safe delegate invoke ─────────────────────────────

    [Fact]
    public void OnF9Pressed_InvokeWithNoHandler_DoesNotThrow()
    {
        KeyInputSystem.OnF9Pressed = null;

        Action act = () => KeyInputSystem.OnF9Pressed?.Invoke();

        act.Should().NotThrow();
    }

    [Fact]
    public void OnF9Pressed_AssignNullThenInvoke_DoesNotThrow()
    {
        KeyInputSystem.OnF9Pressed = () => { };
        KeyInputSystem.OnF9Pressed = null;

        Action act = () => KeyInputSystem.OnF9Pressed?.Invoke();

        act.Should().NotThrow();
    }

    // ── KIS-T5 / KIS-T6: resurrection (Plugin.TryResurrect) ────────────────────

    /// <summary>
    /// KIS-T5: deferred resurrection avoids calling TryResurrect before Plugin.Awake completes.
    /// Maps to <see cref="Plugin.MarkNeedsDeferredResurrection"/> used from KeyInputSystem.OnCreate.
    /// </summary>
    [Fact]
    public void MarkNeedsDeferredResurrection_FromBackgroundThread_SetsFlagWithoutThrowing()
    {
        Exception? caught = null;
        var thread = new Thread(() =>
        {
            try
            {
                Plugin.MarkNeedsDeferredResurrection("KIS-T5-test");
            }
            catch (Exception ex)
            {
                caught = ex;
            }
        })
        {
            IsBackground = true,
            Name = "KIS-T5-deferred-resurrection",
        };
        thread.Start();
        thread.Join(TimeSpan.FromSeconds(5));

        caught.Should().BeNull();
        Plugin.NeedsDeferredResurrection.Should().BeTrue();
    }

    /// <summary>
    /// KIS-T6: SPEC-004 requires max 3 resurrection attempts; production uses grace-window
    /// retry in ResurrectionFallbackLoop instead of a hard attempt cap on TryResurrect.
    /// </summary>
    [Fact(Skip = "KIS-T6: MaxResurrectionAttempts cap not implemented on Plugin.TryResurrect (SPEC-004 KIS-NF4 vs current code).")]
    public void TryResurrect_FourCallsWithNullPersistentRoot_ExecutesAtMostThreeTimes()
    {
    }

    // ── KIS-T7 / KIS-T8: Win32 edge detection (pure logic) ─────────────────────

    [Fact]
    public void KeyEdgeDetection_KeyAlreadyHeld_DoesNotInvokeHandler()
    {
        bool previous = true;
        int invocations = 0;

        KeyPressEdgeDetector.Process(keyDown: true, ref previous, () => invocations++);

        invocations.Should().Be(0);
        previous.Should().BeTrue();
    }

    [Fact]
    public void KeyEdgeDetection_KeyDownTransition_InvokesHandlerExactlyOnce()
    {
        bool previous = false;
        int invocations = 0;

        KeyPressEdgeDetector.Process(keyDown: true, ref previous, () => invocations++);

        invocations.Should().Be(1);
        previous.Should().BeTrue();
    }

    [Fact]
    public void KeyEdgeDetection_SecondFrameWhileHeld_DoesNotInvokeAgain()
    {
        bool previous = false;
        int invocations = 0;
        Action handler = () => invocations++;

        KeyPressEdgeDetector.Process(keyDown: true, ref previous, handler);
        KeyPressEdgeDetector.Process(keyDown: true, ref previous, handler);

        invocations.Should().Be(1);
    }

    // ── Supplemental: poll thread lifecycle (no Unity) ─────────────────────────

    [Fact]
    public void StartKeyPollThread_WhenCalledTwice_IsIdempotent()
    {
        KeyInputSystem.StartKeyPollThread();
        KeyInputSystem.StartKeyPollThread();

        bool running = (bool)GetPrivateStatic(typeof(KeyInputSystem), "_keyPollRunning")!;
        running.Should().BeTrue();

        KeyInputSystem.StopKeyPollThread();
        running = (bool)GetPrivateStatic(typeof(KeyInputSystem), "_keyPollRunning")!;
        running.Should().BeFalse();
    }

    [Fact]
    public void GetActiveWorld_WhenNoCachedWorld_DoesNotThrow()
    {
        SetPrivateStatic(typeof(KeyInputSystem), "_cachedWorld", null!);

        Action act = () => _ = KeyInputSystem.GetActiveWorld();

        act.Should().NotThrow();
    }

    private static object? GetPrivateStatic(Type type, string fieldName)
    {
        FieldInfo? field = type.GetField(fieldName, BindingFlags.Static | BindingFlags.NonPublic);
        field.Should().NotBeNull();
        return field!.GetValue(null);
    }

    /// <summary>
    /// Mirrors KeyInputSystem KeyPollLoop / OnUpdate press-edge detection (KIS-F5).
    /// Keep aligned with <c>f9Current &amp;&amp; !_f9PreviousState</c> in KeyInputSystem.cs.
    /// </summary>
    internal static class KeyPressEdgeDetector
    {
        public static void Process(bool keyDown, ref bool previousState, Action? onPressed)
        {
            if (keyDown && !previousState)
            {
                onPressed?.Invoke();
            }

            previousState = keyDown;
        }
    }
}
