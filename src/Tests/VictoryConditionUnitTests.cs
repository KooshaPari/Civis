using FluentAssertions;
using DINOForge.Domains.Scenario.Models;
using Xunit;

namespace DINOForge.Tests
{
    public class VictoryConditionUnitTests
    {
        [Fact]
        public void Constructor_Defaults_InitializesWithExpectedValues()
        {
            var victory = new VictoryCondition();

            victory.ConditionType.Should().Be(VictoryConditionType.SurviveWaves);
            victory.TargetValue.Should().Be(0);
            victory.TargetId.Should().BeNull();
            victory.Description.Should().Be(string.Empty);
        }

        [Fact]
        public void ConditionType_CanBeSetToSurviveWaves()
        {
            var victory = new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves };

            victory.ConditionType.Should().Be(VictoryConditionType.SurviveWaves);
        }

        [Fact]
        public void ConditionType_CanBeSetToDestroyTarget()
        {
            var victory = new VictoryCondition { ConditionType = VictoryConditionType.DestroyTarget };

            victory.ConditionType.Should().Be(VictoryConditionType.DestroyTarget);
        }

        [Fact]
        public void ConditionType_CanBeSetToReachPopulation()
        {
            var victory = new VictoryCondition { ConditionType = VictoryConditionType.ReachPopulation };

            victory.ConditionType.Should().Be(VictoryConditionType.ReachPopulation);
        }

        [Fact]
        public void ConditionType_CanBeSetToAccumulateResource()
        {
            var victory = new VictoryCondition { ConditionType = VictoryConditionType.AccumulateResource };

            victory.ConditionType.Should().Be(VictoryConditionType.AccumulateResource);
        }

        [Fact]
        public void ConditionType_CanBeSetToTimeSurvival()
        {
            var victory = new VictoryCondition { ConditionType = VictoryConditionType.TimeSurvival };

            victory.ConditionType.Should().Be(VictoryConditionType.TimeSurvival);
        }

        [Fact]
        public void ConditionType_CanBeSetToCustom()
        {
            var victory = new VictoryCondition { ConditionType = VictoryConditionType.Custom };

            victory.ConditionType.Should().Be(VictoryConditionType.Custom);
        }

        [Fact]
        public void TargetValue_CanBePositive()
        {
            var victory = new VictoryCondition { TargetValue = 500 };

            victory.TargetValue.Should().Be(500);
        }

        [Fact]
        public void TargetValue_CanBeZero()
        {
            var victory = new VictoryCondition { TargetValue = 0 };

            victory.TargetValue.Should().Be(0);
        }

        [Fact]
        public void TargetValue_CanBeNegative()
        {
            var victory = new VictoryCondition { TargetValue = -10 };

            victory.TargetValue.Should().Be(-10);
        }

        [Fact]
        public void TargetId_CanBeSet()
        {
            var victory = new VictoryCondition { TargetId = "gold" };

            victory.TargetId.Should().Be("gold");
        }

        [Fact]
        public void TargetId_CanBeNull()
        {
            var victory = new VictoryCondition { TargetId = null };

            victory.TargetId.Should().BeNull();
        }

        [Fact]
        public void Description_CanBeSet()
        {
            var victory = new VictoryCondition { Description = "Survive 10 waves" };

            victory.Description.Should().Be("Survive 10 waves");
        }

        [Fact]
        public void Description_DefaultsToEmpty()
        {
            var victory = new VictoryCondition();

            victory.Description.Should().Be(string.Empty);
        }

        [Fact]
        public void CanCreateCompleteVictoryCondition()
        {
            var victory = new VictoryCondition
            {
                ConditionType = VictoryConditionType.ReachPopulation,
                TargetValue = 1000,
                TargetId = "faction-player",
                Description = "Reach 1000 population"
            };

            victory.ConditionType.Should().Be(VictoryConditionType.ReachPopulation);
            victory.TargetValue.Should().Be(1000);
            victory.TargetId.Should().Be("faction-player");
            victory.Description.Should().Be("Reach 1000 population");
        }

        [Fact]
        public void AllConditionTypes_AreAccessible()
        {
            var allTypes = new[]
            {
                VictoryConditionType.SurviveWaves,
                VictoryConditionType.DestroyTarget,
                VictoryConditionType.ReachPopulation,
                VictoryConditionType.AccumulateResource,
                VictoryConditionType.TimeSurvival,
                VictoryConditionType.Custom
            };

            foreach (var type in allTypes)
            {
                var victory = new VictoryCondition { ConditionType = type };
                victory.ConditionType.Should().Be(type);
            }
        }
    }
}
