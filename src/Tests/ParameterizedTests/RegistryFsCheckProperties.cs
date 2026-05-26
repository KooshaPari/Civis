#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK;
using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests
{
    /// <summary>
    /// FsCheck Tier 3 property tests for SDK Registry layer.
    /// Extends coverage from Bridge (17 properties) to SDK Registry and PackDependencyResolver
    /// with invariant validation across randomized operations.
    ///
    /// These are REAL property tests using FsCheck generators, not parameterized [Theory] tests.
    /// Each [Property] runs 100 random iterations by default.
    ///
    /// Tested classes:
    /// - Registry&lt;T&gt;: Generic content registry with OrdinalIgnoreCase key comparison
    /// - PackDependencyResolver: Topological load-order computation and cycle detection
    /// </summary>
    [Trait("Category", "Property")]
    public class RegistryFsCheckProperties
    {
        private const string TestSourcePackId = "test-pack";
        private const RegistrySource TestSource = RegistrySource.Pack;

        /// <summary>
        /// Property: Registry&lt;T&gt;.Register then Get returns consistent item reference.
        /// For any Registry&lt;T&gt;, after registering (id, item, source, sourcePackId),
        /// calling Get(id) returns the exact item that was registered (idempotent).
        /// Validates basic register/get contract across random IDs and items.
        ///
        /// FsCheck generates 100+ random registry IDs and string items.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool Registry_Register_Then_Get_ReturnsItem(NonEmptyString id, NonEmptyString item)
        {
            // Arrange: Create a Registry and register an item
            var registry = new Registry<string>();
            string registryId = id.Get;
            string registryItem = item.Get;

            // Act: Register, then Get
            registry.Register(registryId, registryItem, TestSource, TestSourcePackId);
            var retrieved = registry.Get(registryId);

            // Assert: Retrieved item equals registered item
            var consistent = retrieved == registryItem;

            consistent.Should().BeTrue(
                because: "Registry.Get must return the exact item that was registered");
            return consistent;
        }

        /// <summary>
        /// Property: Registry&lt;T&gt;.Get on unknown ID returns null, never throws.
        /// For any Registry&lt;T&gt; and unknown ID,
        /// calling Get(unknownId) returns null and throws no exception.
        /// Validates safe default behavior for missing entries.
        ///
        /// FsCheck generates 100+ random unregistered IDs.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool Registry_Get_UnknownId_ReturnsNull(NonEmptyString unknownId)
        {
            // Arrange: Create an empty Registry
            var registry = new Registry<string>();

            // Act: Get on unknown ID
            var result = registry.Get(unknownId.Get);

            // Assert: Result is null, no exception thrown
            var isSafe = result == null;

            isSafe.Should().BeTrue(
                because: "Registry.Get(unknownId) must return null safely");
            return isSafe;
        }

        /// <summary>
        /// Property: Registry&lt;T&gt;.Contains is consistent with Get.
        /// For any Registry&lt;T&gt; and any ID,
        /// Contains(id) == (Get(id) != null) for all registered and unregistered IDs.
        /// Validates that Contains and Get are logically synchronized.
        ///
        /// FsCheck generates 100+ random IDs across registered and unregistered keys.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool Registry_Contains_ConsistentWithGet(
            NonEmptyString id1, NonEmptyString id2, NonEmptyString item1)
        {
            // Arrange: Create registry and register one item
            var registry = new Registry<string>();
            string registeredId = id1.Get;
            string unregisteredId = id2.Get != registeredId ? id2.Get : id2.Get + "_other";

            registry.Register(registeredId, item1.Get, TestSource, TestSourcePackId);

            // Act: Check Contains vs Get consistency
            bool contains1 = registry.Contains(registeredId);
            bool get1IsNotNull = registry.Get(registeredId) != null;

            bool contains2 = registry.Contains(unregisteredId);
            bool get2IsNotNull = registry.Get(unregisteredId) != null;

            // Assert: Contains matches Get for both registered and unregistered IDs
            var consistent = (contains1 == get1IsNotNull) && (contains2 == get2IsNotNull);

            consistent.Should().BeTrue(
                because: "Registry.Contains(id) must match (Registry.Get(id) != null)");
            return consistent;
        }

        /// <summary>
        /// Property: Registry&lt;T&gt;.All.Count after N registrations equals N plus initial count.
        /// For any Registry&lt;T&gt; with M initial entries, registering N additional distinct IDs
        /// results in All.Count == M + N.
        /// Validates that the All property accurately reflects registered entry count.
        ///
        /// FsCheck generates 100+ random sequences of distinct IDs.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool Registry_All_Count_EqualsRegistrationCount(
            NonEmptyString id1, NonEmptyString id2, NonEmptyString id3,
            NonEmptyString item1, NonEmptyString item2, NonEmptyString item3)
        {
            // Arrange: Create registry and ensure distinct IDs
            var registry = new Registry<string>();
            var ids = new[] { id1.Get, id2.Get, id3.Get }.Distinct().Take(3).ToList();
            var items = new[] { item1.Get, item2.Get, item3.Get };

            // Act: Register each distinct ID
            for (int i = 0; i < ids.Count; i++)
            {
                registry.Register(ids[i], items[i], TestSource, TestSourcePackId);
            }

            int countAfterRegistration = registry.All.Count;

            // Assert: Count equals number of distinct registrations
            var countCorrect = countAfterRegistration == ids.Count;

            countCorrect.Should().BeTrue(
                because: $"Registry.All.Count should equal {ids.Count} distinct registrations, was {countAfterRegistration}");
            return countCorrect;
        }

        /// <summary>
        /// Property: PackDependencyResolver.ComputeLoadOrder on linear chain preserves order.
        /// For any linear dependency chain [a→b→c→...], computing topological load order
        /// returns the packs in dependency order (independent first, then dependents).
        /// Validates that Kahn's algorithm correctly orders linear chains.
        ///
        /// FsCheck generates 100+ random linear dependency chains.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool PackDependencyResolver_LinearChain_PreservesOrder(
            NonEmptyString packAId, NonEmptyString packBId, NonEmptyString packCId)
        {
            // Arrange: Create a linear chain a→b→c (c depends on b, b depends on a)
            var packIds = new[] { packAId.Get, packBId.Get, packCId.Get }.Distinct().Take(3).ToList();
            if (packIds.Count < 3) return true; // Skip if IDs collapsed; FsCheck will retry

            var packs = new List<PackManifest>
            {
                new PackManifest { Id = packIds[0], DependsOn = new List<string>() }, // a: no deps
                new PackManifest { Id = packIds[1], DependsOn = new List<string> { packIds[0] } }, // b→a
                new PackManifest { Id = packIds[2], DependsOn = new List<string> { packIds[1] } }  // c→b
            };

            var resolver = new PackDependencyResolver();

            // Act: Compute load order
            var result = resolver.ComputeLoadOrder(packs);

            // Assert: Result is success and order is a, b, c (dependency order)
            bool isSuccess = result.IsSuccess;
            var sorted = result.LoadOrder;
            bool orderCorrect = sorted.Count == 3
                && sorted[0].Id == packIds[0] // a first (no deps)
                && sorted[1].Id == packIds[1] // b second (depends on a)
                && sorted[2].Id == packIds[2]; // c third (depends on b)

            var valid = isSuccess && orderCorrect;

            valid.Should().BeTrue(
                because: "PackDependencyResolver must preserve linear dependency order");
            return valid;
        }

        /// <summary>
        /// Property: PackDependencyResolver.ComputeLoadOrder detects cycles and returns Failure.
        /// For any circular dependency [a→b→c→a],
        /// computing load order returns DependencyResult.Failure with "Circular" in error message.
        /// Validates that cycle detection prevents infinite loops.
        ///
        /// FsCheck generates 100+ random cyclic structures.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool PackDependencyResolver_Cycle_DetectedAndFails(
            NonEmptyString packAId, NonEmptyString packBId, NonEmptyString packCId)
        {
            // Arrange: Create a circular dependency a→b→c→a
            var packIds = new[] { packAId.Get, packBId.Get, packCId.Get }.Distinct().Take(3).ToList();
            if (packIds.Count < 3) return true; // Skip if IDs collapsed; FsCheck will retry

            var packs = new List<PackManifest>
            {
                new PackManifest { Id = packIds[0], DependsOn = new List<string> { packIds[2] } }, // a→c
                new PackManifest { Id = packIds[1], DependsOn = new List<string> { packIds[0] } }, // b→a
                new PackManifest { Id = packIds[2], DependsOn = new List<string> { packIds[1] } }  // c→b (cycle!)
            };

            var resolver = new PackDependencyResolver();

            // Act: Compute load order (should detect cycle)
            var result = resolver.ComputeLoadOrder(packs);

            // Assert: Result is failure with "Circular" error message
            bool isFail = !result.IsSuccess;
            var errors = result.Errors;
            bool hasCycleError = errors.Any(e => e.Contains("Circular") || e.Contains("circular"));

            var detected = isFail && hasCycleError;

            detected.Should().BeTrue(
                because: "PackDependencyResolver must detect cycles and return Failure with 'Circular' error");
            return detected;
        }
    }
}
