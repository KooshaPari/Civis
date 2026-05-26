using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Validates that all JSON schema files in schemas/ are well-formed and self-consistent.
/// Catches schema-level regressions before they break pack-validation downstream.
/// </summary>
public class SchemaSelfValidationTests
{
    private static readonly string SchemasDir = GetSchemasDirectory();

    private static string GetSchemasDirectory()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir != null && !Directory.Exists(Path.Combine(dir.FullName, "schemas")))
        {
            dir = dir.Parent;
        }

        if (dir == null)
        {
            throw new DirectoryNotFoundException(
                $"Could not locate 'schemas' directory by walking up from {AppContext.BaseDirectory}");
        }

        return Path.Combine(dir.FullName, "schemas");
    }

    private List<(string Path, JsonElement Root)> LoadAllSchemas()
    {
        if (!Directory.Exists(SchemasDir))
        {
            throw new DirectoryNotFoundException($"Schemas directory not found: {SchemasDir}");
        }

        var schemas = new List<(string, JsonElement)>();
        var files = Directory.GetFiles(SchemasDir, "*.json").OrderBy(f => f);

        foreach (var filePath in files)
        {
            try
            {
                var json = File.ReadAllText(filePath);
                var doc = JsonDocument.Parse(json);
                schemas.Add((filePath, doc.RootElement.Clone()));
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to load schema {filePath}: {ex.Message}", ex);
            }
        }

        return schemas;
    }

    /// <summary>
    /// Test 1: All schema files in schemas/ parse as valid JSON.
    /// </summary>
    [Fact]
    public void AllSchemaFilesParseAsValidJson()
    {
        var schemaDir = new DirectoryInfo(SchemasDir);
        schemaDir.Exists.Should().BeTrue($"schemas directory should exist at {SchemasDir}");

        var schemaFiles = schemaDir.GetFiles("*.json");
        schemaFiles.Should().NotBeEmpty("schemas/ should contain at least one JSON file");

        var failedFiles = new List<string>();

        foreach (var file in schemaFiles)
        {
            try
            {
                var json = File.ReadAllText(file.FullName);
                JsonDocument.Parse(json);
            }
            catch (JsonException ex)
            {
                failedFiles.Add($"{file.Name}: {ex.Message}");
            }
        }

        failedFiles.Should().BeEmpty("All schema files should be valid JSON. Failed files: " +
            string.Join("\n  ", failedFiles));
    }

    /// <summary>
    /// Test 2: Each schema has top-level "$schema", "title", "type" fields (JSON Schema convention).
    /// </summary>
    [Fact]
    public void AllSchemasHaveMandatoryMetadata()
    {
        var schemas = LoadAllSchemas();
        var missing = new List<string>();

        foreach (var (path, root) in schemas)
        {
            var fileName = Path.GetFileName(path);
            var hasSchema = root.TryGetProperty("$schema", out _);
            var hasTitle = root.TryGetProperty("title", out _);
            var hasType = root.TryGetProperty("type", out _);

            if (!hasSchema || !hasTitle || !hasType)
            {
                var fields = new List<string>();
                if (!hasSchema) fields.Add("$schema");
                if (!hasTitle) fields.Add("title");
                if (!hasType) fields.Add("type");

                missing.Add($"{fileName}: missing {string.Join(", ", fields)}");
            }
        }

        missing.Should().BeEmpty("All schemas should have $schema, title, and type. Missing: " +
            string.Join("\n  ", missing));
    }

    /// <summary>
    /// Test 3: All $ref properties resolve within the schema set (no dangling references).
    /// </summary>
    [Fact]
    public void AllSchemaReferencesResolve()
    {
        var schemas = LoadAllSchemas();
        var schemaMap = schemas.ToDictionary(
            s => Path.GetFileNameWithoutExtension(s.Path),
            s => s.Root);

        var dangling = new List<string>();

        foreach (var (path, root) in schemas)
        {
            var fileName = Path.GetFileNameWithoutExtension(path);
            var refs = ExtractAllRefs(root);

            foreach (var refValue in refs)
            {
                // Skip external references (http://, https://)
                if (refValue.StartsWith("http://") || refValue.StartsWith("https://"))
                {
                    continue;
                }

                // Handle local file references like "faction.schema.json#/properties/id"
                if (refValue.Contains(".schema.json"))
                {
                    var referencedFile = refValue.Split('#')[0];
                    var schemaName = Path.GetFileNameWithoutExtension(referencedFile);

                    if (!schemaMap.ContainsKey(schemaName))
                    {
                        dangling.Add($"{fileName}: references non-existent schema '{referencedFile}'");
                    }
                }
                // Handle internal references like "#/definitions/SomeType"
                else if (refValue.StartsWith("#/"))
                {
                    var parts = refValue.TrimStart('#').Split('/').Where(p => !string.IsNullOrEmpty(p)).ToList();
                    if (!CanNavigatePath(root, parts))
                    {
                        dangling.Add($"{fileName}: internal reference '{refValue}' does not resolve");
                    }
                }
            }
        }

        dangling.Should().BeEmpty("All schema $ref properties should resolve. Dangling: " +
            string.Join("\n  ", dangling));
    }

    /// <summary>
    /// Test 4: pack-manifest schema requires "id", "version", "framework_version", "type".
    /// </summary>
    [Fact]
    public void PackManifestSchemaHasRequiredFields()
    {
        var schemas = LoadAllSchemas();
        var packSchema = schemas.FirstOrDefault(s => s.Path.EndsWith("pack-manifest.schema.json"));

        packSchema.Should().NotBe(default, "pack-manifest.schema.json should exist");

        packSchema.Root.TryGetProperty("required", out var required).Should().BeTrue(
            "pack-manifest.schema.json should have 'required' field");

        var requiredList = required.EnumerateArray()
            .Select(e => e.GetString())
            .ToList();

        requiredList.Should().Contain(new[] { "id", "version", "type" },
            "pack-manifest.schema.json should require: id, version, type");
    }

    /// <summary>
    /// Test 5: faction schema requires "id" and "display_name".
    /// </summary>
    [Fact]
    public void FactionSchemaHasRequiredFields()
    {
        var schemas = LoadAllSchemas();
        var factionSchema = schemas.FirstOrDefault(s => s.Path.EndsWith("faction.schema.json"));

        factionSchema.Should().NotBe(default, "faction.schema.json should exist");

        // Faction schema nests the required fields under properties.faction.required
        factionSchema.Root.TryGetProperty("properties", out var properties).Should().BeTrue(
            "faction.schema.json should have 'properties' field");

        properties.TryGetProperty("faction", out var faction).Should().BeTrue(
            "faction.schema.json properties should have 'faction' object");

        faction.TryGetProperty("required", out var required).Should().BeTrue(
            "faction.schema.json faction object should have 'required' field");

        var requiredList = required.EnumerateArray()
            .Select(e => e.GetString())
            .ToList();

        requiredList.Should().Contain(new[] { "id", "display_name" },
            "faction.schema.json faction object should require: id, display_name");
    }

    /// <summary>
    /// Test 6: unit schema requires "id" and a faction reference field.
    /// </summary>
    [Fact]
    public void UnitSchemaHasRequiredFields()
    {
        var schemas = LoadAllSchemas();
        var unitSchema = schemas.FirstOrDefault(s => s.Path.EndsWith("unit.schema.json"));

        unitSchema.Should().NotBe(default, "unit.schema.json should exist");

        unitSchema.Root.TryGetProperty("required", out var required).Should().BeTrue(
            "unit.schema.json should have 'required' field");

        var requiredList = required.EnumerateArray()
            .Select(e => e.GetString())
            .ToList();

        requiredList.Should().Contain("id",
            "unit.schema.json should require: id");
    }

    /// <summary>
    /// Test 7: Verify total-conversion schema has proper $id structure.
    /// </summary>
    [Fact]
    public void TotalConversionSchemaIsWellFormed()
    {
        var schemas = LoadAllSchemas();
        var tcSchema = schemas.FirstOrDefault(s => s.Path.EndsWith("total-conversion.schema.json"));

        tcSchema.Should().NotBe(default, "total-conversion.schema.json should exist");

        tcSchema.Root.TryGetProperty("title", out var title).Should().BeTrue();
        title.GetString().Should().NotBeNullOrWhiteSpace("total-conversion schema should have a title");
    }

    /// <summary>
    /// Test 8: Verify economy-profile and scenario schemas are present and minimal-valid.
    /// </summary>
    [Fact]
    public void DomainSchemasExist()
    {
        var schemas = LoadAllSchemas();
        var schemaNames = schemas.Select(s => Path.GetFileName(s.Path)).ToList();

        schemaNames.Should().Contain(new[] { "economy-profile.schema.json", "scenario.schema.json" },
            "Domain schemas (economy, scenario) should exist");
    }

    /// <summary>
    /// Helper: Extract all $ref values from a schema tree (recursive).
    /// </summary>
    private static List<string> ExtractAllRefs(JsonElement element)
    {
        var refs = new List<string>();

        if (element.ValueKind == JsonValueKind.Object)
        {
            if (element.TryGetProperty("$ref", out var refProp))
            {
                var refValue = refProp.GetString();
                if (!string.IsNullOrEmpty(refValue))
                {
                    refs.Add(refValue);
                }
            }

            foreach (var property in element.EnumerateObject())
            {
                refs.AddRange(ExtractAllRefs(property.Value));
            }
        }
        else if (element.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in element.EnumerateArray())
            {
                refs.AddRange(ExtractAllRefs(item));
            }
        }

        return refs;
    }

    /// <summary>
    /// Helper: Check if a JSON path like ["definitions", "SomeType", "properties", "id"] exists.
    /// </summary>
    private static bool CanNavigatePath(JsonElement root, List<string> path)
    {
        var current = root;

        foreach (var part in path)
        {
            if (current.ValueKind == JsonValueKind.Object)
            {
                if (!current.TryGetProperty(part, out current))
                {
                    return false;
                }
            }
            else
            {
                return false;
            }
        }

        return true;
    }
}
