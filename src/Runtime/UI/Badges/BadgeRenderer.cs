#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI.Badges
{
    /// <summary>
    /// Runtime badge renderer for the F10 mod menu detail pane.
    ///
    /// Loads 24×24 PNG badge sprites from
    /// <c>{BepInEx/plugins/dinoforge-ui-assets/badges/}</c> (same root as Kenney assets,
    /// populated by the DeployBadgeAssets MSBuild target).  Each badge is rendered as a
    /// coloured square <see cref="Image"/> with a lightweight tooltip label on the right.
    ///
    /// Falls back gracefully when the PNG is absent — a coloured flat-colour box with the
    /// badge initial letter is shown instead, so the UI never breaks when assets are missing.
    /// </summary>
    public static class BadgeRenderer
    {
        // ── Badge catalogue ──────────────────────────────────────────────────────

        /// <summary>
        /// All recognised badge names mapped to their display configuration.
        /// Add entries here when new badges are introduced.
        /// </summary>
        private static readonly Dictionary<string, BadgeDef> KnownBadges =
            new Dictionary<string, BadgeDef>(StringComparer.Ordinal)
            {
                ["verified-author"]      = new BadgeDef("V",  new Color(0.18f, 0.80f, 0.44f, 1f), "Verified Author — identity confirmed by DINOForge team"),
                ["popular"]              = new BadgeDef("P",  new Color(1.00f, 0.60f, 0.00f, 1f), "Popular — 100+ downloads in the registry"),
                ["editors-choice"]       = new BadgeDef("E",  new Color(1.00f, 0.84f, 0.00f, 1f), "Editor's Choice — curated as an outstanding pack"),
                ["early-access"]         = new BadgeDef("EA", new Color(0.20f, 0.60f, 1.00f, 1f), "Early Access — work-in-progress, expect changes"),
                ["total-conversion"]     = new BadgeDef("TC", new Color(0.60f, 0.20f, 0.80f, 1f), "Total Conversion — replaces game factions and content"),
                ["compatibility-tested"] = new BadgeDef("C",  new Color(0.18f, 0.80f, 0.44f, 1f), "Compatibility Tested — passed automated CI pack tests"),
            };

        // ── Sprite cache ─────────────────────────────────────────────────────────

        private static readonly Dictionary<string, Sprite?> _spriteCache =
            new Dictionary<string, Sprite?>(StringComparer.Ordinal);

        // ── Public API ───────────────────────────────────────────────────────────

        /// <summary>
        /// Attempts to load and cache a 24×24 badge sprite.
        /// Returns <c>null</c> when the PNG does not exist; callers must fall back gracefully.
        /// </summary>
        /// <param name="badgeName">Badge identifier, e.g. <c>"verified-author"</c>.</param>
        public static Sprite? LoadBadgeSprite(string badgeName)
        {
            if (!UiAssets.IsInitialized) return null;

            if (_spriteCache.TryGetValue(badgeName, out Sprite? cached)) return cached;

            string relativePath = $"badges/{badgeName}.png";
            Sprite? sprite = UiAssets.LoadSprite(relativePath);

            // Fall back to the generic placeholder if the named sprite is absent.
            if (sprite == null)
                sprite = UiAssets.LoadSprite("badges/default.png");

            _spriteCache[badgeName] = sprite;
            return sprite;
        }

        /// <summary>
        /// Clears the sprite cache (call after hot-reload or asset directory change).
        /// </summary>
        public static void ResetCache()
        {
            _spriteCache.Clear();
        }

        /// <summary>
        /// Builds a horizontal row of badge pills and appends it to <paramref name="parent"/>.
        /// Each pill is a 24×24 image (sprite if available, flat-colour fallback otherwise)
        /// followed by a small label.  The entire row is hidden when <paramref name="badges"/>
        /// is empty.
        /// </summary>
        /// <param name="parent">Transform to parent the row under (detail pane scroll content).</param>
        /// <param name="badges">Ordered badge identifier list from <see cref="PackDisplayInfo.Badges"/>.</param>
        /// <returns>The root <see cref="GameObject"/> of the row (already parented to <paramref name="parent"/>).</returns>
        public static GameObject BuildBadgeRow(Transform parent, IReadOnlyList<string> badges)
        {
            GameObject rowGo = new GameObject("BadgesRow", typeof(RectTransform));
            rowGo.transform.SetParent(parent, false);

            UnityEngine.UI.HorizontalLayoutGroup hlg = rowGo.AddComponent<UnityEngine.UI.HorizontalLayoutGroup>();
            hlg.spacing = 6f;
            hlg.childForceExpandHeight = false;
            hlg.childForceExpandWidth = false;
            hlg.childAlignment = TextAnchor.MiddleLeft;

            UnityEngine.UI.LayoutElement rowLe = rowGo.AddComponent<UnityEngine.UI.LayoutElement>();
            rowLe.preferredHeight = 28f;
            rowLe.flexibleWidth = 1f;

            bool anyAdded = false;
            foreach (string badgeName in badges)
            {
                if (string.IsNullOrWhiteSpace(badgeName)) continue;
                AddBadgePill(rowGo.transform, badgeName);
                anyAdded = true;
            }

            rowGo.SetActive(anyAdded);
            return rowGo;
        }

        // ── Private helpers ──────────────────────────────────────────────────────

        /// <summary>Adds a single badge pill (icon + label) to <paramref name="parent"/>.</summary>
        private static void AddBadgePill(Transform parent, string badgeName)
        {
            BadgeDef def = KnownBadges.TryGetValue(badgeName, out BadgeDef? found)
                ? found
                : new BadgeDef("?", new Color(0.5f, 0.5f, 0.5f, 1f), badgeName);

            // Outer pill container
            GameObject pill = new GameObject($"Badge_{badgeName}", typeof(RectTransform));
            pill.transform.SetParent(parent, false);
            UnityEngine.UI.HorizontalLayoutGroup pillHlg = pill.AddComponent<UnityEngine.UI.HorizontalLayoutGroup>();
            pillHlg.spacing = 3f;
            pillHlg.childForceExpandHeight = false;
            pillHlg.childForceExpandWidth = false;
            pillHlg.childAlignment = TextAnchor.MiddleLeft;
            pillHlg.padding = new RectOffset(4, 4, 2, 2);
            UnityEngine.UI.LayoutElement pillLe = pill.AddComponent<UnityEngine.UI.LayoutElement>();
            pillLe.preferredHeight = 24f;

            // Optional tinted background
            UnityEngine.UI.Image pillBg = pill.AddComponent<UnityEngine.UI.Image>();
            Color bgColor = def.Color;
            bgColor.a = 0.15f;
            pillBg.color = bgColor;

            // Badge icon: sprite if available, else flat-colour square with initial letter
            Sprite? sprite = LoadBadgeSprite(badgeName);

            GameObject iconGo = new GameObject("Icon", typeof(RectTransform));
            iconGo.transform.SetParent(pill.transform, false);
            UnityEngine.UI.LayoutElement iconLe = iconGo.AddComponent<UnityEngine.UI.LayoutElement>();
            iconLe.preferredWidth = 18f;
            iconLe.preferredHeight = 18f;
            iconLe.minWidth = 18f;
            iconLe.minHeight = 18f;

            if (sprite != null)
            {
                UnityEngine.UI.Image iconImg = iconGo.AddComponent<UnityEngine.UI.Image>();
                iconImg.sprite = sprite;
                iconImg.preserveAspect = true;
            }
            else
            {
                // Fallback: coloured square + initial letter
                UnityEngine.UI.Image iconImg = iconGo.AddComponent<UnityEngine.UI.Image>();
                iconImg.color = def.Color;

                GameObject letterGo = new GameObject("Letter", typeof(RectTransform));
                letterGo.transform.SetParent(iconGo.transform, false);
                Text letterTxt = letterGo.AddComponent<Text>();
                letterTxt.text = def.Initial;
                letterTxt.fontSize = 9;
                letterTxt.fontStyle = FontStyle.Bold;
                letterTxt.alignment = TextAnchor.MiddleCenter;
                letterTxt.color = Color.white;
                RectTransform letterRt = letterGo.GetComponent<RectTransform>();
                letterRt.anchorMin = Vector2.zero;
                letterRt.anchorMax = Vector2.one;
                letterRt.offsetMin = Vector2.zero;
                letterRt.offsetMax = Vector2.zero;
            }

            // Label (badge display name derived from ID)
            string labelText = FormatBadgeLabel(badgeName);
            GameObject labelGo = new GameObject("Label", typeof(RectTransform));
            labelGo.transform.SetParent(pill.transform, false);
            Text labelTxt = labelGo.AddComponent<Text>();
            labelTxt.text = labelText;
            labelTxt.fontSize = 10;
            labelTxt.color = def.Color;
            labelTxt.fontStyle = FontStyle.Bold;
            labelTxt.verticalOverflow = VerticalWrapMode.Truncate;
            labelTxt.horizontalOverflow = HorizontalWrapMode.Overflow;
            UnityEngine.UI.LayoutElement labelLe = labelGo.AddComponent<UnityEngine.UI.LayoutElement>();
            labelLe.preferredHeight = 18f;
        }

        /// <summary>Converts badge ID to human-readable label (e.g. "early-access" → "Early Access").</summary>
        private static string FormatBadgeLabel(string badgeName)
        {
            if (string.IsNullOrEmpty(badgeName)) return badgeName;
            string[] words = badgeName.Split('-');
            System.Text.StringBuilder sb = new System.Text.StringBuilder(badgeName.Length + 4); // stringbuilder-capacity-ok: short label
            foreach (string word in words)
            {
                if (sb.Length > 0) sb.Append(' ');
                if (word.Length > 0)
                {
                    sb.Append(char.ToUpperInvariant(word[0]));
                    if (word.Length > 1) sb.Append(word, 1, word.Length - 1);
                }
            }
            return sb.ToString();
        }

        // ── Inner types ──────────────────────────────────────────────────────────

        private sealed class BadgeDef
        {
            public BadgeDef(string initial, Color color, string tooltip)
            {
                Initial = initial;
                Color = color;
                Tooltip = tooltip;
            }

            public string Initial { get; }
            public Color Color { get; }
            public string Tooltip { get; }
        }
    }
}
