// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 6 of #193 SDK split — first extended-side runtime adapter (mod-menu).

#nullable enable
using System;
using ExtendedIModMenuHost = DINOForge.SDK.UI.Extended.IModMenuHost;
using RuntimeIModMenuHost = DINOForge.Runtime.UI.IModMenuHost;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="ExtendedIModMenuHost"/> contract.
    /// Bridges extended packs (which only see <see cref="DINOForge.SDK.UI.Extended.IModMenuHost"/>)
    /// to the existing runtime mod-menu surface — namely <see cref="ContextualModMenuHost"/>,
    /// the routing layer between the IMGUI overlay (<see cref="ModMenuOverlay"/>) and the
    /// upcoming native main-menu host.
    /// </summary>
    /// <remarks>
    /// This adapter is the inverse of <see cref="NativeMenuHostAdapter"/>: instead of exposing
    /// a vanilla-canvas-bound surface, it lets pack code drive the DFCanvas-bound mod menu
    /// without taking a hard reference on the richer Runtime <see cref="RuntimeIModMenuHost"/>
    /// (which carries pack-list / status / callback plumbing not appropriate for the extended SDK).
    ///
    /// Lifetime: the adapter is a process-wide singleton. Plugin.cs calls <see cref="SetTarget"/>
    /// during runtime bootstrap once the real menu host is constructed (DFCanvas.Start). Until
    /// then — and during unit tests — the adapter records intent only; Show/Hide/Toggle become
    /// idempotent no-ops and <see cref="IsVisible"/> tracks the last requested state. This keeps
    /// the SDK contract honest without forcing tests to spin up Unity.
    ///
    /// Threading: ContextualModMenuHost expects calls on the Unity main thread, but mod packs
    /// may invoke from background threads (HMR signal watcher, F10 input). Callers responsible
    /// for thread affinity — adapter does not marshal.
    /// </remarks>
    public sealed class ModMenuHostAdapter : ExtendedIModMenuHost
    {
        private static ModMenuHostAdapter? _instance;

        /// <summary>Singleton accessor — registrations are process-wide.</summary>
        public static ModMenuHostAdapter Instance => _instance ??= new ModMenuHostAdapter();

        private readonly object _lock = new object();
        private RuntimeIModMenuHost? _target;
        private bool _intendedVisible;

        // Opaque holder; fully-qualified to keep UnityEngine.Transform off the public
        // surface so this file compiles in CI (no UnityEngine reference). Stays null
        // in CI / unit tests; DFCanvas wires it in game-installed builds.
        private object? _canvasRoot;

        private ModMenuHostAdapter() { }

        /// <summary>
        /// Wires the adapter to the live runtime menu host. Called by Plugin.cs once
        /// <see cref="ContextualModMenuHost"/> (or its IMGUI fallback) is constructed.
        /// Passing <c>null</c> detaches — used by tests + plugin teardown.
        /// </summary>
        public void SetTarget(RuntimeIModMenuHost? target)
        {
            lock (_lock)
            {
                _target = target;
            }
        }

        /// <summary>
        /// Wires the adapter to the live <c>DFCanvas_Root</c> Transform. Called by
        /// <see cref="DFCanvas"/> once the canvas hierarchy is built. Passing
        /// <c>null</c> detaches — used by tests + plugin teardown.
        /// </summary>
        /// <remarks>
        /// Kept as a sibling-symmetry seam with <see cref="HudElementRendererAdapter.SetCanvasRoot"/>
        /// so DFCanvas can wire all extended adapters in one pass. Today the menu host renders
        /// inside its own <see cref="ModMenuPanel"/> child of <c>DFCanvas_Root</c>, so the
        /// canvas root reference is intent-only — held for future hover/positioning work that
        /// needs the parent transform without re-locating it.
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
        public bool IsVisible
        {
            get
            {
                lock (_lock)
                {
                    return _target?.IsVisible ?? _intendedVisible;
                }
            }
        }

        /// <inheritdoc />
        public void Toggle()
        {
            ToggleCore();
        }

        /// <inheritdoc />
        public void Show()
        {
            ShowCore();
        }

        /// <inheritdoc />
        public void Hide()
        {
            HideCore();
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate any future Unity-touching work behind the
        // public surface. Today they only touch managed state + a runtime-side
        // delegate, but keeping the split mirrors NativeMenuHostAdapter so JIT
        // never loads UnityEngine assemblies during unit tests that don't set
        // a target.
        // ------------------------------------------------------------------ //

        private void ToggleCore()
        {
            lock (_lock)
            {
                if (_target != null)
                {
                    _target.Toggle();
                    _intendedVisible = _target.IsVisible;
                    return;
                }

                _intendedVisible = !_intendedVisible;
            }
        }

        private void ShowCore()
        {
            lock (_lock)
            {
                _intendedVisible = true;
                _target?.Show();
            }
        }

        private void HideCore()
        {
            lock (_lock)
            {
                _intendedVisible = false;
                _target?.Hide();
            }
        }
    }
}
