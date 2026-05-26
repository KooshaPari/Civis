#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.PropertyTests
{
    /// <summary>
    /// Property-based tests for generic registry operations.
    /// Verifies invariants: registration count, retrieval correctness, conflict detection.
    /// </summary>
    [Trait("Category", "Property")]
    public class RegistryPropertyTests
    {
        private sealed class TestEntry
        {
            public string Id { get; set; }
            public string Name { get; set; } = "Test";

            public TestEntry(string id) => Id = id;
        }

        [Theory(DisplayName = "Property: Register entry can be retrieved")]
        [InlineData("entry-1")]
        [InlineData("faction-alpha")]
        [InlineData("doctrine-blitz")]
        public void Register_Then_Get_Returns_Entry(string id)
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            var entry = new TestEntry(id);

            // Act
            registry.Register(id, entry, RegistrySource.BaseGame, "test-pack", 100);
            var retrieved = registry.Get(id);

            // Assert
            retrieved.Should().NotBeNull(because: "Get must return registered entry");
            retrieved!.Id.Should().Be(id, because: "Retrieved entry must have correct ID");
        }

        [Theory(DisplayName = "Property: Contains returns true iff entry was registered")]
        [InlineData("registered", true)]
        [InlineData("not-registered", false)]
        public void Contains_Matches_Registration_Status(string id, bool shouldExist)
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            if (shouldExist)
            {
                registry.Register(id, new TestEntry(id), RegistrySource.BaseGame, "test-pack", 100);
            }

            // Act
            var contains = registry.Contains(id);

            // Assert
            contains.Should().Be(shouldExist,
                because: "Contains must reflect registration status");
        }

        [Theory(DisplayName = "Property: GetAll (via All property) count matches registrations")]
        [InlineData(1)]
        [InlineData(5)]
        [InlineData(10)]
        public void All_Property_Count_Matches_Registrations(int count)
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            var entries = Enumerable.Range(0, count)
                .Select(i => new TestEntry($"entry-{i}"))
                .ToList();

            // Act
            foreach (var entry in entries)
            {
                registry.Register(entry.Id, entry, RegistrySource.BaseGame, "test-pack", 100);
            }

            var allEntries = registry.All;

            // Assert
            allEntries.Should().HaveCount(count,
                because: "All property count must match registration count");
        }

        [Theory(DisplayName = "Property: Get non-existent ID returns null")]
        [InlineData("nonexistent")]
        [InlineData("not-registered")]
        public void Get_NonExistent_Returns_Null(string id)
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            registry.Register("existing", new TestEntry("existing"), RegistrySource.BaseGame, "test-pack", 100);

            // Act
            var result = registry.Get(id);

            // Assert
            result.Should().BeNull(
                because: "Get must return null for non-existent ID, not throw");
        }

        [Fact(DisplayName = "Property: Override replaces existing entry")]
        public void Override_Replaces_Entry()
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            var original = new TestEntry("test");
            var replacement = new TestEntry("test") { Name = "Replaced" };

            registry.Register("test", original, RegistrySource.BaseGame, "pack1", 100);

            // Act
            registry.Override("test", replacement, RegistrySource.Pack, "pack2", 200);
            var retrieved = registry.Get("test");

            // Assert
            retrieved.Should().NotBeNull();
            retrieved!.Name.Should().Be("Replaced",
                because: "Override must replace the entry");
        }

        [Fact(DisplayName = "Property: Higher load order takes precedence in conflict")]
        public void Higher_LoadOrder_Takes_Precedence()
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            var entry1 = new TestEntry("conflict") { Name = "Lower Priority" };
            var entry2 = new TestEntry("conflict") { Name = "Higher Priority" };

            // Act
            registry.Register("conflict", entry1, RegistrySource.BaseGame, "pack1", 100);
            registry.Register("conflict", entry2, RegistrySource.BaseGame, "pack2", 200);
            var retrieved = registry.Get("conflict");

            // Assert
            retrieved.Should().NotBeNull();
            retrieved!.Name.Should().Be("Higher Priority",
                because: "Higher load order should take precedence");
        }

        [Fact(DisplayName = "Property: DetectConflicts finds entries with duplicate top priority")]
        public void DetectConflicts_Finds_Ties()
        {
            // Arrange
            var registry = new Registry<TestEntry>();
            var entry1 = new TestEntry("conflict1");
            var entry2 = new TestEntry("conflict1");

            // Register with SAME load order on SAME source = conflict
            registry.Register("conflict1", entry1, RegistrySource.BaseGame, "pack1", 100);
            registry.Register("conflict1", entry2, RegistrySource.BaseGame, "pack2", 100);

            // Act
            var conflicts = registry.DetectConflicts();

            // Assert
            conflicts.Should().NotBeEmpty(
                because: "Equal priority registrations should be detected as conflicts");
        }

        [Theory(DisplayName = "Property: Multiple registrations without conflicts are allowed")]
        [InlineData(1)]
        [InlineData(5)]
        [InlineData(20)]
        public void Multiple_Different_Entries_No_Conflict(int count)
        {
            // Arrange
            var registry = new Registry<TestEntry>();

            // Act
            for (int i = 0; i < count; i++)
            {
                var entry = new TestEntry($"entry-{i}");
                registry.Register(entry.Id, entry, RegistrySource.BaseGame, "test-pack", 100 + i);
            }

            var conflicts = registry.DetectConflicts();

            // Assert
            conflicts.Should().BeEmpty(
                because: "Different entries should not conflict");
            registry.All.Should().HaveCount(count,
                because: $"All {count} entries should be registered");
        }
    }
}
