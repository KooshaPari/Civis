#nullable enable
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Small always-on badge that marks DINOForge as loaded and active.
    /// </summary>
    public sealed class DinoForgeIndicator : MonoBehaviour
    {
        private Text? _label;
        private Image? _backdrop;

        public void Build(Transform parent)
        {
            GameObject root = new GameObject("DinoForgeIndicator", typeof(RectTransform), typeof(Image));
            root.transform.SetParent(parent, false);

            RectTransform rt = root.GetComponent<RectTransform>();
            rt.anchorMin = new Vector2(0f, 1f);
            rt.anchorMax = new Vector2(0f, 1f);
            rt.pivot = new Vector2(0f, 1f);
            rt.anchoredPosition = new Vector2(10f, -10f);
            rt.sizeDelta = new Vector2(210f, 26f);

            _backdrop = root.GetComponent<Image>();
            _backdrop.raycastTarget = false;
            _backdrop.color = new Color(0f, 0f, 0f, 0.28f);

            GameObject labelGo = new GameObject("Label", typeof(RectTransform), typeof(Text));
            labelGo.transform.SetParent(root.transform, false);
            RectTransform labelRt = labelGo.GetComponent<RectTransform>();
            labelRt.anchorMin = Vector2.zero;
            labelRt.anchorMax = Vector2.one;
            labelRt.offsetMin = new Vector2(10f, 2f);
            labelRt.offsetMax = new Vector2(-10f, -2f);

            _label = labelGo.GetComponent<Text>();
            _label.raycastTarget = false;
            _label.alignment = TextAnchor.MiddleLeft;
            _label.font = Resources.GetBuiltinResource<Font>("Arial.ttf");
            _label.fontSize = 12;
            _label.fontStyle = FontStyle.Bold;
            _label.text = "DINOForge";
            _label.color = new Color(1f, 1f, 1f, 0.78f);
            _label.supportRichText = false;
            _label.horizontalOverflow = HorizontalWrapMode.Overflow;
            _label.verticalOverflow = VerticalWrapMode.Truncate;
        }

        public void SetTheme(string activeConversionName, Color primaryColor)
        {
            if (_label == null || _backdrop == null) return;
            string label = string.IsNullOrEmpty(activeConversionName)
                ? "DINOForge"
                : "DINOForge  " + activeConversionName;
            _label.text = label;
            _label.color = new Color(primaryColor.r, primaryColor.g, primaryColor.b, 0.86f);
            _backdrop.color = new Color(primaryColor.r, primaryColor.g, primaryColor.b, 0.12f);
        }
    }
}
