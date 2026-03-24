#nullable enable
using System;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Always-visible compact HUD strip anchored to the top-right corner.
    /// Shows pack count and error indicator. Click to open mod menu.
    /// Also hosts the toast notification area.
    /// </summary>
    public enum ToastType
    {
        Info,
        Success,
        Warning,
        Error
    }

    /// <summary>
    /// Always-visible compact HUD strip anchored to the top-right corner.
    /// 200x32 px strip showing [DF] pack count and status dot.
    /// Click opens mod menu; also hosts toast notifications.
    /// </summary>
    public class HudStrip : MonoBehaviour
    {
        /// <summary>Fired when the strip is clicked.</summary>
        public Action? OnClicked;

        // ── UI references ────────────────────────────────────────────────────────
        private CanvasGroup? _stripGroup;
        private Text? _labelText;
        private Text? _dotText;

        // Toast
        private GameObject? _toastGo;
        private CanvasGroup? _toastGroup;
        private Text? _toastText;
        private float _toastTimer;
        private const float ToastDuration = 3f;
        private const float ToastFadeTime = 0.4f;

        // Hover fade — strip is fully hidden when idle, fades in on hover
        private bool _hovered;
        private const float AlphaBase = 0f;
        private const float AlphaHover = 1.0f;
        private const float FadeSpeed = 6f;

        // State
        private int _packCount;
        private int _errorCount;

        // ── Bootstrap ────────────────────────────────────────────────────────────

        /// <summary>
        /// Builds the UGUI hierarchy for the HUD strip.
        /// Must be called from the main thread (Start/Awake).
        /// </summary>
        /// <param name="canvasRoot">The root Canvas transform to attach to.</param>
        public void Build(Transform canvasRoot)
        {
            // Strip container — top-right 200×32
            GameObject stripGo = UiBuilder.MakePanel(canvasRoot, "HudStrip", UiBuilder.BgDeep, new Vector2(200f, 32f));
            UiBuilder.AnchorTopRight(stripGo.GetComponent<RectTransform>(), new Vector2(200f, 32f), new Vector2(-8f, -8f));

            _stripGroup = UiBuilder.EnsureCanvasGroup(stripGo);
            _stripGroup.alpha = 0f; // hidden by default; fades in on hover

            // Horizontal layout
            UiBuilder.AddHorizontalLayout(stripGo, 6f, new RectOffset(8, 8, 4, 4));

            // "[DF]" label
            Text dfLabel = UiBuilder.MakeText(stripGo.transform, "DFLabel", "[DF]", 13, UiBuilder.Accent, bold: true);
            dfLabel.GetComponent<RectTransform>().sizeDelta = new Vector2(30f, 24f);
            LayoutElement dfLe = dfLabel.gameObject.AddComponent<LayoutElement>();
            dfLe.preferredWidth = 30f;

            // Pack count text
            _labelText = UiBuilder.MakeText(stripGo.transform, "CountLabel", "0 packs", 12, UiBuilder.TextPrimary);
            LayoutElement labelLe = _labelText.gameObject.AddComponent<LayoutElement>();
            labelLe.flexibleWidth = 1f;

            // Status dot
            _dotText = UiBuilder.MakeText(stripGo.transform, "StatusDot", "●", 14, UiBuilder.Success, bold: false, TextAnchor.MiddleRight);
            _dotText.GetComponent<RectTransform>().sizeDelta = new Vector2(16f, 24f);
            LayoutElement dotLe = _dotText.gameObject.AddComponent<LayoutElement>();
            dotLe.preferredWidth = 16f;

            // Click handler — add a transparent button covering the strip
            Button btn = stripGo.AddComponent<Button>();
            ColorBlock cb = btn.colors;
            cb.normalColor = Color.clear;
            cb.highlightedColor = Color.clear;
            cb.pressedColor = Color.clear;
            cb.selectedColor = Color.clear;
            btn.colors = cb;
            btn.targetGraphic = stripGo.GetComponent<Image>();
            btn.onClick.AddListener(() => OnClicked?.Invoke());

            // Toast panel — below the strip, 280×40
            _toastGo = UiBuilder.MakePanel(canvasRoot, "ToastPanel", UiBuilder.BgDeep, new Vector2(280f, 40f));
            RectTransform toastRt = _toastGo.GetComponent<RectTransform>();
            toastRt.anchorMin = Vector2.one;
            toastRt.anchorMax = Vector2.one;
            toastRt.pivot = Vector2.one;
            toastRt.anchoredPosition = new Vector2(-8f, -(8f + 32f + 6f)); // below strip

            _toastGroup = UiBuilder.EnsureCanvasGroup(_toastGo);
            _toastGroup.alpha = 0f;

            // Left accent bar on toast
            GameObject toastBar = UiBuilder.MakePanel(_toastGo.transform, "AccentBar", UiBuilder.Accent, new Vector2(4f, 0f));
            RectTransform barRt = toastBar.GetComponent<RectTransform>();
            barRt.anchorMin = new Vector2(0f, 0f);
            barRt.anchorMax = new Vector2(0f, 1f);
            barRt.pivot = new Vector2(0f, 0.5f);
            barRt.offsetMin = Vector2.zero;
            barRt.offsetMax = new Vector2(4f, 0f);
            barRt.anchoredPosition = Vector2.zero;

            _toastText = UiBuilder.MakeText(_toastGo.transform, "ToastText", "", 12, UiBuilder.TextPrimary);
            RectTransform txtRt = _toastText.GetComponent<RectTransform>();
            UiBuilder.FillParent(txtRt);
            txtRt.offsetMin = new Vector2(10f, 2f);
            txtRt.offsetMax = new Vector2(-4f, -2f);
            _toastText.horizontalOverflow = HorizontalWrapMode.Wrap;
        }

        // ── Public API ────────────────────────────────────────────────────────────

        /// <summary>Updates the displayed pack and error counts.</summary>
        public void SetStatus(int packCount, int errorCount)
        {
            _packCount = packCount;
            _errorCount = errorCount;
            RefreshLabel();
        }

        /// <summary>Displays a temporary toast notification below the HUD strip.</summary>
        public void ShowToast(string message, ToastType type)
        {
            if (_toastText == null || _toastGo == null) return;

            _toastText.text = message;
            _toastTimer = ToastDuration;

            // Recolor left accent bar
            Image? barImg = _toastGo.transform.Find("AccentBar")?.GetComponent<Image>();
            if (barImg != null)
            {
                barImg.color = type switch
                {
                    ToastType.Success => UiBuilder.Success,
                    ToastType.Warning => UiBuilder.Warning,
                    ToastType.Error => UiBuilder.Error,
                    _ => UiBuilder.Accent,
                };
            }

            if (_toastGroup != null) _toastGroup.alpha = 1f;
        }

        // ── MonoBehaviour ─────────────────────────────────────────────────────────

        private void Update()
        {
            AnimateHover();
            TickToast();
        }

        // ── Private ───────────────────────────────────────────────────────────────

        private void RefreshLabel()
        {
            if (_labelText != null)
                _labelText.text = $"{_packCount} packs";

            if (_dotText != null)
            {
                if (_errorCount > 0)
                {
                    _dotText.text = $"● {_errorCount}";
                    _dotText.color = UiBuilder.Error;
                }
                else
                {
                    _dotText.text = "●";
                    _dotText.color = UiBuilder.Success;
                }
            }
        }

        private void AnimateHover()
        {
            // Detect pointer over strip (simple mouse position check; no EventSystem needed)
            // This is a lightweight alternative to IPointerEnterHandler which requires EventSystem.
            // We simply check if the mouse is inside the strip's screen rect.
            if (_stripGroup == null) return;

            float targetAlpha = _hovered ? AlphaHover : AlphaBase;
            _stripGroup.alpha = Mathf.MoveTowards(_stripGroup.alpha, targetAlpha, FadeSpeed * Time.deltaTime);
        }

        private void TickToast()
        {
            if (_toastGroup == null || _toastTimer <= 0f) return;

            _toastTimer -= Time.deltaTime;

            if (_toastTimer <= ToastFadeTime)
            {
                _toastGroup.alpha = Mathf.Max(0f, _toastTimer / ToastFadeTime);
            }
        }

        /// <summary>Called by DFCanvas mouse-over detection.</summary>
        internal void SetHovered(bool hovered)
        {
            _hovered = hovered;
        }
    }
}
