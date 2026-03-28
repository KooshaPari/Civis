#nullable enable
using System.Collections.Generic;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public class FactionPatchDefinitionTests
{
    [Fact]
    public void FactionPatchDefinition_DefaultConstruction_InitializesCollections()
    {
        FactionPatchDefinition patch = new();

        patch.TargetFaction.Should().BeEmpty();
        patch.Add.Should().NotBeNull();
        patch.Add.Units.Should().BeEmpty();
        patch.Add.Buildings.Should().BeEmpty();
        patch.Add.Doctrines.Should().BeEmpty();
        patch.RoleOverrides.Should().BeEmpty();
    }

    [Fact]
    public void FactionPatchDefinition_CanPopulateAdditionsAndOverrides()
    {
        FactionPatchDefinition patch = new()
        {
            TargetFaction = "player",
            Add = new FactionPatchAdditions
            {
                Units = new List<string> { "unit_a", "unit_b" },
                Buildings = new List<string> { "barracks" },
                Doctrines = new List<string> { "order_doctrine" }
            },
            RoleOverrides = new Dictionary<string, string>
            {
                ["unit_a"] = "line_infantry"
            }
        };

        patch.TargetFaction.Should().Be("player");
        patch.Add.Units.Should().ContainInOrder("unit_a", "unit_b");
        patch.Add.Buildings.Should().ContainSingle().Which.Should().Be("barracks");
        patch.Add.Doctrines.Should().ContainSingle().Which.Should().Be("order_doctrine");
        patch.RoleOverrides["unit_a"].Should().Be("line_infantry");
    }
}
