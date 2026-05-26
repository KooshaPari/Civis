using System;
using System.Collections.Generic;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Unit tests for MockEntityManager, MockEntityQuery, and EntityQueryBuilder.
    /// Validates core ECS mocking functionality for use in AssetSwapSystem and related tests.
    /// </summary>
    public class MockEntityManagerTests : IDisposable
    {
        private readonly MockEntityManager _manager;

        public MockEntityManagerTests()
        {
            _manager = new MockEntityManager();
        }

        public void Dispose()
        {
            _manager.Clear();
        }

        #region Entity Creation Tests

        [Fact]
        public void CreateEntity_ReturnsValidEntity()
        {
            // Act
            var entity = _manager.CreateEntity();

            // Assert
            entity.Index.Should().Be(0);
            entity.Version.Should().Be(0);
        }

        [Fact]
        public void CreateEntity_MultipleEntities_HasUniqueIndices()
        {
            // Act
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();
            var entity3 = _manager.CreateEntity();

            // Assert
            entity1.Index.Should().NotBe(entity2.Index);
            entity2.Index.Should().NotBe(entity3.Index);
            _manager.GetEntityCount().Should().Be(3);
        }

        [Fact]
        public void CreateEntity_WithCount_CreatesMultipleEntities()
        {
            // Act
            var entities = _manager.CreateEntity(5);

            // Assert
            entities.Should().HaveCount(5);
            _manager.GetEntityCount().Should().Be(5);

            for (int i = 0; i < entities.Count; i++)
            {
                entities[i].Index.Should().Be(i);
            }
        }

        #endregion

        #region Component Addition/Removal Tests

        [Fact]
        public void AddComponent_AddsComponentToEntity()
        {
            // Arrange
            var entity = _manager.CreateEntity();

            // Act
            _manager.AddComponent<TestComponent>(entity);

            // Assert
            _manager.HasComponent<TestComponent>(entity).Should().BeTrue();
        }

        [Fact]
        public void AddComponent_WithData_StoresComponentData()
        {
            // Arrange
            var entity = _manager.CreateEntity();
            var data = new TestComponent { Value = 42 };

            // Act
            _manager.AddComponent(entity, data);

            // Assert
            var retrieved = _manager.GetComponentData<TestComponent>(entity);
            retrieved.Value.Should().Be(42);
        }

        [Fact]
        public void RemoveComponent_RemovesComponentFromEntity()
        {
            // Arrange
            var entity = _manager.CreateEntity();
            _manager.AddComponent<TestComponent>(entity);
            _manager.HasComponent<TestComponent>(entity).Should().BeTrue();

            // Act
            _manager.RemoveComponent<TestComponent>(entity);

            // Assert
            _manager.HasComponent<TestComponent>(entity).Should().BeFalse();
        }

        [Fact]
        public void AddMultipleComponents_AllAddedCorrectly()
        {
            // Arrange
            var entity = _manager.CreateEntity();

            // Act
            _manager.AddComponent<TestComponent>(entity);
            _manager.AddComponent<AnotherComponent>(entity);
            _manager.AddComponent<ThirdComponent>(entity);

            // Assert
            _manager.HasComponent<TestComponent>(entity).Should().BeTrue();
            _manager.HasComponent<AnotherComponent>(entity).Should().BeTrue();
            _manager.HasComponent<ThirdComponent>(entity).Should().BeTrue();
        }

        [Fact]
        public void GetComponentData_ThrowsOnMissingComponent()
        {
            // Arrange
            var entity = _manager.CreateEntity();

            // Act
            Action act = () => _manager.GetComponentData<TestComponent>(entity);

            // Assert
            act.Should().Throw<InvalidOperationException>();
        }

        [Fact]
        public void SetComponentData_UpdatesExistingComponentData()
        {
            // Arrange
            var entity = _manager.CreateEntity();
            _manager.AddComponent(entity, new TestComponent { Value = 10 });

            // Act
            _manager.SetComponentData(entity, new TestComponent { Value = 20 });

            // Assert
            var retrieved = _manager.GetComponentData<TestComponent>(entity);
            retrieved.Value.Should().Be(20);
        }

        #endregion

        #region Query Tests

        [Fact]
        public void CreateEntityQuery_SimpleQuery_FiltersCorrectly()
        {
            // Arrange
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();
            var entity3 = _manager.CreateEntity();

            _manager.AddComponent<TestComponent>(entity1);
            _manager.AddComponent<TestComponent>(entity2);
            // entity3 has no TestComponent

            // Act
            var query = _manager.CreateEntityQuery(typeof(TestComponent));
            var results = query.ToEntityArray();

            // Assert
            results.Should().HaveCount(2);
            results.Should().Contain(entity1);
            results.Should().Contain(entity2);
            results.Should().NotContain(entity3);
        }

        [Fact]
        public void CreateEntityQuery_MultipleComponents_RequiresAll()
        {
            // Arrange
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();
            var entity3 = _manager.CreateEntity();

            // entity1 has both
            _manager.AddComponent<TestComponent>(entity1);
            _manager.AddComponent<AnotherComponent>(entity1);

            // entity2 has only TestComponent
            _manager.AddComponent<TestComponent>(entity2);

            // entity3 has only AnotherComponent
            _manager.AddComponent<AnotherComponent>(entity3);

            // Act
            var query = _manager.CreateEntityQuery(
                typeof(TestComponent), typeof(AnotherComponent));
            var results = query.ToEntityArray();

            // Assert
            results.Should().HaveCount(1);
            results.Should().Contain(entity1);
        }

        [Fact]
        public void CreateEntityQuery_WithoutAll_ExcludesEntities()
        {
            // Arrange
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();

            _manager.AddComponent<TestComponent>(entity1);
            _manager.AddComponent<TestComponent>(entity2);
            _manager.AddComponent<AnotherComponent>(entity2);

            // Act
            var query = _manager.CreateEntityQuery(
                withAll: new[] { typeof(TestComponent) },
                withoutAll: new[] { typeof(AnotherComponent) });
            var results = query.ToEntityArray();

            // Assert
            results.Should().HaveCount(1);
            results.Should().Contain(entity1);
            results.Should().NotContain(entity2);
        }

        #endregion

        #region EntityQueryBuilder Tests

        [Fact]
        public void EntityQueryBuilder_WithAll_FiltersCorrectly()
        {
            // Arrange
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();

            _manager.AddComponent<TestComponent>(entity1);
            _manager.AddComponent<TestComponent>(entity2);
            _manager.AddComponent<AnotherComponent>(entity2);

            // Act
            var results = _manager.CreateEntityQueryBuilder()
                .WithAll<TestComponent>()
                .WithAll<AnotherComponent>()
                .Execute();

            // Assert
            results.Should().HaveCount(1);
            results.Should().Contain(entity2);
        }

        [Fact]
        public void EntityQueryBuilder_WithoutAll_ExcludesEntities()
        {
            // Arrange
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();
            var entity3 = _manager.CreateEntity();

            _manager.AddComponent<TestComponent>(entity1);
            _manager.AddComponent<TestComponent>(entity2);
            _manager.AddComponent<AnotherComponent>(entity2);

            _manager.AddComponent<TestComponent>(entity3);
            _manager.AddComponent<ThirdComponent>(entity3);

            // Act
            var results = _manager.CreateEntityQueryBuilder()
                .WithAll<TestComponent>()
                .WithoutAll<AnotherComponent>()
                .Execute();

            // Assert
            results.Should().HaveCount(2);
            results.Should().Contain(entity1);
            results.Should().Contain(entity3);
            results.Should().NotContain(entity2);
        }

        [Fact]
        public void EntityQueryBuilder_ChainedCalls_BuildsCorrectQuery()
        {
            // Arrange
            var entity1 = _manager.CreateEntity();
            var entity2 = _manager.CreateEntity();
            var entity3 = _manager.CreateEntity();

            _manager.AddComponent<TestComponent>(entity1);
            _manager.AddComponent<TestComponent>(entity2);
            _manager.AddComponent<TestComponent>(entity3);

            _manager.AddComponent<AnotherComponent>(entity1);
            _manager.AddComponent<ThirdComponent>(entity3);

            // Act
            var query = _manager.CreateEntityQueryBuilder()
                .WithAll<TestComponent>()
                .WithoutAll<ThirdComponent>()
                .Build();

            var results = query.ToEntityArray();

            // Assert
            results.Should().HaveCount(2);
            results.Should().Contain(entity1);
            results.Should().Contain(entity2);
            results.Should().NotContain(entity3);
        }

        [Fact]
        public void EntityQueryBuilder_Count_ReturnsCorrectCount()
        {
            // Arrange
            for (int i = 0; i < 5; i++)
            {
                var entity = _manager.CreateEntity();
                _manager.AddComponent<TestComponent>(entity);
            }

            // Act
            var count = _manager.CreateEntityQueryBuilder()
                .WithAll<TestComponent>()
                .Count();

            // Assert
            count.Should().Be(5);
        }

        #endregion

        #region Edge Cases

        [Fact]
        public void EmptyQuery_ReturnsNoEntities()
        {
            // Act
            var query = _manager.CreateEntityQuery();

            // Assert
            query.ToEntityArray().Should().BeEmpty();
        }

        [Fact]
        public void Query_NoMatchingEntities_ReturnsEmpty()
        {
            // Arrange
            var entity = _manager.CreateEntity();
            _manager.AddComponent<TestComponent>(entity);

            // Act
            var query = _manager.CreateEntityQuery(typeof(AnotherComponent));
            var results = query.ToEntityArray();

            // Assert
            results.Should().BeEmpty();
        }

        [Fact]
        public void Clear_ResetsAllState()
        {
            // Arrange
            _manager.CreateEntity();
            _manager.CreateEntity();
            _manager.GetEntityCount().Should().Be(2);

            // Act
            _manager.Clear();

            // Assert
            _manager.GetEntityCount().Should().Be(0);
            var newEntity = _manager.CreateEntity();
            newEntity.Index.Should().Be(0);
        }

        [Fact]
        public void MockComponentType_ReadOnly_DifferentiatesAccessMode()
        {
            // Act & Assert
            var readOnly = MockComponentType.ReadOnly(typeof(TestComponent));
            var readWrite = MockComponentType.ReadWrite(typeof(TestComponent));

            readOnly.IsReadOnly.Should().BeTrue();
            readWrite.IsReadOnly.Should().BeFalse();
            readOnly.Should().NotBe(readWrite);
        }

        [Fact]
        public void MockEntity_Equality_WorksCorrectly()
        {
            // Act & Assert
            var entity1 = new MockEntity(1, 0);
            var entity2 = new MockEntity(1, 0);
            var entity3 = new MockEntity(2, 0);

            entity1.Should().Be(entity2);
            entity1.Should().NotBe(entity3);
            entity1.GetHashCode().Should().Be(entity2.GetHashCode());
        }

        #endregion

        // Test marker types used in queries
        private class TestComponent
        {
            public int Value { get; set; }
        }

        private class AnotherComponent
        {
        }

        private class ThirdComponent
        {
        }
    }
}
