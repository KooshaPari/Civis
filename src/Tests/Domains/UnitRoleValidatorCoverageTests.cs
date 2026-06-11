#nullable enable
using System;
using DINOForge.Domains.Warfare.Roles;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="UnitRoleValidator.ValidateRoster"/> — the required-role
/// fill check across <see cref="FactionRoster"/> slots and the unit-registry existence
/// check, plus the argument-null guards. Pure Warfare-domain logic, no ECS.
/// </summary>
public class UnitRoleValidatorCoverageTests
{
    private static Registry<UnitDefinition> RegistryWith(params string[] unitIds)
    {
        Registry<UnitDefinition> reg = new();
        foreach (string id in unitIds)
            reg.Register(id, new UnitDefinition { Id = id }, RegistrySource.BaseGame, "test-pack");
        return reg;
    }

    private static FactionRoster FullRoster() => new()
    {
        CheapInfantry = "u_cheap",
        LineInfantry = "u_line",
        EliteInfantry = "u_elite",
        AntiArmor = "u_aa",
        SupportWeapon = "u_support",
        Recon = "u_recon",
        LightVehicle = "u_light",
        HeavyVehicle = "u_heavy",
        Artillery = "u_arty",
        HeroCommander = "u_hero",
        SpikeUnit = "u_spike"
    };

    private static readonly string[] AllUnitIds =
    {
        "u_cheap", "u_line", "u_elite", "u_aa", "u_support", "u_recon",
        "u_light", "u_heavy", "u_arty", "u_hero", "u_spike"
    };

    [Fact]
    public void ValidateRoster_AllRolesFilledAndRegistered_IsComplete()
    {
        FactionDefinition faction = new() { Roster = FullRoster() };

        RosterValidationResult result = new UnitRoleValidator()
            .ValidateRoster(faction, RegistryWith(AllUnitIds));

        result.IsComplete.Should().BeTrue();
        result.MissingRoles.Should().BeEmpty();
        result.FilledRoles.Should().HaveCount(UnitRoleValidator.RequiredRoles.Count);
        result.RoleToUnitMap["cheap_infantry"].Should().Be("u_cheap");
    }

    [Fact]
    public void ValidateRoster_EmptyRoster_AllRolesMissing()
    {
        FactionDefinition faction = new() { Roster = new FactionRoster() };

        RosterValidationResult result = new UnitRoleValidator()
            .ValidateRoster(faction, RegistryWith(AllUnitIds));

        result.IsComplete.Should().BeFalse();
        result.MissingRoles.Should().HaveCount(UnitRoleValidator.RequiredRoles.Count);
        result.FilledRoles.Should().BeEmpty();
    }

    [Fact]
    public void ValidateRoster_UnitReferencedButNotInRegistry_RoleIsMissing()
    {
        FactionDefinition faction = new() { Roster = FullRoster() };

        // Registry missing "u_hero" → hero_commander should be reported missing.
        Registry<UnitDefinition> reg = RegistryWith(
            "u_cheap", "u_line", "u_elite", "u_aa", "u_support", "u_recon",
            "u_light", "u_heavy", "u_arty", "u_spike");

        RosterValidationResult result = new UnitRoleValidator().ValidateRoster(faction, reg);

        result.IsComplete.Should().BeFalse();
        result.MissingRoles.Should().Contain("hero_commander");
        result.FilledRoles.Should().NotContain("hero_commander");
    }

    [Fact]
    public void ValidateRoster_WhitespaceUnitId_RoleIsMissing()
    {
        FactionRoster roster = FullRoster();
        roster.Artillery = "   "; // whitespace → IsNullOrWhiteSpace → missing
        FactionDefinition faction = new() { Roster = roster };

        RosterValidationResult result = new UnitRoleValidator()
            .ValidateRoster(faction, RegistryWith(AllUnitIds));

        result.MissingRoles.Should().Contain("artillery");
    }

    [Fact]
    public void ValidateRoster_NullFaction_Throws()
    {
        Action act = () => new UnitRoleValidator().ValidateRoster(null!, RegistryWith());

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ValidateRoster_NullRegistry_Throws()
    {
        FactionDefinition faction = new() { Roster = FullRoster() };

        Action act = () => new UnitRoleValidator().ValidateRoster(faction, null!);

        act.Should().Throw<ArgumentNullException>();
    }
}
