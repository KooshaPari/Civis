// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

namespace DINOForge.SDK.UI.Native
{
    /// <summary>
    /// Hosts named native menu screens that piggy-back on vanilla DINO canvases
    /// (e.g. NativeMainMenuModMenu). Phase 2 implementation lives in Runtime/UI.
    /// </summary>
    public interface INativeMenuHost
    {
        /// <summary>Register a screen under <paramref name="menuId"/>.</summary>
        void RegisterScreen(string menuId, INativeMenuScreen screen);

        /// <summary>Drop the previously registered screen for <paramref name="menuId"/>.</summary>
        void UnregisterScreen(string menuId);

        /// <summary>Whether the host has at least one screen currently active.</summary>
        bool IsActive { get; }
    }

    /// <summary>
    /// A native menu screen — i.e. a screen drawn directly inside a vanilla
    /// DINO canvas, not on DFCanvas. Phase 2 will provide concrete impls.
    /// </summary>
    public interface INativeMenuScreen
    {
        /// <summary>Display title shown in vanilla chrome (back button label, etc.).</summary>
        string Title { get; }

        /// <summary>Called when the screen becomes visible.</summary>
        void OnEnter();

        /// <summary>Called when the screen is dismissed.</summary>
        void OnExit();
    }
}
