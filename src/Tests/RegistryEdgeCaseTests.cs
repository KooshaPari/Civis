using System;
using System.Collections.Generic;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Edge-case tests for <see cref="Registry{T}"/> covering priority resolution,
    /// null guards, empty states, and TryGet-style access patterns.
    /// </summary>
    public class RegistryEdgeCaseTests
    {
        // re-use the TestItem record already declared in RegistryTests.cs would cause
        // a duplicate type compilation error, so we use a local alias via a wrapper.
        private record EdgeItem(string Label);

        // ─── priority resolution ──────────────────────────────────────────────

        [Fact]
        public void Register_SameIdDifferentPriority_HigherPriorityWins()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("raptor", new EdgeItem("BaseLow"), RegistrySource.BaseGame, "base", loadOrder: 10);
            registry.Register("raptor", new EdgeItem("PackHigh"), RegistrySource.Pack, "custom-pack", loadOrder: 200);

            // Act
            EdgeItem? result = registry.Get("raptor");

            // Assert
            result.Should().NotBeNull();
            result!.Label.Should().Be("PackHigh");
        }

        [Fact]
        public void Register_SameIdLowerPriority_OriginalEntryWins()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("unit", new EdgeItem("HighPriority"), RegistrySource.Pack, "pack-a", loadOrder: 500);
            registry.Register("unit", new EdgeItem("LowPriority"), RegistrySource.BaseGame, "base", loadOrder: 10);

            // Act
            EdgeItem? result = registry.Get("unit");

            // Assert
            result.Should().NotBeNull();
            result!.Label.Should().Be("HighPriority");
        }

        // ─── empty registry ───────────────────────────────────────────────────

        [Fact]
        public void GetAll_EmptyRegistry_ReturnsEmpty()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();

            // Act
            IReadOnlyDictionary<string, RegistryEntry<EdgeItem>> all = registry.All;

            // Assert
            all.Should().BeEmpty();
        }

        [Fact]
        public void Get_EmptyRegistry_ReturnsNull()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();

            // Act
            EdgeItem? result = registry.Get("any-id");

            // Assert
            result.Should().BeNull();
        }

        [Fact]
        public void Contains_EmptyRegistry_ReturnsFalse()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();

            // Act & Assert
            registry.Contains("missing").Should().BeFalse();
        }

        // ─── TryGet pattern (via Get + Contains) ─────────────────────────────

        [Fact]
        public void TryGet_AfterRegister_ReturnsTrueAndValue()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("sword", new EdgeItem("Sword"), RegistrySource.BaseGame, "base");

            // Act
            bool found = registry.Contains("sword");
            EdgeItem? value = registry.Get("sword");

            // Assert
            found.Should().BeTrue();
            value.Should().NotBeNull();
            value!.Label.Should().Be("Sword");
        }

        [Fact]
        public void TryGet_NotRegistered_ReturnsFalseAndNull()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();

            // Act
            bool found = registry.Contains("ghost");
            EdgeItem? value = registry.Get("ghost");

            // Assert
            found.Should().BeFalse();
            value.Should().BeNull();
        }

        // ─── multiple entries, All property ──────────────────────────────────

        [Fact]
        public void Register_MultipleDistinctIds_AllReturnedViaAll()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("a", new EdgeItem("A"), RegistrySource.BaseGame, "base");
            registry.Register("b", new EdgeItem("B"), RegistrySource.BaseGame, "base");
            registry.Register("c", new EdgeItem("C"), RegistrySource.BaseGame, "base");

            // Act
            IReadOnlyDictionary<string, RegistryEntry<EdgeItem>> all = registry.All;

            // Assert
            all.Should().HaveCount(3);
            all.Should().ContainKey("a");
            all.Should().ContainKey("b");
            all.Should().ContainKey("c");
        }

        // ─── case-sensitive lookups (Pattern #99) ─────────────────────────────

        [Fact]
        public void Get_CaseSensitive_OnlyExactMatchFound()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("MyUnit", new EdgeItem("MyUnit"), RegistrySource.BaseGame, "base");

            // Pattern #99: Registry keys are case-sensitive (StringComparer.Ordinal).
            // User-sourced IDs must match exactly.
            registry.Get("MyUnit")!.Label.Should().Be("MyUnit");
            registry.Get("myunit").Should().BeNull();
            registry.Get("MYUNIT").Should().BeNull();
        }

        // ─── conflict detection ───────────────────────────────────────────────

        [Fact]
        public void DetectConflicts_TwoEntriesSamePriority_ReturnsConflict()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("raptor", new EdgeItem("A"), RegistrySource.Pack, "pack-a", loadOrder: 100);
            registry.Register("raptor", new EdgeItem("B"), RegistrySource.Pack, "pack-b", loadOrder: 100);

            // Act
            IReadOnlyList<RegistryConflict> conflicts = registry.DetectConflicts();

            // Assert
            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("raptor");
        }

        [Fact]
        public void DetectConflicts_EntriesDifferentPriority_NoConflict()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("raptor", new EdgeItem("Base"), RegistrySource.BaseGame, "base", loadOrder: 10);
            registry.Register("raptor", new EdgeItem("Pack"), RegistrySource.Pack, "pack-a", loadOrder: 200);

            // Act
            IReadOnlyList<RegistryConflict> conflicts = registry.DetectConflicts();

            // Assert
            conflicts.Should().BeEmpty();
        }

        // ─── override behaviour ───────────────────────────────────────────────

        [Fact]
        public void Override_HigherPriority_ReplacesGetResult()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("unit", new EdgeItem("Original"), RegistrySource.BaseGame, "base", loadOrder: 50);

            // Act
            registry.Override("unit", new EdgeItem("Overridden"), RegistrySource.Pack, "override-pack", loadOrder: 300);

            // Assert
            registry.Get("unit")!.Label.Should().Be("Overridden");
        }

        // ─── All count after multiple registrations of same id ────────────────

        [Fact]
        public void All_MultiplePrioritiesForSameId_CountsAsOneEntry()
        {
            // Arrange
            Registry<EdgeItem> registry = new Registry<EdgeItem>();
            registry.Register("unit", new EdgeItem("V1"), RegistrySource.BaseGame, "base", loadOrder: 10);
            registry.Register("unit", new EdgeItem("V2"), RegistrySource.Pack, "pack-a", loadOrder: 200);
            registry.Register("unit", new EdgeItem("V3"), RegistrySource.Pack, "pack-b", loadOrder: 300);

            // Act
            IReadOnlyDictionary<string, RegistryEntry<EdgeItem>> all = registry.All;

            // Assert — same id counts once; highest priority wins
            all.Should().HaveCount(1);
            all["unit"].Data.Label.Should().Be("V3");
        }
    }
}
