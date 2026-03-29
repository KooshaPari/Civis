#nullable enable
using System;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Utility methods for UGUI canvas/button manipulation.
    /// Handles both legacy <see cref="UnityEngine.UI.Text"/> and TMPro.TMP_Text labels.
    /// All methods are static and safe to call from any MonoBehaviour context.
    /// </summary>
    internal static class NativeUiHelper
    {
        /// <summary>
        /// Recursively searches <paramref name="root"/> for a <see cref="Button"/> whose
        /// visible label text contains <paramref name="text"/> (case-insensitive).
        /// Returns the first match, or <c>null</c> if none is found.
        /// </summary>
        /// <param name="root">Root transform to search under.</param>
        /// <param name="text">Text to match against button labels.</param>
        public static Button? FindButtonByText(Transform root, string text)
        {
            if (root == null) return null;

            Button[] buttons = root.GetComponentsInChildren<Button>(includeInactive: true);
            foreach (Button btn in buttons)
            {
                string label = GetButtonText(btn);
                if (label.IndexOf(text, StringComparison.OrdinalIgnoreCase) >= 0)
                    return btn;
            }

            return null;
        }

        /// <summary>
        /// Instantiates a copy of <paramref name="source"/>, renames it, and sets its label text.
        /// The clone is placed under the same parent as the source.
        /// </summary>
        /// <param name="source">Button to clone.</param>
        /// <param name="newText">Label text for the cloned button.</param>
        /// <returns>The cloned <see cref="Button"/>.</returns>
        public static Button CloneButton(Button source, string newText)
        {
            GameObject clone = UnityEngine.Object.Instantiate(source.gameObject, source.transform.parent);
            clone.name = "DINOForge_ModsButton";
            Button cloneBtn = clone.GetComponent<Button>();

            // Drop any persistent/runtime callbacks inherited from the source button.
            cloneBtn.onClick = new Button.ButtonClickedEvent();

            // Remove any cloned game-specific scripts and event triggers that can still
            // invoke the original Settings/Options behavior on click/submit.
            StripNonUiBehaviours(clone);

            // Reset navigation to prevent inherited EventSystem conflicts from original button's menu layout
            Navigation nav = cloneBtn.navigation;
            nav.mode = Navigation.Mode.Automatic;
            cloneBtn.navigation = nav;

            SetButtonText(cloneBtn, newText);
            return cloneBtn;
        }

        internal static void SanitizeUiClone(GameObject root)
        {
            StripNonUiBehaviours(root);
        }

        private static void StripNonUiBehaviours(GameObject root)
        {
            MonoBehaviour[] behaviours = root.GetComponentsInChildren<MonoBehaviour>(includeInactive: true);
            foreach (MonoBehaviour behaviour in behaviours)
            {
                if (behaviour == null) continue;

                if (behaviour is Button
                    || behaviour is Graphic
                    || behaviour is LayoutElement
                    || behaviour is LayoutGroup
                    || behaviour is ContentSizeFitter)
                {
                    continue;
                }

                if (behaviour is EventTrigger trigger)
                {
                    trigger.triggers?.Clear();
                    UnityEngine.Object.Destroy(trigger);
                    continue;
                }

                string? ns = behaviour.GetType().Namespace;
                if (ns != null && ns.StartsWith("UnityEngine", StringComparison.Ordinal))
                {
                    continue;
                }

                UnityEngine.Object.Destroy(behaviour);
            }
        }

        /// <summary>
        /// Positions <paramref name="target"/> immediately before <paramref name="sibling"/> in
        /// the shared parent's child list.  Both transforms must share the same parent.
        /// </summary>
        /// <param name="target">Transform to reposition.</param>
        /// <param name="sibling">Reference sibling; target will be placed before this.</param>
        public static void PositionBeforeSibling(RectTransform target, RectTransform sibling)
        {
            if (target == null || sibling == null) return;
            if (target.parent != sibling.parent) return;

            int siblingIndex = sibling.GetSiblingIndex();
            target.SetSiblingIndex(siblingIndex);
        }

        /// <summary>
        /// Positions <paramref name="target"/> immediately after <paramref name="sibling"/> in
        /// the shared parent's child list.  Both transforms must share the same parent.
        /// </summary>
        /// <param name="target">Transform to reposition.</param>
        /// <param name="sibling">Reference sibling; target will be placed after this.</param>
        public static void PositionAfterSibling(RectTransform target, RectTransform sibling)
        {
            if (target == null || sibling == null) return;
            if (target.parent != sibling.parent) return;

            int siblingIndex = sibling.GetSiblingIndex();
            target.SetSiblingIndex(siblingIndex + 1);
        }

        /// <summary>
        /// Sets the visible label text on <paramref name="btn"/>.
        /// Checks for both legacy <see cref="UnityEngine.UI.Text"/> and TMPro.TMP_Text children.
        /// </summary>
        /// <param name="btn">Button whose label should be changed.</param>
        /// <param name="text">New label text.</param>
        public static void SetButtonText(Button btn, string text)
        {
            if (btn == null) return;

            // Try legacy UnityEngine.UI.Text first
            UnityEngine.UI.Text? legacyText = btn.GetComponentInChildren<UnityEngine.UI.Text>();
            if (legacyText != null)
            {
                legacyText.text = text;
                return;
            }

            // Try TMPro via reflection (avoids a hard dependency on the TMPro assembly)
            TrySetTmpText(btn.gameObject, text);
        }

        /// <summary>
        /// Returns the visible label text from a button, preferring TMPro over legacy Text.
        /// Returns an empty string if no text component is found.
        /// </summary>
        /// <param name="btn">Button to read the label from.</param>
        public static string GetButtonText(Button btn)
        {
            if (btn == null) return string.Empty;

            UnityEngine.UI.Text? legacyText = btn.GetComponentInChildren<UnityEngine.UI.Text>();
            if (legacyText != null) return legacyText.text ?? string.Empty;

            return TryGetTmpText(btn.gameObject) ?? string.Empty;
        }

        // ------------------------------------------------------------------ //
        // TMPro helpers — done via reflection so we don't hard-depend on TMPro.
        // ------------------------------------------------------------------ //

        private static void TrySetTmpText(GameObject root, string text)
        {
            try
            {
                // TMP_Text is the base class for both TextMeshPro and TextMeshProUGUI
                Type? tmpType = Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro")
                             ?? Type.GetType("TMPro.TMP_Text, Assembly-CSharp");

                if (tmpType == null) return;

                Component? tmp = root.GetComponentInChildren(tmpType);
                if (tmp == null) return;

                System.Reflection.PropertyInfo? prop = tmpType.GetProperty("text");
                prop?.SetValue(tmp, text);
            }
            catch
            {
                // TMPro not present or reflection failed — silently ignore
            }
        }

        private static string? TryGetTmpText(GameObject root)
        {
            try
            {
                Type? tmpType = Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro")
                             ?? Type.GetType("TMPro.TMP_Text, Assembly-CSharp");

                if (tmpType == null) return null;

                Component? tmp = root.GetComponentInChildren(tmpType);
                if (tmp == null) return null;

                System.Reflection.PropertyInfo? prop = tmpType.GetProperty("text");
                return prop?.GetValue(tmp) as string;
            }
            catch
            {
                return null;
            }
        }
    }
}
