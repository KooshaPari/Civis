#nullable enable
using System;
using System.Collections.Generic;
using BepInEx.Logging;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Applies the active total_conversion pack's <c>ui_theme</c> colors to EVERY non-MainMenu
    /// native canvas — the Settings screen and its sub-tabs (GAME / VIDEO / SOUND / CONTROLS /
    /// TWITCH), plus the game create / select screens. <see cref="MainMenuThemer"/> owns the
    /// MainMenu canvas (full art takeover); this reskinner covers the remaining pages with a
    /// color-only pass so the whole UI reads as one themed skin.
    ///
    /// Design:
    ///   - Idempotent and re-runnable. Settings/sub-tab panels are created lazily by DINO when the
    ///     user navigates, so a single apply at menu-load time misses them. The runtime pump calls
    ///     <see cref="ReskinAllPages"/> on a bounded retry cadence; each call re-walks all active
    ///     canvases and re-applies. A per-object marker component (<see cref="ReskinMarker"/>)
    ///     prevents redundant re-tinting of objects already skinned this session.
    ///   - Never touches the MainMenu canvas (left to MainMenuThemer) or DINOForge-owned objects.
    ///   - Color-only: no sprite/art swaps, so it degrades gracefully on any unknown panel layout.
    ///   - All operations are best-effort and never throw into the pump.
    /// </summary>
    internal sealed class CanvasReskinner
    {
        private readonly ManualLogSource? _log;
        private readonly string _packsDirectory;

        // Cached resolved theme (re-resolved when the pack set changes).
        private MenuThemeReader.MenuTheme _theme = MenuThemeReader.MenuTheme.Default;
        private bool _themeResolved;

        public CanvasReskinner(ManualLogSource? log, string packsDirectory)
        {
            _log = log;
            _packsDirectory = packsDirectory ?? string.Empty;
        }

        /// <summary>Re-resolve the theme on the next reskin pass (call on scene change / pack reload).</summary>
        public void Invalidate() => _themeResolved = false;

        /// <summary>
        /// Marker component left on objects already reskinned this session so repeated pump passes
        /// are cheap and don't fight DINO's own per-frame restyle.
        /// </summary>
        private sealed class ReskinMarker : MonoBehaviour { }

        /// <summary>
        /// Walk every active canvas except MainMenu and apply the theme color block to each
        /// Selectable, and the theme text color to each Text / TMP_Text. Returns the number of
        /// newly-skinned objects (0 when nothing new appeared).
        /// </summary>
        public int ReskinAllPages(IReadOnlyList<PackDisplayInfo>? packs)
        {
            try
            {
                if (!_themeResolved)
                {
                    _theme = MenuThemeReader.Resolve(packs, _packsDirectory);
                    _themeResolved = true;
                }

                int total = 0;
                Canvas[] canvases = UnityEngine.Object.FindObjectsOfType<Canvas>();
                foreach (Canvas canvas in canvases)
                {
                    if (canvas == null || !canvas.gameObject.activeInHierarchy) continue;
                    // MainMenuThemer owns the MainMenu canvas (full art takeover) — skip it.
                    if (canvas.name.IndexOf("MainMenu", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    // Never reskin DINOForge's own UI (F10 panel / native mods page / quick panel).
                    if (canvas.name.IndexOf("DINOForge", StringComparison.Ordinal) >= 0) continue;
                    total += ReskinCanvas(canvas);
                }

                if (total > 0)
                    _log?.LogInfo($"[CanvasReskinner] Skinned {total} new native element(s) across non-MainMenu pages.");
                return total;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[CanvasReskinner] ReskinAllPages failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return 0;
            }
        }

        private int ReskinCanvas(Canvas canvas)
        {
            int hits = 0;

            // Selectables: buttons, sliders, toggles, dropdowns, scrollbars, input fields.
            foreach (Selectable sel in canvas.GetComponentsInChildren<Selectable>(true))
            {
                if (sel == null) continue;
                if (IsExcluded(sel.gameObject)) continue;
                if (sel.GetComponent<ReskinMarker>() != null) continue;
                try
                {
                    ColorBlock colors = sel.colors;
                    // Keep the native frame visible (normal stays near-white so the art reads),
                    // drive highlight/press/selected from the pack theme so hover/press match
                    // the themed MainMenu chrome.
                    colors.highlightedColor = WithAlpha(_theme.Primary, 0.90f);
                    colors.pressedColor = WithAlpha(_theme.Accent, 1.0f);
                    colors.selectedColor = WithAlpha(_theme.Primary, 0.75f);
                    sel.colors = colors;

                    // Tint the immediate label/foreground graphic toward the theme text color
                    // for non-Image targetGraphics is risky; only tint sub-Texts below.
                    sel.gameObject.AddComponent<ReskinMarker>();
                    hits++;
                }
                catch { /* safe-swallow: per-selectable restyle is best-effort */ }
            }

            // Legacy UGUI Text labels.
            foreach (Text t in canvas.GetComponentsInChildren<Text>(true))
            {
                if (t == null) continue;
                if (IsExcluded(t.gameObject)) continue;
                if (t.GetComponent<ReskinMarker>() != null) continue;
                try
                {
                    t.color = _theme.Text;
                    t.gameObject.AddComponent<ReskinMarker>();
                    hits++;
                }
                catch { /* safe-swallow */ }
            }

            // TMP_Text labels (reflection — Runtime has no compile-time TMPro reference).
            foreach (Component c in canvas.GetComponentsInChildren<Component>(true))
            {
                if (c == null) continue;
                if (!(c.GetType().FullName ?? string.Empty).StartsWith("TMPro.", StringComparison.Ordinal)) continue;
                if (IsExcluded(c.gameObject)) continue;
                if (c.GetComponent<ReskinMarker>() != null) continue;
                try
                {
                    System.Reflection.PropertyInfo? colorProp = c.GetType().GetProperty("color");
                    if (colorProp != null && colorProp.CanWrite)
                    {
                        colorProp.SetValue(c, _theme.Text);
                        c.gameObject.AddComponent<ReskinMarker>();
                        hits++;
                    }
                }
                catch { /* safe-swallow */ }
            }

            return hits;
        }

        private static bool IsExcluded(GameObject go)
        {
            string n = go.name;
            return n.IndexOf("DINOForge", StringComparison.Ordinal) >= 0
                || n.IndexOf("Mods_Button", StringComparison.Ordinal) >= 0;
        }

        private static Color WithAlpha(Color c, float a) => new Color(c.r, c.g, c.b, a);
    }
}
