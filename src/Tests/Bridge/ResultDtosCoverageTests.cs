#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Coverage tests for the Bridge.Protocol result DTOs: <see cref="QueryResult"/>,
/// <see cref="StatResult"/>, <see cref="OverrideResult"/> — constructor defaults
/// and JSON round-trip (catches serialization/property drift).
/// </summary>
public class ResultDtosCoverageTests
{
    [Fact]
    public void QueryResult_Defaults_AreEmpty()
    {
        QueryResult result = new();

        result.Count.Should().Be(0);
        result.Entities.Should().NotBeNull().And.BeEmpty();
    }

    [Fact]
    public void QueryResult_RoundTripsThroughJson()
    {
        QueryResult original = new()
        {
            Count = 2,
            Entities = new List<EntityInfo>
            {
                new() { Index = 1, Components = new List<string> { "Health", "Armor" } },
                new() { Index = 7, Components = new List<string> { "Position" } }
            }
        };

        string json = JsonConvert.SerializeObject(original);
        QueryResult? back = JsonConvert.DeserializeObject<QueryResult>(json);

        back.Should().NotBeNull();
        back!.Count.Should().Be(2);
        back.Entities.Should().HaveCount(2);
        back.Entities[0].Index.Should().Be(1);
        back.Entities[0].Components.Should().Contain("Armor");
        back.Entities[1].Components.Should().ContainSingle().Which.Should().Be("Position");
    }

    [Fact]
    public void StatResult_Defaults_AreZeroAndEmpty()
    {
        StatResult result = new();

        result.SdkPath.Should().BeEmpty();
        result.Value.Should().Be(0f);
        result.EntityCount.Should().Be(0);
        result.Values.Should().NotBeNull().And.BeEmpty();
        result.ComponentType.Should().BeEmpty();
        result.FieldName.Should().BeEmpty();
    }

    [Fact]
    public void StatResult_RoundTripsThroughJson()
    {
        StatResult original = new()
        {
            SdkPath = "warfare.unit.health",
            Value = 42.5f,
            EntityCount = 3,
            Values = new List<float> { 1f, 2f, 3f },
            ComponentType = "Health",
            FieldName = "Value"
        };

        string json = JsonConvert.SerializeObject(original);
        StatResult? back = JsonConvert.DeserializeObject<StatResult>(json);

        back.Should().NotBeNull();
        back!.SdkPath.Should().Be("warfare.unit.health");
        back.Value.Should().Be(42.5f);
        back.EntityCount.Should().Be(3);
        back.Values.Should().Equal(1f, 2f, 3f);
        back.ComponentType.Should().Be("Health");
        back.FieldName.Should().Be("Value");
    }

    [Fact]
    public void OverrideResult_Defaults_AreFalseAndEmpty()
    {
        OverrideResult result = new();

        result.Success.Should().BeFalse();
        result.ModifiedCount.Should().Be(0);
        result.SdkPath.Should().BeEmpty();
        result.Message.Should().BeEmpty();
    }

    [Fact]
    public void OverrideResult_RoundTripsThroughJson()
    {
        OverrideResult original = new()
        {
            Success = true,
            ModifiedCount = 5,
            SdkPath = "warfare.unit.armor",
            Message = "applied"
        };

        string json = JsonConvert.SerializeObject(original);
        OverrideResult? back = JsonConvert.DeserializeObject<OverrideResult>(json);

        back.Should().NotBeNull();
        back!.Success.Should().BeTrue();
        back.ModifiedCount.Should().Be(5);
        back.SdkPath.Should().Be("warfare.unit.armor");
        back.Message.Should().Be("applied");
    }
}
