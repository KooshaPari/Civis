// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 7 of #193 SDK split — third extended-side runtime adapter (HUD-element renderer).

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Models;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="IHudElementRenderer"/> contract.
    /// Mounts pack-supplied <see cref="HudElementDefinition"/> instances under
    /// <c>DFCanvas_Root</c> using the existing <see cref="UiBuilder"/> primitives
    /// (the same factory that powers <see cref="HudStrip"/> / <see cref="ModMenuPanel"/>).
    /// </summary>
    /// <remarks>
    /// Dispatch 7 wires the registry-driven HUD pipeline that #194 populated but never
    /// rendered visually. The flow is:
    /// <code>
    ///   pack YAML → HudElementRegistry → IHudElementRenderer.Render → DFCanvas child UGUI.
    /// </code>
    ///
    /// Lifetime: process-wide singleton. <c>DFCanvas.BuildCanvas()</c> calls
    /// <see cref="SetCanvasRoot"/> once <c>DFCanvas_Root</c> is constructed; passing
    /// <c>null</c> detaches (used by tests + plugin teardown). Until a root is wired,
    /// <see cref="Render"/> records intent only and returns a handle whose inner reference
    /// is a sentinel <see cref="DeferredHudMount"/> — letting tests exercise the contract
    /// without spinning up Unity, and letting packs that load before DFCanvas finishes
    /// initialisation enqueue work that the runtime can drain on canvas-ready.
    ///
    /// Threading: pack code may call <see cref="Render"/> from background threads
    /// (HMR watcher). All Unity-touching work runs in <see cref="MountUgui"/> /
    /// <see cref="DestroyUgui"/>, declared <c>partial</c> here and implemented in
    /// <c>HudElementRendererAdapter.Unity.cs</c> which is excluded from CI builds —
    /// keeps unit tests free of <c>FileNotFoundException</c> for UnityEngine assemblies.
    ///
    /// Last-writer-wins on duplicate <see cref="HudElementDefinition.Id"/>: matches the
    /// registry semantics used by sibling hosts (<see cref="ModSettingsHostAdapter"/>,
    /// <c>HudElementRegistry</c>) and supports clean pack hot-reload.
    /// </remarks>
    public sealed partial class HudElementRendererAdapter : IHudElementRenderer
    {
        private static HudElementRendererAdapter? _instance;

        /// <summary>Singleton accessor — registrations are process-wide.</summary>
        public static HudElementRendererAdapter Instance => _instance ??= new HudElementRendererAdapter();

        private readonly object _lock = new object();
        private readonly Dictionary<string, ExtendedPanelHandle> _mounted =
            new Dictionary<string, ExtendedPanelHandle>(StringComparer.Ordinal);

        // Opaque holder; fully-qualified to keep UnityEngine.Transform off the public
        // surface so this file compiles in CI (no UnityEngine reference). Stays null
        // in CI / unit tests; DFCanvas wires it in game-installed builds.
        private object? _canvasRoot;

        private HudElementRendererAdapter() { }

        /// <summary>
        /// Wires the adapter to the live <c>DFCanvas_Root</c> Transform. Called by
        /// <see cref="DFCanvas"/> once the canvas hierarchy is built. Passing
        /// <c>null</c> detaches — used by tests + plugin teardown.
        /// </summary>
        /// <param name="canvasRoot">A <c>UnityEngine.Transform</c> for <c>DFCanvas_Root</c>, or null to detach.</param>
        public void SetCanvasRoot(object? canvasRoot)
        {
            lock (_lock)
            {
                _canvasRoot = canvasRoot;
            }
        }

        /// <inheritdoc />
        public ExtendedHandle Render(HudElementDefinition definition)
        {
            if (definition is null) throw new ArgumentNullException(nameof(definition));
            if (string.IsNullOrEmpty(definition.Id))
                throw new ArgumentException("HudElementDefinition.Id must not be empty", nameof(definition));

            return RenderCore(definition);
        }

        /// <inheritdoc />
        public void Unrender(ExtendedHandle handle)
        {
            if (handle is null) throw new ArgumentNullException(nameof(handle));
            UnrenderCore(handle);
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate UnityEngine touchpoints behind null guards
        // so JIT defers UnityEngine.UIModule loading until a Core method runs.
        // The CI build (GameInstalled=false) never enters the Unity branch
        // because _canvasRoot stays null — Plugin.cs / DFCanvas only call
        // SetCanvasRoot when game assemblies are loaded.
        // ------------------------------------------------------------------ //

        private ExtendedHandle RenderCore(HudElementDefinition definition)
        {
            ExtendedPanelHandle? evicted;
            ExtendedPanelHandle handle;

            lock (_lock)
            {
                _mounted.TryGetValue(definition.Id, out evicted);

                object inner = _canvasRoot != null
                    ? MountUgui(definition, _canvasRoot)
                    : (object)new DeferredHudMount(definition);

                handle = new ExtendedPanelHandle(definition.Id, inner);
                _mounted[definition.Id] = handle;
            }

            // Tear down the evicted mount outside the lock — DestroyUgui may run
            // engine callbacks we must not invoke under the adapter lock.
            if (evicted != null && !ReferenceEquals(evicted, handle))
            {
                TryDestroy(evicted);
            }

            return handle;
        }

        private void UnrenderCore(ExtendedHandle handle)
        {
            ExtendedPanelHandle? removed = null;
            lock (_lock)
            {
                if (_mounted.TryGetValue(handle.Id, out ExtendedPanelHandle? existing)
                    && ReferenceEquals(existing, handle))
                {
                    removed = existing;
                    _mounted.Remove(handle.Id);
                }
            }

            if (removed != null)
            {
                TryDestroy(removed);
            }
        }

        private static void TryDestroy(ExtendedPanelHandle handle)
        {
            try
            {
                DestroyUgui(handle);
            }
            catch
            {
                // Pack-supplied teardown may raise; never propagate so a single bad
                // element cannot block subsequent unrender / hot-reload work.
            }
        }

        // Implemented in HudElementRendererAdapter.Unity.cs (game-installed builds only).
        // CI builds use the partial-method fallbacks defined further down so the public
        // surface compiles without UnityEngine references.
        static partial void MountUguiImpl(HudElementDefinition definition, object canvasRootObj, ref object? result);
        static partial void DestroyUguiImpl(ExtendedPanelHandle handle);

        private static object MountUgui(HudElementDefinition definition, object canvasRootObj)
        {
            object? result = null;
            MountUguiImpl(definition, canvasRootObj, ref result);
            return result ?? new DeferredHudMount(definition);
        }

        private static void DestroyUgui(ExtendedPanelHandle handle)
        {
            DestroyUguiImpl(handle);
        }

        /// <summary>
        /// Sentinel inner reference used when <see cref="Render"/> is called before
        /// <see cref="SetCanvasRoot"/>. Lets unit tests exercise the renderer contract
        /// without UnityEngine and lets packs queue HUD elements that the runtime can
        /// drain on canvas-ready (currently a TODO; see remarks).
        /// </summary>
        internal sealed class DeferredHudMount
        {
            /// <summary>The pending HUD-element definition.</summary>
            public HudElementDefinition Definition { get; }

            /// <summary>Construct a deferred mount sentinel.</summary>
            public DeferredHudMount(HudElementDefinition definition)
            {
                Definition = definition ?? throw new ArgumentNullException(nameof(definition));
            }
        }
    }
}
