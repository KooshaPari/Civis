#nullable enable
using System.Collections.Generic;
using DINOForge.SDK.UI.Bridge;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Native;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock of <see cref="IModButtonInjector"/> for unit tests.
/// Records calls to <see cref="InjectModsButton"/> and <see cref="SetCanvasRoot"/>
/// without touching Unity types. Pattern #125 orphan-mock seam — keeps the UI
/// bridge testable in isolation so future implementations (beyond
/// <c>ModButtonInjectorAdapter</c>) inherit a canonical test double.
/// </summary>
public sealed class MockModButtonInjector : IModButtonInjector
{
    /// <summary>Records every (canvas, target) pair passed to <see cref="InjectModsButton"/>.</summary>
    public List<(NativeCanvasHandle Canvas, IModMenuHost Target)> InjectCalls { get; } = new();

    /// <summary>Records every <see cref="SetCanvasRoot"/> argument, including nulls (detach signals).</summary>
    public List<object?> SetCanvasRootCalls { get; } = new();

    /// <summary>Optional override the next call to <see cref="InjectModsButton"/> returns.</summary>
    public NativeButtonHandle? NextHandle { get; set; }

    /// <summary>Convenience count: number of <see cref="InjectModsButton"/> invocations.</summary>
    public int InjectModsButtonCount => InjectCalls.Count;

    /// <summary>Convenience count: number of <see cref="SetCanvasRoot"/> invocations.</summary>
    public int SetCanvasRootCount => SetCanvasRootCalls.Count;

    /// <inheritdoc />
    public NativeButtonHandle InjectModsButton(NativeCanvasHandle canvas, IModMenuHost target)
    {
        InjectCalls.Add((canvas, target));
        // Construct via internal ctor (InternalsVisibleTo("DINOForge.Tests") in SDK).
        return NextHandle ?? new NativeButtonHandle("mock-mods-button", "Mods", new object());
    }

    /// <inheritdoc />
    public void SetCanvasRoot(object? canvasRoot)
    {
        SetCanvasRootCalls.Add(canvasRoot);
    }
}
