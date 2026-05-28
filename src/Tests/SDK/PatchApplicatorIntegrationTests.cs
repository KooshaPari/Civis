using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Integration tests for the PatchApplicator wired into <see cref="ContentLoader.LoadPacks"/>.
    /// These tests exercise the full pipeline: on-disk YAML packs → patch phase → typed
    /// deserialization → registry population.
    ///
    /// Scenarios:
    ///   1. Pack A patches Pack B's unit stats/hp → loaded unit has new HP value.
    ///   2. Pack A patches a missing target_pack → silently skipped, no crash.
    ///   3. Pack A's patch has a bad path (field not found) → remaining ops skipped,
    ///      other content still loads; failure surfaced in load errors.
    /// </summary>
    public sealed class PatchApplicatorIntegrationTests : IDisposable
    {
        // Root temp directory for this test run — cleaned up in Dispose().
        private readonly string _root;

        public PatchApplicatorIntegrationTests()
        {
            _root = Path.Combine(
                Path.GetTempPath(),
                "dinoforge_patch_integration_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            if (Directory.Exists(_root))
            {
                try { Directory.Delete(_root, recursive: true); }
                catch { /* best-effort cleanup */ }
            }
        }

        // ----------------------------------------------------------------------------------
        // Helpers
        // ----------------------------------------------------------------------------------

        /// <summary>Creates a pack directory and writes pack.yaml + optional content files.</summary>
        private string CreatePack(string packId, string packYaml, Dictionary<string, string>? contentFiles = null)
        {
            string packDir = Path.Combine(_root, packId);
            Directory.CreateDirectory(packDir);
            File.WriteAllText(Path.Combine(packDir, "pack.yaml"), packYaml);

            if (contentFiles != null)
            {
                foreach (KeyValuePair<string, string> kvp in contentFiles)
                {
                    string fullPath = Path.Combine(packDir, kvp.Key);
                    Directory.CreateDirectory(Path.GetDirectoryName(fullPath)!);
                    File.WriteAllText(fullPath, kvp.Value);
                }
            }

            return packDir;
        }

        /// <summary>
        /// Minimal valid unit YAML (satisfies UnitDefinition.Validate: id, display_name,
        /// faction_id, unit_class, and stats.hp > 0 are all required).
        /// </summary>
        private static string UnitYaml(string id, float hp, float damage = 10f) =>
            $@"- id: {id}
  display_name: {id} Display
  faction_id: test_faction
  unit_class: CoreLineInfantry
  stats:
    hp: {hp}
    damage: {damage}
    accuracy: 0.7
";

        private static ContentLoader MakeLoader()
        {
            RegistryManager rm = new RegistryManager();
            return new ContentLoader(rm);
        }

        /// <summary>Reaches the private _registryManager field via reflection.</summary>
        private static RegistryManager GetRegistryManager(ContentLoader loader)
        {
            System.Reflection.FieldInfo? field = typeof(ContentLoader)
                .GetField("_registryManager",
                    System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            field.Should().NotBeNull("ContentLoader must have a _registryManager field");
            return (RegistryManager)field!.GetValue(loader)!;
        }

        // ----------------------------------------------------------------------------------
        // Test 1: patch replaces stats/hp → registry has the new value
        // ----------------------------------------------------------------------------------

        [Fact]
        public void LoadPacks_PatchReplacesUnitHp_RegisteredUnitHasNewHp()
        {
            // Pack B: warrior unit with hp=100
            string packBManifest = @"
id: pack-b
name: Pack B
version: 0.1.0
framework_version: '>=0.1.0 <99.0.0'
type: content
";
            // Pack A: patches pack-b warrior stats/hp → 150
            string packAManifest = @"
id: pack-a
name: Pack A
version: 0.1.0
framework_version: '>=0.1.0 <99.0.0'
type: content
patches:
  - target_pack: pack-b
    operations:
      - op: replace
        path: /units/warrior/stats/hp
        value: 150
";

            CreatePack("pack-b", packBManifest, new Dictionary<string, string>
            {
                ["units/warrior.yaml"] = UnitYaml("warrior", hp: 100)
            });
            CreatePack("pack-a", packAManifest);

            ContentLoader loader = MakeLoader();
            loader.LoadPacks(_root);

            RegistryManager rm = GetRegistryManager(loader);
            rm.Units.All.Should().ContainKey("warrior",
                because: "warrior unit from pack-b must be registered");

            UnitDefinition warrior = rm.Units.All["warrior"].Data;
            warrior.Stats.Hp.Should().BeApproximately(150f, 0.01f,
                because: "pack-a patched pack-b warrior stats/hp from 100 to 150");
        }

        // ----------------------------------------------------------------------------------
        // Test 2: patch targets missing pack → skipped, no crash, existing content loads
        // ----------------------------------------------------------------------------------

        [Fact]
        public void LoadPacks_PatchTargetsMissingPack_SkippedAndOtherPackLoads()
        {
            // Pack B: archer unit (hp=80)
            string packBManifest = @"
id: pack-b
name: Pack B
version: 0.1.0
framework_version: '>=0.1.0 <99.0.0'
type: content
";
            // Pack A: patches a pack that does NOT exist in _root
            string packAManifest = @"
id: pack-a
name: Pack A
version: 0.1.0
framework_version: '>=0.1.0 <99.0.0'
type: content
patches:
  - target_pack: does-not-exist
    operations:
      - op: replace
        path: /units/archer/stats/hp
        value: 1
";

            CreatePack("pack-b", packBManifest, new Dictionary<string, string>
            {
                ["units/archer.yaml"] = UnitYaml("archer", hp: 80)
            });
            CreatePack("pack-a", packAManifest);

            ContentLoader loader = MakeLoader();
            ContentLoadResult result = loader.LoadPacks(_root);

            // No crash. Archer still registered with original hp.
            RegistryManager rm = GetRegistryManager(loader);
            rm.Units.All.Should().ContainKey("archer",
                because: "pack-b content must load even when pack-a's patch target is missing");

            UnitDefinition archer = rm.Units.All["archer"].Data;
            archer.Stats.Hp.Should().BeApproximately(80f, 0.01f,
                because: "the patch was silently skipped since target pack does not exist");

            // Errors list should contain a skip message (not a crash).
            result.Errors.Should().Contain(
                e => e.Contains("Skipping") || e.Contains("not loaded"),
                because: "missing target pack must surface as an informational skip, not a crash");
        }

        // ----------------------------------------------------------------------------------
        // Test 3: bad path in patch → remaining ops skipped (atomic); other content loads
        // ----------------------------------------------------------------------------------

        [Fact]
        public void LoadPacks_PatchHasBadPath_RemainingOpsSkippedAndContentStillLoads()
        {
            // Pack B: knight (hp=200) and peasant (hp=50)
            string packBManifest = @"
id: pack-b
name: Pack B
version: 0.1.0
framework_version: '>=0.1.0 <99.0.0'
type: content
";
            string packBUnits = UnitYaml("knight", hp: 200, damage: 30)
                              + UnitYaml("peasant", hp: 50, damage: 5);

            // Pack A: first op references a field that doesn't exist (bad path).
            // Due to atomic-per-set semantics the second op (hp=1) must be SKIPPED.
            string packAManifest = @"
id: pack-a
name: Pack A
version: 0.1.0
framework_version: '>=0.1.0 <99.0.0'
type: content
patches:
  - target_pack: pack-b
    operations:
      - op: replace
        path: /units/knight/stats/nonexistent_field
        value: 999
      - op: replace
        path: /units/knight/stats/hp
        value: 1
";

            CreatePack("pack-b", packBManifest, new Dictionary<string, string>
            {
                ["units/units.yaml"] = packBUnits
            });
            CreatePack("pack-a", packAManifest);

            ContentLoader loader = MakeLoader();
            ContentLoadResult result = loader.LoadPacks(_root);

            // Both units must be registered.
            RegistryManager rm = GetRegistryManager(loader);
            rm.Units.All.Should().ContainKey("knight");
            rm.Units.All.Should().ContainKey("peasant");

            // Knight hp stays 200: first op failed → second op was skipped (atomic-per-set).
            UnitDefinition knight = rm.Units.All["knight"].Data;
            knight.Stats.Hp.Should().BeApproximately(200f, 0.01f,
                because: "bad-path op failed and atomic semantics skipped the hp=1 replacement");

            // The FAILED patch message must appear in the errors collection.
            result.Errors.Should().Contain(
                e => e.Contains("FAILED"),
                because: "failed patch operation must be surfaced in load errors");
        }
    }
}
