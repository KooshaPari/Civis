#nullable enable
using System;
using System.Collections;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// UGUI developer debug panel (replaces the legacy IMGUI DebugOverlayBehaviour).
    /// 380px wide right-side slide-in panel with collapsible sections:
    /// Platform Status / ECS Worlds / Systems / Archetypes / Errors.
    /// </summary>
    public class DebugPanel : MonoBehaviour
    {
        // ── Constants ─────────────────────────────────────────────────────────────
        private const float PanelWidth = 380f;
        private const float HeaderHeight = 44f;

        // ── State ─────────────────────────────────────────────────────────────────
        private ModPlatform? _modPlatform;
        private bool _bridgeStatusCache;

        private CanvasGroup? _canvasGroup;
        private RectTransform? _panelRt;
        private bool _targetVisible;

        // Section toggle state
        private bool _showPlatform = true;
        private bool _showWorlds = true;
        private bool _showSystems = false;
        private bool _showArchetypes = false;
        private bool _showErrors = false;

        // Content area for dynamic updates
        private RectTransform? _contentRoot;
        private ScrollRect? _scrollRect;
        private float _refreshTimer;
        private bool _refreshQueued;
        private const float RefreshInterval = 0.5f;

        // ── Bootstrap ─────────────────────────────────────────────────────────────

        /// <summary>
        /// Builds the UGUI hierarchy. Call from main thread (Start/Awake).
        /// </summary>
        /// <param name="canvasRoot">Root canvas transform to attach to.</param>
        public void Build(Transform canvasRoot)
        {
            // Root panel — right edge
            GameObject rootGo = UiBuilder.MakePanel(canvasRoot, "DebugPanel",
                UiBuilder.BgDeep, new Vector2(PanelWidth, 0f));
            RectTransform rootRt = rootGo.GetComponent<RectTransform>();
            rootRt.anchorMin = new Vector2(1f, 0f);
            rootRt.anchorMax = Vector2.one;
            rootRt.pivot = new Vector2(1f, 0.5f);
            rootRt.offsetMin = new Vector2(-PanelWidth, 0f);
            rootRt.offsetMax = Vector2.zero;

            _panelRt = rootRt;
            _canvasGroup = UiBuilder.EnsureCanvasGroup(rootGo);
            _canvasGroup.alpha = 0f;
            _canvasGroup.interactable = false;
            _canvasGroup.blocksRaycasts = false;
            _targetVisible = false;

            BuildHeader(rootGo.transform);
            BuildScrollContent(rootGo.transform);
        }

        // ── Public API ────────────────────────────────────────────────────────────

        /// <summary>Provides a ModPlatform reference for status display.</summary>
        public void SetModPlatform(ModPlatform? modPlatform)
        {
            _modPlatform = modPlatform;
            Debug.Log($"[DebugPanel] SetModPlatform called: {(modPlatform != null ? "set to ModPlatform instance" : "set to NULL")}");
            // Refresh regardless of visibility so that the next Show() has fresh data.
            RefreshContent();
        }

        /// <summary>
        /// Forces an immediate content refresh and updates the bridge status snapshot.
        /// Call from RuntimeDriver after F9 is pressed so the panel shows up-to-date data.
        /// </summary>
        public void ForceRefresh()
        {
            _bridgeStatusCache = Plugin.SharedBridgeServer?.IsServerThreadAlive ?? false;
            RefreshContent();
        }

        /// <summary>Shows the panel immediately (no animation).</summary>
        public void Show()
        {
            _targetVisible = true;
            if (_canvasGroup != null)
            {
                _canvasGroup.alpha = 1f;
                _canvasGroup.interactable = true;
                _canvasGroup.blocksRaycasts = true;
            }

            // Set panel to fully open position immediately
            if (_panelRt != null)
            {
                _panelRt.offsetMin = new Vector2(-PanelWidth, 0f);
                _panelRt.offsetMax = Vector2.zero;
                _panelRt.gameObject.SetActive(true);
            }

            // Immediately refresh content so panel displays on first open.
            // ForceRefresh() also captures a fresh bridge-status snapshot.
            ForceRefresh();

            // Force all content children visible
            if (_contentRoot != null)
            {
                _contentRoot.gameObject.SetActive(true);
                for (int i = 0; i < _contentRoot.childCount; i++)
                {
                    _contentRoot.GetChild(i).gameObject.SetActive(true);
                }
                UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_contentRoot);
            }

            // Force entire panel layout rebuild
            if (_panelRt != null)
            {
                UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_panelRt);
            }

            // Fix (iter-149): DINO never runs MonoBehaviour.Update/LateUpdate, so the
            // ScrollRect never repositions its content after it is built/resized while the
            // panel is inactive — leaving the content scrolled out of the viewport (the
            // panel body appeared empty/black). Pin the content back to the top now that the
            // layout has been rebuilt with a known content size.
            if (_scrollRect != null)
            {
                _scrollRect.verticalNormalizedPosition = 1f;
                if (_contentRoot != null)
                    _contentRoot.anchoredPosition = new Vector2(_contentRoot.anchoredPosition.x, 0f);
            }

            if (_autoRefreshCoroutine != null) StopCoroutine(_autoRefreshCoroutine);
            _autoRefreshCoroutine = StartCoroutine(AutoRefreshLoop());

            Debug.Log($"[DebugPanel] Show() called. ModPlatform={(_modPlatform != null ? "set" : "NULL")}. Content refreshed.");
        }

        private Coroutine? _autoRefreshCoroutine;

        /// <summary>Hides the panel immediately (no animation).</summary>
        public void Hide()
        {
            _targetVisible = false;
            if (_autoRefreshCoroutine != null)
            {
                StopCoroutine(_autoRefreshCoroutine);
                _autoRefreshCoroutine = null;
            }
            if (_canvasGroup != null)
            {
                _canvasGroup.alpha = 0f;
                _canvasGroup.interactable = false;
                _canvasGroup.blocksRaycasts = false;
            }

            Debug.Log("[DebugPanel] Hide() called.");
        }

        private IEnumerator AutoRefreshLoop()
        {
            while (_targetVisible)
            {
                yield return new WaitForSecondsRealtime(RefreshInterval);
                if (_targetVisible && _contentRoot != null)
                {
                    RefreshContent();
                }
            }
        }

        /// <summary>Whether the panel is currently visible.</summary>
        public bool IsVisible => _targetVisible;

        // ── MonoBehaviour ─────────────────────────────────────────────────────────

        private void Update()
        {
            // NOTE: MonoBehaviour.Update() NEVER fires in DINO (Unity PlayerLoop is replaced by ECS).
            // This method is kept as a safeguard but will not execute.
            // All panel visibility changes are now immediate (no animation).
            // Panel refresh happens on Show() and via F9 key handler in RuntimeDriver.

            if (!IsVisible) return;

            _refreshTimer = _refreshTimer + Time.deltaTime;
            if (_refreshTimer >= RefreshInterval)
            {
                _refreshTimer = 0f;
                RefreshContent();
            }
        }

        // ── UI construction ────────────────────────────────────────────────────────

        private void BuildHeader(Transform parent)
        {
            GameObject header = UiBuilder.MakePanel(parent, "DebugHeader",
                UiBuilder.BgSurface, new Vector2(0f, HeaderHeight));
            RectTransform hRt = header.GetComponent<RectTransform>();
            hRt.anchorMin = new Vector2(0f, 1f);
            hRt.anchorMax = Vector2.one;
            hRt.pivot = new Vector2(0.5f, 1f);
            hRt.offsetMin = Vector2.zero;
            hRt.offsetMax = Vector2.zero;
            hRt.sizeDelta = new Vector2(0f, HeaderHeight);

            UiBuilder.AddHorizontalLayout(header, 8f, new RectOffset(12, 8, 6, 6));

            Text title = UiBuilder.MakeText(header.transform, "DebugTitle",
                "DINOForge Debug", 14, UiBuilder.Accent, bold: true);
            LayoutElement titleLe = title.gameObject.AddComponent<LayoutElement>();
            titleLe.flexibleWidth = 1f;

            // Keyboard shortcuts strip
            Text shortcutsLabel = UiBuilder.MakeText(header.transform, "Shortcuts",
                "F8=Dump  F9=Debug  F10=Mods", 10, UiBuilder.TextSecondary);
            LayoutElement scLe = shortcutsLabel.gameObject.AddComponent<LayoutElement>();
            scLe.preferredWidth = 160f;

            Button closeBtn = UiBuilder.MakeButton(
                header.transform, "DebugClose", "×",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () =>
                {
                    ClearCurrentSelection();
                    Hide();
                });
            LayoutElement closeLe = closeBtn.gameObject.AddComponent<LayoutElement>();
            closeLe.preferredWidth = 28f;
            closeLe.preferredHeight = 28f;

            // Separator
            GameObject sep = UiBuilder.MakeHorizontalSeparator(parent, UiBuilder.Border);
            RectTransform sepRt = sep.GetComponent<RectTransform>();
            sepRt.anchorMin = new Vector2(0f, 1f);
            sepRt.anchorMax = Vector2.one;
            sepRt.pivot = new Vector2(0.5f, 1f);
            sepRt.anchoredPosition = new Vector2(0f, -HeaderHeight);
            sepRt.sizeDelta = new Vector2(0f, 1f);
        }

        private void BuildScrollContent(Transform parent)
        {
            (ScrollRect scrollRect, RectTransform content) = UiBuilder.MakeScrollView(
                parent, "DebugScroll", Vector2.zero);

            RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
            scrollRt.anchorMin = Vector2.zero;
            scrollRt.anchorMax = Vector2.one;
            scrollRt.offsetMin = Vector2.zero;
            scrollRt.offsetMax = new Vector2(0f, -(HeaderHeight + 1f));

            _scrollRect = scrollRect;
            _contentRoot = content;
            RefreshContent();
        }

        private void QueueRefreshContent()
        {
            if (_refreshQueued) return;
            _refreshQueued = true;
            StartCoroutine(RefreshContentNextFrame());
        }

        private IEnumerator RefreshContentNextFrame()
        {
            yield return null;
            _refreshQueued = false;
            RefreshContent();
        }

        // ── Dynamic content refresh ────────────────────────────────────────────────

        private void RefreshContent()
        {
            if (_contentRoot == null)
            {
                Debug.LogWarning("[DebugPanel.RefreshContent] _contentRoot is NULL - cannot build content");
                return;
            }

            Debug.Log($"[DebugPanel.RefreshContent] Building content. _modPlatform={(_modPlatform != null ? "SET" : "NULL")}, childCount before={_contentRoot.childCount}");

            // Destroy previous content immediately so new children don't coexist
            // with deferred-destroy ghosts from the same frame (#905).
            for (int i = _contentRoot.childCount - 1; i >= 0; i--)
            {
                DestroyImmediate(_contentRoot.GetChild(i).gameObject);
            }

            // Platform Status section
            BuildSection(_contentRoot, "Platform Status", ref _showPlatform, BuildPlatformContent);

            UiBuilder.MakeHorizontalSeparator(_contentRoot, UiBuilder.Border);

            // ECS Worlds section
            BuildSection(_contentRoot, "ECS Worlds", ref _showWorlds, BuildWorldsContent);

            UiBuilder.MakeHorizontalSeparator(_contentRoot, UiBuilder.Border);

            // Systems section
            BuildSection(_contentRoot, "Systems", ref _showSystems, BuildSystemsContent);

            UiBuilder.MakeHorizontalSeparator(_contentRoot, UiBuilder.Border);

            // Archetypes section
            BuildSection(_contentRoot, "Archetypes (top 20)", ref _showArchetypes, BuildArchetypesContent);

            // Errors section (only if there are errors)
            int errorCount = GetErrorCount();
            if (errorCount > 0)
            {
                UiBuilder.MakeHorizontalSeparator(_contentRoot, UiBuilder.Border);
                BuildSection(_contentRoot, $"Errors ({errorCount})", ref _showErrors, BuildErrorsContent);
            }

            // Copy errors button
            if (errorCount > 0)
            {
                Button copyBtn = UiBuilder.MakeButton(
                    _contentRoot, "CopyErrorsBtn", "Copy Errors to Clipboard",
                    UiBuilder.BgSurface, UiBuilder.Warning,
                    CopyErrorsToClipboard);
                LayoutElement copyLe = copyBtn.gameObject.AddComponent<LayoutElement>();
                copyLe.preferredHeight = 30f;
                copyLe.flexibleWidth = 1f;
            }

            // Drive layout immediately so all sections are correctly sized.
            UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_contentRoot);
        }

        private void BuildSection(
            RectTransform parent,
            string title,
            ref bool expanded,
            Action<Transform> buildContent)
        {
            // Capture ref value for lambda
            bool isExpanded = expanded;

            GameObject section = new GameObject($"Section_{title}", typeof(RectTransform));
            section.transform.SetParent(parent, false);
            VerticalLayoutGroup sectionVlg = section.AddComponent<VerticalLayoutGroup>();
            sectionVlg.childForceExpandWidth = true;
            sectionVlg.childForceExpandHeight = false;
            sectionVlg.spacing = 0f;
            LayoutElement sectionLe = section.AddComponent<LayoutElement>();
            sectionLe.flexibleWidth = 1f;

            RectTransform sectionRt = section.GetComponent<RectTransform>();

            // Toggle header row
            GameObject headerRow = UiBuilder.MakePanel(section.transform, "SectionHeader",
                UiBuilder.BgSurface, new Vector2(0f, 28f));
            LayoutElement headerLe = headerRow.AddComponent<LayoutElement>();
            headerLe.preferredHeight = 28f;
            headerLe.flexibleWidth = 1f;

            HorizontalLayoutGroup headerHlg = headerRow.AddComponent<HorizontalLayoutGroup>();
            headerHlg.spacing = 6f;
            headerHlg.padding = new RectOffset(10, 6, 4, 4);
            headerHlg.childForceExpandWidth = false;
            headerHlg.childForceExpandHeight = true;

            Text chevron = UiBuilder.MakeText(headerRow.transform, "Chevron",
                isExpanded ? "▼" : "▶", 11, UiBuilder.TextSecondary);
            chevron.GetComponent<RectTransform>().sizeDelta = new Vector2(16f, 20f);
            LayoutElement chevronLe = chevron.gameObject.AddComponent<LayoutElement>();
            chevronLe.preferredWidth = 16f;

            Text sectionTitle = UiBuilder.MakeText(headerRow.transform, "SectionTitle",
                title, 12, UiBuilder.TextPrimary, bold: true);
            LayoutElement titleLe = sectionTitle.gameObject.AddComponent<LayoutElement>();
            titleLe.flexibleWidth = 1f;

            // Content container (shown/hidden by toggle)
            GameObject contentGo = new GameObject("SectionContent", typeof(RectTransform));
            contentGo.transform.SetParent(section.transform, false);
            VerticalLayoutGroup contentVlg = contentGo.AddComponent<VerticalLayoutGroup>();
            contentVlg.childForceExpandWidth = true;
            contentVlg.childForceExpandHeight = false;
            contentVlg.spacing = 2f;
            contentVlg.padding = new RectOffset(14, 6, 4, 4);
            LayoutElement contentLe = contentGo.AddComponent<LayoutElement>();
            contentLe.flexibleWidth = 1f;
            contentGo.SetActive(isExpanded);

            if (isExpanded)
            {
                buildContent(contentGo.transform);
            }

            // Click header to toggle
            Button headerBtn = headerRow.AddComponent<Button>();
            ColorBlock cb = headerBtn.colors;
            cb.normalColor = UiBuilder.BgSurface;
            cb.highlightedColor = Color.Lerp(UiBuilder.BgSurface, Color.white, 0.08f);
            cb.pressedColor = Color.Lerp(UiBuilder.BgSurface, Color.black, 0.1f);
            cb.selectedColor = UiBuilder.BgSurface;
            cb.colorMultiplier = 1f;
            headerBtn.colors = cb;
            headerBtn.targetGraphic = headerRow.GetComponent<Image>();
            headerBtn.onClick.AddListener(() =>
            {
                // We cannot modify a ref field inside a lambda;
                // instead we schedule a full refresh by flipping the backing state
                // via the section name as a key.
                ToggleSection(title);
            });
        }

        private void ToggleSection(string title)
        {
            switch (title)
            {
                case "Platform Status": _showPlatform = !_showPlatform; break;
                case "ECS Worlds": _showWorlds = !_showWorlds; break;
                case "Systems": _showSystems = !_showSystems; break;
                case string s when s.StartsWith("Archetypes"):
                    _showArchetypes = !_showArchetypes; break;
                case string s when s.StartsWith("Errors"):
                    _showErrors = !_showErrors; break;
            }
            ClearCurrentSelection();
            QueueRefreshContent();
        }

        private void BuildPlatformContent(Transform parent)
        {
            AddInfoRow(parent, "Version", PluginInfo.VERSION, UiBuilder.Accent);

            float fps = 1f / Mathf.Max(Time.unscaledDeltaTime, 0.001f);
            long gcBytes = GC.GetTotalMemory(false);
            string gcMb = (gcBytes / (1024.0 * 1024.0)).ToString("F1");
            AddInfoRow(parent, "FPS", $"{fps:F0}", fps > 30f ? UiBuilder.Success : UiBuilder.Warning);
            AddInfoRow(parent, "GC Heap", $"{gcMb} MB", UiBuilder.TextSecondary);

            AddInfoRow(parent, "Bridge", _bridgeStatusCache ? "connected" : "not connected",
                _bridgeStatusCache ? UiBuilder.Success : UiBuilder.TextSecondary);

            if (_modPlatform != null)
            {
                AddInfoRow(parent, "Initialized", _modPlatform.IsInitialized ? "true" : "false",
                    _modPlatform.IsInitialized ? UiBuilder.Success : UiBuilder.Warning);
                AddInfoRow(parent, "World Ready", _modPlatform.IsWorldReady ? "true" : "false",
                    _modPlatform.IsWorldReady ? UiBuilder.Success : UiBuilder.Warning);
                AddInfoRow(parent, "Packs Dir", TruncatePath(_modPlatform.PacksDirectory, 30),
                    UiBuilder.TextSecondary);

                // Loaded packs — count + names + content summary
                System.Collections.Generic.IReadOnlyList<PackDisplayInfo> packInfos =
                    _modPlatform.GetLoadedPackDisplayInfos();
                int packCount = packInfos.Count;
                AddInfoRow(parent, "Loaded Packs", packCount.ToString(),
                    packCount > 0 ? UiBuilder.Success : UiBuilder.TextSecondary);

                int showMax = Math.Min(packInfos.Count, 10);
                for (int i = 0; i < showMax; i++)
                {
                    PackDisplayInfo pi = packInfos[i];
                    string contentBrief = "";
                    if (pi.ContentSummary.Count > 0)
                    {
                        System.Text.StringBuilder sb = new System.Text.StringBuilder(64);
                        foreach (System.Collections.Generic.KeyValuePair<string, int> kv in pi.ContentSummary)
                        {
                            if (sb.Length > 0) sb.Append(", ");
                            sb.Append($"{kv.Key}:{kv.Value}");
                        }
                        contentBrief = $" [{sb}]";
                    }
                    Color nameColor = pi.IsEnabled ? UiBuilder.TextPrimary : UiBuilder.TextSecondary;
                    string enableTag = pi.IsEnabled ? "" : " (off)";
                    UiBuilder.MakeText(parent, $"Pack_{i}", $"  • {pi.Name}{enableTag}{contentBrief}",
                        10, nameColor);
                }

                if (packInfos.Count > 10)
                {
                    UiBuilder.MakeText(parent, "PacksMore",
                        $"  ... and {packInfos.Count - 10} more", 10, UiBuilder.TextSecondary);
                }

                int errCount = GetErrorCount();
                if (errCount > 0)
                {
                    AddInfoRow(parent, "Load Errors", errCount.ToString(), UiBuilder.Error);
                }
            }
            else
            {
                // Platform not yet initialised — likely still at main menu startup
                UiBuilder.MakeText(parent, "NoPlatform",
                    "  ModPlatform: initializing…", 11, UiBuilder.TextSecondary);
                UiBuilder.MakeText(parent, "NoPlatformHint",
                    "  (packs load after first scene transition)", 10, UiBuilder.TextSecondary);
            }
        }

        private void BuildWorldsContent(Transform parent)
        {
            try
            {
                if (Unity.Entities.World.All.Count == 0)
                {
                    // Normal during main menu — DINO has no ECS world until gameplay starts
                    UiBuilder.MakeText(parent, "NoWorlds",
                        "  No ECS world (main menu — expected)", 11, UiBuilder.TextSecondary);
                    return;
                }

                foreach (Unity.Entities.World world in Unity.Entities.World.All)
                {
                    if (!world.IsCreated) continue;

                    AddInfoRow(parent, world.Name, "", UiBuilder.Accent);

                    try
                    {
                        Unity.Entities.EntityManager em = world.EntityManager;
                        using Unity.Collections.NativeArray<Unity.Entities.Entity> entities =
                            em.GetAllEntities(Unity.Collections.Allocator.Temp);
                        AddInfoRow(parent, "  Entities", entities.Length.ToString(), UiBuilder.TextPrimary);
                        AddInfoRow(parent, "  Systems", world.Systems.Count.ToString(), UiBuilder.TextPrimary);
                    }
                    catch (Exception ex)
                    {
                        UiBuilder.MakeText(parent, "WorldErr", $"  Error: {ex.Message}", 10, UiBuilder.Error);
                    }
                }
            }
            catch (Exception ex)
            {
                UiBuilder.MakeText(parent, "WorldsErr", $"  Error: {ex.Message}", 10, UiBuilder.Error);
            }
        }

        private void BuildSystemsContent(Transform parent)
        {
            try
            {
                if (Unity.Entities.World.All.Count == 0) return;

                foreach (Unity.Entities.World world in Unity.Entities.World.All)
                {
                    if (!world.IsCreated) continue;

                    try
                    {
                        var systems = world.Systems;
                        int limit = Math.Min(systems.Count, 30);
                        for (int i = 0; i < limit; i++)
                        {
                            Unity.Entities.ComponentSystemBase sys = systems[i];
                            Color dotColor = sys.Enabled ? UiBuilder.Success : UiBuilder.TextSecondary;
                            AddInfoRow(parent, $"  {(sys.Enabled ? "●" : "○")} {sys.GetType().Name}",
                                "", dotColor);
                        }
                    }
                    catch { } // safe-swallow: debug panel render best-effort
                }
            }
            catch { } // safe-swallow: debug panel enumeration best-effort
        }

        private void BuildArchetypesContent(Transform parent)
        {
            try
            {
                if (Unity.Entities.World.All.Count == 0)
                {
                    UiBuilder.MakeText(parent, "NoArchetypes", "  No ECS worlds (main menu)", 10, UiBuilder.TextSecondary);
                    return;
                }

                int shown = 0;
                foreach (Unity.Entities.World world in Unity.Entities.World.All)
                {
                    if (!world.IsCreated) continue;
                    using (var query = world.EntityManager.CreateEntityQuery(
                        new Unity.Entities.EntityQueryDesc
                        {
                            All = System.Array.Empty<Unity.Entities.ComponentType>(),
                            Options = Unity.Entities.EntityQueryOptions.IncludePrefab
                        }))
                    {
                        int totalEntities = query.CalculateEntityCount();
                        UiBuilder.MakeText(parent, $"Arch_{world.Name}",
                            $"  {world.Name}: {totalEntities} entities", 10, UiBuilder.TextPrimary);
                    }

                    shown++;
                    if (shown >= 6) break;
                }

                if (shown == 0)
                {
                    UiBuilder.MakeText(parent, "NoArchetypes", "  No active worlds found", 10, UiBuilder.TextSecondary);
                }
            }
            catch (Exception ex)
            {
                UiBuilder.MakeText(parent, "ArchetypeErr",
                    $"  Error: {ex.GetType().Name}", 10, UiBuilder.Error);
            }
        }

        private void BuildErrorsContent(Transform parent)
        {
            System.Collections.Generic.IReadOnlyList<string>? errors =
                _modPlatform?.ContentLoader?.LastLoadErrors;

            if (errors == null || errors.Count == 0)
            {
                UiBuilder.MakeText(parent, "NoErrors", "  (No errors)", 11, UiBuilder.TextSecondary);
                return;
            }

            int maxShow = Math.Min(10, errors.Count);
            for (int i = 0; i < maxShow; i++)
            {
                string display = errors[i].Length > 80
                    ? errors[i].Substring(0, 77) + "..."
                    : errors[i];
                UiBuilder.MakeText(parent, $"Err_{i}", $"  • {display}", 11, UiBuilder.Error);
            }

            if (errors.Count > 10)
            {
                UiBuilder.MakeText(parent, "MoreErrors",
                    $"  ... and {errors.Count - 10} more", 10, UiBuilder.TextSecondary);
            }
        }

        // ── Helpers ────────────────────────────────────────────────────────────────

        private void AddInfoRow(Transform parent, string key, string value, Color valueColor)
        {
            GameObject row = new GameObject($"Row_{key}", typeof(RectTransform));
            row.transform.SetParent(parent, false);
            HorizontalLayoutGroup rowHlg = row.AddComponent<HorizontalLayoutGroup>();
            rowHlg.spacing = 4f;
            rowHlg.childForceExpandWidth = false;
            rowHlg.childForceExpandHeight = false;
            LayoutElement rowLe = row.AddComponent<LayoutElement>();
            rowLe.preferredHeight = 16f;
            rowLe.flexibleWidth = 1f;

            Text keyText = UiBuilder.MakeText(row.transform, "Key", key, 11, UiBuilder.TextSecondary);
            LayoutElement keyLe = keyText.gameObject.AddComponent<LayoutElement>();
            keyLe.preferredWidth = 160f;
            keyLe.minWidth = 100f;

            if (!string.IsNullOrEmpty(value))
            {
                Text valText = UiBuilder.MakeText(row.transform, "Val", value, 11, valueColor);
                LayoutElement valLe = valText.gameObject.AddComponent<LayoutElement>();
                valLe.flexibleWidth = 1f;
            }
        }

        private int GetErrorCount()
        {
            try { return _modPlatform?.ContentLoader?.LastLoadErrorCount ?? 0; }
            catch { return 0; } // safe-swallow: debug panel error badge is best-effort and must never break UI refresh
        }

        private void CopyErrorsToClipboard()
        {
            System.Collections.Generic.IReadOnlyList<string>? errors =
                _modPlatform?.ContentLoader?.LastLoadErrors;
            if (errors == null || errors.Count == 0) return;

            GUIUtility.systemCopyBuffer = string.Join("\n", errors);
            ClearCurrentSelection();
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
            catch { } // safe-swallow: debug UI selection cleanup is best-effort
        }

        private static string TruncatePath(string path, int maxLen)
        {
            if (string.IsNullOrEmpty(path)) return "(none)";
            return path.Length <= maxLen ? path : "..." + path.Substring(path.Length - maxLen);
        }
    }
}
