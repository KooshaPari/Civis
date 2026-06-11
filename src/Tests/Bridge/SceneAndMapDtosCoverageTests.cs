#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Coverage for <see cref="LoadSceneResult"/>, <see cref="ScreenshotResult"/>, and
/// <see cref="ComponentMapResult"/> (+ nested <see cref="ComponentMapEntry"/>) — defaults
/// and JSON round-trip.
/// </summary>
public class SceneAndMapDtosCoverageTests
{
    [Fact]
    public void LoadSceneResult_Defaults_AreFalseEmptyZero()
    {
        LoadSceneResult r = new();

        r.Success.Should().BeFalse();
        r.Scene.Should().BeEmpty();
        r.SceneCount.Should().Be(-1); // sentinel default for "unknown scene count"
        r.BuildIndex.Should().Be(-1); // sentinel default for "no build index"
    }

    [Fact]
    public void LoadSceneResult_RoundTripsThroughJson()
    {
        LoadSceneResult original = new()
        {
            Success = true, Scene = "MainMenu", SceneCount = 4, BuildIndex = 1
        };

        LoadSceneResult? back = JsonConvert.DeserializeObject<LoadSceneResult>(JsonConvert.SerializeObject(original));

        back.Should().NotBeNull();
        back!.Success.Should().BeTrue();
        back.Scene.Should().Be("MainMenu");
        back.SceneCount.Should().Be(4);
        back.BuildIndex.Should().Be(1);
    }

    [Fact]
    public void ScreenshotResult_Defaults_AreEmptyZeroNull()
    {
        ScreenshotResult r = new();

        r.Path.Should().BeEmpty();
        r.Width.Should().Be(0);
        r.Height.Should().Be(0);
        r.Success.Should().BeFalse();
        r.Base64.Should().BeNull();
        r.Timestamp.Should().BeNull();
    }

    [Fact]
    public void ScreenshotResult_RoundTripsThroughJson()
    {
        ScreenshotResult original = new()
        {
            Path = "C:/shots/a.png", Width = 1920, Height = 1080,
            Success = true, Base64 = "AAAA", Timestamp = "2026-06-11T00:00:00Z"
        };

        ScreenshotResult? back = JsonConvert.DeserializeObject<ScreenshotResult>(JsonConvert.SerializeObject(original));

        back.Should().NotBeNull();
        back!.Path.Should().Be("C:/shots/a.png");
        back.Width.Should().Be(1920);
        back.Height.Should().Be(1080);
        back.Success.Should().BeTrue();
        back.Base64.Should().Be("AAAA");
        back.Timestamp.Should().Be("2026-06-11T00:00:00Z");
    }

    [Fact]
    public void ComponentMapResult_Defaults_HasEmptyMappings()
    {
        ComponentMapResult r = new();

        r.Mappings.Should().NotBeNull().And.BeEmpty();
    }

    [Fact]
    public void ComponentMapEntry_Defaults_AreEmptyAndUnresolved()
    {
        ComponentMapEntry e = new();

        e.SdkPath.Should().BeEmpty();
        e.EcsType.Should().BeEmpty();
        e.FieldName.Should().BeEmpty();
        e.Resolved.Should().BeFalse();
    }

    [Fact]
    public void ComponentMapResult_WithEntry_RoundTripsThroughJson()
    {
        ComponentMapResult original = new();
        original.Mappings.Add(new ComponentMapEntry
        {
            SdkPath = "warfare.unit.health",
            EcsType = "Components.Health",
            FieldName = "Value",
            Resolved = true
        });

        ComponentMapResult? back = JsonConvert.DeserializeObject<ComponentMapResult>(JsonConvert.SerializeObject(original));

        back.Should().NotBeNull();
        back!.Mappings.Should().ContainSingle();
        back.Mappings[0].SdkPath.Should().Be("warfare.unit.health");
        back.Mappings[0].EcsType.Should().Be("Components.Health");
        back.Mappings[0].FieldName.Should().Be("Value");
        back.Mappings[0].Resolved.Should().BeTrue();
    }
}
