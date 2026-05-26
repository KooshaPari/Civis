#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Builds a live snapshot of the active Unity UI hierarchy for bridge consumers.
    /// </summary>
    internal static class UiTreeSnapshotBuilder
    {
        /// <summary>
        /// Captures a live snapshot of all active root canvases and their descendants.
        /// </summary>
        /// <param name="selector">Optional selector string echoed back in the result.</param>
        public static UiTreeResult Capture(string? selector)
        {
            UiNode root = new UiNode
            {
                Id = "root",
                Path = "root",
                Name = "root",
                Label = "Unity UI",
                Role = "root",
                ComponentType = "Root",
                Active = true,
                Visible = true,
                Interactable = false,
                RaycastTarget = false
            };

            HashSet<int> includedCanvasIds = new HashSet<int>();
            Canvas[] canvases = Resources.FindObjectsOfTypeAll<Canvas>();
            foreach (Canvas canvas in canvases)
            {
                if (canvas == null || !canvas.gameObject.activeInHierarchy)
                {
                    continue;
                }

                if (canvas.transform.parent != null && canvas.transform.parent.GetComponentInParent<Canvas>() != null)
                {
                    continue;
                }

                if (!includedCanvasIds.Add(canvas.gameObject.GetInstanceID()))
                {
                    continue;
                }

                root.Children.Add(BuildNodeSnapshot(canvas.transform, root.Path));
            }

            return new UiTreeResult
            {
                Success = true,
                Message = root.Children.Count > 0
                    ? $"Captured live Unity UI tree from {root.Children.Count} active root canvas/canvases."
                    : "No active Unity canvases found.",
                Selector = selector,
                GeneratedAtUtc = DateTime.UtcNow.ToString("O"),
                NodeCount = CountNodes(root),
                Root = root
            };
        }

        internal static UiNode BuildNodeSnapshot(Transform transform, string parentPath)
        {
            string segment = $"{transform.name}[{transform.GetSiblingIndex()}]";
            string path = $"{parentPath}/{segment}";
            RectTransform? rectTransform = transform as RectTransform;
            Selectable? selectable = transform.GetComponent<Selectable>();
            Graphic? graphic = transform.GetComponent<Graphic>();
            Canvas? canvas = transform.GetComponent<Canvas>();
            CanvasGroup[] canvasGroups = transform.GetComponents<CanvasGroup>();

            bool interactable = selectable?.IsInteractable() ?? AreCanvasGroupsInteractable(canvasGroups);
            bool raycastTarget = graphic?.raycastTarget ?? false;

            UiNode node = new UiNode
            {
                Id = MakeId(path),
                Path = path,
                Name = transform.name,
                Label = GetLabel(transform),
                Role = InferRole(transform, selectable, graphic, canvas),
                ComponentType = GetPrimaryComponentType(transform, selectable, graphic, canvas),
                Active = transform.gameObject.activeInHierarchy,
                Visible = IsVisible(transform, graphic, canvasGroups),
                Interactable = interactable,
                RaycastTarget = raycastTarget,
                Bounds = GetBounds(rectTransform)
            };

            Button? button = transform.GetComponent<Button>();
            if (button != null)
            {
                node.Style = CaptureButtonStyle(button);
            }

            for (int i = 0; i < transform.childCount; i++)
            {
                Transform child = transform.GetChild(i);
                if (!child.gameObject.activeInHierarchy)
                {
                    continue;
                }

                node.Children.Add(BuildNodeSnapshot(child, path));
            }

            return node;
        }

        private static string InferRole(Transform transform, Selectable? selectable, Graphic? graphic, Canvas? canvas)
        {
            if (canvas != null) return "canvas";
            if (transform.GetComponent<ScrollRect>() != null) return "scroll-view";
            if (transform.GetComponent<Button>() != null) return "button";
            if (transform.GetComponent<Toggle>() != null) return "toggle";
            if (transform.GetComponent<Scrollbar>() != null) return "scrollbar";
            if (transform.GetComponent<InputField>() != null) return "input";
            if (transform.GetComponent<TMP_InputField>() != null) return "input";
            if (transform.GetComponent<Slider>() != null) return "slider";
            if (transform.GetComponent<Mask>() != null) return "mask";
            if (transform.GetComponent<LayoutGroup>() != null) return "layout";
            if (transform.GetComponent<Text>() != null || transform.GetComponent<TMP_Text>() != null) return "text";
            if (graphic is Image) return "image";
            if (selectable != null) return "selectable";
            if (transform.GetComponent<RectTransform>() != null) return "panel";
            return "node";
        }

        private static string GetPrimaryComponentType(Transform transform, Selectable? selectable, Graphic? graphic, Canvas? canvas)
        {
            if (canvas != null) return nameof(Canvas);
            if (transform.GetComponent<ScrollRect>() != null) return nameof(ScrollRect);
            if (selectable != null) return selectable.GetType().Name;
            if (graphic != null) return graphic.GetType().Name;
            return transform.GetType().Name;
        }

        private static string GetLabel(Transform transform)
        {
            Text? text = transform.GetComponent<Text>();
            if (text != null && !string.IsNullOrWhiteSpace(text.text))
            {
                return text.text.Trim();
            }

            TMP_Text? tmpText = transform.GetComponent<TMP_Text>();
            if (tmpText != null && !string.IsNullOrWhiteSpace(tmpText.text))
            {
                return tmpText.text.Trim();
            }

            if (transform.GetComponent<Button>() != null)
            {
                return NativeUiHelper.GetButtonText(transform.GetComponent<Button>()).Trim();
            }

            Text? childText = transform.GetComponentInChildren<Text>(includeInactive: false);
            if (childText != null && !string.IsNullOrWhiteSpace(childText.text))
            {
                return childText.text.Trim();
            }

            TMP_Text? childTmpText = transform.GetComponentInChildren<TMP_Text>(includeInactive: false);
            if (childTmpText != null && !string.IsNullOrWhiteSpace(childTmpText.text))
            {
                return childTmpText.text.Trim();
            }

            return string.Empty;
        }

        private static bool IsVisible(Transform transform, Graphic? graphic, CanvasGroup[] canvasGroups)
        {
            if (!transform.gameObject.activeInHierarchy)
            {
                return false;
            }

            if (graphic != null && (!graphic.enabled || graphic.canvasRenderer.cull))
            {
                return false;
            }

            foreach (CanvasGroup canvasGroup in canvasGroups)
            {
                if (!canvasGroup.enabled)
                {
                    continue;
                }

                if (canvasGroup.alpha <= 0.01f)
                {
                    return false;
                }
            }

            return true;
        }

        private static bool AreCanvasGroupsInteractable(CanvasGroup[] canvasGroups)
        {
            foreach (CanvasGroup canvasGroup in canvasGroups)
            {
                if (!canvasGroup.enabled)
                {
                    continue;
                }

                if (!canvasGroup.interactable)
                {
                    return false;
                }
            }

            return false;
        }

        private static UiBounds? GetBounds(RectTransform? rectTransform)
        {
            if (rectTransform == null)
            {
                return null;
            }

            Vector3[] corners = new Vector3[4];
            rectTransform.GetWorldCorners(corners);

            float minX = corners[0].x;
            float minY = corners[0].y;
            float maxX = corners[2].x;
            float maxY = corners[2].y;

            return new UiBounds
            {
                X = minX,
                Y = minY,
                Width = Mathf.Max(0f, maxX - minX),
                Height = Mathf.Max(0f, maxY - minY)
            };
        }

        private static int CountNodes(UiNode node)
        {
            int count = 1;
            foreach (UiNode child in node.Children)
            {
                count = count + CountNodes(child);
            }

            return count;
        }

        private static string MakeId(string path)
        {
            return path
                .Replace('/', '_')
                .Replace('[', '_')
                .Replace(']', '_')
                .Replace(' ', '_')
                .ToLowerInvariant();
        }

        private static UiStyleSnapshot CaptureButtonStyle(Button button)
        {
            ColorBlock colors = button.colors;
            UiStyleSnapshot style = new UiStyleSnapshot
            {
                Transition = button.transition.ToString(),
                NormalColor = ColorToHex(colors.normalColor),
                HighlightedColor = ColorToHex(colors.highlightedColor),
                PressedColor = ColorToHex(colors.pressedColor),
                DisabledColor = ColorToHex(colors.disabledColor)
            };

            Text? legacyText = button.GetComponentInChildren<Text>(includeInactive: true);
            if (legacyText != null)
            {
                style.FontSize = legacyText.fontSize;
                style.TextColor = ColorToHex(legacyText.color);
            }
            else
            {
                TMP_Text? tmpText = button.GetComponentInChildren<TMP_Text>(includeInactive: true);
                if (tmpText != null)
                {
                    style.FontSize = Mathf.RoundToInt(tmpText.fontSize);
                    style.TextColor = ColorToHex(tmpText.color);
                }
            }

            if (button.targetGraphic is Image image)
            {
                style.ImageColor = ColorToHex(image.color);
            }

            return style;
        }

        private static string ColorToHex(Color color)
        {
            return $"#{Mathf.RoundToInt(color.r * 255f):X2}{Mathf.RoundToInt(color.g * 255f):X2}{Mathf.RoundToInt(color.b * 255f):X2}{Mathf.RoundToInt(color.a * 255f):X2}";
        }
    }
}
