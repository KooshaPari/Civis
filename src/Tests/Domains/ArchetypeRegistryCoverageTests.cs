#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Archetypes;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="ArchetypeRegistry"/> — default registration, Get/TryGet
/// hit+miss paths, custom Register (with overwrite and null guard), and the All snapshot.
/// </summary>
public class ArchetypeRegistryCoverageTests
{
    [Fact]
    public void Ctor_RegistersDefaultArchetypes()
    {
        ArchetypeRegistry reg = new();

        reg.All.Should().NotBeEmpty();
        reg.TryGetArchetype("order", out _).Should().BeTrue();
    }

    [Fact]
    public void GetArchetype_KnownId_ReturnsArchetype()
    {
        ArchetypeRegistry reg = new();

        FactionArchetype order = reg.GetArchetype("order");

        order.Id.Should().Be("order");
    }

    [Fact]
    public void GetArchetype_UnknownId_ThrowsKeyNotFound()
    {
        ArchetypeRegistry reg = new();

        Action act = () => reg.GetArchetype("does_not_exist");

        act.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void TryGetArchetype_UnknownId_ReturnsFalseAndNull()
    {
        ArchetypeRegistry reg = new();

        bool found = reg.TryGetArchetype("nope", out FactionArchetype? archetype);

        found.Should().BeFalse();
        archetype.Should().BeNull();
    }

    [Fact]
    public void Register_CustomArchetype_IsRetrievable()
    {
        ArchetypeRegistry reg = new();
        FactionArchetype custom = new("custom", "Custom", "", new Dictionary<string, float> { ["hp"] = 2f });

        reg.Register(custom);

        reg.GetArchetype("custom").Should().BeSameAs(custom);
    }

    [Fact]
    public void Register_SameId_Overwrites()
    {
        ArchetypeRegistry reg = new();
        FactionArchetype first = new("dup", "First", "", new Dictionary<string, float>());
        FactionArchetype second = new("dup", "Second", "", new Dictionary<string, float>());

        reg.Register(first);
        reg.Register(second);

        reg.GetArchetype("dup").DisplayName.Should().Be("Second");
    }

    [Fact]
    public void Register_Null_Throws()
    {
        ArchetypeRegistry reg = new();

        Action act = () => reg.Register(null!);

        act.Should().Throw<ArgumentNullException>();
    }
}
