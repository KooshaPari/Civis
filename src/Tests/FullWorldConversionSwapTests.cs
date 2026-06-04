using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Assets;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for #975 Phase 1 (full-world Star Wars conversion) asset-swap wiring:
    ///   Gap A — building swaps must register WITH a vanilla_mapping so the runtime
    ///           AssetSwapSystem can target them (previously buildings registered without
    ///           one and ran in "no targeting signal" DIAGNOSTIC MODE).
    ///   Gap B — cims (citizens/workers) modelled as units with vanilla_mapping=cims are
    ///           wired through the existing unit registration path.
    ///
    /// These tests drive <see cref="ContentLoader.LoadPack"/> against a synthetic pack with
    /// on-disk placeholder bundle files and assert the resulting <see cref="AssetSwapRequest"/>
    /// entries carry the expected <see cref="AssetSwapRequest.VanillaMapping"/>.
    /// </summary>
    [Trait("Category", "UserStory")]
    [Trait("UserStory", "US-F5.1")]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateTotalConversion")]
    [Collection(AssetSwapRegistryCollection.Name)]
    public sealed class FullWorldConversionSwapTests : IDisposable
    {
        private readonly string _tempDir;

        public FullWorldConversionSwapTests()
        {
            AssetSwapRegistry.Clear();
            _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge-fwc-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
        }

        public void Dispose()
        {
            AssetSwapRegistry.Clear();
            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, recursive: true);
            }
            catch (IOException) { /* best-effort temp cleanup */ }
        }

        private string CreatePackWithBundles()
        {
            string packDir = Path.Combine(_tempDir, "fwc-pack");
            Directory.CreateDirectory(packDir);
            string bundlesDir = Path.Combine(packDir, "assets", "bundles");
            Directory.CreateDirectory(bundlesDir);
            string unitsDir = Path.Combine(packDir, "units");
            Directory.CreateDirectory(unitsDir);
            string buildingsDir = Path.Combine(packDir, "buildings");
            Directory.CreateDirectory(buildingsDir);

            // Placeholder bundle files (UnityBundle-like header; existence is all RegisterAssetSwaps checks).
            byte[] header = { 0x55, 0x6E, 0x69, 0x74, 0x79, 0x42, 0x75, 0x6E, 0x64, 0x6C, 0x65 };
            foreach (string asset in new[] { "sw-rep-command-center", "sw-rep-clone-engineer" })
            {
                File.WriteAllBytes(Path.Combine(bundlesDir, asset), header);
            }

            File.WriteAllText(Path.Combine(packDir, "pack.yaml"),
@"id: fwc-pack
name: Full World Conversion Test Pack
version: 1.0.0
framework_version: "">=0.1.0 <2.0.0""
type: total_conversion
loads:
  units: [units]
  buildings: [buildings]
");

            // Gap B: cims modelled as a unit with vanilla_mapping=cims.
            File.WriteAllText(Path.Combine(unitsDir, "cims.yaml"),
@"- id: clone_worker
  display_name: Clone Worker
  unit_class: SupportEngineer
  faction_id: republic
  vanilla_mapping: cims
  visual_asset: sw-rep-clone-engineer
  stats:
    hp: 60.0
");

            // Gap A: command building with an explicit vanilla_mapping.
            File.WriteAllText(Path.Combine(buildingsDir, "command.yaml"),
@"- id: command_center
  display_name: Command Center
  building_type: command
  vanilla_mapping: command
  health: 2000
  visual_asset: sw-rep-command-center
");

            return packDir;
        }

        [Fact]
        public void GapB_CimsUnit_RegistersSwapWithCimsVanillaMapping()
        {
            string packDir = CreatePackWithBundles();
            RegistryManager registry = new RegistryManager();
            ContentLoader loader = new ContentLoader(registry);

            ContentLoadResult result = loader.LoadPack(packDir);

            result.IsSuccess.Should().BeTrue(
                "pack should load cleanly; errors: " + string.Join("; ", result.Errors));

            AssetSwapRequest? cimsSwap = AssetSwapRegistry.GetPending()
                .FirstOrDefault(r => r.AssetAddress == "sw-rep-clone-engineer");
            cimsSwap.Should().NotBeNull("cims unit with a present bundle should register a swap");
            cimsSwap!.VanillaMapping.Should().Be("cims",
                "cims swap must carry the cims vanilla_mapping so the runtime can target worker entities");
        }

        [Fact]
        public void GapA_Building_RegistersSwapWithVanillaMapping()
        {
            string packDir = CreatePackWithBundles();
            RegistryManager registry = new RegistryManager();
            ContentLoader loader = new ContentLoader(registry);

            loader.LoadPack(packDir);

            AssetSwapRequest? buildingSwap = AssetSwapRegistry.GetPending()
                .FirstOrDefault(r => r.AssetAddress == "sw-rep-command-center");
            buildingSwap.Should().NotBeNull("building with a present bundle should register a swap");
            buildingSwap!.VanillaMapping.Should().Be("command",
                "Gap A: building swaps must carry a vanilla_mapping (no longer null) so they exit DIAGNOSTIC MODE");
        }

        [Fact]
        public void GapA_Building_WithoutExplicitMapping_FallsBackToBuildingType()
        {
            // Building omits vanilla_mapping but declares building_type — the loader must
            // fall back to building_type so the swap still carries a targeting signal.
            string packDir = Path.Combine(_tempDir, "fallback-pack");
            Directory.CreateDirectory(packDir);
            string bundlesDir = Path.Combine(packDir, "assets", "bundles");
            Directory.CreateDirectory(bundlesDir);
            string buildingsDir = Path.Combine(packDir, "buildings");
            Directory.CreateDirectory(buildingsDir);

            byte[] header = { 0x55, 0x6E, 0x69, 0x74, 0x79, 0x42, 0x75, 0x6E, 0x64, 0x6C, 0x65 };
            File.WriteAllBytes(Path.Combine(bundlesDir, "sw-clone-barracks"), header);

            File.WriteAllText(Path.Combine(packDir, "pack.yaml"),
@"id: fallback-pack
name: Fallback Pack
version: 1.0.0
framework_version: "">=0.1.0 <2.0.0""
type: total_conversion
loads:
  buildings: [buildings]
");
            File.WriteAllText(Path.Combine(buildingsDir, "barracks.yaml"),
@"- id: barracks
  display_name: Barracks
  building_type: barracks
  health: 1000
  visual_asset: sw-clone-barracks
");

            RegistryManager registry = new RegistryManager();
            ContentLoader loader = new ContentLoader(registry);
            loader.LoadPack(packDir);

            AssetSwapRequest? swap = AssetSwapRegistry.GetPending()
                .FirstOrDefault(r => r.AssetAddress == "sw-clone-barracks");
            swap.Should().NotBeNull();
            swap!.VanillaMapping.Should().Be("barracks",
                "building without explicit vanilla_mapping should fall back to building_type");
        }
    }
}
