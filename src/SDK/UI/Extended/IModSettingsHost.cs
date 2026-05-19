// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.
// Phase 2 will move + re-namespace the existing Runtime/UI settings panel
// behind this interface.

namespace DINOForge.SDK.UI.Extended
{
    /// <summary>
    /// Hosts per-mod settings panels under the F10 mod menu chrome.
    /// </summary>
    public interface IModSettingsHost
    {
        /// <summary>Register a settings panel for <paramref name="modId"/>.</summary>
        void RegisterSettingsPanel(string modId, IModSettingsPanel panel);
    }

    /// <summary>
    /// A pluggable settings page rendered inside the F10 mod menu.
    /// Concrete impls supplied by mods.
    /// </summary>
    public interface IModSettingsPanel
    {
        /// <summary>Display title of the panel (sidebar label).</summary>
        string Title { get; }

        /// <summary>Build the panel UI under <paramref name="canvas"/>.</summary>
        void Build(IModCanvas canvas);

        /// <summary>Tear down anything created by <see cref="Build"/>.</summary>
        void Dispose();
    }
}
