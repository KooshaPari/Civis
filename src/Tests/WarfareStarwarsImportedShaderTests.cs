using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Pins warfare-starwars imported JSON and PackCompiler import metadata to URP Lit,
/// not the legacy Built-in <c>Standard</c> shader name.
/// </summary>
public class WarfareStarwarsImportedShaderTests
{
    private const string ForbiddenShader = "Standard";
    private const string ExpectedUrpLitShader = "Universal Render Pipeline/Lit";

    [Fact]
    public void WarfareStarwars_ImportedJson_Materials_DoNotReferenceStandardShader()
    {
        string importedDir = Path.Combine(GetRepoRoot(), "packs", "warfare-starwars", "assets", "imported");
        Directory.Exists(importedDir).Should().BeTrue("imported metadata directory must exist");

        var violations = new List<string>();
        foreach (string jsonPath in Directory.EnumerateFiles(importedDir, "*.json"))
        {
            using var doc = JsonDocument.Parse(File.ReadAllText(jsonPath));
            if (!doc.RootElement.TryGetProperty("materials", out JsonElement materials) ||
                materials.ValueKind != JsonValueKind.Array)
            {
                continue;
            }

            foreach (JsonElement material in materials.EnumerateArray())
            {
                if (!material.TryGetProperty("shaderName", out JsonElement shaderName))
                {
                    continue;
                }

                if (shaderName.GetString() == ForbiddenShader)
                {
                    violations.Add(Path.GetFileName(jsonPath));
                }
            }
        }

        violations.Should().BeEmpty(
            "imported warfare-starwars JSON must not use shaderName Standard; metadata should be URP Lit");
    }

    [Fact]
    public void AssetImportService_DefaultImportedShader_IsUrpLit()
    {
        string servicePath = Path.Combine(
            GetRepoRoot(),
            "src",
            "Tools",
            "PackCompiler",
            "Services",
            "AssetImportService.cs");
        File.Exists(servicePath).Should().BeTrue();

        string source = File.ReadAllText(servicePath);
        source.Should().Contain(
            $"DefaultImportedShaderName = \"{ExpectedUrpLitShader}\"",
            "PackCompiler imports must default to URP Lit per ASSET_PIPELINE.md");
    }

    private static string GetRepoRoot()
    {
        string? current = Path.GetDirectoryName(typeof(WarfareStarwarsImportedShaderTests).Assembly.Location);
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "global.json")))
            {
                return current;
            }

            current = Path.GetDirectoryName(current);
        }

        current = Directory.GetCurrentDirectory();
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "global.json")))
            {
                return current;
            }

            current = Path.GetDirectoryName(current);
        }

        throw new DirectoryNotFoundException("Could not locate repo root (global.json)");
    }
}
