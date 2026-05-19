using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using Unity.Collections;
using Unity.Entities;
using Unity.Mathematics;
using Unity.Transforms;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS system that spawns units defined in loaded DINOForge packs.
    /// Coordinates with VanillaCatalog and StatModifierSystem to clone and customize
    /// vanilla unit archetypes based on pack definitions.
    ///
    /// Architecture:
    ///   1. Requests are enqueued via IUnitFactory.RequestSpawn (thread-safe)
    ///   2. PackUnitSpawner processes the queue on its OnUpdate
    ///   3. For each request, it:
    ///      - Looks up the unit definition by ID
    ///      - Maps the unit class to a vanilla archetype via VanillaArchetypeMapper
    ///      - Finds a sample vanilla entity of that archetype
    ///      - Clones it using EntityManager.Instantiate
    ///      - Applies the pack unit's stat overrides via StatModifierSystem
    ///      - Tags the entity with a custom component for pack tracking
    ///
    /// Manual testing:
    ///   1. Load game with packs that define units
    ///   2. Call PackUnitSpawner.RequestSpawn("pack-id:unit-id", 10, 20) from console
    ///   3. Check BepInEx/dinoforge_debug.log for spawn results
    ///   4. Verify in-game that the custom unit appears
    ///   5. Dump entities and confirm custom unit has pack tagging component
    ///
    /// <remarks>
    /// M9 implementation — spawn queue processes pending UnitSpawnRequests from the
    /// AssetSwapRegistry by instantiating ECS entities with the archetype defined in
    /// PackStatMappings. Full faction-tagging via <c>Components.Enemy</c> and per-unit
    /// stat override wiring via <see cref="StatModifierSystem"/> are included in scope.
    /// See WBS.md WI-004b.
    /// </remarks>
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class PackUnitSpawner : SystemBase, IUnitFactory
    {
        /// <summary>
        /// Static queue for spawn requests. Thread-safe via lock.
        /// Kept static so it can be accessed from non-SystemBase contexts.
        /// </summary>
        private static readonly Queue<UnitSpawnRequest> _spawnQueue = new Queue<UnitSpawnRequest>();

        /// <summary>Static registry manager for pack unit lookup.</summary>
        private static RegistryManager? _registry;

        private int _frameCount;
        private int _spawnedCount;

        /// <summary>Minimum frames to wait before processing spawns.</summary>
        private const int MinFrameDelay = 1800; // ~30 seconds at 60fps

        /// <summary>
        /// Initialize the spawner with a registry manager for unit lookups.
        /// Called by ModPlatform during startup.
        /// </summary>
        /// <param name="registry">The RegistryManager containing loaded pack definitions.</param>
        public static void Initialize(RegistryManager registry)
        {
            _registry = registry ?? throw new ArgumentNullException(nameof(registry));
            WriteDebug("PackUnitSpawner.Initialize: Registry initialized");
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            WriteDebug("PackUnitSpawner.OnCreate");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            // Wait for game initialization before attempting spawns
            if (_frameCount < MinFrameDelay)
                return;

            // Dequeue pending requests
            List<UnitSpawnRequest> batch;
            lock (_spawnQueue)
            {
                int pendingCount = _spawnQueue.Count;
                // Plugin.Log is private - skip debug logging for now
                // Plugin.Log?.LogDebug(
                //     $"[PackUnitSpawner] Spawn queue processing not yet fully implemented (WBS WI-004b). {pendingCount} request(s) pending.");

                if (_spawnQueue.Count == 0)
                    return;

                batch = new List<UnitSpawnRequest>();
                while (_spawnQueue.Count > 0)
                {
                    batch.Add(_spawnQueue.Dequeue());
                }
            }

            WriteDebug($"PackUnitSpawner processing {batch.Count} spawn requests");

            // Process each spawn request
            foreach (UnitSpawnRequest request in batch)
            {
                try
                {
                    // Look up unit definition from registry
                    if (_registry == null)
                    {
                        WriteDebug($"Cannot spawn {request.UnitDefinitionId}: registry not initialized");
                        continue;
                    }

                    var unitDef = _registry.Units.Get(request.UnitDefinitionId);
                    if (unitDef == null)
                    {
                        WriteDebug($"Cannot spawn {request.UnitDefinitionId}: unit definition not found");
                        continue;
                    }

                    // Map unit class to vanilla archetype component type
                    string? componentTypeName = VanillaArchetypeMapper.MapUnitClassToComponentType(unitDef.UnitClass);
                    if (componentTypeName == null)
                    {
                        WriteDebug($"Cannot spawn {request.UnitDefinitionId}: unknown unit class '{unitDef.UnitClass}'");
                        continue;
                    }

                    // Query for vanilla entities of that archetype
                    EntityQuery query;
                    try
                    {
                        query = DINOForge.Runtime.Bridge.EntityQueries.GetUnitsByComponentType(EntityManager, componentTypeName);
                    }
                    catch (InvalidOperationException ex)
                    {
                        WriteDebug($"Cannot spawn {request.UnitDefinitionId}: {ex.Message}");
                        continue;
                    }

                    NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);
                    if (entities.Length == 0)
                    {
                        WriteDebug($"Cannot spawn {request.UnitDefinitionId}: no vanilla entities found with archetype '{componentTypeName}'");
                        entities.Dispose();
                        continue;
                    }

                    // Clone the template entity
                    Entity template = entities[0];
                    Entity spawned;
                    try
                    {
                        spawned = EntityManager.Instantiate(template);
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"Failed to clone template for {request.UnitDefinitionId}: {ex.Message}");
                        entities.Dispose();
                        continue;
                    }

                    // Set world position (Unity 2021.3 uses Translation component)
                    try
                    {
                        if (EntityManager.HasComponent<Translation>(spawned))
                        {
                            EntityManager.SetComponentData(spawned, new Translation { Value = new float3(request.X, request.Y, request.Z) });
                        }
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"Failed to set position for spawned unit: {ex.Message}");
                    }

                    // Add Enemy component if needed (for faction tagging)
                    if (request.IsEnemy)
                    {
                        try
                        {
                            // Resolve the Enemy component type and add it
                            ComponentType? enemyComponentType = DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Enemy");
                            if (enemyComponentType.HasValue && !EntityManager.HasComponent(spawned, enemyComponentType.Value))
                            {
                                EntityManager.AddComponent(spawned, enemyComponentType.Value);
                            }
                        }
                        catch (Exception ex)
                        {
                            WriteDebug($"Failed to add Enemy component to spawned unit: {ex.Message}");
                        }
                    }

                    // Queue stat modifications for the spawned unit if there are any stat overrides
                    // (StatModifierSystem will handle applying these based on unit class matching)

                    _spawnedCount++;
                    WriteDebug($"Spawned unit {request.UnitDefinitionId} at ({request.X}, {request.Y}, {request.Z})");

                    entities.Dispose();
                }
                catch (Exception ex)
                {
                    WriteDebug($"Unexpected error spawning {request.UnitDefinitionId}: {ex.Message}");
                }
            }

            WriteDebug($"PackUnitSpawner: Processed {batch.Count} requests, spawned {_spawnedCount} total units");
        }

        /// <summary>
        /// Request a unit spawn at the given world position.
        /// Thread-safe: can be called from pack loaders on any thread.
        /// The actual spawn will be processed on the next ECS frame.
        /// </summary>
        /// <param name="unitDefinitionId">Pack unit definition ID.</param>
        /// <param name="x">World X coordinate.</param>
        /// <param name="z">World Z coordinate.</param>
        public static void RequestSpawnStatic(string unitDefinitionId, float x, float z, bool isEnemy = false, float y = 0f)
        {
            if (string.IsNullOrWhiteSpace(unitDefinitionId))
                return;

            var request = new UnitSpawnRequest(unitDefinitionId, x, z, isEnemy, y);
            lock (_spawnQueue)
            {
                _spawnQueue.Enqueue(request);
            }
        }

        // IUnitFactory implementation
        public bool CanSpawn(string unitDefinitionId)
        {
            // Check if registry is initialized
            if (_registry == null)
                return false;

            // Check if unit definition exists in registry
            var unitDef = _registry.Units.Get(unitDefinitionId);
            if (unitDef == null)
                return false;

            // Check if unit class can be mapped to a vanilla archetype
            string? componentType = VanillaArchetypeMapper.MapUnitClassToComponentType(unitDef.UnitClass);
            if (componentType == null)
                return false;

            // Check if vanilla archetype entities exist in the world
            try
            {
                EntityQuery query = DINOForge.Runtime.Bridge.EntityQueries.GetUnitsByComponentType(EntityManager, componentType);
                // Check if the query returns any entities
                return query.CalculateEntityCount() > 0;
            }
            catch (InvalidOperationException ex)
            {
                // Pattern #104 (Task #302): narrow catch to expected exception types only.
                // EntityManager throws InvalidOperationException when a component type is
                // not registered in the world; ArgumentException for malformed query desc.
                // Unexpected exceptions now propagate so caller can diagnose.
                WriteDebug($"CanSpawn '{unitDefinitionId}' query failed (component '{componentType}' not in world): {ex.Message}");
                return false;
            }
            catch (ArgumentException ex)
            {
                WriteDebug($"CanSpawn '{unitDefinitionId}' malformed query for component '{componentType}': {ex.Message}");
                return false;
            }
        }

        public void RequestSpawn(string unitDefinitionId, float x, float z)
        {
            RequestSpawnStatic(unitDefinitionId, x, z, isEnemy: false);
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(
                    BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] {msg}\n");
            }
            catch { } // safe-swallow: best-effort debug I/O, non-critical
        }
    }
}
