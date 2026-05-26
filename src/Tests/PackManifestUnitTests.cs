using System.Collections.Generic;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class PackManifestUnitTests
    {
        [Fact]
        public void DefaultConstructor_SetsExpectedDefaults()
        {
            // Arrange & Act
            var manifest = new PackManifest();

            // Assert
            manifest.Id.Should().Be("");
            manifest.Name.Should().Be("");
            manifest.Version.Should().Be("0.1.0");
            manifest.FrameworkVersion.Should().Be(">=0.1.0 <1.0.0");
            manifest.Author.Should().Be("");
            manifest.Type.Should().Be("content");
            manifest.Description.Should().BeNull();
            manifest.DependsOn.Should().BeEmpty();
            manifest.ConflictsWith.Should().BeEmpty();
            manifest.LoadOrder.Should().Be(100);
            manifest.GameVersion.Should().Be(">=0.0.0 <2.0.0");
            manifest.BepInExVersion.Should().Be(">=5.4.0 <6.0.0");
            manifest.UnityVersion.Should().Be(">=2021.3.0 <2022.0.0");
            manifest.Loads.Should().BeNull();
            manifest.Overrides.Should().BeNull();
        }

        [Fact]
        public void DependsOn_DefaultsToEmptyList()
        {
            // Arrange & Act
            var manifest = new PackManifest();

            // Assert
            manifest.DependsOn.Should().NotBeNull();
            manifest.DependsOn.Should().BeOfType<List<string>>();
            manifest.DependsOn.Should().BeEmpty();
        }

        [Fact]
        public void ConflictsWith_DefaultsToEmptyList()
        {
            // Arrange & Act
            var manifest = new PackManifest();

            // Assert
            manifest.ConflictsWith.Should().NotBeNull();
            manifest.ConflictsWith.Should().BeOfType<List<string>>();
            manifest.ConflictsWith.Should().BeEmpty();
        }

        [Fact]
        public void PropertiesCanBeSet_AndRetrieved()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "1.0.0",
                Author = "Test Author",
                Type = "balance",
                Description = "A test pack",
                FrameworkVersion = ">=0.20.0 <1.0.0",
                LoadOrder = 200
            };

            // Act & Assert
            manifest.Id.Should().Be("test-pack");
            manifest.Name.Should().Be("Test Pack");
            manifest.Version.Should().Be("1.0.0");
            manifest.Author.Should().Be("Test Author");
            manifest.Type.Should().Be("balance");
            manifest.Description.Should().Be("A test pack");
            manifest.FrameworkVersion.Should().Be(">=0.20.0 <1.0.0");
            manifest.LoadOrder.Should().Be(200);
        }

        [Fact]
        public void DependsOn_CanBePopulated_AndMutated()
        {
            // Arrange
            var manifest = new PackManifest { Id = "pack-a" };
            manifest.DependsOn.Add("pack-b");
            manifest.DependsOn.Add("pack-c");

            // Act & Assert
            manifest.DependsOn.Should().HaveCount(2);
            manifest.DependsOn.Should().Contain(new[] { "pack-b", "pack-c" });
        }

        [Fact]
        public void ConflictsWith_CanBePopulated_AndMutated()
        {
            // Arrange
            var manifest = new PackManifest { Id = "pack-a" };
            manifest.ConflictsWith.Add("pack-x");
            manifest.ConflictsWith.Add("pack-y");

            // Act & Assert
            manifest.ConflictsWith.Should().HaveCount(2);
            manifest.ConflictsWith.Should().Contain(new[] { "pack-x", "pack-y" });
        }

        [Fact]
        public void Type_EnumMigration_AcceptsStringValues()
        {
            // Arrange & Act
            var manifest = new PackManifest { Type = "total_conversion" };

            // Assert - Pattern #101: stringly-typed enums should accept valid values
            manifest.Type.Should().Be("total_conversion");
        }
    }

    public class PackLoadsUnitTests
    {
        [Fact]
        public void DefaultConstructor_AllPropertiesNull()
        {
            // Arrange & Act
            var loads = new PackLoads();

            // Assert
            loads.Factions.Should().BeNull();
            loads.Units.Should().BeNull();
            loads.Buildings.Should().BeNull();
            loads.Weapons.Should().BeNull();
            loads.Doctrines.Should().BeNull();
            loads.Audio.Should().BeNull();
            loads.Visuals.Should().BeNull();
            loads.Localization.Should().BeNull();
            loads.WaveTemplates.Should().BeNull();
            loads.TechNodes.Should().BeNull();
            loads.Scenarios.Should().BeNull();
            loads.FactionPatches.Should().BeNull();
        }

        [Fact]
        public void PropertiesCanBeSet_AndRetrieved()
        {
            // Arrange
            var loads = new PackLoads
            {
                Units = new List<string> { "units/unit-a.yaml" },
                Buildings = new List<string> { "buildings/building-a.yaml" },
                Doctrines = new List<string> { "doctrines/doctrine-a.yaml" }
            };

            // Act & Assert
            loads.Units.Should().NotBeNull();
            loads.Units.Should().HaveCount(1);
            loads.Buildings.Should().NotBeNull();
            loads.Buildings.Should().HaveCount(1);
            loads.Doctrines.Should().NotBeNull();
            loads.Doctrines.Should().HaveCount(1);
        }
    }

    public class PackOverridesUnitTests
    {
        [Fact]
        public void DefaultConstructor_AllPropertiesNull()
        {
            // Arrange & Act
            var overrides = new PackOverrides();

            // Assert
            overrides.Units.Should().BeNull();
            overrides.Buildings.Should().BeNull();
            overrides.Stats.Should().BeNull();
        }

        [Fact]
        public void PropertiesCanBeSet_AndRetrieved()
        {
            // Arrange
            var overrides = new PackOverrides
            {
                Units = new List<string> { "overrides/units/unit-override.yaml" },
                Stats = new List<string> { "overrides/stats/stat-override.yaml" }
            };

            // Act & Assert
            overrides.Units.Should().NotBeNull();
            overrides.Units.Should().HaveCount(1);
            overrides.Stats.Should().NotBeNull();
            overrides.Stats.Should().HaveCount(1);
        }
    }
}
