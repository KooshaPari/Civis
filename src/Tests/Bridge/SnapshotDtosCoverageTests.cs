#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Coverage for <see cref="ResourceSnapshot"/> (8 resource fields) and
/// <see cref="CatalogSnapshot"/> (+ nested <see cref="CatalogEntry"/>) — defaults
/// and JSON round-trip across the bridge protocol boundary.
/// </summary>
public class SnapshotDtosCoverageTests
{
    [Fact]
    public void ResourceSnapshot_Defaults_AllZero()
    {
        ResourceSnapshot r = new();

        r.Food.Should().Be(0);
        r.Wood.Should().Be(0);
        r.Stone.Should().Be(0);
        r.Iron.Should().Be(0);
        r.Money.Should().Be(0);
        r.Souls.Should().Be(0);
        r.Bones.Should().Be(0);
        r.Spirit.Should().Be(0);
    }

    [Fact]
    public void ResourceSnapshot_RoundTripsThroughJson()
    {
        ResourceSnapshot original = new()
        {
            Food = 100, Wood = 200, Stone = 50, Iron = 25,
            Money = 999, Souls = 7, Bones = 13, Spirit = 3
        };

        string json = JsonConvert.SerializeObject(original);
        ResourceSnapshot? back = JsonConvert.DeserializeObject<ResourceSnapshot>(json);

        back.Should().NotBeNull();
        back!.Food.Should().Be(100);
        back.Wood.Should().Be(200);
        back.Stone.Should().Be(50);
        back.Iron.Should().Be(25);
        back.Money.Should().Be(999);
        back.Souls.Should().Be(7);
        back.Bones.Should().Be(13);
        back.Spirit.Should().Be(3);
    }

    [Fact]
    public void CatalogSnapshot_Defaults_AllListsEmpty()
    {
        CatalogSnapshot c = new();

        c.Units.Should().NotBeNull().And.BeEmpty();
        c.Buildings.Should().NotBeNull().And.BeEmpty();
        c.Projectiles.Should().NotBeNull().And.BeEmpty();
        c.Other.Should().NotBeNull().And.BeEmpty();
    }

    [Fact]
    public void CatalogSnapshot_WithEntries_RoundTripsThroughJson()
    {
        CatalogSnapshot original = new();
        original.Units.Add(new CatalogEntry
        {
            InferredId = "unit:trooper",
            ComponentCount = 12,
            EntityCount = 40,
            Category = "infantry"
        });
        original.Buildings.Add(new CatalogEntry { InferredId = "building:barracks" });

        string json = JsonConvert.SerializeObject(original);
        CatalogSnapshot? back = JsonConvert.DeserializeObject<CatalogSnapshot>(json);

        back.Should().NotBeNull();
        back!.Units.Should().ContainSingle();
        back.Units[0].InferredId.Should().Be("unit:trooper");
        back.Units[0].ComponentCount.Should().Be(12);
        back.Units[0].EntityCount.Should().Be(40);
        back.Units[0].Category.Should().Be("infantry");
        back.Buildings.Should().ContainSingle().Which.InferredId.Should().Be("building:barracks");
        back.Projectiles.Should().BeEmpty();
    }
}
