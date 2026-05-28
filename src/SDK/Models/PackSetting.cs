#nullable enable
using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// A user-configurable setting for a mod pack.
    /// Defines metadata for runtime customization (difficulty, enabled features, etc.).
    /// </summary>
    public sealed class PackSetting
    {
        /// <summary>
        /// Unique identifier for this setting (e.g., "difficulty_multiplier").
        /// Used as the key in the settings store.
        /// </summary>
        [YamlMember(Alias = "key")]
        public string Key { get; set; } = "";

        /// <summary>
        /// Human-readable display name for the setting (e.g., "Difficulty Multiplier").
        /// Shown in the F10 mod menu settings panel.
        /// </summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>
        /// Data type for this setting: bool, int, float, string, or enum.
        /// Determines which UI control is rendered in the mod menu.
        /// </summary>
        [YamlMember(Alias = "type")]
        public string Type { get; set; } = "bool";

        /// <summary>
        /// The default value for this setting if not yet configured.
        /// Deserialized as object; cast to correct type based on <see cref="Type"/>.
        /// </summary>
        [YamlMember(Alias = "default_value")]
        public object? DefaultValue { get; set; }

        /// <summary>
        /// Optional description of what this setting does.
        /// Shown as a tooltip or in help text in the mod menu.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// For type=enum, the list of valid enum option strings.
        /// Example: ["off", "normal", "hard", "hardcore"].
        /// </summary>
        [YamlMember(Alias = "enum_options")]
        public List<string>? EnumOptions { get; set; }

        /// <summary>
        /// Minimum value for type=int or type=float (inclusive).
        /// Defines the lower bound for slider controls.
        /// </summary>
        [YamlMember(Alias = "min")]
        public double? Min { get; set; }

        /// <summary>
        /// Maximum value for type=int or type=float (inclusive).
        /// Defines the upper bound for slider controls.
        /// </summary>
        [YamlMember(Alias = "max")]
        public double? Max { get; set; }
    }
}
