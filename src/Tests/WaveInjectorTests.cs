using DINOForge.SDK.Models;
using FluentAssertions;
using System.Collections.Generic;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for WaveSpawnRequest and WaveDefinition model classes.
    /// Tests validate the data structures that define wave spawning behavior.
    /// SystemBase integration tests are covered by ModPlatform integration tests.
    /// </summary>
    public class WaveInjectorTests
    {
        [Fact]
        public void WaveSpawnRequest_DefaultValues_AreCorrect()
        {
            // Arrange & Act
            var request = new global::DINOForge.Runtime.Bridge.WaveSpawnRequest();

            // Assert
            request.WaveDefinitionId.Should().Be("");
            request.StartDelaySeconds.Should().Be(0f);
            request.UseEnemyFaction.Should().Be(true);
            request.SpawnX.Should().Be(0f);
            request.SpawnZ.Should().Be(0f);
        }

        [Fact]
        public void WaveSpawnRequest_Properties_SetCorrectly()
        {
            // Arrange
            var request = new global::DINOForge.Runtime.Bridge.WaveSpawnRequest();

            // Act
            request.WaveDefinitionId = "test-wave-1";
            request.StartDelaySeconds = 5.5f;
            request.UseEnemyFaction = false;
            request.SpawnX = 10.5f;
            request.SpawnZ = 20.3f;

            // Assert
            request.WaveDefinitionId.Should().Be("test-wave-1");
            request.StartDelaySeconds.Should().Be(5.5f);
            request.UseEnemyFaction.Should().Be(false);
            request.SpawnX.Should().Be(10.5f);
            request.SpawnZ.Should().Be(20.3f);
        }

        [Fact]
        public void WaveDefinition_CanBeCreatedWithSpawnGroups()
        {
            // Arrange
            var spawnGroups = new List<SpawnGroup>
            {
                new SpawnGroup { UnitId = "unit-1", Count = 2, SpawnDelay = 0.5f },
                new SpawnGroup { UnitId = "unit-2", Count = 3, SpawnDelay = 0.5f }
            };

            var wave = new WaveDefinition
            {
                Id = "test-wave",
                DisplayName = "Test Wave",
                WaveNumber = 1,
                SpawnGroups = spawnGroups
            };

            // Act & Assert
            wave.Id.Should().Be("test-wave");
            wave.DisplayName.Should().Be("Test Wave");
            wave.WaveNumber.Should().Be(1);
            wave.SpawnGroups.Should().HaveCount(2);
            wave.SpawnGroups[0].UnitId.Should().Be("unit-1");
            wave.SpawnGroups[0].Count.Should().Be(2);
            wave.SpawnGroups[1].UnitId.Should().Be("unit-2");
            wave.SpawnGroups[1].Count.Should().Be(3);
        }

        [Fact]
        public void WaveDefinition_WithEmptySpawnGroups()
        {
            // Arrange
            var wave = new WaveDefinition
            {
                Id = "test-wave",
                DisplayName = "Test Wave",
                WaveNumber = 1,
                SpawnGroups = new List<SpawnGroup>()
            };

            // Act & Assert
            wave.SpawnGroups.Should().HaveCount(0);
        }

        [Fact]
        public void WaveSpawnRequest_CanBeUsedAsProperty()
        {
            // Arrange
            var request = new global::DINOForge.Runtime.Bridge.WaveSpawnRequest
            {
                WaveDefinitionId = "prop-test",
                StartDelaySeconds = 1.5f
            };

            // Act
            var storedRequest = request;

            // Assert
            storedRequest.WaveDefinitionId.Should().Be("prop-test");
            storedRequest.StartDelaySeconds.Should().Be(1.5f);
        }
    }
}
