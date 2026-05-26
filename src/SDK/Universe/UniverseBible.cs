using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Universe
{
    /// <summary>
    /// Top-level container for a themed universe definition.
    /// A UniverseBible provides all the metadata needed to deterministically
    /// generate a complete mod pack for a given theme (e.g. Star Wars, Modern Warfare).
    /// </summary>
    public class UniverseBible : IValidatable
    {
        /// <summary>
        /// Unique identifier for this universe (e.g. "star-wars-clone-wars").
        /// </summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>
        /// Display name (e.g. "Star Wars: Clone Wars").
        /// </summary>
        [YamlMember(Alias = "name")]
        public string Name { get; set; } = "";

        /// <summary>
        /// Description of the universe theme.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Era or time period (e.g. "Clone Wars 22-19 BBY", "Modern 2000-2025").
        /// </summary>
        [YamlMember(Alias = "era")]
        public string? Era { get; set; }

        /// <summary>
        /// Version of the universe bible.
        /// </summary>
        [YamlMember(Alias = "version")]
        public string Version { get; set; } = "0.1.0";

        /// <summary>
        /// Author of this universe definition.
        /// </summary>
        [YamlMember(Alias = "author")]
        public string? Author { get; set; }

        /// <summary>
        /// Faction taxonomy defining all factions in this universe.
        /// </summary>
        [YamlMember(Alias = "faction_taxonomy")]
        public FactionTaxonomy FactionTaxonomy { get; set; } = new FactionTaxonomy();

        /// <summary>
        /// Crosswalk dictionary mapping vanilla DINO entities to themed equivalents.
        /// </summary>
        [YamlMember(Alias = "crosswalk")]
        public CrosswalkDictionary CrosswalkDictionary { get; set; } = new CrosswalkDictionary();

        /// <summary>
        /// Naming conventions for this universe.
        /// </summary>
        [YamlMember(Alias = "naming")]
        public NamingGuide NamingGuide { get; set; } = new NamingGuide();

        /// <summary>
        /// Visual and audio style rules.
        /// </summary>
        [YamlMember(Alias = "style")]
        public StyleGuide StyleGuide { get; set; } = new StyleGuide();

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the UniverseLoader deserialize sites.
        /// Rejects blank id and blank name (the two universe-level required fields).
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "UniverseBible 'id' is required.", "non_empty"));

            if (string.IsNullOrWhiteSpace(Name))
                errors.Add(new ValidationError("name", "UniverseBible 'name' is required.", "non_empty"));

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }
}
