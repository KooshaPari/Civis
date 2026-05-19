using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Universe
{
    /// <summary>
    /// Rules for naming units, buildings, and weapons in a themed universe.
    /// Provides prefixes, suffixes, and naming patterns per faction and entity type.
    /// </summary>
    public sealed class NamingGuide : IValidatable
    {
        /// <summary>
        /// Per-faction naming rules.
        /// </summary>
        [YamlMember(Alias = "faction_rules")]
        public Dictionary<string, FactionNamingRules> FactionRules { get; set; } = new Dictionary<string, FactionNamingRules>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Global naming rules applied when no faction-specific rule matches.
        /// </summary>
        [YamlMember(Alias = "global_rules")]
        public NamingRuleSet? GlobalRules { get; set; }

        /// <summary>
        /// Applies naming rules to generate a themed name for an entity.
        /// </summary>
        /// <param name="factionId">The faction this entity belongs to.</param>
        /// <param name="entityType">The entity type (e.g. "unit", "building", "weapon").</param>
        /// <param name="baseName">The base name to transform.</param>
        /// <returns>The themed name, or the base name if no rules match.</returns>
        public string ApplyNaming(string factionId, string entityType, string baseName)
        {
            // Try faction-specific rules first
            if (FactionRules.TryGetValue(factionId, out FactionNamingRules? factionRules))
            {
                string? result = factionRules.Apply(entityType, baseName);
                if (result != null)
                    return result;
            }

            // Fall back to global rules
            if (GlobalRules != null)
            {
                string? result = GlobalRules.Apply(entityType, baseName);
                if (result != null)
                    return result;
            }

            return baseName;
        }

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the UniverseLoader naming.yaml deserialize site.
        /// Rejects any blank faction-rule key.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            foreach (string key in FactionRules.Keys)
            {
                if (string.IsNullOrWhiteSpace(key))
                {
                    errors.Add(new ValidationError(
                        "faction_rules",
                        "NamingGuide 'faction_rules' must not have a blank key.",
                        "non_empty"));
                    break;
                }
            }

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }

    /// <summary>
    /// Naming rules for a specific faction.
    /// </summary>
    public sealed class FactionNamingRules
    {
        /// <summary>
        /// Rules for different entity types (unit, building, weapon, vehicle).
        /// </summary>
        [YamlMember(Alias = "rules")]
        public Dictionary<string, NamingRuleSet> Rules { get; set; } = new Dictionary<string, NamingRuleSet>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Applies the appropriate rule set for the given entity type.
        /// </summary>
        public string? Apply(string entityType, string baseName)
        {
            if (Rules.TryGetValue(entityType, out NamingRuleSet? ruleSet))
            {
                return ruleSet.Apply(entityType, baseName);
            }

            return null;
        }
    }

    /// <summary>
    /// A set of naming transformation rules.
    /// </summary>
    public sealed class NamingRuleSet
    {
        /// <summary>
        /// Prefix to add to names (e.g. "Clone " for Republic soldiers).
        /// </summary>
        [YamlMember(Alias = "prefix")]
        public string? Prefix { get; set; }

        /// <summary>
        /// Suffix to add to names.
        /// </summary>
        [YamlMember(Alias = "suffix")]
        public string? Suffix { get; set; }

        /// <summary>
        /// Pattern with {name} placeholder (e.g. "AT-{name}" for walkers).
        /// Takes precedence over prefix/suffix if set.
        /// </summary>
        [YamlMember(Alias = "pattern")]
        public string? Pattern { get; set; }

        /// <summary>
        /// Explicit name overrides. Keys are base names, values are themed names.
        /// Takes highest precedence.
        /// </summary>
        [YamlMember(Alias = "overrides")]
        public Dictionary<string, string>? Overrides { get; set; }

        /// <summary>
        /// Applies this rule set to transform a base name.
        /// </summary>
        public string? Apply(string entityType, string baseName)
        {
            // Check explicit overrides first
            if (Overrides != null && Overrides.TryGetValue(baseName, out string? overrideName))
                return overrideName;

            // Apply pattern if set
            if (!string.IsNullOrEmpty(Pattern))
                return Pattern!.Replace("{name}", baseName);

            // Apply prefix/suffix
            if (!string.IsNullOrEmpty(Prefix) || !string.IsNullOrEmpty(Suffix))
                return $"{Prefix ?? ""}{baseName}{Suffix ?? ""}";

            return null;
        }
    }
}
