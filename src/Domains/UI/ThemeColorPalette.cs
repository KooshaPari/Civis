using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.Domains.UI
{
    // Alias to fix naming conflict
    using ThemeColor = ThemeThemeColor;
    /// <summary>
    /// Represents a faction-specific color palette with WCAG AA compliance validation.
    /// Supports 12+ colors per faction (primary, secondary, accent, text, background, etc.)
    /// </summary>
    public sealed class ThemeThemeColorPalette
    {
        /// <summary>
        /// Unique identifier for this palette (e.g., "republic", "cis").
        /// </summary>
        public string? Id { get; set; }

        /// <summary>
        /// Human-readable name (e.g., "Galactic Republic").
        /// </summary>
        public string? Name { get; set; }

        /// <summary>
        /// Associated faction ID (e.g., "republic", "cis-droid-army").
        /// </summary>
        public string? FactionId { get; set; }

        /// <summary>
        /// Primary color (button backgrounds, key UI elements).
        /// </summary>
        public ThemeColor? Primary { get; set; }

        /// <summary>
        /// Secondary color (accent borders, hover states).
        /// </summary>
        public ThemeColor? Secondary { get; set; }

        /// <summary>
        /// Accent color (highlights, special states).
        /// </summary>
        public ThemeColor? Accent { get; set; }

        /// <summary>
        /// Text color (body text, labels).
        /// </summary>
        public ThemeColor? Text { get; set; }

        /// <summary>
        /// Background color (panels, windows).
        /// </summary>
        public ThemeColor? Background { get; set; }

        /// <summary>
        /// Success state color (green/positive).
        /// </summary>
        public ThemeColor? Success { get; set; }

        /// <summary>
        /// Warning state color (yellow/caution).
        /// </summary>
        public ThemeColor? Warning { get; set; }

        /// <summary>
        /// Danger state color (red/critical).
        /// </summary>
        public ThemeColor? Danger { get; set; }

        /// <summary>
        /// Hover/highlighted state color.
        /// </summary>
        public ThemeColor? Hover { get; set; }

        /// <summary>
        /// Disabled/inactive state color.
        /// </summary>
        public ThemeColor? Disabled { get; set; }

        /// <summary>
        /// Neutral/muted state color.
        /// </summary>
        public ThemeColor? Neutral { get; set; }

        /// <summary>
        /// Button border color.
        /// </summary>
        public ThemeColor? ButtonBorder { get; set; }

        /// <summary>
        /// Gets a color by type name.
        /// </summary>
        /// <param name="colorType">ThemeColor type key (e.g., "primary", "secondary", "text").</param>
        /// <returns>The requested color, or default if not found.</returns>
        public ThemeColor? GetThemeColor(string colorType)
        {
            return colorType.ToLowerInvariant() switch
            {
                "primary" => Primary,
                "secondary" => Secondary,
                "accent" => Accent,
                "text" => Text,
                "background" => Background,
                "success" => Success,
                "warning" => Warning,
                "danger" => Danger,
                "hover" => Hover,
                "disabled" => Disabled,
                "neutral" => Neutral,
                "button_border" => ButtonBorder,
                _ => throw new ArgumentException($"Unknown color type: {colorType}", nameof(colorType))
            };
        }

        /// <summary>
        /// Validates WCAG AA contrast ratios (4.5:1 minimum) for all text/background pairs.
        /// </summary>
        /// <returns>List of validation errors (empty if all valid).</returns>
        public IReadOnlyList<string> ValidateContrast()
        {
            var errors = new List<string>();

            // Validate critical text pairs (text on background)
            if (Text != null && Background != null)
                ValidateTextContrast(errors, Text, Background, "Text on Background");
            if (Text != null && Hover != null)
                ValidateTextContrast(errors, Text, Hover, "Text on Hover State");
            if (Text != null && Primary != null)
                ValidateTextContrast(errors, Text, Primary, "Text on Primary");
            if (Disabled != null && Background != null)
                ValidateTextContrast(errors, Disabled, Background, "Disabled Text on Background");

            // Validate button pairs
            if (Text != null && Secondary != null)
                ValidateTextContrast(errors, Text, Secondary, "Text on Secondary");
            if (Neutral != null && Background != null)
                ValidateTextContrast(errors, Neutral, Background, "Neutral on Background");

            return errors.AsReadOnly();
        }

        private static void ValidateTextContrast(List<string> errors, ThemeColor text, ThemeColor background, string label)
        {
            double ratio = CalculateContrastRatio(text, background);
            if (ratio < 4.5)
            {
                errors.Add($"{label}: {ratio:F2}:1 (WCAG AA requires 4.5:1 minimum)");
            }
        }

        /// <summary>
        /// Calculates the WCAG contrast ratio between two colors (1.0 to 21.0 scale).
        /// </summary>
        /// <param name="color1">First color.</param>
        /// <param name="color2">Second color.</param>
        /// <returns>Contrast ratio (1.0 = same, 21.0 = maximum).</returns>
        public static double CalculateContrastRatio(ThemeColor color1, ThemeColor color2)
        {
            double l1 = GetRelativeLuminance(color1);
            double l2 = GetRelativeLuminance(color2);

            double lighter = Math.Max(l1, l2);
            double darker = Math.Min(l1, l2);

            return (lighter + 0.05) / (darker + 0.05);
        }

        /// <summary>
        /// Gets the relative luminance of a color (WCAG formula).
        /// </summary>
        private static double GetRelativeLuminance(ThemeColor color)
        {
            double r = Linearize(color.R / 255.0);
            double g = Linearize(color.G / 255.0);
            double b = Linearize(color.B / 255.0);

            return 0.2126 * r + 0.7152 * g + 0.0722 * b;
        }

        private static double Linearize(double c)
        {
            return c <= 0.03928 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4);
        }
    }

    /// <summary>
    /// Simple RGB color representation for themes.
    /// Renamed to avoid conflict with UnityEngine.ThemeColor.
    /// </summary>
    public sealed class ThemeThemeColor
    {
        public byte R { get; set; }
        public byte G { get; set; }
        public byte B { get; set; }
        public byte A { get; set; } = 255;

        /// <summary>
        /// Parses a hex color string (e.g., "#1A3A6B") to ThemeThemeColor.
        /// </summary>
        /// <param name="hex">Hex color string (#RRGGBB or #RRGGBBAA).</param>
        /// <returns>Parsed ThemeThemeColor.</returns>
        public static ThemeThemeColor FromHex(string hex)
        {
            if (string.IsNullOrWhiteSpace(hex) || !hex.StartsWith("#"))
                throw new ArgumentException("Hex color must start with #", nameof(hex));

            hex = hex.Substring(1); // Remove #

            if (hex.Length != 6 && hex.Length != 8)
                throw new ArgumentException("Hex color must be 6 or 8 characters long", nameof(hex));

            byte r = byte.Parse(hex.Substring(0, 2), System.Globalization.NumberStyles.HexNumber);
            byte g = byte.Parse(hex.Substring(2, 2), System.Globalization.NumberStyles.HexNumber);
            byte b = byte.Parse(hex.Substring(4, 2), System.Globalization.NumberStyles.HexNumber);
            byte a = 255;

            if (hex.Length == 8)
                a = byte.Parse(hex.Substring(6, 2), System.Globalization.NumberStyles.HexNumber);

            return new ThemeColor { R = r, G = g, B = b, A = a };
        }

        /// <summary>
        /// Converts color to hex string (#RRGGBB).
        /// </summary>
        public string ToHex()
        {
            return $"#{R:X2}{G:X2}{B:X2}";
        }

        /// <summary>
        /// Creates a brighter variant of this color (for hover states).
        /// </summary>
        /// <param name="factor">Brightness factor (1.0 = no change, 1.2 = 20% brighter).</param>
        public ThemeColor Brighten(double factor = 1.2)
        {
            return new ThemeColor
            {
                R = (byte)Math.Min(255, (int)(R * factor)),
                G = (byte)Math.Min(255, (int)(G * factor)),
                B = (byte)Math.Min(255, (int)(B * factor)),
                A = A
            };
        }

        /// <summary>
        /// Creates a darker variant of this color (for pressed states).
        /// </summary>
        /// <param name="factor">Darkness factor (1.0 = no change, 0.8 = 20% darker).</param>
        public ThemeColor Darken(double factor = 0.8)
        {
            return new ThemeColor
            {
                R = (byte)(R * factor),
                G = (byte)(G * factor),
                B = (byte)(B * factor),
                A = A
            };
        }
    }
}
