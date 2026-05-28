#nullable enable
using System;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Full-screen loading overlay shown during DINOForge's mod initialization phase.
    /// Displays progress as packs are loaded (e.g., "Loading pack 3 of 9: Modern Warfare").
    ///
    /// Lifecycle:
    /// - Created by RuntimeDriver.Initialize() when mod loading starts
    /// - SetPackProgress() called by ModPlatform during pack enumeration
    /// - Hide() called when pack loading completes (no longer shown after MainMenu scene loads)
    ///
    /// Architecture:
    /// - ScreenSpaceOverlay Canvas with sortingOrder 9999 (always on top)
    /// - Dark semi-transparent background (#000000 alpha 0.85)
    /// - Centered title, subtitle, progress text, and pack icons row
    /// - Uses UnityEngine.UI.Text for netstandard2.0 compatibility (no TMPro)
    /// </summary>
    internal sealed class ModLoadingOverlay : MonoBehaviour
    {
        private Canvas? _canvas;
        private CanvasGroup? _canvasGroup;
        private Text? _titleText;
        private Text? _subtitleText;
        private Text? _progressText;
        private Transform? _iconContainer;
        private float _fadeOutDuration = 0.5f;
        private float _fadeOutElapsed = 0f;
        private bool _isFadingOut;

        /// <summary>Creates and initializes the loading overlay on the given parent GameObject.</summary>
        public static ModLoadingOverlay? Create(GameObject parent)
        {
            try
            {
                var overlay = parent.AddComponent<ModLoadingOverlay>();
                overlay.BuildUI();
                return overlay;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[ModLoadingOverlay] Failed to create: {ex}");
                return null;
            }
        }

        private void BuildUI()
        {
            // Create canvas GameObject
            var canvasGo = new GameObject("ModLoadingOverlay_Canvas");
            canvasGo.transform.SetParent(transform, false);

            // Canvas component
            _canvas = canvasGo.AddComponent<Canvas>();
            _canvas.renderMode = RenderMode.ScreenSpaceOverlay;
            _canvas.overrideSorting = true;
            _canvas.sortingOrder = 9999; // Always on top

            // CanvasGroup for fade-out effect
            _canvasGroup = canvasGo.AddComponent<CanvasGroup>();
            _canvasGroup.alpha = 1f;

            // RectTransform for canvas
            RectTransform canvasRt = canvasGo.GetComponent<RectTransform>();
            canvasRt.offsetMin = Vector2.zero;
            canvasRt.offsetMax = Vector2.zero;

            // ── Dark semi-transparent background ──────────────────────────────────
            var bgGo = new GameObject("Background");
            bgGo.transform.SetParent(canvasGo.transform, false);
            RectTransform bgRt = bgGo.AddComponent<RectTransform>();
            bgRt.offsetMin = Vector2.zero;
            bgRt.offsetMax = Vector2.zero;

            Image bgImage = bgGo.AddComponent<Image>();
            bgImage.color = new Color(0f, 0f, 0f, 0.85f); // Dark semi-transparent

            // ── Content container (centered) ──────────────────────────────────────
            var contentGo = new GameObject("Content");
            contentGo.transform.SetParent(canvasGo.transform, false);
            RectTransform contentRt = contentGo.AddComponent<RectTransform>();
            contentRt.anchoredPosition = Vector2.zero;
            contentRt.sizeDelta = new Vector2(600f, 300f); // Content box size

            // LayoutGroup for vertical layout
            var layoutGrp = contentGo.AddComponent<VerticalLayoutGroup>();
            layoutGrp.childForceExpandHeight = false;
            layoutGrp.childForceExpandWidth = false;
            layoutGrp.spacing = 20f;

            // ── Title text ────────────────────────────────────────────────────────
            var titleGo = new GameObject("Title");
            titleGo.transform.SetParent(contentGo.transform, false);
            RectTransform titleRt = titleGo.AddComponent<RectTransform>();
            titleRt.sizeDelta = new Vector2(600f, 80f);

            _titleText = titleGo.AddComponent<Text>();
            _titleText.text = "DINOForge";
            _titleText.font = Resources.GetBuiltinResource<Font>("Arial.ttf");
            _titleText.fontSize = 48;
            _titleText.fontStyle = FontStyle.Bold;
            _titleText.alignment = TextAnchor.MiddleCenter;
            _titleText.color = new Color(0.2f, 0.8f, 0.95f, 1f); // DINOForge cyan-ish primary color

            // ── Subtitle text ─────────────────────────────────────────────────────
            var subtitleGo = new GameObject("Subtitle");
            subtitleGo.transform.SetParent(contentGo.transform, false);
            RectTransform subtitleRt = subtitleGo.AddComponent<RectTransform>();
            subtitleRt.sizeDelta = new Vector2(600f, 40f);

            _subtitleText = subtitleGo.AddComponent<Text>();
            _subtitleText.text = "Loading mods...";
            _subtitleText.font = Resources.GetBuiltinResource<Font>("Arial.ttf");
            _subtitleText.fontSize = 24;
            _subtitleText.alignment = TextAnchor.MiddleCenter;
            _subtitleText.color = new Color(0.9f, 0.9f, 0.9f, 1f); // Light gray

            // ── Progress text ────────────────────────────────────────────────────
            var progressGo = new GameObject("Progress");
            progressGo.transform.SetParent(contentGo.transform, false);
            RectTransform progressRt = progressGo.AddComponent<RectTransform>();
            progressRt.sizeDelta = new Vector2(600f, 50f);

            _progressText = progressGo.AddComponent<Text>();
            _progressText.text = "Initializing...";
            _progressText.font = Resources.GetBuiltinResource<Font>("Arial.ttf");
            _progressText.fontSize = 18;
            _progressText.alignment = TextAnchor.MiddleCenter;
            _progressText.color = new Color(0.7f, 0.7f, 0.7f, 1f); // Medium gray

            // ── Pack icons row (for future expansion) ──────────────────────────────
            var iconContainerGo = new GameObject("IconContainer");
            iconContainerGo.transform.SetParent(contentGo.transform, false);
            _iconContainer = iconContainerGo.transform;
            RectTransform iconContainerRt = iconContainerGo.AddComponent<RectTransform>();
            iconContainerRt.sizeDelta = new Vector2(600f, 50f);

            var iconLayout = iconContainerGo.AddComponent<HorizontalLayoutGroup>();
            iconLayout.childForceExpandHeight = true;
            iconLayout.childForceExpandWidth = false;
            iconLayout.spacing = 10f;

            DebugLog.Write("ModLoadingOverlay", "[ModLoadingOverlay] UI built successfully.");
        }

        /// <summary>
        /// Updates the progress display with current pack loading status.
        /// Called by ModPlatform during the ContentLoader pack enumeration loop.
        /// </summary>
        public void SetPackProgress(int current, int total, string packName)
        {
            if (_progressText != null)
            {
                _progressText.text = $"Loading pack {current} of {total}: {packName}";
            }
        }

        /// <summary>
        /// Sets a custom message (e.g., "Verifying assets...", "Building catalog...").
        /// </summary>
        public void SetMessage(string msg)
        {
            if (_progressText != null)
            {
                _progressText.text = msg;
            }
        }

        /// <summary>
        /// Fades out and destroys the overlay after a short delay.
        /// Called when pack loading completes (e.g., in OnActiveSceneChanged).
        /// </summary>
        public void Hide()
        {
            if (_isFadingOut) return;
            _isFadingOut = true;
            _fadeOutElapsed = 0f;
        }

        private void Update()
        {
            if (!_isFadingOut) return;

            _fadeOutElapsed += Time.deltaTime;
            if (_canvasGroup != null)
            {
                _canvasGroup.alpha = Mathf.Lerp(1f, 0f, _fadeOutElapsed / _fadeOutDuration);
            }

            if (_fadeOutElapsed >= _fadeOutDuration)
            {
                Destroy(gameObject);
            }
        }
    }
}
