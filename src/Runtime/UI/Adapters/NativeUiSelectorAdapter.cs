// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 3 of #193 SDK split — fourth runtime adapter.

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Native;
using UnityEngine;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="INativeUiSelector"/> contract.
    /// Wraps <see cref="UiSelectorEngine"/> with hard scoping to vanilla canvases —
    /// any match rooted under <c>DFCanvas_Root</c> (the Extended UI surface) is excluded.
    /// </summary>
    /// <remarks>
    /// Phase 2 Dispatch 3: <see cref="UiSelectorEngine"/> is internal and does not expose a
    /// list-returning entry point; this adapter delegates to the public <c>Query</c> method
    /// for a single match. The DFCanvas exclusion is enforced post-match by walking the
    /// matched node's parent chain. <c>SelectMany</c> currently returns at most one element
    /// because <see cref="UiSelectorEngine"/> is index-disambiguating; a future patch will
    /// add a multi-result entry point on the engine.
    /// </remarks>
    public sealed class NativeUiSelectorAdapter : INativeUiSelector
    {
        private const string ExtendedUiRootName = "DFCanvas_Root";

        private static NativeUiSelectorAdapter? _instance;

        /// <summary>Singleton accessor — adapter holds no per-call state.</summary>
        public static NativeUiSelectorAdapter Instance => _instance ??= new NativeUiSelectorAdapter();

        private NativeUiSelectorAdapter() { }

        /// <inheritdoc />
        public NativeElementHandle? SelectOne(string selector)
        {
            if (selector is null) throw new ArgumentNullException(nameof(selector));
            return SelectOneCore(selector);
        }

        /// <inheritdoc />
        public IReadOnlyList<NativeElementHandle> SelectMany(string selector)
        {
            if (selector is null) throw new ArgumentNullException(nameof(selector));
            return SelectManyCore(selector);
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate UiSelectorEngine (which transitively pulls in
        // UnityEngine.UIModule) behind the null guards above. JIT defers loading
        // until a Core method is actually invoked.
        // ------------------------------------------------------------------ //

        private static NativeElementHandle? SelectOneCore(string selector)
        {
            var result = UiSelectorEngine.Query(selector);
            if (!result.Success || result.MatchedNode == null)
            {
                return null;
            }

            // UiActionResult exposes a UiNode, not the underlying Transform. We can't reach
            // through to the live Transform here without engine-level changes, so until a
            // multi-result + transform-returning entry point is added we wrap the snapshot
            // and rely on its Path to encode lineage. The DFCanvas filter therefore operates
            // on the path string. UiNode initializes Path/Name/Role to "" — never null.
            if (IsExtendedUiPath(result.MatchedNode.Path))
            {
                return null;
            }

            string role = string.IsNullOrEmpty(result.MatchedNode.Role) ? "node" : result.MatchedNode.Role;
            return new NativeElementHandle(
                result.MatchedNode.Name,
                role,
                result.MatchedNode);
        }

        private static IReadOnlyList<NativeElementHandle> SelectManyCore(string selector)
        {
            // UiSelectorEngine.Query returns a single best-match; without a list-returning
            // entry point we surface that one match (if not in DFCanvas) as a single-element
            // collection. This keeps the SDK contract honest while a future engine patch
            // adds true multi-result support.
            NativeElementHandle? one = SelectOneCore(selector);
            return one == null
                ? (IReadOnlyList<NativeElementHandle>)Array.Empty<NativeElementHandle>()
                : new[] { one };
        }

        private static bool IsExtendedUiPath(string path)
        {
            // Path format from UiTreeSnapshotBuilder: "root/Foo[0]/Bar[1]/...". The DFCanvas
            // root is named "DFCanvas_Root", so any path containing that segment is part of
            // the Extended UI surface and must be excluded from the native-only contract.
            return path.IndexOf(ExtendedUiRootName, StringComparison.Ordinal) >= 0;
        }
    }
}
