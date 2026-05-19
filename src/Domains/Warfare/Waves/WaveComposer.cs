using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Warfare.Waves
{
    /// <summary>
    /// Composes and scales wave definitions for a faction based on available units,
    /// wave count, and difficulty settings.
    /// </summary>
    public sealed class WaveComposer
    {
        /// <summary>
        /// Generate a sequence of waves for a faction based on difficulty and wave count.
        /// Units are selected from the faction's roster in the unit registry, with
        /// higher-tier units appearing in later waves.
        /// </summary>
        /// <param name="faction">The faction whose units to compose waves from.</param>
        /// <param name="units">The unit registry containing the faction's units.</param>
        /// <param name="waveCount">Number of waves to generate.</param>
        /// <param name="difficultyMultiplier">Overall difficulty scaling (1.0 = normal).</param>
        /// <returns>An ordered list of wave definitions.</returns>
        public IReadOnlyList<WaveDefinition> ComposeWaves(
            FactionDefinition faction,
            IRegistry<UnitDefinition> units,
            int waveCount,
            float difficultyMultiplier = 1.0f)
        {
            if (faction == null) throw new ArgumentNullException(nameof(faction));
            if (units == null) throw new ArgumentNullException(nameof(units));
            if (waveCount <= 0) throw new ArgumentOutOfRangeException(nameof(waveCount), "Wave count must be positive.");

            // Collect all units belonging to this faction
            string factionId = faction.Faction.Id;
            var factionUnits = units.All.Values
                .Select(e => e.Data)
                .Where(u => string.Equals(u.FactionId, factionId, StringComparison.OrdinalIgnoreCase))
                .OrderBy(u => u.Tier ?? 1)
                .ThenBy(u => u.Id)
                .ToList();

            if (factionUnits.Count == 0)
            {
                // No units available; return empty waves with placeholders
                var emptyWaves = new List<WaveDefinition>();
                for (int i = 1; i <= waveCount; i++)
                {
                    emptyWaves.Add(new WaveDefinition
                    {
                        Id = $"{factionId}_wave_{i}",
                        DisplayName = $"Wave {i}",
                        WaveNumber = i,
                        IsFinalWave = i == waveCount,
                        SpawnGroups = new List<SpawnGroup>()
                    });
                }
                return emptyWaves.AsReadOnly();
            }

            // Partition units by tier
            var tier1 = factionUnits.Where(u => (u.Tier ?? 1) <= 1).ToList();
            var tier2 = factionUnits.Where(u => (u.Tier ?? 1) == 2).ToList();
            var tier3Plus = factionUnits.Where(u => (u.Tier ?? 1) >= 3).ToList();

            // If a tier is empty, fall back to all units
            if (tier1.Count == 0) tier1 = factionUnits;
            if (tier2.Count == 0) tier2 = factionUnits;
            if (tier3Plus.Count == 0) tier3Plus = factionUnits;

            var waves = new List<WaveDefinition>();

            for (int i = 1; i <= waveCount; i++)
            {
                float progress = (float)i / waveCount; // 0..1
                int baseCount = (int)(5 + progress * 20); // 5 at start, 25 at end

                // Pick units based on wave progression
                List<UnitDefinition> pool;
                if (progress < 0.33f)
                    pool = tier1;
                else if (progress < 0.66f)
                    pool = tier2.Count > 0 ? tier2 : tier1;
                else
                    pool = tier3Plus.Count > 0 ? tier3Plus : tier2;

                var spawnGroups = new List<SpawnGroup>();

                // Distribute units across the pool
                int unitsPerGroup = Math.Max(1, baseCount / Math.Max(1, pool.Count));
                foreach (UnitDefinition unit in pool)
                {
                    int count = (int)Math.Max(1, unitsPerGroup * difficultyMultiplier);
                    spawnGroups.Add(new SpawnGroup
                    {
                        UnitId = unit.Id,
                        Count = count,
                        SpawnDelay = 0.5f
                    });
                }

                waves.Add(new WaveDefinition
                {
                    Id = $"{factionId}_wave_{i}",
                    DisplayName = $"Wave {i}",
                    WaveNumber = i,
                    DelaySeconds = i * 60f,
                    IsFinalWave = i == waveCount,
                    SpawnGroups = spawnGroups,
                    DifficultyScaling = new DifficultyScaling
                    {
                        CountMultiplier = difficultyMultiplier,
                        HealthMultiplier = 1.0f + (progress * 0.5f * difficultyMultiplier),
                        DamageMultiplier = 1.0f + (progress * 0.3f * difficultyMultiplier)
                    }
                });
            }

            return waves.AsReadOnly();
        }

        /// <summary>
        /// Scale an existing wave definition by the given difficulty multiplier.
        /// Returns a new wave with adjusted unit counts and scaling parameters.
        /// </summary>
        /// <param name="baseWave">The wave to scale (not mutated).</param>
        /// <param name="difficultyMultiplier">Scaling factor (1.0 = no change).</param>
        /// <returns>A new scaled wave definition.</returns>
        public WaveDefinition ScaleWave(WaveDefinition baseWave, float difficultyMultiplier)
        {
            if (baseWave == null) throw new ArgumentNullException(nameof(baseWave));

            var scaledGroups = new List<SpawnGroup>();
            foreach (SpawnGroup group in baseWave.SpawnGroups)
            {
                scaledGroups.Add(new SpawnGroup
                {
                    UnitId = group.UnitId,
                    Count = Math.Max(1, (int)(group.Count * difficultyMultiplier)),
                    SpawnDelay = group.SpawnDelay,
                    SpawnPoint = group.SpawnPoint
                });
            }

            DifficultyScaling? baseScaling = baseWave.DifficultyScaling;
            var newScaling = new DifficultyScaling
            {
                CountMultiplier = (baseScaling?.CountMultiplier ?? 1.0f) * difficultyMultiplier,
                HealthMultiplier = (baseScaling?.HealthMultiplier ?? 1.0f) * difficultyMultiplier,
                DamageMultiplier = (baseScaling?.DamageMultiplier ?? 1.0f) * difficultyMultiplier
            };

            return new WaveDefinition
            {
                Id = baseWave.Id,
                DisplayName = baseWave.DisplayName,
                Description = baseWave.Description,
                WaveNumber = baseWave.WaveNumber,
                DelaySeconds = baseWave.DelaySeconds,
                IsFinalWave = baseWave.IsFinalWave,
                SpawnGroups = scaledGroups,
                DifficultyScaling = newScaling
            };
        }
    }
}
