using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using DINOForge.SDK.Assets;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="AssetSwapRegistry"/> covering registration, retrieval,
    /// thread safety, and state management.
    /// </summary>
    public class AssetSwapRegistryTests : IDisposable
    {
        public AssetSwapRegistryTests()
        {
            // Ensure clean state for each test (Clear is internal, accessible via InternalsVisibleTo)
            AssetSwapRegistry.Clear();
        }

        public void Dispose()
        {
            AssetSwapRegistry.Clear();
        }

        [Fact]
        public void Register_SingleItem_CanBeRetrievedViaPending()
        {
            // Arrange
            AssetSwapRequest request = new AssetSwapRequest("addr/unit.prefab", "/path/mod.bundle", "UnitAsset");

            // Act
            AssetSwapRegistry.Register(request);
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().HaveCount(1);
            pending[0].AssetAddress.Should().Be("addr/unit.prefab");
            pending[0].ModBundlePath.Should().Be("/path/mod.bundle");
            pending[0].AssetName.Should().Be("UnitAsset");
        }

        [Fact]
        public void Register_MultipleItems_AllReturnedAsPending()
        {
            // Arrange
            AssetSwapRequest req1 = new AssetSwapRequest("addr/unit1.prefab", "/path/mod.bundle", "Unit1");
            AssetSwapRequest req2 = new AssetSwapRequest("addr/unit2.prefab", "/path/mod.bundle", "Unit2");
            AssetSwapRequest req3 = new AssetSwapRequest("addr/unit3.prefab", "/path/mod.bundle", "Unit3");

            // Act
            AssetSwapRegistry.Register(req1);
            AssetSwapRegistry.Register(req2);
            AssetSwapRegistry.Register(req3);
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().HaveCount(3);
        }

        [Fact]
        public void GetPending_EmptyRegistry_ReturnsEmpty()
        {
            // Act
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().BeEmpty();
        }

        [Fact]
        public void Register_NullRequest_ThrowsArgumentNullException()
        {
            // Act
            Action act = () => AssetSwapRegistry.Register(null!);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void GetPending_AfterClear_ReturnsEmpty()
        {
            // Arrange
            AssetSwapRegistry.Register(new AssetSwapRequest("addr/test.prefab", "/mod.bundle", "TestAsset"));
            AssetSwapRegistry.Count.Should().Be(1);

            // Act
            AssetSwapRegistry.Clear();
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().BeEmpty();
            AssetSwapRegistry.Count.Should().Be(0);
        }

        [Fact]
        public void Register_SameAddress_OverwritesPreviousEntry()
        {
            // Arrange
            AssetSwapRequest original = new AssetSwapRequest("addr/unit.prefab", "/original.bundle", "OriginalAsset");
            AssetSwapRequest replacement = new AssetSwapRequest("addr/unit.prefab", "/replacement.bundle", "ReplacedAsset");

            // Act
            AssetSwapRegistry.Register(original);
            AssetSwapRegistry.Register(replacement);
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert — same address replaces previous entry, count stays 1
            AssetSwapRegistry.Count.Should().Be(1);
            pending.Should().HaveCount(1);
            pending[0].ModBundlePath.Should().Be("/replacement.bundle");
        }

        [Fact]
        public void MarkApplied_RemovesEntryFromPending()
        {
            // Arrange
            AssetSwapRequest request = new AssetSwapRequest("addr/unit.prefab", "/mod.bundle", "UnitAsset");
            AssetSwapRegistry.Register(request);
            AssetSwapRegistry.GetPending().Should().HaveCount(1);

            // Act
            AssetSwapRegistry.MarkApplied("addr/unit.prefab");
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert — applied entries are excluded from pending list
            pending.Should().BeEmpty();
            AssetSwapRegistry.Count.Should().Be(1); // still registered, just applied
        }

        [Fact]
        public void Count_ReflectsRegisteredItems()
        {
            // Arrange & Act
            AssetSwapRegistry.Count.Should().Be(0);
            AssetSwapRegistry.Register(new AssetSwapRequest("addr/a.prefab", "/mod.bundle", "A"));
            AssetSwapRegistry.Count.Should().Be(1);
            AssetSwapRegistry.Register(new AssetSwapRequest("addr/b.prefab", "/mod.bundle", "B"));
            AssetSwapRegistry.Count.Should().Be(2);
        }

        [Fact]
        public void Register_WithVanillaMapping_PreservesMapping()
        {
            // Arrange
            AssetSwapRequest request = new AssetSwapRequest(
                "addr/infantry.prefab", "/mod.bundle", "InfantryAsset", "line_infantry");

            // Act
            AssetSwapRegistry.Register(request);
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().HaveCount(1);
            pending[0].VanillaMapping.Should().Be("line_infantry");
        }

        [Fact]
        public void Register_ConcurrentFromMultipleThreads_NoneAreLost()
        {
            // Arrange — 10 threads, each registers 10 unique addresses = 100 total
            const int threadCount = 10;
            const int itemsPerThread = 10;
            int countBefore = AssetSwapRegistry.Count;

            // Act
            Parallel.For(0, threadCount, threadIndex =>
            {
                for (int j = 0; j < itemsPerThread; j++)
                {
                    string address = $"addr/thread{threadIndex}/item{j}.prefab";
                    AssetSwapRegistry.Register(new AssetSwapRequest(address, "/mod.bundle", $"Asset_{threadIndex}_{j}"));
                }
            });

            // Assert — all 100 unique registrations must be present (other tests may also have items)
            AssetSwapRegistry.Count.Should().Be(countBefore + threadCount * itemsPerThread);
        }
    }
}
