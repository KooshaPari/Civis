#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Main-menu-native mod menu host. When the game's MainMenu canvas is active,
    /// clones DINO's native Options panel to display a mod browser page that
    /// visually matches the game's settings UI (fonts, sprites, colors, layout).
    ///
    /// <see cref="CanUseNativeScreen"/> returns true when the MainMenu canvas is
    /// found, causing <see cref="ContextualModMenuHost"/> to route here instead
    /// of the DFCanvas overlay.
    /// </summary>
    public sealed class NativeMainMenuModMenu : IModMenuHost
    {
        private ManualLogSource? _log;
        private NativeModsPage? _modsPage;
        private Canvas? _mainMenuCanvas;
        private GameObject? _mainMenuContent;
        private List<PackDisplayInfo> _cachedPacks = new List<PackDisplayInfo>();
        private string _cachedStatus = "";
        private int _cachedErrorCount;

        /// <summary>
        /// Optional live pack-data source. When set, <see cref="Show"/> pulls fresh pack
        /// info from it if no packs have been pushed via <see cref="SetPacks"/> yet. This
        /// covers the case where the native MODS button is wired through a contextual host
        /// that ModPlatform.UpdateUI never targets — without it the INSTALLED PACKS list
        /// renders empty even though packs are loaded.
        /// </summary>
        public Func<IReadOnlyList<PackDisplayInfo>>? PackDataProvider { get; set; }

        /// <summary>
        /// Whether the vanilla main-menu canvas can host the mod menu.
        /// Returns true when the MainMenu canvas has been located in the scene.
        /// </summary>
        public bool CanUseNativeScreen => FindOrCacheMainMenuCanvas() != null;

        /// <inheritdoc />
        public Action? OnReloadRequested { get; set; }

        /// <inheritdoc />
        public Action<string, bool>? OnPackToggled { get; set; }

        /// <inheritdoc />
        public bool IsVisible => _modsPage != null && _modsPage.IsVisible;

        public void SetLogger(ManualLogSource log)
        {
            _log = log;
        }

        /// <inheritdoc />
        public void Show()
        {
            Canvas? canvas = FindOrCacheMainMenuCanvas();
            if (canvas == null)
            {
                DebugLog.Write("NativeMainMenuModMenu", "Show() called but MainMenu canvas not found");
                return;
            }

            EnsureModsPage(canvas);

            // Pull fresh pack data when nothing has been pushed via SetPacks (the contextual
            // host path is not targeted by ModPlatform.UpdateUI, so _cachedPacks can be empty).
            if (_cachedPacks.Count == 0 && PackDataProvider != null)
            {
                IReadOnlyList<PackDisplayInfo> live = PackDataProvider();
                if (live != null && live.Count > 0)
                    _cachedPacks = new List<PackDisplayInfo>(live);
            }

            if (_modsPage != null)
            {
                _modsPage.Show(canvas, _mainMenuContent);
                _modsPage.SetPacks(new System.Collections.ObjectModel.ReadOnlyCollection<PackDisplayInfo>(_cachedPacks));
                DebugLog.Write("NativeMainMenuModMenu", $"Show() populated NativeModsPage with {_cachedPacks.Count} packs");
            }
        }

        /// <inheritdoc />
        public void Hide()
        {
            if (_modsPage != null)
                _modsPage.Hide();
        }

        /// <inheritdoc />
        public void Toggle()
        {
            if (_modsPage != null && _modsPage.IsVisible)
            {
                Hide();
            }
            else
            {
                Show();
            }
        }

        /// <inheritdoc />
        public void SetPacks(IEnumerable<PackDisplayInfo> packs)
        {
            _cachedPacks = packs?.ToList() ?? new List<PackDisplayInfo>();
            if (_modsPage != null)
            {
                _modsPage.SetPacks(new System.Collections.ObjectModel.ReadOnlyCollection<PackDisplayInfo>(_cachedPacks));
            }
        }

        /// <inheritdoc />
        public void SetStatus(string message, int errorCount = 0)
        {
            _cachedStatus = message;
            _cachedErrorCount = errorCount;
        }

        /// <summary>
        /// Resets cached canvas references. Called on scene change so the native menu
        /// host re-discovers the MainMenu canvas in the new scene.
        /// </summary>
        public void OnSceneChanged()
        {
            _mainMenuCanvas = null;
            _mainMenuContent = null;
            // NativeModsPage is a MonoBehaviour on a DontDestroyOnLoad object;
            // it will be destroyed if the canvas it's parented to is destroyed.
            // We null-check it lazily via the IsVisible path.
            if (_modsPage != null && _modsPage.gameObject == null)
                _modsPage = null;
        }

        // ── Internal helpers ──────────────────────────────────────────────────

        private Canvas? FindOrCacheMainMenuCanvas()
        {
            // Return cached if still alive
            if (_mainMenuCanvas != null && _mainMenuCanvas.gameObject != null && _mainMenuCanvas.gameObject.activeInHierarchy)
                return _mainMenuCanvas;

            // Search for the MainMenu canvas
            Canvas[] canvases = Resources.FindObjectsOfTypeAll<Canvas>();
            foreach (Canvas c in canvases)
            {
                if (c == null) continue;
                if (string.Equals(c.name, "MainMenu", StringComparison.OrdinalIgnoreCase) && c.gameObject.activeInHierarchy)
                {
                    _mainMenuCanvas = c;

                    // Try to identify the main content container (first direct child with multiple children)
                    for (int i = 0; i < c.transform.childCount; i++)
                    {
                        Transform child = c.transform.GetChild(i);
                        if (child.childCount >= 3)
                        {
                            _mainMenuContent = child.gameObject;
                            break;
                        }
                    }

                    DebugLog.Write("NativeMainMenuModMenu", $"Cached MainMenu canvas: '{c.name}' content='{_mainMenuContent?.name ?? "null"}'");
                    return c;
                }
            }

            _mainMenuCanvas = null;
            return null;
        }

        private void EnsureModsPage(Canvas mainMenuCanvas)
        {
            // If existing page is still alive, nothing to do
            if (_modsPage != null && _modsPage.gameObject != null)
                return;

            // Create a host GameObject that lives under the MainMenu canvas
            GameObject host = new GameObject("DINOForge_NativeModsPageHost");
            host.transform.SetParent(mainMenuCanvas.transform, false);

            _modsPage = host.AddComponent<NativeModsPage>();
            if (_log != null) _modsPage.SetLogger(_log);

            _modsPage.OnBackClicked = () => Hide();
            _modsPage.OnReloadRequested = () => OnReloadRequested?.Invoke();
            _modsPage.OnPackToggled = (id, enabled) => OnPackToggled?.Invoke(id, enabled);

            DebugLog.Write("NativeMainMenuModMenu", "Created NativeModsPage host under MainMenu canvas");
        }
    }
}
