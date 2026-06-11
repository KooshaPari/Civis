#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.Domains.Warfare.Doctrines;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="DoctrineEngine"/> — multiplier application (<c>ApplyAll</c>),
/// stat clamping, and <c>ValidateDoctrine</c> warning branches. Pure Warfare-domain math.
/// </summary>
public class DoctrineEngineCoverageTests
{
    private static UnitStats BaseStats() => new()
    {
        Hp = 100f, Damage = 20f, Armor = 10f, Range = 5f, Speed = 4f,
        Accuracy = 0.8f, FireRate = 1f, Morale = 50f,
        Cost = new ResourceCost { Food = 50, Gold = 10 }
    };

    [Fact]
    public void ApplyAll_ArchetypeModifiers_MultiplyStats()
    {
        FactionArchetype archetype = new("aggressive", "Aggressive", "", new Dictionary<string, float> { ["hp"] = 1.5f, ["damage"] = 2f });

        UnitStats result = new DoctrineEngine().ApplyAll(BaseStats(), archetype, doctrine: null);

        result.Hp.Should().Be(150f);    // 100 * 1.5
        result.Damage.Should().Be(40f); // 20 * 2
        result.Armor.Should().Be(10f);  // unchanged
    }

    [Fact]
    public void ApplyAll_DoctrineStacksOnTopOfArchetype()
    {
        FactionArchetype archetype = new("a", "A", "", new Dictionary<string, float> { ["hp"] = 2f });
        DoctrineDefinition doctrine = new()
        {
            Id = "d", DisplayName = "D",
            Modifiers = new Dictionary<string, float> { ["hp"] = 1.5f }
        };

        UnitStats result = new DoctrineEngine().ApplyAll(BaseStats(), archetype, doctrine);

        result.Hp.Should().Be(300f); // 100 * 2 (archetype) * 1.5 (doctrine)
    }

    [Fact]
    public void ApplyAll_ClampsHpToMinimumOne()
    {
        FactionArchetype archetype = new("a", "A", "", new Dictionary<string, float> { ["hp"] = 0f });

        UnitStats result = new DoctrineEngine().ApplyAll(BaseStats(), archetype, null);

        result.Hp.Should().Be(1f); // clamped up from 0
    }

    [Fact]
    public void ApplyAll_ClampsAccuracyToOne()
    {
        FactionArchetype archetype = new("a", "A", "", new Dictionary<string, float> { ["accuracy"] = 5f });

        UnitStats result = new DoctrineEngine().ApplyAll(BaseStats(), archetype, null);

        result.Accuracy.Should().Be(1f);
    }

    [Fact]
    public void ApplyAll_UnknownModifier_IsIgnored()
    {
        FactionArchetype archetype = new("a", "A", "", new Dictionary<string, float> { ["nonexistent_stat"] = 99f });

        UnitStats result = new DoctrineEngine().ApplyAll(BaseStats(), archetype, null);

        result.Hp.Should().Be(100f); // untouched
    }

    [Fact]
    public void ApplyAll_NullArgs_Throw()
    {
        FactionArchetype archetype = new("a", "A", "", new Dictionary<string, float>());

        Action nullStats = () => new DoctrineEngine().ApplyAll(null!, archetype, null);
        Action nullArch = () => new DoctrineEngine().ApplyAll(BaseStats(), null!, null);

        nullStats.Should().Throw<ArgumentNullException>();
        nullArch.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ValidateDoctrine_CleanDoctrine_HasNoErrors()
    {
        DoctrineDefinition doctrine = new()
        {
            Id = "d", DisplayName = "D",
            Modifiers = new Dictionary<string, float> { ["hp"] = 1.5f }
        };

        new DoctrineEngine().ValidateDoctrine(doctrine).Should().BeEmpty();
    }

    [Fact]
    public void ValidateDoctrine_NegativeModifier_ReportsError()
    {
        DoctrineDefinition doctrine = new()
        {
            Id = "d", DisplayName = "D",
            Modifiers = new Dictionary<string, float> { ["hp"] = -1f }
        };

        new DoctrineEngine().ValidateDoctrine(doctrine).Should().ContainSingle(e => e.Contains("negative"));
    }

    [Fact]
    public void ValidateDoctrine_ExtremeModifier_ReportsError()
    {
        DoctrineDefinition doctrine = new()
        {
            Id = "d", DisplayName = "D",
            Modifiers = new Dictionary<string, float> { ["damage"] = 50f } // >10
        };

        new DoctrineEngine().ValidateDoctrine(doctrine).Should().Contain(e => e.Contains("extreme"));
    }

    [Fact]
    public void ValidateDoctrine_MissingIdAndName_ReportsBoth()
    {
        DoctrineDefinition doctrine = new()
        {
            Id = "", DisplayName = "",
            Modifiers = new Dictionary<string, float>()
        };

        IReadOnlyList<string> errors = new DoctrineEngine().ValidateDoctrine(doctrine);

        errors.Should().Contain(e => e.Contains("no id"));
        errors.Should().Contain(e => e.Contains("no display_name"));
    }

    [Fact]
    public void ValidateDoctrine_Null_Throws()
    {
        Action act = () => new DoctrineEngine().ValidateDoctrine(null!);

        act.Should().Throw<ArgumentNullException>();
    }
}
