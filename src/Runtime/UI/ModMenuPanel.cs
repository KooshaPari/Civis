#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using BepInEx.Logging;
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

        // ── Animation ────────────────────────────────────────────────────────────
        private CanvasGroup? _canvasGroup;
        private RectTransform? _panelRt;
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
        private Text? _detailContent;
        private Text? _detailDetectedConflicts;
        private bool _listRefreshQueued;

        // ── Rich detail pane refs (#897) ──────────────────────────────────────
        private Text? _detailLicense;
        private Text? _detailRichContent;
        private RectTransform? _detailTagsRow;
        private RectTransform? _detailLinksRow;
        private RectTransform? _detailGalleryRow;
        private readonly List<Image> _galleryThumbs = new List<Image>();
        private GameObject? _screenshotModal;
        private Image? _modalImage;

        /// <summary>Canvas root used to anchor the dependency prompt dialog.</summary>
        private Transform? _canvasRoot;

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
            LayoutElement scrollLe = scrollRect.gameObject.AddComponent<LayoutElement>();
            scrollLe.preferredWidth = ListWidth;
            scrollLe.minWidth = ListWidth;
            scrollLe.flexibleHeight = 1f;

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
            _detailDeps = UiBuilder.MakeText(c, "DetailDeps", "Dependencies: none", 12, UiBuilder.TextSecondary);
            AddFlexRow(_detailDeps.gameObject, preferredHeight: 18f);

            _detailConflicts = UiBuilder.MakeText(c, "DetailConflicts", "Conflicts: none", 12, UiBuilder.TextSecondary);
            AddFlexRow(_detailConflicts.gameObject, preferredHeight: 18f);

            _detailLoadOrder = UiBuilder.MakeText(c, "DetailLoadOrder", "Load Order: —", 12, UiBuilder.TextSecondary);
            AddFlexRow(_detailLoadOrder.gameObject, preferredHeight: 18f);

            UiBuilder.MakeHorizontalSeparator(c, UiBuilder.Border);

            // ── Rich content section ─────────────────────────────────────────
            // Shows count summary + first N item names (units, buildings, factions)
            _detailRichContent = UiBuilder.MakeText(c, "DetailRichContent", "Content: (none declared)", 12,
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
                btnRow.transform, "ToggleBtn", "Disable",
                UiBuilder.BgSurface, UiBuilder.TextPrimary,
                OnToggleSelected);
            LayoutElement toggleBtnLe = toggleBtn.gameObject.AddComponent<LayoutElement>();
            toggleBtnLe.preferredWidth = 90f;
            toggleBtnLe.minWidth = 90f;
            toggleBtnLe.preferredHeight = 30f;

            // ── Screenshot modal (full-size overlay, initially hidden) ────────
            BuildScreenshotModal();
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
                footer.transform, "ReloadBtn", "↺  Reload Packs",
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
            hlg.childControlWidth = true;
            hlg.childControlHeight = true;

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
                return;
            }

            PackDisplayInfo p = selected;

            // ── Name ──────────────────────────────────────────────────────────
            if (_detailName != null) _detailName.text = p.Name;

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
                    sb.Append("Content: (none declared)");
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
                        try { Application.OpenURL(capturedUrl); }
                        catch { } // safe-swallow: OpenURL is best-effort
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
            catch
            {
                // safe-swallow: screenshot load is best-effort UI decoration
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
    }
}
