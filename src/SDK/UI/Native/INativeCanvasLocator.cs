// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

using System.Collections.Generic;

namespace DINOForge.SDK.UI.Native
{
    /// <summary>
    /// Locates vanilla DINO canvases by name hint. Never instantiates new GameObjects.
    /// Phase 2 implementation will wrap <c>UnityEngine.Canvas</c> + <c>Resources.FindObjectsOfTypeAll</c>.
    /// </summary>
    public interface INativeCanvasLocator
    {
        /// <summary>Returns a handle to the named canvas if found and active, else null.</summary>
        NativeCanvasHandle? FindCanvas(string nameHint);

        /// <summary>Returns all currently-active vanilla canvases (snapshot).</summary>
        IReadOnlyList<NativeCanvasHandle> EnumerateCanvases();
    }

    /// <summary>
    /// Opaque handle to a vanilla canvas. Implementations wrap <c>UnityEngine.Canvas</c>.
    /// SDK consumers do not poke inside <see cref="Inner"/>; runtime adapters cast it.
    /// </summary>
    public sealed class NativeCanvasHandle
    {
        /// <summary>Canvas GameObject name as exposed by the vanilla scene.</summary>
        public string Name { get; }

        /// <summary>Whether the underlying canvas is currently enabled and in an active scene.</summary>
        public bool IsActive { get; }

        /// <summary>Implementation-specific opaque reference. SDK consumers do not unwrap.</summary>
        internal object Inner { get; }

        internal NativeCanvasHandle(string name, bool isActive, object inner)
        {
            Name = name;
            IsActive = isActive;
            Inner = inner;
        }
    }
}
