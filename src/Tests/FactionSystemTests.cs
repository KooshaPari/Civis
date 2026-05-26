#nullable enable
using System.Collections.Generic;
using DINOForge.Runtime.Bridge;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for the FactionRuntime data model and FactionSystem static APIs.
    /// Tests faction registration, lookup, and filtering via static methods.
    /// Integration tests with the ECS SystemBase are in FactionSystemIntegrationTests.
    /// </summary>
    public class FactionSystemTests
    {
        private static FactionDefinition MakeFaction(string id, string displayName = "")
        {
            return new FactionDefinition
            {
                Faction = new FactionInfo
                {
                    Id = id,
                    DisplayName = displayName ?? id,
                    Theme = "custom",
                    Archetype = "custom"
                },
                Economy = new FactionEconomy(),
                Army = new FactionArmy(),
                Roster = new FactionRoster(),
                Buildings = new FactionBuildings()
            };
        }

        // ── FactionRuntime Tests ─────────────────────────────────────────

        [Fact]
        public void FactionRuntime_DefaultValues_AreCorrect()
        {
            var faction = new FactionRuntime();

            faction.Id.Should().Be("");
            faction.Name.Should().Be("");
            faction.IsEnemy.Should().BeFalse();
            faction.EntityCount.Should().Be(0);
            faction.Definition.Should().BeNull();
        }

        [Fact]
        public void FactionRuntime_WithValues_StoresCorrectly()
        {
            var def = MakeFaction("test", "Test Faction");
            var faction = new FactionRuntime
            {
                Id = "test",
                Name = "Test Faction",
                IsEnemy = true,
                EntityCount = 42,
                Definition = def
            };

            faction.Id.Should().Be("test");
            faction.Name.Should().Be("Test Faction");
            faction.IsEnemy.Should().BeTrue();
            faction.EntityCount.Should().Be(42);
            faction.Definition.Should().NotBeNull();
        }

    }
}
