using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public record TestItem(string Name);

    public class RegistryTests
    {
        [Fact]
        public void Register_CaseVariant_LogsWarning()
        {
            // #771: registering an id that differs only in casing from a prior id
            // should emit a warning via the injected logger sink.
            List<string> warnings = new List<string>();
            var registry = new Registry<TestItem>(warnings.Add);

            registry.Register("clone-trooper", new TestItem("CT"), RegistrySource.Pack, "p1");
            registry.Register("Clone-Trooper", new TestItem("CT-Caps"), RegistrySource.Pack, "p2");

            warnings.Should().ContainSingle(w => w.Contains("Case-variant collision") && w.Contains("Clone-Trooper"));
        }

        [Fact]
        public void Register_SameIdTwice_DoesNotLogCaseVariantWarning()
        {
            // #771: exact-duplicate registration is a normal override path and must NOT
            // emit the case-variant warning.
            List<string> warnings = new List<string>();
            var registry = new Registry<TestItem>(warnings.Add);

            registry.Register("clone-trooper", new TestItem("A"), RegistrySource.Pack, "p1");
            registry.Register("clone-trooper", new TestItem("B"), RegistrySource.Pack, "p2");

            warnings.Should().BeEmpty();
        }

        [Fact]
        public void Register_And_Get_ReturnsEntry()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("Raptor"), RegistrySource.BaseGame, "base-game");

            TestItem? result = registry.Get("raptor");

            result.Should().NotBeNull();
            result!.Name.Should().Be("Raptor");
        }

        [Fact]
        public void Get_NonExistent_ReturnsNull()
        {
            var registry = new Registry<TestItem>();

            TestItem? result = registry.Get("does-not-exist");

            result.Should().BeNull();
        }

        [Fact]
        public void Contains_Registered_ReturnsTrue()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("Raptor"), RegistrySource.BaseGame, "base-game");

            registry.Contains("raptor").Should().BeTrue();
        }

        [Fact]
        public void Contains_NotRegistered_ReturnsFalse()
        {
            var registry = new Registry<TestItem>();

            registry.Contains("raptor").Should().BeFalse();
        }

        [Fact]
        public void All_ReturnsHighestPriorityEntries()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("BaseRaptor"), RegistrySource.BaseGame, "base-game");
            registry.Register("raptor", new TestItem("PackRaptor"), RegistrySource.Pack, "dino-pack");

            var all = registry.All;

            all.Should().ContainKey("raptor");
            all["raptor"].Data.Name.Should().Be("PackRaptor");
        }

        [Fact]
        public void Override_HigherPriority_ReplacesExisting()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("BaseRaptor"), RegistrySource.BaseGame, "base-game");
            registry.Override("raptor", new TestItem("PackRaptor"), RegistrySource.Pack, "dino-pack");

            registry.Get("raptor")!.Name.Should().Be("PackRaptor");
        }

        [Fact]
        public void Override_SamePriority_CreatesConflict()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("PackA"), RegistrySource.Pack, "pack-a", 100);
            registry.Override("raptor", new TestItem("PackB"), RegistrySource.Pack, "pack-b", 100);

            var conflicts = registry.DetectConflicts();

            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("raptor");
        }

        [Fact]
        public void DetectConflicts_NoConflicts_ReturnsEmpty()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("BaseRaptor"), RegistrySource.BaseGame, "base-game");
            registry.Register("raptor", new TestItem("PackRaptor"), RegistrySource.Pack, "dino-pack");

            var conflicts = registry.DetectConflicts();

            conflicts.Should().BeEmpty();
        }

        [Fact]
        public void DetectConflicts_SamePriority_ReturnsConflict()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("Alpha"), RegistrySource.Pack, "pack-alpha", 100);
            registry.Register("raptor", new TestItem("Beta"), RegistrySource.Pack, "pack-beta", 100);

            var conflicts = registry.DetectConflicts();

            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("raptor");
        }

        [Fact]
        public void Register_MultipleEntries_AllAccessible()
        {
            var registry = new Registry<TestItem>();
            registry.Register("raptor", new TestItem("Raptor"), RegistrySource.BaseGame, "base");
            registry.Register("triceratops", new TestItem("Triceratops"), RegistrySource.BaseGame, "base");
            registry.Register("ankylosaur", new TestItem("Ankylosaur"), RegistrySource.BaseGame, "base");

            registry.Contains("raptor").Should().BeTrue();
            registry.Contains("triceratops").Should().BeTrue();
            registry.Contains("ankylosaur").Should().BeTrue();
            registry.Get("triceratops")!.Name.Should().Be("Triceratops");
        }

        [Fact]
        public void Registry_GetNonExistent_ReturnsNull()
        {
            // Arrange
            var registry = new Registry<TestItem>();

            // Act
            TestItem? result = registry.Get("does-not-exist");

            // Assert
            result.Should().BeNull();
        }

        [Fact]
        public void Registry_Contains_CaseSensitive()
        {
            // Arrange
            var registry = new Registry<TestItem>();
            registry.Register("MyItem", new TestItem("Test"), RegistrySource.BaseGame, "base");

            // Act & Assert - Pattern #99 governance: registry IDs are case-sensitive
            // (StringComparer.Ordinal). User-sourced keys must match exactly.
            registry.Contains("MyItem").Should().BeTrue();
            registry.Contains("myitem").Should().BeFalse();
            registry.Contains("MYITEM").Should().BeFalse();
        }

        [Fact]
        public void Registry_All_WithMultipleEntries_ReturnsAll()
        {
            // Arrange
            var registry = new Registry<TestItem>();
            registry.Register("item1", new TestItem("Item 1"), RegistrySource.BaseGame, "base");
            registry.Register("item2", new TestItem("Item 2"), RegistrySource.BaseGame, "base");
            registry.Register("item3", new TestItem("Item 3"), RegistrySource.BaseGame, "base");

            // Act
            var all = registry.All;

            // Assert
            all.Should().HaveCount(3);
            all.Should().ContainKeys("item1", "item2", "item3");
        }

        [Fact]
        public void Registry_Register_OverwritesExisting()
        {
            // Arrange
            var registry = new Registry<TestItem>();
            registry.Register("item", new TestItem("Original"), RegistrySource.BaseGame, "base", 100);
            registry.Register("item", new TestItem("Override"), RegistrySource.Pack, "pack-1", 200);

            // Act
            TestItem? result = registry.Get("item");

            // Assert - highest priority (200) should win
            result.Should().NotBeNull();
            result!.Name.Should().Be("Override");
        }

        [Fact]
        public void Registry_Empty_All_ReturnsEmpty()
        {
            // Arrange
            var registry = new Registry<TestItem>();

            // Act
            var all = registry.All;

            // Assert
            all.Should().BeEmpty();
            all.Should().HaveCount(0);
        }
    }
}
