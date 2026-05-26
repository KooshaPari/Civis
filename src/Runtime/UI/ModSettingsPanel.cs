#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using BepInEx;
using BepInEx.Configuration;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// IMGUI panel that wraps the BepInEx ConfigEntry system to render per-mod settings.
    /// Auto-discovers ConfigEntry instances from loaded BepInEx plugins and renders
    /// appropriate controls based on value type (bool toggle, int slider, string field, enum dropdown).
    /// </summary>
    public class ModSettingsPanel : MonoBehaviour, IModSettingsHost
    {
        private bool _visible;
        private Rect _windowRect = new Rect(540, 20, 450, 550);
        private Vector2 _settingsScroll;
        private string _selectedPluginGuid = "";

        private readonly Dictionary<string, PluginSettingsInfo> _pluginSettings =
            new Dictionary<string, PluginSettingsInfo>(StringComparer.OrdinalIgnoreCase);

        /// <summary>Whether the settings panel is currently visible.</summary>
        public bool IsVisible => _visible;

        /// <summary>The GUID of the currently selected plugin, or empty if none.</summary>
        public string SelectedPluginGuid => _selectedPluginGuid;

        /// <summary>Number of discovered plugins with settings.</summary>
        public int PluginCount => _pluginSettings.Count;

        /// <summary>
        /// Shows or hides the settings panel.
        /// </summary>
        /// <param name="visible">Whether to show the panel.</param>
        public void SetVisible(bool visible) => _visible = visible;

        /// <inheritdoc />
        public void Show() => _visible = true;

        /// <inheritdoc />
        public void Hide() => _visible = false;

        /// <inheritdoc />
        public void Toggle() => _visible = !_visible;

        /// <summary>
        /// Discovers and caches config entries from all loaded BepInEx plugins.
        /// Call this after BepInEx finishes loading plugins.
        /// </summary>
        public void DiscoverSettings()
        {
            _pluginSettings.Clear();

            // Find all loaded BepInEx plugins
            BaseUnityPlugin[] plugins = FindObjectsOfType<BaseUnityPlugin>();

            foreach (BaseUnityPlugin plugin in plugins)
            {
                BepInPlugin? pluginAttr = plugin.GetType()
                    .GetCustomAttributes(typeof(BepInPlugin), false)
                    .FirstOrDefault() as BepInPlugin;

                if (pluginAttr == null) continue;

                string guid = pluginAttr.GUID;
                string name = pluginAttr.Name;

                ConfigFile config = plugin.Config;
                List<SettingEntryInfo> entries = new List<SettingEntryInfo>();

                foreach (KeyValuePair<ConfigDefinition, ConfigEntryBase> kvp in config)
                {
                    ConfigDefinition def = kvp.Key;
                    ConfigEntryBase entry = kvp.Value;

                    entries.Add(new SettingEntryInfo(
                        section: def.Section,
                        key: def.Key,
                        description: entry.Description?.Description ?? "",
                        entry: entry));
                }

                if (entries.Count > 0)
                {
                    _pluginSettings[guid] = new PluginSettingsInfo(guid, name, entries);

                    if (string.IsNullOrEmpty(_selectedPluginGuid))
                        _selectedPluginGuid = guid;
                }
            }
        }

        /// <summary>
        /// Manually registers settings for a plugin (for testing or when auto-discovery is not possible).
        /// </summary>
        /// <param name="info">The plugin settings info to register.</param>
        public void RegisterPluginSettings(PluginSettingsInfo info)
        {
            _pluginSettings[info.Guid] = info;
            if (string.IsNullOrEmpty(_selectedPluginGuid))
                _selectedPluginGuid = info.Guid;
        }

        private void OnGUI()
        {
            if (!_visible) return;

            _windowRect = GUI.Window(
                9002,
                _windowRect,
                DrawWindow,
                "Mod Settings");
        }

        private void DrawWindow(int windowId)
        {
            GUILayout.BeginVertical();

            // Plugin selector tabs
            GUILayout.BeginHorizontal();
            foreach (KeyValuePair<string, PluginSettingsInfo> kvp in _pluginSettings)
            {
                bool isSelected = kvp.Key == _selectedPluginGuid;
                Color oldColor = GUI.color;
                if (isSelected) GUI.color = Color.cyan;

                if (GUILayout.Button(kvp.Value.Name, GUILayout.Height(25)))
                {
                    _selectedPluginGuid = kvp.Key;
                }

                GUI.color = oldColor;
            }
            GUILayout.EndHorizontal();

            GUILayout.Space(8);

            // Settings for selected plugin
            if (_pluginSettings.TryGetValue(_selectedPluginGuid, out PluginSettingsInfo? plugin))
            {
                DrawPluginSettings(plugin);
            }
            else
            {
                GUILayout.Label("No plugin selected or no settings found.");
            }

            GUILayout.EndVertical();

            GUI.DragWindow(new Rect(0, 0, _windowRect.width, 20));
        }

        private void DrawPluginSettings(PluginSettingsInfo plugin)
        {
            _settingsScroll = GUILayout.BeginScrollView(_settingsScroll);

            string currentSection = "";

            foreach (SettingEntryInfo setting in plugin.Entries)
            {
                // Section header
                if (!string.Equals(setting.Section, currentSection, StringComparison.Ordinal))
                {
                    currentSection = setting.Section;
                    GUILayout.Space(6);
                    GUILayout.Label($"[{currentSection}]");
                    GUILayout.Space(2);
                }

                DrawSetting(setting);
            }

            GUILayout.EndScrollView();
        }

        private void DrawSetting(SettingEntryInfo setting)
        {
            GUILayout.BeginHorizontal("box");

            ConfigEntryBase entry = setting.Entry;
            Type settingType = entry.SettingType;

            GUILayout.Label(setting.Key, GUILayout.Width(150));

            if (settingType == typeof(bool))
            {
                bool current = (bool)entry.BoxedValue;
                bool newVal = GUILayout.Toggle(current, current ? "On" : "Off");
                if (newVal != current) entry.BoxedValue = newVal;
            }
            else if (settingType == typeof(int))
            {
                int current = (int)entry.BoxedValue;

                // Use acceptable range if defined, else default slider range
                int min = 0;
                int max = 1000;
                if (entry.Description?.AcceptableValues is AcceptableValueRange<int> range)
                {
                    min = range.MinValue;
                    max = range.MaxValue;
                }

                float newVal = GUILayout.HorizontalSlider(current, min, max, GUILayout.Width(150));
                int intVal = Mathf.RoundToInt(newVal);
                GUILayout.Label(intVal.ToString(), GUILayout.Width(50));
                if (intVal != current) entry.BoxedValue = intVal;
            }
            else if (settingType == typeof(float))
            {
                float current = (float)entry.BoxedValue;
                float min = 0f;
                float max = 100f;
                if (entry.Description?.AcceptableValues is AcceptableValueRange<float> range)
                {
                    min = range.MinValue;
                    max = range.MaxValue;
                }

                float newVal = GUILayout.HorizontalSlider(current, min, max, GUILayout.Width(150));
                GUILayout.Label(newVal.ToString("F2"), GUILayout.Width(50));
                if (Math.Abs(newVal - current) > 0.001f) entry.BoxedValue = newVal;
            }
            else if (settingType.IsEnum)
            {
                string current = entry.BoxedValue?.ToString() ?? string.Empty;
                string[] names = Enum.GetNames(settingType);
                int currentIndex = Array.IndexOf(names, current);
                if (currentIndex < 0) currentIndex = 0;

                // Simple next/prev buttons for enum
                if (GUILayout.Button("<", GUILayout.Width(25)))
                {
                    currentIndex = (currentIndex - 1 + names.Length) % names.Length;
                    Array values = Enum.GetValues(settingType);
                    entry.BoxedValue = values.GetValue(currentIndex);
                }
                GUILayout.Label(current, GUILayout.Width(100));
                if (GUILayout.Button(">", GUILayout.Width(25)))
                {
                    currentIndex = (currentIndex + 1) % names.Length;
                    Array values = Enum.GetValues(settingType);
                    entry.BoxedValue = values.GetValue(currentIndex);
                }
            }
            else
            {
                // Default: string field
                string current = entry.BoxedValue?.ToString() ?? "";
                string newVal = GUILayout.TextField(current, GUILayout.Width(200));
                if (newVal != current)
                {
                    try { entry.BoxedValue = newVal; }
                    catch { /* safe-swallow: user-edited text may not convert to the config entry type */ }
                }
            }

            GUILayout.EndHorizontal();

            // Description tooltip
            if (!string.IsNullOrEmpty(setting.Description))
            {
                GUILayout.Label($"  {setting.Description}");
            }
        }
    }

    /// <summary>
    /// Information about a plugin's settings for display in the settings panel.
    /// </summary>
    public sealed class PluginSettingsInfo
    {
        /// <summary>BepInEx plugin GUID.</summary>
        public string Guid { get; }

        /// <summary>Human-readable plugin name.</summary>
        public string Name { get; }

        /// <summary>All config entries for this plugin.</summary>
        public IReadOnlyList<SettingEntryInfo> Entries { get; }

        /// <summary>
        /// Creates a new plugin settings info instance.
        /// </summary>
        public PluginSettingsInfo(string guid, string name, IReadOnlyList<SettingEntryInfo> entries)
        {
            Guid = guid;
            Name = name;
            Entries = entries;
        }
    }

    /// <summary>
    /// Information about a single configuration entry.
    /// </summary>
    public sealed class SettingEntryInfo
    {
        /// <summary>Config section name.</summary>
        public string Section { get; }

        /// <summary>Config key name.</summary>
        public string Key { get; }

        /// <summary>Human-readable description of the setting.</summary>
        public string Description { get; }

        /// <summary>The underlying BepInEx config entry.</summary>
        public ConfigEntryBase Entry { get; }

        /// <summary>
        /// Creates a new setting entry info instance.
        /// </summary>
        public SettingEntryInfo(string section, string key, string description, ConfigEntryBase entry)
        {
            Section = section;
            Key = key;
            Description = description;
            Entry = entry;
        }
    }
}
