using System;

namespace DINOForge.Domains.UI.Models
{
    /// <summary>
    /// Defines a UI theme that controls the overall appearance and styling of the mod menu and HUD elements.
    /// Themes specify colors, fonts, border radius, and other visual properties.
    /// </summary>
    public class ThemeDefinition
    {
        /// <summary>
        /// Unique identifier for this theme (e.g. "dark-theme", "light-theme", "faction-blue").
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display name for this theme.
        /// </summary>
        public string Name { get; set; }

        /// <summary>
        /// Primary color in hex format (e.g. "#FFFFFF"). Used for main UI elements.
        /// </summary>
        public string PrimaryColor { get; set; }

        /// <summary>
        /// Secondary color in hex format (e.g. "#666666"). Used for secondary UI elements.
        /// </summary>
        public string SecondaryColor { get; set; }

        /// <summary>
        /// Accent color in hex format (e.g. "#FF6B00"). Used for highlights and interactive elements.
        /// </summary>
        public string AccentColor { get; set; }

        /// <summary>
        /// Font family name (e.g. "Arial", "Courier New"). Empty string means use default.
        /// </summary>
        public string FontFamily { get; set; } = string.Empty;

        /// <summary>
        /// Border radius in pixels. Controls how rounded corners are on UI elements.
        /// </summary>
        public int BorderRadius { get; set; } = 4;

        /// <summary>
        /// Optional description of this theme.
        /// </summary>
        public string Description { get; set; } = string.Empty;

        /// <summary>
        /// Optional background color for menus and panels in hex format.
        /// </summary>
        public string BackgroundColor { get; set; } = "#0D0D0D";

        /// <summary>
        /// Optional text color for default text in hex format.
        /// </summary>
        public string TextColor { get; set; } = "#FFFFFF";

        /// <summary>
        /// Initializes a new theme definition with default values.
        /// </summary>
        public ThemeDefinition()
        {
            Id = string.Empty;
            Name = string.Empty;
            PrimaryColor = "#FFFFFF";
            SecondaryColor = "#666666";
            AccentColor = "#FF6B00";
        }

        /// <summary>
        /// Initializes a new theme definition with core properties.
        /// </summary>
        public ThemeDefinition(
            string id,
            string name,
            string primaryColor,
            string secondaryColor,
            string accentColor)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Name = name ?? throw new ArgumentNullException(nameof(name));
            PrimaryColor = primaryColor ?? throw new ArgumentNullException(nameof(primaryColor));
            SecondaryColor = secondaryColor ?? throw new ArgumentNullException(nameof(secondaryColor));
            AccentColor = accentColor ?? throw new ArgumentNullException(nameof(accentColor));
        }
    }
}
