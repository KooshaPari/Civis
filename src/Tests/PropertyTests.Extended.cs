#nullable enable

using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Domains.Warfare.Balance;
using DINOForge.Domains.Warfare.Doctrines;
using DINOForge.SDK;
using DINOForge.SDK.Assets;
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
    // ── Shared generators for extended property tests ────────────────────────

    /// <summary>
    /// Generators for valid pack IDs and semver strings.
    /// </summary>
    public static class PackGenerators
    {
        private static readonly string[] ValidIds =
        {
            "alpha", "beta", "gamma", "warfare-test", "economy-mod",
            "scenario-1", "base-pack", "mod-core", "test-pack", "dlc-content"
        };

        private static readonly string[] ValidWithUnderscore =
        {
            "alpha_mod", "beta_pack", "warfare_test", "economy_mod",
            "scenario_one", "base_pack", "mod_core", "test_pack"
        };

        /// <summary>Generates valid pack ID strings (lowercase alphanumeric + dash).</summary>
        public static Arbitrary<string> ValidPackId() =>
            Arb.From(Gen.Elements(ValidIds));

        /// <summary>Generates valid pack IDs including underscores.</summary>
        public static Arbitrary<string> ValidPackIdWithUnderscore() =>
            Arb.From(Gen.Elements(ValidWithUnderscore));

        /// <summary>Generates structurally valid semver strings.</summary>
        public static Arbitrary<string> ValidSemver() =>
            Arb.From(
                from major in Gen.Choose(0, 99)
                from minor in Gen.Choose(0, 99)
                from patch in Gen.Choose(0, 99)
                select $"{major}.{minor}.{patch}");

        /// <summary>Generates semver strings guaranteed to be >= 0.1.0.</summary>
        public static Arbitrary<string> SemverGe010() =>
            Arb.From(
                from major in Gen.Choose(0, 9)
                from minor in Gen.Choose(0, 9)
                from patch in Gen.Choose(0, 9)
                let effective = major == 0 && minor == 0 ? $"0.1.{patch + 1}" : $"{major}.{minor}.{patch}"
                select effective);
    }

    // ── Domain 1: PackManifest / Pack ID validation ──────────────────────────

    /// <summary>
    /// Property-based tests for PackManifest and pack ID validation invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class PackIdValidationPropertyTests
    {
        private static readonly Regex ValidPackIdPattern = new Regex(@"^[a-z0-9][a-z0-9\-_]*$", RegexOptions.Compiled);

        /// <summary>
        /// Property: Pack IDs containing spaces are always invalid (fail the id pattern).
        /// </summary>
        [Property]
        [Trait("Category", "Property")]
        public Property PackIdWithSpaces_AlwaysInvalid(NonEmptyString prefix, NonEmptyString suffix)
        {
            string withSpace = $"{prefix.Item.ToLower()} {suffix.Item.ToLower()}";
            bool isValid = ValidPackIdPattern.IsMatch(withSpace);
            return Prop.ToProperty(!isValid);
        }

        /// <summary>
        /// Property: A pack ID consisting only of [a-z0-9-] characters and starting with a
        /// letter/digit is always accepted by the valid-id regex.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property ValidPackId_MatchesPattern(string packId)
        {
            return Prop.ToProperty(ValidPackIdPattern.IsMatch(packId));
        }

        /// <summary>
        /// Property: Empty string is never a valid pack ID.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void EmptyPackId_AlwaysInvalid()
        {
            bool isValid = ValidPackIdPattern.IsMatch(string.Empty);
            isValid.Should().BeFalse();
        }

        /// <summary>
        /// Property: A manifest built with empty DependsOn list never produces
        /// dependency resolver errors when standing alone.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property ManifestWithNoDeps_NeverCausesResolverError(string packId)
        {
            PackManifest manifest = BuildManifest(packId, version: "1.0.0");
            PackDependencyResolver resolver = new PackDependencyResolver();
            DependencyResult result = resolver.ComputeLoadOrder(new[] { manifest });
            return Prop.ToProperty(result.IsSuccess);
        }

        /// <summary>
        /// Property: A structurally valid semver version string (X.Y.Z) parses correctly
        /// via System.Version. Uses a dedicated semver generator to avoid pack-ID strings.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(ValidSemverGenerators) })]
        [Trait("Category", "Property")]
        public Property ValidSemver_ParsesCorrectly(string semver)
        {
            bool parsed = System.Version.TryParse(semver, out _);
            return Prop.ToProperty(parsed);
        }

        private static PackManifest BuildManifest(string id, string version = "0.1.0") =>
            new PackManifest
            {
                Id = id,
                Name = id,
                Version = version,
                DependsOn = new List<string>(),
                ConflictsWith = new List<string>()
            };
    }

    // ── Domain 2: AssetSwapRegistry thread-safety ────────────────────────────

    /// <summary>
    /// Property-based tests for AssetSwapRegistry concurrent-access invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class AssetSwapRegistryPropertyTests : IDisposable
    {
        public void Dispose()
        {
            // AssetSwapRegistry.Clear() is internal; access via reflection for test teardown.
            typeof(AssetSwapRegistry)
                .GetMethod("Clear", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)
                ?.Invoke(null, null);
        }

        /// <summary>
        /// Property: Concurrent Register calls from N threads never lose registrations.
        /// Count after all threads complete equals number of unique addresses registered.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void ConcurrentRegister_NeverLosesRegistrations()
        {
            // Clear state
            typeof(AssetSwapRegistry)
                .GetMethod("Clear", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)
                ?.Invoke(null, null);

            const int threadCount = 20;
            const int registrationsPerThread = 5;
            CountdownEvent countdown = new CountdownEvent(threadCount);
            Thread[] threads = new Thread[threadCount];

            for (int t = 0; t < threadCount; t++)
            {
                int threadIndex = t;
                threads[t] = new Thread(() =>
                {
                    countdown.Signal();
                    countdown.Wait(); // all threads start simultaneously
                    for (int i = 0; i < registrationsPerThread; i++)
                    {
                        string addr = $"thread-{threadIndex}-asset-{i}";
                        AssetSwapRegistry.Register(new AssetSwapRequest(addr, "/fake/path.bundle", addr));
                    }
                });
                threads[t].Start();
            }

            foreach (Thread thread in threads)
                thread.Join();

            // Each thread registers registrationsPerThread unique addresses => total unique = threadCount * registrationsPerThread
            AssetSwapRegistry.Count.Should().Be(threadCount * registrationsPerThread);
        }

        /// <summary>
        /// Property: After registering N distinct addresses, GetPending returns all N
        /// (none lost, none duplicated) when no items have been marked applied.
        /// </summary>
        [Theory]
        [Trait("Category", "Property")]
        [InlineData(1)]
        [InlineData(5)]
        [InlineData(50)]
        public void GetPending_ReturnsAllRegisteredItems(int count)
        {
            typeof(AssetSwapRegistry)
                .GetMethod("Clear", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)
                ?.Invoke(null, null);

            for (int i = 0; i < count; i++)
            {
                AssetSwapRegistry.Register(new AssetSwapRequest($"asset-{i}", "/fake/path.bundle", $"asset-{i}"));
            }

            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();
            pending.Count.Should().Be(count);
        }

        /// <summary>
        /// Property: MarkApplied transitions items from pending to applied; they no longer appear
        /// in GetPending but the total Count remains unchanged.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void MarkApplied_RemovesFromPendingButPreservesCount()
        {
            typeof(AssetSwapRegistry)
                .GetMethod("Clear", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)
                ?.Invoke(null, null);

            AssetSwapRegistry.Register(new AssetSwapRequest("addr-a", "/fake/a.bundle", "addr-a"));
            AssetSwapRegistry.Register(new AssetSwapRequest("addr-b", "/fake/b.bundle", "addr-b"));

            int totalBefore = AssetSwapRegistry.Count;
            AssetSwapRegistry.MarkApplied("addr-a");

            AssetSwapRegistry.Count.Should().Be(totalBefore); // total unchanged
            AssetSwapRegistry.GetPending().Should().HaveCount(1); // only addr-b pending
            AssetSwapRegistry.GetPending()[0].AssetAddress.Should().Be("addr-b");
        }

        /// <summary>
        /// Property: Registering the same address twice replaces the first registration
        /// (count stays at 1, not 2).
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property Register_SameAddressTwice_ReplacesPrevious(string packId)
        {
            typeof(AssetSwapRegistry)
                .GetMethod("Clear", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)
                ?.Invoke(null, null);

            string addr = $"addr-{packId}";
            AssetSwapRegistry.Register(new AssetSwapRequest(addr, "/fake/v1.bundle", addr));
            AssetSwapRegistry.Register(new AssetSwapRequest(addr, "/fake/v2.bundle", addr));

            return Prop.ToProperty(AssetSwapRegistry.Count == 1);
        }
    }

    // ── Domain 3: DependencyResolver extended ───────────────────────────────

    /// <summary>
    /// Extended property-based tests for DependencyResolver invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class DependencyResolverExtendedPropertyTests
    {
        private readonly PackDependencyResolver _resolver = new PackDependencyResolver();
        private readonly PackLoader _loader = new PackLoader();

        /// <summary>
        /// Property: A pack that depends on itself is always detected as a cycle.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property SelfDependency_IsAlwaysACycle(string packId)
        {
            PackManifest self = MakePack(packId, dependsOn: new[] { packId });
            DependencyResult result = _resolver.ComputeLoadOrder(new[] { self });
            return Prop.ToProperty(!result.IsSuccess);
        }

        /// <summary>
        /// Property: Resolution of N independent packs (no deps between them) always succeeds.
        /// </summary>
        [Property]
        [Trait("Category", "Property")]
        public Property NIndependentPacks_AlwaysResolve(PositiveInt countWrapped)
        {
            int count = Math.Min(countWrapped.Item, 20); // cap at 20 to keep tests fast
            List<PackManifest> packs = Enumerable
                .Range(0, count)
                .Select(i => MakePack($"pack-{i}"))
                .ToList();

            DependencyResult result = _resolver.ComputeLoadOrder(packs);
            return Prop.ToProperty(result.IsSuccess && result.LoadOrder.Count == count);
        }

        /// <summary>
        /// Property: Adding a satisfied dependency (dep exists in the set) never turns
        /// an otherwise successful resolution into a failure.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property AddingSatisfiedDep_DoesNotBreakResolution(string baseId, string depId)
        {
            if (string.Equals(baseId, depId, StringComparison.OrdinalIgnoreCase))
                return Prop.ToProperty(true); // skip degenerate case

            PackManifest dep = MakePack(depId);
            PackManifest withDep = MakePack(baseId, dependsOn: new[] { depId });

            DependencyResult result = _resolver.ComputeLoadOrder(new[] { dep, withDep });
            return Prop.ToProperty(result.IsSuccess);
        }

        /// <summary>
        /// Property: Load order for a chain A→B→C always places A before B before C.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void LinearChain_LoadOrderIsTopological()
        {
            PackManifest a = MakePack("chain-a");
            PackManifest b = MakePack("chain-b", dependsOn: new[] { "chain-a" });
            PackManifest c = MakePack("chain-c", dependsOn: new[] { "chain-b" });

            DependencyResult result = _resolver.ComputeLoadOrder(new[] { c, b, a });

            result.IsSuccess.Should().BeTrue();
            List<string> ids = result.LoadOrder.Select(p => p.Id).ToList();
            ids.IndexOf("chain-a").Should().BeLessThan(ids.IndexOf("chain-b"));
            ids.IndexOf("chain-b").Should().BeLessThan(ids.IndexOf("chain-c"));
        }

        /// <summary>
        /// Property: ComputeLoadOrder is deterministic — calling it twice on the same
        /// input yields the same load order.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property LoadOrder_IsDeterministic(string id1, string id2)
        {
            if (string.Equals(id1, id2, StringComparison.OrdinalIgnoreCase))
                return Prop.ToProperty(true);

            List<PackManifest> packs = new List<PackManifest> { MakePack(id1), MakePack(id2) };
            DependencyResult r1 = _resolver.ComputeLoadOrder(packs);
            DependencyResult r2 = _resolver.ComputeLoadOrder(packs);

            bool same = r1.IsSuccess == r2.IsSuccess &&
                        r1.LoadOrder.Select(p => p.Id)
                         .SequenceEqual(r2.LoadOrder.Select(p => p.Id), StringComparer.OrdinalIgnoreCase);
            return Prop.ToProperty(same);
        }

        private PackManifest MakePack(string id, string[]? dependsOn = null, int loadOrder = 100)
        {
            string deps = dependsOn is { Length: > 0 }
                ? "depends_on:\n" + string.Join("", dependsOn.Select(d => $"  - \"{d}\"\n"))
                : "";

            string yaml = $@"
id: ""{id}""
name: ""{id}""
version: 0.1.0
author: Test
type: content
load_order: {loadOrder}
{deps}";
            return _loader.LoadFromString(yaml);
        }
    }

    // ── Domain 4: Schema version compatibility ───────────────────────────────

    /// <summary>
    /// Dedicated generator for structurally valid semver strings (X.Y.Z).
    /// Kept separate from PackGenerators to avoid Arbitrary-type ambiguity in FsCheck.
    /// </summary>
    public static class ValidSemverGenerators
    {
        /// <summary>Generates X.Y.Z strings that are parseable by System.Version.</summary>
        public static Arbitrary<string> SemverString() =>
            Arb.From(
                from major in Gen.Choose(0, 9)
                from minor in Gen.Choose(0, 9)
                from patch in Gen.Choose(0, 9)
                select $"{major}.{minor}.{patch}");
    }

    /// <summary>
    /// Property-based tests for CompatibilityChecker version range invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class SchemaVersionCompatibilityPropertyTests
    {
        /// <summary>
        /// Property: ">=0.0.0" always accepts any X.Y.Z semver version.
        /// Uses a dedicated semver generator to ensure well-formed inputs.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(ValidSemverGenerators) })]
        [Trait("Category", "Property")]
        public Property GeConstraint_AcceptsMatchingAndHigherVersions(string semver)
        {
            bool result = CompatibilityChecker.IsVersionInRange(semver, ">=0.0.0");
            return Prop.ToProperty(result);
        }

        /// <summary>
        /// Property: ">=0.1.0" accepts 0.1.0 exactly.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void GeConstraint_Accepts_ExactLowerBound()
        {
            CompatibilityChecker.IsVersionInRange("0.1.0", ">=0.1.0").Should().BeTrue();
        }

        /// <summary>
        /// Property: ">=1.0.0 <2.0.0" rejects 2.0.0 and above.
        /// </summary>
        [Theory]
        [Trait("Category", "Property")]
        [InlineData("2.0.0")]
        [InlineData("2.1.0")]
        [InlineData("3.0.0")]
        [InlineData("10.0.0")]
        public void BoundedRange_Rejects_VersionAtOrAboveUpperBound(string version)
        {
            CompatibilityChecker.IsVersionInRange(version, ">=1.0.0 <2.0.0").Should().BeFalse();
        }

        /// <summary>
        /// Property: ">=1.0.0 <2.0.0" accepts versions in [1.0.0, 2.0.0).
        /// </summary>
        [Theory]
        [Trait("Category", "Property")]
        [InlineData("1.0.0")]
        [InlineData("1.5.0")]
        [InlineData("1.99.99")]
        public void BoundedRange_Accepts_VersionsInRange(string version)
        {
            CompatibilityChecker.IsVersionInRange(version, ">=1.0.0 <2.0.0").Should().BeTrue();
        }

        /// <summary>
        /// Property: A pack with a higher framework version requirement than the supplied
        /// framework version yields an incompatibility error.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void HigherFrameworkRequirement_RejectsLowerFramework()
        {
            // Framework is 0.1.0; pack requires >=99.0.0 — should produce error.
            bool inRange = CompatibilityChecker.IsVersionInRange("0.1.0", ">=99.0.0");
            inRange.Should().BeFalse();
        }

        /// <summary>
        /// Property: Wildcard (*) always matches any valid X.Y.Z semver.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(ValidSemverGenerators) })]
        [Trait("Category", "Property")]
        public Property Wildcard_AlwaysMatches(string semver)
        {
            return Prop.ToProperty(CompatibilityChecker.IsVersionInRange(semver, "*"));
        }

        /// <summary>
        /// Property: A version is always compatible with itself as an exact constraint.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(ValidSemverGenerators) })]
        [Trait("Category", "Property")]
        public Property Version_IsCompatibleWithItself(string semver)
        {
            return Prop.ToProperty(CompatibilityChecker.IsVersionInRange(semver, semver));
        }
    }

    // ── Domain 5: Stat modifier combinatorics ───────────────────────────────

    /// <summary>
    /// Property-based tests for stat modifier and doctrine engine combinatorics.
    /// </summary>
    [Trait("Category", "Property")]
    public class StatModifierPropertyTests
    {
        private readonly DoctrineEngine _engine = new DoctrineEngine();

        /// <summary>
        /// Property: Applying a +0% multiplier (modifier value 0) to any unit's HP returns
        /// the original HP unchanged (identity modifier).
        /// Note: DoctrineEngine multiplies by (1 + modifier). When modifier=0, result = original.
        /// Actually the engine uses the modifier as a direct multiplier, so 1.0 = identity.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        [Trait("Category", "Property")]
        public Property IdentityModifier_LeavesStatUnchanged(PositiveInt baseHp)
        {
            float originalHp = (float)baseHp.Item;
            UnitStats stats = MakeStats(hp: originalHp);

            // modifier = 1.0 means "multiply by 1.0" — identity
            DoctrineDefinition doctrine = MakeDoctrine("hp", 1.0f);
            UnitStats result = _engine.ApplyDoctrine(stats, doctrine);

            return Prop.ToProperty(Math.Abs(result.Hp - originalHp) < 0.001f);
        }

        /// <summary>
        /// Property: HP after applying any positive multiplier modifier remains positive.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        [Trait("Category", "Property")]
        public Property PositiveModifier_KeepsHpPositive(PositiveInt baseHp, PositiveInt modifierHundredths)
        {
            float hp = (float)baseHp.Item;
            float modifier = Math.Max(0.01f, (float)modifierHundredths.Item / 100f);
            UnitStats stats = MakeStats(hp: hp);
            DoctrineDefinition doctrine = MakeDoctrine("hp", modifier);
            UnitStats result = _engine.ApplyDoctrine(stats, doctrine);
            return Prop.ToProperty(result.Hp > 0);
        }

        /// <summary>
        /// Property: Stat clamped to non-negative never goes below 0 after applying
        /// any modifier between 0 and 1 (reduction).
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        [Trait("Category", "Property")]
        public Property ReducingModifier_DamageRemainsNonNegative(PositiveInt baseDamage)
        {
            float damage = (float)baseDamage.Item;
            UnitStats stats = MakeStats(damage: damage);

            // 0.5 = reduce to 50%
            DoctrineDefinition doctrine = MakeDoctrine("damage", 0.5f);
            UnitStats result = _engine.ApplyDoctrine(stats, doctrine);

            return Prop.ToProperty(result.Damage >= 0);
        }

        /// <summary>
        /// Property: Applying a doctrine with modifier > 1 always increases a positive stat.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        [Trait("Category", "Property")]
        public Property AmplifyingModifier_IncreasesPositiveStat(PositiveInt baseHp)
        {
            float hp = (float)baseHp.Item;
            UnitStats stats = MakeStats(hp: hp);
            DoctrineDefinition doctrine = MakeDoctrine("hp", 2.0f); // double HP
            UnitStats result = _engine.ApplyDoctrine(stats, doctrine);
            return Prop.ToProperty(result.Hp >= hp);
        }

        /// <summary>
        /// Property: Power rating is always non-negative for any valid unit definition.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(BalanceGenerators) })]
        [Trait("Category", "Property")]
        public Property PowerRating_AlwaysNonNegative(PositiveInt hp, PositiveInt damage)
        {
            UnitDefinition unit = new UnitDefinition
            {
                Id = "test-unit",
                Stats = MakeStats(hp: (float)hp.Item, damage: (float)damage.Item)
            };

            DINOForge.Domains.Warfare.Archetypes.FactionArchetype archetype =
                new DINOForge.Domains.Warfare.Archetypes.FactionArchetype(
                    "test", "Test", "Test",
                    new Dictionary<string, float> { { "hp", 1.0f } });

            BalanceCalculator calc = new BalanceCalculator(_engine);
            float power = calc.CalculatePowerRating(unit, archetype, null);
            return Prop.ToProperty(power >= 0);
        }

        private static UnitStats MakeStats(float hp = 100f, float damage = 10f) =>
            new UnitStats
            {
                Hp = hp,
                Damage = damage,
                Armor = 5f,
                Speed = 3f,
                Cost = new ResourceCost { Food = 50 }
            };

        private static DoctrineDefinition MakeDoctrine(string statKey, float value) =>
            new DoctrineDefinition
            {
                Id = "test-doctrine",
                DisplayName = "Test",
                Modifiers = new Dictionary<string, float> { { statKey, value } }
            };
    }

    // ── Domain 6: Registry generic extended ─────────────────────────────────

    /// <summary>
    /// Extended property-based tests for the generic Registry invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class RegistryExtendedPropertyTests
    {
        /// <summary>
        /// Property: Registry with Pack source always wins over BaseGame source for the same ID.
        /// </summary>
        [Property]
        [Trait("Category", "Property")]
        public Property HigherPrioritySource_WinsConflict(NonEmptyString idNonEmpty)
        {
            string id = TestHelpers.SanitizeId(idNonEmpty.Item);
            if (string.IsNullOrEmpty(id))
                return Prop.ToProperty(true);

            Registry<TestItem> registry = new Registry<TestItem>();
            registry.Register(id, new TestItem("base"), RegistrySource.BaseGame, "base-pack");
            registry.Register(id, new TestItem("pack"), RegistrySource.Pack, "mod-pack");

            TestItem? result = registry.Get(id);
            return Prop.ToProperty(result?.Name == "pack");
        }

        /// <summary>
        /// Property: Attempting to get a key that was never registered returns null (no exception).
        /// </summary>
        [Property]
        [Trait("Category", "Property")]
        public Property GetNonExistentKey_ReturnsNullNoException(NonEmptyString idNonEmpty)
        {
            string id = TestHelpers.SanitizeId(idNonEmpty.Item);
            if (string.IsNullOrEmpty(id))
                return Prop.ToProperty(true);

            Registry<TestItem> registry = new Registry<TestItem>();

            TestItem? result = null;
            Exception? ex = null;
            try { result = registry.Get("definitely-not-there-" + id); }
            catch (Exception e) { ex = e; }

            return Prop.ToProperty(ex == null && result == null);
        }

        /// <summary>
        /// Property: Calling DetectConflicts on a registry with one entry per ID
        /// always returns empty conflict list.
        /// </summary>
        [Property]
        [Trait("Category", "Property")]
        public Property SingleEntryPerKey_NoConflicts(PositiveInt countWrapped)
        {
            int count = Math.Min(countWrapped.Item, 50);
            Registry<TestItem> registry = new Registry<TestItem>();
            for (int i = 0; i < count; i++)
                registry.Register($"item-{i}", new TestItem($"Item {i}"), RegistrySource.BaseGame, "base");

            IReadOnlyList<RegistryConflict> conflicts = registry.DetectConflicts();
            return Prop.ToProperty(conflicts.Count == 0);
        }

        /// <summary>
        /// Property: Registering the same ID with equal priority from two different packs
        /// produces exactly one conflict.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void EqualPriorityDuplicate_ProducesOneConflict()
        {
            Registry<TestItem> registry = new Registry<TestItem>();
            registry.Register("conflict-id", new TestItem("A"), RegistrySource.Pack, "pack-a", 100);
            registry.Register("conflict-id", new TestItem("B"), RegistrySource.Pack, "pack-b", 100);

            IReadOnlyList<RegistryConflict> conflicts = registry.DetectConflicts();
            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("conflict-id");
        }

        /// <summary>
        /// Property: All returns the highest-priority entry for each key.
        /// </summary>
        [Fact]
        [Trait("Category", "Property")]
        public void All_ReturnsHighestPriorityEntry()
        {
            Registry<TestItem> registry = new Registry<TestItem>();
            registry.Register("item", new TestItem("base"), RegistrySource.BaseGame, "base");
            registry.Register("item", new TestItem("framework"), RegistrySource.Framework, "fw");
            registry.Register("item", new TestItem("pack"), RegistrySource.Pack, "mod");

            IReadOnlyDictionary<string, RegistryEntry<TestItem>> all = registry.All;
            all["item"].Data.Name.Should().Be("pack");
        }
    }

    // ── Domain 7: ContentLoader round-trip ──────────────────────────────────

    /// <summary>
    /// Property-based tests for ContentLoader and PackLoader round-trip invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class ContentLoaderRoundTripPropertyTests
    {
        private readonly PackLoader _loader = new PackLoader();

        /// <summary>
        /// Property: A PackManifest built from YAML always has the same ID as specified.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property YamlRoundTrip_PreservesId(string packId)
        {
            string yaml = $@"
id: ""{packId}""
name: ""{packId}""
version: 1.0.0
author: Test
type: content";
            PackManifest manifest = _loader.LoadFromString(yaml);
            return Prop.ToProperty(manifest.Id == packId);
        }

        /// <summary>
        /// Property: A PackManifest built from YAML always has the correct version.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property YamlRoundTrip_PreservesVersion(string semver)
        {
            string yaml = $@"
id: ""test-pack""
name: ""Test Pack""
version: {semver}
author: Test
type: content";
            PackManifest manifest = _loader.LoadFromString(yaml);
            return Prop.ToProperty(manifest.Version == semver);
        }

        /// <summary>
        /// Property: LoadFromString on a structurally valid pack YAML (non-empty id/name/version)
        /// never throws an unexpected exception — only YamlException or InvalidOperationException
        /// are acceptable (both expected parser/validation paths).
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property ValidPackYaml_OnlyThrowsExpectedExceptions(string packId, string version)
        {
            string yaml = $@"
id: ""{packId}""
name: ""Test Pack""
version: {version}
author: DINOForge
type: content
depends_on: []
conflicts_with: []";

            Exception? unexpectedEx = null;
            try { _loader.LoadFromString(yaml); }
            catch (YamlDotNet.Core.YamlException) { /* expected — YAML parse error */ }
            catch (InvalidOperationException) { /* expected — PackLoader validation */ }
            catch (Exception e) { unexpectedEx = e; }

            return Prop.ToProperty(unexpectedEx == null);
        }

        /// <summary>
        /// Property: A pack with DependsOn set to empty list always has empty DependsOn
        /// after deserialization.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property EmptyDependsOn_RoundTrips(string packId)
        {
            string yaml = $@"
id: ""{packId}""
name: ""{packId}""
version: 0.1.0
author: Test
type: content
depends_on: []";
            PackManifest manifest = _loader.LoadFromString(yaml);
            return Prop.ToProperty(manifest.DependsOn.Count == 0);
        }
    }

    // ── Domain 8: Conflict detection ─────────────────────────────────────────

    /// <summary>
    /// Property-based tests for pack conflict declaration invariants.
    /// </summary>
    [Trait("Category", "Property")]
    public class PackConflictPropertyTests
    {
        private readonly PackDependencyResolver _resolver = new PackDependencyResolver();
        private readonly PackLoader _loader = new PackLoader();

        /// <summary>
        /// Property: A pack that declares another active pack as a conflict always produces errors.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property ExplicitConflict_AlwaysDetected(string idA, string idB)
        {
            if (string.Equals(idA, idB, StringComparison.OrdinalIgnoreCase))
                return Prop.ToProperty(true);

            PackManifest a = MakePack(idA, conflictsWith: new[] { idB });
            PackManifest b = MakePack(idB);

            IReadOnlyList<string> conflicts = _resolver.DetectConflicts(new[] { a, b });
            return Prop.ToProperty(conflicts.Count > 0);
        }

        /// <summary>
        /// Property: Packs with no declared conflicts produce no conflict errors.
        /// </summary>
        [Property(Arbitrary = new[] { typeof(PackGenerators) })]
        [Trait("Category", "Property")]
        public Property NoDeclaredConflicts_ProducesNoConflicts(string id1, string id2)
        {
            if (string.Equals(id1, id2, StringComparison.OrdinalIgnoreCase))
                return Prop.ToProperty(true);

            PackManifest a = MakePack(id1);
            PackManifest b = MakePack(id2);

            IReadOnlyList<string> conflicts = _resolver.DetectConflicts(new[] { a, b });
            return Prop.ToProperty(conflicts.Count == 0);
        }

        private PackManifest MakePack(string id, string[]? conflictsWith = null)
        {
            string conflicts = conflictsWith is { Length: > 0 }
                ? "conflicts_with:\n" + string.Join("", conflictsWith.Select(c => $"  - \"{c}\"\n"))
                : "";

            string yaml = $@"
id: ""{id}""
name: ""{id}""
version: 0.1.0
author: Test
type: content
{conflicts}";
            return _loader.LoadFromString(yaml);
        }
    }
}
