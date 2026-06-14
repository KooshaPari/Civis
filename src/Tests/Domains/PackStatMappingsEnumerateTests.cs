#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Runtime.Bridge;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Behavior coverage for <see cref="PackStatMappings.EnumerateStatPaths"/> — maps a
/// <see cref="UnitStats"/> to the SDK stat paths the injector applies. Guards: null throws,
/// only positive stats are emitted, the FireRate→"attack_cooldown" rename, and that
/// non-injected stats (Damage/Accuracy/Morale) are never emitted.
/// </summary>
public class PackStatMappingsEnumerateTests
{
    private static Dictionary<string, float> Paths(UnitStats stats) =>
        PackStatMappings.EnumerateStatPaths(stats).ToDictionary(p => p.SdkPath, p => p.Value);

    [Fact]
    public void EnumerateStatPaths_Null_Throws()
    {
        Action act = () => PackStatMappings.EnumerateStatPaths(null!).ToList();

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EnumerateStatPaths_AllPositive_EmitsFiveMappedPaths()
    {
        UnitStats stats = new() { Hp = 200f, Armor = 6f, Speed = 12f, FireRate = 1.2f, Range = 14f };

        Dictionary<string, float> paths = Paths(stats);

        paths.Should().HaveCount(5);
        paths["unit.stats.hp"].Should().Be(200f);
        paths["unit.stats.armor"].Should().Be(6f);
        paths["unit.stats.speed"].Should().Be(12f);
        paths["unit.stats.range"].Should().Be(14f);
        // FireRate maps to the attack_cooldown SDK path (non-obvious rename — guard it).
        paths["unit.stats.attack_cooldown"].Should().Be(1.2f);
    }

    [Fact]
    public void EnumerateStatPaths_ZeroStats_AreOmitted()
    {
        // Armor/Speed/Range default to 0 → must NOT be emitted. Hp(1) + FireRate(1.0) default > 0.
        UnitStats stats = new() { Hp = 50f, Armor = 0f, Speed = 0f, Range = 0f, FireRate = 0f };

        Dictionary<string, float> paths = Paths(stats);

        paths.Should().ContainKey("unit.stats.hp");
        paths.Should().NotContainKey("unit.stats.armor");
        paths.Should().NotContainKey("unit.stats.speed");
        paths.Should().NotContainKey("unit.stats.range");
        paths.Should().NotContainKey("unit.stats.attack_cooldown");
    }

    [Fact]
    public void EnumerateStatPaths_NonInjectedStats_NeverEmitted()
    {
        // Damage, Accuracy, Morale are real UnitStats props but are NOT part of the injection set.
        UnitStats stats = new() { Hp = 100f, Damage = 50f, Accuracy = 0.9f, Morale = 120f };

        IEnumerable<string> keys = Paths(stats).Keys;

        keys.Should().NotContain(k => k.Contains("damage") || k.Contains("accuracy") || k.Contains("morale"));
    }
}
