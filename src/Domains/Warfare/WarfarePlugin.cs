using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.Domains.Warfare.Balance;
using DINOForge.Domains.Warfare.Doctrines;
using DINOForge.Domains.Warfare.Roles;
using DINOForge.Domains.Warfare.Waves;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Warfare
{
    /// <summary>
    /// Entry point for the Warfare domain plugin. Provides access to all warfare subsystems:
    /// archetype registry, doctrine engine, unit role validation, wave composition, and balance calculation.
    /// </summary>
    public sealed class WarfarePlugin
    {
        /// <summary>
        /// Registry of faction archetypes (Order, Industrial Swarm, Asymmetric, custom).
        /// </summary>
        public ArchetypeRegistry Archetypes { get; }

        /// <summary>
        /// Engine for applying archetype and doctrine modifiers to unit stats.
        /// </summary>
        public DoctrineEngine Doctrines { get; }

        /// <summary>
        /// Validator for ensuring factions fill all required unit role slots.
        /// </summary>
        public UnitRoleValidator RoleValidator { get; }

        /// <summary>
        /// Composer for generating and scaling enemy wave definitions.
        /// </summary>
        public WaveComposer WaveComposer { get; }

        /// <summary>
        /// Calculator for unit and faction power ratings and balance comparisons.
        /// </summary>
        public BalanceCalculator Balance { get; }

        private readonly RegistryManager _registries;

        /// <summary>
        /// Initialize the warfare plugin with pre-loaded registries.
        /// </summary>
        /// <param name="registries">The registry manager containing all loaded content.</param>
        public WarfarePlugin(RegistryManager registries)
        {
            _registries = registries ?? throw new ArgumentNullException(nameof(registries));

            Archetypes = new ArchetypeRegistry();
            Doctrines = new DoctrineEngine();
            RoleValidator = new UnitRoleValidator();
            WaveComposer = new WaveComposer();
            Balance = new BalanceCalculator(Doctrines);
        }

        /// <summary>
        /// Validate a complete warfare pack. Checks:
        /// - All factions reference valid archetypes
        /// - All faction rosters fill required unit roles
        /// - All doctrines have valid modifier values
        /// - All wave definitions reference valid units
        /// </summary>
        /// <param name="packId">The pack identifier to scope validation to.</param>
        /// <param name="registries">The registry manager with loaded content.</param>
        /// <returns>A validation result with errors, warnings, and per-faction roster results.</returns>
        public WarfareValidationResult ValidatePack(string packId, RegistryManager registries)
        {
            if (string.IsNullOrWhiteSpace(packId)) throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (registries == null) throw new ArgumentNullException(nameof(registries));

            var errors = new List<string>();
            var warnings = new List<string>();
            var rosterResults = new Dictionary<string, RosterValidationResult>(StringComparer.OrdinalIgnoreCase);

            // Validate factions from this pack
            foreach (KeyValuePair<string, RegistryEntry<FactionDefinition>> kvp in registries.Factions.All)
            {
                RegistryEntry<FactionDefinition> entry = kvp.Value;
                if (!string.Equals(entry.SourcePackId, packId, StringComparison.OrdinalIgnoreCase))
                    continue;

                FactionDefinition faction = entry.Data;
                string factionId = faction.Faction.Id;

                // Validate archetype reference
                string archetype = faction.Faction.Archetype;
                if (!string.IsNullOrEmpty(archetype) && !Archetypes.TryGetArchetype(archetype, out _))
                {
                    errors.Add($"Faction '{factionId}' references unknown archetype '{archetype}'.");
                }

                // Validate roster completeness
                RosterValidationResult rosterResult = RoleValidator.ValidateRoster(faction, registries.Units);
                rosterResults[factionId] = rosterResult;

                if (!rosterResult.IsComplete)
                {
                    foreach (string missingRole in rosterResult.MissingRoles)
                    {
                        warnings.Add($"Faction '{factionId}' is missing unit for role '{missingRole}'.");
                    }
                }
            }

            // Validate doctrines from this pack
            foreach (KeyValuePair<string, RegistryEntry<DoctrineDefinition>> kvp in registries.Doctrines.All)
            {
                RegistryEntry<DoctrineDefinition> entry = kvp.Value;
                if (!string.Equals(entry.SourcePackId, packId, StringComparison.OrdinalIgnoreCase))
                    continue;

                IReadOnlyList<string> docErrors = Doctrines.ValidateDoctrine(entry.Data);
                foreach (string error in docErrors)
                {
                    errors.Add(error);
                }
            }

            // Validate waves reference valid units
            foreach (KeyValuePair<string, RegistryEntry<WaveDefinition>> kvp in registries.Waves.All)
            {
                RegistryEntry<WaveDefinition> entry = kvp.Value;
                if (!string.Equals(entry.SourcePackId, packId, StringComparison.OrdinalIgnoreCase))
                    continue;

                WaveDefinition wave = entry.Data;
                foreach (SpawnGroup group in wave.SpawnGroups)
                {
                    if (!registries.Units.Contains(group.UnitId))
                    {
                        errors.Add($"Wave '{wave.Id}' spawn group references unknown unit '{group.UnitId}'.");
                    }
                }
            }

            return new WarfareValidationResult(
                packId: packId,
                isValid: errors.Count == 0,
                errors: errors.AsReadOnly(),
                warnings: warnings.AsReadOnly(),
                rosterResults: rosterResults);
        }
    }
}
