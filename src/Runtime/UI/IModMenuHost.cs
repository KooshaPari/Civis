#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Abstracts the runtime mod-menu host so IMGUI, overlay UGUI, and native-hosted
    /// menu implementations can all be wired into the same ModPlatform callbacks.
    /// </summary>
    public interface IModMenuHost
    {
        /// <summary>Callback invoked when the user requests a pack reload.</summary>
        Action? OnReloadRequested { get; set; }

        /// <summary>Callback invoked when the user toggles a pack (packId, isEnabled).</summary>
        Action<string, bool>? OnPackToggled { get; set; }

        /// <summary>Whether the menu is currently visible.</summary>
        bool IsVisible { get; }

        /// <summary>Shows the menu.</summary>
        void Show();

        /// <summary>Hides the menu.</summary>
        void Hide();

        /// <summary>Toggles the menu visibility.</summary>
        void Toggle();

        /// <summary>Replaces the pack list displayed by the host.</summary>
        void SetPacks(IEnumerable<PackDisplayInfo> packs);

        /// <summary>Updates the status line displayed by the host.</summary>
        void SetStatus(string message, int errorCount = 0);
    }
}
