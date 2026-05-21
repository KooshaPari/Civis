using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.SDK.IO;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Domains.Warfare
{
    /// <summary>
    /// Loads warfare definitions from pack directories into warfare registries.
    /// Handles factions/, units/, buildings/, weapons/, projectiles/, doctrines/,
    /// waves/, squads/, and archetypes/ subdirectories.
    /// Mirrors the EconomyContentLoader shape (Domains/Economy/EconomyContentLoader.cs)
    /// for parity across domain plugins (task #843).
    /// </summary>
    public sealed class WarfareContentLoader
    {
        private const int DefaultLoadOrder = 100;

        private readonly IRegistry<FactionDefinition> _factions;
        private readonly IRegistry<UnitDefinition> _units;
        private readonly IRegistry<BuildingDefinition> _buildings;
        private readonly IRegistry<WeaponDefinition> _weapons;
        private readonly IRegistry<ProjectileDefinition> _projectiles;
        private readonly IRegistry<DoctrineDefinition> _doctrines;
        private readonly IRegistry<WaveDefinition> _waves;
        private readonly IRegistry<SquadDefinition> _squads;
        private readonly ArchetypeRegistry _archetypes;
        private readonly IDeserializer _deserializer;

        /// <summary>
        /// Initializes a new warfare content loader with the provided registries.
        /// </summary>
        public WarfareContentLoader(
            IRegistry<FactionDefinition> factions,
            IRegistry<UnitDefinition> units,
            IRegistry<BuildingDefinition> buildings,
            IRegistry<WeaponDefinition> weapons,
            IRegistry<ProjectileDefinition> projectiles,
            IRegistry<DoctrineDefinition> doctrines,
            IRegistry<WaveDefinition> waves,
            IRegistry<SquadDefinition> squads,
            ArchetypeRegistry archetypes)
        {
            _factions = factions ?? throw new ArgumentNullException(nameof(factions));
            _units = units ?? throw new ArgumentNullException(nameof(units));
            _buildings = buildings ?? throw new ArgumentNullException(nameof(buildings));
            _weapons = weapons ?? throw new ArgumentNullException(nameof(weapons));
            _projectiles = projectiles ?? throw new ArgumentNullException(nameof(projectiles));
            _doctrines = doctrines ?? throw new ArgumentNullException(nameof(doctrines));
            _waves = waves ?? throw new ArgumentNullException(nameof(waves));
            _squads = squads ?? throw new ArgumentNullException(nameof(squads));
            _archetypes = archetypes ?? throw new ArgumentNullException(nameof(archetypes));

            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();
        }

        /// <summary>
        /// Load all warfare definitions from a pack directory into the wired registries.
        /// </summary>
        /// <param name="packDir">The root directory of the pack.</param>
        /// <param name="packId">The pack identifier (for registration + logging).</param>
        public void LoadPack(string packDir, string packId)
        {
            if (string.IsNullOrWhiteSpace(packDir)) throw new ArgumentException("Pack directory is required.", nameof(packDir));
            if (string.IsNullOrWhiteSpace(packId)) throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (!Directory.Exists(packDir))
                throw new DirectoryNotFoundException($"Pack directory not found: {packDir}");

            LoadFactions(Path.Combine(packDir, "factions"), packId);
            LoadUnits(Path.Combine(packDir, "units"), packId);
            LoadBuildings(Path.Combine(packDir, "buildings"), packId);
            LoadWeapons(Path.Combine(packDir, "weapons"), packId);
            LoadProjectiles(Path.Combine(packDir, "projectiles"), packId);
            LoadDoctrines(Path.Combine(packDir, "doctrines"), packId);
            LoadWaves(Path.Combine(packDir, "waves"), packId);
            LoadSquads(Path.Combine(packDir, "squads"), packId);
            LoadArchetypes(Path.Combine(packDir, "archetypes"), packId);
        }

        private void LoadFactions(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<FactionDefinition>(file, packId, "faction", def =>
                {
                    string id = def?.Faction?.Id ?? string.Empty;
                    if (string.IsNullOrEmpty(id))
                        throw new InvalidOperationException("Faction definition is missing faction.id.");
                    _factions.Register(id, def!, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadUnits(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<UnitDefinition>(file, packId, "unit", def =>
                {
                    RequireId(def?.Id, "unit");
                    _units.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadBuildings(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<BuildingDefinition>(file, packId, "building", def =>
                {
                    RequireId(def?.Id, "building");
                    _buildings.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadWeapons(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<WeaponDefinition>(file, packId, "weapon", def =>
                {
                    RequireId(def?.Id, "weapon");
                    _weapons.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadProjectiles(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<ProjectileDefinition>(file, packId, "projectile", def =>
                {
                    RequireId(def?.Id, "projectile");
                    _projectiles.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadDoctrines(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<DoctrineDefinition>(file, packId, "doctrine", def =>
                {
                    RequireId(def?.Id, "doctrine");
                    _doctrines.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadWaves(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<WaveDefinition>(file, packId, "wave", def =>
                {
                    RequireId(def?.Id, "wave");
                    _waves.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadSquads(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                ProcessFile<SquadDefinition>(file, packId, "squad", def =>
                {
                    RequireId(def?.Id, "squad");
                    _squads.Register(def!.Id, def, RegistrySource.Pack, packId, DefaultLoadOrder);
                });
            }
        }

        private void LoadArchetypes(string dir, string packId)
        {
            foreach (string file in EnumerateYaml(dir))
            {
                try
                {
                    string yaml = SafeFileIO.ReadText(file);
                    ArchetypeDto? dto = _deserializer.Deserialize<ArchetypeDto>(yaml);
                    if (dto == null)
                        continue;

                    if (string.IsNullOrWhiteSpace(dto.Id))
                        throw new InvalidOperationException("Archetype definition is missing id.");

                    var archetype = new FactionArchetype(
                        dto.Id,
                        dto.DisplayName ?? dto.Id,
                        dto.Description ?? string.Empty,
                        dto.BaseModifiers ?? new Dictionary<string, float>(StringComparer.Ordinal));

                    _archetypes.Register(archetype);
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load archetype from {file} in pack '{packId}'.", ex);
                }
            }
        }

        private static IEnumerable<string> EnumerateYaml(string dir)
        {
            if (!Directory.Exists(dir))
                return Array.Empty<string>();

            return Directory.GetFiles(dir, "*.yaml", SearchOption.AllDirectories);
        }

        private void ProcessFile<T>(string file, string packId, string kind, Action<T?> register) where T : class
        {
            try
            {
                string yaml = SafeFileIO.ReadText(file);
                T? def = _deserializer.Deserialize<T>(yaml);
                if (def == null)
                    return;

                // Task #319 — IValidatable semantic check at the deserialize site.
                JsonGuard.ValidateOrThrow(def, file);
                register(def);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException(
                    $"Failed to load {kind} from {file} in pack '{packId}'.", ex);
            }
        }

        private static void RequireId(string? id, string kind)
        {
            if (string.IsNullOrWhiteSpace(id))
                throw new InvalidOperationException($"{kind} definition is missing id.");
        }

        /// <summary>
        /// YAML DTO for archetype deserialization (FactionArchetype itself is immutable).
        /// </summary>
        private sealed class ArchetypeDto
        {
            public string Id { get; set; } = string.Empty;
            public string? DisplayName { get; set; }
            public string? Description { get; set; }
            public Dictionary<string, float>? BaseModifiers { get; set; }
        }
    }
}
