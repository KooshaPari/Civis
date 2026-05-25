#nullable enable
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Static style kit for all DINOForge IMGUI windows.
    /// Provides a dark navy-charcoal theme with gold accent, status colors, and reusable
    /// <see cref="GUIStyle"/> instances. All styles are initialized lazily on first access
    /// since Unity IMGUI requires an active OnGUI context to build styles.
    /// </summary>
    public static class DinoForgeStyle
    {
        // ── Palette ──────────────────────────────────────────────────────────────

        /// <summary>Dark navy-charcoal background (#1a1a2e).</summary>
        public static readonly Color Background = new Color(0.102f, 0.102f, 0.180f, 0.97f);

        /// <summary>Slightly lighter panel surface for nested boxes.</summary>
        public static readonly Color PanelBackground = new Color(0.13f, 0.13f, 0.22f, 1f);

        /// <summary>Gold/amber accent color (#f0a500) for selected items and headers.</summary>
        public static readonly Color Accent = new Color(0.941f, 0.647f, 0f, 1f);

        /// <summary>Dimmed accent for non-selected but hovered items.</summary>
        public static readonly Color AccentDim = new Color(0.941f, 0.647f, 0f, 0.55f);

        /// <summary>Error color — red-orange (#e63946).</summary>
        public static readonly Color Error = new Color(0.902f, 0.224f, 0.275f, 1f);

        /// <summary>Warning color — amber (#f4a261).</summary>
        public static readonly Color Warning = new Color(0.957f, 0.635f, 0.380f, 1f);

        /// <summary>Success / OK color — teal-green (#2a9d8f).</summary>
        public static readonly Color Success = new Color(0.165f, 0.616f, 0.561f, 1f);

        /// <summary>Primary body text color — off-white.</summary>
        public static readonly Color TextPrimary = new Color(0.90f, 0.90f, 0.90f, 1f);

        /// <summary>Secondary / muted text color.</summary>
        public static readonly Color TextMuted = new Color(0.55f, 0.55f, 0.65f, 1f);

        // ── Lazy-initialized styles ───────────────────────────────────────────────

        private static GUIStyle? _headerStyle;
        private static GUIStyle? _sectionLabelStyle;
        private static GUIStyle? _packNameStyle;
        private static GUIStyle? _packNameSelectedStyle;
        private static GUIStyle? _statusBadgeStyle;
        private static GUIStyle? _errorLabelStyle;
        private static GUIStyle? _warningLabelStyle;
        private static GUIStyle? _successLabelStyle;
        private static GUIStyle? _bodyLabelStyle;
        private static GUIStyle? _fieldLabelStyle;
        private static GUIStyle? _windowStyle;
        private static GUIStyle? _boxStyle;
        private static GUIStyle? _buttonStyle;
        private static GUIStyle? _selectedRowStyle;
        private static Texture2D? _whiteTex;

        // ── Texture helper ────────────────────────────────────────────────────────

        private static Texture2D White
        {
            get
            {
                if (_whiteTex == null)
                {
                    _whiteTex = new Texture2D(1, 1);
                    _whiteTex.SetPixel(0, 0, Color.white);
                    _whiteTex.Apply();
                }
                return _whiteTex;
            }
        }

        private static Texture2D MakeTex(Color color)
        {
            Texture2D tex = new Texture2D(1, 1);
            tex.SetPixel(0, 0, color);
            tex.Apply();
            return tex;
        }

        // ── Style accessors ───────────────────────────────────────────────────────

        /// <summary>Large bold title/header text in gold accent.</summary>
        public static GUIStyle HeaderStyle
        {
            get
            {
                if (_headerStyle == null)
                {
                    _headerStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 13,
                        fontStyle = FontStyle.Bold,
                        alignment = TextAnchor.MiddleLeft
                    };
                    _headerStyle.normal.textColor = Accent;
                }
                return _headerStyle;
            }
        }

        /// <summary>Section divider label — small caps, muted gold.</summary>
        public static GUIStyle SectionLabelStyle
        {
            get
            {
                if (_sectionLabelStyle == null)
                {
                    _sectionLabelStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 10,
                        fontStyle = FontStyle.Bold,
                        alignment = TextAnchor.MiddleLeft
                    };
                    _sectionLabelStyle.normal.textColor = AccentDim;
                }
                return _sectionLabelStyle;
            }
        }

        /// <summary>Normal pack name in the list — off-white.</summary>
        public static GUIStyle PackNameStyle
        {
            get
            {
                if (_packNameStyle == null)
                {
                    _packNameStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 12,
                        fontStyle = FontStyle.Normal,
                        alignment = TextAnchor.MiddleLeft
                    };
                    _packNameStyle.normal.textColor = TextPrimary;
                    _packNameStyle.hover.textColor = Accent;
                }
                return _packNameStyle;
            }
        }

        /// <summary>Selected pack name — bold gold.</summary>
        public static GUIStyle PackNameSelectedStyle
        {
            get
            {
                if (_packNameSelectedStyle == null)
                {
                    _packNameSelectedStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 12,
                        fontStyle = FontStyle.Bold,
                        alignment = TextAnchor.MiddleLeft
                    };
                    _packNameSelectedStyle.normal.textColor = Accent;
                }
                return _packNameSelectedStyle;
            }
        }

        /// <summary>Compact badge label — small, bold, centered.</summary>
        public static GUIStyle StatusBadgeStyle
        {
            get
            {
                if (_statusBadgeStyle == null)
                {
                    _statusBadgeStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 9,
                        fontStyle = FontStyle.Bold,
                        alignment = TextAnchor.MiddleCenter,
                        padding = new RectOffset(4, 4, 1, 1)
                    };
                    _statusBadgeStyle.normal.textColor = Color.white;
                }
                return _statusBadgeStyle;
            }
        }

        /// <summary>Error text — red-orange.</summary>
        public static GUIStyle ErrorLabelStyle
        {
            get
            {
                if (_errorLabelStyle == null)
                {
                    _errorLabelStyle = new GUIStyle(GUI.skin.label) { fontSize = 11 };
                    _errorLabelStyle.normal.textColor = Error;
                }
                return _errorLabelStyle;
            }
        }

        /// <summary>Warning text — amber.</summary>
        public static GUIStyle WarningLabelStyle
        {
            get
            {
                if (_warningLabelStyle == null)
                {
                    _warningLabelStyle = new GUIStyle(GUI.skin.label) { fontSize = 11 };
                    _warningLabelStyle.normal.textColor = Warning;
                }
                return _warningLabelStyle;
            }
        }

        /// <summary>Success text — teal-green.</summary>
        public static GUIStyle SuccessLabelStyle
        {
            get
            {
                if (_successLabelStyle == null)
                {
                    _successLabelStyle = new GUIStyle(GUI.skin.label) { fontSize = 11 };
                    _successLabelStyle.normal.textColor = Success;
                }
                return _successLabelStyle;
            }
        }

        /// <summary>Standard body text — off-white, word-wrapped.</summary>
        public static GUIStyle BodyLabelStyle
        {
            get
            {
                if (_bodyLabelStyle == null)
                {
                    _bodyLabelStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 11,
                        wordWrap = true
                    };
                    _bodyLabelStyle.normal.textColor = TextPrimary;
                }
                return _bodyLabelStyle;
            }
        }

        /// <summary>Bold field label (e.g. "Author:", "Version:") — off-white bold.</summary>
        public static GUIStyle FieldLabelStyle
        {
            get
            {
                if (_fieldLabelStyle == null)
                {
                    _fieldLabelStyle = new GUIStyle(GUI.skin.label)
                    {
                        fontSize = 11,
                        fontStyle = FontStyle.Bold
                    };
                    _fieldLabelStyle.normal.textColor = TextPrimary;
                }
                return _fieldLabelStyle;
            }
        }

        /// <summary>Dark-background window style.</summary>
        public static GUIStyle WindowStyle
        {
            get
            {
                if (_windowStyle == null)
                {
                    _windowStyle = new GUIStyle(GUI.skin.window);
                    _windowStyle.normal.background = MakeTex(Background);
                    _windowStyle.onNormal.background = MakeTex(Background);
                    _windowStyle.normal.textColor = Accent;
                    _windowStyle.fontStyle = FontStyle.Bold;
                    _windowStyle.fontSize = 12;
                }
                return _windowStyle;
            }
        }

        /// <summary>Dark panel box style.</summary>
        public static GUIStyle BoxStyle
        {
            get
            {
                if (_boxStyle == null)
                {
                    _boxStyle = new GUIStyle(GUI.skin.box);
                    _boxStyle.normal.background = MakeTex(PanelBackground);
                }
                return _boxStyle;
            }
        }

        /// <summary>Dark-themed button with gold text on hover.</summary>
        public static GUIStyle ButtonStyle
        {
            get
            {
                if (_buttonStyle == null)
                {
                    _buttonStyle = new GUIStyle(GUI.skin.button)
                    {
                        fontSize = 11,
                        fontStyle = FontStyle.Bold
                    };
                    _buttonStyle.normal.background = MakeTex(new Color(0.18f, 0.18f, 0.30f, 1f));
                    _buttonStyle.hover.background = MakeTex(new Color(0.22f, 0.22f, 0.38f, 1f));
                    _buttonStyle.normal.textColor = TextPrimary;
                    _buttonStyle.hover.textColor = Accent;
                    _buttonStyle.active.textColor = Accent;
                }
                return _buttonStyle;
            }
        }

        /// <summary>Highlighted row background for the selected pack.</summary>
        public static GUIStyle SelectedRowStyle
        {
            get
            {
                if (_selectedRowStyle == null)
                {
                    _selectedRowStyle = new GUIStyle(GUI.skin.box);
                    _selectedRowStyle.normal.background = MakeTex(new Color(0.941f, 0.647f, 0f, 0.18f));
                    _selectedRowStyle.margin = new RectOffset(0, 0, 0, 0);
                    _selectedRowStyle.padding = new RectOffset(2, 2, 2, 2);
                }
                return _selectedRowStyle;
            }
        }

        // ── Drawing helpers ───────────────────────────────────────────────────────

        /// <summary>
        /// Draws a small filled badge rectangle with centered text.
        /// Call inside an OnGUI method. Uses <see cref="GUILayout"/> flow.
        /// </summary>
        /// <param name="text">Text to display inside the badge.</param>
        /// <param name="badgeColor">Fill color of the badge.</param>
        /// <param name="width">Optional fixed width override.</param>
        public static void StatusBadge(string text, Color badgeColor, float width = 0f)
        {
            GUIStyle style = StatusBadgeStyle;
            GUILayoutOption[] opts = width > 0f
                ? new[] { GUILayout.Width(width) }
                : new[] { GUILayout.ExpandWidth(false) };

            Rect rect = GUILayoutUtility.GetRect(
                new GUIContent(text), style, opts);

            Color saved = GUI.backgroundColor;
            GUI.backgroundColor = badgeColor;
            GUI.Box(rect, GUIContent.none);
            GUI.backgroundColor = saved;

            Color savedColor = GUI.color;
            GUI.color = Color.white;
            GUI.Label(rect, text, style);
            GUI.color = savedColor;
        }

        /// <summary>
        /// Invalidates all cached styles. Call when a new OnGUI context begins
        /// (e.g., after scene load) to force re-initialization.
        /// </summary>
        public static void Invalidate()
        {
            _headerStyle = null;
            _sectionLabelStyle = null;
            _packNameStyle = null;
            _packNameSelectedStyle = null;
            _statusBadgeStyle = null;
            _errorLabelStyle = null;
            _warningLabelStyle = null;
            _successLabelStyle = null;
            _bodyLabelStyle = null;
            _fieldLabelStyle = null;
            _windowStyle = null;
            _boxStyle = null;
            _buttonStyle = null;
            _selectedRowStyle = null;
        }
    }
}
