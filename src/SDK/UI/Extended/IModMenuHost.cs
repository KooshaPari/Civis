// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.
// Phase 2 will move + re-namespace the existing Runtime/UI ModMenuPanel impl
// behind this interface.

namespace DINOForge.SDK.UI.Extended
{
    /// <summary>
    /// The F10 mod menu host. Lives on DFCanvas, NOT on a vanilla canvas.
    /// </summary>
    public interface IModMenuHost
    {
        /// <summary>Toggle visibility.</summary>
        void Toggle();

        /// <summary>Show the menu (idempotent).</summary>
        void Show();

        /// <summary>Hide the menu (idempotent).</summary>
        void Hide();

        /// <summary>Whether the menu is currently visible.</summary>
        bool IsVisible { get; }
    }
}
