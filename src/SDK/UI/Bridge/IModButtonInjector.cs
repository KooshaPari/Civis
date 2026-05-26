// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.
// THE SEAM: this is the only place where Native and Extended UI cross.

using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Native;

namespace DINOForge.SDK.UI.Bridge
{
    /// <summary>
    /// The seam — only place where Native and Extended UI cross. Wires a
    /// vanilla DINO button (e.g. the main-menu "Mods" entry) to invoke a
    /// mod-owned <see cref="IModMenuHost"/> on click.
    ///
    /// Phase 2 will lift the existing <c>NativeMenuInjector</c> behaviour
    /// behind this interface.
    /// </summary>
    public interface IModButtonInjector
    {
        /// <summary>
        /// Find vanilla button via <see cref="INativeButtonAdapter"/>; rewire its
        /// onClick to invoke an <see cref="IModMenuHost"/>.
        /// </summary>
        NativeButtonHandle InjectModsButton(NativeCanvasHandle canvas, IModMenuHost target);

        /// <summary>
        /// Wires the injector to the live <c>DFCanvas_Root</c> Transform (boxed as
        /// <see cref="object"/> to keep the SDK free of a UnityEngine reference). The
        /// seam acts on a vanilla <see cref="NativeCanvasHandle"/> per call to
        /// <see cref="InjectModsButton"/>; this canvas-root pointer is held so the
        /// future <c>NativeMenuInjector</c> wire can locate the mod-menu panel without
        /// re-walking the scene graph. Pass <c>null</c> to detach. Task #234 promotes
        /// this from adapter-only API to the SDK contract so DFCanvas can wire through
        /// the interface rather than the concrete adapter singleton.
        /// </summary>
        /// <param name="canvasRoot">A <c>UnityEngine.Transform</c> for <c>DFCanvas_Root</c>, or <c>null</c> to detach.</param>
        void SetCanvasRoot(object? canvasRoot);
    }
}
