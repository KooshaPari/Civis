using System;
using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;

namespace DINOForge.Domains.Economy.Models
{
    /// <summary>
    /// Defines a per-faction economy profile including starting resources,
    /// production bonuses, and trade modifiers. Economy profiles control
    /// how a faction gathers, produces, and trades resources.
    /// </summary>
    public class EconomyProfile : IValidatable
    {
        /// <summary>
        /// Unique identifier for this economy profile.
        /// </summary>
        public string Id { get; set; } = "";

        /// <summary>
        /// Display name of this economy profile.
        /// </summary>
        public string DisplayName { get; set; } = "";

        /// <summary>
        /// Optional description of the economy profile's play style.
        /// </summary>
        public string? Description { get; set; }

        /// <summary>
        /// Starting resources for a faction using this profile.
        /// </summary>
        public ResourceCost StartingResources { get; set; } = new ResourceCost();

        /// <summary>
        /// Per-resource production multipliers. Keys are resource types (food, wood, stone, iron, gold).
        /// Values are multipliers applied to all production of that resource. Default 1.0 for unlisted.
        /// </summary>
        public Dictionary<string, float> ProductionMultipliers { get; set; } = new Dictionary<string, float>(StringComparer.Ordinal); // public-mutable-ok: economy content deserializes mutable multiplier maps

        /// <summary>
        /// Per-resource consumption multipliers. Keys are resource types.
        /// Values less than 1.0 reduce consumption; greater than 1.0 increase it.
        /// </summary>
        public Dictionary<string, float> ConsumptionMultipliers { get; set; } = new Dictionary<string, float>(StringComparer.Ordinal); // public-mutable-ok: economy content deserializes mutable multiplier maps

        /// <summary>
        /// Global trade exchange rate modifier. Values less than 1.0 give better rates;
        /// greater than 1.0 give worse rates. Applied on top of per-route exchange rates.
        /// </summary>
        public float TradeRateModifier { get; set; } = 1.0f;

        /// <summary>
        /// Modifier to trade cooldown duration. Values less than 1.0 reduce cooldown;
        /// greater than 1.0 increase it.
        /// </summary>
        public float TradeCooldownModifier { get; set; } = 1.0f;

        /// <summary>
        /// Storage capacity multiplier. Affects how much of each resource can be stockpiled.
        /// </summary>
        public float StorageMultiplier { get; set; } = 1.0f;

        /// <summary>
        /// Building construction cost multiplier for economy buildings.
        /// </summary>
        public float BuildingCostModifier { get; set; } = 1.0f;

        /// <summary>
        /// Worker efficiency multiplier. Affects how productive workers are in production buildings.
        /// </summary>
        public float WorkerEfficiency { get; set; } = 1.0f;

        /// <summary>
        /// Gets the production multiplier for a specific resource type,
        /// defaulting to 1.0 if not explicitly set.
        /// </summary>
        /// <param name="resourceType">The resource type to look up.</param>
        /// <returns>The production multiplier for the given resource.</returns>
        public float GetProductionMultiplier(string resourceType)
        {
            return ProductionMultipliers.TryGetValue(resourceType, out float multiplier) ? multiplier : 1.0f;
        }

        /// <summary>
        /// Gets the consumption multiplier for a specific resource type,
        /// defaulting to 1.0 if not explicitly set.
        /// </summary>
        /// <param name="resourceType">The resource type to look up.</param>
        /// <returns>The consumption multiplier for the given resource.</returns>
        public float GetConsumptionMultiplier(string resourceType)
        {
            return ConsumptionMultipliers.TryGetValue(resourceType, out float multiplier) ? multiplier : 1.0f;
        }

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the EconomyContentLoader deserialize site.
        /// Rejects blank id, blank display_name, and any negative multiplier
        /// (production / consumption / trade rate / cooldown / storage / building cost / worker efficiency).
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "EconomyProfile 'id' is required.", "non_empty"));

            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "EconomyProfile 'display_name' is required.", "non_empty"));

            if (TradeRateModifier < 0f)
                errors.Add(new ValidationError("trade_rate_modifier", "EconomyProfile 'trade_rate_modifier' must be >= 0.", "min-value"));

            if (TradeCooldownModifier < 0f)
                errors.Add(new ValidationError("trade_cooldown_modifier", "EconomyProfile 'trade_cooldown_modifier' must be >= 0.", "min-value"));

            if (StorageMultiplier < 0f)
                errors.Add(new ValidationError("storage_multiplier", "EconomyProfile 'storage_multiplier' must be >= 0.", "min-value"));

            if (BuildingCostModifier < 0f)
                errors.Add(new ValidationError("building_cost_modifier", "EconomyProfile 'building_cost_modifier' must be >= 0.", "min-value"));

            if (WorkerEfficiency < 0f)
                errors.Add(new ValidationError("worker_efficiency", "EconomyProfile 'worker_efficiency' must be >= 0.", "min-value"));

            foreach (KeyValuePair<string, float> kvp in ProductionMultipliers)
            {
                if (kvp.Value < 0f)
                    errors.Add(new ValidationError(
                        $"production_multipliers.{kvp.Key}",
                        $"EconomyProfile 'production_multipliers.{kvp.Key}' must be >= 0.",
                        "min-value"));
            }

            foreach (KeyValuePair<string, float> kvp in ConsumptionMultipliers)
            {
                if (kvp.Value < 0f)
                    errors.Add(new ValidationError(
                        $"consumption_multipliers.{kvp.Key}",
                        $"EconomyProfile 'consumption_multipliers.{kvp.Key}' must be >= 0.",
                        "min-value"));
            }

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }
}
