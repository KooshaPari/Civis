using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;
using System.Diagnostics.CodeAnalysis;
using YamlDotNet.Serialization;

namespace DINOForge.SDK
{
    /// <summary>
    /// Implementation of <see cref="IRegistryImportService"/> for content registration.
    /// Handles YAML reading, schema validation, deserialization, and registry population.
    /// </summary>
    [ExcludeFromCodeCoverage] // Complex registry wiring — integration tests only
    internal sealed class RegistryImportService : IRegistryImportService
    {
        private readonly RegistryManager _registryManager;
        private readonly ISchemaValidator? _schemaValidator;
        private readonly ISchemaResolverService _schemaResolver;
        private readonly IDeserializer _deserializer;
        private readonly IList<StatOverrideDefinition> _loadedOverrides;
        private readonly Action<string> _log;

        /// <inheritdoc/>
        public IReadOnlyList<StatOverrideDefinition> LoadedOverrides => (IReadOnlyList<StatOverrideDefinition>)_loadedOverrides;

        /// <summary>
        /// Initializes a new <see cref="RegistryImportService"/>.
        /// </summary>
        public RegistryImportService(
            RegistryManager registryManager,
            ISchemaValidator? schemaValidator,
            ISchemaResolverService schemaResolver,
            IDeserializer deserializer,
            IList<StatOverrideDefinition> loadedOverrides,
            Action<string> log)
        {
            _registryManager = registryManager;
            _schemaValidator = schemaValidator;
            _schemaResolver = schemaResolver;
            _deserializer = deserializer;
            _loadedOverrides = loadedOverrides;
            _log = log;
        }

        /// <inheritdoc/>
        public void LoadAndRegisterContent(
            string yamlFilePath,
            string contentType,
            PackManifest manifest,
            IList<string> errors)
        {
            string yamlContent;
            try
            {
                yamlContent = File.ReadAllText(yamlFilePath);
            }
            catch (Exception ex)
            {
                errors.Add($"Failed to read {yamlFilePath}: {ex.Message}");
                return;
            }

            if (!ValidateContent(yamlFilePath, contentType, yamlContent, errors))
            {
                return;
            }

            try
            {
                RegisterContent(yamlContent, contentType, manifest);
                _log($"[ContentLoader] Registered {contentType} from {Path.GetFileName(yamlFilePath)}");
            }
            catch (Exception ex)
            {
                errors.Add($"Failed to deserialize/register {yamlFilePath}: {ex.Message}");
            }
        }

        private bool ValidateContent(
            string yamlFilePath,
            string contentType,
            string yamlContent,
            IList<string> errors)
        {
            if (_schemaValidator == null ||
                !_schemaResolver.TryResolveSchemaName(contentType, out string schemaName))
            {
                return true;
            }

            try
            {
                ValidationResult validationResult = _schemaValidator.Validate(schemaName, yamlContent);
                if (validationResult.IsValid)
                {
                    return true;
                }

                foreach (ValidationError error in validationResult.Errors)
                {
                    errors.Add($"Validation error in {yamlFilePath} [{error.Path}]: {error.Message}");
                }

                return false;
            }
            catch (Exception ex)
            {
                _log($"[ContentLoader] Schema validation skipped for {yamlFilePath}: {ex.Message}");
                return true;
            }
        }

        private void RegisterContent(string yamlContent, string contentType, PackManifest manifest)
        {
            switch (contentType.ToLowerInvariant())
            {
                case "units":
                    RegisterItems<UnitDefinition>(
                        yamlContent,
                        unit => _registryManager.Units.Register(unit.Id, unit, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                case "buildings":
                    RegisterItems<BuildingDefinition>(
                        yamlContent,
                        building => _registryManager.Buildings.Register(building.Id, building, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                case "factions":
                    RegisterItems<FactionDefinition>(
                        yamlContent,
                        faction => _registryManager.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                case "weapons":
                    RegisterItems<WeaponDefinition>(
                        yamlContent,
                        weapon => _registryManager.Weapons.Register(weapon.Id, weapon, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                case "projectiles":
                    RegisterItems<ProjectileDefinition>(
                        yamlContent,
                        projectile => _registryManager.Projectiles.Register(projectile.Id, projectile, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                case "doctrines":
                    RegisterItems<DoctrineDefinition>(
                        yamlContent,
                        doctrine => _registryManager.Doctrines.Register(doctrine.Id, doctrine, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                case "stats":
                    RegisterItems<StatOverrideDefinition>(
                        yamlContent,
                        statOverride => _loadedOverrides.Add(statOverride));
                    break;

                case "faction_patches":
                    RegisterItems<FactionPatchDefinition>(
                        yamlContent,
                        patch => _registryManager.FactionPatches.Register(patch.TargetFaction, patch, RegistrySource.Pack, manifest.Id, manifest.LoadOrder));
                    break;

                default:
                    _log($"[ContentLoader] Unknown content type '{contentType}', skipping.");
                    break;
            }
        }

        private void RegisterItems<T>(string yamlContent, Action<T> register) where T : class
        {
            List<T>? items = null;
            try
            {
                items = _deserializer.Deserialize<List<T>>(yamlContent);
            }
            catch
            {
                // Not a list — fall through to single-object parse.
            }

            if (items != null && items.Count > 0)
            {
                foreach (T item in items)
                {
                    register(item);
                }

                return;
            }

            T single = _deserializer.Deserialize<T>(yamlContent);
            if (single != null)
            {
                register(single);
            }
        }
    }
}
