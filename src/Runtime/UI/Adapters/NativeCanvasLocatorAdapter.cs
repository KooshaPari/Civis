// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 2 of #193 SDK split — second runtime adapter.

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Native;
using UnityEngine;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="INativeCanvasLocator"/> contract.
    /// Wraps <c>Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()</c> filtered to active canvases.
    /// </summary>
    /// <remarks>
    /// Phase 2 Dispatch 2: matches the <see cref="NativeLabelGuardAdapter"/> singleton pattern.
    /// "Active" means <c>canvas.gameObject.activeInHierarchy &amp;&amp; canvas.enabled</c>.
    /// SDK consumers receive opaque <see cref="NativeCanvasHandle"/> instances; the underlying
    /// <c>UnityEngine.Canvas</c> is reachable only via the internal <c>Inner</c> property.
    /// </remarks>
    public sealed class NativeCanvasLocatorAdapter : INativeCanvasLocator
    {
        private static NativeCanvasLocatorAdapter? _instance;

        /// <summary>Singleton accessor — adapter holds no per-call state.</summary>
        public static NativeCanvasLocatorAdapter Instance => _instance ??= new NativeCanvasLocatorAdapter();

        private NativeCanvasLocatorAdapter() { }

        /// <inheritdoc />
        public NativeCanvasHandle? FindCanvas(string nameHint)
        {
            if (nameHint is null) throw new ArgumentNullException(nameof(nameHint));
            return FindCanvasCore(nameHint);
        }

        /// <inheritdoc />
        public IReadOnlyList<NativeCanvasHandle> EnumerateCanvases()
        {
            return EnumerateCanvasesCore();
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate UnityEngine.UIModule references behind the null
        // guards above. JIT defers loading UnityEngine.UIModule until a Core
        // method is actually invoked — so null-arg unit tests don't trigger
        // FileNotFoundException for the Unity assembly.
        // ------------------------------------------------------------------ //

        private static NativeCanvasHandle? FindCanvasCore(string nameHint)
        {
            foreach (Canvas canvas in Resources.FindObjectsOfTypeAll<Canvas>())
            {
                if (!IsActive(canvas)) continue;
                if (string.Equals(canvas.name, nameHint, StringComparison.OrdinalIgnoreCase))
                {
                    return Wrap(canvas);
                }
            }

            return null;
        }

        private static IReadOnlyList<NativeCanvasHandle> EnumerateCanvasesCore()
        {
            List<NativeCanvasHandle> result = new List<NativeCanvasHandle>();
            foreach (Canvas canvas in Resources.FindObjectsOfTypeAll<Canvas>())
            {
                if (!IsActive(canvas)) continue;
                result.Add(Wrap(canvas));
            }
            return result;
        }

        private static bool IsActive(Canvas? canvas)
        {
            return canvas != null
                && canvas.gameObject != null
                && canvas.gameObject.activeInHierarchy
                && canvas.enabled;
        }

        private static NativeCanvasHandle Wrap(Canvas canvas)
        {
            return new NativeCanvasHandle(canvas.name, IsActive(canvas), canvas);
        }
    }
}
