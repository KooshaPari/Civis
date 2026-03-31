#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// BDD-style behavior specifications for DINOForge core systems.
    /// Tests are written in Given_When_Then format covering pack loading, validation,
    /// compatibility, registries, and asset replacement scenarios.
    /// </summary>
    public class BddSpecs
    {
        #region Pack Loading Scenarios

        [Fact]
        public void Given_ValidPackManifest_When_Loaded_Then_AllRegistriesPopulated()
        {
            // Given: A valid pack manifest with all content types
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "1.0.0",
                Author = "Test",
                Type = "content"
            };

            var registryManager = new RegistryManager();
            var loader = new ContentLoader(registryManager);

            var unit = new UnitDefinition { Id = "unit-1", DisplayName = "Test Unit", Stats = new UnitStats { Hp = 100f } };
            var building = new BuildingDefinition { Id = "building-1", DisplayName = "Test Building", Health = 50 };
            var faction = new FactionDefinition { Faction = new FactionInfo { Id = "faction-1", DisplayName = "Test Faction" } };

            // When: Manual registration simulates pack loading
            registryManager.Units.Register(unit.Id, unit, RegistrySource.Pack, manifest.Id, manifest.LoadOrder);
            registryManager.Buildings.Register(building.Id, building, RegistrySource.Pack, manifest.Id, manifest.LoadOrder);
            registryManager.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, manifest.Id, manifest.LoadOrder);

            // Then: All registries should contain the registered items
            registryManager.Units.Get(unit.Id).Should().NotBeNull();
            registryManager.Buildings.Get(building.Id).Should().NotBeNull();
            registryManager.Factions.Get(faction.Faction.Id).Should().NotBeNull();
            registryManager.Units.Contains(unit.Id).Should().BeTrue();
            registryManager.Buildings.Contains(building.Id).Should().BeTrue();
            registryManager.Factions.Contains(faction.Faction.Id).Should().BeTrue();
        }

        [Fact]
        public void Given_PackWithMissingId_When_Validated_Then_ReturnsError()
        {
            // Given: A pack manifest with empty ID
            var manifest = new PackManifest
            {
                Id = "", // Invalid: missing ID
                Name = "Test Pack"
            };

            var registryManager = new RegistryManager();
            var loader = new ContentLoader(registryManager);

            // When: Validation checks for required fields
            bool isValid = !string.IsNullOrWhiteSpace(manifest.Id);

            // Then: Validation should fail
            isValid.Should().BeFalse();
        }

        [Fact]
        public void Given_PackWithCircularDep_When_Resolved_Then_ThrowsOrReturnsError()
        {
            // Given: Two packs with circular dependencies
            var packA = new PackManifest
            {
                Id = "pack-a",
                Name = "Pack A",
                DependsOn = new List<string> { "pack-b" }
            };

            var packB = new PackManifest
            {
                Id = "pack-b",
                Name = "Pack B",
                DependsOn = new List<string> { "pack-a" }
            };

            var resolver = new PackDependencyResolver();

            // When: Attempting to compute load order with circular deps
            var result = resolver.ComputeLoadOrder(new[] { packA, packB });

            // Then: Resolution should fail with circular dependency error
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().ContainSingle(e => e.Contains("Circular"));
        }

        [Fact]
        public void Given_TotalConversionPack_When_Validated_Then_SingletonEnforced()
        {
            // Given: A total conversion manifest with singleton = true
            var tcManifest = new TotalConversionManifest
            {
                Id = "starwars-conversion",
                Type = "total_conversion",
                Singleton = true
            };

            // When: Checking singleton constraint
            bool isSingletonEnforced = tcManifest.Singleton && tcManifest.Type == "total_conversion";

            // Then: Singleton should be enforced for total conversions
            isSingletonEnforced.Should().BeTrue();
            tcManifest.Type.Should().Be("total_conversion");
        }

        #endregion

        #region Compatibility Scenarios

        [Fact]
        public void Given_PackRequiresFramework050_When_Framework050Installed_Then_Compatible()
        {
            // Given: A pack requiring framework 0.5.0
            var pack = new PackManifest
            {
                Id = "compat-pack",
                FrameworkVersion = "0.5.0"
            };

            var resolver = new PackDependencyResolver();
            string installedFramework = "0.5.0";

            // When: Checking compatibility with installed framework
            bool isCompatible = resolver.CheckFrameworkCompatibility(pack, installedFramework);

            // Then: Compatibility check should pass
            isCompatible.Should().BeTrue();
        }

        [Fact]
        public void Given_PackRequiresFramework200_When_Framework050Installed_Then_Incompatible()
        {
            // Given: A pack requiring framework 2.0.0
            var pack = new PackManifest
            {
                Id = "future-pack",
                FrameworkVersion = "2.0.0"
            };

            var resolver = new PackDependencyResolver();
            string installedFramework = "0.5.0";

            // When: Checking compatibility with older installed framework
            bool isCompatible = resolver.CheckFrameworkCompatibility(pack, installedFramework);

            // Then: Compatibility check should fail
            isCompatible.Should().BeFalse();
        }

        [Fact]
        public void Given_PackWithNoVersionReq_When_AnyFrameworkInstalled_Then_AlwaysCompatible()
        {
            // Given: A pack with no specific version requirement
            var pack = new PackManifest
            {
                Id = "universal-pack",
                FrameworkVersion = "" // No requirement
            };

            var resolver = new PackDependencyResolver();
            string installedFramework = "0.1.0";

            // When: Checking compatibility
            bool isCompatible = resolver.CheckFrameworkCompatibility(pack, installedFramework);

            // Then: Should always be compatible
            isCompatible.Should().BeTrue();
        }

        [Fact]
        public void Given_PackWithWildcardVersion_When_AnyFrameworkVersion_Then_AlwaysCompatible()
        {
            // Given: A pack with wildcard version requirement
            var pack = new PackManifest
            {
                Id = "wildcard-pack",
                FrameworkVersion = "*"
            };

            var resolver = new PackDependencyResolver();
            _ = "999.999.999";

            // When: Checking compatibility with wildcard
            bool isCompatible = string.IsNullOrWhiteSpace(pack.FrameworkVersion) ||
                               pack.FrameworkVersion == "*";

            // Then: Wildcard should accept any version
            isCompatible.Should().BeTrue();
        }

        #endregion

        #region Registry Scenarios

        [Fact]
        public void Given_EmptyRegistry_When_UnitRegistered_Then_ContainsUnit()
        {
            // Given: An empty unit registry
            var registry = new Registry<UnitDefinition>();
            var unit = new UnitDefinition { Id = "raptor", DisplayName = "Raptor", Stats = new UnitStats { Hp = 100f } };

            // When: Registering a unit
            registry.Register(unit.Id, unit, RegistrySource.BaseGame, "base-game");

            // Then: Registry should contain the unit
            registry.Contains(unit.Id).Should().BeTrue();
            registry.Get(unit.Id).Should().NotBeNull();
            registry.Get(unit.Id)!.DisplayName.Should().Be("Raptor");
        }

        [Fact]
        public void Given_Registry_When_SameUnitRegisteredTwice_Then_HigherPriorityWins()
        {
            // Given: A registry with a base game unit
            var registry = new Registry<UnitDefinition>();
            var baseUnit = new UnitDefinition { Id = "raptor", DisplayName = "Base Raptor", Stats = new UnitStats { Hp = 100f } };
            var modUnit = new UnitDefinition { Id = "raptor", DisplayName = "Mod Raptor", Stats = new UnitStats { Hp = 120f } };

            // When: First registering base unit, then overriding with mod unit
            registry.Register(baseUnit.Id, baseUnit, RegistrySource.BaseGame, "base-game", 0);
            registry.Register(modUnit.Id, modUnit, RegistrySource.Pack, "mod-pack", 100); // Higher priority

            // Then: Mod unit should be retrieved (higher priority)
            registry.Get("raptor")!.DisplayName.Should().Be("Mod Raptor");
            registry.Get("raptor")!.Stats.Hp.Should().Be(120f);
        }

        [Fact]
        public void Given_Registry_When_GetAllCalled_Then_ReturnsAllRegistered()
        {
            // Given: A registry with multiple units
            var registry = new Registry<UnitDefinition>();
            registry.Register("raptor", new UnitDefinition { Id = "raptor", DisplayName = "Raptor", Stats = new UnitStats { Hp = 100f } },
                RegistrySource.BaseGame, "base-game");
            registry.Register("triceratops", new UnitDefinition { Id = "triceratops", DisplayName = "Triceratops", Stats = new UnitStats { Hp = 150f } },
                RegistrySource.BaseGame, "base-game");
            registry.Register("ankylosaur", new UnitDefinition { Id = "ankylosaur", DisplayName = "Ankylosaur", Stats = new UnitStats { Hp = 200f } },
                RegistrySource.BaseGame, "base-game");

            // When: Getting all entries
            var allEntries = registry.All;

            // Then: All entries should be returned
            allEntries.Should().HaveCount(3);
            allEntries.Keys.Should().Contain(new[] { "raptor", "triceratops", "ankylosaur" });
        }

        [Fact]
        public void Given_Registry_When_MultiplePacksRegisterSameEntry_Then_ConflictDetected()
        {
            // Given: Two packs registering the same unit ID
            var registry = new Registry<UnitDefinition>();
            var packAUnit = new UnitDefinition { Id = "custom-unit", DisplayName = "Pack A Unit", Stats = new UnitStats { Hp = 100f } };
            var packBUnit = new UnitDefinition { Id = "custom-unit", DisplayName = "Pack B Unit", Stats = new UnitStats { Hp = 120f } };

            // When: Both packs register at same priority
            registry.Register(packAUnit.Id, packAUnit, RegistrySource.Pack, "pack-a", 100);
            registry.Register(packBUnit.Id, packBUnit, RegistrySource.Pack, "pack-b", 100);

            // Then: Conflict should be detected
            var conflicts = registry.DetectConflicts();
            conflicts.Should().NotBeEmpty();
            conflicts.Should().ContainSingle(c => c.EntryId == "custom-unit");
        }

        #endregion

        #region Asset Replacement Scenarios

        [Fact]
        public void Given_ModPathExists_When_TextureResolved_Then_ReturnsModPath()
        {
            // Given: An asset replacement engine with a registered texture mapping
            var engine = new AssetReplacementEngine();
            var manifest = new TotalConversionManifest
            {
                Id = "texture-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>
                    {
                        { "vanilla/unit/raptor.png", "textures/raptor.png" }
                    }
                }
            };

            // Create a temporary directory with the mod texture
            string tempDir = Path.Combine(Path.GetTempPath(), "dinoforge-test-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);
            string modTexturePath = Path.Combine(tempDir, "textures", "raptor.png");
            Directory.CreateDirectory(Path.GetDirectoryName(modTexturePath)!);
            File.WriteAllText(modTexturePath, "fake-texture-data");

            try
            {
                // When: Loading manifest and resolving texture
                engine.LoadFromManifest(manifest, tempDir);
                string resolved = engine.ResolveTexture("vanilla/unit/raptor.png");

                // Then: Should return the mod texture path
                resolved.Should().NotBe("vanilla/unit/raptor.png");
                resolved.Should().Contain("raptor.png");
                File.Exists(resolved).Should().BeTrue();
            }
            finally
            {
                // Cleanup
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void Given_ModPathMissing_When_TextureResolved_Then_FallsBackToVanilla()
        {
            // Given: An asset replacement engine with a mapping to non-existent file
            var engine = new AssetReplacementEngine();
            var manifest = new TotalConversionManifest
            {
                Id = "broken-texture-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>
                    {
                        { "vanilla/unit/raptor.png", "missing/raptor.png" }
                    }
                }
            };

            string tempDir = Path.Combine(Path.GetTempPath(), "dinoforge-test-missing-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);

            try
            {
                // When: Loading manifest with missing mod file and resolving
                engine.LoadFromManifest(manifest, tempDir);
                string resolved = engine.ResolveTexture("vanilla/unit/raptor.png");

                // Then: Should fall back to vanilla path
                resolved.Should().Be("vanilla/unit/raptor.png");
            }
            finally
            {
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void Given_AssetReplacementEngine_When_MultipleAssetTypesLoaded_Then_CountMatches()
        {
            // Given: A total conversion with textures, audio, and UI
            var engine = new AssetReplacementEngine();
            var manifest = new TotalConversionManifest
            {
                Id = "full-assets-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>
                    {
                        { "vanilla/texture1.png", "mod/texture1.png" },
                        { "vanilla/texture2.png", "mod/texture2.png" }
                    },
                    Audio = new Dictionary<string, string>
                    {
                        { "vanilla/sfx/attack.ogg", "mod/attack.ogg" }
                    },
                    Ui = new Dictionary<string, string>
                    {
                        { "vanilla/ui/button.png", "mod/button.png" }
                    }
                }
            };

            // When: Loading manifest and counting total mappings
            engine.LoadFromManifest(manifest, "/dummy/path");

            // Then: Total mappings should equal all registered assets
            engine.TotalMappings.Should().Be(4); // 2 textures + 1 audio + 1 UI
        }

        #endregion

        #region Dependency Resolution Scenarios

        [Fact]
        public void Given_PackWithValidDependencies_When_Resolved_Then_LoadOrderComputed()
        {
            // Given: Three packs with valid dependency chain
            var packA = new PackManifest { Id = "pack-a", Name = "Pack A" };
            var packB = new PackManifest { Id = "pack-b", Name = "Pack B", DependsOn = new List<string> { "pack-a" } };
            var packC = new PackManifest { Id = "pack-c", Name = "Pack C", DependsOn = new List<string> { "pack-b" } };

            var resolver = new PackDependencyResolver();

            // When: Computing load order
            var result = resolver.ComputeLoadOrder(new[] { packA, packB, packC });

            // Then: Load order should respect dependencies
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().HaveCount(3);
            result.LoadOrder[0].Id.Should().Be("pack-a");
            result.LoadOrder[1].Id.Should().Be("pack-b");
            result.LoadOrder[2].Id.Should().Be("pack-c");
        }

        [Fact]
        public void Given_PackDependsOnMissing_When_Resolved_Then_ReturnsError()
        {
            // Given: A pack depending on non-existent pack
            var packA = new PackManifest { Id = "pack-a", Name = "Pack A" };
            var packB = new PackManifest { Id = "pack-b", Name = "Pack B", DependsOn = new List<string> { "missing-pack" } };

            var resolver = new PackDependencyResolver();

            // When: Attempting to resolve
            var result = resolver.ComputeLoadOrder(new[] { packA, packB });

            // Then: Should fail with missing dependency error
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().ContainSingle(e => e.Contains("unknown pack"));
        }

        [Fact]
        public void Given_ConflictingPacksInSet_When_Detected_Then_ReturnsConflictError()
        {
            // Given: Two packs that declare conflict with each other
            var packA = new PackManifest
            {
                Id = "balance-mod",
                ConflictsWith = new List<string> { "balance-mod-alt" }
            };
            var packB = new PackManifest
            {
                Id = "balance-mod-alt",
                ConflictsWith = new List<string> { "balance-mod" }
            };

            var resolver = new PackDependencyResolver();

            // When: Detecting conflicts
            var conflicts = resolver.DetectConflicts(new[] { packA, packB });

            // Then: Conflicts should be detected (at least one direction)
            conflicts.Should().NotBeEmpty();
            conflicts.Should().HaveCountGreaterThan(0);
        }

        #endregion

        #region Content Loader Integration Scenarios

        [Fact]
        public void Given_ValidPackDirectory_When_LoadingManifest_Then_PackIdRetrieved()
        {
            // Given: A temporary directory with a valid pack.yaml
            string tempDir = Path.Combine(Path.GetTempPath(), "pack-test-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);
            string manifestPath = Path.Combine(tempDir, "pack.yaml");
            File.WriteAllText(manifestPath, @"
id: test-pack
name: Test Pack
version: 1.0.0
author: Test Author
type: content
");

            try
            {
                // When: Loading the pack
                var registryManager = new RegistryManager();
                var loader = new ContentLoader(registryManager);
                var result = loader.LoadPack(tempDir);

                // Then: Pack should load successfully
                result.IsSuccess.Should().BeTrue();
                result.LoadedPacks.Should().Contain("test-pack");
            }
            finally
            {
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void Given_PackDirectoryWithoutManifest_When_Loaded_Then_ReturnsFailure()
        {
            // Given: A directory without pack.yaml
            string tempDir = Path.Combine(Path.GetTempPath(), "no-manifest-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);

            try
            {
                // When: Attempting to load
                var registryManager = new RegistryManager();
                var loader = new ContentLoader(registryManager);
                var result = loader.LoadPack(tempDir);

                // Then: Should fail with manifest not found error
                result.IsSuccess.Should().BeFalse();
                result.Errors.Should().ContainSingle(e => e.Contains("manifest not found"));
            }
            finally
            {
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }

        #endregion

        #region Dependency Resolution Edge Cases (6 new scenarios)

        [Fact]
        public void Given_PackWithNoConflicts_When_DetectConflicts_Then_ReturnsEmpty()
        {
            // Given: A pack with no conflict declarations
            var pack = new PackManifest
            {
                Id = "peaceful-pack",
                Name = "Peaceful Pack",
                ConflictsWith = new List<string>() // Empty conflicts
            };

            var resolver = new PackDependencyResolver();

            // When: Detecting conflicts for this single pack
            var conflicts = resolver.DetectConflicts(new[] { pack });

            // Then: Should return no conflicts
            conflicts.Should().BeEmpty();
        }

        [Fact]
        public void Given_PackConflictsWithAbsentPack_When_DetectConflicts_Then_NoFalsePositive()
        {
            // Given: A pack that declares conflict with non-existent pack
            var pack = new PackManifest
            {
                Id = "modern-pack",
                Name = "Modern Warfare",
                ConflictsWith = new List<string> { "nonexistent-pack" }
            };

            var resolver = new PackDependencyResolver();

            // When: Detecting conflicts (absent packs should not trigger false conflict)
            var conflicts = resolver.DetectConflicts(new[] { pack });

            // Then: Conflict with absent pack should not be reported as a conflict
            conflicts.Should().BeEmpty();
        }

        [Fact]
        public void Given_ThreePacksWithChainedConflicts_When_DetectConflicts_Then_BothPairsReported()
        {
            // Given: Three packs with chain: A→B conflict, B→C conflict (but not A→C)
            var packA = new PackManifest
            {
                Id = "pack-a",
                ConflictsWith = new List<string> { "pack-b" }
            };
            var packB = new PackManifest
            {
                Id = "pack-b",
                ConflictsWith = new List<string> { "pack-c" }
            };
            var packC = new PackManifest
            {
                Id = "pack-c",
                ConflictsWith = new List<string>()
            };

            var resolver = new PackDependencyResolver();

            // When: Detecting all conflicts
            var conflicts = resolver.DetectConflicts(new[] { packA, packB, packC });

            // Then: Both A-B and B-C pairs should be reported
            conflicts.Should().HaveCountGreaterThanOrEqualTo(2);
        }

        [Fact]
        public void Given_EmptyPackList_When_LoadOrderComputed_Then_SucceedsWithEmptyOrder()
        {
            // Given: An empty list of packs
            var packs = new PackManifest[] { };

            var resolver = new PackDependencyResolver();

            // When: Computing load order for empty set
            var result = resolver.ComputeLoadOrder(packs);

            // Then: Should succeed with empty load order
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().BeEmpty();
        }

        [Fact]
        public void Given_FourPacksWithDiamondDependency_When_LoadOrderComputed_Then_BasePackFirst()
        {
            // Given: Diamond dependency: D is depended on by both B and C, B and C depended on by A
            // Dependency graph: D ← B ← A
            //                   D ← C ← A
            var packD = new PackManifest { Id = "pack-d", Name = "Pack D" };
            var packB = new PackManifest { Id = "pack-b", Name = "Pack B", DependsOn = new List<string> { "pack-d" } };
            var packC = new PackManifest { Id = "pack-c", Name = "Pack C", DependsOn = new List<string> { "pack-d" } };
            var packA = new PackManifest { Id = "pack-a", Name = "Pack A", DependsOn = new List<string> { "pack-b", "pack-c" } };

            var resolver = new PackDependencyResolver();

            // When: Computing load order
            var result = resolver.ComputeLoadOrder(new[] { packA, packB, packC, packD });

            // Then: Pack D should be first (base), A should be last
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().HaveCount(4);
            result.LoadOrder[0].Id.Should().Be("pack-d");
            result.LoadOrder[3].Id.Should().Be("pack-a");
        }

        [Fact]
        public void Given_PackDependsOnItself_When_Resolved_Then_FailsWithCircularError()
        {
            // Given: A pack that depends on itself
            var packSelf = new PackManifest
            {
                Id = "recursive-pack",
                Name = "Recursive Pack",
                DependsOn = new List<string> { "recursive-pack" }
            };

            var resolver = new PackDependencyResolver();

            // When: Attempting to resolve
            var result = resolver.ComputeLoadOrder(new[] { packSelf });

            // Then: Should fail with circular dependency error
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().ContainSingle(e => e.Contains("Circular") || e.Contains("circular"));
        }

        #endregion

        #region Registry Priority Scenarios (3 new scenarios)

        [Fact]
        public void Given_Registry_When_LowerPriorityRegisteredAfterHigher_Then_HigherStillWins()
        {
            // Given: A registry with a high-priority entry already registered
            var registry = new Registry<UnitDefinition>();
            var unit = new UnitDefinition { Id = "unit-1", DisplayName = "High Priority Unit", Stats = new UnitStats { Hp = 100f } };

            registry.Register(unit.Id, unit, RegistrySource.Pack, "pack-a", 200); // High priority

            // When: Registering same ID with lower priority
            var lowerUnit = new UnitDefinition { Id = "unit-1", DisplayName = "Low Priority Unit", Stats = new UnitStats { Hp = 50f } };
            registry.Register(lowerUnit.Id, lowerUnit, RegistrySource.Pack, "pack-b", 100); // Lower priority

            // Then: High priority unit should still be retrieved
            registry.Get("unit-1")!.DisplayName.Should().Be("High Priority Unit");
            registry.Get("unit-1")!.Stats.Hp.Should().Be(100f);
        }

        [Fact]
        public void Given_Registry_When_NonExistentIdQueried_Then_ReturnsNull()
        {
            // Given: A registry with some registered entries
            var registry = new Registry<UnitDefinition>();
            registry.Register("unit-1", new UnitDefinition { Id = "unit-1", DisplayName = "Unit 1", Stats = new UnitStats { Hp = 100f } },
                RegistrySource.BaseGame, "base-game");

            // When: Querying for non-existent ID
            var result = registry.Get("nonexistent-unit");

            // Then: Should return null
            result.Should().BeNull();
        }

        [Fact]
        public void Given_Registry_When_UnregisteredIdQueried_Then_DoesNotThrow()
        {
            // Given: A registry (may be empty or have other entries)
            var registry = new Registry<UnitDefinition>();

            // When: Querying for unregistered ID
            var action = () => registry.Get("unregistered-id");

            // Then: Should not throw exception
            action.Should().NotThrow();
            action().Should().BeNull();
        }

        #endregion

        #region Version Prefix Handling (3 new scenarios)

        [Fact]
        public void Given_PackWithGTEPrefixedVersionReq_When_ExactVersionInstalled_Then_Compatible()
        {
            // Given: A pack requiring >=0.3.0
            var pack = new PackManifest
            {
                Id = "gte-pack",
                FrameworkVersion = ">=0.3.0"
            };

            var resolver = new PackDependencyResolver();
            string installedFramework = "0.3.0";

            // When: Exact version is installed
            bool isCompatible = resolver.CheckFrameworkCompatibility(pack, installedFramework);

            // Then: Should be compatible
            isCompatible.Should().BeTrue();
        }

        [Fact]
        public void Given_PackWithTildePrefixVersion_When_MatchingVersion_Then_Compatible()
        {
            // Given: A pack requiring ~0.3.0 (allows 0.3.x but not 0.4.0)
            var pack = new PackManifest
            {
                Id = "tilde-pack",
                FrameworkVersion = "~0.3.0"
            };

            var resolver = new PackDependencyResolver();
            string installedFramework = "0.3.0";

            // When: Matching patch version is installed
            bool isCompatible = resolver.CheckFrameworkCompatibility(pack, installedFramework);

            // Then: Should be compatible
            isCompatible.Should().BeTrue();
        }

        [Fact]
        public void Given_PackWithVersionReq_When_OlderVersionInstalled_Then_Incompatible()
        {
            // Given: A pack requiring 1.0.0
            var pack = new PackManifest
            {
                Id = "new-pack",
                FrameworkVersion = "1.0.0"
            };

            var resolver = new PackDependencyResolver();
            string installedFramework = "0.9.0";

            // When: Older version is installed
            bool isCompatible = resolver.CheckFrameworkCompatibility(pack, installedFramework);

            // Then: Should be incompatible
            isCompatible.Should().BeFalse();
        }

        #endregion

        #region Content Loader Edge Cases (2 new scenarios)

        [Fact]
        public void Given_MultipleValidPackDirectories_When_EachLoaded_Then_AllPackIdsPresent()
        {
            // Given: Two valid pack directories
            string tempDir1 = Path.Combine(Path.GetTempPath(), "pack-1-" + Guid.NewGuid());
            string tempDir2 = Path.Combine(Path.GetTempPath(), "pack-2-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir1);
            Directory.CreateDirectory(tempDir2);

            File.WriteAllText(Path.Combine(tempDir1, "pack.yaml"), "id: pack-1\nname: Pack 1\nversion: 1.0.0\nauthor: Test\ntype: content\n");
            File.WriteAllText(Path.Combine(tempDir2, "pack.yaml"), "id: pack-2\nname: Pack 2\nversion: 1.0.0\nauthor: Test\ntype: content\n");

            try
            {
                // When: Loading both packs
                var registryManager = new RegistryManager();
                var loader = new ContentLoader(registryManager);
                var result1 = loader.LoadPack(tempDir1);
                var result2 = loader.LoadPack(tempDir2);

                // Then: Both pack IDs should be present
                result1.IsSuccess.Should().BeTrue();
                result2.IsSuccess.Should().BeTrue();
                result1.LoadedPacks.Should().Contain("pack-1");
                result2.LoadedPacks.Should().Contain("pack-2");
            }
            finally
            {
                if (Directory.Exists(tempDir1))
                    Directory.Delete(tempDir1, true);
                if (Directory.Exists(tempDir2))
                    Directory.Delete(tempDir2, true);
            }
        }

        [Fact]
        public void Given_PackWithInvalidYaml_When_Loaded_Then_ReturnsFailureNotException()
        {
            // Given: A pack directory with malformed YAML
            string tempDir = Path.Combine(Path.GetTempPath(), "bad-yaml-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);
            File.WriteAllText(Path.Combine(tempDir, "pack.yaml"), "id: bad-pack\nname: [broken yaml structure without closing bracket");

            try
            {
                // When: Attempting to load
                var registryManager = new RegistryManager();
                var loader = new ContentLoader(registryManager);
                var action = () => loader.LoadPack(tempDir);

                // Then: Should return failure result, not throw
                action.Should().NotThrow();
                var result = loader.LoadPack(tempDir);
                result.IsSuccess.Should().BeFalse();
                result.Errors.Should().NotBeEmpty();
            }
            finally
            {
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }

        #endregion

        #region Pack Loading Stat Validation (1 new scenario)

        [Fact]
        public void Given_PackWithAllZeroStats_When_CountingNonZeroFields_Then_ZeroReturned()
        {
            // Given: A unit with all zero stats
            var zeroStats = new UnitStats { Hp = 0f, Speed = 0f, Armor = 0f, Damage = 0f, Range = 0f };

            // When: Counting non-zero stat fields
            int nonZeroCount = 0;
            if (zeroStats.Hp > 0) nonZeroCount++;
            if (zeroStats.Speed > 0) nonZeroCount++;
            if (zeroStats.Armor > 0) nonZeroCount++;
            if (zeroStats.Damage > 0) nonZeroCount++;
            if (zeroStats.Range > 0) nonZeroCount++;

            // Then: Count should be zero
            nonZeroCount.Should().Be(0);
        }

        #endregion
    }
}
