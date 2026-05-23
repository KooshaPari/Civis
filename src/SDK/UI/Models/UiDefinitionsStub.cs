// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — minimal stub definition types referenced by
// Extended UI interfaces. These deliberately mirror the public surface of
// DINOForge.Domains.UI.Models so Phase 2 can either: (a) remap Domains types
// onto these, or (b) move Domains types here entirely. SDK does NOT
// project-reference Domains/UI to keep the layering one-way.

using System;
using System.Collections.Generic;

namespace DINOForge.SDK.UI.Models
{
    /// <summary>
    /// SDK-side mirror of a HUD element definition. Phase 2 will reconcile
    /// with <c>DINOForge.Domains.UI.Models.HudElementDefinition</c>.
    /// </summary>
    public sealed class HudElementDefinition
    {
        /// <summary>Unique element id within the pack.</summary>
        public string Id { get; set; } = string.Empty;
        /// <summary>Element type token (e.g. health_bar, resource_counter).</summary>
        public string Type { get; set; } = "custom";
        /// <summary>Anchor token (e.g. top_left, top_right, center).</summary>
        public string Position { get; set; } = "top_left";
        /// <summary>Element width in pixels.</summary>
        public int Width { get; set; } = 200;
        /// <summary>Element height in pixels.</summary>
        public int Height { get; set; } = 50;
        /// <summary>Game states in which the element should render.</summary>
        public IReadOnlyList<string> VisibleIn { get; set; } = new List<string>(); // public-mutable-ok: serializer compatibility requires list-backed property shape
        /// <summary>Per-property color overrides (property name → hex).</summary>
        public IReadOnlyDictionary<string, string> ColorOverrides { get; set; } = new Dictionary<string, string>(StringComparer.Ordinal); // public-mutable-ok: serializer compatibility requires dictionary-backed property shape
        /// <summary>Element opacity 0..1.</summary>
        public float Opacity { get; set; } = 1.0f;
        /// <summary>Free-form description for tooling.</summary>
        public string Description { get; set; } = string.Empty;
    }

    /// <summary>
    /// SDK-side mirror of a theme definition. Phase 2 will reconcile with
    /// <c>DINOForge.Domains.UI.Models.ThemeDefinition</c>.
    /// </summary>
    public sealed class ThemeDefinition
    {
        /// <summary>Unique theme id.</summary>
        public string Id { get; set; } = string.Empty;
        /// <summary>Display name.</summary>
        public string Name { get; set; } = string.Empty;
        /// <summary>Primary color (hex).</summary>
        public string PrimaryColor { get; set; } = "#FFFFFF";
        /// <summary>Secondary color (hex).</summary>
        public string SecondaryColor { get; set; } = "#666666";
        /// <summary>Accent color (hex).</summary>
        public string AccentColor { get; set; } = "#FF6B00";
        /// <summary>Font family name; empty = default.</summary>
        public string FontFamily { get; set; } = string.Empty;
        /// <summary>Border radius in pixels.</summary>
        public int BorderRadius { get; set; } = 4;
        /// <summary>Panel background color (hex).</summary>
        public string BackgroundColor { get; set; } = "#0D0D0D";
        /// <summary>Default text color (hex).</summary>
        public string TextColor { get; set; } = "#FFFFFF";
        /// <summary>Free-form description for tooling.</summary>
        public string Description { get; set; } = string.Empty;
    }
}
