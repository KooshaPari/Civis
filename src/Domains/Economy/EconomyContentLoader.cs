using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Registries;
using DINOForge.SDK;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Domains.Economy
{
    /// <summary>
    /// Loads economy definitions from pack directories into economy registries.
    /// Handles resources/, trade_routes/, and economy_profiles/ subdirectories.
    /// </summary>
    public class EconomyContentLoader
    {
        private readonly ResourceRegistry _resourceRegistry;
        private readonly TradeRouteRegistry _tradeRouteRegistry;
        private readonly EconomyProfileRegistry _profileRegistry;
        private readonly IDeserializer _deserializer;

        /// <summary>
        /// Initializes a new economy content loader with the provided registries.
        /// </summary>
        public EconomyContentLoader(
            ResourceRegistry resourceRegistry,
            TradeRouteRegistry tradeRouteRegistry,
            EconomyProfileRegistry profileRegistry)
        {
            _resourceRegistry = resourceRegistry ?? throw new ArgumentNullException(nameof(resourceRegistry));
            _tradeRouteRegistry = tradeRouteRegistry ?? throw new ArgumentNullException(nameof(tradeRouteRegistry));
            _profileRegistry = profileRegistry ?? throw new ArgumentNullException(nameof(profileRegistry));

            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .Build();
        }

        /// <summary>
        /// Load all economy definitions from a pack directory.
        /// </summary>
        /// <param name="packDir">The root directory of the pack.</param>
        /// <param name="packId">The pack identifier (for logging).</param>
        public void LoadPack(string packDir, string packId)
        {
            if (!Directory.Exists(packDir))
                throw new DirectoryNotFoundException($"Pack directory not found: {packDir}");

            LoadResources(Path.Combine(packDir, "resources"), packId);
            LoadTradeRoutes(Path.Combine(packDir, "trade_routes"), packId);
            LoadProfiles(Path.Combine(packDir, "economy_profiles"), packId);
        }

        private void LoadResources(string resourcesDir, string packId)
        {
            if (!Directory.Exists(resourcesDir))
                return;

            string[] files = Directory.GetFiles(resourcesDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = File.ReadAllText(file);
                    ResourceDefinition resource = _deserializer.Deserialize<ResourceDefinition>(yaml);
                    if (resource != null)
                    {
                        _resourceRegistry.Register(resource);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load resource from {file} in pack '{packId}'.", ex);
                }
            }
        }

        private void LoadTradeRoutes(string routesDir, string packId)
        {
            if (!Directory.Exists(routesDir))
                return;

            string[] files = Directory.GetFiles(routesDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = File.ReadAllText(file);
                    TradeRouteWrapper wrapper = _deserializer.Deserialize<TradeRouteWrapper>(yaml);
                    if (wrapper?.Routes != null)
                    {
                        foreach (TradeRouteDefinition route in wrapper.Routes)
                        {
                            _tradeRouteRegistry.Register(route);
                        }
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load trade routes from {file} in pack '{packId}'.", ex);
                }
            }
        }

        private void LoadProfiles(string profilesDir, string packId)
        {
            if (!Directory.Exists(profilesDir))
                return;

            string[] files = Directory.GetFiles(profilesDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = File.ReadAllText(file);
                    EconomyProfile profile = _deserializer.Deserialize<EconomyProfile>(yaml);
                    if (profile != null)
                    {
                        _profileRegistry.Register(profile);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load economy profile from {file} in pack '{packId}'.", ex);
                }
            }
        }

        /// <summary>
        /// YAML wrapper for trade routes array.
        /// </summary>
        private class TradeRouteWrapper
        {
            public List<TradeRouteDefinition> Routes { get; set; } = new List<TradeRouteDefinition>();
        }
    }
}
