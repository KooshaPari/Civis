using Xunit;
using DINOForge.Runtime.Bridge; // For UnitSpawnRequest and VanillaArchetypeMapper only

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for the M9 unit spawner architecture.
    /// Tests focus on mapper logic and data model validation.
    /// Full ECS integration tests will be added during M9 implementation.
    /// </summary>
    public class VanillaArchetypeMapperTests
    {
        [Theory]
        [InlineData("MilitiaLight", "Components.MeleeUnit")]
        [InlineData("CoreLineInfantry", "Components.MeleeUnit")]
        [InlineData("EliteLineInfantry", "Components.MeleeUnit")]
        [InlineData("HeavyInfantry", "Components.MeleeUnit")]
        [InlineData("ShockMelee", "Components.MeleeUnit")]
        [InlineData("SwarmFodder", "Components.MeleeUnit")]
        [InlineData("Skirmisher", "Components.RangeUnit")]
        [InlineData("AntiArmor", "Components.RangeUnit")]
        [InlineData("FastVehicle", "Components.CavalryUnit")]
        [InlineData("MainBattleVehicle", "Components.SiegeUnit")]
        [InlineData("HeavySiege", "Components.SiegeUnit")]
        [InlineData("Artillery", "Components.SiegeUnit")]
        [InlineData("WalkerHeavy", "Components.SiegeUnit")]
        [InlineData("Archer", "Components.Archer")]
        [InlineData("StaticMG", "Components.MeleeUnit")]
        [InlineData("StaticAT", "Components.MeleeUnit")]
        [InlineData("StaticArtillery", "Components.SiegeUnit")]
        [InlineData("SupportEngineer", "Components.MeleeUnit")]
        [InlineData("Recon", "Components.RangeUnit")]
        [InlineData("HeroCommander", "Components.MeleeUnit")]
        [InlineData("AirstrikeProxy", "Components.MeleeUnit")]
        [InlineData("ShieldedElite", "Components.MeleeUnit")]
        public void MapUnitClassToComponentType_ValidClasses_ReturnCorrectComponentType(
            string unitClass, string expectedComponentType)
        {
            var result = VanillaArchetypeMapper.MapUnitClassToComponentType(unitClass);

            Assert.Equal(expectedComponentType, result);
        }

        [Theory]
        [InlineData("MilitiaLight")]
        [InlineData("CoreLineInfantry")]
        [InlineData("Skirmisher")]
        [InlineData("Artillery")]
        [InlineData("Archer")]
        [InlineData("HeroCommander")]
        public void IsSpawnable_ValidClasses_ReturnsTrue(string unitClass)
        {
            var result = VanillaArchetypeMapper.IsSpawnable(unitClass);

            Assert.True(result);
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        [InlineData("InvalidClass")]
        [InlineData("MagicMissile")]
        [InlineData("RandomUnit")]
        public void IsSpawnable_InvalidClasses_ReturnsFalse(string unitClass)
        {
            var result = VanillaArchetypeMapper.IsSpawnable(unitClass);

            Assert.False(result);
        }

        [Fact]
        public void IsSpawnable_NullClass_ReturnsFalse()
        {
            var result = VanillaArchetypeMapper.IsSpawnable(null!);

            Assert.False(result);
        }

        [Theory]
        [InlineData("MilitiaLight")]
        [InlineData("CoreLineInfantry")]
        [InlineData("Artillery")]
        [InlineData("Archer")]
        public void ValidateUnitClass_ValidClasses_ReturnsTrue(string unitClass)
        {
            var result = VanillaArchetypeMapper.ValidateUnitClass(unitClass, out string? error);

            Assert.True(result);
            Assert.Null(error);
        }

        [Theory]
        [InlineData("", "Unit class is empty or null")]
        [InlineData("   ", "Unit class is empty or null")]
        public void ValidateUnitClass_EmptyOrWhitespace_ReturnsFalseWithMessage(string unitClass, string expectedErrorStart)
        {
            var result = VanillaArchetypeMapper.ValidateUnitClass(unitClass, out string? error);

            Assert.False(result);
            Assert.NotNull(error);
            Assert.Contains(expectedErrorStart, error);
        }

        [Fact]
        public void ValidateUnitClass_Null_ReturnsFalseWithMessage()
        {
            var result = VanillaArchetypeMapper.ValidateUnitClass(null!, out string? error);

            Assert.False(result);
            Assert.NotNull(error);
            Assert.Contains("Unit class is empty or null", error);
        }

        [Fact]
        public void ValidateUnitClass_UnknownClass_ReturnsFalseWithInformativeMessage()
        {
            var result = VanillaArchetypeMapper.ValidateUnitClass("InvalidClass", out string? error);

            Assert.False(result);
            Assert.NotNull(error);
            Assert.Contains("Unknown unit class 'InvalidClass'", error);
            Assert.Contains("Valid classes:", error);
        }

        [Fact]
        public void MapUnitClassToComponentType_CaseInsensitive_MapsCorrectly()
        {
            // The mapping should use case-insensitive comparison
            var result1 = VanillaArchetypeMapper.MapUnitClassToComponentType("MILITIALIGHT");
            var result2 = VanillaArchetypeMapper.MapUnitClassToComponentType("militialight");
            var result3 = VanillaArchetypeMapper.MapUnitClassToComponentType("MilitiaLight");

            Assert.Equal("Components.MeleeUnit", result1);
            Assert.Equal("Components.MeleeUnit", result2);
            Assert.Equal("Components.MeleeUnit", result3);
        }

        [Fact]
        public void MapUnitClassToComponentType_NullOrEmpty_ReturnsNull()
        {
            Assert.Null(VanillaArchetypeMapper.MapUnitClassToComponentType(null!));
            Assert.Null(VanillaArchetypeMapper.MapUnitClassToComponentType(""));
            Assert.Null(VanillaArchetypeMapper.MapUnitClassToComponentType("   "));
        }
    }

    /// <summary>
    /// Unit tests for UnitSpawnRequest data structure.
    /// </summary>
    public class UnitSpawnRequestTests
    {
        [Fact]
        public void Constructor_WithAllParameters_InitializesCorrectly()
        {
            var request = new UnitSpawnRequest("modern:m1_abrams", 10.5f, 20.3f, isEnemy: true);

            Assert.Equal("modern:m1_abrams", request.UnitDefinitionId);
            Assert.Equal(10.5f, request.X);
            Assert.Equal(20.3f, request.Z);
            Assert.True(request.IsEnemy);
        }

        [Fact]
        public void Constructor_DefaultFaction_InitializesToPlayerFaction()
        {
            var request = new UnitSpawnRequest("modern:m1_abrams", 10f, 20f);

            Assert.False(request.IsEnemy);
        }

        [Fact]
        public void ToString_ReturnsFormattedString()
        {
            var request = new UnitSpawnRequest("pack:unit", 5f, 15f, isEnemy: false);
            var result = request.ToString();

            Assert.Contains("UnitSpawnRequest", result);
            Assert.Contains("pack:unit", result);
            Assert.Contains("player", result);
        }

        [Fact]
        public void ToString_EnemyFaction_IncludesEnemyLabel()
        {
            var request = new UnitSpawnRequest("pack:unit", 5f, 15f, isEnemy: true);
            var result = request.ToString();

            Assert.Contains("enemy", result);
        }

        [Fact]
        public void Equality_SameValues_ReturnsEqual()
        {
            var request1 = new UnitSpawnRequest("pack:unit", 10f, 20f, isEnemy: true);
            var request2 = new UnitSpawnRequest("pack:unit", 10f, 20f, isEnemy: true);

            // Structs use value equality by default
            Assert.Equal(request1, request2);
        }

        [Fact]
        public void Equality_DifferentValues_ReturnsNotEqual()
        {
            var request1 = new UnitSpawnRequest("pack:unit", 10f, 20f, isEnemy: true);
            var request2 = new UnitSpawnRequest("pack:unit", 10f, 20f, isEnemy: false);

            Assert.NotEqual(request1, request2);
        }
    }

    /// <summary>
    /// Unit tests for PackUnitSpawner spawn request enqueuing.
    /// Tests focus on the UnitSpawnRequest data model and request creation logic.
    /// Full ECS integration tests (system lifecycle, entity creation) deferred to integration layer.
    ///
    /// NOTE: PackUnitSpawner class itself cannot be unit-tested here because it inherits from
    /// SystemBase (Unity.Entities), which is not available in the unit test project. Tests for
    /// the static RequestSpawnStatic method and the spawner lifecycle belong in Integration tests.
    /// </summary>
    public class PackUnitSpawnerTests
    {
        /// <summary>
        /// Tests that UnitSpawnRequest struct initializes correctly with all parameters.
        /// This validates the data model that is enqueued by the spawning system.
        /// </summary>
        [Fact]
        public void RequestSpawn_ValidUnit_EnqueuesCorrectStruct()
        {
            // The UnitSpawnRequest struct is the contract for spawn queue entries.
            // This test validates that the struct correctly captures spawn intent.

            // Arrange
            const string unitId = "modern:m1_abrams";
            const float x = 15.5f;
            const float z = 25.3f;
            const bool isEnemy = true;

            // Act: Create request struct
            var request = new UnitSpawnRequest(unitId, x, z, isEnemy);

            // Assert: All fields captured correctly
            Assert.Equal(unitId, request.UnitDefinitionId);
            Assert.Equal(x, request.X);
            Assert.Equal(z, request.Z);
            Assert.Equal(isEnemy, request.IsEnemy);
        }

        /// <summary>
        /// Tests that UnitSpawnRequest struct defaults faction to player (isEnemy = false).
        /// Validates that the default constructor provides sensible defaults.
        /// </summary>
        [Fact]
        public void RequestSpawn_DefaultFaction_IsPlayer()
        {
            // Arrange
            const string unitId = "test:unit";
            const float x = 10f;
            const float z = 20f;

            // Act: Create request with default faction (omit isEnemy)
            var request = new UnitSpawnRequest(unitId, x, z);

            // Assert: Faction defaults to player
            Assert.False(request.IsEnemy);
        }

        /// <summary>
        /// Tests that UnitSpawnRequest struct can represent enemy-faction units.
        /// Validates that the isEnemy flag is properly captured.
        /// </summary>
        [Fact]
        public void RequestSpawn_EnemyFaction_IsEnemy()
        {
            // Arrange
            const string unitId = "test:unit";
            const float x = 10f;
            const float z = 20f;

            // Act: Create request with enemy faction
            var request = new UnitSpawnRequest(unitId, x, z, isEnemy: true);

            // Assert: Faction is enemy
            Assert.True(request.IsEnemy);
        }

        /// <summary>
        /// Tests that UnitSpawnRequest struct can represent units at any world position.
        /// Validates that X and Z coordinates are preserved without loss of precision.
        /// </summary>
        [Theory]
        [InlineData(0f, 0f)]
        [InlineData(100.5f, 200.7f)]
        [InlineData(-50.3f, -75.9f)]
        [InlineData(float.MaxValue, float.MinValue)]
        public void RequestSpawn_ArbitraryPosition_PreservesCoordinates(float x, float z)
        {
            // Arrange
            const string unitId = "test:unit";

            // Act: Create request at arbitrary position
            var request = new UnitSpawnRequest(unitId, x, z);

            // Assert: Coordinates preserved exactly
            Assert.Equal(x, request.X);
            Assert.Equal(z, request.Z);
        }

        /// <summary>
        /// NOTE: The following tests are deferred to integration test layer:
        /// - RequestSpawnStatic: Requires access to PackUnitSpawner (SystemBase subclass)
        /// - OnUpdate_ProcessesSpawnQueue_CreatesEntities: Requires mocked ECS World and EntityManager
        /// - CanSpawn_ValidUnitDefinition_ReturnsTrue: Requires mocked RegistryManager and unit lookups
        /// - RequestSpawn_UnknownUnitClass_SkipsSpawn: Requires full ECS infrastructure
        ///
        /// These tests validate ECS system lifecycle which cannot be unit-tested here because:
        /// 1. PackUnitSpawner inherits from SystemBase (Unity.Entities), not available in unit tests
        /// 2. RequestSpawnStatic is a static method on PackUnitSpawner class
        /// 3. OnUpdate and CanSpawn depend on SystemBase.EntityManager and entity world state
        /// 4. Full spawn lifecycle requires ECS World, entity queries, and instantiation
        ///
        /// These belong in src/Tests/Integration/ with full ECS fixtures and mocked systems.
        /// </summary>
    }
}
