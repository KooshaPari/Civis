using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.UI.Models;

namespace DINOForge.Domains.UI.Registries
{
    /// <summary>
    /// Registry of UI theme definitions. Manages all themes registered across all packs,
    /// supporting theme selection and customization of UI appearance.
    /// </summary>
    public class ThemeRegistry
    {
        private readonly Dictionary<string, ThemeDefinition> _themes =
            new Dictionary<string, ThemeDefinition>(StringComparer.OrdinalIgnoreCase);

        private string _activeThemeId = string.Empty;

        /// <summary>
        /// All registered themes.
        /// </summary>
        public IReadOnlyList<ThemeDefinition> All => _themes.Values.ToList().AsReadOnly();

        /// <summary>
        /// Number of registered themes.
        /// </summary>
        public int Count => _themes.Count;

        /// <summary>
        /// Gets or sets the currently active theme identifier.
        /// </summary>
        public string ActiveThemeId
        {
            get => _activeThemeId;
            set => _activeThemeId = value ?? string.Empty;
        }

        /// <summary>
        /// Gets the currently active theme definition, or null if no theme is active.
        /// </summary>
        public ThemeDefinition? ActiveTheme
        {
            get
            {
                if (string.IsNullOrEmpty(_activeThemeId))
                    return null;

                _themes.TryGetValue(_activeThemeId, out ThemeDefinition? theme);
                return theme;
            }
        }

        /// <summary>
        /// Creates a new theme registry with a default dark theme pre-registered.
        /// </summary>
        public ThemeRegistry()
        {
            RegisterDefaults();
        }

        /// <summary>
        /// Retrieve a theme definition by its identifier.
        /// </summary>
        /// <param name="id">Theme identifier (e.g. "dark-theme", "faction-blue").</param>
        /// <returns>The matching theme definition.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no theme with the given id exists.</exception>
        public ThemeDefinition GetTheme(string id)
        {
            if (_themes.TryGetValue(id, out ThemeDefinition? theme))
                return theme;

            throw new KeyNotFoundException($"No theme registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve a theme definition by its identifier.
        /// </summary>
        /// <param name="id">Theme identifier.</param>
        /// <param name="theme">The matching theme definition, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetTheme(string id, out ThemeDefinition? theme)
        {
            return _themes.TryGetValue(id, out theme);
        }

        /// <summary>
        /// Check if a theme with the given identifier is registered.
        /// </summary>
        /// <param name="id">Theme identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _themes.ContainsKey(id);
        }

        /// <summary>
        /// Register a theme definition.
        /// </summary>
        /// <param name="theme">The theme definition to register.</param>
        public void Register(ThemeDefinition theme)
        {
            if (theme == null) throw new ArgumentNullException(nameof(theme));
            if (string.IsNullOrWhiteSpace(theme.Id)) throw new ArgumentException("Theme ID cannot be empty.", nameof(theme));
            _themes[theme.Id] = theme;
        }

        /// <summary>
        /// Unregister a theme by identifier. Cannot unregister the active theme.
        /// </summary>
        /// <param name="id">Theme identifier.</param>
        /// <returns>True if a theme was removed; false if not found or is the active theme.</returns>
        public bool Unregister(string id)
        {
            if (string.Equals(id, _activeThemeId, StringComparison.OrdinalIgnoreCase))
                return false; // Cannot unregister active theme

            return _themes.Remove(id);
        }

        /// <summary>
        /// Set the active theme by identifier.
        /// </summary>
        /// <param name="id">Theme identifier to activate.</param>
        /// <returns>True if the theme was activated; false if not found.</returns>
        public bool SetActiveTheme(string id)
        {
            if (!_themes.ContainsKey(id))
                return false;

            _activeThemeId = id;
            return true;
        }

        private void RegisterDefaults()
        {
            ThemeDefinition darkTheme = new ThemeDefinition(
                "dark-theme",
                "Dark Theme",
                "#FFFFFF",
                "#999999",
                "#FF6B00")
            {
                Description = "Default dark theme with white text and orange accents.",
                BackgroundColor = "#0D0D0D",
                TextColor = "#FFFFFF",
                FontFamily = "Arial",
                BorderRadius = 4
            };
            Register(darkTheme);

            ThemeDefinition lightTheme = new ThemeDefinition(
                "light-theme",
                "Light Theme",
                "#333333",
                "#666666",
                "#0066CC")
            {
                Description = "Light theme with dark text and blue accents.",
                BackgroundColor = "#F5F5F5",
                TextColor = "#333333",
                FontFamily = "Arial",
                BorderRadius = 4
            };
            Register(lightTheme);

            // Set dark theme as default
            _activeThemeId = "dark-theme";
        }
    }
}
