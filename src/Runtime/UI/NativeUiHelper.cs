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

        /// <summary>
        /// Clones a <see cref="Selectable"/> donor whose runtime type is NOT
        /// <see cref="Button"/> (e.g. DINO's <c>MainMenuButton : Selectable</c>).
        /// Steps:
        ///   1. Instantiate a copy of the donor's GameObject under the same parent.
        ///   2. Destroy every non-Unity component (game-specific scripts) PLUS the
        ///      original custom-selectable component (identified by type equality).
        ///   3. Add a fresh <see cref="Button"/> component to the clone root.
        ///   4. Clear persistent onclick events and reset navigation.
        ///   5. Set the clone's label text via <see cref="SetButtonText"/>.
        /// Returns the new <see cref="Button"/>, or <c>null</c> if the clone fails.
        /// </summary>
        /// <param name="donor">The custom Selectable to clone. Must not be a Button.</param>
        /// <param name="newText">Label text for the new button.</param>
        public static Button? CloneSelectableAsButton(Selectable donor, string newText)
        {
            if (donor == null) return null;
            if (donor is Button) return null;

            try
            {
                GameObject clone = UnityEngine.Object.Instantiate(donor.gameObject, donor.transform.parent);
                clone.name = "DINOForge_ModsButton";

                StripNonUiBehaviours(clone, preserveSelectables: true);

                Selectable preserved = clone.GetComponent<Selectable>();

                if (preserved != null)
                {
                    Navigation nav = preserved.navigation;
                    nav.mode = Navigation.Mode.Automatic;
                    preserved.navigation = nav;
                }

                Button btn = clone.GetComponent<Button>();
                if (btn == null)
                {
                    btn = clone.AddComponent<Button>();
                }
                btn.onClick = new Button.ButtonClickedEvent();

                if (preserved != null && preserved != btn)
                {
                    CopySelectableVisualState(btn, preserved);
                }

                SetButtonText(btn, newText);

                return btn;
            }
            catch
            {
                return null;
            }
        }

        /// <summary>
        /// Copies hover/press visual state from a donor <see cref="Selectable"/> onto a
        /// freshly-created <see cref="Button"/> so it matches the native menu button appearance
        /// (red highlight on hover/selection in DINO's MainMenuButton style).
        /// Safe to call even when the donor has no configured color block — the default
        /// Unity <see cref="ColorBlock"/> is used as-is in that case.
        /// </summary>
        /// <param name="target">The new Button that needs visual state.</param>
        /// <param name="donor">The original Selectable whose visual properties are the template.</param>
        internal static void CopySelectableVisualState(Button target, Selectable donor)
        {
            if (target == null || donor == null) return;
            try
            {
                target.transition = donor.transition;
                target.colors = donor.colors;
                target.spriteState = donor.spriteState;
                target.animationTriggers = donor.animationTriggers;
            }
            catch { /* safe-swallow: visual-state copy is best-effort; missing/destroyed donor is non-fatal */ }
        }

        private static void StripNonUiBehaviours(GameObject root, bool preserveSelectables = false)
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

                if (preserveSelectables && behaviour is Selectable)
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
            catch { /* safe-swallow: TMPro reflection is optional and best-effort */ }
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
            catch (System.Reflection.TargetInvocationException ex)
            {
                // Pattern #104 (Task #302): narrow catch to expected reflection failures only.
                // PropertyInfo.GetValue wraps the underlying exception in TargetInvocationException;
                // TypeLoadException/MissingMethodException occur when TMPro is wrong version.
                // Unexpected exceptions now propagate so caller can diagnose.
                System.Diagnostics.Debug.WriteLine($"[NativeUiHelper] TryGetTmpText reflection failed: {ex.InnerException ?? (Exception)ex}");
                return null;
            }
            catch (TypeLoadException ex)
            {
                System.Diagnostics.Debug.WriteLine($"[NativeUiHelper] TryGetTmpText TMPro type load failed: {ex}");
                return null;
            }
            catch (MissingMethodException ex)
            {
                System.Diagnostics.Debug.WriteLine($"[NativeUiHelper] TryGetTmpText TMPro API mismatch: {ex}");
                return null;
            }
        }
    }
}
