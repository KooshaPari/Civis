#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Compact, NATIVE-STYLE quick panel shown as the DEFAULT action when the player
    /// clicks the injected "Mods" button in DINO's main menu.
    ///
    /// Unlike <see cref="NativeModsPage"/> (the full-screen deep-management browser),
    /// this panel is intentionally small and looks like it belongs in DINO's own menu:
    ///   - Its rows are CLONED from a native DINO menu button (the same donor the
    ///     "Mods" button itself was cloned from), so they inherit DINO's button frame,
    ///     fonts, sprites and hover/press visuals — no synthetic Arial UI.
    ///   - It is themed by the active total_conversion pack's <c>ui_theme</c> (e.g.
    ///     Star Wars gold <c>#FFE81F</c>) via <see cref="MenuThemeReader"/>.
    ///   - Each row is a quick ON/OFF toggle that enables/disables a pack in-place.
    ///   - A "Browse all" button at the bottom routes to the full
    ///     <see cref="NativeModsPage"/> browser for search/filter/details.
    ///
    /// The panel is built once (lazily) and parented to the MainMenu canvas; it is
    /// shown/hidden rather than rebuilt on every click. Pack rows ARE rebuilt on each
    /// show so toggle states reflect the latest <see cref="PackDisplayInfo"/> snapshot.
    /// </summary>
    public sealed class NativeQuickModPanel : MonoBehaviour
    {
        private const string Tag = "NativeQuickModPanel";

        // Compact panel geometry — looks like a DINO sub-menu, not a takeover.
        private const float PanelWidth = 460f;
        private const float RowHeight = 44f;
        private const float MaxListHeight = 340f;

        private ManualLogSource? _log;
        private IReadOnlyList<PackDisplayInfo> _packs = Array.Empty<PackDisplayInfo>();

        // Native donor button to clone for native-chrome rows + buttons.
        private Button? _donorButton;
        private MenuThemeReader.MenuTheme _theme = MenuThemeReader.MenuTheme.Default;

        // Built UI.
        private GameObject? _root;
        private RectTransform? _listContent;
        private readonly List<GameObject> _rows = new List<GameObject>();

        // Callbacks.
        /// <summary>Invoked when the user toggles a pack (packId, isEnabled).</summary>
        public Action<string, bool>? OnPackToggled { get; set; }

        /// <summary>Invoked when the user clicks "Browse all" — route to the full browser.</summary>
        public Action? OnBrowseAllClicked { get; set; }

        /// <summary>Invoked when the user closes the quick panel.</summary>
        public Action? OnClosed { get; set; }

        public void SetLogger(ManualLogSource log) => _log = log;

        /// <summary>True when the quick panel is currently visible.</summary>
        public bool IsVisible => _root != null && _root.activeSelf;

        /// <summary>
        /// Sets the native menu button to clone for row/button chrome and the active
        /// theme (read from the loaded total_conversion pack's ui_theme on disk).
        /// </summary>
        public void Configure(Button? donorButton, MenuThemeReader.MenuTheme theme)
        {
            _donorButton = donorButton;
            _theme = theme;
        }

        /// <summary>Updates the pack snapshot; rebuilds rows if already visible.</summary>
        public void SetPacks(IReadOnlyList<PackDisplayInfo> packs)
        {
            _packs = packs ?? Array.Empty<PackDisplayInfo>();
            if (IsVisible) RebuildRows();
        }

        /// <summary>Shows the quick panel under the given canvas (built lazily).</summary>
        public void Show(Canvas canvas)
        {
            if (canvas == null) return;
            if (_root == null) BuildUI(canvas.transform);
            _root!.transform.SetAsLastSibling();
            _root.SetActive(true);
            RebuildRows();
            LogInfo($"Quick panel shown ({_packs.Count} packs, theme primary={_theme.Primary})");
        }

        /// <summary>Hides the quick panel.</summary>
        public void Hide()
        {
            if (_root != null) _root.SetActive(false);
            LogInfo("Quick panel hidden");
        }

        // ── UI construction ──────────────────────────────────────────────────

        private void BuildUI(Transform parent)
        {
            // Click-blocker backdrop so clicks outside the panel don't reach the menu,
            // but kept subtle (mostly transparent) so it reads as an in-place sub-panel.
            _root = new GameObject("DINOForge_QuickModPanel_Backdrop");
            _root.transform.SetParent(parent, false);
            RectTransform backdropRt = _root.AddComponent<RectTransform>();
            backdropRt.anchorMin = Vector2.zero;
            backdropRt.anchorMax = Vector2.one;
            backdropRt.offsetMin = Vector2.zero;
            backdropRt.offsetMax = Vector2.zero;
            Image backdropImg = _root.AddComponent<Image>();
            backdropImg.color = new Color(0f, 0f, 0f, 0.45f);
            Button backdropBtn = _root.AddComponent<Button>();
            backdropBtn.transition = Selectable.Transition.None;
            backdropBtn.onClick.AddListener(() => { Hide(); OnClosed?.Invoke(); });

            // The actual compact panel, anchored center.
            GameObject panel = new GameObject("QuickPanel");
            panel.transform.SetParent(_root.transform, false);
            RectTransform panelRt = panel.AddComponent<RectTransform>();
            panelRt.anchorMin = new Vector2(0.5f, 0.5f);
            panelRt.anchorMax = new Vector2(0.5f, 0.5f);
            panelRt.pivot = new Vector2(0.5f, 0.5f);
            panelRt.sizeDelta = new Vector2(PanelWidth, MaxListHeight + 150f);

            Image panelBg = panel.AddComponent<Image>();
            // Dark, slightly tinted toward the theme secondary so it reads as DINO chrome.
            panelBg.color = new Color(_theme.Secondary.r, _theme.Secondary.g, _theme.Secondary.b, 0.96f);

            // Themed border accent (thin outline using a child image frame).
            Outline outline = panel.AddComponent<Outline>();
            outline.effectColor = _theme.Primary;
            outline.effectDistance = new Vector2(2f, 2f);

            VerticalLayoutGroup panelLayout = panel.AddComponent<VerticalLayoutGroup>();
            panelLayout.padding = new RectOffset(14, 14, 12, 12);
            panelLayout.spacing = 8;
            panelLayout.childForceExpandWidth = true;
            panelLayout.childForceExpandHeight = false;
            panelLayout.childControlWidth = true;
            panelLayout.childControlHeight = true;

            ContentSizeFitter panelFitter = panel.AddComponent<ContentSizeFitter>();
            panelFitter.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            // panelBg (the Image added above) is a raycast target, so clicks on the panel
            // body are consumed here and never reach the backdrop's close handler.
            BuildHeader(panel.transform);
            BuildList(panel.transform);
            BuildFooter(panel.transform);
        }

        private void BuildHeader(Transform parent)
        {
            GameObject header = new GameObject("Header");
            header.transform.SetParent(parent, false);
            header.AddComponent<RectTransform>();
            HorizontalLayoutGroup hl = header.AddComponent<HorizontalLayoutGroup>();
            hl.childForceExpandWidth = false;
            hl.childForceExpandHeight = true;
            hl.childControlWidth = true;
            hl.childControlHeight = true;
            hl.childAlignment = TextAnchor.MiddleLeft;
            LayoutElement hle = header.AddComponent<LayoutElement>();
            hle.preferredHeight = 34f;

            GameObject titleGo = new GameObject("Title");
            titleGo.transform.SetParent(header.transform, false);
            titleGo.AddComponent<RectTransform>();
            Text title = titleGo.AddComponent<Text>();
            title.text = "MODS";
            title.font = GetFont();
            title.fontSize = 22;
            title.fontStyle = FontStyle.Bold;
            title.color = _theme.Primary;
            title.alignment = TextAnchor.MiddleLeft;
            LayoutElement titleLe = titleGo.AddComponent<LayoutElement>();
            titleLe.flexibleWidth = 1f;

            // Themed accent underline bar.
            GameObject bar = new GameObject("AccentBar");
            bar.transform.SetParent(parent, false);
            bar.AddComponent<RectTransform>();
            Image barImg = bar.AddComponent<Image>();
            barImg.color = _theme.Primary;
            LayoutElement barLe = bar.AddComponent<LayoutElement>();
            barLe.preferredHeight = 2f;
            barLe.flexibleWidth = 1f;
        }

        private void BuildList(Transform parent)
        {
            // Scroll view using RectMask2D (per iter-149f fix: stencil Mask never writes on
            // DINO's Mono/Unity build, so scroll content vanishes — RectMask2D clips by rect).
            GameObject scroll = new GameObject("QuickList");
            scroll.transform.SetParent(parent, false);
            scroll.AddComponent<RectTransform>();
            Image scrollBg = scroll.AddComponent<Image>();
            scrollBg.color = new Color(0, 0, 0, 0.25f);
            ScrollRect scrollRect = scroll.AddComponent<ScrollRect>();
            scrollRect.horizontal = false;
            scrollRect.vertical = true;
            scrollRect.movementType = ScrollRect.MovementType.Clamped;
            LayoutElement scrollLe = scroll.AddComponent<LayoutElement>();
            scrollLe.preferredHeight = MaxListHeight;
            scrollLe.flexibleWidth = 1f;

            GameObject viewport = new GameObject("Viewport");
            viewport.transform.SetParent(scroll.transform, false);
            RectTransform vpRt = viewport.AddComponent<RectTransform>();
            vpRt.anchorMin = Vector2.zero;
            vpRt.anchorMax = Vector2.one;
            vpRt.offsetMin = Vector2.zero;
            vpRt.offsetMax = Vector2.zero;
            viewport.AddComponent<RectMask2D>(); // rect-clip, no graphic-alpha stencil needed

            GameObject content = new GameObject("Content");
            content.transform.SetParent(viewport.transform, false);
            RectTransform contentRt = content.AddComponent<RectTransform>();
            contentRt.anchorMin = new Vector2(0, 1);
            contentRt.anchorMax = new Vector2(1, 1);
            contentRt.pivot = new Vector2(0.5f, 1);
            contentRt.offsetMin = Vector2.zero;
            contentRt.offsetMax = Vector2.zero;
            VerticalLayoutGroup cl = content.AddComponent<VerticalLayoutGroup>();
            cl.spacing = 4;
            cl.padding = new RectOffset(4, 4, 4, 4);
            cl.childForceExpandWidth = true;
            cl.childForceExpandHeight = false;
            cl.childControlWidth = true;
            cl.childControlHeight = true;
            ContentSizeFitter csf = content.AddComponent<ContentSizeFitter>();
            csf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            scrollRect.viewport = vpRt;
            scrollRect.content = contentRt;
            _listContent = contentRt;
        }

        private void BuildFooter(Transform parent)
        {
            GameObject footer = new GameObject("Footer");
            footer.transform.SetParent(parent, false);
            footer.AddComponent<RectTransform>();
            HorizontalLayoutGroup hl = footer.AddComponent<HorizontalLayoutGroup>();
            hl.spacing = 8;
            hl.childForceExpandWidth = true;
            hl.childForceExpandHeight = true;
            hl.childControlWidth = true;
            hl.childControlHeight = true;
            LayoutElement fle = footer.AddComponent<LayoutElement>();
            fle.preferredHeight = 40f;
            fle.flexibleWidth = 1f;

            // "Browse all" — native-chrome button routing to the full browser.
            CreateChromeButton(footer.transform, "Browse all", () =>
            {
                Hide();
                OnBrowseAllClicked?.Invoke();
            });

            // "Close".
            CreateChromeButton(footer.transform, "Close", () =>
            {
                Hide();
                OnClosed?.Invoke();
            });
        }

        // ── Row / button construction ────────────────────────────────────────

        private void RebuildRows()
        {
            foreach (GameObject r in _rows)
                if (r != null) Destroy(r);
            _rows.Clear();
            if (_listContent == null) return;

            if (_packs.Count == 0)
            {
                GameObject empty = new GameObject("EmptyHint");
                empty.transform.SetParent(_listContent, false);
                empty.AddComponent<RectTransform>();
                Text t = empty.AddComponent<Text>();
                t.text = "No packs installed.";
                t.font = GetFont();
                t.fontSize = 15;
                t.color = new Color(_theme.Text.r, _theme.Text.g, _theme.Text.b, 0.7f);
                t.alignment = TextAnchor.MiddleCenter;
                LayoutElement le = empty.AddComponent<LayoutElement>();
                le.preferredHeight = RowHeight;
                _rows.Add(empty);
                return;
            }

            for (int i = 0; i < _packs.Count; i++)
            {
                GameObject row = CreatePackRow(_packs[i]);
                if (row != null)
                {
                    row.transform.SetParent(_listContent, false);
                    _rows.Add(row);
                }
            }
        }

        private GameObject CreatePackRow(PackDisplayInfo pack)
        {
            GameObject row = new GameObject($"QuickRow_{pack.Id}");
            row.AddComponent<RectTransform>();
            Image rowBg = row.AddComponent<Image>();
            rowBg.color = new Color(_theme.Primary.r, _theme.Primary.g, _theme.Primary.b, 0.06f);
            HorizontalLayoutGroup hl = row.AddComponent<HorizontalLayoutGroup>();
            hl.padding = new RectOffset(8, 8, 4, 4);
            hl.spacing = 8;
            hl.childForceExpandWidth = false;
            hl.childForceExpandHeight = true;
            hl.childControlWidth = true;
            hl.childControlHeight = true;
            hl.childAlignment = TextAnchor.MiddleLeft;
            LayoutElement rle = row.AddComponent<LayoutElement>();
            rle.preferredHeight = RowHeight;
            rle.flexibleWidth = 1f;

            // Pack name (themed text).
            GameObject nameGo = new GameObject("Name");
            nameGo.transform.SetParent(row.transform, false);
            nameGo.AddComponent<RectTransform>();
            Text name = nameGo.AddComponent<Text>();
            name.text = pack.Name;
            name.font = GetFont();
            name.fontSize = 16;
            name.color = _theme.Text;
            name.alignment = TextAnchor.MiddleLeft;
            name.horizontalOverflow = HorizontalWrapMode.Overflow;
            LayoutElement nameLe = nameGo.AddComponent<LayoutElement>();
            nameLe.flexibleWidth = 1f;

            // ON/OFF native-chrome toggle button.
            string packId = pack.Id;
            bool enabled = pack.IsEnabled;
            string label = enabled ? "ON" : "OFF";

            GameObject toggleBtnGo = CreateChromeButton(row.transform, label, null);
            LayoutElement tle = toggleBtnGo.GetComponent<LayoutElement>() ?? toggleBtnGo.AddComponent<LayoutElement>();
            tle.preferredWidth = 88f;
            tle.flexibleWidth = 0f;

            Button toggleBtn = toggleBtnGo.GetComponent<Button>();
            Text? toggleText = toggleBtnGo.GetComponentInChildren<Text>(true);
            ColorTintButtonLabel(toggleBtnGo, enabled ? _theme.Primary : new Color(_theme.Text.r, _theme.Text.g, _theme.Text.b, 0.55f));

            bool localState = enabled;
            if (toggleBtn != null)
            {
                toggleBtn.onClick.AddListener(() =>
                {
                    localState = !localState;
                    if (toggleText != null) toggleText.text = localState ? "ON" : "OFF";
                    ColorTintButtonLabel(toggleBtnGo, localState ? _theme.Primary : new Color(_theme.Text.r, _theme.Text.g, _theme.Text.b, 0.55f));
                    OnPackToggled?.Invoke(packId, localState);
                    LogInfo($"Quick toggle: {packId} -> {localState}");
                });
            }

            return row;
        }

        /// <summary>
        /// Creates a button that, when a native donor button is available, is a CLONE of
        /// DINO's native menu button (inheriting frame/font/sprites/visual states). When no
        /// donor is available, falls back to a themed synthetic button.
        /// </summary>
        private GameObject CreateChromeButton(Transform parent, string label, Action? onClick)
        {
            GameObject go;
            Button btn;

            if (_donorButton != null && _donorButton.gameObject != null)
            {
                // Clone the native donor so the button looks like DINO's own.
                Button clone = NativeUiHelper.CloneButton(_donorButton, label);
                go = clone.gameObject;
                go.name = $"QuickChromeBtn_{label}";
                go.transform.SetParent(parent, false);
                btn = clone;
                // Re-assert label text on all text children (clone may inherit donor label).
                foreach (Text t in go.GetComponentsInChildren<Text>(true)) t.text = label;
            }
            else
            {
                go = new GameObject($"QuickChromeBtn_{label}");
                go.transform.SetParent(parent, false);
                go.AddComponent<RectTransform>();
                Image bg = go.AddComponent<Image>();
                bg.color = new Color(_theme.Primary.r, _theme.Primary.g, _theme.Primary.b, 0.18f);
                btn = go.AddComponent<Button>();
                btn.targetGraphic = bg;

                GameObject lblGo = new GameObject("Label");
                lblGo.transform.SetParent(go.transform, false);
                RectTransform lblRt = lblGo.AddComponent<RectTransform>();
                lblRt.anchorMin = Vector2.zero;
                lblRt.anchorMax = Vector2.one;
                lblRt.offsetMin = Vector2.zero;
                lblRt.offsetMax = Vector2.zero;
                Text lbl = lblGo.AddComponent<Text>();
                lbl.text = label;
                lbl.font = GetFont();
                lbl.fontSize = 15;
                lbl.fontStyle = FontStyle.Bold;
                lbl.color = _theme.Primary;
                lbl.alignment = TextAnchor.MiddleCenter;
            }

            LayoutElement le = go.GetComponent<LayoutElement>() ?? go.AddComponent<LayoutElement>();
            le.preferredHeight = 36f;

            if (onClick != null && btn != null)
                btn.onClick.AddListener(() => onClick());

            return go;
        }

        private static void ColorTintButtonLabel(GameObject buttonGo, Color color)
        {
            foreach (Text t in buttonGo.GetComponentsInChildren<Text>(true))
                t.color = color;
        }

        private Font? _font;
        private Font GetFont()
        {
            if (_font != null) return _font;
            _font = Font.CreateDynamicFontFromOSFont("Arial", 16);
            return _font;
        }

        private void LogInfo(string message)
        {
            _log?.LogInfo($"[{Tag}] {message}");
            DebugLog.Write(Tag, message);
        }
    }
}
