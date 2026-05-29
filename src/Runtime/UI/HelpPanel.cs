#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using BepInEx.Logging;
using DINOForge.SDK.Json;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// In-game FAQ/Help panel displayed via F10 menu.
    /// Loads FAQ content from assets/help/faq.json and displays categorized Q&amp;A pairs
    /// in collapsible accordion sections.
    /// </summary>
    internal sealed class HelpPanel : MonoBehaviour
    {
        // ── Constants ────────────────────────────────────────────────────────────
        private const float PanelWidth = 600f;
        private const float PanelHeight = 500f;
        private const float HeaderHeight = 44f;
        private const float ItemHeight = 50f;
        private const float AnimDuration = 0.15f;

        // ── FAQ Data Structure ───────────────────────────────────────────────────
        [Serializable]
        private sealed class FaqCategory
        {
            [JsonPropertyName("name")]
            public string Name { get; set; } = "";

            [JsonPropertyName("items")]
            public List<FaqItem> Items { get; set; } = new();
        }

        [Serializable]
        private sealed class FaqItem
        {
            [JsonPropertyName("q")]
            public string Question { get; set; } = "";

            [JsonPropertyName("a")]
            public string Answer { get; set; } = "";
        }

        [Serializable]
        private sealed class FaqRoot
        {
            [JsonPropertyName("categories")]
            public List<FaqCategory> Categories { get; set; } = new();
        }

        // ── State ────────────────────────────────────────────────────────────────
        private ManualLogSource? _log;
        private List<FaqCategory>? _faqData;
        private CanvasGroup? _canvasGroup;
        private RectTransform? _panelRt;
        private bool _targetVisible;

        // ── UI References ────────────────────────────────────────────────────────
        private RectTransform? _contentRoot;
        private ScrollRect? _scrollRect;
        private readonly Dictionary<Transform, Button?> _categoryButtons = new();
        private readonly Dictionary<Transform, GameObject?> _contentContainers = new();

        // ── Lifecycle ────────────────────────────────────────────────────────────

        /// <summary>
        /// Initializes the help panel. Call from main thread.
        /// </summary>
        /// <param name="canvasRoot">Parent canvas transform.</param>
        /// <param name="log">Logger instance.</param>
        public void Build(Transform canvasRoot, ManualLogSource log)
        {
            _log = log;

            // Load FAQ data
            if (!LoadFaqData())
            {
                _log.LogWarning("Failed to load FAQ data. Help panel will be empty.");
                _faqData = new List<FaqCategory>();
            }

            // Root panel — centered modal-style
            GameObject rootGo = UiBuilder.MakePanel(canvasRoot, "HelpPanel",
                UiBuilder.BgDeep, new Vector2(PanelWidth, PanelHeight));
            RectTransform rootRt = rootGo.GetComponent<RectTransform>();
            rootRt.anchoredPosition = Vector2.zero;
            _panelRt = rootRt;

            _canvasGroup = rootGo.GetComponent<CanvasGroup>();
            if (_canvasGroup == null)
            {
                _canvasGroup = rootGo.AddComponent<CanvasGroup>();
            }
            _canvasGroup.alpha = 0f;
            _canvasGroup.blocksRaycasts = false;

            // Header: "FAQ / Help"
            GameObject headerGo = UiBuilder.MakePanel(rootGo.transform, "Header",
                UiBuilder.BgSurface, new Vector2(0f, HeaderHeight));
            RectTransform headerRt = headerGo.GetComponent<RectTransform>();
            headerRt.anchorMin = new Vector2(0f, 1f);
            headerRt.anchorMax = Vector2.one;
            headerRt.offsetMin = Vector2.zero;
            headerRt.offsetMax = new Vector2(0f, 0f);

            Text headerText = UiBuilder.MakeText(headerGo.transform, "Title",
                "FAQ / Help", 18, UiBuilder.TextPrimary);
            RectTransform headerTextRt = headerText.GetComponent<RectTransform>();
            headerTextRt.anchoredPosition = new Vector2(12f, 0f);
            headerTextRt.sizeDelta = new Vector2(PanelWidth - 24f, HeaderHeight);

            // Close button in header
            Button closeBtn = UiBuilder.MakeButton(headerGo.transform, "CloseButton",
                "✕", UiBuilder.Error, UiBuilder.TextPrimary, () => Hide());
            RectTransform closeButtonRt = closeBtn.GetComponent<RectTransform>();
            closeButtonRt.anchorMin = Vector2.one;
            closeButtonRt.anchorMax = Vector2.one;
            closeButtonRt.sizeDelta = new Vector2(40f, HeaderHeight);
            closeButtonRt.anchoredPosition = Vector2.zero;

            // Scrollable content area
            GameObject scrollGo = new GameObject("ScrollArea");
            scrollGo.transform.SetParent(rootGo.transform, false);
            RectTransform scrollRt = scrollGo.AddComponent<RectTransform>();
            scrollRt.anchorMin = Vector2.zero;
            scrollRt.anchorMax = Vector2.one;
            scrollRt.offsetMin = new Vector2(0f, 0f);
            scrollRt.offsetMax = new Vector2(0f, -HeaderHeight);

            _scrollRect = scrollGo.AddComponent<ScrollRect>();
            Image scrollBg = scrollGo.AddComponent<Image>();
            scrollBg.color = UiBuilder.BgDeep;

            // Scroll viewport and content
            GameObject viewportGo = new GameObject("Viewport");
            viewportGo.transform.SetParent(scrollGo.transform, false);
            RectTransform viewportRt = viewportGo.AddComponent<RectTransform>();
            viewportRt.anchorMin = Vector2.zero;
            viewportRt.anchorMax = Vector2.one;
            viewportRt.offsetMin = Vector2.zero;
            viewportRt.offsetMax = Vector2.zero;
            Image viewportImage = viewportGo.AddComponent<Image>();
            viewportImage.color = Color.clear;
            Mask viewportMask = viewportGo.AddComponent<Mask>();
            viewportMask.showMaskGraphic = false;

            _scrollRect.viewport = viewportRt;

            GameObject contentGo = new GameObject("Content");
            contentGo.transform.SetParent(viewportGo.transform, false);
            _contentRoot = contentGo.AddComponent<RectTransform>();
            _contentRoot.anchorMin = new Vector2(0f, 1f);
            _contentRoot.anchorMax = new Vector2(1f, 1f);
            _contentRoot.pivot = new Vector2(0.5f, 1f);
            _contentRoot.offsetMin = new Vector2(0f, 0f);
            _contentRoot.offsetMax = Vector2.zero;

            VerticalLayoutGroup contentLayout = contentGo.AddComponent<VerticalLayoutGroup>();
            contentLayout.childForceExpandHeight = false;
            contentLayout.childForceExpandWidth = true;
            contentLayout.spacing = 4f;
            contentLayout.padding = new RectOffset(8, 8, 8, 8);

            _scrollRect.content = _contentRoot;

            // Populate content from FAQ data
            PopulateFaqContent();

            // Make sure we can receive Esc to close
            Button raycastBtn = UiBuilder.MakeButton(rootGo.transform, "RaycastTarget",
                "", Color.clear, Color.clear, () => { });
            raycastBtn.GetComponent<Image>().raycastTarget = true;
            raycastBtn.interactable = false;
        }

        /// <summary>
        /// Shows the panel with fade-in animation.
        /// </summary>
        public void Show()
        {
            _targetVisible = true;
            StartCoroutine(FadeCoroutine(true));
        }

        /// <summary>
        /// Hides the panel with fade-out animation.
        /// </summary>
        public void Hide()
        {
            _targetVisible = false;
            StartCoroutine(FadeCoroutine(false));
        }

        /// <summary>
        /// Returns whether the panel is currently visible or animating visible.
        /// </summary>
        public bool IsVisible => _targetVisible || (_canvasGroup != null && _canvasGroup.alpha > 0.01f);

        // ── Implementation ───────────────────────────────────────────────────────

        private bool LoadFaqData()
        {
            try
            {
                // Try to load from StreamingAssets or bundled path
                string faqPath = Path.Combine(Application.streamingAssetsPath, "help", "faq.json");

                // Fallback: check relative paths for development
                if (!File.Exists(faqPath))
                {
                    faqPath = Path.Combine(Application.persistentDataPath, "help", "faq.json");
                }

                if (!File.Exists(faqPath))
                {
                    _log?.LogWarning($"FAQ file not found at {faqPath}");
                    return false;
                }

                string json = File.ReadAllText(faqPath);
                FaqRoot? root = JsonSerializer.Deserialize<FaqRoot>(json, JsonOptions.Compact);
                if (root?.Categories == null)
                {
                    _log?.LogWarning("FAQ JSON is invalid or missing categories");
                    return false;
                }

                _faqData = root.Categories;
                return true;
            }
            catch (Exception ex)
            {
                _log?.LogError($"Error loading FAQ data: {ex.Message}");
                return false;
            }
        }

        private void PopulateFaqContent()
        {
            if (_faqData == null || _contentRoot == null)
                return;

            foreach (var category in _faqData)
            {
                // Category header button
                Button categoryBtn = UiBuilder.MakeButton(_contentRoot, $"Category_{category.Name}",
                    category.Name, UiBuilder.Accent, UiBuilder.TextPrimary, () => { });
                RectTransform categoryRt = categoryBtn.GetComponent<RectTransform>();
                categoryRt.sizeDelta = new Vector2(0f, ItemHeight);

                // Content container (hidden by default)
                GameObject contentContainerGo = new GameObject($"Content_{category.Name}");
                contentContainerGo.transform.SetParent(_contentRoot, false);
                RectTransform contentContainerRt = contentContainerGo.AddComponent<RectTransform>();
                contentContainerRt.anchorMin = new Vector2(0f, 1f);
                contentContainerRt.anchorMax = new Vector2(1f, 1f);
                contentContainerRt.pivot = new Vector2(0.5f, 1f);
                contentContainerRt.sizeDelta = new Vector2(0f, 0f);

                VerticalLayoutGroup itemLayout = contentContainerGo.AddComponent<VerticalLayoutGroup>();
                itemLayout.childForceExpandHeight = false;
                itemLayout.childForceExpandWidth = true;
                itemLayout.spacing = 2f;
                itemLayout.padding = new RectOffset(12, 12, 4, 4);

                bool isExpanded = category == _faqData[0]; // Expand first category by default

                // Populate Q/A items in this category
                foreach (var item in category.Items)
                {
                    GameObject itemGo = new GameObject($"Item_{item.Question.Substring(0, Math.Min(20, item.Question.Length))}");
                    itemGo.transform.SetParent(contentContainerGo.transform, false);
                    RectTransform itemRt = itemGo.AddComponent<RectTransform>();
                    itemRt.sizeDelta = new Vector2(0f, 0f);

                    VerticalLayoutGroup itemVLayout = itemGo.AddComponent<VerticalLayoutGroup>();
                    itemVLayout.childForceExpandHeight = false;
                    itemVLayout.childForceExpandWidth = true;
                    itemVLayout.spacing = 2f;

                    // Question text
                    Text qText = UiBuilder.MakeText(itemGo.transform, "Question",
                        item.Question, 14, UiBuilder.TextSecondary);
                    RectTransform qRt = qText.GetComponent<RectTransform>();
                    qRt.sizeDelta = new Vector2(0f, 28f);

                    // Answer text
                    Text aText = UiBuilder.MakeText(itemGo.transform, "Answer",
                        item.Answer, 12, UiBuilder.TextSecondary);
                    RectTransform aRt = aText.GetComponent<RectTransform>();
                    aRt.sizeDelta = new Vector2(0f, 60f);
                    aText.gameObject.SetActive(isExpanded);

                    // Recalculate item height based on content visibility
                    LayoutElement itemLayoutElement = itemGo.AddComponent<LayoutElement>();
                    itemLayoutElement.preferredHeight = isExpanded ? 88f : 28f;

                    // Toggle answer visibility on category button click
                    categoryBtn.onClick.AddListener(() =>
                    {
                        isExpanded = !isExpanded;
                        aText.gameObject.SetActive(isExpanded);
                        itemLayoutElement.preferredHeight = isExpanded ? 88f : 28f;
                        LayoutRebuilder.ForceRebuildLayoutImmediate(_contentRoot);
                    });
                }

                // Expand/collapse container
                LayoutElement containerLayout = contentContainerGo.AddComponent<LayoutElement>();
                containerLayout.preferredHeight = isExpanded ? category.Items.Count * 92f : 0f;

                categoryBtn.onClick.AddListener(() =>
                {
                    isExpanded = !isExpanded;
                    contentContainerGo.SetActive(isExpanded);
                    containerLayout.preferredHeight = isExpanded ? category.Items.Count * 92f : 0f;
                    LayoutRebuilder.ForceRebuildLayoutImmediate(_contentRoot);
                });

                contentContainerGo.SetActive(isExpanded);

                _categoryButtons[contentContainerGo.transform] = categoryBtn;
                _contentContainers[categoryBtn.GetComponent<RectTransform>()] = contentContainerGo;
            }

            LayoutRebuilder.ForceRebuildLayoutImmediate(_contentRoot);
        }

        private IEnumerator FadeCoroutine(bool fadeIn)
        {
            if (_canvasGroup == null)
                yield break;

            float elapsed = 0f;
            float startAlpha = _canvasGroup.alpha;
            float targetAlpha = fadeIn ? 1f : 0f;

            _canvasGroup.blocksRaycasts = fadeIn;

            while (elapsed < AnimDuration)
            {
                elapsed += Time.unscaledDeltaTime;
                _canvasGroup.alpha = Mathf.Lerp(startAlpha, targetAlpha, elapsed / AnimDuration);
                yield return null;
            }

            _canvasGroup.alpha = targetAlpha;
            _canvasGroup.blocksRaycasts = fadeIn;
        }
    }
}
