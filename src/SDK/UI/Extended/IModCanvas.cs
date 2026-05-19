// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

using System;
using DINOForge.SDK.UI.Models;

namespace DINOForge.SDK.UI.Extended
{
    /// <summary>
    /// Mod-owned overlay canvas — i.e. <c>DFCanvas_Root</c>. Lets packs spawn
    /// panels / labels / buttons under the DINOForge overlay surface, distinct
    /// from any vanilla DINO canvas. Phase 2 implementation lives in Runtime/UI.
    /// </summary>
    public interface IModCanvas
    {
        /// <summary>Spawn a panel under the mod canvas.</summary>
        ExtendedPanelHandle CreatePanel(PanelSpec spec);

        /// <summary>Spawn a label under the mod canvas.</summary>
        ExtendedLabelHandle CreateLabel(LabelSpec spec);

        /// <summary>Spawn a button under the mod canvas.</summary>
        ExtendedButtonHandle CreateButton(ButtonSpec spec);

        /// <summary>Destroy a previously created handle and any child elements.</summary>
        void Destroy(ExtendedHandle handle);
    }

    /// <summary>Common base for handles returned by <see cref="IModCanvas"/>.</summary>
    public abstract class ExtendedHandle
    {
        /// <summary>Logical id supplied by the spec.</summary>
        public string Id { get; }

        /// <summary>Implementation-specific opaque reference (e.g. GameObject).</summary>
        internal object Inner { get; }

        /// <summary>Initialize the handle with an id and opaque inner reference.</summary>
        protected ExtendedHandle(string id, object inner)
        {
            Id = id;
            Inner = inner;
        }
    }

    /// <summary>Handle to an Extended UI panel.</summary>
    public sealed class ExtendedPanelHandle : ExtendedHandle
    {
        internal ExtendedPanelHandle(string id, object inner) : base(id, inner) { }
    }

    /// <summary>Handle to an Extended UI label.</summary>
    public sealed class ExtendedLabelHandle : ExtendedHandle
    {
        internal ExtendedLabelHandle(string id, object inner) : base(id, inner) { }
    }

    /// <summary>Handle to an Extended UI button.</summary>
    public sealed class ExtendedButtonHandle : ExtendedHandle
    {
        internal ExtendedButtonHandle(string id, object inner) : base(id, inner) { }
    }

    /// <summary>Spec for a panel under <see cref="IModCanvas"/>.</summary>
    public sealed class PanelSpec
    {
        /// <summary>Logical panel id.</summary>
        public string Id { get; }
        /// <summary>Screen anchor for placement.</summary>
        public RectAnchor Anchor { get; }
        /// <summary>Panel size in pixels.</summary>
        public Vector2 Size { get; }
        /// <summary>Background fill color.</summary>
        public ColorRgba Background { get; }

        /// <summary>Construct a panel spec.</summary>
        public PanelSpec(string id, RectAnchor anchor, Vector2 size, ColorRgba background)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Anchor = anchor;
            Size = size;
            Background = background;
        }
    }

    /// <summary>Spec for a label under <see cref="IModCanvas"/>.</summary>
    public sealed class LabelSpec
    {
        /// <summary>Logical label id.</summary>
        public string Id { get; }
        /// <summary>Display text.</summary>
        public string Text { get; }
        /// <summary>Text color.</summary>
        public ColorRgba Color { get; }
        /// <summary>Semantic font size.</summary>
        public FontSize FontSize { get; }

        /// <summary>Construct a label spec.</summary>
        public LabelSpec(string id, string text, ColorRgba color, FontSize fontSize)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Text = text ?? string.Empty;
            Color = color;
            FontSize = fontSize;
        }
    }

    /// <summary>Spec for a button under <see cref="IModCanvas"/>.</summary>
    public sealed class ButtonSpec
    {
        /// <summary>Logical button id.</summary>
        public string Id { get; }
        /// <summary>Button label text.</summary>
        public string Label { get; }
        /// <summary>Click handler invoked when the button is pressed.</summary>
        public Action OnClick { get; }

        /// <summary>Construct a button spec.</summary>
        public ButtonSpec(string id, string label, Action onClick)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Label = label ?? string.Empty;
            OnClick = onClick ?? throw new ArgumentNullException(nameof(onClick));
        }
    }
}
