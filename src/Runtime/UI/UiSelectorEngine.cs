#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;
using TMPro;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Live UI selector/query engine used by the bridge.
    /// Supports constrained selector grammar: id=, name=, role=, path=, component=, label=, text= with &amp;&amp; conjunction.
    /// Supports disambiguation: index=N, first, last.
    /// </summary>
    internal static class UiSelectorEngine
    {
        public static UiActionResult Query(string selector)
        {
            var (matches, disambiguation) = FindMatches(selector);
            var normalized = NormalizeSelector(selector);

            if (matches.Count == 0)
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = $"No UI nodes matched selector '{selector}'.",
                    MatchCount = 0,
                    Actionable = false,
                    ActionabilityReason = "not-found"
                };
            }

            var (index, useFirst, useLast) = disambiguation;
            Transform? target = SelectByDisambiguation(matches, index, useFirst, useLast);

            if (target == null)
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = $"Selector matched {matches.Count} nodes but disambiguation failed.",
                    MatchCount = matches.Count,
                    Actionable = false,
                    ActionabilityReason = "disambiguation-failed"
                };
            }

            var node = UiTreeSnapshotBuilder.BuildNodeSnapshot(target, "root");
            var actionable = IsActionable(target, out string reason);

            return new UiActionResult
            {
                Success = true,
                Selector = selector,
                Message = $"Matched {matches.Count} UI node(s), selected index {matches.IndexOf(target)}.",
                MatchedNode = node,
                MatchCount = matches.Count,
                Actionable = actionable,
                ActionabilityReason = actionable ? string.Empty : reason
            };
        }

        public static UiActionResult Click(string selector)
        {
            var (matches, disambiguation) = FindMatches(selector);

            if (matches.Count == 0)
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = $"No UI nodes matched selector '{selector}'.",
                    MatchCount = 0,
                    Actionable = false,
                    ActionabilityReason = "not-found"
                };
            }

            var (index, useFirst, useLast) = disambiguation;
            Transform? target = SelectByDisambiguation(matches, index, useFirst, useLast);

            if (target == null)
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = $"Selector matched {matches.Count} nodes but disambiguation failed.",
                    MatchCount = matches.Count,
                    Actionable = false,
                    ActionabilityReason = "disambiguation-failed"
                };
            }

            var node = UiTreeSnapshotBuilder.BuildNodeSnapshot(target, "root");

            if (!IsActionable(target, out string reason))
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = $"Matched UI node is not actionable: {reason}",
                    MatchedNode = node,
                    MatchCount = matches.Count,
                    Actionable = false,
                    ActionabilityReason = reason
                };
            }

            // Check occlusion
            if (IsOccluded(target, out string occluder))
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = $"UI node is occluded by '{occluder}'.",
                    MatchedNode = node,
                    MatchCount = matches.Count,
                    Actionable = false,
                    ActionabilityReason = $"occluded-by:{occluder}"
                };
            }

            // Click via Button or direct pointer
            Button? button = target.GetComponent<Button>();
            if (button != null)
            {
                button.onClick.Invoke();
                ExecutePointerClick(button.gameObject);
            }
            else
            {
                ExecutePointerClick(target.gameObject);
            }

            return new UiActionResult
            {
                Success = true,
                Selector = selector,
                Message = $"Clicked UI node '{target.name}'.",
                MatchedNode = node,
                MatchCount = matches.Count,
                Actionable = true,
                ActionabilityReason = string.Empty
            };
        }

        public static UiWaitResult EvaluateState(string selector, string? state)
        {
            string normalizedState = string.IsNullOrWhiteSpace(state) ? "visible" : state!.Trim().ToLowerInvariant();
            var (matches, disambiguation) = FindMatches(selector);

            if (matches.Count == 0)
            {
                return new UiWaitResult
                {
                    Ready = false,
                    Selector = selector,
                    State = normalizedState,
                    Message = $"No UI nodes matched selector '{selector}'.",
                    MatchCount = 0
                };
            }

            var (index, useFirst, useLast) = disambiguation;
            Transform? target = SelectByDisambiguation(matches, index, useFirst, useLast);

            if (target == null)
            {
                return new UiWaitResult
                {
                    Ready = false,
                    Selector = selector,
                    State = normalizedState,
                    Message = $"Matched {matches.Count} nodes but disambiguation failed.",
                    MatchCount = matches.Count
                };
            }

            var node = UiTreeSnapshotBuilder.BuildNodeSnapshot(target, "root");
            bool ready = EvaluateCondition(target, normalizedState, out string message);

            return new UiWaitResult
            {
                Ready = ready,
                Selector = selector,
                State = normalizedState,
                Message = message,
                MatchedNode = node,
                MatchCount = matches.Count
            };
        }

        public static UiExpectationResult Expect(string selector, string condition)
        {
            string normalizedCondition = string.IsNullOrWhiteSpace(condition) ? "visible" : condition.Trim().ToLowerInvariant();
            var (matches, disambiguation) = FindMatches(selector);

            if (matches.Count == 0)
            {
                return new UiExpectationResult
                {
                    Success = false,
                    Selector = selector,
                    Condition = normalizedCondition,
                    Message = $"No UI nodes matched selector '{selector}'.",
                    MatchCount = 0
                };
            }

            var (index, useFirst, useLast) = disambiguation;
            Transform? target = SelectByDisambiguation(matches, index, useFirst, useLast);

            if (target == null)
            {
                return new UiExpectationResult
                {
                    Success = false,
                    Selector = selector,
                    Condition = normalizedCondition,
                    Message = $"Matched {matches.Count} nodes but disambiguation failed.",
                    MatchCount = matches.Count
                };
            }

            var node = UiTreeSnapshotBuilder.BuildNodeSnapshot(target, "root");
            bool success = EvaluateCondition(target, normalizedCondition, out string message);

            return new UiExpectationResult
            {
                Success = success,
                Selector = selector,
                Condition = normalizedCondition,
                Message = message,
                MatchedNode = node,
                MatchCount = matches.Count
            };
        }

        /// <summary>
        /// Resolves a selector to a single target Transform using the same grammar and
        /// disambiguation rules as <see cref="Query"/>/<see cref="Click"/>. Returns null
        /// (with <paramref name="matchCount"/> set) when nothing matches or disambiguation fails.
        /// Used by <see cref="EventSystemDriver"/> to drive the full pointer lifecycle.
        /// Must be called on the Unity main thread.
        /// </summary>
        public static Transform? ResolveTarget(string selector, out int matchCount)
        {
            var (matches, disambiguation) = FindMatches(selector);
            matchCount = matches.Count;
            if (matches.Count == 0)
            {
                return null;
            }

            var (index, useFirst, useLast) = disambiguation;
            return SelectByDisambiguation(matches, index, useFirst, useLast);
        }

        /// <summary>Public actionability check for the pointer driver (main thread only).</summary>
        public static bool IsTargetActionable(Transform target, out string reason) => IsActionable(target, out reason);

        /// <summary>Builds a snapshot node for the supplied transform (main thread only).</summary>
        public static UiNode SnapshotOf(Transform target) => UiTreeSnapshotBuilder.BuildNodeSnapshot(target, "root");

        private static (List<Transform> matches, (int? index, bool useFirst, bool useLast) disambiguation) FindMatches(string selector)
        {
            var normalized = NormalizeSelector(selector);
            int? index = null;
            bool useFirst = false;
            bool useLast = false;

            // Check for disambiguation
            if (normalized.StartsWith("index="))
            {
                var parts = normalized.Split(new[] { "&&" }, StringSplitOptions.RemoveEmptyEntries);
                foreach (var part in parts)
                {
                    if (part.StartsWith("index=") && int.TryParse(part.Substring(6), out int idx))
                    {
                        index = idx;
                    }
                }
            }
            else if (normalized.Contains("&&first"))
            {
                useFirst = true;
            }
            else if (normalized.Contains("&&last"))
            {
                useLast = true;
            }

            var matches = new List<Transform>();
            var canvases = Resources.FindObjectsOfTypeAll<Canvas>();

            foreach (var canvas in canvases)
            {
                if (canvas == null || !canvas.gameObject.activeInHierarchy)
                {
                    continue;
                }

                Traverse(canvas.transform, "root", normalized, matches);
            }

            return (matches, (index, useFirst, useLast));
        }

        private static string NormalizeSelector(string selector)
        {
            return (selector ?? string.Empty).Trim();
        }

        private static Transform? SelectByDisambiguation(List<Transform> matches, int? index, bool useFirst, bool useLast)
        {
            if (index.HasValue && index.Value >= 0 && index.Value < matches.Count)
            {
                return matches[index.Value];
            }

            if (useFirst && matches.Count > 0)
            {
                return matches[0];
            }

            if (useLast && matches.Count > 0)
            {
                return matches[matches.Count - 1];
            }

            // Default: first match
            return matches.Count > 0 ? matches[0] : null;
        }

        private static void Traverse(Transform transform, string parentPath, string selector, List<Transform> matches)
        {
            string segment = $"{transform.name}[{transform.GetSiblingIndex()}]";
            string path = $"{parentPath}/{segment}";

            if (Matches(transform, path, selector))
            {
                matches.Add(transform);
            }

            for (int i = 0; i < transform.childCount; i++)
            {
                var child = transform.GetChild(i);
                if (!child.gameObject.activeInHierarchy)
                {
                    continue;
                }

                Traverse(child, path, selector, matches);
            }
        }

        private static bool Matches(Transform transform, string path, string selector)
        {
            if (string.IsNullOrWhiteSpace(selector))
            {
                return true;
            }

            string lower = selector.ToLowerInvariant();
            string label = GetLabel(transform);

            // Parse conjunctions
            var parts = selector.Split(new[] { "&&" }, StringSplitOptions.RemoveEmptyEntries);
            foreach (var part in parts)
            {
                var p = part.Trim();
                var pLower = p.ToLowerInvariant();

                if (pLower.StartsWith("id="))
                {
                    var idValue = p.Substring(3);
                    var nodeId = MakeId(path);
                    if (nodeId.IndexOf(idValue, StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("name="))
                {
                    if (!string.Equals(transform.name, p.Substring(5), StringComparison.OrdinalIgnoreCase))
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("role="))
                {
                    var role = InferRole(transform);
                    if (role.IndexOf(p.Substring(5), StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("path="))
                {
                    if (path.IndexOf(p.Substring(5), StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("component="))
                {
                    if (GetPrimaryComponentType(transform).IndexOf(p.Substring(10), StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("label=") || pLower.StartsWith("text="))
                {
                    if (label.IndexOf(p.Substring(p.IndexOf('=') + 1), StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("text-exact="))
                {
                    if (!string.Equals(label, p.Substring(11), StringComparison.OrdinalIgnoreCase))
                    {
                        return false;
                    }
                }
                else if (pLower.StartsWith("count="))
                {
                    // Handled at query level, not per-node
                    continue;
                }
                else
                {
                    // Fallback: match against name, label, or path
                    if (transform.name.IndexOf(p, StringComparison.OrdinalIgnoreCase) < 0 &&
                        label.IndexOf(p, StringComparison.OrdinalIgnoreCase) < 0 &&
                        path.IndexOf(p, StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        return false;
                    }
                }
            }

            return true;
        }

        private static string InferRole(Transform transform)
        {
            if (transform.GetComponent<Canvas>() != null) return "canvas";
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
            if (transform.GetComponent<Image>() != null) return "image";
            if (transform.GetComponent<Selectable>() != null) return "selectable";
            if (transform.GetComponent<RectTransform>() != null) return "panel";
            return "node";
        }

        private static string GetPrimaryComponentType(Transform transform)
        {
            if (transform.GetComponent<Canvas>() != null) return "Canvas";
            if (transform.GetComponent<ScrollRect>() != null) return "ScrollRect";
            if (transform.GetComponent<Button>() != null) return transform.GetComponent<Button>().GetType().Name;
            if (transform.GetComponent<Selectable>() != null) return transform.GetComponent<Selectable>().GetType().Name;
            if (transform.GetComponent<Graphic>() != null) return transform.GetComponent<Graphic>().GetType().Name;
            return transform.GetType().Name;
        }

        private static string GetLabel(Transform transform)
        {
            var text = transform.GetComponent<Text>();
            if (text != null && !string.IsNullOrWhiteSpace(text.text))
            {
                return text.text.Trim();
            }

            var tmpText = transform.GetComponent<TMP_Text>();
            if (tmpText != null && !string.IsNullOrWhiteSpace(tmpText.text))
            {
                return tmpText.text.Trim();
            }

            var button = transform.GetComponent<Button>();
            if (button != null)
            {
                return NativeUiHelper.GetButtonText(button).Trim();
            }

            var childText = transform.GetComponentInChildren<Text>(includeInactive: false);
            if (childText != null && !string.IsNullOrWhiteSpace(childText.text))
            {
                return childText.text.Trim();
            }

            var childTmpText = transform.GetComponentInChildren<TMP_Text>(includeInactive: false);
            if (childTmpText != null && !string.IsNullOrWhiteSpace(childTmpText.text))
            {
                return childTmpText.text.Trim();
            }

            return string.Empty;
        }

        private static string MakeId(string path)
        {
            return path.Replace('/', '_').Replace('[', '_').Replace(']', '_').Replace(' ', '_').ToLowerInvariant();
        }

        private static bool IsActionable(Transform transform, out string reason)
        {
            reason = string.Empty;

            if (!transform.gameObject.activeInHierarchy)
            {
                reason = "inactive";
                return false;
            }

            var selectable = transform.GetComponent<Selectable>();
            if (selectable != null && !selectable.IsInteractable())
            {
                reason = "not-interactable";
                return false;
            }

            var canvasGroups = transform.GetComponentsInParent<CanvasGroup>(includeInactive: true);
            foreach (var cg in canvasGroups)
            {
                if (!cg.enabled) continue;
                if (cg.alpha <= 0.01f)
                {
                    reason = "hidden";
                    return false;
                }
                if (!cg.interactable)
                {
                    reason = "canvas-not-interactable";
                    return false;
                }
            }

            reason = string.Empty;
            return true;
        }

        private static bool IsOccluded(Transform transform, out string occluder)
        {
            occluder = string.Empty;

            var rectTransform = transform as RectTransform;
            if (rectTransform == null)
            {
                return false;
            }

            var eventSystem = EventSystem.current;
            if (eventSystem == null)
            {
                return false;
            }

            // Get center of target
            Vector3[] corners = new Vector3[4];
            rectTransform.GetWorldCorners(corners);
            Vector2 screenCenter = RectTransformUtility.WorldToScreenPoint(null, (corners[0] + corners[2]) / 2f);

            var pointerData = new PointerEventData(eventSystem)
            {
                position = screenCenter
            };

            var raycastResults = new List<RaycastResult>();
            eventSystem.RaycastAll(pointerData, raycastResults);

            // Check if target is the first hit
            if (raycastResults.Count > 0 && raycastResults[0].gameObject == transform.gameObject)
            {
                return false;
            }

            // Find first occluder
            foreach (var result in raycastResults)
            {
                if (result.gameObject != transform.gameObject)
                {
                    occluder = result.gameObject.name;
                    return true;
                }
            }

            return false;
        }

        private static bool IsNodeVisible(Transform transform)
        {
            if (!transform.gameObject.activeInHierarchy)
            {
                return false;
            }

            Graphic? graphic = transform.GetComponent<Graphic>();
            if (graphic != null && (!graphic.enabled || graphic.canvasRenderer.cull))
            {
                return false;
            }

            CanvasGroup[] canvasGroups = transform.GetComponents<CanvasGroup>();
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

        private static bool EvaluateCondition(Transform transform, string condition, out string message)
        {
            switch (condition)
            {
                case "exists":
                    message = "Selector exists.";
                    return true;

                case "visible":
                    if (IsNodeVisible(transform))
                    {
                        message = "Matched node is visible.";
                        return true;
                    }

                    message = transform.gameObject.activeInHierarchy
                        ? "Matched node is hidden (CanvasGroup alpha 0)."
                        : "Matched node is inactive.";
                    return false;

                case "hidden":
                    bool hidden = !IsNodeVisible(transform);
                    message = hidden ? "Matched node is hidden." : "Matched node is still visible.";
                    return hidden;

                case "interactable":
                case "actionable":
                    bool actionable = IsActionable(transform, out string reason);
                    message = actionable ? "Matched node is actionable." : $"Matched node is not actionable: {reason}";
                    return actionable;

                default:
                    // Check for text/label conditions
                    if (condition.StartsWith("text=") || condition.StartsWith("label="))
                    {
                        var expectedText = condition.Substring(condition.IndexOf('=') + 1);
                        var label = GetLabel(transform);
                        bool matches = label.IndexOf(expectedText, StringComparison.OrdinalIgnoreCase) >= 0;
                        message = matches ? $"Text matches: '{label}'." : $"Text mismatch: expected '{expectedText}', got '{label}'.";
                        return matches;
                    }

                    if (condition.StartsWith("text-exact="))
                    {
                        var expectedText = condition.Substring(11);
                        var label = GetLabel(transform);
                        bool matches = string.Equals(label, expectedText, StringComparison.OrdinalIgnoreCase);
                        message = matches ? $"Text exact match: '{label}'." : $"Text mismatch: expected '{expectedText}', got '{label}'.";
                        return matches;
                    }

                    message = $"Unknown condition '{condition}'.";
                    return false;
            }
        }

        private static void ExecutePointerClick(GameObject target)
        {
            var eventSystem = EventSystem.current;
            if (eventSystem == null)
            {
                return;
            }

            var eventData = new PointerEventData(eventSystem);
            ExecuteEvents.Execute(target, eventData, ExecuteEvents.pointerClickHandler);
        }
    }
}
