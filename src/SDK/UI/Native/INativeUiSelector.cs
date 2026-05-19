// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

using System.Collections.Generic;

namespace DINOForge.SDK.UI.Native
{
    /// <summary>
    /// SCOPED selector engine for native UI only. Selects within vanilla
    /// canvases and explicitly excludes the DFCanvas subtree (which is the
    /// Extended UI surface). Phase 2 implementation wraps UiSelectorEngine.
    /// </summary>
    public interface INativeUiSelector
    {
        /// <summary>Select within vanilla canvases only. Excludes DFCanvas subtree.</summary>
        NativeElementHandle? SelectOne(string selector);

        /// <summary>Select all matches within vanilla canvases. Excludes DFCanvas subtree.</summary>
        IReadOnlyList<NativeElementHandle> SelectMany(string selector);
    }

    /// <summary>
    /// Opaque handle to any native UI element (canvas-rooted GameObject) found
    /// by an <see cref="INativeUiSelector"/>. Generic counterpart to
    /// <see cref="NativeButtonHandle"/> / <see cref="NativeCanvasHandle"/>.
    /// </summary>
    public sealed class NativeElementHandle
    {
        /// <summary>Element GameObject name.</summary>
        public string Name { get; }

        /// <summary>Best-effort element kind hint, e.g. "Button" / "Text" / "Panel".</summary>
        public string Kind { get; }

        /// <summary>Implementation-specific opaque reference.</summary>
        internal object Inner { get; }

        internal NativeElementHandle(string name, string kind, object inner)
        {
            Name = name;
            Kind = kind;
            Inner = inner;
        }
    }
}
