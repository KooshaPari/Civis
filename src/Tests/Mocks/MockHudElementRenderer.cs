#nullable enable
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Models;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock of <see cref="IHudElementRenderer"/> for unit tests. Stub for
/// Pattern #125 — provides a canonical seam-double now that
/// <c>HudElementRendererAdapter</c> is the sole impl but more (e.g. headless,
/// surface-swap) will land. Tracks render/unrender pairs and canvas-root wiring
/// without depending on UnityEngine.
/// </summary>
public sealed class MockHudElementRenderer : IHudElementRenderer
{
    private int _nextHandleId;

    /// <summary>Every definition handed to <see cref="Render"/>, in call order.</summary>
    public List<HudElementDefinition> Rendered { get; } = new();

    /// <summary>Every handle handed to <see cref="Unrender"/>, in call order.</summary>
    public List<ExtendedHandle> Unrendered { get; } = new();

    /// <summary>Live handles currently considered mounted (Render - Unrender).</summary>
    public List<ExtendedHandle> ActiveHandles { get; } = new();

    /// <summary>Records every <see cref="SetCanvasRoot"/> argument, including nulls.</summary>
    public List<object?> SetCanvasRootCalls { get; } = new();

    /// <summary>Convenience count of <see cref="Render"/> invocations.</summary>
    public int RenderCount => Rendered.Count;

    /// <summary>Convenience count of <see cref="Unrender"/> invocations.</summary>
    public int UnrenderCount => Unrendered.Count;

    /// <inheritdoc />
    public ExtendedHandle Render(HudElementDefinition definition)
    {
        Rendered.Add(definition);
        var handle = new MockExtendedHandle($"mock-hud-{_nextHandleId++}", definition);
        ActiveHandles.Add(handle);
        return handle;
    }

    /// <inheritdoc />
    public void Unrender(ExtendedHandle handle)
    {
        Unrendered.Add(handle);
        ActiveHandles.Remove(handle);
    }

    /// <inheritdoc />
    public void SetCanvasRoot(object? canvasRoot)
    {
        SetCanvasRootCalls.Add(canvasRoot);
    }

    /// <summary>Test-only ExtendedHandle that carries an arbitrary inner payload.</summary>
    public sealed class MockExtendedHandle : ExtendedHandle
    {
        internal MockExtendedHandle(string id, object inner) : base(id, inner) { }
    }
}
