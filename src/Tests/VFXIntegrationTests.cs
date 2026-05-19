#nullable enable
using System;
using System.IO;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Repo-local VFX integration contracts.
    /// These tests avoid a second fake runtime implementation while still locking down
    /// the key source relationships that the runtime VFX stack depends on.
    /// </summary>
    public class VFXIntegrationTests
    {
        private static readonly string RepoRoot = GetRepoRoot();
        private static readonly string VfxPoolManagerSourcePath = Path.Combine(RepoRoot, "src/Runtime/Bridge/VFXPoolManager.cs");
        private static readonly string LodManagerSourcePath = Path.Combine(RepoRoot, "src/Runtime/Bridge/LODManager.cs");
        private static readonly string VfxPrefabDescriptorSourcePath = Path.Combine(RepoRoot, "src/Runtime/VFX/VFXPrefabDescriptor.cs");

        private static string GetRepoRoot()
        {
            string? currentDir = AppContext.BaseDirectory;
            while (currentDir != null && !Directory.Exists(Path.Combine(currentDir, "packs")))
            {
                currentDir = Path.GetDirectoryName(currentDir);
            }

            return currentDir ?? throw new InvalidOperationException("Could not find repo root");
        }

        [Fact]
        public void VfxRuntime_SourceFiles_RemainPresent()
        {
            File.Exists(VfxPoolManagerSourcePath).Should().BeTrue();
            File.Exists(LodManagerSourcePath).Should().BeTrue();
            File.Exists(VfxPrefabDescriptorSourcePath).Should().BeTrue();
        }

        [Fact]
        public void VfxPoolManager_RetainsDescriptorFallback_AndCatalogDependency()
        {
            string source = File.ReadAllText(VfxPoolManagerSourcePath, System.Text.Encoding.UTF8);

            source.Should().Contain("CreatePrefabFromDescriptor");
            source.Should().Contain("VFXPrefabCatalog.GetAllPrefabs()");
            source.Should().Contain("LoadPrefabFromPack");
            source.Should().Contain("Binary prefab not found (");
        }

        [Fact]
        public void VfxStack_RetainsExpectedLodAndFactionContracts()
        {
            string lodSource = File.ReadAllText(LodManagerSourcePath, System.Text.Encoding.UTF8);
            string descriptorSource = File.ReadAllText(VfxPrefabDescriptorSourcePath, System.Text.Encoding.UTF8);

            lodSource.Should().Contain("FullQualityDistance = 100f");
            lodSource.Should().Contain("MediumQualityDistance = 200f");
            lodSource.Should().Contain("LODTier.CULLED => 0.0f");

            descriptorSource.Should().Contain("RepublicBlue");
            descriptorSource.Should().Contain("CISRed");
            descriptorSource.Should().Contain("public static VFXPrefabDescriptor[] GetAllPrefabs()");
        }
    }
}

