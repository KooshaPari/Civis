#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Stub main-menu-native mod menu host. Full UGUI injection is deferred to M11.5 / WI-004a;
    /// <see cref="CanUseNativeScreen"/> remains false so <see cref="ContextualModMenuHost"/>
    /// routes to the overlay fallback.
    /// </summary>
    public sealed class NativeMainMenuModMenu : IModMenuHost
    {
        /// <summary>Whether the vanilla main-menu canvas can host the mod menu (not yet implemented).</summary>
        public bool CanUseNativeScreen => false;

        /// <inheritdoc />
        public Action? OnReloadRequested { get; set; }

        /// <inheritdoc />
        public Action<string, bool>? OnPackToggled { get; set; }

        /// <inheritdoc />
        public bool IsVisible { get; private set; }

        /// <inheritdoc />
        public void Show() => IsVisible = true;

        /// <inheritdoc />
        public void Hide() => IsVisible = false;

        /// <inheritdoc />
        public void Toggle() => IsVisible = !IsVisible;

        /// <inheritdoc />
        public void SetPacks(IEnumerable<PackDisplayInfo> packs)
        {
            // M11.5: bind pack list to native main-menu UI.
        }

        /// <inheritdoc />
        public void SetStatus(string message, int errorCount = 0)
        {
            // M11.5: bind status line to native main-menu UI.
        }
    }
}
