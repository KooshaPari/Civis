// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — minimal stub primitive types so Native/Extended
// interfaces compile against netstandard2.0 without dragging in UnityEngine
// types. Domains/UI may adopt these later, or remap onto richer types in
// Phase 2.

using System;

namespace DINOForge.SDK.UI.Models
{
    /// <summary>
    /// Minimal 2D float vector used by SDK UI specs. Avoids dragging in
    /// UnityEngine.Vector2 — SDK is netstandard2.0.
    /// </summary>
    public readonly struct Vector2 : IEquatable<Vector2>
    {
        /// <summary>X component.</summary>
        public float X { get; }
        /// <summary>Y component.</summary>
        public float Y { get; }
        /// <summary>Construct a Vector2 from components.</summary>
        public Vector2(float x, float y) { X = x; Y = y; }
        /// <inheritdoc/>
        public bool Equals(Vector2 other) => X == other.X && Y == other.Y;
        /// <inheritdoc/>
        public override bool Equals(object? obj) => obj is Vector2 v && Equals(v);
        /// <inheritdoc/>
        public override int GetHashCode() => unchecked((X.GetHashCode() * 397) ^ Y.GetHashCode());
        /// <inheritdoc/>
        public override string ToString() => $"({X}, {Y})";
    }

    /// <summary>
    /// RGBA color (0..1 floats). Stub for SDK use; Domains/UI hex strings can
    /// convert into this on adoption.
    /// </summary>
    public readonly struct ColorRgba : IEquatable<ColorRgba>
    {
        /// <summary>Red component (0..1).</summary>
        public float R { get; }
        /// <summary>Green component (0..1).</summary>
        public float G { get; }
        /// <summary>Blue component (0..1).</summary>
        public float B { get; }
        /// <summary>Alpha component (0..1).</summary>
        public float A { get; }
        /// <summary>Construct an RGBA color from 0..1 components.</summary>
        public ColorRgba(float r, float g, float b, float a) { R = r; G = g; B = b; A = a; }
        /// <summary>Opaque white.</summary>
        public static ColorRgba White => new ColorRgba(1f, 1f, 1f, 1f);
        /// <summary>Opaque black.</summary>
        public static ColorRgba Black => new ColorRgba(0f, 0f, 0f, 1f);
        /// <summary>Fully transparent.</summary>
        public static ColorRgba Transparent => new ColorRgba(0f, 0f, 0f, 0f);
        /// <inheritdoc/>
        public bool Equals(ColorRgba other) => R == other.R && G == other.G && B == other.B && A == other.A;
        /// <inheritdoc/>
        public override bool Equals(object? obj) => obj is ColorRgba c && Equals(c);
        /// <inheritdoc/>
        public override int GetHashCode()
        {
            unchecked
            {
                int h = R.GetHashCode();
                h = (h * 397) ^ G.GetHashCode();
                h = (h * 397) ^ B.GetHashCode();
                h = (h * 397) ^ A.GetHashCode();
                return h;
            }
        }
    }

    /// <summary>
    /// Semantic font size enum. Renderer maps these to pixel values via theme.
    /// </summary>
    public enum FontSize
    {
        /// <summary>Smallest readable size; for dense detail.</summary>
        Small,
        /// <summary>Default body text size.</summary>
        Medium,
        /// <summary>Larger body / emphasis size.</summary>
        Large,
        /// <summary>Section heading size.</summary>
        Heading,
        /// <summary>Top-level title size.</summary>
        Title,
    }

    /// <summary>
    /// 9-point screen anchor for HUD/panel placement.
    /// </summary>
    public enum RectAnchor
    {
        /// <summary>Anchor to top-left corner.</summary>
        TopLeft,
        /// <summary>Anchor to top edge, horizontally centered.</summary>
        TopCenter,
        /// <summary>Anchor to top-right corner.</summary>
        TopRight,
        /// <summary>Anchor to left edge, vertically centered.</summary>
        MiddleLeft,
        /// <summary>Anchor to center of screen.</summary>
        MiddleCenter,
        /// <summary>Anchor to right edge, vertically centered.</summary>
        MiddleRight,
        /// <summary>Anchor to bottom-left corner.</summary>
        BottomLeft,
        /// <summary>Anchor to bottom edge, horizontally centered.</summary>
        BottomCenter,
        /// <summary>Anchor to bottom-right corner.</summary>
        BottomRight,
    }
}
