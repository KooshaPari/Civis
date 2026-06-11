#nullable enable

using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tests.SDK
{
    public class ContentRegistrationServiceCoverageTests : IDisposable
    {
        private readonly string _tempRoot;

        public ContentRegistrationServiceCoverageTests()
        {
            _tempRoot = Path.Combine(Path.GetTempPath(), "dinoforge_content_registration_cov_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
            {
                try
                {
                    Directory.Delete(_tempRoot, true);
                }
                catch
                {
                    // Best-effort cleanup.
                }
            }
        }

        [Fact]
        public void Constructor_ExposesLoadedOverridesList()
        {
            List<StatOverrideDefinition> loadedOverrides = new List<StatOverrideDefinition>();
            RegistryManager registries = new RegistryManager();

            RegistryImportService service = CreateService(registries, loadedOverrides: loadedOverrides);

            service.LoadedOverrides.Should().BeSameAs(loadedOverrides);
            service.LoadedOverrides.Should().BeEmpty();
        }

        [Fact]
        public void SetPatchedYaml_UsesPatchedContentInsteadOfDisk()
        {
            RegistryManager registries = new RegistryManager();
            RegistryImportService service = CreateService(registries);
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string filePath = WriteYamlFile("units.yaml", @"
- id: file_unit
  display_name: File Unit
  unit_class: CoreLineInfantry
  faction_id: file-faction
  tier: 1
  stats:
    hp: 100
");

            service.SetPatchedYaml(filePath, @"
- id: patched_unit
  display_name: Patched Unit
  unit_class: HeavyInfantry
  faction_id: patched-faction
  tier: 2
  stats:
    hp: 220
");

            service.LoadAndRegisterContent(filePath, "units", manifest, errors);

            errors.Should().BeEmpty();
            registries.Units.Contains("patched_unit").Should().BeTrue();
            registries.Units.Contains("file_unit").Should().BeFalse();
        }

        [Fact]
        public void LoadAndRegisterContent_MissingFile_AddsReadError()
        {
            RegistryManager registries = new RegistryManager();
            RegistryImportService service = CreateService(registries);
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string missingPath = Path.Combine(_tempRoot, "missing.yaml");

            service.LoadAndRegisterContent(missingPath, "units", manifest, errors);

            errors.Should().ContainSingle(error => error.Contains("Failed to read") && error.Contains(missingPath));
            registries.Units.All.Should().BeEmpty();
        }

        [Fact]
        public void LoadAndRegisterContent_UnknownContentType_SkipsWithoutErrors()
        {
            RegistryManager registries = new RegistryManager();
            RegistryImportService service = CreateService(registries);
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string filePath = WriteYamlFile("unknown.yaml", "value: 1");

            service.LoadAndRegisterContent(filePath, "unknown_type", manifest, errors);

            errors.Should().BeEmpty();
            registries.Units.All.Should().BeEmpty();
            registries.Buildings.All.Should().BeEmpty();
            registries.FactionPatches.All.Should().BeEmpty();
        }

        [Fact]
        public void LoadAndRegisterContent_RegistersUnitList()
        {
            RegistryManager registries = new RegistryManager();
            RegistryImportService service = CreateService(registries);
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string filePath = WriteYamlFile("unit-list.yaml", @"
- id: list_unit
  display_name: List Unit
  unit_class: CoreLineInfantry
  faction_id: list-faction
  tier: 1
  stats:
    hp: 90
    damage: 12
");

            service.LoadAndRegisterContent(filePath, "units", manifest, errors);

            errors.Should().BeEmpty();
            registries.Units.Contains("list_unit").Should().BeTrue();
            UnitDefinition? unit = registries.Units.Get("list_unit");
            unit.Should().NotBeNull();
            unit!.DisplayName.Should().Be("List Unit");
        }

        [Fact]
        public void LoadAndRegisterContent_RegistersStatOverrides()
        {
            RegistryManager registries = new RegistryManager();
            List<StatOverrideDefinition> loadedOverrides = new List<StatOverrideDefinition>();
            RegistryImportService service = CreateService(registries, loadedOverrides: loadedOverrides);
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string filePath = WriteYamlFile("stats.yaml", @"
- overrides:
  - target: unit.stats.hp
    value: 15
    mode: Add
");

            service.LoadAndRegisterContent(filePath, "stats", manifest, errors);

            errors.Should().BeEmpty();
            service.LoadedOverrides.Should().BeSameAs(loadedOverrides);
            loadedOverrides.Should().ContainSingle();
            loadedOverrides[0].Overrides.Should().ContainSingle();
            loadedOverrides[0].Overrides[0].Target.Should().Be("unit.stats.hp");
        }

        [Fact]
        public void LoadAndRegisterContent_SchemaValidationFailure_AddsErrors()
        {
            RegistryManager registries = new RegistryManager();
            RegistryImportService service = CreateService(registries, new FailingSchemaValidator());
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string filePath = WriteYamlFile("invalid-unit.yaml", @"
- id: bad_unit
  display_name: Bad Unit
  unit_class: CoreLineInfantry
  faction_id: bad-faction
  tier: 1
  stats:
    hp: 80
");

            service.LoadAndRegisterContent(filePath, "units", manifest, errors);

            errors.Should().ContainSingle(error => error.Contains("Validation error in") && error.Contains("[id]") && error.Contains("broken"));
            registries.Units.Contains("bad_unit").Should().BeFalse();
        }

        [Fact]
        public void LoadAndRegisterContent_SchemaValidatorThrows_AddsSchemaError()
        {
            RegistryManager registries = new RegistryManager();
            RegistryImportService service = CreateService(registries, new ThrowingSchemaValidator());
            PackManifest manifest = CreateManifest();
            List<string> errors = new List<string>();
            string filePath = WriteYamlFile("throwing-unit.yaml", @"
- id: throw_unit
  display_name: Throw Unit
  unit_class: CoreLineInfantry
  faction_id: throw-faction
  tier: 1
  stats:
    hp: 80
");

            service.LoadAndRegisterContent(filePath, "units", manifest, errors);

            errors.Should().ContainSingle(error => error.Contains("Schema validation failed for") && error.Contains("schema='unit'") && error.Contains("InvalidOperationException"));
            registries.Units.Contains("throw_unit").Should().BeFalse();
        }

        private static RegistryImportService CreateService(
            RegistryManager registries,
            ISchemaValidator? schemaValidator = null,
            List<StatOverrideDefinition>? loadedOverrides = null)
        {
            List<StatOverrideDefinition> overrides = loadedOverrides ?? new List<StatOverrideDefinition>();

            return new RegistryImportService(
                registries,
                schemaValidator,
                new SchemaResolverService(),
                new DeserializerBuilder()
                    .WithNamingConvention(UnderscoredNamingConvention.Instance)
                    .IgnoreUnmatchedProperties()
                    .Build(),
                overrides,
                _ => { });
        }

        private PackManifest CreateManifest()
        {
            return new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                LoadOrder = 100
            };
        }

        private string WriteYamlFile(string fileName, string yaml)
        {
            string filePath = Path.Combine(_tempRoot, fileName);
            string? directory = Path.GetDirectoryName(filePath);
            if (!string.IsNullOrEmpty(directory))
            {
                Directory.CreateDirectory(directory);
            }

            File.WriteAllText(filePath, yaml);
            return filePath;
        }

        private sealed class FailingSchemaValidator : ISchemaValidator
        {
            public ValidationResult Validate(string schemaName, string yamlContent)
            {
                return ValidationResult.Failure(new List<ValidationError>
                {
                    new ValidationError("id", "broken", "required")
                });
            }
        }

        private sealed class ThrowingSchemaValidator : ISchemaValidator
        {
            public ValidationResult Validate(string schemaName, string yamlContent)
            {
                throw new InvalidOperationException("validator exploded");
            }
        }
    }
}
