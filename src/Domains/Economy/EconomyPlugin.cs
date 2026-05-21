using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Balance;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Rates;
using DINOForge.Domains.Economy.Trade;
using DINOForge.Domains.Economy.Validation;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Economy
{
    /// <summary>
    /// Entry point for the Economy domain plugin. Provides access to all economy subsystems:
    /// production calculation, trade route evaluation, balance analysis, and pack validation.
    /// </summary>
    public sealed class EconomyPlugin
    {
        /// <summary>
        /// Calculator for building and faction production rates.
        /// </summary>
        public ProductionCalculator Production { get; }

        /// <summary>
        /// Engine for trade route evaluation and optimization.
        /// </summary>
        public TradeEngine Trade { get; }

        /// <summary>
        /// Calculator for generating economy balance reports across factions.
        /// </summary>
        public EconomyBalanceCalculator Balance { get; }

        /// <summary>
        /// Validator for economy pack content (profiles, trade routes, buildings).
        /// </summary>
        public EconomyValidator Validator { get; }

        private readonly RegistryManager _registries;

        /// <summary>
        /// Initialize the economy plugin with pre-loaded registries.
        /// </summary>
        /// <param name="registries">The registry manager containing all loaded content.</param>
        public EconomyPlugin(RegistryManager registries)
        {
            _registries = registries ?? throw new ArgumentNullException(nameof(registries));

            Production = new ProductionCalculator();
            Trade = new TradeEngine();
            Balance = new EconomyBalanceCalculator(Production, Trade);
            Validator = new EconomyValidator();
        }

        /// <summary>
        /// Validate a complete economy pack. Checks:
        /// - All production buildings reference valid resource types
        /// - All trade routes have valid source/target resources
        /// - All economy profiles have non-negative starting resources
        /// - No circular trade dependencies allow infinite resource generation
        /// - Exchange rates and modifiers are within reasonable bounds
        /// </summary>
        /// <param name="packId">The pack identifier to scope validation to.</param>
        /// <param name="profiles">Economy profiles defined in this pack.</param>
        /// <param name="tradeRoutes">Trade routes defined in this pack.</param>
        /// <returns>A validation result with errors, warnings, and overall validity.</returns>
        public EconomyValidationResult ValidatePack(
            string packId,
            IReadOnlyList<EconomyProfile> profiles,
            IReadOnlyList<TradeRoute> tradeRoutes)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (profiles == null) throw new ArgumentNullException(nameof(profiles));
            if (tradeRoutes == null) throw new ArgumentNullException(nameof(tradeRoutes));

            return Validator.Validate(packId, _registries, profiles, tradeRoutes);
        }

        /// <summary>
        /// Generate a comprehensive economy balance report for all factions in a pack.
        /// </summary>
        /// <param name="packId">The pack identifier to analyze.</param>
        /// <param name="profiles">Economy profiles keyed by faction ID.</param>
        /// <param name="tradeRoutes">Available trade routes.</param>
        /// <returns>An economy balance report with per-faction summaries and metrics.</returns>
        public EconomyBalanceReport GenerateBalanceReport(
            string packId,
            IReadOnlyDictionary<string, EconomyProfile> profiles,
            IReadOnlyList<TradeRoute> tradeRoutes)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (profiles == null) throw new ArgumentNullException(nameof(profiles));
            if (tradeRoutes == null) throw new ArgumentNullException(nameof(tradeRoutes));

            return Balance.GenerateReport(packId, _registries, profiles, tradeRoutes);
        }
    }
}
