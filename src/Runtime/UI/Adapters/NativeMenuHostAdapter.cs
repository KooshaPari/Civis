// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 4 of #193 SDK split — fifth (final) native-side runtime adapter.

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Native;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="INativeMenuHost"/> contract.
    /// Records named-screen registrations destined for the vanilla DINO main-menu surface
    /// (currently fronted by the stub <c>NativeMainMenuModMenu</c>).
    /// </summary>
    /// <remarks>
    /// Phase 2 Dispatch 4 closes the native side of the #193 split. The Runtime today routes
    /// mod menus through <c>ContextualModMenuHost</c>, which delegates to <c>NativeMainMenuModMenu</c>
    /// only when <c>CanUseNativeScreen</c> returns true. As of this dispatch
    /// <c>NativeMainMenuModMenu.CanUseNativeScreen</c> is a hardcoded <c>false</c>
    /// (full native injection is deferred to M11.5 / WI-004a), so registrations made through
    /// this adapter cannot yet drive a real on-screen surface.
    ///
    /// To keep the SDK contract honest, the adapter:
    ///   1. Validates arguments (null-arg + empty-id) per <see cref="INativeMenuHost"/>.
    ///   2. Records the (menuId, screen) pair in an internal map so duplicate registrations,
    ///      unregister-without-register paths, and <see cref="IsActive"/> reflect real intent.
    ///   3. Exposes the recorded count via <see cref="IsActive"/> — true iff any screen is
    ///      currently registered.
    ///
    /// When M11.5 lands, the Core methods below will route into <c>NativeMainMenuModMenu</c>
    /// (via <c>ContextualModMenuHost</c>) so registrations actually mount on the vanilla canvas.
    /// </remarks>
    public sealed class NativeMenuHostAdapter : INativeMenuHost
    {
        private static NativeMenuHostAdapter? _instance;

        /// <summary>Singleton accessor — registrations are process-wide.</summary>
        public static NativeMenuHostAdapter Instance => _instance ??= new NativeMenuHostAdapter();

        // Guarded by _lock; the Runtime UI thread is single-threaded but Plugin.cs background
        // threads (HMR watcher, F9/F10 input) may also call register/unregister.
        private readonly object _lock = new object();
        private readonly Dictionary<string, INativeMenuScreen> _screens =
            new Dictionary<string, INativeMenuScreen>(StringComparer.Ordinal);

        private NativeMenuHostAdapter() { }

        /// <inheritdoc />
        public bool IsActive
        {
            get
            {
                lock (_lock)
                {
                    return _screens.Count > 0;
                }
            }
        }

        /// <inheritdoc />
        public void RegisterScreen(string menuId, INativeMenuScreen screen)
        {
            if (menuId is null) throw new ArgumentNullException(nameof(menuId));
            if (menuId.Length == 0) throw new ArgumentException("menuId must not be empty", nameof(menuId));
            if (screen is null) throw new ArgumentNullException(nameof(screen));

            RegisterScreenCore(menuId, screen);
        }

        /// <inheritdoc />
        public void UnregisterScreen(string menuId)
        {
            if (menuId is null) throw new ArgumentNullException(nameof(menuId));
            if (menuId.Length == 0) throw new ArgumentException("menuId must not be empty", nameof(menuId));

            UnregisterScreenCore(menuId);
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate any future UnityEngine references behind the
        // null guards above. Today they only touch managed state, but when
        // M11.5 wires NativeMainMenuModMenu the Unity work moves here so JIT
        // defers UnityEngine.UIModule loading until a Core method runs — keeps
        // null-arg unit tests free of FileNotFoundException for Unity assemblies.
        // ------------------------------------------------------------------ //

        private void RegisterScreenCore(string menuId, INativeMenuScreen screen)
        {
            lock (_lock)
            {
                // Last-writer-wins on duplicate menuId; matches the registry semantics used
                // by extended UI hosts and lets pack hot-reload swap a screen cleanly.
                _screens[menuId] = screen;
            }

            // PHASE2-TODO (M11.5 / WI-004a): when NativeMainMenuModMenu.CanUseNativeScreen
            // returns true, route the registration into ContextualModMenuHost so the screen
            // mounts on the vanilla main-menu canvas. Until then the registration is intent-only.
        }

        private void UnregisterScreenCore(string menuId)
        {
            lock (_lock)
            {
                _screens.Remove(menuId);
            }

            // PHASE2-TODO (M11.5 / WI-004a): mirror Register's wire-up — drop the screen from
            // the Runtime menu host so the vanilla canvas reverts.
        }
    }
}
