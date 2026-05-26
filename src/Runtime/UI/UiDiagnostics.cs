#nullable enable
using System.Collections.Generic;
using System.Text;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Runtime diagnostics for UI debugging.
    /// Provides methods to capture and log UI state for troubleshooting.
    /// </summary>
    public static class UiDiagnostics
    {
        /// <summary>
        /// Captures diagnostic info about all canvases in the scene.
        /// </summary>
        public static string CaptureCanvasState()
        {
            var sb = new StringBuilder(1024);  // Capacity ~= canvas count × 6 fields × 30 chars
            sb.AppendLine("=== Canvas State ===");

            Canvas[] canvases = Resources.FindObjectsOfTypeAll<Canvas>();
            sb.AppendLine($"Total canvases: {canvases.Length}");

            foreach (Canvas canvas in canvases)
            {
                sb.AppendLine($"\nCanvas: {canvas.name}");
                sb.AppendLine($"  renderMode: {canvas.renderMode}");
                sb.AppendLine($"  sortingOrder: {canvas.sortingOrder}");
                sb.AppendLine($"  worldCamera: {(canvas.worldCamera != null ? canvas.worldCamera.name : "null")}");
                sb.AppendLine($"  activeInHierarchy: {canvas.gameObject.activeInHierarchy}");

                // Check GraphicRaycaster
                var raycaster = canvas.GetComponent<UnityEngine.UI.GraphicRaycaster>();
                if (raycaster != null)
                {
                    sb.AppendLine($"  GraphicRaycaster: enabled={raycaster.enabled}, ignoreReversedGraphics={raycaster.ignoreReversedGraphics}");
                }
            }

            return sb.ToString();
        }

        /// <summary>
        /// Captures diagnostic info about a specific ScrollRect.
        /// </summary>
        public static string CaptureScrollRectState(ScrollRect scrollRect, string name)
        {
            var sb = new StringBuilder(512);  // Capacity ~= 13 appends × 40 chars
            sb.AppendLine($"=== ScrollRect: {name} ===");

            sb.AppendLine($"enabled: {scrollRect.enabled}");
            sb.AppendLine($"vertical: {scrollRect.vertical}");
            sb.AppendLine($"horizontal: {scrollRect.horizontal}");
            sb.AppendLine($"movementType: {scrollRect.movementType}");
            sb.AppendLine($"scrollSensitivity: {scrollRect.scrollSensitivity}");

            if (scrollRect.content != null)
            {
                sb.AppendLine($"\nContent: {scrollRect.content.name}");
                sb.AppendLine($"  activeInHierarchy: {scrollRect.content.gameObject.activeInHierarchy}");
                sb.AppendLine($"  sizeDelta: {scrollRect.content.sizeDelta}");
                sb.AppendLine($"  rect.height: {scrollRect.content.rect.height}");
                sb.AppendLine($"  childCount: {scrollRect.content.childCount}");

                // Check ContentSizeFitter
                var csf = scrollRect.content.GetComponent<ContentSizeFitter>();
                if (csf != null)
                {
                    sb.AppendLine($"  ContentSizeFitter: enabled={csf.enabled}, verticalFit={csf.verticalFit}, horizontalFit={csf.horizontalFit}");
                }

                // Check VerticalLayoutGroup
                var vlg = scrollRect.content.GetComponent<VerticalLayoutGroup>();
                if (vlg != null)
                {
                    sb.AppendLine($"  VerticalLayoutGroup: enabled={vlg.enabled}, spacing={vlg.spacing}, padding={vlg.padding}");
                }
            }
            else
            {
                sb.AppendLine("  content: NULL");
            }

            // Check viewport
            if (scrollRect.viewport != null)
            {
                sb.AppendLine($"\nViewport: {scrollRect.viewport.name}");
                sb.AppendLine($"  sizeDelta: {scrollRect.viewport.sizeDelta}");
            }

            return sb.ToString();
        }

        /// <summary>
        /// Logs all UI diagnostic info to console.
        /// </summary>
        public static void LogAllDiagnostics()
        {
            Debug.Log(CaptureCanvasState());

            // Check ModMenuPanel if present
            var modMenuPanel = Object.FindObjectOfType<ModMenuPanel>();
            if (modMenuPanel != null)
            {
                Debug.Log($"[UiDiagnostics] ModMenuPanel found, IsVisible={modMenuPanel.IsVisible}");
            }

            // Log font status
            Font? arialFont = Resources.GetBuiltinResource<Font>("Arial.ttf");
            Debug.Log($"[UiDiagnostics] Font loaded: '{(arialFont != null ? "Arial.ttf" : "NONE")}'");
        }

        /// <summary>
        /// Checks for common UI issues and returns a report.
        /// </summary>
        public static string CheckForIssues()
        {
            var issues = new List<string>();

            // Check for missing fonts
            Font? diagFont = Resources.GetBuiltinResource<Font>("Arial.ttf");
            if (diagFont == null)
            {
                issues.Add("CRITICAL: No font loaded - text will not render!");
            }

            // Check canvases
            Canvas[] canvases = Resources.FindObjectsOfTypeAll<Canvas>();
            bool hasActiveCanvas = false;
            foreach (Canvas canvas in canvases)
            {
                if (canvas.gameObject.activeInHierarchy && canvas.renderMode == RenderMode.ScreenSpaceOverlay)
                {
                    hasActiveCanvas = true;
                    var raycaster = canvas.GetComponent<UnityEngine.UI.GraphicRaycaster>();
                    if (raycaster != null && !raycaster.enabled)
                    {
                        issues.Add($"Canvas '{canvas.name}' has disabled GraphicRaycaster");
                    }
                }
            }
            if (!hasActiveCanvas)
            {
                issues.Add("No active ScreenSpaceOverlay canvas found");
            }

            // Check EventSystem
            var eventSystem = Object.FindObjectOfType<UnityEngine.EventSystems.EventSystem>();
            if (eventSystem == null)
            {
                issues.Add("No EventSystem found in scene");
            }
            else if (!eventSystem.enabled)
            {
                issues.Add("EventSystem is disabled");
            }

            if (issues.Count == 0)
            {
                return "No UI issues detected.";
            }

            return "Issues found:\n- " + string.Join("\n- ", issues);
        }
    }
}
