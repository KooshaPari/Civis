#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock of <see cref="IModSettingsHost"/> for unit tests. Records
/// every (modId, panel) pair registered. Pattern #125 seam double — keeps a
/// canonical stub even though the SDK-side interface is allowlisted as
/// observation-only, so downstream callers (e.g. F10 menu plumbing) can
/// assert registration ordering and dedupe semantics without instantiating
/// real Unity-bound hosts.
/// </summary>
public sealed class MockModSettingsHost : IModSettingsHost
{
    /// <summary>All registrations in call order. Allows duplicate detection by tests.</summary>
    public List<(string ModId, IModSettingsPanel Panel)> Registrations { get; } = new();

    /// <summary>Quick lookup: modId → most-recently-registered panel.</summary>
    public Dictionary<string, IModSettingsPanel> Panels { get; }
        = new(StringComparer.Ordinal);

    /// <summary>Convenience count of <see cref="RegisterSettingsPanel"/> invocations.</summary>
    public int RegisterCount => Registrations.Count;

    /// <inheritdoc />
    public void RegisterSettingsPanel(string modId, IModSettingsPanel panel)
    {
        Registrations.Add((modId, panel));
        Panels[modId] = panel;
    }
}
