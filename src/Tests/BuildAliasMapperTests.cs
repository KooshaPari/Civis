// Tests for BuildAliasMapper — the pure-C# alias-resolution layer of BuildMenuInjector.
//
// BuildMenuInjector itself (the reflection/Resources execution layer) requires the live
// game and is excluded from CI builds without DINO. BuildAliasMapper has no Unity dependency
// and is fully testable here.

using System.Collections.Generic;
using DINOForge.Runtime.Bridge;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="BuildAliasMapper.ResolveAlias"/>: explicit alias precedence and
    /// auto-mapping by building category / id heuristics onto vanilla DINO BuildingType slots.
    /// </summary>
    public class BuildAliasMapperTests
    {
        private static BuildingDefinition Def(
            string id, string? buildingType = null, string? alias = null, params string[] defenseTags)
        {
            return new BuildingDefinition
            {
                Id = id,
                DisplayName = id,
                BuildingType = buildingType,
                BuildAlias = alias,
                DefenseTags = new List<string>(defenseTags),
            };
        }

        [Fact]
        public void ExplicitBuildAlias_TakesPrecedence()
        {
            BuildingDefinition def = Def("anything", buildingType: "Defense", alias: "TownHall");
            BuildAliasMapper.ResolveAlias(def).Should().Be("TownHall");
        }

        [Fact]
        public void ExplicitBuildAlias_IsTrimmed()
        {
            BuildingDefinition def = Def("x", alias: "  Stables  ");
            BuildAliasMapper.ResolveAlias(def).Should().Be("Stables");
        }

        [Theory]
        [InlineData("rep_gunship_bay", "Production")]
        [InlineData("cis_droid_hangar", "Production")]
        [InlineData("aviary", "Production")]
        [InlineData("wyvern_roost", null)]
        [InlineData("phoenix_nest", null)]
        [InlineData("dragon_cave", null)]
        [InlineData("airfield_alpha", null)]
        public void ProductionBuildings_MapToStables(string id, string? type)
        {
            BuildAliasMapper.ResolveAlias(Def(id, buildingType: type))
                .Should().Be(BuildAliasMapper.DefaultProductionAlias);
        }

        [Theory]
        [InlineData("ballista_tower", "Defense")]
        [InlineData("thunder_cannon", null)]
        [InlineData("flak_turret", null)]
        public void DefenseBuildings_MapToTower(string id, string? type)
        {
            BuildAliasMapper.ResolveAlias(Def(id, buildingType: type))
                .Should().Be(BuildAliasMapper.DefaultDefenseAlias);
        }

        [Fact]
        public void AntiAirDefenseTag_MapsToTower()
        {
            BuildingDefinition def = Def("generic_emplacement", defenseTags: "AntiAir");
            BuildAliasMapper.ResolveAlias(def).Should().Be(BuildAliasMapper.DefaultDefenseAlias);
        }

        [Theory]
        [InlineData("naval_shipyard", "Naval")]
        [InlineData("trade_port", null)]
        [InlineData("war_harbor", null)]
        public void NavalBuildings_MapToPort(string id, string? type)
        {
            BuildAliasMapper.ResolveAlias(Def(id, buildingType: type))
                .Should().Be(BuildAliasMapper.DefaultNavalAlias);
        }

        [Fact]
        public void UnknownBuilding_FallsBackToStables()
        {
            BuildingDefinition def = Def("mysterious_thing", buildingType: "Wibble");
            BuildAliasMapper.ResolveAlias(def).Should().Be(BuildAliasMapper.FallbackAlias);
        }

        [Fact]
        public void NavalTakesPriorityOverProductionCategory()
        {
            // A production-category building whose id clearly reads naval should still go to Port.
            BuildingDefinition def = Def("ship_production_dock", buildingType: "Production");
            BuildAliasMapper.ResolveAlias(def).Should().Be(BuildAliasMapper.DefaultNavalAlias);
        }
    }
}
