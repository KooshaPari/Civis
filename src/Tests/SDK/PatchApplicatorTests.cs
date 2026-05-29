using System;
using System.Collections.Generic;
using DINOForge.SDK;
using DINOForge.SDK.Patching;
using FluentAssertions;
using FluentAssertions.Collections;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Unit tests for <see cref="PatchApplicator"/> — RimWorld-style cross-mod patching.
    /// Tests cover: replace, add, remove, multiply, wildcard paths, atomic-skip-on-error.
    /// </summary>
    public sealed class PatchApplicatorTests
    {
        // ----------------------------------------------------------------------------------
        // Helpers
        // ----------------------------------------------------------------------------------

        /// <summary>
        /// Builds a minimal pack content dictionary with a "units" section containing two units.
        /// Shape: packId → section → itemId → { field → value }
        /// </summary>
        private static Dictionary<string, Dictionary<string, Dictionary<string, object>>> BuildContent()
        {
            return new Dictionary<string, Dictionary<string, Dictionary<string, object>>>(StringComparer.OrdinalIgnoreCase)
            {
                ["warfare-starwars"] = new Dictionary<string, Dictionary<string, object>>(StringComparer.OrdinalIgnoreCase)
                {
                    ["units"] = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                    {
                        ["rep_clone_trooper"] = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                        {
                            ["stats"] = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                            {
                                ["hp"] = (object)100,
                                ["damage"] = (object)20
                            },
                            ["tags"] = new System.Collections.Generic.List<object> { "infantry" }
                        },
                        ["rep_clone_militia"] = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                        {
                            ["stats"] = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                            {
                                ["hp"] = (object)60,
                                ["damage"] = (object)10
                            },
                            ["tags"] = new System.Collections.Generic.List<object> { "light" }
                        }
                    }
                }
            };
        }

        private static PatchApplicator MakeApplicator(List<string>? log = null)
        {
            return log == null
                ? new PatchApplicator()
                : new PatchApplicator(msg => log.Add(msg));
        }

        private static List<(string, PatchSet)> OneSet(string targetPack, params PatchOperation[] ops)
        {
            return new List<(string, PatchSet)>
            {
                ("balance-mod", new PatchSet
                {
                    TargetPack = targetPack,
                    Operations = new List<PatchOperation>(ops)
                })
            };
        }

        // ----------------------------------------------------------------------------------
        // replace / set
        // ----------------------------------------------------------------------------------

        [Fact]
        public void Replace_ConcreteField_SetsNewValue()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "replace", Path = "/units/rep_clone_trooper/stats/hp", Value = 90 }));

            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            stats["hp"].Should().Be(90);
        }

        [Fact]
        public void Set_IsAliasForReplace()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "set", Path = "/units/rep_clone_trooper/stats/damage", Value = 25 }));

            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            stats["damage"].Should().Be(25);
        }

        [Fact]
        public void Replace_NonExistentKey_Throws_AndSkipsRemainingOps()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            List<string> log = new List<string>();
            PatchApplicator applicator = MakeApplicator(log);

            // First op fails (missing key), second op should be skipped.
            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "replace", Path = "/units/rep_clone_trooper/stats/missing_key", Value = 99 },
                new PatchOperation { Op = "replace", Path = "/units/rep_clone_trooper/stats/hp", Value = 1 }));

            // hp should remain 100 because the set was skipped after failure
            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            stats["hp"].Should().Be(100);
            log.Should().Contain(l => l.Contains("FAILED"));
        }

        // ----------------------------------------------------------------------------------
        // add
        // ----------------------------------------------------------------------------------

        [Fact]
        public void Add_ToExistingList_AppendsValue()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "add", Path = "/units/rep_clone_trooper/tags", Value = "armored" }));

            List<object> tags = (List<object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["tags"];
            tags.Should().Contain("armored");
            tags.Should().HaveCount(2);
        }

        [Fact]
        public void Add_NewKey_CreatesEntry()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "add", Path = "/units/rep_clone_trooper/stats/shield", Value = 50 }));

            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            stats.Should().ContainKey("shield");
            stats["shield"].Should().Be(50);
        }

        // ----------------------------------------------------------------------------------
        // remove
        // ----------------------------------------------------------------------------------

        [Fact]
        public void Remove_EntireItem_DeletesFromSection()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "remove", Path = "/units/rep_clone_militia" }));

            content["warfare-starwars"]["units"].Should().NotContainKey("rep_clone_militia");
        }

        [Fact]
        public void Remove_Field_DeletesFieldFromItem()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "remove", Path = "/units/rep_clone_trooper/tags" }));

            Dictionary<string, object> unit =
                (Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"];
            unit.Should().NotContainKey("tags");
        }

        // ----------------------------------------------------------------------------------
        // multiply
        // ----------------------------------------------------------------------------------

        [Fact]
        public void Multiply_Integer_ReducesValue()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "multiply", Path = "/units/rep_clone_trooper/stats/hp", Factor = 0.9 }));

            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            // 100 * 0.9 = 90 (rounded int)
            stats["hp"].Should().Be(90);
        }

        [Fact]
        public void Multiply_MissingFactor_IsAtomic_SkipsRemainingOps()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            List<string> log = new List<string>();
            PatchApplicator applicator = MakeApplicator(log);

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "multiply", Path = "/units/rep_clone_trooper/stats/hp", Factor = null },
                new PatchOperation { Op = "replace", Path = "/units/rep_clone_trooper/stats/hp", Value = 1 }));

            // hp should remain 100: first op failed, second skipped
            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            stats["hp"].Should().Be(100);
            log.Should().Contain(l => l.Contains("FAILED"));
        }

        // ----------------------------------------------------------------------------------
        // Wildcard
        // ----------------------------------------------------------------------------------

        [Fact]
        public void Wildcard_Multiply_AppliesAllItems()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "multiply", Path = "/units/*/stats/hp", Factor = 0.9 }));

            Dictionary<string, object> trooperStats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            Dictionary<string, object> militiaStats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_militia"])["stats"];

            trooperStats["hp"].Should().Be(90); // 100 * 0.9 = 90
            militiaStats["hp"].Should().Be(54); // 60 * 0.9 = 54
        }

        [Fact]
        public void Wildcard_Replace_AppliesAllItems()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            PatchApplicator applicator = MakeApplicator();

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "replace", Path = "/units/*/stats/damage", Value = 15 }));

            Dictionary<string, object> trooperStats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            Dictionary<string, object> militiaStats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_militia"])["stats"];

            trooperStats["damage"].Should().Be(15);
            militiaStats["damage"].Should().Be(15);
        }

        // ----------------------------------------------------------------------------------
        // Target pack not loaded → silent skip
        // ----------------------------------------------------------------------------------

        [Fact]
        public void Apply_TargetPackNotLoaded_SilentlySkips()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            List<string> log = new List<string>();
            PatchApplicator applicator = MakeApplicator(log);

            applicator.Apply(content, OneSet("non-existent-pack",
                new PatchOperation { Op = "replace", Path = "/units/rep_clone_trooper/stats/hp", Value = 1 }));

            // No crash, no mutation; target was absent
            Dictionary<string, object> stats = (Dictionary<string, object>)
                ((Dictionary<string, object>)content["warfare-starwars"]["units"]["rep_clone_trooper"])["stats"];
            stats["hp"].Should().Be(100);
            log.Should().Contain(l => l.Contains("Skipping"));
        }

        // ----------------------------------------------------------------------------------
        // Unknown op
        // ----------------------------------------------------------------------------------

        [Fact]
        public void UnknownOp_LogsFailureAndSkipsRemainingOps()
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> content = BuildContent();
            List<string> log = new List<string>();
            PatchApplicator applicator = MakeApplicator(log);

            applicator.Apply(content, OneSet("warfare-starwars",
                new PatchOperation { Op = "frobnicate", Path = "/units/rep_clone_trooper/stats/hp", Value = 1 }));

            log.Should().Contain(l => l.Contains("FAILED"));
        }

        // ----------------------------------------------------------------------------------
        // PackManifest.Patches round-trip via YAML deserialization
        // ----------------------------------------------------------------------------------

        [Fact]
        public void PackManifest_Patches_DeserializesFromYaml()
        {
            const string yaml = @"
id: balance-mod
name: Balance Mod
version: 0.1.0
patches:
  - target_pack: warfare-starwars
    operations:
      - op: multiply
        path: /units/*/stats/hp
        factor: 0.9
      - op: add
        path: /units/rep_clone_trooper/tags
        value: armored
";
            PackManifest manifest = YamlLoader.Deserializer.Deserialize<PackManifest>(yaml);

            manifest.Patches.Should().NotBeNull();
            manifest.Patches!.Count.Should().Be(1);
            manifest.Patches[0].TargetPack.Should().Be("warfare-starwars");
            manifest.Patches[0].Operations.Count.Should().Be(2);
            manifest.Patches[0].Operations[0].Op.Should().Be("multiply");
            manifest.Patches[0].Operations[0].Factor.Should().BeApproximately(0.9, 1e-9);
            manifest.Patches[0].Operations[1].Op.Should().Be("add");
            manifest.Patches[0].Operations[1].Value.Should().Be("armored");
        }
    }
}
