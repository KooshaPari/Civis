using FluentAssertions;
using DINOForge.Domains.Scenario.Models;
using Xunit;

namespace DINOForge.Tests
{
    public class ScriptedEventUnitTests
    {
        [Fact]
        public void Constructor_Defaults_InitializesWithExpectedValues()
        {
            var scriptedEvent = new ScriptedEvent();

            scriptedEvent.Id.Should().Be(string.Empty);
            scriptedEvent.TriggerType.Should().Be(TriggerType.OnWave);
            scriptedEvent.TriggerValue.Should().Be(0);
            scriptedEvent.TriggerTarget.Should().BeNull();
            scriptedEvent.Actions.Should().NotBeNull().And.BeEmpty();
        }

        [Fact]
        public void TriggerType_CanBeSetToOnWave()
        {
            var scriptedEvent = new ScriptedEvent { TriggerType = TriggerType.OnWave };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnWave);
        }

        [Fact]
        public void TriggerType_CanBeSetToOnTime()
        {
            var scriptedEvent = new ScriptedEvent { TriggerType = TriggerType.OnTime };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnTime);
        }

        [Fact]
        public void TriggerType_CanBeSetToOnPopulation()
        {
            var scriptedEvent = new ScriptedEvent { TriggerType = TriggerType.OnPopulation };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnPopulation);
        }

        [Fact]
        public void TriggerType_CanBeSetToOnResource()
        {
            var scriptedEvent = new ScriptedEvent { TriggerType = TriggerType.OnResource };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnResource);
        }

        [Fact]
        public void TriggerType_CanBeSetToOnBuildingBuilt()
        {
            var scriptedEvent = new ScriptedEvent { TriggerType = TriggerType.OnBuildingBuilt };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnBuildingBuilt);
        }

        [Fact]
        public void TriggerType_CanBeSetToOnUnitKilled()
        {
            var scriptedEvent = new ScriptedEvent { TriggerType = TriggerType.OnUnitKilled };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnUnitKilled);
        }

        [Fact]
        public void TriggerValue_CanBePositive()
        {
            var scriptedEvent = new ScriptedEvent { TriggerValue = 10 };

            scriptedEvent.TriggerValue.Should().Be(10);
        }

        [Fact]
        public void TriggerValue_CanBeZero()
        {
            var scriptedEvent = new ScriptedEvent { TriggerValue = 0 };

            scriptedEvent.TriggerValue.Should().Be(0);
        }

        [Fact]
        public void TriggerValue_CanBeNegative()
        {
            var scriptedEvent = new ScriptedEvent { TriggerValue = -5 };

            scriptedEvent.TriggerValue.Should().Be(-5);
        }

        [Fact]
        public void TriggerTarget_CanBeSet()
        {
            var scriptedEvent = new ScriptedEvent { TriggerTarget = "gold" };

            scriptedEvent.TriggerTarget.Should().Be("gold");
        }

        [Fact]
        public void TriggerTarget_CanBeNull()
        {
            var scriptedEvent = new ScriptedEvent { TriggerTarget = null };

            scriptedEvent.TriggerTarget.Should().BeNull();
        }

        [Fact]
        public void Actions_IsEmptyByDefault()
        {
            var scriptedEvent = new ScriptedEvent();

            scriptedEvent.Actions.Should().BeEmpty();
        }

        [Fact]
        public void Actions_CanAddEventAction()
        {
            var scriptedEvent = new ScriptedEvent();
            var action = new EventAction();
            scriptedEvent.Actions.Add(action);

            scriptedEvent.Actions.Should().HaveCount(1);
            scriptedEvent.Actions[0].Should().Be(action);
        }

        [Fact]
        public void Actions_CanAddMultipleEventActions()
        {
            var scriptedEvent = new ScriptedEvent();
            var action1 = new EventAction();
            var action2 = new EventAction();
            var action3 = new EventAction();

            scriptedEvent.Actions.Add(action1);
            scriptedEvent.Actions.Add(action2);
            scriptedEvent.Actions.Add(action3);

            scriptedEvent.Actions.Should().HaveCount(3);
            scriptedEvent.Actions.Should().Contain(new[] { action1, action2, action3 });
        }

        [Fact]
        public void Id_CanBeSet()
        {
            var scriptedEvent = new ScriptedEvent { Id = "event-wave-5" };

            scriptedEvent.Id.Should().Be("event-wave-5");
        }

        [Fact]
        public void Id_DefaultsToEmpty()
        {
            var scriptedEvent = new ScriptedEvent();

            scriptedEvent.Id.Should().Be(string.Empty);
        }

        [Fact]
        public void CanCreateWaveTriggeredEvent()
        {
            var scriptedEvent = new ScriptedEvent
            {
                Id = "spawn-reinforcements",
                TriggerType = TriggerType.OnWave,
                TriggerValue = 5,
                TriggerTarget = null,
                Actions = new()
            };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnWave);
            scriptedEvent.TriggerValue.Should().Be(5);
        }

        [Fact]
        public void CanCreateTimeTriggeredEvent()
        {
            var scriptedEvent = new ScriptedEvent
            {
                Id = "supply-bonus",
                TriggerType = TriggerType.OnTime,
                TriggerValue = 120,
                TriggerTarget = null
            };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnTime);
            scriptedEvent.TriggerValue.Should().Be(120);
        }

        [Fact]
        public void CanCreatePopulationTriggeredEvent()
        {
            var scriptedEvent = new ScriptedEvent
            {
                Id = "population-bonus",
                TriggerType = TriggerType.OnPopulation,
                TriggerValue = 500,
                TriggerTarget = "faction-player"
            };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnPopulation);
            scriptedEvent.TriggerTarget.Should().Be("faction-player");
        }

        [Fact]
        public void CanCreateResourceTriggeredEvent()
        {
            var scriptedEvent = new ScriptedEvent
            {
                Id = "gold-threshold-event",
                TriggerType = TriggerType.OnResource,
                TriggerValue = 1000,
                TriggerTarget = "gold"
            };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnResource);
            scriptedEvent.TriggerTarget.Should().Be("gold");
        }

        [Fact]
        public void CanCreateBuildingTriggeredEvent()
        {
            var scriptedEvent = new ScriptedEvent
            {
                Id = "castle-built-event",
                TriggerType = TriggerType.OnBuildingBuilt,
                TriggerValue = 0,
                TriggerTarget = "castle"
            };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnBuildingBuilt);
            scriptedEvent.TriggerTarget.Should().Be("castle");
        }

        [Fact]
        public void CanCreateUnitKilledTriggeredEvent()
        {
            var scriptedEvent = new ScriptedEvent
            {
                Id = "enemy-general-killed",
                TriggerType = TriggerType.OnUnitKilled,
                TriggerValue = 1,
                TriggerTarget = "enemy-general-unit"
            };

            scriptedEvent.TriggerType.Should().Be(TriggerType.OnUnitKilled);
            scriptedEvent.TriggerTarget.Should().Be("enemy-general-unit");
        }

        [Fact]
        public void AllTriggerTypes_AreAccessible()
        {
            var allTypes = new[]
            {
                TriggerType.OnWave,
                TriggerType.OnTime,
                TriggerType.OnPopulation,
                TriggerType.OnResource,
                TriggerType.OnBuildingBuilt,
                TriggerType.OnUnitKilled
            };

            foreach (var type in allTypes)
            {
                var scriptedEvent = new ScriptedEvent { TriggerType = type };
                scriptedEvent.TriggerType.Should().Be(type);
            }
        }

        [Fact]
        public void ActionsCanBeCleared()
        {
            var scriptedEvent = new ScriptedEvent();
            scriptedEvent.Actions.Add(new EventAction());
            scriptedEvent.Actions.Add(new EventAction());

            scriptedEvent.Actions.Clear();

            scriptedEvent.Actions.Should().BeEmpty();
        }

        [Fact]
        public void ActionsCanBeRemovedByIndex()
        {
            var scriptedEvent = new ScriptedEvent();
            var action1 = new EventAction();
            var action2 = new EventAction();

            scriptedEvent.Actions.Add(action1);
            scriptedEvent.Actions.Add(action2);
            scriptedEvent.Actions.RemoveAt(0);

            scriptedEvent.Actions.Should().HaveCount(1);
            scriptedEvent.Actions[0].Should().Be(action2);
        }

        [Fact]
        public void PropertiesAreIndependent()
        {
            var event1 = new ScriptedEvent { Id = "event-1", TriggerType = TriggerType.OnWave };
            var event2 = new ScriptedEvent { Id = "event-2", TriggerType = TriggerType.OnTime };

            event1.Id.Should().NotBe(event2.Id);
            event1.TriggerType.Should().NotBe(event2.TriggerType);
        }
    }
}
