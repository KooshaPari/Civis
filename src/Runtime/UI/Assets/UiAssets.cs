#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Runtime loader for Kenney CC0 UI sprite assets.
    /// Sprites are read from disk at the deployed <c>dinoforge-ui-assets/</c> directory
    /// that sits alongside the plugin DLL inside BepInEx/plugins/.
    ///
    /// Usage:
    /// <code>
    ///   UiAssets.Initialize(pluginDirectory);   // call once from Plugin.Awake
    ///   Sprite? bg = UiAssets.PanelBackground;  // null when pack not installed
    /// </code>
    ///
    /// All methods return <c>null</c> (never throw) when the asset file is absent so
    /// callers can fall back to flat-colour UGUI gracefully.
    /// </summary>
    public static class UiAssets
    {
        // ── Configuration ────────────────────────────────────────────────────────

        /// <summary>Sub-folder inside the deployed assets root for each Kenney pack.</summary>
        private const string UiPackDir = "kenney/ui-pack/PNG";
        private const string SciFiPackDir = "kenney/ui-pack-sci-fi/PNG";
        private const string FantasyBordersDir = "kenney/fantasy-ui-borders/PNG";
        private const string AdventurePackDir = "kenney/ui-pack-adventure/PNG";

        // ── State ────────────────────────────────────────────────────────────────

        private static string _assetDir = string.Empty;
        private static bool _initialized;

        /// <summary>Paths that failed to load, for diagnostics.</summary>
        private static readonly List<string> _missingFiles = new List<string>();

        // ── Lazy sprite cache ────────────────────────────────────────────────────

        private static Sprite? _panelBackground;
        private static Sprite? _panelSciFi;
        private static Sprite? _panelFantasy;
        private static Sprite? _buttonNormal;
        private static Sprite? _buttonPressed;
        private static Sprite? _checkboxUnchecked;
        private static Sprite? _checkboxChecked;
        private static Sprite? _scrollTrack;
        private static Sprite? _scrollHandle;

        // ── Initialization ───────────────────────────────────────────────────────

        /// <summary>
        /// Points the asset loader at the plugin's deployed asset directory.
        /// Must be called once from <c>Plugin.Awake()</c> (or equivalent) before
        /// any sprite property is accessed.
        /// </summary>
        /// <param name="pluginDir">
        /// Absolute path to the directory that contains the plugin DLL.
        /// Kenney assets are expected at <c>{pluginDir}/dinoforge-ui-assets/</c>.
        /// </param>
        public static void Initialize(string pluginDir)
        {
            _assetDir = Path.Combine(pluginDir, "dinoforge-ui-assets");
            _initialized = true;
            _missingFiles.Clear();

            // Pre-warm the most critical sprites so failures surface early in logs.
            _ = PanelBackground;
            _ = ButtonNormal;
        }

        // ── Well-known sprite accessors ──────────────────────────────────────────

        /// <summary>
        /// 9-sliceable panel background from the Kenney UI Pack.
        /// Returns <c>null</c> when the pack has not been installed.
        /// Use with <see cref="Image.type"/> = <see cref="Image.Type.Sliced"/>.
        /// </summary>
        public static Sprite? PanelBackground =>
            _panelBackground ??= LoadSlicedSprite(UiPackDir + "/panel_background.png", border: 8);

        /// <summary>Sci-Fi themed panel corner from the Kenney UI Pack Sci-Fi.</summary>
        public static Sprite? PanelSciFi =>
            _panelSciFi ??= LoadSlicedSprite(SciFiPackDir + "/panel_metalCorner.png", border: 12);

        /// <summary>Fantasy ornate bordered panel from the Kenney Fantasy UI Borders pack.</summary>
        public static Sprite? PanelFantasy =>
            _panelFantasy ??= LoadSlicedSprite(FantasyBordersDir + "/panel_ornate.png", border: 16);

        /// <summary>Flat rectangle button — normal / idle state.</summary>
        public static Sprite? ButtonNormal =>
            _buttonNormal ??= LoadSlicedSprite(UiPackDir + "/button_rectangleFlat.png", border: 6);

        /// <summary>Flat rectangle button — depressed / pressed state.</summary>
        public static Sprite? ButtonPressed =>
            _buttonPressed ??= LoadSlicedSprite(UiPackDir + "/button_rectangleDepressed.png", border: 6);

        /// <summary>Checkbox in unchecked state.</summary>
        public static Sprite? CheckboxUnchecked =>
            _checkboxUnchecked ??= LoadSprite(UiPackDir + "/checkbox_unchecked.png");

        /// <summary>Checkbox in checked state.</summary>
        public static Sprite? CheckboxChecked =>
            _checkboxChecked ??= LoadSprite(UiPackDir + "/checkbox_checked.png");

        /// <summary>Scrollbar track (horizontal strip).</summary>
        public static Sprite? ScrollTrack =>
            _scrollTrack ??= LoadSlicedSprite(UiPackDir + "/slide_horizontal_color_n.png", border: 4);

        /// <summary>Scrollbar drag handle.</summary>
        public static Sprite? ScrollHandle =>
            _scrollHandle ??= LoadSprite(UiPackDir + "/slide_hangle.png");

        // ── Generic loaders ──────────────────────────────────────────────────────

        /// <summary>
        /// Loads a PNG from disk and wraps it in a <see cref="Sprite"/>.
        /// The sprite pivot is centred at (0.5, 0.5).
        /// Returns <c>null</c> (never throws) if the file does not exist.
        /// </summary>
        /// <param name="relativePath">
        /// Path relative to the deployed asset root, e.g.
        /// <c>"kenney/ui-pack/PNG/panel_background.png"</c>.
        /// </param>
        public static Sprite? LoadSprite(string relativePath)
        {
            if (!_initialized) return null;

            string fullPath = Path.Combine(_assetDir, relativePath);
            if (!File.Exists(fullPath))
            {
                RecordMissing(fullPath);
                return null;
            }

            try
            {
                byte[] data = File.ReadAllBytes(fullPath);
                Texture2D tex = new Texture2D(2, 2, TextureFormat.RGBA32, mipChain: false)
                {
                    filterMode = FilterMode.Bilinear,
                    wrapMode = TextureWrapMode.Clamp
                };

                if (!tex.LoadImage(data))
                {
                    RecordMissing(fullPath);
                    return null;
                }

                return Sprite.Create(
                    tex,
                    new Rect(0f, 0f, tex.width, tex.height),
                    new Vector2(0.5f, 0.5f),
                    pixelsPerUnit: 100f);
            }
            catch (Exception ex)
            {
                BepInEx.Logging.Logger.CreateLogSource("UiAssets").LogWarning(
                    $"[UiAssets] Failed to load sprite '{relativePath}': {ex}");
                return null;
            }
        }

        /// <summary>
        /// Loads a PNG from disk and wraps it in a 9-slice <see cref="Sprite"/> suitable for
        /// scalable panels and buttons.
        /// Returns <c>null</c> (never throws) if the file does not exist.
        /// </summary>
        /// <param name="relativePath">Path relative to the deployed asset root.</param>
        /// <param name="border">Uniform border width in pixels for all four edges.</param>
        public static Sprite? LoadSlicedSprite(string relativePath, int border)
        {
            if (!_initialized) return null;

            string fullPath = Path.Combine(_assetDir, relativePath);
            if (!File.Exists(fullPath))
            {
                RecordMissing(fullPath);
                return null;
            }

            try
            {
                byte[] data = File.ReadAllBytes(fullPath);
                Texture2D tex = new Texture2D(2, 2, TextureFormat.RGBA32, mipChain: false)
                {
                    filterMode = FilterMode.Bilinear,
                    wrapMode = TextureWrapMode.Clamp
                };

                if (!tex.LoadImage(data))
                {
                    RecordMissing(fullPath);
                    return null;
                }

                Vector4 borders = new Vector4(border, border, border, border);

                return Sprite.Create(
                    tex,
                    new Rect(0f, 0f, tex.width, tex.height),
                    new Vector2(0.5f, 0.5f),
                    pixelsPerUnit: 100f,
                    extrude: 0,
                    meshType: SpriteMeshType.FullRect,
                    border: borders);
            }
            catch (Exception ex)
            {
                BepInEx.Logging.Logger.CreateLogSource("UiAssets").LogWarning(
                    $"[UiAssets] Failed to load sliced sprite '{relativePath}': {ex}");
                return null;
            }
        }

        // ── Cache control ────────────────────────────────────────────────────────

        /// <summary>
        /// Clears all cached sprites and resets state.
        /// Call when re-initializing (e.g. after the plugin directory changes in tests).
        /// </summary>
        public static void Reset()
        {
            _panelBackground = null;
            _panelSciFi = null;
            _panelFantasy = null;
            _buttonNormal = null;
            _buttonPressed = null;
            _checkboxUnchecked = null;
            _checkboxChecked = null;
            _scrollTrack = null;
            _scrollHandle = null;
            _missingFiles.Clear();
            _initialized = false;
            _assetDir = string.Empty;
        }

        // ── Diagnostics ──────────────────────────────────────────────────────────

        /// <summary>
        /// Returns the list of full paths that could not be loaded.
        /// Useful for in-game diagnostics or surfacing setup instructions to the user.
        /// </summary>
        public static IReadOnlyList<string> MissingFiles => _missingFiles;

        /// <summary>Whether <see cref="Initialize"/> has been called.</summary>
        public static bool IsInitialized => _initialized;

        /// <summary>Resolved absolute path to the asset root directory.</summary>
        public static string AssetDirectory => _assetDir;

        // ── Private helpers ───────────────────────────────────────────────────────

        private static void RecordMissing(string fullPath)
        {
            if (!_missingFiles.Contains(fullPath))
                _missingFiles.Add(fullPath);
        }
    }
}
