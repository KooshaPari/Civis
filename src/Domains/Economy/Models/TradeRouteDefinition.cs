using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;

namespace DINOForge.Domains.Economy.Models
{
    /// <summary>
    /// Defines a trade route: the resources involved, exchange rate, and transaction constraints.
    /// </summary>
    public class TradeRouteDefinition : IValidatable
    {
        /// <summary>
        /// Unique identifier for this trade route (e.g. "trade-wood-to-gold").
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display name shown in-game for this trade option.
        /// </summary>
        public string DisplayName { get; set; }

        /// <summary>
        /// The resource to trade away (e.g. "wood").
        /// </summary>
        public string SourceResource { get; set; }

        /// <summary>
        /// The resource to receive (e.g. "gold").
        /// </summary>
        public string TargetResource { get; set; }

        /// <summary>
        /// Exchange rate: how many source units per target unit.
        /// E.g., 10.0 means 10 wood = 1 gold.
        /// </summary>
        public float ExchangeRate { get; set; }

        /// <summary>
        /// Cooldown in game ticks between consecutive trades on this route.
        /// Typical tick duration is 0.016 seconds (~60 ticks per second).
        /// </summary>
        public int CooldownTicks { get; set; }

        /// <summary>
        /// Maximum source units transferred per single transaction.
        /// </summary>
        public float MaxPerTransaction { get; set; }

        /// <summary>
        /// Whether this trade route is currently available.
        /// </summary>
        public bool Enabled { get; set; }

        /// <summary>
        /// Initializes a new trade route definition with default values.
        /// </summary>
        public TradeRouteDefinition()
        {
            Id = string.Empty;
            DisplayName = string.Empty;
            SourceResource = string.Empty;
            TargetResource = string.Empty;
            ExchangeRate = 1.0f;
            CooldownTicks = 60;
            MaxPerTransaction = 1000.0f;
            Enabled = true;
        }

        /// <summary>
        /// Initializes a new trade route with all properties.
        /// </summary>
        public TradeRouteDefinition(
            string id,
            string displayName,
            string sourceResource,
            string targetResource,
            float exchangeRate,
            int cooldownTicks,
            float maxPerTransaction,
            bool enabled)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            DisplayName = displayName ?? throw new ArgumentNullException(nameof(displayName));
            SourceResource = sourceResource ?? throw new ArgumentNullException(nameof(sourceResource));
            TargetResource = targetResource ?? throw new ArgumentNullException(nameof(targetResource));
            ExchangeRate = exchangeRate;
            CooldownTicks = cooldownTicks;
            MaxPerTransaction = maxPerTransaction;
            Enabled = enabled;
        }

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the EconomyContentLoader deserialize site.
        /// Rejects blank id, blank source/target resource, and non-positive exchange_rate.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "TradeRouteDefinition 'id' is required.", "non_empty"));

            if (string.IsNullOrWhiteSpace(SourceResource))
                errors.Add(new ValidationError("source_resource", "TradeRouteDefinition 'source_resource' is required.", "non_empty"));

            if (string.IsNullOrWhiteSpace(TargetResource))
                errors.Add(new ValidationError("target_resource", "TradeRouteDefinition 'target_resource' is required.", "non_empty"));

            if (ExchangeRate <= 0f)
                errors.Add(new ValidationError("exchange_rate", "TradeRouteDefinition 'exchange_rate' must be > 0.", "min-value"));

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }
}
