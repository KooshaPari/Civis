using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using DINOForge.SDK;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Domains.UI
{
    /// <summary>
    /// Loads UI definitions from pack directories into UI registries.
    /// Handles hud_elements/, menus/, and themes/ subdirectories.
    /// </summary>
    public class UIContentLoader
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
                    string yaml = File.ReadAllText(file);
                    HudElementWrapper wrapper = _deserializer.Deserialize<HudElementWrapper>(yaml);
                    if (wrapper?.Elements != null)
                    {
                        foreach (HudElementDefinition element in wrapper.Elements)
                        {
                            _hudElementRegistry.Register(element);
                        }
                    }
                    else if (wrapper?.Element != null)
                    {
                        _hudElementRegistry.Register(wrapper.Element);
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
                    string yaml = File.ReadAllText(file);
                    MenuWrapper wrapper = _deserializer.Deserialize<MenuWrapper>(yaml);
                    if (wrapper?.Menus != null)
                    {
                        foreach (MenuDefinition menu in wrapper.Menus)
                        {
                            _menuRegistry.Register(menu);
                        }
                    }
                    else if (wrapper?.Menu != null)
                    {
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
                    string yaml = File.ReadAllText(file);
                    ThemeWrapper wrapper = _deserializer.Deserialize<ThemeWrapper>(yaml);
                    if (wrapper?.Themes != null)
                    {
                        foreach (ThemeDefinition theme in wrapper.Themes)
                        {
                            _themeRegistry.Register(theme);
                        }
                    }
                    else if (wrapper?.Theme != null)
                    {
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
        /// </summary>
        private class HudElementWrapper
        {
            public List<HudElementDefinition> Elements { get; set; } = new List<HudElementDefinition>();
            public HudElementDefinition? Element { get; set; }
        }

        /// <summary>
        /// YAML wrapper for menus array or single menu.
        /// </summary>
        private class MenuWrapper
        {
            public List<MenuDefinition> Menus { get; set; } = new List<MenuDefinition>();
            public MenuDefinition? Menu { get; set; }
        }

        /// <summary>
        /// YAML wrapper for themes array or single theme.
        /// </summary>
        private class ThemeWrapper
        {
            public List<ThemeDefinition> Themes { get; set; } = new List<ThemeDefinition>();
            public ThemeDefinition? Theme { get; set; }
        }
    }
}
