using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for pack dependency resolution including cycles and conflicts.
    /// Maps to: US-F1.1 Pack Manifest Loading (dependency resolution)
    /// </summary>
    [Trait("Category", "UserStory")]
    [Trait("UserStory", "US-F1.1")]
    [Trait("Category", "Epic")]
    [Trait("Epic", "Epic-PackSystem")]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-Debug")]
    public class DependencyResolverTests
    {
        private readonly PackLoader _loader = new();
        private readonly PackDependencyResolver _resolver = new();

        private PackManifest MakePack(string id, string[]? dependsOn = null, string[]? conflictsWith = null, int loadOrder = 100)
        {
            string deps = dependsOn is { Length: > 0 }
                ? "depends_on:\n" + string.Join("", System.Array.ConvertAll(dependsOn, d => $"  - \"{d}\"\n"))
                : "";

            string conflicts = conflictsWith is { Length: > 0 }
                ? "conflicts_with:\n" + string.Join("", System.Array.ConvertAll(conflictsWith, c => $"  - \"{c}\"\n"))
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

        // ── ResolveDependencies ─────────────────────────────────────────────

        [Fact]
        public void ResolveDeps_NoDependencies_Succeeds()
        {
            PackManifest target = MakePack("standalone");
            var available = new List<PackManifest> { target };

            DependencyResult result = _resolver.ResolveDependencies(available, target);

            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().ContainSingle(p => p.Id == "standalone");
        }

        [Fact]
        public void ResolveDeps_SatisfiedDependency_Succeeds()
        {
            PackManifest core = MakePack("core");
            PackManifest addon = MakePack("addon", dependsOn: new[] { "core" });
            var available = new List<PackManifest> { core, addon };

            DependencyResult result = _resolver.ResolveDependencies(available, addon);

            result.IsSuccess.Should().BeTrue();
            result.Errors.Should().BeEmpty();
            result.LoadOrder.Should().Contain(p => p.Id == "addon");
        }

        [Fact]
        public void ResolveDeps_MissingDependency_Fails()
        {
            PackManifest addon = MakePack("addon", dependsOn: new[] { "missing-pack" });
            var available = new List<PackManifest> { addon };

            DependencyResult result = _resolver.ResolveDependencies(available, addon);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("missing-pack");
        }

        // ── DetectConflicts ─────────────────────────────────────────────────

        [Fact]
        public void DetectConflicts_NoConflicts_ReturnsEmpty()
        {
            PackManifest alpha = MakePack("alpha");
            PackManifest beta = MakePack("beta");
            var active = new List<PackManifest> { alpha, beta };

            var conflicts = _resolver.DetectConflicts(active);

            conflicts.Should().BeEmpty();
        }

        [Fact]
        public void DetectConflicts_ConflictingPacks_ReturnsConflicts()
        {
            PackManifest alpha = MakePack("alpha", conflictsWith: new[] { "beta" });
            PackManifest beta = MakePack("beta");
            var active = new List<PackManifest> { alpha, beta };

            var conflicts = _resolver.DetectConflicts(active);

            conflicts.Should().HaveCount(1);
            conflicts[0].Should().Contain("alpha");
            conflicts[0].Should().Contain("beta");
        }

        // ── ComputeLoadOrder ────────────────────────────────────────────────

        [Fact]
        public void ComputeLoadOrder_SinglePack_ReturnsSingle()
        {
            PackManifest solo = MakePack("solo");
            var packs = new List<PackManifest> { solo };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().ContainSingle(p => p.Id == "solo");
        }

        [Fact]
        public void ComputeLoadOrder_WithDeps_RespectsDependencyOrder()
        {
            PackManifest core = MakePack("core", loadOrder: 50);
            PackManifest addon = MakePack("addon", dependsOn: new[] { "core" }, loadOrder: 100);
            var packs = new List<PackManifest> { addon, core }; // deliberately reversed

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeTrue();
            var ids = result.LoadOrder.Select(p => p.Id).ToList();
            ids.IndexOf("core").Should().BeLessThan(ids.IndexOf("addon"), "core must be loaded before addon");
        }

        [Fact]
        public void ComputeLoadOrder_CircularDeps_Fails()
        {
            PackManifest alpha = MakePack("alpha", dependsOn: new[] { "beta" });
            PackManifest beta = MakePack("beta", dependsOn: new[] { "alpha" });
            var packs = new List<PackManifest> { alpha, beta };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeFalse();
            result.Errors[0].Should().Contain("ircular");
        }

        // ── Edge Cases ───────────────────────────────────────────────────

        [Fact]
        public void ComputeLoadOrder_DiamondDependency_Resolves()
        {
            // A depends on B and C, both B and C depend on D
            PackManifest d = MakePack("d", loadOrder: 10);
            PackManifest b = MakePack("b", dependsOn: new[] { "d" }, loadOrder: 20);
            PackManifest c = MakePack("c", dependsOn: new[] { "d" }, loadOrder: 20);
            PackManifest a = MakePack("a", dependsOn: new[] { "b", "c" }, loadOrder: 30);
            var packs = new List<PackManifest> { a, b, c, d };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeTrue();
            var ids = result.LoadOrder.Select(p => p.Id).ToList();
            ids.IndexOf("d").Should().BeLessThan(ids.IndexOf("b"));
            ids.IndexOf("d").Should().BeLessThan(ids.IndexOf("c"));
            ids.IndexOf("b").Should().BeLessThan(ids.IndexOf("a"));
            ids.IndexOf("c").Should().BeLessThan(ids.IndexOf("a"));
        }

        [Fact]
        public void ComputeLoadOrder_LongChainDependency_Resolves()
        {
            // A -> B -> C -> D -> E
            PackManifest e = MakePack("e", loadOrder: 10);
            PackManifest d = MakePack("d", dependsOn: new[] { "e" }, loadOrder: 20);
            PackManifest c = MakePack("c", dependsOn: new[] { "d" }, loadOrder: 30);
            PackManifest b = MakePack("b", dependsOn: new[] { "c" }, loadOrder: 40);
            PackManifest a = MakePack("a", dependsOn: new[] { "b" }, loadOrder: 50);
            var packs = new List<PackManifest> { a, c, e, b, d }; // deliberately shuffled

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeTrue();
            var ids = result.LoadOrder.Select(p => p.Id).ToList();
            ids.Should().Equal("e", "d", "c", "b", "a");
        }

        [Fact]
        public void ComputeLoadOrder_SelfDependency_FailsOrHandled()
        {
            PackManifest self = MakePack("self-dep", dependsOn: new[] { "self-dep" });
            var packs = new List<PackManifest> { self };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            // Self-dependency creates a cycle - should be detected
            result.IsSuccess.Should().BeFalse();
        }

        [Fact]
        public void DetectConflicts_BidirectionalConflicts_BothDetected()
        {
            // Alpha conflicts with beta AND beta conflicts with alpha
            PackManifest alpha = MakePack("alpha", conflictsWith: new[] { "beta" });
            PackManifest beta = MakePack("beta", conflictsWith: new[] { "alpha" });
            var active = new List<PackManifest> { alpha, beta };

            var conflicts = _resolver.DetectConflicts(active);

            // Should detect conflicts from both directions
            conflicts.Should().HaveCountGreaterOrEqualTo(2);
        }

        [Fact]
        public void CheckFrameworkCompatibility_EmptyFrameworkVersion_ReturnsTrue()
        {
            PackManifest pack = MakePack("compat-test");
            pack.GetType().GetProperty("FrameworkVersion")!.SetValue(pack, "");

            bool result = _resolver.CheckFrameworkCompatibility(pack, "0.1.0");

            result.Should().BeTrue();
        }

        [Fact]
        public void CheckFrameworkCompatibility_ExactMatch_ReturnsTrue()
        {
            string yaml = @"
id: compat
name: Compat
version: 0.1.0
author: Test
type: content
framework_version: '0.1.0'
";
            PackManifest pack = _loader.LoadFromString(yaml);

            bool result = _resolver.CheckFrameworkCompatibility(pack, "0.1.0");

            result.Should().BeTrue();
        }

        [Fact]
        public void CheckFrameworkCompatibility_VersionWithPrefix_StripsPrefix()
        {
            string yaml = @"
id: compat
name: Compat
version: 0.1.0
author: Test
type: content
framework_version: '>=0.2.0'
";
            PackManifest pack = _loader.LoadFromString(yaml);

            bool result = _resolver.CheckFrameworkCompatibility(pack, "0.2.0");

            result.Should().BeTrue();
        }

        [Fact]
        public void ComputeLoadOrder_ThreePackCircle_Fails()
        {
            PackManifest a = MakePack("a", dependsOn: new[] { "c" });
            PackManifest b = MakePack("b", dependsOn: new[] { "a" });
            PackManifest c = MakePack("c", dependsOn: new[] { "b" });
            var packs = new List<PackManifest> { a, b, c };

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeFalse();
        }

        [Fact]
        public void ComputeLoadOrder_EmptyList_Succeeds()
        {
            var packs = new List<PackManifest>();

            DependencyResult result = _resolver.ComputeLoadOrder(packs);

            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().BeEmpty();
        }

        [Fact]
        public void ResolveDeps_TransitiveDependency_AllIncluded()
        {
            // addon depends on core, core depends on engine
            PackManifest engine = MakePack("engine", loadOrder: 10);
            PackManifest core = MakePack("core", dependsOn: new[] { "engine" }, loadOrder: 20);
            PackManifest addon = MakePack("addon", dependsOn: new[] { "core" }, loadOrder: 30);
            var available = new List<PackManifest> { engine, core, addon };

            DependencyResult result = _resolver.ResolveDependencies(available, addon);

            result.IsSuccess.Should().BeTrue();
            var ids = result.LoadOrder.Select(p => p.Id).ToList();
            ids.Should().Contain("engine");
            ids.Should().Contain("core");
            ids.Should().Contain("addon");
        }
    }
}
