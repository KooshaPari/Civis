#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

public class GameStatusCoverageTests
{
    [Fact]
    public void CtorDefaults_InitializesExpectedValues()
    {
        // Arrange & Act
        GameStatus status = new GameStatus();

        // Assert
        status.Running.Should().BeFalse();
        status.WorldReady.Should().BeFalse();
        status.WorldName.Should().BeEmpty();
        status.EntityCount.Should().Be(0);
        status.ModPlatformReady.Should().BeFalse();
        status.LoadedPacks.Should().NotBeNull();
        status.LoadedPacks.Should().BeEmpty();
        status.Version.Should().BeEmpty();
    }

    [Fact]
    public void Properties_CanBeSetAndReadBack()
    {
        // Arrange
        List<string> loadedPacks = new List<string>
        {
            "core",
            "expansion"
        };

        GameStatus status = new GameStatus
        {
            Running = true,
            WorldReady = true,
            WorldName = "Battlefield-01",
            EntityCount = 128,
            ModPlatformReady = true,
            LoadedPacks = loadedPacks,
            Version = "1.2.3"
        };

        // Assert
        status.Running.Should().BeTrue();
        status.WorldReady.Should().BeTrue();
        status.WorldName.Should().Be("Battlefield-01");
        status.EntityCount.Should().Be(128);
        status.ModPlatformReady.Should().BeTrue();
        status.LoadedPacks.Should().BeSameAs(loadedPacks);
        status.LoadedPacks.Should().Equal("core", "expansion");
        status.Version.Should().Be("1.2.3");
    }

    [Fact]
    public void JsonSerialization_RoundTripsAllProperties()
    {
        // Arrange
        GameStatus original = new GameStatus
        {
            Running = true,
            WorldReady = false,
            WorldName = "Overworld",
            EntityCount = 42,
            ModPlatformReady = true,
            LoadedPacks = new List<string> { "base", "winter" },
            Version = "2026.06"
        };

        // Act
        string json = JsonConvert.SerializeObject(original);
        GameStatus? deserialized = JsonConvert.DeserializeObject<GameStatus>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.Running.Should().BeTrue();
        deserialized.WorldReady.Should().BeFalse();
        deserialized.WorldName.Should().Be("Overworld");
        deserialized.EntityCount.Should().Be(42);
        deserialized.ModPlatformReady.Should().BeTrue();
        deserialized.LoadedPacks.Should().Equal("base", "winter");
        deserialized.Version.Should().Be("2026.06");
    }
}
