// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 6 of #193 SDK split — second extended-side runtime adapter (mod-settings).

#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;
using ExtendedIModSettingsHost = DINOForge.SDK.UI.Extended.IModSettingsHost;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="ExtendedIModSettingsHost"/> contract.
    /// Records per-mod settings-panel registrations destined for the F10 mod-menu chrome
    /// (currently fronted by <see cref="ModSettingsPanel"/>).
    /// </summary>
    /// <remarks>
    /// The runtime today renders settings via BepInEx <c>ConfigEntry</c> auto-discovery
    /// (see <see cref="ModSettingsPanel.DiscoverSettings"/>). This adapter holds the
    /// pack-supplied <see cref="IModSettingsPanel"/> instances until the runtime gains a
    /// real renderer for them — at which point Plugin.cs / DFCanvas wiring will pull
    /// from <see cref="GetRegisteredPanels"/> and <see cref="IModSettingsPanel.Build"/>
    /// each one against the mod canvas.
    ///
    /// Until that wiring lands, registrations are intent-only. The adapter still validates
    /// arguments per the <see cref="IModSettingsHost"/> contract so packs fail fast on
    /// misuse rather than silently dropping registrations.
    ///
    /// Last-writer-wins on duplicate <c>modId</c>: matches the registry semantics used by
    /// other extended UI hosts (see <see cref="HudElementRegistry"/> and friends) and lets
    /// pack hot-reload swap a panel cleanly without an explicit unregister call.
    ///
    /// Naming note: the runtime exposes its own <c>DINOForge.Runtime.UI.IModSettingsHost</c>
    /// (visibility-control surface for the IMGUI BepInEx panel) — this adapter intentionally
    /// implements the SDK contract instead, which is registration-focused. The two interfaces
    /// are deliberately separate; this adapter does NOT implement the Runtime one.
    /// </remarks>
    public sealed class ModSettingsHostAdapter : ExtendedIModSettingsHost
    {
        private static ModSettingsHostAdapter? _instance;

        /// <summary>Singleton accessor — registrations are process-wide.</summary>
        public static ModSettingsHostAdapter Instance => _instance ??= new ModSettingsHostAdapter();

        private readonly object _lock = new object();
        private readonly Dictionary<string, IModSettingsPanel> _panels =
            new Dictionary<string, IModSettingsPanel>(StringComparer.Ordinal);

        // Opaque holder; fully-qualified to keep UnityEngine.Transform off the public
        // surface so this file compiles in CI (no UnityEngine reference). Stays null
        // in CI / unit tests; DFCanvas wires it in game-installed builds. The future
        // settings renderer (M11.5+) will use this as the parent transform when
        // calling <see cref="IModSettingsPanel.Build"/> against a real mod canvas.
        private object? _canvasRoot;

        private ModSettingsHostAdapter() { }

        /// <summary>
        /// Wires the adapter to the live <c>DFCanvas_Root</c> Transform. Called by
        /// <see cref="DFCanvas"/> once the canvas hierarchy is built. Passing
        /// <c>null</c> detaches — used by tests + plugin teardown.
        /// </summary>
        /// <remarks>
        /// Mirrors <see cref="HudElementRendererAdapter.SetCanvasRoot"/> so DFCanvas can wire
        /// all extended adapters in one pass. Settings renderer is intent-only today; once
        /// the runtime gains a real <see cref="IModSettingsPanel"/> renderer it will call
        /// <c>Build</c> against the mod canvas and parent the resulting widgets here.
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
        public void RegisterSettingsPanel(string modId, IModSettingsPanel panel)
        {
            if (modId is null) throw new ArgumentNullException(nameof(modId));
            if (modId.Length == 0) throw new ArgumentException("modId must not be empty", nameof(modId));
            if (panel is null) throw new ArgumentNullException(nameof(panel));

            RegisterSettingsPanelCore(modId, panel);
        }

        /// <summary>
        /// Returns a snapshot of currently registered panels keyed by <c>modId</c>.
        /// Used by the runtime renderer (when wired) to enumerate panels for display
        /// and by tests to verify registration behavior.
        /// </summary>
        public IReadOnlyDictionary<string, IModSettingsPanel> GetRegisteredPanels()
        {
            lock (_lock)
            {
                // Copy under lock so callers can iterate without holding the adapter lock.
                return new Dictionary<string, IModSettingsPanel>(_panels, StringComparer.Ordinal);
            }
        }

        /// <summary>
        /// Removes a previously registered panel and disposes it. No-op if <paramref name="modId"/>
        /// was never registered. Used by pack hot-reload teardown.
        /// </summary>
        public void UnregisterSettingsPanel(string modId)
        {
            if (modId is null) throw new ArgumentNullException(nameof(modId));
            if (modId.Length == 0) throw new ArgumentException("modId must not be empty", nameof(modId));

            UnregisterSettingsPanelCore(modId);
        }

        // ------------------------------------------------------------------ //
        // *Core methods mirror the split used by sibling adapters: keeps the
        // public surface argument-validation-only so unit tests run without
        // loading Unity (or BepInEx) assemblies, and leaves a clear seam for
        // the runtime renderer to hook into when M11.5 lands.
        // ------------------------------------------------------------------ //

        private void RegisterSettingsPanelCore(string modId, IModSettingsPanel panel)
        {
            IModSettingsPanel? evicted;
            lock (_lock)
            {
                _panels.TryGetValue(modId, out evicted);
                _panels[modId] = panel;
            }

            // Dispose the evicted panel outside the lock — Dispose() may run user code
            // (pack-supplied) that we must not invoke under the adapter lock.
            if (evicted != null && !ReferenceEquals(evicted, panel))
            {
                try { evicted.Dispose(); } catch { /* pack Dispose failures must not break re-registration */ }
            }
        }

        private void UnregisterSettingsPanelCore(string modId)
        {
            IModSettingsPanel? removed;
            lock (_lock)
            {
                if (!_panels.TryGetValue(modId, out removed))
                {
                    return;
                }
                _panels.Remove(modId);
            }

            if (removed != null)
            {
                try { removed.Dispose(); } catch { /* pack Dispose failures must not break unregister */ }
            }
        }
    }
}
