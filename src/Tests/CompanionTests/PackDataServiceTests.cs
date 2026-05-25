using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;

namespace DINOForge.DesktopCompanion.Tests;

/// <summary>
/// Tests for pack loading logic — uses real filesystem with temp directories.
/// Tests the YAML parsing contract that FileSystemPackDataService must satisfy.
/// </summary>
public class PackDataServiceTests : IDisposable
{
    private readonly string _tempDir;
    private readonly IDeserializer _yaml = new DeserializerBuilder().Build();

    public PackDataServiceTests()
        => _tempDir = Directory.CreateTempSubdirectory("df-companion-test-").FullName;

    public void Dispose() => Directory.Delete(_tempDir, recursive: true);

    private void WritePackYaml(string packId, string yaml)
    {
        string dir = Path.Combine(_tempDir, packId);
        Directory.CreateDirectory(dir);
        File.WriteAllText(Path.Combine(dir, "pack.yaml"), yaml);
    }

    [Fact]
    public void ParsePackYaml_ValidMinimal_ExtractsIdAndName()
    {
        WritePackYaml("test-pack", """
            id: test-pack
            name: Test Pack
            version: 0.1.0
            """);

        string yaml = File.ReadAllText(Path.Combine(_tempDir, "test-pack", "pack.yaml"));
        Dictionary<string, object> parsed = _yaml.Deserialize<Dictionary<string, object>>(yaml);

        parsed["id"].Should().Be("test-pack");
        parsed["name"].Should().Be("Test Pack");
        parsed["version"].Should().Be("0.1.0");
    }

    [Fact]
    public void ParsePackYaml_WithAllFields_ExtractsAll()
    {
        WritePackYaml("full-pack", """
            id: full-pack
            name: Full Pack
            version: 1.2.3
            author: Test Author
            type: content
            framework_version: ">=0.1.0"
            depends_on: []
            conflicts_with: []
            """);

        string yaml = File.ReadAllText(Path.Combine(_tempDir, "full-pack", "pack.yaml"));
        Dictionary<string, object> parsed = _yaml.Deserialize<Dictionary<string, object>>(yaml);

        parsed.Should().ContainKey("author");
        parsed["author"].Should().Be("Test Author");
        parsed.Should().ContainKey("type");
        parsed["type"].Should().Be("content");
    }

    [Fact]
    public void ScanDirectory_MultiplePackDirs_FindsAll()
    {
        WritePackYaml("pack-a", "id: pack-a\nname: A\nversion: 0.1.0");
        WritePackYaml("pack-b", "id: pack-b\nname: B\nversion: 0.2.0");
        WritePackYaml("pack-c", "id: pack-c\nname: C\nversion: 0.3.0");

        string[] packDirs = Directory.GetDirectories(_tempDir)
            .Where(d => File.Exists(Path.Combine(d, "pack.yaml")))
            .ToArray();

        packDirs.Should().HaveCount(3);
    }

    [Fact]
    public void ScanDirectory_EmptyDir_ReturnsNoPacks()
    {
        string[] packDirs = Directory.GetDirectories(_tempDir)
            .Where(d => File.Exists(Path.Combine(d, "pack.yaml")))
            .ToArray();

        packDirs.Should().BeEmpty();
    }

    [Fact]
    public void ScanDirectory_NonPackSubdirIgnored_OnlyPacksWithYamlCount()
    {
        WritePackYaml("real-pack", "id: real-pack\nname: Real\nversion: 0.1.0");
        Directory.CreateDirectory(Path.Combine(_tempDir, "not-a-pack")); // no pack.yaml

        string[] packDirs = Directory.GetDirectories(_tempDir)
            .Where(d => File.Exists(Path.Combine(d, "pack.yaml")))
            .ToArray();

        packDirs.Should().HaveCount(1);
    }

    [Fact]
    public void ParsePackYaml_MissingOptionalFields_DoesNotThrow()
    {
        WritePackYaml("minimal", "id: minimal\nname: Min\nversion: 0.0.1");
        string yaml = File.ReadAllText(Path.Combine(_tempDir, "minimal", "pack.yaml"));

        Action act = () => _yaml.Deserialize<Dictionary<string, object>>(yaml);
        act.Should().NotThrow();
    }
}
