// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.
// Counterpart to Domains/UI's HudElementRegistry: the registry holds
// definitions, this interface renders them.

using DINOForge.SDK.UI.Models;

namespace DINOForge.SDK.UI.Extended
{
    /// <summary>
    /// Bind a <see cref="HudElementDefinition"/> to a live UGUI element under
    /// <c>DFCanvas_Root</c>. Phase 2 implementation drives the registry-render
    /// pipeline that #194 wired but did not populate visually.
    /// </summary>
    public interface IHudElementRenderer
    {
        /// <summary>Bind a HudElementDefinition to a live UGUI element under DFCanvas_Root.</summary>
        ExtendedHandle Render(HudElementDefinition definition);

        /// <summary>Tear down a previously rendered element.</summary>
        void Unrender(ExtendedHandle handle);

        /// <summary>
        /// Wires the renderer to the live <c>DFCanvas_Root</c> Transform (boxed as
        /// <see cref="object"/> to keep the SDK free of a UnityEngine reference). Pass
        /// <c>null</c> to detach (used by tests + plugin teardown). Implementations may
        /// drain a deferred-mount queue when a non-null root is wired so packs that
        /// loaded before the canvas existed still render. Task #234 promotes this from
        /// adapter-only API to the SDK contract so DFCanvas can wire through the
        /// interface rather than the concrete adapter singleton.
        /// </summary>
        /// <param name="canvasRoot">A <c>UnityEngine.Transform</c> for <c>DFCanvas_Root</c>, or <c>null</c> to detach.</param>
        void SetCanvasRoot(object? canvasRoot);
    }
}
