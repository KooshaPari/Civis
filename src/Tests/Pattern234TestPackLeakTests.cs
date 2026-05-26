using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Xunit;

namespace DINOForge.Tests.Patterns;

/// <summary>
/// Pattern #234: Test Fixture IDs Leaking Into Deployed Packs
/// Ensures test pack IDs (Test*, test-invalid, test-valid, etc.) do NOT appear
/// in production packs/ directory or MSBuild DeployPacks outputs.
/// </summary>
public class Pattern234TestPackLeakTests
{
    private static string GetRepoRoot()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        var maxIterations = 20;
        var iterations = 0;

        while (dir != null && iterations < maxIterations)
        {
            // global.json is the definitive marker for .NET repo root
            if (File.Exists(Path.Combine(dir.FullName, "global.json")))
            {
                return dir.FullName;
            }
            dir = dir.Parent;
            iterations++;
        }

        throw new InvalidOperationException($"Repo root not found starting from {AppContext.BaseDirectory}");
    }

    [Fact]
    public void NoTestPackIdsInProductionPacks()
    {
        // Arrange
        var repoRoot = GetRepoRoot();
        var packsDir = new DirectoryInfo(Path.Combine(repoRoot, "packs"));
        Assert.True(packsDir.Exists, $"packs/ directory should exist at {packsDir.FullName} (repo root: {repoRoot})");

        var testFixturesDir = new DirectoryInfo("src/Tests/Fixtures");
        var testPatterns = new[] { "Test", "test-invalid", "test-valid", "test-bad" };

        // Act: Find all pack.yaml files
        var packYamls = packsDir.GetFiles("pack.yaml", SearchOption.AllDirectories);
        var violations = new List<string>();

        foreach (var yaml in packYamls)
        {
            // Skip if in test fixtures
            if (yaml.FullName.Contains("Tests" + Path.DirectorySeparatorChar + "Fixtures"))
                continue;

            var content = File.ReadAllText(yaml.FullName);
            var idMatch = System.Text.RegularExpressions.Regex.Match(
                content,
                @"^\s*id:\s*[""']?([^\s""']+)[""']?",
                System.Text.RegularExpressions.RegexOptions.Multiline);

            if (idMatch.Success)
            {
                var packId = idMatch.Groups[1].Value;
                if (testPatterns.Any(p => packId.StartsWith(p, StringComparison.OrdinalIgnoreCase)))
                {
                    violations.Add($"{yaml.FullName}: test ID '{packId}'");
                }
            }
        }

        // Assert
        Assert.Empty(violations);
    }

    [Fact]
    public void DeployPacksCsprojContainsTestExclusion()
    {
        // Arrange
        var repoRoot = GetRepoRoot();
        var runtimeCsproj = new FileInfo(Path.Combine(repoRoot, "src", "Runtime", "DINOForge.Runtime.csproj"));
        Assert.True(runtimeCsproj.Exists, $"Runtime csproj should exist at {runtimeCsproj.FullName}");

        var content = File.ReadAllText(runtimeCsproj.FullName);

        // Act: Check for PackFiles + test exclusion pattern
        var hasPackFiles = content.Contains("<PackFiles Include=");
        var hasTestExclusion = content.Contains("Exclude=") &&
            (content.Contains("test-") || content.Contains("Test"));

        // Assert
        if (hasPackFiles)
        {
            Assert.True(hasTestExclusion,
                "DeployPacks should exclude test-* pack IDs to prevent fixture leaks");
        }
    }
}
