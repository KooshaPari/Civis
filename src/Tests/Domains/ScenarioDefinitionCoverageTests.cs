#nullable enable
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="ScenarioDefinition.Validate"/> — required Id/DisplayName,
/// WaveCount &gt; 0, MaxDuration &gt;= 0, and the indexed AllowedFactions blank-entry check.
/// First Scenario-domain coverage.
/// </summary>
public class ScenarioDefinitionCoverageTests
{
    private static ScenarioDefinition Valid() => new()
    {
        Id = "scn:tutorial",
        DisplayName = "Tutorial",
        WaveCount = 5,
        MaxDuration = 0
    };

    [Fact]
    public void Validate_PopulatedScenario_IsValid()
    {
        Valid().Validate().IsValid.Should().BeTrue();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Validate_MissingId_Fails(string id)
    {
        ScenarioDefinition s = Valid();
        s.Id = id;

        s.Validate().Errors.Should().Contain(e => e.Path == "id");
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Validate_MissingDisplayName_Fails(string name)
    {
        ScenarioDefinition s = Valid();
        s.DisplayName = name;

        s.Validate().Errors.Should().Contain(e => e.Path == "display_name");
    }

    [Theory]
    [InlineData(0)]
    [InlineData(-3)]
    public void Validate_NonPositiveWaveCount_Fails(int waveCount)
    {
        ScenarioDefinition s = Valid();
        s.WaveCount = waveCount;

        ValidationResult result = s.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "wave_count");
    }

    [Fact]
    public void Validate_NegativeMaxDuration_Fails()
    {
        ScenarioDefinition s = Valid();
        s.MaxDuration = -1;

        s.Validate().Errors.Should().Contain(e => e.Path == "max_duration");
    }

    [Fact]
    public void Validate_BlankAllowedFactionEntry_FailsWithIndexedPath()
    {
        ScenarioDefinition s = Valid();
        s.AllowedFactions = new List<string> { "rebels", "   ", "empire" };

        ValidationResult result = s.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "allowed_factions[1]");
    }

    [Fact]
    public void Validate_ValidAllowedFactions_AreAccepted()
    {
        ScenarioDefinition s = Valid();
        s.AllowedFactions = new List<string> { "rebels", "empire" };

        s.Validate().IsValid.Should().BeTrue();
    }

    [Fact]
    public void Validate_MultipleViolations_ReportsAll()
    {
        ScenarioDefinition s = new() { Id = "", DisplayName = "", WaveCount = 0, MaxDuration = -1 };

        s.Validate().Errors.Should().HaveCount(4);
    }
}
