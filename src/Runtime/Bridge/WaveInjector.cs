using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.Runtime.Diagnostics;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS system that injects DINOForge pack wave definitions into the game world.
    /// Translates wave definitions to unit spawn requests via PackUnitSpawner.
    ///
    /// Architecture:
    ///   1. Wave spawn requests are queued via QueueWave() from packs or UI
    ///   2. WaveInjector dequeues requests, looks up WaveDefinition from registry
    ///   3. For each active wave, iterates through spawn groups and units
    ///   4. Calls PackUnitSpawner.RequestSpawnStatic() for each unit with staggered timing
    ///   5. Tracks active waves and removes completed ones
    ///
    /// Manual testing:
    ///   1. Load game with wave definitions in loaded packs
    ///   2. Call WaveInjector.QueueWave(waveDefId) from console or UI
    ///   3. Check BepInEx/dinoforge_debug.log for wave injection results
    ///   4. Verify in-game that units spawn in the expected pattern and timing
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class WaveInjector : SystemBase
    {
        /// <summary>
        /// Queue for incoming wave spawn requests (thread-safe).
        /// </summary>
        private static readonly Queue<WaveSpawnRequest> _waveQueue = new Queue<WaveSpawnRequest>();

        /// <summary>
        /// Tracks currently active waves being processed.
        /// </summary>
        private readonly List<ActiveWave> _activeWaves = new List<ActiveWave>();

        /// <summary>
        /// Static registry manager for wave definition lookups.
        /// </summary>
        private static RegistryManager? _registry;

        private int _frameCount;
        private int _totalWavesProcessed;

        /// <summary>
        /// Minimum frames to wait before processing waves.
        /// Allows PackUnitSpawner to initialize first.
        /// </summary>
        private const int MinFrameDelay = 1800; // ~30 seconds at 60fps

        /// <summary>
        /// Initialize the injector with a registry manager for wave lookups.
        /// Called by ModPlatform during startup.
        /// </summary>
        /// <param name="registry">The RegistryManager containing loaded pack definitions.</param>
        public static void SetRegistryManager(RegistryManager registry)
        {
            _registry = registry ?? throw new ArgumentNullException(nameof(registry));
            DebugLog.Write("WaveInjector", "WaveInjector.SetRegistryManager: Registry initialized");
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("WaveInjector", "WaveInjector.OnCreate");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            // Wait for game initialization before attempting wave injections
            if (_frameCount < MinFrameDelay)
                return;

            // Dequeue new wave requests
            List<WaveSpawnRequest> newRequests;
            lock (_waveQueue)
            {
                if (_waveQueue.Count == 0)
                {
                    newRequests = new List<WaveSpawnRequest>();
                }
                else
                {
                    newRequests = new List<WaveSpawnRequest>();
                    while (_waveQueue.Count > 0)
                    {
                        newRequests.Add(_waveQueue.Dequeue());
                    }
                }
            }

            // Process new wave requests
            foreach (WaveSpawnRequest request in newRequests)
            {
                try
                {
                    ProcessWaveRequest(request);
                }
                catch (Exception ex)
                {
                    DebugLog.Write("WaveInjector", $"Error processing wave request {request.WaveDefinitionId}: {ex.Message}");
                }
            }

            // Update active waves
            int completedCount = 0;
            for (int i = _activeWaves.Count - 1; i >= 0; i--)
            {
                ActiveWave wave = _activeWaves[i];

                if (wave.IsComplete)
                {
                    DebugLog.Write("WaveInjector", $"Wave {wave.Definition.Id} completed");
                    _activeWaves.RemoveAt(i);
                    completedCount++;
                }
                else
                {
                    // Process next unit spawn if it's time
                    ProcessWaveUnitSpawns(wave);
                }
            }

            if (newRequests.Count > 0 || completedCount > 0)
            {
                DebugLog.Write("WaveInjector", $"WaveInjector: Dequeued {newRequests.Count} wave request(s), " +
                    $"removed {completedCount} completed wave(s), " +
                    $"active waves: {_activeWaves.Count}");
            }
        }

        /// <summary>
        /// Process a single wave spawn request: look up the definition and create an active wave.
        /// </summary>
        private void ProcessWaveRequest(WaveSpawnRequest request)
        {
            if (_registry == null)
            {
                DebugLog.Write("WaveInjector", $"Cannot process wave {request.WaveDefinitionId}: registry not initialized");
                return;
            }

            // Look up wave definition
            WaveDefinition? wavedef = _registry.Waves.Get(request.WaveDefinitionId);
            if (wavedef == null)
            {
                DebugLog.Write("WaveInjector", $"Wave definition not found: {request.WaveDefinitionId}");
                return;
            }

            // Create and queue an active wave
            var activeWave = new ActiveWave
            {
                Definition = wavedef,
                StartTime = (float)World.Time.ElapsedTime,
                Request = request,
                NextUnitIndex = 0,
                NextSpawnTime = (float)World.Time.ElapsedTime + request.StartDelaySeconds + wavedef.DelaySeconds
            };

            _activeWaves.Add(activeWave);
            _totalWavesProcessed++;

            DebugLog.Write("WaveInjector", $"Queued wave {wavedef.Id} (display: {wavedef.DisplayName}) " +
                $"with {wavedef.SpawnGroups.Count} spawn group(s)");
        }

        /// <summary>
        /// Process unit spawns for an active wave based on timing.
        /// </summary>
        private void ProcessWaveUnitSpawns(ActiveWave wave)
        {
            float currentTime = (float)World.Time.ElapsedTime;

            // Skip until it's time to spawn
            while (!wave.IsComplete && currentTime >= wave.NextSpawnTime)
            {
                // Find the next unit to spawn
                (SpawnGroup? group, int unitIndex) = FindNextUnitInWave(wave);

                if (group == null)
                {
                    wave.NextUnitIndex = wave.Definition.SpawnGroups.Count;
                    break;
                }

                // Request the spawn
                try
                {
                    float spawnX = wave.Request.SpawnX;
                    float spawnZ = wave.Request.SpawnZ;
                    float spawnY = wave.Request.SpawnY;

                    PackUnitSpawner.RequestSpawnStatic(group.UnitId, spawnX, spawnZ, wave.Request.UseEnemyFaction, spawnY);

                    DebugLog.Write("WaveInjector", $"Spawned unit from wave {wave.Definition.Id}: {group.UnitId} " +
                        $"(group {unitIndex}) at ({spawnX}, {spawnY}, {spawnZ})");
                }
                catch (Exception ex)
                {
                    DebugLog.Write("WaveInjector", $"Failed to spawn unit {group.UnitId} from wave {wave.Definition.Id}: {ex.Message}");
                }

                // Calculate next spawn time
                wave.NextSpawnTime = currentTime + group.SpawnDelay;
            }
        }

        /// <summary>
        /// Find the next unit to spawn in the wave, accounting for spawn group counts.
        /// Returns the spawn group and the group index, or (null, -1) if all units are done.
        /// </summary>
        private (SpawnGroup?, int) FindNextUnitInWave(ActiveWave wave)
        {
            int unitCount = 0;

            for (int groupIdx = 0; groupIdx < wave.Definition.SpawnGroups.Count; groupIdx++)
            {
                SpawnGroup group = wave.Definition.SpawnGroups[groupIdx];
                int groupUnitCount = Math.Max(1, group.Count);

                if (unitCount + groupUnitCount > wave.NextUnitIndex)
                {
                    // This is the group we need to spawn from
                    wave.NextUnitIndex = wave.NextUnitIndex + 1;
                    return (group, groupIdx);
                }

                unitCount = unitCount + groupUnitCount;
            }

            return (null, -1);
        }

        /// <summary>
        /// Queue a wave spawn request. Thread-safe: can be called from pack loaders on any thread.
        /// </summary>
        /// <param name="request">The wave spawn request to queue.</param>
        public static void QueueWave(WaveSpawnRequest request)
        {
            if (request == null)
                return;

            lock (_waveQueue)
            {
                _waveQueue.Enqueue(request);
            }
        }

        /// <summary>
        /// Queue a wave spawn request by definition ID with optional delay.
        /// </summary>
        /// <param name="waveDefinitionId">The ID of the wave definition to spawn.</param>
        /// <param name="delaySeconds">Optional delay before spawning. Default 0.</param>
        /// <param name="spawnX">World X coordinate for spawning. Default 0.</param>
        /// <param name="spawnZ">World Z coordinate for spawning. Default 0.</param>
        /// <param name="useEnemyFaction">Whether to mark spawned units as enemies. Default true.</param>
        public static void QueueWaveSimple(string waveDefinitionId, float delaySeconds = 0f,
            float spawnX = 0f, float spawnZ = 0f, bool useEnemyFaction = true)
        {
            var request = new WaveSpawnRequest
            {
                WaveDefinitionId = waveDefinitionId,
                StartDelaySeconds = delaySeconds,
                SpawnX = spawnX,
                SpawnZ = spawnZ,
                UseEnemyFaction = useEnemyFaction
            };

            QueueWave(request);
        }

        /// <summary>
        /// Get the number of currently active waves being processed.
        /// </summary>
        public int GetActiveWaveCount()
        {
            return _activeWaves.Count;
        }

    }

    /// <summary>
    /// Request to spawn a wave from a pack definition.
    /// </summary>
    public sealed class WaveSpawnRequest
    {
        /// <summary>ID of the wave definition to spawn.</summary>
        public string WaveDefinitionId { get; set; } = "";

        /// <summary>Delay in seconds before spawning starts. Default 0.</summary>
        public float StartDelaySeconds { get; set; } = 0f;

        /// <summary>Whether to tag spawned units as enemies. Default true.</summary>
        public bool UseEnemyFaction { get; set; } = true;

        /// <summary>World X coordinate for spawn location. Default 0.</summary>
        public float SpawnX { get; set; } = 0f;

        /// <summary>World Z coordinate for spawn location. Default 0.</summary>
        public float SpawnZ { get; set; } = 0f;

        /// <summary>World Y coordinate (altitude) for spawn location. Default 0.</summary>
        public float SpawnY { get; set; } = 0f;
    }

    /// <summary>
    /// Internal tracking for an active wave being processed.
    /// </summary>
    internal sealed class ActiveWave
    {
        /// <summary>The wave definition being executed.</summary>
        public WaveDefinition Definition { get; set; } = null!;

        /// <summary>The original spawn request.</summary>
        public WaveSpawnRequest Request { get; set; } = null!;

        /// <summary>Game time when this wave started.</summary>
        public float StartTime { get; set; }

        /// <summary>Index of the next unit to spawn within all units across all groups.</summary>
        public int NextUnitIndex { get; set; }

        /// <summary>Game time when the next unit should spawn.</summary>
        public float NextSpawnTime { get; set; }

        /// <summary>Whether all units in this wave have been spawned.</summary>
        public bool IsComplete => NextUnitIndex >= GetTotalUnitCount();

        private int GetTotalUnitCount()
        {
            int total = 0;
            foreach (SpawnGroup group in Definition.SpawnGroups)
            {
                total = total + Math.Max(1, group.Count);
            }
            return total;
        }
    }
}
