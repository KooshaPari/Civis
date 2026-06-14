#nullable enable
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Behavior coverage for <see cref="PackStatMappings.TryResolveMapping"/> — the swap-mapping
/// resolution path. Guards the regression class where vehicle/naval vanilla_mappings silently
/// failed to resolve (commit e4b92038): unmapped values must return false, and the vehicle
/// chassis mappings must resolve to their confirmed ECS archetypes.
/// </summary>
public class PackStatMappingsResolveTests
{
    [Theory]
    [InlineData("militia", "Components.MeleeUnit")]
    [InlineData("ranged_infantry", "Components.RangeUnit")]
    [InlineData("cavalry", "Components.CavalryUnit")]
    [InlineData("siege", "Components.SiegeUnit")]
    // Vehicle/naval chassis — the regression e4b92038 fixed (were unmapped → silent swap failure).
    [InlineData("fast_vehicle", "Components.CavalryUnit")]
    [InlineData("light_vehicle", "Components.CavalryUnit")]
    [InlineData("heavy_vehicle", "Components.SiegeUnit")]
    [InlineData("main_battle_vehicle", "Components.SiegeUnit")]
    // #975 full-world conversion archetypes.
    [InlineData("cims", "Components.Citizen")]
    [InlineData("building", "Components.BuildingBase")]
    public void TryResolveMapping_KnownMapping_ResolvesToExpectedComponent(string mapping, string expected)
    {
        bool resolved = PackStatMappings.TryResolveMapping(mapping, out string? componentType);

        resolved.Should().BeTrue($"'{mapping}' is a registered vanilla_mapping");
        componentType.Should().Be(expected);
    }

    [Fact]
    public void TryResolveMapping_IsCaseInsensitive()
    {
        PackStatMappings.TryResolveMapping("FAST_VEHICLE", out string? upper).Should().BeTrue();
        upper.Should().Be("Components.CavalryUnit");

        PackStatMappings.TryResolveMapping("Cavalry", out string? mixed).Should().BeTrue();
        mixed.Should().Be("Components.CavalryUnit");
    }

    [Fact]
    public void TryResolveMapping_AerialFighter_RegisteredButNullComponent()
    {
        // aerial_fighter is intentionally registered with a null component (PackStatInjector skips
        // it; AerialSpawnSystem owns aerial behaviour). true+null is the documented contract —
        // null here does NOT mean "no swap target". Guards against treating it as unrecognized.
        bool resolved = PackStatMappings.TryResolveMapping("aerial_fighter", out string? componentType);

        resolved.Should().BeTrue("aerial_fighter is a registered mapping (with intentional null component)");
        componentType.Should().BeNull();
    }

    [Theory]
    [InlineData("not_a_real_mapping")]
    [InlineData("")]
    [InlineData("   ")]
    [InlineData(null)]
    public void TryResolveMapping_UnknownOrBlank_ReturnsFalseAndNull(string? mapping)
    {
        bool resolved = PackStatMappings.TryResolveMapping(mapping, out string? componentType);

        resolved.Should().BeFalse($"'{mapping ?? "<null>"}' is not a registered vanilla_mapping");
        componentType.Should().BeNull();
    }
}
