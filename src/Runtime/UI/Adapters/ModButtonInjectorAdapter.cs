// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 7 of #193 SDK split — bridge adapter (mod-button injection).

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Bridge;
using DINOForge.SDK.UI.Native;
using ExtendedIModMenuHost = DINOForge.SDK.UI.Extended.IModMenuHost;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="IModButtonInjector"/> contract — the
    /// SEAM where Native and Extended UI cross. Records intent to wire a vanilla DINO button
    /// (e.g. the main-menu "Mods" entry) to invoke a mod-owned <see cref="IModMenuHost"/> on click.
    /// </summary>
    /// <remarks>
    /// Per the <see cref="IModButtonInjector"/> contract this is the ONLY place where Native and
    /// Extended UI cross. The native side supplies a <see cref="NativeCanvasHandle"/> (vanilla
    /// DINO canvas), the extended side supplies an <see cref="IModMenuHost"/> (DFCanvas-bound
    /// mod menu host), and this adapter wires them together.
    ///
    /// Phase 2 Dispatch 7 closes the bridge side of the #193 split. The Runtime today routes
    /// the actual native-button rewire through <c>NativeMenuInjector</c> (the legacy IMGUI-era
    /// helper), which is not yet driven from the SDK contract. As of this dispatch the
    /// adapter:
    ///   1. Validates arguments per <see cref="IModButtonInjector"/> (null canvas / null target).
    ///   2. Records the (canvas, target) pair in an internal map keyed by a stable canvas
    ///      identifier so duplicate injections, hot-reload re-injections, and tests can observe
    ///      what was wired. Last-writer-wins on duplicate canvas key — matches the registry
    ///      semantics used by sibling adapters (see <see cref="ModSettingsHostAdapter"/> +
    ///      <see cref="NativeMenuHostAdapter"/>) and lets pack hot-reload swap the target
    ///      cleanly without an explicit unregister.
    ///   3. Returns a <see cref="NativeButtonHandle"/> placeholder so callers (and tests) get
    ///      a valid handle even before <c>NativeMenuInjector</c> is routed through this seam.
    ///
    /// When M11.5 lands, <see cref="InjectModsButtonCore"/> will route into the native-button
    /// adapter (find vanilla "Mods" button, rewire onClick to <c>target.Toggle()</c>) and the
    /// returned handle will wrap the real <c>UnityEngine.UI.Button</c>. Until then the
    /// registration is intent-only — Plugin.cs / DFCanvas wiring will pull from
    /// <see cref="GetInjections"/> when it gains real Unity wiring.
    ///
    /// Canvas keying: the adapter keys by <see cref="NativeCanvasHandle.Name"/> when non-empty,
    /// otherwise it falls back to the <c>Inner</c> reference (boxed object identity). This
    /// matches how <see cref="NativeMenuHostAdapter"/> handles its own keyed-by-string registry
    /// and tolerates tests that construct handles with <c>new object()</c> as <c>Inner</c>.
    ///
    /// Threading: Plugin.cs / NativeMenuInjector expect calls on the Unity main thread, but
    /// pack code (HMR signal watcher, F10 input) may invoke from background threads. The
    /// adapter guards its registry under <c>_lock</c>; callers remain responsible for thread
    /// affinity on any future Unity-touching work inside <see cref="InjectModsButtonCore"/>.
    /// </remarks>
    public sealed class ModButtonInjectorAdapter : IModButtonInjector
    {
        private static ModButtonInjectorAdapter? _instance;

        /// <summary>Singleton accessor — injections are process-wide.</summary>
        public static ModButtonInjectorAdapter Instance => _instance ??= new ModButtonInjectorAdapter();

        // Guarded by _lock; the Unity main thread is the primary caller but Plugin.cs background
        // threads (HMR watcher, F9/F10 input) may also drive injection on hot-reload.
        private readonly object _lock = new object();
        private readonly Dictionary<object, InjectionRecord> _injections =
            new Dictionary<object, InjectionRecord>();

        // Opaque holder; fully-qualified to keep UnityEngine.Transform off the public
        // surface so this file compiles in CI (no UnityEngine reference). Stays null
        // in CI / unit tests; DFCanvas wires it in game-installed builds. The seam adapter
        // itself acts on a vanilla canvas (NativeCanvasHandle), but holds the DFCanvas root
        // so the future <c>NativeMenuInjector</c> wire can locate the mod-owned mod menu
        // host's panel without re-walking the scene graph.
        private object? _canvasRoot;

        private ModButtonInjectorAdapter() { }

        /// <summary>
        /// Wires the adapter to the live <c>DFCanvas_Root</c> Transform. Called by
        /// <see cref="DFCanvas"/> once the canvas hierarchy is built. Passing
        /// <c>null</c> detaches — used by tests + plugin teardown.
        /// </summary>
        /// <remarks>
        /// Mirrors <see cref="HudElementRendererAdapter.SetCanvasRoot"/> so DFCanvas can wire
        /// all extended adapters in one pass. The injector's primary canvas is the vanilla
        /// <see cref="NativeCanvasHandle"/> supplied per call to <see cref="InjectModsButton"/>;
        /// the DFCanvas root is held for future <c>NativeMenuInjector</c> work that needs to
        /// resolve the mod-menu panel without re-locating it.
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

        /// <summary>
        /// Records intent to wire the vanilla "Mods" button on <paramref name="canvas"/> to
        /// invoke <paramref name="target"/> on click. Returns a handle to the (eventually
        /// rewired) button.
        /// </summary>
        /// <param name="canvas">The vanilla canvas hosting the button to rewire.</param>
        /// <param name="target">The mod-owned menu host to invoke when the button is clicked.</param>
        /// <returns>
        /// A <see cref="NativeButtonHandle"/> that callers can pass to other native adapters.
        /// Until M11.5 wires real <c>UnityEngine.UI.Button</c> rewire, the handle's
        /// <c>Inner</c> is the supplied <paramref name="target"/> reference (intent marker).
        /// </returns>
        /// <exception cref="ArgumentNullException">
        /// Thrown when <paramref name="canvas"/> or <paramref name="target"/> is null.
        /// </exception>
        public NativeButtonHandle InjectModsButton(NativeCanvasHandle canvas, ExtendedIModMenuHost target)
        {
            if (canvas is null) throw new ArgumentNullException(nameof(canvas));
            if (target is null) throw new ArgumentNullException(nameof(target));

            return InjectModsButtonCore(canvas, target);
        }

        /// <summary>
        /// Returns a snapshot of currently recorded injections keyed by canvas identifier.
        /// Used by the runtime renderer (when wired) to enumerate seam crossings and by tests
        /// to verify <see cref="InjectModsButton"/> behavior.
        /// </summary>
        /// <remarks>
        /// The returned dictionary is a copy — callers may iterate without holding the adapter
        /// lock. The key is the stable canvas identifier used internally
        /// (<see cref="NativeCanvasHandle.Name"/> when non-empty, else the <c>Inner</c>
        /// reference). The value carries both the canvas handle and the resolved target so
        /// tests can assert the full seam crossing.
        /// </remarks>
        public IReadOnlyDictionary<object, InjectionRecord> GetInjections()
        {
            lock (_lock)
            {
                return new Dictionary<object, InjectionRecord>(_injections);
            }
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate any future UnityEngine references behind the
        // null guards above. Today they only touch managed state, but when
        // M11.5 wires NativeMenuInjector through this seam the Unity work
        // moves here so JIT defers UnityEngine.UIModule loading until a Core
        // method runs — keeps null-arg unit tests free of FileNotFoundException
        // for Unity assemblies, matching the pattern used by sibling adapters.
        // ------------------------------------------------------------------ //

        private NativeButtonHandle InjectModsButtonCore(NativeCanvasHandle canvas, ExtendedIModMenuHost target)
        {
            object key = ResolveCanvasKey(canvas);
            var record = new InjectionRecord(canvas, target);

            lock (_lock)
            {
                // Last-writer-wins on duplicate canvas; matches sibling-adapter registry
                // semantics and lets pack hot-reload swap the target cleanly.
                _injections[key] = record;
            }

            // PHASE2-TODO (M11.5 / WI-004b): when NativeMenuInjector routes through this seam,
            // find the vanilla "Mods" button on the supplied canvas via INativeButtonAdapter,
            // rewire its onClick to invoke target.Toggle(), and wrap the real UnityEngine.UI.Button
            // in the returned handle. Until then the handle's Inner is the target reference —
            // a marker that the seam was crossed but not yet realized on the native canvas.
            string handleName = string.IsNullOrEmpty(canvas.Name) ? "ModsButton" : canvas.Name + ".ModsButton";
            return new NativeButtonHandle(handleName, "Mods", target);
        }

        private static object ResolveCanvasKey(NativeCanvasHandle canvas)
        {
            // Prefer the canvas Name (Ordinal-comparable string) when non-empty; tests + real
            // Unity canvases both supply a non-empty name. Fall back to Inner reference identity
            // for the (rare) anonymous-canvas case — matches how NativeMenuHostAdapter keys.
            if (!string.IsNullOrEmpty(canvas.Name))
            {
                return canvas.Name;
            }
            return canvas.Inner;
        }

        /// <summary>
        /// Snapshot of a single (canvas, target) injection recorded by
        /// <see cref="ModButtonInjectorAdapter.InjectModsButton"/>.
        /// </summary>
        public sealed class InjectionRecord
        {
            /// <summary>The vanilla canvas the button was injected onto.</summary>
            public NativeCanvasHandle Canvas { get; }

            /// <summary>The mod-owned menu host the button invokes on click.</summary>
            public ExtendedIModMenuHost Target { get; }

            internal InjectionRecord(NativeCanvasHandle canvas, ExtendedIModMenuHost target)
            {
                Canvas = canvas;
                Target = target;
            }
        }
    }
}
