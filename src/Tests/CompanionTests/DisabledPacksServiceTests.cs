using FluentAssertions;
using System.Text.Json;
using Xunit;

namespace DINOForge.DesktopCompanion.Tests;

/// <summary>
/// Tests for the disabled_packs.json persistence contract.
/// Both the game and companion must use this exact format.
/// </summary>
public class DisabledPacksServiceTests : IDisposable
{
    private readonly string _tempDir;
    private readonly string _filePath;

    public DisabledPacksServiceTests()
    {
        _tempDir = Directory.CreateTempSubdirectory("df-disabled-test-").FullName;
        _filePath = Path.Combine(_tempDir, "disabled_packs.json");
    }

    public void Dispose() => Directory.Delete(_tempDir, recursive: true);

    [Fact]
    public async Task Write_EmptySet_ProducesEmptyJsonArray()
    {
        await File.WriteAllTextAsync(_filePath,
            JsonSerializer.Serialize(Array.Empty<string>()));

        string content = await File.ReadAllTextAsync(_filePath);
        content.Should().Be("[]");
    }

    [Fact]
    public async Task Write_ThenRead_RoundTripsAllIds()
    {
        string[] ids = ["warfare-starwars", "warfare-modern", "example-pack"];
        await File.WriteAllTextAsync(_filePath, JsonSerializer.Serialize(ids));

        string json = await File.ReadAllTextAsync(_filePath);
        string[]? loaded = JsonSerializer.Deserialize<string[]>(json);

        loaded.Should().NotBeNull();
        loaded.Should().BeEquivalentTo(ids);
    }

    [Fact]
    public void Format_IsStringArray_NotObject()
    {
        // The format MUST be a plain JSON array, not {"disabled": [...]}
        string[] ids = ["pack-a"];
        string json = JsonSerializer.Serialize(ids);

        json.Should().StartWith("[");
        json.Should().EndWith("]");
    }

    [Fact]
    public async Task MissingFile_ReturnsEmptySet()
    {
        // File doesn't exist
        _filePath.Should().NotBeNullOrEmpty();
        bool exists = File.Exists(_filePath);
        exists.Should().BeFalse();

        // Contract: when file missing, return empty
        HashSet<string> result = new();
        if (File.Exists(_filePath))
        {
            string json = await File.ReadAllTextAsync(_filePath);
            result = new HashSet<string>(
                JsonSerializer.Deserialize<string[]>(json) ?? []);
        }
        result.Should().BeEmpty();
    }

    [Fact]
    public async Task Write_OverwriteExisting_ReplacesContent()
    {
        await File.WriteAllTextAsync(_filePath, JsonSerializer.Serialize(new[] { "old-pack" }));
        await File.WriteAllTextAsync(_filePath, JsonSerializer.Serialize(new[] { "new-pack" }));

        string json = await File.ReadAllTextAsync(_filePath);
        string[]? loaded = JsonSerializer.Deserialize<string[]>(json);
        loaded.Should().BeEquivalentTo(new[] { "new-pack" });
    }
}
