using System;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK.Models;

namespace DINOForge.Domains.Scenario.Balance
{
    /// <summary>
    /// Scales scenario parameters based on difficulty level. Easy grants more resources
    /// and weaker enemies; Hard and Nightmare reduce resources and increase enemy intensity.
    /// </summary>
    public sealed class DifficultyScaler
    {
        /// <summary>
        /// Get the base multiplier for a given difficulty level.
        /// Easy = 1.5, Normal = 1.0, Hard = 0.7, Nightmare = 0.5.
        /// </summary>
        /// <param name="difficulty">The difficulty level.</param>
        /// <returns>A float multiplier for resource scaling.</returns>
        public float GetDifficultyMultiplier(Difficulty difficulty)
        {
            switch (difficulty)
            {
                case Difficulty.Easy:
                    return 1.5f;
                case Difficulty.Normal:
                    return 1.0f;
                case Difficulty.Hard:
                    return 0.7f;
                case Difficulty.Nightmare:
                    return 0.5f;
                default:
                    return 1.0f;
            }
        }

        /// <summary>
        /// Scale starting resources based on difficulty. Easy gives a 1.5x bonus,
        /// Hard gives 0.7x, Nightmare gives 0.5x.
        /// </summary>
        /// <param name="baseResources">The base resource amounts to scale.</param>
        /// <param name="difficulty">The difficulty level.</param>
        /// <returns>A new ResourceCost with scaled values. Values are clamped to zero minimum.</returns>
        public ResourceCost ScaleResources(ResourceCost baseResources, Difficulty difficulty)
        {
            if (baseResources == null) throw new ArgumentNullException(nameof(baseResources));

            float multiplier = GetDifficultyMultiplier(difficulty);

            return new ResourceCost
            {
                Food = Math.Max(0, (int)(baseResources.Food * multiplier)),
                Wood = Math.Max(0, (int)(baseResources.Wood * multiplier)),
                Stone = Math.Max(0, (int)(baseResources.Stone * multiplier)),
                Iron = Math.Max(0, (int)(baseResources.Iron * multiplier)),
                Gold = Math.Max(0, (int)(baseResources.Gold * multiplier))
            };
        }

        /// <summary>
        /// Scale wave intensity based on difficulty and wave number. Higher difficulties
        /// increase the base intensity and scale more aggressively with wave progression.
        /// The intensity multiplier is applied to enemy count and stat modifiers.
        /// </summary>
        /// <param name="baseIntensity">The baseline intensity value (e.g., 1.0 for standard).</param>
        /// <param name="difficulty">The difficulty level.</param>
        /// <param name="waveNumber">The current wave number (1-based).</param>
        /// <returns>Scaled intensity value. Higher means more/stronger enemies.</returns>
        public float ScaleWaveIntensity(float baseIntensity, Difficulty difficulty, int waveNumber)
        {
            if (waveNumber < 1) throw new ArgumentOutOfRangeException(nameof(waveNumber), "Wave number must be positive.");

            // Inverse of resource multiplier: harder difficulty = higher enemy intensity
            float difficultyFactor = GetEnemyIntensityMultiplier(difficulty);

            // Waves ramp up: each wave adds 10% intensity compounding
            float waveScaling = 1.0f + ((waveNumber - 1) * 0.1f);

            // Harder difficulties scale more aggressively per wave
            float aggressionFactor = GetAggressionFactor(difficulty);
            float adjustedWaveScaling = 1.0f + ((waveScaling - 1.0f) * aggressionFactor);

            return baseIntensity * difficultyFactor * adjustedWaveScaling;
        }

        /// <summary>
        /// Get the enemy intensity multiplier for a difficulty. This is the inverse
        /// of the resource multiplier: harder = more enemies.
        /// </summary>
        private float GetEnemyIntensityMultiplier(Difficulty difficulty)
        {
            switch (difficulty)
            {
                case Difficulty.Easy:
                    return 0.7f;
                case Difficulty.Normal:
                    return 1.0f;
                case Difficulty.Hard:
                    return 1.4f;
                case Difficulty.Nightmare:
                    return 2.0f;
                default:
                    return 1.0f;
            }
        }

        /// <summary>
        /// Get how aggressively wave scaling compounds per difficulty.
        /// </summary>
        private float GetAggressionFactor(Difficulty difficulty)
        {
            switch (difficulty)
            {
                case Difficulty.Easy:
                    return 0.8f;
                case Difficulty.Normal:
                    return 1.0f;
                case Difficulty.Hard:
                    return 1.3f;
                case Difficulty.Nightmare:
                    return 1.6f;
                default:
                    return 1.0f;
            }
        }
    }
}
