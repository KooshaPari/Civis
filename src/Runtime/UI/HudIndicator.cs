#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Small always-visible HUD strip rendered in the top-right corner when the mod menu is closed.
    /// Displays pack count and error state at a glance. Clicking opens the mod menu.
    /// Fades to 30% opacity after 5 seconds of inactivity and returns to full opacity on hover.
    /// Hosts a small toast notification area below it for transient messages.
    /// </summary>
    public class HudIndicator : MonoBehaviour
    {
        // ── Config ─────────────────────────────────────────────────────────────────

        private const float HudWidth = 180f;
        private const float HudHeight = 28f;
        private const float HudMarginRight = 10f;
        private const float HudMarginTop = 10f;
        private const float FadeDelay = 5f;
        private const float FadeAlpha = 0.3f;
        private const float ToastDuration = 3f;
        private const float ToastHeight = 22f;
        private const float ToastFadeTime = 0.5f;

        // ── State ──────────────────────────────────────────────────────────────────

        private IModMenuHost? _modMenu;
        private int _packCount;
        private int _errorCount;
        private float _lastActivityTime;
        private bool _hovered;

        private readonly Queue<ToastEntry> _toastQueue = new Queue<ToastEntry>();
        private readonly List<ToastEntry> _activeToasts = new List<ToastEntry>();

        // ── Public API ─────────────────────────────────────────────────────────────

        /// <summary>
        /// Sets the mod menu overlay reference so clicking the HUD strip can open it.
        /// </summary>
        /// <param name="menu">The overlay to open/close on click.</param>
        public void SetModMenu(IModMenuHost? menu)
        {
            _modMenu = menu;
        }

        /// <summary>
        /// Updates the displayed pack and error counts.
        /// </summary>
        /// <param name="packCount">Total number of loaded packs.</param>
        /// <param name="errorCount">Number of packs with errors.</param>
        public void UpdateCounts(int packCount, int errorCount)
        {
            _packCount = packCount;
            _errorCount = errorCount;
        }

        /// <summary>
        /// Enqueues a toast notification displayed below the HUD strip for <see cref="ToastDuration"/> seconds.
        /// </summary>
        /// <param name="message">The message to display.</param>
        public void ShowToast(string message)
        {
            _toastQueue.Enqueue(new ToastEntry(message, Time.realtimeSinceStartup + ToastDuration));
            _lastActivityTime = Time.realtimeSinceStartup;
        }

        // ── Unity lifecycle ────────────────────────────────────────────────────────

        private void Update()
        {
            // Drain queued toasts into the active list
            while (_toastQueue.Count > 0)
            {
                _activeToasts.Add(_toastQueue.Dequeue());
            }

            // Expire toasts
            _activeToasts.RemoveAll(t => Time.realtimeSinceStartup > t.ExpireTime + ToastFadeTime);
        }

        private void OnGUI()
        {
            // Hide when the full mod menu is open — it already shows all info
            if (_modMenu != null && _modMenu.IsVisible) return;

            float screenW = Screen.width;
            float x = screenW - HudWidth - HudMarginRight;
            float y = HudMarginTop;
            Rect hudRect = new Rect(x, y, HudWidth, HudHeight);

            // Determine opacity
            bool mouseOverHud = hudRect.Contains(Event.current.mousePosition);
            if (mouseOverHud)
            {
                _lastActivityTime = Time.realtimeSinceStartup;
                _hovered = true;
            }
            else
            {
                _hovered = false;
            }

            float timeSinceActivity = Time.realtimeSinceStartup - _lastActivityTime;
            float alpha = timeSinceActivity > FadeDelay && !_hovered
                ? FadeAlpha
                : 1f;

            Color saved = GUI.color;
            GUI.color = new Color(1f, 1f, 1f, alpha);

            DrawHudStrip(hudRect);

            // Draw toasts below the HUD strip
            float toastY = y + HudHeight + 4f;
            DrawToasts(x, toastY, alpha);

            GUI.color = saved;
        }

        // ── Drawing ────────────────────────────────────────────────────────────────

        private void DrawHudStrip(Rect rect)
        {
            // Background
            Color bgColor = _errorCount > 0
                ? new Color(DinoForgeStyle.Error.r, DinoForgeStyle.Error.g, DinoForgeStyle.Error.b, 0.88f)
                : new Color(DinoForgeStyle.Background.r, DinoForgeStyle.Background.g, DinoForgeStyle.Background.b, 0.92f);

            GUI.color = new Color(bgColor.r, bgColor.g, bgColor.b, bgColor.a * GUI.color.a);
            GUI.Box(rect, GUIContent.none);
            GUI.color = Color.white;

            // Label: "[DF] 3 packs ● 0 errors"
            string dot = _errorCount > 0 ? "●" : "●";
            Color dotColor = _errorCount > 0 ? DinoForgeStyle.Error : DinoForgeStyle.Success;
            string label = $"[DF]  {_packCount} packs";

            GUIStyle labelStyle = DinoForgeStyle.SectionLabelStyle;

            // Draw label
            Rect labelRect = new Rect(rect.x + 6f, rect.y, rect.width - 40f, rect.height);
            GUI.Label(labelRect, label, labelStyle);

            // Draw colored dot + error count
            Rect dotRect = new Rect(rect.x + rect.width - 52f, rect.y, 50f, rect.height);
            Color savedColor = GUI.color;
            GUI.color = new Color(dotColor.r, dotColor.g, dotColor.b, GUI.color.a);
            GUI.Label(dotRect, _errorCount > 0 ? $"● {_errorCount} err" : "● OK", labelStyle);
            GUI.color = savedColor;

            // Click to open mod menu
            if (Event.current.type == EventType.MouseUp && rect.Contains(Event.current.mousePosition))
            {
                _modMenu?.Toggle();
                _lastActivityTime = Time.realtimeSinceStartup;
                Event.current.Use();
            }
        }

        private void DrawToasts(float x, float startY, float parentAlpha)
        {
            float currentY = startY;

            for (int i = _activeToasts.Count - 1; i >= 0; i--)
            {
                ToastEntry toast = _activeToasts[i];
                float timeLeft = toast.ExpireTime - Time.realtimeSinceStartup;
                float toastAlpha = parentAlpha;

                // Fade out during the last ToastFadeTime seconds
                if (timeLeft < 0f)
                {
                    float fadeProgress = -timeLeft / ToastFadeTime; // 0..1
                    toastAlpha = parentAlpha * (1f - Mathf.Clamp01(fadeProgress));
                }

                if (toastAlpha < 0.02f) continue;

                Rect toastRect = new Rect(x, currentY, HudWidth, ToastHeight);

                Color savedColor = GUI.color;
                GUI.color = new Color(1f, 1f, 1f, toastAlpha);

                Color bgColor = new Color(DinoForgeStyle.PanelBackground.r, DinoForgeStyle.PanelBackground.g, DinoForgeStyle.PanelBackground.b, 0.92f);
                GUI.color = new Color(bgColor.r, bgColor.g, bgColor.b, bgColor.a * toastAlpha);
                GUI.Box(toastRect, GUIContent.none);
                GUI.color = new Color(DinoForgeStyle.Success.r, DinoForgeStyle.Success.g, DinoForgeStyle.Success.b, toastAlpha);
                GUI.Label(toastRect, $"  ✓  {toast.Message}", DinoForgeStyle.SectionLabelStyle);
                GUI.color = savedColor;

                currentY = currentY + ToastHeight + 2f;
            }
        }

        // ── Inner types ────────────────────────────────────────────────────────────

        private sealed class ToastEntry
        {
            /// <summary>Message text to display.</summary>
            public string Message { get; }

            /// <summary>Time (realtimeSinceStartup) when the toast expires.</summary>
            public float ExpireTime { get; }

            /// <summary>Creates a new toast entry.</summary>
            public ToastEntry(string message, float expireTime)
            {
                Message = message;
                ExpireTime = expireTime;
            }
        }
    }
}
