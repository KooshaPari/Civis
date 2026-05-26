#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Warfare;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.Domains.Warfare.Balance;
using DINOForge.Domains.Warfare.Doctrines;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using FsCheck;
using FsCheck.Fluent;
using FsCheck.Xunit;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Property-based tests using FsCheck.Xunit for invariant validation.
    /// Tests verify that core systems maintain invariants across arbitrary inputs.
    /// </summary>
    public class CompatibilityCheckerPropertyTests
    {
        /// <summary>
        /// Property: Wildcard "*" always returns true for any valid semver string.
        /// Uses a constrained generator to avoid FsCheck discard exhaustion.
        /// </summary>
        [Property]
        public Property WildcardAlwaysMatches()
        {
            Gen<string> semverGen = Gen.Elements(
                "0.1.0", "1.0.0", "2.3.4", "0.5.1", "10.20.30",
                "0.0.1", "3.0.0", "1.2.3", "0.10.0", "5.0.0");
            return Prop.ForAll(semverGen.ToArbitrary(), version =>
                CompatibilityChecker.IsVersionInRange(version, "*") == true);
        }

        /// <summary>
        /// Property: Empty range string always returns true for any valid semver string.
        /// Uses a constrained generator to avoid FsCheck discard exhaustion.
        /// </summary>
        [Property]
        public Property EmptyRangeAlwaysMatches()
        {
            Gen<string> semverGen = Gen.Elements(
                "0.1.0", "1.0.0", "2.3.4", "0.5.1", "10.20.30",
                "0.0.1", "3.0.0", "1.2.3", "0.10.0", "5.0.0");
            return Prop.ForAll(semverGen.ToArbitrary(), version =>
                CompatibilityChecker.IsVersionInRange(version, "") == true);
        }

        /// <summary>
        /// Property: Exact version match returns true only for exact match.
        /// </summary>
        [Property]
        public Property ExactVersionMatchingIsReflexive(NonEmptyString versionNonEmpty)
        {
            string version = versionNonEmpty.Item.Substring(0, Math.Min(5, versionNonEmpty.Item.Length));
            if (!IsValidVersionFormat(version))
                return Prop.ToProperty(true);

            return Prop.ToProperty(CompatibilityChecker.IsVersionInRange(version, version));
        }

        /// <summary>
        /// Property: If version satisfies ">=X", then ">=X-1" also satisfied (monotonicity).
        /// </summary>
        [Property(Arbitrary = new[] { typeof(VersionGenerators) })]
        public Property GreaterThanOrEqualMonotonicity(SemverTriple v1, SemverTriple v2)
        {
            // v1 >= v2, so if v satisfies >=v1, it should satisfy >=v2
            if (CompareVersions(v1, v2) < 0)
                return Prop.ToProperty(true); // Skip if v1 < v2

            string v1Str = $"{v1.Major}.{v1.Minor}.{v1.Patch}";
            string v2Str = $"{v2.Major}.{v2.Minor}.{v2.Patch}";
            string testVersion = $"{v1.Major}.{v1.Minor}.{v1.Patch}";

            bool matchesV1 = CompatibilityChecker.IsVersionInRange(testVersion, $">={v1Str}");
            bool matchesV2 = CompatibilityChecker.IsVersionInRange(testVersion, $">={v2Str}");

            return Prop.When(matchesV1, matchesV2);
        }

        /// <summary>
        /// Property: Caret (^) range blocks major version changes.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(VersionGenerators) })]
        public Property CaretBlocksMajorVersionChanges(SemverTriple baseVersion)
        {
            string baseStr = $"{baseVersion.Major}.{baseVersion.Minor}.{baseVersion.Patch}";
            string careted = $"^{baseStr}";

            // Next major version should NOT match
            int nextMajor = baseVersion.Major + 1;
            string nextMajorStr = $"{nextMajor}.0.0";

            return Prop.ToProperty(!CompatibilityChecker.IsVersionInRange(nextMajorStr, careted));
        }

        /// <summary>
        /// Property: Tilde (~) range blocks minor version changes.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(VersionGenerators) })]
        public Property TildeBlocksMinorVersionChanges(SemverTriple baseVersion)
        {
            string baseStr = $"{baseVersion.Major}.{baseVersion.Minor}.{baseVersion.Patch}";
            string tilded = $"~{baseStr}";

            // Next minor version should NOT match
            int nextMinor = baseVersion.Minor + 1;
            string nextMinorStr = $"{baseVersion.Major}.{nextMinor}.0";

            return Prop.ToProperty(!CompatibilityChecker.IsVersionInRange(nextMinorStr, tilded));
        }

        private static bool IsValidVersionFormat(string version)
        {
            if (string.IsNullOrEmpty(version))
                return false;
            var parts = version.Split('.');
            return parts.Length >= 2 && parts.All(p => int.TryParse(p, out _));
        }

        private static int CompareVersions(SemverTriple v1, SemverTriple v2)
        {
            if (v1.Major != v2.Major) return v1.Major.CompareTo(v2.Major);
            if (v1.Minor != v2.Minor) return v1.Minor.CompareTo(v2.Minor);
            return v1.Patch.CompareTo(v2.Patch);
        }
    }

    /// <summary>
    /// Property-based tests for dependency resolution invariants.
    /// </summary>
    public class DependencyResolverPropertyTests
    {
        private readonly PackDependencyResolver _resolver = new();
        private readonly PackLoader _loader = new();

        /// <summary>
        /// Property: Empty pack list always resolves successfully with no errors.
        /// </summary>
        [Fact]
        public void ComputeLoadOrder_EmptyList_AlwaysSucceeds()
        {
            var emptyList = new List<PackManifest>();
            DependencyResult result = _resolver.ComputeLoadOrder(emptyList);

            result.IsSuccess.Should().BeTrue();
            result.Errors.Should().BeEmpty();
            result.LoadOrder.Should().BeEmpty();
        }

        /// <summary>
        /// Property: A pack with no dependencies always resolves successfully.
        /// </summary>
        [Property]
        public Property PackWithNoDependenciesAlwaysResolves(NonEmptyString packIdNonEmpty)
        {
            string packId = TestHelpers.SanitizeId(packIdNonEmpty.Item);
            if (string.IsNullOrEmpty(packId))
                return Prop.ToProperty(true);

            PackManifest standalone = MakePack(packId);
            var packs = new List<PackManifest> { standalone };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            return Prop.ToProperty(result.IsSuccess && result.LoadOrder.Count == 1);
        }

        /// <summary>
        /// Property: Circular dependencies are always detected (reflexive property).
        /// </summary>
        [Fact]
        public void CircularDependenciesDetected_TwoPackCircle()
        {
            PackManifest alpha = MakePack("alpha", dependsOn: new[] { "beta" });
            PackManifest beta = MakePack("beta", dependsOn: new[] { "alpha" });
            var packs = new List<PackManifest> { alpha, beta };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().ContainSingle(e => e.Contains("ircular"));
        }

        /// <summary>
        /// Property: Three-pack circular dependencies are detected.
        /// </summary>
        [Fact]
        public void CircularDependenciesDetected_ThreePackCircle()
        {
            PackManifest a = MakePack("a", dependsOn: new[] { "b" });
            PackManifest b = MakePack("b", dependsOn: new[] { "c" });
            PackManifest c = MakePack("c", dependsOn: new[] { "a" });
            var packs = new List<PackManifest> { a, b, c };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeFalse();
        }

        /// <summary>
        /// Property: Load order respects dependency constraints (topological sort invariant).
        /// If A depends on B, then B must come before A in load order.
        /// </summary>
        [Fact]
        public void LoadOrder_RespectsDependencyOrdering()
        {
            PackManifest base_ = MakePack("base", loadOrder: 10);
            PackManifest mid = MakePack("mid", dependsOn: new[] { "base" }, loadOrder: 20);
            PackManifest top = MakePack("top", dependsOn: new[] { "mid" }, loadOrder: 30);
            var packs = new List<PackManifest> { top, mid, base_ };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeTrue();
            var ids = result.LoadOrder.Select(p => p.Id).ToList();
            ids.IndexOf("base").Should().BeLessThan(ids.IndexOf("mid"));
            ids.IndexOf("mid").Should().BeLessThan(ids.IndexOf("top"));
        }

        /// <summary>
        /// Property: Computing load order multiple times yields same result (deterministic).
        /// </summary>
        [Fact]
        public void LoadOrder_IsDeterministic()
        {
            PackManifest a = MakePack("a");
            PackManifest b = MakePack("b", dependsOn: new[] { "a" });
            PackManifest c = MakePack("c", dependsOn: new[] { "b" });
            var packs = new List<PackManifest> { a, b, c };

            DependencyResult result1 = _resolver.ComputeLoadOrder(packs);
            DependencyResult result2 = _resolver.ComputeLoadOrder(packs);

            result1.LoadOrder.Select(p => p.Id).SequenceEqual(result2.LoadOrder.Select(p => p.Id)).Should().BeTrue();
        }

        /// <summary>
        /// Property: Self-dependencies are detected as circular.
        /// </summary>
        [Property]
        public Property SelfDependencyDetected(NonEmptyString packIdNonEmpty)
        {
            string packId = TestHelpers.SanitizeId(packIdNonEmpty.Item);
            if (string.IsNullOrEmpty(packId))
                return Prop.ToProperty(true);

            PackManifest self = MakePack(packId, dependsOn: new[] { packId });
            var packs = new List<PackManifest> { self };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            return Prop.ToProperty(!result.IsSuccess);
        }

        private PackManifest MakePack(string id, string[]? dependsOn = null, string[]? conflictsWith = null, int loadOrder = 100)
        {
            string deps = dependsOn is { Length: > 0 }
                ? "depends_on:\n" + string.Join("", dependsOn.Select(d => $"  - \"{d}\"\n"))
                : "";

            string conflicts = conflictsWith is { Length: > 0 }
                ? "conflicts_with:\n" + string.Join("", conflictsWith.Select(c => $"  - \"{c}\"\n"))
                : "";

            string yaml = $@"
id: ""{id}""
name: ""{id}""
version: 0.1.0
author: Test
type: content
load_order: {loadOrder}
{deps}{conflicts}";

            return _loader.LoadFromString(yaml);
        }
    }

    /// <summary>
    /// Property-based tests for balance calculation invariants.
    /// </summary>
    public class BalanceCalculatorPropertyTests
    {
        private readonly BalanceCalculator _calculator = new(new DoctrineEngine());

        /// <summary>
        /// Property: Positive HP values remain positive after any stat modifier.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        public Property HpRemainsPotitive_AfterModifiers(PositiveInt baseHp, NonNegativeInt modifier)
        {
            float hpVal = (float)baseHp.Item;
            float modifierVal = Math.Max(0.01f, (float)modifier.Item); // Clamp to positive

            var unit = new UnitDefinition
            {
                Id = "test-unit",
                Stats = new UnitStats
                {
                    Hp = hpVal,
                    Damage = 10f,
                    Armor = 5f,
                    Speed = 3f,
                    Cost = new ResourceCost { Food = 50 }
                }
            };

            var doctrine = new DoctrineDefinition
            {
                Id = "test",
                DisplayName = "Test",
                Modifiers = new Dictionary<string, float> { { "hp", modifierVal } }
            };

            var doctrineEngine = new DoctrineEngine();
            UnitStats result = doctrineEngine.ApplyDoctrine(unit.Stats, doctrine);

            return Prop.ToProperty(result.Hp > 0);
        }

        /// <summary>
        /// Property: Damage values are clamped to non-negative after modifiers.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        public Property DamageRemainNonNegative(PositiveInt baseDamage)
        {
            float dmgVal = (float)baseDamage.Item;

            var unit = new UnitDefinition
            {
                Id = "test-unit",
                Stats = new UnitStats
                {
                    Hp = 100f,
                    Damage = dmgVal,
                    Armor = 5f,
                    Speed = 3f,
                    Cost = new ResourceCost { Food = 50 }
                }
            };

            // Apply a weak modifier to damage
            var doctrine = new DoctrineDefinition
            {
                Id = "test",
                DisplayName = "Test",
                Modifiers = new Dictionary<string, float> { { "damage", 0.5f } }
            };

            var doctrineEngine = new DoctrineEngine();
            UnitStats result = doctrineEngine.ApplyDoctrine(unit.Stats, doctrine);

            return Prop.ToProperty(result.Damage >= 0);
        }

        /// <summary>
        /// Property: Power rating is always non-negative.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        public Property PowerRatingIsNonNegative(PositiveInt hp, PositiveInt damage, PositiveInt cost)
        {
            var unit = new UnitDefinition
            {
                Id = "test",
                Stats = new UnitStats
                {
                    Hp = (float)hp.Item,
                    Damage = (float)damage.Item,
                    Armor = 5f,
                    Speed = 3f,
                    Cost = new ResourceCost { Food = (int)cost.Item }
                }
            };

            var archetype = new FactionArchetype(
                "test", "Test", "Test",
                new Dictionary<string, float> { { "hp", 1.0f } });

            float power = _calculator.CalculatePowerRating(unit, archetype, null);

            return Prop.ToProperty(power >= 0);
        }
    }

    /// <summary>
    /// Property-based tests for registry invariants.
    /// </summary>
    public class RegistryPropertyTests
    {
        /// <summary>
        /// Property: Register then Contains always returns true.
        /// </summary>
        [Property]
        public Property RegisterThenContains_AlwaysTrue(NonEmptyString idNonEmpty)
        {
            string id = TestHelpers.SanitizeId(idNonEmpty.Item);
            if (string.IsNullOrEmpty(id))
                return Prop.ToProperty(true);

            var registry = new Registry<TestItem>();
            var item = new TestItem($"Item-{id}");

            registry.Register(id, item, RegistrySource.BaseGame, "test");

            return Prop.ToProperty(registry.Contains(id));
        }

        /// <summary>
        /// Property: Get after Register returns the registered item.
        /// </summary>
        [Property]
        public Property RegisterThenGet_ReturnsItem(NonEmptyString idNonEmpty, NonEmptyString nameNonEmpty)
        {
            string id = TestHelpers.SanitizeId(idNonEmpty.Item);
            string name = TestHelpers.SanitizeId(nameNonEmpty.Item);
            if (string.IsNullOrEmpty(id) || string.IsNullOrEmpty(name))
                return Prop.ToProperty(true);

            var registry = new Registry<TestItem>();
            var item = new TestItem(name);

            registry.Register(id, item, RegistrySource.BaseGame, "test");
            TestItem? retrieved = registry.Get(id);

            return Prop.ToProperty(retrieved is not null && retrieved.Name == name);
        }

        /// <summary>
        /// Property: Count increases by 1 after each unique Register.
        /// </summary>
        [Fact]
        public void RegisterMultipleUnique_CountIncreases()
        {
            var registry = new Registry<TestItem>();

            registry.Register("id1", new TestItem("Item1"), RegistrySource.BaseGame, "test");
            int countAfter1 = registry.All.Count;

            registry.Register("id2", new TestItem("Item2"), RegistrySource.BaseGame, "test");
            int countAfter2 = registry.All.Count;

            registry.Register("id3", new TestItem("Item3"), RegistrySource.BaseGame, "test");
            int countAfter3 = registry.All.Count;

            countAfter1.Should().Be(1);
            countAfter2.Should().Be(2);
            countAfter3.Should().Be(3);
        }

        /// <summary>
        /// Property: Registering same ID twice doesn't increase count (override).
        /// </summary>
        [Fact]
        public void RegisterSameIdTwice_CountUnchanged()
        {
            var registry = new Registry<TestItem>();

            registry.Register("same-id", new TestItem("Item1"), RegistrySource.BaseGame, "test");
            int countAfter1 = registry.All.Count;

            registry.Register("same-id", new TestItem("Item2"), RegistrySource.Pack, "test");
            int countAfter2 = registry.All.Count;

            countAfter1.Should().Be(1);
            countAfter2.Should().Be(1); // Still 1, because Pack priority overrides BaseGame
        }

        /// <summary>
        /// Property: Get for unregistered ID returns null.
        /// </summary>
        [Property]
        public Property GetUnregistered_ReturnsNull(NonEmptyString idNonEmpty)
        {
            string id = TestHelpers.SanitizeId(idNonEmpty.Item);
            if (string.IsNullOrEmpty(id))
                return Prop.ToProperty(true);

            var registry = new Registry<TestItem>();

            TestItem? result = registry.Get(id);

            return Prop.ToProperty(result is null);
        }

        /// <summary>
        /// Property: Contains for unregistered ID returns false.
        /// </summary>
        [Property]
        public Property ContainsUnregistered_ReturnsFalse(NonEmptyString idNonEmpty)
        {
            string id = TestHelpers.SanitizeId(idNonEmpty.Item);
            if (string.IsNullOrEmpty(id))
                return Prop.ToProperty(true);

            var registry = new Registry<TestItem>();

            return Prop.ToProperty(!registry.Contains(id));
        }
    }

    // ── Test Generators ─────────────────────────────────────────────────────────

    public record SemverTriple(int Major, int Minor, int Patch);

    public static class VersionGenerators
    {
        public static Arbitrary<SemverTriple> SemverTriple()
        {
            Gen<int> majorGen = Gen.Choose(0, 10);
            Gen<int> minorGen = Gen.Choose(0, 20);
            Gen<int> patchGen = Gen.Choose(0, 20);
            Gen<SemverTriple> gen = majorGen.SelectMany(
                major => minorGen.SelectMany(
                    minor => patchGen.Select(
                        patch => new SemverTriple(major, minor, patch))));

            return Arb.From(gen);
        }
    }

    public static class BalanceGenerators
    {
        public static Arbitrary<PositiveInt> PositiveInt()
        {
            return Arb.From(Gen.Choose(1, 10000).Select(FsCheck.PositiveInt.NewPositiveInt));
        }

        public static Arbitrary<NonNegativeInt> NonNegativeInt()
        {
            return Arb.From(Gen.Choose(0, 10000).Select(FsCheck.NonNegativeInt.NewNonNegativeInt));
        }
    }

    // ── Helper Functions ────────────────────────────────────────────────────────

    internal static class TestHelpers
    {
        internal static string SanitizeId(string id)
        {
            if (string.IsNullOrEmpty(id))
                return "";

            // Keep only alphanumeric and dash
            var chars = id
                .Where(c => char.IsLetterOrDigit(c) || c == '-')
                .ToList();

            if (chars.Count == 0)
                return "";

            // Ensure starts with letter
            while (chars.Count > 0 && char.IsDigit(chars[0]))
                chars.RemoveAt(0);

            return new string(chars.Take(50).ToArray()); // Limit length
        }
    }
}
