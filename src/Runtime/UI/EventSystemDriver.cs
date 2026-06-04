#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// In-process pointer driver for DINO's Unity uGUI EventSystem.
    ///
    /// WHY THIS EXISTS: synthetic OS input (SetCursorPos / SendInput / MCP game_input) is
    /// NOT delivered to DINO's EventSystem — confirmed against the native Options button,
    /// which shows no hover under synthetic OS input. Because DINOForge runs INSIDE the game
    /// via BepInEx, we bypass the OS entirely and drive Unity's EventSystem directly:
    /// resolve a target (by selector or screen coords via the active GraphicRaycasters),
    /// build a synthetic <see cref="PointerEventData"/>, and fire the proper pointer lifecycle
    /// (enter → down → up → click, plus exit) through <see cref="ExecuteEvents"/>. We also
    /// drive <see cref="Selectable"/> state (OnPointerEnter/Exit/Down/Up/Select) so DINO's
    /// MainMenuButton highlight driver paints hover/press visuals.
    ///
    /// THREADING: the EventSystem is main-thread-only. Every public method here assumes it is
    /// already running on the Unity main thread — callers route via
    /// <see cref="Bridge.MainThreadDispatcher.RunOnMainThread{T}"/>. (Pattern #235: every path
    /// guards on EventSystem.current != null.)
    /// </summary>
    internal static class EventSystemDriver
    {
        /// <summary>Recognized pointer events.</summary>
        public enum PointerEvent
        {
            Enter,
            Exit,
            Down,
            Up,
            Click,
            /// <summary>Enter only — leaves the cursor "hovering" the target.</summary>
            Hover,
            /// <summary>Full enter → down → up → click (the cursor stays entered).</summary>
            Press,
        }

        /// <summary>
        /// The transform currently "hovered" by the driver, so a subsequent enter on a new
        /// target can fire exit on the previous one (matching real pointer movement).
        /// </summary>
        private static Transform? _hovered;

        /// <summary>
        /// Drive a pointer event against a selector-resolved target.
        /// Main thread only.
        /// </summary>
        public static UiActionResult Drive(string selector, string eventName)
        {
            EventSystem? es = EventSystem.current;
            if (es == null)
            {
                return Fail(selector, "EventSystem.current is null (no active EventSystem).", "no-eventsystem");
            }

            if (!TryParseEvent(eventName, out PointerEvent evt))
            {
                return Fail(selector, $"Unknown pointer event '{eventName}'. Expected enter|exit|down|up|click|hover|press.", "bad-event");
            }

            Transform? target = UiSelectorEngine.ResolveTarget(selector, out int matchCount);
            if (target == null)
            {
                return new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = matchCount == 0
                        ? $"No UI nodes matched selector '{selector}'."
                        : $"Selector matched {matchCount} nodes but disambiguation failed.",
                    MatchCount = matchCount,
                    Actionable = false,
                    ActionabilityReason = matchCount == 0 ? "not-found" : "disambiguation-failed",
                };
            }

            return DriveTarget(es, target, evt, selector, matchCount);
        }

        /// <summary>
        /// Drive a pointer event at screen coordinates, resolving the target via the active
        /// GraphicRaycasters (this is what real pointer input would hit). Main thread only.
        /// </summary>
        public static UiActionResult DriveAt(float x, float y, string eventName)
        {
            EventSystem? es = EventSystem.current;
            if (es == null)
            {
                return Fail($"({x},{y})", "EventSystem.current is null (no active EventSystem).", "no-eventsystem");
            }

            if (!TryParseEvent(eventName, out PointerEvent evt))
            {
                return Fail($"({x},{y})", $"Unknown pointer event '{eventName}'.", "bad-event");
            }

            var ped = new PointerEventData(es) { position = new Vector2(x, y) };
            var hits = new List<RaycastResult>();
            es.RaycastAll(ped, hits);
            if (hits.Count == 0)
            {
                return Fail($"({x},{y})", $"No raycast hit at screen ({x},{y}).", "no-hit");
            }

            Transform target = hits[0].gameObject.transform;
            return DriveTarget(es, target, evt, $"({x},{y})", 1);
        }

        private static UiActionResult DriveTarget(EventSystem es, Transform target, PointerEvent evt, string selector, int matchCount)
        {
            GameObject go = target.gameObject;
            UiNode node = UiSelectorEngine.SnapshotOf(target);

            // Build a synthetic pointer event positioned over the target's center so any
            // handler that inspects ped.position behaves sanely.
            var ped = new PointerEventData(es)
            {
                button = PointerEventData.InputButton.Left,
                position = ScreenCenterOf(target),
                pointerEnter = go,
                pointerPress = go,
                pointerCurrentRaycast = MakeRaycast(go),
                pointerPressRaycast = MakeRaycast(go),
            };

            bool fired;
            switch (evt)
            {
                case PointerEvent.Enter:
                case PointerEvent.Hover:
                    fired = FireEnter(go, ped);
                    break;
                case PointerEvent.Exit:
                    fired = FireExit(go, ped);
                    break;
                case PointerEvent.Down:
                    fired = FireDown(go, ped);
                    break;
                case PointerEvent.Up:
                    fired = FireUp(go, ped);
                    break;
                case PointerEvent.Click:
                    fired = FireClick(go, ped);
                    break;
                case PointerEvent.Press:
                    // Full lifecycle: enter (hover paint) → down (press paint) → up → click.
                    fired = FireEnter(go, ped);
                    fired |= FireDown(go, ped);
                    fired |= FireUp(go, ped);
                    fired |= FireClick(go, ped);
                    break;
                default:
                    fired = false;
                    break;
            }

            UiSelectorEngine.IsTargetActionable(target, out string reason);
            return new UiActionResult
            {
                Success = fired,
                Selector = selector,
                Message = fired
                    ? $"Drove pointer '{evt}' on '{go.name}'."
                    : $"Pointer '{evt}' on '{go.name}' had no handler/effect.",
                MatchedNode = node,
                MatchCount = matchCount,
                Actionable = string.IsNullOrEmpty(reason),
                ActionabilityReason = reason,
            };
        }

        // ── Lifecycle primitives ───────────────────────────────────────────

        private static bool FireEnter(GameObject go, PointerEventData ped)
        {
            // Exit any previously-hovered target first (mirrors real pointer movement).
            if (_hovered != null && _hovered.gameObject != go && _hovered.gameObject != null)
            {
                ExecuteEvents.ExecuteHierarchy(_hovered.gameObject, ped, ExecuteEvents.pointerExitHandler);
            }

            ped.pointerEnter = go;
            bool h = ExecuteEvents.ExecuteHierarchy(go, ped, ExecuteEvents.pointerEnterHandler) != null;

            // Drive Selectable highlight state directly (covers MainMenuButton-style drivers
            // that react to OnPointerEnter without an explicit IPointerEnterHandler relay).
            Selectable? sel = go.GetComponentInParent<Selectable>();
            if (sel != null)
            {
                sel.OnPointerEnter(ped);
            }

            _hovered = go.transform;
            return h || sel != null;
        }

        private static bool FireExit(GameObject go, PointerEventData ped)
        {
            bool h = ExecuteEvents.ExecuteHierarchy(go, ped, ExecuteEvents.pointerExitHandler) != null;
            Selectable? sel = go.GetComponentInParent<Selectable>();
            if (sel != null)
            {
                sel.OnPointerExit(ped);
            }
            if (_hovered != null && _hovered.gameObject == go)
            {
                _hovered = null;
            }
            return h || sel != null;
        }

        private static bool FireDown(GameObject go, PointerEventData ped)
        {
            ped.pointerPress = ExecuteEvents.GetEventHandler<IPointerDownHandler>(go);
            GameObject? handled = ExecuteEvents.ExecuteHierarchy(go, ped, ExecuteEvents.pointerDownHandler);

            Selectable? sel = go.GetComponentInParent<Selectable>();
            if (sel != null)
            {
                sel.OnPointerDown(ped);
                // Select the control so the highlight driver paints the pressed/selected state.
                EventSystem.current?.SetSelectedGameObject(sel.gameObject, ped);
            }
            return handled != null || sel != null;
        }

        private static bool FireUp(GameObject go, PointerEventData ped)
        {
            bool h = ExecuteEvents.ExecuteHierarchy(go, ped, ExecuteEvents.pointerUpHandler) != null;
            Selectable? sel = go.GetComponentInParent<Selectable>();
            if (sel != null)
            {
                sel.OnPointerUp(ped);
            }
            return h || sel != null;
        }

        private static bool FireClick(GameObject go, PointerEventData ped)
        {
            ped.eligibleForClick = true;
            ped.clickCount = 1;
            GameObject? handled = ExecuteEvents.ExecuteHierarchy(go, ped, ExecuteEvents.pointerClickHandler);

            // Button.onClick fallback (some buttons relay only via Selectable).
            Button? button = go.GetComponentInParent<Button>();
            if (button != null && button.IsInteractable())
            {
                button.onClick.Invoke();
                return true;
            }

            // Toggle support.
            Toggle? toggle = go.GetComponentInParent<Toggle>();
            if (handled == null && toggle != null && toggle.IsInteractable())
            {
                toggle.isOn = !toggle.isOn;
                return true;
            }

            return handled != null;
        }

        // ── Helpers ────────────────────────────────────────────────────────

        private static bool TryParseEvent(string name, out PointerEvent evt)
        {
            switch ((name ?? string.Empty).Trim().ToLowerInvariant())
            {
                case "enter": evt = PointerEvent.Enter; return true;
                case "exit": evt = PointerEvent.Exit; return true;
                case "down": evt = PointerEvent.Down; return true;
                case "up": evt = PointerEvent.Up; return true;
                case "click": evt = PointerEvent.Click; return true;
                case "hover": evt = PointerEvent.Hover; return true;
                case "press": evt = PointerEvent.Press; return true;
                default: evt = PointerEvent.Click; return false;
            }
        }

        private static Vector2 ScreenCenterOf(Transform target)
        {
            if (target is RectTransform rt)
            {
                Vector3[] corners = new Vector3[4];
                rt.GetWorldCorners(corners);
                Vector3 worldCenter = (corners[0] + corners[2]) / 2f;
                Canvas? canvas = target.GetComponentInParent<Canvas>();
                Camera? cam = (canvas != null && canvas.renderMode != RenderMode.ScreenSpaceOverlay)
                    ? canvas.worldCamera
                    : null;
                return RectTransformUtility.WorldToScreenPoint(cam, worldCenter);
            }
            return new Vector2(Screen.width / 2f, Screen.height / 2f);
        }

        private static RaycastResult MakeRaycast(GameObject go) => new RaycastResult
        {
            gameObject = go,
            module = go.GetComponentInParent<GraphicRaycaster>(),
            screenPosition = ScreenCenterOf(go.transform),
        };

        private static UiActionResult Fail(string selector, string message, string reason) => new UiActionResult
        {
            Success = false,
            Selector = selector,
            Message = message,
            MatchCount = 0,
            Actionable = false,
            ActionabilityReason = reason,
        };
    }
}
