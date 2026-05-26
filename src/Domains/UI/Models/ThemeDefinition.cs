using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;
using DINOForge.SDK.Validation;

namespace DINOForge.Domains.UI.Models
{
    /// <summary>
    /// Defines a UI theme that controls the overall appearance and styling of the mod menu and HUD elements.
    /// Themes specify colors, fonts, border radius, and other visual properties.
    /// </summary>
    public class ThemeDefinition : IValidatable
    {
        /// <summary>
        /// Unique identifier for this theme (e.g. "dark-theme", "light-theme", "faction-blue").
        /// </summary>
        public string Id { get; set; } = string.Empty;

        /// <summary>
        /// Display name for this theme.
        /// </summary>
        public string Name { get; set; } = string.Empty;

        /// <summary>
        /// Primary color in hex format (e.g. "#FFFFFF"). Used for main UI elements.
        /// </summary>
        public string PrimaryColor { get; set; } = string.Empty;

        /// <summary>
        /// Secondary color in hex format (e.g. "#666666"). Used for secondary UI elements.
        /// </summary>
        public string SecondaryColor { get; set; } = string.Empty;

        /// <summary>
        /// Accent color in hex format (e.g. "#FF6B00"). Used for highlights and interactive elements.
        /// </summary>
        public string AccentColor { get; set; } = string.Empty;

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

        private static readonly Regex HexColorRegex = new Regex(
            "^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$",
            RegexOptions.Compiled);

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the UIContentLoader deserialize site.
        /// Rejects blank id, blank name, and any malformed hex color.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "ThemeDefinition 'id' is required.", "non_empty"));

            if (string.IsNullOrWhiteSpace(Name))
                errors.Add(new ValidationError("name", "ThemeDefinition 'name' is required.", "non_empty"));

            ValidateHex(PrimaryColor, "primary_color", errors);
            ValidateHex(SecondaryColor, "secondary_color", errors);
            ValidateHex(AccentColor, "accent_color", errors);

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }

        private static void ValidateHex(string value, string path, List<ValidationError> errors)
        {
            if (string.IsNullOrEmpty(value))
                return;
            if (!HexColorRegex.IsMatch(value))
                errors.Add(new ValidationError(path, $"ThemeDefinition '{path}' must be a hex color (e.g. #RRGGBB).", "format"));
        }
    }
}
