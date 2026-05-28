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
    /// Full-screen native mods page that clones DINO's Options canvas to inherit
    /// native styling (fonts, sprites, colors, layout). Replaces the floating UGUI
    /// overlay with a settings-style page that hides the main menu and shows a
    /// pack browser with detail pane.
    ///
    /// Strategy:
    ///   1. Find the 'Options' Canvas in the scene (DINO's settings panel).
    ///   2. Clone its root GameObject to inherit background, frame, fonts.
    ///   3. Strip game-specific content children (sliders, tabs, etc.).
    ///   4. Populate with our mod list content inside the cloned shell.
    ///   5. Steal TMP_Text fonts from the cloned children before stripping.
    ///
    /// Fallback: If the Options canvas is not found, builds UI from scratch
    /// using fonts harvested from any available native text component.
    /// </summary>
    public sealed class NativeModsPage : MonoBehaviour
    {
        private const string Tag = "NativeModsPage";

        // ── State ──────────────────────────────────────────────────────────────
        private ManualLogSource? _log;
        private IReadOnlyList<PackDisplayInfo> _packs = Array.Empty<PackDisplayInfo>();
        private int _selectedIndex = -1;

        // ── Callbacks ──────────────────────────────────────────────────────────
        /// <summary>Called when user toggles a pack (packId, isEnabled).</summary>
        public Action<string, bool>? OnPackToggled { get; set; }

        /// <summary>Called when user requests pack reload.</summary>
        public Action? OnReloadRequested { get; set; }

        /// <summary>Called when user presses Back to return to the main menu.</summary>
        public Action? OnBackPressed { get; set; }

        // ── References ────────────────────────────────────────────────────────
        private GameObject? _mainMenuContent;

        // ── Harvested native assets ───────────────────────────────────────────
        private Font? _nativeFont;
        private Sprite? _nativeBgSprite;
        private Color _nativeBgColor = new Color(0.102f, 0.102f, 0.180f, 1f);
        private Color _nativeTextColor = new Color(0.878f, 0.878f, 0.878f, 1f);
        private Color _nativeButtonBgColor = new Color(0.2f, 0.2f, 0.3f, 1f);
        // TMP font asset reference (stored as object to avoid compile-time TMPro dependency)
        private object? _nativeTmpFontAsset;
        private Type? _tmpTextType;
        private Type? _tmpFontAssetType;

        // ── Palette (overwritten by harvested native colors when available) ───
        private static readonly Color AccentColor = new Color(0.306f, 0.804f, 0.769f, 1f);    // #4ECDC4
        private static readonly Color SelectedColor = new Color(0.173f, 0.243f, 0.314f, 1f);  // #2C3E50
        private static readonly Color ErrorColor = new Color(0.9f, 0.3f, 0.3f, 1f);
        private static readonly Color DisabledColor = new Color(0.5f, 0.5f, 0.5f, 1f);
        private static readonly Color EnabledBadgeColor = new Color(0.3f, 0.8f, 0.4f, 1f);

        // ── UI hierarchy built at Show time ────────────────────────────────────
        private GameObject? _root;
        private RectTransform? _packListContent;
        private readonly List<GameObject> _packRows = new List<GameObject>();

        // Detail pane elements (stored as Component to support both Text and TMP_Text)
        private Component? _detailTitle;
        private Component? _detailVersion;
        private Component? _detailAuthor;
        private Component? _detailType;
        private Component? _detailDescription;
        private Component? _detailContent;
        private Component? _detailErrors;
        private GameObject? _detailPanel;

        // ── Public API ─────────────────────────────────────────────────────────

        public void SetLogger(ManualLogSource log) => _log = log;

        /// <summary>
        /// Updates the pack list displayed by the page.
        /// If visible, rebuilds the pack list UI immediately.
        /// </summary>
        public void SetPacks(IReadOnlyList<PackDisplayInfo> packs)
        {
            _packs = packs ?? Array.Empty<PackDisplayInfo>();
            if (_root != null && _root.activeSelf)
            {
                RebuildPackList();
                RefreshDetailPane();
            }
        }

        /// <summary>
        /// Shows the mods page, hiding the main menu content.
        /// </summary>
        public void Show(Canvas mainMenuCanvas, GameObject? mainMenuContent)
        {
            _mainMenuContent = mainMenuContent;

            if (_mainMenuContent != null)
                _mainMenuContent.SetActive(false);

            if (_root == null)
                BuildUI(mainMenuCanvas);

            _root!.SetActive(true);
            RebuildPackList();
            RefreshDetailPane();

            LogInfo("NativeModsPage shown");
        }

        /// <summary>Hides the mods page and re-shows the main menu content.</summary>
        public void Hide()
        {
            if (_root != null)
                _root.SetActive(false);

            if (_mainMenuContent != null)
                _mainMenuContent.SetActive(true);

            LogInfo("NativeModsPage hidden");
        }

        /// <summary>Whether the page is currently visible.</summary>
        public bool IsVisible => _root != null && _root.activeSelf;

        // ── UI Construction ────────────────────────────────────────────────────

        private void BuildUI(Canvas mainMenuCanvas)
        {
            // Resolve TMP types once
            ResolveTmpTypes();

            // Try to find and clone the Options canvas for native look and feel
            Canvas? optionsCanvas = FindOptionsCanvas();

            if (optionsCanvas != null)
            {
                LogInfo($"Found Options canvas '{optionsCanvas.name}' — cloning for native styling");
                BuildFromClonedOptionsCanvas(optionsCanvas, mainMenuCanvas.transform);
            }
            else
            {
                LogInfo("Options canvas not found — building UI with harvested native fonts");
                HarvestNativeAssets(mainMenuCanvas);
                BuildFromScratch(mainMenuCanvas.transform);
            }
        }

        /// <summary>
        /// Finds the 'Options' canvas in the scene. DINO has a top-level Canvas
        /// named 'Options' that holds the settings panel.
        /// </summary>
        private Canvas? FindOptionsCanvas()
        {
            Canvas[] allCanvases = Resources.FindObjectsOfTypeAll<Canvas>();
            foreach (Canvas c in allCanvases)
            {
                if (c == null) continue;
                if (string.Equals(c.name, "Options", StringComparison.OrdinalIgnoreCase))
                    return c;
            }
            return null;
        }

        /// <summary>
        /// Clones the Options canvas, strips its content, and populates with mod list.
        /// Harvests fonts and sprites from the native elements before stripping.
        /// </summary>
        private void BuildFromClonedOptionsCanvas(Canvas optionsCanvas, Transform parentTransform)
        {
            // 1. Harvest native fonts and sprites from the Options canvas BEFORE cloning
            HarvestNativeAssets(optionsCanvas);

            // 2. Clone the entire Options canvas GameObject
            GameObject clone = UnityEngine.Object.Instantiate(optionsCanvas.gameObject, parentTransform);
            clone.name = "DINOForge_NativeModsPage";

            // 3. Remove the Canvas component from the clone (we're parenting into MainMenu canvas)
            Canvas? cloneCanvas = clone.GetComponent<Canvas>();
            if (cloneCanvas != null)
                UnityEngine.Object.Destroy(cloneCanvas);

            // Remove CanvasScaler and GraphicRaycaster from the clone since the parent canvas provides these
            CanvasScaler? scaler = clone.GetComponent<CanvasScaler>();
            if (scaler != null) UnityEngine.Object.Destroy(scaler);

            GraphicRaycaster? raycaster = clone.GetComponent<GraphicRaycaster>();
            if (raycaster != null) UnityEngine.Object.Destroy(raycaster);

            // 4. Strip all game-specific MonoBehaviours from the clone
            NativeUiHelper.SanitizeUiClone(clone);

            // 5. Strip all child content but keep the root's RectTransform, Image, etc.
            StripChildContent(clone.transform);

            // 6. Set up as full-screen panel inside the MainMenu canvas
            RectTransform rootRt = clone.GetComponent<RectTransform>();
            if (rootRt == null)
                rootRt = clone.AddComponent<RectTransform>();
            rootRt.anchorMin = Vector2.zero;
            rootRt.anchorMax = Vector2.one;
            rootRt.offsetMin = Vector2.zero;
            rootRt.offsetMax = Vector2.zero;

            // Ensure background image exists
            Image? rootBg = clone.GetComponent<Image>();
            if (rootBg == null)
                rootBg = clone.AddComponent<Image>();

            // Use native background sprite if available, otherwise use harvested color
            if (_nativeBgSprite != null)
            {
                rootBg.sprite = _nativeBgSprite;
                rootBg.type = Image.Type.Sliced;
            }
            rootBg.color = _nativeBgColor;

            // 7. Add our layout and content
            VerticalLayoutGroup rootLayout = clone.AddComponent<VerticalLayoutGroup>();
            rootLayout.padding = new RectOffset(20, 20, 10, 10);
            rootLayout.spacing = 10;
            rootLayout.childForceExpandWidth = true;
            rootLayout.childForceExpandHeight = false;
            rootLayout.childControlWidth = true;
            rootLayout.childControlHeight = true;

            _root = clone;

            BuildTitleBar(_root.transform);
            BuildBody(_root.transform);

            LogInfo("Built mods page from cloned Options canvas");
        }

        /// <summary>
        /// Fallback: builds the UI from scratch using harvested native fonts.
        /// </summary>
        private void BuildFromScratch(Transform parent)
        {
            _root = CreatePanel("DINOForge_NativeModsPage", parent);
            RectTransform rootRt = _root.GetComponent<RectTransform>();
            rootRt.anchorMin = Vector2.zero;
            rootRt.anchorMax = Vector2.one;
            rootRt.offsetMin = Vector2.zero;
            rootRt.offsetMax = Vector2.zero;

            Image rootBg = _root.GetComponent<Image>();
            if (_nativeBgSprite != null)
            {
                rootBg.sprite = _nativeBgSprite;
                rootBg.type = Image.Type.Sliced;
            }
            rootBg.color = _nativeBgColor;

            VerticalLayoutGroup rootLayout = _root.AddComponent<VerticalLayoutGroup>();
            rootLayout.padding = new RectOffset(20, 20, 10, 10);
            rootLayout.spacing = 10;
            rootLayout.childForceExpandWidth = true;
            rootLayout.childForceExpandHeight = false;
            rootLayout.childControlWidth = true;
            rootLayout.childControlHeight = true;

            BuildTitleBar(_root.transform);
            BuildBody(_root.transform);

            LogInfo("Built mods page from scratch with harvested native fonts");
        }

        // ── Native asset harvesting ───────────────────────────────────────────

        /// <summary>
        /// Harvests fonts, sprites, and colors from the given canvas or any
        /// available native UI elements. This lets us match DINO's visual style
        /// even when building UI from scratch.
        /// </summary>
        private void HarvestNativeAssets(Canvas canvas)
        {
            try
            {
                HarvestNativeAssets(canvas.transform);
            }
            catch (Exception ex)
            {
                LogInfo($"HarvestNativeAssets exception (non-fatal): {ex.Message}");
            }
        }

        private void HarvestNativeAssets(Transform root)
        {
            // Harvest legacy font from any Text component
            if (_nativeFont == null)
            {
                Text[] texts = root.GetComponentsInChildren<Text>(true);
                foreach (Text t in texts)
                {
                    if (t != null && t.font != null)
                    {
                        _nativeFont = t.font;
                        _nativeTextColor = t.color;
                        LogInfo($"Harvested native Font: '{_nativeFont.name}'");
                        break;
                    }
                }
            }

            // Harvest TMP font from TMP_Text components via reflection
            if (_nativeTmpFontAsset == null && _tmpTextType != null)
            {
                Component[] allComponents = root.GetComponentsInChildren<Component>(true);
                foreach (Component c in allComponents)
                {
                    if (c == null) continue;
                    Type cType = c.GetType();
                    if (!cType.FullName.StartsWith("TMPro.")) continue;

                    try
                    {
                        var fontProp = cType.GetProperty("font");
                        object? fontAsset = fontProp?.GetValue(c);
                        if (fontAsset != null)
                        {
                            _nativeTmpFontAsset = fontAsset;
                            var colorProp = cType.GetProperty("color");
                            object? colorVal = colorProp?.GetValue(c);
                            if (colorVal is Color col)
                                _nativeTextColor = col;
                            LogInfo($"Harvested native TMP font asset: {fontAsset}");
                            break;
                        }
                    }
                    catch { /* safe-swallow: TMP reflection is best-effort */ }
                }
            }

            // Harvest background sprite from the largest Image
            Image[] images = root.GetComponentsInChildren<Image>(true);
            float largestArea = 0f;
            foreach (Image img in images)
            {
                if (img == null || img.sprite == null) continue;
                RectTransform rt = img.GetComponent<RectTransform>();
                if (rt == null) continue;
                float area = Mathf.Abs(rt.rect.width * rt.rect.height);
                if (area > largestArea)
                {
                    largestArea = area;
                    _nativeBgSprite = img.sprite;
                    _nativeBgColor = img.color;
                }
            }

            // Harvest button background color from any Button Image
            Button[] buttons = root.GetComponentsInChildren<Button>(true);
            foreach (Button btn in buttons)
            {
                if (btn == null) continue;
                Image? btnImg = btn.targetGraphic as Image;
                if (btnImg != null)
                {
                    _nativeButtonBgColor = btnImg.color;
                    break;
                }
            }

            // Fallback font: search ALL canvases if we still don't have one
            if (_nativeFont == null)
            {
                Canvas[] allCanvases = Resources.FindObjectsOfTypeAll<Canvas>();
                foreach (Canvas c in allCanvases)
                {
                    if (c == null) continue;
                    Text[] texts = c.GetComponentsInChildren<Text>(true);
                    foreach (Text t in texts)
                    {
                        if (t != null && t.font != null)
                        {
                            _nativeFont = t.font;
                            LogInfo($"Harvested fallback native Font from canvas '{c.name}': '{_nativeFont.name}'");
                            break;
                        }
                    }
                    if (_nativeFont != null) break;
                }
            }
        }

        /// <summary>
        /// Strips all children of the given transform, destroying them.
        /// Used after cloning the Options canvas to remove settings-specific controls.
        /// </summary>
        private static void StripChildContent(Transform parent)
        {
            // Destroy in reverse order to avoid index shifting issues
            for (int i = parent.childCount - 1; i >= 0; i--)
            {
                UnityEngine.Object.Destroy(parent.GetChild(i).gameObject);
            }
        }

        private void ResolveTmpTypes()
        {
            _tmpTextType = Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro")
                        ?? Type.GetType("TMPro.TMP_Text, Assembly-CSharp");
            _tmpFontAssetType = Type.GetType("TMPro.TMP_FontAsset, Unity.TextMeshPro")
                             ?? Type.GetType("TMPro.TMP_FontAsset, Assembly-CSharp");
        }

        // ── Title bar ─────────────────────────────────────────────────────────

        private void BuildTitleBar(Transform parent)
        {
            GameObject titleBar = CreatePanel("TitleBar", parent);
            Image titleBg = titleBar.GetComponent<Image>();
            titleBg.color = new Color(_nativeBgColor.r * 0.8f, _nativeBgColor.g * 0.8f, _nativeBgColor.b * 0.8f, 0.95f);

            HorizontalLayoutGroup titleLayout = titleBar.AddComponent<HorizontalLayoutGroup>();
            titleLayout.padding = new RectOffset(16, 16, 8, 8);
            titleLayout.spacing = 16;
            titleLayout.childForceExpandWidth = false;
            titleLayout.childForceExpandHeight = true;
            titleLayout.childControlWidth = true;
            titleLayout.childControlHeight = true;
            titleLayout.childAlignment = TextAnchor.MiddleLeft;

            LayoutElement titleLe = titleBar.AddComponent<LayoutElement>();
            titleLe.preferredHeight = 50;
            titleLe.flexibleHeight = 0;

            // Back button
            GameObject backBtn = CreateNativeButton("BackButton", titleBar.transform, "< Back", () =>
            {
                Hide();
                OnBackPressed?.Invoke();
            });
            LayoutElement backLe = backBtn.AddComponent<LayoutElement>();
            backLe.preferredWidth = 100;
            backLe.preferredHeight = 36;

            // Title text
            GameObject titleObj = CreateNativeTextObj("TitleText", titleBar.transform, "MODS", 24, AccentColor);
            LayoutElement titleTextLe = titleObj.AddComponent<LayoutElement>();
            titleTextLe.flexibleWidth = 1;

            // Reload button
            GameObject reloadBtn = CreateNativeButton("ReloadButton", titleBar.transform, "Reload Packs", () =>
            {
                OnReloadRequested?.Invoke();
            });
            LayoutElement reloadLe = reloadBtn.AddComponent<LayoutElement>();
            reloadLe.preferredWidth = 130;
            reloadLe.preferredHeight = 36;
        }

        // ── Body ──────────────────────────────────────────────────────────────

        private void BuildBody(Transform parent)
        {
            GameObject body = CreatePanel("Body", parent);
            Image bodyBg = body.GetComponent<Image>();
            bodyBg.color = new Color(0, 0, 0, 0); // transparent

            HorizontalLayoutGroup bodyLayout = body.AddComponent<HorizontalLayoutGroup>();
            bodyLayout.spacing = 12;
            bodyLayout.childForceExpandWidth = false;
            bodyLayout.childForceExpandHeight = true;
            bodyLayout.childControlWidth = true;
            bodyLayout.childControlHeight = true;

            LayoutElement bodyLe = body.AddComponent<LayoutElement>();
            bodyLe.flexibleHeight = 1;

            BuildPackListPanel(body.transform);
            BuildDetailPanel(body.transform);
        }

        private void BuildPackListPanel(Transform parent)
        {
            GameObject panel = CreatePanel("PackListPanel", parent);
            Image panelBg = panel.GetComponent<Image>();
            panelBg.color = new Color(_nativeBgColor.r * 1.1f, _nativeBgColor.g * 1.1f, _nativeBgColor.b * 1.1f, 0.9f);
            if (_nativeBgSprite != null)
            {
                panelBg.sprite = _nativeBgSprite;
                panelBg.type = Image.Type.Sliced;
            }

            VerticalLayoutGroup panelLayout = panel.AddComponent<VerticalLayoutGroup>();
            panelLayout.padding = new RectOffset(8, 8, 8, 8);
            panelLayout.spacing = 4;
            panelLayout.childForceExpandWidth = true;
            panelLayout.childForceExpandHeight = false;
            panelLayout.childControlWidth = true;
            panelLayout.childControlHeight = true;

            LayoutElement panelLe = panel.AddComponent<LayoutElement>();
            panelLe.preferredWidth = 360;
            panelLe.flexibleWidth = 0.4f;
            panelLe.flexibleHeight = 1;

            CreateNativeTextObj("PackListHeader", panel.transform, "Installed Packs", 18, AccentColor);

            // Scroll view
            GameObject scrollView = CreateScrollView("PackListScroll", panel.transform);
            LayoutElement scrollLe = scrollView.AddComponent<LayoutElement>();
            scrollLe.flexibleHeight = 1;
            scrollLe.flexibleWidth = 1;

            Transform viewport = scrollView.transform.Find("Viewport");
            if (viewport != null)
            {
                Transform content = viewport.Find("Content");
                if (content != null)
                    _packListContent = content.GetComponent<RectTransform>();
            }
        }

        private void BuildDetailPanel(Transform parent)
        {
            _detailPanel = CreatePanel("DetailPanel", parent);
            Image detailBg = _detailPanel.GetComponent<Image>();
            detailBg.color = new Color(_nativeBgColor.r * 1.1f, _nativeBgColor.g * 1.1f, _nativeBgColor.b * 1.1f, 0.9f);
            if (_nativeBgSprite != null)
            {
                detailBg.sprite = _nativeBgSprite;
                detailBg.type = Image.Type.Sliced;
            }

            VerticalLayoutGroup detailLayout = _detailPanel.AddComponent<VerticalLayoutGroup>();
            detailLayout.padding = new RectOffset(16, 16, 12, 12);
            detailLayout.spacing = 6;
            detailLayout.childForceExpandWidth = true;
            detailLayout.childForceExpandHeight = false;
            detailLayout.childControlWidth = true;
            detailLayout.childControlHeight = true;

            LayoutElement detailLe = _detailPanel.AddComponent<LayoutElement>();
            detailLe.flexibleWidth = 0.6f;
            detailLe.flexibleHeight = 1;

            _detailTitle = CreateNativeTextComponent("DetailTitle", _detailPanel.transform, "", 22, AccentColor);
            _detailVersion = CreateLabeledField("Version", _detailPanel.transform);
            _detailAuthor = CreateLabeledField("Author", _detailPanel.transform);
            _detailType = CreateLabeledField("Type", _detailPanel.transform);

            CreateSeparator(_detailPanel.transform);

            CreateNativeTextObj("DescHeader", _detailPanel.transform, "Description", 16, AccentColor);
            _detailDescription = CreateNativeTextComponent("DetailDescription", _detailPanel.transform,
                "Select a pack to view details.", 14, _nativeTextColor);

            CreateSeparator(_detailPanel.transform);

            CreateNativeTextObj("ContentHeader", _detailPanel.transform, "Content Summary", 16, AccentColor);
            _detailContent = CreateNativeTextComponent("DetailContent", _detailPanel.transform, "", 14, _nativeTextColor);

            _detailErrors = CreateNativeTextComponent("DetailErrors", _detailPanel.transform, "", 14, ErrorColor);
        }

        // ── Pack list population ───────────────────────────────────────────────

        private void RebuildPackList()
        {
            foreach (GameObject row in _packRows)
            {
                if (row != null) Destroy(row);
            }
            _packRows.Clear();

            if (_packListContent == null) return;

            for (int i = 0; i < _packs.Count; i++)
            {
                PackDisplayInfo pack = _packs[i];
                int index = i;

                GameObject row = CreatePackRow(pack, index);
                row.transform.SetParent(_packListContent, false);
                _packRows.Add(row);
            }
        }

        private GameObject CreatePackRow(PackDisplayInfo pack, int index)
        {
            bool isSelected = index == _selectedIndex;

            GameObject row = new GameObject($"PackRow_{pack.Id}");
            row.AddComponent<RectTransform>();

            Image rowBg = row.AddComponent<Image>();
            rowBg.color = isSelected ? SelectedColor : new Color(0, 0, 0, 0);

            HorizontalLayoutGroup rowLayout = row.AddComponent<HorizontalLayoutGroup>();
            rowLayout.padding = new RectOffset(8, 8, 4, 4);
            rowLayout.spacing = 8;
            rowLayout.childForceExpandWidth = false;
            rowLayout.childForceExpandHeight = true;
            rowLayout.childControlWidth = true;
            rowLayout.childControlHeight = true;
            rowLayout.childAlignment = TextAnchor.MiddleLeft;

            LayoutElement rowLe = row.AddComponent<LayoutElement>();
            rowLe.preferredHeight = 36;
            rowLe.flexibleWidth = 1;

            // Make the whole row clickable
            Button rowButton = row.AddComponent<Button>();
            rowButton.targetGraphic = rowBg;
            ColorBlock cb = rowButton.colors;
            cb.normalColor = isSelected ? SelectedColor : new Color(0, 0, 0, 0);
            cb.highlightedColor = new Color(_nativeBgColor.r * 1.3f, _nativeBgColor.g * 1.3f, _nativeBgColor.b * 1.3f, 1f);
            cb.pressedColor = SelectedColor;
            cb.selectedColor = isSelected ? SelectedColor : new Color(0, 0, 0, 0);
            rowButton.colors = cb;

            rowButton.onClick.AddListener(() =>
            {
                _selectedIndex = index;
                RebuildPackList();
                RefreshDetailPane();
            });

            // Enable/disable toggle
            GameObject toggleObj = new GameObject("Toggle");
            toggleObj.transform.SetParent(row.transform, false);
            toggleObj.AddComponent<RectTransform>();
            Toggle toggle = toggleObj.AddComponent<Toggle>();

            GameObject toggleBg = new GameObject("Background");
            toggleBg.transform.SetParent(toggleObj.transform, false);
            RectTransform toggleBgRt = toggleBg.AddComponent<RectTransform>();
            toggleBgRt.sizeDelta = new Vector2(20, 20);
            Image toggleBgImg = toggleBg.AddComponent<Image>();
            toggleBgImg.color = _nativeButtonBgColor;

            GameObject checkmark = new GameObject("Checkmark");
            checkmark.transform.SetParent(toggleBg.transform, false);
            RectTransform checkRt = checkmark.AddComponent<RectTransform>();
            checkRt.anchorMin = new Vector2(0.15f, 0.15f);
            checkRt.anchorMax = new Vector2(0.85f, 0.85f);
            checkRt.offsetMin = Vector2.zero;
            checkRt.offsetMax = Vector2.zero;
            Image checkImg = checkmark.AddComponent<Image>();
            checkImg.color = AccentColor;

            toggle.targetGraphic = toggleBgImg;
            toggle.graphic = checkImg;
            toggle.isOn = pack.IsEnabled;

            LayoutElement toggleLe = toggleObj.AddComponent<LayoutElement>();
            toggleLe.preferredWidth = 24;
            toggleLe.preferredHeight = 24;

            string packId = pack.Id;
            toggle.onValueChanged.AddListener((bool val) =>
            {
                OnPackToggled?.Invoke(packId, val);
            });

            // Pack name
            GameObject nameObj = CreateNativeTextObj($"Name_{pack.Id}", row.transform, pack.Name, 15, _nativeTextColor);
            LayoutElement nameLe = nameObj.AddComponent<LayoutElement>();
            nameLe.flexibleWidth = 1;

            // Version badge
            GameObject versionObj = CreateNativeTextObj($"Version_{pack.Id}", row.transform, $"v{pack.Version}", 12, DisabledColor);
            LayoutElement versionLe = versionObj.AddComponent<LayoutElement>();
            versionLe.preferredWidth = 60;

            // Type badge
            GameObject typeObj = CreateNativeTextObj($"Type_{pack.Id}", row.transform, pack.Type, 12, AccentColor);
            LayoutElement typeLe = typeObj.AddComponent<LayoutElement>();
            typeLe.preferredWidth = 70;

            // Status indicator
            Color statusColor = pack.Errors.Count > 0 ? ErrorColor
                : pack.IsEnabled ? EnabledBadgeColor
                : DisabledColor;
            string statusText = pack.Errors.Count > 0 ? "ERR"
                : pack.IsEnabled ? "ON"
                : "OFF";
            GameObject statusObj = CreateNativeTextObj($"Status_{pack.Id}", row.transform, statusText, 12, statusColor);
            LayoutElement statusLe = statusObj.AddComponent<LayoutElement>();
            statusLe.preferredWidth = 35;

            return row;
        }

        // ── Detail pane ────────────────────────────────────────────────────────

        private void RefreshDetailPane()
        {
            if (_selectedIndex < 0 || _selectedIndex >= _packs.Count)
            {
                SetTextComponentValue(_detailTitle, "");
                SetTextComponentValue(_detailVersion, "");
                SetTextComponentValue(_detailAuthor, "");
                SetTextComponentValue(_detailType, "");
                SetTextComponentValue(_detailDescription, "Select a pack to view details.");
                SetTextComponentValue(_detailContent, "");
                SetTextComponentValue(_detailErrors, "");
                return;
            }

            PackDisplayInfo pack = _packs[_selectedIndex];

            SetTextComponentValue(_detailTitle, pack.Name);
            SetTextComponentValue(_detailVersion, pack.Version);
            SetTextComponentValue(_detailAuthor, pack.Author);
            SetTextComponentValue(_detailType, pack.Type);
            SetTextComponentValue(_detailDescription,
                string.IsNullOrEmpty(pack.Description) ? "(No description)" : pack.Description!);

            // Content summary
            if (pack.ContentSummary != null && pack.ContentSummary.Count > 0)
            {
                System.Text.StringBuilder sb = new System.Text.StringBuilder(256);
                foreach (var kvp in pack.ContentSummary)
                {
                    if (sb.Length > 0) sb.Append("\n");
                    sb.Append($"  {kvp.Key}: {kvp.Value}");
                }
                SetTextComponentValue(_detailContent, sb.ToString());
            }
            else
            {
                SetTextComponentValue(_detailContent, "(none declared)");
            }

            // Errors
            if (pack.Errors.Count > 0)
            {
                System.Text.StringBuilder sb = new System.Text.StringBuilder(256);
                sb.Append("Errors:\n");
                foreach (string err in pack.Errors)
                {
                    sb.Append($"  - {err}\n");
                }
                SetTextComponentValue(_detailErrors, sb.ToString());
            }
            else
            {
                SetTextComponentValue(_detailErrors, "");
            }
        }

        // ── UI Helpers (native-font-aware) ─────────────────────────────────────

        /// <summary>
        /// Creates a text GameObject using TMP_Text when available (native DINO font),
        /// falling back to legacy Text with the harvested native font.
        /// </summary>
        private GameObject CreateNativeTextObj(string name, Transform parent, string text, int fontSize, Color color)
        {
            GameObject go = new GameObject(name);
            go.transform.SetParent(parent, false);
            go.AddComponent<RectTransform>();

            // Try TMP_Text first (matches DINO's native font)
            if (TryAddTmpText(go, text, fontSize, color))
                return go;

            // Fallback: legacy Text with harvested native font
            Text t = go.AddComponent<Text>();
            t.text = text;
            t.font = GetFont();
            t.fontSize = fontSize;
            t.color = color;
            t.alignment = TextAnchor.MiddleLeft;
            t.horizontalOverflow = HorizontalWrapMode.Wrap;
            t.verticalOverflow = VerticalWrapMode.Truncate;
            t.supportRichText = true;
            return go;
        }

        /// <summary>
        /// Creates a text component and returns the Component reference (works for both
        /// TMP_Text and legacy Text). Used for detail pane fields that need updating.
        /// </summary>
        private Component CreateNativeTextComponent(string name, Transform parent, string text, int fontSize, Color color)
        {
            GameObject go = CreateNativeTextObj(name, parent, text, fontSize, color);

            // Return whichever text component is on the GO
            if (_tmpTextType != null)
            {
                Component? tmp = go.GetComponent(_tmpTextType);
                if (tmp != null) return tmp;
            }

            Text? legacyText = go.GetComponent<Text>();
            if (legacyText != null) return legacyText;

            // Should not happen, but safety
            return go.AddComponent<Text>();
        }

        /// <summary>
        /// Attempts to add a TMPro.TextMeshProUGUI component via reflection.
        /// Returns true on success.
        /// </summary>
        private bool TryAddTmpText(GameObject go, string text, int fontSize, Color color)
        {
            if (_tmpTextType == null || _nativeTmpFontAsset == null) return false;

            try
            {
                // TextMeshProUGUI is the UGUI variant of TMP_Text
                Type? tmpUguiType = Type.GetType("TMPro.TextMeshProUGUI, Unity.TextMeshPro")
                                 ?? Type.GetType("TMPro.TextMeshProUGUI, Assembly-CSharp");
                if (tmpUguiType == null) return false;

                Component tmp = go.AddComponent(tmpUguiType);
                if (tmp == null) return false;

                // Set text
                var textProp = tmpUguiType.GetProperty("text");
                textProp?.SetValue(tmp, text);

                // Set font
                var fontProp = tmpUguiType.GetProperty("font");
                fontProp?.SetValue(tmp, _nativeTmpFontAsset);

                // Set font size
                var sizeProp = tmpUguiType.GetProperty("fontSize");
                sizeProp?.SetValue(tmp, (float)fontSize);

                // Set color
                var colorProp = tmpUguiType.GetProperty("color");
                colorProp?.SetValue(tmp, color);

                // Set alignment to Left + Middle
                // TMPro.TextAlignmentOptions.Left = 257 (0x101)
                var alignProp = tmpUguiType.GetProperty("alignment");
                if (alignProp != null)
                {
                    Type? alignType = alignProp.PropertyType;
                    // TextAlignmentOptions — these are compile-time-known enum names, not user input.
                    // Enum.Parse is correct here; Enum.TryParse<T> requires a generic type param
                    // we don't have (the type is discovered via reflection at runtime).
                    try
                    {
                        object? midlineLeft = Enum.Parse(alignType, "MidlineLeft"); // DF1009-ok: fixed enum name via reflection
                        alignProp.SetValue(tmp, midlineLeft);
                    }
                    catch
                    {
                        try
                        {
                            object? left = Enum.Parse(alignType, "Left"); // DF1009-ok: fixed enum name via reflection
                            alignProp.SetValue(tmp, left);
                        }
                        catch { /* safe-swallow: alignment is cosmetic */ }
                    }
                }

                // Enable wrapping
                var wrapProp = tmpUguiType.GetProperty("enableWordWrapping");
                wrapProp?.SetValue(tmp, true);

                // Rich text
                var richProp = tmpUguiType.GetProperty("richText");
                richProp?.SetValue(tmp, true);

                return true;
            }
            catch (Exception ex)
            {
                LogInfo($"TryAddTmpText failed (non-fatal, falling back to legacy Text): {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Sets the text value on a Component that is either a Text or TMP_Text.
        /// </summary>
        private void SetTextComponentValue(Component? component, string text)
        {
            if (component == null) return;

            if (component is Text legacyText)
            {
                legacyText.text = text;
                return;
            }

            // TMP_Text via reflection
            try
            {
                var textProp = component.GetType().GetProperty("text");
                textProp?.SetValue(component, text);
            }
            catch { /* safe-swallow: text update is best-effort */ }
        }

        private Font GetFont()
        {
            if (_nativeFont != null) return _nativeFont;
            // Last resort: OS font
            _nativeFont = Font.CreateDynamicFontFromOSFont("Arial", 16);
            return _nativeFont;
        }

        private Component CreateLabeledField(string label, Transform parent)
        {
            GameObject row = new GameObject($"Field_{label}");
            row.transform.SetParent(parent, false);
            row.AddComponent<RectTransform>();

            HorizontalLayoutGroup layout = row.AddComponent<HorizontalLayoutGroup>();
            layout.spacing = 8;
            layout.childForceExpandWidth = false;
            layout.childForceExpandHeight = true;
            layout.childControlWidth = true;
            layout.childControlHeight = true;

            LayoutElement rowLe = row.AddComponent<LayoutElement>();
            rowLe.preferredHeight = 22;

            CreateNativeTextObj($"Label_{label}", row.transform, $"{label}:", 14, DisabledColor);
            LayoutElement labelLe = row.transform.Find($"Label_{label}")?.gameObject.AddComponent<LayoutElement>()!;
            labelLe.preferredWidth = 80;

            Component valueComponent = CreateNativeTextComponent($"Value_{label}", row.transform, "", 14, _nativeTextColor);
            LayoutElement valueLe = valueComponent.gameObject.AddComponent<LayoutElement>();
            valueLe.flexibleWidth = 1;

            return valueComponent;
        }

        private GameObject CreatePanel(string name, Transform parent)
        {
            GameObject go = new GameObject(name);
            go.transform.SetParent(parent, false);
            go.AddComponent<RectTransform>();
            go.AddComponent<Image>();
            return go;
        }

        private void CreateSeparator(Transform parent)
        {
            GameObject sep = new GameObject("Separator");
            sep.transform.SetParent(parent, false);
            sep.AddComponent<RectTransform>();
            Image sepImg = sep.AddComponent<Image>();
            sepImg.color = new Color(1, 1, 1, 0.1f);
            LayoutElement sepLe = sep.AddComponent<LayoutElement>();
            sepLe.preferredHeight = 1;
            sepLe.flexibleWidth = 1;
        }

        private GameObject CreateNativeButton(string name, Transform parent, string label, Action onClick)
        {
            GameObject btn = new GameObject(name);
            btn.transform.SetParent(parent, false);
            btn.AddComponent<RectTransform>();

            Image btnBg = btn.AddComponent<Image>();
            btnBg.color = _nativeButtonBgColor;
            if (_nativeBgSprite != null)
            {
                btnBg.sprite = _nativeBgSprite;
                btnBg.type = Image.Type.Sliced;
            }

            Button button = btn.AddComponent<Button>();
            button.targetGraphic = btnBg;
            ColorBlock colors = button.colors;
            colors.normalColor = _nativeButtonBgColor;
            colors.highlightedColor = new Color(_nativeButtonBgColor.r * 1.3f, _nativeButtonBgColor.g * 1.3f, _nativeButtonBgColor.b * 1.3f, 1f);
            colors.pressedColor = new Color(_nativeButtonBgColor.r * 0.7f, _nativeButtonBgColor.g * 0.7f, _nativeButtonBgColor.b * 0.7f, 1f);
            button.colors = colors;

            button.onClick.AddListener(() => onClick?.Invoke());

            // Label
            GameObject labelObj = new GameObject("Label");
            labelObj.transform.SetParent(btn.transform, false);
            RectTransform labelRt = labelObj.AddComponent<RectTransform>();
            labelRt.anchorMin = Vector2.zero;
            labelRt.anchorMax = Vector2.one;
            labelRt.offsetMin = Vector2.zero;
            labelRt.offsetMax = Vector2.zero;

            // Try TMP label first, fall back to legacy
            if (!TryAddTmpText(labelObj, label, 14, _nativeTextColor))
            {
                Text text = labelObj.AddComponent<Text>();
                text.text = label;
                text.font = GetFont();
                text.fontSize = 14;
                text.color = _nativeTextColor;
                text.alignment = TextAnchor.MiddleCenter;
                text.fontStyle = FontStyle.Bold;
            }
            else
            {
                // Center-align the TMP text
                try
                {
                    Component? tmp = labelObj.GetComponent(_tmpTextType!);
                    if (tmp != null)
                    {
                        var alignProp = tmp.GetType().GetProperty("alignment");
                        if (alignProp != null)
                        {
                            try
                            {
                                object? center = Enum.Parse(alignProp.PropertyType, "Center"); // DF1009-ok: fixed enum name via reflection
                                alignProp.SetValue(tmp, center);
                            }
                            catch { /* safe-swallow: alignment is cosmetic */ }
                        }
                    }
                }
                catch { /* safe-swallow: TMP alignment is cosmetic */ }
            }

            return btn;
        }

        private GameObject CreateScrollView(string name, Transform parent)
        {
            GameObject scrollGo = new GameObject(name);
            scrollGo.transform.SetParent(parent, false);
            scrollGo.AddComponent<RectTransform>();

            Image scrollBg = scrollGo.AddComponent<Image>();
            scrollBg.color = new Color(0, 0, 0, 0);

            ScrollRect scrollRect = scrollGo.AddComponent<ScrollRect>();
            scrollRect.horizontal = false;
            scrollRect.vertical = true;
            scrollRect.movementType = ScrollRect.MovementType.Clamped;

            // Viewport
            GameObject viewport = new GameObject("Viewport");
            viewport.transform.SetParent(scrollGo.transform, false);
            RectTransform vpRt = viewport.AddComponent<RectTransform>();
            vpRt.anchorMin = Vector2.zero;
            vpRt.anchorMax = Vector2.one;
            vpRt.offsetMin = Vector2.zero;
            vpRt.offsetMax = Vector2.zero;

            Image vpImage = viewport.AddComponent<Image>();
            vpImage.color = new Color(1, 1, 1, 0.003f);
            Mask vpMask = viewport.AddComponent<Mask>();
            vpMask.showMaskGraphic = false;

            // Content container
            GameObject content = new GameObject("Content");
            content.transform.SetParent(viewport.transform, false);
            RectTransform contentRt = content.AddComponent<RectTransform>();
            contentRt.anchorMin = new Vector2(0, 1);
            contentRt.anchorMax = new Vector2(1, 1);
            contentRt.pivot = new Vector2(0.5f, 1);
            contentRt.offsetMin = Vector2.zero;
            contentRt.offsetMax = Vector2.zero;

            VerticalLayoutGroup contentLayout = content.AddComponent<VerticalLayoutGroup>();
            contentLayout.spacing = 2;
            contentLayout.childForceExpandWidth = true;
            contentLayout.childForceExpandHeight = false;
            contentLayout.childControlWidth = true;
            contentLayout.childControlHeight = true;

            ContentSizeFitter csf = content.AddComponent<ContentSizeFitter>();
            csf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            scrollRect.viewport = vpRt;
            scrollRect.content = contentRt;

            return scrollGo;
        }

        // ── Logging helpers ────────────────────────────────────────────────────

        private void LogInfo(string message)
        {
            _log?.LogInfo($"[{Tag}] {message}");
            DebugLog.Write(Tag, message);
        }
    }
}
