// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 8 of #193 SDK split — extended-side runtime adapter (mod canvas).

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="IModCanvas"/> contract. Records
    /// pack-supplied <see cref="PanelSpec"/> / <see cref="LabelSpec"/> / <see cref="ButtonSpec"/>
    /// requests destined for the mod-owned overlay surface (<c>DFCanvas_Root</c>), distinct from
    /// any vanilla DINO canvas.
    /// </summary>
    /// <remarks>
    /// Intent-only until Plugin.cs / DFCanvas wiring binds the inner GameObjects: today
    /// <see cref="CreatePanel"/> / <see cref="CreateLabel"/> / <see cref="CreateButton"/> validate
    /// arguments per the SDK contract and stash an opaque marker reference on each handle. When the
    /// runtime renderer lands (companion to the <see cref="HudElementRendererAdapter"/> wiring), it
    /// will pull from <see cref="GetCreatedHandles"/> and mount real UGUI widgets under
    /// <c>DFCanvas_Root</c> using the existing <c>UiBuilder</c> primitives.
    ///
    /// The handle registry is process-wide (singleton), keyed by <c>handle.Id</c>. Last-writer-wins
    /// on duplicate ids: matches the registry semantics used by sibling extended hosts
    /// (<see cref="ModSettingsHostAdapter"/>, <see cref="HudElementRendererAdapter"/>) and lets
    /// pack hot-reload swap a panel / label / button cleanly without an explicit destroy call.
    ///
    /// Thread-safety: all registry mutations happen under <c>_lock</c>. The evicted handle is
    /// disposed (if it implements <see cref="IDisposable"/>) outside the lock — defensive against
    /// future handle types that hold engine resources, even though the current
    /// <see cref="ExtendedPanelHandle"/> / <see cref="ExtendedLabelHandle"/> /
    /// <see cref="ExtendedButtonHandle"/> types are inert. Pack code may invoke from background
    /// threads (HMR signal watcher); engine teardown work, when wired, must marshal to the Unity
    /// main thread separately.
    ///
    /// The <c>ExtendedXxxHandle</c> ctors are <c>internal sealed</c>; <c>DINOForge.SDK.csproj</c>
    /// already grants <c>[InternalsVisibleTo("DINOForge.Runtime")]</c> (Phase 2 of #193) so this
    /// adapter constructs them directly — no reflection needed, matching
    /// <see cref="HudElementRendererAdapter"/>.
    /// </remarks>
    public sealed class ModCanvasAdapter : IModCanvas
    {
        private static ModCanvasAdapter? _instance;

        /// <summary>Singleton accessor — the canvas registry is process-wide.</summary>
        public static ModCanvasAdapter Instance => _instance ??= new ModCanvasAdapter();

        private readonly object _lock = new object();
        private readonly Dictionary<string, ExtendedHandle> _handles =
            new Dictionary<string, ExtendedHandle>(StringComparer.Ordinal);

        // Opaque holder; fully-qualified to keep UnityEngine.Transform off the public
        // surface so this file compiles in CI (no UnityEngine reference). Stays null
        // in CI / unit tests; DFCanvas wires it in game-installed builds. The future
        // canvas renderer (M11.5+) will use this as the parent transform when mounting
        // panels / labels / buttons that the inert handles describe today.
        private object? _canvasRoot;

        private ModCanvasAdapter() { }

        /// <summary>
        /// Wires the adapter to the live <c>DFCanvas_Root</c> Transform. Called by
        /// <see cref="DFCanvas"/> once the canvas hierarchy is built. Passing
        /// <c>null</c> detaches — used by tests + plugin teardown.
        /// </summary>
        /// <remarks>
        /// Mirrors <see cref="HudElementRendererAdapter.SetCanvasRoot"/> so DFCanvas can wire
        /// all extended adapters in one pass. The pending-handle queue (see
        /// <see cref="GetCreatedHandles"/>) becomes drainable when the renderer lands; until
        /// then the canvas root is recorded but unused.
        /// </remarks>
        /// <param name="canvasRoot">A <c>UnityEngine.Transform</c> for <c>DFCanvas_Root</c>, or null to detach.</param>
        public void SetCanvasRoot(object? canvasRoot)
        {
            lock (_lock)
            {
                _canvasRoot = canvasRoot;
            }
        }

        /// <summary>Returns the wired canvas root for diagnostics/tests; null until DFCanvas wires it.</summary>
        public object? GetCanvasRoot()
        {
            lock (_lock) { return _canvasRoot; }
        }

        /// <inheritdoc />
        public ExtendedPanelHandle CreatePanel(PanelSpec spec)
        {
            if (spec is null) throw new ArgumentNullException(nameof(spec));
            if (string.IsNullOrEmpty(spec.Id))
                throw new ArgumentException("PanelSpec.Id must not be empty", nameof(spec));

            return CreatePanelCore(spec);
        }

        /// <inheritdoc />
        public ExtendedLabelHandle CreateLabel(LabelSpec spec)
        {
            if (spec is null) throw new ArgumentNullException(nameof(spec));
            if (string.IsNullOrEmpty(spec.Id))
                throw new ArgumentException("LabelSpec.Id must not be empty", nameof(spec));

            return CreateLabelCore(spec);
        }

        /// <inheritdoc />
        public ExtendedButtonHandle CreateButton(ButtonSpec spec)
        {
            if (spec is null) throw new ArgumentNullException(nameof(spec));
            if (string.IsNullOrEmpty(spec.Id))
                throw new ArgumentException("ButtonSpec.Id must not be empty", nameof(spec));
            if (spec.OnClick is null)
                throw new ArgumentException("ButtonSpec.OnClick must not be null", nameof(spec));

            return CreateButtonCore(spec);
        }

        /// <inheritdoc />
        public void Destroy(ExtendedHandle handle)
        {
            if (handle is null) throw new ArgumentNullException(nameof(handle));
            if (string.IsNullOrEmpty(handle.Id))
                throw new ArgumentException("ExtendedHandle.Id must not be empty", nameof(handle));

            DestroyCore(handle);
        }

        /// <summary>
        /// Returns a snapshot of currently registered handles keyed by <c>handle.Id</c>. Used by
        /// the runtime renderer (when wired) to enumerate pending mounts and by tests to verify
        /// registration behavior without spinning up Unity.
        /// </summary>
        public IReadOnlyDictionary<string, ExtendedHandle> GetCreatedHandles()
        {
            lock (_lock)
            {
                // Copy under lock so callers can iterate without holding the adapter lock.
                return new Dictionary<string, ExtendedHandle>(_handles, StringComparer.Ordinal);
            }
        }

        // ------------------------------------------------------------------ //
        // *Core methods mirror the split used by sibling adapters: keeps the
        // public surface argument-validation-only so unit tests run without
        // loading Unity (or BepInEx) assemblies, and leaves a clear seam for
        // the runtime renderer to hook into when the DFCanvas wiring lands.
        // ------------------------------------------------------------------ //

        private ExtendedPanelHandle CreatePanelCore(PanelSpec spec)
        {
            // Inner is an opaque marker today; the runtime renderer will replace this with a
            // real UnityEngine.GameObject reference once SetCanvasRoot-style wiring lands.
            var handle = new ExtendedPanelHandle(spec.Id, new object());
            RegisterHandle(handle);
            return handle;
        }

        private ExtendedLabelHandle CreateLabelCore(LabelSpec spec)
        {
            var handle = new ExtendedLabelHandle(spec.Id, new object());
            RegisterHandle(handle);
            return handle;
        }

        private ExtendedButtonHandle CreateButtonCore(ButtonSpec spec)
        {
            var handle = new ExtendedButtonHandle(spec.Id, new object());
            RegisterHandle(handle);
            return handle;
        }

        private void RegisterHandle(ExtendedHandle handle)
        {
            ExtendedHandle? evicted;
            lock (_lock)
            {
                _handles.TryGetValue(handle.Id, out evicted);
                _handles[handle.Id] = handle;
            }

            // Dispose the evicted handle outside the lock — Dispose() may run engine code
            // (when handles eventually carry Unity references) that we must not invoke under
            // the adapter lock. Today's handle types are inert; the IDisposable path is
            // defensive against future shapes.
            if (evicted != null && !ReferenceEquals(evicted, handle))
            {
                TryDispose(evicted);
            }
        }

        private void DestroyCore(ExtendedHandle handle)
        {
            ExtendedHandle? removed = null;
            lock (_lock)
            {
                if (_handles.TryGetValue(handle.Id, out ExtendedHandle? existing)
                    && ReferenceEquals(existing, handle))
                {
                    removed = existing;
                    _handles.Remove(handle.Id);
                }
            }

            if (removed != null)
            {
                TryDispose(removed);
            }
        }

        private static void TryDispose(ExtendedHandle handle)
        {
            if (handle is IDisposable disposable)
            {
                try
                {
                    disposable.Dispose();
                }
                catch
                {
                    // Pack-supplied / engine teardown may raise; never propagate so a single bad
                    // handle cannot block subsequent destroy / hot-reload work.
                }
            }
        }
    }
}
