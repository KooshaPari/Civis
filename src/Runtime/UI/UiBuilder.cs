#nullable enable
using System;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Static helper factory for building UGUI elements procedurally in code.
    /// All methods create fully-configured GameObjects and attach them to the given parent.
    /// No Unity prefabs or asset bundles are required.
    /// </summary>
    public static class UiBuilder
    {
        private static Font? _cachedFont;
        private static bool _fontResolved;
        // ── Palette ─────────────────────────────────────────────────────────────
        public static readonly Color BgDeep = HexColor("#0d1a0f", 0.92f);
        public static readonly Color BgSurface = HexColor("#1c2b1e", 1f);
        public static readonly Color TextPrimary = HexColor("#e8d5b0", 1f);
        public static readonly Color TextSecondary = HexColor("#a89070", 1f);
        public static readonly Color Accent = HexColor("#c9a84c", 1f);
        public static readonly Color Success = HexColor("#4caf7d", 1f);
        public static readonly Color Error = HexColor("#e05252", 1f);
        public static readonly Color Warning = HexColor("#e8a020", 1f);
        public static readonly Color Border = HexColor("#2d4a32", 1f);
        public static readonly Color Transparent = new Color(0f, 0f, 0f, 0f);

        // ── Color helpers ────────────────────────────────────────────────────────

        /// <summary>Parses a CSS hex color string (#rrggbb or #rrggbbaa) and applies a uniform alpha override.</summary>
        public static Color HexColor(string hex, float alpha = 1f)
        {
            if (ColorUtility.TryParseHtmlString(hex, out Color c))
            {
                c.a = alpha;
                return c;
            }
            return new Color(1f, 0f, 1f, alpha); // magenta = bad hex, easy to spot
        }

        // ── Layout helpers ───────────────────────────────────────────────────────

        /// <summary>
        /// Sets a RectTransform to fill its parent completely (anchor all corners).
        /// </summary>
        public static void FillParent(RectTransform rt)
        {
            rt.anchorMin = Vector2.zero;
            rt.anchorMax = Vector2.one;
            rt.offsetMin = Vector2.zero;
            rt.offsetMax = Vector2.zero;
        }

        /// <summary>
        /// Anchors a RectTransform to the top-right corner with the given size.
        /// </summary>
        public static void AnchorTopRight(RectTransform rt, Vector2 size, Vector2 offset = default)
        {
            rt.anchorMin = Vector2.one;
            rt.anchorMax = Vector2.one;
            rt.pivot = Vector2.one;
            rt.sizeDelta = size;
            rt.anchoredPosition = offset;
        }

        // ── Primitive constructors ───────────────────────────────────────────────

        /// <summary>
        /// Creates a panel (Image) with either a Kenney 9-sliced sprite background or a flat colour fallback.
        /// When <see cref="UiAssets.PanelBackground"/> is available the Image is rendered as
        /// <see cref="Image.Type.Sliced"/> for crisp scaling; otherwise a solid colour is used.
        /// </summary>
        /// <param name="parent">Parent transform to attach to.</param>
        /// <param name="name">GameObject name.</param>
        /// <param name="bgColor">Background colour (tints the sprite, or used directly as flat fill).</param>
        /// <param name="size">Width/height in pixels. Use Vector2.zero to fill parent.</param>
        /// <returns>The root GameObject of the panel.</returns>
        public static GameObject MakePanel(Transform parent, string name, Color bgColor, Vector2 size)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image));
            go.transform.SetParent(parent, false);

            Image img = go.GetComponent<Image>();

            Sprite? panelSprite = UiAssets.PanelBackground;
            if (panelSprite != null)
            {
                img.sprite = panelSprite;
                img.type = Image.Type.Sliced;
                img.color = bgColor; // tints the sprite; pass Color.white for unmodified sprite
            }
            else
            {
                img.color = bgColor;
            }

            RectTransform rt = go.GetComponent<RectTransform>();
            if (size == Vector2.zero)
            {
                FillParent(rt);
            }
            else
            {
                rt.sizeDelta = size;
            }

            return go;
        }

        /// <summary>
        /// Creates a Text element.
        /// </summary>
        public static Text MakeText(
            Transform parent,
            string name,
            string text,
            int fontSize,
            Color color,
            bool bold = false,
            TextAnchor alignment = TextAnchor.MiddleLeft)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Text));
            go.transform.SetParent(parent, false);

            RectTransform rt = go.GetComponent<RectTransform>();
            rt.sizeDelta = new Vector2(200f, fontSize + 4f);  // Ensure text has visible dimensions

            Text t = go.GetComponent<Text>();
            t.text = text;
            t.fontSize = fontSize;
            t.color = color;
            t.alignment = alignment;
            if (!_fontResolved)
            {
                _fontResolved = true;
                _cachedFont = Resources.GetBuiltinResource<Font>("Arial.ttf");
                if (_cachedFont == null)
                    _cachedFont = Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf");
                if (_cachedFont == null)
                    _cachedFont = Font.CreateDynamicFontFromOSFont("Arial", 14);
            }
            if (_cachedFont != null)
            {
                t.font = _cachedFont;
                _cachedFont.RequestCharactersInTexture(text, fontSize,
                    bold ? FontStyle.Bold : FontStyle.Normal);
            }
            t.fontStyle = bold ? FontStyle.Bold : FontStyle.Normal;
            t.supportRichText = true;
            t.horizontalOverflow = HorizontalWrapMode.Wrap;
            t.verticalOverflow = VerticalWrapMode.Overflow;

            return t;
        }

        /// <summary>
        /// Creates a Button with a background image and label text.
        /// </summary>
        public static Button MakeButton(
            Transform parent,
            string name,
            string label,
            Color bgColor,
            Color textColor,
            Action onClick)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image), typeof(Button));
            go.transform.SetParent(parent, false);

            Image img = go.GetComponent<Image>();

            Sprite? btnSprite = UiAssets.ButtonNormal;
            if (btnSprite != null)
            {
                img.sprite = btnSprite;
                img.type = Image.Type.Sliced;
                img.color = bgColor; // tints sprite; pass Color.white for unmodified
            }
            else
            {
                img.color = bgColor;
            }

            Button btn = go.GetComponent<Button>();

            // Sprite swap on press if available, otherwise colour tint
            SpriteState ss = btn.spriteState;
            Sprite? btnPressedSprite = UiAssets.ButtonPressed;
            if (btnPressedSprite != null)
            {
                ss.pressedSprite = btnPressedSprite;
                ss.highlightedSprite = btnSprite; // same as normal — colour block handles highlight
                btn.spriteState = ss;
                btn.transition = Selectable.Transition.SpriteSwap;
            }

            // Colour tint block (also applies when using SpriteSwap for tint overlay)
            ColorBlock cb = btn.colors;
            cb.normalColor = bgColor;
            cb.highlightedColor = Color.Lerp(bgColor, Color.white, 0.15f);
            cb.pressedColor = Color.Lerp(bgColor, Color.black, 0.2f);
            cb.selectedColor = bgColor;
            cb.disabledColor = new Color(bgColor.r, bgColor.g, bgColor.b, 0.4f);
            cb.colorMultiplier = 1f;
            cb.fadeDuration = 0.1f;
            btn.colors = cb;

            btn.onClick.AddListener(() => onClick());

            // Label child
            Text txt = MakeText(go.transform, "Label", label, 13, textColor, bold: false, TextAnchor.MiddleCenter);
            FillParent(txt.GetComponent<RectTransform>());

            return btn;
        }

        /// <summary>
        /// Creates a ScrollView. Returns the ScrollRect component and the content RectTransform
        /// that child elements should be added to.
        /// </summary>
        public static (ScrollRect scrollRect, RectTransform content) MakeScrollView(
            Transform parent,
            string name,
            Vector2 size)
        {
            // Viewport
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image), typeof(ScrollRect), typeof(Mask));
            go.transform.SetParent(parent, false);

            RectTransform rt = go.GetComponent<RectTransform>();
            rt.sizeDelta = size;

            Image bgImg = go.GetComponent<Image>();
            bgImg.color = new Color(0f, 0f, 0f, 0f); // transparent, mask needs Image
            bgImg.raycastTarget = true;

            Mask mask = go.GetComponent<Mask>();
            mask.showMaskGraphic = false;

            // Content container
            GameObject content = new GameObject("Content", typeof(RectTransform));
            content.transform.SetParent(go.transform, false);

            RectTransform contentRt = content.GetComponent<RectTransform>();
            contentRt.anchorMin = new Vector2(0f, 1f);
            contentRt.anchorMax = new Vector2(1f, 1f);
            contentRt.pivot = new Vector2(0.5f, 1f);
            contentRt.offsetMin = Vector2.zero;
            contentRt.offsetMax = Vector2.zero;

            // Content size fitter
            ContentSizeFitter csf = content.AddComponent<ContentSizeFitter>();
            csf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            // Vertical layout group on content
            VerticalLayoutGroup vlg = content.AddComponent<VerticalLayoutGroup>();
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;
            vlg.spacing = 2f;
            vlg.padding = new RectOffset(4, 4, 4, 4);

            ScrollRect scrollRect = go.GetComponent<ScrollRect>();
            scrollRect.content = contentRt;
            scrollRect.horizontal = false;
            scrollRect.vertical = true;
            scrollRect.scrollSensitivity = 20f;
            scrollRect.movementType = ScrollRect.MovementType.Clamped;
            scrollRect.viewport = rt;

            return (scrollRect, contentRt);
        }

        /// <summary>
        /// Creates a single-line InputField.
        /// </summary>
        public static InputField MakeInputField(
            Transform parent,
            string name,
            string placeholder,
            Action<string> onChanged)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Image));
            go.transform.SetParent(parent, false);

            Image bgImg = go.GetComponent<Image>();
            bgImg.color = HexColor("#0a130b", 1f);

            InputField input = go.AddComponent<InputField>();

            // Placeholder
            Text ph = MakeText(go.transform, "Placeholder", placeholder, 13, TextSecondary);
            RectTransform phRt = ph.GetComponent<RectTransform>();
            FillParent(phRt);
            phRt.offsetMin = new Vector2(6f, 0f);
            ph.fontStyle = FontStyle.Italic;
            input.placeholder = ph;

            // Text child
            Text txt = MakeText(go.transform, "Text", "", 13, TextPrimary);
            RectTransform txtRt = txt.GetComponent<RectTransform>();
            FillParent(txtRt);
            txtRt.offsetMin = new Vector2(6f, 0f);
            input.textComponent = txt;

            input.onValueChanged.AddListener(v => onChanged(v));

            return input;
        }

        /// <summary>
        /// Creates a Toggle (checkbox-style).
        /// </summary>
        public static Toggle MakeToggle(
            Transform parent,
            string name,
            bool initial,
            Action<bool> onChanged)
        {
            GameObject go = new GameObject(name, typeof(RectTransform), typeof(Toggle));
            go.transform.SetParent(parent, false);

            // Background box
            GameObject bg = MakePanel(go.transform, "Background", HexColor("#0a130b", 1f), new Vector2(20f, 20f));
            RectTransform bgRt = bg.GetComponent<RectTransform>();
            bgRt.anchorMin = new Vector2(0f, 0.5f);
            bgRt.anchorMax = new Vector2(0f, 0.5f);
            bgRt.pivot = new Vector2(0f, 0.5f);
            bgRt.anchoredPosition = Vector2.zero;

            // Checkmark
            GameObject checkGo = MakePanel(bg.transform, "Checkmark", Accent, Vector2.zero);
            RectTransform checkRt = checkGo.GetComponent<RectTransform>();
            checkRt.anchorMin = new Vector2(0.1f, 0.1f);
            checkRt.anchorMax = new Vector2(0.9f, 0.9f);
            checkRt.offsetMin = Vector2.zero;
            checkRt.offsetMax = Vector2.zero;

            Toggle toggle = go.GetComponent<Toggle>();
            toggle.isOn = initial;
            toggle.targetGraphic = bg.GetComponent<Image>();
            toggle.graphic = checkGo.GetComponent<Image>();
            toggle.onValueChanged.AddListener(v => onChanged(v));

            return toggle;
        }

        /// <summary>
        /// Creates a thin horizontal separator line.
        /// </summary>
        public static GameObject MakeHorizontalSeparator(Transform parent, Color color)
        {
            GameObject go = new GameObject("Separator", typeof(RectTransform), typeof(Image));
            go.transform.SetParent(parent, false);

            Image img = go.GetComponent<Image>();
            img.color = color;

            RectTransform rt = go.GetComponent<RectTransform>();
            rt.sizeDelta = new Vector2(0f, 1f);

            // Stretch horizontally
            LayoutElement le = go.AddComponent<LayoutElement>();
            le.minHeight = 1f;
            le.preferredHeight = 1f;
            le.flexibleWidth = 1f;

            return go;
        }

        /// <summary>
        /// Creates a CanvasGroup on a GameObject for alpha/interaction control.
        /// </summary>
        public static CanvasGroup EnsureCanvasGroup(GameObject go)
        {
            CanvasGroup cg = go.GetComponent<CanvasGroup>();
            if (cg == null) cg = go.AddComponent<CanvasGroup>();
            return cg;
        }

        /// <summary>
        /// Adds a vertical layout group to an existing GameObject.
        /// </summary>
        public static VerticalLayoutGroup AddVerticalLayout(
            GameObject go,
            float spacing = 4f,
            RectOffset? padding = null)
        {
            VerticalLayoutGroup vlg = go.GetComponent<VerticalLayoutGroup>();
            if (vlg == null) vlg = go.AddComponent<VerticalLayoutGroup>();
            vlg.spacing = spacing;
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;
            if (padding != null) vlg.padding = padding;
            return vlg;
        }

        /// <summary>
        /// Adds a horizontal layout group to an existing GameObject.
        /// </summary>
        public static HorizontalLayoutGroup AddHorizontalLayout(
            GameObject go,
            float spacing = 4f,
            RectOffset? padding = null)
        {
            HorizontalLayoutGroup hlg = go.GetComponent<HorizontalLayoutGroup>();
            if (hlg == null) hlg = go.AddComponent<HorizontalLayoutGroup>();
            hlg.spacing = spacing;
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = true;
            if (padding != null) hlg.padding = padding;
            return hlg;
        }
    }

}
