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
            hlg.childControlWidth = true;
            hlg.childControlHeight = true;
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
            // Absolute-positioned list pane — no layout groups, no ScrollRect.
            // DINO's PlayerLoop replacement breaks Unity's layout rebuild cycle,
            // so we position everything manually with RectTransform anchors.
            GameObject pane = new GameObject("ListPane", typeof(RectTransform));
            pane.transform.SetParent(parent, false);

            LayoutElement paneLe = pane.AddComponent<LayoutElement>();
            paneLe.preferredWidth = ListWidth;
            paneLe.minWidth = ListWidth;

            // "Loaded Packs" header — top 32px of the pane
            GameObject listHeader = UiBuilder.MakePanel(pane.transform, "ListHeader",
                UiBuilder.BgSurface, new Vector2(ListWidth, 32f));
            RectTransform lhRt = listHeader.GetComponent<RectTransform>();
            lhRt.anchorMin = new Vector2(0f, 1f);
            lhRt.anchorMax = new Vector2(1f, 1f);
            lhRt.pivot = new Vector2(0.5f, 1f);
            lhRt.anchoredPosition = Vector2.zero;
            lhRt.sizeDelta = new Vector2(0f, 32f);

            Text lhTitle = UiBuilder.MakeText(listHeader.transform, "ListTitle",
                "Loaded Packs", 12, UiBuilder.TextSecondary, bold: false);
            RectTransform lhTitleRt = lhTitle.GetComponent<RectTransform>();
            lhTitleRt.anchorMin = Vector2.zero;
            lhTitleRt.anchorMax = Vector2.one;
            lhTitleRt.offsetMin = new Vector2(8f, 0f);
            lhTitleRt.offsetMax = new Vector2(-8f, 0f);

            // Content container — fills remaining space below header
            GameObject contentGo = new GameObject("ListContent", typeof(RectTransform));
            contentGo.transform.SetParent(pane.transform, false);
            RectTransform contentRt = contentGo.GetComponent<RectTransform>();
            contentRt.anchorMin = Vector2.zero;
            contentRt.anchorMax = Vector2.one;
            contentRt.offsetMin = Vector2.zero;
            contentRt.offsetMax = new Vector2(0f, -32f);

            _listContent = contentRt;
        }

        private void BuildDetailPane(Transform parent)
        {
            _detailPane = new GameObject("DetailPane", typeof(RectTransform), typeof(Image), typeof(Mask));
            _detailPane.transform.SetParent(parent, false);

            Image detailBg = _detailPane.GetComponent<Image>();
            detailBg.color = new Color(0f, 0f, 0f, 0.01f);
            detailBg.raycastTarget = false;
            Mask detailMask = _detailPane.GetComponent<Mask>();
            detailMask.showMaskGraphic = false;

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

            // Content + Overlaps side-by-side
            GameObject contentRow = new GameObject("ContentRow", typeof(RectTransform));
            contentRow.transform.SetParent(_detailPane.transform, false);
            HorizontalLayoutGroup contentHlg = contentRow.AddComponent<HorizontalLayoutGroup>();
            contentHlg.spacing = 12f;
            contentHlg.childForceExpandWidth = false;
            contentHlg.childForceExpandHeight = true;
            contentHlg.childControlWidth = true;
            LayoutElement contentRowLe = contentRow.AddComponent<LayoutElement>();
            contentRowLe.preferredHeight = 60f;
            contentRowLe.flexibleWidth = 1f;
            contentRowLe.flexibleHeight = 0.5f;

            _detailContent = UiBuilder.MakeText(contentRow.transform, "DetailContent",
                "Content: (none)", 12, new Color(0.6f, 0.85f, 0.6f, 1f));
            LayoutElement contentLe = _detailContent.gameObject.AddComponent<LayoutElement>();
            contentLe.flexibleWidth = 1f;

            _detailDetectedConflicts = UiBuilder.MakeText(contentRow.transform, "DetailDetectedConflicts",
                "", 12, new Color(0.9f, 0.6f, 0.2f, 1f));
            LayoutElement dcLe = _detailDetectedConflicts.gameObject.AddComponent<LayoutElement>();
            dcLe.flexibleWidth = 1f;

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
            if (_listContent == null) return;

            for (int i = _listContent.childCount - 1; i >= 0; i--)
            {
                Transform child = _listContent.GetChild(i);
                child.SetParent(null, false);
                Destroy(child.gameObject);
            }

            for (int i = 0; i < _presenter.Packs.Count; i++)
            {
                BuildPackListItem(_presenter.Packs[i], i);
            }
        }

        private void BuildPackListItem(PackDisplayInfo pack, int index)
        {
            if (_listContent == null) return;

            bool isSelected = index == _presenter.SelectedIndex;
            Color bgColor = isSelected ? UiBuilder.BgSurface : UiBuilder.BgDeep;

            GameObject card = UiBuilder.MakePanel(_listContent, $"PackItem_{pack.Id}", bgColor,
                new Vector2(ListWidth, ItemHeight));
            RectTransform cardRt = card.GetComponent<RectTransform>();
            cardRt.anchorMin = new Vector2(0f, 1f);
            cardRt.anchorMax = new Vector2(1f, 1f);
            cardRt.pivot = new Vector2(0.5f, 1f);
            cardRt.anchoredPosition = new Vector2(0f, -(index * (ItemHeight + 2f)));
            cardRt.sizeDelta = new Vector2(0f, ItemHeight);

            // Amber left-border for enabled packs
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
            }

            // Pack name — absolute positioned inside card
            Color nameColor = pack.IsEnabled ? UiBuilder.TextPrimary : UiBuilder.TextSecondary;
            if (!pack.IsEnabled) nameColor = new Color(nameColor.r, nameColor.g, nameColor.b, 0.6f);
            Text nameText = UiBuilder.MakeText(card.transform, "PackName", pack.Name, 13,
                nameColor, bold: isSelected);
            RectTransform nameRt = nameText.GetComponent<RectTransform>();
            nameRt.anchorMin = new Vector2(0f, 0f);
            nameRt.anchorMax = new Vector2(1f, 1f);
            int leftPad = pack.IsEnabled ? 10 : 6;
            nameRt.offsetMin = new Vector2(leftPad, 2f);
            nameRt.offsetMax = new Vector2(-6f, -2f);

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
                if (_detailDesc != null) _detailDesc.text = "";
                if (_detailDeps != null) _detailDeps.text = "Dependencies: none";
                if (_detailConflicts != null) _detailConflicts.text = "Conflicts: none";
                if (_detailLoadOrder != null) _detailLoadOrder.text = "Load Order: —";
                if (_detailContent != null) _detailContent.text = "Content: (none)";
                if (_detailDetectedConflicts != null) _detailDetectedConflicts.text = "";
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
                    descText = descText + "\n\n<color=#e05252>Errors:</color>\n"
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

            if (_detailContent != null)
            {
                if (p.ContentSummary.Count == 0)
                {
                    _detailContent.text = "Content: (none declared)";
                }
                else
                {
                    System.Text.StringBuilder sb = new System.Text.StringBuilder(256);
                    sb.Append("<color=#88dd88>Content:</color>\n");
                    foreach (System.Collections.Generic.KeyValuePair<string, int> kv in p.ContentSummary)
                    {
                        sb.Append($"  {kv.Key}: {kv.Value} file(s)\n");
                    }
                    _detailContent.text = sb.ToString().TrimEnd('\n');
                }
            }

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

            ClearCurrentSelection();
            OnPackToggled?.Invoke(updated.Id, updated.IsEnabled);
            QueueListRefresh();
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
