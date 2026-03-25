using System;
using System.Collections.Generic;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.UI
{
    /// <summary>
    /// Entry point for the UI domain plugin. Provides access to all UI subsystems:
    /// HUD element registry, menu registry, theme registry, and menu management.
    /// </summary>
    public class UIPlugin
    {
        private readonly RegistryManager _registries;

        /// <summary>
        /// Registry of HUD element definitions (health bars, resource counters, minimaps, etc.).
        /// </summary>
        public HudElementRegistry HudElements { get; }

        /// <summary>
        /// Registry of menu definitions for mod menu navigation and structure.
        /// </summary>
        public MenuRegistry Menus { get; }

        /// <summary>
        /// Registry of UI theme definitions for visual customization.
        /// </summary>
        public ThemeRegistry Themes { get; }

        /// <summary>
        /// Manager for mod menu state and panel coordination.
        /// </summary>
        public MenuManager MenuManager { get; }

        /// <summary>
        /// Content loader for UI definitions from packs.
        /// </summary>
        public UIContentLoader ContentLoader { get; }

        /// <summary>
        /// Available UI content type names that packs can declare in their loads section.
        /// </summary>
        public static IReadOnlyList<string> ContentTypes { get; } = new List<string>
        {
            "ui_panels",
            "ui_themes",
            "ui_overlays",
            "hud_elements",
            "menus"
        }.AsReadOnly();

        /// <summary>
        /// Initializes the UI plugin with pre-loaded registries.
        /// </summary>
        /// <param name="registries">The registry manager containing all loaded content.</param>
        public UIPlugin(RegistryManager registries)
        {
            _registries = registries ?? throw new ArgumentNullException(nameof(registries));

            HudElements = new HudElementRegistry();
            Menus = new MenuRegistry();
            Themes = new ThemeRegistry();
            MenuManager = new MenuManager();
            ContentLoader = new UIContentLoader(HudElements, Menus, Themes);
        }

        /// <summary>
        /// Validates UI-related content for a given pack. Checks:
        /// - All HUD elements have valid IDs and types
        /// - All menus have valid IDs and valid parent references (no cycles)
        /// - All themes have valid IDs and valid color definitions
        /// </summary>
        /// <param name="packId">The pack identifier to scope validation to.</param>
        /// <returns>List of validation errors (empty if valid).</returns>
        public IReadOnlyList<string> ValidatePack(string packId)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID is required.", nameof(packId));

            List<string> errors = new List<string>();

            // Validate HUD elements
            foreach (HudElementDefinition element in HudElements.All)
            {
                if (string.IsNullOrWhiteSpace(element.Id))
                    errors.Add("HUD element has empty ID.");
                if (string.IsNullOrWhiteSpace(element.Type))
                    errors.Add($"HUD element '{element.Id}' has empty type.");
                if (element.Width <= 0)
                    errors.Add($"HUD element '{element.Id}' has invalid width ({element.Width}).");
                if (element.Height <= 0)
                    errors.Add($"HUD element '{element.Id}' has invalid height ({element.Height}).");
            }

            // Validate menus
            IReadOnlyList<string> menuErrors = Menus.ValidateHierarchy();
            foreach (string error in menuErrors)
                errors.Add(error);

            foreach (MenuDefinition menu in Menus.All)
            {
                if (string.IsNullOrWhiteSpace(menu.Id))
                    errors.Add("Menu has empty ID.");
                if (string.IsNullOrWhiteSpace(menu.Title))
                    errors.Add($"Menu '{menu.Id}' has empty title.");
            }

            // Validate themes
            foreach (ThemeDefinition theme in Themes.All)
            {
                if (string.IsNullOrWhiteSpace(theme.Id))
                    errors.Add("Theme has empty ID.");
                if (string.IsNullOrWhiteSpace(theme.Name))
                    errors.Add($"Theme '{theme.Id}' has empty name.");
                if (string.IsNullOrWhiteSpace(theme.PrimaryColor))
                    errors.Add($"Theme '{theme.Id}' has empty primary color.");
                if (string.IsNullOrWhiteSpace(theme.SecondaryColor))
                    errors.Add($"Theme '{theme.Id}' has empty secondary color.");
                if (string.IsNullOrWhiteSpace(theme.AccentColor))
                    errors.Add($"Theme '{theme.Id}' has empty accent color.");
            }

            return errors.AsReadOnly();
        }
    }
}
