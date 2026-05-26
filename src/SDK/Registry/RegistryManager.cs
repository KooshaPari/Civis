using DINOForge.SDK.Models;

namespace DINOForge.SDK.Registry
{
    /// <summary>
    /// Central access point for all content registries. Provides typed registries
    /// for each content domain (units, buildings, factions, etc.).
    /// </summary>
    public sealed class RegistryManager
    {
        /// <summary>Registry for unit definitions.</summary>
        public IRegistry<UnitDefinition> Units { get; }

        /// <summary>Registry for building definitions.</summary>
        public IRegistry<BuildingDefinition> Buildings { get; }

        /// <summary>Registry for faction definitions.</summary>
        public IRegistry<FactionDefinition> Factions { get; }

        /// <summary>Registry for weapon definitions.</summary>
        public IRegistry<WeaponDefinition> Weapons { get; }

        /// <summary>Registry for projectile definitions.</summary>
        public IRegistry<ProjectileDefinition> Projectiles { get; }

        /// <summary>Registry for doctrine definitions.</summary>
        public IRegistry<DoctrineDefinition> Doctrines { get; }

        /// <summary>Registry for skill definitions.</summary>
        public IRegistry<SkillDefinition> Skills { get; }

        /// <summary>Registry for wave definitions.</summary>
        public IRegistry<WaveDefinition> Waves { get; }

        /// <summary>Registry for squad definitions.</summary>
        public IRegistry<SquadDefinition> Squads { get; }

        /// <summary>Registry for faction patch definitions.</summary>
        public IRegistry<FactionPatchDefinition> FactionPatches { get; }

        /// <summary>
        /// Initializes a new <see cref="RegistryManager"/> with empty registries for all content types.
        /// </summary>
        public RegistryManager()
        {
            Units = new Registry<UnitDefinition>();
            Buildings = new Registry<BuildingDefinition>();
            Factions = new Registry<FactionDefinition>();
            Weapons = new Registry<WeaponDefinition>();
            Projectiles = new Registry<ProjectileDefinition>();
            Doctrines = new Registry<DoctrineDefinition>();
            Skills = new Registry<SkillDefinition>();
            Waves = new Registry<WaveDefinition>();
            Squads = new Registry<SquadDefinition>();
            FactionPatches = new Registry<FactionPatchDefinition>();
        }
    }
}
