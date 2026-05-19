// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 1 of #193 SDK split — first runtime adapter.

#nullable enable
using DINOForge.SDK.UI.Native;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="INativeLabelGuard"/> contract.
    /// Wraps the existing <c>UiGridHarmonyPatch.ModsButtonTextPatch</c> which uses Harmony
    /// to defend a button's label against vanilla UiGrid rewriting.
    /// </summary>
    /// <remarks>
    /// Phase 2 Dispatch 1: validates the SDK→Runtime adapter pattern with the smallest
    /// possible migration. Subsequent dispatches (2–5) will follow the same shape for the
    /// other 4 native interfaces, then dispatches (6–10) cover the 5 extended UI interfaces.
    ///
    /// The current <c>ModsButtonTextPatch</c> is install-once via Harmony prefixes on
    /// <c>TMP_Text.SetText</c> + <c>UnityEngine.UI.Text.set_text</c>, with a single hardcoded
    /// substitution target (<c>NativeMenuInjector.RepurposedModsButtonGoName</c>). It does NOT
    /// support per-button pinning today. Phase 2b will refactor the patch for multi-label
    /// support; until then this adapter records the pin intent without altering the active
    /// Harmony patch (which is already installed at <c>Plugin.cs</c> startup).
    /// </remarks>
    public sealed class NativeLabelGuardAdapter : INativeLabelGuard
    {
        private static NativeLabelGuardAdapter? _instance;

        /// <summary>Singleton accessor — Harmony patches are install-once by design.</summary>
        public static NativeLabelGuardAdapter Instance => _instance ??= new NativeLabelGuardAdapter();

        private NativeLabelGuardAdapter() { }

        /// <inheritdoc />
        public void PinLabel(NativeButtonHandle button, string text)
        {
            if (button is null) throw new System.ArgumentNullException(nameof(button));
            if (string.IsNullOrEmpty(text)) throw new System.ArgumentException("text must not be empty", nameof(text));

            // PHASE2-TODO: when UiGridHarmonyPatch is refactored for multi-label support,
            // route the (button, text) pair into a per-button pin map. For now the existing
            // singleton patch is install-once and substitutes "OPTIONS" → "Mods" on the
            // repurposed button identified by NativeMenuInjector.RepurposedModsButtonGoName.
            // The pin call is therefore a contract-level acknowledgement; the actual
            // substitution is governed by the install-time Harmony hook.
        }

        /// <inheritdoc />
        public void UnpinLabel(NativeButtonHandle button)
        {
            if (button is null) throw new System.ArgumentNullException(nameof(button));
            // PHASE2-TODO: ModsButtonTextPatch is currently single-label, install-once.
            // Unpinning is a no-op until the multi-label refactor lands.
        }
    }
}
