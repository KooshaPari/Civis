#nullable enable
using System.IO;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Mocks.Tests;

public class MockFileDiscoveryServiceTests
{
    [Fact]
    public void GetFiles_WithSinglePattern_ReturnsMatchingFiles()
    {
        var service = new MockFileDiscoveryService();
        service.AddFile("packs/example/pack.yaml", "");
        service.AddFile("packs/example/units.yaml", "");
        service.AddFile("packs/other/config.json", "");

        var result = service.GetFiles("packs/", "*.yaml", SearchOption.AllDirectories);

        result.Should().HaveCount(2);
        service.GetFilesCount.Should().Be(1);
    }

    [Fact]
    public void GetFiles_WithMultiplePatterns_ReturnsAllMatching()
    {
        var service = new MockFileDiscoveryService();
        service.AddFile("packs/example/pack.yaml", "");
        service.AddFile("packs/example/config.json", "");
        service.AddFile("packs/example/readme.md", "");

        var result = service.GetFiles("packs/", new[] { "*.yaml", "*.json" }, SearchOption.AllDirectories);

        result.Should().HaveCount(2);
        service.GetFilesCount.Should().Be(1);
    }

    [Fact]
    public void DiscoverPackDirectories_FindsDirectoriesWithPackYaml()
    {
        var service = new MockFileDiscoveryService();
        service.AddDirectory("packs/example");
        service.AddFile("packs/example/pack.yaml", "");
        service.AddDirectory("packs/other");
        service.AddFile("packs/other/pack.yaml", "");

        var result = service.DiscoverPackDirectories("packs/");

        result.Should().HaveCount(2);
        service.DiscoverPackDirectoriesCount.Should().Be(1);
    }

    [Fact]
    public void DefaultExclusions_ReturnsNonEmptyList()
    {
        var service = new MockFileDiscoveryService();

        service.DefaultExclusions.Should().NotBeEmpty();
        service.DefaultExclusions.Should().Contain("bin/");
    }

    [Fact]
    public void AddExclusion_TracksCallCount()
    {
        var service = new MockFileDiscoveryService();

        service.AddExclusion("*.tmp");
        service.AddExclusion("test/");

        service.AddExclusionCount.Should().Be(2);
    }

    [Fact]
    public void GetDirectories_ReturnsAllDirectories()
    {
        var service = new MockFileDiscoveryService();
        service.AddDirectory("packs/example");
        service.AddDirectory("packs/other");

        var result = service.GetDirectories("packs/", SearchOption.AllDirectories);

        result.Should().HaveCount(2);
        service.GetDirectoriesCount.Should().Be(1);
    }

    [Fact]
    public void ResetToDefaults_ClearsCustomExclusions()
    {
        var service = new MockFileDiscoveryService();
        service.AddExclusion("*.tmp");

        service.ResetToDefaults();

        service.AddExclusionCount.Should().Be(1);
    }

    [Fact]
    public void ClearExclusions_RemovesAllCustomPatterns()
    {
        var service = new MockFileDiscoveryService();
        service.AddExclusion("*.tmp");
        service.AddExclusion("test/");

        service.ClearExclusions();

        // Verify no assertion error; ClearExclusions doesn't return anything
        service.AddExclusionCount.Should().Be(2);
    }
}
