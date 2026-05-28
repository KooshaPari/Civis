#nullable enable
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using DINOForge.Runtime.Diagnostics;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Runtime state for a single faction tracked in the ECS system.
    /// Holds identity, configuration, and in-world entity counts.
    /// </summary>
    public class FactionRuntime
    {
        /// <summary>Unique faction identifier (e.g. "republic", "empire").</summary>
        public string Id { get; set; } = "";

        /// <summary>Human-readable faction name.</summary>
        public string Name { get; set; } = "";

        /// <summary>Whether this is an enemy faction (true) or player-owned (false).</summary>
        public bool IsEnemy { get; set; }

        /// <summary>Number of entities currently belonging to this faction in the world.</summary>
        public int EntityCount { get; set; }

        /// <summary>Reference to the source FactionDefinition from the pack system.</summary>
        public FactionDefinition? Definition { get; set; }
    }

    /// <summary>
    /// ECS SystemBase that maintains a dictionary of faction runtime states and provides
    /// entity-faction mapping based on the Enemy tag and unit type markers.
    ///
    /// Responsibilities:
    ///   1. Initialize factions from the loaded registry on demand
    ///   2. Provide lookup: GetFactionForEntity(Entity) -> faction ID
    ///   3. Provide tagging: SetEntityFaction(Entity, factionId) -> adds/removes Enemy tag
    ///   4. Track per-faction entity counts for debug overlay
    ///
    /// DINO has no explicit faction component — player vs enemy is represented via
    /// the Components.Enemy tag (absent = player). DINOForge factions in packs are
    /// logical groupings (west, republic, etc.) that map at runtime to this binary split.
    ///
    /// Usage:
    ///   1. ModPlatform.OnWorldReady() creates this system
    ///   2. InitializeFactions(registryManager.Factions) registers loaded pack factions
    ///   3. GetFactionForEntity() and SetEntityFaction() manage mappings
    ///
    /// Testing:
    ///   See src/Tests/FactionSystemTests.cs for unit and integration tests.
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class FactionSystem : SystemBase, IFactionSystem
    {
        private static readonly Dictionary<string, FactionRuntime> _factions =
            new Dictionary<string, FactionRuntime>(StringComparer.OrdinalIgnoreCase);

        private static bool _initialized;

        /// <summary>
        /// Read-only collection of all registered faction IDs.
        /// </summary>
        public IReadOnlyCollection<string> RegisteredFactions
        {
            get
            {
                lock (_factions)
                {
                    // allocation-ok: snapshot required for thread-safe enumeration outside lock
                    return new ReadOnlyCollection<string>(_factions.Keys.ToList());
                }
            }
        }

        public override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("FactionSystem", "FactionSystem.OnCreate");
        }

        public override void OnUpdate()
        {
            // This system is primarily a data holder and query interface.
            // Updates to faction state (entity counts) would happen here if needed.
            // For now, the system just maintains the static registry.
        }

        /// <summary>
        /// Initialize the faction system with definitions loaded from a pack registry.
        /// Registers each faction with a default ownership (player or enemy based on pack context).
        /// This is called once from ModPlatform.OnWorldReady().
        /// </summary>
        /// <param name="factionRegistry">The faction registry from RegistryManager, containing loaded pack factions.</param>
        public static void InitializeFactions(IRegistry<FactionDefinition> factionRegistry)
        {
            if (factionRegistry == null) throw new ArgumentNullException(nameof(factionRegistry));

            if (_initialized)
            {
                DebugLog.Write("FactionSystem", "FactionSystem.InitializeFactions already called; skipping re-initialization");
                return;
            }

            lock (_factions)
            {
                foreach (RegistryEntry<FactionDefinition> entry in factionRegistry.All.Values)
                {
                    FactionDefinition def = entry.Data;

                    var runtime = new FactionRuntime
                    {
                        Id = def.Faction.Id,
                        Name = def.Faction.DisplayName,
                        Definition = def,
                        IsEnemy = false, // Default to player-owned; can be toggled per-scenario
                        EntityCount = 0
                    };

                    _factions[def.Faction.Id] = runtime;
                    DebugLog.Write("FactionSystem", $"Registered faction: {def.Faction.Id} ({def.Faction.DisplayName})");
                }
            }

            _initialized = true;
            DebugLog.Write("FactionSystem", $"FactionSystem initialized with {_factions.Count} faction(s)");
        }

        /// <summary>
        /// Check if a faction with the given ID is registered.
        /// </summary>
        /// <param name="factionId">The faction ID to check.</param>
        /// <returns>True if registered, false otherwise.</returns>
        public bool IsFactionRegistered(string factionId)
        {
            if (string.IsNullOrEmpty(factionId)) return false;

            lock (_factions)
            {
                return _factions.ContainsKey(factionId);
            }
        }

        /// <summary>
        /// Register a single faction at runtime. Allows dynamic registration beyond
        /// what was loaded from packs.
        /// </summary>
        /// <param name="faction">The FactionDefinition to register.</param>
        /// <param name="isEnemy">Whether to mark this faction as enemy.</param>
        public void RegisterFaction(FactionDefinition faction, bool isEnemy)
        {
            if (faction == null) throw new ArgumentNullException(nameof(faction));
            if (string.IsNullOrEmpty(faction.Faction.Id))
                throw new ArgumentException("Faction definition must have a non-empty Id", nameof(faction));

            var runtime = new FactionRuntime
            {
                Id = faction.Faction.Id,
                Name = faction.Faction.DisplayName,
                Definition = faction,
                IsEnemy = isEnemy,
                EntityCount = 0
            };

            lock (_factions)
            {
                _factions[faction.Faction.Id] = runtime;
                DebugLog.Write("FactionSystem", $"Registered faction at runtime: {faction.Faction.Id} (isEnemy={isEnemy})");
            }
        }

        /// <summary>
        /// Get all player-owned factions (isEnemy == false).
        /// </summary>
        /// <returns>Read-only collection of player faction IDs.</returns>
        public static IReadOnlyCollection<string> GetPlayerFactions()
        {
            lock (_factions)
            {
                return new ReadOnlyCollection<string>(
                    _factions.Values
                        .Where(f => !f.IsEnemy)
                        .Select(f => f.Id)
                        .ToList()
                );
            }
        }

        /// <summary>
        /// Get all enemy factions (isEnemy == true).
        /// </summary>
        /// <returns>Read-only collection of enemy faction IDs.</returns>
        public static IReadOnlyCollection<string> GetEnemyFactions()
        {
            lock (_factions)
            {
                return new ReadOnlyCollection<string>(
                    _factions.Values
                        .Where(f => f.IsEnemy)
                        .Select(f => f.Id)
                        .ToList()
                );
            }
        }

        /// <summary>
        /// Retrieve the faction definition for a given faction ID.
        /// </summary>
        /// <param name="factionId">The faction ID.</param>
        /// <returns>The FactionRuntime, or null if not found.</returns>
        public static FactionRuntime? GetFaction(string factionId)
        {
            if (string.IsNullOrEmpty(factionId)) return null;

            lock (_factions)
            {
                _factions.TryGetValue(factionId, out FactionRuntime? faction);
                return faction;
            }
        }

        /// <summary>
        /// Get the faction ID for a given entity based on its Enemy tag.
        /// Returns the first player faction if no Enemy tag, or the first enemy faction if Enemy tag is present.
        /// This is a simplistic mapping suitable for single-player campaigns; multi-faction scenarios
        /// would require additional entity components or a more sophisticated lookup strategy.
        /// </summary>
        /// <param name="entity">The entity to check.</param>
        /// <param name="entityManager">The ECS EntityManager.</param>
        /// <returns>The faction ID, or empty string if no suitable faction found.</returns>
        public static string GetFactionForEntity(Entity entity, EntityManager entityManager)
        {
            bool isEnemy = entityManager.HasComponent<Components.Enemy>(entity);

            lock (_factions)
            {
                var targetFactions = _factions.Values
                    .Where(f => f.IsEnemy == isEnemy)
                    .ToList();

                if (targetFactions.Count > 0)
                {
                    return targetFactions[0].Id;
                }
            }

            return "";
        }

        /// <summary>
        /// Set or change the faction for an entity by toggling the Enemy tag.
        /// If the entity's current faction differs from the target, updates the tag state.
        /// </summary>
        /// <param name="entity">The entity to update.</param>
        /// <param name="factionId">The target faction ID.</param>
        /// <param name="entityManager">The ECS EntityManager.</param>
        public static void SetEntityFaction(Entity entity, string factionId, EntityManager entityManager)
        {
            if (string.IsNullOrEmpty(factionId))
            {
                DebugLog.Write("FactionSystem", "SetEntityFaction: factionId is empty");
                return;
            }

            FactionRuntime? faction = GetFaction(factionId);
            if (faction == null)
            {
                DebugLog.Write("FactionSystem", $"SetEntityFaction: faction {factionId} not registered");
                return;
            }

            // Determine target tag state based on faction's isEnemy flag
            bool shouldHaveEnemyTag = faction.IsEnemy;
            bool currentlyHasEnemyTag = entityManager.HasComponent<Components.Enemy>(entity);

            if (shouldHaveEnemyTag && !currentlyHasEnemyTag)
            {
                // Add Enemy tag
                entityManager.AddComponentData(entity, new Components.Enemy { });
                DebugLog.Write("FactionSystem", $"Added Enemy tag to entity {entity.Index} for faction {factionId}");
            }
            else if (!shouldHaveEnemyTag && currentlyHasEnemyTag)
            {
                // Remove Enemy tag
                entityManager.RemoveComponent<Components.Enemy>(entity);
                DebugLog.Write("FactionSystem", $"Removed Enemy tag from entity {entity.Index} for faction {factionId}");
            }
        }

        /// <summary>
        /// Update the entity count for a faction based on current world state.
        /// Queries all entities with the Unit tag and counts by faction.
        /// </summary>
        public void UpdateFactionCounts()
        {
            // This would typically be called from debug/overlay systems to refresh counts
            lock (_factions)
            {
                foreach (var faction in _factions.Values)
                {
                    faction.EntityCount = 0; // Reset; would need entity query to update properly
                }
            }
        }

        /// <summary>
        /// Get a snapshot of all faction runtime states for debugging/UI purposes.
        /// </summary>
        /// <returns>Dictionary mapping faction ID to FactionRuntime snapshot.</returns>
        public static IReadOnlyDictionary<string, FactionRuntime> GetFactionSnapshot()
        {
            lock (_factions)
            {
                var snapshot = new Dictionary<string, FactionRuntime>(StringComparer.OrdinalIgnoreCase);
                foreach (var kvp in _factions) snapshot[kvp.Key] = kvp.Value;
                return new ReadOnlyDictionary<string, FactionRuntime>(snapshot);
            }
        }

    }
}
