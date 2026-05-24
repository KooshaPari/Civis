using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using DINOForge.SDK;
using DINOForge.SDK.IO;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Domains.UI
{
    /// <summary>
    /// Loads UI definitions from pack directories into UI registries.
    /// Handles hud_elements/, menus/, and themes/ subdirectories.
    /// </summary>
    public sealed class UIContentLoader
    {
        private readonly HudElementRegistry _hudElementRegistry;
        private readonly MenuRegistry _menuRegistry;
        private readonly ThemeRegistry _themeRegistry;
        private readonly IDeserializer _deserializer;

        /// <summary>
        /// Initializes a new UI content loader with the provided registries.
        /// </summary>
        public UIContentLoader(
            HudElementRegistry hudElementRegistry,
            MenuRegistry menuRegistry,
            ThemeRegistry themeRegistry)
        {
            _hudElementRegistry = hudElementRegistry ?? throw new ArgumentNullException(nameof(hudElementRegistry));
            _menuRegistry = menuRegistry ?? throw new ArgumentNullException(nameof(menuRegistry));
            _themeRegistry = themeRegistry ?? throw new ArgumentNullException(nameof(themeRegistry));

            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .Build();
        }

        /// <summary>
        /// Load all UI definitions from a pack directory.
        /// </summary>
        /// <param name="packDir">The root directory of the pack.</param>
        /// <param name="packId">The pack identifier (for logging).</param>
        public void LoadPack(string packDir, string packId)
        {
            if (!Directory.Exists(packDir))
                throw new DirectoryNotFoundException($"Pack directory not found: {packDir}");

            LoadHudElements(Path.Combine(packDir, "hud_elements"), packId);
            LoadMenus(Path.Combine(packDir, "menus"), packId);
            LoadThemes(Path.Combine(packDir, "themes"), packId);
        }

        private void LoadHudElements(string elementsDir, string packId)
        {
            if (!Directory.Exists(elementsDir))
                return;

            string[] files = Directory.GetFiles(elementsDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = SafeFileIO.ReadText(file);
                    HudElementWrapper wrapper = _deserializer.Deserialize<HudElementWrapper>(yaml);
                    if (wrapper?.HudElements != null && wrapper.HudElements.Count > 0)
                    {
                        foreach (HudElementDefinition element in wrapper.HudElements)
                        {
                            // Task #319 — IValidatable semantic check at the deserialize site.
                            JsonGuard.ValidateOrThrow(element, file);
                            _hudElementRegistry.Register(element);
                        }
                    }
                    else if (wrapper?.HudElement != null)
                    {
                        JsonGuard.ValidateOrThrow(wrapper.HudElement, file);
                        _hudElementRegistry.Register(wrapper.HudElement);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load HUD element from {file} in pack '{packId}'.", ex);
                }
            }
        }

        private void LoadMenus(string menusDir, string packId)
        {
            if (!Directory.Exists(menusDir))
                return;

            string[] files = Directory.GetFiles(menusDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = SafeFileIO.ReadText(file);
                    MenuWrapper wrapper = _deserializer.Deserialize<MenuWrapper>(yaml);
                    if (wrapper?.Menus != null && wrapper.Menus.Count > 0)
                    {
                        foreach (MenuDefinition menu in wrapper.Menus)
                        {
                            // Task #319 — IValidatable semantic check at the deserialize site.
                            JsonGuard.ValidateOrThrow(menu, file);
                            _menuRegistry.Register(menu);
                        }
                    }
                    else if (wrapper?.Menu != null)
                    {
                        JsonGuard.ValidateOrThrow(wrapper.Menu, file);
                        _menuRegistry.Register(wrapper.Menu);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load menu from {file} in pack '{packId}'.", ex);
                }
            }
        }

        private void LoadThemes(string themesDir, string packId)
        {
            if (!Directory.Exists(themesDir))
                return;

            string[] files = Directory.GetFiles(themesDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = SafeFileIO.ReadText(file);
                    ThemeWrapper wrapper = _deserializer.Deserialize<ThemeWrapper>(yaml);
                    if (wrapper?.Themes != null && wrapper.Themes.Count > 0)
                    {
                        foreach (ThemeDefinition theme in wrapper.Themes)
                        {
                            // Task #319 — IValidatable semantic check at the deserialize site.
                            JsonGuard.ValidateOrThrow(theme, file);
                            _themeRegistry.Register(theme);
                        }
                    }
                    else if (wrapper?.Theme != null)
                    {
                        JsonGuard.ValidateOrThrow(wrapper.Theme, file);
                        _themeRegistry.Register(wrapper.Theme);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load theme from {file} in pack '{packId}'.", ex);
                }
            }
        }

        /// <summary>
        /// YAML wrapper for HUD elements array or single element.
        /// Property names are mapped via UnderscoredNamingConvention so
        /// <c>HudElements</c> → <c>hud_elements</c> (matches pack manifest top-level key).
        /// </summary>
        private class HudElementWrapper
        {
            public List<HudElementDefinition> HudElements { get; set; } = new List<HudElementDefinition>(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet
            public HudElementDefinition? HudElement { get; set; }
        }

        /// <summary>
        /// YAML wrapper for menus array or single menu.
        /// </summary>
        private class MenuWrapper
        {
            public List<MenuDefinition> Menus { get; set; } = new List<MenuDefinition>(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet
            public MenuDefinition? Menu { get; set; }
        }

        /// <summary>
        /// YAML wrapper for themes array or single theme.
        /// </summary>
        private class ThemeWrapper
        {
            public List<ThemeDefinition> Themes { get; set; } = new List<ThemeDefinition>(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet
            public ThemeDefinition? Theme { get; set; }
        }
    }
}
