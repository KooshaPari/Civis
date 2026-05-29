#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Modal UGUI dialog displayed when the user tries to enable a pack whose dependencies
    /// are currently disabled.
    ///
    /// Layout:
    ///   - Full-screen dark overlay (blocks all input below)
    ///   - Centered card: title · dep list · "Enable All" + "Cancel" buttons
    ///
    /// Usage:
    ///   1. Call <see cref="DepEnableDialog.Show"/> with the missing-dep names and two callbacks.
    ///   2. The dialog destroys itself when either button is pressed.
    ///
    /// Color palette reuses <see cref="UiBuilder"/> constants so it matches the rest of the
    /// mod menu.
    /// </summary>
    public sealed class DepEnableDialog : MonoBehaviour
    {
        // ── Constants ─────────────────────────────────────────────────────────────
        private const float CardWidth = 420f;
        private const float CardMinHeight = 180f;

        // ── Factory ───────────────────────────────────────────────────────────────

        /// <summary>
        /// Builds and shows the dialog as a child of <paramref name="canvasRoot"/>.
        /// Returns the dialog MonoBehaviour so callers can destroy it early if needed.
        /// </summary>
        /// <param name="canvasRoot">Root of the DINOForge canvas (or any active canvas).</param>
        /// <param name="packName">Display name of the pack being enabled.</param>
        /// <param name="missingDepNames">Human-readable names / IDs of the disabled dependencies.</param>
        /// <param name="onEnableAll">Invoked when the user clicks "Enable All".</param>
        /// <param name="onCancel">Invoked when the user clicks "Cancel".</param>
        public static DepEnableDialog Show(
            Transform canvasRoot,
            string packName,
            IReadOnlyList<string> missingDepNames,
            Action onEnableAll,
            Action onCancel)
        {
            GameObject go = new GameObject("DepEnableDialog", typeof(RectTransform));
            go.transform.SetParent(canvasRoot, false);

            DepEnableDialog dialog = go.AddComponent<DepEnableDialog>();
            dialog.Build(packName, missingDepNames, onEnableAll, onCancel);
            return dialog;
        }

        // ── Internal build ────────────────────────────────────────────────────────

        private void Build(
            string packName,
            IReadOnlyList<string> missingDepNames,
            Action onEnableAll,
            Action onCancel)
        {
            // ── Full-screen overlay (blocks input, dims background) ──
            RectTransform rootRt = gameObject.GetComponent<RectTransform>();
            rootRt.anchorMin = Vector2.zero;
            rootRt.anchorMax = Vector2.one;
            rootRt.offsetMin = Vector2.zero;
            rootRt.offsetMax = Vector2.zero;

            Image overlay = gameObject.AddComponent<Image>();
            overlay.color = new Color(0f, 0f, 0f, 0.65f);
            overlay.raycastTarget = true; // blocks clicks behind the dialog

            // Ensure the overlay is drawn on top of everything else in the panel
            Canvas? sortCanvas = gameObject.AddComponent<Canvas>();
            sortCanvas.overrideSorting = true;
            sortCanvas.sortingOrder = 200; // well above the mod-menu panel
            Plugin.EnsureEventSystemAlive(); // Pattern #235: EventSystem before GraphicRaycaster
            gameObject.AddComponent<GraphicRaycaster>();

            // ── Centered card ──
            GameObject card = new GameObject("Card", typeof(RectTransform), typeof(Image));
            card.transform.SetParent(gameObject.transform, false);

            Image cardImg = card.GetComponent<Image>();
            cardImg.color = UiBuilder.BgSurface;

            RectTransform cardRt = card.GetComponent<RectTransform>();
            cardRt.anchorMin = new Vector2(0.5f, 0.5f);
            cardRt.anchorMax = new Vector2(0.5f, 0.5f);
            cardRt.pivot = new Vector2(0.5f, 0.5f);
            cardRt.anchoredPosition = Vector2.zero;
            cardRt.sizeDelta = new Vector2(CardWidth, CardMinHeight);

            VerticalLayoutGroup cardLayout = card.AddComponent<VerticalLayoutGroup>();
            cardLayout.padding = new RectOffset(20, 20, 16, 16);
            cardLayout.spacing = 10f;
            cardLayout.childForceExpandWidth = true;
            cardLayout.childForceExpandHeight = false;
            cardLayout.childControlWidth = true;
            cardLayout.childControlHeight = true;

            ContentSizeFitter csf = card.AddComponent<ContentSizeFitter>();
            csf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            // ── Title ──
            Text title = UiBuilder.MakeText(card.transform, "Title",
                $"\"{packName}\" requires:", 15, UiBuilder.Accent, bold: true);
            LayoutElement titleLe = title.gameObject.AddComponent<LayoutElement>();
            titleLe.preferredHeight = 22f;
            titleLe.flexibleWidth = 1f;

            // ── Separator ──
            UiBuilder.MakeHorizontalSeparator(card.transform, UiBuilder.Border);

            // ── Dependency list ──
            System.Text.StringBuilder depSb = new System.Text.StringBuilder(128);
            for (int i = 0; i < missingDepNames.Count; i++)
            {
                if (i > 0) depSb.Append('\n');
                depSb.Append("  • ").Append(missingDepNames[i]).Append("  (disabled)");
            }

            Text depList = UiBuilder.MakeText(card.transform, "DepList",
                depSb.ToString(), 13, UiBuilder.TextPrimary);
            depList.verticalOverflow = VerticalWrapMode.Overflow;
            LayoutElement depLe = depList.gameObject.AddComponent<LayoutElement>();
            depLe.flexibleWidth = 1f;
            // height: approx 20px per dependency line
            depLe.preferredHeight = Mathf.Max(20f, missingDepNames.Count * 20f);

            // ── Sub-label ──
            Text subLabel = UiBuilder.MakeText(card.transform, "SubLabel",
                "Enable these packs too?", 12, UiBuilder.TextSecondary);
            LayoutElement subLe = subLabel.gameObject.AddComponent<LayoutElement>();
            subLe.preferredHeight = 18f;
            subLe.flexibleWidth = 1f;

            // ── Separator ──
            UiBuilder.MakeHorizontalSeparator(card.transform, UiBuilder.Border);

            // ── Button row ──
            GameObject btnRow = new GameObject("ButtonRow", typeof(RectTransform));
            btnRow.transform.SetParent(card.transform, false);
            HorizontalLayoutGroup btnLayout = btnRow.AddComponent<HorizontalLayoutGroup>();
            btnLayout.spacing = 10f;
            btnLayout.childForceExpandWidth = false;
            btnLayout.childForceExpandHeight = false;
            btnLayout.childAlignment = TextAnchor.MiddleCenter;
            LayoutElement btnRowLe = btnRow.AddComponent<LayoutElement>();
            btnRowLe.preferredHeight = 34f;
            btnRowLe.flexibleWidth = 1f;

            // "Cancel" button (left)
            Button cancelBtn = UiBuilder.MakeButton(
                btnRow.transform, "CancelBtn", "Cancel",
                UiBuilder.BgDeep, UiBuilder.TextSecondary,
                () =>
                {
                    onCancel?.Invoke();
                    Destroy(gameObject);
                });
            LayoutElement cancelLe = cancelBtn.gameObject.AddComponent<LayoutElement>();
            cancelLe.preferredWidth = 100f;
            cancelLe.preferredHeight = 32f;

            // "Enable All" button (right — accented)
            Button enableAllBtn = UiBuilder.MakeButton(
                btnRow.transform, "EnableAllBtn", "Enable All",
                UiBuilder.Accent, UiBuilder.BgDeep,
                () =>
                {
                    onEnableAll?.Invoke();
                    Destroy(gameObject);
                });
            LayoutElement enableAllLe = enableAllBtn.gameObject.AddComponent<LayoutElement>();
            enableAllLe.preferredWidth = 120f;
            enableAllLe.preferredHeight = 32f;

            // Force card to compute correct height immediately so it does not flash small
            UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(cardRt);
        }
    }
}
