// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

namespace DINOForge.SDK.UI.Native
{
    /// <summary>
    /// Wraps a Harmony patch that pins a button's label against vanilla rewriting.
    /// Today: <c>UiGridHarmonyPatch.ModsButtonTextPatch</c>. Phase 2 lifts that
    /// patch into a concrete <see cref="INativeLabelGuard"/> implementation.
    /// </summary>
    public interface INativeLabelGuard
    {
        /// <summary>Pin <paramref name="button"/>'s label to <paramref name="text"/>.</summary>
        void PinLabel(NativeButtonHandle button, string text);

        /// <summary>Release a previously pinned label.</summary>
        void UnpinLabel(NativeButtonHandle button);
    }
}
