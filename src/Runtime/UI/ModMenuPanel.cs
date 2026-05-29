#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using BepInEx.Logging;
using DINOForge.Runtime.Conflicts;
using DINOForge.Runtime.Localization;
using DINOForge.Runtime.Profiles;
using DINOForge.Runtime.Settings;
using DINOForge.Runtime.Telemetry;
using DINOForge.Runtime.Updates;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// UGUI mod menu panel. Replaces the legacy IMGUI ModMenuOverlay.
    /// Layout: header bar | split (pack list / detail pane) | footer.
    /// Exposes the same public API as <see cref="ModMenuOverlay"/> so ModPlatform
    /// does not need changes.
    ///
    /// Entry points: Opened by F10 hotkey OR by clicking the injected MODS button on the
    /// native menu (main menu / pause menu). Both paths call <see cref="Toggle"/> on this
    /// panel's <see cref="IModMenuHost"/> implementation.
    /// </summary>
    public class ModMenuPanel : MonoBehaviour, IModMenuHost
    {
        // ── Public API surface (mirrors ModMenuOverlay) ──────────────────────────
        /// <summary>Callback invoked when the user clicks Reload Packs.</summary>
        public Action? OnReloadRequested { get; set; }

        /// <summary>Callback invoked when a pack is toggled (packId, isEnabled).</summary>
        public Action<string, bool>? OnPackToggled { get; set; }

        /// <summary>Whether this panel is currently visible or transitioning visible.</summary>
        public bool IsVisible => _targetVisible;

        /// <summary>The currently selected pack index in the current presenter list, or -1 if none.</summary>
        public int SelectedPackIndex => _presenter.SelectedIndex;

        // ── Panel layout constants ────────────────────────────────────────────────
        private const float PanelWidth = 680f;
        private const float PanelHeight = 560f;
        private const float HeaderHeight = 44f;
        private const float FooterHeight = 44f;
        private const float ListWidth = 220f;
        private const float ItemHeight = 40f;
        private const float AnimDuration = 0.15f;

        // ── State ────────────────────────────────────────────────────────────────
        private readonly ModMenuPresenter _presenter = new ModMenuPresenter();
        private ManualLogSource? _log;

        // ── Filter state ─────────────────────────────────────────────────────────
        private string _searchText = "";
        private string _tierFilter = "All";  // All, Engine Extension, Content, Total Conversion, Baseline
        private string _stateFilter = "All"; // All, Enabled, Disabled, Has Errors
        private string _sortBy = "Name";     // Name, Type, Version, Recently Updated
        private readonly List<int> _filteredIndices = new List<int>();

        // ── Keyboard navigation state ────────────────────────────────────────────
        private int _keyboardFocusedRowIndex = -1; // -1 = no focus, 0+ = index in filtered pack list

        // ── Animation ────────────────────────────────────────────────────────────
        private CanvasGroup? _canvasGroup;
        private RectTransform? _panelRt;
        private bool _targetVisible;

        // ── UI references ────────────────────────────────────────────────────────
        private Text? _headerStatusText;
        private RectTransform? _listContent;
        private Text? _listCounterText; // "N of M packs"
        private InputField? _searchInput;
        private Dropdown? _tierDropdown;
        private Dropdown? _stateDropdown;
        private Dropdown? _sortDropdown;
        private GameObject? _detailPane;
        private Text? _detailName;
        private Text? _detailMeta;
        private Text? _detailDesc;
        private Text? _detailDeps;
        private Text? _detailConflicts;
        private Text? _detailLoadOrder;
        private Text? _detailContent;
        private Text? _detailDetectedConflicts;
        private bool _listRefreshQueued;

        // ── Rich detail pane refs (#897) ──────────────────────────────────────
        private Text? _detailLicense;
        private Text? _detailRichContent;
        private RectTransform? _detailTagsRow;
        private RectTransform? _detailLinksRow;
        private RectTransform? _detailGalleryRow;

        // ── Badge row (#928-935) ──────────────────────────────────────────────
        /// <summary>Root GameObject of the horizontal badge row; rebuilt on each selection change.</summary>
        private GameObject? _detailBadgesRow;
        private readonly List<Image> _galleryThumbs = new List<Image>();
        private GameObject? _screenshotModal;
        private Image? _modalImage;

        // ── Conflict resolution (#903) ────────────────────────────────────────
        private RectTransform? _conflictSection;
        private GameObject? _diffModal;
        private Text? _diffModalTitle;
        private Text? _diffLeftText;
        private Text? _diffRightText;
        private ConflictResolutionStore? _conflictStore;
        private string _packsDirectory = string.Empty;

        // ── Update banner (#899) ──────────────────────────────────────────────
        /// <summary>Container row for the updates-available banner (hidden until updates found).</summary>
        private GameObject? _updateBannerRoot;
        /// <summary>Vertical content inside the banner — one row per pending update.</summary>
        private RectTransform? _updateBannerContent;
        private const float UpdateBannerRowHeight = 24f;

        // ── Profile manager (#918) ────────────────────────────────────────────
        private ProfileManager? _profileManager;
        private Dropdown? _profileDropdown;
        private InputField? _profileNameInput;
        private GameObject? _profileSaveModal;

        /// <summary>Callback invoked when the user loads a profile. Arg = list of pack IDs to enable.</summary>
        public Action<System.Collections.Generic.IReadOnlyList<string>>? OnProfileLoaded { get; set; }

        // ── Pack settings panel (#925) ────────────────────────────────────────
        /// <summary>Container for the per-pack settings section in the detail pane.</summary>
        private GameObject? _settingsSection;
        /// <summary>Content area inside the settings section for dynamically added setting controls.</summary>
        private RectTransform? _settingsContent;

        /// <summary>Canvas root used to anchor the dependency prompt dialog.</summary>
        private Transform? _canvasRoot;

        // ── Telemetry tab (#921) ──────────────────────────────────────────────
        /// <summary>Text element that displays the MetricsCollector DumpMarkdown output.</summary>
        private Text? _telemetryText;
        /// <summary>Running coroutine handle for the 2-second auto-refresh loop; null when not running.</summary>
        private Coroutine? _telemetryRefreshCoroutine;
        private const float TelemetryRefreshIntervalSec = 2f;

        // ── Help panel (#937) ──────────────────────────────────────────────────
        /// <summary>FAQ/help panel component; built on-demand when first shown.</summary>
        private HelpPanel? _helpPanel;

        // ── Bootstrap ────────────────────────────────────────────────────────────

        /// <summary>
        /// Initializes the logger. Must be called before Build().
        /// </summary>
        /// <param name="log">BepInEx logger for diagnostics.</param>
        public void Initialize(ManualLogSource log)
        {
            _log = log;
            _log?.LogInfo("[ModMenuPanel] Initialized with logger.");
        }

        /// <summary>
        /// Provides the packs directory path so the diff modal can read definition YAML files.
        /// Call before or after Build(); safe to call at any time.
        /// </summary>
        public void SetPacksDirectory(string packsDirectory)
        {
            _packsDirectory = packsDirectory ?? string.Empty;
        }

        /// <summary>
        /// Provides the conflict-resolution persistence store.
        /// Call before or after Build(); safe to call at any time.
        /// </summary>
        public void SetConflictResolutionStore(ConflictResolutionStore store)
        {
            _conflictStore = store;
        }

        /// <summary>
        /// Provides the profile manager that backs the Profiles section (#918).
        /// Call before or after Build(); safe to call at any time.
        /// When set before Build() the profile dropdown is populated immediately;
        /// when set after Build() the dropdown is refreshed on the next call to
        /// <see cref="RefreshProfileDropdown"/>.
        /// </summary>
        internal void SetProfileManager(ProfileManager profileManager)
        {
            _profileManager = profileManager;
            RefreshProfileDropdown();
        }

        /// <summary>
        /// Builds the full UGUI hierarchy. Call from DFCanvas.Start() on the main thread.
        /// </summary>
        /// <param name="canvasRoot">Root canvas transform to attach to.</param>
        public void Build(Transform canvasRoot)
        {
            _canvasRoot = canvasRoot;
            _log?.LogInfo("[ModMenuPanel.Build] Starting UGUI hierarchy construction...");

            // Root panel — centered
            GameObject rootGo = UiBuilder.MakePanel(canvasRoot, "ModMenuPanel",
                UiBuilder.BgDeep, new Vector2(PanelWidth, PanelHeight));
            RectTransform rootRt = rootGo.GetComponent<RectTransform>();
            rootRt.anchorMin = new Vector2(0.5f, 0.5f);
            rootRt.anchorMax = new Vector2(0.5f, 0.5f);
            rootRt.pivot = new Vector2(0.5f, 0.5f);
            rootRt.anchoredPosition = new Vector2(300f, 0f); // slide-in offset start

            _panelRt = rootRt;
            _canvasGroup = UiBuilder.EnsureCanvasGroup(rootGo);
            _canvasGroup.alpha = 0f;
            _canvasGroup.interactable = false;
            _canvasGroup.blocksRaycasts = false;

            BuildHeader(rootGo.transform);
            BuildBody(rootGo.transform);
            BuildFooter(rootGo.transform);

            // If packs were already loaded before the UI finished building, render
            // them immediately so the list does not stay blank until the next refresh.
            RebuildPackList();
            RefreshDetail();

            _log?.LogInfo($"[ModMenuPanel.Build] UGUI hierarchy complete. _listContent={(_listContent != null ? _listContent.name : "NULL")}");
        }

        // ── Public API ────────────────────────────────────────────────────────────

        /// <summary>Replaces the pack list and refreshes the UI.</summary>
        public void SetPacks(IEnumerable<PackDisplayInfo> packs)
        {
            int beforeCount = _presenter.Packs.Count;
            _presenter.SetPacks(packs);

            _log?.LogInfo($"");
            _log?.LogInfo($"╔════════════════════════════════════════════════════════════════════════════════════╗");
            _log?.LogInfo($"║ [ModMenuPanel.SetPacks] ENTRY                                                       ║");
            _log?.LogInfo($"╚════════════════════════════════════════════════════════════════════════════════════╝");
            _log?.LogInfo($"  Before: {beforeCount} packs, After: {_presenter.Packs.Count} packs");
            _log?.LogInfo($"  _listContent: {(_listContent != null ? $"READY (name={_listContent.name}, active={_listContent.gameObject.activeSelf})" : "NULL")}");
            _log?.LogInfo($"  SelectedIndex: {_presenter.SelectedIndex}");

            if (_presenter.Packs.Count > 0)
            {
                _log?.LogInfo($"  Pack list:");
                foreach (PackDisplayInfo p in _presenter.Packs)
                {
                    _log?.LogInfo($"    • {p.Name} (ID: {p.Id}, enabled: {p.IsEnabled})");
                }
            }

            // Safety check: if _listContent is null, it means Build() hasn't been called yet
            // or failed. This can happen if SetPacks is called before the UI hierarchy is complete.
            if (_listContent == null)
            {
                _log?.LogWarning("[ModMenuPanel.SetPacks] _listContent is NULL! UI hierarchy not initialized. " +
                    "Packs will queue and render when UI is ready. Check DFCanvas.Start() completion.");
            }

            // Fix #944/B2: ApplyFilters initialises _filteredIndices from the new pack set.
            // Without this, _filteredIndices stays empty after SetPacks → counter shows "0 of N"
            // and filtering is broken on first display.
            _log?.LogInfo($"[ModMenuPanel.SetPacks] Calling ApplyFilters() to populate _filteredIndices...");
            ApplyFilters();
            _log?.LogInfo($"[ModMenuPanel.SetPacks] ApplyFilters() complete ({_filteredIndices.Count} of {_presenter.Packs.Count} visible). Calling RefreshDetail()...");
            RefreshDetail();
            _log?.LogInfo($"[ModMenuPanel.SetPacks] RefreshDetail() complete. EXIT.");
            _log?.LogInfo($"");
        }

        /// <summary>Updates the header status text.</summary>
        public void SetStatus(string message, int errorCount = 0)
        {
            _presenter.SetStatus(message, errorCount);
            if (_headerStatusText != null)
            {
                _headerStatusText.text = BuildStatusLine();
                _headerStatusText.color = _presenter.ErrorCount > 0 ? UiBuilder.Error : UiBuilder.TextSecondary;
            }
        }

        /// <summary>Shows the panel with a slide-in animation.</summary>
        public void Show()
        {
            // Immediate visibility - no animation (Update() never fires in DINO)
            _targetVisible = true;
            if (_canvasGroup != null)
            {
                _canvasGroup.alpha = 1f;
                _canvasGroup.interactable = true;
                _canvasGroup.blocksRaycasts = true;
            }

            // Force panel to be fully visible
            if (_panelRt != null)
            {
                _panelRt.gameObject.SetActive(true);
                _panelRt.anchoredPosition = Vector2.zero; // Ensure no slide offset
            }

            // Force all children to be visible
            if (_listContent != null)
            {
                _listContent.gameObject.SetActive(true);
                for (int i = 0; i < _listContent.childCount; i++)
                {
                    _listContent.GetChild(i).gameObject.SetActive(true);
                }
                UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_listContent);
            }

            // Also force the entire panel hierarchy to rebuild
            if (_panelRt != null)
            {
                UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_panelRt);
            }
        }

        /// <summary>Hides the panel immediately (no animation, Update() never fires).</summary>
        public void Hide()
        {
            _targetVisible = false;
            if (_canvasGroup != null)
            {
                _canvasGroup.alpha = 0f;
                _canvasGroup.interactable = false;
                _canvasGroup.blocksRaycasts = false;
            }

            if (_panelRt != null)
            {
                _panelRt.gameObject.SetActive(false);
            }
        }

        /// <inheritdoc />
        public void Toggle()
        {
            if (IsVisible) Hide();
            else Show();
        }

        /// <summary>
        /// Shows the FAQ/help panel. Called when the Help button is clicked in the header.
        /// </summary>
        private void ShowHelpPanel()
        {
            if (_helpPanel == null && _canvasRoot != null)
            {
                // Build the help panel on first use
                _helpPanel = new GameObject("HelpPanel").AddComponent<HelpPanel>();
                _helpPanel.Build(_canvasRoot, _log);
            }

            if (_helpPanel != null)
            {
                _helpPanel.Show();
            }
        }

        // ── MonoBehaviour ─────────────────────────────────────────────────────────

        private void Update()
        {
            AnimatePanel();
            KeyboardUpdate();
        }

        // ── Animation ─────────────────────────────────────────────────────────────

        private void AnimatePanel()
        {
            // No-op: Update() never fires in DINO (MonoBehaviour.Update is not called).
            // Show()/Hide() set state immediately instead.
        }

        // ── Keyboard navigation ────────────────────────────────────────────────────

        /// <summary>
        /// Handles keyboard input for the mod menu panel.
        ///
        /// Keymap:
        ///   - Arrow Up/Down: Navigate pack list (keyboard focus only)
        ///   - Enter/Space: Toggle enable/disable on the focused pack
        ///   - Tab: Cycle focus between sections (search → pack list)
        ///   - Esc: Close the menu
        ///   - '/': Focus the search input field
        ///   - Ctrl+R: Reload packs
        ///   - Ctrl+S: Reserved for future profile save feature
        ///
        /// Guards: Only processes input when the panel is visible.
        /// </summary>
        private void KeyboardUpdate()
        {
            if (!IsVisible)
                return;

            // Esc always closes, even if input is focused
            if (Input.GetKeyDown(KeyCode.Escape))
            {
                ClearCurrentSelection();
                Hide();
                return;
            }

            // Check if an InputField currently has focus (e.g., search box)
            bool inputFieldFocused = EventSystem.current?.currentSelectedGameObject?.GetComponent<InputField>() != null;

            // Arrow keys + Enter/Space only work when no input field is focused
            if (!inputFieldFocused)
            {
                // Arrow Up: Move focus up in the pack list
                if (Input.GetKeyDown(KeyCode.UpArrow))
                {
                    if (_filteredIndices.Count > 0)
                    {
                        if (_keyboardFocusedRowIndex <= 0)
                            _keyboardFocusedRowIndex = _filteredIndices.Count - 1;
                        else
                            _keyboardFocusedRowIndex--;
                        RefreshPackListFocusIndicator();
                    }
                    return;
                }

                // Arrow Down: Move focus down in the pack list
                if (Input.GetKeyDown(KeyCode.DownArrow))
                {
                    if (_filteredIndices.Count > 0)
                    {
                        _keyboardFocusedRowIndex++;
                        if (_keyboardFocusedRowIndex >= _filteredIndices.Count)
                            _keyboardFocusedRowIndex = 0;
                        RefreshPackListFocusIndicator();
                    }
                    return;
                }

                // Enter/Space: Toggle the focused pack
                if (Input.GetKeyDown(KeyCode.Return) || Input.GetKeyDown(KeyCode.Space))
                {
                    if (_keyboardFocusedRowIndex >= 0 && _keyboardFocusedRowIndex < _filteredIndices.Count)
                    {
                        int realIndex = _filteredIndices[_keyboardFocusedRowIndex];
                        SelectPack(realIndex);
                        OnToggleSelected();
                    }
                    return;
                }
            }

            // Tab: Cycle focus between UI sections (works even with input focused)
            if (Input.GetKeyDown(KeyCode.Tab))
            {
                if (inputFieldFocused)
                {
                    // If search input is focused, move to pack list
                    _keyboardFocusedRowIndex = 0;
                    RefreshPackListFocusIndicator();
                }
                else if (_keyboardFocusedRowIndex >= 0)
                {
                    // If pack list is focused, move back to search input
                    if (_searchInput != null)
                    {
                        EventSystem.current?.SetSelectedGameObject(_searchInput.gameObject);
                        _keyboardFocusedRowIndex = -1;
                        RefreshPackListFocusIndicator();
                    }
                }
                return;
            }

            // '/': Focus the search input (vim-style command prefix)
            if (Input.GetKeyDown(KeyCode.Slash))
            {
                if (_searchInput != null)
                {
                    EventSystem.current?.SetSelectedGameObject(_searchInput.gameObject);
                    _keyboardFocusedRowIndex = -1;
                    RefreshPackListFocusIndicator();
                }
                return;
            }

            // Ctrl+R: Reload packs
            if (Input.GetKey(KeyCode.LeftControl) || Input.GetKey(KeyCode.RightControl))
            {
                if (Input.GetKeyDown(KeyCode.R))
                {
                    ClearCurrentSelection();
                    OnReloadRequested?.Invoke();
                    return;
                }

                // Ctrl+S: Reserved for profile save (placeholder)
                if (Input.GetKeyDown(KeyCode.S))
                {
                    // Future: implement profile save
                    return;
                }
            }
        }

        /// <summary>
        /// Refreshes the visual focus indicator on the pack list rows.
        /// Applies a colored border highlight to the focused row.
        /// </summary>
        private void RefreshPackListFocusIndicator()
        {
            if (_listContent == null)
                return;

            // Clear all focus borders
            for (int i = 0; i < _listContent.childCount; i++)
            {
                Transform child = _listContent.GetChild(i);
                Outline outline = child.GetComponent<Outline>();
                if (outline != null)
                    Destroy(outline);
            }

            // Apply focus border to the focused row
            if (_keyboardFocusedRowIndex >= 0 && _keyboardFocusedRowIndex < _listContent.childCount)
            {
                Transform focusedRow = _listContent.GetChild(_keyboardFocusedRowIndex);
                Outline outline = focusedRow.gameObject.AddComponent<Outline>();
                outline.effectColor = UiBuilder.Accent;
                outline.effectDistance = new Vector2(2f, 2f);
            }
        }

        // ── UI construction ────────────────────────────────────────────────────────

        private void BuildHeader(Transform parent)
        {
            // Gradient background: top dark gray #1A1F2E → bottom slightly lighter #232938
            GameObject headerGradient = UiBuilder.MakePanel(parent, "HeaderGradient",
                HexColor("#1A1F2E"), new Vector2(0f, HeaderHeight));
            RectTransform hGradRt = headerGradient.GetComponent<RectTransform>();
            hGradRt.anchorMin = new Vector2(0f, 1f);
            hGradRt.anchorMax = Vector2.one;
            hGradRt.pivot = new Vector2(0.5f, 1f);
            hGradRt.offsetMin = Vector2.zero;
            hGradRt.offsetMax = Vector2.zero;
            hGradRt.sizeDelta = new Vector2(0f, HeaderHeight);

            // Overlay gradient layer (lighter at bottom)
            GameObject headerGradient2 = UiBuilder.MakePanel(headerGradient.transform, "GradientOverlay",
                HexColor("#232938"), new Vector2(0f, HeaderHeight));
            RectTransform hGrad2Rt = headerGradient2.GetComponent<RectTransform>();
            hGrad2Rt.anchorMin = Vector2.zero;
            hGrad2Rt.anchorMax = Vector2.one;
            hGrad2Rt.offsetMin = Vector2.zero;
            hGrad2Rt.offsetMax = Vector2.zero;
            Image grad2Img = headerGradient2.GetComponent<Image>();
            grad2Img.color = new Color(grad2Img.color.r, grad2Img.color.g, grad2Img.color.b, 0.4f);

            UiBuilder.AddHorizontalLayout(headerGradient, 8f, new RectOffset(12, 8, 6, 6));

            // Title
            Text title = UiBuilder.MakeText(headerGradient.transform, "Title", L10n.T("menu.title", "DINOForge"), 16,
                UiBuilder.Accent, bold: true);
            LayoutElement titleLe = title.gameObject.AddComponent<LayoutElement>();
            titleLe.preferredWidth = 120f;
            titleLe.minWidth = 80f;

            // Status text (flexible)
            _headerStatusText = UiBuilder.MakeText(headerGradient.transform, "Status",
                BuildStatusLine(), 12, UiBuilder.TextSecondary);
            LayoutElement statusLe = _headerStatusText.gameObject.AddComponent<LayoutElement>();
            statusLe.preferredWidth = 300f;
            statusLe.flexibleWidth = 1f;

            // Help button
            Button helpBtn = UiBuilder.MakeButton(
                headerGradient.transform, "HelpBtn", "?",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () =>
                {
                    ShowHelpPanel();
                });
            RectTransform helpBtnRt = helpBtn.GetComponent<RectTransform>();
            LayoutElement helpLe = helpBtn.gameObject.AddComponent<LayoutElement>();
            helpLe.preferredWidth = 28f;
            helpLe.preferredHeight = 28f;

            // Close button
            Button closeBtn = UiBuilder.MakeButton(
                headerGradient.transform, "CloseBtn", "×",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () =>
                {
                    ClearCurrentSelection();
                    Hide();
                });
            RectTransform closeBtnRt = closeBtn.GetComponent<RectTransform>();
            LayoutElement closeLe = closeBtn.gameObject.AddComponent<LayoutElement>();
            closeLe.preferredWidth = 28f;
            closeLe.preferredHeight = 28f;

            // Bottom separator
            GameObject sep = UiBuilder.MakeHorizontalSeparator(parent, UiBuilder.Border);
            RectTransform sepRt = sep.GetComponent<RectTransform>();
            sepRt.anchorMin = new Vector2(0f, 1f);
            sepRt.anchorMax = Vector2.one;
            sepRt.pivot = new Vector2(0.5f, 1f);
            sepRt.anchoredPosition = new Vector2(0f, -HeaderHeight);
            sepRt.sizeDelta = new Vector2(0f, 1f);
        }

        /// <summary>Helper to create a hex color without needing to reference external hex parser.</summary>
        private static Color HexColor(string hex)
        {
            if (ColorUtility.TryParseHtmlString(hex, out Color c))
                return c;
            return new Color(1f, 0f, 1f, 1f); // magenta fallback
        }

        private void BuildBody(Transform parent)
        {
            // Body container between header and footer
            GameObject body = new GameObject("Body", typeof(RectTransform));
            body.transform.SetParent(parent, false);
            RectTransform bodyRt = body.GetComponent<RectTransform>();
            bodyRt.anchorMin = Vector2.zero;
            bodyRt.anchorMax = Vector2.one;
            bodyRt.offsetMin = new Vector2(0f, FooterHeight + 1f);
            bodyRt.offsetMax = new Vector2(0f, -(HeaderHeight + 1f));

            HorizontalLayoutGroup hlg = body.AddComponent<HorizontalLayoutGroup>();
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;
            hlg.spacing = 0f;

            BuildListPane(body.transform);

            // Vertical divider
            GameObject divider = UiBuilder.MakePanel(body.transform, "Divider",
                UiBuilder.Border, new Vector2(1f, 0f));
            LayoutElement divLe = divider.AddComponent<LayoutElement>();
            divLe.preferredWidth = 1f;
            divLe.minWidth = 1f;

            BuildDetailPane(body.transform);
        }

        private void BuildListPane(Transform parent)
        {
            _log?.LogInfo("[ModMenuPanel.BuildListPane] Starting pack list pane construction...");

            GameObject pane = new GameObject("ListPane", typeof(RectTransform));
            pane.transform.SetParent(parent, false);

            LayoutElement paneLe = pane.AddComponent<LayoutElement>();
            paneLe.preferredWidth = ListWidth;
            paneLe.minWidth = ListWidth;
            paneLe.flexibleHeight = 1f;  // CRITICAL: Allow ListPane to expand to fill parent height!

            VerticalLayoutGroup paneLayout = pane.AddComponent<VerticalLayoutGroup>();
            paneLayout.childForceExpandWidth = true;
            paneLayout.childForceExpandHeight = false;
            paneLayout.childControlWidth = true;
            paneLayout.childControlHeight = false;
            paneLayout.childAlignment = TextAnchor.UpperLeft;
            paneLayout.spacing = 0f;
            paneLayout.padding = new RectOffset(0, 0, 0, 0);

            // List header
            GameObject listHeader = UiBuilder.MakePanel(pane.transform, "ListHeader",
                UiBuilder.BgSurface, new Vector2(ListWidth, 32f));
            RectTransform lhRt = listHeader.GetComponent<RectTransform>();
            lhRt.anchorMin = new Vector2(0f, 1f);
            lhRt.anchorMax = new Vector2(1f, 1f);
            lhRt.pivot = new Vector2(0.5f, 1f);
            lhRt.sizeDelta = new Vector2(0f, 32f);

            UiBuilder.AddHorizontalLayout(listHeader, 4f, new RectOffset(8, 8, 6, 6));
            Text lhTitle = UiBuilder.MakeText(listHeader.transform, "ListTitle",
                "Loaded Packs", 12, UiBuilder.TextSecondary, bold: false);
            LayoutElement lhTitleLe = lhTitle.gameObject.AddComponent<LayoutElement>();
            lhTitleLe.flexibleWidth = 1f;
            LayoutElement listHeaderLe = listHeader.AddComponent<LayoutElement>();
            listHeaderLe.preferredWidth = ListWidth;
            listHeaderLe.minWidth = ListWidth;
            listHeaderLe.preferredHeight = 32f;

            // Build filter controls
            BuildListFilters(pane.transform);

            // Scroll view for pack items
            _log?.LogInfo("[ModMenuPanel.BuildListPane] Creating scroll view...");
            (ScrollRect scrollRect, RectTransform content) = UiBuilder.MakeScrollView(
                pane.transform, "PackListScroll",
                new Vector2(ListWidth, 0f));

            // Validate the result
            if (content == null || scrollRect == null)
            {
                _log?.LogError("[ModMenuPanel.BuildListPane] CRITICAL: MakeScrollView failed! " +
                    $"scrollRect={scrollRect != null}, content={content != null}. " +
                    "Pack list will not render. Check UiBuilder.MakeScrollView for exceptions.");
                _listContent = null;
                return;
            }

            // Fix #944/B1: Do NOT override offsetMin/Max with absolute values here.
            // The ListPane uses a VerticalLayoutGroup (header 32px + FilterContainer ~240px +
            // scroll). Manually setting offsetMax to -32px collapses the scroll rect behind the
            // filter bar. Let the LayoutGroup manage positions via LayoutElement.flexibleHeight.
            RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
            LayoutElement scrollLe = scrollRect.gameObject.AddComponent<LayoutElement>();
            scrollLe.preferredWidth = ListWidth;
            scrollLe.minWidth = ListWidth;
            scrollLe.flexibleHeight = 1f;  // fills remaining space below filter container

            content.SetSizeWithCurrentAnchors(RectTransform.Axis.Horizontal, ListWidth);
            content.sizeDelta = new Vector2(ListWidth, content.sizeDelta.y);

            _listContent = content;
            _log?.LogInfo($"[ModMenuPanel.BuildListPane] Scroll view initialized successfully.");
            _log?.LogInfo($"  scrollRt.rect.size={scrollRt.rect.size} (viewport visible area)");
            _log?.LogInfo($"  scrollRt.sizeDelta={scrollRt.sizeDelta}, anchorMin={scrollRt.anchorMin}, anchorMax={scrollRt.anchorMax}");
            _log?.LogInfo($"  content.name={content.name}");
            _log?.LogInfo($"  content.active={content.gameObject.activeSelf}");
            _log?.LogInfo($"  content.anchorMin={content.anchorMin}, anchorMax={content.anchorMax}");
            _log?.LogInfo($"  content.sizeDelta={content.sizeDelta}");
            _log?.LogInfo($"  content.anchoredPosition={content.anchoredPosition}");
            _log?.LogInfo($"  ScrollRect component on: {scrollRect.name}");
            _log?.LogInfo($"  ScrollRect.content set to: {scrollRect.content?.name ?? "NULL"}");
            _log?.LogInfo($"  ScrollRect.vertical={scrollRect.vertical}");
            _log?.LogInfo($"  ScrollRect.enabled={scrollRect.enabled}");
            Image viewportImage = scrollRect.GetComponent<Image>();
            _log?.LogInfo($"  Viewport Image: exists={viewportImage != null}, raycastTarget={viewportImage?.raycastTarget}");

            // Verify components on content
            ContentSizeFitter csf = content.GetComponent<ContentSizeFitter>();
            VerticalLayoutGroup vlg = content.GetComponent<VerticalLayoutGroup>();
            _log?.LogInfo($"  content has ContentSizeFitter: {csf != null} (verticalFit={csf?.verticalFit})");
            _log?.LogInfo($"  content has VerticalLayoutGroup: {vlg != null} (childForceExpandHeight={vlg?.childForceExpandHeight}, spacing={vlg?.spacing})");
        }

        private void BuildDetailPane(Transform parent)
        {
            // Outer container (uses full flexible width; houses a scroll view so all rich
            // content fits without clipping even on a 560-px-tall panel).
            _detailPane = new GameObject("DetailPane", typeof(RectTransform));
            _detailPane.transform.SetParent(parent, false);
            LayoutElement detailLe = _detailPane.AddComponent<LayoutElement>();
            detailLe.flexibleWidth = 1f;
            detailLe.flexibleHeight = 1f;

            // Outer layout: stacks the scroll area + the fixed action-buttons row
            VerticalLayoutGroup outerVlg = _detailPane.AddComponent<VerticalLayoutGroup>();
            outerVlg.childForceExpandWidth = true;
            outerVlg.childForceExpandHeight = false;
            outerVlg.spacing = 0f;
            outerVlg.padding = new RectOffset(0, 0, 0, 0);

            // ── Scrollable content area ───────────────────────────────────────
            (ScrollRect detailScroll, RectTransform detailContent) =
                UiBuilder.MakeScrollView(_detailPane.transform, "DetailScroll", Vector2.zero);
            LayoutElement scrollLe = detailScroll.gameObject.AddComponent<LayoutElement>();
            scrollLe.flexibleWidth = 1f;
            scrollLe.flexibleHeight = 1f;
            scrollLe.minHeight = 100f;

            // Override the content VLG to use detail-pane padding
            VerticalLayoutGroup contentVlg = detailContent.GetComponent<VerticalLayoutGroup>();
            if (contentVlg != null)
            {
                contentVlg.padding = new RectOffset(14, 14, 12, 8);
                contentVlg.spacing = 6f;
            }

            Transform c = detailContent; // shorthand

            // ── Pack name ────────────────────────────────────────────────────
            _detailName = UiBuilder.MakeText(c, "DetailName", "Select a pack", 15,
                UiBuilder.TextPrimary, bold: true);
            AddFlexRow(_detailName.gameObject, preferredHeight: 22f);

            // ── Meta row (author · type · license badge) ─────────────────────
            GameObject metaRow = new GameObject("MetaRow", typeof(RectTransform));
            metaRow.transform.SetParent(c, false);
            HorizontalLayoutGroup metaHlg = metaRow.AddComponent<HorizontalLayoutGroup>();
            metaHlg.spacing = 6f;
            metaHlg.childForceExpandHeight = false;
            metaHlg.childForceExpandWidth = false;
            metaRow.AddComponent<LayoutElement>().preferredHeight = 18f;

            _detailMeta = UiBuilder.MakeText(metaRow.transform, "DetailMeta", "", 12, UiBuilder.TextSecondary);
            _detailMeta.gameObject.AddComponent<LayoutElement>().flexibleWidth = 1f;

            _detailLicense = UiBuilder.MakeText(metaRow.transform, "LicenseBadge", "", 11, UiBuilder.Accent, bold: true);
            LayoutElement licenseLe = _detailLicense.gameObject.AddComponent<LayoutElement>();
            licenseLe.preferredWidth = 80f;
            licenseLe.minWidth = 0f;

            // ── Tags row ─────────────────────────────────────────────────────
            GameObject tagsHost = new GameObject("TagsRow", typeof(RectTransform));
            tagsHost.transform.SetParent(c, false);
            _detailTagsRow = tagsHost.GetComponent<RectTransform>();
            HorizontalLayoutGroup tagsHlg = tagsHost.AddComponent<HorizontalLayoutGroup>();
            tagsHlg.spacing = 4f;
            tagsHlg.childForceExpandHeight = false;
            tagsHlg.childForceExpandWidth = false;
            tagsHlg.childAlignment = TextAnchor.MiddleLeft;
            LayoutElement tagsRowLe = tagsHost.AddComponent<LayoutElement>();
            tagsRowLe.preferredHeight = 22f;
            tagsRowLe.flexibleWidth = 1f;
            tagsHost.SetActive(false); // hidden until pack with tags is selected

            // ── Badges row (#928-935) ─────────────────────────────────────────
            // Rebuilt dynamically in RefreshDetail via RefreshBadgesRow(); no static
            // host needed — the row is created fresh each time the selection changes.

            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            // ── Description ──────────────────────────────────────────────────
            _detailDesc = UiBuilder.MakeText(c, "DetailDesc", "", 12, UiBuilder.TextPrimary);
            LayoutElement descLe = _detailDesc.gameObject.AddComponent<LayoutElement>();
            descLe.preferredHeight = 60f;
            descLe.flexibleWidth = 1f;
            _detailDesc.verticalOverflow = VerticalWrapMode.Overflow;

            // ── External links row ───────────────────────────────────────────
            GameObject linksHost = new GameObject("LinksRow", typeof(RectTransform));
            linksHost.transform.SetParent(c, false);
            _detailLinksRow = linksHost.GetComponent<RectTransform>();
            HorizontalLayoutGroup linksHlg = linksHost.AddComponent<HorizontalLayoutGroup>();
            linksHlg.spacing = 6f;
            linksHlg.childForceExpandHeight = false;
            linksHlg.childForceExpandWidth = false;
            LayoutElement linksRowLe = linksHost.AddComponent<LayoutElement>();
            linksRowLe.preferredHeight = 26f;
            linksRowLe.flexibleWidth = 1f;
            linksHost.SetActive(false); // hidden when no URLs present

            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            // ── Dependencies / Conflicts / Load order ────────────────────────
            _detailDeps = UiBuilder.MakeText(c, "DetailDeps", L10n.T("menu.detail.dependencies", "Dependencies: none"), 12, UiBuilder.TextSecondary);
            AddFlexRow(_detailDeps.gameObject, preferredHeight: 18f);

            _detailConflicts = UiBuilder.MakeText(c, "DetailConflicts", L10n.T("menu.detail.conflicts", "Conflicts: none"), 12, UiBuilder.TextSecondary);
            AddFlexRow(_detailConflicts.gameObject, preferredHeight: 18f);

            _detailLoadOrder = UiBuilder.MakeText(c, "DetailLoadOrder", "Load Order: —", 12, UiBuilder.TextSecondary);
            AddFlexRow(_detailLoadOrder.gameObject, preferredHeight: 18f);

            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            // ── Rich content section ─────────────────────────────────────────
            // Shows count summary + first N item names (units, buildings, factions)
            _detailRichContent = UiBuilder.MakeText(c, "DetailRichContent", L10n.T("menu.detail.content", "Content: (none declared)"), 12,
                new Color(0.6f, 0.85f, 0.6f, 1f));
            LayoutElement richContentLe = _detailRichContent.gameObject.AddComponent<LayoutElement>();
            richContentLe.preferredHeight = 60f;
            richContentLe.flexibleWidth = 1f;
            _detailRichContent.verticalOverflow = VerticalWrapMode.Overflow;

            // Keep the legacy _detailContent / _detailDetectedConflicts wired so
            // RefreshDetail doesn't crash — but hide them (rich content replaces them).
            _detailContent = _detailRichContent; // alias — same text field
            _detailDetectedConflicts = UiBuilder.MakeText(c, "DetailDetectedConflicts", "", 12,
                new Color(0.9f, 0.6f, 0.2f, 1f));
            LayoutElement dcLe = _detailDetectedConflicts.gameObject.AddComponent<LayoutElement>();
            dcLe.preferredHeight = 30f;
            dcLe.flexibleWidth = 1f;
            _detailDetectedConflicts.verticalOverflow = VerticalWrapMode.Overflow;

            // ── Screenshot gallery ───────────────────────────────────────────
            // Horizontal scroll strip of 200×150 thumbnails; hidden when no screenshots.
            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            GameObject galleryHost = new GameObject("GalleryRow", typeof(RectTransform));
            galleryHost.transform.SetParent(c, false);
            _detailGalleryRow = galleryHost.GetComponent<RectTransform>();
            LayoutElement galleryRowLe = galleryHost.AddComponent<LayoutElement>();
            galleryRowLe.preferredHeight = 160f;
            galleryRowLe.flexibleWidth = 1f;
            galleryHost.SetActive(false); // hidden until screenshots found

            // Horizontal scroll view for thumbnails
            (ScrollRect galleryScroll, RectTransform galleryContent) =
                BuildHorizontalScrollView(galleryHost.transform, "GalleryScroll");
            RectTransform galleryScrollRt = galleryScroll.GetComponent<RectTransform>();
            UiBuilder.FillParent(galleryScrollRt);

            // We lazily populate galleryContent in RefreshDetail via the _detailGalleryRow tag.
            // Store the content RT by tagging the galleryHost for later lookup.
            galleryHost.name = "GalleryRow";

            // ── Conflict resolution section (#903) ────────────────────────────
            // Populated dynamically in RefreshConflictButtons(); hidden when no conflicts.
            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            GameObject conflictHost = new GameObject("ConflictSection", typeof(RectTransform));
            conflictHost.transform.SetParent(c, false);
            _conflictSection = conflictHost.GetComponent<RectTransform>();
            VerticalLayoutGroup conflictVlg = conflictHost.AddComponent<VerticalLayoutGroup>();
            conflictVlg.spacing = 6f;
            conflictVlg.childForceExpandWidth = true;
            conflictVlg.childForceExpandHeight = false;
            conflictVlg.padding = new RectOffset(0, 0, 4, 4);
            ContentSizeFitter conflictCsf = conflictHost.AddComponent<ContentSizeFitter>();
            conflictCsf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;
            LayoutElement conflictHostLe = conflictHost.AddComponent<LayoutElement>();
            conflictHostLe.flexibleWidth = 1f;
            conflictHost.SetActive(false); // hidden until conflicts present

            // ── Per-pack runtime settings section (#925) ──────────────────────
            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            GameObject settingsHost = new GameObject("SettingsSection", typeof(RectTransform));
            settingsHost.transform.SetParent(c, false);
            _settingsSection = settingsHost;
            VerticalLayoutGroup settingsVlg = settingsHost.AddComponent<VerticalLayoutGroup>();
            settingsVlg.spacing = 8f;
            settingsVlg.childForceExpandWidth = true;
            settingsVlg.childForceExpandHeight = false;
            settingsVlg.padding = new RectOffset(12, 12, 8, 8);
            ContentSizeFitter settingsCsf = settingsHost.AddComponent<ContentSizeFitter>();
            settingsCsf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;
            LayoutElement settingsHostLe = settingsHost.AddComponent<LayoutElement>();
            settingsHostLe.flexibleWidth = 1f;
            settingsHost.SetActive(false); // hidden until pack has settings

            // Stores the setting controls for this pack
            _settingsContent = settingsHost.GetComponent<RectTransform>();

            // ── Telemetry section (#921) ──────────────────────────────────────
            BuildTelemetrySection(c);

            // ── Fixed action-buttons row (outside scroll) ────────────────────
            GameObject btnRow = new GameObject("ActionButtons", typeof(RectTransform));
            btnRow.transform.SetParent(_detailPane.transform, false);
            HorizontalLayoutGroup btnHlg = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnHlg.spacing = 8f;
            btnHlg.childForceExpandHeight = false;
            btnHlg.childForceExpandWidth = false;
            btnHlg.padding = new RectOffset(14, 14, 6, 6);
            LayoutElement btnRowLe = btnRow.AddComponent<LayoutElement>();
            btnRowLe.preferredHeight = 40f;
            btnRowLe.minHeight = 40f;

            Button toggleBtn = UiBuilder.MakeButton(
                btnRow.transform, "ToggleBtn", L10n.T("menu.button.disable", "Disable"),
                UiBuilder.BgSurface, UiBuilder.TextPrimary,
                OnToggleSelected);
            LayoutElement toggleBtnLe = toggleBtn.gameObject.AddComponent<LayoutElement>();
            toggleBtnLe.preferredWidth = 90f;
            toggleBtnLe.minWidth = 90f;
            toggleBtnLe.preferredHeight = 30f;

            // ── Screenshot modal (full-size overlay, initially hidden) ────────
            BuildScreenshotModal();

            // ── Diff modal (#903) ─────────────────────────────────────────────
            BuildDiffModal();
        }

        /// <summary>Adds a LayoutElement to a GO that occupies one full-width row.</summary>
        private static void AddFlexRow(GameObject go, float preferredHeight)
        {
            LayoutElement le = go.AddComponent<LayoutElement>();
            le.preferredHeight = preferredHeight;
            le.flexibleWidth = 1f;
        }

        /// <summary>Builds a horizontal-only scroll view (no vertical scroll).</summary>
        private static (ScrollRect scrollRect, RectTransform content) BuildHorizontalScrollView(Transform parent, string name)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image), typeof(ScrollRect), typeof(Mask));
            go.transform.SetParent(parent, false);

            Image bgImg = go.GetComponent<Image>();
            bgImg.color = new Color(0f, 0f, 0f, 0f);
            bgImg.raycastTarget = true;

            Mask mask = go.GetComponent<Mask>();
            mask.showMaskGraphic = false;

            GameObject content = new GameObject("Content", typeof(RectTransform));
            content.transform.SetParent(go.transform, false);
            RectTransform contentRt = content.GetComponent<RectTransform>();
            contentRt.anchorMin = new Vector2(0f, 0f);
            contentRt.anchorMax = new Vector2(0f, 1f);
            contentRt.pivot = new Vector2(0f, 0.5f);
            contentRt.offsetMin = Vector2.zero;
            contentRt.offsetMax = Vector2.zero;

            ContentSizeFitter csf = content.AddComponent<ContentSizeFitter>();
            csf.horizontalFit = ContentSizeFitter.FitMode.PreferredSize;

            HorizontalLayoutGroup hlg = content.AddComponent<HorizontalLayoutGroup>();
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;
            hlg.spacing = 6f;
            hlg.padding = new RectOffset(4, 4, 4, 4);

            ScrollRect scrollRect = go.GetComponent<ScrollRect>();
            scrollRect.content = contentRt;
            scrollRect.horizontal = true;
            scrollRect.vertical = false;
            scrollRect.scrollSensitivity = 20f;
            scrollRect.movementType = ScrollRect.MovementType.Clamped;
            scrollRect.viewport = go.GetComponent<RectTransform>();

            return (scrollRect, contentRt);
        }

        /// <summary>Builds the full-screen screenshot modal (hidden by default).</summary>
        private void BuildScreenshotModal()
        {
            if (_panelRt == null) return;
            _screenshotModal = UiBuilder.MakePanel(_panelRt, "ScreenshotModal",
                new Color(0f, 0f, 0f, 0.88f), Vector2.zero);
            UiBuilder.FillParent(_screenshotModal.GetComponent<RectTransform>());

            // The modal image (fills modal with aspect-ratio letterboxing via AspectRatioFitter)
            GameObject imgGo = new GameObject("ModalImage", typeof(RectTransform), typeof(Image));
            imgGo.transform.SetParent(_screenshotModal.transform, false);
            RectTransform imgRt = imgGo.GetComponent<RectTransform>();
            imgRt.anchorMin = new Vector2(0.05f, 0.1f);
            imgRt.anchorMax = new Vector2(0.95f, 0.95f);
            imgRt.offsetMin = Vector2.zero;
            imgRt.offsetMax = Vector2.zero;
            _modalImage = imgGo.GetComponent<Image>();
            _modalImage.preserveAspect = true;

            // Click anywhere on the modal to close
            Button closeOverlay = _screenshotModal.AddComponent<Button>();
            closeOverlay.targetGraphic = _screenshotModal.GetComponent<Image>();
            closeOverlay.onClick.AddListener(() => _screenshotModal.SetActive(false));

            _screenshotModal.SetActive(false);
        }

        private void BuildFooter(Transform parent)
        {
            // Separator above footer
            GameObject sep = UiBuilder.MakeHorizontalSeparator(parent, UiBuilder.Border);
            RectTransform sepRt = sep.GetComponent<RectTransform>();
            sepRt.anchorMin = new Vector2(0f, 0f);
            sepRt.anchorMax = new Vector2(1f, 0f);
            sepRt.pivot = new Vector2(0.5f, 0f);
            sepRt.anchoredPosition = new Vector2(0f, FooterHeight);
            sepRt.sizeDelta = new Vector2(0f, 1f);

            GameObject footer = UiBuilder.MakePanel(parent, "Footer",
                UiBuilder.BgSurface, new Vector2(0f, FooterHeight));
            RectTransform fRt = footer.GetComponent<RectTransform>();
            fRt.anchorMin = Vector2.zero;
            fRt.anchorMax = new Vector2(1f, 0f);
            fRt.pivot = new Vector2(0.5f, 0f);
            fRt.offsetMin = Vector2.zero;
            fRt.offsetMax = Vector2.zero;
            fRt.sizeDelta = new Vector2(0f, FooterHeight);

            UiBuilder.AddHorizontalLayout(footer, 8f, new RectOffset(12, 12, 7, 7));

            // Reload button
            Button reloadBtn = UiBuilder.MakeButton(
                footer.transform, "ReloadBtn", L10n.T("menu.button.reload", "↺  Reload Packs"),
                UiBuilder.BgDeep, UiBuilder.Accent,
                () =>
                {
                    ClearCurrentSelection();
                    OnReloadRequested?.Invoke();
                });
            LayoutElement reloadLe = reloadBtn.gameObject.AddComponent<LayoutElement>();
            reloadLe.preferredWidth = 140f;
            reloadLe.preferredHeight = 30f;

            // Spacer
            GameObject spacer = new GameObject("FooterSpacer", typeof(RectTransform));
            spacer.transform.SetParent(footer.transform, false);
            LayoutElement spacerLe = spacer.AddComponent<LayoutElement>();
            spacerLe.flexibleWidth = 1f;
        }

        // ── Pack list rendering ────────────────────────────────────────────────────

        private void RebuildPackList()
        {
            if (_listContent == null)
            {
                _log?.LogWarning("[ModMenuPanel.RebuildPackList] _listContent is NULL — UI not initialized yet. " +
                    "Pack list will render once Build() completes. Ensure DFCanvas.Start() runs before SetPacks() is called.");
                return;
            }

            // Reset keyboard focus when rebuilding the list
            _keyboardFocusedRowIndex = -1;

            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] START: presenter.Packs.Count={_presenter.Packs.Count}, _listContent={_listContent.name}, active={_listContent.gameObject.activeSelf}");
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] _listContent RectTransform: position={_listContent.anchoredPosition}, sizeDelta={_listContent.sizeDelta}");
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] Clearing {_listContent.childCount} existing items");

            _listContent.SetSizeWithCurrentAnchors(RectTransform.Axis.Horizontal, ListWidth);
            _listContent.sizeDelta = new Vector2(ListWidth, _listContent.sizeDelta.y);

            // Remove existing items immediately from the layout tree to avoid
            // same-frame duplicate entries when SetPacks triggers rapid rebuilds.
            for (int i = _listContent.childCount - 1; i >= 0; i--)
            {
                Transform child = _listContent.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] After clear: childCount={_listContent.childCount}. Now rendering {_presenter.Packs.Count} pack(s)...");

            for (int i = 0; i < _presenter.Packs.Count; i++)
            {
                _log?.LogInfo($"[ModMenuPanel.RebuildPackList] Creating item {i}: '{_presenter.Packs[i].Name}' (ID: {_presenter.Packs[i].Id})");
                BuildPackListItem(_presenter.Packs[i], i);

                Transform item = _listContent.GetChild(i);
                RectTransform rt = item.GetComponent<RectTransform>();
                _log?.LogInfo($"[ModMenuPanel.probe] item[{i}] name={item.name} childCount={item.transform.childCount} rect=({rt.sizeDelta.x}x{rt.sizeDelta.y}) active={item.gameObject.activeSelf}");
                for (int childIndex = 0; childIndex < item.transform.childCount; childIndex++)
                {
                    Transform child = item.transform.GetChild(childIndex);
                    Text text = child.GetComponent<Text>();
                    if (text != null)
                    {
                        RectTransform textRt = text.GetComponent<RectTransform>();
                        _log?.LogInfo($"[ModMenuPanel.probe] item[{i}].child[{childIndex}] Text color={text.color} text='{text.text}' active={text.gameObject.activeSelf} rect=({textRt.sizeDelta.x}x{textRt.sizeDelta.y})");
                    }

                    Image image = child.GetComponent<Image>();
                    if (image != null)
                    {
                        _log?.LogInfo($"[ModMenuPanel.probe] item[{i}].child[{childIndex}] Image color={image.color} spriteNull={image.sprite == null} raycastTarget={image.raycastTarget}");
                    }
                }
            }

            // CRITICAL FIX: Manually set content height since ContentSizeFitter is not calculating correctly
            // Calculate: padding.top + (itemCount * itemHeight) + (itemCount-1 * spacing) + padding.bottom
            float padding_top = 4f, padding_bottom = 4f, spacing = 2f, itemHeight = 40f;
            float calculatedHeight = padding_top + (_presenter.Packs.Count * itemHeight) + (Mathf.Max(0, _presenter.Packs.Count - 1) * spacing) + padding_bottom;
            _listContent.sizeDelta = new Vector2(_listContent.sizeDelta.x, calculatedHeight);
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] MANUAL FIX APPLIED: Set content height to {calculatedHeight} (was {_listContent.sizeDelta.y})");

            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] COMPLETE: childCount={_listContent.childCount}. Listing items:");
            for (int i = 0; i < _listContent.childCount; i++)
            {
                Transform child = _listContent.GetChild(i);
                RectTransform childRt = child.GetComponent<RectTransform>();
                _log?.LogInfo($"  Item {i}: name={child.name}, active={child.gameObject.activeSelf}, sizeDelta={childRt.sizeDelta}, childCount={child.childCount}");
            }

            // CRITICAL: Log content size AFTER all items are created and VerticalLayoutGroup has calculated
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] FINAL CONTENT SIZE: sizeDelta={_listContent.sizeDelta}, rect.height={_listContent.rect.height}");
            ContentSizeFitter csf = _listContent.GetComponent<ContentSizeFitter>();
            VerticalLayoutGroup vlg = _listContent.GetComponent<VerticalLayoutGroup>();
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] ContentSizeFitter: {(csf != null ? $"enabled={csf.enabled}, verticalFit={csf.verticalFit}" : "NULL")}");
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] VerticalLayoutGroup: {(vlg != null ? $"enabled={vlg.enabled}, spacing={vlg.spacing}, padding={vlg.padding}, preferredHeight={vlg.preferredHeight}" : "NULL")}");

            // Calculate expected height manually
            float expectedHeight = 0f;
            if (vlg != null)
            {
                expectedHeight = vlg.padding.top + vlg.padding.bottom;
                for (int i = 0; i < _listContent.childCount; i++)
                {
                    Transform child = _listContent.GetChild(i);
                    LayoutElement childLe = child.GetComponent<LayoutElement>();
                    if (childLe != null && childLe.preferredHeight > 0)
                    {
                        expectedHeight = expectedHeight + childLe.preferredHeight;
                        if (i > 0) expectedHeight = expectedHeight + vlg.spacing;
                    }
                }
            }
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] MANUAL CALCULATION: expected total height={expectedHeight} (padding.top={vlg?.padding.top}, padding.bottom={vlg?.padding.bottom}, spacing={vlg?.spacing}, items={_listContent.childCount})");

            // Drive layout immediately so the ScrollRect sees the correct content bounds
            // even when the panel is currently hidden.
            UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_listContent);
        }

        /// <summary>Builds the filter control UI (search, tier filter, state filter, sort) above the pack list.</summary>
        private void BuildListFilters(Transform parent)
        {
            // Section header bar with colored accent
            GameObject headerBar = new GameObject("FilterHeader", typeof(RectTransform));
            headerBar.transform.SetParent(parent, false);
            RectTransform headerBarRt = headerBar.GetComponent<RectTransform>();

            // Top colored bar (4px)
            GameObject colorBar = UiBuilder.MakePanel(headerBar.transform, "AccentBar",
                UiBuilder.Accent, new Vector2(0f, 4f));
            RectTransform colorBarRt = colorBar.GetComponent<RectTransform>();
            colorBarRt.anchorMin = new Vector2(0f, 1f);
            colorBarRt.anchorMax = Vector2.one;
            colorBarRt.pivot = new Vector2(0.5f, 1f);
            colorBarRt.offsetMin = Vector2.zero;
            colorBarRt.offsetMax = new Vector2(0f, -4f);
            colorBarRt.sizeDelta = new Vector2(0f, 4f);

            LayoutElement headerLe = headerBar.AddComponent<LayoutElement>();
            headerLe.preferredHeight = 28f;

            HorizontalLayoutGroup headerLayout = headerBar.AddComponent<HorizontalLayoutGroup>();
            headerLayout.padding = new RectOffset(8, 8, 4, 4);
            headerLayout.spacing = 0f;
            headerLayout.childForceExpandWidth = true;
            headerLayout.childForceExpandHeight = false;

            Text headerTitle = UiBuilder.MakeText(headerBar.transform, "HeaderTitle", L10n.T("menu.filter.label", "FILTERS"), 12,
                UiBuilder.TextSecondary, bold: true);
            LayoutElement headerTitleLe = headerTitle.gameObject.AddComponent<LayoutElement>();
            headerTitleLe.flexibleWidth = 1f;

            // Filters container
            GameObject filterContainer = new GameObject("FilterContainer", typeof(RectTransform));
            filterContainer.transform.SetParent(parent, false);
            RectTransform fcRt = filterContainer.GetComponent<RectTransform>();

            VerticalLayoutGroup fcLayout = filterContainer.AddComponent<VerticalLayoutGroup>();
            fcLayout.childForceExpandWidth = true;
            fcLayout.childForceExpandHeight = false;
            fcLayout.spacing = 4f;
            fcLayout.padding = new RectOffset(8, 8, 8, 8);

            LayoutElement fcLe = filterContainer.AddComponent<LayoutElement>();
            fcLe.preferredHeight = 212f;  // Enough for profiles row + 3 rows of controls
            fcLe.minHeight = 160f;
            fcLe.flexibleWidth = 1f;

            // ── Profiles row (#918) ──────────────────────────────────────────
            BuildProfilesSection(filterContainer.transform);

            // Search row
            GameObject searchRow = new GameObject("SearchRow", typeof(RectTransform));
            searchRow.transform.SetParent(filterContainer.transform, false);
            HorizontalLayoutGroup searchHlg = searchRow.AddComponent<HorizontalLayoutGroup>();
            searchHlg.spacing = 4f;
            searchHlg.childForceExpandWidth = true;
            searchHlg.childForceExpandHeight = false;
            LayoutElement searchRowLe = searchRow.AddComponent<LayoutElement>();
            searchRowLe.preferredHeight = 28f;

            _searchInput = UiBuilder.MakeInputField(searchRow.transform, "SearchInput", L10n.T("menu.search.placeholder", "Search packs..."),
                OnSearchChanged);
            LayoutElement searchInputLe = _searchInput.gameObject.AddComponent<LayoutElement>();
            searchInputLe.preferredHeight = 28f;
            searchInputLe.flexibleWidth = 1f;
            searchInputLe.minWidth = 100f;

            // Filter row 1: Tier filter
            GameObject filterRow1 = new GameObject("FilterRow1", typeof(RectTransform));
            filterRow1.transform.SetParent(filterContainer.transform, false);
            HorizontalLayoutGroup filterHlg1 = filterRow1.AddComponent<HorizontalLayoutGroup>();
            filterHlg1.spacing = 4f;
            filterHlg1.childForceExpandWidth = false;
            filterHlg1.childForceExpandHeight = false;
            LayoutElement filterRow1Le = filterRow1.AddComponent<LayoutElement>();
            filterRow1Le.preferredHeight = 28f;
            filterRow1Le.flexibleWidth = 1f;

            Text tierLabel = UiBuilder.MakeText(filterRow1.transform, "TierLabel", "Tier:", 11, UiBuilder.TextSecondary);
            LayoutElement tierLabelLe = tierLabel.gameObject.AddComponent<LayoutElement>();
            tierLabelLe.preferredWidth = 40f;

            _tierDropdown = MakeDropdown(filterRow1.transform, "TierDropdown",
                new[] { L10n.T("menu.filter.tier.all", "All"), L10n.T("menu.filter.tier.engine_extension", "Engine Extension"), L10n.T("menu.filter.tier.content", "Content"), L10n.T("menu.filter.tier.total_conversion", "Total Conversion"), L10n.T("menu.filter.tier.baseline", "Baseline") },
                OnTierFilterChanged);
            LayoutElement tierDdLe = _tierDropdown.gameObject.AddComponent<LayoutElement>();
            tierDdLe.preferredWidth = 90f;
            tierDdLe.minWidth = 80f;
            tierDdLe.preferredHeight = 28f;

            // Filter row 2: State filter
            GameObject filterRow2 = new GameObject("FilterRow2", typeof(RectTransform));
            filterRow2.transform.SetParent(filterContainer.transform, false);
            HorizontalLayoutGroup filterHlg2 = filterRow2.AddComponent<HorizontalLayoutGroup>();
            filterHlg2.spacing = 4f;
            filterHlg2.childForceExpandWidth = false;
            filterHlg2.childForceExpandHeight = false;
            LayoutElement filterRow2Le = filterRow2.AddComponent<LayoutElement>();
            filterRow2Le.preferredHeight = 28f;
            filterRow2Le.flexibleWidth = 1f;

            Text stateLabel = UiBuilder.MakeText(filterRow2.transform, "StateLabel", L10n.T("menu.filter.state", "State:"), 11, UiBuilder.TextSecondary);
            LayoutElement stateLabelLe = stateLabel.gameObject.AddComponent<LayoutElement>();
            stateLabelLe.preferredWidth = 40f;

            _stateDropdown = MakeDropdown(filterRow2.transform, "StateDropdown",
                new[] { L10n.T("menu.filter.state.all", "All"), L10n.T("menu.filter.state.enabled", "Enabled"), L10n.T("menu.filter.state.disabled", "Disabled"), L10n.T("menu.filter.state.errors", "Has Errors") },
                OnStateFilterChanged);
            LayoutElement stateDdLe = _stateDropdown.gameObject.AddComponent<LayoutElement>();
            stateDdLe.preferredWidth = 90f;
            stateDdLe.minWidth = 80f;
            stateDdLe.preferredHeight = 28f;

            // Sort row
            GameObject sortRow = new GameObject("SortRow", typeof(RectTransform));
            sortRow.transform.SetParent(filterContainer.transform, false);
            HorizontalLayoutGroup sortHlg = sortRow.AddComponent<HorizontalLayoutGroup>();
            sortHlg.spacing = 4f;
            sortHlg.childForceExpandWidth = false;
            sortHlg.childForceExpandHeight = false;
            LayoutElement sortRowLe = sortRow.AddComponent<LayoutElement>();
            sortRowLe.preferredHeight = 28f;
            sortRowLe.flexibleWidth = 1f;

            Text sortLabel = UiBuilder.MakeText(sortRow.transform, "SortLabel", L10n.T("menu.sort.label", "Sort:"), 11, UiBuilder.TextSecondary);
            LayoutElement sortLabelLe = sortLabel.gameObject.AddComponent<LayoutElement>();
            sortLabelLe.preferredWidth = 40f;

            _sortDropdown = MakeDropdown(sortRow.transform, "SortDropdown",
                new[] { L10n.T("menu.sort.name", "Name"), L10n.T("menu.sort.type", "Type"), L10n.T("menu.sort.version", "Version") },
                OnSortChanged);
            LayoutElement sortDdLe = _sortDropdown.gameObject.AddComponent<LayoutElement>();
            sortDdLe.preferredWidth = 90f;
            sortDdLe.minWidth = 80f;
            sortDdLe.preferredHeight = 28f;

            // Counter text (flexible space, then counter)
            GameObject spacer = new GameObject("Spacer", typeof(RectTransform));
            spacer.transform.SetParent(sortRow.transform, false);
            LayoutElement spacerLe = spacer.AddComponent<LayoutElement>();
            spacerLe.flexibleWidth = 1f;

            _listCounterText = UiBuilder.MakeText(sortRow.transform, "CounterText", "0 of 0", 10, UiBuilder.TextSecondary);
            LayoutElement counterLe = _listCounterText.gameObject.AddComponent<LayoutElement>();
            counterLe.preferredWidth = 60f;
        }

        /// <summary>Creates a dropdown with the given options and callback.</summary>
        private Dropdown MakeDropdown(Transform parent, string name, string[] options, Action<int> onValueChanged)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image), typeof(Dropdown));
            go.transform.SetParent(parent, false);

            Image bgImg = go.GetComponent<Image>();
            bgImg.color = UiBuilder.BgDeep;

            Dropdown dropdown = go.GetComponent<Dropdown>();

            // Populate options
            dropdown.options.Clear();
            foreach (string option in options)
            {
                dropdown.options.Add(new Dropdown.OptionData(option));
            }
            dropdown.value = 0;

            // Setup label
            GameObject label = new GameObject("Label", typeof(RectTransform), typeof(Text));
            label.transform.SetParent(go.transform, false);
            RectTransform labelRt = label.GetComponent<RectTransform>();
            labelRt.anchorMin = Vector2.zero;
            labelRt.anchorMax = Vector2.one;
            labelRt.offsetMin = new Vector2(8f, 0f);
            labelRt.offsetMax = new Vector2(-8f, 0f);

            Text labelText = label.GetComponent<Text>();
            labelText.text = options[0];
            labelText.font = Resources.GetBuiltinResource<Font>("Arial.ttf");
            labelText.fontSize = 12;
            labelText.color = UiBuilder.TextPrimary;
            labelText.alignment = TextAnchor.MiddleLeft;

            dropdown.targetGraphic = bgImg;
            dropdown.onValueChanged.AddListener(new UnityEngine.Events.UnityAction<int>(onValueChanged));

            return dropdown;
        }

        private void OnSearchChanged(string text)
        {
            _searchText = text;
            ApplyFilters();
        }

        private void OnTierFilterChanged(int index)
        {
            if (_tierDropdown != null)
                _tierFilter = _tierDropdown.options[index].text;
            ApplyFilters();
        }

        private void OnStateFilterChanged(int index)
        {
            if (_stateDropdown != null)
                _stateFilter = _stateDropdown.options[index].text;
            ApplyFilters();
        }

        private void OnSortChanged(int index)
        {
            if (_sortDropdown != null)
                _sortBy = _sortDropdown.options[index].text;
            ApplyFilters();
        }

        /// <summary>Applies all filters and sorts the pack list, rebuilding the UI.</summary>
        private void ApplyFilters()
        {
            _filteredIndices.Clear();

            // Fix #944/B3: use dropdown VALUE (index) to compare filters, NOT localized option text.
            // Comparing against "All"/"Engine Extension"/etc. breaks once i18n renames the labels
            // (e.g. "All Tiers" in some locales). Index 0 always means "no filter" regardless of locale.
            int tierIndex = _tierDropdown != null ? _tierDropdown.value : 0;
            int stateIndex = _stateDropdown != null ? _stateDropdown.value : 0;

            for (int i = 0; i < _presenter.Packs.Count; i++)
            {
                PackDisplayInfo pack = _presenter.Packs[i];

                // Search filter
                if (!string.IsNullOrWhiteSpace(_searchText))
                {
                    string searchLower = _searchText.ToLowerInvariant();
                    if (!pack.Name.ToLowerInvariant().Contains(searchLower) &&
                        !pack.Id.ToLowerInvariant().Contains(searchLower))
                    {
                        continue;
                    }
                }

                // Tier filter (index 0 = All; 1 = Engine Extension, 2 = Content, 3 = Total Conversion, 4 = Baseline)
                if (tierIndex != 0)
                {
                    bool matches = false;
                    if (tierIndex == 1 && pack.Classification == "engine_extension") matches = true;
                    if (tierIndex == 2 && pack.Classification == "content") matches = true;
                    if (tierIndex == 3 && pack.Classification == "total_conversion") matches = true;
                    if (tierIndex == 4 && pack.Classification == "baseline") matches = true;
                    if (!matches) continue;
                }

                // State filter (index 0 = All; 1 = Enabled, 2 = Disabled, 3 = Has Errors)
                if (stateIndex != 0)
                {
                    bool matches = false;
                    if (stateIndex == 1 && pack.IsEnabled) matches = true;
                    if (stateIndex == 2 && !pack.IsEnabled) matches = true;
                    if (stateIndex == 3 && pack.Errors.Count > 0) matches = true;
                    if (!matches) continue;
                }

                _filteredIndices.Add(i);
            }

            // Apply sorting
            ApplySorting();

            // Update counter
            if (_listCounterText != null)
            {
                _listCounterText.text = $"{_filteredIndices.Count} of {_presenter.Packs.Count}";
            }

            // Rebuild the filtered list
            RebuildFilteredPackList();
        }

        /// <summary>Sorts the filtered pack indices according to the current sort selection.</summary>
        private void ApplySorting()
        {
            if (_sortBy == "Type")
            {
                _filteredIndices.Sort((a, b) =>
                    _presenter.Packs[a].Type.CompareTo(_presenter.Packs[b].Type));
            }
            else if (_sortBy == "Version")
            {
                // Simple string version sort (not semver)
                _filteredIndices.Sort((a, b) =>
                    _presenter.Packs[a].Version.CompareTo(_presenter.Packs[b].Version));
            }
            else // Name (default)
            {
                _filteredIndices.Sort((a, b) =>
                    _presenter.Packs[a].Name.CompareTo(_presenter.Packs[b].Name));
            }
        }

        /// <summary>Renders only the filtered packs into the scroll list.</summary>
        private void RebuildFilteredPackList()
        {
            if (_listContent == null) return;

            // Clear existing items
            for (int i = _listContent.childCount - 1; i >= 0; i--)
            {
                Transform child = _listContent.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            // Render filtered items
            for (int displayIndex = 0; displayIndex < _filteredIndices.Count; displayIndex++)
            {
                int realIndex = _filteredIndices[displayIndex];
                BuildPackListItem(_presenter.Packs[realIndex], realIndex);
            }

            // Update content height
            float padding_top = 4f, padding_bottom = 4f, spacing = 2f, itemHeight = 40f;
            float calculatedHeight = padding_top + (_filteredIndices.Count * itemHeight) +
                                      (Mathf.Max(0, _filteredIndices.Count - 1) * spacing) + padding_bottom;
            _listContent.sizeDelta = new Vector2(_listContent.sizeDelta.x, calculatedHeight);

            UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_listContent);
        }

        private void BuildPackListItem(PackDisplayInfo pack, int index)
        {
            if (_listContent == null)
            {
                _log?.LogWarning($"[ModMenuPanel.BuildPackListItem] _listContent is NULL for pack '{pack.Id}' — item {index} skipped.");
                return;
            }

            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Starting item {index}: '{pack.Name}' (enabled={pack.IsEnabled}, selected={index == _presenter.SelectedIndex})");

            bool isSelected = index == _presenter.SelectedIndex;
            bool hasErrors = pack.Errors.Count > 0;
            bool hasConflicts = pack.Conflicts.Count > 0;

            // Zebra striping: alternate row colors
            // Even rows (#1A1F2E) / Odd rows (#1F2536)
            bool isEvenRow = index % 2 == 0;
            Color bgColor = isSelected ? UiBuilder.BgSurface : (isEvenRow ? HexColor("#1A1F2E") : HexColor("#1F2536"));
            Color alpha = pack.IsEnabled ? Color.white : new Color(1f, 1f, 1f, 0.6f);

            GameObject card = UiBuilder.MakePanel(_listContent, $"PackItem_{pack.Id}", bgColor, new Vector2(0f, ItemHeight));
            RectTransform cardRt = card.GetComponent<RectTransform>();
            cardRt.SetSizeWithCurrentAnchors(RectTransform.Axis.Horizontal, ListWidth - 8f);
            cardRt.sizeDelta = new Vector2(ListWidth - 8f, ItemHeight);
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} card created: sizeDelta={cardRt.sizeDelta}, active={card.activeSelf}");

            LayoutElement cardLe = card.AddComponent<LayoutElement>();
            cardLe.minWidth = ListWidth - 8f;
            cardLe.preferredWidth = ListWidth - 8f;
            cardLe.minHeight = ItemHeight;
            cardLe.preferredHeight = ItemHeight;
            cardLe.flexibleWidth = 1f;
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} LayoutElement set: minHeight={cardLe.minHeight}, preferredHeight={cardLe.preferredHeight}");

            // Content layout with improved spacing (8px padding, 4px rows)
            HorizontalLayoutGroup hlg = card.AddComponent<HorizontalLayoutGroup>();
            hlg.spacing = 6f;
            hlg.padding = new RectOffset(8, 8, 4, 4);
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;
            hlg.childControlWidth = true;
            hlg.childControlHeight = true;

            // Status indicator dot (green=enabled, red=error, yellow=conflict)
            Color dotColor = hasErrors ? UiBuilder.Error : (hasConflicts ? UiBuilder.Warning : (pack.IsEnabled ? UiBuilder.Success : new Color(0.5f, 0.5f, 0.5f, 1f)));
            GameObject dotGo = UiBuilder.MakePanel(card.transform, "StatusDot", dotColor, new Vector2(8f, 8f));
            RectTransform dotRt = dotGo.GetComponent<RectTransform>();
            LayoutElement dotLe = dotGo.AddComponent<LayoutElement>();
            dotLe.preferredWidth = 8f;
            dotLe.preferredHeight = 8f;
            dotLe.minWidth = 8f;
            dotLe.minHeight = 8f;

            // Pack name
            Color nameColor = pack.IsEnabled ? UiBuilder.TextPrimary : UiBuilder.TextSecondary;
            Text nameText = UiBuilder.MakeText(card.transform, "PackName", pack.Name, 13,
                nameColor, bold: isSelected);
            RectTransform nameTextRt = nameText.GetComponent<RectTransform>();
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} nameText created: text='{pack.Name}', fontSize={nameText.fontSize}, color={nameColor}, sizeDelta={nameTextRt.sizeDelta}, font={nameText.font?.name}");

            if (!pack.IsEnabled)
            {
                nameText.color = new Color(nameColor.r, nameColor.g, nameColor.b, 0.6f);
            }
            LayoutElement nameLe = nameText.gameObject.AddComponent<LayoutElement>();
            nameLe.minWidth = 100f;
            nameLe.flexibleWidth = 1f;
            nameLe.minHeight = 16f;
            nameLe.preferredHeight = ItemHeight - 8f;
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} nameText LayoutElement: minWidth={nameLe.minWidth}, flexibleWidth={nameLe.flexibleWidth}, minHeight={nameLe.minHeight}");

            // Error / Conflict badge
            if (hasErrors)
            {
                GameObject badge = UiBuilder.MakePanel(card.transform, "ErrorBadge",
                    UiBuilder.Error, new Vector2(32f, 18f));
                LayoutElement badgeLe = badge.AddComponent<LayoutElement>();
                badgeLe.preferredWidth = 32f;
                badgeLe.preferredHeight = 18f;

                Text badgeText = UiBuilder.MakeText(badge.transform, "BadgeText", "ERR",
                    10, Color.white, bold: true, TextAnchor.MiddleCenter);
                UiBuilder.FillParent(badgeText.GetComponent<RectTransform>());
            }
            else if (hasConflicts)
            {
                GameObject badge = UiBuilder.MakePanel(card.transform, "ConflictBadge",
                    UiBuilder.Warning, new Vector2(40f, 18f));
                LayoutElement badgeLe = badge.AddComponent<LayoutElement>();
                badgeLe.preferredWidth = 40f;
                badgeLe.preferredHeight = 18f;

                Text badgeText = UiBuilder.MakeText(badge.transform, "BadgeText", "CONF",
                    10, Color.black, bold: true, TextAnchor.MiddleCenter);
                UiBuilder.FillParent(badgeText.GetComponent<RectTransform>());
            }

            // Version label
            Text versionText = UiBuilder.MakeText(card.transform, "Version",
                $"v{pack.Version}", 11, UiBuilder.TextSecondary);
            LayoutElement verLe = versionText.gameObject.AddComponent<LayoutElement>();
            verLe.preferredWidth = 50f;
            verLe.minWidth = 40f;
            verLe.minHeight = 16f;

            // Click to select
            int capturedIndex = index;
            Button btn = card.AddComponent<Button>();
            ColorBlock cb = btn.colors;
            cb.normalColor = bgColor;
            // Hover: lighten by ~10%
            cb.highlightedColor = Color.Lerp(bgColor, Color.white, 0.1f);
            cb.pressedColor = Color.Lerp(bgColor, Color.black, 0.15f);
            cb.selectedColor = bgColor;
            cb.colorMultiplier = 1f;
            cb.fadeDuration = 0.05f;
            btn.colors = cb;
            btn.targetGraphic = card.GetComponent<Image>();
            btn.onClick.AddListener(() => SelectPack(capturedIndex));
        }

        private void SelectPack(int index)
        {
            _presenter.SelectIndex(index);
            ClearCurrentSelection();
            QueueListRefresh();
        }

        private void RefreshDetail()
        {
            PackDisplayInfo? selected = _presenter.SelectedPack;
            if (selected == null)
            {
                if (_detailName != null) _detailName.text = "Select a pack";
                if (_detailMeta != null) _detailMeta.text = "";
                if (_detailLicense != null) _detailLicense.text = "";
                if (_detailDesc != null) _detailDesc.text = "";
                if (_detailDeps != null) _detailDeps.text = "Dependencies: none";
                if (_detailConflicts != null) _detailConflicts.text = "Conflicts: none";
                if (_detailLoadOrder != null) _detailLoadOrder.text = "Load Order: —";
                if (_detailContent != null) _detailContent.text = "Content: (none)";
                if (_detailDetectedConflicts != null) _detailDetectedConflicts.text = "";
                if (_detailTagsRow != null) _detailTagsRow.gameObject.SetActive(false);
                if (_detailLinksRow != null) _detailLinksRow.gameObject.SetActive(false);
                if (_detailGalleryRow != null) _detailGalleryRow.gameObject.SetActive(false);
                if (_detailBadgesRow != null) { UnityEngine.Object.Destroy(_detailBadgesRow); _detailBadgesRow = null; }
                return;
            }

            PackDisplayInfo p = selected;

            // ── Name ──────────────────────────────────────────────────────────
            if (_detailName != null) _detailName.text = p.Name;

            // ── Badges row (#928-935) — rendered above meta for visibility ─────
            RefreshBadgesRow(p);

            // ── Meta + license badge ──────────────────────────────────────────
            if (_detailMeta != null) _detailMeta.text = $"by {p.Author}  ·  {p.Type}  ·  v{p.Version}";
            if (_detailLicense != null)
            {
                if (!string.IsNullOrEmpty(p.License))
                {
                    _detailLicense.text = $"[{p.License}]";
                    _detailLicense.gameObject.SetActive(true);
                }
                else
                {
                    _detailLicense.text = "";
                    _detailLicense.gameObject.SetActive(false);
                }
            }

            // ── Tags row ──────────────────────────────────────────────────────
            RefreshTagsRow(p);

            // ── Description ───────────────────────────────────────────────────
            if (_detailDesc != null)
            {
                string descText = string.IsNullOrEmpty(p.Description)
                    ? "(no description)"
                    : p.Description!;

                if (p.Errors.Count > 0)
                {
                    descText = descText + "\n\n<color=#e05252>Errors:</color>\n"
                        + string.Join("\n", p.Errors);
                }

                _detailDesc.text = descText;
            }

            // ── External links row ────────────────────────────────────────────
            RefreshLinksRow(p);

            // ── Deps / Conflicts / Load order ─────────────────────────────────
            if (_detailDeps != null)
            {
                _detailDeps.text = p.Dependencies.Count == 0
                    ? "Dependencies: none"
                    : "Dependencies: " + string.Join(", ", p.Dependencies);
            }

            if (_detailConflicts != null)
            {
                _detailConflicts.text = p.Conflicts.Count == 0
                    ? "Conflicts: none"
                    : "<color=#e8a020>Conflicts: " + string.Join(", ", p.Conflicts) + "</color>";
            }

            if (_detailLoadOrder != null)
            {
                _detailLoadOrder.text = $"Load Order: {p.LoadOrder}";
            }

            // ── Rich content section ──────────────────────────────────────────
            if (_detailRichContent != null)
            {
                System.Text.StringBuilder sb = new System.Text.StringBuilder(512);
                if (p.ContentSummary.Count == 0)
                {
                    sb.Append(L10n.T("menu.detail.content", "Content: (none declared)"));
                }
                else
                {
                    sb.Append("<color=#88dd88>Content:</color>\n");
                    foreach (System.Collections.Generic.KeyValuePair<string, int> kv in p.ContentSummary)
                    {
                        string label = kv.Key.Replace('_', ' ');
                        sb.Append($"  {kv.Value} {label}\n");
                    }

                    // Show first N unit / building / faction names as previews
                    if (p.UnitNames.Count > 0)
                    {
                        int shown = System.Math.Min(5, p.UnitNames.Count);
                        string preview = string.Join(", ", SubList(p.UnitNames, shown));
                        string more = p.UnitNames.Count > shown ? $" and {p.UnitNames.Count - shown} more" : "";
                        sb.Append($"\n<color=#aaddaa>Units: </color>{preview}{more}\n");
                    }
                    if (p.BuildingNames.Count > 0)
                    {
                        int shown = System.Math.Min(3, p.BuildingNames.Count);
                        string preview = string.Join(", ", SubList(p.BuildingNames, shown));
                        string more = p.BuildingNames.Count > shown ? $" and {p.BuildingNames.Count - shown} more" : "";
                        sb.Append($"<color=#aaddaa>Buildings: </color>{preview}{more}\n");
                    }
                    if (p.FactionNames.Count > 0)
                    {
                        string factionList = string.Join(", ", p.FactionNames);
                        sb.Append($"<color=#aaddaa>Factions: </color>{factionList}\n");
                    }
                }

                _detailRichContent.text = sb.ToString().TrimEnd('\n');
            }

            // ── Detected conflicts ────────────────────────────────────────────
            if (_detailDetectedConflicts != null)
            {
                if (p.DetectedConflicts.Count == 0)
                {
                    _detailDetectedConflicts.text = "";
                }
                else
                {
                    _detailDetectedConflicts.text = "<color=#e8a020>Content Overlaps:</color>\n"
                        + string.Join("\n", p.DetectedConflicts);
                }
            }

            // ── Screenshot gallery (lazy-load textures via coroutine) ─────────
            RefreshGallery(p);

            // ── Conflict resolution buttons (#903) ────────────────────────────
            RefreshConflictButtons(p);

            // ── Per-pack runtime settings (#925) ───────────────────────────────
            RefreshSettings(p);

            // ── Toggle button label ───────────────────────────────────────────
            if (_detailPane != null)
            {
                Transform btnRow = _detailPane.transform.Find("ActionButtons");
                if (btnRow != null)
                {
                    Transform toggleBtnT = btnRow.Find("ToggleBtn");
                    if (toggleBtnT != null)
                    {
                        Text? btnLabel = toggleBtnT.Find("Label")?.GetComponent<Text>();
                        if (btnLabel != null)
                            btnLabel.text = p.IsEnabled ? "Disable" : "Enable";
                    }
                }
            }
        }

        /// <summary>
        /// Rebuilds the badge row for the selected pack (#928-935).
        /// Destroys any previous row, then delegates to <see cref="DINOForge.Runtime.UI.Badges.BadgeRenderer"/>
        /// to create the new one.  The row is inserted as the first child of the detail scroll content
        /// so badges appear at the very top of the detail pane.
        /// </summary>
        private void RefreshBadgesRow(PackDisplayInfo p)
        {
            // Destroy previous dynamic row
            if (_detailBadgesRow != null)
            {
                Destroy(_detailBadgesRow);
                _detailBadgesRow = null;
            }

            if (p.Badges.Count == 0) return;

            // Find the detail scroll content (parent of _detailName)
            Transform? contentParent = _detailName?.transform.parent;
            if (contentParent == null) return;

            _detailBadgesRow = DINOForge.Runtime.UI.Badges.BadgeRenderer.BuildBadgeRow(contentParent, p.Badges);

            // Move to just before the pack name (index 0) so badges appear at the top
            _detailBadgesRow.transform.SetSiblingIndex(0);
        }

        /// <summary>Populates the tags chip row for the selected pack.</summary>
        private void RefreshTagsRow(PackDisplayInfo p)
        {
            if (_detailTagsRow == null) return;
            // Clear existing chips
            for (int i = _detailTagsRow.childCount - 1; i >= 0; i--)
            {
                Transform child = _detailTagsRow.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            if (p.Tags.Count == 0)
            {
                _detailTagsRow.gameObject.SetActive(false);
                return;
            }

            _detailTagsRow.gameObject.SetActive(true);
            // Cycle through a small set of muted accent colours for variety
            Color[] chipColors = new Color[]
            {
                UiBuilder.HexColor("#2a4a38", 1f),
                UiBuilder.HexColor("#3a3420", 1f),
                UiBuilder.HexColor("#2a3a4a", 1f),
                UiBuilder.HexColor("#4a2a38", 1f),
            };
            int colorIdx = 0;
            foreach (string tag in p.Tags)
            {
                if (string.IsNullOrWhiteSpace(tag)) continue;
                Color chipBg = chipColors[colorIdx % chipColors.Length];
                colorIdx++;

                GameObject chip = UiBuilder.MakePanel(_detailTagsRow, $"Tag_{tag}",
                    chipBg, new Vector2(0f, 20f));
                HorizontalLayoutGroup chipHlg = chip.AddComponent<HorizontalLayoutGroup>();
                chipHlg.padding = new RectOffset(6, 6, 2, 2);
                chipHlg.childForceExpandWidth = false;
                LayoutElement chipLe = chip.AddComponent<LayoutElement>();
                chipLe.preferredHeight = 20f;

                Text chipText = UiBuilder.MakeText(chip.transform, "TagLabel", tag, 10,
                    UiBuilder.TextPrimary, bold: false, TextAnchor.MiddleCenter);
                LayoutElement textLe = chipText.gameObject.AddComponent<LayoutElement>();
                textLe.preferredWidth = tag.Length * 6.5f + 12f; // threshold-ok: character-width estimate
                textLe.minWidth = 20f;
            }
        }

        /// <summary>
        /// Returns true only for absolute http/https URLs.
        /// Prevents javascript:, file:, and other dangerous schemes from reaching
        /// <see cref="Application.OpenURL"/>.
        /// </summary>
        private static bool IsSafeWebUrl(string? url)
        {
            if (string.IsNullOrWhiteSpace(url)) return false;
            if (!Uri.TryCreate(url, UriKind.Absolute, out Uri? uri)) return false;
            return uri.Scheme == Uri.UriSchemeHttp || uri.Scheme == Uri.UriSchemeHttps;
        }

        /// <summary>Populates the external-links button row (homepage, GitHub, Discord).</summary>
        private void RefreshLinksRow(PackDisplayInfo p)
        {
            if (_detailLinksRow == null) return;
            // Clear existing buttons
            for (int i = _detailLinksRow.childCount - 1; i >= 0; i--)
            {
                Transform child = _detailLinksRow.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            bool hasLinks = !string.IsNullOrEmpty(p.HomepageUrl)
                || !string.IsNullOrEmpty(p.GithubUrl)
                || !string.IsNullOrEmpty(p.DiscordUrl);

            if (!hasLinks)
            {
                _detailLinksRow.gameObject.SetActive(false);
                return;
            }

            _detailLinksRow.gameObject.SetActive(true);

            void MakeLinkBtn(string label, string? url, Color accent)
            {
                if (string.IsNullOrEmpty(url)) return;
                string capturedUrl = url!;
                Button btn = UiBuilder.MakeButton(_detailLinksRow, $"Btn_{label}",
                    label, UiBuilder.BgSurface, accent,
                    () =>
                    {
                        if (IsSafeWebUrl(capturedUrl))
                        {
                            try { Application.OpenURL(capturedUrl); }
                            catch { } // safe-swallow: OpenURL is best-effort
                        }
                        else
                        {
                            _log?.LogWarning($"[ModMenuPanel] Refusing unsafe URL: {capturedUrl}");
                        }
                    });
                LayoutElement le = btn.gameObject.AddComponent<LayoutElement>();
                le.preferredWidth = label.Length * 7f + 20f; // threshold-ok: button width estimate
                le.minWidth = 60f;
                le.preferredHeight = 24f;
            }

            MakeLinkBtn("🌐 Homepage", p.HomepageUrl, UiBuilder.Accent);
            MakeLinkBtn("⌥ GitHub", p.GithubUrl, UiBuilder.TextPrimary);
            MakeLinkBtn("💬 Discord", p.DiscordUrl, UiBuilder.HexColor("#7289da", 1f));
        }

        /// <summary>Populates the screenshot gallery strip; lazy-loads PNG/JPG textures via coroutine.</summary>
        private void RefreshGallery(PackDisplayInfo p)
        {
            if (_detailGalleryRow == null) return;

            // Clear old thumbnails
            _galleryThumbs.Clear();
            Transform galleryScroll = _detailGalleryRow.Find("GalleryScroll");
            if (galleryScroll != null)
            {
                Transform content = galleryScroll.Find("Content");
                if (content != null)
                {
                    for (int i = content.childCount - 1; i >= 0; i--)
                    {
                        Transform child = content.GetChild(i);
                        child.SetParent(null, false);
                        Destroy(child.gameObject);
                    }
                }
            }

            if (p.ScreenshotPaths.Count == 0)
            {
                _detailGalleryRow.gameObject.SetActive(false);
                return;
            }

            _detailGalleryRow.gameObject.SetActive(true);

            if (galleryScroll == null) return;
            Transform galleryContent = galleryScroll.Find("Content");
            if (galleryContent == null) return;

            // Spawn placeholder cards and kick off the texture load coroutine
            for (int i = 0; i < p.ScreenshotPaths.Count; i++)
            {
                int capturedIdx = i;
                string imgPath = p.ScreenshotPaths[i];

                // Thumbnail card (200 × 150)
                GameObject thumbCard = UiBuilder.MakePanel(galleryContent, $"Thumb_{i}",
                    UiBuilder.BgSurface, new Vector2(200f, 150f));
                LayoutElement thumbLe = thumbCard.AddComponent<LayoutElement>();
                thumbLe.preferredWidth = 200f;
                thumbLe.minWidth = 200f;
                thumbLe.preferredHeight = 150f;
                thumbLe.minHeight = 150f;

                // The image placeholder (starts grey, filled by coroutine)
                Image thumbImg = thumbCard.GetComponent<Image>();
                thumbImg.preserveAspect = true;
                _galleryThumbs.Add(thumbImg);

                // "Loading…" label
                Text loadingTxt = UiBuilder.MakeText(thumbCard.transform, "Loading",
                    "…", 11, UiBuilder.TextSecondary, bold: false, TextAnchor.MiddleCenter);
                UiBuilder.FillParent(loadingTxt.GetComponent<RectTransform>());

                // Click to open full-size in modal
                Button thumbBtn = thumbCard.AddComponent<Button>();
                thumbBtn.targetGraphic = thumbImg;
                int idx = capturedIdx;
                thumbBtn.onClick.AddListener(() => OpenScreenshotModal(idx));

                // Start lazy texture load
                StartCoroutine(LoadTextureAsync(imgPath, thumbImg, loadingTxt));
            }
        }

        /// <summary>Coroutine: loads an image file from disk into a Texture2D and assigns it to target.</summary>
        private System.Collections.IEnumerator LoadTextureAsync(string path, Image target, Text loadingLabel)
        {
            yield return null; // yield once so we don't block the frame

            Texture2D? tex = null;
            try
            {
                byte[] bytes = System.IO.File.ReadAllBytes(path);
                tex = new Texture2D(2, 2, TextureFormat.RGBA32, false);
                if (!tex.LoadImage(bytes))
                {
                    Destroy(tex);
                    tex = null;
                }
            }
            catch (Exception ex)
            {
                // safe-swallow: screenshot load is best-effort UI decoration
                System.Diagnostics.Debug.WriteLine($"ModMenuPanel screenshot load failed: {ex.Message}");
            }

            if (target == null) yield break; // UI was destroyed while loading

            if (tex != null)
            {
                target.sprite = Sprite.Create(tex, new Rect(0, 0, tex.width, tex.height),
                    new Vector2(0.5f, 0.5f));
                target.color = Color.white;
                if (loadingLabel != null) Destroy(loadingLabel.gameObject);
            }
            else
            {
                if (loadingLabel != null) loadingLabel.text = "⚠";
            }
        }

        /// <summary>Opens the full-size screenshot modal for the given gallery index.</summary>
        private void OpenScreenshotModal(int thumbIndex)
        {
            if (_screenshotModal == null || _modalImage == null) return;
            if (thumbIndex < 0 || thumbIndex >= _galleryThumbs.Count) return;

            Image thumbImg = _galleryThumbs[thumbIndex];
            if (thumbImg == null || thumbImg.sprite == null) return;

            _modalImage.sprite = thumbImg.sprite;
            _screenshotModal.SetActive(true);
        }

        /// <summary>Returns the first <paramref name="count"/> elements of a read-only list.</summary>
        private static IEnumerable<string> SubList(IReadOnlyList<string> list, int count)
        {
            for (int i = 0; i < count && i < list.Count; i++)
                yield return list[i];
        }

        // ── Conflict resolution UI (#903) ──────────────────────────────────────────

        /// <summary>
        /// Builds the full-screen diff modal (hidden by default).
        /// Called once from <see cref="BuildDetailPane"/>.
        /// </summary>
        private void BuildDiffModal()
        {
            if (_panelRt == null) return;

            _diffModal = new GameObject("DiffModal", typeof(RectTransform));
            _diffModal.transform.SetParent(_panelRt, false);
            UiBuilder.FillParent(_diffModal.GetComponent<RectTransform>());

            Image dimmerImg = _diffModal.AddComponent<Image>();
            dimmerImg.color = new Color(0f, 0f, 0f, 0.88f);
            dimmerImg.raycastTarget = true;

            GameObject card = UiBuilder.MakePanel(_diffModal.transform, "DiffCard",
                UiBuilder.BgSurface, new Vector2(640f, 440f));
            RectTransform cardRt = card.GetComponent<RectTransform>();
            cardRt.anchorMin = new Vector2(0.5f, 0.5f);
            cardRt.anchorMax = new Vector2(0.5f, 0.5f);
            cardRt.pivot = new Vector2(0.5f, 0.5f);
            cardRt.anchoredPosition = Vector2.zero;

            VerticalLayoutGroup cardVlg = card.AddComponent<VerticalLayoutGroup>();
            cardVlg.spacing = 6f;
            cardVlg.padding = new RectOffset(12, 12, 10, 10);
            cardVlg.childForceExpandWidth = true;
            cardVlg.childForceExpandHeight = false;

            // Title row: heading + close button
            GameObject titleRow = new GameObject("TitleRow", typeof(RectTransform));
            titleRow.transform.SetParent(card.transform, false);
            HorizontalLayoutGroup titleHlg = titleRow.AddComponent<HorizontalLayoutGroup>();
            titleHlg.spacing = 8f;
            titleHlg.childForceExpandWidth = false;
            titleHlg.childForceExpandHeight = false;
            titleRow.AddComponent<LayoutElement>().preferredHeight = 24f;

            _diffModalTitle = UiBuilder.MakeText(titleRow.transform, "DiffTitle",
                "Conflict Diff", 14, UiBuilder.Accent, bold: true);
            _diffModalTitle.gameObject.AddComponent<LayoutElement>().flexibleWidth = 1f;

            GameObject diffModalRef = _diffModal; // capture for closure
            Button diffCloseBtn = UiBuilder.MakeButton(titleRow.transform, "DiffCloseBtn", "X",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () => diffModalRef.SetActive(false));
            LayoutElement diffCloseLe = diffCloseBtn.gameObject.AddComponent<LayoutElement>();
            diffCloseLe.preferredWidth = 24f;
            diffCloseLe.preferredHeight = 24f;

            UiBuilder.MakeHorizontalSeparator(card.transform, UiBuilder.Border);

            // Two-column scroll area
            GameObject cols = new GameObject("Columns", typeof(RectTransform));
            cols.transform.SetParent(card.transform, false);
            HorizontalLayoutGroup colsHlg = cols.AddComponent<HorizontalLayoutGroup>();
            colsHlg.spacing = 6f;
            colsHlg.childForceExpandHeight = true;
            colsHlg.childForceExpandWidth = false;
            LayoutElement colsLe = cols.AddComponent<LayoutElement>();
            colsLe.flexibleWidth = 1f;
            colsLe.preferredHeight = 360f;
            colsLe.minHeight = 200f; // threshold-ok: minimum readable diff panel height

            _diffLeftText = BuildDiffColumn(cols.transform, "LeftColumn", "Pack A");
            _diffRightText = BuildDiffColumn(cols.transform, "RightColumn", "Pack B");

            _diffModal.SetActive(false);
        }

        /// <summary>Builds one scrollable text column inside the diff modal.</summary>
        private static Text BuildDiffColumn(Transform parent, string name, string headerLabel)
        {
            GameObject col = new GameObject(name, typeof(RectTransform));
            col.transform.SetParent(parent, false);
            VerticalLayoutGroup colVlg = col.AddComponent<VerticalLayoutGroup>();
            colVlg.spacing = 4f;
            colVlg.childForceExpandWidth = true;
            colVlg.childForceExpandHeight = false;
            LayoutElement colLe = col.AddComponent<LayoutElement>();
            colLe.flexibleWidth = 1f;
            colLe.flexibleHeight = 1f;

            Text colHeader = UiBuilder.MakeText(col.transform, "Header", headerLabel, 12,
                UiBuilder.Accent, bold: true);
            colHeader.gameObject.AddComponent<LayoutElement>().preferredHeight = 18f;

            (ScrollRect colScroll, RectTransform colContent) =
                UiBuilder.MakeScrollView(col.transform, name + "Scroll", Vector2.zero);
            LayoutElement colScrollLe = colScroll.gameObject.AddComponent<LayoutElement>();
            colScrollLe.flexibleWidth = 1f;
            colScrollLe.flexibleHeight = 1f;
            colScrollLe.minHeight = 140f; // threshold-ok: minimum readable diff scroll height

            VerticalLayoutGroup colContentVlg = colContent.GetComponent<VerticalLayoutGroup>();
            if (colContentVlg != null) colContentVlg.padding = new RectOffset(6, 6, 4, 4);

            Text bodyText = UiBuilder.MakeText(colContent, "Body", "", 10, UiBuilder.TextPrimary);
            bodyText.verticalOverflow = VerticalWrapMode.Overflow;
            bodyText.horizontalOverflow = HorizontalWrapMode.Wrap;
            bodyText.gameObject.AddComponent<LayoutElement>().flexibleWidth = 1f;

            return bodyText;
        }

        /// <summary>Opens the diff modal populated with pack.yaml content for both packs.</summary>
        private void OpenDiffModal(string selfPackId, string otherPackId, string overlapSummary)
        {
            if (_diffModal == null || _diffModalTitle == null ||
                _diffLeftText == null || _diffRightText == null) return;

            _diffModalTitle.text = "Diff: " + selfPackId + "  vs  " + otherPackId;
            _diffLeftText.text = LoadPackYamlPreview(selfPackId, overlapSummary);
            _diffRightText.text = LoadPackYamlPreview(otherPackId, overlapSummary);
            _diffModal.SetActive(true);
        }

        /// <summary>
        /// Reads pack.yaml for the given pack ID and returns a preview string (first 120 lines).
        /// Falls back to an error notice if the file cannot be read.
        /// </summary>
        private string LoadPackYamlPreview(string packId, string overlapSummary)
        {
            if (string.IsNullOrEmpty(_packsDirectory)) return "(packs directory not set)";
            string yamlPath = Path.Combine(_packsDirectory, packId, "pack.yaml");
            try
            {
                if (!File.Exists(yamlPath)) return "(pack.yaml not found for " + packId + ")";
                string yaml = File.ReadAllText(yamlPath, System.Text.Encoding.UTF8);
                string[] lines = yaml.Split('\n');
                int lineLimit = System.Math.Min(lines.Length, 120); // threshold-ok: modal line display limit
                System.Text.StringBuilder sb = new System.Text.StringBuilder(2048);
                sb.Append("=== ").Append(packId).Append(" / pack.yaml ===\n");
                sb.Append("(overlap: ").Append(overlapSummary).Append(")\n\n");
                for (int i = 0; i < lineLimit; i++)
                {
                    sb.Append(lines[i]);
                    sb.Append('\n');
                }
                if (lines.Length > lineLimit)
                    sb.Append("\n... (").Append(lines.Length - lineLimit).Append(" more lines)");
                return sb.ToString();
            }
            catch (Exception ex)
            {
                // safe-swallow: diff preview is best-effort UI decoration
                System.Diagnostics.Debug.WriteLine($"ModMenuPanel pack.yaml preview failed for {packId}: {ex.Message}");
                return "(error reading pack.yaml for " + packId + ")";
            }
        }

        /// <summary>
        /// Rebuilds the interactive conflict-resolution button rows for the selected pack.
        /// Called from <see cref="RefreshDetail"/> after the gallery section.
        /// Hides the conflict section entirely when there are no detected conflicts.
        /// </summary>
        private void RefreshConflictButtons(PackDisplayInfo p)
        {
            if (_conflictSection == null) return;

            for (int i = _conflictSection.childCount - 1; i >= 0; i--)
            {
                Transform child = _conflictSection.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            if (p.DetectedConflicts.Count == 0)
            {
                _conflictSection.gameObject.SetActive(false);
                return;
            }

            _conflictSection.gameObject.SetActive(true);

            Text conflictHeader = UiBuilder.MakeText(_conflictSection, "ConflictHeader",
                "Content Overlaps  --  Resolution", 12, UiBuilder.Warning, bold: true);
            conflictHeader.gameObject.AddComponent<LayoutElement>().preferredHeight = 18f;

            UiBuilder.MakeHorizontalSeparator(_conflictSection, UiBuilder.Border);

            foreach (string conflict in p.DetectedConflicts)
                BuildConflictRow(p.Id, conflict);
        }

        /// <summary>
        /// Builds one conflict row: description + status label + 4 action buttons.
        /// DetectedConflicts format: "warfare-starwars also loads: factions, units"
        /// </summary>
        private void BuildConflictRow(string selfPackId, string conflictEntry)
        {
            if (_conflictSection == null) return;

            string otherPackId = conflictEntry;
            string overlapSummary = conflictEntry;

            int alsoIdx = conflictEntry.IndexOf(" also loads:", StringComparison.Ordinal);
            if (alsoIdx > 0)
            {
                otherPackId = conflictEntry.Substring(0, alsoIdx).Trim();
                overlapSummary = conflictEntry.Substring(alsoIdx + " also loads:".Length).Trim();
            }

            string capturedSelf = selfPackId;
            string capturedOther = otherPackId;
            string capturedOverlap = overlapSummary;

            GameObject row = new GameObject("ConflictRow_" + otherPackId, typeof(RectTransform));
            row.transform.SetParent(_conflictSection, false);
            VerticalLayoutGroup rowVlg = row.AddComponent<VerticalLayoutGroup>();
            rowVlg.spacing = 4f;
            rowVlg.childForceExpandWidth = true;
            rowVlg.childForceExpandHeight = false;
            rowVlg.padding = new RectOffset(0, 0, 2, 6);
            ContentSizeFitter rowCsf = row.AddComponent<ContentSizeFitter>();
            rowCsf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;
            row.AddComponent<LayoutElement>().flexibleWidth = 1f;

            string descText = "Pack \"" + otherPackId + "\" overlaps on: " + overlapSummary;
            Text descLabel = UiBuilder.MakeText(row.transform, "ConflictDesc",
                descText, 11, UiBuilder.Warning);
            descLabel.verticalOverflow = VerticalWrapMode.Overflow;
            descLabel.horizontalOverflow = HorizontalWrapMode.Wrap;
            LayoutElement descLe = descLabel.gameObject.AddComponent<LayoutElement>();
            descLe.flexibleWidth = 1f;
            descLe.preferredHeight = 28f;

            Text statusLabel = UiBuilder.MakeText(row.transform, "ResolutionStatus",
                GetResolutionStatusText(capturedSelf, capturedOther), 10, UiBuilder.TextSecondary);
            statusLabel.gameObject.AddComponent<LayoutElement>().preferredHeight = 14f;

            GameObject btnRow = new GameObject("ResolutionBtns", typeof(RectTransform));
            btnRow.transform.SetParent(row.transform, false);
            HorizontalLayoutGroup btnHlg = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnHlg.spacing = 4f;
            btnHlg.childForceExpandWidth = false;
            btnHlg.childForceExpandHeight = false;
            btnHlg.padding = new RectOffset(0, 0, 2, 0);
            btnRow.AddComponent<LayoutElement>().preferredHeight = 26f;

            Text capturedStatus = statusLabel;

            Button useSelfBtn = UiBuilder.MakeButton(btnRow.transform, "Btn_prefer_self",
                "Use This Pack", UiBuilder.BgDeep, UiBuilder.Success,
                () =>
                {
                    _conflictStore?.SetResolution(capturedSelf, capturedOther, "prefer_self");
                    _conflictStore?.Save();
                    if (capturedStatus != null)
                        capturedStatus.text = GetResolutionStatusText(capturedSelf, capturedOther);
                });
            LayoutElement useSelfLe = useSelfBtn.gameObject.AddComponent<LayoutElement>();
            useSelfLe.preferredWidth = 95f;
            useSelfLe.minWidth = 80f;
            useSelfLe.preferredHeight = 24f;

            Button useOtherBtn = UiBuilder.MakeButton(btnRow.transform, "Btn_prefer_other",
                "Use Other Pack", UiBuilder.BgDeep, UiBuilder.Warning,
                () =>
                {
                    _conflictStore?.SetResolution(capturedSelf, capturedOther, "prefer_other");
                    _conflictStore?.Save();
                    if (capturedStatus != null)
                        capturedStatus.text = GetResolutionStatusText(capturedSelf, capturedOther);
                });
            LayoutElement useOtherLe = useOtherBtn.gameObject.AddComponent<LayoutElement>();
            useOtherLe.preferredWidth = 100f;
            useOtherLe.minWidth = 85f;
            useOtherLe.preferredHeight = 24f;

            Button mergeBtn = UiBuilder.MakeButton(btnRow.transform, "Btn_merge",
                "Keep Both", UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () =>
                {
                    _conflictStore?.SetResolution(capturedSelf, capturedOther, "merge");
                    _conflictStore?.Save();
                    if (capturedStatus != null)
                        capturedStatus.text = GetResolutionStatusText(capturedSelf, capturedOther);
                });
            LayoutElement mergeLe = mergeBtn.gameObject.AddComponent<LayoutElement>();
            mergeLe.preferredWidth = 80f;
            mergeLe.minWidth = 70f;
            mergeLe.preferredHeight = 24f;

            Button showDiffBtn = UiBuilder.MakeButton(btnRow.transform, "Btn_ShowDiff",
                "Show Diff", UiBuilder.BgSurface, UiBuilder.Accent,
                () => OpenDiffModal(capturedSelf, capturedOther, capturedOverlap));
            LayoutElement showDiffLe = showDiffBtn.gameObject.AddComponent<LayoutElement>();
            showDiffLe.preferredWidth = 80f;
            showDiffLe.minWidth = 70f;
            showDiffLe.preferredHeight = 24f;
        }

        /// <summary>Returns a human-readable summary of the stored resolution for the given pack pair.</summary>
        private string GetResolutionStatusText(string selfId, string otherId)
        {
            string? res = _conflictStore?.GetResolution(selfId, otherId);
            if (string.IsNullOrEmpty(res) || string.Equals(res, "merge", StringComparison.Ordinal))
                return "Resolution: Keep Both (default)";
            if (string.Equals(res, "prefer_self", StringComparison.Ordinal))
                return "Resolution: Prefer \"" + selfId + "\"";
            if (string.Equals(res, "prefer_other", StringComparison.Ordinal))
                return "Resolution: Prefer \"" + otherId + "\"";
            return "Resolution: " + res;
        }

        /// <summary>Populates the per-pack settings section with runtime configuration controls.</summary>
        private void RefreshSettings(PackDisplayInfo p)
        {
            if (_settingsSection == null || _settingsContent == null) return;

            // Clear existing controls
            for (int i = _settingsContent.childCount - 1; i >= 0; i--)
            {
                Transform child = _settingsContent.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            // Get settings from the pack manifest
            if (p.Settings == null || p.Settings.Count == 0)
            {
                _settingsSection.SetActive(false);
                return;
            }

            _settingsSection.SetActive(true);

            // Add a header
            Text settingsHeader = UiBuilder.MakeText(_settingsContent, "SettingsHeader",
                "Pack Settings", 12, UiBuilder.Accent, bold: true);
            settingsHeader.gameObject.AddComponent<LayoutElement>().preferredHeight = 18f;

            UiBuilder.MakeHorizontalSeparator(_settingsContent, UiBuilder.Border);

            // Get the settings store
            var settingsStore = Settings.PackSettingsStore.Instance;

            // Render each setting
            foreach (SDK.Models.PackSetting setting in p.Settings)
            {
                BuildSettingControl(_settingsContent, p.Id, setting, settingsStore);
            }
        }

        /// <summary>Builds a single setting control row (label + input) and attaches save handlers.</summary>
        private void BuildSettingControl(RectTransform parent, string packId,
            SDK.Models.PackSetting setting, Settings.PackSettingsStore settingsStore)
        {
            // Container for this setting
            GameObject row = new GameObject($"Setting_{setting.Key}", typeof(RectTransform));
            row.transform.SetParent(parent, false);
            VerticalLayoutGroup rowVlg = row.AddComponent<VerticalLayoutGroup>();
            rowVlg.spacing = 4f;
            rowVlg.childForceExpandWidth = true;
            rowVlg.childForceExpandHeight = false;
            LayoutElement rowLe = row.AddComponent<LayoutElement>();
            rowLe.flexibleWidth = 1f;

            // Label
            Text label = UiBuilder.MakeText(row.transform, "Label", setting.DisplayName, 11,
                UiBuilder.TextPrimary);
            label.gameObject.AddComponent<LayoutElement>().preferredHeight = 16f;

            // Control based on type
            object currentValue = setting.DefaultValue ?? GetDefaultForType(setting.Type);
            currentValue = settingsStore.Get(packId, setting.Key, currentValue);

            if (setting.Type == "bool")
            {
                BuildBoolControl(row.transform, packId, setting, currentValue, settingsStore);
            }
            else if (setting.Type == "int" || setting.Type == "float")
            {
                BuildSliderControl(row.transform, packId, setting, currentValue, settingsStore);
            }
            else if (setting.Type == "enum")
            {
                BuildEnumControl(row.transform, packId, setting, currentValue, settingsStore);
            }
            else if (setting.Type == "string")
            {
                BuildStringControl(row.transform, packId, setting, currentValue, settingsStore);
            }

            // Optional description
            if (!string.IsNullOrEmpty(setting.Description))
            {
                Text desc = UiBuilder.MakeText(row.transform, "Description",
                    setting.Description, 9, UiBuilder.TextSecondary);
                desc.verticalOverflow = VerticalWrapMode.Overflow;
                desc.horizontalOverflow = HorizontalWrapMode.Wrap;
                LayoutElement descLe = desc.gameObject.AddComponent<LayoutElement>();
                descLe.flexibleWidth = 1f;
                descLe.preferredHeight = 20f;
            }
        }

        /// <summary>Builds a toggle control for bool settings.</summary>
        private void BuildBoolControl(Transform parent, string packId,
            SDK.Models.PackSetting setting, object currentValue, Settings.PackSettingsStore settingsStore)
        {
            GameObject toggleGo = new GameObject("BoolControl", typeof(RectTransform));
            toggleGo.transform.SetParent(parent, false);

            Toggle toggle = toggleGo.AddComponent<Toggle>();
            toggle.isOn = currentValue is bool b ? b : false;

            Image toggleBg = UiBuilder.MakePanel(toggleGo.transform, "Background",
                UiBuilder.BgSurface, new Vector2(30f, 20f)).GetComponent<Image>();

            Image checkmark = UiBuilder.MakePanel(toggleGo.transform, "Checkmark",
                UiBuilder.Accent, new Vector2(14f, 14f)).GetComponent<Image>();
            toggle.graphic = checkmark;

            string capturedKey = setting.Key;
            toggle.onValueChanged.AddListener(newValue =>
            {
                settingsStore.Set(packId, capturedKey, newValue);
            });

            LayoutElement toggleLe = toggleGo.AddComponent<LayoutElement>();
            toggleLe.preferredWidth = 30f;
            toggleLe.preferredHeight = 20f;
        }

        /// <summary>Builds a slider control for int/float settings.</summary>
        private void BuildSliderControl(Transform parent, string packId,
            SDK.Models.PackSetting setting, object currentValue, Settings.PackSettingsStore settingsStore)
        {
            GameObject container = new GameObject("SliderControl", typeof(RectTransform));
            container.transform.SetParent(parent, false);
            HorizontalLayoutGroup hlg = container.AddComponent<HorizontalLayoutGroup>();
            hlg.spacing = 6f;
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;

            Text valueLabel = UiBuilder.MakeText(container.transform, "ValueLabel",
                FormatSettingValue(currentValue), 10, UiBuilder.TextSecondary);
            LayoutElement valueLe = valueLabel.gameObject.AddComponent<LayoutElement>();
            valueLe.preferredWidth = 50f;

            GameObject sliderGo = new GameObject("Slider", typeof(RectTransform));
            sliderGo.transform.SetParent(container.transform, false);
            Slider slider = sliderGo.AddComponent<Slider>();
            slider.direction = Slider.Direction.LeftToRight;

            Image sliderBg = UiBuilder.MakePanel(sliderGo.transform, "Background",
                UiBuilder.BgSurface, new Vector2(150f, 10f)).GetComponent<Image>();
            slider.targetGraphic = sliderBg;

            GameObject handleGo = UiBuilder.MakePanel(sliderGo.transform, "Handle",
                UiBuilder.Accent, new Vector2(12f, 20f));
            slider.handleRect = handleGo.GetComponent<RectTransform>();

            double minVal = setting.Min ?? 0.0;
            double maxVal = setting.Max ?? 100.0;
            slider.minValue = (float)minVal;
            slider.maxValue = (float)maxVal;
            slider.value = ConvertToFloat(currentValue);

            string capturedKey = setting.Key;
            slider.onValueChanged.AddListener(newValue =>
            {
                object valueToStore = setting.Type == "int" ? (object)(int)newValue : (object)newValue;
                settingsStore.Set(packId, capturedKey, valueToStore);
                if (valueLabel != null)
                    valueLabel.text = FormatSettingValue(valueToStore);
            });

            LayoutElement sliderLe = sliderGo.AddComponent<LayoutElement>();
            sliderLe.flexibleWidth = 1f;
            sliderLe.preferredHeight = 20f;
        }

        /// <summary>Builds a dropdown control for enum settings.</summary>
        private void BuildEnumControl(Transform parent, string packId,
            SDK.Models.PackSetting setting, object currentValue, Settings.PackSettingsStore settingsStore)
        {
            GameObject dropdownGo = new GameObject("EnumControl", typeof(RectTransform));
            dropdownGo.transform.SetParent(parent, false);

            Dropdown dropdown = dropdownGo.AddComponent<Dropdown>();

            if (setting.EnumOptions != null)
            {
                foreach (string option in setting.EnumOptions)
                {
                    dropdown.options.Add(new Dropdown.OptionData(option));
                }
            }

            string currentStr = currentValue?.ToString() ?? "";
            for (int i = 0; i < dropdown.options.Count; i++)
            {
                if (dropdown.options[i].text == currentStr)
                {
                    dropdown.value = i;
                    break;
                }
            }

            string capturedKey = setting.Key;
            dropdown.onValueChanged.AddListener(index =>
            {
                if (index >= 0 && index < dropdown.options.Count)
                {
                    string selectedValue = dropdown.options[index].text;
                    settingsStore.Set(packId, capturedKey, selectedValue);
                }
            });

            LayoutElement dropdownLe = dropdownGo.AddComponent<LayoutElement>();
            dropdownLe.flexibleWidth = 1f;
            dropdownLe.preferredHeight = 28f;
        }

        /// <summary>Builds a text input control for string settings.</summary>
        private void BuildStringControl(Transform parent, string packId,
            SDK.Models.PackSetting setting, object currentValue, Settings.PackSettingsStore settingsStore)
        {
            GameObject inputGo = new GameObject("StringControl", typeof(RectTransform));
            inputGo.transform.SetParent(parent, false);

            InputField inputField = inputGo.AddComponent<InputField>();
            inputField.text = currentValue?.ToString() ?? "";
            inputField.lineType = InputField.LineType.SingleLine;

            Image inputBg = UiBuilder.MakePanel(inputGo.transform, "Background",
                UiBuilder.BgSurface, new Vector2(200f, 28f)).GetComponent<Image>();
            inputField.targetGraphic = inputBg;

            Text placeholder = UiBuilder.MakeText(inputGo.transform, "Placeholder",
                "Enter value...", 11, UiBuilder.TextSecondary);
            inputField.placeholder = placeholder;

            string capturedKey = setting.Key;
            inputField.onEndEdit.AddListener(newValue =>
            {
                settingsStore.Set(packId, capturedKey, newValue);
            });

            LayoutElement inputLe = inputGo.AddComponent<LayoutElement>();
            inputLe.flexibleWidth = 1f;
            inputLe.preferredHeight = 28f;
        }

        private object GetDefaultForType(string type) => type switch
        {
            "bool" => false,
            "int" => 0,
            "float" => 0.0f,
            "string" => "",
            "enum" => "",
            _ => ""
        };

        private float ConvertToFloat(object value) => value switch
        {
            float f => f,
            int i => i,
            double d => (float)d,
            long l => l,
            _ => 0f
        };

        private string FormatSettingValue(object value) => value switch
        {
            float f => f.ToString("F2"),
            double d => d.ToString("F2"),
            _ => value?.ToString() ?? ""
        };

        private void OnToggleSelected()
        {
            int index = _presenter.SelectedIndex;
            if (!_presenter.IsValidIndex(index)) return;

            PackDisplayInfo current = _presenter.Packs[index];

            // Only prompt when the user is enabling (not disabling) a pack.
            if (!current.IsEnabled)
            {
                List<string> missingDeps = CollectMissingDeps(current);
                if (missingDeps.Count > 0 && _canvasRoot != null)
                {
                    // Show the dependency dialog; defer the actual enable until the user confirms.
                    DepEnableDialog.Show(
                        _canvasRoot,
                        current.Name,
                        missingDeps,
                        onEnableAll: () =>
                        {
                            // Enable each missing dependency first, then the target pack.
                            foreach (string depId in missingDeps)
                            {
                                int depIdx = FindPackIndexById(depId);
                                if (depIdx < 0) continue;
                                if (!_presenter.TryToggleEnabled(depIdx, out PackDisplayInfo depUpdated)) continue;
                                OnPackToggled?.Invoke(depUpdated.Id, depUpdated.IsEnabled);
                            }

                            // Now enable the originally-selected pack.
                            if (_presenter.TryToggleEnabled(index, out PackDisplayInfo updated))
                            {
                                ClearCurrentSelection();
                                OnPackToggled?.Invoke(updated.Id, updated.IsEnabled);
                                QueueListRefresh();
                            }
                        },
                        onCancel: () =>
                        {
                            // Nothing to do — toggle was not applied.
                            _log?.LogInfo($"[ModMenuPanel] Dependency prompt cancelled for pack '{current.Id}'.");
                        });
                    return; // Dialog is showing; do not proceed with the toggle now.
                }
            }

            // No missing deps (or disabling) — apply the toggle immediately.
            if (!_presenter.TryToggleEnabled(index, out PackDisplayInfo immediateUpdated)) return;

            ClearCurrentSelection();
            OnPackToggled?.Invoke(immediateUpdated.Id, immediateUpdated.IsEnabled);
            QueueListRefresh();
        }

        /// <summary>
        /// Returns the IDs of dependencies that are declared by <paramref name="pack"/>
        /// but are currently disabled (or not present) in the presenter pack list.
        /// </summary>
        private List<string> CollectMissingDeps(PackDisplayInfo pack)
        {
            List<string> missing = new List<string>();
            foreach (string depId in pack.Dependencies)
            {
                bool depEnabled = false;
                foreach (PackDisplayInfo p in _presenter.Packs)
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

        /// <summary>Returns the presenter index of the pack with <paramref name="packId"/>, or -1.</summary>
        private int FindPackIndexById(string packId)
        {
            for (int i = 0; i < _presenter.Packs.Count; i++)
            {
                if (string.Equals(_presenter.Packs[i].Id, packId, StringComparison.Ordinal))
                    return i;
            }
            return -1;
        }

        private void QueueListRefresh()
        {
            if (_listRefreshQueued) return;
            _listRefreshQueued = true;
            StartCoroutine(RefreshListNextFrame());
        }

        private IEnumerator RefreshListNextFrame()
        {
            yield return null;
            _listRefreshQueued = false;
            RebuildPackList();
            RefreshDetail();
        }

        private static void ClearCurrentSelection()
        {
            try
            {
                EventSystem current = EventSystem.current;
                if (current != null)
                {
                    current.SetSelectedGameObject(null);
                }
            }
            catch { } // safe-swallow: UI selection cleanup is best-effort
        }

        // ── HMR notification API ──────────────────────────────────────────────────

        /// <summary>
        /// Displays a transient toast notification at the top of the panel.
        /// Falls back to updating the status line when the HUD strip is unavailable.
        /// Safe to call from any thread — delegates via the Unity main-thread coroutine.
        /// </summary>
        /// <param name="message">Text to show.</param>
        /// <param name="kind">Visual severity (Info / Warning / Error).</param>
        public void ShowToast(string message, HotReload.HmrToastKind kind)
        {
            // Map HmrToastKind to the existing DINOForge ToastType (defined in HudStrip.cs).
            ToastType toastType = kind switch
            {
                HotReload.HmrToastKind.Warning => ToastType.Warning,
                HotReload.HmrToastKind.Error => ToastType.Error,
                _ => ToastType.Info,
            };

            // Best-effort: set status so the header always reflects the last toast.
            SetStatus(message, kind == HotReload.HmrToastKind.Error ? 1 : 0);

            // Propagate to the DFCanvas HudStrip toast if we can reach it.
            try
            {
                DFCanvas? canvas = GetComponentInParent<DFCanvas>();
                canvas?.ShowToast(message, toastType);
            }
            catch { } // safe-swallow: toast fallback is best-effort UI only
        }

        /// <summary>
        /// Shows a blocking-style confirmation dialog overlay inside the mod menu panel.
        /// Presents <paramref name="message"/> with "Yes" and "No" buttons.
        /// Callbacks are invoked on the Unity main thread (inside the dialog button click handlers).
        /// </summary>
        /// <param name="message">Body text of the confirmation prompt.</param>
        /// <param name="onConfirm">Called when the user presses "Yes".</param>
        /// <param name="onCancel">Called when the user presses "No" or dismisses.</param>
        public void ShowConfirmDialog(string message, Action onConfirm, Action onCancel)
        {
            if (_panelRt == null)
            {
                // Panel not yet built — fall back to executing cancel immediately.
                onCancel?.Invoke();
                return;
            }

            // Build a dimmed overlay that blocks interaction with the pack list beneath it.
            GameObject overlay = new GameObject("HmrConfirmOverlay", typeof(RectTransform));
            overlay.transform.SetParent(_panelRt, false);
            RectTransform overlayRt = overlay.GetComponent<RectTransform>();
            overlayRt.anchorMin = Vector2.zero;
            overlayRt.anchorMax = Vector2.one;
            overlayRt.offsetMin = Vector2.zero;
            overlayRt.offsetMax = Vector2.zero;

            // Dimmed backdrop
            Image backdrop = overlay.AddComponent<Image>();
            backdrop.color = new Color(0f, 0f, 0f, 0.72f);
            backdrop.raycastTarget = true;

            // Dialog box
            GameObject dialog = UiBuilder.MakePanel(overlay.transform, "DialogBox",
                UiBuilder.BgSurface, new Vector2(380f, 200f));
            RectTransform dialogRt = dialog.GetComponent<RectTransform>();
            dialogRt.anchorMin = new Vector2(0.5f, 0.5f);
            dialogRt.anchorMax = new Vector2(0.5f, 0.5f);
            dialogRt.pivot = new Vector2(0.5f, 0.5f);
            dialogRt.anchoredPosition = Vector2.zero;

            VerticalLayoutGroup vlg = dialog.AddComponent<VerticalLayoutGroup>();
            vlg.spacing = 10f;
            vlg.padding = new RectOffset(16, 16, 14, 14);
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;

            // Title
            Text titleText = UiBuilder.MakeText(dialog.transform, "DialogTitle",
                "DINOForge — Mod Reload", 14, UiBuilder.Accent, bold: true);
            LayoutElement titleLe = titleText.gameObject.AddComponent<LayoutElement>();
            titleLe.preferredHeight = 20f;

            // Body
            Text bodyText = UiBuilder.MakeText(dialog.transform, "DialogBody", message,
                12, UiBuilder.TextPrimary);
            bodyText.horizontalOverflow = HorizontalWrapMode.Wrap;
            bodyText.verticalOverflow = VerticalWrapMode.Overflow;
            LayoutElement bodyLe = bodyText.gameObject.AddComponent<LayoutElement>();
            bodyLe.preferredHeight = 90f;
            bodyLe.flexibleHeight = 1f;

            // Button row
            GameObject btnRow = new GameObject("BtnRow", typeof(RectTransform));
            btnRow.transform.SetParent(dialog.transform, false);
            HorizontalLayoutGroup btnHlg = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnHlg.spacing = 10f;
            btnHlg.childForceExpandHeight = false;
            btnHlg.childForceExpandWidth = false;
            btnHlg.childAlignment = TextAnchor.MiddleRight;
            LayoutElement btnRowLe = btnRow.AddComponent<LayoutElement>();
            btnRowLe.preferredHeight = 34f;
            btnRowLe.flexibleWidth = 1f;

            // Spacer pushes buttons to the right
            GameObject spacer = new GameObject("Spacer", typeof(RectTransform));
            spacer.transform.SetParent(btnRow.transform, false);
            LayoutElement spacerLe = spacer.AddComponent<LayoutElement>();
            spacerLe.flexibleWidth = 1f;

            Button cancelBtn = UiBuilder.MakeButton(
                btnRow.transform, "CancelBtn", "No — keep playing",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () =>
                {
                    Destroy(overlay);
                    onCancel?.Invoke();
                });
            LayoutElement cancelLe = cancelBtn.gameObject.AddComponent<LayoutElement>();
            cancelLe.preferredWidth = 130f;
            cancelLe.preferredHeight = 30f;

            Button confirmBtn = UiBuilder.MakeButton(
                btnRow.transform, "ConfirmBtn", "Yes — reload",
                UiBuilder.Accent, Color.black,
                () =>
                {
                    Destroy(overlay);
                    onConfirm?.Invoke();
                });
            LayoutElement confirmLe = confirmBtn.gameObject.AddComponent<LayoutElement>();
            confirmLe.preferredWidth = 110f;
            confirmLe.preferredHeight = 30f;

            // Ensure panel is visible so user sees the dialog.
            if (!IsVisible) Show();
        }

        // ── Helpers ────────────────────────────────────────────────────────────────

        private string BuildStatusLine()
        {
            string errPart = _presenter.ErrorCount > 0 ? $"  {_presenter.ErrorCount} errors" : "";
            int totalContent = 0;
            int enabledCount = 0;
            foreach (PackDisplayInfo pack in _presenter.Packs)
            {
                if (!pack.IsEnabled) continue;
                enabledCount++;
                foreach (System.Collections.Generic.KeyValuePair<string, int> kv in pack.ContentSummary)
                    totalContent += kv.Value;
            }
            string contentPart = totalContent > 0 ? $"  ({totalContent} content files)" : "";
            return $"{enabledCount}/{_presenter.Packs.Count} packs active{errPart}{contentPart}";
        }

        // ── Update banner (#899) ─────────────────────────────────────────────────

        /// <summary>
        /// Displays an "Updates Available" banner showing each pending update with a
        /// "View on GitHub" button. Hides the banner when the list is empty or null.
        /// Must be called on the Unity main thread.
        /// </summary>
        public void SetUpdatesAvailable(IReadOnlyList<Updates.UpdateInfo> updates)
        {
            // Lazily build the banner the first time updates arrive.
            if (_updateBannerRoot == null && updates != null && updates.Count > 0)
            {
                BuildUpdateBanner();
            }

            if (_updateBannerRoot == null) return;

            bool hasUpdates = updates != null && updates.Count > 0;
            _updateBannerRoot.SetActive(hasUpdates);

            if (!hasUpdates || _updateBannerContent == null) return;

            // Clear existing rows.
            for (int i = _updateBannerContent.childCount - 1; i >= 0; i--)
            {
                Transform child = _updateBannerContent.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            foreach (Updates.UpdateInfo info in updates!)
            {
                string rowLabel = $"{info.DisplayName}: {info.CurrentVersion} → {info.NewVersion}";
                string capturedUrl = info.ReleaseUrl;

                GameObject row = new GameObject($"UpdateRow_{info.ComponentId}", typeof(RectTransform));
                row.transform.SetParent(_updateBannerContent, false);

                HorizontalLayoutGroup rowHlg = row.AddComponent<HorizontalLayoutGroup>();
                rowHlg.spacing = 6f;
                rowHlg.childForceExpandHeight = false;
                rowHlg.childForceExpandWidth = false;
                rowHlg.padding = new RectOffset(8, 8, 2, 2);

                LayoutElement rowLe = row.AddComponent<LayoutElement>();
                rowLe.preferredHeight = UpdateBannerRowHeight;
                rowLe.flexibleWidth = 1f;

                Text labelText = UiBuilder.MakeText(row.transform, "UpdateLabel", rowLabel, 11, UiBuilder.Accent);
                LayoutElement labelLe = labelText.gameObject.AddComponent<LayoutElement>();
                labelLe.flexibleWidth = 1f;
                labelLe.minWidth = 60f;

                Button viewBtn = UiBuilder.MakeButton(
                    row.transform, "ViewBtn", "View on GitHub",
                    UiBuilder.BgSurface, UiBuilder.TextPrimary,
                    () =>
                    {
                        if (IsSafeWebUrl(capturedUrl))
                        {
                            try { Application.OpenURL(capturedUrl); }
                            catch { } // safe-swallow: OpenURL is best-effort
                        }
                        else
                        {
                            _log?.LogWarning($"[ModMenuPanel] Refusing unsafe URL: {capturedUrl}");
                        }
                    });
                LayoutElement viewLe = viewBtn.gameObject.AddComponent<LayoutElement>();
                viewLe.preferredWidth = 110f;
                viewLe.minWidth = 90f;
                viewLe.preferredHeight = UpdateBannerRowHeight - 4f;
            }

            UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_updateBannerContent);
        }

        // ── Profiles section (#918) ────────────────────────────────────────────

        /// <summary>
        /// Builds the Profiles row: dropdown of saved profiles + Load / Save As / Export /
        /// Import / Delete buttons.  Inserted above the search / filter controls.
        /// </summary>
        private void BuildProfilesSection(Transform parent)
        {
            // Section label row
            GameObject labelRow = new GameObject("ProfileLabelRow", typeof(RectTransform));
            labelRow.transform.SetParent(parent, false);
            HorizontalLayoutGroup labelHlg = labelRow.AddComponent<HorizontalLayoutGroup>();
            labelHlg.spacing = 4f;
            labelHlg.childForceExpandWidth = true;
            labelHlg.childForceExpandHeight = false;
            LayoutElement labelRowLe = labelRow.AddComponent<LayoutElement>();
            labelRowLe.preferredHeight = 18f;
            labelRowLe.flexibleWidth = 1f;

            Text sectionLabel = UiBuilder.MakeText(labelRow.transform, "ProfilesLabel", "Profiles", 11,
                UiBuilder.Accent, bold: true);
            sectionLabel.gameObject.AddComponent<LayoutElement>().flexibleWidth = 1f;

            // Dropdown row
            GameObject ddRow = new GameObject("ProfileDropdownRow", typeof(RectTransform));
            ddRow.transform.SetParent(parent, false);
            HorizontalLayoutGroup ddHlg = ddRow.AddComponent<HorizontalLayoutGroup>();
            ddHlg.spacing = 4f;
            ddHlg.childForceExpandWidth = false;
            ddHlg.childForceExpandHeight = false;
            LayoutElement ddRowLe = ddRow.AddComponent<LayoutElement>();
            ddRowLe.preferredHeight = 26f;
            ddRowLe.flexibleWidth = 1f;

            _profileDropdown = MakeDropdown(ddRow.transform, "ProfileDropdown",
                new[] { "(no profiles)" }, _ => { });
            LayoutElement profileDdLe = _profileDropdown.gameObject.AddComponent<LayoutElement>();
            profileDdLe.flexibleWidth = 1f;
            profileDdLe.preferredHeight = 26f;

            // Action buttons row
            GameObject btnRow = new GameObject("ProfileButtonsRow", typeof(RectTransform));
            btnRow.transform.SetParent(parent, false);
            HorizontalLayoutGroup btnHlg = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnHlg.spacing = 3f;
            btnHlg.childForceExpandWidth = false;
            btnHlg.childForceExpandHeight = false;
            LayoutElement btnRowLe = btnRow.AddComponent<LayoutElement>();
            btnRowLe.preferredHeight = 24f;
            btnRowLe.flexibleWidth = 1f;

            // Load
            Button loadBtn = UiBuilder.MakeButton(btnRow.transform, "ProfileLoadBtn", "Load",
                UiBuilder.BgSurface, UiBuilder.TextPrimary, OnProfileLoad);
            loadBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 40f;
            loadBtn.gameObject.GetComponent<LayoutElement>().preferredHeight = 22f;

            // Save As
            Button saveBtn = UiBuilder.MakeButton(btnRow.transform, "ProfileSaveBtn", "Save…",
                UiBuilder.BgSurface, UiBuilder.Accent, OnProfileSaveAs);
            saveBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 46f;
            saveBtn.gameObject.GetComponent<LayoutElement>().preferredHeight = 22f;

            // Export to clipboard
            Button exportBtn = UiBuilder.MakeButton(btnRow.transform, "ProfileExportBtn", "Export",
                UiBuilder.BgSurface, UiBuilder.TextSecondary, OnProfileExport);
            exportBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 48f;
            exportBtn.gameObject.GetComponent<LayoutElement>().preferredHeight = 22f;

            // Import from clipboard
            Button importBtn = UiBuilder.MakeButton(btnRow.transform, "ProfileImportBtn", "Import",
                UiBuilder.BgSurface, UiBuilder.TextSecondary, OnProfileImport);
            importBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 48f;
            importBtn.gameObject.GetComponent<LayoutElement>().preferredHeight = 22f;

            // Delete
            Button deleteBtn = UiBuilder.MakeButton(btnRow.transform, "ProfileDeleteBtn", "Del",
                UiBuilder.BgDeep, UiBuilder.Error, OnProfileDelete);
            deleteBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 30f;
            deleteBtn.gameObject.GetComponent<LayoutElement>().preferredHeight = 22f;

            // Spacer
            GameObject spacer = new GameObject("ProfileBtnSpacer", typeof(RectTransform));
            spacer.transform.SetParent(btnRow.transform, false);
            spacer.AddComponent<LayoutElement>().flexibleWidth = 1f;

            // Separator below profiles section
            UiBuilder.MakeHorizontalSeparator(parent, UiBuilder.Border);

            RefreshProfileDropdown();
        }

        /// <summary>Refreshes the profile dropdown options from the ProfileManager.</summary>
        private void RefreshProfileDropdown()
        {
            if (_profileDropdown == null) return;

            _profileDropdown.options.Clear();

            IReadOnlyList<string> names = _profileManager?.ListProfiles() ?? new List<string>();
            if (names.Count == 0)
            {
                _profileDropdown.options.Add(new Dropdown.OptionData("(no profiles)"));
            }
            else
            {
                foreach (string name in names)
                    _profileDropdown.options.Add(new Dropdown.OptionData(name));
            }

            _profileDropdown.value = 0;
            _profileDropdown.RefreshShownValue();
        }

        /// <summary>Returns the currently selected profile name, or null if none.</summary>
        private string? SelectedProfileName()
        {
            if (_profileDropdown == null) return null;
            if (_profileDropdown.options.Count == 0) return null;
            string name = _profileDropdown.options[_profileDropdown.value].text;
            if (name == "(no profiles)") return null;
            return name;
        }

        private void OnProfileLoad()
        {
            if (_profileManager == null)
            {
                _log?.LogWarning("[ModMenuPanel] OnProfileLoad: no ProfileManager.");
                return;
            }

            string? name = SelectedProfileName();
            if (name == null)
            {
                _log?.LogInfo("[ModMenuPanel] OnProfileLoad: no profile selected.");
                return;
            }

            ModProfile? profile = _profileManager.Load(name);
            if (profile == null)
            {
                _log?.LogWarning($"[ModMenuPanel] OnProfileLoad: failed to load profile '{name}'.");
                return;
            }

            _log?.LogInfo($"[ModMenuPanel] OnProfileLoad: applying profile '{name}' ({profile.EnabledPacks.Count} packs).");
            OnProfileLoaded?.Invoke(profile.EnabledPacks);
        }

        private void OnProfileSaveAs()
        {
            if (_profileManager == null)
            {
                _log?.LogWarning("[ModMenuPanel] OnProfileSaveAs: no ProfileManager.");
                return;
            }

            if (_panelRt == null) return;

            // Build inline save-as modal
            ShowProfileSaveModal();
        }

        private void ShowProfileSaveModal()
        {
            if (_panelRt == null) return;

            // Destroy any existing modal
            if (_profileSaveModal != null)
            {
                Destroy(_profileSaveModal);
                _profileSaveModal = null;
            }

            GameObject overlay = new GameObject("ProfileSaveModal", typeof(RectTransform));
            overlay.transform.SetParent(_panelRt, false);
            RectTransform overlayRt = overlay.GetComponent<RectTransform>();
            overlayRt.anchorMin = Vector2.zero;
            overlayRt.anchorMax = Vector2.one;
            overlayRt.offsetMin = Vector2.zero;
            overlayRt.offsetMax = Vector2.zero;

            Image backdrop = overlay.AddComponent<Image>();
            backdrop.color = new Color(0f, 0f, 0f, 0.72f);
            backdrop.raycastTarget = true;

            GameObject dialog = UiBuilder.MakePanel(overlay.transform, "SaveAsDialog",
                UiBuilder.BgSurface, new Vector2(320f, 160f));
            RectTransform dialogRt = dialog.GetComponent<RectTransform>();
            dialogRt.anchorMin = new Vector2(0.5f, 0.5f);
            dialogRt.anchorMax = new Vector2(0.5f, 0.5f);
            dialogRt.pivot = new Vector2(0.5f, 0.5f);
            dialogRt.anchoredPosition = Vector2.zero;

            VerticalLayoutGroup vlg = dialog.AddComponent<VerticalLayoutGroup>();
            vlg.spacing = 8f;
            vlg.padding = new RectOffset(14, 14, 12, 12);
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;

            Text titleTxt = UiBuilder.MakeText(dialog.transform, "ModalTitle",
                "Save Profile As…", 13, UiBuilder.Accent, bold: true);
            titleTxt.gameObject.AddComponent<LayoutElement>().preferredHeight = 18f;

            _profileNameInput = UiBuilder.MakeInputField(dialog.transform, "ProfileNameInput",
                "Profile name…", _ => { });
            LayoutElement inputLe = _profileNameInput.gameObject.AddComponent<LayoutElement>();
            inputLe.preferredHeight = 28f;
            inputLe.flexibleWidth = 1f;

            // Pre-fill with currently selected profile name if any
            string? currentName = SelectedProfileName();
            if (!string.IsNullOrEmpty(currentName))
                _profileNameInput.text = currentName;

            GameObject btnRow = new GameObject("BtnRow", typeof(RectTransform));
            btnRow.transform.SetParent(dialog.transform, false);
            HorizontalLayoutGroup btnHlg = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnHlg.spacing = 8f;
            btnHlg.childForceExpandWidth = false;
            btnHlg.childForceExpandHeight = false;
            btnHlg.childAlignment = TextAnchor.MiddleRight;
            btnRow.AddComponent<LayoutElement>().preferredHeight = 30f;

            GameObject spacer = new GameObject("Spacer", typeof(RectTransform));
            spacer.transform.SetParent(btnRow.transform, false);
            spacer.AddComponent<LayoutElement>().flexibleWidth = 1f;

            InputField capturedInput = _profileNameInput;
            GameObject capturedOverlay = overlay;

            Button cancelBtn = UiBuilder.MakeButton(btnRow.transform, "CancelBtn", "Cancel",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () => { Destroy(capturedOverlay); });
            cancelBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 64f;

            Button saveBtn = UiBuilder.MakeButton(btnRow.transform, "SaveBtn", "Save",
                UiBuilder.Accent, Color.black,
                () =>
                {
                    string profileName = capturedInput != null ? capturedInput.text.Trim() : string.Empty;
                    if (string.IsNullOrWhiteSpace(profileName))
                    {
                        _log?.LogWarning("[ModMenuPanel] Save profile: name is empty.");
                        return;
                    }

                    // Collect currently enabled packs
                    List<string> enabledIds = new List<string>();
                    foreach (PackDisplayInfo pack in _presenter.Packs)
                    {
                        if (pack.IsEnabled)
                            enabledIds.Add(pack.Id);
                    }

                    _profileManager?.SaveCurrent(profileName, enabledIds);
                    Destroy(capturedOverlay);
                    RefreshProfileDropdown();

                    // Select the newly saved profile in the dropdown
                    if (_profileDropdown != null)
                    {
                        for (int i = 0; i < _profileDropdown.options.Count; i++)
                        {
                            if (_profileDropdown.options[i].text == profileName)
                            {
                                _profileDropdown.value = i;
                                break;
                            }
                        }
                    }

                    _log?.LogInfo($"[ModMenuPanel] Profile '{profileName}' saved.");
                });
            saveBtn.gameObject.AddComponent<LayoutElement>().preferredWidth = 56f;

            _profileSaveModal = overlay;
        }

        private void OnProfileExport()
        {
            if (_profileManager == null) return;

            string? name = SelectedProfileName();
            if (name == null)
            {
                _log?.LogInfo("[ModMenuPanel] OnProfileExport: no profile selected.");
                return;
            }

            string json = _profileManager.ExportJson(name);
            if (string.IsNullOrEmpty(json))
            {
                _log?.LogWarning($"[ModMenuPanel] OnProfileExport: ExportJson returned empty for '{name}'.");
                return;
            }

            try
            {
                GUIUtility.systemCopyBuffer = json;
                _log?.LogInfo($"[ModMenuPanel] Profile '{name}' exported to clipboard.");
                SetStatus($"Profile '{name}' copied to clipboard");
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[ModMenuPanel] OnProfileExport clipboard write failed: {ex.Message}");
            }
        }

        private void OnProfileImport()
        {
            if (_profileManager == null)
            {
                _log?.LogWarning("[ModMenuPanel] OnProfileImport: no ProfileManager.");
                return;
            }

            string json;
            try
            {
                json = GUIUtility.systemCopyBuffer ?? string.Empty;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[ModMenuPanel] OnProfileImport clipboard read failed: {ex.Message}");
                return;
            }

            if (string.IsNullOrWhiteSpace(json))
            {
                _log?.LogInfo("[ModMenuPanel] OnProfileImport: clipboard is empty.");
                SetStatus("Clipboard is empty — nothing to import");
                return;
            }

            try
            {
                _profileManager.ImportJson(json);
                RefreshProfileDropdown();
                SetStatus("Profile imported from clipboard");
                _log?.LogInfo("[ModMenuPanel] Profile imported from clipboard.");
            }
            catch (InvalidOperationException ex)
            {
                _log?.LogWarning($"[ModMenuPanel] OnProfileImport validation failed: {ex.Message}");
                SetStatus($"Import failed: {ex.Message}", 1);
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[ModMenuPanel] OnProfileImport unexpected error: {ex.Message}");
                SetStatus("Import failed — see log for details", 1);
            }
        }

        private void OnProfileDelete()
        {
            if (_profileManager == null) return;

            string? name = SelectedProfileName();
            if (name == null)
            {
                _log?.LogInfo("[ModMenuPanel] OnProfileDelete: no profile selected.");
                return;
            }

            // Show confirm dialog before deleting
            ShowConfirmDialog(
                $"Delete profile \"{name}\"?\nThis cannot be undone.",
                onConfirm: () =>
                {
                    bool deleted = _profileManager.Delete(name);
                    if (deleted)
                    {
                        RefreshProfileDropdown();
                        SetStatus($"Profile '{name}' deleted");
                        _log?.LogInfo($"[ModMenuPanel] Profile '{name}' deleted.");
                    }
                    else
                    {
                        _log?.LogWarning($"[ModMenuPanel] OnProfileDelete: profile '{name}' not found.");
                    }
                },
                onCancel: () => { /* cancelled, nothing to do */ });
        }

        // ── Telemetry section helpers (#921) ──────────────────────────────────

        /// <summary>
        /// Builds the telemetry section appended at the bottom of the detail pane's scroll content.
        /// Displays <see cref="MetricsCollector.Instance.DumpMarkdown()"/> as monospace plain text;
        /// auto-refreshes every 2 seconds via coroutine; "Copy to Clipboard" button for sharing.
        /// </summary>
        private void BuildTelemetrySection(Transform parent)
        {
            UiBuilder.MakeHorizontalSeparator(parent, UiBuilder.Border);

            // Section header row
            GameObject headerRow = new GameObject("TelemetryHeader", typeof(RectTransform));
            headerRow.transform.SetParent(parent, false);
            HorizontalLayoutGroup hHlg = headerRow.AddComponent<HorizontalLayoutGroup>();
            hHlg.spacing = 6f;
            hHlg.childForceExpandHeight = false;
            hHlg.childForceExpandWidth = false;
            hHlg.padding = new RectOffset(0, 0, 4, 2);
            headerRow.AddComponent<LayoutElement>().preferredHeight = 24f;

            Text sectionTitle = UiBuilder.MakeText(headerRow.transform, "TelemetryTitle",
                "TELEMETRY", 11, UiBuilder.Accent, bold: true);
            sectionTitle.gameObject.AddComponent<LayoutElement>().flexibleWidth = 1f;

            // "Copy to Clipboard" button
            Button copyBtn = UiBuilder.MakeButton(
                headerRow.transform, "CopyMetricsBtn", "Copy",
                UiBuilder.BgSurface, UiBuilder.TextSecondary,
                OnCopyMetricsToClipboard);
            LayoutElement copyLe = copyBtn.gameObject.AddComponent<LayoutElement>();
            copyLe.preferredWidth = 48f;
            copyLe.preferredHeight = 20f;

            // "Refresh" button (manual trigger)
            Button refreshBtn = UiBuilder.MakeButton(
                headerRow.transform, "RefreshMetricsBtn", "↺",
                UiBuilder.BgSurface, UiBuilder.TextSecondary,
                RefreshTelemetryText);
            LayoutElement refreshLe = refreshBtn.gameObject.AddComponent<LayoutElement>();
            refreshLe.preferredWidth = 24f;
            refreshLe.preferredHeight = 20f;

            // Text area for metrics dump
            _telemetryText = UiBuilder.MakeText(parent, "TelemetryText",
                "(no metrics yet)", 10, new Color(0.65f, 0.75f, 0.65f, 1f));
            LayoutElement textLe = _telemetryText.gameObject.AddComponent<LayoutElement>();
            textLe.preferredHeight = 120f;
            textLe.flexibleWidth = 1f;
            _telemetryText.verticalOverflow = VerticalWrapMode.Overflow;
            _telemetryText.alignment = TextAnchor.UpperLeft;

            // Populate immediately
            RefreshTelemetryText();

            // Start the 2-second auto-refresh coroutine
            if (_telemetryRefreshCoroutine != null)
                StopCoroutine(_telemetryRefreshCoroutine);
            _telemetryRefreshCoroutine = StartCoroutine(TelemetryAutoRefreshCoroutine());
        }

        /// <summary>Updates the telemetry text with the latest <see cref="MetricsCollector.Instance.DumpMarkdown"/> output.</summary>
        private void RefreshTelemetryText()
        {
            if (_telemetryText == null) return;
            try
            {
                string md = MetricsCollector.Instance.DumpMarkdown();
                _telemetryText.text = string.IsNullOrWhiteSpace(md) ? "(no metrics recorded yet)" : md;
            }
            catch (Exception ex)
            {
                // best-effort: telemetry display must never crash the UI
                if (_telemetryText != null)
                    _telemetryText.text = $"(metrics error: {ex.Message})";
            }
        }

        /// <summary>Copies the current metrics dump to the system clipboard.</summary>
        private void OnCopyMetricsToClipboard()
        {
            try
            {
                string md = MetricsCollector.Instance.DumpMarkdown();
                GUIUtility.systemCopyBuffer = string.IsNullOrWhiteSpace(md) ? "(no metrics)" : md;
            }
            catch (Exception ex)
            {
                // best-effort
                System.Diagnostics.Debug.WriteLine($"ModMenuPanel metrics clipboard copy failed: {ex.Message}");
            }
        }

        /// <summary>Coroutine: refreshes the telemetry text every <see cref="TelemetryRefreshIntervalSec"/> seconds.</summary>
        private IEnumerator TelemetryAutoRefreshCoroutine()
        {
            while (true)
            {
                yield return new WaitForSeconds(TelemetryRefreshIntervalSec);
                RefreshTelemetryText();
            }
        }

        // ─────────────────────────────────────────────────────────────────────

        /// <summary>Lazily builds the amber update banner anchored just below the header bar.</summary>
        private void BuildUpdateBanner()
        {
            if (_panelRt == null) return;

            GameObject bannerRoot = UiBuilder.MakePanel(_panelRt, "UpdateBanner",
                UiBuilder.HexColor("#3a2a00", 1f), Vector2.zero);
            RectTransform bannerRt = bannerRoot.GetComponent<RectTransform>();
            bannerRt.anchorMin = new Vector2(0f, 1f);
            bannerRt.anchorMax = new Vector2(1f, 1f);
            bannerRt.pivot = new Vector2(0.5f, 1f);
            bannerRt.anchoredPosition = new Vector2(0f, -(HeaderHeight + 1f));
            bannerRt.sizeDelta = Vector2.zero;

            ContentSizeFitter bannerCsf = bannerRoot.AddComponent<ContentSizeFitter>();
            bannerCsf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            VerticalLayoutGroup bannerVlg = bannerRoot.AddComponent<VerticalLayoutGroup>();
            bannerVlg.childForceExpandWidth = true;
            bannerVlg.childForceExpandHeight = false;
            bannerVlg.spacing = 0f;
            bannerVlg.padding = new RectOffset(0, 0, 4, 4);

            GameObject titleRow = new GameObject("BannerTitle", typeof(RectTransform));
            titleRow.transform.SetParent(bannerRoot.transform, false);
            titleRow.AddComponent<LayoutElement>().preferredHeight = 18f;
            UiBuilder.AddHorizontalLayout(titleRow, 4f, new RectOffset(8, 8, 2, 2));
            Text titleTxt = UiBuilder.MakeText(titleRow.transform, "BannerTitleText",
                "Updates Available", 11, UiBuilder.Accent, bold: true);
            titleTxt.gameObject.AddComponent<LayoutElement>().flexibleWidth = 1f;

            GameObject content = new GameObject("UpdateBannerContent", typeof(RectTransform));
            content.transform.SetParent(bannerRoot.transform, false);
            _updateBannerContent = content.GetComponent<RectTransform>();
            VerticalLayoutGroup contentVlg = content.AddComponent<VerticalLayoutGroup>();
            contentVlg.childForceExpandWidth = true;
            contentVlg.childForceExpandHeight = false;
            contentVlg.spacing = 2f;
            ContentSizeFitter contentCsf = content.AddComponent<ContentSizeFitter>();
            contentCsf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;
            content.AddComponent<LayoutElement>().flexibleWidth = 1f;

            _updateBannerRoot = bannerRoot;
            bannerRoot.SetActive(false);
        }

    }
}
