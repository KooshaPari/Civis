#nullable enable
using System;
using System.Collections.Generic;
using BepInEx.Logging;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// UGUI mod menu panel. Replaces the legacy IMGUI ModMenuOverlay.
    /// Layout: header bar | split (pack list / detail pane) | footer.
    /// Exposes the same public API as <see cref="ModMenuOverlay"/> so ModPlatform
    /// does not need changes.
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

        // ── Animation ────────────────────────────────────────────────────────────
        private CanvasGroup? _canvasGroup;
        private RectTransform? _panelRt;
        private float _animT;          // 0 = fully hidden, 1 = fully visible
        private bool _targetVisible;

        // ── UI references ────────────────────────────────────────────────────────
        private Text? _headerStatusText;
        private RectTransform? _listContent;
        private GameObject? _detailPane;
        private Text? _detailName;
        private Text? _detailMeta;
        private Text? _detailDesc;
        private Text? _detailDeps;
        private Text? _detailConflicts;
        private Text? _detailLoadOrder;

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
        /// Builds the full UGUI hierarchy. Call from DFCanvas.Start() on the main thread.
        /// </summary>
        /// <param name="canvasRoot">Root canvas transform to attach to.</param>
        public void Build(Transform canvasRoot)
        {
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

            _log?.LogInfo($"[ModMenuPanel.SetPacks] Calling RebuildPackList()...");
            RebuildPackList();
            _log?.LogInfo($"[ModMenuPanel.SetPacks] RebuildPackList() complete. Calling RefreshDetail()...");
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
            _animT = 1f;

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
            _animT = 0f;

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

        // ── MonoBehaviour ─────────────────────────────────────────────────────────

        private void Update()
        {
            AnimatePanel();
        }

        // ── Animation ─────────────────────────────────────────────────────────────

        private void AnimatePanel()
        {
            // No-op: Update() never fires in DINO (MonoBehaviour.Update is not called).
            // Show()/Hide() set state immediately instead.
        }

        // ── UI construction ────────────────────────────────────────────────────────

        private void BuildHeader(Transform parent)
        {
            GameObject header = UiBuilder.MakePanel(parent, "Header",
                UiBuilder.BgSurface, new Vector2(0f, HeaderHeight));
            RectTransform hRt = header.GetComponent<RectTransform>();
            hRt.anchorMin = new Vector2(0f, 1f);
            hRt.anchorMax = Vector2.one;
            hRt.pivot = new Vector2(0.5f, 1f);
            hRt.offsetMin = Vector2.zero;
            hRt.offsetMax = Vector2.zero;
            hRt.sizeDelta = new Vector2(0f, HeaderHeight);

            UiBuilder.AddHorizontalLayout(header, 8f, new RectOffset(12, 8, 6, 6));

            // Title
            Text title = UiBuilder.MakeText(header.transform, "Title", "DINOForge", 16,
                UiBuilder.Accent, bold: true);
            LayoutElement titleLe = title.gameObject.AddComponent<LayoutElement>();
            titleLe.preferredWidth = 120f;
            titleLe.minWidth = 80f;

            // Status text (flexible)
            _headerStatusText = UiBuilder.MakeText(header.transform, "Status",
                BuildStatusLine(), 12, UiBuilder.TextSecondary);
            LayoutElement statusLe = _headerStatusText.gameObject.AddComponent<LayoutElement>();
            statusLe.preferredWidth = 300f;
            statusLe.flexibleWidth = 1f;

            // Close button
            Button closeBtn = UiBuilder.MakeButton(
                header.transform, "CloseBtn", "×",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () => Hide());
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

            RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
            scrollRt.anchorMin = Vector2.zero;
            scrollRt.anchorMax = Vector2.one;
            scrollRt.offsetMin = new Vector2(0f, 0f);
            scrollRt.offsetMax = new Vector2(0f, -32f);
            scrollRt.sizeDelta = Vector2.zero;

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
            _detailPane = new GameObject("DetailPane", typeof(RectTransform));
            _detailPane.transform.SetParent(parent, false);

            RectTransform detailRt = _detailPane.GetComponent<RectTransform>();
            LayoutElement detailLe = _detailPane.AddComponent<LayoutElement>();
            detailLe.flexibleWidth = 1f;

            VerticalLayoutGroup vlg = _detailPane.AddComponent<VerticalLayoutGroup>();
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;
            vlg.spacing = 6f;
            vlg.padding = new RectOffset(14, 14, 12, 12);

            // Name
            _detailName = UiBuilder.MakeText(_detailPane.transform, "DetailName",
                "Select a pack", 15, UiBuilder.TextPrimary, bold: true);
            LayoutElement nameLe = _detailName.gameObject.AddComponent<LayoutElement>();
            nameLe.preferredHeight = 22f;
            nameLe.flexibleWidth = 1f;

            // Meta (author · type)
            _detailMeta = UiBuilder.MakeText(_detailPane.transform, "DetailMeta",
                "", 12, UiBuilder.TextSecondary);
            LayoutElement metaLe = _detailMeta.gameObject.AddComponent<LayoutElement>();
            metaLe.preferredHeight = 18f;
            metaLe.flexibleWidth = 1f;

            UiBuilder.MakeHorizontalSeparator(_detailPane.transform, UiBuilder.Border);

            // Description (scrollable text area)
            _detailDesc = UiBuilder.MakeText(_detailPane.transform, "DetailDesc",
                "", 12, UiBuilder.TextPrimary);
            LayoutElement descLe = _detailDesc.gameObject.AddComponent<LayoutElement>();
            descLe.preferredHeight = 80f;
            descLe.flexibleWidth = 1f;
            descLe.flexibleHeight = 1f;
            _detailDesc.verticalOverflow = VerticalWrapMode.Truncate;

            UiBuilder.MakeHorizontalSeparator(_detailPane.transform, UiBuilder.Border);

            // Dependencies
            _detailDeps = UiBuilder.MakeText(_detailPane.transform, "DetailDeps",
                "Dependencies: none", 12, UiBuilder.TextSecondary);
            LayoutElement depsLe = _detailDeps.gameObject.AddComponent<LayoutElement>();
            depsLe.preferredHeight = 18f;
            depsLe.flexibleWidth = 1f;

            // Conflicts
            _detailConflicts = UiBuilder.MakeText(_detailPane.transform, "DetailConflicts",
                "Conflicts: none", 12, UiBuilder.TextSecondary);
            LayoutElement conflictsLe = _detailConflicts.gameObject.AddComponent<LayoutElement>();
            conflictsLe.preferredHeight = 18f;
            conflictsLe.flexibleWidth = 1f;

            // Load order
            _detailLoadOrder = UiBuilder.MakeText(_detailPane.transform, "DetailLoadOrder",
                "Load Order: —", 12, UiBuilder.TextSecondary);
            LayoutElement loLe = _detailLoadOrder.gameObject.AddComponent<LayoutElement>();
            loLe.preferredHeight = 18f;
            loLe.flexibleWidth = 1f;

            UiBuilder.MakeHorizontalSeparator(_detailPane.transform, UiBuilder.Border);

            // Action buttons row
            GameObject btnRow = new GameObject("ActionButtons", typeof(RectTransform));
            btnRow.transform.SetParent(_detailPane.transform, false);
            HorizontalLayoutGroup btnHlg = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnHlg.spacing = 8f;
            btnHlg.childForceExpandHeight = false;
            btnHlg.childForceExpandWidth = false;
            LayoutElement btnRowLe = btnRow.AddComponent<LayoutElement>();
            btnRowLe.preferredHeight = 32f;

            Button toggleBtn = UiBuilder.MakeButton(
                btnRow.transform, "ToggleBtn", "Disable",
                UiBuilder.BgSurface, UiBuilder.TextPrimary,
                OnToggleSelected);
            LayoutElement toggleBtnLe = toggleBtn.gameObject.AddComponent<LayoutElement>();
            toggleBtnLe.preferredWidth = 90f;
            toggleBtnLe.minWidth = 90f;
            toggleBtnLe.preferredHeight = 30f;
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
                footer.transform, "ReloadBtn", "↺  Reload Packs",
                UiBuilder.BgDeep, UiBuilder.Accent,
                () => OnReloadRequested?.Invoke());
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

            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] START: presenter.Packs.Count={_presenter.Packs.Count}, _listContent={_listContent.name}, active={_listContent.gameObject.activeSelf}");
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] _listContent RectTransform: position={_listContent.anchoredPosition}, sizeDelta={_listContent.sizeDelta}");
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] Clearing {_listContent.childCount} existing items");

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
                        expectedHeight += childLe.preferredHeight;
                        if (i > 0) expectedHeight += vlg.spacing;
                    }
                }
            }
            _log?.LogInfo($"[ModMenuPanel.RebuildPackList] MANUAL CALCULATION: expected total height={expectedHeight} (padding.top={vlg?.padding.top}, padding.bottom={vlg?.padding.bottom}, spacing={vlg?.spacing}, items={_listContent.childCount})");

            // Drive layout immediately so the ScrollRect sees the correct content bounds
            // even when the panel is currently hidden.
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

            // Card
            Color bgColor = isSelected ? UiBuilder.BgSurface : UiBuilder.BgDeep;
            Color alpha = pack.IsEnabled ? Color.white : new Color(1f, 1f, 1f, 0.6f);

            GameObject card = UiBuilder.MakePanel(_listContent, $"PackItem_{pack.Id}", bgColor, new Vector2(0f, ItemHeight));
            RectTransform cardRt = card.GetComponent<RectTransform>();
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} card created: sizeDelta={cardRt.sizeDelta}, active={card.activeSelf}");

            LayoutElement cardLe = card.AddComponent<LayoutElement>();
            cardLe.minHeight = ItemHeight;
            cardLe.preferredHeight = ItemHeight;
            cardLe.flexibleWidth = 1f;
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} LayoutElement set: minHeight={cardLe.minHeight}, preferredHeight={cardLe.preferredHeight}");

            // Amber left-border strip for enabled packs
            if (pack.IsEnabled)
            {
                GameObject border = UiBuilder.MakePanel(card.transform, "EnabledBorder",
                    UiBuilder.Accent, new Vector2(4f, 0f));
                RectTransform bRt = border.GetComponent<RectTransform>();
                bRt.anchorMin = new Vector2(0f, 0f);
                bRt.anchorMax = new Vector2(0f, 1f);
                bRt.pivot = new Vector2(0f, 0.5f);
                bRt.offsetMin = Vector2.zero;
                bRt.offsetMax = new Vector2(4f, 0f);
                bRt.anchoredPosition = Vector2.zero;
            }

            // Content layout
            HorizontalLayoutGroup hlg = card.AddComponent<HorizontalLayoutGroup>();
            hlg.spacing = 6f;
            hlg.padding = new RectOffset(pack.IsEnabled ? 10 : 6, 6, 4, 4);
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;

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
            _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} nameText LayoutElement: minWidth={nameLe.minWidth}, flexibleWidth={nameLe.flexibleWidth}");

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

            // Click to select
            int capturedIndex = index;
            Button btn = card.AddComponent<Button>();
            ColorBlock cb = btn.colors;
            cb.normalColor = bgColor;
            cb.highlightedColor = Color.Lerp(bgColor, Color.white, 0.08f);
            cb.pressedColor = Color.Lerp(bgColor, Color.black, 0.1f);
            cb.selectedColor = bgColor;
            cb.colorMultiplier = 1f;
            btn.colors = cb;
            btn.targetGraphic = card.GetComponent<Image>();
            btn.onClick.AddListener(() => SelectPack(capturedIndex));
        }

        private void SelectPack(int index)
        {
            _presenter.SelectIndex(index);
            RebuildPackList();
            RefreshDetail();
        }

        private void RefreshDetail()
        {
            PackDisplayInfo? selected = _presenter.SelectedPack;
            if (selected == null)
            {
                if (_detailName != null) _detailName.text = "Select a pack";
                if (_detailMeta != null) _detailMeta.text = "";
                if (_detailDesc != null) _detailDesc.text = "";
                if (_detailDeps != null) _detailDeps.text = "Dependencies: none";
                if (_detailConflicts != null) _detailConflicts.text = "Conflicts: none";
                if (_detailLoadOrder != null) _detailLoadOrder.text = "Load Order: —";
                return;
            }

            PackDisplayInfo p = selected;

            if (_detailName != null) _detailName.text = p.Name;
            if (_detailMeta != null) _detailMeta.text = $"by {p.Author}  ·  {p.Type}  ·  v{p.Version}";
            if (_detailDesc != null)
            {
                string descText = string.IsNullOrEmpty(p.Description)
                    ? "(no description)"
                    : p.Description!;

                if (p.Errors.Count > 0)
                {
                    descText += "\n\n<color=#e05252>Errors:</color>\n"
                        + string.Join("\n", p.Errors);
                }

                _detailDesc.text = descText;
            }

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

            // Update toggle button label
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

        private void OnToggleSelected()
        {
            if (!_presenter.TryToggleEnabled(_presenter.SelectedIndex, out PackDisplayInfo updated)) return;

            OnPackToggled?.Invoke(updated.Id, updated.IsEnabled);
            RebuildPackList();
            RefreshDetail();
        }

        // ── Helpers ────────────────────────────────────────────────────────────────

        private string BuildStatusLine()
        {
            string errPart = _presenter.ErrorCount > 0 ? $"  {_presenter.ErrorCount} errors" : "  0 errors";
            return $"{_presenter.Packs.Count} packs{errPart}  {_presenter.StatusMessage}";
        }
    }
}
