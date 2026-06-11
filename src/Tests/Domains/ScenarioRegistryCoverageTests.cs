#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="ScenarioRegistry"/> — Register (guards + overwrite), Get/TryGet/Contains
/// hit+miss, Unregister, and GetScenariosByDifficulty filtering.
/// </summary>
public class ScenarioRegistryCoverageTests
{
    private static ScenarioDefinition Scenario(string id, Difficulty difficulty = Difficulty.Normal) => new()
    {
        Id = id,
        DisplayName = id,
        WaveCount = 3,
        Difficulty = difficulty
    };

    [Fact]
    public void Register_ThenGet_ReturnsScenario()
    {
        ScenarioRegistry reg = new();
        ScenarioDefinition scn = Scenario("scn:a");

        reg.Register(scn);

        reg.GetScenario("scn:a").Should().BeSameAs(scn);
        reg.Contains("scn:a").Should().BeTrue();
        reg.All.Should().ContainSingle();
    }

    [Fact]
    public void GetScenario_UnknownId_Throws()
    {
        Action act = () => new ScenarioRegistry().GetScenario("missing");

        act.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void TryGetScenario_Miss_ReturnsFalseAndNull()
    {
        bool found = new ScenarioRegistry().TryGetScenario("nope", out ScenarioDefinition? scn);

        found.Should().BeFalse();
        scn.Should().BeNull();
    }

    [Fact]
    public void Register_Null_Throws()
    {
        Action act = () => new ScenarioRegistry().Register(null!);

        act.Should().Throw<ArgumentNullException>();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Register_EmptyId_Throws(string id)
    {
        Action act = () => new ScenarioRegistry().Register(Scenario(id));

        act.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void Register_SameId_Overwrites()
    {
        ScenarioRegistry reg = new();
        reg.Register(new ScenarioDefinition { Id = "dup", DisplayName = "First", WaveCount = 1 });
        reg.Register(new ScenarioDefinition { Id = "dup", DisplayName = "Second", WaveCount = 1 });

        reg.GetScenario("dup").DisplayName.Should().Be("Second");
        reg.All.Should().ContainSingle();
    }

    [Fact]
    public void Unregister_RemovesScenario()
    {
        ScenarioRegistry reg = new();
        reg.Register(Scenario("scn:x"));

        reg.Unregister("scn:x").Should().BeTrue();
        reg.Contains("scn:x").Should().BeFalse();
        reg.Unregister("scn:x").Should().BeFalse(); // already gone
    }

    [Fact]
    public void GetScenariosByDifficulty_FiltersCorrectly()
    {
        ScenarioRegistry reg = new();
        reg.Register(Scenario("easy1", Difficulty.Easy));
        reg.Register(Scenario("hard1", Difficulty.Hard));
        reg.Register(Scenario("hard2", Difficulty.Hard));

        reg.GetScenariosByDifficulty(Difficulty.Hard).Should().HaveCount(2);
        reg.GetScenariosByDifficulty(Difficulty.Easy).Should().ContainSingle();
        reg.GetScenariosByDifficulty(Difficulty.Nightmare).Should().BeEmpty();
    }
}
