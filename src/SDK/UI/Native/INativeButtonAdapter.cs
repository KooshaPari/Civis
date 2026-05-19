// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

using System;

namespace DINOForge.SDK.UI.Native
{
    /// <summary>
    /// Find / clone / relabel vanilla DINO buttons. Phase 2 implementation will
    /// wrap <c>UnityEngine.UI.Button</c> + <c>TMPro.TMP_Text</c>.
    /// </summary>
    public interface INativeButtonAdapter
    {
        /// <summary>
        /// Find a button by display text within a canvas. Case-sensitive Ordinal compare.
        /// </summary>
        NativeButtonHandle? FindButtonByText(NativeCanvasHandle canvas, string text);

        /// <summary>
        /// Clone an existing vanilla button, relabel, rewire onClick.
        /// </summary>
        NativeButtonHandle Clone(NativeButtonHandle source, string newLabel, Action onClick);

        /// <summary>
        /// Set the button's display text. Handles both UGUI Text and TMP_Text.
        /// </summary>
        void SetLabel(NativeButtonHandle button, string text);
    }

    /// <summary>
    /// Opaque handle to a vanilla button. Implementations wrap UnityEngine.UI.Button.
    /// </summary>
    public sealed class NativeButtonHandle
    {
        /// <summary>Button GameObject name in the vanilla scene.</summary>
        public string Name { get; }

        /// <summary>Current display label (best-effort snapshot at handle creation).</summary>
        public string Label { get; }

        /// <summary>Implementation-specific opaque reference.</summary>
        internal object Inner { get; }

        internal NativeButtonHandle(string name, string label, object inner)
        {
            Name = name;
            Label = label;
            Inner = inner;
        }
    }
}
