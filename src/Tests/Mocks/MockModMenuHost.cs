#nullable enable
using DINOForge.SDK.UI.Extended;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock of <see cref="IModMenuHost"/> for unit tests. Behavior-verifying
/// double: tracks Toggle/Show/Hide counts and exposes <see cref="IsVisible"/>
/// for assertions. Note: <c>IModMenuHost</c> is allowlisted as observation-only
/// in <c>docs/qa/orphan-interface-mocks-allowlist.txt</c> at the SDK level, but
/// downstream tests routinely need a fake for plumbing pieces like
/// <see cref="MockModButtonInjector"/>. Providing a canonical one here avoids
/// per-test ad-hoc handrolls.
/// </summary>
public sealed class MockModMenuHost : IModMenuHost
{
    /// <summary>Number of <see cref="Toggle"/> calls.</summary>
    public int ToggleCount { get; private set; }

    /// <summary>Number of <see cref="Show"/> calls.</summary>
    public int ShowCount { get; private set; }

    /// <summary>Number of <see cref="Hide"/> calls.</summary>
    public int HideCount { get; private set; }

    /// <inheritdoc />
    public bool IsVisible { get; private set; }

    /// <inheritdoc />
    public void Toggle()
    {
        ToggleCount++;
        IsVisible = !IsVisible;
    }

    /// <inheritdoc />
    public void Show()
    {
        ShowCount++;
        IsVisible = true;
    }

    /// <inheritdoc />
    public void Hide()
    {
        HideCount++;
        IsVisible = false;
    }
}
