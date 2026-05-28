#nullable enable
using System;
using System.Collections.Generic;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Full-screen UGUI panel for the mods menu that clones DINO's native Options canvas
    /// for styling consistency. Provides a pack list, detail pane, and back navigation.
    ///
    /// Layout:
    /// <code>
    /// ┌─────────────────────────────────────────┐
    /// │  MODS                         [Back]    │  ← Title bar
    /// ├──────────────┬──────────────────────────┤
    /// │  Pack List   │  Detail Pane             │
    /// │  (scroll)    │  Name, Version, Author   │
    /// │              │  Description, Toggle      │
    /// │              │                           │
    /// └──────────────┴──────────────────────────┘
    /// </code>
    ///
    /// Instantiated as a MonoBehaviour on a DontDestroyOnLoad GameObject.
    /// Clones DINO's Options canvas to inherit native fonts, sprites, and colors.
    /// Falls back to a dark theme when the Options canvas is not found.
    /// </summary>
    internal sealed class NativeModsPage : MonoBehaviour
    {
        // ── Fallback theme colors ────────────────────────────────────────────────
        private static readonly Color FallbackBg = UiBuilder.HexColor("#1A1A2E", 0.95f);
        private static readonly Color FallbackText = UiBuilder.HexColor("#E0E0E0", 1f);
        private static readonly Color FallbackAccent = UiBuilder.HexColor("#4ECDC4", 1f);
        private static readonly Color FallbackDarkBg = UiBuilder.HexColor("#12122A", 1f);
        private static readonly Color FallbackRowHover = UiBuilder.HexColor("#2A2A4E", 1f);
        private static readonly Color FallbackBadgeBg = UiBuilder.HexColor("#333366", 1f);

        // ── Layout constants ─────────────────────────────────────────────────────
        private const float ListWidthFraction = 0.35f;
        private const float RowHeight = 48f;
        private const float TitleBarHeight = 56f;
        private const float PaddingOuter = 12f;

        // ── Callbacks ────────────────────────────────────────────────────────────

        /// <summary>Invoked when the Back button is clicked.</summary>
        public Action? OnBackClicked;

        /// <summary>Invoked when a pack is toggled enabled/disabled (packId, isEnabled).</summary>
        public Action<string, bool>? OnPackToggled;

        /// <summary>Invoked when the user clicks Reload.</summary>
        public Action? OnReloadRequested;

        // ── State ────────────────────────────────────────────────────────────────
        private ManualLogSource? _log;
        private GameObject? _rootPanel;
        private RectTransform? _listContent;
        private GameObject? _detailPane;
        private Text? _detailName;
        private Text? _detailVersion;
        private Text? _detailAuthor;
        private Text? _detailType;
        private Text? _detailDescription;
        private IReadOnlyList<PackDisplayInfo>? _packs;
        private readonly List<GameObject> _packRows = new List<GameObject>();
        private int _selectedIndex = -1;
        private bool _visible;

        // Cloned style references (from Options canvas or fallback)
        private Font? _font;
        private Color _bgColor = FallbackBg;
        private Color _textColor = FallbackText;
        private Color _accentColor = FallbackAccent;

        // References to main menu content to hide/show
        private Canvas? _mainMenuCanvas;

        // ── Public API ───────────────────────────────────────────────────────────

        /// <summary>
        /// Sets the BepInEx logger.
        /// </summary>
        public void SetLogger(ManualLogSource log)
        {
            _log = log;
        }

        /// <summary>
        /// Populates the pack list with the given packs.
        /// </summary>
        public void SetPacks(IReadOnlyList<PackDisplayInfo> packs)
        {
            _packs = packs;
            if (_rootPanel != null)
            {
                RebuildPackList();
            }
        }

        /// <summary>
        /// Shows the mods page and hides the main menu content.
        /// </summary>
        public void Show()
        {
            if (_rootPanel == null)
            {
                BuildPanel();
            }

            if (_rootPanel != null)
            {
                _rootPanel.SetActive(true);
                _visible = true;
            }

            // Hide main menu content
            if (_mainMenuCanvas != null)
            {
                // Disable the main menu canvas rendering but keep it alive
                _mainMenuCanvas.enabled = false;
            }

            _log?.LogInfo("[NativeModsPage] Show()");
            DebugLog.Write("NativeModsPage", "Shown");
        }

        /// <summary>
        /// Hides the mods page and shows the main menu content.
        /// </summary>
        public void Hide()
        {
            if (_rootPanel != null)
            {
                _rootPanel.SetActive(false);
                _visible = false;
            }

            // Restore main menu content
            if (_mainMenuCanvas != null)
            {
                _mainMenuCanvas.enabled = true;
            }

            _log?.LogInfo("[NativeModsPage] Hide()");
            DebugLog.Write("NativeModsPage", "Hidden");
        }

        /// <summary>Whether the page is currently visible.</summary>
        public bool IsVisible => _visible;

        // ── Panel construction ───────────────────────────────────────────────────

        private void BuildPanel()
        {
            try
            {
                // Try to clone style from DINO's Options canvas
                CloneStyleFromOptionsCanvas();

                // Resolve fallback font
                if (_font == null)
                {
                    _font = Font.CreateDynamicFontFromOSFont("Arial", 16);
                }

                // Create root panel
                _rootPanel = new GameObject("DINOForge_NativeModsPage", typeof(RectTransform), typeof(Canvas), typeof(CanvasScaler), typeof(GraphicRaycaster));
                _rootPanel.transform.SetParent(transform, false);
                DontDestroyOnLoad(_rootPanel);

                Canvas canvas = _rootPanel.GetComponent<Canvas>();
                canvas.renderMode = RenderMode.ScreenSpaceOverlay;
                canvas.overrideSorting = true;
                canvas.sortingOrder = 32766; // just below DFCanvas

                CanvasScaler scaler = _rootPanel.GetComponent<CanvasScaler>();
                scaler.uiScaleMode = CanvasScaler.ScaleMode.ScaleWithScreenSize;
                scaler.referenceResolution = new Vector2(1920f, 1080f);
                scaler.matchWidthOrHeight = 0.5f;

                // Pattern #235: ensure EventSystem exists before adding GraphicRaycaster
                Plugin.EnsureEventSystemAlive();

                RectTransform rootRt = _rootPanel.GetComponent<RectTransform>();
                UiBuilder.FillParent(rootRt);

                // Background
                GameObject bgGo = new GameObject("Background", typeof(RectTransform), typeof(Image));
                bgGo.transform.SetParent(_rootPanel.transform, false);
                Image bgImg = bgGo.GetComponent<Image>();
                bgImg.color = _bgColor;
                bgImg.raycastTarget = true;
                UiBuilder.FillParent(bgGo.GetComponent<RectTransform>());

                // Title bar
                BuildTitleBar(_rootPanel.transform);

                // Content area (below title bar)
                GameObject contentGo = new GameObject("ContentArea", typeof(RectTransform));
                contentGo.transform.SetParent(_rootPanel.transform, false);
                RectTransform contentRt = contentGo.GetComponent<RectTransform>();
                contentRt.anchorMin = Vector2.zero;
                contentRt.anchorMax = Vector2.one;
                contentRt.offsetMin = new Vector2(PaddingOuter, PaddingOuter);
                contentRt.offsetMax = new Vector2(-PaddingOuter, -(TitleBarHeight + PaddingOuter));

                // Left panel: scrollable pack list
                BuildPackListPanel(contentRt);

                // Right panel: detail pane
                BuildDetailPane(contentRt);

                _rootPanel.SetActive(false);

                if (_packs != null && _packs.Count > 0)
                {
                    RebuildPackList();
                }

                _log?.LogInfo("[NativeModsPage] Panel built successfully.");
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[NativeModsPage] BuildPanel failed: {ex}");
                DebugLog.Write("NativeModsPage", $"BuildPanel exception: {ex.Message}");
            }
        }

        private void CloneStyleFromOptionsCanvas()
        {
            try
            {
                Canvas[] allCanvases = Resources.FindObjectsOfTypeAll<Canvas>();
                Canvas? optionsCanvas = null;

                foreach (Canvas c in allCanvases)
                {
                    if (c == null) continue;
                    if (c.name != null && c.name.IndexOf("Options", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        optionsCanvas = c;
                        break;
                    }
                }

                if (optionsCanvas == null)
                {
                    _log?.LogInfo("[NativeModsPage] Options canvas not found — using fallback dark theme.");
                    return;
                }

                _log?.LogInfo($"[NativeModsPage] Cloning style from Options canvas: '{optionsCanvas.name}'");
                _mainMenuCanvas = FindMainMenuCanvas();

                // Extract font from first Text component found
                Text[] texts = optionsCanvas.GetComponentsInChildren<Text>(true);
                if (texts.Length > 0 && texts[0].font != null)
                {
                    _font = texts[0].font;
                    _textColor = texts[0].color;
                    _log?.LogInfo($"[NativeModsPage] Cloned font: '{_font.name}', text color: {ColorUtility.ToHtmlStringRGBA(_textColor)}");
                }

                // Try TMP font via reflection as well
                if (_font == null)
                {
                    Type? tmpType = Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro");
                    if (tmpType != null)
                    {
                        Component? tmpComp = optionsCanvas.GetComponentInChildren(tmpType, true);
                        if (tmpComp != null)
                        {
                            System.Reflection.PropertyInfo? colorProp = tmpType.GetProperty("color");
                            object? colorObj = colorProp?.GetValue(tmpComp);
                            if (colorObj is Color c)
                            {
                                _textColor = c;
                            }
                        }
                    }
                }

                // Extract background color from largest Image
                Image[] images = optionsCanvas.GetComponentsInChildren<Image>(true);
                Image? largest = null;
                float largestArea = 0f;
                foreach (Image img in images)
                {
                    if (img == null) continue;
                    RectTransform? rt = img.GetComponent<RectTransform>();
                    if (rt == null) continue;
                    float area = rt.rect.width * rt.rect.height;
                    if (area > largestArea)
                    {
                        largestArea = area;
                        largest = img;
                    }
                }
                if (largest != null)
                {
                    _bgColor = largest.color;
                    _bgColor.a = 0.95f;
                    _log?.LogInfo($"[NativeModsPage] Cloned bg color: {ColorUtility.ToHtmlStringRGBA(_bgColor)}");
                }

                // Extract accent from first Button/Selectable
                Selectable[] selectables = optionsCanvas.GetComponentsInChildren<Selectable>(true);
                if (selectables.Length > 0)
                {
                    _accentColor = selectables[0].colors.highlightedColor;
                    _log?.LogInfo($"[NativeModsPage] Cloned accent color: {ColorUtility.ToHtmlStringRGBA(_accentColor)}");
                }
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[NativeModsPage] CloneStyleFromOptionsCanvas failed: {ex.Message}");
            }
        }

        private Canvas? FindMainMenuCanvas()
        {
            Canvas[] allCanvases = Resources.FindObjectsOfTypeAll<Canvas>();
            foreach (Canvas c in allCanvases)
            {
                if (c == null) continue;
                if (c.name != null && c.name.IndexOf("MainMenu", StringComparison.OrdinalIgnoreCase) >= 0
                    && c.gameObject.activeInHierarchy)
                {
                    return c;
                }
            }
            return null;
        }

        // ── Title bar ────────────────────────────────────────────────────────────

        private void BuildTitleBar(Transform parent)
        {
            GameObject barGo = new GameObject("TitleBar", typeof(RectTransform), typeof(Image));
            barGo.transform.SetParent(parent, false);

            Image barImg = barGo.GetComponent<Image>();
            barImg.color = new Color(_bgColor.r * 0.7f, _bgColor.g * 0.7f, _bgColor.b * 0.7f, 0.98f);
            barImg.raycastTarget = false;

            RectTransform barRt = barGo.GetComponent<RectTransform>();
            barRt.anchorMin = new Vector2(0f, 1f);
            barRt.anchorMax = Vector2.one;
            barRt.pivot = new Vector2(0.5f, 1f);
            barRt.sizeDelta = new Vector2(0f, TitleBarHeight);
            barRt.offsetMin = new Vector2(0f, barRt.offsetMin.y);
            barRt.offsetMax = new Vector2(0f, 0f);

            HorizontalLayoutGroup hlg = barGo.AddComponent<HorizontalLayoutGroup>();
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;
            hlg.spacing = 8f;
            hlg.padding = new RectOffset(16, 16, 6, 6);
            hlg.childAlignment = TextAnchor.MiddleLeft;

            // Title text
            Text titleText = CreateText(barGo.transform, "TitleText", "MODS", 28, _textColor, FontStyle.Bold);
            LayoutElement titleLe = titleText.gameObject.AddComponent<LayoutElement>();
            titleLe.flexibleWidth = 1f;

            // Back button
            GameObject backGo = CreateButtonGo(barGo.transform, "BackButton", "Back", () =>
            {
                OnBackClicked?.Invoke();
            });
            LayoutElement backLe = backGo.AddComponent<LayoutElement>();
            backLe.preferredWidth = 100f;
            backLe.preferredHeight = 36f;
        }

        // ── Pack list panel ──────────────────────────────────────────────────────

        private void BuildPackListPanel(RectTransform contentParent)
        {
            GameObject listPanelGo = new GameObject("PackListPanel", typeof(RectTransform), typeof(Image));
            listPanelGo.transform.SetParent(contentParent, false);

            Image listBg = listPanelGo.GetComponent<Image>();
            listBg.color = FallbackDarkBg;
            listBg.raycastTarget = true;

            RectTransform listRt = listPanelGo.GetComponent<RectTransform>();
            listRt.anchorMin = Vector2.zero;
            listRt.anchorMax = new Vector2(ListWidthFraction, 1f);
            listRt.offsetMin = Vector2.zero;
            listRt.offsetMax = new Vector2(-4f, 0f);

            // Scroll view
            (ScrollRect scrollRect, RectTransform scrollContent) = UiBuilder.MakeScrollView(
                listPanelGo.transform, "PackListScroll", Vector2.zero);

            RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
            UiBuilder.FillParent(scrollRt);
            scrollRt.offsetMin = new Vector2(4f, 4f);
            scrollRt.offsetMax = new Vector2(-4f, -4f);

            _listContent = scrollContent;
        }

        // ── Detail pane ──────────────────────────────────────────────────────────

        private void BuildDetailPane(RectTransform contentParent)
        {
            GameObject detailGo = new GameObject("DetailPane", typeof(RectTransform), typeof(Image));
            detailGo.transform.SetParent(contentParent, false);

            Image detailBg = detailGo.GetComponent<Image>();
            detailBg.color = FallbackDarkBg;
            detailBg.raycastTarget = false;

            RectTransform detailRt = detailGo.GetComponent<RectTransform>();
            detailRt.anchorMin = new Vector2(ListWidthFraction, 0f);
            detailRt.anchorMax = Vector2.one;
            detailRt.offsetMin = new Vector2(4f, 0f);
            detailRt.offsetMax = Vector2.zero;

            _detailPane = detailGo;

            // Vertical layout for detail fields
            VerticalLayoutGroup vlg = detailGo.AddComponent<VerticalLayoutGroup>();
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;
            vlg.spacing = 8f;
            vlg.padding = new RectOffset(16, 16, 16, 16);

            _detailName = CreateText(detailGo.transform, "DetailName", "", 24, _textColor, FontStyle.Bold);
            _detailVersion = CreateText(detailGo.transform, "DetailVersion", "", 16, _textColor);
            _detailAuthor = CreateText(detailGo.transform, "DetailAuthor", "", 16, _textColor);
            _detailType = CreateText(detailGo.transform, "DetailType", "", 16, _accentColor);

            // Separator
            UiBuilder.MakeHorizontalSeparator(detailGo.transform, new Color(_textColor.r, _textColor.g, _textColor.b, 0.3f));

            _detailDescription = CreateText(detailGo.transform, "DetailDescription", "Select a pack to view details.", 14, _textColor);
            _detailDescription.horizontalOverflow = HorizontalWrapMode.Wrap;
            _detailDescription.verticalOverflow = VerticalWrapMode.Overflow;

            // Reload button at bottom
            GameObject reloadGo = CreateButtonGo(detailGo.transform, "ReloadButton", "Reload Packs", () =>
            {
                OnReloadRequested?.Invoke();
            });
            LayoutElement reloadLe = reloadGo.AddComponent<LayoutElement>();
            reloadLe.preferredHeight = 36f;
        }

        // ── Pack list population ─────────────────────────────────────────────────

        private void RebuildPackList()
        {
            if (_listContent == null || _packs == null) return;

            // Clear existing rows
            foreach (GameObject row in _packRows)
            {
                if (row != null) Destroy(row);
            }
            _packRows.Clear();
            _selectedIndex = -1;

            for (int i = 0; i < _packs.Count; i++)
            {
                PackDisplayInfo pack = _packs[i];
                int capturedIndex = i;

                GameObject rowGo = new GameObject($"PackRow_{pack.Id}", typeof(RectTransform), typeof(Image));
                rowGo.transform.SetParent(_listContent, false);

                Image rowBg = rowGo.GetComponent<Image>();
                rowBg.color = (i % 2 == 0) ? FallbackDarkBg : FallbackRowHover;
                rowBg.raycastTarget = true;

                LayoutElement rowLe = rowGo.AddComponent<LayoutElement>();
                rowLe.preferredHeight = RowHeight;
                rowLe.flexibleWidth = 1f;

                HorizontalLayoutGroup rowHlg = rowGo.AddComponent<HorizontalLayoutGroup>();
                rowHlg.childForceExpandWidth = false;
                rowHlg.childForceExpandHeight = true;
                rowHlg.spacing = 6f;
                rowHlg.padding = new RectOffset(8, 8, 4, 4);
                rowHlg.childAlignment = TextAnchor.MiddleLeft;

                // Enable/disable toggle
                Toggle toggle = UiBuilder.MakeToggle(rowGo.transform, $"Toggle_{pack.Id}", pack.IsEnabled, (bool enabled) =>
                {
                    OnPackToggled?.Invoke(pack.Id, enabled);
                });
                LayoutElement toggleLe = toggle.gameObject.AddComponent<LayoutElement>();
                toggleLe.preferredWidth = 24f;
                toggleLe.preferredHeight = 24f;

                // Pack name (clickable to select)
                GameObject nameButtonGo = CreateButtonGo(rowGo.transform, $"Name_{pack.Id}", pack.Name, () =>
                {
                    SelectPack(capturedIndex);
                });
                LayoutElement nameLe = nameButtonGo.AddComponent<LayoutElement>();
                nameLe.flexibleWidth = 1f;

                // Version text
                Text versionText = CreateText(rowGo.transform, $"Version_{pack.Id}", pack.Version, 12, new Color(_textColor.r, _textColor.g, _textColor.b, 0.6f));
                LayoutElement versionLe = versionText.gameObject.AddComponent<LayoutElement>();
                versionLe.preferredWidth = 50f;

                // Type badge
                GameObject badgeGo = new GameObject($"Badge_{pack.Id}", typeof(RectTransform), typeof(Image));
                badgeGo.transform.SetParent(rowGo.transform, false);
                Image badgeBg = badgeGo.GetComponent<Image>();
                badgeBg.color = FallbackBadgeBg;

                LayoutElement badgeLe = badgeGo.AddComponent<LayoutElement>();
                badgeLe.preferredWidth = 70f;
                badgeLe.preferredHeight = 20f;

                Text badgeText = CreateText(badgeGo.transform, "BadgeText", pack.Type, 10, _accentColor, FontStyle.Normal, TextAnchor.MiddleCenter);
                UiBuilder.FillParent(badgeText.GetComponent<RectTransform>());

                _packRows.Add(rowGo);
            }
        }

        private void SelectPack(int index)
        {
            if (_packs == null || index < 0 || index >= _packs.Count) return;
            _selectedIndex = index;
            PackDisplayInfo pack = _packs[index];

            if (_detailName != null) _detailName.text = pack.Name;
            if (_detailVersion != null) _detailVersion.text = $"Version: {pack.Version}";
            if (_detailAuthor != null) _detailAuthor.text = $"Author: {pack.Author}";
            if (_detailType != null) _detailType.text = $"Type: {pack.Type}";
            if (_detailDescription != null)
            {
                _detailDescription.text = !string.IsNullOrEmpty(pack.Description)
                    ? pack.Description
                    : "(No description provided.)";
            }

            _log?.LogInfo($"[NativeModsPage] Selected pack: {pack.Id} ({pack.Name})");
        }

        // ── UI factory helpers ───────────────────────────────────────────────────

        private Text CreateText(
            Transform parent,
            string name,
            string text,
            int fontSize,
            Color color,
            FontStyle style = FontStyle.Normal,
            TextAnchor alignment = TextAnchor.MiddleLeft)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Text));
            go.transform.SetParent(parent, false);

            RectTransform rt = go.GetComponent<RectTransform>();
            rt.sizeDelta = new Vector2(200f, fontSize + 8f);

            Text t = go.GetComponent<Text>();
            t.text = text;
            t.fontSize = fontSize;
            t.color = color;
            t.alignment = alignment;
            t.fontStyle = style;
            t.supportRichText = true;
            t.horizontalOverflow = HorizontalWrapMode.Wrap;
            t.verticalOverflow = VerticalWrapMode.Truncate;

            if (_font != null)
            {
                t.font = _font;
            }
            else
            {
                // Fallback: create dynamic OS font
                Font? fallbackFont = Font.CreateDynamicFontFromOSFont("Arial", fontSize);
                if (fallbackFont != null)
                    t.font = fallbackFont;
            }

            return t;
        }

        private GameObject CreateButtonGo(Transform parent, string name, string label, Action onClick)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image), typeof(Button));
            go.transform.SetParent(parent, false);

            Image img = go.GetComponent<Image>();
            img.color = _accentColor;
            img.raycastTarget = true;

            Button btn = go.GetComponent<Button>();
            ColorBlock cb = btn.colors;
            cb.normalColor = _accentColor;
            cb.highlightedColor = Color.Lerp(_accentColor, Color.white, 0.2f);
            cb.pressedColor = Color.Lerp(_accentColor, Color.black, 0.3f);
            cb.selectedColor = _accentColor;
            cb.fadeDuration = 0.1f;
            btn.colors = cb;

            btn.onClick.AddListener(() => onClick());

            Text txt = CreateText(go.transform, "Label", label, 14, _textColor, FontStyle.Bold, TextAnchor.MiddleCenter);
            UiBuilder.FillParent(txt.GetComponent<RectTransform>());

            return go;
        }
    }
}
