using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Universe
{
    /// <summary>
    /// Defines the faction hierarchy for a themed universe.
    /// Maps to the existing FactionDefinition model in the SDK.
    /// </summary>
    public sealed class FactionTaxonomy : IValidatable
    {
        /// <summary>
        /// All factions defined in this universe.
        /// </summary>
        [YamlMember(Alias = "factions")]
        public List<TaxonomyFaction> Factions { get; set; } = new List<TaxonomyFaction>(); // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the UniverseLoader factions.yaml deserialize site.
        /// Rejects any faction entry with a blank id.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            for (int i = 0; i < Factions.Count; i++)
            {
                TaxonomyFaction faction = Factions[i];
                if (faction == null || string.IsNullOrWhiteSpace(faction.Id))
                {
                    errors.Add(new ValidationError(
                        $"factions[{i}].id",
                        "TaxonomyFaction 'id' is required.",
                        "non_empty"));
                }
            }

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }

    /// <summary>
    /// A faction entry in the universe taxonomy.
    /// </summary>
    public sealed class TaxonomyFaction
    {
        /// <summary>
        /// Unique faction identifier (e.g. "republic", "cis").
        /// </summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>
        /// Display name (e.g. "Galactic Republic").
        /// </summary>
        [YamlMember(Alias = "name")]
        public string Name { get; set; } = "";

        /// <summary>
        /// Description of the faction.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Alignment in the game: Player, Enemy, or Neutral.
        /// </summary>
        [YamlMember(Alias = "alignment")]
        public string Alignment { get; set; } = "Player";

        /// <summary>
        /// Faction archetype: order, industrial_swarm, asymmetric, custom.
        /// </summary>
        [YamlMember(Alias = "archetype")]
        public string Archetype { get; set; } = "order";

        /// <summary>
        /// Optional sub-factions (e.g. 501st Legion under Republic).
        /// </summary>
        [YamlMember(Alias = "sub_factions")]
        public List<TaxonomySubFaction>? SubFactions { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Unit roster mapping abstract roles to themed unit IDs.
        /// </summary>
        [YamlMember(Alias = "unit_roster")]
        public Dictionary<string, string>? UnitRoster { get; set; } // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet
    }

    /// <summary>
    /// A sub-faction within a parent faction.
    /// </summary>
    public sealed class TaxonomySubFaction
    {
        /// <summary>
        /// Sub-faction identifier.
        /// </summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>
        /// Display name.
        /// </summary>
        [YamlMember(Alias = "name")]
        public string Name { get; set; } = "";

        /// <summary>
        /// Description.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Specialization or variant traits.
        /// </summary>
        [YamlMember(Alias = "specialization")]
        public string? Specialization { get; set; }
    }
}
