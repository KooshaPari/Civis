using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Universe
{
    /// <summary>
    /// Maps vanilla DINO entities to themed equivalents and vice versa.
    /// Supports exact matches and wildcard patterns (e.g. "enemy_*" -> "cis_*").
    /// </summary>
    public sealed class CrosswalkDictionary : IValidatable
    {
        /// <summary>
        /// Exact crosswalk entries keyed by vanilla ID.
        /// </summary>
        [YamlMember(Alias = "entries")]
        public Dictionary<string, CrosswalkEntry> Entries { get; set; } = new Dictionary<string, CrosswalkEntry>(StringComparer.OrdinalIgnoreCase); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet

        /// <summary>
        /// Wildcard pattern mappings (e.g. "enemy_*" -> "cis_*").
        /// </summary>
        [YamlMember(Alias = "patterns")]
        public List<CrosswalkPattern> Patterns { get; set; } = new List<CrosswalkPattern>(); // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Looks up a themed entry by vanilla ID. Tries exact match first, then patterns.
        /// </summary>
        /// <param name="vanillaId">The vanilla entity identifier.</param>
        /// <returns>The crosswalk entry, or null if no mapping exists.</returns>
        public CrosswalkEntry? LookupByVanillaId(string vanillaId)
        {
            if (Entries.TryGetValue(vanillaId, out CrosswalkEntry? entry))
                return entry;

            // Try wildcard patterns
            foreach (CrosswalkPattern pattern in Patterns)
            {
                if (MatchesPattern(vanillaId, pattern.VanillaPattern))
                {
                    return ApplyPattern(vanillaId, pattern);
                }
            }

            return null;
        }

        /// <summary>
        /// Reverse lookup: finds the vanilla ID for a themed ID.
        /// </summary>
        /// <param name="themedId">The themed entity identifier.</param>
        /// <returns>The vanilla ID, or null if no mapping exists.</returns>
        public string? LookupByThemedId(string themedId)
        {
            foreach (KeyValuePair<string, CrosswalkEntry> kvp in Entries)
            {
                if (string.Equals(kvp.Value.ThemedId, themedId, StringComparison.OrdinalIgnoreCase))
                    return kvp.Key;
            }

            // Try reverse pattern matching
            foreach (CrosswalkPattern pattern in Patterns)
            {
                if (MatchesPattern(themedId, pattern.ThemedPattern))
                {
                    return ReverseApplyPattern(themedId, pattern);
                }
            }

            return null;
        }

        /// <summary>
        /// Returns all exact entries as a flat list.
        /// </summary>
        public IReadOnlyList<CrosswalkEntry> GetAllEntries()
        {
            return Entries.Values.ToList();
        }

        /// <summary>
        /// Tests whether a value matches a simple wildcard pattern (supports * only).
        /// </summary>
        public static bool MatchesPattern(string value, string pattern)
        {
            if (string.IsNullOrEmpty(pattern))
                return false;

            // Convert wildcard pattern to regex
            string regexPattern = "^" + Regex.Escape(pattern).Replace("\\*", "(.+)") + "$";
            return Regex.IsMatch(value, regexPattern, RegexOptions.IgnoreCase);
        }

        /// <summary>
        /// Applies a wildcard pattern to generate a CrosswalkEntry from a vanilla ID.
        /// </summary>
        public static CrosswalkEntry ApplyPattern(string vanillaId, CrosswalkPattern pattern)
        {
            string capturedPart = ExtractWildcardValue(vanillaId, pattern.VanillaPattern);
            string themedId = pattern.ThemedPattern.Replace("*", capturedPart);

            return new CrosswalkEntry
            {
                VanillaId = vanillaId,
                ThemedId = themedId,
                ThemedName = pattern.ThemedNamePattern?.Replace("*", capturedPart),
                ThemedDescription = pattern.ThemedDescriptionPattern?.Replace("*", capturedPart),
                StatModifiers = pattern.StatModifiers
            };
        }

        /// <summary>
        /// Reverse-applies a pattern to get the vanilla ID from a themed ID.
        /// </summary>
        public static string ReverseApplyPattern(string themedId, CrosswalkPattern pattern)
        {
            string capturedPart = ExtractWildcardValue(themedId, pattern.ThemedPattern);
            return pattern.VanillaPattern.Replace("*", capturedPart);
        }

        /// <summary>
        /// Extracts the portion of a value matched by the * in a pattern.
        /// </summary>
        public static string ExtractWildcardValue(string value, string pattern)
        {
            string regexPattern = "^" + Regex.Escape(pattern).Replace("\\*", "(.+)") + "$";
            Match match = Regex.Match(value, regexPattern, RegexOptions.IgnoreCase);
            return match.Success ? match.Groups[1].Value : value;
        }

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the UniverseLoader crosswalk.yaml deserialize site.
        /// Rejects any entry with a blank themed_id.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            foreach (KeyValuePair<string, CrosswalkEntry> kvp in Entries)
            {
                if (kvp.Value == null || string.IsNullOrWhiteSpace(kvp.Value.ThemedId))
                {
                    errors.Add(new ValidationError(
                        $"entries.{kvp.Key}.themed_id",
                        "CrosswalkDictionary entry 'themed_id' is required.",
                        "non_empty"));
                }
            }

            for (int i = 0; i < Patterns.Count; i++)
            {
                CrosswalkPattern pat = Patterns[i];
                if (string.IsNullOrWhiteSpace(pat.VanillaPattern))
                    errors.Add(new ValidationError(
                        $"patterns[{i}].vanilla_pattern",
                        "CrosswalkPattern 'vanilla_pattern' is required.",
                        "non_empty"));
                if (string.IsNullOrWhiteSpace(pat.ThemedPattern))
                    errors.Add(new ValidationError(
                        $"patterns[{i}].themed_pattern",
                        "CrosswalkPattern 'themed_pattern' is required.",
                        "non_empty"));
            }

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }

    /// <summary>
    /// A single exact crosswalk mapping from a vanilla entity to a themed entity.
    /// </summary>
    public sealed class CrosswalkEntry
    {
        /// <summary>
        /// The original vanilla DINO entity ID.
        /// </summary>
        [YamlMember(Alias = "vanilla_id")]
        public string VanillaId { get; set; } = "";

        /// <summary>
        /// The themed replacement entity ID.
        /// </summary>
        [YamlMember(Alias = "themed_id")]
        public string ThemedId { get; set; } = "";

        /// <summary>
        /// Display name in the themed universe.
        /// </summary>
        [YamlMember(Alias = "themed_name")]
        public string? ThemedName { get; set; }

        /// <summary>
        /// Description in the themed universe.
        /// </summary>
        [YamlMember(Alias = "themed_description")]
        public string? ThemedDescription { get; set; }

        /// <summary>
        /// Optional sprite/texture override path.
        /// </summary>
        [YamlMember(Alias = "sprite_override")]
        public string? SpriteOverride { get; set; }

        /// <summary>
        /// Optional sound override path.
        /// </summary>
        [YamlMember(Alias = "sound_override")]
        public string? SoundOverride { get; set; }

        /// <summary>
        /// Optional stat modifiers applied to the themed version.
        /// Keys are stat names, values are multipliers or absolute values.
        /// </summary>
        [YamlMember(Alias = "stat_modifiers")]
        public Dictionary<string, float>? StatModifiers { get; set; } // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet
    }

    /// <summary>
    /// A wildcard-based crosswalk pattern for batch-mapping entities.
    /// </summary>
    public sealed class CrosswalkPattern
    {
        /// <summary>
        /// Vanilla ID pattern with * wildcard (e.g. "enemy_*").
        /// </summary>
        [YamlMember(Alias = "vanilla_pattern")]
        public string VanillaPattern { get; set; } = "";

        /// <summary>
        /// Themed ID pattern with * wildcard (e.g. "cis_*").
        /// </summary>
        [YamlMember(Alias = "themed_pattern")]
        public string ThemedPattern { get; set; } = "";

        /// <summary>
        /// Optional themed name pattern with * wildcard.
        /// </summary>
        [YamlMember(Alias = "themed_name_pattern")]
        public string? ThemedNamePattern { get; set; }

        /// <summary>
        /// Optional themed description pattern with * wildcard.
        /// </summary>
        [YamlMember(Alias = "themed_description_pattern")]
        public string? ThemedDescriptionPattern { get; set; }

        /// <summary>
        /// Optional stat modifiers applied to all matches.
        /// </summary>
        [YamlMember(Alias = "stat_modifiers")]
        public Dictionary<string, float>? StatModifiers { get; set; } // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet
    }
}
