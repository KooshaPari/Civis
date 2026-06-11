#nullable enable
using System;
using System.IO;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK;

[Trait("Category", "SDK")]
public sealed class FileDiscoveryServiceCoverageTests : IDisposable
{
    private readonly string _root;

    public FileDiscoveryServiceCoverageTests()
    {
        _root = Path.Combine(Path.GetTempPath(), "dinoforge_file_discovery_coverage_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_root);
    }

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_root))
            {
                Directory.Delete(_root, recursive: true);
            }
        }
        catch
        {
            // Best-effort cleanup for temp test artifacts.
        }
    }

    [Fact]
    public void DefaultConstructor_ExposesExpectedDefaultExclusions()
    {
        var service = new FileDiscoveryService();

        service.DefaultExclusions.Should().Contain("bin");
        service.DefaultExclusions.Should().Contain("obj");
        service.DefaultExclusions.Should().Contain(".git");
        service.DefaultExclusions.Should().NotBeEmpty();
    }

    [Fact]
    public void BooleanConstructor_WithUseDefaultsFalse_KeepsUnderscoreDirectoriesVisible()
    {
        string visibleFile = WriteFile("_scratch", "visible.txt");
        var service = new FileDiscoveryService(useDefaults: false);

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().ContainSingle().Which.Should().Be(visibleFile);
    }

    [Fact]
    public void EnumerableConstructor_WithCustomExclusions_UsesOnlyCustomRules()
    {
        string excludedFile = WriteFile("generated", "blocked.txt");
        string includedFile = WriteFile("content", "allowed.txt");
        var service = new FileDiscoveryService(new[] { "generated" });

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().ContainSingle().Which.Should().Be(includedFile);
        result.Should().NotContain(excludedFile);
    }

    [Fact]
    public void GetFiles_SinglePattern_ReturnsSortedMatches()
    {
        string alpha = WriteFile("content", "alpha.txt");
        string beta = WriteFile("content", "beta.txt");
        var service = new FileDiscoveryService();

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().ContainInOrder(alpha, beta);
    }

    [Fact]
    public void GetFiles_MultiplePatterns_DeduplicatesAndSortsResults()
    {
        string alpha = WriteFile("content", "alpha.txt");
        string beta = WriteFile("content", "beta.json");
        var service = new FileDiscoveryService();

        string[] result = service.GetFiles(_root, new[] { "*.txt", "*.*" }, SearchOption.AllDirectories);

        result.Should().Contain(alpha);
        result.Should().Contain(beta);
        result.Should().OnlyHaveUniqueItems();
        result.Should().ContainInOrder(alpha, beta);
    }

    [Fact]
    public void GetDirectories_TopLevel_ExcludesDefaultDirectories()
    {
        string keepDirectory = CreateDirectory("content");
        CreateDirectory("bin");
        CreateDirectory("obj");
        CreateDirectory(".git");

        var service = new FileDiscoveryService();

        string[] result = service.GetDirectories(_root, SearchOption.TopDirectoryOnly);

        result.Should().ContainSingle().Which.Should().Be(keepDirectory);
    }

    [Fact]
    public void DiscoverPackDirectories_ReturnsOnlyDirectoriesWithPackYaml()
    {
        string packDirectory = CreateDirectory("pack-a");
        WriteFile("pack-a", "pack.yaml");
        CreateDirectory("pack-b.disabled");
        WriteFile("pack-b.disabled", "pack.yaml");
        CreateDirectory("pack-c");
        WriteFile("pack-c", "notes.txt");
        var service = new FileDiscoveryService();

        string[] result = service.DiscoverPackDirectories(_root);

        result.Should().ContainSingle().Which.Should().Be(packDirectory);
    }

    [Fact]
    public void AddExclusion_TrimsPatternAndFiltersMatchingFiles()
    {
        string excludedFile = WriteFile("temp", "blocked.txt");
        string includedFile = WriteFile("content", "allowed.txt");
        var service = new FileDiscoveryService(useDefaults: false);

        service.AddExclusion(" temp ");

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().ContainSingle().Which.Should().Be(includedFile);
        result.Should().NotContain(excludedFile);
    }

    [Fact]
    public void RemoveExclusion_AllowsMatchingFilesAgain()
    {
        string tempFile = WriteFile("temp", "blocked.txt");
        var service = new FileDiscoveryService(useDefaults: false);
        service.AddExclusion("temp");

        service.GetFiles(_root, "*.txt", SearchOption.AllDirectories).Should().BeEmpty();

        service.RemoveExclusion(" temp ");

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().ContainSingle().Which.Should().Be(tempFile);
    }

    [Fact]
    public void ClearExclusions_RemovesCustomExclusions()
    {
        string excludedFile = WriteFile("generated", "blocked.txt");
        string includedFile = WriteFile("content", "allowed.txt");
        var service = new FileDiscoveryService(new[] { "generated" });

        service.GetFiles(_root, "*.txt", SearchOption.AllDirectories).Should().ContainSingle().Which.Should().Be(includedFile);

        service.ClearExclusions();

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().Contain(excludedFile);
        result.Should().Contain(includedFile);
    }

    [Fact]
    public void ResetToDefaults_RestoresDefaultExclusions()
    {
        string excludedDirectory = CreateDirectory("generated");
        string excludedFile = WriteFile("generated", "blocked.txt");
        var service = new FileDiscoveryService();
        service.ClearExclusions();

        service.GetFiles(_root, "*.txt", SearchOption.AllDirectories).Should().Contain(excludedFile);

        service.ResetToDefaults();

        string[] result = service.GetFiles(_root, "*.txt", SearchOption.AllDirectories);

        result.Should().NotContain(excludedFile);
        result.Should().NotContain(Path.Combine(excludedDirectory, "blocked.txt"));
    }

    private string CreateDirectory(string relativePath)
    {
        string directory = Path.Combine(_root, relativePath);
        Directory.CreateDirectory(directory);
        return directory;
    }

    private string WriteFile(string relativeDirectory, string fileName)
    {
        string directory = CreateDirectory(relativeDirectory);
        string path = Path.Combine(directory, fileName);
        File.WriteAllText(path, "test");
        return path;
    }
}
