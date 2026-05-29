#nullable enable
using System;
using System.Collections.Generic;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.Localization;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Full-screen native mods page that lives INSIDE the MainMenu canvas.
    /// Replaces the floating UGUI overlay with a settings-style page that
    /// hides the main menu content and shows a pack browser with detail pane.
    ///
    /// Layout:
    ///   - Title bar: "MODS" + Back button
    ///   - Left panel: scrollable pack list (name, version, type, enable/disable)
    ///   - Right panel: detail pane for selected pack
    ///
    /// Colors (dark theme):
    ///   Background: #1A1A2E   Text: #E0E0E0   Accent: #4ECDC4   Selected: #2C3E50
    /// </summary>
    public sealed class NativeModsPage : MonoBehaviour
    {
        // ── Color palette ──────────────────────────────────────────────────────
        private static readonly Color BgColor = new Color(0.102f, 0.102f, 0.180f, 1f);       // #1A1A2E
        private static readonly Color TextColor = new Color(0.878f, 0.878f, 0.878f, 1f);      // #E0E0E0
        private static readonly Color AccentColor = new Color(0.306f, 0.804f, 0.769f, 1f);    // #4ECDC4
        private static readonly Color SelectedColor = new Color(0.173f, 0.243f, 0.314f, 1f);  // #2C3E50
        private static readonly Color PanelBgColor = new Color(0.133f, 0.133f, 0.220f, 1f);   // slightly lighter
        private static readonly Color RowHoverColor = new Color(0.15f, 0.15f, 0.25f, 1f);
        private static readonly Color ErrorColor = new Color(0.9f, 0.3f, 0.3f, 1f);
        private static readonly Color DisabledColor = new Color(0.5f, 0.5f, 0.5f, 1f);
        private static readonly Color EnabledBadgeColor = new Color(0.3f, 0.8f, 0.4f, 1f);

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
        public Action? OnBackClicked { get; set; }

        // ── References to main menu content we hide/show ───────────────────────
        private GameObject? _mainMenuContent;
        private Font? _font;

        /// <summary>Canvas root used to anchor the dependency prompt dialog.</summary>
        private Transform? _canvasRoot;

        // ── UI hierarchy built at Show time ────────────────────────────────────
        private GameObject? _root;
        private RectTransform? _packListContent;
        private readonly List<GameObject> _packRows = new List<GameObject>();

        // Detail pane elements
        private Text? _detailTitle;
        private Text? _detailVersion;
        private Text? _detailAuthor;
        private Text? _detailType;
        private Text? _detailDescription;
        private Text? _detailContent;
        private Text? _detailErrors;
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
        /// <param name="mainMenuCanvas">The MainMenu canvas to parent into.</param>
        /// <param name="mainMenuContent">The content container to hide (re-shown on Back).</param>
        public void Show(Canvas mainMenuCanvas, GameObject? mainMenuContent)
        {
            _mainMenuContent = mainMenuContent;
            _canvasRoot = mainMenuCanvas.transform;

            if (_mainMenuContent != null)
                _mainMenuContent.SetActive(false);

            if (_root == null)
                BuildUI(mainMenuCanvas.transform);

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

        private Font GetFont()
        {
            if (_font != null) return _font;
            _font = Font.CreateDynamicFontFromOSFont("Arial", 16);
            return _font;
        }

        private void BuildUI(Transform parent)
        {
            // Root panel — full-screen overlay inside the MainMenu canvas
            _root = CreatePanel("DINOForge_NativeModsPage", parent);
            RectTransform rootRt = _root.GetComponent<RectTransform>();
            rootRt.anchorMin = Vector2.zero;
            rootRt.anchorMax = Vector2.one;
            rootRt.offsetMin = Vector2.zero;
            rootRt.offsetMax = Vector2.zero;

            // Fix #944/D2: render above all menu siblings so the mods page isn't hidden
            // behind DINO's native buttons/panels that share the same canvas.
            _root.transform.SetAsLastSibling();

            Image rootBg = _root.GetComponent<Image>();
            rootBg.color = BgColor;

            // Vertical layout: title bar + body
            VerticalLayoutGroup rootLayout = _root.AddComponent<VerticalLayoutGroup>();
            rootLayout.padding = new RectOffset(20, 20, 10, 10);
            rootLayout.spacing = 10;
            rootLayout.childForceExpandWidth = true;
            rootLayout.childForceExpandHeight = false;
            rootLayout.childControlWidth = true;
            rootLayout.childControlHeight = true;

            // ── Title bar ──────────────────────────────────────────────────────
            BuildTitleBar(_root.transform);

            // ── Body: left pack list + right detail ────────────────────────────
            BuildBody(_root.transform);
        }

        private void BuildTitleBar(Transform parent)
        {
            GameObject titleBar = CreatePanel("TitleBar", parent);
            Image titleBg = titleBar.GetComponent<Image>();
            titleBg.color = PanelBgColor;

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
            GameObject backBtn = CreateButton("BackButton", titleBar.transform, L10n.T("menu.button.cancel", "Back"), () =>
            {
                Hide();
                OnBackClicked?.Invoke();
            });
            LayoutElement backLe = backBtn.AddComponent<LayoutElement>();
            backLe.preferredWidth = 80;
            backLe.preferredHeight = 36;

            // Title text
            GameObject titleObj = CreateTextObj("TitleText", titleBar.transform, L10n.T("menu.title", "MODS"), 24, AccentColor);
            LayoutElement titleTextLe = titleObj.AddComponent<LayoutElement>();
            titleTextLe.flexibleWidth = 1;

            // Reload button
            GameObject reloadBtn = CreateButton("ReloadButton", titleBar.transform, "Reload Packs", () =>
            {
                OnReloadRequested?.Invoke();
            });
            LayoutElement reloadLe = reloadBtn.AddComponent<LayoutElement>();
            reloadLe.preferredWidth = 120;
            reloadLe.preferredHeight = 36;
        }

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

            // ── Left panel: pack list ──────────────────────────────────────────
            BuildPackListPanel(body.transform);

            // ── Right panel: detail pane ───────────────────────────────────────
            BuildDetailPanel(body.transform);
        }

        private void BuildPackListPanel(Transform parent)
        {
            GameObject panel = CreatePanel("PackListPanel", parent);
            Image panelBg = panel.GetComponent<Image>();
            panelBg.color = PanelBgColor;

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

            // Header with colored section bar
            GameObject headerContainer = CreatePanel("HeaderContainer", panel.transform);
            Image headerBg = headerContainer.GetComponent<Image>();
            headerBg.color = new Color(0, 0, 0, 0); // transparent

            VerticalLayoutGroup headerLayout = headerContainer.AddComponent<VerticalLayoutGroup>();
            headerLayout.padding = new RectOffset(0, 0, 0, 0);
            headerLayout.spacing = 0;
            headerLayout.childForceExpandWidth = true;
            headerLayout.childForceExpandHeight = false;

            LayoutElement headerLe = headerContainer.AddComponent<LayoutElement>();
            headerLe.preferredHeight = 28;
            headerLe.flexibleWidth = 1;

            // Top colored bar (4px tall)
            GameObject colorBar = CreatePanel("AccentBar", headerContainer.transform);
            Image colorBarImg = colorBar.GetComponent<Image>();
            colorBarImg.color = AccentColor;
            LayoutElement colorBarLe = colorBar.AddComponent<LayoutElement>();
            colorBarLe.preferredHeight = 4;
            colorBarLe.flexibleWidth = 1;

            // Header text
            CreateTextObj("PackListHeader", headerContainer.transform, "INSTALLED PACKS", 14, AccentColor);

            // Scroll view
            GameObject scrollView = CreateScrollView("PackListScroll", panel.transform);
            LayoutElement scrollLe = scrollView.AddComponent<LayoutElement>();
            scrollLe.flexibleHeight = 1;
            scrollLe.flexibleWidth = 1;

            // Content container inside the scroll view
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
            detailBg.color = PanelBgColor;

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

            // Header with colored section bar
            GameObject detailHeaderContainer = CreatePanel("DetailHeaderContainer", _detailPanel.transform);
            Image detailHeaderBg = detailHeaderContainer.GetComponent<Image>();
            detailHeaderBg.color = new Color(0, 0, 0, 0); // transparent

            VerticalLayoutGroup detailHeaderLayout = detailHeaderContainer.AddComponent<VerticalLayoutGroup>();
            detailHeaderLayout.padding = new RectOffset(0, 0, 0, 0);
            detailHeaderLayout.spacing = 0;
            detailHeaderLayout.childForceExpandWidth = true;
            detailHeaderLayout.childForceExpandHeight = false;

            LayoutElement detailHeaderLe = detailHeaderContainer.AddComponent<LayoutElement>();
            detailHeaderLe.preferredHeight = 28;
            detailHeaderLe.flexibleWidth = 1;

            // Top colored bar (4px tall)
            GameObject detailColorBar = CreatePanel("AccentBar", detailHeaderContainer.transform);
            Image detailColorBarImg = detailColorBar.GetComponent<Image>();
            detailColorBarImg.color = AccentColor;
            LayoutElement detailColorBarLe = detailColorBar.AddComponent<LayoutElement>();
            detailColorBarLe.preferredHeight = 4;
            detailColorBarLe.flexibleWidth = 1;

            // Detail fields: title in header, rest below
            _detailTitle = CreateTextObj("DetailTitle", detailHeaderContainer.transform, "", 18, AccentColor).GetComponent<Text>();

            // Metadata fields
            _detailVersion = CreateLabeledField("Version", _detailPanel.transform);
            _detailAuthor = CreateLabeledField("Author", _detailPanel.transform);
            _detailType = CreateLabeledField("Type", _detailPanel.transform);

            // Separator
            CreateSeparator(_detailPanel.transform);

            // Description header + body
            CreateTextObj("DescHeader", _detailPanel.transform, "Description", 16, AccentColor);
            GameObject descObj = CreateTextObj("DetailDescription", _detailPanel.transform, "Select a pack to view details.", 14, TextColor);
            _detailDescription = descObj.GetComponent<Text>();

            CreateSeparator(_detailPanel.transform);

            // Content summary
            CreateTextObj("ContentHeader", _detailPanel.transform, "Content Summary", 16, AccentColor);
            GameObject contentObj = CreateTextObj("DetailContent", _detailPanel.transform, "", 14, TextColor);
            _detailContent = contentObj.GetComponent<Text>();

            // Errors
            GameObject errorsObj = CreateTextObj("DetailErrors", _detailPanel.transform, "", 14, ErrorColor);
            _detailErrors = errorsObj.GetComponent<Text>();
        }

        // ── Pack list population ───────────────────────────────────────────────

        private void RebuildPackList()
        {
            // Clear existing rows
            foreach (GameObject row in _packRows)
            {
                if (row != null) Destroy(row);
            }
            _packRows.Clear();

            if (_packListContent == null) return;

            for (int i = 0; i < _packs.Count; i++)
            {
                PackDisplayInfo pack = _packs[i];
                int index = i; // capture for closure

                GameObject row = CreatePackRow(pack, index);
                row.transform.SetParent(_packListContent, false);
                _packRows.Add(row);
            }
        }

        private GameObject CreatePackRow(PackDisplayInfo pack, int index)
        {
            bool isSelected = index == _selectedIndex;

            GameObject row = new GameObject($"PackRow_{pack.Id}");
            RectTransform rowRt = row.AddComponent<RectTransform>();

            Image rowBg = row.AddComponent<Image>();
            // Zebra striping: even rows #1A1F2E / odd rows #1F2536
            bool isEvenRow = index % 2 == 0;
            Color stripedBg = isEvenRow ? new Color(0.102f, 0.122f, 0.180f, 1f) : new Color(0.122f, 0.145f, 0.212f, 1f);
            rowBg.color = isSelected ? SelectedColor : stripedBg;

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
            cb.normalColor = isSelected ? SelectedColor : stripedBg;
            cb.highlightedColor = RowHoverColor;
            cb.pressedColor = SelectedColor;
            cb.selectedColor = isSelected ? SelectedColor : stripedBg;
            cb.fadeDuration = 0.05f;
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

            // Toggle background
            GameObject toggleBg = new GameObject("Background");
            toggleBg.transform.SetParent(toggleObj.transform, false);
            RectTransform toggleBgRt = toggleBg.AddComponent<RectTransform>();
            toggleBgRt.sizeDelta = new Vector2(20, 20);
            Image toggleBgImg = toggleBg.AddComponent<Image>();
            toggleBgImg.color = new Color(0.2f, 0.2f, 0.3f, 1f);

            // Toggle checkmark
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
            string packName = pack.Name;
            IReadOnlyList<string> packDeps = pack.Dependencies;
            bool wasEnabled = pack.IsEnabled;
            toggle.onValueChanged.AddListener((bool val) =>
            {
                // Only intercept when switching from disabled → enabled.
                if (val && !wasEnabled)
                {
                    List<string> missing = CollectMissingDeps(packDeps);
                    if (missing.Count > 0 && _canvasRoot != null)
                    {
                        // Revert the visual toggle immediately — we'll re-apply after confirmation.
                        toggle.SetIsOnWithoutNotify(false);

                        DepEnableDialog.Show(
                            _canvasRoot,
                            packName,
                            missing,
                            onEnableAll: () =>
                            {
                                // Enable each missing dependency first.
                                foreach (string depId in missing)
                                    OnPackToggled?.Invoke(depId, true);

                                // Then enable the target pack and update the toggle visual.
                                toggle.SetIsOnWithoutNotify(true);
                                OnPackToggled?.Invoke(packId, true);
                            },
                            onCancel: () =>
                            {
                                // Toggle already reverted — nothing more to do.
                            });
                        return;
                    }
                }

                OnPackToggled?.Invoke(packId, val);
            });

            // Pack name
            GameObject nameObj = CreateTextObj($"Name_{pack.Id}", row.transform, pack.Name, 15, TextColor);
            LayoutElement nameLe = nameObj.AddComponent<LayoutElement>();
            nameLe.flexibleWidth = 1;

            // Version badge
            GameObject versionObj = CreateTextObj($"Version_{pack.Id}", row.transform, $"v{pack.Version}", 12, DisabledColor);
            LayoutElement versionLe = versionObj.AddComponent<LayoutElement>();
            versionLe.preferredWidth = 60;

            // Type badge
            GameObject typeObj = CreateTextObj($"Type_{pack.Id}", row.transform, pack.Type, 12, AccentColor);
            LayoutElement typeLe = typeObj.AddComponent<LayoutElement>();
            typeLe.preferredWidth = 70;

            // Status indicator
            Color statusColor = pack.Errors.Count > 0 ? ErrorColor
                : pack.IsEnabled ? EnabledBadgeColor
                : DisabledColor;
            string statusText = pack.Errors.Count > 0 ? "ERR"
                : pack.IsEnabled ? "ON"
                : "OFF";
            GameObject statusObj = CreateTextObj($"Status_{pack.Id}", row.transform, statusText, 12, statusColor);
            LayoutElement statusLe = statusObj.AddComponent<LayoutElement>();
            statusLe.preferredWidth = 35;

            return row;
        }

        // ── Detail pane ────────────────────────────────────────────────────────

        private void RefreshDetailPane()
        {
            if (_selectedIndex < 0 || _selectedIndex >= _packs.Count)
            {
                // No selection
                if (_detailTitle != null) _detailTitle.text = "";
                if (_detailVersion != null) _detailVersion.text = "";
                if (_detailAuthor != null) _detailAuthor.text = "";
                if (_detailType != null) _detailType.text = "";
                if (_detailDescription != null) _detailDescription.text = "Select a pack to view details.";
                if (_detailContent != null) _detailContent.text = "";
                if (_detailErrors != null) _detailErrors.text = "";
                return;
            }

            PackDisplayInfo pack = _packs[_selectedIndex];

            if (_detailTitle != null)
                _detailTitle.text = pack.Name;

            if (_detailVersion != null)
                _detailVersion.text = pack.Version;

            if (_detailAuthor != null)
                _detailAuthor.text = pack.Author;

            if (_detailType != null)
                _detailType.text = pack.Type;

            if (_detailDescription != null)
                _detailDescription.text = string.IsNullOrEmpty(pack.Description) ? "(No description)" : pack.Description;

            // Content summary
            if (_detailContent != null)
            {
                if (pack.ContentSummary != null && pack.ContentSummary.Count > 0)
                {
                    System.Text.StringBuilder sb = new System.Text.StringBuilder(256);
                    foreach (var kvp in pack.ContentSummary)
                    {
                        if (sb.Length > 0) sb.Append("\n");
                        sb.Append($"  {kvp.Key}: {kvp.Value}");
                    }
                    _detailContent.text = sb.ToString();
                }
                else
                {
                    _detailContent.text = "(none declared)";
                }
            }

            // Errors
            if (_detailErrors != null)
            {
                if (pack.Errors.Count > 0)
                {
                    System.Text.StringBuilder sb = new System.Text.StringBuilder(256);
                    sb.Append("Errors:\n");
                    foreach (string err in pack.Errors)
                    {
                        sb.Append($"  - {err}\n");
                    }
                    _detailErrors.text = sb.ToString();
                }
                else
                {
                    _detailErrors.text = "";
                }
            }
        }

        // ── Dependency helpers ─────────────────────────────────────────────────

        /// <summary>
        /// Returns the IDs of <paramref name="dependencies"/> that are not currently enabled
        /// in the loaded pack list.
        /// </summary>
        private List<string> CollectMissingDeps(IReadOnlyList<string> dependencies)
        {
            List<string> missing = new List<string>();
            foreach (string depId in dependencies)
            {
                bool depEnabled = false;
                foreach (PackDisplayInfo p in _packs)
                {
                    if (string.Equals(p.Id, depId, StringComparison.Ordinal) && p.IsEnabled)
                    {
                        depEnabled = true;
                        break;
                    }
                }
                if (!depEnabled) missing.Add(depId);
            }
            return missing;
        }

        // ── UI Helpers ─────────────────────────────────────────────────────────

        private GameObject CreatePanel(string name, Transform parent)
        {
            GameObject go = new GameObject(name);
            go.transform.SetParent(parent, false);
            go.AddComponent<RectTransform>();
            go.AddComponent<Image>();
            return go;
        }

        private GameObject CreateTextObj(string name, Transform parent, string text, int fontSize, Color color)
        {
            GameObject go = new GameObject(name);
            go.transform.SetParent(parent, false);
            go.AddComponent<RectTransform>();
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

        private Text CreateLabeledField(string label, Transform parent)
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

            // Label
            GameObject labelObj = CreateTextObj($"Label_{label}", row.transform, $"{label}:", 14, DisabledColor);
            LayoutElement labelLe = labelObj.AddComponent<LayoutElement>();
            labelLe.preferredWidth = 80;

            // Value
            GameObject valueObj = CreateTextObj($"Value_{label}", row.transform, "", 14, TextColor);
            LayoutElement valueLe = valueObj.AddComponent<LayoutElement>();
            valueLe.flexibleWidth = 1;

            return valueObj.GetComponent<Text>();
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

        private GameObject CreateButton(string name, Transform parent, string label, Action onClick)
        {
            GameObject btn = new GameObject(name);
            btn.transform.SetParent(parent, false);
            btn.AddComponent<RectTransform>();

            Image btnBg = btn.AddComponent<Image>();
            btnBg.color = AccentColor;

            Button button = btn.AddComponent<Button>();
            button.targetGraphic = btnBg;
            ColorBlock colors = button.colors;
            colors.normalColor = AccentColor;
            colors.highlightedColor = new Color(AccentColor.r * 1.2f, AccentColor.g * 1.2f, AccentColor.b * 1.2f, 1f);
            colors.pressedColor = new Color(AccentColor.r * 0.8f, AccentColor.g * 0.8f, AccentColor.b * 0.8f, 1f);
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

            Text text = labelObj.AddComponent<Text>();
            text.text = label;
            text.font = GetFont();
            text.fontSize = 14;
            text.color = new Color(0.1f, 0.1f, 0.15f, 1f); // dark text on accent bg
            text.alignment = TextAnchor.MiddleCenter;
            text.fontStyle = FontStyle.Bold;

            return btn;
        }

        private GameObject CreateScrollView(string name, Transform parent)
        {
            // ScrollRect root
            GameObject scrollGo = new GameObject(name);
            scrollGo.transform.SetParent(parent, false);
            RectTransform scrollRt = scrollGo.AddComponent<RectTransform>();

            Image scrollBg = scrollGo.AddComponent<Image>();
            scrollBg.color = new Color(0, 0, 0, 0); // transparent

            ScrollRect scrollRect = scrollGo.AddComponent<ScrollRect>();
            scrollRect.horizontal = false;
            scrollRect.vertical = true;
            scrollRect.movementType = ScrollRect.MovementType.Clamped;

            // Viewport (masks content)
            GameObject viewport = new GameObject("Viewport");
            viewport.transform.SetParent(scrollGo.transform, false);
            RectTransform vpRt = viewport.AddComponent<RectTransform>();
            vpRt.anchorMin = Vector2.zero;
            vpRt.anchorMax = Vector2.one;
            vpRt.offsetMin = Vector2.zero;
            vpRt.offsetMax = Vector2.zero;

            Image vpImage = viewport.AddComponent<Image>();
            vpImage.color = new Color(1, 1, 1, 0.003f); // near-invisible but needed for mask
            Mask vpMask = viewport.AddComponent<Mask>();
            vpMask.showMaskGraphic = false;

            // Content container (vertical layout)
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
